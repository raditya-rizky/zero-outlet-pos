//! Membaca pengaturan toko (tabel kunci-nilai).

use sqlx::PgExecutor;

use crate::AppResult;

pub const BATAS_DISKON_KASIR: &str = "batas_diskon_kasir_persen";
pub const BATAS_HARI_RETUR: &str = "batas_hari_retur";
pub const HARI_LAMBAT_TERJUAL: &str = "hari_lambat_terjual";

/// Nilai pengaturan berupa bilangan bulat; memakai `bawaan` bila kosong atau tidak valid.
pub async fn angka<'e>(db: impl PgExecutor<'e>, kunci: &str, bawaan: i64) -> AppResult<i64> {
    let nilai = sqlx::query_scalar!("SELECT nilai FROM pengaturan WHERE kunci = $1", kunci)
        .fetch_optional(db)
        .await?;
    Ok(nilai.and_then(|n| n.trim().parse().ok()).unwrap_or(bawaan))
}
