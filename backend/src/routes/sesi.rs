//! Sesi kasir: buka dengan modal awal, tutup dengan kas fisik, dan
//! rekonsiliasi kas beserta rekap nontunai (KF-19).

use axum::{Json, Router, extract::{Path, Query, State}, routing::{get, post}};
use serde_json::json;
use sqlx::PgConnection;

use crate::{
    AppError, AppResult, AppState,
    error::unique_violation,
    middleware::auth::PenggunaLogin,
    models::sesi::{BukaSesi, CariSesi, Rekap, Sesi, TutupSesi},
    services::{
        hitung,
        log::{self, Catatan},
    },
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sesi", get(daftar))
        .route("/sesi/buka", post(buka))
        .route("/sesi/aktif", get(aktif))
        .route("/sesi/{id}/rekap", get(rekap))
        .route("/sesi/{id}/tutup", post(tutup))
}

async fn ambil(conn: &mut PgConnection, id_sesi: i64) -> AppResult<Sesi> {
    sqlx::query_as!(
        Sesi,
        r#"SELECT s.id_sesi, s.id_pengguna, p.nama AS nama_kasir, s.waktu_buka, s.modal_awal, s.waktu_tutup,
                  s.kas_fisik, s.kas_seharusnya, s.selisih, s.alasan_selisih
           FROM sesi_kasir s JOIN pengguna p ON p.id_pengguna = s.id_pengguna
           WHERE s.id_sesi = $1"#,
        id_sesi
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or_else(|| AppError::tidak_ditemukan("Sesi kasir tidak ditemukan"))
}

/// Kasir hanya boleh melihat dan menutup sesinya sendiri (Tabel 2).
fn periksa_akses(pengguna: &PenggunaLogin, sesi: &Sesi) -> AppResult<()> {
    if pengguna.is_admin() || sesi.id_pengguna == pengguna.id_pengguna {
        Ok(())
    } else {
        Err(AppError::TidakBerhak)
    }
}

async fn hitung_rekap(conn: &mut PgConnection, sesi: Sesi) -> AppResult<Rekap> {
    let jual = sqlx::query!(
        r#"SELECT count(*) AS "jumlah!",
                  coalesce(sum(total) FILTER (WHERE metode_bayar = 'tunai'), 0)::bigint AS "tunai!",
                  coalesce(sum(total) FILTER (WHERE metode_bayar = 'transfer'), 0)::bigint AS "transfer!",
                  coalesce(sum(total) FILTER (WHERE metode_bayar = 'qris'), 0)::bigint AS "qris!"
           FROM transaksi WHERE id_sesi = $1 AND status = 'selesai'"#,
        sesi.id_sesi
    )
    .fetch_one(&mut *conn)
    .await?;

    let refund_tunai = sqlx::query_scalar!(
        r#"SELECT coalesce(sum(total_pengembalian), 0)::bigint AS "refund!"
           FROM retur WHERE id_sesi = $1 AND metode_pengembalian = 'tunai'"#,
        sesi.id_sesi
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok(Rekap {
        jumlah_transaksi: jual.jumlah,
        penjualan_tunai: jual.tunai,
        penjualan_transfer: jual.transfer,
        penjualan_qris: jual.qris,
        total_penjualan: jual.tunai + jual.transfer + jual.qris,
        refund_tunai,
        kas_seharusnya: hitung::kas_seharusnya(sesi.modal_awal, jual.tunai, refund_tunai),
        sesi,
    })
}

async fn buka(pengguna: PenggunaLogin, State(state): State<AppState>, Json(req): Json<BukaSesi>) -> AppResult<Json<Rekap>> {
    if req.modal_awal < 0 {
        return Err(AppError::validasi("Modal awal tidak boleh negatif"));
    }
    let mut tx = state.pool.begin().await?;
    let id_sesi = sqlx::query_scalar!(
        "INSERT INTO sesi_kasir (id_pengguna, modal_awal) VALUES ($1, $2) RETURNING id_sesi",
        pengguna.id_pengguna,
        req.modal_awal
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match unique_violation(&e) {
        Some(_) => AppError::konflik("Anda masih memiliki sesi kasir yang aktif. Tutup sesi tersebut terlebih dahulu."),
        None => e.into(),
    })?;
    log::catat(
        &mut *tx,
        Catatan::new(pengguna.id_pengguna, "buka_sesi", "sesi_kasir").id_objek(id_sesi).baru(json!({ "modal_awal": req.modal_awal })),
    )
    .await?;
    let sesi = ambil(&mut tx, id_sesi).await?;
    let rekap = hitung_rekap(&mut tx, sesi).await?;
    tx.commit().await?;
    Ok(Json(rekap))
}

async fn aktif(pengguna: PenggunaLogin, State(state): State<AppState>) -> AppResult<Json<Option<Rekap>>> {
    let mut conn = state.pool.acquire().await?;
    let id_sesi = sqlx::query_scalar!(
        "SELECT id_sesi FROM sesi_kasir WHERE id_pengguna = $1 AND waktu_tutup IS NULL",
        pengguna.id_pengguna
    )
    .fetch_optional(&mut *conn)
    .await?;
    let Some(id_sesi) = id_sesi else {
        return Ok(Json(None));
    };
    let sesi = ambil(&mut conn, id_sesi).await?;
    Ok(Json(Some(hitung_rekap(&mut conn, sesi).await?)))
}

async fn rekap(pengguna: PenggunaLogin, State(state): State<AppState>, Path(id): Path<i64>) -> AppResult<Json<Rekap>> {
    let mut conn = state.pool.acquire().await?;
    let sesi = ambil(&mut conn, id).await?;
    periksa_akses(&pengguna, &sesi)?;
    Ok(Json(hitung_rekap(&mut conn, sesi).await?))
}

async fn tutup(pengguna: PenggunaLogin, State(state): State<AppState>, Path(id): Path<i64>, Json(req): Json<TutupSesi>) -> AppResult<Json<Rekap>> {
    if req.kas_fisik < 0 {
        return Err(AppError::validasi("Kas fisik tidak boleh negatif"));
    }
    let mut tx = state.pool.begin().await?;
    // Kunci sesi agar tidak ada transaksi baru yang masuk selama penutupan.
    sqlx::query_scalar!("SELECT id_sesi FROM sesi_kasir WHERE id_sesi = $1 FOR UPDATE", id)
        .fetch_optional(&mut *tx)
        .await?;
    let sesi = ambil(&mut tx, id).await?;
    periksa_akses(&pengguna, &sesi)?;
    if sesi.waktu_tutup.is_some() {
        return Err(AppError::konflik("Sesi kasir ini sudah ditutup"));
    }

    let rekap = hitung_rekap(&mut tx, sesi).await?;
    let selisih = hitung::selisih_kas(req.kas_fisik, rekap.kas_seharusnya);
    let alasan = req.alasan_selisih.map(|a| a.trim().to_string()).filter(|a| !a.is_empty());
    if selisih != 0 && alasan.is_none() {
        return Err(AppError::validasi(format!("Terdapat selisih kas sebesar Rp{selisih}. Alasan selisih wajib diisi.")));
    }

    sqlx::query!(
        r#"UPDATE sesi_kasir
           SET waktu_tutup = now(), kas_fisik = $2, kas_seharusnya = $3, selisih = $4, alasan_selisih = $5
           WHERE id_sesi = $1"#,
        id,
        req.kas_fisik,
        rekap.kas_seharusnya,
        selisih,
        alasan,
    )
    .execute(&mut *tx)
    .await?;
    log::catat(
        &mut *tx,
        Catatan::new(pengguna.id_pengguna, "tutup_sesi", "sesi_kasir").id_objek(id).baru(json!({
            "kas_fisik": req.kas_fisik,
            "kas_seharusnya": rekap.kas_seharusnya,
            "selisih": selisih,
            "alasan_selisih": alasan,
            "penjualan_transfer": rekap.penjualan_transfer,
            "penjualan_qris": rekap.penjualan_qris,
        })),
    )
    .await?;

    let sesi = ambil(&mut tx, id).await?;
    let rekap = hitung_rekap(&mut tx, sesi).await?;
    tx.commit().await?;
    Ok(Json(rekap))
}

async fn daftar(pengguna: PenggunaLogin, State(state): State<AppState>, Query(q): Query<CariSesi>) -> AppResult<Json<Vec<Sesi>>> {
    let id_pengguna = if pengguna.is_admin() { q.id_pengguna } else { Some(pengguna.id_pengguna) };
    let rows = sqlx::query_as!(
        Sesi,
        r#"SELECT s.id_sesi, s.id_pengguna, p.nama AS nama_kasir, s.waktu_buka, s.modal_awal, s.waktu_tutup,
                  s.kas_fisik, s.kas_seharusnya, s.selisih, s.alasan_selisih
           FROM sesi_kasir s JOIN pengguna p ON p.id_pengguna = s.id_pengguna
           WHERE ($1::bigint IS NULL OR s.id_pengguna = $1)
             AND ($2::date IS NULL OR s.waktu_buka >= $2::date::timestamp AT TIME ZONE $4)
             AND ($3::date IS NULL OR s.waktu_buka < ($3::date + 1)::timestamp AT TIME ZONE $4)
           ORDER BY s.waktu_buka DESC
           LIMIT 200"#,
        id_pengguna,
        q.dari,
        q.sampai,
        state.config.zona_waktu,
    )
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}
