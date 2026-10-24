//! Pencatatan log aktivitas (KF-20). Dipanggil dengan executor yang sama
//! dengan perubahan datanya agar log dan perubahan tersimpan atau batal bersama.

use serde_json::Value;
use sqlx::PgExecutor;

#[derive(Debug, Default)]
pub struct Catatan {
    pub id_pengguna: Option<i64>,
    pub aksi: &'static str,
    pub objek: &'static str,
    pub id_objek: Option<String>,
    pub nilai_lama: Option<Value>,
    pub nilai_baru: Option<Value>,
    pub keterangan: Option<String>,
}

impl Catatan {
    pub fn new(id_pengguna: i64, aksi: &'static str, objek: &'static str) -> Self {
        Self { id_pengguna: Some(id_pengguna), aksi, objek, ..Default::default() }
    }

    pub fn id_objek(mut self, id: impl ToString) -> Self {
        self.id_objek = Some(id.to_string());
        self
    }

    pub fn lama(mut self, nilai: Value) -> Self {
        self.nilai_lama = Some(nilai);
        self
    }

    pub fn baru(mut self, nilai: Value) -> Self {
        self.nilai_baru = Some(nilai);
        self
    }

    pub fn keterangan(mut self, ket: impl Into<String>) -> Self {
        self.keterangan = Some(ket.into());
        self
    }
}

pub async fn catat<'e>(db: impl PgExecutor<'e>, c: Catatan) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO log_aktivitas (id_pengguna, aksi, objek, id_objek, nilai_lama, nilai_baru, keterangan)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        c.id_pengguna,
        c.aksi,
        c.objek,
        c.id_objek,
        c.nilai_lama,
        c.nilai_baru,
        c.keterangan,
    )
    .execute(db)
    .await?;
    Ok(())
}
