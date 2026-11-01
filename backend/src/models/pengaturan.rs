use serde::{Deserialize, Serialize};

/// Pengaturan toko yang dapat diubah pemilik (struk, batas diskon, retur, stok lambat).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pengaturan {
    pub nama_toko: String,
    pub alamat_toko: String,
    pub catatan_struk: String,
    pub batas_diskon_kasir_persen: i64,
    pub batas_hari_retur: i64,
    pub hari_lambat_terjual: i64,
}
