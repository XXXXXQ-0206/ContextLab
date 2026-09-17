//! Provider-free, redacted benchmark workspace projections.

use crate::execution_receipt::BenchmarkWorkspaceDefinitionFacts;
use crate::{
    BenchmarkDatasetId, BenchmarkDecisionComparisonError, BenchmarkDecisionMetricDiff,
    BenchmarkDecisionMetricInput, BenchmarkExecutionCohortId, BenchmarkExecutionPlan,
    BenchmarkExecutionReceipt, BenchmarkSuiteId, MetricKind, RegressionCheckStatus,
    RegressionDecisionStatus, ThresholdDirection,
};
use serde::Serialize;
use thiserror::Error;

const BENCHMARK_WORKSPACE_PROJECTION_SCHEMA_VERSION: u16 = 1;

/// Safe, versioned benchmark workspace facts derived from one sealed execution receipt.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkWorkspaceProjectionV1 {
    schema_version: u16,
    receipt: BenchmarkExecutionReceiptWorkspaceSummaryV1,
    suite: BenchmarkSuiteWorkspaceSummaryV1,
    datasets: Vec<BenchmarkDatasetWorkspaceSummaryV1>,
    runs: Vec<BenchmarkRunWorkspaceSummaryV1>,
    scorecard: BenchmarkScorecardWorkspaceSummaryV1,
    regression_status: RegressionDecisionStatus,
    evaluation_diff: Option<BenchmarkEvaluationDiffWorkspaceSummaryV1>,
}

impl BenchmarkWorkspaceProjectionV1 {
    /// Projects safe workspace facts without invoking providers or re-evaluating policy.
    pub fn from_receipt(
        receipt: &BenchmarkExecutionReceipt,
        plan: &BenchmarkExecutionPlan,
    ) -> Result<Self, BenchmarkWorkspaceProjectionError> {
        Self::build(receipt, plan, None)
    }

    /// Projects a revised sealed receipt with the existing diff against a comparable baseline.
    pub fn from_receipts(
        baseline: &BenchmarkExecutionReceipt,
        revised: &BenchmarkExecutionReceipt,
        plan: &BenchmarkExecutionPlan,
    ) -> Result<Self, BenchmarkWorkspaceProjectionError> {
        Self::validate_suite(baseline, plan)?;
        Self::validate_suite(revised, plan)?;
        if baseline.cohort_id() == revised.cohort_id() {
            return Err(BenchmarkWorkspaceProjectionError::IdenticalReceiptCohort {
                cohort_id: baseline.cohort_id(),
            });
        }
        let diff = crate::BenchmarkDecisionDiff::between(
            baseline.decision_input().clone(),
            revised.decision_input().clone(),
        )?;

        Self::build(
            revised,
            plan,
            Some(BenchmarkEvaluationDiffWorkspaceSummaryV1::from_diff(
                baseline.cohort_id(),
                revised.cohort_id(),
                diff,
            )),
        )
    }

    fn build(
        receipt: &BenchmarkExecutionReceipt,
        plan: &BenchmarkExecutionPlan,
        evaluation_diff: Option<BenchmarkEvaluationDiffWorkspaceSummaryV1>,
    ) -> Result<Self, BenchmarkWorkspaceProjectionError> {
        Self::validate_suite(receipt, plan)?;

        let definition = receipt.workspace_definition();
        let datasets = definition
            .datasets()
            .iter()
            .map(|dataset| BenchmarkDatasetWorkspaceSummaryV1 {
                id: dataset.id(),
                name: dataset.name().to_owned(),
                case_count: dataset.case_count(),
            })
            .collect();
        let runs = receipt
            .cohort()
            .entries()
            .iter()
            .map(|entry| BenchmarkRunWorkspaceSummaryV1 {
                dataset_id: entry.key().dataset_id(),
                case_id: entry.key().case_id(),
                metric_count: entry.run().measurements().len(),
            })
            .collect();
        let scorecard = receipt.evaluation().scorecard();
        let metrics = receipt
            .evaluation()
            .decision()
            .checks()
            .iter()
            .map(|check| BenchmarkScorecardMetricSummaryV1 {
                metric: check.metric(),
                threshold_direction: check.threshold().direction(),
                threshold_value: check.threshold().value(),
                observed: check.observed(),
                sample_count: scorecard.sample_count(check.metric()),
                required_sample_count: scorecard.run_count(),
                has_complete_coverage: check.has_complete_coverage(),
                outcome: check.status(),
            })
            .collect();

        Ok(Self {
            schema_version: BENCHMARK_WORKSPACE_PROJECTION_SCHEMA_VERSION,
            receipt: BenchmarkExecutionReceiptWorkspaceSummaryV1 {
                cohort_id: receipt.cohort_id(),
            },
            suite: BenchmarkSuiteWorkspaceSummaryV1 {
                id: definition.suite_id(),
                name: definition.suite_name().to_owned(),
            },
            datasets,
            runs,
            scorecard: BenchmarkScorecardWorkspaceSummaryV1 {
                run_count: scorecard.run_count(),
                metrics,
            },
            regression_status: receipt.evaluation().decision().status(),
            evaluation_diff,
        })
    }

    fn validate_suite(
        receipt: &BenchmarkExecutionReceipt,
        plan: &BenchmarkExecutionPlan,
    ) -> Result<(), BenchmarkWorkspaceProjectionError> {
        if receipt.suite_id() != plan.suite().id() {
            return Err(BenchmarkWorkspaceProjectionError::SuiteMismatch {
                receipt_suite_id: receipt.suite_id(),
                plan_suite_id: plan.suite().id(),
            });
        }
        if receipt.workspace_definition() != &BenchmarkWorkspaceDefinitionFacts::from_plan(plan) {
            return Err(BenchmarkWorkspaceProjectionError::PlanDefinitionMismatch {
                suite_id: receipt.suite_id(),
            });
        }
        Ok(())
    }

    /// Returns the explicit projection schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the safe identity of the sealed receipt behind this projection.
    #[must_use]
    pub const fn receipt(&self) -> &BenchmarkExecutionReceiptWorkspaceSummaryV1 {
        &self.receipt
    }

    /// Returns the safe benchmark suite summary.
    #[must_use]
    pub const fn suite(&self) -> &BenchmarkSuiteWorkspaceSummaryV1 {
        &self.suite
    }

    /// Returns safe dataset summaries in stable identifier order.
    #[must_use]
    pub fn datasets(&self) -> &[BenchmarkDatasetWorkspaceSummaryV1] {
        &self.datasets
    }

    /// Returns case-to-run safe summaries in stable composite identifier order.
    #[must_use]
    pub fn runs(&self) -> &[BenchmarkRunWorkspaceSummaryV1] {
        &self.runs
    }

    /// Returns the policy-owned safe scorecard summary.
    #[must_use]
    pub const fn scorecard(&self) -> &BenchmarkScorecardWorkspaceSummaryV1 {
        &self.scorecard
    }

    /// Returns the already-calculated regression decision status.
    #[must_use]
    pub const fn regression_status(&self) -> RegressionDecisionStatus {
        self.regression_status
    }

    /// Returns the existing evaluation diff when a comparable baseline was projected.
    #[must_use]
    pub const fn evaluation_diff(&self) -> Option<&BenchmarkEvaluationDiffWorkspaceSummaryV1> {
        self.evaluation_diff.as_ref()
    }
}

/// Safe identity for the sealed execution receipt behind a workspace projection.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkExecutionReceiptWorkspaceSummaryV1 {
    cohort_id: BenchmarkExecutionCohortId,
}

impl BenchmarkExecutionReceiptWorkspaceSummaryV1 {
    /// Returns the stable execution cohort identity.
    #[must_use]
    pub const fn cohort_id(&self) -> BenchmarkExecutionCohortId {
        self.cohort_id
    }
}

/// Safe suite identity and name for a benchmark workspace.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkSuiteWorkspaceSummaryV1 {
    id: BenchmarkSuiteId,
    name: String,
}

impl BenchmarkSuiteWorkspaceSummaryV1 {
    /// Returns the stable suite identity.
    #[must_use]
    pub const fn id(&self) -> BenchmarkSuiteId {
        self.id
    }

    /// Returns the safe suite name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Safe dataset identity, name, and case count for a benchmark workspace.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkDatasetWorkspaceSummaryV1 {
    id: BenchmarkDatasetId,
    name: String,
    case_count: usize,
}

impl BenchmarkDatasetWorkspaceSummaryV1 {
    /// Returns the stable dataset identity.
    #[must_use]
    pub const fn id(&self) -> BenchmarkDatasetId {
        self.id
    }

    /// Returns the safe dataset name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the sealed case count without exposing cases.
    #[must_use]
    pub const fn case_count(&self) -> usize {
        self.case_count
    }
}

/// Safe case-to-run provenance facts with no raw case input or model output.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkRunWorkspaceSummaryV1 {
    dataset_id: BenchmarkDatasetId,
    case_id: crate::BenchmarkCaseId,
    metric_count: usize,
}

impl BenchmarkRunWorkspaceSummaryV1 {
    /// Returns the owning dataset identity.
    #[must_use]
    pub const fn dataset_id(&self) -> BenchmarkDatasetId {
        self.dataset_id
    }

    /// Returns the immutable case identity.
    #[must_use]
    pub const fn case_id(&self) -> crate::BenchmarkCaseId {
        self.case_id
    }

    /// Returns only the number of recorded metrics.
    #[must_use]
    pub const fn metric_count(&self) -> usize {
        self.metric_count
    }
}

/// Safe scorecard facts derived by the existing benchmark evaluation policy.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkScorecardWorkspaceSummaryV1 {
    run_count: usize,
    metrics: Vec<BenchmarkScorecardMetricSummaryV1>,
}

impl BenchmarkScorecardWorkspaceSummaryV1 {
    /// Returns the number of evaluated runs.
    #[must_use]
    pub const fn run_count(&self) -> usize {
        self.run_count
    }

    /// Returns metric summaries in stable metric order.
    #[must_use]
    pub fn metrics(&self) -> &[BenchmarkScorecardMetricSummaryV1] {
        &self.metrics
    }
}

/// Safe evidence and outcome facts for one thresholded scorecard metric.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkScorecardMetricSummaryV1 {
    metric: MetricKind,
    threshold_direction: ThresholdDirection,
    threshold_value: f64,
    observed: Option<f64>,
    sample_count: usize,
    required_sample_count: usize,
    has_complete_coverage: bool,
    outcome: RegressionCheckStatus,
}

impl BenchmarkScorecardMetricSummaryV1 {
    /// Returns the metric identity.
    #[must_use]
    pub const fn metric(&self) -> MetricKind {
        self.metric
    }

    /// Returns the preserved threshold direction without recalculating policy.
    #[must_use]
    pub const fn threshold_direction(&self) -> ThresholdDirection {
        self.threshold_direction
    }

    /// Returns the preserved threshold value without recalculating policy.
    #[must_use]
    pub const fn threshold_value(&self) -> f64 {
        self.threshold_value
    }

    /// Returns the policy-observed metric value when coverage is usable.
    #[must_use]
    pub const fn observed(&self) -> Option<f64> {
        self.observed
    }

    /// Returns the number of runs contributing this metric.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Returns the total run count required for complete coverage.
    #[must_use]
    pub const fn required_sample_count(&self) -> usize {
        self.required_sample_count
    }

    /// Returns whether every run contributed valid evidence for this metric.
    #[must_use]
    pub const fn has_complete_coverage(&self) -> bool {
        self.has_complete_coverage
    }

    /// Returns the existing policy outcome without recalculation.
    #[must_use]
    pub const fn outcome(&self) -> RegressionCheckStatus {
        self.outcome
    }
}

/// Safe existing evaluation-diff facts between two comparable sealed receipts.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkEvaluationDiffWorkspaceSummaryV1 {
    baseline_cohort_id: BenchmarkExecutionCohortId,
    revised_cohort_id: BenchmarkExecutionCohortId,
    status_change: Option<(RegressionDecisionStatus, RegressionDecisionStatus)>,
    metric_changes: Vec<BenchmarkEvaluationMetricChangeWorkspaceSummaryV1>,
}

impl BenchmarkEvaluationDiffWorkspaceSummaryV1 {
    fn from_diff(
        baseline_cohort_id: BenchmarkExecutionCohortId,
        revised_cohort_id: BenchmarkExecutionCohortId,
        diff: crate::BenchmarkDecisionDiff,
    ) -> Self {
        Self {
            baseline_cohort_id,
            revised_cohort_id,
            status_change: diff.status_change(),
            metric_changes: diff
                .metric_changes()
                .iter()
                .map(BenchmarkEvaluationMetricChangeWorkspaceSummaryV1::from_diff)
                .collect(),
        }
    }

    /// Returns the stable baseline execution cohort identity.
    #[must_use]
    pub const fn baseline_cohort_id(&self) -> BenchmarkExecutionCohortId {
        self.baseline_cohort_id
    }

    /// Returns the stable revised execution cohort identity.
    #[must_use]
    pub const fn revised_cohort_id(&self) -> BenchmarkExecutionCohortId {
        self.revised_cohort_id
    }

    /// Returns the existing overall regression status transition.
    #[must_use]
    pub const fn status_change(
        &self,
    ) -> Option<(RegressionDecisionStatus, RegressionDecisionStatus)> {
        self.status_change
    }

    /// Returns safe metric changes in the existing stable metric order.
    #[must_use]
    pub fn metric_changes(&self) -> &[BenchmarkEvaluationMetricChangeWorkspaceSummaryV1] {
        &self.metric_changes
    }
}

/// The structural kind of one existing evaluation metric diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEvaluationMetricChangeKindV1 {
    /// The metric exists only in the revised receipt.
    Added,
    /// The metric exists only in the baseline receipt.
    Removed,
    /// The metric exists in both receipts with changed evidence.
    Modified,
}

/// Redacted baseline/revised evidence for one changed evaluation metric.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkEvaluationMetricChangeWorkspaceSummaryV1 {
    metric: MetricKind,
    change_kind: BenchmarkEvaluationMetricChangeKindV1,
    baseline: Option<BenchmarkScorecardMetricSummaryV1>,
    revised: Option<BenchmarkScorecardMetricSummaryV1>,
}

impl BenchmarkEvaluationMetricChangeWorkspaceSummaryV1 {
    fn from_diff(diff: &BenchmarkDecisionMetricDiff) -> Self {
        let (change_kind, baseline, revised) = match diff {
            BenchmarkDecisionMetricDiff::Added { revised, .. } => (
                BenchmarkEvaluationMetricChangeKindV1::Added,
                None,
                Some(BenchmarkScorecardMetricSummaryV1::from_evidence(revised)),
            ),
            BenchmarkDecisionMetricDiff::Removed { baseline, .. } => (
                BenchmarkEvaluationMetricChangeKindV1::Removed,
                Some(BenchmarkScorecardMetricSummaryV1::from_evidence(baseline)),
                None,
            ),
            BenchmarkDecisionMetricDiff::Modified {
                baseline, revised, ..
            } => (
                BenchmarkEvaluationMetricChangeKindV1::Modified,
                Some(BenchmarkScorecardMetricSummaryV1::from_evidence(baseline)),
                Some(BenchmarkScorecardMetricSummaryV1::from_evidence(revised)),
            ),
        };
        Self {
            metric: diff.metric(),
            change_kind,
            baseline,
            revised,
        }
    }

    /// Returns the stable metric identity.
    #[must_use]
    pub const fn metric(&self) -> MetricKind {
        self.metric
    }

    /// Returns the existing diff classification.
    #[must_use]
    pub const fn change_kind(&self) -> BenchmarkEvaluationMetricChangeKindV1 {
        self.change_kind
    }

    /// Returns redacted baseline evidence when the metric existed.
    #[must_use]
    pub const fn baseline(&self) -> Option<&BenchmarkScorecardMetricSummaryV1> {
        self.baseline.as_ref()
    }

    /// Returns redacted revised evidence when the metric existed.
    #[must_use]
    pub const fn revised(&self) -> Option<&BenchmarkScorecardMetricSummaryV1> {
        self.revised.as_ref()
    }
}

impl BenchmarkScorecardMetricSummaryV1 {
    fn from_evidence(evidence: &BenchmarkDecisionMetricInput) -> Self {
        Self {
            metric: evidence.metric(),
            threshold_direction: evidence.threshold().direction(),
            threshold_value: evidence.threshold().value(),
            observed: evidence.observed(),
            sample_count: evidence.sample_count(),
            required_sample_count: evidence.required_sample_count(),
            has_complete_coverage: evidence.has_complete_coverage(),
            outcome: evidence.outcome(),
        }
    }
}

/// Errors that prevent a receipt from being projected against a different suite plan.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BenchmarkWorkspaceProjectionError {
    /// The receipt and plan do not reference the same immutable benchmark suite.
    #[error("benchmark workspace receipt and plan reference different suites")]
    SuiteMismatch {
        /// Suite identity stored by the receipt.
        receipt_suite_id: BenchmarkSuiteId,
        /// Suite identity supplied by the projection plan.
        plan_suite_id: BenchmarkSuiteId,
    },
    /// The receipt and plan reuse one suite identity for different immutable definitions.
    #[error("benchmark workspace receipt and plan use different definitions for suite {suite_id}")]
    PlanDefinitionMismatch {
        /// Reused suite identity whose immutable definition drifted.
        suite_id: BenchmarkSuiteId,
    },
    /// The baseline and revised projections refer to the same sealed execution cohort.
    #[error("benchmark workspace baseline and revised receipts reference the same cohort")]
    IdenticalReceiptCohort {
        /// Cohort identity repeated on both comparison sides.
        cohort_id: BenchmarkExecutionCohortId,
    },
    /// The existing evaluation diff rejected incomparable sealed decision evidence.
    #[error(transparent)]
    DecisionComparison(#[from] BenchmarkDecisionComparisonError),
}
