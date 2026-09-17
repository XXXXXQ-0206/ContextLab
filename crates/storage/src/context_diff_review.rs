//! Private storage-backed review over two immutable exact-commit snapshots.

use crate::context_diff_snapshot::ContextDiffSnapshotV1PairRepository;
use crate::{
    CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1, ContextDiffSnapshotPersistenceError,
    ContextDiffSnapshotV1Record,
};
use contextlab_diff_engine::{
    VersionedContextDiffReviewError, VersionedContextDiffReviewProjectionV1,
    VersionedContextDiffReviewRequestV1, VersionedContextDiffReviewService,
    VersionedContextScopeV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The side of a persisted snapshot read used by a version comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedContextDiffReviewSide {
    /// The baseline snapshot.
    Source,
    /// The revised snapshot.
    Target,
}

/// Fail-closed errors from storage-backed version comparison.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PersistedContextDiffReviewError {
    /// One requested exact scope contains an unusable identifier.
    #[error("persisted Context diff review scope is invalid for {side:?}: {reason}")]
    InvalidScope {
        /// The side with the invalid scope.
        side: PersistedContextDiffReviewSide,
        /// The rejected exact scope.
        scope: VersionedContextScopeV1,
        /// Stable validation reason.
        reason: &'static str,
    },
    /// The two exact scopes do not identify one Context.
    #[error(
        "persisted Context diff review scopes do not match: {source_scope:?} vs {target_scope:?}"
    )]
    MismatchedContextScope {
        /// The baseline scope.
        source_scope: VersionedContextScopeV1,
        /// The revised scope.
        target_scope: VersionedContextScopeV1,
    },
    /// A review cannot compare one exact commit with itself.
    #[error("persisted Context diff review repeats exact scope {scope:?}")]
    IdenticalVersionScopes {
        /// The repeated exact scope.
        scope: VersionedContextScopeV1,
    },
    /// One required source was unavailable in the repository.
    #[error("persisted Context diff review source {side:?} is unavailable: {source}")]
    SnapshotRead {
        /// The side that could not be read.
        side: PersistedContextDiffReviewSide,
        /// The exact scope that was requested.
        scope: VersionedContextScopeV1,
        /// The repository failure, without connection details.
        #[source]
        source: ContextDiffSnapshotPersistenceError,
    },
    /// The repository could not load the requested pair through one boundary.
    #[error("persisted Context diff review snapshot pair read failed")]
    SnapshotPairRead {
        /// The requested baseline scope.
        source_scope: VersionedContextScopeV1,
        /// The requested revised scope.
        target_scope: VersionedContextScopeV1,
        /// The repository failure, without connection details.
        #[source]
        source: Box<ContextDiffSnapshotPersistenceError>,
    },
    /// A repository returned a record outside the requested exact scope.
    #[error("persisted Context diff review returned an out-of-scope {side:?} record")]
    StoredScopeMismatch {
        /// The side returned by the repository.
        side: PersistedContextDiffReviewSide,
        /// The requested exact scope.
        expected: VersionedContextScopeV1,
        /// The scope found in storage.
        actual: VersionedContextScopeV1,
    },
    /// A repository returned a record with an unsupported snapshot schema.
    #[error("persisted Context diff review returned an unsupported snapshot schema")]
    StoredSchemaMismatch {
        /// The side returned by the repository.
        side: PersistedContextDiffReviewSide,
        /// The exact scope carried by the record.
        scope: VersionedContextScopeV1,
        /// The schema found in storage.
        actual: String,
    },
    /// The existing unified diff service rejected the complete pair.
    #[error("versioned Context diff review failed: {source}")]
    ReviewFailed {
        /// The structured unified diff error.
        #[source]
        source: VersionedContextDiffReviewError,
    },
}

/// Private adapter that turns immutable exact-commit storage into one unified review.
#[derive(Debug, Clone)]
pub struct PersistedContextDiffReviewService<'repository, R: ?Sized> {
    repository: &'repository R,
}

impl<'repository, R> PersistedContextDiffReviewService<'repository, R>
where
    R: ContextDiffSnapshotV1PairRepository + ?Sized,
{
    /// Creates a review service over one storage repository.
    #[must_use]
    pub const fn new(repository: &'repository R) -> Self {
        Self { repository }
    }

    /// Reads both exact snapshots and delegates comparison to the unified diff service.
    pub async fn review(
        &self,
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Result<VersionedContextDiffReviewProjectionV1, PersistedContextDiffReviewError> {
        validate_scope_pair(source_scope, target_scope)?;
        let pair = self
            .repository
            .read_context_diff_snapshot_pair(source_scope, target_scope)
            .await
            .map_err(|source| PersistedContextDiffReviewError::SnapshotPairRead {
                source_scope,
                target_scope,
                source: Box::new(source),
            })?;
        let source = validate_snapshot_record(
            PersistedContextDiffReviewSide::Source,
            source_scope,
            pair.source().clone(),
        )?;
        let target = validate_snapshot_record(
            PersistedContextDiffReviewSide::Target,
            target_scope,
            pair.target().clone(),
        )?;
        let review_request = VersionedContextDiffReviewRequestV1::new(
            source_scope,
            target_scope,
            source.snapshot().clone(),
            target.snapshot().clone(),
        )
        .map_err(|source| PersistedContextDiffReviewError::ReviewFailed { source })?;
        VersionedContextDiffReviewService::project(review_request)
            .map_err(|source| PersistedContextDiffReviewError::ReviewFailed { source })
    }
}

fn validate_snapshot_record(
    side: PersistedContextDiffReviewSide,
    expected: VersionedContextScopeV1,
    record: ContextDiffSnapshotV1Record,
) -> Result<ContextDiffSnapshotV1Record, PersistedContextDiffReviewError> {
    if record.scope() != expected {
        return Err(PersistedContextDiffReviewError::StoredScopeMismatch {
            side,
            expected,
            actual: record.scope(),
        });
    }
    if record.schema_version() != CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1 {
        return Err(PersistedContextDiffReviewError::StoredSchemaMismatch {
            side,
            scope: expected,
            actual: record.schema_version().to_owned(),
        });
    }
    Ok(record)
}

fn validate_scope_pair(
    source_scope: VersionedContextScopeV1,
    target_scope: VersionedContextScopeV1,
) -> Result<(), PersistedContextDiffReviewError> {
    validate_scope(PersistedContextDiffReviewSide::Source, source_scope)?;
    validate_scope(PersistedContextDiffReviewSide::Target, target_scope)?;
    if source_scope.project_id() != target_scope.project_id()
        || source_scope.context_id() != target_scope.context_id()
    {
        return Err(PersistedContextDiffReviewError::MismatchedContextScope {
            source_scope,
            target_scope,
        });
    }
    if source_scope == target_scope {
        return Err(PersistedContextDiffReviewError::IdenticalVersionScopes {
            scope: source_scope,
        });
    }
    Ok(())
}

fn validate_scope(
    side: PersistedContextDiffReviewSide,
    scope: VersionedContextScopeV1,
) -> Result<(), PersistedContextDiffReviewError> {
    let reason = if scope.project_id().as_uuid().is_nil() {
        Some("project identifier is nil")
    } else if scope.context_id().as_uuid().is_nil() {
        Some("Context identifier is nil")
    } else if scope.commit_id().as_uuid().is_nil() {
        Some("commit identifier is nil")
    } else {
        None
    };

    reason.map_or(Ok(()), |reason| {
        Err(PersistedContextDiffReviewError::InvalidScope {
            side,
            scope,
            reason,
        })
    })
}

/// Compatibility name for callers that prefer the adapter terminology.
pub type PersistedContextDiffReviewAdapter<'repository, R> =
    PersistedContextDiffReviewService<'repository, R>;
