//! Immutable storage contract for redacted Workflow execution status projections.

use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_workflow::{
    WorkflowCapabilitySnapshot, WorkflowContextBinding, WorkflowExecution, WorkflowExecutionLogV1,
    WorkflowExecutionStatusProjectionError, WorkflowExecutionStatusProjectionSchemaVersion,
    WorkflowExecutionStatusProjectionV1, WorkflowRunId,
};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use uuid::Uuid;

/// An immutable projection write command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistWorkflowExecutionStatusV1 {
    projection: WorkflowExecutionStatusProjectionV1,
}

impl PersistWorkflowExecutionStatusV1 {
    /// Wraps a validated workflow execution projection for immutable persistence.
    #[must_use]
    pub const fn new(projection: WorkflowExecutionStatusProjectionV1) -> Self {
        Self { projection }
    }

    /// Returns the projection carried by this command.
    #[must_use]
    pub const fn projection(&self) -> &WorkflowExecutionStatusProjectionV1 {
        &self.projection
    }
}

/// Whether an immutable projection was newly stored or replayed unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowExecutionStatusWriteDisposition {
    /// The projection was stored for the first time.
    Created,
    /// The same immutable projection already existed.
    Replayed,
}

/// Result of an immutable workflow execution status write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowExecutionStatusWriteResult {
    projection: WorkflowExecutionStatusProjectionV1,
    disposition: WorkflowExecutionStatusWriteDisposition,
}

impl WorkflowExecutionStatusWriteResult {
    pub(crate) fn new(
        projection: WorkflowExecutionStatusProjectionV1,
        disposition: WorkflowExecutionStatusWriteDisposition,
    ) -> Self {
        Self {
            projection,
            disposition,
        }
    }

    /// Returns the stored or replayed projection.
    #[must_use]
    pub const fn projection(&self) -> &WorkflowExecutionStatusProjectionV1 {
        &self.projection
    }

    /// Returns the write disposition.
    #[must_use]
    pub const fn disposition(&self) -> WorkflowExecutionStatusWriteDisposition {
        self.disposition
    }
}

/// Fail-closed errors from workflow execution status persistence.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowExecutionStatusPersistenceError {
    /// The projection has an unusable exact scope or schema.
    #[error("workflow execution status projection has an invalid scope: {reason}")]
    InvalidScope {
        /// Stable validation reason.
        reason: &'static str,
    },
    /// An immutable run identifier was reused with different facts.
    #[error("workflow execution status conflicts for {context_id}/{run_id}")]
    Conflict {
        /// Exact Context identifier.
        context_id: String,
        /// Exact workflow run identifier.
        run_id: String,
    },
    /// The in-memory repository lock could not be acquired.
    #[error("workflow execution status repository is unavailable")]
    RepositoryUnavailable,
}

/// Errors raised while projecting and persisting a Workflow execution log.
#[derive(Debug, Error)]
pub enum WorkflowExecutionStatusServiceError {
    /// The domain replay validator rejected the execution log.
    #[error("workflow execution status projection failed: {0}")]
    Projection(#[from] WorkflowExecutionStatusProjectionError),
    /// The immutable repository rejected the resulting projection.
    #[error("workflow execution status persistence failed: {0}")]
    Persistence(#[from] WorkflowExecutionStatusPersistenceError),
}

/// Storage port for immutable exact-Context workflow execution status projections.
#[async_trait]
pub trait WorkflowExecutionStatusRepository: Send + Sync {
    /// Persists a projection or replays an identical immutable projection.
    async fn persist_workflow_execution_status(
        &self,
        command: PersistWorkflowExecutionStatusV1,
    ) -> Result<WorkflowExecutionStatusWriteResult, WorkflowExecutionStatusPersistenceError>;

    /// Reads one projection by exact Context and run identity.
    async fn read_workflow_execution_status(
        &self,
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> Result<Option<WorkflowExecutionStatusProjectionV1>, WorkflowExecutionStatusPersistenceError>;
}

/// Provider-free application service that projects validated logs before persistence.
pub struct WorkflowExecutionStatusService {
    repository: Arc<dyn WorkflowExecutionStatusRepository>,
}

impl WorkflowExecutionStatusService {
    /// Creates a producer over an immutable status repository.
    #[must_use]
    pub fn new(repository: impl WorkflowExecutionStatusRepository + 'static) -> Self {
        Self {
            repository: Arc::new(repository),
        }
    }

    /// Validates a root execution log and persists only its redacted projection.
    pub async fn persist_log(
        &self,
        binding: WorkflowContextBinding,
        snapshot: &WorkflowCapabilitySnapshot,
        log: WorkflowExecutionLogV1,
    ) -> Result<WorkflowExecutionStatusWriteResult, WorkflowExecutionStatusServiceError> {
        let projection = WorkflowExecutionStatusProjectionV1::from_log(binding, snapshot, log)?;
        self.repository
            .persist_workflow_execution_status(PersistWorkflowExecutionStatusV1::new(projection))
            .await
            .map_err(Into::into)
    }

    /// Validates a replay log against its terminal source and persists its projection.
    pub async fn persist_replay_log(
        &self,
        binding: WorkflowContextBinding,
        snapshot: &WorkflowCapabilitySnapshot,
        log: WorkflowExecutionLogV1,
        source: &WorkflowExecution,
    ) -> Result<WorkflowExecutionStatusWriteResult, WorkflowExecutionStatusServiceError> {
        let projection =
            WorkflowExecutionStatusProjectionV1::from_replay_log(binding, snapshot, log, source)?;
        self.repository
            .persist_workflow_execution_status(PersistWorkflowExecutionStatusV1::new(projection))
            .await
            .map_err(Into::into)
    }
}

/// Deterministic in-memory repository for local fixtures and provider-free execution receipts.
#[derive(Debug, Clone, Default)]
pub struct InMemoryWorkflowExecutionStatusRepository {
    projections: Arc<RwLock<BTreeMap<(Uuid, Uuid), WorkflowExecutionStatusProjectionV1>>>,
}

impl InMemoryWorkflowExecutionStatusRepository {
    fn validate_projection(
        projection: &WorkflowExecutionStatusProjectionV1,
    ) -> Result<(), WorkflowExecutionStatusPersistenceError> {
        if projection.schema_version() != WorkflowExecutionStatusProjectionSchemaVersion::V1 {
            return Err(WorkflowExecutionStatusPersistenceError::InvalidScope {
                reason: "unsupported projection schema",
            });
        }
        if projection.context_source().context_id().as_uuid().is_nil() {
            return Err(WorkflowExecutionStatusPersistenceError::InvalidScope {
                reason: "Context identifier is nil",
            });
        }
        if projection.context_source().commit_id().as_uuid().is_nil() {
            return Err(WorkflowExecutionStatusPersistenceError::InvalidScope {
                reason: "Context commit identifier is nil",
            });
        }
        if projection.run_id().as_uuid().is_nil() {
            return Err(WorkflowExecutionStatusPersistenceError::InvalidScope {
                reason: "workflow run identifier is nil",
            });
        }
        Ok(())
    }

    fn key(projection: &WorkflowExecutionStatusProjectionV1) -> (Uuid, Uuid) {
        (
            projection.context_source().context_id().as_uuid(),
            projection.run_id().as_uuid(),
        )
    }
}

#[async_trait]
impl WorkflowExecutionStatusRepository for InMemoryWorkflowExecutionStatusRepository {
    async fn persist_workflow_execution_status(
        &self,
        command: PersistWorkflowExecutionStatusV1,
    ) -> Result<WorkflowExecutionStatusWriteResult, WorkflowExecutionStatusPersistenceError> {
        Self::validate_projection(command.projection())?;
        let projection = command.projection().clone();
        let key = Self::key(&projection);
        let mut projections = self
            .projections
            .write()
            .map_err(|_| WorkflowExecutionStatusPersistenceError::RepositoryUnavailable)?;
        match projections.get(&key) {
            Some(existing) if existing == &projection => {
                Ok(WorkflowExecutionStatusWriteResult::new(
                    projection,
                    WorkflowExecutionStatusWriteDisposition::Replayed,
                ))
            }
            Some(_) => Err(WorkflowExecutionStatusPersistenceError::Conflict {
                context_id: key.0.to_string(),
                run_id: key.1.to_string(),
            }),
            None => {
                projections.insert(key, projection.clone());
                Ok(WorkflowExecutionStatusWriteResult::new(
                    projection,
                    WorkflowExecutionStatusWriteDisposition::Created,
                ))
            }
        }
    }

    async fn read_workflow_execution_status(
        &self,
        context_id: ContextId,
        run_id: WorkflowRunId,
    ) -> Result<Option<WorkflowExecutionStatusProjectionV1>, WorkflowExecutionStatusPersistenceError>
    {
        if context_id.as_uuid().is_nil() || run_id.as_uuid().is_nil() {
            return Err(WorkflowExecutionStatusPersistenceError::InvalidScope {
                reason: "read scope contains a nil identifier",
            });
        }
        self.projections
            .read()
            .map_err(|_| WorkflowExecutionStatusPersistenceError::RepositoryUnavailable)
            .map(|projections| {
                projections
                    .get(&(context_id.as_uuid(), run_id.as_uuid()))
                    .cloned()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_context_core::ContextId;
    use contextlab_versioning::CommitId;
    use contextlab_workflow::{
        ContextCommitSource, WorkflowCapabilitySnapshot, WorkflowContextBinding,
        WorkflowContextBindingId, WorkflowDefinition, WorkflowExecution,
        WorkflowExecutionStatusProjectionV1, WorkflowId, WorkflowNode, WorkflowNodeId,
        WorkflowRevision, WorkflowRunId,
    };
    use uuid::Uuid;

    fn uuid(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn projection(run_id: WorkflowRunId, workflow_id: u128) -> WorkflowExecutionStatusProjectionV1 {
        let revision = WorkflowRevision::new(1).expect("revision");
        let binding = WorkflowContextBinding::new(
            WorkflowContextBindingId::from_uuid(uuid(workflow_id + 100)),
            WorkflowDefinition::new(
                WorkflowId::from_uuid(uuid(workflow_id)),
                revision,
                vec![WorkflowNode::new(
                    WorkflowNodeId::from_uuid(uuid(12)),
                    revision,
                )],
                Vec::new(),
            )
            .expect("definition"),
            ContextCommitSource::new(
                ContextId::from_uuid(uuid(13)),
                CommitId::from_uuid(uuid(14)),
            ),
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
    async fn repository_persists_and_replays_exact_projection() {
        let repository = InMemoryWorkflowExecutionStatusRepository::default();
        let projection = projection(WorkflowRunId::from_uuid(uuid(20)), 11);
        let command = PersistWorkflowExecutionStatusV1::new(projection.clone());

        let created = repository
            .persist_workflow_execution_status(command.clone())
            .await
            .expect("create");
        assert_eq!(
            created.disposition(),
            WorkflowExecutionStatusWriteDisposition::Created
        );

        let replayed = repository
            .persist_workflow_execution_status(command)
            .await
            .expect("replay");
        assert_eq!(
            replayed.disposition(),
            WorkflowExecutionStatusWriteDisposition::Replayed
        );
        assert_eq!(replayed.projection(), &projection);

        let read = repository
            .read_workflow_execution_status(
                projection.context_source().context_id(),
                projection.run_id(),
            )
            .await
            .expect("read")
            .expect("stored projection");
        assert_eq!(read, projection);
    }

    #[tokio::test]
    async fn repository_rejects_conflicting_immutable_reuse_and_scope_misses() {
        let repository = InMemoryWorkflowExecutionStatusRepository::default();
        let first = projection(WorkflowRunId::from_uuid(uuid(30)), 11);
        repository
            .persist_workflow_execution_status(PersistWorkflowExecutionStatusV1::new(first.clone()))
            .await
            .expect("create");

        let conflicting = projection(WorkflowRunId::from_uuid(uuid(30)), 12);
        assert_ne!(conflicting, first);
        let error = repository
            .persist_workflow_execution_status(PersistWorkflowExecutionStatusV1::new(conflicting))
            .await
            .expect_err("immutable reuse must conflict");
        assert!(matches!(
            error,
            WorkflowExecutionStatusPersistenceError::Conflict { .. }
        ));

        assert_eq!(
            repository
                .read_workflow_execution_status(
                    ContextId::from_uuid(uuid(99)),
                    WorkflowRunId::from_uuid(uuid(30)),
                )
                .await
                .expect("missing scope is a safe empty read"),
            None
        );
    }

    #[tokio::test]
    async fn producer_projects_logs_before_persisting_and_replays_the_receipt() {
        let revision = WorkflowRevision::new(1).expect("revision");
        let context_id = ContextId::from_uuid(uuid(40));
        let binding = WorkflowContextBinding::new(
            WorkflowContextBindingId::from_uuid(uuid(41)),
            WorkflowDefinition::new(
                WorkflowId::from_uuid(uuid(42)),
                revision,
                vec![WorkflowNode::new(
                    WorkflowNodeId::from_uuid(uuid(43)),
                    revision,
                )],
                Vec::new(),
            )
            .expect("definition"),
            ContextCommitSource::new(context_id, CommitId::from_uuid(uuid(44))),
        );
        let run_id = WorkflowRunId::from_uuid(uuid(45));
        let execution = WorkflowExecution::start(
            binding.clone(),
            run_id,
            &WorkflowCapabilitySnapshot::default(),
        )
        .expect("execution");
        let repository = InMemoryWorkflowExecutionStatusRepository::default();
        let producer = WorkflowExecutionStatusService::new(repository.clone());

        let created = producer
            .persist_log(
                binding.clone(),
                &WorkflowCapabilitySnapshot::default(),
                execution.log().clone(),
            )
            .await
            .expect("projection is persisted");
        assert_eq!(
            created.disposition(),
            WorkflowExecutionStatusWriteDisposition::Created
        );
        assert_eq!(created.projection().event_count(), 1);
        assert_eq!(created.projection().replay_of(), None);

        let replayed = producer
            .persist_log(
                binding,
                &WorkflowCapabilitySnapshot::default(),
                execution.log().clone(),
            )
            .await
            .expect("identical projection replays");
        assert_eq!(
            replayed.disposition(),
            WorkflowExecutionStatusWriteDisposition::Replayed
        );
        assert_eq!(replayed.projection(), created.projection());
    }
}
