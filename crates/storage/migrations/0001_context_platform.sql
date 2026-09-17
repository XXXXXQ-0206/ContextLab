CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE workspaces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    slug TEXT NOT NULL CHECK (length(trim(slug)) > 0),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    slug TEXT NOT NULL CHECK (length(trim(slug)) > 0),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE experiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    branch_name TEXT NOT NULL CHECK (length(trim(branch_name)) > 0),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id),
    experiment_id UUID REFERENCES experiments(id),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    description TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE context_components (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    context_id UUID NOT NULL REFERENCES contexts(id),
    kind TEXT NOT NULL CHECK (kind IN (
        'prompt',
        'system_prompt',
        'memory',
        'knowledge',
        'retrieval',
        'embedding',
        'model_configuration',
        'tool',
        'mcp_server',
        'variable',
        'output_schema',
        'workflow',
        'conversation',
        'evaluation'
    )),
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    content_hash TEXT NOT NULL CHECK (length(trim(content_hash)) > 0),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE context_commits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    context_id UUID NOT NULL REFERENCES contexts(id),
    branch_name TEXT NOT NULL CHECK (length(trim(branch_name)) > 0),
    message TEXT NOT NULL CHECK (length(trim(message)) > 0),
    changes JSONB NOT NULL DEFAULT '[]'::jsonb,
    authored_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE context_commit_parents (
    commit_id UUID NOT NULL REFERENCES context_commits(id) ON DELETE CASCADE,
    parent_commit_id UUID NOT NULL REFERENCES context_commits(id) ON DELETE RESTRICT,
    position INTEGER NOT NULL CHECK (position >= 0),
    PRIMARY KEY (commit_id, parent_commit_id),
    UNIQUE (commit_id, position),
    CHECK (commit_id <> parent_commit_id)
);

CREATE TABLE evaluation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    context_id UUID NOT NULL REFERENCES contexts(id),
    suite_name TEXT NOT NULL CHECK (length(trim(suite_name)) > 0),
    model_version TEXT NOT NULL CHECK (length(trim(model_version)) > 0),
    temperature REAL NOT NULL,
    metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_workspaces_slug_active ON workspaces(slug) WHERE deleted_at IS NULL;
CREATE INDEX idx_projects_workspace_id ON projects(workspace_id) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_projects_workspace_slug_active ON projects(workspace_id, slug) WHERE deleted_at IS NULL;
CREATE INDEX idx_experiments_project_id ON experiments(project_id) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX idx_experiments_project_branch_active ON experiments(project_id, branch_name) WHERE deleted_at IS NULL;
CREATE INDEX idx_contexts_project_id ON contexts(project_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_contexts_experiment_id ON contexts(experiment_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_context_components_context_id ON context_components(context_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_context_commits_context_id ON context_commits(context_id);
CREATE INDEX idx_context_commit_parents_parent_id ON context_commit_parents(parent_commit_id);
CREATE INDEX idx_evaluation_runs_context_id ON evaluation_runs(context_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_workspaces_metadata_gin ON workspaces USING gin(metadata);
CREATE INDEX idx_projects_metadata_gin ON projects USING gin(metadata);
CREATE INDEX idx_contexts_metadata_gin ON contexts USING gin(metadata);
