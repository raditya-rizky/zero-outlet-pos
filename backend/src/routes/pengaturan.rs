//! Pengaturan toko: identitas struk, batas diskon kasir (KF-07), batas hari
//! retur (KF-10), dan ambang produk lambat terjual (KF-14).

use std::collections::HashMap;

use axum::{Json, Router, extract::State, routing::get};
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::{Admin, PenggunaLogin},
    models::{pengaturan::Pengaturan, wajib},
    services::{
        log::{self, Catatan},
        pengaturan::{BATAS_DISKON_KASIR, BATAS_HARI_RETUR, HARI_LAMBAT_TERJUAL},
    },
};

pub fn router() -> Router<AppState> {
    Router::new().route("/pengaturan", get(ambil).put(ubah))
}

async fn muat(conn: &mut PgConnection) -> AppResult<Pengaturan> {
    let baris: HashMap<String, String> = sqlx::query!("SELECT kunci, nilai FROM pengaturan")
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|b| (b.kunci, b.nilai))
        .collect();
    let teks = |k: &str, bawaan: &str| baris.get(k).cloned().unwrap_or_else(|| bawaan.to_string());
    let angka = |k: &str, bawaan: i64| baris.get(k).and_then(|v| v.trim().parse().ok()).unwrap_or(bawaan);
    Ok(Pengaturan {
        nama_toko: teks("nama_toko", "Zero Outlet"),
        alamat_toko: teks("alamat_toko", ""),
        catatan_struk: teks("catatan_struk", ""),
        batas_diskon_kasir_persen: angka(BATAS_DISKON_KASIR, 10),
        batas_hari_retur: angka(BATAS_HARI_RETUR, 3),
        hari_lambat_terjual: angka(HARI_LAMBAT_TERJUAL, 30),
    })
}

async fn ambil(_: PenggunaLogin, State(state): State<AppState>) -> AppResult<Json<Pengaturan>> {
    let mut conn = state.pool.acquire().await?;
    Ok(Json(muat(&mut conn).await?))
}

async fn ubah(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<Pengaturan>) -> AppResult<Json<Pengaturan>> {
    let nama_toko = wajib(&req.nama_toko, "Nama toko")?;
    if !(0..=100).contains(&req.batas_diskon_kasir_persen) {
        return Err(AppError::validasi("Batas diskon kasir harus 0 sampai 100%"));
    }
    if !(1..=90).contains(&req.batas_hari_retur) {
        return Err(AppError::validasi("Batas hari retur harus 1 sampai 90 hari"));
    }
    if !(7..=365).contains(&req.hari_lambat_terjual) {
        return Err(AppError::validasi("Ambang lambat terjual harus 7 sampai 365 hari"));
    }

    let mut tx = state.pool.begin().await?;
    let lama = muat(&mut tx).await?;
    let nilai = [
        ("nama_toko", nama_toko),
        ("alamat_toko", req.alamat_toko.trim().to_string()),
        ("catatan_struk", req.catatan_struk.trim().to_string()),
        (BATAS_DISKON_KASIR, req.batas_diskon_kasir_persen.to_string()),
        (BATAS_HARI_RETUR, req.batas_hari_retur.to_string()),
        (HARI_LAMBAT_TERJUAL, req.hari_lambat_terjual.to_string()),
    ];
    for (kunci, isi) in nilai {
        sqlx::query!(
            "INSERT INTO pengaturan (kunci, nilai) VALUES ($1, $2) ON CONFLICT (kunci) DO UPDATE SET nilai = EXCLUDED.nilai",
            kunci,
            isi,
        )
        .execute(&mut *tx)
        .await?;
    }
    let baru = muat(&mut tx).await?;
    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, "ubah", "pengaturan")
            .lama(serde_json::to_value(&lama).unwrap_or_default())
            .baru(serde_json::to_value(&baru).unwrap_or_default()),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(baru))
}
