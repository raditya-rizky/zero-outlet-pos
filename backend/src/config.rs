use std::{env, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    /// Sesi login berakhir bila tidak ada aktivitas selama durasi ini (KF-01).
    pub session_idle_minutes: i32,
    /// Atribut `Secure` pada cookie; aktifkan pada mode cloud (HTTPS).
    pub cookie_secure: bool,
    /// Folder hasil build frontend; `None` bila backend hanya menyajikan API.
    pub frontend_dist: Option<PathBuf>,
    /// Zona waktu toko untuk penomoran faktur dan batas hari pada filter tanggal.
    pub zona_waktu: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            session_idle_minutes: 30,
            cookie_secure: false,
            frontend_dist: None,
            zona_waktu: "Asia/Makassar".into(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let bawaan = Self::default();
        Self {
            session_idle_minutes: env::var("SESSION_IDLE_MINUTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(bawaan.session_idle_minutes),
            cookie_secure: env::var("COOKIE_SECURE").is_ok_and(|v| v == "true"),
            frontend_dist: env::var("FRONTEND_DIST")
                .ok()
                .map(PathBuf::from)
                .filter(|p| p.join("index.html").exists()),
            zona_waktu: env::var("ZONA_WAKTU").unwrap_or(bawaan.zona_waktu),
        }
    }
}
