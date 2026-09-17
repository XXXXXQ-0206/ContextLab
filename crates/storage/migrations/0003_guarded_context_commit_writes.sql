CREATE TABLE workspace_memberships (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    principal_id TEXT NOT NULL CHECK (length(trim(principal_id)) > 0),
    role TEXT NOT NULL CHECK (role IN ('owner', 'editor', 'reader')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (workspace_id, principal_id)
);

CREATE TABLE context_branches (
    context_id UUID NOT NULL REFERENCES contexts(id) ON DELETE CASCADE,
    branch_name TEXT NOT NULL CHECK (length(trim(branch_name)) > 0),
    head_commit_id UUID REFERENCES context_commits(id) ON DELETE RESTRICT,
    revision BIGINT NOT NULL DEFAULT 0 CHECK (revision >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (context_id, branch_name)
);

CREATE TABLE context_commit_idempotency (
    principal_id TEXT NOT NULL CHECK (length(trim(principal_id)) > 0),
    context_id UUID NOT NULL REFERENCES contexts(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL CHECK (length(trim(idempotency_key)) > 0),
    request_digest TEXT NOT NULL CHECK (length(trim(request_digest)) > 0),
    commit_id UUID NOT NULL REFERENCES context_commits(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (principal_id, context_id, idempotency_key)
);

CREATE INDEX idx_workspace_memberships_principal_id ON workspace_memberships(principal_id);
CREATE INDEX idx_context_branches_head_commit_id ON context_branches(head_commit_id);
CREATE INDEX idx_context_commit_idempotency_commit_id ON context_commit_idempotency(commit_id);
