//! Contract tests for the provider-free workflow read projection.

use contextlab_workflow::{
    WorkflowCapability, WorkflowCapabilityRequirement, WorkflowCapabilitySnapshot,
    WorkflowCapabilityVersion, WorkflowDefinition, WorkflowEdge, WorkflowEdgeId, WorkflowFailure,
    WorkflowId, WorkflowNode, WorkflowNodeId, WorkflowRevision, WorkflowRunId, WorkflowScheduler,
};
use serde_json::json;
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn revision() -> WorkflowRevision {
    WorkflowRevision::new(1).expect("non-zero revision")
}

#[test]
fn status_projection_serializes_a_redacted_v1_view_of_failed_and_replayed_runs() {
    let first_node_id = WorkflowNodeId::from_uuid(uuid(1));
    let independent_node_id = WorkflowNodeId::from_uuid(uuid(2));
    let dependent_node_id = WorkflowNodeId::from_uuid(uuid(3));
    let first_edge_id = WorkflowEdgeId::from_uuid(uuid(20));
    let second_edge_id = WorkflowEdgeId::from_uuid(uuid(21));
    let workflow_id = WorkflowId::from_uuid(uuid(10));
    let source_run_id = WorkflowRunId::from_uuid(uuid(30));
    let replay_run_id = WorkflowRunId::from_uuid(uuid(31));
    let alpha_requirement = WorkflowCapabilityRequirement::new(
        "context.alpha",
        WorkflowCapabilityVersion::new(1, 0, 0),
    )
    .expect("valid capability requirement");
    let zeta_requirement =
        WorkflowCapabilityRequirement::new("context.zeta", WorkflowCapabilityVersion::new(2, 1, 0))
            .expect("valid capability requirement");
    let workflow = WorkflowDefinition::new(
        workflow_id,
        revision(),
        vec![
            WorkflowNode::new(dependent_node_id, revision()),
            WorkflowNode::new(independent_node_id, revision()),
            WorkflowNode::with_capability_requirements(
                first_node_id,
                revision(),
                vec![zeta_requirement, alpha_requirement],
            )
            .expect("unique capability requirements"),
        ],
        vec![
            WorkflowEdge::new(
                second_edge_id,
                revision(),
                independent_node_id,
                dependent_node_id,
            )
            .expect("valid dependency"),
            WorkflowEdge::new(first_edge_id, revision(), first_node_id, dependent_node_id)
                .expect("valid dependency"),
        ],
    )
    .expect("valid workflow");
    let capability_snapshot = WorkflowCapabilitySnapshot::new(vec![
        WorkflowCapability::new("context.zeta", WorkflowCapabilityVersion::new(2, 1, 3))
            .expect("valid capability"),
        WorkflowCapability::new("context.alpha", WorkflowCapabilityVersion::new(1, 2, 0))
            .expect("valid capability"),
    ])
    .expect("unique capabilities");
    let mut scheduler = WorkflowScheduler::new(workflow, source_run_id, &capability_snapshot)
        .expect("compatible capability snapshot");

    let claim = scheduler
        .claim_next()
        .expect("claim operation succeeds")
        .expect("first node is ready");
    assert_eq!(claim.node_id(), first_node_id);
    scheduler
        .mark_failed(
            first_node_id,
            WorkflowFailure::new("executor_timeout: test-secret").expect("valid failure code"),
        )
        .expect("failure transition succeeds");

    let source_projection = scheduler
        .status_projection()
        .expect("consistent scheduler projects a read model");

    assert_eq!(
        serde_json::to_value(&source_projection).expect("projection serializes"),
        json!({
            "schema_version": "v1",
            "workflow_id": workflow_id.as_uuid(),
            "workflow_revision": 1,
            "run_id": source_run_id.as_uuid(),
            "replay_of": null,
            "run_state": "failed",
            "nodes": [
                {
                    "node_id": first_node_id.as_uuid(),
                    "node_revision": 1,
                    "capability_statuses": [
                        {
                            "capability": "context.alpha",
                            "required_version": "1.0.0",
                            "provided_version": "1.2.0",
                            "status": "satisfied"
                        },
                        {
                            "capability": "context.zeta",
                            "required_version": "2.1.0",
                            "provided_version": "2.1.3",
                            "status": "satisfied"
                        }
                    ],
                    "state": {
                        "state": "failed",
                        "attempt": 1,
                        "failure": { "redacted": true }
                    }
                },
                {
                    "node_id": independent_node_id.as_uuid(),
                    "node_revision": 1,
                    "capability_statuses": [],
                    "state": { "state": "blocked" }
                },
                {
                    "node_id": dependent_node_id.as_uuid(),
                    "node_revision": 1,
                    "capability_statuses": [],
                    "state": { "state": "blocked" }
                }
            ],
            "edges": [
                {
                    "edge_id": first_edge_id.as_uuid(),
                    "edge_revision": 1,
                    "source_node_id": first_node_id.as_uuid(),
                    "target_node_id": dependent_node_id.as_uuid()
                },
                {
                    "edge_id": second_edge_id.as_uuid(),
                    "edge_revision": 1,
                    "source_node_id": independent_node_id.as_uuid(),
                    "target_node_id": dependent_node_id.as_uuid()
                }
            ]
        })
    );
    assert!(
        !serde_json::to_string(&source_projection)
            .expect("projection serializes")
            .contains("test-secret")
    );

    let replay_projection = scheduler
        .replay(replay_run_id)
        .expect("terminal run is replayable")
        .status_projection()
        .expect("replay projects a read model");
    let replay = serde_json::to_value(replay_projection).expect("projection serializes");

    assert_eq!(replay["schema_version"], "v1");
    assert_eq!(replay["workflow_id"], json!(workflow_id.as_uuid()));
    assert_eq!(replay["run_id"], json!(replay_run_id.as_uuid()));
    assert_eq!(replay["replay_of"], json!(source_run_id.as_uuid()));
    assert_eq!(replay["run_state"], "pending");
    assert_eq!(replay["nodes"][0]["state"], json!({ "state": "pending" }));
    assert_eq!(replay["nodes"][1]["state"], json!({ "state": "pending" }));
    assert_eq!(replay["nodes"][2]["state"], json!({ "state": "pending" }));
}
