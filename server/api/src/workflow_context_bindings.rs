//! Private, redacted Workflow source-binding response contract.

use contextlab_context_core::ContextId;
use contextlab_versioning::CommitId as VersioningCommitId;
use contextlab_workflow::WorkflowContextBinding;
use serde::Serialize;

pub(crate) const WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1: &str =
    "contextlab.local-workflow-context-bindings.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalWorkflowContextBindingsResponse {
    schema_version: &'static str,
    context_id: String,
    commit_id: String,
    bindings: Vec<LocalWorkflowContextBindingSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalWorkflowContextBindingSummary {
    binding_id: String,
    workflow_id: String,
    workflow_revision: u64,
    node_count: usize,
    edge_count: usize,
}

impl LocalWorkflowContextBindingsResponse {
    pub(crate) fn from_bindings(
        context_id: ContextId,
        commit_id: VersioningCommitId,
        bindings: Vec<WorkflowContextBinding>,
    ) -> Self {
        Self {
            schema_version: WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
            context_id: context_id.to_string(),
            commit_id: commit_id.to_string(),
            bindings: bindings
                .into_iter()
                .map(|binding| LocalWorkflowContextBindingSummary {
                    binding_id: binding.id().as_uuid().to_string(),
                    workflow_id: binding.workflow_id().as_uuid().to_string(),
                    workflow_revision: binding.workflow_revision().get(),
                    node_count: binding.workflow_definition().nodes().len(),
                    edge_count: binding.workflow_definition().edges().len(),
                })
                .collect(),
        }
    }
}
