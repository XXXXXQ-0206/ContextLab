//! Append-only persistence contracts for sealed Workflow Context sources.

use crate::StorageRepositoryError;
use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_versioning::CommitId;
use contextlab_workflow::{WorkflowContextBinding, WorkflowId, WorkflowRevision};

/// Whether a Workflow source binding was newly persisted or replayed unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowContextBindingWriteDisposition {
    /// The immutable binding was persisted for the first time.
    Created,
    /// An identical binding already exists for this immutable Workflow revision.
    Replayed,
}

/// Result of an append-only Workflow Context source-binding write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowContextBindingWriteResult {
    binding: WorkflowContextBinding,
    disposition: WorkflowContextBindingWriteDisposition,
}

impl WorkflowContextBindingWriteResult {
    /// Returns the sealed source binding.
    #[must_use]
    pub const fn binding(&self) -> &WorkflowContextBinding {
        &self.binding
    }

    /// Returns whether the binding was persisted or replayed.
    #[must_use]
    pub const fn disposition(&self) -> WorkflowContextBindingWriteDisposition {
        self.disposition
    }

    pub(crate) const fn created(binding: WorkflowContextBinding) -> Self {
        Self {
            binding,
            disposition: WorkflowContextBindingWriteDisposition::Created,
        }
    }

    pub(crate) const fn replayed(binding: WorkflowContextBinding) -> Self {
        Self {
            binding,
            disposition: WorkflowContextBindingWriteDisposition::Replayed,
        }
    }
}

/// Private storage port for immutable Workflow definitions sourced from exact Context commits.
///
/// Implementations accept a binding only when its typed Context source already has a materialized
/// graph snapshot. Reads at a Context commit are scoped by both typed identifiers and sorted by
/// workflow identifier, revision, and binding identifier.
#[async_trait]
pub trait ContextWorkflowBindingRepository: Send + Sync {
    /// Persists an immutable Workflow source binding or returns an identical replay.
    async fn persist_workflow_context_binding(
        &self,
        binding: WorkflowContextBinding,
    ) -> Result<WorkflowContextBindingWriteResult, StorageRepositoryError>;

    /// Finds the immutable Context source for one exact Workflow definition revision.
    async fn get_workflow_context_binding(
        &self,
        workflow_id: WorkflowId,
        workflow_revision: WorkflowRevision,
    ) -> Result<Option<WorkflowContextBinding>, StorageRepositoryError>;

    /// Lists bindings attached to one exact materialized Context commit in canonical order.
    async fn list_workflow_context_bindings_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<Vec<WorkflowContextBinding>, StorageRepositoryError>;
}
