use crate::config::AppConfig;
use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub client: Client,
    pub db_pool: Option<Arc<SqlitePool>>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Self {
        // Get the pool, convert Result to Option, then wrap in Arc
        let db_pool = match crate::database::init_db().await {
            Ok(pool) => Some(Arc::new(pool)),
            Err(e) => {
                eprintln!("⚠️  Failed to initialize database: {}", e);
                None
            }
        };

        Self {
            config: Arc::new(config),
            client: Client::new(),
            db_pool,  // Now it's Option<Arc<SqlitePool>>
        }
    }
}
