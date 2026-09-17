ALTER TABLE workspace_memberships ADD COLUMN identity_source TEXT COLLATE "C";
UPDATE workspace_memberships SET identity_source = 'legacy';
ALTER TABLE workspace_memberships ALTER COLUMN identity_source SET NOT NULL;
ALTER TABLE workspace_memberships
    ADD CONSTRAINT chk_workspace_memberships_identity_source
    CHECK (
        octet_length(identity_source) > 0
        AND octet_length(identity_source) <= 2048
        AND identity_source = btrim(identity_source)
        AND identity_source !~ '^[[:space:]]|[[:space:]]$'
        AND identity_source !~ '[[:cntrl:]]'
    );
ALTER TABLE workspace_memberships DROP CONSTRAINT workspace_memberships_pkey;
ALTER TABLE workspace_memberships
    ALTER COLUMN principal_id TYPE TEXT COLLATE "C" USING principal_id;
ALTER TABLE workspace_memberships
    ADD CONSTRAINT chk_workspace_memberships_principal_id_identity
    CHECK (
        octet_length(principal_id) > 0
        AND octet_length(principal_id) <= 512
        AND principal_id = btrim(principal_id)
        AND principal_id !~ '^[[:space:]]|[[:space:]]$'
        AND principal_id !~ '[[:cntrl:]]'
    ) NOT VALID;
ALTER TABLE workspace_memberships
    ADD PRIMARY KEY (workspace_id, identity_source, principal_id);

ALTER TABLE context_commit_idempotency ADD COLUMN identity_source TEXT COLLATE "C";
UPDATE context_commit_idempotency SET identity_source = 'legacy';
ALTER TABLE context_commit_idempotency ALTER COLUMN identity_source SET NOT NULL;
ALTER TABLE context_commit_idempotency
    ADD CONSTRAINT chk_context_commit_idempotency_identity_source
    CHECK (
        octet_length(identity_source) > 0
        AND octet_length(identity_source) <= 2048
        AND identity_source = btrim(identity_source)
        AND identity_source !~ '^[[:space:]]|[[:space:]]$'
        AND identity_source !~ '[[:cntrl:]]'
    );
ALTER TABLE context_commit_idempotency DROP CONSTRAINT context_commit_idempotency_pkey;
ALTER TABLE context_commit_idempotency
    ALTER COLUMN principal_id TYPE TEXT COLLATE "C" USING principal_id;
ALTER TABLE context_commit_idempotency
    ADD CONSTRAINT chk_context_commit_idempotency_principal_id_identity
    CHECK (
        octet_length(principal_id) > 0
        AND octet_length(principal_id) <= 512
        AND principal_id = btrim(principal_id)
        AND principal_id !~ '^[[:space:]]|[[:space:]]$'
        AND principal_id !~ '[[:cntrl:]]'
    ) NOT VALID;
ALTER TABLE context_commit_idempotency
    ADD PRIMARY KEY (identity_source, principal_id, context_id, idempotency_key);

ALTER TABLE context_authorization_audit_events ADD COLUMN identity_source TEXT COLLATE "C";
UPDATE context_authorization_audit_events SET identity_source = 'legacy';
ALTER TABLE context_authorization_audit_events ALTER COLUMN identity_source SET NOT NULL;
ALTER TABLE context_authorization_audit_events
    ADD CONSTRAINT chk_context_authorization_audit_events_identity_source
    CHECK (
        octet_length(identity_source) > 0
        AND octet_length(identity_source) <= 2048
        AND identity_source = btrim(identity_source)
        AND identity_source !~ '^[[:space:]]|[[:space:]]$'
        AND identity_source !~ '[[:cntrl:]]'
    );
ALTER TABLE context_authorization_audit_events
    ALTER COLUMN principal_id TYPE TEXT COLLATE "C" USING principal_id;
ALTER TABLE context_authorization_audit_events
    ADD CONSTRAINT chk_context_authorization_audit_events_principal_id_identity
    CHECK (
        octet_length(principal_id) > 0
        AND octet_length(principal_id) <= 512
        AND principal_id = btrim(principal_id)
        AND principal_id !~ '^[[:space:]]|[[:space:]]$'
        AND principal_id !~ '[[:cntrl:]]'
    ) NOT VALID;

CREATE INDEX idx_workspace_memberships_identity_source_principal_id
    ON workspace_memberships(identity_source, principal_id);

CREATE INDEX idx_context_commit_idempotency_identity_source_principal_id
    ON context_commit_idempotency(identity_source, principal_id);

CREATE INDEX idx_context_authorization_audit_events_identity_principal_recorded_at
    ON context_authorization_audit_events(identity_source, principal_id, recorded_at DESC, id DESC);
