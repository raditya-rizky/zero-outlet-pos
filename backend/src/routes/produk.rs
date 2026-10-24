//! Data produk, varian ukuran-warna, kategori, dan pencarian (KF-03, KF-04, KF-05).

use std::collections::HashSet;

use axum::{Json, Router, extract::{Path, Query, State}, routing::get};
use serde_json::{Value, json};
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    error::unique_violation,
    middleware::auth::{Admin, PenggunaLogin},
    models::{
        produk::{CariProduk, HasilSku, Kategori, Produk, SimpanProduk, SimpanVarian, TambahKategori, Varian},
        wajib,
    },
    services::{
        log::{self, Catatan},
        stok::{self, Mutasi},
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/kategori", get(daftar_kategori).post(tambah_kategori))
        .route("/produk", get(cari).post(tambah))
        .route("/produk/{id}", get(detail).put(ubah).delete(hapus))
        .route("/varian/sku/{sku}", get(cari_sku))
}

async fn daftar_kategori(_: PenggunaLogin, State(state): State<AppState>) -> AppResult<Json<Vec<Kategori>>> {
    let rows = sqlx::query_as!(Kategori, "SELECT id_kategori, nama_kategori FROM kategori ORDER BY nama_kategori")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(rows))
}

async fn tambah_kategori(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<TambahKategori>) -> AppResult<Json<Kategori>> {
    let nama = wajib(&req.nama_kategori, "Nama kategori")?;
    let mut tx = state.pool.begin().await?;
    let kategori = sqlx::query_as!(
        Kategori,
        "INSERT INTO kategori (nama_kategori) VALUES ($1) RETURNING id_kategori, nama_kategori",
        nama
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(pesan_unik)?;
    log::catat(
        &mut *tx,
        Catatan::new(admin.id_pengguna, "tambah", "kategori").id_objek(kategori.id_kategori).baru(json!({ "nama_kategori": nama })),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(kategori))
}

/// Filter pemuatan produk. `semua_varian` juga menampilkan varian nonaktif (untuk admin).
struct Filter<'a> {
    kata: &'a [String],
    id_kategori: Option<i64>,
    aktif: Option<bool>,
    id_produk: Option<i64>,
    semua_varian: bool,
    harga_beli: bool,
}

/// Escape karakter khusus LIKE agar input pengguna dicari apa adanya.
fn pola_like(kata: &str) -> String {
    kata.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

async fn muat(conn: &mut PgConnection, f: Filter<'_>) -> AppResult<Vec<Produk>> {
    let kata: Vec<String> = f.kata.iter().map(|k| pola_like(k)).collect();
    let baris = sqlx::query!(
        r#"SELECT p.id_produk, p.kode, p.nama, p.id_kategori, k.nama_kategori, p.harga_beli, p.harga_jual, p.aktif
           FROM produk p JOIN kategori k ON k.id_kategori = p.id_kategori
           WHERE ($2::bigint IS NULL OR p.id_kategori = $2)
             AND ($3::bool IS NULL OR p.aktif = $3)
             AND ($4::bigint IS NULL OR p.id_produk = $4)
             -- Setiap kata harus cocok dengan produk atau salah satu variannya.
             AND NOT EXISTS (
               SELECT 1 FROM unnest($1::text[]) AS t(kata)
               WHERE NOT (
                 p.nama ILIKE '%' || t.kata || '%'
                 OR p.kode ILIKE '%' || t.kata || '%'
                 OR k.nama_kategori ILIKE '%' || t.kata || '%'
                 OR EXISTS (
                   SELECT 1 FROM varian v
                   WHERE v.id_produk = p.id_produk AND ($5 OR v.aktif)
                     AND (v.sku ILIKE '%' || t.kata || '%' OR v.ukuran ILIKE t.kata OR v.warna ILIKE '%' || t.kata || '%')
                 )
               )
             )
           ORDER BY p.nama
           LIMIT 200"#,
        &kata,
        f.id_kategori,
        f.aktif,
        f.id_produk,
        f.semua_varian,
    )
    .fetch_all(&mut *conn)
    .await?;

    let ids: Vec<i64> = baris.iter().map(|b| b.id_produk).collect();
    let varian = sqlx::query_as!(
        Varian,
        r#"SELECT id_varian, id_produk, sku, ukuran, warna, stok, stok_minimum, aktif
           FROM varian
           WHERE id_produk = ANY($1) AND ($2 OR aktif)
           ORDER BY id_produk,
                    array_position(ARRAY['XS','S','M','L','XL','XXL','XXXL'], upper(ukuran)) NULLS LAST,
                    ukuran, warna"#,
        &ids,
        f.semua_varian,
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(baris
        .into_iter()
        .map(|b| Produk {
            varian: varian.iter().filter(|v| v.id_produk == b.id_produk).cloned().collect(),
            id_produk: b.id_produk,
            kode: b.kode,
            nama: b.nama,
            id_kategori: b.id_kategori,
            nama_kategori: b.nama_kategori,
            harga_beli: f.harga_beli.then_some(b.harga_beli),
            harga_jual: b.harga_jual,
            aktif: b.aktif,
        })
        .collect())
}

async fn muat_satu(conn: &mut PgConnection, id: i64, admin: bool) -> AppResult<Produk> {
    let filter = Filter {
        kata: &[],
        id_kategori: None,
        aktif: if admin { None } else { Some(true) },
        id_produk: Some(id),
        semua_varian: admin,
        harga_beli: admin,
    };
    muat(conn, filter).await?.pop().ok_or_else(|| AppError::tidak_ditemukan("Produk tidak ditemukan"))
}

async fn cari(pengguna: PenggunaLogin, State(state): State<AppState>, Query(q): Query<CariProduk>) -> AppResult<Json<Vec<Produk>>> {
    let admin = pengguna.is_admin();
    let kata: Vec<String> = q.cari.unwrap_or_default().split_whitespace().map(str::to_owned).collect();
    let aktif = match (admin, q.status.as_deref()) {
        (true, Some("semua")) => None,
        (true, Some("nonaktif")) => Some(false),
        _ => Some(true),
    };
    let mut conn = state.pool.acquire().await?;
    let produk = muat(
        &mut conn,
        Filter { kata: &kata, id_kategori: q.id_kategori, aktif, id_produk: None, semua_varian: admin, harga_beli: admin },
    )
    .await?;
    Ok(Json(produk))
}

async fn detail(pengguna: PenggunaLogin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Produk>> {
    let mut conn = state.pool.acquire().await?;
    Ok(Json(muat_satu(&mut conn, id, pengguna.is_admin()).await?))
}

/// Pencarian tepat berdasarkan SKU/barcode untuk pemindai (KF-05).
async fn cari_sku(pengguna: PenggunaLogin, State(state): State<AppState>, Path(sku): Path<String>) -> AppResult<Json<HasilSku>> {
    let id_produk = sqlx::query_scalar!(
        r#"SELECT v.id_produk FROM varian v JOIN produk p USING (id_produk)
           WHERE upper(v.sku) = upper($1) AND v.aktif AND p.aktif"#,
        sku.trim()
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan(format!("Barang dengan SKU \"{}\" tidak ditemukan", sku.trim())))?;

    let mut conn = state.pool.acquire().await?;
    let produk = muat_satu(&mut conn, id_produk, pengguna.is_admin()).await?;
    let varian = produk
        .varian
        .iter()
        .find(|v| v.sku.eq_ignore_ascii_case(sku.trim()))
        .cloned()
        .ok_or_else(|| AppError::tidak_ditemukan("Varian tidak ditemukan"))?;
    Ok(Json(HasilSku { produk, varian }))
}

fn pesan_unik(e: sqlx::Error) -> AppError {
    match unique_violation(&e) {
        Some("produk_kode_key") => AppError::konflik("Kode produk sudah dipakai produk lain"),
        Some("varian_sku_key") => AppError::konflik("SKU/barcode sudah dipakai varian lain"),
        Some("varian_id_produk_ukuran_warna_key") => AppError::konflik("Kombinasi ukuran dan warna tidak boleh ganda"),
        Some("kategori_nama_kategori_key") => AppError::konflik("Nama kategori sudah ada"),
        _ => e.into(),
    }
}

struct VarianBersih {
    id_varian: Option<i64>,
    sku: String,
    ukuran: String,
    warna: String,
    stok_awal: i32,
    stok_minimum: i32,
    aktif: bool,
}

/// Memvalidasi isian produk dan menyiapkan SKU otomatis.
fn validasi(req: &SimpanProduk) -> AppResult<(String, String, Vec<VarianBersih>)> {
    let kode = wajib(&req.kode, "Kode produk")?.to_uppercase();
    let nama = wajib(&req.nama, "Nama produk")?;
    if req.harga_beli < 0 || req.harga_jual < 0 {
        return Err(AppError::validasi("Harga tidak boleh negatif"));
    }
    if req.harga_jual < req.harga_beli {
        return Err(AppError::validasi("Harga jual tidak boleh lebih kecil dari harga beli"));
    }
    if req.varian.is_empty() {
        return Err(AppError::validasi("Produk minimal memiliki satu varian"));
    }

    let mut kombinasi = HashSet::new();
    let mut hasil = Vec::with_capacity(req.varian.len());
    for v in &req.varian {
        let SimpanVarian { id_varian, sku, ukuran, warna, stok_awal, stok_minimum, aktif } = v;
        let ukuran = wajib(ukuran, "Ukuran")?.to_uppercase();
        let warna = wajib(warna, "Warna")?;
        if !kombinasi.insert((ukuran.clone(), warna.to_lowercase())) {
            return Err(AppError::validasi(format!("Varian {ukuran} / {warna} tercantum lebih dari sekali")));
        }
        if *stok_awal < 0 || *stok_minimum < 0 {
            return Err(AppError::validasi("Stok tidak boleh negatif"));
        }
        let sku = match sku.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) => s.to_uppercase(),
            None => format!("{kode}-{ukuran}-{warna}").to_uppercase().split_whitespace().collect::<String>(),
        };
        hasil.push(VarianBersih {
            id_varian: *id_varian,
            sku,
            ukuran,
            warna,
            stok_awal: *stok_awal,
            stok_minimum: *stok_minimum,
            aktif: *aktif,
        });
    }
    Ok((kode, nama, hasil))
}

async fn sisipkan_varian(conn: &mut PgConnection, id_produk: i64, v: &VarianBersih, id_pengguna: i64) -> AppResult<()> {
    let id_varian = sqlx::query_scalar!(
        r#"INSERT INTO varian (id_produk, sku, ukuran, warna, stok_minimum, aktif)
           VALUES ($1, $2, $3, $4, $5, $6) RETURNING id_varian"#,
        id_produk,
        v.sku,
        v.ukuran,
        v.warna,
        v.stok_minimum,
        v.aktif,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(pesan_unik)?;

    if v.stok_awal > 0 {
        stok::terapkan(
            conn,
            Mutasi {
                id_varian,
                jenis: "penyesuaian",
                jumlah: v.stok_awal,
                referensi: Some("STOK-AWAL"),
                id_pengguna,
                keterangan: Some("Stok awal saat varian ditambahkan"),
            },
        )
        .await?;
    }
    Ok(())
}

fn snapshot(p: &Produk) -> Value {
    serde_json::to_value(p).unwrap_or(Value::Null)
}

async fn tambah(Admin(admin): Admin, State(state): State<AppState>, Json(req): Json<SimpanProduk>) -> AppResult<Json<Produk>> {
    let (kode, nama, varian) = validasi(&req)?;
    let mut tx = state.pool.begin().await?;

    let id_produk = sqlx::query_scalar!(
        r#"INSERT INTO produk (kode, nama, id_kategori, harga_beli, harga_jual, aktif)
           VALUES ($1, $2, $3, $4, $5, $6) RETURNING id_produk"#,
        kode,
        nama,
        req.id_kategori,
        req.harga_beli,
        req.harga_jual,
        req.aktif,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(pesan_unik)?;

    for v in &varian {
        sisipkan_varian(&mut tx, id_produk, v, admin.id_pengguna).await?;
    }

    let produk = muat_satu(&mut tx, id_produk, true).await?;
    log::catat(&mut *tx, Catatan::new(admin.id_pengguna, "tambah", "produk").id_objek(id_produk).baru(snapshot(&produk))).await?;
    tx.commit().await?;
    Ok(Json(produk))
}

async fn ubah(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>, Json(req): Json<SimpanProduk>) -> AppResult<Json<Produk>> {
    let (kode, nama, varian) = validasi(&req)?;
    let mut tx = state.pool.begin().await?;

    // Kunci baris produk agar dua admin tidak menimpa perubahan satu sama lain.
    sqlx::query_scalar!("SELECT id_produk FROM produk WHERE id_produk = $1 FOR UPDATE", id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::tidak_ditemukan("Produk tidak ditemukan"))?;
    let lama = muat_satu(&mut tx, id, true).await?;

    sqlx::query!(
        r#"UPDATE produk SET kode = $2, nama = $3, id_kategori = $4, harga_beli = $5, harga_jual = $6, aktif = $7
           WHERE id_produk = $1"#,
        id,
        kode,
        nama,
        req.id_kategori,
        req.harga_beli,
        req.harga_jual,
        req.aktif,
    )
    .execute(&mut *tx)
    .await
    .map_err(pesan_unik)?;

    let id_lama: HashSet<i64> = lama.varian.iter().map(|v| v.id_varian).collect();
    let mut dipertahankan = HashSet::new();
    for v in &varian {
        match v.id_varian {
            Some(id_varian) => {
                if !id_lama.contains(&id_varian) {
                    return Err(AppError::validasi("Varian tidak termasuk dalam produk ini"));
                }
                dipertahankan.insert(id_varian);
                sqlx::query!(
                    "UPDATE varian SET sku = $2, ukuran = $3, warna = $4, stok_minimum = $5, aktif = $6 WHERE id_varian = $1",
                    id_varian,
                    v.sku,
                    v.ukuran,
                    v.warna,
                    v.stok_minimum,
                    v.aktif,
                )
                .execute(&mut *tx)
                .await
                .map_err(pesan_unik)?;
            }
            None => sisipkan_varian(&mut tx, id, v, admin.id_pengguna).await?,
        }
    }
    // Varian yang dihapus dari formulir dinonaktifkan agar riwayatnya tetap utuh.
    let dinonaktifkan: Vec<i64> = id_lama.difference(&dipertahankan).copied().collect();
    sqlx::query!("UPDATE varian SET aktif = FALSE WHERE id_varian = ANY($1)", &dinonaktifkan)
        .execute(&mut *tx)
        .await?;

    let baru = muat_satu(&mut tx, id, true).await?;
    let mut catatan = Catatan::new(admin.id_pengguna, "ubah", "produk").id_objek(id).lama(snapshot(&lama)).baru(snapshot(&baru));
    if lama.harga_jual != baru.harga_jual {
        catatan = catatan.keterangan(format!("Harga jual {} -> {}", lama.harga_jual, baru.harga_jual));
    }
    log::catat(&mut *tx, catatan).await?;
    tx.commit().await?;
    Ok(Json(baru))
}

/// Produk tanpa riwayat dihapus; produk yang sudah memiliki riwayat transaksi
/// atau mutasi stok hanya dinonaktifkan agar laporan tetap utuh.
async fn hapus(Admin(admin): Admin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Value>> {
    let mut tx = state.pool.begin().await?;
    let lama = muat_satu(&mut tx, id, true).await?;

    let punya_riwayat = sqlx::query_scalar!(
        r#"SELECT EXISTS (
             SELECT 1 FROM varian v
             WHERE v.id_produk = $1 AND (
               EXISTS (SELECT 1 FROM detail_transaksi d WHERE d.id_varian = v.id_varian)
               OR EXISTS (SELECT 1 FROM mutasi_stok m WHERE m.id_varian = v.id_varian)
               OR EXISTS (SELECT 1 FROM detail_barang_masuk b WHERE b.id_varian = v.id_varian)
             )
           ) AS "ada!""#,
        id
    )
    .fetch_one(&mut *tx)
    .await?;

    let hasil = if punya_riwayat {
        sqlx::query!("UPDATE produk SET aktif = FALSE WHERE id_produk = $1", id).execute(&mut *tx).await?;
        log::catat(
            &mut *tx,
            Catatan::new(admin.id_pengguna, "nonaktifkan", "produk")
                .id_objek(id)
                .lama(json!({ "aktif": lama.aktif }))
                .baru(json!({ "aktif": false })),
        )
        .await?;
        json!({ "dihapus": false, "pesan": "Produk sudah memiliki riwayat stok/transaksi sehingga dinonaktifkan" })
    } else {
        sqlx::query!("DELETE FROM produk WHERE id_produk = $1", id).execute(&mut *tx).await?;
        log::catat(&mut *tx, Catatan::new(admin.id_pengguna, "hapus", "produk").id_objek(id).lama(snapshot(&lama))).await?;
        json!({ "dihapus": true, "pesan": "Produk dihapus" })
    };
    tx.commit().await?;
    Ok(Json(hasil))
}
