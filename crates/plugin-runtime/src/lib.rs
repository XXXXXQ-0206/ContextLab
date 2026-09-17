//! Provider-free, fail-closed plugin lifecycle runtime.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use contextlab_mcp::{
    CapabilityAvailabilityEntry, CapabilityAvailabilityProjection, CapabilityCompatibility,
    CapabilityDescriptor, CapabilityDiagnosticCode, CapabilityId, CapabilityRegistry,
    CompatibilityError, McpServerCapabilityDescriptor, McpServerCapabilityDescriptorParseError,
    PluginId, PluginManifest, RegistryError, RuntimeCompatibility,
};
use thiserror::Error;

/// The stage at which a plugin load attempt succeeds or fails.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LoadPhase {
    /// Manifest and capability compatibility validation.
    Compatibility,
    /// Atomic capability registry validation or publication.
    Registry,
    /// Plugin factory construction.
    Factory,
    /// Plugin initialization.
    Initialize,
    /// Plugin activation.
    Activate,
    /// Plugin deactivation during cleanup.
    Deactivate,
    /// Plugin shutdown during cleanup.
    Shutdown,
}

impl fmt::Display for LoadPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Compatibility => "compatibility",
            Self::Registry => "registry",
            Self::Factory => "factory",
            Self::Initialize => "initialize",
            Self::Activate => "activate",
            Self::Deactivate => "deactivate",
            Self::Shutdown => "shutdown",
        };
        formatter.write_str(name)
    }
}

/// An error returned by a plugin lifecycle hook.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct PluginError {
    message: String,
}

impl PluginError {
    /// Creates a lifecycle hook error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// An error returned by a plugin factory.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct PluginLoadError {
    message: String,
}

impl PluginLoadError {
    /// Creates a factory load error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// The lifecycle hooks implemented by an in-process plugin.
pub trait Plugin {
    /// Initializes plugin state.
    fn initialize(&mut self) -> Result<(), PluginError>;
    /// Activates plugin capabilities.
    fn activate(&mut self) -> Result<(), PluginError>;
    /// Deactivates plugin capabilities.
    fn deactivate(&mut self) -> Result<(), PluginError>;
    /// Shuts down plugin state.
    fn shutdown(&mut self) -> Result<(), PluginError>;
}

/// A factory that constructs one plugin instance from its validated manifest.
pub trait PluginFactory {
    /// Constructs a plugin instance.
    fn load(&self, manifest: &PluginManifest) -> Result<Box<dyn Plugin>, PluginLoadError>;
}

/// A source, manifest, and factory awaiting runtime loading.
pub struct PluginBundle {
    source: String,
    manifest: PluginManifest,
    factory: Box<dyn PluginFactory>,
}

impl PluginBundle {
    /// Creates a plugin load bundle.
    pub fn new(
        source: impl Into<String>,
        manifest: PluginManifest,
        factory: Box<dyn PluginFactory>,
    ) -> Self {
        Self {
            source: source.into(),
            manifest,
            factory,
        }
    }

    /// Returns the source label used for deterministic ordering and diagnostics.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the validated manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

/// An isolated diagnostic for one plugin load attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginLoadDiagnostic {
    source: String,
    plugin_id: PluginId,
    phase: LoadPhase,
    message: String,
}

impl PluginLoadDiagnostic {
    /// Returns the source label.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the plugin identifier.
    pub fn plugin_id(&self) -> &PluginId {
        &self.plugin_id
    }

    /// Returns the failed load phase.
    pub const fn phase(&self) -> LoadPhase {
        self.phase
    }

    /// Returns the stable diagnostic message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Alias for the diagnostic message.
    pub fn error(&self) -> &str {
        self.message()
    }
}

/// The stable category for a rejected MCP capability negotiation description.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CapabilityNegotiationDiagnosticCode {
    /// The raw descriptor cannot be parsed as the strict local schema.
    MalformedDescriptor,
    /// The raw descriptor declares a schema version this runtime does not support.
    UnsupportedDescriptorVersion,
    /// The raw descriptor repeats a stable capability identifier.
    DuplicateCapability,
    /// The descriptor identifies a different plugin than the manifest.
    PluginIdMismatch,
    /// The manifest declares a capability absent from the descriptor.
    MissingCapability,
    /// The descriptor declares a capability absent from the manifest.
    UnexpectedCapability,
    /// One capability identifier has different exact kinds.
    CapabilityKindMismatch,
    /// One capability identifier has different exact contract versions.
    CapabilityVersionMismatch,
}

/// One deterministic, safe diagnostic from local capability negotiation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityNegotiationDiagnostic {
    source: String,
    code: CapabilityNegotiationDiagnosticCode,
    capability_id: Option<CapabilityId>,
    message: String,
}

impl CapabilityNegotiationDiagnostic {
    /// Returns the caller-supplied local descriptor source label.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the stable diagnostic category.
    pub const fn code(&self) -> CapabilityNegotiationDiagnosticCode {
        self.code
    }

    /// Returns the affected stable capability identifier when one exists.
    pub fn capability_id(&self) -> Option<&CapabilityId> {
        self.capability_id.as_ref()
    }

    /// Returns the stable human-readable diagnostic message.
    pub fn message(&self) -> &str {
        &self.message
    }

    fn new(
        source: String,
        code: CapabilityNegotiationDiagnosticCode,
        capability_id: Option<CapabilityId>,
        message: String,
    ) -> Self {
        Self {
            source,
            code,
            capability_id,
            message,
        }
    }
}

/// The fail-closed result of comparing one plugin manifest to one MCP descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityNegotiationReport {
    capabilities: Vec<CapabilityDescriptor>,
    diagnostics: Vec<CapabilityNegotiationDiagnostic>,
}

impl CapabilityNegotiationReport {
    /// Returns canonical negotiated capabilities, or none when any diagnostic exists.
    pub fn capabilities(&self) -> &[CapabilityDescriptor] {
        &self.capabilities
    }

    /// Returns deterministic structured diagnostics.
    pub fn diagnostics(&self) -> &[CapabilityNegotiationDiagnostic] {
        &self.diagnostics
    }
}

/// Compares a plugin manifest and a raw local MCP capability descriptor without loading either.
pub struct McpCapabilityNegotiator;

impl McpCapabilityNegotiator {
    /// Negotiates only exact plugin identity, capability kind, and capability version matches.
    pub fn negotiate(
        source: impl Into<String>,
        manifest: &PluginManifest,
        raw_descriptor: &str,
    ) -> CapabilityNegotiationReport {
        let source = source.into();
        let descriptor = match McpServerCapabilityDescriptor::parse_json(raw_descriptor) {
            Ok(descriptor) => descriptor,
            Err(error) => return parse_failure_report(source, error),
        };

        if descriptor.plugin_id() != manifest.plugin_id() {
            return CapabilityNegotiationReport {
                capabilities: Vec::new(),
                diagnostics: vec![CapabilityNegotiationDiagnostic::new(
                    source,
                    CapabilityNegotiationDiagnosticCode::PluginIdMismatch,
                    None,
                    format!(
                        "descriptor plugin {} does not match manifest plugin {}",
                        descriptor.plugin_id(),
                        manifest.plugin_id()
                    ),
                )],
            };
        }

        let manifest_capabilities: BTreeMap<_, _> = manifest
            .capabilities()
            .iter()
            .map(|capability| (capability.capability_id().clone(), capability))
            .collect();
        let descriptor_capabilities: BTreeMap<_, _> = descriptor
            .capabilities()
            .iter()
            .map(|capability| (capability.capability_id().clone(), capability))
            .collect();
        let capability_ids: BTreeSet<_> = manifest_capabilities
            .keys()
            .chain(descriptor_capabilities.keys())
            .cloned()
            .collect();

        let mut diagnostics = Vec::new();
        for capability_id in capability_ids {
            match (
                manifest_capabilities.get(&capability_id),
                descriptor_capabilities.get(&capability_id),
            ) {
                (Some(_), None) => diagnostics.push(CapabilityNegotiationDiagnostic::new(
                    source.clone(),
                    CapabilityNegotiationDiagnosticCode::MissingCapability,
                    Some(capability_id.clone()),
                    format!("descriptor is missing manifest capability {capability_id}"),
                )),
                (None, Some(_)) => diagnostics.push(CapabilityNegotiationDiagnostic::new(
                    source.clone(),
                    CapabilityNegotiationDiagnosticCode::UnexpectedCapability,
                    Some(capability_id.clone()),
                    format!("descriptor declares unexpected capability {capability_id}"),
                )),
                (Some(manifest_capability), Some(descriptor_capability)) => {
                    if manifest_capability.kind() != descriptor_capability.kind() {
                        diagnostics.push(CapabilityNegotiationDiagnostic::new(
                            source.clone(),
                            CapabilityNegotiationDiagnosticCode::CapabilityKindMismatch,
                            Some(capability_id.clone()),
                            format!("descriptor capability {capability_id} has a different kind"),
                        ));
                    } else if manifest_capability.version() != descriptor_capability.version() {
                        diagnostics.push(CapabilityNegotiationDiagnostic::new(
                            source.clone(),
                            CapabilityNegotiationDiagnosticCode::CapabilityVersionMismatch,
                            Some(capability_id.clone()),
                            format!(
                                "descriptor capability {capability_id} has a different version"
                            ),
                        ));
                    }
                }
                (None, None) => unreachable!("capability ID was collected from one map"),
            }
        }

        if diagnostics.is_empty() {
            CapabilityNegotiationReport {
                capabilities: descriptor.capabilities().to_vec(),
                diagnostics,
            }
        } else {
            CapabilityNegotiationReport {
                capabilities: Vec::new(),
                diagnostics,
            }
        }
    }
}

fn parse_failure_report(
    source: String,
    error: McpServerCapabilityDescriptorParseError,
) -> CapabilityNegotiationReport {
    let (code, capability_id, message) = match &error {
        McpServerCapabilityDescriptorParseError::Malformed { .. } => (
            CapabilityNegotiationDiagnosticCode::MalformedDescriptor,
            None,
            "malformed MCP server capability descriptor".to_owned(),
        ),
        McpServerCapabilityDescriptorParseError::UnsupportedSchemaVersion { .. } => (
            CapabilityNegotiationDiagnosticCode::UnsupportedDescriptorVersion,
            None,
            error.to_string(),
        ),
        McpServerCapabilityDescriptorParseError::DuplicateCapability { capability } => (
            CapabilityNegotiationDiagnosticCode::DuplicateCapability,
            Some(capability.clone()),
            error.to_string(),
        ),
    };
    CapabilityNegotiationReport {
        capabilities: Vec::new(),
        diagnostics: vec![CapabilityNegotiationDiagnostic::new(
            source,
            code,
            capability_id,
            message,
        )],
    }
}

/// The result of one deterministic batch load.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginLoadReport {
    loaded_plugins: Vec<PluginId>,
    diagnostics: Vec<PluginLoadDiagnostic>,
    capability_entries: Vec<CapabilityAvailabilityEntry>,
}

impl PluginLoadReport {
    /// Returns successfully activated plugin identifiers in source order.
    pub fn loaded_plugins(&self) -> &[PluginId] {
        &self.loaded_plugins
    }

    /// Returns isolated load diagnostics in source order.
    pub fn diagnostics(&self) -> &[PluginLoadDiagnostic] {
        &self.diagnostics
    }

    /// Projects all attempted manifest capabilities as deterministic safe availability entries.
    #[must_use]
    pub fn capability_availability(&self) -> CapabilityAvailabilityProjection {
        CapabilityAvailabilityProjection::new(self.capability_entries.clone())
    }
}

struct LoadedPlugin {
    plugin: Box<dyn Plugin>,
}

/// A provider-free runtime that publishes capabilities only after activation.
pub struct PluginRuntime {
    compatibility: RuntimeCompatibility,
    registry: CapabilityRegistry,
    loaded: BTreeMap<PluginId, LoadedPlugin>,
}

impl PluginRuntime {
    /// Creates a runtime with explicit compatibility rules.
    pub fn new(compatibility: RuntimeCompatibility) -> Self {
        Self {
            compatibility,
            registry: CapabilityRegistry::new(),
            loaded: BTreeMap::new(),
        }
    }

    /// Creates a runtime for the current provider-free contract.
    pub fn current() -> Self {
        Self::new(RuntimeCompatibility::current())
    }

    /// Returns the runtime compatibility rules.
    pub fn compatibility(&self) -> &RuntimeCompatibility {
        &self.compatibility
    }

    /// Returns the atomically published capability registry.
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }

    /// Loads bundles in deterministic source order.
    pub fn load_all(
        &mut self,
        bundles: impl IntoIterator<Item = PluginBundle>,
    ) -> PluginLoadReport {
        let mut bundles: Vec<_> = bundles.into_iter().collect();
        bundles.sort_by(|left, right| {
            left.source()
                .cmp(right.source())
                .then_with(|| left.manifest().id().cmp(right.manifest().id()))
        });

        let mut report = PluginLoadReport {
            loaded_plugins: Vec::new(),
            diagnostics: Vec::new(),
            capability_entries: Vec::new(),
        };
        for bundle in bundles {
            let source = bundle.source;
            let manifest = bundle.manifest;
            let plugin_id = manifest.plugin_id().clone();
            if let Err(error) = self.compatibility.check_manifest(&manifest) {
                report.record_unavailable(
                    &manifest,
                    CapabilityCompatibility::Incompatible,
                    CapabilityDiagnosticCode::ManifestIncompatible,
                );
                report.diagnostics.push(PluginLoadDiagnostic::new(
                    source,
                    plugin_id,
                    LoadPhase::Compatibility,
                    error.to_string(),
                ));
                continue;
            }
            if let Err(error) = self.registry.can_register(&manifest) {
                report.record_unavailable(
                    &manifest,
                    CapabilityCompatibility::Incompatible,
                    CapabilityDiagnosticCode::RegistryRejected,
                );
                report.diagnostics.push(PluginLoadDiagnostic::new(
                    source,
                    plugin_id,
                    LoadPhase::Registry,
                    error.to_string(),
                ));
                continue;
            }

            let mut plugin = match bundle.factory.load(&manifest) {
                Ok(plugin) => plugin,
                Err(error) => {
                    report.record_unavailable(
                        &manifest,
                        CapabilityCompatibility::Compatible,
                        CapabilityDiagnosticCode::FactoryRejected,
                    );
                    report.diagnostics.push(PluginLoadDiagnostic::new(
                        source,
                        plugin_id,
                        LoadPhase::Factory,
                        error.to_string(),
                    ));
                    continue;
                }
            };
            if let Err(error) = plugin.initialize() {
                let _ = plugin.shutdown();
                report.record_unavailable(
                    &manifest,
                    CapabilityCompatibility::Compatible,
                    CapabilityDiagnosticCode::InitializationRejected,
                );
                report.diagnostics.push(PluginLoadDiagnostic::new(
                    source,
                    plugin_id,
                    LoadPhase::Initialize,
                    error.to_string(),
                ));
                continue;
            }
            if let Err(error) = plugin.activate() {
                let _ = plugin.shutdown();
                report.record_unavailable(
                    &manifest,
                    CapabilityCompatibility::Compatible,
                    CapabilityDiagnosticCode::ActivationRejected,
                );
                report.diagnostics.push(PluginLoadDiagnostic::new(
                    source,
                    plugin_id,
                    LoadPhase::Activate,
                    error.to_string(),
                ));
                continue;
            }

            if let Err(error) = self.registry.register(manifest.clone()) {
                let _ = plugin.deactivate();
                let _ = plugin.shutdown();
                report.record_unavailable(
                    &manifest,
                    CapabilityCompatibility::Incompatible,
                    CapabilityDiagnosticCode::RegistryRejected,
                );
                report.diagnostics.push(PluginLoadDiagnostic::new(
                    source,
                    plugin_id,
                    LoadPhase::Registry,
                    error.to_string(),
                ));
                continue;
            }
            self.loaded
                .insert(plugin_id.clone(), LoadedPlugin { plugin });
            report.record_available(&manifest);
            report.loaded_plugins.push(plugin_id);
        }
        report
    }

    /// Deactivates and shuts down all loaded plugins, clearing published capabilities.
    pub fn shutdown_all(&mut self) -> Vec<PluginLoadDiagnostic> {
        let loaded = std::mem::take(&mut self.loaded);
        let mut diagnostics = Vec::new();
        for (plugin_id, mut loaded_plugin) in loaded {
            if let Err(error) = loaded_plugin.plugin.deactivate() {
                diagnostics.push(PluginLoadDiagnostic::new(
                    plugin_id.to_string(),
                    plugin_id.clone(),
                    LoadPhase::Deactivate,
                    error.to_string(),
                ));
            }
            if let Err(error) = loaded_plugin.plugin.shutdown() {
                diagnostics.push(PluginLoadDiagnostic::new(
                    plugin_id.to_string(),
                    plugin_id.clone(),
                    LoadPhase::Shutdown,
                    error.to_string(),
                ));
            }
        }
        self.registry = CapabilityRegistry::new();
        diagnostics
    }
}

impl PluginLoadReport {
    fn record_available(&mut self, manifest: &PluginManifest) {
        self.capability_entries
            .extend(manifest.capabilities().iter().map(|capability| {
                CapabilityAvailabilityEntry::available(manifest.plugin_id(), capability)
            }));
    }

    fn record_unavailable(
        &mut self,
        manifest: &PluginManifest,
        compatibility: CapabilityCompatibility,
        diagnostic_code: CapabilityDiagnosticCode,
    ) {
        self.capability_entries
            .extend(manifest.capabilities().iter().map(|capability| {
                CapabilityAvailabilityEntry::unavailable(
                    manifest.plugin_id(),
                    capability,
                    compatibility,
                    diagnostic_code,
                )
            }));
    }
}

impl PluginLoadDiagnostic {
    fn new(source: String, plugin_id: PluginId, phase: LoadPhase, message: String) -> Self {
        Self {
            source,
            plugin_id,
            phase,
            message,
        }
    }
}

impl From<CompatibilityError> for PluginLoadError {
    fn from(error: CompatibilityError) -> Self {
        Self::new(error.to_string())
    }
}

impl From<RegistryError> for PluginLoadError {
    fn from(error: RegistryError) -> Self {
        Self::new(error.to_string())
    }
}
