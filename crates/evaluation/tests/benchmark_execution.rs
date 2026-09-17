//! Integration tests for sealed benchmark execution planning.

use chrono::{TimeZone, Utc};
use contextlab_context_core::ContextId;
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
    BenchmarkDatasetId, BenchmarkExecutionError, BenchmarkExecutionPlan, BenchmarkExpectedOutput,
    BenchmarkSuite, EvaluationRun, EvaluationRunId, MetricKind, MetricMeasurement,
    RegressionThreshold, ThresholdDirection,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn plan_orders_sealed_cases_and_assembles_runs_from_matching_results() {
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
        ],
    )
    .expect("suite");

    let plan = BenchmarkExecutionPlan::new(suite, vec![second_dataset, first_dataset])
        .expect("execution plan");

    assert_eq!(
        plan.cases()
            .iter()
            .map(|case| (case.dataset_id(), case.case_id()))
            .collect::<Vec<_>>(),
        vec![
            (first_dataset_id, first_case_id),
            (second_dataset_id, second_case_id),
        ]
    );

    let decision_namespace = Uuid::from_u128(100);
    let cohort = plan
        .assemble_cohort(
            decision_namespace,
            ContextId::new(),
            "fixture-evaluator",
            0.0,
            Utc.with_ymd_and_hms(2026, 7, 18, 0, 0, 0)
                .single()
                .expect("timestamp"),
            vec![
                result(second_dataset_id, second_case_id, 0.96).expect("result"),
                result(first_dataset_id, first_case_id, 0.94).expect("result"),
            ],
        )
        .expect("cohort");
    let repeated = plan
        .assemble_cohort(
            decision_namespace,
            ContextId::new(),
            "fixture-evaluator",
            0.0,
            Utc.with_ymd_and_hms(2026, 7, 18, 0, 0, 0)
                .single()
                .expect("timestamp"),
            vec![
                result(first_dataset_id, first_case_id, 0.94).expect("result"),
                result(second_dataset_id, second_case_id, 0.96).expect("result"),
            ],
        )
        .expect("repeated cohort");
    let different = plan
        .assemble_cohort(
            Uuid::from_u128(101),
            ContextId::new(),
            "fixture-evaluator",
            0.0,
            Utc.with_ymd_and_hms(2026, 7, 18, 0, 0, 0)
                .single()
                .expect("timestamp"),
            vec![
                result(first_dataset_id, first_case_id, 0.94).expect("result"),
                result(second_dataset_id, second_case_id, 0.96).expect("result"),
            ],
        )
        .expect("different cohort");

    assert_eq!(cohort.entries().len(), 2);
    assert_eq!(cohort.entries()[0].key().dataset_id(), first_dataset_id);
    assert_eq!(cohort.entries()[0].key().case_id(), first_case_id);
    assert_eq!(cohort.entries()[0].run().measurements()[0].value(), 0.94);
    assert_eq!(cohort.entries()[1].run().measurements()[0].value(), 0.96);
    assert_eq!(
        cohort.entries()[0].run().id(),
        repeated.entries()[0].run().id()
    );
    assert_ne!(
        cohort.entries()[0].run().id(),
        different.entries()[0].run().id()
    );
}

#[test]
fn plan_rejects_duplicate_unknown_and_missing_case_results() {
    let dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(10));
    let plan = BenchmarkExecutionPlan::new(
        BenchmarkSuite::new(
            "Release gate",
            vec![dataset_id],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("threshold"),
            ],
        )
        .expect("suite"),
        vec![dataset(dataset_id, case_id, "Only")],
    )
    .expect("execution plan");
    let context_id = ContextId::new();
    let executed_at = Utc
        .with_ymd_and_hms(2026, 7, 18, 0, 0, 0)
        .single()
        .expect("timestamp");

    assert!(matches!(
        plan.assemble_cohort(
            Uuid::from_u128(100),
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            vec![
                result(dataset_id, case_id, 0.95).expect("result"),
                result(dataset_id, case_id, 0.94).expect("result")
            ],
        ),
        Err(BenchmarkExecutionError::DuplicateCaseResult { .. })
    ));
    assert!(matches!(
        plan.assemble_cohort(
            Uuid::from_u128(100),
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            vec![
                result(
                    dataset_id,
                    BenchmarkCaseId::from_uuid(Uuid::from_u128(99)),
                    0.95,
                )
                .expect("result")
            ],
        ),
        Err(BenchmarkExecutionError::UnknownCaseResult { .. })
    ));
    assert!(matches!(
        plan.assemble_cohort(
            Uuid::from_u128(100),
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            Vec::new()
        ),
        Err(BenchmarkExecutionError::MissingCaseResult { .. })
    ));
}

#[test]
fn result_rejects_duplicate_metric_kinds_before_cohort_assembly() {
    let dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(10));

    assert!(matches!(
        BenchmarkCaseExecutionResult::new(
            dataset_id,
            case_id,
            vec![
                MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement"),
                MetricMeasurement::new(MetricKind::Accuracy, 0.94).expect("measurement"),
            ],
        ),
        Err(BenchmarkExecutionError::DuplicateMetric { .. })
    ));
}

#[test]
fn plan_rejects_dataset_membership_that_differs_from_the_suite() {
    let expected_dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let supplied_dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(2));
    let suite = BenchmarkSuite::new(
        "Release gate",
        vec![expected_dataset_id],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");

    assert!(matches!(
        BenchmarkExecutionPlan::new(
            suite,
            vec![dataset(
                supplied_dataset_id,
                BenchmarkCaseId::from_uuid(Uuid::from_u128(10)),
                "Unexpected",
            )],
        ),
        Err(BenchmarkExecutionError::DatasetMembershipMismatch)
    ));
}

#[test]
fn plan_reconstructs_results_by_deterministic_run_identity_instead_of_list_position() {
    let first_dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let second_dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(2));
    let first_case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(10));
    let second_case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(20));
    let plan = BenchmarkExecutionPlan::new(
        BenchmarkSuite::new(
            "Release gate",
            vec![second_dataset_id, first_dataset_id],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("threshold"),
            ],
        )
        .expect("suite"),
        vec![
            dataset(second_dataset_id, second_case_id, "Second"),
            dataset(first_dataset_id, first_case_id, "First"),
        ],
    )
    .expect("execution plan");
    let decision_namespace = Uuid::from_u128(100);
    let context_id = ContextId::new();
    let executed_at = Utc
        .with_ymd_and_hms(2026, 7, 18, 0, 0, 0)
        .single()
        .expect("timestamp");
    let cohort = plan
        .assemble_cohort(
            decision_namespace,
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            vec![
                result(first_dataset_id, first_case_id, 0.94).expect("result"),
                result(second_dataset_id, second_case_id, 0.96).expect("result"),
            ],
        )
        .expect("cohort");
    let mut stored_runs = cohort.runs();
    stored_runs.reverse();

    let reconstructed = plan
        .reconstruct_results_from_runs(
            decision_namespace,
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            stored_runs,
        )
        .expect("identity-mapped stored runs");

    assert_eq!(
        reconstructed
            .iter()
            .map(|result| (result.key(), result.measurements()[0].value()))
            .collect::<Vec<_>>(),
        vec![(plan.cases()[0].key(), 0.94), (plan.cases()[1].key(), 0.96),]
    );
    assert_eq!(
        plan.cases()[0]
            .key()
            .deterministic_run_id(decision_namespace),
        cohort.entries()[0].run().id()
    );
}

#[test]
fn plan_reconstruction_fails_closed_for_missing_unknown_or_mismatched_stored_runs() {
    let dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
    let case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(10));
    let plan = BenchmarkExecutionPlan::new(
        BenchmarkSuite::new(
            "Release gate",
            vec![dataset_id],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("threshold"),
            ],
        )
        .expect("suite"),
        vec![dataset(dataset_id, case_id, "Only")],
    )
    .expect("execution plan");
    let decision_namespace = Uuid::from_u128(100);
    let context_id = ContextId::new();
    let executed_at = Utc
        .with_ymd_and_hms(2026, 7, 18, 0, 0, 0)
        .single()
        .expect("timestamp");
    let expected_run_id = plan.cases()[0]
        .key()
        .deterministic_run_id(decision_namespace);

    assert!(matches!(
        plan.reconstruct_results_from_runs(
            decision_namespace,
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            Vec::new(),
        ),
        Err(BenchmarkExecutionError::MissingStoredRun { .. })
    ));

    let unknown_run = EvaluationRun::with_id(
        EvaluationRunId::from_uuid(Uuid::from_u128(999)),
        context_id,
        "fixture-evaluator",
        0.0,
        vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement")],
        executed_at,
    )
    .expect("unknown run");
    assert!(matches!(
        plan.reconstruct_results_from_runs(
            decision_namespace,
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            vec![unknown_run],
        ),
        Err(BenchmarkExecutionError::UnknownStoredRun { .. })
    ));

    let mismatched_run = EvaluationRun::with_id(
        expected_run_id,
        ContextId::new(),
        "fixture-evaluator",
        0.0,
        vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("measurement")],
        executed_at,
    )
    .expect("mismatched run");
    assert!(matches!(
        plan.reconstruct_results_from_runs(
            decision_namespace,
            context_id,
            "fixture-evaluator",
            0.0,
            executed_at,
            vec![mismatched_run],
        ),
        Err(BenchmarkExecutionError::StoredRunProvenanceMismatch { .. })
    ));
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
) -> Result<BenchmarkCaseExecutionResult, BenchmarkExecutionError> {
    BenchmarkCaseExecutionResult::new(
        dataset_id,
        case_id,
        vec![MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("measurement")],
    )
}
