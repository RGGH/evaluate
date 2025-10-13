use actix_web::{web, HttpResponse, Result};
use serde::Serialize;
use crate::api::AppState;

#[derive(Serialize)]
pub struct HistoryResponse {
    pub results: Vec<crate::database::HistoryEntry>,
}

pub async fn get_history(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match crate::database::get_all_evaluations(pool).await {
                Ok(results) => Ok(HttpResponse::Ok().json(HistoryResponse { results })),
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to fetch history"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}
