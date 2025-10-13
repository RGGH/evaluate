pub mod handlers;
pub mod routes;

pub use routes::configure_routes;

use crate::config::AppConfig;
use crate::database;
use reqwest::Client;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub client: Client,
    pub db_pool: Option<SqlitePool>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let db_pool = match database::init_db().await {
            Ok(pool) => Some(pool),
            Err(e) => {
                log::error!("Failed to initialize database: {}", e);
                None
            }
        };
        Self { config, client: Client::new(), db_pool }
    }
}