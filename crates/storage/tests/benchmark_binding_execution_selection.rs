//! Red-first contract tests for exact benchmark binding execution selection.

use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkDataset, BenchmarkExpectedOutput, BenchmarkSuite, MetricKind,
    RegressionThreshold, ThresholdDirection,
};
use contextlab_storage::{
    BenchmarkDefinitionBinding, BenchmarkDefinitionBindingExecutionSelection,
    BenchmarkDefinitionBindingId,
};
use contextlab_versioning::{BranchName, CommitId};
use serde_json::json;
use uuid::Uuid;

const PROJECT_ID: Uuid = Uuid::from_u128(0x11111111111111111111111111111111);
const CONTEXT_ID: Uuid = Uuid::from_u128(0x22222222222222222222222222222222);
const COMMIT_ID: Uuid = Uuid::from_u128(0x33333333333333333333333333333333);

fn dataset(id: u128, case_id: u128, name: &str) -> BenchmarkDataset {
    BenchmarkDataset::with_id(
        contextlab_evaluation::BenchmarkDatasetId::from_uuid(Uuid::from_u128(id)),
        name,
        vec![
            BenchmarkCase::with_id(
                contextlab_evaluation::BenchmarkCaseId::from_uuid(Uuid::from_u128(case_id)),
                "private-case",
                json!({"secret_input": "must stay internal"}),
                BenchmarkExpectedOutput::Unspecified,
            )
            .expect("case"),
        ],
    )
    .expect("dataset")
}

fn binding() -> BenchmarkDefinitionBinding {
    let first = dataset(
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2,
        "first",
    );
    let second = dataset(
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4,
        "second",
    );
    let suite = BenchmarkSuite::with_id(
        contextlab_evaluation::BenchmarkSuiteId::from_uuid(Uuid::from_u128(
            0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb01,
        )),
        "release",
        vec![first.id(), second.id()],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    BenchmarkDefinitionBinding::rehydrate(
        BenchmarkDefinitionBindingId::from_uuid(Uuid::from_u128(
            0xcccccccccccccccccccccccccccccc01,
        )),
        ProjectId::from_uuid(PROJECT_ID),
        ContextId::from_uuid(CONTEXT_ID),
        CommitId::from_uuid(COMMIT_ID),
        BranchName::new("main").expect("branch"),
        1,
        vec![second, first],
        suite,
        Utc.with_ymd_and_hms(2026, 7, 27, 0, 0, 0)
            .single()
            .expect("timestamp"),
    )
    .expect("binding")
}

#[test]
fn selection_assembles_exact_definition_without_exposing_case_payloads() {
    let binding = binding();
    let selection = BenchmarkDefinitionBindingExecutionSelection::for_exact_scope(
        &binding,
        ProjectId::from_uuid(PROJECT_ID),
        ContextId::from_uuid(CONTEXT_ID),
        CommitId::from_uuid(COMMIT_ID),
    )
    .expect("selection");

    assert_eq!(selection.binding_id(), binding.id());
    assert_eq!(selection.project_id(), ProjectId::from_uuid(PROJECT_ID));
    assert_eq!(selection.context_id(), ContextId::from_uuid(CONTEXT_ID));
    assert_eq!(
        selection.context_commit_id(),
        CommitId::from_uuid(COMMIT_ID)
    );
    assert_eq!(selection.suite_id(), binding.suite().id());
    assert_eq!(selection.dataset_ids(), binding.dataset_ids());
    assert_eq!(selection.case_count(), 2);
    let debug = format!("{selection:?}");
    assert!(!debug.contains("secret_input"));
    assert!(!debug.contains("private-case"));
}

#[test]
fn selection_rejects_scope_drift_before_plan_use() {
    let binding = binding();
    let error = BenchmarkDefinitionBindingExecutionSelection::for_exact_scope(
        &binding,
        ProjectId::from_uuid(Uuid::from_u128(0x44444444444444444444444444444444)),
        ContextId::from_uuid(CONTEXT_ID),
        CommitId::from_uuid(COMMIT_ID),
    )
    .expect_err("scope drift must fail closed");

    assert!(matches!(
        error,
        contextlab_storage::BenchmarkDefinitionBindingExecutionSelectionError::ScopeMismatch { .. }
    ));
}
