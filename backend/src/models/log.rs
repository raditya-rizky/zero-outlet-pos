use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::kosong_jadi_none;

#[derive(Debug, Serialize)]
pub struct LogAktivitas {
    pub id_log: i64,
    pub waktu: DateTime<Utc>,
    pub id_pengguna: Option<i64>,
    pub nama_pengguna: Option<String>,
    pub aksi: String,
    pub objek: String,
    pub id_objek: Option<String>,
    pub nilai_lama: Option<Value>,
    pub nilai_baru: Option<Value>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CariLog {
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub id_pengguna: Option<i64>,
    #[serde(default)]
    pub aksi: Option<String>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub dari: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub sampai: Option<NaiveDate>,
    #[serde(default, deserialize_with = "kosong_jadi_none")]
    pub halaman: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct HalamanLog {
    pub data: Vec<LogAktivitas>,
    pub total: i64,
    pub halaman: i64,
    pub per_halaman: i64,
}
