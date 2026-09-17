//! Integration tests for private benchmark execution orchestration.

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkExpectedOutput, BenchmarkSuite, BenchmarkSuiteId, EvaluationRun,
    EvaluationRunId, MetricKind, MetricMeasurement, RegressionThreshold, ThresholdDirection,
};
use contextlab_storage::{
    BenchmarkCaseEvaluationRequest, BenchmarkCaseEvaluator, BenchmarkCaseEvaluatorError,
    BenchmarkDecisionComparisonScope, BenchmarkDecisionEvidence, BenchmarkDecisionId,
    BenchmarkDecisionPair, BenchmarkDefinitionBinding, BenchmarkDefinitionBindingId,
    BenchmarkEvidenceRepository, BenchmarkEvidenceWriteResult, BenchmarkEvidenceWriter,
    BenchmarkExecutionDisposition, BenchmarkExecutionRequest, BenchmarkExecutionService,
    BenchmarkWorkspaceProjectionContractError, BenchmarkWorkspaceProjectionPersistenceError,
    BenchmarkWorkspaceProjectionV1Query, BenchmarkWorkspaceProjectionV1Reader,
    BenchmarkWorkspaceProjectionV1Writer, BenchmarkWorkspaceProjectionWriteDisposition,
    BenchmarkWorkspaceProjectionWriteResult, ContextCommitRecord, ContextGraphProjection,
    ContextRecord, InMemoryContextGraphRepository, PersistBenchmarkEvaluationEvidence,
    PersistBenchmarkWorkspaceProjectionV1, ProjectRecord, StorageRepositoryError,
};
use contextlab_versioning::{BranchName, CommitId};
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[test]
fn in_memory_context_graph_repository_has_projection_port_parity() {
    fn assert_projection_ports<T>()
    where
        T: BenchmarkWorkspaceProjectionV1Reader + BenchmarkWorkspaceProjectionV1Writer,
    {
    }

    assert_projection_ports::<InMemoryContextGraphRepository>();
}

#[tokio::test]
async fn execution_uses_sealed_definitions_persists_once_and_replays_without_evaluation() {
    let fixture = fixture();
    let inner = fixture.repository();
    inner
        .persist_benchmark_evaluation(fixture.seed_command())
        .await
        .expect("persist sealed benchmark definitions");
    let repository = RecordingRepository::new(inner);
    let evaluator = RecordingEvaluator::new();
    let service = BenchmarkExecutionService::new(&repository, &evaluator);
    let request = fixture.execution_request();

    let created = service
        .execute(request.clone())
        .await
        .expect("execute benchmark");

    assert_eq!(
        created.disposition(),
        BenchmarkExecutionDisposition::Created
    );
    assert_eq!(created.evidence().run_ids().len(), 2);
    assert_eq!(
        created.evidence().run_ids(),
        &expected_run_ids(&fixture, request.decision_id())
    );
    assert_eq!(evaluator.call_count(), 2);
    assert!(evaluator.calls().iter().all(|call| {
        call.project_id() == fixture.project_id
            && call.context_id() == fixture.context_id
            && call.context_commit_id() == fixture.commit_id
            && call.model_version() == "fixture-model"
            && call.temperature() == 0.0
    }));
    assert_eq!(
        evaluator
            .calls()
            .iter()
            .map(|call| (call.case().key().dataset_id(), call.case().key().case_id()))
            .collect::<Vec<_>>(),
        fixture
            .dataset
            .cases()
            .iter()
            .map(|case| (fixture.dataset.id(), case.id()))
            .collect::<Vec<_>>()
    );
    assert_eq!(repository.write_count(), 1);
    assert_eq!(
        created.workspace_projection_disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Created
    );
    let created_scope = *created.workspace_projection_scope();
    assert_eq!(created_scope.project_id(), fixture.project_id);
    assert_eq!(created_scope.context_id(), fixture.context_id);
    assert_eq!(created_scope.context_commit_id(), fixture.commit_id);
    let projection = repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
            created_scope,
        ))
        .await
        .expect("read execution projection at its exact scope");
    assert_eq!(projection.receipt().cohort_id(), created_scope.cohort_id());
    assert_eq!(projection.runs().len(), 2);
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                request.decision_id(),
            )
            .await
            .expect("load created decision"),
        Some(created.evidence().clone())
    );

    let replayed = service
        .execute(request)
        .await
        .expect("replay benchmark execution");

    assert_eq!(
        replayed.disposition(),
        BenchmarkExecutionDisposition::Replayed
    );
    assert_eq!(replayed.evidence(), created.evidence());
    assert_eq!(replayed.workspace_projection_scope(), &created_scope);
    assert_eq!(
        replayed.workspace_projection_disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Replayed
    );
    assert_eq!(
        evaluator.call_count(),
        2,
        "an exact stored decision must avoid another evaluator invocation"
    );
    assert_eq!(repository.write_count(), 1);
    assert_eq!(repository.projection_write_count(), 2);
}

#[tokio::test]
async fn execution_from_exact_binding_uses_its_private_plan_and_preserves_replay_contract() {
    let fixture = fixture();
    let repository = RecordingRepository::new(fixture.repository());
    let evaluator = RecordingEvaluator::new();
    let request = fixture.binding_execution_request();

    let result = BenchmarkExecutionService::new(&repository, &evaluator)
        .execute(request)
        .await
        .expect("execute the exact binding selection");

    assert_eq!(result.disposition(), BenchmarkExecutionDisposition::Created);
    assert_eq!(result.evidence().run_ids().len(), 2);
    assert_eq!(evaluator.call_count(), 2);
    assert_eq!(repository.write_count(), 1);
    assert_eq!(repository.projection_write_count(), 1);
}

#[tokio::test]
async fn projection_failure_after_evidence_persistence_is_repaired_without_reevaluation() {
    let fixture = fixture();
    let inner = fixture.repository();
    inner
        .persist_benchmark_evaluation(fixture.seed_command())
        .await
        .expect("persist sealed benchmark definitions");
    let repository = RecordingRepository::failing_first_projection(inner);
    let evaluator = RecordingEvaluator::new();
    let service = BenchmarkExecutionService::new(&repository, &evaluator);
    let request = fixture.execution_request();

    let first_error = service
        .execute(request.clone())
        .await
        .expect_err("the first projection write is injected to fail");

    assert!(matches!(
        first_error,
        contextlab_storage::BenchmarkExecutionServiceError::WorkspaceProjection(
            BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable
        )
    ));
    assert_eq!(evaluator.call_count(), 2);
    assert_eq!(repository.write_count(), 1);
    assert_eq!(repository.projection_write_count(), 1);

    let repaired = service
        .execute(request)
        .await
        .expect("sealed evidence replay repairs the missing projection");

    assert_eq!(
        repaired.disposition(),
        BenchmarkExecutionDisposition::Replayed
    );
    assert_eq!(
        repaired.workspace_projection_disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Created
    );
    assert_eq!(
        evaluator.call_count(),
        2,
        "repair must reconstruct from sealed runs without evaluator invocation"
    );
    assert_eq!(repository.write_count(), 1);
    assert_eq!(repository.projection_write_count(), 2);
    let projection = repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
            *repaired.workspace_projection_scope(),
        ))
        .await
        .expect("read repaired projection at exact scope");
    assert_eq!(
        projection.receipt().cohort_id(),
        repaired.workspace_projection_scope().cohort_id()
    );
}

#[tokio::test]
async fn replay_accepts_storage_timestamp_normalization_within_the_same_microsecond() {
    let fixture = fixture();
    let inner = fixture.repository();
    let stored_at = timestamp();
    inner
        .persist_benchmark_evaluation(fixture.execution_seed_command(stored_at))
        .await
        .expect("persist storage-normalized execution evidence");
    let repository = RecordingRepository::new(inner);
    let evaluator = RecordingEvaluator::new();
    let service = BenchmarkExecutionService::new(&repository, &evaluator);
    let requested_at = stored_at + chrono::Duration::nanoseconds(789);

    let replayed = service
        .execute(fixture.execution_request_at(requested_at))
        .await
        .expect("sub-microsecond adapter normalization must preserve replay identity");

    assert_eq!(
        replayed.disposition(),
        BenchmarkExecutionDisposition::Replayed
    );
    assert_eq!(evaluator.call_count(), 0);
    repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
            *replayed.workspace_projection_scope(),
        ))
        .await
        .expect("read projection materialized from normalized evidence");
}

#[tokio::test]
async fn replay_fails_closed_for_missing_runs_and_evidence_receipt_mismatch() {
    let fixture = fixture();
    let inner = fixture.repository();
    inner
        .persist_benchmark_evaluation(fixture.seed_command())
        .await
        .expect("persist sealed benchmark definitions");
    let evaluator = RecordingEvaluator::new();
    let request = fixture.execution_request();
    let created = BenchmarkExecutionService::new(&inner, &evaluator)
        .execute(request.clone())
        .await
        .expect("create sealed execution and projection");
    let run_id = created.evidence().run_ids()[0];

    let missing_repository = RecordingRepository::hiding_run(inner.clone(), run_id);
    let missing_error = BenchmarkExecutionService::new(&missing_repository, &FailingEvaluator)
        .execute(request.clone())
        .await
        .expect_err("missing sealed run must fail before projection replay");
    assert!(matches!(
        missing_error,
        contextlab_storage::BenchmarkExecutionServiceError::StoredRunUnavailable { .. }
    ));
    assert_eq!(missing_repository.projection_write_count(), 0);

    let changed_run = EvaluationRun::with_id(
        run_id,
        fixture.context_id,
        "fixture-model",
        0.0,
        vec![MetricMeasurement::new(MetricKind::Accuracy, 0.1).expect("measurement")],
        timestamp(),
    )
    .expect("changed stored run");
    let changed_repository = RecordingRepository::replacing_run(inner, run_id, changed_run);
    let changed_error = BenchmarkExecutionService::new(&changed_repository, &FailingEvaluator)
        .execute(request)
        .await
        .expect_err("receipt that no longer reproduces sealed evidence must fail closed");
    assert!(matches!(
        changed_error,
        contextlab_storage::BenchmarkExecutionServiceError::WorkspaceProjectionContract(
            BenchmarkWorkspaceProjectionContractError::DecisionEvidenceMismatch
        )
    ));
    assert_eq!(changed_repository.projection_write_count(), 0);
}

#[tokio::test]
async fn projection_conflict_fails_closed_after_evidence_persistence() {
    let fixture = fixture();
    let inner = fixture.repository();
    inner
        .persist_benchmark_evaluation(fixture.seed_command())
        .await
        .expect("persist sealed benchmark definitions");
    let repository = RecordingRepository::conflicting_projection(inner);
    let evaluator = RecordingEvaluator::new();
    let request = fixture.execution_request();

    let error = BenchmarkExecutionService::new(&repository, &evaluator)
        .execute(request.clone())
        .await
        .expect_err("projection conflict must not be represented as execution success");

    assert!(matches!(
        error,
        contextlab_storage::BenchmarkExecutionServiceError::WorkspaceProjection(
            BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict { .. }
        )
    ));
    assert_eq!(evaluator.call_count(), 2);
    assert_eq!(repository.write_count(), 1);
    assert_eq!(repository.projection_write_count(), 1);
    assert!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                request.decision_id(),
            )
            .await
            .expect("load persisted evidence")
            .is_some(),
        "projection conflict occurs after evidence is sealed"
    );
}

#[test]
fn evaluator_failure_hides_adapter_diagnostics() {
    let error = BenchmarkCaseEvaluatorError::new("provider response included secret-like detail");

    assert_eq!(error.to_string(), "benchmark evaluator is unavailable");
}

#[tokio::test]
async fn evaluator_failure_does_not_persist_partial_evidence() {
    let fixture = fixture();
    let inner = fixture.repository();
    inner
        .persist_benchmark_evaluation(fixture.seed_command())
        .await
        .expect("persist sealed benchmark definitions");
    let repository = RecordingRepository::new(inner);
    let service = BenchmarkExecutionService::new(&repository, &FailingEvaluator);
    let request = fixture.execution_request();

    let error = service
        .execute(request.clone())
        .await
        .expect_err("evaluator failure must prevent persistence");

    assert!(matches!(
        error,
        contextlab_storage::BenchmarkExecutionServiceError::Evaluator(_)
    ));
    assert_eq!(repository.write_count(), 0);
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                request.decision_id(),
            )
            .await
            .expect("load failed decision"),
        None
    );
}

#[tokio::test]
async fn missing_suite_or_dataset_prevents_evaluation_and_persistence() {
    let fixture = fixture();
    let missing_suite_repository = RecordingRepository::new(fixture.repository());
    let evaluator = RecordingEvaluator::new();
    let missing_suite_service =
        BenchmarkExecutionService::new(&missing_suite_repository, &evaluator);

    let suite_error = missing_suite_service
        .execute(fixture.execution_request())
        .await
        .expect_err("missing suite must fail before evaluation");

    assert!(matches!(
        suite_error,
        contextlab_storage::BenchmarkExecutionServiceError::SuiteUnavailable
    ));
    assert_eq!(evaluator.call_count(), 0);
    assert_eq!(missing_suite_repository.write_count(), 0);

    let inner = fixture.repository();
    inner
        .persist_benchmark_evaluation(fixture.seed_command())
        .await
        .expect("persist definitions before hiding one dataset");
    let missing_dataset_repository =
        RecordingRepository::hiding_dataset(inner, fixture.dataset.id());
    let missing_dataset_service =
        BenchmarkExecutionService::new(&missing_dataset_repository, &evaluator);

    let dataset_error = missing_dataset_service
        .execute(fixture.execution_request())
        .await
        .expect_err("missing dataset must fail before evaluation");

    assert!(matches!(
        dataset_error,
        contextlab_storage::BenchmarkExecutionServiceError::DatasetUnavailable
    ));
    assert_eq!(evaluator.call_count(), 0);
    assert_eq!(missing_dataset_repository.write_count(), 0);
}

#[tokio::test]
async fn invalid_request_and_unknown_case_result_prevent_persistence() {
    let fixture = fixture();
    assert!(matches!(
        BenchmarkExecutionRequest::new(
            fixture.execution_decision_id,
            fixture.project_id,
            fixture.context_id,
            fixture.commit_id,
            fixture.suite.id(),
            " ",
            0.0,
            "fixture-evaluator",
            "v1",
            timestamp(),
        ),
        Err(contextlab_storage::BenchmarkExecutionRequestError::InvalidRunConfiguration(_))
    ));

    let inner = fixture.repository();
    inner
        .persist_benchmark_evaluation(fixture.seed_command())
        .await
        .expect("persist sealed benchmark definitions");
    let repository = RecordingRepository::new(inner);
    let service = BenchmarkExecutionService::new(&repository, &UnknownCaseEvaluator);

    let error = service
        .execute(fixture.execution_request())
        .await
        .expect_err("unknown evaluator result must not persist");

    assert!(matches!(
        error,
        contextlab_storage::BenchmarkExecutionServiceError::Plan(
            contextlab_evaluation::BenchmarkExecutionError::UnknownCaseResult { .. }
        )
    ));
    assert_eq!(repository.write_count(), 0);
}

#[derive(Clone)]
struct RecordingEvaluator {
    calls: Arc<Mutex<Vec<BenchmarkCaseEvaluationRequest>>>,
}

impl RecordingEvaluator {
    fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.lock().expect("calls lock").len()
    }

    fn calls(&self) -> Vec<BenchmarkCaseEvaluationRequest> {
        self.calls.lock().expect("calls lock").clone()
    }
}

#[async_trait]
impl BenchmarkCaseEvaluator for RecordingEvaluator {
    async fn evaluate_case(
        &self,
        request: BenchmarkCaseEvaluationRequest,
    ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError> {
        let result = BenchmarkCaseExecutionResult::new(
            request.case().dataset_id(),
            request.case().case_id(),
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement")],
        )
        .expect("result");
        self.calls.lock().expect("calls lock").push(request);
        Ok(result)
    }
}

struct FailingEvaluator;

#[async_trait]
impl BenchmarkCaseEvaluator for FailingEvaluator {
    async fn evaluate_case(
        &self,
        _request: BenchmarkCaseEvaluationRequest,
    ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError> {
        Err(BenchmarkCaseEvaluatorError::new("untrusted adapter detail"))
    }
}

struct UnknownCaseEvaluator;

#[async_trait]
impl BenchmarkCaseEvaluator for UnknownCaseEvaluator {
    async fn evaluate_case(
        &self,
        request: BenchmarkCaseEvaluationRequest,
    ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError> {
        BenchmarkCaseExecutionResult::new(
            request.case().dataset_id(),
            BenchmarkCaseId::new(),
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement")],
        )
        .map_err(|error| BenchmarkCaseEvaluatorError::new(error.to_string()))
    }
}

#[derive(Clone)]
struct RecordingRepository {
    inner: InMemoryContextGraphRepository,
    write_count: Arc<Mutex<usize>>,
    projection_write_count: Arc<Mutex<usize>>,
    projection_fault: ProjectionFault,
    hidden_dataset_id: Option<BenchmarkDatasetId>,
    run_fault: RunFault,
}

#[derive(Clone, Copy, Default)]
enum ProjectionFault {
    #[default]
    None,
    FailFirst,
    Conflict,
}

#[derive(Clone, Default)]
enum RunFault {
    #[default]
    None,
    Missing(EvaluationRunId),
    Replaced {
        requested_id: EvaluationRunId,
        replacement: EvaluationRun,
    },
}

impl RecordingRepository {
    fn new(inner: InMemoryContextGraphRepository) -> Self {
        Self {
            inner,
            write_count: Arc::new(Mutex::new(0)),
            projection_write_count: Arc::new(Mutex::new(0)),
            projection_fault: ProjectionFault::None,
            hidden_dataset_id: None,
            run_fault: RunFault::None,
        }
    }

    fn hiding_dataset(
        inner: InMemoryContextGraphRepository,
        dataset_id: BenchmarkDatasetId,
    ) -> Self {
        Self {
            inner,
            write_count: Arc::new(Mutex::new(0)),
            projection_write_count: Arc::new(Mutex::new(0)),
            projection_fault: ProjectionFault::None,
            hidden_dataset_id: Some(dataset_id),
            run_fault: RunFault::None,
        }
    }

    fn failing_first_projection(inner: InMemoryContextGraphRepository) -> Self {
        Self {
            inner,
            write_count: Arc::new(Mutex::new(0)),
            projection_write_count: Arc::new(Mutex::new(0)),
            projection_fault: ProjectionFault::FailFirst,
            hidden_dataset_id: None,
            run_fault: RunFault::None,
        }
    }

    fn conflicting_projection(inner: InMemoryContextGraphRepository) -> Self {
        Self {
            inner,
            write_count: Arc::new(Mutex::new(0)),
            projection_write_count: Arc::new(Mutex::new(0)),
            projection_fault: ProjectionFault::Conflict,
            hidden_dataset_id: None,
            run_fault: RunFault::None,
        }
    }

    fn hiding_run(inner: InMemoryContextGraphRepository, run_id: EvaluationRunId) -> Self {
        Self {
            inner,
            write_count: Arc::new(Mutex::new(0)),
            projection_write_count: Arc::new(Mutex::new(0)),
            projection_fault: ProjectionFault::None,
            hidden_dataset_id: None,
            run_fault: RunFault::Missing(run_id),
        }
    }

    fn replacing_run(
        inner: InMemoryContextGraphRepository,
        requested_id: EvaluationRunId,
        replacement: EvaluationRun,
    ) -> Self {
        Self {
            inner,
            write_count: Arc::new(Mutex::new(0)),
            projection_write_count: Arc::new(Mutex::new(0)),
            projection_fault: ProjectionFault::None,
            hidden_dataset_id: None,
            run_fault: RunFault::Replaced {
                requested_id,
                replacement,
            },
        }
    }

    fn write_count(&self) -> usize {
        *self.write_count.lock().expect("write count lock")
    }

    fn projection_write_count(&self) -> usize {
        *self
            .projection_write_count
            .lock()
            .expect("projection write count lock")
    }
}

#[async_trait]
impl BenchmarkEvidenceWriter for RecordingRepository {
    async fn persist_benchmark_evaluation(
        &self,
        command: PersistBenchmarkEvaluationEvidence,
    ) -> Result<BenchmarkEvidenceWriteResult, StorageRepositoryError> {
        *self.write_count.lock().expect("write count lock") += 1;
        self.inner.persist_benchmark_evaluation(command).await
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Writer for RecordingRepository {
    async fn persist_benchmark_workspace_projection(
        &self,
        command: PersistBenchmarkWorkspaceProjectionV1,
    ) -> Result<BenchmarkWorkspaceProjectionWriteResult, BenchmarkWorkspaceProjectionPersistenceError>
    {
        let attempt = {
            let mut count = self
                .projection_write_count
                .lock()
                .expect("projection write count lock");
            *count += 1;
            *count
        };
        match self.projection_fault {
            ProjectionFault::FailFirst if attempt == 1 => {
                return Err(BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable);
            }
            ProjectionFault::Conflict => {
                return Err(
                    BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict {
                        cohort_id: command.scope().cohort_id(),
                    },
                );
            }
            ProjectionFault::None | ProjectionFault::FailFirst => {}
        }
        self.inner
            .persist_benchmark_workspace_projection(command)
            .await
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Reader for RecordingRepository {
    async fn read_benchmark_workspace_projection(
        &self,
        query: BenchmarkWorkspaceProjectionV1Query,
    ) -> Result<
        contextlab_evaluation::BenchmarkWorkspaceProjectionV1,
        BenchmarkWorkspaceProjectionPersistenceError,
    > {
        self.inner.read_benchmark_workspace_projection(query).await
    }
}

#[async_trait]
impl BenchmarkEvidenceRepository for RecordingRepository {
    async fn get_benchmark_dataset(
        &self,
        project_id: ProjectId,
        dataset_id: BenchmarkDatasetId,
    ) -> Result<Option<BenchmarkDataset>, StorageRepositoryError> {
        if self.hidden_dataset_id == Some(dataset_id) {
            return Ok(None);
        }
        self.inner
            .get_benchmark_dataset(project_id, dataset_id)
            .await
    }

    async fn get_benchmark_suite(
        &self,
        project_id: ProjectId,
        suite_id: BenchmarkSuiteId,
    ) -> Result<Option<BenchmarkSuite>, StorageRepositoryError> {
        self.inner.get_benchmark_suite(project_id, suite_id).await
    }

    async fn get_benchmark_run(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        run_id: EvaluationRunId,
    ) -> Result<Option<EvaluationRun>, StorageRepositoryError> {
        match &self.run_fault {
            RunFault::Missing(hidden_id) if *hidden_id == run_id => return Ok(None),
            RunFault::Replaced {
                requested_id,
                replacement,
            } if *requested_id == run_id => return Ok(Some(replacement.clone())),
            RunFault::None | RunFault::Missing(_) | RunFault::Replaced { .. } => {}
        }
        self.inner
            .get_benchmark_run(project_id, context_id, context_commit_id, run_id)
            .await
    }

    async fn get_benchmark_decision(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
    ) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError> {
        self.inner
            .get_benchmark_decision(project_id, context_id, context_commit_id, decision_id)
            .await
    }

    async fn get_benchmark_decision_pair(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        baseline: BenchmarkDecisionComparisonScope,
        revised: BenchmarkDecisionComparisonScope,
    ) -> Result<BenchmarkDecisionPair, StorageRepositoryError> {
        self.inner
            .get_benchmark_decision_pair(project_id, context_id, baseline, revised)
            .await
    }
}

struct Fixture {
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
    seed_decision_id: BenchmarkDecisionId,
    execution_decision_id: BenchmarkDecisionId,
    dataset: BenchmarkDataset,
    suite: BenchmarkSuite,
}

impl Fixture {
    fn repository(&self) -> InMemoryContextGraphRepository {
        let timestamp = timestamp();
        InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: self.project_id.to_string(),
                workspace_id: Uuid::new_v4().to_string(),
                name: "Benchmark project".to_owned(),
                slug: "benchmark-project".to_owned(),
                created_at: timestamp,
            }],
            contexts: vec![ContextRecord {
                id: self.context_id.to_string(),
                project_id: self.project_id.to_string(),
                experiment_id: None,
                name: "Benchmark context".to_owned(),
                description: None,
                created_at: timestamp,
            }],
            commits: vec![ContextCommitRecord {
                id: self.commit_id.to_string(),
                context_id: self.context_id.to_string(),
                branch_name: "main".to_owned(),
                message: "Benchmark candidate".to_owned(),
                parent_commit_ids: Vec::new(),
                changes: json!([]),
                change_count: 0,
                authored_at: timestamp,
                created_at: timestamp,
            }],
            ..ContextGraphProjection::default()
        })
    }

    fn seed_command(&self) -> PersistBenchmarkEvaluationEvidence {
        let run = contextlab_evaluation::EvaluationRun::new(
            self.context_id,
            "fixture-model",
            0.0,
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement")],
            timestamp(),
        )
        .expect("seed run");
        let evaluation = self.suite.evaluate_runs(std::slice::from_ref(&run));
        PersistBenchmarkEvaluationEvidence::new(
            self.seed_decision_id,
            self.project_id,
            self.commit_id,
            vec![self.dataset.clone()],
            self.suite.clone(),
            vec![run],
            evaluation,
            "fixture-evaluator",
            "v1",
            timestamp(),
        )
        .expect("seed command")
    }

    fn execution_request(&self) -> BenchmarkExecutionRequest {
        self.execution_request_at(timestamp())
    }

    fn execution_request_at(
        &self,
        recorded_at: chrono::DateTime<Utc>,
    ) -> BenchmarkExecutionRequest {
        BenchmarkExecutionRequest::new(
            self.execution_decision_id,
            self.project_id,
            self.context_id,
            self.commit_id,
            self.suite.id(),
            "fixture-model",
            0.0,
            "fixture-evaluator",
            "v1",
            recorded_at,
        )
        .expect("execution request")
    }

    fn binding_execution_request(&self) -> BenchmarkExecutionRequest {
        let binding = BenchmarkDefinitionBinding::rehydrate(
            BenchmarkDefinitionBindingId::from_uuid(Uuid::from_u128(
                0xcccccccccccccccccccccccccccccc01,
            )),
            self.project_id,
            self.context_id,
            self.commit_id,
            BranchName::new("main").expect("branch"),
            1,
            vec![self.dataset.clone()],
            self.suite.clone(),
            timestamp(),
        )
        .expect("binding");
        BenchmarkExecutionRequest::from_definition_binding(
            self.execution_decision_id,
            &binding,
            self.project_id,
            self.context_id,
            self.commit_id,
            "fixture-model",
            0.0,
            "fixture-evaluator",
            "v1",
            timestamp(),
        )
        .expect("binding execution request")
    }

    fn execution_seed_command(
        &self,
        recorded_at: chrono::DateTime<Utc>,
    ) -> PersistBenchmarkEvaluationEvidence {
        let runs = self
            .dataset
            .cases()
            .iter()
            .map(|case| {
                EvaluationRun::with_id(
                    expected_run_id(self.execution_decision_id, self.dataset.id(), case.id()),
                    self.context_id,
                    "fixture-model",
                    0.0,
                    vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement")],
                    recorded_at,
                )
                .expect("execution run")
            })
            .collect::<Vec<_>>();
        let evaluation = self.suite.evaluate_runs(&runs);
        PersistBenchmarkEvaluationEvidence::new(
            self.execution_decision_id,
            self.project_id,
            self.commit_id,
            vec![self.dataset.clone()],
            self.suite.clone(),
            runs,
            evaluation,
            "fixture-evaluator",
            "v1",
            recorded_at,
        )
        .expect("execution seed command")
    }
}

fn fixture() -> Fixture {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let dataset = BenchmarkDataset::new(
        "Support cases",
        vec![
            BenchmarkCase::new(
                "Refund request",
                json!({"question": "Can I return this?"}),
                BenchmarkExpectedOutput::Exact(json!({"eligible": true})),
            )
            .expect("case"),
            BenchmarkCase::new(
                "Account cancellation",
                json!({"question": "How do I cancel?"}),
                BenchmarkExpectedOutput::Exact(json!({"eligible": false})),
            )
            .expect("case"),
        ],
    )
    .expect("dataset");
    let suite = BenchmarkSuite::new(
        "Release gate",
        vec![dataset.id()],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");

    Fixture {
        project_id,
        context_id,
        commit_id: CommitId::new(),
        seed_decision_id: BenchmarkDecisionId::new(),
        execution_decision_id: BenchmarkDecisionId::new(),
        dataset,
        suite,
    }
}

fn timestamp() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 18, 0, 0, 0)
        .single()
        .expect("timestamp")
}

fn expected_run_ids(fixture: &Fixture, decision_id: BenchmarkDecisionId) -> Vec<EvaluationRunId> {
    let mut run_ids = fixture
        .dataset
        .cases()
        .iter()
        .map(|case| expected_run_id(decision_id, fixture.dataset.id(), case.id()))
        .collect::<Vec<_>>();
    run_ids.sort_unstable();
    run_ids
}

fn expected_run_id(
    decision_id: BenchmarkDecisionId,
    dataset_id: contextlab_evaluation::BenchmarkDatasetId,
    case_id: contextlab_evaluation::BenchmarkCaseId,
) -> EvaluationRunId {
    let mut name = [0_u8; 32];
    name[..16].copy_from_slice(dataset_id.as_uuid().as_bytes());
    name[16..].copy_from_slice(case_id.as_uuid().as_bytes());
    EvaluationRunId::from_uuid(Uuid::new_v5(&decision_id.as_uuid(), &name))
}
