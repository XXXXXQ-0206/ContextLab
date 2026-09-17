//! Storage contracts, migrations, and graph projections for ContextLab.
//!
//! This crate owns persistence-facing record shapes, migration assets,
//! repository boundaries, and deterministic projections into the
//! framework-independent `contextlab-graph` contract. API handlers compose
//! repository contracts rather than reaching into database tables directly.

mod authorization_audit_purge;
mod authorization_audit_review;
mod benchmark_definition_authoring;
mod benchmark_evidence;
mod benchmark_execution;
mod benchmark_workspace_projection;
mod branch_head;
mod commit;
mod commit_graph;
mod commit_graph_snapshot;
mod commit_snapshot_writer;
mod component;
mod component_content_revision;
mod component_descriptor_revision;
mod component_removal;
mod component_state_at_commit;
mod context;
mod context_commit_history;
mod context_diff_review;
mod context_diff_snapshot;
mod context_graph_diff_review;
mod context_graph_history_review;
mod context_lifecycle;
mod context_merge_review;
mod evaluation_run;
mod experiment;
mod guarded_commit_write;
mod knowledge_memory_projection;
mod listing;
mod memory;
mod postgres;
mod project;
mod projection;
mod protected_route_rate_limit;
mod records;
mod replay_graph_consistency;
mod replay_state_at_commit;
mod repository;
mod workflow_context_binding;
mod workflow_execution_status;
mod workspace;

pub use authorization_audit_purge::{
    AuthorizationAuditPurgeExecutor, AuthorizationAuditPurgeRequest, AuthorizationAuditPurgeResult,
};
pub use authorization_audit_review::{
    AuthorizationAuditRetentionDisposition, AuthorizationAuditReviewCursor,
    AuthorizationAuditReviewItem, AuthorizationAuditReviewPage, AuthorizationAuditReviewQuery,
    AuthorizationAuditReviewRepository,
};
pub use benchmark_definition_authoring::{
    BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION, BenchmarkDefinitionBinding,
    BenchmarkDefinitionBindingCommand, BenchmarkDefinitionBindingError,
    BenchmarkDefinitionBindingId, BenchmarkDefinitionBindingRepository,
    BenchmarkDefinitionBindingSummary, BenchmarkDefinitionBindingWriteDisposition,
    BenchmarkDefinitionBindingWriteResult, BenchmarkDefinitionBindingWriter,
};
pub use benchmark_evidence::{
    BenchmarkComparability, BenchmarkDecisionComparisonScope, BenchmarkDecisionComparisonService,
    BenchmarkDecisionComparisonServiceError, BenchmarkDecisionDatasetSummary,
    BenchmarkDecisionDefinitionSummary, BenchmarkDecisionDefinitionSummaryError,
    BenchmarkDecisionDefinitionSummaryService, BenchmarkDecisionDiscoveryRepository,
    BenchmarkDecisionDiscoveryService, BenchmarkDecisionDiscoveryServiceError,
    BenchmarkDecisionDiscoverySuiteSummary, BenchmarkDecisionDiscoverySummary,
    BenchmarkDecisionEvidence, BenchmarkDecisionId, BenchmarkDecisionPair,
    BenchmarkDecisionRunDetails, BenchmarkDecisionRunDetailsError,
    BenchmarkDecisionRunDetailsService, BenchmarkDecisionRunMeasurementSummary,
    BenchmarkDecisionRunSummary, BenchmarkDecisionSuiteSummary, BenchmarkEvidenceError,
    BenchmarkEvidenceRepository, BenchmarkEvidenceWriteDisposition, BenchmarkEvidenceWriteResult,
    BenchmarkEvidenceWriter, BenchmarkExecutionIdempotencyReceipt, BenchmarkMetricDecisionEvidence,
    PersistBenchmarkEvaluationEvidence,
};
pub use benchmark_execution::{
    BenchmarkCaseEvaluationRequest, BenchmarkCaseEvaluator, BenchmarkCaseEvaluatorError,
    BenchmarkDefinitionBindingExecutionSelection,
    BenchmarkDefinitionBindingExecutionSelectionError, BenchmarkExecutionDisposition,
    BenchmarkExecutionRequest, BenchmarkExecutionRequestError, BenchmarkExecutionResult,
    BenchmarkExecutionService, BenchmarkExecutionServiceError,
};
pub use benchmark_workspace_projection::{
    BenchmarkWorkspaceProjectionContractError, BenchmarkWorkspaceProjectionDecisionQuery,
    BenchmarkWorkspaceProjectionPersistenceError, BenchmarkWorkspaceProjectionReceiptScope,
    BenchmarkWorkspaceProjectionV1Query, BenchmarkWorkspaceProjectionV1Reader,
    BenchmarkWorkspaceProjectionV1Writer, BenchmarkWorkspaceProjectionWriteDisposition,
    BenchmarkWorkspaceProjectionWriteResult, InMemoryBenchmarkWorkspaceProjectionV1Repository,
    PersistBenchmarkWorkspaceProjectionV1,
};
pub use branch_head::{ContextBranchHead, ContextBranchRepository, ContextBranchRepositoryError};
pub use commit::{
    CommitDetail, CommitList, CommitListItem, CommitListPagination, CommitListQuery, CommitSort,
    CommitSortParseError, ContextCommitRepository,
};
pub use commit_graph::ContextCommitGraphRepository;
pub use commit_graph_snapshot::{
    COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1, CommitGraphSnapshot, CommitGraphSnapshotError,
    CommitGraphSnapshotReplay, CommitGraphSnapshotRepository, CommitGraphSnapshotScope,
};
pub use commit_snapshot_writer::{
    CommitSnapshotWriteError, ContextCommitSnapshotWriter, CreateContextCommitSnapshot,
};
pub use component::{
    ComponentDetail, ComponentList, ComponentListItem, ComponentListPagination, ComponentListQuery,
    ComponentSort, ComponentSortParseError, ContextComponentRepository,
};
pub use component_content_revision::{
    ComponentContentCreationError, ComponentContentCreationWrite, ComponentContentRevision,
    ComponentContentRevisionError, ComponentContentRevisionRepository,
    ComponentContentRevisionWrite,
};
pub use component_descriptor_revision::{
    ComponentDescriptorRevisionError, ComponentDescriptorRevisionWrite,
};
pub use component_removal::{ComponentRemovalError, ComponentRemovalWrite};
pub use component_state_at_commit::{
    ComponentStateAtCommit, ComponentStateAtCommitRepository,
    ContextComponentStateSnapshotAtCommit, ContextComponentStateSnapshotAtCommitRepository,
};
pub use context::{
    ContextList, ContextListItem, ContextListPagination, ContextListQuery, ContextRepository,
    ContextSort, ContextSortParseError,
};
pub use context_commit_history::{
    ContextCommitHistoryRepository, ContextCommitHistoryRepositoryAdapter,
};
pub use context_diff_review::{
    PersistedContextDiffReviewAdapter, PersistedContextDiffReviewError,
    PersistedContextDiffReviewService, PersistedContextDiffReviewSide,
};
pub use context_diff_snapshot::{
    CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1, ContextDiffSnapshotPersistenceError,
    ContextDiffSnapshotV1Pair, ContextDiffSnapshotV1PairRepository, ContextDiffSnapshotV1Record,
    ContextDiffSnapshotV1Repository, ContextDiffSnapshotV1ReviewRepository,
    ContextDiffSnapshotWriteDisposition, ContextDiffSnapshotWriteResult,
    InMemoryContextDiffSnapshotV1Repository, PersistContextDiffSnapshotV1,
};
pub use context_graph_diff_review::{
    PersistedContextGraphDiffReviewError, PersistedContextGraphDiffReviewProjection,
    PersistedContextGraphDiffReviewService, PersistedContextGraphDiffReviewSide,
};
pub use context_graph_history_review::{
    ContextGraphBranchHeadReviewWitness, ContextGraphBranchHeadReviewWitnessRepository,
    ContextGraphBranchHeadReviewWitnessRepositoryError, ContextGraphReviewWitness,
    ContextGraphReviewWitnessError, ContextGraphReviewWitnessRepository,
    PersistedContextGraphHistoryReviewError, PersistedContextGraphHistoryReviewProjection,
    PersistedContextGraphHistoryReviewService, PersistedContextGraphWitnessReviewService,
};
pub use context_lifecycle::{
    ContextLifecycleCommand, ContextLifecycleComponentState, ContextLifecycleError,
    ContextLifecycleOperation, ContextLifecycleReadFacts, ContextLifecycleReadRepository,
    ContextLifecycleRepository, ContextLifecycleRoot, ContextLifecycleRootRepository,
    ContextLifecycleService, ContextLifecycleStateAtCommit, ContextLifecycleWriteResult,
};
pub use context_merge_review::{
    ContextMergeInputIdentifier, ContextMergeInputScope, ContextMergeInputScopeError,
    ContextMergeReviewWitness, ContextMergeReviewWitnessError, ContextMergeReviewWitnessRepository,
    ContextMergeReviewWitnessRepositoryError, ContextMergeTipScope,
    PersistedContextGraphMergeReviewError, PersistedContextGraphMergeReviewService,
    PersistedContextGraphMergeReviewSide,
};
pub use evaluation_run::{
    EvaluationRunDetail, EvaluationRunList, EvaluationRunListItem, EvaluationRunListPagination,
    EvaluationRunListQuery, EvaluationRunRepository, EvaluationRunSort,
    EvaluationRunSortParseError, EvaluationScorecard, EvaluationScorecardMetric,
    EvaluationScorecardQuery,
};
pub use experiment::{
    ExperimentList, ExperimentListItem, ExperimentListPagination, ExperimentListQuery,
    ExperimentRepository, ExperimentSort, ExperimentSortParseError,
};
pub(crate) use guarded_commit_write::{
    ComponentContentMutationWrite, validate_component_content_attachment,
};
pub use guarded_commit_write::{
    GuardedCommitWriteDisposition, GuardedCommitWriteError, GuardedCommitWriteResult,
    GuardedContextCommitWrite, GuardedContextCommitWriter, IdempotencyKey, RequestDigest,
};
pub use knowledge_memory_projection::{
    InMemoryKnowledgeMemoryProjectionV1Repository, KnowledgeMemoryProjectionPersistenceError,
    KnowledgeMemoryProjectionScope, KnowledgeMemoryProjectionV1Repository,
    KnowledgeMemoryProjectionWriteDisposition, KnowledgeMemoryProjectionWriteResult,
    PersistKnowledgeMemoryProjectionV1,
};
pub use listing::{DEFAULT_PAGE, DEFAULT_PER_PAGE, ListPagination, MAX_PER_PAGE};
pub use memory::InMemoryContextGraphRepository;
pub use postgres::{PostgresAuthorizationAuditPurgeExecutor, PostgresContextGraphRepository};
pub use project::{
    ProjectList, ProjectListItem, ProjectListPagination, ProjectListQuery, ProjectRepository,
    ProjectSort, ProjectSortParseError,
};
pub use projection::{ContextGraphProjection, StorageProjectionError};
pub use protected_route_rate_limit::PostgresProtectedRouteRateLimiter;
pub use records::{
    ContextCommitRecord, ContextComponentRecord, ContextRecord, EvaluationRunRecord,
    ExperimentRecord, ProjectRecord, StoredComponentKind, StoredComponentKindParseError,
    WorkspaceRecord,
};
pub use replay_state_at_commit::ContextReplayStateAtCommitRepository;
pub use repository::{
    ContextGraphProjectionRepository, GraphProjectionScope, StorageRepositoryError,
};
pub use workflow_context_binding::{
    ContextWorkflowBindingRepository, WorkflowContextBindingWriteDisposition,
    WorkflowContextBindingWriteResult,
};
pub use workflow_execution_status::{
    InMemoryWorkflowExecutionStatusRepository, PersistWorkflowExecutionStatusV1,
    WorkflowExecutionStatusPersistenceError, WorkflowExecutionStatusRepository,
    WorkflowExecutionStatusService, WorkflowExecutionStatusServiceError,
    WorkflowExecutionStatusWriteDisposition, WorkflowExecutionStatusWriteResult,
};
pub use workspace::{
    WorkspaceList, WorkspaceListItem, WorkspaceListPagination, WorkspaceListQuery,
    WorkspaceRepository, WorkspaceSort, WorkspaceSortParseError,
};

/// The first PostgreSQL migration for the Context platform schema.
pub const CONTEXT_PLATFORM_MIGRATION: &str = concat!(
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
    "\n",
    include_str!("../migrations/0013_component_content_initial_revisions.sql"),
    "\n",
    include_str!("../migrations/0014_component_content_initial_revision_integrity.sql"),
    "\n",
    include_str!("../migrations/0015_context_commit_parent_scope_integrity.sql"),
    "\n",
    include_str!("../migrations/0016_benchmark_definition_decision_evidence.sql"),
    "\n",
    include_str!("../migrations/0017_branch_scoped_commit_idempotency.sql"),
    "\n",
    include_str!("../migrations/0018_context_workflow_bindings.sql"),
    "\n",
    include_str!("../migrations/0019_benchmark_workspace_projection_receipts.sql"),
    "\n",
    include_str!("../migrations/0020_benchmark_definition_bindings.sql"),
    "\n",
    include_str!("../migrations/0021_benchmark_execution_idempotency.sql"),
    "\n",
    include_str!("../migrations/0022_knowledge_memory_context_projections.sql"),
    "\n",
    include_str!("../migrations/0023_context_diff_snapshots.sql"),
    "\n",
    include_str!("../migrations/0024_workflow_execution_status.sql"),
    "\n",
    include_str!("../migrations/0025_context_commit_graph_snapshot_integrity.sql"),
);

/// Private immutable workflow execution status projection migration.
pub const WORKFLOW_EXECUTION_STATUS_MIGRATION: &str =
    include_str!("../migrations/0024_workflow_execution_status.sql");

/// Immutable benchmark definition and decision evidence migration.
pub const BENCHMARK_EVIDENCE_MIGRATION: &str =
    include_str!("../migrations/0016_benchmark_definition_decision_evidence.sql");

/// Private exact-Context benchmark definition binding migration.
pub const BENCHMARK_DEFINITION_BINDING_MIGRATION: &str =
    include_str!("../migrations/0020_benchmark_definition_bindings.sql");

/// Private benchmark execution idempotency receipt migration.
pub const BENCHMARK_EXECUTION_IDEMPOTENCY_MIGRATION: &str =
    include_str!("../migrations/0021_benchmark_execution_idempotency.sql");

/// Private exact-commit diff snapshot migration.
pub const CONTEXT_DIFF_SNAPSHOT_MIGRATION: &str =
    include_str!("../migrations/0023_context_diff_snapshots.sql");

/// Private immutable Context Graph snapshot integrity migration.
pub const CONTEXT_COMMIT_GRAPH_SNAPSHOT_INTEGRITY_MIGRATION: &str =
    include_str!("../migrations/0025_context_commit_graph_snapshot_integrity.sql");

/// Operator-provisioned PostgreSQL roles for the private audit purge boundary.
pub const CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP: &str =
    include_str!("../privileged/contextlab_audit_purge_roles.sql");

/// Private PostgreSQL procedure and manifest contract for audit retention purges.
pub const CONTEXTLAB_AUDIT_PURGE_EXECUTOR: &str =
    include_str!("../privileged/contextlab_audit_purge_executor.sql");

/// Deterministic PostgreSQL seed data for workspace graph integration tests.
pub const WORKSPACE_GRAPH_SEED: &str = include_str!("../fixtures/workspace_graph_seed.sql");

/// Workspace id used by [`WORKSPACE_GRAPH_SEED`].
pub const WORKSPACE_GRAPH_SEED_WORKSPACE_ID: &str = "11111111-1111-4111-8111-111111111111";

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_graph::{GraphEdgeKind, GraphNodeKind};

    #[test]
    fn migration_declares_core_tables_and_indexes() {
        for expected in [
            "CREATE TABLE workspaces",
            "CREATE TABLE projects",
            "CREATE TABLE experiments",
            "CREATE TABLE contexts",
            "CREATE TABLE context_components",
            "CREATE TABLE context_commits",
            "CREATE TABLE context_commit_parents",
            "CREATE TABLE context_commit_graph_snapshots",
            "CREATE TABLE workspace_memberships",
            "CREATE TABLE context_branches",
            "CREATE TABLE context_commit_idempotency",
            "CREATE TABLE context_authorization_audit_events",
            "CREATE FUNCTION prevent_context_authorization_audit_event_mutation()",
            "CREATE TRIGGER context_authorization_audit_events_append_only",
            "CREATE TABLE evaluation_runs",
            "CREATE INDEX idx_contexts_project_id",
            "CREATE UNIQUE INDEX idx_workspaces_slug_active",
            "CREATE UNIQUE INDEX idx_projects_workspace_slug_active",
            "CREATE UNIQUE INDEX idx_experiments_project_branch_active",
            "CREATE INDEX idx_context_commit_parents_parent_id",
            "CREATE INDEX idx_context_commits_context_id_id",
            "CHECK (role IN ('owner', 'editor', 'reader'))",
            "PRIMARY KEY (context_id, branch_name)",
            "PRIMARY KEY (principal_id, context_id, idempotency_key)",
            "CHECK (permission IN ('read', 'write'))",
            "CHECK (decision IN ('granted', 'forbidden', 'unavailable'))",
            "BEFORE UPDATE OR DELETE",
            "REFERENCES contexts(id) ON DELETE RESTRICT",
            "UNIQUE (context_id, id)",
            "CREATE TABLE context_component_content_revisions",
            "uq_context_components_context_id_id",
            "idx_context_component_content_revisions_context_component",
            "fk_context_branches_head_commit_same_context",
            "fk_context_commit_idempotency_same_context",
            "ALTER TABLE context_commit_parents\n    ADD COLUMN context_id UUID",
            "UPDATE context_commit_parents AS parent_link",
            "fk_context_commit_parents_child_same_context",
            "fk_context_commit_parents_parent_same_context",
            "CREATE TABLE benchmark_dataset_definitions",
            "CREATE TABLE benchmark_decision_evidence",
            "CREATE TABLE benchmark_execution_idempotency",
            "CREATE TRIGGER benchmark_execution_idempotency_append_only",
            "CREATE TABLE knowledge_memory_context_projections",
            "knowledge-memory-context-projection-v1",
            "uq_contexts_project_id_id",
            "REFERENCES contexts(project_id, id) ON DELETE RESTRICT",
            "REFERENCES context_commits(context_id, id) ON DELETE RESTRICT",
            "knowledge_memory_context_projection_append_only",
            "CREATE TABLE context_workflow_bindings",
            "fk_context_workflow_bindings_context_commit",
            "fk_context_workflow_bindings_materialized_snapshot",
            "workflow_revision BIGINT NOT NULL CHECK (workflow_revision > 0)",
            "uq_context_workflow_bindings_workflow_revision",
            "CHECK (jsonb_typeof(graph) = 'object')",
            "CREATE INDEX idx_context_authorization_audit_events_context_recorded_at",
            "CREATE INDEX idx_context_authorization_audit_events_principal_recorded_at",
            "'evaluation'",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "migration should contain {expected}"
            );
        }
    }

    #[test]
    fn benchmark_execution_idempotency_is_a_forward_migration_after_evidence() {
        assert!(
            BENCHMARK_EXECUTION_IDEMPOTENCY_MIGRATION
                .contains("REFERENCES benchmark_decision_evidence(")
        );
        assert!(
            BENCHMARK_EXECUTION_IDEMPOTENCY_MIGRATION
                .contains("CREATE TRIGGER benchmark_execution_idempotency_append_only")
        );
        assert!(!BENCHMARK_EVIDENCE_MIGRATION.contains("benchmark_execution_idempotency"));
        assert!(CONTEXT_PLATFORM_MIGRATION.contains(BENCHMARK_EXECUTION_IDEMPOTENCY_MIGRATION));
    }

    #[test]
    fn migration_scopes_guarded_idempotency_receipts_to_a_backfilled_branch() {
        for expected in [
            "ADD COLUMN branch_name",
            "SET branch_name = commit.branch_name",
            "chk_context_commit_idempotency_branch_name",
            "branch_name,\n        idempotency_key",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "branch-scoped idempotency migration should contain {expected}"
            );
        }
    }

    #[test]
    fn parent_scope_migration_preflights_history_before_schema_changes() {
        let migration =
            include_str!("../migrations/0015_context_commit_parent_scope_integrity.sql");
        let preflight_position = migration
            .find("IF EXISTS (")
            .expect("parent-scope migration must preflight historical links");
        let schema_change_position = migration
            .find("ALTER TABLE context_commit_parents")
            .expect("parent-scope migration must alter the parent table");

        assert!(
            preflight_position < schema_change_position,
            "historical parent scope must be checked before any schema change"
        );
        for expected in [
            "child_commit.context_id <> parent_commit.context_id",
            "ERRCODE = '23503'",
            "CONSTRAINT = 'fk_context_commit_parents_parent_same_context'",
        ] {
            assert!(
                migration.contains(expected),
                "parent-scope preflight must contain {expected}"
            );
        }
    }

    #[test]
    fn migration_component_kind_check_matches_stored_taxonomy() {
        for kind in StoredComponentKind::ALL {
            let expected = format!("'{}'", kind.as_str());
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(&expected),
                "migration component kind check should contain {expected}"
            );
        }
    }

    #[test]
    fn migration_allows_null_prior_hashes_only_for_initial_component_revisions() {
        for expected in [
            "ALTER TABLE context_component_content_revisions\n    ALTER COLUMN previous_content_hash DROP NOT NULL",
            "CREATE FUNCTION validate_context_component_initial_content_revision()",
            "CREATE TRIGGER context_component_content_revisions_initial_revision_integrity",
            "uq_context_component_content_revisions_initial_prior",
            "null previous_content_hash is valid only for a component initial revision",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "initial component revision migration should contain {expected}"
            );
        }
    }

    #[test]
    fn migration_namespaces_protected_identity_keys_by_source_and_subject() {
        for expected in [
            "ALTER TABLE workspace_memberships ADD COLUMN identity_source",
            "PRIMARY KEY (workspace_id, identity_source, principal_id)",
            "ALTER TABLE context_commit_idempotency ADD COLUMN identity_source",
            "PRIMARY KEY (identity_source, principal_id, context_id, idempotency_key)",
            "ALTER TABLE context_authorization_audit_events ADD COLUMN identity_source",
            "UPDATE workspace_memberships SET identity_source = 'legacy'",
            "UPDATE context_commit_idempotency SET identity_source = 'legacy'",
            "UPDATE context_authorization_audit_events SET identity_source = 'legacy'",
            "idx_workspace_memberships_identity_source_principal_id",
            "idx_context_authorization_audit_events_identity_principal_recorded_at",
            "chk_workspace_memberships_principal_id_identity",
            "chk_context_commit_idempotency_principal_id_identity",
            "chk_context_authorization_audit_events_principal_id_identity",
            "ADD COLUMN identity_source TEXT COLLATE \"C\"",
            "ALTER COLUMN principal_id TYPE TEXT COLLATE \"C\" USING principal_id",
            "octet_length(identity_source) <= 2048",
            "octet_length(principal_id) <= 512",
            "identity_source !~ '^[[:space:]]|[[:space:]]$'",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "identity migration should contain {expected}"
            );
        }
        assert_eq!(
            CONTEXT_PLATFORM_MIGRATION.matches("NOT VALID").count(),
            3,
            "existing principal subjects must not make an upgrade fail while new writes remain checked"
        );
    }

    #[test]
    fn migration_scopes_external_group_bindings_to_workspaces_and_identity_sources() {
        for expected in [
            "CREATE TABLE workspace_external_group_role_bindings",
            "identity_source TEXT COLLATE \"C\"",
            "external_group_id TEXT COLLATE \"C\"",
            "role IN ('reader', 'editor')",
            "octet_length(external_group_id) <= 512",
            "deleted_at TIMESTAMPTZ",
            "idx_workspace_external_group_role_bindings_active",
            "WHERE deleted_at IS NULL",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "external group binding migration should contain {expected}"
            );
        }
    }

    #[test]
    fn migration_defaults_authorization_audit_retention_to_hold_with_immutable_revisions() {
        for expected in [
            "CREATE TABLE context_authorization_audit_retention_policies",
            "CREATE TABLE context_authorization_audit_retention_policy_scopes",
            "CREATE TABLE context_authorization_audit_purge_manifests",
            "retention_disposition TEXT NOT NULL DEFAULT 'hold'",
            "CHECK (retention_disposition IN ('hold', 'purge_eligible'))",
            "retention_policy_revision_id UUID",
            "purge_eligible_at TIMESTAMPTZ",
            "retention_duration INTERVAL NOT NULL",
            "CHECK (retention_duration >= interval '0 seconds')",
            "FOREIGN KEY (workspace_id, active_policy_revision_id)",
            "idx_context_authorization_audit_events_purge_eligible_context_expiry",
            "NEW.purge_eligible_at := NEW.recorded_at + policy_retention_duration",
            "selected_event_count BIGINT NOT NULL",
            "CREATE FUNCTION prevent_context_authorization_audit_retention_policy_mutation()",
            "CREATE FUNCTION prevent_context_authorization_audit_purge_manifest_mutation()",
            "CREATE TRIGGER context_authorization_audit_purge_manifests_append_only",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "audit retention migration should contain {expected}"
            );
        }

        assert!(
            !CONTEXT_PLATFORM_MIGRATION.contains("SECURITY DEFINER"),
            "the migration must not claim a privileged purge boundary before runtime roles are verified"
        );
    }

    #[test]
    fn privileged_audit_purge_artifacts_keep_the_base_migration_unprivileged() {
        assert!(
            !CONTEXT_PLATFORM_MIGRATION.contains("SECURITY DEFINER"),
            "the base migration must not include privileged artifacts"
        );

        for expected in [
            "contextlab_audit_purge_owner",
            "contextlab_audit_purge_executor",
            "NOLOGIN",
            "NOINHERIT",
            "NOSUPERUSER",
            "NOCREATEDB",
            "NOCREATEROLE",
            "NOREPLICATION",
            "GRANT USAGE ON SCHEMA public",
            "GRANT SELECT, UPDATE, DELETE ON TABLE public.context_authorization_audit_events",
            "public.context_authorization_audit_retention_policies",
            "public.contexts",
            "public.projects",
            "GRANT SELECT, INSERT ON TABLE public.context_authorization_audit_purge_manifests",
            "GRANT USAGE ON SCHEMA public TO contextlab_audit_purge_executor",
        ] {
            assert!(
                CONTEXTLAB_AUDIT_PURGE_ROLE_BOOTSTRAP.contains(expected),
                "role bootstrap should contain {expected}"
            );
        }

        for expected in [
            "CREATE TABLE public.context_authorization_audit_purge_manifest_items",
            "REFERENCES public.context_authorization_audit_purge_manifests(id)",
            "REVOKE ALL PRIVILEGES ON TABLE public.context_authorization_audit_purge_manifest_items",
            "GRANT SELECT, INSERT ON TABLE public.context_authorization_audit_purge_manifest_items",
            "PRIMARY KEY (manifest_id, event_id)",
            "UNIQUE (event_id)",
            "CREATE OR REPLACE FUNCTION public.prevent_context_authorization_audit_purge_manifest_item_mutation()",
            "CREATE OR REPLACE FUNCTION public.prevent_context_authorization_audit_event_mutation()",
            "DROP TRIGGER IF EXISTS context_authorization_audit_purge_manifest_items_append_only\n    ON public.context_authorization_audit_purge_manifest_items",
            "CREATE TRIGGER context_authorization_audit_purge_manifest_items_append_only\n    BEFORE UPDATE OR DELETE ON public.context_authorization_audit_purge_manifest_items",
            "EXECUTE FUNCTION public.prevent_context_authorization_audit_purge_manifest_item_mutation()",
            "DROP TRIGGER IF EXISTS context_authorization_audit_events_append_only\n    ON public.context_authorization_audit_events",
            "CREATE TRIGGER context_authorization_audit_events_append_only\n    BEFORE UPDATE OR DELETE ON public.context_authorization_audit_events",
            "EXECUTE FUNCTION public.prevent_context_authorization_audit_event_mutation()",
            "current_user = 'contextlab_audit_purge_owner'",
            "current_setting('contextlab.audit_purge_manifest_id', true)",
            "context_authorization_audit_purge_manifest_items",
            "CREATE OR REPLACE FUNCTION public.purge_context_authorization_audit_events(",
            "p_workspace_id UUID",
            "p_policy_revision_id UUID",
            "p_cutoff TIMESTAMPTZ",
            "p_limit INTEGER DEFAULT 100",
            "SECURITY DEFINER",
            "SET search_path = pg_catalog, public",
            "p_limit <= 0 OR p_limit > 1000",
            "purge_eligible_at < p_cutoff",
            "FOR UPDATE SKIP LOCKED",
            "set_config(",
            "'contextlab.audit_purge_manifest_id'",
            "REVOKE ALL ON FUNCTION public.purge_context_authorization_audit_events",
            "GRANT EXECUTE ON FUNCTION public.purge_context_authorization_audit_events",
            "contextlab_audit_purge_executor",
            "GRANT CREATE ON SCHEMA public TO contextlab_audit_purge_owner",
            "ALTER FUNCTION public.purge_context_authorization_audit_events",
            "OWNER TO contextlab_audit_purge_owner",
            "REVOKE CREATE ON SCHEMA public FROM contextlab_audit_purge_owner",
        ] {
            assert!(
                CONTEXTLAB_AUDIT_PURGE_EXECUTOR.contains(expected),
                "privileged purge executor should contain {expected}"
            );
        }

        let temporary_create_grant = CONTEXTLAB_AUDIT_PURGE_EXECUTOR
            .find("GRANT CREATE ON SCHEMA public TO contextlab_audit_purge_owner")
            .expect("function ownership transfer should grant schema create temporarily");
        let function_ownership_transfer = CONTEXTLAB_AUDIT_PURGE_EXECUTOR
            .find("ALTER FUNCTION public.purge_context_authorization_audit_events")
            .expect("purge function ownership transfer should be schema-qualified");
        let temporary_create_revoke = CONTEXTLAB_AUDIT_PURGE_EXECUTOR
            .find("REVOKE CREATE ON SCHEMA public FROM contextlab_audit_purge_owner")
            .expect("function ownership transfer should revoke schema create immediately");

        assert!(
            temporary_create_grant < function_ownership_transfer
                && function_ownership_transfer < temporary_create_revoke,
            "the purge owner may hold schema CREATE only during function ownership transfer"
        );
    }

    #[test]
    fn migration_ledger_is_append_only_and_excludes_privileged_role_contracts() {
        for expected in [
            "CREATE TABLE contextlab_schema_migration_ledger",
            "migration_id TEXT PRIMARY KEY",
            "migration_sha256 TEXT NOT NULL",
            "applied_at TIMESTAMPTZ NOT NULL DEFAULT now()",
            "CREATE TRIGGER contextlab_schema_migration_ledger_append_only",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "platform migration should contain {expected}"
            );
        }
        assert!(
            !CONTEXT_PLATFORM_MIGRATION.contains("contextlab_audit_purge_owner"),
            "privileged role provisioning must remain outside the regular ledger"
        );
    }

    #[test]
    fn migration_declares_private_shared_protected_route_rate_limit_state() {
        for expected in [
            "CREATE TABLE public.protected_route_rate_limit_configurations",
            "CREATE TABLE public.protected_route_rate_limit_states",
            "PRIMARY KEY (identity_source, principal_id, operation)",
            "RENAME COLUMN max_tracked_principals TO max_tracked_keys",
            "DROP COLUMN last_observed_at",
            "CREATE FUNCTION public.prevent_protected_route_rate_limit_configuration_mutation()",
            "max_requests INTEGER NOT NULL CHECK (max_requests BETWEEN 1 AND 1000)",
            "window_seconds INTEGER NOT NULL CHECK (window_seconds BETWEEN 1 AND 3600)",
            "max_tracked_principals INTEGER NOT NULL CHECK (max_tracked_principals BETWEEN 1 AND 100000)",
            "request_timestamps TIMESTAMPTZ[] NOT NULL",
            "CHECK (cardinality(request_timestamps) BETWEEN 1 AND 1000)",
            "idx_protected_route_rate_limit_states_expires_at",
            "CREATE TRIGGER protected_route_rate_limit_configurations_append_only",
        ] {
            assert!(
                CONTEXT_PLATFORM_MIGRATION.contains(expected),
                "shared protected-route rate-limit migration should contain {expected}"
            );
        }
        assert!(
            !CONTEXT_PLATFORM_MIGRATION.contains("SECURITY DEFINER"),
            "shared protected-route limiter must not introduce a privileged database boundary"
        );
    }

    #[test]
    fn workspace_graph_seed_declares_graph_and_soft_deleted_rows() {
        for expected in [
            WORKSPACE_GRAPH_SEED_WORKSPACE_ID,
            "Seed Support Resolution Context",
            "Seed Safety Regression",
            "Deleted Seed Context",
            "deleted_at",
        ] {
            assert!(
                WORKSPACE_GRAPH_SEED.contains(expected),
                "workspace graph seed should contain {expected}"
            );
        }
    }

    #[test]
    fn projects_preview_records_into_context_graph() {
        let graph = ContextGraphProjection::context_engineering_preview()
            .project()
            .expect("valid projection");

        assert_eq!(graph.nodes().len(), 10);
        assert_eq!(graph.edges().len(), 10);
        assert!(
            graph
                .nodes()
                .values()
                .any(|node| node.kind() == GraphNodeKind::Context)
        );
        assert!(
            graph
                .edges()
                .iter()
                .any(|edge| edge.kind() == GraphEdgeKind::Evaluates)
        );
    }

    #[test]
    fn rejects_projection_with_missing_parent() {
        let projection = ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: "orphan".to_owned(),
                workspace_id: "missing".to_owned(),
                name: "Orphan Project".to_owned(),
                slug: "orphan".to_owned(),
                created_at: chrono::DateTime::parse_from_rfc3339("2026-07-09T00:00:00Z")
                    .expect("valid timestamp")
                    .with_timezone(&chrono::Utc),
            }],
            ..ContextGraphProjection::default()
        };

        let error = projection
            .project()
            .expect_err("missing workspace must fail");

        assert!(matches!(error, StorageProjectionError::Graph(_)));
    }
}
