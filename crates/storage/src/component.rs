//! Context component listing contracts for storage-backed API queries.

use crate::listing::{
    ListPagination, normalize_page, normalize_per_page, normalize_search, offset,
};
use crate::{StorageRepositoryError, StoredComponentKind};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use thiserror::Error;

/// Query options for listing components inside one context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentListQuery {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Optional case-insensitive search term.
    pub search: Option<String>,
    /// Optional exact component kind filter.
    pub kind: Option<StoredComponentKind>,
    /// Sort order.
    pub sort: ComponentSort,
}

impl ComponentListQuery {
    /// Creates a normalized query from raw values.
    #[must_use]
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        kind: Option<StoredComponentKind>,
        sort: ComponentSort,
    ) -> Self {
        let page = normalize_page(page);
        let per_page = normalize_per_page(per_page);
        let search = normalize_search(search);

        Self {
            page,
            per_page,
            search,
            kind,
            sort,
        }
    }

    /// Returns the SQL offset for this query.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        offset(self.page, self.per_page)
    }
}

impl Default for ComponentListQuery {
    fn default() -> Self {
        Self::new(None, None, None, None, ComponentSort::default())
    }
}

/// Supported component sort orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentSort {
    /// Sort by name ascending.
    NameAsc,
    /// Sort by name descending.
    NameDesc,
    /// Sort by component kind ascending.
    #[default]
    KindAsc,
    /// Sort by component kind descending.
    KindDesc,
    /// Sort by creation time ascending.
    CreatedAtAsc,
    /// Sort by creation time descending.
    CreatedAtDesc,
}

impl ComponentSort {
    /// Returns the public query parameter value.
    #[must_use]
    pub const fn as_query_value(self) -> &'static str {
        match self {
            Self::NameAsc => "name",
            Self::NameDesc => "-name",
            Self::KindAsc => "kind",
            Self::KindDesc => "-kind",
            Self::CreatedAtAsc => "created_at",
            Self::CreatedAtDesc => "-created_at",
        }
    }

    /// Returns a safe SQL `ORDER BY` clause for this sort order.
    #[must_use]
    pub const fn order_by_sql(self) -> &'static str {
        match self {
            Self::NameAsc => "context_components.name ASC, context_components.id ASC",
            Self::NameDesc => "context_components.name DESC, context_components.id ASC",
            Self::KindAsc => "context_components.kind ASC, context_components.id ASC",
            Self::KindDesc => "context_components.kind DESC, context_components.id ASC",
            Self::CreatedAtAsc => "context_components.created_at ASC, context_components.id ASC",
            Self::CreatedAtDesc => "context_components.created_at DESC, context_components.id ASC",
        }
    }
}

impl FromStr for ComponentSort {
    type Err = ComponentSortParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "" | "kind" => Ok(Self::KindAsc),
            "-kind" => Ok(Self::KindDesc),
            "name" => Ok(Self::NameAsc),
            "-name" => Ok(Self::NameDesc),
            "created_at" => Ok(Self::CreatedAtAsc),
            "-created_at" => Ok(Self::CreatedAtDesc),
            other => Err(ComponentSortParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when a component sort query value is unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unsupported component sort: {value}")]
pub struct ComponentSortParseError {
    value: String,
}

/// A component list item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentListItem {
    /// Component id.
    pub id: String,
    /// Parent context id.
    pub context_id: String,
    /// Component kind.
    pub kind: StoredComponentKind,
    /// Component display name.
    pub name: String,
    /// Reproducible content fingerprint.
    pub content_hash: String,
    /// Component row creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Full component detail available before body-content storage is introduced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentDetail {
    /// Component id.
    pub id: String,
    /// Parent context id.
    pub context_id: String,
    /// Component kind.
    pub kind: StoredComponentKind,
    /// Component display name.
    pub name: String,
    /// Reproducible content fingerprint.
    pub content_hash: String,
    /// Flexible component metadata stored in PostgreSQL.
    pub metadata: Value,
    /// Component row creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Component row update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// Pagination metadata for component list APIs.
pub type ComponentListPagination = ListPagination;

/// Paginated component list result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentList {
    /// Matching component items.
    pub items: Vec<ComponentListItem>,
    /// Pagination metadata.
    pub pagination: ComponentListPagination,
}

impl ComponentList {
    /// Builds a list response from query metadata.
    #[must_use]
    pub const fn new(
        items: Vec<ComponentListItem>,
        query: &ComponentListQuery,
        total: u64,
    ) -> Self {
        Self {
            items,
            pagination: ComponentListPagination::new(query.page, query.per_page, total),
        }
    }
}

/// Repository contract for context-scoped component list queries.
#[async_trait]
pub trait ContextComponentRepository: Send + Sync {
    /// Lists components for one context using pagination, filtering, and sorting.
    async fn list_components(
        &self,
        context_id: String,
        query: ComponentListQuery,
    ) -> Result<ComponentList, StorageRepositoryError>;

    /// Returns one component inside one context.
    async fn get_component(
        &self,
        context_id: String,
        component_id: String,
    ) -> Result<ComponentDetail, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_PAGE, MAX_PER_PAGE};

    #[test]
    fn component_list_query_normalizes_raw_values() {
        let query = ComponentListQuery::new(
            Some(0),
            Some(MAX_PER_PAGE + 1),
            Some("  policy  ".to_owned()),
            Some(StoredComponentKind::Knowledge),
            ComponentSort::NameDesc,
        );

        assert_eq!(query.page, DEFAULT_PAGE);
        assert_eq!(query.per_page, MAX_PER_PAGE);
        assert_eq!(query.search.as_deref(), Some("policy"));
        assert_eq!(query.kind, Some(StoredComponentKind::Knowledge));
        assert_eq!(query.sort, ComponentSort::NameDesc);
    }

    #[test]
    fn component_list_query_offset_handles_large_pages_without_overflow() {
        let query = ComponentListQuery::new(
            Some(u32::MAX),
            Some(MAX_PER_PAGE),
            None,
            None,
            ComponentSort::CreatedAtDesc,
        );

        assert_eq!(
            query.offset(),
            (u64::from(u32::MAX) - 1) * u64::from(MAX_PER_PAGE)
        );
    }

    #[test]
    fn component_sort_parses_public_query_values() {
        assert_eq!("name".parse::<ComponentSort>(), Ok(ComponentSort::NameAsc));
        assert_eq!(
            "-name".parse::<ComponentSort>(),
            Ok(ComponentSort::NameDesc)
        );
        assert_eq!("kind".parse::<ComponentSort>(), Ok(ComponentSort::KindAsc));
        assert_eq!(
            "-kind".parse::<ComponentSort>(),
            Ok(ComponentSort::KindDesc)
        );
        assert_eq!(
            "created_at".parse::<ComponentSort>(),
            Ok(ComponentSort::CreatedAtAsc)
        );
        assert_eq!(
            "-created_at".parse::<ComponentSort>(),
            Ok(ComponentSort::CreatedAtDesc)
        );
        assert!("content_hash".parse::<ComponentSort>().is_err());
    }
}
