//! Project listing contracts for storage-backed API queries.

use crate::StorageRepositoryError;
use crate::listing::{
    ListPagination, normalize_page, normalize_per_page, normalize_search, offset,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// Query options for listing projects inside one workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectListQuery {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Optional case-insensitive search term.
    pub search: Option<String>,
    /// Sort order.
    pub sort: ProjectSort,
}

impl ProjectListQuery {
    /// Creates a normalized query from raw values.
    #[must_use]
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        sort: ProjectSort,
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

impl Default for ProjectListQuery {
    fn default() -> Self {
        Self::new(None, None, None, ProjectSort::default())
    }
}

/// Supported project sort orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSort {
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

impl ProjectSort {
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

impl FromStr for ProjectSort {
    type Err = ProjectSortParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "" | "name" => Ok(Self::NameAsc),
            "-name" => Ok(Self::NameDesc),
            "created_at" => Ok(Self::CreatedAtAsc),
            "-created_at" => Ok(Self::CreatedAtDesc),
            other => Err(ProjectSortParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when a project sort query value is unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unsupported project sort: {value}")]
pub struct ProjectSortParseError {
    value: String,
}

/// A project list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectListItem {
    /// Project id.
    pub id: String,
    /// Parent workspace id.
    pub workspace_id: String,
    /// Project display name.
    pub name: String,
    /// Project slug.
    pub slug: String,
    /// Project creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Pagination metadata for project list APIs.
pub type ProjectListPagination = ListPagination;

/// Paginated project list result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectList {
    /// Matching project items.
    pub items: Vec<ProjectListItem>,
    /// Pagination metadata.
    pub pagination: ProjectListPagination,
}

impl ProjectList {
    /// Builds a list response from query metadata.
    #[must_use]
    pub const fn new(items: Vec<ProjectListItem>, query: &ProjectListQuery, total: u64) -> Self {
        Self {
            items,
            pagination: ProjectListPagination::new(query.page, query.per_page, total),
        }
    }
}

/// Repository contract for workspace-scoped project list queries.
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Lists projects for one workspace using pagination, filtering, and sorting.
    async fn list_projects(
        &self,
        workspace_id: String,
        query: ProjectListQuery,
    ) -> Result<ProjectList, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_PAGE, MAX_PER_PAGE};

    #[test]
    fn project_list_query_normalizes_raw_values() {
        let query = ProjectListQuery::new(
            Some(0),
            Some(MAX_PER_PAGE + 1),
            Some("  support  ".to_owned()),
            ProjectSort::NameDesc,
        );

        assert_eq!(query.page, DEFAULT_PAGE);
        assert_eq!(query.per_page, MAX_PER_PAGE);
        assert_eq!(query.search.as_deref(), Some("support"));
        assert_eq!(query.sort, ProjectSort::NameDesc);
    }

    #[test]
    fn project_list_query_offset_handles_large_pages_without_overflow() {
        let query = ProjectListQuery::new(
            Some(u32::MAX),
            Some(MAX_PER_PAGE),
            None,
            ProjectSort::CreatedAtDesc,
        );

        assert_eq!(
            query.offset(),
            (u64::from(u32::MAX) - 1) * u64::from(MAX_PER_PAGE)
        );
    }

    #[test]
    fn project_sort_parses_public_query_values() {
        assert_eq!("".parse::<ProjectSort>(), Ok(ProjectSort::NameAsc));
        assert_eq!("name".parse::<ProjectSort>(), Ok(ProjectSort::NameAsc));
        assert_eq!("-name".parse::<ProjectSort>(), Ok(ProjectSort::NameDesc));
        assert_eq!(
            "created_at".parse::<ProjectSort>(),
            Ok(ProjectSort::CreatedAtAsc)
        );
        assert_eq!(
            "-created_at".parse::<ProjectSort>(),
            Ok(ProjectSort::CreatedAtDesc)
        );
        assert!("updated_at".parse::<ProjectSort>().is_err());
    }
}
