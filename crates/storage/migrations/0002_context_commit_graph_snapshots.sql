CREATE TABLE context_commit_graph_snapshots (
    commit_id UUID PRIMARY KEY REFERENCES context_commits(id) ON DELETE RESTRICT,
    schema_version SMALLINT NOT NULL CHECK (schema_version > 0),
    graph JSONB NOT NULL CHECK (jsonb_typeof(graph) = 'object'),
    captured_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_context_commits_context_id_id ON context_commits(context_id, id);
