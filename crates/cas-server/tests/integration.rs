//! Integration tests for the CAS server.
//!
//! Tests are split into two tiers:
//!
//! **Infrastructure-free** — always run, no real DB or S3 required:
//!   - Token vending (read + write)
//!   - JWT middleware: missing/wrong/expired tokens → 401
//!   - Input validation: bad prefix, bad hash encoding → 400
//!
//! **Full-stack** — skipped unless `INTEGRATION_DATABASE_URL` is set:
//!   - `GET /health` against a live Postgres
//!   - End-to-end: upload xorb → upload shard → dedup query → reconstruction
//!
//! To run the full-stack tests:
//!
//! ```sh
//! docker compose up -d
//! INTEGRATION_DATABASE_URL=postgresql://xet:xet@localhost:5432/xetdb \
//! INTEGRATION_S3_ENDPOINT=http://localhost:9000              \
//! AWS_ACCESS_KEY_ID=minioadmin                               \
//! AWS_SECRET_ACCESS_KEY=minioadmin                           \
//! cargo test -- --include-ignored
//! ```

use axum::http::{header, Request, StatusCode};
use bytes::Bytes;
use http_body_util::BodyExt;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header as JwtHeader, Validation};
use serde_json::Value;
use tower::ServiceExt; // oneshot

use cas_server::{middleware::Claims, router::build_router, state::AppState};
use common::AppConfig;
use s3_storage::S3Client;

// ── Test constants ────────────────────────────────────────────────────────────

const TEST_SECRET: &str = "test-jwt-secret-for-integration-only";

// ── Infrastructure helpers ────────────────────────────────────────────────────

/// Build a config for tests that need no external services.
fn test_config() -> AppConfig {
    AppConfig {
        bind_addr:       "127.0.0.1:0".into(),
        database_url:    "postgresql://test:test@127.0.0.1:5555/noconnect".into(),
        s3_endpoint:     Some("http://127.0.0.1:19000".into()),
        s3_bucket:       "test-bucket".into(),
        s3_region:       "us-east-1".into(),
        jwt_secret:      TEST_SECRET.into(),
        jwt_expiry_secs: 3600,
        cas_base_url:    "http://localhost:3000".into(),
    }
}

/// Create an S3 client that points at a non-existent endpoint.
/// AWS SDK construction is pure configuration — no network calls happen here.
async fn fake_s3() -> S3Client {
    S3Client::new(
        Some("http://127.0.0.1:19000"),
        "us-east-1",
        "test-bucket",
        "test-key",
        "test-secret",
    )
    .await
    .expect("fake S3 client construction failed")
}

/// Create a lazy Postgres pool (no connection until first query).
fn lazy_pool(url: &str) -> db_layer::PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(200))
        .connect_lazy(url)
        .expect("connect_lazy: URL is syntactically invalid")
}

/// Build an AppState that needs no live infrastructure.
async fn infra_free_state() -> AppState {
    let pool = lazy_pool("postgresql://test:test@127.0.0.1:5555/noconnect");
    let s3   = fake_s3().await;
    AppState::new(pool, s3, test_config())
}

// ── JWT helpers ───────────────────────────────────────────────────────────────

fn make_jwt(secret: &str, scope: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let claims = Claims {
        sub:      "models/testns/testrepo".into(),
        scope:    scope.to_owned(),
        revision: "main".into(),
        exp:      (now + 3600) as usize,
    };
    encode(
        &JwtHeader::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("test JWT encode failed")
}

fn make_expired_jwt(secret: &str) -> String {
    let claims = Claims {
        sub:      "models/testns/testrepo".into(),
        scope:    "read".into(),
        revision: "main".into(),
        exp:      1_000_000, // far in the past
    };
    encode(
        &JwtHeader::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("test expired JWT encode failed")
}

async fn json_body(body: axum::body::Body) -> Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

// ── Infrastructure-free tests ─────────────────────────────────────────────────

#[tokio::test]
async fn test_issue_read_token_returns_valid_jwt() {
    let state  = infra_free_state().await;
    let router = build_router(state);

    let req = Request::builder()
        .method("GET")
        .uri("/api/models/testns/myrepo/xet-read-token/main")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = json_body(resp.into_body()).await;
    assert!(body["accessToken"].is_string(), "accessToken missing: {body}");
    assert!(body["exp"].is_number(),         "exp missing: {body}");
    assert_eq!(body["casUrl"], "http://localhost:3000");

    // Decode and verify claims
    let token_str = body["accessToken"].as_str().unwrap();
    let decoded = jsonwebtoken::decode::<Claims>(
        token_str,
        &DecodingKey::from_secret(TEST_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .expect("returned token must be decodable with the server secret");

    assert_eq!(decoded.claims.scope, "read");
    assert_eq!(decoded.claims.sub,   "models/testns/myrepo");
    assert_eq!(decoded.claims.revision, "main");
}

#[tokio::test]
async fn test_issue_write_token_claims_scope() {
    let state  = infra_free_state().await;
    let router = build_router(state);

    let req = Request::builder()
        .method("GET")
        .uri("/api/datasets/nspc/ds/xet-write-token/v2")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp   = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body   = json_body(resp.into_body()).await;
    let token  = body["accessToken"].as_str().unwrap();

    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(TEST_SECRET.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .unwrap();

    assert_eq!(decoded.claims.scope,    "write");
    assert_eq!(decoded.claims.sub,      "datasets/nspc/ds");
    assert_eq!(decoded.claims.revision, "v2");
}

#[tokio::test]
async fn test_write_token_via_post() {
    let state  = infra_free_state().await;
    let router = build_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/models/nspc/repo/xet-write-token/main")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_protected_route_without_auth_is_401() {
    let state  = infra_free_state().await;
    let router = build_router(state);

    let req = Request::builder()
        .method("POST")
        .uri(format!("/v1/xorbs/default/{}", "0".repeat(64)))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_wrong_secret_is_401() {
    let state     = infra_free_state().await;
    let router    = build_router(state);
    let bad_token = make_jwt("completely-wrong-secret", "read");

    let req = Request::builder()
        .method("POST")
        .uri(format!("/v1/xorbs/default/{}", "0".repeat(64)))
        .header(header::AUTHORIZATION, format!("Bearer {bad_token}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_expired_token_is_401() {
    let state   = infra_free_state().await;
    let router  = build_router(state);
    let expired = make_expired_jwt(TEST_SECRET);

    let req = Request::builder()
        .method("POST")
        .uri(format!("/v1/xorbs/default/{}", "0".repeat(64)))
        .header(header::AUTHORIZATION, format!("Bearer {expired}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_dedup_wrong_prefix_is_400() {
    let state  = infra_free_state().await;
    let router = build_router(state);
    let token  = make_jwt(TEST_SECRET, "read");

    let req = Request::builder()
        .method("GET")
        .uri(format!("/v1/chunks/wrong-prefix/{}", "0".repeat(64)))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_xorb_wrong_prefix_is_400() {
    let state  = infra_free_state().await;
    let router = build_router(state);
    let token  = make_jwt(TEST_SECRET, "write");

    let req = Request::builder()
        .method("POST")
        .uri(format!("/v1/xorbs/bad-prefix/{}", "0".repeat(64)))
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_xorb_invalid_hash_is_400() {
    let state  = infra_free_state().await;
    let router = build_router(state);
    let token  = make_jwt(TEST_SECRET, "write");

    let req = Request::builder()
        .method("POST")
        .uri("/v1/xorbs/default/notahash")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_reconstruction_invalid_file_id_is_400() {
    let state  = infra_free_state().await;
    let router = build_router(state);
    let token  = make_jwt(TEST_SECRET, "read");

    let req = Request::builder()
        .method("GET")
        .uri("/v1/reconstructions/notahash")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_health_returns_json_with_status_key() {
    // The health endpoint will get a DB error (lazy pool, unreachable host)
    // but must still return HTTP 200 with a JSON body containing "status".
    let state  = infra_free_state().await;
    let router = build_router(state);

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp.into_body()).await;
    assert!(body["status"].is_string(), "health body missing 'status': {body}");
    assert!(body["db"].is_string(),     "health body missing 'db': {body}");
}

// ── Shard binary builder (mirrors build_test_shard in shard-parser tests) ────

fn build_upload_shard(
    file_hash:  [u8; 32],
    sha256:     [u8; 32],
    xorb_hash:  [u8; 32],
    chunk_hash: [u8; 32],
) -> Vec<u8> {
    use shard_parser::types::{
        MDB_SHARD_HEADER_TAG, MDB_SHARD_HEADER_VERSION,
        MDB_FILE_FLAG_WITH_METADATA_EXT, MDB_FILE_FLAG_WITH_VERIFICATION, BOOKEND_HASH,
    };

    let mut buf = Vec::new();
    macro_rules! w32 { ($v:expr) => { buf.extend_from_slice(&($v as u32).to_le_bytes()) } }
    macro_rules! w64 { ($v:expr) => { buf.extend_from_slice(&($v as u64).to_le_bytes()) } }

    // Header (48 bytes)
    buf.extend_from_slice(&MDB_SHARD_HEADER_TAG);
    w64!(MDB_SHARD_HEADER_VERSION);
    w64!(0u64); // footer_size = 0 (no footer on upload)

    // FileDataSequenceHeader
    buf.extend_from_slice(&file_hash);
    w32!(MDB_FILE_FLAG_WITH_VERIFICATION | MDB_FILE_FLAG_WITH_METADATA_EXT);
    w32!(1u32);           // num_entries = 1
    buf.extend_from_slice(&[0u8; 8]); // _unused

    // FileDataSequenceEntry
    buf.extend_from_slice(&xorb_hash);
    w32!(0u32);           // cas_flags
    w32!(65536u32);       // unpacked_segment_bytes
    w32!(0u32);           // chunk_index_start
    w32!(1u32);           // chunk_index_end

    // FileVerificationEntry (1 entry matching num_entries)
    buf.extend_from_slice(&[0xAAu8; 32]); // range_hash
    buf.extend_from_slice(&[0u8; 16]);

    // FileMetadataExt
    buf.extend_from_slice(&sha256);
    buf.extend_from_slice(&[0u8; 16]);

    // File Info bookend
    buf.extend_from_slice(&BOOKEND_HASH);
    buf.extend_from_slice(&[0u8; 16]);

    // CASChunkSequenceHeader
    buf.extend_from_slice(&xorb_hash);
    w32!(0u32);           // cas_flags
    w32!(1u32);           // num_entries
    w32!(65536u32);       // num_bytes_in_cas
    w32!(66000u32);       // num_bytes_on_disk

    // CASChunkSequenceEntry
    buf.extend_from_slice(&chunk_hash);
    w32!(0u32);           // chunk_byte_range_start
    w32!(65536u32);       // unpacked_segment_bytes
    buf.extend_from_slice(&[0u8; 8]); // _unused

    // CAS Info bookend
    buf.extend_from_slice(&BOOKEND_HASH);
    buf.extend_from_slice(&[0u8; 16]);

    buf
}

// ── Full-stack tests ──────────────────────────────────────────────────────────

/// Returns integration DB URL from env, or None (test will be skipped).
fn integration_db_url() -> Option<String> {
    std::env::var("INTEGRATION_DATABASE_URL").ok()
}

fn integration_s3_endpoint() -> String {
    std::env::var("INTEGRATION_S3_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:9000".into())
}

async fn integration_state(db_url: &str) -> AppState {
    let pool = db_layer::create_pool(db_url)
        .await
        .expect("integration DB pool failed — is docker-compose running?");

    let s3 = S3Client::new(
        Some(&integration_s3_endpoint()),
        "us-east-1",
        "xet-storage",
        &std::env::var("AWS_ACCESS_KEY_ID").unwrap_or_else(|_| "minioadmin".into()),
        &std::env::var("AWS_SECRET_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".into()),
    )
    .await
    .expect("integration S3 client failed — is MinIO running?");

    let config = AppConfig {
        bind_addr:       "127.0.0.1:0".into(),
        database_url:    db_url.into(),
        s3_endpoint:     Some(integration_s3_endpoint()),
        s3_bucket:       "xet-storage".into(),
        s3_region:       "us-east-1".into(),
        jwt_secret:      TEST_SECRET.into(),
        jwt_expiry_secs: 3600,
        cas_base_url:    "http://localhost:3000".into(),
    };

    AppState::new(pool, s3, config)
}

#[tokio::test]
#[ignore = "requires INTEGRATION_DATABASE_URL + docker-compose stack"]
async fn test_health_with_live_db() {
    let db_url = match integration_db_url() {
        Some(u) => u,
        None    => { eprintln!("Skipped: INTEGRATION_DATABASE_URL not set"); return; }
    };

    let state  = integration_state(&db_url).await;
    let router = build_router(state);

    let req = Request::builder()
        .method("GET")
        .uri("/health")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp.into_body()).await;
    assert_eq!(body["status"], "ok",        "db not connected: {body}");
    assert_eq!(body["db"],     "connected", "db not connected: {body}");
}

#[tokio::test]
#[ignore = "requires INTEGRATION_DATABASE_URL + docker-compose stack"]
async fn test_full_upload_dedup_reconstruct_flow() {
    let db_url = match integration_db_url() {
        Some(u) => u,
        None    => { eprintln!("Skipped: INTEGRATION_DATABASE_URL not set"); return; }
    };

    let state  = integration_state(&db_url).await;
    let token  = make_jwt(TEST_SECRET, "write");
    let bearer = format!("Bearer {token}");

    // Use unique-ish hashes to avoid cross-test collisions
    let xorb_raw:  [u8; 32] = [0xA3u8; 32];
    let chunk_raw: [u8; 32] = [0xA4u8; 32];
    let file_raw:  [u8; 32] = [0xA1u8; 32];
    let sha256:    [u8; 32] = [0xA2u8; 32];

    let xorb_api  = common::hash_to_api_string(&xorb_raw);
    let chunk_api = common::hash_to_api_string(&chunk_raw);
    let file_api  = common::hash_to_api_string(&file_raw);

    // ── Step 1: Upload the xorb ───────────────────────────────────────────────
    {
        let resp = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/v1/xorbs/default/{xorb_api}"))
                    .header(header::AUTHORIZATION, &bearer)
                    .body(axum::body::Body::from(Bytes::from(vec![0xABu8; 256])))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "xorb upload failed");
        let body = json_body(resp.into_body()).await;
        assert!(
            body["was_inserted"].as_bool().is_some(),
            "was_inserted missing: {body}"
        );
    }

    // ── Step 2: Upload a shard referencing that xorb ──────────────────────────
    {
        let shard_bytes = build_upload_shard(file_raw, sha256, xorb_raw, chunk_raw);

        let resp = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/shards")
                    .header(header::AUTHORIZATION, &bearer)
                    .body(axum::body::Body::from(Bytes::from(shard_bytes)))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status     = resp.status();
        let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            status,
            StatusCode::OK,
            "shard upload failed: {}",
            String::from_utf8_lossy(&body_bytes)
        );
    }

    // ── Step 3: Global dedup query for the chunk ──────────────────────────────
    {
        let resp = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/v1/chunks/default-merkledb/{chunk_api}"))
                    .header(header::AUTHORIZATION, &bearer)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "dedup query failed");

        let dedup_bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let parsed = shard_parser::parse_shard(&dedup_bytes)
            .expect("dedup response must be a parseable MDB shard");

        // The dedup shard must describe the xorb containing our chunk
        assert_eq!(parsed.xorbs.len(), 1, "expected exactly 1 xorb in dedup shard");
        assert_eq!(parsed.xorbs[0].xorb_hash, xorb_raw, "wrong xorb in dedup shard");
        assert_eq!(parsed.xorbs[0].chunks.len(), 1);
        assert_eq!(parsed.xorbs[0].chunks[0].chunk_hash, chunk_raw);
    }

    // ── Step 4: Reconstruction query for the file ─────────────────────────────
    {
        let resp = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/v1/reconstructions/{file_api}"))
                    .header(header::AUTHORIZATION, &bearer)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK, "reconstruction query failed");

        let recon = json_body(resp.into_body()).await;
        let terms = recon["terms"].as_array().expect("terms must be an array");
        assert!(!terms.is_empty(), "reconstruction returned no terms");
        assert_eq!(
            terms[0]["hash"], xorb_api,
            "first term must reference our xorb"
        );
        assert!(
            recon.get("fetch_info").is_some(),
            "fetch_info missing from reconstruction response"
        );
    }

    // ── Step 5: Unknown chunk returns 404 ─────────────────────────────────────
    {
        let unknown = common::hash_to_api_string(&[0xFFu8; 32]);
        let resp = build_router(state.clone())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/v1/chunks/default-merkledb/{unknown}"))
                    .header(header::AUTHORIZATION, &bearer)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "unknown chunk must return 404");
    }
}
