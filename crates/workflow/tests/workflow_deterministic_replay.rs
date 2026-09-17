//! Contract tests for provider-free deterministic workflow execution and replay.

use contextlab_context_core::ContextId;
use contextlab_versioning::CommitId;
use contextlab_workflow::{
    ContextCommitSource, WorkflowCapability, WorkflowCapabilityRequirement,
    WorkflowCapabilitySnapshot, WorkflowCapabilityVersion, WorkflowContextBinding,
    WorkflowContextBindingId, WorkflowDefinition, WorkflowExecution, WorkflowExecutionEventV1,
    WorkflowExecutionReplayError, WorkflowFailure, WorkflowId, WorkflowNode, WorkflowNodeId,
    WorkflowNodeState, WorkflowRevision, WorkflowRunId, WorkflowRunState,
};
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn revision() -> WorkflowRevision {
    WorkflowRevision::new(1).expect("non-zero workflow revision")
}

fn binding() -> WorkflowContextBinding {
    WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(1_001)),
        WorkflowDefinition::new(
            WorkflowId::from_uuid(uuid(1_002)),
            revision(),
            vec![WorkflowNode::new(
                WorkflowNodeId::from_uuid(uuid(1_003)),
                revision(),
            )],
            Vec::new(),
        )
        .expect("valid workflow definition"),
        ContextCommitSource::new(
            ContextId::from_uuid(uuid(1_004)),
            CommitId::from_uuid(uuid(1_005)),
        ),
    )
}

fn capability_binding() -> WorkflowContextBinding {
    WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(1_101)),
        WorkflowDefinition::new(
            WorkflowId::from_uuid(uuid(1_102)),
            revision(),
            vec![
                WorkflowNode::with_capability_requirements(
                    WorkflowNodeId::from_uuid(uuid(1_103)),
                    revision(),
                    vec![
                        WorkflowCapabilityRequirement::new(
                            "workflow.alpha",
                            WorkflowCapabilityVersion::new(1, 0, 0),
                        )
                        .expect("valid capability requirement"),
                    ],
                )
                .expect("valid workflow node"),
            ],
            Vec::new(),
        )
        .expect("valid workflow definition"),
        ContextCommitSource::new(
            ContextId::from_uuid(uuid(1_104)),
            CommitId::from_uuid(uuid(1_105)),
        ),
    )
}

fn capability_snapshot(alpha_patch: u64, include_zeta: bool) -> WorkflowCapabilitySnapshot {
    let mut capabilities = vec![
        WorkflowCapability::new(
            "workflow.alpha",
            WorkflowCapabilityVersion::new(1, 1, alpha_patch),
        )
        .expect("valid alpha capability"),
    ];
    if include_zeta {
        capabilities.push(
            WorkflowCapability::new("workflow.zeta", WorkflowCapabilityVersion::new(2, 0, 0))
                .expect("valid zeta capability"),
        );
    }
    WorkflowCapabilitySnapshot::new(capabilities).expect("unique capability snapshot")
}

fn capability_log() -> (
    WorkflowContextBinding,
    WorkflowCapabilitySnapshot,
    WorkflowExecution,
) {
    let binding = capability_binding();
    let snapshot = capability_snapshot(0, true);
    let execution = WorkflowExecution::start(
        binding.clone(),
        WorkflowRunId::from_uuid(uuid(9_001)),
        &snapshot,
    )
    .expect("compatible capability snapshot starts the workflow");
    (binding, snapshot, execution)
}

#[test]
fn execution_records_canonical_success_events_for_a_bound_workflow() {
    let binding = binding();
    let run_id = WorkflowRunId::from_uuid(uuid(2_001));
    let node_id = binding.workflow_definition().nodes()[0].id();
    let mut execution =
        WorkflowExecution::start(binding, run_id, &WorkflowCapabilitySnapshot::default())
            .expect("bound workflow starts with its existing capability snapshot");

    let claim = execution
        .claim_next()
        .expect("scheduler claims the ready node")
        .expect("one node is ready");
    execution
        .mark_succeeded(claim.node_id())
        .expect("claimed node succeeds");

    assert_eq!(
        execution.log().events(),
        [
            WorkflowExecutionEventV1::RunStarted { sequence: 1 },
            WorkflowExecutionEventV1::NodeClaimed {
                sequence: 2,
                node_id,
                attempt: 1,
            },
            WorkflowExecutionEventV1::NodeSucceeded {
                sequence: 3,
                node_id,
                attempt: 1,
            },
            WorkflowExecutionEventV1::RunSucceeded { sequence: 4 },
        ]
    );
}

#[test]
fn failure_is_terminal_and_reconstructs_from_the_exact_canonical_log() {
    let binding = binding();
    let run_id = WorkflowRunId::from_uuid(uuid(3_001));
    let node_id = binding.workflow_definition().nodes()[0].id();
    let snapshot = WorkflowCapabilitySnapshot::default();
    let mut execution = WorkflowExecution::start(binding.clone(), run_id, &snapshot)
        .expect("bound workflow starts");
    execution
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready");
    execution
        .mark_failed(
            node_id,
            WorkflowFailure::new("executor_timeout").expect("valid typed failure"),
        )
        .expect("running node fails terminally");

    assert_eq!(execution.run_state(), WorkflowRunState::Failed);
    assert!(matches!(
        execution.node_state(node_id),
        Some(WorkflowNodeState::Failed { failure, .. })
            if failure.code().as_str() == "executor_timeout"
    ));
    assert_eq!(
        execution.log().events(),
        [
            WorkflowExecutionEventV1::RunStarted { sequence: 1 },
            WorkflowExecutionEventV1::NodeClaimed {
                sequence: 2,
                node_id,
                attempt: 1,
            },
            WorkflowExecutionEventV1::NodeFailed {
                sequence: 3,
                node_id,
                attempt: 1,
                failure: WorkflowFailure::new("executor_timeout").expect("valid typed failure"),
            },
            WorkflowExecutionEventV1::RunFailed {
                sequence: 4,
                failed_node_id: node_id,
            },
        ]
    );

    let reconstructed = WorkflowExecution::from_log(binding, &snapshot, execution.log().clone())
        .expect("exact canonical log reconstructs the same terminal state");

    assert_eq!(reconstructed.log(), execution.log());
    assert_eq!(reconstructed.run_state(), WorkflowRunState::Failed);
    assert_eq!(
        reconstructed.node_state(node_id),
        execution.node_state(node_id)
    );
}

#[test]
fn terminal_execution_starts_a_fresh_replay_with_explicit_provenance() {
    let source_run_id = WorkflowRunId::from_uuid(uuid(4_001));
    let replay_run_id = WorkflowRunId::from_uuid(uuid(4_002));
    let binding = binding();
    let node_id = binding.workflow_definition().nodes()[0].id();
    let mut source = WorkflowExecution::start(
        binding,
        source_run_id,
        &WorkflowCapabilitySnapshot::default(),
    )
    .expect("bound workflow starts");
    source
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready");
    source
        .mark_succeeded(node_id)
        .expect("node completes terminally");

    let replay = source
        .replay_as(replay_run_id)
        .expect("terminal execution can start a fresh replay");

    assert_eq!(replay.run_state(), WorkflowRunState::Pending);
    assert_eq!(replay.log().run_id(), replay_run_id);
    assert_eq!(replay.log().replay_of(), Some(source_run_id));
    assert_eq!(
        replay.log().events(),
        [WorkflowExecutionEventV1::RunStarted { sequence: 1 }]
    );
}

#[test]
fn reconstruction_rejects_self_referential_replay_provenance() {
    let binding = binding();
    let snapshot = WorkflowCapabilitySnapshot::default();
    let run_id = WorkflowRunId::from_uuid(uuid(4_101));
    let execution = WorkflowExecution::start(binding.clone(), run_id, &snapshot)
        .expect("bound workflow starts");
    let mut payload = serde_json::to_value(execution.log()).expect("log serializes");
    payload["replay_of"] = serde_json::json!(run_id.as_uuid());
    let forged_log = serde_json::from_value(payload).expect("typed V1 log deserializes");

    assert_eq!(
        WorkflowExecution::from_log(binding, &snapshot, forged_log)
            .expect_err("a run cannot claim itself as replay provenance"),
        WorkflowExecutionReplayError::ReplaySourceIsCurrentRun { run_id }
    );
}

#[test]
fn replay_log_reconstruction_requires_a_validated_terminal_source() {
    let binding = binding();
    let snapshot = WorkflowCapabilitySnapshot::default();
    let source_run_id = WorkflowRunId::from_uuid(uuid(4_201));
    let replay_run_id = WorkflowRunId::from_uuid(uuid(4_202));
    let node_id = binding.workflow_definition().nodes()[0].id();
    let mut source =
        WorkflowExecution::start(binding.clone(), source_run_id, &snapshot).expect("source starts");
    source
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready");
    source
        .mark_succeeded(node_id)
        .expect("source becomes terminal");
    let replay = source
        .replay_as(replay_run_id)
        .expect("fresh replay starts");

    assert_eq!(
        WorkflowExecution::from_log(binding, &snapshot, replay.log().clone())
            .expect_err("a bare replay_of UUID is not validated provenance"),
        WorkflowExecutionReplayError::ReplaySourceValidationRequired {
            claimed_source_run_id: source_run_id,
        }
    );
}

#[test]
fn replay_log_reconstructs_only_from_the_exact_terminal_source() {
    let binding = binding();
    let snapshot = WorkflowCapabilitySnapshot::default();
    let source_run_id = WorkflowRunId::from_uuid(uuid(4_301));
    let replay_run_id = WorkflowRunId::from_uuid(uuid(4_302));
    let node_id = binding.workflow_definition().nodes()[0].id();
    let mut source =
        WorkflowExecution::start(binding.clone(), source_run_id, &snapshot).expect("source starts");
    source
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready");
    source
        .mark_succeeded(node_id)
        .expect("source becomes terminal");
    let replay = source
        .replay_as(replay_run_id)
        .expect("fresh replay starts");

    let reconstructed =
        WorkflowExecution::from_replay_log(binding, &snapshot, replay.log().clone(), &source)
            .expect("the exact validated terminal source authorizes reconstruction");

    assert_eq!(reconstructed.log(), replay.log());
    assert_eq!(reconstructed.run_state(), WorkflowRunState::Pending);
}

#[test]
fn replay_log_rejects_a_different_or_non_terminal_source_execution() {
    let binding = binding();
    let snapshot = WorkflowCapabilitySnapshot::default();
    let claimed_source_run_id = WorkflowRunId::from_uuid(uuid(4_401));
    let replay_run_id = WorkflowRunId::from_uuid(uuid(4_402));
    let node_id = binding.workflow_definition().nodes()[0].id();
    let mut claimed_source =
        WorkflowExecution::start(binding.clone(), claimed_source_run_id, &snapshot)
            .expect("source starts");
    claimed_source
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready");
    claimed_source
        .mark_succeeded(node_id)
        .expect("source becomes terminal");
    let replay = claimed_source
        .replay_as(replay_run_id)
        .expect("fresh replay starts");

    let wrong_source_run_id = WorkflowRunId::from_uuid(uuid(4_403));
    let wrong_source = WorkflowExecution::start(binding.clone(), wrong_source_run_id, &snapshot)
        .expect("different source starts");
    assert_eq!(
        WorkflowExecution::from_replay_log(
            binding.clone(),
            &snapshot,
            replay.log().clone(),
            &wrong_source,
        )
        .expect_err("an unrelated source run cannot authorize replay provenance"),
        WorkflowExecutionReplayError::ReplaySourceMismatch {
            expected: wrong_source_run_id,
            provided: claimed_source_run_id,
        }
    );

    let non_terminal_source =
        WorkflowExecution::start(binding.clone(), claimed_source_run_id, &snapshot)
            .expect("same-id non-terminal source starts");
    assert!(matches!(
        WorkflowExecution::from_replay_log(
            binding,
            &snapshot,
            replay.log().clone(),
            &non_terminal_source,
        ),
        Err(WorkflowExecutionReplayError::SourceReplay(_))
    ));
}

#[test]
fn reconstruction_rejects_sequence_gaps_and_events_after_terminal_state() {
    let binding = binding();
    let snapshot = WorkflowCapabilitySnapshot::default();
    let node_id = binding.workflow_definition().nodes()[0].id();
    let mut execution = WorkflowExecution::start(
        binding.clone(),
        WorkflowRunId::from_uuid(uuid(5_001)),
        &snapshot,
    )
    .expect("bound workflow starts");
    execution
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready");
    execution
        .mark_succeeded(node_id)
        .expect("node completes terminally");

    let mut gap_payload = serde_json::to_value(execution.log()).expect("log serializes");
    gap_payload["events"][1]["sequence"] = serde_json::json!(9);
    let gap_log = serde_json::from_value(gap_payload).expect("typed V1 log deserializes");
    let gap_error = WorkflowExecution::from_log(binding.clone(), &snapshot, gap_log)
        .expect_err("sequence gaps must fail closed");
    assert_eq!(
        gap_error,
        WorkflowExecutionReplayError::SequenceMismatch {
            expected: 2,
            provided: 9,
        }
    );

    let mut after_terminal_payload = serde_json::to_value(execution.log()).expect("log serializes");
    after_terminal_payload["events"]
        .as_array_mut()
        .expect("events are an array")
        .push(serde_json::json!({
            "event": "node_claimed",
            "sequence": 5,
            "node_id": node_id.as_uuid(),
            "attempt": 1
        }));
    let after_terminal_log =
        serde_json::from_value(after_terminal_payload).expect("typed V1 log deserializes");
    let terminal_error = WorkflowExecution::from_log(binding, &snapshot, after_terminal_log)
        .expect_err("events after a terminal marker must fail closed");
    assert_eq!(
        terminal_error,
        WorkflowExecutionReplayError::EventAfterTerminal { sequence: 5 }
    );
}

#[test]
fn reconstruction_rejects_wrong_context_source_and_unknown_schema() {
    let binding = binding();
    let snapshot = WorkflowCapabilitySnapshot::default();
    let execution = WorkflowExecution::start(
        binding.clone(),
        WorkflowRunId::from_uuid(uuid(6_001)),
        &snapshot,
    )
    .expect("bound workflow starts");
    let mut source_payload = serde_json::to_value(execution.log()).expect("log serializes");
    source_payload["context_source"]["commit_id"] = serde_json::json!(Uuid::from_u128(6_002));
    let wrong_source_log =
        serde_json::from_value(source_payload).expect("typed V1 log deserializes");

    assert!(matches!(
        WorkflowExecution::from_log(binding, &snapshot, wrong_source_log),
        Err(WorkflowExecutionReplayError::ContextSourceMismatch { .. })
    ));

    let mut schema_payload = serde_json::to_value(execution.log()).expect("log serializes");
    schema_payload["schema_version"] = serde_json::json!("v2");
    let schema_error =
        serde_json::from_value::<contextlab_workflow::WorkflowExecutionLogV1>(schema_payload)
            .expect_err("unknown execution schema must fail at the serialization boundary");
    assert!(schema_error.to_string().contains("v2"));
}

#[test]
fn execution_log_schema_rejects_unknown_event_fields_and_blank_failure_codes() {
    let binding = binding();
    let mut execution = WorkflowExecution::start(
        binding,
        WorkflowRunId::from_uuid(uuid(7_001)),
        &WorkflowCapabilitySnapshot::default(),
    )
    .expect("bound workflow starts");
    let node_id = execution
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready")
        .node_id();
    execution
        .mark_failed(
            node_id,
            WorkflowFailure::new("executor_timeout").expect("valid typed failure"),
        )
        .expect("node fails terminally");

    let mut unknown_field_payload = serde_json::to_value(execution.log()).expect("log serializes");
    unknown_field_payload["events"][1]["unexpected"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<contextlab_workflow::WorkflowExecutionLogV1>(
            unknown_field_payload
        )
        .is_err()
    );

    let mut blank_failure_payload = serde_json::to_value(execution.log()).expect("log serializes");
    blank_failure_payload["events"][2]["failure"]["code"] = serde_json::json!("  ");
    assert!(
        serde_json::from_value::<contextlab_workflow::WorkflowExecutionLogV1>(
            blank_failure_payload
        )
        .is_err()
    );
}

#[test]
fn terminal_reads_do_not_append_a_second_terminal_event() {
    let binding = binding();
    let node_id = binding.workflow_definition().nodes()[0].id();
    let mut execution = WorkflowExecution::start(
        binding,
        WorkflowRunId::from_uuid(uuid(8_001)),
        &WorkflowCapabilitySnapshot::default(),
    )
    .expect("bound workflow starts");
    execution
        .claim_next()
        .expect("claim succeeds")
        .expect("node is ready");
    execution
        .mark_succeeded(node_id)
        .expect("node completes terminally");
    let terminal_log = execution.log().clone();

    assert_eq!(
        execution.claim_next().expect("terminal state is readable"),
        None
    );
    assert_eq!(execution.log(), &terminal_log);
}

#[test]
fn execution_log_seals_canonical_capability_snapshot_and_digest() {
    let (_binding, snapshot, execution) = capability_log();
    let payload = serde_json::to_value(execution.log()).expect("log serializes");

    assert_eq!(payload["capability_snapshot"]["schema_version"], "v1");
    assert_eq!(
        payload["capability_snapshot"]["capabilities"]
            .as_array()
            .expect("capabilities are an array")
            .iter()
            .map(|entry| entry["capability"].clone())
            .collect::<Vec<_>>(),
        vec![
            serde_json::json!("workflow.alpha"),
            serde_json::json!("workflow.zeta")
        ]
    );
    assert_eq!(
        payload["capability_snapshot_digest"],
        snapshot.canonical_digest()
    );

    let reordered = WorkflowCapabilitySnapshot::new([
        WorkflowCapability::new("workflow.zeta", WorkflowCapabilityVersion::new(2, 0, 0))
            .expect("valid zeta capability"),
        WorkflowCapability::new("workflow.alpha", WorkflowCapabilityVersion::new(1, 1, 0))
            .expect("valid alpha capability"),
    ])
    .expect("unique reordered snapshot");
    let reordered_execution = WorkflowExecution::start(
        capability_binding(),
        WorkflowRunId::from_uuid(uuid(9_001)),
        &reordered,
    )
    .expect("reordered input is normalized before sealing");
    assert_eq!(
        serde_json::to_value(reordered_execution.log()).expect("log serializes"),
        payload
    );
}

#[test]
fn replay_rejects_a_compatible_but_different_capability_snapshot() {
    let (binding, snapshot, execution) = capability_log();
    let compatible_different = capability_snapshot(1, true);

    assert!(matches!(
        WorkflowExecution::from_log(binding, &compatible_different, execution.log().clone()),
        Err(WorkflowExecutionReplayError::CapabilitySnapshotMismatch { .. })
    ));

    let missing_capability = capability_snapshot(0, false);
    assert!(matches!(
        WorkflowExecution::from_log(
            capability_binding(),
            &missing_capability,
            execution.log().clone()
        ),
        Err(WorkflowExecutionReplayError::CapabilitySnapshotMismatch { .. })
    ));
    assert_eq!(
        snapshot.canonical_digest(),
        execution.log().capability_snapshot_digest()
    );
}

#[test]
fn replay_rejects_snapshot_ordering_duplicates_and_digest_tampering() {
    let (binding, snapshot, execution) = capability_log();
    let payload = serde_json::to_value(execution.log()).expect("log serializes");

    let mut reordered = payload.clone();
    reordered["capability_snapshot"]["capabilities"] = serde_json::json!([
        {"capability": "workflow.zeta", "version": "2.0.0"},
        {"capability": "workflow.alpha", "version": "1.1.0"}
    ]);
    let reordered_log = serde_json::from_value(reordered).expect("reordered V1 log deserializes");
    assert!(matches!(
        WorkflowExecution::from_log(binding.clone(), &snapshot, reordered_log),
        Err(WorkflowExecutionReplayError::CapabilitySnapshotOrderingMismatch { .. })
    ));

    let mut duplicate = payload.clone();
    duplicate["capability_snapshot"]["capabilities"] = serde_json::json!([
        {"capability": "workflow.alpha", "version": "1.1.0"},
        {"capability": "workflow.alpha", "version": "1.1.0"}
    ]);
    let duplicate_log = serde_json::from_value(duplicate).expect("duplicate V1 log deserializes");
    assert!(matches!(
        WorkflowExecution::from_log(binding.clone(), &snapshot, duplicate_log),
        Err(WorkflowExecutionReplayError::CapabilitySnapshotDuplicate { .. })
    ));

    let mut tampered_digest = payload;
    tampered_digest["capability_snapshot_digest"] = serde_json::json!("sha256:tampered");
    let tampered_log =
        serde_json::from_value(tampered_digest).expect("tampered V1 log deserializes");
    assert!(matches!(
        WorkflowExecution::from_log(binding, &snapshot, tampered_log),
        Err(WorkflowExecutionReplayError::CapabilitySnapshotDigestMismatch { .. })
    ));
}

#[test]
fn capability_snapshot_log_schema_rejects_unknown_fields_and_unsupported_schema() {
    let (_binding, _snapshot, execution) = capability_log();

    let mut unknown_field = serde_json::to_value(execution.log()).expect("log serializes");
    unknown_field["capability_snapshot"]["unexpected"] = serde_json::json!(true);
    assert!(
        serde_json::from_value::<contextlab_workflow::WorkflowExecutionLogV1>(unknown_field)
            .is_err()
    );

    let mut unsupported_schema = serde_json::to_value(execution.log()).expect("log serializes");
    unsupported_schema["capability_snapshot"]["schema_version"] = serde_json::json!("v2");
    assert!(
        serde_json::from_value::<contextlab_workflow::WorkflowExecutionLogV1>(unsupported_schema)
            .is_err()
    );
}
