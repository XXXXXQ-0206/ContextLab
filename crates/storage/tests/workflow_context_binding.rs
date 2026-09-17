//! Contract tests for append-only Workflow source bindings.

use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_graph::ContextGraph;
use contextlab_storage::{
    ContextCommitSnapshotWriter, ContextGraphProjection, ContextRecord,
    ContextWorkflowBindingRepository, CreateContextCommitSnapshot, InMemoryContextGraphRepository,
    ProjectRecord, WorkflowContextBindingWriteDisposition,
};
use contextlab_versioning::{BranchName, CommitId, ContextChange, ContextCommit};
use contextlab_workflow::{
    ContextCommitSource, WorkflowContextBinding, WorkflowContextBindingId, WorkflowDefinition,
    WorkflowId, WorkflowNode, WorkflowNodeId, WorkflowRevision,
};
use uuid::Uuid;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn timestamp(seconds: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 19, 0, 0, seconds)
        .single()
        .expect("valid timestamp")
}

fn project_id() -> ProjectId {
    ProjectId::from_uuid(uuid(10_000))
}

fn workflow_definition(workflow_id: u128, revision: u64) -> WorkflowDefinition {
    let revision = WorkflowRevision::new(revision).expect("positive revision");
    WorkflowDefinition::new(
        WorkflowId::from_uuid(uuid(workflow_id)),
        revision,
        vec![WorkflowNode::new(
            WorkflowNodeId::from_uuid(uuid(workflow_id + 1)),
            revision,
        )],
        Vec::new(),
    )
    .expect("valid workflow definition")
}

fn repository(context_id: ContextId) -> InMemoryContextGraphRepository {
    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id().to_string(),
            workspace_id: "workflow-workspace".to_owned(),
            name: "Workflow Binding Project".to_owned(),
            slug: "workflow-binding".to_owned(),
            created_at: timestamp(0),
        }],
        contexts: vec![ContextRecord {
            id: context_id.to_string(),
            project_id: project_id().to_string(),
            experiment_id: None,
            name: "Workflow binding Context".to_owned(),
            description: None,
            created_at: timestamp(0),
        }],
        ..ContextGraphProjection::default()
    })
}

async fn persist_source_commit(
    repository: &InMemoryContextGraphRepository,
    context_id: ContextId,
    parent_ids: Vec<CommitId>,
) -> CommitId {
    let commit = ContextCommit::new(
        context_id,
        BranchName::default(),
        "Seal workflow source Context",
        parent_ids,
        vec![ContextChange::created_context("workflow source")],
        timestamp(1),
    )
    .expect("valid source commit");
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
            .expect("valid source snapshot"),
        )
        .await
        .expect("persist source commit and snapshot");
    commit_id
}

#[tokio::test]
async fn binding_persists_and_replays_only_for_an_exact_materialized_context_commit() {
    let context_id = ContextId::from_uuid(uuid(10_001));
    let repository = repository(context_id);
    let commit_id = persist_source_commit(&repository, context_id, Vec::new()).await;
    let binding = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(20_001)),
        workflow_definition(30_001, 1),
        ContextCommitSource::new(context_id, commit_id),
    );

    let first = repository
        .persist_workflow_context_binding(binding.clone())
        .await
        .expect("persist binding");
    let replay = repository
        .persist_workflow_context_binding(binding.clone())
        .await
        .expect("identical binding replay");

    assert_eq!(
        first.disposition(),
        WorkflowContextBindingWriteDisposition::Created
    );
    assert_eq!(
        replay.disposition(),
        WorkflowContextBindingWriteDisposition::Replayed
    );
    assert_eq!(
        repository
            .get_workflow_context_binding(binding.workflow_id(), binding.workflow_revision())
            .await
            .expect("read binding"),
        Some(binding.clone())
    );
    assert_eq!(
        repository
            .list_workflow_context_bindings_at_commit(context_id, commit_id)
            .await
            .expect("list exact source bindings"),
        vec![binding]
    );

    let later_commit_id = persist_source_commit(&repository, context_id, vec![commit_id]).await;
    assert!(
        repository
            .list_workflow_context_bindings_at_commit(context_id, later_commit_id)
            .await
            .expect("read later exact source scope")
            .is_empty()
    );
}

#[tokio::test]
async fn binding_rejects_a_missing_context_snapshot_and_conflicting_workflow_revision() {
    let context_id = ContextId::from_uuid(uuid(10_101));
    let repository = repository(context_id);
    let first_commit_id = persist_source_commit(&repository, context_id, Vec::new()).await;
    let binding = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(20_101)),
        workflow_definition(30_101, 1),
        ContextCommitSource::new(context_id, first_commit_id),
    );
    repository
        .persist_workflow_context_binding(binding.clone())
        .await
        .expect("persist initial binding");

    let missing_snapshot = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(20_102)),
        workflow_definition(30_102, 1),
        ContextCommitSource::new(context_id, CommitId::from_uuid(uuid(10_102))),
    );
    assert!(
        repository
            .persist_workflow_context_binding(missing_snapshot)
            .await
            .is_err()
    );

    let second_commit_id =
        persist_source_commit(&repository, context_id, vec![first_commit_id]).await;
    let conflicting = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(20_103)),
        workflow_definition(30_101, 1),
        ContextCommitSource::new(context_id, second_commit_id),
    );
    assert!(
        repository
            .persist_workflow_context_binding(conflicting)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn binding_list_uses_canonical_workflow_order_independent_of_insert_order() {
    let context_id = ContextId::from_uuid(uuid(10_201));
    let repository = repository(context_id);
    let commit_id = persist_source_commit(&repository, context_id, Vec::new()).await;
    let source = ContextCommitSource::new(context_id, commit_id);
    let later = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(20_202)),
        workflow_definition(30_202, 1),
        source,
    );
    let earlier = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(uuid(20_201)),
        workflow_definition(30_201, 1),
        source,
    );

    repository
        .persist_workflow_context_binding(later.clone())
        .await
        .expect("persist later workflow binding first");
    repository
        .persist_workflow_context_binding(earlier.clone())
        .await
        .expect("persist earlier workflow binding second");

    assert_eq!(
        repository
            .list_workflow_context_bindings_at_commit(context_id, commit_id)
            .await
            .expect("list canonical bindings"),
        vec![earlier, later]
    );
}
