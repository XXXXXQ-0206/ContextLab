//! Context commit listing contracts for storage-backed API queries.

use crate::StorageRepositoryError;
use crate::listing::{
    ListPagination, normalize_page, normalize_per_page, normalize_search, offset,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use thiserror::Error;

/// Query options for listing commits inside one context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitListQuery {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Optional case-insensitive search term.
    pub search: Option<String>,
    /// Optional exact branch name filter.
    pub branch_name: Option<String>,
    /// Sort order.
    pub sort: CommitSort,
}

impl CommitListQuery {
    /// Creates a normalized query from raw values.
    #[must_use]
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        branch_name: Option<String>,
        sort: CommitSort,
    ) -> Self {
        let page = normalize_page(page);
        let per_page = normalize_per_page(per_page);
        let search = normalize_search(search);
        let branch_name = normalize_search(branch_name);

        Self {
            page,
            per_page,
            search,
            branch_name,
            sort,
        }
    }

    /// Returns the SQL offset for this query.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        offset(self.page, self.per_page)
    }
}

impl Default for CommitListQuery {
    fn default() -> Self {
        Self::new(None, None, None, None, CommitSort::default())
    }
}

/// Supported commit sort orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitSort {
    /// Sort by authored time ascending.
    AuthoredAtAsc,
    /// Sort by authored time descending.
    #[default]
    AuthoredAtDesc,
    /// Sort by creation time ascending.
    CreatedAtAsc,
    /// Sort by creation time descending.
    CreatedAtDesc,
    /// Sort by branch name ascending.
    BranchNameAsc,
    /// Sort by branch name descending.
    BranchNameDesc,
}

impl CommitSort {
    /// Returns the public query parameter value.
    #[must_use]
    pub const fn as_query_value(self) -> &'static str {
        match self {
            Self::AuthoredAtAsc => "authored_at",
            Self::AuthoredAtDesc => "-authored_at",
            Self::CreatedAtAsc => "created_at",
            Self::CreatedAtDesc => "-created_at",
            Self::BranchNameAsc => "branch_name",
            Self::BranchNameDesc => "-branch_name",
        }
    }

    /// Returns a safe SQL `ORDER BY` clause for this sort order.
    #[must_use]
    pub const fn order_by_sql(self) -> &'static str {
        match self {
            Self::AuthoredAtAsc => "context_commits.authored_at ASC, context_commits.id ASC",
            Self::AuthoredAtDesc => "context_commits.authored_at DESC, context_commits.id ASC",
            Self::CreatedAtAsc => "context_commits.created_at ASC, context_commits.id ASC",
            Self::CreatedAtDesc => "context_commits.created_at DESC, context_commits.id ASC",
            Self::BranchNameAsc => "context_commits.branch_name ASC, context_commits.id ASC",
            Self::BranchNameDesc => "context_commits.branch_name DESC, context_commits.id ASC",
        }
    }
}

impl FromStr for CommitSort {
    type Err = CommitSortParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "" | "-authored_at" => Ok(Self::AuthoredAtDesc),
            "authored_at" => Ok(Self::AuthoredAtAsc),
            "created_at" => Ok(Self::CreatedAtAsc),
            "-created_at" => Ok(Self::CreatedAtDesc),
            "branch_name" => Ok(Self::BranchNameAsc),
            "-branch_name" => Ok(Self::BranchNameDesc),
            other => Err(CommitSortParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when a commit sort query value is unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unsupported commit sort: {value}")]
pub struct CommitSortParseError {
    value: String,
}

/// A commit list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitListItem {
    /// Commit id.
    pub id: String,
    /// Parent context id.
    pub context_id: String,
    /// Branch name associated with the commit.
    pub branch_name: String,
    /// Commit message.
    pub message: String,
    /// Ordered parent commit ids.
    pub parent_commit_ids: Vec<String>,
    /// Number of replayable changes stored on the commit.
    pub change_count: u32,
    /// Authored timestamp.
    pub authored_at: DateTime<Utc>,
    /// Commit row creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Detailed commit payload, including replayable changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitDetail {
    /// Commit id.
    pub id: String,
    /// Parent context id.
    pub context_id: String,
    /// Branch name associated with the commit.
    pub branch_name: String,
    /// Commit message.
    pub message: String,
    /// Ordered parent commit ids.
    pub parent_commit_ids: Vec<String>,
    /// Replayable changes stored on the commit.
    pub changes: Value,
    /// Number of replayable changes stored on the commit.
    pub change_count: u32,
    /// Authored timestamp.
    pub authored_at: DateTime<Utc>,
    /// Commit row creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Pagination metadata for commit list APIs.
pub type CommitListPagination = ListPagination;

/// Paginated commit list result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitList {
    /// Matching commit items.
    pub items: Vec<CommitListItem>,
    /// Pagination metadata.
    pub pagination: CommitListPagination,
}

impl CommitList {
    /// Builds a list response from query metadata.
    #[must_use]
    pub const fn new(items: Vec<CommitListItem>, query: &CommitListQuery, total: u64) -> Self {
        Self {
            items,
            pagination: CommitListPagination::new(query.page, query.per_page, total),
        }
    }
}

/// Repository contract for context-scoped commit list queries.
#[async_trait]
pub trait ContextCommitRepository: Send + Sync {
    /// Lists commits for one context using pagination, filtering, and sorting.
    async fn list_commits(
        &self,
        context_id: String,
        query: CommitListQuery,
    ) -> Result<CommitList, StorageRepositoryError>;

    /// Returns one commit for one context.
    async fn get_commit(
        &self,
        context_id: String,
        commit_id: String,
    ) -> Result<CommitDetail, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_PAGE, MAX_PER_PAGE};

    #[test]
    fn commit_list_query_normalizes_raw_values() {
        let query = CommitListQuery::new(
            Some(0),
            Some(MAX_PER_PAGE + 1),
            Some("  create  ".to_owned()),
            Some("  main  ".to_owned()),
            CommitSort::BranchNameDesc,
        );

        assert_eq!(query.page, DEFAULT_PAGE);
        assert_eq!(query.per_page, MAX_PER_PAGE);
        assert_eq!(query.search.as_deref(), Some("create"));
        assert_eq!(query.branch_name.as_deref(), Some("main"));
        assert_eq!(query.sort, CommitSort::BranchNameDesc);
    }

    #[test]
    fn commit_list_query_offset_handles_large_pages_without_overflow() {
        let query = CommitListQuery::new(
            Some(u32::MAX),
            Some(MAX_PER_PAGE),
            None,
            None,
            CommitSort::AuthoredAtDesc,
        );

        assert_eq!(
            query.offset(),
            (u64::from(u32::MAX) - 1) * u64::from(MAX_PER_PAGE)
        );
    }

    #[test]
    fn commit_sort_parses_public_query_values() {
        assert_eq!(
            "authored_at".parse::<CommitSort>(),
            Ok(CommitSort::AuthoredAtAsc)
        );
        assert_eq!(
            "-authored_at".parse::<CommitSort>(),
            Ok(CommitSort::AuthoredAtDesc)
        );
        assert_eq!(
            "created_at".parse::<CommitSort>(),
            Ok(CommitSort::CreatedAtAsc)
        );
        assert_eq!(
            "-created_at".parse::<CommitSort>(),
            Ok(CommitSort::CreatedAtDesc)
        );
        assert_eq!(
            "branch_name".parse::<CommitSort>(),
            Ok(CommitSort::BranchNameAsc)
        );
        assert_eq!(
            "-branch_name".parse::<CommitSort>(),
            Ok(CommitSort::BranchNameDesc)
        );
        assert!("message".parse::<CommitSort>().is_err());
    }
}
