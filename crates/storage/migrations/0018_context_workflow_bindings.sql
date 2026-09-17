-- Immutable provider-free Workflow definitions sourced from exact materialized Context commits.
CREATE TABLE context_workflow_bindings (
    id UUID PRIMARY KEY,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    workflow_id UUID NOT NULL,
    workflow_revision BIGINT NOT NULL CHECK (workflow_revision > 0),
    binding JSONB NOT NULL CHECK (jsonb_typeof(binding) = 'object'),
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_context_workflow_bindings_context_commit
        FOREIGN KEY (context_id, context_commit_id)
        REFERENCES context_commits(context_id, id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_context_workflow_bindings_materialized_snapshot
        FOREIGN KEY (context_commit_id)
        REFERENCES context_commit_graph_snapshots(commit_id)
        ON DELETE RESTRICT,
    CONSTRAINT uq_context_workflow_bindings_workflow_revision
        UNIQUE (workflow_id, workflow_revision)
);

CREATE INDEX idx_context_workflow_bindings_context_commit
    ON context_workflow_bindings(context_id, context_commit_id, workflow_id, workflow_revision, id);
