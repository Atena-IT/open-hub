CREATE TABLE IF NOT EXISTS repo_refs (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    repo_id     UUID        NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    ref_type    TEXT        NOT NULL,
    target_sha  TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, name)
);

CREATE INDEX IF NOT EXISTS idx_repo_refs_repo ON repo_refs(repo_id);
CREATE INDEX IF NOT EXISTS idx_repo_refs_type ON repo_refs(repo_id, ref_type);
