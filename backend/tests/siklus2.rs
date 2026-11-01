//! Pengujian integrasi siklus 2: diskon (KF-07), pengajuan pembatalan/retur/tukar
//! dengan persetujuan pemilik (KF-10, UJ-07), barang masuk (KF-11), stok opname
//! (KF-12, UJ-08), kartu stok (KF-13), dan peringatan stok (KF-14).

mod common;

use axum::http::{Method, StatusCode};
use common::{Klien, SANDI, buat_kasir, buat_produk, siapkan, stok};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn jual(kasir: &Klien, items: Value, metode: &str, bayar: Option<i64>) -> Value {
    let (status, struk) = kasir.post("/api/transaksi", json!({ "items": items, "metode_bayar": metode, "bayar": bayar })).await;
    assert_eq!(status, StatusCode::OK, "{struk}");
    struk
}

async fn rekap_aktif(kasir: &Klien) -> Value {
    kasir.get("/api/sesi/aktif").await.1
}

async fn kasir_lain(k: &common::Konteks, username: &str) -> Klien {
    buat_kasir(&k.pool, username).await;
    let mut klien = Klien::new(&k.app);
    assert_eq!(klien.login(username, SANDI).await, StatusCode::OK);
    klien
}

// ---------- KF-07 Diskon ----------

#[sqlx::test]
async fn uj05_diskon_transaksi_10_persen_tunai(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, a) = buat_produk(&k.admin, "A", 100_000, &[5]).await;
    let (_, b) = buat_produk(&k.admin, "B", 120_000, &[5]).await;
    let (_, c) = buat_produk(&k.admin, "C", 80_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;

    let (status, struk) = k
        .kasir
        .post(
            "/api/transaksi",
            json!({
                "items": [{ "id_varian": a[0], "jumlah": 1 }, { "id_varian": b[0], "jumlah": 1 }, { "id_varian": c[0], "jumlah": 1 }],
                "diskon_transaksi": 30_000,
                "metode_bayar": "tunai",
                "bayar": 300_000
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{struk}");
    assert_eq!(struk["subtotal"], 300_000);
    assert_eq!(struk["diskon"], 30_000);
    assert_eq!(struk["total"], 270_000);
    assert_eq!(struk["kembalian"], 30_000);
}

#[sqlx::test]
async fn diskon_item_tersimpan_per_baris(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 50_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 2, "diskon_item": 10_000 }]), "qris", None).await;
    assert_eq!(struk["items"][0]["diskon_item"], 10_000);
    assert_eq!(struk["items"][0]["subtotal"], 90_000);
    assert_eq!(struk["total"], 90_000);
}

#[sqlx::test]
async fn diskon_kasir_dibatasi_admin_tidak(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "JAKET", 100_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    k.admin.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let body = |diskon: i64| json!({ "items": [{ "id_varian": v[0], "jumlah": 1 }], "diskon_transaksi": diskon, "metode_bayar": "qris" });

    // Tepat 10% boleh, lebih sedikit saja ditolak.
    assert_eq!(k.kasir.post("/api/transaksi", body(10_000)).await.0, StatusCode::OK);
    let (status, galat) = k.kasir.post("/api/transaksi", body(10_001)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(galat["pesan"].as_str().unwrap().contains("batas 10%"));
    assert_eq!(k.admin.post("/api/transaksi", body(50_000)).await.0, StatusCode::OK);
}

#[sqlx::test]
async fn varian_ganda_dalam_satu_transaksi_ditolak(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let (status, _) = k
        .kasir
        .post(
            "/api/transaksi",
            json!({ "items": [{ "id_varian": v[0], "jumlah": 1 }, { "id_varian": v[0], "jumlah": 1 }], "metode_bayar": "qris" }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ---------- KF-10 Pengajuan ----------

#[sqlx::test]
async fn uj07_retur_menunggu_persetujuan_pemilik(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 100_000 })).await;
    let struk = jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 2 }]), "tunai", Some(130_000)).await;

    let (status, pengajuan) = k
        .kasir
        .post(
            "/api/pengajuan",
            json!({
                "id_transaksi": struk["id_transaksi"],
                "jenis": "retur",
                "alasan": "Ukuran kekecilan",
                "items": [{ "id_detail": struk["items"][0]["id_detail"], "jumlah": 1 }]
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{pengajuan}");
    assert_eq!(pengajuan["status"], "menunggu");
    assert_eq!(pengajuan["total_pengembalian"], 65_000);
    let id = pengajuan["id_retur"].as_i64().unwrap();

    // Tanpa otorisasi pemilik: stok dan kas belum berubah, kasir tidak dapat menyetujui sendiri.
    assert_eq!(stok(&k.pool, v[0]).await, 3);
    assert_eq!(rekap_aktif(&k.kasir).await["refund_tunai"], 0);
    assert_eq!(k.kasir.post(&format!("/api/pengajuan/{id}/setujui"), json!({})).await.0, StatusCode::FORBIDDEN);

    // Dengan otorisasi: stok bertambah dan mutasi retur tercatat.
    let (status, disetujui) = k.admin.post(&format!("/api/pengajuan/{id}/setujui"), json!({})).await;
    assert_eq!(status, StatusCode::OK, "{disetujui}");
    assert_eq!(disetujui["status"], "disetujui");
    assert_eq!(stok(&k.pool, v[0]).await, 4);
    let referensi: String = sqlx::query_scalar("SELECT referensi FROM mutasi_stok WHERE jenis = 'retur'")
        .fetch_one(&k.pool)
        .await
        .unwrap();
    assert_eq!(referensi, format!("RT-{id}"));

    let rekap = rekap_aktif(&k.kasir).await;
    assert_eq!(rekap["refund_tunai"], 65_000);
    assert_eq!(rekap["kas_seharusnya"], 100_000 + 130_000 - 65_000);

    // Pengajuan yang sudah diputuskan tidak dapat diputuskan lagi.
    assert_eq!(k.admin.post(&format!("/api/pengajuan/{id}/setujui"), json!({})).await.0, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn pengajuan_ditolak_atau_ditarik_tidak_mengubah_apa_pun(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 2 }]), "tunai", Some(130_000)).await;
    let ajukan = json!({
        "id_transaksi": struk["id_transaksi"],
        "jenis": "retur",
        "alasan": "Berubah pikiran",
        "items": [{ "id_detail": struk["items"][0]["id_detail"], "jumlah": 1 }]
    });

    let id = k.kasir.post("/api/pengajuan", ajukan.clone()).await.1["id_retur"].as_i64().unwrap();
    assert_eq!(k.admin.post(&format!("/api/pengajuan/{id}/tolak"), json!({ "alasan": " " })).await.0, StatusCode::BAD_REQUEST);
    let (_, ditolak) = k.admin.post(&format!("/api/pengajuan/{id}/tolak"), json!({ "alasan": "Barang sudah dipakai" })).await;
    assert_eq!(ditolak["status"], "ditolak");
    assert_eq!(ditolak["alasan_tolak"], "Barang sudah dipakai");

    let id = k.kasir.post("/api/pengajuan", ajukan).await.1["id_retur"].as_i64().unwrap();
    let (_, ditarik) = k.kasir.post(&format!("/api/pengajuan/{id}/tarik"), json!({})).await;
    assert_eq!(ditarik["status"], "dibatalkan");

    assert_eq!(stok(&k.pool, v[0]).await, 3);
    assert_eq!(rekap_aktif(&k.kasir).await["refund_tunai"], 0);
}

#[sqlx::test]
async fn pembatalan_di_sesi_yang_sama_mengembalikan_stok_tanpa_hitung_ganda(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5, 5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 100_000 })).await;
    let struk = jual(
        &k.kasir,
        json!([{ "id_varian": v[0], "jumlah": 2 }, { "id_varian": v[1], "jumlah": 1 }]),
        "tunai",
        Some(195_000),
    )
    .await;

    let (_, p) = k
        .kasir
        .post("/api/pengajuan", json!({ "id_transaksi": struk["id_transaksi"], "jenis": "batal", "alasan": "Salah input barang" }))
        .await;
    assert_eq!(p["total_pengembalian"], 195_000);
    let id = p["id_retur"].as_i64().unwrap();
    k.admin.post(&format!("/api/pengajuan/{id}/setujui"), json!({})).await;

    assert_eq!(stok(&k.pool, v[0]).await, 5);
    assert_eq!(stok(&k.pool, v[1]).await, 5);
    let (_, ulang) = k.kasir.get(&format!("/api/transaksi/{}/struk", struk["id_transaksi"])).await;
    assert_eq!(ulang["status"], "batal");
    let rekap = rekap_aktif(&k.kasir).await;
    assert_eq!(rekap["penjualan_tunai"], 0);
    assert_eq!(rekap["refund_tunai"], 0);
    assert_eq!(rekap["kas_seharusnya"], 100_000);
}

#[sqlx::test]
async fn pembatalan_transaksi_sesi_lain_ditolak(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    let (_, sesi) = k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 1 }]), "qris", None).await;
    let id_sesi = sesi["sesi"]["id_sesi"].as_i64().unwrap();
    k.kasir.post(&format!("/api/sesi/{id_sesi}/tutup"), json!({ "kas_fisik": 0 })).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;

    let (status, galat) = k
        .kasir
        .post("/api/pengajuan", json!({ "id_transaksi": struk["id_transaksi"], "jenis": "batal", "alasan": "Salah" }))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(galat["pesan"].as_str().unwrap().contains("retur"));
}

#[sqlx::test]
async fn tukar_ukuran_hanya_produk_yang_sama(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, kaos) = buat_produk(&k.admin, "KAOS", 65_000, &[5, 5]).await;
    let (_, lain) = buat_produk(&k.admin, "TOPI", 65_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!([{ "id_varian": kaos[0], "jumlah": 1 }]), "qris", None).await;
    let id_detail = struk["items"][0]["id_detail"].clone();
    let tukar = |pengganti: i64| {
        json!({
            "id_transaksi": struk["id_transaksi"],
            "jenis": "tukar",
            "alasan": "Kekecilan",
            "items": [{ "id_detail": id_detail, "jumlah": 1, "id_varian_pengganti": pengganti }]
        })
    };

    assert_eq!(k.kasir.post("/api/pengajuan", tukar(lain[0])).await.0, StatusCode::BAD_REQUEST);
    let (status, p) = k.kasir.post("/api/pengajuan", tukar(kaos[1])).await;
    assert_eq!(status, StatusCode::OK, "{p}");
    assert_eq!(p["total_pengembalian"], 0);
    k.admin.post(&format!("/api/pengajuan/{}/setujui", p["id_retur"]), json!({})).await;

    assert_eq!(stok(&k.pool, kaos[0]).await, 5);
    assert_eq!(stok(&k.pool, kaos[1]).await, 4);
}

#[sqlx::test]
async fn retur_melebihi_sisa_ditolak(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 2 }]), "qris", None).await;
    let retur = |jumlah: i32| {
        json!({
            "id_transaksi": struk["id_transaksi"],
            "jenis": "retur",
            "alasan": "Cacat",
            "items": [{ "id_detail": struk["items"][0]["id_detail"], "jumlah": jumlah }]
        })
    };
    assert_eq!(k.kasir.post("/api/pengajuan", retur(3)).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(k.kasir.post("/api/pengajuan", retur(2)).await.0, StatusCode::OK);
    // Pengajuan yang masih menunggu ikut mengurangi sisa.
    assert_eq!(k.kasir.post("/api/pengajuan", retur(1)).await.0, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn retur_dengan_diskon_transaksi_dibagi_proporsional(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 100_000, &[5]).await;
    k.admin.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    // Admin bebas batas diskon: 3 unit 300.000 dengan diskon 10.000, dibayar 290.000.
    let struk = jual(&k.admin, json!([{ "id_varian": v[0], "jumlah": 3 }]), "qris", None).await;
    assert_eq!(struk["total"], 300_000);
    let (_, struk) = k
        .admin
        .post(
            "/api/transaksi",
            json!({ "items": [{ "id_varian": v[0], "jumlah": 2 }], "diskon_transaksi": 10_001, "metode_bayar": "qris" }),
        )
        .await;
    let mut total = 0;
    for _ in 0..2 {
        let (_, p) = k
            .admin
            .post(
                "/api/pengajuan",
                json!({
                    "id_transaksi": struk["id_transaksi"],
                    "jenis": "retur",
                    "alasan": "Cacat",
                    "items": [{ "id_detail": struk["items"][0]["id_detail"], "jumlah": 1 }]
                }),
            )
            .await;
        // Admin mengajukan sendiri: langsung disetujui.
        assert_eq!(p["status"], "disetujui");
        total += p["total_pengembalian"].as_i64().unwrap();
    }
    assert_eq!(total, 200_000 - 10_001);
}

#[sqlx::test]
async fn tutup_sesi_terhalang_pengajuan_menunggu(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    let (_, sesi) = k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let id_sesi = sesi["sesi"]["id_sesi"].as_i64().unwrap();
    let struk = jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 1 }]), "qris", None).await;
    let (_, p) = k
        .kasir
        .post(
            "/api/pengajuan",
            json!({
                "id_transaksi": struk["id_transaksi"],
                "jenis": "retur",
                "alasan": "Cacat",
                "items": [{ "id_detail": struk["items"][0]["id_detail"], "jumlah": 1 }]
            }),
        )
        .await;

    let url = format!("/api/sesi/{id_sesi}/tutup");
    let (status, galat) = k.kasir.post(&url, json!({ "kas_fisik": 0 })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(galat["pesan"].as_str().unwrap().contains("menunggu persetujuan"));

    k.kasir.post(&format!("/api/pengajuan/{}/tarik", p["id_retur"]), json!({})).await;
    assert_eq!(k.kasir.post(&url, json!({ "kas_fisik": 0 })).await.0, StatusCode::OK);
}

#[sqlx::test]
async fn kasir_lain_dapat_mencari_faktur_tepat_untuk_retur(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5, 5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let struk = jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 1 }]), "qris", None).await;

    let kasir2 = kasir_lain(&k, "kasir2").await;
    kasir2.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let no = struk["no_faktur"].as_str().unwrap();
    let (status, faktur) = kasir2.get(&format!("/api/transaksi/faktur/{}", no.to_lowercase())).await;
    assert_eq!(status, StatusCode::OK, "{faktur}");
    assert_eq!(faktur["items"][0]["sisa"], 1);
    assert_eq!(faktur["bisa_dibatalkan"], false);
    assert_eq!(faktur["items"][0]["pengganti"].as_array().unwrap().len(), 1);
    // Riwayat dan struk kasir lain tetap tertutup.
    assert_eq!(kasir2.get(&format!("/api/transaksi/{}/struk", struk["id_transaksi"])).await.0, StatusCode::FORBIDDEN);

    let (status, _) = kasir2
        .post(
            "/api/pengajuan",
            json!({
                "id_transaksi": faktur["id_transaksi"],
                "jenis": "retur",
                "alasan": "Cacat jahitan",
                "items": [{ "id_detail": faktur["items"][0]["id_detail"], "jumlah": 1 }]
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(k.kasir.get("/api/transaksi/faktur/ZO-TIDAK-ADA").await.0, StatusCode::NOT_FOUND);
}

// ---------- KF-11 Barang masuk ----------

#[sqlx::test]
async fn barang_masuk_menambah_stok_dan_memperbarui_harga_beli(pool: PgPool) {
    let k = siapkan(pool).await;
    let (id_produk, v) = buat_produk(&k.admin, "KAOS", 65_000, &[2]).await;
    let (_, pemasok) = k.admin.post("/api/pemasok", json!({ "nama": "CV Konveksi Bali" })).await;

    let (status, hasil) = k
        .admin
        .post(
            "/api/barang-masuk",
            json!({ "id_pemasok": pemasok["id_pemasok"], "no_referensi": "NOTA-12", "items": [{ "id_varian": v[0], "jumlah": 10, "harga_beli": 40_000 }] }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{hasil}");
    assert_eq!(hasil["barang_masuk"]["total_nilai"], 400_000);
    assert_eq!(hasil["peringatan"].as_array().unwrap().len(), 0);
    assert_eq!(stok(&k.pool, v[0]).await, 12);
    let (_, produk) = k.admin.get(&format!("/api/produk/{id_produk}")).await;
    assert_eq!(produk["harga_beli"], 40_000);

    let (_, log) = k.admin.get("/api/log-aktivitas?aksi=ubah").await;
    let entri = &log["data"][0];
    assert_eq!(entri["nilai_lama"]["harga_beli"], 32_500);
    assert_eq!(entri["nilai_baru"]["harga_beli"], 40_000);

    // Harga beli di atas harga jual tetap dicatat, disertai peringatan.
    let (_, hasil) = k
        .admin
        .post("/api/barang-masuk", json!({ "items": [{ "id_varian": v[0], "jumlah": 1, "harga_beli": 70_000 }] }))
        .await;
    assert_eq!(hasil["peringatan"].as_array().unwrap().len(), 1);
}

// ---------- KF-12 Stok opname, KF-13 Kartu stok ----------

#[sqlx::test]
async fn uj08_stok_opname_wajib_alasan_dan_menghitung_akurasi(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5, 8]).await;
    let items = json!([{ "id_varian": v[0], "stok_fisik": 5 }, { "id_varian": v[1], "stok_fisik": 7 }]);

    let (status, _) = k.admin.post("/api/stok/opname", json!({ "alasan": "", "items": items })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(stok(&k.pool, v[1]).await, 8);

    let (status, hasil) = k.admin.post("/api/stok/opname", json!({ "alasan": "Opname bulanan, 1 hilang", "items": items })).await;
    assert_eq!(status, StatusCode::OK, "{hasil}");
    assert_eq!(hasil["dihitung"], 2);
    assert_eq!(hasil["sesuai"], 1);
    assert_eq!(hasil["akurasi_persen"], 50.0);
    assert_eq!(stok(&k.pool, v[1]).await, 7);
    assert!(hasil["referensi"].as_str().unwrap().starts_with("OPN-"));
}

#[sqlx::test]
async fn kartu_stok_konsisten_dengan_stok_akhir(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    jual(&k.kasir, json!([{ "id_varian": v[0], "jumlah": 2 }]), "qris", None).await;
    k.admin.post("/api/barang-masuk", json!({ "items": [{ "id_varian": v[0], "jumlah": 4, "harga_beli": 30_000 }] })).await;
    k.admin.post("/api/stok/opname", json!({ "alasan": "Cek", "items": [{ "id_varian": v[0], "stok_fisik": 6 }] })).await;

    let (status, kartu) = k.admin.get(&format!("/api/varian/{}/kartu-stok", v[0])).await;
    assert_eq!(status, StatusCode::OK);
    let mutasi = kartu["mutasi"].as_array().unwrap();
    let jenis: Vec<&str> = mutasi.iter().map(|m| m["jenis"].as_str().unwrap()).collect();
    assert_eq!(jenis, ["penyesuaian", "keluar", "masuk", "penyesuaian"]);
    let jumlah: i64 = mutasi.iter().map(|m| m["jumlah"].as_i64().unwrap()).sum();
    assert_eq!(kartu["saldo_awal"].as_i64().unwrap() + jumlah, kartu["stok"].as_i64().unwrap());
    assert_eq!(kartu["stok"], 6);
}

// ---------- KF-14 Peringatan stok ----------

#[sqlx::test]
async fn daftar_lambat_terjual_dan_stok_menipis(pool: PgPool) {
    let k = siapkan(pool).await;
    let (lama, v_lama) = buat_produk(&k.admin, "LAMA", 65_000, &[5]).await;
    let (laku, v_laku) = buat_produk(&k.admin, "LAKU", 65_000, &[5]).await;
    let (_, v_baru) = buat_produk(&k.admin, "BARU", 65_000, &[1]).await;
    sqlx::query("UPDATE produk SET dibuat_pada = now() - interval '60 days' WHERE id_produk = ANY($1)")
        .bind(vec![lama, laku])
        .execute(&k.pool)
        .await
        .unwrap();
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    jual(&k.kasir, json!([{ "id_varian": v_laku[0], "jumlah": 1 }]), "qris", None).await;

    let (status, p) = k.admin.get("/api/stok/peringatan").await;
    assert_eq!(status, StatusCode::OK);
    let lambat: Vec<i64> = p["lambat_terjual"].as_array().unwrap().iter().map(|v| v["id_varian"].as_i64().unwrap()).collect();
    assert_eq!(lambat, vec![v_lama[0]]);
    // stok_minimum bawaan dari buat_produk = 1, sehingga varian baru berstok 1 menipis.
    let menipis: Vec<i64> = p["menipis"].as_array().unwrap().iter().map(|v| v["id_varian"].as_i64().unwrap()).collect();
    assert_eq!(menipis, vec![v_baru[0]]);

    let (_, lencana) = k.admin.kirim_dengan(Method::GET, "/api/ringkasan/lencana", None, true).await;
    assert_eq!(lencana["stok_menipis"], 1);
    assert_eq!(lencana["pengajuan_menunggu"], 0);
}

// ---------- Pengaturan, polling, dan hak akses ----------

#[sqlx::test]
async fn pengaturan_batas_diskon_berlaku(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, mut atur) = k.kasir.get("/api/pengaturan").await;
    assert_eq!(atur["batas_diskon_kasir_persen"], 10);
    atur["batas_diskon_kasir_persen"] = json!(20);
    assert_eq!(k.kasir.put("/api/pengaturan", atur.clone()).await.0, StatusCode::FORBIDDEN);
    assert_eq!(k.admin.put("/api/pengaturan", atur.clone()).await.0, StatusCode::OK);
    atur["batas_diskon_kasir_persen"] = json!(150);
    assert_eq!(k.admin.put("/api/pengaturan", atur).await.0, StatusCode::BAD_REQUEST);

    let (_, v) = buat_produk(&k.admin, "KAOS", 100_000, &[5]).await;
    k.kasir.post("/api/sesi/buka", json!({ "modal_awal": 0 })).await;
    let (status, _) = k
        .kasir
        .post("/api/transaksi", json!({ "items": [{ "id_varian": v[0], "jumlah": 1 }], "diskon_transaksi": 20_000, "metode_bayar": "qris" }))
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn permintaan_latar_tidak_memperpanjang_sesi_login(pool: PgPool) {
    let k = siapkan(pool).await;
    sqlx::query("UPDATE sesi_login SET terakhir_aktif = now() - interval '20 minutes'")
        .execute(&k.pool)
        .await
        .unwrap();
    assert_eq!(k.kasir.kirim_dengan(Method::GET, "/api/auth/me", None, true).await.0, StatusCode::OK);
    let tidak_aktif: bool = sqlx::query_scalar(
        "SELECT bool_and(terakhir_aktif < now() - interval '19 minutes') FROM sesi_login",
    )
    .fetch_one(&k.pool)
    .await
    .unwrap();
    assert!(tidak_aktif, "polling tidak boleh memperbarui terakhir_aktif");

    assert_eq!(k.kasir.get("/api/auth/me").await.0, StatusCode::OK);
    let segar: i64 = sqlx::query_scalar("SELECT count(*) FROM sesi_login WHERE terakhir_aktif > now() - interval '1 minute'")
        .fetch_one(&k.pool)
        .await
        .unwrap();
    assert_eq!(segar, 1);
}

#[sqlx::test]
async fn kasir_tidak_dapat_membuka_fitur_stok_dan_persetujuan(pool: PgPool) {
    let k = siapkan(pool).await;
    let (_, v) = buat_produk(&k.admin, "KAOS", 65_000, &[5]).await;
    let terlarang = [
        (Method::POST, "/api/pengajuan/1/setujui".to_string(), Some(json!({}))),
        (Method::POST, "/api/pengajuan/1/tolak".to_string(), Some(json!({ "alasan": "x" }))),
        (Method::POST, "/api/barang-masuk".to_string(), Some(json!({ "items": [{ "id_varian": v[0], "jumlah": 1, "harga_beli": 1 }] }))),
        (Method::POST, "/api/stok/opname".to_string(), Some(json!({ "alasan": "x", "items": [{ "id_varian": v[0], "stok_fisik": 1 }] }))),
        (Method::GET, "/api/stok/peringatan".to_string(), None),
        (Method::GET, format!("/api/varian/{}/kartu-stok", v[0]), None),
        (Method::GET, "/api/pemasok".to_string(), None),
    ];
    for (method, uri, body) in terlarang {
        assert_eq!(k.kasir.kirim(method, &uri, body).await.0, StatusCode::FORBIDDEN, "{uri}");
    }
}
