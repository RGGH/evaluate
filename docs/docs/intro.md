---
sidebar_position: 1
slug: /
---

# evaluate

Supply ground truth for comparison versus LLM output and let a 2nd LLM be the "judge"

- Pass or Fail
- Eval history is stored in Sqlite database


## Installation

Get started by **setting up your development environment**.

Or **try Evaluate immediately** by cloning from **[GitHub](https://github.com/RGGH/evaluate)**.

### What you'll need

- [Rust](https://www.rust-lang.org/tools/install) version 1.70 or above
- [Node.js](https://nodejs.org/en/download/) version 20.0 or above (for the documentation site)
- An LLM Provider, for example:
  - An OpenAI or Claude subscription API KEY
  - A Gemini API key from [Google AI Studio](https://makersuite.google.com/app/apikey)
  - Or a running [Ollama](https://ollama.com/) instance for local models
- Git for version control
- Sqlite ~ (installed by default)

## Clone and Setup

Clone the Evaluate repository and set up your environment:

```sh
# 1. Clone the repository
git clone git@github.com:RGGH/evaluate.git

# 2. Navigate into the project directory
cd evaluate
```

Create a `.env` file in the root directory with your configuration:

```bash
DATABASE_URL=sqlite:data/evals.db
api_base = "https://generativelanguage.googleapis.com"
api_key = "AIzaSyAkQnssdafsdfasdfasxxxxxxxxxxxxxxxxxxx"
```

The project will automatically install all necessary dependencies when you build it.

## Start your application

Run the development server:

```bash
cargo run
```

The `cargo run` command builds your Rust application and starts the evaluation server locally at http://127.0.0.1:8080/.

You should see output similar to:

```
                   _                          
                  | |               _         
 _____ _   _ _____| | _   _ _____ _| |_ _____ 
| ___ | | | (____ | || | | (____ (_   _) ___ |
| ____|\ V // ___ | || |_| / ___ | | |_| ____|
|_____) \_/ \_____|\_)____/\_____|  \__)_____)
                                                                                                                                                                                                       

    LLM Evaluation & Testing Framework

✅ DATABASE_URL set to: sqlite:data/evals.db
✅ Created database directory: data
📦 Database file path: /home/pop/rust/evaluate/data/evals.db
📦 Connecting to: sqlite:///home/pop/rust/evaluate/data/evals.db?mode=rwc
✅ Database connected successfully
✅ Database migrations completed
🚀 Starting server...
📊 Frontend available at http://127.0.0.1:8080
[2025-10-14T15:40:17Z INFO  actix_server::builder] starting 22 workers
[2025-10-14T15:40:17Z INFO  actix_server::server] Actix runtime found; starting in Actix runtime
[2025-10-14T15:40:17Z INFO  actix_server::server] starting service: "actix-web-service-0.0.0.0:8080", workers: 22, listening on: 0.0.0.0:8080

```

Open your browser and navigate to `http://127.0.0.1:8080` to access the **built-in GUI**. You can now start running evaluations and the database **automatically saves your history**.

## Next Steps

Now that your server is running, you can:

- Test the API with sample cURL commands
- Use the web interface to run single evaluations
- Submit batch evaluations using JSON files
- View your evaluation history in the GUI

Explore the documentation to learn more about configuring models, writing eval definitions, and using the AI-powered judging capabilities.

## Clear the database
If you wish to start again or clear the results, you can delete the database and it will create a new one next time you run evaluate.

`rm data/evals.db`
