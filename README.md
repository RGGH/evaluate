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
- [ ] Visualize output
- [ ] Image Classifier Evals
- [ ] Add OpenAI, Anthropic and more...

### Install

```bash
# 1. Clone the repository
git clone git@github.com:RGGH/evaluate.git

# 2. Navigate into the new project directory
cd evaluate
```

Add into .env

(see env.example)

```bash
DATABASE_URL=sqlite:data/evals.db

api_base = "https://generativelanguage.googleapis.com"
api_key = "AIzaSyAkQnxxxxxxxxxxxxxxxx"


GEMINI_MODELS=gemini-2.5-pro,gemini-2.5-flash,gemini-1.5-pro-latest,gemini-1.5-flash-latest

OLLAMA_API_BASE="http://localhost:11434"
```

Run the code

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

Call the endpoint

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


