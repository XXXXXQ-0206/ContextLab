//! Integration coverage for reusable benchmark regression decisions.

use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseId, BenchmarkDataset, BenchmarkDatasetId, BenchmarkEvaluation,
    BenchmarkExpectedOutput, BenchmarkSuite, EvaluationRun, MetricKind, MetricMeasurement,
    RegressionCheckStatus, RegressionDecisionStatus, RegressionThreshold, Scorecard,
    ThresholdDirection,
};
use serde_json::json;
use uuid::Uuid;

macro_rules! evaluation_run {
    ($($argument:expr),* $(,)?) => {
        EvaluationRun::new($($argument),*).expect("valid evaluation run")
    };
}

#[test]
fn typed_ids_round_trip_existing_uuids() {
    let value = Uuid::parse_str("00000000-0000-0000-0000-000000000099").expect("uuid");

    assert_eq!(BenchmarkCaseId::from_uuid(value).as_uuid(), value);
    assert_eq!(BenchmarkDatasetId::from_uuid(value).as_uuid(), value);
}

#[test]
fn benchmark_names_and_empty_threshold_sets_are_rejected() {
    assert!(BenchmarkCase::new("   ", json!({}), BenchmarkExpectedOutput::Unspecified,).is_err());
    let case =
        BenchmarkCase::new("Case", json!({}), BenchmarkExpectedOutput::Unspecified).expect("case");
    assert!(BenchmarkDataset::new("   ", vec![case]).is_err());
    assert!(BenchmarkSuite::new("Suite", vec![BenchmarkDatasetId::new()], Vec::new()).is_err());
}

#[test]
fn suite_normalizes_dataset_membership_and_exposes_identity() {
    let lower = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let higher = BenchmarkDatasetId::from_uuid(Uuid::from_u128(2));
    let suite_id = contextlab_evaluation::BenchmarkSuiteId::from_uuid(Uuid::from_u128(3));
    let suite = BenchmarkSuite::with_id(
        suite_id,
        "Suite",
        vec![higher, lower],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");

    assert_eq!(suite.id(), suite_id);
    assert_eq!(suite.name(), "Suite");
    assert_eq!(suite.dataset_ids(), &[lower, higher]);
    assert_eq!(suite.thresholds().len(), 1);
}

#[test]
fn dataset_preserves_validated_cases_in_stable_identifier_order() {
    let first_id = BenchmarkCaseId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("uuid"),
    );
    let second_id = BenchmarkCaseId::from_uuid(
        uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").expect("uuid"),
    );
    let first = BenchmarkCase::with_id(
        first_id,
        "Refund policy",
        json!({"question": "Can I return this?"}),
        BenchmarkExpectedOutput::Exact(json!({"decision": "eligible"})),
    )
    .expect("case");
    let second = BenchmarkCase::with_id(
        second_id,
        "Escalation",
        json!({"question": "I need a specialist"}),
        BenchmarkExpectedOutput::Unspecified,
    )
    .expect("case");

    let dataset = BenchmarkDataset::with_id(
        BenchmarkDatasetId::new(),
        "Support regression",
        vec![second, first],
    )
    .expect("dataset");

    assert_eq!(
        dataset
            .cases()
            .iter()
            .map(BenchmarkCase::id)
            .collect::<Vec<_>>(),
        vec![first_id, second_id]
    );
    assert_eq!(dataset.cases()[0].name(), "Refund policy");
    assert_eq!(
        dataset.cases()[0].input(),
        &json!({"question": "Can I return this?"})
    );
    assert_eq!(
        dataset.cases()[0].expected_output(),
        &BenchmarkExpectedOutput::Exact(json!({"decision": "eligible"}))
    );
}

#[test]
fn dataset_rejects_empty_and_duplicate_case_membership() {
    assert!(BenchmarkDataset::new("Empty", Vec::new()).is_err());

    let case = BenchmarkCase::new(
        "Case",
        json!({"input": 1}),
        BenchmarkExpectedOutput::Unspecified,
    )
    .expect("case");
    let duplicate = case.clone();
    assert!(BenchmarkDataset::new("Duplicate", vec![case, duplicate]).is_err());
}

#[test]
fn suite_rejects_empty_duplicate_membership_and_duplicate_metrics() {
    let dataset_id = BenchmarkDatasetId::new();
    let accuracy = RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
        .expect("threshold");

    assert!(BenchmarkSuite::new("Empty", Vec::new(), vec![accuracy]).is_err());
    assert!(
        BenchmarkSuite::new(
            "Duplicate dataset",
            vec![dataset_id, dataset_id],
            vec![accuracy],
        )
        .is_err()
    );
    assert!(
        BenchmarkSuite::new(
            "Duplicate metric",
            vec![dataset_id],
            vec![
                accuracy,
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Maximum, 0.1,)
                    .expect("threshold"),
            ],
        )
        .is_err()
    );
}

#[test]
fn threshold_rejects_non_finite_values() {
    assert!(
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, f64::NAN,)
            .is_err()
    );
    assert!(
        RegressionThreshold::new(
            MetricKind::Accuracy,
            ThresholdDirection::Maximum,
            f64::INFINITY,
        )
        .is_err()
    );
    assert!(
        RegressionThreshold::new(
            MetricKind::Accuracy,
            ThresholdDirection::Maximum,
            f64::NEG_INFINITY,
        )
        .is_err()
    );
}

#[test]
fn decision_passes_inclusive_minimum_and_maximum_boundaries() {
    let scorecard = scorecard(&[(MetricKind::Accuracy, 0.9), (MetricKind::LatencyMs, 800.0)]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
            .expect("maximum"),
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("minimum"),
    ]);

    let decision = suite.evaluate(&scorecard);

    assert_eq!(decision.status(), RegressionDecisionStatus::Passed);
    assert!(decision.checks().iter().all(|check| check.passed()));
    assert_eq!(
        decision
            .checks()
            .iter()
            .map(|check| check.metric())
            .collect::<Vec<_>>(),
        vec![MetricKind::LatencyMs, MetricKind::Accuracy]
    );
}

#[test]
fn decision_reports_regression_when_any_complete_metric_breaches() {
    let scorecard = scorecard(&[(MetricKind::Accuracy, 0.89), (MetricKind::LatencyMs, 801.0)]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("minimum"),
        RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
            .expect("maximum"),
    ]);

    let decision = suite.evaluate(&scorecard);

    assert_eq!(decision.status(), RegressionDecisionStatus::Regressed);
    assert_eq!(decision.checks()[0].observed(), Some(801.0));
    assert!(!decision.checks()[0].passed());
    assert_eq!(decision.checks()[1].observed(), Some(0.89));
    assert!(!decision.checks()[1].passed());
}

#[test]
fn decision_fails_closed_when_a_required_metric_is_missing() {
    let scorecard = scorecard(&[(MetricKind::Accuracy, 0.95)]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("minimum"),
        RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
            .expect("maximum"),
    ]);

    let decision = suite.evaluate(&scorecard);

    assert_eq!(
        decision.status(),
        RegressionDecisionStatus::InsufficientData
    );
    assert_eq!(decision.checks()[0].observed(), None);
    assert!(!decision.checks()[0].passed());
}

#[test]
fn decision_fails_closed_when_a_metric_has_partial_run_coverage() {
    let context_id = contextlab_context_core::ContextId::new();
    let complete = evaluation_run!(
        context_id,
        "model-a",
        0.2,
        vec![
            MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("accuracy"),
            MetricMeasurement::new(MetricKind::LatencyMs, 700.0).expect("latency"),
        ],
        chrono::Utc::now(),
    );
    let missing_latency = evaluation_run!(
        context_id,
        "model-a",
        0.2,
        vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("accuracy")],
        chrono::Utc::now(),
    );
    let scorecard = Scorecard::from_runs(&[complete, missing_latency]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("minimum"),
        RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
            .expect("maximum"),
    ]);

    let decision = suite.evaluate(&scorecard);

    assert_eq!(scorecard.sample_count(MetricKind::LatencyMs), 1);
    assert_eq!(
        decision.status(),
        RegressionDecisionStatus::InsufficientData
    );
    assert!(!decision.checks()[0].has_complete_coverage());
}

#[test]
fn benchmark_evaluation_artifact_keeps_policy_calculation_inside_the_domain() {
    let context_id = contextlab_context_core::ContextId::new();
    let runs = [
        evaluation_run!(
            context_id,
            "model-a",
            0.2,
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("accuracy")],
            chrono::Utc::now(),
        ),
        evaluation_run!(
            context_id,
            "model-a",
            0.2,
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.96).expect("accuracy")],
            chrono::Utc::now(),
        ),
    ];
    let threshold =
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("threshold");
    let suite = suite(vec![threshold]);

    let evaluation = BenchmarkEvaluation::from_runs(&suite, &runs);
    let mut expected_run_ids = runs
        .iter()
        .map(contextlab_evaluation::EvaluationRun::id)
        .collect::<Vec<_>>();
    expected_run_ids.sort_unstable();

    assert_eq!(evaluation.suite_id(), suite.id());
    assert_eq!(evaluation.run_ids(), expected_run_ids);
    assert_eq!(evaluation.scorecard().sample_count(MetricKind::Accuracy), 2);
    assert_eq!(
        evaluation.decision().status(),
        RegressionDecisionStatus::Passed
    );
    assert_eq!(
        threshold.status_for_evidence(Some(0.9), true),
        RegressionCheckStatus::Passed
    );
}

#[test]
fn known_regression_takes_precedence_over_another_missing_metric() {
    let scorecard = scorecard(&[(MetricKind::Accuracy, 0.5)]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("minimum"),
        RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
            .expect("maximum"),
    ]);

    let decision = suite.evaluate(&scorecard);

    assert_eq!(decision.status(), RegressionDecisionStatus::Regressed);
    assert!(decision.checks().iter().any(|check| !check.passed()));
    assert!(
        decision
            .checks()
            .iter()
            .any(|check| !check.has_complete_coverage())
    );
}

#[test]
fn scorecard_avoids_overflow_for_large_finite_measurements() {
    let context_id = contextlab_context_core::ContextId::new();
    let runs = [
        evaluation_run!(
            context_id,
            "model-a",
            0.2,
            vec![MetricMeasurement::new(MetricKind::LatencyMs, f64::MAX).expect("measurement")],
            chrono::Utc::now(),
        ),
        evaluation_run!(
            context_id,
            "model-a",
            0.2,
            vec![MetricMeasurement::new(MetricKind::LatencyMs, f64::MAX).expect("measurement")],
            chrono::Utc::now(),
        ),
        evaluation_run!(
            context_id,
            "model-a",
            0.2,
            vec![MetricMeasurement::new(MetricKind::LatencyMs, f64::MAX).expect("measurement")],
            chrono::Utc::now(),
        ),
    ];
    let scorecard = Scorecard::from_runs(&runs);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Minimum, 1.0)
            .expect("minimum"),
    ]);

    let decision = suite.evaluate(&scorecard);

    assert_eq!(scorecard.average(MetricKind::LatencyMs), Some(f64::MAX));
    assert_eq!(decision.status(), RegressionDecisionStatus::Passed);
    assert!(decision.checks()[0].has_complete_coverage());
    assert_ne!(
        serde_json::to_value(&decision).expect("serialize")["checks"][0]["observed"],
        serde_json::Value::Null
    );
}

#[test]
fn scorecard_preserves_subnormal_metric_averages() {
    let value = f64::from_bits(1);
    let context_id = contextlab_context_core::ContextId::new();
    let build_run = || {
        evaluation_run!(
            context_id,
            "model-a",
            0.2,
            vec![MetricMeasurement::new(MetricKind::CostUsd, value).expect("measurement")],
            chrono::Utc::now(),
        )
    };
    let scorecard = Scorecard::from_runs(&[build_run(), build_run()]);

    assert_eq!(scorecard.average(MetricKind::CostUsd), Some(value));
}

#[test]
fn empty_scorecard_is_insufficient_data() {
    let scorecard = Scorecard::from_runs(&[]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("threshold"),
    ]);

    assert_eq!(
        suite.evaluate(&scorecard).status(),
        RegressionDecisionStatus::InsufficientData
    );
}

#[test]
fn scorecard_average_is_stable_under_run_permutation() {
    let context_id = contextlab_context_core::ContextId::new();
    let build_run = |value| {
        evaluation_run!(
            context_id,
            "model-a",
            0.2,
            vec![MetricMeasurement::new(MetricKind::CostUsd, value).expect("measurement")],
            chrono::Utc::now(),
        )
    };
    let first = Scorecard::from_runs(&[build_run(1.0e16), build_run(1.0), build_run(1.0)]);
    let second = Scorecard::from_runs(&[build_run(1.0), build_run(1.0), build_run(1.0e16)]);

    assert_eq!(
        first.average(MetricKind::CostUsd),
        second.average(MetricKind::CostUsd)
    );
}

#[test]
fn duplicate_metric_in_one_run_is_insufficient_data() {
    let scorecard = scorecard(&[(MetricKind::Accuracy, 0.9), (MetricKind::Accuracy, 0.95)]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("minimum"),
    ]);

    let decision = suite.evaluate(&scorecard);

    assert_eq!(
        decision.status(),
        RegressionDecisionStatus::InsufficientData
    );
    assert_eq!(scorecard.sample_count(MetricKind::Accuracy), 1);
    assert!(!decision.checks()[0].has_complete_coverage());
}

#[test]
fn duplicate_metric_cannot_mask_another_run_missing_that_metric() {
    let context_id = contextlab_context_core::ContextId::new();
    let duplicated = evaluation_run!(
        context_id,
        "model-a",
        0.2,
        vec![
            MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement"),
            MetricMeasurement::new(MetricKind::Accuracy, 0.96).expect("measurement"),
        ],
        chrono::Utc::now(),
    );
    let missing = evaluation_run!(context_id, "model-a", 0.2, Vec::new(), chrono::Utc::now(),);
    let scorecard = Scorecard::from_runs(&[duplicated, missing]);
    let suite = suite(vec![
        RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
            .expect("minimum"),
    ]);

    assert_eq!(
        suite.evaluate(&scorecard).status(),
        RegressionDecisionStatus::InsufficientData
    );
}

#[test]
fn benchmark_oracle_distinguishes_unspecified_from_exact_null() {
    let unspecified = BenchmarkCase::new(
        "No oracle",
        json!({"input": 1}),
        BenchmarkExpectedOutput::Unspecified,
    )
    .expect("case");
    let exact_null = BenchmarkCase::new(
        "Null oracle",
        json!({"input": 1}),
        BenchmarkExpectedOutput::Exact(serde_json::Value::Null),
    )
    .expect("case");

    assert_ne!(
        serde_json::to_value(unspecified).expect("serialize"),
        serde_json::to_value(exact_null).expect("serialize")
    );
}

fn suite(thresholds: Vec<RegressionThreshold>) -> BenchmarkSuite {
    BenchmarkSuite::new("Release gate", vec![BenchmarkDatasetId::new()], thresholds).expect("suite")
}

fn scorecard(metrics: &[(MetricKind, f64)]) -> Scorecard {
    let context_id = contextlab_context_core::ContextId::new();
    let measurements = metrics
        .iter()
        .map(|(kind, value)| MetricMeasurement::new(*kind, *value).expect("measurement"))
        .collect();
    let run = evaluation_run!(context_id, "model-a", 0.2, measurements, chrono::Utc::now(),);
    Scorecard::from_runs(&[run])
}
