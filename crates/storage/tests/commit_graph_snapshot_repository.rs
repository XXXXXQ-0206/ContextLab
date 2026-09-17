//! Contract tests for durable Context/commit to project scope resolution.

use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_graph::ContextGraph;
use contextlab_storage::{
    CommitGraphSnapshotRepository, ContextCommitSnapshotWriter, ContextGraphProjection,
    ContextRecord, CreateContextCommitSnapshot, InMemoryContextGraphRepository, ProjectRecord,
};
use contextlab_versioning::{BranchName, CommitId, ContextChange, ContextCommit};
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn timestamp(seconds: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 28, 0, 0, seconds)
        .single()
        .expect("valid timestamp")
}

fn project_id() -> ProjectId {
    ProjectId::from_uuid(uuid(10_000))
}

fn repository(context_id: ContextId) -> InMemoryContextGraphRepository {
    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id().to_string(),
            workspace_id: "snapshot-workspace".to_owned(),
            name: "Snapshot project".to_owned(),
            slug: "snapshot-project".to_owned(),
            created_at: timestamp(0),
        }],
        contexts: vec![ContextRecord {
            id: context_id.to_string(),
            project_id: project_id().to_string(),
            experiment_id: None,
            name: "Snapshot context".to_owned(),
            description: None,
            created_at: timestamp(0),
        }],
        ..ContextGraphProjection::default()
    })
}

async fn persist_commit(
    repository: &InMemoryContextGraphRepository,
    context_id: ContextId,
) -> CommitId {
    let commit = ContextCommit::new(
        context_id,
        BranchName::default(),
        "Resolve snapshot scope",
        Vec::new(),
        vec![ContextChange::created_context("snapshot context")],
        timestamp(1),
    )
    .expect("valid commit");
    let commit_id = commit.id();
    repository
        .create_commit_snapshot(
            CreateContextCommitSnapshot::new(
                project_id(),
                commit,
                ContextGraph::new(),
                timestamp(2),
                1,
            )
            .expect("valid snapshot command"),
        )
        .await
        .expect("persist commit snapshot");
    commit_id
}

#[tokio::test]
async fn resolver_derives_project_from_durable_context_owner() {
    let context_id = ContextId::from_uuid(uuid(10_001));
    let repository = repository(context_id);
    let commit_id = persist_commit(&repository, context_id).await;

    assert_eq!(
        repository
            .scope_for_context_commit(context_id, commit_id)
            .await
            .expect("resolve durable scope"),
        contextlab_storage::CommitGraphSnapshotScope::new(project_id(), context_id, commit_id)
    );
}

#[tokio::test]
async fn resolver_rejects_unknown_context_or_commit_without_request_project() {
    let context_id = ContextId::from_uuid(uuid(10_002));
    let repository = repository(context_id);

    let error = repository
        .scope_for_context_commit(context_id, CommitId::from_uuid(uuid(10_003)))
        .await
        .expect_err("unknown commit must fail closed");
    assert!(matches!(
        error,
        contextlab_storage::StorageRepositoryError::ScopeUnavailable { .. }
    ));

    let error = repository
        .scope_for_context_commit(ContextId::from_uuid(uuid(10_004)), CommitId::new())
        .await
        .expect_err("unknown context must fail closed");
    assert!(matches!(
        error,
        contextlab_storage::StorageRepositoryError::ScopeUnavailable { .. }
    ));
}

#[tokio::test]
async fn project_resolver_reads_only_durable_context_ownership() {
    let context_id = ContextId::from_uuid(uuid(10_005));
    let repository = repository(context_id);

    assert_eq!(
        repository
            .project_id_for_context(context_id)
            .await
            .expect("resolve durable Context owner"),
        project_id()
    );

    let error = repository
        .project_id_for_context(ContextId::from_uuid(uuid(10_006)))
        .await
        .expect_err("unknown Context must fail closed");
    assert!(matches!(
        error,
        contextlab_storage::StorageRepositoryError::ScopeUnavailable { .. }
    ));
}

#[tokio::test]
async fn project_resolver_rejects_malformed_durable_owner() {
    let context_id = ContextId::from_uuid(uuid(10_007));
    let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
        contexts: vec![ContextRecord {
            id: context_id.to_string(),
            project_id: "not-a-project-uuid".to_owned(),
            experiment_id: None,
            name: "Malformed owner Context".to_owned(),
            description: None,
            created_at: timestamp(0),
        }],
        ..ContextGraphProjection::default()
    });

    let error = repository
        .project_id_for_context(context_id)
        .await
        .expect_err("malformed durable owner must fail closed");
    assert!(matches!(
        error,
        contextlab_storage::StorageRepositoryError::InvalidScope { .. }
    ));
}

#[test]
fn snapshot_serialization_keeps_scope_fields_at_the_top_level() {
    let project_id = project_id();
    let context_id = ContextId::from_uuid(uuid(10_008));
    let commit_id = CommitId::from_uuid(uuid(10_009));
    let scope =
        contextlab_storage::CommitGraphSnapshotScope::new(project_id, context_id, commit_id);
    let snapshot =
        contextlab_storage::CommitGraphSnapshot::new(scope, ContextGraph::new(), timestamp(3), 1)
            .expect("valid snapshot");

    let value = serde_json::to_value(snapshot).expect("serialize snapshot");
    assert_eq!(value["project_id"], project_id.to_string());
    assert_eq!(value["context_id"], context_id.to_string());
    assert_eq!(value["commit_id"], commit_id.to_string());
    assert!(value.get("graph").is_some());
    assert!(value.get("scope").is_none());
}
