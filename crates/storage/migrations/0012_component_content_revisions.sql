ALTER TABLE context_components
    ADD CONSTRAINT uq_context_components_context_id_id UNIQUE (context_id, id);

CREATE TABLE context_component_content_revisions (
    context_id UUID NOT NULL,
    commit_id UUID NOT NULL REFERENCES context_commits(id) ON DELETE RESTRICT,
    component_id UUID NOT NULL REFERENCES context_components(id) ON DELETE RESTRICT,
    previous_content_hash TEXT NOT NULL CHECK (length(trim(previous_content_hash)) > 0),
    content_hash TEXT NOT NULL CHECK (length(trim(content_hash)) > 0),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (commit_id, component_id),
    FOREIGN KEY (context_id, commit_id)
        REFERENCES context_commits (context_id, id)
        ON DELETE RESTRICT,
    FOREIGN KEY (context_id, component_id)
        REFERENCES context_components (context_id, id)
        ON DELETE RESTRICT
);

CREATE INDEX idx_context_component_content_revisions_context_component
    ON context_component_content_revisions (context_id, component_id, created_at DESC);
