# Eval Framework

A lightweight, flexible evaluation framework for testing models with automated judging capabilities. (Gemini initially)

## Features

- 🚀 **Simple Configuration** - TOML-based config for API settings
- 📝 **JSON Eval Definitions** - Easy-to-write test cases
- 🤖 **AI-Powered Judging** - Use Gemini models to evaluate outputs semantically
- ⚡ **Async Execution** - Fast parallel evaluation support
- 🎯 **Flexible** - Test any Gemini model with any prompt

## Installation

### Prerequisites

- Rust 1.89+ (install from [rustup.rs](https://rustup.rs))
- Google AI Studio API key ([get one here](https://makersuite.google.com/app/apikey))

### Setup

```bash
# Clone the repository
git clone https://github.com/RGGH/evaluate
cd evaluate

# Build the project
cargo build --release
```

## Quick Start

### 1. Configure API Access

Create `config.toml` in the project root:

```toml
api_base = "https://generativelanguage.googleapis.com"
api_key = "YOUR_GEMINI_API_KEY_HERE"
evals = [
    "evals/math.json",
    "evals/reasoning.json"
]
```

### 2. Create Your First Eval

Create `evals/math.json`:

```json
{
  "model": "gemini-2.5-flash",
  "prompt": "What is 15 * 23?",
  "expected": "345",
  "judge_model": "gemini-2.5-flash"
}
```

### 3. Run Evaluations

```bash
cargo run --release
```

Expected output:
```
📡 Calling: https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent
📥 Response status: 200 OK
🔹 Model Output [gemini-2.5-flash]: 345
🔸 Judge Output [gemini-2.5-flash]: YES
```

## Usage

### Eval File Format

Each eval is a JSON file with the following structure:

```json
{
  "model": "gemini-2.5-pro",           // Required: Model to test
  "prompt": "Your prompt here",         // Required: Input prompt
  "expected": "Expected output",        // Optional: For judging
  "judge_model": "gemini-2.5-flash"    // Optional: Model to judge output
}
```

**Fields:**
- `model` - The Gemini model to evaluate (e.g., `gemini-2.5-pro`, `gemini-2.5-flash`)
- `prompt` - The input text to send to the model
- `expected` - (Optional) The expected output. If provided, requires `judge_model`
- `judge_model` - (Optional) Model to use for semantic comparison of output vs expected

### Available Models

- `gemini-2.5-pro` - Most capable, best for complex tasks
- `gemini-2.5-flash` - Faster, cost-effective for simple tasks


### Example Eval Scenarios

#### Simple Output Test (No Judging)

```json
{
  "model": "gemini-2.5-flash",
  "prompt": "Write a haiku about coding"
}
```

#### Math Problem with Judging

```json
{
  "model": "gemini-2.5-pro",
  "prompt": "If a train travels 120 km in 2 hours, what is its average speed?",
  "expected": "60 km/h",
  "judge_model": "gemini-2.5-flash"
}
```

#### Reasoning Test

```json
{
  "model": "gemini-2.5-pro",
  "prompt": "Is it ethical to use AI for medical diagnosis? Give pros and cons.",
  "expected": "AI can improve accuracy and speed but raises concerns about liability and the need for human oversight.",
  "judge_model": "gemini-2.5-pro"
}
```

#### Code Generation

```json
{
  "model": "gemini-2.5-pro",
  "prompt": "Write a Python function to reverse a string",
  "expected": "A function that takes a string and returns it reversed",
  "judge_model": "gemini-2.5-flash"
}
```

### Organizing Evals

Create an `evals/` directory to organize your test cases:

```
evals/
├── math/
│   ├── basic_arithmetic.json
│   └── word_problems.json
├── reasoning/
│   ├── logical_puzzles.json
│   └── ethical_dilemmas.json
└── coding/
    ├── python_basics.json
    └── algorithms.json
```

Update `config.toml` to reference all evals:

```toml
api_base = "https://generativelanguage.googleapis.com"
api_key = "YOUR_API_KEY"
evals = [
    "evals/math/basic_arithmetic.json",
    "evals/math/word_problems.json",
    "evals/reasoning/logical_puzzles.json",
    "evals/coding/python_basics.json"
]
```

## Advanced Usage

### Batch Testing

Create multiple evals to test model consistency:

```bash
# Generate 10 similar evals with slight variations
for i in {1..10}; do
  cat > "evals/batch_$i.json" << EOF
{
  "model": "gemini-2.5-flash",
  "prompt": "What is $(($i * 7))?",
  "expected": "$(($i * 7))",
  "judge_model": "gemini-2.5-flash"
}
EOF
done
```

### Compare Models

Test the same prompt across different models:

```json
// evals/compare_pro.json
{
  "model": "gemini-2.5-pro",
  "prompt": "Explain quantum entanglement in simple terms"
}

// evals/compare_flash.json
{
  "model": "gemini-2.5-flash",
  "prompt": "Explain quantum entanglement in simple terms"
}
```

### Custom Judge Prompts

For more sophisticated judging, you could extend the framework to support custom judge prompts (future feature).

## Configuration Reference

### config.toml

| Field | Type | Description |
|-------|------|-------------|
| `api_base` | string | Gemini API base URL |
| `api_key` | string | Your Google AI Studio API key |
| `evals` | array | List of eval JSON file paths |

### Eval JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `model` | string | Yes | Gemini model identifier |
| `prompt` | string | Yes | Input prompt to evaluate |
| `expected` | string | No | Expected output for judging |
| `judge_model` | string | No | Model to use as judge |

## Troubleshooting

### "Empty response from API"

- Check your API key is valid
- Verify you have API access enabled in Google AI Studio
- Check model name is correct

### "404 Not Found"

- Ensure `api_base` doesn't include trailing paths
- Should be: `https://generativelanguage.googleapis.com`
- Not: `https://generativelanguage.googleapis.com/v1beta/models/`

### Rate Limiting

If you hit rate limits, add delays between evals or use a smaller model for judging.

## Roadmap

- [ ] Support for custom judge prompts
- [ ] Parallel eval execution
- [ ] CSV/JSON output reports
- [ ] Score aggregation and statistics
- [ ] Support for vision models
- [ ] Streaming response support
- [ ] Cost tracking

## Contributing

Contributions welcome! Please open an issue or PR.

## License

MIT License - see LICENSE file for details

## Acknowledgments

Built with:
- [Rust](https://rust-lang.org)
- [Tokio](https://tokio.rs) - Async runtime
- [Reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [Serde](https://serde.rs) - Serialization
- Google Gemini API