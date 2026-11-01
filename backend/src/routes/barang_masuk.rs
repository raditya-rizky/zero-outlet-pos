//! Pencatatan barang masuk dari pemasok (KF-11). Setiap baris menambah stok
//! lewat kartu stok, dan harga beli produk diperbarui ke harga pembelian terbaru.

use std::collections::{BTreeMap, HashMap, HashSet};

use axum::{Json, Router, extract::{Path, Query, State}, routing::get};
use serde_json::json;
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::Admin,
    models::{
        stok::{BarangMasuk, BuatBarangMasuk, CariRentang, HasilBarangMasuk, ItemBarangMasuk, Pemasok, TambahPemasok},
        wajib,
    },
    services::{
        log::{self, Catatan},
        stok::{self, Mutasi},
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pemasok", get(daftar_pemasok).post(tambah_pemasok))
        .route("/barang-masuk", get(daftar).post(buat))
        .route("/barang-masuk/{id}", get(detail))
}

fn kosong_jadi_none(s: Option<String>) -> Option<String> {
    s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

async fn daftar_pemasok(_: Admin, State(state): State<AppState>) -> AppResult<Json<Vec<Pemasok>>> {
    let rows = sqlx::query_as!(Pemasok, "SELECT id_pemasok, nama, kontak, alamat FROM pemasok ORDER BY nama")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn tambah_pemasok(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<TambahPemasok>) -> AppResult<Json<Pemasok>> {
    let nama = wajib(&req.nama, "Nama pemasok")?;
    let mut tx = state.pool.begin().await?;
    let pemasok = sqlx::query_as!(
        Pemasok,
        "INSERT INTO pemasok (nama, kontak, alamat) VALUES ($1, $2, $3) RETURNING id_pemasok, nama, kontak, alamat",
        nama,
        kosong_jadi_none(req.kontak),
        kosong_jadi_none(req.alamat),
    )
    .fetch_one(&mut *tx)
    .await?;
    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, "tambah", "pemasok").id_objek(pemasok.id_pemasok).baru(json!({ "nama": pemasok.nama })),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(pemasok))
}

struct Filter {
    id: Option<i64>,
    rentang: CariRentang,
}

async fn muat(conn: &mut PgConnection, f: Filter) -> AppResult<Vec<BarangMasuk>> {
    let header = sqlx::query!(
        r#"SELECT b.id_barang_masuk, b.tanggal, b.no_referensi, b.id_pemasok, s.nama AS "nama_pemasok?", p.nama AS nama_pengguna,
                  b.keterangan, b.dibuat_pada,
                  coalesce((SELECT sum(d.jumlah) FROM detail_barang_masuk d WHERE d.id_barang_masuk = b.id_barang_masuk), 0)::bigint AS "jumlah_unit!",
                  coalesce((SELECT sum(d.jumlah * d.harga_beli) FROM detail_barang_masuk d WHERE d.id_barang_masuk = b.id_barang_masuk), 0)::bigint AS "total_nilai!"
           FROM barang_masuk b
           JOIN pengguna p ON p.id_pengguna = b.id_pengguna
           LEFT JOIN pemasok s ON s.id_pemasok = b.id_pemasok
           WHERE ($1::bigint IS NULL OR b.id_barang_masuk = $1)
             AND ($2::date IS NULL OR b.tanggal >= $2)
             AND ($3::date IS NULL OR b.tanggal <= $3)
           ORDER BY b.tanggal DESC, b.id_barang_masuk DESC
           LIMIT 100"#,
        f.id,
        f.rentang.dari,
        f.rentang.sampai,
    )
    .fetch_all(&mut *conn)
    .await?;

    let ids: Vec<i64> = header.iter().map(|h| h.id_barang_masuk).collect();
    let items = sqlx::query!(
        r#"SELECT d.id_detail, d.id_barang_masuk, d.id_varian, p.nama || ' ' || v.ukuran || '/' || v.warna AS "nama_barang!",
                  v.sku, d.jumlah, d.harga_beli
           FROM detail_barang_masuk d
           JOIN varian v ON v.id_varian = d.id_varian
           JOIN produk p ON p.id_produk = v.id_produk
           WHERE d.id_barang_masuk = ANY($1)
           ORDER BY d.id_detail"#,
        &ids
    )
    .fetch_all(&mut *conn)
    .await?;

    let mut per_header: HashMap<i64, Vec<ItemBarangMasuk>> = HashMap::new();
    for i in items {
        per_header.entry(i.id_barang_masuk).or_default().push(ItemBarangMasuk {
            id_detail: i.id_detail,
            id_varian: i.id_varian,
            nama_barang: i.nama_barang,
            sku: i.sku,
            jumlah: i.jumlah,
            harga_beli: i.harga_beli,
        });
    }
    Ok(header
        .into_iter()
        .map(|h| BarangMasuk {
            items: per_header.remove(&h.id_barang_masuk).unwrap_or_default(),
            id_barang_masuk: h.id_barang_masuk,
            tanggal: h.tanggal,
            no_referensi: h.no_referensi,
            id_pemasok: h.id_pemasok,
            nama_pemasok: h.nama_pemasok,
            nama_pengguna: h.nama_pengguna,
            keterangan: h.keterangan,
            jumlah_unit: h.jumlah_unit,
            total_nilai: h.total_nilai,
            dibuat_pada: h.dibuat_pada,
        })
        .collect())
}

async fn daftar(_: Admin, State(state): State<AppState>, Query(q): Query<CariRentang>) -> AppResult<Json<Vec<BarangMasuk>>> {
    let mut conn = state.pool.acquire().await?;
    Ok(Json(muat(&mut conn, Filter { id: None, rentang: q }).await?))
}

async fn detail(_: Admin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<BarangMasuk>> {
    let mut conn = state.pool.acquire().await?;
    muat(&mut conn, Filter { id: Some(id), rentang: CariRentang { dari: None, sampai: None } })
        .await?
        .pop()
        .map(Json)
        .ok_or_else(|| AppError::tidak_ditemukan("Barang masuk tidak ditemukan"))
}

async fn buat(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<BuatBarangMasuk>) -> AppResult<Json<HasilBarangMasuk>> {
    if req.items.is_empty() {
        return Err(AppError::validasi("Tambahkan minimal satu barang"));
    }
    let mut unik = HashSet::new();
    for i in &req.items {
        if i.jumlah <= 0 {
            return Err(AppError::validasi("Jumlah barang masuk harus lebih dari 0"));
        }
        if i.harga_beli < 0 {
            return Err(AppError::validasi("Harga beli tidak boleh negatif"));
        }
        if !unik.insert(i.id_varian) {
            return Err(AppError::validasi("Varian yang sama tercantum lebih dari sekali"));
        }
    }

    let mut tx = state.pool.begin().await?;
    if let Some(id) = req.id_pemasok {
        sqlx::query_scalar!("SELECT id_pemasok FROM pemasok WHERE id_pemasok = $1", id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::tidak_ditemukan("Pemasok tidak ditemukan"))?;
    }

    let ids: Vec<i64> = req.items.iter().map(|i| i.id_varian).collect();
    let varian = sqlx::query!(
        r#"SELECT v.id_varian, p.id_produk, p.nama, p.harga_beli, p.harga_jual
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk
           WHERE v.id_varian = ANY($1)"#,
        &ids
    )
    .fetch_all(&mut *tx)
    .await?;
    if varian.len() != ids.len() {
        return Err(AppError::tidak_ditemukan("Ada varian yang tidak ditemukan"));
    }

    let id_barang_masuk = sqlx::query_scalar!(
        r#"INSERT INTO barang_masuk (no_referensi, tanggal, id_pemasok, id_pengguna, keterangan)
           VALUES ($1, coalesce($2, CURRENT_DATE), $3, $4, $5) RETURNING id_barang_masuk"#,
        kosong_jadi_none(req.no_referensi),
        req.tanggal,
        req.id_pemasok,
        admin.id_pengguna,
        kosong_jadi_none(req.keterangan),
    )
    .fetch_one(&mut *tx)
    .await?;

    for i in &req.items {
        sqlx::query!(
            "INSERT INTO detail_barang_masuk (id_barang_masuk, id_varian, jumlah, harga_beli) VALUES ($1, $2, $3, $4)",
            id_barang_masuk,
            i.id_varian,
            i.jumlah,
            i.harga_beli,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Tambah stok dengan urutan varian yang konsisten (mencegah deadlock).
    let referensi = format!("BM-{id_barang_masuk}");
    let mut urut = req.items.clone();
    urut.sort_by_key(|i| i.id_varian);
    for i in &urut {
        stok::terapkan(
            &mut tx,
            Mutasi {
                id_varian: i.id_varian,
                jenis: "masuk",
                jumlah: i.jumlah,
                referensi: Some(&referensi),
                id_pengguna: admin.id_pengguna,
                keterangan: Some("Barang masuk dari pemasok"),
            },
        )
        .await?;
    }

    // Harga beli produk mengikuti harga pembelian terbaru (baris terakhir per produk).
    let mut harga_terbaru: BTreeMap<i64, i64> = BTreeMap::new();
    for i in &req.items {
        let v = varian.iter().find(|v| v.id_varian == i.id_varian).expect("varian sudah divalidasi");
        harga_terbaru.insert(v.id_produk, i.harga_beli);
    }
    let mut peringatan = Vec::new();
    for (id_produk, harga_baru) in harga_terbaru {
        let p = varian.iter().find(|v| v.id_produk == id_produk).expect("produk sudah divalidasi");
        if harga_baru != p.harga_beli {
            sqlx::query!("UPDATE produk SET harga_beli = $2 WHERE id_produk = $1", id_produk, harga_baru)
                .execute(&mut *tx)
                .await?;
            log::catat(
                &mut *tx,
                Catatan::new(admin.id_pengguna, "ubah", "produk")
                    .id_objek(id_produk)
                    .lama(json!({ "harga_beli": p.harga_beli }))
                    .baru(json!({ "harga_beli": harga_baru }))
                    .keterangan(format!("Harga beli dari barang masuk {referensi}")),
            )
            .await?;
        }
        if harga_baru > p.harga_jual {
            peringatan.push(format!(
                "Harga beli {} (Rp{harga_baru}) melebihi harga jual (Rp{}). Sesuaikan harga jual di halaman produk.",
                p.nama, p.harga_jual
            ));
        }
    }

    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, "tambah", "barang_masuk").id_objek(id_barang_masuk).baru(json!({
            "referensi": referensi,
            "id_pemasok": req.id_pemasok,
            "jumlah_baris": req.items.len(),
            "jumlah_unit": req.items.iter().map(|i| i.jumlah).sum::<i32>(),
        })),
    )
    .await?;

    let barang_masuk = muat(&mut tx, Filter { id: Some(id_barang_masuk), rentang: CariRentang { dari: None, sampai: None } })
        .await?
        .pop()
        .ok_or_else(|| AppError::Internal("barang masuk baru tidak terbaca".into()))?;
    tx.commit().await?;
    Ok(Json(HasilBarangMasuk { barang_masuk, peringatan }))
}
