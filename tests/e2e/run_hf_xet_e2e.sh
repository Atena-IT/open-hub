#!/usr/bin/env bash
# =============================================================================
# run_hf_xet_e2e.sh — HuggingFace xet-core round-trip test
#
# Usage:
#   ./tests/e2e/run_hf_xet_e2e.sh [--no-build] [--keep-up] [--use-docker]
#
# Flags:
#   --no-build    Skip `docker compose -f deployment/docker-compose.yml build`
#   --keep-up     Leave the docker-compose stack running after the test
#   --use-docker  Run the Python test inside the docker-compose test-runner
#                 service instead of in a local Python venv
#
# Requirements (local mode):
#   python3, pip, docker compose -f deployment/docker-compose.yml v2, curl
# Requirements (docker mode):
#   docker compose -f deployment/docker-compose.yml v2
# =============================================================================
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}[OK]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*" >&2; exit 1; }
info() { echo -e "${YELLOW}[INFO]${NC} $*"; }

# ── Args ───────────────────────────────────────────────────────────────────────
DO_BUILD=true
KEEP_UP=false
USE_DOCKER=false
for arg in "$@"; do
  case "$arg" in
    --no-build)   DO_BUILD=false   ;;
    --keep-up)    KEEP_UP=true     ;;
    --use-docker) USE_DOCKER=true  ;;
    *) echo "Unknown flag: $arg" >&2; exit 1 ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

# ── Cleanup ────────────────────────────────────────────────────────────────────
on_exit() {
  if [[ "$KEEP_UP" == "false" ]]; then
    info "Tearing down docker-compose stack"
    docker compose -f deployment/docker-compose.yml down -v --remove-orphans 2>/dev/null || true
  else
    info "Stack left running. Stop with: docker compose -f deployment/docker-compose.yml down -v"
  fi
}
trap on_exit EXIT

# ── Start stack ────────────────────────────────────────────────────────────────
# Ensure .env exists (needed by docker-compose env_file directive)
if [[ ! -f "$REPO_ROOT/.env" ]]; then
  info ".env not found — copying from deployment/.env.example"
  cp "$REPO_ROOT/deployment/.env.example" "$REPO_ROOT/.env"
fi

info "Starting docker-compose stack"
if [[ "$DO_BUILD" == "true" ]]; then
  docker compose -f deployment/docker-compose.yml build xet-server
fi
docker compose -f deployment/docker-compose.yml up -d postgres minio minio-init

info "Waiting for Postgres…"
timeout 60 bash -c \
  'until docker compose -f deployment/docker-compose.yml exec -T postgres pg_isready -U xet -d xetdb &>/dev/null; do sleep 2; done'
ok "Postgres ready"

info "Waiting for MinIO…"
timeout 60 bash -c \
  'until curl -sf http://localhost:9000/minio/health/live &>/dev/null; do sleep 2; done'
ok "MinIO ready"

docker compose -f deployment/docker-compose.yml up -d xet-server
info "Waiting for CAS server /health…"
timeout 60 bash -c \
  'until curl -sf http://localhost:8080/health 2>/dev/null | grep -q '"'"'"ok"'"'"'; do sleep 2; done'
ok "CAS server healthy"

# ── Run test ───────────────────────────────────────────────────────────────────
if [[ "$USE_DOCKER" == "true" ]]; then
  info "Running hf_xet_roundtrip.py inside docker test-runner…"
  docker compose -f deployment/docker-compose.yml run --rm \
    -e CAS_URL=http://xet-server:3000 \
    test-runner \
    python3 /tests/e2e/hf_xet_roundtrip.py
else
  info "Running hf_xet_roundtrip.py in local Python venv…"

  VENV="$REPO_ROOT/.venv-e2e"
  if [[ ! -d "$VENV" ]]; then
    python3 -m venv "$VENV"
  fi
  # shellcheck source=/dev/null
  source "$VENV/bin/activate"

  # Install dependencies (hf_xet is optional but tried)
  pip install --quiet --upgrade pip
  pip install --quiet requests blake3 lz4

  # Attempt hf_xet install — non-fatal if wheels are unavailable for this platform
  pip install --quiet hf_xet 2>/dev/null \
    || info "hf_xet not available on this platform — Python fallback will be used"

  CAS_URL=http://localhost:8080 \
    python3 tests/e2e/hf_xet_roundtrip.py

  deactivate
fi

ok "hf_xet round-trip test PASSED"
