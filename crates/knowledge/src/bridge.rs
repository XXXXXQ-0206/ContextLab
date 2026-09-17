//! Provider-free bridge from MCP capability resolution to redacted citation metadata.

use contextlab_mcp::{
    CapabilityId, CapabilityKind, CapabilityRegistry, CapabilityResolutionError, IdentifierError,
    PluginId, Version,
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::{
    CitationCompatibility, CitationCompatibilityRequirement, CitationSchemaVersion,
    KnowledgeCitation, KnowledgeCitationCapability, KnowledgeCitationCapabilityId, KnowledgeError,
    KnowledgeRetrievalId, KnowledgeScope,
};

/// Schema version for the stable Knowledge-to-Plugin citation projection.
pub const KNOWLEDGE_PLUGIN_CITATION_BRIDGE_SCHEMA_VERSION: Version = Version::new(1, 0, 0);

/// Typed requirements for resolving one plugin-provided citation resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KnowledgePluginCitationRequirement {
    capability_id: CapabilityId,
    minimum_version: Version,
    citation_compatibility: CitationCompatibilityRequirement,
}

impl KnowledgePluginCitationRequirement {
    /// Creates a validated requirement for an MCP resource capability and citation contract.
    pub fn new(
        capability_id: impl AsRef<str>,
        minimum_version: Version,
        citation_compatibility: CitationCompatibilityRequirement,
    ) -> Result<Self, IdentifierError> {
        Ok(Self {
            capability_id: CapabilityId::new(capability_id)?,
            minimum_version,
            citation_compatibility,
        })
    }

    /// Returns the stable MCP capability identifier.
    #[must_use]
    pub const fn capability_id(&self) -> &CapabilityId {
        &self.capability_id
    }

    /// Returns the minimum compatible MCP capability version.
    #[must_use]
    pub const fn minimum_version(&self) -> Version {
        self.minimum_version
    }

    /// Returns the exact citation and embedding compatibility requirement.
    #[must_use]
    pub const fn citation_compatibility(&self) -> &CitationCompatibilityRequirement {
        &self.citation_compatibility
    }
}

/// Fail-closed errors from Knowledge-to-Plugin capability resolution.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum KnowledgePluginCitationBridgeError {
    /// MCP capability lookup or version compatibility failed.
    #[error(transparent)]
    CapabilityResolution(#[from] CapabilityResolutionError),
    /// The capability resolved by ID and version but is not a readable resource.
    #[error(
        "capability {capability} has kind {actual:?}; knowledge citations require {expected:?}"
    )]
    WrongCapabilityKind {
        /// Stable capability identifier.
        capability: CapabilityId,
        /// Required capability category.
        expected: CapabilityKind,
        /// Registered capability category.
        actual: CapabilityKind,
    },
    /// The resolved registry entry unexpectedly has no owning plugin.
    #[error("capability owner is unavailable: {capability}")]
    MissingCapabilityOwner {
        /// Stable capability identifier.
        capability: CapabilityId,
    },
    /// Citation or embedding compatibility did not exactly match the consumer requirement.
    #[error("knowledge citation compatibility check failed: {0}")]
    CitationCompatibility(#[source] KnowledgeError),
}

/// Stable, provider-free metadata connecting one plugin resource to one citation result.
///
/// This projection deliberately omits source text, private queries and their fingerprints,
/// embedding vectors, provider configuration, and credentials.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct KnowledgePluginCitationProjection {
    schema_version: Version,
    plugin_id: PluginId,
    capability_id: CapabilityId,
    capability_kind: CapabilityKind,
    capability_version: Version,
    citation_capability_id: KnowledgeCitationCapabilityId,
    citation_capability_schema_version: CitationSchemaVersion,
    citation_compatibility: CitationCompatibility,
    retrieval_id: KnowledgeRetrievalId,
    retrieval_version: String,
    scope: Option<KnowledgeScope>,
    citations: Vec<KnowledgeCitation>,
}

impl<'de> Deserialize<'de> for KnowledgePluginCitationProjection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireProjection {
            schema_version: Version,
            plugin_id: PluginId,
            capability_id: CapabilityId,
            capability_kind: CapabilityKind,
            capability_version: Version,
            citation_capability_id: KnowledgeCitationCapabilityId,
            citation_capability_schema_version: CitationSchemaVersion,
            citation_compatibility: CitationCompatibility,
            retrieval_id: KnowledgeRetrievalId,
            retrieval_version: String,
            scope: Option<KnowledgeScope>,
            citations: Vec<KnowledgeCitation>,
        }

        let wire = WireProjection::deserialize(deserializer)?;
        if wire.schema_version != KNOWLEDGE_PLUGIN_CITATION_BRIDGE_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(
                "unsupported Knowledge-to-Plugin citation bridge schema",
            ));
        }
        if wire.capability_kind != CapabilityKind::Resource {
            return Err(serde::de::Error::custom(
                "Knowledge citation bridge capabilities must be resources",
            ));
        }
        if wire.citation_capability_schema_version.as_str() != "knowledge-citation-capability-v1" {
            return Err(serde::de::Error::custom(
                "unsupported source citation capability schema",
            ));
        }
        if wire
            .citations
            .windows(2)
            .any(|pair| pair[0].chunk_id() >= pair[1].chunk_id())
        {
            return Err(serde::de::Error::custom(
                "citations must be in strictly ascending chunk-identifier order",
            ));
        }

        Ok(Self {
            schema_version: wire.schema_version,
            plugin_id: wire.plugin_id,
            capability_id: wire.capability_id,
            capability_kind: wire.capability_kind,
            capability_version: wire.capability_version,
            citation_capability_id: wire.citation_capability_id,
            citation_capability_schema_version: wire.citation_capability_schema_version,
            citation_compatibility: wire.citation_compatibility,
            retrieval_id: wire.retrieval_id,
            retrieval_version: wire.retrieval_version,
            scope: wire.scope,
            citations: wire.citations,
        })
    }
}

impl KnowledgePluginCitationProjection {
    /// Returns this bridge projection's explicit schema version.
    #[must_use]
    pub const fn schema_version(&self) -> Version {
        self.schema_version
    }

    /// Returns the owning plugin's stable identifier.
    #[must_use]
    pub fn plugin_id(&self) -> &str {
        self.plugin_id.as_str()
    }

    /// Returns the resolved capability's stable identifier.
    #[must_use]
    pub fn capability_id(&self) -> &str {
        self.capability_id.as_str()
    }

    /// Returns the resolved typed capability category.
    #[must_use]
    pub const fn capability_kind(&self) -> CapabilityKind {
        self.capability_kind
    }

    /// Returns the resolved capability contract version.
    #[must_use]
    pub const fn capability_version(&self) -> Version {
        self.capability_version
    }

    /// Returns the stable Knowledge citation capability identity.
    #[must_use]
    pub const fn citation_capability_id(&self) -> KnowledgeCitationCapabilityId {
        self.citation_capability_id
    }

    /// Returns the Knowledge citation capability record schema version.
    #[must_use]
    pub const fn citation_capability_schema_version(&self) -> &CitationSchemaVersion {
        &self.citation_capability_schema_version
    }

    /// Returns redacted citation and embedding compatibility metadata.
    #[must_use]
    pub const fn citation_compatibility(&self) -> &CitationCompatibility {
        &self.citation_compatibility
    }

    /// Returns the portable citation schema version.
    #[must_use]
    pub const fn citation_schema_version(&self) -> &CitationSchemaVersion {
        self.citation_compatibility.citation_schema_version()
    }

    /// Returns the stable retrieval identity without the private query or its fingerprint.
    #[must_use]
    pub const fn retrieval_id(&self) -> KnowledgeRetrievalId {
        self.retrieval_id
    }

    /// Returns the deterministic retrieval algorithm version.
    #[must_use]
    pub fn retrieval_version(&self) -> &str {
        &self.retrieval_version
    }

    /// Returns the optional stable knowledge scope.
    #[must_use]
    pub const fn scope(&self) -> Option<KnowledgeScope> {
        self.scope
    }

    /// Returns canonical citation metadata in ascending chunk-identifier order.
    #[must_use]
    pub fn citations(&self) -> &[KnowledgeCitation] {
        &self.citations
    }

    /// Confirms that raw private source or chunk content is absent by construction.
    #[must_use]
    pub const fn contains_raw_private_content(&self) -> bool {
        false
    }

    /// Confirms that embedding vector values are absent by construction.
    #[must_use]
    pub const fn contains_raw_vectors(&self) -> bool {
        false
    }

    /// Confirms that both raw queries and query fingerprints are absent by construction.
    #[must_use]
    pub const fn contains_raw_query(&self) -> bool {
        false
    }

    /// Confirms that provider secrets and provider configuration are absent by construction.
    #[must_use]
    pub const fn contains_provider_secrets(&self) -> bool {
        false
    }
}

/// Stateless adapter for version-checked MCP registry resolution and Knowledge metadata projection.
pub struct KnowledgePluginCitationBridge;

impl KnowledgePluginCitationBridge {
    /// Resolves and validates a typed resource capability without invoking any provider.
    pub fn resolve(
        registry: &CapabilityRegistry,
        citation_capability: &KnowledgeCitationCapability,
        requirement: &KnowledgePluginCitationRequirement,
    ) -> Result<KnowledgePluginCitationProjection, KnowledgePluginCitationBridgeError> {
        let descriptor = registry.resolve_capability(
            requirement.capability_id.as_str(),
            requirement.minimum_version,
        )?;
        if descriptor.kind() != CapabilityKind::Resource {
            return Err(KnowledgePluginCitationBridgeError::WrongCapabilityKind {
                capability: descriptor.capability_id().clone(),
                expected: CapabilityKind::Resource,
                actual: descriptor.kind(),
            });
        }
        citation_capability
            .compatibility()
            .require_compatible(&requirement.citation_compatibility)
            .map_err(KnowledgePluginCitationBridgeError::CitationCompatibility)?;

        let plugin_id = registry
            .capability_owner(descriptor.id())
            .cloned()
            .ok_or_else(
                || KnowledgePluginCitationBridgeError::MissingCapabilityOwner {
                    capability: descriptor.capability_id().clone(),
                },
            )?;
        let mut citations = citation_capability
            .sources()
            .iter()
            .map(|source| source.citation().clone())
            .collect::<Vec<_>>();
        citations.sort_by_key(KnowledgeCitation::chunk_id);

        Ok(KnowledgePluginCitationProjection {
            schema_version: KNOWLEDGE_PLUGIN_CITATION_BRIDGE_SCHEMA_VERSION,
            plugin_id,
            capability_id: descriptor.capability_id().clone(),
            capability_kind: descriptor.kind(),
            capability_version: descriptor.version(),
            citation_capability_id: citation_capability.id(),
            citation_capability_schema_version: citation_capability.schema_version().clone(),
            citation_compatibility: citation_capability.compatibility().clone(),
            retrieval_id: citation_capability.read_record().retrieval_id(),
            retrieval_version: citation_capability
                .read_record()
                .retrieval_version()
                .to_owned(),
            scope: citation_capability.read_record().scope(),
            citations,
        })
    }
}
