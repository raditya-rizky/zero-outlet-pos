//! Perubahan stok varian. Setiap perubahan selalu disertai baris mutasi_stok
//! (kartu stok, KF-13) dalam transaksi basis data yang sama.

use sqlx::PgConnection;

use crate::{AppError, AppResult, models::stok::VarianStok};

pub struct Mutasi<'a> {
    pub id_varian: i64,
    pub jenis: &'static str,
    /// Positif menambah stok, negatif mengurangi.
    pub jumlah: i32,
    pub referensi: Option<&'a str>,
    pub id_pengguna: i64,
    pub keterangan: Option<&'a str>,
}

/// Menerapkan mutasi dan mengembalikan stok sesudahnya. Pengurangan memakai
/// syarat `stok >= jumlah` sehingga dua kasir yang menjual unit terakhir secara
/// bersamaan tidak membuat stok negatif: transaksi yang kalah mendapat 0 baris.
pub async fn terapkan(conn: &mut PgConnection, m: Mutasi<'_>) -> AppResult<i32> {
    if m.jumlah == 0 {
        return Err(AppError::validasi("Jumlah mutasi stok tidak boleh 0"));
    }

    let stok_sesudah = sqlx::query_scalar!(
        "UPDATE varian SET stok = stok + $2 WHERE id_varian = $1 AND stok + $2 >= 0 RETURNING stok",
        m.id_varian,
        m.jumlah,
    )
    .fetch_optional(&mut *conn)
    .await?;

    let Some(stok_sesudah) = stok_sesudah else {
        let info = sqlx::query!(
            r#"SELECT p.nama, v.ukuran, v.warna, v.stok
               FROM varian v JOIN produk p USING (id_produk) WHERE v.id_varian = $1"#,
            m.id_varian
        )
        .fetch_optional(&mut *conn)
        .await?;
        return Err(match info {
            Some(v) => AppError::konflik(format!(
                "Stok tidak cukup untuk {} ({} / {}): tersedia {}",
                v.nama, v.ukuran, v.warna, v.stok
            )),
            None => AppError::tidak_ditemukan("Varian tidak ditemukan"),
        });
    };

    sqlx::query!(
        r#"INSERT INTO mutasi_stok (id_varian, jenis, jumlah, stok_sesudah, referensi, id_pengguna, keterangan)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        m.id_varian,
        m.jenis,
        m.jumlah,
        stok_sesudah,
        m.referensi,
        m.id_pengguna,
        m.keterangan,
    )
    .execute(&mut *conn)
    .await?;

    Ok(stok_sesudah)
}

/// Varian lambat terjual per hari ini (KF-14): masih ada stok, tidak terjual
/// selama `hari` hari terakhir, dan produknya bukan barang baru.
pub async fn lambat_terjual(conn: &mut PgConnection, hari: i64) -> AppResult<Vec<VarianStok>> {
    sqlx::query_as!(
        VarianStok,
        r#"SELECT v.id_varian, v.id_produk, p.nama AS nama_produk, p.kode, v.sku, v.ukuran, v.warna, v.stok, v.stok_minimum,
                  (SELECT max(t.tanggal_waktu) FROM detail_transaksi d JOIN transaksi t ON t.id_transaksi = d.id_transaksi
                   WHERE d.id_varian = v.id_varian AND t.status = 'selesai') AS terakhir_terjual
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk
           WHERE v.aktif AND p.aktif AND v.stok > 0
             AND p.dibuat_pada < now() - make_interval(days => $1)
             AND NOT EXISTS (
               SELECT 1 FROM detail_transaksi d JOIN transaksi t ON t.id_transaksi = d.id_transaksi
               WHERE d.id_varian = v.id_varian AND t.status = 'selesai'
                 AND t.tanggal_waktu >= now() - make_interval(days => $1)
             )
           ORDER BY terakhir_terjual NULLS FIRST, p.nama, v.ukuran, v.warna"#,
        hari as i32
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(Into::into)
}
