//! Branch naming and validation.

use crate::CommitId;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors produced by versioning constructors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VersioningError {
    /// A branch or commit message was empty.
    #[error("{field} must not be empty")]
    Empty {
        /// Name of the invalid field.
        field: &'static str,
    },

    /// A branch name contained unsupported characters.
    #[error("branch name must contain only letters, numbers, '/', '-', '_', or '.'")]
    InvalidBranchName,
}

/// A validated branch name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BranchName(String);

impl BranchName {
    /// Creates a branch name.
    pub fn new(value: impl Into<String>) -> Result<Self, VersioningError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(VersioningError::Empty {
                field: "branch.name",
            });
        }
        if !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '/' | '-' | '_' | '.')
        }) {
            return Err(VersioningError::InvalidBranchName);
        }
        Ok(Self(value))
    }

    /// Returns the branch name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BranchName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Default for BranchName {
    fn default() -> Self {
        Self("main".to_owned())
    }
}

/// The branch head a client observed before preparing a normal commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectedBranchHead {
    /// The client observed an unborn branch with no commit head.
    Unborn,
    /// The client observed this specific commit as the branch head.
    Commit(CommitId),
}

/// A conflict between a client-observed branch head and the current branch head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("branch head conflict: expected {expected:?}, actual {actual:?}")]
pub struct BranchHeadConflict {
    /// Head observed by the client, or no head for an initial commit.
    pub expected: Option<CommitId>,
    /// Current branch head, or no head for an unborn branch.
    pub actual: Option<CommitId>,
}

/// Selects the single normal parent for an optimistic branch-head update.
///
/// A matching observed head fast-forwards from that commit. An unborn branch
/// accepts only an initial commit and therefore has no parent.
pub fn normal_commit_parent(
    expected: ExpectedBranchHead,
    actual: Option<CommitId>,
) -> Result<Option<CommitId>, BranchHeadConflict> {
    let expected = match expected {
        ExpectedBranchHead::Unborn => None,
        ExpectedBranchHead::Commit(commit_id) => Some(commit_id),
    };

    if expected == actual {
        return Ok(actual);
    }

    Err(BranchHeadConflict { expected, actual })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CommitId, ExpectedBranchHead, normal_commit_parent};

    #[test]
    fn accepts_path_like_branch_names() {
        let branch = BranchName::new("feature/context-diff.v1").expect("valid branch");

        assert_eq!(branch.as_str(), "feature/context-diff.v1");
    }

    #[test]
    fn rejects_branch_names_with_spaces() {
        let error = BranchName::new("feature branch").expect_err("space should fail");

        assert_eq!(error, VersioningError::InvalidBranchName);
    }

    #[test]
    fn selects_no_parent_for_an_unborn_branch() {
        let parent = normal_commit_parent(ExpectedBranchHead::Unborn, None)
            .expect("an unborn branch accepts an initial commit");

        assert_eq!(parent, None);
    }

    #[test]
    fn selects_the_current_head_as_the_normal_commit_parent() {
        let current_head = CommitId::new();
        let parent =
            normal_commit_parent(ExpectedBranchHead::Commit(current_head), Some(current_head))
                .expect("matching head accepts a fast-forward commit");

        assert_eq!(parent, Some(current_head));
    }

    #[test]
    fn rejects_a_stale_expected_branch_head() {
        let expected_head = CommitId::new();
        let current_head = CommitId::new();
        let error = normal_commit_parent(
            ExpectedBranchHead::Commit(expected_head),
            Some(current_head),
        )
        .expect_err("a stale head must not create a second branch tip");

        assert_eq!(
            error,
            BranchHeadConflict {
                expected: Some(expected_head),
                actual: Some(current_head)
            }
        );
    }

    #[test]
    fn rejects_an_initial_commit_when_the_branch_already_has_a_head() {
        let current_head = CommitId::new();
        let error = normal_commit_parent(ExpectedBranchHead::Unborn, Some(current_head))
            .expect_err("an existing branch cannot accept an initial commit");

        assert_eq!(
            error,
            BranchHeadConflict {
                expected: None,
                actual: Some(current_head)
            }
        );
    }
}
