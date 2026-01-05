mod api;
mod config;
mod error;
mod models;

use actix_web::{middleware::Logger, App, HttpServer, web};
use config::Config;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    /// Shared Postgres connection pool.
    pub db: sqlx::PgPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let cfg = Config::from_env();
    let addr: SocketAddr = cfg.bind_addr.parse().expect("invalid BIND_ADDR");

    let db_pool = PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .connect(&cfg.database_url)
        .await
        .expect("failed to connect to database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: db_pool.clone(),
            }))
            .wrap(Logger::default())
            .configure(api::configure)
    })
    .bind(addr)?
    .run()
    .await
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}
