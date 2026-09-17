//! Context listing contracts for storage-backed API queries.

use crate::StorageRepositoryError;
use crate::listing::{
    ListPagination, normalize_page, normalize_per_page, normalize_search, offset,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// Query options for listing contexts inside one project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextListQuery {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Optional case-insensitive search term.
    pub search: Option<String>,
    /// Optional experiment id filter.
    pub experiment_id: Option<String>,
    /// Sort order.
    pub sort: ContextSort,
}

impl ContextListQuery {
    /// Creates a normalized query from raw values.
    #[must_use]
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        experiment_id: Option<String>,
        sort: ContextSort,
    ) -> Self {
        let page = normalize_page(page);
        let per_page = normalize_per_page(per_page);
        let search = normalize_search(search);
        let experiment_id = normalize_search(experiment_id);

        Self {
            page,
            per_page,
            search,
            experiment_id,
            sort,
        }
    }

    /// Returns the SQL offset for this query.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        offset(self.page, self.per_page)
    }
}

impl Default for ContextListQuery {
    fn default() -> Self {
        Self::new(None, None, None, None, ContextSort::default())
    }
}

/// Supported context sort orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSort {
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

impl ContextSort {
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

impl FromStr for ContextSort {
    type Err = ContextSortParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "" | "name" => Ok(Self::NameAsc),
            "-name" => Ok(Self::NameDesc),
            "created_at" => Ok(Self::CreatedAtAsc),
            "-created_at" => Ok(Self::CreatedAtDesc),
            other => Err(ContextSortParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when a context sort query value is unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unsupported context sort: {value}")]
pub struct ContextSortParseError {
    value: String,
}

/// A context list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextListItem {
    /// Context id.
    pub id: String,
    /// Parent project id.
    pub project_id: String,
    /// Optional tracked experiment id.
    pub experiment_id: Option<String>,
    /// Context display name.
    pub name: String,
    /// Optional context description.
    pub description: Option<String>,
    /// Context creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Pagination metadata for context list APIs.
pub type ContextListPagination = ListPagination;

/// Paginated context list result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextList {
    /// Matching context items.
    pub items: Vec<ContextListItem>,
    /// Pagination metadata.
    pub pagination: ContextListPagination,
}

impl ContextList {
    /// Builds a list response from query metadata.
    #[must_use]
    pub const fn new(items: Vec<ContextListItem>, query: &ContextListQuery, total: u64) -> Self {
        Self {
            items,
            pagination: ContextListPagination::new(query.page, query.per_page, total),
        }
    }
}

/// Repository contract for project-scoped context list queries.
#[async_trait]
pub trait ContextRepository: Send + Sync {
    /// Lists contexts for one project using pagination, filtering, and sorting.
    async fn list_contexts(
        &self,
        project_id: String,
        query: ContextListQuery,
    ) -> Result<ContextList, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_PAGE, MAX_PER_PAGE};

    #[test]
    fn context_list_query_normalizes_raw_values() {
        let query = ContextListQuery::new(
            Some(0),
            Some(MAX_PER_PAGE + 1),
            Some("  support  ".to_owned()),
            Some("  rag-v2  ".to_owned()),
            ContextSort::NameDesc,
        );

        assert_eq!(query.page, DEFAULT_PAGE);
        assert_eq!(query.per_page, MAX_PER_PAGE);
        assert_eq!(query.search.as_deref(), Some("support"));
        assert_eq!(query.experiment_id.as_deref(), Some("rag-v2"));
        assert_eq!(query.sort, ContextSort::NameDesc);
    }

    #[test]
    fn context_list_query_offset_handles_large_pages_without_overflow() {
        let query = ContextListQuery::new(
            Some(u32::MAX),
            Some(MAX_PER_PAGE),
            None,
            None,
            ContextSort::CreatedAtDesc,
        );

        assert_eq!(
            query.offset(),
            (u64::from(u32::MAX) - 1) * u64::from(MAX_PER_PAGE)
        );
    }

    #[test]
    fn context_sort_parses_public_query_values() {
        assert_eq!("name".parse::<ContextSort>(), Ok(ContextSort::NameAsc));
        assert_eq!("-name".parse::<ContextSort>(), Ok(ContextSort::NameDesc));
        assert_eq!(
            "created_at".parse::<ContextSort>(),
            Ok(ContextSort::CreatedAtAsc)
        );
        assert_eq!(
            "-created_at".parse::<ContextSort>(),
            Ok(ContextSort::CreatedAtDesc)
        );
        assert!("updated_at".parse::<ContextSort>().is_err());
    }
}
