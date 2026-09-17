//! Private immutable benchmark-definition authoring and exact Context binding.

use crate::{IdempotencyKey, RequestDigest, StorageRepositoryError};
use async_trait::async_trait;
use chrono::{DateTime, Timelike, Utc};
use contextlab_auth::AuthenticatedPrincipal;
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{BenchmarkDataset, BenchmarkDatasetId, BenchmarkSuite};
use contextlab_versioning::{BranchName, CommitId};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Current schema version for a private benchmark-definition binding.
pub const BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION: u16 = 1;

/// Stable identity of one immutable benchmark-definition binding revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BenchmarkDefinitionBindingId(Uuid);

impl BenchmarkDefinitionBindingId {
    /// Wraps a stable UUID.
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

impl fmt::Display for BenchmarkDefinitionBindingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Errors raised before a benchmark definition can reach a repository.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BenchmarkDefinitionBindingError {
    /// The command used an unsupported schema version.
    #[error(
        "benchmark definition binding schema version {actual} is unsupported; expected {expected}"
    )]
    InvalidSchemaVersion {
        /// Supported schema version.
        expected: u16,
        /// Supplied schema version.
        actual: u16,
    },
    /// The expected branch head was not the exact immutable Context source.
    #[error(
        "benchmark definition binding expected head {expected} does not match source {source_commit_id}"
    )]
    ExpectedHeadDoesNotMatchSource {
        /// Immutable Context commit selected by the binding.
        source_commit_id: CommitId,
        /// Branch head supplied by the caller.
        expected: CommitId,
    },
    /// No complete dataset definitions were supplied.
    #[error("benchmark definition binding must include at least one dataset")]
    EmptyDatasets,
    /// The idempotency key was empty after validation.
    #[error("benchmark definition binding idempotency key must not be empty")]
    EmptyIdempotencyKey,
    /// The request digest was empty after validation.
    #[error("benchmark definition binding request digest must not be empty")]
    EmptyRequestDigest,
    /// Supplied datasets did not exactly match suite membership.
    #[error("benchmark definition binding dataset membership does not match the suite")]
    DatasetMembershipMismatch,
}

/// Immutable benchmark definitions bound to one exact Context commit.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDefinitionBinding {
    id: BenchmarkDefinitionBindingId,
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    branch: BranchName,
    schema_version: u16,
    datasets: Vec<BenchmarkDataset>,
    suite: BenchmarkSuite,
    captured_at: DateTime<Utc>,
}

impl BenchmarkDefinitionBinding {
    /// Rehydrates one persisted binding after validating its payload schema version.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: BenchmarkDefinitionBindingId,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        branch: BranchName,
        schema_version: u16,
        mut datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkDefinitionBindingError> {
        validate_schema_version(schema_version)?;
        validate_definition_membership(&mut datasets, &suite)?;
        Ok(Self {
            id,
            project_id,
            context_id,
            context_commit_id,
            branch,
            schema_version,
            datasets,
            suite,
            captured_at: postgres_timestamp_precision(captured_at),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: BenchmarkDefinitionBindingId,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        branch: BranchName,
        schema_version: u16,
        datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, BenchmarkDefinitionBindingError> {
        if schema_version != BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION {
            return Err(BenchmarkDefinitionBindingError::InvalidSchemaVersion {
                expected: BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
                actual: schema_version,
            });
        }
        Self::rehydrate(
            id,
            project_id,
            context_id,
            context_commit_id,
            branch,
            schema_version,
            datasets,
            suite,
            captured_at,
        )
    }

    /// Returns the immutable binding identity.
    #[must_use]
    pub const fn id(&self) -> BenchmarkDefinitionBindingId {
        self.id
    }

    /// Returns the owning project.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the owning Context.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the exact immutable Context commit.
    #[must_use]
    pub const fn context_commit_id(&self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the branch whose head was guarded when this binding was authored.
    #[must_use]
    pub const fn branch(&self) -> &BranchName {
        &self.branch
    }

    /// Returns the explicit schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns datasets in stable identifier order.
    #[must_use]
    pub fn datasets(&self) -> &[BenchmarkDataset] {
        &self.datasets
    }

    /// Returns dataset IDs in stable identifier order.
    #[must_use]
    pub fn dataset_ids(&self) -> Vec<BenchmarkDatasetId> {
        self.datasets.iter().map(BenchmarkDataset::id).collect()
    }

    /// Returns the immutable suite policy.
    #[must_use]
    pub const fn suite(&self) -> &BenchmarkSuite {
        &self.suite
    }

    /// Returns the authoring capture time.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }
}

/// Redacted, immutable metadata for one exact benchmark-definition binding.
///
/// This projection deliberately omits cases, inputs, expected outputs, thresholds, and request
/// digests so it can cross a protected local read boundary without exposing authoring payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDefinitionBindingSummary {
    binding_id: BenchmarkDefinitionBindingId,
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    branch: BranchName,
    schema_version: u16,
    suite_id: contextlab_evaluation::BenchmarkSuiteId,
    suite_name: String,
    dataset_ids: Vec<BenchmarkDatasetId>,
    dataset_names: Vec<String>,
    captured_at: DateTime<Utc>,
}

impl BenchmarkDefinitionBindingSummary {
    /// Builds a redacted projection without copying private case payloads.
    #[must_use]
    pub fn from_binding(binding: &BenchmarkDefinitionBinding) -> Self {
        Self {
            binding_id: binding.id,
            project_id: binding.project_id,
            context_id: binding.context_id,
            context_commit_id: binding.context_commit_id,
            branch: binding.branch.clone(),
            schema_version: binding.schema_version,
            suite_id: binding.suite.id(),
            suite_name: binding.suite.name().to_owned(),
            dataset_ids: binding.dataset_ids(),
            dataset_names: binding
                .datasets
                .iter()
                .map(|dataset| dataset.name().to_owned())
                .collect(),
            captured_at: binding.captured_at,
        }
    }

    /// Returns the immutable binding identity.
    #[must_use]
    pub const fn binding_id(&self) -> BenchmarkDefinitionBindingId {
        self.binding_id
    }

    /// Returns the owning project.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the owning Context.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the exact immutable Context commit.
    #[must_use]
    pub const fn context_commit_id(&self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the guarded branch.
    #[must_use]
    pub const fn branch(&self) -> &BranchName {
        &self.branch
    }

    /// Returns the definition schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the immutable suite identity.
    #[must_use]
    pub const fn suite_id(&self) -> contextlab_evaluation::BenchmarkSuiteId {
        self.suite_id
    }

    /// Returns the immutable suite name.
    #[must_use]
    pub fn suite_name(&self) -> &str {
        &self.suite_name
    }

    /// Returns dataset identities in deterministic order.
    #[must_use]
    pub fn dataset_ids(&self) -> &[BenchmarkDatasetId] {
        &self.dataset_ids
    }

    /// Returns redacted dataset names in the same deterministic order as `dataset_ids`.
    #[must_use]
    pub fn dataset_names(&self) -> &[String] {
        &self.dataset_names
    }

    /// Returns the immutable capture time.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }
}

/// Validated request for an atomic private benchmark-definition binding.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDefinitionBindingCommand {
    principal: AuthenticatedPrincipal,
    binding_id: BenchmarkDefinitionBindingId,
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
    branch: BranchName,
    expected_head: CommitId,
    idempotency_key: IdempotencyKey,
    request_digest: RequestDigest,
    schema_version: u16,
    datasets: Vec<BenchmarkDataset>,
    suite: BenchmarkSuite,
    captured_at: DateTime<Utc>,
}

impl BenchmarkDefinitionBindingCommand {
    /// Creates and validates a private benchmark-definition binding command.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        principal: AuthenticatedPrincipal,
        binding_id: Uuid,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: impl Into<String>,
        request_digest: impl Into<String>,
        datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        captured_at: DateTime<Utc>,
        schema_version: u16,
    ) -> Result<Self, BenchmarkDefinitionBindingError> {
        Self::try_new(
            principal,
            binding_id,
            project_id,
            context_id,
            context_commit_id,
            branch,
            expected_head,
            idempotency_key,
            request_digest,
            datasets,
            suite,
            captured_at,
            schema_version,
        )
    }

    /// Fallible constructor with an explicit name for adapter-facing callers.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        principal: AuthenticatedPrincipal,
        binding_id: Uuid,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: impl Into<String>,
        request_digest: impl Into<String>,
        mut datasets: Vec<BenchmarkDataset>,
        suite: BenchmarkSuite,
        captured_at: DateTime<Utc>,
        schema_version: u16,
    ) -> Result<Self, BenchmarkDefinitionBindingError> {
        validate_schema_version(schema_version)?;
        if context_commit_id != expected_head {
            return Err(
                BenchmarkDefinitionBindingError::ExpectedHeadDoesNotMatchSource {
                    source_commit_id: context_commit_id,
                    expected: expected_head,
                },
            );
        }
        validate_definition_membership(&mut datasets, &suite)?;
        let idempotency_key = IdempotencyKey::new(idempotency_key)
            .map_err(|_| BenchmarkDefinitionBindingError::EmptyIdempotencyKey)?;
        let request_digest = RequestDigest::new(request_digest)
            .map_err(|_| BenchmarkDefinitionBindingError::EmptyRequestDigest)?;
        Ok(Self {
            principal,
            binding_id: BenchmarkDefinitionBindingId::from_uuid(binding_id),
            project_id,
            context_id,
            context_commit_id,
            branch,
            expected_head,
            idempotency_key,
            request_digest,
            schema_version,
            datasets,
            suite,
            captured_at: postgres_timestamp_precision(captured_at),
        })
    }

    /// Returns the exact Context source.
    #[must_use]
    pub const fn context_commit_id(&self) -> CommitId {
        self.context_commit_id
    }

    /// Returns the expected branch head guard.
    #[must_use]
    pub const fn expected_head(&self) -> CommitId {
        self.expected_head
    }

    /// Returns the explicit schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns datasets in stable identifier order.
    #[must_use]
    pub fn datasets(&self) -> &[BenchmarkDataset] {
        &self.datasets
    }

    /// Returns dataset IDs in stable identifier order.
    #[must_use]
    pub fn dataset_ids(&self) -> Vec<BenchmarkDatasetId> {
        self.datasets.iter().map(BenchmarkDataset::id).collect()
    }

    /// Returns the immutable suite policy.
    #[must_use]
    pub const fn suite(&self) -> &BenchmarkSuite {
        &self.suite
    }

    /// Returns the capture time carried by the command.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }

    pub(crate) fn with_captured_at(mut self, captured_at: DateTime<Utc>) -> Self {
        self.captured_at = captured_at;
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        AuthenticatedPrincipal,
        BenchmarkDefinitionBinding,
        IdempotencyKey,
        RequestDigest,
    ) {
        let binding = BenchmarkDefinitionBinding::new(
            self.binding_id,
            self.project_id,
            self.context_id,
            self.context_commit_id,
            self.branch,
            self.schema_version,
            self.datasets,
            self.suite,
            self.captured_at,
        )
        .expect("validated benchmark definition command must form a binding");
        (
            self.principal,
            binding,
            self.idempotency_key,
            self.request_digest,
        )
    }
}

fn validate_schema_version(schema_version: u16) -> Result<(), BenchmarkDefinitionBindingError> {
    if schema_version != BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION {
        return Err(BenchmarkDefinitionBindingError::InvalidSchemaVersion {
            expected: BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
            actual: schema_version,
        });
    }
    Ok(())
}

fn postgres_timestamp_precision(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    timestamp
        .with_nanosecond((timestamp.nanosecond() / 1_000) * 1_000)
        .expect("a valid UTC timestamp keeps its microsecond-truncated value")
}

fn validate_definition_membership(
    datasets: &mut [BenchmarkDataset],
    suite: &BenchmarkSuite,
) -> Result<(), BenchmarkDefinitionBindingError> {
    if datasets.is_empty() {
        return Err(BenchmarkDefinitionBindingError::EmptyDatasets);
    }
    datasets.sort_by_key(BenchmarkDataset::id);
    let dataset_ids = datasets
        .iter()
        .map(BenchmarkDataset::id)
        .collect::<Vec<_>>();
    if dataset_ids != suite.dataset_ids() {
        return Err(BenchmarkDefinitionBindingError::DatasetMembershipMismatch);
    }
    Ok(())
}

/// Whether a binding was newly persisted or safely replayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkDefinitionBindingWriteDisposition {
    /// The binding and all immutable definitions were created.
    Created,
    /// An identical idempotent request returned its existing binding.
    Replayed,
}

/// Successful result from the private definition-binding writer.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkDefinitionBindingWriteResult {
    binding: BenchmarkDefinitionBinding,
    disposition: BenchmarkDefinitionBindingWriteDisposition,
}

impl BenchmarkDefinitionBindingWriteResult {
    pub(crate) const fn new(
        binding: BenchmarkDefinitionBinding,
        disposition: BenchmarkDefinitionBindingWriteDisposition,
    ) -> Self {
        Self {
            binding,
            disposition,
        }
    }

    /// Returns the immutable binding.
    #[must_use]
    pub const fn binding(&self) -> &BenchmarkDefinitionBinding {
        &self.binding
    }

    /// Returns the create/replay disposition.
    #[must_use]
    pub const fn disposition(&self) -> BenchmarkDefinitionBindingWriteDisposition {
        self.disposition
    }
}

/// Private atomic writer for authored benchmark definitions and exact Context binding.
#[async_trait]
pub trait BenchmarkDefinitionBindingWriter: Send + Sync {
    /// Persists or replays one complete immutable definition binding.
    async fn persist_benchmark_definition_binding(
        &self,
        command: BenchmarkDefinitionBindingCommand,
    ) -> Result<BenchmarkDefinitionBindingWriteResult, StorageRepositoryError>;
}

/// Private read port for exact benchmark definition bindings.
#[async_trait]
pub trait BenchmarkDefinitionBindingRepository: Send + Sync {
    /// Reads one binding at its exact project, Context, and commit scope.
    async fn get_benchmark_definition_binding(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
        binding_id: BenchmarkDefinitionBindingId,
    ) -> Result<Option<BenchmarkDefinitionBinding>, StorageRepositoryError>;

    /// Lists bindings at one exact immutable Context commit in stable order.
    async fn list_benchmark_definition_bindings_at_commit(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Result<Vec<BenchmarkDefinitionBinding>, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::{
        BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION, BenchmarkDefinitionBinding,
        BenchmarkDefinitionBindingError,
    };
    use chrono::Utc;
    use contextlab_context_core::{ContextId, ProjectId};
    use contextlab_evaluation::{
        BenchmarkCase, BenchmarkDataset, BenchmarkExpectedOutput, BenchmarkSuite,
    };
    use contextlab_versioning::BranchName;
    use uuid::Uuid;

    #[test]
    fn rehydration_rejects_unknown_schema_versions_before_membership_is_trusted() {
        let dataset_id = contextlab_evaluation::BenchmarkDatasetId::from_uuid(Uuid::from_u128(1));
        let dataset = BenchmarkDataset::with_id(
            dataset_id,
            "cases",
            vec![
                BenchmarkCase::with_id(
                    contextlab_evaluation::BenchmarkCaseId::from_uuid(Uuid::from_u128(2)),
                    "case",
                    serde_json::json!({"input": "value"}),
                    BenchmarkExpectedOutput::Unspecified,
                )
                .expect("case"),
            ],
        )
        .expect("dataset");
        let suite = BenchmarkSuite::with_id(
            contextlab_evaluation::BenchmarkSuiteId::from_uuid(Uuid::from_u128(3)),
            "suite",
            vec![dataset_id],
            vec![
                contextlab_evaluation::RegressionThreshold::new(
                    contextlab_evaluation::MetricKind::Accuracy,
                    contextlab_evaluation::ThresholdDirection::Minimum,
                    0.5,
                )
                .expect("threshold"),
            ],
        )
        .expect("suite");

        let error = BenchmarkDefinitionBinding::new(
            super::BenchmarkDefinitionBindingId::from_uuid(Uuid::from_u128(4)),
            ProjectId::from_uuid(Uuid::from_u128(5)),
            ContextId::from_uuid(Uuid::from_u128(6)),
            contextlab_versioning::CommitId::from_uuid(Uuid::from_u128(7)),
            BranchName::new("main").expect("branch"),
            BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION + 1,
            vec![dataset],
            suite,
            Utc::now(),
        )
        .expect_err("unknown stored schema version must fail closed");

        assert_eq!(
            error,
            BenchmarkDefinitionBindingError::InvalidSchemaVersion {
                expected: BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
                actual: BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION + 1,
            }
        );
    }
}
