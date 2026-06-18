mod config;
mod errors;
mod handlers;
mod middleware;
mod models;
mod response;
mod routes;
mod seeder;
mod services;
mod utils;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: AnyPool,
    pub config: Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let db_url = config.database_url();

    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    run_migrations(&pool, &config).await?;
    seeder::seed(&pool).await?;

    std::fs::create_dir_all(&config.storage_path)?;

    let state = AppState { pool, config: config.clone() };
    let storage_dir = PathBuf::from(&config.storage_path);

    let app = routes::build_router(state)
        .nest_service("/storage/photos", ServeDir::new(storage_dir))
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = format!("0.0.0.0:{}", config.app_port).parse()?;
    info!("Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_migrations(pool: &AnyPool, config: &Config) -> anyhow::Result<()> {
    let sql = include_str!("../migrations/001_init.sql");
    for statement in sql.split(';') {
        let stmt = statement.trim();
        if stmt.is_empty() { continue; }
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}
