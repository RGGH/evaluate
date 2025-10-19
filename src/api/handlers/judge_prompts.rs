// src/api/handlers/judge_prompts.rs
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::api::AppState;
use crate::database;

#[derive(Serialize)]
pub struct JudgePromptsResponse {
    pub prompts: Vec<database::JudgePrompt>,
}

#[derive(Serialize)]
pub struct JudgePromptResponse {
    pub prompt: database::JudgePrompt,
}

#[derive(Deserialize)]
pub struct CreateJudgePromptRequest {
    pub name: String,
    pub template: String,
    pub description: Option<String>,
    #[serde(default)]
    pub set_active: bool,
}

#[derive(Deserialize)]
pub struct SetActiveRequest {
    pub version: i64,
}

/// GET /api/v1/judge-prompts - Get all judge prompt versions
pub async fn get_all_judge_prompts(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::get_all_judge_prompts(pool).await {
                Ok(prompts) => Ok(HttpResponse::Ok().json(JudgePromptsResponse { prompts })),
                Err(e) => {
                    log::error!("Failed to fetch judge prompts: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to fetch judge prompts"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}

/// GET /api/v1/judge-prompts/active - Get the active judge prompt
pub async fn get_active_judge_prompt(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::get_active_judge_prompt(pool).await {
                Ok(prompt) => Ok(HttpResponse::Ok().json(JudgePromptResponse { prompt })),
                Err(e) => {
                    log::error!("Failed to fetch active judge prompt: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to fetch active judge prompt"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}

/// GET /api/v1/judge-prompts/{version} - Get a specific judge prompt by version
pub async fn get_judge_prompt_by_version(
    state: web::Data<AppState>,
    path: web::Path<i64>,
) -> Result<HttpResponse> {
    let version = path.into_inner();
    
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::get_judge_prompt_by_version(pool, version).await {
                Ok(prompt) => Ok(HttpResponse::Ok().json(JudgePromptResponse { prompt })),
                Err(sqlx::Error::RowNotFound) => {
                    Ok(HttpResponse::NotFound().json(serde_json::json!({
                        "error": format!("Judge prompt version {} not found", version)
                    })))
                }
                Err(e) => {
                    log::error!("Failed to fetch judge prompt: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to fetch judge prompt"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}

/// POST /api/v1/judge-prompts - Create a new judge prompt version
pub async fn create_judge_prompt(
    state: web::Data<AppState>,
    req: web::Json<CreateJudgePromptRequest>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::create_judge_prompt(
                pool,
                req.name.clone(),
                req.template.clone(),
                req.description.clone(),
                req.set_active,
            ).await {
                Ok(prompt) => {
                    println!("✅ Created judge prompt version {}: {}", prompt.version, prompt.name);
                    Ok(HttpResponse::Created().json(JudgePromptResponse { prompt }))
                }
                Err(e) => {
                    log::error!("Failed to create judge prompt: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create judge prompt"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}

/// PUT /api/v1/judge-prompts/active - Set a judge prompt version as active
pub async fn set_active_judge_prompt(
    state: web::Data<AppState>,
    req: web::Json<SetActiveRequest>,
) -> Result<HttpResponse> {
    match state.db_pool.as_ref() {
        Some(pool) => {
            match database::set_active_judge_prompt(pool, req.version).await {
                Ok(_) => {
                    println!("✅ Set judge prompt version {} as active", req.version);
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "message": format!("Judge prompt version {} is now active", req.version)
                    })))
                }
                Err(e) => {
                    log::error!("Failed to set active judge prompt: {}", e);
                    Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to set active judge prompt"
                    })))
                }
            }
        }
        None => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Database not initialized"
        }))),
    }
}
