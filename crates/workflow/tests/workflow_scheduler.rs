//! Behavioral tests for the provider-free workflow scheduler.

use contextlab_workflow::{
    WorkflowCapability, WorkflowCapabilityId, WorkflowCapabilityRequirement,
    WorkflowCapabilitySnapshot, WorkflowCapabilityValidationError, WorkflowCapabilityVersion,
    WorkflowDefinition, WorkflowEdge, WorkflowEdgeId, WorkflowFailure, WorkflowId, WorkflowNode,
    WorkflowNodeId, WorkflowNodeState, WorkflowRevision, WorkflowRunId, WorkflowRunState,
    WorkflowScheduler,
};
use serde_json::json;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn revision() -> WorkflowRevision {
    WorkflowRevision::new(1).expect("non-zero revision")
}

fn capability_version(major: u64, minor: u64, patch: u64) -> WorkflowCapabilityVersion {
    WorkflowCapabilityVersion::new(major, minor, patch)
}

fn capability_requirement(
    id: &str,
    minimum_version: WorkflowCapabilityVersion,
) -> WorkflowCapabilityRequirement {
    WorkflowCapabilityRequirement::new(id, minimum_version).expect("valid capability requirement")
}

fn capability(id: &str, version: WorkflowCapabilityVersion) -> WorkflowCapability {
    WorkflowCapability::new(id, version).expect("valid capability")
}

fn capability_snapshot(capabilities: Vec<WorkflowCapability>) -> WorkflowCapabilitySnapshot {
    WorkflowCapabilitySnapshot::new(capabilities).expect("unique capability snapshot")
}

#[test]
fn node_serializes_explicit_empty_capability_requirements() {
    let node_id = WorkflowNodeId::from_uuid(uuid(1_001));
    let workflow = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(1_010)),
        revision(),
        vec![WorkflowNode::new(node_id, revision())],
        Vec::new(),
    )
    .expect("valid workflow");

    let serialized = serde_json::to_value(workflow).expect("workflow serializes");

    assert_eq!(serialized["nodes"][0]["capability_requirements"], json!([]));
}

#[test]
fn scheduler_rejects_the_first_missing_capability_in_canonical_order() {
    let first_node = WorkflowNodeId::from_uuid(uuid(1_101));
    let second_node = WorkflowNodeId::from_uuid(uuid(1_102));
    let alpha = capability_requirement("context.alpha", capability_version(1, 0, 0));
    let zeta = capability_requirement("context.zeta", capability_version(1, 0, 0));
    let workflow = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(1_110)),
        revision(),
        vec![
            WorkflowNode::with_capability_requirements(second_node, revision(), vec![zeta])
                .expect("valid requirements"),
            WorkflowNode::with_capability_requirements(
                first_node,
                revision(),
                vec![
                    capability_requirement("context.beta", capability_version(1, 0, 0)),
                    alpha,
                ],
            )
            .expect("valid requirements"),
        ],
        Vec::new(),
    )
    .expect("valid workflow");

    let error = WorkflowScheduler::new(
        workflow,
        WorkflowRunId::from_uuid(uuid(1_120)),
        &capability_snapshot(Vec::new()),
    )
    .expect_err("a missing required capability must prevent scheduling");

    assert_eq!(
        error,
        WorkflowCapabilityValidationError::MissingCapability {
            node_id: first_node,
            capability: WorkflowCapabilityId::new("context.alpha").expect("valid capability ID"),
            required: capability_version(1, 0, 0),
        }
    );
}

#[test]
fn scheduler_rejects_an_incompatible_capability_before_scheduling() {
    let node_id = WorkflowNodeId::from_uuid(uuid(1_201));
    let workflow = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(1_210)),
        revision(),
        vec![
            WorkflowNode::with_capability_requirements(
                node_id,
                revision(),
                vec![capability_requirement(
                    "context.execute",
                    capability_version(1, 2, 0),
                )],
            )
            .expect("valid requirements"),
        ],
        Vec::new(),
    )
    .expect("valid workflow");

    let error = WorkflowScheduler::new(
        workflow,
        WorkflowRunId::from_uuid(uuid(1_220)),
        &capability_snapshot(vec![capability(
            "context.execute",
            capability_version(1, 1, 9),
        )]),
    )
    .expect_err("an incompatible required capability must prevent scheduling");

    assert_eq!(
        error,
        WorkflowCapabilityValidationError::IncompatibleCapability {
            node_id,
            capability: WorkflowCapabilityId::new("context.execute").expect("valid capability ID"),
            required: capability_version(1, 2, 0),
            provided: capability_version(1, 1, 9),
        }
    );
}

#[test]
fn scheduler_claims_a_node_only_after_its_capabilities_validate() {
    let node_id = WorkflowNodeId::from_uuid(uuid(1_301));
    let workflow = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(1_310)),
        revision(),
        vec![
            WorkflowNode::with_capability_requirements(
                node_id,
                revision(),
                vec![capability_requirement(
                    "context.execute",
                    capability_version(1, 2, 0),
                )],
            )
            .expect("valid requirements"),
        ],
        Vec::new(),
    )
    .expect("valid workflow");
    let mut scheduler = WorkflowScheduler::new(
        workflow,
        WorkflowRunId::from_uuid(uuid(1_320)),
        &capability_snapshot(vec![capability(
            "context.execute",
            capability_version(1, 2, 3),
        )]),
    )
    .expect("compatible capability snapshot");

    assert_eq!(
        scheduler
            .claim_next()
            .expect("validated scheduler can claim work")
            .expect("node is ready")
            .node_id(),
        node_id
    );
}

#[test]
fn scheduler_claims_ready_nodes_by_uuid_and_releases_dependents() {
    let first = WorkflowNodeId::from_uuid(uuid(1));
    let second = WorkflowNodeId::from_uuid(uuid(2));
    let dependent = WorkflowNodeId::from_uuid(uuid(3));

    let workflow = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(10)),
        revision(),
        vec![
            WorkflowNode::new(dependent, revision()),
            WorkflowNode::new(second, revision()),
            WorkflowNode::new(first, revision()),
        ],
        vec![
            WorkflowEdge::new(
                WorkflowEdgeId::from_uuid(uuid(20)),
                revision(),
                first,
                dependent,
            )
            .expect("valid dependency"),
        ],
    )
    .expect("valid workflow");
    let mut scheduler = WorkflowScheduler::new(
        workflow,
        WorkflowRunId::from_uuid(uuid(30)),
        &capability_snapshot(Vec::new()),
    )
    .expect("empty snapshot satisfies nodes without requirements");

    let claim = scheduler
        .claim_next()
        .expect("claim operation succeeds")
        .expect("first node is ready");
    assert_eq!(claim.node_id(), first);
    assert_eq!(claim.attempt(), 1);

    scheduler
        .mark_succeeded(first)
        .expect("first node succeeds");

    let claim = scheduler
        .claim_next()
        .expect("claim operation succeeds")
        .expect("second node is ready");
    assert_eq!(claim.node_id(), second);
    scheduler
        .mark_succeeded(second)
        .expect("second node succeeds");

    let claim = scheduler
        .claim_next()
        .expect("claim operation succeeds")
        .expect("dependent node is ready after predecessor");
    assert_eq!(claim.node_id(), dependent);
    scheduler
        .mark_succeeded(dependent)
        .expect("dependent node succeeds");

    assert_eq!(scheduler.run_state(), WorkflowRunState::Succeeded);
    assert_eq!(
        scheduler.claim_next().expect("terminal run is readable"),
        None
    );
}
#[test]
fn definition_rejects_a_dependency_with_a_missing_target() {
    let source = WorkflowNodeId::from_uuid(uuid(101));
    let missing_target = WorkflowNodeId::from_uuid(uuid(102));

    let error = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(110)),
        revision(),
        vec![WorkflowNode::new(source, revision())],
        vec![
            WorkflowEdge::new(
                WorkflowEdgeId::from_uuid(uuid(120)),
                revision(),
                source,
                missing_target,
            )
            .expect("edge shape is valid before graph validation"),
        ],
    )
    .expect_err("definition must reject missing targets");

    assert!(error.to_string().contains("target"));
}
#[test]
fn definition_rejects_duplicate_node_identifiers() {
    let node_id = WorkflowNodeId::from_uuid(uuid(201));

    let error = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(210)),
        revision(),
        vec![
            WorkflowNode::new(node_id, revision()),
            WorkflowNode::new(node_id, revision()),
        ],
        Vec::new(),
    )
    .expect_err("definition must reject duplicate node identifiers");

    assert!(error.to_string().contains("duplicate"));
}
#[test]
fn definition_rejects_duplicate_edge_identifiers() {
    let source = WorkflowNodeId::from_uuid(uuid(301));
    let target = WorkflowNodeId::from_uuid(uuid(302));
    let edge_id = WorkflowEdgeId::from_uuid(uuid(320));
    let edge = WorkflowEdge::new(edge_id, revision(), source, target).expect("valid edge");

    let error = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(310)),
        revision(),
        vec![
            WorkflowNode::new(source, revision()),
            WorkflowNode::new(target, revision()),
        ],
        vec![edge.clone(), edge],
    )
    .expect_err("definition must reject duplicate edge identifiers");

    assert!(error.to_string().contains("edge"));
    assert!(error.to_string().contains("duplicated"));
}
#[test]
fn definition_rejects_cyclic_dependencies() {
    let first = WorkflowNodeId::from_uuid(uuid(401));
    let second = WorkflowNodeId::from_uuid(uuid(402));

    let error = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(410)),
        revision(),
        vec![
            WorkflowNode::new(first, revision()),
            WorkflowNode::new(second, revision()),
        ],
        vec![
            WorkflowEdge::new(
                WorkflowEdgeId::from_uuid(uuid(420)),
                revision(),
                first,
                second,
            )
            .expect("valid edge"),
            WorkflowEdge::new(
                WorkflowEdgeId::from_uuid(uuid(421)),
                revision(),
                second,
                first,
            )
            .expect("valid edge"),
        ],
    )
    .expect_err("definition must reject cycles");

    assert!(error.to_string().contains("cycle"));
}
#[test]
fn deserialization_rejects_a_zero_revision() {
    let workflow_id = WorkflowId::from_uuid(uuid(501));
    let node_id = WorkflowNodeId::from_uuid(uuid(502));
    let payload = json!({
        "id": workflow_id.as_uuid(),
        "revision": 0,
        "nodes": [{
            "id": node_id.as_uuid(),
            "revision": 1,
            "capability_requirements": []
        }],
        "edges": []
    });

    let error = serde_json::from_value::<WorkflowDefinition>(payload)
        .expect_err("deserialization must reject zero revisions");

    assert!(error.to_string().contains("positive"));
}
#[test]
fn deserialization_rejects_a_self_dependency() {
    let workflow_id = WorkflowId::from_uuid(uuid(601));
    let node_id = WorkflowNodeId::from_uuid(uuid(602));
    let edge_id = WorkflowEdgeId::from_uuid(uuid(603));
    let payload = json!({
        "id": workflow_id.as_uuid(),
        "revision": 1,
        "nodes": [{
            "id": node_id.as_uuid(),
            "revision": 1,
            "capability_requirements": []
        }],
        "edges": [{
            "id": edge_id.as_uuid(),
            "revision": 1,
            "source": node_id.as_uuid(),
            "target": node_id.as_uuid()
        }]
    });

    let error = serde_json::from_value::<WorkflowDefinition>(payload)
        .expect_err("deserialization must reject self dependencies");

    assert!(error.to_string().contains("itself"));
}
#[test]
fn failure_makes_a_run_terminal_and_blocks_unstarted_nodes() {
    let first = WorkflowNodeId::from_uuid(uuid(701));
    let second = WorkflowNodeId::from_uuid(uuid(702));
    let workflow = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(710)),
        revision(),
        vec![
            WorkflowNode::new(second, revision()),
            WorkflowNode::new(first, revision()),
        ],
        Vec::new(),
    )
    .expect("valid independent workflow");
    let mut scheduler = WorkflowScheduler::new(
        workflow,
        WorkflowRunId::from_uuid(uuid(720)),
        &capability_snapshot(Vec::new()),
    )
    .expect("empty snapshot satisfies nodes without requirements");

    let claim = scheduler
        .claim_next()
        .expect("claim operation succeeds")
        .expect("first node is ready");
    assert_eq!(claim.node_id(), first);

    scheduler
        .mark_failed(
            first,
            WorkflowFailure::new("executor_timeout").expect("valid failure code"),
        )
        .expect("failure transition succeeds");

    assert_eq!(scheduler.run_state(), WorkflowRunState::Failed);
    assert!(matches!(
        scheduler.node_state(first),
        Some(WorkflowNodeState::Failed { failure, .. }) if failure.code().as_str() == "executor_timeout"
    ));
    assert!(matches!(
        scheduler.node_state(second),
        Some(WorkflowNodeState::Blocked)
    ));
    assert_eq!(
        scheduler.claim_next().expect("terminal run is readable"),
        None
    );
}
#[test]
fn replay_creates_a_fresh_pending_run_from_a_terminal_execution() {
    let node_id = WorkflowNodeId::from_uuid(uuid(801));
    let original_run_id = WorkflowRunId::from_uuid(uuid(820));
    let replay_run_id = WorkflowRunId::from_uuid(uuid(821));
    let workflow = WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(810)),
        revision(),
        vec![WorkflowNode::new(node_id, revision())],
        Vec::new(),
    )
    .expect("valid workflow");
    let mut scheduler =
        WorkflowScheduler::new(workflow, original_run_id, &capability_snapshot(Vec::new()))
            .expect("empty snapshot satisfies nodes without requirements");

    scheduler
        .claim_next()
        .expect("claim operation succeeds")
        .expect("node is ready");
    scheduler
        .mark_failed(
            node_id,
            WorkflowFailure::new("executor_timeout").expect("valid failure code"),
        )
        .expect("failure transition succeeds");

    let mut replay = scheduler
        .replay(replay_run_id)
        .expect("terminal execution is replayable");

    assert_eq!(replay.run_id(), replay_run_id);
    assert_eq!(replay.replay_of(), Some(original_run_id));
    assert_eq!(replay.run_state(), WorkflowRunState::Pending);
    assert!(matches!(
        replay.node_state(node_id),
        Some(WorkflowNodeState::Pending)
    ));
    assert_eq!(
        replay
            .claim_next()
            .expect("replay claim operation succeeds")
            .expect("replay node is ready")
            .node_id(),
        node_id
    );
}
#[test]
fn replay_rejects_an_active_execution() {
    let node_id = WorkflowNodeId::from_uuid(uuid(901));
    let scheduler = WorkflowScheduler::new(
        WorkflowDefinition::new(
            WorkflowId::from_uuid(uuid(910)),
            revision(),
            vec![WorkflowNode::new(node_id, revision())],
            Vec::new(),
        )
        .expect("valid workflow"),
        WorkflowRunId::from_uuid(uuid(920)),
        &capability_snapshot(Vec::new()),
    )
    .expect("empty snapshot satisfies nodes without requirements");

    let error = scheduler
        .replay(WorkflowRunId::from_uuid(uuid(921)))
        .expect_err("active runs must not be replayed");

    assert!(error.to_string().contains("terminal"));
}

#[test]
fn deserialization_validates_dependency_endpoints() {
    let workflow_id = WorkflowId::from_uuid(uuid(1001));
    let source = WorkflowNodeId::from_uuid(uuid(1002));
    let missing_target = WorkflowNodeId::from_uuid(uuid(1003));
    let edge_id = WorkflowEdgeId::from_uuid(uuid(1004));
    let payload = json!({
        "id": workflow_id.as_uuid(),
        "revision": 1,
        "nodes": [{
            "id": source.as_uuid(),
            "revision": 1,
            "capability_requirements": []
        }],
        "edges": [{
            "id": edge_id.as_uuid(),
            "revision": 1,
            "source": source.as_uuid(),
            "target": missing_target.as_uuid()
        }]
    });

    let error = serde_json::from_value::<WorkflowDefinition>(payload)
        .expect_err("deserialization must validate graph endpoints");

    assert!(error.to_string().contains("missing target"));
}
