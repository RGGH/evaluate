use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::api::AppState;
use crate::database;

#[derive(Serialize)]
pub struct PromptVersionsResponse {
    pub prompts: Vec<database::PromptVersion>,
}

#[derive(Serialize)]
pub struct PromptVersionResponse {
    pub prompt: database::PromptVersion,
}

#[derive(Serialize)]
pub struct PromptStatsResponse {
    pub stats: database::PromptStats,
}

#[derive(Deserialize)]
pub struct CreatePromptVersionRequest {
    pub name: String,
    pub prompt_template: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub set_active: bool,
}

/// GET /api/v1/prompt-versions - Get all prompt versions
pub async fn get_all_prompt_versions(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::get_all_prompt_versions(pool).await {
                Ok(prompts) => Ok(HttpResponse::Ok().json(PromptVersionsResponse { prompts })),
                Err(e) => {
                    log::error!("Failed to fetch prompt versions: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to fetch prompt versions"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}

/// GET /api/v1/prompt-versions/active - Get active prompt version
pub async fn get_active_prompt_version(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::get_active_prompt_version(pool).await {
                Ok(prompt) => Ok(HttpResponse::Ok().json(PromptVersionResponse { prompt })),
                Err(e) => {
                    log::error!("Failed to fetch active prompt version: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to fetch active prompt version"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}

/// POST /api/v1/prompt-versions - Create new prompt version
pub async fn create_prompt_version(
    state: web::Data<AppState>,
    req: web::Json<CreatePromptVersionRequest>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::create_prompt_version(
                pool,
                req.name.clone(),
                req.prompt_template.clone(),
                req.description.clone(),
                req.tags.clone(),
                req.set_active,
            ).await {
                Ok(prompt) => {
                    println!("✅ Created prompt version {}: {}", prompt.version, prompt.name);
                    Ok(HttpResponse::Created().json(PromptVersionResponse { prompt }))
                }
                Err(e) => {
                    log::error!("Failed to create prompt version: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create prompt version"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}

/// GET /api/v1/prompt-versions/{version}/stats - Get performance stats for a prompt version
pub async fn get_prompt_version_stats(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let version = path.into_inner();
    
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::get_prompt_version_stats(pool, version).await {
                Ok(stats) => Ok(HttpResponse::Ok().json(PromptStatsResponse { stats })),
                Err(e) => {
                    log::error!("Failed to fetch prompt version stats: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to fetch prompt version stats"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}
