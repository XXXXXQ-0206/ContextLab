-- Private execution retries bind to the exact immutable evidence scope. The receipt is kept
-- separate from the evidence payload so a retry may carry a new decision UUID without creating
-- a second evaluator call or changing append-only evidence.
CREATE TABLE benchmark_execution_idempotency (
    project_id UUID NOT NULL,
    context_id UUID NOT NULL,
    context_commit_id UUID NOT NULL,
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
    decision_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, context_id, context_commit_id, idempotency_key),
    UNIQUE (project_id, context_id, context_commit_id, decision_id),
    FOREIGN KEY (project_id, context_id, context_commit_id, decision_id)
        REFERENCES benchmark_decision_evidence(
            project_id, context_id, context_commit_id, decision_id
        ) ON DELETE RESTRICT
);

CREATE INDEX idx_benchmark_execution_idempotency_decision
    ON benchmark_execution_idempotency(
        project_id, context_id, context_commit_id, decision_id
    );

CREATE FUNCTION prevent_benchmark_execution_idempotency_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'benchmark execution idempotency receipts are append-only';
END;
$$;

CREATE TRIGGER benchmark_execution_idempotency_append_only
BEFORE UPDATE OR DELETE ON benchmark_execution_idempotency
FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_execution_idempotency_mutation();
