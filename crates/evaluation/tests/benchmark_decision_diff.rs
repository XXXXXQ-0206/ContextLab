//! Regression coverage for pure benchmark decision comparison.

use contextlab_evaluation::{
    BenchmarkDecisionComparisonInput, BenchmarkDecisionDiff, BenchmarkDecisionMetricDiff,
    BenchmarkDecisionMetricInput, MetricKind, RegressionCheckStatus, RegressionDecisionStatus,
    RegressionThreshold, ThresholdDirection,
};

#[allow(clippy::too_many_arguments)]
fn metric(
    kind: MetricKind,
    threshold_direction: ThresholdDirection,
    threshold_value: f64,
    observed: Option<f64>,
    sample_count: usize,
    required_sample_count: usize,
    has_complete_coverage: bool,
    outcome: RegressionCheckStatus,
) -> BenchmarkDecisionMetricInput {
    BenchmarkDecisionMetricInput::new(
        RegressionThreshold::new(kind, threshold_direction, threshold_value)
            .expect("valid threshold"),
        observed,
        sample_count,
        required_sample_count,
        has_complete_coverage,
        outcome,
    )
    .expect("valid metric input")
}

fn decision(
    status: RegressionDecisionStatus,
    fingerprint: &str,
    metrics: Vec<BenchmarkDecisionMetricInput>,
) -> BenchmarkDecisionComparisonInput {
    BenchmarkDecisionComparisonInput::new(status, fingerprint, metrics)
        .expect("valid decision input")
}

#[test]
fn compares_status_and_metric_evidence_in_stable_metric_order() {
    let baseline = decision(
        RegressionDecisionStatus::Passed,
        "sha256:shared",
        vec![
            metric(
                MetricKind::Accuracy,
                ThresholdDirection::Minimum,
                0.9,
                Some(0.94),
                3,
                3,
                true,
                RegressionCheckStatus::Passed,
            ),
            metric(
                MetricKind::LatencyMs,
                ThresholdDirection::Maximum,
                300.0,
                Some(250.0),
                3,
                3,
                true,
                RegressionCheckStatus::Passed,
            ),
        ],
    );
    let revised = decision(
        RegressionDecisionStatus::Regressed,
        "sha256:shared",
        vec![
            metric(
                MetricKind::Accuracy,
                ThresholdDirection::Minimum,
                0.9,
                Some(0.87),
                2,
                3,
                false,
                RegressionCheckStatus::InsufficientData,
            ),
            metric(
                MetricKind::OutputQuality,
                ThresholdDirection::Minimum,
                0.8,
                Some(0.76),
                3,
                3,
                true,
                RegressionCheckStatus::Regressed,
            ),
        ],
    );

    let diff = BenchmarkDecisionDiff::between(baseline, revised).expect("comparable decisions");

    assert_eq!(
        diff.status_change(),
        Some((
            RegressionDecisionStatus::Passed,
            RegressionDecisionStatus::Regressed
        ))
    );
    assert_eq!(diff.metric_changes().len(), 3);
    assert_eq!(diff.metric_changes()[0].metric(), MetricKind::LatencyMs);
    assert_eq!(diff.metric_changes()[1].metric(), MetricKind::Accuracy);
    assert_eq!(diff.metric_changes()[2].metric(), MetricKind::OutputQuality);
    assert!(matches!(
        diff.metric_changes()[0],
        BenchmarkDecisionMetricDiff::Removed { .. }
    ));
    assert!(matches!(
        diff.metric_changes()[1],
        BenchmarkDecisionMetricDiff::Modified { .. }
    ));
    assert!(matches!(
        diff.metric_changes()[2],
        BenchmarkDecisionMetricDiff::Added { .. }
    ));
}

#[test]
fn rejects_duplicate_metric_inputs_before_comparison() {
    let duplicate = BenchmarkDecisionComparisonInput::new(
        RegressionDecisionStatus::Passed,
        "sha256:shared",
        vec![
            metric(
                MetricKind::Accuracy,
                ThresholdDirection::Minimum,
                0.9,
                Some(0.94),
                1,
                1,
                true,
                RegressionCheckStatus::Passed,
            ),
            metric(
                MetricKind::Accuracy,
                ThresholdDirection::Minimum,
                0.9,
                Some(0.94),
                1,
                1,
                true,
                RegressionCheckStatus::Passed,
            ),
        ],
    );

    assert!(duplicate.is_err());
}

#[test]
fn rejects_decisions_with_different_comparability_fingerprints() {
    let baseline = decision(
        RegressionDecisionStatus::Passed,
        "sha256:baseline",
        vec![metric(
            MetricKind::Accuracy,
            ThresholdDirection::Minimum,
            0.9,
            Some(0.94),
            1,
            1,
            true,
            RegressionCheckStatus::Passed,
        )],
    );
    let revised = decision(
        RegressionDecisionStatus::Regressed,
        "sha256:revised",
        vec![metric(
            MetricKind::Accuracy,
            ThresholdDirection::Minimum,
            0.9,
            Some(0.84),
            1,
            1,
            true,
            RegressionCheckStatus::Regressed,
        )],
    );

    assert!(BenchmarkDecisionDiff::between(baseline, revised).is_err());
}
