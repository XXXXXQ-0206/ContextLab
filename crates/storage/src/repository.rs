//! Repository contracts for storage-backed projections.

use crate::ContextGraphProjection;
use async_trait::async_trait;
use thiserror::Error;

/// Scope used when loading a graph projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphProjectionScope {
    /// Load the deterministic preview projection.
    Preview,
    /// Load graph data for a specific workspace.
    Workspace {
        /// Workspace identifier.
        workspace_id: String,
    },
}

/// Repository contract for loading Context Graph projections.
#[async_trait]
pub trait ContextGraphProjectionRepository: Send + Sync {
    /// Loads projection records for a graph scope.
    async fn load_context_graph_projection(
        &self,
        scope: GraphProjectionScope,
    ) -> Result<ContextGraphProjection, StorageRepositoryError>;
}

/// Repository-level storage errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StorageRepositoryError {
    /// The principal no longer has permission when a guarded write reaches its transaction.
    #[error("guarded write permission is forbidden")]
    GuardedWriteForbidden,
    /// Guarded write authorization state could not be read reliably inside its transaction.
    #[error("guarded write authorization state is unavailable")]
    GuardedWriteAuthorizationUnavailable,
    /// Requested scope is not available in this repository.
    #[error("graph projection scope is not available: {scope}")]
    ScopeUnavailable {
        /// Human-readable scope summary.
        scope: String,
    },
    /// Scope input could not be used by this repository.
    #[error("invalid graph projection scope {scope}: {reason}")]
    InvalidScope {
        /// Human-readable scope summary.
        scope: String,
        /// Validation failure reason.
        reason: String,
    },
    /// Stored component kind is not part of the v1 taxonomy.
    #[error("invalid stored component kind: {kind}")]
    InvalidStoredComponentKind {
        /// Raw component kind value from storage.
        kind: String,
    },
    /// An immutable benchmark definition identifier was reused with different content.
    #[error("benchmark {definition_kind} definition conflicts: {definition_id}")]
    BenchmarkDefinitionConflict {
        /// Dataset or suite definition kind.
        definition_kind: &'static str,
        /// Conflicting definition identifier.
        definition_id: String,
    },
    /// An immutable benchmark definition binding identifier was reused with different content.
    #[error("benchmark definition binding conflicts: {binding_id}")]
    BenchmarkDefinitionBindingConflict {
        /// Conflicting binding identifier.
        binding_id: String,
    },
    /// An evaluation run identifier reused a different evidence digest.
    #[error("benchmark evidence digest conflicts for run {run_id}")]
    BenchmarkEvidenceDigestConflict {
        /// Conflicting evaluation run identifier.
        run_id: String,
    },
    /// A Context commit identifier has already been persisted.
    #[error("context commit already exists: {context_id}/{commit_id}")]
    CommitAlreadyExists {
        /// Context identifier.
        context_id: String,
        /// Commit identifier.
        commit_id: String,
    },
    /// A guarded write used an idempotency key with a different request digest.
    #[error(
        "idempotency key was reused for a different request: {context_id}/{principal_id}/{idempotency_key}"
    )]
    IdempotencyKeyReused {
        /// Context identifier.
        context_id: String,
        /// Authenticated principal identifier.
        principal_id: String,
        /// Reused idempotency key.
        idempotency_key: String,
    },
    /// The branch head observed by a guarded writer is no longer current.
    #[error("branch head conflict: expected {expected:?}, actual {actual:?}")]
    BranchHeadConflict {
        /// Head observed by the writer, or no head for an initial commit.
        expected: Option<String>,
        /// Current branch head, or no head for an unborn branch.
        actual: Option<String>,
    },
    /// The commit parent list did not match the selected normal branch parent.
    #[error("guarded commit parent does not match the current branch head")]
    CommitParentMismatch {
        /// Parent required by the branch head, if any.
        expected_parent_id: Option<String>,
        /// Parent ids supplied by the commit command.
        actual_parent_ids: Vec<String>,
    },
    /// A component revision did not match the active component projection.
    #[error("component content revision conflict: {reason}")]
    ComponentContentRevisionConflict {
        /// Conflict detail suitable for private logs and tests.
        reason: String,
    },
    /// A component state replay cannot be reconstructed from durable history.
    #[error("component state replay conflict: {reason}")]
    ComponentStateReplayConflict {
        /// Conflict detail suitable for private logs and tests.
        reason: String,
    },
    /// An immutable Workflow definition revision was already bound to another Context source.
    #[error("workflow Context source binding conflicts: {workflow_id}@{workflow_revision}")]
    WorkflowContextBindingConflict {
        /// Stable Workflow definition identifier.
        workflow_id: String,
        /// Immutable Workflow definition revision.
        workflow_revision: u64,
    },
    /// The in-memory repository state could not be accessed safely.
    #[error("in-memory repository state is unavailable")]
    InMemoryStateUnavailable,
    /// Database operation failed.
    #[error("database operation failed: {message}")]
    Database {
        /// Redacted database error message.
        message: String,
    },
}
