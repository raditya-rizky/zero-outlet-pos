//! Hash kata sandi dengan Argon2 (kebutuhan keamanan Tabel 4).

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::AppError;

pub const PANJANG_MINIMUM: usize = 8;

pub fn validasi(sandi: &str) -> Result<(), AppError> {
    if sandi.chars().count() < PANJANG_MINIMUM {
        return Err(AppError::validasi(format!(
            "Kata sandi minimal {PANJANG_MINIMUM} karakter"
        )));
    }
    Ok(())
}

/// Argon2 sengaja lambat, sehingga dijalankan di thread terpisah agar tidak
/// menahan runtime async.
pub async fn hash(sandi: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(sandi.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(format!("gagal membuat hash: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
}

pub async fn cocok(sandi: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash)
            .map(|h| Argon2::default().verify_password(sandi.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}
