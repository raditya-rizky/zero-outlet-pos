use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::kosong_jadi_none;
use crate::services::hitung::MetodeBayar;

#[derive(Debug, Deserialize)]
pub struct BuatTransaksi {
    pub items: Vec<ItemBaru>,
    pub metode_bayar: MetodeBayar,
    /// Wajib untuk pembayaran tunai.
    pub bayar: Option<i64>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ItemBaru {
    pub id_varian: i64,
    pub jumlah: i32,
}

#[derive(Debug, Serialize)]
pub struct RingkasTransaksi {
    pub id_transaksi: i64,
    pub no_faktur: String,
    pub tanggal_waktu: DateTime<Utc>,
    pub id_pengguna: i64,
    pub nama_kasir: String,
    pub total: i64,
    pub metode_bayar: String,
    pub status: String,
    pub jumlah_barang: i64,
}

#[derive(Debug, Serialize)]
pub struct ItemStruk {
    pub id_detail: i64,
    pub id_varian: i64,
    pub nama_barang: String,
    pub jumlah: i32,
    pub harga_satuan: i64,
    pub diskon_item: i64,
    pub subtotal: i64,
}

#[derive(Debug, Serialize)]
pub struct InfoToko {
    pub nama: String,
    pub alamat: String,
    pub catatan: String,
}

#[derive(Debug, Serialize)]
pub struct Struk {
    pub toko: InfoToko,
    pub id_transaksi: i64,
    pub no_faktur: String,
    pub tanggal_waktu: DateTime<Utc>,
    pub id_sesi: i64,
    pub id_pengguna: i64,
    pub nama_kasir: String,
    pub subtotal: i64,
    pub diskon: i64,
    pub total: i64,
    pub metode_bayar: String,
    pub bayar: i64,
    pub kembalian: i64,
    pub status: String,
    pub jumlah_cetak: i32,
    pub cetak_ulang: bool,
    pub items: Vec<ItemStruk>,
}

#[derive(Debug, Deserialize)]
pub struct CariTransaksi {
    #[serde(default)]
    pub no_faktur: Option<String>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub dari: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub sampai: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub id_pengguna: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OpsiStruk {
    #[serde(default)]
    pub cetak_ulang: bool,
}
