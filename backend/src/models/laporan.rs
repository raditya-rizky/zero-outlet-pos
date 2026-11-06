use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::{kosong_jadi_none, stok::VarianStok};

#[derive(Debug, Deserialize)]
pub struct CariLaporan {
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub dari: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub sampai: Option<NaiveDate>,
    /// Admin dapat menyaring per kasir (penjual); kasir selalu melihat transaksinya sendiri.
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub id_kasir: Option<i64>,
}

/// Angka utama penjualan pada satu periode. Retur dihitung pada tanggal disetujui.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Ringkasan {
    pub transaksi: i64,
    pub unit: i64,
    /// Jumlah x harga jual sebelum diskon apa pun.
    pub kotor: i64,
    pub diskon: i64,
    pub retur: i64,
    /// Kotor - diskon - retur (= total transaksi - retur).
    pub bersih: i64,
    /// Rata-rata nilai transaksi setelah diskon.
    pub rata_rata: i64,
    /// Harga pokok barang terjual dikurangi barang yang diretur.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hpp: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TitikTren {
    pub tanggal: NaiveDate,
    /// Total transaksi setelah diskon, sebelum retur.
    pub total: i64,
    pub transaksi: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BarisRincian {
    pub kunci: String,
    pub label: String,
    pub unit: i64,
    pub penjualan: i64,
}

#[derive(Debug, Serialize)]
pub struct BarisMetode {
    pub metode: String,
    pub transaksi: i64,
    pub penjualan: i64,
    pub retur: i64,
}

#[derive(Debug, Serialize)]
pub struct BarisKasir {
    pub id_pengguna: i64,
    pub nama: String,
    pub transaksi: i64,
    pub penjualan: i64,
    pub retur: i64,
}

#[derive(Debug, Serialize)]
pub struct LaporanPenjualan {
    pub dari: NaiveDate,
    pub sampai: NaiveDate,
    pub kelompok: &'static str,
    pub ringkasan: Ringkasan,
    pub tren: Vec<TitikTren>,
    pub per_produk: Vec<BarisRincian>,
    pub per_kategori: Vec<BarisRincian>,
    pub per_ukuran: Vec<BarisRincian>,
    pub per_metode: Vec<BarisMetode>,
    /// Hanya untuk admin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_kasir: Option<Vec<BarisKasir>>,
    pub terlaris: Vec<BarisRincian>,
    /// Per hari ini, bukan per periode laporan; hanya untuk admin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lambat_terjual: Option<Vec<VarianStok>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BiayaKategori {
    pub kategori: String,
    pub nominal: i64,
}

#[derive(Debug, Serialize)]
pub struct PeriodeLabaRugi {
    pub dari: NaiveDate,
    pub sampai: NaiveDate,
    pub kotor: i64,
    pub diskon: i64,
    pub retur: i64,
    pub bersih: i64,
    pub hpp: i64,
    pub laba_kotor: i64,
    pub biaya: i64,
    pub laba_bersih: i64,
    pub margin_kotor_bps: Option<i64>,
    pub margin_bersih_bps: Option<i64>,
    pub biaya_per_kategori: Vec<BiayaKategori>,
}

#[derive(Debug, Serialize)]
pub struct LabaRugi {
    pub sekarang: PeriodeLabaRugi,
    pub sebelumnya: PeriodeLabaRugi,
}

#[derive(Debug, Serialize)]
pub struct BarisStok {
    pub id_varian: i64,
    pub id_produk: i64,
    pub kode: String,
    pub nama_produk: String,
    pub kategori: String,
    pub ukuran: String,
    pub warna: String,
    pub sku: String,
    pub stok: i32,
    pub stok_minimum: i32,
    pub harga_beli: i64,
    pub nilai: i64,
}

#[derive(Debug, Serialize)]
pub struct LaporanStok {
    pub varian: Vec<BarisStok>,
    pub total_unit: i64,
    pub total_nilai: i64,
}

#[derive(Debug, Serialize)]
pub struct SesiRingkas {
    pub id_sesi: i64,
    pub nama_kasir: String,
    pub waktu_buka: DateTime<Utc>,
    pub waktu_tutup: Option<DateTime<Utc>>,
    pub selisih: Option<i64>,
    pub alasan_selisih: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Dasbor {
    pub hari_ini: NaiveDate,
    pub ringkasan_hari_ini: Ringkasan,
    pub ringkasan_kemarin: Ringkasan,
    pub tren: Vec<TitikTren>,
    pub terlaris_7_hari: Vec<BarisRincian>,
    pub stok_menipis: i64,
    pub varian_menipis: Vec<VarianStok>,
    pub pengajuan_menunggu: i64,
    pub sesi_aktif: Vec<SesiRingkas>,
    pub sesi_terakhir: Vec<SesiRingkas>,
}
