//! Provider-free bridge from redacted knowledge citations to memory replay facts.

use contextlab_memory::{
    MemoryCapabilitySchemaVersion, MemoryId, MemoryReplayState, MemoryRetentionCapability,
    MemoryRetentionCapabilityId, MemoryScope, RetentionDecision, RetentionPolicyVersion,
    TimelineVersion,
};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    CitationSchemaVersion, KnowledgeCitation, KnowledgeLocalCitationProjectionId,
    KnowledgeLocalCitationProjectionV1, KnowledgeRetrievalId,
};

const KNOWLEDGE_MEMORY_LOCAL_REPLAY_NAMESPACE: Uuid =
    Uuid::from_u128(0x325a_50f1_9b23_55c8_8566_2a4b_84a3_cfd0);
const KNOWLEDGE_MEMORY_LOCAL_REPLAY_SCHEMA_VERSION: &str = "knowledge-memory-local-replay-v2";

/// Stable UUID v5 identifier for a redacted Knowledge-to-Memory replay projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KnowledgeMemoryReplayProjectionId(Uuid);

impl KnowledgeMemoryReplayProjectionId {
    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

/// Structured errors from fail-closed Knowledge-to-Memory bridge validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeMemoryReplayBridgeError {
    /// The supplied local citation projection uses an unsupported schema version.
    #[error("citation projection schema mismatch: expected {expected:?}, received {actual:?}")]
    CitationProjectionSchemaMismatch {
        /// Bridge-supported local citation projection schema version.
        expected: CitationSchemaVersion,
        /// Schema version received from the citation projection.
        actual: CitationSchemaVersion,
    },
    /// The supplied memory retention capability uses an unsupported schema version.
    #[error("memory capability schema mismatch: expected {expected:?}, received {actual:?}")]
    MemoryCapabilitySchemaMismatch {
        /// Bridge-supported memory retention capability schema version.
        expected: MemoryCapabilitySchemaVersion,
        /// Schema version received from the memory capability.
        actual: MemoryCapabilitySchemaVersion,
    },
    /// The local citation projection is not bound to one exact knowledge scope.
    #[error("knowledge citation projection must declare an exact scope")]
    CitationScopeRequired,
}

/// Redacted local replay facts composed from existing Knowledge and Memory records.
///
/// The projection deliberately excludes document and memory content, retrieval query
/// text, embedding vectors, provider configuration, and credentials. Citation
/// `content_fingerprint` values are retained only as stable redacted metadata and
/// never contain source bodies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnowledgeMemoryReplayProjection {
    id: KnowledgeMemoryReplayProjectionId,
    schema_version: CitationSchemaVersion,
    citation_projection_id: KnowledgeLocalCitationProjectionId,
    citation_projection_schema_version: CitationSchemaVersion,
    knowledge_scope: crate::KnowledgeScope,
    retrieval_id: KnowledgeRetrievalId,
    retrieval_version: String,
    citations: Vec<KnowledgeCitation>,
    memory_id: MemoryId,
    memory_capability_id: MemoryRetentionCapabilityId,
    memory_scope: MemoryScope,
    memory_timeline_version: TimelineVersion,
    memory_capability_schema_version: MemoryCapabilitySchemaVersion,
    retention_policy_version: RetentionPolicyVersion,
    retention_decision: RetentionDecision,
    replay_state: MemoryReplayState,
}

impl<'de> Deserialize<'de> for KnowledgeMemoryReplayProjection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireProjection {
            id: KnowledgeMemoryReplayProjectionId,
            schema_version: CitationSchemaVersion,
            citation_projection_id: KnowledgeLocalCitationProjectionId,
            citation_projection_schema_version: CitationSchemaVersion,
            knowledge_scope: crate::KnowledgeScope,
            retrieval_id: KnowledgeRetrievalId,
            retrieval_version: String,
            citations: Vec<KnowledgeCitation>,
            memory_id: MemoryId,
            memory_capability_id: MemoryRetentionCapabilityId,
            memory_scope: MemoryScope,
            memory_timeline_version: TimelineVersion,
            memory_capability_schema_version: MemoryCapabilitySchemaVersion,
            retention_policy_version: RetentionPolicyVersion,
            retention_decision: RetentionDecision,
            replay_state: MemoryReplayState,
        }

        let wire = WireProjection::deserialize(deserializer)?;
        let projection = Self {
            id: wire.id,
            schema_version: wire.schema_version,
            citation_projection_id: wire.citation_projection_id,
            citation_projection_schema_version: wire.citation_projection_schema_version,
            knowledge_scope: wire.knowledge_scope,
            retrieval_id: wire.retrieval_id,
            retrieval_version: wire.retrieval_version,
            citations: wire.citations,
            memory_id: wire.memory_id,
            memory_capability_id: wire.memory_capability_id,
            memory_scope: wire.memory_scope,
            memory_timeline_version: wire.memory_timeline_version,
            memory_capability_schema_version: wire.memory_capability_schema_version,
            retention_policy_version: wire.retention_policy_version,
            retention_decision: wire.retention_decision,
            replay_state: wire.replay_state,
        };
        projection
            .validate_deserialized()
            .map_err(serde::de::Error::custom)?;
        Ok(projection)
    }
}

impl KnowledgeMemoryReplayProjection {
    fn validate_deserialized(&self) -> Result<(), &'static str> {
        if self.schema_version.as_str() != KNOWLEDGE_MEMORY_LOCAL_REPLAY_SCHEMA_VERSION {
            return Err("unsupported Knowledge-to-Memory replay projection schema");
        }
        if self.citation_projection_schema_version.as_str()
            != "knowledge-local-citation-projection-v1"
        {
            return Err("unsupported source citation projection schema");
        }
        if self.memory_capability_schema_version.as_str() != "memory-retention-capability-v1" {
            return Err("unsupported source memory capability schema");
        }
        if self.knowledge_scope.as_uuid().is_nil() {
            return Err("knowledge scope must be a non-nil exact scope");
        }
        if self
            .citations
            .windows(2)
            .any(|pair| pair[0].chunk_id() >= pair[1].chunk_id())
        {
            return Err("citations must be in strictly ascending chunk-identifier order");
        }
        if matches!(
            (self.replay_state, self.retention_decision),
            (MemoryReplayState::Forgotten, decision)
                if decision != RetentionDecision::ExpiredForgotten
        ) || matches!(
            (self.replay_state, self.retention_decision),
            (
                MemoryReplayState::Active,
                RetentionDecision::ExpiredForgotten
            )
        ) {
            return Err("memory replay state is inconsistent with the retention decision");
        }
        if self.id != replay_projection_id(self) {
            return Err("replay projection ID does not match its source facts");
        }
        Ok(())
    }

    /// Returns this bridge projection's explicit schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &CitationSchemaVersion {
        &self.schema_version
    }

    /// Returns the deterministic bridge projection identity.
    #[must_use]
    pub const fn id(&self) -> KnowledgeMemoryReplayProjectionId {
        self.id
    }

    /// Returns the source local citation projection identifier.
    #[must_use]
    pub const fn citation_projection_id(&self) -> KnowledgeLocalCitationProjectionId {
        self.citation_projection_id
    }

    /// Returns the source local citation projection schema version.
    #[must_use]
    pub const fn citation_projection_schema_version(&self) -> &CitationSchemaVersion {
        &self.citation_projection_schema_version
    }

    /// Returns the exact knowledge scope bound to this replay projection.
    #[must_use]
    pub const fn knowledge_scope(&self) -> crate::KnowledgeScope {
        self.knowledge_scope
    }

    /// Returns the source deterministic retrieval identifier.
    #[must_use]
    pub const fn retrieval_id(&self) -> KnowledgeRetrievalId {
        self.retrieval_id
    }

    /// Returns the source retrieval algorithm version.
    #[must_use]
    pub fn retrieval_version(&self) -> &str {
        &self.retrieval_version
    }

    /// Returns citations in ascending stable chunk-identifier order.
    #[must_use]
    pub fn citations(&self) -> &[KnowledgeCitation] {
        &self.citations
    }

    /// Returns the stable memory timeline identifier.
    #[must_use]
    pub const fn memory_id(&self) -> MemoryId {
        self.memory_id
    }

    /// Returns the exact source memory retention capability identity.
    #[must_use]
    pub const fn memory_capability_id(&self) -> MemoryRetentionCapabilityId {
        self.memory_capability_id
    }

    /// Returns the memory scope boundary.
    #[must_use]
    pub const fn memory_scope(&self) -> MemoryScope {
        self.memory_scope
    }

    /// Returns the replayed memory timeline version.
    #[must_use]
    pub const fn memory_timeline_version(&self) -> TimelineVersion {
        self.memory_timeline_version
    }

    /// Returns the exact source memory capability schema version.
    #[must_use]
    pub const fn memory_capability_schema_version(&self) -> &MemoryCapabilitySchemaVersion {
        &self.memory_capability_schema_version
    }

    /// Returns the source retention policy version.
    #[must_use]
    pub const fn retention_policy_version(&self) -> &RetentionPolicyVersion {
        &self.retention_policy_version
    }

    /// Returns the retention decision already calculated by Memory.
    #[must_use]
    pub const fn retention_decision(&self) -> RetentionDecision {
        self.retention_decision
    }

    /// Returns the replay state already calculated by Memory.
    #[must_use]
    pub const fn replay_state(&self) -> MemoryReplayState {
        self.replay_state
    }

    /// Confirms that raw knowledge content is absent by construction.
    #[must_use]
    pub const fn contains_raw_knowledge_content(&self) -> bool {
        false
    }

    /// Confirms that raw memory content is absent by construction.
    #[must_use]
    pub const fn contains_raw_memory_content(&self) -> bool {
        false
    }

    /// Confirms that embedding vectors are absent by construction.
    #[must_use]
    pub const fn contains_raw_vectors(&self) -> bool {
        false
    }

    /// Confirms that raw retrieval queries and their fingerprints are absent by construction.
    #[must_use]
    pub const fn contains_raw_query(&self) -> bool {
        false
    }
}

/// Stateless local composition of existing redacted citation and memory replay records.
pub struct KnowledgeMemoryReplayBridge;

impl KnowledgeMemoryReplayBridge {
    /// Projects provider-free replay metadata without reading or storing private payloads.
    pub fn project(
        citations: &KnowledgeLocalCitationProjectionV1,
        retention: &MemoryRetentionCapability,
    ) -> Result<KnowledgeMemoryReplayProjection, KnowledgeMemoryReplayBridgeError> {
        let expected_citation_schema =
            CitationSchemaVersion::new("knowledge-local-citation-projection-v1")
                .expect("supported citation projection schema is a non-empty fixed constant");
        if citations.schema_version() != &expected_citation_schema {
            return Err(
                KnowledgeMemoryReplayBridgeError::CitationProjectionSchemaMismatch {
                    expected: expected_citation_schema,
                    actual: citations.schema_version().clone(),
                },
            );
        }
        let Some(knowledge_scope) = citations.scope() else {
            return Err(KnowledgeMemoryReplayBridgeError::CitationScopeRequired);
        };
        let expected_memory_schema =
            MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1")
                .expect("supported memory capability schema is a non-empty fixed constant");
        if retention.schema_version() != &expected_memory_schema {
            return Err(
                KnowledgeMemoryReplayBridgeError::MemoryCapabilitySchemaMismatch {
                    expected: expected_memory_schema,
                    actual: retention.schema_version().clone(),
                },
            );
        }

        let mut canonical_citations = citations.citations().to_vec();
        canonical_citations.sort_by_key(KnowledgeCitation::chunk_id);
        let mut projection = KnowledgeMemoryReplayProjection {
            id: KnowledgeMemoryReplayProjectionId(Uuid::nil()),
            schema_version: CitationSchemaVersion::new(
                KNOWLEDGE_MEMORY_LOCAL_REPLAY_SCHEMA_VERSION,
            )
            .expect("bridge schema version is a non-empty fixed constant"),
            citation_projection_id: citations.id(),
            citation_projection_schema_version: citations.schema_version().clone(),
            knowledge_scope,
            retrieval_id: citations.retrieval_id(),
            retrieval_version: citations.retrieval_version().to_owned(),
            citations: canonical_citations,
            memory_id: retention.memory_id(),
            memory_capability_id: retention.id(),
            memory_scope: retention.scope(),
            memory_timeline_version: retention.timeline_version(),
            memory_capability_schema_version: retention.schema_version().clone(),
            retention_policy_version: retention.policy_version().clone(),
            retention_decision: retention.decision(),
            replay_state: retention.replay_state(),
        };
        projection.id = replay_projection_id(&projection);
        Ok(projection)
    }
}

fn replay_projection_id(
    projection: &KnowledgeMemoryReplayProjection,
) -> KnowledgeMemoryReplayProjectionId {
    let mut identity = Vec::new();
    append_identity_text(&mut identity, projection.schema_version.as_str());
    append_identity_uuid(&mut identity, projection.citation_projection_id.as_uuid());
    append_identity_text(
        &mut identity,
        projection.citation_projection_schema_version.as_str(),
    );
    append_identity_uuid(&mut identity, projection.knowledge_scope.as_uuid());
    append_identity_uuid(&mut identity, projection.retrieval_id.as_uuid());
    append_identity_text(&mut identity, &projection.retrieval_version);
    append_identity_u64(
        &mut identity,
        u64::try_from(projection.citations.len()).expect("citation count fits u64"),
    );
    for citation in &projection.citations {
        append_identity_uuid(&mut identity, citation.document_id().as_uuid());
        append_identity_uuid(&mut identity, citation.document_revision_id().as_uuid());
        append_identity_uuid(&mut identity, citation.chunk_id().as_uuid());
        append_identity_text(&mut identity, citation.source_version().as_str());
        append_identity_text(&mut identity, citation.chunking_version().as_str());
        append_identity_u64(&mut identity, u64::from(citation.ordinal()));
        append_identity_u64(
            &mut identity,
            u64::try_from(citation.range().start_byte()).expect("citation start byte fits u64"),
        );
        append_identity_u64(
            &mut identity,
            u64::try_from(citation.range().end_byte()).expect("citation end byte fits u64"),
        );
        append_identity_text(&mut identity, citation.content_fingerprint());
    }
    append_identity_uuid(&mut identity, projection.memory_id.as_uuid());
    append_identity_uuid(&mut identity, projection.memory_capability_id.as_uuid());
    append_identity_uuid(&mut identity, projection.memory_scope.as_uuid());
    append_identity_u64(&mut identity, projection.memory_timeline_version.get());
    append_identity_text(
        &mut identity,
        projection.memory_capability_schema_version.as_str(),
    );
    append_identity_text(&mut identity, projection.retention_policy_version.as_str());
    append_identity_text(
        &mut identity,
        retention_decision_identity(projection.retention_decision),
    );
    append_identity_text(
        &mut identity,
        replay_state_identity(projection.replay_state),
    );
    KnowledgeMemoryReplayProjectionId(Uuid::new_v5(
        &KNOWLEDGE_MEMORY_LOCAL_REPLAY_NAMESPACE,
        &identity,
    ))
}

fn append_identity_uuid(identity: &mut Vec<u8>, value: Uuid) {
    append_identity_component(identity, value.as_bytes());
}

fn append_identity_text(identity: &mut Vec<u8>, value: &str) {
    append_identity_component(identity, value.as_bytes());
}

fn append_identity_u64(identity: &mut Vec<u8>, value: u64) {
    append_identity_component(identity, &value.to_be_bytes());
}

fn append_identity_component(identity: &mut Vec<u8>, value: &[u8]) {
    let length = u64::try_from(value.len()).expect("identity component length fits u64");
    identity.extend_from_slice(&length.to_be_bytes());
    identity.extend_from_slice(value);
}

const fn retention_decision_identity(decision: RetentionDecision) -> &'static str {
    match decision {
        RetentionDecision::RetainedPinned => "retained_pinned",
        RetentionDecision::RetainedFresh => "retained_fresh",
        RetentionDecision::RetainedImportant => "retained_important",
        RetentionDecision::ExpiredForgotten => "expired_forgotten",
        RetentionDecision::ExpiredLowImportance => "expired_low_importance",
    }
}

const fn replay_state_identity(state: MemoryReplayState) -> &'static str {
    match state {
        MemoryReplayState::Active => "active",
        MemoryReplayState::Forgotten => "forgotten",
    }
}
