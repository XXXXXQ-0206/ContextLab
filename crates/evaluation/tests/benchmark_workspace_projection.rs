//! Contract tests for the provider-free benchmark workspace projection.

use chrono::{TimeZone, Utc};
use contextlab_context_core::ContextId;
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkEvaluationMetricChangeKindV1, BenchmarkExecutionPlan,
    BenchmarkExecutionReceipt, BenchmarkExpectedOutput, BenchmarkSuite,
    BenchmarkWorkspaceProjectionV1, MetricKind, MetricMeasurement, RegressionCheckStatus,
    RegressionDecisionStatus, RegressionThreshold, ThresholdDirection,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn projects_safe_v1_workspace_facts_with_stable_ordering() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let receipt = receipt(
        &plan,
        vec![
            result(second_dataset_id, second_case_id, 0.96, 710.0),
            result(first_dataset_id, first_case_id, 0.94, 700.0),
        ],
    );

    let projection = BenchmarkWorkspaceProjectionV1::from_receipt(&receipt, &plan)
        .expect("valid workspace projection");

    assert_eq!(projection.schema_version(), 1);
    assert_eq!(projection.suite().id(), plan.suite().id());
    assert_eq!(projection.suite().name(), "Release gate");
    assert_eq!(
        projection
            .datasets()
            .iter()
            .map(|dataset| (dataset.id(), dataset.name(), dataset.case_count()))
            .collect::<Vec<_>>(),
        vec![
            (first_dataset_id, "First", 1),
            (second_dataset_id, "Second", 1),
        ]
    );
    assert_eq!(
        projection
            .runs()
            .iter()
            .map(|run| (run.dataset_id(), run.case_id(), run.metric_count()))
            .collect::<Vec<_>>(),
        vec![
            (first_dataset_id, first_case_id, 2),
            (second_dataset_id, second_case_id, 2)
        ]
    );
    assert_eq!(projection.scorecard().run_count(), 2);
    assert_eq!(
        projection
            .scorecard()
            .metrics()
            .iter()
            .map(|metric| {
                (
                    metric.metric(),
                    metric.observed(),
                    metric.sample_count(),
                    metric.required_sample_count(),
                    metric.has_complete_coverage(),
                    metric.outcome(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (
                MetricKind::LatencyMs,
                Some(705.0),
                2,
                2,
                true,
                RegressionCheckStatus::Passed,
            ),
            (
                MetricKind::Accuracy,
                Some(0.95),
                2,
                2,
                true,
                RegressionCheckStatus::Passed,
            ),
        ]
    );
    assert_eq!(
        projection.regression_status(),
        RegressionDecisionStatus::Passed
    );

    let serialized = serde_json::to_string(&projection).expect("serialize projection");
    assert!(serialized.contains("\"schema_version\":1"));
    for forbidden in [
        "cases",
        "input",
        "expected_output",
        "output",
        "measurements",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "raw field leaked: {forbidden}"
        );
    }
}

#[test]
fn projection_is_identical_when_sealed_results_are_permuted() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let first = receipt(
        &plan,
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    );
    let second = receipt(
        &plan,
        vec![
            result(second_dataset_id, second_case_id, 0.96, 710.0),
            result(first_dataset_id, first_case_id, 0.94, 700.0),
        ],
    );

    let first_projection =
        BenchmarkWorkspaceProjectionV1::from_receipt(&first, &plan).expect("first projection");
    let second_projection =
        BenchmarkWorkspaceProjectionV1::from_receipt(&second, &plan).expect("second projection");

    assert_eq!(first_projection, second_projection);
    assert_eq!(
        serde_json::to_string(&first_projection).expect("serialize first"),
        serde_json::to_string(&second_projection).expect("serialize second")
    );
}

#[test]
fn projection_preserves_the_sealed_receipt_identity_for_local_reads() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let first = receipt(
        &plan,
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    );
    let second = BenchmarkExecutionReceipt::from_plan(
        &plan,
        Uuid::from_u128(100),
        ContextId::from_uuid(Uuid::from_u128(3)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 7, 19, 0, 1, 0)
            .single()
            .expect("timestamp"),
        "sha256:fixture",
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    )
    .expect("second execution receipt");

    let first_projection =
        BenchmarkWorkspaceProjectionV1::from_receipt(&first, &plan).expect("first projection");
    let second_projection =
        BenchmarkWorkspaceProjectionV1::from_receipt(&second, &plan).expect("second projection");

    assert_ne!(first.cohort_id(), second.cohort_id());
    assert_eq!(first_projection.receipt().cohort_id(), first.cohort_id());
    assert_eq!(second_projection.receipt().cohort_id(), second.cohort_id());

    let serialized = serde_json::to_string(&first_projection).expect("serialize projection");
    assert!(serialized.contains("\"cohort_id\""));
    assert!(!serialized.contains("decision_namespace"));
}

#[test]
fn projection_composes_the_existing_evaluation_diff_as_redacted_ordered_evidence() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let baseline = receipt(
        &plan,
        vec![
            result(first_dataset_id, first_case_id, 0.88, 900.0),
            result(second_dataset_id, second_case_id, 0.90, 850.0),
        ],
    );
    let revised = BenchmarkExecutionReceipt::from_plan(
        &plan,
        Uuid::from_u128(101),
        ContextId::from_uuid(Uuid::from_u128(3)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 7, 19, 0, 1, 0)
            .single()
            .expect("timestamp"),
        "sha256:fixture",
        vec![
            result(second_dataset_id, second_case_id, 0.96, 710.0),
            result(first_dataset_id, first_case_id, 0.94, 700.0),
        ],
    )
    .expect("revised execution receipt");

    let projection = BenchmarkWorkspaceProjectionV1::from_receipts(&baseline, &revised, &plan)
        .expect("comparable workspace projection");
    let evaluation_diff = projection
        .evaluation_diff()
        .expect("baseline comparison should be present");

    assert_eq!(
        evaluation_diff.status_change(),
        Some((
            RegressionDecisionStatus::Regressed,
            RegressionDecisionStatus::Passed,
        ))
    );
    assert_eq!(
        evaluation_diff
            .metric_changes()
            .iter()
            .map(|change| (
                change.metric(),
                change.change_kind(),
                change.baseline().map(|evidence| evidence.outcome()),
                change.revised().map(|evidence| evidence.outcome()),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                MetricKind::LatencyMs,
                BenchmarkEvaluationMetricChangeKindV1::Modified,
                Some(RegressionCheckStatus::Regressed),
                Some(RegressionCheckStatus::Passed),
            ),
            (
                MetricKind::Accuracy,
                BenchmarkEvaluationMetricChangeKindV1::Modified,
                Some(RegressionCheckStatus::Regressed),
                Some(RegressionCheckStatus::Passed),
            ),
        ]
    );

    let serialized = serde_json::to_string(&projection).expect("serialize projection");
    for forbidden in [
        "cases",
        "input",
        "expected_output",
        "output",
        "measurements",
        "model_version",
        "temperature",
    ] {
        assert!(
            !serialized.contains(forbidden),
            "raw field leaked: {forbidden}"
        );
    }
}

#[test]
fn projection_fails_closed_on_revised_scope_before_comparison() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let baseline = receipt(
        &plan,
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    );
    let other_plan = BenchmarkExecutionPlan::new(
        BenchmarkSuite::with_id(
            contextlab_evaluation::BenchmarkSuiteId::from_uuid(Uuid::from_u128(99)),
            "Other suite",
            vec![first_dataset_id, second_dataset_id],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
            ],
        )
        .expect("other suite"),
        vec![
            dataset(second_dataset_id, second_case_id, "Second"),
            dataset(first_dataset_id, first_case_id, "First"),
        ],
    )
    .expect("other execution plan");
    let mismatched_revised = BenchmarkExecutionReceipt::from_plan(
        &other_plan,
        Uuid::from_u128(101),
        ContextId::from_uuid(Uuid::from_u128(3)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 7, 19, 0, 1, 0)
            .single()
            .expect("timestamp"),
        "sha256:incomparable",
        vec![
            result(second_dataset_id, second_case_id, 0.96, 710.0),
            result(first_dataset_id, first_case_id, 0.94, 700.0),
        ],
    )
    .expect("mismatched receipt");

    let error =
        BenchmarkWorkspaceProjectionV1::from_receipts(&baseline, &mismatched_revised, &plan)
            .expect_err("revised receipt scope must fail before comparison");

    assert!(matches!(
        error,
        contextlab_evaluation::BenchmarkWorkspaceProjectionError::SuiteMismatch {
            receipt_suite_id,
            plan_suite_id,
        } if receipt_suite_id == other_plan.suite().id() && plan_suite_id == plan.suite().id()
    ));
}

#[test]
fn projection_fails_closed_when_same_suite_id_has_definition_drift() {
    let (sealed_plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let sealed_receipt = receipt(
        &sealed_plan,
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    );
    let drifted_plan = BenchmarkExecutionPlan::new(
        BenchmarkSuite::with_id(
            sealed_plan.suite().id(),
            "Unrelated release gate",
            vec![first_dataset_id, second_dataset_id],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
            ],
        )
        .expect("drifted suite"),
        vec![
            dataset(second_dataset_id, second_case_id, "Unrelated second"),
            dataset(first_dataset_id, first_case_id, "Unrelated first"),
        ],
    )
    .expect("drifted execution plan");

    let error = BenchmarkWorkspaceProjectionV1::from_receipt(&sealed_receipt, &drifted_plan)
        .expect_err("same suite UUID must not authorize unrelated plan metadata");

    assert!(matches!(
        error,
        contextlab_evaluation::BenchmarkWorkspaceProjectionError::PlanDefinitionMismatch {
            suite_id,
        } if suite_id == sealed_plan.suite().id()
    ));
}

#[test]
fn projection_fails_closed_when_baseline_and_revised_are_the_same_run() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let receipt = receipt(
        &plan,
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    );

    let error = BenchmarkWorkspaceProjectionV1::from_receipts(&receipt, &receipt, &plan)
        .expect_err("a workspace comparison must contain two distinct sealed runs");

    assert!(matches!(
        error,
        contextlab_evaluation::BenchmarkWorkspaceProjectionError::IdenticalReceiptCohort {
            cohort_id
        } if cohort_id == receipt.cohort_id()
    ));
}

fn plan() -> (
    BenchmarkExecutionPlan,
    BenchmarkDatasetId,
    BenchmarkDatasetId,
    BenchmarkCaseId,
    BenchmarkCaseId,
) {
    let first_dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let second_dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(2));
    let first_case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(10));
    let second_case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(20));
    let first_dataset = dataset(first_dataset_id, first_case_id, "First");
    let second_dataset = dataset(second_dataset_id, second_case_id, "Second");
    let suite = BenchmarkSuite::new(
        "Release gate",
        vec![second_dataset_id, first_dataset_id],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("accuracy threshold"),
            RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                .expect("latency threshold"),
        ],
    )
    .expect("suite");

    (
        BenchmarkExecutionPlan::new(suite, vec![second_dataset, first_dataset])
            .expect("execution plan"),
        first_dataset_id,
        second_dataset_id,
        first_case_id,
        second_case_id,
    )
}

fn dataset(
    dataset_id: BenchmarkDatasetId,
    case_id: BenchmarkCaseId,
    name: &str,
) -> BenchmarkDataset {
    BenchmarkDataset::with_id(
        dataset_id,
        name,
        vec![
            BenchmarkCase::with_id(
                case_id,
                format!("{name} case"),
                json!({"secret_input": name}),
                BenchmarkExpectedOutput::Exact(json!({"secret_output": name})),
            )
            .expect("case"),
        ],
    )
    .expect("dataset")
}

fn receipt(
    plan: &BenchmarkExecutionPlan,
    results: Vec<BenchmarkCaseExecutionResult>,
) -> BenchmarkExecutionReceipt {
    BenchmarkExecutionReceipt::from_plan(
        plan,
        Uuid::from_u128(100),
        ContextId::from_uuid(Uuid::from_u128(3)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 7, 19, 0, 0, 0)
            .single()
            .expect("timestamp"),
        "sha256:fixture",
        results,
    )
    .expect("execution receipt")
}

fn result(
    dataset_id: BenchmarkDatasetId,
    case_id: BenchmarkCaseId,
    accuracy: f64,
    latency: f64,
) -> BenchmarkCaseExecutionResult {
    BenchmarkCaseExecutionResult::new(
        dataset_id,
        case_id,
        vec![
            MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("accuracy"),
            MetricMeasurement::new(MetricKind::LatencyMs, latency).expect("latency"),
        ],
    )
    .expect("case result")
}
