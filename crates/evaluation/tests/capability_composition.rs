//! Contract tests for binding sealed benchmark evidence to a provider-free capability snapshot.

use chrono::{TimeZone, Utc};
use contextlab_context_core::ContextId;
use contextlab_evaluation::{
    BenchmarkCapabilityCompositionError, BenchmarkCapabilityFactV1, BenchmarkCapabilityKindV1,
    BenchmarkCapabilitySnapshotV1, BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId,
    BenchmarkDataset, BenchmarkDatasetId, BenchmarkEvaluationCapabilityCompositionV1,
    BenchmarkExecutionPlan, BenchmarkExecutionReceipt, BenchmarkExpectedOutput, BenchmarkSuite,
    MetricKind, MetricMeasurement, RegressionThreshold, ThresholdDirection,
};
use serde_json::json;
use uuid::Uuid;

fn receipt() -> BenchmarkExecutionReceipt {
    let case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(10));
    let dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let dataset = BenchmarkDataset::with_id(
        dataset_id,
        "fixture",
        vec![
            BenchmarkCase::with_id(
                case_id,
                "fixture case",
                json!({"fixture": "input"}),
                BenchmarkExpectedOutput::Unspecified,
            )
            .expect("case"),
        ],
    )
    .expect("dataset");
    let suite = BenchmarkSuite::new(
        "fixture suite",
        vec![dataset_id],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let plan = BenchmarkExecutionPlan::new(suite, vec![dataset]).expect("plan");
    BenchmarkExecutionReceipt::from_plan(
        &plan,
        Uuid::from_u128(100),
        ContextId::from_uuid(Uuid::from_u128(3)),
        "fixture-model",
        0.2,
        Utc.with_ymd_and_hms(2026, 8, 27, 0, 0, 0)
            .single()
            .expect("timestamp"),
        "sha256:fixture",
        vec![
            BenchmarkCaseExecutionResult::new(
                dataset_id,
                case_id,
                vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("metric")],
            )
            .expect("result"),
        ],
    )
    .expect("receipt")
}

fn evaluator_snapshot(kind: BenchmarkCapabilityKindV1) -> BenchmarkCapabilitySnapshotV1 {
    BenchmarkCapabilitySnapshotV1::new(vec![
        BenchmarkCapabilityFactV1::new("evaluator.local", kind, 1, 2, 0).expect("capability fact"),
    ])
    .expect("capability snapshot")
}

#[test]
fn sealed_evaluation_consumes_a_redacted_provider_free_capability_snapshot() {
    let composition = BenchmarkEvaluationCapabilityCompositionV1::from_receipt(
        &receipt(),
        &evaluator_snapshot(BenchmarkCapabilityKindV1::Evaluator),
        "evaluator.local",
        BenchmarkCapabilityKindV1::Evaluator,
        (1, 0, 0),
    )
    .expect("compatible capability snapshot");

    assert_eq!(composition.capability_id(), "evaluator.local");
    assert_eq!(composition.capability_version(), (1, 2, 0));
    assert_eq!(
        composition.decision_status(),
        receipt().evaluation().decision().status()
    );
    let encoded = serde_json::to_string(&composition).expect("safe composition JSON");
    assert!(!encoded.contains("fixture-model"));
    assert!(!encoded.contains("input"));
    assert!(!encoded.contains("output"));
}

#[test]
fn capability_kind_mismatch_fails_closed_before_binding_evaluation() {
    let error = BenchmarkEvaluationCapabilityCompositionV1::from_receipt(
        &receipt(),
        &evaluator_snapshot(BenchmarkCapabilityKindV1::Tool),
        "evaluator.local",
        BenchmarkCapabilityKindV1::Evaluator,
        (1, 0, 0),
    )
    .expect_err("tool capability must not satisfy evaluator requirement");

    assert!(matches!(
        error,
        BenchmarkCapabilityCompositionError::KindMismatch { .. }
    ));
}

#[test]
fn capability_version_mismatch_fails_closed_before_binding_evaluation() {
    let major_error = BenchmarkEvaluationCapabilityCompositionV1::from_receipt(
        &receipt(),
        &evaluator_snapshot(BenchmarkCapabilityKindV1::Evaluator),
        "evaluator.local",
        BenchmarkCapabilityKindV1::Evaluator,
        (2, 0, 0),
    )
    .expect_err("a different major version must not satisfy the requirement");
    assert!(matches!(
        major_error,
        BenchmarkCapabilityCompositionError::MajorVersionMismatch { .. }
    ));

    let minimum_error = BenchmarkEvaluationCapabilityCompositionV1::from_receipt(
        &receipt(),
        &evaluator_snapshot(BenchmarkCapabilityKindV1::Evaluator),
        "evaluator.local",
        BenchmarkCapabilityKindV1::Evaluator,
        (1, 3, 0),
    )
    .expect_err("an older minor version must not satisfy the requirement");
    assert!(matches!(
        minimum_error,
        BenchmarkCapabilityCompositionError::VersionTooOld { .. }
    ));
}

#[test]
fn capability_snapshot_rejects_unsupported_schema_and_unknown_raw_fields() {
    let snapshot = evaluator_snapshot(BenchmarkCapabilityKindV1::Evaluator);
    let mut unsupported = serde_json::to_value(&snapshot).expect("snapshot JSON");
    unsupported["schema_version"] = json!(2);
    assert!(serde_json::from_value::<BenchmarkCapabilitySnapshotV1>(unsupported).is_err());

    let mut raw = serde_json::to_value(snapshot).expect("snapshot JSON");
    raw["capabilities"][0]["raw_content"] = json!("private payload");
    assert!(serde_json::from_value::<BenchmarkCapabilitySnapshotV1>(raw).is_err());
}
