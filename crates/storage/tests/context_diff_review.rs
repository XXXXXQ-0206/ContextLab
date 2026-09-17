//! Focused tests for the private persisted Context diff review adapter.

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ContextMetadata, ProjectId};
use contextlab_diff_engine::{
    BehaviorObservationV1, BehaviorOutcomeV1, BehaviorSnapshotV1, ContextDiffSnapshotV1,
    ContextMetadataChangeV1, EvaluationMetricObservationV1, EvaluationSnapshotV1,
    SemanticDocumentV1, SemanticSnapshotV1, VersionedContextScopeV1,
};
use contextlab_graph::ContextGraph;
use contextlab_storage::{
    ContextDiffSnapshotPersistenceError, ContextDiffSnapshotV1Pair,
    ContextDiffSnapshotV1PairRepository, ContextDiffSnapshotV1Repository,
    ContextDiffSnapshotWriteResult, InMemoryContextDiffSnapshotV1Repository,
    PersistContextDiffSnapshotV1, PersistedContextDiffReviewAdapter,
    PersistedContextDiffReviewError, PersistedContextDiffReviewSide,
};
use contextlab_versioning::CommitId;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

fn scope(seed: u128) -> VersionedContextScopeV1 {
    VersionedContextScopeV1::new(
        ProjectId::from_uuid(Uuid::from_u128(seed)),
        ContextId::from_uuid(Uuid::from_u128(seed + 1)),
        CommitId::from_uuid(Uuid::from_u128(seed + 2)),
    )
}

fn snapshot(fingerprint: &str) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(ContextGraph::new(), Vec::new()).expect("semantic snapshot"),
        BehaviorSnapshotV1::new(Vec::new()).expect("behavior snapshot"),
        EvaluationSnapshotV1::new(fingerprint, Vec::new()).expect("evaluation snapshot"),
    )
    .expect("diff snapshot")
}

fn changed_snapshot(document: &str, output: &str, accuracy: f64) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(
            ContextGraph::new(),
            vec![SemanticDocumentV1::new("prompt:system", document).expect("document")],
        )
        .expect("semantic snapshot"),
        BehaviorSnapshotV1::new(vec![
            BehaviorObservationV1::new(
                "case:one",
                "sha256:input",
                BehaviorOutcomeV1::succeeded(output),
            )
            .expect("behavior observation"),
        ])
        .expect("behavior snapshot"),
        EvaluationSnapshotV1::new(
            "suite:local:v1",
            vec![
                EvaluationMetricObservationV1::new("accuracy", accuracy, 1)
                    .expect("evaluation metric"),
            ],
        )
        .expect("evaluation snapshot"),
    )
    .expect("complete diff snapshot")
}

fn metadata_snapshot(metadata: ContextMetadata) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new_with_metadata(ContextGraph::new(), Vec::new(), metadata)
            .expect("semantic snapshot"),
        BehaviorSnapshotV1::new(Vec::new()).expect("behavior snapshot"),
        EvaluationSnapshotV1::new("suite:local:v1", Vec::new()).expect("evaluation snapshot"),
    )
    .expect("metadata diff snapshot")
}

fn persist_command(
    exact_scope: VersionedContextScopeV1,
    fingerprint: &str,
) -> PersistContextDiffSnapshotV1 {
    PersistContextDiffSnapshotV1::new(
        exact_scope,
        "context-diff-snapshot-v1",
        snapshot(fingerprint),
        Utc.timestamp_opt(1_753_680_000, 123_456_000)
            .single()
            .expect("capture timestamp"),
    )
    .expect("persist command")
}

#[tokio::test]
async fn reads_two_exact_persisted_records_and_delegates_to_versioned_review() {
    let source_scope = scope(100);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(103)),
    );
    let repository = InMemoryContextDiffSnapshotV1Repository::new();
    repository
        .persist_context_diff_snapshot(persist_command(source_scope, "same-suite"))
        .await
        .expect("source snapshot");
    repository
        .persist_context_diff_snapshot(persist_command(target_scope, "same-suite"))
        .await
        .expect("target snapshot");

    let review = PersistedContextDiffReviewAdapter::new(&repository)
        .review(source_scope, target_scope)
        .await
        .expect("persisted review");

    assert_eq!(review.source_scope(), source_scope);
    assert_eq!(review.target_scope(), target_scope);
    assert!(review.diff().semantic().graph_diff().is_empty());
}

#[tokio::test]
async fn delegates_the_complete_semantic_behavior_and_evaluation_pair() {
    let source_scope = scope(150);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(153)),
    );
    let repository = InMemoryContextDiffSnapshotV1Repository::new();
    repository
        .persist_context_diff_snapshot(
            PersistContextDiffSnapshotV1::new(
                source_scope,
                "context-diff-snapshot-v1",
                changed_snapshot("Answer briefly.", "old", 0.80),
                Utc.timestamp_opt(1_753_680_000, 123_456_000)
                    .single()
                    .expect("capture timestamp"),
            )
            .expect("source snapshot"),
        )
        .await
        .expect("persist source");
    repository
        .persist_context_diff_snapshot(
            PersistContextDiffSnapshotV1::new(
                target_scope,
                "context-diff-snapshot-v1",
                changed_snapshot("Answer with citations.", "new", 0.90),
                Utc.timestamp_opt(1_753_680_001, 123_456_000)
                    .single()
                    .expect("capture timestamp"),
            )
            .expect("target snapshot"),
        )
        .await
        .expect("persist target");

    let review = PersistedContextDiffReviewAdapter::new(&repository)
        .review(source_scope, target_scope)
        .await
        .expect("complete persisted review");

    assert_eq!(review.diff().semantic().document_changes().len(), 1);
    assert_eq!(review.diff().behavior().case_changes().len(), 1);
    assert_eq!(review.diff().evaluation().metric_changes().len(), 1);
}

#[tokio::test]
async fn replays_metadata_snapshots_through_the_versioned_review_adapter() {
    let source_scope = scope(175);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(178)),
    );
    let created_at = Utc
        .timestamp_opt(1_753_680_000, 0)
        .single()
        .expect("metadata timestamp");
    let revised_at = Utc
        .timestamp_opt(1_753_680_001, 0)
        .single()
        .expect("metadata timestamp");
    let mut source_metadata = ContextMetadata::new(created_at);
    source_metadata.set_label("owner", "support", created_at);
    let mut target_metadata = source_metadata.clone();
    target_metadata.set_label("owner", "platform", revised_at);
    let repository = InMemoryContextDiffSnapshotV1Repository::new();

    for (scope, snapshot, captured_at) in [
        (source_scope, metadata_snapshot(source_metadata), created_at),
        (target_scope, metadata_snapshot(target_metadata), revised_at),
    ] {
        repository
            .persist_context_diff_snapshot(
                PersistContextDiffSnapshotV1::new(
                    scope,
                    "context-diff-snapshot-v1",
                    snapshot,
                    captured_at,
                )
                .expect("persist metadata snapshot"),
            )
            .await
            .expect("persist metadata snapshot");
    }

    let review = PersistedContextDiffReviewAdapter::new(&repository)
        .review(source_scope, target_scope)
        .await
        .expect("versioned metadata review");

    assert!(matches!(
        review.diff().semantic().metadata_change(),
        Some(ContextMetadataChangeV1::Modified { original, revised })
            if original.labels().get("owner").map(String::as_str) == Some("support")
                && revised.labels().get("owner").map(String::as_str) == Some("platform")
    ));
}

#[tokio::test]
async fn replays_added_and_removed_metadata_for_exact_commit_pairs() {
    let created_at = Utc
        .timestamp_opt(1_753_680_000, 0)
        .single()
        .expect("metadata timestamp");
    let revised_at = Utc
        .timestamp_opt(1_753_680_001, 0)
        .single()
        .expect("metadata timestamp");
    let mut added_metadata = ContextMetadata::new(created_at);
    added_metadata.set_label("owner", "platform", revised_at);
    let mut removed_metadata = ContextMetadata::new(created_at);
    removed_metadata.set_label("owner", "support", created_at);

    for (seed, source_snapshot, target_snapshot, expected_change) in [
        (
            225,
            snapshot("suite:local:v1"),
            metadata_snapshot(added_metadata),
            "added",
        ),
        (
            275,
            metadata_snapshot(removed_metadata),
            snapshot("suite:local:v1"),
            "removed",
        ),
    ] {
        let source_scope = scope(seed);
        let target_scope = VersionedContextScopeV1::new(
            source_scope.project_id(),
            source_scope.context_id(),
            CommitId::from_uuid(Uuid::from_u128(seed + 3)),
        );
        let repository = InMemoryContextDiffSnapshotV1Repository::new();

        for (exact_scope, snapshot, captured_at) in [
            (source_scope, source_snapshot, created_at),
            (target_scope, target_snapshot, revised_at),
        ] {
            repository
                .persist_context_diff_snapshot(
                    PersistContextDiffSnapshotV1::new(
                        exact_scope,
                        "context-diff-snapshot-v1",
                        snapshot,
                        captured_at,
                    )
                    .expect("persist exact metadata snapshot"),
                )
                .await
                .expect("persist exact metadata snapshot");
        }

        let review = PersistedContextDiffReviewAdapter::new(&repository)
            .review(source_scope, target_scope)
            .await
            .expect("replay exact metadata pair");

        match (expected_change, review.diff().semantic().metadata_change()) {
            ("added", Some(ContextMetadataChangeV1::Added { revised })) => {
                assert_eq!(
                    revised.labels().get("owner").map(String::as_str),
                    Some("platform")
                );
            }
            ("removed", Some(ContextMetadataChangeV1::Removed { original })) => {
                assert_eq!(
                    original.labels().get("owner").map(String::as_str),
                    Some("support")
                );
            }
            (expected, actual) => panic!("expected {expected} metadata change, got {actual:?}"),
        }
    }
}

#[tokio::test]
async fn rejects_invalid_pair_before_reading_persisted_state() {
    let source_scope = scope(200);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        source_scope.commit_id(),
    );
    let reads = Arc::new(Mutex::new(Vec::new()));
    let repository = RecordingRepository::new(reads.clone());

    let error = PersistedContextDiffReviewAdapter::new(&repository)
        .review(source_scope, target_scope)
        .await
        .expect_err("identical versions must fail closed");

    assert!(matches!(
        error,
        PersistedContextDiffReviewError::IdenticalVersionScopes { scope } if scope == source_scope
    ));
    assert!(reads.lock().expect("read log").is_empty());
}

#[tokio::test]
async fn review_reads_one_pair_and_never_falls_back_to_individual_reads() {
    let source_scope = scope(2750);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(2753)),
    );
    let reads = Arc::new(Mutex::new(Vec::new()));
    let pair_reads = Arc::new(Mutex::new(0_u32));
    let repository = RecordingRepository::new_with_pair_reads(reads.clone(), pair_reads.clone());

    let error = PersistedContextDiffReviewAdapter::new(&repository)
        .review(source_scope, target_scope)
        .await
        .expect_err("recording repository intentionally has no snapshots");

    assert!(matches!(
        error,
        PersistedContextDiffReviewError::SnapshotPairRead {
            source_scope: actual_source,
            target_scope: actual_target,
            source,
        } if actual_source == source_scope
            && actual_target == target_scope
            && matches!(
                source.as_ref(),
                ContextDiffSnapshotPersistenceError::NotFound { scope } if *scope == source_scope
            )
    ));
    assert_eq!(*pair_reads.lock().expect("pair read log"), 1);
    assert!(reads.lock().expect("single read log").is_empty());
}

#[tokio::test]
async fn rejects_nil_exact_scope_before_storage_access() {
    let valid_scope = scope(250);
    let invalid_scope = VersionedContextScopeV1::new(
        ProjectId::from_uuid(Uuid::nil()),
        valid_scope.context_id(),
        valid_scope.commit_id(),
    );
    let reads = Arc::new(Mutex::new(Vec::new()));
    let repository = RecordingRepository::new(reads.clone());

    let error = PersistedContextDiffReviewAdapter::new(&repository)
        .review(invalid_scope, valid_scope)
        .await
        .expect_err("nil project identity must fail closed");

    assert!(matches!(
        error,
        PersistedContextDiffReviewError::InvalidScope {
            side: PersistedContextDiffReviewSide::Source,
            scope,
            reason: "project identifier is nil",
        } if scope == invalid_scope
    ));
    assert!(reads.lock().expect("read log").is_empty());
}

#[tokio::test]
async fn wraps_the_exact_side_when_persisted_read_fails() {
    let source_scope = scope(300);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(303)),
    );
    let repository = FailingRepository;

    let error = PersistedContextDiffReviewAdapter::new(&repository)
        .review(source_scope, target_scope)
        .await
        .expect_err("missing source must fail closed");

    assert!(matches!(
        error,
        PersistedContextDiffReviewError::SnapshotPairRead {
            source_scope: scope,
            source,
            ..
        } if scope == source_scope
            && matches!(
                source.as_ref(),
                ContextDiffSnapshotPersistenceError::NotFound { scope: missing }
                    if *missing == source_scope
            )
    ));
}

#[tokio::test]
async fn repeated_reviews_are_deterministic_and_cross_scope_is_rejected() {
    let source_scope = scope(400);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(403)),
    );
    let repository = InMemoryContextDiffSnapshotV1Repository::new();
    for (exact_scope, fingerprint) in [(source_scope, "same-suite"), (target_scope, "same-suite")] {
        repository
            .persist_context_diff_snapshot(persist_command(exact_scope, fingerprint))
            .await
            .expect("snapshot");
    }

    let service = PersistedContextDiffReviewAdapter::new(&repository);
    let first = service
        .review(source_scope, target_scope)
        .await
        .expect("first review");
    let second = service
        .review(source_scope, target_scope)
        .await
        .expect("second review");
    assert_eq!(first, second);

    let different_context = VersionedContextScopeV1::new(
        source_scope.project_id(),
        ContextId::from_uuid(Uuid::from_u128(999)),
        target_scope.commit_id(),
    );
    let error = service
        .review(source_scope, different_context)
        .await
        .expect_err("mixed Context scopes must fail closed");
    assert!(matches!(
        error,
        PersistedContextDiffReviewError::MismatchedContextScope {
            source_scope: actual_source,
            target_scope: actual_target,
        } if actual_source == source_scope && actual_target == different_context
    ));
}

#[derive(Clone)]
struct RecordingRepository {
    reads: Arc<Mutex<Vec<VersionedContextScopeV1>>>,
    pair_reads: Arc<Mutex<u32>>,
}

impl RecordingRepository {
    fn new(reads: Arc<Mutex<Vec<VersionedContextScopeV1>>>) -> Self {
        Self {
            reads,
            pair_reads: Arc::new(Mutex::new(0)),
        }
    }

    fn new_with_pair_reads(
        reads: Arc<Mutex<Vec<VersionedContextScopeV1>>>,
        pair_reads: Arc<Mutex<u32>>,
    ) -> Self {
        Self { reads, pair_reads }
    }
}

#[async_trait]
impl ContextDiffSnapshotV1Repository for RecordingRepository {
    async fn persist_context_diff_snapshot(
        &self,
        _command: PersistContextDiffSnapshotV1,
    ) -> Result<ContextDiffSnapshotWriteResult, ContextDiffSnapshotPersistenceError> {
        Err(ContextDiffSnapshotPersistenceError::RepositoryUnavailable)
    }

    async fn read_context_diff_snapshot(
        &self,
        scope: VersionedContextScopeV1,
    ) -> Result<contextlab_storage::ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError>
    {
        self.reads.lock().expect("read log").push(scope);
        Err(ContextDiffSnapshotPersistenceError::NotFound { scope })
    }
}

#[async_trait]
impl ContextDiffSnapshotV1PairRepository for RecordingRepository {
    async fn read_context_diff_snapshot_pair(
        &self,
        source_scope: VersionedContextScopeV1,
        _target_scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Pair, ContextDiffSnapshotPersistenceError> {
        *self.pair_reads.lock().expect("pair read log") += 1;
        Err(ContextDiffSnapshotPersistenceError::NotFound {
            scope: source_scope,
        })
    }
}

struct FailingRepository;

#[async_trait]
impl ContextDiffSnapshotV1Repository for FailingRepository {
    async fn persist_context_diff_snapshot(
        &self,
        _command: PersistContextDiffSnapshotV1,
    ) -> Result<ContextDiffSnapshotWriteResult, ContextDiffSnapshotPersistenceError> {
        Err(ContextDiffSnapshotPersistenceError::RepositoryUnavailable)
    }

    async fn read_context_diff_snapshot(
        &self,
        scope: VersionedContextScopeV1,
    ) -> Result<contextlab_storage::ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError>
    {
        Err(ContextDiffSnapshotPersistenceError::NotFound { scope })
    }
}

#[async_trait]
impl ContextDiffSnapshotV1PairRepository for FailingRepository {
    async fn read_context_diff_snapshot_pair(
        &self,
        source_scope: VersionedContextScopeV1,
        _target_scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Pair, ContextDiffSnapshotPersistenceError> {
        Err(ContextDiffSnapshotPersistenceError::NotFound {
            scope: source_scope,
        })
    }
}
