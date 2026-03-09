Building an AI factory capable of managing datasets and checkpoints across all model sizes is an incredible undertaking. If your goal is to use the Xet protocol as your Tier 3 immutable registry (the cold store where all versioning and collaboration happen), implementing a custom backend in Rust that speaks natively to Hugging Face’s `xet-core` client is the most performant path forward.

Here is a structured Statement of Work (SOW) to build, verify, and cloud-integrate this Rust-based Xet backend.

# Statement of Work: Custom Xet Storage Backend (Rust)

**Project Objective:** To design, implement, and verify a custom Content-Addressable Storage (CAS) backend in Rust that strictly adheres to the open Xet protocol. The backend will serve as a private Hugging Face Hub alternative, natively supporting the open-source `xet-core` client, performing chunk-level deduplication, and leveraging AWS S3 (or compatible object storage) for scalable data persistence.

### 1. Reference Documentation & Research Scope

Before and during implementation, the engineering team must source, navigate, and search the following official Hugging Face documentation to ensure protocol compliance:

**Client Implementation & Usage:**

* **Official Rust Client:** [github.com/huggingface/xet-core](https://github.com/huggingface/xet-core)
* **Hub Integration:** [docs/hub/xet/index](https://huggingface.co/docs/hub/xet/index), [overview](https://huggingface.co/docs/hub/xet/overview), [using-xet-storage](https://huggingface.co/docs/hub/xet/using-xet-storage)
* **Legacy & Security:** [security](https://huggingface.co/docs/hub/xet/security), [legacy-git-lfs](https://huggingface.co/docs/hub/xet/legacy-git-lfs)

**Xet Protocol API & Interactions:**

* **Protocol Basics:** [docs/xet/index](https://huggingface.co/docs/xet/index)
* **Network Flows:** [upload-protocol](https://huggingface.co/docs/xet/upload-protocol), [download-protocol](https://huggingface.co/docs/xet/download-protocol)
* **API & Auth:** [api](https://huggingface.co/docs/xet/api), [auth](https://huggingface.co/docs/xet/auth), [file-id](https://huggingface.co/docs/xet/file-id)

**Underlying Architecture & Data Structures:**

* **Chunking & Hashing:** [chunking](https://huggingface.co/docs/xet/chunking), [hashing](https://huggingface.co/docs/xet/hashing)
* **Storage Formats:** [xorb](https://huggingface.co/docs/xet/xorb), [shard](https://huggingface.co/docs/xet/shard)
* **Reconstruction & Optimization:** [file-reconstruction](https://huggingface.co/docs/xet/file-reconstruction), [deduplication](https://huggingface.co/docs/xet/deduplication)

---

### 2. Phase 1: Architecture & Foundation

Establish the core server components and connections to local storage/database layers.

* **Web Framework:** Initialize a high-performance asynchronous HTTP server using Rust's `axum` framework paired with the `tokio` runtime to handle thousands of concurrent chunk requests.
* **Database Layer (PostgreSQL):** Integrate `sqlx` for asynchronous database connections. Define the schema to map Git revisions to File IDs, and File IDs to `CASReconstructionTerm` arrays (which map chunks to Xorbs).
* **Storage Layer (S3 API):** Implement the `aws-sdk-s3` crate. Configure the client to accept custom endpoint URLs so it can target a local MinIO instance during testing and Amazon S3 in production.

### 3. Phase 2: Protocol Implementation

Implement the endpoints defined in the Xet CAS API specification.

* **Authentication Mock (`GET /api/models/.../xet-read-token`):** Build a token vending machine that mirrors Hugging Face's auth logic, returning short-lived JWTs and the `casUrl` pointing to your new backend.
* **Upload Protocol (`POST /v1/xorbs/...` & `POST /v1/shards/...`):** * Create endpoints to receive binary Xorb payloads (up to 64MB) and stream them directly into S3 using Multipart Uploads.
* Implement an MDB Shard parser (reusing data structures from `xet-core` where possible) to extract chunk metadata from uploaded shards and persist them to PostgreSQL.


* **Global Deduplication (`GET /v1/chunks/...`):** Implement the endpoint that allows clients to query the Postgres database for existing chunk hashes, enabling the client to skip uploading data that already exists in your S3 bucket.
* **Download Protocol (`GET /v1/reconstructions/...`):** Build the endpoint that takes a requested file hash, queries Postgres for the associated Xorbs, and returns the byte-range mapping. Crucially, this must generate **S3 Pre-signed URLs** so the `xet-core` client can download chunks directly from S3, bypassing the Axum server.

### 4. Phase 3: Local Verification & Testing

Before touching cloud infrastructure, the entire system must be verified locally against the official Hugging Face client.

* **Docker Compose Setup:** Create a `docker-compose.yml` that spins up the Rust Axum server, a PostgreSQL container, and a MinIO container (acting as local S3).
* **Client Integration:** Compile the official `xet-core` client locally.
* **End-to-End Test:** Configure the `xet-core` environment variables (or a custom wrapper script) to point to your local Auth endpoint. Execute a `push` of a gigabyte-sized dummy file.
* **Validation:** 1. Verify MinIO contains the raw Xorb binary blobs.
2. Verify Postgres contains the correct chunk-to-xorb mappings.
3. Execute a `pull` using `xet-core` and run a SHA-256 checksum to guarantee the reconstructed file matches the original perfectly.

### 5. Phase 4: Cloud Provider Integration (AWS)

Transition the locally verified system to an enterprise cloud environment.

* **Compute & Scaling:** Containerize the Rust backend and deploy it to an orchestrator like Amazon EKS (Kubernetes) or ECS. Because Xet is highly parallel, configure horizontal pod autoscaling based on network I/O.
* **Managed Database:** Migrate the database tier to Amazon RDS for PostgreSQL. Ensure high read-replica availability, as global deduplication queries will hit the database heavily during large model uploads.
* **S3 Optimization:** Switch the S3 SDK target from MinIO to Amazon S3. Implement S3 Lifecycle policies to manage partial or failed Xorb multipart uploads.