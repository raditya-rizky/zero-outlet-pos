//! Pengelolaan akun pengguna oleh Pemilik/Admin (KF-02).

use axum::{Json, Router, extract::{Path, State}, routing::{get, put}};
use serde_json::json;

use crate::{
    AppError, AppResult, AppState,
    error::unique_violation,
    middleware::auth::Admin,
    models::{
        pengguna::{Pengguna, TambahPengguna, UbahPengguna, UbahStatus},
        wajib,
    },
    services::{log::{self, Catatan}, sandi},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/pengguna", get(daftar).post(tambah))
        .route("/pengguna/{id}", put(ubah))
        .route("/pengguna/{id}/status", put(ubah_status))
}

async fn daftar(_: Admin, State(state): State<AppState>) -> AppResult<Json<Vec<Pengguna>>> {
    let rows = sqlx::query_as!(
        Pengguna,
        "SELECT id_pengguna, nama, username, peran, aktif, dibuat_pada FROM pengguna ORDER BY aktif DESC, peran, nama"
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn ambil(db: impl sqlx::PgExecutor<'_>, id: i64) -> AppResult<Pengguna> {
    sqlx::query_as!(
        Pengguna,
        "SELECT id_pengguna, nama, username, peran, aktif, dibuat_pada FROM pengguna WHERE id_pengguna = $1",
        id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan("Pengguna tidak ditemukan"))
}

async fn tambah(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<TambahPengguna>) -> AppResult<Json<Pengguna>> {
    let nama = wajib(&req.nama, "Nama")?;
    let username = wajib(&req.username, "Nama pengguna")?.to_lowercase();
    if username.contains(char::is_whitespace) {
        return Err(AppError::validasi("Nama pengguna tidak boleh mengandung spasi"));
    }
    sandi::validasi(&req.kata_sandi)?;
    let hash = sandi::hash(req.kata_sandi).await?;

    let mut tx = state.pool.begin().await?;
    let id = sqlx::query_scalar!(
        "INSERT INTO pengguna (nama, username, kata_sandi, peran) VALUES ($1, $2, $3, $4) RETURNING id_pengguna",
        nama,
        username,
        hash,
        req.peran.as_str(),
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match unique_violation(&e) {
        Some(_) => AppError::konflik(format!("Nama pengguna \"{username}\" sudah dipakai")),
        None => e.into(),
    })?;

    let baru = ambil(&mut *tx, id).await?;
    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, "tambah", "pengguna")
            .id_objek(id)
            .baru(json!({ "nama": baru.nama, "username": baru.username, "peran": baru.peran })),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(baru))
}

async fn ubah(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>, Json(req): Json<UbahPengguna>) -> AppResult<Json<Pengguna>> {
    let nama = wajib(&req.nama, "Nama")?;
    if id == admin.id_pengguna && req.peran.as_str() != "admin" {
        return Err(AppError::validasi("Anda tidak dapat menurunkan peran akun sendiri"));
    }
    let hash = match req.kata_sandi.filter(|s| !s.is_empty()) {
        Some(s) => {
            sandi::validasi(&s)?;
            Some(sandi::hash(s).await?)
        }
        None => None,
    };

    let mut tx = state.pool.begin().await?;
    let lama = ambil(&mut *tx, id).await?;
    sqlx::query!(
        "UPDATE pengguna SET nama = $2, peran = $3, kata_sandi = coalesce($4, kata_sandi) WHERE id_pengguna = $1",
        id,
        nama,
        req.peran.as_str(),
        hash,
    )
    .execute(&mut *tx)
    .await?;
    if hash.is_some() {
        // Kata sandi direset: paksa login ulang di semua perangkat.
        sqlx::query!("DELETE FROM sesi_login WHERE id_pengguna = $1", id).execute(&mut *tx).await?;
    }
    let baru = ambil(&mut *tx, id).await?;
    let mut catatan = Catatan::new(admin.id_pengguna, "ubah", "pengguna")
        .id_objek(id)
        .lama(json!({ "nama": lama.nama, "peran": lama.peran }))
        .baru(json!({ "nama": baru.nama, "peran": baru.peran }));
    if hash.is_some() {
        catatan = catatan.keterangan("Kata sandi direset");
    }
    log::catat(&mut *tx, catatan).await?;
    tx.commit().await?;
    Ok(Json(baru))
}

async fn ubah_status(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>, Json(req): Json<UbahStatus>) -> AppResult<Json<Pengguna>> {
    if id == admin.id_pengguna && !req.aktif {
        return Err(AppError::validasi("Anda tidak dapat menonaktifkan akun sendiri"));
    }
    let mut tx = state.pool.begin().await?;
    let lama = ambil(&mut *tx, id).await?;
    sqlx::query!("UPDATE pengguna SET aktif = $2 WHERE id_pengguna = $1", id, req.aktif)
        .execute(&mut *tx)
        .await?;
    if !req.aktif {
        sqlx::query!("DELETE FROM sesi_login WHERE id_pengguna = $1", id).execute(&mut *tx).await?;
    }
    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, if req.aktif { "aktifkan" } else { "nonaktifkan" }, "pengguna")
            .id_objek(id)
            .lama(json!({ "aktif": lama.aktif }))
            .baru(json!({ "aktif": req.aktif })),
    )
    .await?;
    let baru = ambil(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(Json(baru))
}
