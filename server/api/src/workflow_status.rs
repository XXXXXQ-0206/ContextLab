use serde::Serialize;

const WORKFLOW_CAPABILITY_STATUS_SCHEMA_VERSION: &str =
    "contextlab.local-workflow-capability-status.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkflowCapabilityAvailability {
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WorkflowCapabilityStatusV1 {
    schema_version: &'static str,
    capability: &'static str,
    enabled: bool,
    availability: WorkflowCapabilityAvailability,
    reason: &'static str,
}

impl Default for WorkflowCapabilityStatusV1 {
    fn default() -> Self {
        Self {
            schema_version: WORKFLOW_CAPABILITY_STATUS_SCHEMA_VERSION,
            capability: "workflow",
            enabled: false,
            availability: WorkflowCapabilityAvailability::Unavailable,
            reason: "shared_integration_not_registered",
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct UnavailableWorkflowCapabilityAdapter;

impl UnavailableWorkflowCapabilityAdapter {
    pub(crate) fn status(&self) -> WorkflowCapabilityStatusV1 {
        WorkflowCapabilityStatusV1::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UnavailableWorkflowCapabilityAdapter, WorkflowCapabilityAvailability,
        WorkflowCapabilityStatusV1,
    };

    #[test]
    fn default_status_is_typed_unavailable_and_disabled() {
        let status = UnavailableWorkflowCapabilityAdapter.status();

        assert_eq!(status, WorkflowCapabilityStatusV1::default());
        assert!(!status.enabled);
        assert_eq!(
            status.availability,
            WorkflowCapabilityAvailability::Unavailable
        );
        assert_eq!(status.reason, "shared_integration_not_registered");
    }
}
