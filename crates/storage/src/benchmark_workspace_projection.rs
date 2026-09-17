//! Immutable persistence contracts for redacted benchmark workspace projections.

use crate::{
    BenchmarkDecisionEvidence, BenchmarkDecisionId, BenchmarkEvidenceError,
    PersistBenchmarkEvaluationEvidence,
};
use async_trait::async_trait;
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkDataset, BenchmarkExecutionCohortId, BenchmarkExecutionError, BenchmarkExecutionPlan,
    BenchmarkExecutionReceipt, BenchmarkWorkspaceProjectionError, BenchmarkWorkspaceProjectionV1,
};
use contextlab_versioning::CommitId;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use uuid::Uuid;

/// Exact immutable scope of one persisted benchmark execution receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkWorkspaceProjectionReceiptScope {
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    cohort_id: BenchmarkExecutionCohortId,
}

impl BenchmarkWorkspaceProjectionReceiptScope {
    /// Creates one exact project, Context, commit, and cohort scope.
    #[must_use]
    pub const fn new(
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        cohort_id: BenchmarkExecutionCohortId,
    ) -> Self {
        Self {
            project_id,
            context_id,
            context_commit_id,
            cohort_id,
        }
    }

    /// Returns the owning project identity.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the evaluated Context identity.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the exact immutable Context commit identity.
    #[must_use]
    pub const fn context_commit_id(self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the stable execution cohort identity.
    #[must_use]
    pub const fn cohort_id(self) -> BenchmarkExecutionCohortId {
        self.cohort_id
    }
}

/// Validated append-only source for one redacted workspace projection.
///
/// The exact plan remains private because it contains sealed benchmark cases. Consumers can read
/// only the redacted [`BenchmarkWorkspaceProjectionV1`] produced by the repository reader.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistBenchmarkWorkspaceProjectionV1 {
    scope: BenchmarkWorkspaceProjectionReceiptScope,
    decision_id: BenchmarkDecisionId,
    evidence_digest: String,
    plan: BenchmarkExecutionPlan,
    receipt: BenchmarkExecutionReceipt,
}

impl PersistBenchmarkWorkspaceProjectionV1 {
    /// Joins exact definitions, sealed decision evidence, and one deterministic receipt.
    pub fn new(
        evidence: BenchmarkDecisionEvidence,
        datasets: Vec<BenchmarkDataset>,
        suite: contextlab_evaluation::BenchmarkSuite,
        receipt: BenchmarkExecutionReceipt,
    ) -> Result<Self, BenchmarkWorkspaceProjectionContractError> {
        if evidence.decision_id().as_uuid() != receipt.decision_namespace() {
            return Err(
                BenchmarkWorkspaceProjectionContractError::DecisionNamespaceMismatch {
                    evidence_decision_id: evidence.decision_id(),
                    receipt_decision_namespace: receipt.decision_namespace(),
                },
            );
        }

        let plan = BenchmarkExecutionPlan::new(suite.clone(), datasets.clone())?;
        BenchmarkWorkspaceProjectionV1::from_receipt(&receipt, &plan)?;

        let planned_case_keys = plan
            .cases()
            .iter()
            .map(contextlab_evaluation::BenchmarkExecutionCase::key)
            .collect::<Vec<_>>();
        let receipt_case_keys = receipt
            .cohort()
            .entries()
            .iter()
            .map(|entry| entry.key())
            .collect::<Vec<_>>();
        if planned_case_keys != receipt_case_keys {
            return Err(BenchmarkWorkspaceProjectionContractError::CaseProvenanceMismatch);
        }

        let reconstructed = PersistBenchmarkEvaluationEvidence::new(
            evidence.decision_id(),
            evidence.project_id(),
            evidence.context_commit_id(),
            datasets,
            suite,
            receipt.cohort().runs(),
            receipt.evaluation().clone(),
            evidence.comparability().evaluator_key(),
            evidence.comparability().evaluator_version(),
            evidence.recorded_at(),
        )?;
        if reconstructed.evidence() != &evidence
            || receipt.decision_input().comparability_fingerprint()
                != evidence.comparability().fingerprint()
        {
            return Err(BenchmarkWorkspaceProjectionContractError::DecisionEvidenceMismatch);
        }

        let scope = BenchmarkWorkspaceProjectionReceiptScope::new(
            evidence.project_id(),
            evidence.context_id(),
            evidence.context_commit_id(),
            receipt.cohort_id(),
        );
        Ok(Self {
            scope,
            decision_id: evidence.decision_id(),
            evidence_digest: evidence.evidence_digest().to_owned(),
            plan,
            receipt,
        })
    }

    /// Returns the exact immutable receipt scope accepted by the writer.
    #[must_use]
    pub const fn scope(&self) -> &BenchmarkWorkspaceProjectionReceiptScope {
        &self.scope
    }

    /// Returns the canonical digest of the sealed decision evidence bound to this receipt.
    #[must_use]
    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }

    pub(crate) fn source_facts(&self) -> BenchmarkWorkspaceProjectionSourceFacts {
        BenchmarkWorkspaceProjectionSourceFacts {
            scope: self.scope,
            decision_id: self.decision_id,
            evidence_digest: self.evidence_digest.clone(),
            receipt_schema_version: self.receipt.schema_version(),
            cases: self
                .receipt
                .cohort()
                .entries()
                .iter()
                .map(|entry| BenchmarkWorkspaceProjectionCaseLink {
                    dataset_id: entry.key().dataset_id().as_uuid(),
                    case_id: entry.key().case_id().as_uuid(),
                    run_id: entry.run().id().as_uuid(),
                })
                .collect(),
        }
    }

    pub(crate) fn project(
        &self,
    ) -> Result<BenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError> {
        Ok(BenchmarkWorkspaceProjectionV1::from_receipt(
            &self.receipt,
            &self.plan,
        )?)
    }

    pub(crate) fn project_against(
        &self,
        baseline: &Self,
    ) -> Result<BenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError> {
        if baseline.scope.project_id() != self.scope.project_id()
            || baseline.scope.context_id() != self.scope.context_id()
        {
            return Err(BenchmarkWorkspaceProjectionPersistenceError::ComparisonScopeMismatch);
        }
        if baseline.plan != self.plan {
            return Err(BenchmarkWorkspaceProjectionPersistenceError::ComparisonPlanMismatch);
        }
        Ok(BenchmarkWorkspaceProjectionV1::from_receipts(
            &baseline.receipt,
            &self.receipt,
            &self.plan,
        )?)
    }

    fn decision_key(&self) -> (Uuid, Uuid, Uuid, Uuid) {
        (
            self.scope.project_id().as_uuid(),
            self.scope.context_id().as_uuid(),
            self.scope.context_commit_id().as_uuid(),
            self.decision_id.as_uuid(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceProjectionSourceFacts {
    pub(crate) scope: BenchmarkWorkspaceProjectionReceiptScope,
    pub(crate) decision_id: BenchmarkDecisionId,
    pub(crate) evidence_digest: String,
    pub(crate) receipt_schema_version: u16,
    pub(crate) cases: Vec<BenchmarkWorkspaceProjectionCaseLink>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceProjectionCaseLink {
    pub(crate) dataset_id: Uuid,
    pub(crate) case_id: Uuid,
    pub(crate) run_id: Uuid,
}

/// Cross-contract failures that prevent a projection source from being persisted.
#[derive(Debug, Error)]
pub enum BenchmarkWorkspaceProjectionContractError {
    /// The receipt namespace does not identify the sealed decision evidence.
    #[error("benchmark workspace receipt namespace does not match sealed decision evidence")]
    DecisionNamespaceMismatch {
        /// Decision identity carried by sealed storage evidence.
        evidence_decision_id: BenchmarkDecisionId,
        /// Namespace carried by the execution receipt.
        receipt_decision_namespace: Uuid,
    },
    /// The supplied plan does not carry the receipt's exact dataset/case provenance.
    #[error("benchmark workspace receipt case provenance does not match its execution plan")]
    CaseProvenanceMismatch,
    /// The receipt does not reproduce the supplied sealed decision evidence exactly.
    #[error("benchmark workspace receipt does not match sealed decision evidence")]
    DecisionEvidenceMismatch,
    /// The sealed definitions cannot form an exact deterministic execution plan.
    #[error(transparent)]
    InvalidPlan(#[from] BenchmarkExecutionError),
    /// The redacted projection rejected the receipt and plan scope.
    #[error(transparent)]
    Projection(#[from] BenchmarkWorkspaceProjectionError),
    /// The existing evidence contract rejected reconstructed receipt evidence.
    #[error("benchmark workspace receipt cannot reproduce sealed decision evidence")]
    Evidence(#[from] BenchmarkEvidenceError),
}

/// Whether a projection source was newly persisted or replayed unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkWorkspaceProjectionWriteDisposition {
    /// The immutable projection source was persisted for the first time.
    Created,
    /// An identical immutable projection source already existed.
    Replayed,
}

/// Result of one append-only benchmark workspace projection source write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkWorkspaceProjectionWriteResult {
    scope: BenchmarkWorkspaceProjectionReceiptScope,
    disposition: BenchmarkWorkspaceProjectionWriteDisposition,
}

impl BenchmarkWorkspaceProjectionWriteResult {
    pub(crate) const fn new(
        scope: BenchmarkWorkspaceProjectionReceiptScope,
        disposition: BenchmarkWorkspaceProjectionWriteDisposition,
    ) -> Self {
        Self { scope, disposition }
    }

    /// Returns the exact persisted receipt scope.
    #[must_use]
    pub const fn scope(&self) -> &BenchmarkWorkspaceProjectionReceiptScope {
        &self.scope
    }

    /// Returns whether the source was created or replayed.
    #[must_use]
    pub const fn disposition(&self) -> BenchmarkWorkspaceProjectionWriteDisposition {
        self.disposition
    }
}

/// Exact single-receipt or baseline/revised workspace projection query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkWorkspaceProjectionV1Query {
    baseline: Option<BenchmarkWorkspaceProjectionReceiptScope>,
    revised: BenchmarkWorkspaceProjectionReceiptScope,
}

/// Exact immutable decision scope used to resolve a persisted workspace receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkWorkspaceProjectionDecisionQuery {
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    decision_id: BenchmarkDecisionId,
}

impl BenchmarkWorkspaceProjectionDecisionQuery {
    /// Creates an exact project, Context, commit, and sealed decision lookup.
    #[must_use]
    pub const fn new(
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
    ) -> Self {
        Self {
            project_id,
            context_id,
            context_commit_id,
            decision_id,
        }
    }

    /// Returns the owning project identity.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the exact Context identity.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the exact immutable Context commit identity.
    #[must_use]
    pub const fn context_commit_id(self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the exact sealed decision identity.
    #[must_use]
    pub const fn decision_id(self) -> BenchmarkDecisionId {
        self.decision_id
    }

    fn decision_key(self) -> (Uuid, Uuid, Uuid, Uuid) {
        (
            self.project_id.as_uuid(),
            self.context_id.as_uuid(),
            self.context_commit_id.as_uuid(),
            self.decision_id.as_uuid(),
        )
    }
}

impl BenchmarkWorkspaceProjectionV1Query {
    /// Requests one redacted projection with no evaluation diff.
    #[must_use]
    pub const fn single(revised: BenchmarkWorkspaceProjectionReceiptScope) -> Self {
        Self {
            baseline: None,
            revised,
        }
    }

    /// Requests one revised projection with an existing comparable baseline diff.
    #[must_use]
    pub const fn comparing(
        baseline: BenchmarkWorkspaceProjectionReceiptScope,
        revised: BenchmarkWorkspaceProjectionReceiptScope,
    ) -> Self {
        Self {
            baseline: Some(baseline),
            revised,
        }
    }

    /// Returns the optional exact baseline scope.
    #[must_use]
    pub(crate) const fn baseline(self) -> Option<BenchmarkWorkspaceProjectionReceiptScope> {
        self.baseline
    }

    /// Returns the exact revised scope.
    #[must_use]
    pub(crate) const fn revised(self) -> BenchmarkWorkspaceProjectionReceiptScope {
        self.revised
    }
}

/// Fail-closed projection persistence and read errors.
#[derive(Debug, Error)]
pub enum BenchmarkWorkspaceProjectionPersistenceError {
    /// No receipt exists at the complete requested scope.
    #[error("benchmark workspace execution receipt is unavailable at the requested scope")]
    ReceiptUnavailable {
        /// Exact unavailable scope, retained for trusted internal handling.
        scope: BenchmarkWorkspaceProjectionReceiptScope,
    },
    /// One cohort identity was reused for different immutable source facts.
    #[error("benchmark workspace execution receipt conflicts for cohort {cohort_id:?}")]
    ReceiptConflict {
        /// Conflicting stable cohort identity.
        cohort_id: BenchmarkExecutionCohortId,
    },
    /// A baseline and revised receipt do not share project and Context scope.
    #[error("benchmark workspace comparison scopes do not share one project and Context")]
    ComparisonScopeMismatch,
    /// A baseline and revised receipt do not share the exact sealed definitions.
    #[error("benchmark workspace comparison receipts use different execution plans")]
    ComparisonPlanMismatch,
    /// The existing redacted projection rejected persisted source facts.
    #[error(transparent)]
    Projection(#[from] BenchmarkWorkspaceProjectionError),
    /// The selected repository could not be read or written.
    #[error("benchmark workspace projection repository is unavailable")]
    RepositoryUnavailable,
    /// Durable source facts could not reproduce the sealed projection contract.
    #[error("stored benchmark workspace projection source is invalid")]
    StoredSourceInvalid,
}

/// Private append-only writer for validated benchmark workspace projection sources.
#[async_trait]
pub trait BenchmarkWorkspaceProjectionV1Writer: Send + Sync {
    /// Persists one immutable source or returns an identical replay.
    async fn persist_benchmark_workspace_projection(
        &self,
        command: PersistBenchmarkWorkspaceProjectionV1,
    ) -> Result<BenchmarkWorkspaceProjectionWriteResult, BenchmarkWorkspaceProjectionPersistenceError>;
}

/// Narrow reader dependency required by the local benchmark workspace adapter.
#[async_trait]
pub trait BenchmarkWorkspaceProjectionV1Reader: Send + Sync {
    /// Reads only the existing redacted V1 projection at exact immutable receipt scopes.
    async fn read_benchmark_workspace_projection(
        &self,
        query: BenchmarkWorkspaceProjectionV1Query,
    ) -> Result<BenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError>;

    /// Resolves a sealed decision to its already persisted immutable workspace receipt scope.
    ///
    /// The mapping is stored with the projection source and is never reconstructed from run
    /// identifiers by a transport or UI consumer. Implementations that do not expose this
    /// private bridge fail closed until they provide a durable resolver.
    async fn resolve_benchmark_workspace_projection_scope(
        &self,
        query: BenchmarkWorkspaceProjectionDecisionQuery,
    ) -> Result<
        Option<BenchmarkWorkspaceProjectionReceiptScope>,
        BenchmarkWorkspaceProjectionPersistenceError,
    > {
        let _ = query;
        Err(BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable)
    }
}

#[derive(Debug, Default)]
struct InMemoryBenchmarkWorkspaceProjectionState {
    receipts: BTreeMap<BenchmarkExecutionCohortId, PersistBenchmarkWorkspaceProjectionV1>,
    decision_cohorts: BTreeMap<(Uuid, Uuid, Uuid, Uuid), BenchmarkExecutionCohortId>,
}

/// Standalone in-memory adapter for immutable benchmark workspace projection sources.
#[derive(Debug, Clone, Default)]
pub struct InMemoryBenchmarkWorkspaceProjectionV1Repository {
    state: Arc<RwLock<InMemoryBenchmarkWorkspaceProjectionState>>,
}

impl InMemoryBenchmarkWorkspaceProjectionV1Repository {
    /// Creates an empty append-only projection source repository.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Writer for InMemoryBenchmarkWorkspaceProjectionV1Repository {
    async fn persist_benchmark_workspace_projection(
        &self,
        command: PersistBenchmarkWorkspaceProjectionV1,
    ) -> Result<BenchmarkWorkspaceProjectionWriteResult, BenchmarkWorkspaceProjectionPersistenceError>
    {
        let cohort_id = command.scope.cohort_id();
        let decision_key = command.decision_key();
        let mut state = self
            .state
            .write()
            .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable)?;

        if let Some(existing) = state.receipts.get(&cohort_id) {
            if existing == &command {
                return Ok(BenchmarkWorkspaceProjectionWriteResult::new(
                    command.scope,
                    BenchmarkWorkspaceProjectionWriteDisposition::Replayed,
                ));
            }
            return Err(
                BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict { cohort_id },
            );
        }
        if state
            .decision_cohorts
            .get(&decision_key)
            .is_some_and(|stored_cohort_id| *stored_cohort_id != cohort_id)
        {
            return Err(
                BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict { cohort_id },
            );
        }

        let scope = command.scope;
        state.decision_cohorts.insert(decision_key, cohort_id);
        state.receipts.insert(cohort_id, command);
        Ok(BenchmarkWorkspaceProjectionWriteResult::new(
            scope,
            BenchmarkWorkspaceProjectionWriteDisposition::Created,
        ))
    }
}

#[async_trait]
impl BenchmarkWorkspaceProjectionV1Reader for InMemoryBenchmarkWorkspaceProjectionV1Repository {
    async fn read_benchmark_workspace_projection(
        &self,
        query: BenchmarkWorkspaceProjectionV1Query,
    ) -> Result<BenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError> {
        let state = self
            .state
            .read()
            .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable)?;
        let revised = exact_receipt(&state, query.revised)?;

        let Some(baseline_scope) = query.baseline else {
            return revised.project();
        };
        let baseline = exact_receipt(&state, baseline_scope)?;
        revised.project_against(baseline)
    }

    async fn resolve_benchmark_workspace_projection_scope(
        &self,
        query: BenchmarkWorkspaceProjectionDecisionQuery,
    ) -> Result<
        Option<BenchmarkWorkspaceProjectionReceiptScope>,
        BenchmarkWorkspaceProjectionPersistenceError,
    > {
        let state = self
            .state
            .read()
            .map_err(|_| BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable)?;
        Ok(state
            .decision_cohorts
            .get(&query.decision_key())
            .map(|cohort_id| {
                BenchmarkWorkspaceProjectionReceiptScope::new(
                    query.project_id(),
                    query.context_id(),
                    query.context_commit_id(),
                    *cohort_id,
                )
            }))
    }
}

fn exact_receipt(
    state: &InMemoryBenchmarkWorkspaceProjectionState,
    scope: BenchmarkWorkspaceProjectionReceiptScope,
) -> Result<&PersistBenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError> {
    state
        .receipts
        .get(&scope.cohort_id())
        .filter(|receipt| receipt.scope == scope)
        .ok_or(BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { scope })
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_context_core::{ContextId, ProjectId};
    use contextlab_versioning::CommitId;

    #[tokio::test]
    async fn decision_scope_resolver_returns_exact_persisted_mapping() {
        let repository = InMemoryBenchmarkWorkspaceProjectionV1Repository::new();
        let project_id = ProjectId::new();
        let context_id = ContextId::new();
        let context_commit_id = CommitId::new();
        let decision_id = BenchmarkDecisionId::new();
        let cohort_id = BenchmarkExecutionCohortId::from_uuid(Uuid::new_v4());
        let query = BenchmarkWorkspaceProjectionDecisionQuery::new(
            project_id,
            context_id,
            context_commit_id,
            decision_id,
        );

        {
            let mut state = repository.state.write().expect("state lock");
            state
                .decision_cohorts
                .insert(query.decision_key(), cohort_id);
        }

        assert_eq!(
            repository
                .resolve_benchmark_workspace_projection_scope(query)
                .await
                .expect("resolver succeeds"),
            Some(BenchmarkWorkspaceProjectionReceiptScope::new(
                project_id,
                context_id,
                context_commit_id,
                cohort_id,
            ))
        );
    }

    #[tokio::test]
    async fn decision_scope_resolver_fails_closed_for_unknown_decision() {
        let repository = InMemoryBenchmarkWorkspaceProjectionV1Repository::new();
        let query = BenchmarkWorkspaceProjectionDecisionQuery::new(
            ProjectId::new(),
            ContextId::new(),
            CommitId::new(),
            BenchmarkDecisionId::new(),
        );

        assert_eq!(
            repository
                .resolve_benchmark_workspace_projection_scope(query)
                .await
                .expect("resolver succeeds"),
            None
        );
    }
}
