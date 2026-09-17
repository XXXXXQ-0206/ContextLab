use crate::{
    WorkflowCapabilityId, WorkflowCapabilityVersion, WorkflowEdgeId, WorkflowNodeId,
    WorkflowNodeState, WorkflowRevision, WorkflowRunId, WorkflowRunState, WorkflowScheduler,
};
use serde::Serialize;
use thiserror::Error;

/// The explicit schema version for a workflow status read projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatusProjectionSchemaVersion {
    /// The initial stable workflow status projection schema.
    V1,
}

/// A provider-free, redacted, read-only projection of one workflow execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowStatusProjectionV1 {
    schema_version: WorkflowStatusProjectionSchemaVersion,
    workflow_id: crate::WorkflowId,
    workflow_revision: WorkflowRevision,
    run_id: WorkflowRunId,
    replay_of: Option<WorkflowRunId>,
    run_state: WorkflowRunState,
    nodes: Vec<WorkflowStatusProjectionNodeV1>,
    edges: Vec<WorkflowStatusProjectionEdgeV1>,
}

impl WorkflowStatusProjectionV1 {
    pub(crate) fn from_scheduler(
        scheduler: &WorkflowScheduler,
    ) -> Result<Self, WorkflowStatusProjectionError> {
        let nodes = scheduler
            .definition()
            .nodes()
            .iter()
            .map(|node| {
                let state = scheduler.node_state(node.id()).ok_or(
                    WorkflowStatusProjectionError::MissingNodeState { node_id: node.id() },
                )?;
                let capability_statuses = node
                    .capability_requirements()
                    .iter()
                    .map(|requirement| {
                        let provided_version = scheduler
                            .capability_snapshot
                            .version(requirement.capability())
                            .ok_or_else(|| WorkflowStatusProjectionError::MissingCapability {
                                node_id: node.id(),
                                capability: requirement.capability().clone(),
                                required_version: requirement.minimum_version(),
                            })?;
                        if !provided_version.satisfies(requirement.minimum_version()) {
                            return Err(WorkflowStatusProjectionError::IncompatibleCapability {
                                node_id: node.id(),
                                capability: requirement.capability().clone(),
                                required_version: requirement.minimum_version(),
                                provided_version,
                            });
                        }
                        Ok(WorkflowCapabilityStatusProjectionV1 {
                            capability: requirement.capability().clone(),
                            required_version: requirement.minimum_version(),
                            provided_version,
                            status: WorkflowCapabilityStatusV1::Satisfied,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(WorkflowStatusProjectionNodeV1 {
                    node_id: node.id(),
                    node_revision: node.revision(),
                    capability_statuses,
                    state: WorkflowStatusProjectionNodeStateV1::from(state),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let edges = scheduler
            .definition()
            .edges()
            .iter()
            .map(|edge| WorkflowStatusProjectionEdgeV1 {
                edge_id: edge.id(),
                edge_revision: edge.revision(),
                source_node_id: edge.source(),
                target_node_id: edge.target(),
            })
            .collect();

        Ok(Self {
            schema_version: WorkflowStatusProjectionSchemaVersion::V1,
            workflow_id: scheduler.definition().id(),
            workflow_revision: scheduler.definition().revision(),
            run_id: scheduler.run_id(),
            replay_of: scheduler.replay_of(),
            run_state: scheduler.run_state(),
            nodes,
            edges,
        })
    }

    /// Returns the explicit projection schema version.
    #[must_use]
    pub const fn schema_version(&self) -> WorkflowStatusProjectionSchemaVersion {
        self.schema_version
    }

    /// Returns the stable workflow identifier.
    #[must_use]
    pub const fn workflow_id(&self) -> crate::WorkflowId {
        self.workflow_id
    }

    /// Returns the immutable workflow revision.
    #[must_use]
    pub const fn workflow_revision(&self) -> WorkflowRevision {
        self.workflow_revision
    }

    /// Returns the stable workflow run identifier.
    #[must_use]
    pub const fn run_id(&self) -> WorkflowRunId {
        self.run_id
    }

    /// Returns the terminal source run when this projection represents a replay.
    #[must_use]
    pub const fn replay_of(&self) -> Option<WorkflowRunId> {
        self.replay_of
    }

    /// Returns the current workflow execution state.
    #[must_use]
    pub const fn run_state(&self) -> WorkflowRunState {
        self.run_state
    }

    /// Returns nodes in ascending stable node identifier order.
    #[must_use]
    pub fn nodes(&self) -> &[WorkflowStatusProjectionNodeV1] {
        &self.nodes
    }

    /// Returns edges in ascending stable edge identifier order.
    #[must_use]
    pub fn edges(&self) -> &[WorkflowStatusProjectionEdgeV1] {
        &self.edges
    }
}

/// The compatibility state of one required workflow capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowCapabilityStatusV1 {
    /// The injected capability satisfies the immutable requirement.
    Satisfied,
}

/// A validated requirement and its injected capability status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowCapabilityStatusProjectionV1 {
    capability: WorkflowCapabilityId,
    required_version: WorkflowCapabilityVersion,
    provided_version: WorkflowCapabilityVersion,
    status: WorkflowCapabilityStatusV1,
}

impl WorkflowCapabilityStatusProjectionV1 {
    /// Returns the stable capability identifier.
    #[must_use]
    pub const fn capability(&self) -> &WorkflowCapabilityId {
        &self.capability
    }

    /// Returns the minimum version required by the workflow node.
    #[must_use]
    pub const fn required_version(&self) -> WorkflowCapabilityVersion {
        self.required_version
    }

    /// Returns the compatible version injected into the workflow run.
    #[must_use]
    pub const fn provided_version(&self) -> WorkflowCapabilityVersion {
        self.provided_version
    }

    /// Returns the capability compatibility state.
    #[must_use]
    pub const fn status(&self) -> WorkflowCapabilityStatusV1 {
        self.status
    }
}

/// One immutable workflow node and its provider-free execution state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowStatusProjectionNodeV1 {
    node_id: WorkflowNodeId,
    node_revision: WorkflowRevision,
    capability_statuses: Vec<WorkflowCapabilityStatusProjectionV1>,
    state: WorkflowStatusProjectionNodeStateV1,
}

impl WorkflowStatusProjectionNodeV1 {
    /// Returns the stable workflow node identifier.
    #[must_use]
    pub const fn node_id(&self) -> WorkflowNodeId {
        self.node_id
    }

    /// Returns the immutable workflow node revision.
    #[must_use]
    pub const fn node_revision(&self) -> WorkflowRevision {
        self.node_revision
    }

    /// Returns capability statuses in ascending capability identifier order.
    #[must_use]
    pub fn capability_statuses(&self) -> &[WorkflowCapabilityStatusProjectionV1] {
        &self.capability_statuses
    }

    /// Returns the redacted execution state for the node.
    #[must_use]
    pub const fn state(&self) -> &WorkflowStatusProjectionNodeStateV1 {
        &self.state
    }
}

/// A redacted node execution state in the V1 status projection schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum WorkflowStatusProjectionNodeStateV1 {
    /// The node is waiting for prerequisites.
    Pending,
    /// The node is currently claimed by the workflow scheduler.
    Running {
        /// The deterministic claim attempt number.
        attempt: u32,
    },
    /// The node completed successfully.
    Succeeded {
        /// The deterministic completion attempt number.
        attempt: u32,
    },
    /// The node failed with details redacted from this read model.
    Failed {
        /// The deterministic failed attempt number.
        attempt: u32,
        /// A marker confirming that failure details are intentionally omitted.
        failure: WorkflowStatusProjectionFailureV1,
    },
    /// The node did not start because another node failed.
    Blocked,
}

impl From<&WorkflowNodeState> for WorkflowStatusProjectionNodeStateV1 {
    fn from(value: &WorkflowNodeState) -> Self {
        match value {
            WorkflowNodeState::Pending => Self::Pending,
            WorkflowNodeState::Running { attempt } => Self::Running { attempt: *attempt },
            WorkflowNodeState::Succeeded { attempt } => Self::Succeeded { attempt: *attempt },
            WorkflowNodeState::Failed { attempt, .. } => Self::Failed {
                attempt: *attempt,
                failure: WorkflowStatusProjectionFailureV1::redacted(),
            },
            WorkflowNodeState::Blocked => Self::Blocked,
        }
    }
}

/// A marker that prevents failure details from crossing the read projection boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WorkflowStatusProjectionFailureV1 {
    redacted: bool,
}

impl WorkflowStatusProjectionFailureV1 {
    const fn redacted() -> Self {
        Self { redacted: true }
    }

    /// Returns whether the failure detail is redacted.
    #[must_use]
    pub const fn is_redacted(&self) -> bool {
        self.redacted
    }
}

/// One immutable directed dependency in the V1 status projection schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowStatusProjectionEdgeV1 {
    edge_id: WorkflowEdgeId,
    edge_revision: WorkflowRevision,
    source_node_id: WorkflowNodeId,
    target_node_id: WorkflowNodeId,
}

impl WorkflowStatusProjectionEdgeV1 {
    /// Returns the stable workflow edge identifier.
    #[must_use]
    pub const fn edge_id(&self) -> WorkflowEdgeId {
        self.edge_id
    }

    /// Returns the immutable workflow edge revision.
    #[must_use]
    pub const fn edge_revision(&self) -> WorkflowRevision {
        self.edge_revision
    }

    /// Returns the stable prerequisite node identifier.
    #[must_use]
    pub const fn source_node_id(&self) -> WorkflowNodeId {
        self.source_node_id
    }

    /// Returns the stable dependent node identifier.
    #[must_use]
    pub const fn target_node_id(&self) -> WorkflowNodeId {
        self.target_node_id
    }
}

/// Errors raised when a workflow run cannot be safely projected for reading.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowStatusProjectionError {
    /// A workflow definition node has no corresponding execution state.
    #[error("workflow status projection is missing state for node {node_id:?}")]
    MissingNodeState {
        /// The stable workflow node identifier.
        node_id: WorkflowNodeId,
    },
    /// The retained capability snapshot does not contain a declared requirement.
    #[error(
        "workflow status projection is missing capability {capability} at version {required_version} for node {node_id:?}"
    )]
    MissingCapability {
        /// The stable workflow node identifier.
        node_id: WorkflowNodeId,
        /// The missing stable capability identifier.
        capability: WorkflowCapabilityId,
        /// The minimum compatible capability version.
        required_version: WorkflowCapabilityVersion,
    },
    /// The retained capability snapshot cannot prove a declared requirement is compatible.
    #[error(
        "workflow status projection has incompatible capability {capability} for node {node_id:?}: requires {required_version}, found {provided_version}"
    )]
    IncompatibleCapability {
        /// The stable workflow node identifier.
        node_id: WorkflowNodeId,
        /// The incompatible stable capability identifier.
        capability: WorkflowCapabilityId,
        /// The minimum compatible capability version.
        required_version: WorkflowCapabilityVersion,
        /// The provided incompatible capability version.
        provided_version: WorkflowCapabilityVersion,
    },
}
