//! Version-bound, local-only review projection over the unified diff contract.

use crate::{
    ContextDiffError, ContextDiffRequestV1, ContextDiffResultV1, ContextDiffService,
    ContextDiffSnapshotV1, DiffInputError,
};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_versioning::CommitId;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// The explicit schema revision for a version-bound diff review projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffReviewProjectionSchemaVersion {
    /// The initial stable version-bound review schema.
    V1,
}

/// The exact project/Context/commit identity of one private diff input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedContextScopeV1 {
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
}

impl VersionedContextScopeV1 {
    /// Creates an exact project/Context/commit review scope.
    #[must_use]
    pub const fn new(project_id: ProjectId, context_id: ContextId, commit_id: CommitId) -> Self {
        Self {
            project_id,
            context_id,
            commit_id,
        }
    }

    /// Returns the project identity.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the Context identity.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the exact commit identity.
    #[must_use]
    pub const fn commit_id(self) -> CommitId {
        self.commit_id
    }
}

/// Complete diff inputs bound to one exact ordered pair of Context versions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VersionedContextDiffReviewRequestV1 {
    schema_version: DiffReviewProjectionSchemaVersion,
    source_scope: VersionedContextScopeV1,
    target_scope: VersionedContextScopeV1,
    source: ContextDiffSnapshotV1,
    target: ContextDiffSnapshotV1,
}

impl VersionedContextDiffReviewRequestV1 {
    /// Creates a review request for two distinct exact Context versions.
    pub fn new(
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
        source: ContextDiffSnapshotV1,
        target: ContextDiffSnapshotV1,
    ) -> Result<Self, VersionedContextDiffReviewError> {
        if source_scope.project_id() != target_scope.project_id()
            || source_scope.context_id() != target_scope.context_id()
        {
            return Err(VersionedContextDiffReviewError::MismatchedContextScope {
                source_scope,
                target_scope,
            });
        }
        if source_scope == target_scope {
            return Err(VersionedContextDiffReviewError::IdenticalVersionScopes {
                scope: source_scope,
            });
        }

        Ok(Self {
            schema_version: DiffReviewProjectionSchemaVersion::V1,
            source_scope,
            target_scope,
            source,
            target,
        })
    }

    /// Returns the explicit request schema version.
    #[must_use]
    pub const fn schema_version(&self) -> DiffReviewProjectionSchemaVersion {
        self.schema_version
    }

    /// Returns the exact baseline version identifier.
    #[must_use]
    pub const fn source_version_id(&self) -> CommitId {
        self.source_scope.commit_id()
    }

    /// Returns the exact revised version identifier.
    #[must_use]
    pub const fn target_version_id(&self) -> CommitId {
        self.target_scope.commit_id()
    }

    /// Returns the exact baseline project/Context/commit scope.
    #[must_use]
    pub const fn source_scope(&self) -> VersionedContextScopeV1 {
        self.source_scope
    }

    /// Returns the exact revised project/Context/commit scope.
    #[must_use]
    pub const fn target_scope(&self) -> VersionedContextScopeV1 {
        self.target_scope
    }

    /// Returns the baseline complete diff snapshot.
    #[must_use]
    pub const fn source(&self) -> &ContextDiffSnapshotV1 {
        &self.source
    }

    /// Returns the revised complete diff snapshot.
    #[must_use]
    pub const fn target(&self) -> &ContextDiffSnapshotV1 {
        &self.target
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionedContextDiffReviewRequestWireV1 {
    schema_version: DiffReviewProjectionSchemaVersion,
    source_scope: VersionedContextScopeV1,
    target_scope: VersionedContextScopeV1,
    source: ContextDiffSnapshotV1,
    target: ContextDiffSnapshotV1,
}

impl TryFrom<VersionedContextDiffReviewRequestWireV1> for VersionedContextDiffReviewRequestV1 {
    type Error = VersionedContextDiffReviewError;

    fn try_from(value: VersionedContextDiffReviewRequestWireV1) -> Result<Self, Self::Error> {
        let request = Self::new(
            value.source_scope,
            value.target_scope,
            value.source,
            value.target,
        )?;

        if value.schema_version != request.schema_version {
            return Err(VersionedContextDiffReviewError::UnsupportedSchemaVersion {
                schema_version: value.schema_version,
            });
        }

        Ok(request)
    }
}

impl<'de> Deserialize<'de> for VersionedContextDiffReviewRequestV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        VersionedContextDiffReviewRequestWireV1::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

/// Provider-free, local-only structured review of one ordered Context version pair.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VersionedContextDiffReviewProjectionV1 {
    schema_version: DiffReviewProjectionSchemaVersion,
    source_scope: VersionedContextScopeV1,
    target_scope: VersionedContextScopeV1,
    diff: ContextDiffResultV1,
}

impl VersionedContextDiffReviewProjectionV1 {
    fn new(
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
        diff: ContextDiffResultV1,
    ) -> Self {
        Self {
            schema_version: DiffReviewProjectionSchemaVersion::V1,
            source_scope,
            target_scope,
            diff,
        }
    }

    /// Returns the explicit projection schema version.
    #[must_use]
    pub const fn schema_version(&self) -> DiffReviewProjectionSchemaVersion {
        self.schema_version
    }

    /// Returns the exact baseline version identifier.
    #[must_use]
    pub const fn source_version_id(&self) -> CommitId {
        self.source_scope.commit_id()
    }

    /// Returns the exact revised version identifier.
    #[must_use]
    pub const fn target_version_id(&self) -> CommitId {
        self.target_scope.commit_id()
    }

    /// Returns the exact baseline project/Context/commit scope.
    #[must_use]
    pub const fn source_scope(&self) -> VersionedContextScopeV1 {
        self.source_scope
    }

    /// Returns the exact revised project/Context/commit scope.
    #[must_use]
    pub const fn target_scope(&self) -> VersionedContextScopeV1 {
        self.target_scope
    }

    /// Returns the complete existing semantic, behavior, and evaluation diff.
    #[must_use]
    pub const fn diff(&self) -> &ContextDiffResultV1 {
        &self.diff
    }
}

/// An error that prevents a complete trustworthy version-bound review projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedContextDiffReviewError {
    /// A review cannot compare one exact scope with itself.
    IdenticalVersionScopes {
        /// The repeated exact project/Context/commit scope.
        scope: VersionedContextScopeV1,
    },
    /// A review cannot compare snapshots from different projects or Contexts.
    MismatchedContextScope {
        /// The baseline project/Context/commit scope.
        source_scope: VersionedContextScopeV1,
        /// The revised project/Context/commit scope.
        target_scope: VersionedContextScopeV1,
    },
    /// A serialized request claimed an unsupported schema version.
    UnsupportedSchemaVersion {
        /// The unsupported request schema version.
        schema_version: DiffReviewProjectionSchemaVersion,
    },
    /// One supplied snapshot violated the unified diff input contract.
    InvalidDiffInput {
        /// The exact baseline version identifier.
        source_version_id: CommitId,
        /// The exact revised version identifier.
        target_version_id: CommitId,
        /// The invalid structural input.
        source: DiffInputError,
    },
    /// The existing unified diff engine rejected the complete comparison.
    ContextDiffFailed {
        /// The exact baseline version identifier.
        source_version_id: CommitId,
        /// The exact revised version identifier.
        target_version_id: CommitId,
        /// The structured unified diff error.
        source: ContextDiffError,
    },
}

impl fmt::Display for VersionedContextDiffReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "versioned context diff review failed: {self:?}")
    }
}

impl std::error::Error for VersionedContextDiffReviewError {}

/// Projects an existing unified Context diff for an exact ordered version pair.
#[derive(Debug, Default, Clone, Copy)]
pub struct VersionedContextDiffReviewService;

impl VersionedContextDiffReviewService {
    /// Produces one complete review projection or returns a structured error with no partial result.
    pub fn project(
        request: VersionedContextDiffReviewRequestV1,
    ) -> Result<VersionedContextDiffReviewProjectionV1, VersionedContextDiffReviewError> {
        let source_scope = request.source_scope;
        let target_scope = request.target_scope;
        let diff_request =
            ContextDiffRequestV1::new(request.source, request.target).map_err(|source| {
                VersionedContextDiffReviewError::InvalidDiffInput {
                    source_version_id: source_scope.commit_id(),
                    target_version_id: target_scope.commit_id(),
                    source,
                }
            })?;
        let diff = ContextDiffService::compare(diff_request).map_err(|source| {
            VersionedContextDiffReviewError::ContextDiffFailed {
                source_version_id: source_scope.commit_id(),
                target_version_id: target_scope.commit_id(),
                source,
            }
        })?;

        Ok(VersionedContextDiffReviewProjectionV1::new(
            source_scope,
            target_scope,
            diff,
        ))
    }
}
