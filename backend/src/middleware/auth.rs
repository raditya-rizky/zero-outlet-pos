//! Autentikasi berbasis cookie dan otorisasi peran (Tabel 2).
//!
//! Handler yang membutuhkan login cukup menerima `PenggunaLogin`; handler
//! khusus Pemilik/Admin menerima `Admin`. Keduanya ditolak sebelum handler
//! dijalankan bila syaratnya tidak terpenuhi.

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::cookie::CookieJar;
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{AppError, AppState};

pub const NAMA_COOKIE: &str = "zo_sesi";

#[derive(Debug, Clone, Serialize)]
pub struct PenggunaLogin {
    pub id_pengguna: i64,
    pub nama: String,
    pub username: String,
    pub peran: String,
}

impl PenggunaLogin {
    pub fn is_admin(&self) -> bool {
        self.peran == "admin"
    }
}

/// Pengguna yang sudah login dan berperan Pemilik/Admin.
#[derive(Debug, Clone)]
pub struct Admin(pub PenggunaLogin);

pub fn buat_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

impl FromRequestParts<AppState> for PenggunaLogin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar.get(NAMA_COOKIE).map(|c| c.value().to_owned()).ok_or_else(AppError::belum_login)?;

        // Sekaligus memeriksa batas waktu tidak aktif dan memperbarui waktu aktivitas terakhir.
        let pengguna = sqlx::query_as!(
            PenggunaLogin,
            r#"UPDATE sesi_login s SET terakhir_aktif = now()
               FROM pengguna p
               WHERE s.token_hash = $1
                 AND p.id_pengguna = s.id_pengguna
                 AND p.aktif
                 AND s.terakhir_aktif > now() - make_interval(mins => $2)
               RETURNING p.id_pengguna, p.nama, p.username, p.peran"#,
            hash_token(&token),
            state.config.session_idle_minutes,
        )
        .fetch_optional(&state.pool)
        .await?;

        pengguna.ok_or_else(|| AppError::TidakTerautentikasi("Sesi login telah berakhir. Silakan login kembali.".into()))
    }
}

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let pengguna = PenggunaLogin::from_request_parts(parts, state).await?;
        if !pengguna.is_admin() {
            return Err(AppError::TidakBerhak);
        }
        Ok(Admin(pengguna))
    }
}
