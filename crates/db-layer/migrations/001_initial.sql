-- 001_initial.sql
-- All multi-byte integers stored as bytea (raw 32 bytes) for hashes.

CREATE TABLE IF NOT EXISTS xorbs (
    hash        BYTEA        PRIMARY KEY,
    s3_key      TEXT         NOT NULL,
    size_bytes  BIGINT       NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS chunks (
    hash                    BYTEA   PRIMARY KEY,
    xorb_hash               BYTEA   NOT NULL REFERENCES xorbs(hash) ON DELETE RESTRICT,
    chunk_index_in_xorb     INT     NOT NULL,
    byte_range_start        INT     NOT NULL,
    unpacked_segment_bytes  INT     NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_chunks_xorb_hash ON chunks(xorb_hash);

CREATE TABLE IF NOT EXISTS shards (
    hash        BYTEA        PRIMARY KEY,
    s3_key      TEXT         NOT NULL,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- reconstruction_terms is a JSONB array of {xorb_hash, chunk_index_start, chunk_index_end, unpacked_length}
CREATE TABLE IF NOT EXISTS file_mappings (
    file_hash            BYTEA        PRIMARY KEY,
    sha256               BYTEA        NOT NULL,
    reconstruction_terms JSONB        NOT NULL,
    created_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_file_mappings_sha256 ON file_mappings(sha256);
