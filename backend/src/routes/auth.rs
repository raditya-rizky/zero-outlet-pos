//! Login dan logout (KF-01).

use axum::{Json, Router, extract::State, routing::{get, post}};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde_json::json;

use crate::{
    AppError, AppResult, AppState,
    middleware::auth::{NAMA_COOKIE, PenggunaLogin, buat_token, hash_token},
    models::pengguna::Login,
    services::{log::{self, Catatan}, sandi},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(saya))
}

async fn login(State(state): State<AppState>, jar: CookieJar, Json(req): Json<Login>) -> AppResult<(CookieJar, Json<PenggunaLogin>)> {
    let username = req.username.trim().to_lowercase();
    let akun = sqlx::query!(
        "SELECT id_pengguna, nama, username, peran, kata_sandi, aktif FROM pengguna WHERE username = $1",
        username
    )
    .fetch_optional(&state.pool)
    .await?;

    let sandi_benar = match &akun {
        Some(a) => sandi::cocok(req.kata_sandi, a.kata_sandi.clone()).await,
        None => false,
    };

    let akun = match akun {
        Some(a) if sandi_benar && a.aktif => a,
        lain => {
            let alasan = match &lain {
                Some(a) if sandi_benar && !a.aktif => "akun nonaktif",
                Some(_) => "kata sandi salah",
                None => "username tidak dikenal",
            };
            log::catat(
                &state.pool,
                Catatan {
                    id_pengguna: lain.as_ref().map(|a| a.id_pengguna),
                    aksi: "login_gagal",
                    objek: "pengguna",
                    keterangan: Some(format!("{username}: {alasan}")),
                    ..Default::default()
                },
            )
            .await?;
            return Err(AppError::TidakTerautentikasi("Nama pengguna atau kata sandi salah".into()));
        }
    };

    let token = buat_token();
    let mut tx = state.pool.begin().await?;
    // Bersihkan sesi kedaluwarsa milik pengguna ini.
    sqlx::query!(
        "DELETE FROM sesi_login WHERE id_pengguna = $1 AND terakhir_aktif < now() - make_interval(mins => $2)",
        akun.id_pengguna,
        state.config.session_idle_minutes,
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "INSERT INTO sesi_login (token_hash, id_pengguna) VALUES ($1, $2)",
        hash_token(&token),
        akun.id_pengguna
    )
    .execute(&mut *tx)
    .await?;
    log::catat(&mut *tx, Catatan::new(akun.id_pengguna, "login", "pengguna").id_objek(akun.id_pengguna)).await?;
    tx.commit().await?;

    let cookie = Cookie::build((NAMA_COOKIE, token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(state.config.cookie_secure);

    let pengguna = PenggunaLogin { id_pengguna: akun.id_pengguna, nama: akun.nama, username: akun.username, peran: akun.peran };
    Ok((jar.add(cookie), Json(pengguna)))
}

async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    if let Some(token) = jar.get(NAMA_COOKIE).map(|c| c.value().to_owned()) {
        let mut tx = state.pool.begin().await?;
        let id = sqlx::query_scalar!(
            "DELETE FROM sesi_login WHERE token_hash = $1 RETURNING id_pengguna",
            hash_token(&token)
        )
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = id {
            log::catat(&mut *tx, Catatan::new(id, "logout", "pengguna").id_objek(id)).await?;
        }
        tx.commit().await?;
    }
    let jar = jar.remove(Cookie::build(NAMA_COOKIE).path("/"));
    Ok((jar, Json(json!({ "pesan": "Berhasil logout" }))))
}

async fn saya(pengguna: PenggunaLogin) -> Json<PenggunaLogin> {
    Json(pengguna)
}
