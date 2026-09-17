//! Provider-free workflow domain and scheduling primitives.

mod context_binding;
mod execution;
mod execution_status;
mod plugin_capability_bridge;
mod projection;

pub use context_binding::{ContextCommitSource, WorkflowContextBinding, WorkflowContextBindingId};
pub use execution::{
    WorkflowExecution, WorkflowExecutionEventV1, WorkflowExecutionLogV1,
    WorkflowExecutionReplayError, WorkflowExecutionSchemaVersion, WorkflowExecutionStartError,
};
pub use execution_status::{
    WorkflowExecutionStatusProjectionError, WorkflowExecutionStatusProjectionSchemaVersion,
    WorkflowExecutionStatusProjectionV1, WorkflowNodeStatusCountsV1,
};
pub use plugin_capability_bridge::{
    WorkflowPluginCapabilityBridge, WorkflowPluginCapabilityBridgeError,
    WorkflowPluginCapabilityRequirement,
};
pub use projection::{
    WorkflowCapabilityStatusProjectionV1, WorkflowCapabilityStatusV1,
    WorkflowStatusProjectionEdgeV1, WorkflowStatusProjectionError,
    WorkflowStatusProjectionFailureV1, WorkflowStatusProjectionNodeStateV1,
    WorkflowStatusProjectionNodeV1, WorkflowStatusProjectionSchemaVersion,
    WorkflowStatusProjectionV1,
};

use contextlab_context_core::ComponentContent;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    str::FromStr,
};
use thiserror::Error;
use uuid::Uuid;

macro_rules! workflow_uuid_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(Uuid);

        impl $name {
            /// Creates an identifier from a UUID.
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
    };
}

workflow_uuid_id!(WorkflowId, "Stable workflow identifier.");
workflow_uuid_id!(WorkflowNodeId, "Stable workflow node identifier.");
workflow_uuid_id!(WorkflowEdgeId, "Stable workflow edge identifier.");
workflow_uuid_id!(WorkflowRunId, "Stable workflow execution identifier.");

/// A positive revision number for an immutable workflow contract member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct WorkflowRevision(u64);

impl<'de> Deserialize<'de> for WorkflowRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl WorkflowRevision {
    /// Creates a positive workflow revision.
    pub fn new(value: u64) -> Result<Self, WorkflowValidationError> {
        if value == 0 {
            return Err(WorkflowValidationError::ZeroRevision);
        }
        Ok(Self(value))
    }

    /// Returns the revision number.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A strict three-part version for a workflow capability contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkflowCapabilityVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl WorkflowCapabilityVersion {
    /// Creates a workflow capability contract version.
    #[must_use]
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns the major version component.
    #[must_use]
    pub const fn major(self) -> u64 {
        self.major
    }

    /// Returns the minor version component.
    #[must_use]
    pub const fn minor(self) -> u64 {
        self.minor
    }

    /// Returns the patch version component.
    #[must_use]
    pub const fn patch(self) -> u64 {
        self.patch
    }

    fn satisfies(self, required: Self) -> bool {
        self.major == required.major && self >= required
    }
}

impl fmt::Display for WorkflowCapabilityVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for WorkflowCapabilityVersion {
    type Err = WorkflowCapabilityVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut components = value.split('.');
        let major = components
            .next()
            .ok_or(WorkflowCapabilityVersionParseError::InvalidFormat)?;
        let minor = components
            .next()
            .ok_or(WorkflowCapabilityVersionParseError::InvalidFormat)?;
        let patch = components
            .next()
            .ok_or(WorkflowCapabilityVersionParseError::InvalidFormat)?;
        if components.next().is_some() {
            return Err(WorkflowCapabilityVersionParseError::InvalidFormat);
        }

        fn parse_component(value: &str) -> Result<u64, WorkflowCapabilityVersionParseError> {
            if value.is_empty() || (value.len() > 1 && value.starts_with('0')) {
                return Err(WorkflowCapabilityVersionParseError::InvalidComponent(
                    value.to_owned(),
                ));
            }
            value.parse().map_err(|_| {
                WorkflowCapabilityVersionParseError::InvalidComponent(value.to_owned())
            })
        }

        Ok(Self::new(
            parse_component(major)?,
            parse_component(minor)?,
            parse_component(patch)?,
        ))
    }
}

impl Serialize for WorkflowCapabilityVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkflowCapabilityVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// A stable, provider-neutral workflow capability identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkflowCapabilityId(String);

impl WorkflowCapabilityId {
    /// Creates a stable capability identifier from ASCII letters, digits, `.`, `-`, `_`, or `:`.
    pub fn new(value: impl AsRef<str>) -> Result<Self, WorkflowCapabilityIdentifierError> {
        let value = value.as_ref();
        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return Err(WorkflowCapabilityIdentifierError::Empty);
        }
        if bytes
            .first()
            .is_some_and(|byte| !byte.is_ascii_alphanumeric())
            || bytes
                .last()
                .is_some_and(|byte| !byte.is_ascii_alphanumeric())
            || bytes.iter().any(|byte| {
                !byte.is_ascii_alphanumeric() && !matches!(byte, b'.' | b'-' | b'_' | b':')
            })
        {
            return Err(WorkflowCapabilityIdentifierError::Invalid {
                value: value.to_owned(),
            });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the stable identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkflowCapabilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for WorkflowCapabilityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for WorkflowCapabilityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// A versioned capability requirement declared by one workflow node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowCapabilityRequirement {
    capability: WorkflowCapabilityId,
    minimum_version: WorkflowCapabilityVersion,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowCapabilityRequirementWire {
    capability: WorkflowCapabilityId,
    minimum_version: WorkflowCapabilityVersion,
}

impl<'de> Deserialize<'de> for WorkflowCapabilityRequirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WorkflowCapabilityRequirementWire::deserialize(deserializer)?;
        Ok(Self {
            capability: wire.capability,
            minimum_version: wire.minimum_version,
        })
    }
}

impl WorkflowCapabilityRequirement {
    /// Creates a requirement satisfied by the same major version at or above `minimum_version`.
    pub fn new(
        capability: impl AsRef<str>,
        minimum_version: WorkflowCapabilityVersion,
    ) -> Result<Self, WorkflowCapabilityIdentifierError> {
        Ok(Self {
            capability: WorkflowCapabilityId::new(capability)?,
            minimum_version,
        })
    }

    /// Returns the required capability identifier.
    #[must_use]
    pub const fn capability(&self) -> &WorkflowCapabilityId {
        &self.capability
    }

    /// Returns the minimum compatible capability contract version.
    #[must_use]
    pub const fn minimum_version(&self) -> WorkflowCapabilityVersion {
        self.minimum_version
    }
}

/// A versioned capability available to the workflow scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowCapability {
    capability: WorkflowCapabilityId,
    version: WorkflowCapabilityVersion,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowCapabilityWire {
    capability: WorkflowCapabilityId,
    version: WorkflowCapabilityVersion,
}

impl<'de> Deserialize<'de> for WorkflowCapability {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WorkflowCapabilityWire::deserialize(deserializer)?;
        Ok(Self {
            capability: wire.capability,
            version: wire.version,
        })
    }
}

impl WorkflowCapability {
    /// Creates a versioned capability available in a snapshot.
    pub fn new(
        capability: impl AsRef<str>,
        version: WorkflowCapabilityVersion,
    ) -> Result<Self, WorkflowCapabilityIdentifierError> {
        Ok(Self {
            capability: WorkflowCapabilityId::new(capability)?,
            version,
        })
    }

    /// Returns the stable capability identifier.
    #[must_use]
    pub const fn capability(&self) -> &WorkflowCapabilityId {
        &self.capability
    }

    /// Returns the available capability contract version.
    #[must_use]
    pub const fn version(&self) -> WorkflowCapabilityVersion {
        self.version
    }
}

/// A deterministic, injected view of capabilities available for one scheduling decision.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowCapabilitySnapshot {
    capabilities: BTreeMap<WorkflowCapabilityId, WorkflowCapabilityVersion>,
}

/// The explicit schema version for the canonical capability snapshot sealed into an execution log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowCapabilitySnapshotSchemaVersion {
    /// The first stable canonical capability snapshot schema.
    V1,
}

/// One provider-free capability entry in a canonical V1 snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowCapabilitySnapshotEntryV1 {
    capability: WorkflowCapabilityId,
    version: WorkflowCapabilityVersion,
}

impl WorkflowCapabilitySnapshotEntryV1 {
    /// Returns the stable capability identifier.
    #[must_use]
    pub const fn capability(&self) -> &WorkflowCapabilityId {
        &self.capability
    }

    /// Returns the exact capability contract version.
    #[must_use]
    pub const fn version(&self) -> WorkflowCapabilityVersion {
        self.version
    }
}

/// The immutable, versioned canonical representation sealed into a workflow execution log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowCapabilitySnapshotV1 {
    schema_version: WorkflowCapabilitySnapshotSchemaVersion,
    capabilities: Vec<WorkflowCapabilitySnapshotEntryV1>,
}

impl WorkflowCapabilitySnapshotV1 {
    /// Returns the explicit snapshot schema version.
    #[must_use]
    pub const fn schema_version(&self) -> WorkflowCapabilitySnapshotSchemaVersion {
        self.schema_version
    }

    /// Returns capability entries in their sealed order.
    #[must_use]
    pub fn capabilities(&self) -> &[WorkflowCapabilitySnapshotEntryV1] {
        &self.capabilities
    }

    /// Returns the deterministic digest of this exact ordered representation.
    #[must_use]
    pub fn canonical_digest(&self) -> String {
        ComponentContent::new(self.canonical_representation())
            .content_hash()
            .as_str()
            .to_owned()
    }

    fn canonical_representation(&self) -> String {
        let mut representation = String::from("workflow-capability-snapshot:v1\n");
        for capability in &self.capabilities {
            representation.push_str(capability.capability.as_str());
            representation.push('\t');
            representation.push_str(&capability.version.to_string());
            representation.push('\n');
        }
        representation
    }
}

impl WorkflowCapabilitySnapshot {
    /// Creates a snapshot and rejects duplicate capability identifiers.
    pub fn new(
        capabilities: impl IntoIterator<Item = WorkflowCapability>,
    ) -> Result<Self, WorkflowCapabilitySnapshotError> {
        let mut values = BTreeMap::new();
        for capability in capabilities {
            let capability_id = capability.capability;
            let version = capability.version;
            if values.insert(capability_id.clone(), version).is_some() {
                return Err(WorkflowCapabilitySnapshotError::DuplicateCapability {
                    capability: capability_id,
                });
            }
        }
        Ok(Self {
            capabilities: values,
        })
    }

    /// Returns the available version for one capability, if present.
    #[must_use]
    pub fn version(&self, capability: &WorkflowCapabilityId) -> Option<WorkflowCapabilityVersion> {
        self.capabilities.get(capability).copied()
    }

    /// Returns the canonical V1 representation used for execution provenance.
    #[must_use]
    pub fn canonical_v1(&self) -> WorkflowCapabilitySnapshotV1 {
        WorkflowCapabilitySnapshotV1 {
            schema_version: WorkflowCapabilitySnapshotSchemaVersion::V1,
            capabilities: self
                .capabilities
                .iter()
                .map(|(capability, version)| WorkflowCapabilitySnapshotEntryV1 {
                    capability: capability.clone(),
                    version: *version,
                })
                .collect(),
        }
    }

    /// Returns the deterministic identity of this provider-free snapshot.
    #[must_use]
    pub fn canonical_digest(&self) -> String {
        self.canonical_v1().canonical_digest()
    }
}

/// An immutable node in a versioned workflow definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowNode {
    id: WorkflowNodeId,
    revision: WorkflowRevision,
    capability_requirements: Vec<WorkflowCapabilityRequirement>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowNodeWire {
    id: WorkflowNodeId,
    revision: WorkflowRevision,
    capability_requirements: Vec<WorkflowCapabilityRequirement>,
}

impl<'de> Deserialize<'de> for WorkflowNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WorkflowNodeWire::deserialize(deserializer)?;
        Self::with_capability_requirements(wire.id, wire.revision, wire.capability_requirements)
            .map_err(serde::de::Error::custom)
    }
}

impl WorkflowNode {
    /// Creates a workflow node with no external capability requirements.
    #[must_use]
    pub const fn new(id: WorkflowNodeId, revision: WorkflowRevision) -> Self {
        Self {
            id,
            revision,
            capability_requirements: Vec::new(),
        }
    }

    /// Creates a workflow node with canonical, versioned capability requirements.
    pub fn with_capability_requirements(
        id: WorkflowNodeId,
        revision: WorkflowRevision,
        mut capability_requirements: Vec<WorkflowCapabilityRequirement>,
    ) -> Result<Self, WorkflowNodeValidationError> {
        capability_requirements.sort_by(|left, right| {
            left.capability()
                .cmp(right.capability())
                .then_with(|| left.minimum_version().cmp(&right.minimum_version()))
        });
        for requirements in capability_requirements.windows(2) {
            if requirements[0].capability() == requirements[1].capability() {
                return Err(
                    WorkflowNodeValidationError::DuplicateCapabilityRequirement {
                        capability: requirements[0].capability().clone(),
                    },
                );
            }
        }
        Ok(Self {
            id,
            revision,
            capability_requirements,
        })
    }

    /// Returns the stable node identifier.
    #[must_use]
    pub const fn id(&self) -> WorkflowNodeId {
        self.id
    }

    /// Returns the immutable node revision.
    #[must_use]
    pub const fn revision(&self) -> WorkflowRevision {
        self.revision
    }

    /// Returns versioned requirements in ascending capability identifier order.
    #[must_use]
    pub fn capability_requirements(&self) -> &[WorkflowCapabilityRequirement] {
        &self.capability_requirements
    }
}

/// A directed dependency between two workflow nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowEdge {
    id: WorkflowEdgeId,
    revision: WorkflowRevision,
    source: WorkflowNodeId,
    target: WorkflowNodeId,
}

#[derive(Deserialize)]
struct WorkflowEdgeWire {
    id: WorkflowEdgeId,
    revision: WorkflowRevision,
    source: WorkflowNodeId,
    target: WorkflowNodeId,
}

impl<'de> Deserialize<'de> for WorkflowEdge {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WorkflowEdgeWire::deserialize(deserializer)?;
        Self::new(wire.id, wire.revision, wire.source, wire.target)
            .map_err(serde::de::Error::custom)
    }
}

impl WorkflowEdge {
    /// Creates a dependency from `source` to `target`.
    pub fn new(
        id: WorkflowEdgeId,
        revision: WorkflowRevision,
        source: WorkflowNodeId,
        target: WorkflowNodeId,
    ) -> Result<Self, WorkflowValidationError> {
        if source == target {
            return Err(WorkflowValidationError::SelfDependency {
                edge_id: id,
                node_id: source,
            });
        }
        Ok(Self {
            id,
            revision,
            source,
            target,
        })
    }

    /// Returns the stable edge identifier.
    #[must_use]
    pub const fn id(&self) -> WorkflowEdgeId {
        self.id
    }

    /// Returns the immutable edge revision.
    #[must_use]
    pub const fn revision(&self) -> WorkflowRevision {
        self.revision
    }

    /// Returns the prerequisite node.
    #[must_use]
    pub const fn source(&self) -> WorkflowNodeId {
        self.source
    }

    /// Returns the dependent node.
    #[must_use]
    pub const fn target(&self) -> WorkflowNodeId {
        self.target
    }
}

/// A sealed, versioned directed workflow graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowDefinition {
    id: WorkflowId,
    revision: WorkflowRevision,
    nodes: Vec<WorkflowNode>,
    edges: Vec<WorkflowEdge>,
}

#[derive(Deserialize)]
struct WorkflowDefinitionWire {
    id: WorkflowId,
    revision: WorkflowRevision,
    nodes: Vec<WorkflowNode>,
    edges: Vec<WorkflowEdge>,
}

impl<'de> Deserialize<'de> for WorkflowDefinition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WorkflowDefinitionWire::deserialize(deserializer)?;
        Self::new(wire.id, wire.revision, wire.nodes, wire.edges).map_err(serde::de::Error::custom)
    }
}

impl WorkflowDefinition {
    /// Creates a workflow definition with deterministic member ordering.
    pub fn new(
        id: WorkflowId,
        revision: WorkflowRevision,
        mut nodes: Vec<WorkflowNode>,
        mut edges: Vec<WorkflowEdge>,
    ) -> Result<Self, WorkflowValidationError> {
        nodes.sort_by_key(WorkflowNode::id);
        edges.sort_by_key(WorkflowEdge::id);
        let mut node_ids = BTreeSet::new();
        for node in &nodes {
            if !node_ids.insert(node.id()) {
                return Err(WorkflowValidationError::DuplicateNode { node_id: node.id() });
            }
        }
        let mut edge_ids = BTreeSet::new();
        for edge in &edges {
            if !edge_ids.insert(edge.id()) {
                return Err(WorkflowValidationError::DuplicateEdge { edge_id: edge.id() });
            }
            if !node_ids.contains(&edge.source()) {
                return Err(WorkflowValidationError::MissingSourceNode {
                    edge_id: edge.id(),
                    node_id: edge.source(),
                });
            }
            if !node_ids.contains(&edge.target()) {
                return Err(WorkflowValidationError::MissingTargetNode {
                    edge_id: edge.id(),
                    node_id: edge.target(),
                });
            }
        }
        let mut inbound_counts = node_ids
            .iter()
            .copied()
            .map(|node_id| (node_id, 0_usize))
            .collect::<BTreeMap<_, _>>();
        let mut dependents = BTreeMap::<WorkflowNodeId, BTreeSet<WorkflowNodeId>>::new();
        let mut dependencies = BTreeSet::new();
        for edge in &edges {
            if !dependencies.insert((edge.source(), edge.target())) {
                return Err(WorkflowValidationError::DuplicateDependency {
                    source_node: edge.source(),
                    target: edge.target(),
                });
            }
            *inbound_counts.get_mut(&edge.target()).ok_or(
                WorkflowValidationError::MissingTargetNode {
                    edge_id: edge.id(),
                    node_id: edge.target(),
                },
            )? += 1;
            dependents
                .entry(edge.source())
                .or_default()
                .insert(edge.target());
        }
        let mut ready = inbound_counts
            .iter()
            .filter_map(|(node_id, count)| (*count == 0).then_some(*node_id))
            .collect::<BTreeSet<_>>();
        let mut visited = 0_usize;
        while let Some(node_id) = ready.pop_first() {
            visited += 1;
            if let Some(targets) = dependents.get(&node_id) {
                for target in targets {
                    let count = inbound_counts
                        .get_mut(target)
                        .ok_or(WorkflowValidationError::CycleDetected)?;
                    *count -= 1;
                    if *count == 0 {
                        ready.insert(*target);
                    }
                }
            }
        }
        if visited != nodes.len() {
            return Err(WorkflowValidationError::CycleDetected);
        }
        Ok(Self {
            id,
            revision,
            nodes,
            edges,
        })
    }

    /// Returns the stable workflow identifier.
    #[must_use]
    pub const fn id(&self) -> WorkflowId {
        self.id
    }

    /// Returns the immutable workflow revision.
    #[must_use]
    pub const fn revision(&self) -> WorkflowRevision {
        self.revision
    }

    /// Returns nodes in ascending UUID order.
    #[must_use]
    pub fn nodes(&self) -> &[WorkflowNode] {
        &self.nodes
    }

    /// Returns edges in ascending UUID order.
    #[must_use]
    pub fn edges(&self) -> &[WorkflowEdge] {
        &self.edges
    }

    /// Validates every node capability requirement against an injected snapshot.
    ///
    /// Nodes and requirements are traversed in canonical identifier order, so the first returned
    /// error is stable for the same immutable definition and snapshot.
    pub fn validate_capabilities(
        &self,
        snapshot: &WorkflowCapabilitySnapshot,
    ) -> Result<(), WorkflowCapabilityValidationError> {
        for node in &self.nodes {
            for requirement in node.capability_requirements() {
                let Some(provided) = snapshot.version(requirement.capability()) else {
                    return Err(WorkflowCapabilityValidationError::MissingCapability {
                        node_id: node.id(),
                        capability: requirement.capability().clone(),
                        required: requirement.minimum_version(),
                    });
                };
                if !provided.satisfies(requirement.minimum_version()) {
                    return Err(WorkflowCapabilityValidationError::IncompatibleCapability {
                        node_id: node.id(),
                        capability: requirement.capability().clone(),
                        required: requirement.minimum_version(),
                        provided,
                    });
                }
            }
        }
        Ok(())
    }
}

/// A terminal or active workflow execution state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunState {
    /// The workflow has not claimed a node.
    Pending,
    /// A workflow node has been claimed or completed while work remains.
    Running,
    /// Every workflow node completed successfully.
    Succeeded,
    /// A workflow node failed.
    Failed,
}

/// The current state of one workflow node execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum WorkflowNodeState {
    /// The node is waiting for all prerequisites.
    Pending,
    /// The node has been exclusively claimed by the scheduler.
    Running {
        /// The deterministic attempt number.
        attempt: u32,
    },
    /// The node completed successfully.
    Succeeded {
        /// The attempt that completed successfully.
        attempt: u32,
    },
    /// The node reported a typed failure.
    Failed {
        /// The attempt that failed.
        attempt: u32,
        /// The typed failure reported by the executor.
        failure: WorkflowFailure,
    },
    /// The node did not start because another node failed.
    Blocked,
}

/// A validated, provider-neutral failure code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct WorkflowFailureCode(String);

impl<'de> Deserialize<'de> for WorkflowFailureCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl WorkflowFailureCode {
    /// Creates a non-empty, normalized failure code.
    pub fn new(value: impl Into<String>) -> Result<Self, WorkflowFailureValidationError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(WorkflowFailureValidationError::EmptyCode);
        }
        Ok(Self(value))
    }

    /// Returns the normalized failure code.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A typed failure reported by provider-independent workflow execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowFailure {
    code: WorkflowFailureCode,
}

impl WorkflowFailure {
    /// Creates a workflow failure from a validated code.
    pub fn new(value: impl Into<String>) -> Result<Self, WorkflowFailureValidationError> {
        Ok(Self {
            code: WorkflowFailureCode::new(value)?,
        })
    }

    /// Returns the typed failure code.
    #[must_use]
    pub const fn code(&self) -> &WorkflowFailureCode {
        &self.code
    }
}

/// A deterministic claim for one ready workflow node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkflowNodeClaim {
    node_id: WorkflowNodeId,
    attempt: u32,
}

impl WorkflowNodeClaim {
    /// Returns the exclusively claimed node identifier.
    #[must_use]
    pub const fn node_id(self) -> WorkflowNodeId {
        self.node_id
    }

    /// Returns the deterministic attempt number.
    #[must_use]
    pub const fn attempt(self) -> u32 {
        self.attempt
    }
}

/// A serial, provider-free scheduler for one immutable workflow definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowScheduler {
    definition: WorkflowDefinition,
    capability_snapshot: WorkflowCapabilitySnapshot,
    run_id: WorkflowRunId,
    replay_of: Option<WorkflowRunId>,
    run_state: WorkflowRunState,
    node_states: BTreeMap<WorkflowNodeId, WorkflowNodeState>,
}

impl WorkflowScheduler {
    /// Validates an injected capability snapshot before starting a fresh workflow execution.
    pub fn new(
        definition: WorkflowDefinition,
        run_id: WorkflowRunId,
        snapshot: &WorkflowCapabilitySnapshot,
    ) -> Result<Self, WorkflowCapabilityValidationError> {
        definition.validate_capabilities(snapshot)?;
        Ok(Self::new_validated(definition, run_id, snapshot.clone()))
    }

    fn new_validated(
        definition: WorkflowDefinition,
        run_id: WorkflowRunId,
        capability_snapshot: WorkflowCapabilitySnapshot,
    ) -> Self {
        let node_states = definition
            .nodes()
            .iter()
            .map(|node| (node.id(), WorkflowNodeState::Pending))
            .collect();
        Self {
            definition,
            capability_snapshot,
            run_id,
            replay_of: None,
            run_state: WorkflowRunState::Pending,
            node_states,
        }
    }

    /// Returns the immutable definition scheduled by this execution.
    #[must_use]
    pub const fn definition(&self) -> &WorkflowDefinition {
        &self.definition
    }

    /// Returns the stable run identifier.
    #[must_use]
    pub const fn run_id(&self) -> WorkflowRunId {
        self.run_id
    }

    /// Returns the terminal execution that this run replays, if any.
    #[must_use]
    pub const fn replay_of(&self) -> Option<WorkflowRunId> {
        self.replay_of
    }

    /// Replays a terminal execution with a distinct run identifier.
    pub fn replay(&self, run_id: WorkflowRunId) -> Result<Self, WorkflowReplayError> {
        if !matches!(
            self.run_state,
            WorkflowRunState::Succeeded | WorkflowRunState::Failed
        ) {
            return Err(WorkflowReplayError::SourceRunNotTerminal {
                state: self.run_state,
            });
        }
        if self.run_id == run_id {
            return Err(WorkflowReplayError::ReusedRunId { run_id });
        }
        let mut replay = Self::new_validated(
            self.definition.clone(),
            run_id,
            self.capability_snapshot.clone(),
        );
        replay.replay_of = Some(self.run_id);
        Ok(replay)
    }

    /// Builds a redacted, provider-free V1 status projection without executing or mutating work.
    pub fn status_projection(
        &self,
    ) -> Result<WorkflowStatusProjectionV1, WorkflowStatusProjectionError> {
        WorkflowStatusProjectionV1::from_scheduler(self)
    }

    /// Returns the current run state.
    #[must_use]
    pub const fn run_state(&self) -> WorkflowRunState {
        self.run_state
    }

    /// Returns the state of one known node.
    #[must_use]
    pub fn node_state(&self, node_id: WorkflowNodeId) -> Option<&WorkflowNodeState> {
        self.node_states.get(&node_id)
    }

    /// Claims the smallest UUID among ready nodes, if no node is currently running.
    pub fn claim_next(&mut self) -> Result<Option<WorkflowNodeClaim>, WorkflowExecutionError> {
        if matches!(
            self.run_state,
            WorkflowRunState::Succeeded | WorkflowRunState::Failed
        ) {
            return Ok(None);
        }
        if self
            .node_states
            .values()
            .any(|state| matches!(state, WorkflowNodeState::Running { .. }))
        {
            return Ok(None);
        }

        let node_id = self
            .definition
            .nodes()
            .iter()
            .map(WorkflowNode::id)
            .find(|node_id| self.is_ready(*node_id));

        if let Some(node_id) = node_id {
            let state = self
                .node_states
                .get_mut(&node_id)
                .ok_or(WorkflowExecutionError::InconsistentState)?;
            *state = WorkflowNodeState::Running { attempt: 1 };
            self.run_state = WorkflowRunState::Running;
            return Ok(Some(WorkflowNodeClaim {
                node_id,
                attempt: 1,
            }));
        }

        if self
            .node_states
            .values()
            .all(|state| matches!(state, WorkflowNodeState::Succeeded { .. }))
        {
            self.run_state = WorkflowRunState::Succeeded;
            return Ok(None);
        }

        Err(WorkflowExecutionError::InconsistentState)
    }

    /// Marks the exclusively running node as successfully completed.
    pub fn mark_succeeded(
        &mut self,
        node_id: WorkflowNodeId,
    ) -> Result<(), WorkflowExecutionError> {
        let state = self
            .node_states
            .get_mut(&node_id)
            .ok_or(WorkflowExecutionError::UnknownNode { node_id })?;
        let attempt = match state {
            WorkflowNodeState::Running { attempt } => *attempt,
            current_state => {
                return Err(WorkflowExecutionError::NodeNotRunning {
                    node_id,
                    state: current_state.clone(),
                });
            }
        };
        *state = WorkflowNodeState::Succeeded { attempt };
        if self
            .node_states
            .values()
            .all(|state| matches!(state, WorkflowNodeState::Succeeded { .. }))
        {
            self.run_state = WorkflowRunState::Succeeded;
        }
        Ok(())
    }

    /// Marks the exclusively running node as failed and blocks all pending nodes.
    pub fn mark_failed(
        &mut self,
        node_id: WorkflowNodeId,
        failure: WorkflowFailure,
    ) -> Result<(), WorkflowExecutionError> {
        let state = self
            .node_states
            .get_mut(&node_id)
            .ok_or(WorkflowExecutionError::UnknownNode { node_id })?;
        let attempt = match state {
            WorkflowNodeState::Running { attempt } => *attempt,
            current_state => {
                return Err(WorkflowExecutionError::NodeNotRunning {
                    node_id,
                    state: current_state.clone(),
                });
            }
        };
        *state = WorkflowNodeState::Failed { attempt, failure };
        for node_state in self.node_states.values_mut() {
            if matches!(node_state, WorkflowNodeState::Pending) {
                *node_state = WorkflowNodeState::Blocked;
            }
        }
        self.run_state = WorkflowRunState::Failed;
        Ok(())
    }

    fn is_ready(&self, node_id: WorkflowNodeId) -> bool {
        if !matches!(
            self.node_states.get(&node_id),
            Some(WorkflowNodeState::Pending)
        ) {
            return false;
        }
        self.definition
            .edges()
            .iter()
            .filter(|edge| edge.target() == node_id)
            .all(|edge| {
                matches!(
                    self.node_states.get(&edge.source()),
                    Some(WorkflowNodeState::Succeeded { .. })
                )
            })
    }
}

/// Errors raised while building an immutable workflow definition.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowValidationError {
    /// A revision was zero.
    #[error("workflow revisions must be positive")]
    ZeroRevision,
    /// An edge points back to the same node.
    #[error("workflow edge {edge_id:?} cannot depend on node {node_id:?} itself")]
    SelfDependency {
        /// The invalid edge identifier.
        edge_id: WorkflowEdgeId,
        /// The self-referenced node identifier.
        node_id: WorkflowNodeId,
    },
    /// A node identifier is repeated.
    #[error("workflow node {node_id:?} is duplicated")]
    DuplicateNode {
        /// The duplicated node identifier.
        node_id: WorkflowNodeId,
    },
    /// An edge identifier is repeated.
    #[error("workflow edge {edge_id:?} is duplicated")]
    DuplicateEdge {
        /// The duplicated edge identifier.
        edge_id: WorkflowEdgeId,
    },
    /// More than one edge expresses the same dependency.
    #[error("workflow dependency {source_node:?} -> {target:?} is duplicated")]
    DuplicateDependency {
        /// The prerequisite node identifier.
        source_node: WorkflowNodeId,
        /// The dependent node identifier.
        target: WorkflowNodeId,
    },
    /// The graph contains at least one dependency cycle.
    #[error("workflow dependencies contain a cycle")]
    CycleDetected,
    /// An edge source is not a workflow node.
    #[error("workflow edge {edge_id:?} references missing source node {node_id:?}")]
    MissingSourceNode {
        /// The invalid edge identifier.
        edge_id: WorkflowEdgeId,
        /// The missing source node identifier.
        node_id: WorkflowNodeId,
    },
    /// An edge target is not a workflow node.
    #[error("workflow edge {edge_id:?} references missing target node {node_id:?}")]
    MissingTargetNode {
        /// The invalid edge identifier.
        edge_id: WorkflowEdgeId,
        /// The missing target node identifier.
        node_id: WorkflowNodeId,
    },
}

/// Errors raised while parsing workflow capability versions.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowCapabilityVersionParseError {
    /// The version did not contain exactly three numeric components.
    #[error("workflow capability version must contain exactly three numeric components")]
    InvalidFormat,
    /// A version component was not a canonical unsigned integer.
    #[error("workflow capability version component is not canonical: {0}")]
    InvalidComponent(String),
}

/// Errors raised while validating workflow capability identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowCapabilityIdentifierError {
    /// The capability identifier was empty.
    #[error("workflow capability identifier is empty")]
    Empty,
    /// The capability identifier contains unsupported characters or surrounding whitespace.
    #[error("workflow capability identifier is invalid: {value}")]
    Invalid {
        /// The rejected identifier value.
        value: String,
    },
}

/// Errors raised while constructing a workflow node.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowNodeValidationError {
    /// A node declares the same capability more than once.
    #[error("workflow node capability requirement is duplicated: {capability}")]
    DuplicateCapabilityRequirement {
        /// The repeated capability identifier.
        capability: WorkflowCapabilityId,
    },
}

/// Errors raised while constructing an injected capability snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowCapabilitySnapshotError {
    /// A snapshot declares the same capability more than once.
    #[error("workflow capability snapshot is duplicated: {capability}")]
    DuplicateCapability {
        /// The repeated capability identifier.
        capability: WorkflowCapabilityId,
    },
}

/// Errors raised when an injected snapshot cannot satisfy workflow requirements.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowCapabilityValidationError {
    /// A node requires a capability absent from the injected snapshot.
    #[error(
        "workflow node {node_id:?} requires missing capability {capability} at version {required}"
    )]
    MissingCapability {
        /// The blocked workflow node.
        node_id: WorkflowNodeId,
        /// The missing capability identifier.
        capability: WorkflowCapabilityId,
        /// The minimum compatible version required by the node.
        required: WorkflowCapabilityVersion,
    },
    /// A snapshot capability cannot satisfy a node's version requirement.
    #[error(
        "workflow node {node_id:?} requires capability {capability} version {required}, but snapshot provides {provided}"
    )]
    IncompatibleCapability {
        /// The blocked workflow node.
        node_id: WorkflowNodeId,
        /// The incompatible capability identifier.
        capability: WorkflowCapabilityId,
        /// The minimum compatible version required by the node.
        required: WorkflowCapabilityVersion,
        /// The version present in the injected snapshot.
        provided: WorkflowCapabilityVersion,
    },
}

/// Errors raised while validating failure values.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowFailureValidationError {
    /// A failure code was empty after normalization.
    #[error("workflow failure code must not be empty")]
    EmptyCode,
}

/// Errors raised while replaying a workflow execution.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowReplayError {
    /// The source execution is still active.
    #[error("workflow run must be terminal before replay")]
    SourceRunNotTerminal {
        /// The active source state.
        state: WorkflowRunState,
    },
    /// The replay reused the source run identifier.
    #[error("workflow replay run id {run_id:?} must differ from its source")]
    ReusedRunId {
        /// The duplicated run identifier.
        run_id: WorkflowRunId,
    },
}

/// Errors raised while progressing an execution.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WorkflowExecutionError {
    /// A supplied node is absent from the workflow definition.
    #[error("workflow node {node_id:?} does not exist")]
    UnknownNode {
        /// The missing node identifier.
        node_id: WorkflowNodeId,
    },
    /// A completion was reported for a node that is not running.
    #[error("workflow node {node_id:?} is not running")]
    NodeNotRunning {
        /// The node whose state rejected the transition.
        node_id: WorkflowNodeId,
        /// The current node state.
        state: WorkflowNodeState,
    },
    /// The execution state cannot be safely scheduled.
    #[error("workflow execution state is inconsistent")]
    InconsistentState,
}
