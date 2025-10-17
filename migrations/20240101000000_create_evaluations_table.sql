-- Create the evaluations table with all columns
CREATE TABLE IF NOT EXISTS evaluations (
    id TEXT PRIMARY KEY NOT NULL,
    status TEXT,
    model TEXT,
    prompt TEXT,
    model_output TEXT,
    expected TEXT,
    judge_model TEXT,
    judge_verdict TEXT,
    judge_reasoning TEXT,
    error_message TEXT,
    latency_ms INTEGER,
    judge_latency_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    judge_input_tokens INTEGER,
    judge_output_tokens INTEGER,
    created_at TEXT NOT NULL
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_evaluations_created_at ON evaluations(created_at);
CREATE INDEX IF NOT EXISTS idx_evaluations_status ON evaluations(status);
CREATE INDEX IF NOT EXISTS idx_evaluations_model ON evaluations(model);
