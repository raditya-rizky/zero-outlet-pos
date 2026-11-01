use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::kosong_jadi_none;

#[derive(Debug, Serialize)]
pub struct Pemasok {
    pub id_pemasok: i64,
    pub nama: String,
    pub kontak: Option<String>,
    pub alamat: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TambahPemasok {
    pub nama: String,
    pub kontak: Option<String>,
    pub alamat: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BuatBarangMasuk {
    /// Bawaan: hari ini.
    pub tanggal: Option<NaiveDate>,
    pub id_pemasok: Option<i64>,
    pub no_referensi: Option<String>,
    pub keterangan: Option<String>,
    pub items: Vec<ItemBarangMasukBaru>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ItemBarangMasukBaru {
    pub id_varian: i64,
    pub jumlah: i32,
    pub harga_beli: i64,
}

#[derive(Debug, Serialize)]
pub struct ItemBarangMasuk {
    pub id_detail: i64,
    pub id_varian: i64,
    pub nama_barang: String,
    pub sku: String,
    pub jumlah: i32,
    pub harga_beli: i64,
}

#[derive(Debug, Serialize)]
pub struct BarangMasuk {
    pub id_barang_masuk: i64,
    pub tanggal: NaiveDate,
    pub no_referensi: Option<String>,
    pub id_pemasok: Option<i64>,
    pub nama_pemasok: Option<String>,
    pub nama_pengguna: String,
    pub keterangan: Option<String>,
    pub jumlah_unit: i64,
    pub total_nilai: i64,
    pub dibuat_pada: DateTime<Utc>,
    pub items: Vec<ItemBarangMasuk>,
}

#[derive(Debug, Serialize)]
pub struct HasilBarangMasuk {
    pub barang_masuk: BarangMasuk,
    /// Misalnya harga beli baru melebihi harga jual.
    pub peringatan: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CariRentang {
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub dari: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub sampai: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct StokOpname {
    pub alasan: String,
    pub items: Vec<HitungFisik>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct HitungFisik {
    pub id_varian: i64,
    pub stok_fisik: i32,
}

#[derive(Debug, Serialize)]
pub struct SelisihOpname {
    pub id_varian: i64,
    pub nama_barang: String,
    pub sku: String,
    pub stok_sistem: i32,
    pub stok_fisik: i32,
    pub selisih: i32,
}

/// Ringkasan stok opname; akurasi menjadi data indikator Tabel 1 (>= 98%).
#[derive(Debug, Serialize)]
pub struct HasilOpname {
    pub referensi: String,
    pub dihitung: usize,
    pub sesuai: usize,
    pub akurasi_persen: f64,
    pub selisih: Vec<SelisihOpname>,
}

#[derive(Debug, Serialize)]
pub struct MutasiKartu {
    pub id_mutasi: i64,
    pub tanggal: DateTime<Utc>,
    pub jenis: String,
    pub jumlah: i32,
    pub stok_sesudah: i32,
    pub referensi: Option<String>,
    pub keterangan: Option<String>,
    pub nama_pengguna: String,
}

#[derive(Debug, Serialize)]
pub struct KartuStok {
    pub id_varian: i64,
    pub id_produk: i64,
    pub nama_produk: String,
    pub sku: String,
    pub ukuran: String,
    pub warna: String,
    pub stok: i32,
    pub stok_minimum: i32,
    /// Stok sebelum `dari` (0 bila tanpa filter tanggal).
    pub saldo_awal: i32,
    pub mutasi: Vec<MutasiKartu>,
}

#[derive(Debug, Serialize)]
pub struct VarianStok {
    pub id_varian: i64,
    pub id_produk: i64,
    pub nama_produk: String,
    pub kode: String,
    pub sku: String,
    pub ukuran: String,
    pub warna: String,
    pub stok: i32,
    pub stok_minimum: i32,
    pub terakhir_terjual: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PeringatanStok {
    pub hari_lambat_terjual: i64,
    pub menipis: Vec<VarianStok>,
    pub lambat_terjual: Vec<VarianStok>,
}

#[derive(Debug, Serialize)]
pub struct Lencana {
    pub pengajuan_menunggu: i64,
    pub stok_menipis: i64,
}
