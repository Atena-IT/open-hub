# Test Report — xet-backend CAS Server

## Test Matrix

| Suite | Test | Infrastructure | Status |
|-------|------|----------------|--------|
| Unit — shard-parser | `parse_minimal_shard` | none | run with `cargo test` |
| Unit — shard-parser | `parse_rejects_bad_tag` | none | run with `cargo test` |
| Unit — shard-parser | `parse_rejects_bad_version` | none | run with `cargo test` |
| Unit — shard-parser | `parse_empty_shard` | none | run with `cargo test` |
| Unit — shard-parser | `dedup_shard_roundtrip_size` | none | run with `cargo test` |
| Unit — common | `spec_known_vector` | none | run with `cargo test` |
| Unit — common | `roundtrip_identity` | none | run with `cargo test` |
| Unit — common | `all_zeros` | none | run with `cargo test` |
| Unit — common | `all_ones` | none | run with `cargo test` |
| Unit — common | `rejects_wrong_length` | none | run with `cargo test` |
| Integration — infra-free | `test_issue_read_token_returns_valid_jwt` | none | run with `cargo test` |
| Integration — infra-free | `test_issue_write_token_claims_scope` | none | run with `cargo test` |
| Integration — infra-free | `test_write_token_via_post` | none | run with `cargo test` |
| Integration — infra-free | `test_protected_route_without_auth_is_401` | none | run with `cargo test` |
| Integration — infra-free | `test_protected_route_wrong_secret_is_401` | none | run with `cargo test` |
| Integration — infra-free | `test_protected_route_expired_token_is_401` | none | run with `cargo test` |
| Integration — infra-free | `test_dedup_wrong_prefix_is_400` | none | run with `cargo test` |
| Integration — infra-free | `test_upload_xorb_wrong_prefix_is_400` | none | run with `cargo test` |
| Integration — infra-free | `test_upload_xorb_invalid_hash_is_400` | none | run with `cargo test` |
| Integration — infra-free | `test_reconstruction_invalid_file_id_is_400` | none | run with `cargo test` |
| Integration — infra-free | `test_health_returns_json_with_status_key` | none | run with `cargo test` |
| Integration — full-stack | `test_health_with_live_db` | Postgres | `--include-ignored` |
| Integration — full-stack | `test_full_upload_dedup_reconstruct_flow` | Postgres + MinIO | `--include-ignored` |
| E2E | `tests/e2e/run_e2e.sh` | docker-compose | `./tests/e2e/run_e2e.sh` |

---

## How to Run

### 1. Unit and infrastructure-free integration tests

```sh
cargo test
```

Expected: all non-`#[ignore]` tests pass. No external services required.

### 2. Full-stack integration tests

Start the docker-compose stack first, then export env vars:

```sh
docker compose up -d
export INTEGRATION_DATABASE_URL=postgresql://xet:xet@localhost:5432/xetdb
export INTEGRATION_S3_ENDPOINT=http://localhost:9000
export AWS_ACCESS_KEY_ID=minioadmin
export AWS_SECRET_ACCESS_KEY=minioadmin

cargo test -- --include-ignored
```

### 3. End-to-end script

```sh
chmod +x tests/e2e/run_e2e.sh
./tests/e2e/run_e2e.sh
```

Flags:
- `--no-build` — skip `docker compose build` (use existing image)
- `--keep-up`  — leave the stack running after the test completes

The E2E script:
1. Starts the full docker-compose stack
2. Issues a write token via the token-vending API
3. Generates a 10 MiB file of pseudo-random data
4. Uploads the file as a single "xorb"
5. Builds and uploads an MDB shard (using a Python helper embedded in the script)
6. Queries the global dedup endpoint for the chunk
7. Queries the reconstruction endpoint for the file
8. Downloads the xorb via the presigned URL returned by reconstruction
9. Compares SHA-256 of the downloaded bytes to the original file
10. Verifies the xorb object exists in MinIO

---

## Coverage Notes

| Protocol Concern | Covered By |
|-----------------|------------|
| Hash API encoding (known vectors, roundtrip, lengths) | `common` unit tests |
| MDB shard parse — minimal, empty, bad tag, bad version | `shard-parser` unit tests |
| Dedup response shard builder + footer | `shard-parser` unit test `dedup_shard_roundtrip_size` |
| JWT issuance (read/write, claims, expiry) | Integration infra-free |
| JWT validation (missing, wrong secret, expired) | Integration infra-free |
| Input validation (bad prefix, bad hash encoding) | Integration infra-free |
| Health check structure | Integration infra-free |
| DB connectivity check in health endpoint | Integration full-stack |
| Xorb upload → S3 + DB | Integration full-stack + E2E |
| Shard upload → chunk index + file mapping | Integration full-stack + E2E |
| Global dedup (chunk → xorb shard) | Integration full-stack + E2E |
| Reconstruction (file → presigned URLs + terms) | Integration full-stack + E2E |
| Presigned URL download integrity | E2E only |
| MinIO object presence | E2E only |

---

## Known Gaps / TODOs

- **Range requests** (`Range: bytes=...`): `offset_into_first_range` is always 0. Byte-accurate range support needs chunk boundary data from the DB.
- **HMAC-protected chunk hashes**: the dedup response shard uses a zeroed HMAC key. Production deployments should set a real key per the xet-core spec.
- **Multipart S3 upload**: xorbs > 5 GiB require multipart; current implementation uses single-part PUT.
- **Concurrent upload stress test**: not covered — add with a load-testing tool (e.g., k6, hey) against the docker-compose stack.
- **SHA-256 idempotency**: uploading the same file twice should return the same file hash; no explicit test yet.

---

## Results (fill in after running)

| Run | Date | Commit | Unit | Integration | E2E | Notes |
|-----|------|--------|------|-------------|-----|-------|
| 1 | — | — | — | — | — | first run |
