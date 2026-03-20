# /// script
# requires-python = ">=3.9"
# dependencies = [
#   "dlt[huggingface]>=1.4.0",
#   "huggingface_hub>=0.23.0",
#   "requests",
#   "pyarrow",
# ]
# ///

"""
dlt traces-to-training-data pipeline — open-hub edition
========================================================

Reproduces the pattern from:
  https://dlthub.com/blog/your-traces-aren-t-training-data-yet-here-s-the-pipeline-that-makes-them

Uses **open-hub** as a drop-in replacement for the Hugging Face Hub by setting
the HF_ENDPOINT environment variable before any huggingface_hub call is made.
No dlt or pipeline code needs to change — the substitution is fully transparent.

Usage:
    HF_ENDPOINT=http://localhost:8080 uv run examples/dlt_traces_to_training_data/traces_pipeline.py

Optional environment variables:
    HUB_USERNAME   - username to register / login (default: traces_user)
    HUB_PASSWORD   - password for that user      (default: tracespassword123)
    TRACE_COUNT    - number of synthetic traces   (default: 200)
"""

from __future__ import annotations

import os
import random
import time
import uuid
from typing import Iterator

import requests

# ---------------------------------------------------------------------------
# Point huggingface_hub at open-hub BEFORE importing it or dlt (which also
# imports it internally).  This single line is the entire "drop-in" change.
# ---------------------------------------------------------------------------
HF_ENDPOINT = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["HF_ENDPOINT"] = HF_ENDPOINT

import dlt  # noqa: E402  (import after env var is set)
from huggingface_hub import HfApi  # noqa: E402

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
HUB_USERNAME = os.environ.get("HUB_USERNAME", "traces_user")
HUB_PASSWORD = os.environ.get("HUB_PASSWORD", "tracespassword123")
TRACE_COUNT = int(os.environ.get("TRACE_COUNT", "200"))

DATASET_NAME = f"{HUB_USERNAME}/llm-traces-training-data"

# ---------------------------------------------------------------------------
# Hub helpers
# ---------------------------------------------------------------------------

def _register_or_login(endpoint: str, username: str, password: str) -> str:
    """Return a Bearer token, registering the user if they don't exist yet."""
    resp = requests.post(
        f"{endpoint}/api/auth/register",
        json={"username": username, "password": password},
        timeout=10,
    )
    if resp.status_code == 409:
        resp = requests.post(
            f"{endpoint}/api/auth/login",
            json={"username": username, "password": password},
            timeout=10,
        )
    resp.raise_for_status()
    return resp.json()["token"]


def _ensure_dataset_repo(api: HfApi, repo_id: str) -> None:
    """Create the dataset repository if it does not already exist."""
    try:
        api.create_repo(repo_id=repo_id, repo_type="dataset", private=False)
        print(f"[hub] Created dataset repo: {repo_id}")
    except Exception as exc:
        if "already exists" in str(exc) or "409" in str(exc):
            print(f"[hub] Dataset repo already exists: {repo_id}")
        else:
            raise


# ---------------------------------------------------------------------------
# Synthetic trace generation
# ---------------------------------------------------------------------------

_SYSTEM_PROMPTS = [
    "You are a helpful assistant.",
    "You are a concise coding assistant. Reply with code only.",
    "You are a customer-support agent. Be empathetic and solution-focused.",
    "You are a creative writing partner.",
]

_USER_PROMPTS = [
    "Explain the difference between supervised and unsupervised learning.",
    "Write a Python function that reverses a linked list.",
    "My order hasn't arrived yet. What can I do?",
    "Continue this story: The spaceship landed in a cornfield at midnight…",
    "Summarise the key points of the transformer architecture.",
    "How do I centre a div in CSS?",
    "What are the pros and cons of microservices?",
    "Give me a haiku about unit tests.",
]

_RESPONSES = [
    "Supervised learning uses labelled data; unsupervised learning finds patterns without labels.",
    "```python\ndef reverse(head):\n    prev = None\n    while head:\n        head.next, prev, head = prev, head, head.next\n    return prev\n```",
    "I'm sorry to hear that! Please share your order number and I'll look into it right away.",
    "The alien crew stepped out, adjusting their disguises — corn-silk wigs and flannel shirts.",
    "Transformers use self-attention to weigh the importance of each token relative to every other token in the sequence.",
    "Use `display: flex; justify-content: center; align-items: center;` on the parent element.",
    "Microservices improve scalability and independent deployability but add operational complexity.",
    "Green bar glows bright /\nA hundred assertions pass /\nCommit with great pride",
]

_MODELS = ["gpt-4o-mini", "claude-3-haiku", "mistral-7b", "llama-3-8b"]


def generate_traces(n: int) -> list[dict]:
    """Return *n* synthetic LLM trace records."""
    traces: list[dict] = []
    for _ in range(n):
        prompt_idx = random.randrange(len(_USER_PROMPTS))
        traces.append(
            {
                "trace_id": str(uuid.uuid4()),
                "timestamp": time.strftime(
                    "%Y-%m-%dT%H:%M:%SZ",
                    time.gmtime(time.time() - random.randint(0, 7 * 24 * 3600)),
                ),
                "model": random.choice(_MODELS),
                "system_prompt": random.choice(_SYSTEM_PROMPTS),
                "user_message": _USER_PROMPTS[prompt_idx],
                "assistant_message": _RESPONSES[prompt_idx],
                "latency_ms": random.randint(120, 3500),
                "prompt_tokens": random.randint(20, 200),
                "completion_tokens": random.randint(10, 300),
                # Synthetic quality signal: rating from 1–5 given by a hypothetical
                # evaluator or user-feedback widget.
                "rating": random.choices([1, 2, 3, 4, 5], weights=[2, 5, 15, 45, 33])[0],
            }
        )
    return traces


# ---------------------------------------------------------------------------
# dlt source
# ---------------------------------------------------------------------------

@dlt.source
def llm_traces_source(traces: list[dict]):  # type: ignore[return]  # dlt sources yield resources, not a plain return value
    """dlt source that yields processed trace records as a single resource."""

    @dlt.resource(name="training_pairs", write_disposition="replace")
    def _training_pairs() -> Iterator[dict]:
        for trace in traces:
            # Only keep traces with a rating of 4 or 5 — those are the ones we
            # want to use for supervised fine-tuning.
            if trace["rating"] < 4:
                continue
            yield {
                "trace_id": trace["trace_id"],
                "timestamp": trace["timestamp"],
                "model": trace["model"],
                # Reformat as a simple instruction-following pair that most
                # fine-tuning frameworks (TRL, Axolotl, …) can consume directly.
                "instruction": trace["user_message"],
                "response": trace["assistant_message"],
                "system": trace["system_prompt"],
                "latency_ms": trace["latency_ms"],
                "token_count": trace["prompt_tokens"] + trace["completion_tokens"],
                "score": trace["rating"] / 5.0,
            }

    @dlt.resource(name="raw_traces", write_disposition="replace")
    def _raw_traces() -> Iterator[dict]:
        yield from traces

    yield _training_pairs
    yield _raw_traces


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    print("=== dlt Traces-to-Training-Data Pipeline (open-hub edition) ===")
    print(f"Hub endpoint : {HF_ENDPOINT}")
    print(f"Dataset repo : {DATASET_NAME}")
    print(f"Trace count  : {TRACE_COUNT}")
    print()

    # 1. Authenticate with open-hub and obtain a token.
    print("[auth] Registering / logging in …")
    token = _register_or_login(HF_ENDPOINT, HUB_USERNAME, HUB_PASSWORD)
    os.environ["HF_TOKEN"] = token
    print(f"[auth] OK — logged in as '{HUB_USERNAME}'")

    # 2. Ensure the target dataset repository exists.
    api = HfApi(endpoint=HF_ENDPOINT, token=token)
    _ensure_dataset_repo(api, DATASET_NAME)

    # 3. Generate synthetic traces.
    print(f"\n[data] Generating {TRACE_COUNT} synthetic LLM traces …")
    traces = generate_traces(TRACE_COUNT)
    high_quality = [t for t in traces if t["rating"] >= 4]
    print(
        f"[data] {len(traces)} total traces, "
        f"{len(high_quality)} high-quality (rating ≥ 4) kept for training pairs"
    )

    # 4. Build and run the dlt pipeline.
    #
    # The `huggingface` destination reads HF_ENDPOINT and HF_TOKEN from the
    # environment, so it automatically targets open-hub without any extra
    # configuration.  The `dataset_name` parameter maps to the repository that
    # was created in step 2.
    print("\n[dlt] Building pipeline …")
    pipeline = dlt.pipeline(
        pipeline_name="llm_traces",
        destination=dlt.destinations.huggingface(
            dataset_name=DATASET_NAME,
        ),
        dataset_name="llm_traces",
    )

    print("[dlt] Running pipeline …")
    load_info = pipeline.run(llm_traces_source(traces))

    # 5. Report results.
    print("\n[dlt] Load completed.")
    print(load_info)

    repo_url = f"{HF_ENDPOINT}/datasets/{DATASET_NAME}"
    print(f"\n✅  Dataset available at: {repo_url}")
    print(
        "    Browse it in the open-hub Web UI, or download it with the `datasets` library:\n\n"
        "        pip install datasets\n"
        f"        HF_ENDPOINT={HF_ENDPOINT} python -c \"\n"
        "        import os\n"
        "        os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')\n"
        "        from datasets import load_dataset\n"
        f"        ds = load_dataset('{DATASET_NAME}', split='training_pairs')\n"
        "        print(ds)\n"
        '        "'
    )


if __name__ == "__main__":
    main()
