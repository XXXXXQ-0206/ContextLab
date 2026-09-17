//! Red-first coverage for the private typed branch-head read contract.

use chrono::{TimeZone, Utc};
use contextlab_auth::{AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_graph::ContextGraph;
use contextlab_storage::{
    ContextBranchHead, ContextBranchRepository, ContextBranchRepositoryError,
    ContextGraphProjection, ContextRecord, CreateContextCommitSnapshot, GuardedContextCommitWrite,
    GuardedContextCommitWriter, IdempotencyKey, InMemoryContextGraphRepository, ProjectRecord,
    RequestDigest,
};
use contextlab_versioning::{
    BranchName, CommitId, ContextChange, ContextCommit, ExpectedBranchHead,
};

fn repository_with_context(context_id: ContextId) -> InMemoryContextGraphRepository {
    repository_with_contexts([context_id])
}

fn repository_with_contexts(
    context_ids: impl IntoIterator<Item = ContextId>,
) -> InMemoryContextGraphRepository {
    let context_ids = context_ids.into_iter().collect::<Vec<_>>();
    let contexts = context_ids
        .iter()
        .map(|context_id| ContextRecord {
            id: context_id.to_string(),
            project_id: project_id_for_context(*context_id).to_string(),
            experiment_id: None,
            name: "Test Context".to_owned(),
            description: Some("private branch-head fixture".to_owned()),
            created_at: timestamp(0),
        })
        .collect();
    let projects = context_ids
        .iter()
        .map(|context_id| ProjectRecord {
            id: project_id_for_context(*context_id).to_string(),
            workspace_id: "branch-head-workspace".to_owned(),
            name: "Branch-head test project".to_owned(),
            slug: format!("branch-head-{}", context_id),
            created_at: timestamp(0),
        })
        .collect();

    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects,
        contexts,
        ..ContextGraphProjection::default()
    })
}

fn project_id_for_context(context_id: ContextId) -> ProjectId {
    ProjectId::from_uuid(context_id.as_uuid())
}

fn timestamp(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 30, 0, 0, seconds as u32)
        .single()
        .expect("valid timestamp")
}

fn principal() -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(PrincipalIdentity::new(
        IdentitySourceId::new("https://issuer.contextlab.test").expect("identity source"),
        PrincipalId::new("user:branch-head-tests").expect("principal"),
    ))
}

async fn persist_branch_head(
    repository: &InMemoryContextGraphRepository,
    context_id: ContextId,
    branch_name: &str,
    expected_branch_head: ExpectedBranchHead,
    idempotency_suffix: &str,
) -> CommitId {
    let branch = BranchName::new(branch_name).expect("branch name");
    let parent_ids = match expected_branch_head {
        ExpectedBranchHead::Unborn => Vec::new(),
        ExpectedBranchHead::Commit(commit_id) => vec![commit_id],
    };
    let commit = ContextCommit::new(
        context_id,
        branch,
        format!("Persist {branch_name} branch head"),
        parent_ids,
        vec![ContextChange::created_context("branch-head fixture")],
        timestamp(1),
    )
    .expect("valid commit");
    let commit_id = commit.id();
    let snapshot_command = CreateContextCommitSnapshot::new(
        project_id_for_context(context_id),
        commit,
        ContextGraph::new(),
        timestamp(2),
        1,
    )
    .expect("valid snapshot command");
    let command = GuardedContextCommitWrite::new(
        principal(),
        expected_branch_head,
        IdempotencyKey::new(format!("branch-head-{idempotency_suffix}")).expect("idempotency key"),
        RequestDigest::new(format!("sha256:branch-head-{idempotency_suffix}"))
            .expect("request digest"),
        snapshot_command,
    )
    .expect("valid guarded command");

    repository
        .create_guarded_commit_snapshot(command)
        .await
        .expect("persist branch head");
    commit_id
}

#[tokio::test]
async fn lists_known_context_branches_as_an_empty_stable_collection() {
    let context_id = ContextId::new();
    let repository = repository_with_context(context_id);

    let branches = repository
        .list_context_branch_heads(context_id)
        .await
        .expect("known Context branch list");

    assert!(branches.is_empty());
}

#[tokio::test]
async fn distinguishes_unknown_context_from_unknown_branch() {
    let context_id = ContextId::new();
    let repository = repository_with_context(context_id);
    let branch = BranchName::new("feature/read-only").expect("branch");

    let unknown_context = repository
        .list_context_branch_heads(ContextId::new())
        .await
        .expect_err("unknown Context must fail closed");
    assert!(matches!(
        unknown_context,
        ContextBranchRepositoryError::UnknownContext { .. }
    ));

    let unknown_branch = repository
        .get_context_branch_head(context_id, branch.clone())
        .await
        .expect_err("unknown branch must fail closed");
    assert!(matches!(
        unknown_branch,
        ContextBranchRepositoryError::UnknownBranch {
            context_id: actual_context,
            branch: actual_branch,
        } if actual_context == context_id && actual_branch == branch
    ));
}

#[tokio::test]
async fn lists_only_the_exact_context_in_deterministic_branch_order() {
    let context_id = ContextId::new();
    let other_context_id = ContextId::new();
    let repository = repository_with_contexts([context_id, other_context_id]);

    let z_commit_id = persist_branch_head(
        &repository,
        context_id,
        "feature/z",
        ExpectedBranchHead::Unborn,
        "context-z",
    )
    .await;
    let a_commit_id = persist_branch_head(
        &repository,
        context_id,
        "feature/a",
        ExpectedBranchHead::Unborn,
        "context-a",
    )
    .await;
    let other_commit_id = persist_branch_head(
        &repository,
        other_context_id,
        "feature/a",
        ExpectedBranchHead::Unborn,
        "other-a",
    )
    .await;

    let heads = repository
        .list_context_branch_heads(context_id)
        .await
        .expect("exact Context branch heads");
    assert_eq!(
        heads
            .iter()
            .map(|head| head.branch().as_str())
            .collect::<Vec<_>>(),
        vec!["feature/a", "feature/z"]
    );
    assert!(heads.iter().all(|head| head.context_id() == context_id));
    assert_eq!(heads[0].head_commit_id(), Some(a_commit_id));
    assert_eq!(heads[0].revision(), 1);
    assert_eq!(heads[1].head_commit_id(), Some(z_commit_id));
    assert_eq!(heads[1].revision(), 1);

    let other_heads = repository
        .list_context_branch_heads(other_context_id)
        .await
        .expect("other exact Context branch heads");
    assert_eq!(other_heads.len(), 1);
    assert_eq!(other_heads[0].context_id(), other_context_id);
    assert_eq!(other_heads[0].head_commit_id(), Some(other_commit_id));

    let cross_context_branch = repository
        .get_context_branch_head(context_id, BranchName::new("feature/a").expect("branch"))
        .await
        .expect("exact branch lookup");
    assert_eq!(cross_context_branch.head_commit_id(), Some(a_commit_id));
    assert_ne!(cross_context_branch.head_commit_id(), Some(other_commit_id));

    let error = repository
        .get_context_branch_head(
            context_id,
            BranchName::new("feature/missing").expect("branch"),
        )
        .await
        .expect_err("a branch from another Context must not be visible");
    assert!(matches!(
        error,
        ContextBranchRepositoryError::UnknownBranch {
            context_id: actual_context,
            ..
        } if actual_context == context_id
    ));
}

#[tokio::test]
async fn repeated_branch_writes_keep_one_head_and_advance_its_revision() {
    let context_id = ContextId::new();
    let repository = repository_with_context(context_id);
    let first_commit_id = persist_branch_head(
        &repository,
        context_id,
        "main",
        ExpectedBranchHead::Unborn,
        "main-first",
    )
    .await;
    let second_commit_id = persist_branch_head(
        &repository,
        context_id,
        "main",
        ExpectedBranchHead::Commit(first_commit_id),
        "main-second",
    )
    .await;

    let heads = repository
        .list_context_branch_heads(context_id)
        .await
        .expect("branch heads after fast-forward");
    assert_eq!(heads.len(), 1, "one logical branch must have one head row");
    assert_eq!(heads[0].branch().as_str(), "main");
    assert_eq!(heads[0].head_commit_id(), Some(second_commit_id));
    assert_eq!(heads[0].revision(), 2);
}

#[test]
fn public_branch_head_model_preserves_an_unborn_head_without_a_synthetic_commit() {
    let context_id = ContextId::new();
    let head = ContextBranchHead::new(
        context_id,
        BranchName::new("main").expect("branch"),
        None,
        0,
    );

    assert_eq!(head.context_id(), context_id);
    assert_eq!(head.head_commit_id(), None);
    assert_eq!(head.revision(), 0);
    let serialized = serde_json::to_value(head).expect("serialize unborn branch head");
    assert_eq!(serialized["head_commit_id"], serde_json::Value::Null);
}
