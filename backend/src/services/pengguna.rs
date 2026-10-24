use sqlx::PgPool;

use crate::{AppError, services::sandi};

/// Membuat akun admin pertama bila belum ada admin aktif.
/// Mengembalikan `true` bila akun baru dibuat.
pub async fn pastikan_admin(pool: &PgPool, nama: &str, username: &str, kata_sandi: &str) -> Result<bool, AppError> {
    let ada_admin = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM pengguna WHERE peran = 'admin' AND aktif) AS "ada!""#
    )
    .fetch_one(pool)
    .await?;
    if ada_admin {
        return Ok(false);
    }

    sandi::validasi(kata_sandi)?;
    let hash = sandi::hash(kata_sandi.to_string()).await?;
    sqlx::query!(
        "INSERT INTO pengguna (nama, username, kata_sandi, peran) VALUES ($1, $2, $3, 'admin')",
        nama,
        username,
        hash,
    )
    .execute(pool)
    .await?;
    Ok(true)
}
