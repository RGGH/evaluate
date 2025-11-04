-- ========================================
-- 20251104162000_initial_schema.sql
-- Initial schema for fresh DB
-- ========================================

-- Create evaluations table
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
    created_at TEXT NOT NULL,
    judge_prompt_version INTEGER
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_evaluations_created_at ON evaluations(created_at);
CREATE INDEX IF NOT EXISTS idx_evaluations_status ON evaluations(status);
CREATE INDEX IF NOT EXISTS idx_evaluations_model ON evaluations(model);
CREATE INDEX IF NOT EXISTS idx_evaluations_judge_prompt_version ON evaluations(judge_prompt_version);

-- Create judge_prompts table
CREATE TABLE IF NOT EXISTS judge_prompts (
    version INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    template TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default judge prompt
INSERT INTO judge_prompts (name, template, description, is_active, created_at)
VALUES (
    'Default Judge Prompt',
    'You are an expert evaluator comparing two text outputs.

EVALUATION CRITERIA:
{{criteria}}

EXPECTED OUTPUT:
{{expected}}

ACTUAL OUTPUT:
{{actual}}

INSTRUCTIONS:
1. Carefully compare both outputs
2. Consider semantic equivalence, not just exact wording
3. Provide your verdict as the first line: "Verdict: PASS" or "Verdict: FAIL"
4. Then explain your reasoning in 2-3 sentences

Your evaluation:',
    'Default prompt for LLM-as-a-judge evaluation',
    TRUE,
    datetime('now')
);

