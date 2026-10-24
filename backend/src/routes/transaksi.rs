//! Transaksi penjualan, riwayat, dan struk (KF-06, KF-08, KF-09).

use std::collections::BTreeMap;

use axum::{Json, Router, extract::{Path, Query, State}, routing::get};
use serde_json::json;
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::PenggunaLogin,
    models::transaksi::{BuatTransaksi, CariTransaksi, InfoToko, ItemStruk, OpsiStruk, RingkasTransaksi, Struk},
    services::{
        hitung::{self, Baris},
        log::{self, Catatan},
        stok::{self, Mutasi},
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/transaksi", get(daftar).post(buat))
        .route("/transaksi/{id}/struk", get(struk))
}

/// Menyimpan transaksi secara atomik: header, detail, pengurangan stok, kartu
/// stok, dan log tersimpan bersama atau dibatalkan seluruhnya (Bab III.C.4).
async fn buat(pengguna: PenggunaLogin, State(state): State<AppState>, Json(req): Json<BuatTransaksi>) -> AppResult<Json<Struk>> {
    // Gabungkan varian yang sama dan urutkan berdasarkan id agar urutan
    // penguncian baris konsisten antarkasir (mencegah deadlock).
    let mut jumlah_per_varian: BTreeMap<i64, i32> = BTreeMap::new();
    for item in &req.items {
        if item.jumlah <= 0 {
            return Err(AppError::validasi("Jumlah barang harus lebih dari 0"));
        }
        *jumlah_per_varian.entry(item.id_varian).or_default() += item.jumlah;
    }
    if jumlah_per_varian.is_empty() {
        return Err(AppError::validasi("Keranjang masih kosong"));
    }

    let mut tx = state.pool.begin().await?;

    let id_sesi = sqlx::query_scalar!(
        "SELECT id_sesi FROM sesi_kasir WHERE id_pengguna = $1 AND waktu_tutup IS NULL FOR UPDATE",
        pengguna.id_pengguna
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::konflik("Buka sesi kasir terlebih dahulu sebelum melakukan transaksi"))?;

    let ids: Vec<i64> = jumlah_per_varian.keys().copied().collect();
    let data_varian = sqlx::query!(
        r#"SELECT v.id_varian, v.ukuran, v.warna, v.aktif AS varian_aktif,
                  p.nama, p.harga_jual, p.harga_beli, p.aktif AS produk_aktif
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk
           WHERE v.id_varian = ANY($1)"#,
        &ids
    )
    .fetch_all(&mut *tx)
    .await?;

    struct BarisSiap {
        id_varian: i64,
        nama_barang: String,
        jumlah: i32,
        harga_satuan: i64,
        harga_pokok: i64,
    }
    let mut baris = Vec::with_capacity(ids.len());
    for (&id_varian, &jumlah) in &jumlah_per_varian {
        let v = data_varian
            .iter()
            .find(|v| v.id_varian == id_varian)
            .ok_or_else(|| AppError::tidak_ditemukan("Barang di keranjang tidak ditemukan"))?;
        if !v.varian_aktif || !v.produk_aktif {
            return Err(AppError::validasi(format!("{} ({} / {}) sudah tidak dijual", v.nama, v.ukuran, v.warna)));
        }
        baris.push(BarisSiap {
            id_varian,
            nama_barang: format!("{} {}/{}", v.nama, v.ukuran, v.warna),
            jumlah,
            harga_satuan: v.harga_jual,
            harga_pokok: v.harga_beli,
        });
    }

    // Harga selalu diambil dari basis data, bukan dari isian antarmuka.
    let input_hitung: Vec<Baris> =
        baris.iter().map(|b| Baris { jumlah: b.jumlah, harga_satuan: b.harga_satuan, diskon_item: 0 }).collect();
    let rincian = hitung::hitung_transaksi(&input_hitung, 0, req.metode_bayar, req.bayar)?;

    let no_faktur = sqlx::query_scalar!(
        r#"SELECT 'ZO-' || to_char(now() AT TIME ZONE $1, 'YYMMDD') || '-' || lpad(nextval('no_faktur_seq')::text, 6, '0') AS "no!""#,
        state.config.zona_waktu
    )
    .fetch_one(&mut *tx)
    .await?;

    let id_transaksi = sqlx::query_scalar!(
        r#"INSERT INTO transaksi (no_faktur, id_sesi, id_pengguna, subtotal, diskon, total, metode_bayar, bayar, kembalian)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id_transaksi"#,
        no_faktur,
        id_sesi,
        pengguna.id_pengguna,
        rincian.subtotal,
        rincian.diskon,
        rincian.total,
        req.metode_bayar.as_str(),
        rincian.bayar,
        rincian.kembalian,
    )
    .fetch_one(&mut *tx)
    .await?;

    for (b, subtotal) in baris.iter().zip(&rincian.subtotal_baris) {
        sqlx::query!(
            r#"INSERT INTO detail_transaksi (id_transaksi, id_varian, nama_barang, jumlah, harga_satuan, harga_pokok, diskon_item, subtotal)
               VALUES ($1, $2, $3, $4, $5, $6, 0, $7)"#,
            id_transaksi,
            b.id_varian,
            b.nama_barang,
            b.jumlah,
            b.harga_satuan,
            b.harga_pokok,
            subtotal,
        )
        .execute(&mut *tx)
        .await?;

        stok::terapkan(
            &mut tx,
            Mutasi {
                id_varian: b.id_varian,
                jenis: "keluar",
                jumlah: -b.jumlah,
                referensi: Some(&no_faktur),
                id_pengguna: pengguna.id_pengguna,
                keterangan: Some("Penjualan"),
            },
        )
        .await?;
    }

    log::catat(
        &mut *tx,
        Catatan::new(pengguna.id_pengguna, "transaksi", "transaksi").id_objek(id_transaksi).baru(json!({
            "no_faktur": no_faktur,
            "total": rincian.total,
            "metode_bayar": req.metode_bayar.as_str(),
            "jumlah_item": baris.len(),
        })),
    )
    .await?;

    let hasil = muat_struk(&mut tx, id_transaksi, false).await?;
    tx.commit().await?;
    Ok(Json(hasil))
}

async fn muat_struk(conn: &mut PgConnection, id_transaksi: i64, cetak_ulang: bool) -> AppResult<Struk> {
    let t = sqlx::query!(
        r#"SELECT t.id_transaksi, t.no_faktur, t.tanggal_waktu, t.id_sesi, t.id_pengguna, p.nama AS nama_kasir,
                  t.subtotal, t.diskon, t.total, t.metode_bayar, t.bayar, t.kembalian, t.status, t.jumlah_cetak
           FROM transaksi t JOIN pengguna p ON p.id_pengguna = t.id_pengguna
           WHERE t.id_transaksi = $1"#,
        id_transaksi
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan("Transaksi tidak ditemukan"))?;

    let items = sqlx::query_as!(
        ItemStruk,
        r#"SELECT id_detail, id_varian, nama_barang, jumlah, harga_satuan, diskon_item, subtotal
           FROM detail_transaksi WHERE id_transaksi = $1 ORDER BY id_detail"#,
        id_transaksi
    )
    .fetch_all(&mut *conn)
    .await?;

    let toko = sqlx::query!(
        r#"SELECT
             coalesce((SELECT nilai FROM pengaturan WHERE kunci = 'nama_toko'), 'Zero Outlet') AS "nama!",
             coalesce((SELECT nilai FROM pengaturan WHERE kunci = 'alamat_toko'), '') AS "alamat!",
             coalesce((SELECT nilai FROM pengaturan WHERE kunci = 'catatan_struk'), '') AS "catatan!""#
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(Struk {
        toko: InfoToko { nama: toko.nama, alamat: toko.alamat, catatan: toko.catatan },
        id_transaksi: t.id_transaksi,
        no_faktur: t.no_faktur,
        tanggal_waktu: t.tanggal_waktu,
        id_sesi: t.id_sesi,
        id_pengguna: t.id_pengguna,
        nama_kasir: t.nama_kasir,
        subtotal: t.subtotal,
        diskon: t.diskon,
        total: t.total,
        metode_bayar: t.metode_bayar,
        bayar: t.bayar,
        kembalian: t.kembalian,
        status: t.status,
        jumlah_cetak: t.jumlah_cetak,
        cetak_ulang,
        items,
    })
}

/// Detail transaksi dalam bentuk struk. Dengan `?cetak_ulang=true`, jumlah
/// cetak bertambah, struk ditandai CETAK ULANG, dan aktivitasnya dicatat.
async fn struk(
    pengguna: PenggunaLogin,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(opsi): Query<OpsiStruk>,
) -> AppResult<Json<Struk>> {
    let mut tx = state.pool.begin().await?;
    let pemilik = sqlx::query_scalar!("SELECT id_pengguna FROM transaksi WHERE id_transaksi = $1", id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::tidak_ditemukan("Transaksi tidak ditemukan"))?;
    if !pengguna.is_admin() && pemilik != pengguna.id_pengguna {
        return Err(AppError::TidakBerhak);
    }

    if opsi.cetak_ulang {
        let ke = sqlx::query_scalar!(
            "UPDATE transaksi SET jumlah_cetak = jumlah_cetak + 1 WHERE id_transaksi = $1 RETURNING jumlah_cetak",
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        log::catat(
            &mut *tx,
            Catatan::new(pengguna.id_pengguna, "cetak_ulang", "transaksi").id_objek(id).keterangan(format!("Cetakan ke-{ke}")),
        )
        .await?;
    }

    let hasil = muat_struk(&mut tx, id, opsi.cetak_ulang).await?;
    tx.commit().await?;
    Ok(Json(hasil))
}

/// Riwayat transaksi. Kasir hanya melihat transaksinya sendiri (Tabel 2).
async fn daftar(pengguna: PenggunaLogin, State(state): State<AppState>, Query(q): Query<CariTransaksi>) -> AppResult<Json<Vec<RingkasTransaksi>>> {
    let id_pengguna = if pengguna.is_admin() { q.id_pengguna } else { Some(pengguna.id_pengguna) };
    let no_faktur = q.no_faktur.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let rows = sqlx::query_as!(
        RingkasTransaksi,
        r#"SELECT t.id_transaksi, t.no_faktur, t.tanggal_waktu, t.id_pengguna, p.nama AS nama_kasir,
                  t.total, t.metode_bayar, t.status,
                  (SELECT coalesce(sum(d.jumlah), 0) FROM detail_transaksi d WHERE d.id_transaksi = t.id_transaksi)::bigint AS "jumlah_barang!"
           FROM transaksi t JOIN pengguna p ON p.id_pengguna = t.id_pengguna
           WHERE ($1::bigint IS NULL OR t.id_pengguna = $1)
             AND ($2::text IS NULL OR t.no_faktur ILIKE '%' || $2 || '%')
             AND ($3::date IS NULL OR t.tanggal_waktu >= $3::date::timestamp AT TIME ZONE $5)
             AND ($4::date IS NULL OR t.tanggal_waktu < ($4::date + 1)::timestamp AT TIME ZONE $5)
           ORDER BY t.tanggal_waktu DESC
           LIMIT 200"#,
        id_pengguna,
        no_faktur,
        q.dari,
        q.sampai,
        state.config.zona_waktu,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}
