//! Workspace listing contracts for storage-backed API queries.

use crate::StorageRepositoryError;
use crate::listing::{
    ListPagination, normalize_page, normalize_per_page, normalize_search, offset,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// Query options for listing workspaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceListQuery {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Optional case-insensitive search term.
    pub search: Option<String>,
    /// Sort order.
    pub sort: WorkspaceSort,
}

impl WorkspaceListQuery {
    /// Creates a normalized query from raw values.
    #[must_use]
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        sort: WorkspaceSort,
    ) -> Self {
        let page = normalize_page(page);
        let per_page = normalize_per_page(per_page);
        let search = normalize_search(search);

        Self {
            page,
            per_page,
            search,
            sort,
        }
    }

    /// Returns the SQL offset for this query.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        offset(self.page, self.per_page)
    }
}

impl Default for WorkspaceListQuery {
    fn default() -> Self {
        Self::new(None, None, None, WorkspaceSort::default())
    }
}

/// Supported workspace sort orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSort {
    /// Sort by name ascending.
    #[default]
    NameAsc,
    /// Sort by name descending.
    NameDesc,
    /// Sort by creation time ascending.
    CreatedAtAsc,
    /// Sort by creation time descending.
    CreatedAtDesc,
}

impl WorkspaceSort {
    /// Returns the public query parameter value.
    #[must_use]
    pub const fn as_query_value(self) -> &'static str {
        match self {
            Self::NameAsc => "name",
            Self::NameDesc => "-name",
            Self::CreatedAtAsc => "created_at",
            Self::CreatedAtDesc => "-created_at",
        }
    }

    /// Returns a safe SQL `ORDER BY` clause for this sort order.
    #[must_use]
    pub const fn order_by_sql(self) -> &'static str {
        match self {
            Self::NameAsc => "name ASC, id ASC",
            Self::NameDesc => "name DESC, id ASC",
            Self::CreatedAtAsc => "created_at ASC, id ASC",
            Self::CreatedAtDesc => "created_at DESC, id ASC",
        }
    }
}

impl FromStr for WorkspaceSort {
    type Err = WorkspaceSortParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "" | "name" => Ok(Self::NameAsc),
            "-name" => Ok(Self::NameDesc),
            "created_at" => Ok(Self::CreatedAtAsc),
            "-created_at" => Ok(Self::CreatedAtDesc),
            other => Err(WorkspaceSortParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when a workspace sort query value is unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unsupported workspace sort: {value}")]
pub struct WorkspaceSortParseError {
    value: String,
}

/// A workspace list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceListItem {
    /// Workspace id.
    pub id: String,
    /// Workspace display name.
    pub name: String,
    /// Workspace slug.
    pub slug: String,
    /// Workspace creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Pagination metadata for workspace list APIs.
pub type WorkspaceListPagination = ListPagination;

/// Paginated workspace list result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceList {
    /// Matching workspace items.
    pub items: Vec<WorkspaceListItem>,
    /// Pagination metadata.
    pub pagination: WorkspaceListPagination,
}

impl WorkspaceList {
    /// Builds a list response from query metadata.
    #[must_use]
    pub const fn new(
        items: Vec<WorkspaceListItem>,
        query: &WorkspaceListQuery,
        total: u64,
    ) -> Self {
        Self {
            items,
            pagination: WorkspaceListPagination::new(query.page, query.per_page, total),
        }
    }
}

/// Repository contract for workspace list queries.
#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    /// Lists workspaces using pagination, filtering, and sorting.
    async fn list_workspaces(
        &self,
        query: WorkspaceListQuery,
    ) -> Result<WorkspaceList, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_PAGE, MAX_PER_PAGE};

    #[test]
    fn workspace_list_query_normalizes_raw_values() {
        let query = WorkspaceListQuery::new(
            Some(0),
            Some(MAX_PER_PAGE + 1),
            Some("  support  ".to_owned()),
            WorkspaceSort::NameDesc,
        );

        assert_eq!(query.page, DEFAULT_PAGE);
        assert_eq!(query.per_page, MAX_PER_PAGE);
        assert_eq!(query.search.as_deref(), Some("support"));
        assert_eq!(query.sort, WorkspaceSort::NameDesc);
    }

    #[test]
    fn workspace_list_query_offset_handles_large_pages_without_overflow() {
        let query = WorkspaceListQuery::new(
            Some(u32::MAX),
            Some(MAX_PER_PAGE),
            None,
            WorkspaceSort::CreatedAtDesc,
        );

        assert_eq!(
            query.offset(),
            (u64::from(u32::MAX) - 1) * u64::from(MAX_PER_PAGE)
        );
    }

    #[test]
    fn workspace_sort_parses_public_query_values() {
        assert_eq!("".parse::<WorkspaceSort>(), Ok(WorkspaceSort::NameAsc));
        assert_eq!("name".parse::<WorkspaceSort>(), Ok(WorkspaceSort::NameAsc));
        assert_eq!(
            "-name".parse::<WorkspaceSort>(),
            Ok(WorkspaceSort::NameDesc)
        );
        assert_eq!(
            "created_at".parse::<WorkspaceSort>(),
            Ok(WorkspaceSort::CreatedAtAsc)
        );
        assert_eq!(
            "-created_at".parse::<WorkspaceSort>(),
            Ok(WorkspaceSort::CreatedAtDesc)
        );
        assert!("updated_at".parse::<WorkspaceSort>().is_err());
    }
}
