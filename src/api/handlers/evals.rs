// src/api/handlers/evals.rs - Complete fixed version
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::AppState;
use crate::api::handlers::ws::{WsBroker, EvalUpdate};
use crate::config::EvalConfig;
use crate::runner;
use crate::errors::EvalError;
use serde_json::json;

#[derive(Clone, Deserialize)]
pub struct RunEvalRequest {
    pub model: String,
    pub prompt: String,
    pub expected: Option<String>,
    pub judge_model: Option<String>,
    pub criteria: Option<String>,
}

#[derive(Serialize)]
pub struct EvalResponse {
    pub id: String,
    pub status: String,
    pub result: Option<runner::EvalResult>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct BatchEvalResponse {
    pub batch_id: String,
    pub status: String,
    pub total: usize,
    pub completed: usize,
    pub passed: usize,
    pub failed: usize,
    pub average_model_latency_ms: u64,
    pub average_judge_latency_ms: u64,
    pub results: Vec<EvalResponse>,
}

pub async fn run_eval(
    state: web::Data<AppState>,
    broker: web::Data<WsBroker>,
    req: web::Json<RunEvalRequest>,
) -> Result<HttpResponse> {
    let eval_id = Uuid::new_v4().to_string();
    let req_body = req.into_inner();
    let eval_config = EvalConfig {
        model: req_body.model.clone(),
        prompt: req_body.prompt,
        expected: req_body.expected,
        judge_model: req_body.judge_model,
        criteria: req_body.criteria,
        tags: Vec::new(),
        metadata: None,
    };

    // Extract the pool reference properly for the new Option<Arc<SqlitePool>> structure
    let db_pool_ref = state.db_pool.as_ref().map(|arc| arc.as_ref());
    
    match runner::run_eval_with_pool(&state.config, &eval_config, &state.client, db_pool_ref).await {
        Ok(result) => {
            let status = if let Some(judge) = &result.judge_result {
                match judge.verdict {
                    runner::JudgeVerdict::Pass => "passed",
                    runner::JudgeVerdict::Fail => "failed",
                    runner::JudgeVerdict::Uncertain => "uncertain",
                }
            } else {
                "completed"
            };

            // Broadcast via WebSocket
            broker.broadcast(EvalUpdate {
                id: eval_id.clone(),
                status: status.to_string(),
                model: Some(req_body.model),
                verdict: result.judge_result.as_ref().map(|j| j.verdict.to_string()),
                latency_ms: Some(result.latency_ms),
            }).await;

            let response = EvalResponse {
                id: eval_id.clone(),
                status: status.to_string(),
                result: Some(result.clone()),
                error: None,
            };

            // Save to database
            if let Some(pool_arc) = state.db_pool.as_ref() {
                println!("💾 Saving successful evaluation to database: {}", eval_id);
                let api_response = crate::models::ApiResponse {
                    id: eval_id.clone(),
                    status: status.to_string(),
                    result: crate::models::EvalResult::Success(result),
                };
                match crate::database::save_evaluation(pool_arc, &api_response).await {
                    Ok(_) => println!("✅ Successfully saved evaluation {} to database", eval_id),
                    Err(e) => {
                        eprintln!("❌ Failed to save evaluation to database: {}", e);
                        log::error!("Database save error: {:?}", e);
                    }
                }
            } else {
                eprintln!("⚠️  Database pool is None - evaluation not saved!");
            }

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            let error_string = e.to_string();
            
            let status_code = match &e {
                EvalError::ProviderNotFound(_) | EvalError::Config(_) => 400,
                EvalError::ModelFailure { .. } => 400,
                _ => 500,
            };

            // Broadcast error via WebSocket
            broker.broadcast(EvalUpdate {
                id: eval_id.clone(),
                status: "error".to_string(),
                model: Some(req_body.model.clone()),
                verdict: None,
                latency_ms: None,
            }).await;

            let response = EvalResponse {
                id: eval_id.clone(),
                status: "error".to_string(),
                result: None,
                error: Some(error_string.clone()),
            };

            // Save error to database
            if let Some(pool_arc) = state.db_pool.as_ref() {
                println!("💾 Saving error evaluation to database: {}", eval_id);
                let api_response = crate::models::ApiResponse {
                    id: eval_id.clone(),
                    status: "error".to_string(),
                    result: crate::models::EvalResult::Error(crate::models::ApiError {
                        message: error_string.clone(),
                    }),
                };
                match crate::database::save_evaluation(pool_arc, &api_response).await {
                    Ok(_) => println!("✅ Successfully saved error evaluation {} to database", eval_id),
                    Err(e) => {
                        eprintln!("❌ Failed to save error evaluation to database: {}", e);
                        log::error!("Database save error: {:?}", e);
                    }
                }
            } else {
                eprintln!("⚠️  Database pool is None - error evaluation not saved!");
            }

            match status_code {
                400 => Ok(HttpResponse::BadRequest().json(response)),
                _ => Ok(HttpResponse::InternalServerError().json(response)),
            }
        }
    }
}

pub async fn run_batch(
    state: web::Data<AppState>,
    broker: web::Data<WsBroker>,
    eval_configs: web::Json<Vec<EvalConfig>>,
) -> Result<HttpResponse> {
    let batch_id = Uuid::new_v4().to_string();
    let total = eval_configs.len();

    // Extract the pool reference properly for the new Option<Arc<SqlitePool>> structure
    let db_pool_ref = state.db_pool.as_ref().map(|arc| arc.as_ref());
    
    let results = runner::run_batch_evals_with_pool(
        &state.config,
        eval_configs.into_inner(),
        &state.client,
        db_pool_ref,
    ).await;

    let mut responses = Vec::new();
    let mut completed = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut total_model_latency = 0;
    let mut model_latency_count = 0;
    let mut total_judge_latency = 0;
    let mut judge_latency_count = 0;

    for result in results {
        let eval_id = Uuid::new_v4().to_string();
        
        match result {
            Ok(eval_result) => {
                completed += 1;
                total_model_latency += eval_result.latency_ms;
                model_latency_count += 1;
                if let Some(judge_latency) = eval_result.judge_latency_ms {
                    total_judge_latency += judge_latency;
                    judge_latency_count += 1;
                }
                
                let status = if let Some(judge) = &eval_result.judge_result {
                    match judge.verdict {
                        runner::JudgeVerdict::Pass => {
                            passed += 1;
                            "passed"
                        }
                        runner::JudgeVerdict::Fail => {
                            failed += 1;
                            "failed"
                        }
                        runner::JudgeVerdict::Uncertain => "uncertain",
                    }
                } else {
                    "completed"
                };

                broker.broadcast(EvalUpdate {
                    id: eval_id.clone(),
                    status: status.to_string(),
                    model: Some(eval_result.model.clone()),
                    verdict: eval_result.judge_result.as_ref().map(|j| j.verdict.to_string()),
                    latency_ms: Some(eval_result.latency_ms),
                }).await;

                let response = EvalResponse {
                    id: eval_id.clone(),
                    status: status.to_string(),
                    result: Some(eval_result.clone()),
                    error: None,
                };

                if let Some(pool_arc) = state.db_pool.as_ref() {
                    let api_response = crate::models::ApiResponse {
                        id: eval_id,
                        status: status.to_string(),
                        result: crate::models::EvalResult::Success(eval_result),
                    };
                    if let Err(e) = crate::database::save_evaluation(pool_arc, &api_response).await {
                        log::error!("Failed to save batch evaluation to database: {}", e);
                    }
                }
                responses.push(response);
            }
            Err(e) => {
                failed += 1;
                let error_string = e.to_string();
                
                broker.broadcast(EvalUpdate {
                    id: eval_id.clone(),
                    status: "error".to_string(),
                    model: None,
                    verdict: None,
                    latency_ms: None,
                }).await;

                let response = EvalResponse {
                    id: eval_id.clone(),
                    status: "error".to_string(),
                    result: None,
                    error: Some(error_string.clone()),
                };

                if let Some(pool_arc) = state.db_pool.as_ref() {
                    let api_response = crate::models::ApiResponse {
                        id: eval_id,
                        status: "error".to_string(),
                        result: crate::models::EvalResult::Error(crate::models::ApiError {
                            message: error_string,
                        }),
                    };
                    if let Err(e) = crate::database::save_evaluation(pool_arc, &api_response).await {
                        log::error!("Failed to save batch error to database: {}", e);
                    }
                }

                responses.push(response);
            }
        }
    }

    let average_model_latency_ms = if model_latency_count > 0 { total_model_latency / model_latency_count as u64 } else { 0 };
    let average_judge_latency_ms = if judge_latency_count > 0 { total_judge_latency / judge_latency_count as u64 } else { 0 };

    Ok(HttpResponse::Ok().json(BatchEvalResponse {
        batch_id,
        status: "completed".to_string(),
        total,
        completed,
        passed,
        failed,
        average_model_latency_ms,
        average_judge_latency_ms,
        results: responses,
    }))
}

pub async fn get_eval(path: web::Path<String>) -> Result<HttpResponse> {
    let eval_id = path.into_inner();
    
    Ok(HttpResponse::Ok().json(json!({
        "id": eval_id,
        "status": "completed",
        "message": "This endpoint would return stored eval results"
    })))
}

pub async fn get_status(path: web::Path<String>) -> Result<HttpResponse> {
    let eval_id = path.into_inner();
    
    Ok(HttpResponse::Ok().json(json!({
        "id": eval_id,
        "status": "completed",
        "progress": 100
    })))
}

#[derive(Serialize)]
pub struct HistoryResponse {
    pub results: Vec<crate::database::HistoryEntry>,
}

pub async fn get_history(state: web::Data<AppState>) -> Result<HttpResponse> {
    if let Some(pool_arc) = state.db_pool.as_ref() {
        match crate::database::get_all_evaluations(pool_arc).await {
            Ok(history) => Ok(HttpResponse::Ok().json(HistoryResponse { results: history })),
            Err(e) => {
                log::error!("Failed to fetch evaluation history: {}", e);
                Ok(HttpResponse::InternalServerError()
                    .json(json!({"error": "Failed to load history from database."})))
            }
        }
    } else {
        Ok(HttpResponse::Ok().json(HistoryResponse { results: vec![] }))
    }
}

#[derive(Serialize)]
pub struct ModelsResponse {
    pub models: Vec<String>,
}

pub async fn get_models(state: web::Data<AppState>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(ModelsResponse { models: state.config.models.clone() }))
}
