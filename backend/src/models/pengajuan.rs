use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::services::hitung::MetodeBayar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JenisPengajuan {
    Batal,
    Retur,
    Tukar,
}

impl JenisPengajuan {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Batal => "batal",
            Self::Retur => "retur",
            Self::Tukar => "tukar",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BuatPengajuan {
    pub id_transaksi: i64,
    pub jenis: JenisPengajuan,
    pub alasan: String,
    /// Bawaan: metode bayar transaksi asal. Diabaikan untuk tukar ukuran.
    pub metode_pengembalian: Option<MetodeBayar>,
    /// Kosong untuk pembatalan (seluruh baris otomatis ikut).
    #[serde(default)]
    pub items: Vec<ItemPengajuanBaru>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ItemPengajuanBaru {
    pub id_detail: i64,
    pub jumlah: i32,
    /// Wajib untuk tukar ukuran: varian lain dari produk yang sama.
    pub id_varian_pengganti: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ItemPengajuan {
    pub id_detail_retur: i64,
    pub id_detail: i64,
    pub id_varian: i64,
    pub nama_barang: String,
    pub jumlah: i32,
    pub nilai: i64,
    pub id_varian_pengganti: Option<i64>,
    pub nama_pengganti: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Pengajuan {
    pub id_retur: i64,
    pub id_transaksi: i64,
    pub no_faktur: String,
    pub tanggal_transaksi: DateTime<Utc>,
    pub jenis: String,
    pub status: String,
    pub alasan: String,
    pub alasan_tolak: Option<String>,
    pub total_pengembalian: i64,
    pub metode_pengembalian: Option<String>,
    pub id_sesi: i64,
    pub id_pengguna: i64,
    pub nama_pengaju: String,
    pub nama_otorisator: Option<String>,
    pub tanggal_waktu: DateTime<Utc>,
    pub waktu_keputusan: Option<DateTime<Utc>>,
    /// Transaksi lebih tua dari batas hari retur (peringatan untuk pemilik).
    pub lewat_batas_hari: bool,
    pub items: Vec<ItemPengajuan>,
}

#[derive(Debug, Deserialize)]
pub struct CariPengajuan {
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TolakPengajuan {
    pub alasan: String,
}

/// Hasil pencarian faktur tepat untuk memulai pengajuan.
#[derive(Debug, Serialize)]
pub struct FakturPengajuan {
    pub id_transaksi: i64,
    pub no_faktur: String,
    pub tanggal_waktu: DateTime<Utc>,
    pub nama_kasir: String,
    pub status: String,
    pub metode_bayar: String,
    pub subtotal: i64,
    pub diskon: i64,
    pub total: i64,
    pub lewat_batas_hari: bool,
    /// Pembatalan hanya untuk transaksi pada sesi kasir yang sedang dibuka pengaju.
    pub bisa_dibatalkan: bool,
    pub items: Vec<BarisFaktur>,
}

#[derive(Debug, Serialize)]
pub struct BarisFaktur {
    pub id_detail: i64,
    pub id_varian: i64,
    pub nama_barang: String,
    pub jumlah: i32,
    pub subtotal: i64,
    /// Nilai yang benar-benar dibayar untuk baris ini setelah bagian diskon transaksi.
    pub dibayar: i64,
    /// Unit yang masih dapat diretur (dikurangi pengajuan menunggu dan disetujui).
    pub sisa: i32,
    /// Varian lain dari produk yang sama untuk tukar ukuran.
    pub pengganti: Vec<VarianPengganti>,
}

#[derive(Debug, Serialize)]
pub struct VarianPengganti {
    pub id_varian: i64,
    pub ukuran: String,
    pub warna: String,
    pub stok: i32,
}

#[derive(Debug, Serialize)]
pub struct RingkasPengajuan {
    pub id_retur: i64,
    pub jenis: String,
    pub status: String,
    pub total_pengembalian: i64,
    pub tanggal_waktu: DateTime<Utc>,
}
