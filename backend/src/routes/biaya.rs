//! Pencatatan biaya operasional untuk laporan laba rugi (KF-17). Tanpa
//! perhitungan penggajian; setiap perubahan tercatat di log aktivitas.

use axum::{Json, Router, extract::{Path, Query, State}, routing::{get, put}};
use serde_json::{Value, json};
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::Admin,
    models::{
        biaya::{Biaya, KATEGORI_BIAYA, SimpanBiaya},
        stok::CariRentang,
    },
    services::log::{self, Catatan},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/biaya", get(daftar).post(tambah))
        .route("/biaya/{id}", put(ubah).delete(hapus))
}

fn validasi(req: &SimpanBiaya) -> AppResult<Option<String>> {
    if !KATEGORI_BIAYA.contains(&req.kategori.as_str()) {
        return Err(AppError::validasi("Kategori biaya tidak dikenal"));
    }
    if req.nominal <= 0 {
        return Err(AppError::validasi("Nominal biaya harus lebih dari 0"));
    }
    let keterangan = req.keterangan.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    if req.kategori == "Lain-lain" && keterangan.is_none() {
        return Err(AppError::validasi("Keterangan wajib diisi untuk kategori Lain-lain"));
    }
    Ok(keterangan)
}

async fn ambil(conn: &mut PgConnection, id: i64) -> AppResult<Biaya> {
    sqlx::query_as!(
        Biaya,
        r#"SELECT b.id_biaya, b.tanggal, b.kategori, b.nominal, b.keterangan, p.nama AS nama_pengguna, b.dibuat_pada
           FROM biaya_operasional b JOIN pengguna p ON p.id_pengguna = b.id_pengguna
           WHERE b.id_biaya = $1"#,
        id
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan("Biaya tidak ditemukan"))
}

fn ringkas(b: &Biaya) -> Value {
    json!({ "tanggal": b.tanggal, "kategori": b.kategori, "nominal": b.nominal, "keterangan": b.keterangan })
}

async fn daftar(_: Admin, State(state): State<AppState>, Query(q): Query<CariRentang>) -> AppResult<Json<Vec<Biaya>>> {
    let rows = sqlx::query_as!(
        Biaya,
        r#"SELECT b.id_biaya, b.tanggal, b.kategori, b.nominal, b.keterangan, p.nama AS nama_pengguna, b.dibuat_pada
           FROM biaya_operasional b JOIN pengguna p ON p.id_pengguna = b.id_pengguna
           WHERE ($1::date IS NULL OR b.tanggal >= $1) AND ($2::date IS NULL OR b.tanggal <= $2)
           ORDER BY b.tanggal DESC, b.id_biaya DESC
           LIMIT 500"#,
        q.dari,
        q.sampai,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn tambah(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<SimpanBiaya>) -> AppResult<Json<Biaya>> {
    let keterangan = validasi(&req)?;
    let mut tx = state.pool.begin().await?;
    let id = sqlx::query_scalar!(
        "INSERT INTO biaya_operasional (tanggal, kategori, nominal, keterangan, id_pengguna) VALUES ($1, $2, $3, $4, $5) RETURNING id_biaya",
        req.tanggal,
        req.kategori,
        req.nominal,
        keterangan,
        admin.id_pengguna,
    )
    .fetch_one(&mut *tx)
    .await?;
    let baru = ambil(&mut tx, id).await?;
    log::catat(&mut *tx, Catatan::new(admin.id_pengguna, "tambah", "biaya").id_objek(id).baru(ringkas(&baru))).await?;
    tx.commit().await?;
    Ok(Json(baru))
}

async fn ubah(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>, Json(req): Json<SimpanBiaya>) -> AppResult<Json<Biaya>> {
    let keterangan = validasi(&req)?;
    let mut tx = state.pool.begin().await?;
    let lama = ambil(&mut tx, id).await?;
    sqlx::query!(
        "UPDATE biaya_operasional SET tanggal = $2, kategori = $3, nominal = $4, keterangan = $5 WHERE id_biaya = $1",
        id,
        req.tanggal,
        req.kategori,
        req.nominal,
        keterangan,
    )
    .execute(&mut *tx)
    .await?;
    let baru = ambil(&mut tx, id).await?;
    log::catat(&mut *tx, Catatan::new(admin.id_pengguna, "ubah", "biaya").id_objek(id).lama(ringkas(&lama)).baru(ringkas(&baru))).await?;
    tx.commit().await?;
    Ok(Json(baru))
}

async fn hapus(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let lama = ambil(&mut tx, id).await?;
    sqlx::query!("DELETE FROM biaya_operasional WHERE id_biaya = $1", id).execute(&mut *tx).await?;
    log::catat(&mut *tx, Catatan::new(admin.id_pengguna, "hapus", "biaya").id_objek(id).lama(ringkas(&lama))).await?;
    tx.commit().await?;
    Ok(Json(json!({ "pesan": "Biaya dihapus" })))
}
