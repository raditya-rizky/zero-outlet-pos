//! Stok opname (KF-12), kartu stok (KF-13), dan peringatan stok (KF-14).

use std::collections::HashSet;

use axum::{Json, Router, extract::{Path, Query, State}, routing::{get, post}};
use serde_json::json;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::Admin,
    models::{
        stok::{CariRentang, HasilOpname, KartuStok, Lencana, MutasiKartu, PeringatanStok, SelisihOpname, StokOpname, VarianStok},
        wajib,
    },
    services::{
        log::{self, Catatan},
        pengaturan,
        stok::{self, Mutasi},
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/stok/opname", post(opname))
        .route("/stok/peringatan", get(peringatan))
        .route("/varian/{id}/kartu-stok", get(kartu_stok))
        .route("/ringkasan/lencana", get(lencana))
}

/// Menyesuaikan stok sistem dengan hasil hitung fisik. Alasan wajib (UJ-08);
/// ringkasan akurasi menjadi data uji akurasi stok pada Tabel 1.
async fn opname(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<StokOpname>) -> AppResult<Json<HasilOpname>> {
    let alasan = wajib(&req.alasan, "Alasan penyesuaian stok")?;
    if req.items.is_empty() {
        return Err(AppError::validasi("Isi hasil hitung fisik minimal satu varian"));
    }
    let mut unik = HashSet::new();
    for i in &req.items {
        if i.stok_fisik < 0 {
            return Err(AppError::validasi("Stok fisik tidak boleh negatif"));
        }
        if !unik.insert(i.id_varian) {
            return Err(AppError::validasi("Varian yang sama tercantum lebih dari sekali"));
        }
    }

    let mut tx = state.pool.begin().await?;
    let ids: Vec<i64> = req.items.iter().map(|i| i.id_varian).collect();
    // Kunci semua varian sekaligus dengan urutan id agar penjualan bersamaan menunggu.
    let varian = sqlx::query!(
        r#"SELECT v.id_varian, v.stok, v.sku, p.nama || ' ' || v.ukuran || '/' || v.warna AS "nama_barang!"
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk
           WHERE v.id_varian = ANY($1)
           ORDER BY v.id_varian
           FOR UPDATE OF v"#,
        &ids
    )
    .fetch_all(&mut *tx)
    .await?;
    if varian.len() != ids.len() {
        return Err(AppError::tidak_ditemukan("Ada varian yang tidak ditemukan"));
    }

    let referensi = sqlx::query_scalar!(
        r#"SELECT 'OPN-' || to_char(now() AT TIME ZONE $1, 'YYMMDD-HH24MISS') AS "ref!""#,
        state.config.zona_waktu
    )
    .fetch_one(&mut *tx)
    .await?;

    let mut daftar_selisih = Vec::new();
    for v in &varian {
        let fisik = req.items.iter().find(|i| i.id_varian == v.id_varian).map(|i| i.stok_fisik).unwrap_or(v.stok);
        let selisih = fisik - v.stok;
        if selisih != 0 {
            stok::terapkan(
                &mut tx,
                Mutasi {
                    id_varian: v.id_varian,
                    jenis: "penyesuaian",
                    jumlah: selisih,
                    referensi: Some(&referensi),
                    id_pengguna: admin.id_pengguna,
                    keterangan: Some(&alasan),
                },
            )
            .await?;
        }
        daftar_selisih.push(SelisihOpname {
            id_varian: v.id_varian,
            nama_barang: v.nama_barang.clone(),
            sku: v.sku.clone(),
            stok_sistem: v.stok,
            stok_fisik: fisik,
            selisih,
        });
    }

    let dihitung = daftar_selisih.len();
    let sesuai = daftar_selisih.iter().filter(|s| s.selisih == 0).count();
    let akurasi_persen = (sesuai as f64 * 10_000.0 / dihitung as f64).round() / 100.0;
    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, "stok_opname", "stok").id_objek(&referensi).keterangan(alasan.clone()).baru(json!({
            "dihitung": dihitung,
            "sesuai": sesuai,
            "akurasi_persen": akurasi_persen,
            "selisih": daftar_selisih.iter().filter(|s| s.selisih != 0).map(|s| json!({
                "sku": s.sku, "stok_sistem": s.stok_sistem, "stok_fisik": s.stok_fisik, "selisih": s.selisih
            })).collect::<Vec<_>>(),
        })),
    )
    .await?;
    tx.commit().await?;

    Ok(Json(HasilOpname { referensi, dihitung, sesuai, akurasi_persen, selisih: daftar_selisih }))
}

async fn kartu_stok(_: Admin, State(state): State<AppState>, Path(id): Path<i64>, Query(q): Query<CariRentang>) -> AppResult<Json<KartuStok>> {
    let v = sqlx::query!(
        r#"SELECT v.id_varian, v.id_produk, p.nama AS nama_produk, v.sku, v.ukuran, v.warna, v.stok, v.stok_minimum
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk WHERE v.id_varian = $1"#,
        id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan("Varian tidak ditemukan"))?;

    let saldo_awal = match q.dari {
        Some(dari) => sqlx::query_scalar!(
            r#"SELECT stok_sesudah FROM mutasi_stok
               WHERE id_varian = $1 AND tanggal < $2::date::timestamp AT TIME ZONE $3
               ORDER BY tanggal DESC, id_mutasi DESC LIMIT 1"#,
            id,
            dari,
            state.config.zona_waktu,
        )
        .fetch_optional(&state.pool)
        .await?
        .unwrap_or(0),
        None => 0,
    };

    let mutasi = sqlx::query_as!(
        MutasiKartu,
        r#"SELECT m.id_mutasi, m.tanggal, m.jenis, m.jumlah, m.stok_sesudah, m.referensi, m.keterangan, p.nama AS nama_pengguna
           FROM mutasi_stok m JOIN pengguna p ON p.id_pengguna = m.id_pengguna
           WHERE m.id_varian = $1
             AND ($2::date IS NULL OR m.tanggal >= $2::date::timestamp AT TIME ZONE $4)
             AND ($3::date IS NULL OR m.tanggal < ($3::date + 1)::timestamp AT TIME ZONE $4)
           ORDER BY m.tanggal, m.id_mutasi
           LIMIT 1000"#,
        id,
        q.dari,
        q.sampai,
        state.config.zona_waktu,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(KartuStok {
        id_varian: v.id_varian,
        id_produk: v.id_produk,
        nama_produk: v.nama_produk,
        sku: v.sku,
        ukuran: v.ukuran,
        warna: v.warna,
        stok: v.stok,
        stok_minimum: v.stok_minimum,
        saldo_awal,
        mutasi,
    }))
}

/// Varian yang mencapai stok minimum dan varian yang lambat terjual: masih ada
/// stok, tidak terjual selama N hari, dan produknya bukan barang baru.
async fn peringatan(_: Admin, State(state): State<AppState>) -> AppResult<Json<PeringatanStok>> {
    let mut conn = state.pool.acquire().await?;
    let hari = pengaturan::angka(&mut *conn, pengaturan::HARI_LAMBAT_TERJUAL, 30).await?;

    let menipis = sqlx::query_as!(
        VarianStok,
        r#"SELECT v.id_varian, v.id_produk, p.nama AS nama_produk, p.kode, v.sku, v.ukuran, v.warna, v.stok, v.stok_minimum,
                  (SELECT max(t.tanggal_waktu) FROM detail_transaksi d JOIN transaksi t ON t.id_transaksi = d.id_transaksi
                   WHERE d.id_varian = v.id_varian AND t.status = 'selesai') AS terakhir_terjual
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk
           WHERE v.aktif AND p.aktif AND v.stok <= v.stok_minimum
           ORDER BY v.stok - v.stok_minimum, p.nama, v.ukuran, v.warna"#
    )
    .fetch_all(&mut *conn)
    .await?;

    let lambat_terjual = stok::lambat_terjual(&mut conn, hari).await?;

    Ok(Json(PeringatanStok { hari_lambat_terjual: hari, menipis, lambat_terjual }))
}

/// Angka kecil untuk lencana menu pemilik; dipanggil berkala sebagai permintaan latar.
async fn lencana(_: Admin, State(state): State<AppState>) -> AppResult<Json<Lencana>> {
    let r = sqlx::query!(
        r#"SELECT
             (SELECT count(*) FROM retur WHERE status = 'menunggu') AS "pengajuan_menunggu!",
             (SELECT count(*) FROM varian v JOIN produk p ON p.id_produk = v.id_produk
              WHERE v.aktif AND p.aktif AND v.stok <= v.stok_minimum) AS "stok_menipis!""#
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(Lencana { pengajuan_menunggu: r.pengajuan_menunggu, stok_menipis: r.stok_menipis }))
}
