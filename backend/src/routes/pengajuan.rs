//! Pengajuan pembatalan, retur, dan tukar ukuran dengan persetujuan pemilik (KF-10).
//!
//! Kasir mengajukan dari sesinya yang sedang dibuka; pemilik menyetujui atau
//! menolak (biasanya dari ponsel). Saat disetujui, stok dikembalikan dan nilai
//! pengembalian dibukukan pada sesi kasir pengaju. Pengajuan oleh admin langsung
//! disetujui melalui jalur yang sama.

use std::collections::{HashMap, HashSet};

use axum::{Json, Router, extract::{Path, Query, State}, routing::{get, post}};
use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::{Admin, PenggunaLogin},
    models::{
        pengajuan::{
            BarisFaktur, BuatPengajuan, CariPengajuan, FakturPengajuan, ItemPengajuan, JenisPengajuan, Pengajuan,
            TolakPengajuan, VarianPengganti,
        },
        wajib,
    },
    services::{
        hitung,
        log::{self, Catatan},
        pengaturan,
        stok::{self, Mutasi},
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pengajuan", get(daftar).post(buat))
        .route("/pengajuan/{id}", get(detail))
        .route("/pengajuan/{id}/setujui", post(setujui))
        .route("/pengajuan/{id}/tolak", post(tolak))
        .route("/pengajuan/{id}/tarik", post(tarik))
        .route("/transaksi/faktur/{no_faktur}", get(cari_faktur))
}

fn lewat_batas(tanggal: DateTime<Utc>, batas_hari: i64) -> bool {
    Utc::now() - tanggal > Duration::days(batas_hari)
}

/// Baris transaksi beserta unit yang sudah terpakai oleh pengajuan aktif
/// (menunggu atau disetujui) dan nilai yang dibayar setelah diskon transaksi.
struct BarisTransaksi {
    id_detail: i64,
    id_varian: i64,
    id_produk: i64,
    nama_barang: String,
    jumlah: i32,
    subtotal: i64,
    terpakai: i32,
    dibayar: i64,
}

async fn baris_transaksi(conn: &mut PgConnection, id_transaksi: i64, diskon_transaksi: i64) -> AppResult<Vec<BarisTransaksi>> {
    let baris = sqlx::query!(
        r#"SELECT d.id_detail, d.id_varian, v.id_produk, d.nama_barang, d.jumlah, d.subtotal,
                  coalesce((SELECT sum(dr.jumlah) FROM detail_retur dr JOIN retur r ON r.id_retur = dr.id_retur
                            WHERE dr.id_detail = d.id_detail AND r.status IN ('menunggu', 'disetujui')), 0)::int AS "terpakai!"
           FROM detail_transaksi d JOIN varian v ON v.id_varian = d.id_varian
           WHERE d.id_transaksi = $1
           ORDER BY d.id_detail"#,
        id_transaksi
    )
    .fetch_all(&mut *conn)
    .await?;

    let subtotal: Vec<i64> = baris.iter().map(|b| b.subtotal).collect();
    let bagian = hitung::bagi_diskon_transaksi(&subtotal, diskon_transaksi);
    Ok(baris
        .into_iter()
        .zip(bagian)
        .map(|(b, potongan)| BarisTransaksi {
            id_detail: b.id_detail,
            id_varian: b.id_varian,
            id_produk: b.id_produk,
            nama_barang: b.nama_barang,
            jumlah: b.jumlah,
            subtotal: b.subtotal,
            terpakai: b.terpakai,
            dibayar: b.subtotal - potongan,
        })
        .collect())
}

async fn id_sesi_aktif(conn: &mut PgConnection, id_pengguna: i64) -> AppResult<i64> {
    sqlx::query_scalar!(
        "SELECT id_sesi FROM sesi_kasir WHERE id_pengguna = $1 AND waktu_tutup IS NULL FOR UPDATE",
        id_pengguna
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| AppError::konflik("Buka sesi kasir terlebih dahulu sebelum mengajukan pembatalan atau retur"))
}

struct Filter {
    id_retur: Option<i64>,
    id_pengguna: Option<i64>,
    status: Option<String>,
}

async fn muat(conn: &mut PgConnection, f: Filter) -> AppResult<Vec<Pengajuan>> {
    let batas_hari = pengaturan::angka(&mut *conn, pengaturan::BATAS_HARI_RETUR, 3).await?;
    let header = sqlx::query!(
        r#"SELECT r.id_retur, r.id_transaksi, t.no_faktur, t.tanggal_waktu AS tanggal_transaksi, r.jenis, r.status,
                  r.alasan, r.alasan_tolak, r.total_pengembalian, r.metode_pengembalian, r.id_sesi, r.id_pengguna,
                  p.nama AS nama_pengaju, o.nama AS "nama_otorisator?", r.tanggal_waktu, r.waktu_keputusan
           FROM retur r
           JOIN transaksi t ON t.id_transaksi = r.id_transaksi
           JOIN pengguna p ON p.id_pengguna = r.id_pengguna
           LEFT JOIN pengguna o ON o.id_pengguna = r.id_otorisator
           WHERE ($1::bigint IS NULL OR r.id_retur = $1)
             AND ($2::bigint IS NULL OR r.id_pengguna = $2)
             AND ($3::text IS NULL OR r.status = $3)
           ORDER BY r.tanggal_waktu DESC
           LIMIT 200"#,
        f.id_retur,
        f.id_pengguna,
        f.status,
    )
    .fetch_all(&mut *conn)
    .await?;

    let ids: Vec<i64> = header.iter().map(|h| h.id_retur).collect();
    let items = sqlx::query!(
        r#"SELECT dr.id_detail_retur, dr.id_retur, dr.id_detail, d.id_varian, d.nama_barang, dr.jumlah, dr.nilai,
                  dr.id_varian_pengganti,
                  CASE WHEN vp.id_varian IS NULL THEN NULL ELSE pp.nama || ' ' || vp.ukuran || '/' || vp.warna END AS "nama_pengganti?"
           FROM detail_retur dr
           JOIN detail_transaksi d ON d.id_detail = dr.id_detail
           LEFT JOIN varian vp ON vp.id_varian = dr.id_varian_pengganti
           LEFT JOIN produk pp ON pp.id_produk = vp.id_produk
           WHERE dr.id_retur = ANY($1)
           ORDER BY dr.id_detail_retur"#,
        &ids
    )
    .fetch_all(&mut *conn)
    .await?;

    let mut per_retur: HashMap<i64, Vec<ItemPengajuan>> = HashMap::new();
    for i in items {
        per_retur.entry(i.id_retur).or_default().push(ItemPengajuan {
            id_detail_retur: i.id_detail_retur,
            id_detail: i.id_detail,
            id_varian: i.id_varian,
            nama_barang: i.nama_barang,
            jumlah: i.jumlah,
            nilai: i.nilai,
            id_varian_pengganti: i.id_varian_pengganti,
            nama_pengganti: i.nama_pengganti,
        });
    }

    Ok(header
        .into_iter()
        .map(|h| Pengajuan {
            items: per_retur.remove(&h.id_retur).unwrap_or_default(),
            lewat_batas_hari: lewat_batas(h.tanggal_transaksi, batas_hari),
            id_retur: h.id_retur,
            id_transaksi: h.id_transaksi,
            no_faktur: h.no_faktur,
            tanggal_transaksi: h.tanggal_transaksi,
            jenis: h.jenis,
            status: h.status,
            alasan: h.alasan,
            alasan_tolak: h.alasan_tolak,
            total_pengembalian: h.total_pengembalian,
            metode_pengembalian: h.metode_pengembalian,
            id_sesi: h.id_sesi,
            id_pengguna: h.id_pengguna,
            nama_pengaju: h.nama_pengaju,
            nama_otorisator: h.nama_otorisator,
            tanggal_waktu: h.tanggal_waktu,
            waktu_keputusan: h.waktu_keputusan,
        })
        .collect())
}

async fn muat_satu(conn: &mut PgConnection, id_retur: i64) -> AppResult<Pengajuan> {
    muat(conn, Filter { id_retur: Some(id_retur), id_pengguna: None, status: None })
        .await?
        .pop()
        .ok_or_else(|| AppError::tidak_ditemukan("Pengajuan tidak ditemukan"))
}

/// Pencarian faktur tepat (nomor lengkap dari struk pelanggan). Kasir boleh
/// mencari transaksi kasir lain di sini agar retur dapat dilayani siapa pun.
async fn cari_faktur(pengguna: PenggunaLogin, State(state): State<AppState>, Path(no_faktur): Path<String>) -> AppResult<Json<FakturPengajuan>> {
    let mut conn = state.pool.acquire().await?;
    let t = sqlx::query!(
        r#"SELECT t.id_transaksi, t.no_faktur, t.tanggal_waktu, t.id_sesi, t.status, t.metode_bayar, t.subtotal, t.diskon,
                  t.total, p.nama AS nama_kasir
           FROM transaksi t JOIN pengguna p ON p.id_pengguna = t.id_pengguna
           WHERE upper(t.no_faktur) = upper($1)"#,
        no_faktur.trim()
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan(format!("Faktur \"{}\" tidak ditemukan. Periksa nomor pada struk.", no_faktur.trim())))?;

    let baris = baris_transaksi(&mut conn, t.id_transaksi, t.diskon).await?;
    let id_produk: Vec<i64> = baris.iter().map(|b| b.id_produk).collect();
    let varian = sqlx::query!(
        r#"SELECT v.id_varian, v.id_produk, v.ukuran, v.warna, v.stok
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk
           WHERE v.id_produk = ANY($1) AND v.aktif AND p.aktif
           ORDER BY array_position(ARRAY['XS','S','M','L','XL','XXL','XXXL'], upper(v.ukuran)) NULLS LAST, v.ukuran, v.warna"#,
        &id_produk
    )
    .fetch_all(&mut *conn)
    .await?;

    let sesi_aktif = sqlx::query_scalar!(
        "SELECT id_sesi FROM sesi_kasir WHERE id_pengguna = $1 AND waktu_tutup IS NULL",
        pengguna.id_pengguna
    )
    .fetch_optional(&mut *conn)
    .await?;
    let batas_hari = pengaturan::angka(&mut *conn, pengaturan::BATAS_HARI_RETUR, 3).await?;

    Ok(Json(FakturPengajuan {
        bisa_dibatalkan: t.status == "selesai" && sesi_aktif == Some(t.id_sesi) && baris.iter().all(|b| b.terpakai == 0),
        lewat_batas_hari: lewat_batas(t.tanggal_waktu, batas_hari),
        items: baris
            .iter()
            .map(|b| BarisFaktur {
                id_detail: b.id_detail,
                id_varian: b.id_varian,
                nama_barang: b.nama_barang.clone(),
                jumlah: b.jumlah,
                subtotal: b.subtotal,
                dibayar: b.dibayar,
                sisa: b.jumlah - b.terpakai,
                pengganti: varian
                    .iter()
                    .filter(|v| v.id_produk == b.id_produk && v.id_varian != b.id_varian)
                    .map(|v| VarianPengganti { id_varian: v.id_varian, ukuran: v.ukuran.clone(), warna: v.warna.clone(), stok: v.stok })
                    .collect(),
            })
            .collect(),
        id_transaksi: t.id_transaksi,
        no_faktur: t.no_faktur,
        tanggal_waktu: t.tanggal_waktu,
        nama_kasir: t.nama_kasir,
        status: t.status,
        metode_bayar: t.metode_bayar,
        subtotal: t.subtotal,
        diskon: t.diskon,
        total: t.total,
    }))
}

struct BarisBaru {
    id_detail: i64,
    jumlah: i32,
    nilai: i64,
    id_varian_pengganti: Option<i64>,
}

async fn buat(pengguna: PenggunaLogin, State(state): State<AppState>, Json(req): Json<BuatPengajuan>) -> AppResult<Json<Pengajuan>> {
    let alasan = wajib(&req.alasan, "Alasan pengajuan")?;
    let mut tx = state.pool.begin().await?;

    // Urutan kunci: sesi pengaju lalu transaksi (sama seperti penyimpanan transaksi).
    let id_sesi = id_sesi_aktif(&mut tx, pengguna.id_pengguna).await?;
    let t = sqlx::query!(
        "SELECT id_transaksi, id_sesi, status, diskon, total, metode_bayar, no_faktur FROM transaksi WHERE id_transaksi = $1 FOR UPDATE",
        req.id_transaksi
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan("Transaksi tidak ditemukan"))?;
    if t.status != "selesai" {
        return Err(AppError::validasi("Transaksi ini sudah dibatalkan"));
    }

    let baris = baris_transaksi(&mut tx, t.id_transaksi, t.diskon).await?;
    let mut baru: Vec<BarisBaru> = Vec::new();

    match req.jenis {
        JenisPengajuan::Batal => {
            if t.id_sesi != id_sesi {
                return Err(AppError::validasi(
                    "Pembatalan hanya untuk transaksi pada sesi kasir Anda yang sedang dibuka. Gunakan retur untuk transaksi lain.",
                ));
            }
            if baris.iter().any(|b| b.terpakai > 0) {
                return Err(AppError::validasi("Transaksi ini sudah memiliki pengajuan retur atau tukar, sehingga tidak dapat dibatalkan"));
            }
            baru.extend(baris.iter().map(|b| BarisBaru { id_detail: b.id_detail, jumlah: b.jumlah, nilai: b.dibayar, id_varian_pengganti: None }));
        }
        JenisPengajuan::Retur | JenisPengajuan::Tukar => {
            if req.items.is_empty() {
                return Err(AppError::validasi("Pilih minimal satu barang"));
            }
            let mut dipilih = HashSet::new();
            for item in &req.items {
                let b = baris
                    .iter()
                    .find(|b| b.id_detail == item.id_detail)
                    .ok_or_else(|| AppError::validasi("Barang tidak termasuk dalam transaksi ini"))?;
                if !dipilih.insert(item.id_detail) {
                    return Err(AppError::validasi("Barang yang sama dipilih lebih dari sekali"));
                }
                let sisa = b.jumlah - b.terpakai;
                if item.jumlah <= 0 || item.jumlah > sisa {
                    return Err(AppError::validasi(format!("Jumlah {} melebihi sisa yang dapat diretur ({sisa})", b.nama_barang)));
                }

                let pengganti = if req.jenis == JenisPengajuan::Tukar {
                    let id = item
                        .id_varian_pengganti
                        .ok_or_else(|| AppError::validasi(format!("Pilih ukuran/warna pengganti untuk {}", b.nama_barang)))?;
                    let sah = sqlx::query_scalar!(
                        r#"SELECT EXISTS (SELECT 1 FROM varian v JOIN produk p ON p.id_produk = v.id_produk
                                          WHERE v.id_varian = $1 AND v.id_produk = $2 AND v.aktif AND p.aktif) AS "sah!""#,
                        id,
                        b.id_produk
                    )
                    .fetch_one(&mut *tx)
                    .await?;
                    if !sah || id == b.id_varian {
                        return Err(AppError::validasi("Tukar ukuran hanya ke ukuran/warna lain dari produk yang sama"));
                    }
                    Some(id)
                } else {
                    None
                };

                let nilai = match req.jenis {
                    JenisPengajuan::Tukar => 0,
                    _ => hitung::nilai_retur(b.dibayar, b.jumlah, b.terpakai, item.jumlah),
                };
                baru.push(BarisBaru { id_detail: b.id_detail, jumlah: item.jumlah, nilai, id_varian_pengganti: pengganti });
            }
        }
    }

    let total: i64 = baru.iter().map(|b| b.nilai).sum();
    let metode = match req.jenis {
        JenisPengajuan::Tukar => None,
        _ => Some(req.metode_pengembalian.map(|m| m.as_str().to_string()).unwrap_or(t.metode_bayar.clone())),
    };

    let id_retur = sqlx::query_scalar!(
        r#"INSERT INTO retur (id_transaksi, jenis, alasan, total_pengembalian, metode_pengembalian, id_sesi, id_pengguna)
           VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id_retur"#,
        t.id_transaksi,
        req.jenis.as_str(),
        alasan,
        total,
        metode,
        id_sesi,
        pengguna.id_pengguna,
    )
    .fetch_one(&mut *tx)
    .await?;
    for b in &baru {
        sqlx::query!(
            "INSERT INTO detail_retur (id_retur, id_detail, jumlah, nilai, id_varian_pengganti) VALUES ($1, $2, $3, $4, $5)",
            id_retur,
            b.id_detail,
            b.jumlah,
            b.nilai,
            b.id_varian_pengganti,
        )
        .execute(&mut *tx)
        .await?;
    }
    log::catat(
        &mut *tx,
        Catatan {
            id_pengguna: Some(pengguna.id_pengguna),
            aksi: match req.jenis {
                JenisPengajuan::Batal => "ajukan_batal",
                JenisPengajuan::Retur => "ajukan_retur",
                JenisPengajuan::Tukar => "ajukan_tukar",
            },
            objek: "pengajuan",
            id_objek: Some(id_retur.to_string()),
            nilai_baru: Some(json!({
                "no_faktur": t.no_faktur,
                "total_pengembalian": total,
                "metode_pengembalian": metode,
                "alasan": alasan,
                "jumlah_barang": baru.iter().map(|b| b.jumlah).sum::<i32>(),
            })),
            ..Default::default()
        },
    )
    .await?;

    // Pemilik/admin tidak perlu menunggu persetujuan dirinya sendiri.
    if pengguna.is_admin() {
        terapkan_persetujuan(&mut tx, id_retur, pengguna.id_pengguna).await?;
    }

    let hasil = muat_satu(&mut tx, id_retur).await?;
    tx.commit().await?;
    Ok(Json(hasil))
}

/// Mengunci pengajuan yang masih menunggu; 409 bila sudah diputuskan.
async fn kunci_menunggu(conn: &mut PgConnection, id_retur: i64) -> AppResult<(i64, String)> {
    let r = sqlx::query!("SELECT id_pengguna, status FROM retur WHERE id_retur = $1 FOR UPDATE", id_retur)
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| AppError::tidak_ditemukan("Pengajuan tidak ditemukan"))?;
    if r.status != "menunggu" {
        return Err(AppError::konflik(format!("Pengajuan ini sudah {}", r.status)));
    }
    Ok((r.id_pengguna, r.status))
}

/// Menjalankan pengajuan yang disetujui: stok kembali (dan keluar untuk barang
/// pengganti), transaksi dibatalkan bila jenisnya batal, lalu status disetujui.
/// Baris pengajuan harus sudah terkunci oleh pemanggil.
async fn terapkan_persetujuan(conn: &mut PgConnection, id_retur: i64, id_otorisator: i64) -> AppResult<()> {
    let r = sqlx::query!(
        r#"SELECT r.jenis, r.id_transaksi, r.total_pengembalian, t.no_faktur
           FROM retur r JOIN transaksi t ON t.id_transaksi = r.id_transaksi
           WHERE r.id_retur = $1"#,
        id_retur
    )
    .fetch_one(&mut *conn)
    .await?;

    let status_transaksi = sqlx::query_scalar!("SELECT status FROM transaksi WHERE id_transaksi = $1 FOR UPDATE", r.id_transaksi)
        .fetch_one(&mut *conn)
        .await?;
    if status_transaksi != "selesai" {
        return Err(AppError::konflik("Transaksi asal sudah dibatalkan"));
    }

    let baris = sqlx::query!(
        r#"SELECT d.id_varian, dr.jumlah, dr.id_varian_pengganti,
                  (vp.id_varian IS NULL OR (vp.aktif AND pp.aktif)) AS "pengganti_aktif!"
           FROM detail_retur dr
           JOIN detail_transaksi d ON d.id_detail = dr.id_detail
           LEFT JOIN varian vp ON vp.id_varian = dr.id_varian_pengganti
           LEFT JOIN produk pp ON pp.id_produk = vp.id_produk
           WHERE dr.id_retur = $1"#,
        id_retur
    )
    .fetch_all(&mut *conn)
    .await?;

    // Kumpulkan seluruh mutasi lalu urutkan per varian agar urutan kunci konsisten.
    let mut mutasi: Vec<(i64, i32, &'static str)> = Vec::new();
    for b in &baris {
        if !b.pengganti_aktif {
            return Err(AppError::konflik("Ukuran/warna pengganti sudah tidak dijual. Tolak pengajuan ini."));
        }
        mutasi.push((b.id_varian, b.jumlah, "retur"));
        if let Some(pengganti) = b.id_varian_pengganti {
            mutasi.push((pengganti, -b.jumlah, "keluar"));
        }
    }
    mutasi.sort_by_key(|m| m.0);

    let referensi = format!("RT-{id_retur}");
    let keterangan = format!("{} {}", r.jenis, r.no_faktur);
    for (id_varian, jumlah, jenis) in mutasi {
        stok::terapkan(
            conn,
            Mutasi { id_varian, jenis, jumlah, referensi: Some(&referensi), id_pengguna: id_otorisator, keterangan: Some(&keterangan) },
        )
        .await?;
    }

    if r.jenis == "batal" {
        sqlx::query!("UPDATE transaksi SET status = 'batal' WHERE id_transaksi = $1", r.id_transaksi)
            .execute(&mut *conn)
            .await?;
    }
    sqlx::query!(
        "UPDATE retur SET status = 'disetujui', id_otorisator = $2, waktu_keputusan = now() WHERE id_retur = $1",
        id_retur,
        id_otorisator
    )
    .execute(&mut *conn)
    .await?;

    log::catat(
        &mut *conn,
        Catatan {
            id_pengguna: Some(id_otorisator),
            aksi: match r.jenis.as_str() {
                "batal" => "setujui_batal",
                "tukar" => "setujui_tukar",
                _ => "setujui_retur",
            },
            objek: "pengajuan",
            id_objek: Some(id_retur.to_string()),
            nilai_lama: Some(json!({ "status": "menunggu" })),
            nilai_baru: Some(json!({ "status": "disetujui", "no_faktur": r.no_faktur, "total_pengembalian": r.total_pengembalian })),
            keterangan: None,
        },
    )
    .await?;
    Ok(())
}

async fn setujui(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Pengajuan>> {
    let mut tx = state.pool.begin().await?;
    kunci_menunggu(&mut tx, id).await?;
    terapkan_persetujuan(&mut tx, id, admin.id_pengguna).await?;
    let hasil = muat_satu(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(hasil))
}

async fn tolak(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>, Json(req): Json<TolakPengajuan>) -> AppResult<Json<Pengajuan>> {
    let alasan = wajib(&req.alasan, "Alasan penolakan")?;
    let mut tx = state.pool.begin().await?;
    kunci_menunggu(&mut tx, id).await?;
    sqlx::query!(
        "UPDATE retur SET status = 'ditolak', id_otorisator = $2, waktu_keputusan = now(), alasan_tolak = $3 WHERE id_retur = $1",
        id,
        admin.id_pengguna,
        alasan
    )
    .execute(&mut *tx)
    .await?;
    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, "tolak_pengajuan", "pengajuan")
            .id_objek(id)
            .lama(json!({ "status": "menunggu" }))
            .baru(json!({ "status": "ditolak", "alasan_tolak": alasan })),
    )
    .await?;
    let hasil = muat_satu(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(hasil))
}

/// Kasir menarik pengajuannya sendiri yang belum diputuskan, misalnya agar sesi dapat ditutup.
async fn tarik(pengguna: PenggunaLogin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Pengajuan>> {
    let mut tx = state.pool.begin().await?;
    let (pengaju, _) = kunci_menunggu(&mut tx, id).await?;
    if pengaju != pengguna.id_pengguna {
        return Err(AppError::TidakBerhak);
    }
    sqlx::query!("UPDATE retur SET status = 'dibatalkan', waktu_keputusan = now() WHERE id_retur = $1", id)
        .execute(&mut *tx)
        .await?;
    log::catat(
        &mut *tx,
        Catatan::new(pengguna.id_pengguna, "tarik_pengajuan", "pengajuan")
            .id_objek(id)
            .lama(json!({ "status": "menunggu" }))
            .baru(json!({ "status": "dibatalkan" })),
    )
    .await?;
    let hasil = muat_satu(&mut tx, id).await?;
    tx.commit().await?;
    Ok(Json(hasil))
}

async fn daftar(pengguna: PenggunaLogin, State(state): State<AppState>, Query(q): Query<CariPengajuan>) -> AppResult<Json<Vec<Pengajuan>>> {
    let mut conn = state.pool.acquire().await?;
    let filter = Filter {
        id_retur: None,
        id_pengguna: (!pengguna.is_admin()).then_some(pengguna.id_pengguna),
        status: q.status.filter(|s| !s.trim().is_empty()),
    };
    Ok(Json(muat(&mut conn, filter).await?))
}

async fn detail(pengguna: PenggunaLogin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Pengajuan>> {
    let mut conn = state.pool.acquire().await?;
    let p = muat_satu(&mut conn, id).await?;
    if !pengguna.is_admin() && p.id_pengguna != pengguna.id_pengguna {
        return Err(AppError::TidakBerhak);
    }
    Ok(Json(p))
}
