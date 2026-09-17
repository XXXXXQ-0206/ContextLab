//! Breadth receipt for the private benchmark authoring-to-workspace path.

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use contextlab_auth::{AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkExpectedOutput, BenchmarkSuite, BenchmarkSuiteId, MetricKind,
    MetricMeasurement, RegressionDecisionStatus, RegressionThreshold, ThresholdDirection,
};
use contextlab_storage::{
    BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION, BenchmarkCaseEvaluationRequest,
    BenchmarkCaseEvaluator, BenchmarkCaseEvaluatorError, BenchmarkDecisionId,
    BenchmarkDefinitionBindingCommand, BenchmarkDefinitionBindingWriteDisposition,
    BenchmarkDefinitionBindingWriter, BenchmarkEvidenceRepository, BenchmarkExecutionDisposition,
    BenchmarkExecutionRequest, BenchmarkExecutionService,
    BenchmarkWorkspaceProjectionDecisionQuery, BenchmarkWorkspaceProjectionPersistenceError,
    BenchmarkWorkspaceProjectionReceiptScope, BenchmarkWorkspaceProjectionV1Query,
    BenchmarkWorkspaceProjectionV1Reader, BenchmarkWorkspaceProjectionWriteDisposition,
    ContextCommitRecord, ContextGraphProjection, ContextRecord, InMemoryContextGraphRepository,
    ProjectRecord,
};
use contextlab_versioning::{BranchName, CommitId};
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

const PROJECT: u128 = 0x100;
const CONTEXT: u128 = 0x200;
const BASELINE_COMMIT: u128 = 0x300;
const REVISED_COMMIT: u128 = 0x400;
const SUITE: u128 = 0x500;
const DATASET_ALPHA: u128 = 0x610;
const DATASET_BETA: u128 = 0x620;
const ALPHA_CASE_ONE: u128 = 0x611;
const ALPHA_CASE_TWO: u128 = 0x612;
const BETA_CASE_ONE: u128 = 0x621;
const BETA_CASE_TWO: u128 = 0x622;
const BASELINE_DECISION: u128 = 0x700;
const REVISED_DECISION: u128 = 0x800;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EvaluationCall {
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
    dataset_id: BenchmarkDatasetId,
    case_id: BenchmarkCaseId,
}

#[derive(Debug, Clone)]
struct BreadthEvaluator {
    revised_commit: CommitId,
    calls: Arc<Mutex<Vec<EvaluationCall>>>,
}

#[async_trait]
impl BenchmarkCaseEvaluator for BreadthEvaluator {
    async fn evaluate_case(
        &self,
        request: BenchmarkCaseEvaluationRequest,
    ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError> {
        self.calls
            .lock()
            .expect("call recorder")
            .push(EvaluationCall {
                project_id: request.project_id(),
                context_id: request.context_id(),
                commit_id: request.context_commit_id(),
                dataset_id: request.case().dataset_id(),
                case_id: request.case().case_id(),
            });

        let regressed = request.context_commit_id() == self.revised_commit;
        let accuracy = if regressed { 0.75 } else { 0.95 };
        let latency = if regressed { 900.0 } else { 700.0 };
        BenchmarkCaseExecutionResult::new(
            request.case().dataset_id(),
            request.case().case_id(),
            vec![
                MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("finite accuracy"),
                MetricMeasurement::new(MetricKind::LatencyMs, latency).expect("finite latency"),
            ],
        )
        .map_err(|_| BenchmarkCaseEvaluatorError::new("invalid fixture measurement"))
    }
}

#[tokio::test]
async fn multi_dataset_benchmark_path_preserves_breadth_and_safe_projection() {
    let project_id = ProjectId::from_uuid(uuid(PROJECT));
    let context_id = ContextId::from_uuid(uuid(CONTEXT));
    let baseline_commit = CommitId::from_uuid(uuid(BASELINE_COMMIT));
    let revised_commit = CommitId::from_uuid(uuid(REVISED_COMMIT));
    let datasets = datasets();
    let suite = suite();
    let repository = repository(project_id, context_id, baseline_commit, revised_commit);

    let baseline_binding_command = binding_command(
        baseline_commit,
        0x900,
        "baseline",
        datasets.clone(),
        suite.clone(),
    );
    let baseline_created = repository
        .persist_benchmark_definition_binding(baseline_binding_command.clone())
        .await
        .expect("create baseline binding");
    let baseline_replayed = repository
        .persist_benchmark_definition_binding(baseline_binding_command)
        .await
        .expect("replay baseline binding");
    assert_eq!(
        baseline_created.disposition(),
        BenchmarkDefinitionBindingWriteDisposition::Created
    );
    assert_eq!(
        baseline_replayed.disposition(),
        BenchmarkDefinitionBindingWriteDisposition::Replayed
    );
    assert_eq!(baseline_replayed.binding(), baseline_created.binding());

    let revised_binding = repository
        .persist_benchmark_definition_binding(binding_command(
            revised_commit,
            0xa00,
            "revised",
            datasets.clone(),
            suite.clone(),
        ))
        .await
        .expect("create revised binding")
        .binding()
        .clone();

    assert_eq!(
        baseline_created.binding().project_id(),
        project_id,
        "binding must retain the exact project scope"
    );
    assert_eq!(baseline_created.binding().context_id(), context_id);
    assert_eq!(
        baseline_created.binding().context_commit_id(),
        baseline_commit
    );
    assert_eq!(
        baseline_created.binding().dataset_ids(),
        vec![
            BenchmarkDatasetId::from_uuid(uuid(DATASET_ALPHA)),
            BenchmarkDatasetId::from_uuid(uuid(DATASET_BETA)),
        ]
    );
    assert_eq!(
        baseline_created
            .binding()
            .datasets()
            .iter()
            .flat_map(|dataset| dataset.cases().iter().map(BenchmarkCase::id))
            .collect::<Vec<_>>(),
        vec![
            BenchmarkCaseId::from_uuid(uuid(ALPHA_CASE_ONE)),
            BenchmarkCaseId::from_uuid(uuid(ALPHA_CASE_TWO)),
            BenchmarkCaseId::from_uuid(uuid(BETA_CASE_ONE)),
            BenchmarkCaseId::from_uuid(uuid(BETA_CASE_TWO)),
        ]
    );

    let calls = Arc::new(Mutex::new(Vec::new()));
    let evaluator = BreadthEvaluator {
        revised_commit,
        calls: Arc::clone(&calls),
    };
    let service = BenchmarkExecutionService::new(&repository, &evaluator);
    let baseline_request = BenchmarkExecutionRequest::from_definition_binding(
        BenchmarkDecisionId::from_uuid(uuid(BASELINE_DECISION)),
        baseline_created.binding(),
        project_id,
        context_id,
        baseline_commit,
        "breadth-model",
        0.0,
        "contextlab.exact-match",
        "breadth-v1",
        timestamp(),
    )
    .expect("baseline request from exact binding");
    let revised_request = BenchmarkExecutionRequest::from_definition_binding(
        BenchmarkDecisionId::from_uuid(uuid(REVISED_DECISION)),
        &revised_binding,
        project_id,
        context_id,
        revised_commit,
        "breadth-model",
        0.0,
        "contextlab.exact-match",
        "breadth-v1",
        timestamp(),
    )
    .expect("revised request from exact binding");

    let baseline = service
        .execute(baseline_request.clone())
        .await
        .expect("execute baseline breadth cohort");
    let baseline_replay = service
        .execute(baseline_request)
        .await
        .expect("replay baseline breadth cohort");
    let revised = service
        .execute(revised_request)
        .await
        .expect("execute revised breadth cohort");

    assert_eq!(
        baseline.disposition(),
        BenchmarkExecutionDisposition::Created
    );
    assert_eq!(
        baseline_replay.disposition(),
        BenchmarkExecutionDisposition::Replayed
    );
    assert_eq!(baseline_replay.evidence(), baseline.evidence());
    assert_eq!(
        baseline_replay.workspace_projection_disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Replayed
    );
    assert_eq!(
        revised.disposition(),
        BenchmarkExecutionDisposition::Created
    );
    assert_eq!(calls.lock().expect("call recorder").len(), 8);

    let expected_calls = expected_calls(project_id, context_id, baseline_commit, revised_commit);
    assert_eq!(*calls.lock().expect("call recorder"), expected_calls);

    let baseline_evidence = baseline.evidence();
    let revised_evidence = revised.evidence();
    for (evidence, commit, status) in [
        (
            baseline_evidence,
            baseline_commit,
            RegressionDecisionStatus::Passed,
        ),
        (
            revised_evidence,
            revised_commit,
            RegressionDecisionStatus::Regressed,
        ),
    ] {
        assert_eq!(evidence.project_id(), project_id);
        assert_eq!(evidence.context_id(), context_id);
        assert_eq!(evidence.context_commit_id(), commit);
        assert_eq!(evidence.dataset_ids(), suite.dataset_ids());
        assert_eq!(evidence.run_ids().len(), 4);
        assert_eq!(evidence.metric_results().len(), 2);
        assert_eq!(evidence.status(), status);
        assert!(evidence.evidence_digest().starts_with("sha256:"));
        assert!(
            evidence
                .comparability()
                .fingerprint()
                .starts_with("sha256:")
        );
        for run_id in evidence.run_ids() {
            assert!(
                repository
                    .get_benchmark_run(project_id, context_id, commit, *run_id)
                    .await
                    .expect("read persisted run")
                    .is_some(),
                "every sealed run id must be persisted at its exact commit"
            );
        }
    }

    let baseline_scope = *baseline.workspace_projection_scope();
    let revised_scope = *revised.workspace_projection_scope();
    assert_eq!(
        baseline_scope,
        BenchmarkWorkspaceProjectionReceiptScope::new(
            project_id,
            context_id,
            baseline_commit,
            baseline_scope.cohort_id(),
        )
    );
    assert_eq!(revised_scope.project_id(), project_id);
    assert_eq!(revised_scope.context_id(), context_id);
    assert_eq!(revised_scope.context_commit_id(), revised_commit);

    let baseline_projection = repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
            baseline_scope,
        ))
        .await
        .expect("read baseline workspace projection");
    assert_eq!(baseline_projection.datasets().len(), 2);
    assert_eq!(
        baseline_projection
            .datasets()
            .iter()
            .map(|dataset| (dataset.id(), dataset.case_count()))
            .collect::<Vec<_>>(),
        vec![
            (BenchmarkDatasetId::from_uuid(uuid(DATASET_ALPHA)), 2),
            (BenchmarkDatasetId::from_uuid(uuid(DATASET_BETA)), 2),
        ]
    );
    assert_eq!(baseline_projection.runs().len(), 4);
    assert_eq!(
        baseline_projection
            .runs()
            .iter()
            .map(|run| (run.dataset_id(), run.case_id(), run.metric_count()))
            .collect::<Vec<_>>(),
        expected_case_projection()
            .into_iter()
            .map(|(dataset_id, case_id)| (dataset_id, case_id, 2))
            .collect::<Vec<_>>()
    );
    assert_eq!(baseline_projection.scorecard().run_count(), 4);
    assert_eq!(
        baseline_projection.regression_status(),
        RegressionDecisionStatus::Passed
    );
    assert_eq!(baseline_projection.scorecard().metrics().len(), 2);
    assert_metric_metadata(&baseline_projection, 0.95, 700.0);
    assert!(baseline_projection.evaluation_diff().is_none());

    let revised_projection = repository
        .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::comparing(
            baseline_scope,
            revised_scope,
        ))
        .await
        .expect("read revised workspace projection with baseline diff");
    assert_eq!(revised_projection.runs().len(), 4);
    assert_eq!(revised_projection.scorecard().run_count(), 4);
    assert_eq!(
        revised_projection.regression_status(),
        RegressionDecisionStatus::Regressed
    );
    assert_metric_metadata(&revised_projection, 0.75, 900.0);
    let diff = revised_projection
        .evaluation_diff()
        .expect("evaluation diff");
    assert_eq!(diff.baseline_cohort_id(), baseline_scope.cohort_id());
    assert_eq!(diff.revised_cohort_id(), revised_scope.cohort_id());
    assert_eq!(
        diff.status_change(),
        Some((
            RegressionDecisionStatus::Passed,
            RegressionDecisionStatus::Regressed,
        ))
    );
    assert_eq!(diff.metric_changes().len(), 2);
    assert_eq!(diff.metric_changes()[0].metric(), MetricKind::LatencyMs);
    assert_eq!(diff.metric_changes()[1].metric(), MetricKind::Accuracy);
    assert!(diff.metric_changes().iter().all(|change| {
        change.change_kind()
            == contextlab_evaluation::BenchmarkEvaluationMetricChangeKindV1::Modified
    }));

    let decision_scope = repository
        .resolve_benchmark_workspace_projection_scope(
            BenchmarkWorkspaceProjectionDecisionQuery::new(
                project_id,
                context_id,
                revised_commit,
                revised_evidence.decision_id(),
            ),
        )
        .await
        .expect("resolve decision-bound projection scope")
        .expect("decision has a projection scope");
    assert_eq!(decision_scope, revised_scope);

    let wrong_scope = BenchmarkWorkspaceProjectionReceiptScope::new(
        ProjectId::from_uuid(uuid(0xdead)),
        context_id,
        revised_commit,
        revised_scope.cohort_id(),
    );
    assert!(matches!(
        repository
            .read_benchmark_workspace_projection(BenchmarkWorkspaceProjectionV1Query::single(
                wrong_scope,
            ))
            .await
            .expect_err("workspace query must require exact project scope"),
        BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { scope }
            if scope == wrong_scope
    ));

    let serialized = serde_json::to_string(&revised_projection).expect("serialize safe projection");
    for forbidden in [
        "raw_case_input",
        "raw_expected_output",
        "RAW_CASE_",
        "RAW_EXPECTED_",
        "model_output",
        "measurements",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "safe workspace projection leaked raw payload marker: {forbidden}"
        );
    }
}

fn datasets() -> Vec<BenchmarkDataset> {
    vec![
        dataset(
            DATASET_BETA,
            "Beta",
            vec![(BETA_CASE_TWO, "Beta two"), (BETA_CASE_ONE, "Beta one")],
        ),
        dataset(
            DATASET_ALPHA,
            "Alpha",
            vec![(ALPHA_CASE_TWO, "Alpha two"), (ALPHA_CASE_ONE, "Alpha one")],
        ),
    ]
}

fn dataset(id: u128, name: &str, cases: Vec<(u128, &str)>) -> BenchmarkDataset {
    BenchmarkDataset::with_id(
        BenchmarkDatasetId::from_uuid(uuid(id)),
        name,
        cases
            .into_iter()
            .map(|(case_id, name)| {
                BenchmarkCase::with_id(
                    BenchmarkCaseId::from_uuid(uuid(case_id)),
                    name,
                    json!({"raw_case_input": format!("RAW_CASE_{case_id:x}")}),
                    BenchmarkExpectedOutput::Exact(json!({
                        "raw_expected_output": format!("RAW_EXPECTED_{case_id:x}")
                    })),
                )
                .expect("valid breadth case")
            })
            .collect(),
    )
    .expect("valid breadth dataset")
}

fn suite() -> BenchmarkSuite {
    BenchmarkSuite::with_id(
        BenchmarkSuiteId::from_uuid(uuid(SUITE)),
        "Breadth release gate",
        vec![
            BenchmarkDatasetId::from_uuid(uuid(DATASET_BETA)),
            BenchmarkDatasetId::from_uuid(uuid(DATASET_ALPHA)),
        ],
        vec![
            RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                .expect("latency threshold"),
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("accuracy threshold"),
        ],
    )
    .expect("valid breadth suite")
}

fn binding_command(
    commit_id: CommitId,
    binding_id: u128,
    label: &str,
    datasets: Vec<BenchmarkDataset>,
    suite: BenchmarkSuite,
) -> BenchmarkDefinitionBindingCommand {
    BenchmarkDefinitionBindingCommand::new(
        principal(),
        uuid(binding_id),
        ProjectId::from_uuid(uuid(PROJECT)),
        ContextId::from_uuid(uuid(CONTEXT)),
        commit_id,
        BranchName::new("main").expect("branch"),
        commit_id,
        format!("breadth-binding-{label}"),
        format!("sha256:breadth-binding-{label}"),
        datasets,
        suite,
        timestamp(),
        BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
    )
    .expect("valid binding command")
}

fn repository(
    project_id: ProjectId,
    context_id: ContextId,
    baseline_commit: CommitId,
    revised_commit: CommitId,
) -> InMemoryContextGraphRepository {
    let created_at = timestamp();
    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id.to_string(),
            workspace_id: uuid(0x999).to_string(),
            name: "Breadth project".to_owned(),
            slug: "breadth-project".to_owned(),
            created_at,
        }],
        contexts: vec![ContextRecord {
            id: context_id.to_string(),
            project_id: project_id.to_string(),
            experiment_id: None,
            name: "Breadth context".to_owned(),
            description: None,
            created_at,
        }],
        commits: vec![
            commit_record(context_id, baseline_commit, created_at),
            commit_record(context_id, revised_commit, created_at),
        ],
        ..ContextGraphProjection::default()
    })
}

fn commit_record(
    context_id: ContextId,
    commit_id: CommitId,
    created_at: chrono::DateTime<Utc>,
) -> ContextCommitRecord {
    ContextCommitRecord {
        id: commit_id.to_string(),
        context_id: context_id.to_string(),
        branch_name: "main".to_owned(),
        message: "Benchmark breadth candidate".to_owned(),
        parent_commit_ids: Vec::new(),
        changes: json!([]),
        change_count: 0,
        authored_at: created_at,
        created_at,
    }
}

fn expected_calls(
    project_id: ProjectId,
    context_id: ContextId,
    baseline_commit: CommitId,
    revised_commit: CommitId,
) -> Vec<EvaluationCall> {
    [baseline_commit, revised_commit]
        .into_iter()
        .flat_map(|commit_id| {
            expected_case_projection()
                .into_iter()
                .map(move |(dataset_id, case_id)| EvaluationCall {
                    project_id,
                    context_id,
                    commit_id,
                    dataset_id,
                    case_id,
                })
        })
        .collect()
}

fn expected_case_projection() -> Vec<(BenchmarkDatasetId, BenchmarkCaseId)> {
    vec![
        (
            BenchmarkDatasetId::from_uuid(uuid(DATASET_ALPHA)),
            BenchmarkCaseId::from_uuid(uuid(ALPHA_CASE_ONE)),
        ),
        (
            BenchmarkDatasetId::from_uuid(uuid(DATASET_ALPHA)),
            BenchmarkCaseId::from_uuid(uuid(ALPHA_CASE_TWO)),
        ),
        (
            BenchmarkDatasetId::from_uuid(uuid(DATASET_BETA)),
            BenchmarkCaseId::from_uuid(uuid(BETA_CASE_ONE)),
        ),
        (
            BenchmarkDatasetId::from_uuid(uuid(DATASET_BETA)),
            BenchmarkCaseId::from_uuid(uuid(BETA_CASE_TWO)),
        ),
    ]
}

fn assert_metric_metadata(
    projection: &contextlab_evaluation::BenchmarkWorkspaceProjectionV1,
    accuracy: f64,
    latency: f64,
) {
    let metrics = projection.scorecard().metrics();
    let latency_metric = metrics
        .iter()
        .find(|metric| metric.metric() == MetricKind::LatencyMs)
        .expect("latency scorecard metadata");
    assert_eq!(
        latency_metric.threshold_direction(),
        ThresholdDirection::Maximum
    );
    assert_eq!(latency_metric.threshold_value(), 800.0);
    assert_eq!(latency_metric.observed(), Some(latency));
    assert_eq!(latency_metric.sample_count(), 4);
    assert_eq!(latency_metric.required_sample_count(), 4);
    assert!(latency_metric.has_complete_coverage());

    let accuracy_metric = metrics
        .iter()
        .find(|metric| metric.metric() == MetricKind::Accuracy)
        .expect("accuracy scorecard metadata");
    assert_eq!(
        accuracy_metric.threshold_direction(),
        ThresholdDirection::Minimum
    );
    assert_eq!(accuracy_metric.threshold_value(), 0.9);
    assert_eq!(accuracy_metric.observed(), Some(accuracy));
    assert_eq!(accuracy_metric.sample_count(), 4);
    assert_eq!(accuracy_metric.required_sample_count(), 4);
    assert!(accuracy_metric.has_complete_coverage());
}

fn principal() -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(PrincipalIdentity::new(
        IdentitySourceId::new("https://issuer.contextlab.test").expect("identity source"),
        PrincipalId::new("user:benchmark-breadth").expect("principal id"),
    ))
}

fn timestamp() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0)
        .single()
        .expect("timestamp")
}

const fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}
