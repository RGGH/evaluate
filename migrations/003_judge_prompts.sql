-- Create judge_prompts table
CREATE TABLE IF NOT EXISTS judge_prompts (
    version INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    template TEXT NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL
);

-- Insert a default judge prompt
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
