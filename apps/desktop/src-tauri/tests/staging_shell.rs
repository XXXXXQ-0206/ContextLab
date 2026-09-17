//! Integration tests for the ContextLab desktop Tauri staging shell.

use contextlab_adapter_contract::{
    AdapterRequest, DiffRequest, LocalCapabilityAvailability, SharedIntegration,
    UnavailableApplicationAdapter,
};
use contextlab_desktop_tauri_staging::{DesktopStagingShell, DesktopStatus};

#[test]
fn desktop_shell_delegates_to_the_shared_unavailable_adapter() {
    let shell = DesktopStagingShell::new(UnavailableApplicationAdapter);
    let execution = shell
        .invoke(AdapterRequest::Diff(DiffRequest::compare(
            "revision-a",
            "revision-b",
        )))
        .expect("matching adapter identity should project");

    assert_eq!(
        execution.status,
        DesktopStatus::Unavailable(SharedIntegration::DiffEngine)
    );
    assert_eq!(
        execution.message,
        "diff compare: unavailable; awaiting contextlab-diff-engine registration"
    );
    assert_eq!(
        execution.availability.availability(),
        LocalCapabilityAvailability::Unavailable
    );
    assert_eq!(execution.availability.operation_id(), "diff-compare");
}

#[test]
fn desktop_shell_hands_the_serialized_contract_to_the_shared_bilingual_presenter() {
    let shell = DesktopStagingShell::new(UnavailableApplicationAdapter);
    let serialized = r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"workflow-inspect","integration":"contextlab-workflow","availability":"unavailable","reason":"shared_integration_not_registered"}"#;

    let presentation = shell
        .present_serialized_local_capability_availability(serialized)
        .expect("the exact unavailable shared contract should be accepted");

    assert_eq!(
        presentation.schema_version(),
        "contextlab.local-capability-availability.v1"
    );
    assert_eq!(presentation.operation_id(), "workflow-inspect");
    assert_eq!(
        presentation.capability_id(),
        "local-capability-workflow-inspect"
    );
    assert_eq!(
        presentation.availability(),
        LocalCapabilityAvailability::Unavailable
    );
    assert_eq!(presentation.capability().en(), "Workflow engine");
    assert_eq!(presentation.capability().zh(), "工作流引擎");
    assert_eq!(
        presentation.summary().en(),
        "Workflow engine is unavailable because shared registration has not been completed."
    );
    assert_eq!(
        presentation.summary().zh(),
        "工作流引擎因共享注册尚未完成而不可用。"
    );
}

#[test]
fn tauri_configuration_declares_a_staging_identifier() {
    let configuration = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tauri.conf.json"));

    assert!(configuration.contains("\"identifier\": \"org.contextlab.staging\""));
}

#[test]
fn desktop_shell_presents_a_typed_replay_snapshot_projection() {
    let shell = DesktopStagingShell::new(UnavailableApplicationAdapter);
    let serialized = r#"{"schema_version":1,"context_id":"00000000-0000-0000-0000-000000000001","initialized":true,"commit_id":"00000000-0000-0000-0000-000000000004","context_metadata":null,"components":[{"component_id":"00000000-0000-0000-0000-000000000003","kind":"knowledge","name":"Knowledge","metadata":{"rank":3},"content_hash":"sha256:three"},{"component_id":"00000000-0000-0000-0000-000000000002","kind":"prompt","name":"Prompt","metadata":{"rank":2},"content_hash":"sha256:two"}],"relationships":[{"source_component_id":"00000000-0000-0000-0000-000000000003","target_component_id":"00000000-0000-0000-0000-000000000002"}]}"#;

    let projection = shell
        .present_serialized_replay_state_snapshot(serialized)
        .expect("valid replay snapshot");

    assert_eq!(projection.schema_version(), 1);
    assert_eq!(projection.component_count(), 2);
    assert_eq!(projection.relationship_count(), 1);
    assert_eq!(
        projection.snapshot().components()[0]
            .component_id()
            .to_string(),
        "00000000-0000-0000-0000-000000000002"
    );
}

#[test]
fn desktop_shell_rejects_replay_snapshot_schema_drift() {
    let shell = DesktopStagingShell::new(UnavailableApplicationAdapter);
    let serialized = r#"{"schema_version":2,"context_id":"00000000-0000-0000-0000-000000000001","initialized":true,"commit_id":null,"context_metadata":null,"components":[],"relationships":[]}"#;

    assert!(
        shell
            .present_serialized_replay_state_snapshot(serialized)
            .is_err()
    );
}
