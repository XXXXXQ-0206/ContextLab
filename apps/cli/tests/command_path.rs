//! Integration tests for the ContextLab CLI staging shell.

use std::process::Command;

use contextlab_adapter_contract::{
    AdapterRequest, ContextRequest, DiffRequest, EvaluationRequest, LocalCapabilityAvailability,
    SharedIntegration, UnavailableApplicationAdapter, WorkflowRequest, WorkspaceRequest,
};
use contextlab_cli::{CliStatus, execute, parse_arguments};

fn replay_snapshot_json() -> String {
    r#"{"schema_version":1,"context_id":"00000000-0000-0000-0000-000000000001","initialized":true,"commit_id":"00000000-0000-0000-0000-000000000004","context_metadata":null,"components":[{"component_id":"00000000-0000-0000-0000-000000000003","kind":"knowledge","name":"Knowledge","metadata":{"rank":3},"content_hash":"sha256:three"},{"component_id":"00000000-0000-0000-0000-000000000002","kind":"prompt","name":"Prompt","metadata":{"rank":2},"content_hash":"sha256:two"}],"relationships":[{"source_component_id":"00000000-0000-0000-0000-000000000003","target_component_id":"00000000-0000-0000-0000-000000000002"}]}"#.to_owned()
}

#[test]
fn minimal_commands_parse_into_typed_adapter_requests() {
    let cases = [
        (
            vec!["workspace", "inspect", "workspace-alpha"],
            AdapterRequest::Workspace(WorkspaceRequest::inspect("workspace-alpha")),
        ),
        (
            vec!["context", "inspect", "context-alpha"],
            AdapterRequest::Context(ContextRequest::inspect("context-alpha")),
        ),
        (
            vec!["evaluation", "run", "context-alpha", "suite-alpha"],
            AdapterRequest::Evaluation(EvaluationRequest::run("context-alpha", "suite-alpha")),
        ),
        (
            vec!["diff", "compare", "revision-a", "revision-b"],
            AdapterRequest::Diff(DiffRequest::compare("revision-a", "revision-b")),
        ),
        (
            vec!["workflow", "inspect", "workflow-alpha"],
            AdapterRequest::Workflow(WorkflowRequest::inspect("workflow-alpha")),
        ),
    ];

    for (arguments, expected_request) in cases {
        assert_eq!(parse_arguments(arguments).unwrap(), expected_request);
    }
}

#[test]
fn evaluation_command_exposes_the_unavailable_shared_integration() {
    let request = parse_arguments(["evaluation", "run", "context-alpha", "suite-alpha"]).unwrap();
    let execution = execute(&UnavailableApplicationAdapter, request).expect("matching identity");

    assert_eq!(
        execution.status,
        CliStatus::Unavailable(SharedIntegration::Evaluation)
    );
    assert_eq!(
        execution.message,
        "evaluation run: unavailable; awaiting contextlab-evaluation registration"
    );
    assert_eq!(
        execution.availability.availability(),
        LocalCapabilityAvailability::Unavailable
    );
    assert_eq!(execution.availability.operation_id(), "evaluation-run");
}

#[test]
fn capability_inspect_renders_a_deterministic_bilingual_unavailable_projection() {
    let serialized = r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-evaluation","availability":"unavailable","reason":"shared_integration_not_registered"}"#;

    let output = Command::new(env!("CARGO_BIN_EXE_contextlab-cli"))
        .args(["capability", "inspect", serialized])
        .output()
        .expect("CLI binary should start");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("CLI output should be UTF-8"),
        concat!(
            "schema_version: contextlab.local-capability-availability.v1\n",
            "operation_id: evaluation-run\n",
            "capability_id: local-capability-evaluation-run\n",
            "integration: contextlab-evaluation\n",
            "availability: unavailable\n",
            "capability: Evaluation engine / 评测引擎\n",
            "summary: Evaluation engine is unavailable because shared registration has not been completed. / 评测引擎因共享注册尚未完成而不可用。\n",
            "detail: Shared integration: contextlab-evaluation. / 共享集成：contextlab-evaluation。\n"
        )
    );
}

#[test]
fn capability_inspect_fails_closed_when_serialized_integration_does_not_match_its_operation() {
    let serialized = r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-diff-engine","availability":"unavailable","reason":"shared_integration_not_registered"}"#;

    let output = Command::new(env!("CARGO_BIN_EXE_contextlab-cli"))
        .args(["capability", "inspect", serialized])
        .output()
        .expect("CLI binary should start");

    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("CLI error output should be UTF-8"),
        "invalid local capability availability contract / 本地能力可用性契约无效\n"
    );
}

#[test]
fn replay_inspect_renders_canonical_read_only_snapshot_summary() {
    let output = Command::new(env!("CARGO_BIN_EXE_contextlab-cli"))
        .args(["replay", "inspect", &replay_snapshot_json()])
        .output()
        .expect("CLI binary should start");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).expect("CLI output should be UTF-8"),
        concat!(
            "schema_version: 1\n",
            "context_id: 00000000-0000-0000-0000-000000000001\n",
            "commit_id: 00000000-0000-0000-0000-000000000004\n",
            "initialized: true\n",
            "component_count: 2\n",
            "relationship_count: 1\n",
            "component_ids:\n",
            "- 00000000-0000-0000-0000-000000000002\n",
            "- 00000000-0000-0000-0000-000000000003\n",
            "relationship: 00000000-0000-0000-0000-000000000003 -> 00000000-0000-0000-0000-000000000002\n"
        )
    );
}
