CREATE TABLE workflow_execution_status_projections (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    run_id UUID NOT NULL,
    schema_version TEXT COLLATE "C" NOT NULL CHECK (
        schema_version = 'v1'
    ),
    projection JSONB NOT NULL CHECK (
        jsonb_typeof(projection) = 'object'
        AND projection ->> 'schema_version' = schema_version
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, context_id, context_commit_id, run_id),
    UNIQUE (project_id, context_id, run_id),
    FOREIGN KEY (project_id, context_id)
        REFERENCES contexts(project_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (context_id, context_commit_id)
        REFERENCES context_commits(context_id, id) ON DELETE RESTRICT
);

CREATE INDEX idx_workflow_execution_status_projections_context_run
    ON workflow_execution_status_projections(context_id, run_id);

CREATE FUNCTION prevent_workflow_execution_status_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'workflow execution status projections are append-only';
END;
$$;

CREATE TRIGGER workflow_execution_status_projections_append_only
BEFORE UPDATE OR DELETE ON workflow_execution_status_projections
FOR EACH ROW EXECUTE FUNCTION prevent_workflow_execution_status_mutation();
