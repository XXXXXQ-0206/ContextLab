//! Private immutable benchmark definition, run, and decision evidence contracts.

use crate::{IdempotencyKey, RequestDigest, StorageRepositoryError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_context_core::{ContextId, NonEmptyString, ProjectId};
use contextlab_evaluation::{
    BenchmarkDataset, BenchmarkDatasetId, BenchmarkDecisionComparisonError,
    BenchmarkDecisionComparisonInput, BenchmarkDecisionDiff, BenchmarkDecisionMetricInput,
    BenchmarkEvaluation, BenchmarkSuite, BenchmarkSuiteId, EvaluationRun, EvaluationRunId,
    MetricKind, RegressionCheckStatus, RegressionDecisionStatus, RegressionThreshold,
};
use contextlab_versioning::CommitId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Stable identity of one immutable benchmark decision aggregate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BenchmarkDecisionId(Uuid);

impl BenchmarkDecisionId {
    /// Creates a new decision identity.
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

impl Default for BenchmarkDecisionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Exact immutable decision identity used as one side of a comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkDecisionComparisonScope {
    context_commit_id: CommitId,
    decision_id: BenchmarkDecisionId,
}

impl BenchmarkDecisionComparisonScope {
    /// Creates one exact commit-and-decision scope.
    #[must_use]
    pub const fn new(context_commit_id: CommitId, decision_id: BenchmarkDecisionId) -> Self {
        Self {
            context_commit_id,
            decision_id,
        }
    }

    /// Returns the Context commit identity.
    #[must_use]
    pub const fn context_commit_id(self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the decision identity.
    #[must_use]
    pub const fn decision_id(self) -> BenchmarkDecisionId {
        self.decision_id
    }
}

/// Two exact decision lookups loaded from one consistent repository snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDecisionPair {
    baseline: Option<BenchmarkDecisionEvidence>,
    revised: Option<BenchmarkDecisionEvidence>,
}

impl BenchmarkDecisionPair {
    /// Creates a pair of exact immutable decision lookup results.
    #[must_use]
    pub const fn new(
        baseline: Option<BenchmarkDecisionEvidence>,
        revised: Option<BenchmarkDecisionEvidence>,
    ) -> Self {
        Self { baseline, revised }
    }

    /// Returns the baseline decision when its exact scope exists.
    #[must_use]
    pub const fn baseline(&self) -> Option<&BenchmarkDecisionEvidence> {
        self.baseline.as_ref()
    }

    /// Returns the revised decision when its exact scope exists.
    #[must_use]
    pub const fn revised(&self) -> Option<&BenchmarkDecisionEvidence> {
        self.revised.as_ref()
    }
}

impl fmt::Display for BenchmarkDecisionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Errors produced while preparing immutable benchmark evidence.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BenchmarkEvidenceError {
    /// No dataset definitions were supplied.
    #[error("benchmark evidence must include dataset membership")]
    EmptyDatasets,
    /// No evaluation runs were supplied.
    #[error("benchmark evidence must include at least one evaluation run")]
    EmptyRuns,
    /// One dataset identifier was repeated.
    #[error("benchmark evidence repeats dataset {dataset_id}")]
    DuplicateDataset {
        /// Repeated dataset identifier.
        dataset_id: BenchmarkDatasetId,
    },
    /// One run identifier was repeated.
    #[error("benchmark evidence repeats run {run_id:?}")]
    DuplicateRun {
        /// Repeated run identifier.
        run_id: EvaluationRunId,
    },
    /// Supplied definitions do not exactly match suite membership.
    #[error("benchmark evidence dataset membership does not match the suite")]
    DatasetMembershipMismatch,
    /// Runs do not belong to one Context.
    #[error("benchmark evidence runs must belong to one Context")]
    RunContextMismatch,
    /// Runs do not use one model version and temperature.
    #[error("benchmark evidence runs must share one model configuration")]
    RunModelConfigurationMismatch,
    /// A run cannot satisfy persisted benchmark evidence constraints.
    #[error("benchmark evaluation run {run_id:?} is invalid for persistence")]
    InvalidRunConfiguration {
        /// Invalid evaluation-run identity.
        run_id: EvaluationRunId,
    },
    /// Domain-evaluated evidence did not belong to the supplied suite and runs.
    #[error("benchmark domain evaluation does not match the supplied suite and runs")]
    DomainEvaluationMismatch,
    /// An evaluator identity was invalid.
    #[error("benchmark evaluator identity is invalid")]
    InvalidEvaluatorIdentity,
    /// A persisted comparability fingerprint did not match its canonical payload.
    #[error("benchmark comparability fingerprint does not match its canonical payload")]
    ComparabilityFingerprintMismatch,
    /// A metric result referenced a threshold for another metric.
    #[error("benchmark metric result threshold does not match {metric:?}")]
    MetricThresholdMismatch {
        /// Metric whose threshold identity was inconsistent.
        metric: MetricKind,
    },
    /// A persisted observed metric was NaN or infinite.
    #[error("benchmark observed value for {metric:?} must be finite")]
    NonFiniteObserved {
        /// Metric whose observed value was invalid.
        metric: MetricKind,
    },
    /// Persisted sample counts or observed-value presence were inconsistent.
    #[error("benchmark sample counts for {metric:?} are inconsistent")]
    InvalidMetricCounts {
        /// Metric whose counts were inconsistent.
        metric: MetricKind,
    },
    /// One metric result was repeated.
    #[error("benchmark evidence repeats metric result {metric:?}")]
    DuplicateMetricResult {
        /// Repeated metric result.
        metric: MetricKind,
    },
    /// Metric results did not exactly cover the suite thresholds.
    #[error("benchmark metric result membership does not match the suite")]
    MetricMembershipMismatch,
    /// A metric result's required count did not match run membership.
    #[error("benchmark required sample count for {metric:?} does not match run membership")]
    RequiredSampleCountMismatch {
        /// Metric whose required count was inconsistent.
        metric: MetricKind,
    },
    /// A persisted evidence digest did not match its canonical payload.
    #[error("benchmark evidence digest does not match its canonical payload")]
    EvidenceDigestMismatch,
    /// Canonical evidence could not be serialized.
    #[error("benchmark evidence could not be canonicalized")]
    CanonicalizationFailed,
}

/// Exact comparison scope shared by future baseline and candidate decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BenchmarkComparability {
    evaluator_key: NonEmptyString,
    evaluator_version: NonEmptyString,
    fingerprint: String,
}

impl BenchmarkComparability {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_persisted(
        project_id: ProjectId,
        context_id: ContextId,
        suite_id: BenchmarkSuiteId,
        dataset_ids: &[BenchmarkDatasetId],
        evaluator_key: impl Into<String>,
        evaluator_version: impl Into<String>,
        model_version: &str,
        temperature: f32,
        fingerprint: impl Into<String>,
    ) -> Result<Self, BenchmarkEvidenceError> {
        let fingerprint = fingerprint.into();
        let comparability = Self::canonical(
            project_id,
            context_id,
            suite_id,
            dataset_ids,
            evaluator_key,
            evaluator_version,
            model_version,
            temperature,
        )?;
        if fingerprint != comparability.fingerprint {
            return Err(BenchmarkEvidenceError::ComparabilityFingerprintMismatch);
        }
        Ok(comparability)
    }

    #[allow(clippy::too_many_arguments)]
    fn canonical(
        project_id: ProjectId,
        context_id: ContextId,
        suite_id: BenchmarkSuiteId,
        dataset_ids: &[BenchmarkDatasetId],
        evaluator_key: impl Into<String>,
        evaluator_version: impl Into<String>,
        model_version: &str,
        temperature: f32,
    ) -> Result<Self, BenchmarkEvidenceError> {
        if dataset_ids.is_empty() {
            return Err(BenchmarkEvidenceError::EmptyDatasets);
        }
        let mut dataset_ids = dataset_ids.to_vec();
        dataset_ids.sort_unstable();
        if let Some(duplicate) = dataset_ids.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(BenchmarkEvidenceError::DuplicateDataset {
                dataset_id: duplicate[0],
            });
        }
        let evaluator_key = NonEmptyString::new("benchmark evaluator key", evaluator_key)
            .map_err(|_| BenchmarkEvidenceError::InvalidEvaluatorIdentity)?;
        let evaluator_version =
            NonEmptyString::new("benchmark evaluator version", evaluator_version)
                .map_err(|_| BenchmarkEvidenceError::InvalidEvaluatorIdentity)?;
        let comparability_payload = (
            project_id,
            context_id,
            suite_id,
            &dataset_ids,
            evaluator_key.as_str(),
            evaluator_version.as_str(),
            model_version,
            temperature.to_bits(),
        );
        let fingerprint = canonical_digest(&comparability_payload)?;

        Ok(Self {
            evaluator_key,
            evaluator_version,
            fingerprint,
        })
    }

    /// Returns the evaluator key.
    #[must_use]
    pub fn evaluator_key(&self) -> &str {
        self.evaluator_key.as_str()
    }

    /// Returns the evaluator version.
    #[must_use]
    pub fn evaluator_version(&self) -> &str {
        self.evaluator_version.as_str()
    }

    /// Returns the canonical comparison fingerprint.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
}

/// Per-metric decision evidence persisted without recalculating policy.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkMetricDecisionEvidence {
    metric: MetricKind,
    threshold: RegressionThreshold,
    observed: Option<f64>,
    sample_count: usize,
    required_sample_count: usize,
    has_complete_coverage: bool,
    outcome: RegressionCheckStatus,
}

impl BenchmarkMetricDecisionEvidence {
    pub(crate) fn from_persisted_with_coverage(
        metric: MetricKind,
        threshold: RegressionThreshold,
        observed: Option<f64>,
        sample_count: usize,
        required_sample_count: usize,
        has_complete_coverage: bool,
        outcome: RegressionCheckStatus,
    ) -> Result<Self, BenchmarkEvidenceError> {
        let evidence = Self {
            metric,
            threshold,
            observed,
            sample_count,
            required_sample_count,
            has_complete_coverage,
            outcome,
        };
        evidence.validate()?;
        Ok(evidence)
    }

    fn validate(&self) -> Result<(), BenchmarkEvidenceError> {
        if self.threshold.metric() != self.metric {
            return Err(BenchmarkEvidenceError::MetricThresholdMismatch {
                metric: self.metric,
            });
        }
        if self.observed.is_some_and(|value| !value.is_finite()) {
            return Err(BenchmarkEvidenceError::NonFiniteObserved {
                metric: self.metric,
            });
        }
        if self.observed.is_some() != (self.sample_count > 0)
            || (self.has_complete_coverage && self.sample_count != self.required_sample_count)
        {
            return Err(BenchmarkEvidenceError::InvalidMetricCounts {
                metric: self.metric,
            });
        }
        Ok(())
    }

    /// Returns the metric kind.
    #[must_use]
    pub const fn metric(&self) -> MetricKind {
        self.metric
    }
    /// Returns the threshold.
    #[must_use]
    pub const fn threshold(&self) -> RegressionThreshold {
        self.threshold
    }
    /// Returns the observed value.
    #[must_use]
    pub const fn observed(&self) -> Option<f64> {
        self.observed
    }
    /// Returns contributing run count.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }
    /// Returns required run count.
    #[must_use]
    pub const fn required_sample_count(&self) -> usize {
        self.required_sample_count
    }
    /// Returns whether every contributing run supplied exactly one finite value.
    #[must_use]
    pub const fn has_complete_coverage(&self) -> bool {
        self.has_complete_coverage
    }
    /// Returns metric outcome.
    #[must_use]
    pub const fn outcome(&self) -> RegressionCheckStatus {
        self.outcome
    }
}

/// Immutable decision evidence for one homogeneous run cohort.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BenchmarkDecisionEvidence {
    decision_id: BenchmarkDecisionId,
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    run_ids: Vec<EvaluationRunId>,
    suite_id: BenchmarkSuiteId,
    dataset_ids: Vec<BenchmarkDatasetId>,
    comparability: BenchmarkComparability,
    evidence_digest: String,
    status: RegressionDecisionStatus,
    metric_results: Vec<BenchmarkMetricDecisionEvidence>,
    recorded_at: DateTime<Utc>,
}

impl BenchmarkDecisionEvidence {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_persisted(
        decision_id: BenchmarkDecisionId,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        runs: Vec<EvaluationRun>,
        comparability: BenchmarkComparability,
        evidence_digest: impl Into<String>,
        status: RegressionDecisionStatus,
        metric_results: Vec<BenchmarkMetricDecisionEvidence>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkEvidenceError> {
        Self::validated(
            decision_id,
            project_id,
            context_id,
            context_commit_id,
            datasets,
            suite,
            runs,
            comparability,
            Some(evidence_digest.into()),
            status,
            metric_results,
            recorded_at,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_calculated(
        decision_id: BenchmarkDecisionId,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        runs: Vec<EvaluationRun>,
        comparability: BenchmarkComparability,
        status: RegressionDecisionStatus,
        metric_results: Vec<BenchmarkMetricDecisionEvidence>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkEvidenceError> {
        Self::validated(
            decision_id,
            project_id,
            context_id,
            context_commit_id,
            datasets,
            suite,
            runs,
            comparability,
            None,
            status,
            metric_results,
            recorded_at,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn validated(
        decision_id: BenchmarkDecisionId,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        runs: Vec<EvaluationRun>,
        comparability: BenchmarkComparability,
        persisted_evidence_digest: Option<String>,
        status: RegressionDecisionStatus,
        mut metric_results: Vec<BenchmarkMetricDecisionEvidence>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkEvidenceError> {
        let (datasets, dataset_ids) = validated_dataset_membership(datasets, &suite)?;
        let (runs, persisted_context_id) = validated_run_cohort(runs)?;
        if persisted_context_id != context_id {
            return Err(BenchmarkEvidenceError::RunContextMismatch);
        }

        metric_results.sort_by_key(BenchmarkMetricDecisionEvidence::metric);
        if let Some(duplicate) = metric_results
            .windows(2)
            .find(|pair| pair[0].metric() == pair[1].metric())
        {
            return Err(BenchmarkEvidenceError::DuplicateMetricResult {
                metric: duplicate[0].metric(),
            });
        }
        if metric_results.len() != suite.thresholds().len()
            || metric_results
                .iter()
                .zip(suite.thresholds())
                .any(|(result, threshold)| {
                    result.metric() != threshold.metric() || result.threshold() != *threshold
                })
        {
            return Err(BenchmarkEvidenceError::MetricMembershipMismatch);
        }
        for result in &metric_results {
            result.validate()?;
            if result.required_sample_count() != runs.len() {
                return Err(BenchmarkEvidenceError::RequiredSampleCountMismatch {
                    metric: result.metric(),
                });
            }
        }
        let first_run = &runs[0];
        let comparability = BenchmarkComparability::from_persisted(
            project_id,
            context_id,
            suite.id(),
            &dataset_ids,
            comparability.evaluator_key(),
            comparability.evaluator_version(),
            first_run.model_version(),
            first_run.temperature(),
            comparability.fingerprint(),
        )?;
        let digest_payload = (
            decision_id,
            project_id,
            context_id,
            context_commit_id,
            &datasets,
            &suite,
            &runs,
            &comparability,
            status,
            &metric_results,
        );
        let canonical_evidence_digest = canonical_digest(&digest_payload)?;
        let evidence_digest = match persisted_evidence_digest {
            Some(evidence_digest) if evidence_digest != canonical_evidence_digest => {
                return Err(BenchmarkEvidenceError::EvidenceDigestMismatch);
            }
            Some(evidence_digest) => evidence_digest,
            None => canonical_evidence_digest,
        };

        Ok(Self {
            decision_id,
            project_id,
            context_id,
            context_commit_id,
            run_ids: runs.iter().map(EvaluationRun::id).collect(),
            suite_id: suite.id(),
            dataset_ids,
            comparability,
            evidence_digest,
            status,
            metric_results,
            recorded_at,
        })
    }

    /// Returns decision identity.
    #[must_use]
    pub const fn decision_id(&self) -> BenchmarkDecisionId {
        self.decision_id
    }
    /// Returns owning project.
    /// Returns the owning project identity.
    /// Returns the owning project identity.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }
    /// Returns evaluated Context.
    /// Returns the owning Context identity.
    /// Returns the owning Context identity.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }
    /// Returns exact Context commit.
    /// Returns the exact Context commit identity.
    /// Returns the exact Context commit identity.
    #[must_use]
    pub const fn context_commit_id(&self) -> CommitId {
        self.context_commit_id
    }
    /// Returns exact run membership.
    #[must_use]
    pub fn run_ids(&self) -> &[EvaluationRunId] {
        &self.run_ids
    }
    /// Returns suite identity.
    #[must_use]
    pub const fn suite_id(&self) -> BenchmarkSuiteId {
        self.suite_id
    }
    /// Returns dataset identities.
    #[must_use]
    pub fn dataset_ids(&self) -> &[BenchmarkDatasetId] {
        &self.dataset_ids
    }
    /// Returns comparability evidence.
    #[must_use]
    pub const fn comparability(&self) -> &BenchmarkComparability {
        &self.comparability
    }
    /// Returns canonical aggregate digest.
    #[must_use]
    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }
    /// Returns overall status.
    #[must_use]
    pub const fn status(&self) -> RegressionDecisionStatus {
        self.status
    }
    /// Returns metric evidence.
    #[must_use]
    pub fn metric_results(&self) -> &[BenchmarkMetricDecisionEvidence] {
        &self.metric_results
    }
    /// Returns capture time.
    #[must_use]
    pub const fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }
}

/// Validated aggregate persisted through one atomic adapter boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistBenchmarkEvaluationEvidence {
    datasets: Vec<BenchmarkDataset>,
    suite: BenchmarkSuite,
    runs: Vec<EvaluationRun>,
    evaluation: BenchmarkEvaluation,
    evidence: BenchmarkDecisionEvidence,
    idempotency: Option<BenchmarkExecutionIdempotency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BenchmarkExecutionIdempotency {
    key: IdempotencyKey,
    request_digest: RequestDigest,
}

impl PersistBenchmarkEvaluationEvidence {
    /// Validates one homogeneous cohort and records a domain-evaluated decision artifact.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        decision_id: BenchmarkDecisionId,
        project_id: ProjectId,
        context_commit_id: CommitId,
        datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        runs: Vec<EvaluationRun>,
        evaluation: BenchmarkEvaluation,
        evaluator_key: impl Into<String>,
        evaluator_version: impl Into<String>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkEvidenceError> {
        let (datasets, dataset_ids) = validated_dataset_membership(datasets, &suite)?;
        let (runs, context_id) = validated_run_cohort(runs)?;
        let run_ids = runs.iter().map(EvaluationRun::id).collect::<Vec<_>>();
        if evaluation.suite_id() != suite.id()
            || evaluation.run_ids() != run_ids
            || evaluation.runs() != runs
        {
            return Err(BenchmarkEvidenceError::DomainEvaluationMismatch);
        }
        let model_version = runs[0].model_version();
        let temperature = runs[0].temperature();
        let comparability = BenchmarkComparability::canonical(
            project_id,
            context_id,
            suite.id(),
            &dataset_ids,
            evaluator_key,
            evaluator_version,
            model_version,
            temperature,
        )?;
        let metric_results = evaluation
            .decision()
            .checks()
            .iter()
            .map(|check| {
                BenchmarkMetricDecisionEvidence::from_persisted_with_coverage(
                    check.metric(),
                    check.threshold(),
                    check.observed(),
                    evaluation.scorecard().sample_count(check.metric()),
                    evaluation.scorecard().run_count(),
                    check.has_complete_coverage(),
                    check.status(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let evidence = BenchmarkDecisionEvidence::from_calculated(
            decision_id,
            project_id,
            context_id,
            context_commit_id,
            datasets.clone(),
            suite.clone(),
            runs.clone(),
            comparability,
            evaluation.decision().status(),
            metric_results,
            recorded_at,
        )?;
        Ok(Self {
            datasets,
            suite,
            runs,
            evaluation,
            evidence,
            idempotency: None,
        })
    }

    /// Attaches the idempotency contract for one private benchmark execution.
    #[must_use]
    pub fn with_idempotency(
        mut self,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
    ) -> Self {
        self.idempotency = Some(BenchmarkExecutionIdempotency {
            key: idempotency_key,
            request_digest,
        });
        self
    }

    /// Returns the execution idempotency key, when this command is protected by one.
    #[must_use]
    pub fn idempotency_key(&self) -> Option<&IdempotencyKey> {
        self.idempotency.as_ref().map(|value| &value.key)
    }

    /// Returns the canonical request digest, when this command is protected by one.
    #[must_use]
    pub fn request_digest(&self) -> Option<&RequestDigest> {
        self.idempotency.as_ref().map(|value| &value.request_digest)
    }

    pub(crate) fn datasets(&self) -> &[BenchmarkDataset] {
        &self.datasets
    }
    pub(crate) const fn suite(&self) -> &BenchmarkSuite {
        &self.suite
    }
    pub(crate) fn runs(&self) -> &[EvaluationRun] {
        &self.runs
    }
    pub(crate) const fn evidence(&self) -> &BenchmarkDecisionEvidence {
        &self.evidence
    }
}

fn canonical_digest(value: &impl Serialize) -> Result<String, BenchmarkEvidenceError> {
    let bytes =
        serde_json::to_vec(value).map_err(|_| BenchmarkEvidenceError::CanonicalizationFailed)?;
    let digest = Sha256::digest(bytes);
    Ok(format!("sha256:{digest:x}"))
}

fn validated_dataset_membership(
    mut datasets: Vec<BenchmarkDataset>,
    suite: &BenchmarkSuite,
) -> Result<(Vec<BenchmarkDataset>, Vec<BenchmarkDatasetId>), BenchmarkEvidenceError> {
    if datasets.is_empty() {
        return Err(BenchmarkEvidenceError::EmptyDatasets);
    }
    datasets.sort_by_key(BenchmarkDataset::id);
    if let Some(duplicate) = datasets
        .windows(2)
        .find(|pair| pair[0].id() == pair[1].id())
    {
        return Err(BenchmarkEvidenceError::DuplicateDataset {
            dataset_id: duplicate[0].id(),
        });
    }
    let dataset_ids = datasets
        .iter()
        .map(BenchmarkDataset::id)
        .collect::<Vec<_>>();
    if dataset_ids != suite.dataset_ids() {
        return Err(BenchmarkEvidenceError::DatasetMembershipMismatch);
    }
    Ok((datasets, dataset_ids))
}

fn validated_run_cohort(
    mut runs: Vec<EvaluationRun>,
) -> Result<(Vec<EvaluationRun>, ContextId), BenchmarkEvidenceError> {
    if runs.is_empty() {
        return Err(BenchmarkEvidenceError::EmptyRuns);
    }
    for run in &runs {
        EvaluationRun::from_persisted(
            run.id(),
            run.context_id(),
            run.model_version(),
            run.temperature(),
            run.measurements().to_vec(),
            run.executed_at(),
        )
        .map_err(|_| BenchmarkEvidenceError::InvalidRunConfiguration { run_id: run.id() })?;
    }
    runs.sort_by_key(EvaluationRun::id);
    if let Some(duplicate) = runs.windows(2).find(|pair| pair[0].id() == pair[1].id()) {
        return Err(BenchmarkEvidenceError::DuplicateRun {
            run_id: duplicate[0].id(),
        });
    }
    let context_id = runs[0].context_id();
    if runs.iter().any(|run| run.context_id() != context_id) {
        return Err(BenchmarkEvidenceError::RunContextMismatch);
    }
    let model_version = runs[0].model_version();
    let temperature_bits = runs[0].temperature().to_bits();
    if runs.iter().any(|run| {
        run.model_version() != model_version || run.temperature().to_bits() != temperature_bits
    }) {
        return Err(BenchmarkEvidenceError::RunModelConfigurationMismatch);
    }
    Ok((runs, context_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_evaluation::{
        BenchmarkCase, BenchmarkExpectedOutput, MetricMeasurement, ThresholdDirection,
    };
    use serde_json::json;

    #[test]
    fn rehydrates_canonical_benchmark_decision_without_re_evaluation() {
        let command = evidence_fixture();
        let original = command.evidence().clone();
        let mut datasets = command.datasets().to_vec();
        let mut runs = command.runs().to_vec();
        let mut metric_results = original.metric_results().to_vec();
        datasets.reverse();
        runs.reverse();
        metric_results.reverse();

        let comparability =
            rehydrate_comparability(&command, original.comparability().fingerprint())
                .expect("valid comparability");
        let rehydrated = BenchmarkDecisionEvidence::from_persisted(
            original.decision_id(),
            original.project_id(),
            original.context_id(),
            original.context_commit_id(),
            datasets,
            command.suite().clone(),
            runs,
            comparability,
            original.evidence_digest(),
            original.status(),
            metric_results,
            original.recorded_at(),
        )
        .expect("valid persisted decision");

        assert_eq!(rehydrated, original);
    }

    #[test]
    fn rejects_tampered_comparability_and_evidence_digests() {
        let command = evidence_fixture();
        let original = command.evidence();

        assert_eq!(
            rehydrate_comparability(&command, &format!("sha256:{}", "0".repeat(64))),
            Err(BenchmarkEvidenceError::ComparabilityFingerprintMismatch)
        );

        let comparability =
            rehydrate_comparability(&command, original.comparability().fingerprint())
                .expect("valid comparability");
        assert_eq!(
            BenchmarkDecisionEvidence::from_persisted(
                original.decision_id(),
                original.project_id(),
                original.context_id(),
                original.context_commit_id(),
                command.datasets().to_vec(),
                command.suite().clone(),
                command.runs().to_vec(),
                comparability,
                format!("sha256:{}", "0".repeat(64)),
                original.status(),
                original.metric_results().to_vec(),
                original.recorded_at(),
            ),
            Err(BenchmarkEvidenceError::EvidenceDigestMismatch)
        );
    }

    #[test]
    fn rejects_duplicate_persisted_membership() {
        let command = evidence_fixture();
        let original = command.evidence();
        let dataset = command.datasets()[0].clone();
        let run = command.runs()[0].clone();
        let metric_result = original.metric_results()[0].clone();
        let comparability =
            rehydrate_comparability(&command, original.comparability().fingerprint())
                .expect("valid comparability");

        assert!(matches!(
            BenchmarkDecisionEvidence::from_persisted(
                original.decision_id(),
                original.project_id(),
                original.context_id(),
                original.context_commit_id(),
                vec![dataset.clone(), dataset],
                command.suite().clone(),
                command.runs().to_vec(),
                comparability.clone(),
                original.evidence_digest(),
                original.status(),
                original.metric_results().to_vec(),
                original.recorded_at(),
            ),
            Err(BenchmarkEvidenceError::DuplicateDataset { .. })
        ));
        assert!(matches!(
            BenchmarkDecisionEvidence::from_persisted(
                original.decision_id(),
                original.project_id(),
                original.context_id(),
                original.context_commit_id(),
                command.datasets().to_vec(),
                command.suite().clone(),
                vec![run.clone(), run],
                comparability.clone(),
                original.evidence_digest(),
                original.status(),
                original.metric_results().to_vec(),
                original.recorded_at(),
            ),
            Err(BenchmarkEvidenceError::DuplicateRun { .. })
        ));
        assert!(matches!(
            BenchmarkDecisionEvidence::from_persisted(
                original.decision_id(),
                original.project_id(),
                original.context_id(),
                original.context_commit_id(),
                command.datasets().to_vec(),
                command.suite().clone(),
                command.runs().to_vec(),
                comparability,
                original.evidence_digest(),
                original.status(),
                vec![metric_result.clone(), metric_result],
                original.recorded_at(),
            ),
            Err(BenchmarkEvidenceError::DuplicateMetricResult { .. })
        ));
    }

    #[test]
    fn rejects_invalid_metric_counts_outcome_and_observed_value() {
        let threshold =
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold");

        assert_eq!(
            BenchmarkMetricDecisionEvidence::from_persisted_with_coverage(
                MetricKind::Accuracy,
                threshold,
                Some(f64::NAN),
                1,
                1,
                true,
                RegressionCheckStatus::Passed,
            ),
            Err(BenchmarkEvidenceError::NonFiniteObserved {
                metric: MetricKind::Accuracy
            })
        );
        assert_eq!(
            BenchmarkMetricDecisionEvidence::from_persisted_with_coverage(
                MetricKind::Accuracy,
                threshold,
                Some(0.95),
                2,
                1,
                true,
                RegressionCheckStatus::Passed,
            ),
            Err(BenchmarkEvidenceError::InvalidMetricCounts {
                metric: MetricKind::Accuracy
            })
        );
    }

    #[test]
    fn rehydration_preserves_the_domain_recorded_metric_outcome() {
        let threshold =
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold");

        let evidence = BenchmarkMetricDecisionEvidence::from_persisted_with_coverage(
            MetricKind::Accuracy,
            threshold,
            Some(0.8),
            1,
            1,
            true,
            RegressionCheckStatus::Passed,
        )
        .expect("storage should preserve domain-recorded policy output");

        assert_eq!(evidence.outcome(), RegressionCheckStatus::Passed);
    }

    #[test]
    fn rejects_metric_coverage_mismatches() {
        let command = evidence_fixture();
        let original = command.evidence();
        let comparability =
            rehydrate_comparability(&command, original.comparability().fingerprint())
                .expect("valid comparability");
        let mut wrong_count = original.metric_results().to_vec();
        wrong_count[0] = BenchmarkMetricDecisionEvidence::from_persisted_with_coverage(
            wrong_count[0].metric(),
            wrong_count[0].threshold(),
            wrong_count[0].observed(),
            1,
            1,
            true,
            wrong_count[0].outcome(),
        )
        .expect("internally valid metric evidence");

        assert!(matches!(
            BenchmarkDecisionEvidence::from_persisted(
                original.decision_id(),
                original.project_id(),
                original.context_id(),
                original.context_commit_id(),
                command.datasets().to_vec(),
                command.suite().clone(),
                command.runs().to_vec(),
                comparability,
                original.evidence_digest(),
                original.status(),
                wrong_count,
                original.recorded_at(),
            ),
            Err(BenchmarkEvidenceError::RequiredSampleCountMismatch { .. })
        ));
    }

    fn rehydrate_comparability(
        command: &PersistBenchmarkEvaluationEvidence,
        fingerprint: &str,
    ) -> Result<BenchmarkComparability, BenchmarkEvidenceError> {
        let evidence = command.evidence();
        let run = &command.runs()[0];
        BenchmarkComparability::from_persisted(
            evidence.project_id(),
            evidence.context_id(),
            command.suite().id(),
            command.suite().dataset_ids(),
            evidence.comparability().evaluator_key(),
            evidence.comparability().evaluator_version(),
            run.model_version(),
            run.temperature(),
            fingerprint,
        )
    }

    fn evidence_fixture() -> PersistBenchmarkEvaluationEvidence {
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let datasets = ["Dataset B", "Dataset A"]
            .into_iter()
            .map(|name| {
                BenchmarkDataset::new(
                    name,
                    vec![
                        BenchmarkCase::new(
                            format!("{name} case"),
                            json!({"input": name}),
                            BenchmarkExpectedOutput::Exact(json!({"accepted": true})),
                        )
                        .expect("case"),
                    ],
                )
                .expect("dataset")
            })
            .collect::<Vec<_>>();
        let suite = BenchmarkSuite::new(
            "Release gate",
            datasets.iter().map(BenchmarkDataset::id).collect(),
            vec![
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
            ],
        )
        .expect("suite");
        let runs = [(0.95, 700.0), (0.96, 710.0)]
            .into_iter()
            .map(|(accuracy, latency)| {
                EvaluationRun::new(
                    context_id,
                    "model-a",
                    0.2,
                    vec![
                        MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("accuracy"),
                        MetricMeasurement::new(MetricKind::LatencyMs, latency).expect("latency"),
                    ],
                    Utc::now(),
                )
                .expect("valid evaluation run")
            })
            .collect::<Vec<_>>();
        let evaluation = BenchmarkEvaluation::from_runs(&suite, &runs);

        PersistBenchmarkEvaluationEvidence::new(
            BenchmarkDecisionId::new(),
            project_id,
            CommitId::new(),
            datasets,
            suite,
            runs,
            evaluation,
            "contextlab.exact-match",
            "v1",
            Utc::now(),
        )
        .expect("benchmark evidence")
    }
}

/// Whether an evidence write created or replayed prior state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkEvidenceWriteDisposition {
    /// New immutable evidence was created.
    Created,
    /// The same canonical aggregate replayed the first evidence.
    Replayed,
}

/// Durable receipt that binds one execution idempotency key to its decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkExecutionIdempotencyReceipt {
    decision_id: BenchmarkDecisionId,
    request_digest: RequestDigest,
}

impl BenchmarkExecutionIdempotencyReceipt {
    pub(crate) const fn new(
        decision_id: BenchmarkDecisionId,
        request_digest: RequestDigest,
    ) -> Self {
        Self {
            decision_id,
            request_digest,
        }
    }

    /// Returns the decision persisted by the first request using this key.
    #[must_use]
    pub const fn decision_id(&self) -> BenchmarkDecisionId {
        self.decision_id
    }

    /// Returns the canonical request digest bound to this key.
    #[must_use]
    pub const fn request_digest(&self) -> &RequestDigest {
        &self.request_digest
    }
}

/// Result of an atomic benchmark evidence write.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkEvidenceWriteResult {
    disposition: BenchmarkEvidenceWriteDisposition,
    evidence: BenchmarkDecisionEvidence,
}

impl BenchmarkEvidenceWriteResult {
    pub(crate) const fn new(
        disposition: BenchmarkEvidenceWriteDisposition,
        evidence: BenchmarkDecisionEvidence,
    ) -> Self {
        Self {
            disposition,
            evidence,
        }
    }
    /// Returns write disposition.
    #[must_use]
    pub const fn disposition(&self) -> BenchmarkEvidenceWriteDisposition {
        self.disposition
    }
    /// Returns immutable evidence.
    #[must_use]
    pub const fn evidence(&self) -> &BenchmarkDecisionEvidence {
        &self.evidence
    }
}

/// Private atomic writer for definitions, runs, membership, and decision evidence.
#[async_trait]
pub trait BenchmarkEvidenceWriter: Send + Sync {
    /// Persists or replays one complete aggregate.
    async fn persist_benchmark_evaluation(
        &self,
        command: PersistBenchmarkEvaluationEvidence,
    ) -> Result<BenchmarkEvidenceWriteResult, StorageRepositoryError>;
}

/// Private read contract for immutable benchmark persistence.
#[async_trait]
pub trait BenchmarkEvidenceRepository: Send + Sync {
    /// Loads the durable receipt for one private benchmark execution key.
    ///
    /// Repositories that do not implement execution idempotency fail closed; existing
    /// non-idempotent callers never invoke this port.
    async fn get_benchmark_execution_idempotency(
        &self,
        _project_id: ProjectId,
        _context_id: ContextId,
        _context_commit_id: CommitId,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<Option<BenchmarkExecutionIdempotencyReceipt>, StorageRepositoryError> {
        Err(StorageRepositoryError::Database {
            message: "benchmark execution idempotency receipt store is unavailable".to_owned(),
        })
    }

    /// Returns one project-scoped immutable dataset definition.
    async fn get_benchmark_dataset(
        &self,
        project_id: ProjectId,
        dataset_id: BenchmarkDatasetId,
    ) -> Result<Option<BenchmarkDataset>, StorageRepositoryError>;
    /// Returns one project-scoped immutable suite definition.
    async fn get_benchmark_suite(
        &self,
        project_id: ProjectId,
        suite_id: BenchmarkSuiteId,
    ) -> Result<Option<BenchmarkSuite>, StorageRepositoryError>;
    /// Returns one private run bound to an exact Context commit.
    async fn get_benchmark_run(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        run_id: EvaluationRunId,
    ) -> Result<Option<EvaluationRun>, StorageRepositoryError>;
    /// Returns one project-, Context-, and commit-scoped decision aggregate.
    async fn get_benchmark_decision(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
    ) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError>;
    /// Returns two exact decision aggregates from one consistent repository snapshot.
    async fn get_benchmark_decision_pair(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        baseline: BenchmarkDecisionComparisonScope,
        revised: BenchmarkDecisionComparisonScope,
    ) -> Result<BenchmarkDecisionPair, StorageRepositoryError>;
}

/// Private read port for discovering sealed benchmark decisions at one exact scope.
///
/// This is intentionally separate from [`BenchmarkEvidenceRepository`]: callers that only
/// discover redacted decision summaries must not make every evidence adapter implement the list
/// contract.
#[async_trait]
pub trait BenchmarkDecisionDiscoveryRepository: Send + Sync {
    /// Lists only sealed decision aggregates for one exact project, Context, and commit.
    async fn list_benchmark_decisions(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Vec<BenchmarkDecisionDiscoverySummary>, StorageRepositoryError>;
}

/// Errors returned while loading and comparing two sealed decision artifacts.
#[derive(Debug, Error)]
pub enum BenchmarkDecisionComparisonServiceError {
    /// Storage could not load both scopes from one consistent snapshot.
    #[error("benchmark decision comparison storage access failed")]
    Storage(#[from] StorageRepositoryError),
    /// The sealed evidence cannot safely be compared.
    #[error("benchmark decision comparison is not valid")]
    Comparison(#[from] BenchmarkDecisionComparisonError),
}

/// Private service that compares two exact immutable decision scopes.
#[derive(Debug)]
pub struct BenchmarkDecisionComparisonService<'a, Repository: ?Sized> {
    repository: &'a Repository,
}

impl<'a, Repository: ?Sized> BenchmarkDecisionComparisonService<'a, Repository> {
    /// Binds the service to one repository that supports atomic pair reads.
    #[must_use]
    pub const fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }
}

impl<Repository: ?Sized> BenchmarkDecisionComparisonService<'_, Repository>
where
    Repository: BenchmarkEvidenceRepository,
{
    /// Compares two exact sealed decisions without re-evaluating benchmark policy.
    pub async fn compare(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        baseline: BenchmarkDecisionComparisonScope,
        revised: BenchmarkDecisionComparisonScope,
    ) -> Result<Option<BenchmarkDecisionDiff>, BenchmarkDecisionComparisonServiceError> {
        let decisions = self
            .repository
            .get_benchmark_decision_pair(project_id, context_id, baseline, revised)
            .await?;
        let (Some(baseline), Some(revised)) = (decisions.baseline(), decisions.revised()) else {
            return Ok(None);
        };

        Ok(Some(BenchmarkDecisionDiff::between(
            comparison_input(baseline)?,
            comparison_input(revised)?,
        )?))
    }
}

/// Safe sealed suite metadata for one benchmark decision inspection.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDecisionSuiteSummary {
    id: BenchmarkSuiteId,
    name: String,
    thresholds: Vec<RegressionThreshold>,
}

impl BenchmarkDecisionSuiteSummary {
    /// Returns the sealed suite identity.
    #[must_use]
    pub const fn id(&self) -> BenchmarkSuiteId {
        self.id
    }

    /// Returns the sealed suite name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns thresholds in the suite's stable metric order.
    #[must_use]
    pub fn thresholds(&self) -> &[RegressionThreshold] {
        &self.thresholds
    }
}

/// Safe sealed dataset metadata for one benchmark decision inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDecisionDatasetSummary {
    id: BenchmarkDatasetId,
    name: String,
    case_count: usize,
}

impl BenchmarkDecisionDatasetSummary {
    pub(crate) fn new(id: BenchmarkDatasetId, name: String, case_count: usize) -> Self {
        Self {
            id,
            name,
            case_count,
        }
    }

    /// Returns the sealed dataset identity.
    #[must_use]
    pub const fn id(&self) -> BenchmarkDatasetId {
        self.id
    }

    /// Returns the sealed dataset name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the count of sealed cases without returning case payloads.
    #[must_use]
    pub const fn case_count(&self) -> usize {
        self.case_count
    }
}

/// Safe suite and dataset metadata bound to one exact sealed decision.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDecisionDefinitionSummary {
    suite: BenchmarkDecisionSuiteSummary,
    datasets: Vec<BenchmarkDecisionDatasetSummary>,
}

impl BenchmarkDecisionDefinitionSummary {
    /// Returns safe sealed suite metadata.
    #[must_use]
    pub const fn suite(&self) -> &BenchmarkDecisionSuiteSummary {
        &self.suite
    }

    /// Returns safe sealed dataset metadata in stable identifier order.
    #[must_use]
    pub fn datasets(&self) -> &[BenchmarkDecisionDatasetSummary] {
        &self.datasets
    }
}

/// Errors while resolving safe metadata for one sealed decision.
#[derive(Debug, Error)]
pub enum BenchmarkDecisionDefinitionSummaryError {
    /// Immutable definition storage was unavailable.
    #[error("benchmark definition summary storage access failed")]
    Storage(#[from] StorageRepositoryError),
    /// A sealed decision references unavailable definition metadata.
    #[error("benchmark definition summary is unavailable")]
    DefinitionUnavailable,
    /// Loaded definitions do not match the sealed decision membership.
    #[error("benchmark definition summary does not match sealed decision membership")]
    DefinitionMembershipMismatch,
}

/// Private projection service for safe metadata behind one exact sealed decision.
#[derive(Debug)]
pub struct BenchmarkDecisionDefinitionSummaryService<'a, Repository: ?Sized> {
    repository: &'a Repository,
}

impl<'a, Repository: ?Sized> BenchmarkDecisionDefinitionSummaryService<'a, Repository> {
    /// Binds the projection to one immutable evidence repository.
    #[must_use]
    pub const fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }
}

impl<Repository: ?Sized> BenchmarkDecisionDefinitionSummaryService<'_, Repository>
where
    Repository: BenchmarkEvidenceRepository,
{
    /// Resolves safe metadata for one already-loaded sealed decision without exposing cases.
    pub async fn summarize(
        &self,
        decision: &BenchmarkDecisionEvidence,
    ) -> Result<BenchmarkDecisionDefinitionSummary, BenchmarkDecisionDefinitionSummaryError> {
        let project_id = decision.project_id();
        let suite = self
            .repository
            .get_benchmark_suite(project_id, decision.suite_id())
            .await?
            .ok_or(BenchmarkDecisionDefinitionSummaryError::DefinitionUnavailable)?;
        if suite.id() != decision.suite_id() || suite.dataset_ids() != decision.dataset_ids() {
            return Err(BenchmarkDecisionDefinitionSummaryError::DefinitionMembershipMismatch);
        }

        let mut datasets = Vec::with_capacity(decision.dataset_ids().len());
        for dataset_id in decision.dataset_ids() {
            let dataset = self
                .repository
                .get_benchmark_dataset(project_id, *dataset_id)
                .await?
                .ok_or(BenchmarkDecisionDefinitionSummaryError::DefinitionUnavailable)?;
            if dataset.id() != *dataset_id {
                return Err(BenchmarkDecisionDefinitionSummaryError::DefinitionMembershipMismatch);
            }
            datasets.push(BenchmarkDecisionDatasetSummary {
                id: dataset.id(),
                name: dataset.name().to_owned(),
                case_count: dataset.cases().len(),
            });
        }
        datasets.sort_by_key(BenchmarkDecisionDatasetSummary::id);

        Ok(BenchmarkDecisionDefinitionSummary {
            suite: BenchmarkDecisionSuiteSummary {
                id: suite.id(),
                name: suite.name().to_owned(),
                thresholds: suite.thresholds().to_vec(),
            },
            datasets,
        })
    }
}

/// Safe suite identity and name for one sealed decision discovery row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDecisionDiscoverySuiteSummary {
    id: BenchmarkSuiteId,
    name: String,
}

impl BenchmarkDecisionDiscoverySuiteSummary {
    pub(crate) fn new(id: BenchmarkSuiteId, name: String) -> Self {
        Self { id, name }
    }

    /// Returns the stable suite identity.
    #[must_use]
    pub const fn id(&self) -> BenchmarkSuiteId {
        self.id
    }

    /// Returns the suite display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Redacted deterministic metadata for one sealed benchmark decision.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDecisionDiscoverySummary {
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    decision_id: BenchmarkDecisionId,
    suite: BenchmarkDecisionDiscoverySuiteSummary,
    datasets: Vec<BenchmarkDecisionDatasetSummary>,
    status: RegressionDecisionStatus,
    recorded_at: DateTime<Utc>,
    run_count: usize,
}

/// Storage-owned fields used to construct one redacted discovery summary.
pub(crate) struct BenchmarkDecisionDiscoverySummaryInput {
    pub(crate) project_id: ProjectId,
    pub(crate) context_id: ContextId,
    pub(crate) context_commit_id: CommitId,
    pub(crate) decision_id: BenchmarkDecisionId,
    pub(crate) suite: BenchmarkDecisionDiscoverySuiteSummary,
    pub(crate) datasets: Vec<BenchmarkDecisionDatasetSummary>,
    pub(crate) status: RegressionDecisionStatus,
    pub(crate) recorded_at: DateTime<Utc>,
    pub(crate) run_count: usize,
}

impl BenchmarkDecisionDiscoverySummary {
    pub(crate) fn new(input: BenchmarkDecisionDiscoverySummaryInput) -> Self {
        Self {
            project_id: input.project_id,
            context_id: input.context_id,
            context_commit_id: input.context_commit_id,
            decision_id: input.decision_id,
            suite: input.suite,
            datasets: input.datasets,
            status: input.status,
            recorded_at: input.recorded_at,
            run_count: input.run_count,
        }
    }

    pub(crate) fn push_dataset(&mut self, dataset: BenchmarkDecisionDatasetSummary) {
        self.datasets.push(dataset);
    }

    pub(crate) fn sort_datasets(&mut self) {
        self.datasets
            .sort_by_key(BenchmarkDecisionDatasetSummary::id);
    }

    /// Returns the owning project identity.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the owning Context identity.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the exact Context commit identity.
    #[must_use]
    pub const fn context_commit_id(&self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the stable decision identity.
    #[must_use]
    pub const fn decision_id(&self) -> BenchmarkDecisionId {
        self.decision_id
    }

    /// Returns safe suite identity and name.
    #[must_use]
    pub const fn suite(&self) -> &BenchmarkDecisionDiscoverySuiteSummary {
        &self.suite
    }

    /// Returns safe dataset metadata in stable identifier order.
    #[must_use]
    pub fn datasets(&self) -> &[BenchmarkDecisionDatasetSummary] {
        &self.datasets
    }

    /// Returns the sealed decision outcome recorded by evaluation.
    #[must_use]
    pub const fn status(&self) -> RegressionDecisionStatus {
        self.status
    }

    /// Returns the immutable decision recording time.
    #[must_use]
    pub const fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }

    /// Returns the number of sealed evaluation runs in the decision.
    #[must_use]
    pub const fn run_count(&self) -> usize {
        self.run_count
    }
}

/// Errors while discovering sealed decisions at one exact Context commit.
#[derive(Debug, Error)]
pub enum BenchmarkDecisionDiscoveryServiceError {
    /// Immutable decision storage was unavailable or outside the caller's scope.
    #[error("benchmark decision discovery storage access failed")]
    Storage(#[from] StorageRepositoryError),
    /// A sealed decision's immutable definitions could not be projected safely.
    #[error("benchmark decision discovery definition projection failed")]
    Definition(#[from] BenchmarkDecisionDefinitionSummaryError),
    /// The repository returned a decision outside the requested exact scope.
    #[error("benchmark decision discovery returned an out-of-scope decision")]
    ScopeMismatch,
}

/// Private application projection for the exact sealed-decision discovery path.
#[derive(Debug)]
pub struct BenchmarkDecisionDiscoveryService<'a, Repository: ?Sized> {
    repository: &'a Repository,
}

impl<'a, Repository: ?Sized> BenchmarkDecisionDiscoveryService<'a, Repository> {
    /// Binds discovery to its safe summary repository port.
    #[must_use]
    pub const fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }
}

impl<Repository: ?Sized> BenchmarkDecisionDiscoveryService<'_, Repository>
where
    Repository: BenchmarkDecisionDiscoveryRepository,
{
    /// Lists only sealed decisions for the exact project, Context, and commit.
    pub async fn summarize(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Vec<BenchmarkDecisionDiscoverySummary>, BenchmarkDecisionDiscoveryServiceError>
    {
        let mut decisions = self
            .repository
            .list_benchmark_decisions(project_id, context_id, context_commit_id)
            .await?;

        for decision in &decisions {
            if decision.project_id() != project_id
                || decision.context_id() != context_id
                || decision.context_commit_id() != context_commit_id
            {
                return Err(BenchmarkDecisionDiscoveryServiceError::ScopeMismatch);
            }
        }

        decisions.sort_by(|left, right| {
            right
                .recorded_at()
                .cmp(&left.recorded_at())
                .then_with(|| left.decision_id().cmp(&right.decision_id()))
        });
        Ok(decisions)
    }
}

/// Safe numeric measurement attached to one sealed benchmark-decision run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchmarkDecisionRunMeasurementSummary {
    metric: MetricKind,
    value: f64,
}

impl BenchmarkDecisionRunMeasurementSummary {
    /// Returns the measured metric kind.
    #[must_use]
    pub const fn metric(&self) -> MetricKind {
        self.metric
    }

    /// Returns the finite recorded numeric value.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }
}

/// Safe immutable run facts resolved from one sealed benchmark decision.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDecisionRunSummary {
    id: EvaluationRunId,
    model_version: String,
    temperature: f32,
    measurements: Vec<BenchmarkDecisionRunMeasurementSummary>,
    executed_at: DateTime<Utc>,
}

impl BenchmarkDecisionRunSummary {
    /// Returns the sealed run identity.
    #[must_use]
    pub const fn id(&self) -> EvaluationRunId {
        self.id
    }

    /// Returns the recorded model version.
    #[must_use]
    pub fn model_version(&self) -> &str {
        &self.model_version
    }

    /// Returns the recorded sampling temperature.
    #[must_use]
    pub const fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns finite numeric measurements in their persisted order.
    #[must_use]
    pub fn measurements(&self) -> &[BenchmarkDecisionRunMeasurementSummary] {
        &self.measurements
    }

    /// Returns the recorded execution timestamp.
    #[must_use]
    pub const fn executed_at(&self) -> DateTime<Utc> {
        self.executed_at
    }
}

/// Ordered safe run cohort behind one exact sealed benchmark decision.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDecisionRunDetails {
    runs: Vec<BenchmarkDecisionRunSummary>,
}

impl BenchmarkDecisionRunDetails {
    /// Returns sealed runs in the decision's immutable membership order.
    #[must_use]
    pub fn runs(&self) -> &[BenchmarkDecisionRunSummary] {
        &self.runs
    }
}

/// Errors while resolving safe run details for one sealed decision.
#[derive(Debug, Error)]
pub enum BenchmarkDecisionRunDetailsError {
    /// Immutable run storage was unavailable.
    #[error("benchmark decision run details storage access failed")]
    Storage(#[from] StorageRepositoryError),
    /// A sealed run can no longer be resolved at its exact scope.
    #[error("benchmark decision run details are unavailable")]
    RunUnavailable,
    /// A resolved run does not match the sealed member or Context scope.
    #[error("benchmark decision run details do not match sealed membership")]
    RunMembershipMismatch,
}

/// Private projection service for safe ordered runs behind one sealed decision.
#[derive(Debug)]
pub struct BenchmarkDecisionRunDetailsService<'a, Repository: ?Sized> {
    repository: &'a Repository,
}

impl<'a, Repository: ?Sized> BenchmarkDecisionRunDetailsService<'a, Repository> {
    /// Binds the projection to one immutable evidence repository.
    #[must_use]
    pub const fn new(repository: &'a Repository) -> Self {
        Self { repository }
    }
}

impl<Repository: ?Sized> BenchmarkDecisionRunDetailsService<'_, Repository>
where
    Repository: BenchmarkEvidenceRepository,
{
    /// Resolves only the already-sealed run membership without re-evaluation.
    pub async fn summarize(
        &self,
        decision: &BenchmarkDecisionEvidence,
    ) -> Result<BenchmarkDecisionRunDetails, BenchmarkDecisionRunDetailsError> {
        let mut runs = Vec::with_capacity(decision.run_ids().len());
        for run_id in decision.run_ids() {
            let run = self
                .repository
                .get_benchmark_run(
                    decision.project_id(),
                    decision.context_id(),
                    decision.context_commit_id(),
                    *run_id,
                )
                .await?
                .ok_or(BenchmarkDecisionRunDetailsError::RunUnavailable)?;
            if run.id() != *run_id || run.context_id() != decision.context_id() {
                return Err(BenchmarkDecisionRunDetailsError::RunMembershipMismatch);
            }
            runs.push(BenchmarkDecisionRunSummary {
                id: run.id(),
                model_version: run.model_version().to_owned(),
                temperature: run.temperature(),
                measurements: run
                    .measurements()
                    .iter()
                    .map(|measurement| BenchmarkDecisionRunMeasurementSummary {
                        metric: measurement.kind(),
                        value: measurement.value(),
                    })
                    .collect(),
                executed_at: run.executed_at(),
            });
        }
        Ok(BenchmarkDecisionRunDetails { runs })
    }
}

fn comparison_input(
    evidence: &BenchmarkDecisionEvidence,
) -> Result<BenchmarkDecisionComparisonInput, BenchmarkDecisionComparisonError> {
    let metrics = evidence
        .metric_results()
        .iter()
        .map(|metric| {
            BenchmarkDecisionMetricInput::new(
                metric.threshold(),
                metric.observed(),
                metric.sample_count(),
                metric.required_sample_count(),
                metric.has_complete_coverage(),
                metric.outcome(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    BenchmarkDecisionComparisonInput::new(
        evidence.status(),
        evidence.comparability().fingerprint(),
        metrics,
    )
}
