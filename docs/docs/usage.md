---
sidebar_position: 2
---

# batch evals

Make batch eval using the API

## step 1 : edit the qa_sample.json in the project root
Alterrnative make your own, using the same structure as below

```json
[
  {
    "model": "gemini-2.5-flash",
    "prompt": "What is the capital of {{country}}?",
    "expected": "{{capital}}",
    "judge_model": "gemini-2.5-pro",
    "criteria": "The model must name the correct capital city of the country.",
    "tags": ["geography", "capital-cities"],
    "metadata": { "country": "France", "capital": "Paris" }
  },
  {
    "model": "gemini-2.5-flash",
    "prompt": "What is the capital of {{country}}?",
    "expected": "{{capital}}",
    "judge_model": "gemini-2.5-pro",
    "criteria": "The model must name the correct capital city of the country.",
    "tags": ["geography", "capital-cities"],
    "metadata": { "country": "Germany", "capital": "Berlin" }
  }
```

## step 2 : call the API endpoint (cURL example)
```bash
curl -X POST http://127.0.0.1:8080/api/v1/evals/batch \
-H "Content-Type: application/json" \
-d '@qa_sample.json' | jq
```

## Example output

```bash
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  7522  100  5236  100  2286    254    111  0:00:20  0:00:20 --:--:--  1607
{
  "batch_id": "0f20b672-8b6a-4496-af30-9dd37f52f7c3",
  "status": "completed",
  "total": 5,
  "completed": 5,
  "passed": 5,
  "failed": 0,
  "average_model_latency_ms": 3359,
  "average_judge_latency_ms": 9756,
  "results": [
    {
      "id": "85be2ffc-41c0-424d-8052-ecab1c21a5cf",
      "status": "passed",
      "result": {
        "model": "gemini-2.5-flash",
        "prompt": "What is the capital of France?",
        "model_output": "The capital of France is **Paris**.",
        "expected": "Paris",
        "judge_result": {
          "judge_model": "gemini-2.5-pro",
          "verdict": "Pass",
          "reasoning": "Verdict: PASS\nThe actual output correctly identifies Paris as the capital city, fulfilling the evaluation criteria. Although the response is a full sentence rather than just the city name, it contains the expected correct answer. The additional context is acceptable as it does not change the accuracy of the core information.",
          "confidence": null
        },
        "timestamp": "2025-10-14T15:41:10.851040066+00:00",
        "latency_ms": 767,
        "judge_latency_ms": 8885,
        "total_latency_ms": 9652
      },
      "error": null
    },
    {
      "id": "071f4fe4-4432-4f96-bf28-cff00585a1ab",
      "status": "passed",
      "result": {
        "model": "gemini-2.5-flash",
        "prompt": "What is the capital of Germany?",
        "model_output": "The capital of Germany is **Berlin**.",
        "expected": "Berlin",
        "judge_result": {
          "judge_model": "gemini-2.5-pro",
          "verdict": "Pass",
          "reasoning": "Verdict: PASS\nThe actual output correctly identifies Berlin as the capital of Germany. While it provides the answer in a full sentence instead of just the city name, it fully satisfies the evaluation criteria.",
          "confidence": null
        },
        "timestamp": "2025-10-14T15:41:12.375311912+00:00",
        "latency_ms": 578,
        "judge_latency_ms": 10593,
        "total_latency_ms": 11172
      },
      "error": null
    },
    {
      "id": "f25fae66-a396-4ca8-b35c-62e17417e704",
      "status": "passed",
      "result": {
        "model": "gemini-2.5-flash",
        "prompt": "What is the capital of Japan?",
        "model_output": "The capital of Japan is **Tokyo**.",
        "expected": "Tokyo",
        "judge_result": {
          "judge_model": "gemini-2.5-pro",
          "verdict": "Pass",
          "reasoning": "Verdict: PASS\nThe actual output correctly identifies Tokyo as the capital city, which aligns with the expected output. While it is a full sentence rather than just the city's name, it directly and accurately answers the implied question and meets the evaluation criteria. The core information required is present and correct.",
          "confidence": null
        },
        "timestamp": "2025-10-14T15:41:10.556247882+00:00",
        "latency_ms": 572,
        "judge_latency_ms": 8775,
        "total_latency_ms": 9348
      },
      "error": null
    },
    {
      "id": "8b5ba856-c700-43d3-9626-f6c76a6477c2",
      "status": "passed",
      "result": {
        "model": "gemini-2.5-flash",
        "prompt": "A cyclist travels at a constant speed of {{speed}} km/h. How long will it take them to travel {{distance}} km? Provide the answer in hours and minutes.",
        "model_output": "To calculate the time, we use the formula: Time = Distance / Speed.\n\nGiven:\nSpeed = {{speed}} km/h\nDistance = {{distance}} km\n\nTime = {{distance}} km / {{speed}} km/h\nTime = {{distance / speed}} hours\n\nNow, let's convert this into hours and minutes.\nThe whole number part is the hours.\nThe decimal part needs to be multiplied by 60 to get minutes.\n\n{{ (distance / speed) | floor }} hours is {{ (distance / speed) | floor }} hours.\nThe remaining minutes are {{ ((distance / speed) % 1) * 60 }} minutes.\nThis is {{ ((distance / speed) % 1) * 60 | round(0) }} minutes (rounded to the nearest minute).\n\nSo, it will take the cyclist {{ (distance / speed) | floor }} hours and {{ ((distance / speed) % 1) * 60 | round(0) }} minutes.",
        "expected": "It will take {{hours}} hours and {{minutes}} minutes.",
        "judge_result": {
          "judge_model": "gemini-2.5-pro",
          "verdict": "Pass",
          "reasoning": "Verdict: PASS\nThe model correctly calculates the travel time by dividing distance by speed and then converting the decimal result into hours and minutes. The final sentence clearly states the answer in the required format, matching the semantic structure of the expected output. The inclusion of the step-by-step calculation is additional context but does not violate the core instructions.",
          "confidence": null
        },
        "timestamp": "2025-10-14T15:41:16.139303226+00:00",
        "latency_ms": 3895,
        "judge_latency_ms": 11029,
        "total_latency_ms": 14926
      },
      "error": null
    },
    {
      "id": "6a11833c-b95b-4bc8-937f-ae15b66450d0",
      "status": "passed",
      "result": {
        "model": "gemini-2.5-flash",
        "prompt": "In one sentence, what is the main philosophical question explored in the book 'Do Androids Dream of Electric Sheep?' by Philip K. Dick?",
        "model_output": "What truly defines humanity and distinguishes it from advanced artificial life, especially concerning the role of empathy?",
        "expected": "The book explores the nature of empathy and what it truly means to be human in a world with sophisticated androids.",
        "judge_result": {
          "judge_model": "gemini-2.5-pro",
          "verdict": "Pass",
          "reasoning": "Verdict: PASS\nThe actual output is semantically identical to the expected one, successfully identifying the book's core philosophical theme. It correctly focuses on the definition of humanity and the role of empathy as the distinguishing factor from artificial life. Phrasing the theme as the central question the book explores is a perfectly valid and accurate way to convey the concept.",
          "confidence": null
        },
        "timestamp": "2025-10-14T15:41:21.703438926+00:00",
        "latency_ms": 10984,
        "judge_latency_ms": 9500,
        "total_latency_ms": 20486
      },
      "error": null
    }
  ]
}
```