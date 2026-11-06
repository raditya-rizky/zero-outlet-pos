//! Pengujian integrasi siklus 3: laporan penjualan (KF-15), ekspor CSV (KF-16),
//! biaya operasional (KF-17), laba rugi (KF-18), dan dasbor pemilik.
//!
//! Skenario memakai tanggal tetap Oktober-November 2025 (diundurkan lewat SQL)
//! agar tidak pernah bertumpuk dengan transaksi "hari ini".

mod common;

use axum::http::{Method, StatusCode};
use common::{Klien, Konteks, SANDI, buat_kasir, buat_produk, siapkan};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn jual(klien: &Klien, body: Value) -> Value {
    let (status, struk) = klien.post("/api/transaksi", body).await;
    assert_eq!(status, StatusCode::OK, "{struk}");
    struk
}

async fn undur_transaksi(k: &Konteks, struk: &Value, waktu: &str) {
    sqlx::query("UPDATE transaksi SET tanggal_waktu = $2::timestamptz WHERE id_transaksi = $1")
        .bind(struk["id_transaksi"].as_i64().unwrap())
        .bind(waktu)
        .execute(&k.pool)
        .await
        .unwrap();
}

async fn undur_retur(k: &Konteks, id_retur: i64, waktu: &str) {
    sqlx::query("UPDATE retur SET tanggal_waktu = $2::timestamptz, waktu_keputusan = $2::timestamptz WHERE id_retur = $1")
        .bind(id_retur)
        .bind(waktu)
        .execute(&k.pool)
        .await
        .unwrap();
}

/// Skenario tetap untuk Oktober-November 2025 (semua oleh admin, tanpa batas diskon):
/// - 10 Okt, tunai: A x2 diskon baris 20.000 + B(S) x1 = total 240.000, HPP 130.000
/// - 12 Okt, QRIS: A x1 + B(S) x2, diskon transaksi 10.001 = total 209.999, HPP 110.000
/// - 15 Okt: B(S) x1 dibatalkan (tidak dihitung)
/// - retur A x1 dari transaksi 12 Okt, disetujui 3 Nov: Rp95.455, HPP retur 50.000
/// - tukar B S -> M dari transaksi 10 Okt, disetujui 4 Nov (tanpa pengaruh)
/// - biaya Oktober: sewa 300.000 (1 Okt) dan listrik 50.000 (20 Okt)
async fn skenario(k: &Konteks) {
    let (_, a) = buat_produk(&k.admin, "A", 100_000, &[10]).await;
    let (_, b) = buat_produk(&k.admin, "B", 60_000, &[10, 10]).await;
    k.admin.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;

    let s1 = jual(
        &k.admin,
        json!({
            "items": [{ "id_varian": a[0], "jumlah": 2, "diskon_item": 20_000 }, { "id_varian": b[0], "jumlah": 1 }],
            "metode_bayar": "tunai", "bayar": 240_000
        }),
    )
    .await;
    assert_eq!(s1["total"], 240_000);
    let s2 = jual(
        &k.admin,
        json!({
            "items": [{ "id_varian": a[0], "jumlah": 1 }, { "id_varian": b[0], "jumlah": 2 }],
            "diskon_transaksi": 10_001, "metode_bayar": "qris"
        }),
    )
    .await;
    assert_eq!(s2["total"], 209_999);
    let s3 = jual(&k.admin, json!({ "items": [{ "id_varian": b[0], "jumlah": 1 }], "metode_bayar": "qris" })).await;
    let (_, batal) = k
        .admin
        .post("/api/pengajuan", json!({ "id_transaksi": s3["id_transaksi"], "jenis": "batal", "alasan": "Salah input" }))
        .await;
    assert_eq!(batal["status"], "disetujui");

    let baris_a = s2["items"].as_array().unwrap().iter().find(|i| i["id_varian"] == a[0]).unwrap()["id_detail"].clone();
    let (_, retur) = k
        .admin
        .post(
            "/api/pengajuan",
            json!({ "id_transaksi": s2["id_transaksi"], "jenis": "retur", "alasan": "Cacat", "items": [{ "id_detail": baris_a, "jumlah": 1 }] }),
        )
        .await;
    assert_eq!(retur["total_pengembalian"], 95_455);
    let baris_b = s1["items"].as_array().unwrap().iter().find(|i| i["id_varian"] == b[0]).unwrap()["id_detail"].clone();
    let (_, tukar) = k
        .admin
        .post(
            "/api/pengajuan",
            json!({
                "id_transaksi": s1["id_transaksi"], "jenis": "tukar", "alasan": "Kekecilan",
                "items": [{ "id_detail": baris_b, "jumlah": 1, "id_varian_pengganti": b[1] }]
            }),
        )
        .await;
    assert_eq!(tukar["status"], "disetujui");

    undur_transaksi(k, &s1, "2025-10-10 10:00+08").await;
    undur_transaksi(k, &s2, "2025-10-12 15:00+08").await;
    undur_transaksi(k, &s3, "2025-10-15 11:00+08").await;
    undur_retur(k, retur["id_retur"].as_i64().unwrap(), "2025-11-03 09:00+08").await;
    undur_retur(k, tukar["id_retur"].as_i64().unwrap(), "2025-11-04 09:00+08").await;

    for (tanggal, kategori, nominal) in [("2025-10-01", "Sewa tempat", 300_000), ("2025-10-20", "Listrik dan air", 50_000)] {
        let (status, _) = k.admin.post("/api/biaya", json!({ "tanggal": tanggal, "kategori": kategori, "nominal": nominal })).await;
        assert_eq!(status, StatusCode::OK);
    }
}

#[sqlx::test]
async fn laba_rugi_oktober_dan_retur_di_november(pool: PgPool) {
    let k = siapkan(pool).await;
    skenario(&k).await;

    let (status, l) = k.admin.get("/api/laporan/laba-rugi?dari=2025-10-01&sampai=2025-10-31").await;
    assert_eq!(status, StatusCode::OK, "{l}");
    let okt = &l["sekarang"];
    assert_eq!(okt["kotor"], 480_000);
    assert_eq!(okt["diskon"], 30_001);
    assert_eq!(okt["retur"], 0, "retur dihitung pada tanggal disetujui (November)");
    assert_eq!(okt["bersih"], 449_999);
    assert_eq!(okt["hpp"], 240_000);
    assert_eq!(okt["laba_kotor"], 209_999);
    assert_eq!(okt["biaya"], 350_000);
    assert_eq!(okt["laba_bersih"], -140_001);
    assert_eq!(okt["margin_kotor_bps"], 4_667);
    assert_eq!(okt["margin_bersih_bps"], -3_111);
    assert_eq!(okt["biaya_per_kategori"][0]["kategori"], "Sewa tempat");
    assert_eq!(l["sebelumnya"]["dari"], "2025-09-01");
    assert_eq!(l["sebelumnya"]["sampai"], "2025-09-30");
    assert_eq!(l["sebelumnya"]["bersih"], 0);

    let (_, l) = k.admin.get("/api/laporan/laba-rugi?dari=2025-11-01&sampai=2025-11-30").await;
    let nov = &l["sekarang"];
    assert_eq!(nov["kotor"], 0);
    assert_eq!(nov["retur"], 95_455);
    assert_eq!(nov["bersih"], -95_455);
    assert_eq!(nov["hpp"], -50_000);
    assert_eq!(nov["laba_kotor"], -45_455);
    assert!(nov["margin_kotor_bps"].is_null(), "margin kosong bila penjualan bersih tidak positif");
    // November penuh dibandingkan dengan Oktober penuh.
    assert_eq!(l["sebelumnya"]["sampai"], "2025-10-31");
    assert_eq!(l["sebelumnya"]["bersih"], 449_999);
}

#[sqlx::test]
async fn laporan_penjualan_rincian_berjumlah_tepat(pool: PgPool) {
    let k = siapkan(pool).await;
    skenario(&k).await;

    let (status, l) = k.admin.get("/api/laporan/penjualan?dari=2025-10-01&sampai=2025-10-31").await;
    assert_eq!(status, StatusCode::OK, "{l}");
    let r = &l["ringkasan"];
    assert_eq!(r["transaksi"], 2);
    assert_eq!(r["unit"], 6);
    assert_eq!(r["bersih"], 449_999);
    assert_eq!(r["rata_rata"], 224_999);
    assert_eq!(r["hpp"], 240_000);

    // Diskon transaksi 10.001 dibagi: A 4.545, B 5.456 (sisa ke subtotal terbesar).
    let produk = l["per_produk"].as_array().unwrap();
    assert_eq!(produk.len(), 2);
    assert_eq!(produk[0]["penjualan"], 275_455);
    assert_eq!(produk[1]["penjualan"], 174_544);
    let jumlah: i64 = produk.iter().map(|p| p["penjualan"].as_i64().unwrap()).sum();
    assert_eq!(jumlah, 449_999);
    assert_eq!(l["per_kategori"].as_array().unwrap().len(), 2);
    assert_eq!(l["per_ukuran"][0]["kunci"], "S");

    let metode = l["per_metode"].as_array().unwrap();
    assert_eq!(metode[0]["metode"], "tunai");
    assert_eq!(metode[0]["penjualan"], 240_000);
    assert_eq!(metode[2]["metode"], "qris");
    assert_eq!(metode[2]["penjualan"], 209_999);
    assert_eq!(l["terlaris"][0]["unit"], 3);
    assert_eq!(l["per_kasir"][0]["penjualan"], 449_999);

    let tren = l["tren"].as_array().unwrap();
    assert_eq!(tren.len(), 31);
    assert_eq!(tren[9]["tanggal"], "2025-10-10");
    assert_eq!(tren[9]["total"], 240_000);
    assert_eq!(tren[11]["total"], 209_999);
    assert_eq!(tren[14]["total"], 0, "transaksi batal tidak masuk tren");

    let (_, nov) = k.admin.get("/api/laporan/penjualan?dari=2025-11-01&sampai=2025-11-30").await;
    assert_eq!(nov["ringkasan"]["retur"], 95_455);
    assert_eq!(nov["per_metode"][2]["retur"], 95_455);

    // Rentang panjang memakai tren bulanan.
    let (_, tahun) = k.admin.get("/api/laporan/penjualan?dari=2025-01-01&sampai=2025-12-31").await;
    assert_eq!(tahun["kelompok"], "bulan");
    assert_eq!(tahun["tren"].as_array().unwrap().len(), 12);
    assert_eq!(tahun["tren"][9]["total"], 449_999);
}

#[sqlx::test]
async fn pembagian_diskon_sama_dengan_nilai_retur(pool: PgPool) {
    let k = siapkan(pool).await;
    let mut varian = vec![];
    for kode in ["X", "Y", "Z"] {
        varian.push(buat_produk(&k.admin, kode, 100_000, &[5]).await.1[0]);
    }
    k.admin.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(
        &k.admin,
        json!({ "items": varian.iter().map(|v| json!({ "id_varian": v, "jumlah": 1 })).collect::<Vec<_>>(), "diskon_transaksi": 10_000, "metode_bayar": "qris" }),
    )
    .await;
    undur_transaksi(&k, &struk, "2025-10-05 10:00+08").await;

    let (_, l) = k.admin.get("/api/laporan/penjualan?dari=2025-10-05&sampai=2025-10-05").await;
    let mut laporan: Vec<i64> = l["per_produk"].as_array().unwrap().iter().map(|p| p["penjualan"].as_i64().unwrap()).collect();
    laporan.sort();
    assert_eq!(laporan, vec![96_666, 96_667, 96_667]);

    let (_, faktur) = k.admin.get(&format!("/api/transaksi/faktur/{}", struk["no_faktur"].as_str().unwrap())).await;
    let mut dibayar: Vec<i64> = faktur["items"].as_array().unwrap().iter().map(|i| i["dibayar"].as_i64().unwrap()).collect();
    dibayar.sort();
    assert_eq!(laporan, dibayar, "rincian laporan harus sama dengan dasar nilai retur");
}

#[sqlx::test]
async fn kasir_hanya_melihat_penjualannya_sendiri(pool: PgPool) {
    let k = siapkan(pool).await;
    skenario(&k).await;
    let (_, v) = buat_produk(&k.admin, "K", 50_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!({ "items": [{ "id_varian": v[0], "jumlah": 1 }], "metode_bayar": "qris" })).await;
    undur_transaksi(&k, &struk, "2025-10-20 10:00+08").await;

    let (status, l) = k.kasir.get("/api/laporan/penjualan?dari=2025-10-01&sampai=2025-10-31").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(l["ringkasan"]["transaksi"], 1);
    assert_eq!(l["ringkasan"]["bersih"], 50_000);
    assert!(l["ringkasan"].get("hpp").is_none());
    assert!(l.get("per_kasir").is_none());
    assert!(l.get("lambat_terjual").is_none());

    // Admin melihat kedua penjual pada rincian per kasir.
    let (_, l) = k.admin.get("/api/laporan/penjualan?dari=2025-10-01&sampai=2025-10-31").await;
    assert_eq!(l["per_kasir"].as_array().unwrap().len(), 2);

    for uri in [
        "/api/laporan/laba-rugi?dari=2025-10-01&sampai=2025-10-31",
        "/api/laporan/penjualan.csv?dari=2025-10-01&sampai=2025-10-31",
        "/api/laporan/stok",
        "/api/biaya",
        "/api/ringkasan/dasbor",
    ] {
        assert_eq!(k.kasir.get(uri).await.0, StatusCode::FORBIDDEN, "{uri}");
    }
}

#[sqlx::test]
async fn validasi_rentang_laporan(pool: PgPool) {
    let k = siapkan(pool).await;
    assert_eq!(k.admin.get("/api/laporan/penjualan").await.0, StatusCode::BAD_REQUEST);
    assert_eq!(k.admin.get("/api/laporan/penjualan?dari=2025-10-02&sampai=2025-10-01").await.0, StatusCode::BAD_REQUEST);
    assert_eq!(k.admin.get("/api/laporan/penjualan?dari=2015-01-01&sampai=2025-01-01").await.0, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn ekspor_csv_berformat_excel_indonesia(pool: PgPool) {
    let k = siapkan(pool).await;
    skenario(&k).await;
    let (status, headers, isi) = k.admin.mentah("/api/laporan/penjualan.csv?dari=2025-10-01&sampai=2025-10-31").await;
    assert_eq!(status, StatusCode::OK);
    assert!(headers["content-type"].to_str().unwrap().starts_with("text/csv"));
    assert!(headers["content-disposition"].to_str().unwrap().contains("laporan-penjualan-2025-10-01_2025-10-31.csv"));
    assert_eq!(&isi[..3], "\u{feff}".as_bytes(), "BOM agar Excel membaca UTF-8");
    let teks = String::from_utf8(isi).unwrap();
    assert!(teks.contains("Penjualan bersih;449999\r\n"), "{teks}");
    assert!(teks.contains("Per produk;Unit;Penjualan"));

    let (_, _, isi) = k.admin.mentah("/api/laporan/laba-rugi.csv?dari=2025-10-01&sampai=2025-10-31").await;
    let teks = String::from_utf8(isi).unwrap();
    assert!(teks.contains("Laba bersih;-140001;0"), "{teks}");
    assert!(teks.contains("Margin laba kotor (%);46.67;"));

    let (status, _, isi) = k.admin.mentah("/api/laporan/stok.csv").await;
    assert_eq!(status, StatusCode::OK);
    assert!(String::from_utf8(isi).unwrap().starts_with("\u{feff}Kode;Produk;Kategori"));
}

#[sqlx::test]
async fn csv_menetralkan_rumus_lembar_kerja(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, kategori) = k.admin.post("/api/kategori", json!({ "nama_kategori": "=HYPERLINK(\"x\")" })).await;
    k.admin
        .post(
            "/api/produk",
            json!({ "kode": "RUMUS", "nama": "Kaos; biasa", "id_kategori": kategori["id_kategori"], "harga_beli": 1, "harga_jual": 2,
                    "varian": [{ "ukuran": "M", "warna": "Hitam", "stok_awal": 1 }] }),
        )
        .await;
    let (_, _, isi) = k.admin.mentah("/api/laporan/stok.csv").await;
    let teks = String::from_utf8(isi).unwrap();
    assert!(teks.contains("\"Kaos; biasa\""), "{teks}");
    assert!(teks.contains("\"'=HYPERLINK(\"\"x\"\")\""), "{teks}");
}

#[sqlx::test]
async fn biaya_operasional_divalidasi_dan_tercatat(pool: PgPool) {
    let k = siapkan(pool).await;
    let (status, _) = k.admin.post("/api/biaya", json!({ "tanggal": "2025-10-01", "kategori": "Hiburan", "nominal": 1 })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = k.admin.post("/api/biaya", json!({ "tanggal": "2025-10-01", "kategori": "Lain-lain", "nominal": 1 })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = k.admin.post("/api/biaya", json!({ "tanggal": "2025-10-01", "kategori": "Promosi", "nominal": 0 })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, biaya) = k
        .admin
        .post("/api/biaya", json!({ "tanggal": "2025-10-01", "kategori": "Lain-lain", "nominal": 25_000, "keterangan": "Servis AC" }))
        .await;
    let id = biaya["id_biaya"].as_i64().unwrap();
    let (status, ubah) = k
        .admin
        .put(&format!("/api/biaya/{id}"), json!({ "tanggal": "2025-10-02", "kategori": "Promosi", "nominal": 30_000 }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ubah["nominal"], 30_000);
    let (_, daftar) = k.admin.get("/api/biaya?dari=2025-10-01&sampai=2025-10-31").await;
    assert_eq!(daftar.as_array().unwrap().len(), 1);
    assert_eq!(k.admin.kirim(Method::DELETE, &format!("/api/biaya/{id}"), None).await.0, StatusCode::OK);

    let aksi: Vec<String> = sqlx::query_scalar("SELECT aksi FROM log_aktivitas WHERE objek = 'biaya' ORDER BY id_log")
        .fetch_all(&k.pool)
        .await
        .unwrap();
    assert_eq!(aksi, ["tambah", "ubah", "hapus"]);
}

#[sqlx::test]
async fn dasbor_menampilkan_ringkasan_hari_ini(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "D", 50_000, &[5, 1]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!({ "items": [{ "id_varian": v[0], "jumlah": 2 }], "metode_bayar": "qris" })).await;
    k.kasir
        .post(
            "/api/pengajuan",
            json!({ "id_transaksi": struk["id_transaksi"], "jenis": "retur", "alasan": "Cacat",
                    "items": [{ "id_detail": struk["items"][0]["id_detail"], "jumlah": 1 }] }),
        )
        .await;
    buat_kasir(&k.pool, "kasir2").await;
    let mut kasir2 = Klien::new(&k.app);
    kasir2.login("kasir2", SANDI).await;

    let (status, d) = k.admin.get("/api/ringkasan/dasbor").await;
    assert_eq!(status, StatusCode::OK, "{d}");
    assert_eq!(d["ringkasan_hari_ini"]["transaksi"], 1);
    assert_eq!(d["ringkasan_hari_ini"]["bersih"], 100_000);
    assert_eq!(d["ringkasan_kemarin"]["transaksi"], 0);
    assert_eq!(d["tren"].as_array().unwrap().len(), 14);
    assert_eq!(d["terlaris_7_hari"][0]["unit"], 2);
    assert_eq!(d["pengajuan_menunggu"], 1);
    assert_eq!(d["stok_menipis"], 1, "varian berstok 1 dengan minimum 1");
    assert_eq!(d["sesi_aktif"].as_array().unwrap().len(), 1);
}
