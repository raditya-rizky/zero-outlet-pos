use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Kesalahan aplikasi. Setiap varian dipetakan ke kode status HTTP dan
/// dikirim ke antarmuka sebagai `{ "pesan": "..." }` berbahasa Indonesia.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validasi(String),
    #[error("{0}")]
    TidakTerautentikasi(String),
    #[error("Anda tidak memiliki akses ke fitur ini")]
    TidakBerhak,
    #[error("{0}")]
    TidakDitemukan(String),
    #[error("{0}")]
    Konflik(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn validasi(pesan: impl Into<String>) -> Self {
        Self::Validasi(pesan.into())
    }

    pub fn tidak_ditemukan(pesan: impl Into<String>) -> Self {
        Self::TidakDitemukan(pesan.into())
    }

    pub fn konflik(pesan: impl Into<String>) -> Self {
        Self::Konflik(pesan.into())
    }

    pub fn belum_login() -> Self {
        Self::TidakTerautentikasi("Silakan login terlebih dahulu".into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Validasi(_) => StatusCode::BAD_REQUEST,
            Self::TidakTerautentikasi(_) => StatusCode::UNAUTHORIZED,
            Self::TidakBerhak => StatusCode::FORBIDDEN,
            Self::TidakDitemukan(_) => StatusCode::NOT_FOUND,
            Self::Konflik(_) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let pesan = if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = ?self, "kesalahan server");
            "Terjadi kesalahan pada server. Silakan coba lagi.".to_string()
        } else {
            self.to_string()
        };

        (status, Json(json!({ "pesan": pesan }))).into_response()
    }
}

/// Mengembalikan nama constraint bila kesalahan berasal dari pelanggaran UNIQUE.
pub fn unique_violation(err: &sqlx::Error) -> Option<&str> {
    match err {
        sqlx::Error::Database(db) if db.is_unique_violation() => db.constraint(),
        _ => None,
    }
}
