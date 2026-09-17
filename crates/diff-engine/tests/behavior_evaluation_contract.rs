//! Behavior, evaluation, error, and serialization contract tests.

use contextlab_diff_engine::{
    BehaviorObservationV1, BehaviorOutcomeV1, BehaviorSnapshotV1, ContextDiffError,
    ContextDiffRequestV1, ContextDiffResultV1, ContextDiffService, ContextDiffSnapshotV1,
    DiffInputError, EvaluationMetricObservationV1, EvaluationSnapshotV1, SemanticSnapshotV1,
};
use contextlab_graph::ContextGraph;

fn snapshot(
    behavior: Vec<BehaviorObservationV1>,
    evaluation_fingerprint: &str,
    evaluation: Vec<EvaluationMetricObservationV1>,
) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(ContextGraph::new(), Vec::new()).expect("semantic snapshot"),
        BehaviorSnapshotV1::new(behavior).expect("behavior snapshot"),
        EvaluationSnapshotV1::new(evaluation_fingerprint, evaluation).expect("evaluation snapshot"),
    )
    .expect("complete snapshot")
}

#[test]
fn returns_behavior_and_evaluation_changes_in_stable_identifier_order() {
    let original = snapshot(
        vec![
            BehaviorObservationV1::new(
                "case:z",
                "sha256:input-z",
                BehaviorOutcomeV1::succeeded("original"),
            )
            .expect("case z"),
            BehaviorObservationV1::new(
                "case:a",
                "sha256:input-a",
                BehaviorOutcomeV1::succeeded("removed"),
            )
            .expect("case a"),
        ],
        "suite:customer-support:v1",
        vec![
            EvaluationMetricObservationV1::new("latency_ms", 120.0, 5).expect("latency"),
            EvaluationMetricObservationV1::new("accuracy", 0.8, 5).expect("accuracy"),
        ],
    );
    let revised = snapshot(
        vec![
            BehaviorObservationV1::new(
                "case:b",
                "sha256:input-b",
                BehaviorOutcomeV1::succeeded("added"),
            )
            .expect("case b"),
            BehaviorObservationV1::new(
                "case:z",
                "sha256:input-z",
                BehaviorOutcomeV1::failed("provider_timeout").expect("failure outcome"),
            )
            .expect("case z"),
        ],
        "suite:customer-support:v1",
        vec![
            EvaluationMetricObservationV1::new("cost_usd", 0.25, 5).expect("cost"),
            EvaluationMetricObservationV1::new("accuracy", 0.9, 5).expect("accuracy"),
        ],
    );

    let result = ContextDiffService::compare(
        ContextDiffRequestV1::new(original, revised).expect("comparison request"),
    )
    .expect("comparable snapshots");

    assert_eq!(
        result
            .behavior()
            .case_changes()
            .iter()
            .map(|change| change.case_id().as_str())
            .collect::<Vec<_>>(),
        vec!["case:a", "case:b", "case:z"]
    );
    assert_eq!(
        result
            .evaluation()
            .metric_changes()
            .iter()
            .map(|change| change.metric_id().as_str())
            .collect::<Vec<_>>(),
        vec!["accuracy", "cost_usd", "latency_ms"]
    );

    let serialized = serde_json::to_value(&result).expect("stable result json");
    assert_eq!(serialized["contract_version"], "v1");
    let restored: ContextDiffResultV1 =
        serde_json::from_value(serialized).expect("result json round trip");
    assert_eq!(restored, result);
}

#[test]
fn fails_closed_when_a_behavior_case_changes_its_input_fingerprint() {
    let original = snapshot(
        vec![
            BehaviorObservationV1::new(
                "case:refund",
                "sha256:original-input",
                BehaviorOutcomeV1::succeeded("approved"),
            )
            .expect("original case"),
        ],
        "suite:customer-support:v1",
        Vec::new(),
    );
    let revised = snapshot(
        vec![
            BehaviorObservationV1::new(
                "case:refund",
                "sha256:revised-input",
                BehaviorOutcomeV1::succeeded("approved"),
            )
            .expect("revised case"),
        ],
        "suite:customer-support:v1",
        Vec::new(),
    );

    let error = ContextDiffService::compare(
        ContextDiffRequestV1::new(original, revised).expect("comparison request"),
    )
    .expect_err("same case identity with a new input is not comparable");

    assert!(matches!(
        error,
        ContextDiffError::BehaviorInputMismatch { case_id, .. }
            if case_id.as_str() == "case:refund"
    ));
}

#[test]
fn fails_closed_when_evaluation_conditions_do_not_match() {
    let original = snapshot(Vec::new(), "suite:customer-support:v1", Vec::new());
    let revised = snapshot(Vec::new(), "suite:customer-support:v2", Vec::new());

    let error = ContextDiffService::compare(
        ContextDiffRequestV1::new(original, revised).expect("comparison request"),
    )
    .expect_err("different evaluation conditions are not comparable");

    assert!(matches!(
        error,
        ContextDiffError::EvaluationComparabilityMismatch {
            original_fingerprint,
            revised_fingerprint,
        } if original_fingerprint.as_str() == "suite:customer-support:v1"
            && revised_fingerprint.as_str() == "suite:customer-support:v2"
    ));
}

#[test]
fn rejects_non_finite_evaluation_values_before_comparison() {
    let error = EvaluationMetricObservationV1::new("accuracy", f64::NAN, 5)
        .expect_err("NaN metric must be rejected");

    assert!(matches!(
        error,
        DiffInputError::NonFiniteEvaluationMetric { metric_id }
            if metric_id.as_str() == "accuracy"
    ));
}
