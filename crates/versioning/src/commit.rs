//! Context commit model.

use crate::{BranchName, ContextChange, VersioningError};
use chrono::{DateTime, Utc};
use contextlab_context_core::ContextId;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Stable identifier for a Context commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitId(Uuid);

impl CommitId {
    /// Creates a new random commit identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Reconstructs a commit identifier from its persisted UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for CommitId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CommitId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// A replayable commit over a Context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextCommit {
    id: CommitId,
    parent_ids: Vec<CommitId>,
    context_id: ContextId,
    branch: BranchName,
    message: String,
    changes: Vec<ContextChange>,
    authored_at: DateTime<Utc>,
}

impl ContextCommit {
    /// Creates a commit with validated message and explicit changes.
    pub fn new(
        context_id: ContextId,
        branch: BranchName,
        message: impl Into<String>,
        parent_ids: Vec<CommitId>,
        changes: Vec<ContextChange>,
        authored_at: DateTime<Utc>,
    ) -> Result<Self, VersioningError> {
        let message = message.into().trim().to_owned();
        if message.is_empty() {
            return Err(VersioningError::Empty {
                field: "commit.message",
            });
        }

        Ok(Self {
            id: CommitId::new(),
            parent_ids,
            context_id,
            branch,
            message,
            changes,
            authored_at,
        })
    }

    /// Reconstructs a commit with its persisted stable identifier.
    ///
    /// Storage adapters use this boundary to hand durable commit history back
    /// to the reusable replay domain without generating a new identity.
    pub fn from_persisted(
        id: CommitId,
        context_id: ContextId,
        branch: BranchName,
        message: impl Into<String>,
        parent_ids: Vec<CommitId>,
        changes: Vec<ContextChange>,
        authored_at: DateTime<Utc>,
    ) -> Result<Self, VersioningError> {
        let mut commit = Self::new(
            context_id,
            branch,
            message,
            parent_ids,
            changes,
            authored_at,
        )?;
        commit.id = id;
        Ok(commit)
    }

    /// Returns commit identifier.
    #[must_use]
    pub const fn id(&self) -> CommitId {
        self.id
    }

    /// Returns parent commit identifiers.
    #[must_use]
    pub fn parent_ids(&self) -> &[CommitId] {
        &self.parent_ids
    }

    /// Returns associated Context identifier.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns branch name.
    #[must_use]
    pub const fn branch(&self) -> &BranchName {
        &self.branch
    }

    /// Returns commit message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns changes in replay order.
    #[must_use]
    pub fn changes(&self) -> &[ContextChange] {
        &self.changes
    }

    /// Returns author timestamp.
    #[must_use]
    pub const fn authored_at(&self) -> DateTime<Utc> {
        self.authored_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContextChangeKind;

    #[test]
    fn creates_context_commit() {
        let commit = ContextCommit::new(
            ContextId::new(),
            BranchName::default(),
            "Create context",
            Vec::new(),
            vec![ContextChange::created_context("Initial context")],
            Utc::now(),
        )
        .expect("valid commit");

        assert_eq!(commit.branch().as_str(), "main");
        assert_eq!(
            commit.changes()[0].kind(),
            ContextChangeKind::CreatedContext
        );
    }

    #[test]
    fn reconstructs_a_commit_identifier_from_a_persisted_uuid() {
        let persisted = Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("uuid");

        assert_eq!(CommitId::from_uuid(persisted).as_uuid(), persisted);
    }

    #[test]
    fn reconstructs_a_persisted_commit_without_generating_a_new_identity() {
        let id = CommitId::from_uuid(Uuid::from_u128(7));
        let context_id = ContextId::from_uuid(Uuid::from_u128(8));
        let commit = ContextCommit::from_persisted(
            id,
            context_id,
            BranchName::default(),
            "Persisted commit",
            Vec::new(),
            vec![ContextChange::created_context("root")],
            Utc::now(),
        )
        .expect("valid persisted commit");

        assert_eq!(commit.id(), id);
        assert_eq!(commit.context_id(), context_id);
    }
}
