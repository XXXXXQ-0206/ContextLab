ALTER TABLE context_commit_graph_snapshots
    ADD CONSTRAINT context_commit_graph_snapshots_v1_schema
    CHECK (schema_version = 1);

CREATE FUNCTION prevent_context_commit_graph_snapshot_mutation()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'Context commit graph snapshots are append-only';
END;
$$;

CREATE TRIGGER context_commit_graph_snapshots_append_only
BEFORE UPDATE OR DELETE ON context_commit_graph_snapshots
FOR EACH ROW EXECUTE FUNCTION prevent_context_commit_graph_snapshot_mutation();
