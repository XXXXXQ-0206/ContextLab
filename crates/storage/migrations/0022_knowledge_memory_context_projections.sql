CREATE TABLE knowledge_memory_context_projections (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    schema_version TEXT COLLATE "C" NOT NULL CHECK (
        schema_version = 'knowledge-memory-context-projection-v1'
    ),
    projection JSONB NOT NULL CHECK (jsonb_typeof(projection) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, context_id, context_commit_id),
    FOREIGN KEY (project_id, context_id)
        REFERENCES contexts(project_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (context_id, context_commit_id)
        REFERENCES context_commits(context_id, id) ON DELETE RESTRICT
);

CREATE INDEX idx_knowledge_memory_context_projections_context
    ON knowledge_memory_context_projections(context_id, context_commit_id);

CREATE FUNCTION prevent_knowledge_memory_context_projection_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Knowledge/Memory context projections are append-only';
END;
$$;

CREATE TRIGGER knowledge_memory_context_projection_append_only
BEFORE UPDATE OR DELETE ON knowledge_memory_context_projections
FOR EACH ROW EXECUTE FUNCTION prevent_knowledge_memory_context_projection_mutation();
