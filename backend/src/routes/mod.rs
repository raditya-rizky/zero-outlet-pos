use axum::Router;

use crate::{AppError, AppState};

mod auth;
mod barang_masuk;
mod biaya;
mod laporan;
mod log;
mod pengajuan;
mod pengaturan;
mod pengguna;
mod produk;
mod sesi;
mod stok;
mod transaksi;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(auth::router())
        .merge(pengguna::router())
        .merge(produk::router())
        .merge(sesi::router())
        .merge(transaksi::router())
        .merge(log::router())
        .merge(pengaturan::router())
        .merge(pengajuan::router())
        .merge(barang_masuk::router())
        .merge(stok::router())
        .merge(laporan::router())
        .merge(biaya::router())
        .fallback(|| async { AppError::tidak_ditemukan("Endpoint tidak ditemukan") })
}
