use serde::{Deserialize, Serialize};

use super::kosong_jadi_none;

#[derive(Debug, Serialize)]
pub struct Kategori {
    pub id_kategori: i64,
    pub nama_kategori: String,
}

#[derive(Debug, Deserialize)]
pub struct TambahKategori {
    pub nama_kategori: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Varian {
    pub id_varian: i64,
    pub id_produk: i64,
    pub sku: String,
    pub ukuran: String,
    pub warna: String,
    pub stok: i32,
    pub stok_minimum: i32,
    pub aktif: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Produk {
    pub id_produk: i64,
    pub kode: String,
    pub nama: String,
    pub id_kategori: i64,
    pub nama_kategori: String,
    /// Disembunyikan dari kasir (Tabel 2: kasir tidak melihat harga beli).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harga_beli: Option<i64>,
    pub harga_jual: i64,
    pub aktif: bool,
    pub varian: Vec<Varian>,
}

#[derive(Debug, Deserialize)]
pub struct CariProduk {
    #[serde(default)]
    pub cari: Option<String>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub id_kategori: Option<i64>,
    /// `aktif` (bawaan), `nonaktif`, atau `semua`. Kasir selalu `aktif`.
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SimpanProduk {
    pub kode: String,
    pub nama: String,
    pub id_kategori: i64,
    pub harga_beli: i64,
    pub harga_jual: i64,
    #[serde(default = "benar")]
    pub aktif: bool,
    pub varian: Vec<SimpanVarian>,
}

#[derive(Debug, Deserialize)]
pub struct SimpanVarian {
    /// Kosong untuk varian baru.
    pub id_varian: Option<i64>,
    /// Dibuat otomatis dari kode-ukuran-warna bila kosong.
    pub sku: Option<String>,
    pub ukuran: String,
    pub warna: String,
    /// Hanya dipakai untuk varian baru; perubahan stok berikutnya lewat barang masuk/stok opname.
    #[serde(default)]
    pub stok_awal: i32,
    #[serde(default)]
    pub stok_minimum: i32,
    #[serde(default = "benar")]
    pub aktif: bool,
}

#[derive(Debug, Serialize)]
pub struct HasilSku {
    pub produk: Produk,
    pub varian: Varian,
}

fn benar() -> bool {
    true
}
