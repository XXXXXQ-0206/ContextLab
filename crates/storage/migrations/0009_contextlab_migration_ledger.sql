CREATE TABLE contextlab_schema_migration_ledger (
    migration_id TEXT PRIMARY KEY,
    migration_sha256 TEXT NOT NULL CHECK (migration_sha256 ~ '^[0-9a-f]{64}$'),
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE FUNCTION prevent_contextlab_schema_migration_ledger_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'contextlab schema migration ledger is append-only';
END;
$$;

CREATE TRIGGER contextlab_schema_migration_ledger_append_only
    BEFORE UPDATE OR DELETE ON contextlab_schema_migration_ledger
    FOR EACH ROW
    EXECUTE FUNCTION prevent_contextlab_schema_migration_ledger_mutation();
