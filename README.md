<div align="center">

  <img width="687" height="391" alt="Screenshot from 2025-10-13 21-00-43" src="https://github.com/user-attachments/assets/43b41099-8cbb-47e8-81c3-dbacf5b225a8" />

</div>

  [![Rust](https://github.com/RGGH/evaluate/actions/workflows/rust.yml/badge.svg)](https://github.com/RGGH/evaluate/actions/workflows/rust.yml) 
  # Evaluate - An LLM Eval Framework made in Rust

A lightweight, flexible evaluation framework for testing models with automated judging capabilities. (Gemini initially)

### Install

```bash
# 1. Clone the repository
git clone git@github.com:RGGH/evaluate.git

# 2. Navigate into the new project directory
cd evaluate
```

Add config.toml into src/


```bash
api_base = "https://generativelanguage.googleapis.com"
api_key = "AIzaSyAkQnssdafsdfasdfasxxxxxxxxxxxxxxxxxxx"
```

```
cargo run
```

You should see output similar to:
```
[INFO] Starting database migration...
[INFO] Starting server at 127.0.0.1:8080
```

## Features

- 🚀 **Simple Configuration** - TOML-based config for API settings
- 📝 **JSON Eval Definitions** - Easy-to-write test cases
- 🤖 **AI-Powered Judging** - Use Gemini models to evaluate outputs semantically
- ⚡ **Async Execution** - Fast parallel evaluation support
- 🎯 **Flexible** - Test any Gemini model with any prompt

# Sample output
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
    "timestamp": "2025-10-13T08:48:38.987866175+00:00"
  },
  "error": null
}
```
---

# Built in GUI 

### Single Eval

<img width="1402" height="979" alt="image" src="https://github.com/user-attachments/assets/c705cd51-e9b8-4308-b985-f837445f2ea4" />


### History

<img width="1402" height="979" alt="image" src="https://github.com/user-attachments/assets/03cdd052-8fd3-444b-8dec-dc5ce7ebfc9d" />
