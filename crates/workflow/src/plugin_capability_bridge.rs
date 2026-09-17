use crate::{
    WorkflowCapability, WorkflowCapabilityId, WorkflowCapabilityRequirement,
    WorkflowCapabilitySnapshot, WorkflowCapabilitySnapshotError, WorkflowCapabilityVersion,
};
use contextlab_mcp::{
    CapabilityKind, CapabilityRegistry, CapabilityRegistrySnapshotV1, CapabilityResolutionError,
    IdentifierError, Version,
};
use std::collections::BTreeMap;
use thiserror::Error;

/// A Workflow capability requirement paired with its expected plugin capability category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowPluginCapabilityRequirement {
    requirement: WorkflowCapabilityRequirement,
    expected_kind: CapabilityKind,
}

impl WorkflowPluginCapabilityRequirement {
    /// Creates a typed plugin requirement without changing the Workflow requirement contract.
    #[must_use]
    pub const fn new(
        requirement: WorkflowCapabilityRequirement,
        expected_kind: CapabilityKind,
    ) -> Self {
        Self {
            requirement,
            expected_kind,
        }
    }

    /// Returns the existing versioned Workflow requirement.
    #[must_use]
    pub const fn requirement(&self) -> &WorkflowCapabilityRequirement {
        &self.requirement
    }

    /// Returns the exact plugin capability category required by the Workflow adapter.
    #[must_use]
    pub const fn expected_kind(&self) -> CapabilityKind {
        self.expected_kind
    }
}

/// Provider-free adapter from an activated plugin registry to a Workflow capability snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct WorkflowPluginCapabilityBridge;

impl WorkflowPluginCapabilityBridge {
    /// Resolves canonical requirements through the MCP registry and builds the existing snapshot.
    ///
    /// Registry version resolution remains authoritative. The bridge adds exact category checking
    /// and fails closed before a Workflow scheduler can observe an incomplete snapshot.
    pub fn snapshot(
        registry: &CapabilityRegistry,
        requirements: impl IntoIterator<Item = WorkflowPluginCapabilityRequirement>,
    ) -> Result<WorkflowCapabilitySnapshot, WorkflowPluginCapabilityBridgeError> {
        Self::snapshot_from_registry(registry, requirements)
    }

    /// Resolves requirements against one immutable MCP registry snapshot.
    ///
    /// The snapshot is the dependency boundary for a scheduling decision: later registry
    /// registration cannot change the capability set observed by this bridge call.
    pub fn snapshot_from_registry_snapshot(
        registry_snapshot: &CapabilityRegistrySnapshotV1,
        requirements: impl IntoIterator<Item = WorkflowPluginCapabilityRequirement>,
    ) -> Result<WorkflowCapabilitySnapshot, WorkflowPluginCapabilityBridgeError> {
        Self::snapshot_from_registry(registry_snapshot.registry(), requirements)
    }

    fn snapshot_from_registry(
        registry: &CapabilityRegistry,
        requirements: impl IntoIterator<Item = WorkflowPluginCapabilityRequirement>,
    ) -> Result<WorkflowCapabilitySnapshot, WorkflowPluginCapabilityBridgeError> {
        let requirements = canonical_requirements(requirements)?;
        let capabilities = requirements
            .into_iter()
            .map(|(capability, requirement)| {
                let required = requirement.minimum_version;
                let descriptor = registry
                    .resolve_capability(capability.as_str(), to_mcp_version(required))
                    .map_err(|error| map_resolution_error(error, capability.clone(), required))?;

                if descriptor.kind() != requirement.expected_kind {
                    return Err(WorkflowPluginCapabilityBridgeError::WrongCapabilityKind {
                        capability,
                        expected: requirement.expected_kind,
                        provided: descriptor.kind(),
                    });
                }

                WorkflowCapability::new(descriptor.id(), to_workflow_version(descriptor.version()))
                    .map_err(|source| {
                        WorkflowPluginCapabilityBridgeError::InvalidWorkflowCapability {
                            capability: descriptor.id().to_owned(),
                            source,
                        }
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        WorkflowCapabilitySnapshot::new(capabilities).map_err(Into::into)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CanonicalRequirement {
    minimum_version: WorkflowCapabilityVersion,
    expected_kind: CapabilityKind,
}

fn canonical_requirements(
    requirements: impl IntoIterator<Item = WorkflowPluginCapabilityRequirement>,
) -> Result<BTreeMap<WorkflowCapabilityId, CanonicalRequirement>, WorkflowPluginCapabilityBridgeError>
{
    let mut requirements: Vec<_> = requirements.into_iter().collect();
    requirements.sort_by(|left, right| {
        left.requirement
            .capability()
            .cmp(right.requirement.capability())
            .then_with(|| left.expected_kind.cmp(&right.expected_kind))
            .then_with(|| {
                left.requirement
                    .minimum_version()
                    .cmp(&right.requirement.minimum_version())
            })
    });

    let mut canonical = BTreeMap::new();
    for plugin_requirement in requirements {
        let capability = plugin_requirement.requirement.capability().clone();
        let candidate = CanonicalRequirement {
            minimum_version: plugin_requirement.requirement.minimum_version(),
            expected_kind: plugin_requirement.expected_kind,
        };
        let Some(existing) = canonical.get_mut(&capability) else {
            canonical.insert(capability, candidate);
            continue;
        };

        if existing.expected_kind != candidate.expected_kind {
            return Err(
                WorkflowPluginCapabilityBridgeError::ConflictingCapabilityKind {
                    capability,
                    first: existing.expected_kind,
                    second: candidate.expected_kind,
                },
            );
        }
        if existing.minimum_version.major() != candidate.minimum_version.major() {
            return Err(
                WorkflowPluginCapabilityBridgeError::ConflictingCapabilityVersion {
                    capability,
                    first: existing.minimum_version,
                    second: candidate.minimum_version,
                },
            );
        }
        existing.minimum_version = existing.minimum_version.max(candidate.minimum_version);
    }
    Ok(canonical)
}

const fn to_mcp_version(version: WorkflowCapabilityVersion) -> Version {
    Version::new(version.major(), version.minor(), version.patch())
}

const fn to_workflow_version(version: Version) -> WorkflowCapabilityVersion {
    WorkflowCapabilityVersion::new(version.major(), version.minor(), version.patch())
}

fn map_resolution_error(
    error: CapabilityResolutionError,
    capability: WorkflowCapabilityId,
    required: WorkflowCapabilityVersion,
) -> WorkflowPluginCapabilityBridgeError {
    match error {
        CapabilityResolutionError::InvalidIdentifier(source) => {
            WorkflowPluginCapabilityBridgeError::InvalidRegistryCapability { capability, source }
        }
        CapabilityResolutionError::Missing(_) => {
            WorkflowPluginCapabilityBridgeError::MissingCapability {
                capability,
                required,
            }
        }
        CapabilityResolutionError::IncompatibleVersion { provided, .. } => {
            WorkflowPluginCapabilityBridgeError::IncompatibleCapability {
                capability,
                required,
                provided: to_workflow_version(provided),
            }
        }
    }
}

/// Errors raised while deriving a Workflow snapshot from the plugin capability registry.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowPluginCapabilityBridgeError {
    /// A Workflow identifier unexpectedly failed the registry identifier contract.
    #[error("workflow capability {capability} is invalid for the plugin registry: {source}")]
    InvalidRegistryCapability {
        /// The stable Workflow capability identifier.
        capability: WorkflowCapabilityId,
        /// The registry identifier validation error.
        source: IdentifierError,
    },
    /// A resolved registry descriptor unexpectedly failed the Workflow identifier contract.
    #[error("plugin capability {capability} is invalid for Workflow: {source}")]
    InvalidWorkflowCapability {
        /// The rejected plugin capability identifier.
        capability: String,
        /// The Workflow identifier validation error.
        source: crate::WorkflowCapabilityIdentifierError,
    },
    /// No activated registry capability has the required stable identifier.
    #[error("plugin registry is missing workflow capability {capability} at version {required}")]
    MissingCapability {
        /// The missing stable capability identifier.
        capability: WorkflowCapabilityId,
        /// The minimum version required by Workflow.
        required: WorkflowCapabilityVersion,
    },
    /// The registry capability cannot satisfy the Workflow version requirement.
    #[error(
        "plugin capability {capability} version {provided} cannot satisfy workflow version {required}"
    )]
    IncompatibleCapability {
        /// The stable capability identifier.
        capability: WorkflowCapabilityId,
        /// The minimum version required by Workflow.
        required: WorkflowCapabilityVersion,
        /// The incompatible version registered by the plugin.
        provided: WorkflowCapabilityVersion,
    },
    /// The registry capability has a different category from the Workflow adapter contract.
    #[error(
        "plugin capability {capability} has kind {provided:?}, but workflow requires {expected:?}"
    )]
    WrongCapabilityKind {
        /// The stable capability identifier.
        capability: WorkflowCapabilityId,
        /// The category required by the Workflow adapter.
        expected: CapabilityKind,
        /// The category registered by the plugin.
        provided: CapabilityKind,
    },
    /// Duplicate requirements assign different plugin categories to one stable identifier.
    #[error(
        "workflow capability {capability} has conflicting plugin kinds {first:?} and {second:?}"
    )]
    ConflictingCapabilityKind {
        /// The conflicting stable capability identifier.
        capability: WorkflowCapabilityId,
        /// The first category in canonical order.
        first: CapabilityKind,
        /// The second category in canonical order.
        second: CapabilityKind,
    },
    /// Duplicate requirements use incompatible major-version families for one stable identifier.
    #[error(
        "workflow capability {capability} has conflicting required versions {first} and {second}"
    )]
    ConflictingCapabilityVersion {
        /// The conflicting stable capability identifier.
        capability: WorkflowCapabilityId,
        /// The first required version in canonical order.
        first: WorkflowCapabilityVersion,
        /// The second required version in canonical order.
        second: WorkflowCapabilityVersion,
    },
    /// Canonical capability conversion could not produce the existing Workflow snapshot.
    #[error(transparent)]
    Snapshot(#[from] WorkflowCapabilitySnapshotError),
}
