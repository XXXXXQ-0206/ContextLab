//! Exact Context commit-DAG loading for versioning and merge consumers.

use crate::{ContextCommitRecord, StorageRepositoryError};
use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_versioning::{CommitGraph, CommitGraphNode, CommitId};
use uuid::Uuid;

/// Repository contract for loading one complete Context commit DAG.
///
/// Implementations must return only commits owned by `context_id`. Consumers can
/// then resolve ancestry with the reusable versioning crate instead of trusting
/// caller-supplied merge plans.
#[async_trait]
pub trait ContextCommitGraphRepository: Send + Sync {
    /// Loads and validates the complete commit graph for one Context.
    async fn load_context_commit_graph(
        &self,
        context_id: ContextId,
    ) -> Result<CommitGraph, StorageRepositoryError>;
}

/// Converts persisted commit rows into the reusable validated versioning graph.
pub(crate) fn commit_graph_from_records(
    context_id: ContextId,
    records: impl IntoIterator<Item = ContextCommitRecord>,
) -> Result<CommitGraph, StorageRepositoryError> {
    let expected_context = context_id.to_string();
    let mut nodes = Vec::new();
    for record in records {
        if record.context_id != expected_context {
            return Err(invalid_graph(
                context_id,
                format!(
                    "commit {} belongs to context {}",
                    record.id, record.context_id
                ),
            ));
        }
        let id = parse_commit_id(context_id, &record.id, "commit")?;
        let parent_ids = record
            .parent_commit_ids
            .iter()
            .map(|parent_id| parse_commit_id(context_id, parent_id, "parent"))
            .collect::<Result<Vec<_>, _>>()?;
        nodes.push(CommitGraphNode::new(id, context_id, parent_ids));
    }

    commit_graph_from_nodes(context_id, nodes)
}

pub(crate) fn commit_graph_from_nodes(
    context_id: ContextId,
    nodes: impl IntoIterator<Item = CommitGraphNode>,
) -> Result<CommitGraph, StorageRepositoryError> {
    CommitGraph::try_from_nodes(nodes).map_err(|error| invalid_graph(context_id, error.to_string()))
}

fn parse_commit_id(
    context_id: ContextId,
    raw: &str,
    family: &str,
) -> Result<CommitId, StorageRepositoryError> {
    Uuid::parse_str(raw)
        .map(CommitId::from_uuid)
        .map_err(|error| invalid_graph(context_id, format!("invalid {family} id {raw}: {error}")))
}

pub(crate) fn invalid_graph(
    context_id: ContextId,
    reason: impl Into<String>,
) -> StorageRepositoryError {
    StorageRepositoryError::InvalidScope {
        scope: format!("commit_graph:{context_id}"),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    fn record(id: &str, context_id: &str, parents: &[&str]) -> ContextCommitRecord {
        ContextCommitRecord {
            id: id.to_owned(),
            context_id: context_id.to_owned(),
            branch_name: "main".to_owned(),
            message: "test commit".to_owned(),
            parent_commit_ids: parents.iter().map(|value| (*value).to_owned()).collect(),
            changes: json!([]),
            change_count: 0,
            authored_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn builds_a_complete_exact_context_graph() {
        let context_id = ContextId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("context"),
        );
        let root = "22222222-2222-4222-8222-222222222222";
        let child = "33333333-3333-4333-8333-333333333333";
        let graph = commit_graph_from_records(
            context_id,
            [
                record(root, &context_id.to_string(), &[]),
                record(child, &context_id.to_string(), &[root]),
            ],
        )
        .expect("valid graph");

        assert!(
            graph
                .node(CommitId::from_uuid(Uuid::parse_str(child).expect("child")))
                .is_some()
        );
    }

    #[test]
    fn rejects_invalid_parent_identity_before_versioning() {
        let context_id = ContextId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("context"),
        );
        let error = commit_graph_from_records(
            context_id,
            [record(
                "22222222-2222-4222-8222-222222222222",
                &context_id.to_string(),
                &["not-a-uuid"],
            )],
        )
        .expect_err("invalid parent");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
        assert!(error.to_string().contains("invalid parent id"));
    }

    #[test]
    fn rejects_duplicate_and_missing_parent_history() {
        let context_id = ContextId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("context"),
        );
        let duplicate = record(
            "22222222-2222-4222-8222-222222222222",
            &context_id.to_string(),
            &[],
        );
        let error = commit_graph_from_records(context_id, [duplicate.clone(), duplicate])
            .expect_err("duplicate commit");
        assert!(error.to_string().contains("duplicate commit"));

        let error = commit_graph_from_records(
            context_id,
            [record(
                "33333333-3333-4333-8333-333333333333",
                &context_id.to_string(),
                &["44444444-4444-4444-8444-444444444444"],
            )],
        )
        .expect_err("missing parent");
        assert!(error.to_string().contains("missing parent"));
    }
}
