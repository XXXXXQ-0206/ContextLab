//! Contract tests for immutable Workflow-to-Context source provenance.

use contextlab_context_core::ContextId;
use contextlab_versioning::CommitId;
use contextlab_workflow::{
    ContextCommitSource, WorkflowContextBinding, WorkflowContextBindingId, WorkflowDefinition,
    WorkflowId, WorkflowNode, WorkflowNodeId, WorkflowRevision,
};
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn revision() -> WorkflowRevision {
    WorkflowRevision::new(1).expect("non-zero workflow revision")
}

fn definition() -> WorkflowDefinition {
    WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(1_001)),
        revision(),
        vec![WorkflowNode::new(
            WorkflowNodeId::from_uuid(uuid(1_002)),
            revision(),
        )],
        Vec::new(),
    )
    .expect("valid sealed workflow definition")
}

#[test]
fn binding_seals_a_definition_to_one_exact_context_commit() {
    let source = ContextCommitSource::new(
        ContextId::from_uuid(uuid(2_001)),
        CommitId::from_uuid(uuid(2_002)),
    );
    let definition = definition();
    let binding = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(3_001)),
        definition.clone(),
        source,
    );

    assert_eq!(
        binding.id(),
        WorkflowContextBindingId::from_uuid(uuid(3_001))
    );
    assert_eq!(binding.context_source(), source);
    assert_eq!(binding.workflow_definition(), &definition);
    assert_eq!(binding.workflow_id(), definition.id());
    assert_eq!(binding.workflow_revision(), definition.revision());
}

#[test]
fn binding_rejects_unknown_serialized_fields() {
    let binding = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(3_101)),
        definition(),
        ContextCommitSource::new(
            ContextId::from_uuid(uuid(2_101)),
            CommitId::from_uuid(uuid(2_102)),
        ),
    );
    let mut payload = serde_json::to_value(binding).expect("binding serializes");
    payload["unexpected"] = serde_json::Value::Bool(true);

    assert!(serde_json::from_value::<WorkflowContextBinding>(payload).is_err());
}
