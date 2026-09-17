-- Deterministic PostgreSQL seed data for workspace Context Graph integration tests.
-- This fixture is intentionally small and belongs to contextlab-storage so SQL
-- details stay behind the repository boundary.

INSERT INTO workspaces (id, name, slug, metadata, deleted_at)
VALUES
    (
        '11111111-1111-4111-8111-111111111111',
        'Seed Workspace',
        'seed-workspace',
        '{"seed": true, "locale": "en-US"}'::jsonb,
        NULL
    ),
    (
        '11111111-1111-4111-8111-999999999999',
        'Deleted Seed Workspace',
        'deleted-seed-workspace',
        '{"seed": true, "deleted": true}'::jsonb,
        now()
    )
ON CONFLICT (id) DO UPDATE
SET name = EXCLUDED.name,
    slug = EXCLUDED.slug,
    metadata = EXCLUDED.metadata,
    deleted_at = EXCLUDED.deleted_at;

INSERT INTO projects (id, workspace_id, name, slug, metadata, deleted_at)
VALUES
    (
        '22222222-2222-4222-8222-222222222222',
        '11111111-1111-4111-8111-111111111111',
        'Seed Support AI Project',
        'seed-support-ai',
        '{"seed": true}'::jsonb,
        NULL
    ),
    (
        '22222222-2222-4222-8222-999999999999',
        '11111111-1111-4111-8111-111111111111',
        'Deleted Seed Project',
        'deleted-seed-project',
        '{"seed": true, "deleted": true}'::jsonb,
        now()
    )
ON CONFLICT (id) DO UPDATE
SET workspace_id = EXCLUDED.workspace_id,
    name = EXCLUDED.name,
    slug = EXCLUDED.slug,
    metadata = EXCLUDED.metadata,
    deleted_at = EXCLUDED.deleted_at;

INSERT INTO experiments (id, project_id, name, branch_name, metadata, deleted_at)
VALUES
    (
        '33333333-3333-4333-8333-333333333333',
        '22222222-2222-4222-8222-222222222222',
        'Seed RAG Evaluation',
        'seed/rag-evaluation',
        '{"seed": true}'::jsonb,
        NULL
    )
ON CONFLICT (id) DO UPDATE
SET project_id = EXCLUDED.project_id,
    name = EXCLUDED.name,
    branch_name = EXCLUDED.branch_name,
    metadata = EXCLUDED.metadata,
    deleted_at = EXCLUDED.deleted_at;

INSERT INTO contexts (id, project_id, experiment_id, name, description, metadata, deleted_at)
VALUES
    (
        '44444444-4444-4444-8444-444444444444',
        '22222222-2222-4222-8222-222222222222',
        '33333333-3333-4333-8333-333333333333',
        'Seed Support Resolution Context',
        'Seed context used by SQLx workspace graph integration tests.',
        '{"seed": true}'::jsonb,
        NULL
    ),
    (
        '44444444-4444-4444-8444-999999999999',
        '22222222-2222-4222-8222-222222222222',
        '33333333-3333-4333-8333-333333333333',
        'Deleted Seed Context',
        'Soft-deleted context that should not appear in graph projections.',
        '{"seed": true, "deleted": true}'::jsonb,
        now()
    )
ON CONFLICT (id) DO UPDATE
SET project_id = EXCLUDED.project_id,
    experiment_id = EXCLUDED.experiment_id,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    metadata = EXCLUDED.metadata,
    deleted_at = EXCLUDED.deleted_at;

INSERT INTO context_components (id, context_id, kind, name, content_hash, metadata, deleted_at)
VALUES
    (
        '55555555-5555-4555-8555-555555555550',
        '44444444-4444-4444-8444-444444444444',
        'system_prompt',
        'Seed System Contract',
        'sha256:seed-system-contract',
        '{"seed": true}'::jsonb,
        NULL
    ),
    (
        '55555555-5555-4555-8555-555555555551',
        '44444444-4444-4444-8444-444444444444',
        'memory',
        'Seed Memory Timeline',
        'sha256:seed-memory-timeline',
        '{"seed": true}'::jsonb,
        NULL
    ),
    (
        '55555555-5555-4555-8555-555555555552',
        '44444444-4444-4444-8444-444444444444',
        'knowledge',
        'Seed Refund Policy Knowledge',
        'sha256:seed-refund-policy',
        '{"seed": true}'::jsonb,
        NULL
    ),
    (
        '55555555-5555-4555-8555-555555555553',
        '44444444-4444-4444-8444-444444444444',
        'mcp_server',
        'Seed MCP Search',
        'sha256:seed-mcp-search',
        '{"seed": true}'::jsonb,
        NULL
    ),
    (
        '55555555-5555-4555-8555-555555555554',
        '44444444-4444-4444-8444-444444444444',
        'model_configuration',
        'Seed Model Configuration',
        'sha256:seed-model-config',
        '{"seed": true}'::jsonb,
        NULL
    ),
    (
        '55555555-5555-4555-8555-999999999999',
        '44444444-4444-4444-8444-444444444444',
        'prompt',
        'Deleted Seed Prompt',
        'sha256:deleted-seed-prompt',
        '{"seed": true, "deleted": true}'::jsonb,
        now()
    )
ON CONFLICT (id) DO UPDATE
SET context_id = EXCLUDED.context_id,
    kind = EXCLUDED.kind,
    name = EXCLUDED.name,
    content_hash = EXCLUDED.content_hash,
    metadata = EXCLUDED.metadata,
    deleted_at = EXCLUDED.deleted_at;

INSERT INTO context_commits (id, context_id, branch_name, message, changes)
VALUES
    (
        '77777777-7777-4777-8777-777777777777',
        '44444444-4444-4444-8444-444444444444',
        'main',
        'Seed materialized graph snapshot',
        '[]'::jsonb
    ),
    (
        '77777777-7777-4777-8777-777777777778',
        '44444444-4444-4444-8444-444444444444',
        'main',
        'Seed commit without graph snapshot',
        '[]'::jsonb
    )
ON CONFLICT (id) DO UPDATE
SET context_id = EXCLUDED.context_id,
    branch_name = EXCLUDED.branch_name,
    message = EXCLUDED.message,
    changes = EXCLUDED.changes;

INSERT INTO context_commit_graph_snapshots (commit_id, schema_version, graph, captured_at)
VALUES
    (
        '77777777-7777-4777-8777-777777777777',
        1,
        '{"nodes":[{"id":"context:44444444-4444-4444-8444-444444444444","kind":"context","label":"Seed Support Resolution Context"}],"edges":[]}'::jsonb,
        now()
    )
ON CONFLICT (commit_id) DO UPDATE
SET schema_version = EXCLUDED.schema_version,
    graph = EXCLUDED.graph,
    captured_at = EXCLUDED.captured_at;

INSERT INTO evaluation_runs (
    id,
    context_id,
    suite_name,
    model_version,
    temperature,
    metrics,
    executed_at,
    deleted_at
)
VALUES
    (
        '66666666-6666-4666-8666-666666666666',
        '44444444-4444-4444-8444-444444444444',
        'Seed Safety Regression',
        'deepseek-chat',
        0.2,
        '{"accuracy": 0.92, "latency_ms": 820}'::jsonb,
        now(),
        NULL
    ),
    (
        '66666666-6666-4666-8666-999999999999',
        '44444444-4444-4444-8444-444444444444',
        'Deleted Seed Regression',
        'deepseek-chat',
        0.2,
        '{"accuracy": 0.1}'::jsonb,
        now(),
        now()
    )
ON CONFLICT (id) DO UPDATE
SET context_id = EXCLUDED.context_id,
    suite_name = EXCLUDED.suite_name,
    model_version = EXCLUDED.model_version,
    temperature = EXCLUDED.temperature,
    metrics = EXCLUDED.metrics,
    executed_at = EXCLUDED.executed_at,
    deleted_at = EXCLUDED.deleted_at;
