CREATE TABLE context_authorization_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    principal_id TEXT NOT NULL CHECK (length(trim(principal_id)) > 0),
    context_id UUID NOT NULL REFERENCES contexts(id) ON DELETE RESTRICT,
    permission TEXT NOT NULL CHECK (permission IN ('read', 'write')),
    decision TEXT NOT NULL CHECK (decision IN ('granted', 'forbidden', 'unavailable')),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_context_authorization_audit_events_context_recorded_at
    ON context_authorization_audit_events(context_id, recorded_at DESC, id DESC);

CREATE INDEX idx_context_authorization_audit_events_principal_recorded_at
    ON context_authorization_audit_events(principal_id, recorded_at DESC, id DESC);

CREATE FUNCTION prevent_context_authorization_audit_event_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'context authorization audit events are append-only';
END;
$$;

CREATE TRIGGER context_authorization_audit_events_append_only
    BEFORE UPDATE OR DELETE ON context_authorization_audit_events
    FOR EACH ROW
    EXECUTE FUNCTION prevent_context_authorization_audit_event_mutation();
