use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Peran {
    Kasir,
    Admin,
}

impl Peran {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Kasir => "kasir",
            Self::Admin => "admin",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Pengguna {
    pub id_pengguna: i64,
    pub nama: String,
    pub username: String,
    pub peran: String,
    pub aktif: bool,
    pub dibuat_pada: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Login {
    pub username: String,
    pub kata_sandi: String,
}

#[derive(Debug, Deserialize)]
pub struct TambahPengguna {
    pub nama: String,
    pub username: String,
    pub kata_sandi: String,
    pub peran: Peran,
}

#[derive(Debug, Deserialize)]
pub struct UbahPengguna {
    pub nama: String,
    pub peran: Peran,
    /// Diisi hanya bila kata sandi direset oleh admin.
    pub kata_sandi: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UbahStatus {
    pub aktif: bool,
}
