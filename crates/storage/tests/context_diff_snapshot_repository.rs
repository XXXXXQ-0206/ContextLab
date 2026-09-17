//! Focused Memory repository contracts for immutable exact-commit snapshots.

use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::{
    BehaviorSnapshotV1, ContextDiffSnapshotV1, EvaluationSnapshotV1, SemanticSnapshotV1,
    VersionedContextScopeV1,
};
use contextlab_graph::ContextGraph;
use contextlab_storage::{
    CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1, ContextDiffSnapshotPersistenceError,
    ContextDiffSnapshotV1PairRepository, ContextDiffSnapshotV1Repository,
    ContextDiffSnapshotWriteDisposition, InMemoryContextDiffSnapshotV1Repository,
    InMemoryContextGraphRepository, PersistContextDiffSnapshotV1,
};
use contextlab_versioning::CommitId;
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

fn captured_at(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(seconds, 123_456_789)
        .single()
        .expect("timestamp")
}

fn command(
    exact_scope: VersionedContextScopeV1,
    fingerprint: &str,
) -> PersistContextDiffSnapshotV1 {
    PersistContextDiffSnapshotV1::new(
        exact_scope,
        CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1,
        snapshot(fingerprint),
        captured_at(1_753_680_000),
    )
    .expect("valid command")
}

#[tokio::test]
async fn memory_round_trip_replay_and_digest_are_exact() {
    let exact_scope = scope(100);
    let command = command(exact_scope, "same-input");
    let repository = InMemoryContextDiffSnapshotV1Repository::new();

    let created = repository
        .persist_context_diff_snapshot(command.clone())
        .await
        .expect("create snapshot");
    assert_eq!(
        created.disposition(),
        ContextDiffSnapshotWriteDisposition::Created
    );

    let replay = repository
        .persist_context_diff_snapshot(command.clone())
        .await
        .expect("replay snapshot");
    assert_eq!(
        replay.disposition(),
        ContextDiffSnapshotWriteDisposition::Replayed
    );

    let loaded = repository
        .read_context_diff_snapshot(exact_scope)
        .await
        .expect("read snapshot");
    assert_eq!(loaded.scope(), exact_scope);
    assert_eq!(loaded.snapshot(), command.record().snapshot());
    assert_eq!(loaded.snapshot_digest(), command.record().snapshot_digest());
    assert_eq!(
        loaded.captured_at(),
        Utc.timestamp_opt(1_753_680_000, 123_456_000)
            .single()
            .expect("normalized timestamp")
    );
    assert_eq!(
        loaded.snapshot_digest(),
        repository
            .read_context_diff_snapshot(exact_scope)
            .await
            .expect("deterministic read")
            .snapshot_digest()
    );
}

#[tokio::test]
async fn memory_rejects_changed_payload_and_preserves_append_only_state() {
    let exact_scope = scope(200);
    let repository = InMemoryContextDiffSnapshotV1Repository::new();
    repository
        .persist_context_diff_snapshot(command(exact_scope, "original"))
        .await
        .expect("create snapshot");

    let error = repository
        .persist_context_diff_snapshot(command(exact_scope, "changed"))
        .await
        .expect_err("changed payload must conflict");
    assert!(matches!(
        error,
        ContextDiffSnapshotPersistenceError::Conflict { scope } if scope == exact_scope
    ));
    assert_eq!(
        repository
            .read_context_diff_snapshot(exact_scope)
            .await
            .expect("original remains")
            .snapshot(),
        command(exact_scope, "original").record().snapshot()
    );
}

#[tokio::test]
async fn memory_reads_only_exact_project_context_and_commit_scope_and_rejects_invalid_schema() {
    let exact_scope = scope(300);
    let repository = InMemoryContextDiffSnapshotV1Repository::new();
    repository
        .persist_context_diff_snapshot(command(exact_scope, "scope"))
        .await
        .expect("create snapshot");

    let other_scope = scope(400);
    for (label, drifted_scope) in [
        (
            "project",
            VersionedContextScopeV1::new(
                other_scope.project_id(),
                exact_scope.context_id(),
                exact_scope.commit_id(),
            ),
        ),
        (
            "context",
            VersionedContextScopeV1::new(
                exact_scope.project_id(),
                other_scope.context_id(),
                exact_scope.commit_id(),
            ),
        ),
        (
            "commit",
            VersionedContextScopeV1::new(
                exact_scope.project_id(),
                exact_scope.context_id(),
                other_scope.commit_id(),
            ),
        ),
    ] {
        let missing = repository
            .read_context_diff_snapshot(drifted_scope)
            .await
            .expect_err("scope drift must be missing");
        assert!(
            matches!(
                missing,
                ContextDiffSnapshotPersistenceError::NotFound { scope } if scope == drifted_scope,
            ),
            "{label} drift must remain exact"
        );
    }

    let unknown_schema = PersistContextDiffSnapshotV1::new(
        exact_scope,
        "context-diff-snapshot-v99",
        snapshot("schema"),
        captured_at(2),
    )
    .expect_err("unknown schema must be rejected");
    assert!(matches!(
        unknown_schema,
        ContextDiffSnapshotPersistenceError::UnsupportedSchema { .. }
    ));

    let invalid_scope = PersistContextDiffSnapshotV1::new(
        VersionedContextScopeV1::new(
            ProjectId::from_uuid(Uuid::nil()),
            exact_scope.context_id(),
            exact_scope.commit_id(),
        ),
        CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1,
        snapshot("invalid-scope"),
        captured_at(3),
    )
    .expect_err("nil scope identifiers must be rejected");
    assert!(matches!(
        invalid_scope,
        ContextDiffSnapshotPersistenceError::InvalidScope { .. }
    ));
}

#[tokio::test]
async fn graph_memory_repository_reads_both_exact_snapshots_through_one_pair_boundary() {
    let source_scope = scope(600);
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(603)),
    );
    let repository = InMemoryContextGraphRepository::context_engineering_preview();
    repository
        .persist_context_diff_snapshot(command(source_scope, "source"))
        .await
        .expect("persist source snapshot");
    repository
        .persist_context_diff_snapshot(command(target_scope, "target"))
        .await
        .expect("persist target snapshot");

    let pair = repository
        .read_context_diff_snapshot_pair(source_scope, target_scope)
        .await
        .expect("read exact snapshot pair");

    assert_eq!(pair.source().scope(), source_scope);
    assert_eq!(pair.target().scope(), target_scope);
    assert_eq!(
        pair.source()
            .snapshot()
            .evaluation()
            .comparability_fingerprint()
            .as_str(),
        "source"
    );
    assert_eq!(
        pair.target()
            .snapshot()
            .evaluation()
            .comparability_fingerprint()
            .as_str(),
        "target"
    );
}

#[test]
fn identical_snapshots_have_deterministic_serialization_and_digest() {
    let first = command(scope(500), "deterministic");
    let second = command(scope(501), "deterministic");
    assert_eq!(first.record().snapshot(), second.record().snapshot());
    assert_eq!(
        first.record().snapshot_digest(),
        second.record().snapshot_digest()
    );
    assert_eq!(
        serde_json::to_vec(first.record().snapshot()).expect("serialize first"),
        serde_json::to_vec(second.record().snapshot()).expect("serialize second")
    );
}
