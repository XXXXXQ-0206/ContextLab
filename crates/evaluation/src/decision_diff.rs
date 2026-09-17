use crate::{MetricKind, RegressionCheckStatus, RegressionDecisionStatus, RegressionThreshold};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Validated, policy-preserving evidence for one metric in a sealed benchmark decision.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkDecisionMetricInput {
    threshold: RegressionThreshold,
    observed: Option<f64>,
    sample_count: usize,
    required_sample_count: usize,
    has_complete_coverage: bool,
    outcome: RegressionCheckStatus,
}

impl BenchmarkDecisionMetricInput {
    /// Creates validated metric evidence without re-evaluating its policy outcome.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        threshold: RegressionThreshold,
        observed: Option<f64>,
        sample_count: usize,
        required_sample_count: usize,
        has_complete_coverage: bool,
        outcome: RegressionCheckStatus,
    ) -> Result<Self, BenchmarkDecisionComparisonError> {
        if observed.is_some_and(|value| !value.is_finite()) {
            return Err(BenchmarkDecisionComparisonError::NonFiniteObserved {
                metric: threshold.metric(),
            });
        }
        if sample_count > required_sample_count {
            return Err(
                BenchmarkDecisionComparisonError::SampleCountExceedsRequired {
                    metric: threshold.metric(),
                    sample_count,
                    required_sample_count,
                },
            );
        }

        Ok(Self {
            threshold,
            observed,
            sample_count,
            required_sample_count,
            has_complete_coverage,
            outcome,
        })
    }

    /// Returns the metric identity.
    #[must_use]
    pub const fn metric(&self) -> MetricKind {
        self.threshold.metric()
    }

    /// Returns the preserved threshold evidence.
    #[must_use]
    pub const fn threshold(&self) -> RegressionThreshold {
        self.threshold
    }

    /// Returns the observed aggregate value, if recorded.
    #[must_use]
    pub const fn observed(&self) -> Option<f64> {
        self.observed
    }

    /// Returns the recorded sample count.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Returns the required sample count.
    #[must_use]
    pub const fn required_sample_count(&self) -> usize {
        self.required_sample_count
    }

    /// Returns the preserved coverage fact.
    #[must_use]
    pub const fn has_complete_coverage(&self) -> bool {
        self.has_complete_coverage
    }

    /// Returns the domain-produced metric outcome.
    #[must_use]
    pub const fn outcome(&self) -> RegressionCheckStatus {
        self.outcome
    }
}

/// One immutable benchmark-decision projection suitable for comparison.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkDecisionComparisonInput {
    status: RegressionDecisionStatus,
    comparability_fingerprint: String,
    metrics: BTreeMap<MetricKind, BenchmarkDecisionMetricInput>,
}

impl BenchmarkDecisionComparisonInput {
    /// Creates a comparison input with one unique evidence record per metric.
    pub fn new(
        status: RegressionDecisionStatus,
        comparability_fingerprint: impl Into<String>,
        metrics: Vec<BenchmarkDecisionMetricInput>,
    ) -> Result<Self, BenchmarkDecisionComparisonError> {
        let comparability_fingerprint = comparability_fingerprint.into();
        if comparability_fingerprint.trim().is_empty() {
            return Err(BenchmarkDecisionComparisonError::EmptyComparabilityFingerprint);
        }
        let mut indexed_metrics = BTreeMap::new();
        for metric in metrics {
            let kind = metric.metric();
            if indexed_metrics.insert(kind, metric).is_some() {
                return Err(BenchmarkDecisionComparisonError::DuplicateMetric { metric: kind });
            }
        }

        Ok(Self {
            status,
            comparability_fingerprint,
            metrics: indexed_metrics,
        })
    }

    /// Returns the preserved overall decision status.
    #[must_use]
    pub const fn status(&self) -> RegressionDecisionStatus {
        self.status
    }

    /// Returns the immutable comparability fingerprint.
    #[must_use]
    pub fn comparability_fingerprint(&self) -> &str {
        &self.comparability_fingerprint
    }

    /// Returns metric evidence in stable metric order.
    #[must_use]
    pub const fn metrics(&self) -> &BTreeMap<MetricKind, BenchmarkDecisionMetricInput> {
        &self.metrics
    }
}

/// One change between corresponding metric evidence in two sealed decisions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum BenchmarkDecisionMetricDiff {
    /// The metric exists only in the revised decision.
    Added {
        /// Stable metric identity.
        metric: MetricKind,
        /// Revised metric evidence.
        revised: BenchmarkDecisionMetricInput,
    },
    /// The metric exists only in the baseline decision.
    Removed {
        /// Stable metric identity.
        metric: MetricKind,
        /// Baseline metric evidence.
        baseline: BenchmarkDecisionMetricInput,
    },
    /// The metric exists in both decisions but its evidence changed.
    Modified {
        /// Stable metric identity.
        metric: MetricKind,
        /// Baseline metric evidence.
        baseline: BenchmarkDecisionMetricInput,
        /// Revised metric evidence.
        revised: BenchmarkDecisionMetricInput,
    },
}

impl BenchmarkDecisionMetricDiff {
    /// Returns the stable metric identity.
    #[must_use]
    pub const fn metric(&self) -> MetricKind {
        match self {
            Self::Added { metric, .. }
            | Self::Removed { metric, .. }
            | Self::Modified { metric, .. } => *metric,
        }
    }

    /// Returns baseline evidence when present.
    #[must_use]
    pub const fn baseline(&self) -> Option<&BenchmarkDecisionMetricInput> {
        match self {
            Self::Added { .. } => None,
            Self::Removed { baseline, .. } | Self::Modified { baseline, .. } => Some(baseline),
        }
    }

    /// Returns revised evidence when present.
    #[must_use]
    pub const fn revised(&self) -> Option<&BenchmarkDecisionMetricInput> {
        match self {
            Self::Removed { .. } => None,
            Self::Added { revised, .. } | Self::Modified { revised, .. } => Some(revised),
        }
    }
}

/// Deterministic comparison of two domain-produced benchmark decision projections.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkDecisionDiff {
    status_change: Option<(RegressionDecisionStatus, RegressionDecisionStatus)>,
    metric_changes: Vec<BenchmarkDecisionMetricDiff>,
}

impl BenchmarkDecisionDiff {
    /// Compares two immutable projections without recalculating evaluation policy.
    pub fn between(
        baseline: BenchmarkDecisionComparisonInput,
        revised: BenchmarkDecisionComparisonInput,
    ) -> Result<Self, BenchmarkDecisionComparisonError> {
        if baseline.comparability_fingerprint != revised.comparability_fingerprint {
            return Err(BenchmarkDecisionComparisonError::ComparabilityMismatch {
                baseline: baseline.comparability_fingerprint,
                revised: revised.comparability_fingerprint,
            });
        }
        let metric_kinds = baseline
            .metrics
            .keys()
            .chain(revised.metrics.keys())
            .copied()
            .collect::<BTreeSet<_>>();
        let metric_changes = metric_kinds
            .into_iter()
            .filter_map(|metric| {
                match (baseline.metrics.get(&metric), revised.metrics.get(&metric)) {
                    (None, Some(revised)) => Some(BenchmarkDecisionMetricDiff::Added {
                        metric,
                        revised: revised.clone(),
                    }),
                    (Some(baseline), None) => Some(BenchmarkDecisionMetricDiff::Removed {
                        metric,
                        baseline: baseline.clone(),
                    }),
                    (Some(baseline), Some(revised)) if baseline != revised => {
                        Some(BenchmarkDecisionMetricDiff::Modified {
                            metric,
                            baseline: baseline.clone(),
                            revised: revised.clone(),
                        })
                    }
                    (Some(_), Some(_)) | (None, None) => None,
                }
            })
            .collect();

        Ok(Self {
            status_change: (baseline.status != revised.status)
                .then_some((baseline.status, revised.status)),
            metric_changes,
        })
    }

    /// Returns the overall status transition when it changed.
    #[must_use]
    pub const fn status_change(
        &self,
    ) -> Option<(RegressionDecisionStatus, RegressionDecisionStatus)> {
        self.status_change
    }

    /// Returns changed metric evidence in stable metric order.
    #[must_use]
    pub fn metric_changes(&self) -> &[BenchmarkDecisionMetricDiff] {
        &self.metric_changes
    }
}

/// Structural errors that prevent safe decision comparison.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BenchmarkDecisionComparisonError {
    /// The immutable decision projection did not provide a fingerprint.
    #[error("comparability fingerprint must not be empty")]
    EmptyComparabilityFingerprint,
    /// The two immutable decisions describe different evaluation conditions.
    #[error("benchmark decision comparability fingerprints differ")]
    ComparabilityMismatch {
        /// Baseline fingerprint.
        baseline: String,
        /// Revised fingerprint.
        revised: String,
    },
    /// One metric evidence value was non-finite.
    #[error("observed value for {metric:?} must be finite")]
    NonFiniteObserved {
        /// Metric with the invalid value.
        metric: MetricKind,
    },
    /// A metric claimed more samples than its required cohort size.
    #[error(
        "sample count {sample_count} for {metric:?} exceeds required count {required_sample_count}"
    )]
    SampleCountExceedsRequired {
        /// Metric with the invalid count.
        metric: MetricKind,
        /// Recorded sample count.
        sample_count: usize,
        /// Recorded required sample count.
        required_sample_count: usize,
    },
    /// A projection contained the same metric more than once.
    #[error("duplicate decision metric {metric:?}")]
    DuplicateMetric {
        /// Duplicated metric identity.
        metric: MetricKind,
    },
}
