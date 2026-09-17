//! Contract tests for immutable benchmark workspace projection persistence.

use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkEvaluationMetricChangeKindV1, BenchmarkExecutionCohortId,
    BenchmarkExecutionPlan, BenchmarkExecutionReceipt, BenchmarkExpectedOutput, BenchmarkSuite,
    BenchmarkSuiteId, MetricKind, MetricMeasurement, RegressionDecisionStatus, RegressionThreshold,
    ThresholdDirection,
};
use contextlab_storage::{
    BenchmarkEvidenceWriter, BenchmarkWorkspaceProjectionContractError,
    BenchmarkWorkspaceProjectionPersistenceError, BenchmarkWorkspaceProjectionReceiptScope,
    BenchmarkWorkspaceProjectionV1Query, BenchmarkWorkspaceProjectionV1Reader,
    BenchmarkWorkspaceProjectionV1Writer, BenchmarkWorkspaceProjectionWriteDisposition,
    ContextCommitRecord, ContextGraphProjection, ContextRecord,
    InMemoryBenchmarkWorkspaceProjectionV1Repository, InMemoryContextGraphRepository,
    PersistBenchmarkEvaluationEvidence, PersistBenchmarkWorkspaceProjectionV1,
    PostgresContextGraphRepository, ProjectRecord,
};
use contextlab_versioning::CommitId;
use serde_json::json;
use uuid::Uuid;

const PROJECT_ID: u128 = 10;
const CONTEXT_ID: u128 = 20;
const DATASET_ID: u128 = 30;
const FIRST_CASE_ID: u128 = 31;
const SECOND_CASE_ID: u128 = 32;
const SUITE_ID: u128 = 40;

#[test]
fn postgres_repository_implements_benchmark_workspace_projection_ports() {
    fn assert_projection_repository<T>()
    where
        T: BenchmarkWorkspaceProjectionV1Reader + BenchmarkWorkspaceProjectionV1Writer,
    {
    }

    assert_projection_repository::<PostgresContextGraphRepository>();
}

#[test]
fn postgres_migration_seals_exact_receipt_and_case_to_run_provenance() {
    let migration = include_str!("../migrations/0019_benchmark_workspace_projection_receipts.sql");

    for required in [
        "CREATE TABLE benchmark_workspace_projection_receipts",
        "CREATE TABLE benchmark_workspace_projection_cases",
        "CREATE TABLE benchmark_workspace_projection_receipt_seals",
        "uq_benchmark_decision_evidence_digest",
        "REFERENCES benchmark_decision_seals",
        "REFERENCES benchmark_dataset_cases",
        "REFERENCES benchmark_decision_runs",
        "validate_benchmark_workspace_projection_complete",
        "DEFERRABLE INITIALLY DEFERRED",
        "EXCEPT",
        "prevent_benchmark_workspace_projection_mutation",
        "prevent_benchmark_workspace_projection_case_insert_after_seal",
        "benchmark_workspace_projection_receipts_append_only",
        "benchmark_workspace_projection_cases_append_only",
        "benchmark_workspace_projection_receipt_seals_append_only",
        "benchmark_workspace_projection_cases_reject_after_seal",
    ] {
        assert!(
            migration.contains(required),
            "migration is missing required projection invariant: {required}"
        );
    }

    for forbidden in [
        "model_output",
        "expected_output",
        "input JSONB",
        "measurements JSONB",
    ] {
        assert!(
            !migration.contains(forbidden),
            "projection provenance migration stores forbidden raw content: {forbidden}"
        );
    }
}

#[tokio::test]
async fn projection_source_persists_replays_and_reads_only_at_its_exact_scope() {
    let material = source_material(50, 60, 0.95, 700.0).await;
    let scope = material.scope();
    let command = material.into_command().expect("validated source command");
    let repository = InMemoryBenchmarkWorkspaceProjectionV1Repository::new();

    let first = repository
        .persist_benchmark_workspace_projection(command.clone())
        .await
        .expect("persist projection source");
    let replay = repository
        .persist_benchmark_workspace_projection(command)
        .await
        .expect("replay identical projection source");
    let projection = repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(scope))
        .await
        .expect("read exact projection source");

    assert_eq!(
        first.disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Created
    );
    assert_eq!(
        replay.disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Replayed
    );
    assert_eq!(first.scope(), &scope);
    assert_eq!(projection.receipt().cohort_id(), scope.cohort_id());
    assert_eq!(projection.suite().id().as_uuid(), uuid(SUITE_ID));
    assert_eq!(projection.datasets()[0].id().as_uuid(), uuid(DATASET_ID));
    assert_eq!(
        projection
            .runs()
            .iter()
            .map(|run| run.case_id().as_uuid())
            .collect::<Vec<_>>(),
        vec![uuid(FIRST_CASE_ID), uuid(SECOND_CASE_ID)]
    );

    let wrong_scope = BenchmarkWorkspaceProjectionReceiptScope::new(
        scope.project_id(),
        scope.context_id(),
        CommitId::from_uuid(uuid(61)),
        scope.cohort_id(),
    );
    let error = repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
            wrong_scope,
        ))
        .await
        .expect_err("commit scope drift must fail closed");
    assert!(matches!(
        error,
        BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { .. }
    ));
}

#[tokio::test]
async fn projection_reader_requires_every_exact_scope_dimension() {
    let material = source_material(56, 67, 0.95, 700.0).await;
    let scope = material.scope();
    let command = material.into_command().expect("validated source command");
    let repository = InMemoryBenchmarkWorkspaceProjectionV1Repository::new();
    repository
        .persist_benchmark_workspace_projection(command)
        .await
        .expect("persist exact projection source");

    let wrong_scopes = [
        BenchmarkWorkspaceProjectionReceiptScope::new(
            ProjectId::new(),
            scope.context_id(),
            scope.context_commit_id(),
            scope.cohort_id(),
        ),
        BenchmarkWorkspaceProjectionReceiptScope::new(
            scope.project_id(),
            ContextId::new(),
            scope.context_commit_id(),
            scope.cohort_id(),
        ),
        BenchmarkWorkspaceProjectionReceiptScope::new(
            scope.project_id(),
            scope.context_id(),
            CommitId::new(),
            scope.cohort_id(),
        ),
        BenchmarkWorkspaceProjectionReceiptScope::new(
            scope.project_id(),
            scope.context_id(),
            scope.context_commit_id(),
            BenchmarkExecutionCohortId::from_uuid(uuid(68)),
        ),
    ];

    for wrong_scope in wrong_scopes {
        let error = repository
            .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
                wrong_scope,
            ))
            .await
            .expect_err("scope drift must fail closed");
        assert!(matches!(
            error,
            BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { scope }
                if scope == wrong_scope
        ));
    }
}

#[tokio::test]
async fn projection_source_rejects_cross_contract_mismatch_before_storage() {
    let first = source_material(51, 62, 0.95, 700.0).await;
    let other = source_material(52, 63, 0.95, 700.0).await;

    let error = PersistBenchmarkWorkspaceProjectionV1::new(
        first.evidence,
        first.datasets,
        first.suite,
        other.receipt,
    )
    .expect_err("evidence and receipt identity must match");

    assert!(matches!(
        error,
        BenchmarkWorkspaceProjectionContractError::DecisionNamespaceMismatch { .. }
    ));
}

#[tokio::test]
async fn projection_source_rejects_a_conflicting_reuse_of_one_cohort() {
    let first = source_material(53, 64, 0.95, 700.0)
        .await
        .into_command()
        .expect("first source");
    let conflicting = source_material(53, 64, 0.75, 900.0)
        .await
        .into_command()
        .expect("conflicting source with the same deterministic cohort");
    assert_eq!(first.scope().cohort_id(), conflicting.scope().cohort_id());
    let repository = InMemoryBenchmarkWorkspaceProjectionV1Repository::new();
    repository
        .persist_benchmark_workspace_projection(first)
        .await
        .expect("persist first source");

    let error = repository
        .persist_benchmark_workspace_projection(conflicting)
        .await
        .expect_err("different receipt evidence cannot reuse a cohort");

    assert!(matches!(
        error,
        BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict { .. }
    ));
}

#[tokio::test]
async fn reader_projects_an_existing_diff_without_exposing_raw_benchmark_content() {
    let baseline = source_material(54, 65, 0.95, 700.0)
        .await
        .into_command()
        .expect("baseline source");
    let revised = source_material(55, 66, 0.75, 900.0)
        .await
        .into_command()
        .expect("revised source");
    let baseline_scope = *baseline.scope();
    let revised_scope = *revised.scope();
    let repository = InMemoryBenchmarkWorkspaceProjectionV1Repository::new();
    repository
        .persist_benchmark_workspace_projection(revised)
        .await
        .expect("persist revised first");
    repository
        .persist_benchmark_workspace_projection(baseline)
        .await
        .expect("persist baseline second");

    let projection = repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::comparing(
            baseline_scope,
            revised_scope,
        ))
        .await
        .expect("project existing comparison");
    let diff = projection.evaluation_diff().expect("evaluation diff");

    assert_eq!(diff.baseline_cohort_id(), baseline_scope.cohort_id());
    assert_eq!(diff.revised_cohort_id(), revised_scope.cohort_id());
    assert_eq!(diff.metric_changes().len(), 2);
    assert_eq!(
        diff.metric_changes()
            .iter()
            .map(|change| change.metric())
            .collect::<Vec<_>>(),
        vec![MetricKind::LatencyMs, MetricKind::Accuracy]
    );
    assert_eq!(
        diff.status_change(),
        Some((
            RegressionDecisionStatus::Passed,
            RegressionDecisionStatus::Regressed,
        ))
    );

    let latency = &diff.metric_changes()[0];
    assert_eq!(
        latency.change_kind(),
        BenchmarkEvaluationMetricChangeKindV1::Modified
    );
    assert_eq!(
        latency.baseline().and_then(|metric| metric.observed()),
        Some(700.0)
    );
    assert_eq!(
        latency.revised().and_then(|metric| metric.observed()),
        Some(900.0)
    );

    let accuracy = &diff.metric_changes()[1];
    assert_eq!(
        accuracy.change_kind(),
        BenchmarkEvaluationMetricChangeKindV1::Modified
    );
    assert_eq!(
        accuracy.baseline().and_then(|metric| metric.observed()),
        Some(0.95)
    );
    assert_eq!(
        accuracy.revised().and_then(|metric| metric.observed()),
        Some(0.75)
    );

    let serialized = serde_json::to_string(&projection).expect("serialize redacted projection");
    for forbidden in [
        "input",
        "expected_output",
        "model_output",
        "measurements",
        "refund",
        "password",
    ] {
        assert!(
            !serialized.to_ascii_lowercase().contains(forbidden),
            "projection leaked forbidden raw field: {forbidden}"
        );
    }
}

struct SourceMaterial {
    evidence: contextlab_storage::BenchmarkDecisionEvidence,
    datasets: Vec<BenchmarkDataset>,
    suite: BenchmarkSuite,
    receipt: BenchmarkExecutionReceipt,
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
}

impl SourceMaterial {
    fn scope(&self) -> BenchmarkWorkspaceProjectionReceiptScope {
        BenchmarkWorkspaceProjectionReceiptScope::new(
            self.project_id,
            self.context_id,
            self.commit_id,
            self.receipt.cohort_id(),
        )
    }

    fn into_command(
        self,
    ) -> Result<PersistBenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionContractError>
    {
        PersistBenchmarkWorkspaceProjectionV1::new(
            self.evidence,
            self.datasets,
            self.suite,
            self.receipt,
        )
    }
}

async fn source_material(
    decision_id: u128,
    commit_id: u128,
    accuracy: f64,
    latency_ms: f64,
) -> SourceMaterial {
    let project_id = ProjectId::from_uuid(uuid(PROJECT_ID));
    let context_id = ContextId::from_uuid(uuid(CONTEXT_ID));
    let commit_id = CommitId::from_uuid(uuid(commit_id));
    let decision_id = contextlab_storage::BenchmarkDecisionId::from_uuid(uuid(decision_id));
    let datasets = datasets();
    let suite = suite();
    let plan = BenchmarkExecutionPlan::new(suite.clone(), datasets.clone()).expect("exact plan");
    let executed_at = Utc
        .with_ymd_and_hms(2026, 7, 23, 1, 2, 3)
        .single()
        .expect("timestamp");
    let results = vec![
        case_result(SECOND_CASE_ID, accuracy, latency_ms),
        case_result(FIRST_CASE_ID, accuracy, latency_ms),
    ];
    let cohort = plan
        .assemble_cohort(
            decision_id.as_uuid(),
            context_id,
            "model-v1",
            0.2,
            executed_at,
            results.clone(),
        )
        .expect("cohort");
    let runs = cohort.runs();
    let evaluation = suite.evaluate_runs(&runs);
    let evidence_command = PersistBenchmarkEvaluationEvidence::new(
        decision_id,
        project_id,
        commit_id,
        datasets.clone(),
        suite.clone(),
        runs,
        evaluation,
        "contextlab.exact-match",
        "evaluator-v1",
        executed_at,
    )
    .expect("sealed evidence command");
    let evidence_repository = evidence_repository(project_id, context_id, commit_id);
    let evidence = evidence_repository
        .persist_benchmark_evaluation(evidence_command)
        .await
        .expect("persist sealed evidence")
        .evidence()
        .clone();
    let receipt = BenchmarkExecutionReceipt::from_plan(
        &plan,
        decision_id.as_uuid(),
        context_id,
        "model-v1",
        0.2,
        executed_at,
        evidence.comparability().fingerprint(),
        results,
    )
    .expect("sealed receipt");

    SourceMaterial {
        evidence,
        datasets,
        suite,
        receipt,
        project_id,
        context_id,
        commit_id,
    }
}

fn datasets() -> Vec<BenchmarkDataset> {
    vec![
        BenchmarkDataset::with_id(
            BenchmarkDatasetId::from_uuid(uuid(DATASET_ID)),
            "Support cases",
            vec![
                BenchmarkCase::with_id(
                    BenchmarkCaseId::from_uuid(uuid(SECOND_CASE_ID)),
                    "Password reset",
                    json!({"secret_question": "redacted fixture"}),
                    BenchmarkExpectedOutput::Exact(json!({"allowed": true})),
                )
                .expect("second case"),
                BenchmarkCase::with_id(
                    BenchmarkCaseId::from_uuid(uuid(FIRST_CASE_ID)),
                    "Refund request",
                    json!({"question": "Can I return this?"}),
                    BenchmarkExpectedOutput::Exact(json!({"eligible": true})),
                )
                .expect("first case"),
            ],
        )
        .expect("dataset"),
    ]
}

fn suite() -> BenchmarkSuite {
    BenchmarkSuite::with_id(
        BenchmarkSuiteId::from_uuid(uuid(SUITE_ID)),
        "Release gate",
        vec![BenchmarkDatasetId::from_uuid(uuid(DATASET_ID))],
        vec![
            RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                .expect("latency threshold"),
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("accuracy threshold"),
        ],
    )
    .expect("suite")
}

fn case_result(case_id: u128, accuracy: f64, latency_ms: f64) -> BenchmarkCaseExecutionResult {
    BenchmarkCaseExecutionResult::new(
        BenchmarkDatasetId::from_uuid(uuid(DATASET_ID)),
        BenchmarkCaseId::from_uuid(uuid(case_id)),
        vec![
            MetricMeasurement::new(MetricKind::LatencyMs, latency_ms).expect("latency"),
            MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("accuracy"),
        ],
    )
    .expect("case result")
}

fn evidence_repository(
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
) -> InMemoryContextGraphRepository {
    let timestamp = Utc
        .with_ymd_and_hms(2026, 7, 23, 1, 0, 0)
        .single()
        .expect("timestamp");
    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id.to_string(),
            workspace_id: uuid(1).to_string(),
            name: "Benchmark project".to_owned(),
            slug: "benchmark-project".to_owned(),
            created_at: timestamp,
        }],
        contexts: vec![ContextRecord {
            id: context_id.to_string(),
            project_id: project_id.to_string(),
            experiment_id: None,
            name: "Benchmark Context".to_owned(),
            description: None,
            created_at: timestamp,
        }],
        commits: vec![ContextCommitRecord {
            id: commit_id.to_string(),
            context_id: context_id.to_string(),
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

const fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}
