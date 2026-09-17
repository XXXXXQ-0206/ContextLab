//! Provider-free, versioned benchmark execution receipts.

use crate::{
    BenchmarkCaseExecutionResult, BenchmarkDecisionComparisonError,
    BenchmarkDecisionComparisonInput, BenchmarkDecisionMetricInput, BenchmarkEvaluation,
    BenchmarkExecutionCaseKey, BenchmarkExecutionCohort, BenchmarkExecutionError,
    BenchmarkExecutionPlan,
};
use chrono::{DateTime, Utc};
use contextlab_context_core::ContextId;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

const RECEIPT_SCHEMA_VERSION: u16 = 1;

/// Stable version for the provider-free benchmark execution receipt contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionReceiptVersion {
    /// The initial execution receipt contract.
    V1,
}

impl BenchmarkExecutionReceiptVersion {
    /// Returns the current receipt contract version.
    #[must_use]
    pub const fn current() -> Self {
        Self::V1
    }

    /// Returns the numeric schema version used by persistence-facing callers.
    #[must_use]
    pub const fn schema_version(self) -> u16 {
        match self {
            Self::V1 => RECEIPT_SCHEMA_VERSION,
        }
    }
}

/// Stable UUID for one deterministic benchmark execution cohort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BenchmarkExecutionCohortId(Uuid);

impl BenchmarkExecutionCohortId {
    /// Wraps a UUID as a cohort identity.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the wrapped UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BenchmarkWorkspaceDefinitionFacts {
    suite_id: crate::BenchmarkSuiteId,
    suite_name: String,
    thresholds: Vec<crate::RegressionThreshold>,
    datasets: Vec<BenchmarkWorkspaceDatasetFacts>,
}

impl BenchmarkWorkspaceDefinitionFacts {
    pub(crate) fn from_plan(plan: &BenchmarkExecutionPlan) -> Self {
        Self {
            suite_id: plan.suite().id(),
            suite_name: plan.suite().name().to_owned(),
            thresholds: plan.suite().thresholds().to_vec(),
            datasets: plan
                .datasets()
                .iter()
                .map(|dataset| BenchmarkWorkspaceDatasetFacts {
                    id: dataset.id(),
                    name: dataset.name().to_owned(),
                    case_ids: dataset
                        .cases()
                        .iter()
                        .map(crate::BenchmarkCase::id)
                        .collect(),
                })
                .collect(),
        }
    }

    pub(crate) const fn suite_id(&self) -> crate::BenchmarkSuiteId {
        self.suite_id
    }

    pub(crate) fn suite_name(&self) -> &str {
        &self.suite_name
    }

    pub(crate) fn datasets(&self) -> &[BenchmarkWorkspaceDatasetFacts] {
        &self.datasets
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BenchmarkWorkspaceDatasetFacts {
    id: crate::BenchmarkDatasetId,
    name: String,
    case_ids: Vec<crate::BenchmarkCaseId>,
}

impl BenchmarkWorkspaceDatasetFacts {
    pub(crate) const fn id(&self) -> crate::BenchmarkDatasetId {
        self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn case_count(&self) -> usize {
        self.case_ids.len()
    }
}

/// Immutable result of composing a plan, cohort, evaluation, and diff input.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionReceipt {
    version: BenchmarkExecutionReceiptVersion,
    cohort_id: BenchmarkExecutionCohortId,
    decision_namespace: Uuid,
    suite_id: crate::BenchmarkSuiteId,
    workspace_definition: BenchmarkWorkspaceDefinitionFacts,
    cohort: BenchmarkExecutionCohort,
    evaluation: BenchmarkEvaluation,
    decision_input: BenchmarkDecisionComparisonInput,
}

impl BenchmarkExecutionReceipt {
    /// Builds a deterministic receipt without calling a provider or adapter.
    #[allow(clippy::too_many_arguments)]
    pub fn from_plan(
        plan: &BenchmarkExecutionPlan,
        decision_namespace: Uuid,
        context_id: ContextId,
        model_version: &str,
        temperature: f32,
        executed_at: DateTime<Utc>,
        comparability_fingerprint: impl Into<String>,
        results: Vec<BenchmarkCaseExecutionResult>,
    ) -> Result<Self, BenchmarkExecutionReceiptError> {
        let cohort = plan.assemble_cohort(
            decision_namespace,
            context_id,
            model_version,
            temperature,
            executed_at,
            results,
        )?;
        let evaluation = plan.suite().evaluate_runs(&cohort.runs());
        let decision_input = decision_input(&evaluation, comparability_fingerprint)?;
        let cohort_id = BenchmarkExecutionCohortId::from_uuid(Uuid::new_v5(
            &decision_namespace,
            &cohort_name(plan, context_id, model_version, temperature, executed_at),
        ));

        Ok(Self {
            version: BenchmarkExecutionReceiptVersion::current(),
            cohort_id,
            decision_namespace,
            suite_id: plan.suite().id(),
            workspace_definition: BenchmarkWorkspaceDefinitionFacts::from_plan(plan),
            cohort,
            evaluation,
            decision_input,
        })
    }

    /// Returns the versioned receipt contract.
    #[must_use]
    pub const fn version(&self) -> BenchmarkExecutionReceiptVersion {
        self.version
    }

    /// Returns the numeric receipt schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.version.schema_version()
    }

    /// Returns the stable cohort identity.
    #[must_use]
    pub const fn cohort_id(&self) -> BenchmarkExecutionCohortId {
        self.cohort_id
    }

    /// Returns the UUID namespace used for deterministic run identities.
    #[must_use]
    pub const fn decision_namespace(&self) -> Uuid {
        self.decision_namespace
    }

    /// Returns the evaluated benchmark suite identity.
    #[must_use]
    pub const fn suite_id(&self) -> crate::BenchmarkSuiteId {
        self.suite_id
    }

    pub(crate) const fn workspace_definition(&self) -> &BenchmarkWorkspaceDefinitionFacts {
        &self.workspace_definition
    }

    /// Returns the deterministic case-to-run cohort.
    #[must_use]
    pub const fn cohort(&self) -> &BenchmarkExecutionCohort {
        &self.cohort
    }

    /// Returns the domain-owned benchmark evaluation.
    #[must_use]
    pub const fn evaluation(&self) -> &BenchmarkEvaluation {
        &self.evaluation
    }

    /// Returns the existing decision-diff comparison projection.
    #[must_use]
    pub const fn decision_input(&self) -> &BenchmarkDecisionComparisonInput {
        &self.decision_input
    }
}

/// Typed failures that prevent a safe execution receipt from being emitted.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BenchmarkExecutionReceiptError {
    /// Plan or cohort assembly failed validation.
    #[error(transparent)]
    Execution(#[from] BenchmarkExecutionError),
    /// Evaluation evidence could not form a safe decision-diff input.
    #[error(transparent)]
    DecisionInput(#[from] BenchmarkDecisionComparisonError),
}

fn decision_input(
    evaluation: &BenchmarkEvaluation,
    comparability_fingerprint: impl Into<String>,
) -> Result<BenchmarkDecisionComparisonInput, BenchmarkDecisionComparisonError> {
    let metrics = evaluation
        .decision()
        .checks()
        .iter()
        .map(|check| {
            BenchmarkDecisionMetricInput::new(
                check.threshold(),
                check.observed(),
                evaluation.scorecard().sample_count(check.metric()),
                evaluation.scorecard().run_count(),
                check.has_complete_coverage(),
                check.status(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    BenchmarkDecisionComparisonInput::new(
        evaluation.decision().status(),
        comparability_fingerprint,
        metrics,
    )
}

fn cohort_name(
    plan: &BenchmarkExecutionPlan,
    context_id: ContextId,
    model_version: &str,
    temperature: f32,
    executed_at: DateTime<Utc>,
) -> Vec<u8> {
    let mut name = Vec::with_capacity(128);
    name.extend_from_slice(b"contextlab/benchmark-execution-cohort/v1\0");
    name.extend_from_slice(plan.suite().id().as_uuid().as_bytes());
    name.extend_from_slice(context_id.as_uuid().as_bytes());
    append_bytes(&mut name, model_version.as_bytes());
    name.extend_from_slice(&temperature.to_bits().to_be_bytes());
    name.extend_from_slice(&executed_at.timestamp().to_be_bytes());
    name.extend_from_slice(&executed_at.timestamp_subsec_nanos().to_be_bytes());
    for case in plan.cases() {
        append_case_key(&mut name, case.key());
    }
    name
}

fn append_case_key(name: &mut Vec<u8>, key: BenchmarkExecutionCaseKey) {
    name.extend_from_slice(key.dataset_id().as_uuid().as_bytes());
    name.extend_from_slice(key.case_id().as_uuid().as_bytes());
}

fn append_bytes(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_be_bytes());
    target.extend_from_slice(value);
}
