use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::kosong_jadi_none;

#[derive(Debug, Clone, Serialize)]
pub struct Sesi {
    pub id_sesi: i64,
    pub id_pengguna: i64,
    pub nama_kasir: String,
    pub waktu_buka: DateTime<Utc>,
    pub modal_awal: i64,
    pub waktu_tutup: Option<DateTime<Utc>>,
    pub kas_fisik: Option<i64>,
    pub kas_seharusnya: Option<i64>,
    pub selisih: Option<i64>,
    pub alasan_selisih: Option<String>,
}

/// Rekap sesi untuk layar tutup sesi dan peninjauan admin (KF-19).
#[derive(Debug, Serialize)]
pub struct Rekap {
    pub sesi: Sesi,
    pub jumlah_transaksi: i64,
    pub penjualan_tunai: i64,
    pub penjualan_transfer: i64,
    pub penjualan_qris: i64,
    pub total_penjualan: i64,
    pub refund_tunai: i64,
    pub kas_seharusnya: i64,
}

#[derive(Debug, Deserialize)]
pub struct BukaSesi {
    pub modal_awal: i64,
}

#[derive(Debug, Deserialize)]
pub struct TutupSesi {
    pub kas_fisik: i64,
    pub alasan_selisih: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CariSesi {
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub dari: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub sampai: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub id_pengguna: Option<i64>,
}
