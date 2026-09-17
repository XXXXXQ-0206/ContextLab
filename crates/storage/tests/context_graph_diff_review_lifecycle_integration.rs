//! Integration coverage for the guarded Context lifecycle to versioned graph review path.

use chrono::{DateTime, TimeZone, Utc};
use contextlab_auth::{AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity};
use contextlab_context_core::{ComponentContent, ContextComponentKind, ContextId, ProjectId};
use contextlab_diff_engine::VersionedContextScopeV1;
use contextlab_storage::{
    CommitGraphSnapshotScope, ContextGraphProjection, ContextLifecycleCommand,
    ContextLifecycleService, ContextRecord, InMemoryContextGraphRepository,
    PersistedContextGraphDiffReviewError, PersistedContextGraphDiffReviewService, ProjectRecord,
};
use contextlab_versioning::{BranchName, CommitId};
use serde_json::json;
use uuid::Uuid;

fn project_id() -> ProjectId {
    ProjectId::from_uuid(Uuid::from_u128(100))
}

fn context_id() -> ContextId {
    ContextId::from_uuid(Uuid::from_u128(200))
}

fn timestamp(seconds: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 2, 9, 0, seconds as u32)
        .single()
        .expect("valid fixture timestamp")
}

fn principal() -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(PrincipalIdentity::new(
        IdentitySourceId::new("https://id.contextlab.test").expect("identity source"),
        PrincipalId::new("graph-diff-lifecycle-integration").expect("principal id"),
    ))
}

fn repository() -> InMemoryContextGraphRepository {
    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id().to_string(),
            workspace_id: "integration-workspace".to_owned(),
            name: "Lifecycle Graph Diff Project".to_owned(),
            slug: "lifecycle-graph-diff".to_owned(),
            created_at: timestamp(0),
        }],
        contexts: vec![ContextRecord {
            id: context_id().to_string(),
            project_id: project_id().to_string(),
            experiment_id: None,
            name: "Lifecycle Graph Diff Context".to_owned(),
            description: None,
            created_at: timestamp(0),
        }],
        ..ContextGraphProjection::default()
    })
}

#[tokio::test]
async fn guarded_lifecycle_facts_feed_exact_versioned_graph_diff() {
    let repository = repository();
    let lifecycle = ContextLifecycleService::new(&repository);
    let context = context_id();
    let branch = BranchName::default();

    let root = lifecycle
        .execute(ContextLifecycleCommand::initialize(
            principal(),
            context,
            branch.clone(),
            contextlab_storage::IdempotencyKey::new("graph-diff-root").expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-root")
                .expect("request digest"),
            "Create graph diff Context",
            timestamp(1),
        ))
        .await
        .expect("initialize Context")
        .commit_id();

    let source_created = lifecycle
        .execute(ContextLifecycleCommand::create(
            principal(),
            context,
            branch.clone(),
            root,
            contextlab_storage::IdempotencyKey::new("graph-diff-source").expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-source")
                .expect("request digest"),
            "Create graph diff source",
            ContextComponentKind::Prompt,
            "Source prompt",
            json!({"locale": "en-US"}),
            ComponentContent::new("Use the source prompt."),
            timestamp(2),
        ))
        .await
        .expect("create source component")
        .commit_id();

    let source_component_id = lifecycle
        .read_state_at_commit(context, source_created)
        .await
        .expect("read created source lifecycle state")
        .components()[0]
        .state()
        .component()
        .id();
    let source_updated = lifecycle
        .execute(ContextLifecycleCommand::update(
            principal(),
            context,
            branch.clone(),
            source_created,
            contextlab_storage::IdempotencyKey::new("graph-diff-source-update")
                .expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-source-update")
                .expect("request digest"),
            "Revise graph diff source content",
            source_component_id,
            ComponentContent::new("Use the revised source prompt."),
            timestamp(3),
        ))
        .await
        .expect("update source component content")
        .commit_id();
    let updated_source_state = lifecycle
        .read_state_at_commit(context, source_updated)
        .await
        .expect("read updated source lifecycle state");
    let updated_source = updated_source_state
        .components()
        .iter()
        .find(|component| component.state().component().id() == source_component_id)
        .expect("updated source component");
    assert_eq!(updated_source.content().commit_id(), source_updated);
    assert_eq!(
        updated_source.content().content().as_str(),
        "Use the revised source prompt."
    );

    let content_review = PersistedContextGraphDiffReviewService::new(&repository)
        .review(
            CommitGraphSnapshotScope::new(project_id(), context, source_created),
            CommitGraphSnapshotScope::new(project_id(), context, source_updated),
        )
        .await
        .expect("review exact content revision pair");
    assert!(content_review.review().diff().is_empty());

    let source_and_target_components = lifecycle
        .execute(ContextLifecycleCommand::create(
            principal(),
            context,
            branch.clone(),
            source_updated,
            contextlab_storage::IdempotencyKey::new("graph-diff-target").expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-target")
                .expect("request digest"),
            "Create graph diff target",
            ContextComponentKind::Knowledge,
            "Target knowledge",
            json!({"locale": "en-US"}),
            ComponentContent::new("Use the target knowledge."),
            timestamp(4),
        ))
        .await
        .expect("create target component")
        .commit_id();

    let materialized = lifecycle
        .read_state_at_commit(context, source_and_target_components)
        .await
        .expect("read exact pre-relationship lifecycle state");
    let target_component_id = materialized
        .components()
        .iter()
        .find(|component| component.state().component().name().as_str() == "Target knowledge")
        .expect("target component")
        .state()
        .component()
        .id();

    let revised = lifecycle
        .execute(ContextLifecycleCommand::add_uses_relationship(
            principal(),
            context,
            branch,
            source_and_target_components,
            contextlab_storage::IdempotencyKey::new("graph-diff-relationship")
                .expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-relationship")
                .expect("request digest"),
            "Connect source prompt to target knowledge",
            source_component_id,
            target_component_id,
            timestamp(5),
        ))
        .await
        .expect("persist relationship lifecycle transition")
        .commit_id();

    let removed = lifecycle
        .execute(ContextLifecycleCommand::remove_uses_relationship(
            principal(),
            context,
            BranchName::default(),
            revised,
            contextlab_storage::IdempotencyKey::new("graph-diff-relationship-remove")
                .expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-relationship-remove")
                .expect("request digest"),
            "Disconnect source prompt from target knowledge",
            source_component_id,
            target_component_id,
            timestamp(6),
        ))
        .await
        .expect("persist relationship removal lifecycle transition")
        .commit_id();

    let source_scope =
        CommitGraphSnapshotScope::new(project_id(), context, source_and_target_components);
    let target_scope = CommitGraphSnapshotScope::new(project_id(), context, revised);
    let source_state = lifecycle
        .read_state_at_commit(context, source_and_target_components)
        .await
        .expect("read exact source lifecycle state");
    let target_state = lifecycle
        .read_state_at_commit(context, revised)
        .await
        .expect("read exact target lifecycle state");
    let removed_state = lifecycle
        .read_state_at_commit(context, removed)
        .await
        .expect("read exact relationship removal lifecycle state");

    assert_eq!(source_state.scope(), source_scope);
    assert_eq!(target_state.scope(), target_scope);
    assert_eq!(source_state.graph_snapshot().scope(), source_scope);
    assert_eq!(target_state.graph_snapshot().scope(), target_scope);
    assert!(
        !source_state
            .graph_snapshot()
            .graph()
            .edges()
            .iter()
            .any(|edge| {
                edge.source().as_str() == format!("component:{source_component_id}")
                    && edge.target().as_str() == format!("component:{target_component_id}")
            })
    );
    assert!(
        target_state
            .graph_snapshot()
            .graph()
            .edges()
            .iter()
            .any(|edge| {
                edge.source().as_str() == format!("component:{source_component_id}")
                    && edge.target().as_str() == format!("component:{target_component_id}")
            })
    );
    assert!(
        !removed_state
            .graph_snapshot()
            .graph()
            .edges()
            .iter()
            .any(|edge| {
                edge.source().as_str() == format!("component:{source_component_id}")
                    && edge.target().as_str() == format!("component:{target_component_id}")
            })
    );

    let review = PersistedContextGraphDiffReviewService::new(&repository)
        .review(source_scope, target_scope)
        .await
        .expect("review exact guarded lifecycle pair");

    assert_eq!(review.source().scope(), source_scope);
    assert_eq!(review.target().scope(), target_scope);
    assert_eq!(
        review.review().source_scope(),
        VersionedContextScopeV1::new(
            source_scope.project_id(),
            source_scope.context_id(),
            source_scope.commit_id(),
        )
    );
    assert_eq!(
        review.review().target_scope(),
        VersionedContextScopeV1::new(
            target_scope.project_id(),
            target_scope.context_id(),
            target_scope.commit_id(),
        )
    );
    assert_eq!(review.review().diff().added_edges().len(), 1);
    assert!(review.review().diff().removed_edges().is_empty());
    assert_eq!(
        review.review().diff().added_edges()[0].kind(),
        contextlab_graph::GraphEdgeKind::Uses
    );

    let removal_review = PersistedContextGraphDiffReviewService::new(&repository)
        .review(
            target_scope,
            CommitGraphSnapshotScope::new(project_id(), context, removed),
        )
        .await
        .expect("review exact relationship removal pair");
    assert_eq!(removal_review.review().diff().added_edges().len(), 0);
    assert_eq!(removal_review.review().diff().removed_edges().len(), 1);
    assert_eq!(
        removal_review.review().diff().removed_edges()[0].kind(),
        contextlab_graph::GraphEdgeKind::Uses
    );
}

#[tokio::test]
async fn graph_diff_review_fails_closed_for_missing_exact_lifecycle_witness() {
    let repository = repository();
    let root = ContextLifecycleService::new(&repository)
        .execute(ContextLifecycleCommand::initialize(
            principal(),
            context_id(),
            BranchName::default(),
            contextlab_storage::IdempotencyKey::new("graph-diff-missing-root")
                .expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-missing-root")
                .expect("request digest"),
            "Create missing-witness Context",
            timestamp(1),
        ))
        .await
        .expect("initialize Context")
        .commit_id();
    let source_scope = CommitGraphSnapshotScope::new(project_id(), context_id(), root);
    let missing_scope = CommitGraphSnapshotScope::new(project_id(), context_id(), CommitId::new());

    let error = PersistedContextGraphDiffReviewService::new(&repository)
        .review(source_scope, missing_scope)
        .await
        .expect_err("missing exact lifecycle witness must fail closed");

    assert!(matches!(
        error,
        PersistedContextGraphDiffReviewError::LifecycleMissing { scope, .. }
            if scope == missing_scope
    ));
}

#[tokio::test]
async fn graph_diff_review_fails_closed_for_mismatched_exact_scope() {
    let repository = repository();
    let root = ContextLifecycleService::new(&repository)
        .execute(ContextLifecycleCommand::initialize(
            principal(),
            context_id(),
            BranchName::default(),
            contextlab_storage::IdempotencyKey::new("graph-diff-scope-root")
                .expect("idempotency key"),
            contextlab_storage::RequestDigest::new("sha256:graph-diff-scope-root")
                .expect("request digest"),
            "Create scope-witness Context",
            timestamp(1),
        ))
        .await
        .expect("initialize Context")
        .commit_id();
    let source_scope = CommitGraphSnapshotScope::new(project_id(), context_id(), root);
    let mismatched_scope = CommitGraphSnapshotScope::new(
        ProjectId::from_uuid(Uuid::from_u128(999)),
        context_id(),
        CommitId::new(),
    );

    let error = PersistedContextGraphDiffReviewService::new(&repository)
        .review(source_scope, mismatched_scope)
        .await
        .expect_err("mixed project scope must fail closed before reads");

    assert!(matches!(
        error,
        PersistedContextGraphDiffReviewError::MismatchedContextScope {
            source_scope: actual_source,
            target_scope: actual_target,
        } if actual_source == source_scope && actual_target == mismatched_scope
    ));
}
