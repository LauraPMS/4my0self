use axum::{
    routing::{get, post},
    Router, Extension,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod error;
mod models;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL manquante");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Application automatique des migrations au démarrage
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/api/auth/register", post(routes::auth::register))
        .route("/api/auth/login", post(routes::auth::login))
        .route("/api/auth/logout", post(routes::auth::logout))
        .route("/api/modules", get(routes::modules::list_modules).post(routes::modules::toggle_module))
        .layer(CookieManagerLayer::new())
        .layer(Extension(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Core Engine écoute sur {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}