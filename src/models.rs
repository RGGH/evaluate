// src/models.rs
use crate::runner;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub enum EvalResult {
    Success(runner::EvalResult),
    Error(ApiError),
}

#[derive(Serialize, Clone, Debug)]
pub struct ApiError {
    pub message: String,
}

#[derive(Serialize, Clone)]
pub struct ApiResponse {
    pub id: String,
    pub status: String,
    pub result: EvalResult,
}