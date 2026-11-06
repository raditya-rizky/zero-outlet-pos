//! Struktur data permintaan dan respons API.

pub mod biaya;
pub mod laporan;
pub mod log;
pub mod pengajuan;
pub mod pengaturan;
pub mod pengguna;
pub mod produk;
pub mod sesi;
pub mod stok;
pub mod transaksi;

use serde::{Deserialize, Deserializer};

/// Mengubah string kosong pada query string menjadi `None`.
pub fn kosong_jadi_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match Option::<String>::deserialize(de)? {
        Some(s) if !s.trim().is_empty() => s.trim().parse().map(Some).map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

/// Memangkas spasi dan memastikan teks wajib tidak kosong.
pub fn wajib(nilai: &str, nama_kolom: &str) -> Result<String, crate::AppError> {
    let nilai = nilai.trim();
    if nilai.is_empty() {
        return Err(crate::AppError::validasi(format!("{nama_kolom} wajib diisi")));
    }
    Ok(nilai.to_string())
}
