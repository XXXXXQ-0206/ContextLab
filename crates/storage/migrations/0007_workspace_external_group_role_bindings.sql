CREATE TABLE workspace_external_group_role_bindings (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    identity_source TEXT COLLATE "C" NOT NULL CHECK (
        octet_length(identity_source) > 0
        AND octet_length(identity_source) <= 2048
        AND identity_source = btrim(identity_source)
        AND identity_source !~ '^[[:space:]]|[[:space:]]$'
        AND identity_source !~ '[[:cntrl:]]'
    ),
    external_group_id TEXT COLLATE "C" NOT NULL CHECK (
        octet_length(external_group_id) > 0
        AND octet_length(external_group_id) <= 512
        AND external_group_id = btrim(external_group_id)
        AND external_group_id !~ '^[[:space:]]|[[:space:]]$'
        AND external_group_id !~ '[[:cntrl:]]'
    ),
    role TEXT NOT NULL CHECK (role IN ('reader', 'editor')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_workspace_external_group_role_bindings_active
    ON workspace_external_group_role_bindings(workspace_id, identity_source, external_group_id)
    WHERE deleted_at IS NULL;
