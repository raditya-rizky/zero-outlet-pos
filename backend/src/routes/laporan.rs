//! Laporan penjualan (KF-15), ekspor CSV (KF-16), laba rugi (KF-18), laporan
//! stok, dan dasbor pemilik (Tabel 21).
//!
//! Aturan angka:
//! - penjualan hanya dari transaksi `selesai`; transaksi batal tidak dihitung;
//! - angka header (total, diskon transaksi, jumlah transaksi) dijumlahkan dari
//!   tabel transaksi saja, angka baris dari detail, agar diskon tidak berlipat;
//! - retur dihitung pada tanggal disetujui dan hanya `jenis = 'retur'`;
//!   tukar ukuran tidak memengaruhi penjualan maupun HPP;
//! - rincian per produk membagi diskon transaksi persis seperti
//!   `hitung::bagi_diskon_transaksi`, sehingga jumlahnya sama dengan total.

use std::collections::BTreeMap;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{Duration, NaiveDate};
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::{Admin, PenggunaLogin},
    models::laporan::{
        BarisKasir, BarisMetode, BarisRincian, BarisStok, BiayaKategori, CariLaporan, Dasbor, LabaRugi, LaporanPenjualan,
        LaporanStok, PeriodeLabaRugi, Ringkasan, SesiRingkas, TitikTren,
    },
    models::stok::VarianStok,
    services::{
        pengaturan,
        periode::{self, Kelompok},
        stok,
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/laporan/penjualan", get(penjualan))
        .route("/laporan/penjualan.csv", get(penjualan_csv))
        .route("/laporan/laba-rugi", get(laba_rugi))
        .route("/laporan/laba-rugi.csv", get(laba_rugi_csv))
        .route("/laporan/stok", get(laporan_stok))
        .route("/laporan/stok.csv", get(laporan_stok_csv))
        .route("/ringkasan/dasbor", get(dasbor))
}

fn rentang(q: &CariLaporan) -> AppResult<(NaiveDate, NaiveDate)> {
    let (Some(dari), Some(sampai)) = (q.dari, q.sampai) else {
        return Err(AppError::validasi("Pilih tanggal awal dan akhir laporan"));
    };
    periode::validasi(dari, sampai)?;
    Ok((dari, sampai))
}

/// Ringkasan satu periode. `kasir` menyaring berdasarkan penjual transaksi;
/// retur ikut mengurangi angka penjual transaksi asalnya.
async fn ringkasan(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate, kasir: Option<i64>) -> AppResult<Ringkasan> {
    let header = sqlx::query!(
        r#"SELECT count(*) AS "transaksi!", coalesce(sum(t.total), 0)::bigint AS "total!"
           FROM transaksi t
           WHERE t.status = 'selesai'
             AND t.tanggal_waktu >= $1::date::timestamp AT TIME ZONE $3
             AND t.tanggal_waktu < ($2::date + 1)::timestamp AT TIME ZONE $3
             AND ($4::bigint IS NULL OR t.id_pengguna = $4)"#,
        dari,
        sampai,
        tz,
        kasir,
    )
    .fetch_one(&mut *conn)
    .await?;

    let baris = sqlx::query!(
        r#"SELECT coalesce(sum(d.jumlah), 0)::bigint AS "unit!",
                  coalesce(sum(d.jumlah * d.harga_satuan), 0)::bigint AS "kotor!",
                  coalesce(sum(d.jumlah * d.harga_pokok), 0)::bigint AS "hpp!"
           FROM detail_transaksi d JOIN transaksi t ON t.id_transaksi = d.id_transaksi
           WHERE t.status = 'selesai'
             AND t.tanggal_waktu >= $1::date::timestamp AT TIME ZONE $3
             AND t.tanggal_waktu < ($2::date + 1)::timestamp AT TIME ZONE $3
             AND ($4::bigint IS NULL OR t.id_pengguna = $4)"#,
        dari,
        sampai,
        tz,
        kasir,
    )
    .fetch_one(&mut *conn)
    .await?;

    let retur = sqlx::query_scalar!(
        r#"SELECT coalesce(sum(r.total_pengembalian), 0)::bigint AS "retur!"
           FROM retur r JOIN transaksi t ON t.id_transaksi = r.id_transaksi
           WHERE r.status = 'disetujui' AND r.jenis = 'retur'
             AND r.waktu_keputusan >= $1::date::timestamp AT TIME ZONE $3
             AND r.waktu_keputusan < ($2::date + 1)::timestamp AT TIME ZONE $3
             AND ($4::bigint IS NULL OR t.id_pengguna = $4)"#,
        dari,
        sampai,
        tz,
        kasir,
    )
    .fetch_one(&mut *conn)
    .await?;

    let hpp_retur = sqlx::query_scalar!(
        r#"SELECT coalesce(sum(dr.jumlah * d.harga_pokok), 0)::bigint AS "hpp!"
           FROM detail_retur dr
           JOIN retur r ON r.id_retur = dr.id_retur
           JOIN detail_transaksi d ON d.id_detail = dr.id_detail
           JOIN transaksi t ON t.id_transaksi = r.id_transaksi
           WHERE r.status = 'disetujui' AND r.jenis = 'retur'
             AND r.waktu_keputusan >= $1::date::timestamp AT TIME ZONE $3
             AND r.waktu_keputusan < ($2::date + 1)::timestamp AT TIME ZONE $3
             AND ($4::bigint IS NULL OR t.id_pengguna = $4)"#,
        dari,
        sampai,
        tz,
        kasir,
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(Ringkasan {
        transaksi: header.transaksi,
        unit: baris.unit,
        kotor: baris.kotor,
        diskon: baris.kotor - header.total,
        retur,
        bersih: header.total - retur,
        rata_rata: if header.transaksi > 0 { header.total / header.transaksi } else { 0 },
        hpp: Some(baris.hpp - hpp_retur),
    })
}

async fn tren(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate, kasir: Option<i64>, kelompok: Kelompok) -> AppResult<Vec<TitikTren>> {
    let satuan = match kelompok {
        Kelompok::Hari => "day",
        Kelompok::Bulan => "month",
    };
    let rows = sqlx::query_as!(
        TitikTren,
        r#"SELECT s.t::date AS "tanggal!", coalesce(x.total, 0)::bigint AS "total!", coalesce(x.transaksi, 0)::bigint AS "transaksi!"
           FROM generate_series(date_trunc($5, $1::date::timestamp), $2::date::timestamp,
                                CASE WHEN $5 = 'month' THEN interval '1 month' ELSE interval '1 day' END) AS s(t)
           LEFT JOIN (
             SELECT date_trunc($5, t.tanggal_waktu AT TIME ZONE $3)::date AS kunci, sum(t.total) AS total, count(*) AS transaksi
             FROM transaksi t
             WHERE t.status = 'selesai'
               AND t.tanggal_waktu >= $1::date::timestamp AT TIME ZONE $3
               AND t.tanggal_waktu < ($2::date + 1)::timestamp AT TIME ZONE $3
               AND ($4::bigint IS NULL OR t.id_pengguna = $4)
             GROUP BY 1
           ) x ON x.kunci = s.t::date
           ORDER BY 1"#,
        dari,
        sampai,
        tz,
        kasir,
        satuan,
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

/// Nilai bersih per varian dengan pembagian diskon transaksi yang sama persis
/// dengan perhitungan retur: bagian proporsional, sisa ke baris bernilai terbesar.
struct NilaiVarian {
    id_produk: i64,
    kode: String,
    nama_produk: String,
    kategori: String,
    ukuran: String,
    unit: i64,
    nilai: i64,
}

async fn nilai_per_varian(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate, kasir: Option<i64>) -> AppResult<Vec<NilaiVarian>> {
    let rows = sqlx::query!(
        r#"WITH baris AS (
             SELECT d.id_detail, d.id_transaksi, d.id_varian, d.jumlah, d.subtotal, t.diskon,
                    CASE WHEN t.subtotal > 0 THEN t.diskon * d.subtotal / t.subtotal ELSE 0 END AS bagian,
                    row_number() OVER (PARTITION BY d.id_transaksi ORDER BY d.subtotal DESC, d.id_detail DESC) AS urut
             FROM detail_transaksi d JOIN transaksi t ON t.id_transaksi = d.id_transaksi
             WHERE t.status = 'selesai'
               AND t.tanggal_waktu >= $1::date::timestamp AT TIME ZONE $3
               AND t.tanggal_waktu < ($2::date + 1)::timestamp AT TIME ZONE $3
               AND ($4::bigint IS NULL OR t.id_pengguna = $4)
           ), bersih AS (
             SELECT b.id_varian, b.jumlah,
                    b.subtotal - b.bagian
                      - CASE WHEN b.urut = 1 THEN b.diskon - sum(b.bagian) OVER (PARTITION BY b.id_transaksi) ELSE 0 END AS nilai
             FROM baris b
           )
           SELECT p.id_produk, p.kode, p.nama AS nama_produk, k.nama_kategori AS kategori, upper(v.ukuran) AS "ukuran!",
                  sum(x.jumlah)::bigint AS "unit!", sum(x.nilai)::bigint AS "nilai!"
           FROM bersih x
           JOIN varian v ON v.id_varian = x.id_varian
           JOIN produk p ON p.id_produk = v.id_produk
           JOIN kategori k ON k.id_kategori = p.id_kategori
           GROUP BY p.id_produk, p.kode, p.nama, k.nama_kategori, upper(v.ukuran)"#,
        dari,
        sampai,
        tz,
        kasir,
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| NilaiVarian {
            id_produk: r.id_produk,
            kode: r.kode,
            nama_produk: r.nama_produk,
            kategori: r.kategori,
            ukuran: r.ukuran,
            unit: r.unit,
            nilai: r.nilai,
        })
        .collect())
}

/// Mengelompokkan nilai per varian menjadi baris rincian, urut penjualan terbesar.
fn kelompokkan(data: &[NilaiVarian], kunci: impl Fn(&NilaiVarian) -> (String, String)) -> Vec<BarisRincian> {
    let mut peta: BTreeMap<String, BarisRincian> = BTreeMap::new();
    for d in data {
        let (k, label) = kunci(d);
        let baris = peta.entry(k.clone()).or_insert(BarisRincian { kunci: k, label, unit: 0, penjualan: 0 });
        baris.unit += d.unit;
        baris.penjualan += d.nilai;
    }
    let mut hasil: Vec<BarisRincian> = peta.into_values().collect();
    hasil.sort_by(|a, b| b.penjualan.cmp(&a.penjualan).then(b.unit.cmp(&a.unit)).then(a.label.cmp(&b.label)));
    hasil
}

const URUTAN_UKURAN: [&str; 7] = ["XS", "S", "M", "L", "XL", "XXL", "XXXL"];

fn rincian_produk(data: &[NilaiVarian]) -> Vec<BarisRincian> {
    kelompokkan(data, |d| (d.id_produk.to_string(), format!("{} ({})", d.nama_produk, d.kode)))
}

fn terlaris(data: &[NilaiVarian], n: usize) -> Vec<BarisRincian> {
    let mut hasil = rincian_produk(data);
    hasil.sort_by(|a, b| b.unit.cmp(&a.unit).then(b.penjualan.cmp(&a.penjualan)));
    hasil.truncate(n);
    hasil
}

async fn per_metode(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate, kasir: Option<i64>) -> AppResult<Vec<BarisMetode>> {
    let jual = sqlx::query!(
        r#"SELECT t.metode_bayar, count(*) AS "transaksi!", sum(t.total)::bigint AS "penjualan!"
           FROM transaksi t
           WHERE t.status = 'selesai'
             AND t.tanggal_waktu >= $1::date::timestamp AT TIME ZONE $3
             AND t.tanggal_waktu < ($2::date + 1)::timestamp AT TIME ZONE $3
             AND ($4::bigint IS NULL OR t.id_pengguna = $4)
           GROUP BY t.metode_bayar"#,
        dari,
        sampai,
        tz,
        kasir,
    )
    .fetch_all(&mut *conn)
    .await?;
    let retur = sqlx::query!(
        r#"SELECT r.metode_pengembalian AS "metode!", sum(r.total_pengembalian)::bigint AS "retur!"
           FROM retur r JOIN transaksi t ON t.id_transaksi = r.id_transaksi
           WHERE r.status = 'disetujui' AND r.jenis = 'retur'
             AND r.waktu_keputusan >= $1::date::timestamp AT TIME ZONE $3
             AND r.waktu_keputusan < ($2::date + 1)::timestamp AT TIME ZONE $3
             AND ($4::bigint IS NULL OR t.id_pengguna = $4)
           GROUP BY r.metode_pengembalian"#,
        dari,
        sampai,
        tz,
        kasir,
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(["tunai", "transfer", "qris"]
        .into_iter()
        .map(|m| {
            let j = jual.iter().find(|j| j.metode_bayar == m);
            BarisMetode {
                metode: m.to_string(),
                transaksi: j.map(|j| j.transaksi).unwrap_or(0),
                penjualan: j.map(|j| j.penjualan).unwrap_or(0),
                retur: retur.iter().find(|r| r.metode == m).map(|r| r.retur).unwrap_or(0),
            }
        })
        .collect())
}

async fn per_kasir(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate) -> AppResult<Vec<BarisKasir>> {
    let rows = sqlx::query_as!(
        BarisKasir,
        r#"WITH jual AS (
             SELECT t.id_pengguna, count(*) AS transaksi, sum(t.total) AS penjualan
             FROM transaksi t
             WHERE t.status = 'selesai'
               AND t.tanggal_waktu >= $1::date::timestamp AT TIME ZONE $3
               AND t.tanggal_waktu < ($2::date + 1)::timestamp AT TIME ZONE $3
             GROUP BY t.id_pengguna
           ), kembali AS (
             SELECT t.id_pengguna, sum(r.total_pengembalian) AS retur
             FROM retur r JOIN transaksi t ON t.id_transaksi = r.id_transaksi
             WHERE r.status = 'disetujui' AND r.jenis = 'retur'
               AND r.waktu_keputusan >= $1::date::timestamp AT TIME ZONE $3
               AND r.waktu_keputusan < ($2::date + 1)::timestamp AT TIME ZONE $3
             GROUP BY t.id_pengguna
           )
           SELECT p.id_pengguna, p.nama, coalesce(j.transaksi, 0)::bigint AS "transaksi!",
                  coalesce(j.penjualan, 0)::bigint AS "penjualan!", coalesce(k.retur, 0)::bigint AS "retur!"
           FROM pengguna p
           LEFT JOIN jual j ON j.id_pengguna = p.id_pengguna
           LEFT JOIN kembali k ON k.id_pengguna = p.id_pengguna
           WHERE j.id_pengguna IS NOT NULL OR k.id_pengguna IS NOT NULL
           ORDER BY coalesce(j.penjualan, 0) DESC, p.nama"#,
        dari,
        sampai,
        tz,
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows)
}

async fn muat_penjualan(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate, kasir: Option<i64>, admin: bool) -> AppResult<LaporanPenjualan> {
    let kelompok = periode::kelompok(dari, sampai);
    let mut r = ringkasan(conn, tz, dari, sampai, kasir).await?;
    if !admin {
        // Kasir tidak melihat harga pokok (Tabel 2).
        r.hpp = None;
    }
    let data = nilai_per_varian(conn, tz, dari, sampai, kasir).await?;
    let mut per_ukuran = kelompokkan(&data, |d| (d.ukuran.clone(), d.ukuran.clone()));
    per_ukuran.sort_by_key(|b| (URUTAN_UKURAN.iter().position(|u| *u == b.kunci).unwrap_or(URUTAN_UKURAN.len()), b.kunci.clone()));

    let lambat = if admin {
        let hari = pengaturan::angka(&mut *conn, pengaturan::HARI_LAMBAT_TERJUAL, 30).await?;
        Some(stok::lambat_terjual(conn, hari).await?)
    } else {
        None
    };

    Ok(LaporanPenjualan {
        dari,
        sampai,
        kelompok: kelompok.as_str(),
        ringkasan: r,
        tren: tren(conn, tz, dari, sampai, kasir, kelompok).await?,
        per_produk: rincian_produk(&data),
        per_kategori: kelompokkan(&data, |d| (d.kategori.clone(), d.kategori.clone())),
        per_ukuran,
        per_metode: per_metode(conn, tz, dari, sampai, kasir).await?,
        per_kasir: if admin && kasir.is_none() { Some(per_kasir(conn, tz, dari, sampai).await?) } else { None },
        terlaris: terlaris(&data, 10),
        lambat_terjual: lambat,
    })
}

async fn penjualan(pengguna: PenggunaLogin, State(state): State<AppState>, Query(q): Query<CariLaporan>) -> AppResult<Json<LaporanPenjualan>> {
    let (dari, sampai) = rentang(&q)?;
    let admin = pengguna.is_admin();
    // Kasir hanya melihat penjualannya sendiri (Tabel 2).
    let kasir = if admin { q.id_kasir } else { Some(pengguna.id_pengguna) };
    let mut conn = state.pool.acquire().await?;
    Ok(Json(muat_penjualan(&mut conn, &state.config.zona_waktu, dari, sampai, kasir, admin).await?))
}

async fn periode_laba_rugi(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate) -> AppResult<PeriodeLabaRugi> {
    let r = ringkasan(conn, tz, dari, sampai, None).await?;
    let biaya_per_kategori = sqlx::query_as!(
        BiayaKategori,
        r#"SELECT kategori, sum(nominal)::bigint AS "nominal!"
           FROM biaya_operasional WHERE tanggal BETWEEN $1 AND $2
           GROUP BY kategori ORDER BY sum(nominal) DESC"#,
        dari,
        sampai,
    )
    .fetch_all(&mut *conn)
    .await?;
    let biaya: i64 = biaya_per_kategori.iter().map(|b| b.nominal).sum();
    let hpp = r.hpp.unwrap_or(0);
    let laba_kotor = r.bersih - hpp;
    let laba_bersih = laba_kotor - biaya;
    Ok(PeriodeLabaRugi {
        dari,
        sampai,
        kotor: r.kotor,
        diskon: r.diskon,
        retur: r.retur,
        bersih: r.bersih,
        hpp,
        laba_kotor,
        biaya,
        laba_bersih,
        margin_kotor_bps: periode::bps(laba_kotor, r.bersih),
        margin_bersih_bps: periode::bps(laba_bersih, r.bersih),
        biaya_per_kategori,
    })
}

async fn muat_laba_rugi(conn: &mut PgConnection, tz: &str, dari: NaiveDate, sampai: NaiveDate) -> AppResult<LabaRugi> {
    let (dari_lalu, sampai_lalu) = periode::sebelumnya(dari, sampai);
    Ok(LabaRugi {
        sekarang: periode_laba_rugi(conn, tz, dari, sampai).await?,
        sebelumnya: periode_laba_rugi(conn, tz, dari_lalu, sampai_lalu).await?,
    })
}

async fn laba_rugi(_: Admin, State(state): State<AppState>, Query(q): Query<CariLaporan>) -> AppResult<Json<LabaRugi>> {
    let (dari, sampai) = rentang(&q)?;
    let mut conn = state.pool.acquire().await?;
    Ok(Json(muat_laba_rugi(&mut conn, &state.config.zona_waktu, dari, sampai).await?))
}

async fn muat_stok(conn: &mut PgConnection) -> AppResult<LaporanStok> {
    let varian = sqlx::query_as!(
        BarisStok,
        r#"SELECT v.id_varian, p.id_produk, p.kode, p.nama AS nama_produk, k.nama_kategori AS kategori, v.ukuran, v.warna, v.sku,
                  v.stok, v.stok_minimum, p.harga_beli, (v.stok * p.harga_beli)::bigint AS "nilai!"
           FROM varian v
           JOIN produk p ON p.id_produk = v.id_produk
           JOIN kategori k ON k.id_kategori = p.id_kategori
           WHERE v.aktif AND p.aktif
           ORDER BY k.nama_kategori, p.nama,
                    array_position(ARRAY['XS','S','M','L','XL','XXL','XXXL'], upper(v.ukuran)) NULLS LAST, v.ukuran, v.warna"#
    )
    .fetch_all(&mut *conn)
    .await?;
    let total_unit = varian.iter().map(|v| v.stok as i64).sum();
    let total_nilai = varian.iter().map(|v| v.nilai).sum();
    Ok(LaporanStok { varian, total_unit, total_nilai })
}

async fn laporan_stok(_: Admin, State(state): State<AppState>) -> AppResult<Json<LaporanStok>> {
    let mut conn = state.pool.acquire().await?;
    Ok(Json(muat_stok(&mut conn).await?))
}

// ---------- Ekspor CSV (KF-16) ----------

/// Sel CSV. Teks yang diawali = + - @ diberi tanda kutip tunggal agar tidak
/// dijalankan sebagai rumus oleh aplikasi lembar kerja; angka dibiarkan apa adanya.
enum Sel {
    Teks(String),
    Angka(i64),
}

fn teks(s: impl Into<String>) -> Sel {
    Sel::Teks(s.into())
}

fn csv(baris: Vec<Vec<Sel>>) -> String {
    let mut keluaran = String::from("\u{feff}");
    for b in baris {
        let sel: Vec<String> = b
            .into_iter()
            .map(|s| match s {
                Sel::Angka(n) => n.to_string(),
                Sel::Teks(t) => {
                    let t = if t.starts_with(['=', '+', '-', '@']) { format!("'{t}") } else { t };
                    if t.contains([';', '"', '\n', '\r']) { format!("\"{}\"", t.replace('"', "\"\"")) } else { t }
                }
            })
            .collect();
        keluaran.push_str(&sel.join(";"));
        keluaran.push_str("\r\n");
    }
    keluaran
}

fn unduhan(nama_berkas: String, isi: String) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{nama_berkas}\"")),
        ],
        isi,
    )
        .into_response()
}

async fn penjualan_csv(_: Admin, State(state): State<AppState>, Query(q): Query<CariLaporan>) -> AppResult<Response> {
    let (dari, sampai) = rentang(&q)?;
    let mut conn = state.pool.acquire().await?;
    let l = muat_penjualan(&mut conn, &state.config.zona_waktu, dari, sampai, q.id_kasir, true).await?;
    let r = &l.ringkasan;
    let mut b: Vec<Vec<Sel>> = vec![
        vec![teks("Laporan penjualan"), teks(format!("{dari} s.d. {sampai}"))],
        vec![],
        vec![teks("Ringkasan"), teks("Nilai")],
        vec![teks("Jumlah transaksi"), Sel::Angka(r.transaksi)],
        vec![teks("Barang terjual"), Sel::Angka(r.unit)],
        vec![teks("Penjualan kotor"), Sel::Angka(r.kotor)],
        vec![teks("Diskon"), Sel::Angka(r.diskon)],
        vec![teks("Retur"), Sel::Angka(r.retur)],
        vec![teks("Penjualan bersih"), Sel::Angka(r.bersih)],
        vec![teks("Rata-rata per transaksi"), Sel::Angka(r.rata_rata)],
        vec![],
        vec![teks("Per produk"), teks("Unit"), teks("Penjualan")],
    ];
    b.extend(l.per_produk.iter().map(|p| vec![teks(&p.label), Sel::Angka(p.unit), Sel::Angka(p.penjualan)]));
    b.push(vec![]);
    b.push(vec![teks("Per metode bayar"), teks("Transaksi"), teks("Penjualan"), teks("Retur")]);
    b.extend(l.per_metode.iter().map(|m| vec![teks(&m.metode), Sel::Angka(m.transaksi), Sel::Angka(m.penjualan), Sel::Angka(m.retur)]));
    b.push(vec![]);
    b.push(vec![teks("Tren"), teks("Transaksi"), teks("Total")]);
    b.extend(l.tren.iter().map(|t| vec![teks(t.tanggal.to_string()), Sel::Angka(t.transaksi), Sel::Angka(t.total)]));
    Ok(unduhan(format!("laporan-penjualan-{dari}_{sampai}.csv"), csv(b)))
}

async fn laba_rugi_csv(_: Admin, State(state): State<AppState>, Query(q): Query<CariLaporan>) -> AppResult<Response> {
    let (dari, sampai) = rentang(&q)?;
    let mut conn = state.pool.acquire().await?;
    let l = muat_laba_rugi(&mut conn, &state.config.zona_waktu, dari, sampai).await?;
    let (a, z) = (&l.sekarang, &l.sebelumnya);
    let pos = |nama: &str, x: i64, y: i64| vec![teks(nama), Sel::Angka(x), Sel::Angka(y)];
    let margin = |v: Option<i64>| v.map(|b| format!("{:.2}", b as f64 / 100.0)).unwrap_or_default();
    let mut b = vec![
        vec![teks("Laporan laba rugi sederhana")],
        vec![teks("Pos"), teks(format!("{} s.d. {}", a.dari, a.sampai)), teks(format!("{} s.d. {}", z.dari, z.sampai))],
        pos("Penjualan kotor", a.kotor, z.kotor),
        pos("Diskon", a.diskon, z.diskon),
        pos("Retur", a.retur, z.retur),
        pos("Penjualan bersih", a.bersih, z.bersih),
        pos("Harga pokok penjualan", a.hpp, z.hpp),
        pos("Laba kotor", a.laba_kotor, z.laba_kotor),
        pos("Biaya operasional", a.biaya, z.biaya),
        pos("Laba bersih", a.laba_bersih, z.laba_bersih),
        vec![teks("Margin laba kotor (%)"), teks(margin(a.margin_kotor_bps)), teks(margin(z.margin_kotor_bps))],
        vec![teks("Margin laba bersih (%)"), teks(margin(a.margin_bersih_bps)), teks(margin(z.margin_bersih_bps))],
        vec![],
        vec![teks("Biaya per kategori"), teks("Periode ini")],
    ];
    b.extend(a.biaya_per_kategori.iter().map(|k| vec![teks(&k.kategori), Sel::Angka(k.nominal)]));
    Ok(unduhan(format!("laporan-laba-rugi-{dari}_{sampai}.csv"), csv(b)))
}

async fn laporan_stok_csv(_: Admin, State(state): State<AppState>) -> AppResult<Response> {
    let mut conn = state.pool.acquire().await?;
    let l = muat_stok(&mut conn).await?;
    let tanggal = sqlx::query_scalar!(r#"SELECT (now() AT TIME ZONE $1)::date AS "hari!""#, state.config.zona_waktu)
        .fetch_one(&mut *conn)
        .await?;
    let mut b = vec![vec![
        teks("Kode"),
        teks("Produk"),
        teks("Kategori"),
        teks("Ukuran"),
        teks("Warna"),
        teks("SKU"),
        teks("Stok"),
        teks("Stok minimum"),
        teks("Harga beli"),
        teks("Nilai persediaan"),
    ]];
    b.extend(l.varian.iter().map(|v| {
        vec![
            teks(&v.kode),
            teks(&v.nama_produk),
            teks(&v.kategori),
            teks(&v.ukuran),
            teks(&v.warna),
            teks(&v.sku),
            Sel::Angka(v.stok as i64),
            Sel::Angka(v.stok_minimum as i64),
            Sel::Angka(v.harga_beli),
            Sel::Angka(v.nilai),
        ]
    }));
    b.push(vec![teks("Total"), teks(""), teks(""), teks(""), teks(""), teks(""), Sel::Angka(l.total_unit), teks(""), teks(""), Sel::Angka(l.total_nilai)]);
    Ok(unduhan(format!("laporan-stok-{tanggal}.csv"), csv(b)))
}

// ---------- Dasbor pemilik ----------

async fn dasbor(_: Admin, State(state): State<AppState>) -> AppResult<Json<Dasbor>> {
    let tz = state.config.zona_waktu.clone();
    let mut conn = state.pool.acquire().await?;
    let hari_ini = sqlx::query_scalar!(r#"SELECT (now() AT TIME ZONE $1)::date AS "hari!""#, tz)
        .fetch_one(&mut *conn)
        .await?;
    let kemarin = hari_ini - Duration::days(1);

    let mut ringkasan_hari_ini = ringkasan(&mut conn, &tz, hari_ini, hari_ini, None).await?;
    let mut ringkasan_kemarin = ringkasan(&mut conn, &tz, kemarin, kemarin, None).await?;
    ringkasan_hari_ini.hpp = None;
    ringkasan_kemarin.hpp = None;
    let tren = tren(&mut conn, &tz, hari_ini - Duration::days(13), hari_ini, None, Kelompok::Hari).await?;
    let data_7 = nilai_per_varian(&mut conn, &tz, hari_ini - Duration::days(6), hari_ini, None).await?;

    let menipis = sqlx::query_as!(
        VarianStok,
        r#"SELECT v.id_varian, v.id_produk, p.nama AS nama_produk, p.kode, v.sku, v.ukuran, v.warna, v.stok, v.stok_minimum,
                  NULL::timestamptz AS terakhir_terjual
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk
           WHERE v.aktif AND p.aktif AND v.stok <= v.stok_minimum
           ORDER BY v.stok - v.stok_minimum, p.nama, v.ukuran, v.warna"#
    )
    .fetch_all(&mut *conn)
    .await?;
    let pengajuan_menunggu = sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM retur WHERE status = 'menunggu'"#)
        .fetch_one(&mut *conn)
        .await?;
    let sesi_aktif = sqlx::query_as!(
        SesiRingkas,
        r#"SELECT s.id_sesi, p.nama AS nama_kasir, s.waktu_buka, s.waktu_tutup, s.selisih, s.alasan_selisih
           FROM sesi_kasir s JOIN pengguna p ON p.id_pengguna = s.id_pengguna
           WHERE s.waktu_tutup IS NULL ORDER BY s.waktu_buka"#
    )
    .fetch_all(&mut *conn)
    .await?;
    let sesi_terakhir = sqlx::query_as!(
        SesiRingkas,
        r#"SELECT s.id_sesi, p.nama AS nama_kasir, s.waktu_buka, s.waktu_tutup, s.selisih, s.alasan_selisih
           FROM sesi_kasir s JOIN pengguna p ON p.id_pengguna = s.id_pengguna
           WHERE s.waktu_tutup IS NOT NULL ORDER BY s.waktu_tutup DESC LIMIT 5"#
    )
    .fetch_all(&mut *conn)
    .await?;

    Ok(Json(Dasbor {
        hari_ini,
        ringkasan_hari_ini,
        ringkasan_kemarin,
        tren,
        terlaris_7_hari: terlaris(&data_7, 5),
        stok_menipis: menipis.len() as i64,
        varian_menipis: menipis.into_iter().take(5).collect(),
        pengajuan_menunggu,
        sesi_aktif,
        sesi_terakhir,
    }))
}
