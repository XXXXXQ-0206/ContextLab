CREATE TABLE context_authorization_audit_retention_policies (
    revision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE RESTRICT,
    retention_duration INTERVAL NOT NULL CHECK (retention_duration >= interval '0 seconds'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (workspace_id, revision_id)
);

CREATE TABLE context_authorization_audit_retention_policy_scopes (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE RESTRICT,
    active_policy_revision_id UUID NOT NULL,
    activated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (workspace_id, active_policy_revision_id)
        REFERENCES context_authorization_audit_retention_policies(workspace_id, revision_id)
        ON DELETE RESTRICT
);

CREATE FUNCTION prevent_context_authorization_audit_retention_policy_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'context authorization audit retention policies are immutable';
END;
$$;

CREATE TRIGGER context_authorization_audit_retention_policies_append_only
    BEFORE UPDATE OR DELETE ON context_authorization_audit_retention_policies
    FOR EACH ROW
    EXECUTE FUNCTION prevent_context_authorization_audit_retention_policy_mutation();

ALTER TABLE context_authorization_audit_events
    ADD COLUMN retention_disposition TEXT NOT NULL DEFAULT 'hold'
        CHECK (retention_disposition IN ('hold', 'purge_eligible')),
    ADD COLUMN retention_policy_revision_id UUID,
    ADD COLUMN purge_eligible_at TIMESTAMPTZ,
    ADD CONSTRAINT chk_context_authorization_audit_events_retention_disposition
        CHECK (
            (retention_disposition = 'hold'
                AND retention_policy_revision_id IS NULL
                AND purge_eligible_at IS NULL)
            OR (retention_disposition = 'purge_eligible'
                AND retention_policy_revision_id IS NOT NULL
                AND purge_eligible_at IS NOT NULL)
        ),
    ADD CONSTRAINT fk_context_authorization_audit_events_retention_policy_revision
        FOREIGN KEY (retention_policy_revision_id)
        REFERENCES context_authorization_audit_retention_policies(revision_id)
        ON DELETE RESTRICT;

CREATE FUNCTION validate_context_authorization_audit_event_retention_policy()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
DECLARE
    policy_retention_duration INTERVAL;
BEGIN
    IF NEW.retention_policy_revision_id IS NULL THEN
        RETURN NEW;
    END IF;

    SELECT context_authorization_audit_retention_policies.retention_duration
    INTO policy_retention_duration
    FROM contexts
    JOIN projects ON projects.id = contexts.project_id
    JOIN context_authorization_audit_retention_policies
        ON context_authorization_audit_retention_policies.workspace_id = projects.workspace_id
    WHERE contexts.id = NEW.context_id
        AND context_authorization_audit_retention_policies.revision_id = NEW.retention_policy_revision_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'authorization audit retention policy must belong to the event workspace';
    END IF;

    IF NEW.retention_disposition = 'purge_eligible' THEN
        NEW.purge_eligible_at := NEW.recorded_at + policy_retention_duration;
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER context_authorization_audit_events_retention_policy_workspace
    BEFORE INSERT ON context_authorization_audit_events
    FOR EACH ROW
    EXECUTE FUNCTION validate_context_authorization_audit_event_retention_policy();

CREATE INDEX idx_context_authorization_audit_events_purge_eligible_context_expiry
    ON context_authorization_audit_events(context_id, purge_eligible_at, id)
    WHERE retention_disposition = 'purge_eligible';

CREATE TABLE context_authorization_audit_purge_manifests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE RESTRICT,
    policy_revision_id UUID NOT NULL,
    cutoff TIMESTAMPTZ NOT NULL,
    selected_event_count BIGINT NOT NULL CHECK (selected_event_count >= 0),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (workspace_id, policy_revision_id)
        REFERENCES context_authorization_audit_retention_policies(workspace_id, revision_id)
        ON DELETE RESTRICT
);

CREATE FUNCTION prevent_context_authorization_audit_purge_manifest_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'context authorization audit purge manifests are append-only';
END;
$$;

CREATE TRIGGER context_authorization_audit_purge_manifests_append_only
    BEFORE UPDATE OR DELETE ON context_authorization_audit_purge_manifests
    FOR EACH ROW
    EXECUTE FUNCTION prevent_context_authorization_audit_purge_manifest_mutation();
