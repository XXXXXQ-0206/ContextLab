//! Pure execution-plan assembly for sealed benchmark definitions.

use crate::{
    BenchmarkCase, BenchmarkCaseId, BenchmarkDataset, BenchmarkDatasetId, BenchmarkSuite,
    EvaluationError, EvaluationRun, EvaluationRunId, MetricKind, MetricMeasurement,
};
use chrono::{DateTime, Utc};
use contextlab_context_core::ContextId;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use uuid::Uuid;

/// Stable composite identity for one sealed benchmark case execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BenchmarkExecutionCaseKey {
    dataset_id: BenchmarkDatasetId,
    case_id: BenchmarkCaseId,
}

impl BenchmarkExecutionCaseKey {
    const fn new(dataset_id: BenchmarkDatasetId, case_id: BenchmarkCaseId) -> Self {
        Self {
            dataset_id,
            case_id,
        }
    }

    /// Returns the sealed dataset identity.
    #[must_use]
    pub const fn dataset_id(self) -> BenchmarkDatasetId {
        self.dataset_id
    }

    /// Returns the immutable case identity.
    #[must_use]
    pub const fn case_id(self) -> BenchmarkCaseId {
        self.case_id
    }

    /// Returns the domain-owned deterministic run identity for this case and decision.
    #[must_use]
    pub fn deterministic_run_id(self, decision_namespace: Uuid) -> EvaluationRunId {
        EvaluationRunId::from_uuid(Uuid::new_v5(&decision_namespace, &case_key_name(self)))
    }
}

/// One immutable benchmark case scheduled by a sealed execution plan.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionCase {
    key: BenchmarkExecutionCaseKey,
    case: BenchmarkCase,
}

impl BenchmarkExecutionCase {
    /// Returns the sealed dataset identity that owns this case.
    #[must_use]
    pub const fn dataset_id(&self) -> BenchmarkDatasetId {
        self.key.dataset_id()
    }

    /// Returns the immutable case identity.
    #[must_use]
    pub const fn case_id(&self) -> BenchmarkCaseId {
        self.key.case_id()
    }

    /// Returns the composite identity used to bind evaluator results and runs.
    #[must_use]
    pub const fn key(&self) -> BenchmarkExecutionCaseKey {
        self.key
    }

    /// Returns the immutable benchmark case for a trusted evaluator port.
    #[must_use]
    pub const fn case(&self) -> &BenchmarkCase {
        &self.case
    }
}

/// Measurements produced for one immutable benchmark case.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkCaseExecutionResult {
    key: BenchmarkExecutionCaseKey,
    measurements: Vec<MetricMeasurement>,
}

impl BenchmarkCaseExecutionResult {
    /// Creates a result for one planned dataset case.
    pub fn new(
        dataset_id: BenchmarkDatasetId,
        case_id: BenchmarkCaseId,
        measurements: Vec<MetricMeasurement>,
    ) -> Result<Self, BenchmarkExecutionError> {
        let key = BenchmarkExecutionCaseKey::new(dataset_id, case_id);
        let mut metric_kinds = BTreeSet::new();
        for measurement in &measurements {
            if !metric_kinds.insert(measurement.kind()) {
                return Err(BenchmarkExecutionError::DuplicateMetric {
                    key,
                    metric: measurement.kind(),
                });
            }
        }
        Ok(Self { key, measurements })
    }

    /// Returns the sealed dataset identity.
    #[must_use]
    pub const fn dataset_id(&self) -> BenchmarkDatasetId {
        self.key.dataset_id()
    }

    /// Returns the immutable case identity.
    #[must_use]
    pub const fn case_id(&self) -> BenchmarkCaseId {
        self.key.case_id()
    }

    /// Returns the composite identity selected by the evaluator.
    #[must_use]
    pub const fn key(&self) -> BenchmarkExecutionCaseKey {
        self.key
    }

    /// Returns finite metric measurements recorded by the evaluator.
    #[must_use]
    pub fn measurements(&self) -> &[MetricMeasurement] {
        &self.measurements
    }
}

/// One case-to-run provenance record in an execution cohort.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutedCase {
    key: BenchmarkExecutionCaseKey,
    run: EvaluationRun,
}

impl BenchmarkExecutedCase {
    /// Returns the immutable benchmark case identity that produced this run.
    #[must_use]
    pub const fn key(&self) -> BenchmarkExecutionCaseKey {
        self.key
    }

    /// Returns the deterministically identified evaluation run.
    #[must_use]
    pub const fn run(&self) -> &EvaluationRun {
        &self.run
    }
}

/// A deterministic, provenance-preserving cohort assembled from sealed cases.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionCohort {
    entries: Vec<BenchmarkExecutedCase>,
}

impl BenchmarkExecutionCohort {
    /// Returns case-to-run entries in stable case-key order.
    #[must_use]
    pub fn entries(&self) -> &[BenchmarkExecutedCase] {
        &self.entries
    }

    /// Returns the run cohort for the domain-owned scorecard and regression policy.
    #[must_use]
    pub fn runs(&self) -> Vec<EvaluationRun> {
        self.entries.iter().map(|entry| entry.run.clone()).collect()
    }
}

/// A deterministic execution schedule for one sealed benchmark suite.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionPlan {
    suite: BenchmarkSuite,
    datasets: Vec<BenchmarkDataset>,
    cases: Vec<BenchmarkExecutionCase>,
}

impl BenchmarkExecutionPlan {
    /// Creates an execution plan from the exact datasets named by a suite.
    pub fn new(
        suite: BenchmarkSuite,
        mut datasets: Vec<BenchmarkDataset>,
    ) -> Result<Self, BenchmarkExecutionError> {
        datasets.sort_by_key(BenchmarkDataset::id);
        let dataset_ids = datasets
            .iter()
            .map(BenchmarkDataset::id)
            .collect::<Vec<_>>();
        if dataset_ids != suite.dataset_ids() {
            return Err(BenchmarkExecutionError::DatasetMembershipMismatch);
        }

        let cases = datasets
            .iter()
            .flat_map(|dataset| {
                let dataset_id = dataset.id();
                dataset
                    .cases()
                    .iter()
                    .cloned()
                    .map(move |case| BenchmarkExecutionCase {
                        key: BenchmarkExecutionCaseKey::new(dataset_id, case.id()),
                        case,
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(Self {
            suite,
            datasets,
            cases,
        })
    }

    /// Returns the suite whose policy will later evaluate assembled runs.
    #[must_use]
    pub const fn suite(&self) -> &BenchmarkSuite {
        &self.suite
    }

    /// Returns immutable cases in stable dataset and case identifier order.
    #[must_use]
    pub fn cases(&self) -> &[BenchmarkExecutionCase] {
        &self.cases
    }

    /// Returns validated sealed datasets in stable identifier order for internal projections.
    #[must_use]
    pub(crate) fn datasets(&self) -> &[BenchmarkDataset] {
        &self.datasets
    }

    /// Returns the number of evaluator calls required by this plan.
    #[must_use]
    pub fn case_count(&self) -> usize {
        self.cases.len()
    }

    /// Builds a deterministic, provenance-preserving run cohort from matching results.
    pub fn assemble_cohort(
        &self,
        decision_namespace: Uuid,
        context_id: ContextId,
        model_version: &str,
        temperature: f32,
        executed_at: DateTime<Utc>,
        results: Vec<BenchmarkCaseExecutionResult>,
    ) -> Result<BenchmarkExecutionCohort, BenchmarkExecutionError> {
        let planned_cases = self
            .cases
            .iter()
            .map(BenchmarkExecutionCase::key)
            .collect::<BTreeSet<_>>();
        let mut indexed_results = BTreeMap::new();

        for result in results {
            let key = result.key();
            if !planned_cases.contains(&key) {
                return Err(BenchmarkExecutionError::UnknownCaseResult { key });
            }
            if indexed_results.insert(key, result).is_some() {
                return Err(BenchmarkExecutionError::DuplicateCaseResult { key });
            }
        }

        self.cases
            .iter()
            .map(|case| {
                let key = case.key();
                let result = indexed_results
                    .remove(&key)
                    .ok_or(BenchmarkExecutionError::MissingCaseResult { key })?;
                let run_id = key.deterministic_run_id(decision_namespace);
                let run = EvaluationRun::with_id(
                    run_id,
                    context_id,
                    model_version,
                    temperature,
                    result.measurements,
                    executed_at,
                )
                .map_err(BenchmarkExecutionError::InvalidEvaluationRun)?;
                Ok(BenchmarkExecutedCase { key, run })
            })
            .collect::<Result<Vec<_>, BenchmarkExecutionError>>()
            .map(|entries| BenchmarkExecutionCohort { entries })
    }

    /// Reconstructs evaluator-shaped results from exact stored runs by deterministic identity.
    #[allow(clippy::too_many_arguments)]
    pub fn reconstruct_results_from_runs(
        &self,
        decision_namespace: Uuid,
        context_id: ContextId,
        model_version: &str,
        temperature: f32,
        executed_at: DateTime<Utc>,
        runs: Vec<EvaluationRun>,
    ) -> Result<Vec<BenchmarkCaseExecutionResult>, BenchmarkExecutionError> {
        let expected_runs = self
            .cases
            .iter()
            .map(|case| {
                (
                    case.key().deterministic_run_id(decision_namespace),
                    case.key(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut indexed_runs = BTreeMap::new();

        for run in runs {
            let run_id = run.id();
            if !expected_runs.contains_key(&run_id) {
                return Err(BenchmarkExecutionError::UnknownStoredRun { run_id });
            }
            if run.context_id() != context_id
                || run.model_version() != model_version
                || run.temperature().to_bits() != temperature.to_bits()
                || run.executed_at() != executed_at
            {
                return Err(BenchmarkExecutionError::StoredRunProvenanceMismatch { run_id });
            }
            if indexed_runs.insert(run_id, run).is_some() {
                return Err(BenchmarkExecutionError::DuplicateStoredRun { run_id });
            }
        }

        self.cases
            .iter()
            .map(|case| {
                let key = case.key();
                let run_id = key.deterministic_run_id(decision_namespace);
                let run = indexed_runs
                    .remove(&run_id)
                    .ok_or(BenchmarkExecutionError::MissingStoredRun { key })?;
                BenchmarkCaseExecutionResult::new(
                    key.dataset_id(),
                    key.case_id(),
                    run.measurements().to_vec(),
                )
            })
            .collect()
    }
}

fn case_key_name(key: BenchmarkExecutionCaseKey) -> [u8; 32] {
    let mut name = [0_u8; 32];
    name[..16].copy_from_slice(key.dataset_id().as_uuid().as_bytes());
    name[16..].copy_from_slice(key.case_id().as_uuid().as_bytes());
    name
}

/// Errors produced while scheduling or assembling a benchmark execution.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BenchmarkExecutionError {
    /// Supplied datasets did not exactly match suite membership.
    #[error("benchmark execution datasets do not exactly match suite membership")]
    DatasetMembershipMismatch,
    /// An evaluator returned a result for an unplanned case.
    #[error("benchmark execution returned an unknown case result")]
    UnknownCaseResult {
        /// Composite identity supplied by the evaluator.
        key: BenchmarkExecutionCaseKey,
    },
    /// An evaluator returned more than one result for a planned case.
    #[error("benchmark execution repeated a case result")]
    DuplicateCaseResult {
        /// Composite identity supplied by the evaluator.
        key: BenchmarkExecutionCaseKey,
    },
    /// An evaluator failed to return a result for a planned case.
    #[error("benchmark execution omitted a planned case result")]
    MissingCaseResult {
        /// Composite identity missing from evaluator output.
        key: BenchmarkExecutionCaseKey,
    },
    /// A stored replay run did not belong to any deterministic planned case identity.
    #[error("benchmark execution replay contained an unknown stored run")]
    UnknownStoredRun {
        /// Stored run identity that was not derived from the sealed plan.
        run_id: EvaluationRunId,
    },
    /// Stored replay data repeated one deterministic run identity.
    #[error("benchmark execution replay repeated a stored run")]
    DuplicateStoredRun {
        /// Repeated stored run identity.
        run_id: EvaluationRunId,
    },
    /// Stored replay data omitted the deterministic run for one planned case.
    #[error("benchmark execution replay omitted a stored run")]
    MissingStoredRun {
        /// Planned case whose deterministic run was unavailable.
        key: BenchmarkExecutionCaseKey,
    },
    /// A deterministic stored run did not match the receipt provenance being reconstructed.
    #[error("benchmark execution replay stored run provenance does not match")]
    StoredRunProvenanceMismatch {
        /// Stored run identity carrying mismatched immutable provenance.
        run_id: EvaluationRunId,
    },
    /// An evaluator repeated one metric in a case result.
    #[error("benchmark execution repeated metric {metric:?} for a case result")]
    DuplicateMetric {
        /// Composite identity owning the malformed result.
        key: BenchmarkExecutionCaseKey,
        /// Repeated metric identity.
        metric: MetricKind,
    },
    /// A result could not form a valid evaluation run.
    #[error(transparent)]
    InvalidEvaluationRun(#[from] EvaluationError),
}
