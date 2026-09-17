//! Read-only review composition that binds graph snapshots to validated commit history.

use crate::context_graph_diff_review::{validate_scope_pair, validate_single_scope};
use crate::{
    CommitGraphSnapshot, CommitGraphSnapshotScope, ContextCommitHistoryRepository,
    ContextLifecycleReadRepository, PersistedContextGraphDiffReviewError,
    PersistedContextGraphDiffReviewProjection, PersistedContextGraphDiffReviewService,
    PersistedContextGraphDiffReviewSide, StorageRepositoryError,
};
use contextlab_versioning::{BranchName, CommitHistory, CommitId, HistoryError};
use thiserror::Error;

/// A backend-owned, exact-scope witness for a version-backed Context Graph review.
///
/// The aggregate deliberately retains the complete history, including branch
/// heads, alongside both immutable graph snapshots. Concrete repositories must
/// construct it inside one read guard or transaction; consumers must not rebuild
/// it from the narrower repository ports.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextGraphReviewWitness {
    history: CommitHistory,
    source: CommitGraphSnapshot,
    target: CommitGraphSnapshot,
    intermediate_snapshots: Vec<CommitGraphSnapshot>,
}

impl ContextGraphReviewWitness {
    /// Validates and creates one exact project/Context/commit witness.
    pub fn try_from_parts(
        history: CommitHistory,
        source: CommitGraphSnapshot,
        target: CommitGraphSnapshot,
    ) -> Result<Self, ContextGraphReviewWitnessError> {
        Self::try_from_parts_with_intermediate_snapshots(history, source, target, Vec::new())
    }

    /// Validates an exact review witness and every immutable snapshot on its
    /// normal first-parent path.
    pub fn try_from_parts_with_intermediate_snapshots(
        history: CommitHistory,
        source: CommitGraphSnapshot,
        target: CommitGraphSnapshot,
        intermediate_snapshots: Vec<CommitGraphSnapshot>,
    ) -> Result<Self, ContextGraphReviewWitnessError> {
        validate_witness_scope(source.scope(), target.scope())?;
        if history.context_id() != source.context_id()
            || history.context_id() != target.context_id()
        {
            return Err(ContextGraphReviewWitnessError::HistoryContextMismatch {
                expected: source.context_id(),
                actual: history.context_id(),
            });
        }
        for (side, snapshot) in [
            (PersistedContextGraphDiffReviewSide::Source, &source),
            (PersistedContextGraphDiffReviewSide::Target, &target),
        ] {
            if history.commit(snapshot.commit_id()).is_none() {
                return Err(ContextGraphReviewWitnessError::CommitMissing {
                    side,
                    commit_id: snapshot.commit_id(),
                });
            }
            if snapshot.schema_version() != crate::COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1 {
                return Err(ContextGraphReviewWitnessError::SnapshotSchemaMismatch {
                    side,
                    scope: snapshot.scope(),
                    schema_version: snapshot.schema_version(),
                });
            }
        }
        let path = history
            .normal_first_parent_path(source.commit_id(), target.commit_id())
            .map_err(|reason| ContextGraphReviewWitnessError::ReplayPathInvalid {
                source_commit: source.commit_id(),
                target_commit: target.commit_id(),
                reason,
            })?;

        let expected_intermediate = path.iter().skip(1).take(path.len().saturating_sub(2));
        for (expected_commit_id, snapshot) in expected_intermediate.zip(&intermediate_snapshots) {
            let expected_scope = CommitGraphSnapshotScope::new(
                source.project_id(),
                source.context_id(),
                *expected_commit_id,
            );
            if snapshot.scope() != expected_scope {
                return Err(
                    ContextGraphReviewWitnessError::IntermediateSnapshotScopeMismatch {
                        expected: expected_scope,
                        actual: snapshot.scope(),
                    },
                );
            }
            if snapshot.schema_version() != crate::COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1 {
                return Err(
                    ContextGraphReviewWitnessError::IntermediateSnapshotSchemaMismatch {
                        scope: snapshot.scope(),
                        schema_version: snapshot.schema_version(),
                    },
                );
            }
        }
        let expected_count = path.len().saturating_sub(2);
        if intermediate_snapshots.len() < expected_count {
            return Err(
                ContextGraphReviewWitnessError::IntermediateSnapshotMissing {
                    commit_id: path[1 + intermediate_snapshots.len()],
                },
            );
        }
        if intermediate_snapshots.len() > expected_count {
            return Err(
                ContextGraphReviewWitnessError::IntermediateSnapshotUnexpected {
                    commit_id: intermediate_snapshots[expected_count].commit_id(),
                },
            );
        }

        Ok(Self {
            history,
            source,
            target,
            intermediate_snapshots,
        })
    }

    /// Returns the complete validated commit history and branch heads.
    #[must_use]
    pub const fn history(&self) -> &CommitHistory {
        &self.history
    }

    /// Returns the exact baseline snapshot.
    #[must_use]
    pub const fn source(&self) -> &CommitGraphSnapshot {
        &self.source
    }

    /// Returns the exact revised snapshot.
    #[must_use]
    pub const fn target(&self) -> &CommitGraphSnapshot {
        &self.target
    }

    /// Returns ordered immutable snapshots between source and target.
    #[must_use]
    pub fn intermediate_snapshots(&self) -> &[CommitGraphSnapshot] {
        &self.intermediate_snapshots
    }
}

/// An immutable witness whose revised snapshot is bound to one server-owned branch head.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextGraphBranchHeadReviewWitness {
    branch: BranchName,
    target_commit_id: CommitId,
    witness: ContextGraphReviewWitness,
}

impl ContextGraphBranchHeadReviewWitness {
    /// Validates that the witness target is exactly the selected branch head.
    pub fn try_from_parts(
        branch: BranchName,
        witness: ContextGraphReviewWitness,
    ) -> Result<Self, ContextGraphBranchHeadReviewWitnessRepositoryError> {
        if !witness
            .history()
            .branches()
            .any(|candidate| candidate == &branch)
        {
            return Err(
                ContextGraphBranchHeadReviewWitnessRepositoryError::BranchUnknown { branch },
            );
        }
        let expected = witness.history().head(&branch).ok_or_else(|| {
            ContextGraphBranchHeadReviewWitnessRepositoryError::BranchUnborn {
                branch: branch.clone(),
            }
        })?;
        let actual = witness.target().commit_id();
        if actual != expected {
            return Err(
                ContextGraphBranchHeadReviewWitnessRepositoryError::TargetNotBranchHead {
                    branch,
                    expected,
                    actual,
                },
            );
        }
        Ok(Self {
            branch,
            target_commit_id: actual,
            witness,
        })
    }

    /// Returns the server-selected branch.
    #[must_use]
    pub const fn branch(&self) -> &BranchName {
        &self.branch
    }

    /// Returns the exact commit selected as the branch head.
    #[must_use]
    pub const fn target_commit_id(&self) -> CommitId {
        self.target_commit_id
    }

    /// Returns the complete history-plus-snapshot witness.
    #[must_use]
    pub const fn witness(&self) -> &ContextGraphReviewWitness {
        &self.witness
    }
}

/// Fail-closed errors for branch-head witness selection and validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContextGraphBranchHeadReviewWitnessRepositoryError {
    /// The backend read failed before a witness could be constructed.
    #[error("Context Graph branch-head witness storage read failed: {source}")]
    Storage {
        /// Redacted repository failure.
        #[from]
        source: StorageRepositoryError,
    },
    /// The selected branch is absent from the complete history.
    #[error("Context Graph branch-head witness has no branch {branch}")]
    BranchUnknown {
        /// Requested branch name.
        branch: BranchName,
    },
    /// The selected branch exists but has no commit head.
    #[error("Context Graph branch-head witness branch {branch} is unborn")]
    BranchUnborn {
        /// Requested branch name.
        branch: BranchName,
    },
    /// The snapshot target did not equal the selected branch head.
    #[error(
        "Context Graph branch-head witness target does not match branch {branch}: expected {expected}, got {actual}"
    )]
    TargetNotBranchHead {
        /// Requested branch name.
        branch: BranchName,
        /// Head selected from complete history.
        expected: CommitId,
        /// Target present in the snapshot witness.
        actual: CommitId,
    },
    /// The exact history-plus-snapshot witness failed validation.
    #[error("Context Graph branch-head witness is invalid: {source}")]
    Witness {
        /// Witness validation failure.
        #[source]
        source: ContextGraphReviewWitnessError,
    },
}

/// Fail-closed errors for a backend-owned graph review witness.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContextGraphReviewWitnessError {
    /// One requested scope contains a nil identifier.
    #[error("Context Graph review witness scope is invalid")]
    InvalidScope {
        /// Invalid exact scope.
        scope: CommitGraphSnapshotScope,
    },
    /// The source and target do not share one project and Context.
    #[error("Context Graph review witness scopes do not match")]
    MismatchedContextScope {
        /// Baseline scope.
        source_scope: CommitGraphSnapshotScope,
        /// Revised scope.
        target_scope: CommitGraphSnapshotScope,
    },
    /// A review cannot compare one commit with itself.
    #[error("Context Graph review witness repeats exact scope {scope}")]
    IdenticalVersionScope {
        /// Repeated scope.
        scope: CommitGraphSnapshotScope,
    },
    /// The history belongs to another Context.
    #[error("Context Graph review witness history belongs to another Context")]
    HistoryContextMismatch {
        /// Requested Context.
        expected: contextlab_context_core::ContextId,
        /// History Context.
        actual: contextlab_context_core::ContextId,
    },
    /// A snapshot commit is absent from the complete history.
    #[error("Context Graph review witness is missing {side:?} commit {commit_id}")]
    CommitMissing {
        /// Missing side.
        side: PersistedContextGraphDiffReviewSide,
        /// Missing commit.
        commit_id: CommitId,
    },
    /// A snapshot uses an unsupported schema.
    #[error("Context Graph review witness returned an unsupported {side:?} snapshot schema")]
    SnapshotSchemaMismatch {
        /// Side with the unsupported schema.
        side: PersistedContextGraphDiffReviewSide,
        /// Snapshot scope.
        scope: CommitGraphSnapshotScope,
        /// Unsupported schema version.
        schema_version: u16,
    },
    /// An intermediate normal-replay commit has no materialized snapshot.
    #[error("Context Graph review witness is missing intermediate commit snapshot {commit_id}")]
    IntermediateSnapshotMissing {
        /// Missing intermediate commit.
        commit_id: CommitId,
    },
    /// An intermediate snapshot does not match the normal-replay commit scope.
    #[error("Context Graph review witness intermediate snapshot scope does not match")]
    IntermediateSnapshotScopeMismatch {
        /// Expected path scope.
        expected: CommitGraphSnapshotScope,
        /// Returned snapshot scope.
        actual: CommitGraphSnapshotScope,
    },
    /// An intermediate snapshot uses an unsupported schema.
    #[error("Context Graph review witness returned an unsupported intermediate snapshot schema")]
    IntermediateSnapshotSchemaMismatch {
        /// Snapshot scope.
        scope: CommitGraphSnapshotScope,
        /// Unsupported schema version.
        schema_version: u16,
    },
    /// A repository returned a snapshot not belonging to the requested path.
    #[error(
        "Context Graph review witness returned an unexpected intermediate snapshot {commit_id}"
    )]
    IntermediateSnapshotUnexpected {
        /// Unexpected snapshot commit.
        commit_id: CommitId,
    },
    /// The exact pair is not a normal first-parent replay range.
    #[error(
        "Context Graph review witness cannot replay from {source_commit} to {target_commit}: {reason}"
    )]
    ReplayPathInvalid {
        /// Requested baseline commit.
        source_commit: CommitId,
        /// Requested revised commit.
        target_commit: CommitId,
        /// Fail-closed versioning contract result.
        #[source]
        reason: HistoryError,
    },
}

/// Backend contract for one atomic history-plus-snapshot review read.
#[async_trait::async_trait]
pub trait ContextGraphReviewWitnessRepository: Send + Sync {
    /// Reads and validates both snapshots with the complete Context history.
    async fn read_context_graph_review_witness(
        &self,
        source_scope: CommitGraphSnapshotScope,
        target_scope: CommitGraphSnapshotScope,
    ) -> Result<ContextGraphReviewWitness, StorageRepositoryError>;
}

/// Repository contract for an atomic server-owned branch-head graph witness.
#[async_trait::async_trait]
pub trait ContextGraphBranchHeadReviewWitnessRepository: Send + Sync {
    /// Selects a branch head and reads its complete history and graph snapshots atomically.
    async fn read_context_graph_branch_head_review_witness(
        &self,
        source_scope: CommitGraphSnapshotScope,
        branch: BranchName,
    ) -> Result<
        ContextGraphBranchHeadReviewWitness,
        ContextGraphBranchHeadReviewWitnessRepositoryError,
    >;
}

pub(crate) fn select_branch_head(
    history: &CommitHistory,
    branch: &BranchName,
) -> Result<CommitId, ContextGraphBranchHeadReviewWitnessRepositoryError> {
    if !history.branches().any(|candidate| candidate == branch) {
        return Err(
            ContextGraphBranchHeadReviewWitnessRepositoryError::BranchUnknown {
                branch: branch.clone(),
            },
        );
    }
    history.head(branch).ok_or_else(|| {
        ContextGraphBranchHeadReviewWitnessRepositoryError::BranchUnborn {
            branch: branch.clone(),
        }
    })
}

pub(crate) fn normal_first_parent_intermediate_commit_ids(
    history: &CommitHistory,
    source: CommitId,
    target: CommitId,
) -> Result<Vec<CommitId>, HistoryError> {
    let path = history.normal_first_parent_path(source, target)?;
    let intermediate_count = path.len().saturating_sub(2);
    Ok(path.into_iter().skip(1).take(intermediate_count).collect())
}

fn validate_witness_scope(
    source: CommitGraphSnapshotScope,
    target: CommitGraphSnapshotScope,
) -> Result<(), ContextGraphReviewWitnessError> {
    for scope in [source, target] {
        if scope.project_id().as_uuid().is_nil()
            || scope.context_id().as_uuid().is_nil()
            || scope.commit_id().as_uuid().is_nil()
        {
            return Err(ContextGraphReviewWitnessError::InvalidScope { scope });
        }
    }
    if source.project_id() != target.project_id() || source.context_id() != target.context_id() {
        return Err(ContextGraphReviewWitnessError::MismatchedContextScope {
            source_scope: source,
            target_scope: target,
        });
    }
    if source.commit_id() == target.commit_id() {
        return Err(ContextGraphReviewWitnessError::IdenticalVersionScope { scope: source });
    }
    Ok(())
}

/// Projection for a graph review with its complete immutable Context history.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistedContextGraphHistoryReviewProjection {
    history: CommitHistory,
    graph_review: PersistedContextGraphDiffReviewProjection,
}

impl PersistedContextGraphHistoryReviewProjection {
    /// Returns the validated commit history used by this review.
    #[must_use]
    pub const fn history(&self) -> &CommitHistory {
        &self.history
    }

    /// Returns the existing version-backed graph review projection.
    #[must_use]
    pub const fn graph_review(&self) -> &PersistedContextGraphDiffReviewProjection {
        &self.graph_review
    }
}

/// Errors from the private history-bound graph review adapter.
#[derive(Debug, Error)]
pub enum PersistedContextGraphHistoryReviewError {
    /// The backend-owned multi-record witness could not be read or validated.
    #[error("Context Graph review witness read failed: {source}")]
    WitnessRead {
        /// Underlying storage failure.
        #[source]
        source: StorageRepositoryError,
    },
    /// The branch-bound graph witness could not be read or validated.
    #[error("Context branch-head graph review witness failed: {source}")]
    BranchWitnessRead {
        /// Underlying branch-bound repository failure.
        #[source]
        source: ContextGraphBranchHeadReviewWitnessRepositoryError,
    },
    /// The commit history could not be loaded or validated.
    #[error("Context commit history read failed for {context_id}: {source}")]
    HistoryRead {
        /// Context whose history was requested.
        context_id: contextlab_context_core::ContextId,
        /// Underlying storage error.
        #[source]
        source: StorageRepositoryError,
    },
    /// A requested review side does not exist in the validated history.
    #[error("Context commit history is missing {side:?} commit {commit_id}")]
    CommitMissing {
        /// Missing review side.
        side: PersistedContextGraphDiffReviewSide,
        /// Missing commit identity.
        commit_id: CommitId,
    },
    /// The exact pair is not a normal first-parent replay range.
    #[error("Context commit history cannot replay from {source} to {target}: {reason}")]
    ReplayPathInvalid {
        /// Requested baseline commit.
        source: CommitId,
        /// Requested revised commit.
        target: CommitId,
        /// Fail-closed versioning contract result.
        #[source]
        reason: HistoryError,
    },
    /// The loaded history belongs to a different Context than the requested snapshots.
    #[error(
        "Context commit history scope mismatch: expected Context {expected}, got Context {actual}"
    )]
    HistoryContextMismatch {
        /// Context requested by the graph review.
        expected: contextlab_context_core::ContextId,
        /// Context owned by the loaded history.
        actual: contextlab_context_core::ContextId,
    },
    /// The existing exact-snapshot graph review rejected the pair.
    #[error("persisted Context Graph review failed: {source}")]
    GraphReview {
        /// Existing graph review error.
        #[source]
        source: PersistedContextGraphDiffReviewError,
    },
}

/// Private adapter that validates history before delegating to the graph review service.
#[derive(Debug, Clone, Copy)]
pub struct PersistedContextGraphHistoryReviewService<'repository, S: ?Sized, H: ?Sized> {
    snapshot_repository: &'repository S,
    history_repository: &'repository H,
}

impl<'repository, S, H> PersistedContextGraphHistoryReviewService<'repository, S, H>
where
    S: ContextLifecycleReadRepository + ?Sized,
    H: ContextCommitHistoryRepository + ?Sized,
{
    /// Creates a history-bound review adapter over one repository.
    #[must_use]
    pub const fn new(
        snapshot_repository: &'repository S,
        history_repository: &'repository H,
    ) -> Self {
        Self {
            snapshot_repository,
            history_repository,
        }
    }

    /// Loads one complete history, validates both commits, then reuses graph review.
    pub async fn review(
        &self,
        source_scope: CommitGraphSnapshotScope,
        target_scope: CommitGraphSnapshotScope,
    ) -> Result<PersistedContextGraphHistoryReviewProjection, PersistedContextGraphHistoryReviewError>
    {
        validate_scope_pair(source_scope, target_scope)
            .map_err(|source| PersistedContextGraphHistoryReviewError::GraphReview { source })?;
        let history = self.load_history(source_scope.context_id()).await?;
        self.review_loaded(source_scope, target_scope, history)
            .await
    }

    async fn load_history(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<CommitHistory, PersistedContextGraphHistoryReviewError> {
        self.history_repository
            .load_context_commit_history(context_id)
            .await
            .map_err(
                |source| PersistedContextGraphHistoryReviewError::HistoryRead {
                    context_id,
                    source,
                },
            )
    }

    async fn review_loaded(
        &self,
        source_scope: CommitGraphSnapshotScope,
        target_scope: CommitGraphSnapshotScope,
        history: CommitHistory,
    ) -> Result<PersistedContextGraphHistoryReviewProjection, PersistedContextGraphHistoryReviewError>
    {
        if history.context_id() != source_scope.context_id()
            || history.context_id() != target_scope.context_id()
        {
            return Err(
                PersistedContextGraphHistoryReviewError::HistoryContextMismatch {
                    expected: source_scope.context_id(),
                    actual: history.context_id(),
                },
            );
        }
        for (side, commit_id) in [
            (
                PersistedContextGraphDiffReviewSide::Source,
                source_scope.commit_id(),
            ),
            (
                PersistedContextGraphDiffReviewSide::Target,
                target_scope.commit_id(),
            ),
        ] {
            if history.commit(commit_id).is_none() {
                return Err(PersistedContextGraphHistoryReviewError::CommitMissing {
                    side,
                    commit_id,
                });
            }
        }

        history
            .normal_first_parent_path(source_scope.commit_id(), target_scope.commit_id())
            .map_err(
                |reason| PersistedContextGraphHistoryReviewError::ReplayPathInvalid {
                    source: source_scope.commit_id(),
                    target: target_scope.commit_id(),
                    reason,
                },
            )?;

        let graph_review = PersistedContextGraphDiffReviewService::new(self.snapshot_repository)
            .review(source_scope, target_scope)
            .await
            .map_err(|source| PersistedContextGraphHistoryReviewError::GraphReview { source })?;

        Ok(PersistedContextGraphHistoryReviewProjection {
            history,
            graph_review,
        })
    }
}

/// Private review adapter over one backend-owned history-plus-snapshot witness.
#[derive(Debug, Clone, Copy)]
pub struct PersistedContextGraphWitnessReviewService<'repository, R: ?Sized> {
    repository: &'repository R,
}

impl<'repository, R> PersistedContextGraphWitnessReviewService<'repository, R>
where
    R: ContextGraphReviewWitnessRepository + ?Sized,
{
    /// Creates a review adapter over one atomic witness repository.
    #[must_use]
    pub const fn new(repository: &'repository R) -> Self {
        Self { repository }
    }

    /// Reads one witness and delegates the pair to the existing graph review core.
    pub async fn review(
        &self,
        source_scope: CommitGraphSnapshotScope,
        target_scope: CommitGraphSnapshotScope,
    ) -> Result<PersistedContextGraphHistoryReviewProjection, PersistedContextGraphHistoryReviewError>
    {
        validate_scope_pair(source_scope, target_scope)
            .map_err(|source| PersistedContextGraphHistoryReviewError::GraphReview { source })?;
        let witness = self
            .repository
            .read_context_graph_review_witness(source_scope, target_scope)
            .await
            .map_err(|source| PersistedContextGraphHistoryReviewError::WitnessRead { source })?;
        let graph_review = PersistedContextGraphDiffReviewService::<
            dyn ContextLifecycleReadRepository,
        >::project_snapshots(
            witness.source().clone(), witness.target().clone()
        )
        .map_err(|source| PersistedContextGraphHistoryReviewError::GraphReview { source })?;
        Ok(PersistedContextGraphHistoryReviewProjection {
            history: witness.history().clone(),
            graph_review,
        })
    }
}

impl<R> PersistedContextGraphWitnessReviewService<'_, R>
where
    R: ContextGraphBranchHeadReviewWitnessRepository + ?Sized,
{
    /// Reads a branch-bound witness and delegates its exact pair to graph review.
    pub async fn review_branch_head(
        &self,
        source_scope: CommitGraphSnapshotScope,
        branch: BranchName,
    ) -> Result<PersistedContextGraphHistoryReviewProjection, PersistedContextGraphHistoryReviewError>
    {
        validate_single_scope(PersistedContextGraphDiffReviewSide::Source, source_scope)
            .map_err(|source| PersistedContextGraphHistoryReviewError::GraphReview { source })?;
        let witness = self
            .repository
            .read_context_graph_branch_head_review_witness(source_scope, branch)
            .await
            .map_err(
                |source| PersistedContextGraphHistoryReviewError::BranchWitnessRead { source },
            )?;
        let graph_review = PersistedContextGraphDiffReviewService::<
            dyn ContextLifecycleReadRepository,
        >::project_snapshots(
            witness.witness().source().clone(),
            witness.witness().target().clone(),
        )
        .map_err(|source| PersistedContextGraphHistoryReviewError::GraphReview { source })?;
        Ok(PersistedContextGraphHistoryReviewProjection {
            history: witness.witness().history().clone(),
            graph_review,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1, CommitGraphSnapshot,
        ContextComponentStateSnapshotAtCommit, ContextLifecycleReadFacts,
        ContextLifecycleReadRepository, StorageRepositoryError,
    };
    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use contextlab_context_core::{ContextId, ProjectId};
    use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
    use contextlab_versioning::{BranchHead, ContextChange, ContextCommit, ReplayState};
    use std::collections::BTreeMap;

    struct Repository {
        history: CommitHistory,
        facts: BTreeMap<String, ContextLifecycleReadFacts>,
    }

    #[async_trait]
    impl ContextCommitHistoryRepository for Repository {
        async fn load_context_commit_history(
            &self,
            _context_id: ContextId,
        ) -> Result<CommitHistory, StorageRepositoryError> {
            Ok(self.history.clone())
        }
    }

    #[async_trait]
    impl ContextLifecycleReadRepository for Repository {
        async fn get_context_lifecycle_read_facts(
            &self,
            _context_id: ContextId,
            commit_id: CommitId,
        ) -> Result<ContextLifecycleReadFacts, StorageRepositoryError> {
            self.facts
                .get(&commit_id.to_string())
                .cloned()
                .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                    scope: format!("commit:{commit_id}"),
                })
        }
    }

    fn fixture() -> (
        Repository,
        CommitGraphSnapshotScope,
        CommitGraphSnapshotScope,
    ) {
        let project = ProjectId::from_uuid(uuid::Uuid::from_u128(1));
        let context = ContextId::from_uuid(uuid::Uuid::from_u128(2));
        let root = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(3)),
            context,
            BranchName::default(),
            "Create Context",
            Vec::new(),
            vec![ContextChange::created_context("Context")],
            timestamp(1),
        )
        .expect("root");
        let revised = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(4)),
            context,
            BranchName::default(),
            "Update Context",
            vec![root.id()],
            vec![ContextChange::updated_metadata(
                contextlab_context_core::ContextMetadata::new(timestamp(2)),
                "Update Context",
            )],
            timestamp(2),
        )
        .expect("revised");
        let history = CommitHistory::try_from_parts(
            context,
            [root.clone(), revised.clone()],
            [BranchHead::new(BranchName::default(), Some(revised.id()))],
        )
        .expect("history");
        let source_scope = CommitGraphSnapshotScope::new(project, context, root.id());
        let target_scope = CommitGraphSnapshotScope::new(project, context, revised.id());
        let source_state =
            ReplayState::from_commits(context, std::slice::from_ref(&root)).expect("source replay");
        let target_state =
            ReplayState::from_commits(context, &[root, revised]).expect("target replay");
        let facts = BTreeMap::from([
            (
                source_scope.commit_id().to_string(),
                lifecycle_facts(source_scope, source_state, "v1"),
            ),
            (
                target_scope.commit_id().to_string(),
                lifecycle_facts(target_scope, target_state, "v2"),
            ),
        ]);
        (Repository { history, facts }, source_scope, target_scope)
    }

    fn lifecycle_facts(
        scope: CommitGraphSnapshotScope,
        replay: ReplayState,
        label: &str,
    ) -> ContextLifecycleReadFacts {
        ContextLifecycleReadFacts::from_parts(
            scope,
            ContextComponentStateSnapshotAtCommit::new(
                scope.context_id(),
                scope.commit_id(),
                Vec::new(),
            ),
            Vec::new(),
            replay,
            snapshot(scope, label),
        )
        .expect("lifecycle facts")
    }

    fn snapshot(scope: CommitGraphSnapshotScope, label: &str) -> CommitGraphSnapshot {
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{}", scope.context_id()),
                    GraphNodeKind::Context,
                    label,
                )
                .expect("node"),
            )
            .expect("graph node");
        CommitGraphSnapshot::new(scope, graph, timestamp(1), COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1)
            .expect("snapshot")
    }

    fn timestamp(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().expect("timestamp")
    }

    #[tokio::test]
    async fn binds_complete_history_to_the_existing_graph_review() {
        let (repository, source_scope, target_scope) = fixture();
        let projection = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
            .review(source_scope, target_scope)
            .await
            .expect("history-bound graph review");

        assert_eq!(projection.history().context_id(), source_scope.context_id());
        assert_eq!(
            projection.history().head(&BranchName::default()),
            Some(target_scope.commit_id())
        );
        assert_eq!(projection.graph_review().source().scope(), source_scope);
        assert_eq!(
            projection
                .graph_review()
                .review()
                .diff()
                .modified_nodes()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn preserves_exact_snapshot_missing_error_after_history_validation() {
        let (mut repository, source_scope, target_scope) = fixture();
        repository
            .facts
            .remove(&target_scope.commit_id().to_string());
        let error = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
            .review(source_scope, target_scope)
            .await
            .expect_err("missing exact snapshot");
        assert!(matches!(
            error,
            PersistedContextGraphHistoryReviewError::GraphReview {
                source: PersistedContextGraphDiffReviewError::LifecycleMissing { .. }
            }
        ));
    }

    #[tokio::test]
    async fn rejects_a_snapshot_commit_missing_from_complete_history() {
        let (mut repository, source_scope, target_scope) = fixture();
        let source_commit = repository
            .history
            .commit(source_scope.commit_id())
            .cloned()
            .expect("source commit");
        repository.history = CommitHistory::try_from_parts(
            source_scope.context_id(),
            [source_commit],
            [BranchHead::new(
                BranchName::default(),
                Some(source_scope.commit_id()),
            )],
        )
        .expect("truncated history fixture");

        let error = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
            .review(source_scope, target_scope)
            .await
            .expect_err("target commit is absent from history");
        assert!(matches!(
            error,
            PersistedContextGraphHistoryReviewError::CommitMissing {
                side: PersistedContextGraphDiffReviewSide::Target,
                commit_id,
            } if commit_id == target_scope.commit_id()
        ));
    }

    #[tokio::test]
    async fn rejects_a_reversed_source_and_target_before_snapshot_review() {
        let (repository, source_scope, target_scope) = fixture();
        let error = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
            .review(target_scope, source_scope)
            .await
            .expect_err("reversed history range must fail closed");

        assert!(matches!(
            error,
            PersistedContextGraphHistoryReviewError::ReplayPathInvalid {
                source,
                target,
                reason: HistoryError::InvalidNormalReplayRange {
                    source_commit,
                    target_commit,
                },
            } if source == target_scope.commit_id()
                && target == source_scope.commit_id()
                && source_commit == target_scope.commit_id()
                && target_commit == source_scope.commit_id()
        ));
    }

    #[test]
    fn atomic_witness_rejects_a_reversed_normal_replay_range() {
        let (repository, source_scope, target_scope) = fixture();
        let history = repository.history.clone();
        let source = repository
            .facts
            .get(&source_scope.commit_id().to_string())
            .expect("source facts")
            .graph_snapshot()
            .clone();
        let target = repository
            .facts
            .get(&target_scope.commit_id().to_string())
            .expect("target facts")
            .graph_snapshot()
            .clone();

        let error = ContextGraphReviewWitness::try_from_parts(history, target, source)
            .expect_err("reversed atomic witness must fail closed");

        assert!(matches!(
            error,
            ContextGraphReviewWitnessError::ReplayPathInvalid {
                source_commit,
                target_commit,
                reason: HistoryError::InvalidNormalReplayRange {
                    source_commit: reason_source,
                    target_commit: reason_target,
                },
            } if source_commit == target_scope.commit_id()
                && target_commit == source_scope.commit_id()
                && reason_source == target_scope.commit_id()
                && reason_target == source_scope.commit_id()
        ));
    }

    #[test]
    fn atomic_witness_rejects_a_missing_intermediate_snapshot() {
        let project = ProjectId::from_uuid(uuid::Uuid::from_u128(10));
        let context = ContextId::from_uuid(uuid::Uuid::from_u128(11));
        let root = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(12)),
            context,
            BranchName::default(),
            "Root",
            Vec::new(),
            vec![ContextChange::created_context("Context")],
            timestamp(1),
        )
        .expect("root");
        let middle = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(13)),
            context,
            BranchName::default(),
            "Middle",
            vec![root.id()],
            vec![ContextChange::updated_metadata(
                contextlab_context_core::ContextMetadata::new(timestamp(2)),
                "Middle",
            )],
            timestamp(2),
        )
        .expect("middle");
        let target = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(14)),
            context,
            BranchName::default(),
            "Target",
            vec![middle.id()],
            vec![ContextChange::updated_metadata(
                contextlab_context_core::ContextMetadata::new(timestamp(3)),
                "Target",
            )],
            timestamp(3),
        )
        .expect("target");
        let history = CommitHistory::try_from_parts(
            context,
            [root.clone(), middle.clone(), target.clone()],
            [BranchHead::new(BranchName::default(), Some(target.id()))],
        )
        .expect("history");
        let source_scope = CommitGraphSnapshotScope::new(project, context, root.id());
        let target_scope = CommitGraphSnapshotScope::new(project, context, target.id());
        let error = ContextGraphReviewWitness::try_from_parts_with_intermediate_snapshots(
            history,
            snapshot(source_scope, "root"),
            snapshot(target_scope, "target"),
            Vec::new(),
        )
        .expect_err("missing intermediate snapshot must fail closed");

        assert!(matches!(
            error,
            ContextGraphReviewWitnessError::IntermediateSnapshotMissing { commit_id }
                if commit_id == middle.id()
        ));
    }

    #[tokio::test]
    async fn rejects_unrelated_commits_before_snapshot_review() {
        let (mut repository, source_scope, target_scope) = fixture();
        let unrelated = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(7)),
            source_scope.context_id(),
            BranchName::new("unrelated").expect("branch"),
            "Unrelated root",
            Vec::new(),
            vec![ContextChange::created_context("Unrelated")],
            timestamp(3),
        )
        .expect("unrelated commit");
        let root = repository
            .history
            .commit(source_scope.commit_id())
            .cloned()
            .expect("root");
        let revised = repository
            .history
            .commit(target_scope.commit_id())
            .cloned()
            .expect("revised");
        repository.history = CommitHistory::try_from_parts(
            source_scope.context_id(),
            [root, revised, unrelated.clone()],
            [
                BranchHead::new(BranchName::default(), Some(target_scope.commit_id())),
                BranchHead::new(
                    BranchName::new("unrelated").expect("branch"),
                    Some(unrelated.id()),
                ),
            ],
        )
        .expect("history with unrelated root");
        let unrelated_scope = CommitGraphSnapshotScope::new(
            source_scope.project_id(),
            source_scope.context_id(),
            unrelated.id(),
        );

        let error = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
            .review(source_scope, unrelated_scope)
            .await
            .expect_err("unrelated history range must fail closed");

        assert!(matches!(
            error,
            PersistedContextGraphHistoryReviewError::ReplayPathInvalid {
                reason: HistoryError::InvalidNormalReplayRange { .. },
                ..
            }
        ));
    }

    #[tokio::test]
    async fn rejects_merge_ancestry_before_normal_graph_review() {
        let (mut repository, source_scope, target_scope) = fixture();
        let root = repository
            .history
            .commit(source_scope.commit_id())
            .cloned()
            .expect("root");
        let revised = repository
            .history
            .commit(target_scope.commit_id())
            .cloned()
            .expect("revised");
        let side = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(8)),
            source_scope.context_id(),
            BranchName::new("feature").expect("branch"),
            "Feature commit",
            vec![root.id()],
            vec![ContextChange::created_context("Feature")],
            timestamp(3),
        )
        .expect("side commit");
        let merge = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(9)),
            source_scope.context_id(),
            BranchName::default(),
            "Merge commit",
            vec![revised.id(), side.id()],
            vec![ContextChange::created_context("Merge")],
            timestamp(4),
        )
        .expect("merge commit");
        repository.history = CommitHistory::try_from_parts(
            source_scope.context_id(),
            [root, revised, side, merge.clone()],
            [BranchHead::new(BranchName::default(), Some(merge.id()))],
        )
        .expect("history with merge");
        let merge_scope = CommitGraphSnapshotScope::new(
            source_scope.project_id(),
            source_scope.context_id(),
            merge.id(),
        );

        let error = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
            .review(source_scope, merge_scope)
            .await
            .expect_err("merge ancestry belongs to merge review");

        assert!(matches!(
            error,
            PersistedContextGraphHistoryReviewError::ReplayPathInvalid {
                reason: HistoryError::NonLinearAncestry { commit_id },
                ..
            } if commit_id == merge.id()
        ));
    }

    #[tokio::test]
    async fn rejects_history_owned_by_a_different_context_before_commit_membership() {
        let (mut repository, source_scope, target_scope) = fixture();
        let other_context = ContextId::from_uuid(uuid::Uuid::from_u128(5));
        let other_commit = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(6)),
            other_context,
            BranchName::default(),
            "Other Context",
            Vec::new(),
            vec![ContextChange::created_context("Other Context")],
            timestamp(1),
        )
        .expect("other context commit");
        repository.history = CommitHistory::try_from_parts(
            other_context,
            [other_commit.clone()],
            [BranchHead::new(
                BranchName::default(),
                Some(other_commit.id()),
            )],
        )
        .expect("other context history");

        let error = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
            .review(source_scope, target_scope)
            .await
            .expect_err("cross-context history must fail closed");
        assert!(matches!(
            error,
            PersistedContextGraphHistoryReviewError::HistoryContextMismatch {
                expected,
                actual,
            } if expected == source_scope.context_id() && actual == other_context
        ));
    }
}
