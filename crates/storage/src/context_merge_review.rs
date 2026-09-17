//! Private storage-backed three-way Context Graph review.

use crate::{
    CommitGraphSnapshot, CommitGraphSnapshotRepository, CommitGraphSnapshotScope,
    StorageRepositoryError,
};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::{
    GraphMergeClassification, GraphMergeClassificationError, GraphMergeSnapshotSide,
    VersionedContextGraphMergeReviewError, VersionedContextGraphMergeReviewProjectionV1,
    VersionedContextGraphMergeReviewRequestV1, VersionedContextGraphMergeReviewService,
    VersionedContextGraphSnapshotV1, VersionedContextScopeV1,
};
use contextlab_versioning::{CommitId, MergePlan, MergePlanError};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// The side of one persisted snapshot in a three-way review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedContextGraphMergeReviewSide {
    /// The common base snapshot.
    Base,
    /// The left branch snapshot.
    Left,
    /// The right branch snapshot.
    Right,
}

/// Exact scope for a read-only persisted three-way Context Graph review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ContextMergeInputScope {
    project_id: ProjectId,
    context_id: ContextId,
    base_commit_id: CommitId,
    left_commit_id: CommitId,
    right_commit_id: CommitId,
}

#[derive(Deserialize)]
struct ContextMergeInputScopeWire {
    project_id: ProjectId,
    context_id: ContextId,
    base_commit_id: CommitId,
    left_commit_id: CommitId,
    right_commit_id: CommitId,
}

impl<'de> Deserialize<'de> for ContextMergeInputScope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ContextMergeInputScopeWire::deserialize(deserializer)?;
        Self::new(
            wire.project_id,
            wire.context_id,
            wire.base_commit_id,
            wire.left_commit_id,
            wire.right_commit_id,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl ContextMergeInputScope {
    /// Creates a scope with one project/Context and three distinct commits.
    pub fn new(
        project_id: ProjectId,
        context_id: ContextId,
        base_commit_id: CommitId,
        left_commit_id: CommitId,
        right_commit_id: CommitId,
    ) -> Result<Self, ContextMergeInputScopeError> {
        if base_commit_id == left_commit_id || base_commit_id == right_commit_id {
            return Err(ContextMergeInputScopeError::DuplicateCommit {
                side: if base_commit_id == left_commit_id {
                    PersistedContextGraphMergeReviewSide::Left
                } else {
                    PersistedContextGraphMergeReviewSide::Right
                },
                commit_id: if base_commit_id == left_commit_id {
                    left_commit_id
                } else {
                    right_commit_id
                },
            });
        }
        if left_commit_id == right_commit_id {
            return Err(ContextMergeInputScopeError::DuplicateCommit {
                side: PersistedContextGraphMergeReviewSide::Right,
                commit_id: right_commit_id,
            });
        }

        for (identifier, value) in [
            (ContextMergeInputIdentifier::Project, project_id.as_uuid()),
            (ContextMergeInputIdentifier::Context, context_id.as_uuid()),
            (
                ContextMergeInputIdentifier::BaseCommit,
                base_commit_id.as_uuid(),
            ),
            (
                ContextMergeInputIdentifier::LeftCommit,
                left_commit_id.as_uuid(),
            ),
            (
                ContextMergeInputIdentifier::RightCommit,
                right_commit_id.as_uuid(),
            ),
        ] {
            if value.is_nil() {
                return Err(ContextMergeInputScopeError::NilIdentifier { identifier });
            }
        }

        Ok(Self {
            project_id,
            context_id,
            base_commit_id,
            left_commit_id,
            right_commit_id,
        })
    }

    /// Returns the project scope.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the Context scope.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the base commit scope.
    #[must_use]
    pub const fn base_commit_id(self) -> CommitId {
        self.base_commit_id
    }

    /// Returns the left commit scope.
    #[must_use]
    pub const fn left_commit_id(self) -> CommitId {
        self.left_commit_id
    }

    /// Returns the right commit scope.
    #[must_use]
    pub const fn right_commit_id(self) -> CommitId {
        self.right_commit_id
    }

    pub(crate) fn snapshot_scope(
        self,
        side: PersistedContextGraphMergeReviewSide,
    ) -> CommitGraphSnapshotScope {
        let commit_id = match side {
            PersistedContextGraphMergeReviewSide::Base => self.base_commit_id,
            PersistedContextGraphMergeReviewSide::Left => self.left_commit_id,
            PersistedContextGraphMergeReviewSide::Right => self.right_commit_id,
        };
        CommitGraphSnapshotScope::new(self.project_id, self.context_id, commit_id)
    }
}

/// Exact Context and two branch tips used for server-owned ancestry resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ContextMergeTipScope {
    project_id: ProjectId,
    context_id: ContextId,
    left_commit_id: CommitId,
    right_commit_id: CommitId,
}

#[derive(Deserialize)]
struct ContextMergeTipScopeWire {
    project_id: ProjectId,
    context_id: ContextId,
    left_commit_id: CommitId,
    right_commit_id: CommitId,
}

impl<'de> Deserialize<'de> for ContextMergeTipScope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ContextMergeTipScopeWire::deserialize(deserializer)?;
        Self::new(
            wire.project_id,
            wire.context_id,
            wire.left_commit_id,
            wire.right_commit_id,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl ContextMergeTipScope {
    /// Creates a scope with one Context and two distinct branch tips.
    pub fn new(
        project_id: ProjectId,
        context_id: ContextId,
        left_commit_id: CommitId,
        right_commit_id: CommitId,
    ) -> Result<Self, ContextMergeInputScopeError> {
        if left_commit_id == right_commit_id {
            return Err(ContextMergeInputScopeError::DuplicateCommit {
                side: PersistedContextGraphMergeReviewSide::Right,
                commit_id: right_commit_id,
            });
        }

        for (identifier, value) in [
            (ContextMergeInputIdentifier::Project, project_id.as_uuid()),
            (ContextMergeInputIdentifier::Context, context_id.as_uuid()),
            (
                ContextMergeInputIdentifier::LeftCommit,
                left_commit_id.as_uuid(),
            ),
            (
                ContextMergeInputIdentifier::RightCommit,
                right_commit_id.as_uuid(),
            ),
        ] {
            if value.is_nil() {
                return Err(ContextMergeInputScopeError::NilIdentifier { identifier });
            }
        }

        Ok(Self {
            project_id,
            context_id,
            left_commit_id,
            right_commit_id,
        })
    }

    /// Returns the project scope.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the Context scope.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the left branch tip.
    #[must_use]
    pub const fn left_commit_id(self) -> CommitId {
        self.left_commit_id
    }

    /// Returns the right branch tip.
    #[must_use]
    pub const fn right_commit_id(self) -> CommitId {
        self.right_commit_id
    }
}

/// The server-owned merge plan and its three immutable Context Graph snapshots.
///
/// A repository must construct this witness inside one consistent read boundary;
/// consumers cannot construct it from separately observed plan and snapshot data.
#[derive(Debug, Clone)]
pub struct ContextMergeReviewWitness {
    plan: MergePlan,
    base: CommitGraphSnapshot,
    left: CommitGraphSnapshot,
    right: CommitGraphSnapshot,
}

impl ContextMergeReviewWitness {
    /// Binds one three-way plan to its exact base/left/right snapshot scopes.
    pub fn new(
        plan: MergePlan,
        base: CommitGraphSnapshot,
        left: CommitGraphSnapshot,
        right: CommitGraphSnapshot,
    ) -> Result<Self, ContextMergeReviewWitnessError> {
        let MergePlan::ThreeWay {
            base: base_commit_id,
            left: left_commit_id,
            right: right_commit_id,
        } = plan.clone()
        else {
            return Err(ContextMergeReviewWitnessError::NonThreeWay);
        };

        let base_scope = base.scope();
        let scope = ContextMergeInputScope::new(
            base_scope.project_id(),
            base_scope.context_id(),
            base_commit_id,
            left_commit_id,
            right_commit_id,
        )
        .map_err(ContextMergeReviewWitnessError::InvalidInputScope)?;

        for (side, actual, expected) in [
            (
                PersistedContextGraphMergeReviewSide::Base,
                base.scope(),
                scope.snapshot_scope(PersistedContextGraphMergeReviewSide::Base),
            ),
            (
                PersistedContextGraphMergeReviewSide::Left,
                left.scope(),
                scope.snapshot_scope(PersistedContextGraphMergeReviewSide::Left),
            ),
            (
                PersistedContextGraphMergeReviewSide::Right,
                right.scope(),
                scope.snapshot_scope(PersistedContextGraphMergeReviewSide::Right),
            ),
        ] {
            if actual != expected {
                return Err(ContextMergeReviewWitnessError::SnapshotScopeMismatch {
                    side,
                    expected,
                    actual,
                });
            }
        }

        Ok(Self {
            plan,
            base,
            left,
            right,
        })
    }

    /// Verifies that the witness represents the exact tips requested by the caller.
    ///
    /// The repository port receives the requested scope, but a custom or faulty
    /// implementation must not be allowed to return a witness for another pair of
    /// tips. Keep this check in the application service as a second fail-closed
    /// boundary.
    fn validate_requested_scope(
        &self,
        requested: ContextMergeTipScope,
    ) -> Result<(), ContextMergeReviewWitnessError> {
        let MergePlan::ThreeWay {
            left: left_commit_id,
            right: right_commit_id,
            ..
        } = self.plan
        else {
            return Err(ContextMergeReviewWitnessError::NonThreeWay);
        };
        let actual = ContextMergeTipScope::new(
            self.base.scope().project_id(),
            self.base.scope().context_id(),
            left_commit_id,
            right_commit_id,
        )
        .map_err(ContextMergeReviewWitnessError::InvalidInputScope)?;
        if actual != requested {
            return Err(ContextMergeReviewWitnessError::TipScopeMismatch);
        }
        Ok(())
    }

    /// Returns the server-owned merge plan.
    #[must_use]
    pub fn plan(&self) -> &MergePlan {
        &self.plan
    }

    /// Returns the immutable base snapshot.
    #[must_use]
    pub fn base(&self) -> &CommitGraphSnapshot {
        &self.base
    }

    /// Returns the immutable left snapshot.
    #[must_use]
    pub fn left(&self) -> &CommitGraphSnapshot {
        &self.left
    }

    /// Returns the immutable right snapshot.
    #[must_use]
    pub fn right(&self) -> &CommitGraphSnapshot {
        &self.right
    }

    fn into_parts(
        self,
    ) -> (
        MergePlan,
        CommitGraphSnapshot,
        CommitGraphSnapshot,
        CommitGraphSnapshot,
    ) {
        (self.plan, self.base, self.left, self.right)
    }
}

/// Errors constructing an atomic server-owned merge witness.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContextMergeReviewWitnessError {
    /// The resolved plan was not a three-way merge.
    #[error("merge review witness requires a three-way plan")]
    NonThreeWay,
    /// The plan scopes contained invalid identities.
    #[error("merge review witness has an invalid input scope: {0}")]
    InvalidInputScope(#[source] ContextMergeInputScopeError),
    /// A snapshot did not match the plan-derived exact scope.
    #[error("merge review witness has a {side:?} snapshot scope mismatch")]
    SnapshotScopeMismatch {
        /// Snapshot side with scope drift.
        side: PersistedContextGraphMergeReviewSide,
        /// Plan-derived expected scope.
        expected: CommitGraphSnapshotScope,
        /// Stored snapshot scope.
        actual: CommitGraphSnapshotScope,
    },
    /// The repository returned a witness for different requested branch tips.
    #[error("merge review witness tip scope does not match the request")]
    TipScopeMismatch,
}

/// Errors returned while loading a server-owned merge witness.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContextMergeReviewWitnessRepositoryError {
    /// The consistent read boundary could not be completed.
    #[error("merge review witness read failed: {0}")]
    Read(#[source] StorageRepositoryError),
    /// One plan-derived snapshot was not materialized.
    #[error("merge review witness snapshot is unavailable: {scope}")]
    SnapshotUnavailable {
        /// Snapshot side that was absent.
        side: PersistedContextGraphMergeReviewSide,
        /// Exact missing scope.
        scope: CommitGraphSnapshotScope,
    },
    /// One plan-derived snapshot could not be materialized or validated.
    #[error("merge review witness snapshot read failed for {side:?}: {source}")]
    SnapshotRead {
        /// Snapshot side that failed.
        side: PersistedContextGraphMergeReviewSide,
        /// Exact requested scope.
        scope: CommitGraphSnapshotScope,
        /// Repository failure.
        #[source]
        source: StorageRepositoryError,
    },
    /// The server-owned commit graph could not resolve a merge plan.
    #[error("merge review witness plan resolution failed: {0}")]
    PlanResolution(#[source] MergePlanError),
    /// The repository returned a witness that violates its exact contract.
    #[error("merge review witness is invalid: {0}")]
    InvalidWitness(#[source] ContextMergeReviewWitnessError),
}

/// Repository boundary for atomically reading server-owned merge review inputs.
#[async_trait::async_trait]
pub trait ContextMergeReviewWitnessRepository: Send + Sync {
    /// Resolves the plan and reads all three immutable snapshots in one boundary.
    async fn load_context_merge_review_witness(
        &self,
        scope: ContextMergeTipScope,
    ) -> Result<ContextMergeReviewWitness, ContextMergeReviewWitnessRepositoryError>;
}

/// Errors constructing an exact three-way input scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ContextMergeInputScopeError {
    /// One commit identity was repeated across the three sides.
    #[error("three-way merge input repeats the {side:?} commit {commit_id}")]
    DuplicateCommit {
        /// The later side that repeated the commit.
        side: PersistedContextGraphMergeReviewSide,
        /// Repeated commit identity.
        commit_id: CommitId,
    },
    /// The identity family containing the nil identifier.
    #[error("three-way merge input contains a nil {identifier:?} identifier")]
    NilIdentifier {
        /// Identity family containing the nil identifier.
        identifier: ContextMergeInputIdentifier,
    },
}

/// Identity family used to attribute invalid exact-scope input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMergeInputIdentifier {
    /// Project identity.
    Project,
    /// Context identity.
    Context,
    /// Base commit identity.
    BaseCommit,
    /// Left commit identity.
    LeftCommit,
    /// Right commit identity.
    RightCommit,
}

/// Fail-closed errors from the persisted three-way review service.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PersistedContextGraphMergeReviewError {
    /// The exact Context commit DAG could not be loaded.
    #[error(
        "persisted Context Graph merge review could not read commit ancestry for {context_id}: {source}"
    )]
    CommitGraphRead {
        /// Exact Context whose commit graph was requested.
        context_id: ContextId,
        /// Repository failure.
        #[source]
        source: StorageRepositoryError,
    },
    /// The server-owned tips could not produce an ancestry plan.
    #[error(
        "persisted Context Graph merge review could not resolve the server-owned plan: {source}"
    )]
    PlanResolutionFailed {
        /// Ancestry resolution failure.
        #[source]
        source: MergePlanError,
    },
    /// A validated server-owned plan could not form the exact snapshot scope.
    #[error("persisted Context Graph merge review received an invalid exact scope: {source}")]
    InvalidInputScope {
        /// Exact scope validation failure.
        #[source]
        source: ContextMergeInputScopeError,
    },
    /// A requested snapshot could not be read.
    #[error(
        "persisted Context Graph merge review could not read {side:?} snapshot {scope}: {source}"
    )]
    SnapshotRead {
        /// Snapshot side that failed.
        side: PersistedContextGraphMergeReviewSide,
        /// Exact requested scope.
        scope: CommitGraphSnapshotScope,
        /// Repository failure.
        #[source]
        source: StorageRepositoryError,
    },
    /// A requested snapshot was not materialized.
    #[error("persisted Context Graph merge review has no {side:?} snapshot at {scope}")]
    SnapshotUnavailable {
        /// Snapshot side that was absent.
        side: PersistedContextGraphMergeReviewSide,
        /// Exact requested scope.
        scope: CommitGraphSnapshotScope,
    },
    /// A repository returned a snapshot outside the requested exact scope.
    #[error("persisted Context Graph merge review returned an out-of-scope {side:?} snapshot")]
    StoredScopeMismatch {
        /// Snapshot side with scope drift.
        side: PersistedContextGraphMergeReviewSide,
        /// Requested scope.
        expected: CommitGraphSnapshotScope,
        /// Returned scope.
        actual: CommitGraphSnapshotScope,
    },
    /// A repository returned a witness for different requested branch tips.
    #[error(
        "persisted Context Graph merge review returned a witness outside the requested tip scope"
    )]
    RequestedTipScopeMismatch,
    /// The pure classifier rejected the exact plan or graph inputs.
    #[error("persisted Context Graph merge review classification failed: {source}")]
    ClassificationFailed {
        /// Structured classifier error.
        #[source]
        source: GraphMergeClassificationError,
    },
}

/// Private adapter that reads exact persisted snapshots and delegates classification.
#[derive(Debug, Clone, Copy)]
pub struct PersistedContextGraphMergeReviewService<'repository, R: ?Sized> {
    repository: &'repository R,
}

impl<'repository, R: ?Sized> PersistedContextGraphMergeReviewService<'repository, R> {
    /// Creates a review service over one repository boundary.
    #[must_use]
    pub const fn new(repository: &'repository R) -> Self {
        Self { repository }
    }
}

impl<R> PersistedContextGraphMergeReviewService<'_, R>
where
    R: CommitGraphSnapshotRepository + ?Sized,
{
    /// Reads the complete base/left/right set once and delegates to the pure classifier.
    pub async fn review(
        &self,
        scope: ContextMergeInputScope,
        plan: &MergePlan,
    ) -> Result<GraphMergeClassification, PersistedContextGraphMergeReviewError> {
        self.review_projection(scope, plan)
            .await
            .map(|projection| projection.classification().clone())
    }

    async fn review_projection(
        &self,
        scope: ContextMergeInputScope,
        plan: &MergePlan,
    ) -> Result<VersionedContextGraphMergeReviewProjectionV1, PersistedContextGraphMergeReviewError>
    {
        validate_plan(scope, plan)?;
        let (base, left, right) = self
            .repository
            .get_commit_graph_snapshot_batch(scope)
            .await
            .map_err(|source| map_batch_read_error(scope, source))?;
        let base =
            validate_batch_snapshot(scope, PersistedContextGraphMergeReviewSide::Base, base)?;
        let left =
            validate_batch_snapshot(scope, PersistedContextGraphMergeReviewSide::Left, left)?;
        let right =
            validate_batch_snapshot(scope, PersistedContextGraphMergeReviewSide::Right, right)?;

        classify_review_inputs(scope, plan, base, left, right)
    }
}

impl<R> PersistedContextGraphMergeReviewService<'_, R>
where
    R: ContextMergeReviewWitnessRepository + ?Sized,
{
    /// Resolves and reviews a server-owned plan/snapshot witness from one repository boundary.
    pub async fn review_server_owned(
        &self,
        scope: ContextMergeTipScope,
    ) -> Result<GraphMergeClassification, PersistedContextGraphMergeReviewError> {
        self.review_server_owned_projection(scope)
            .await
            .map(|projection| projection.classification().clone())
    }

    /// Resolves the server-owned witness and preserves the complete V1 review projection.
    pub async fn review_server_owned_projection(
        &self,
        scope: ContextMergeTipScope,
    ) -> Result<VersionedContextGraphMergeReviewProjectionV1, PersistedContextGraphMergeReviewError>
    {
        let witness = self
            .repository
            .load_context_merge_review_witness(scope)
            .await
            .map_err(|source| map_witness_repository_error(scope.context_id(), source))?;
        witness.validate_requested_scope(scope).map_err(|source| {
            map_witness_repository_error(
                scope.context_id(),
                ContextMergeReviewWitnessRepositoryError::InvalidWitness(source),
            )
        })?;
        let (plan, base, left, right) = witness.into_parts();
        let MergePlan::ThreeWay {
            base: base_commit_id,
            left: left_commit_id,
            right: right_commit_id,
        } = plan.clone()
        else {
            unreachable!("the witness constructor rejects non-three-way plans")
        };
        let base_scope = base.scope();
        let input_scope = ContextMergeInputScope::new(
            base_scope.project_id(),
            base_scope.context_id(),
            base_commit_id,
            left_commit_id,
            right_commit_id,
        )
        .map_err(|source| PersistedContextGraphMergeReviewError::InvalidInputScope { source })?;

        classify_review_inputs(input_scope, &plan, base, left, right)
    }
}

fn classify_review_inputs(
    scope: ContextMergeInputScope,
    plan: &MergePlan,
    base: CommitGraphSnapshot,
    left: CommitGraphSnapshot,
    right: CommitGraphSnapshot,
) -> Result<VersionedContextGraphMergeReviewProjectionV1, PersistedContextGraphMergeReviewError> {
    let base = validate_batch_snapshot(scope, PersistedContextGraphMergeReviewSide::Base, base)?;
    let left = validate_batch_snapshot(scope, PersistedContextGraphMergeReviewSide::Left, left)?;
    let right = validate_batch_snapshot(scope, PersistedContextGraphMergeReviewSide::Right, right)?;

    let request = VersionedContextGraphMergeReviewRequestV1::new(
        plan.clone(),
        VersionedContextGraphSnapshotV1::new(
            VersionedContextScopeV1::new(
                scope.project_id(),
                scope.context_id(),
                scope.base_commit_id(),
            ),
            base.graph().clone(),
        ),
        VersionedContextGraphSnapshotV1::new(
            VersionedContextScopeV1::new(
                scope.project_id(),
                scope.context_id(),
                scope.left_commit_id(),
            ),
            left.graph().clone(),
        ),
        VersionedContextGraphSnapshotV1::new(
            VersionedContextScopeV1::new(
                scope.project_id(),
                scope.context_id(),
                scope.right_commit_id(),
            ),
            right.graph().clone(),
        ),
    );

    VersionedContextGraphMergeReviewService::review(request).map_err(|error| match error {
        VersionedContextGraphMergeReviewError::ClassificationFailed { source } => {
            PersistedContextGraphMergeReviewError::ClassificationFailed { source }
        }
        VersionedContextGraphMergeReviewError::UnsupportedSchemaVersion { .. } => {
            unreachable!("a constructed V1 request cannot have an unsupported schema")
        }
        VersionedContextGraphMergeReviewError::InvalidSnapshotScope { .. }
        | VersionedContextGraphMergeReviewError::DuplicateCommitIdentity { .. } => {
            unreachable!("the storage scope constructor already validates identities")
        }
    })
}

fn map_witness_repository_error(
    context_id: ContextId,
    source: ContextMergeReviewWitnessRepositoryError,
) -> PersistedContextGraphMergeReviewError {
    match source {
        ContextMergeReviewWitnessRepositoryError::Read(source) => {
            PersistedContextGraphMergeReviewError::CommitGraphRead { context_id, source }
        }
        ContextMergeReviewWitnessRepositoryError::SnapshotUnavailable { side, scope } => {
            PersistedContextGraphMergeReviewError::SnapshotUnavailable { side, scope }
        }
        ContextMergeReviewWitnessRepositoryError::SnapshotRead {
            side,
            scope,
            source,
        } => PersistedContextGraphMergeReviewError::SnapshotRead {
            side,
            scope,
            source,
        },
        ContextMergeReviewWitnessRepositoryError::PlanResolution(source) => {
            PersistedContextGraphMergeReviewError::PlanResolutionFailed { source }
        }
        ContextMergeReviewWitnessRepositoryError::InvalidWitness(source) => match source {
            ContextMergeReviewWitnessError::NonThreeWay => {
                PersistedContextGraphMergeReviewError::ClassificationFailed {
                    source: GraphMergeClassificationError::NotThreeWay,
                }
            }
            ContextMergeReviewWitnessError::InvalidInputScope(source) => {
                PersistedContextGraphMergeReviewError::InvalidInputScope { source }
            }
            ContextMergeReviewWitnessError::SnapshotScopeMismatch {
                side,
                expected,
                actual,
            } => PersistedContextGraphMergeReviewError::StoredScopeMismatch {
                side,
                expected,
                actual,
            },
            ContextMergeReviewWitnessError::TipScopeMismatch => {
                PersistedContextGraphMergeReviewError::RequestedTipScopeMismatch
            }
        },
    }
}

fn validate_batch_snapshot(
    scope: ContextMergeInputScope,
    side: PersistedContextGraphMergeReviewSide,
    snapshot: CommitGraphSnapshot,
) -> Result<CommitGraphSnapshot, PersistedContextGraphMergeReviewError> {
    let expected = scope.snapshot_scope(side);
    if snapshot.scope() != expected {
        return Err(PersistedContextGraphMergeReviewError::StoredScopeMismatch {
            side,
            expected,
            actual: snapshot.scope(),
        });
    }
    Ok(snapshot)
}

fn map_batch_read_error(
    scope: ContextMergeInputScope,
    source: StorageRepositoryError,
) -> PersistedContextGraphMergeReviewError {
    let unavailable_side = match &source {
        StorageRepositoryError::ScopeUnavailable {
            scope: unavailable_scope,
        } => [
            PersistedContextGraphMergeReviewSide::Base,
            PersistedContextGraphMergeReviewSide::Left,
            PersistedContextGraphMergeReviewSide::Right,
        ]
        .into_iter()
        .find(|side| scope.snapshot_scope(*side).to_string() == *unavailable_scope),
        _ => None,
    };
    let side = unavailable_side.unwrap_or(PersistedContextGraphMergeReviewSide::Base);
    let expected = scope.snapshot_scope(side);

    if unavailable_side.is_some() {
        PersistedContextGraphMergeReviewError::SnapshotUnavailable {
            side,
            scope: expected,
        }
    } else {
        PersistedContextGraphMergeReviewError::SnapshotRead {
            side,
            scope: expected,
            source,
        }
    }
}

fn validate_plan(
    scope: ContextMergeInputScope,
    plan: &MergePlan,
) -> Result<(), PersistedContextGraphMergeReviewError> {
    let MergePlan::ThreeWay { base, left, right } = plan else {
        return Err(
            PersistedContextGraphMergeReviewError::ClassificationFailed {
                source: GraphMergeClassificationError::NotThreeWay,
            },
        );
    };

    for (side, expected, actual) in [
        (GraphMergeSnapshotSide::Base, *base, scope.base_commit_id()),
        (GraphMergeSnapshotSide::Left, *left, scope.left_commit_id()),
        (
            GraphMergeSnapshotSide::Right,
            *right,
            scope.right_commit_id(),
        ),
    ] {
        if expected != actual {
            return Err(
                PersistedContextGraphMergeReviewError::ClassificationFailed {
                    source: GraphMergeClassificationError::PlanIdentityMismatch {
                        side,
                        expected,
                        actual,
                    },
                },
            );
        }
    }

    Ok(())
}
