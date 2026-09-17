CREATE TABLE context_diff_snapshots (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    schema_version TEXT COLLATE "C" NOT NULL CHECK (
        schema_version = 'context-diff-snapshot-v1'
    ),
    snapshot JSONB NOT NULL CHECK (jsonb_typeof(snapshot) = 'object'),
    snapshot_digest TEXT COLLATE "C" NOT NULL CHECK (
        snapshot_digest ~ '^sha256:[0-9a-f]{64}$'
    ),
    captured_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, context_id, context_commit_id, schema_version),
    FOREIGN KEY (project_id, context_id)
        REFERENCES contexts(project_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (context_id, context_commit_id)
        REFERENCES context_commits(context_id, id) ON DELETE RESTRICT
);

CREATE INDEX idx_context_diff_snapshots_context_commit
    ON context_diff_snapshots(context_id, context_commit_id, schema_version);

CREATE FUNCTION prevent_context_diff_snapshot_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Context diff snapshots are append-only';
END;
$$;

CREATE TRIGGER context_diff_snapshots_append_only
BEFORE UPDATE OR DELETE ON context_diff_snapshots
FOR EACH ROW EXECUTE FUNCTION prevent_context_diff_snapshot_mutation();
