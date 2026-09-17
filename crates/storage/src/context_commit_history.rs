//! Read-only composition of durable commits and branch heads into versioning history.

use crate::replay_state_at_commit::context_commit_from_record;
use crate::{
    CommitDetail, CommitListQuery, CommitSort, ContextBranchHead, ContextBranchRepository,
    ContextCommitRecord, ContextCommitRepository, MAX_PER_PAGE, StorageRepositoryError,
};
use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_versioning::{BranchHead, CommitHistory};

/// Repository boundary for a complete, validated Context commit history.
///
/// Concrete repositories provide a backend-owned consistent read. The generic
/// adapter below remains useful for isolated test doubles, but it cannot make
/// separate ports atomic.
///
/// This is an application read boundary. It composes the existing commit and
/// branch-head ports without moving `CommitHistory` ownership out of the
/// versioning crate.
#[async_trait]
pub trait ContextCommitHistoryRepository: Send + Sync {
    /// Loads all commits and explicit branch heads for one Context.
    async fn load_context_commit_history(
        &self,
        context_id: ContextId,
    ) -> Result<CommitHistory, StorageRepositoryError>;
}

/// Adapter that composes separately owned commit and branch-head ports.
#[derive(Clone, Copy)]
pub struct ContextCommitHistoryRepositoryAdapter<'repository> {
    commits: &'repository dyn ContextCommitRepository,
    branch_heads: &'repository dyn ContextBranchRepository,
}

impl<'repository> ContextCommitHistoryRepositoryAdapter<'repository> {
    /// Creates a history adapter over the existing storage read ports.
    #[must_use]
    pub const fn new(
        commits: &'repository dyn ContextCommitRepository,
        branch_heads: &'repository dyn ContextBranchRepository,
    ) -> Self {
        Self {
            commits,
            branch_heads,
        }
    }
}

#[async_trait]
impl ContextCommitHistoryRepository for ContextCommitHistoryRepositoryAdapter<'_> {
    async fn load_context_commit_history(
        &self,
        context_id: ContextId,
    ) -> Result<CommitHistory, StorageRepositoryError> {
        load_context_commit_history_from_ports(self.commits, self.branch_heads, context_id).await
    }
}

/// Compose the existing typed repository ports into the reusable versioning contract.
///
/// The list/detail calls are intentionally routed through the established
/// repository contracts so Memory and PostgreSQL adapters share the same
/// rehydration and fail-closed validation rules. A future atomic read can
/// replace this adapter without changing consumers.
async fn load_context_commit_history_from_ports<C, B>(
    commits_repository: &C,
    branch_heads_repository: &B,
    context_id: ContextId,
) -> Result<CommitHistory, StorageRepositoryError>
where
    C: ContextCommitRepository + ?Sized,
    B: ContextBranchRepository + ?Sized,
{
    let mut page = 1_u32;
    let mut commits = Vec::new();
    let mut total = None;

    loop {
        let listed = commits_repository
            .list_commits(
                context_id.to_string(),
                CommitListQuery::new(
                    Some(page),
                    Some(MAX_PER_PAGE),
                    None,
                    None,
                    CommitSort::AuthoredAtAsc,
                ),
            )
            .await?;
        let expected_total = *total.get_or_insert(listed.pagination.total);
        if listed.pagination.total != expected_total {
            return Err(history_scope_error(
                context_id,
                "commit count changed while loading history",
            ));
        }
        if listed.items.is_empty() && commits.len() < expected_total as usize {
            return Err(history_scope_error(
                context_id,
                "commit list ended before its declared total",
            ));
        }

        for item in listed.items {
            let detail = commits_repository
                .get_commit(context_id.to_string(), item.id.clone())
                .await?;
            commits.push(detail);
        }

        if commits.len() >= expected_total as usize {
            break;
        }
        page = page
            .checked_add(1)
            .ok_or_else(|| history_scope_error(context_id, "commit history pagination overflow"))?;
    }

    let heads = branch_heads_repository
        .list_context_branch_heads(context_id)
        .await
        .map_err(|error| history_scope_error(context_id, error.to_string()))?
        .into_iter()
        .collect::<Vec<_>>();

    assemble_context_commit_history(context_id, commits, heads)
}

/// Rehydrates a complete history after all backend reads share one boundary.
pub(crate) fn assemble_context_commit_history(
    context_id: ContextId,
    details: impl IntoIterator<Item = CommitDetail>,
    heads: impl IntoIterator<Item = ContextBranchHead>,
) -> Result<CommitHistory, StorageRepositoryError> {
    let commits = details
        .into_iter()
        .map(|detail| context_commit_from_detail(context_id, detail))
        .collect::<Result<Vec<_>, _>>()?;
    let heads = heads
        .into_iter()
        .map(branch_head_from_storage)
        .collect::<Result<Vec<_>, _>>()?;
    CommitHistory::try_from_parts(context_id, commits, heads)
        .map_err(|error| history_scope_error(context_id, error.to_string()))
}

pub(crate) fn assemble_context_commit_history_from_commits(
    context_id: ContextId,
    commits: impl IntoIterator<Item = contextlab_versioning::ContextCommit>,
    heads: impl IntoIterator<Item = BranchHead>,
) -> Result<CommitHistory, StorageRepositoryError> {
    CommitHistory::try_from_parts(context_id, commits, heads)
        .map_err(|error| history_scope_error(context_id, error.to_string()))
}

fn context_commit_from_detail(
    expected_context_id: ContextId,
    detail: CommitDetail,
) -> Result<contextlab_versioning::ContextCommit, StorageRepositoryError> {
    let record = ContextCommitRecord {
        id: detail.id,
        context_id: detail.context_id,
        branch_name: detail.branch_name,
        message: detail.message,
        parent_commit_ids: detail.parent_commit_ids,
        changes: detail.changes,
        change_count: detail.change_count,
        authored_at: detail.authored_at,
        created_at: detail.created_at,
    };
    let commit = context_commit_from_record(record)?;
    if commit.context_id() != expected_context_id {
        return Err(history_scope_error(
            expected_context_id,
            "commit detail crossed the requested Context scope",
        ));
    }
    Ok(commit)
}

fn branch_head_from_storage(head: ContextBranchHead) -> Result<BranchHead, StorageRepositoryError> {
    Ok(BranchHead::new(
        head.branch().clone(),
        head.head_commit_id(),
    ))
}

fn history_scope_error(context_id: ContextId, reason: impl Into<String>) -> StorageRepositoryError {
    StorageRepositoryError::InvalidScope {
        scope: format!("context_commit_history:{context_id}"),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CommitList, CommitListItem, ContextBranchHead, ContextBranchRepositoryError,
        ContextCommitRepository, StorageRepositoryError,
    };
    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use contextlab_versioning::{BranchName, CommitId};
    use serde_json::json;
    use std::collections::BTreeMap;
    use uuid::Uuid;

    #[derive(Default)]
    struct FakeRepository {
        commits: BTreeMap<String, CommitDetail>,
        heads: Vec<ContextBranchHead>,
    }

    #[async_trait]
    impl ContextCommitRepository for FakeRepository {
        async fn list_commits(
            &self,
            _context_id: String,
            query: CommitListQuery,
        ) -> Result<CommitList, StorageRepositoryError> {
            let items = self
                .commits
                .values()
                .map(|detail| CommitListItem {
                    id: detail.id.clone(),
                    context_id: detail.context_id.clone(),
                    branch_name: detail.branch_name.clone(),
                    message: detail.message.clone(),
                    parent_commit_ids: detail.parent_commit_ids.clone(),
                    change_count: detail.change_count,
                    authored_at: detail.authored_at,
                    created_at: detail.created_at,
                })
                .collect::<Vec<_>>();
            Ok(CommitList::new(items, &query, self.commits.len() as u64))
        }

        async fn get_commit(
            &self,
            _context_id: String,
            commit_id: String,
        ) -> Result<CommitDetail, StorageRepositoryError> {
            self.commits
                .get(&commit_id)
                .cloned()
                .ok_or_else(|| StorageRepositoryError::ScopeUnavailable { scope: commit_id })
        }
    }

    #[async_trait]
    impl ContextBranchRepository for FakeRepository {
        async fn list_context_branch_heads(
            &self,
            _context_id: contextlab_context_core::ContextId,
        ) -> Result<Vec<ContextBranchHead>, ContextBranchRepositoryError> {
            Ok(self.heads.clone())
        }

        async fn get_context_branch_head(
            &self,
            _context_id: contextlab_context_core::ContextId,
            _branch: BranchName,
        ) -> Result<ContextBranchHead, ContextBranchRepositoryError> {
            Err(ContextBranchRepositoryError::UnknownBranch {
                context_id: contextlab_context_core::ContextId::new(),
                branch: BranchName::default(),
            })
        }
    }

    fn detail(id: Uuid, context_id: Uuid, parents: Vec<Uuid>, message: &str) -> CommitDetail {
        CommitDetail {
            id: id.to_string(),
            context_id: context_id.to_string(),
            branch_name: "main".to_owned(),
            message: message.to_owned(),
            parent_commit_ids: parents.into_iter().map(|id| id.to_string()).collect(),
            changes: json!([]),
            change_count: 0,
            authored_at: Utc.timestamp_opt(1, 0).single().expect("timestamp"),
            created_at: Utc.timestamp_opt(1, 0).single().expect("timestamp"),
        }
    }

    #[tokio::test]
    async fn composes_complete_commits_and_branch_heads_into_versioning_history() {
        let context_uuid = Uuid::from_u128(1);
        let root_uuid = Uuid::from_u128(2);
        let child_uuid = Uuid::from_u128(3);
        let repository = FakeRepository {
            commits: BTreeMap::from([
                (
                    root_uuid.to_string(),
                    detail(root_uuid, context_uuid, vec![], "root"),
                ),
                (
                    child_uuid.to_string(),
                    detail(child_uuid, context_uuid, vec![root_uuid], "child"),
                ),
            ]),
            heads: vec![ContextBranchHead::new(
                contextlab_context_core::ContextId::from_uuid(context_uuid),
                BranchName::default(),
                Some(CommitId::from_uuid(child_uuid)),
                2,
            )],
        };

        let history = ContextCommitHistoryRepositoryAdapter::new(&repository, &repository)
            .load_context_commit_history(contextlab_context_core::ContextId::from_uuid(
                context_uuid,
            ))
            .await
            .expect("history");

        assert_eq!(history.context_id().as_uuid(), context_uuid);
        assert_eq!(
            history
                .commit(CommitId::from_uuid(root_uuid))
                .unwrap()
                .message(),
            "root"
        );
        assert_eq!(
            history.head(&BranchName::default()),
            Some(CommitId::from_uuid(child_uuid))
        );
    }

    #[tokio::test]
    async fn rejects_malformed_persisted_changes_before_history_is_exposed() {
        let context_uuid = Uuid::from_u128(1);
        let commit_uuid = Uuid::from_u128(2);
        let mut repository = FakeRepository::default();
        let mut malformed = detail(commit_uuid, context_uuid, vec![], "root");
        malformed.changes = json!({"not": "an array"});
        repository
            .commits
            .insert(commit_uuid.to_string(), malformed);
        repository.heads.push(ContextBranchHead::new(
            contextlab_context_core::ContextId::from_uuid(context_uuid),
            BranchName::default(),
            Some(CommitId::from_uuid(commit_uuid)),
            1,
        ));

        let error = ContextCommitHistoryRepositoryAdapter::new(&repository, &repository)
            .load_context_commit_history(contextlab_context_core::ContextId::from_uuid(
                context_uuid,
            ))
            .await
            .expect_err("malformed changes");
        assert!(matches!(
            error,
            StorageRepositoryError::ComponentStateReplayConflict { .. }
        ));
    }
}
