//! Provider-free manifest, capability, discovery, and compatibility contracts.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};
use std::str::FromStr;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// A strict three-part contract version.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Version {
    /// Creates a contract version.
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Returns the major component.
    pub const fn major(self) -> u64 {
        self.major
    }

    /// Returns the minor component.
    pub const fn minor(self) -> u64 {
        self.minor
    }

    /// Returns the patch component.
    pub const fn patch(self) -> u64 {
        self.patch
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Errors returned while parsing a contract version.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum VersionParseError {
    /// The version did not contain exactly three numeric components.
    #[error("version must contain exactly three numeric components")]
    InvalidFormat,
    /// A component was not a canonical unsigned integer.
    #[error("version component is not canonical: {0}")]
    InvalidComponent(String),
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut components = value.split('.');
        let major = components.next().ok_or(VersionParseError::InvalidFormat)?;
        let minor = components.next().ok_or(VersionParseError::InvalidFormat)?;
        let patch = components.next().ok_or(VersionParseError::InvalidFormat)?;
        if components.next().is_some() {
            return Err(VersionParseError::InvalidFormat);
        }

        fn parse_component(value: &str) -> Result<u64, VersionParseError> {
            if value.is_empty() || (value.len() > 1 && value.starts_with('0')) {
                return Err(VersionParseError::InvalidComponent(value.to_owned()));
            }
            value
                .parse::<u64>()
                .map_err(|_| VersionParseError::InvalidComponent(value.to_owned()))
        }

        Ok(Self::new(
            parse_component(major)?,
            parse_component(minor)?,
            parse_component(patch)?,
        ))
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

fn validate_identifier(value: &str, kind: &'static str) -> Result<(), IdentifierError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(IdentifierError::Empty { kind });
    }
    if bytes
        .first()
        .is_some_and(|byte| !byte.is_ascii_alphanumeric())
        || bytes
            .last()
            .is_some_and(|byte| !byte.is_ascii_alphanumeric())
        || bytes
            .iter()
            .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'.' | b'-' | b'_' | b':'))
    {
        return Err(IdentifierError::Invalid {
            kind,
            value: value.to_owned(),
        });
    }
    Ok(())
}

/// Errors returned for stable manifest and capability identifiers.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum IdentifierError {
    /// The identifier was empty.
    #[error("{kind} identifier is empty")]
    Empty {
        /// The identifier category rejected by validation.
        kind: &'static str,
    },
    /// The identifier contains unsupported characters or surrounding whitespace.
    #[error("invalid {kind} identifier: {value}")]
    Invalid {
        /// The identifier category rejected by validation.
        kind: &'static str,
        /// The rejected identifier value.
        value: String,
    },
}

/// A validated plugin identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PluginId(String);

impl PluginId {
    /// Creates a validated plugin identifier.
    pub fn new(value: impl AsRef<str>) -> Result<Self, IdentifierError> {
        let value = value.as_ref();
        validate_identifier(value, "plugin")?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for PluginId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PluginId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// A validated capability identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapabilityId(String);

impl CapabilityId {
    /// Creates a validated capability identifier.
    pub fn new(value: impl AsRef<str>) -> Result<Self, IdentifierError> {
        let value = value.as_ref();
        validate_identifier(value, "capability")?;
        Ok(Self(value.to_owned()))
    }

    /// Returns the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for CapabilityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CapabilityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// The provider-free categories a plugin can register.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityKind {
    /// A callable tool.
    Tool,
    /// A readable resource.
    Resource,
    /// A model adapter.
    Model,
    /// An evaluator.
    Evaluator,
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

impl CapabilityKind {
    const ALL: [Self; 10] = [
        Self::Tool,
        Self::Resource,
        Self::Model,
        Self::Evaluator,
        Self::Storage,
        Self::Authentication,
        Self::McpServer,
        Self::Importer,
        Self::Exporter,
        Self::Renderer,
    ];
}

/// A versioned capability exposed by a plugin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityDescriptor {
    id: CapabilityId,
    kind: CapabilityKind,
    version: Version,
}

impl CapabilityDescriptor {
    /// Creates a validated capability descriptor.
    pub fn new(
        id: impl AsRef<str>,
        kind: CapabilityKind,
        version: Version,
    ) -> Result<Self, IdentifierError> {
        Ok(Self {
            id: CapabilityId::new(id)?,
            kind,
            version,
        })
    }

    /// Returns the stable capability identifier.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the typed capability identifier.
    pub fn capability_id(&self) -> &CapabilityId {
        &self.id
    }

    /// Returns the capability category.
    pub const fn kind(&self) -> CapabilityKind {
        self.kind
    }

    /// Returns the capability contract version.
    pub const fn version(&self) -> Version {
        self.version
    }
}

/// The only MCP server capability descriptor schema supported by this crate.
pub const MCP_SERVER_CAPABILITY_DESCRIPTOR_V1: Version = Version::new(1, 0, 0);

/// Errors returned while parsing an MCP server capability descriptor.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum McpServerCapabilityDescriptorParseError {
    /// The raw JSON does not satisfy the strict descriptor wire format.
    #[error("malformed MCP server capability descriptor: {message}")]
    Malformed {
        /// The parser's stable human-readable reason.
        message: String,
    },
    /// The descriptor schema version is not exactly supported.
    #[error("unsupported MCP server capability descriptor version {found}; supported {supported}")]
    UnsupportedSchemaVersion {
        /// The schema version declared by the server descriptor.
        found: Version,
        /// The only schema version supported by this crate.
        supported: Version,
    },
    /// A stable capability identifier occurs more than once.
    #[error("MCP server capability descriptor declares capability more than once: {capability}")]
    DuplicateCapability {
        /// The ambiguous stable capability identifier.
        capability: CapabilityId,
    },
}

/// A strict, versioned capability description for one local MCP server.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct McpServerCapabilityDescriptor {
    schema_version: Version,
    plugin_id: PluginId,
    capabilities: Vec<CapabilityDescriptor>,
}

impl McpServerCapabilityDescriptor {
    /// Parses one strict JSON descriptor without performing network or process I/O.
    pub fn parse_json(raw: &str) -> Result<Self, McpServerCapabilityDescriptorParseError> {
        let wire: McpServerCapabilityDescriptorWire =
            serde_json::from_str(raw).map_err(|error| {
                McpServerCapabilityDescriptorParseError::Malformed {
                    message: error.to_string(),
                }
            })?;
        Self::new(wire.schema_version, wire.plugin_id, wire.capabilities)
    }

    /// Creates a validated, canonicalized descriptor.
    pub fn new(
        schema_version: Version,
        plugin_id: PluginId,
        mut capabilities: Vec<CapabilityDescriptor>,
    ) -> Result<Self, McpServerCapabilityDescriptorParseError> {
        if schema_version != MCP_SERVER_CAPABILITY_DESCRIPTOR_V1 {
            return Err(
                McpServerCapabilityDescriptorParseError::UnsupportedSchemaVersion {
                    found: schema_version,
                    supported: MCP_SERVER_CAPABILITY_DESCRIPTOR_V1,
                },
            );
        }
        capabilities.sort_by(|left, right| {
            left.capability_id()
                .cmp(right.capability_id())
                .then_with(|| left.kind().cmp(&right.kind()))
                .then_with(|| left.version().cmp(&right.version()))
        });
        for pair in capabilities.windows(2) {
            if pair[0].capability_id() == pair[1].capability_id() {
                return Err(
                    McpServerCapabilityDescriptorParseError::DuplicateCapability {
                        capability: pair[0].capability_id().clone(),
                    },
                );
            }
        }
        Ok(Self {
            schema_version,
            plugin_id,
            capabilities,
        })
    }

    /// Returns the exact descriptor schema version.
    pub const fn schema_version(&self) -> Version {
        self.schema_version
    }

    /// Returns the plugin identity the server description belongs to.
    pub fn plugin_id(&self) -> &PluginId {
        &self.plugin_id
    }

    /// Returns capabilities in canonical stable identifier order.
    pub fn capabilities(&self) -> &[CapabilityDescriptor] {
        &self.capabilities
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct McpServerCapabilityDescriptorWire {
    schema_version: Version,
    plugin_id: PluginId,
    capabilities: Vec<CapabilityDescriptor>,
}

/// The explicit schema version for capability availability projections.
pub const CAPABILITY_AVAILABILITY_PROJECTION_V1: Version = Version::new(1, 0, 0);

/// Schema version for an immutable, provider-free capability registry snapshot.
pub const CAPABILITY_REGISTRY_SNAPSHOT_V1: Version = Version::new(1, 0, 0);

/// Whether a capability is available for a future consumer to use.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityAvailability {
    /// The capability was registered after successful runtime activation.
    Available,
    /// The capability was not admitted for use.
    Unavailable,
}

/// Whether a capability satisfies the local plugin compatibility contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCompatibility {
    /// The manifest and capability contract are compatible with this runtime.
    Compatible,
    /// The capability could not be admitted by the local compatibility or registry contract.
    Incompatible,
}

/// A stable, secret-free code explaining why a capability is unavailable.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDiagnosticCode {
    /// The manifest was incompatible with the runtime contract.
    ManifestIncompatible,
    /// The capability was rejected by the atomic registry contract.
    RegistryRejected,
    /// Plugin construction failed before initialization.
    FactoryRejected,
    /// Plugin initialization failed before activation.
    InitializationRejected,
    /// Plugin activation failed before registry publication.
    ActivationRejected,
}

impl CapabilityDiagnosticCode {
    /// Returns the stable machine-readable diagnostic code.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManifestIncompatible => "manifest_incompatible",
            Self::RegistryRejected => "registry_rejected",
            Self::FactoryRejected => "factory_rejected",
            Self::InitializationRejected => "initialization_rejected",
            Self::ActivationRejected => "activation_rejected",
        }
    }
}

impl fmt::Display for CapabilityDiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One deterministic capability availability result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityAvailabilityEntry {
    plugin_id: PluginId,
    capability_id: CapabilityId,
    capability_version: Version,
    availability: CapabilityAvailability,
    compatibility: CapabilityCompatibility,
    diagnostic_code: Option<CapabilityDiagnosticCode>,
}

impl CapabilityAvailabilityEntry {
    /// Creates an available result for one registered, activated capability.
    pub fn available(plugin_id: &PluginId, capability: &CapabilityDescriptor) -> Self {
        Self {
            plugin_id: plugin_id.clone(),
            capability_id: capability.capability_id().clone(),
            capability_version: capability.version(),
            availability: CapabilityAvailability::Available,
            compatibility: CapabilityCompatibility::Compatible,
            diagnostic_code: None,
        }
    }

    /// Creates an unavailable result with a safe diagnostic code.
    pub fn unavailable(
        plugin_id: &PluginId,
        capability: &CapabilityDescriptor,
        compatibility: CapabilityCompatibility,
        diagnostic_code: CapabilityDiagnosticCode,
    ) -> Self {
        Self {
            plugin_id: plugin_id.clone(),
            capability_id: capability.capability_id().clone(),
            capability_version: capability.version(),
            availability: CapabilityAvailability::Unavailable,
            compatibility,
            diagnostic_code: Some(diagnostic_code),
        }
    }

    /// Returns the stable owning plugin identifier.
    pub fn plugin_id(&self) -> &PluginId {
        &self.plugin_id
    }

    /// Returns the stable capability identifier.
    pub fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    /// Returns the explicit capability contract version.
    pub const fn capability_version(&self) -> Version {
        self.capability_version
    }

    /// Returns whether the capability is available to consumers.
    pub const fn availability(&self) -> CapabilityAvailability {
        self.availability
    }

    /// Returns whether the capability is compatible with the local contract.
    pub const fn compatibility(&self) -> CapabilityCompatibility {
        self.compatibility
    }

    /// Returns the safe unavailable diagnostic code, when there is one.
    pub const fn diagnostic_code(&self) -> Option<CapabilityDiagnosticCode> {
        self.diagnostic_code
    }
}

/// A canonical V1 view of capability availability for an external bridge.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityAvailabilityProjection {
    version: Version,
    entries: Vec<CapabilityAvailabilityEntry>,
}

impl CapabilityAvailabilityProjection {
    /// Creates a V1 projection with entries in canonical identifier and outcome order.
    pub fn new(entries: impl IntoIterator<Item = CapabilityAvailabilityEntry>) -> Self {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_by(|left, right| {
            left.plugin_id
                .cmp(&right.plugin_id)
                .then_with(|| left.capability_id.cmp(&right.capability_id))
                .then_with(|| left.capability_version.cmp(&right.capability_version))
                .then_with(|| left.availability.cmp(&right.availability))
                .then_with(|| left.compatibility.cmp(&right.compatibility))
                .then_with(|| left.diagnostic_code.cmp(&right.diagnostic_code))
        });
        Self {
            version: CAPABILITY_AVAILABILITY_PROJECTION_V1,
            entries,
        }
    }

    /// Returns the explicit projection schema version.
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Returns availability entries in canonical order.
    pub fn entries(&self) -> &[CapabilityAvailabilityEntry] {
        &self.entries
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapabilityAvailabilityProjectionWire {
    version: Version,
    entries: Vec<CapabilityAvailabilityEntry>,
}

impl<'de> Deserialize<'de> for CapabilityAvailabilityProjection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = CapabilityAvailabilityProjectionWire::deserialize(deserializer)?;
        if wire.version != CAPABILITY_AVAILABILITY_PROJECTION_V1 {
            return Err(D::Error::custom(format!(
                "unsupported capability availability projection version {}; runtime supports {}",
                wire.version, CAPABILITY_AVAILABILITY_PROJECTION_V1
            )));
        }
        Ok(Self::new(wire.entries))
    }
}

/// Lifecycle phases understood by the current runtime.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePhase {
    /// Allocate and validate plugin state.
    Initialize,
    /// Make plugin capabilities active.
    Activate,
    /// Stop active work before shutdown.
    Deactivate,
    /// Release plugin state.
    Shutdown,
}

impl fmt::Display for LifecyclePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Initialize => "initialize",
            Self::Activate => "activate",
            Self::Deactivate => "deactivate",
            Self::Shutdown => "shutdown",
        };
        formatter.write_str(name)
    }
}

/// Errors returned while constructing a lifecycle contract.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LifecycleContractError {
    /// The contract did not declare any phases.
    #[error("lifecycle contract must declare at least one phase")]
    Empty,
    /// A phase was declared more than once.
    #[error("lifecycle phase declared more than once: {0}")]
    DuplicatePhase(LifecyclePhase),
}

/// A versioned set of lifecycle phases required by a plugin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleContract {
    version: Version,
    required_phases: Vec<LifecyclePhase>,
}

impl LifecycleContract {
    /// Creates a canonical lifecycle contract.
    pub fn new(
        version: Version,
        required_phases: impl IntoIterator<Item = LifecyclePhase>,
    ) -> Result<Self, LifecycleContractError> {
        let mut declared = BTreeSet::new();
        for phase in required_phases {
            if !declared.insert(phase) {
                return Err(LifecycleContractError::DuplicatePhase(phase));
            }
        }
        if declared.is_empty() {
            return Err(LifecycleContractError::Empty);
        }

        let required_phases = [
            LifecyclePhase::Initialize,
            LifecyclePhase::Activate,
            LifecyclePhase::Deactivate,
            LifecyclePhase::Shutdown,
        ]
        .into_iter()
        .filter(|phase| declared.contains(phase))
        .collect();

        Ok(Self {
            version,
            required_phases,
        })
    }

    /// Returns the lifecycle contract version.
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Returns phases in canonical runtime order.
    pub fn required_phases(&self) -> &[LifecyclePhase] {
        &self.required_phases
    }

    /// Returns whether the contract requires a phase.
    pub fn requires(&self, phase: LifecyclePhase) -> bool {
        self.required_phases.contains(&phase)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LifecycleContractWire {
    version: Version,
    required_phases: Vec<LifecyclePhase>,
}

impl<'de> Deserialize<'de> for LifecycleContract {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = LifecycleContractWire::deserialize(deserializer)?;
        Self::new(wire.version, wire.required_phases).map_err(D::Error::custom)
    }
}

/// Errors returned while constructing a plugin manifest.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ManifestError {
    /// The manifest schema version must be non-zero.
    #[error("manifest version must be non-zero")]
    ZeroManifestVersion,
    /// The display name is empty or padded with whitespace.
    #[error("manifest name must be non-empty and trimmed")]
    InvalidName,
    /// A capability identifier occurs more than once.
    #[error("capability is declared more than once: {0}")]
    DuplicateCapability(CapabilityId),
    /// A nested identifier or lifecycle contract is invalid.
    #[error(transparent)]
    InvalidContract(#[from] LifecycleContractError),
}

/// A versioned, serializable plugin manifest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    manifest_version: u16,
    id: PluginId,
    name: String,
    version: Version,
    capabilities: Vec<CapabilityDescriptor>,
    lifecycle: LifecycleContract,
}

impl PluginManifest {
    /// Creates a validated and canonicalized manifest.
    pub fn new(
        manifest_version: u16,
        id: PluginId,
        name: impl Into<String>,
        version: Version,
        capabilities: Vec<CapabilityDescriptor>,
        lifecycle: LifecycleContract,
    ) -> Result<Self, ManifestError> {
        if manifest_version == 0 {
            return Err(ManifestError::ZeroManifestVersion);
        }
        let name = name.into();
        if name.is_empty() || name.trim() != name {
            return Err(ManifestError::InvalidName);
        }
        let mut capabilities = capabilities;
        capabilities.sort_by(|left, right| {
            left.capability_id()
                .cmp(right.capability_id())
                .then_with(|| left.kind().cmp(&right.kind()))
                .then_with(|| left.version().cmp(&right.version()))
        });
        for pair in capabilities.windows(2) {
            if pair[0].capability_id() == pair[1].capability_id() {
                return Err(ManifestError::DuplicateCapability(
                    pair[0].capability_id().clone(),
                ));
            }
        }

        Ok(Self {
            manifest_version,
            id,
            name,
            version,
            capabilities,
            lifecycle,
        })
    }

    /// Returns the manifest schema version.
    pub const fn manifest_version(&self) -> u16 {
        self.manifest_version
    }

    /// Returns the plugin identifier.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the typed plugin identifier.
    pub fn plugin_id(&self) -> &PluginId {
        &self.id
    }

    /// Returns the display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the plugin implementation version.
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Returns capabilities in canonical identifier order.
    pub fn capabilities(&self) -> &[CapabilityDescriptor] {
        &self.capabilities
    }

    /// Returns the lifecycle contract.
    pub fn lifecycle(&self) -> &LifecycleContract {
        &self.lifecycle
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PluginManifestWire {
    manifest_version: u16,
    id: PluginId,
    name: String,
    version: Version,
    capabilities: Vec<CapabilityDescriptor>,
    lifecycle: LifecycleContract,
}

impl<'de> Deserialize<'de> for PluginManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PluginManifestWire::deserialize(deserializer)?;
        Self::new(
            wire.manifest_version,
            wire.id,
            wire.name,
            wire.version,
            wire.capabilities,
            wire.lifecycle,
        )
        .map_err(D::Error::custom)
    }
}

/// Compatibility errors that prevent a manifest from being used.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CompatibilityError {
    /// The manifest schema is not supported by this runtime.
    #[error("unsupported manifest version {found}; runtime supports {supported}")]
    UnsupportedManifestVersion {
        /// The manifest schema version declared by the candidate.
        found: u16,
        /// The manifest schema version understood by this runtime.
        supported: u16,
    },
    /// The lifecycle contract has an incompatible major version.
    #[error("lifecycle major version {found} is incompatible with runtime {supported}")]
    LifecycleMajorMismatch {
        /// The lifecycle contract version declared by the candidate.
        found: Version,
        /// The lifecycle contract version understood by this runtime.
        supported: Version,
    },
    /// The lifecycle contract requires a newer compatible version.
    #[error("lifecycle version {found} is newer than runtime {supported}")]
    LifecycleVersionTooNew {
        /// The lifecycle contract version declared by the candidate.
        found: Version,
        /// The lifecycle contract version understood by this runtime.
        supported: Version,
    },
    /// The manifest requires a lifecycle phase this runtime does not expose.
    #[error("unsupported lifecycle phase: {phase}")]
    UnsupportedLifecyclePhase {
        /// The requested lifecycle phase unsupported by this runtime.
        phase: LifecyclePhase,
    },
    /// A capability has an incompatible major version.
    #[error(
        "capability {capability} major version {found} is incompatible with runtime {supported}"
    )]
    CapabilityMajorMismatch {
        /// The capability whose major version is incompatible.
        capability: CapabilityId,
        /// The capability version declared by the candidate.
        found: Version,
        /// The capability version understood by this runtime.
        supported: Version,
    },
    /// A capability requires a newer compatible version.
    #[error("capability {capability} version {found} is newer than runtime {supported}")]
    CapabilityVersionTooNew {
        /// The capability whose compatible version is too new.
        capability: CapabilityId,
        /// The capability version declared by the candidate.
        found: Version,
        /// The capability version understood by this runtime.
        supported: Version,
    },
}

/// The versions and phases understood by a provider-free runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeCompatibility {
    manifest_version: u16,
    lifecycle_version: Version,
    capability_versions: BTreeMap<CapabilityKind, Version>,
    lifecycle_phases: BTreeSet<LifecyclePhase>,
}

impl RuntimeCompatibility {
    /// Creates compatibility rules for one manifest and lifecycle schema.
    pub fn new(manifest_version: u16, lifecycle_version: Version) -> Self {
        Self {
            manifest_version,
            lifecycle_version,
            capability_versions: CapabilityKind::ALL
                .into_iter()
                .map(|kind| (kind, lifecycle_version))
                .collect(),
            lifecycle_phases: [
                LifecyclePhase::Initialize,
                LifecyclePhase::Activate,
                LifecyclePhase::Deactivate,
                LifecyclePhase::Shutdown,
            ]
            .into_iter()
            .collect(),
        }
    }

    /// Returns the current ContextLab compatibility contract.
    pub fn current() -> Self {
        Self::new(1, Version::new(1, 0, 0))
    }

    /// Returns the supported manifest schema version.
    pub const fn manifest_version(&self) -> u16 {
        self.manifest_version
    }

    /// Returns the supported lifecycle version.
    pub const fn lifecycle_version(&self) -> Version {
        self.lifecycle_version
    }

    /// Checks every version and lifecycle requirement in a manifest.
    pub fn check_manifest(&self, manifest: &PluginManifest) -> Result<(), CompatibilityError> {
        if manifest.manifest_version() != self.manifest_version {
            return Err(CompatibilityError::UnsupportedManifestVersion {
                found: manifest.manifest_version(),
                supported: self.manifest_version,
            });
        }
        self.check_version(
            manifest.lifecycle().version(),
            self.lifecycle_version,
            |found, supported| CompatibilityError::LifecycleMajorMismatch { found, supported },
            |found, supported| CompatibilityError::LifecycleVersionTooNew { found, supported },
        )?;
        for phase in manifest.lifecycle().required_phases() {
            if !self.lifecycle_phases.contains(phase) {
                return Err(CompatibilityError::UnsupportedLifecyclePhase { phase: *phase });
            }
        }
        for capability in manifest.capabilities() {
            self.check_capability(capability)?;
        }
        Ok(())
    }

    /// Checks one capability against the runtime capability contract.
    pub fn check_capability(
        &self,
        capability: &CapabilityDescriptor,
    ) -> Result<(), CompatibilityError> {
        let supported = self
            .capability_versions
            .get(&capability.kind())
            .copied()
            .unwrap_or(self.lifecycle_version);
        self.check_version(
            capability.version(),
            supported,
            |found, supported| CompatibilityError::CapabilityMajorMismatch {
                capability: capability.capability_id().clone(),
                found,
                supported,
            },
            |found, supported| CompatibilityError::CapabilityVersionTooNew {
                capability: capability.capability_id().clone(),
                found,
                supported,
            },
        )
    }

    fn check_version<E, F, G>(
        &self,
        found: Version,
        supported: Version,
        major_error: F,
        newer_error: G,
    ) -> Result<(), E>
    where
        F: FnOnce(Version, Version) -> E,
        G: FnOnce(Version, Version) -> E,
    {
        if found.major() != supported.major() {
            return Err(major_error(found, supported));
        }
        if found > supported {
            return Err(newer_error(found, supported));
        }
        Ok(())
    }
}

/// A source candidate discovered from a manifest file or equivalent input.
#[derive(Clone, Debug)]
pub struct DiscoveryCandidate {
    source: String,
    parsed: Result<PluginManifest, String>,
}

impl DiscoveryCandidate {
    /// Parses one JSON manifest while retaining parse errors for isolated reporting.
    pub fn parse(source: impl Into<String>, json: &str) -> Self {
        Self {
            source: source.into(),
            parsed: serde_json::from_str(json).map_err(|error| error.to_string()),
        }
    }

    /// Creates a valid discovery candidate from a manifest.
    pub fn from_manifest(source: impl Into<String>, manifest: PluginManifest) -> Self {
        Self {
            source: source.into(),
            parsed: Ok(manifest),
        }
    }

    /// Returns the discovery source label.
    pub fn source(&self) -> &str {
        &self.source
    }

    fn tie_breaker(&self) -> String {
        match &self.parsed {
            Ok(manifest) => manifest.id().to_owned(),
            Err(error) => error.clone(),
        }
    }
}

/// An isolated diagnostic for one discovery candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryDiagnostic {
    source: String,
    message: String,
}

impl DiscoveryDiagnostic {
    /// Returns the source that produced the diagnostic.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the stable human-readable reason.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the diagnostic reason.
    pub fn reason(&self) -> &str {
        self.message()
    }
}

/// Errors raised while adding a manifest to a capability registry.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum RegistryError {
    /// The plugin identifier is already registered.
    #[error("plugin is already registered: {0}")]
    DuplicatePlugin(PluginId),
    /// A capability identifier is already owned by another plugin.
    #[error("capability {capability} is already registered by {plugin}")]
    DuplicateCapability {
        /// The capability identifier already owned by another plugin.
        capability: CapabilityId,
        /// The plugin that already owns the capability identifier.
        plugin: PluginId,
    },
}

/// Errors raised while resolving a versioned capability.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CapabilityResolutionError {
    /// The requested identifier is invalid.
    #[error(transparent)]
    InvalidIdentifier(#[from] IdentifierError),
    /// No registered plugin exposes the requested capability.
    #[error("capability is not registered: {0}")]
    Missing(CapabilityId),
    /// A registered capability cannot satisfy the requested version.
    #[error("capability {capability} version {provided} cannot satisfy {required}")]
    IncompatibleVersion {
        /// The requested capability identifier.
        capability: CapabilityId,
        /// The version required by the consumer.
        required: Version,
        /// The version provided by the registered plugin.
        provided: Version,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RegisteredCapability {
    plugin: PluginId,
    descriptor: CapabilityDescriptor,
}

/// A deterministic registry of compatible plugins and capabilities.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CapabilityRegistry {
    plugins: BTreeMap<PluginId, PluginManifest>,
    capabilities: BTreeMap<CapabilityId, RegisteredCapability>,
}

/// An immutable capability registry view shared by local domain consumers.
///
/// The snapshot owns a clone of the registry and a canonical availability projection. Consumers
/// can therefore use one exact capability set for a request without observing later runtime
/// registration changes. The fingerprint covers the validated manifest set only; it contains no
/// provider payloads, credentials, or raw content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityRegistrySnapshotV1 {
    schema_version: Version,
    registry: CapabilityRegistry,
    availability: CapabilityAvailabilityProjection,
    fingerprint: String,
}

impl CapabilityRegistrySnapshotV1 {
    /// Creates a V1 snapshot from an owned, already validated registry.
    #[must_use]
    pub fn new(registry: CapabilityRegistry) -> Self {
        let availability = registry.capability_availability();
        let fingerprint = registry_fingerprint(&registry);
        Self {
            schema_version: CAPABILITY_REGISTRY_SNAPSHOT_V1,
            registry,
            availability,
            fingerprint,
        }
    }

    /// Returns the explicit snapshot schema version.
    #[must_use]
    pub const fn schema_version(&self) -> Version {
        self.schema_version
    }

    /// Returns the immutable registry captured by this snapshot.
    #[must_use]
    pub const fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }

    /// Returns the canonical, secret-free availability projection captured by this snapshot.
    #[must_use]
    pub const fn availability(&self) -> &CapabilityAvailabilityProjection {
        &self.availability
    }

    /// Returns the deterministic SHA-256 fingerprint of the validated manifest set.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
}

fn registry_fingerprint(registry: &CapabilityRegistry) -> String {
    let manifests = registry.plugins().collect::<Vec<_>>();
    let encoded = serde_json::to_vec(&(
        CAPABILITY_REGISTRY_SNAPSHOT_V1,
        manifests,
        registry.capability_availability(),
    ))
    .expect("validated capability registry data must serialize");
    let digest = Sha256::digest(encoded);
    let mut fingerprint = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut fingerprint, "{byte:02x}").expect("writing to a String cannot fail");
    }
    fingerprint
}

impl CapabilityRegistry {
    /// Discovers candidates in source order and isolates every rejected source.
    pub fn discover(
        candidates: impl IntoIterator<Item = DiscoveryCandidate>,
        compatibility: &RuntimeCompatibility,
    ) -> DiscoveryReport {
        let mut candidates: Vec<_> = candidates.into_iter().collect();
        candidates.sort_by(|left, right| {
            left.source
                .cmp(&right.source)
                .then_with(|| left.tie_breaker().cmp(&right.tie_breaker()))
        });

        let mut registry = Self::default();
        let mut accepted_plugins = Vec::new();
        let mut diagnostics = Vec::new();
        for candidate in candidates {
            let source = candidate.source;
            let manifest = match candidate.parsed {
                Ok(manifest) => manifest,
                Err(message) => {
                    diagnostics.push(DiscoveryDiagnostic { source, message });
                    continue;
                }
            };

            if let Err(error) = compatibility.check_manifest(&manifest) {
                diagnostics.push(DiscoveryDiagnostic {
                    source,
                    message: error.to_string(),
                });
                continue;
            }
            if let Err(error) = registry.register(manifest.clone()) {
                diagnostics.push(DiscoveryDiagnostic {
                    source,
                    message: error.to_string(),
                });
                continue;
            }
            accepted_plugins.push(manifest.plugin_id().clone());
        }

        DiscoveryReport {
            registry,
            accepted_plugins,
            diagnostics,
        }
    }

    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Captures one immutable V1 snapshot for cross-domain consumers.
    #[must_use]
    pub fn snapshot(&self) -> CapabilityRegistrySnapshotV1 {
        CapabilityRegistrySnapshotV1::new(self.clone())
    }

    /// Checks whether a manifest can be registered without changing state.
    pub fn can_register(&self, manifest: &PluginManifest) -> Result<(), RegistryError> {
        if self.plugins.contains_key(manifest.plugin_id()) {
            return Err(RegistryError::DuplicatePlugin(manifest.plugin_id().clone()));
        }
        for capability in manifest.capabilities() {
            if let Some(existing) = self.capabilities.get(capability.capability_id()) {
                return Err(RegistryError::DuplicateCapability {
                    capability: capability.capability_id().clone(),
                    plugin: existing.plugin.clone(),
                });
            }
        }
        Ok(())
    }

    /// Registers a manifest atomically.
    pub fn register(&mut self, manifest: PluginManifest) -> Result<(), RegistryError> {
        self.can_register(&manifest)?;
        let plugin = manifest.plugin_id().clone();
        for descriptor in manifest.capabilities().iter().cloned() {
            self.capabilities.insert(
                descriptor.capability_id().clone(),
                RegisteredCapability {
                    plugin: plugin.clone(),
                    descriptor,
                },
            );
        }
        self.plugins.insert(plugin, manifest);
        Ok(())
    }

    /// Returns registered plugin manifests in identifier order.
    pub fn plugins(&self) -> impl Iterator<Item = &PluginManifest> {
        self.plugins.values()
    }

    /// Returns registered capability descriptors in identifier order.
    pub fn capabilities(&self) -> impl Iterator<Item = &CapabilityDescriptor> {
        self.capabilities.values().map(|entry| &entry.descriptor)
    }

    /// Projects registered capabilities as canonical V1 available entries.
    pub fn capability_availability(&self) -> CapabilityAvailabilityProjection {
        CapabilityAvailabilityProjection::new(
            self.capabilities.values().map(|entry| {
                CapabilityAvailabilityEntry::available(&entry.plugin, &entry.descriptor)
            }),
        )
    }

    /// Returns a capability by identifier without applying a version requirement.
    pub fn capability(&self, id: &str) -> Option<&CapabilityDescriptor> {
        self.capabilities
            .get(&CapabilityId::new(id).ok()?)
            .map(|entry| &entry.descriptor)
    }

    /// Returns the owning plugin for a capability.
    pub fn capability_owner(&self, id: &str) -> Option<&PluginId> {
        self.capabilities
            .get(&CapabilityId::new(id).ok()?)
            .map(|entry| &entry.plugin)
    }

    /// Resolves a capability only when its version requirement is satisfied.
    pub fn resolve_capability(
        &self,
        id: &str,
        required: Version,
    ) -> Result<&CapabilityDescriptor, CapabilityResolutionError> {
        let capability_id = CapabilityId::new(id)?;
        let entry = self
            .capabilities
            .get(&capability_id)
            .ok_or_else(|| CapabilityResolutionError::Missing(capability_id.clone()))?;
        if entry.descriptor.version().major() != required.major()
            || entry.descriptor.version() < required
        {
            return Err(CapabilityResolutionError::IncompatibleVersion {
                capability: capability_id,
                required,
                provided: entry.descriptor.version(),
            });
        }
        Ok(&entry.descriptor)
    }

    /// Alias for version-checked capability resolution.
    pub fn resolve(
        &self,
        id: &str,
        required: Version,
    ) -> Result<&CapabilityDescriptor, CapabilityResolutionError> {
        self.resolve_capability(id, required)
    }
}

/// The deterministic result of registry discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryReport {
    registry: CapabilityRegistry,
    accepted_plugins: Vec<PluginId>,
    diagnostics: Vec<DiscoveryDiagnostic>,
}

impl DiscoveryReport {
    /// Returns accepted plugin identifiers in source order.
    pub fn accepted_plugins(&self) -> &[PluginId] {
        &self.accepted_plugins
    }

    /// Returns the resulting registry.
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }

    /// Returns isolated candidate diagnostics in source order.
    pub fn diagnostics(&self) -> &[DiscoveryDiagnostic] {
        &self.diagnostics
    }
}
