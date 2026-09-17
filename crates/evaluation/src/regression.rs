//! Deterministic benchmark regression policy.

use crate::{MetricKind, Scorecard};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Direction used to interpret an inclusive metric threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdDirection {
    /// The observed metric must be greater than or equal to the threshold.
    Minimum,
    /// The observed metric must be less than or equal to the threshold.
    Maximum,
}

/// A finite inclusive threshold for one evaluation metric.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RegressionThreshold {
    metric: MetricKind,
    direction: ThresholdDirection,
    value: f64,
}

impl RegressionThreshold {
    /// Creates a validated regression threshold.
    pub fn new(
        metric: MetricKind,
        direction: ThresholdDirection,
        value: f64,
    ) -> Result<Self, RegressionValidationError> {
        if !value.is_finite() {
            return Err(RegressionValidationError::NonFiniteThreshold { metric });
        }

        Ok(Self {
            metric,
            direction,
            value,
        })
    }

    /// Returns the threshold metric.
    #[must_use]
    pub const fn metric(self) -> MetricKind {
        self.metric
    }

    /// Returns the comparison direction.
    #[must_use]
    pub const fn direction(self) -> ThresholdDirection {
        self.direction
    }

    /// Returns the inclusive threshold value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Derives the policy outcome for one persisted metric evidence record.
    #[must_use]
    pub fn status_for_evidence(
        self,
        observed: Option<f64>,
        has_complete_coverage: bool,
    ) -> RegressionCheckStatus {
        if !has_complete_coverage {
            return RegressionCheckStatus::InsufficientData;
        }
        let Some(observed) = observed.filter(|value| value.is_finite()) else {
            return RegressionCheckStatus::InsufficientData;
        };
        if self.accepts(observed) {
            RegressionCheckStatus::Passed
        } else {
            RegressionCheckStatus::Regressed
        }
    }

    const fn accepts(self, observed: f64) -> bool {
        match self.direction {
            ThresholdDirection::Minimum => observed >= self.value,
            ThresholdDirection::Maximum => observed <= self.value,
        }
    }
}

/// Errors produced by regression-policy constructors.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum RegressionValidationError {
    /// A threshold value was NaN or infinite.
    #[error("threshold for {metric:?} must be finite")]
    NonFiniteThreshold {
        /// Metric whose threshold was invalid.
        metric: MetricKind,
    },
}

/// Overall result of applying a suite's thresholds to a scorecard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegressionDecisionStatus {
    /// Every required metric is present and satisfies its threshold.
    Passed,
    /// At least one completely observed metric breaches its threshold.
    Regressed,
    /// At least one required metric is absent.
    InsufficientData,
}

/// Outcome of one metric threshold check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegressionCheckStatus {
    /// Complete evidence satisfies the threshold.
    Passed,
    /// Complete evidence breaches the threshold.
    Regressed,
    /// Evidence is missing, partial, duplicated, or invalid.
    InsufficientData,
}

/// The result of checking one scorecard metric against one threshold.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct RegressionCheck {
    threshold: RegressionThreshold,
    observed: Option<f64>,
    has_complete_coverage: bool,
    passed: bool,
    status: RegressionCheckStatus,
}

impl RegressionCheck {
    /// Returns the checked metric.
    #[must_use]
    pub const fn metric(&self) -> MetricKind {
        self.threshold.metric()
    }

    /// Returns the applied threshold.
    #[must_use]
    pub const fn threshold(&self) -> RegressionThreshold {
        self.threshold
    }

    /// Returns the observed scorecard value, if present.
    #[must_use]
    pub const fn observed(&self) -> Option<f64> {
        self.observed
    }

    /// Returns whether every scorecard run contributed this metric.
    #[must_use]
    pub const fn has_complete_coverage(&self) -> bool {
        self.has_complete_coverage
    }

    /// Returns whether the observed value satisfies the threshold.
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.passed
    }

    /// Returns the explicit metric-level outcome.
    #[must_use]
    pub const fn status(&self) -> RegressionCheckStatus {
        self.status
    }
}

/// Deterministic outcome of applying all suite thresholds to one scorecard.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegressionDecision {
    status: RegressionDecisionStatus,
    checks: Vec<RegressionCheck>,
}

impl RegressionDecision {
    pub(crate) fn evaluate(thresholds: &[RegressionThreshold], scorecard: &Scorecard) -> Self {
        let checks = thresholds
            .iter()
            .map(|threshold| {
                let observed = scorecard.average(threshold.metric());
                let has_complete_coverage = scorecard.has_complete_metric(threshold.metric());
                let status = threshold.status_for_evidence(observed, has_complete_coverage);
                let passed = status == RegressionCheckStatus::Passed;
                RegressionCheck {
                    threshold: *threshold,
                    observed,
                    has_complete_coverage,
                    passed,
                    status,
                }
            })
            .collect::<Vec<_>>();
        let status = Self::status_from_check_statuses(checks.iter().map(RegressionCheck::status));

        Self { status, checks }
    }

    /// Returns the overall regression status.
    #[must_use]
    pub const fn status(&self) -> RegressionDecisionStatus {
        self.status
    }

    /// Returns the stable per-metric checks.
    #[must_use]
    pub fn checks(&self) -> &[RegressionCheck] {
        &self.checks
    }

    /// Derives the overall status using the stable regression-over-insufficient precedence.
    #[must_use]
    pub fn status_from_check_statuses(
        statuses: impl IntoIterator<Item = RegressionCheckStatus>,
    ) -> RegressionDecisionStatus {
        let mut has_incomplete_data = false;
        for status in statuses {
            match status {
                RegressionCheckStatus::Regressed => return RegressionDecisionStatus::Regressed,
                RegressionCheckStatus::InsufficientData => has_incomplete_data = true,
                RegressionCheckStatus::Passed => {}
            }
        }
        if has_incomplete_data {
            RegressionDecisionStatus::InsufficientData
        } else {
            RegressionDecisionStatus::Passed
        }
    }
}
