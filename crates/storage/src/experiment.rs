//! Experiment listing contracts for storage-backed API queries.

use crate::StorageRepositoryError;
use crate::listing::{
    ListPagination, normalize_page, normalize_per_page, normalize_search, offset,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// Query options for listing experiments inside one project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperimentListQuery {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Optional case-insensitive search term.
    pub search: Option<String>,
    /// Sort order.
    pub sort: ExperimentSort,
}

impl ExperimentListQuery {
    /// Creates a normalized query from raw values.
    #[must_use]
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        sort: ExperimentSort,
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

impl Default for ExperimentListQuery {
    fn default() -> Self {
        Self::new(None, None, None, ExperimentSort::default())
    }
}

/// Supported experiment sort orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentSort {
    /// Sort by name ascending.
    #[default]
    NameAsc,
    /// Sort by name descending.
    NameDesc,
    /// Sort by branch name ascending.
    BranchNameAsc,
    /// Sort by branch name descending.
    BranchNameDesc,
    /// Sort by creation time ascending.
    CreatedAtAsc,
    /// Sort by creation time descending.
    CreatedAtDesc,
}

impl ExperimentSort {
    /// Returns the public query parameter value.
    #[must_use]
    pub const fn as_query_value(self) -> &'static str {
        match self {
            Self::NameAsc => "name",
            Self::NameDesc => "-name",
            Self::BranchNameAsc => "branch_name",
            Self::BranchNameDesc => "-branch_name",
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
            Self::BranchNameAsc => "branch_name ASC, id ASC",
            Self::BranchNameDesc => "branch_name DESC, id ASC",
            Self::CreatedAtAsc => "created_at ASC, id ASC",
            Self::CreatedAtDesc => "created_at DESC, id ASC",
        }
    }
}

impl FromStr for ExperimentSort {
    type Err = ExperimentSortParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "" | "name" => Ok(Self::NameAsc),
            "-name" => Ok(Self::NameDesc),
            "branch_name" => Ok(Self::BranchNameAsc),
            "-branch_name" => Ok(Self::BranchNameDesc),
            "created_at" => Ok(Self::CreatedAtAsc),
            "-created_at" => Ok(Self::CreatedAtDesc),
            other => Err(ExperimentSortParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when an experiment sort query value is unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unsupported experiment sort: {value}")]
pub struct ExperimentSortParseError {
    value: String,
}

/// An experiment list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperimentListItem {
    /// Experiment id.
    pub id: String,
    /// Parent project id.
    pub project_id: String,
    /// Experiment display name.
    pub name: String,
    /// Versioning branch tracked by this experiment.
    pub branch_name: String,
    /// Experiment creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Pagination metadata for experiment list APIs.
pub type ExperimentListPagination = ListPagination;

/// Paginated experiment list result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperimentList {
    /// Matching experiment items.
    pub items: Vec<ExperimentListItem>,
    /// Pagination metadata.
    pub pagination: ExperimentListPagination,
}

impl ExperimentList {
    /// Builds a list response from query metadata.
    #[must_use]
    pub const fn new(
        items: Vec<ExperimentListItem>,
        query: &ExperimentListQuery,
        total: u64,
    ) -> Self {
        Self {
            items,
            pagination: ExperimentListPagination::new(query.page, query.per_page, total),
        }
    }
}

/// Repository contract for project-scoped experiment list queries.
#[async_trait]
pub trait ExperimentRepository: Send + Sync {
    /// Lists experiments for one project using pagination, filtering, and sorting.
    async fn list_experiments(
        &self,
        project_id: String,
        query: ExperimentListQuery,
    ) -> Result<ExperimentList, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_PAGE, MAX_PER_PAGE};

    #[test]
    fn experiment_list_query_normalizes_raw_values() {
        let query = ExperimentListQuery::new(
            Some(0),
            Some(MAX_PER_PAGE + 1),
            Some("  rag  ".to_owned()),
            ExperimentSort::BranchNameDesc,
        );

        assert_eq!(query.page, DEFAULT_PAGE);
        assert_eq!(query.per_page, MAX_PER_PAGE);
        assert_eq!(query.search.as_deref(), Some("rag"));
        assert_eq!(query.sort, ExperimentSort::BranchNameDesc);
    }

    #[test]
    fn experiment_list_query_offset_handles_large_pages_without_overflow() {
        let query = ExperimentListQuery::new(
            Some(u32::MAX),
            Some(MAX_PER_PAGE),
            None,
            ExperimentSort::CreatedAtDesc,
        );

        assert_eq!(
            query.offset(),
            (u64::from(u32::MAX) - 1) * u64::from(MAX_PER_PAGE)
        );
    }

    #[test]
    fn experiment_sort_parses_public_query_values() {
        assert_eq!(
            "name".parse::<ExperimentSort>(),
            Ok(ExperimentSort::NameAsc)
        );
        assert_eq!(
            "-name".parse::<ExperimentSort>(),
            Ok(ExperimentSort::NameDesc)
        );
        assert_eq!(
            "branch_name".parse::<ExperimentSort>(),
            Ok(ExperimentSort::BranchNameAsc)
        );
        assert_eq!(
            "-branch_name".parse::<ExperimentSort>(),
            Ok(ExperimentSort::BranchNameDesc)
        );
        assert_eq!(
            "created_at".parse::<ExperimentSort>(),
            Ok(ExperimentSort::CreatedAtAsc)
        );
        assert_eq!(
            "-created_at".parse::<ExperimentSort>(),
            Ok(ExperimentSort::CreatedAtDesc)
        );
        assert!("updated_at".parse::<ExperimentSort>().is_err());
    }
}
