use crate::config::AppConfig;
use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub client: Client,
    pub db_pool: Arc<Option<SqlitePool>>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        let db_pool = crate::database::init_db()
            .await
            .ok();

        Self {
            config: Arc::new(config),
            client: Client::new(),
            db_pool: Arc::new(db_pool),
        }
    }
}