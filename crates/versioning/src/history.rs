//! Provider-free, read-only contracts for branch heads and commit history.

use crate::{CommitGraph, CommitGraphNode, CommitId, ContextCommit, MergePlan, MergePlanError};
use contextlab_context_core::ContextId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// The read-only head of one named branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchHead {
    branch: crate::BranchName,
    head: Option<CommitId>,
}

impl BranchHead {
    /// Creates a branch head, including an unborn branch represented by `None`.
    #[must_use]
    pub fn new(branch: crate::BranchName, head: Option<CommitId>) -> Self {
        Self { branch, head }
    }

    /// Returns the branch name.
    #[must_use]
    pub const fn branch(&self) -> &crate::BranchName {
        &self.branch
    }

    /// Returns the current commit head, if the branch is born.
    #[must_use]
    pub const fn head(&self) -> Option<CommitId> {
        self.head
    }
}

/// Fail-closed validation and read errors for a commit history.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HistoryError {
    /// A branch was declared more than once.
    #[error("duplicate branch head: {branch}")]
    DuplicateBranch {
        /// Duplicated branch name.
        branch: String,
    },
    /// A born branch points at a commit absent from the supplied history.
    #[error("branch {branch} points at unknown head {commit_id}")]
    UnknownHead {
        /// Branch carrying the invalid head.
        branch: String,
        /// Missing commit identity.
        commit_id: String,
    },
    /// A requested commit is absent from the complete history.
    #[error("unknown commit: {commit_id}")]
    UnknownCommit {
        /// Missing commit identity.
        commit_id: CommitId,
    },
    /// A requested branch is absent from the history snapshot.
    #[error("unknown branch: {branch}")]
    UnknownBranch {
        /// Requested branch name.
        branch: String,
    },
    /// A branch has no commit to replay.
    #[error("branch {branch} is unborn")]
    UnbornBranch {
        /// Unborn branch name.
        branch: String,
    },
    /// A replay path contains a merge commit and is not linear.
    #[error("branch history contains non-linear ancestry at commit {commit_id}")]
    NonLinearAncestry {
        /// Merge commit encountered while following the branch head.
        commit_id: CommitId,
    },
    /// The target is not a normal first-parent descendant of the source.
    #[error("commit {target_commit} is not a normal first-parent descendant of {source_commit}")]
    InvalidNormalReplayRange {
        /// Requested baseline commit.
        source_commit: CommitId,
        /// Requested revised commit.
        target_commit: CommitId,
    },
    /// The underlying validated graph rejected the supplied commit set.
    #[error("invalid commit history graph: {0}")]
    Graph(#[from] crate::CommitGraphValidationError),
    /// The branch tips could not be classified using ancestry alone.
    #[error("cannot resolve merge plan: {0}")]
    MergePlan(#[from] MergePlanError),
}

/// A complete immutable commit set plus explicit branch heads.
///
/// This type is intentionally a read model. It does not create commits, move
/// heads, access providers, or transport data to another process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitHistory {
    context_id: ContextId,
    commits: BTreeMap<String, ContextCommit>,
    heads: BTreeMap<crate::BranchName, Option<CommitId>>,
    graph: CommitGraph,
}

impl CommitHistory {
    /// Validates a complete commit set and its explicit branch heads.
    pub fn try_from_parts(
        context_id: ContextId,
        commits: impl IntoIterator<Item = ContextCommit>,
        heads: impl IntoIterator<Item = BranchHead>,
    ) -> Result<Self, HistoryError> {
        let mut indexed = BTreeMap::new();
        let mut graph_nodes = Vec::new();
        for commit in commits {
            let key = commit.id().to_string();
            if commit.context_id() != context_id {
                return Err(HistoryError::Graph(
                    crate::CommitGraphValidationError::CrossContextNode {
                        commit_id: key,
                        expected_context: context_id,
                        actual_context: commit.context_id(),
                    },
                ));
            }
            graph_nodes.push(CommitGraphNode::new(
                commit.id(),
                commit.context_id(),
                commit.parent_ids().to_vec(),
            ));
            if indexed.insert(key, commit).is_some() {
                return Err(HistoryError::Graph(
                    crate::CommitGraphValidationError::DuplicateCommit {
                        commit_id: graph_nodes
                            .last()
                            .expect("node was pushed")
                            .id()
                            .to_string(),
                    },
                ));
            }
        }

        let graph = CommitGraph::try_from_nodes(graph_nodes)?;
        let mut indexed_heads = BTreeMap::new();
        for branch_head in heads {
            let branch = branch_head.branch().clone();
            if indexed_heads
                .insert(branch.clone(), branch_head.head())
                .is_some()
            {
                return Err(HistoryError::DuplicateBranch {
                    branch: branch.to_string(),
                });
            }
            if let Some(head) = branch_head.head() {
                let commit =
                    indexed
                        .get(&head.to_string())
                        .ok_or_else(|| HistoryError::UnknownHead {
                            branch: branch.to_string(),
                            commit_id: head.to_string(),
                        })?;
                if commit.context_id() != context_id {
                    return Err(HistoryError::Graph(
                        crate::CommitGraphValidationError::CrossContextNode {
                            commit_id: head.to_string(),
                            expected_context: context_id,
                            actual_context: commit.context_id(),
                        },
                    ));
                }
            }
        }

        Ok(Self {
            context_id,
            commits: indexed,
            heads: indexed_heads,
            graph,
        })
    }

    /// Returns the Context scope of this history.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns branch names in deterministic lexical order.
    pub fn branches(&self) -> impl Iterator<Item = &crate::BranchName> {
        self.heads.keys()
    }

    /// Returns the exact read head for a known branch.
    #[must_use]
    pub fn head(&self, branch: &crate::BranchName) -> Option<CommitId> {
        self.heads.get(branch).copied().flatten()
    }

    /// Returns the immutable commit for an exact stable UUID.
    #[must_use]
    pub fn commit(&self, id: CommitId) -> Option<&ContextCommit> {
        self.commits.get(&id.to_string())
    }

    /// Returns one branch's commits from root to head in replay order.
    pub fn commits_for_branch(
        &self,
        branch: &crate::BranchName,
    ) -> Result<Vec<&ContextCommit>, HistoryError> {
        let head = self
            .heads
            .get(branch)
            .ok_or_else(|| HistoryError::UnknownBranch {
                branch: branch.to_string(),
            })?;
        let Some(mut current) = *head else {
            return Ok(Vec::new());
        };

        let mut ordered = Vec::new();
        loop {
            let commit = self
                .commit(current)
                .expect("validated branch heads and graph parents exist");
            if commit.parent_ids().len() > 1 {
                return Err(HistoryError::NonLinearAncestry { commit_id: current });
            }
            ordered.push(commit);
            let Some(parent) = commit.parent_ids().first().copied() else {
                break;
            };
            current = parent;
        }
        ordered.reverse();
        Ok(ordered)
    }

    /// Returns the inclusive normal first-parent replay path from `source` to `target`.
    ///
    /// The target must be reachable by repeatedly following the first parent
    /// from target back to source. Any merge commit on that path is rejected;
    /// merge ancestry belongs to the dedicated merge-review contract.
    pub fn normal_first_parent_path(
        &self,
        source: CommitId,
        target: CommitId,
    ) -> Result<Vec<CommitId>, HistoryError> {
        self.commit(source)
            .ok_or(HistoryError::UnknownCommit { commit_id: source })?;
        self.commit(target)
            .ok_or(HistoryError::UnknownCommit { commit_id: target })?;

        let mut current = target;
        let mut path = Vec::new();
        loop {
            let commit = self
                .commit(current)
                .expect("source and target membership were validated");
            if commit.parent_ids().len() > 1 {
                return Err(HistoryError::NonLinearAncestry { commit_id: current });
            }
            path.push(current);
            if current == source {
                path.reverse();
                return Ok(path);
            }
            let Some(parent) = commit.parent_ids().first().copied() else {
                return Err(HistoryError::InvalidNormalReplayRange {
                    source_commit: source,
                    target_commit: target,
                });
            };
            current = parent;
        }
    }

    /// Resolves the ancestry-only merge plan for two named born branches.
    pub fn merge_plan(
        &self,
        left: &crate::BranchName,
        right: &crate::BranchName,
    ) -> Result<MergePlan, HistoryError> {
        let left_head = self
            .heads
            .get(left)
            .ok_or_else(|| HistoryError::UnknownBranch {
                branch: left.to_string(),
            })?
            .ok_or_else(|| HistoryError::UnbornBranch {
                branch: left.to_string(),
            })?;
        let right_head = self
            .heads
            .get(right)
            .ok_or_else(|| HistoryError::UnknownBranch {
                branch: right.to_string(),
            })?
            .ok_or_else(|| HistoryError::UnbornBranch {
                branch: right.to_string(),
            })?;
        Ok(MergePlan::resolve(&self.graph, left_head, right_head)?)
    }
}
