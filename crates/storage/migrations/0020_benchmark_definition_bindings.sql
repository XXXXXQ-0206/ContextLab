CREATE TABLE benchmark_definition_bindings (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
    binding_id UUID NOT NULL,
    suite_id UUID NOT NULL,
    branch_name TEXT COLLATE "C" NOT NULL CHECK (
        octet_length(branch_name) > 0
        AND branch_name = btrim(branch_name)
        AND branch_name !~ '^[[:space:]]|[[:space:]]$'
        AND branch_name !~ '[[:cntrl:]]'
    ),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 1),
    identity_source TEXT COLLATE "C" NOT NULL CHECK (
        octet_length(identity_source) > 0
        AND identity_source = btrim(identity_source)
        AND identity_source !~ '[[:cntrl:]]'
    ),
    principal_id TEXT COLLATE "C" NOT NULL CHECK (
        octet_length(principal_id) > 0
        AND principal_id = btrim(principal_id)
        AND principal_id !~ '[[:cntrl:]]'
    ),
    idempotency_key TEXT COLLATE "C" NOT NULL CHECK (
        octet_length(idempotency_key) > 0
        AND idempotency_key = btrim(idempotency_key)
        AND idempotency_key !~ '[[:cntrl:]]'
    ),
    request_digest TEXT COLLATE "C" NOT NULL CHECK (
        octet_length(request_digest) > 0
        AND request_digest = btrim(request_digest)
        AND request_digest !~ '[[:cntrl:]]'
    ),
    captured_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, binding_id),
    UNIQUE (project_id, context_id, context_commit_id, suite_id),
    UNIQUE (
        identity_source,
        principal_id,
        context_id,
        branch_name,
        idempotency_key
    ),
    FOREIGN KEY (project_id, context_id)
        REFERENCES contexts(project_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (context_id, context_commit_id)
        REFERENCES context_commits(context_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (project_id, suite_id)
        REFERENCES benchmark_suite_definitions(project_id, id) ON DELETE RESTRICT
);

CREATE INDEX idx_benchmark_definition_bindings_context_commit
    ON benchmark_definition_bindings(project_id, context_id, context_commit_id, binding_id);

CREATE FUNCTION prevent_benchmark_definition_binding_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'benchmark definition bindings are append-only';
END;
$$;

CREATE TRIGGER benchmark_definition_bindings_append_only
BEFORE UPDATE OR DELETE ON benchmark_definition_bindings
FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_definition_binding_mutation();
