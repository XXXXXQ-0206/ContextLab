//! Shared list-query primitives for storage repositories.

use serde::{Deserialize, Serialize};

/// Default page number for list APIs.
pub const DEFAULT_PAGE: u32 = 1;
/// Default page size for list APIs.
pub const DEFAULT_PER_PAGE: u32 = 20;
/// Maximum page size for list APIs.
pub const MAX_PER_PAGE: u32 = 100;

/// Pagination metadata for list APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPagination {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total matching items before pagination.
    pub total: u64,
}

impl ListPagination {
    /// Builds pagination metadata from normalized query values.
    #[must_use]
    pub const fn new(page: u32, per_page: u32, total: u64) -> Self {
        Self {
            page,
            per_page,
            total,
        }
    }
}

pub(crate) fn normalize_page(page: Option<u32>) -> u32 {
    page.unwrap_or(DEFAULT_PAGE).max(1)
}

pub(crate) fn normalize_per_page(per_page: Option<u32>) -> u32 {
    per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE)
}

pub(crate) fn normalize_search(search: Option<String>) -> Option<String> {
    search
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub(crate) const fn offset(page: u32, per_page: u32) -> u64 {
    (page.saturating_sub(1) as u64) * (per_page as u64)
}
