//! Contract tests for the validated, redacted execution status projection.

use contextlab_context_core::ContextId;
use contextlab_versioning::CommitId;
use contextlab_workflow::{
    ContextCommitSource, WorkflowCapabilitySnapshot, WorkflowContextBinding,
    WorkflowContextBindingId, WorkflowDefinition, WorkflowExecution, WorkflowExecutionReplayError,
    WorkflowExecutionStatusProjectionError, WorkflowExecutionStatusProjectionV1, WorkflowFailure,
    WorkflowId, WorkflowNode, WorkflowNodeId, WorkflowRevision, WorkflowRunId, WorkflowRunState,
};
use serde_json::json;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn revision() -> WorkflowRevision {
    WorkflowRevision::new(7).expect("non-zero workflow revision")
}

fn binding() -> WorkflowContextBinding {
    WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(10)),
        WorkflowDefinition::new(
            WorkflowId::from_uuid(uuid(11)),
            revision(),
            vec![
                WorkflowNode::new(WorkflowNodeId::from_uuid(uuid(12)), revision()),
                WorkflowNode::new(WorkflowNodeId::from_uuid(uuid(13)), revision()),
            ],
            Vec::new(),
        )
        .expect("valid workflow definition"),
        ContextCommitSource::new(
            ContextId::from_uuid(uuid(14)),
            CommitId::from_uuid(uuid(15)),
        ),
    )
}

fn successful_execution(run_id: WorkflowRunId) -> (WorkflowContextBinding, WorkflowExecution) {
    let binding = binding();
    let mut execution = WorkflowExecution::start(
        binding.clone(),
        run_id,
        &WorkflowCapabilitySnapshot::default(),
    )
    .expect("bound workflow starts");
    for _ in 0..2 {
        let claim = execution
            .claim_next()
            .expect("claim succeeds")
            .expect("a node is ready");
        execution
            .mark_succeeded(claim.node_id())
            .expect("node succeeds");
    }
    (binding, execution)
}

#[test]
fn successful_log_projects_exact_provenance_and_deterministic_counts() {
    let run_id = WorkflowRunId::from_uuid(uuid(20));
    let (binding, execution) = successful_execution(run_id);
    let projection = WorkflowExecutionStatusProjectionV1::from_log(
        binding.clone(),
        &WorkflowCapabilitySnapshot::default(),
        execution.log().clone(),
    )
    .expect("canonical success log projects");

    assert_eq!(projection.run_id(), run_id);
    assert_eq!(projection.binding_id(), binding.id());
    assert_eq!(projection.workflow_id(), binding.workflow_id());
    assert_eq!(projection.workflow_revision(), revision());
    assert_eq!(projection.context_source(), binding.context_source());
    assert_eq!(projection.run_state(), WorkflowRunState::Succeeded);
    assert_eq!(projection.event_count(), 6);
    assert_eq!(projection.last_sequence(), 6);
    assert_eq!(projection.replay_of(), None);
    assert_eq!(
        projection.capability_snapshot_digest(),
        execution.log().capability_snapshot_digest()
    );
    assert_eq!(projection.node_status_counts().pending(), 0);
    assert_eq!(projection.node_status_counts().running(), 0);
    assert_eq!(projection.node_status_counts().succeeded(), 2);
    assert_eq!(projection.node_status_counts().failed(), 0);
    assert_eq!(projection.node_status_counts().blocked(), 0);

    let payload = serde_json::to_value(&projection).expect("projection serializes");
    assert_eq!(payload["schema_version"], "v1");
    assert_eq!(payload["event_count"], 6);
    assert_eq!(payload["last_sequence"], 6);
    assert!(payload.get("events").is_none());
}

#[test]
fn failed_log_projects_redacted_state_without_failure_payload() {
    let binding = binding();
    let run_id = WorkflowRunId::from_uuid(uuid(30));
    let failed_node_id = binding.workflow_definition().nodes()[0].id();
    let mut execution = WorkflowExecution::start(
        binding.clone(),
        run_id,
        &WorkflowCapabilitySnapshot::default(),
    )
    .expect("bound workflow starts");
    execution
        .claim_next()
        .expect("claim succeeds")
        .expect("a node is ready");
    execution
        .mark_failed(
            failed_node_id,
            WorkflowFailure::new("provider_secret: should_not_cross_boundary")
                .expect("valid failure code"),
        )
        .expect("node fails");

    let projection = WorkflowExecutionStatusProjectionV1::from_log(
        binding,
        &WorkflowCapabilitySnapshot::default(),
        execution.log().clone(),
    )
    .expect("canonical failure log projects");
    assert_eq!(projection.run_state(), WorkflowRunState::Failed);
    assert_eq!(projection.event_count(), 4);
    assert_eq!(projection.last_sequence(), 4);
    assert_eq!(projection.node_status_counts().failed(), 1);
    assert_eq!(projection.node_status_counts().blocked(), 1);

    let serialized = serde_json::to_string(&projection).expect("projection serializes");
    assert!(!serialized.contains("provider_secret"));
    assert!(!serialized.contains("should_not_cross_boundary"));
    assert!(!serialized.contains("failure"));
    assert!(!serialized.contains("events"));
}

#[test]
fn validated_replay_projects_fresh_run_and_exact_source_provenance() {
    let source_run_id = WorkflowRunId::from_uuid(uuid(40));
    let replay_run_id = WorkflowRunId::from_uuid(uuid(41));
    let (binding, source) = successful_execution(source_run_id);
    let replay = source
        .replay_as(replay_run_id)
        .expect("terminal source can be replayed");

    let projection = WorkflowExecutionStatusProjectionV1::from_replay_log(
        binding,
        &WorkflowCapabilitySnapshot::default(),
        replay.log().clone(),
        &source,
    )
    .expect("validated replay log projects");

    assert_eq!(projection.run_id(), replay_run_id);
    assert_eq!(projection.replay_of(), Some(source_run_id));
    assert_eq!(projection.run_state(), WorkflowRunState::Pending);
    assert_eq!(projection.event_count(), 1);
    assert_eq!(projection.last_sequence(), 1);
    assert_eq!(projection.node_status_counts().pending(), 2);
}

#[test]
fn projection_rejects_scope_sequence_and_unvalidated_replay_provenance() {
    let run_id = WorkflowRunId::from_uuid(uuid(50));
    let (binding, execution) = successful_execution(run_id);

    let mismatched_binding = WorkflowContextBinding::new(
        binding.id(),
        binding.workflow_definition().clone(),
        ContextCommitSource::new(
            ContextId::from_uuid(uuid(51)),
            CommitId::from_uuid(uuid(52)),
        ),
    );
    assert!(matches!(
        WorkflowExecutionStatusProjectionV1::from_log(
            mismatched_binding,
            &WorkflowCapabilitySnapshot::default(),
            execution.log().clone(),
        ),
        Err(WorkflowExecutionStatusProjectionError::Replay(
            WorkflowExecutionReplayError::ContextSourceMismatch { .. }
        ))
    ));

    let mut sequence_payload = serde_json::to_value(execution.log()).expect("log serializes");
    sequence_payload["events"][1]["sequence"] = json!(9);
    let sequence_log = serde_json::from_value(sequence_payload).expect("V1 log deserializes");
    assert!(matches!(
        WorkflowExecutionStatusProjectionV1::from_log(
            binding.clone(),
            &WorkflowCapabilitySnapshot::default(),
            sequence_log,
        ),
        Err(WorkflowExecutionStatusProjectionError::Replay(
            WorkflowExecutionReplayError::SequenceMismatch {
                expected: 2,
                provided: 9
            }
        ))
    ));

    let (_, source) = successful_execution(WorkflowRunId::from_uuid(uuid(53)));
    let replay = source
        .replay_as(WorkflowRunId::from_uuid(uuid(54)))
        .expect("terminal source can be replayed");
    assert!(matches!(
        WorkflowExecutionStatusProjectionV1::from_log(
            binding,
            &WorkflowCapabilitySnapshot::default(),
            replay.log().clone(),
        ),
        Err(WorkflowExecutionStatusProjectionError::Replay(
            WorkflowExecutionReplayError::ReplaySourceValidationRequired { .. }
        ))
    ));
}

#[test]
fn schema_drift_is_rejected_before_projection() {
    let (_, execution) = successful_execution(WorkflowRunId::from_uuid(uuid(60)));
    let mut payload = serde_json::to_value(execution.log()).expect("log serializes");
    payload["schema_version"] = json!("v2");

    let error = serde_json::from_value::<contextlab_workflow::WorkflowExecutionLogV1>(payload)
        .expect_err("unsupported schema cannot cross the typed log boundary");
    assert!(error.to_string().contains("v2"));
}
