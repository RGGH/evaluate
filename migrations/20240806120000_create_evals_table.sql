-- Create the evaluations table
CREATE TABLE IF NOT EXISTS evaluations (
    id TEXT PRIMARY KEY NOT NULL,
    status TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt TEXT NOT NULL,
    model_output TEXT,
    expected TEXT,
    judge_model TEXT,
    judge_verdict TEXT,
    judge_reasoning TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL
);
