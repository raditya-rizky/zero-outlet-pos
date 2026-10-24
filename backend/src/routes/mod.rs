use axum::Router;

use crate::{AppError, AppState};

mod auth;
mod log;
mod pengguna;
mod produk;
mod sesi;
mod transaksi;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(auth::router())
        .merge(pengguna::router())
        .merge(produk::router())
        .merge(sesi::router())
        .merge(transaksi::router())
        .merge(log::router())
        .fallback(|| async { AppError::tidak_ditemukan("Endpoint tidak ditemukan") })
}
