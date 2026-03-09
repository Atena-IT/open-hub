#!/usr/bin/env bash
# =============================================================================
# run_e2e.sh — End-to-End verification for the Xet CAS backend
#
# Usage:
#   ./tests/e2e/run_e2e.sh [--no-build] [--keep-up]
#
# Flags:
#   --no-build   Skip `docker compose build` (use cached image)
#   --keep-up    Leave the docker-compose stack running after the test
#
# Requirements:
#   - docker + docker compose v2
#   - python3, curl, jq
#   - sha256sum (Linux) or shasum (macOS)
#
# What this script verifies:
#   1. Stack starts and /health returns {"status":"ok"}
#   2. Token vending issues a valid JWT
#   3. POST /v1/xorbs/default/{hash}  — upload succeeds
#   4. POST /v1/shards                — shard upload indexes file
#   5. GET  /v1/chunks/default-merkledb/{hash} — dedup returns a shard binary
#   6. GET  /v1/reconstructions/{file_id}      — returns terms + presigned URLs
#   7. Presigned URL delivers the same bytes that were uploaded (SHA-256 check)
#   8. MinIO contains the xorb object under the expected key
# =============================================================================

set -euo pipefail

# ── Colour output ─────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
ok()   { echo -e "${GREEN}[PASS]${NC} $*"; }
fail() { echo -e "${RED}[FAIL]${NC} $*" >&2; exit 1; }
info() { echo -e "${YELLOW}[INFO]${NC} $*"; }

# ── Argument parsing ──────────────────────────────────────────────────────────
DO_BUILD=true
KEEP_UP=false
for arg in "$@"; do
  case "$arg" in
    --no-build) DO_BUILD=false ;;
    --keep-up)  KEEP_UP=true  ;;
    *) echo "Unknown flag: $arg" >&2; exit 1 ;;
  esac
done

# ── Repository root ───────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# ── sha256 portability ────────────────────────────────────────────────────────
sha256_of() { # sha256_of <file>
  if command -v sha256sum &>/dev/null; then sha256sum "$1" | awk '{print $1}'
  else shasum -a 256 "$1" | awk '{print $1}'; fi
}

# ── Cleanup on exit ───────────────────────────────────────────────────────────
on_exit() {
  if [[ "$KEEP_UP" == "false" ]]; then
    info "Tearing down docker-compose stack"
    docker compose down -v --remove-orphans 2>/dev/null || true
  else
    info "Stack left running (--keep-up). Stop with: docker compose down -v"
  fi
}
trap on_exit EXIT

CAS_URL="http://localhost:8080"

# =============================================================================
# Step 1 — Start the stack
# =============================================================================
info "Step 1: Starting docker-compose stack"
if [[ "$DO_BUILD" == "true" ]]; then
  docker compose build xet-server
fi
docker compose up -d postgres minio minio-init

info "  Waiting for Postgres…"
timeout 60 bash -c \
  'until docker compose exec -T postgres pg_isready -U xet -d xetdb &>/dev/null; do sleep 2; done'
ok "Postgres ready"

info "  Waiting for MinIO…"
timeout 60 bash -c \
  'until curl -sf http://localhost:9000/minio/health/live &>/dev/null; do sleep 2; done'
ok "MinIO ready"

docker compose up -d xet-server

info "  Waiting for CAS server /health…"
timeout 60 bash -c \
  "until curl -sf \"$CAS_URL/health\" 2>/dev/null | grep -q '\"ok\"'; do sleep 2; done"
ok "CAS server healthy"

# =============================================================================
# Step 2 — Obtain a write token
# =============================================================================
info "Step 2: Fetching write token"

info "Registering a user to get Hub API token"
HUB_TOKEN_JSON=$(curl -sf -X POST -H "Content-Type: application/json" -d "{\"username\": \"e2e_user_$RANDOM\", \"password\": \"password\"}" "$CAS_URL/api/auth/register")
HUB_TOKEN=$(echo "$HUB_TOKEN_JSON" | jq -r '.token')
[[ -n "$HUB_TOKEN" && "$HUB_TOKEN" != "null" ]] \
  || fail "Could not register user: $HUB_TOKEN_JSON"

info "Fetching CAS write token with Hub API token"
TOKEN_JSON=$(curl -sf -H "Authorization: Bearer $HUB_TOKEN" "$CAS_URL/api/models/e2e-ns/e2e-repo/xet-write-token/main")
ACCESS_TOKEN=$(echo "$TOKEN_JSON" | jq -r '.accessToken')
[[ -n "$ACCESS_TOKEN" && "$ACCESS_TOKEN" != "null" ]] \
  || fail "Could not obtain write token: $TOKEN_JSON"
ok "Write token obtained (exp=$(echo "$TOKEN_JSON" | jq '.exp'))"

AUTH_HDR="Authorization: Bearer $ACCESS_TOKEN"

# =============================================================================
# Step 3 — Generate a 10 MiB test file
# =============================================================================
info "Step 3: Generating 10 MiB test file"
TEST_FILE="$TMP/source.bin"
dd if=/dev/urandom of="$TEST_FILE" bs=1M count=10 2>/dev/null
ORIGINAL_SHA=$(sha256_of "$TEST_FILE")
FILE_SIZE=$(wc -c < "$TEST_FILE")
info "  size=$FILE_SIZE  sha256=$ORIGINAL_SHA"

# =============================================================================
# Step 4 — Compute xorb / chunk / file hashes with Python
#
# NOTE: xet-core uses CDC + Blake3 in production. This script uses SHA-256 as
#       a stand-in to keep the E2E test self-contained without needing xet-core.
# =============================================================================
info "Step 4: Computing content hashes"

python3 > "$TMP/hashes.env" <<PYEOF
import sys, hashlib, struct, pathlib

data   = pathlib.Path("$TEST_FILE").read_bytes()
size   = len(data)

xorb_raw  = hashlib.sha256(data).digest()
chunk_raw = hashlib.sha256(b"chunk:" + data[:65536]).digest()
file_raw  = hashlib.sha256(b"file:"  + data).digest()
sha256_raw = xorb_raw  # reuse xorb hash as the "sha256" for FileMetadataExt

def api_encode(b32):
    """32-byte hash → 64-char API hex string (4 × reversed u64)."""
    out = ""
    for i in range(4):
        val = struct.unpack_from("<Q", b32, i * 8)[0]
        out += f"{val:016x}"
    return out

print(f"XORB_API={api_encode(xorb_raw)}")
print(f"CHUNK_API={api_encode(chunk_raw)}")
print(f"FILE_API={api_encode(file_raw)}")
print(f"XORB_HEX={xorb_raw.hex()}")
print(f"CHUNK_HEX={chunk_raw.hex()}")
print(f"FILE_HEX={file_raw.hex()}")
print(f"SHA256_HEX={sha256_raw.hex()}")
print(f"FILE_SIZE={size}")
PYEOF

# shellcheck source=/dev/null
source "$TMP/hashes.env"
info "  xorb_api:  $XORB_API"
info "  chunk_api: $CHUNK_API"
info "  file_api:  $FILE_API"

# =============================================================================
# Step 5 — Upload the xorb
# =============================================================================
info "Step 5: Uploading xorb (${FILE_SIZE} bytes)"
XORB_RESP=$(curl -sf -X POST \
  -H "$AUTH_HDR" \
  -H "Content-Type: application/octet-stream" \
  --data-binary "@$TEST_FILE" \
  "$CAS_URL/v1/xorbs/default/$XORB_API")
echo "  response: $XORB_RESP"
echo "$XORB_RESP" | jq -e '.was_inserted != null' > /dev/null \
  || fail "xorb upload response missing was_inserted: $XORB_RESP"
ok "Xorb uploaded: was_inserted=$(echo "$XORB_RESP" | jq '.was_inserted')"

# =============================================================================
# Step 6 — Build and upload a minimal MDB shard
# =============================================================================
info "Step 6: Building and uploading shard"

python3 > "$TMP/shard.bin" <<PYEOF
import struct, pathlib, sys

xorb_raw  = bytes.fromhex("$XORB_HEX")
chunk_raw = bytes.fromhex("$CHUNK_HEX")
file_raw  = bytes.fromhex("$FILE_HEX")
sha256    = bytes.fromhex("$SHA256_HEX")
sz        = $FILE_SIZE

TAG = (
    b'HFRepoMetaData0'
    + bytes([85,105,103,69,106,123,129,87,131,165,189,217,92,205,209,74,169])
)
assert len(TAG) == 32
BOOKEND            = bytes([0xFF]*32) + bytes(16)
HEADER_VERSION     = 2
FLAG_VERIFICATION  = 0x8000_0000
FLAG_METADATA_EXT  = 0x4000_0000

buf = bytearray()

# Header (48 bytes) — footer_size=0 means no footer (upload form)
buf += TAG
buf += struct.pack("<Q", HEADER_VERSION)
buf += struct.pack("<Q", 0)

# FileDataSequenceHeader
buf += file_raw
buf += struct.pack("<I", FLAG_VERIFICATION | FLAG_METADATA_EXT)
buf += struct.pack("<I", 1)   # num_entries
buf += bytes(8)               # _unused

# FileDataSequenceEntry
buf += xorb_raw
buf += struct.pack("<I", 0)   # cas_flags
buf += struct.pack("<I", sz)  # unpacked_segment_bytes
buf += struct.pack("<I", 0)   # chunk_index_start
buf += struct.pack("<I", 1)   # chunk_index_end

# FileVerificationEntry (1)
buf += bytes([0xBB]*32) + bytes(16)

# FileMetadataExt
buf += sha256 + bytes(16)

# File Info bookend
buf += BOOKEND

# CASChunkSequenceHeader
buf += xorb_raw
buf += struct.pack("<I", 0)        # cas_flags
buf += struct.pack("<I", 1)        # num_entries
buf += struct.pack("<I", sz)       # num_bytes_in_cas
buf += struct.pack("<I", sz + 64)  # num_bytes_on_disk

# CASChunkSequenceEntry
buf += chunk_raw
buf += struct.pack("<I", 0)    # chunk_byte_range_start
buf += struct.pack("<I", sz)   # unpacked_segment_bytes
buf += bytes(8)                # _unused

# CAS Info bookend
buf += BOOKEND

sys.stdout.buffer.write(bytes(buf))
PYEOF

SHARD_RESP=$(curl -sf -X POST \
  -H "$AUTH_HDR" \
  -H "Content-Type: application/octet-stream" \
  --data-binary "@$TMP/shard.bin" \
  "$CAS_URL/v1/shards")
echo "  response: $SHARD_RESP"
echo "$SHARD_RESP" | jq -e '.result == 1' > /dev/null \
  || fail "shard upload failed: $SHARD_RESP"
ok "Shard uploaded"

# =============================================================================
# Step 7 — Global dedup query
# =============================================================================
info "Step 7: Global dedup query for chunk"
DEDUP_SIZE=$(curl -sf \
  -H "$AUTH_HDR" \
  "$CAS_URL/v1/chunks/default-merkledb/$CHUNK_API" \
  -o "$TMP/dedup.bin" -w "%{size_download}")
[[ "$DEDUP_SIZE" -gt 100 ]] \
  || fail "Dedup response too small (${DEDUP_SIZE}B) — chunk not found?"
ok "Dedup response: ${DEDUP_SIZE} bytes"

# =============================================================================
# Step 8 — Reconstruction query
# =============================================================================
info "Step 8: Reconstruction query"
RECON=$(curl -sf -H "$AUTH_HDR" "$CAS_URL/v1/reconstructions/$FILE_API")
NUM_TERMS=$(echo "$RECON" | jq '.terms | length')
[[ "$NUM_TERMS" -gt 0 ]] \
  || fail "Reconstruction returned 0 terms: $RECON"
echo "$RECON" | jq '{num_terms: (.terms|length), first_hash: .terms[0].hash, has_fetch_info: (.fetch_info!=null)}'
ok "Reconstruction: $NUM_TERMS term(s)"

# Extract presigned URL — try direct key first, then first entry
PRESIGNED=$(echo "$RECON" | jq -r ".fetch_info[\"$XORB_API\"][0].url // empty" 2>/dev/null || true)
if [[ -z "$PRESIGNED" ]]; then
  PRESIGNED=$(echo "$RECON" | jq -r '.fetch_info | to_entries[0].value[0].url')
fi
[[ -n "$PRESIGNED" && "$PRESIGNED" != "null" ]] \
  || fail "No presigned URL in reconstruction response"
info "  presigned URL: ${PRESIGNED:0:80}…"

# =============================================================================
# Step 9 — Download via presigned URL and verify SHA-256
# =============================================================================
info "Step 9: Downloading xorb via presigned URL"
curl -sf "$PRESIGNED" -o "$TMP/downloaded.bin" \
  || fail "Failed to download from presigned URL"

DOWNLOADED_SHA=$(sha256_of "$TMP/downloaded.bin")
info "  original:   $ORIGINAL_SHA"
info "  downloaded: $DOWNLOADED_SHA"
[[ "$ORIGINAL_SHA" == "$DOWNLOADED_SHA" ]] \
  || fail "SHA-256 mismatch — content corrupted!"
ok "SHA-256 verified: downloaded content matches original"

# =============================================================================
# Step 10 — Verify xorb object exists in MinIO
# =============================================================================
info "Step 10: Verifying xorb in MinIO"
XORB_S3_KEY="xorbs/${XORB_HEX:0:2}/${XORB_HEX:2:2}/$XORB_HEX"
MC_STATUS=$(docker compose exec -T minio /bin/sh -c \
  "mc alias set local http://localhost:9000 minioadmin minioadmin >/dev/null 2>&1 \
   && mc stat --json local/xet-storage/$XORB_S3_KEY 2>&1" || true)
echo "$MC_STATUS" | head -5
echo "$MC_STATUS" | grep -q '"status":"success"' \
  || fail "Xorb not found in MinIO at key: $XORB_S3_KEY"
ok "Xorb confirmed in MinIO"

# =============================================================================
# Summary
# =============================================================================
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║      E2E TEST SUITE: ALL PASSED          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""
printf '  %-26s %s\n' "Stack health"          "✓"
printf '  %-26s %s\n' "Token vending"         "✓"
printf '  %-26s %s\n' "Xorb upload"           "✓"
printf '  %-26s %s\n' "Shard upload"          "✓"
printf '  %-26s %s\n' "Global dedup query"    "✓"
printf '  %-26s %s\n' "Reconstruction query"  "✓"
printf '  %-26s %s\n' "Presigned download"    "✓"
printf '  %-26s %s\n' "SHA-256 integrity"     "✓"
printf '  %-26s %s\n' "MinIO object present"  "✓"
echo ""
