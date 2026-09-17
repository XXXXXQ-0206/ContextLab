//! SQLx/PostgreSQL repository implementation for Context Graph projections.

#[path = "postgres_workflow_execution_status.rs"]
mod postgres_workflow_execution_status;

use crate::{
    AuthorizationAuditPurgeExecutor, AuthorizationAuditPurgeRequest, AuthorizationAuditPurgeResult,
    AuthorizationAuditRetentionDisposition, AuthorizationAuditReviewCursor,
    AuthorizationAuditReviewItem, AuthorizationAuditReviewPage, AuthorizationAuditReviewQuery,
    AuthorizationAuditReviewRepository, BenchmarkComparability, BenchmarkDecisionComparisonScope,
    BenchmarkDecisionDatasetSummary, BenchmarkDecisionDiscoveryRepository,
    BenchmarkDecisionDiscoverySuiteSummary, BenchmarkDecisionDiscoverySummary,
    BenchmarkDecisionEvidence, BenchmarkDecisionId, BenchmarkDecisionPair,
    BenchmarkDefinitionBinding, BenchmarkDefinitionBindingCommand, BenchmarkDefinitionBindingId,
    BenchmarkDefinitionBindingRepository, BenchmarkDefinitionBindingWriteDisposition,
    BenchmarkDefinitionBindingWriteResult, BenchmarkDefinitionBindingWriter,
    BenchmarkEvidenceRepository, BenchmarkEvidenceWriteDisposition, BenchmarkEvidenceWriteResult,
    BenchmarkEvidenceWriter, BenchmarkMetricDecisionEvidence,
    BenchmarkWorkspaceProjectionDecisionQuery, BenchmarkWorkspaceProjectionPersistenceError,
    BenchmarkWorkspaceProjectionReceiptScope, BenchmarkWorkspaceProjectionV1Query,
    BenchmarkWorkspaceProjectionV1Reader, BenchmarkWorkspaceProjectionV1Writer,
    BenchmarkWorkspaceProjectionWriteDisposition, BenchmarkWorkspaceProjectionWriteResult,
    CommitDetail, CommitGraphSnapshot, CommitGraphSnapshotRepository, CommitList, CommitListItem,
    CommitListQuery, ComponentContentCreationWrite, ComponentContentMutationWrite,
    ComponentContentRevision, ComponentContentRevisionRepository, ComponentContentRevisionWrite,
    ComponentDescriptorRevisionWrite, ComponentDetail, ComponentList, ComponentListItem,
    ComponentListQuery, ComponentStateAtCommitRepository, ContextBranchHead,
    ContextBranchRepository, ContextBranchRepositoryError, ContextCommitGraphRepository,
    ContextCommitHistoryRepository, ContextCommitRecord, ContextCommitRepository,
    ContextCommitSnapshotWriter, ContextComponentRecord, ContextComponentRepository,
    ContextComponentStateSnapshotAtCommitRepository, ContextDiffSnapshotV1Pair,
    ContextDiffSnapshotV1PairRepository, ContextDiffSnapshotV1Repository,
    ContextDiffSnapshotWriteDisposition, ContextDiffSnapshotWriteResult,
    ContextGraphBranchHeadReviewWitness, ContextGraphBranchHeadReviewWitnessRepository,
    ContextGraphBranchHeadReviewWitnessRepositoryError, ContextGraphProjection,
    ContextGraphProjectionRepository, ContextGraphReviewWitness,
    ContextGraphReviewWitnessRepository, ContextLifecycleReadFacts, ContextLifecycleReadRepository,
    ContextLifecycleRoot, ContextLifecycleRootRepository, ContextList, ContextListItem,
    ContextListQuery, ContextMergeInputScope, ContextMergeReviewWitness,
    ContextMergeReviewWitnessError, ContextMergeReviewWitnessRepository,
    ContextMergeReviewWitnessRepositoryError, ContextMergeTipScope, ContextRecord,
    ContextReplayStateAtCommitRepository, ContextRepository, ContextWorkflowBindingRepository,
    CreateContextCommitSnapshot, EvaluationRunDetail, EvaluationRunList, EvaluationRunListItem,
    EvaluationRunListQuery, EvaluationRunRecord, EvaluationRunRepository, EvaluationScorecard,
    EvaluationScorecardMetric, EvaluationScorecardQuery, ExperimentList, ExperimentListItem,
    ExperimentListQuery, ExperimentRecord, ExperimentRepository, GraphProjectionScope,
    GuardedContextCommitWriter, IdempotencyKey, KnowledgeMemoryProjectionPersistenceError,
    KnowledgeMemoryProjectionScope, KnowledgeMemoryProjectionV1Repository,
    KnowledgeMemoryProjectionWriteDisposition, KnowledgeMemoryProjectionWriteResult,
    PersistBenchmarkEvaluationEvidence, PersistBenchmarkWorkspaceProjectionV1,
    PersistContextDiffSnapshotV1, PersistKnowledgeMemoryProjectionV1, ProjectList, ProjectListItem,
    ProjectListQuery, ProjectRecord, ProjectRepository, RequestDigest, StorageRepositoryError,
    WorkflowContextBindingWriteResult, WorkspaceList, WorkspaceListItem, WorkspaceListQuery,
    WorkspaceRecord, WorkspaceRepository, validate_component_content_attachment,
};
use async_trait::async_trait;
use chrono::{DateTime, Timelike, Utc};
use contextlab_auth::{
    AuthenticatedPrincipal, AuthorizationAuditError, AuthorizationAuditEvent,
    AuthorizationAuditSink, AuthorizationError, ContextPermission, ContextRoleResolver,
    GroupWorkspaceRole, WorkspaceRole, WorkspaceRoleAssignments, WorkspaceRoleResolver,
};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::{ContextDiffSnapshotV1, VersionedContextScopeV1};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkEvaluation, BenchmarkExecutionPlan, BenchmarkExecutionReceipt,
    BenchmarkExpectedOutput, BenchmarkSuite, BenchmarkSuiteId, BenchmarkWorkspaceProjectionV1,
    EvaluationRun, EvaluationRunId, MetricKind, MetricMeasurement, RegressionCheckStatus,
    RegressionDecisionStatus, RegressionThreshold, ThresholdDirection,
};
use contextlab_versioning::{
    BranchName, CommitGraphNode, CommitId, ContextChange, ContextCommit, MergePlan,
    normal_commit_parent,
};
use contextlab_workflow::{
    ContextCommitSource, WorkflowContextBinding, WorkflowContextBindingId, WorkflowId,
    WorkflowRevision,
};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use crate::commit_graph::{commit_graph_from_nodes, invalid_graph};
use crate::commit_graph_snapshot::{COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1, CommitGraphSnapshotScope};
use crate::component_state_at_commit::{
    ComponentStateReplayStep, component_state_conflict, replay_component_state,
    replay_context_component_state_snapshot,
};
use crate::context_commit_history::assemble_context_commit_history;
use crate::context_diff_snapshot::{
    CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1, ContextDiffSnapshotPersistenceError,
    context_diff_snapshot_database_error, decode_context_diff_snapshot, validate_scope,
};
use crate::replay_state_at_commit::replay_state_from_records;
use crate::{
    benchmark_evidence::BenchmarkDecisionDiscoverySummaryInput,
    component_content_revision::PersistedComponentContentRevision,
};

const WORKSPACES_BY_ID_SQL: &str = r#"
SELECT id, name, slug, created_at
FROM workspaces
WHERE id = $1 AND deleted_at IS NULL
ORDER BY created_at, id
"#;

const PROJECTS_BY_WORKSPACE_SQL: &str = r#"
SELECT id, workspace_id, name, slug, created_at
FROM projects
WHERE workspace_id = $1 AND deleted_at IS NULL
ORDER BY created_at, id
"#;

const EXPERIMENTS_BY_WORKSPACE_SQL: &str = r#"
SELECT experiments.id,
       experiments.project_id,
       experiments.name,
       experiments.branch_name,
       experiments.created_at
FROM experiments
JOIN projects ON projects.id = experiments.project_id
WHERE projects.workspace_id = $1
  AND projects.deleted_at IS NULL
  AND experiments.deleted_at IS NULL
ORDER BY experiments.created_at, experiments.id
"#;

const CONTEXTS_BY_WORKSPACE_SQL: &str = r#"
SELECT contexts.id,
       contexts.project_id,
       contexts.experiment_id,
       contexts.name,
       contexts.description,
       contexts.created_at
FROM contexts
JOIN projects ON projects.id = contexts.project_id
WHERE projects.workspace_id = $1
  AND projects.deleted_at IS NULL
  AND contexts.deleted_at IS NULL
ORDER BY contexts.created_at, contexts.id
"#;

const COMPONENTS_BY_WORKSPACE_SQL: &str = r#"
SELECT context_components.id,
       context_components.context_id,
       context_components.kind,
       context_components.name,
       context_components.content_hash,
       context_components.metadata,
       context_components.created_at,
       context_components.updated_at
FROM context_components
JOIN contexts ON contexts.id = context_components.context_id
JOIN projects ON projects.id = contexts.project_id
WHERE projects.workspace_id = $1
  AND projects.deleted_at IS NULL
  AND contexts.deleted_at IS NULL
  AND context_components.deleted_at IS NULL
ORDER BY context_components.created_at, context_components.id
"#;

const EVALUATION_RUNS_BY_WORKSPACE_SQL: &str = r#"
SELECT evaluation_runs.id,
       evaluation_runs.context_id,
       evaluation_runs.suite_name,
       evaluation_runs.model_version,
       evaluation_runs.temperature,
       CASE jsonb_typeof(evaluation_runs.metrics)
           WHEN 'object' THEN (
               SELECT COUNT(*)::INTEGER
               FROM jsonb_object_keys(evaluation_runs.metrics)
           )
           ELSE 0
       END AS metric_count,
       evaluation_runs.metrics,
       evaluation_runs.executed_at,
       evaluation_runs.created_at
FROM evaluation_runs
JOIN contexts ON contexts.id = evaluation_runs.context_id
JOIN projects ON projects.id = contexts.project_id
WHERE projects.workspace_id = $1
  AND projects.deleted_at IS NULL
  AND contexts.deleted_at IS NULL
  AND evaluation_runs.deleted_at IS NULL
ORDER BY evaluation_runs.executed_at, evaluation_runs.id
"#;

const WORKSPACE_COUNT_SQL: &str = r#"
SELECT COUNT(*)::BIGINT
FROM workspaces
WHERE deleted_at IS NULL
  AND (
      $1::TEXT IS NULL
      OR name ILIKE '%' || $1 || '%'
      OR slug ILIKE '%' || $1 || '%'
      OR id::TEXT ILIKE '%' || $1 || '%'
  )
"#;

const WORKSPACE_EXISTS_SQL: &str = r#"
SELECT EXISTS (
    SELECT 1
    FROM workspaces
    WHERE id = $1 AND deleted_at IS NULL
)
"#;

const CONTEXT_MEMBERSHIP_ROLE_SQL: &str = r#"
SELECT workspace_memberships.role
FROM contexts
JOIN projects ON projects.id = contexts.project_id
JOIN workspaces ON workspaces.id = projects.workspace_id
JOIN workspace_memberships
  ON workspace_memberships.workspace_id = projects.workspace_id
WHERE contexts.id = $1
  AND contexts.deleted_at IS NULL
  AND projects.deleted_at IS NULL
  AND workspaces.deleted_at IS NULL
  AND workspace_memberships.identity_source = $2
  AND workspace_memberships.principal_id = $3
"#;

const WORKSPACE_DIRECT_MEMBERSHIP_ROLE_SQL: &str = r#"
SELECT workspace_memberships.role
FROM workspaces
JOIN workspace_memberships
  ON workspace_memberships.workspace_id = workspaces.id
WHERE workspaces.id = $1
  AND workspaces.deleted_at IS NULL
  AND workspace_memberships.identity_source = $2
  AND workspace_memberships.principal_id = $3
"#;

const CONTEXT_EXTERNAL_GROUP_ROLE_SQL: &str = r#"
SELECT workspace_external_group_role_bindings.role
FROM contexts
JOIN projects ON projects.id = contexts.project_id
JOIN workspaces ON workspaces.id = projects.workspace_id
JOIN workspace_external_group_role_bindings
  ON workspace_external_group_role_bindings.workspace_id = projects.workspace_id
WHERE contexts.id = $1
  AND contexts.deleted_at IS NULL
  AND projects.deleted_at IS NULL
  AND workspaces.deleted_at IS NULL
  AND workspace_external_group_role_bindings.identity_source = $2
  AND workspace_external_group_role_bindings.external_group_id = ANY($3)
  AND workspace_external_group_role_bindings.deleted_at IS NULL
"#;

const CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL: &str = r#"
SELECT workspace_memberships.role
FROM contexts
JOIN projects ON projects.id = contexts.project_id
JOIN workspaces ON workspaces.id = projects.workspace_id
JOIN workspace_memberships
  ON workspace_memberships.workspace_id = projects.workspace_id
WHERE contexts.id = $1
  AND contexts.deleted_at IS NULL
  AND projects.deleted_at IS NULL
  AND workspaces.deleted_at IS NULL
  AND workspace_memberships.identity_source = $2
  AND workspace_memberships.principal_id = $3
FOR UPDATE OF contexts, projects, workspaces, workspace_memberships
"#;

const CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL: &str = r#"
SELECT workspace_external_group_role_bindings.role
FROM contexts
JOIN projects ON projects.id = contexts.project_id
JOIN workspaces ON workspaces.id = projects.workspace_id
JOIN workspace_external_group_role_bindings
  ON workspace_external_group_role_bindings.workspace_id = projects.workspace_id
WHERE contexts.id = $1
  AND contexts.deleted_at IS NULL
  AND projects.deleted_at IS NULL
  AND workspaces.deleted_at IS NULL
  AND workspace_external_group_role_bindings.identity_source = $2
  AND workspace_external_group_role_bindings.external_group_id = ANY($3)
  AND workspace_external_group_role_bindings.deleted_at IS NULL
FOR UPDATE OF contexts, projects, workspaces, workspace_external_group_role_bindings
"#;

const PROJECT_COUNT_BY_WORKSPACE_SQL: &str = r#"
SELECT COUNT(*)::BIGINT
FROM projects
WHERE workspace_id = $1
  AND deleted_at IS NULL
  AND (
      $2::TEXT IS NULL
      OR name ILIKE '%' || $2 || '%'
      OR slug ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
"#;

const PROJECT_EXISTS_SQL: &str = r#"
SELECT EXISTS (
    SELECT 1
    FROM projects
    WHERE id = $1 AND deleted_at IS NULL
)
"#;

const EXPERIMENT_COUNT_BY_PROJECT_SQL: &str = r#"
SELECT COUNT(*)::BIGINT
FROM experiments
WHERE project_id = $1
  AND deleted_at IS NULL
  AND (
      $2::TEXT IS NULL
      OR name ILIKE '%' || $2 || '%'
      OR branch_name ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
"#;

const CONTEXT_COUNT_BY_PROJECT_SQL: &str = r#"
SELECT COUNT(*)::BIGINT
FROM contexts
WHERE project_id = $1
  AND deleted_at IS NULL
  AND ($3::UUID IS NULL OR experiment_id = $3)
  AND (
      $2::TEXT IS NULL
      OR name ILIKE '%' || $2 || '%'
      OR description ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
"#;

const CONTEXT_EXISTS_SQL: &str = r#"
SELECT EXISTS (
    SELECT 1
    FROM contexts
    WHERE id = $1 AND deleted_at IS NULL
)
"#;

const COMPONENT_COUNT_BY_CONTEXT_SQL: &str = r#"
SELECT COUNT(*)::BIGINT
FROM context_components
WHERE context_id = $1
  AND deleted_at IS NULL
  AND ($3::TEXT IS NULL OR kind = $3)
  AND (
      $2::TEXT IS NULL
      OR name ILIKE '%' || $2 || '%'
      OR kind ILIKE '%' || $2 || '%'
      OR content_hash ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
"#;

const COMPONENT_BY_CONTEXT_SQL: &str = r#"
SELECT context_components.id,
       context_components.context_id,
       context_components.kind,
       context_components.name,
       context_components.content_hash,
       context_components.metadata,
       context_components.created_at,
       context_components.updated_at
FROM context_components
WHERE context_components.context_id = $1
  AND context_components.id = $2
  AND context_components.deleted_at IS NULL
"#;

const COMMIT_COUNT_BY_CONTEXT_SQL: &str = r#"
SELECT COUNT(*)::BIGINT
FROM context_commits
WHERE context_id = $1
  AND ($3::TEXT IS NULL OR branch_name = $3)
  AND (
      $2::TEXT IS NULL
      OR message ILIKE '%' || $2 || '%'
      OR branch_name ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
)
"#;

const COMMIT_GRAPH_BY_CONTEXT_SQL: &str = r#"
SELECT context_commits.id,
       context_commits.context_id,
       COALESCE(
           ARRAY_AGG(
               context_commit_parents.parent_commit_id
               ORDER BY context_commit_parents.position
           ) FILTER (WHERE context_commit_parents.parent_commit_id IS NOT NULL),
           ARRAY[]::UUID[]
       ) AS parent_commit_ids
FROM context_commits
LEFT JOIN context_commit_parents
  ON context_commit_parents.commit_id = context_commits.id
WHERE context_commits.context_id = $1
GROUP BY context_commits.id, context_commits.context_id
ORDER BY context_commits.id
"#;

const COMMIT_BY_CONTEXT_SQL: &str = r#"
SELECT context_commits.id,
       context_commits.context_id,
       context_commits.branch_name,
       context_commits.message,
       COALESCE(
           ARRAY_AGG(
               context_commit_parents.parent_commit_id
               ORDER BY context_commit_parents.position
           ) FILTER (WHERE context_commit_parents.parent_commit_id IS NOT NULL),
           ARRAY[]::UUID[]
       ) AS parent_commit_ids,
       context_commits.changes,
       jsonb_array_length(context_commits.changes)::INTEGER AS change_count,
       context_commits.authored_at,
       context_commits.created_at
FROM context_commits
LEFT JOIN context_commit_parents
  ON context_commit_parents.commit_id = context_commits.id
WHERE context_commits.context_id = $1
  AND context_commits.id = $2
GROUP BY context_commits.id

"#;

const CONTEXT_COMMIT_HISTORY_ROWS_SQL: &str = r#"
SELECT context_commits.id,
       context_commits.context_id,
       context_commits.branch_name,
       context_commits.message,
       COALESCE(
           ARRAY_AGG(
               context_commit_parents.parent_commit_id
               ORDER BY context_commit_parents.position
           ) FILTER (WHERE context_commit_parents.parent_commit_id IS NOT NULL),
           ARRAY[]::UUID[]
       ) AS parent_commit_ids,
       context_commits.changes,
       jsonb_array_length(context_commits.changes)::INTEGER AS change_count,
       context_commits.authored_at,
       context_commits.created_at
FROM context_commits
LEFT JOIN context_commit_parents
  ON context_commit_parents.commit_id = context_commits.id
WHERE context_commits.context_id = $1
GROUP BY context_commits.id
ORDER BY context_commits.authored_at ASC, context_commits.id ASC
"#;

const CONTEXT_PROJECT_BY_ID_SQL: &str = r#"
SELECT project_id
FROM contexts
WHERE id = $1
  AND deleted_at IS NULL
"#;

const CONTEXT_PROJECT_FOR_COMMIT_SCOPE_SQL: &str = r#"
SELECT contexts.project_id
FROM contexts
JOIN context_commits
  ON context_commits.context_id = contexts.id
WHERE contexts.id = $1
  AND context_commits.id = $2
  AND contexts.deleted_at IS NULL
"#;

const COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL: &str = r#"
SELECT context_commit_graph_snapshots.schema_version,
       context_commit_graph_snapshots.graph,
       context_commit_graph_snapshots.captured_at
FROM contexts
JOIN context_commits
  ON context_commits.context_id = contexts.id
LEFT JOIN context_commit_graph_snapshots
  ON context_commit_graph_snapshots.commit_id = context_commits.id
WHERE contexts.project_id = $1
  AND contexts.id = $2
  AND context_commits.id = $3
  AND contexts.deleted_at IS NULL
"#;

const CONTEXT_FOR_COMMIT_SNAPSHOT_WRITE_SQL: &str = r#"
SELECT id, project_id
FROM contexts
WHERE id = $1
  AND deleted_at IS NULL
FOR UPDATE
"#;

const CONTEXT_LIFECYCLE_ROOT_SQL: &str = r#"
SELECT project_id, name
FROM contexts
WHERE id = $1
  AND deleted_at IS NULL
"#;

const PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL: &str = r#"
SELECT context_commits.id
FROM context_commits
JOIN context_commit_graph_snapshots
  ON context_commit_graph_snapshots.commit_id = context_commits.id
WHERE context_commits.context_id = $1
  AND context_commits.id = ANY($2)
FOR KEY SHARE OF context_commits
"#;

const INSERT_CONTEXT_COMMIT_SQL: &str = r#"
INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO NOTHING
RETURNING id
"#;

const INSERT_CONTEXT_COMMIT_PARENT_SQL: &str = r#"
INSERT INTO context_commit_parents (context_id, commit_id, parent_commit_id, position)
VALUES ($1, $2, $3, $4)
"#;

const INSERT_CONTEXT_COMMIT_GRAPH_SNAPSHOT_SQL: &str = r#"
INSERT INTO context_commit_graph_snapshots (commit_id, schema_version, graph, captured_at)
VALUES ($1, $2, $3, $4)
"#;

const MATERIALIZED_CONTEXT_COMMIT_FOR_WORKFLOW_BINDING_SQL: &str = r#"
SELECT context_commits.id
FROM context_commits
JOIN contexts ON contexts.id = context_commits.context_id
JOIN context_commit_graph_snapshots
  ON context_commit_graph_snapshots.commit_id = context_commits.id
WHERE context_commits.context_id = $1
  AND context_commits.id = $2
  AND contexts.deleted_at IS NULL
FOR SHARE OF context_commits, contexts, context_commit_graph_snapshots
"#;

const WORKFLOW_CONTEXT_BINDING_ADVISORY_LOCK_SQL: &str =
    "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))";

const WORKFLOW_CONTEXT_BINDING_BY_WORKFLOW_REVISION_SQL: &str = r#"
SELECT id, context_id, context_commit_id, workflow_id, workflow_revision, binding
FROM context_workflow_bindings
WHERE workflow_id = $1
  AND workflow_revision = $2
FOR UPDATE
"#;

const INSERT_WORKFLOW_CONTEXT_BINDING_SQL: &str = r#"
INSERT INTO context_workflow_bindings (
    id,
    context_id,
    context_commit_id,
    workflow_id,
    workflow_revision,
    binding,
    created_at
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
"#;

const WORKFLOW_CONTEXT_BINDINGS_AT_COMMIT_SQL: &str = r#"
SELECT id, context_id, context_commit_id, workflow_id, workflow_revision, binding
FROM context_workflow_bindings
WHERE context_id = $1
  AND context_commit_id = $2
ORDER BY workflow_id ASC, workflow_revision ASC, id ASC
"#;

const INSERT_CONTEXT_AUTHORIZATION_AUDIT_EVENT_SQL: &str = r#"
INSERT INTO context_authorization_audit_events (
    identity_source, principal_id, context_id, permission, decision
) VALUES ($1, $2, $3, $4, $5)
"#;

const AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL: &str = r#"
SELECT context_authorization_audit_events.id,
       context_authorization_audit_events.recorded_at,
       context_authorization_audit_events.context_id,
       context_authorization_audit_events.permission,
       context_authorization_audit_events.decision,
       context_authorization_audit_events.retention_disposition,
       context_authorization_audit_events.retention_policy_revision_id
FROM context_authorization_audit_events
JOIN contexts ON contexts.id = context_authorization_audit_events.context_id
JOIN projects ON projects.id = contexts.project_id
JOIN workspaces ON workspaces.id = projects.workspace_id
WHERE workspaces.id = $1
  AND workspaces.deleted_at IS NULL
  AND projects.deleted_at IS NULL
  AND contexts.deleted_at IS NULL
  AND (
      $2::TIMESTAMPTZ IS NULL
      OR (context_authorization_audit_events.recorded_at, context_authorization_audit_events.id)
          < ($2, $3::UUID)
  )
ORDER BY context_authorization_audit_events.recorded_at DESC,
         context_authorization_audit_events.id DESC
LIMIT $4
"#;

const INSERT_CONTEXT_BRANCH_SQL: &str = r#"
INSERT INTO context_branches (context_id, branch_name)
VALUES ($1, $2)
ON CONFLICT (context_id, branch_name) DO NOTHING
"#;

const CONTEXT_BRANCH_FOR_UPDATE_SQL: &str = r#"
SELECT head_commit_id
FROM context_branches
WHERE context_id = $1 AND branch_name = $2
FOR UPDATE
"#;

const CONTEXT_EXISTS_FOR_BRANCH_HEAD_SQL: &str = r#"
SELECT EXISTS (
    SELECT 1
    FROM contexts
    WHERE id = $1 AND deleted_at IS NULL
)
"#;

const CONTEXT_BRANCH_HEADS_SQL: &str = r#"
SELECT branches.branch_name,
       branches.head_commit_id,
       branches.revision,
       (
           branches.head_commit_id IS NULL
           OR EXISTS (
               SELECT 1
               FROM context_commits AS commits
               WHERE commits.context_id = branches.context_id
                 AND commits.id = branches.head_commit_id
           )
       ) AS head_belongs_to_context
FROM context_branches AS branches
WHERE branches.context_id = $1
ORDER BY branches.branch_name ASC
"#;

const CONTEXT_BRANCH_HEAD_SQL: &str = r#"
SELECT branches.branch_name,
       branches.head_commit_id,
       branches.revision,
       (
           branches.head_commit_id IS NULL
           OR EXISTS (
               SELECT 1
               FROM context_commits AS commits
               WHERE commits.context_id = branches.context_id
                 AND commits.id = branches.head_commit_id
           )
       ) AS head_belongs_to_context
FROM context_branches AS branches
WHERE branches.context_id = $1
  AND branches.branch_name = $2
"#;

const CONTEXT_IDEMPOTENCY_SQL: &str = r#"
SELECT receipt.request_digest, receipt.commit_id
FROM context_commit_idempotency AS receipt
JOIN context_commits AS commit
  ON commit.context_id = receipt.context_id
 AND commit.id = receipt.commit_id
WHERE receipt.identity_source = $1
  AND receipt.principal_id = $2
  AND receipt.context_id = $3
  AND receipt.branch_name = $4
  AND commit.branch_name = $4
  AND receipt.idempotency_key = $5
"#;

const GUARDED_IDEMPOTENCY_ADVISORY_LOCK_SQL: &str =
    "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))";

const INSERT_CONTEXT_IDEMPOTENCY_SQL: &str = r#"
INSERT INTO context_commit_idempotency (
    identity_source,
    principal_id,
    context_id,
    branch_name,
    idempotency_key,
    request_digest,
    commit_id
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
"#;

const UPDATE_CONTEXT_BRANCH_HEAD_SQL: &str = r#"
UPDATE context_branches
SET head_commit_id = $1,
    revision = revision + 1,
    updated_at = now()
WHERE context_id = $2 AND branch_name = $3
"#;

const COMPONENT_FOR_CONTENT_REVISION_UPDATE_SQL: &str = r#"
SELECT kind, content_hash
FROM context_components
WHERE context_id = $1
  AND id = $2
  AND deleted_at IS NULL
FOR UPDATE
"#;

const UPDATE_COMPONENT_DESCRIPTOR_SQL: &str = r#"
UPDATE context_components
SET name = $1,
    metadata = $2,
    updated_at = $3
WHERE context_id = $4
  AND id = $5
  AND deleted_at IS NULL
"#;

const GUARDED_COMPONENT_CREATION_ADVISORY_LOCK_SQL: &str = r#"
SELECT pg_advisory_xact_lock(hashtextextended($1, 0))
"#;

const COMPONENT_FOR_CONTENT_CREATION_SQL: &str = r#"
SELECT id
FROM context_components
WHERE id = $1
FOR UPDATE
"#;

const INSERT_CONTEXT_COMPONENT_SQL: &str = r#"
INSERT INTO context_components (
    id, context_id, kind, name, content_hash, metadata, created_at, updated_at
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
"#;

const INSERT_COMPONENT_CONTENT_REVISION_SQL: &str = r#"
INSERT INTO context_component_content_revisions (
    context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
"#;

const UPDATE_COMPONENT_CONTENT_HASH_SQL: &str = r#"
UPDATE context_components
SET content_hash = $1,
    updated_at = now()
WHERE context_id = $2
  AND id = $3
"#;

const SOFT_REMOVE_COMPONENT_SQL: &str = r#"
UPDATE context_components
SET deleted_at = now(),
    updated_at = now()
WHERE context_id = $1
  AND id = $2
  AND deleted_at IS NULL
"#;

const COMPONENT_CONTENT_REVISION_BY_ID_SQL: &str = r#"
SELECT context_component_content_revisions.previous_content_hash,
       context_component_content_revisions.content_hash,
       context_component_content_revisions.content,
       context_component_content_revisions.created_at,
       context_components.kind
FROM context_component_content_revisions
JOIN context_components
  ON context_components.id = context_component_content_revisions.component_id
WHERE context_component_content_revisions.context_id = $1
  AND context_component_content_revisions.commit_id = $2
  AND context_component_content_revisions.component_id = $3
"#;

const NORMAL_FIRST_PARENT_HISTORY_CTE: &str = r#"
WITH RECURSIVE normal_history AS (
    SELECT context_commits.id,
           context_commits.context_id,
           0::BIGINT AS depth,
           (
               SELECT COUNT(*)
               FROM context_commit_parents
               WHERE context_commit_parents.commit_id = context_commits.id
           ) AS parent_count,
           ARRAY[context_commits.id] AS ancestry_path,
           FALSE AS cycle_detected,
           FALSE AS invalid_parent
    FROM context_commits
    WHERE context_commits.context_id = $1
      AND context_commits.id = $2

    UNION ALL

    SELECT parent_commit.id,
           normal_history.context_id,
           normal_history.depth + 1,
           (
               SELECT COUNT(*)
               FROM context_commit_parents
               WHERE context_commit_parents.commit_id = parent.parent_commit_id
           ) AS parent_count,
           normal_history.ancestry_path || parent.parent_commit_id,
           parent.parent_commit_id = ANY(normal_history.ancestry_path),
           parent_commit.context_id <> normal_history.context_id
    FROM normal_history
    JOIN context_commit_parents AS parent
      ON parent.commit_id = normal_history.id
     AND parent.position = 0
    JOIN context_commits AS parent_commit
      ON parent_commit.id = parent.parent_commit_id
    WHERE normal_history.parent_count <= 1
      AND NOT normal_history.cycle_detected
      AND NOT normal_history.invalid_parent
)
"#;

const COMPONENT_CONTENT_AT_COMMIT_SQL_SUFFIX: &str = r#"
SELECT normal_history.id,
       normal_history.depth,
       normal_history.parent_count,
       normal_history.cycle_detected,
       normal_history.invalid_parent,
       revisions.previous_content_hash,
       revisions.content_hash,
       revisions.content,
       revisions.created_at,
       components.kind
FROM normal_history
LEFT JOIN context_component_content_revisions AS revisions
  ON revisions.context_id = normal_history.context_id
 AND revisions.commit_id = normal_history.id
 AND revisions.component_id = $3
LEFT JOIN context_components AS components
  ON components.context_id = revisions.context_id
 AND components.id = revisions.component_id
ORDER BY normal_history.depth
"#;

const COMPONENT_STATE_AT_COMMIT_SQL_SUFFIX: &str = r#"
SELECT normal_history.id,
       normal_history.depth,
       normal_history.parent_count,
       normal_history.cycle_detected,
       normal_history.invalid_parent,
       context_commits.changes,
       revisions.previous_content_hash,
       revisions.content_hash,
       revisions.content,
       revisions.created_at,
       components.kind
FROM normal_history
JOIN context_commits
  ON context_commits.id = normal_history.id
LEFT JOIN context_component_content_revisions AS revisions
  ON revisions.context_id = normal_history.context_id
 AND revisions.commit_id = normal_history.id
 AND revisions.component_id = $3
LEFT JOIN context_components AS components
  ON components.context_id = revisions.context_id
 AND components.id = revisions.component_id
ORDER BY normal_history.depth
"#;

const COMPONENT_STATE_SNAPSHOT_HISTORY_SQL_SUFFIX: &str = r#"
SELECT normal_history.id,
       normal_history.depth,
       normal_history.parent_count,
       normal_history.cycle_detected,
       normal_history.invalid_parent,
       context_commits.changes
FROM normal_history
JOIN context_commits
  ON context_commits.id = normal_history.id
ORDER BY normal_history.depth
"#;

const REPLAY_STATE_HISTORY_SQL_SUFFIX: &str = r#"
SELECT normal_history.id,
       normal_history.context_id,
       normal_history.depth,
       normal_history.parent_count,
       normal_history.cycle_detected,
       normal_history.invalid_parent,
       context_commits.branch_name,
       context_commits.message,
       COALESCE(
           ARRAY_AGG(
               context_commit_parents.parent_commit_id
               ORDER BY context_commit_parents.position
           ) FILTER (WHERE context_commit_parents.parent_commit_id IS NOT NULL),
           ARRAY[]::UUID[]
       ) AS parent_commit_ids,
       context_commits.changes,
       context_commits.authored_at,
       context_commits.created_at
FROM normal_history
JOIN context_commits
  ON context_commits.id = normal_history.id
LEFT JOIN context_commit_parents
  ON context_commit_parents.commit_id = context_commits.id
GROUP BY normal_history.id,
         normal_history.context_id,
         normal_history.depth,
         normal_history.parent_count,
         normal_history.cycle_detected,
         normal_history.invalid_parent,
         context_commits.branch_name,
         context_commits.message,
         context_commits.changes,
         context_commits.authored_at,
         context_commits.created_at
ORDER BY normal_history.depth DESC, normal_history.id
"#;

const COMPONENT_CONTENT_REVISIONS_FOR_COMMITS_SQL: &str = r#"
SELECT revisions.commit_id,
       revisions.component_id,
       revisions.previous_content_hash,
       revisions.content_hash,
       revisions.content,
       revisions.created_at,
       components.kind
FROM context_component_content_revisions AS revisions
JOIN context_components AS components
  ON components.context_id = revisions.context_id
 AND components.id = revisions.component_id
WHERE revisions.context_id = $1
  AND revisions.commit_id = ANY($2)
ORDER BY revisions.commit_id, revisions.component_id
"#;

fn component_content_at_commit_sql() -> String {
    format!("{NORMAL_FIRST_PARENT_HISTORY_CTE}{COMPONENT_CONTENT_AT_COMMIT_SQL_SUFFIX}")
}

fn component_state_at_commit_sql() -> String {
    format!("{NORMAL_FIRST_PARENT_HISTORY_CTE}{COMPONENT_STATE_AT_COMMIT_SQL_SUFFIX}")
}

fn component_state_snapshot_history_sql() -> String {
    format!("{NORMAL_FIRST_PARENT_HISTORY_CTE}{COMPONENT_STATE_SNAPSHOT_HISTORY_SQL_SUFFIX}")
}

fn replay_state_history_sql() -> String {
    format!("{NORMAL_FIRST_PARENT_HISTORY_CTE}{REPLAY_STATE_HISTORY_SQL_SUFFIX}")
}

type ReplayStateHistoryRow = (
    Uuid,
    Uuid,
    i64,
    i64,
    bool,
    bool,
    String,
    String,
    Vec<Uuid>,
    Value,
    DateTime<Utc>,
    DateTime<Utc>,
);

const EVALUATION_RUN_COUNT_BY_CONTEXT_SQL: &str = r#"
SELECT COUNT(*)::BIGINT
FROM evaluation_runs
WHERE context_id = $1
  AND deleted_at IS NULL
  AND ($3::TEXT IS NULL OR suite_name = $3)
  AND ($4::TEXT IS NULL OR model_version = $4)
  AND (
      $2::TEXT IS NULL
      OR suite_name ILIKE '%' || $2 || '%'
      OR model_version ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
"#;

const EVALUATION_RUN_LIST_ITEMS_SQL_TEMPLATE: &str = r#"
SELECT evaluation_runs.id,
       evaluation_runs.context_id,
       evaluation_runs.suite_name,
       evaluation_runs.model_version,
       evaluation_runs.temperature,
       CASE jsonb_typeof(evaluation_runs.metrics)
           WHEN 'object' THEN (
               SELECT COUNT(*)::INTEGER
               FROM jsonb_object_keys(evaluation_runs.metrics)
           )
           ELSE 0
       END AS metric_count,
       evaluation_runs.executed_at,
       evaluation_runs.created_at
FROM evaluation_runs
WHERE evaluation_runs.context_id = $1
  AND evaluation_runs.deleted_at IS NULL
  AND ($3::TEXT IS NULL OR evaluation_runs.suite_name = $3)
  AND ($4::TEXT IS NULL OR evaluation_runs.model_version = $4)
  AND (
      $2::TEXT IS NULL
      OR evaluation_runs.suite_name ILIKE '%' || $2 || '%'
      OR evaluation_runs.model_version ILIKE '%' || $2 || '%'
      OR evaluation_runs.id::TEXT ILIKE '%' || $2 || '%'
  )
ORDER BY {order_by}
LIMIT $5 OFFSET $6
"#;

const EVALUATION_RUN_BY_CONTEXT_SQL: &str = r#"
SELECT evaluation_runs.id,
       evaluation_runs.context_id,
       evaluation_runs.suite_name,
       evaluation_runs.model_version,
       evaluation_runs.temperature,
       CASE jsonb_typeof(evaluation_runs.metrics)
           WHEN 'object' THEN (
               SELECT COUNT(*)::INTEGER
               FROM jsonb_object_keys(evaluation_runs.metrics)
           )
           ELSE 0
       END AS metric_count,
       evaluation_runs.metrics,
       evaluation_runs.executed_at,
       evaluation_runs.created_at
FROM evaluation_runs
WHERE evaluation_runs.context_id = $1
  AND evaluation_runs.id = $2
  AND evaluation_runs.deleted_at IS NULL
"#;

const EVALUATION_SCORECARD_RUNS_SQL: &str = r#"
SELECT evaluation_runs.metrics
FROM evaluation_runs
WHERE evaluation_runs.context_id = $1
  AND evaluation_runs.deleted_at IS NULL
  AND ($3::TEXT IS NULL OR evaluation_runs.suite_name = $3)
  AND ($4::TEXT IS NULL OR evaluation_runs.model_version = $4)
  AND (
      $2::TEXT IS NULL
      OR evaluation_runs.suite_name ILIKE '%' || $2 || '%'
      OR evaluation_runs.model_version ILIKE '%' || $2 || '%'
      OR evaluation_runs.id::TEXT ILIKE '%' || $2 || '%'
  )
ORDER BY evaluation_runs.executed_at DESC, evaluation_runs.id ASC
"#;

const PURGE_CONTEXT_AUTHORIZATION_AUDIT_EVENTS_SQL: &str = r#"
SELECT manifest_id, purged_event_count
FROM public.purge_context_authorization_audit_events($1, $2, $3, $4)
"#;

/// PostgreSQL adapter for a separately authenticated private audit purge executor.
#[derive(Debug, Clone)]
pub struct PostgresAuthorizationAuditPurgeExecutor {
    pool: PgPool,
}

impl PostgresAuthorizationAuditPurgeExecutor {
    /// Creates a private executor from a pool authenticated as the purge role.
    #[must_use]
    pub const fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorizationAuditPurgeExecutor for PostgresAuthorizationAuditPurgeExecutor {
    async fn purge_authorization_audit_events(
        &self,
        request: AuthorizationAuditPurgeRequest,
    ) -> Result<AuthorizationAuditPurgeResult, StorageRepositoryError> {
        let (manifest_id, purged_event_count) =
            sqlx::query_as::<_, (Option<Uuid>, i64)>(PURGE_CONTEXT_AUTHORIZATION_AUDIT_EVENTS_SQL)
                .bind(request.workspace_id().as_uuid())
                .bind(request.policy_revision_id())
                .bind(request.cutoff())
                .bind(i32::try_from(request.batch_limit()).unwrap_or(i32::MAX))
                .fetch_one(&self.pool)
                .await
                .map_err(database_error)?;

        Ok(AuthorizationAuditPurgeResult::new(
            manifest_id,
            u64::try_from(purged_event_count).unwrap_or(0),
        ))
    }
}

/// SQLx-backed repository for PostgreSQL Context Graph projection reads.
#[derive(Debug, Clone)]
pub struct PostgresContextGraphRepository {
    pool: PgPool,
}

impl PostgresContextGraphRepository {
    /// Creates a repository from an existing PostgreSQL pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a repository from a PostgreSQL connection string without connecting immediately.
    ///
    /// This keeps API startup and unit tests independent from a live database while still
    /// validating the connection string shape.
    pub fn connect_lazy(database_url: &str) -> Result<Self, StorageRepositoryError> {
        let pool = PgPoolOptions::new()
            .connect_lazy(database_url)
            .map_err(database_configuration_error)?;

        Ok(Self::new(pool))
    }

    /// Returns the underlying PostgreSQL pool.
    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn load_workspace_projection(
        &self,
        workspace_id: String,
    ) -> Result<ContextGraphProjection, StorageRepositoryError> {
        let workspace_id = parse_workspace_id(&workspace_id)?;
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let workspaces =
            sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(WORKSPACES_BY_ID_SQL)
                .bind(workspace_id)
                .fetch_all(&mut *transaction)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|(id, name, slug, created_at)| WorkspaceRecord {
                    id: id.to_string(),
                    name,
                    slug,
                    created_at,
                })
                .collect::<Vec<_>>();

        if workspaces.is_empty() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("workspace:{workspace_id}"),
            });
        }

        let projects = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>)>(
            PROJECTS_BY_WORKSPACE_SQL,
        )
        .bind(workspace_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(|(id, workspace_id, name, slug, created_at)| ProjectRecord {
            id: id.to_string(),
            workspace_id: workspace_id.to_string(),
            name,
            slug,
            created_at,
        })
        .collect();

        let experiments = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>)>(
            EXPERIMENTS_BY_WORKSPACE_SQL,
        )
        .bind(workspace_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(id, project_id, name, branch_name, created_at)| ExperimentRecord {
                id: id.to_string(),
                project_id: project_id.to_string(),
                name,
                branch_name,
                created_at,
            },
        )
        .collect();

        let contexts = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<Uuid>,
                String,
                Option<String>,
                DateTime<Utc>,
            ),
        >(CONTEXTS_BY_WORKSPACE_SQL)
        .bind(workspace_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(id, project_id, experiment_id, name, description, created_at)| ContextRecord {
                id: id.to_string(),
                project_id: project_id.to_string(),
                experiment_id: experiment_id.map(|id| id.to_string()),
                name,
                description,
                created_at,
            },
        )
        .collect();

        let components = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                String,
                Value,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(COMPONENTS_BY_WORKSPACE_SQL)
        .bind(workspace_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(id, context_id, kind, name, content_hash, metadata, created_at, updated_at)| {
                let parsed_kind = kind.parse().map_err(|_| {
                    StorageRepositoryError::InvalidStoredComponentKind { kind: kind.clone() }
                })?;

                Ok(ContextComponentRecord {
                    id: id.to_string(),
                    context_id: context_id.to_string(),
                    kind: parsed_kind,
                    name,
                    content_hash,
                    metadata,
                    created_at,
                    updated_at,
                })
            },
        )
        .collect::<Result<Vec<_>, StorageRepositoryError>>()?;

        let evaluation_runs = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                f32,
                i32,
                Value,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(EVALUATION_RUNS_BY_WORKSPACE_SQL)
        .bind(workspace_id)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(
                id,
                context_id,
                suite_name,
                model_version,
                temperature,
                metric_count,
                metrics,
                executed_at,
                created_at,
            )| EvaluationRunRecord {
                id: id.to_string(),
                context_id: context_id.to_string(),
                suite_name,
                model_version,
                temperature,
                metric_count: u32::try_from(metric_count).unwrap_or(0),
                metrics,
                executed_at,
                created_at,
            },
        )
        .collect();

        let projection = ContextGraphProjection {
            workspaces,
            projects,
            experiments,
            contexts,
            commits: Vec::new(),
            components,
            evaluation_runs,
        };

        transaction.commit().await.map_err(database_error)?;

        Ok(projection)
    }
}

#[async_trait]
impl ContextGraphProjectionRepository for PostgresContextGraphRepository {
    async fn load_context_graph_projection(
        &self,
        scope: GraphProjectionScope,
    ) -> Result<ContextGraphProjection, StorageRepositoryError> {
        match scope {
            GraphProjectionScope::Workspace { workspace_id } => {
                self.load_workspace_projection(workspace_id).await
            }
            GraphProjectionScope::Preview => Err(StorageRepositoryError::ScopeUnavailable {
                scope: "preview".to_owned(),
            }),
        }
    }
}

#[async_trait]
impl KnowledgeMemoryProjectionV1Repository for PostgresContextGraphRepository {
    async fn persist_knowledge_memory_projection(
        &self,
        command: PersistKnowledgeMemoryProjectionV1,
    ) -> Result<KnowledgeMemoryProjectionWriteResult, KnowledgeMemoryProjectionPersistenceError>
    {
        let scope = command.scope();
        let project_id = scope.project_id().as_uuid();
        let context_id = scope.context_id().as_uuid();
        let context_commit_id = scope.context_commit_id().as_uuid();
        let schema_version = command.projection().schema_version().as_str();
        let projection = serde_json::to_value(command.projection())
            .map_err(|_| KnowledgeMemoryProjectionPersistenceError::StoredProjectionInvalid)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(knowledge_memory_database_error)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!(
                "knowledge-memory-projection:{project_id}:{context_id}:{context_commit_id}"
            ))
            .execute(&mut *transaction)
            .await
            .map_err(knowledge_memory_database_error)?;

        let existing = sqlx::query_as::<_, (String, Value)>(
            "SELECT schema_version, projection FROM knowledge_memory_context_projections WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 FOR UPDATE",
        )
        .bind(project_id)
        .bind(context_id)
        .bind(context_commit_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(knowledge_memory_database_error)?;

        if let Some((existing_schema, existing_projection)) = existing {
            let stored =
                decode_knowledge_memory_projection(scope, existing_schema, existing_projection)?;
            if stored != *command.projection() {
                return Err(KnowledgeMemoryProjectionPersistenceError::Conflict { scope });
            }
            transaction
                .commit()
                .await
                .map_err(knowledge_memory_database_error)?;
            return Ok(KnowledgeMemoryProjectionWriteResult::new(
                scope,
                KnowledgeMemoryProjectionWriteDisposition::Replayed,
            ));
        }

        sqlx::query(
            "INSERT INTO knowledge_memory_context_projections (project_id, context_id, context_commit_id, schema_version, projection) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(project_id)
        .bind(context_id)
        .bind(context_commit_id)
        .bind(schema_version)
        .bind(projection)
        .execute(&mut *transaction)
        .await
        .map_err(knowledge_memory_database_error)?;
        let (stored_schema, stored_projection) = sqlx::query_as::<_, (String, Value)>(
            "SELECT schema_version, projection FROM knowledge_memory_context_projections WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3",
        )
        .bind(project_id)
        .bind(context_id)
        .bind(context_commit_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(knowledge_memory_database_error)?;
        let stored = decode_knowledge_memory_projection(scope, stored_schema, stored_projection)?;
        if stored != *command.projection() {
            return Err(KnowledgeMemoryProjectionPersistenceError::StoredProjectionInvalid);
        }
        transaction
            .commit()
            .await
            .map_err(knowledge_memory_database_error)?;
        Ok(KnowledgeMemoryProjectionWriteResult::new(
            scope,
            KnowledgeMemoryProjectionWriteDisposition::Created,
        ))
    }

    async fn read_knowledge_memory_projection(
        &self,
        scope: KnowledgeMemoryProjectionScope,
    ) -> Result<
        contextlab_knowledge::KnowledgeMemoryContextProjectionV1,
        KnowledgeMemoryProjectionPersistenceError,
    > {
        let mut transaction = begin_consistent_read_transaction(&self.pool)
            .await
            .map_err(|_| KnowledgeMemoryProjectionPersistenceError::RepositoryUnavailable)?;
        let row = sqlx::query_as::<_, (String, Value)>(
            "SELECT schema_version, projection FROM knowledge_memory_context_projections WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3",
        )
        .bind(scope.project_id().as_uuid())
        .bind(scope.context_id().as_uuid())
        .bind(scope.context_commit_id().as_uuid())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(knowledge_memory_database_error)?
        .ok_or(KnowledgeMemoryProjectionPersistenceError::NotFound { scope })?;
        let projection = decode_knowledge_memory_projection(scope, row.0, row.1)?;
        transaction
            .commit()
            .await
            .map_err(knowledge_memory_database_error)?;
        Ok(projection)
    }
}

#[async_trait]
impl ContextDiffSnapshotV1Repository for PostgresContextGraphRepository {
    async fn persist_context_diff_snapshot(
        &self,
        command: PersistContextDiffSnapshotV1,
    ) -> Result<ContextDiffSnapshotWriteResult, ContextDiffSnapshotPersistenceError> {
        let record = command.record().clone();
        let scope = record.scope();
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(context_diff_snapshot_database_error)?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!(
                "context-diff-snapshot:{}:{}:{}:{}",
                scope.project_id(),
                scope.context_id(),
                scope.commit_id(),
                record.schema_version()
            ))
            .execute(&mut *transaction)
            .await
            .map_err(context_diff_snapshot_database_error)?;

        let existing = sqlx::query_as::<_, (String, Value, String, DateTime<Utc>)>(
            "SELECT schema_version, snapshot, snapshot_digest, captured_at FROM context_diff_snapshots WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND schema_version = $4 FOR UPDATE",
        )
        .bind(scope.project_id().as_uuid())
        .bind(scope.context_id().as_uuid())
        .bind(scope.commit_id().as_uuid())
        .bind(record.schema_version())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(context_diff_snapshot_database_error)?;

        if let Some((schema_version, snapshot, snapshot_digest, captured_at)) = existing {
            let stored = decode_context_diff_snapshot(
                scope,
                schema_version,
                snapshot,
                snapshot_digest,
                captured_at,
            )?;
            if stored != record {
                return Err(ContextDiffSnapshotPersistenceError::Conflict { scope });
            }
            transaction
                .commit()
                .await
                .map_err(context_diff_snapshot_database_error)?;
            return Ok(ContextDiffSnapshotWriteResult::new(
                scope,
                ContextDiffSnapshotWriteDisposition::Replayed,
            ));
        }

        let snapshot = serde_json::to_value(record.snapshot())
            .map_err(|_| ContextDiffSnapshotPersistenceError::SnapshotInvalid)?;
        sqlx::query(
            "INSERT INTO context_diff_snapshots (project_id, context_id, context_commit_id, schema_version, snapshot, snapshot_digest, captured_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(scope.project_id().as_uuid())
        .bind(scope.context_id().as_uuid())
        .bind(scope.commit_id().as_uuid())
        .bind(record.schema_version())
        .bind(snapshot)
        .bind(record.snapshot_digest())
        .bind(record.captured_at())
        .execute(&mut *transaction)
        .await
        .map_err(context_diff_snapshot_database_error)?;

        let (schema_version, snapshot, snapshot_digest, captured_at) =
            sqlx::query_as::<_, (String, Value, String, DateTime<Utc>)>(
                "SELECT schema_version, snapshot, snapshot_digest, captured_at FROM context_diff_snapshots WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND schema_version = $4",
            )
            .bind(scope.project_id().as_uuid())
            .bind(scope.context_id().as_uuid())
            .bind(scope.commit_id().as_uuid())
            .bind(record.schema_version())
            .fetch_one(&mut *transaction)
            .await
            .map_err(context_diff_snapshot_database_error)?;
        let stored = decode_context_diff_snapshot(
            scope,
            schema_version,
            snapshot,
            snapshot_digest,
            captured_at,
        )?;
        if stored != record {
            return Err(ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid);
        }
        transaction
            .commit()
            .await
            .map_err(context_diff_snapshot_database_error)?;
        Ok(ContextDiffSnapshotWriteResult::new(
            scope,
            ContextDiffSnapshotWriteDisposition::Created,
        ))
    }

    async fn read_context_diff_snapshot(
        &self,
        scope: VersionedContextScopeV1,
    ) -> Result<crate::ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError> {
        validate_scope(scope)?;
        let mut transaction = begin_consistent_read_transaction(&self.pool)
            .await
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;
        let row = sqlx::query_as::<_, (String, Value, String, DateTime<Utc>)>(
            "SELECT schema_version, snapshot, snapshot_digest, captured_at FROM context_diff_snapshots WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND schema_version = $4",
        )
        .bind(scope.project_id().as_uuid())
        .bind(scope.context_id().as_uuid())
        .bind(scope.commit_id().as_uuid())
        .bind(CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(context_diff_snapshot_database_error)?
        .ok_or(ContextDiffSnapshotPersistenceError::NotFound { scope })?;
        let stored = decode_context_diff_snapshot(scope, row.0, row.1, row.2, row.3)?;
        transaction
            .commit()
            .await
            .map_err(context_diff_snapshot_database_error)?;
        Ok(stored)
    }
}

#[async_trait]
impl ContextDiffSnapshotV1PairRepository for PostgresContextGraphRepository {
    async fn read_context_diff_snapshot_pair(
        &self,
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Pair, ContextDiffSnapshotPersistenceError> {
        validate_scope(source_scope)?;
        validate_scope(target_scope)?;
        let mut transaction = begin_consistent_read_transaction(&self.pool)
            .await
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;

        let source_row = sqlx::query_as::<_, (String, Value, String, DateTime<Utc>)>(
            "SELECT schema_version, snapshot, snapshot_digest, captured_at FROM context_diff_snapshots WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND schema_version = $4",
        )
        .bind(source_scope.project_id().as_uuid())
        .bind(source_scope.context_id().as_uuid())
        .bind(source_scope.commit_id().as_uuid())
        .bind(CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(context_diff_snapshot_database_error)?
        .ok_or(ContextDiffSnapshotPersistenceError::NotFound {
            scope: source_scope,
        })?;

        let target_row = sqlx::query_as::<_, (String, Value, String, DateTime<Utc>)>(
            "SELECT schema_version, snapshot, snapshot_digest, captured_at FROM context_diff_snapshots WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND schema_version = $4",
        )
        .bind(target_scope.project_id().as_uuid())
        .bind(target_scope.context_id().as_uuid())
        .bind(target_scope.commit_id().as_uuid())
        .bind(CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(context_diff_snapshot_database_error)?
        .ok_or(ContextDiffSnapshotPersistenceError::NotFound {
            scope: target_scope,
        })?;

        let source = decode_context_diff_snapshot(
            source_scope,
            source_row.0,
            source_row.1,
            source_row.2,
            source_row.3,
        )?;
        let target = decode_context_diff_snapshot(
            target_scope,
            target_row.0,
            target_row.1,
            target_row.2,
            target_row.3,
        )?;
        transaction
            .commit()
            .await
            .map_err(context_diff_snapshot_database_error)?;
        Ok(ContextDiffSnapshotV1Pair::new(source, target))
    }
}

fn decode_knowledge_memory_projection(
    scope: KnowledgeMemoryProjectionScope,
    schema_version: String,
    projection: Value,
) -> Result<
    contextlab_knowledge::KnowledgeMemoryContextProjectionV1,
    KnowledgeMemoryProjectionPersistenceError,
> {
    if schema_version
        != crate::knowledge_memory_projection::KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_SCHEMA_V1
    {
        return Err(
            KnowledgeMemoryProjectionPersistenceError::UnsupportedSchema {
                received: schema_version,
            },
        );
    }
    let projection = serde_json::from_value(projection)
        .map_err(|_| KnowledgeMemoryProjectionPersistenceError::StoredProjectionInvalid)?;
    PersistKnowledgeMemoryProjectionV1::new(scope, projection)
        .map(|command| command.projection().clone())
        .map_err(|_| KnowledgeMemoryProjectionPersistenceError::StoredProjectionInvalid)
}

fn knowledge_memory_database_error(
    _error: sqlx::Error,
) -> KnowledgeMemoryProjectionPersistenceError {
    KnowledgeMemoryProjectionPersistenceError::Database {
        message: "database query failed".to_owned(),
    }
}

#[async_trait]
impl ContextLifecycleRootRepository for PostgresContextGraphRepository {
    async fn get_context_lifecycle_root(
        &self,
        context_id: ContextId,
    ) -> Result<ContextLifecycleRoot, StorageRepositoryError> {
        let root = sqlx::query_as::<_, (Uuid, String)>(CONTEXT_LIFECYCLE_ROOT_SQL)
            .bind(context_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;
        let (project_id, name) = root.ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("context:{context_id}"),
        })?;
        Ok(ContextLifecycleRoot::new(
            ProjectId::from_uuid(project_id),
            name,
        ))
    }
}

#[async_trait]
impl ContextRoleResolver for PostgresContextGraphRepository {
    async fn resolve_context_role(
        &self,
        principal: &AuthenticatedPrincipal,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<Option<WorkspaceRole>, AuthorizationError> {
        let role = sqlx::query_as::<_, (String,)>(CONTEXT_MEMBERSHIP_ROLE_SQL)
            .bind(context_id.as_uuid())
            .bind(principal.identity().source().as_str())
            .bind(principal.id().as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthorizationError::Unavailable)?
            .and_then(|(value,)| WorkspaceRole::from_storage(&value));

        Ok(role)
    }

    async fn resolve_context_role_assignments(
        &self,
        principal: &AuthenticatedPrincipal,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<WorkspaceRoleAssignments, AuthorizationError> {
        let direct_role = self.resolve_context_role(principal, context_id).await?;
        if direct_role.is_some() || principal.external_groups().is_empty() {
            return Ok(WorkspaceRoleAssignments::new(direct_role, []));
        }

        let external_group_ids = principal
            .external_groups()
            .iter()
            .map(|group| group.as_str().to_owned())
            .collect::<Vec<_>>();
        let group_roles = sqlx::query_as::<_, (String,)>(CONTEXT_EXTERNAL_GROUP_ROLE_SQL)
            .bind(context_id.as_uuid())
            .bind(principal.identity().source().as_str())
            .bind(external_group_ids)
            .fetch_all(&self.pool)
            .await
            .map_err(|_| AuthorizationError::Unavailable)?
            .into_iter()
            .map(|(value,)| {
                GroupWorkspaceRole::from_storage(&value).ok_or(AuthorizationError::Unavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(WorkspaceRoleAssignments::new(None, group_roles))
    }
}

#[async_trait]
impl WorkspaceRoleResolver for PostgresContextGraphRepository {
    async fn resolve_workspace_role(
        &self,
        principal: &AuthenticatedPrincipal,
        workspace_id: contextlab_context_core::WorkspaceId,
    ) -> Result<Option<WorkspaceRole>, AuthorizationError> {
        let role = sqlx::query_as::<_, (String,)>(WORKSPACE_DIRECT_MEMBERSHIP_ROLE_SQL)
            .bind(workspace_id.as_uuid())
            .bind(principal.identity().source().as_str())
            .bind(principal.id().as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AuthorizationError::Unavailable)?
            .and_then(|(value,)| WorkspaceRole::from_storage(&value));

        Ok(role)
    }
}

#[async_trait]
impl AuthorizationAuditSink for PostgresContextGraphRepository {
    async fn record(&self, event: AuthorizationAuditEvent) -> Result<(), AuthorizationAuditError> {
        sqlx::query(INSERT_CONTEXT_AUTHORIZATION_AUDIT_EVENT_SQL)
            .bind(event.principal_identity().source().as_str())
            .bind(event.principal_id().as_str())
            .bind(event.context_id().as_uuid())
            .bind(event.permission().as_str())
            .bind(event.decision().as_str())
            .execute(&self.pool)
            .await
            .map_err(|_| AuthorizationAuditError::Unavailable)?;

        Ok(())
    }
}

#[async_trait]
impl AuthorizationAuditReviewRepository for PostgresContextGraphRepository {
    async fn list_authorization_audit_review(
        &self,
        workspace_id: contextlab_context_core::WorkspaceId,
        query: AuthorizationAuditReviewQuery,
    ) -> Result<AuthorizationAuditReviewPage, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let workspace_exists = sqlx::query_as::<_, (bool,)>(WORKSPACE_EXISTS_SQL)
            .bind(workspace_id.as_uuid())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;
        if !workspace_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("workspace:{workspace_id}"),
            });
        }

        let cursor_recorded_at = query
            .cursor()
            .map(AuthorizationAuditReviewCursor::recorded_at);
        let cursor_audit_event_id = query
            .cursor()
            .map(AuthorizationAuditReviewCursor::audit_event_id);
        let page_size = usize::try_from(query.per_page()).expect("bounded review page size");
        let fetch_limit = i64::from(query.per_page()).saturating_add(1);
        let mut rows = sqlx::query_as::<
            _,
            (
                Uuid,
                DateTime<Utc>,
                Uuid,
                String,
                String,
                String,
                Option<Uuid>,
            ),
        >(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL)
        .bind(workspace_id.as_uuid())
        .bind(cursor_recorded_at)
        .bind(cursor_audit_event_id)
        .bind(fetch_limit)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;

        let has_more = rows.len() > page_size;
        rows.truncate(page_size);
        let next_cursor = has_more
            .then(|| {
                rows.last().map(|(audit_event_id, recorded_at, ..)| {
                    AuthorizationAuditReviewCursor::from_database(*recorded_at, *audit_event_id)
                })
            })
            .flatten();
        let items = rows
            .into_iter()
            .map(
                |(
                    _,
                    recorded_at,
                    context_id,
                    permission,
                    decision,
                    retention_disposition,
                    retention_policy_revision_id,
                )| {
                    let permission = match permission.as_str() {
                        "read" => ContextPermission::Read,
                        "write" => ContextPermission::Write,
                        _ => {
                            return Err(StorageRepositoryError::Database {
                                message:
                                    "authorization audit review contains an unknown permission"
                                        .to_owned(),
                            });
                        }
                    };
                    let decision = match decision.as_str() {
                        "granted" => contextlab_auth::AuthorizationDecision::Granted,
                        "forbidden" => contextlab_auth::AuthorizationDecision::Forbidden,
                        "unavailable" => contextlab_auth::AuthorizationDecision::Unavailable,
                        _ => {
                            return Err(StorageRepositoryError::Database {
                                message: "authorization audit review contains an unknown decision"
                                    .to_owned(),
                            });
                        }
                    };

                    Ok(AuthorizationAuditReviewItem::new(
                        recorded_at,
                        contextlab_context_core::ContextId::from_uuid(context_id),
                        permission,
                        decision,
                        AuthorizationAuditRetentionDisposition::from_database(
                            &retention_disposition,
                        )?,
                        retention_policy_revision_id,
                    ))
                },
            )
            .collect::<Result<Vec<_>, StorageRepositoryError>>()?;

        transaction.commit().await.map_err(database_error)?;
        Ok(AuthorizationAuditReviewPage::new(items, next_cursor))
    }
}

#[async_trait]
impl WorkspaceRepository for PostgresContextGraphRepository {
    async fn list_workspaces(
        &self,
        query: WorkspaceListQuery,
    ) -> Result<WorkspaceList, StorageRepositoryError> {
        let search = query.search.clone();
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let total = sqlx::query_as::<_, (i64,)>(WORKSPACE_COUNT_SQL)
            .bind(search.as_deref())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0 as u64;

        let items_sql = format!(
            r#"
SELECT id, name, slug, created_at
FROM workspaces
WHERE deleted_at IS NULL
  AND (
      $1::TEXT IS NULL
      OR name ILIKE '%' || $1 || '%'
      OR slug ILIKE '%' || $1 || '%'
      OR id::TEXT ILIKE '%' || $1 || '%'
  )
ORDER BY {}
LIMIT $2 OFFSET $3
"#,
            query.sort.order_by_sql()
        );
        let offset = i64::try_from(query.offset()).unwrap_or(i64::MAX);

        let items = sqlx::query_as::<_, (Uuid, String, String, DateTime<Utc>)>(&items_sql)
            .bind(search.as_deref())
            .bind(i64::from(query.per_page))
            .bind(offset)
            .fetch_all(&mut *transaction)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(|(id, name, slug, created_at)| WorkspaceListItem {
                id: id.to_string(),
                name,
                slug,
                created_at,
            })
            .collect();

        transaction.commit().await.map_err(database_error)?;

        Ok(WorkspaceList::new(items, &query, total))
    }
}

#[async_trait]
impl ProjectRepository for PostgresContextGraphRepository {
    async fn list_projects(
        &self,
        workspace_id: String,
        query: ProjectListQuery,
    ) -> Result<ProjectList, StorageRepositoryError> {
        let workspace_id = parse_workspace_id(&workspace_id)?;
        let search = query.search.clone();
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let workspace_exists = sqlx::query_as::<_, (bool,)>(WORKSPACE_EXISTS_SQL)
            .bind(workspace_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !workspace_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("workspace:{workspace_id}"),
            });
        }

        let total = sqlx::query_as::<_, (i64,)>(PROJECT_COUNT_BY_WORKSPACE_SQL)
            .bind(workspace_id)
            .bind(search.as_deref())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0 as u64;

        let items_sql = format!(
            r#"
SELECT id, workspace_id, name, slug, created_at
FROM projects
WHERE workspace_id = $1
  AND deleted_at IS NULL
  AND (
      $2::TEXT IS NULL
      OR name ILIKE '%' || $2 || '%'
      OR slug ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
ORDER BY {}
LIMIT $3 OFFSET $4
"#,
            query.sort.order_by_sql()
        );
        let offset = i64::try_from(query.offset()).unwrap_or(i64::MAX);

        let items = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>)>(&items_sql)
            .bind(workspace_id)
            .bind(search.as_deref())
            .bind(i64::from(query.per_page))
            .bind(offset)
            .fetch_all(&mut *transaction)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(
                |(id, workspace_id, name, slug, created_at)| ProjectListItem {
                    id: id.to_string(),
                    workspace_id: workspace_id.to_string(),
                    name,
                    slug,
                    created_at,
                },
            )
            .collect();

        transaction.commit().await.map_err(database_error)?;

        Ok(ProjectList::new(items, &query, total))
    }
}

#[async_trait]
impl ExperimentRepository for PostgresContextGraphRepository {
    async fn list_experiments(
        &self,
        project_id: String,
        query: ExperimentListQuery,
    ) -> Result<ExperimentList, StorageRepositoryError> {
        let project_id = parse_project_id(&project_id)?;
        let search = query.search.clone();
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let project_exists = sqlx::query_as::<_, (bool,)>(PROJECT_EXISTS_SQL)
            .bind(project_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !project_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }

        let total = sqlx::query_as::<_, (i64,)>(EXPERIMENT_COUNT_BY_PROJECT_SQL)
            .bind(project_id)
            .bind(search.as_deref())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0 as u64;

        let items_sql = format!(
            r#"
SELECT id, project_id, name, branch_name, created_at
FROM experiments
WHERE project_id = $1
  AND deleted_at IS NULL
  AND (
      $2::TEXT IS NULL
      OR name ILIKE '%' || $2 || '%'
      OR branch_name ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
ORDER BY {}
LIMIT $3 OFFSET $4
"#,
            query.sort.order_by_sql()
        );
        let offset = i64::try_from(query.offset()).unwrap_or(i64::MAX);

        let items = sqlx::query_as::<_, (Uuid, Uuid, String, String, DateTime<Utc>)>(&items_sql)
            .bind(project_id)
            .bind(search.as_deref())
            .bind(i64::from(query.per_page))
            .bind(offset)
            .fetch_all(&mut *transaction)
            .await
            .map_err(database_error)?
            .into_iter()
            .map(
                |(id, project_id, name, branch_name, created_at)| ExperimentListItem {
                    id: id.to_string(),
                    project_id: project_id.to_string(),
                    name,
                    branch_name,
                    created_at,
                },
            )
            .collect();

        transaction.commit().await.map_err(database_error)?;

        Ok(ExperimentList::new(items, &query, total))
    }
}

#[async_trait]
impl ContextRepository for PostgresContextGraphRepository {
    async fn list_contexts(
        &self,
        project_id: String,
        query: ContextListQuery,
    ) -> Result<ContextList, StorageRepositoryError> {
        let project_id = parse_project_id(&project_id)?;
        let experiment_id = query
            .experiment_id
            .as_deref()
            .map(parse_experiment_id)
            .transpose()?;
        let search = query.search.clone();
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let project_exists = sqlx::query_as::<_, (bool,)>(PROJECT_EXISTS_SQL)
            .bind(project_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !project_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }

        let total = sqlx::query_as::<_, (i64,)>(CONTEXT_COUNT_BY_PROJECT_SQL)
            .bind(project_id)
            .bind(search.as_deref())
            .bind(experiment_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0 as u64;

        let items_sql = format!(
            r#"
SELECT id, project_id, experiment_id, name, description, created_at
FROM contexts
WHERE project_id = $1
  AND deleted_at IS NULL
  AND ($3::UUID IS NULL OR experiment_id = $3)
  AND (
      $2::TEXT IS NULL
      OR name ILIKE '%' || $2 || '%'
      OR description ILIKE '%' || $2 || '%'
      OR id::TEXT ILIKE '%' || $2 || '%'
  )
ORDER BY {}
LIMIT $4 OFFSET $5
"#,
            query.sort.order_by_sql()
        );
        let offset = i64::try_from(query.offset()).unwrap_or(i64::MAX);

        let items = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<Uuid>,
                String,
                Option<String>,
                DateTime<Utc>,
            ),
        >(&items_sql)
        .bind(project_id)
        .bind(search.as_deref())
        .bind(experiment_id)
        .bind(i64::from(query.per_page))
        .bind(offset)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(id, project_id, experiment_id, name, description, created_at)| ContextListItem {
                id: id.to_string(),
                project_id: project_id.to_string(),
                experiment_id: experiment_id.map(|id| id.to_string()),
                name,
                description,
                created_at,
            },
        )
        .collect();

        transaction.commit().await.map_err(database_error)?;

        Ok(ContextList::new(items, &query, total))
    }
}

#[async_trait]
impl ContextCommitRepository for PostgresContextGraphRepository {
    async fn list_commits(
        &self,
        context_id: String,
        query: CommitListQuery,
    ) -> Result<CommitList, StorageRepositoryError> {
        let context_id = parse_context_id(&context_id)?;
        let search = query.search.clone();
        let branch_name = query.branch_name.clone();
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let total = sqlx::query_as::<_, (i64,)>(COMMIT_COUNT_BY_CONTEXT_SQL)
            .bind(context_id)
            .bind(search.as_deref())
            .bind(branch_name.as_deref())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0 as u64;

        let items_sql = format!(
            r#"
SELECT context_commits.id,
       context_commits.context_id,
       context_commits.branch_name,
       context_commits.message,
       COALESCE(
           ARRAY_AGG(
               context_commit_parents.parent_commit_id
               ORDER BY context_commit_parents.position
           ) FILTER (WHERE context_commit_parents.parent_commit_id IS NOT NULL),
           ARRAY[]::UUID[]
       ) AS parent_commit_ids,
       jsonb_array_length(context_commits.changes)::INTEGER AS change_count,
       context_commits.authored_at,
       context_commits.created_at
FROM context_commits
LEFT JOIN context_commit_parents
  ON context_commit_parents.commit_id = context_commits.id
WHERE context_commits.context_id = $1
  AND ($3::TEXT IS NULL OR context_commits.branch_name = $3)
  AND (
      $2::TEXT IS NULL
      OR context_commits.message ILIKE '%' || $2 || '%'
      OR context_commits.branch_name ILIKE '%' || $2 || '%'
      OR context_commits.id::TEXT ILIKE '%' || $2 || '%'
  )
GROUP BY context_commits.id
ORDER BY {}
LIMIT $4 OFFSET $5
"#,
            query.sort.order_by_sql()
        );
        let offset = i64::try_from(query.offset()).unwrap_or(i64::MAX);

        let items = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                Vec<Uuid>,
                i32,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(&items_sql)
        .bind(context_id)
        .bind(search.as_deref())
        .bind(branch_name.as_deref())
        .bind(i64::from(query.per_page))
        .bind(offset)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(
                id,
                context_id,
                branch_name,
                message,
                parent_commit_ids,
                change_count,
                authored_at,
                created_at,
            )| CommitListItem {
                id: id.to_string(),
                context_id: context_id.to_string(),
                branch_name,
                message,
                parent_commit_ids: parent_commit_ids
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect(),
                change_count: u32::try_from(change_count).unwrap_or(0),
                authored_at,
                created_at,
            },
        )
        .collect();

        transaction.commit().await.map_err(database_error)?;

        Ok(CommitList::new(items, &query, total))
    }

    async fn get_commit(
        &self,
        context_id: String,
        commit_id: String,
    ) -> Result<CommitDetail, StorageRepositoryError> {
        let context_id = parse_context_id(&context_id)?;
        let commit_id = parse_commit_id(&commit_id)?;
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let commit = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                Vec<Uuid>,
                Value,
                i32,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(COMMIT_BY_CONTEXT_SQL)
        .bind(context_id)
        .bind(commit_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        let Some((
            id,
            context_id,
            branch_name,
            message,
            parent_commit_ids,
            changes,
            change_count,
            authored_at,
            created_at,
        )) = commit
        else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        };

        transaction.commit().await.map_err(database_error)?;

        Ok(CommitDetail {
            id: id.to_string(),
            context_id: context_id.to_string(),
            branch_name,
            message,
            parent_commit_ids: parent_commit_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            changes,
            change_count: u32::try_from(change_count).unwrap_or(0),
            authored_at,
            created_at,
        })
    }
}

#[async_trait]
impl ContextCommitHistoryRepository for PostgresContextGraphRepository {
    async fn load_context_commit_history(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<contextlab_versioning::CommitHistory, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let history =
            load_context_commit_history_in_transaction(&mut transaction, context_id).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(history)
    }
}

#[async_trait]
impl ContextGraphReviewWitnessRepository for PostgresContextGraphRepository {
    async fn read_context_graph_review_witness(
        &self,
        source_scope: CommitGraphSnapshotScope,
        target_scope: CommitGraphSnapshotScope,
    ) -> Result<ContextGraphReviewWitness, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let history =
            load_context_commit_history_in_transaction(&mut transaction, source_scope.context_id())
                .await?;
        let source = load_commit_graph_snapshot_in_transaction(&mut transaction, source_scope)
            .await?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: source_scope.to_string(),
            })?;
        let target = load_commit_graph_snapshot_in_transaction(&mut transaction, target_scope)
            .await?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: target_scope.to_string(),
            })?;
        let intermediate_snapshots =
            load_context_graph_review_intermediate_snapshots_in_transaction(
                &mut transaction,
                &history,
                source_scope,
                target_scope,
            )
            .await?;
        transaction.commit().await.map_err(database_error)?;

        ContextGraphReviewWitness::try_from_parts_with_intermediate_snapshots(
            history,
            source,
            target,
            intermediate_snapshots,
        )
        .map_err(|error| StorageRepositoryError::InvalidScope {
            scope: format!("context_graph_review:{source_scope}/{target_scope}"),
            reason: error.to_string(),
        })
    }
}

#[async_trait]
impl ContextGraphBranchHeadReviewWitnessRepository for PostgresContextGraphRepository {
    async fn read_context_graph_branch_head_review_witness(
        &self,
        source_scope: CommitGraphSnapshotScope,
        branch: BranchName,
    ) -> Result<
        ContextGraphBranchHeadReviewWitness,
        ContextGraphBranchHeadReviewWitnessRepositoryError,
    > {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let history =
            load_context_commit_history_in_transaction(&mut transaction, source_scope.context_id())
                .await?;
        let target_commit_id =
            crate::context_graph_history_review::select_branch_head(&history, &branch)?;
        let target_scope = CommitGraphSnapshotScope::new(
            source_scope.project_id(),
            source_scope.context_id(),
            target_commit_id,
        );
        let source = load_commit_graph_snapshot_in_transaction(&mut transaction, source_scope)
            .await?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: source_scope.to_string(),
            })?;
        let target = load_commit_graph_snapshot_in_transaction(&mut transaction, target_scope)
            .await?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: target_scope.to_string(),
            })?;
        let intermediate_snapshots =
            load_context_graph_review_intermediate_snapshots_in_transaction(
                &mut transaction,
                &history,
                source_scope,
                target_scope,
            )
            .await
            .map_err(|source| {
                ContextGraphBranchHeadReviewWitnessRepositoryError::Storage { source }
            })?;
        transaction.commit().await.map_err(database_error)?;

        let witness = ContextGraphReviewWitness::try_from_parts_with_intermediate_snapshots(
            history,
            source,
            target,
            intermediate_snapshots,
        )
        .map_err(|source| ContextGraphBranchHeadReviewWitnessRepositoryError::Witness { source })?;
        ContextGraphBranchHeadReviewWitness::try_from_parts(branch, witness)
    }
}

async fn load_context_commit_history_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: ContextId,
) -> Result<contextlab_versioning::CommitHistory, StorageRepositoryError> {
    let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
        .bind(context_id.as_uuid())
        .fetch_one(&mut **transaction)
        .await
        .map_err(database_error)?
        .0;
    if !context_exists {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("context:{context_id}"),
        });
    }

    let details = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            String,
            String,
            Vec<Uuid>,
            Value,
            i32,
            DateTime<Utc>,
            DateTime<Utc>,
        ),
    >(CONTEXT_COMMIT_HISTORY_ROWS_SQL)
    .bind(context_id.as_uuid())
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?
    .into_iter()
    .map(
        |(
            id,
            row_context_id,
            branch_name,
            message,
            parent_commit_ids,
            changes,
            change_count,
            authored_at,
            created_at,
        )| CommitDetail {
            id: id.to_string(),
            context_id: row_context_id.to_string(),
            branch_name,
            message,
            parent_commit_ids: parent_commit_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            changes,
            change_count: u32::try_from(change_count).unwrap_or(0),
            authored_at,
            created_at,
        },
    )
    .collect::<Vec<_>>();

    let heads = sqlx::query_as::<_, (String, Option<Uuid>, i64, bool)>(CONTEXT_BRANCH_HEADS_SQL)
        .bind(context_id.as_uuid())
        .fetch_all(&mut **transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(branch, head_commit_id, revision, head_belongs_to_context)| {
                ContextBranchHead::from_stored(
                    context_id,
                    branch,
                    head_commit_id,
                    revision,
                    head_belongs_to_context,
                )
            },
        )
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| StorageRepositoryError::InvalidScope {
            scope: format!("context_commit_history:{context_id}"),
            reason: error.to_string(),
        })?;

    assemble_context_commit_history(context_id, details, heads)
}

async fn load_commit_graph_snapshot_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    scope: CommitGraphSnapshotScope,
) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
    let project_id = sqlx::query_as::<_, (Uuid,)>(CONTEXT_PROJECT_BY_ID_SQL)
        .bind(scope.context_id().as_uuid())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;
    let Some((project_id,)) = project_id else {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("context:{}", scope.context_id()),
        });
    };
    if ProjectId::from_uuid(project_id) != scope.project_id() {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: scope.to_string(),
        });
    }

    let row = sqlx::query_as::<_, (Option<i16>, Option<Value>, Option<DateTime<Utc>>)>(
        COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL,
    )
    .bind(scope.project_id().as_uuid())
    .bind(scope.context_id().as_uuid())
    .bind(scope.commit_id().as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    let Some(row) = row else {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: scope.to_string(),
        });
    };
    materialize_commit_graph_snapshot(scope, row)
}

async fn load_context_graph_review_intermediate_snapshots_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    history: &contextlab_versioning::CommitHistory,
    source_scope: CommitGraphSnapshotScope,
    target_scope: CommitGraphSnapshotScope,
) -> Result<Vec<CommitGraphSnapshot>, StorageRepositoryError> {
    let commit_ids =
        crate::context_graph_history_review::normal_first_parent_intermediate_commit_ids(
            history,
            source_scope.commit_id(),
            target_scope.commit_id(),
        )
        .map_err(|reason| StorageRepositoryError::InvalidScope {
            scope: format!("context_graph_review:{source_scope}/{target_scope}"),
            reason: reason.to_string(),
        })?;

    let mut snapshots = Vec::with_capacity(commit_ids.len());
    for commit_id in commit_ids {
        let scope = CommitGraphSnapshotScope::new(
            source_scope.project_id(),
            source_scope.context_id(),
            commit_id,
        );
        let snapshot = load_commit_graph_snapshot_in_transaction(transaction, scope)
            .await?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: scope.to_string(),
            })?;
        snapshots.push(snapshot);
    }
    Ok(snapshots)
}

#[async_trait]
impl ContextCommitGraphRepository for PostgresContextGraphRepository {
    async fn load_context_commit_graph(
        &self,
        context_id: ContextId,
    ) -> Result<contextlab_versioning::CommitGraph, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id.as_uuid())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;
        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let rows = sqlx::query_as::<_, (Uuid, Uuid, Vec<Uuid>)>(COMMIT_GRAPH_BY_CONTEXT_SQL)
            .bind(context_id.as_uuid())
            .fetch_all(&mut *transaction)
            .await
            .map_err(database_error)?;
        transaction.commit().await.map_err(database_error)?;

        let nodes = rows.into_iter().map(|(id, stored_context_id, parents)| {
            if stored_context_id != context_id.as_uuid() {
                return Err(invalid_graph(
                    context_id,
                    format!("commit {id} belongs to context {stored_context_id}"),
                ));
            }
            Ok(CommitGraphNode::new(
                CommitId::from_uuid(id),
                context_id,
                parents.into_iter().map(CommitId::from_uuid).collect(),
            ))
        });
        commit_graph_from_nodes(context_id, nodes.collect::<Result<Vec<_>, _>>()?)
    }
}

#[async_trait]
impl ContextMergeReviewWitnessRepository for PostgresContextGraphRepository {
    async fn load_context_merge_review_witness(
        &self,
        scope: ContextMergeTipScope,
    ) -> Result<ContextMergeReviewWitness, ContextMergeReviewWitnessRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool)
            .await
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        let project_id = sqlx::query_scalar::<_, Uuid>(CONTEXT_PROJECT_BY_ID_SQL)
            .bind(scope.context_id().as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        let Some(project_id) = project_id else {
            return Err(ContextMergeReviewWitnessRepositoryError::Read(
                StorageRepositoryError::ScopeUnavailable {
                    scope: format!("context:{}", scope.context_id()),
                },
            ));
        };
        if ProjectId::from_uuid(project_id) != scope.project_id() {
            return Err(ContextMergeReviewWitnessRepositoryError::Read(
                StorageRepositoryError::ScopeUnavailable {
                    scope: format!(
                        "project:{}/context:{}",
                        scope.project_id(),
                        scope.context_id()
                    ),
                },
            ));
        }

        let rows = sqlx::query_as::<_, (Uuid, Uuid, Vec<Uuid>)>(COMMIT_GRAPH_BY_CONTEXT_SQL)
            .bind(scope.context_id().as_uuid())
            .fetch_all(&mut *transaction)
            .await
            .map_err(database_error)
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        let nodes = rows.into_iter().map(|(id, stored_context_id, parents)| {
            if stored_context_id != scope.context_id().as_uuid() {
                return Err(StorageRepositoryError::InvalidScope {
                    scope: format!("commit_graph:{}", scope.context_id()),
                    reason: format!("commit {id} belongs to context {stored_context_id}"),
                });
            }
            Ok(CommitGraphNode::new(
                CommitId::from_uuid(id),
                scope.context_id(),
                parents.into_iter().map(CommitId::from_uuid).collect(),
            ))
        });
        let nodes = nodes
            .collect::<Result<Vec<_>, _>>()
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        let graph = commit_graph_from_nodes(scope.context_id(), nodes)
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        let plan = MergePlan::resolve(&graph, scope.left_commit_id(), scope.right_commit_id())
            .map_err(ContextMergeReviewWitnessRepositoryError::PlanResolution)?;
        let MergePlan::ThreeWay { base, left, right } = plan.clone() else {
            return Err(ContextMergeReviewWitnessRepositoryError::InvalidWitness(
                ContextMergeReviewWitnessError::NonThreeWay,
            ));
        };
        let input_scope =
            ContextMergeInputScope::new(scope.project_id(), scope.context_id(), base, left, right)
                .map_err(|error| {
                    ContextMergeReviewWitnessRepositoryError::InvalidWitness(
                        ContextMergeReviewWitnessError::InvalidInputScope(error),
                    )
                })?;
        let scopes = [
            (
                crate::PersistedContextGraphMergeReviewSide::Base,
                input_scope.snapshot_scope(crate::PersistedContextGraphMergeReviewSide::Base),
            ),
            (
                crate::PersistedContextGraphMergeReviewSide::Left,
                input_scope.snapshot_scope(crate::PersistedContextGraphMergeReviewSide::Left),
            ),
            (
                crate::PersistedContextGraphMergeReviewSide::Right,
                input_scope.snapshot_scope(crate::PersistedContextGraphMergeReviewSide::Right),
            ),
        ];
        let mut snapshots = Vec::with_capacity(scopes.len());
        for (side, expected) in scopes {
            let row = sqlx::query_as::<_, (Option<i16>, Option<Value>, Option<DateTime<Utc>>)>(
                COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL,
            )
            .bind(expected.project_id().as_uuid())
            .bind(expected.context_id().as_uuid())
            .bind(expected.commit_id().as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
            let Some(row) = row else {
                return Err(
                    ContextMergeReviewWitnessRepositoryError::SnapshotUnavailable {
                        side,
                        scope: expected,
                    },
                );
            };
            let snapshot = materialize_commit_graph_snapshot(expected, row)
                .map_err(
                    |source| ContextMergeReviewWitnessRepositoryError::SnapshotRead {
                        side,
                        scope: expected,
                        source,
                    },
                )?
                .ok_or(
                    ContextMergeReviewWitnessRepositoryError::SnapshotUnavailable {
                        side,
                        scope: expected,
                    },
                )?;
            snapshots.push(snapshot);
        }
        let [base_snapshot, left_snapshot, right_snapshot] = snapshots
            .try_into()
            .expect("the merge witness always contains three snapshots");

        let witness =
            ContextMergeReviewWitness::new(plan, base_snapshot, left_snapshot, right_snapshot)
                .map_err(ContextMergeReviewWitnessRepositoryError::InvalidWitness)?;
        transaction
            .commit()
            .await
            .map_err(database_error)
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        Ok(witness)
    }
}

#[async_trait]
impl CommitGraphSnapshotRepository for PostgresContextGraphRepository {
    async fn project_id_for_context(
        &self,
        context_id: ContextId,
    ) -> Result<ProjectId, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let project_id = sqlx::query_scalar::<_, Uuid>(CONTEXT_PROJECT_BY_ID_SQL)
            .bind(context_id.as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        let Some(project_id) = project_id else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        };
        transaction.commit().await.map_err(database_error)?;
        Ok(ProjectId::from_uuid(project_id))
    }

    async fn scope_for_context_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let project_id = sqlx::query_scalar::<_, Uuid>(CONTEXT_PROJECT_FOR_COMMIT_SCOPE_SQL)
            .bind(context_id.as_uuid())
            .bind(commit_id.as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        let Some(project_id) = project_id else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}/commit:{commit_id}"),
            });
        };
        transaction.commit().await.map_err(database_error)?;

        Ok(CommitGraphSnapshotScope::new(
            ProjectId::from_uuid(project_id),
            context_id,
            commit_id,
        ))
    }

    async fn get_commit_graph_snapshot(
        &self,
        scope: CommitGraphSnapshotScope,
    ) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let project_id = sqlx::query_as::<_, (Uuid,)>(CONTEXT_PROJECT_BY_ID_SQL)
            .bind(scope.context_id().as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        let Some((project_id,)) = project_id else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{}", scope.context_id()),
            });
        };
        if ProjectId::from_uuid(project_id) != scope.project_id() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: scope.to_string(),
            });
        }

        let row = sqlx::query_as::<_, (Option<i16>, Option<Value>, Option<DateTime<Utc>>)>(
            COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL,
        )
        .bind(scope.project_id().as_uuid())
        .bind(scope.context_id().as_uuid())
        .bind(scope.commit_id().as_uuid())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;
        let Some(row) = row else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: scope.to_string(),
            });
        };
        let snapshot = materialize_commit_graph_snapshot(scope, row)?;
        transaction.commit().await.map_err(database_error)?;
        Ok(snapshot)
    }

    async fn get_commit_graph_snapshot_batch(
        &self,
        scope: ContextMergeInputScope,
    ) -> Result<
        (
            CommitGraphSnapshot,
            CommitGraphSnapshot,
            CommitGraphSnapshot,
        ),
        StorageRepositoryError,
    > {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let project_id = sqlx::query_as::<_, (Uuid,)>(CONTEXT_PROJECT_BY_ID_SQL)
            .bind(scope.context_id().as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        let Some((project_id,)) = project_id else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{}", scope.context_id()),
            });
        };
        if ProjectId::from_uuid(project_id) != scope.project_id() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!(
                    "project:{}/context:{}",
                    scope.project_id(),
                    scope.context_id()
                ),
            });
        }

        let scopes = [
            CommitGraphSnapshotScope::new(
                scope.project_id(),
                scope.context_id(),
                scope.base_commit_id(),
            ),
            CommitGraphSnapshotScope::new(
                scope.project_id(),
                scope.context_id(),
                scope.left_commit_id(),
            ),
            CommitGraphSnapshotScope::new(
                scope.project_id(),
                scope.context_id(),
                scope.right_commit_id(),
            ),
        ];
        let mut snapshots = Vec::with_capacity(scopes.len());
        for expected in scopes {
            let row = sqlx::query_as::<_, (Option<i16>, Option<Value>, Option<DateTime<Utc>>)>(
                COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL,
            )
            .bind(expected.project_id().as_uuid())
            .bind(expected.context_id().as_uuid())
            .bind(expected.commit_id().as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
            let Some(row) = row else {
                return Err(StorageRepositoryError::ScopeUnavailable {
                    scope: expected.to_string(),
                });
            };
            let snapshot = materialize_commit_graph_snapshot(expected, row)?.ok_or_else(|| {
                StorageRepositoryError::ScopeUnavailable {
                    scope: expected.to_string(),
                }
            })?;
            snapshots.push(snapshot);
        }
        transaction.commit().await.map_err(database_error)?;

        let [base, left, right] = snapshots
            .try_into()
            .expect("the batch scope always contains three snapshots");
        Ok((base, left, right))
    }
}

#[async_trait]
impl ContextCommitSnapshotWriter for PostgresContextGraphRepository {
    async fn create_commit_snapshot(
        &self,
        command: CreateContextCommitSnapshot,
    ) -> Result<CommitGraphSnapshot, StorageRepositoryError> {
        let (commit, snapshot, diff_snapshot) = command.into_parts();
        let context_id = commit.context_id().as_uuid();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let context = sqlx::query_as::<_, (Uuid, Uuid)>(CONTEXT_FOR_COMMIT_SNAPSHOT_WRITE_SQL)
            .bind(context_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        let Some((_, project_id)) = context else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        };

        insert_commit_snapshot(
            &mut transaction,
            ProjectId::from_uuid(project_id),
            &commit,
            &snapshot,
        )
        .await?;
        insert_context_diff_snapshot(
            &mut transaction,
            VersionedContextScopeV1::new(
                ProjectId::from_uuid(project_id),
                commit.context_id(),
                commit.id(),
            ),
            &diff_snapshot,
            snapshot.captured_at(),
        )
        .await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(snapshot)
    }
}

#[async_trait]
impl ContextWorkflowBindingRepository for PostgresContextGraphRepository {
    async fn persist_workflow_context_binding(
        &self,
        binding: WorkflowContextBinding,
    ) -> Result<WorkflowContextBindingWriteResult, StorageRepositoryError> {
        let source = binding.context_source();
        let context_id = source.context_id().as_uuid();
        let commit_id = source.commit_id().as_uuid();
        let workflow_id = binding.workflow_id().as_uuid();
        let workflow_revision = i64::try_from(binding.workflow_revision().get()).map_err(|_| {
            StorageRepositoryError::Database {
                message: "workflow revision exceeds PostgreSQL storage limit".to_owned(),
            }
        })?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let materialized =
            sqlx::query_as::<_, (Uuid,)>(MATERIALIZED_CONTEXT_COMMIT_FOR_WORKFLOW_BINDING_SQL)
                .bind(context_id)
                .bind(commit_id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(database_error)?;
        if materialized.is_none() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        sqlx::query(WORKFLOW_CONTEXT_BINDING_ADVISORY_LOCK_SQL)
            .bind(format!(
                "workflow-context-binding:{workflow_id}:{workflow_revision}"
            ))
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        let existing = sqlx::query_as::<_, (Uuid, Uuid, Uuid, Uuid, i64, Value)>(
            WORKFLOW_CONTEXT_BINDING_BY_WORKFLOW_REVISION_SQL,
        )
        .bind(workflow_id)
        .bind(workflow_revision)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;
        if let Some(row) = existing {
            let existing = decode_workflow_context_binding(row)?;
            transaction.commit().await.map_err(database_error)?;
            return if existing == binding {
                Ok(WorkflowContextBindingWriteResult::replayed(existing))
            } else {
                Err(workflow_context_binding_conflict(
                    binding.workflow_id(),
                    binding.workflow_revision(),
                ))
            };
        }

        let payload =
            serde_json::to_value(&binding).map_err(|_| StorageRepositoryError::Database {
                message: "workflow Context binding cannot be serialized".to_owned(),
            })?;
        sqlx::query(INSERT_WORKFLOW_CONTEXT_BINDING_SQL)
            .bind(binding.id().as_uuid())
            .bind(context_id)
            .bind(commit_id)
            .bind(workflow_id)
            .bind(workflow_revision)
            .bind(payload)
            .bind(Utc::now())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        transaction.commit().await.map_err(database_error)?;
        Ok(WorkflowContextBindingWriteResult::created(binding))
    }

    async fn get_workflow_context_binding(
        &self,
        workflow_id: WorkflowId,
        workflow_revision: WorkflowRevision,
    ) -> Result<Option<WorkflowContextBinding>, StorageRepositoryError> {
        let workflow_revision = i64::try_from(workflow_revision.get()).map_err(|_| {
            StorageRepositoryError::Database {
                message: "workflow revision exceeds PostgreSQL storage limit".to_owned(),
            }
        })?;
        let row = sqlx::query_as::<_, (Uuid, Uuid, Uuid, Uuid, i64, Value)>(
            WORKFLOW_CONTEXT_BINDING_BY_WORKFLOW_REVISION_SQL,
        )
        .bind(workflow_id.as_uuid())
        .bind(workflow_revision)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;
        row.map(decode_workflow_context_binding).transpose()
    }

    async fn list_workflow_context_bindings_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<Vec<WorkflowContextBinding>, StorageRepositoryError> {
        let context_id = context_id.as_uuid();
        let commit_id = commit_id.as_uuid();
        let materialized =
            sqlx::query_as::<_, (Uuid,)>(MATERIALIZED_CONTEXT_COMMIT_FOR_WORKFLOW_BINDING_SQL)
                .bind(context_id)
                .bind(commit_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(database_error)?;
        if materialized.is_none() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        sqlx::query_as::<_, (Uuid, Uuid, Uuid, Uuid, i64, Value)>(
            WORKFLOW_CONTEXT_BINDINGS_AT_COMMIT_SQL,
        )
        .bind(context_id)
        .bind(commit_id)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(decode_workflow_context_binding)
        .collect()
    }
}

async fn insert_commit_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: ProjectId,
    commit: &ContextCommit,
    snapshot: &CommitGraphSnapshot,
) -> Result<(), StorageRepositoryError> {
    let context_id = commit.context_id().as_uuid();
    let commit_id = commit.id().as_uuid();
    validate_commit_graph_snapshot_write_scope(project_id, commit, snapshot)?;
    let parent_commit_ids = commit
        .parent_ids()
        .iter()
        .map(|parent_id| parent_id.as_uuid())
        .collect::<Vec<_>>();

    if !parent_commit_ids.is_empty() {
        let available_parent_ids =
            sqlx::query_as::<_, (Uuid,)>(PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL)
                .bind(context_id)
                .bind(&parent_commit_ids)
                .fetch_all(&mut **transaction)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|(parent_id,)| parent_id)
                .collect::<BTreeSet<_>>();
        let missing_parent_id = parent_commit_ids
            .iter()
            .find(|parent_id| !available_parent_ids.contains(parent_id));

        if let Some(parent_commit_id) = missing_parent_id {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("parent_commit:{context_id}/{parent_commit_id}"),
            });
        }
    }

    let changes =
        serde_json::to_value(commit.changes()).map_err(|_| StorageRepositoryError::Database {
            message: "commit changes serialization failed".to_owned(),
        })?;
    let inserted_commit = sqlx::query_as::<_, (Uuid,)>(INSERT_CONTEXT_COMMIT_SQL)
        .bind(commit_id)
        .bind(context_id)
        .bind(commit.branch().as_str())
        .bind(commit.message())
        .bind(changes)
        .bind(commit.authored_at())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;
    if inserted_commit.is_none() {
        return Err(StorageRepositoryError::CommitAlreadyExists {
            context_id: context_id.to_string(),
            commit_id: commit_id.to_string(),
        });
    }

    for (position, parent_commit_id) in parent_commit_ids.into_iter().enumerate() {
        let position = i32::try_from(position).map_err(|_| StorageRepositoryError::Database {
            message: "commit parent count exceeds storage limit".to_owned(),
        })?;
        sqlx::query(INSERT_CONTEXT_COMMIT_PARENT_SQL)
            .bind(context_id)
            .bind(commit_id)
            .bind(parent_commit_id)
            .bind(position)
            .execute(&mut **transaction)
            .await
            .map_err(database_error)?;
    }

    sqlx::query(INSERT_CONTEXT_COMMIT_GRAPH_SNAPSHOT_SQL)
        .bind(commit_id)
        .bind(i16::try_from(snapshot.schema_version()).map_err(|_| {
            StorageRepositoryError::Database {
                message: "snapshot schema version exceeds storage limit".to_owned(),
            }
        })?)
        .bind(snapshot.graph_payload())
        .bind(snapshot.captured_at())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    Ok(())
}

async fn insert_context_diff_snapshot(
    transaction: &mut Transaction<'_, Postgres>,
    scope: VersionedContextScopeV1,
    snapshot: &ContextDiffSnapshotV1,
    captured_at: DateTime<Utc>,
) -> Result<(), StorageRepositoryError> {
    let record = PersistContextDiffSnapshotV1::new(
        scope,
        CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1,
        snapshot.clone(),
        captured_at,
    )
    .map_err(|_| StorageRepositoryError::Database {
        message: "context diff snapshot command is invalid".to_owned(),
    })?
    .record()
    .clone();
    let existing = sqlx::query_as::<_, (String, Value, String, DateTime<Utc>)>(
        "SELECT schema_version, snapshot, snapshot_digest, captured_at FROM context_diff_snapshots WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND schema_version = $4 FOR UPDATE",
    )
    .bind(scope.project_id().as_uuid())
    .bind(scope.context_id().as_uuid())
    .bind(scope.commit_id().as_uuid())
    .bind(record.schema_version())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    if let Some((schema_version, payload, digest, captured_at)) = existing {
        let stored =
            decode_context_diff_snapshot(scope, schema_version, payload, digest, captured_at)
                .map_err(|_| StorageRepositoryError::Database {
                    message: "stored context diff snapshot is invalid".to_owned(),
                })?;
        if stored != record {
            return Err(StorageRepositoryError::Database {
                message: "context diff snapshot conflicts with the commit".to_owned(),
            });
        }
        return Ok(());
    }

    let payload =
        serde_json::to_value(record.snapshot()).map_err(|_| StorageRepositoryError::Database {
            message: "context diff snapshot serialization failed".to_owned(),
        })?;
    sqlx::query(
        "INSERT INTO context_diff_snapshots (project_id, context_id, context_commit_id, schema_version, snapshot, snapshot_digest, captured_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(scope.project_id().as_uuid())
    .bind(scope.context_id().as_uuid())
    .bind(scope.commit_id().as_uuid())
    .bind(record.schema_version())
    .bind(payload)
    .bind(record.snapshot_digest())
    .bind(record.captured_at())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(())
}

#[async_trait]
impl GuardedContextCommitWriter for PostgresContextGraphRepository {
    async fn create_guarded_commit_snapshot(
        &self,
        command: crate::GuardedContextCommitWrite,
    ) -> Result<crate::GuardedCommitWriteResult, StorageRepositoryError> {
        let (
            principal,
            expected_branch_head,
            idempotency_key,
            request_digest,
            snapshot_command,
            component_content_mutation,
        ) = command.into_parts();
        let commit = snapshot_command.commit();
        let context_id = commit.context_id().as_uuid();
        let branch_name = commit.branch().as_str();
        let identity_source = principal.identity().source().as_str();
        let principal_id = principal.id().as_str();
        let commit_id = commit.id().as_uuid();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let context = sqlx::query_as::<_, (Uuid, Uuid)>(CONTEXT_FOR_COMMIT_SNAPSHOT_WRITE_SQL)
            .bind(context_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        let Some((_, project_id)) = context else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        };
        let project_id = ProjectId::from_uuid(project_id);

        let direct_role = sqlx::query_as::<_, (String,)>(CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL)
            .bind(context_id)
            .bind(identity_source)
            .bind(principal_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| StorageRepositoryError::GuardedWriteAuthorizationUnavailable)?
            .and_then(|(value,)| WorkspaceRole::from_storage(&value));
        let group_write_granted =
            if direct_role.is_none() && !principal.external_groups().is_empty() {
                let external_group_ids = principal
                    .external_groups()
                    .iter()
                    .map(|group| group.as_str().to_owned())
                    .collect::<Vec<_>>();
                sqlx::query_as::<_, (String,)>(CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL)
                    .bind(context_id)
                    .bind(identity_source)
                    .bind(external_group_ids)
                    .fetch_all(&mut *transaction)
                    .await
                    .map_err(|_| StorageRepositoryError::GuardedWriteAuthorizationUnavailable)?
                    .into_iter()
                    .map(|(value,)| {
                        GroupWorkspaceRole::from_storage(&value)
                            .ok_or(StorageRepositoryError::GuardedWriteAuthorizationUnavailable)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .any(|role| role.allows(ContextPermission::Write))
            } else {
                false
            };
        if !direct_role.is_some_and(|role| role.allows(ContextPermission::Write))
            && !group_write_granted
        {
            return Err(StorageRepositoryError::GuardedWriteForbidden);
        }

        let idempotency_lock_key = format!(
            "{}:{identity_source}|{}:{principal_id}|{context_id}|{}:{branch_name}|{}:{}",
            identity_source.len(),
            principal_id.len(),
            branch_name.len(),
            idempotency_key.as_str().len(),
            idempotency_key.as_str(),
        );
        sqlx::query(GUARDED_IDEMPOTENCY_ADVISORY_LOCK_SQL)
            .bind(idempotency_lock_key)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        let existing_idempotency = sqlx::query_as::<_, (String, Uuid)>(CONTEXT_IDEMPOTENCY_SQL)
            .bind(identity_source)
            .bind(principal_id)
            .bind(context_id)
            .bind(branch_name)
            .bind(idempotency_key.as_str())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
        if let Some((stored_digest, stored_commit_id)) = existing_idempotency {
            if stored_digest != request_digest.as_str() {
                return Err(StorageRepositoryError::IdempotencyKeyReused {
                    context_id: context_id.to_string(),
                    principal_id: principal_id.to_owned(),
                    idempotency_key: idempotency_key.to_string(),
                });
            }

            let row = sqlx::query_as::<_, (Option<i16>, Option<Value>, Option<DateTime<Utc>>)>(
                COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL,
            )
            .bind(project_id.as_uuid())
            .bind(context_id)
            .bind(stored_commit_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
            let Some(row) = row else {
                return Err(StorageRepositoryError::Database {
                    message: "idempotency result snapshot is unavailable".to_owned(),
                });
            };
            let snapshot = materialize_commit_graph_snapshot(
                CommitGraphSnapshotScope::new(
                    project_id,
                    ContextId::from_uuid(context_id),
                    CommitId::from_uuid(stored_commit_id),
                ),
                row,
            )?
            .ok_or_else(|| StorageRepositoryError::Database {
                message: "idempotency result snapshot is unavailable".to_owned(),
            })?;

            transaction.commit().await.map_err(database_error)?;
            return Ok(crate::GuardedCommitWriteResult {
                snapshot,
                disposition: crate::GuardedCommitWriteDisposition::Replayed,
            });
        }

        validate_component_content_attachment(
            snapshot_command.commit(),
            component_content_mutation.as_ref(),
        )
        .map_err(
            |error| StorageRepositoryError::ComponentContentRevisionConflict {
                reason: error.to_string(),
            },
        )?;

        sqlx::query(INSERT_CONTEXT_BRANCH_SQL)
            .bind(context_id)
            .bind(branch_name)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        let actual_branch_head =
            sqlx::query_as::<_, (Option<Uuid>,)>(CONTEXT_BRANCH_FOR_UPDATE_SQL)
                .bind(context_id)
                .bind(branch_name)
                .fetch_one(&mut *transaction)
                .await
                .map_err(database_error)?
                .0
                .map(CommitId::from_uuid);
        let required_parent = normal_commit_parent(expected_branch_head, actual_branch_head)
            .map_err(|conflict| StorageRepositoryError::BranchHeadConflict {
                expected: conflict.expected.map(|commit_id| commit_id.to_string()),
                actual: conflict.actual.map(|commit_id| commit_id.to_string()),
            })?;
        let required_parent_ids = required_parent.into_iter().collect::<Vec<_>>();
        if snapshot_command.commit().parent_ids() != required_parent_ids.as_slice() {
            return Err(StorageRepositoryError::CommitParentMismatch {
                expected_parent_id: required_parent_ids.first().map(ToString::to_string),
                actual_parent_ids: snapshot_command
                    .commit()
                    .parent_ids()
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            });
        }

        if let Some(component_content_mutation) = component_content_mutation.as_ref() {
            match component_content_mutation {
                ComponentContentMutationWrite::Revision(revision) => {
                    verify_component_content_revision(&mut transaction, context_id, revision)
                        .await?;
                }
                ComponentContentMutationWrite::Creation(creation) => {
                    verify_component_content_creation(&mut transaction, context_id, creation)
                        .await?;
                }
                ComponentContentMutationWrite::Removal(removal) => {
                    verify_component_removal(&mut transaction, context_id, removal).await?;
                }
                ComponentContentMutationWrite::Descriptor(descriptor) => {
                    verify_component_descriptor_revision(&mut transaction, context_id, descriptor)
                        .await?;
                }
            }
        }

        insert_commit_snapshot(
            &mut transaction,
            project_id,
            commit,
            snapshot_command.snapshot(),
        )
        .await?;
        insert_context_diff_snapshot(
            &mut transaction,
            VersionedContextScopeV1::new(
                project_id,
                snapshot_command.commit().context_id(),
                snapshot_command.commit().id(),
            ),
            snapshot_command.diff_snapshot(),
            snapshot_command.snapshot().captured_at(),
        )
        .await?;
        if let Some(component_content_mutation) = component_content_mutation {
            match component_content_mutation {
                ComponentContentMutationWrite::Revision(revision) => {
                    insert_component_content_revision(
                        &mut transaction,
                        context_id,
                        commit_id,
                        revision,
                    )
                    .await?;
                }
                ComponentContentMutationWrite::Creation(creation) => {
                    insert_component_content_creation(
                        &mut transaction,
                        context_id,
                        commit_id,
                        creation,
                    )
                    .await?;
                }
                ComponentContentMutationWrite::Removal(removal) => {
                    soft_remove_component(&mut transaction, context_id, removal).await?;
                }
                ComponentContentMutationWrite::Descriptor(descriptor) => {
                    update_component_descriptor(&mut transaction, context_id, descriptor).await?;
                }
            }
        }
        let updated_branch = sqlx::query(UPDATE_CONTEXT_BRANCH_HEAD_SQL)
            .bind(commit_id)
            .bind(context_id)
            .bind(branch_name)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        if updated_branch.rows_affected() != 1 {
            return Err(StorageRepositoryError::Database {
                message: "context branch head update failed".to_owned(),
            });
        }

        sqlx::query(INSERT_CONTEXT_IDEMPOTENCY_SQL)
            .bind(identity_source)
            .bind(principal_id)
            .bind(context_id)
            .bind(branch_name)
            .bind(idempotency_key.as_str())
            .bind(request_digest.as_str())
            .bind(commit_id)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;
        Ok(crate::GuardedCommitWriteResult {
            snapshot: snapshot_command.snapshot().clone(),
            disposition: crate::GuardedCommitWriteDisposition::Created,
        })
    }
}

#[async_trait]
impl ContextLifecycleReadRepository for PostgresContextGraphRepository {
    async fn get_context_lifecycle_read_facts(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<ContextLifecycleReadFacts, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let project_id = sqlx::query_scalar::<_, Uuid>(CONTEXT_PROJECT_BY_ID_SQL)
            .bind(context_id.as_uuid())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?
            .map(ProjectId::from_uuid)
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            })?;
        let scope = CommitGraphSnapshotScope::new(project_id, context_id, commit_id);

        let graph_row = sqlx::query_as::<_, (Option<i16>, Option<Value>, Option<DateTime<Utc>>)>(
            COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL,
        )
        .bind(scope.project_id().as_uuid())
        .bind(scope.context_id().as_uuid())
        .bind(scope.commit_id().as_uuid())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: scope.to_string(),
        })?;
        let graph_snapshot =
            materialize_commit_graph_snapshot(scope, graph_row)?.ok_or_else(|| {
                StorageRepositoryError::ScopeUnavailable {
                    scope: scope.to_string(),
                }
            })?;

        let history_rows = sqlx::query_as::<_, (Uuid, i64, i64, bool, bool, Value)>(
            component_state_snapshot_history_sql().as_str(),
        )
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;
        if history_rows.is_empty() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        let history_commit_ids = history_rows
            .iter()
            .map(|(source_commit_id, _, _, _, _, _)| *source_commit_id)
            .collect::<Vec<_>>();
        let revision_rows = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<String>,
                String,
                String,
                DateTime<Utc>,
                String,
            ),
        >(COMPONENT_CONTENT_REVISIONS_FOR_COMMITS_SQL)
        .bind(context_id.as_uuid())
        .bind(history_commit_ids)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;
        let mut revisions_by_commit = BTreeMap::<Uuid, Vec<ComponentContentRevision>>::new();
        for (
            revision_commit_id,
            revision_component_id,
            previous_content_hash,
            resulting_content_hash,
            content,
            captured_at,
            kind,
        ) in revision_rows
        {
            let revision = component_state_revision_from_row(ComponentStateRevisionRow {
                context_id,
                commit_id: CommitId::from_uuid(revision_commit_id),
                component_id: contextlab_context_core::ComponentId::from_uuid(
                    revision_component_id,
                ),
                previous_content_hash,
                resulting_content_hash: Some(resulting_content_hash),
                content: Some(content),
                captured_at: Some(captured_at),
                kind: Some(kind),
            })?
            .expect("complete revision query returns a revision");
            revisions_by_commit
                .entry(revision_commit_id)
                .or_default()
                .push(revision);
        }
        let all_revisions = revisions_by_commit
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        let mut steps = Vec::with_capacity(history_rows.len());
        for (source_commit_id, _depth, parent_count, cycle_detected, invalid_parent, changes) in
            history_rows
        {
            validate_component_state_history_row(parent_count, cycle_detected, invalid_parent)?;
            let changes = serde_json::from_value::<Vec<ContextChange>>(changes).map_err(|_| {
                component_state_conflict(
                    "Context lifecycle aggregate read encountered invalid commit changes",
                )
            })?;
            steps.push(ComponentStateReplayStep::new(
                CommitId::from_uuid(source_commit_id),
                changes,
                revisions_by_commit
                    .remove(&source_commit_id)
                    .unwrap_or_default(),
            ));
        }
        steps.reverse();
        let inventory = replay_context_component_state_snapshot(context_id, commit_id, steps)?;
        let contents = inventory
            .components()
            .iter()
            .map(|component| {
                all_revisions
                    .iter()
                    .find(|revision| {
                        revision.context_id() == context_id
                            && revision.component_id() == component.component().id()
                            && revision.commit_id() == component.content_commit_id()
                    })
                    .cloned()
                    .ok_or_else(
                        || StorageRepositoryError::ComponentContentRevisionConflict {
                            reason: format!(
                                "component {} has no content witness at {}",
                                component.component().id(),
                                scope
                            ),
                        },
                    )
            })
            .collect::<Result<Vec<_>, StorageRepositoryError>>()?;

        let replay_rows = sqlx::query_as::<_, ReplayStateHistoryRow>(&replay_state_history_sql())
            .bind(context_id.as_uuid())
            .bind(commit_id.as_uuid())
            .fetch_all(&mut *transaction)
            .await
            .map_err(database_error)?;
        let replay_state = replay_state_from_records(
            context_id,
            replay_state_records_from_rows(context_id, commit_id, replay_rows)?,
        )?;
        let facts = ContextLifecycleReadFacts::from_parts(
            scope,
            inventory,
            contents,
            replay_state,
            graph_snapshot,
        )?;
        transaction.commit().await.map_err(database_error)?;
        Ok(facts)
    }
}

#[async_trait]
impl ComponentContentRevisionRepository for PostgresContextGraphRepository {
    async fn get_component_content_revision(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<ComponentContentRevision>, StorageRepositoryError> {
        let row = sqlx::query_as::<_, (Option<String>, String, String, DateTime<Utc>, String)>(
            COMPONENT_CONTENT_REVISION_BY_ID_SQL,
        )
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .bind(component_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;
        let Some((previous_content_hash, resulting_content_hash, content, captured_at, kind)) = row
        else {
            return Ok(None);
        };
        let component_kind =
            crate::component_content_revision::parse_component_kind_storage_value(&kind)
                .ok_or(StorageRepositoryError::InvalidStoredComponentKind { kind })?;
        let previous_content_hash = previous_content_hash
            .map(contextlab_context_core::ContentHash::new)
            .transpose()
            .map_err(|_| invalid_component_content_revision_error())?;
        let resulting_content_hash =
            contextlab_context_core::ContentHash::new(resulting_content_hash)
                .map_err(|_| invalid_component_content_revision_error())?;

        let content = contextlab_context_core::ComponentContent::new(content);
        if content.content_hash() != resulting_content_hash {
            return Err(invalid_component_content_revision_error());
        }

        Ok(Some(ComponentContentRevision::from_persisted(
            PersistedComponentContentRevision {
                context_id,
                commit_id,
                component_id,
                component_kind,
                previous_content_hash,
                content,
                resulting_content_hash,
                captured_at,
            },
        )))
    }

    async fn get_component_content_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<ComponentContentRevision>, StorageRepositoryError> {
        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(database_error)?
            .0;
        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let history_query = component_content_at_commit_sql();
        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                i64,
                i64,
                bool,
                bool,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<DateTime<Utc>>,
                Option<String>,
            ),
        >(&history_query)
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .bind(component_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        if rows.is_empty() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        let mut revision = None;
        for (
            source_commit_id,
            _depth,
            parent_count,
            cycle_detected,
            invalid_parent,
            previous_content_hash,
            resulting_content_hash,
            content,
            captured_at,
            kind,
        ) in rows
        {
            if cycle_detected {
                return Err(StorageRepositoryError::ComponentContentRevisionConflict {
                    reason: "component content replay encountered a commit ancestry cycle"
                        .to_owned(),
                });
            }
            if invalid_parent {
                return Err(StorageRepositoryError::ComponentContentRevisionConflict {
                    reason: "component content replay encountered a cross-context parent"
                        .to_owned(),
                });
            }
            if parent_count > 1 {
                return Err(StorageRepositoryError::ComponentContentRevisionConflict {
                    reason: "component content replay does not support merge ancestry".to_owned(),
                });
            }
            if revision.is_some() || resulting_content_hash.is_none() {
                continue;
            }
            let Some(content) = content else {
                return Err(invalid_component_content_revision_error());
            };
            let Some(captured_at) = captured_at else {
                return Err(invalid_component_content_revision_error());
            };
            let Some(kind) = kind else {
                return Err(invalid_component_content_revision_error());
            };
            let component_kind =
                crate::component_content_revision::parse_component_kind_storage_value(&kind)
                    .ok_or(StorageRepositoryError::InvalidStoredComponentKind { kind })?;
            let previous_content_hash = previous_content_hash
                .map(contextlab_context_core::ContentHash::new)
                .transpose()
                .map_err(|_| invalid_component_content_revision_error())?;
            let resulting_content_hash = contextlab_context_core::ContentHash::new(
                resulting_content_hash.expect("checked above"),
            )
            .map_err(|_| invalid_component_content_revision_error())?;
            let content = contextlab_context_core::ComponentContent::new(content);
            if content.content_hash() != resulting_content_hash {
                return Err(invalid_component_content_revision_error());
            }
            revision = Some(ComponentContentRevision::from_persisted(
                PersistedComponentContentRevision {
                    context_id,
                    commit_id: CommitId::from_uuid(source_commit_id),
                    component_id,
                    component_kind,
                    previous_content_hash,
                    content,
                    resulting_content_hash,
                    captured_at,
                },
            ));
        }

        Ok(revision)
    }
}

#[async_trait]
impl ComponentStateAtCommitRepository for PostgresContextGraphRepository {
    async fn get_component_state_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<crate::ComponentStateAtCommit>, StorageRepositoryError> {
        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(database_error)?
            .0;
        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let history_query = component_state_at_commit_sql();
        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                i64,
                i64,
                bool,
                bool,
                Value,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<DateTime<Utc>>,
                Option<String>,
            ),
        >(&history_query)
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .bind(component_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        if rows.is_empty() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        let mut steps = Vec::with_capacity(rows.len());
        for (
            source_commit_id,
            _depth,
            parent_count,
            cycle_detected,
            invalid_parent,
            changes,
            previous_content_hash,
            resulting_content_hash,
            content,
            captured_at,
            kind,
        ) in rows
        {
            validate_component_state_history_row(parent_count, cycle_detected, invalid_parent)?;
            let changes = serde_json::from_value::<Vec<ContextChange>>(changes).map_err(|_| {
                component_state_conflict(
                    "component state replay encountered invalid stored commit changes",
                )
            })?;
            let revision = component_state_revision_from_row(ComponentStateRevisionRow {
                context_id,
                commit_id: CommitId::from_uuid(source_commit_id),
                component_id,
                previous_content_hash,
                resulting_content_hash,
                content,
                captured_at,
                kind,
            })?;
            steps.push(ComponentStateReplayStep::new(
                CommitId::from_uuid(source_commit_id),
                changes,
                revision,
            ));
        }
        steps.reverse();

        replay_component_state(context_id, commit_id, component_id, steps)
    }
}

#[async_trait]
impl ContextComponentStateSnapshotAtCommitRepository for PostgresContextGraphRepository {
    async fn get_context_component_state_snapshot_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
    ) -> Result<crate::ContextComponentStateSnapshotAtCommit, StorageRepositoryError> {
        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(database_error)?
            .0;
        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let history_rows = sqlx::query_as::<_, (Uuid, i64, i64, bool, bool, Value)>(
            component_state_snapshot_history_sql().as_str(),
        )
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        if history_rows.is_empty() {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        let history_commit_ids = history_rows
            .iter()
            .map(|(source_commit_id, _, _, _, _, _)| *source_commit_id)
            .collect::<Vec<_>>();
        let revision_rows = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                Option<String>,
                String,
                String,
                DateTime<Utc>,
                String,
            ),
        >(COMPONENT_CONTENT_REVISIONS_FOR_COMMITS_SQL)
        .bind(context_id.as_uuid())
        .bind(history_commit_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        let mut revisions_by_commit = BTreeMap::<Uuid, Vec<ComponentContentRevision>>::new();
        for (
            revision_commit_id,
            revision_component_id,
            previous_content_hash,
            resulting_content_hash,
            content,
            captured_at,
            kind,
        ) in revision_rows
        {
            let revision = component_state_revision_from_row(ComponentStateRevisionRow {
                context_id,
                commit_id: CommitId::from_uuid(revision_commit_id),
                component_id: contextlab_context_core::ComponentId::from_uuid(
                    revision_component_id,
                ),
                previous_content_hash,
                resulting_content_hash: Some(resulting_content_hash),
                content: Some(content),
                captured_at: Some(captured_at),
                kind: Some(kind),
            })?
            .expect("complete revision query returns a revision");
            revisions_by_commit
                .entry(revision_commit_id)
                .or_default()
                .push(revision);
        }

        let mut steps = Vec::with_capacity(history_rows.len());
        for (source_commit_id, _depth, parent_count, cycle_detected, invalid_parent, changes) in
            history_rows
        {
            validate_component_state_history_row(parent_count, cycle_detected, invalid_parent)?;
            let changes = serde_json::from_value::<Vec<ContextChange>>(changes).map_err(|_| {
                component_state_conflict(
                    "Context component state replay encountered invalid stored commit changes",
                )
            })?;
            steps.push(ComponentStateReplayStep::new(
                CommitId::from_uuid(source_commit_id),
                changes,
                revisions_by_commit
                    .remove(&source_commit_id)
                    .unwrap_or_default(),
            ));
        }
        steps.reverse();

        replay_context_component_state_snapshot(context_id, commit_id, steps)
    }
}

#[async_trait]
impl ContextReplayStateAtCommitRepository for PostgresContextGraphRepository {
    async fn get_context_replay_state_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<contextlab_versioning::ReplayState, StorageRepositoryError> {
        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(database_error)?
            .0;
        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let rows = sqlx::query_as::<_, ReplayStateHistoryRow>(&replay_state_history_sql())
            .bind(context_id.as_uuid())
            .bind(commit_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;
        let records = replay_state_records_from_rows(context_id, commit_id, rows)?;

        replay_state_from_records(context_id, records)
    }
}

fn replay_state_records_from_rows(
    context_id: ContextId,
    commit_id: CommitId,
    rows: Vec<ReplayStateHistoryRow>,
) -> Result<Vec<ContextCommitRecord>, StorageRepositoryError> {
    if rows.is_empty() {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("commit:{context_id}/{commit_id}"),
        });
    }

    rows.into_iter()
        .map(
            |(
                id,
                stored_context_id,
                _depth,
                parent_count,
                cycle_detected,
                invalid_parent,
                branch_name,
                message,
                parent_commit_ids,
                changes,
                authored_at,
                created_at,
            )| {
                validate_replay_state_history_row(parent_count, cycle_detected, invalid_parent)?;
                let change_count = changes
                    .as_array()
                    .ok_or_else(|| {
                        component_state_conflict(
                            "replay state encountered a non-array stored change payload",
                        )
                    })?
                    .len();
                let change_count = u32::try_from(change_count).map_err(|_| {
                    component_state_conflict(
                        "replay state stored change count exceeds the supported limit",
                    )
                })?;
                Ok(ContextCommitRecord {
                    id: id.to_string(),
                    context_id: stored_context_id.to_string(),
                    branch_name,
                    message,
                    parent_commit_ids: parent_commit_ids
                        .into_iter()
                        .map(|parent_id| parent_id.to_string())
                        .collect(),
                    changes,
                    change_count,
                    authored_at,
                    created_at,
                })
            },
        )
        .collect()
}

fn validate_replay_state_history_row(
    parent_count: i64,
    cycle_detected: bool,
    invalid_parent: bool,
) -> Result<(), StorageRepositoryError> {
    if cycle_detected {
        return Err(component_state_conflict(
            "replay state encountered a commit ancestry cycle",
        ));
    }
    if invalid_parent {
        return Err(component_state_conflict(
            "replay state encountered a cross-context parent",
        ));
    }
    if parent_count > 1 {
        return Err(component_state_conflict(
            "replay state does not support merge ancestry",
        ));
    }

    Ok(())
}

fn validate_component_state_history_row(
    parent_count: i64,
    cycle_detected: bool,
    invalid_parent: bool,
) -> Result<(), StorageRepositoryError> {
    if cycle_detected {
        return Err(component_state_conflict(
            "component state replay encountered a commit ancestry cycle",
        ));
    }
    if invalid_parent {
        return Err(component_state_conflict(
            "component state replay encountered a cross-context parent",
        ));
    }
    if parent_count > 1 {
        return Err(component_state_conflict(
            "component state replay does not support merge ancestry",
        ));
    }

    Ok(())
}

struct ComponentStateRevisionRow {
    context_id: contextlab_context_core::ContextId,
    commit_id: CommitId,
    component_id: contextlab_context_core::ComponentId,
    previous_content_hash: Option<String>,
    resulting_content_hash: Option<String>,
    content: Option<String>,
    captured_at: Option<DateTime<Utc>>,
    kind: Option<String>,
}

fn component_state_revision_from_row(
    row: ComponentStateRevisionRow,
) -> Result<Option<ComponentContentRevision>, StorageRepositoryError> {
    let ComponentStateRevisionRow {
        context_id,
        commit_id,
        component_id,
        previous_content_hash,
        resulting_content_hash,
        content,
        captured_at,
        kind,
    } = row;
    let Some(resulting_content_hash) = resulting_content_hash else {
        if previous_content_hash.is_none()
            && content.is_none()
            && captured_at.is_none()
            && kind.is_none()
        {
            return Ok(None);
        }
        return Err(component_state_conflict(
            "component state replay encountered a partial immutable revision",
        ));
    };
    let content = content.ok_or_else(|| {
        component_state_conflict("component state replay encountered a partial immutable revision")
    })?;
    let captured_at = captured_at.ok_or_else(|| {
        component_state_conflict("component state replay encountered a partial immutable revision")
    })?;
    let kind = kind.ok_or_else(|| {
        component_state_conflict("component state replay encountered a partial immutable revision")
    })?;
    let component_kind =
        crate::component_content_revision::parse_component_kind_storage_value(&kind)
            .ok_or(StorageRepositoryError::InvalidStoredComponentKind { kind })?;
    let previous_content_hash = previous_content_hash
        .map(contextlab_context_core::ContentHash::new)
        .transpose()
        .map_err(|_| {
            component_state_conflict(
                "component state replay encountered an invalid immutable revision",
            )
        })?;
    let resulting_content_hash = contextlab_context_core::ContentHash::new(resulting_content_hash)
        .map_err(|_| {
            component_state_conflict(
                "component state replay encountered an invalid immutable revision",
            )
        })?;
    let content = contextlab_context_core::ComponentContent::new(content);
    if content.content_hash() != resulting_content_hash {
        return Err(component_state_conflict(
            "component state replay encountered an immutable revision hash mismatch",
        ));
    }

    Ok(Some(ComponentContentRevision::from_persisted(
        PersistedComponentContentRevision {
            context_id,
            commit_id,
            component_id,
            component_kind,
            previous_content_hash,
            content,
            resulting_content_hash,
            captured_at,
        },
    )))
}

async fn verify_component_content_revision(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    revision: &ComponentContentRevisionWrite,
) -> Result<(), StorageRepositoryError> {
    let component =
        sqlx::query_as::<_, (String, String)>(COMPONENT_FOR_CONTENT_REVISION_UPDATE_SQL)
            .bind(context_id)
            .bind(revision.component_id().as_uuid())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(database_error)?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("component:{context_id}/{}", revision.component_id()),
            })?;
    if component.0
        != crate::component_content_revision::component_kind_storage_value(
            revision.component_kind(),
        )
    {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component kind does not match the revision".to_owned(),
        });
    }
    if component.1 != revision.previous_content_hash().as_str() {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component content hash is stale".to_owned(),
        });
    }
    Ok(())
}

async fn verify_component_content_creation(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    creation: &ComponentContentCreationWrite,
) -> Result<(), StorageRepositoryError> {
    let creation_lock_key = format!("component-creation:{}", creation.component().id());
    sqlx::query(GUARDED_COMPONENT_CREATION_ADVISORY_LOCK_SQL)
        .bind(creation_lock_key)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    let existing = sqlx::query_as::<_, (Uuid,)>(COMPONENT_FOR_CONTENT_CREATION_SQL)
        .bind(creation.component().id().as_uuid())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;
    if existing.is_some() {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: format!(
                "component:{} already exists for a guarded creation in context:{context_id}",
                creation.component().id()
            ),
        });
    }

    Ok(())
}

async fn verify_component_descriptor_revision(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    descriptor: &ComponentDescriptorRevisionWrite,
) -> Result<(), StorageRepositoryError> {
    let component =
        sqlx::query_as::<_, (String, String)>(COMPONENT_FOR_CONTENT_REVISION_UPDATE_SQL)
            .bind(context_id)
            .bind(descriptor.component_id().as_uuid())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(database_error)?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("component:{context_id}/{}", descriptor.component_id()),
            })?;
    if component.0
        != crate::component_content_revision::component_kind_storage_value(
            descriptor.component_kind(),
        )
    {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component kind does not match the descriptor revision".to_owned(),
        });
    }
    if component.1 != descriptor.component().content_hash().as_str() {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component content hash is stale for the descriptor revision".to_owned(),
        });
    }
    Ok(())
}

async fn verify_component_removal(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    removal: &crate::ComponentRemovalWrite,
) -> Result<(), StorageRepositoryError> {
    let component =
        sqlx::query_as::<_, (String, String)>(COMPONENT_FOR_CONTENT_REVISION_UPDATE_SQL)
            .bind(context_id)
            .bind(removal.component_id().as_uuid())
            .fetch_optional(&mut **transaction)
            .await
            .map_err(database_error)?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("component:{context_id}/{}", removal.component_id()),
            })?;
    if component.0
        != crate::component_content_revision::component_kind_storage_value(removal.component_kind())
    {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component kind does not match the removal".to_owned(),
        });
    }
    if component.1 != removal.previous_content_hash().as_str() {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component content hash is stale".to_owned(),
        });
    }

    Ok(())
}

async fn insert_component_content_revision(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    commit_id: Uuid,
    revision: ComponentContentRevisionWrite,
) -> Result<(), StorageRepositoryError> {
    let content_hash = revision.resulting_content_hash();
    sqlx::query(INSERT_COMPONENT_CONTENT_REVISION_SQL)
        .bind(context_id)
        .bind(commit_id)
        .bind(revision.component_id().as_uuid())
        .bind(revision.previous_content_hash().as_str())
        .bind(content_hash.as_str())
        .bind(revision.content().as_str())
        .bind(revision.captured_at())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    let updated = sqlx::query(UPDATE_COMPONENT_CONTENT_HASH_SQL)
        .bind(content_hash.as_str())
        .bind(context_id)
        .bind(revision.component_id().as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    if updated.rows_affected() != 1 {
        return Err(StorageRepositoryError::Database {
            message: "component content hash update failed".to_owned(),
        });
    }
    Ok(())
}

async fn update_component_descriptor(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    descriptor: ComponentDescriptorRevisionWrite,
) -> Result<(), StorageRepositoryError> {
    let updated = sqlx::query(UPDATE_COMPONENT_DESCRIPTOR_SQL)
        .bind(descriptor.component().name().as_str())
        .bind(descriptor.metadata().clone())
        .bind(descriptor.captured_at())
        .bind(context_id)
        .bind(descriptor.component_id().as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    if updated.rows_affected() != 1 {
        return Err(StorageRepositoryError::Database {
            message: "component descriptor update failed".to_owned(),
        });
    }
    Ok(())
}

async fn insert_component_content_creation(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    commit_id: Uuid,
    creation: ComponentContentCreationWrite,
) -> Result<(), StorageRepositoryError> {
    let component = creation.component();
    let content_hash = creation.resulting_content_hash();
    let component_id = component.id().as_uuid();
    let component_kind =
        crate::component_content_revision::component_kind_storage_value(component.kind());
    let component_name = component.name().as_str();
    let metadata = creation.metadata().clone();
    let captured_at = creation.captured_at();
    let content = creation.content().as_str().to_owned();

    sqlx::query(INSERT_CONTEXT_COMPONENT_SQL)
        .bind(component_id)
        .bind(context_id)
        .bind(component_kind)
        .bind(component_name)
        .bind(content_hash.as_str())
        .bind(metadata)
        .bind(captured_at)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query(INSERT_COMPONENT_CONTENT_REVISION_SQL)
        .bind(context_id)
        .bind(commit_id)
        .bind(component_id)
        .bind(Option::<String>::None)
        .bind(content_hash.as_str())
        .bind(content)
        .bind(captured_at)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    Ok(())
}

async fn soft_remove_component(
    transaction: &mut Transaction<'_, Postgres>,
    context_id: Uuid,
    removal: crate::ComponentRemovalWrite,
) -> Result<(), StorageRepositoryError> {
    let removed = sqlx::query(SOFT_REMOVE_COMPONENT_SQL)
        .bind(context_id)
        .bind(removal.component_id().as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    if removed.rows_affected() != 1 {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component removal did not update the active projection".to_owned(),
        });
    }

    Ok(())
}

fn invalid_component_content_revision_error() -> StorageRepositoryError {
    StorageRepositoryError::Database {
        message: "stored component content revision is invalid".to_owned(),
    }
}

#[async_trait]
impl ContextComponentRepository for PostgresContextGraphRepository {
    async fn list_components(
        &self,
        context_id: String,
        query: ComponentListQuery,
    ) -> Result<ComponentList, StorageRepositoryError> {
        let context_id = parse_context_id(&context_id)?;
        let search = query.search.clone();
        let kind = query.kind.map(|kind| kind.as_str());
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let total = sqlx::query_as::<_, (i64,)>(COMPONENT_COUNT_BY_CONTEXT_SQL)
            .bind(context_id)
            .bind(search.as_deref())
            .bind(kind)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0 as u64;

        let items_sql = format!(
            r#"
SELECT context_components.id,
       context_components.context_id,
       context_components.kind,
       context_components.name,
       context_components.content_hash,
       context_components.created_at
FROM context_components
WHERE context_components.context_id = $1
  AND context_components.deleted_at IS NULL
  AND ($3::TEXT IS NULL OR context_components.kind = $3)
  AND (
      $2::TEXT IS NULL
      OR context_components.name ILIKE '%' || $2 || '%'
      OR context_components.kind ILIKE '%' || $2 || '%'
      OR context_components.content_hash ILIKE '%' || $2 || '%'
      OR context_components.id::TEXT ILIKE '%' || $2 || '%'
  )
ORDER BY {}
LIMIT $4 OFFSET $5
"#,
            query.sort.order_by_sql()
        );
        let offset = i64::try_from(query.offset()).unwrap_or(i64::MAX);

        let items =
            sqlx::query_as::<_, (Uuid, Uuid, String, String, String, DateTime<Utc>)>(&items_sql)
                .bind(context_id)
                .bind(search.as_deref())
                .bind(kind)
                .bind(i64::from(query.per_page))
                .bind(offset)
                .fetch_all(&mut *transaction)
                .await
                .map_err(database_error)?
                .into_iter()
                .map(|(id, context_id, kind, name, content_hash, created_at)| {
                    let parsed_kind = kind.parse().map_err(|_| {
                        StorageRepositoryError::InvalidStoredComponentKind { kind: kind.clone() }
                    })?;

                    Ok(ComponentListItem {
                        id: id.to_string(),
                        context_id: context_id.to_string(),
                        kind: parsed_kind,
                        name,
                        content_hash,
                        created_at,
                    })
                })
                .collect::<Result<Vec<_>, StorageRepositoryError>>()?;

        transaction.commit().await.map_err(database_error)?;

        Ok(ComponentList::new(items, &query, total))
    }

    async fn get_component(
        &self,
        context_id: String,
        component_id: String,
    ) -> Result<ComponentDetail, StorageRepositoryError> {
        let context_id = parse_context_id(&context_id)?;
        let component_id = parse_component_id(&component_id)?;
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let component = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                String,
                Value,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(COMPONENT_BY_CONTEXT_SQL)
        .bind(context_id)
        .bind(component_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        let Some((id, context_id, kind, name, content_hash, metadata, created_at, updated_at)) =
            component
        else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("component:{context_id}/{component_id}"),
            });
        };

        transaction.commit().await.map_err(database_error)?;

        let parsed_kind =
            kind.parse()
                .map_err(|_| StorageRepositoryError::InvalidStoredComponentKind {
                    kind: kind.clone(),
                })?;

        Ok(ComponentDetail {
            id: id.to_string(),
            context_id: context_id.to_string(),
            kind: parsed_kind,
            name,
            content_hash,
            metadata,
            created_at,
            updated_at,
        })
    }
}

#[async_trait]
impl BenchmarkDefinitionBindingWriter for PostgresContextGraphRepository {
    async fn persist_benchmark_definition_binding(
        &self,
        command: BenchmarkDefinitionBindingCommand,
    ) -> Result<BenchmarkDefinitionBindingWriteResult, StorageRepositoryError> {
        let command = benchmark_definition_command_at_postgres_precision(command)?;
        let (principal, binding, idempotency_key, request_digest) = command.into_parts();
        let project_id = binding.project_id().as_uuid();
        let context_id = binding.context_id().as_uuid();
        let commit_id = binding.context_commit_id().as_uuid();
        let branch_name = binding.branch().as_str();
        let identity_source = principal.identity().source().as_str();
        let principal_id = principal.id().as_str();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let project_exists = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM projects WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(project_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .is_some();
        if !project_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }

        let context_exists = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM contexts WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(project_id)
        .bind(context_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .is_some();
        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let commit_branch = sqlx::query_as::<_, (String,)>(
            "SELECT branch_name FROM context_commits WHERE context_id = $1 AND id = $2 FOR UPDATE",
        )
        .bind(context_id)
        .bind(commit_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;
        let Some((commit_branch,)) = commit_branch else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context_commit:{context_id}/{commit_id}"),
            });
        };
        if commit_branch != branch_name {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("branch:{context_id}/{branch_name}"),
            });
        }

        let direct_role = sqlx::query_as::<_, (String,)>(CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL)
            .bind(context_id)
            .bind(identity_source)
            .bind(principal_id)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(|_| StorageRepositoryError::GuardedWriteAuthorizationUnavailable)?
            .and_then(|(value,)| WorkspaceRole::from_storage(&value));
        let group_write_granted =
            if direct_role.is_none() && !principal.external_groups().is_empty() {
                let external_group_ids = principal
                    .external_groups()
                    .iter()
                    .map(|group| group.as_str().to_owned())
                    .collect::<Vec<_>>();
                sqlx::query_as::<_, (String,)>(CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL)
                    .bind(context_id)
                    .bind(identity_source)
                    .bind(external_group_ids)
                    .fetch_all(&mut *transaction)
                    .await
                    .map_err(|_| StorageRepositoryError::GuardedWriteAuthorizationUnavailable)?
                    .into_iter()
                    .map(|(value,)| {
                        GroupWorkspaceRole::from_storage(&value)
                            .ok_or(StorageRepositoryError::GuardedWriteAuthorizationUnavailable)
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .any(|role| role.allows(ContextPermission::Write))
            } else {
                false
            };
        if !direct_role.is_some_and(|role| role.allows(ContextPermission::Write))
            && !group_write_granted
        {
            return Err(StorageRepositoryError::GuardedWriteForbidden);
        }

        let idempotency_lock_key = format!(
            "benchmark-definition:{identity_source}|{principal_id}|{context_id}|{branch_name}|{}",
            idempotency_key.as_str()
        );
        sqlx::query(GUARDED_IDEMPOTENCY_ADVISORY_LOCK_SQL)
            .bind(idempotency_lock_key)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        let existing_idempotency = sqlx::query_as::<_, (String, Uuid)>(
            r#"
            SELECT request_digest, binding_id
            FROM benchmark_definition_bindings
            WHERE identity_source = $1
              AND principal_id = $2
              AND context_id = $3
              AND branch_name = $4
              AND idempotency_key = $5
            FOR UPDATE
            "#,
        )
        .bind(identity_source)
        .bind(principal_id)
        .bind(context_id)
        .bind(branch_name)
        .bind(idempotency_key.as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;
        if let Some((stored_digest, stored_binding_id)) = existing_idempotency {
            if stored_digest != request_digest.as_str() {
                return Err(StorageRepositoryError::IdempotencyKeyReused {
                    context_id: context_id.to_string(),
                    principal_id: principal_id.to_owned(),
                    idempotency_key: idempotency_key.to_string(),
                });
            }
            let stored = load_benchmark_definition_binding(
                &mut transaction,
                project_id,
                context_id,
                commit_id,
                stored_binding_id,
            )
            .await?
            .ok_or_else(|| StorageRepositoryError::Database {
                message: "benchmark definition idempotency binding is unavailable".to_owned(),
            })?;
            transaction.commit().await.map_err(database_error)?;
            return Ok(BenchmarkDefinitionBindingWriteResult::new(
                stored,
                BenchmarkDefinitionBindingWriteDisposition::Replayed,
            ));
        }

        let actual_branch_head =
            sqlx::query_as::<_, (Option<Uuid>,)>(CONTEXT_BRANCH_FOR_UPDATE_SQL)
                .bind(context_id)
                .bind(branch_name)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(database_error)?
                .and_then(|row| row.0);
        if actual_branch_head != Some(commit_id) {
            return Err(StorageRepositoryError::BranchHeadConflict {
                expected: Some(commit_id.to_string()),
                actual: actual_branch_head.map(|value| value.to_string()),
            });
        }

        if let Some((existing_binding_id,)) = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT binding_id
            FROM benchmark_definition_bindings
            WHERE project_id = $1 AND context_id = $2
              AND context_commit_id = $3 AND suite_id = $4
            FOR UPDATE
            "#,
        )
        .bind(project_id)
        .bind(context_id)
        .bind(commit_id)
        .bind(binding.suite().id().as_uuid())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        {
            return Err(StorageRepositoryError::BenchmarkDefinitionBindingConflict {
                binding_id: existing_binding_id.to_string(),
            });
        }
        if sqlx::query_as::<_, (Uuid,)>(
            "SELECT binding_id FROM benchmark_definition_bindings WHERE project_id = $1 AND binding_id = $2 FOR UPDATE",
        )
        .bind(project_id)
        .bind(binding.id().as_uuid())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .is_some()
        {
            return Err(StorageRepositoryError::BenchmarkDefinitionBindingConflict {
                binding_id: binding.id().to_string(),
            });
        }

        for dataset in binding.datasets() {
            match load_benchmark_dataset(&mut transaction, project_id, dataset.id().as_uuid())
                .await?
            {
                Some(stored) if stored != *dataset => {
                    return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                        definition_kind: "dataset",
                        definition_id: dataset.id().to_string(),
                    });
                }
                Some(_) => {}
                None => {
                    insert_benchmark_dataset(
                        &mut transaction,
                        project_id,
                        dataset,
                        binding.captured_at(),
                    )
                    .await?;
                }
            }
        }
        match load_benchmark_suite(&mut transaction, project_id, binding.suite().id().as_uuid())
            .await?
        {
            Some(stored) if stored != *binding.suite() => {
                return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                    definition_kind: "suite",
                    definition_id: binding.suite().id().to_string(),
                });
            }
            Some(_) => {}
            None => {
                insert_benchmark_suite(
                    &mut transaction,
                    project_id,
                    binding.suite(),
                    binding.captured_at(),
                )
                .await?;
            }
        }

        sqlx::query(
            r#"
            INSERT INTO benchmark_definition_bindings (
                project_id, context_id, context_commit_id, binding_id, suite_id,
                branch_name, schema_version, identity_source, principal_id,
                idempotency_key, request_digest, captured_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(project_id)
        .bind(context_id)
        .bind(commit_id)
        .bind(binding.id().as_uuid())
        .bind(binding.suite().id().as_uuid())
        .bind(branch_name)
        .bind(i16::try_from(binding.schema_version()).map_err(|_| {
            StorageRepositoryError::Database {
                message: "benchmark definition schema version is invalid".to_owned(),
            }
        })?)
        .bind(identity_source)
        .bind(principal_id)
        .bind(idempotency_key.as_str())
        .bind(request_digest.as_str())
        .bind(binding.captured_at())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        transaction.commit().await.map_err(database_error)?;
        Ok(BenchmarkDefinitionBindingWriteResult::new(
            binding,
            BenchmarkDefinitionBindingWriteDisposition::Created,
        ))
    }
}

fn benchmark_definition_command_at_postgres_precision(
    command: BenchmarkDefinitionBindingCommand,
) -> Result<BenchmarkDefinitionBindingCommand, StorageRepositoryError> {
    let captured_at = command.captured_at();
    let normalized_nanos = (captured_at.nanosecond() / 1_000) * 1_000;
    let normalized = captured_at
        .with_nanosecond(normalized_nanos)
        .ok_or_else(|| StorageRepositoryError::Database {
            message: "benchmark definition capture time cannot be normalized".to_owned(),
        })?;
    Ok(command.with_captured_at(normalized))
}

#[async_trait]
impl BenchmarkDefinitionBindingRepository for PostgresContextGraphRepository {
    async fn get_benchmark_definition_binding(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        binding_id: BenchmarkDefinitionBindingId,
    ) -> Result<Option<BenchmarkDefinitionBinding>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let binding = load_benchmark_definition_binding(
            &mut transaction,
            project_id.as_uuid(),
            context_id.as_uuid(),
            context_commit_id.as_uuid(),
            binding_id.as_uuid(),
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(binding)
    }

    async fn list_benchmark_definition_bindings_at_commit(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Vec<BenchmarkDefinitionBinding>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        let rows = sqlx::query_as::<_, (Uuid, Uuid, Uuid, Uuid, Uuid, String, i16, DateTime<Utc>)>(
            r#"
            SELECT project_id, context_id, context_commit_id, binding_id, suite_id,
                   branch_name, schema_version, captured_at
            FROM benchmark_definition_bindings
            WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3
            ORDER BY binding_id
            "#,
        )
        .bind(project_id.as_uuid())
        .bind(context_id.as_uuid())
        .bind(context_commit_id.as_uuid())
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;
        let mut bindings = Vec::with_capacity(rows.len());
        for row in rows {
            bindings.push(load_benchmark_definition_binding_from_row(&mut transaction, row).await?);
        }
        transaction.commit().await.map_err(database_error)?;
        Ok(bindings)
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Writer for PostgresContextGraphRepository {
    async fn persist_benchmark_workspace_projection(
        &self,
        command: PersistBenchmarkWorkspaceProjectionV1,
    ) -> Result<BenchmarkWorkspaceProjectionWriteResult, BenchmarkWorkspaceProjectionPersistenceError>
    {
        let facts = command.source_facts();
        let scope = facts.scope;
        let project_id = scope.project_id().as_uuid();
        let context_id = scope.context_id().as_uuid();
        let context_commit_id = scope.context_commit_id().as_uuid();
        let cohort_id = scope.cohort_id().as_uuid();
        let decision_id = facts.decision_id.as_uuid();
        let receipt_schema_version = i16::try_from(facts.receipt_schema_version)
            .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
        let expected_cases = benchmark_workspace_case_rows(&facts)?;
        let case_count = i32::try_from(expected_cases.len())
            .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(benchmark_workspace_database_error)?;

        for lock_key in [
            format!("benchmark-workspace-cohort:{cohort_id}"),
            format!(
                "benchmark-workspace-decision:{project_id}:{context_id}:{context_commit_id}:{decision_id}"
            ),
        ] {
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                .bind(lock_key)
                .execute(&mut *transaction)
                .await
                .map_err(benchmark_workspace_database_error)?;
        }

        let existing = sqlx::query_as::<_, (Uuid, Uuid, Uuid, Uuid, i16, String, i32)>(
            r#"
            SELECT project_id, context_id, context_commit_id, decision_id,
                   receipt_schema_version, evidence_digest, case_count
            FROM benchmark_workspace_projection_receipts
            WHERE cohort_id = $1
            FOR UPDATE
            "#,
        )
        .bind(cohort_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(benchmark_workspace_database_error)?;

        if let Some(existing) = existing {
            let existing_scope = BenchmarkWorkspaceProjectionReceiptScope::new(
                ProjectId::from_uuid(existing.0),
                ContextId::from_uuid(existing.1),
                CommitId::from_uuid(existing.2),
                scope.cohort_id(),
            );
            let stored =
                load_benchmark_workspace_projection_source(&mut transaction, existing_scope)
                    .await
                    .map_err(|error| match error {
                        BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable {
                            ..
                        } => BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid,
                        other => other,
                    })?;
            if stored != command {
                return Err(
                    BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict {
                        cohort_id: scope.cohort_id(),
                    },
                );
            }
            transaction
                .commit()
                .await
                .map_err(benchmark_workspace_database_error)?;
            return Ok(BenchmarkWorkspaceProjectionWriteResult::new(
                scope,
                BenchmarkWorkspaceProjectionWriteDisposition::Replayed,
            ));
        }

        let decision_cohort = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT cohort_id
            FROM benchmark_workspace_projection_receipts
            WHERE project_id = $1
              AND context_id = $2
              AND context_commit_id = $3
              AND decision_id = $4
            FOR UPDATE
            "#,
        )
        .bind(project_id)
        .bind(context_id)
        .bind(context_commit_id)
        .bind(decision_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(benchmark_workspace_database_error)?;
        if decision_cohort.is_some() {
            return Err(
                BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict {
                    cohort_id: scope.cohort_id(),
                },
            );
        }

        let sealed_evidence_digest = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT evidence.evidence_digest
            FROM benchmark_decision_evidence AS evidence
            INNER JOIN benchmark_decision_seals AS seal
              ON seal.project_id = evidence.project_id
             AND seal.context_id = evidence.context_id
             AND seal.context_commit_id = evidence.context_commit_id
             AND seal.decision_id = evidence.decision_id
            WHERE evidence.project_id = $1
              AND evidence.context_id = $2
              AND evidence.context_commit_id = $3
              AND evidence.decision_id = $4
            FOR SHARE OF evidence, seal
            "#,
        )
        .bind(project_id)
        .bind(context_id)
        .bind(context_commit_id)
        .bind(decision_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(benchmark_workspace_database_error)?;
        let Some((sealed_evidence_digest,)) = sealed_evidence_digest else {
            return Err(BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { scope });
        };
        if sealed_evidence_digest != facts.evidence_digest {
            return Err(
                BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict {
                    cohort_id: scope.cohort_id(),
                },
            );
        }

        sqlx::query(
            r#"
            INSERT INTO benchmark_workspace_projection_receipts (
                project_id,
                context_id,
                context_commit_id,
                cohort_id,
                decision_id,
                receipt_schema_version,
                evidence_digest,
                case_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(project_id)
        .bind(context_id)
        .bind(context_commit_id)
        .bind(cohort_id)
        .bind(decision_id)
        .bind(receipt_schema_version)
        .bind(&facts.evidence_digest)
        .bind(case_count)
        .execute(&mut *transaction)
        .await
        .map_err(benchmark_workspace_database_error)?;

        for (position, dataset_id, case_id, run_id) in expected_cases {
            sqlx::query(
                r#"
                INSERT INTO benchmark_workspace_projection_cases (
                    project_id,
                    context_id,
                    context_commit_id,
                    decision_id,
                    cohort_id,
                    position,
                    dataset_id,
                    case_id,
                    run_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(project_id)
            .bind(context_id)
            .bind(context_commit_id)
            .bind(decision_id)
            .bind(cohort_id)
            .bind(position)
            .bind(dataset_id)
            .bind(case_id)
            .bind(run_id)
            .execute(&mut *transaction)
            .await
            .map_err(benchmark_workspace_database_error)?;
        }

        sqlx::query("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(&mut *transaction)
            .await
            .map_err(benchmark_workspace_database_error)?;
        let stored = load_benchmark_workspace_projection_source(&mut transaction, scope).await?;
        if stored != command {
            return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
        }
        transaction
            .commit()
            .await
            .map_err(benchmark_workspace_database_error)?;

        Ok(BenchmarkWorkspaceProjectionWriteResult::new(
            scope,
            BenchmarkWorkspaceProjectionWriteDisposition::Created,
        ))
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Reader for PostgresContextGraphRepository {
    async fn read_benchmark_workspace_projection(
        &self,
        query: BenchmarkWorkspaceProjectionV1Query,
    ) -> Result<BenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool)
            .await
            .map_err(benchmark_workspace_storage_error)?;
        let revised =
            load_benchmark_workspace_projection_source(&mut transaction, query.revised()).await?;

        let projection = if let Some(baseline_scope) = query.baseline() {
            let baseline =
                load_benchmark_workspace_projection_source(&mut transaction, baseline_scope)
                    .await?;
            revised.project_against(&baseline)?
        } else {
            revised.project()?
        };

        transaction
            .commit()
            .await
            .map_err(benchmark_workspace_database_error)?;
        Ok(projection)
    }

    async fn resolve_benchmark_workspace_projection_scope(
        &self,
        query: BenchmarkWorkspaceProjectionDecisionQuery,
    ) -> Result<
        Option<BenchmarkWorkspaceProjectionReceiptScope>,
        BenchmarkWorkspaceProjectionPersistenceError,
    > {
        let mut transaction = begin_consistent_read_transaction(&self.pool)
            .await
            .map_err(benchmark_workspace_storage_error)?;
        let row = sqlx::query_as::<_, (Uuid,)>(
            r#"
            SELECT cohort_id
            FROM benchmark_workspace_projection_receipts
            WHERE project_id = $1
              AND context_id = $2
              AND context_commit_id = $3
              AND decision_id = $4
            "#,
        )
        .bind(query.project_id().as_uuid())
        .bind(query.context_id().as_uuid())
        .bind(query.context_commit_id().as_uuid())
        .bind(query.decision_id().as_uuid())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(benchmark_workspace_database_error)?;
        transaction
            .commit()
            .await
            .map_err(benchmark_workspace_database_error)?;
        Ok(row.map(|(cohort_id,)| {
            BenchmarkWorkspaceProjectionReceiptScope::new(
                query.project_id(),
                query.context_id(),
                query.context_commit_id(),
                contextlab_evaluation::BenchmarkExecutionCohortId::from_uuid(cohort_id),
            )
        }))
    }
}

#[async_trait]
impl BenchmarkEvidenceWriter for PostgresContextGraphRepository {
    async fn persist_benchmark_evaluation(
        &self,
        command: PersistBenchmarkEvaluationEvidence,
    ) -> Result<BenchmarkEvidenceWriteResult, StorageRepositoryError> {
        let command = benchmark_command_at_postgres_precision(command)?;
        let evidence = command.evidence().clone();
        let project_id = evidence.project_id().as_uuid();
        let context_id = evidence.context_id().as_uuid();
        let commit_id = evidence.context_commit_id().as_uuid();
        let decision_id = evidence.decision_id().as_uuid();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let project_locked = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM projects WHERE id = $1 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(project_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .is_some();
        if !project_locked {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{}", evidence.project_id()),
            });
        }

        let context_locked = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM contexts WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL FOR UPDATE",
        )
        .bind(project_id)
        .bind(context_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .is_some();
        if !context_locked {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let commit_locked = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM context_commits WHERE context_id = $1 AND id = $2 FOR UPDATE",
        )
        .bind(context_id)
        .bind(commit_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .is_some();
        if !commit_locked {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context_commit:{context_id}/{commit_id}"),
            });
        }

        if let Some(stored) = load_benchmark_decision_by_identity(
            &mut transaction,
            project_id,
            context_id,
            commit_id,
            decision_id,
        )
        .await?
        {
            if stored.evidence_digest() == evidence.evidence_digest() {
                transaction.commit().await.map_err(database_error)?;
                return Ok(BenchmarkEvidenceWriteResult::new(
                    BenchmarkEvidenceWriteDisposition::Replayed,
                    stored,
                ));
            }
            return Err(StorageRepositoryError::BenchmarkEvidenceDigestConflict {
                run_id: evidence.decision_id().to_string(),
            });
        }

        if let (Some(idempotency_key), Some(request_digest)) =
            (command.idempotency_key(), command.request_digest())
        {
            let existing = sqlx::query_as::<_, (String, Uuid)>(
                r#"
                SELECT request_digest, decision_id
                FROM benchmark_execution_idempotency
                WHERE project_id = $1
                  AND context_id = $2
                  AND context_commit_id = $3
                  AND idempotency_key = $4
                FOR UPDATE
                "#,
            )
            .bind(project_id)
            .bind(context_id)
            .bind(commit_id)
            .bind(idempotency_key.as_str())
            .fetch_optional(&mut *transaction)
            .await
            .map_err(database_error)?;
            if let Some((stored_digest, stored_decision_id)) = existing {
                if stored_digest != request_digest.as_str() {
                    return Err(StorageRepositoryError::IdempotencyKeyReused {
                        context_id: evidence.context_id().to_string(),
                        principal_id: "benchmark-execution".to_owned(),
                        idempotency_key: idempotency_key.to_string(),
                    });
                }
                let stored = load_benchmark_decision_by_identity(
                    &mut transaction,
                    project_id,
                    context_id,
                    commit_id,
                    stored_decision_id,
                )
                .await?
                .ok_or_else(invalid_stored_benchmark_evidence_error)?;
                transaction.commit().await.map_err(database_error)?;
                return Ok(BenchmarkEvidenceWriteResult::new(
                    BenchmarkEvidenceWriteDisposition::Replayed,
                    stored,
                ));
            }
        }

        let mut missing_dataset_ids = BTreeSet::new();
        for dataset in command.datasets() {
            match load_benchmark_dataset(&mut transaction, project_id, dataset.id().as_uuid())
                .await?
            {
                Some(stored) if stored != *dataset => {
                    return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                        definition_kind: "dataset",
                        definition_id: dataset.id().to_string(),
                    });
                }
                Some(_) => {}
                None => {
                    missing_dataset_ids.insert(dataset.id());
                }
            }
        }

        let suite_missing = match load_benchmark_suite(
            &mut transaction,
            project_id,
            command.suite().id().as_uuid(),
        )
        .await?
        {
            Some(stored) if stored != *command.suite() => {
                return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                    definition_kind: "suite",
                    definition_id: command.suite().id().to_string(),
                });
            }
            Some(_) => false,
            None => true,
        };

        let mut missing_run_ids = BTreeSet::new();
        for run in command.runs() {
            match load_benchmark_run_by_project_context(
                &mut transaction,
                project_id,
                context_id,
                run.id().as_uuid(),
            )
            .await?
            {
                Some((stored_commit_id, stored))
                    if stored_commit_id != commit_id || stored != *run =>
                {
                    return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                        definition_kind: "evaluation_run",
                        definition_id: run.id().as_uuid().to_string(),
                    });
                }
                Some(_) => {}
                None => {
                    missing_run_ids.insert(run.id());
                }
            }
        }

        for dataset in command
            .datasets()
            .iter()
            .filter(|dataset| missing_dataset_ids.contains(&dataset.id()))
        {
            insert_benchmark_dataset(
                &mut transaction,
                project_id,
                dataset,
                evidence.recorded_at(),
            )
            .await?;
        }
        if suite_missing {
            insert_benchmark_suite(
                &mut transaction,
                project_id,
                command.suite(),
                evidence.recorded_at(),
            )
            .await?;
        }
        for run in command
            .runs()
            .iter()
            .filter(|run| missing_run_ids.contains(&run.id()))
        {
            insert_benchmark_run(
                &mut transaction,
                project_id,
                commit_id,
                run,
                evidence.recorded_at(),
            )
            .await?;
        }

        sqlx::query(
            r#"
            INSERT INTO benchmark_decision_evidence (
                project_id, context_id, decision_id, context_commit_id, suite_id,
                evaluator_key, evaluator_version, comparability_fingerprint,
                evidence_digest, status, recorded_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(project_id)
        .bind(context_id)
        .bind(decision_id)
        .bind(commit_id)
        .bind(evidence.suite_id().as_uuid())
        .bind(evidence.comparability().evaluator_key())
        .bind(evidence.comparability().evaluator_version())
        .bind(evidence.comparability().fingerprint())
        .bind(evidence.evidence_digest())
        .bind(regression_decision_status_to_storage(evidence.status()))
        .bind(evidence.recorded_at())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        for (position, run_id) in evidence.run_ids().iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO benchmark_decision_runs (
                    project_id, context_id, context_commit_id, decision_id, run_id, position
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(project_id)
            .bind(context_id)
            .bind(commit_id)
            .bind(decision_id)
            .bind(run_id.as_uuid())
            .bind(benchmark_position(position)?)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        for metric_result in evidence.metric_results() {
            sqlx::query(
                r#"
                INSERT INTO benchmark_decision_metric_results (
                    project_id, context_id, context_commit_id, decision_id, suite_id, metric,
                    observed_value, sample_count, required_sample_count, has_complete_coverage, outcome
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
            )
            .bind(project_id)
            .bind(context_id)
            .bind(commit_id)
            .bind(decision_id)
            .bind(evidence.suite_id().as_uuid())
            .bind(metric_kind_to_storage(metric_result.metric()))
            .bind(metric_result.observed())
            .bind(benchmark_count(metric_result.sample_count())?)
            .bind(benchmark_count(metric_result.required_sample_count())?)
            .bind(metric_result.has_complete_coverage())
            .bind(regression_check_status_to_storage(metric_result.outcome()))
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        let stored = load_benchmark_decision_by_identity(
            &mut transaction,
            project_id,
            context_id,
            commit_id,
            evidence.decision_id().as_uuid(),
        )
        .await?
        .ok_or_else(invalid_stored_benchmark_evidence_error)?;
        if let (Some(idempotency_key), Some(request_digest)) =
            (command.idempotency_key(), command.request_digest())
        {
            sqlx::query(
                r#"
                INSERT INTO benchmark_execution_idempotency (
                    project_id, context_id, context_commit_id,
                    idempotency_key, request_digest, decision_id
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(project_id)
            .bind(context_id)
            .bind(commit_id)
            .bind(idempotency_key.as_str())
            .bind(request_digest.as_str())
            .bind(decision_id)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }
        transaction.commit().await.map_err(database_error)?;

        Ok(BenchmarkEvidenceWriteResult::new(
            BenchmarkEvidenceWriteDisposition::Created,
            stored,
        ))
    }
}

#[async_trait]
impl BenchmarkEvidenceRepository for PostgresContextGraphRepository {
    async fn get_benchmark_execution_idempotency(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        idempotency_key: &IdempotencyKey,
    ) -> Result<
        Option<crate::benchmark_evidence::BenchmarkExecutionIdempotencyReceipt>,
        StorageRepositoryError,
    > {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        ensure_benchmark_project_context_scope(&mut transaction, project_id, context_id).await?;
        let receipt = sqlx::query_as::<_, (String, Uuid)>(
            r#"
            SELECT request_digest, decision_id
            FROM benchmark_execution_idempotency
            WHERE project_id = $1
              AND context_id = $2
              AND context_commit_id = $3
              AND idempotency_key = $4
            "#,
        )
        .bind(project_id.as_uuid())
        .bind(context_id.as_uuid())
        .bind(context_commit_id.as_uuid())
        .bind(idempotency_key.as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?
        .map(|(request_digest, decision_id)| {
            crate::benchmark_evidence::BenchmarkExecutionIdempotencyReceipt::new(
                BenchmarkDecisionId::from_uuid(decision_id),
                RequestDigest::new(request_digest)
                    .expect("database request digest satisfies migration checks"),
            )
        });
        transaction.commit().await.map_err(database_error)?;
        Ok(receipt)
    }

    async fn get_benchmark_dataset(
        &self,
        project_id: ProjectId,
        dataset_id: BenchmarkDatasetId,
    ) -> Result<Option<BenchmarkDataset>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        ensure_benchmark_project_scope(&mut transaction, project_id).await?;
        let dataset =
            load_benchmark_dataset(&mut transaction, project_id.as_uuid(), dataset_id.as_uuid())
                .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(dataset)
    }

    async fn get_benchmark_suite(
        &self,
        project_id: ProjectId,
        suite_id: BenchmarkSuiteId,
    ) -> Result<Option<BenchmarkSuite>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        ensure_benchmark_project_scope(&mut transaction, project_id).await?;
        let suite =
            load_benchmark_suite(&mut transaction, project_id.as_uuid(), suite_id.as_uuid())
                .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(suite)
    }

    async fn get_benchmark_run(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        run_id: EvaluationRunId,
    ) -> Result<Option<EvaluationRun>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        ensure_benchmark_project_context_scope(&mut transaction, project_id, context_id).await?;
        let run = load_benchmark_run_by_identity(
            &mut transaction,
            project_id.as_uuid(),
            context_id.as_uuid(),
            context_commit_id.as_uuid(),
            run_id.as_uuid(),
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(run)
    }

    async fn get_benchmark_decision(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
    ) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        ensure_benchmark_project_context_scope(&mut transaction, project_id, context_id).await?;
        let decision = load_benchmark_decision_by_identity(
            &mut transaction,
            project_id.as_uuid(),
            context_id.as_uuid(),
            context_commit_id.as_uuid(),
            decision_id.as_uuid(),
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(decision)
    }

    async fn get_benchmark_decision_pair(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        baseline: BenchmarkDecisionComparisonScope,
        revised: BenchmarkDecisionComparisonScope,
    ) -> Result<BenchmarkDecisionPair, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        ensure_benchmark_project_context_scope(&mut transaction, project_id, context_id).await?;
        let baseline = load_benchmark_decision_by_identity(
            &mut transaction,
            project_id.as_uuid(),
            context_id.as_uuid(),
            baseline.context_commit_id().as_uuid(),
            baseline.decision_id().as_uuid(),
        )
        .await?;
        let revised = load_benchmark_decision_by_identity(
            &mut transaction,
            project_id.as_uuid(),
            context_id.as_uuid(),
            revised.context_commit_id().as_uuid(),
            revised.decision_id().as_uuid(),
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(BenchmarkDecisionPair::new(baseline, revised))
    }
}

#[async_trait]
impl BenchmarkDecisionDiscoveryRepository for PostgresContextGraphRepository {
    async fn list_benchmark_decisions(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Vec<BenchmarkDecisionDiscoverySummary>, StorageRepositoryError> {
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;
        ensure_benchmark_project_context_scope(&mut transaction, project_id, context_id).await?;
        let rows = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                DateTime<Utc>,
                i64,
                Uuid,
                String,
                i64,
                i32,
            ),
        >(
            r#"
            SELECT evidence.decision_id,
                   evidence.suite_id,
                   suite.name,
                   evidence.status,
                   evidence.recorded_at,
                   COUNT(DISTINCT decision_runs.run_id) AS run_count,
                   dataset.id AS dataset_id,
                   dataset.name AS dataset_name,
                   COUNT(DISTINCT dataset_case.case_id) AS case_count,
                   suite_dataset.position
            FROM benchmark_decision_evidence AS evidence
            INNER JOIN benchmark_decision_seals AS seals
                ON seals.project_id = evidence.project_id
                AND seals.context_id = evidence.context_id
                AND seals.context_commit_id = evidence.context_commit_id
                AND seals.decision_id = evidence.decision_id
            INNER JOIN benchmark_suite_definitions AS suite
                ON suite.project_id = evidence.project_id
                AND suite.id = evidence.suite_id
            INNER JOIN benchmark_suite_datasets AS suite_dataset
                ON suite_dataset.project_id = suite.project_id
                AND suite_dataset.suite_id = suite.id
            INNER JOIN benchmark_dataset_definitions AS dataset
                ON dataset.project_id = suite_dataset.project_id
                AND dataset.id = suite_dataset.dataset_id
            LEFT JOIN benchmark_dataset_cases AS dataset_case
                ON dataset_case.project_id = dataset.project_id
                AND dataset_case.dataset_id = dataset.id
            LEFT JOIN benchmark_decision_runs AS decision_runs
                ON decision_runs.project_id = evidence.project_id
                AND decision_runs.context_id = evidence.context_id
                AND decision_runs.context_commit_id = evidence.context_commit_id
                AND decision_runs.decision_id = evidence.decision_id
            WHERE evidence.project_id = $1
              AND evidence.context_id = $2
              AND evidence.context_commit_id = $3
            GROUP BY evidence.decision_id, evidence.suite_id, suite.name,
                     evidence.status, evidence.recorded_at, dataset.id,
                     dataset.name, suite_dataset.position
            ORDER BY evidence.recorded_at DESC, evidence.decision_id ASC,
                     suite_dataset.position ASC
            "#,
        )
        .bind(project_id.as_uuid())
        .bind(context_id.as_uuid())
        .bind(context_commit_id.as_uuid())
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;
        let mut decisions: Vec<BenchmarkDecisionDiscoverySummary> = Vec::new();
        for (
            decision_id,
            suite_id,
            suite_name,
            status,
            recorded_at,
            run_count,
            dataset_id,
            dataset_name,
            case_count,
            _dataset_position,
        ) in rows
        {
            let dataset = BenchmarkDecisionDatasetSummary::new(
                BenchmarkDatasetId::from_uuid(dataset_id),
                dataset_name,
                usize::try_from(case_count)
                    .map_err(|_| invalid_stored_benchmark_evidence_error())?,
            );
            if let Some(existing) = decisions.iter_mut().find(|summary| {
                summary.decision_id() == BenchmarkDecisionId::from_uuid(decision_id)
            }) {
                existing.push_dataset(dataset);
            } else {
                decisions.push(BenchmarkDecisionDiscoverySummary::new(
                    BenchmarkDecisionDiscoverySummaryInput {
                        project_id,
                        context_id,
                        context_commit_id,
                        decision_id: BenchmarkDecisionId::from_uuid(decision_id),
                        suite: BenchmarkDecisionDiscoverySuiteSummary::new(
                            BenchmarkSuiteId::from_uuid(suite_id),
                            suite_name,
                        ),
                        datasets: vec![dataset],
                        status: regression_decision_status_from_storage(&status)?,
                        recorded_at,
                        run_count: usize::try_from(run_count)
                            .map_err(|_| invalid_stored_benchmark_evidence_error())?,
                    },
                ));
            }
        }
        for decision in &mut decisions {
            decision.sort_datasets();
        }
        transaction.commit().await.map_err(database_error)?;
        Ok(decisions)
    }
}

async fn ensure_benchmark_project_scope(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: ProjectId,
) -> Result<(), StorageRepositoryError> {
    let exists = sqlx::query_as::<_, (bool,)>(PROJECT_EXISTS_SQL)
        .bind(project_id.as_uuid())
        .fetch_one(&mut **transaction)
        .await
        .map_err(database_error)?
        .0;
    if !exists {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("project:{project_id}"),
        });
    }
    Ok(())
}

async fn ensure_benchmark_project_context_scope(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: ProjectId,
    context_id: ContextId,
) -> Result<(), StorageRepositoryError> {
    let exists = sqlx::query_as::<_, (bool,)>(
        "SELECT EXISTS (SELECT 1 FROM contexts WHERE project_id = $1 AND id = $2 AND deleted_at IS NULL)",
    )
        .bind(project_id.as_uuid())
        .bind(context_id.as_uuid())
        .fetch_one(&mut **transaction)
        .await
        .map_err(database_error)?
        .0;
    if !exists {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("project:{project_id}/context:{context_id}"),
        });
    }
    Ok(())
}

type BenchmarkWorkspaceCaseRow = (i32, Uuid, Uuid, Uuid);

fn benchmark_workspace_case_rows(
    facts: &crate::benchmark_workspace_projection::BenchmarkWorkspaceProjectionSourceFacts,
) -> Result<Vec<BenchmarkWorkspaceCaseRow>, BenchmarkWorkspaceProjectionPersistenceError> {
    facts
        .cases
        .iter()
        .enumerate()
        .map(|(position, link)| {
            Ok((
                i32::try_from(position).map_err(|_| {
                    BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid
                })?,
                link.dataset_id,
                link.case_id,
                link.run_id,
            ))
        })
        .collect()
}

async fn load_benchmark_workspace_case_rows(
    transaction: &mut Transaction<'_, Postgres>,
    cohort_id: Uuid,
) -> Result<Vec<BenchmarkWorkspaceCaseRow>, BenchmarkWorkspaceProjectionPersistenceError> {
    sqlx::query_as::<_, BenchmarkWorkspaceCaseRow>(
        r#"
        SELECT position, dataset_id, case_id, run_id
        FROM benchmark_workspace_projection_cases
        WHERE cohort_id = $1
        ORDER BY position
        "#,
    )
    .bind(cohort_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(benchmark_workspace_database_error)
}

async fn load_benchmark_workspace_projection_source(
    transaction: &mut Transaction<'_, Postgres>,
    scope: BenchmarkWorkspaceProjectionReceiptScope,
) -> Result<PersistBenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError> {
    let project_id = scope.project_id().as_uuid();
    let context_id = scope.context_id().as_uuid();
    let context_commit_id = scope.context_commit_id().as_uuid();
    let cohort_id = scope.cohort_id().as_uuid();
    let source = sqlx::query_as::<_, (Uuid, i16, String, i32)>(
        r#"
        SELECT receipt.decision_id,
               receipt.receipt_schema_version,
               receipt.evidence_digest,
               receipt.case_count
        FROM benchmark_workspace_projection_receipts AS receipt
        INNER JOIN benchmark_workspace_projection_receipt_seals AS seal
          ON seal.cohort_id = receipt.cohort_id
        WHERE receipt.project_id = $1
          AND receipt.context_id = $2
          AND receipt.context_commit_id = $3
          AND receipt.cohort_id = $4
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(context_commit_id)
    .bind(cohort_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(benchmark_workspace_database_error)?;
    let Some((decision_id, receipt_schema_version, evidence_digest, case_count)) = source else {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { scope });
    };
    if receipt_schema_version != 1 || case_count <= 0 {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
    }

    let case_rows = load_benchmark_workspace_case_rows(transaction, cohort_id).await?;
    if usize::try_from(case_count).ok() != Some(case_rows.len())
        || case_rows
            .iter()
            .enumerate()
            .any(|(position, row)| i32::try_from(position).ok() != Some(row.0))
    {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
    }

    let evidence = load_benchmark_decision_by_identity(
        transaction,
        project_id,
        context_id,
        context_commit_id,
        decision_id,
    )
    .await
    .map_err(benchmark_workspace_storage_error)?
    .ok_or(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
    if evidence.evidence_digest() != evidence_digest {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
    }

    let suite = load_benchmark_suite(transaction, project_id, evidence.suite_id().as_uuid())
        .await
        .map_err(benchmark_workspace_storage_error)?
        .ok_or(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
    let mut datasets = Vec::with_capacity(evidence.dataset_ids().len());
    for dataset_id in evidence.dataset_ids() {
        datasets.push(
            load_benchmark_dataset(transaction, project_id, dataset_id.as_uuid())
                .await
                .map_err(benchmark_workspace_storage_error)?
                .ok_or(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?,
        );
    }
    let plan = BenchmarkExecutionPlan::new(suite.clone(), datasets.clone())
        .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
    if plan.cases().len() != case_rows.len() {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
    }

    let mut runs = Vec::with_capacity(case_rows.len());
    let mut results = Vec::with_capacity(case_rows.len());
    for (planned_case, (_, dataset_id, case_id, run_id)) in plan.cases().iter().zip(&case_rows) {
        if planned_case.dataset_id().as_uuid() != *dataset_id
            || planned_case.case_id().as_uuid() != *case_id
        {
            return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
        }
        let run = load_benchmark_run_by_identity(
            transaction,
            project_id,
            context_id,
            context_commit_id,
            *run_id,
        )
        .await
        .map_err(benchmark_workspace_storage_error)?
        .ok_or(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
        results.push(
            BenchmarkCaseExecutionResult::new(
                planned_case.dataset_id(),
                planned_case.case_id(),
                run.measurements().to_vec(),
            )
            .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?,
        );
        runs.push(run);
    }

    let first_run = runs
        .first()
        .ok_or(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
    if runs.iter().any(|run| {
        run.model_version() != first_run.model_version()
            || run.temperature().to_bits() != first_run.temperature().to_bits()
            || run.executed_at() != first_run.executed_at()
    }) {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
    }

    let receipt = BenchmarkExecutionReceipt::from_plan(
        &plan,
        decision_id,
        scope.context_id(),
        first_run.model_version(),
        first_run.temperature(),
        first_run.executed_at(),
        evidence.comparability().fingerprint(),
        results,
    )
    .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
    if receipt.cohort_id() != scope.cohort_id()
        || receipt
            .cohort()
            .entries()
            .iter()
            .zip(&case_rows)
            .any(|(entry, row)| entry.run().id().as_uuid() != row.3)
    {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
    }

    let command = PersistBenchmarkWorkspaceProjectionV1::new(evidence, datasets, suite, receipt)
        .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid)?;
    let facts = command.source_facts();
    if facts.scope != scope
        || facts.decision_id.as_uuid() != decision_id
        || facts.evidence_digest != evidence_digest
    {
        return Err(BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid);
    }
    Ok(command)
}

fn benchmark_workspace_database_error(
    _error: sqlx::Error,
) -> BenchmarkWorkspaceProjectionPersistenceError {
    BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable
}

fn benchmark_workspace_storage_error(
    _error: StorageRepositoryError,
) -> BenchmarkWorkspaceProjectionPersistenceError {
    BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable
}

fn benchmark_command_at_postgres_precision(
    command: PersistBenchmarkEvaluationEvidence,
) -> Result<PersistBenchmarkEvaluationEvidence, StorageRepositoryError> {
    let evidence = command.evidence();
    let runs = command
        .runs()
        .iter()
        .map(|run| {
            EvaluationRun::from_persisted(
                run.id(),
                run.context_id(),
                run.model_version(),
                run.temperature(),
                run.measurements().to_vec(),
                postgres_benchmark_timestamp_precision(run.executed_at()),
            )
            .map_err(|_| invalid_stored_benchmark_evidence_error())
        })
        .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
    let evaluation = BenchmarkEvaluation::from_runs(command.suite(), &runs);
    let normalized = PersistBenchmarkEvaluationEvidence::new(
        evidence.decision_id(),
        evidence.project_id(),
        evidence.context_commit_id(),
        command.datasets().to_vec(),
        command.suite().clone(),
        runs,
        evaluation,
        evidence.comparability().evaluator_key(),
        evidence.comparability().evaluator_version(),
        postgres_benchmark_timestamp_precision(evidence.recorded_at()),
    )
    .map_err(|_| invalid_stored_benchmark_evidence_error())?;
    match (command.idempotency_key(), command.request_digest()) {
        (Some(idempotency_key), Some(request_digest)) => {
            Ok(normalized.with_idempotency(idempotency_key.clone(), request_digest.clone()))
        }
        (None, None) => Ok(normalized),
        _ => Err(StorageRepositoryError::Database {
            message: "benchmark execution idempotency contract is incomplete".to_owned(),
        }),
    }
}

fn postgres_benchmark_timestamp_precision(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    timestamp
        .with_nanosecond((timestamp.timestamp_subsec_nanos() / 1_000) * 1_000)
        .expect("a valid UTC timestamp keeps its microsecond-truncated value")
}

async fn insert_benchmark_dataset(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    dataset: &BenchmarkDataset,
    created_at: DateTime<Utc>,
) -> Result<(), StorageRepositoryError> {
    sqlx::query(
        "INSERT INTO benchmark_dataset_definitions (project_id, id, name, created_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(project_id)
    .bind(dataset.id().as_uuid())
    .bind(dataset.name())
    .bind(created_at)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    for (position, case) in dataset.cases().iter().enumerate() {
        let (expected_mode, expected_output) = match case.expected_output() {
            BenchmarkExpectedOutput::Unspecified => ("unspecified", None),
            BenchmarkExpectedOutput::Exact(value) => ("exact", Some(value)),
        };
        sqlx::query(
            r#"
            INSERT INTO benchmark_dataset_cases (
                project_id, dataset_id, case_id, position, name, input,
                expected_mode, expected_output
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(project_id)
        .bind(dataset.id().as_uuid())
        .bind(case.id().as_uuid())
        .bind(benchmark_position(position)?)
        .bind(case.name())
        .bind(case.input())
        .bind(expected_mode)
        .bind(expected_output)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }
    Ok(())
}

async fn insert_benchmark_suite(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    suite: &BenchmarkSuite,
    created_at: DateTime<Utc>,
) -> Result<(), StorageRepositoryError> {
    sqlx::query(
        "INSERT INTO benchmark_suite_definitions (project_id, id, name, created_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(project_id)
    .bind(suite.id().as_uuid())
    .bind(suite.name())
    .bind(created_at)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    for (position, dataset_id) in suite.dataset_ids().iter().enumerate() {
        sqlx::query(
            "INSERT INTO benchmark_suite_datasets (project_id, suite_id, dataset_id, position) VALUES ($1, $2, $3, $4)",
        )
        .bind(project_id)
        .bind(suite.id().as_uuid())
        .bind(dataset_id.as_uuid())
        .bind(benchmark_position(position)?)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }
    for threshold in suite.thresholds() {
        sqlx::query(
            r#"
            INSERT INTO benchmark_suite_thresholds (
                project_id, suite_id, metric, direction, threshold_value
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(project_id)
        .bind(suite.id().as_uuid())
        .bind(metric_kind_to_storage(threshold.metric()))
        .bind(threshold_direction_to_storage(threshold.direction()))
        .bind(threshold.value())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }
    Ok(())
}

async fn insert_benchmark_run(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    context_commit_id: Uuid,
    run: &EvaluationRun,
    created_at: DateTime<Utc>,
) -> Result<(), StorageRepositoryError> {
    sqlx::query(
        r#"
        INSERT INTO benchmark_evaluation_runs (
            project_id, context_id, context_commit_id, run_id, model_version, temperature,
            executed_at, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(project_id)
    .bind(run.context_id().as_uuid())
    .bind(context_commit_id)
    .bind(run.id().as_uuid())
    .bind(run.model_version())
    .bind(run.temperature())
    .bind(run.executed_at())
    .bind(created_at)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    for (position, measurement) in run.measurements().iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO benchmark_run_measurements (
                project_id, context_id, context_commit_id, run_id, position, metric, value
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(project_id)
        .bind(run.context_id().as_uuid())
        .bind(context_commit_id)
        .bind(run.id().as_uuid())
        .bind(benchmark_position(position)?)
        .bind(metric_kind_to_storage(measurement.kind()))
        .bind(measurement.value())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }
    Ok(())
}

async fn load_benchmark_dataset(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    dataset_id: Uuid,
) -> Result<Option<BenchmarkDataset>, StorageRepositoryError> {
    let name = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM benchmark_dataset_definitions WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(dataset_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    let Some((name,)) = name else {
        return Ok(None);
    };
    let rows = sqlx::query_as::<_, (Uuid, String, Value, String, Option<Value>)>(
        r#"
        SELECT case_id, name, input, expected_mode, expected_output
        FROM benchmark_dataset_cases
        WHERE project_id = $1 AND dataset_id = $2
        ORDER BY position
        "#,
    )
    .bind(project_id)
    .bind(dataset_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;
    let cases = rows
        .into_iter()
        .map(
            |(case_id, case_name, input, expected_mode, expected_output)| {
                let expected_output = match (expected_mode.as_str(), expected_output) {
                    ("unspecified", None) => BenchmarkExpectedOutput::Unspecified,
                    ("exact", Some(value)) => BenchmarkExpectedOutput::Exact(value),
                    _ => return Err(invalid_stored_benchmark_evidence_error()),
                };
                BenchmarkCase::with_id(
                    BenchmarkCaseId::from_uuid(case_id),
                    case_name,
                    input,
                    expected_output,
                )
                .map_err(|_| invalid_stored_benchmark_evidence_error())
            },
        )
        .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
    BenchmarkDataset::with_id(BenchmarkDatasetId::from_uuid(dataset_id), name, cases)
        .map(Some)
        .map_err(|_| invalid_stored_benchmark_evidence_error())
}

async fn load_benchmark_suite(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    suite_id: Uuid,
) -> Result<Option<BenchmarkSuite>, StorageRepositoryError> {
    let name = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM benchmark_suite_definitions WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(suite_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    let Some((name,)) = name else {
        return Ok(None);
    };
    let dataset_ids = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT dataset_id
        FROM benchmark_suite_datasets
        WHERE project_id = $1 AND suite_id = $2
        ORDER BY position
        "#,
    )
    .bind(project_id)
    .bind(suite_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?
    .into_iter()
    .map(|(dataset_id,)| BenchmarkDatasetId::from_uuid(dataset_id))
    .collect::<Vec<_>>();
    let thresholds = sqlx::query_as::<_, (String, String, f64)>(
        r#"
        SELECT metric, direction, threshold_value
        FROM benchmark_suite_thresholds
        WHERE project_id = $1 AND suite_id = $2
        ORDER BY metric
        "#,
    )
    .bind(project_id)
    .bind(suite_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?
    .into_iter()
    .map(|(metric, direction, value)| {
        RegressionThreshold::new(
            metric_kind_from_storage(&metric)?,
            threshold_direction_from_storage(&direction)?,
            value,
        )
        .map_err(|_| invalid_stored_benchmark_evidence_error())
    })
    .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
    BenchmarkSuite::with_id(
        BenchmarkSuiteId::from_uuid(suite_id),
        name,
        dataset_ids,
        thresholds,
    )
    .map(Some)
    .map_err(|_| invalid_stored_benchmark_evidence_error())
}

type StoredBenchmarkDefinitionBindingRow =
    (Uuid, Uuid, Uuid, Uuid, Uuid, String, i16, DateTime<Utc>);

async fn load_benchmark_definition_binding(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    context_id: Uuid,
    context_commit_id: Uuid,
    binding_id: Uuid,
) -> Result<Option<BenchmarkDefinitionBinding>, StorageRepositoryError> {
    let row = sqlx::query_as::<_, StoredBenchmarkDefinitionBindingRow>(
        r#"
        SELECT project_id, context_id, context_commit_id, binding_id, suite_id,
               branch_name, schema_version, captured_at
        FROM benchmark_definition_bindings
        WHERE project_id = $1
          AND context_id = $2
          AND context_commit_id = $3
          AND binding_id = $4
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(context_commit_id)
    .bind(binding_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(
        load_benchmark_definition_binding_from_row(transaction, row).await?,
    ))
}

async fn load_benchmark_definition_binding_from_row(
    transaction: &mut Transaction<'_, Postgres>,
    row: StoredBenchmarkDefinitionBindingRow,
) -> Result<BenchmarkDefinitionBinding, StorageRepositoryError> {
    let (
        project_id,
        context_id,
        context_commit_id,
        binding_id,
        suite_id,
        branch_name,
        schema_version,
        captured_at,
    ) = row;
    let schema_version =
        u16::try_from(schema_version).map_err(|_| StorageRepositoryError::Database {
            message: "stored benchmark definition schema version is invalid".to_owned(),
        })?;
    let branch = BranchName::new(branch_name).map_err(|_| StorageRepositoryError::Database {
        message: "stored benchmark definition branch is invalid".to_owned(),
    })?;
    let suite = load_benchmark_suite(transaction, project_id, suite_id)
        .await?
        .ok_or_else(invalid_stored_benchmark_evidence_error)?;
    let mut datasets = Vec::with_capacity(suite.dataset_ids().len());
    for dataset_id in suite.dataset_ids() {
        datasets.push(
            load_benchmark_dataset(transaction, project_id, dataset_id.as_uuid())
                .await?
                .ok_or_else(invalid_stored_benchmark_evidence_error)?,
        );
    }
    BenchmarkDefinitionBinding::new(
        BenchmarkDefinitionBindingId::from_uuid(binding_id),
        ProjectId::from_uuid(project_id),
        ContextId::from_uuid(context_id),
        CommitId::from_uuid(context_commit_id),
        branch,
        schema_version,
        datasets,
        suite,
        captured_at,
    )
    .map_err(|_| invalid_stored_benchmark_evidence_error())
}

type StoredBenchmarkRunRow = (Uuid, Uuid, Uuid, Uuid, String, f32, DateTime<Utc>);

async fn load_benchmark_run_by_identity(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    context_id: Uuid,
    context_commit_id: Uuid,
    run_id: Uuid,
) -> Result<Option<EvaluationRun>, StorageRepositoryError> {
    let row = sqlx::query_as::<_, StoredBenchmarkRunRow>(
        r#"
        SELECT project_id, context_id, context_commit_id, run_id, model_version, temperature, executed_at
        FROM benchmark_evaluation_runs
        WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND run_id = $4
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(context_commit_id)
    .bind(run_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    Ok(load_benchmark_run_from_row(transaction, row)
        .await?
        .map(|(_, run)| run))
}

async fn load_benchmark_run_by_project_context(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    context_id: Uuid,
    run_id: Uuid,
) -> Result<Option<(Uuid, EvaluationRun)>, StorageRepositoryError> {
    let row = sqlx::query_as::<_, StoredBenchmarkRunRow>(
        r#"
        SELECT project_id, context_id, context_commit_id, run_id, model_version, temperature, executed_at
        FROM benchmark_evaluation_runs
        WHERE project_id = $1 AND context_id = $2 AND run_id = $3
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(run_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    load_benchmark_run_from_row(transaction, row).await
}

async fn load_benchmark_run_from_row(
    transaction: &mut Transaction<'_, Postgres>,
    row: Option<StoredBenchmarkRunRow>,
) -> Result<Option<(Uuid, EvaluationRun)>, StorageRepositoryError> {
    let Some((
        project_id,
        context_id,
        context_commit_id,
        run_id,
        model_version,
        temperature,
        executed_at,
    )) = row
    else {
        return Ok(None);
    };
    let measurements = sqlx::query_as::<_, (Uuid, String, f64)>(
        r#"
        SELECT context_id, metric, value
        FROM benchmark_run_measurements
        WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND run_id = $4
        ORDER BY position
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(context_commit_id)
    .bind(run_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?
    .into_iter()
    .map(|(stored_context_id, metric, value)| {
        if stored_context_id != context_id {
            return Err(invalid_stored_benchmark_evidence_error());
        }
        MetricMeasurement::new(metric_kind_from_storage(&metric)?, value)
            .map_err(|_| invalid_stored_benchmark_evidence_error())
    })
    .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
    let run = EvaluationRun::from_persisted(
        EvaluationRunId::from_uuid(run_id),
        ContextId::from_uuid(context_id),
        model_version,
        temperature,
        measurements,
        executed_at,
    )
    .map_err(|_| invalid_stored_benchmark_evidence_error())?;
    Ok(Some((context_commit_id, run)))
}

type StoredBenchmarkDecisionRow = (
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    String,
    String,
    String,
    String,
    String,
    DateTime<Utc>,
);

async fn load_benchmark_decision_by_identity(
    transaction: &mut Transaction<'_, Postgres>,
    project_id: Uuid,
    context_id: Uuid,
    context_commit_id: Uuid,
    decision_id: Uuid,
) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError> {
    let row = sqlx::query_as::<_, StoredBenchmarkDecisionRow>(
        r#"
        SELECT project_id, context_id, decision_id, context_commit_id, suite_id,
               evaluator_key, evaluator_version, comparability_fingerprint,
               evidence_digest, status, recorded_at
        FROM benchmark_decision_evidence
        WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND decision_id = $4
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(context_commit_id)
    .bind(decision_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;
    load_benchmark_decision_from_row(transaction, row).await
}

async fn load_benchmark_decision_from_row(
    transaction: &mut Transaction<'_, Postgres>,
    row: Option<StoredBenchmarkDecisionRow>,
) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError> {
    let Some((
        project_id,
        context_id,
        decision_id,
        context_commit_id,
        suite_id,
        evaluator_key,
        evaluator_version,
        comparability_fingerprint,
        evidence_digest,
        status,
        recorded_at,
    )) = row
    else {
        return Ok(None);
    };
    let suite = load_benchmark_suite(transaction, project_id, suite_id)
        .await?
        .ok_or_else(invalid_stored_benchmark_evidence_error)?;
    let mut datasets = Vec::with_capacity(suite.dataset_ids().len());
    for dataset_id in suite.dataset_ids() {
        datasets.push(
            load_benchmark_dataset(transaction, project_id, dataset_id.as_uuid())
                .await?
                .ok_or_else(invalid_stored_benchmark_evidence_error)?,
        );
    }
    let run_ids = sqlx::query_as::<_, (Uuid, Uuid, Uuid)>(
        r#"
        SELECT context_id, context_commit_id, run_id
        FROM benchmark_decision_runs
        WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND decision_id = $4
        ORDER BY position
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(context_commit_id)
    .bind(decision_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;
    let mut runs = Vec::with_capacity(run_ids.len());
    for (stored_context_id, stored_context_commit_id, run_id) in run_ids {
        if stored_context_id != context_id || stored_context_commit_id != context_commit_id {
            return Err(invalid_stored_benchmark_evidence_error());
        }
        runs.push(
            load_benchmark_run_by_identity(
                transaction,
                project_id,
                context_id,
                context_commit_id,
                run_id,
            )
            .await?
            .ok_or_else(invalid_stored_benchmark_evidence_error)?,
        );
    }
    let first_run = runs
        .first()
        .ok_or_else(invalid_stored_benchmark_evidence_error)?;
    let comparability = BenchmarkComparability::from_persisted(
        ProjectId::from_uuid(project_id),
        ContextId::from_uuid(context_id),
        BenchmarkSuiteId::from_uuid(suite_id),
        suite.dataset_ids(),
        evaluator_key,
        evaluator_version,
        first_run.model_version(),
        first_run.temperature(),
        comparability_fingerprint,
    )
    .map_err(|_| invalid_stored_benchmark_evidence_error())?;
    let metric_rows = sqlx::query_as::<
        _,
        (
            Uuid,
            Uuid,
            Uuid,
            String,
            Option<f64>,
            i64,
            i64,
            bool,
            String,
        ),
    >(
        r#"
        SELECT context_id, context_commit_id, suite_id, metric, observed_value,
               sample_count, required_sample_count, has_complete_coverage, outcome
        FROM benchmark_decision_metric_results
        WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3 AND decision_id = $4
        ORDER BY metric
        "#,
    )
    .bind(project_id)
    .bind(context_id)
    .bind(context_commit_id)
    .bind(decision_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;
    let metric_results = metric_rows
        .into_iter()
        .map(|row| {
            let (
                stored_context_id,
                stored_context_commit_id,
                stored_suite_id,
                metric,
                observed,
                sample_count,
                required_sample_count,
                has_complete_coverage,
                outcome,
            ) = row;
            if stored_context_id != context_id
                || stored_context_commit_id != context_commit_id
                || stored_suite_id != suite_id
            {
                return Err(invalid_stored_benchmark_evidence_error());
            }
            let metric = metric_kind_from_storage(&metric)?;
            let threshold = suite
                .thresholds()
                .iter()
                .find(|threshold| threshold.metric() == metric)
                .copied()
                .ok_or_else(invalid_stored_benchmark_evidence_error)?;
            BenchmarkMetricDecisionEvidence::from_persisted_with_coverage(
                metric,
                threshold,
                observed,
                benchmark_usize(sample_count)?,
                benchmark_usize(required_sample_count)?,
                has_complete_coverage,
                regression_check_status_from_storage(&outcome)?,
            )
            .map_err(|_| invalid_stored_benchmark_evidence_error())
        })
        .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
    BenchmarkDecisionEvidence::from_persisted(
        BenchmarkDecisionId::from_uuid(decision_id),
        ProjectId::from_uuid(project_id),
        ContextId::from_uuid(context_id),
        CommitId::from_uuid(context_commit_id),
        datasets,
        suite,
        runs,
        comparability,
        evidence_digest,
        regression_decision_status_from_storage(&status)?,
        metric_results,
        recorded_at,
    )
    .map(Some)
    .map_err(|_| invalid_stored_benchmark_evidence_error())
}

fn benchmark_position(position: usize) -> Result<i32, StorageRepositoryError> {
    i32::try_from(position).map_err(|_| StorageRepositoryError::Database {
        message: "benchmark evidence exceeds storage position limit".to_owned(),
    })
}

fn benchmark_count(count: usize) -> Result<i64, StorageRepositoryError> {
    i64::try_from(count).map_err(|_| StorageRepositoryError::Database {
        message: "benchmark evidence exceeds storage count limit".to_owned(),
    })
}

fn benchmark_usize(count: i64) -> Result<usize, StorageRepositoryError> {
    usize::try_from(count).map_err(|_| invalid_stored_benchmark_evidence_error())
}

const fn metric_kind_to_storage(metric: MetricKind) -> &'static str {
    match metric {
        MetricKind::LatencyMs => "latency_ms",
        MetricKind::CostUsd => "cost_usd",
        MetricKind::Accuracy => "accuracy",
        MetricKind::HallucinationRate => "hallucination_rate",
        MetricKind::ToolUsageCount => "tool_usage_count",
        MetricKind::TokenCount => "token_count",
        MetricKind::ExecutionTimeMs => "execution_time_ms",
        MetricKind::OutputQuality => "output_quality",
        MetricKind::SuccessRate => "success_rate",
    }
}

fn metric_kind_from_storage(value: &str) -> Result<MetricKind, StorageRepositoryError> {
    match value {
        "latency_ms" => Ok(MetricKind::LatencyMs),
        "cost_usd" => Ok(MetricKind::CostUsd),
        "accuracy" => Ok(MetricKind::Accuracy),
        "hallucination_rate" => Ok(MetricKind::HallucinationRate),
        "tool_usage_count" => Ok(MetricKind::ToolUsageCount),
        "token_count" => Ok(MetricKind::TokenCount),
        "execution_time_ms" => Ok(MetricKind::ExecutionTimeMs),
        "output_quality" => Ok(MetricKind::OutputQuality),
        "success_rate" => Ok(MetricKind::SuccessRate),
        _ => Err(invalid_stored_benchmark_evidence_error()),
    }
}

const fn threshold_direction_to_storage(direction: ThresholdDirection) -> &'static str {
    match direction {
        ThresholdDirection::Minimum => "minimum",
        ThresholdDirection::Maximum => "maximum",
    }
}

fn threshold_direction_from_storage(
    value: &str,
) -> Result<ThresholdDirection, StorageRepositoryError> {
    match value {
        "minimum" => Ok(ThresholdDirection::Minimum),
        "maximum" => Ok(ThresholdDirection::Maximum),
        _ => Err(invalid_stored_benchmark_evidence_error()),
    }
}

const fn regression_decision_status_to_storage(status: RegressionDecisionStatus) -> &'static str {
    match status {
        RegressionDecisionStatus::Passed => "passed",
        RegressionDecisionStatus::Regressed => "regressed",
        RegressionDecisionStatus::InsufficientData => "insufficient_data",
    }
}

fn regression_decision_status_from_storage(
    value: &str,
) -> Result<RegressionDecisionStatus, StorageRepositoryError> {
    match value {
        "passed" => Ok(RegressionDecisionStatus::Passed),
        "regressed" => Ok(RegressionDecisionStatus::Regressed),
        "insufficient_data" => Ok(RegressionDecisionStatus::InsufficientData),
        _ => Err(invalid_stored_benchmark_evidence_error()),
    }
}

const fn regression_check_status_to_storage(status: RegressionCheckStatus) -> &'static str {
    match status {
        RegressionCheckStatus::Passed => "passed",
        RegressionCheckStatus::Regressed => "regressed",
        RegressionCheckStatus::InsufficientData => "insufficient_data",
    }
}

fn regression_check_status_from_storage(
    value: &str,
) -> Result<RegressionCheckStatus, StorageRepositoryError> {
    match value {
        "passed" => Ok(RegressionCheckStatus::Passed),
        "regressed" => Ok(RegressionCheckStatus::Regressed),
        "insufficient_data" => Ok(RegressionCheckStatus::InsufficientData),
        _ => Err(invalid_stored_benchmark_evidence_error()),
    }
}

fn invalid_stored_benchmark_evidence_error() -> StorageRepositoryError {
    StorageRepositoryError::Database {
        message: "stored benchmark evidence is invalid".to_owned(),
    }
}

#[async_trait]
impl EvaluationRunRepository for PostgresContextGraphRepository {
    async fn list_evaluation_runs(
        &self,
        context_id: String,
        query: EvaluationRunListQuery,
    ) -> Result<EvaluationRunList, StorageRepositoryError> {
        let context_id = parse_context_id(&context_id)?;
        let search = query.search.clone();
        let suite_name = query.suite_name.clone();
        let model_version = query.model_version.clone();
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let total = sqlx::query_as::<_, (i64,)>(EVALUATION_RUN_COUNT_BY_CONTEXT_SQL)
            .bind(context_id)
            .bind(search.as_deref())
            .bind(suite_name.as_deref())
            .bind(model_version.as_deref())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0 as u64;

        let items_sql =
            EVALUATION_RUN_LIST_ITEMS_SQL_TEMPLATE.replace("{order_by}", query.sort.order_by_sql());
        let offset = i64::try_from(query.offset()).unwrap_or(i64::MAX);

        let items = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                f32,
                i32,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(&items_sql)
        .bind(context_id)
        .bind(search.as_deref())
        .bind(suite_name.as_deref())
        .bind(model_version.as_deref())
        .bind(i64::from(query.per_page))
        .bind(offset)
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?
        .into_iter()
        .map(
            |(
                id,
                context_id,
                suite_name,
                model_version,
                temperature,
                metric_count,
                executed_at,
                created_at,
            )| EvaluationRunListItem {
                id: id.to_string(),
                context_id: context_id.to_string(),
                suite_name,
                model_version,
                temperature,
                metric_count: u32::try_from(metric_count).unwrap_or(0),
                executed_at,
                created_at,
            },
        )
        .collect();

        transaction.commit().await.map_err(database_error)?;

        Ok(EvaluationRunList::new(items, &query, total))
    }

    async fn get_evaluation_run(
        &self,
        context_id: String,
        run_id: String,
    ) -> Result<EvaluationRunDetail, StorageRepositoryError> {
        let context_id = parse_context_id(&context_id)?;
        let run_id = parse_evaluation_run_id(&run_id)?;
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let run = sqlx::query_as::<
            _,
            (
                Uuid,
                Uuid,
                String,
                String,
                f32,
                i32,
                Value,
                DateTime<Utc>,
                DateTime<Utc>,
            ),
        >(EVALUATION_RUN_BY_CONTEXT_SQL)
        .bind(context_id)
        .bind(run_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        let Some((
            id,
            context_id,
            suite_name,
            model_version,
            temperature,
            metric_count,
            metrics,
            executed_at,
            created_at,
        )) = run
        else {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("evaluation_run:{context_id}/{run_id}"),
            });
        };

        transaction.commit().await.map_err(database_error)?;

        Ok(EvaluationRunDetail {
            id: id.to_string(),
            context_id: context_id.to_string(),
            suite_name,
            model_version,
            temperature,
            metric_count: u32::try_from(metric_count).unwrap_or(0),
            metrics,
            executed_at,
            created_at,
        })
    }

    async fn get_evaluation_scorecard(
        &self,
        context_id: String,
        query: EvaluationScorecardQuery,
    ) -> Result<EvaluationScorecard, StorageRepositoryError> {
        let raw_context_id = context_id.clone();
        let context_id = parse_context_id(&context_id)?;
        let search = query.search.clone();
        let suite_name = query.suite_name.clone();
        let model_version = query.model_version.clone();
        let mut transaction = begin_consistent_read_transaction(&self.pool).await?;

        let context_exists = sqlx::query_as::<_, (bool,)>(CONTEXT_EXISTS_SQL)
            .bind(context_id)
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_error)?
            .0;

        if !context_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let metric_payloads = sqlx::query_as::<_, (Value,)>(EVALUATION_SCORECARD_RUNS_SQL)
            .bind(context_id)
            .bind(search.as_deref())
            .bind(suite_name.as_deref())
            .bind(model_version.as_deref())
            .fetch_all(&mut *transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        Ok(build_evaluation_scorecard(
            raw_context_id,
            metric_payloads.iter().map(|(metrics,)| metrics),
        ))
    }
}

fn build_evaluation_scorecard<'a>(
    context_id: String,
    metrics: impl Iterator<Item = &'a Value>,
) -> EvaluationScorecard {
    let mut totals = BTreeMap::<String, (f64, u32)>::new();
    let mut run_count = 0_u64;

    for metric_payload in metrics {
        run_count += 1;

        let Some(metric_object) = metric_payload.as_object() else {
            continue;
        };

        for (name, value) in metric_object {
            let Some(value) = value.as_f64().filter(|value| value.is_finite()) else {
                continue;
            };
            let entry = totals.entry(name.clone()).or_insert((0.0, 0));
            entry.0 += value;
            entry.1 = entry.1.saturating_add(1);
        }
    }

    let metrics = totals
        .into_iter()
        .map(|(name, (total, sample_count))| EvaluationScorecardMetric {
            name,
            average: total / f64::from(sample_count),
            sample_count,
        })
        .collect();

    EvaluationScorecard {
        context_id,
        run_count,
        metrics,
    }
}

fn materialize_commit_graph_snapshot(
    scope: CommitGraphSnapshotScope,
    row: (Option<i16>, Option<Value>, Option<DateTime<Utc>>),
) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
    let (schema_version, graph, captured_at) = row;
    match (schema_version, graph, captured_at) {
        (Some(schema_version), Some(graph), Some(captured_at)) => {
            let schema_version =
                decode_commit_graph_snapshot_schema_version(scope, schema_version)?;
            CommitGraphSnapshot::from_graph_payload(scope, graph, captured_at, schema_version)
                .map(Some)
                .map_err(|_| invalid_stored_commit_graph_snapshot(scope))
        }
        (None, None, None) => Ok(None),
        _ => Err(invalid_stored_commit_graph_snapshot(scope)),
    }
}

fn decode_commit_graph_snapshot_schema_version(
    scope: CommitGraphSnapshotScope,
    schema_version: i16,
) -> Result<u16, StorageRepositoryError> {
    let schema_version = u16::try_from(schema_version)
        .ok()
        .filter(|schema_version| *schema_version == COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1)
        .ok_or_else(|| invalid_stored_commit_graph_snapshot(scope))?;
    Ok(schema_version)
}

fn validate_commit_graph_snapshot_write_scope(
    project_id: ProjectId,
    commit: &ContextCommit,
    snapshot: &CommitGraphSnapshot,
) -> Result<(), StorageRepositoryError> {
    let expected_scope =
        CommitGraphSnapshotScope::new(project_id, commit.context_id(), commit.id());
    if snapshot.scope() != expected_scope
        || snapshot.schema_version() != COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1
    {
        return Err(invalid_stored_commit_graph_snapshot(expected_scope));
    }
    Ok(())
}

fn invalid_stored_commit_graph_snapshot(scope: CommitGraphSnapshotScope) -> StorageRepositoryError {
    StorageRepositoryError::Database {
        message: format!("stored Context Graph snapshot is invalid for {scope}",),
    }
}

fn parse_workspace_id(workspace_id: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(workspace_id).map_err(|error| StorageRepositoryError::InvalidScope {
        scope: format!("workspace:{workspace_id}"),
        reason: error.to_string(),
    })
}

fn parse_project_id(project_id: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(project_id).map_err(|error| StorageRepositoryError::InvalidScope {
        scope: format!("project:{project_id}"),
        reason: error.to_string(),
    })
}

fn parse_context_id(context_id: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(context_id).map_err(|error| StorageRepositoryError::InvalidScope {
        scope: format!("context:{context_id}"),
        reason: error.to_string(),
    })
}

fn parse_commit_id(commit_id: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(commit_id).map_err(|error| StorageRepositoryError::InvalidScope {
        scope: format!("commit:{commit_id}"),
        reason: error.to_string(),
    })
}

fn parse_component_id(component_id: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(component_id).map_err(|error| StorageRepositoryError::InvalidScope {
        scope: format!("component:{component_id}"),
        reason: error.to_string(),
    })
}

fn parse_evaluation_run_id(run_id: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(run_id).map_err(|error| StorageRepositoryError::InvalidScope {
        scope: format!("evaluation_run:{run_id}"),
        reason: error.to_string(),
    })
}

fn parse_experiment_id(experiment_id: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(experiment_id).map_err(|error| StorageRepositoryError::InvalidScope {
        scope: format!("experiment:{experiment_id}"),
        reason: error.to_string(),
    })
}

async fn begin_consistent_read_transaction(
    pool: &PgPool,
) -> Result<Transaction<'_, Postgres>, StorageRepositoryError> {
    let mut transaction = pool.begin().await.map_err(database_error)?;

    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

    Ok(transaction)
}

fn decode_workflow_context_binding(
    row: (Uuid, Uuid, Uuid, Uuid, i64, Value),
) -> Result<WorkflowContextBinding, StorageRepositoryError> {
    let (binding_id, context_id, commit_id, workflow_id, workflow_revision, payload) = row;
    let workflow_revision =
        u64::try_from(workflow_revision).map_err(|_| StorageRepositoryError::Database {
            message: "stored workflow revision is invalid".to_owned(),
        })?;
    let binding = serde_json::from_value::<WorkflowContextBinding>(payload).map_err(|_| {
        StorageRepositoryError::Database {
            message: "stored workflow Context binding is invalid".to_owned(),
        }
    })?;
    if binding.id() != WorkflowContextBindingId::from_uuid(binding_id)
        || binding.context_source()
            != ContextCommitSource::new(
                ContextId::from_uuid(context_id),
                CommitId::from_uuid(commit_id),
            )
        || binding.workflow_id() != WorkflowId::from_uuid(workflow_id)
        || binding.workflow_revision().get() != workflow_revision
    {
        return Err(StorageRepositoryError::Database {
            message: "stored workflow Context binding columns conflict with payload".to_owned(),
        });
    }
    Ok(binding)
}

fn workflow_context_binding_conflict(
    workflow_id: WorkflowId,
    workflow_revision: WorkflowRevision,
) -> StorageRepositoryError {
    StorageRepositoryError::WorkflowContextBindingConflict {
        workflow_id: workflow_id.as_uuid().to_string(),
        workflow_revision: workflow_revision.get(),
    }
}

fn database_error(error: sqlx::Error) -> StorageRepositoryError {
    let _ = error;
    StorageRepositoryError::Database {
        message: "database query failed".to_owned(),
    }
}

fn database_configuration_error(error: sqlx::Error) -> StorageRepositoryError {
    let _ = error;
    StorageRepositoryError::Database {
        message: "database pool configuration failed".to_owned(),
    }
}

impl PostgresContextGraphRepository {
    async fn ensure_branch_head_context(
        &self,
        context_id: ContextId,
    ) -> Result<(), ContextBranchRepositoryError> {
        let exists = sqlx::query_scalar::<_, bool>(CONTEXT_EXISTS_FOR_BRANCH_HEAD_SQL)
            .bind(context_id.as_uuid())
            .fetch_one(&self.pool)
            .await
            .map_err(|error| ContextBranchRepositoryError::Database {
                message: error.to_string(),
            })?;
        if exists {
            Ok(())
        } else {
            Err(ContextBranchRepositoryError::UnknownContext { context_id })
        }
    }
}

#[async_trait]
impl ContextBranchRepository for PostgresContextGraphRepository {
    async fn list_context_branch_heads(
        &self,
        context_id: ContextId,
    ) -> Result<Vec<ContextBranchHead>, ContextBranchRepositoryError> {
        self.ensure_branch_head_context(context_id).await?;
        let rows = sqlx::query_as::<_, (String, Option<Uuid>, i64, bool)>(CONTEXT_BRANCH_HEADS_SQL)
            .bind(context_id.as_uuid())
            .fetch_all(&self.pool)
            .await
            .map_err(|error| ContextBranchRepositoryError::Database {
                message: error.to_string(),
            })?;

        rows.into_iter()
            .map(
                |(branch, head_commit_id, revision, head_belongs_to_context)| {
                    ContextBranchHead::from_stored(
                        context_id,
                        branch,
                        head_commit_id,
                        revision,
                        head_belongs_to_context,
                    )
                },
            )
            .collect()
    }

    async fn get_context_branch_head(
        &self,
        context_id: ContextId,
        branch: BranchName,
    ) -> Result<ContextBranchHead, ContextBranchRepositoryError> {
        self.ensure_branch_head_context(context_id).await?;
        let row = sqlx::query_as::<_, (String, Option<Uuid>, i64, bool)>(CONTEXT_BRANCH_HEAD_SQL)
            .bind(context_id.as_uuid())
            .bind(branch.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| ContextBranchRepositoryError::Database {
                message: error.to_string(),
            })?
            .ok_or_else(|| ContextBranchRepositoryError::UnknownBranch {
                context_id,
                branch: branch.clone(),
            })?;

        ContextBranchHead::from_stored(context_id, row.0, row.1, row.2, row.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthorizationAuditPurgeExecutor, AuthorizationAuditPurgeRequest,
        BenchmarkCaseEvaluationRequest, BenchmarkCaseEvaluator, BenchmarkCaseEvaluatorError,
        BenchmarkDecisionComparisonScope, BenchmarkDecisionComparisonService, BenchmarkDecisionId,
        BenchmarkDecisionRunDetailsService, BenchmarkDefinitionBindingCommand,
        BenchmarkDefinitionBindingRepository, BenchmarkDefinitionBindingWriteDisposition,
        BenchmarkDefinitionBindingWriter, BenchmarkEvidenceRepository,
        BenchmarkEvidenceWriteDisposition, BenchmarkEvidenceWriter, BenchmarkExecutionDisposition,
        BenchmarkExecutionRequest, BenchmarkExecutionService,
        BenchmarkWorkspaceProjectionReceiptScope, BenchmarkWorkspaceProjectionV1Query,
        BenchmarkWorkspaceProjectionV1Reader, BenchmarkWorkspaceProjectionV1Writer,
        BenchmarkWorkspaceProjectionWriteDisposition, BenchmarkWorkspaceProjectionWriteResult,
        CONTEXT_PLATFORM_MIGRATION, CONTEXTLAB_AUDIT_PURGE_EXECUTOR,
        CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP, ComponentContentCreationWrite,
        ComponentContentRevisionRepository, ComponentContentRevisionWrite,
        ComponentDescriptorRevisionWrite, CreateContextCommitSnapshot,
        GuardedCommitWriteDisposition, GuardedContextCommitWrite, GuardedContextCommitWriter,
        IdempotencyKey, PersistBenchmarkEvaluationEvidence, PersistBenchmarkWorkspaceProjectionV1,
        PostgresAuthorizationAuditPurgeExecutor, PostgresProtectedRouteRateLimiter, RequestDigest,
        StoredComponentKind, WORKSPACE_GRAPH_SEED, WORKSPACE_GRAPH_SEED_WORKSPACE_ID,
    };
    use chrono::{TimeZone, Utc};
    use contextlab_auth::{
        AuthenticatedPrincipal, AuthorizationAuditEvent, AuthorizationAuditSink,
        AuthorizationDecision, ContextAuthorizer, ContextPermission, ContextRoleResolver,
        ExternalGroupId, IdentitySourceId, PrincipalId, PrincipalIdentity, ProtectedRouteOperation,
        ProtectedRouteRateLimitKey, ProtectedRouteRateLimitPolicy, ProtectedRouteRateLimiter,
        RateLimitDecision, RateLimitError, RoleBasedContextAuthorizer, TrustedExternalGroups,
    };
    use contextlab_context_core::{
        ComponentContent, ComponentId, ContentHash, ContextComponent, ContextComponentKind,
        ContextId, ProjectId, WorkspaceId,
    };
    use contextlab_evaluation::{
        BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkDataset, BenchmarkEvaluation,
        BenchmarkExecutionPlan, BenchmarkExecutionReceipt, BenchmarkExpectedOutput, BenchmarkSuite,
        EvaluationRun, MetricKind, MetricMeasurement, RegressionDecisionStatus,
        RegressionThreshold, ThresholdDirection,
    };
    use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
    use contextlab_versioning::{
        BranchName, CommitId, ContextChange, ContextChangeKind, ContextCommit, ExpectedBranchHead,
    };
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
    use std::str::FromStr;
    use std::{
        collections::BTreeSet,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    const PRE_IDENTITY_NAMESPACE_MIGRATION: &str = concat!(
        include_str!("../migrations/0001_context_platform.sql"),
        "\n",
        include_str!("../migrations/0002_context_commit_graph_snapshots.sql"),
        "\n",
        include_str!("../migrations/0003_guarded_context_commit_writes.sql"),
        "\n",
        include_str!("../migrations/0004_guarded_context_commit_scope_constraints.sql"),
        "\n",
        include_str!("../migrations/0005_context_authorization_audit_events.sql"),
    );
    const PRINCIPAL_IDENTITY_NAMESPACE_MIGRATION: &str =
        include_str!("../migrations/0006_principal_identity_namespace.sql");
    const PRE_AUDIT_RETENTION_GOVERNANCE_MIGRATION: &str = concat!(
        include_str!("../migrations/0001_context_platform.sql"),
        "\n",
        include_str!("../migrations/0002_context_commit_graph_snapshots.sql"),
        "\n",
        include_str!("../migrations/0003_guarded_context_commit_writes.sql"),
        "\n",
        include_str!("../migrations/0004_guarded_context_commit_scope_constraints.sql"),
        "\n",
        include_str!("../migrations/0005_context_authorization_audit_events.sql"),
        "\n",
        include_str!("../migrations/0006_principal_identity_namespace.sql"),
        "\n",
        include_str!("../migrations/0007_workspace_external_group_role_bindings.sql"),
    );

    const AUDIT_RETENTION_GOVERNANCE_MIGRATION: &str =
        include_str!("../migrations/0008_context_authorization_audit_governance.sql");
    const MIGRATION_LEDGER_MIGRATION: &str =
        include_str!("../migrations/0009_contextlab_migration_ledger.sql");
    const SHARED_PROTECTED_ROUTE_RATE_LIMIT_MIGRATION: &str =
        include_str!("../migrations/0010_shared_protected_route_rate_limits.sql");
    const SHARED_PROTECTED_ROUTE_RATE_LIMIT_HARDENING_MIGRATION: &str =
        include_str!("../migrations/0011_shared_protected_route_rate_limit_hardening.sql");
    const PRE_INITIAL_COMPONENT_REVISIONS_MIGRATION: &str = concat!(
        include_str!("../migrations/0001_context_platform.sql"),
        "\n",
        include_str!("../migrations/0002_context_commit_graph_snapshots.sql"),
        "\n",
        include_str!("../migrations/0003_guarded_context_commit_writes.sql"),
        "\n",
        include_str!("../migrations/0004_guarded_context_commit_scope_constraints.sql"),
        "\n",
        include_str!("../migrations/0005_context_authorization_audit_events.sql"),
        "\n",
        include_str!("../migrations/0006_principal_identity_namespace.sql"),
        "\n",
        include_str!("../migrations/0007_workspace_external_group_role_bindings.sql"),
        "\n",
        include_str!("../migrations/0008_context_authorization_audit_governance.sql"),
        "\n",
        include_str!("../migrations/0009_contextlab_migration_ledger.sql"),
        "\n",
        include_str!("../migrations/0010_shared_protected_route_rate_limits.sql"),
        "\n",
        include_str!("../migrations/0011_shared_protected_route_rate_limit_hardening.sql"),
        "\n",
        include_str!("../migrations/0012_component_content_revisions.sql"),
    );
    const INITIAL_COMPONENT_REVISIONS_MIGRATION: &str =
        include_str!("../migrations/0013_component_content_initial_revisions.sql");
    const INITIAL_COMPONENT_REVISIONS_INTEGRITY_MIGRATION: &str =
        include_str!("../migrations/0014_component_content_initial_revision_integrity.sql");
    const PARENT_SCOPE_INTEGRITY_MIGRATION: &str =
        include_str!("../migrations/0015_context_commit_parent_scope_integrity.sql");

    async fn unseeded_disposable_test_pool(max_connections: u32) -> Option<sqlx::PgPool> {
        let Some(database_url) = std::env::var("CONTEXTLAB_TEST_DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
        else {
            eprintln!("skipping: CONTEXTLAB_TEST_DATABASE_URL is not configured");
            return None;
        };

        Some(
            PgPoolOptions::new()
                .max_connections(max_connections)
                .connect(&database_url)
                .await
                .expect("connect to disposable test database"),
        )
    }

    async fn disposable_test_pool(max_connections: u32) -> Option<sqlx::PgPool> {
        let pool = unseeded_disposable_test_pool(max_connections).await?;
        sqlx::raw_sql(CONTEXT_PLATFORM_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply migration to empty test database");
        sqlx::raw_sql(WORKSPACE_GRAPH_SEED)
            .execute(&pool)
            .await
            .expect("apply seed fixture");
        sqlx::query(
            "INSERT INTO workspace_memberships (workspace_id, identity_source, principal_id, role) VALUES ($1, $2, $3, $4)",
        )
        .bind(
            WORKSPACE_GRAPH_SEED_WORKSPACE_ID
                .parse::<Uuid>()
                .expect("workspace uuid"),
        )
        .bind("https://issuer.contextlab.test")
        .bind("user:alex")
        .bind("owner")
        .execute(&pool)
        .await
        .expect("insert membership fixture");

        Some(pool)
    }

    async fn apply_parent_scope_predecessor(pool: &sqlx::PgPool) {
        sqlx::raw_sql(PRE_INITIAL_COMPONENT_REVISIONS_MIGRATION)
            .execute(pool)
            .await
            .expect("apply schema through the parent-scope migration predecessor");
        sqlx::raw_sql(INITIAL_COMPONENT_REVISIONS_MIGRATION)
            .execute(pool)
            .await
            .expect("apply nullable-prior migration");
        sqlx::raw_sql(INITIAL_COMPONENT_REVISIONS_INTEGRITY_MIGRATION)
            .execute(pool)
            .await
            .expect("apply initial revision integrity migration");
        sqlx::raw_sql(WORKSPACE_GRAPH_SEED)
            .execute(pool)
            .await
            .expect("seed parent-scope migration predecessor");
    }

    struct PostgresBenchmarkFixture {
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
        dataset: BenchmarkDataset,
        suite: BenchmarkSuite,
        runs: Vec<EvaluationRun>,
        recorded_at: DateTime<Utc>,
    }

    impl PostgresBenchmarkFixture {
        fn command(&self, evaluator_version: &str) -> PersistBenchmarkEvaluationEvidence {
            self.command_with(
                self.decision_id,
                vec![self.dataset.clone()],
                self.suite.clone(),
                self.runs.clone(),
                evaluator_version,
            )
        }

        fn command_with(
            &self,
            decision_id: BenchmarkDecisionId,
            datasets: Vec<BenchmarkDataset>,
            suite: BenchmarkSuite,
            runs: Vec<EvaluationRun>,
            evaluator_version: &str,
        ) -> PersistBenchmarkEvaluationEvidence {
            self.command_with_commit(
                decision_id,
                self.commit_id,
                datasets,
                suite,
                runs,
                evaluator_version,
            )
        }

        fn command_with_commit(
            &self,
            decision_id: BenchmarkDecisionId,
            commit_id: CommitId,
            datasets: Vec<BenchmarkDataset>,
            suite: BenchmarkSuite,
            runs: Vec<EvaluationRun>,
            evaluator_version: &str,
        ) -> PersistBenchmarkEvaluationEvidence {
            let evaluation = BenchmarkEvaluation::from_runs(&suite, &runs);
            PersistBenchmarkEvaluationEvidence::new(
                decision_id,
                self.project_id,
                commit_id,
                datasets,
                suite,
                runs,
                evaluation,
                "contextlab.exact-match",
                evaluator_version,
                self.recorded_at,
            )
            .expect("valid PostgreSQL benchmark command")
        }
    }

    fn postgres_benchmark_fixture() -> PostgresBenchmarkFixture {
        let project_id = ProjectId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project uuid"),
        );
        let context_id = ContextId::from_uuid(
            Uuid::parse_str("44444444-4444-4444-8444-444444444444").expect("context uuid"),
        );
        let dataset = BenchmarkDataset::new(
            "PostgreSQL support cases",
            vec![
                BenchmarkCase::new(
                    "Exact null remains an oracle",
                    serde_json::json!({"question": "unknown"}),
                    BenchmarkExpectedOutput::Exact(Value::Null),
                )
                .expect("benchmark case"),
            ],
        )
        .expect("benchmark dataset");
        let suite = BenchmarkSuite::new(
            "PostgreSQL release gate",
            vec![dataset.id()],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
            ],
        )
        .expect("benchmark suite");
        let executed_at = Utc
            .with_ymd_and_hms(2026, 7, 16, 12, 0, 0)
            .single()
            .expect("timestamp")
            .with_nanosecond(123_456_789)
            .expect("nanosecond timestamp");
        let runs = vec![
            EvaluationRun::new(
                context_id,
                "model-postgres-v1",
                0.2,
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("metric"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 700.0).expect("metric"),
                ],
                executed_at,
            )
            .expect("valid evaluation run"),
            EvaluationRun::new(
                context_id,
                "model-postgres-v1",
                0.2,
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.97).expect("metric"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 720.0).expect("metric"),
                ],
                executed_at + chrono::Duration::seconds(1),
            )
            .expect("valid evaluation run"),
        ];

        PostgresBenchmarkFixture {
            project_id,
            context_id,
            commit_id: CommitId::from_uuid(
                Uuid::parse_str("77777777-7777-4777-8777-777777777778").expect("commit uuid"),
            ),
            decision_id: BenchmarkDecisionId::new(),
            dataset,
            suite,
            runs,
            recorded_at: executed_at + chrono::Duration::seconds(2),
        }
    }

    fn postgres_benchmark_breadth_datasets() -> Vec<BenchmarkDataset> {
        vec![
            BenchmarkDataset::new(
                "PostgreSQL breadth alpha",
                vec![
                    BenchmarkCase::new(
                        "alpha one",
                        serde_json::json!({"question": "alpha one"}),
                        BenchmarkExpectedOutput::Exact(serde_json::json!({"answer": "alpha one"})),
                    )
                    .expect("alpha one case"),
                    BenchmarkCase::new(
                        "alpha two",
                        serde_json::json!({"question": "alpha two"}),
                        BenchmarkExpectedOutput::Exact(serde_json::json!({"answer": "alpha two"})),
                    )
                    .expect("alpha two case"),
                ],
            )
            .expect("alpha dataset"),
            BenchmarkDataset::new(
                "PostgreSQL breadth beta",
                vec![
                    BenchmarkCase::new(
                        "beta one",
                        serde_json::json!({"question": "beta one"}),
                        BenchmarkExpectedOutput::Exact(serde_json::json!({"answer": "beta one"})),
                    )
                    .expect("beta one case"),
                    BenchmarkCase::new(
                        "beta two",
                        serde_json::json!({"question": "beta two"}),
                        BenchmarkExpectedOutput::Exact(serde_json::json!({"answer": "beta two"})),
                    )
                    .expect("beta two case"),
                ],
            )
            .expect("beta dataset"),
        ]
    }

    fn postgres_benchmark_breadth_suite(datasets: &[BenchmarkDataset]) -> BenchmarkSuite {
        BenchmarkSuite::new(
            "PostgreSQL breadth release gate",
            datasets.iter().map(BenchmarkDataset::id).collect(),
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
            ],
        )
        .expect("breadth suite")
    }

    struct PostgresBenchmarkBreadthProjectionInput<'a> {
        fixture: &'a PostgresBenchmarkFixture,
        datasets: &'a [BenchmarkDataset],
        suite: &'a BenchmarkSuite,
        decision_id: BenchmarkDecisionId,
        commit_id: CommitId,
        accuracy: f64,
        latency: f64,
        offset_seconds: i64,
    }

    fn postgres_benchmark_breadth_runs(
        fixture: &PostgresBenchmarkFixture,
        accuracy: f64,
        latency: f64,
        offset_seconds: i64,
    ) -> Vec<EvaluationRun> {
        (0..4)
            .map(|index| {
                EvaluationRun::new(
                    fixture.context_id,
                    "model-postgres-breadth-v1",
                    0.2,
                    vec![
                        MetricMeasurement::new(MetricKind::Accuracy, accuracy)
                            .expect("accuracy measurement"),
                        MetricMeasurement::new(MetricKind::LatencyMs, latency)
                            .expect("latency measurement"),
                    ],
                    fixture.recorded_at
                        + chrono::Duration::seconds(offset_seconds + i64::from(index)),
                )
                .expect("breadth evaluation run")
            })
            .collect()
    }

    async fn persist_postgres_benchmark_breadth_projection(
        repository: &PostgresContextGraphRepository,
        input: PostgresBenchmarkBreadthProjectionInput<'_>,
    ) -> (
        PersistBenchmarkWorkspaceProjectionV1,
        BenchmarkWorkspaceProjectionWriteResult,
    ) {
        let PostgresBenchmarkBreadthProjectionInput {
            fixture,
            datasets,
            suite,
            decision_id,
            commit_id,
            accuracy,
            latency,
            offset_seconds,
        } = input;
        let plan = BenchmarkExecutionPlan::new(suite.clone(), datasets.to_vec())
            .expect("breadth projection plan");
        let results = plan
            .cases()
            .iter()
            .map(|case| {
                BenchmarkCaseExecutionResult::new(
                    case.dataset_id(),
                    case.case_id(),
                    vec![
                        MetricMeasurement::new(MetricKind::Accuracy, accuracy)
                            .expect("breadth accuracy measurement"),
                        MetricMeasurement::new(MetricKind::LatencyMs, latency)
                            .expect("breadth latency measurement"),
                    ],
                )
                .expect("breadth case execution result")
            })
            .collect::<Vec<_>>();
        let executed_at = fixture.recorded_at + chrono::Duration::seconds(offset_seconds);
        let executed_at = executed_at
            - chrono::Duration::nanoseconds(i64::from(
                executed_at.timestamp_subsec_nanos() % 1_000,
            ));
        let cohort = plan
            .assemble_cohort(
                decision_id.as_uuid(),
                fixture.context_id,
                "model-postgres-breadth-v1",
                0.2,
                executed_at,
                results.clone(),
            )
            .expect("assemble breadth cohort");
        let runs = cohort.runs();
        let evidence = repository
            .persist_benchmark_evaluation(
                PersistBenchmarkEvaluationEvidence::new(
                    decision_id,
                    fixture.project_id,
                    commit_id,
                    datasets.to_vec(),
                    suite.clone(),
                    runs,
                    BenchmarkEvaluation::from_runs(suite, &cohort.runs()),
                    "contextlab.exact-match",
                    "breadth-v1",
                    executed_at,
                )
                .expect("breadth evidence command"),
            )
            .await
            .expect("persist breadth decision evidence")
            .evidence()
            .clone();
        let receipt = BenchmarkExecutionReceipt::from_plan(
            &plan,
            decision_id.as_uuid(),
            fixture.context_id,
            "model-postgres-breadth-v1",
            0.2,
            executed_at,
            evidence.comparability().fingerprint(),
            results,
        )
        .expect("breadth execution receipt");
        let command = PersistBenchmarkWorkspaceProjectionV1::new(
            evidence,
            datasets.to_vec(),
            suite.clone(),
            receipt,
        )
        .expect("breadth projection command");
        let created = repository
            .persist_benchmark_workspace_projection(command.clone())
            .await
            .expect("persist breadth projection");
        (command, created)
    }

    #[test]
    fn postgres_benchmark_internal_digest_uses_database_timestamp_precision() {
        let fixture = postgres_benchmark_fixture();
        let command = fixture.command("evaluator-v1");
        let normalized = benchmark_command_at_postgres_precision(command.clone())
            .expect("normalize benchmark evidence");

        assert_ne!(
            command.evidence().evidence_digest(),
            normalized.evidence().evidence_digest()
        );
        assert_eq!(
            normalized.runs()[0].executed_at().timestamp_subsec_nanos(),
            123_456_000
        );
        assert_eq!(
            normalized.evidence().recorded_at().timestamp_subsec_nanos(),
            123_456_000
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_breadth_persists_reads_exact_scopes_scorecard_coverage_and_evaluation_diff()
     {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let fixture = postgres_benchmark_fixture();
        let datasets = postgres_benchmark_breadth_datasets();
        let suite = postgres_benchmark_breadth_suite(&datasets);
        let revised_commit_id = CommitId::new();
        let revised_decision_id = BenchmarkDecisionId::new();

        sqlx::query(
            "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at, created_at) VALUES ($1, $2, 'main', 'PostgreSQL breadth revision', '[]'::jsonb, $3, $3)",
        )
        .bind(revised_commit_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.recorded_at)
        .execute(&pool)
        .await
        .expect("insert revised Context commit");

        let baseline = repository
            .persist_benchmark_evaluation(fixture.command_with_commit(
                fixture.decision_id,
                fixture.commit_id,
                datasets.clone(),
                suite.clone(),
                postgres_benchmark_breadth_runs(&fixture, 0.95, 700.0, 10),
                "breadth-v1",
            ))
            .await
            .expect("persist baseline breadth evidence")
            .evidence()
            .clone();
        let revised = repository
            .persist_benchmark_evaluation(fixture.command_with_commit(
                revised_decision_id,
                revised_commit_id,
                datasets,
                suite.clone(),
                postgres_benchmark_breadth_runs(&fixture, 0.75, 900.0, 20),
                "breadth-v1",
            ))
            .await
            .expect("persist revised breadth evidence")
            .evidence()
            .clone();

        for (expected, commit_id, status, accuracy, latency) in [
            (
                &baseline,
                fixture.commit_id,
                RegressionDecisionStatus::Passed,
                0.95,
                700.0,
            ),
            (
                &revised,
                revised_commit_id,
                RegressionDecisionStatus::Regressed,
                0.75,
                900.0,
            ),
        ] {
            let read = repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    commit_id,
                    expected.decision_id(),
                )
                .await
                .expect("read exact breadth decision")
                .expect("persisted breadth decision");
            assert_eq!(read.project_id(), fixture.project_id);
            assert_eq!(read.context_id(), fixture.context_id);
            assert_eq!(read.context_commit_id(), commit_id);
            assert_eq!(read.dataset_ids(), suite.dataset_ids());
            assert_eq!(read.run_ids().len(), 4);
            assert_eq!(read.metric_results().len(), 2);
            assert_eq!(read.status(), status);
            for (metric, observed) in [
                (MetricKind::Accuracy, accuracy),
                (MetricKind::LatencyMs, latency),
            ] {
                let result = read
                    .metric_results()
                    .iter()
                    .find(|result| result.metric() == metric)
                    .expect("read expected scorecard metric");
                assert_eq!(result.observed(), Some(observed));
                assert_eq!(result.sample_count(), 4);
                assert_eq!(result.required_sample_count(), 4);
                assert!(result.has_complete_coverage());
            }
            for run_id in read.run_ids() {
                let run = repository
                    .get_benchmark_run(fixture.project_id, fixture.context_id, commit_id, *run_id)
                    .await
                    .expect("read exact breadth run")
                    .expect("persisted breadth run");
                assert_eq!(run.id(), *run_id);
                assert_eq!(run.measurements().len(), 2);
            }
        }

        let diff = BenchmarkDecisionComparisonService::new(&repository)
            .compare(
                fixture.project_id,
                fixture.context_id,
                BenchmarkDecisionComparisonScope::new(fixture.commit_id, baseline.decision_id()),
                BenchmarkDecisionComparisonScope::new(revised_commit_id, revised.decision_id()),
            )
            .await
            .expect("compare exact persisted breadth scopes")
            .expect("both breadth decisions must be readable");
        assert_eq!(
            diff.status_change(),
            Some((
                RegressionDecisionStatus::Passed,
                RegressionDecisionStatus::Regressed,
            ))
        );
        assert_eq!(diff.metric_changes().len(), 2);
        for metric in [MetricKind::Accuracy, MetricKind::LatencyMs] {
            assert!(
                diff.metric_changes()
                    .iter()
                    .any(|change| change.metric() == metric),
                "evaluation diff must retain {metric:?} evidence"
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_breadth_round_trips_projection_provenance_redaction_replay_and_scope()
     {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let fixture = postgres_benchmark_fixture();
        let datasets = postgres_benchmark_breadth_datasets();
        let suite = postgres_benchmark_breadth_suite(&datasets);
        let revised_commit_id = CommitId::new();
        sqlx::query(
            "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at, created_at) VALUES ($1, $2, 'main', 'PostgreSQL breadth projection revision', '[]'::jsonb, $3, $3)",
        )
        .bind(revised_commit_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.recorded_at)
        .execute(&pool)
        .await
        .expect("insert revised breadth projection commit");

        let baseline_decision_id = BenchmarkDecisionId::new();
        let revised_decision_id = BenchmarkDecisionId::new();
        let (baseline_command, baseline_created) = persist_postgres_benchmark_breadth_projection(
            &repository,
            PostgresBenchmarkBreadthProjectionInput {
                fixture: &fixture,
                datasets: &datasets,
                suite: &suite,
                decision_id: baseline_decision_id,
                commit_id: fixture.commit_id,
                accuracy: 0.95,
                latency: 700.0,
                offset_seconds: 30,
            },
        )
        .await;
        let (revised_command, revised_created) = persist_postgres_benchmark_breadth_projection(
            &repository,
            PostgresBenchmarkBreadthProjectionInput {
                fixture: &fixture,
                datasets: &datasets,
                suite: &suite,
                decision_id: revised_decision_id,
                commit_id: revised_commit_id,
                accuracy: 0.75,
                latency: 900.0,
                offset_seconds: 40,
            },
        )
        .await;
        let baseline_scope = *baseline_command.scope();
        let revised_scope = *revised_command.scope();

        assert_eq!(
            baseline_created.disposition(),
            BenchmarkWorkspaceProjectionWriteDisposition::Created
        );
        assert_eq!(
            revised_created.disposition(),
            BenchmarkWorkspaceProjectionWriteDisposition::Created
        );
        let replayed = repository
            .persist_benchmark_workspace_projection(baseline_command)
            .await
            .expect("replay breadth projection");
        assert_eq!(
            replayed.disposition(),
            BenchmarkWorkspaceProjectionWriteDisposition::Replayed
        );

        let projection = repository
            .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::comparing(
                baseline_scope,
                revised_scope,
            ))
            .await
            .expect("read compared breadth projection");
        assert_eq!(projection.datasets().len(), 2);
        assert!(
            projection
                .datasets()
                .iter()
                .all(|dataset| dataset.case_count() == 2)
        );
        assert_eq!(projection.runs().len(), 4);
        assert_eq!(projection.scorecard().run_count(), 4);
        assert!(
            projection
                .scorecard()
                .metrics()
                .iter()
                .all(|metric| metric.sample_count() == 4 && metric.required_sample_count() == 4)
        );
        assert_eq!(
            projection.regression_status(),
            RegressionDecisionStatus::Regressed
        );
        assert!(projection.evaluation_diff().is_some());
        let serialized = serde_json::to_string(&projection).expect("serialize safe projection");
        for raw in [
            "alpha one",
            "alpha two",
            "beta one",
            "beta two",
            "question",
            "answer",
        ] {
            assert!(
                !serialized.contains(raw),
                "safe projection must not expose raw benchmark payload: {raw}"
            );
        }

        let wrong_scope = BenchmarkWorkspaceProjectionReceiptScope::new(
            ProjectId::new(),
            fixture.context_id,
            revised_scope.context_commit_id(),
            revised_scope.cohort_id(),
        );
        assert!(
            repository
                .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
                    wrong_scope,
                ))
                .await
                .is_err()
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_workspace_projection_creates_replays_and_reads_exact_scope() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let project_id = ProjectId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
        );
        let context_id = ContextId::from_uuid(
            Uuid::parse_str("44444444-4444-4444-8444-444444444444").expect("context id"),
        );
        let commit_id = CommitId::from_uuid(
            Uuid::parse_str("77777777-7777-4777-8777-777777777778").expect("commit id"),
        );
        let decision_id = BenchmarkDecisionId::new();
        let dataset = BenchmarkDataset::new(
            "Projection cases",
            vec![
                BenchmarkCase::new(
                    "Projection case",
                    serde_json::json!({"private": "input"}),
                    BenchmarkExpectedOutput::Exact(serde_json::json!({"private": "expected"})),
                )
                .expect("projection case"),
            ],
        )
        .expect("projection dataset");
        let suite = BenchmarkSuite::new(
            "Projection gate",
            vec![dataset.id()],
            vec![
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
            ],
        )
        .expect("projection suite");
        let plan = BenchmarkExecutionPlan::new(suite.clone(), vec![dataset.clone()])
            .expect("projection plan");
        let executed_at = Utc
            .with_ymd_and_hms(2026, 7, 23, 3, 0, 0)
            .single()
            .expect("projection timestamp");
        let result = BenchmarkCaseExecutionResult::new(
            dataset.id(),
            dataset.cases()[0].id(),
            vec![
                MetricMeasurement::new(MetricKind::LatencyMs, 700.0).expect("latency"),
                MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("accuracy"),
            ],
        )
        .expect("projection result");
        let cohort = plan
            .assemble_cohort(
                decision_id.as_uuid(),
                context_id,
                "model-projection-v1",
                0.2,
                executed_at,
                vec![result.clone()],
            )
            .expect("projection cohort");
        let runs = cohort.runs();
        let evidence = repository
            .persist_benchmark_evaluation(
                PersistBenchmarkEvaluationEvidence::new(
                    decision_id,
                    project_id,
                    commit_id,
                    vec![dataset.clone()],
                    suite.clone(),
                    runs.clone(),
                    BenchmarkEvaluation::from_runs(&suite, &runs),
                    "contextlab.exact-match",
                    "projection-v1",
                    executed_at,
                )
                .expect("projection evidence command"),
            )
            .await
            .expect("persist projection evidence")
            .evidence()
            .clone();
        let receipt = BenchmarkExecutionReceipt::from_plan(
            &plan,
            decision_id.as_uuid(),
            context_id,
            "model-projection-v1",
            0.2,
            executed_at,
            evidence.comparability().fingerprint(),
            vec![result],
        )
        .expect("projection receipt");
        let command = PersistBenchmarkWorkspaceProjectionV1::new(
            evidence,
            vec![dataset.clone()],
            suite.clone(),
            receipt,
        )
        .expect("projection source command");
        let scope = *command.scope();

        let created = repository
            .persist_benchmark_workspace_projection(command.clone())
            .await
            .expect("create projection source");
        let replayed = repository
            .persist_benchmark_workspace_projection(command)
            .await
            .expect("replay projection source");
        let projection = repository
            .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(scope))
            .await
            .expect("read exact projection");

        assert_eq!(
            created.disposition(),
            BenchmarkWorkspaceProjectionWriteDisposition::Created
        );
        assert_eq!(
            replayed.disposition(),
            BenchmarkWorkspaceProjectionWriteDisposition::Replayed
        );
        assert_eq!(projection.receipt().cohort_id(), scope.cohort_id());
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM benchmark_workspace_projection_receipt_seals WHERE cohort_id = $1",
            )
            .bind(scope.cohort_id().as_uuid())
            .fetch_one(&pool)
            .await
            .expect("count projection seals"),
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM benchmark_workspace_projection_cases WHERE cohort_id = $1",
            )
            .bind(scope.cohort_id().as_uuid())
            .fetch_one(&pool)
            .await
            .expect("count projection cases"),
            1
        );

        let post_seal_error = sqlx::query(
            r#"
            INSERT INTO benchmark_workspace_projection_cases (
                project_id, context_id, context_commit_id, decision_id, cohort_id,
                position, dataset_id, case_id, run_id
            ) VALUES ($1, $2, $3, $4, $5, 99, $6, $7, $8)
            "#,
        )
        .bind(project_id.as_uuid())
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .bind(decision_id.as_uuid())
        .bind(scope.cohort_id().as_uuid())
        .bind(dataset.id().as_uuid())
        .bind(dataset.cases()[0].id().as_uuid())
        .bind(runs[0].id().as_uuid())
        .execute(&pool)
        .await
        .expect_err("sealed projection must reject later provenance rows");
        assert!(
            post_seal_error
                .to_string()
                .contains("projection receipt is sealed")
        );

        let incomplete_decision_id = BenchmarkDecisionId::new();
        let incomplete_evidence = repository
            .persist_benchmark_evaluation(
                PersistBenchmarkEvaluationEvidence::new(
                    incomplete_decision_id,
                    project_id,
                    commit_id,
                    vec![dataset],
                    suite.clone(),
                    runs.clone(),
                    BenchmarkEvaluation::from_runs(&suite, &runs),
                    "contextlab.exact-match",
                    "projection-v1",
                    executed_at,
                )
                .expect("incomplete projection evidence command"),
            )
            .await
            .expect("persist incomplete projection evidence")
            .evidence()
            .clone();
        let incomplete_cohort_id = Uuid::new_v4();
        let mut transaction = pool
            .begin()
            .await
            .expect("begin incomplete projection write");
        sqlx::query(
            r#"
            INSERT INTO benchmark_workspace_projection_receipts (
                project_id, context_id, context_commit_id, cohort_id, decision_id,
                receipt_schema_version, evidence_digest, case_count
            ) VALUES ($1, $2, $3, $4, $5, 1, $6, 1)
            "#,
        )
        .bind(project_id.as_uuid())
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .bind(incomplete_cohort_id)
        .bind(incomplete_decision_id.as_uuid())
        .bind(incomplete_evidence.evidence_digest())
        .execute(&mut *transaction)
        .await
        .expect("insert incomplete projection parent");
        let incomplete_error = sqlx::query("SET CONSTRAINTS ALL IMMEDIATE")
            .execute(&mut *transaction)
            .await
            .expect_err("incomplete projection must fail deferred completeness validation");
        assert!(
            incomplete_error
                .to_string()
                .contains("projection receipt is incomplete")
        );
        transaction
            .rollback()
            .await
            .expect("rollback incomplete projection");
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM benchmark_workspace_projection_receipts WHERE cohort_id = $1",
            )
            .bind(incomplete_cohort_id)
            .fetch_one(&pool)
            .await
            .expect("count incomplete projection residue"),
            0
        );
    }

    #[derive(Default)]
    struct PostgresBenchmarkEvaluator {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl BenchmarkCaseEvaluator for PostgresBenchmarkEvaluator {
        async fn evaluate_case(
            &self,
            request: BenchmarkCaseEvaluationRequest,
        ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            BenchmarkCaseExecutionResult::new(
                request.case().dataset_id(),
                request.case().case_id(),
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.96)
                        .expect("accuracy measurement"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 710.0)
                        .expect("latency measurement"),
                ],
            )
            .map_err(|error| BenchmarkCaseEvaluatorError::new(error.to_string()))
        }
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_execution_materializes_and_replays_workspace_projection() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool);
        let fixture = postgres_benchmark_fixture();
        repository
            .persist_benchmark_evaluation(fixture.command("definition-seed-v1"))
            .await
            .expect("persist sealed benchmark definitions");
        let evaluator = PostgresBenchmarkEvaluator::default();
        let service = BenchmarkExecutionService::new(&repository, &evaluator);
        let request = BenchmarkExecutionRequest::new(
            BenchmarkDecisionId::new(),
            fixture.project_id,
            fixture.context_id,
            fixture.commit_id,
            fixture.suite.id(),
            "model-materialization-v1",
            0.2,
            "contextlab.exact-match",
            "materialization-v1",
            fixture.recorded_at + chrono::Duration::seconds(3),
        )
        .expect("execution request")
        .with_idempotency(
            IdempotencyKey::new("postgres-benchmark-execution-replay").expect("key"),
            RequestDigest::new("sha256:postgres-benchmark-execution-replay")
                .expect("request digest"),
        );

        let created = service
            .execute(request.clone())
            .await
            .expect("execute and materialize benchmark workspace");
        let projection = repository
            .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
                *created.workspace_projection_scope(),
            ))
            .await
            .expect("read materialized benchmark workspace");
        let replay_request = BenchmarkExecutionRequest::new(
            BenchmarkDecisionId::new(),
            fixture.project_id,
            fixture.context_id,
            fixture.commit_id,
            fixture.suite.id(),
            "model-materialization-v1",
            0.2,
            "contextlab.exact-match",
            "materialization-v1",
            fixture.recorded_at + chrono::Duration::seconds(3),
        )
        .expect("replay execution request")
        .with_idempotency(
            IdempotencyKey::new("postgres-benchmark-execution-replay").expect("key"),
            RequestDigest::new("sha256:postgres-benchmark-execution-replay")
                .expect("request digest"),
        );
        let replayed = service
            .execute(replay_request)
            .await
            .expect("replay materialized benchmark workspace");

        assert_eq!(
            created.disposition(),
            BenchmarkExecutionDisposition::Created
        );
        assert_eq!(
            created.workspace_projection_disposition(),
            BenchmarkWorkspaceProjectionWriteDisposition::Created
        );
        assert_eq!(
            projection.receipt().cohort_id(),
            created.workspace_projection_scope().cohort_id()
        );
        assert_eq!(
            replayed.disposition(),
            BenchmarkExecutionDisposition::Replayed
        );
        assert_eq!(
            replayed.workspace_projection_disposition(),
            BenchmarkWorkspaceProjectionWriteDisposition::Replayed
        );
        assert_eq!(evaluator.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_definition_binding_replays_and_reads_exact_scope() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let fixture = postgres_benchmark_fixture();
        sqlx::query(
            "INSERT INTO context_branches (context_id, branch_name, head_commit_id) VALUES ($1, $2, $3)",
        )
        .bind(fixture.context_id.as_uuid())
        .bind("main")
        .bind(fixture.commit_id.as_uuid())
        .execute(&pool)
        .await
        .expect("insert exact branch head");

        let repository = PostgresContextGraphRepository::new(pool);
        let command = BenchmarkDefinitionBindingCommand::new(
            test_principal(),
            Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1),
            fixture.project_id,
            fixture.context_id,
            fixture.commit_id,
            BranchName::new("main").expect("branch"),
            fixture.commit_id,
            "benchmark-definition-binding-001",
            "sha256:benchmark-definition-binding-001",
            vec![fixture.dataset.clone()],
            fixture.suite.clone(),
            fixture.recorded_at,
            crate::BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
        )
        .expect("valid definition binding command");

        let created = repository
            .persist_benchmark_definition_binding(command.clone())
            .await
            .expect("persist benchmark definition binding");
        let replayed = repository
            .persist_benchmark_definition_binding(command)
            .await
            .expect("replay benchmark definition binding");
        assert_eq!(
            created.disposition(),
            BenchmarkDefinitionBindingWriteDisposition::Created
        );
        assert_eq!(
            replayed.disposition(),
            BenchmarkDefinitionBindingWriteDisposition::Replayed
        );
        assert_eq!(created.binding(), replayed.binding());

        let exact = repository
            .get_benchmark_definition_binding(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                created.binding().id(),
            )
            .await
            .expect("read exact benchmark definition binding")
            .expect("binding exists");
        assert_eq!(&exact, created.binding());
        let listed = repository
            .list_benchmark_definition_bindings_at_commit(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
            )
            .await
            .expect("list exact benchmark definition bindings");
        assert_eq!(listed, vec![exact]);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_evidence_persists_replays_and_rehydrates_multi_run_evidence() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool);
        let fixture = postgres_benchmark_fixture();
        let command = fixture.command("evaluator-v1");
        let normalized = benchmark_command_at_postgres_precision(command.clone())
            .expect("normalize expected benchmark evidence");

        let first_repository = repository.clone();
        let second_repository = repository.clone();
        let (first, second) = tokio::join!(
            first_repository.persist_benchmark_evaluation(command.clone()),
            second_repository.persist_benchmark_evaluation(command),
        );
        let first = first.expect("first concurrent benchmark write");
        let second = second.expect("second concurrent benchmark write");

        assert_eq!(
            [first.disposition(), second.disposition()]
                .into_iter()
                .filter(|disposition| {
                    *disposition == BenchmarkEvidenceWriteDisposition::Created
                })
                .count(),
            1
        );
        assert_eq!(
            [first.disposition(), second.disposition()]
                .into_iter()
                .filter(|disposition| {
                    *disposition == BenchmarkEvidenceWriteDisposition::Replayed
                })
                .count(),
            1
        );
        assert_eq!(first.evidence(), second.evidence());
        let created = if first.disposition() == BenchmarkEvidenceWriteDisposition::Created {
            &first
        } else {
            &second
        };
        assert_eq!(created.evidence(), normalized.evidence());
        assert_eq!(created.evidence().run_ids().len(), 2);
        assert_eq!(
            created.evidence().status(),
            RegressionDecisionStatus::Passed
        );
        assert_eq!(
            repository
                .get_benchmark_dataset(fixture.project_id, fixture.dataset.id())
                .await
                .expect("read dataset"),
            Some(fixture.dataset.clone())
        );
        assert_eq!(
            repository
                .get_benchmark_suite(fixture.project_id, fixture.suite.id())
                .await
                .expect("read suite"),
            Some(fixture.suite.clone())
        );
        for run in normalized.runs() {
            assert_eq!(
                repository
                    .get_benchmark_run(
                        fixture.project_id,
                        fixture.context_id,
                        fixture.commit_id,
                        run.id(),
                    )
                    .await
                    .expect("read run"),
                Some(run.clone())
            );
        }
        assert_eq!(
            repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.commit_id,
                    fixture.decision_id,
                )
                .await
                .expect("read decision"),
            Some(created.evidence().clone())
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_decision_run_details_preserve_sealed_membership_order_and_scope() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool);
        let fixture = postgres_benchmark_fixture();
        let command = fixture.command("evaluator-v1");
        let normalized = benchmark_command_at_postgres_precision(command.clone())
            .expect("normalize expected benchmark evidence");
        let evidence = repository
            .persist_benchmark_evaluation(command)
            .await
            .expect("persist sealed benchmark evidence")
            .evidence()
            .clone();

        let summary = BenchmarkDecisionRunDetailsService::new(&repository)
            .summarize(&evidence)
            .await
            .expect("resolve sealed benchmark run details");

        assert_eq!(
            summary
                .runs()
                .iter()
                .map(|run| run.id())
                .collect::<Vec<_>>(),
            evidence.run_ids()
        );
        for (actual, expected) in summary.runs().iter().zip(normalized.runs()) {
            assert_eq!(actual.model_version(), expected.model_version());
            assert_eq!(actual.temperature(), expected.temperature());
            assert_eq!(actual.executed_at(), expected.executed_at());
            assert_eq!(
                actual
                    .measurements()
                    .iter()
                    .map(|measurement| (measurement.metric(), measurement.value()))
                    .collect::<Vec<_>>(),
                expected
                    .measurements()
                    .iter()
                    .map(|measurement| (measurement.kind(), measurement.value()))
                    .collect::<Vec<_>>()
            );
        }
        assert_eq!(
            repository
                .get_benchmark_run(
                    fixture.project_id,
                    fixture.context_id,
                    CommitId::new(),
                    evidence.run_ids()[0],
                )
                .await
                .expect("read wrong-scope benchmark run"),
            None,
            "a sealed member must remain unavailable outside its exact commit scope"
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_evidence_compares_two_exact_commit_scopes_in_one_consistent_read() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let fixture = postgres_benchmark_fixture();
        repository
            .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
            .await
            .expect("persist baseline benchmark evidence");

        let revised_commit_id = CommitId::new();
        sqlx::query(
            "INSERT INTO context_commits (id, context_id, branch_name, message, changes) VALUES ($1, $2, 'main', 'Benchmark revision', '[]'::jsonb)",
        )
        .bind(revised_commit_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .execute(&pool)
        .await
        .expect("insert revised Context commit");
        repository
            .persist_benchmark_evaluation(fixture.command_with_commit(
                fixture.decision_id,
                revised_commit_id,
                vec![fixture.dataset.clone()],
                fixture.suite.clone(),
                vec![
                    EvaluationRun::new(
                        fixture.context_id,
                        "model-postgres-v1",
                        0.2,
                        vec![
                            MetricMeasurement::new(MetricKind::Accuracy, 0.84)
                                .expect("revised accuracy"),
                            MetricMeasurement::new(MetricKind::LatencyMs, 700.0)
                                .expect("revised latency"),
                        ],
                        fixture.runs[0].executed_at() + chrono::Duration::seconds(3),
                    )
                    .expect("first revised run"),
                    EvaluationRun::new(
                        fixture.context_id,
                        "model-postgres-v1",
                        0.2,
                        vec![
                            MetricMeasurement::new(MetricKind::Accuracy, 0.85)
                                .expect("revised accuracy"),
                            MetricMeasurement::new(MetricKind::LatencyMs, 710.0)
                                .expect("revised latency"),
                        ],
                        fixture.runs[1].executed_at() + chrono::Duration::seconds(3),
                    )
                    .expect("second revised run"),
                ],
                "evaluator-v1",
            ))
            .await
            .expect("persist revised benchmark evidence");

        let comparison = BenchmarkDecisionComparisonService::new(&repository)
            .compare(
                fixture.project_id,
                fixture.context_id,
                BenchmarkDecisionComparisonScope::new(fixture.commit_id, fixture.decision_id),
                BenchmarkDecisionComparisonScope::new(revised_commit_id, fixture.decision_id),
            )
            .await
            .expect("compare exact decision scopes")
            .expect("both persisted decisions must be available");

        assert_eq!(
            comparison.status_change(),
            Some((
                RegressionDecisionStatus::Passed,
                RegressionDecisionStatus::Regressed
            ))
        );
        assert_eq!(comparison.metric_changes().len(), 1);
        assert_eq!(
            comparison.metric_changes()[0].metric(),
            MetricKind::Accuracy
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_evidence_rejects_measurement_append_after_run_sealing() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let fixture = postgres_benchmark_fixture();
        repository
            .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
            .await
            .expect("persist benchmark evidence");

        let error = sqlx::query(
            r#"
            INSERT INTO benchmark_run_measurements (
                project_id, context_id, context_commit_id, run_id, position, metric, value
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.commit_id.as_uuid())
        .bind(fixture.runs[0].id().as_uuid())
        .bind(999_i32)
        .bind("accuracy")
        .bind(0.1_f64)
        .execute(&pool)
        .await
        .expect_err("sealed run must reject later measurement inserts");

        assert!(error.to_string().contains("benchmark run is sealed"));
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_evidence_rejects_out_of_range_temperature() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let fixture = postgres_benchmark_fixture();

        let error = sqlx::query(
            "INSERT INTO benchmark_evaluation_runs (project_id, context_id, context_commit_id, run_id, model_version, temperature, executed_at, created_at) VALUES ($1, $2, $3, $4, 'model-postgres-v1', 2.1, $5, $5)",
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.commit_id.as_uuid())
        .bind(Uuid::new_v4())
        .bind(fixture.recorded_at)
        .execute(&pool)
        .await
        .expect_err("out-of-range temperature must fail before benchmark persistence");

        assert!(
            error
                .to_string()
                .contains("benchmark_evaluation_runs_temperature_check")
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_evidence_rejects_all_sealed_child_appends() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let fixture = postgres_benchmark_fixture();
        repository
            .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
            .await
            .expect("persist sealed benchmark evidence");

        let dataset_case_error = sqlx::query(
            "INSERT INTO benchmark_dataset_cases (project_id, dataset_id, case_id, position, name, input, expected_mode, expected_output) VALUES ($1, $2, $3, 99, 'Late dataset case', '{}'::jsonb, 'unspecified', NULL)",
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.dataset.id().as_uuid())
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await
        .expect_err("sealed dataset must reject later cases");
        assert!(
            dataset_case_error
                .to_string()
                .contains("benchmark dataset is sealed")
        );

        let suite_dataset_error = sqlx::query(
            "INSERT INTO benchmark_suite_datasets (project_id, suite_id, dataset_id, position) VALUES ($1, $2, $3, 99)",
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.suite.id().as_uuid())
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await
        .expect_err("sealed suite must reject later dataset membership");
        assert!(
            suite_dataset_error
                .to_string()
                .contains("benchmark suite is sealed")
        );

        let suite_threshold_error = sqlx::query(
            "INSERT INTO benchmark_suite_thresholds (project_id, suite_id, metric, direction, threshold_value) VALUES ($1, $2, 'cost_usd', 'maximum', 1.0)",
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.suite.id().as_uuid())
        .execute(&pool)
        .await
        .expect_err("sealed suite must reject later thresholds");
        assert!(
            suite_threshold_error
                .to_string()
                .contains("benchmark suite is sealed")
        );

        let run_measurement_error = sqlx::query(
            "INSERT INTO benchmark_run_measurements (project_id, context_id, context_commit_id, run_id, position, metric, value) VALUES ($1, $2, $3, $4, 99, 'accuracy', 0.1)",
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.commit_id.as_uuid())
        .bind(fixture.runs[0].id().as_uuid())
        .execute(&pool)
        .await
        .expect_err("sealed run must reject later measurements");
        assert!(
            run_measurement_error
                .to_string()
                .contains("benchmark run is sealed")
        );

        let decision_run_error = sqlx::query(
            "INSERT INTO benchmark_decision_runs (project_id, context_id, context_commit_id, decision_id, run_id, position) VALUES ($1, $2, $3, $4, $5, 99)",
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.commit_id.as_uuid())
        .bind(fixture.decision_id.as_uuid())
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await
        .expect_err("sealed decision must reject later run membership");
        assert!(
            decision_run_error
                .to_string()
                .contains("benchmark decision is sealed")
        );

        let decision_metric_error = sqlx::query(
            "INSERT INTO benchmark_decision_metric_results (project_id, context_id, context_commit_id, decision_id, suite_id, metric, observed_value, sample_count, required_sample_count, has_complete_coverage, outcome) VALUES ($1, $2, $3, $4, $5, 'cost_usd', 1.0, 1, 1, TRUE, 'passed')",
        )
        .bind(fixture.project_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.commit_id.as_uuid())
        .bind(fixture.decision_id.as_uuid())
        .bind(fixture.suite.id().as_uuid())
        .execute(&pool)
        .await
        .expect_err("sealed decision must reject later metric evidence");
        assert!(
            decision_metric_error
                .to_string()
                .contains("benchmark decision is sealed")
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_evidence_scopes_same_decision_to_exact_commit() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let fixture = postgres_benchmark_fixture();
        let first = repository
            .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
            .await
            .expect("persist first commit evidence");
        let second_commit_id = CommitId::new();
        sqlx::query(
            "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at, created_at) VALUES ($1, $2, 'main', 'Second benchmark commit', '[]'::jsonb, $3, $3)",
        )
        .bind(second_commit_id.as_uuid())
        .bind(fixture.context_id.as_uuid())
        .bind(fixture.recorded_at)
        .execute(&pool)
        .await
        .expect("insert second benchmark commit");
        let second_runs = vec![
            EvaluationRun::from_persisted(
                EvaluationRunId::new(),
                fixture.context_id,
                "model-postgres-v1",
                0.2,
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.94).expect("metric"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 720.0).expect("metric"),
                ],
                fixture.recorded_at + chrono::Duration::seconds(1),
            )
            .expect("second benchmark run"),
            EvaluationRun::from_persisted(
                EvaluationRunId::new(),
                fixture.context_id,
                "model-postgres-v1",
                0.2,
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("metric"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 730.0).expect("metric"),
                ],
                fixture.recorded_at + chrono::Duration::seconds(2),
            )
            .expect("second benchmark run"),
        ];
        let second_command = fixture.command_with_commit(
            fixture.decision_id,
            second_commit_id,
            vec![fixture.dataset.clone()],
            fixture.suite.clone(),
            second_runs,
            "evaluator-v1",
        );
        let second = repository
            .persist_benchmark_evaluation(second_command.clone())
            .await
            .expect("persist same decision for second commit");
        let replayed = repository
            .persist_benchmark_evaluation(second_command)
            .await
            .expect("replay same decision for second commit");

        assert_eq!(
            replayed.disposition(),
            BenchmarkEvidenceWriteDisposition::Replayed
        );
        assert_eq!(
            repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.commit_id,
                    fixture.decision_id,
                )
                .await
                .expect("read first decision"),
            Some(first.evidence().clone())
        );
        assert_eq!(
            repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    second_commit_id,
                    fixture.decision_id,
                )
                .await
                .expect("read second decision"),
            Some(second.evidence().clone())
        );
        assert_eq!(
            repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    CommitId::new(),
                    fixture.decision_id,
                )
                .await
                .expect("read unrelated decision"),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_benchmark_evidence_rejects_conflicts_without_partial_writes() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool);
        let fixture = postgres_benchmark_fixture();
        repository
            .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
            .await
            .expect("persist benchmark evidence");

        let digest_error = repository
            .persist_benchmark_evaluation(fixture.command("evaluator-v2"))
            .await
            .expect_err("changed decision digest must conflict");
        assert!(matches!(
            digest_error,
            StorageRepositoryError::BenchmarkEvidenceDigestConflict { .. }
        ));

        let conflicting_dataset = BenchmarkDataset::with_id(
            fixture.dataset.id(),
            "Changed PostgreSQL support cases",
            fixture.dataset.cases().to_vec(),
        )
        .expect("conflicting dataset");
        let next_decision_id = BenchmarkDecisionId::new();
        let conflict_error = repository
            .persist_benchmark_evaluation(fixture.command_with(
                next_decision_id,
                vec![conflicting_dataset],
                fixture.suite.clone(),
                fixture.runs.clone(),
                "evaluator-v1",
            ))
            .await
            .expect_err("changed immutable definition must conflict");
        assert!(matches!(
            conflict_error,
            StorageRepositoryError::BenchmarkDefinitionConflict {
                definition_kind: "dataset",
                ..
            }
        ));
        assert_eq!(
            repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.commit_id,
                    next_decision_id,
                )
                .await
                .expect("read absent decision"),
            None
        );

        let unpersisted_dataset = BenchmarkDataset::new(
            "Unpersisted suite conflict dataset",
            fixture.dataset.cases().to_vec(),
        )
        .expect("unpersisted dataset");
        let conflicting_suite = BenchmarkSuite::with_id(
            fixture.suite.id(),
            "Changed PostgreSQL release gate",
            vec![unpersisted_dataset.id()],
            fixture.suite.thresholds().to_vec(),
        )
        .expect("conflicting suite");
        let suite_conflict_decision_id = BenchmarkDecisionId::new();
        let suite_error = repository
            .persist_benchmark_evaluation(fixture.command_with(
                suite_conflict_decision_id,
                vec![unpersisted_dataset.clone()],
                conflicting_suite,
                fixture.runs.clone(),
                "evaluator-v1",
            ))
            .await
            .expect_err("changed immutable suite must conflict");
        assert!(matches!(
            suite_error,
            StorageRepositoryError::BenchmarkDefinitionConflict {
                definition_kind: "suite",
                ..
            }
        ));
        assert_eq!(
            repository
                .get_benchmark_dataset(fixture.project_id, unpersisted_dataset.id())
                .await
                .expect("read unpersisted dataset"),
            None
        );
        assert_eq!(
            repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.commit_id,
                    suite_conflict_decision_id,
                )
                .await
                .expect("read absent suite-conflict decision"),
            None
        );

        let original_run = &fixture.runs[0];
        let conflicting_run = EvaluationRun::from_persisted(
            original_run.id(),
            original_run.context_id(),
            original_run.model_version(),
            original_run.temperature(),
            vec![
                MetricMeasurement::new(MetricKind::Accuracy, 0.91).expect("metric"),
                MetricMeasurement::new(MetricKind::LatencyMs, 790.0).expect("metric"),
            ],
            original_run.executed_at(),
        )
        .expect("conflicting persisted run identity");
        let mut conflicting_runs = fixture.runs.clone();
        conflicting_runs[0] = conflicting_run;
        let run_conflict_decision_id = BenchmarkDecisionId::new();
        let run_error = repository
            .persist_benchmark_evaluation(fixture.command_with(
                run_conflict_decision_id,
                vec![fixture.dataset.clone()],
                fixture.suite.clone(),
                conflicting_runs,
                "evaluator-v1",
            ))
            .await
            .expect_err("changed immutable run must conflict");
        assert!(matches!(
            run_error,
            StorageRepositoryError::BenchmarkDefinitionConflict {
                definition_kind: "evaluation_run",
                ..
            }
        ));
        assert_eq!(
            repository
                .get_benchmark_decision(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.commit_id,
                    run_conflict_decision_id,
                )
                .await
                .expect("read absent run-conflict decision"),
            None
        );
    }

    fn protected_route_rate_limit_key(
        identity_source: &str,
        principal_id: &str,
    ) -> ProtectedRouteRateLimitKey {
        ProtectedRouteRateLimitKey::new(
            PrincipalIdentity::new(
                IdentitySourceId::new(identity_source).expect("identity source"),
                PrincipalId::new(principal_id).expect("principal id"),
            ),
            ProtectedRouteOperation::ContextCommitWrite,
        )
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_shared_protected_route_rate_limiter_is_atomic_across_independent_pools() {
        let Some(first_pool) = disposable_test_pool(8).await else {
            return;
        };
        let Some(second_pool) = unseeded_disposable_test_pool(8).await else {
            return;
        };
        let policy = ProtectedRouteRateLimitPolicy::new(3, 60, 10).expect("policy");
        let first_limiter = Arc::new(PostgresProtectedRouteRateLimiter::from_pool(
            first_pool, policy,
        ));
        let second_limiter = Arc::new(PostgresProtectedRouteRateLimiter::from_pool(
            second_pool,
            policy,
        ));
        let key = protected_route_rate_limit_key(
            "https://issuer.contextlab.test/tenant-a",
            "user:shared-limit",
        );

        let mut checks = tokio::task::JoinSet::new();
        for index in 0..12 {
            let limiter: Arc<PostgresProtectedRouteRateLimiter> = if index % 2 == 0 {
                Arc::clone(&first_limiter)
            } else {
                Arc::clone(&second_limiter)
            };
            let key = key.clone();
            checks.spawn(async move { limiter.check(key).await });
        }

        let mut allowed = 0;
        let mut rejected = 0;
        while let Some(result) = checks.join_next().await {
            match result.expect("rate-limit task") {
                Ok(RateLimitDecision::Allowed) => allowed += 1,
                Ok(RateLimitDecision::Rejected { .. }) => rejected += 1,
                Err(error) => panic!("shared limiter must decide without an outage: {error}"),
            }
        }

        assert_eq!(allowed, 3);
        assert_eq!(rejected, 9);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_shared_protected_route_rate_limiter_isolates_identities_recovers_expiry_and_rejects_policy_drift()
     {
        let Some(first_pool) = disposable_test_pool(4).await else {
            return;
        };
        let Some(second_pool) = unseeded_disposable_test_pool(4).await else {
            return;
        };
        let policy = ProtectedRouteRateLimitPolicy::new(1, 1, 4).expect("policy");
        let first_limiter = PostgresProtectedRouteRateLimiter::from_pool(first_pool, policy);
        let second_pool_for_policy_drift = second_pool.clone();
        let second_limiter = PostgresProtectedRouteRateLimiter::from_pool(second_pool, policy);
        let first_identity = protected_route_rate_limit_key(
            "https://issuer.contextlab.test/tenant-a",
            "user:identity-isolation",
        );

        assert_eq!(
            first_limiter.check(first_identity.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        assert!(matches!(
            second_limiter.check(first_identity.clone()).await,
            Ok(RateLimitDecision::Rejected { .. })
        ));
        assert_eq!(
            second_limiter
                .check(protected_route_rate_limit_key(
                    "https://issuer.contextlab.test/tenant-a",
                    "user:separate-subject",
                ))
                .await,
            Ok(RateLimitDecision::Allowed)
        );
        assert_eq!(
            second_limiter
                .check(protected_route_rate_limit_key(
                    "https://issuer.contextlab.test/tenant-b",
                    "user:identity-isolation",
                ))
                .await,
            Ok(RateLimitDecision::Allowed)
        );

        tokio::time::sleep(Duration::from_millis(1_100)).await;

        assert_eq!(
            first_limiter.check(first_identity).await,
            Ok(RateLimitDecision::Allowed)
        );

        let mismatched_policy = ProtectedRouteRateLimitPolicy::new(2, 1, 4).expect("policy");
        let mismatched_limiter = PostgresProtectedRouteRateLimiter::from_pool(
            second_pool_for_policy_drift,
            mismatched_policy,
        );
        assert_eq!(
            mismatched_limiter
                .check(protected_route_rate_limit_key(
                    "https://issuer.contextlab.test/tenant-a",
                    "user:policy-drift",
                ))
                .await,
            Err(RateLimitError::Unavailable)
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_shared_protected_route_rate_limiter_does_not_block_existing_keys_on_global_admission_lock()
     {
        let Some(first_pool) = disposable_test_pool(4).await else {
            return;
        };
        let Some(second_pool) = unseeded_disposable_test_pool(4).await else {
            return;
        };
        let limiter = PostgresProtectedRouteRateLimiter::from_pool(
            first_pool,
            ProtectedRouteRateLimitPolicy::new(3, 60, 10).expect("policy"),
        );
        let key = protected_route_rate_limit_key(
            "https://issuer.contextlab.test/tenant-a",
            "user:existing-key",
        );
        assert_eq!(
            limiter.check(key.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );

        let mut admission_transaction = second_pool.begin().await.expect("begin admission lock");
        sqlx::query(
            "SELECT pg_advisory_xact_lock(hashtextextended('contextlab.protected_route_rate_limit.v1', 0))",
        )
        .execute(&mut *admission_transaction)
        .await
        .expect("hold global admission lock");

        let decision = tokio::time::timeout(Duration::from_secs(1), limiter.check(key)).await;
        admission_transaction
            .rollback()
            .await
            .expect("release global admission lock");

        assert_eq!(decision, Ok(Ok(RateLimitDecision::Allowed)));
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_shared_protected_route_rate_limiter_fails_closed_for_a_future_state_timestamp()
     {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let limiter = PostgresProtectedRouteRateLimiter::from_pool(
            pool.clone(),
            ProtectedRouteRateLimitPolicy::new(2, 60, 10).expect("policy"),
        );
        let key = protected_route_rate_limit_key(
            "https://issuer.contextlab.test/tenant-a",
            "user:future-state",
        );
        assert_eq!(
            limiter.check(key.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );

        sqlx::query(
            "UPDATE public.protected_route_rate_limit_states \
             SET request_timestamps = ARRAY[clock_timestamp() + interval '120 seconds'], \
                 expires_at = clock_timestamp() + interval '180 seconds' \
             WHERE identity_source = $1 AND principal_id = $2 AND operation = $3",
        )
        .bind(key.principal_identity().source().as_str())
        .bind(key.principal_id().as_str())
        .bind("context_commit_write")
        .execute(&pool)
        .await
        .expect("inject future timestamp");

        assert_eq!(limiter.check(key).await, Err(RateLimitError::Unavailable));
    }

    #[tokio::test]
    async fn postgres_protected_route_rate_limiter_fails_closed_for_a_closed_pool() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        pool.close().await;
        let limiter = PostgresProtectedRouteRateLimiter::from_pool(
            pool,
            ProtectedRouteRateLimitPolicy::new(1, 60, 10).expect("policy"),
        );

        assert_eq!(
            limiter
                .check(protected_route_rate_limit_key(
                    "https://issuer.contextlab.test/tenant-a",
                    "user:closed-pool",
                ))
                .await,
            Err(RateLimitError::Unavailable)
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_restricted_audit_purge_requires_executor_role_and_preserves_manifest_evidence()
     {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        sqlx::raw_sql(CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP)
            .execute(&pool)
            .await
            .expect("provision purge roles");
        sqlx::raw_sql(CONTEXTLAB_AUDIT_PURGE_EXECUTOR)
            .execute(&pool)
            .await
            .expect("install private purge procedure");

        let workspace_id = WORKSPACE_GRAPH_SEED_WORKSPACE_ID
            .parse::<Uuid>()
            .expect("workspace uuid");
        let context_id = seeded_context_id().as_uuid();
        let policy_revision_id = Uuid::new_v4();
        let cutoff = Utc
            .with_ymd_and_hms(2026, 7, 13, 0, 0, 0)
            .single()
            .expect("cutoff");
        sqlx::query(
            "INSERT INTO context_authorization_audit_retention_policies \
             (revision_id, workspace_id, retention_duration) VALUES ($1, $2, interval '0 seconds')",
        )
        .bind(policy_revision_id)
        .bind(workspace_id)
        .execute(&pool)
        .await
        .expect("insert retention policy");

        let eligible_event_id = Uuid::new_v4();
        let boundary_event_id = Uuid::new_v4();
        for (event_id, recorded_at) in [
            (eligible_event_id, cutoff - chrono::Duration::seconds(1)),
            (boundary_event_id, cutoff),
        ] {
            sqlx::query(
                "INSERT INTO context_authorization_audit_events \
                 (id, identity_source, principal_id, context_id, permission, decision, recorded_at, retention_disposition, retention_policy_revision_id) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, 'purge_eligible', $8)",
            )
            .bind(event_id)
            .bind("https://issuer.contextlab.test")
            .bind("user:purge-regression")
            .bind(context_id)
            .bind("write")
            .bind("granted")
            .bind(recorded_at)
            .bind(policy_revision_id)
            .execute(&pool)
            .await
            .expect("insert eligible audit event");
        }

        let runtime_role = "contextlab_audit_purge_runtime_test";
        sqlx::query(&format!(
            "DO $$ BEGIN \
             IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '{runtime_role}') THEN \
                 CREATE ROLE {runtime_role} LOGIN PASSWORD 'contextlab-test-purge-runtime' INHERIT; \
             ELSE \
                 ALTER ROLE {runtime_role} LOGIN PASSWORD 'contextlab-test-purge-runtime' INHERIT; \
             END IF; \
             END $$"
        ))
        .execute(&pool)
        .await
        .expect("create disposable runtime login");
        sqlx::query(&format!(
            "GRANT contextlab_audit_purge_executor TO {runtime_role}"
        ))
        .execute(&pool)
        .await
        .expect("grant executor membership");

        let database_url =
            std::env::var("CONTEXTLAB_TEST_DATABASE_URL").expect("disposable database URL");
        let runtime_options = PgConnectOptions::from_str(&database_url)
            .expect("parse disposable database URL")
            .username(runtime_role)
            .password("contextlab-test-purge-runtime");
        let runtime_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(runtime_options)
            .await
            .expect("connect as purge runtime");

        assert!(
            sqlx::query("DELETE FROM context_authorization_audit_events WHERE id = $1")
                .bind(eligible_event_id)
                .execute(&runtime_pool)
                .await
                .is_err()
        );
        assert!(
            sqlx::query("SELECT * FROM context_authorization_audit_events")
                .fetch_all(&runtime_pool)
                .await
                .is_err()
        );

        let executor = PostgresAuthorizationAuditPurgeExecutor::from_pool(runtime_pool);
        let result = executor
            .purge_authorization_audit_events(AuthorizationAuditPurgeRequest::new(
                WorkspaceId::from_uuid(workspace_id),
                policy_revision_id,
                cutoff,
                Some(10),
            ))
            .await
            .expect("purge through executor role");
        assert_eq!(result.purged_event_count(), 1);
        let manifest_id = result.manifest_id().expect("manifest for selected event");

        let remaining_event_ids = sqlx::query_as::<_, (Uuid,)>(
            "SELECT id FROM context_authorization_audit_events ORDER BY id",
        )
        .fetch_all(&pool)
        .await
        .expect("read remaining events")
        .into_iter()
        .map(|(id,)| id)
        .collect::<Vec<_>>();
        assert!(remaining_event_ids.contains(&boundary_event_id));
        assert!(!remaining_event_ids.contains(&eligible_event_id));

        let manifest_item = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
            "SELECT manifest_id, event_id, permission, decision \
             FROM context_authorization_audit_purge_manifest_items WHERE manifest_id = $1",
        )
        .bind(manifest_id)
        .fetch_one(&pool)
        .await
        .expect("read redacted manifest item");
        assert_eq!(manifest_item.0, manifest_id);
        assert_eq!(manifest_item.1, eligible_event_id);
        assert_eq!(manifest_item.2, "write");
        assert_eq!(manifest_item.3, "granted");

        let second_result = executor
            .purge_authorization_audit_events(AuthorizationAuditPurgeRequest::new(
                WorkspaceId::from_uuid(workspace_id),
                policy_revision_id,
                cutoff,
                Some(10),
            ))
            .await
            .expect("idempotent empty purge");
        assert_eq!(second_result.purged_event_count(), 0);
        assert_eq!(second_result.manifest_id(), None);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn principal_identity_migration_preserves_legacy_memberships_and_checks_new_writes() {
        let Some(pool) = unseeded_disposable_test_pool(1).await else {
            return;
        };
        sqlx::raw_sql(PRE_IDENTITY_NAMESPACE_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply migrations before the identity namespace");

        let workspace_id = Uuid::new_v4();
        let legacy_subject = format!(" {} ", "x".repeat(513));
        sqlx::query("INSERT INTO workspaces (id, name, slug) VALUES ($1, $2, $3)")
            .bind(workspace_id)
            .bind("Legacy Upgrade Workspace")
            .bind(format!("legacy-upgrade-{workspace_id}"))
            .execute(&pool)
            .await
            .expect("insert legacy workspace");
        sqlx::query(
            "INSERT INTO workspace_memberships (workspace_id, principal_id, role) VALUES ($1, $2, $3)",
        )
        .bind(workspace_id)
        .bind(&legacy_subject)
        .bind("reader")
        .execute(&pool)
        .await
        .expect("insert pre-namespace membership");

        sqlx::raw_sql(PRINCIPAL_IDENTITY_NAMESPACE_MIGRATION)
            .execute(&pool)
            .await
            .expect("upgrade legacy membership rows");

        let migrated = sqlx::query_as::<_, (String, String)>(
            "SELECT identity_source, principal_id FROM workspace_memberships WHERE workspace_id = $1",
        )
        .bind(workspace_id)
        .fetch_one(&pool)
        .await
        .expect("read migrated membership");
        assert_eq!(migrated, ("legacy".to_owned(), legacy_subject));

        let invalid_new_write = sqlx::query(
            "INSERT INTO workspace_memberships (workspace_id, identity_source, principal_id, role) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id)
        .bind("https://issuer.contextlab.test")
        .bind(" subject ")
        .bind("reader")
        .execute(&pool)
        .await
        .expect_err("new writes must satisfy the principal identity constraint");
        assert_eq!(
            invalid_new_write
                .as_database_error()
                .and_then(|error| error.constraint()),
            Some("chk_workspace_memberships_principal_id_identity")
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn audit_retention_migration_defaults_existing_events_to_hold_and_preserves_append_only()
    {
        let Some(pool) = unseeded_disposable_test_pool(1).await else {
            return;
        };
        sqlx::raw_sql(PRE_AUDIT_RETENTION_GOVERNANCE_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply migrations before audit retention governance");

        let workspace_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let context_id = Uuid::new_v4();
        sqlx::query("INSERT INTO workspaces (id, name, slug) VALUES ($1, $2, $3)")
            .bind(workspace_id)
            .bind("Audit Retention Upgrade Workspace")
            .bind(format!("audit-retention-upgrade-{workspace_id}"))
            .execute(&pool)
            .await
            .expect("insert workspace before audit retention migration");
        sqlx::query("INSERT INTO projects (id, workspace_id, name, slug) VALUES ($1, $2, $3, $4)")
            .bind(project_id)
            .bind(workspace_id)
            .bind("Audit Retention Upgrade Project")
            .bind("audit-retention-upgrade-project")
            .execute(&pool)
            .await
            .expect("insert project before audit retention migration");
        sqlx::query("INSERT INTO contexts (id, project_id, name) VALUES ($1, $2, $3)")
            .bind(context_id)
            .bind(project_id)
            .bind("Audit Retention Upgrade Context")
            .execute(&pool)
            .await
            .expect("insert context before audit retention migration");
        sqlx::query(
            "INSERT INTO context_authorization_audit_events \
             (identity_source, principal_id, context_id, permission, decision) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind("https://issuer.contextlab.test")
        .bind("user:audit-retention-upgrade")
        .bind(context_id)
        .bind("write")
        .bind("granted")
        .execute(&pool)
        .await
        .expect("insert audit event before audit retention migration");

        sqlx::raw_sql(include_str!(
            "../migrations/0008_context_authorization_audit_governance.sql"
        ))
        .execute(&pool)
        .await
        .expect("upgrade authorization audit events with retention governance");

        let disposition = sqlx::query_as::<_, (String, Option<Uuid>, Option<DateTime<Utc>>)>(
            "SELECT retention_disposition, retention_policy_revision_id, purge_eligible_at \
             FROM context_authorization_audit_events WHERE context_id = $1",
        )
        .bind(context_id)
        .fetch_one(&pool)
        .await
        .expect("read upgraded audit event disposition");
        assert_eq!(disposition, ("hold".to_owned(), None, None));

        let policy_revision_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO context_authorization_audit_retention_policies \
             (revision_id, workspace_id, retention_duration) \
             VALUES ($1, $2, interval '30 days')",
        )
        .bind(policy_revision_id)
        .bind(workspace_id)
        .execute(&pool)
        .await
        .expect("insert immutable retention policy revision");
        sqlx::query(
            "INSERT INTO context_authorization_audit_retention_policy_scopes \
             (workspace_id, active_policy_revision_id) VALUES ($1, $2)",
        )
        .bind(workspace_id)
        .bind(policy_revision_id)
        .execute(&pool)
        .await
        .expect("activate retention policy for its workspace");

        for statement in [
            "UPDATE context_authorization_audit_events \
             SET retention_disposition = 'purge_eligible' WHERE context_id = $1",
            "DELETE FROM context_authorization_audit_events WHERE context_id = $1",
            "UPDATE context_authorization_audit_retention_policies \
             SET retention_duration = interval '31 days' WHERE revision_id = $1",
        ] {
            sqlx::query(statement)
                .bind(if statement.contains("retention_policies") {
                    policy_revision_id
                } else {
                    context_id
                })
                .execute(&pool)
                .await
                .expect_err("ordinary audit retention mutations must be rejected");
        }
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn production_like_migration_rehearsal_upgrades_history_and_preserves_ledger() {
        let Some(pool) = unseeded_disposable_test_pool(1).await else {
            return;
        };
        sqlx::raw_sql(PRE_AUDIT_RETENTION_GOVERNANCE_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply historical production-like schema");

        let workspace_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let context_id = Uuid::new_v4();
        sqlx::query("INSERT INTO workspaces (id, name, slug) VALUES ($1, $2, $3)")
            .bind(workspace_id)
            .bind("Production Rehearsal Workspace")
            .bind(format!("production-rehearsal-{workspace_id}"))
            .execute(&pool)
            .await
            .expect("insert historical workspace");
        sqlx::query("INSERT INTO projects (id, workspace_id, name, slug) VALUES ($1, $2, $3, $4)")
            .bind(project_id)
            .bind(workspace_id)
            .bind("Production Rehearsal Project")
            .bind(format!("production-rehearsal-{project_id}"))
            .execute(&pool)
            .await
            .expect("insert historical project");
        sqlx::query("INSERT INTO contexts (id, project_id, name) VALUES ($1, $2, $3)")
            .bind(context_id)
            .bind(project_id)
            .bind("Production Rehearsal Context")
            .execute(&pool)
            .await
            .expect("insert historical context");
        let audit_event_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO context_authorization_audit_events \
             (id, identity_source, principal_id, context_id, permission, decision) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(audit_event_id)
        .bind("legacy")
        .bind("production-rehearsal-principal")
        .bind(context_id)
        .bind("write")
        .bind("granted")
        .execute(&pool)
        .await
        .expect("insert historical audit event");

        sqlx::raw_sql(AUDIT_RETENTION_GOVERNANCE_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply retention governance forward migration");
        sqlx::raw_sql(MIGRATION_LEDGER_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply migration ledger forward migration");
        sqlx::raw_sql(SHARED_PROTECTED_ROUTE_RATE_LIMIT_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply shared protected-route rate-limit forward migration");
        sqlx::raw_sql(SHARED_PROTECTED_ROUTE_RATE_LIMIT_HARDENING_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply shared protected-route rate-limit hardening migration");
        sqlx::raw_sql(CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP)
            .execute(&pool)
            .await
            .expect("provision private purge roles");
        sqlx::raw_sql(CONTEXTLAB_AUDIT_PURGE_EXECUTOR)
            .execute(&pool)
            .await
            .expect("install private purge procedure");

        let disposition = sqlx::query_as::<_, (String, Option<Uuid>, Option<DateTime<Utc>>)>(
            "SELECT retention_disposition, retention_policy_revision_id, purge_eligible_at \
             FROM context_authorization_audit_events WHERE id = $1",
        )
        .bind(audit_event_id)
        .fetch_one(&pool)
        .await
        .expect("read upgraded historical event");
        assert_eq!(disposition, ("hold".to_owned(), None, None));

        sqlx::query(
            "INSERT INTO protected_route_rate_limit_configurations \
             (max_requests, window_seconds, max_tracked_keys) VALUES ($1, $2, $3)",
        )
        .bind(3_i32)
        .bind(60_i32)
        .bind(10_i32)
        .execute(&pool)
        .await
        .expect("insert upgraded immutable rate-limit configuration");
        assert!(
            sqlx::query(
                "UPDATE protected_route_rate_limit_configurations \
             SET max_requests = $1 WHERE singleton = TRUE",
            )
            .bind(4_i32)
            .execute(&pool)
            .await
            .is_err()
        );

        let ledger_digest = "0".repeat(64);
        sqlx::query(
            "INSERT INTO contextlab_schema_migration_ledger (migration_id, migration_sha256) \
             VALUES ($1, $2)",
        )
        .bind("0009_contextlab_migration_ledger.sql")
        .bind(&ledger_digest)
        .execute(&pool)
        .await
        .expect("record rehearsed ledger migration");
        assert!(sqlx::query(
            "UPDATE contextlab_schema_migration_ledger SET migration_sha256 = $1 WHERE migration_id = $2",
        )
        .bind("f".repeat(64))
        .bind("0009_contextlab_migration_ledger.sql")
        .execute(&pool)
        .await
        .is_err());
        assert!(sqlx::query(
            "INSERT INTO contextlab_schema_migration_ledger (migration_id, migration_sha256) VALUES ($1, $2)",
        )
        .bind("0009_contextlab_migration_ledger.sql")
        .bind(ledger_digest)
        .execute(&pool)
        .await
        .is_err());
    }

    fn seeded_context_id() -> ContextId {
        ContextId::from_uuid(
            "44444444-4444-4444-8444-444444444444"
                .parse()
                .expect("context uuid"),
        )
    }

    fn seeded_project_id() -> ProjectId {
        ProjectId::from_uuid(
            "22222222-2222-4222-8222-222222222222"
                .parse()
                .expect("project uuid"),
        )
    }

    fn test_principal() -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(PrincipalIdentity::new(
            IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:alex").expect("principal"),
        ))
    }

    fn test_principal_with_groups(
        identity_source: &str,
        subject: &str,
        groups: &[&str],
    ) -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::with_trusted_external_groups(
            PrincipalIdentity::new(
                IdentitySourceId::new(identity_source).expect("source"),
                PrincipalId::new(subject).expect("subject"),
            ),
            TrustedExternalGroups::new(
                groups
                    .iter()
                    .map(|group| ExternalGroupId::new(*group).expect("group")),
            )
            .expect("trusted groups"),
        )
    }

    fn guarded_command(
        context_id: ContextId,
        parent_ids: Vec<CommitId>,
        expected_branch_head: ExpectedBranchHead,
        message: &str,
        idempotency_key: &str,
        request_digest: &str,
    ) -> (CommitId, GuardedContextCommitWrite) {
        guarded_command_for_principal(
            test_principal(),
            context_id,
            parent_ids,
            expected_branch_head,
            message,
            idempotency_key,
            request_digest,
        )
    }

    fn guarded_command_for_principal(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        parent_ids: Vec<CommitId>,
        expected_branch_head: ExpectedBranchHead,
        message: &str,
        idempotency_key: &str,
        request_digest: &str,
    ) -> (CommitId, GuardedContextCommitWrite) {
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            message,
            parent_ids,
            vec![ContextChange::created_context(message)],
            Utc::now(),
        )
        .expect("commit");
        let commit_id = commit.id();
        let snapshot = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            commit,
            ContextGraph::new(),
            Utc::now(),
            1,
        )
        .expect("snapshot command");
        let command = GuardedContextCommitWrite::new(
            principal,
            expected_branch_head,
            IdempotencyKey::new(idempotency_key).expect("idempotency key"),
            RequestDigest::new(request_digest).expect("request digest"),
            snapshot,
        )
        .expect("guarded command");

        (commit_id, command)
    }

    #[test]
    fn parses_stored_component_kinds_from_database_values() {
        assert_eq!(
            "system_prompt"
                .parse::<StoredComponentKind>()
                .expect("kind"),
            StoredComponentKind::SystemPrompt
        );
        assert_eq!(
            StoredComponentKind::Evaluation.as_str(),
            "evaluation",
            "evaluation must be part of the v1 storage taxonomy"
        );
        assert!("unsupported".parse::<StoredComponentKind>().is_err());
    }

    #[test]
    fn commit_snapshot_writer_locks_active_context_against_soft_deletes() {
        assert!(CONTEXT_FOR_COMMIT_SNAPSHOT_WRITE_SQL.contains("FOR UPDATE"));
    }

    #[test]
    fn parent_commit_write_sql_requires_a_materialized_graph_snapshot() {
        assert!(
            PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL.contains("JOIN context_commit_graph_snapshots")
        );
        assert!(
            PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL
                .contains("context_commit_graph_snapshots.commit_id = context_commits.id")
        );
        assert!(PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL.contains("context_commits.context_id = $1"));
        assert!(PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL.contains("context_commits.id = ANY($2)"));
        assert!(PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL.contains("FOR KEY SHARE OF context_commits"));
    }

    #[test]
    fn commit_graph_snapshot_scope_sql_binds_project_context_and_commit() {
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("contexts.project_id = $1"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("contexts.id = $2"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("context_commits.id = $3"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("contexts.deleted_at IS NULL"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("JOIN context_commits"));
    }

    #[test]
    fn merge_review_witness_sql_contract_keeps_dag_and_snapshots_in_exact_scope() {
        assert!(COMMIT_GRAPH_BY_CONTEXT_SQL.contains("context_commits.context_id = $1"));
        assert!(COMMIT_GRAPH_BY_CONTEXT_SQL.contains("GROUP BY context_commits.id"));
        assert!(COMMIT_GRAPH_BY_CONTEXT_SQL.contains("context_commit_parents.position"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("contexts.project_id = $1"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("contexts.id = $2"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("context_commits.id = $3"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("contexts.deleted_at IS NULL"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("context_commit_graph_snapshots"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("LEFT JOIN"));
    }

    #[test]
    fn context_lifecycle_aggregate_sql_binds_one_exact_scope_and_normal_ancestry() {
        let component_history = component_state_snapshot_history_sql();
        let replay_history = replay_state_history_sql();
        let revisions = COMPONENT_CONTENT_REVISIONS_FOR_COMMITS_SQL;

        for sql in [component_history.as_str(), replay_history.as_str()] {
            assert!(sql.contains("context_commits.context_id = $1"));
            assert!(sql.contains("context_commits.id = $2"));
            assert!(sql.contains("parent.position = 0"));
            assert!(sql.contains("normal_history.cycle_detected"));
            assert!(sql.contains("normal_history.invalid_parent"));
        }
        assert!(revisions.contains("revisions.context_id = $1"));
        assert!(revisions.contains("revisions.commit_id = ANY($2)"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("contexts.project_id = $1"));
        assert!(COMMIT_GRAPH_SNAPSHOT_BY_CONTEXT_SQL.contains("context_commits.id = $3"));
    }

    #[test]
    fn commit_graph_snapshot_schema_version_is_fail_closed_to_v1() {
        let scope = CommitGraphSnapshotScope::new(
            ProjectId::from_uuid(
                Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("project id"),
            ),
            ContextId::from_uuid(
                Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("context id"),
            ),
            CommitId::from_uuid(
                Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
            ),
        );

        assert_eq!(
            decode_commit_graph_snapshot_schema_version(scope, 1).expect("v1 schema"),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1
        );
        for unsupported in [-1, 0, 2] {
            assert!(decode_commit_graph_snapshot_schema_version(scope, unsupported).is_err());
        }
    }

    #[test]
    fn commit_graph_snapshot_materialization_preserves_missing_snapshot() {
        let scope = CommitGraphSnapshotScope::new(
            ProjectId::from_uuid(
                Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("project id"),
            ),
            ContextId::from_uuid(
                Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("context id"),
            ),
            CommitId::from_uuid(
                Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
            ),
        );

        assert_eq!(
            materialize_commit_graph_snapshot(scope, (None, None, None))
                .expect("unmaterialized commit is not corrupted"),
            None
        );
    }

    #[test]
    fn replay_state_history_sql_reads_the_complete_normal_parent_chain() {
        let sql = replay_state_history_sql();

        for required in [
            "context_commits.context_id = $1",
            "context_commits.id = $2",
            "parent.position = 0",
            "WHERE normal_history.parent_count <= 1",
            "normal_history.parent_count",
            "normal_history.cycle_detected",
            "normal_history.invalid_parent",
            "ORDER BY normal_history.depth DESC, normal_history.id",
            "ORDER BY context_commit_parents.position",
        ] {
            assert!(
                sql.contains(required),
                "missing replay SQL contract: {required}"
            );
        }

        let group_by = sql.split_once("GROUP BY").expect("replay SQL grouping").1;
        for grouped in [
            "normal_history.parent_count",
            "normal_history.cycle_detected",
            "normal_history.invalid_parent",
        ] {
            assert!(
                group_by.contains(grouped),
                "missing replay SQL grouping: {grouped}"
            );
        }
    }

    #[test]
    fn context_commit_history_sql_preserves_one_read_transaction_shape() {
        assert!(CONTEXT_COMMIT_HISTORY_ROWS_SQL.contains("WHERE context_commits.context_id = $1"));
        assert!(
            CONTEXT_COMMIT_HISTORY_ROWS_SQL.contains("ORDER BY context_commits.authored_at ASC")
        );
        assert!(
            CONTEXT_COMMIT_HISTORY_ROWS_SQL.contains("ORDER BY context_commit_parents.position")
        );
        assert!(CONTEXT_BRANCH_HEADS_SQL.contains("WHERE branches.context_id = $1"));
        assert!(CONTEXT_BRANCH_HEADS_SQL.contains("ORDER BY branches.branch_name ASC"));
    }

    fn replay_history_row(
        context_id: Uuid,
        commit_id: Uuid,
        depth: i64,
        parent_commit_ids: &[Uuid],
        changes: Vec<ContextChange>,
    ) -> ReplayStateHistoryRow {
        let authored_at = Utc
            .timestamp_opt(1_700_000_000 + depth, 0)
            .single()
            .expect("time");
        (
            commit_id,
            context_id,
            depth,
            i64::try_from(parent_commit_ids.len()).expect("parent count"),
            false,
            false,
            "main".to_owned(),
            format!("commit-{depth}"),
            parent_commit_ids.to_vec(),
            serde_json::to_value(changes).expect("changes"),
            authored_at,
            authored_at,
        )
    }

    #[test]
    fn replay_state_history_rows_preserve_root_to_target_order_for_shared_replay() {
        let context_uuid = Uuid::from_u128(10);
        let root_uuid = Uuid::from_u128(11);
        let child_uuid = Uuid::from_u128(12);
        let target_uuid = Uuid::from_u128(13);
        let context_id = ContextId::from_uuid(context_uuid);
        let target_commit_id = CommitId::from_uuid(target_uuid);
        let records = replay_state_records_from_rows(
            context_id,
            target_commit_id,
            vec![
                replay_history_row(
                    context_uuid,
                    root_uuid,
                    2,
                    &[],
                    vec![ContextChange::created_context("root")],
                ),
                replay_history_row(context_uuid, child_uuid, 1, &[root_uuid], Vec::new()),
                replay_history_row(context_uuid, target_uuid, 0, &[child_uuid], Vec::new()),
            ],
        )
        .expect("normal parent rows");

        assert_eq!(
            records
                .iter()
                .map(|record| record.id.clone())
                .collect::<Vec<_>>(),
            vec![
                root_uuid.to_string(),
                child_uuid.to_string(),
                target_uuid.to_string()
            ]
        );
        let state = replay_state_from_records(context_id, records).expect("shared replay state");
        assert_eq!(state.context_id(), context_id);
        assert_eq!(state.commit_id(), Some(target_commit_id));
    }

    #[test]
    fn replay_state_history_rows_fail_closed_for_an_unknown_commit() {
        let context_id = ContextId::from_uuid(Uuid::from_u128(20));
        let commit_id = CommitId::from_uuid(Uuid::from_u128(21));

        assert_eq!(
            replay_state_records_from_rows(context_id, commit_id, Vec::new())
                .expect_err("unknown commit"),
            StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            }
        );
    }

    #[test]
    fn replay_state_history_rows_fail_closed_for_corrupted_ancestry_flags() {
        let context_uuid = Uuid::from_u128(30);
        let context_id = ContextId::from_uuid(context_uuid);
        let commit_id = CommitId::from_uuid(Uuid::from_u128(31));

        for (parent_count, cycle_detected, invalid_parent) in
            [(2, false, false), (1, true, false), (1, false, true)]
        {
            let mut row = replay_history_row(
                context_uuid,
                commit_id.as_uuid().to_owned(),
                0,
                &[],
                vec![ContextChange::created_context("corrupt")],
            );
            row.3 = parent_count;
            row.4 = cycle_detected;
            row.5 = invalid_parent;

            assert!(matches!(
                replay_state_records_from_rows(context_id, commit_id, vec![row]),
                Err(StorageRepositoryError::ComponentStateReplayConflict { .. })
            ));
        }
    }

    #[test]
    fn guarded_writer_serializes_missing_idempotency_keys_and_branch_heads() {
        assert!(GUARDED_IDEMPOTENCY_ADVISORY_LOCK_SQL.contains("pg_advisory_xact_lock"));
        assert!(CONTEXT_BRANCH_FOR_UPDATE_SQL.contains("FOR UPDATE"));
        assert!(INSERT_CONTEXT_BRANCH_SQL.contains("ON CONFLICT"));
    }

    #[test]
    fn membership_authorizer_query_scopes_roles_to_active_contexts() {
        assert!(CONTEXT_MEMBERSHIP_ROLE_SQL.contains("workspace_memberships"));
        assert!(CONTEXT_MEMBERSHIP_ROLE_SQL.contains("deleted_at IS NULL"));
        assert!(CONTEXT_MEMBERSHIP_ROLE_SQL.contains("identity_source = $2"));
        assert!(CONTEXT_MEMBERSHIP_ROLE_SQL.contains("principal_id = $3"));
    }

    #[test]
    fn group_role_resolver_query_scopes_bindings_to_source_and_active_workspace() {
        assert!(CONTEXT_EXTERNAL_GROUP_ROLE_SQL.contains("workspace_external_group_role_bindings"));
        assert!(CONTEXT_EXTERNAL_GROUP_ROLE_SQL.contains("identity_source = $2"));
        assert!(CONTEXT_EXTERNAL_GROUP_ROLE_SQL.contains("external_group_id = ANY($3)"));
        assert!(CONTEXT_EXTERNAL_GROUP_ROLE_SQL.contains("deleted_at IS NULL"));
        assert!(CONTEXT_EXTERNAL_GROUP_ROLE_SQL.contains("workspaces.deleted_at IS NULL"));
    }

    #[test]
    fn membership_resolver_query_excludes_deleted_workspaces() {
        assert!(CONTEXT_MEMBERSHIP_ROLE_SQL.contains("JOIN workspaces"));
        assert!(CONTEXT_MEMBERSHIP_ROLE_SQL.contains("workspaces.deleted_at IS NULL"));
    }

    #[test]
    fn guarded_writer_locks_active_membership_before_persisting_a_commit() {
        assert!(CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL.contains("JOIN workspaces"));
        assert!(CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL.contains("workspaces.deleted_at IS NULL"));
        assert!(CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL.contains("identity_source = $2"));
        assert!(CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL.contains("principal_id = $3"));
        assert!(
            CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL
                .contains("FOR UPDATE OF contexts, projects, workspaces, workspace_memberships",)
        );
    }

    #[test]
    fn guarded_writer_locks_active_group_bindings_before_persisting_a_commit() {
        assert!(
            CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL
                .contains("workspace_external_group_role_bindings")
        );
        assert!(CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL.contains("identity_source = $2"));
        assert!(
            CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL
                .contains("external_group_id = ANY($3)")
        );
        assert!(CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL.contains("deleted_at IS NULL"));
        assert!(CONTEXT_WRITE_EXTERNAL_GROUP_ROLES_FOR_UPDATE_SQL.contains(
            "FOR UPDATE OF contexts, projects, workspaces, workspace_external_group_role_bindings",
        ));
    }

    #[test]
    fn idempotency_queries_scope_replays_by_identity_source_subject_and_branch() {
        assert!(CONTEXT_IDEMPOTENCY_SQL.contains("identity_source = $1"));
        assert!(CONTEXT_IDEMPOTENCY_SQL.contains("principal_id = $2"));
        assert!(CONTEXT_IDEMPOTENCY_SQL.contains("receipt.branch_name = $4"));
        assert!(CONTEXT_IDEMPOTENCY_SQL.contains("commit.branch_name = $4"));
        assert!(INSERT_CONTEXT_IDEMPOTENCY_SQL.contains("identity_source"));
        assert!(INSERT_CONTEXT_IDEMPOTENCY_SQL.contains("branch_name"));
    }

    #[test]
    fn postgres_repository_is_a_context_role_resolver() {
        fn assert_context_role_resolver<T: ContextRoleResolver>() {}

        assert_context_role_resolver::<PostgresContextGraphRepository>();
    }

    #[test]
    fn workspace_audit_review_resolver_uses_direct_active_membership() {
        assert!(WORKSPACE_DIRECT_MEMBERSHIP_ROLE_SQL.contains("workspace_memberships"));
        assert!(WORKSPACE_DIRECT_MEMBERSHIP_ROLE_SQL.contains("workspaces.deleted_at IS NULL"));
        assert!(WORKSPACE_DIRECT_MEMBERSHIP_ROLE_SQL.contains("identity_source = $2"));
        assert!(WORKSPACE_DIRECT_MEMBERSHIP_ROLE_SQL.contains("principal_id = $3"));
        assert!(
            !WORKSPACE_DIRECT_MEMBERSHIP_ROLE_SQL
                .contains("workspace_external_group_role_bindings")
        );
    }

    #[test]
    fn authorization_audit_review_query_is_workspace_scoped_redacted_and_cursor_ordered() {
        assert!(
            AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL
                .contains("FROM context_authorization_audit_events")
        );
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("JOIN contexts"));
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("JOIN projects"));
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("JOIN workspaces"));
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("workspaces.id = $1"));
        assert!(
            AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("workspaces.deleted_at IS NULL")
        );
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("recorded_at DESC"));
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("id DESC"));
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("$2::TIMESTAMPTZ IS NULL"));
        assert!(AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains("LIMIT $4"));

        for forbidden in [
            "identity_source",
            "principal_id",
            "context_commits",
            "context_commit_graph_snapshots",
            "context_commit_idempotency",
        ] {
            assert!(
                !AUTHORIZATION_AUDIT_REVIEW_BY_WORKSPACE_SQL.contains(forbidden),
                "redacted review query must not expose or join {forbidden}"
            );
        }
    }

    #[test]
    fn evaluation_metric_count_queries_use_postgres_supported_jsonb_object_counting() {
        for query in [
            EVALUATION_RUNS_BY_WORKSPACE_SQL,
            EVALUATION_RUN_BY_CONTEXT_SQL,
            EVALUATION_RUN_LIST_ITEMS_SQL_TEMPLATE,
        ] {
            assert!(query.contains("CASE jsonb_typeof(evaluation_runs.metrics)"));
            assert!(query.contains("jsonb_object_keys(evaluation_runs.metrics)"));
            assert!(
                !query.contains("jsonb_object_length"),
                "PostgreSQL does not provide jsonb_object_length"
            );
        }
    }

    #[test]
    fn postgres_repository_is_a_workspace_role_resolver() {
        fn assert_workspace_role_resolver<T: contextlab_auth::WorkspaceRoleResolver>() {}

        assert_workspace_role_resolver::<PostgresContextGraphRepository>();
    }

    #[test]
    fn postgres_repository_is_an_authorization_audit_review_repository() {
        fn assert_authorization_audit_review_repository<T: AuthorizationAuditReviewRepository>() {}

        assert_authorization_audit_review_repository::<PostgresContextGraphRepository>();
    }

    #[tokio::test]
    async fn rejects_non_uuid_workspace_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool.clone());

        let error = repository
            .load_context_graph_projection(GraphProjectionScope::Workspace {
                workspace_id: "not-a-uuid".to_owned(),
            })
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_preview_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .load_context_graph_projection(GraphProjectionScope::Preview)
            .await
            .expect_err("postgres repository is workspace scoped");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "preview".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn connect_lazy_rejects_malformed_database_url_without_exposing_it() {
        let error = PostgresContextGraphRepository::connect_lazy("not-a-valid-postgres-url")
            .expect_err("malformed url should fail");

        assert_eq!(
            error,
            StorageRepositoryError::Database {
                message: "database pool configuration failed".to_owned()
            }
        );
        assert!(!error.to_string().contains("not-a-valid-postgres-url"));
    }

    #[test]
    fn workspace_sort_values_map_to_safe_order_clauses() {
        for sort in [
            crate::WorkspaceSort::NameAsc,
            crate::WorkspaceSort::NameDesc,
            crate::WorkspaceSort::CreatedAtAsc,
            crate::WorkspaceSort::CreatedAtDesc,
        ] {
            assert!(!sort.order_by_sql().contains(';'));
            assert!(!sort.order_by_sql().contains("--"));
        }
    }

    #[test]
    fn project_sort_values_map_to_safe_order_clauses() {
        for sort in [
            crate::ProjectSort::NameAsc,
            crate::ProjectSort::NameDesc,
            crate::ProjectSort::CreatedAtAsc,
            crate::ProjectSort::CreatedAtDesc,
        ] {
            assert!(!sort.order_by_sql().contains(';'));
            assert!(!sort.order_by_sql().contains("--"));
        }
    }

    #[test]
    fn experiment_sort_values_map_to_safe_order_clauses() {
        for sort in [
            crate::ExperimentSort::NameAsc,
            crate::ExperimentSort::NameDesc,
            crate::ExperimentSort::BranchNameAsc,
            crate::ExperimentSort::BranchNameDesc,
            crate::ExperimentSort::CreatedAtAsc,
            crate::ExperimentSort::CreatedAtDesc,
        ] {
            assert!(!sort.order_by_sql().contains(';'));
            assert!(!sort.order_by_sql().contains("--"));
        }
    }

    #[test]
    fn context_sort_values_map_to_safe_order_clauses() {
        for sort in [
            crate::ContextSort::NameAsc,
            crate::ContextSort::NameDesc,
            crate::ContextSort::CreatedAtAsc,
            crate::ContextSort::CreatedAtDesc,
        ] {
            assert!(!sort.order_by_sql().contains(';'));
            assert!(!sort.order_by_sql().contains("--"));
        }
    }

    #[test]
    fn commit_sort_values_map_to_safe_order_clauses() {
        for sort in [
            crate::CommitSort::AuthoredAtAsc,
            crate::CommitSort::AuthoredAtDesc,
            crate::CommitSort::CreatedAtAsc,
            crate::CommitSort::CreatedAtDesc,
            crate::CommitSort::BranchNameAsc,
            crate::CommitSort::BranchNameDesc,
        ] {
            assert!(!sort.order_by_sql().contains(';'));
            assert!(!sort.order_by_sql().contains("--"));
        }
    }

    #[test]
    fn component_sort_values_map_to_safe_order_clauses() {
        for sort in [
            crate::ComponentSort::NameAsc,
            crate::ComponentSort::NameDesc,
            crate::ComponentSort::KindAsc,
            crate::ComponentSort::KindDesc,
            crate::ComponentSort::CreatedAtAsc,
            crate::ComponentSort::CreatedAtDesc,
        ] {
            assert!(!sort.order_by_sql().contains(';'));
            assert!(!sort.order_by_sql().contains("--"));
        }
    }

    #[test]
    fn evaluation_run_sort_values_map_to_safe_order_clauses() {
        for sort in [
            crate::EvaluationRunSort::ExecutedAtAsc,
            crate::EvaluationRunSort::ExecutedAtDesc,
            crate::EvaluationRunSort::CreatedAtAsc,
            crate::EvaluationRunSort::CreatedAtDesc,
            crate::EvaluationRunSort::SuiteNameAsc,
            crate::EvaluationRunSort::SuiteNameDesc,
            crate::EvaluationRunSort::ModelVersionAsc,
            crate::EvaluationRunSort::ModelVersionDesc,
        ] {
            assert!(!sort.order_by_sql().contains(';'));
            assert!(!sort.order_by_sql().contains("--"));
        }
    }

    #[tokio::test]
    async fn rejects_non_uuid_project_list_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .list_projects("not-a-uuid".to_owned(), ProjectListQuery::default())
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_experiment_list_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .list_experiments("not-a-uuid".to_owned(), ExperimentListQuery::default())
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_context_list_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .list_contexts("not-a-uuid".to_owned(), ContextListQuery::default())
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_context_experiment_filter_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .list_contexts(
                "11111111-1111-4111-8111-111111111111".to_owned(),
                ContextListQuery::new(
                    None,
                    None,
                    None,
                    Some("not-a-uuid".to_owned()),
                    crate::ContextSort::NameAsc,
                ),
            )
            .await
            .expect_err("invalid experiment filter should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_commit_list_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .list_commits("not-a-uuid".to_owned(), CommitListQuery::default())
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_commit_detail_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .get_commit(
                "11111111-1111-4111-8111-111111111111".to_owned(),
                "not-a-uuid".to_owned(),
            )
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[test]
    fn commit_snapshot_repository_accepts_only_typed_exact_scopes() {
        let scope = CommitGraphSnapshotScope::new(
            seeded_project_id(),
            seeded_context_id(),
            CommitId::from_uuid(
                "77777777-7777-4777-8777-777777777777"
                    .parse()
                    .expect("commit uuid"),
            ),
        );

        assert_eq!(scope.project_id(), seeded_project_id());
        assert_eq!(scope.context_id(), seeded_context_id());
    }

    #[tokio::test]
    async fn rejects_non_uuid_component_list_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .list_components("not-a-uuid".to_owned(), ComponentListQuery::default())
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_component_detail_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .get_component(
                "11111111-1111-4111-8111-111111111111".to_owned(),
                "not-a-uuid".to_owned(),
            )
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_evaluation_run_list_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .list_evaluation_runs("not-a-uuid".to_owned(), EvaluationRunListQuery::default())
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn rejects_non_uuid_evaluation_run_detail_scope_without_connecting_to_database() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://contextlab:contextlab@localhost/contextlab")
            .expect("lazy pool");
        let repository = PostgresContextGraphRepository::new(pool);

        let error = repository
            .get_evaluation_run(
                "11111111-1111-4111-8111-111111111111".to_owned(),
                "not-a-uuid".to_owned(),
            )
            .await
            .expect_err("invalid uuid should fail before query execution");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn projects_seed_workspace_graph_from_postgres() {
        let Some(database_url) = std::env::var("CONTEXTLAB_TEST_DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
        else {
            eprintln!("skipping: CONTEXTLAB_TEST_DATABASE_URL is not configured");
            return;
        };
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("connect to disposable test database");

        sqlx::raw_sql(CONTEXT_PLATFORM_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply migration to empty test database");
        sqlx::raw_sql(WORKSPACE_GRAPH_SEED)
            .execute(&pool)
            .await
            .expect("apply seed fixture");

        let repository = PostgresContextGraphRepository::new(pool);
        let projection = repository
            .load_context_graph_projection(GraphProjectionScope::Workspace {
                workspace_id: WORKSPACE_GRAPH_SEED_WORKSPACE_ID.to_owned(),
            })
            .await
            .expect("load seed projection");
        let graph = projection.project().expect("project seed graph");

        let project_id = seeded_project_id();
        let context_id = seeded_context_id();
        let snapshot = repository
            .get_commit_graph_snapshot(CommitGraphSnapshotScope::new(
                project_id,
                context_id,
                CommitId::from_uuid(
                    "77777777-7777-4777-8777-777777777777"
                        .parse()
                        .expect("seed commit uuid"),
                ),
            ))
            .await
            .expect("snapshot query")
            .expect("materialized snapshot");
        let missing_snapshot = repository
            .get_commit_graph_snapshot(CommitGraphSnapshotScope::new(
                project_id,
                context_id,
                CommitId::from_uuid(
                    "77777777-7777-4777-8777-777777777778"
                        .parse()
                        .expect("unmaterialized commit uuid"),
                ),
            ))
            .await
            .expect("unmaterialized snapshot query");

        assert!(
            graph
                .nodes()
                .values()
                .any(|node| node.kind() == GraphNodeKind::Context)
        );
        assert!(
            graph
                .nodes()
                .values()
                .any(|node| node.kind() == GraphNodeKind::Evaluation)
        );
        assert_eq!(
            graph.nodes().len(),
            10,
            "soft-deleted seed rows should not appear in the graph"
        );
        assert_eq!(snapshot.schema_version(), 1);
        assert_eq!(snapshot.graph().nodes().len(), 1);
        assert_eq!(missing_snapshot, None);

        let root_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Capture root graph",
            Vec::new(),
            vec![ContextChange::created_context("Root graph")],
            Utc::now(),
        )
        .expect("root commit");
        let root_commit_id = root_commit.id();
        let root_snapshot = repository
            .create_commit_snapshot(
                CreateContextCommitSnapshot::new(
                    seeded_project_id(),
                    root_commit,
                    ContextGraph::new(),
                    Utc::now(),
                    1,
                )
                .expect("root command"),
            )
            .await
            .expect("persist root commit snapshot");
        let second_parent = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Capture second parent graph",
            Vec::new(),
            vec![ContextChange::created_context("Second parent graph")],
            Utc::now(),
        )
        .expect("second parent commit");
        let second_parent_id = second_parent.id();
        repository
            .create_commit_snapshot(
                CreateContextCommitSnapshot::new(
                    seeded_project_id(),
                    second_parent,
                    ContextGraph::new(),
                    Utc::now(),
                    1,
                )
                .expect("second parent command"),
            )
            .await
            .expect("persist second parent snapshot");
        let child_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Capture child graph",
            vec![root_commit_id, second_parent_id],
            vec![ContextChange::created_context("Child graph")],
            Utc::now(),
        )
        .expect("child commit");
        let child_commit_id = child_commit.id();
        repository
            .create_commit_snapshot(
                CreateContextCommitSnapshot::new(
                    seeded_project_id(),
                    child_commit,
                    ContextGraph::new(),
                    Utc::now(),
                    1,
                )
                .expect("child command"),
            )
            .await
            .expect("persist child commit snapshot");
        let child_detail = repository
            .get_commit(context_id.to_string(), child_commit_id.to_string())
            .await
            .expect("read child commit");
        let child_snapshot = repository
            .get_commit_graph_snapshot(CommitGraphSnapshotScope::new(
                project_id,
                context_id,
                child_commit_id,
            ))
            .await
            .expect("read child snapshot")
            .expect("child snapshot");

        assert_eq!(root_snapshot.commit_id(), root_commit_id);
        assert_eq!(
            child_detail.parent_commit_ids,
            vec![root_commit_id.to_string(), second_parent_id.to_string()]
        );
        assert_eq!(child_snapshot.schema_version(), 1);

        let rejected_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Reject missing parent graph",
            vec![CommitId::new()],
            vec![ContextChange::created_context("Rejected graph")],
            Utc::now(),
        )
        .expect("rejected commit");
        let rejected_commit_id = rejected_commit.id().to_string();
        let error = repository
            .create_commit_snapshot(
                CreateContextCommitSnapshot::new(
                    seeded_project_id(),
                    rejected_commit,
                    ContextGraph::new(),
                    Utc::now(),
                    1,
                )
                .expect("rejected command"),
            )
            .await
            .expect_err("missing parent should reject the transaction");

        assert!(matches!(
            error,
            StorageRepositoryError::ScopeUnavailable { .. }
        ));
        assert!(matches!(
            repository
                .get_commit(context_id.to_string(), rejected_commit_id)
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_group_editor_can_write_but_direct_reader_overrides_the_group() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let workspace_id = WORKSPACE_GRAPH_SEED_WORKSPACE_ID
            .parse::<Uuid>()
            .expect("workspace uuid");
        let identity_source = "https://groups.contextlab.test";
        let group_id = "group:context-editors";
        sqlx::query(
            "INSERT INTO workspace_external_group_role_bindings (workspace_id, identity_source, external_group_id, role) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id)
        .bind(identity_source)
        .bind(group_id)
        .bind("editor")
        .execute(&pool)
        .await
        .expect("insert editor group binding");

        let group_editor =
            test_principal_with_groups(identity_source, "user:group-editor", &[group_id]);
        assert!(
            RoleBasedContextAuthorizer::new(repository.clone())
                .authorize(&group_editor, context_id, ContextPermission::Write)
                .await
                .is_ok()
        );
        let (root_id, group_command) = guarded_command_for_principal(
            group_editor,
            context_id,
            Vec::new(),
            ExpectedBranchHead::Unborn,
            "Group editor root",
            "group-editor-root",
            "sha256:group-editor-root",
        );
        assert_eq!(
            repository
                .create_guarded_commit_snapshot(group_command)
                .await
                .expect("group editor write")
                .disposition,
            GuardedCommitWriteDisposition::Created
        );

        sqlx::query(
            "INSERT INTO workspace_memberships (workspace_id, identity_source, principal_id, role) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id)
        .bind(identity_source)
        .bind("user:direct-reader")
        .bind("reader")
        .execute(&pool)
        .await
        .expect("insert direct reader membership");
        let direct_reader =
            test_principal_with_groups(identity_source, "user:direct-reader", &[group_id]);
        assert_eq!(
            RoleBasedContextAuthorizer::new(repository.clone())
                .authorize(&direct_reader, context_id, ContextPermission::Write)
                .await,
            Err(AuthorizationError::Forbidden)
        );
        let (_, denied_command) = guarded_command_for_principal(
            direct_reader,
            context_id,
            vec![root_id],
            ExpectedBranchHead::Commit(root_id),
            "Direct reader must not write",
            "direct-reader-denied",
            "sha256:direct-reader-denied",
        );
        assert_eq!(
            repository
                .create_guarded_commit_snapshot(denied_command)
                .await,
            Err(StorageRepositoryError::GuardedWriteForbidden)
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_guarded_writer_replays_the_same_key_concurrently() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let (_, command) = guarded_command(
            context_id,
            Vec::new(),
            ExpectedBranchHead::Unborn,
            "Concurrent guarded root",
            "request-concurrent-root",
            "sha256:concurrent-root",
        );

        let first_repository = repository.clone();
        let second_repository = repository.clone();
        let (first, second) = tokio::join!(
            first_repository.create_guarded_commit_snapshot(command.clone()),
            second_repository.create_guarded_commit_snapshot(command),
        );
        let outcomes = [first, second];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    Ok(result) if result.disposition == GuardedCommitWriteDisposition::Created
                ))
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    Ok(result) if result.disposition == GuardedCommitWriteDisposition::Replayed
                ))
                .count(),
            1
        );

        let counts = sqlx::query_as::<_, (i64, i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM context_commits WHERE context_id = $1), (SELECT COUNT(*) FROM context_commit_graph_snapshots WHERE commit_id IN (SELECT id FROM context_commits WHERE context_id = $1)), (SELECT COUNT(*) FROM context_commit_idempotency WHERE context_id = $1)",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count guarded replay records");
        assert_eq!(counts, (3, 2, 1));

        let branch = sqlx::query_as::<_, (i64, Option<Uuid>)>(
            "SELECT revision, head_commit_id FROM context_branches WHERE context_id = $1 AND branch_name = 'main'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("read guarded replay branch");
        assert_eq!(branch.0, 1);
        assert!(branch.1.is_some());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_guarded_component_content_update_replays_and_projects_revision() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let component_id = ComponentId::from_uuid(
            "55555555-5555-4555-8555-555555555550"
                .parse()
                .expect("seed component uuid"),
        );
        let previous_content_hash =
            ContentHash::new("sha256:seed-system-contract").expect("seed content hash");
        let content = ComponentContent::new("You are the revised support contract.");
        let captured_at = Utc
            .with_ymd_and_hms(2026, 7, 14, 16, 0, 0)
            .single()
            .expect("revision capture time");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Revise seed system contract",
            Vec::new(),
            vec![ContextChange::updated_component_content(
                component_id,
                ContextComponentKind::SystemPrompt,
                previous_content_hash.clone(),
                content.content_hash(),
                "Revise seed system contract",
            )],
            captured_at,
        )
        .expect("content revision commit");
        let commit_id = commit.id();
        let snapshot = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            commit,
            ContextGraph::new(),
            captured_at,
            1,
        )
        .expect("commit snapshot");
        let command = GuardedContextCommitWrite::new(
            test_principal(),
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("component-content-revision-001").expect("idempotency key"),
            RequestDigest::new("sha256:component-content-revision-001").expect("request digest"),
            snapshot,
        )
        .expect("guarded command")
        .with_component_content_revision(ComponentContentRevisionWrite::new(
            component_id,
            ContextComponentKind::SystemPrompt,
            previous_content_hash.clone(),
            content.clone(),
            captured_at,
        ))
        .expect("matching component revision");

        let created = repository
            .create_guarded_commit_snapshot(command.clone())
            .await
            .expect("create component revision");
        let replayed = repository
            .create_guarded_commit_snapshot(command)
            .await
            .expect("replay component revision");
        let revision = repository
            .get_component_content_revision(context_id, commit_id, component_id)
            .await
            .expect("read component revision")
            .expect("persisted component revision");
        let component = repository
            .get_component(context_id.to_string(), component_id.to_string())
            .await
            .expect("read component projection");
        let revision_count = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM context_component_content_revisions WHERE context_id = $1 AND commit_id = $2 AND component_id = $3",
        )
        .bind(context_id.as_uuid())
        .bind(commit_id.as_uuid())
        .bind(component_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count component revisions");

        assert_eq!(created.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(
            revision.previous_content_hash(),
            Some(&previous_content_hash)
        );
        assert_eq!(revision.content(), &content);
        assert_eq!(revision.resulting_content_hash(), &content.content_hash());
        assert_eq!(revision.captured_at(), captured_at);
        assert_eq!(component.content_hash, content.content_hash().as_str());
        assert_eq!(revision_count.0, 1);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_guarded_descriptor_revision_updates_current_component_projection() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let component_id = ComponentId::from_uuid(
            "55555555-5555-4555-8555-555555555550"
                .parse()
                .expect("seed component uuid"),
        );
        let content_hash =
            ContentHash::new("sha256:seed-system-contract").expect("seed content hash");
        let captured_at = Utc
            .with_ymd_and_hms(2026, 7, 18, 10, 0, 0)
            .single()
            .expect("descriptor capture time");
        let component = ContextComponent::with_id(
            component_id,
            ContextComponentKind::SystemPrompt,
            "Localized System Contract",
            content_hash.as_str(),
        )
        .expect("replacement component descriptor");
        let metadata = serde_json::json!({"locale": "zh-CN", "reviewed": true});
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{component_id}"),
                    GraphNodeKind::Prompt,
                    "Localized System Contract",
                )
                .expect("component node"),
            )
            .expect("add component node");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Localize current component descriptor",
            Vec::new(),
            vec![
                ContextChange::updated_component_descriptor(
                    component_id,
                    ContextComponentKind::SystemPrompt,
                    "Localized System Contract",
                    metadata.clone(),
                    "Localize current component descriptor",
                )
                .expect("descriptor change"),
            ],
            captured_at,
        )
        .expect("descriptor commit");
        let snapshot =
            CreateContextCommitSnapshot::new(seeded_project_id(), commit, graph, captured_at, 1)
                .expect("descriptor snapshot");
        let command = GuardedContextCommitWrite::new(
            test_principal(),
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("descriptor-projection-001").expect("idempotency key"),
            RequestDigest::new("sha256:descriptor-projection-001").expect("request digest"),
            snapshot,
        )
        .expect("guarded descriptor command")
        .with_component_descriptor_revision(ComponentDescriptorRevisionWrite::new(
            component,
            metadata.clone(),
            captured_at,
        ))
        .expect("matching descriptor attachment");

        let created = repository
            .create_guarded_commit_snapshot(command.clone())
            .await
            .expect("create descriptor revision");
        let replayed = repository
            .create_guarded_commit_snapshot(command)
            .await
            .expect("replay descriptor revision");
        let current = repository
            .get_component(context_id.to_string(), component_id.to_string())
            .await
            .expect("read current descriptor projection");

        assert_eq!(created.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(current.name, "Localized System Contract");
        assert_eq!(current.metadata, metadata);
        assert_eq!(current.content_hash, content_hash.as_str());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_lifecycle_initialization_creates_and_replays_an_unborn_branch_root() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let command = crate::ContextLifecycleCommand::initialize(
            test_principal(),
            context_id,
            BranchName::new("initialize-root").expect("branch"),
            IdempotencyKey::new("postgres-lifecycle-initialize-001").expect("idempotency key"),
            RequestDigest::new("sha256:postgres-lifecycle-initialize-001").expect("request digest"),
            "Initialize Context lifecycle",
            Utc::now(),
        );

        let created = crate::ContextLifecycleService::new(&repository)
            .execute(command.clone())
            .await
            .expect("initialize Context branch");
        let replayed = crate::ContextLifecycleService::new(&repository)
            .execute(command)
            .await
            .expect("replay Context initialization");
        let branch_head = sqlx::query_scalar::<_, Uuid>(
            "SELECT head_commit_id FROM context_branches WHERE context_id = $1 AND branch_name = 'initialize-root'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("initialized branch head");
        let persisted_changes = sqlx::query_scalar::<_, serde_json::Value>(
            "SELECT changes FROM context_commits WHERE context_id = $1 AND id = $2",
        )
        .bind(context_id.as_uuid())
        .bind(created.commit_id().as_uuid())
        .fetch_one(&pool)
        .await
        .expect("read persisted root changes");
        let root_changes: Vec<ContextChange> =
            serde_json::from_value(persisted_changes).expect("decode root changes");
        let root_change = root_changes.first().expect("one root change");
        let persisted_counts = sqlx::query_as::<_, (i64, i64, i64, i64, i64, i64, i64)>(
            "SELECT \
                (SELECT COUNT(*) FROM context_commits WHERE context_id = $1 AND id = $2), \
                (SELECT COUNT(*) FROM context_commit_parents WHERE context_id = $1 AND commit_id = $2), \
                (SELECT COUNT(*) FROM context_commit_graph_snapshots WHERE commit_id = $2), \
                (SELECT COUNT(*) FROM context_branches WHERE context_id = $1 AND branch_name = 'initialize-root' AND head_commit_id = $2), \
                (SELECT COUNT(*) FROM context_commit_idempotency WHERE context_id = $1 AND branch_name = 'initialize-root' AND idempotency_key = 'postgres-lifecycle-initialize-001' AND commit_id = $2), \
                (SELECT COUNT(*) FROM context_components WHERE context_id = $1), \
                (SELECT COUNT(*) FROM context_component_content_revisions WHERE context_id = $1 AND commit_id = $2)",
        )
        .bind(context_id.as_uuid())
        .bind(created.commit_id().as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count persisted initialization records");

        assert_eq!(
            created.disposition(),
            GuardedCommitWriteDisposition::Created
        );
        assert_eq!(
            replayed.disposition(),
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(created.commit_id(), replayed.commit_id());
        assert_eq!(branch_head, created.commit_id().as_uuid());
        // The lifecycle test uses the seeded workspace; initialization adds no new components.
        assert_eq!(persisted_counts, (1, 0, 1, 1, 1, 6, 0));
        assert_eq!(root_changes.len(), 1);
        assert_eq!(root_change.kind(), ContextChangeKind::CreatedContext);
        assert_eq!(root_change.component_id(), None);
        assert_eq!(root_change.component_kind(), None);
        assert_eq!(root_change.component_name(), None);
        assert_eq!(root_change.component_metadata(), None);
        assert_eq!(root_change.previous_content_hash(), None);
        assert_eq!(root_change.resulting_content_hash(), None);
        assert_eq!(created.snapshot().graph().nodes().len(), 1);
        assert!(created.snapshot().graph().edges().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_lifecycle_replays_typed_uses_relationship_addition_and_removal() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let lifecycle = crate::ContextLifecycleService::new(&repository);
        let captured_at = Utc
            .with_ymd_and_hms(2026, 7, 18, 9, 0, 0)
            .single()
            .expect("lifecycle capture time");
        let root = lifecycle
            .execute(crate::ContextLifecycleCommand::initialize(
                test_principal(),
                context_id,
                BranchName::default(),
                IdempotencyKey::new("postgres-uses-initialize-001").expect("idempotency key"),
                RequestDigest::new("sha256:postgres-uses-initialize-001").expect("request digest"),
                "Initialize Uses lifecycle",
                captured_at,
            ))
            .await
            .expect("initialize Context branch");
        let source = lifecycle
            .execute(crate::ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                root.commit_id(),
                IdempotencyKey::new("postgres-uses-create-source-001").expect("idempotency key"),
                RequestDigest::new("sha256:postgres-uses-create-source-001")
                    .expect("request digest"),
                "Create Uses source",
                ContextComponentKind::Prompt,
                "Uses source",
                serde_json::json!({"role": "source"}),
                ComponentContent::new("Source body."),
                captured_at,
            ))
            .await
            .expect("create source component");
        let target = lifecycle
            .execute(crate::ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                source.commit_id(),
                IdempotencyKey::new("postgres-uses-create-target-001").expect("idempotency key"),
                RequestDigest::new("sha256:postgres-uses-create-target-001")
                    .expect("request digest"),
                "Create Uses target",
                ContextComponentKind::Knowledge,
                "Uses target",
                serde_json::json!({"role": "target"}),
                ComponentContent::new("Target body."),
                captured_at,
            ))
            .await
            .expect("create target component");
        let state = lifecycle
            .read_state_at_commit(context_id, target.commit_id())
            .await
            .expect("read component state");
        let source_component_id = state
            .components()
            .iter()
            .find(|component| component.state().component().name().as_str() == "Uses source")
            .expect("source component state")
            .state()
            .component()
            .id();
        let target_component_id = state
            .components()
            .iter()
            .find(|component| component.state().component().name().as_str() == "Uses target")
            .expect("target component state")
            .state()
            .component()
            .id();
        let add_command = crate::ContextLifecycleCommand::add_uses_relationship(
            test_principal(),
            context_id,
            BranchName::default(),
            target.commit_id(),
            IdempotencyKey::new("postgres-uses-add-001").expect("idempotency key"),
            RequestDigest::new("sha256:postgres-uses-add-001").expect("request digest"),
            "Add Uses relationship",
            source_component_id,
            target_component_id,
            captured_at,
        );
        let added = lifecycle
            .execute(add_command.clone())
            .await
            .expect("add Uses relationship");
        let replayed = lifecycle
            .execute(add_command)
            .await
            .expect("replay Uses relationship");
        let removed = lifecycle
            .execute(crate::ContextLifecycleCommand::remove_uses_relationship(
                test_principal(),
                context_id,
                BranchName::default(),
                added.commit_id(),
                IdempotencyKey::new("postgres-uses-remove-001").expect("idempotency key"),
                RequestDigest::new("sha256:postgres-uses-remove-001").expect("request digest"),
                "Remove Uses relationship",
                source_component_id,
                target_component_id,
                captured_at,
            ))
            .await
            .expect("remove Uses relationship");
        let branch = sqlx::query_as::<_, (i64, Uuid)>(
            "SELECT revision, head_commit_id FROM context_branches WHERE context_id = $1 AND branch_name = 'main'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("read branch head");

        assert_eq!(root.disposition(), GuardedCommitWriteDisposition::Created);
        assert_eq!(source.disposition(), GuardedCommitWriteDisposition::Created);
        assert_eq!(target.disposition(), GuardedCommitWriteDisposition::Created);
        assert_eq!(added.disposition(), GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition(),
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(replayed.commit_id(), added.commit_id());
        assert_eq!(state.components().len(), 2);
        assert!(added.snapshot().graph().edges().iter().any(|edge| {
            edge.source().as_str() == format!("component:{source_component_id}")
                && edge.target().as_str() == format!("component:{target_component_id}")
                && edge.kind() == GraphEdgeKind::Uses
        }));
        assert!(!removed.snapshot().graph().edges().iter().any(|edge| {
            edge.source().as_str() == format!("component:{source_component_id}")
                && edge.target().as_str() == format!("component:{target_component_id}")
                && edge.kind() == GraphEdgeKind::Uses
        }));
        assert_eq!(branch.0, 5);
        assert_eq!(branch.1, removed.commit_id().as_uuid());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_guarded_component_content_creation_replays_and_projects_initial_revision() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let component_id = ComponentId::from_uuid(
            "88888888-8888-4888-8888-888888888881"
                .parse()
                .expect("new component uuid"),
        );
        let metadata = serde_json::json!({"locale": "en", "visibility": "private"});
        let content = ComponentContent::new("You are the initial support contract.");
        let captured_at = Utc
            .with_ymd_and_hms(2026, 7, 15, 8, 0, 0)
            .single()
            .expect("creation capture time");
        let creation = ComponentContentCreationWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            "Initial Support Prompt",
            metadata.clone(),
            content.clone(),
            captured_at,
        )
        .expect("valid creation");
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Seed Support Resolution Context",
                )
                .expect("context node"),
            )
            .expect("add context node");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{component_id}"),
                    GraphNodeKind::Prompt,
                    "Initial Support Prompt",
                )
                .expect("component node"),
            )
            .expect("add component node");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    format!("component:{component_id}"),
                    GraphEdgeKind::Contains,
                )
                .expect("component relationship"),
            )
            .expect("add component relationship");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create initial support prompt",
            Vec::new(),
            vec![
                ContextChange::added_component_content_with_details(
                    component_id,
                    ContextComponentKind::Prompt,
                    "Initial Support Prompt",
                    metadata.clone(),
                    content.content_hash(),
                    "Create initial support prompt",
                )
                .expect("replayable component change"),
            ],
            captured_at,
        )
        .expect("creation commit");
        let commit_id = commit.id();
        let snapshot =
            CreateContextCommitSnapshot::new(seeded_project_id(), commit, graph, captured_at, 1)
                .expect("commit snapshot");
        let command = GuardedContextCommitWrite::new(
            test_principal(),
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("component-content-creation-001").expect("idempotency key"),
            RequestDigest::new("sha256:component-content-creation-001").expect("request digest"),
            snapshot,
        )
        .expect("guarded command")
        .with_component_content_creation(creation)
        .expect("matching component creation");

        let created = repository
            .create_guarded_commit_snapshot(command.clone())
            .await
            .expect("create component");
        let replayed = repository
            .create_guarded_commit_snapshot(command)
            .await
            .expect("replay component creation");
        let revision = repository
            .get_component_content_revision(context_id, commit_id, component_id)
            .await
            .expect("read revision")
            .expect("initial revision exists");
        let component = repository
            .get_component(context_id.to_string(), component_id.to_string())
            .await
            .expect("read component projection");
        let state_at_commit = repository
            .get_component_state_at_commit(context_id, commit_id, component_id)
            .await
            .expect("replay created component state")
            .expect("created component state is replayable");
        let counts = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM context_components WHERE context_id = $1 AND id = $2), (SELECT COUNT(*) FROM context_component_content_revisions WHERE context_id = $1 AND commit_id = $3 AND component_id = $2), (SELECT COUNT(*) FROM context_commit_graph_snapshots WHERE commit_id = $3), (SELECT COUNT(*) FROM context_commit_idempotency WHERE context_id = $1 AND commit_id = $3)",
        )
        .bind(context_id.as_uuid())
        .bind(component_id.as_uuid())
        .bind(commit_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count created records");
        let branch = sqlx::query_as::<_, (i64, Option<Uuid>)>(
            "SELECT revision, head_commit_id FROM context_branches WHERE context_id = $1 AND branch_name = 'main'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("read branch head");

        assert_eq!(created.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(revision.previous_content_hash(), None);
        assert_eq!(revision.content(), &content);
        assert_eq!(component.name, "Initial Support Prompt");
        assert_eq!(component.metadata, metadata);
        assert_eq!(component.content_hash, content.content_hash().as_str());
        assert_eq!(
            state_at_commit.component().name().as_str(),
            "Initial Support Prompt"
        );
        assert_eq!(state_at_commit.metadata(), &metadata);
        assert_eq!(
            state_at_commit.component().content_hash(),
            &content.content_hash()
        );
        assert_eq!(state_at_commit.creation_commit_id(), commit_id);
        assert_eq!(state_at_commit.content_commit_id(), commit_id);
        assert_eq!(counts, (1, 1, 1, 1));
        assert_eq!(branch.0, 1);
        assert_eq!(branch.1, Some(commit_id.as_uuid()));

        let duplicate_creation = ComponentContentCreationWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            "Initial Support Prompt",
            metadata.clone(),
            content.clone(),
            captured_at,
        )
        .expect("valid duplicate creation command");
        let duplicate_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Attempt duplicate support prompt",
            vec![commit_id],
            vec![
                ContextChange::added_component_content_with_details(
                    component_id,
                    ContextComponentKind::Prompt,
                    "Initial Support Prompt",
                    metadata.clone(),
                    content.content_hash(),
                    "Attempt duplicate support prompt",
                )
                .expect("replayable duplicate change"),
            ],
            captured_at,
        )
        .expect("duplicate commit");
        let duplicate_snapshot = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            duplicate_commit,
            created.snapshot.graph().clone(),
            captured_at,
            1,
        )
        .expect("duplicate snapshot");
        let duplicate_error = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    test_principal(),
                    ExpectedBranchHead::Commit(commit_id),
                    IdempotencyKey::new("component-content-creation-duplicate-001")
                        .expect("duplicate idempotency key"),
                    RequestDigest::new("sha256:component-content-creation-duplicate-001")
                        .expect("duplicate request digest"),
                    duplicate_snapshot,
                )
                .expect("duplicate guarded command")
                .with_component_content_creation(duplicate_creation)
                .expect("matching duplicate creation"),
            )
            .await
            .expect_err("duplicate component creation must fail");
        let post_failure_counts = sqlx::query_as::<_, (i64, i64, i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM context_components WHERE context_id = $1 AND id = $2), (SELECT COUNT(*) FROM context_component_content_revisions WHERE context_id = $1 AND component_id = $2), (SELECT COUNT(*) FROM context_commits WHERE context_id = $1), (SELECT COUNT(*) FROM context_commit_idempotency WHERE context_id = $1)",
        )
        .bind(context_id.as_uuid())
        .bind(component_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count records after duplicate failure");
        let post_failure_branch = sqlx::query_as::<_, (i64, Option<Uuid>)>(
            "SELECT revision, head_commit_id FROM context_branches WHERE context_id = $1 AND branch_name = 'main'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("read branch after duplicate failure");

        assert!(matches!(
            duplicate_error,
            StorageRepositoryError::ComponentContentRevisionConflict { .. }
        ));
        assert_eq!(post_failure_counts, (1, 1, 3, 1));
        assert_eq!(post_failure_branch.0, 1);
        assert_eq!(post_failure_branch.1, Some(commit_id.as_uuid()));

        let stale_removal_hash =
            ContentHash::new("sha256:stale-removal").expect("stale removal hash");
        let stale_removal_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Reject stale initial support prompt removal",
            vec![commit_id],
            vec![ContextChange::removed_component(
                component_id,
                ContextComponentKind::Prompt,
                stale_removal_hash.clone(),
                "Reject stale initial support prompt removal",
            )],
            captured_at,
        )
        .expect("stale removal commit");
        let stale_removal_commit_id = stale_removal_commit.id();
        let stale_removal_snapshot = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            stale_removal_commit,
            ContextGraph::new(),
            captured_at,
            1,
        )
        .expect("stale removal snapshot");
        let stale_removal_error = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    test_principal(),
                    ExpectedBranchHead::Commit(commit_id),
                    IdempotencyKey::new("component-removal-stale-001").expect("idempotency key"),
                    RequestDigest::new("sha256:component-removal-stale-001")
                        .expect("request digest"),
                    stale_removal_snapshot,
                )
                .expect("guarded stale removal command")
                .with_component_removal(crate::ComponentRemovalWrite::new(
                    component_id,
                    ContextComponentKind::Prompt,
                    stale_removal_hash,
                ))
                .expect("matching stale removal attachment"),
            )
            .await
            .expect_err("stale removal must not persist a commit");
        assert!(matches!(
            stale_removal_error,
            StorageRepositoryError::ComponentContentRevisionConflict { .. }
        ));
        assert!(matches!(
            repository
                .get_commit(context_id.to_string(), stale_removal_commit_id.to_string())
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));

        let removal_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Remove initial support prompt",
            vec![commit_id],
            vec![ContextChange::removed_component(
                component_id,
                ContextComponentKind::Prompt,
                content.content_hash(),
                "Remove initial support prompt",
            )],
            captured_at,
        )
        .expect("removal commit");
        let removal_commit_id = removal_commit.id();
        let removal_snapshot = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            removal_commit,
            ContextGraph::new(),
            captured_at,
            1,
        )
        .expect("removal snapshot");
        let removal_command = GuardedContextCommitWrite::new(
            test_principal(),
            ExpectedBranchHead::Commit(commit_id),
            IdempotencyKey::new("component-removal-001").expect("idempotency key"),
            RequestDigest::new("sha256:component-removal-001").expect("request digest"),
            removal_snapshot,
        )
        .expect("guarded removal command")
        .with_component_removal(crate::ComponentRemovalWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            content.content_hash(),
        ))
        .expect("matching component removal");
        let removed = repository
            .create_guarded_commit_snapshot(removal_command.clone())
            .await
            .expect("remove component");
        let removal_replayed = repository
            .create_guarded_commit_snapshot(removal_command)
            .await
            .expect("replay component removal");
        let removed_snapshot = repository
            .get_context_component_state_snapshot_at_commit(context_id, removal_commit_id)
            .await
            .expect("replay removed Context inventory");

        assert_eq!(removed.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            removal_replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert!(matches!(
            repository
                .get_component(context_id.to_string(), component_id.to_string())
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));
        assert!(
            repository
                .get_component_state_at_commit(context_id, removal_commit_id, component_id)
                .await
                .expect("replay removed component")
                .is_none()
        );
        assert!(removed_snapshot.components().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn component_content_initial_revision_migration_preserves_history_and_enforces_null_prior()
     {
        let Some(pool) = unseeded_disposable_test_pool(1).await else {
            return;
        };
        sqlx::raw_sql(PRE_INITIAL_COMPONENT_REVISIONS_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply schema through initial revision migration predecessor");
        sqlx::raw_sql(WORKSPACE_GRAPH_SEED)
            .execute(&pool)
            .await
            .expect("seed predecessor schema");
        let context_id = seeded_context_id().as_uuid();
        let commit_id =
            Uuid::parse_str("77777777-7777-4777-8777-777777777777").expect("seed commit uuid");
        let existing_component_id =
            Uuid::parse_str("55555555-5555-4555-8555-555555555550").expect("seed component uuid");
        sqlx::query(
            "INSERT INTO context_component_content_revisions (context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at) VALUES ($1, $2, $3, 'sha256:legacy-prior', 'sha256:legacy-result', 'legacy body', now())",
        )
        .bind(context_id)
        .bind(commit_id)
        .bind(existing_component_id)
        .execute(&pool)
        .await
        .expect("insert historical non-null revision");

        sqlx::raw_sql(INITIAL_COMPONENT_REVISIONS_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply nullable-prior migration");
        sqlx::raw_sql(INITIAL_COMPONENT_REVISIONS_INTEGRITY_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply initial revision integrity migration");
        let initial_component_id = Uuid::parse_str("88888888-8888-4888-8888-888888888882")
            .expect("initial component uuid");
        let initial_created_at = Utc
            .timestamp_opt(1_740_000_000, 0)
            .single()
            .expect("fixed initial revision timestamp");
        sqlx::query(
            "INSERT INTO context_components (id, context_id, kind, name, content_hash, metadata, created_at, updated_at) VALUES ($1, $2, 'prompt', 'Initial migration prompt', 'sha256:initial-result', '{}'::jsonb, $3, $3)",
        )
        .bind(initial_component_id)
        .bind(context_id)
        .bind(initial_created_at)
        .execute(&pool)
        .await
        .expect("insert initial component");
        sqlx::query(
            "INSERT INTO context_component_content_revisions (context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at) VALUES ($1, $2, $3, NULL, 'sha256:initial-result', 'initial body', $4)",
        )
        .bind(context_id)
        .bind(commit_id)
        .bind(initial_component_id)
        .bind(initial_created_at)
        .execute(&pool)
        .await
        .expect("insert null-prior initial revision");
        let historical_prior = sqlx::query_as::<_, (String,)>(
            "SELECT previous_content_hash FROM context_component_content_revisions WHERE component_id = $1",
        )
        .bind(existing_component_id)
        .fetch_one(&pool)
        .await
        .expect("read historical revision")
        .0;
        let initial_prior = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT previous_content_hash FROM context_component_content_revisions WHERE component_id = $1",
        )
        .bind(initial_component_id)
        .fetch_one(&pool)
        .await
        .expect("read initial revision")
        .0;

        assert_eq!(historical_prior, "sha256:legacy-prior");
        assert_eq!(initial_prior, None);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn component_content_initial_revision_integrity_rejects_malformed_null_prior() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let context_id = seeded_context_id().as_uuid();
        let created_at = Utc
            .timestamp_opt(1_740_000_000, 0)
            .single()
            .expect("fixed creation timestamp");
        let first_commit_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at, created_at) VALUES ($1, $2, 'main', 'Create component', '[]'::jsonb, $3, $3)",
        )
        .bind(first_commit_id)
        .bind(context_id)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("insert initial component commit");

        let component_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO context_components (id, context_id, kind, name, content_hash, metadata, created_at, updated_at) VALUES ($1, $2, 'prompt', 'Initial prompt', 'sha256:initial', '{}'::jsonb, $3, $3)",
        )
        .bind(component_id)
        .bind(context_id)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("insert component with creation timestamp");
        sqlx::query(
            "INSERT INTO context_component_content_revisions (context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at) VALUES ($1, $2, $3, NULL, 'sha256:initial', 'initial body', $4)",
        )
        .bind(context_id)
        .bind(first_commit_id)
        .bind(component_id)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("insert matching initial revision");

        let second_commit_id = Uuid::new_v4();
        let revised_at = created_at + chrono::Duration::seconds(1);
        sqlx::query(
            "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at, created_at) VALUES ($1, $2, 'main', 'Revise component', '[]'::jsonb, $3, $3)",
        )
        .bind(second_commit_id)
        .bind(context_id)
        .bind(revised_at)
        .execute(&pool)
        .await
        .expect("insert revision commit");
        let non_initial_error = sqlx::query(
            "INSERT INTO context_component_content_revisions (context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at) VALUES ($1, $2, $3, NULL, 'sha256:invalid-second', 'invalid second body', $4)",
        )
        .bind(context_id)
        .bind(second_commit_id)
        .bind(component_id)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect_err("a null prior hash must be rejected after the initial revision");
        assert!(non_initial_error.as_database_error().is_some());

        let mismatched_component_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO context_components (id, context_id, kind, name, content_hash, metadata, created_at, updated_at) VALUES ($1, $2, 'prompt', 'Mismatched prompt', 'sha256:mismatched', '{}'::jsonb, $3, $3)",
        )
        .bind(mismatched_component_id)
        .bind(context_id)
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("insert mismatched component");
        let mismatched_timestamp_error = sqlx::query(
            "INSERT INTO context_component_content_revisions (context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at) VALUES ($1, $2, $3, NULL, 'sha256:mismatched', 'mismatched body', $4)",
        )
        .bind(context_id)
        .bind(second_commit_id)
        .bind(mismatched_component_id)
        .bind(revised_at)
        .execute(&pool)
        .await
        .expect_err("a null prior hash must use the component creation timestamp");
        assert!(mismatched_timestamp_error.as_database_error().is_some());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_component_content_at_commit_replays_nearest_normal_parent_revision() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let context_id = seeded_context_id();
        let component_id = Uuid::new_v4();
        let root_commit_id = Uuid::new_v4();
        let unchanged_commit_id = Uuid::new_v4();
        let revised_commit_id = Uuid::new_v4();
        let merge_commit_id = Uuid::new_v4();
        let created_at = Utc
            .timestamp_opt(1_740_000_000, 0)
            .single()
            .expect("fixed creation timestamp");
        let revised_at = Utc
            .timestamp_opt(1_740_000_003, 0)
            .single()
            .expect("fixed revision timestamp");
        let initial_content = ComponentContent::new("postgres initial replay body");
        let revised_content = ComponentContent::new("postgres revised replay body");

        sqlx::query(
            "INSERT INTO context_components (id, context_id, kind, name, content_hash, metadata, created_at, updated_at) VALUES ($1, $2, 'prompt', 'Replay prompt', 'sha256:replay-initial', '{}'::jsonb, $3, $3)",
        )
        .bind(component_id)
        .bind(context_id.as_uuid())
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("insert replay component");

        for (commit_id, message, authored_at) in [
            (root_commit_id, "Replay root", created_at),
            (unchanged_commit_id, "Replay unchanged child", created_at),
            (revised_commit_id, "Replay revised child", revised_at),
            (merge_commit_id, "Replay merge", revised_at),
        ] {
            sqlx::query(
                "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at, created_at) VALUES ($1, $2, 'main', $3, '[]'::jsonb, $4, $4)",
            )
            .bind(commit_id)
            .bind(context_id.as_uuid())
            .bind(message)
            .bind(authored_at)
            .execute(&pool)
            .await
            .expect("insert replay commit");
        }
        for (commit_id, parent_commit_id, position) in [
            (unchanged_commit_id, root_commit_id, 0_i32),
            (revised_commit_id, unchanged_commit_id, 0_i32),
            (merge_commit_id, revised_commit_id, 0_i32),
            (merge_commit_id, root_commit_id, 1_i32),
        ] {
            sqlx::query(
                "INSERT INTO context_commit_parents (context_id, commit_id, parent_commit_id, position) VALUES ($1, $2, $3, $4)",
            )
            .bind(context_id.as_uuid())
            .bind(commit_id)
            .bind(parent_commit_id)
            .bind(position)
            .execute(&pool)
            .await
            .expect("insert replay parent");
        }
        sqlx::query(
            "INSERT INTO context_component_content_revisions (context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at) VALUES ($1, $2, $3, NULL, $4, $5, $6)",
        )
        .bind(context_id.as_uuid())
        .bind(root_commit_id)
        .bind(component_id)
        .bind(initial_content.content_hash().as_str())
        .bind(initial_content.as_str())
        .bind(created_at)
        .execute(&pool)
        .await
        .expect("insert initial replay revision");
        sqlx::query(
            "INSERT INTO context_component_content_revisions (context_id, commit_id, component_id, previous_content_hash, content_hash, content, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(context_id.as_uuid())
        .bind(revised_commit_id)
        .bind(component_id)
        .bind(initial_content.content_hash().as_str())
        .bind(revised_content.content_hash().as_str())
        .bind(revised_content.as_str())
        .bind(revised_at)
        .execute(&pool)
        .await
        .expect("insert revised replay revision");
        sqlx::query("UPDATE context_commits SET changes = $1 WHERE id = $2")
            .bind(
                serde_json::to_value(vec![
                    ContextChange::added_component_content_with_details(
                        ComponentId::from_uuid(component_id),
                        ContextComponentKind::Prompt,
                        "Replay prompt",
                        serde_json::json!({"source": "postgres"}),
                        initial_content.content_hash(),
                        "Create replay prompt",
                    )
                    .expect("replayable creation change"),
                ])
                .expect("serialize creation change"),
            )
            .bind(root_commit_id)
            .execute(&pool)
            .await
            .expect("record replayable creation change");
        sqlx::query("UPDATE context_commits SET changes = $1 WHERE id = $2")
            .bind(
                serde_json::to_value(vec![ContextChange::updated_component_content(
                    ComponentId::from_uuid(component_id),
                    ContextComponentKind::Prompt,
                    initial_content.content_hash(),
                    revised_content.content_hash(),
                    "Revise replay prompt",
                )])
                .expect("serialize revision change"),
            )
            .bind(revised_commit_id)
            .execute(&pool)
            .await
            .expect("record replayable revision change");

        let repository = PostgresContextGraphRepository::new(pool.clone());
        let component_id = ComponentId::from_uuid(component_id);
        let root_commit = CommitId::from_uuid(root_commit_id);
        let unchanged_commit = CommitId::from_uuid(unchanged_commit_id);
        let revised_commit = CommitId::from_uuid(revised_commit_id);
        let merge_commit = CommitId::from_uuid(merge_commit_id);
        let unchanged = repository
            .get_component_content_at_commit(context_id, unchanged_commit, component_id)
            .await
            .expect("resolve unchanged child")
            .expect("initial revision is reachable");
        let revised = repository
            .get_component_content_at_commit(context_id, revised_commit, component_id)
            .await
            .expect("resolve revised child")
            .expect("revised revision is reachable");
        assert_eq!(unchanged.commit_id(), root_commit);
        assert_eq!(unchanged.content(), &initial_content);
        assert_eq!(revised.commit_id(), revised_commit);
        assert_eq!(revised.content(), &revised_content);
        let unchanged_state = repository
            .get_component_state_at_commit(context_id, unchanged_commit, component_id)
            .await
            .expect("replay unchanged component state")
            .expect("creation state is reachable");
        let revised_state = repository
            .get_component_state_at_commit(context_id, revised_commit, component_id)
            .await
            .expect("replay revised component state")
            .expect("revised state is reachable");
        assert_eq!(unchanged_state.component().name().as_str(), "Replay prompt");
        assert_eq!(
            unchanged_state.metadata(),
            &serde_json::json!({"source": "postgres"})
        );
        assert_eq!(
            unchanged_state.component().content_hash(),
            &initial_content.content_hash()
        );
        assert_eq!(unchanged_state.creation_commit_id(), root_commit);
        assert_eq!(unchanged_state.content_commit_id(), root_commit);
        assert_eq!(
            revised_state.component().content_hash(),
            &revised_content.content_hash()
        );
        assert_eq!(revised_state.creation_commit_id(), root_commit);
        assert_eq!(revised_state.content_commit_id(), revised_commit);
        let unchanged_context_snapshot = repository
            .get_context_component_state_snapshot_at_commit(context_id, unchanged_commit)
            .await
            .expect("replay unchanged PostgreSQL Context component inventory");
        let context_snapshot = repository
            .get_context_component_state_snapshot_at_commit(context_id, revised_commit)
            .await
            .expect("replay PostgreSQL Context component inventory");
        assert_eq!(unchanged_context_snapshot.components().len(), 1);
        assert_eq!(
            unchanged_context_snapshot.components()[0]
                .component()
                .content_hash(),
            &initial_content.content_hash()
        );
        assert_eq!(context_snapshot.context_id(), context_id);
        assert_eq!(context_snapshot.target_commit_id(), revised_commit);
        assert_eq!(context_snapshot.components().len(), 1);
        assert_eq!(
            context_snapshot.components()[0].component().content_hash(),
            &revised_content.content_hash()
        );
        assert_eq!(
            repository
                .get_component_state_at_commit(context_id, root_commit, ComponentId::new())
                .await
                .expect("resolve component state without replayable creation"),
            None
        );
        assert_eq!(
            repository
                .get_component_content_at_commit(context_id, root_commit, ComponentId::new())
                .await
                .expect("resolve component without body"),
            None
        );
        assert!(matches!(
            repository
                .get_component_content_at_commit(context_id, CommitId::new(), component_id)
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));
        assert!(matches!(
            repository
                .get_component_state_at_commit(context_id, CommitId::new(), component_id)
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));
        let unknown_context_id = ContextId::new();
        let expected_unknown_context_scope = format!("context:{unknown_context_id}");
        let unknown_context_error = repository
            .get_component_content_at_commit(unknown_context_id, root_commit, component_id)
            .await
            .expect_err("unknown context must be distinguishable from an unknown commit");
        assert!(matches!(
            unknown_context_error,
            StorageRepositoryError::ScopeUnavailable { scope }
                if scope == expected_unknown_context_scope
        ));
        assert!(matches!(
            repository
                .get_component_state_at_commit(unknown_context_id, root_commit, component_id)
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));
        assert!(matches!(
            repository
                .get_component_content_at_commit(context_id, merge_commit, component_id)
                .await,
            Err(StorageRepositoryError::ComponentContentRevisionConflict { .. })
        ));
        assert!(matches!(
            repository
                .get_component_state_at_commit(context_id, merge_commit, component_id)
                .await,
            Err(StorageRepositoryError::ComponentStateReplayConflict { .. })
        ));

        let foreign_context_id = Uuid::new_v4();
        let foreign_commit_id = Uuid::new_v4();
        let malformed_child_commit_id = CommitId::new();
        sqlx::query("INSERT INTO contexts (id, project_id, name) VALUES ($1, $2, $3)")
            .bind(foreign_context_id)
            .bind(
                Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("seed project uuid"),
            )
            .bind("Foreign replay context")
            .execute(&pool)
            .await
            .expect("insert foreign replay context");
        for (commit_id, commit_context_id, message) in [
            (foreign_commit_id, foreign_context_id, "Foreign replay root"),
            (
                malformed_child_commit_id.as_uuid(),
                context_id.as_uuid(),
                "Malformed cross-context replay child",
            ),
        ] {
            sqlx::query(
                "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at, created_at) VALUES ($1, $2, 'main', $3, '[]'::jsonb, $4, $4)",
            )
            .bind(commit_id)
            .bind(commit_context_id)
            .bind(message)
            .bind(revised_at)
            .execute(&pool)
            .await
            .expect("insert malformed replay commit");
        }
        sqlx::query(
            "ALTER TABLE context_commit_parents DROP CONSTRAINT fk_context_commit_parents_parent_same_context",
        )
        .execute(&pool)
        .await
        .expect("remove parent scope constraint to emulate a legacy-corrupted history");
        sqlx::query(
            "INSERT INTO context_commit_parents (context_id, commit_id, parent_commit_id, position) VALUES ($1, $2, $3, 0)",
        )
        .bind(context_id.as_uuid())
        .bind(malformed_child_commit_id.as_uuid())
        .bind(foreign_commit_id)
        .execute(&pool)
        .await
        .expect("insert malformed cross-context parent");

        assert!(matches!(
            repository
                .get_component_content_at_commit(
                    context_id,
                    malformed_child_commit_id,
                    component_id
                )
                .await,
            Err(StorageRepositoryError::ComponentContentRevisionConflict { .. })
        ));
        assert!(matches!(
            repository
                .get_component_state_at_commit(context_id, malformed_child_commit_id, component_id)
                .await,
            Err(StorageRepositoryError::ComponentStateReplayConflict { .. })
        ));
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_context_commit_parent_scope_rejects_a_cross_context_direct_insert() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let context_id = seeded_context_id().as_uuid();
        let foreign_context_id = Uuid::new_v4();
        let local_child_commit_id = Uuid::new_v4();
        let foreign_parent_commit_id = Uuid::new_v4();

        sqlx::query("INSERT INTO contexts (id, project_id, name) VALUES ($1, $2, $3)")
            .bind(foreign_context_id)
            .bind(
                Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("seed project uuid"),
            )
            .bind("Foreign parent scope context")
            .execute(&pool)
            .await
            .expect("insert foreign context");
        for (commit_id, commit_context_id, message) in [
            (local_child_commit_id, context_id, "Local child commit"),
            (
                foreign_parent_commit_id,
                foreign_context_id,
                "Foreign parent commit",
            ),
        ] {
            sqlx::query(
                "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at) VALUES ($1, $2, 'main', $3, '[]'::jsonb, now())",
            )
            .bind(commit_id)
            .bind(commit_context_id)
            .bind(message)
            .execute(&pool)
            .await
            .expect("insert commit for parent-scope check");
        }

        let error = sqlx::query(
            "INSERT INTO context_commit_parents (context_id, commit_id, parent_commit_id, position) VALUES ($1, $2, $3, 0)",
        )
        .bind(context_id)
        .bind(local_child_commit_id)
        .bind(foreign_parent_commit_id)
        .execute(&pool)
        .await
        .expect_err("a cross-context parent must be rejected by a foreign-key constraint");

        assert_eq!(
            error
                .as_database_error()
                .and_then(|database_error| database_error.code())
                .as_deref(),
            Some("23503")
        );

        let error = sqlx::query(
            "INSERT INTO context_commit_parents (context_id, commit_id, parent_commit_id, position) VALUES ($1, $2, $3, 0)",
        )
        .bind(foreign_context_id)
        .bind(local_child_commit_id)
        .bind(foreign_parent_commit_id)
        .execute(&pool)
        .await
        .expect_err("a child commit must use its owning context");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|database_error| database_error.code())
                .as_deref(),
            Some("23503")
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn parent_scope_migration_backfills_valid_commit_parent_history() {
        let Some(pool) = unseeded_disposable_test_pool(1).await else {
            return;
        };
        apply_parent_scope_predecessor(&pool).await;
        let context_id = seeded_context_id().as_uuid();
        let root_commit_id = Uuid::new_v4();
        let child_commit_id = Uuid::new_v4();
        for (commit_id, message) in [
            (root_commit_id, "Historical parent root"),
            (child_commit_id, "Historical parent child"),
        ] {
            sqlx::query(
                "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at) VALUES ($1, $2, 'main', $3, '[]'::jsonb, now())",
            )
            .bind(commit_id)
            .bind(context_id)
            .bind(message)
            .execute(&pool)
            .await
            .expect("insert historical commit");
        }
        sqlx::query(
            "INSERT INTO context_commit_parents (commit_id, parent_commit_id, position) VALUES ($1, $2, 0)",
        )
        .bind(child_commit_id)
        .bind(root_commit_id)
        .execute(&pool)
        .await
        .expect("insert valid historical parent before migration");

        sqlx::raw_sql(PARENT_SCOPE_INTEGRITY_MIGRATION)
            .execute(&pool)
            .await
            .expect("backfill valid parent history");

        let stored_context_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT context_id FROM context_commit_parents WHERE commit_id = $1",
        )
        .bind(child_commit_id)
        .fetch_one(&pool)
        .await
        .expect("read backfilled parent context");
        assert_eq!(stored_context_id, context_id);
        let parent_delete_error = sqlx::query("DELETE FROM context_commits WHERE id = $1")
            .bind(root_commit_id)
            .execute(&pool)
            .await
            .expect_err("a referenced parent commit must remain restricted");
        assert_eq!(
            parent_delete_error
                .as_database_error()
                .and_then(|database_error| database_error.code())
                .as_deref(),
            Some("23503")
        );
        sqlx::query("DELETE FROM context_commits WHERE id = $1")
            .bind(child_commit_id)
            .execute(&pool)
            .await
            .expect("deleting a child must cascade its parent link");
        let parent_link_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*)::BIGINT FROM context_commit_parents WHERE parent_commit_id = $1",
        )
        .bind(root_commit_id)
        .fetch_one(&pool)
        .await
        .expect("count parent links after child deletion");
        assert_eq!(parent_link_count, 0);
        sqlx::query("DELETE FROM context_commits WHERE id = $1")
            .bind(root_commit_id)
            .execute(&pool)
            .await
            .expect("unreferenced parent may be deleted");
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn parent_scope_migration_rejects_legacy_cross_context_parent_history() {
        let Some(pool) = unseeded_disposable_test_pool(1).await else {
            return;
        };
        apply_parent_scope_predecessor(&pool).await;
        let context_id = seeded_context_id().as_uuid();
        let foreign_context_id = Uuid::new_v4();
        let local_child_commit_id = Uuid::new_v4();
        let foreign_parent_commit_id = Uuid::new_v4();
        sqlx::query("INSERT INTO contexts (id, project_id, name) VALUES ($1, $2, $3)")
            .bind(foreign_context_id)
            .bind(
                Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("seed project uuid"),
            )
            .bind("Historical foreign parent context")
            .execute(&pool)
            .await
            .expect("insert historical foreign context");
        for (commit_id, commit_context_id, message) in [
            (local_child_commit_id, context_id, "Historical local child"),
            (
                foreign_parent_commit_id,
                foreign_context_id,
                "Historical foreign parent",
            ),
        ] {
            sqlx::query(
                "INSERT INTO context_commits (id, context_id, branch_name, message, changes, authored_at) VALUES ($1, $2, 'main', $3, '[]'::jsonb, now())",
            )
            .bind(commit_id)
            .bind(commit_context_id)
            .bind(message)
            .execute(&pool)
            .await
            .expect("insert historical commit");
        }
        sqlx::query(
            "INSERT INTO context_commit_parents (commit_id, parent_commit_id, position) VALUES ($1, $2, 0)",
        )
        .bind(local_child_commit_id)
        .bind(foreign_parent_commit_id)
        .execute(&pool)
        .await
        .expect("insert malformed historical parent before migration");

        let error = sqlx::raw_sql(PARENT_SCOPE_INTEGRITY_MIGRATION)
            .execute(&pool)
            .await
            .expect_err("the migration must reject malformed historical parent scope");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|database_error| database_error.code())
                .as_deref(),
            Some("23503")
        );

        let context_column_exists = sqlx::query_scalar::<_, bool>(
            r#"
SELECT EXISTS (
    SELECT 1
    FROM information_schema.columns
    WHERE table_schema = 'public'
      AND table_name = 'context_commit_parents'
      AND column_name = 'context_id'
)
"#,
        )
        .fetch_one(&pool)
        .await
        .expect("inspect parent-scope column after rejected migration");
        let scope_constraint_count = sqlx::query_scalar::<_, i64>(
            r#"
SELECT COUNT(*)
FROM pg_constraint
WHERE conrelid = 'context_commit_parents'::regclass
  AND conname IN (
      'fk_context_commit_parents_child_same_context',
      'fk_context_commit_parents_parent_same_context'
  )
"#,
        )
        .fetch_one(&pool)
        .await
        .expect("inspect parent-scope constraints after rejected migration");

        assert!(!context_column_exists);
        assert_eq!(scope_constraint_count, 0);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_guarded_writer_allows_only_one_concurrent_stale_head_writer() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let (root_id, root_command) = guarded_command(
            context_id,
            Vec::new(),
            ExpectedBranchHead::Unborn,
            "Concurrent stale-head root",
            "request-stale-root",
            "sha256:stale-root",
        );
        let root = repository
            .create_guarded_commit_snapshot(root_command)
            .await
            .expect("create root");
        assert_eq!(root.disposition, GuardedCommitWriteDisposition::Created);

        let (_, first_child) = guarded_command(
            context_id,
            vec![root_id],
            ExpectedBranchHead::Commit(root_id),
            "Concurrent child one",
            "request-child-one",
            "sha256:child-one",
        );
        let (_, second_child) = guarded_command(
            context_id,
            vec![root_id],
            ExpectedBranchHead::Commit(root_id),
            "Concurrent child two",
            "request-child-two",
            "sha256:child-two",
        );
        let first_repository = repository.clone();
        let second_repository = repository.clone();
        let (first, second) = tokio::join!(
            first_repository.create_guarded_commit_snapshot(first_child),
            second_repository.create_guarded_commit_snapshot(second_child),
        );
        let outcomes = [first, second];
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    Ok(result) if result.disposition == GuardedCommitWriteDisposition::Created
                ))
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    Err(StorageRepositoryError::BranchHeadConflict { .. })
                ))
                .count(),
            1
        );

        let counts = sqlx::query_as::<_, (i64, i64, i64)>(
            "SELECT (SELECT COUNT(*) FROM context_commits WHERE context_id = $1), (SELECT COUNT(*) FROM context_commit_graph_snapshots WHERE commit_id IN (SELECT id FROM context_commits WHERE context_id = $1)), (SELECT COUNT(*) FROM context_commit_idempotency WHERE context_id = $1)",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count stale-head records");
        assert_eq!(counts, (4, 3, 2));

        let branch = sqlx::query_as::<_, (i64, Option<Uuid>)>(
            "SELECT revision, head_commit_id FROM context_branches WHERE context_id = $1 AND branch_name = 'main'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("read stale-head branch");
        assert_eq!(branch.0, 2);
        assert!(branch.1.is_some());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_guarded_scope_constraints_reject_cross_context_references() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let primary_context_id = seeded_context_id().as_uuid();
        let foreign_context_id =
            Uuid::parse_str("44444444-4444-4444-8444-888888888888").expect("foreign context uuid");
        let foreign_commit_id =
            Uuid::parse_str("77777777-7777-4777-8777-888888888888").expect("foreign commit uuid");
        sqlx::query("INSERT INTO contexts (id, project_id, name) VALUES ($1, $2, $3)")
            .bind(foreign_context_id)
            .bind(Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project uuid"))
            .bind("Foreign Context")
            .execute(&pool)
            .await
            .expect("insert foreign context");
        sqlx::query(
            "INSERT INTO context_commits (id, context_id, branch_name, message, changes) VALUES ($1, $2, 'main', 'Foreign commit', '[]'::jsonb)",
        )
        .bind(foreign_commit_id)
        .bind(foreign_context_id)
        .execute(&pool)
        .await
        .expect("insert foreign commit");

        let branch_error = sqlx::query(
            "INSERT INTO context_branches (context_id, branch_name, head_commit_id) VALUES ($1, 'foreign-head', $2)",
        )
        .bind(primary_context_id)
        .bind(foreign_commit_id)
        .execute(&pool)
        .await
        .expect_err("branch head must reference a same-Context commit");
        assert_eq!(
            branch_error
                .as_database_error()
                .and_then(|error| error.constraint()),
            Some("fk_context_branches_head_commit_same_context")
        );

        let idempotency_error = sqlx::query(
            "INSERT INTO context_commit_idempotency (identity_source, principal_id, context_id, branch_name, idempotency_key, request_digest, commit_id) VALUES ('https://issuer.contextlab.test', 'user:alex', $1, 'main', 'foreign-result', 'sha256:foreign-result', $2)",
        )
        .bind(primary_context_id)
        .bind(foreign_commit_id)
        .execute(&pool)
        .await
        .expect_err("idempotency result must reference a same-Context commit");
        assert_eq!(
            idempotency_error
                .as_database_error()
                .and_then(|error| error.constraint()),
            Some("fk_context_commit_idempotency_same_context")
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_authorizer_reports_membership_database_failures_as_unavailable() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        sqlx::query("DROP TABLE workspace_memberships")
            .execute(&pool)
            .await
            .expect("drop membership table for failure test");
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let error = RoleBasedContextAuthorizer::new(repository)
            .authorize(
                &test_principal(),
                seeded_context_id(),
                ContextPermission::Write,
            )
            .await
            .expect_err("membership lookup failure must be distinguishable");

        assert_eq!(error, AuthorizationError::Unavailable);
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_authorization_audit_sink_persists_safe_decisions() {
        let Some(pool) = disposable_test_pool(2).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = seeded_context_id();
        let principal_identity = PrincipalIdentity::new(
            IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:alex").expect("principal"),
        );

        for decision in [
            AuthorizationDecision::Granted,
            AuthorizationDecision::Forbidden,
            AuthorizationDecision::Unavailable,
        ] {
            repository
                .record(AuthorizationAuditEvent::new(
                    principal_identity.clone(),
                    context_id,
                    ContextPermission::Write,
                    decision,
                ))
                .await
                .expect("record authorization audit event");
        }

        for statement in [
            "UPDATE context_authorization_audit_events
             SET decision = 'granted'
             WHERE context_id = $1",
            "DELETE FROM context_authorization_audit_events WHERE context_id = $1",
        ] {
            sqlx::query(statement)
                .bind(context_id.as_uuid())
                .execute(&pool)
                .await
                .expect_err("authorization audit events must be append-only");
        }

        let events = sqlx::query_as::<_, (String, String, Uuid, String, String, DateTime<Utc>, bool)>(
            "SELECT identity_source, principal_id, context_id, permission, decision, recorded_at,
                    recorded_at <= now()
             FROM context_authorization_audit_events
             ORDER BY recorded_at ASC, id ASC",
        )
        .fetch_all(&pool)
        .await
        .expect("read authorization audit events");

        assert_eq!(events.len(), 3);
        assert!(events.iter().all(|event| event.6));
        let decisions = events
            .into_iter()
            .map(|(source, principal, context, permission, decision, _, _)| {
                (source, principal, context, permission, decision)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            decisions,
            BTreeSet::from([
                (
                    "https://issuer.contextlab.test".to_owned(),
                    "user:alex".to_owned(),
                    context_id.as_uuid(),
                    "write".to_owned(),
                    "granted".to_owned(),
                ),
                (
                    "https://issuer.contextlab.test".to_owned(),
                    "user:alex".to_owned(),
                    context_id.as_uuid(),
                    "write".to_owned(),
                    "forbidden".to_owned(),
                ),
                (
                    "https://issuer.contextlab.test".to_owned(),
                    "user:alex".to_owned(),
                    context_id.as_uuid(),
                    "write".to_owned(),
                    "unavailable".to_owned(),
                ),
            ])
        );
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn postgres_authorization_audit_review_is_redacted_and_cursor_paginated() {
        let Some(pool) = disposable_test_pool(1).await else {
            return;
        };
        let repository = PostgresContextGraphRepository::new(pool.clone());
        let workspace_id = contextlab_context_core::WorkspaceId::from_uuid(
            WORKSPACE_GRAPH_SEED_WORKSPACE_ID
                .parse()
                .expect("workspace uuid"),
        );
        let context_id = seeded_context_id();
        let oldest = Utc
            .with_ymd_and_hms(2026, 7, 11, 0, 0, 0)
            .single()
            .expect("oldest timestamp");
        let newest = Utc
            .with_ymd_and_hms(2026, 7, 13, 0, 0, 0)
            .single()
            .expect("newest timestamp");

        for (identity_source, principal_id, decision, recorded_at) in [
            (
                "https://review-one.contextlab.test",
                "user:should-not-appear-one",
                "granted",
                oldest,
            ),
            (
                "https://review-two.contextlab.test",
                "user:should-not-appear-two",
                "forbidden",
                newest,
            ),
        ] {
            sqlx::query(
                "INSERT INTO context_authorization_audit_events \
                 (identity_source, principal_id, context_id, permission, decision, recorded_at) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(identity_source)
            .bind(principal_id)
            .bind(context_id.as_uuid())
            .bind("write")
            .bind(decision)
            .bind(recorded_at)
            .execute(&pool)
            .await
            .expect("insert review fixture event");
        }

        let first_page = repository
            .list_authorization_audit_review(
                workspace_id,
                AuthorizationAuditReviewQuery::new(None, Some(1)),
            )
            .await
            .expect("read first redacted review page");
        assert_eq!(first_page.items.len(), 1);
        assert_eq!(first_page.items[0].recorded_at, newest);
        assert_eq!(first_page.items[0].context_id, context_id);
        assert_eq!(first_page.items[0].permission, ContextPermission::Write);
        assert_eq!(
            first_page.items[0].retention_disposition,
            AuthorizationAuditRetentionDisposition::Hold
        );
        assert_eq!(first_page.items[0].retention_policy_revision_id, None);
        assert!(first_page.next_cursor.is_some());
        let first_debug = format!("{:?}", first_page.items[0]);
        assert!(!first_debug.contains("review-two.contextlab.test"));
        assert!(!first_debug.contains("user:should-not-appear-two"));

        let second_page = repository
            .list_authorization_audit_review(
                workspace_id,
                AuthorizationAuditReviewQuery::new(first_page.next_cursor, Some(1)),
            )
            .await
            .expect("read second redacted review page");
        assert_eq!(second_page.items.len(), 1);
        assert_eq!(second_page.items[0].recorded_at, oldest);
        assert!(second_page.next_cursor.is_none());
    }

    #[tokio::test]
    #[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
    async fn guarded_writer_replays_and_advances_a_matching_branch_head() {
        let Some(database_url) = std::env::var("CONTEXTLAB_TEST_DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
        else {
            eprintln!("skipping: CONTEXTLAB_TEST_DATABASE_URL is not configured");
            return;
        };
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("connect to disposable test database");
        sqlx::raw_sql(CONTEXT_PLATFORM_MIGRATION)
            .execute(&pool)
            .await
            .expect("apply migration to empty test database");
        sqlx::raw_sql(WORKSPACE_GRAPH_SEED)
            .execute(&pool)
            .await
            .expect("apply seed fixture");
        sqlx::query(
            "INSERT INTO workspace_memberships (workspace_id, identity_source, principal_id, role) VALUES ($1, $2, $3, $4)",
        )
        .bind(
            WORKSPACE_GRAPH_SEED_WORKSPACE_ID
                .parse::<Uuid>()
                .expect("workspace uuid"),
        )
        .bind("https://issuer.contextlab.test")
        .bind("user:alex")
        .bind("owner")
        .execute(&pool)
        .await
        .expect("insert membership fixture");

        let repository = PostgresContextGraphRepository::new(pool.clone());
        let context_id = ContextId::from_uuid(
            "44444444-4444-4444-8444-444444444444"
                .parse()
                .expect("context uuid"),
        );
        let principal = AuthenticatedPrincipal::new(PrincipalIdentity::new(
            IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:alex").expect("principal"),
        ));
        RoleBasedContextAuthorizer::new(repository.clone())
            .authorize(&principal, context_id, ContextPermission::Write)
            .await
            .expect("owner membership authorizes guarded writes");
        let root_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create guarded root",
            Vec::new(),
            vec![ContextChange::created_context("Guarded root")],
            Utc::now(),
        )
        .expect("root commit");
        let root_commit_id = root_commit.id();
        let root_command = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            root_commit,
            ContextGraph::new(),
            Utc::now(),
            1,
        )
        .expect("root snapshot command");

        let created = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    principal.clone(),
                    ExpectedBranchHead::Unborn,
                    IdempotencyKey::new("request-root").expect("key"),
                    RequestDigest::new("sha256:root").expect("digest"),
                    root_command.clone(),
                )
                .expect("guarded command"),
            )
            .await
            .expect("create root");
        let replayed = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    principal.clone(),
                    ExpectedBranchHead::Unborn,
                    IdempotencyKey::new("request-root").expect("key"),
                    RequestDigest::new("sha256:root").expect("digest"),
                    root_command,
                )
                .expect("replay command"),
            )
            .await
            .expect("replay root");

        assert_eq!(created.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(replayed.snapshot, created.snapshot);

        let feature_commit = ContextCommit::new(
            context_id,
            BranchName::new("feature").expect("feature branch"),
            "Create guarded feature root",
            Vec::new(),
            vec![ContextChange::created_context("Guarded feature root")],
            Utc::now(),
        )
        .expect("feature root commit");
        let feature_command = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            feature_commit,
            ContextGraph::new(),
            Utc::now(),
            1,
        )
        .expect("feature root snapshot command");
        let feature = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    principal.clone(),
                    ExpectedBranchHead::Unborn,
                    IdempotencyKey::new("request-root").expect("key"),
                    RequestDigest::new("sha256:root").expect("digest"),
                    feature_command,
                )
                .expect("feature guarded command"),
            )
            .await
            .expect("create feature root with a branch-scoped idempotency key");

        assert_eq!(feature.disposition, GuardedCommitWriteDisposition::Created);
        assert_ne!(feature.snapshot.commit_id(), created.snapshot.commit_id());

        let child_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Advance guarded branch",
            vec![root_commit_id],
            vec![ContextChange::created_context("Guarded child")],
            Utc::now(),
        )
        .expect("child commit");
        let child_commit_id = child_commit.id();
        let child_command = CreateContextCommitSnapshot::new(
            seeded_project_id(),
            child_commit,
            ContextGraph::new(),
            Utc::now(),
            1,
        )
        .expect("child snapshot command");
        let child = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    principal.clone(),
                    ExpectedBranchHead::Commit(root_commit_id),
                    IdempotencyKey::new("request-child").expect("key"),
                    RequestDigest::new("sha256:child").expect("digest"),
                    child_command.clone(),
                )
                .expect("child command"),
            )
            .await
            .expect("advance branch");

        assert_eq!(child.disposition, GuardedCommitWriteDisposition::Created);
        let replayed_child = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    principal.clone(),
                    ExpectedBranchHead::Commit(root_commit_id),
                    IdempotencyKey::new("request-child").expect("key"),
                    RequestDigest::new("sha256:child").expect("digest"),
                    child_command,
                )
                .expect("replayed child command"),
            )
            .await
            .expect("replay parented child");
        assert_eq!(
            replayed_child.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(replayed_child.snapshot, child.snapshot);

        let child_commit_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM context_commits WHERE id = $1")
                .bind(child_commit_id.as_uuid())
                .fetch_one(&pool)
                .await
                .expect("count replayed child commits");
        let child_parent_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM context_commit_parents WHERE commit_id = $1",
        )
        .bind(child_commit_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count replayed child parents");
        let child_snapshot_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM context_commit_graph_snapshots WHERE commit_id = $1",
        )
        .bind(child_commit_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count replayed child snapshots");
        let child_idempotency_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM context_commit_idempotency WHERE context_id = $1 AND idempotency_key = 'request-child'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("count replayed child idempotency receipts");
        let branch_head = sqlx::query_scalar::<_, Uuid>(
            "SELECT head_commit_id FROM context_branches WHERE context_id = $1 AND branch_name = 'main'",
        )
        .bind(context_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("read branch head after parented replay");

        assert_eq!(
            [
                child_commit_count,
                child_parent_count,
                child_snapshot_count,
                child_idempotency_count,
            ],
            [1, 1, 1, 1]
        );
        assert_eq!(branch_head, child_commit_id.as_uuid());
        let stale = repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    principal,
                    ExpectedBranchHead::Unborn,
                    IdempotencyKey::new("request-stale").expect("key"),
                    RequestDigest::new("sha256:stale").expect("digest"),
                    CreateContextCommitSnapshot::new(
                        seeded_project_id(),
                        ContextCommit::new(
                            context_id,
                            BranchName::default(),
                            "Stale guarded branch",
                            Vec::new(),
                            vec![ContextChange::created_context("Stale branch")],
                            Utc::now(),
                        )
                        .expect("stale commit"),
                        ContextGraph::new(),
                        Utc::now(),
                        1,
                    )
                    .expect("stale snapshot command"),
                )
                .expect("stale command"),
            )
            .await
            .expect_err("stale head must not create a second tip");

        assert!(matches!(
            stale,
            StorageRepositoryError::BranchHeadConflict { .. }
        ));
    }
}
