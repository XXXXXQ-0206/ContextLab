//! Evaluation run listing contracts for storage-backed API queries.

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

/// Query options for listing evaluation runs inside one context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationRunListQuery {
    /// One-based page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Optional case-insensitive search term.
    pub search: Option<String>,
    /// Optional exact suite name filter.
    pub suite_name: Option<String>,
    /// Optional exact model version filter.
    pub model_version: Option<String>,
    /// Sort order.
    pub sort: EvaluationRunSort,
}

impl EvaluationRunListQuery {
    /// Creates a normalized query from raw values.
    #[must_use]
    pub fn new(
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<String>,
        suite_name: Option<String>,
        model_version: Option<String>,
        sort: EvaluationRunSort,
    ) -> Self {
        let page = normalize_page(page);
        let per_page = normalize_per_page(per_page);
        let search = normalize_search(search);
        let suite_name = normalize_search(suite_name);
        let model_version = normalize_search(model_version);

        Self {
            page,
            per_page,
            search,
            suite_name,
            model_version,
            sort,
        }
    }

    /// Returns the SQL offset for this query.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        offset(self.page, self.per_page)
    }
}

impl Default for EvaluationRunListQuery {
    fn default() -> Self {
        Self::new(None, None, None, None, None, EvaluationRunSort::default())
    }
}

/// Query options for building an evaluation scorecard inside one context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationScorecardQuery {
    /// Optional case-insensitive search term applied to run identity fields.
    pub search: Option<String>,
    /// Optional exact suite name filter.
    pub suite_name: Option<String>,
    /// Optional exact model version filter.
    pub model_version: Option<String>,
}

impl EvaluationScorecardQuery {
    /// Creates a normalized scorecard query from raw values.
    #[must_use]
    pub fn new(
        search: Option<String>,
        suite_name: Option<String>,
        model_version: Option<String>,
    ) -> Self {
        Self {
            search: normalize_search(search),
            suite_name: normalize_search(suite_name),
            model_version: normalize_search(model_version),
        }
    }
}

impl Default for EvaluationScorecardQuery {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}

/// Supported evaluation run sort orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationRunSort {
    /// Sort by execution time ascending.
    ExecutedAtAsc,
    /// Sort by execution time descending.
    #[default]
    ExecutedAtDesc,
    /// Sort by creation time ascending.
    CreatedAtAsc,
    /// Sort by creation time descending.
    CreatedAtDesc,
    /// Sort by suite name ascending.
    SuiteNameAsc,
    /// Sort by suite name descending.
    SuiteNameDesc,
    /// Sort by model version ascending.
    ModelVersionAsc,
    /// Sort by model version descending.
    ModelVersionDesc,
}

impl EvaluationRunSort {
    /// Returns the public query parameter value.
    #[must_use]
    pub const fn as_query_value(self) -> &'static str {
        match self {
            Self::ExecutedAtAsc => "executed_at",
            Self::ExecutedAtDesc => "-executed_at",
            Self::CreatedAtAsc => "created_at",
            Self::CreatedAtDesc => "-created_at",
            Self::SuiteNameAsc => "suite_name",
            Self::SuiteNameDesc => "-suite_name",
            Self::ModelVersionAsc => "model_version",
            Self::ModelVersionDesc => "-model_version",
        }
    }

    /// Returns a safe SQL `ORDER BY` clause for this sort order.
    #[must_use]
    pub const fn order_by_sql(self) -> &'static str {
        match self {
            Self::ExecutedAtAsc => "evaluation_runs.executed_at ASC, evaluation_runs.id ASC",
            Self::ExecutedAtDesc => "evaluation_runs.executed_at DESC, evaluation_runs.id ASC",
            Self::CreatedAtAsc => "evaluation_runs.created_at ASC, evaluation_runs.id ASC",
            Self::CreatedAtDesc => "evaluation_runs.created_at DESC, evaluation_runs.id ASC",
            Self::SuiteNameAsc => "evaluation_runs.suite_name ASC, evaluation_runs.id ASC",
            Self::SuiteNameDesc => "evaluation_runs.suite_name DESC, evaluation_runs.id ASC",
            Self::ModelVersionAsc => "evaluation_runs.model_version ASC, evaluation_runs.id ASC",
            Self::ModelVersionDesc => "evaluation_runs.model_version DESC, evaluation_runs.id ASC",
        }
    }
}

impl FromStr for EvaluationRunSort {
    type Err = EvaluationRunSortParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "" | "-executed_at" => Ok(Self::ExecutedAtDesc),
            "executed_at" => Ok(Self::ExecutedAtAsc),
            "created_at" => Ok(Self::CreatedAtAsc),
            "-created_at" => Ok(Self::CreatedAtDesc),
            "suite_name" => Ok(Self::SuiteNameAsc),
            "-suite_name" => Ok(Self::SuiteNameDesc),
            "model_version" => Ok(Self::ModelVersionAsc),
            "-model_version" => Ok(Self::ModelVersionDesc),
            other => Err(EvaluationRunSortParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when an evaluation run sort query value is unsupported.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unsupported evaluation run sort: {value}")]
pub struct EvaluationRunSortParseError {
    value: String,
}

/// An evaluation run list item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRunListItem {
    /// Evaluation run id.
    pub id: String,
    /// Evaluated context id.
    pub context_id: String,
    /// Evaluation suite name.
    pub suite_name: String,
    /// Model version used for this run.
    pub model_version: String,
    /// Sampling temperature used for this run.
    pub temperature: f32,
    /// Number of stored metric values.
    pub metric_count: u32,
    /// Execution timestamp.
    pub executed_at: DateTime<Utc>,
    /// Run row creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// A detailed evaluation run response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRunDetail {
    /// Evaluation run id.
    pub id: String,
    /// Evaluated context id.
    pub context_id: String,
    /// Evaluation suite name.
    pub suite_name: String,
    /// Model version used for this run.
    pub model_version: String,
    /// Sampling temperature used for this run.
    pub temperature: f32,
    /// Number of stored metric values.
    pub metric_count: u32,
    /// Persisted metrics payload for scorecards and evaluation diff workflows.
    pub metrics: Value,
    /// Execution timestamp.
    pub executed_at: DateTime<Utc>,
    /// Run row creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Aggregated numeric metric average for an evaluation scorecard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationScorecardMetric {
    /// Metric key from persisted evaluation run metrics JSON.
    pub name: String,
    /// Average of numeric metric values across matching runs.
    pub average: f64,
    /// Number of runs contributing this metric value.
    pub sample_count: u32,
}

/// Context-level evaluation scorecard built from persisted run metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationScorecard {
    /// Evaluated context id.
    pub context_id: String,
    /// Number of matching evaluation runs included in the scorecard.
    pub run_count: u64,
    /// Numeric metric averages sorted by metric name.
    pub metrics: Vec<EvaluationScorecardMetric>,
}

/// Pagination metadata for evaluation run list APIs.
pub type EvaluationRunListPagination = ListPagination;

/// Paginated evaluation run list result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRunList {
    /// Matching evaluation run items.
    pub items: Vec<EvaluationRunListItem>,
    /// Pagination metadata.
    pub pagination: EvaluationRunListPagination,
}

impl EvaluationRunList {
    /// Builds a list response from query metadata.
    #[must_use]
    pub const fn new(
        items: Vec<EvaluationRunListItem>,
        query: &EvaluationRunListQuery,
        total: u64,
    ) -> Self {
        Self {
            items,
            pagination: EvaluationRunListPagination::new(query.page, query.per_page, total),
        }
    }
}

/// Repository contract for context-scoped evaluation run list queries.
#[async_trait]
pub trait EvaluationRunRepository: Send + Sync {
    /// Lists evaluation runs for one context using pagination, filtering, and sorting.
    async fn list_evaluation_runs(
        &self,
        context_id: String,
        query: EvaluationRunListQuery,
    ) -> Result<EvaluationRunList, StorageRepositoryError>;

    /// Returns one evaluation run with its persisted metrics payload.
    async fn get_evaluation_run(
        &self,
        context_id: String,
        run_id: String,
    ) -> Result<EvaluationRunDetail, StorageRepositoryError>;

    /// Builds a context-level scorecard from persisted numeric metric values.
    async fn get_evaluation_scorecard(
        &self,
        context_id: String,
        query: EvaluationScorecardQuery,
    ) -> Result<EvaluationScorecard, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEFAULT_PAGE, MAX_PER_PAGE};

    #[test]
    fn evaluation_run_list_query_normalizes_raw_values() {
        let query = EvaluationRunListQuery::new(
            Some(0),
            Some(MAX_PER_PAGE + 1),
            Some("  safety  ".to_owned()),
            Some("  Safety Regression  ".to_owned()),
            Some("  deepseek-chat  ".to_owned()),
            EvaluationRunSort::ModelVersionDesc,
        );

        assert_eq!(query.page, DEFAULT_PAGE);
        assert_eq!(query.per_page, MAX_PER_PAGE);
        assert_eq!(query.search.as_deref(), Some("safety"));
        assert_eq!(query.suite_name.as_deref(), Some("Safety Regression"));
        assert_eq!(query.model_version.as_deref(), Some("deepseek-chat"));
        assert_eq!(query.sort, EvaluationRunSort::ModelVersionDesc);
    }

    #[test]
    fn evaluation_run_list_query_offset_handles_large_pages_without_overflow() {
        let query = EvaluationRunListQuery::new(
            Some(u32::MAX),
            Some(MAX_PER_PAGE),
            None,
            None,
            None,
            EvaluationRunSort::ExecutedAtDesc,
        );

        assert_eq!(
            query.offset(),
            (u64::from(u32::MAX) - 1) * u64::from(MAX_PER_PAGE)
        );
    }

    #[test]
    fn evaluation_scorecard_query_normalizes_raw_values() {
        let query = EvaluationScorecardQuery::new(
            Some("  safety  ".to_owned()),
            Some("  Safety Regression  ".to_owned()),
            Some("  deepseek-chat  ".to_owned()),
        );

        assert_eq!(query.search.as_deref(), Some("safety"));
        assert_eq!(query.suite_name.as_deref(), Some("Safety Regression"));
        assert_eq!(query.model_version.as_deref(), Some("deepseek-chat"));
    }

    #[test]
    fn evaluation_run_sort_parses_public_query_values() {
        assert_eq!(
            "executed_at".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::ExecutedAtAsc)
        );
        assert_eq!(
            "-executed_at".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::ExecutedAtDesc)
        );
        assert_eq!(
            "created_at".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::CreatedAtAsc)
        );
        assert_eq!(
            "-created_at".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::CreatedAtDesc)
        );
        assert_eq!(
            "suite_name".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::SuiteNameAsc)
        );
        assert_eq!(
            "-suite_name".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::SuiteNameDesc)
        );
        assert_eq!(
            "model_version".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::ModelVersionAsc)
        );
        assert_eq!(
            "-model_version".parse::<EvaluationRunSort>(),
            Ok(EvaluationRunSort::ModelVersionDesc)
        );
        assert!("metrics".parse::<EvaluationRunSort>().is_err());
    }
}
