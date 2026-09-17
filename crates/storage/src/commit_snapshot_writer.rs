//! Atomic Context commit and graph snapshot write contracts.

use crate::{
    CommitGraphSnapshot, CommitGraphSnapshotError, CommitGraphSnapshotScope, StorageRepositoryError,
};
use async_trait::async_trait;
use chrono::{DateTime, Timelike, Utc};
use contextlab_context_core::{ContextMetadata, ProjectId};
use contextlab_diff_engine::{
    BehaviorSnapshotV1, ContextDiffSnapshotV1, EvaluationSnapshotV1, SemanticSnapshotV1,
};
use contextlab_graph::ContextGraph;
use contextlab_versioning::ContextCommit;
use std::collections::HashSet;
use thiserror::Error;

/// Validated input for atomically persisting a Context commit and graph snapshot.
#[derive(Debug, Clone)]
pub struct CreateContextCommitSnapshot {
    commit: ContextCommit,
    snapshot: CommitGraphSnapshot,
    diff_snapshot: ContextDiffSnapshotV1,
    metadata: Option<ContextMetadata>,
}

impl CreateContextCommitSnapshot {
    /// Creates a command that binds one Context commit to its captured graph state.
    pub fn new(
        project_id: ProjectId,
        commit: ContextCommit,
        graph: ContextGraph,
        captured_at: DateTime<Utc>,
        schema_version: u16,
    ) -> Result<Self, CommitSnapshotWriteError> {
        Self::new_with_metadata(project_id, commit, graph, captured_at, schema_version, None)
    }

    /// Creates a command with the exact Context metadata at the captured commit.
    pub fn new_with_metadata(
        project_id: ProjectId,
        commit: ContextCommit,
        graph: ContextGraph,
        captured_at: DateTime<Utc>,
        schema_version: u16,
        metadata: Option<ContextMetadata>,
    ) -> Result<Self, CommitSnapshotWriteError> {
        let mut parent_ids = HashSet::new();
        for parent_id in commit.parent_ids() {
            if !parent_ids.insert(*parent_id) {
                return Err(CommitSnapshotWriteError::DuplicateParentCommitId {
                    parent_commit_id: parent_id.to_string(),
                });
            }
        }

        let semantic_snapshot = match metadata.as_ref() {
            Some(metadata) => {
                SemanticSnapshotV1::new_with_metadata(graph.clone(), Vec::new(), metadata.clone())?
            }
            None => SemanticSnapshotV1::new(graph.clone(), Vec::new())?,
        };
        let diff_snapshot = ContextDiffSnapshotV1::new(
            semantic_snapshot,
            BehaviorSnapshotV1::new(Vec::new())?,
            EvaluationSnapshotV1::new("context-commit:v1", Vec::new())?,
        )?;
        let snapshot = CommitGraphSnapshot::new(
            CommitGraphSnapshotScope::new(project_id, commit.context_id(), commit.id()),
            graph,
            postgres_timestamp_precision(captured_at),
            schema_version,
        )?;

        Ok(Self {
            commit,
            snapshot,
            diff_snapshot,
            metadata,
        })
    }

    /// Returns the immutable versioning commit to persist.
    #[must_use]
    pub const fn commit(&self) -> &ContextCommit {
        &self.commit
    }

    /// Returns the graph snapshot that must be persisted with the commit.
    #[must_use]
    pub const fn snapshot(&self) -> &CommitGraphSnapshot {
        &self.snapshot
    }

    /// Returns the complete diff input captured from the same immutable graph.
    #[must_use]
    pub const fn diff_snapshot(&self) -> &ContextDiffSnapshotV1 {
        &self.diff_snapshot
    }

    /// Returns the exact Context metadata captured for this commit, when present.
    #[must_use]
    pub const fn context_metadata(&self) -> Option<&ContextMetadata> {
        self.metadata.as_ref()
    }

    pub(crate) fn into_parts(self) -> (ContextCommit, CommitGraphSnapshot, ContextDiffSnapshotV1) {
        (self.commit, self.snapshot, self.diff_snapshot)
    }
}

fn postgres_timestamp_precision(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    timestamp
        .with_nanosecond((timestamp.timestamp_subsec_nanos() / 1_000) * 1_000)
        .expect("a valid UTC timestamp keeps its microsecond-truncated value")
}

/// Errors returned while validating an atomic commit snapshot command.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommitSnapshotWriteError {
    /// A commit repeated the same parent identifier.
    #[error("context commit repeats parent id: {parent_commit_id}")]
    DuplicateParentCommitId {
        /// Repeated parent identifier.
        parent_commit_id: String,
    },
    /// The snapshot could not be constructed from valid graph state.
    #[error(transparent)]
    Snapshot(#[from] CommitGraphSnapshotError),
    /// The derived V1 diff input could not be constructed.
    #[error(transparent)]
    DiffSnapshot(#[from] contextlab_diff_engine::DiffInputError),
}

/// Persists a Context commit, its ordered parents, and a graph snapshot atomically.
#[async_trait]
pub trait ContextCommitSnapshotWriter: Send + Sync {
    /// Creates one commit and exactly one immutable graph snapshot.
    async fn create_commit_snapshot(
        &self,
        command: CreateContextCommitSnapshot,
    ) -> Result<CommitGraphSnapshot, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contextlab_context_core::{ContextId, ProjectId};
    use contextlab_graph::ContextGraph;
    use contextlab_versioning::{BranchName, CommitId, ContextChange, ContextCommit};

    use super::{CommitSnapshotWriteError, CreateContextCommitSnapshot};

    fn commit(parent_ids: Vec<CommitId>) -> ContextCommit {
        ContextCommit::new(
            ContextId::new(),
            BranchName::default(),
            "Capture graph state",
            parent_ids,
            vec![ContextChange::created_context("Support agent")],
            Utc.with_ymd_and_hms(2026, 7, 11, 8, 0, 0)
                .single()
                .expect("timestamp"),
        )
        .expect("commit")
    }

    #[test]
    fn command_binds_a_commit_to_its_matching_snapshot() {
        let commit = commit(Vec::new());
        let command = CreateContextCommitSnapshot::new(
            ProjectId::new(),
            commit,
            ContextGraph::new(),
            Utc.with_ymd_and_hms(2026, 7, 11, 8, 1, 0)
                .single()
                .expect("timestamp"),
            1,
        )
        .expect("valid command");

        assert_eq!(command.snapshot().commit_id(), command.commit().id());
        assert_eq!(
            command.snapshot().context_id(),
            command.commit().context_id()
        );
    }

    #[test]
    fn command_normalizes_capture_time_to_postgres_microsecond_precision() {
        let captured_at = Utc
            .timestamp_opt(1_784_838_400, 123_456_789)
            .single()
            .expect("timestamp");
        let command = CreateContextCommitSnapshot::new(
            ProjectId::new(),
            commit(Vec::new()),
            ContextGraph::new(),
            captured_at,
            1,
        )
        .expect("valid command");

        assert_eq!(
            command.snapshot().captured_at().timestamp(),
            captured_at.timestamp()
        );
        assert_eq!(
            command.snapshot().captured_at().timestamp_subsec_nanos(),
            123_456_000
        );
    }

    #[test]
    fn command_rejects_duplicate_parent_ids_before_persistence() {
        let parent_id = CommitId::new();

        assert!(matches!(
            CreateContextCommitSnapshot::new(
                ProjectId::new(),
                commit(vec![parent_id, parent_id]),
                ContextGraph::new(),
                Utc.with_ymd_and_hms(2026, 7, 11, 8, 1, 0)
                    .single()
                    .expect("timestamp"),
                1,
            ),
            Err(CommitSnapshotWriteError::DuplicateParentCommitId { .. })
        ));
    }
}
