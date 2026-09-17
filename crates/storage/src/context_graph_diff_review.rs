//! Private storage-backed review over two immutable Context Graph snapshots.

use crate::{
    CommitGraphSnapshot, CommitGraphSnapshotScope, ContextLifecycleError,
    ContextLifecycleReadRepository, StorageRepositoryError,
};
use contextlab_diff_engine::{
    VersionedContextGraphDiffReviewError, VersionedContextGraphDiffReviewProjectionV1,
    VersionedContextGraphDiffReviewRequestV1, VersionedContextGraphDiffReviewService,
    VersionedContextScopeV1,
};
use thiserror::Error;

/// Side of a persisted two-way Context Graph review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistedContextGraphDiffReviewSide {
    /// The baseline snapshot.
    Source,
    /// The revised snapshot.
    Target,
}

/// Fail-closed errors from the private exact-snapshot graph review adapter.
#[derive(Debug, Error)]
pub enum PersistedContextGraphDiffReviewError {
    /// One exact scope contains a nil identity.
    #[error("persisted Context Graph review scope is invalid for {side:?}")]
    InvalidScope {
        /// Side with the invalid scope.
        side: PersistedContextGraphDiffReviewSide,
        /// Invalid exact storage scope.
        scope: CommitGraphSnapshotScope,
    },
    /// The two requested records do not belong to one project and Context.
    #[error("persisted Context Graph review scopes do not match")]
    MismatchedContextScope {
        /// Baseline exact scope.
        source_scope: CommitGraphSnapshotScope,
        /// Revised exact scope.
        target_scope: CommitGraphSnapshotScope,
    },
    /// A review cannot compare one exact commit with itself.
    #[error("persisted Context Graph review repeats exact scope {scope}")]
    IdenticalVersionScopes {
        /// Repeated exact scope.
        scope: CommitGraphSnapshotScope,
    },
    /// A required snapshot does not exist at its exact scope.
    #[error("persisted Context Graph review snapshot is missing for {side:?}: {scope}")]
    SnapshotMissing {
        /// Missing side.
        side: PersistedContextGraphDiffReviewSide,
        /// Exact missing scope.
        scope: CommitGraphSnapshotScope,
    },
    /// The snapshot repository could not be read.
    #[error("persisted Context Graph review snapshot read failed for {side:?}: {source}")]
    SnapshotRead {
        /// Side that could not be read.
        side: PersistedContextGraphDiffReviewSide,
        /// Exact requested scope.
        scope: CommitGraphSnapshotScope,
        /// Underlying repository failure.
        #[source]
        source: StorageRepositoryError,
    },
    /// The exact lifecycle witness could not be read or validated.
    #[error("persisted Context Graph review lifecycle witness failed for {side:?}: {source}")]
    LifecycleRead {
        /// Side whose lifecycle witness failed.
        side: PersistedContextGraphDiffReviewSide,
        /// Exact requested scope.
        scope: CommitGraphSnapshotScope,
        /// Underlying lifecycle failure.
        #[source]
        source: ContextLifecycleError,
    },
    /// No complete lifecycle witness exists at the exact requested scope.
    #[error("persisted Context Graph review lifecycle witness is missing for {side:?}: {scope}")]
    LifecycleMissing {
        /// Side whose lifecycle witness is missing.
        side: PersistedContextGraphDiffReviewSide,
        /// Exact requested scope.
        scope: CommitGraphSnapshotScope,
    },
    /// A repository returned a record outside the requested exact scope.
    #[error("persisted Context Graph review returned an out-of-scope {side:?} snapshot")]
    StoredScopeMismatch {
        /// Side returned by the repository.
        side: PersistedContextGraphDiffReviewSide,
        /// Exact requested scope.
        expected: CommitGraphSnapshotScope,
        /// Scope returned by storage.
        actual: CommitGraphSnapshotScope,
    },
    /// A repository returned an unsupported snapshot schema.
    #[error("persisted Context Graph review returned an unsupported {side:?} snapshot schema")]
    StoredSchemaMismatch {
        /// Side returned by the repository.
        side: PersistedContextGraphDiffReviewSide,
        /// Exact record scope.
        scope: CommitGraphSnapshotScope,
        /// Unsupported stored schema.
        actual: u16,
    },
    /// The reusable versioned graph contract rejected the complete pair.
    #[error("versioned Context Graph review failed: {source}")]
    ReviewFailed {
        /// Structured core error.
        #[source]
        source: VersionedContextGraphDiffReviewError,
    },
}

/// Complete private projection retaining immutable snapshot metadata for presentation adapters.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistedContextGraphDiffReviewProjection {
    source: CommitGraphSnapshot,
    target: CommitGraphSnapshot,
    review: VersionedContextGraphDiffReviewProjectionV1,
}

impl PersistedContextGraphDiffReviewProjection {
    /// Returns the baseline immutable snapshot.
    #[must_use]
    pub const fn source(&self) -> &CommitGraphSnapshot {
        &self.source
    }

    /// Returns the revised immutable snapshot.
    #[must_use]
    pub const fn target(&self) -> &CommitGraphSnapshot {
        &self.target
    }

    /// Returns the reusable versioned graph review projection.
    #[must_use]
    pub const fn review(&self) -> &VersionedContextGraphDiffReviewProjectionV1 {
        &self.review
    }
}

/// Private adapter that reads exact lifecycle witnesses and delegates graph comparison to core.
#[derive(Debug, Clone, Copy)]
pub struct PersistedContextGraphDiffReviewService<'repository, R: ?Sized> {
    repository: &'repository R,
}

impl<'repository, R> PersistedContextGraphDiffReviewService<'repository, R>
where
    R: ContextLifecycleReadRepository + ?Sized,
{
    /// Creates a review adapter over one snapshot repository.
    #[must_use]
    pub const fn new(repository: &'repository R) -> Self {
        Self { repository }
    }

    /// Reads and validates both exact lifecycle witnesses before comparison.
    pub async fn review(
        &self,
        source_scope: CommitGraphSnapshotScope,
        target_scope: CommitGraphSnapshotScope,
    ) -> Result<PersistedContextGraphDiffReviewProjection, PersistedContextGraphDiffReviewError>
    {
        validate_scope_pair(source_scope, target_scope)?;
        let source = self
            .read_snapshot(PersistedContextGraphDiffReviewSide::Source, source_scope)
            .await?;
        let target = self
            .read_snapshot(PersistedContextGraphDiffReviewSide::Target, target_scope)
            .await?;

        Self::project_snapshots(source, target)
    }

    /// Projects two already atomically-read snapshots through the versioned core.
    ///
    /// This method contains no storage reads. It is the only adapter entry point
    /// for a backend-owned multi-record witness.
    pub fn project_snapshots(
        source: CommitGraphSnapshot,
        target: CommitGraphSnapshot,
    ) -> Result<PersistedContextGraphDiffReviewProjection, PersistedContextGraphDiffReviewError>
    {
        let source_scope = source.scope();
        let target_scope = target.scope();
        validate_scope_pair(source_scope, target_scope)?;

        let request = VersionedContextGraphDiffReviewRequestV1::new(
            contextlab_diff_engine::VersionedContextGraphSnapshotV1::new(
                versioned_scope(source_scope),
                source.graph().clone(),
            ),
            contextlab_diff_engine::VersionedContextGraphSnapshotV1::new(
                versioned_scope(target_scope),
                target.graph().clone(),
            ),
        )
        .map_err(|source| PersistedContextGraphDiffReviewError::ReviewFailed { source })?;
        let review = VersionedContextGraphDiffReviewService::project(request);

        Ok(PersistedContextGraphDiffReviewProjection {
            source,
            target,
            review,
        })
    }

    async fn read_snapshot(
        &self,
        side: PersistedContextGraphDiffReviewSide,
        expected: CommitGraphSnapshotScope,
    ) -> Result<CommitGraphSnapshot, PersistedContextGraphDiffReviewError> {
        let facts = match self
            .repository
            .get_context_lifecycle_read_facts(expected.context_id(), expected.commit_id())
            .await
        {
            Ok(facts) => facts,
            Err(StorageRepositoryError::ScopeUnavailable { .. }) => {
                return Err(PersistedContextGraphDiffReviewError::LifecycleMissing {
                    side,
                    scope: expected,
                });
            }
            Err(source) => {
                return Err(PersistedContextGraphDiffReviewError::SnapshotRead {
                    side,
                    scope: expected,
                    source,
                });
            }
        };
        if facts.scope() != expected {
            return Err(PersistedContextGraphDiffReviewError::StoredScopeMismatch {
                side,
                expected,
                actual: facts.scope(),
            });
        }
        facts.validate_consistency().map_err(|source| {
            PersistedContextGraphDiffReviewError::LifecycleRead {
                side,
                scope: expected,
                source,
            }
        })?;
        Ok(facts.graph_snapshot().clone())
    }
}

pub(crate) fn validate_scope_pair(
    source_scope: CommitGraphSnapshotScope,
    target_scope: CommitGraphSnapshotScope,
) -> Result<(), PersistedContextGraphDiffReviewError> {
    validate_single_scope(PersistedContextGraphDiffReviewSide::Source, source_scope)?;
    validate_single_scope(PersistedContextGraphDiffReviewSide::Target, target_scope)?;
    VersionedContextGraphDiffReviewRequestV1::validate_scopes(
        versioned_scope(source_scope),
        versioned_scope(target_scope),
    )
    .map_err(|error| match error {
        VersionedContextGraphDiffReviewError::InvalidSnapshotScope { side, .. } => {
            PersistedContextGraphDiffReviewError::InvalidScope {
                side: match side {
                    contextlab_diff_engine::VersionedGraphDiffReviewSide::Source => {
                        PersistedContextGraphDiffReviewSide::Source
                    }
                    contextlab_diff_engine::VersionedGraphDiffReviewSide::Target => {
                        PersistedContextGraphDiffReviewSide::Target
                    }
                },
                scope: match side {
                    contextlab_diff_engine::VersionedGraphDiffReviewSide::Source => source_scope,
                    contextlab_diff_engine::VersionedGraphDiffReviewSide::Target => target_scope,
                },
            }
        }
        VersionedContextGraphDiffReviewError::MismatchedContextScope { .. } => {
            PersistedContextGraphDiffReviewError::MismatchedContextScope {
                source_scope,
                target_scope,
            }
        }
        VersionedContextGraphDiffReviewError::IdenticalVersionScopes { .. } => {
            PersistedContextGraphDiffReviewError::IdenticalVersionScopes {
                scope: source_scope,
            }
        }
        VersionedContextGraphDiffReviewError::UnsupportedSchemaVersion { .. } => {
            PersistedContextGraphDiffReviewError::ReviewFailed { source: error }
        }
    })
}

pub(crate) fn validate_single_scope(
    side: PersistedContextGraphDiffReviewSide,
    scope: CommitGraphSnapshotScope,
) -> Result<(), PersistedContextGraphDiffReviewError> {
    if scope.project_id().as_uuid().is_nil()
        || scope.context_id().as_uuid().is_nil()
        || scope.commit_id().as_uuid().is_nil()
    {
        return Err(PersistedContextGraphDiffReviewError::InvalidScope { side, scope });
    }
    Ok(())
}

fn versioned_scope(scope: CommitGraphSnapshotScope) -> VersionedContextScopeV1 {
    VersionedContextScopeV1::new(scope.project_id(), scope.context_id(), scope.commit_id())
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
    use contextlab_versioning::{BranchName, CommitId, ContextChange, ContextCommit, ReplayState};
    use std::collections::BTreeMap;

    fn scope(project: u128, context: u128, commit: u128) -> CommitGraphSnapshotScope {
        CommitGraphSnapshotScope::new(
            ProjectId::from_uuid(uuid::Uuid::from_u128(project)),
            ContextId::from_uuid(uuid::Uuid::from_u128(context)),
            CommitId::from_uuid(uuid::Uuid::from_u128(commit)),
        )
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
        CommitGraphSnapshot::new(
            scope,
            graph,
            Utc.timestamp_opt(1, 0).single().expect("timestamp"),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("snapshot")
    }

    fn repository() -> LifecycleFactsRepository {
        let context = ContextId::from_uuid(uuid::Uuid::from_u128(2));
        let root = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(3)),
            context,
            BranchName::default(),
            "Create Context",
            Vec::new(),
            vec![ContextChange::created_context("Context")],
            Utc.timestamp_opt(1, 0).single().expect("timestamp"),
        )
        .expect("root commit");
        let revised = ContextCommit::from_persisted(
            CommitId::from_uuid(uuid::Uuid::from_u128(4)),
            context,
            BranchName::default(),
            "Update Context metadata",
            vec![root.id()],
            vec![ContextChange::updated_metadata(
                contextlab_context_core::ContextMetadata::new(
                    Utc.timestamp_opt(2, 0).single().expect("timestamp"),
                ),
                "Update Context metadata",
            )],
            Utc.timestamp_opt(2, 0).single().expect("timestamp"),
        )
        .expect("revised commit");
        let project = ProjectId::from_uuid(uuid::Uuid::from_u128(1));
        let source_scope = CommitGraphSnapshotScope::new(project, context, root.id());
        let target_scope = CommitGraphSnapshotScope::new(project, context, revised.id());
        let source_state =
            ReplayState::from_commits(context, std::slice::from_ref(&root)).expect("source replay");
        let target_state =
            ReplayState::from_commits(context, &[root, revised]).expect("target replay");
        LifecycleFactsRepository {
            facts: BTreeMap::from([
                (
                    source_scope.commit_id().to_string(),
                    ContextLifecycleReadFacts::from_parts(
                        source_scope,
                        ContextComponentStateSnapshotAtCommit::new(
                            context,
                            source_scope.commit_id(),
                            Vec::new(),
                        ),
                        Vec::new(),
                        source_state,
                        snapshot(source_scope, "v1"),
                    )
                    .expect("source facts"),
                ),
                (
                    target_scope.commit_id().to_string(),
                    ContextLifecycleReadFacts::from_parts(
                        target_scope,
                        ContextComponentStateSnapshotAtCommit::new(
                            context,
                            target_scope.commit_id(),
                            Vec::new(),
                        ),
                        Vec::new(),
                        target_state,
                        snapshot(target_scope, "v2"),
                    )
                    .expect("target facts"),
                ),
            ]),
        }
    }

    struct LifecycleFactsRepository {
        facts: BTreeMap<String, ContextLifecycleReadFacts>,
    }

    #[async_trait]
    impl ContextLifecycleReadRepository for LifecycleFactsRepository {
        async fn get_context_lifecycle_read_facts(
            &self,
            context_id: ContextId,
            commit_id: CommitId,
        ) -> Result<ContextLifecycleReadFacts, StorageRepositoryError> {
            self.facts
                .get(&commit_id.to_string())
                .cloned()
                .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                    scope: format!("context:{context_id}/commit:{commit_id}"),
                })
        }
    }

    #[tokio::test]
    async fn reads_exact_pair_and_preserves_snapshot_metadata() {
        let source_scope = scope(1, 2, 3);
        let target_scope = scope(1, 2, 4);
        let projection = PersistedContextGraphDiffReviewService::new(&repository())
            .review(source_scope, target_scope)
            .await
            .expect("review");

        assert_eq!(projection.source().scope(), source_scope);
        assert_eq!(projection.target().scope(), target_scope);
        assert_eq!(
            projection.review().source_scope(),
            versioned_scope(source_scope)
        );
        assert_eq!(projection.review().diff().modified_nodes().len(), 1);
    }

    #[tokio::test]
    async fn rejects_invalid_pairs_before_snapshot_reads() {
        let repository = repository();
        let source = scope(1, 2, 3);
        assert!(matches!(
            PersistedContextGraphDiffReviewService::new(&repository)
                .review(source, source)
                .await,
            Err(PersistedContextGraphDiffReviewError::IdenticalVersionScopes { .. })
        ));
        assert!(matches!(
            PersistedContextGraphDiffReviewService::new(&repository)
                .review(source, scope(9, 2, 4))
                .await,
            Err(PersistedContextGraphDiffReviewError::MismatchedContextScope { .. })
        ));
    }

    #[tokio::test]
    async fn rejects_missing_exact_snapshot() {
        let repository = repository();
        let error = PersistedContextGraphDiffReviewService::new(&repository)
            .review(scope(1, 2, 3), scope(1, 2, 9))
            .await
            .expect_err("missing snapshot");
        assert!(matches!(
            error,
            PersistedContextGraphDiffReviewError::LifecycleMissing {
                side: PersistedContextGraphDiffReviewSide::Target,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn rejects_mixed_lifecycle_witness_scope_before_graph_diff() {
        let mut repository = repository();
        let target_scope = scope(1, 2, 4);
        let drifted_scope = scope(9, 2, 4);
        let drifted_facts = ContextLifecycleReadFacts::from_parts(
            drifted_scope,
            ContextComponentStateSnapshotAtCommit::new(
                drifted_scope.context_id(),
                drifted_scope.commit_id(),
                Vec::new(),
            ),
            Vec::new(),
            ReplayState::from_commits(
                drifted_scope.context_id(),
                &[ContextCommit::from_persisted(
                    drifted_scope.commit_id(),
                    drifted_scope.context_id(),
                    BranchName::default(),
                    "Drifted Context",
                    Vec::new(),
                    vec![ContextChange::created_context("Context")],
                    Utc.timestamp_opt(3, 0).single().expect("timestamp"),
                )
                .expect("drifted commit")],
            )
            .expect("drifted replay"),
            snapshot(drifted_scope, "drifted"),
        )
        .expect("drifted facts");
        repository
            .facts
            .insert(target_scope.commit_id().to_string(), drifted_facts);

        let error = PersistedContextGraphDiffReviewService::new(&repository)
            .review(scope(1, 2, 3), target_scope)
            .await
            .expect_err("mixed lifecycle witness");
        assert!(matches!(
            error,
            PersistedContextGraphDiffReviewError::StoredScopeMismatch {
                side: PersistedContextGraphDiffReviewSide::Target,
                expected,
                actual,
            } if expected == target_scope && actual == drifted_scope
        ));
    }
}
