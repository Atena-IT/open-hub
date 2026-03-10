#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== Starting Xet Hub stack ==="
docker compose up -d --build --wait

echo "=== Running roundtrip test ==="
HF_ENDPOINT=http://localhost:8080 uv run integration/roundtrip.py
rc=$?

echo "=== Tearing down ==="
docker compose down

exit $rc
