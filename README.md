[![Rust Tests](https://github.com/RGGH/evaluate/actions/workflows/test.yml/badge.svg)](https://github.com/RGGH/evaluate/actions/workflows/test.yml)
<div align="center">
  <img width="687" height="391" alt="Screenshot from 2025-10-13 21-00-43" src="https://github.com/user-attachments/assets/43b41099-8cbb-47e8-81c3-dbacf5b225a8" />
</div>

---

# Evaluate - An LLM Eval Framework

A lightweight, flexible evaluation framework for testing models with automated judging capabilities, supporting both Gemini and Ollama.

- Sqlite database for saving history
- Specify LLM provider for LLM and Judge
- batch evals to multiple providers/models
- API endpoints

## Todo
- [ ] Image Classifier Evals
- [ ] Add Anthropic and more...

# Getting started

### set up the .env file

You can either download the binary, compile from source (using Rust/Cargo) or try out in Docker

Whichever option (1 or 2) you use, you will need to set up your ```.env``` file 

You will need to create one with your text editor and add the following (add your own API keys)

Add into .env

(see env.example)
```bash
DATABASE_URL=sqlite:./data/evals.db
GEMINI_API_BASE=https://generativelanguage.googleapis.com
GEMINI_API_KEY=AIzaxxxxxxxxxxxxxxxxxxxxxxxxxxc
GEMINI_MODELS=gemini-2.5-pro,gemini-2.5-flash
OLLAMA_API_BASE=http://host.docker.internal:11434
OPENAI_API_BASE=https://api.openai.com/v1
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxxxxxxxxxx
OPENAI_MODELS=gpt-4o,gpt-4o-mini,gpt-3.5-turbo
RUST_LOG=info
```

## Option 1 : Try it out with Docker

### Linux
```bash
docker run --rm -it \
  --network host \
  --env-file .env \
  -v $(pwd)/data:/usr/local/bin/data \
  -e OLLAMA_API_BASE=http://localhost:11434 \
  evaluate:latest
```

### Mac    
```bash
docker run --rm -it -p 8080:8080 \
  --env-file .env \
  -v $(pwd)/data:/usr/local/bin/data \
  evaluate:latest
```

### Windoze (powershell)

```powershell
docker run --rm -it -p 8080:8080 `
  --env-file .env `
  -v ${PWD}/data:/usr/local/bin/data `
  evaluate:latest
```

## Option 2 : Install

```bash
# 1. Clone the repository
git clone git@github.com:RGGH/evaluate.git

# 2. Navigate into the new project directory
cd evaluate
```

Run the code (you will need RUST + Cargo installed)

```bash
cargo run
```

You should see output similar to:
```
[INFO] Starting database migration...
[INFO] Starting server at 127.0.0.1:8080
```
<img width="1208" height="580" alt="image" src="https://github.com/user-attachments/assets/b61d4b8c-0afb-4fb2-aabb-ff40ac6ad4ca" />


## Features

- 🚀 **Simple Configuration** - env file for API settings
- 📝 **JSON Eval Definitions** - Easy-to-write test cases
- 🤖 **AI-Powered Judging** - Use Gemini models to evaluate outputs semantically
- ⚡ **Async Execution** - Fast parallel evaluation support
- 🎯 **Flexible** - Test any Gemini model with any prompt

# Sample output - API - Gemini
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
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100   909  100   681  100   228     63     21  0:00:10  0:00:10 --:--:--   150
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
      "reasoning": "Verdict: PASS\n\nThe actual output correctly names Paris as the capital city, which is the core requirement of the evaluation criteria. Although it is a complete sentence rather than just the city name, it is semantically equivalent to the expected output. The necessary information is present and accurate.",
      "confidence": null
    },
    "timestamp": "2024-07-29T10:30:00.123456789+00:00"
  },
  "error": null
}
```

# Sample output Ollama
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

{
  "id": "3c7e8a90-9170-414e-9b33-981deac4007b",
  "status": "passed",
  "result": {
    "model": "ollama:llama3",
    "prompt": "What is the capital of France?",
    "model_output": "The capital of France is Paris.",
    "expected": "Paris",
    "judge_result": {
      "judge_model": "ollama:llama3",
      "verdict": "Pass",
      "reasoning": "Verdict: PASS\n\nThe actual output correctly names Paris as the capital city of France, which aligns with the expected output. Although the wording is slightly different, the semantic meaning and intent behind both outputs are identical, making it a pass according to the evaluation criteria.",
      "confidence": null
    },
    "timestamp": "2025-10-14T18:35:46.295118492+00:00",
    "latency_ms": 3726,
    "judge_latency_ms": 10691,
    "total_latency_ms": 14417
  },
  "error": null
}


```

# Batch Evals API 

You can set the provider in the json file and use a generic syntax for batch evals

```json
    "model": "gemini:gemini-1.5-flash-latest",
    "prompt": "What is the capital of France?",
    "expected": "Paris",
    "judge_model": "gemini:gemini-1.5-pro-latest"
 ```   

Call the endpoint ```api/v1/evals/batch``` and supply a modified/synthetic 'qa_sameple.json' file, or your own .json file

```bash
curl -X POST http://127.0.0.1:8080/api/v1/evals/batch \
-H "Content-Type: application/json" \
-d '@qa_sample.json' | jq
```


---

# Built in GUI 

### Single Eval
<img width="994" height="940" alt="Screenshot from 2025-10-14 19-27-23" src="https://github.com/user-attachments/assets/c773fb8e-f74e-4bc9-a169-cf418fd25057" />


### History
<img width="994" height="940" alt="Screenshot from 2025-10-14 19-27-14" src="https://github.com/user-attachments/assets/982935f7-98b8-47ac-ab09-7cf95356f3ec" />

# Built in Results Dashboard

<img width="1225" height="921" alt="Screenshot from 2025-10-15 13-01-10" src="https://github.com/user-attachments/assets/ab887445-e816-42f3-96c5-96f356a8f98c" />

<img width="1225" height="921" alt="Screenshot from 2025-10-15 13-01-25" src="https://github.com/user-attachments/assets/fcdd4516-ac83-41f8-95bd-c34626491e63" />

# API Endpoints

Base URL: `http://localhost:8080/api/v1`

## Health & System

| Method | Endpoint | Description | Request Body | Response |
|--------|----------|-------------|--------------|----------|
| GET | `/health` | Health check endpoint | - | `{"status": "healthy", "service": "eval-api", "version": "..."}` |
| GET | `/models` | List all available models | - | `{"models": ["gemini:model-name", "ollama:model-name", ...]}` |

## Evaluations

| Method | Endpoint | Description | Request Body | Response |
|--------|----------|-------------|--------------|----------|
| POST | `/evals/run` | Run a single evaluation | [RunEvalRequest](#runevalrequest) | [EvalResponse](#evalresponse) |
| POST | `/evals/batch` | Run multiple evaluations concurrently | Array of [EvalConfig](#evalconfig) | [BatchEvalResponse](#batchevalresponse) |
| GET | `/evals/history` | Get all evaluation history | - | [HistoryResponse](#historyresponse) |
| GET | `/evals/{id}` | Get specific evaluation result | - | Evaluation details |
| GET | `/evals/{id}/status` | Get evaluation status | - | `{"id": "...", "status": "...", "progress": 100}` |

## Experiments

| Method | Endpoint | Description | Request Body | Response |
|--------|----------|-------------|--------------|----------|
| POST | `/experiments` | Create a new experiment | [CreateExperimentRequest](#createexperimentrequest) | [ExperimentResponse](#experimentresponse) |
| GET | `/experiments/{id}` | Get experiment details | - | Experiment details with results |

## WebSocket

| Protocol | Endpoint | Description | Message Format |
|----------|----------|-------------|----------------|
| WS | `/ws` | Real-time evaluation updates | [EvalUpdate](#evalupdate) |

---

## Request/Response Schemas

### RunEvalRequest

```json
{
  "model": "gemini:gemini-1.5-flash-latest",
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
  "judge_model": "gemini:gemini-1.5-pro-latest",
  "criteria": "The explanation should be accurate and accessible",
  "tags": ["physics", "computing"],
  "metadata": {
    "category": "science",
    "difficulty": "advanced"
  }
}
```

**Fields:**
- `model` (required): Model identifier
- `prompt` (required): The prompt text
- `expected` (optional): Expected output
- `judge_model` (optional): Judge model identifier
- `criteria` (optional): Evaluation criteria
- `tags` (optional): Array of tags for organization
- `metadata` (optional): Additional metadata object

### EvalResponse

```json
{
  "id": "uuid-string",
  "status": "passed",
  "result": {
    "model": "gemini:gemini-1.5-flash-latest",
    "prompt": "What is 2+2?",
    "model_output": "2+2 equals 4",
    "expected": "4",
    "judge_result": {
      "judge_model": "gemini:gemini-1.5-pro-latest",
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
  "results": [
    // Array of EvalResponse objects
  ]
}
```

### HistoryResponse

```json
{
  "results": [
    {
      "id": "uuid-string",
      "status": "passed",
      "model": "gemini:gemini-1.5-flash-latest",
      "prompt": "What is 2+2?",
      "model_output": "4",
      "expected": "4",
      "judge_model": "gemini:gemini-1.5-pro-latest",
      "judge_verdict": "Pass",
      "judge_reasoning": "Correct answer",
      "error_message": null,
      "created_at": "2025-10-15T12:34:56Z"
    }
  ]
}
```

### CreateExperimentRequest

```json
{
  "name": "My Experiment",
  "description": "Testing various models on math problems",
  "eval_ids": ["eval-id-1", "eval-id-2", "eval-id-3"]
}
```

### ExperimentResponse

```json
{
  "id": "experiment-uuid",
  "name": "My Experiment",
  "status": "created",
  "created_at": "2025-10-15T12:34:56Z"
}
```

### EvalUpdate (WebSocket)

```json
{
  "id": "uuid-string",
  "status": "passed",
  "model": "gemini:gemini-1.5-flash-latest",
  "verdict": "Pass",
  "latency_ms": 450
}
```

**Broadcast events:** Sent in real-time for each evaluation completion

---

## Model Format

Models are specified in the format `provider:model_name`:

**Supported Providers:**
- `gemini:gemini-1.5-flash-latest`
- `gemini:gemini-1.5-pro-latest`
- `ollama:llama3`
- `ollama:gemma`
- `openai:gpt-4o`
- `openai:gpt-4o-mini`
- `openai:gpt-3.5-turbo`

If no provider is specified, `gemini` is used as the default.

---

## Example Usage

### cURL Examples

**Run a single evaluation:**
```bash
curl -X POST http://localhost:8080/api/v1/evals/run \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gemini:gemini-1.5-flash-latest",
    "prompt": "What is the capital of France?",
    "expected": "Paris",
    "judge_model": "gemini:gemini-1.5-pro-latest"
  }'
```

**Run batch evaluations:**
```bash
curl -X POST http://localhost:8080/api/v1/evals/batch \
  -H "Content-Type: application/json" \
  -d '[
    {
      "model": "gemini:gemini-1.5-flash-latest",
      "prompt": "What is 2+2?",
      "expected": "4"
    },
    {
      "model": "openai:gpt-4o",
      "prompt": "What is 3+3?",
      "expected": "6"
    }
  ]'
```

**Get evaluation history:**
```bash
curl http://localhost:8080/api/v1/evals/history
```

**Connect to WebSocket:**
```javascript
const ws = new WebSocket('ws://localhost:8080/api/v1/ws');
ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  console.log('Eval update:', update);
};
```



