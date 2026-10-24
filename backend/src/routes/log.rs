//! Penampilan log aktivitas untuk Pemilik/Admin (KF-20). Tidak ada endpoint
//! untuk mengubah atau menghapus log.

use axum::{Json, Router, extract::{Query, State}, routing::get};

use crate::{
    AppResult, AppState,
    middleware::auth::Admin,
    models::log::{CariLog, HalamanLog, LogAktivitas},
};

const PER_HALAMAN: i64 = 50;

pub fn router() -> Router<AppState> {
    Router::new().route("/log-aktivitas", get(daftar))
}

async fn daftar(_: Admin, State(state): State<AppState>, Query(q): Query<CariLog>) -> AppResult<Json<HalamanLog>> {
    let halaman = q.halaman.unwrap_or(1).max(1);
    let aksi = q.aksi.filter(|a| !a.trim().is_empty());

    let total = sqlx::query_scalar!(
        r#"SELECT count(*) AS "total!" FROM log_aktivitas l
           WHERE ($1::bigint IS NULL OR l.id_pengguna = $1)
             AND ($2::text IS NULL OR l.aksi = $2)
             AND ($3::date IS NULL OR l.waktu >= $3::date::timestamp AT TIME ZONE $5)
             AND ($4::date IS NULL OR l.waktu < ($4::date + 1)::timestamp AT TIME ZONE $5)"#,
        q.id_pengguna,
        aksi,
        q.dari,
        q.sampai,
        state.config.zona_waktu,
    )
    .fetch_one(&state.pool)
    .await?;

    let data = sqlx::query_as!(
        LogAktivitas,
        r#"SELECT l.id_log, l.waktu, l.id_pengguna, p.nama AS "nama_pengguna?", l.aksi, l.objek, l.id_objek,
                  l.nilai_lama, l.nilai_baru, l.keterangan
           FROM log_aktivitas l LEFT JOIN pengguna p ON p.id_pengguna = l.id_pengguna
           WHERE ($1::bigint IS NULL OR l.id_pengguna = $1)
             AND ($2::text IS NULL OR l.aksi = $2)
             AND ($3::date IS NULL OR l.waktu >= $3::date::timestamp AT TIME ZONE $5)
             AND ($4::date IS NULL OR l.waktu < ($4::date + 1)::timestamp AT TIME ZONE $5)
           ORDER BY l.waktu DESC, l.id_log DESC
           LIMIT $6 OFFSET $7"#,
        q.id_pengguna,
        aksi,
        q.dari,
        q.sampai,
        state.config.zona_waktu,
        PER_HALAMAN,
        (halaman - 1) * PER_HALAMAN,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(HalamanLog { data, total, halaman, per_halaman: PER_HALAMAN }))
}
