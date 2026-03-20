# Examples

This folder contains self-contained examples that demonstrate using **open-hub** as a
drop-in replacement for the Hugging Face Hub.  Each example targets a realistic
machine-learning workflow and shows how to point third-party tools at a locally-running
open-hub instance by changing a single environment variable (`HF_ENDPOINT`).

## Available examples

| Folder | What it shows |
|---|---|
| [`dlt_traces_to_training_data/`](dlt_traces_to_training_data/) | Use [dlt](https://dlthub.com) to collect LLM application traces and push them as a Parquet dataset to your open-hub instance — mirroring the pattern from the [dlthub blog post](https://dlthub.com/blog/your-traces-aren-t-training-data-yet-here-s-the-pipeline-that-makes-them). |

## Prerequisites

All examples assume a running open-hub stack.  The easiest way to start one is:

```bash
# From the repository root
cp deployment/.env.example .env
docker compose -f deployment/docker-compose.yml up -d --build --wait
```

The hub will then be available at **http://localhost:8080**.

## Running an example

Each script uses [PEP 723](https://peps.python.org/pep-0723/) inline metadata so that
[`uv`](https://github.com/astral-sh/uv) can resolve and install dependencies
automatically:

```bash
HF_ENDPOINT=http://localhost:8080 uv run examples/<example>/<script>.py
```

See the `README.md` inside each folder for example-specific instructions.
