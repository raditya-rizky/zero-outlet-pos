//! Aplikasi kasir Zero Outlet: REST API (Axum) + penyajian SPA Vue.

pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;

use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub use config::Config;
pub use error::AppError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self { pool, config: Arc::new(config) }
    }
}

/// Router lengkap: `/api/*` untuk REST API, selain itu berkas statis SPA.
pub fn app(state: AppState) -> Router {
    let mut router = Router::new().nest("/api", routes::router());

    if let Some(dist) = &state.config.frontend_dist {
        // Rute SPA yang tidak dikenal dikembalikan ke index.html agar ditangani Vue Router.
        let spa = ServeDir::new(dist).fallback(ServeFile::new(dist.join("index.html")));
        router = router.fallback_service(spa);
    }

    router.layer(TraceLayer::new_for_http()).with_state(state)
}
