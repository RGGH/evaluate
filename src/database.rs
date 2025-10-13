use crate::models::{ApiResponse, EvalResult, JudgeResult};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;
use uuid::Uuid;

/// Initializes the database connection pool, creates the database file if it doesn't exist,
/// and runs migrations.
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_path = get_db_path()?;
    let db_url = format!("sqlite:{}", db_path.to_str().unwrap());

    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| sqlx::Error::Io(e))?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Determines the path for the SQLite database file within the user's config directory.
fn get_db_path() -> Result<PathBuf, sqlx::Error> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| sqlx::Error::Configuration("Cannot find config directory".into()))?
        .join("evaluate");

    Ok(config_dir.join("evals.db"))
}

/// Inserts a single evaluation result into the database.
pub async fn save_evaluation(pool: &SqlitePool, response: &ApiResponse) -> Result<(), sqlx::Error> {
    let id = response.id;
    let status = response.status.to_string();
    let created_at = response.result.timestamp.to_rfc3339();

    let (
        model_output,
        expected,
        judge_model,
        judge_verdict,
        judge_reasoning,
        error_message,
    ) = match &response.result {
        EvalResult::Success(res) => (
            Some(res.model_output.clone()),
            res.expected.clone(),
            res.judge_result.as_ref().map(|j| j.judge_model.clone()),
            res.judge_result
                .as_ref()
                .map(|j| j.verdict.to_string()),
            res.judge_result.as_ref().map(|j| j.reasoning.clone()),
            None,
        ),
        EvalResult::Error(err) => (None, None, None, None, None, Some(err.message.clone())),
    };

    sqlx::query!(
        r#"
        INSERT INTO evaluations (id, status, model, prompt, model_output, expected, judge_model, judge_verdict, judge_reasoning, error_message, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
        id, status, response.result.model(), response.result.prompt(), model_output, expected, judge_model, judge_verdict, judge_reasoning, error_message, created_at
    )
    .execute(pool)
    .await?;

    Ok(())
}