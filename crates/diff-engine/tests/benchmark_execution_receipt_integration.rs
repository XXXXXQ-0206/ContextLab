//! Cross-crate integration for sealed benchmark decision evidence.

use chrono::{TimeZone, Utc};
use contextlab_context_core::ContextId;
use contextlab_diff_engine::{
    BehaviorSnapshotV1, ContextDiffRequestV1, ContextDiffService, ContextDiffSnapshotV1,
    DiffInputError, EvaluationMetricChangeV1, EvaluationSnapshotV1, SemanticSnapshotV1,
};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkExecutionPlan, BenchmarkExecutionReceipt, BenchmarkExpectedOutput,
    BenchmarkSuite, MetricKind, MetricMeasurement, RegressionDecisionStatus, RegressionThreshold,
    ThresholdDirection,
};
use contextlab_graph::ContextGraph;
use serde_json::json;
use uuid::Uuid;

#[test]
fn receipt_decision_input_flows_into_unified_context_diff_without_policy_re_evaluation() {
    let baseline = receipt(0.95);
    let revised = receipt(0.85);

    assert_eq!(
        baseline.decision_input().status(),
        RegressionDecisionStatus::Passed
    );
    assert_eq!(
        revised.decision_input().status(),
        RegressionDecisionStatus::Regressed
    );

    let result = ContextDiffService::compare(
        ContextDiffRequestV1::new(snapshot(&baseline), snapshot(&revised))
            .expect("valid unified diff request"),
    )
    .expect("comparable receipt projections");

    assert_eq!(
        result.evaluation().comparability_fingerprint().as_str(),
        "sha256:fixture"
    );
    assert_eq!(result.evaluation().metric_changes().len(), 1);
    match &result.evaluation().metric_changes()[0] {
        EvaluationMetricChangeV1::Modified { original, revised } => {
            assert_eq!(original.metric_id().as_str(), "accuracy");
            assert_eq!(original.value(), 0.95);
            assert_eq!(revised.value(), 0.85);
            assert_eq!(original.sample_count(), 1);
            assert_eq!(revised.sample_count(), 1);
        }
        change => panic!("expected modified accuracy, got {change:?}"),
    }
}

#[test]
fn decision_projection_fails_closed_when_observed_value_is_missing() {
    let error =
        EvaluationSnapshotV1::from_decision_metrics("sha256:fixture", vec![("accuracy", None, 0)])
            .expect_err("missing evidence cannot become an aggregate snapshot");

    assert_eq!(
        error,
        DiffInputError::MissingEvaluationMetricValue {
            metric_id: "accuracy".to_owned()
        }
    );
}

fn snapshot(receipt: &BenchmarkExecutionReceipt) -> ContextDiffSnapshotV1 {
    let evaluation = EvaluationSnapshotV1::from_decision_metrics(
        receipt.decision_input().comparability_fingerprint(),
        receipt
            .decision_input()
            .metrics()
            .iter()
            .map(|(metric, evidence)| {
                (
                    metric_id(*metric),
                    evidence.observed(),
                    evidence.sample_count(),
                )
            }),
    )
    .expect("receipt decision metrics are valid snapshot evidence");

    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(ContextGraph::new(), Vec::new()).expect("empty semantic snapshot"),
        BehaviorSnapshotV1::new(Vec::new()).expect("empty behavior snapshot"),
        evaluation,
    )
    .expect("valid unified diff snapshot")
}

fn receipt(accuracy: f64) -> BenchmarkExecutionReceipt {
    let dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(2));
    let dataset = BenchmarkDataset::with_id(
        dataset_id,
        "Fixture dataset",
        vec![
            BenchmarkCase::with_id(
                case_id,
                "Fixture case",
                json!({"fixture": "receipt"}),
                BenchmarkExpectedOutput::Unspecified,
            )
            .expect("benchmark case"),
        ],
    )
    .expect("benchmark dataset");
    let suite = BenchmarkSuite::new(
        "Fixture suite",
        vec![dataset_id],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("benchmark suite");
    let plan = BenchmarkExecutionPlan::new(suite, vec![dataset]).expect("execution plan");

    BenchmarkExecutionReceipt::from_plan(
        &plan,
        Uuid::from_u128(3),
        ContextId::from_uuid(Uuid::from_u128(4)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 7, 19, 0, 0, 0)
            .single()
            .expect("timestamp"),
        "sha256:fixture",
        vec![
            BenchmarkCaseExecutionResult::new(
                dataset_id,
                case_id,
                vec![MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("accuracy")],
            )
            .expect("case result"),
        ],
    )
    .expect("execution receipt")
}

fn metric_id(metric: MetricKind) -> &'static str {
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
