//! Redacted, transport-ready status for one validated workflow execution log.

use crate::{
    ContextCommitSource, WorkflowContextBinding, WorkflowContextBindingId, WorkflowExecution,
    WorkflowExecutionLogV1, WorkflowExecutionReplayError, WorkflowNodeState, WorkflowRevision,
    WorkflowRunId, WorkflowRunState,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The explicit schema version for an execution status read projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowExecutionStatusProjectionSchemaVersion {
    /// The first stable execution status projection schema.
    V1,
}

/// Counts of node states in one validated workflow execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowNodeStatusCountsV1 {
    pending: u64,
    running: u64,
    succeeded: u64,
    failed: u64,
    blocked: u64,
}

impl WorkflowNodeStatusCountsV1 {
    /// Returns the number of pending nodes.
    #[must_use]
    pub const fn pending(self) -> u64 {
        self.pending
    }

    /// Returns the number of running nodes.
    #[must_use]
    pub const fn running(self) -> u64 {
        self.running
    }

    /// Returns the number of succeeded nodes.
    #[must_use]
    pub const fn succeeded(self) -> u64 {
        self.succeeded
    }

    /// Returns the number of failed nodes.
    #[must_use]
    pub const fn failed(self) -> u64 {
        self.failed
    }

    /// Returns the number of blocked nodes.
    #[must_use]
    pub const fn blocked(self) -> u64 {
        self.blocked
    }

    fn record(&mut self, state: &WorkflowNodeState) {
        match state {
            WorkflowNodeState::Pending => self.pending += 1,
            WorkflowNodeState::Running { .. } => self.running += 1,
            WorkflowNodeState::Succeeded { .. } => self.succeeded += 1,
            WorkflowNodeState::Failed { .. } => self.failed += 1,
            WorkflowNodeState::Blocked => self.blocked += 1,
        }
    }
}

/// A provider-free, schema-versioned read projection of one validated execution log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowExecutionStatusProjectionV1 {
    schema_version: WorkflowExecutionStatusProjectionSchemaVersion,
    run_id: WorkflowRunId,
    binding_id: WorkflowContextBindingId,
    workflow_id: crate::WorkflowId,
    workflow_revision: WorkflowRevision,
    context_source: ContextCommitSource,
    run_state: WorkflowRunState,
    event_count: u64,
    last_sequence: u64,
    replay_of: Option<WorkflowRunId>,
    capability_snapshot_digest: String,
    node_status_counts: WorkflowNodeStatusCountsV1,
}

impl WorkflowExecutionStatusProjectionV1 {
    /// Validates and projects a root execution log without exposing event payloads.
    pub fn from_log(
        binding: WorkflowContextBinding,
        snapshot: &crate::WorkflowCapabilitySnapshot,
        log: WorkflowExecutionLogV1,
    ) -> Result<Self, WorkflowExecutionStatusProjectionError> {
        let projection_binding = binding.clone();
        let execution = WorkflowExecution::from_log(binding, snapshot, log)
            .map_err(WorkflowExecutionStatusProjectionError::Replay)?;
        Self::from_validated_execution(&execution, &projection_binding)
    }

    /// Validates and projects a replay log against its exact validated terminal source.
    pub fn from_replay_log(
        binding: WorkflowContextBinding,
        snapshot: &crate::WorkflowCapabilitySnapshot,
        log: WorkflowExecutionLogV1,
        source: &WorkflowExecution,
    ) -> Result<Self, WorkflowExecutionStatusProjectionError> {
        let projection_binding = binding.clone();
        let execution = WorkflowExecution::from_replay_log(binding, snapshot, log, source)
            .map_err(WorkflowExecutionStatusProjectionError::Replay)?;
        Self::from_validated_execution(&execution, &projection_binding)
    }

    fn from_validated_execution(
        execution: &WorkflowExecution,
        binding: &WorkflowContextBinding,
    ) -> Result<Self, WorkflowExecutionStatusProjectionError> {
        let log = execution.log();
        let mut node_status_counts = WorkflowNodeStatusCountsV1::default();
        for node in binding.workflow_definition().nodes() {
            let state = execution.node_state(node.id()).ok_or(
                WorkflowExecutionStatusProjectionError::MissingNodeState { node_id: node.id() },
            )?;
            node_status_counts.record(state);
        }

        let last_sequence = u64::try_from(log.events().len()).map_err(|_| {
            WorkflowExecutionStatusProjectionError::EventCountOverflow {
                event_count: log.events().len(),
            }
        })?;

        Ok(Self {
            schema_version: WorkflowExecutionStatusProjectionSchemaVersion::V1,
            run_id: log.run_id(),
            binding_id: binding.id(),
            workflow_id: binding.workflow_id(),
            workflow_revision: binding.workflow_revision(),
            context_source: binding.context_source(),
            run_state: execution.run_state(),
            event_count: last_sequence,
            last_sequence,
            replay_of: log.replay_of(),
            capability_snapshot_digest: log.capability_snapshot_digest().to_owned(),
            node_status_counts,
        })
    }

    /// Returns the explicit projection schema version.
    #[must_use]
    pub const fn schema_version(&self) -> WorkflowExecutionStatusProjectionSchemaVersion {
        self.schema_version
    }

    /// Returns the exact run identifier.
    #[must_use]
    pub const fn run_id(&self) -> WorkflowRunId {
        self.run_id
    }

    /// Returns the exact Context-to-Workflow binding identifier.
    #[must_use]
    pub const fn binding_id(&self) -> WorkflowContextBindingId {
        self.binding_id
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

    /// Returns the exact immutable Context commit source.
    #[must_use]
    pub const fn context_source(&self) -> ContextCommitSource {
        self.context_source
    }

    /// Returns the reconstructed run state.
    #[must_use]
    pub const fn run_state(&self) -> WorkflowRunState {
        self.run_state
    }

    /// Returns the number of validated canonical events.
    #[must_use]
    pub const fn event_count(&self) -> u64 {
        self.event_count
    }

    /// Returns the final canonical event sequence number.
    #[must_use]
    pub const fn last_sequence(&self) -> u64 {
        self.last_sequence
    }

    /// Returns the exact source run when this execution is a validated replay.
    #[must_use]
    pub const fn replay_of(&self) -> Option<WorkflowRunId> {
        self.replay_of
    }

    /// Returns the digest of the sealed capability snapshot.
    #[must_use]
    pub fn capability_snapshot_digest(&self) -> &str {
        &self.capability_snapshot_digest
    }

    /// Returns deterministic counts without exposing node or provider payloads.
    #[must_use]
    pub const fn node_status_counts(&self) -> WorkflowNodeStatusCountsV1 {
        self.node_status_counts
    }
}

/// Errors raised when an execution log cannot be safely projected.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowExecutionStatusProjectionError {
    /// The existing deterministic replay validator rejected the log or its scope.
    #[error("workflow execution status projection rejected the execution log: {0}")]
    Replay(WorkflowExecutionReplayError),
    /// A validated execution did not provide state for a bound workflow node.
    #[error("workflow execution status projection is missing state for node {node_id:?}")]
    MissingNodeState {
        /// The bound node whose state was absent.
        node_id: crate::WorkflowNodeId,
    },
    /// The host usize could not be represented in the transport schema's u64 count.
    #[error("workflow execution status projection event count {event_count} overflows u64")]
    EventCountOverflow {
        /// The host event count that could not be represented.
        event_count: usize,
    },
}
