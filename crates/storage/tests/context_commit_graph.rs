//! Contract tests for the exact Context commit graph repository.

use chrono::Utc;
use contextlab_context_core::ContextId;
use contextlab_storage::{
    ContextCommitGraphRepository, ContextCommitRecord, ContextGraphProjection, ContextRecord,
    InMemoryContextGraphRepository, ProjectRecord,
};
use contextlab_versioning::CommitId;
use serde_json::json;
use uuid::Uuid;

fn id(value: &str) -> String {
    value.to_owned()
}

fn commit(id_value: &str, context_id: &str, parents: &[&str]) -> ContextCommitRecord {
    ContextCommitRecord {
        id: id(id_value),
        context_id: id(context_id),
        branch_name: "main".to_owned(),
        message: "commit".to_owned(),
        parent_commit_ids: parents.iter().map(|value| id(value)).collect(),
        changes: json!([]),
        change_count: 0,
        authored_at: Utc::now(),
        created_at: Utc::now(),
    }
}

#[tokio::test]
async fn memory_repository_returns_the_complete_exact_context_dag() {
    let project_id = "11111111-1111-4111-8111-111111111111";
    let context_id = "22222222-2222-4222-8222-222222222222";
    let root = "33333333-3333-4333-8333-333333333333";
    let left = "44444444-4444-4444-8444-444444444444";
    let right = "55555555-5555-4555-8555-555555555555";
    let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: id(project_id),
            workspace_id: "workspace".to_owned(),
            name: "Project".to_owned(),
            slug: "project".to_owned(),
            created_at: Utc::now(),
        }],
        contexts: vec![ContextRecord {
            id: id(context_id),
            project_id: id(project_id),
            experiment_id: None,
            name: "Context".to_owned(),
            description: None,
            created_at: Utc::now(),
        }],
        commits: vec![
            commit(root, context_id, &[]),
            commit(left, context_id, &[root]),
            commit(right, context_id, &[root]),
        ],
        ..ContextGraphProjection::default()
    });

    let graph = repository
        .load_context_commit_graph(ContextId::from_uuid(
            Uuid::parse_str(context_id).expect("context"),
        ))
        .await
        .expect("complete graph");

    assert!(
        graph
            .node(CommitId::from_uuid(Uuid::parse_str(left).expect("left")))
            .is_some()
    );
    assert!(
        graph
            .node(CommitId::from_uuid(Uuid::parse_str(right).expect("right")))
            .is_some()
    );
}

#[tokio::test]
async fn memory_repository_fails_closed_for_an_invalid_persisted_commit_id() {
    let context_id = "22222222-2222-4222-8222-222222222222";
    let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
        contexts: vec![ContextRecord {
            id: id(context_id),
            project_id: "11111111-1111-4111-8111-111111111111".to_owned(),
            experiment_id: None,
            name: "Context".to_owned(),
            description: None,
            created_at: Utc::now(),
        }],
        commits: vec![commit("not-a-uuid", context_id, &[])],
        ..ContextGraphProjection::default()
    });

    let error = repository
        .load_context_commit_graph(ContextId::from_uuid(
            Uuid::parse_str(context_id).expect("context"),
        ))
        .await
        .expect_err("invalid graph");

    assert!(error.to_string().contains("invalid commit id"));
}
