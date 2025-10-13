// src/api/handlers/evals.rs
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::AppState;
use crate::config::EvalConfig;
use crate::runner;
use serde_json::json;

#[derive(Clone, Deserialize)]
pub struct RunEvalRequest {
    pub model: String,
    pub prompt: String,
    pub expected: Option<String>,
    pub judge_model: Option<String>,
    pub criteria: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct BatchEvalRequest {
    pub evals: Vec<RunEvalRequest>,
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
    pub results: Vec<EvalResponse>,
}

impl From<RunEvalRequest> for EvalConfig {
    fn from(req: RunEvalRequest) -> Self {
        EvalConfig {
            model: req.model,
            prompt: req.prompt,
            expected: req.expected,
            judge_model: req.judge_model,
            criteria: req.criteria,
            tags: Vec::new(),
            metadata: None,
        }
    }
}

pub async fn run_eval(
    state: web::Data<AppState>,
    req: web::Json<RunEvalRequest>,
) -> Result<HttpResponse> {
    let eval_id = Uuid::new_v4().to_string();
    let eval_config: EvalConfig = req.into_inner().into();

    match runner::run_eval(&state.config, &eval_config, &state.client).await {
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

            let response = EvalResponse {
                id: eval_id.clone(),
                status: status.to_string(),
                result: Some(result.clone()),
                error: None,
            };

            // Save to database
            if let Some(pool) = state.db_pool.as_ref() {
                let api_response = crate::models::ApiResponse {
                    id: eval_id,
                    status: status.to_string(),
                    result: crate::models::EvalResult::Success(result),
                };
                if let Err(e) = crate::database::save_evaluation(pool, &api_response).await {
                    log::error!("Failed to save evaluation to database: {}", e);
                }
            }

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            let error_string = e.to_string();
            Ok(HttpResponse::InternalServerError().json(EvalResponse {
                id: eval_id,
                status: "error".to_string(),
                result: None,
                error: Some(error_string),
            }))
        }
    }
}

pub async fn run_batch(
    state: web::Data<AppState>,
    req: web::Json<BatchEvalRequest>,
) -> Result<HttpResponse> {
    let batch_id = Uuid::new_v4().to_string();
    let total = req.evals.len();
    let req_body = req.into_inner();

    let eval_configs: Vec<EvalConfig> = req_body.evals.into_iter().map(Into::into).collect();

    let results = runner::run_batch_evals(
        &state.config,
        eval_configs,
        &state.client,
    ).await;

    let mut responses = Vec::new();
    let mut completed = 0;
    let mut passed = 0;
    let mut failed = 0;

    for result in results {
        let eval_id = Uuid::new_v4().to_string();
        
        match result {
            Ok(eval_result) => {
                completed += 1;
                
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

                responses.push(EvalResponse {
                    id: eval_id,
                    status: status.to_string(),
                    result: Some(eval_result),
                    error: None,
                });
            }
            Err(e) => {
                failed += 1;
                responses.push(EvalResponse {
                    id: eval_id,
                    status: "error".to_string(),
                    result: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(HttpResponse::Ok().json(BatchEvalResponse {
        batch_id,
        status: "completed".to_string(),
        total,
        completed,
        passed,
        failed,
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
    if let Some(pool) = state.db_pool.as_ref() {
        match crate::database::get_all_evaluations(pool).await {
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