-- migrations/20240101000000_create_evaluations_table.sql
CREATE TABLE IF NOT EXISTS evaluations (
    id TEXT PRIMARY KEY,
    status TEXT,
    model TEXT,
    prompt TEXT,
    model_output TEXT,
    expected TEXT,
    judge_model TEXT,
    judge_verdict TEXT,
    judge_reasoning TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL
);

