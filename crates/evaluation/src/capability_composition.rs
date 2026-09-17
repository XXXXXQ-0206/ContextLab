//! Provider-free capability facts consumed by sealed benchmark decisions.

use crate::{BenchmarkExecutionCohortId, BenchmarkExecutionReceipt, RegressionDecisionStatus};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The only capability snapshot schema currently accepted by evaluation consumers.
pub const BENCHMARK_CAPABILITY_SNAPSHOT_SCHEMA_VERSION: u16 = 1;

/// Provider-free capability categories relevant to benchmark execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCapabilityKindV1 {
    /// A callable evaluation implementation.
    Evaluator,
    /// A callable tool available to an evaluation.
    Tool,
    /// A readable resource available to an evaluation.
    Resource,
    /// A model adapter, represented without provider configuration.
    Model,
    /// A storage adapter.
    Storage,
    /// An authentication adapter.
    Authentication,
    /// An MCP server.
    McpServer,
    /// An importer.
    Importer,
    /// An exporter.
    Exporter,
    /// A renderer.
    Renderer,
}

/// One validated, secret-free capability fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCapabilityFactV1 {
    capability_id: String,
    kind: BenchmarkCapabilityKindV1,
    major: u64,
    minor: u64,
    patch: u64,
}

impl BenchmarkCapabilityFactV1 {
    /// Creates one capability fact with a stable identifier and semantic version.
    pub fn new(
        capability_id: impl Into<String>,
        kind: BenchmarkCapabilityKindV1,
        major: u64,
        minor: u64,
        patch: u64,
    ) -> Result<Self, BenchmarkCapabilitySnapshotError> {
        let capability_id = capability_id.into();
        if capability_id.trim().is_empty() || capability_id != capability_id.trim() {
            return Err(BenchmarkCapabilitySnapshotError::InvalidCapabilityId);
        }
        Ok(Self {
            capability_id,
            kind,
            major,
            minor,
            patch,
        })
    }

    /// Returns the stable capability identifier.
    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    /// Returns the capability category.
    #[must_use]
    pub const fn kind(&self) -> BenchmarkCapabilityKindV1 {
        self.kind
    }

    /// Returns the semantic version tuple.
    #[must_use]
    pub const fn version(&self) -> (u64, u64, u64) {
        (self.major, self.minor, self.patch)
    }
}

/// An immutable, canonical, provider-free capability snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCapabilitySnapshotV1 {
    schema_version: u16,
    capabilities: Vec<BenchmarkCapabilityFactV1>,
}

impl BenchmarkCapabilitySnapshotV1 {
    /// Creates a canonical V1 snapshot and rejects duplicate capability identifiers.
    pub fn new(
        capabilities: impl IntoIterator<Item = BenchmarkCapabilityFactV1>,
    ) -> Result<Self, BenchmarkCapabilitySnapshotError> {
        let mut capabilities: Vec<_> = capabilities.into_iter().collect();
        capabilities.sort_by(|left, right| {
            left.capability_id
                .cmp(&right.capability_id)
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.version().cmp(&right.version()))
        });
        if let Some(pair) = capabilities
            .windows(2)
            .find(|pair| pair[0].capability_id == pair[1].capability_id)
        {
            return Err(BenchmarkCapabilitySnapshotError::DuplicateCapability {
                capability_id: pair[0].capability_id.clone(),
            });
        }
        Ok(Self {
            schema_version: BENCHMARK_CAPABILITY_SNAPSHOT_SCHEMA_VERSION,
            capabilities,
        })
    }

    /// Converts the shared MCP registry snapshot into the evaluation projection.
    pub fn from_mcp_snapshot(
        snapshot: &contextlab_mcp::CapabilityRegistrySnapshotV1,
    ) -> Result<Self, BenchmarkCapabilitySnapshotError> {
        if snapshot.schema_version() != contextlab_mcp::CAPABILITY_REGISTRY_SNAPSHOT_V1 {
            return Err(BenchmarkCapabilitySnapshotError::UnsupportedSourceSchema);
        }
        Self::new(snapshot.registry().capabilities().map(|capability| {
            BenchmarkCapabilityFactV1::new(
                capability.id(),
                capability_kind(capability.kind()),
                capability.version().major(),
                capability.version().minor(),
                capability.version().patch(),
            )
            .expect("validated MCP capability identifiers must remain valid")
        }))
    }

    /// Returns the explicit snapshot schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns capabilities in deterministic identifier order.
    #[must_use]
    pub fn capabilities(&self) -> &[BenchmarkCapabilityFactV1] {
        &self.capabilities
    }

    fn find(&self, capability_id: &str) -> Option<&BenchmarkCapabilityFactV1> {
        self.capabilities
            .binary_search_by_key(&capability_id, |fact| fact.capability_id.as_str())
            .ok()
            .map(|index| &self.capabilities[index])
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BenchmarkCapabilitySnapshotWireV1 {
    schema_version: u16,
    capabilities: Vec<BenchmarkCapabilityFactV1>,
}

impl<'de> Deserialize<'de> for BenchmarkCapabilitySnapshotV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = BenchmarkCapabilitySnapshotWireV1::deserialize(deserializer)?;
        if wire.schema_version != BENCHMARK_CAPABILITY_SNAPSHOT_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(
                BenchmarkCapabilitySnapshotError::UnsupportedSchemaVersion {
                    found: wire.schema_version,
                    supported: BENCHMARK_CAPABILITY_SNAPSHOT_SCHEMA_VERSION,
                },
            ));
        }
        Self::new(wire.capabilities).map_err(serde::de::Error::custom)
    }
}

/// Errors that prevent a capability snapshot from entering evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BenchmarkCapabilitySnapshotError {
    /// The capability identifier is blank or padded.
    #[error("capability identifier must be non-empty and trimmed")]
    InvalidCapabilityId,
    /// A capability identifier appears more than once.
    #[error("capability {capability_id} is declared more than once")]
    DuplicateCapability {
        /// The duplicated stable identifier.
        capability_id: String,
    },
    /// The serialized snapshot uses an unsupported schema.
    #[error("unsupported benchmark capability snapshot schema {found}; supported {supported}")]
    UnsupportedSchemaVersion {
        /// The received schema version.
        found: u16,
        /// The only supported schema version.
        supported: u16,
    },
    /// The shared source snapshot has an unexpected schema.
    #[error("shared capability snapshot schema is unsupported")]
    UnsupportedSourceSchema,
}

/// Errors that prevent a sealed benchmark decision from consuming a capability.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BenchmarkCapabilityCompositionError {
    /// The capability snapshot failed strict validation.
    #[error(transparent)]
    Snapshot(#[from] BenchmarkCapabilitySnapshotError),
    /// The requested capability is absent from the immutable snapshot.
    #[error("required benchmark capability is missing")]
    MissingCapability,
    /// The registered capability category does not match the evaluation requirement.
    #[error("benchmark capability kind does not satisfy the evaluation requirement")]
    KindMismatch {
        /// The kind found in the snapshot.
        found: BenchmarkCapabilityKindV1,
        /// The kind required by the evaluation.
        required: BenchmarkCapabilityKindV1,
    },
    /// The registered capability major version is incompatible.
    #[error("benchmark capability major version does not satisfy the evaluation requirement")]
    MajorVersionMismatch {
        /// The found semantic version.
        found: (u64, u64, u64),
        /// The required semantic version.
        required: (u64, u64, u64),
    },
    /// The registered capability is older than the minimum required version.
    #[error("benchmark capability version is below the evaluation requirement")]
    VersionTooOld {
        /// The found semantic version.
        found: (u64, u64, u64),
        /// The required semantic version.
        required: (u64, u64, u64),
    },
}

/// A redacted evaluation projection bound to one sealed receipt and one capability snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkEvaluationCapabilityCompositionV1 {
    schema_version: u16,
    cohort_id: BenchmarkExecutionCohortId,
    decision_status: RegressionDecisionStatus,
    capability_id: String,
    capability_kind: BenchmarkCapabilityKindV1,
    capability_major: u64,
    capability_minor: u64,
    capability_patch: u64,
}

impl BenchmarkEvaluationCapabilityCompositionV1 {
    /// Binds a sealed evaluation decision to a compatible provider-free capability fact.
    pub fn from_receipt(
        receipt: &BenchmarkExecutionReceipt,
        snapshot: &BenchmarkCapabilitySnapshotV1,
        capability_id: &str,
        required_kind: BenchmarkCapabilityKindV1,
        required_version: (u64, u64, u64),
    ) -> Result<Self, BenchmarkCapabilityCompositionError> {
        let capability = snapshot
            .find(capability_id)
            .ok_or(BenchmarkCapabilityCompositionError::MissingCapability)?;
        if capability.kind != required_kind {
            return Err(BenchmarkCapabilityCompositionError::KindMismatch {
                found: capability.kind,
                required: required_kind,
            });
        }
        let version = capability.version();
        if version.0 != required_version.0 {
            return Err(BenchmarkCapabilityCompositionError::MajorVersionMismatch {
                found: version,
                required: required_version,
            });
        }
        if version < required_version {
            return Err(BenchmarkCapabilityCompositionError::VersionTooOld {
                found: version,
                required: required_version,
            });
        }
        Ok(Self {
            schema_version: BENCHMARK_CAPABILITY_SNAPSHOT_SCHEMA_VERSION,
            cohort_id: receipt.cohort_id(),
            decision_status: receipt.evaluation().decision().status(),
            capability_id: capability.capability_id.clone(),
            capability_kind: capability.kind,
            capability_major: version.0,
            capability_minor: version.1,
            capability_patch: version.2,
        })
    }

    /// Returns the projection schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the sealed execution cohort identity.
    #[must_use]
    pub const fn cohort_id(&self) -> BenchmarkExecutionCohortId {
        self.cohort_id
    }

    /// Returns the server-owned decision status.
    #[must_use]
    pub const fn decision_status(&self) -> RegressionDecisionStatus {
        self.decision_status
    }

    /// Returns the consumed capability identifier.
    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    /// Returns the consumed capability version.
    #[must_use]
    pub const fn capability_version(&self) -> (u64, u64, u64) {
        (
            self.capability_major,
            self.capability_minor,
            self.capability_patch,
        )
    }
}

fn capability_kind(kind: contextlab_mcp::CapabilityKind) -> BenchmarkCapabilityKindV1 {
    match kind {
        contextlab_mcp::CapabilityKind::Tool => BenchmarkCapabilityKindV1::Tool,
        contextlab_mcp::CapabilityKind::Resource => BenchmarkCapabilityKindV1::Resource,
        contextlab_mcp::CapabilityKind::Model => BenchmarkCapabilityKindV1::Model,
        contextlab_mcp::CapabilityKind::Evaluator => BenchmarkCapabilityKindV1::Evaluator,
        contextlab_mcp::CapabilityKind::Storage => BenchmarkCapabilityKindV1::Storage,
        contextlab_mcp::CapabilityKind::Authentication => BenchmarkCapabilityKindV1::Authentication,
        contextlab_mcp::CapabilityKind::McpServer => BenchmarkCapabilityKindV1::McpServer,
        contextlab_mcp::CapabilityKind::Importer => BenchmarkCapabilityKindV1::Importer,
        contextlab_mcp::CapabilityKind::Exporter => BenchmarkCapabilityKindV1::Exporter,
        contextlab_mcp::CapabilityKind::Renderer => BenchmarkCapabilityKindV1::Renderer,
    }
}
