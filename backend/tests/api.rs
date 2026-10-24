//! Pengujian integrasi endpoint API berdasarkan skenario uji Tabel 25.

mod common;

use axum::http::StatusCode;
use common::{Klien, SANDI, buat_kasir, buat_produk, siapkan, stok};
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn uj01_login_dengan_kata_sandi_salah_ditolak(pool: PgPool) {
    let k = siapkan(pool).await;
    let mut tamu = Klien::new(&k.app);
    assert_eq!(tamu.login("kasir1", "salah-sandi").await, StatusCode::UNAUTHORIZED);
    assert_eq!(tamu.get("/api/auth/me").await.0, StatusCode::UNAUTHORIZED);

    let gagal: i64 = sqlx::query_scalar("SELECT count(*) FROM log_aktivitas WHERE aksi = 'login_gagal'")
        .fetch_one(&k.pool)
        .await
        .unwrap();
    assert_eq!(gagal, 1);
}

#[sqlx::test]
async fn uj02_kasir_tidak_dapat_membuka_fitur_admin(pool: PgPool) {
    let k = siapkan(pool).await;
    assert_eq!(k.kasir.get("/api/log-aktivitas").await.0, StatusCode::FORBIDDEN);
    assert_eq!(k.kasir.get("/api/pengguna").await.0, StatusCode::FORBIDDEN);
    assert_eq!(k.kasir.post("/api/kategori", json!({ "nama_kategori": "X" })).await.0, StatusCode::FORBIDDEN);
    assert_eq!(k.admin.get("/api/log-aktivitas").await.0, StatusCode::OK);
}

#[sqlx::test]
async fn tanpa_login_ditolak(pool: PgPool) {
    let k = siapkan(pool).await;
    assert_eq!(Klien::new(&k.app).get("/api/produk").await.0, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn sesi_login_berakhir_setelah_tidak_aktif(pool: PgPool) {
    let k = siapkan(pool).await;
    sqlx::query("UPDATE sesi_login SET terakhir_aktif = now() - interval '31 minutes'")
        .execute(&k.pool)
        .await
        .unwrap();
    assert_eq!(k.kasir.get("/api/auth/me").await.0, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn kasir_tidak_melihat_harga_beli(pool: PgPool) {
    let k = siapkan(pool).await;
    let (id_produk, _) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    let (_, p) = k.kasir.get(&format!("/api/produk/{id_produk}")).await;
    assert!(p.get("harga_beli").is_none());
    let (_, p) = k.admin.get(&format!("/api/produk/{id_produk}")).await;
    assert_eq!(p["harga_beli"], 32_500);
}

#[sqlx::test]
async fn uj03_transaksi_tanpa_sesi_aktif_ditolak(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, varian) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    let (status, body) = k
        .kasir
        .post("/api/transaksi", json!({ "items": [{ "id_varian": varian[0], "jumlah": 1 }], "metode_bayar": "qris" }))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["pesan"].as_str().unwrap().contains("Buka sesi"));
}

#[sqlx::test]
async fn transaksi_tunai_menghitung_kembalian_dan_mengurangi_stok(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, varian) = buat_produk(&k.admin, "KAOS", 65_000, &[5, 4]).await;
    assert_eq!(k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 200_000 })).await.0, StatusCode::OK);

    let (status, struk) = k
        .kasir
        .post(
            "/api/transaksi",
            json!({
                "items": [{ "id_varian": varian[0], "jumlah": 2 }, { "id_varian": varian[1], "jumlah": 1 }],
                "metode_bayar": "tunai",
                "bayar": 200_000
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{struk}");
    assert_eq!(struk["total"], 195_000);
    assert_eq!(struk["kembalian"], 5_000);
    assert_eq!(struk["items"].as_array().unwrap().len(), 2);
    assert!(struk["no_faktur"].as_str().unwrap().starts_with("ZO-"));

    assert_eq!(stok(&k.pool, varian[0]).await, 3);
    assert_eq!(stok(&k.pool, varian[1]).await, 3);
    let keluar: i64 = sqlx::query_scalar("SELECT count(*) FROM mutasi_stok WHERE jenis = 'keluar'")
        .fetch_one(&k.pool)
        .await
        .unwrap();
    assert_eq!(keluar, 2);
}

#[sqlx::test]
async fn uj04_stok_tidak_cukup_membatalkan_seluruh_transaksi(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, varian) = buat_produk(&k.admin, "KAOS", 65_000, &[5, 2]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;

    let (status, body) = k
        .kasir
        .post(
            "/api/transaksi",
            json!({
                "items": [{ "id_varian": varian[0], "jumlah": 1 }, { "id_varian": varian[1], "jumlah": 3 }],
                "metode_bayar": "transfer"
            }),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["pesan"].as_str().unwrap().contains("Stok tidak cukup"));

    // Varian pertama sempat dikurangi di dalam transaksi, tetapi harus kembali utuh.
    assert_eq!(stok(&k.pool, varian[0]).await, 5);
    assert_eq!(stok(&k.pool, varian[1]).await, 2);
    let jumlah: i64 = sqlx::query_scalar("SELECT count(*) FROM transaksi").fetch_one(&k.pool).await.unwrap();
    assert_eq!(jumlah, 0);
    let keluar: i64 = sqlx::query_scalar("SELECT count(*) FROM mutasi_stok WHERE jenis = 'keluar'")
        .fetch_one(&k.pool)
        .await
        .unwrap();
    assert_eq!(keluar, 0);
}

#[sqlx::test]
async fn dua_kasir_menjual_unit_terakhir_bersamaan(pool: PgPool) {
    let k = siapkan(pool).await;
    buat_kasir(&k.pool, "kasir2").await;
    let mut kasir2 = Klien::new(&k.app);
    kasir2.login("kasir2", SANDI).await;
    let (_, varian) = buat_produk(&k.admin, "JAKET", 250_000, &[1]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    kasir2.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;

    let body = json!({ "items": [{ "id_varian": varian[0], "jumlah": 1 }], "metode_bayar": "qris" });
    let (a, b) = tokio::join!(k.kasir.post("/api/transaksi", body.clone()), kasir2.post("/api/transaksi", body));
    let mut status = [a.0, b.0];
    status.sort();
    assert_eq!(status, [StatusCode::OK, StatusCode::CONFLICT]);
    assert_eq!(stok(&k.pool, varian[0]).await, 0);
}

#[sqlx::test]
async fn uj09_selisih_kas_saat_tutup_sesi_wajib_beralasan(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, varian) = buat_produk(&k.admin, "KAOS", 65_000, &[10]).await;
    let (_, sesi) = k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 200_000 })).await;
    let id_sesi = sesi["sesi"]["id_sesi"].as_i64().unwrap();

    k.kasir
        .post("/api/transaksi", json!({ "items": [{ "id_varian": varian[0], "jumlah": 2 }], "metode_bayar": "tunai", "bayar": 150_000 }))
        .await;
    k.kasir
        .post("/api/transaksi", json!({ "items": [{ "id_varian": varian[0], "jumlah": 1 }], "metode_bayar": "qris" }))
        .await;

    let (_, rekap) = k.kasir.get(&format!("/api/sesi/{id_sesi}/rekap")).await;
    assert_eq!(rekap["penjualan_tunai"], 130_000);
    assert_eq!(rekap["penjualan_qris"], 65_000);
    assert_eq!(rekap["kas_seharusnya"], 330_000);

    let url = format!("/api/sesi/{id_sesi}/tutup");
    let (status, body) = k.kasir.post(&url, json!({ "kas_fisik": 320_000 })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["pesan"].as_str().unwrap().contains("-10000"));

    let (status, body) = k.kasir.post(&url, json!({ "kas_fisik": 320_000, "alasan_selisih": "Salah memberi kembalian" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["sesi"]["selisih"], -10_000);

    // Sesi sudah ditutup: transaksi baru ditolak.
    let (status, _) = k
        .kasir
        .post("/api/transaksi", json!({ "items": [{ "id_varian": varian[0], "jumlah": 1 }], "metode_bayar": "qris" }))
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn hanya_satu_sesi_aktif_per_kasir(pool: PgPool) {
    let k = siapkan(pool).await;
    assert_eq!(k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await.0, StatusCode::OK);
    assert_eq!(k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await.0, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn uj10_perubahan_harga_jual_tercatat_di_log(pool: PgPool) {
    let k = siapkan(pool).await;
    let (id_produk, _) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    let (_, mut produk) = k.admin.get(&format!("/api/produk/{id_produk}")).await;
    produk["harga_jual"] = json!(70_000);
    let (status, _) = k.admin.put(&format!("/api/produk/{id_produk}"), produk).await;
    assert_eq!(status, StatusCode::OK);

    let (_, log) = k.admin.get("/api/log-aktivitas?aksi=ubah").await;
    let entri = &log["data"][0];
    assert_eq!(entri["objek"], "produk");
    assert_eq!(entri["nama_pengguna"], "Pemilik");
    assert_eq!(entri["nilai_lama"]["harga_jual"], 65_000);
    assert_eq!(entri["nilai_baru"]["harga_jual"], 70_000);
}

#[sqlx::test]
async fn uj06_cetak_ulang_struk_ditandai_dan_dibatasi_pemiliknya(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, varian) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let (_, asli) = k
        .kasir
        .post("/api/transaksi", json!({ "items": [{ "id_varian": varian[0], "jumlah": 1 }], "metode_bayar": "qris" }))
        .await;
    let id = asli["id_transaksi"].as_i64().unwrap();

    let (status, ulang) = k.kasir.get(&format!("/api/transaksi/{id}/struk?cetak_ulang=true")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ulang["cetak_ulang"], true);
    assert_eq!(ulang["jumlah_cetak"], 2);
    assert_eq!(ulang["items"], asli["items"]);
    assert_eq!(ulang["total"], asli["total"]);

    buat_kasir(&k.pool, "kasir2").await;
    let mut kasir2 = Klien::new(&k.app);
    kasir2.login("kasir2", SANDI).await;
    assert_eq!(kasir2.get(&format!("/api/transaksi/{id}/struk")).await.0, StatusCode::FORBIDDEN);
    let (_, riwayat) = kasir2.get("/api/transaksi").await;
    assert_eq!(riwayat.as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn produk_bertransaksi_hanya_dinonaktifkan(pool: PgPool) {
    let k = siapkan(pool).await;
    let (id_kosong, _) = buat_produk(&k.admin, "BARU", 50_000, &[0]).await;
    let (_, hasil) = k.admin.kirim(axum::http::Method::DELETE, &format!("/api/produk/{id_kosong}"), None).await;
    assert_eq!(hasil["dihapus"], true);

    let (id_berstok, _) = buat_produk(&k.admin, "LAMA", 50_000, &[3]).await;
    let (_, hasil) = k.admin.kirim(axum::http::Method::DELETE, &format!("/api/produk/{id_berstok}"), None).await;
    assert_eq!(hasil["dihapus"], false);
    let (_, p) = k.admin.get(&format!("/api/produk/{id_berstok}")).await;
    assert_eq!(p["aktif"], false);
}

#[sqlx::test]
async fn pencarian_produk_berdasarkan_kata_dan_sku(pool: PgPool) {
    let k = siapkan(pool).await;
    buat_produk(&k.admin, "KMJ", 150_000, &[3, 3]).await;
    buat_produk(&k.admin, "KAOS", 65_000, &[3]).await;

    let (_, hasil) = k.kasir.get("/api/produk?cari=kmj%20hitam").await;
    assert_eq!(hasil.as_array().unwrap().len(), 1);
    let (status, hasil) = k.kasir.get("/api/varian/sku/kmj-m-hitam").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(hasil["varian"]["ukuran"], "M");
    assert_eq!(k.kasir.get("/api/varian/sku/tidak-ada").await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn log_aktivitas_tidak_dapat_diubah_atau_dihapus(pool: PgPool) {
    let k = siapkan(pool).await;
    assert!(sqlx::query("UPDATE log_aktivitas SET aksi = 'x'").execute(&k.pool).await.is_err());
    assert!(sqlx::query("DELETE FROM log_aktivitas").execute(&k.pool).await.is_err());
}
