//! Integration tests for the unavailable adapter contract.

use contextlab_adapter_contract::{
    AdapterRequest, AdapterResponse, AdapterState, ApplicationAdapter, ContextRequest, DiffRequest,
    EvaluationRequest, LocalCapabilityAvailability, LocalCapabilityAvailabilityV1,
    ReplayStateSnapshotProjectionV1, SharedIntegration, UnavailableApplicationAdapter,
    WorkflowRequest, WorkspaceRequest,
};
use serde_json::{Value, json};

fn replay_snapshot_value() -> Value {
    json!({
        "schema_version": 1,
        "context_id": "00000000-0000-0000-0000-000000000001",
        "initialized": true,
        "commit_id": "00000000-0000-0000-0000-000000000004",
        "context_metadata": null,
        "components": [
            {
                "component_id": "00000000-0000-0000-0000-000000000003",
                "kind": "knowledge",
                "name": "Knowledge",
                "metadata": {"rank": 3},
                "content_hash": "sha256:three"
            },
            {
                "component_id": "00000000-0000-0000-0000-000000000002",
                "kind": "prompt",
                "name": "Prompt",
                "metadata": {"rank": 2},
                "content_hash": "sha256:two"
            }
        ],
        "relationships": [
            {
                "source_component_id": "00000000-0000-0000-0000-000000000003",
                "target_component_id": "00000000-0000-0000-0000-000000000002"
            }
        ]
    })
}

#[test]
fn unavailable_adapter_reports_the_required_shared_integration_for_each_request() {
    let adapter = UnavailableApplicationAdapter;
    let cases = [
        (
            AdapterRequest::Workspace(WorkspaceRequest::inspect("workspace-alpha")),
            SharedIntegration::ContextCore,
        ),
        (
            AdapterRequest::Context(ContextRequest::inspect("context-alpha")),
            SharedIntegration::ContextCore,
        ),
        (
            AdapterRequest::Evaluation(EvaluationRequest::run("context-alpha", "suite-alpha")),
            SharedIntegration::Evaluation,
        ),
        (
            AdapterRequest::Diff(DiffRequest::compare("revision-a", "revision-b")),
            SharedIntegration::DiffEngine,
        ),
        (
            AdapterRequest::Workflow(WorkflowRequest::inspect("workflow-alpha")),
            SharedIntegration::Workflow,
        ),
    ];

    for (request, expected_integration) in cases {
        let response = adapter.execute(request);

        assert_eq!(response.state, AdapterState::Unavailable);
        assert_eq!(response.integration, expected_integration);
    }
}

#[test]
fn unavailable_response_projects_a_versioned_local_capability_dto() {
    let adapter = UnavailableApplicationAdapter;
    let request =
        AdapterRequest::Evaluation(EvaluationRequest::run("context-alpha", "suite-alpha"));
    let response = adapter.execute(request.clone());
    let availability = response
        .local_capability_availability(&request)
        .expect("matching adapter identity should project");

    assert_eq!(
        availability.schema_version(),
        "contextlab.local-capability-availability.v1"
    );
    assert_eq!(availability.operation_id(), "evaluation-run");
    assert_eq!(availability.integration(), "contextlab-evaluation");
    assert_eq!(
        availability.availability(),
        LocalCapabilityAvailability::Unavailable
    );
    assert_eq!(availability.reason(), "shared_integration_not_registered");
}

#[test]
fn local_capability_projection_fails_closed_when_response_identity_does_not_match_request() {
    let request =
        AdapterRequest::Evaluation(EvaluationRequest::run("context-alpha", "suite-alpha"));
    let mismatched_response = AdapterResponse {
        state: AdapterState::Unavailable,
        integration: SharedIntegration::DiffEngine,
    };

    assert!(
        mismatched_response
            .local_capability_availability(&request)
            .is_err()
    );
}

#[test]
fn requests_expose_stable_command_labels_for_all_presentation_adapters() {
    let cases = [
        (
            AdapterRequest::Workspace(WorkspaceRequest::inspect("workspace-alpha")),
            "workspace inspect",
        ),
        (
            AdapterRequest::Context(ContextRequest::inspect("context-alpha")),
            "context inspect",
        ),
        (
            AdapterRequest::Evaluation(EvaluationRequest::run("context-alpha", "suite-alpha")),
            "evaluation run",
        ),
        (
            AdapterRequest::Diff(DiffRequest::compare("revision-a", "revision-b")),
            "diff compare",
        ),
        (
            AdapterRequest::Workflow(WorkflowRequest::inspect("workflow-alpha")),
            "workflow inspect",
        ),
    ];

    for (request, expected_label) in cases {
        assert_eq!(request.command_label(), expected_label);
    }
}

#[test]
fn serialized_local_availability_parser_fails_closed_for_contract_drift() {
    let invalid_contracts = [
        r#"{"schema_version":"contextlab.local-capability-availability.v2","operation_id":"evaluation-run","integration":"contextlab-evaluation","availability":"unavailable","reason":"shared_integration_not_registered"}"#,
        r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"unknown-operation","integration":"contextlab-evaluation","availability":"unavailable","reason":"shared_integration_not_registered"}"#,
        r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-diff-engine","availability":"unavailable","reason":"shared_integration_not_registered"}"#,
        r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-evaluation","availability":"available","reason":"shared_integration_not_registered"}"#,
        r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-evaluation","availability":"unavailable","reason":"other"}"#,
        r#"{"schema_version":"contextlab.local-capability-availability.v1","operation_id":"evaluation-run","integration":"contextlab-evaluation","availability":"unavailable","reason":"shared_integration_not_registered","unexpected":true}"#,
    ];

    for serialized in invalid_contracts {
        assert!(LocalCapabilityAvailabilityV1::parse_serialized(serialized).is_err());
    }
}

#[test]
fn serialized_local_availability_parser_accepts_equivalent_json_formatting() {
    let serialized = r#"
        {
            "reason": "shared_integration_not_registered",
            "availability": "unavailable",
            "integration": "contextlab-evaluation",
            "operation_id": "evaluation-run",
            "schema_version": "contextlab.local-capability-availability.v1"
        }
    "#;

    let availability = LocalCapabilityAvailabilityV1::parse_serialized(serialized)
        .expect("equivalent JSON formatting should preserve the wire contract");

    assert_eq!(availability.operation_id(), "evaluation-run");
    assert_eq!(availability.integration(), "contextlab-evaluation");
}

#[test]
fn replay_snapshot_consumer_canonicalizes_component_and_relationship_order() {
    let serialized = serde_json::to_string(&replay_snapshot_value()).expect("fixture JSON");

    let projection = ReplayStateSnapshotProjectionV1::parse_serialized(&serialized)
        .expect("valid replay snapshot");

    assert_eq!(projection.schema_version(), 1);
    assert!(projection.is_initialized());
    assert_eq!(projection.component_count(), 2);
    assert_eq!(projection.relationship_count(), 1);
    assert_eq!(
        projection
            .snapshot()
            .components()
            .iter()
            .map(|component| component.component_id().to_string())
            .collect::<Vec<_>>(),
        vec![
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-000000000003"
        ]
    );

    let canonical = projection
        .to_serialized()
        .expect("projection should serialize");
    let reparsed = ReplayStateSnapshotProjectionV1::parse_serialized(&canonical)
        .expect("canonical replay snapshot");
    assert_eq!(reparsed.to_serialized().expect("re-serialize"), canonical);
}

#[test]
fn replay_snapshot_consumer_fails_closed_for_schema_drift_and_invalid_relationships() {
    let mut unsupported_schema = replay_snapshot_value();
    unsupported_schema["schema_version"] = json!(2);

    let mut unknown_outer_field = replay_snapshot_value();
    unknown_outer_field["unexpected"] = json!(true);

    let mut unknown_nested_field = replay_snapshot_value();
    unknown_nested_field["components"][0]["unexpected"] = json!(true);

    let mut duplicate_relationship = replay_snapshot_value();
    let relationship = duplicate_relationship["relationships"][0].clone();
    duplicate_relationship["relationships"]
        .as_array_mut()
        .expect("relationships array")
        .push(relationship);

    for value in [
        unsupported_schema,
        unknown_outer_field,
        unknown_nested_field,
        duplicate_relationship,
    ] {
        let serialized = serde_json::to_string(&value).expect("fixture JSON");
        assert!(
            ReplayStateSnapshotProjectionV1::parse_serialized(&serialized).is_err(),
            "schema drift must not be accepted: {serialized}"
        );
    }
}
