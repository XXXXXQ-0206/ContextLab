//! Provider-free, Context-scoped composition of Knowledge and Memory read facts.

use contextlab_context_core::ContextId;
use contextlab_memory::{MemoryRetentionCapability, MemoryScope};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    CitationSchemaVersion, KnowledgeLocalCitationProjectionV1, KnowledgeMemoryReplayBridge,
    KnowledgeMemoryReplayBridgeError, KnowledgeMemoryReplayProjection, KnowledgeScope,
};

const KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_NAMESPACE: Uuid =
    Uuid::from_u128(0x9d73_46e4_32c1_5d71_9f05_2f74_6e2e_4f2d);
const KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_SCHEMA_VERSION: &str =
    "knowledge-memory-context-projection-v1";

/// Stable UUID v5 identifier for one Context-scoped Knowledge/Memory projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KnowledgeMemoryContextProjectionId(Uuid);

impl KnowledgeMemoryContextProjectionId {
    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

/// Errors produced while composing or validating a Context-scoped projection.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeMemoryContextProjectionError {
    /// The source Knowledge projection could not be composed with Memory facts.
    #[error(transparent)]
    Replay(#[from] KnowledgeMemoryReplayBridgeError),
    /// The Knowledge projection was not explicitly scoped to this Context.
    #[error("knowledge projection is outside the requested Context scope")]
    KnowledgeScopeMismatch {
        /// Requested Context identity.
        context_id: ContextId,
        /// Scope carried by the Knowledge projection, if any.
        knowledge_scope: Option<KnowledgeScope>,
    },
    /// The Memory capability was not scoped to this Context.
    #[error("memory capability is outside the requested Context scope")]
    MemoryScopeMismatch {
        /// Requested Context identity.
        context_id: ContextId,
        /// Scope carried by the Memory capability.
        memory_scope: MemoryScope,
    },
}

/// Exact, versioned, redacted Knowledge/Memory facts for one Context.
///
/// The context UUID is the scope root. Knowledge and Memory adapters must use
/// the existing `context:<ContextId>` stable scope key; any other scope is rejected.
/// The nested replay projection contains only identifiers, versions, fingerprints,
/// decisions, and citations. It never contains source bodies, query text, vectors,
/// provider configuration, or credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnowledgeMemoryContextProjectionV1 {
    id: KnowledgeMemoryContextProjectionId,
    schema_version: CitationSchemaVersion,
    context_id: ContextId,
    knowledge_scope: KnowledgeScope,
    memory_scope: MemoryScope,
    replay_projection: KnowledgeMemoryReplayProjection,
}

impl<'de> Deserialize<'de> for KnowledgeMemoryContextProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireProjection {
            id: KnowledgeMemoryContextProjectionId,
            schema_version: CitationSchemaVersion,
            context_id: ContextId,
            knowledge_scope: KnowledgeScope,
            memory_scope: MemoryScope,
            replay_projection: KnowledgeMemoryReplayProjection,
        }

        let wire = WireProjection::deserialize(deserializer)?;
        let projection = Self {
            id: wire.id,
            schema_version: wire.schema_version,
            context_id: wire.context_id,
            knowledge_scope: wire.knowledge_scope,
            memory_scope: wire.memory_scope,
            replay_projection: wire.replay_projection,
        };
        projection
            .validate_deserialized()
            .map_err(serde::de::Error::custom)?;
        Ok(projection)
    }
}

impl KnowledgeMemoryContextProjectionV1 {
    fn validate_deserialized(&self) -> Result<(), &'static str> {
        if self.schema_version.as_str() != KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_SCHEMA_VERSION {
            return Err("unsupported Knowledge/Memory Context projection schema");
        }
        let expected_knowledge_scope = knowledge_scope_for_context(self.context_id);
        if self.knowledge_scope != expected_knowledge_scope {
            return Err("knowledge scope is not bound to the Context");
        }
        let expected_memory_scope = memory_scope_for_context(self.context_id);
        if self.memory_scope != expected_memory_scope {
            return Err("memory scope is not bound to the Context");
        }
        if self.replay_projection.memory_scope() != self.memory_scope {
            return Err("replay memory scope disagrees with the Context projection");
        }
        if self.replay_projection.knowledge_scope() != self.knowledge_scope {
            return Err("replay knowledge scope disagrees with the Context projection");
        }
        if self.id != context_projection_id(self) {
            return Err("Context projection ID does not match its source facts");
        }
        Ok(())
    }

    /// Returns the projection's stable identity.
    #[must_use]
    pub const fn id(&self) -> KnowledgeMemoryContextProjectionId {
        self.id
    }

    /// Returns the exact V1 schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &CitationSchemaVersion {
        &self.schema_version
    }

    /// Returns the Context scope root.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the Knowledge scope bound to the Context UUID.
    #[must_use]
    pub const fn knowledge_scope(&self) -> KnowledgeScope {
        self.knowledge_scope
    }

    /// Returns the Memory scope bound to the Context UUID.
    #[must_use]
    pub const fn memory_scope(&self) -> MemoryScope {
        self.memory_scope
    }

    /// Returns the nested redacted Knowledge/Memory replay facts.
    #[must_use]
    pub const fn replay_projection(&self) -> &KnowledgeMemoryReplayProjection {
        &self.replay_projection
    }

    /// Returns canonical citations from the nested replay facts.
    #[must_use]
    pub fn citations(&self) -> &[crate::KnowledgeCitation] {
        self.replay_projection.citations()
    }

    /// Confirms that raw Knowledge content is absent by construction.
    #[must_use]
    pub const fn contains_raw_knowledge_content(&self) -> bool {
        false
    }

    /// Confirms that raw Memory content is absent by construction.
    #[must_use]
    pub const fn contains_raw_memory_content(&self) -> bool {
        false
    }

    /// Confirms that embedding vector values are absent by construction.
    #[must_use]
    pub const fn contains_raw_vectors(&self) -> bool {
        false
    }

    /// Confirms that retrieval query text and fingerprints are absent by construction.
    #[must_use]
    pub const fn contains_raw_query(&self) -> bool {
        false
    }

    /// Confirms that provider credentials and configuration are absent by construction.
    #[must_use]
    pub const fn contains_provider_secrets(&self) -> bool {
        false
    }
}

/// Stateless composition of exact Context-scoped Knowledge and Memory facts.
pub struct KnowledgeMemoryContextProjectionBridge;

impl KnowledgeMemoryContextProjectionBridge {
    /// Projects redacted provider-free facts for one exact Context.
    pub fn project(
        context_id: ContextId,
        citations: &KnowledgeLocalCitationProjectionV1,
        retention: &MemoryRetentionCapability,
    ) -> Result<KnowledgeMemoryContextProjectionV1, KnowledgeMemoryContextProjectionError> {
        let knowledge_scope = knowledge_scope_for_context(context_id);
        if citations.scope() != Some(knowledge_scope) {
            return Err(
                KnowledgeMemoryContextProjectionError::KnowledgeScopeMismatch {
                    context_id,
                    knowledge_scope: citations.scope(),
                },
            );
        }
        let memory_scope = memory_scope_for_context(context_id);
        if retention.scope() != memory_scope {
            return Err(KnowledgeMemoryContextProjectionError::MemoryScopeMismatch {
                context_id,
                memory_scope: retention.scope(),
            });
        }

        let replay_projection = KnowledgeMemoryReplayBridge::project(citations, retention)?;
        let mut projection = KnowledgeMemoryContextProjectionV1 {
            id: KnowledgeMemoryContextProjectionId(Uuid::nil()),
            schema_version: CitationSchemaVersion::new(
                KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_SCHEMA_VERSION,
            )
            .expect("projection schema version is a non-empty fixed constant"),
            context_id,
            knowledge_scope,
            memory_scope,
            replay_projection,
        };
        projection.id = context_projection_id(&projection);
        Ok(projection)
    }
}

fn knowledge_scope_for_context(context_id: ContextId) -> KnowledgeScope {
    KnowledgeScope::from_stable_key(format!("context:{context_id}"))
        .expect("Context UUID produces a valid Knowledge scope key")
}

fn memory_scope_for_context(context_id: ContextId) -> MemoryScope {
    MemoryScope::from_stable_key(format!("context:{context_id}"))
        .expect("Context UUID produces a valid Memory scope key")
}

fn context_projection_id(
    projection: &KnowledgeMemoryContextProjectionV1,
) -> KnowledgeMemoryContextProjectionId {
    let identity = format!(
        "{}\\0{}\\0{}\\0{}\\0{}",
        projection.schema_version.as_str(),
        projection.context_id,
        projection.knowledge_scope,
        projection.memory_scope,
        projection.replay_projection.id().as_uuid(),
    );
    KnowledgeMemoryContextProjectionId(Uuid::new_v5(
        &KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_NAMESPACE,
        identity.as_bytes(),
    ))
}
