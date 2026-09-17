//! Version-bound, local-only review projection for three-way Context Graphs.

use crate::{
    GraphMergeClassification, GraphMergeClassificationError, GraphMergeConflictClassifier,
    GraphMergeSnapshotSide, GraphSnapshotRef, VersionedContextScopeV1,
};
use contextlab_graph::ContextGraph;
use contextlab_versioning::{CommitId, MergePlan};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// The explicit schema revision for a version-bound graph merge review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionedGraphMergeReviewSchemaVersion {
    /// The initial stable version-bound graph review schema.
    V1,
}

/// An owned Context Graph snapshot bound to one exact version identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedContextGraphSnapshotV1 {
    scope: VersionedContextScopeV1,
    graph: ContextGraph,
}

impl VersionedContextGraphSnapshotV1 {
    /// Creates an owned graph snapshot with an exact project/Context/commit scope.
    #[must_use]
    pub const fn new(scope: VersionedContextScopeV1, graph: ContextGraph) -> Self {
        Self { scope, graph }
    }

    /// Returns the exact snapshot scope.
    #[must_use]
    pub const fn scope(&self) -> VersionedContextScopeV1 {
        self.scope
    }

    /// Returns the owned graph snapshot.
    #[must_use]
    pub const fn graph(&self) -> &ContextGraph {
        &self.graph
    }
}

/// Complete version-bound inputs for a three-way Context Graph review.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VersionedContextGraphMergeReviewRequestV1 {
    schema_version: VersionedGraphMergeReviewSchemaVersion,
    plan: MergePlan,
    base: VersionedContextGraphSnapshotV1,
    left: VersionedContextGraphSnapshotV1,
    right: VersionedContextGraphSnapshotV1,
}

impl VersionedContextGraphMergeReviewRequestV1 {
    /// Creates a V1 request. The classifier performs plan and cross-snapshot validation.
    #[must_use]
    pub const fn new(
        plan: MergePlan,
        base: VersionedContextGraphSnapshotV1,
        left: VersionedContextGraphSnapshotV1,
        right: VersionedContextGraphSnapshotV1,
    ) -> Self {
        Self {
            schema_version: VersionedGraphMergeReviewSchemaVersion::V1,
            plan,
            base,
            left,
            right,
        }
    }

    /// Returns the explicit request schema version.
    #[must_use]
    pub const fn schema_version(&self) -> VersionedGraphMergeReviewSchemaVersion {
        self.schema_version
    }

    /// Returns the ancestry plan.
    #[must_use]
    pub const fn plan(&self) -> &MergePlan {
        &self.plan
    }

    /// Returns the base snapshot.
    #[must_use]
    pub const fn base(&self) -> &VersionedContextGraphSnapshotV1 {
        &self.base
    }

    /// Returns the left snapshot.
    #[must_use]
    pub const fn left(&self) -> &VersionedContextGraphSnapshotV1 {
        &self.left
    }

    /// Returns the right snapshot.
    #[must_use]
    pub const fn right(&self) -> &VersionedContextGraphSnapshotV1 {
        &self.right
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionedContextGraphMergeReviewRequestWireV1 {
    schema_version: VersionedGraphMergeReviewSchemaVersion,
    plan: MergePlan,
    base: VersionedContextGraphSnapshotV1,
    left: VersionedContextGraphSnapshotV1,
    right: VersionedContextGraphSnapshotV1,
}

impl<'de> Deserialize<'de> for VersionedContextGraphMergeReviewRequestV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = VersionedContextGraphMergeReviewRequestWireV1::deserialize(deserializer)?;
        if wire.schema_version != VersionedGraphMergeReviewSchemaVersion::V1 {
            return Err(serde::de::Error::custom(
                VersionedContextGraphMergeReviewError::UnsupportedSchemaVersion {
                    schema_version: wire.schema_version,
                },
            ));
        }
        Ok(Self::new(wire.plan, wire.base, wire.left, wire.right))
    }
}

/// A complete, version-bound result of a three-way graph review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedContextGraphMergeReviewProjectionV1 {
    schema_version: VersionedGraphMergeReviewSchemaVersion,
    plan: MergePlan,
    base_scope: VersionedContextScopeV1,
    left_scope: VersionedContextScopeV1,
    right_scope: VersionedContextScopeV1,
    classification: GraphMergeClassification,
}

impl VersionedContextGraphMergeReviewProjectionV1 {
    /// Returns the explicit result schema version.
    #[must_use]
    pub const fn schema_version(&self) -> VersionedGraphMergeReviewSchemaVersion {
        self.schema_version
    }

    /// Returns the ancestry plan used by the review.
    #[must_use]
    pub const fn plan(&self) -> &MergePlan {
        &self.plan
    }

    /// Returns the exact base scope.
    #[must_use]
    pub const fn base_scope(&self) -> VersionedContextScopeV1 {
        self.base_scope
    }

    /// Returns the exact left scope.
    #[must_use]
    pub const fn left_scope(&self) -> VersionedContextScopeV1 {
        self.left_scope
    }

    /// Returns the exact right scope.
    #[must_use]
    pub const fn right_scope(&self) -> VersionedContextScopeV1 {
        self.right_scope
    }

    /// Returns the deterministic graph classification.
    #[must_use]
    pub const fn classification(&self) -> &GraphMergeClassification {
        &self.classification
    }
}

/// Fail-closed errors for a version-bound graph merge review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedContextGraphMergeReviewError {
    /// A serialized request claimed an unsupported schema version.
    UnsupportedSchemaVersion {
        /// The unsupported schema version.
        schema_version: VersionedGraphMergeReviewSchemaVersion,
    },
    /// One exact snapshot scope contains a nil identity.
    InvalidSnapshotScope {
        /// Snapshot side with the invalid identity.
        side: GraphMergeSnapshotSide,
    },
    /// Two of the three exact snapshots use the same commit identity.
    DuplicateCommitIdentity {
        /// Later snapshot side that repeats the commit.
        side: GraphMergeSnapshotSide,
        /// Repeated commit identity.
        commit_id: CommitId,
    },
    /// The graph classifier rejected the exact plan or snapshot scopes.
    ClassificationFailed {
        /// Structured classifier failure.
        source: GraphMergeClassificationError,
    },
}

impl fmt::Display for VersionedContextGraphMergeReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { schema_version } => {
                write!(
                    formatter,
                    "unsupported versioned graph merge review schema: {schema_version:?}"
                )
            }
            Self::InvalidSnapshotScope { side } => {
                write!(
                    formatter,
                    "versioned graph merge review has an invalid {side:?} scope"
                )
            }
            Self::DuplicateCommitIdentity { side, commit_id } => {
                write!(
                    formatter,
                    "versioned graph merge review repeats {side:?} commit {commit_id}"
                )
            }
            Self::ClassificationFailed { source } => {
                write!(
                    formatter,
                    "versioned Context Graph merge review failed: {source}"
                )
            }
        }
    }
}

impl std::error::Error for VersionedContextGraphMergeReviewError {}

/// Projects the existing graph classifier for exact version-bound snapshots.
#[derive(Debug, Default, Clone, Copy)]
pub struct VersionedContextGraphMergeReviewService;

impl VersionedContextGraphMergeReviewService {
    /// Produces one complete V1 projection without calculating a second graph diff.
    pub fn review(
        request: VersionedContextGraphMergeReviewRequestV1,
    ) -> Result<VersionedContextGraphMergeReviewProjectionV1, VersionedContextGraphMergeReviewError>
    {
        let base_scope = request.base.scope();
        let left_scope = request.left.scope();
        let right_scope = request.right.scope();
        for (side, scope) in [
            (GraphMergeSnapshotSide::Base, base_scope),
            (GraphMergeSnapshotSide::Left, left_scope),
            (GraphMergeSnapshotSide::Right, right_scope),
        ] {
            if scope.project_id().as_uuid().is_nil()
                || scope.context_id().as_uuid().is_nil()
                || scope.commit_id().as_uuid().is_nil()
            {
                return Err(VersionedContextGraphMergeReviewError::InvalidSnapshotScope { side });
            }
        }
        if base_scope.commit_id() == left_scope.commit_id() {
            return Err(
                VersionedContextGraphMergeReviewError::DuplicateCommitIdentity {
                    side: GraphMergeSnapshotSide::Left,
                    commit_id: left_scope.commit_id(),
                },
            );
        }
        if base_scope.commit_id() == right_scope.commit_id() {
            return Err(
                VersionedContextGraphMergeReviewError::DuplicateCommitIdentity {
                    side: GraphMergeSnapshotSide::Right,
                    commit_id: right_scope.commit_id(),
                },
            );
        }
        if left_scope.commit_id() == right_scope.commit_id() {
            return Err(
                VersionedContextGraphMergeReviewError::DuplicateCommitIdentity {
                    side: GraphMergeSnapshotSide::Right,
                    commit_id: right_scope.commit_id(),
                },
            );
        }
        let classification = GraphMergeConflictClassifier::classify(
            &request.plan,
            GraphSnapshotRef::new(
                base_scope.project_id(),
                base_scope.context_id(),
                base_scope.commit_id(),
                request.base.graph(),
            ),
            GraphSnapshotRef::new(
                left_scope.project_id(),
                left_scope.context_id(),
                left_scope.commit_id(),
                request.left.graph(),
            ),
            GraphSnapshotRef::new(
                right_scope.project_id(),
                right_scope.context_id(),
                right_scope.commit_id(),
                request.right.graph(),
            ),
        )
        .map_err(|source| VersionedContextGraphMergeReviewError::ClassificationFailed { source })?;

        Ok(VersionedContextGraphMergeReviewProjectionV1 {
            schema_version: VersionedGraphMergeReviewSchemaVersion::V1,
            plan: request.plan,
            base_scope,
            left_scope,
            right_scope,
            classification,
        })
    }
}
