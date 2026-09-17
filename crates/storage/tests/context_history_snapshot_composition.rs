//! Contract tests for composing read-only Context history and graph snapshots.

use chrono::{TimeZone, Utc};
use contextlab_auth::{AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
use contextlab_storage::{
    CommitGraphSnapshot, CommitGraphSnapshotScope, ContextCommitRecord,
    ContextGraphBranchHeadReviewWitnessRepository,
    ContextGraphBranchHeadReviewWitnessRepositoryError, ContextGraphProjection,
    ContextGraphReviewWitnessRepository, ContextRecord, CreateContextCommitSnapshot,
    GuardedContextCommitWrite, GuardedContextCommitWriter, IdempotencyKey,
    InMemoryContextGraphRepository, PersistedContextGraphDiffReviewSide,
    PersistedContextGraphHistoryReviewError, PersistedContextGraphHistoryReviewService,
    PersistedContextGraphWitnessReviewService, ProjectRecord, RequestDigest,
    StorageRepositoryError,
};
use contextlab_versioning::{
    BranchName, CommitId, ContextChange, ContextCommit, ExpectedBranchHead,
};
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn project_id() -> ProjectId {
    ProjectId::from_uuid(uuid(40_000))
}

fn context_id() -> ContextId {
    ContextId::from_uuid(uuid(40_001))
}

fn timestamp(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(seconds, 0)
        .single()
        .expect("valid fixture timestamp")
}

fn repository() -> InMemoryContextGraphRepository {
    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id().to_string(),
            workspace_id: "history-snapshot-workspace".to_owned(),
            name: "History snapshot project".to_owned(),
            slug: "history-snapshot".to_owned(),
            created_at: timestamp(0),
        }],
        contexts: vec![ContextRecord {
            id: context_id().to_string(),
            project_id: project_id().to_string(),
            experiment_id: None,
            name: "History snapshot Context".to_owned(),
            description: None,
            created_at: timestamp(0),
        }],
        ..ContextGraphProjection::default()
    })
}

fn principal() -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(PrincipalIdentity::new(
        IdentitySourceId::new("https://issuer.contextlab.test").expect("identity source"),
        PrincipalId::new("user:history-snapshot-tests").expect("principal"),
    ))
}

fn graph(label: &str) -> ContextGraph {
    let mut graph = ContextGraph::new();
    graph
        .add_node(
            GraphNode::new(
                format!("context:{}", context_id()),
                GraphNodeKind::Context,
                label,
            )
            .expect("graph node"),
        )
        .expect("insert graph node");
    graph
}

fn commit_record(
    commit_id: CommitId,
    parent_commit_ids: Vec<CommitId>,
    message: &str,
    authored_at: i64,
) -> ContextCommitRecord {
    let changes = if parent_commit_ids.is_empty() {
        vec![ContextChange::created_context("history snapshot fixture")]
    } else {
        vec![ContextChange::updated_metadata(
            contextlab_context_core::ContextMetadata::new(timestamp(authored_at)),
            "history snapshot fixture",
        )]
    };

    ContextCommitRecord {
        id: commit_id.to_string(),
        context_id: context_id().to_string(),
        branch_name: BranchName::default().to_string(),
        message: message.to_owned(),
        parent_commit_ids: parent_commit_ids
            .into_iter()
            .map(|parent_commit_id| parent_commit_id.to_string())
            .collect(),
        changes: serde_json::to_value(changes).expect("fixture changes"),
        change_count: 1,
        authored_at: timestamp(authored_at),
        created_at: timestamp(authored_at),
    }
}

async fn persist_commit(
    repository: &InMemoryContextGraphRepository,
    branch: BranchName,
    expected_head: ExpectedBranchHead,
    message: &str,
    idempotency_suffix: &str,
) -> CommitId {
    let parent_ids = match expected_head {
        ExpectedBranchHead::Unborn => Vec::new(),
        ExpectedBranchHead::Commit(commit_id) => vec![commit_id],
    };
    let changes = if expected_head == ExpectedBranchHead::Unborn {
        vec![ContextChange::created_context("history snapshot fixture")]
    } else {
        vec![ContextChange::updated_metadata(
            contextlab_context_core::ContextMetadata::new(timestamp(2)),
            "history snapshot fixture",
        )]
    };
    let commit = ContextCommit::new(
        context_id(),
        branch,
        message,
        parent_ids,
        changes,
        timestamp(if expected_head == ExpectedBranchHead::Unborn {
            1
        } else {
            2
        }),
    )
    .expect("valid fixture commit");
    let commit_id = commit.id();
    let snapshot = CreateContextCommitSnapshot::new(
        project_id(),
        commit,
        graph(message),
        timestamp(if expected_head == ExpectedBranchHead::Unborn {
            1
        } else {
            2
        }),
        1,
    )
    .expect("valid snapshot command");
    let command = GuardedContextCommitWrite::new(
        principal(),
        expected_head,
        IdempotencyKey::new(format!("history-snapshot-{idempotency_suffix}"))
            .expect("idempotency key"),
        RequestDigest::new(format!("sha256:history-snapshot-{idempotency_suffix}"))
            .expect("request digest"),
        snapshot,
    )
    .expect("valid guarded write");

    repository
        .create_guarded_commit_snapshot(command)
        .await
        .expect("persist fixture commit");
    commit_id
}

#[tokio::test]
async fn read_only_composition_accepts_complete_history_heads_and_graph_snapshot() {
    let repository = repository();
    let main = BranchName::default();
    let root_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Unborn,
        "History snapshot root",
        "root",
    )
    .await;
    let head_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Commit(root_id),
        "History snapshot head",
        "head",
    )
    .await;

    let review = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
        .review(
            CommitGraphSnapshotScope::new(project_id(), context_id(), root_id),
            CommitGraphSnapshotScope::new(project_id(), context_id(), head_id),
        )
        .await
        .expect("history-bound graph review");
    let history = review.history();
    assert_eq!(history.context_id(), context_id());
    assert_eq!(history.branches().collect::<Vec<_>>(), vec![&main]);
    assert_eq!(history.head(&main), Some(head_id));
    assert_eq!(
        history
            .commits_for_branch(&main)
            .expect("linear branch history")
            .iter()
            .map(|commit| commit.id())
            .collect::<Vec<_>>(),
        vec![root_id, head_id]
    );
    assert_eq!(review.graph_review().source().commit_id(), root_id);
    assert_eq!(review.graph_review().target().commit_id(), head_id);
    assert_eq!(
        review.graph_review().review().diff().modified_nodes().len(),
        1
    );
}

#[tokio::test]
async fn atomic_witness_binds_history_and_snapshots_before_graph_review() {
    let repository = repository();
    let main = BranchName::default();
    let root_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Unborn,
        "Atomic witness root",
        "witness-root",
    )
    .await;
    let head_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Commit(root_id),
        "Atomic witness head",
        "witness-head",
    )
    .await;

    let review = PersistedContextGraphWitnessReviewService::new(&repository)
        .review(
            CommitGraphSnapshotScope::new(project_id(), context_id(), root_id),
            CommitGraphSnapshotScope::new(project_id(), context_id(), head_id),
        )
        .await
        .expect("atomic history and snapshot witness");

    assert_eq!(review.history().head(&main), Some(head_id));
    assert_eq!(review.graph_review().source().commit_id(), root_id);
    assert_eq!(review.graph_review().target().commit_id(), head_id);
    assert_eq!(
        review.graph_review().review().diff().modified_nodes().len(),
        1
    );
}

#[tokio::test]
async fn branch_head_witness_binds_selected_head_to_the_same_snapshot_observation() {
    let repository = repository();
    let main = BranchName::default();
    let root_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Unborn,
        "Branch-bound root",
        "branch-bound-root",
    )
    .await;
    let head_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Commit(root_id),
        "Branch-bound head",
        "branch-bound-head",
    )
    .await;

    let latest_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Commit(head_id),
        "Branch-bound latest head",
        "branch-bound-latest-head",
    )
    .await;

    let witness = repository
        .read_context_graph_branch_head_review_witness(
            CommitGraphSnapshotScope::new(project_id(), context_id(), root_id),
            main.clone(),
        )
        .await
        .expect("branch-bound witness");

    assert_eq!(witness.branch(), &main);
    assert_eq!(witness.target_commit_id(), latest_id);
    assert_eq!(witness.witness().history().head(&main), Some(latest_id));
    assert_eq!(witness.witness().target().commit_id(), latest_id);

    let review = PersistedContextGraphWitnessReviewService::new(&repository)
        .review_branch_head(
            CommitGraphSnapshotScope::new(project_id(), context_id(), root_id),
            main,
        )
        .await
        .expect("branch-bound graph review");
    assert_eq!(review.graph_review().target().commit_id(), latest_id);
    assert_eq!(
        review.graph_review().review().diff().modified_nodes().len(),
        1
    );
}

#[tokio::test]
async fn atomic_witness_rejects_a_persisted_missing_intermediate_snapshot() {
    let root_id = CommitId::from_uuid(uuid(40_010));
    let middle_id = CommitId::from_uuid(uuid(40_011));
    let target_id = CommitId::from_uuid(uuid(40_012));
    let mut projection = ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id().to_string(),
            workspace_id: "history-snapshot-workspace".to_owned(),
            name: "History snapshot project".to_owned(),
            slug: "history-snapshot".to_owned(),
            created_at: timestamp(0),
        }],
        contexts: vec![ContextRecord {
            id: context_id().to_string(),
            project_id: project_id().to_string(),
            experiment_id: None,
            name: "History snapshot Context".to_owned(),
            description: None,
            created_at: timestamp(0),
        }],
        ..ContextGraphProjection::default()
    };
    projection.commits = vec![
        commit_record(root_id, Vec::new(), "Snapshot root", 1),
        commit_record(middle_id, vec![root_id], "Snapshot middle", 2),
        commit_record(target_id, vec![middle_id], "Snapshot target", 3),
    ];

    let source_scope = CommitGraphSnapshotScope::new(project_id(), context_id(), root_id);
    let middle_scope = CommitGraphSnapshotScope::new(project_id(), context_id(), middle_id);
    let target_scope = CommitGraphSnapshotScope::new(project_id(), context_id(), target_id);
    let repository = InMemoryContextGraphRepository::with_commit_graph_snapshots(
        projection,
        [
            CommitGraphSnapshot::new(source_scope, graph("Snapshot root"), timestamp(1), 1)
                .expect("source snapshot"),
            CommitGraphSnapshot::new(target_scope, graph("Snapshot target"), timestamp(3), 1)
                .expect("target snapshot"),
        ],
    )
    .expect("endpoint snapshot fixture");

    let error = repository
        .read_context_graph_review_witness(source_scope, target_scope)
        .await
        .expect_err("missing intermediate snapshot must fail before graph review");

    assert!(matches!(
        error,
        StorageRepositoryError::ScopeUnavailable { scope } if scope == middle_scope.to_string()
    ));
}

#[tokio::test]
async fn branch_head_witness_rejects_unknown_branch_before_snapshot_projection() {
    let repository = repository();
    let main = BranchName::default();
    let root_id = persist_commit(
        &repository,
        main,
        ExpectedBranchHead::Unborn,
        "Unknown branch root",
        "unknown-branch-root",
    )
    .await;
    let missing = BranchName::new("missing").expect("branch");

    let error = repository
        .read_context_graph_branch_head_review_witness(
            CommitGraphSnapshotScope::new(project_id(), context_id(), root_id),
            missing.clone(),
        )
        .await
        .expect_err("unknown branch");
    assert!(matches!(
        error,
        ContextGraphBranchHeadReviewWitnessRepositoryError::BranchUnknown { branch }
            if branch == missing
    ));
}

#[tokio::test]
async fn atomic_witness_rejects_cross_project_scope_before_projection() {
    let repository = repository();
    let main = BranchName::default();
    let root_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Unborn,
        "Invalid witness root",
        "invalid-witness-root",
    )
    .await;
    let head_id = persist_commit(
        &repository,
        main,
        ExpectedBranchHead::Commit(root_id),
        "Invalid witness head",
        "invalid-witness-head",
    )
    .await;

    let error = repository
        .read_context_graph_review_witness(
            CommitGraphSnapshotScope::new(
                ProjectId::from_uuid(uuid(40_098)),
                context_id(),
                root_id,
            ),
            CommitGraphSnapshotScope::new(project_id(), context_id(), head_id),
        )
        .await
        .expect_err("cross-project witness scope");
    assert!(matches!(
        error,
        contextlab_storage::StorageRepositoryError::InvalidScope { .. }
    ));
}

#[tokio::test]
async fn graph_snapshot_review_rejects_a_commit_missing_from_complete_history() {
    let repository = repository();
    let main = BranchName::default();
    let root_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Unborn,
        "Incomplete history root",
        "incomplete-root",
    )
    .await;
    let head_id = persist_commit(
        &repository,
        main.clone(),
        ExpectedBranchHead::Commit(root_id),
        "Incomplete history head",
        "incomplete-head",
    )
    .await;

    let missing_target = CommitId::from_uuid(uuid(40_099));
    let error = PersistedContextGraphHistoryReviewService::new(&repository, &repository)
        .review(
            CommitGraphSnapshotScope::new(project_id(), context_id(), head_id),
            CommitGraphSnapshotScope::new(project_id(), context_id(), missing_target),
        )
        .await
        .expect_err("missing target must fail before graph review");
    assert!(matches!(
        error,
        PersistedContextGraphHistoryReviewError::CommitMissing {
            side: PersistedContextGraphDiffReviewSide::Target,
            commit_id,
        } if commit_id == missing_target
    ));
}
