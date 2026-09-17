//! Evaluation metrics and scorecards for ContextLab.

mod benchmark;
mod benchmark_execution;
mod benchmark_workspace;
mod capability_composition;
mod decision_diff;
mod execution_receipt;
mod regression;

use chrono::{DateTime, Utc};
use contextlab_context_core::ContextId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use uuid::Uuid;

const MIN_SUPPORTED_TEMPERATURE: f32 = 0.0;
const MAX_SUPPORTED_TEMPERATURE: f32 = 2.0;

pub use benchmark::{
    BenchmarkCase, BenchmarkCaseId, BenchmarkDataset, BenchmarkDatasetId, BenchmarkEvaluation,
    BenchmarkExpectedOutput, BenchmarkSuite, BenchmarkSuiteId, BenchmarkValidationError,
};
pub use benchmark_execution::{
    BenchmarkCaseExecutionResult, BenchmarkExecutedCase, BenchmarkExecutionCase,
    BenchmarkExecutionCaseKey, BenchmarkExecutionCohort, BenchmarkExecutionError,
    BenchmarkExecutionPlan,
};
pub use benchmark_workspace::{
    BenchmarkDatasetWorkspaceSummaryV1, BenchmarkEvaluationDiffWorkspaceSummaryV1,
    BenchmarkEvaluationMetricChangeKindV1, BenchmarkEvaluationMetricChangeWorkspaceSummaryV1,
    BenchmarkExecutionReceiptWorkspaceSummaryV1, BenchmarkRunWorkspaceSummaryV1,
    BenchmarkScorecardMetricSummaryV1, BenchmarkScorecardWorkspaceSummaryV1,
    BenchmarkSuiteWorkspaceSummaryV1, BenchmarkWorkspaceProjectionError,
    BenchmarkWorkspaceProjectionV1,
};
pub use capability_composition::{
    BENCHMARK_CAPABILITY_SNAPSHOT_SCHEMA_VERSION, BenchmarkCapabilityCompositionError,
    BenchmarkCapabilityFactV1, BenchmarkCapabilityKindV1, BenchmarkCapabilitySnapshotError,
    BenchmarkCapabilitySnapshotV1, BenchmarkEvaluationCapabilityCompositionV1,
};
pub use decision_diff::{
    BenchmarkDecisionComparisonError, BenchmarkDecisionComparisonInput, BenchmarkDecisionDiff,
    BenchmarkDecisionMetricDiff, BenchmarkDecisionMetricInput,
};
pub use execution_receipt::{
    BenchmarkExecutionCohortId, BenchmarkExecutionReceipt, BenchmarkExecutionReceiptError,
    BenchmarkExecutionReceiptVersion,
};
pub use regression::{
    RegressionCheck, RegressionCheckStatus, RegressionDecision, RegressionDecisionStatus,
    RegressionThreshold, RegressionValidationError, ThresholdDirection,
};

/// Stable identifier for an evaluation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EvaluationRunId(Uuid);

impl EvaluationRunId {
    /// Creates a new run identifier.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Wraps an existing UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for EvaluationRunId {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluation metric kinds tracked by ContextLab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricKind {
    /// End-to-end latency in milliseconds.
    LatencyMs,
    /// Total model or provider cost.
    CostUsd,
    /// Accuracy from 0.0 to 1.0.
    Accuracy,
    /// Hallucination rate from 0.0 to 1.0.
    HallucinationRate,
    /// Tool calls per run.
    ToolUsageCount,
    /// Token count.
    TokenCount,
    /// Execution time in milliseconds.
    ExecutionTimeMs,
    /// Output quality score from 0.0 to 1.0.
    OutputQuality,
    /// Success rate from 0.0 to 1.0.
    SuccessRate,
}

/// A single measured metric.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MetricMeasurement {
    kind: MetricKind,
    value: f64,
}

impl MetricMeasurement {
    /// Creates a metric measurement.
    pub fn new(kind: MetricKind, value: f64) -> Result<Self, EvaluationError> {
        if !value.is_finite() {
            return Err(EvaluationError::NonFiniteMetric { kind });
        }
        Ok(Self { kind, value })
    }

    /// Returns the metric kind.
    #[must_use]
    pub const fn kind(&self) -> MetricKind {
        self.kind
    }

    /// Returns the metric value.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }
}

/// Errors produced by evaluation constructors.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum EvaluationError {
    /// Model version was empty or whitespace-only.
    #[error("model version must not be empty")]
    EmptyModelVersion,
    /// Sampling temperature was NaN or infinite.
    #[error("temperature must be finite")]
    NonFiniteTemperature,
    /// Sampling temperature was outside the supported range.
    #[error("temperature must be between 0.0 and 2.0")]
    UnsupportedTemperature,
    /// Metric value was NaN or infinite.
    #[error("metric {kind:?} must be finite")]
    NonFiniteMetric {
        /// The metric kind that received a non-finite value.
        kind: MetricKind,
    },
}

/// A single evaluation run against a Context.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvaluationRun {
    id: EvaluationRunId,
    context_id: ContextId,
    model_version: String,
    temperature: f32,
    measurements: Vec<MetricMeasurement>,
    executed_at: DateTime<Utc>,
}

impl EvaluationRun {
    /// Creates an evaluation run.
    pub fn new(
        context_id: ContextId,
        model_version: impl Into<String>,
        temperature: f32,
        measurements: Vec<MetricMeasurement>,
        executed_at: DateTime<Utc>,
    ) -> Result<Self, EvaluationError> {
        Self::with_id(
            EvaluationRunId::new(),
            context_id,
            model_version,
            temperature,
            measurements,
            executed_at,
        )
    }

    /// Creates a validated evaluation run with an existing identifier.
    pub fn with_id(
        id: EvaluationRunId,
        context_id: ContextId,
        model_version: impl Into<String>,
        temperature: f32,
        measurements: Vec<MetricMeasurement>,
        executed_at: DateTime<Utc>,
    ) -> Result<Self, EvaluationError> {
        let model_version = model_version.into();
        if model_version.trim().is_empty() {
            return Err(EvaluationError::EmptyModelVersion);
        }
        if !temperature.is_finite() {
            return Err(EvaluationError::NonFiniteTemperature);
        }
        if !(MIN_SUPPORTED_TEMPERATURE..=MAX_SUPPORTED_TEMPERATURE).contains(&temperature) {
            return Err(EvaluationError::UnsupportedTemperature);
        }

        Ok(Self {
            id,
            context_id,
            model_version,
            temperature,
            measurements,
            executed_at,
        })
    }

    /// Rehydrates a persisted evaluation run after validating stored values.
    pub fn from_persisted(
        id: EvaluationRunId,
        context_id: ContextId,
        model_version: impl Into<String>,
        temperature: f32,
        measurements: Vec<MetricMeasurement>,
        executed_at: DateTime<Utc>,
    ) -> Result<Self, EvaluationError> {
        Self::with_id(
            id,
            context_id,
            model_version,
            temperature,
            measurements,
            executed_at,
        )
    }

    /// Returns the run identifier.
    #[must_use]
    pub const fn id(&self) -> EvaluationRunId {
        self.id
    }

    /// Returns associated Context identifier.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the evaluated model version.
    #[must_use]
    pub fn model_version(&self) -> &str {
        &self.model_version
    }

    /// Returns the sampling temperature.
    #[must_use]
    pub const fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns measurements.
    #[must_use]
    pub fn measurements(&self) -> &[MetricMeasurement] {
        &self.measurements
    }

    /// Returns the execution timestamp.
    #[must_use]
    pub const fn executed_at(&self) -> DateTime<Utc> {
        self.executed_at
    }
}

/// Aggregated averages for a set of evaluation runs.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scorecard {
    run_count: usize,
    averages: BTreeMap<MetricKind, f64>,
    sample_counts: BTreeMap<MetricKind, usize>,
    invalid_metrics: BTreeSet<MetricKind>,
}

impl Scorecard {
    /// Builds average metrics from evaluation runs.
    #[must_use]
    pub fn from_runs(runs: &[EvaluationRun]) -> Self {
        let mut values: BTreeMap<MetricKind, Vec<f64>> = BTreeMap::new();
        let mut invalid_metrics = BTreeSet::new();

        for run in runs {
            let mut seen = BTreeSet::new();
            for measurement in run.measurements() {
                if !seen.insert(measurement.kind()) {
                    invalid_metrics.insert(measurement.kind());
                    continue;
                }
                values
                    .entry(measurement.kind())
                    .or_default()
                    .push(measurement.value());
            }
        }

        let sample_counts = values
            .iter()
            .map(|(kind, values)| (*kind, values.len()))
            .collect();
        let averages = values
            .into_iter()
            .map(|(kind, mut values)| {
                values.sort_by(f64::total_cmp);
                let mut average = values[0];
                for (index, value) in values.iter().enumerate().skip(1) {
                    let sample_count = (index + 1) as f64;
                    let delta = value - average;
                    average = if delta.is_finite() {
                        average + delta / sample_count
                    } else {
                        average * ((sample_count - 1.0) / sample_count) + value / sample_count
                    };
                }
                (kind, average)
            })
            .collect();

        Self {
            run_count: runs.len(),
            averages,
            sample_counts,
            invalid_metrics,
        }
    }

    /// Returns the number of runs included.
    #[must_use]
    pub const fn run_count(&self) -> usize {
        self.run_count
    }

    /// Returns an average metric if present.
    #[must_use]
    pub fn average(&self, kind: MetricKind) -> Option<f64> {
        self.averages.get(&kind).copied()
    }

    /// Returns the number of runs that contributed the metric.
    #[must_use]
    pub fn sample_count(&self, kind: MetricKind) -> usize {
        self.sample_counts.get(&kind).copied().unwrap_or_default()
    }

    /// Returns whether every run contributed exactly one finite value for the metric.
    #[must_use]
    pub fn has_complete_metric(&self, kind: MetricKind) -> bool {
        self.run_count > 0
            && self.sample_count(kind) == self.run_count
            && !self.invalid_metrics.contains(&kind)
            && self.average(kind).is_some_and(f64::is_finite)
    }

    /// Returns all averages.
    #[must_use]
    pub const fn averages(&self) -> &BTreeMap<MetricKind, f64> {
        &self.averages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_metric_averages() {
        let context_id = ContextId::new();
        let runs = vec![
            EvaluationRun::new(
                context_id,
                "model-a",
                0.2,
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.8).expect("finite metric"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 120.0).expect("finite metric"),
                ],
                Utc::now(),
            )
            .expect("valid evaluation run"),
            EvaluationRun::new(
                context_id,
                "model-a",
                0.2,
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 1.0).expect("finite metric"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 80.0).expect("finite metric"),
                ],
                Utc::now(),
            )
            .expect("valid evaluation run"),
        ];

        let scorecard = Scorecard::from_runs(&runs);

        assert_eq!(scorecard.run_count(), 2);
        assert_eq!(scorecard.average(MetricKind::Accuracy), Some(0.9));
        assert_eq!(scorecard.average(MetricKind::LatencyMs), Some(100.0));
    }

    #[test]
    fn rejects_non_finite_metric_values() {
        let error = MetricMeasurement::new(MetricKind::Accuracy, f64::NAN)
            .expect_err("NaN metric should fail");

        assert_eq!(
            error,
            EvaluationError::NonFiniteMetric {
                kind: MetricKind::Accuracy
            }
        );
    }

    #[test]
    fn rejects_blank_model_identity_during_evaluation_run_creation() {
        let error = EvaluationRun::new(ContextId::new(), " \t ", 0.2, Vec::new(), Utc::now())
            .expect_err("blank model identity should fail");

        assert_eq!(error, EvaluationError::EmptyModelVersion);
    }

    #[test]
    fn rejects_non_finite_and_unsupported_evaluation_run_temperatures() {
        for temperature in [f32::NAN, f32::INFINITY] {
            let error = EvaluationRun::new(
                ContextId::new(),
                "model-a",
                temperature,
                Vec::new(),
                Utc::now(),
            )
            .expect_err("non-finite temperature should fail");

            assert_eq!(error, EvaluationError::NonFiniteTemperature);
        }
        let below_range =
            EvaluationRun::new(ContextId::new(), "model-a", -0.01, Vec::new(), Utc::now())
                .expect_err("temperature below the supported range should fail");
        let above_range =
            EvaluationRun::new(ContextId::new(), "model-a", 2.01, Vec::new(), Utc::now())
                .expect_err("temperature above the supported range should fail");

        assert_eq!(below_range, EvaluationError::UnsupportedTemperature);
        assert_eq!(above_range, EvaluationError::UnsupportedTemperature);
    }

    #[test]
    fn accepts_evaluation_run_temperature_at_supported_boundaries() {
        for temperature in [0.0, 2.0] {
            EvaluationRun::new(
                ContextId::new(),
                "model-a",
                temperature,
                Vec::new(),
                Utc::now(),
            )
            .expect("temperature boundary should be supported");
        }
    }

    #[test]
    fn rehydrates_evaluation_runs_with_their_persisted_identity() {
        let run_id = EvaluationRunId::new();
        let context_id = ContextId::new();
        let executed_at = Utc::now();
        let measurements =
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("finite metric")];

        let run = EvaluationRun::from_persisted(
            run_id,
            context_id,
            "model-a",
            0.2,
            measurements.clone(),
            executed_at,
        )
        .expect("persisted run");
        let with_id = EvaluationRun::with_id(
            run_id,
            context_id,
            "model-a",
            0.2,
            measurements,
            executed_at,
        )
        .expect("run with existing identity");

        assert_eq!(run.id(), run_id);
        assert_eq!(run, with_id);
    }

    #[test]
    fn rejects_invalid_persisted_evaluation_run_configuration() {
        let run_id = EvaluationRunId::new();
        let context_id = ContextId::new();

        assert_eq!(
            EvaluationRun::from_persisted(run_id, context_id, "  ", 0.2, Vec::new(), Utc::now(),),
            Err(EvaluationError::EmptyModelVersion)
        );
        assert_eq!(
            EvaluationRun::from_persisted(
                run_id,
                context_id,
                "model-a",
                f32::INFINITY,
                Vec::new(),
                Utc::now(),
            ),
            Err(EvaluationError::NonFiniteTemperature)
        );
        assert_eq!(
            EvaluationRun::from_persisted(
                run_id,
                context_id,
                "model-a",
                2.01,
                Vec::new(),
                Utc::now(),
            ),
            Err(EvaluationError::UnsupportedTemperature)
        );
    }
}
