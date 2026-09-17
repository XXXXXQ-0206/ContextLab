//! In-memory storage repository implementation.

use crate::{
    BenchmarkDecisionComparisonScope, BenchmarkDecisionDatasetSummary,
    BenchmarkDecisionDiscoveryRepository, BenchmarkDecisionDiscoverySuiteSummary,
    BenchmarkDecisionDiscoverySummary, BenchmarkDecisionEvidence, BenchmarkDecisionPair,
    BenchmarkDefinitionBinding, BenchmarkDefinitionBindingCommand,
    BenchmarkDefinitionBindingRepository, BenchmarkDefinitionBindingWriteDisposition,
    BenchmarkDefinitionBindingWriteResult, BenchmarkDefinitionBindingWriter,
    BenchmarkEvidenceRepository, BenchmarkEvidenceWriteDisposition, BenchmarkEvidenceWriteResult,
    BenchmarkEvidenceWriter, BenchmarkWorkspaceProjectionDecisionQuery,
    BenchmarkWorkspaceProjectionPersistenceError, BenchmarkWorkspaceProjectionV1Query,
    BenchmarkWorkspaceProjectionV1Reader, BenchmarkWorkspaceProjectionV1Writer,
    BenchmarkWorkspaceProjectionWriteResult, CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1, CommitDetail,
    CommitGraphSnapshot, CommitGraphSnapshotRepository, CommitList, CommitListItem,
    CommitListQuery, CommitSort, ComponentContentMutationWrite, ComponentContentRevision,
    ComponentContentRevisionRepository, ComponentDescriptorRevisionWrite, ComponentDetail,
    ComponentList, ComponentListItem, ComponentListQuery, ComponentSort,
    ComponentStateAtCommitRepository, ContextBranchHead, ContextBranchRepository,
    ContextBranchRepositoryError, ContextCommitGraphRepository, ContextCommitHistoryRepository,
    ContextCommitRepository, ContextCommitSnapshotWriter, ContextComponentRepository,
    ContextComponentStateSnapshotAtCommitRepository, ContextDiffSnapshotPersistenceError,
    ContextDiffSnapshotV1Pair, ContextDiffSnapshotV1PairRepository, ContextDiffSnapshotV1Record,
    ContextDiffSnapshotV1Repository, ContextDiffSnapshotWriteDisposition,
    ContextDiffSnapshotWriteResult, ContextGraphBranchHeadReviewWitness,
    ContextGraphBranchHeadReviewWitnessRepository,
    ContextGraphBranchHeadReviewWitnessRepositoryError, ContextGraphProjection,
    ContextGraphProjectionRepository, ContextGraphReviewWitness,
    ContextGraphReviewWitnessRepository, ContextLifecycleReadFacts, ContextLifecycleReadRepository,
    ContextLifecycleRoot, ContextLifecycleRootRepository, ContextList, ContextListItem,
    ContextListQuery, ContextMergeInputScope, ContextMergeReviewWitness,
    ContextMergeReviewWitnessError, ContextMergeReviewWitnessRepository,
    ContextMergeReviewWitnessRepositoryError, ContextReplayStateAtCommitRepository,
    ContextRepository, ContextSort, ContextWorkflowBindingRepository, CreateContextCommitSnapshot,
    EvaluationRunDetail, EvaluationRunList, EvaluationRunListItem, EvaluationRunListQuery,
    EvaluationRunRepository, EvaluationRunSort, EvaluationScorecard, EvaluationScorecardMetric,
    EvaluationScorecardQuery, ExperimentList, ExperimentListItem, ExperimentListQuery,
    ExperimentRepository, ExperimentSort, GraphProjectionScope, GuardedCommitWriteDisposition,
    GuardedCommitWriteResult, GuardedContextCommitWrite, GuardedContextCommitWriter,
    IdempotencyKey, InMemoryBenchmarkWorkspaceProjectionV1Repository,
    InMemoryKnowledgeMemoryProjectionV1Repository, KnowledgeMemoryProjectionPersistenceError,
    KnowledgeMemoryProjectionScope, KnowledgeMemoryProjectionV1Repository,
    KnowledgeMemoryProjectionWriteResult, PersistBenchmarkEvaluationEvidence,
    PersistBenchmarkWorkspaceProjectionV1, PersistContextDiffSnapshotV1,
    PersistKnowledgeMemoryProjectionV1, ProjectList, ProjectListItem, ProjectListQuery,
    ProjectRepository, ProjectSort, StorageRepositoryError, WorkflowContextBindingWriteResult,
    WorkspaceList, WorkspaceListItem, WorkspaceListQuery, WorkspaceRepository, WorkspaceSort,
    validate_component_content_attachment,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::VersionedContextScopeV1;
use contextlab_evaluation::{
    BenchmarkDataset, BenchmarkDatasetId, BenchmarkSuite, BenchmarkSuiteId, EvaluationRun,
    EvaluationRunId,
};
use contextlab_versioning::{BranchHead, CommitId, MergePlan, normal_commit_parent};
use contextlab_workflow::{WorkflowContextBinding, WorkflowId, WorkflowRevision};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::benchmark_evidence::BenchmarkDecisionDiscoverySummaryInput;
use crate::commit_graph::commit_graph_from_records;
use crate::commit_graph_snapshot::CommitGraphSnapshotScope;
use crate::component_state_at_commit::{
    ComponentStateReplayStep, component_state_conflict, replay_component_state,
    replay_context_component_state_snapshot,
};
use crate::context_commit_history::assemble_context_commit_history_from_commits;
use crate::replay_state_at_commit::{context_commit_from_record, replay_state_from_records};

/// In-memory repository for tests and early preview routes.
#[derive(Debug, Clone)]
pub struct InMemoryContextGraphRepository {
    preview: ContextGraphProjection,
    commit_snapshot_state: Arc<RwLock<CommitSnapshotState>>,
    benchmark_evidence_state: Arc<RwLock<BenchmarkEvidenceState>>,
    benchmark_workspace_projection_repository: InMemoryBenchmarkWorkspaceProjectionV1Repository,
    knowledge_memory_projection_repository: InMemoryKnowledgeMemoryProjectionV1Repository,
}

#[derive(Debug, Default)]
struct CommitSnapshotState {
    commits: BTreeMap<(String, String), crate::ContextCommitRecord>,
    snapshots: BTreeMap<(String, String, String), CommitGraphSnapshot>,
    diff_snapshots: BTreeMap<(Uuid, Uuid, Uuid, String), ContextDiffSnapshotV1Record>,
    branch_heads: BTreeMap<(String, String), Option<CommitId>>,
    branch_revisions: BTreeMap<(String, String), u64>,
    idempotency: BTreeMap<(String, String, String, String, String), IdempotencyRecord>,
    component_content_hashes: BTreeMap<(String, String), String>,
    component_content_updated_at: BTreeMap<(String, String), DateTime<Utc>>,
    components: BTreeMap<(String, String), crate::ContextComponentRecord>,
    removed_components: BTreeSet<(String, String)>,
    component_content_revisions: BTreeMap<(String, String, String), ComponentContentRevision>,
    workflow_context_bindings: BTreeMap<(WorkflowId, WorkflowRevision), WorkflowContextBinding>,
    benchmark_definition_bindings: BTreeMap<
        (Uuid, Uuid, Uuid, crate::BenchmarkDefinitionBindingId),
        BenchmarkDefinitionBinding,
    >,
    benchmark_definition_idempotency:
        BTreeMap<(String, String, String, String, String), BenchmarkDefinitionIdempotencyRecord>,
}

#[derive(Debug, Clone)]
struct IdempotencyRecord {
    request_digest: String,
    commit_id: String,
}

#[derive(Debug, Clone)]
struct BenchmarkDefinitionIdempotencyRecord {
    request_digest: String,
    binding_id: crate::BenchmarkDefinitionBindingId,
}

#[derive(Debug, Default)]
struct BenchmarkEvidenceState {
    datasets: BTreeMap<(Uuid, BenchmarkDatasetId), BenchmarkDataset>,
    suites: BTreeMap<(Uuid, BenchmarkSuiteId), BenchmarkSuite>,
    runs: BTreeMap<(Uuid, Uuid, Uuid, Uuid), EvaluationRun>,
    decisions: BTreeMap<(Uuid, Uuid, Uuid, crate::BenchmarkDecisionId), StoredBenchmarkEvidence>,
    execution_idempotency:
        BTreeMap<(Uuid, Uuid, Uuid, String), BenchmarkExecutionIdempotencyRecord>,
}

#[derive(Debug, Clone)]
struct StoredBenchmarkEvidence {
    evidence_digest: String,
    evidence: BenchmarkDecisionEvidence,
    sealed: bool,
}

#[derive(Debug, Clone)]
struct BenchmarkExecutionIdempotencyRecord {
    request_digest: String,
    decision_id: crate::BenchmarkDecisionId,
}

impl InMemoryContextGraphRepository {
    /// Creates a repository with explicit preview records.
    #[must_use]
    pub fn new(preview: ContextGraphProjection) -> Self {
        let component_content_hashes = component_content_hashes(&preview);
        Self {
            preview,
            commit_snapshot_state: Arc::new(RwLock::new(CommitSnapshotState {
                component_content_hashes,
                ..CommitSnapshotState::default()
            })),
            benchmark_evidence_state: Arc::new(RwLock::new(BenchmarkEvidenceState::default())),
            benchmark_workspace_projection_repository:
                InMemoryBenchmarkWorkspaceProjectionV1Repository::new(),
            knowledge_memory_projection_repository:
                InMemoryKnowledgeMemoryProjectionV1Repository::new(),
        }
    }

    /// Creates a repository with explicit preview records and commit graph snapshots.
    pub fn with_commit_graph_snapshots(
        preview: ContextGraphProjection,
        snapshots: impl IntoIterator<Item = CommitGraphSnapshot>,
    ) -> Result<Self, StorageRepositoryError> {
        let mut snapshot_map = BTreeMap::new();

        for snapshot in snapshots {
            let scope = snapshot.scope();
            ensure_commit_graph_snapshot_scope_exists(&preview, scope)?;
            let key = commit_graph_snapshot_key(scope);
            if snapshot_map.insert(key.clone(), snapshot).is_some() {
                return Err(StorageRepositoryError::CommitAlreadyExists {
                    context_id: scope.context_id().to_string(),
                    commit_id: scope.commit_id().to_string(),
                });
            }
        }

        let component_content_hashes = component_content_hashes(&preview);
        Ok(Self {
            preview,
            commit_snapshot_state: Arc::new(RwLock::new(CommitSnapshotState {
                commits: BTreeMap::new(),
                snapshots: snapshot_map,
                component_content_hashes,
                ..CommitSnapshotState::default()
            })),
            benchmark_evidence_state: Arc::new(RwLock::new(BenchmarkEvidenceState::default())),
            benchmark_workspace_projection_repository:
                InMemoryBenchmarkWorkspaceProjectionV1Repository::new(),
            knowledge_memory_projection_repository:
                InMemoryKnowledgeMemoryProjectionV1Repository::new(),
        })
    }

    /// Creates a repository backed by the deterministic Context Engineering preview.
    #[must_use]
    pub fn context_engineering_preview() -> Self {
        Self::new(ContextGraphProjection::context_engineering_preview())
    }

    fn ensure_knowledge_memory_projection_scope(
        &self,
        scope: KnowledgeMemoryProjectionScope,
    ) -> Result<(), KnowledgeMemoryProjectionPersistenceError> {
        let project_key = scope.project_id().to_string();
        let context_key = scope.context_id().to_string();
        let commit_key = scope.context_commit_id().to_string();
        let project_exists = self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_key);
        let context_exists = self
            .preview
            .contexts
            .iter()
            .any(|context| context.id == context_key && context.project_id == project_key);
        if !project_exists || !context_exists {
            return Err(KnowledgeMemoryProjectionPersistenceError::NotFound { scope });
        }

        let commit_exists_in_preview = self
            .preview
            .commits
            .iter()
            .any(|commit| commit.id == commit_key && commit.context_id == context_key);
        let commit_exists_in_private_state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| KnowledgeMemoryProjectionPersistenceError::RepositoryUnavailable)?
            .commits
            .contains_key(&(context_key, commit_key));
        if !commit_exists_in_preview && !commit_exists_in_private_state {
            return Err(KnowledgeMemoryProjectionPersistenceError::NotFound { scope });
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn persisted_branch_head(
        &self,
        context_id: ContextId,
        branch: &contextlab_versioning::BranchName,
    ) -> Option<CommitId> {
        self.commit_snapshot_state
            .read()
            .expect("read commit snapshot state")
            .branch_heads
            .get(&(context_id.to_string(), branch.as_str().to_owned()))
            .copied()
            .flatten()
    }

    #[cfg(test)]
    pub(crate) fn persisted_idempotency_receipt_commit_id(
        &self,
        identity_source: &contextlab_auth::IdentitySourceId,
        principal_id: &contextlab_auth::PrincipalId,
        context_id: ContextId,
        branch: &contextlab_versioning::BranchName,
        idempotency_key: &crate::IdempotencyKey,
    ) -> Option<CommitId> {
        self.commit_snapshot_state
            .read()
            .expect("read commit snapshot state")
            .idempotency
            .get(&(
                identity_source.as_str().to_owned(),
                principal_id.as_str().to_owned(),
                context_id.to_string(),
                branch.as_str().to_owned(),
                idempotency_key.as_str().to_owned(),
            ))
            .map(|record| {
                CommitId::from_uuid(
                    Uuid::parse_str(&record.commit_id).expect("valid persisted commit id"),
                )
            })
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Writer for InMemoryContextGraphRepository {
    async fn persist_benchmark_workspace_projection(
        &self,
        command: PersistBenchmarkWorkspaceProjectionV1,
    ) -> Result<BenchmarkWorkspaceProjectionWriteResult, BenchmarkWorkspaceProjectionPersistenceError>
    {
        self.benchmark_workspace_projection_repository
            .persist_benchmark_workspace_projection(command)
            .await
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Reader for InMemoryContextGraphRepository {
    async fn read_benchmark_workspace_projection(
        &self,
        query: BenchmarkWorkspaceProjectionV1Query,
    ) -> Result<
        contextlab_evaluation::BenchmarkWorkspaceProjectionV1,
        BenchmarkWorkspaceProjectionPersistenceError,
    > {
        self.benchmark_workspace_projection_repository
            .read_benchmark_workspace_projection(query)
            .await
    }

    async fn resolve_benchmark_workspace_projection_scope(
        &self,
        query: BenchmarkWorkspaceProjectionDecisionQuery,
    ) -> Result<
        Option<crate::BenchmarkWorkspaceProjectionReceiptScope>,
        BenchmarkWorkspaceProjectionPersistenceError,
    > {
        self.benchmark_workspace_projection_repository
            .resolve_benchmark_workspace_projection_scope(query)
            .await
    }
}

#[async_trait]
impl KnowledgeMemoryProjectionV1Repository for InMemoryContextGraphRepository {
    async fn persist_knowledge_memory_projection(
        &self,
        command: PersistKnowledgeMemoryProjectionV1,
    ) -> Result<KnowledgeMemoryProjectionWriteResult, KnowledgeMemoryProjectionPersistenceError>
    {
        self.ensure_knowledge_memory_projection_scope(command.scope())?;
        self.knowledge_memory_projection_repository
            .persist_knowledge_memory_projection(command)
            .await
    }

    async fn read_knowledge_memory_projection(
        &self,
        scope: KnowledgeMemoryProjectionScope,
    ) -> Result<
        contextlab_knowledge::KnowledgeMemoryContextProjectionV1,
        KnowledgeMemoryProjectionPersistenceError,
    > {
        self.ensure_knowledge_memory_projection_scope(scope)?;
        self.knowledge_memory_projection_repository
            .read_knowledge_memory_projection(scope)
            .await
    }
}

#[async_trait]
impl BenchmarkEvidenceWriter for InMemoryContextGraphRepository {
    async fn persist_benchmark_evaluation(
        &self,
        command: PersistBenchmarkEvaluationEvidence,
    ) -> Result<BenchmarkEvidenceWriteResult, StorageRepositoryError> {
        let evidence = command.evidence();
        let project_id = evidence.project_id();
        let context_id = evidence.context_id();
        let project_key = project_id.to_string();
        let context_key = context_id.to_string();
        if !self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_key)
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }
        if !self
            .preview
            .contexts
            .iter()
            .any(|context| context.id == context_key && context.project_id == project_key)
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }
        let commit_key = evidence.context_commit_id().to_string();
        let commit_exists_in_preview = self
            .preview
            .commits
            .iter()
            .any(|commit| commit.id == commit_key && commit.context_id == context_key);
        let commit_exists_in_private_state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?
            .commits
            .contains_key(&(context_key.clone(), commit_key));
        if !commit_exists_in_preview && !commit_exists_in_private_state {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!(
                    "context_commit:{context_id}/{}",
                    evidence.context_commit_id()
                ),
            });
        }
        let project_uuid = project_id.as_uuid();
        let context_uuid = context_id.as_uuid();
        let commit_uuid = evidence.context_commit_id().as_uuid();
        let decision_key = (
            project_uuid,
            context_uuid,
            commit_uuid,
            evidence.decision_id(),
        );
        let mut state = self
            .benchmark_evidence_state
            .write()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;

        if let Some(idempotency_key) = command.idempotency_key() {
            let key = (
                project_uuid,
                context_uuid,
                commit_uuid,
                idempotency_key.as_str().to_owned(),
            );
            if let Some(receipt) = state.execution_idempotency.get(&key) {
                if command
                    .request_digest()
                    .is_some_and(|digest| digest.as_str() == receipt.request_digest)
                {
                    let stored_key = (project_uuid, context_uuid, commit_uuid, receipt.decision_id);
                    let stored = state
                        .decisions
                        .get(&stored_key)
                        .ok_or(StorageRepositoryError::InMemoryStateUnavailable)?;
                    return Ok(BenchmarkEvidenceWriteResult::new(
                        BenchmarkEvidenceWriteDisposition::Replayed,
                        stored.evidence.clone(),
                    ));
                }
                return Err(StorageRepositoryError::IdempotencyKeyReused {
                    context_id: context_id.to_string(),
                    principal_id: "benchmark-execution".to_owned(),
                    idempotency_key: idempotency_key.to_string(),
                });
            }
        }

        if let Some(stored) = state.decisions.get(&decision_key) {
            if stored.evidence_digest == evidence.evidence_digest() {
                return Ok(BenchmarkEvidenceWriteResult::new(
                    BenchmarkEvidenceWriteDisposition::Replayed,
                    stored.evidence.clone(),
                ));
            }
            return Err(StorageRepositoryError::BenchmarkEvidenceDigestConflict {
                run_id: evidence.decision_id().to_string(),
            });
        }

        for dataset in command.datasets() {
            let key = (project_uuid, dataset.id());
            if state
                .datasets
                .get(&key)
                .is_some_and(|stored| stored != dataset)
            {
                return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                    definition_kind: "dataset",
                    definition_id: dataset.id().to_string(),
                });
            }
        }
        if state
            .suites
            .get(&(project_uuid, command.suite().id()))
            .is_some_and(|stored| stored != command.suite())
        {
            return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                definition_kind: "suite",
                definition_id: command.suite().id().to_string(),
            });
        }
        for run in command.runs() {
            let key = (project_uuid, context_uuid, commit_uuid, run.id().as_uuid());
            if state.runs.get(&key).is_some_and(|stored| stored != run) {
                return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                    definition_kind: "evaluation_run",
                    definition_id: run.id().as_uuid().to_string(),
                });
            }
            if state.runs.keys().any(
                |(stored_project_id, stored_context_id, stored_commit_id, stored_run_id)| {
                    *stored_project_id == project_uuid
                        && *stored_context_id == context_uuid
                        && *stored_run_id == run.id().as_uuid()
                        && *stored_commit_id != commit_uuid
                },
            ) {
                return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                    definition_kind: "evaluation_run",
                    definition_id: run.id().as_uuid().to_string(),
                });
            }
        }

        for dataset in command.datasets() {
            state
                .datasets
                .entry((project_uuid, dataset.id()))
                .or_insert_with(|| dataset.clone());
        }
        state
            .suites
            .entry((project_uuid, command.suite().id()))
            .or_insert_with(|| command.suite().clone());
        for run in command.runs() {
            state
                .runs
                .entry((project_uuid, context_uuid, commit_uuid, run.id().as_uuid()))
                .or_insert_with(|| run.clone());
        }
        let evidence = evidence.clone();
        state.decisions.insert(
            decision_key,
            StoredBenchmarkEvidence {
                evidence_digest: evidence.evidence_digest().to_owned(),
                evidence: evidence.clone(),
                sealed: true,
            },
        );
        if let (Some(idempotency_key), Some(request_digest)) =
            (command.idempotency_key(), command.request_digest())
        {
            state.execution_idempotency.insert(
                (
                    project_uuid,
                    context_uuid,
                    commit_uuid,
                    idempotency_key.as_str().to_owned(),
                ),
                BenchmarkExecutionIdempotencyRecord {
                    request_digest: request_digest.as_str().to_owned(),
                    decision_id: evidence.decision_id(),
                },
            );
        }

        Ok(BenchmarkEvidenceWriteResult::new(
            BenchmarkEvidenceWriteDisposition::Created,
            evidence,
        ))
    }
}

#[async_trait]
impl BenchmarkEvidenceRepository for InMemoryContextGraphRepository {
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
        let state = self
            .benchmark_evidence_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state
            .execution_idempotency
            .get(&(
                project_id.as_uuid(),
                context_id.as_uuid(),
                context_commit_id.as_uuid(),
                idempotency_key.as_str().to_owned(),
            ))
            .map(|receipt| {
                crate::benchmark_evidence::BenchmarkExecutionIdempotencyReceipt::new(
                    receipt.decision_id,
                    crate::RequestDigest::new(receipt.request_digest.clone())
                        .expect("persisted request digest is validated"),
                )
            }))
    }

    async fn get_benchmark_dataset(
        &self,
        project_id: ProjectId,
        dataset_id: BenchmarkDatasetId,
    ) -> Result<Option<BenchmarkDataset>, StorageRepositoryError> {
        if !self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_id.to_string())
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }
        let state = self
            .benchmark_evidence_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state
            .datasets
            .get(&(project_id.as_uuid(), dataset_id))
            .cloned())
    }

    async fn get_benchmark_suite(
        &self,
        project_id: ProjectId,
        suite_id: BenchmarkSuiteId,
    ) -> Result<Option<BenchmarkSuite>, StorageRepositoryError> {
        if !self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_id.to_string())
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }
        let state = self
            .benchmark_evidence_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state.suites.get(&(project_id.as_uuid(), suite_id)).cloned())
    }

    async fn get_benchmark_decision(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        decision_id: crate::BenchmarkDecisionId,
    ) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError> {
        if !self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_id.to_string())
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }
        if !self.preview.contexts.iter().any(|context| {
            context.id == context_id.to_string() && context.project_id == project_id.to_string()
        }) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }
        let state = self
            .benchmark_evidence_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state
            .decisions
            .get(&(
                project_id.as_uuid(),
                context_id.as_uuid(),
                context_commit_id.as_uuid(),
                decision_id,
            ))
            .map(|stored| stored.evidence.clone()))
    }

    async fn get_benchmark_decision_pair(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        baseline: BenchmarkDecisionComparisonScope,
        revised: BenchmarkDecisionComparisonScope,
    ) -> Result<BenchmarkDecisionPair, StorageRepositoryError> {
        if !self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_id.to_string())
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }
        if !self.preview.contexts.iter().any(|context| {
            context.id == context_id.to_string() && context.project_id == project_id.to_string()
        }) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }
        let state = self
            .benchmark_evidence_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let baseline = state
            .decisions
            .get(&(
                project_id.as_uuid(),
                context_id.as_uuid(),
                baseline.context_commit_id().as_uuid(),
                baseline.decision_id(),
            ))
            .map(|stored| stored.evidence.clone());
        let revised = state
            .decisions
            .get(&(
                project_id.as_uuid(),
                context_id.as_uuid(),
                revised.context_commit_id().as_uuid(),
                revised.decision_id(),
            ))
            .map(|stored| stored.evidence.clone());
        Ok(BenchmarkDecisionPair::new(baseline, revised))
    }

    async fn get_benchmark_run(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        run_id: EvaluationRunId,
    ) -> Result<Option<EvaluationRun>, StorageRepositoryError> {
        if !self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_id.to_string())
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }
        if !self.preview.contexts.iter().any(|context| {
            context.id == context_id.to_string() && context.project_id == project_id.to_string()
        }) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }
        let state = self
            .benchmark_evidence_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state
            .runs
            .get(&(
                project_id.as_uuid(),
                context_id.as_uuid(),
                context_commit_id.as_uuid(),
                run_id.as_uuid(),
            ))
            .cloned())
    }
}

#[async_trait]
impl BenchmarkDecisionDiscoveryRepository for InMemoryContextGraphRepository {
    async fn list_benchmark_decisions(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Vec<BenchmarkDecisionDiscoverySummary>, StorageRepositoryError> {
        if !self
            .preview
            .projects
            .iter()
            .any(|project| project.id == project_id.to_string())
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}"),
            });
        }
        if !self.preview.contexts.iter().any(|context| {
            context.id == context_id.to_string() && context.project_id == project_id.to_string()
        }) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }
        let state = self
            .benchmark_evidence_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let mut decisions = state
            .decisions
            .iter()
            .filter(
                |((stored_project, stored_context, stored_commit, _), stored)| {
                    *stored_project == project_id.as_uuid()
                        && *stored_context == context_id.as_uuid()
                        && *stored_commit == context_commit_id.as_uuid()
                        && stored.sealed
                },
            )
            .map(|(_, stored)| {
                let evidence = &stored.evidence;
                let suite = state
                    .suites
                    .get(&(project_id.as_uuid(), evidence.suite_id()))
                    .ok_or_else(|| StorageRepositoryError::Database {
                        message: "invalid stored benchmark discovery metadata".to_owned(),
                    })?;
                let mut datasets = evidence
                    .dataset_ids()
                    .iter()
                    .map(|dataset_id| {
                        let dataset = state
                            .datasets
                            .get(&(project_id.as_uuid(), *dataset_id))
                            .ok_or_else(|| StorageRepositoryError::Database {
                                message: "invalid stored benchmark discovery metadata".to_owned(),
                            })?;
                        Ok(BenchmarkDecisionDatasetSummary::new(
                            dataset.id(),
                            dataset.name().to_owned(),
                            dataset.cases().len(),
                        ))
                    })
                    .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
                datasets.sort_by_key(BenchmarkDecisionDatasetSummary::id);
                Ok(BenchmarkDecisionDiscoverySummary::new(
                    BenchmarkDecisionDiscoverySummaryInput {
                        project_id: evidence.project_id(),
                        context_id: evidence.context_id(),
                        context_commit_id: evidence.context_commit_id(),
                        decision_id: evidence.decision_id(),
                        suite: BenchmarkDecisionDiscoverySuiteSummary::new(
                            suite.id(),
                            suite.name().to_owned(),
                        ),
                        datasets,
                        status: evidence.status(),
                        recorded_at: evidence.recorded_at(),
                        run_count: evidence.run_ids().len(),
                    },
                ))
            })
            .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
        decisions.sort_by(|left, right| {
            right
                .recorded_at()
                .cmp(&left.recorded_at())
                .then_with(|| left.decision_id().cmp(&right.decision_id()))
        });
        Ok(decisions)
    }
}

#[async_trait]
impl ContextGraphProjectionRepository for InMemoryContextGraphRepository {
    async fn load_context_graph_projection(
        &self,
        scope: GraphProjectionScope,
    ) -> Result<ContextGraphProjection, StorageRepositoryError> {
        let projection = match scope {
            GraphProjectionScope::Preview => self.preview.clone(),
            GraphProjectionScope::Workspace { workspace_id } => {
                if self
                    .preview
                    .workspaces
                    .iter()
                    .any(|workspace| workspace.id == workspace_id)
                {
                    self.preview.clone()
                } else {
                    return Err(StorageRepositoryError::ScopeUnavailable {
                        scope: format!("workspace:{workspace_id}"),
                    });
                }
            }
        };
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let mut projection = projection;
        projection.components.retain(|component| {
            !state
                .removed_components
                .contains(&(component.context_id.clone(), component.id.clone()))
        });
        for component in &mut projection.components {
            if let Some(replacement) = state
                .components
                .get(&(component.context_id.clone(), component.id.clone()))
            {
                component.clone_from(replacement);
            }
            overlay_component_content(component, &state);
        }
        let additional_components = state
            .components
            .values()
            .filter(|component| {
                !state
                    .removed_components
                    .contains(&(component.context_id.clone(), component.id.clone()))
                    && !projection.components.iter().any(|projected| {
                        projected.context_id == component.context_id && projected.id == component.id
                    })
            })
            .cloned()
            .map(|mut component| {
                overlay_component_content(&mut component, &state);
                component
            })
            .collect::<Vec<_>>();
        projection.components.extend(additional_components);
        Ok(projection)
    }
}

#[async_trait]
impl ContextBranchRepository for InMemoryContextGraphRepository {
    async fn list_context_branch_heads(
        &self,
        context_id: ContextId,
    ) -> Result<Vec<ContextBranchHead>, ContextBranchRepositoryError> {
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| ContextBranchRepositoryError::InMemoryStateUnavailable)?;
        let context_key = context_id.to_string();
        let context_exists = self
            .preview
            .contexts
            .iter()
            .any(|context| context.id == context_key)
            || state
                .commits
                .keys()
                .any(|(stored_context_id, _)| stored_context_id == &context_key);
        if !context_exists {
            return Err(ContextBranchRepositoryError::UnknownContext { context_id });
        }

        let mut heads = state
            .branch_heads
            .iter()
            .filter(|((stored_context_id, _), _)| stored_context_id == &context_key)
            .map(|((_, branch), head_commit_id)| {
                let revision = state
                    .branch_revisions
                    .get(&(context_key.clone(), branch.clone()))
                    .copied()
                    .unwrap_or_default();
                let head_belongs_to_context = (*head_commit_id).is_none_or(|head_commit_id| {
                    state
                        .commits
                        .contains_key(&(context_key.clone(), head_commit_id.to_string()))
                        || self.preview.commits.iter().any(|commit| {
                            commit.context_id == context_key
                                && commit.id == head_commit_id.to_string()
                        })
                });
                ContextBranchHead::from_stored(
                    context_id,
                    branch.clone(),
                    (*head_commit_id).map(CommitId::as_uuid),
                    i64::try_from(revision).map_err(|_| {
                        ContextBranchRepositoryError::RevisionOverflow {
                            context_id,
                            branch: contextlab_versioning::BranchName::new(branch.clone())
                                .expect("branch state names are validated on write"),
                            revision,
                        }
                    })?,
                    head_belongs_to_context,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        heads.sort_by(|left, right| left.branch().as_str().cmp(right.branch().as_str()));
        Ok(heads)
    }

    async fn get_context_branch_head(
        &self,
        context_id: ContextId,
        branch: contextlab_versioning::BranchName,
    ) -> Result<ContextBranchHead, ContextBranchRepositoryError> {
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| ContextBranchRepositoryError::InMemoryStateUnavailable)?;
        let context_key = context_id.to_string();
        let context_exists = self
            .preview
            .contexts
            .iter()
            .any(|context| context.id == context_key)
            || state
                .commits
                .keys()
                .any(|(stored_context_id, _)| stored_context_id == &context_key);
        if !context_exists {
            return Err(ContextBranchRepositoryError::UnknownContext { context_id });
        }

        let branch_key = (context_key.clone(), branch.as_str().to_owned());
        let head_commit_id = state
            .branch_heads
            .get(&branch_key)
            .copied()
            .ok_or_else(|| ContextBranchRepositoryError::UnknownBranch {
                context_id,
                branch: branch.clone(),
            })?;
        let revision = state
            .branch_revisions
            .get(&branch_key)
            .copied()
            .unwrap_or_default();
        let head_belongs_to_context = head_commit_id.is_none_or(|head_commit_id| {
            state
                .commits
                .contains_key(&(context_key.clone(), head_commit_id.to_string()))
                || self.preview.commits.iter().any(|commit| {
                    commit.context_id == context_key && commit.id == head_commit_id.to_string()
                })
        });
        ContextBranchHead::from_stored(
            context_id,
            branch.as_str().to_owned(),
            head_commit_id.map(CommitId::as_uuid),
            i64::try_from(revision).map_err(|_| {
                ContextBranchRepositoryError::RevisionOverflow {
                    context_id,
                    branch: branch.clone(),
                    revision,
                }
            })?,
            head_belongs_to_context,
        )
    }
}

#[async_trait]
impl ContextCommitHistoryRepository for InMemoryContextGraphRepository {
    async fn load_context_commit_history(
        &self,
        context_id: ContextId,
    ) -> Result<contextlab_versioning::CommitHistory, StorageRepositoryError> {
        // Keep the static preview and mutable commit/branch state at one read
        // point. The guard is never held across an await.
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        context_commit_history_at_state(&self.preview, &state, context_id)
    }
}

#[async_trait]
impl ContextGraphReviewWitnessRepository for InMemoryContextGraphRepository {
    async fn read_context_graph_review_witness(
        &self,
        source_scope: crate::CommitGraphSnapshotScope,
        target_scope: crate::CommitGraphSnapshotScope,
    ) -> Result<ContextGraphReviewWitness, StorageRepositoryError> {
        let context_id = source_scope.context_id();
        let context_key = context_id.to_string();
        let project_id = project_id_for_context(&self.preview, context_id)?;
        if source_scope.project_id() != project_id || target_scope.project_id() != project_id {
            return Err(StorageRepositoryError::InvalidScope {
                scope: format!("context_graph_review:{source_scope}/{target_scope}"),
                reason: "requested project does not own the Context".to_owned(),
            });
        }
        ensure_context_exists(&self.preview, &context_key)?;

        // Keep commit records, branch heads, and both immutable snapshots at
        // one observation point. The guard is not held across an await.
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let history = context_commit_history_at_state(&self.preview, &state, context_id)?;
        let source = state
            .snapshots
            .get(&commit_graph_snapshot_key(source_scope))
            .cloned()
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: source_scope.to_string(),
            })?;
        let target = state
            .snapshots
            .get(&commit_graph_snapshot_key(target_scope))
            .cloned()
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: target_scope.to_string(),
            })?;
        let intermediate_snapshots = load_context_graph_review_intermediate_snapshots(
            &state,
            &history,
            source_scope,
            target_scope,
        )?;

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
impl ContextGraphBranchHeadReviewWitnessRepository for InMemoryContextGraphRepository {
    async fn read_context_graph_branch_head_review_witness(
        &self,
        source_scope: crate::CommitGraphSnapshotScope,
        branch: contextlab_versioning::BranchName,
    ) -> Result<
        ContextGraphBranchHeadReviewWitness,
        ContextGraphBranchHeadReviewWitnessRepositoryError,
    > {
        let context_id = source_scope.context_id();
        let context_key = context_id.to_string();
        let project_id = project_id_for_context(&self.preview, context_id).map_err(|source| {
            ContextGraphBranchHeadReviewWitnessRepositoryError::Storage { source }
        })?;
        if source_scope.project_id() != project_id {
            return Err(
                ContextGraphBranchHeadReviewWitnessRepositoryError::Storage {
                    source: StorageRepositoryError::InvalidScope {
                        scope: source_scope.to_string(),
                        reason: "requested project does not own the Context".to_owned(),
                    },
                },
            );
        }
        ensure_context_exists(&self.preview, &context_key).map_err(|source| {
            ContextGraphBranchHeadReviewWitnessRepositoryError::Storage { source }
        })?;

        let state = self.commit_snapshot_state.read().map_err(|_| {
            ContextGraphBranchHeadReviewWitnessRepositoryError::Storage {
                source: StorageRepositoryError::InMemoryStateUnavailable,
            }
        })?;
        let history = context_commit_history_at_state(&self.preview, &state, context_id).map_err(
            |source| ContextGraphBranchHeadReviewWitnessRepositoryError::Storage { source },
        )?;
        let target_commit_id =
            crate::context_graph_history_review::select_branch_head(&history, &branch)?;
        let target_scope = crate::CommitGraphSnapshotScope::new(
            source_scope.project_id(),
            source_scope.context_id(),
            target_commit_id,
        );
        let source = state
            .snapshots
            .get(&commit_graph_snapshot_key(source_scope))
            .cloned()
            .ok_or_else(
                || ContextGraphBranchHeadReviewWitnessRepositoryError::Storage {
                    source: StorageRepositoryError::ScopeUnavailable {
                        scope: source_scope.to_string(),
                    },
                },
            )?;
        let target = state
            .snapshots
            .get(&commit_graph_snapshot_key(target_scope))
            .cloned()
            .ok_or_else(
                || ContextGraphBranchHeadReviewWitnessRepositoryError::Storage {
                    source: StorageRepositoryError::ScopeUnavailable {
                        scope: target_scope.to_string(),
                    },
                },
            )?;
        let intermediate_snapshots = load_context_graph_review_intermediate_snapshots(
            &state,
            &history,
            source_scope,
            target_scope,
        )
        .map_err(|source| ContextGraphBranchHeadReviewWitnessRepositoryError::Storage { source })?;
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

#[async_trait]
impl ContextLifecycleRootRepository for InMemoryContextGraphRepository {
    async fn get_context_lifecycle_root(
        &self,
        context_id: ContextId,
    ) -> Result<ContextLifecycleRoot, StorageRepositoryError> {
        self.preview
            .contexts
            .iter()
            .find(|context| context.id == context_id.to_string())
            .map(|context| {
                let project_id = Uuid::parse_str(&context.project_id).map_err(|error| {
                    StorageRepositoryError::InvalidScope {
                        scope: format!("project:{}", context.project_id),
                        reason: error.to_string(),
                    }
                })?;
                Ok::<ContextLifecycleRoot, StorageRepositoryError>(ContextLifecycleRoot::new(
                    ProjectId::from_uuid(project_id),
                    context.name.clone(),
                ))
            })
            .transpose()?
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            })
    }
}

#[async_trait]
impl ContextLifecycleReadRepository for InMemoryContextGraphRepository {
    async fn get_context_lifecycle_read_facts(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<ContextLifecycleReadFacts, StorageRepositoryError> {
        let project_id = project_id_for_context(&self.preview, context_id)?;
        let scope = CommitGraphSnapshotScope::new(project_id, context_id, commit_id);
        let context_key = context_id.to_string();
        let commit_key = commit_id.to_string();
        ensure_context_exists(&self.preview, &context_key)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let history = validated_normal_first_parent_history(
            &self.preview,
            &state,
            &context_key,
            &commit_key,
            "Context lifecycle aggregate read",
            component_state_replay_conflict,
        )?;
        let graph_snapshot = state
            .snapshots
            .get(&commit_graph_snapshot_key(scope))
            .cloned()
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: scope.to_string(),
            })?;
        if graph_snapshot.scope() != scope {
            return Err(StorageRepositoryError::InvalidScope {
                scope: scope.to_string(),
                reason: format!("stored graph snapshot scope is {}", graph_snapshot.scope()),
            });
        }

        let steps = history
            .iter()
            .rev()
            .map(|commit| {
                let source_commit_id = Uuid::parse_str(&commit.id).map_err(|_| {
                    component_state_conflict(
                        "Context lifecycle aggregate read encountered an invalid commit identifier",
                    )
                })?;
                let changes = serde_json::from_value(commit.changes.clone()).map_err(|_| {
                    component_state_conflict(
                        "Context lifecycle aggregate read encountered invalid commit changes",
                    )
                })?;
                let revisions = state
                    .component_content_revisions
                    .iter()
                    .filter(|((revision_context_id, revision_commit_id, _), _)| {
                        revision_context_id == &context_key && revision_commit_id == &commit.id
                    })
                    .map(|(_, revision)| revision.clone());
                Ok(ComponentStateReplayStep::new(
                    CommitId::from_uuid(source_commit_id),
                    changes,
                    revisions,
                ))
            })
            .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
        let inventory = replay_context_component_state_snapshot(context_id, commit_id, steps)?;
        let contents = inventory
            .components()
            .iter()
            .filter_map(|component| {
                history.iter().find_map(|commit| {
                    state
                        .component_content_revisions
                        .get(&(
                            context_key.clone(),
                            commit.id.clone(),
                            component.component().id().to_string(),
                        ))
                        .cloned()
                })
            })
            .collect::<Vec<_>>();
        let replay_state =
            replay_state_from_records(context_id, history.into_iter().rev().collect::<Vec<_>>())?;

        ContextLifecycleReadFacts::from_parts(
            scope,
            inventory,
            contents,
            replay_state,
            graph_snapshot,
        )
    }
}

#[async_trait]
impl CommitGraphSnapshotRepository for InMemoryContextGraphRepository {
    async fn project_id_for_context(
        &self,
        context_id: ContextId,
    ) -> Result<ProjectId, StorageRepositoryError> {
        project_id_for_context(&self.preview, context_id)
    }

    async fn scope_for_context_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError> {
        let context_key = context_id.to_string();
        let project_id = project_id_for_context(&self.preview, context_id)?;

        let commit_key = commit_id.to_string();
        let static_commit_exists = self
            .preview
            .commits
            .iter()
            .any(|commit| commit.context_id == context_key && commit.id == commit_key);
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let dynamic_commit_exists = state
            .commits
            .contains_key(&(context_key.clone(), commit_key.clone()));
        if !static_commit_exists && !dynamic_commit_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}/commit:{commit_id}"),
            });
        }

        Ok(CommitGraphSnapshotScope::new(
            project_id, context_id, commit_id,
        ))
    }

    async fn get_commit_graph_snapshot(
        &self,
        scope: CommitGraphSnapshotScope,
    ) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
        ensure_commit_graph_snapshot_context_exists(&self.preview, scope)?;
        let context_id = scope.context_id().to_string();
        let commit_id = scope.commit_id().to_string();
        let static_commit_exists = self
            .preview
            .commits
            .iter()
            .any(|commit| commit.context_id == context_id && commit.id == commit_id);
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let dynamic_commit_exists = state
            .commits
            .contains_key(&(context_id.clone(), commit_id.clone()));

        if !static_commit_exists && !dynamic_commit_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: scope.to_string(),
            });
        }

        Ok(state
            .snapshots
            .get(&commit_graph_snapshot_key(scope))
            .cloned())
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
        ensure_commit_graph_snapshot_context_exists(&self.preview, scopes[0])?;

        let context_key = scope.context_id().to_string();
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let mut snapshots = Vec::with_capacity(scopes.len());

        for expected in scopes {
            let commit_key = expected.commit_id().to_string();
            let static_commit_exists = self
                .preview
                .commits
                .iter()
                .any(|commit| commit.context_id == context_key && commit.id == commit_key);
            let dynamic_commit_exists = state
                .commits
                .contains_key(&(context_key.clone(), commit_key.clone()));
            if !static_commit_exists && !dynamic_commit_exists {
                return Err(StorageRepositoryError::ScopeUnavailable {
                    scope: expected.to_string(),
                });
            }

            let snapshot = state
                .snapshots
                .get(&commit_graph_snapshot_key(expected))
                .cloned()
                .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                    scope: expected.to_string(),
                })?;
            if snapshot.scope() != expected {
                return Err(StorageRepositoryError::InvalidScope {
                    scope: expected.to_string(),
                    reason: format!("stored snapshot scope is {}", snapshot.scope()),
                });
            }
            snapshots.push(snapshot);
        }

        let [base, left, right] = snapshots
            .try_into()
            .expect("the batch scope always contains three snapshots");
        Ok((base, left, right))
    }
}

#[async_trait]
impl ContextMergeReviewWitnessRepository for InMemoryContextGraphRepository {
    async fn load_context_merge_review_witness(
        &self,
        scope: crate::ContextMergeTipScope,
    ) -> Result<ContextMergeReviewWitness, ContextMergeReviewWitnessRepositoryError> {
        let context_key = scope.context_id().to_string();
        ensure_context_exists(&self.preview, &context_key)
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        let project_id = project_id_for_context(&self.preview, scope.context_id())
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        if project_id != scope.project_id() {
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

        let state = self.commit_snapshot_state.read().map_err(|_| {
            ContextMergeReviewWitnessRepositoryError::Read(
                StorageRepositoryError::InMemoryStateUnavailable,
            )
        })?;
        let mut records = self
            .preview
            .commits
            .iter()
            .filter(|commit| commit.context_id == context_key)
            .cloned()
            .collect::<Vec<_>>();
        records.extend(
            state
                .commits
                .values()
                .filter(|commit| commit.context_id == context_key)
                .cloned(),
        );
        let graph = commit_graph_from_records(scope.context_id(), records)
            .map_err(ContextMergeReviewWitnessRepositoryError::Read)?;
        let plan = MergePlan::resolve(&graph, scope.left_commit_id(), scope.right_commit_id())
            .map_err(ContextMergeReviewWitnessRepositoryError::PlanResolution)?;
        let MergePlan::ThreeWay { base, left, right } = plan.clone() else {
            return Err(ContextMergeReviewWitnessRepositoryError::InvalidWitness(
                ContextMergeReviewWitnessError::NonThreeWay,
            ));
        };
        let input_scope = crate::ContextMergeInputScope::new(
            scope.project_id(),
            scope.context_id(),
            base,
            left,
            right,
        )
        .map_err(|error| {
            ContextMergeReviewWitnessRepositoryError::InvalidWitness(
                ContextMergeReviewWitnessError::InvalidInputScope(error),
            )
        })?;

        let snapshots = [
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
        ]
        .into_iter()
        .map(|(side, expected)| {
            let commit_key = expected.commit_id().to_string();
            let static_commit_exists = self
                .preview
                .commits
                .iter()
                .any(|commit| commit.context_id == context_key && commit.id == commit_key);
            let dynamic_commit_exists = state
                .commits
                .contains_key(&(context_key.clone(), commit_key));
            if !static_commit_exists && !dynamic_commit_exists {
                return Err(ContextMergeReviewWitnessRepositoryError::Read(
                    StorageRepositoryError::ScopeUnavailable {
                        scope: expected.to_string(),
                    },
                ));
            }
            let snapshot = state
                .snapshots
                .get(&commit_graph_snapshot_key(expected))
                .cloned()
                .ok_or(
                    ContextMergeReviewWitnessRepositoryError::SnapshotUnavailable {
                        side,
                        scope: expected,
                    },
                )?;
            if snapshot.scope() != expected {
                return Err(ContextMergeReviewWitnessRepositoryError::SnapshotRead {
                    side,
                    scope: expected,
                    source: StorageRepositoryError::InvalidScope {
                        scope: expected.to_string(),
                        reason: format!("stored snapshot scope is {}", snapshot.scope()),
                    },
                });
            }
            Ok(snapshot)
        })
        .collect::<Result<Vec<_>, _>>()?;
        let [base_snapshot, left_snapshot, right_snapshot] = snapshots
            .try_into()
            .expect("the merge witness always contains three snapshots");

        ContextMergeReviewWitness::new(plan, base_snapshot, left_snapshot, right_snapshot)
            .map_err(ContextMergeReviewWitnessRepositoryError::InvalidWitness)
    }
}

#[async_trait]
impl ContextCommitSnapshotWriter for InMemoryContextGraphRepository {
    async fn create_commit_snapshot(
        &self,
        command: CreateContextCommitSnapshot,
    ) -> Result<CommitGraphSnapshot, StorageRepositoryError> {
        let mut state = self
            .commit_snapshot_state
            .write()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        persist_commit_snapshot(&self.preview, &mut state, command)
    }
}

#[async_trait]
impl ContextDiffSnapshotV1Repository for InMemoryContextGraphRepository {
    async fn persist_context_diff_snapshot(
        &self,
        command: PersistContextDiffSnapshotV1,
    ) -> Result<ContextDiffSnapshotWriteResult, ContextDiffSnapshotPersistenceError> {
        let record = command.record().clone();
        let scope = record.scope();
        let key = context_diff_snapshot_key(scope, record.schema_version());
        let mut state = self
            .commit_snapshot_state
            .write()
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;
        if let Some(existing) = state.diff_snapshots.get(&key) {
            if existing == &record {
                return Ok(ContextDiffSnapshotWriteResult::new(
                    scope,
                    ContextDiffSnapshotWriteDisposition::Replayed,
                ));
            }
            return Err(ContextDiffSnapshotPersistenceError::Conflict { scope });
        }
        state.diff_snapshots.insert(key, record);
        Ok(ContextDiffSnapshotWriteResult::new(
            scope,
            ContextDiffSnapshotWriteDisposition::Created,
        ))
    }

    async fn read_context_diff_snapshot(
        &self,
        scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError> {
        crate::context_diff_snapshot::validate_scope(scope)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;
        read_context_diff_snapshot_from_state(&state, scope)
    }
}

#[async_trait]
impl ContextDiffSnapshotV1PairRepository for InMemoryContextGraphRepository {
    async fn read_context_diff_snapshot_pair(
        &self,
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Pair, ContextDiffSnapshotPersistenceError> {
        crate::context_diff_snapshot::validate_scope(source_scope)?;
        crate::context_diff_snapshot::validate_scope(target_scope)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;
        let source = read_context_diff_snapshot_from_state(&state, source_scope)?;
        let target = read_context_diff_snapshot_from_state(&state, target_scope)?;
        Ok(ContextDiffSnapshotV1Pair::new(source, target))
    }
}

#[async_trait]
impl ContextWorkflowBindingRepository for InMemoryContextGraphRepository {
    async fn persist_workflow_context_binding(
        &self,
        binding: WorkflowContextBinding,
    ) -> Result<WorkflowContextBindingWriteResult, StorageRepositoryError> {
        let source = binding.context_source();
        let context_id = source.context_id().to_string();
        let commit_id = source.commit_id().to_string();
        ensure_context_exists(&self.preview, &context_id)?;

        let mut state = self
            .commit_snapshot_state
            .write()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        if !snapshot_exists_for_context_commit(&state, &context_id, &commit_id) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        let workflow_key = (binding.workflow_id(), binding.workflow_revision());
        if let Some(existing) = state.workflow_context_bindings.get(&workflow_key) {
            if existing == &binding {
                return Ok(WorkflowContextBindingWriteResult::replayed(
                    existing.clone(),
                ));
            }
            return Err(workflow_context_binding_conflict(
                binding.workflow_id(),
                binding.workflow_revision(),
            ));
        }

        state
            .workflow_context_bindings
            .insert(workflow_key, binding.clone());
        Ok(WorkflowContextBindingWriteResult::created(binding))
    }

    async fn get_workflow_context_binding(
        &self,
        workflow_id: WorkflowId,
        workflow_revision: WorkflowRevision,
    ) -> Result<Option<WorkflowContextBinding>, StorageRepositoryError> {
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state
            .workflow_context_bindings
            .get(&(workflow_id, workflow_revision))
            .cloned())
    }

    async fn list_workflow_context_bindings_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<Vec<WorkflowContextBinding>, StorageRepositoryError> {
        let context_id_text = context_id.to_string();
        let commit_id_text = commit_id.to_string();
        ensure_context_exists(&self.preview, &context_id_text)?;

        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        if !snapshot_exists_for_context_commit(&state, &context_id_text, &commit_id_text) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id_text}/{commit_id_text}"),
            });
        }

        let mut bindings = state
            .workflow_context_bindings
            .values()
            .filter(|binding| {
                binding.context_source().context_id() == context_id
                    && binding.context_source().commit_id() == commit_id
            })
            .cloned()
            .collect::<Vec<_>>();
        bindings.sort_by_key(|binding| {
            (
                binding.workflow_id(),
                binding.workflow_revision(),
                binding.id(),
            )
        });
        Ok(bindings)
    }
}

#[async_trait]
impl BenchmarkDefinitionBindingWriter for InMemoryContextGraphRepository {
    async fn persist_benchmark_definition_binding(
        &self,
        command: BenchmarkDefinitionBindingCommand,
    ) -> Result<BenchmarkDefinitionBindingWriteResult, StorageRepositoryError> {
        let (_principal, binding, idempotency_key, request_digest) = command.into_parts();
        let project_id = binding.project_id().as_uuid();
        let context_id = binding.context_id().as_uuid();
        let commit_id = binding.context_commit_id().as_uuid();
        let context_key = context_id.to_string();
        let project_key = project_id.to_string();
        ensure_project_exists(&self.preview, &project_key)?;
        if !self
            .preview
            .contexts
            .iter()
            .any(|context| context.id == context_key && context.project_id == project_key)
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }

        let mut commit_state = self
            .commit_snapshot_state
            .write()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let static_commit_exists = self.preview.commits.iter().any(|commit| {
            commit.context_id == context_key
                && commit.id == commit_id.to_string()
                && commit.branch_name == binding.branch().as_str()
        });
        let dynamic_commit_exists = commit_state
            .commits
            .contains_key(&(context_key.clone(), commit_id.to_string()));
        if !static_commit_exists && !dynamic_commit_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            });
        }

        let identity_key = (
            _principal.identity().source().as_str().to_owned(),
            _principal.id().as_str().to_owned(),
            context_key.clone(),
            binding.branch().as_str().to_owned(),
            idempotency_key.as_str().to_owned(),
        );
        if let Some(existing) = commit_state
            .benchmark_definition_idempotency
            .get(&identity_key)
        {
            if existing.request_digest != request_digest.as_str() {
                return Err(StorageRepositoryError::IdempotencyKeyReused {
                    context_id: context_key,
                    principal_id: _principal.id().as_str().to_owned(),
                    idempotency_key: idempotency_key.to_string(),
                });
            }
            let existing_binding = commit_state
                .benchmark_definition_bindings
                .get(&(project_id, context_id, commit_id, existing.binding_id))
                .cloned()
                .ok_or_else(|| StorageRepositoryError::Database {
                    message: "benchmark definition idempotency binding is unavailable".to_owned(),
                })?;
            return Ok(BenchmarkDefinitionBindingWriteResult::new(
                existing_binding,
                BenchmarkDefinitionBindingWriteDisposition::Replayed,
            ));
        }

        let actual_branch_head = commit_state
            .branch_heads
            .get(&(context_key.clone(), binding.branch().as_str().to_owned()))
            .copied()
            .flatten()
            .or_else(|| static_commit_exists.then_some(binding.context_commit_id()));
        if actual_branch_head != Some(binding.context_commit_id()) {
            return Err(StorageRepositoryError::BranchHeadConflict {
                expected: Some(binding.context_commit_id().to_string()),
                actual: actual_branch_head.map(|value| value.to_string()),
            });
        }

        let binding_key = (project_id, context_id, commit_id, binding.id());
        if let Some(existing) = commit_state.benchmark_definition_bindings.get(&binding_key) {
            if existing != &binding {
                return Err(StorageRepositoryError::BenchmarkDefinitionBindingConflict {
                    binding_id: binding.id().to_string(),
                });
            }
            return Err(StorageRepositoryError::BenchmarkDefinitionBindingConflict {
                binding_id: binding.id().to_string(),
            });
        }
        if let Some(existing) =
            commit_state
                .benchmark_definition_bindings
                .values()
                .find(|existing| {
                    existing.project_id() == binding.project_id()
                        && existing.context_id() == binding.context_id()
                        && existing.context_commit_id() == binding.context_commit_id()
                        && existing.suite().id() == binding.suite().id()
                })
        {
            return Err(StorageRepositoryError::BenchmarkDefinitionBindingConflict {
                binding_id: existing.id().to_string(),
            });
        }

        let mut evidence_state = self
            .benchmark_evidence_state
            .write()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        for dataset in binding.datasets() {
            let key = (project_id, dataset.id());
            if evidence_state
                .datasets
                .get(&key)
                .is_some_and(|stored| stored != dataset)
            {
                return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                    definition_kind: "dataset",
                    definition_id: dataset.id().to_string(),
                });
            }
        }
        if evidence_state
            .suites
            .get(&(project_id, binding.suite().id()))
            .is_some_and(|stored| stored != binding.suite())
        {
            return Err(StorageRepositoryError::BenchmarkDefinitionConflict {
                definition_kind: "suite",
                definition_id: binding.suite().id().to_string(),
            });
        }
        for dataset in binding.datasets() {
            evidence_state
                .datasets
                .entry((project_id, dataset.id()))
                .or_insert_with(|| dataset.clone());
        }
        evidence_state
            .suites
            .entry((project_id, binding.suite().id()))
            .or_insert_with(|| binding.suite().clone());
        commit_state
            .benchmark_definition_bindings
            .insert(binding_key, binding.clone());
        commit_state.benchmark_definition_idempotency.insert(
            identity_key,
            BenchmarkDefinitionIdempotencyRecord {
                request_digest: request_digest.as_str().to_owned(),
                binding_id: binding.id(),
            },
        );
        Ok(BenchmarkDefinitionBindingWriteResult::new(
            binding,
            BenchmarkDefinitionBindingWriteDisposition::Created,
        ))
    }
}

#[async_trait]
impl BenchmarkDefinitionBindingRepository for InMemoryContextGraphRepository {
    async fn get_benchmark_definition_binding(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        binding_id: crate::BenchmarkDefinitionBindingId,
    ) -> Result<Option<BenchmarkDefinitionBinding>, StorageRepositoryError> {
        ensure_project_exists(&self.preview, &project_id.to_string())?;
        if !self.preview.contexts.iter().any(|context| {
            context.id == context_id.to_string() && context.project_id == project_id.to_string()
        }) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("context:{context_id}"),
            });
        }
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state
            .benchmark_definition_bindings
            .get(&(
                project_id.as_uuid(),
                context_id.as_uuid(),
                context_commit_id.as_uuid(),
                binding_id,
            ))
            .cloned())
    }

    async fn list_benchmark_definition_bindings_at_commit(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Vec<BenchmarkDefinitionBinding>, StorageRepositoryError> {
        ensure_project_exists(&self.preview, &project_id.to_string())?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let mut bindings = state
            .benchmark_definition_bindings
            .values()
            .filter(|binding| {
                binding.project_id() == project_id
                    && binding.context_id() == context_id
                    && binding.context_commit_id() == context_commit_id
            })
            .cloned()
            .collect::<Vec<_>>();
        bindings.sort_by_key(BenchmarkDefinitionBinding::id);
        Ok(bindings)
    }
}

#[async_trait]
impl GuardedContextCommitWriter for InMemoryContextGraphRepository {
    async fn create_guarded_commit_snapshot(
        &self,
        command: GuardedContextCommitWrite,
    ) -> Result<GuardedCommitWriteResult, StorageRepositoryError> {
        let (
            principal,
            expected_branch_head,
            idempotency_key,
            request_digest,
            snapshot_command,
            component_content_mutation,
        ) = command.into_parts();
        let context_id = snapshot_command.commit().context_id().to_string();
        let branch_name = snapshot_command.commit().branch().as_str().to_owned();
        let identity_source = principal.identity().source().as_str().to_owned();
        let principal_id = principal.id().as_str().to_owned();
        ensure_context_exists(&self.preview, &context_id)?;

        let mut state = self
            .commit_snapshot_state
            .write()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let idempotency_state_key = (
            identity_source,
            principal_id.clone(),
            context_id.clone(),
            branch_name.clone(),
            idempotency_key.as_str().to_owned(),
        );

        if let Some(existing) = state.idempotency.get(&idempotency_state_key) {
            if existing.request_digest != request_digest.as_str() {
                return Err(StorageRepositoryError::IdempotencyKeyReused {
                    context_id,
                    principal_id,
                    idempotency_key: idempotency_key.to_string(),
                });
            }

            let snapshot = snapshot_for_context_commit(&state, &context_id, &existing.commit_id)
                .ok_or_else(|| StorageRepositoryError::Database {
                    message: "idempotency result snapshot is unavailable".to_owned(),
                })?;
            return Ok(GuardedCommitWriteResult {
                snapshot,
                disposition: GuardedCommitWriteDisposition::Replayed,
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

        let branch_state_key = (context_id.clone(), branch_name);
        let actual_branch_head = state.branch_heads.get(&branch_state_key).copied().flatten();
        let next_branch_revision = state
            .branch_revisions
            .get(&branch_state_key)
            .copied()
            .unwrap_or_default()
            .checked_add(1)
            .ok_or_else(|| StorageRepositoryError::Database {
                message: "branch revision overflow".to_owned(),
            })?;
        let required_parent = normal_commit_parent(expected_branch_head, actual_branch_head)
            .map_err(|conflict| StorageRepositoryError::BranchHeadConflict {
                expected: conflict.expected.map(|commit_id| commit_id.to_string()),
                actual: conflict.actual.map(|commit_id| commit_id.to_string()),
            })?;
        let supplied_parent_ids = snapshot_command.commit().parent_ids();
        let required_parent_ids = required_parent.into_iter().collect::<Vec<_>>();

        if supplied_parent_ids != required_parent_ids.as_slice() {
            return Err(StorageRepositoryError::CommitParentMismatch {
                expected_parent_id: required_parent_ids.first().map(ToString::to_string),
                actual_parent_ids: supplied_parent_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            });
        }

        let (persisted_component, persisted_revision, persisted_removal, persisted_descriptor) =
            match component_content_mutation {
                Some(ComponentContentMutationWrite::Revision(revision)) => (
                    None,
                    Some(prepare_component_content_revision(
                        &self.preview,
                        &state,
                        &snapshot_command,
                        revision,
                    )?),
                    None,
                    None,
                ),
                Some(ComponentContentMutationWrite::Creation(creation)) => {
                    let (component, revision) = prepare_component_content_creation(
                        &self.preview,
                        &state,
                        &snapshot_command,
                        creation,
                    )?;
                    (Some(component), Some(revision), None, None)
                }
                Some(ComponentContentMutationWrite::Removal(removal)) => (
                    None,
                    None,
                    Some(prepare_component_removal(
                        &self.preview,
                        &state,
                        &snapshot_command,
                        removal,
                    )?),
                    None,
                ),
                Some(ComponentContentMutationWrite::Descriptor(descriptor)) => (
                    None,
                    None,
                    None,
                    Some(prepare_component_descriptor_revision(
                        &self.preview,
                        &state,
                        &snapshot_command,
                        descriptor,
                    )?),
                ),
                None => (None, None, None, None),
            };

        let commit_id = snapshot_command.commit().id();
        let snapshot = persist_commit_snapshot(&self.preview, &mut state, snapshot_command)?;
        if let Some(revision) = persisted_revision {
            let component_state_key = (context_id.clone(), revision.component_id().to_string());
            state.component_content_hashes.insert(
                component_state_key.clone(),
                revision.resulting_content_hash().as_str().to_owned(),
            );
            state
                .component_content_updated_at
                .insert(component_state_key, revision.captured_at());
            state.component_content_revisions.insert(
                (
                    context_id.clone(),
                    revision.commit_id().to_string(),
                    revision.component_id().to_string(),
                ),
                revision,
            );
        }
        if let Some(component) = persisted_component {
            state.components.insert(
                (component.context_id.clone(), component.id.clone()),
                component,
            );
        }
        if let Some(component) = persisted_descriptor {
            state.components.insert(
                (component.context_id.clone(), component.id.clone()),
                component,
            );
        }
        if let Some(removal) = persisted_removal {
            let component_state_key = (context_id.clone(), removal.component_id().to_string());
            state.removed_components.insert(component_state_key.clone());
            state.component_content_hashes.remove(&component_state_key);
            state
                .component_content_updated_at
                .remove(&component_state_key);
        }
        let branch_revision_key = branch_state_key.clone();
        state.branch_heads.insert(branch_state_key, Some(commit_id));
        state
            .branch_revisions
            .insert(branch_revision_key, next_branch_revision);
        state.idempotency.insert(
            idempotency_state_key,
            IdempotencyRecord {
                request_digest: request_digest.as_str().to_owned(),
                commit_id: snapshot.commit_id().to_string(),
            },
        );

        Ok(GuardedCommitWriteResult {
            snapshot,
            disposition: GuardedCommitWriteDisposition::Created,
        })
    }
}

#[async_trait]
impl ComponentContentRevisionRepository for InMemoryContextGraphRepository {
    async fn get_component_content_revision(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<ComponentContentRevision>, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id.to_string())?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        Ok(state
            .component_content_revisions
            .get(&(
                context_id.to_string(),
                commit_id.to_string(),
                component_id.to_string(),
            ))
            .cloned())
    }

    async fn get_component_content_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<ComponentContentRevision>, StorageRepositoryError> {
        let context_id = context_id.to_string();
        let component_id = component_id.to_string();
        ensure_context_exists(&self.preview, &context_id)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let history = validated_normal_first_parent_history(
            &self.preview,
            &state,
            &context_id,
            &commit_id.to_string(),
            "component content replay",
            component_content_replay_conflict,
        )?;

        Ok(history.iter().find_map(|commit| {
            state
                .component_content_revisions
                .get(&(context_id.clone(), commit.id.clone(), component_id.clone()))
                .cloned()
        }))
    }
}

#[async_trait]
impl ComponentStateAtCommitRepository for InMemoryContextGraphRepository {
    async fn get_component_state_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<crate::ComponentStateAtCommit>, StorageRepositoryError> {
        let context_key = context_id.to_string();
        let component_key = component_id.to_string();
        ensure_context_exists(&self.preview, &context_key)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let history = validated_normal_first_parent_history(
            &self.preview,
            &state,
            &context_key,
            &commit_id.to_string(),
            "component state replay",
            component_state_replay_conflict,
        )?;
        let steps = history
            .iter()
            .rev()
            .map(|commit| {
                let source_commit_id = Uuid::parse_str(&commit.id).map_err(|_| {
                    component_state_conflict(
                        "component state replay encountered an invalid stored commit identifier",
                    )
                })?;
                let changes = serde_json::from_value(commit.changes.clone()).map_err(|_| {
                    component_state_conflict(
                        "component state replay encountered invalid stored commit changes",
                    )
                })?;
                Ok(ComponentStateReplayStep::new(
                    CommitId::from_uuid(source_commit_id),
                    changes,
                    state
                        .component_content_revisions
                        .get(&(
                            context_key.clone(),
                            commit.id.clone(),
                            component_key.clone(),
                        ))
                        .cloned(),
                ))
            })
            .collect::<Result<Vec<_>, StorageRepositoryError>>()?;

        replay_component_state(context_id, commit_id, component_id, steps)
    }
}

#[async_trait]
impl ContextComponentStateSnapshotAtCommitRepository for InMemoryContextGraphRepository {
    async fn get_context_component_state_snapshot_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: CommitId,
    ) -> Result<crate::ContextComponentStateSnapshotAtCommit, StorageRepositoryError> {
        let context_key = context_id.to_string();
        ensure_context_exists(&self.preview, &context_key)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let history = validated_normal_first_parent_history(
            &self.preview,
            &state,
            &context_key,
            &commit_id.to_string(),
            "Context component state replay",
            component_state_replay_conflict,
        )?;
        let steps = history
            .iter()
            .rev()
            .map(|commit| {
                let source_commit_id = Uuid::parse_str(&commit.id).map_err(|_| {
                    component_state_conflict(
                        "Context component state replay encountered an invalid stored commit identifier",
                    )
                })?;
                let changes = serde_json::from_value(commit.changes.clone()).map_err(|_| {
                    component_state_conflict(
                        "Context component state replay encountered invalid stored commit changes",
                    )
                })?;
                let revisions = state
                    .component_content_revisions
                    .iter()
                    .filter(|((revision_context_id, revision_commit_id, _), _)| {
                        revision_context_id == &context_key && revision_commit_id == &commit.id
                    })
                    .map(|(_, revision)| revision.clone());
                Ok(ComponentStateReplayStep::new(
                    CommitId::from_uuid(source_commit_id),
                    changes,
                    revisions,
                ))
            })
            .collect::<Result<Vec<_>, StorageRepositoryError>>()?;

        replay_context_component_state_snapshot(context_id, commit_id, steps)
    }
}

#[async_trait]
impl ContextReplayStateAtCommitRepository for InMemoryContextGraphRepository {
    async fn get_context_replay_state_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<contextlab_versioning::ReplayState, StorageRepositoryError> {
        let context_key = context_id.to_string();
        ensure_context_exists(&self.preview, &context_key)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        let history = validated_normal_first_parent_history(
            &self.preview,
            &state,
            &context_key,
            &commit_id.to_string(),
            "Context replay state",
            component_state_replay_conflict,
        )?;

        replay_state_from_records(context_id, history.into_iter().rev().collect::<Vec<_>>())
    }
}

fn context_commit_history_at_state(
    preview: &ContextGraphProjection,
    state: &CommitSnapshotState,
    context_id: ContextId,
) -> Result<contextlab_versioning::CommitHistory, StorageRepositoryError> {
    let context_key = context_id.to_string();
    ensure_context_exists(preview, &context_key)?;
    let mut commits = preview
        .commits
        .iter()
        .filter(|commit| commit.context_id == context_key)
        .cloned()
        .chain(
            state
                .commits
                .values()
                .filter(|commit| commit.context_id == context_key)
                .cloned(),
        )
        .map(context_commit_from_record)
        .collect::<Result<Vec<_>, _>>()?;
    commits.sort_by(|left, right| {
        left.authored_at()
            .cmp(&right.authored_at())
            .then_with(|| left.id().as_uuid().cmp(&right.id().as_uuid()))
    });

    let mut heads = state
        .branch_heads
        .iter()
        .filter(|((stored_context_id, _), _)| stored_context_id == &context_key)
        .map(|((_, branch), head_commit_id)| {
            let branch =
                contextlab_versioning::BranchName::new(branch.clone()).map_err(|error| {
                    StorageRepositoryError::InvalidScope {
                        scope: format!("context_commit_history:{context_id}"),
                        reason: error.to_string(),
                    }
                })?;
            Ok(BranchHead::new(branch, *head_commit_id))
        })
        .collect::<Result<Vec<_>, StorageRepositoryError>>()?;
    heads.sort_by(|left, right| left.branch().as_str().cmp(right.branch().as_str()));

    assemble_context_commit_history_from_commits(context_id, commits, heads)
}

fn validated_normal_first_parent_history(
    preview: &ContextGraphProjection,
    state: &CommitSnapshotState,
    context_id: &str,
    target_commit_id: &str,
    operation: &str,
    conflict: fn(String) -> StorageRepositoryError,
) -> Result<Vec<crate::ContextCommitRecord>, StorageRepositoryError> {
    let mut current_commit_id = target_commit_id.to_owned();
    let mut visited_commit_ids = BTreeSet::new();
    let mut history = Vec::new();

    loop {
        if !visited_commit_ids.insert(current_commit_id.clone()) {
            return Err(conflict(format!(
                "{operation} encountered a commit ancestry cycle"
            )));
        }
        let commit = preview
            .commits
            .iter()
            .find(|commit| commit.context_id == context_id && commit.id == current_commit_id)
            .cloned()
            .or_else(|| {
                state
                    .commits
                    .get(&(context_id.to_owned(), current_commit_id.clone()))
                    .cloned()
            })
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{current_commit_id}"),
            })?;

        if commit.context_id != context_id {
            return Err(conflict(format!(
                "{operation} encountered a cross-context parent"
            )));
        }
        if commit.parent_commit_ids.len() > 1 {
            return Err(conflict(format!(
                "{operation} does not support merge ancestry"
            )));
        }

        let parent_commit_id = commit.parent_commit_ids.first().cloned();
        history.push(commit);
        let Some(parent_commit_id) = parent_commit_id else {
            return Ok(history);
        };
        current_commit_id = parent_commit_id;
    }
}

fn component_content_replay_conflict(reason: String) -> StorageRepositoryError {
    StorageRepositoryError::ComponentContentRevisionConflict { reason }
}

fn component_state_replay_conflict(reason: String) -> StorageRepositoryError {
    component_state_conflict(reason)
}

fn component_content_hashes(
    preview: &ContextGraphProjection,
) -> BTreeMap<(String, String), String> {
    preview
        .components
        .iter()
        .map(|component| {
            (
                (component.context_id.clone(), component.id.clone()),
                component.content_hash.clone(),
            )
        })
        .collect()
}

fn overlay_component_content(
    component: &mut crate::ContextComponentRecord,
    state: &CommitSnapshotState,
) {
    let key = (component.context_id.clone(), component.id.clone());
    if let Some(content_hash) = state.component_content_hashes.get(&key) {
        component.content_hash.clone_from(content_hash);
    }
    if let Some(updated_at) = state.component_content_updated_at.get(&key) {
        component.updated_at = *updated_at;
    }
}

fn prepare_component_content_revision(
    preview: &ContextGraphProjection,
    state: &CommitSnapshotState,
    snapshot_command: &CreateContextCommitSnapshot,
    revision: crate::ComponentContentRevisionWrite,
) -> Result<ComponentContentRevision, StorageRepositoryError> {
    let context_id = snapshot_command.commit().context_id();
    let component_id = revision.component_id();
    if state
        .removed_components
        .contains(&(context_id.to_string(), component_id.to_string()))
    {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        });
    }
    let component = state
        .components
        .get(&(context_id.to_string(), component_id.to_string()))
        .or_else(|| {
            preview.components.iter().find(|component| {
                component.context_id == context_id.to_string()
                    && component.id == component_id.to_string()
            })
        })
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        })?;
    if component.kind.as_str()
        != crate::component_content_revision::component_kind_storage_value(
            revision.component_kind(),
        )
    {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component kind does not match the revision".to_owned(),
        });
    }
    let current_hash = state
        .component_content_hashes
        .get(&(context_id.to_string(), component_id.to_string()))
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        })?;
    if current_hash != revision.previous_content_hash().as_str() {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component content hash is stale".to_owned(),
        });
    }

    Ok(revision.into_revision(context_id, snapshot_command.commit().id()))
}

fn prepare_component_removal(
    preview: &ContextGraphProjection,
    state: &CommitSnapshotState,
    snapshot_command: &CreateContextCommitSnapshot,
    removal: crate::ComponentRemovalWrite,
) -> Result<crate::ComponentRemovalWrite, StorageRepositoryError> {
    let context_id = snapshot_command.commit().context_id();
    let component_id = removal.component_id();
    let component_key = (context_id.to_string(), component_id.to_string());
    if state.removed_components.contains(&component_key) {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        });
    }
    let component = state
        .components
        .get(&component_key)
        .or_else(|| {
            preview.components.iter().find(|component| {
                component.context_id == context_id.to_string()
                    && component.id == component_id.to_string()
            })
        })
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        })?;
    if component.kind.as_str()
        != crate::component_content_revision::component_kind_storage_value(removal.component_kind())
    {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component kind does not match the removal".to_owned(),
        });
    }
    let current_hash = state
        .component_content_hashes
        .get(&component_key)
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        })?;
    if current_hash != removal.previous_content_hash().as_str() {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component content hash is stale".to_owned(),
        });
    }

    Ok(removal)
}

fn prepare_component_descriptor_revision(
    preview: &ContextGraphProjection,
    state: &CommitSnapshotState,
    snapshot_command: &CreateContextCommitSnapshot,
    descriptor: ComponentDescriptorRevisionWrite,
) -> Result<crate::ContextComponentRecord, StorageRepositoryError> {
    let context_id = snapshot_command.commit().context_id();
    let component_id = descriptor.component_id();
    let component_key = (context_id.to_string(), component_id.to_string());
    if state.removed_components.contains(&component_key) {
        return Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        });
    }

    let existing = state
        .components
        .get(&component_key)
        .or_else(|| {
            preview.components.iter().find(|component| {
                component.context_id == context_id.to_string()
                    && component.id == component_id.to_string()
            })
        })
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        })?;
    if existing.kind
        != crate::StoredComponentKind::from_context_component_kind(descriptor.component_kind())
    {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component kind does not match the descriptor revision".to_owned(),
        });
    }
    let current_hash = state
        .component_content_hashes
        .get(&component_key)
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("component:{context_id}/{component_id}"),
        })?;
    if current_hash != descriptor.component().content_hash().as_str() {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component content hash is stale for the descriptor revision".to_owned(),
        });
    }

    Ok(crate::ContextComponentRecord {
        id: component_id.to_string(),
        context_id: context_id.to_string(),
        kind: existing.kind,
        name: descriptor.component().name().as_str().to_owned(),
        content_hash: current_hash.clone(),
        metadata: descriptor.metadata().clone(),
        created_at: existing.created_at,
        updated_at: descriptor.captured_at(),
    })
}

fn prepare_component_content_creation(
    preview: &ContextGraphProjection,
    state: &CommitSnapshotState,
    snapshot_command: &CreateContextCommitSnapshot,
    creation: crate::ComponentContentCreationWrite,
) -> Result<(crate::ContextComponentRecord, ComponentContentRevision), StorageRepositoryError> {
    let context_id = snapshot_command.commit().context_id();
    let component = creation.component();
    let component_id = component.id().to_string();
    let component_exists = preview
        .components
        .iter()
        .any(|stored| stored.context_id == context_id.to_string() && stored.id == component_id)
        || state
            .components
            .contains_key(&(context_id.to_string(), component_id.clone()));
    if component_exists {
        return Err(StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component already exists".to_owned(),
        });
    }

    let record = crate::ContextComponentRecord {
        id: component_id,
        context_id: context_id.to_string(),
        kind: crate::StoredComponentKind::from_context_component_kind(component.kind()),
        name: component.name().as_str().to_owned(),
        content_hash: creation.resulting_content_hash().as_str().to_owned(),
        metadata: creation.metadata().clone(),
        created_at: creation.captured_at(),
        updated_at: creation.captured_at(),
    };
    let revision = creation.into_revision(context_id, snapshot_command.commit().id());

    Ok((record, revision))
}

fn persist_commit_snapshot(
    preview: &ContextGraphProjection,
    state: &mut CommitSnapshotState,
    command: CreateContextCommitSnapshot,
) -> Result<CommitGraphSnapshot, StorageRepositoryError> {
    let diff_scope = VersionedContextScopeV1::new(
        command.snapshot().scope().project_id(),
        command.snapshot().scope().context_id(),
        command.snapshot().scope().commit_id(),
    );
    let diff_record = PersistContextDiffSnapshotV1::new(
        diff_scope,
        CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1,
        command.diff_snapshot().clone(),
        command.snapshot().captured_at(),
    )
    .map_err(|_| StorageRepositoryError::Database {
        message: "context diff snapshot command is invalid".to_owned(),
    })?
    .record()
    .clone();
    let diff_key = context_diff_snapshot_key(diff_record.scope(), diff_record.schema_version());
    let (commit, snapshot, _diff_snapshot) = command.into_parts();
    let context_id = commit.context_id().to_string();
    let commit_id = commit.id().to_string();
    let snapshot_scope = snapshot.scope();
    if snapshot_scope.context_id() != commit.context_id()
        || snapshot_scope.commit_id() != commit.id()
    {
        return Err(StorageRepositoryError::InvalidScope {
            scope: snapshot_scope.to_string(),
            reason: "snapshot scope must match the commit Context and identifier".to_owned(),
        });
    }
    ensure_commit_graph_snapshot_context_exists(preview, snapshot_scope)?;

    let key = (context_id.clone(), commit_id.clone());
    let snapshot_key = commit_graph_snapshot_key(snapshot_scope);
    let static_commit_exists = preview
        .commits
        .iter()
        .any(|stored| stored.context_id == context_id && stored.id == commit_id);
    if static_commit_exists
        || state.commits.contains_key(&key)
        || state.snapshots.contains_key(&snapshot_key)
    {
        return Err(StorageRepositoryError::CommitAlreadyExists {
            context_id,
            commit_id,
        });
    }
    if let Some(existing) = state.diff_snapshots.get(&diff_key) {
        if existing != &diff_record {
            return Err(StorageRepositoryError::Database {
                message: "context diff snapshot conflicts with the commit".to_owned(),
            });
        }
    }

    for parent_id in commit.parent_ids() {
        let parent_commit_id = parent_id.to_string();
        let static_parent_exists = preview
            .commits
            .iter()
            .any(|stored| stored.context_id == context_id && stored.id == parent_commit_id);
        let dynamic_parent_exists = state
            .commits
            .contains_key(&(context_id.clone(), parent_commit_id.clone()));
        if !static_parent_exists && !dynamic_parent_exists {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("parent_commit:{context_id}/{parent_commit_id}"),
            });
        }
        if !snapshot_exists_for_context_commit(state, &context_id, &parent_commit_id) {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("parent_commit:{context_id}/{parent_commit_id}"),
            });
        }
    }

    let changes =
        serde_json::to_value(commit.changes()).map_err(|_| StorageRepositoryError::Database {
            message: "commit changes serialization failed".to_owned(),
        })?;
    let record = crate::ContextCommitRecord {
        id: commit_id.clone(),
        context_id: context_id.clone(),
        branch_name: commit.branch().as_str().to_owned(),
        message: commit.message().to_owned(),
        parent_commit_ids: commit
            .parent_ids()
            .iter()
            .map(ToString::to_string)
            .collect(),
        change_count: u32::try_from(commit.changes().len()).unwrap_or(u32::MAX),
        changes,
        authored_at: commit.authored_at(),
        created_at: snapshot.captured_at(),
    };

    state.commits.insert(key.clone(), record);
    state.snapshots.insert(snapshot_key, snapshot.clone());
    state.diff_snapshots.insert(diff_key, diff_record);

    Ok(snapshot)
}

#[async_trait]
impl WorkspaceRepository for InMemoryContextGraphRepository {
    async fn list_workspaces(
        &self,
        query: WorkspaceListQuery,
    ) -> Result<WorkspaceList, StorageRepositoryError> {
        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let mut items = self
            .preview
            .workspaces
            .iter()
            .map(|workspace| WorkspaceListItem {
                id: workspace.id.clone(),
                name: workspace.name.clone(),
                slug: workspace.slug.clone(),
                created_at: workspace.created_at,
            })
            .filter(|workspace| {
                search.as_ref().is_none_or(|search| {
                    workspace.id.to_lowercase().contains(search)
                        || workspace.name.to_lowercase().contains(search)
                        || workspace.slug.to_lowercase().contains(search)
                })
            })
            .collect::<Vec<_>>();

        sort_workspaces(&mut items, query.sort);

        let total = items.len() as u64;
        let start = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let end = start
            .saturating_add(query.per_page as usize)
            .min(items.len());
        let items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(WorkspaceList::new(items, &query, total))
    }
}

#[async_trait]
impl ProjectRepository for InMemoryContextGraphRepository {
    async fn list_projects(
        &self,
        workspace_id: String,
        query: ProjectListQuery,
    ) -> Result<ProjectList, StorageRepositoryError> {
        ensure_workspace_exists(&self.preview, &workspace_id)?;

        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let mut items = self
            .preview
            .projects
            .iter()
            .filter(|project| project.workspace_id == workspace_id)
            .map(|project| ProjectListItem {
                id: project.id.clone(),
                workspace_id: project.workspace_id.clone(),
                name: project.name.clone(),
                slug: project.slug.clone(),
                created_at: project.created_at,
            })
            .filter(|project| {
                search.as_ref().is_none_or(|search| {
                    project.id.to_lowercase().contains(search)
                        || project.name.to_lowercase().contains(search)
                        || project.slug.to_lowercase().contains(search)
                })
            })
            .collect::<Vec<_>>();

        sort_projects(&mut items, query.sort);

        let total = items.len() as u64;
        let start = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let end = start
            .saturating_add(query.per_page as usize)
            .min(items.len());
        let items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(ProjectList::new(items, &query, total))
    }
}

#[async_trait]
impl ExperimentRepository for InMemoryContextGraphRepository {
    async fn list_experiments(
        &self,
        project_id: String,
        query: ExperimentListQuery,
    ) -> Result<ExperimentList, StorageRepositoryError> {
        ensure_project_exists(&self.preview, &project_id)?;

        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let mut items = self
            .preview
            .experiments
            .iter()
            .filter(|experiment| experiment.project_id == project_id)
            .map(|experiment| ExperimentListItem {
                id: experiment.id.clone(),
                project_id: experiment.project_id.clone(),
                name: experiment.name.clone(),
                branch_name: experiment.branch_name.clone(),
                created_at: experiment.created_at,
            })
            .filter(|experiment| {
                search.as_ref().is_none_or(|search| {
                    experiment.id.to_lowercase().contains(search)
                        || experiment.name.to_lowercase().contains(search)
                        || experiment.branch_name.to_lowercase().contains(search)
                })
            })
            .collect::<Vec<_>>();

        sort_experiments(&mut items, query.sort);

        let total = items.len() as u64;
        let start = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let end = start
            .saturating_add(query.per_page as usize)
            .min(items.len());
        let items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(ExperimentList::new(items, &query, total))
    }
}

#[async_trait]
impl ContextRepository for InMemoryContextGraphRepository {
    async fn list_contexts(
        &self,
        project_id: String,
        query: ContextListQuery,
    ) -> Result<ContextList, StorageRepositoryError> {
        ensure_project_exists(&self.preview, &project_id)?;

        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let mut items =
            self.preview
                .contexts
                .iter()
                .filter(|context| context.project_id == project_id)
                .filter(|context| {
                    query.experiment_id.as_ref().is_none_or(|experiment_id| {
                        context.experiment_id.as_ref() == Some(experiment_id)
                    })
                })
                .map(|context| ContextListItem {
                    id: context.id.clone(),
                    project_id: context.project_id.clone(),
                    experiment_id: context.experiment_id.clone(),
                    name: context.name.clone(),
                    description: context.description.clone(),
                    created_at: context.created_at,
                })
                .filter(|context| {
                    search.as_ref().is_none_or(|search| {
                        context.id.to_lowercase().contains(search)
                            || context.name.to_lowercase().contains(search)
                            || context.description.as_ref().is_some_and(|description| {
                                description.to_lowercase().contains(search)
                            })
                    })
                })
                .collect::<Vec<_>>();

        sort_contexts(&mut items, query.sort);

        let total = items.len() as u64;
        let start = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let end = start
            .saturating_add(query.per_page as usize)
            .min(items.len());
        let items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(ContextList::new(items, &query, total))
    }
}

#[async_trait]
impl ContextCommitRepository for InMemoryContextGraphRepository {
    async fn list_commits(
        &self,
        context_id: String,
        query: CommitListQuery,
    ) -> Result<CommitList, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id)?;

        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let mut items = self
            .preview
            .commits
            .iter()
            .filter(|commit| commit.context_id == context_id)
            .filter(|commit| {
                query
                    .branch_name
                    .as_ref()
                    .is_none_or(|branch_name| commit.branch_name == *branch_name)
            })
            .map(|commit| CommitListItem {
                id: commit.id.clone(),
                context_id: commit.context_id.clone(),
                branch_name: commit.branch_name.clone(),
                message: commit.message.clone(),
                parent_commit_ids: commit.parent_commit_ids.clone(),
                change_count: commit.change_count,
                authored_at: commit.authored_at,
                created_at: commit.created_at,
            })
            .filter(|commit| {
                search.as_ref().is_none_or(|search| {
                    commit.id.to_lowercase().contains(search)
                        || commit.message.to_lowercase().contains(search)
                        || commit.branch_name.to_lowercase().contains(search)
                })
            })
            .collect::<Vec<_>>();

        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        items.extend(
            state
                .commits
                .values()
                .filter(|commit| commit.context_id == context_id)
                .filter(|commit| {
                    query
                        .branch_name
                        .as_ref()
                        .is_none_or(|branch_name| commit.branch_name == *branch_name)
                })
                .map(|commit| CommitListItem {
                    id: commit.id.clone(),
                    context_id: commit.context_id.clone(),
                    branch_name: commit.branch_name.clone(),
                    message: commit.message.clone(),
                    parent_commit_ids: commit.parent_commit_ids.clone(),
                    change_count: commit.change_count,
                    authored_at: commit.authored_at,
                    created_at: commit.created_at,
                })
                .filter(|commit| {
                    search.as_ref().is_none_or(|search| {
                        commit.id.to_lowercase().contains(search)
                            || commit.message.to_lowercase().contains(search)
                            || commit.branch_name.to_lowercase().contains(search)
                    })
                }),
        );

        sort_commits(&mut items, query.sort);

        let total = items.len() as u64;
        let start = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let end = start
            .saturating_add(query.per_page as usize)
            .min(items.len());
        let items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(CommitList::new(items, &query, total))
    }

    async fn get_commit(
        &self,
        context_id: String,
        commit_id: String,
    ) -> Result<CommitDetail, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id)?;

        let static_commit = self
            .preview
            .commits
            .iter()
            .find(|commit| commit.context_id == context_id && commit.id == commit_id)
            .cloned();
        let dynamic_commit = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?
            .commits
            .get(&(context_id.clone(), commit_id.clone()))
            .cloned();
        let commit = static_commit.or(dynamic_commit).ok_or_else(|| {
            StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{commit_id}"),
            }
        })?;

        Ok(CommitDetail {
            id: commit.id,
            context_id: commit.context_id,
            branch_name: commit.branch_name,
            message: commit.message,
            parent_commit_ids: commit.parent_commit_ids,
            changes: commit.changes,
            change_count: commit.change_count,
            authored_at: commit.authored_at,
            created_at: commit.created_at,
        })
    }
}

#[async_trait]
impl ContextCommitGraphRepository for InMemoryContextGraphRepository {
    async fn load_context_commit_graph(
        &self,
        context_id: ContextId,
    ) -> Result<contextlab_versioning::CommitGraph, StorageRepositoryError> {
        let context_key = context_id.to_string();
        ensure_context_exists(&self.preview, &context_key)?;

        let mut records = self
            .preview
            .commits
            .iter()
            .filter(|commit| commit.context_id == context_key)
            .cloned()
            .collect::<Vec<_>>();
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        records.extend(
            state
                .commits
                .values()
                .filter(|commit| commit.context_id == context_key)
                .cloned(),
        );

        commit_graph_from_records(context_id, records)
    }
}

#[async_trait]
impl ContextComponentRepository for InMemoryContextGraphRepository {
    async fn list_components(
        &self,
        context_id: String,
        query: ComponentListQuery,
    ) -> Result<ComponentList, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id)?;
        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;

        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let mut components = self
            .preview
            .components
            .iter()
            .filter(|component| component.context_id == context_id)
            .map(|component| (component.id.clone(), component.clone()))
            .collect::<BTreeMap<_, _>>();
        for component in state
            .components
            .values()
            .filter(|component| component.context_id == context_id)
        {
            components.insert(component.id.clone(), component.clone());
        }
        let mut items = components
            .into_values()
            .filter(|component| {
                !state
                    .removed_components
                    .contains(&(component.context_id.clone(), component.id.clone()))
            })
            .filter(|component| query.kind.is_none_or(|kind| component.kind == kind))
            .map(|component| ComponentListItem {
                id: component.id.clone(),
                context_id: component.context_id.clone(),
                kind: component.kind,
                name: component.name.clone(),
                content_hash: state
                    .component_content_hashes
                    .get(&(component.context_id.clone(), component.id.clone()))
                    .cloned()
                    .unwrap_or_else(|| component.content_hash.clone()),
                created_at: component.created_at,
            })
            .filter(|component| {
                search.as_ref().is_none_or(|search| {
                    component.id.to_lowercase().contains(search)
                        || component.kind.as_str().contains(search)
                        || component.content_hash.to_lowercase().contains(search)
                        || component.name.to_lowercase().contains(search)
                })
            })
            .collect::<Vec<_>>();

        sort_components(&mut items, query.sort);

        let total = items.len() as u64;
        let start = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let end = start
            .saturating_add(query.per_page as usize)
            .min(items.len());
        let items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(ComponentList::new(items, &query, total))
    }

    async fn get_component(
        &self,
        context_id: String,
        component_id: String,
    ) -> Result<ComponentDetail, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id)?;

        let state = self
            .commit_snapshot_state
            .read()
            .map_err(|_| StorageRepositoryError::InMemoryStateUnavailable)?;
        if state
            .removed_components
            .contains(&(context_id.clone(), component_id.clone()))
        {
            return Err(StorageRepositoryError::ScopeUnavailable {
                scope: format!("component:{context_id}/{component_id}"),
            });
        }
        let component = state
            .components
            .get(&(context_id.clone(), component_id.clone()))
            .or_else(|| {
                self.preview.components.iter().find(|component| {
                    component.context_id == context_id && component.id == component_id
                })
            })
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("component:{context_id}/{component_id}"),
            })?;

        Ok(ComponentDetail {
            id: component.id.clone(),
            context_id: component.context_id.clone(),
            kind: component.kind,
            name: component.name.clone(),
            content_hash: state
                .component_content_hashes
                .get(&(component.context_id.clone(), component.id.clone()))
                .cloned()
                .unwrap_or_else(|| component.content_hash.clone()),
            metadata: component.metadata.clone(),
            created_at: component.created_at,
            updated_at: component.updated_at,
        })
    }
}

#[async_trait]
impl EvaluationRunRepository for InMemoryContextGraphRepository {
    async fn list_evaluation_runs(
        &self,
        context_id: String,
        query: EvaluationRunListQuery,
    ) -> Result<EvaluationRunList, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id)?;

        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let mut items = self
            .preview
            .evaluation_runs
            .iter()
            .filter(|run| run.context_id == context_id)
            .filter(|run| {
                query
                    .suite_name
                    .as_ref()
                    .is_none_or(|suite_name| run.suite_name == *suite_name)
            })
            .filter(|run| {
                query
                    .model_version
                    .as_ref()
                    .is_none_or(|model_version| run.model_version == *model_version)
            })
            .map(|run| EvaluationRunListItem {
                id: run.id.clone(),
                context_id: run.context_id.clone(),
                suite_name: run.suite_name.clone(),
                model_version: run.model_version.clone(),
                temperature: run.temperature,
                metric_count: run.metric_count,
                executed_at: run.executed_at,
                created_at: run.created_at,
            })
            .filter(|run| {
                search.as_ref().is_none_or(|search| {
                    run.id.to_lowercase().contains(search)
                        || run.suite_name.to_lowercase().contains(search)
                        || run.model_version.to_lowercase().contains(search)
                })
            })
            .collect::<Vec<_>>();

        sort_evaluation_runs(&mut items, query.sort);

        let total = items.len() as u64;
        let start = usize::try_from(query.offset()).unwrap_or(usize::MAX);
        let end = start
            .saturating_add(query.per_page as usize)
            .min(items.len());
        let items = if start < items.len() {
            items[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok(EvaluationRunList::new(items, &query, total))
    }

    async fn get_evaluation_run(
        &self,
        context_id: String,
        run_id: String,
    ) -> Result<EvaluationRunDetail, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id)?;

        let run = self
            .preview
            .evaluation_runs
            .iter()
            .find(|run| run.context_id == context_id && run.id == run_id)
            .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                scope: format!("evaluation_run:{context_id}/{run_id}"),
            })?;

        Ok(EvaluationRunDetail {
            id: run.id.clone(),
            context_id: run.context_id.clone(),
            suite_name: run.suite_name.clone(),
            model_version: run.model_version.clone(),
            temperature: run.temperature,
            metric_count: run.metric_count,
            metrics: run.metrics.clone(),
            executed_at: run.executed_at,
            created_at: run.created_at,
        })
    }

    async fn get_evaluation_scorecard(
        &self,
        context_id: String,
        query: EvaluationScorecardQuery,
    ) -> Result<EvaluationScorecard, StorageRepositoryError> {
        ensure_context_exists(&self.preview, &context_id)?;

        let search = query.search.as_ref().map(|value| value.to_lowercase());
        let runs = self
            .preview
            .evaluation_runs
            .iter()
            .filter(|run| run.context_id == context_id)
            .filter(|run| {
                query
                    .suite_name
                    .as_ref()
                    .is_none_or(|suite_name| run.suite_name == *suite_name)
            })
            .filter(|run| {
                query
                    .model_version
                    .as_ref()
                    .is_none_or(|model_version| run.model_version == *model_version)
            })
            .filter(|run| {
                search.as_ref().is_none_or(|search| {
                    run.id.to_lowercase().contains(search)
                        || run.suite_name.to_lowercase().contains(search)
                        || run.model_version.to_lowercase().contains(search)
                })
            })
            .collect::<Vec<_>>();

        Ok(build_evaluation_scorecard(
            context_id,
            runs.into_iter().map(|run| &run.metrics),
        ))
    }
}

fn build_evaluation_scorecard<'a>(
    context_id: String,
    metrics: impl Iterator<Item = &'a serde_json::Value>,
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

fn ensure_workspace_exists(
    projection: &ContextGraphProjection,
    workspace_id: &str,
) -> Result<(), StorageRepositoryError> {
    if projection
        .workspaces
        .iter()
        .any(|workspace| workspace.id == workspace_id)
    {
        Ok(())
    } else {
        Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("workspace:{workspace_id}"),
        })
    }
}

fn ensure_project_exists(
    projection: &ContextGraphProjection,
    project_id: &str,
) -> Result<(), StorageRepositoryError> {
    if projection
        .projects
        .iter()
        .any(|project| project.id == project_id)
    {
        Ok(())
    } else {
        Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("project:{project_id}"),
        })
    }
}

fn ensure_context_exists(
    projection: &ContextGraphProjection,
    context_id: &str,
) -> Result<(), StorageRepositoryError> {
    if projection
        .contexts
        .iter()
        .any(|context| context.id == context_id)
    {
        Ok(())
    } else {
        Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("context:{context_id}"),
        })
    }
}

fn project_id_for_context(
    projection: &ContextGraphProjection,
    context_id: ContextId,
) -> Result<ProjectId, StorageRepositoryError> {
    let context = projection
        .contexts
        .iter()
        .find(|context| context.id == context_id.to_string())
        .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
            scope: format!("context:{context_id}"),
        })?;
    let project_id = Uuid::parse_str(&context.project_id).map_err(|error| {
        StorageRepositoryError::InvalidScope {
            scope: format!("project:{}", context.project_id),
            reason: error.to_string(),
        }
    })?;
    let project_id = ProjectId::from_uuid(project_id);
    ensure_project_exists(projection, &project_id.to_string())?;
    Ok(project_id)
}

fn commit_graph_snapshot_key(scope: CommitGraphSnapshotScope) -> (String, String, String) {
    (
        scope.project_id().to_string(),
        scope.context_id().to_string(),
        scope.commit_id().to_string(),
    )
}

fn load_context_graph_review_intermediate_snapshots(
    state: &CommitSnapshotState,
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

    commit_ids
        .into_iter()
        .map(|commit_id| {
            let scope = CommitGraphSnapshotScope::new(
                source_scope.project_id(),
                source_scope.context_id(),
                commit_id,
            );
            state
                .snapshots
                .get(&commit_graph_snapshot_key(scope))
                .cloned()
                .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                    scope: scope.to_string(),
                })
        })
        .collect()
}

fn snapshot_exists_for_context_commit(
    state: &CommitSnapshotState,
    context_id: &str,
    commit_id: &str,
) -> bool {
    state
        .snapshots
        .keys()
        .any(|(_, stored_context_id, stored_commit_id)| {
            stored_context_id == context_id && stored_commit_id == commit_id
        })
}

fn context_diff_snapshot_key(
    scope: VersionedContextScopeV1,
    schema_version: &str,
) -> (Uuid, Uuid, Uuid, String) {
    (
        scope.project_id().as_uuid(),
        scope.context_id().as_uuid(),
        scope.commit_id().as_uuid(),
        schema_version.to_owned(),
    )
}

fn read_context_diff_snapshot_from_state(
    state: &CommitSnapshotState,
    scope: VersionedContextScopeV1,
) -> Result<ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError> {
    let key = context_diff_snapshot_key(scope, CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1);
    let record = state
        .diff_snapshots
        .get(&key)
        .cloned()
        .ok_or(ContextDiffSnapshotPersistenceError::NotFound { scope })?;
    let restored = crate::context_diff_snapshot::decode_context_diff_snapshot(
        scope,
        record.schema_version().to_owned(),
        serde_json::to_value(record.snapshot())
            .map_err(|_| ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid)?,
        record.snapshot_digest().to_owned(),
        record.captured_at(),
    )?;
    if restored != record {
        return Err(ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid);
    }
    Ok(restored)
}

fn snapshot_for_context_commit(
    state: &CommitSnapshotState,
    context_id: &str,
    commit_id: &str,
) -> Option<CommitGraphSnapshot> {
    state
        .snapshots
        .iter()
        .find_map(|((_, stored_context_id, stored_commit_id), snapshot)| {
            (stored_context_id == context_id && stored_commit_id == commit_id)
                .then(|| snapshot.clone())
        })
}

fn ensure_commit_graph_snapshot_context_exists(
    projection: &ContextGraphProjection,
    scope: CommitGraphSnapshotScope,
) -> Result<(), StorageRepositoryError> {
    let project_id = scope.project_id().to_string();
    let context_id = scope.context_id().to_string();
    ensure_project_exists(projection, &project_id)?;

    if projection
        .contexts
        .iter()
        .any(|context| context.id == context_id && context.project_id == project_id)
    {
        Ok(())
    } else {
        Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("project:{project_id}/context:{context_id}"),
        })
    }
}

fn ensure_commit_graph_snapshot_scope_exists(
    projection: &ContextGraphProjection,
    scope: CommitGraphSnapshotScope,
) -> Result<(), StorageRepositoryError> {
    ensure_commit_graph_snapshot_context_exists(projection, scope)?;

    let context_id = scope.context_id().to_string();
    let commit_id = scope.commit_id().to_string();
    if projection
        .commits
        .iter()
        .any(|commit| commit.context_id == context_id && commit.id == commit_id)
    {
        Ok(())
    } else {
        Err(StorageRepositoryError::ScopeUnavailable {
            scope: scope.to_string(),
        })
    }
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

fn sort_workspaces(items: &mut [WorkspaceListItem], sort: WorkspaceSort) {
    match sort {
        WorkspaceSort::NameAsc => {
            items.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        WorkspaceSort::NameDesc => {
            items.sort_by(|left, right| {
                right
                    .name
                    .cmp(&left.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        WorkspaceSort::CreatedAtAsc => {
            items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        WorkspaceSort::CreatedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
    }
}

fn sort_projects(items: &mut [ProjectListItem], sort: ProjectSort) {
    match sort {
        ProjectSort::NameAsc => {
            items.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ProjectSort::NameDesc => {
            items.sort_by(|left, right| {
                right
                    .name
                    .cmp(&left.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ProjectSort::CreatedAtAsc => {
            items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ProjectSort::CreatedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
    }
}

fn sort_experiments(items: &mut [ExperimentListItem], sort: ExperimentSort) {
    match sort {
        ExperimentSort::NameAsc => {
            items.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ExperimentSort::NameDesc => {
            items.sort_by(|left, right| {
                right
                    .name
                    .cmp(&left.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ExperimentSort::BranchNameAsc => {
            items.sort_by(|left, right| {
                left.branch_name
                    .cmp(&right.branch_name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ExperimentSort::BranchNameDesc => {
            items.sort_by(|left, right| {
                right
                    .branch_name
                    .cmp(&left.branch_name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ExperimentSort::CreatedAtAsc => {
            items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ExperimentSort::CreatedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
    }
}

fn sort_contexts(items: &mut [ContextListItem], sort: ContextSort) {
    match sort {
        ContextSort::NameAsc => {
            items.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ContextSort::NameDesc => {
            items.sort_by(|left, right| {
                right
                    .name
                    .cmp(&left.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ContextSort::CreatedAtAsc => {
            items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ContextSort::CreatedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
    }
}

fn sort_commits(items: &mut [CommitListItem], sort: CommitSort) {
    match sort {
        CommitSort::AuthoredAtAsc => {
            items.sort_by(|left, right| {
                left.authored_at
                    .cmp(&right.authored_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        CommitSort::AuthoredAtDesc => {
            items.sort_by(|left, right| {
                right
                    .authored_at
                    .cmp(&left.authored_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        CommitSort::CreatedAtAsc => {
            items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        CommitSort::CreatedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        CommitSort::BranchNameAsc => {
            items.sort_by(|left, right| {
                left.branch_name
                    .cmp(&right.branch_name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        CommitSort::BranchNameDesc => {
            items.sort_by(|left, right| {
                right
                    .branch_name
                    .cmp(&left.branch_name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
    }
}

fn sort_components(items: &mut [ComponentListItem], sort: ComponentSort) {
    match sort {
        ComponentSort::NameAsc => {
            items.sort_by(|left, right| {
                left.name
                    .cmp(&right.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ComponentSort::NameDesc => {
            items.sort_by(|left, right| {
                right
                    .name
                    .cmp(&left.name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ComponentSort::KindAsc => {
            items.sort_by(|left, right| {
                left.kind
                    .as_str()
                    .cmp(right.kind.as_str())
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ComponentSort::KindDesc => {
            items.sort_by(|left, right| {
                right
                    .kind
                    .as_str()
                    .cmp(left.kind.as_str())
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ComponentSort::CreatedAtAsc => {
            items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        ComponentSort::CreatedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
    }
}

fn sort_evaluation_runs(items: &mut [EvaluationRunListItem], sort: EvaluationRunSort) {
    match sort {
        EvaluationRunSort::ExecutedAtAsc => {
            items.sort_by(|left, right| {
                left.executed_at
                    .cmp(&right.executed_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        EvaluationRunSort::ExecutedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .executed_at
                    .cmp(&left.executed_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        EvaluationRunSort::CreatedAtAsc => {
            items.sort_by(|left, right| {
                left.created_at
                    .cmp(&right.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        EvaluationRunSort::CreatedAtDesc => {
            items.sort_by(|left, right| {
                right
                    .created_at
                    .cmp(&left.created_at)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        EvaluationRunSort::SuiteNameAsc => {
            items.sort_by(|left, right| {
                left.suite_name
                    .cmp(&right.suite_name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        EvaluationRunSort::SuiteNameDesc => {
            items.sort_by(|left, right| {
                right
                    .suite_name
                    .cmp(&left.suite_name)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        EvaluationRunSort::ModelVersionAsc => {
            items.sort_by(|left, right| {
                left.model_version
                    .cmp(&right.model_version)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
        EvaluationRunSort::ModelVersionDesc => {
            items.sort_by(|left, right| {
                right
                    .model_version
                    .cmp(&left.model_version)
                    .then_with(|| left.id.cmp(&right.id))
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CommitGraphSnapshot, ComponentContentCreationWrite, ComponentListQuery,
        ContextBranchRepository, ContextBranchRepositoryError, ContextCommitRecord,
        ContextComponentRecord, ContextGraphProjection, ContextRecord, EvaluationRunRecord,
        ExperimentRecord, GuardedCommitWriteDisposition, GuardedContextCommitWrite,
        GuardedContextCommitWriter, IdempotencyKey, ProjectRecord, RequestDigest,
        StoredComponentKind, WorkspaceRecord,
        component_content_revision::PersistedComponentContentRevision,
    };
    use chrono::{TimeZone, Utc};
    use contextlab_auth::{
        AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity,
    };
    use contextlab_context_core::{
        ComponentContent, ComponentId, ContentHash, ContextComponentKind, ContextId,
        ContextMetadata, ProjectId,
    };
    use contextlab_evaluation::{
        BenchmarkCase, BenchmarkDataset, BenchmarkEvaluation, BenchmarkExpectedOutput,
        BenchmarkSuite, EvaluationRun, MetricKind, MetricMeasurement, RegressionThreshold,
        ThresholdDirection,
    };
    use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
    use contextlab_versioning::{
        BranchName, CommitId, ContextChange, ContextCommit, ExpectedBranchHead,
    };
    use serde_json::json;

    #[tokio::test]
    async fn concrete_history_read_observes_dynamic_commit_and_branch_head_together() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let command = snapshot_command(context_id, Vec::new());
        let commit_id = command.commit().id();

        repository
            .create_guarded_commit_snapshot(guarded_command(
                command,
                ExpectedBranchHead::Unborn,
                "history-read",
                "history-read-digest",
            ))
            .await
            .expect("guarded commit");

        let history = repository
            .load_context_commit_history(context_id)
            .await
            .expect("consistent in-memory history");

        assert_eq!(history.context_id(), context_id);
        assert!(history.commit(commit_id).is_some());
        assert_eq!(history.head(&BranchName::default()), Some(commit_id));
    }

    #[tokio::test]
    async fn in_memory_benchmark_execution_idempotency_replays_and_conflicts() {
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let commit_id = CommitId::new();
        let timestamp = Utc::now();
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: project_id.to_string(),
                workspace_id: "workspace".to_owned(),
                name: "Benchmark project".to_owned(),
                slug: "benchmark-project".to_owned(),
                created_at: timestamp,
            }],
            contexts: vec![ContextRecord {
                id: context_id.to_string(),
                project_id: project_id.to_string(),
                experiment_id: None,
                name: "Benchmark context".to_owned(),
                description: None,
                created_at: timestamp,
            }],
            commits: vec![ContextCommitRecord {
                id: commit_id.to_string(),
                context_id: context_id.to_string(),
                branch_name: "main".to_owned(),
                message: "benchmark fixture".to_owned(),
                parent_commit_ids: Vec::new(),
                changes: json!([]),
                change_count: 0,
                authored_at: timestamp,
                created_at: timestamp,
            }],
            ..ContextGraphProjection::default()
        });
        let dataset = BenchmarkDataset::new(
            "Idempotency dataset",
            vec![
                BenchmarkCase::new(
                    "private case",
                    json!({"input": "private"}),
                    BenchmarkExpectedOutput::Exact(json!({"ok": true})),
                )
                .expect("case"),
            ],
        )
        .expect("dataset");
        let suite = BenchmarkSuite::new(
            "Idempotency suite",
            vec![dataset.id()],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("threshold"),
            ],
        )
        .expect("suite");
        let run = EvaluationRun::new(
            context_id,
            "model-v1",
            0.0,
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("metric")],
            timestamp,
        )
        .expect("run");
        let decision_id = crate::BenchmarkDecisionId::new();
        let command = PersistBenchmarkEvaluationEvidence::new(
            decision_id,
            project_id,
            commit_id,
            vec![dataset.clone()],
            suite.clone(),
            vec![run.clone()],
            BenchmarkEvaluation::from_runs(&suite, std::slice::from_ref(&run)),
            "contextlab.test-evaluator",
            "v1",
            timestamp,
        )
        .expect("command")
        .with_idempotency(
            IdempotencyKey::new("benchmark-execution-replay").expect("key"),
            RequestDigest::new("sha256:benchmark-execution-replay").expect("digest"),
        );

        let created = repository
            .persist_benchmark_evaluation(command.clone())
            .await
            .expect("create evidence");
        let replay = repository
            .persist_benchmark_evaluation(
                PersistBenchmarkEvaluationEvidence::new(
                    crate::BenchmarkDecisionId::new(),
                    project_id,
                    commit_id,
                    vec![dataset.clone()],
                    suite.clone(),
                    vec![run.clone()],
                    BenchmarkEvaluation::from_runs(&suite, std::slice::from_ref(&run)),
                    "contextlab.test-evaluator",
                    "v1",
                    timestamp,
                )
                .expect("replay command")
                .with_idempotency(
                    IdempotencyKey::new("benchmark-execution-replay").expect("key"),
                    RequestDigest::new("sha256:benchmark-execution-replay").expect("digest"),
                ),
            )
            .await
            .expect("replay evidence");

        assert_eq!(
            replay.disposition(),
            BenchmarkEvidenceWriteDisposition::Replayed
        );
        assert_eq!(replay.evidence(), created.evidence());
        assert_eq!(
            repository
                .get_benchmark_execution_idempotency(
                    project_id,
                    context_id,
                    commit_id,
                    &IdempotencyKey::new("benchmark-execution-replay").expect("key"),
                )
                .await
                .expect("read receipt")
                .expect("receipt")
                .decision_id(),
            decision_id
        );

        let conflict = repository
            .persist_benchmark_evaluation(
                PersistBenchmarkEvaluationEvidence::new(
                    crate::BenchmarkDecisionId::new(),
                    project_id,
                    commit_id,
                    vec![dataset],
                    suite.clone(),
                    vec![run.clone()],
                    BenchmarkEvaluation::from_runs(&suite, std::slice::from_ref(&run)),
                    "contextlab.test-evaluator",
                    "v1",
                    timestamp,
                )
                .expect("conflict command")
                .with_idempotency(
                    IdempotencyKey::new("benchmark-execution-replay").expect("key"),
                    RequestDigest::new("sha256:benchmark-execution-changed").expect("digest"),
                ),
            )
            .await
            .expect_err("same key with changed digest must conflict");
        assert!(matches!(
            conflict,
            StorageRepositoryError::IdempotencyKeyReused { .. }
        ));
    }

    #[tokio::test]
    async fn in_memory_repository_loads_preview_projection() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let projection = repository
            .load_context_graph_projection(GraphProjectionScope::Preview)
            .await
            .expect("projection");
        let graph = projection.project().expect("graph");

        assert!(
            graph
                .nodes()
                .values()
                .any(|node| node.kind() == GraphNodeKind::Context)
        );
    }

    #[tokio::test]
    async fn in_memory_replay_state_repository_matches_normal_parent_and_missing_commit_contract() {
        let context_id = ContextId::new();
        let root_commit_id = CommitId::new();
        let child_commit_id = CommitId::new();
        let metadata = ContextMetadata::new(timestamp(2));
        let record = |id: CommitId, parent_ids: Vec<CommitId>, changes: Vec<ContextChange>| {
            let changes = serde_json::to_value(changes).expect("changes");
            ContextCommitRecord {
                id: id.to_string(),
                context_id: context_id.to_string(),
                branch_name: "main".to_owned(),
                message: "Replay state fixture".to_owned(),
                parent_commit_ids: parent_ids
                    .into_iter()
                    .map(|parent_id| parent_id.to_string())
                    .collect(),
                change_count: changes.as_array().expect("array").len() as u32,
                changes,
                authored_at: timestamp(1),
                created_at: timestamp(1),
            }
        };
        let mut projection = projection_with_context(context_id);
        projection.commits = vec![
            record(
                root_commit_id,
                Vec::new(),
                vec![ContextChange::created_context("Replay state context")],
            ),
            record(
                child_commit_id,
                vec![root_commit_id],
                vec![ContextChange::updated_metadata(
                    metadata.clone(),
                    "metadata",
                )],
            ),
        ];
        let repository = InMemoryContextGraphRepository::new(projection);

        let replayed = ContextReplayStateAtCommitRepository::get_context_replay_state_at_commit(
            &repository,
            context_id,
            child_commit_id,
        )
        .await
        .expect("replay normal-parent history");
        assert_eq!(replayed.context_id(), context_id);
        assert_eq!(replayed.commit_id(), Some(child_commit_id));
        assert_eq!(
            replayed.context_metadata().expect("metadata").metadata(),
            &metadata
        );

        let missing_commit_id = CommitId::new();
        let error = ContextReplayStateAtCommitRepository::get_context_replay_state_at_commit(
            &repository,
            context_id,
            missing_commit_id,
        )
        .await
        .expect_err("missing commit should return a structured scope error");
        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: format!("commit:{context_id}/{missing_commit_id}"),
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_unknown_workspace_scope() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .load_context_graph_projection(GraphProjectionScope::Workspace {
                workspace_id: "missing".to_owned(),
            })
            .await
            .expect_err("missing workspace should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "workspace:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_branch_heads_preserve_unborn_rows_revision_and_typed_order() {
        let context_id = ContextId::new();
        let project_id = project_id_for_context(context_id);
        let repository = InMemoryContextGraphRepository::new(graph_snapshot_projection(
            project_id,
            context_id,
            [],
        ));
        let first_commit_id = CommitId::new();
        {
            let mut state = repository
                .commit_snapshot_state
                .write()
                .expect("write branch state");
            state
                .branch_heads
                .insert((context_id.to_string(), "feature/z".to_owned()), None);
            state
                .branch_revisions
                .insert((context_id.to_string(), "feature/z".to_owned()), 0);
            state.branch_heads.insert(
                (context_id.to_string(), "feature/a".to_owned()),
                Some(first_commit_id),
            );
            state
                .branch_revisions
                .insert((context_id.to_string(), "feature/a".to_owned()), 4);
            state.commits.insert(
                (context_id.to_string(), first_commit_id.to_string()),
                ContextCommitRecord {
                    id: first_commit_id.to_string(),
                    context_id: context_id.to_string(),
                    branch_name: "feature/a".to_owned(),
                    message: "branch-head fixture".to_owned(),
                    parent_commit_ids: Vec::new(),
                    changes: json!([]),
                    change_count: 0,
                    authored_at: timestamp(0),
                    created_at: timestamp(0),
                },
            );
        }

        let heads = repository
            .list_context_branch_heads(context_id)
            .await
            .expect("branch heads");

        assert_eq!(
            heads
                .iter()
                .map(|head| head.branch().as_str())
                .collect::<Vec<_>>(),
            vec!["feature/a", "feature/z"]
        );
        assert_eq!(heads[0].head_commit_id(), Some(first_commit_id));
        assert_eq!(heads[0].revision(), 4);
        assert_eq!(heads[1].head_commit_id(), None);
        assert_eq!(heads[1].revision(), 0);
    }

    #[tokio::test]
    async fn in_memory_exact_branch_read_ignores_unrelated_invalid_rows() {
        let context_id = ContextId::new();
        let project_id = project_id_for_context(context_id);
        let repository = InMemoryContextGraphRepository::new(graph_snapshot_projection(
            project_id,
            context_id,
            [],
        ));
        let main_commit_id = CommitId::new();
        {
            let mut state = repository
                .commit_snapshot_state
                .write()
                .expect("write branch state");
            state.branch_heads.insert(
                (context_id.to_string(), "main".to_owned()),
                Some(main_commit_id),
            );
            state
                .branch_revisions
                .insert((context_id.to_string(), "main".to_owned()), 2);
            state.branch_heads.insert(
                (context_id.to_string(), String::new()),
                Some(main_commit_id),
            );
            state
                .branch_revisions
                .insert((context_id.to_string(), String::new()), u64::MAX);
            state.commits.insert(
                (context_id.to_string(), main_commit_id.to_string()),
                ContextCommitRecord {
                    id: main_commit_id.to_string(),
                    context_id: context_id.to_string(),
                    branch_name: "main".to_owned(),
                    message: "exact branch fixture".to_owned(),
                    parent_commit_ids: Vec::new(),
                    changes: json!([]),
                    change_count: 0,
                    authored_at: timestamp(0),
                    created_at: timestamp(0),
                },
            );
        }

        let head = repository
            .get_context_branch_head(context_id, BranchName::new("main").expect("branch name"))
            .await
            .expect("exact branch head");

        assert_eq!(head.head_commit_id(), Some(main_commit_id));
        assert_eq!(head.revision(), 2);
    }

    #[tokio::test]
    async fn in_memory_exact_branch_read_rejects_cross_context_head_and_overflow() {
        let context_id = ContextId::new();
        let project_id = project_id_for_context(context_id);
        let repository = InMemoryContextGraphRepository::new(graph_snapshot_projection(
            project_id,
            context_id,
            [],
        ));
        let foreign_context_id = ContextId::new();
        let foreign_commit_id = CommitId::new();
        {
            let mut state = repository
                .commit_snapshot_state
                .write()
                .expect("write branch state");
            state.branch_heads.insert(
                (context_id.to_string(), "foreign".to_owned()),
                Some(foreign_commit_id),
            );
            state
                .branch_revisions
                .insert((context_id.to_string(), "foreign".to_owned()), 1);
            state.branch_heads.insert(
                (context_id.to_string(), "overflow".to_owned()),
                Some(foreign_commit_id),
            );
            state
                .branch_revisions
                .insert((context_id.to_string(), "overflow".to_owned()), u64::MAX);
            state.commits.insert(
                (
                    foreign_context_id.to_string(),
                    foreign_commit_id.to_string(),
                ),
                ContextCommitRecord {
                    id: foreign_commit_id.to_string(),
                    context_id: foreign_context_id.to_string(),
                    branch_name: "foreign".to_owned(),
                    message: "foreign branch fixture".to_owned(),
                    parent_commit_ids: Vec::new(),
                    changes: json!([]),
                    change_count: 0,
                    authored_at: timestamp(0),
                    created_at: timestamp(0),
                },
            );
        }

        let foreign_error = repository
            .get_context_branch_head(context_id, BranchName::new("foreign").expect("branch name"))
            .await
            .expect_err("cross-context head must fail closed");
        assert!(matches!(
            foreign_error,
            ContextBranchRepositoryError::BranchHeadIntegrityViolation { .. }
        ));

        let overflow_error = repository
            .get_context_branch_head(
                context_id,
                BranchName::new("overflow").expect("branch name"),
            )
            .await
            .expect_err("revision overflow must fail closed");
        assert!(matches!(
            overflow_error,
            ContextBranchRepositoryError::RevisionOverflow { revision, .. }
                if revision == u64::MAX
        ));
    }

    #[tokio::test]
    async fn in_memory_repository_reads_materialized_commit_graph_snapshots() {
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let materialized_commit_id = CommitId::new();
        let unmaterialized_commit_id = CommitId::new();
        let projection = graph_snapshot_projection(
            project_id,
            context_id,
            [materialized_commit_id, unmaterialized_commit_id],
        );
        let materialized_scope =
            CommitGraphSnapshotScope::new(project_id, context_id, materialized_commit_id);
        let unmaterialized_scope =
            CommitGraphSnapshotScope::new(project_id, context_id, unmaterialized_commit_id);
        let snapshot = CommitGraphSnapshot::new(
            materialized_scope,
            projection.project().expect("graph"),
            timestamp(1),
            1,
        )
        .expect("snapshot");
        let repository =
            InMemoryContextGraphRepository::with_commit_graph_snapshots(projection, [snapshot])
                .expect("repository");

        let materialized = repository
            .get_commit_graph_snapshot(materialized_scope)
            .await
            .expect("materialized snapshot")
            .expect("snapshot exists");
        let missing = repository
            .get_commit_graph_snapshot(unmaterialized_scope)
            .await
            .expect("known commit without snapshot");

        assert_eq!(materialized.schema_version(), 1);
        assert_eq!(materialized.scope(), materialized_scope);
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_unknown_commit_snapshot_scope() {
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(graph_snapshot_projection(
            project_id,
            context_id,
            std::iter::empty(),
        ));
        let scope = CommitGraphSnapshotScope::new(project_id, context_id, CommitId::new());

        let error = repository
            .get_commit_graph_snapshot(scope)
            .await
            .expect_err("unknown commit should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: scope.to_string()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_unknown_context_snapshot_scope() {
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(graph_snapshot_projection(
            project_id,
            context_id,
            std::iter::empty(),
        ));
        let missing_context_id = ContextId::new();

        let error = repository
            .get_commit_graph_snapshot(CommitGraphSnapshotScope::new(
                project_id,
                missing_context_id,
                CommitId::new(),
            ))
            .await
            .expect_err("unknown context should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{project_id}/context:{missing_context_id}")
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_cross_project_snapshot_scope() {
        let project_id = ProjectId::new();
        let other_project_id = ProjectId::new();
        let context_id = ContextId::new();
        let commit_id = CommitId::new();
        let mut projection = graph_snapshot_projection(project_id, context_id, [commit_id]);
        projection.projects.push(ProjectRecord {
            id: other_project_id.to_string(),
            workspace_id: "workspace".to_owned(),
            name: "Other project".to_owned(),
            slug: "other-project".to_owned(),
            created_at: timestamp(0),
        });
        let repository = InMemoryContextGraphRepository::new(projection);

        let error = repository
            .get_commit_graph_snapshot(CommitGraphSnapshotScope::new(
                other_project_id,
                context_id,
                commit_id,
            ))
            .await
            .expect_err("a Context cannot be read through another project");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: format!("project:{other_project_id}/context:{context_id}")
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_creates_commit_snapshot_atomically() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let command = snapshot_command(context_id, Vec::new());
        let commit_id = command.commit().id().to_string();
        let expected_diff_snapshot = command.diff_snapshot().clone();

        let snapshot = repository
            .create_commit_snapshot(command)
            .await
            .expect("persist snapshot");
        let persisted_snapshot = repository
            .get_commit_graph_snapshot(snapshot.scope())
            .await
            .expect("read snapshot")
            .expect("snapshot exists");
        let persisted_commit = repository
            .get_commit(context_id.to_string(), commit_id)
            .await
            .expect("read commit");

        assert_eq!(snapshot, persisted_snapshot);
        assert_eq!(persisted_commit.message, "Capture context graph");
        assert_eq!(persisted_commit.change_count, 1);

        let persisted_diff = repository
            .read_context_diff_snapshot(VersionedContextScopeV1::new(
                project_id_for_context(context_id),
                context_id,
                snapshot.commit_id(),
            ))
            .await
            .expect("read derived diff snapshot");
        assert_eq!(persisted_diff.snapshot(), &expected_diff_snapshot);
        assert_eq!(persisted_diff.captured_at(), snapshot.captured_at());
    }

    #[tokio::test]
    async fn in_memory_guarded_writer_replays_an_identical_idempotency_key_before_checking_head() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let snapshot_command = snapshot_command(context_id, Vec::new());
        let expected_snapshot = snapshot_command.snapshot().clone();
        let expected_diff_snapshot = snapshot_command.diff_snapshot().clone();

        let created = repository
            .create_guarded_commit_snapshot(guarded_command(
                snapshot_command.clone(),
                ExpectedBranchHead::Unborn,
                "request-001",
                "sha256:request-001",
            ))
            .await
            .expect("initial commit");
        let replayed = repository
            .create_guarded_commit_snapshot(guarded_command(
                snapshot_command,
                ExpectedBranchHead::Unborn,
                "request-001",
                "sha256:request-001",
            ))
            .await
            .expect("idempotent replay");

        assert_eq!(created.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(replayed.snapshot, expected_snapshot);

        let persisted_diff = repository
            .read_context_diff_snapshot(VersionedContextScopeV1::new(
                project_id_for_context(context_id),
                context_id,
                expected_snapshot.commit_id(),
            ))
            .await
            .expect("read derived diff snapshot");
        assert_eq!(persisted_diff.snapshot(), &expected_diff_snapshot);
        assert_eq!(
            persisted_diff.scope().commit_id(),
            expected_snapshot.commit_id()
        );

        let replayed_diff = repository
            .read_context_diff_snapshot(persisted_diff.scope())
            .await
            .expect("re-read derived diff snapshot");
        assert_eq!(replayed_diff, persisted_diff);
    }

    #[tokio::test]
    async fn in_memory_guarded_writer_scopes_idempotency_by_branch() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let main_snapshot = snapshot_command_on_branch(context_id, "main", Vec::new());
        let feature_snapshot = snapshot_command_on_branch(context_id, "feature", Vec::new());

        let main = repository
            .create_guarded_commit_snapshot(guarded_command(
                main_snapshot,
                ExpectedBranchHead::Unborn,
                "branch-scoped-key",
                "sha256:branch-scoped-request",
            ))
            .await
            .expect("create main branch commit");
        let feature = repository
            .create_guarded_commit_snapshot(guarded_command(
                feature_snapshot,
                ExpectedBranchHead::Unborn,
                "branch-scoped-key",
                "sha256:branch-scoped-request",
            ))
            .await
            .expect("create feature branch commit");

        assert_eq!(main.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(feature.disposition, GuardedCommitWriteDisposition::Created);
        assert_ne!(main.snapshot.commit_id(), feature.snapshot.commit_id());
    }

    #[tokio::test]
    async fn in_memory_guarded_writer_replays_without_rolling_back_a_later_branch_head() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let root_snapshot = snapshot_command(context_id, Vec::new());
        let root_commit_id = root_snapshot.commit().id();

        let root = repository
            .create_guarded_commit_snapshot(guarded_command(
                root_snapshot.clone(),
                ExpectedBranchHead::Unborn,
                "later-head-retry",
                "sha256:later-head-retry",
            ))
            .await
            .expect("create root commit");
        let child_snapshot = snapshot_command(context_id, vec![root_commit_id]);
        let child_commit_id = child_snapshot.commit().id();
        repository
            .create_guarded_commit_snapshot(guarded_command(
                child_snapshot,
                ExpectedBranchHead::Commit(root_commit_id),
                "later-head-child",
                "sha256:later-head-child",
            ))
            .await
            .expect("advance branch head");

        let replayed = repository
            .create_guarded_commit_snapshot(guarded_command(
                root_snapshot,
                ExpectedBranchHead::Unborn,
                "later-head-retry",
                "sha256:later-head-retry",
            ))
            .await
            .expect("replay root without checking the later head");

        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(replayed.snapshot.commit_id(), root.snapshot.commit_id());
        assert_eq!(
            repository
                .commit_snapshot_state
                .read()
                .expect("read branch state")
                .branch_heads
                .get(&(context_id.to_string(), "main".to_owned())),
            Some(&Some(child_commit_id))
        );
    }

    #[tokio::test]
    async fn guarded_component_content_update_replays_body_revision() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let previous_content_hash = ContentHash::new("sha256:previous").expect("previous hash");
        let content = ComponentContent::new("new prompt body");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Update prompt body",
            Vec::new(),
            vec![ContextChange::updated_component_content(
                component_id,
                ContextComponentKind::Prompt,
                previous_content_hash.clone(),
                content.content_hash(),
                "Update prompt body",
            )],
            timestamp(1),
        )
        .expect("commit");
        let snapshot = crate::CreateContextCommitSnapshot::new(
            project_id_for_context(context_id),
            commit,
            ContextGraph::new(),
            timestamp(2),
            1,
        )
        .expect("snapshot");
        let commit_id = snapshot.commit().id();
        let mut projection = projection_with_context(context_id);
        projection.components.push(ContextComponentRecord {
            id: component_id.to_string(),
            context_id: context_id.to_string(),
            kind: StoredComponentKind::Prompt,
            name: "Prompt".to_owned(),
            content_hash: previous_content_hash.as_str().to_owned(),
            metadata: json!({}),
            created_at: timestamp(0),
            updated_at: timestamp(0),
        });
        let repository = InMemoryContextGraphRepository::new(projection);
        let command = guarded_command(
            snapshot,
            ExpectedBranchHead::Unborn,
            "component-update-001",
            "sha256:component-update-001",
        )
        .with_component_content_revision(crate::ComponentContentRevisionWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            previous_content_hash,
            content.clone(),
            timestamp(2),
        ))
        .expect("revision matches the commit");

        let created = repository
            .create_guarded_commit_snapshot(command.clone())
            .await
            .expect("create revision");
        let replayed = repository
            .create_guarded_commit_snapshot(command)
            .await
            .expect("replay revision");
        let revision = repository
            .get_component_content_revision(context_id, commit_id, component_id)
            .await
            .expect("read revision")
            .expect("revision exists");
        let component = repository
            .get_component(context_id.to_string(), component_id.to_string())
            .await
            .expect("read updated component");
        let projection = repository
            .load_context_graph_projection(GraphProjectionScope::Preview)
            .await
            .expect("load projection");

        assert_eq!(created.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(revision.content(), &content);
        assert_eq!(component.content_hash, content.content_hash().as_str());
        let projected_component = projection
            .components
            .iter()
            .find(|component| component.id == component_id.to_string())
            .expect("preseeded component is projected");
        assert_eq!(
            projected_component.content_hash,
            content.content_hash().as_str()
        );
        assert_eq!(projected_component.updated_at, timestamp(2));
    }

    #[tokio::test]
    async fn guarded_component_content_creation_projects_the_latest_dynamic_body_revision() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let metadata = json!({"locale": "en", "visibility": "private"});
        let content = ComponentContent::new("initial prompt body");
        let captured_at = timestamp(2);
        let creation = ComponentContentCreationWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            "Initial Prompt",
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
                    "Writer test context",
                )
                .expect("context node"),
            )
            .expect("add context node");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{component_id}"),
                    GraphNodeKind::Prompt,
                    "Initial Prompt",
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
                .expect("context relationship"),
            )
            .expect("add context relationship");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create initial prompt",
            Vec::new(),
            vec![
                ContextChange::added_component_content_with_details(
                    component_id,
                    ContextComponentKind::Prompt,
                    "Initial Prompt",
                    metadata.clone(),
                    content.content_hash(),
                    "Create initial prompt",
                )
                .expect("replayable component change"),
            ],
            captured_at,
        )
        .expect("commit");
        let commit_id = commit.id();
        let snapshot = crate::CreateContextCommitSnapshot::new(
            project_id_for_context(context_id),
            commit,
            graph,
            captured_at,
            1,
        )
        .expect("snapshot");
        let command = guarded_command(
            snapshot,
            ExpectedBranchHead::Unborn,
            "component-creation-001",
            "sha256:component-creation-001",
        )
        .with_component_content_creation(creation)
        .expect("matching creation");
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));

        let created = repository
            .create_guarded_commit_snapshot(command.clone())
            .await
            .expect("create component");
        let replayed = repository
            .create_guarded_commit_snapshot(command)
            .await
            .expect("replay component creation");
        let revised_content = ComponentContent::new("revised prompt body");
        let revised_at = timestamp(3);
        let revision_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Revise initial prompt",
            vec![commit_id],
            vec![ContextChange::updated_component_content(
                component_id,
                ContextComponentKind::Prompt,
                content.content_hash(),
                revised_content.content_hash(),
                "Revise initial prompt",
            )],
            revised_at,
        )
        .expect("revision commit");
        let revision_commit_id = revision_commit.id();
        let revision_snapshot = crate::CreateContextCommitSnapshot::new(
            project_id_for_context(context_id),
            revision_commit,
            ContextGraph::new(),
            revised_at,
            1,
        )
        .expect("revision snapshot");
        repository
            .create_guarded_commit_snapshot(
                guarded_command(
                    revision_snapshot,
                    ExpectedBranchHead::Commit(commit_id),
                    "component-revision-001",
                    "sha256:component-revision-001",
                )
                .with_component_content_revision(crate::ComponentContentRevisionWrite::new(
                    component_id,
                    ContextComponentKind::Prompt,
                    content.content_hash(),
                    revised_content.clone(),
                    revised_at,
                ))
                .expect("matching component revision"),
            )
            .await
            .expect("revise dynamic component");
        let revision = repository
            .get_component_content_revision(context_id, commit_id, component_id)
            .await
            .expect("read revision")
            .expect("initial revision exists");
        let component = repository
            .get_component(context_id.to_string(), component_id.to_string())
            .await
            .expect("read component");
        let list = repository
            .list_components(context_id.to_string(), ComponentListQuery::default())
            .await
            .expect("list components");
        let projection = repository
            .load_context_graph_projection(GraphProjectionScope::Preview)
            .await
            .expect("load projection");

        assert_eq!(created.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(revision.previous_content_hash(), None);
        assert_eq!(revision.content(), &content);
        assert_eq!(component.name, "Initial Prompt");
        assert_eq!(component.metadata, metadata);
        assert_eq!(
            component.content_hash,
            revised_content.content_hash().as_str()
        );
        assert_eq!(list.pagination.total, 1);
        assert_eq!(list.items[0].id, component_id.to_string());
        let projected_component = projection
            .components
            .iter()
            .find(|component| component.id == component_id.to_string())
            .expect("dynamic component is projected");
        assert_eq!(
            projected_component.content_hash,
            revised_content.content_hash().as_str()
        );
        assert_eq!(projected_component.updated_at, revised_at);

        let initial_state = repository
            .get_component_state_at_commit(context_id, commit_id, component_id)
            .await
            .expect("replay initial component state")
            .expect("initial component state is replayable");
        let revised_state = repository
            .get_component_state_at_commit(context_id, revision_commit_id, component_id)
            .await
            .expect("replay revised component state")
            .expect("revised component state is replayable");

        assert_eq!(initial_state.component().id(), component_id);
        assert_eq!(
            initial_state.component().kind(),
            ContextComponentKind::Prompt
        );
        assert_eq!(initial_state.component().name().as_str(), "Initial Prompt");
        assert_eq!(initial_state.metadata(), &metadata);
        assert_eq!(
            initial_state.component().content_hash(),
            &content.content_hash()
        );
        assert_eq!(initial_state.creation_commit_id(), commit_id);
        assert_eq!(initial_state.content_commit_id(), commit_id);
        assert_eq!(revised_state.component().name().as_str(), "Initial Prompt");
        assert_eq!(revised_state.metadata(), &metadata);
        assert_eq!(
            revised_state.component().content_hash(),
            &revised_content.content_hash()
        );
        assert_eq!(revised_state.creation_commit_id(), commit_id);
        assert_eq!(revised_state.content_commit_id(), revision_commit_id);

        let initial_context_snapshot = repository
            .get_context_component_state_snapshot_at_commit(context_id, commit_id)
            .await
            .expect("replay initial Context component inventory");
        let context_snapshot = repository
            .get_context_component_state_snapshot_at_commit(context_id, revision_commit_id)
            .await
            .expect("replay Context component inventory");
        assert_eq!(initial_context_snapshot.components().len(), 1);
        assert_eq!(
            initial_context_snapshot.components()[0]
                .component()
                .content_hash(),
            &content.content_hash()
        );
        assert_eq!(context_snapshot.context_id(), context_id);
        assert_eq!(context_snapshot.target_commit_id(), revision_commit_id);
        assert_eq!(context_snapshot.components().len(), 1);
        assert_eq!(
            context_snapshot.components()[0].component().content_hash(),
            &revised_content.content_hash()
        );

        let stale_removal_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Reject stale prompt removal",
            vec![revision_commit_id],
            vec![ContextChange::removed_component(
                component_id,
                ContextComponentKind::Prompt,
                ContentHash::new("sha256:stale-removal").expect("stale hash"),
                "Reject stale prompt removal",
            )],
            timestamp(4),
        )
        .expect("stale removal commit");
        let stale_removal_commit_id = stale_removal_commit.id();
        let stale_removal_snapshot = crate::CreateContextCommitSnapshot::new(
            project_id_for_context(context_id),
            stale_removal_commit,
            ContextGraph::new(),
            timestamp(4),
            1,
        )
        .expect("stale removal snapshot");
        let stale_removal_error = repository
            .create_guarded_commit_snapshot(
                guarded_command(
                    stale_removal_snapshot,
                    ExpectedBranchHead::Commit(revision_commit_id),
                    "component-removal-stale-001",
                    "sha256:component-removal-stale-001",
                )
                .with_component_removal(crate::ComponentRemovalWrite::new(
                    component_id,
                    ContextComponentKind::Prompt,
                    ContentHash::new("sha256:stale-removal").expect("stale hash"),
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

        let removal_at = timestamp(4);
        let removal_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Remove initial prompt",
            vec![revision_commit_id],
            vec![ContextChange::removed_component(
                component_id,
                ContextComponentKind::Prompt,
                revised_content.content_hash(),
                "Remove initial prompt",
            )],
            removal_at,
        )
        .expect("removal commit");
        let removal_commit_id = removal_commit.id();
        let removal_snapshot = crate::CreateContextCommitSnapshot::new(
            project_id_for_context(context_id),
            removal_commit,
            ContextGraph::new(),
            removal_at,
            1,
        )
        .expect("removal snapshot");
        let removal_command = guarded_command(
            removal_snapshot,
            ExpectedBranchHead::Commit(revision_commit_id),
            "component-removal-001",
            "sha256:component-removal-001",
        )
        .with_component_removal(crate::ComponentRemovalWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            revised_content.content_hash(),
        ))
        .expect("matching component removal");
        let removed = repository
            .create_guarded_commit_snapshot(removal_command.clone())
            .await
            .expect("remove dynamic component");
        let removal_replayed = repository
            .create_guarded_commit_snapshot(removal_command)
            .await
            .expect("replay component removal");
        let removed_list = repository
            .list_components(context_id.to_string(), ComponentListQuery::default())
            .await
            .expect("list after removal");
        let removed_snapshot = repository
            .get_context_component_state_snapshot_at_commit(context_id, removal_commit_id)
            .await
            .expect("replay removal inventory");

        assert_eq!(removed.disposition, GuardedCommitWriteDisposition::Created);
        assert_eq!(
            removal_replayed.disposition,
            GuardedCommitWriteDisposition::Replayed
        );
        assert!(removed_list.items.is_empty());
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
    async fn in_memory_component_content_replays_the_nearest_normal_parent_revision() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let root_commit_id = CommitId::new();
        let unchanged_commit_id = CommitId::new();
        let revised_commit_id = CommitId::new();
        let merge_commit_id = CommitId::new();
        let post_merge_revision_commit_id = CommitId::new();
        let cycle_a_commit_id = CommitId::new();
        let cycle_b_commit_id = CommitId::new();
        let initial_content = ComponentContent::new("initial replay body");
        let revised_content = ComponentContent::new("revised replay body");
        let post_merge_content = ComponentContent::new("post-merge replay body");
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let commit_record =
            |commit_id: CommitId, parent_commit_ids: Vec<CommitId>| ContextCommitRecord {
                id: commit_id.to_string(),
                context_id: context_id.to_string(),
                branch_name: "main".to_owned(),
                message: "Replay test commit".to_owned(),
                parent_commit_ids: parent_commit_ids
                    .into_iter()
                    .map(|parent_id| parent_id.to_string())
                    .collect(),
                changes: json!([]),
                change_count: 0,
                authored_at: timestamp(1),
                created_at: timestamp(1),
            };
        let initial_revision =
            ComponentContentRevision::from_persisted(PersistedComponentContentRevision {
                context_id,
                commit_id: root_commit_id,
                component_id,
                component_kind: ContextComponentKind::Prompt,
                previous_content_hash: None,
                content: initial_content.clone(),
                resulting_content_hash: initial_content.content_hash(),
                captured_at: timestamp(1),
            });
        let revised_revision =
            ComponentContentRevision::from_persisted(PersistedComponentContentRevision {
                context_id,
                commit_id: revised_commit_id,
                component_id,
                component_kind: ContextComponentKind::Prompt,
                previous_content_hash: Some(initial_content.content_hash()),
                content: revised_content.clone(),
                resulting_content_hash: revised_content.content_hash(),
                captured_at: timestamp(3),
            });
        let post_merge_revision =
            ComponentContentRevision::from_persisted(PersistedComponentContentRevision {
                context_id,
                commit_id: post_merge_revision_commit_id,
                component_id,
                component_kind: ContextComponentKind::Prompt,
                previous_content_hash: Some(revised_content.content_hash()),
                content: post_merge_content.clone(),
                resulting_content_hash: post_merge_content.content_hash(),
                captured_at: timestamp(4),
            });
        {
            let mut state = repository.commit_snapshot_state.write().expect("state");
            state.commits.insert(
                (context_id.to_string(), root_commit_id.to_string()),
                commit_record(root_commit_id, Vec::new()),
            );
            state.commits.insert(
                (context_id.to_string(), unchanged_commit_id.to_string()),
                commit_record(unchanged_commit_id, vec![root_commit_id]),
            );
            state.commits.insert(
                (context_id.to_string(), revised_commit_id.to_string()),
                commit_record(revised_commit_id, vec![unchanged_commit_id]),
            );
            state.commits.insert(
                (context_id.to_string(), merge_commit_id.to_string()),
                commit_record(merge_commit_id, vec![revised_commit_id, root_commit_id]),
            );
            state.commits.insert(
                (
                    context_id.to_string(),
                    post_merge_revision_commit_id.to_string(),
                ),
                commit_record(post_merge_revision_commit_id, vec![merge_commit_id]),
            );
            state.commits.insert(
                (context_id.to_string(), cycle_a_commit_id.to_string()),
                commit_record(cycle_a_commit_id, vec![cycle_b_commit_id]),
            );
            state.commits.insert(
                (context_id.to_string(), cycle_b_commit_id.to_string()),
                commit_record(cycle_b_commit_id, vec![cycle_a_commit_id]),
            );
            state.component_content_revisions.insert(
                (
                    context_id.to_string(),
                    root_commit_id.to_string(),
                    component_id.to_string(),
                ),
                initial_revision,
            );
            state.component_content_revisions.insert(
                (
                    context_id.to_string(),
                    revised_commit_id.to_string(),
                    component_id.to_string(),
                ),
                revised_revision,
            );
            state.component_content_revisions.insert(
                (
                    context_id.to_string(),
                    post_merge_revision_commit_id.to_string(),
                    component_id.to_string(),
                ),
                post_merge_revision,
            );
        }

        let unchanged = repository
            .get_component_content_at_commit(context_id, unchanged_commit_id, component_id)
            .await
            .expect("resolve unchanged child")
            .expect("initial body is reachable");
        let revised = repository
            .get_component_content_at_commit(context_id, revised_commit_id, component_id)
            .await
            .expect("resolve revised child")
            .expect("revised body is reachable");
        assert_eq!(unchanged.commit_id(), root_commit_id);
        assert_eq!(unchanged.content(), &initial_content);
        assert_eq!(revised.commit_id(), revised_commit_id);
        assert_eq!(revised.content(), &revised_content);
        assert_eq!(
            repository
                .get_component_state_at_commit(context_id, unchanged_commit_id, component_id)
                .await
                .expect("resolve state without replayable creation"),
            None
        );
        assert_eq!(
            repository
                .get_component_content_at_commit(
                    context_id,
                    unchanged_commit_id,
                    ComponentId::new(),
                )
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
        let unknown_context_id = ContextId::new();
        let expected_unknown_context_scope = format!("context:{unknown_context_id}");
        let unknown_context_error = repository
            .get_component_content_at_commit(unknown_context_id, root_commit_id, component_id)
            .await
            .expect_err("unknown context must be distinguishable from an unknown commit");
        assert!(matches!(
            unknown_context_error,
            StorageRepositoryError::ScopeUnavailable { scope }
                if scope == expected_unknown_context_scope
        ));
        assert!(matches!(
            repository
                .get_component_content_at_commit(context_id, merge_commit_id, component_id)
                .await,
            Err(StorageRepositoryError::ComponentContentRevisionConflict { .. })
        ));
        assert!(matches!(
            repository
                .get_component_state_at_commit(context_id, merge_commit_id, component_id)
                .await,
            Err(StorageRepositoryError::ComponentStateReplayConflict { .. })
        ));
        assert!(matches!(
            repository
                .get_component_content_at_commit(
                    context_id,
                    post_merge_revision_commit_id,
                    component_id,
                )
                .await,
            Err(StorageRepositoryError::ComponentContentRevisionConflict { .. })
        ));
        assert!(matches!(
            repository
                .get_component_content_at_commit(context_id, cycle_a_commit_id, component_id)
                .await,
            Err(StorageRepositoryError::ComponentContentRevisionConflict { .. })
        ));
    }

    #[tokio::test]
    async fn in_memory_guarded_writer_rejects_a_stale_branch_head_for_a_new_request() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let first_command = snapshot_command(context_id, Vec::new());
        let first_commit_id = first_command.commit().id();

        repository
            .create_guarded_commit_snapshot(guarded_command(
                first_command,
                ExpectedBranchHead::Unborn,
                "request-001",
                "sha256:request-001",
            ))
            .await
            .expect("initial commit");

        let stale_command = snapshot_command(context_id, vec![first_commit_id]);
        let error = repository
            .create_guarded_commit_snapshot(guarded_command(
                stale_command,
                ExpectedBranchHead::Unborn,
                "request-002",
                "sha256:request-002",
            ))
            .await
            .expect_err("new writes must compare their expected branch head");

        assert!(matches!(
            error,
            StorageRepositoryError::BranchHeadConflict { .. }
        ));
    }

    #[tokio::test]
    async fn in_memory_guarded_writer_rejects_a_reused_key_with_a_different_digest() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let snapshot_command = snapshot_command(context_id, Vec::new());

        repository
            .create_guarded_commit_snapshot(guarded_command(
                snapshot_command.clone(),
                ExpectedBranchHead::Unborn,
                "request-001",
                "sha256:request-001",
            ))
            .await
            .expect("initial commit");

        let error = repository
            .create_guarded_commit_snapshot(guarded_command(
                snapshot_command,
                ExpectedBranchHead::Unborn,
                "request-001",
                "sha256:different-request",
            ))
            .await
            .expect_err("reusing a key for another payload must fail");

        assert!(matches!(
            error,
            StorageRepositoryError::IdempotencyKeyReused { .. }
        ));
    }

    #[tokio::test]
    async fn in_memory_idempotency_does_not_replay_across_identity_sources() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let first = snapshot_command(context_id, Vec::new());

        repository
            .create_guarded_commit_snapshot(guarded_command_for_identity_source(
                "https://id.contextlab.test/tenant-a",
                first,
                ExpectedBranchHead::Unborn,
                "request-shared",
                "sha256:request-shared",
            ))
            .await
            .expect("first identity creates the commit");

        let second = snapshot_command(context_id, Vec::new());
        let error = repository
            .create_guarded_commit_snapshot(guarded_command_for_identity_source(
                "https://id.contextlab.test/tenant-b",
                second,
                ExpectedBranchHead::Unborn,
                "request-shared",
                "sha256:request-shared",
            ))
            .await
            .expect_err("second identity must not replay the first identity's request");

        assert!(matches!(
            error,
            StorageRepositoryError::BranchHeadConflict { .. }
        ));
    }

    #[tokio::test]
    async fn in_memory_writer_rejects_unknown_parent_without_partial_commit() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let command = snapshot_command(context_id, vec![CommitId::new()]);
        let commit_id = command.commit().id().to_string();

        let error = repository
            .create_commit_snapshot(command)
            .await
            .expect_err("unknown parent should fail");

        assert!(matches!(
            error,
            StorageRepositoryError::ScopeUnavailable { .. }
        ));
        assert!(matches!(
            repository
                .get_commit(context_id.to_string(), commit_id)
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));
    }

    #[tokio::test]
    async fn in_memory_writer_rejects_parent_without_materialized_graph_snapshot() {
        let context_id = ContextId::new();
        let parent_commit_id = CommitId::new();
        let mut projection = projection_with_context(context_id);
        projection.commits.push(ContextCommitRecord {
            id: parent_commit_id.to_string(),
            context_id: context_id.to_string(),
            branch_name: "main".to_owned(),
            message: "Parent commit without graph snapshot".to_owned(),
            parent_commit_ids: Vec::new(),
            changes: json!([]),
            change_count: 0,
            authored_at: timestamp(1),
            created_at: timestamp(1),
        });
        let repository = InMemoryContextGraphRepository::new(projection);
        let command = snapshot_command(context_id, vec![parent_commit_id]);
        let child_commit_id = command.commit().id().to_string();

        let error = repository
            .create_commit_snapshot(command)
            .await
            .expect_err("a parent without a graph snapshot must fail closed");

        assert!(matches!(
            error,
            StorageRepositoryError::ScopeUnavailable { ref scope }
                if scope == &format!("parent_commit:{context_id}/{parent_commit_id}")
        ));
        assert!(matches!(
            repository
                .get_commit(context_id.to_string(), child_commit_id)
                .await,
            Err(StorageRepositoryError::ScopeUnavailable { .. })
        ));
    }

    #[tokio::test]
    async fn in_memory_writer_rejects_repeated_command_without_replacing_snapshot() {
        let context_id = ContextId::new();
        let repository = InMemoryContextGraphRepository::new(projection_with_context(context_id));
        let command = snapshot_command(context_id, Vec::new());
        let expected_snapshot = command.snapshot().clone();

        repository
            .create_commit_snapshot(command.clone())
            .await
            .expect("first create succeeds");
        let error = repository
            .create_commit_snapshot(command)
            .await
            .expect_err("second create should fail");
        let stored_snapshot = repository
            .get_commit_graph_snapshot(expected_snapshot.scope())
            .await
            .expect("snapshot query")
            .expect("snapshot remains");

        assert_eq!(
            error,
            StorageRepositoryError::CommitAlreadyExists {
                context_id: context_id.to_string(),
                commit_id: expected_snapshot.commit_id().to_string(),
            }
        );
        assert_eq!(stored_snapshot, expected_snapshot);
    }

    #[tokio::test]
    async fn in_memory_writer_does_not_overwrite_a_preseeded_snapshot_key() {
        let context_id = ContextId::new();
        let command = snapshot_command(context_id, Vec::new());
        let expected_snapshot = command.snapshot().clone();
        let mut projection = projection_with_context(context_id);
        projection.commits.push(ContextCommitRecord {
            id: expected_snapshot.commit_id().to_string(),
            context_id: context_id.to_string(),
            branch_name: "main".to_owned(),
            message: "Preseeded snapshot fixture".to_owned(),
            parent_commit_ids: Vec::new(),
            changes: json!([]),
            change_count: 0,
            authored_at: timestamp(1),
            created_at: timestamp(1),
        });
        let repository = InMemoryContextGraphRepository::with_commit_graph_snapshots(
            projection,
            [expected_snapshot.clone()],
        )
        .expect("repository");

        let error = repository
            .create_commit_snapshot(command)
            .await
            .expect_err("preseeded snapshot key must remain immutable");
        let stored_snapshot = repository
            .get_commit_graph_snapshot(expected_snapshot.scope())
            .await
            .expect("the durable commit remains readable")
            .expect("the existing snapshot remains readable");

        assert_eq!(
            error,
            StorageRepositoryError::CommitAlreadyExists {
                context_id: context_id.to_string(),
                commit_id: expected_snapshot.commit_id().to_string(),
            }
        );
        assert_eq!(stored_snapshot, expected_snapshot);
    }

    #[test]
    fn in_memory_repository_rejects_duplicate_commit_graph_snapshot_scopes() {
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let commit_id = CommitId::new();
        let projection = graph_snapshot_projection(project_id, context_id, [commit_id]);
        let graph = projection.project().expect("graph");
        let scope = CommitGraphSnapshotScope::new(project_id, context_id, commit_id);
        let first = CommitGraphSnapshot::new(scope, graph.clone(), timestamp(1), 1)
            .expect("first snapshot");
        let duplicate =
            CommitGraphSnapshot::new(scope, graph, timestamp(2), 1).expect("duplicate snapshot");

        assert!(matches!(
            InMemoryContextGraphRepository::with_commit_graph_snapshots(
                projection,
                [first, duplicate]
            ),
            Err(StorageRepositoryError::CommitAlreadyExists { .. })
        ));
    }

    #[test]
    fn in_memory_repository_rejects_unreachable_preseeded_snapshot_scope() {
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let unknown_commit_id = CommitId::new();
        let projection = graph_snapshot_projection(project_id, context_id, std::iter::empty());
        let scope = CommitGraphSnapshotScope::new(project_id, context_id, unknown_commit_id);
        let snapshot = CommitGraphSnapshot::new(scope, ContextGraph::new(), timestamp(1), 1)
            .expect("snapshot");

        assert_eq!(
            InMemoryContextGraphRepository::with_commit_graph_snapshots(projection, [snapshot])
                .expect_err("unreachable snapshots must not be retained"),
            StorageRepositoryError::ScopeUnavailable {
                scope: scope.to_string()
            }
        );
    }

    #[test]
    fn in_memory_snapshot_replay_is_deterministic_and_conflicts_are_immutable() {
        let scope =
            CommitGraphSnapshotScope::new(ProjectId::new(), ContextId::new(), CommitId::new());
        let original = CommitGraphSnapshot::new(scope, ContextGraph::new(), timestamp(1), 1)
            .expect("original snapshot");
        let replay = original.clone();
        let conflict = CommitGraphSnapshot::new(scope, ContextGraph::new(), timestamp(2), 1)
            .expect("conflicting snapshot");

        assert_eq!(
            original.replay_outcome(&replay),
            crate::commit_graph_snapshot::CommitGraphSnapshotReplay::Replayed
        );
        assert_eq!(
            original.replay_outcome(&conflict),
            crate::commit_graph_snapshot::CommitGraphSnapshotReplay::Conflict
        );
    }

    #[tokio::test]
    async fn in_memory_repository_lists_workspaces_with_search_and_pagination() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let result = repository
            .list_workspaces(WorkspaceListQuery::new(
                Some(1),
                Some(1),
                Some("default".to_owned()),
                WorkspaceSort::NameAsc,
            ))
            .await
            .expect("workspace list");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].slug, "default");
        assert_eq!(result.items[0].created_at, timestamp(0));
        assert_eq!(result.pagination.total, 1);
    }

    #[tokio::test]
    async fn in_memory_repository_lists_real_workspace_slugs() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![WorkspaceRecord {
                id: "workspace-001".to_owned(),
                name: "Alpha Workspace".to_owned(),
                slug: "alpha-lab".to_owned(),
                created_at: timestamp(10),
            }],
            ..ContextGraphProjection::default()
        });

        let result = repository
            .list_workspaces(WorkspaceListQuery::default())
            .await
            .expect("workspace list");

        assert_eq!(result.items[0].id, "workspace-001");
        assert_eq!(result.items[0].slug, "alpha-lab");
        assert_ne!(result.items[0].id, result.items[0].slug);
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_workspaces_by_created_at() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![
                WorkspaceRecord {
                    id: "newer".to_owned(),
                    name: "Alpha Workspace".to_owned(),
                    slug: "alpha".to_owned(),
                    created_at: timestamp(20),
                },
                WorkspaceRecord {
                    id: "older".to_owned(),
                    name: "Zulu Workspace".to_owned(),
                    slug: "zulu".to_owned(),
                    created_at: timestamp(10),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_workspaces(WorkspaceListQuery::new(
                Some(1),
                Some(20),
                None,
                WorkspaceSort::CreatedAtAsc,
            ))
            .await
            .expect("ascending list");
        let descending = repository
            .list_workspaces(WorkspaceListQuery::new(
                Some(1),
                Some(20),
                None,
                WorkspaceSort::CreatedAtDesc,
            ))
            .await
            .expect("descending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|workspace| workspace.id.as_str())
                .collect::<Vec<_>>(),
            vec!["older", "newer"]
        );
        assert_eq!(
            descending
                .items
                .iter()
                .map(|workspace| workspace.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
    }

    #[tokio::test]
    async fn in_memory_repository_lists_projects_with_search_and_pagination() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let result = repository
            .list_projects(
                "default".to_owned(),
                ProjectListQuery::new(
                    Some(1),
                    Some(1),
                    Some("support".to_owned()),
                    ProjectSort::NameAsc,
                ),
            )
            .await
            .expect("project list");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, "support-ai");
        assert_eq!(result.items[0].workspace_id, "default");
        assert_eq!(result.items[0].slug, "support-ai");
        assert_eq!(result.items[0].created_at, timestamp(0));
        assert_eq!(result.pagination.total, 1);
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_project_list_for_unknown_workspace() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .list_projects("missing".to_owned(), ProjectListQuery::default())
            .await
            .expect_err("missing workspace should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "workspace:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_lists_real_project_slugs() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![ProjectRecord {
                id: "project-001".to_owned(),
                workspace_id: "workspace-001".to_owned(),
                name: "Alpha Project".to_owned(),
                slug: "alpha-lab".to_owned(),
                created_at: timestamp(10),
            }],
            ..ContextGraphProjection::default()
        });

        let result = repository
            .list_projects("workspace-001".to_owned(), ProjectListQuery::default())
            .await
            .expect("project list");

        assert_eq!(result.items[0].id, "project-001");
        assert_eq!(result.items[0].slug, "alpha-lab");
        assert_ne!(result.items[0].id, result.items[0].slug);
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_projects_by_created_at() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![
                ProjectRecord {
                    id: "newer".to_owned(),
                    workspace_id: "workspace-001".to_owned(),
                    name: "Alpha Project".to_owned(),
                    slug: "alpha".to_owned(),
                    created_at: timestamp(20),
                },
                ProjectRecord {
                    id: "older".to_owned(),
                    workspace_id: "workspace-001".to_owned(),
                    name: "Zulu Project".to_owned(),
                    slug: "zulu".to_owned(),
                    created_at: timestamp(10),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_projects(
                "workspace-001".to_owned(),
                ProjectListQuery::new(Some(1), Some(20), None, ProjectSort::CreatedAtAsc),
            )
            .await
            .expect("ascending list");
        let descending = repository
            .list_projects(
                "workspace-001".to_owned(),
                ProjectListQuery::new(Some(1), Some(20), None, ProjectSort::CreatedAtDesc),
            )
            .await
            .expect("descending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|project| project.id.as_str())
                .collect::<Vec<_>>(),
            vec!["older", "newer"]
        );
        assert_eq!(
            descending
                .items
                .iter()
                .map(|project| project.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
    }

    #[tokio::test]
    async fn in_memory_repository_lists_experiments_with_search_and_pagination() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let result = repository
            .list_experiments(
                "support-ai".to_owned(),
                ExperimentListQuery::new(
                    Some(1),
                    Some(1),
                    Some("rag".to_owned()),
                    ExperimentSort::NameAsc,
                ),
            )
            .await
            .expect("experiment list");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, "rag-v2");
        assert_eq!(result.items[0].project_id, "support-ai");
        assert_eq!(result.items[0].branch_name, "experiment/rag-v2");
        assert_eq!(result.items[0].created_at, timestamp(0));
        assert_eq!(result.pagination.total, 1);
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_experiment_list_for_unknown_project() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .list_experiments("missing".to_owned(), ExperimentListQuery::default())
            .await
            .expect_err("missing project should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "project:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_experiments_by_created_at() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![project("project-001", "workspace-001")],
            experiments: vec![
                ExperimentRecord {
                    id: "newer".to_owned(),
                    project_id: "project-001".to_owned(),
                    name: "Alpha Experiment".to_owned(),
                    branch_name: "experiment/alpha".to_owned(),
                    created_at: timestamp(20),
                },
                ExperimentRecord {
                    id: "older".to_owned(),
                    project_id: "project-001".to_owned(),
                    name: "Zulu Experiment".to_owned(),
                    branch_name: "experiment/zulu".to_owned(),
                    created_at: timestamp(10),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_experiments(
                "project-001".to_owned(),
                ExperimentListQuery::new(Some(1), Some(20), None, ExperimentSort::CreatedAtAsc),
            )
            .await
            .expect("ascending list");
        let descending = repository
            .list_experiments(
                "project-001".to_owned(),
                ExperimentListQuery::new(Some(1), Some(20), None, ExperimentSort::CreatedAtDesc),
            )
            .await
            .expect("descending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|experiment| experiment.id.as_str())
                .collect::<Vec<_>>(),
            vec!["older", "newer"]
        );
        assert_eq!(
            descending
                .items
                .iter()
                .map(|experiment| experiment.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_experiments_by_branch_name() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![project("project-001", "workspace-001")],
            experiments: vec![
                ExperimentRecord {
                    id: "b".to_owned(),
                    project_id: "project-001".to_owned(),
                    name: "Beta Experiment".to_owned(),
                    branch_name: "experiment/beta".to_owned(),
                    created_at: timestamp(10),
                },
                ExperimentRecord {
                    id: "a".to_owned(),
                    project_id: "project-001".to_owned(),
                    name: "Alpha Experiment".to_owned(),
                    branch_name: "experiment/alpha".to_owned(),
                    created_at: timestamp(20),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_experiments(
                "project-001".to_owned(),
                ExperimentListQuery::new(Some(1), Some(20), None, ExperimentSort::BranchNameAsc),
            )
            .await
            .expect("ascending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|experiment| experiment.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b"]
        );
    }

    #[tokio::test]
    async fn in_memory_repository_lists_contexts_with_search_and_pagination() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let result = repository
            .list_contexts(
                "support-ai".to_owned(),
                ContextListQuery::new(
                    Some(1),
                    Some(1),
                    Some("customer cases".to_owned()),
                    None,
                    ContextSort::NameAsc,
                ),
            )
            .await
            .expect("context list");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, "support-resolution-agent");
        assert_eq!(result.items[0].project_id, "support-ai");
        assert_eq!(result.items[0].experiment_id.as_deref(), Some("rag-v2"));
        assert_eq!(
            result.items[0].description.as_deref(),
            Some("Production support context for resolving customer cases.")
        );
        assert_eq!(result.items[0].created_at, timestamp(0));
        assert_eq!(result.pagination.total, 1);
    }

    #[tokio::test]
    async fn in_memory_repository_filters_contexts_by_experiment_id() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();

        let matching = repository
            .list_contexts(
                "support-ai".to_owned(),
                ContextListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some("rag-v2".to_owned()),
                    ContextSort::NameAsc,
                ),
            )
            .await
            .expect("matching context list");
        let missing = repository
            .list_contexts(
                "support-ai".to_owned(),
                ContextListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some("other".to_owned()),
                    ContextSort::NameAsc,
                ),
            )
            .await
            .expect("missing context list");

        assert_eq!(matching.pagination.total, 1);
        assert!(missing.items.is_empty());
        assert_eq!(missing.pagination.total, 0);
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_context_list_for_unknown_project() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .list_contexts("missing".to_owned(), ContextListQuery::default())
            .await
            .expect_err("missing project should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "project:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_contexts_by_created_at() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![project("project-001", "workspace-001")],
            contexts: vec![
                ContextRecord {
                    id: "newer".to_owned(),
                    project_id: "project-001".to_owned(),
                    experiment_id: None,
                    name: "Alpha Context".to_owned(),
                    description: Some("Newer context".to_owned()),
                    created_at: timestamp(20),
                },
                ContextRecord {
                    id: "older".to_owned(),
                    project_id: "project-001".to_owned(),
                    experiment_id: None,
                    name: "Zulu Context".to_owned(),
                    description: Some("Older context".to_owned()),
                    created_at: timestamp(10),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_contexts(
                "project-001".to_owned(),
                ContextListQuery::new(Some(1), Some(20), None, None, ContextSort::CreatedAtAsc),
            )
            .await
            .expect("ascending list");
        let descending = repository
            .list_contexts(
                "project-001".to_owned(),
                ContextListQuery::new(Some(1), Some(20), None, None, ContextSort::CreatedAtDesc),
            )
            .await
            .expect("descending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|context| context.id.as_str())
                .collect::<Vec<_>>(),
            vec!["older", "newer"]
        );
        assert_eq!(
            descending
                .items
                .iter()
                .map(|context| context.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
    }

    #[tokio::test]
    async fn in_memory_repository_lists_components_with_search_and_pagination() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let result = repository
            .list_components(
                "support-resolution-agent".to_owned(),
                ComponentListQuery::new(
                    Some(1),
                    Some(1),
                    Some("policy".to_owned()),
                    None,
                    ComponentSort::NameAsc,
                ),
            )
            .await
            .expect("component list");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, "refund-policy");
        assert_eq!(result.items[0].context_id, "support-resolution-agent");
        assert_eq!(result.items[0].kind, StoredComponentKind::Knowledge);
        assert_eq!(result.items[0].name, "Refund Policy Knowledge");
        assert_eq!(result.items[0].content_hash, "sha256:preview-refund-policy");
        assert_eq!(result.items[0].created_at, timestamp(0));
        assert_eq!(result.pagination.total, 1);
    }

    #[tokio::test]
    async fn in_memory_repository_filters_components_by_kind() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();

        let matching = repository
            .list_components(
                "support-resolution-agent".to_owned(),
                ComponentListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some(StoredComponentKind::Memory),
                    ComponentSort::KindAsc,
                ),
            )
            .await
            .expect("matching component list");
        let missing = repository
            .list_components(
                "support-resolution-agent".to_owned(),
                ComponentListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some(StoredComponentKind::Conversation),
                    ComponentSort::KindAsc,
                ),
            )
            .await
            .expect("missing component list");

        assert_eq!(matching.pagination.total, 1);
        assert_eq!(matching.items[0].id, "timeline");
        assert!(missing.items.is_empty());
        assert_eq!(missing.pagination.total, 0);
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_component_list_for_unknown_context() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .list_components("missing".to_owned(), ComponentListQuery::default())
            .await
            .expect_err("missing context should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "context:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_gets_component_detail() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let detail = repository
            .get_component(
                "support-resolution-agent".to_owned(),
                "refund-policy".to_owned(),
            )
            .await
            .expect("component detail");

        assert_eq!(detail.id, "refund-policy");
        assert_eq!(detail.context_id, "support-resolution-agent");
        assert_eq!(detail.kind, StoredComponentKind::Knowledge);
        assert_eq!(detail.name, "Refund Policy Knowledge");
        assert_eq!(detail.content_hash, "sha256:preview-refund-policy");
        assert_eq!(detail.metadata["source"], "policy-handbook");
        assert_eq!(detail.created_at, timestamp(0));
        assert_eq!(detail.updated_at, timestamp(0));
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_component_detail_for_missing_component() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .get_component("support-resolution-agent".to_owned(), "missing".to_owned())
            .await
            .expect_err("missing component should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "component:support-resolution-agent/missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_component_detail_for_missing_context() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .get_component("missing".to_owned(), "refund-policy".to_owned())
            .await
            .expect_err("missing context should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "context:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_components_by_created_at() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![project("project-001", "workspace-001")],
            contexts: vec![ContextRecord {
                id: "context-001".to_owned(),
                project_id: "project-001".to_owned(),
                experiment_id: None,
                name: "Context".to_owned(),
                description: None,
                created_at: timestamp(0),
            }],
            components: vec![
                ContextComponentRecord {
                    id: "newer".to_owned(),
                    context_id: "context-001".to_owned(),
                    kind: StoredComponentKind::Prompt,
                    name: "Alpha Prompt".to_owned(),
                    content_hash: "sha256:newer".to_owned(),
                    metadata: json!({ "order": "newer" }),
                    created_at: timestamp(20),
                    updated_at: timestamp(21),
                },
                ContextComponentRecord {
                    id: "older".to_owned(),
                    context_id: "context-001".to_owned(),
                    kind: StoredComponentKind::Knowledge,
                    name: "Zulu Knowledge".to_owned(),
                    content_hash: "sha256:older".to_owned(),
                    metadata: json!({ "order": "older" }),
                    created_at: timestamp(10),
                    updated_at: timestamp(11),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_components(
                "context-001".to_owned(),
                ComponentListQuery::new(Some(1), Some(20), None, None, ComponentSort::CreatedAtAsc),
            )
            .await
            .expect("ascending list");
        let descending = repository
            .list_components(
                "context-001".to_owned(),
                ComponentListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    None,
                    ComponentSort::CreatedAtDesc,
                ),
            )
            .await
            .expect("descending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|component| component.id.as_str())
                .collect::<Vec<_>>(),
            vec!["older", "newer"]
        );
        assert_eq!(
            descending
                .items
                .iter()
                .map(|component| component.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
    }

    #[tokio::test]
    async fn in_memory_repository_lists_commits_with_search_and_pagination() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let result = repository
            .list_commits(
                "support-resolution-agent".to_owned(),
                CommitListQuery::new(
                    Some(1),
                    Some(1),
                    Some("support resolution".to_owned()),
                    None,
                    CommitSort::AuthoredAtDesc,
                ),
            )
            .await
            .expect("commit list");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, "support-resolution-agent-initial");
        assert_eq!(result.items[0].context_id, "support-resolution-agent");
        assert_eq!(result.items[0].branch_name, "main");
        assert_eq!(result.items[0].message, "Create support resolution context");
        assert!(result.items[0].parent_commit_ids.is_empty());
        assert_eq!(result.items[0].change_count, 1);
        assert_eq!(result.items[0].authored_at, timestamp(0));
        assert_eq!(result.pagination.total, 1);
    }

    #[tokio::test]
    async fn in_memory_repository_filters_commits_by_branch_name() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();

        let matching = repository
            .list_commits(
                "support-resolution-agent".to_owned(),
                CommitListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some("main".to_owned()),
                    CommitSort::AuthoredAtDesc,
                ),
            )
            .await
            .expect("matching commit list");
        let missing = repository
            .list_commits(
                "support-resolution-agent".to_owned(),
                CommitListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some("experiment/rag-v2".to_owned()),
                    CommitSort::AuthoredAtDesc,
                ),
            )
            .await
            .expect("missing commit list");

        assert_eq!(matching.pagination.total, 1);
        assert!(missing.items.is_empty());
        assert_eq!(missing.pagination.total, 0);
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_commit_list_for_unknown_context() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .list_commits("missing".to_owned(), CommitListQuery::default())
            .await
            .expect_err("missing context should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "context:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_commits_by_authored_at() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![project("project-001", "workspace-001")],
            contexts: vec![ContextRecord {
                id: "context-001".to_owned(),
                project_id: "project-001".to_owned(),
                experiment_id: None,
                name: "Context".to_owned(),
                description: None,
                created_at: timestamp(0),
            }],
            commits: vec![
                ContextCommitRecord {
                    id: "newer".to_owned(),
                    context_id: "context-001".to_owned(),
                    branch_name: "main".to_owned(),
                    message: "Newer commit".to_owned(),
                    parent_commit_ids: vec!["older".to_owned()],
                    changes: serde_json::json!([
                        {
                            "operation": "update_component",
                            "path": "/components/system-contract"
                        },
                        {
                            "operation": "update_component",
                            "path": "/components/refund-policy"
                        }
                    ]),
                    change_count: 2,
                    authored_at: timestamp(20),
                    created_at: timestamp(21),
                },
                ContextCommitRecord {
                    id: "older".to_owned(),
                    context_id: "context-001".to_owned(),
                    branch_name: "main".to_owned(),
                    message: "Older commit".to_owned(),
                    parent_commit_ids: Vec::new(),
                    changes: serde_json::json!([
                        {
                            "operation": "create_context",
                            "path": "/contexts/context-001"
                        }
                    ]),
                    change_count: 1,
                    authored_at: timestamp(10),
                    created_at: timestamp(11),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_commits(
                "context-001".to_owned(),
                CommitListQuery::new(Some(1), Some(20), None, None, CommitSort::AuthoredAtAsc),
            )
            .await
            .expect("ascending list");
        let descending = repository
            .list_commits(
                "context-001".to_owned(),
                CommitListQuery::new(Some(1), Some(20), None, None, CommitSort::AuthoredAtDesc),
            )
            .await
            .expect("descending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|commit| commit.id.as_str())
                .collect::<Vec<_>>(),
            vec!["older", "newer"]
        );
        assert_eq!(
            descending
                .items
                .iter()
                .map(|commit| commit.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
        assert_eq!(
            descending.items[0].parent_commit_ids,
            vec!["older".to_owned()]
        );
        assert_eq!(descending.items[0].change_count, 2);
    }

    #[tokio::test]
    async fn in_memory_repository_lists_evaluation_runs_with_search_and_pagination() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let result = repository
            .list_evaluation_runs(
                "support-resolution-agent".to_owned(),
                EvaluationRunListQuery::new(
                    Some(1),
                    Some(1),
                    Some("safety".to_owned()),
                    None,
                    None,
                    EvaluationRunSort::ExecutedAtDesc,
                ),
            )
            .await
            .expect("evaluation run list");

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].id, "safety-regression");
        assert_eq!(result.items[0].context_id, "support-resolution-agent");
        assert_eq!(result.items[0].suite_name, "Safety Regression Suite");
        assert_eq!(result.items[0].model_version, "deepseek-chat");
        assert_eq!(result.items[0].temperature, 0.2);
        assert_eq!(result.items[0].metric_count, 2);
        assert_eq!(result.items[0].executed_at, timestamp(0));
        assert_eq!(result.pagination.total, 1);
    }

    #[tokio::test]
    async fn in_memory_repository_filters_evaluation_runs_by_suite_and_model() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();

        let matching = repository
            .list_evaluation_runs(
                "support-resolution-agent".to_owned(),
                EvaluationRunListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some("Safety Regression Suite".to_owned()),
                    Some("deepseek-chat".to_owned()),
                    EvaluationRunSort::ExecutedAtDesc,
                ),
            )
            .await
            .expect("matching evaluation run list");
        let missing = repository
            .list_evaluation_runs(
                "support-resolution-agent".to_owned(),
                EvaluationRunListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    Some("Safety Regression Suite".to_owned()),
                    Some("other-model".to_owned()),
                    EvaluationRunSort::ExecutedAtDesc,
                ),
            )
            .await
            .expect("missing evaluation run list");

        assert_eq!(matching.pagination.total, 1);
        assert!(missing.items.is_empty());
        assert_eq!(missing.pagination.total, 0);
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_evaluation_run_list_for_unknown_context() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .list_evaluation_runs("missing".to_owned(), EvaluationRunListQuery::default())
            .await
            .expect_err("missing context should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "context:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_gets_evaluation_run_detail() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let detail = repository
            .get_evaluation_run(
                "support-resolution-agent".to_owned(),
                "safety-regression".to_owned(),
            )
            .await
            .expect("evaluation run detail");

        assert_eq!(detail.id, "safety-regression");
        assert_eq!(detail.context_id, "support-resolution-agent");
        assert_eq!(detail.suite_name, "Safety Regression Suite");
        assert_eq!(detail.model_version, "deepseek-chat");
        assert_eq!(detail.temperature, 0.2);
        assert_eq!(detail.metric_count, 2);
        assert_eq!(detail.metrics["accuracy"], 0.92);
        assert_eq!(detail.metrics["latency_ms"], 820);
        assert_eq!(detail.executed_at, timestamp(0));
        assert_eq!(detail.created_at, timestamp(0));
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_evaluation_run_detail_for_missing_run() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .get_evaluation_run("support-resolution-agent".to_owned(), "missing".to_owned())
            .await
            .expect_err("missing evaluation run should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "evaluation_run:support-resolution-agent/missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_rejects_evaluation_run_detail_for_missing_context() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let error = repository
            .get_evaluation_run("missing".to_owned(), "safety-regression".to_owned())
            .await
            .expect_err("missing context should fail");

        assert_eq!(
            error,
            StorageRepositoryError::ScopeUnavailable {
                scope: "context:missing".to_owned()
            }
        );
    }

    #[tokio::test]
    async fn in_memory_repository_sorts_evaluation_runs_by_executed_at() {
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            workspaces: vec![workspace("workspace-001")],
            projects: vec![project("project-001", "workspace-001")],
            contexts: vec![ContextRecord {
                id: "context-001".to_owned(),
                project_id: "project-001".to_owned(),
                experiment_id: None,
                name: "Context".to_owned(),
                description: None,
                created_at: timestamp(0),
            }],
            evaluation_runs: vec![
                EvaluationRunRecord {
                    id: "newer".to_owned(),
                    context_id: "context-001".to_owned(),
                    suite_name: "Regression".to_owned(),
                    model_version: "model-b".to_owned(),
                    temperature: 0.4,
                    metric_count: 3,
                    metrics: json!({
                        "accuracy": 0.88,
                        "latency_ms": 900,
                        "cost_usd": 0.03
                    }),
                    executed_at: timestamp(20),
                    created_at: timestamp(21),
                },
                EvaluationRunRecord {
                    id: "older".to_owned(),
                    context_id: "context-001".to_owned(),
                    suite_name: "Baseline".to_owned(),
                    model_version: "model-a".to_owned(),
                    temperature: 0.2,
                    metric_count: 2,
                    metrics: json!({
                        "accuracy": 0.82,
                        "latency_ms": 1040
                    }),
                    executed_at: timestamp(10),
                    created_at: timestamp(11),
                },
            ],
            ..ContextGraphProjection::default()
        });

        let ascending = repository
            .list_evaluation_runs(
                "context-001".to_owned(),
                EvaluationRunListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    None,
                    None,
                    EvaluationRunSort::ExecutedAtAsc,
                ),
            )
            .await
            .expect("ascending list");
        let descending = repository
            .list_evaluation_runs(
                "context-001".to_owned(),
                EvaluationRunListQuery::new(
                    Some(1),
                    Some(20),
                    None,
                    None,
                    None,
                    EvaluationRunSort::ExecutedAtDesc,
                ),
            )
            .await
            .expect("descending list");

        assert_eq!(
            ascending
                .items
                .iter()
                .map(|run| run.id.as_str())
                .collect::<Vec<_>>(),
            vec!["older", "newer"]
        );
        assert_eq!(
            descending
                .items
                .iter()
                .map(|run| run.id.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
        assert_eq!(descending.items[0].metric_count, 3);
    }

    fn workspace(id: &str) -> WorkspaceRecord {
        WorkspaceRecord {
            id: id.to_owned(),
            name: format!("{id} Workspace"),
            slug: id.to_owned(),
            created_at: timestamp(0),
        }
    }

    fn project(id: &str, workspace_id: &str) -> ProjectRecord {
        ProjectRecord {
            id: id.to_owned(),
            workspace_id: workspace_id.to_owned(),
            name: format!("{id} Project"),
            slug: id.to_owned(),
            created_at: timestamp(0),
        }
    }

    fn projection_with_context(context_id: ContextId) -> ContextGraphProjection {
        let project_id = project_id_for_context(context_id);
        ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: project_id.to_string(),
                workspace_id: "workspace".to_owned(),
                name: "Writer test project".to_owned(),
                slug: "writer-test-project".to_owned(),
                created_at: timestamp(0),
            }],
            contexts: vec![ContextRecord {
                id: context_id.to_string(),
                project_id: project_id.to_string(),
                experiment_id: None,
                name: "Writer test context".to_owned(),
                description: None,
                created_at: timestamp(0),
            }],
            ..ContextGraphProjection::default()
        }
    }

    fn project_id_for_context(context_id: ContextId) -> ProjectId {
        ProjectId::from_uuid(context_id.as_uuid())
    }

    fn graph_snapshot_projection(
        project_id: ProjectId,
        context_id: ContextId,
        commit_ids: impl IntoIterator<Item = CommitId>,
    ) -> ContextGraphProjection {
        let captured_at = timestamp(0);
        ContextGraphProjection {
            workspaces: vec![WorkspaceRecord {
                id: "workspace".to_owned(),
                name: "Snapshot workspace".to_owned(),
                slug: "snapshot-workspace".to_owned(),
                created_at: captured_at,
            }],
            projects: vec![ProjectRecord {
                id: project_id.to_string(),
                workspace_id: "workspace".to_owned(),
                name: "Snapshot project".to_owned(),
                slug: "snapshot-project".to_owned(),
                created_at: captured_at,
            }],
            contexts: vec![ContextRecord {
                id: context_id.to_string(),
                project_id: project_id.to_string(),
                experiment_id: None,
                name: "Snapshot context".to_owned(),
                description: None,
                created_at: captured_at,
            }],
            commits: commit_ids
                .into_iter()
                .map(|commit_id| ContextCommitRecord {
                    id: commit_id.to_string(),
                    context_id: context_id.to_string(),
                    branch_name: "main".to_owned(),
                    message: "Snapshot fixture".to_owned(),
                    parent_commit_ids: Vec::new(),
                    changes: json!([]),
                    change_count: 0,
                    authored_at: captured_at,
                    created_at: captured_at,
                })
                .collect(),
            ..ContextGraphProjection::default()
        }
    }

    fn snapshot_command(
        context_id: ContextId,
        parent_ids: Vec<CommitId>,
    ) -> crate::CreateContextCommitSnapshot {
        snapshot_command_on_branch(context_id, "main", parent_ids)
    }

    fn snapshot_command_on_branch(
        context_id: ContextId,
        branch_name: &str,
        parent_ids: Vec<CommitId>,
    ) -> crate::CreateContextCommitSnapshot {
        let commit = ContextCommit::new(
            context_id,
            BranchName::new(branch_name).expect("branch name"),
            "Capture context graph",
            parent_ids,
            vec![ContextChange::created_context("Writer test context")],
            timestamp(1),
        )
        .expect("commit");

        crate::CreateContextCommitSnapshot::new(
            project_id_for_context(context_id),
            commit,
            ContextGraph::new(),
            timestamp(2),
            1,
        )
        .expect("command")
    }

    fn guarded_command(
        snapshot_command: crate::CreateContextCommitSnapshot,
        expected_branch_head: ExpectedBranchHead,
        idempotency_key: &str,
        request_digest: &str,
    ) -> GuardedContextCommitWrite {
        guarded_command_for_identity_source(
            "https://issuer.contextlab.test",
            snapshot_command,
            expected_branch_head,
            idempotency_key,
            request_digest,
        )
    }

    fn guarded_command_for_identity_source(
        identity_source: &str,
        snapshot_command: crate::CreateContextCommitSnapshot,
        expected_branch_head: ExpectedBranchHead,
        idempotency_key: &str,
        request_digest: &str,
    ) -> GuardedContextCommitWrite {
        GuardedContextCommitWrite::new(
            AuthenticatedPrincipal::new(PrincipalIdentity::new(
                IdentitySourceId::new(identity_source).expect("source"),
                PrincipalId::new("user:alex").expect("principal"),
            )),
            expected_branch_head,
            IdempotencyKey::new(idempotency_key).expect("idempotency key"),
            RequestDigest::new(request_digest).expect("request digest"),
            snapshot_command,
        )
        .expect("guarded command")
    }

    fn timestamp(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 9, 0, 0, seconds as u32)
            .single()
            .expect("valid timestamp")
    }
}
