# dlt → open-hub: Traces to Training Data

This example reproduces the pattern described in the
[dlthub blog post "Your Traces Aren't Training Data Yet — Here's the Pipeline That Makes Them"](https://dlthub.com/blog/your-traces-aren-t-training-data-yet-here-s-the-pipeline-that-makes-them)
but replaces the Hugging Face Hub with a locally-running **open-hub** instance.

Because open-hub implements the same Hub API that `huggingface_hub` targets, the only
change required is setting `HF_ENDPOINT` — no dlt or pipeline code needs to be
modified.

---

## What the pipeline does

1. **Generates synthetic LLM traces** — prompt / response pairs that simulate what an
   LLM-powered application would log at runtime.
2. **Cleans and structures the traces** — extracts fields useful for fine-tuning:
   `instruction`, `response`, `model`, `latency_ms`, `token_count`, and a per-trace
   quality `score`.
3. **Loads the data with dlt** — the `huggingface` destination writes Parquet files and
   commits them to the dataset repository on open-hub, exactly as it would on the real
   Hugging Face Hub.

---

## Architecture

```
  LLM application          dlt pipeline            open-hub
  ─────────────────        ─────────────────────   ─────────────────────────
  traces (JSON logs)  ──►  extract / transform ──►  Parquet dataset repo
                           load (HF destination)    (huggingface_hub API)
```

The dlt `huggingface` destination uses `huggingface_hub` internally.  Setting the
`HF_ENDPOINT` environment variable re-points every `HfApi` call to your open-hub
server, making it a transparent drop-in replacement.

---

## Prerequisites

### 1. Start the open-hub stack

```bash
# From the repository root
cp deployment/.env.example .env
docker compose -f deployment/docker-compose.yml up -d --build --wait
```

The hub will be available at **http://localhost:8080**.

### 2. Install `uv` (recommended)

```bash
curl -Ls https://astral.sh/uv/install.sh | sh
```

`uv` reads the PEP 723 inline metadata at the top of the script and installs all Python
dependencies into an isolated environment automatically.  No manual `pip install` is
required.

---

## Running the example

```bash
# From the repository root
HF_ENDPOINT=http://localhost:8080 uv run examples/dlt_traces_to_training_data/traces_pipeline.py
```

The script will:

1. Register (or log in to) a test user on open-hub.
2. Create a dataset repository named `<username>/llm-traces-training-data`.
3. Run the dlt pipeline, which uploads a Parquet file containing the processed traces.
4. Print a summary with the repository URL and row count.

You can then inspect the uploaded dataset through the open-hub Web UI at
[http://localhost:8080](http://localhost:8080).

### Optional environment variables

| Variable | Default | Description |
|---|---|---|
| `HF_ENDPOINT` | `http://localhost:8080` | URL of your open-hub (or real HF Hub) instance |
| `HUB_USERNAME` | `traces_user` | Username to register / log in with |
| `HUB_PASSWORD` | `tracespassword123` | Password for that user |
| `TRACE_COUNT` | `200` | Number of synthetic traces to generate |

---

## How the drop-in replacement works

dlt's `huggingface` destination delegates all Hub operations to `huggingface_hub`:

```python
# Inside dlt's HuggingFace destination (simplified):
from huggingface_hub import HfApi
api = HfApi()  # reads HF_ENDPOINT from the environment
api.upload_file(...)
```

Because `HfApi` honours the `HF_ENDPOINT` environment variable, pointing it at
open-hub requires no code changes inside dlt:

```bash
export HF_ENDPOINT=http://localhost:8080
# Run any dlt pipeline with the huggingface destination — it talks to open-hub now.
```

---

## Extending the example

- **Real traces**: Replace `generate_traces()` with a reader that tails your production
  log files or queries a tracing backend (Langfuse, Langsmith, Weave, …).
- **Incremental loads**: Add `dlt.sources.incremental("timestamp")` to the source
  function to only process new traces on each pipeline run.
- **Quality filtering**: Add a `min_score` threshold inside `process_traces()` to drop
  low-quality pairs before they reach the dataset.
- **Multiple tables**: Return multiple `dlt.resource` objects to store raw traces,
  cleaned pairs, and evaluation scores in separate Parquet files within the same
  dataset repo.
