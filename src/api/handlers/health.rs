// src/api/handlers/health.rs
use actix_web::{HttpResponse, Result};
use serde_json::json;

pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "eval-api",
        "version": env!("CARGO_PKG_VERSION")
    })))
}
