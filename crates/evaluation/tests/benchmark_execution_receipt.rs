//! Contract tests for provider-free benchmark execution receipts.

use chrono::{TimeZone, Utc};
use contextlab_context_core::ContextId;
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkDecisionComparisonError, BenchmarkDecisionDiff,
    BenchmarkExecutionError, BenchmarkExecutionPlan, BenchmarkExecutionReceipt,
    BenchmarkExecutionReceiptError, BenchmarkExecutionReceiptVersion, BenchmarkExpectedOutput,
    BenchmarkSuite, MetricKind, MetricMeasurement, RegressionThreshold, ThresholdDirection,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn receipt_is_versioned_and_stable_under_result_permutation() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let context_id = ContextId::from_uuid(Uuid::from_u128(3));
    let decision_namespace = Uuid::from_u128(100);
    let executed_at = Utc
        .with_ymd_and_hms(2026, 7, 19, 0, 0, 0)
        .single()
        .expect("timestamp");

    let first = BenchmarkExecutionReceipt::from_plan(
        &plan,
        decision_namespace,
        context_id,
        "fixture-model",
        0.2,
        executed_at,
        "sha256:fixture",
        vec![
            result(second_dataset_id, second_case_id, 0.96, 710.0),
            result(first_dataset_id, first_case_id, 0.94, 700.0),
        ],
    )
    .expect("receipt");
    let repeated = BenchmarkExecutionReceipt::from_plan(
        &plan,
        decision_namespace,
        context_id,
        "fixture-model",
        0.2,
        executed_at,
        "sha256:fixture",
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    )
    .expect("repeated receipt");

    assert_eq!(first.version(), BenchmarkExecutionReceiptVersion::V1);
    assert_eq!(first.schema_version(), 1);
    assert_eq!(first.cohort_id(), repeated.cohort_id());
    assert_eq!(first.cohort(), repeated.cohort());
    assert_eq!(
        first
            .cohort()
            .entries()
            .iter()
            .map(|entry| (entry.key().dataset_id(), entry.key().case_id()))
            .collect::<Vec<_>>(),
        vec![
            (first_dataset_id, first_case_id),
            (second_dataset_id, second_case_id),
        ]
    );
    assert_eq!(first.evaluation(), repeated.evaluation());
    assert_eq!(first.decision_input(), repeated.decision_input());
}

#[test]
fn receipt_projects_evaluation_into_existing_decision_diff_input() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let receipt = BenchmarkExecutionReceipt::from_plan(
        &plan,
        Uuid::from_u128(101),
        ContextId::from_uuid(Uuid::from_u128(4)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 7, 19, 0, 0, 0)
            .single()
            .expect("timestamp"),
        "sha256:fixture",
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    )
    .expect("receipt");

    assert_eq!(
        receipt.decision_input().status(),
        receipt.evaluation().decision().status()
    );
    assert_eq!(
        receipt
            .decision_input()
            .metrics()
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        vec![MetricKind::LatencyMs, MetricKind::Accuracy]
    );
    let diff = BenchmarkDecisionDiff::between(
        receipt.decision_input().clone(),
        receipt.decision_input().clone(),
    )
    .expect("same decision input is comparable");
    assert_eq!(diff.status_change(), None);
    assert!(diff.metric_changes().is_empty());
}

#[test]
fn receipt_fails_closed_with_typed_plan_and_decision_errors() {
    let (plan, first_dataset_id, second_dataset_id, first_case_id, second_case_id) = plan();
    let common = (
        &plan,
        Uuid::from_u128(102),
        ContextId::from_uuid(Uuid::from_u128(5)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 7, 19, 0, 0, 0)
            .single()
            .expect("timestamp"),
    );

    let decision_error = BenchmarkExecutionReceipt::from_plan(
        common.0,
        common.1,
        common.2,
        common.3,
        common.4,
        common.5,
        "   ",
        vec![
            result(first_dataset_id, first_case_id, 0.94, 700.0),
            result(second_dataset_id, second_case_id, 0.96, 710.0),
        ],
    )
    .expect_err("empty fingerprint must fail closed");
    assert!(matches!(
        decision_error,
        BenchmarkExecutionReceiptError::DecisionInput(
            BenchmarkDecisionComparisonError::EmptyComparabilityFingerprint
        )
    ));

    let plan_error = BenchmarkExecutionReceipt::from_plan(
        common.0,
        common.1,
        common.2,
        common.3,
        common.4,
        common.5,
        "sha256:fixture",
        Vec::new(),
    )
    .expect_err("missing planned result must fail closed");
    assert!(matches!(
        plan_error,
        BenchmarkExecutionReceiptError::Execution(
            BenchmarkExecutionError::MissingCaseResult { .. }
        )
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
                .expect("threshold"),
            RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                .expect("threshold"),
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
                json!({"fixture": name}),
                BenchmarkExpectedOutput::Unspecified,
            )
            .expect("case"),
        ],
    )
    .expect("dataset")
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
    .expect("result")
}
