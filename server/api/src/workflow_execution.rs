//! Private, redacted read adapter for one workflow execution status projection.
//!
//! The workflow crate owns execution validation and replay provenance. This module only
//! translates that safe projection into the local API contract and keeps storage integration
//! behind a read-only port.

use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_workflow::{
    WorkflowExecutionStatusProjectionV1, WorkflowNodeStatusCountsV1, WorkflowRunId,
    WorkflowRunState,
};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;

/// Stable private local workflow execution status schema.
pub(crate) const LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1: &str =
    "contextlab.local-workflow-execution-status.v1";

/// A safe, ordered count of workflow node execution states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalWorkflowExecutionNodeStatusCounts {
    pending: u64,
    running: u64,
    succeeded: u64,
    failed: u64,
    blocked: u64,
}

#[allow(dead_code)]
impl LocalWorkflowExecutionNodeStatusCounts {
    fn from_projection(counts: WorkflowNodeStatusCountsV1) -> Self {
        Self {
            pending: counts.pending(),
            running: counts.running(),
            succeeded: counts.succeeded(),
            failed: counts.failed(),
            blocked: counts.blocked(),
        }
    }

    /// Returns the number of pending nodes.
    pub(crate) const fn pending(self) -> u64 {
        self.pending
    }

    /// Returns the number of running nodes.
    pub(crate) const fn running(self) -> u64 {
        self.running
    }

    /// Returns the number of succeeded nodes.
    pub(crate) const fn succeeded(self) -> u64 {
        self.succeeded
    }

    /// Returns the number of failed nodes.
    pub(crate) const fn failed(self) -> u64 {
        self.failed
    }

    /// Returns the number of blocked nodes.
    pub(crate) const fn blocked(self) -> u64 {
        self.blocked
    }
}

/// Stable API representation of the provider-free workflow run state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LocalWorkflowExecutionRunState {
    Pending,
    Running,
    Succeeded,
    Failed,
}

impl From<WorkflowRunState> for LocalWorkflowExecutionRunState {
    fn from(state: WorkflowRunState) -> Self {
        match state {
            WorkflowRunState::Pending => Self::Pending,
            WorkflowRunState::Running => Self::Running,
            WorkflowRunState::Succeeded => Self::Succeeded,
            WorkflowRunState::Failed => Self::Failed,
        }
    }
}

/// Exact Context-scoped redacted workflow execution status resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalWorkflowExecutionStatusResource {
    schema_version: &'static str,
    context_id: String,
    context_commit_id: String,
    binding_id: String,
    workflow_id: String,
    workflow_revision: u64,
    run_id: String,
    replay_of: Option<String>,
    run_state: LocalWorkflowExecutionRunState,
    event_count: u64,
    last_event_sequence: u64,
    capability_snapshot_digest: String,
    node_status_counts: LocalWorkflowExecutionNodeStatusCounts,
}

#[allow(dead_code)]
impl LocalWorkflowExecutionStatusResource {
    /// Adapts the validated workflow execution projection without exposing raw events or failures.
    pub(crate) fn from_projection(projection: &WorkflowExecutionStatusProjectionV1) -> Self {
        let source = projection.context_source();
        Self {
            schema_version: LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1,
            context_id: source.context_id().as_uuid().to_string(),
            context_commit_id: source.commit_id().as_uuid().to_string(),
            binding_id: projection.binding_id().as_uuid().to_string(),
            workflow_id: projection.workflow_id().as_uuid().to_string(),
            workflow_revision: projection.workflow_revision().get(),
            run_id: projection.run_id().as_uuid().to_string(),
            replay_of: projection
                .replay_of()
                .map(|run_id| run_id.as_uuid().to_string()),
            run_state: projection.run_state().into(),
            event_count: projection.event_count(),
            last_event_sequence: projection.last_sequence(),
            capability_snapshot_digest: projection.capability_snapshot_digest().to_owned(),
            node_status_counts: LocalWorkflowExecutionNodeStatusCounts::from_projection(
                projection.node_status_counts(),
            ),
        }
    }

    /// Validates both exact request keys before the resource crosses the API boundary.
    pub(crate) fn validate_for_scope(
        &self,
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> Result<(), WorkflowExecutionStatusStorageError> {
        if self.schema_version != LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1
            || self.context_id != context_id.as_uuid().to_string()
            || self.run_id != run_id.as_uuid().to_string()
        {
            return Err(WorkflowExecutionStatusStorageError::InvalidScope);
        }
        Ok(())
    }

    pub(crate) const fn schema_version(&self) -> &'static str {
        self.schema_version
    }

    pub(crate) fn context_id(&self) -> &str {
        &self.context_id
    }

    pub(crate) fn context_commit_id(&self) -> &str {
        &self.context_commit_id
    }

    pub(crate) fn binding_id(&self) -> &str {
        &self.binding_id
    }

    pub(crate) fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    pub(crate) const fn workflow_revision(&self) -> u64 {
        self.workflow_revision
    }

    pub(crate) fn run_id(&self) -> &str {
        &self.run_id
    }

    pub(crate) fn replay_of(&self) -> Option<&str> {
        self.replay_of.as_deref()
    }

    pub(crate) const fn run_state(&self) -> LocalWorkflowExecutionRunState {
        self.run_state
    }

    pub(crate) const fn event_count(&self) -> u64 {
        self.event_count
    }

    pub(crate) const fn last_event_sequence(&self) -> u64 {
        self.last_event_sequence
    }

    pub(crate) fn capability_snapshot_digest(&self) -> &str {
        &self.capability_snapshot_digest
    }

    pub(crate) const fn node_status_counts(&self) -> LocalWorkflowExecutionNodeStatusCounts {
        self.node_status_counts
    }
}

/// Typed failures at the private workflow execution status storage boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub(crate) enum WorkflowExecutionStatusStorageError {
    /// The backing execution status store is not registered or cannot be reached.
    #[error("workflow execution status storage is unavailable")]
    Unavailable,
    /// A returned resource does not match the exact requested Context and run.
    #[error("workflow execution status resource does not match the requested scope")]
    InvalidScope,
}

/// Read-only repository port for one exact Context and workflow run.
#[async_trait]
pub(crate) trait WorkflowExecutionStatusRepository: Send + Sync {
    /// Reads a redacted status resource, or returns `None` when the run is absent.
    async fn read(
        &self,
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> Result<Option<LocalWorkflowExecutionStatusResource>, WorkflowExecutionStatusStorageError>;

    /// Reads and validates both exact request keys before returning a resource.
    async fn read_scoped(
        &self,
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> Result<Option<LocalWorkflowExecutionStatusResource>, WorkflowExecutionStatusStorageError>
    {
        self.read(context_id, run_id)
            .await?
            .map(|resource| {
                resource.validate_for_scope(context_id, run_id)?;
                Ok(resource)
            })
            .transpose()
    }
}

/// Fail-closed default adapter used until a durable execution projection is registered.
#[derive(Debug, Default)]
pub(crate) struct UnavailableWorkflowExecutionStatusAdapter;

#[async_trait]
impl WorkflowExecutionStatusRepository for UnavailableWorkflowExecutionStatusAdapter {
    async fn read(
        &self,
        _context_id: ContextId,
        _run_id: WorkflowRunId,
    ) -> Result<Option<LocalWorkflowExecutionStatusResource>, WorkflowExecutionStatusStorageError>
    {
        Err(WorkflowExecutionStatusStorageError::Unavailable)
    }
}

/// Adapts the reusable storage projection repository to the private redacted API resource.
///
/// The default application state intentionally does not construct this adapter. A caller must
/// inject an explicit local repository, keeping absent runtime execution state fail-closed.
pub(crate) struct StorageWorkflowExecutionStatusAdapter {
    repository: Arc<dyn contextlab_storage::WorkflowExecutionStatusRepository>,
}

impl StorageWorkflowExecutionStatusAdapter {
    /// Creates an API read adapter over an immutable storage repository.
    pub(crate) fn new(
        repository: impl contextlab_storage::WorkflowExecutionStatusRepository + 'static,
    ) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }
}

#[async_trait]
impl WorkflowExecutionStatusRepository for StorageWorkflowExecutionStatusAdapter {
    async fn read(
        &self,
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> Result<Option<LocalWorkflowExecutionStatusResource>, WorkflowExecutionStatusStorageError>
    {
        self.repository
            .read_workflow_execution_status(context_id, run_id)
            .await
            .map_err(|error| match error {
                contextlab_storage::WorkflowExecutionStatusPersistenceError::InvalidScope {
                    ..
                } => WorkflowExecutionStatusStorageError::InvalidScope,
                contextlab_storage::WorkflowExecutionStatusPersistenceError::Conflict { .. }
                | contextlab_storage::WorkflowExecutionStatusPersistenceError::RepositoryUnavailable => {
                    WorkflowExecutionStatusStorageError::Unavailable
                }
            })
            .map(|projection| {
                projection.map(|projection| LocalWorkflowExecutionStatusResource::from_projection(&projection))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_storage::{
        InMemoryWorkflowExecutionStatusRepository, PersistWorkflowExecutionStatusV1,
        WorkflowExecutionStatusRepository as StorageWorkflowExecutionStatusRepository,
    };
    use contextlab_versioning::CommitId;
    use contextlab_workflow::{
        ContextCommitSource, WorkflowCapabilitySnapshot, WorkflowContextBinding,
        WorkflowContextBindingId, WorkflowDefinition, WorkflowExecution,
        WorkflowExecutionStatusProjectionV1, WorkflowId, WorkflowNode, WorkflowNodeId,
        WorkflowRevision,
    };
    use serde_json::json;
    use uuid::Uuid;

    fn resource(
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> LocalWorkflowExecutionStatusResource {
        LocalWorkflowExecutionStatusResource {
            schema_version: LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1,
            context_id: context_id.as_uuid().to_string(),
            context_commit_id: Uuid::new_v4().to_string(),
            binding_id: Uuid::new_v4().to_string(),
            workflow_id: Uuid::new_v4().to_string(),
            workflow_revision: 1,
            run_id: run_id.as_uuid().to_string(),
            replay_of: None,
            run_state: LocalWorkflowExecutionRunState::Succeeded,
            event_count: 3,
            last_event_sequence: 3,
            capability_snapshot_digest: "sha256:test".to_owned(),
            node_status_counts: LocalWorkflowExecutionNodeStatusCounts {
                pending: 0,
                running: 0,
                succeeded: 2,
                failed: 0,
                blocked: 0,
            },
        }
    }

    #[test]
    fn response_schema_is_stable_and_contains_no_raw_execution_data() {
        let context_id = ContextId::new();
        let run_id = WorkflowRunId::from_uuid(Uuid::new_v4());
        let value =
            serde_json::to_value(resource(context_id, run_id)).expect("resource serializes");

        assert_eq!(
            value["schema_version"],
            LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1
        );
        assert_eq!(value["context_id"], context_id.as_uuid().to_string());
        assert_eq!(value["run_id"], run_id.as_uuid().to_string());
        assert_eq!(value["last_event_sequence"], 3);
        assert_eq!(value["node_status_counts"]["succeeded"], 2);
        assert!(value.get("events").is_none());
        assert!(value.get("failure").is_none());
    }

    #[test]
    fn scope_validation_rejects_context_or_run_mismatch() {
        let context_id = ContextId::new();
        let run_id = WorkflowRunId::from_uuid(Uuid::new_v4());
        let resource = resource(context_id, run_id);

        assert!(resource.validate_for_scope(context_id, run_id).is_ok());
        assert_eq!(
            resource.validate_for_scope(ContextId::new(), run_id),
            Err(WorkflowExecutionStatusStorageError::InvalidScope)
        );
        assert_eq!(
            resource.validate_for_scope(context_id, WorkflowRunId::from_uuid(Uuid::new_v4())),
            Err(WorkflowExecutionStatusStorageError::InvalidScope)
        );
    }

    #[tokio::test]
    async fn default_adapter_fails_closed_without_a_repository() {
        let result = UnavailableWorkflowExecutionStatusAdapter
            .read(ContextId::new(), WorkflowRunId::from_uuid(Uuid::new_v4()))
            .await;

        assert_eq!(
            result,
            Err(WorkflowExecutionStatusStorageError::Unavailable)
        );
    }

    fn projection(
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> WorkflowExecutionStatusProjectionV1 {
        let revision = WorkflowRevision::new(1).expect("revision");
        let binding = WorkflowContextBinding::new(
            WorkflowContextBindingId::from_uuid(Uuid::new_v4()),
            WorkflowDefinition::new(
                WorkflowId::from_uuid(Uuid::new_v4()),
                revision,
                vec![WorkflowNode::new(
                    WorkflowNodeId::from_uuid(Uuid::new_v4()),
                    revision,
                )],
                Vec::new(),
            )
            .expect("definition"),
            ContextCommitSource::new(context_id, CommitId::from_uuid(Uuid::new_v4())),
        );
        let execution = WorkflowExecution::start(
            binding.clone(),
            run_id,
            &WorkflowCapabilitySnapshot::default(),
        )
        .expect("execution");
        WorkflowExecutionStatusProjectionV1::from_log(
            binding,
            &WorkflowCapabilitySnapshot::default(),
            execution.log().clone(),
        )
        .expect("projection")
    }

    #[tokio::test]
    async fn storage_adapter_reads_exact_redacted_projection() {
        let context_id = ContextId::new();
        let run_id = WorkflowRunId::from_uuid(Uuid::new_v4());
        let projection = projection(context_id, run_id);
        let repository = InMemoryWorkflowExecutionStatusRepository::default();
        repository
            .persist_workflow_execution_status(PersistWorkflowExecutionStatusV1::new(
                projection.clone(),
            ))
            .await
            .expect("persist");

        let adapter = StorageWorkflowExecutionStatusAdapter::new(repository);
        let resource = adapter
            .read_scoped(context_id, run_id)
            .await
            .expect("exact read")
            .expect("resource");
        let value = serde_json::to_value(&resource).expect("resource serializes");
        assert_eq!(resource.context_id(), context_id.as_uuid().to_string());
        assert_eq!(resource.run_id(), run_id.as_uuid().to_string());
        assert!(value.get("events").is_none());
        assert!(value.get("failure").is_none());
        assert_eq!(
            adapter
                .read_scoped(ContextId::new(), run_id)
                .await
                .expect("unknown exact Context scope is an empty read"),
            None
        );
    }

    #[test]
    fn serde_shape_is_exactly_redacted() {
        let context_id = ContextId::new();
        let run_id = WorkflowRunId::from_uuid(Uuid::new_v4());
        let value =
            serde_json::to_value(resource(context_id, run_id)).expect("resource serializes");
        assert_eq!(
            value,
            json!({
                "schema_version": LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1,
                "context_id": context_id.as_uuid().to_string(),
                "context_commit_id": value["context_commit_id"],
                "binding_id": value["binding_id"],
                "workflow_id": value["workflow_id"],
                "workflow_revision": 1,
                "run_id": run_id.as_uuid().to_string(),
                "replay_of": null,
                "run_state": "succeeded",
                "event_count": 3,
                "last_event_sequence": 3,
                "capability_snapshot_digest": "sha256:test",
                "node_status_counts": {
                    "pending": 0,
                    "running": 0,
                    "succeeded": 2,
                    "failed": 0,
                    "blocked": 0
                }
            })
        );
    }
}
