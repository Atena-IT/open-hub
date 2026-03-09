-- 002_hub_schema.sql — Hub platform tables

CREATE TABLE IF NOT EXISTS users (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    username    TEXT        NOT NULL UNIQUE,
    email       TEXT        UNIQUE,
    password_hash TEXT      NOT NULL,
    full_name   TEXT        NOT NULL DEFAULT '',
    is_org      BOOLEAN     NOT NULL DEFAULT FALSE,
    avatar_url  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

CREATE TABLE IF NOT EXISTS org_members (
    org_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role        TEXT        NOT NULL DEFAULT 'member',  -- owner, admin, member
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (org_id, user_id)
);

CREATE TABLE IF NOT EXISTS access_tokens (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    token_hash  BYTEA       NOT NULL UNIQUE,  -- SHA-256 of the ox_* token
    scopes      TEXT[]      NOT NULL DEFAULT '{}',
    last_used   TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_access_tokens_user ON access_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_access_tokens_hash ON access_tokens(token_hash);

CREATE TABLE IF NOT EXISTS repositories (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    full_name   TEXT        NOT NULL UNIQUE,   -- "owner/repo"
    repo_type   TEXT        NOT NULL DEFAULT 'model',  -- model, dataset, space
    private     BOOLEAN     NOT NULL DEFAULT FALSE,
    head_sha    TEXT,
    description TEXT        NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_repos_owner ON repositories(owner_id);
CREATE INDEX IF NOT EXISTS idx_repos_full_name ON repositories(full_name);

CREATE TABLE IF NOT EXISTS repo_files (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id     UUID        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    path        TEXT        NOT NULL,
    size        BIGINT      NOT NULL DEFAULT 0,
    sha256      TEXT,
    s3_key      TEXT,
    is_lfs      BOOLEAN     NOT NULL DEFAULT FALSE,
    lfs_oid     TEXT,        -- LFS SHA-256 OID if is_lfs
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, path)
);
CREATE INDEX IF NOT EXISTS idx_repo_files_repo ON repo_files(repo_id);

CREATE TABLE IF NOT EXISTS commits (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id     UUID        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    sha         TEXT        NOT NULL,
    message     TEXT        NOT NULL DEFAULT '',
    author_id   UUID        REFERENCES users(id),
    parent_sha  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_commits_repo ON commits(repo_id);
CREATE INDEX IF NOT EXISTS idx_commits_sha ON commits(repo_id, sha);

CREATE TABLE IF NOT EXISTS lfs_objects (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id     UUID        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    oid         TEXT        NOT NULL,  -- SHA-256 hex
    size        BIGINT      NOT NULL,
    s3_key      TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, oid)
);
CREATE INDEX IF NOT EXISTS idx_lfs_objects_repo ON lfs_objects(repo_id);
