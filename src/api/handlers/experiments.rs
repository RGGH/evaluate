// src/api/handlers/experiments.rs
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateExperimentRequest {
    pub name: String,
    pub description: Option<String>,
    pub eval_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct ExperimentResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
}

pub async fn create_experiment(
    req: web::Json<CreateExperimentRequest>,
) -> Result<HttpResponse> {
    let experiment_id = Uuid::new_v4().to_string();
    
    Ok(HttpResponse::Created().json(ExperimentResponse {
        id: experiment_id,
        name: req.name.clone(),
        status: "created".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

pub async fn get_experiment(path: web::Path<String>) -> Result<HttpResponse> {
    let experiment_id = path.into_inner();
    
    Ok(HttpResponse::Ok().json(json!({
        "id": experiment_id,
        "name": "Mock Experiment",
        "status": "completed",
        "results": {
            "total_evals": 10,
            "passed": 8,
            "failed": 2
        }
    })))
}
