//! Utilitas pengujian integrasi: klien HTTP tiruan yang menyimpan cookie sesi.

#![allow(dead_code)]

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use zero_outlet_pos::{AppState, Config, app, services::{pengguna::pastikan_admin, sandi}};

pub const SANDI: &str = "rahasia123";

#[derive(Clone)]
pub struct Klien {
    app: Router,
    cookie: Option<String>,
}

impl Klien {
    pub fn new(app: &Router) -> Self {
        Self { app: app.clone(), cookie: None }
    }

    pub async fn kirim(&self, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        self.kirim_dengan(method, uri, body, false).await
    }

    /// Seperti `kirim`; dengan `latar = true` meniru polling antarmuka (header X-Latar).
    pub async fn kirim_dengan(&self, method: Method, uri: &str, body: Option<Value>, latar: bool) -> (StatusCode, Value) {
        let mut req = Request::builder().method(method).uri(uri);
        if latar {
            req = req.header("x-latar", "1");
        }
        if let Some(c) = &self.cookie {
            req = req.header(header::COOKIE, c);
        }
        let req = match body {
            Some(b) => req.header(header::CONTENT_TYPE, "application/json").body(Body::from(b.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let json = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
        (status, json)
    }

    /// GET tanpa mengurai JSON, untuk unduhan berkas (CSV).
    pub async fn mentah(&self, uri: &str) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
        let mut req = Request::builder().method(Method::GET).uri(uri);
        if let Some(c) = &self.cookie {
            req = req.header(header::COOKIE, c);
        }
        let res = self.app.clone().oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let bytes = res.into_body().collect().await.unwrap().to_bytes().to_vec();
        (status, headers, bytes)
    }

    pub async fn get(&self, uri: &str) -> (StatusCode, Value) {
        self.kirim(Method::GET, uri, None).await
    }

    pub async fn post(&self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.kirim(Method::POST, uri, Some(body)).await
    }

    pub async fn put(&self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.kirim(Method::PUT, uri, Some(body)).await
    }

    pub async fn login(&mut self, username: &str, sandi: &str) -> StatusCode {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "username": username, "kata_sandi": sandi }).to_string()))
            .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        if let Some(set_cookie) = res.headers().get(header::SET_COOKIE) {
            let pasangan = set_cookie.to_str().unwrap().split(';').next().unwrap().to_string();
            self.cookie = Some(pasangan);
        }
        res.status()
    }
}

pub struct Konteks {
    pub pool: PgPool,
    pub app: Router,
    pub admin: Klien,
    pub kasir: Klien,
}

pub async fn buat_kasir(pool: &PgPool, username: &str) {
    let hash = sandi::hash(SANDI.into()).await.unwrap();
    sqlx::query("INSERT INTO pengguna (nama, username, kata_sandi, peran) VALUES ($1, $1, $2, 'kasir')")
        .bind(username)
        .bind(hash)
        .execute(pool)
        .await
        .unwrap();
}

/// Menyiapkan aplikasi dengan satu admin dan satu kasir yang sudah login.
pub async fn siapkan(pool: PgPool) -> Konteks {
    pastikan_admin(&pool, "Pemilik", "admin", SANDI).await.unwrap();
    buat_kasir(&pool, "kasir1").await;

    let app = app(AppState::new(pool.clone(), Config::default()));
    let mut admin = Klien::new(&app);
    assert_eq!(admin.login("admin", SANDI).await, StatusCode::OK);
    let mut kasir = Klien::new(&app);
    assert_eq!(kasir.login("kasir1", SANDI).await, StatusCode::OK);
    Konteks { pool, app, admin, kasir }
}

/// Membuat produk lewat API admin dan mengembalikan id varian sesuai urutan.
pub async fn buat_produk(admin: &Klien, kode: &str, harga_jual: i64, stok: &[i32]) -> (i64, Vec<i64>) {
    let (_, kategori) = admin.post("/api/kategori", json!({ "nama_kategori": format!("Kategori {kode}") })).await;
    let varian: Vec<Value> = stok
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let ukuran = ["S", "M", "L", "XL"][i];
            json!({ "ukuran": ukuran, "warna": "Hitam", "stok_awal": s, "stok_minimum": 1 })
        })
        .collect();
    let (status, produk) = admin
        .post(
            "/api/produk",
            json!({
                "kode": kode,
                "nama": format!("Produk {kode}"),
                "id_kategori": kategori["id_kategori"],
                "harga_beli": harga_jual / 2,
                "harga_jual": harga_jual,
                "varian": varian,
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{produk}");
    let ids = produk["varian"].as_array().unwrap().iter().map(|v| v["id_varian"].as_i64().unwrap()).collect();
    (produk["id_produk"].as_i64().unwrap(), ids)
}

pub async fn stok(pool: &PgPool, id_varian: i64) -> i32 {
    sqlx::query_scalar("SELECT stok FROM varian WHERE id_varian = $1").bind(id_varian).fetch_one(pool).await.unwrap()
}
