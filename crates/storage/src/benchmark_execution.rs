//! Private orchestration for executing sealed benchmark definitions.

use crate::{
    BenchmarkDecisionEvidence, BenchmarkDecisionId, BenchmarkDefinitionBinding,
    BenchmarkDefinitionBindingId, BenchmarkEvidenceError, BenchmarkEvidenceRepository,
    BenchmarkEvidenceWriteDisposition, BenchmarkEvidenceWriter,
    BenchmarkWorkspaceProjectionContractError, BenchmarkWorkspaceProjectionPersistenceError,
    BenchmarkWorkspaceProjectionReceiptScope, BenchmarkWorkspaceProjectionV1Writer,
    BenchmarkWorkspaceProjectionWriteDisposition, BenchmarkWorkspaceProjectionWriteResult,
    PersistBenchmarkEvaluationEvidence, PersistBenchmarkWorkspaceProjectionV1, RequestDigest,
    StorageRepositoryError,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_context_core::{ContextId, NonEmptyString, ProjectId};
use contextlab_evaluation::{
    BenchmarkCaseExecutionResult, BenchmarkExecutionCase, BenchmarkExecutionError,
    BenchmarkExecutionPlan, BenchmarkExecutionReceipt, BenchmarkExecutionReceiptError,
    BenchmarkSuiteId, EvaluationError, EvaluationRun, EvaluationRunId,
};
use contextlab_versioning::{BranchName, CommitId};
use std::fmt;
use thiserror::Error;

/// Immutable context and case scope supplied to one trusted evaluator call.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkCaseEvaluationRequest {
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    model_version: String,
    temperature: f32,
    case: BenchmarkExecutionCase,
}

impl BenchmarkCaseEvaluationRequest {
    fn new(
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        model_version: String,
        temperature: f32,
        case: BenchmarkExecutionCase,
    ) -> Self {
        Self {
            project_id,
            context_id,
            context_commit_id,
            model_version,
            temperature,
            case,
        }
    }

    /// Returns the owning project identity.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the evaluated Context identity.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the exact immutable Context commit under evaluation.
    #[must_use]
    pub const fn context_commit_id(&self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the declared model identity.
    #[must_use]
    pub fn model_version(&self) -> &str {
        &self.model_version
    }

    /// Returns the declared sampling temperature.
    #[must_use]
    pub const fn temperature(&self) -> f32 {
        self.temperature
    }

    /// Returns the immutable sealed case selected by the execution plan.
    #[must_use]
    pub const fn case(&self) -> &BenchmarkExecutionCase {
        &self.case
    }
}

/// Failure reported by a trusted benchmark evaluator.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("benchmark evaluator is unavailable")]
pub struct BenchmarkCaseEvaluatorError;

impl BenchmarkCaseEvaluatorError {
    /// Creates a redacted evaluator failure while discarding adapter diagnostics.
    #[must_use]
    pub fn new(_diagnostic: impl Into<String>) -> Self {
        Self
    }
}

/// Error raised while converting one immutable definition binding into an execution selection.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BenchmarkDefinitionBindingExecutionSelectionError {
    /// The binding was supplied for a different exact resource scope.
    #[error("benchmark definition binding scope does not match the requested execution scope")]
    ScopeMismatch {
        /// Scope expected by the caller.
        expected_project_id: ProjectId,
        /// Scope carried by the binding.
        actual_project_id: ProjectId,
        /// Context expected by the caller.
        expected_context_id: ContextId,
        /// Context carried by the binding.
        actual_context_id: ContextId,
        /// Commit expected by the caller.
        expected_context_commit_id: CommitId,
        /// Commit carried by the binding.
        actual_context_commit_id: CommitId,
    },
    /// The binding schema is not supported by this execution boundary.
    #[error(
        "benchmark definition binding schema version {actual} is unsupported; expected {expected}"
    )]
    InvalidSchemaVersion {
        /// Supported schema version.
        expected: u16,
        /// Binding schema version.
        actual: u16,
    },
    /// The immutable suite and datasets could not form a deterministic plan.
    #[error("benchmark definition binding execution plan is invalid")]
    Plan(#[from] BenchmarkExecutionError),
}

/// Safe identity and private plan source selected from one exact immutable binding.
///
/// The suite and dataset definitions are retained for the trusted execution layer, but the
/// public accessors expose only stable identity and counts. Its custom `Debug` implementation is
/// deliberately redacted so diagnostics cannot print case inputs or expected outputs.
#[derive(Clone, PartialEq)]
pub struct BenchmarkDefinitionBindingExecutionSelection {
    binding_id: BenchmarkDefinitionBindingId,
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    branch: BranchName,
    schema_version: u16,
    suite: contextlab_evaluation::BenchmarkSuite,
    datasets: Vec<contextlab_evaluation::BenchmarkDataset>,
    dataset_ids: Vec<contextlab_evaluation::BenchmarkDatasetId>,
    case_count: usize,
}

impl fmt::Debug for BenchmarkDefinitionBindingExecutionSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BenchmarkDefinitionBindingExecutionSelection")
            .field("binding_id", &self.binding_id)
            .field("project_id", &self.project_id)
            .field("context_id", &self.context_id)
            .field("context_commit_id", &self.context_commit_id)
            .field("branch", &self.branch)
            .field("schema_version", &self.schema_version)
            .field("suite_id", &self.suite.id())
            .field("dataset_ids", &self.dataset_ids)
            .field("case_count", &self.case_count)
            .finish()
    }
}

impl BenchmarkDefinitionBindingExecutionSelection {
    /// Builds a selection while requiring the caller's exact project/Context/commit scope.
    pub fn for_exact_scope(
        binding: &BenchmarkDefinitionBinding,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Self, BenchmarkDefinitionBindingExecutionSelectionError> {
        if binding.project_id() != project_id
            || binding.context_id() != context_id
            || binding.context_commit_id() != context_commit_id
        {
            return Err(
                BenchmarkDefinitionBindingExecutionSelectionError::ScopeMismatch {
                    expected_project_id: project_id,
                    actual_project_id: binding.project_id(),
                    expected_context_id: context_id,
                    actual_context_id: binding.context_id(),
                    expected_context_commit_id: context_commit_id,
                    actual_context_commit_id: binding.context_commit_id(),
                },
            );
        }
        Self::try_from_binding(binding)
    }

    /// Builds a selection from a validated immutable binding.
    pub fn try_from_binding(
        binding: &BenchmarkDefinitionBinding,
    ) -> Result<Self, BenchmarkDefinitionBindingExecutionSelectionError> {
        if binding.schema_version() != crate::BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION {
            return Err(
                BenchmarkDefinitionBindingExecutionSelectionError::InvalidSchemaVersion {
                    expected: crate::BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
                    actual: binding.schema_version(),
                },
            );
        }
        let suite = binding.suite().clone();
        let datasets = binding.datasets().to_vec();
        let plan = BenchmarkExecutionPlan::new(suite.clone(), datasets.clone())?;
        let dataset_ids = binding.dataset_ids();
        Ok(Self {
            binding_id: binding.id(),
            project_id: binding.project_id(),
            context_id: binding.context_id(),
            context_commit_id: binding.context_commit_id(),
            branch: binding.branch().clone(),
            schema_version: binding.schema_version(),
            suite,
            datasets,
            dataset_ids,
            case_count: plan.case_count(),
        })
    }

    /// Returns the immutable binding identity.
    #[must_use]
    pub const fn binding_id(&self) -> BenchmarkDefinitionBindingId {
        self.binding_id
    }

    /// Returns the exact project scope.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the exact Context scope.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the exact immutable Context commit scope.
    #[must_use]
    pub const fn context_commit_id(&self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the guarded branch name.
    #[must_use]
    pub const fn branch(&self) -> &BranchName {
        &self.branch
    }

    /// Returns the explicit binding schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the selected immutable suite identity.
    #[must_use]
    pub const fn suite_id(&self) -> contextlab_evaluation::BenchmarkSuiteId {
        self.suite.id()
    }

    /// Returns selected datasets in deterministic identifier order.
    #[must_use]
    pub fn dataset_ids(&self) -> &[contextlab_evaluation::BenchmarkDatasetId] {
        &self.dataset_ids
    }

    /// Returns the number of immutable cases selected for evaluation.
    #[must_use]
    pub const fn case_count(&self) -> usize {
        self.case_count
    }

    fn execution_plan(
        &self,
    ) -> Result<BenchmarkExecutionPlan, BenchmarkDefinitionBindingExecutionSelectionError> {
        Ok(BenchmarkExecutionPlan::new(
            self.suite.clone(),
            self.datasets.clone(),
        )?)
    }

    pub(crate) fn definition_parts(
        &self,
    ) -> (
        contextlab_evaluation::BenchmarkSuite,
        Vec<contextlab_evaluation::BenchmarkDataset>,
    ) {
        (self.suite.clone(), self.datasets.clone())
    }
}

/// Private port that evaluates one sealed benchmark case.
#[async_trait]
pub trait BenchmarkCaseEvaluator: Send + Sync {
    /// Produces measurements for the exact case and Context commit in the request.
    async fn evaluate_case(
        &self,
        request: BenchmarkCaseEvaluationRequest,
    ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError>;
}

/// Validated request to execute and persist one benchmark decision.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionRequest {
    decision_id: BenchmarkDecisionId,
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    suite_id: BenchmarkSuiteId,
    model_version: String,
    temperature: f32,
    evaluator_key: String,
    evaluator_version: String,
    recorded_at: DateTime<Utc>,
    definition_selection: Option<BenchmarkDefinitionBindingExecutionSelection>,
    idempotency_key: Option<crate::IdempotencyKey>,
    request_digest: Option<RequestDigest>,
}

impl BenchmarkExecutionRequest {
    /// Creates a validated execution request without accepting definition or measurement payloads.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        decision_id: BenchmarkDecisionId,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        suite_id: BenchmarkSuiteId,
        model_version: impl Into<String>,
        temperature: f32,
        evaluator_key: impl Into<String>,
        evaluator_version: impl Into<String>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkExecutionRequestError> {
        let model_version = model_version.into();
        EvaluationRun::new(
            context_id,
            &model_version,
            temperature,
            Vec::new(),
            recorded_at,
        )
        .map_err(BenchmarkExecutionRequestError::InvalidRunConfiguration)?;
        let evaluator_key = NonEmptyString::new("benchmark evaluator key", evaluator_key)
            .map_err(|_| BenchmarkExecutionRequestError::InvalidEvaluatorIdentity)?
            .as_str()
            .to_owned();
        let evaluator_version =
            NonEmptyString::new("benchmark evaluator version", evaluator_version)
                .map_err(|_| BenchmarkExecutionRequestError::InvalidEvaluatorIdentity)?
                .as_str()
                .to_owned();

        Ok(Self {
            decision_id,
            project_id,
            context_id,
            context_commit_id,
            suite_id,
            model_version,
            temperature,
            evaluator_key,
            evaluator_version,
            recorded_at,
            definition_selection: None,
            idempotency_key: None,
            request_digest: None,
        })
    }

    /// Protects this request with one canonical idempotency key and request digest.
    #[must_use]
    pub fn with_idempotency(
        mut self,
        idempotency_key: crate::IdempotencyKey,
        request_digest: RequestDigest,
    ) -> Self {
        self.idempotency_key = Some(idempotency_key);
        self.request_digest = Some(request_digest);
        self
    }

    /// Creates an execution request whose suite and dataset selection is bound to one exact
    /// immutable benchmark definition binding.
    #[allow(clippy::too_many_arguments)]
    pub fn from_definition_binding(
        decision_id: BenchmarkDecisionId,
        binding: &BenchmarkDefinitionBinding,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        model_version: impl Into<String>,
        temperature: f32,
        evaluator_key: impl Into<String>,
        evaluator_version: impl Into<String>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkExecutionRequestError> {
        let selection = BenchmarkDefinitionBindingExecutionSelection::for_exact_scope(
            binding,
            project_id,
            context_id,
            context_commit_id,
        )?;
        let mut request = Self::new(
            decision_id,
            project_id,
            context_id,
            context_commit_id,
            selection.suite_id(),
            model_version,
            temperature,
            evaluator_key,
            evaluator_version,
            recorded_at,
        )?;
        request.definition_selection = Some(selection);
        Ok(request)
    }

    /// Returns the requested immutable decision identity.
    #[must_use]
    pub const fn decision_id(&self) -> BenchmarkDecisionId {
        self.decision_id
    }
}

/// Input validation errors for a benchmark execution request.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BenchmarkExecutionRequestError {
    /// The run configuration is invalid before evaluator invocation.
    #[error(transparent)]
    InvalidRunConfiguration(#[from] EvaluationError),
    /// The evaluator identity cannot identify immutable evidence.
    #[error("benchmark evaluator key and version must not be blank")]
    InvalidEvaluatorIdentity,
    /// The immutable definition binding could not be selected for the request scope.
    #[error(transparent)]
    InvalidDefinitionBindingSelection(#[from] BenchmarkDefinitionBindingExecutionSelectionError),
}

/// Whether an execution newly persisted evidence or reused an existing exact decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkExecutionDisposition {
    /// The evaluator ran and the writer created evidence.
    Created,
    /// Existing or concurrently persisted evidence was reused.
    Replayed,
}

/// Result of one private benchmark execution request.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionResult {
    disposition: BenchmarkExecutionDisposition,
    evidence: BenchmarkDecisionEvidence,
    workspace_projection_scope: BenchmarkWorkspaceProjectionReceiptScope,
    workspace_projection_disposition: BenchmarkWorkspaceProjectionWriteDisposition,
}

impl BenchmarkExecutionResult {
    fn new(
        disposition: BenchmarkExecutionDisposition,
        evidence: BenchmarkDecisionEvidence,
        workspace_projection: BenchmarkWorkspaceProjectionWriteResult,
    ) -> Self {
        Self {
            disposition,
            evidence,
            workspace_projection_scope: *workspace_projection.scope(),
            workspace_projection_disposition: workspace_projection.disposition(),
        }
    }

    /// Returns whether execution created or reused immutable evidence.
    #[must_use]
    pub const fn disposition(&self) -> BenchmarkExecutionDisposition {
        self.disposition
    }

    /// Returns the immutable evidence artifact available to existing readers.
    #[must_use]
    pub const fn evidence(&self) -> &BenchmarkDecisionEvidence {
        &self.evidence
    }

    /// Returns the exact project, Context, commit, and cohort scope materialized for workspace reads.
    #[must_use]
    pub const fn workspace_projection_scope(&self) -> &BenchmarkWorkspaceProjectionReceiptScope {
        &self.workspace_projection_scope
    }

    /// Returns whether the immutable workspace projection source was created or replayed.
    #[must_use]
    pub const fn workspace_projection_disposition(
        &self,
    ) -> BenchmarkWorkspaceProjectionWriteDisposition {
        self.workspace_projection_disposition
    }
}

/// Errors from private benchmark execution orchestration.
#[derive(Debug, Error)]
pub enum BenchmarkExecutionServiceError {
    /// Definition or evidence storage failed.
    #[error("benchmark execution storage access failed")]
    Storage(#[from] StorageRepositoryError),
    /// A sealed suite was absent from the declared project.
    #[error("benchmark suite is unavailable")]
    SuiteUnavailable,
    /// A suite-referenced dataset was absent from the declared project.
    #[error("benchmark dataset is unavailable")]
    DatasetUnavailable,
    /// The pure domain plan rejected definitions or evaluator output.
    #[error("benchmark execution plan is invalid")]
    Plan(#[from] BenchmarkExecutionError),
    /// The selected immutable definition binding could not form an execution plan.
    #[error("benchmark definition binding execution selection is invalid")]
    BindingSelection(#[from] BenchmarkDefinitionBindingExecutionSelectionError),
    /// The evaluator failed before any evidence write.
    #[error(transparent)]
    Evaluator(#[from] BenchmarkCaseEvaluatorError),
    /// Immutable evidence could not be prepared from valid domain output.
    #[error("benchmark execution evidence is invalid")]
    Evidence(#[from] BenchmarkEvidenceError),
    /// A deterministic receipt could not be reconstructed from domain-owned evidence.
    #[error("benchmark execution receipt is invalid")]
    Receipt(#[from] BenchmarkExecutionReceiptError),
    /// Sealed evidence, definitions, and the reconstructed receipt did not agree exactly.
    #[error("benchmark execution workspace projection source is invalid")]
    WorkspaceProjectionContract(#[from] BenchmarkWorkspaceProjectionContractError),
    /// Persisting or replaying the immutable workspace projection failed.
    #[error("benchmark execution workspace projection persistence failed")]
    WorkspaceProjection(#[from] BenchmarkWorkspaceProjectionPersistenceError),
    /// An exact sealed run needed for decision replay was unavailable.
    #[error("benchmark execution replay stored run is unavailable")]
    StoredRunUnavailable {
        /// Missing immutable run identity.
        run_id: EvaluationRunId,
    },
    /// Existing decision evidence did not match the exact replay request or loaded definitions.
    #[error("benchmark execution replay evidence does not match the request")]
    StoredEvidenceMismatch,
    /// The request supplied only one half of the idempotency contract.
    #[error("benchmark execution idempotency key and request digest must be supplied together")]
    InvalidIdempotencyContract,
}

/// Private service that executes sealed cases and persists the immutable result.
#[derive(Debug)]
pub struct BenchmarkExecutionService<'a, Repository: ?Sized, Evaluator: ?Sized> {
    repository: &'a Repository,
    evaluator: &'a Evaluator,
}

impl<'a, Repository: ?Sized, Evaluator: ?Sized>
    BenchmarkExecutionService<'a, Repository, Evaluator>
{
    /// Binds execution to one evidence repository and one trusted evaluator port.
    #[must_use]
    pub const fn new(repository: &'a Repository, evaluator: &'a Evaluator) -> Self {
        Self {
            repository,
            evaluator,
        }
    }
}

impl<Repository: ?Sized, Evaluator: ?Sized> BenchmarkExecutionService<'_, Repository, Evaluator>
where
    Repository: BenchmarkEvidenceRepository
        + BenchmarkEvidenceWriter
        + BenchmarkWorkspaceProjectionV1Writer,
    Evaluator: BenchmarkCaseEvaluator,
{
    /// Executes one sealed suite at an exact Context commit and persists its decision evidence.
    pub async fn execute(
        &self,
        request: BenchmarkExecutionRequest,
    ) -> Result<BenchmarkExecutionResult, BenchmarkExecutionServiceError> {
        if let Some(evidence) = self.load_idempotent_evidence(&request).await? {
            let replay_request = request_for_stored_evidence(&request, &evidence);
            let projection = self
                .materialize_workspace_projection(&replay_request, &evidence)
                .await?;
            return Ok(BenchmarkExecutionResult::new(
                BenchmarkExecutionDisposition::Replayed,
                evidence,
                projection,
            ));
        }

        if let Some(evidence) = self
            .repository
            .get_benchmark_decision(
                request.project_id,
                request.context_id,
                request.context_commit_id,
                request.decision_id,
            )
            .await?
        {
            let projection = self
                .materialize_workspace_projection(&request, &evidence)
                .await?;
            return Ok(BenchmarkExecutionResult::new(
                BenchmarkExecutionDisposition::Replayed,
                evidence,
                projection,
            ));
        }

        let (datasets, plan) = if let Some(selection) = request.definition_selection.as_ref() {
            let plan = selection.execution_plan()?;
            let (_, datasets) = selection.definition_parts();
            (datasets, plan)
        } else {
            let suite = self
                .repository
                .get_benchmark_suite(request.project_id, request.suite_id)
                .await?
                .ok_or(BenchmarkExecutionServiceError::SuiteUnavailable)?;
            if suite.id() != request.suite_id {
                return Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch);
            }
            let mut datasets = Vec::with_capacity(suite.dataset_ids().len());
            for dataset_id in suite.dataset_ids() {
                let dataset = self
                    .repository
                    .get_benchmark_dataset(request.project_id, *dataset_id)
                    .await?
                    .ok_or(BenchmarkExecutionServiceError::DatasetUnavailable)?;
                if dataset.id() != *dataset_id {
                    return Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch);
                }
                datasets.push(dataset);
            }
            let plan = BenchmarkExecutionPlan::new(suite.clone(), datasets.clone())?;
            (datasets, plan)
        };
        let mut results = Vec::with_capacity(plan.case_count());
        for case in plan.cases() {
            results.push(
                self.evaluator
                    .evaluate_case(BenchmarkCaseEvaluationRequest::new(
                        request.project_id,
                        request.context_id,
                        request.context_commit_id,
                        request.model_version.clone(),
                        request.temperature,
                        case.clone(),
                    ))
                    .await?,
            );
        }

        let cohort = plan.assemble_cohort(
            request.decision_id.as_uuid(),
            request.context_id,
            &request.model_version,
            request.temperature,
            request.recorded_at,
            results,
        )?;
        let runs = cohort.runs();
        let evaluation = plan.suite().evaluate_runs(&runs);
        let command = PersistBenchmarkEvaluationEvidence::new(
            request.decision_id,
            request.project_id,
            request.context_commit_id,
            datasets.clone(),
            plan.suite().clone(),
            runs,
            evaluation,
            request.evaluator_key.clone(),
            request.evaluator_version.clone(),
            request.recorded_at,
        )?;
        let command = match (
            request.idempotency_key.clone(),
            request.request_digest.clone(),
        ) {
            (Some(idempotency_key), Some(request_digest)) => {
                command.with_idempotency(idempotency_key, request_digest)
            }
            (None, None) => command,
            _ => return Err(BenchmarkExecutionServiceError::InvalidIdempotencyContract),
        };
        let persisted = self
            .repository
            .persist_benchmark_evaluation(command)
            .await?;
        let disposition = match persisted.disposition() {
            BenchmarkEvidenceWriteDisposition::Created => BenchmarkExecutionDisposition::Created,
            BenchmarkEvidenceWriteDisposition::Replayed => BenchmarkExecutionDisposition::Replayed,
        };
        let projection = self
            .materialize_workspace_projection(
                &request_for_stored_evidence(&request, persisted.evidence()),
                persisted.evidence(),
            )
            .await?;

        Ok(BenchmarkExecutionResult::new(
            disposition,
            persisted.evidence().clone(),
            projection,
        ))
    }

    async fn load_idempotent_evidence(
        &self,
        request: &BenchmarkExecutionRequest,
    ) -> Result<Option<BenchmarkDecisionEvidence>, BenchmarkExecutionServiceError> {
        let (Some(idempotency_key), Some(request_digest)) = (
            request.idempotency_key.as_ref(),
            request.request_digest.as_ref(),
        ) else {
            if request.idempotency_key.is_some() || request.request_digest.is_some() {
                return Err(BenchmarkExecutionServiceError::InvalidIdempotencyContract);
            }
            return Ok(None);
        };

        let Some(receipt) = self
            .repository
            .get_benchmark_execution_idempotency(
                request.project_id,
                request.context_id,
                request.context_commit_id,
                idempotency_key,
            )
            .await?
        else {
            return Ok(None);
        };
        if receipt.request_digest() != request_digest {
            return Err(BenchmarkExecutionServiceError::Storage(
                StorageRepositoryError::IdempotencyKeyReused {
                    context_id: request.context_id.to_string(),
                    principal_id: "benchmark-execution".to_owned(),
                    idempotency_key: idempotency_key.to_string(),
                },
            ));
        }
        self.repository
            .get_benchmark_decision(
                request.project_id,
                request.context_id,
                request.context_commit_id,
                receipt.decision_id(),
            )
            .await?
            .ok_or(BenchmarkExecutionServiceError::StoredEvidenceMismatch)
            .map(Some)
    }

    async fn materialize_workspace_projection(
        &self,
        request: &BenchmarkExecutionRequest,
        evidence: &BenchmarkDecisionEvidence,
    ) -> Result<BenchmarkWorkspaceProjectionWriteResult, BenchmarkExecutionServiceError> {
        if evidence.decision_id() != request.decision_id
            || evidence.project_id() != request.project_id
            || evidence.context_id() != request.context_id
            || evidence.context_commit_id() != request.context_commit_id
            || evidence.suite_id() != request.suite_id
            || !same_persisted_timestamp(evidence.recorded_at(), request.recorded_at)
            || evidence.comparability().evaluator_key() != request.evaluator_key
            || evidence.comparability().evaluator_version() != request.evaluator_version
        {
            return Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch);
        }

        let (suite, datasets, plan) = if let Some(selection) = request.definition_selection.as_ref()
        {
            if selection.project_id() != evidence.project_id()
                || selection.context_id() != evidence.context_id()
                || selection.context_commit_id() != evidence.context_commit_id()
                || selection.suite_id() != evidence.suite_id()
                || selection.dataset_ids() != evidence.dataset_ids()
            {
                return Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch);
            }
            let plan = selection.execution_plan()?;
            let (suite, datasets) = selection.definition_parts();
            (suite, datasets, plan)
        } else {
            let suite = self
                .repository
                .get_benchmark_suite(evidence.project_id(), evidence.suite_id())
                .await?
                .ok_or(BenchmarkExecutionServiceError::SuiteUnavailable)?;
            if suite.id() != evidence.suite_id() || suite.dataset_ids() != evidence.dataset_ids() {
                return Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch);
            }
            let mut datasets = Vec::with_capacity(evidence.dataset_ids().len());
            for dataset_id in evidence.dataset_ids() {
                let dataset = self
                    .repository
                    .get_benchmark_dataset(evidence.project_id(), *dataset_id)
                    .await?
                    .ok_or(BenchmarkExecutionServiceError::DatasetUnavailable)?;
                if dataset.id() != *dataset_id {
                    return Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch);
                }
                datasets.push(dataset);
            }
            let plan = BenchmarkExecutionPlan::new(suite.clone(), datasets.clone())?;
            (suite, datasets, plan)
        };

        let mut runs = Vec::with_capacity(evidence.run_ids().len());
        for run_id in evidence.run_ids() {
            runs.push(
                self.repository
                    .get_benchmark_run(
                        evidence.project_id(),
                        evidence.context_id(),
                        evidence.context_commit_id(),
                        *run_id,
                    )
                    .await?
                    .ok_or(BenchmarkExecutionServiceError::StoredRunUnavailable {
                        run_id: *run_id,
                    })?,
            );
        }
        let first_run = runs
            .first()
            .ok_or(BenchmarkExecutionServiceError::StoredEvidenceMismatch)?;
        if first_run.model_version() != request.model_version
            || first_run.temperature().to_bits() != request.temperature.to_bits()
            || !same_persisted_timestamp(first_run.executed_at(), evidence.recorded_at())
        {
            return Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch);
        }
        let model_version = first_run.model_version().to_owned();
        let temperature = first_run.temperature();
        let executed_at = first_run.executed_at();
        let results = plan.reconstruct_results_from_runs(
            evidence.decision_id().as_uuid(),
            evidence.context_id(),
            &model_version,
            temperature,
            executed_at,
            runs,
        )?;
        let receipt = BenchmarkExecutionReceipt::from_plan(
            &plan,
            evidence.decision_id().as_uuid(),
            evidence.context_id(),
            &model_version,
            temperature,
            executed_at,
            evidence.comparability().fingerprint(),
            results,
        )?;
        self.repository
            .persist_benchmark_workspace_projection(PersistBenchmarkWorkspaceProjectionV1::new(
                evidence.clone(),
                datasets,
                suite,
                receipt,
            )?)
            .await
            .map_err(BenchmarkExecutionServiceError::from)
    }
}

fn request_for_stored_evidence(
    request: &BenchmarkExecutionRequest,
    evidence: &BenchmarkDecisionEvidence,
) -> BenchmarkExecutionRequest {
    let mut replay_request = request.clone();
    replay_request.decision_id = evidence.decision_id();
    replay_request.suite_id = evidence.suite_id();
    replay_request.recorded_at = evidence.recorded_at();
    replay_request
}

fn same_persisted_timestamp(left: DateTime<Utc>, right: DateTime<Utc>) -> bool {
    left.timestamp_micros() == right.timestamp_micros()
}
