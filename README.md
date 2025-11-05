[![Rust Tests](https://github.com/RGGH/evaluate/actions/workflows/test.yml/badge.svg)](https://github.com/RGGH/evaluate/actions/workflows/test.yml)


<img width="864" height="561" alt="image" src="https://github.com/user-attachments/assets/47f11b9d-dbaa-4503-8d0f-87a15484a9aa" />

# Evaluate - An LLM Testing Framework

A lightweight, flexible evaluation 'eval' framework for testing models with automated judging capabilities, supporting Gemini, Anthropic, OpenAI, and Ollama.

<img width="150" height="150" alt="evaluate" src="https://github.com/user-attachments/assets/76bccec4-68fc-4a6c-a8d3-a84194822b2b" />

## LLM as a judge

Step 1: Build the Question for the Judge
We create a prompt that asks another AI model to evaluate the first model's answer. This prompt contains:

The rules for what counts as a good answer (you can customize these or use the defaults)
What the correct answer should be
What the model actually said
Simple instructions telling the judge how to evaluate

Step 2: Send to the Right AI Service

We figure out which AI service to use (like Anthropic, OpenAI, etc.) from the judge model name
We send the evaluation question to that service (using the same system we used for the first model)
We track how long it takes and how many tokens it uses

Step 3: Understand the Judge's Answer
We read the judge model's response and pull out:

The decision: Did it pass or fail? (We look for words like "PASS", "FAIL", "yes", or "no")
The explanation: Why did the judge decide this?
If we can't tell what the verdict is, we mark it as "Uncertain"

▶️ [Watch on YouTube](https://youtu.be/CyErTQbwiXA)

## Features

- SQLite database for saving history
- Specify LLM provider for both model and judge
- Batch evaluations to multiple providers/models
- API endpoints for developers to consume
- Built-in GUI and results dashboard
- Additional evaluation criteria options (exact match, semantic similarity, etc.)
- Python SDK available
- Real-time WebSocket updates

### Python SDK

If you want to use 'evaluate' via your own Python scripts or Jupyter Notebooks, you can use the SDK:

https://pypi.org/project/llmeval-sdk/

(Example usage with Python is shown on that PyPi page)

## Quick Start

### Prerequisites

You'll need:
- Docker (recommended) OR Rust/Cargo
- API keys for your LLM provider(s)

If you use Ollama:
```bash
ollama pull llama3
```

### Setup Environment Variables

Create a `.env` file in your project root (see `env.example`):

```bash
DATABASE_URL=sqlite:./data/evals.db
GEMINI_API_BASE=https://generativelanguage.googleapis.com
GEMINI_API_KEY=AIzaxxxxxxxxxxxxxxxxxxxxxxxxxxc
GEMINI_MODELS=gemini-2.5-pro,gemini-2.5-flash
OLLAMA_API_BASE=http://host.docker.internal:11434
OPENAI_API_BASE=https://api.openai.com/v1
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxxxxxxxxxx
OPENAI_MODELS=gpt-4o,gpt-4o-mini,gpt-3.5-turbo
ANTHROPIC_API_KEY=sk-placeholder-ant-a1b2c3d4e5f6-a1b2c3d4e5f6-a1b2c3d4e5f6-a1b2c3d4e5f6
ANTHROPIC_MODELS=claude-opus-4,claude-sonnet-4-5,claude-haiku-4
RUST_LOG=info
```

### Installation Options

#### Option 1: Docker (Recommended)

**Build the image:**
```bash
docker build -t evaluate:latest .
```

**Run on Linux:**
```bash
docker run --rm -it \
  --network host \
  --env-file .env \
  -v $(pwd)/data:/usr/local/bin/data \
  -e OLLAMA_API_BASE=http://localhost:11434 \
  evaluate:latest
```

**Run on Mac:**
```bash
docker run --rm -it -p 8080:8080 \
  --env-file .env \
  -v $(pwd)/data:/usr/local/bin/data \
  evaluate:latest
```

**Run on Windows (PowerShell):**
```powershell
docker run --rm -it -p 8080:8080 `
  --env-file .env `
  -v ${PWD}/data:/usr/local/bin/data `
  evaluate:latest
```

#### Option 2: Install from Source

```bash
# 1. Clone the repository
git clone git@github.com:RGGH/evaluate.git

# 2. Navigate into the project directory
cd evaluate

# 3. Run with Cargo (requires Rust/Cargo installed)
cargo run
```

You should see output similar to:
```
[INFO] Starting database migration...
[INFO] Starting server at 127.0.0.1:8080
```

Access the application at `http://localhost:8080`

## Usage Examples

### Single Evaluation (API)

**Gemini Example:**
```bash
curl -X POST http://127.0.0.1:8080/api/v1/evals/run \
-H "Content-Type: application/json" \
-d '{
  "model": "gemini-2.5-pro",             
  "prompt": "What is the capital of France?", 
  "expected": "Paris",
  "judge_model": "gemini-2.5-pro",             
  "criteria": "Does the output correctly name the capital city?"
}' | jq
```

**Response:**
```json
{
  "id": "619cd32a-4376-4969-ac48-0f25b37bc933",
  "status": "passed",
  "result": {
    "model": "gemini-2.5-pro",
    "prompt": "What is the capital of France?",
    "model_output": "The capital of France is **Paris**.",
    "expected": "Paris",
    "judge_result": {
      "judge_model": "gemini-2.5-pro",
      "verdict": "Pass",
      "reasoning": "Verdict: PASS\n\nThe actual output correctly names Paris as the capital city...",
      "confidence": null
    },
    "timestamp": "2024-07-29T10:30:00.123456789+00:00"
  },
  "error": null
}
```

**Ollama Example:**
```bash
curl -X POST http://127.0.0.1:8080/api/v1/evals/run \
-H "Content-Type: application/json" \
-d '{
  "model": "ollama:llama3",
  "prompt": "What is the capital of France?",
  "expected": "Paris",
  "judge_model": "ollama:llama3",
  "criteria": "Does the output correctly name the capital city?"
}' | jq
```

### Batch Evaluations

You can set the provider in the JSON file and use generic syntax for batch evals:

```json
{
  "model": "gemini:gemini-2.5-flash-latest",
  "prompt": "What is the capital of France?",
  "expected": "Paris",
  "judge_model": "gemini:gemini-2.5-pro-latest"
}
```

Call the `api/v1/evals/batch` endpoint:

```bash
curl -X POST http://127.0.0.1:8080/api/v1/evals/batch \
-H "Content-Type: application/json" \
-d '@qa_sample.json' | jq
```

```bash
curl -X POST http://127.0.0.1:8080/api/v1/evals/batch \
-H "Content-Type: application/json" \
-d '@qa_f1.json' | jq
```

### Built-in GUI

#### Single Eval Interface
<img width="994" height="940" alt="Screenshot from 2025-10-14 19-27-23" src="https://github.com/user-attachments/assets/c773fb8e-f74e-4bc9-a169-cf418fd25057" />

#### History View
<img width="994" height="940" alt="Screenshot from 2025-10-14 19-27-14" src="https://github.com/user-attachments/assets/982935f7-98b8-47ac-ab09-7cf95356f3ec" />

#### Results Dashboard
<img width="989" height="923" alt="Screenshot from 2025-10-17 17-17-04" src="https://github.com/user-attachments/assets/5ac129d3-5696-4f97-8506-25d89e4d1353" />

<img width="989" height="923" alt="Screenshot from 2025-10-17 17-17-12" src="https://github.com/user-attachments/assets/29102426-8c7c-43da-9dbe-983cbda32bfd" />

## API Reference

Base URL: `http://localhost:8080/api/v1`

### Health & System

| Method | Endpoint | Description | Response |
|--------|----------|-------------|----------|
| GET | `/health` | Health check endpoint | `{"status": "healthy", "service": "eval-api", "version": "..."}` |
| GET | `/models` | List all available models | `{"models": ["gemini:model-name", "ollama:model-name", ...]}` |

### Evaluations

| Method | Endpoint | Description | Request Body |
|--------|----------|-------------|--------------|
| POST | `/evals/run` | Run a single evaluation | `RunEvalRequest` |
| POST | `/evals/batch` | Run multiple evaluations concurrently | Array of `EvalConfig` |
| GET | `/evals/history` | Get all evaluation history | - |
| GET | `/evals/{id}` | Get specific evaluation result | - |
| GET | `/evals/{id}/status` | Get evaluation status | - |

### Judge Prompts

| Method | Endpoint | Description | Request Body |
|--------|----------|-------------|--------------|
| GET | `/judge-prompts` | Get all judge prompt versions | - |
| GET | `/judge-prompts/active` | Get the currently active judge prompt | - |
| GET | `/judge-prompts/{version}` | Get a specific judge prompt by version | - |
| POST | `/judge-prompts` | Create a new judge prompt version | `CreateJudgePromptRequest` |
| PUT | `/judge-prompts/active` | Set a judge prompt version as active | `{"version": 2}` |

#### Judge Prompt Examples

**Get all judge prompts:**
```bash
curl http://localhost:8080/api/v1/judge-prompts
```

**Create a new judge prompt:**
```bash
curl -X POST http://localhost:8080/api/v1/judge-prompts \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Strict Evaluator",
    "template": "Compare:\nExpected: {{expected}}\nActual: {{actual}}\nVerdict: PASS or FAIL",
    "description": "Requires exact semantic match",
    "set_active": true
  }'
```

**Set a version as active:**
```bash
curl -X PUT http://localhost:8080/api/v1/judge-prompts/active \
  -H "Content-Type: application/json" \
  -d '{"version": 2}'
```

### Experiments

| Method | Endpoint | Description | Request Body |
|--------|----------|-------------|--------------|
| POST | `/experiments` | Create a new experiment | `CreateExperimentRequest` |
| GET | `/experiments/{id}` | Get experiment details | - |

### WebSocket

| Protocol | Endpoint | Description |
|----------|----------|-------------|
| WS | `/ws` | Real-time evaluation updates |

**Connect to WebSocket:**
```javascript
const ws = new WebSocket('ws://localhost:8080/api/v1/ws');
ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Eval update:', update);
};
```

## Request/Response Schemas

### RunEvalRequest

```json
{
  "model": "gemini:gemini-2.5-flash-latest",
  "prompt": "What is 2+2?",
  "expected": "4",
  "judge_model": "gemini:gemini-1.5-pro-latest",
  "criteria": "The output should be mathematically correct"
}
```

**Fields:**
- `model` (required): Model identifier in format `provider:model_name`
- `prompt` (required): The prompt to send to the model
- `expected` (optional): Expected output for comparison
- `judge_model` (optional): Judge model for LLM-as-a-judge evaluation
- `criteria` (optional): Custom evaluation criteria

### EvalConfig

```json
{
  "model": "openai:gpt-4o",
  "prompt": "Explain quantum computing",
  "expected": "Quantum computing uses quantum bits...",
  "judge_model": "gemini:gemini-2.5-pro-latest",
  "criteria": "The explanation should be accurate and accessible",
  "tags": ["physics", "computing"],
  "metadata": {
    "category": "science",
    "difficulty": "advanced"
  }
}
```

### EvalResponse

```json
{
  "id": "uuid-string",
  "status": "passed",
  "result": {
    "model": "gemini:gemini-2.5-flash-latest",
    "prompt": "What is 2+2?",
    "model_output": "2+2 equals 4",
    "expected": "4",
    "judge_result": {
      "judge_model": "gemini:gemini-2.5-pro-latest",
      "verdict": "Pass",
      "reasoning": "The output correctly identifies that 2+2 equals 4...",
      "confidence": null
    },
    "timestamp": "2025-10-15T12:34:56Z",
    "latency_ms": 450,
    "judge_latency_ms": 320,
    "total_latency_ms": 770
  },
  "error": null
}
```

**Status values:** `"passed"`, `"failed"`, `"uncertain"`, `"completed"`, `"error"`

**Verdict values:** `"Pass"`, `"Fail"`, `"Uncertain"`

### BatchEvalResponse

```json
{
  "batch_id": "uuid-string",
  "status": "completed",
  "total": 10,
  "completed": 10,
  "passed": 8,
  "failed": 2,
  "average_model_latency_ms": 425,
  "average_judge_latency_ms": 315,
  "results": []
}
```

### Other Schemas

See full documentation for:
- `HistoryResponse`
- `CreateExperimentRequest`
- `ExperimentResponse`
- `CreateJudgePromptRequest`
- `JudgePrompt`
- `EvalUpdate` (WebSocket)

## Supported Models

Models are specified in the format `provider:model_name`:

**Gemini:**
- `gemini:gemini-2.5-flash-latest`
- `gemini:gemini-2.5-pro-latest`

**Ollama:**
- `ollama:llama3`
- `ollama:gemma`

**OpenAI:**
- `openai:gpt-4o`
- `openai:gpt-4o-mini`
- `openai:gpt-3.5-turbo`

**Anthropic:**
- `anthropic:claude-opus-4`
- `anthropic:claude-sonnet-4`
- `anthropic:claude-sonnet-4-5`
- `anthropic:claude-haiku-4`

If no provider is specified, `gemini` is used as the default.


### 📝 Judge Prompt Management

This framework now supports versioned and dynamically loaded judge prompts, allowing you to change the LLM evaluation criteria without restarting the server.

**Key Features:**
* **Version Control:** Prompts are stored in the database with version numbers.
* **API Control:** The active prompt can be set via a dedicated API endpoint.

**Default Prompt:** An initial default judge prompt is inserted automatically by the database migration.

### API Endpoints for Prompts

| Method | Endpoint | Description | Body |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/v1/judge-prompts/active` | Retrieves the currently active prompt template. | N/A |
| `POST` | `/api/v1/judge-prompts` | Creates a new prompt version. | `{name: "new prompt", template: "...", set_active: false}` |
| `PUT` | `/api/v1/judge-prompts/active` | **Sets a specific version as active.** | `{version: 3}` (Requires the version number) |

## Example: Create New Judge Prompt

```bash
curl -X POST 'http://127.0.0.1:8080/api/v1/judge-prompts' \
-H 'Content-Type: application/json' \
-d '{
    "name": "Relaxed Math Judge",
    "template": "You are an expert evaluator comparing two text outputs. When evaluating mathematical or factual answers, prioritize the core numerical or fact value. Ignore auxiliary text, equations, or prefixes (like \"The answer is\") if the core value is correct. EVALUATION CRITERIA: {{criteria}} EXPECTED OUTPUT: {{expected}} ACTUAL OUTPUT: {{actual}} INSTRUCTIONS: 1. Carefully compare both outputs 2. Provide your verdict as the first line: \"Verdict: PASS\" or \"Verdict: FAIL\" 3. Then explain your reasoning in 2-3 sentences. Your evaluation:",
    "description": "A prompt designed to be less strict than the default, allowing for correct answers that include extraneous text.",
    "set_active": false
}'
```

Key Components Explained:

    -X POST: Specifies the HTTP method.

    http://127.0.0.1:8080/api/v1/judge-prompts: Your local API endpoint for creating prompts.

    -H 'Content-Type: application/json': Tells the server to expect JSON data in the body.

    -d '...': The data payload containing the fields required by your create_judge_prompt handler:

        name: A human-readable identifier.

        template: The full new prompt text, which now includes instructions to be more flexible on matching.

        set_active: We set this to false because we usually create the prompt first, then manually activate it (Step 2).
        

Next Step: Activating the New Prompt

After running the POST command, the API will respond with the newly created JudgePrompt object, which includes its unique version number (e.g., version: 2).

You would then use a PUT request to make that new version the official, active prompt:


## Example curl command to set version 2 as active
```bash
curl -X PUT 'http://127.0.0.1:8080/api/v1/judge-prompts/active' \
-H 'Content-Type: application/json' \
-d '{"version": 2}'
```


## Understanding LLM-as-Judge Limitations

### Knowledge Recency
One major limitation of LLMs is knowledge recency. Since these models are trained on fixed datasets that quickly become outdated, they often struggle with topics that rely on the latest information — such as new laws, policies, or medical guidance. This means their judgements can be based on old or irrelevant data, leading to unreliable results. To keep them up to date, techniques like retrieval-augmented generation (RAG), regular fine-tuning, and continual learning can help ensure LLMs-as-judges have access to the most current knowledge when making decisions.

### Hallucination
Another key weakness is hallucination, where LLMs confidently generate information that isn't true. In an evaluation context, this could mean inventing fake references, misinterpreting facts, or fabricating evidence — all of which can undermine trust in their output. Building in robust fact-checking systems that verify claims against reliable sources is essential to reduce the impact of these errors and maintain fairness in judgement.

### Domain-Specific Knowledge Gaps
Lastly, LLMs often face domain-specific knowledge gaps. While they're great generalists, they can lack the deep understanding needed for complex areas like law, finance, or medicine. Integrating domain-specific knowledge graphs or using RAG to pull in expert information can help bridge this gap, allowing them to deliver more accurate and context-aware evaluations.

## Contributing

Thank you for your interest in contributing! 🎉

We welcome contributions of all kinds — bug fixes, improvements, documentation, examples, or new features. 🦀 Rust, Python, and front-end JS/TS contributions are all welcome. See current issues for ideas.

### How to Contribute

1. Fork the repository and create a new branch for your changes
2. Make your changes with clear, descriptive commit messages
3. Open a Pull Request explaining what you've done and why

Please make sure your code follows the existing style and passes any tests. For larger changes, feel free to open an issue first to discuss your approach.

By contributing, you agree that your work will be licensed under this project's license.

Thank you for helping make this project better! 💡

## References

- https://arxiv.org/html/2412.05579v2
- https://github.com/openai/evals
- https://learn.microsoft.com/en-us/azure/ai-foundry/how-to/evaluate-generative-ai-app#query-and-response-metric-requirements

## Roadmap

- [ ] Image Classifier Evals

![Application Demo](https://github.com/RGGH/evaluate/blob/main/assets/output.webp)
