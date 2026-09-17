//! Immutable benchmark dataset and suite contracts.

use crate::{
    EvaluationRun, EvaluationRunId, MetricKind, RegressionDecision, RegressionThreshold, Scorecard,
};
use contextlab_context_core::{DomainValidationError, NonEmptyString};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

macro_rules! benchmark_id {
    ($name:ident, $label:literal) => {
        #[doc = concat!("Stable identifier for a benchmark ", $label, ".")]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new random identifier.
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

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

benchmark_id!(BenchmarkCaseId, "case");
benchmark_id!(BenchmarkDatasetId, "dataset");
benchmark_id!(BenchmarkSuiteId, "suite");

/// Oracle semantics for a benchmark case.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "mode", content = "value", rename_all = "snake_case")]
pub enum BenchmarkExpectedOutput {
    /// The case does not define an exact output oracle.
    Unspecified,
    /// The case requires this exact structured output, including JSON null.
    Exact(Value),
}

/// One immutable input and optional expected output in a benchmark dataset.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkCase {
    id: BenchmarkCaseId,
    name: NonEmptyString,
    input: Value,
    expected_output: BenchmarkExpectedOutput,
}

impl BenchmarkCase {
    /// Creates a benchmark case with a new identifier.
    pub fn new(
        name: impl Into<String>,
        input: Value,
        expected_output: BenchmarkExpectedOutput,
    ) -> Result<Self, BenchmarkValidationError> {
        Self::with_id(BenchmarkCaseId::new(), name, input, expected_output)
    }

    /// Rehydrates a benchmark case with an existing identifier.
    pub fn with_id(
        id: BenchmarkCaseId,
        name: impl Into<String>,
        input: Value,
        expected_output: BenchmarkExpectedOutput,
    ) -> Result<Self, BenchmarkValidationError> {
        Ok(Self {
            id,
            name: NonEmptyString::new("benchmark case name", name)?,
            input,
            expected_output,
        })
    }

    /// Returns the case identifier.
    #[must_use]
    pub const fn id(&self) -> BenchmarkCaseId {
        self.id
    }

    /// Returns the validated case name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the structured case input.
    #[must_use]
    pub const fn input(&self) -> &Value {
        &self.input
    }

    /// Returns the structured expected output, if present.
    #[must_use]
    pub const fn expected_output(&self) -> &BenchmarkExpectedOutput {
        &self.expected_output
    }
}

/// An immutable reusable collection of benchmark cases.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkDataset {
    id: BenchmarkDatasetId,
    name: NonEmptyString,
    cases: Vec<BenchmarkCase>,
}

impl BenchmarkDataset {
    /// Creates a benchmark dataset with a new identifier.
    pub fn new(
        name: impl Into<String>,
        cases: Vec<BenchmarkCase>,
    ) -> Result<Self, BenchmarkValidationError> {
        Self::with_id(BenchmarkDatasetId::new(), name, cases)
    }

    /// Rehydrates a benchmark dataset with an existing identifier.
    pub fn with_id(
        id: BenchmarkDatasetId,
        name: impl Into<String>,
        mut cases: Vec<BenchmarkCase>,
    ) -> Result<Self, BenchmarkValidationError> {
        if cases.is_empty() {
            return Err(BenchmarkValidationError::EmptyDataset);
        }
        cases.sort_by_key(BenchmarkCase::id);
        if let Some(duplicate) = cases.windows(2).find(|pair| pair[0].id() == pair[1].id()) {
            return Err(BenchmarkValidationError::DuplicateCase {
                case_id: duplicate[0].id(),
            });
        }

        Ok(Self {
            id,
            name: NonEmptyString::new("benchmark dataset name", name)?,
            cases,
        })
    }

    /// Returns the dataset identifier.
    #[must_use]
    pub const fn id(&self) -> BenchmarkDatasetId {
        self.id
    }

    /// Returns the validated dataset name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns cases in stable identifier order.
    #[must_use]
    pub fn cases(&self) -> &[BenchmarkCase] {
        &self.cases
    }
}

/// An immutable benchmark policy over reusable datasets.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkSuite {
    id: BenchmarkSuiteId,
    name: NonEmptyString,
    dataset_ids: Vec<BenchmarkDatasetId>,
    thresholds: Vec<RegressionThreshold>,
}

impl BenchmarkSuite {
    /// Creates a benchmark suite with a new identifier.
    pub fn new(
        name: impl Into<String>,
        dataset_ids: Vec<BenchmarkDatasetId>,
        thresholds: Vec<RegressionThreshold>,
    ) -> Result<Self, BenchmarkValidationError> {
        Self::with_id(BenchmarkSuiteId::new(), name, dataset_ids, thresholds)
    }

    /// Rehydrates a benchmark suite with an existing identifier.
    pub fn with_id(
        id: BenchmarkSuiteId,
        name: impl Into<String>,
        mut dataset_ids: Vec<BenchmarkDatasetId>,
        mut thresholds: Vec<RegressionThreshold>,
    ) -> Result<Self, BenchmarkValidationError> {
        if dataset_ids.is_empty() {
            return Err(BenchmarkValidationError::EmptySuiteDatasets);
        }
        if thresholds.is_empty() {
            return Err(BenchmarkValidationError::EmptySuiteThresholds);
        }

        dataset_ids.sort_unstable();
        if let Some(duplicate) = dataset_ids.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(BenchmarkValidationError::DuplicateDataset {
                dataset_id: duplicate[0],
            });
        }

        thresholds.sort_by_key(|threshold| threshold.metric());
        if let Some(duplicate) = thresholds
            .windows(2)
            .find(|pair| pair[0].metric() == pair[1].metric())
        {
            return Err(BenchmarkValidationError::DuplicateMetricThreshold {
                metric: duplicate[0].metric(),
            });
        }

        Ok(Self {
            id,
            name: NonEmptyString::new("benchmark suite name", name)?,
            dataset_ids,
            thresholds,
        })
    }

    /// Returns the suite identifier.
    #[must_use]
    pub const fn id(&self) -> BenchmarkSuiteId {
        self.id
    }

    /// Returns the validated suite name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns dataset membership in stable identifier order.
    #[must_use]
    pub fn dataset_ids(&self) -> &[BenchmarkDatasetId] {
        &self.dataset_ids
    }

    /// Returns thresholds in stable metric order.
    #[must_use]
    pub fn thresholds(&self) -> &[RegressionThreshold] {
        &self.thresholds
    }

    /// Applies this suite's thresholds to a scorecard.
    #[must_use]
    pub fn evaluate(&self, scorecard: &Scorecard) -> RegressionDecision {
        RegressionDecision::evaluate(&self.thresholds, scorecard)
    }

    /// Evaluates a run cohort through the domain-owned scorecard and regression policy.
    #[must_use]
    pub fn evaluate_runs(&self, runs: &[EvaluationRun]) -> BenchmarkEvaluation {
        BenchmarkEvaluation::from_runs(self, runs)
    }
}

/// Domain-owned scorecard and regression decision for one benchmark suite run cohort.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkEvaluation {
    suite_id: BenchmarkSuiteId,
    runs: Vec<EvaluationRun>,
    run_ids: Vec<EvaluationRunId>,
    scorecard: Scorecard,
    decision: RegressionDecision,
}

impl BenchmarkEvaluation {
    /// Calculates a scorecard and deterministic regression decision for one run cohort.
    #[must_use]
    pub fn from_runs(suite: &BenchmarkSuite, runs: &[EvaluationRun]) -> Self {
        let mut runs = runs.to_vec();
        runs.sort_by_key(EvaluationRun::id);
        let run_ids = runs.iter().map(EvaluationRun::id).collect::<Vec<_>>();
        let scorecard = Scorecard::from_runs(&runs);
        let decision = suite.evaluate(&scorecard);
        Self {
            suite_id: suite.id(),
            runs,
            run_ids,
            scorecard,
            decision,
        }
    }

    /// Returns the evaluated suite identity.
    #[must_use]
    pub const fn suite_id(&self) -> BenchmarkSuiteId {
        self.suite_id
    }

    /// Returns contributing run identities in stable order.
    #[must_use]
    pub fn run_ids(&self) -> &[EvaluationRunId] {
        &self.run_ids
    }

    /// Returns the exact run payloads used for this domain calculation.
    #[must_use]
    pub fn runs(&self) -> &[EvaluationRun] {
        &self.runs
    }

    /// Returns the domain-owned aggregate scorecard.
    #[must_use]
    pub const fn scorecard(&self) -> &Scorecard {
        &self.scorecard
    }

    /// Returns the domain-owned regression decision.
    #[must_use]
    pub const fn decision(&self) -> &RegressionDecision {
        &self.decision
    }
}

/// Errors produced by benchmark constructors.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BenchmarkValidationError {
    /// A benchmark name failed shared domain validation.
    #[error(transparent)]
    InvalidName(#[from] DomainValidationError),
    /// A dataset contained no cases.
    #[error("benchmark dataset must contain at least one case")]
    EmptyDataset,
    /// A dataset repeated one case identifier.
    #[error("benchmark dataset contains duplicate case {case_id}")]
    DuplicateCase {
        /// Repeated case identifier.
        case_id: BenchmarkCaseId,
    },
    /// A suite contained no datasets.
    #[error("benchmark suite must contain at least one dataset")]
    EmptySuiteDatasets,
    /// A suite repeated one dataset identifier.
    #[error("benchmark suite contains duplicate dataset {dataset_id}")]
    DuplicateDataset {
        /// Repeated dataset identifier.
        dataset_id: BenchmarkDatasetId,
    },
    /// A suite contained no regression thresholds.
    #[error("benchmark suite must contain at least one regression threshold")]
    EmptySuiteThresholds,
    /// A suite repeated one metric threshold.
    #[error("benchmark suite contains duplicate threshold for {metric:?}")]
    DuplicateMetricThreshold {
        /// Repeated metric kind.
        metric: MetricKind,
    },
}
