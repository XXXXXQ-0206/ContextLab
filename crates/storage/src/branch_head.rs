//! Typed read contracts for durable Context branch heads.

use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_versioning::{BranchName, CommitId};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// The durable head and revision of one Context branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextBranchHead {
    context_id: ContextId,
    branch: BranchName,
    head_commit_id: Option<CommitId>,
    revision: u64,
}

impl ContextBranchHead {
    /// Creates a validated branch-head read model.
    #[must_use]
    pub const fn new(
        context_id: ContextId,
        branch: BranchName,
        head_commit_id: Option<CommitId>,
        revision: u64,
    ) -> Self {
        Self {
            context_id,
            branch,
            head_commit_id,
            revision,
        }
    }

    /// Returns the Context owning this branch.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the validated branch name.
    #[must_use]
    pub fn branch(&self) -> &BranchName {
        &self.branch
    }

    /// Returns the current head, or `None` for an unborn branch.
    #[must_use]
    pub const fn head_commit_id(&self) -> Option<CommitId> {
        self.head_commit_id
    }

    /// Returns the monotonic branch revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn from_stored(
        context_id: ContextId,
        branch: String,
        head_commit_id: Option<Uuid>,
        revision: i64,
        head_belongs_to_context: bool,
    ) -> Result<Self, ContextBranchRepositoryError> {
        let branch = BranchName::new(branch.clone()).map_err(|error| {
            ContextBranchRepositoryError::InvalidStoredBranchName {
                context_id,
                branch,
                reason: error.to_string(),
            }
        })?;
        let revision = u64::try_from(revision).map_err(|_| {
            ContextBranchRepositoryError::InvalidStoredRevision {
                context_id,
                branch: branch.clone(),
                revision,
            }
        })?;
        let head_commit_id = head_commit_id.map(CommitId::from_uuid);
        if let Some(head_commit_id) = head_commit_id {
            if !head_belongs_to_context {
                return Err(ContextBranchRepositoryError::BranchHeadIntegrityViolation {
                    context_id,
                    branch,
                    head_commit_id,
                });
            }
        }
        Ok(Self::new(context_id, branch, head_commit_id, revision))
    }
}

/// Errors returned by the private branch-head read contract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContextBranchRepositoryError {
    /// The requested Context does not exist in the durable repository.
    #[error("Context does not exist: {context_id}")]
    UnknownContext {
        /// Missing Context identity.
        context_id: ContextId,
    },
    /// The Context exists, but the requested branch does not.
    #[error("branch does not exist for Context {context_id}: {branch}")]
    UnknownBranch {
        /// Context identity that owns the missing branch.
        context_id: ContextId,
        /// Missing branch identity.
        branch: BranchName,
    },
    /// A stored branch name could not be rehydrated as a typed name.
    #[error("stored branch name is invalid for Context {context_id}: {branch} ({reason})")]
    InvalidStoredBranchName {
        /// Context identity owning the invalid row.
        context_id: ContextId,
        /// Raw branch value read from storage.
        branch: String,
        /// Validation failure from the versioning crate.
        reason: String,
    },
    /// A stored signed database revision could not be converted to a revision.
    #[error("stored branch revision is invalid for Context {context_id}/{branch}: {revision}")]
    InvalidStoredRevision {
        /// Context identity owning the invalid row.
        context_id: ContextId,
        /// Typed branch identity.
        branch: BranchName,
        /// Raw signed revision from storage.
        revision: i64,
    },
    /// An in-memory revision exceeded the database-compatible signed range.
    #[error("branch revision overflow for Context {context_id}/{branch}: {revision}")]
    RevisionOverflow {
        /// Context identity owning the overflowing row.
        context_id: ContextId,
        /// Typed branch identity.
        branch: BranchName,
        /// Raw in-memory revision.
        revision: u64,
    },
    /// A stored head points outside its Context ownership boundary.
    #[error("branch head violates Context ownership for {context_id}/{branch}: {head_commit_id}")]
    BranchHeadIntegrityViolation {
        /// Context identity owning the branch.
        context_id: ContextId,
        /// Typed branch identity.
        branch: BranchName,
        /// Head commit that points outside the Context.
        head_commit_id: CommitId,
    },
    /// In-memory state could not be read safely.
    #[error("in-memory branch-head state is unavailable")]
    InMemoryStateUnavailable,
    /// A database failure was redacted before crossing the repository boundary.
    #[error("branch-head database operation failed: {message}")]
    Database {
        /// Redacted database failure detail.
        message: String,
    },
}

/// Repository port for exact Context branch-head discovery.
#[async_trait]
pub trait ContextBranchRepository: Send + Sync {
    /// Lists all durable branches for one Context in stable branch-name order.
    async fn list_context_branch_heads(
        &self,
        context_id: ContextId,
    ) -> Result<Vec<ContextBranchHead>, ContextBranchRepositoryError>;

    /// Reads one exact durable branch head.
    async fn get_context_branch_head(
        &self,
        context_id: ContextId,
        branch: BranchName,
    ) -> Result<ContextBranchHead, ContextBranchRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_typed_unborn_head_and_revision() {
        let context_id = ContextId::new();
        let branch = BranchName::new("main").expect("branch");
        let head = ContextBranchHead::new(context_id, branch.clone(), None, 0);

        assert_eq!(head.context_id(), context_id);
        assert_eq!(head.branch(), &branch);
        assert_eq!(head.head_commit_id(), None);
        assert_eq!(head.revision(), 0);
    }

    #[test]
    fn rehydrates_a_typed_head_and_revision_from_storage_values() {
        let context_id = ContextId::new();
        let commit_id = CommitId::new();
        let head = ContextBranchHead::from_stored(
            context_id,
            "feature/read-only".to_owned(),
            Some(commit_id.as_uuid()),
            7,
            true,
        )
        .expect("valid stored branch head");

        assert_eq!(head.context_id(), context_id);
        assert_eq!(head.branch().as_str(), "feature/read-only");
        assert_eq!(head.head_commit_id(), Some(commit_id));
        assert_eq!(head.revision(), 7);
    }

    #[test]
    fn rejects_a_head_that_is_outside_the_context_scope() {
        let context_id = ContextId::new();
        let commit_id = CommitId::new();
        let error = ContextBranchHead::from_stored(
            context_id,
            "main".to_owned(),
            Some(commit_id.as_uuid()),
            1,
            false,
        )
        .expect_err("cross-Context heads must fail closed");

        assert!(matches!(
            error,
            ContextBranchRepositoryError::BranchHeadIntegrityViolation {
                context_id: actual_context,
                head_commit_id: actual_commit,
                ..
            } if actual_context == context_id && actual_commit == commit_id
        ));
    }

    #[test]
    fn rejects_negative_stored_revisions() {
        let context_id = ContextId::new();
        let error = ContextBranchHead::from_stored(context_id, "main".to_owned(), None, -1, true)
            .expect_err("negative database revisions must fail closed");

        assert!(matches!(
            error,
            ContextBranchRepositoryError::InvalidStoredRevision { revision: -1, .. }
        ));
    }
}
