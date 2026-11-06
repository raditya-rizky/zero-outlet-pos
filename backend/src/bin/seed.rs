//! Mengisi data contoh sintetis berpola toko pakaian ritel (risiko nomor 2 pada
//! Tabel 8: daftar produk dari toko belum diperoleh). Aman dijalankan berulang;
//! data hanya diisi bila tabel produk masih kosong.
//!
//! Jalankan: `cargo run --bin seed`

use sqlx::{PgPool, postgres::PgPoolOptions};
use zero_outlet_pos::services::{
    pengguna::pastikan_admin,
    sandi,
    stok::{self, Mutasi},
};

struct ContohProduk {
    kode: &'static str,
    nama: &'static str,
    kategori: &'static str,
    harga_beli: i64,
    harga_jual: i64,
    ukuran: &'static [&'static str],
    warna: &'static [&'static str],
}

const UKURAN_BAJU: &[&str] = &["S", "M", "L", "XL"];
const UKURAN_CELANA: &[&str] = &["28", "30", "32", "34"];
const ALL_SIZE: &[&str] = &["ALL SIZE"];

const PRODUK: &[ContohProduk] = &[
    ContohProduk { kode: "KAOS-POLOS", nama: "Kaos Polos Cotton Combed 30s", kategori: "Atasan", harga_beli: 38_000, harga_jual: 65_000, ukuran: UKURAN_BAJU, warna: &["Hitam", "Putih", "Navy"] },
    ContohProduk { kode: "KAOS-OVS", nama: "Kaos Oversize Grafis", kategori: "Atasan", harga_beli: 55_000, harga_jual: 95_000, ukuran: UKURAN_BAJU, warna: &["Hitam", "Abu"] },
    ContohProduk { kode: "POLO", nama: "Kaos Polo Pique", kategori: "Atasan", harga_beli: 70_000, harga_jual: 120_000, ukuran: UKURAN_BAJU, warna: &["Putih", "Maroon", "Hijau Botol"] },
    ContohProduk { kode: "KMJ-FLN", nama: "Kemeja Flanel Kotak", kategori: "Atasan", harga_beli: 85_000, harga_jual: 150_000, ukuran: UKURAN_BAJU, warna: &["Merah", "Biru"] },
    ContohProduk { kode: "KMJ-LNN", nama: "Kemeja Linen Lengan Pendek", kategori: "Atasan", harga_beli: 90_000, harga_jual: 165_000, ukuran: UKURAN_BAJU, warna: &["Krem", "Putih"] },
    ContohProduk { kode: "KMJ-OXF", nama: "Kemeja Oxford Lengan Panjang", kategori: "Atasan", harga_beli: 95_000, harga_jual: 175_000, ukuran: UKURAN_BAJU, warna: &["Biru Muda", "Putih"] },
    ContohProduk { kode: "BLS-WNT", nama: "Blouse Wanita Ruffle", kategori: "Atasan", harga_beli: 65_000, harga_jual: 115_000, ukuran: &["S", "M", "L"], warna: &["Dusty Pink", "Sage", "Hitam"] },
    ContohProduk { kode: "CRP-TOP", nama: "Crop Top Rajut", kategori: "Atasan", harga_beli: 45_000, harga_jual: 85_000, ukuran: &["S", "M", "L"], warna: &["Putih", "Mocca"] },
    ContohProduk { kode: "JNS-SLM", nama: "Celana Jeans Slim Fit", kategori: "Bawahan", harga_beli: 120_000, harga_jual: 210_000, ukuran: UKURAN_CELANA, warna: &["Biru Tua", "Hitam"] },
    ContohProduk { kode: "CHN-PJG", nama: "Celana Chino Panjang", kategori: "Bawahan", harga_beli: 95_000, harga_jual: 175_000, ukuran: UKURAN_CELANA, warna: &["Khaki", "Navy", "Hitam"] },
    ContohProduk { kode: "CLN-PDK", nama: "Celana Pendek Cargo", kategori: "Bawahan", harga_beli: 60_000, harga_jual: 110_000, ukuran: UKURAN_CELANA, warna: &["Army", "Hitam"] },
    ContohProduk { kode: "JGR", nama: "Celana Jogger Fleece", kategori: "Bawahan", harga_beli: 70_000, harga_jual: 125_000, ukuran: UKURAN_BAJU, warna: &["Abu", "Hitam"] },
    ContohProduk { kode: "KLT-PLT", nama: "Kulot Plisket Wanita", kategori: "Bawahan", harga_beli: 55_000, harga_jual: 99_000, ukuran: ALL_SIZE, warna: &["Hitam", "Mocca", "Navy", "Sage"] },
    ContohProduk { kode: "RK-SPN", nama: "Rok Span Midi", kategori: "Bawahan", harga_beli: 60_000, harga_jual: 105_000, ukuran: &["S", "M", "L"], warna: &["Hitam", "Krem"] },
    ContohProduk { kode: "HDY", nama: "Hoodie Polos Fleece", kategori: "Outerwear", harga_beli: 110_000, harga_jual: 195_000, ukuran: UKURAN_BAJU, warna: &["Hitam", "Abu", "Navy"] },
    ContohProduk { kode: "JKT-DNM", nama: "Jaket Denim", kategori: "Outerwear", harga_beli: 150_000, harga_jual: 265_000, ukuran: UKURAN_BAJU, warna: &["Biru"] },
    ContohProduk { kode: "JKT-BMB", nama: "Jaket Bomber", kategori: "Outerwear", harga_beli: 140_000, harga_jual: 245_000, ukuran: UKURAN_BAJU, warna: &["Hitam", "Army"] },
    ContohProduk { kode: "CRD-RJT", nama: "Cardigan Rajut", kategori: "Outerwear", harga_beli: 80_000, harga_jual: 145_000, ukuran: ALL_SIZE, warna: &["Krem", "Coklat", "Hitam"] },
    ContohProduk { kode: "GMS-SYR", nama: "Gamis Syari Polos", kategori: "Busana Muslim", harga_beli: 130_000, harga_jual: 225_000, ukuran: &["M", "L", "XL"], warna: &["Hitam", "Navy", "Dusty Pink"] },
    ContohProduk { kode: "TNK-WNT", nama: "Tunik Wanita Motif", kategori: "Busana Muslim", harga_beli: 85_000, harga_jual: 150_000, ukuran: &["M", "L", "XL"], warna: &["Sage", "Mocca"] },
    ContohProduk { kode: "KK-PRIA", nama: "Baju Koko Pria", kategori: "Busana Muslim", harga_beli: 90_000, harga_jual: 160_000, ukuran: UKURAN_BAJU, warna: &["Putih", "Abu", "Hitam"] },
    ContohProduk { kode: "HJB-SGI", nama: "Hijab Segi Empat Voal", kategori: "Busana Muslim", harga_beli: 25_000, harga_jual: 45_000, ukuran: ALL_SIZE, warna: &["Hitam", "Putih", "Mocca", "Sage", "Dusty Pink"] },
    ContohProduk { kode: "TOPI", nama: "Topi Baseball", kategori: "Aksesori", harga_beli: 30_000, harga_jual: 55_000, ukuran: ALL_SIZE, warna: &["Hitam", "Putih", "Navy"] },
    ContohProduk { kode: "IKT-PNG", nama: "Ikat Pinggang Kulit", kategori: "Aksesori", harga_beli: 45_000, harga_jual: 85_000, ukuran: ALL_SIZE, warna: &["Hitam", "Coklat"] },
    ContohProduk { kode: "KS-KAKI", nama: "Kaos Kaki (3 pasang)", kategori: "Aksesori", harga_beli: 18_000, harga_jual: 35_000, ukuran: ALL_SIZE, warna: &["Hitam", "Putih"] },
];

/// Stok contoh yang bervariasi tetapi deterministik, termasuk beberapa varian
/// yang mendekati batas minimum agar peringatan stok dapat dicoba.
fn stok_contoh(i: usize, j: usize) -> i32 {
    match (i * 7 + j * 3) % 13 {
        0 => 1,
        1 => 2,
        n => n as i32 + 3,
    }
}

async fn isi_pengguna(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    for (nama, username) in [("Kasir Satu", "kasir1"), ("Kasir Dua", "kasir2")] {
        let ada = sqlx::query_scalar!(r#"SELECT EXISTS (SELECT 1 FROM pengguna WHERE username = $1) AS "ada!""#, username)
            .fetch_one(pool)
            .await?;
        if !ada {
            let hash = sandi::hash("kasir12345".into()).await?;
            sqlx::query!("INSERT INTO pengguna (nama, username, kata_sandi, peran) VALUES ($1, $2, $3, 'kasir')", nama, username, hash)
                .execute(pool)
                .await?;
            println!("  akun kasir: {username} / kasir12345");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let url = std::env::var("DATABASE_URL")?;
    let riwayat = argumen_riwayat()?;
    if riwayat.is_some() && !url.contains("_uji") {
        return Err("--riwayat hanya untuk basis data uji (DATABASE_URL harus mengandung \"_uji\")".into());
    }
    let pool = PgPoolOptions::new().connect(&url).await?;
    sqlx::migrate!().run(&pool).await?;

    let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
    let sandi_admin = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin12345".into());
    let nama_admin = std::env::var("ADMIN_NAMA").unwrap_or_else(|_| "Pemilik Toko".into());
    if pastikan_admin(&pool, &nama_admin, &username, &sandi_admin).await? {
        println!("  akun admin: {username} / {sandi_admin}");
    }
    isi_pengguna(&pool).await?;

    let sudah_ada = sqlx::query_scalar!(r#"SELECT EXISTS (SELECT 1 FROM produk) AS "ada!""#).fetch_one(&pool).await?;
    if sudah_ada {
        println!("Tabel produk sudah berisi data; pengisian produk dilewati.");
    } else {
        isi_produk(&pool).await?;
    }

    if let Some(hari) = riwayat {
        isi_riwayat(&pool, &url, hari).await?;
    }

    Ok(())
}

async fn isi_produk(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let id_admin = sqlx::query_scalar!("SELECT id_pengguna FROM pengguna WHERE peran = 'admin' ORDER BY id_pengguna LIMIT 1")
        .fetch_one(pool)
        .await?;

    let mut tx = pool.begin().await?;
    let mut jumlah_varian = 0;
    for (i, p) in PRODUK.iter().enumerate() {
        let id_kategori = sqlx::query_scalar!(
            r#"INSERT INTO kategori (nama_kategori) VALUES ($1)
               ON CONFLICT (nama_kategori) DO UPDATE SET nama_kategori = EXCLUDED.nama_kategori
               RETURNING id_kategori"#,
            p.kategori
        )
        .fetch_one(&mut *tx)
        .await?;

        let id_produk = sqlx::query_scalar!(
            "INSERT INTO produk (kode, nama, id_kategori, harga_beli, harga_jual) VALUES ($1, $2, $3, $4, $5) RETURNING id_produk",
            p.kode,
            p.nama,
            id_kategori,
            p.harga_beli,
            p.harga_jual,
        )
        .fetch_one(&mut *tx)
        .await?;

        let mut j = 0;
        for ukuran in p.ukuran {
            for warna in p.warna {
                let sku = format!("{}-{}-{}", p.kode, ukuran, warna).to_uppercase().split_whitespace().collect::<String>();
                let id_varian = sqlx::query_scalar!(
                    "INSERT INTO varian (id_produk, sku, ukuran, warna, stok_minimum) VALUES ($1, $2, $3, $4, 3) RETURNING id_varian",
                    id_produk,
                    sku,
                    ukuran,
                    warna,
                )
                .fetch_one(&mut *tx)
                .await?;
                stok::terapkan(
                    &mut tx,
                    Mutasi {
                        id_varian,
                        jenis: "penyesuaian",
                        jumlah: stok_contoh(i, j),
                        referensi: Some("STOK-AWAL"),
                        id_pengguna: id_admin,
                        keterangan: Some("Stok awal (data contoh sintetis)"),
                    },
                )
                .await?;
                j += 1;
                jumlah_varian += 1;
            }
        }
    }
    tx.commit().await?;
    println!("Data contoh diisi: {} produk, {jumlah_varian} varian.", PRODUK.len());
    Ok(())
}

/// `--riwayat <hari>`: jumlah hari riwayat penjualan sintetis yang dibuat.
fn argumen_riwayat() -> Result<Option<i64>, Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    match args.iter().position(|a| a == "--riwayat") {
        Some(i) => {
            let hari: i64 = args.get(i + 1).ok_or("--riwayat butuh jumlah hari, mis. --riwayat 90")?.parse()?;
            if !(1..=730).contains(&hari) {
                return Err("--riwayat harus 1 sampai 730 hari".into());
            }
            Ok(Some(hari))
        }
        None => Ok(None),
    }
}

/// Membuat riwayat penjualan SINTETIS (sekitar 40 transaksi per hari) untuk
/// uji kinerja laporan (Tabel 1: laporan bulanan <= 5 detik). Hanya untuk
/// basis data uji: kasirnya bernama "Data Contoh (sintetis)" dan faktur
/// berawalan UJI-, sehingga tidak pernah tertukar dengan data nyata toko.
/// Stok dan kartu stok sengaja tidak diubah.
async fn isi_riwayat(pool: &PgPool, url: &str, hari: i64) -> Result<(), Box<dyn std::error::Error>> {
    use rand::{Rng, SeedableRng, rngs::StdRng};

    debug_assert!(url.contains("_uji"));
    let hash = sandi::hash("sintetis12345".into()).await?;
    let id_kasir = sqlx::query_scalar!(
        r#"INSERT INTO pengguna (nama, username, kata_sandi, peran, aktif)
           VALUES ('Data Contoh (sintetis)', 'kasir_sintetis', $1, 'kasir', FALSE)
           ON CONFLICT (username) DO UPDATE SET nama = EXCLUDED.nama
           RETURNING id_pengguna"#,
        hash
    )
    .fetch_one(pool)
    .await?;

    let varian = sqlx::query!(
        r#"SELECT v.id_varian, p.nama || ' ' || v.ukuran || '/' || v.warna AS "nama!", p.harga_jual, p.harga_beli
           FROM varian v JOIN produk p ON p.id_produk = v.id_produk WHERE v.aktif AND p.aktif ORDER BY v.id_varian"#
    )
    .fetch_all(pool)
    .await?;
    if varian.is_empty() {
        return Err("Belum ada produk; jalankan seed tanpa --riwayat terlebih dahulu".into());
    }

    let mut acak = StdRng::seed_from_u64(20261101);
    let mut tx = pool.begin().await?;
    let mut jumlah = 0usize;
    for h in (1..=hari).rev() {
        let id_sesi = sqlx::query_scalar!(
            r#"INSERT INTO sesi_kasir (id_pengguna, waktu_buka, modal_awal)
               VALUES ($1, (CURRENT_DATE - $2::int)::timestamp + interval '9 hours', 0) RETURNING id_sesi"#,
            id_kasir,
            h as i32
        )
        .fetch_one(&mut *tx)
        .await?;

        let mut tunai = 0i64;
        let banyak = acak.gen_range(30..=50);
        for n in 0..banyak {
            let baris = acak.gen_range(1..=3);
            let mut item = Vec::new();
            for _ in 0..baris {
                let v = &varian[acak.gen_range(0..varian.len())];
                let q: i32 = acak.gen_range(1..=2);
                item.push((v.id_varian, v.nama.clone(), q, v.harga_jual, v.harga_beli));
            }
            let total: i64 = item.iter().map(|i| i.2 as i64 * i.3).sum();
            let metode = ["tunai", "tunai", "qris", "transfer"][acak.gen_range(0..4)];
            if metode == "tunai" {
                tunai += total;
            }
            let menit = 9 * 60 + n * (12 * 60) / banyak;
            let id_transaksi = sqlx::query_scalar!(
                r#"INSERT INTO transaksi (no_faktur, id_sesi, id_pengguna, tanggal_waktu, subtotal, diskon, total, metode_bayar, bayar, kembalian)
                   VALUES ('UJI-' || to_char(CURRENT_DATE - $1::int, 'YYMMDD') || '-' || lpad($2::text, 4, '0'), $3, $4,
                           (CURRENT_DATE - $1::int)::timestamp + make_interval(mins => $5), $6, 0, $6, $7, $6, 0)
                   RETURNING id_transaksi"#,
                h as i32,
                n.to_string(),
                id_sesi,
                id_kasir,
                menit as i32,
                total,
                metode,
            )
            .fetch_one(&mut *tx)
            .await?;
            for (id_varian, nama, q, harga, pokok) in item {
                sqlx::query!(
                    r#"INSERT INTO detail_transaksi (id_transaksi, id_varian, nama_barang, jumlah, harga_satuan, harga_pokok, diskon_item, subtotal)
                       VALUES ($1, $2, $3, $4, $5, $6, 0, $7)"#,
                    id_transaksi,
                    id_varian,
                    nama,
                    q,
                    harga,
                    pokok,
                    q as i64 * harga,
                )
                .execute(&mut *tx)
                .await?;
            }
            jumlah += 1;
        }
        sqlx::query!(
            r#"UPDATE sesi_kasir SET waktu_tutup = waktu_buka + interval '12 hours', kas_fisik = $2, kas_seharusnya = $2, selisih = 0
               WHERE id_sesi = $1"#,
            id_sesi,
            tunai
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    println!("Riwayat sintetis dibuat: {jumlah} transaksi selama {hari} hari (kasir \"Data Contoh (sintetis)\").");
    Ok(())
}
