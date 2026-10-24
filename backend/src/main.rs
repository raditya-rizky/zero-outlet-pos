use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;
use zero_outlet_pos::{AppState, Config, app, services::pengguna};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().max_connections(10).connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;

    if let (Ok(username), Ok(sandi)) = (std::env::var("ADMIN_USERNAME"), std::env::var("ADMIN_PASSWORD")) {
        let nama = std::env::var("ADMIN_NAMA").unwrap_or_else(|_| "Pemilik Toko".into());
        if pengguna::pastikan_admin(&pool, &nama, &username, &sandi).await? {
            tracing::info!(%username, "akun admin pertama dibuat");
        }
    }

    let config = Config::from_env();
    match &config.frontend_dist {
        Some(dist) => tracing::info!(dist = %dist.display(), "menyajikan frontend"),
        None => tracing::warn!("FRONTEND_DIST tidak ditemukan; hanya API yang disajikan"),
    }

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "server berjalan");

    axum::serve(listener, app(AppState::new(pool, config)))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
