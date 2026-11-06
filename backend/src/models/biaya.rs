use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Kategori biaya operasional sesuai contoh pada proposal (KF-17).
pub const KATEGORI_BIAYA: [&str; 6] = ["Sewa tempat", "Listrik dan air", "Upah karyawan", "Pengemasan", "Promosi", "Lain-lain"];

#[derive(Debug, Serialize)]
pub struct Biaya {
    pub id_biaya: i64,
    pub tanggal: NaiveDate,
    pub kategori: String,
    pub nominal: i64,
    pub keterangan: Option<String>,
    pub nama_pengguna: String,
    pub dibuat_pada: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SimpanBiaya {
    pub tanggal: NaiveDate,
    pub kategori: String,
    pub nominal: i64,
    pub keterangan: Option<String>,
}
