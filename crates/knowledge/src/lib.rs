//! Provider-free knowledge ingestion, chunking, retrieval, and citations.
//!
//! Raw document bodies are accepted only at ingestion time. The deterministic
//! local repository retains chunk embeddings and verifiable citation metadata,
//! then emits privacy-safe read records without raw query or document content.

#![forbid(unsafe_code)]

mod bridge;
mod context_projection;
mod memory_replay;

pub use bridge::{
    KNOWLEDGE_PLUGIN_CITATION_BRIDGE_SCHEMA_VERSION, KnowledgePluginCitationBridge,
    KnowledgePluginCitationBridgeError, KnowledgePluginCitationProjection,
    KnowledgePluginCitationRequirement,
};
pub use context_projection::{
    KnowledgeMemoryContextProjectionBridge, KnowledgeMemoryContextProjectionError,
    KnowledgeMemoryContextProjectionId, KnowledgeMemoryContextProjectionV1,
};
pub use memory_replay::{
    KnowledgeMemoryReplayBridge, KnowledgeMemoryReplayBridgeError, KnowledgeMemoryReplayProjection,
    KnowledgeMemoryReplayProjectionId,
};

use contextlab_embedding::{
    Embedding, EmbeddingCapability, EmbeddingCapabilityPort, EmbeddingCapabilityRequirement,
    EmbeddingError, EmbeddingProvenance, EmbeddingProvider, EmbeddingText, VectorIndex,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

const KNOWLEDGE_SCOPE_NAMESPACE: Uuid = Uuid::from_u128(0x1c2f_6eb7_8c58_5e48_b815_30a1_365c_7418);
const KNOWLEDGE_DOCUMENT_NAMESPACE: Uuid =
    Uuid::from_u128(0x4dd3_43fa_40e7_59d8_90b2_eaeb_0bd3_06d4);
const KNOWLEDGE_REVISION_NAMESPACE: Uuid =
    Uuid::from_u128(0xa1b4_5df2_6a5f_5cbc_a5d4_2942_3e12_5f2e);
const KNOWLEDGE_CHUNK_NAMESPACE: Uuid = Uuid::from_u128(0x5a38_5196_1829_5f60_9298_9eaa_bf18_1d91);
const KNOWLEDGE_RETRIEVAL_NAMESPACE: Uuid =
    Uuid::from_u128(0x7111_4be3_3e4d_57ee_bcd0_0805_0110_a5e8);
const KNOWLEDGE_CITATION_CAPABILITY_NAMESPACE: Uuid =
    Uuid::from_u128(0xf4ae_2c62_5ea7_50ad_9bc7_f96f_5f79_8467);
const KNOWLEDGE_LOCAL_CITATION_PROJECTION_NAMESPACE: Uuid =
    Uuid::from_u128(0x96ed_014d_c1da_5d27_a1c0_c7d3_7e38_d7b6);
const KNOWLEDGE_LOCAL_CITATION_PROJECTION_SCHEMA_VERSION: &str =
    "knowledge-local-citation-projection-v1";
const MAX_LABEL_CHARS: usize = 240;
const MAX_EXTERNAL_KEY_CHARS: usize = 512;
const MAX_CONTENT_CHARS: usize = 1_000_000;
const MAX_RESULT_LIMIT: usize = 100;

/// Errors produced by knowledge domain constructors and local adapters.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeError {
    /// A required field was blank after trimming.
    #[error("{field} must not be empty")]
    Empty {
        /// Stable field name.
        field: &'static str,
    },
    /// A bounded field exceeded its maximum character length.
    #[error("{field} must be at most {max} characters")]
    TooLong {
        /// Stable field name.
        field: &'static str,
        /// Maximum accepted character length.
        max: usize,
    },
    /// Chunk size or overlap was invalid.
    #[error("chunk overlap must be smaller than the maximum chunk size")]
    InvalidChunkOverlap,
    /// A retrieval request did not request a bounded positive result limit.
    #[error("retrieval limit must be between 1 and {max}")]
    InvalidRetrievalLimit {
        /// Maximum supported request size.
        max: usize,
    },
    /// A repository lock was poisoned and the read cannot be trusted.
    #[error("knowledge repository state is unavailable")]
    RepositoryUnavailable,
    /// An embedding operation failed.
    #[error(transparent)]
    Embedding(#[from] EmbeddingError),
    /// A requested citation could not be resolved from deterministic index state.
    #[error("retrieval index contained an unknown chunk source")]
    UnknownIndexedChunk,
    /// A consumer required a citation schema or embedding capability that was unavailable.
    #[error("citation capability does not satisfy the requested compatibility requirement")]
    CitationCompatibilityMismatch {
        /// Consumer-declared compatibility requirement.
        expected: Box<CitationCompatibilityRequirement>,
        /// Capability metadata available from this retrieval.
        actual: Box<CitationCompatibility>,
    },
    /// Citation provenance disagreed with the repository's advertised embedding capability.
    #[error("citation provenance embedding metadata does not match the active capability")]
    CitationSourceEmbeddingMismatch {
        /// Citation chunk with incompatible provenance metadata.
        chunk_id: KnowledgeChunkId,
    },
}

macro_rules! stable_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(Uuid);

        impl $name {
            /// Wraps an existing stable UUID.
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

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

stable_id!(
    KnowledgeDocumentId,
    "Stable UUID v5 identifier for a logical knowledge document."
);
stable_id!(
    KnowledgeDocumentRevisionId,
    "Stable UUID v5 identifier for an immutable document revision."
);
stable_id!(
    KnowledgeChunkId,
    "Stable UUID v5 identifier for an immutable knowledge chunk."
);
stable_id!(
    KnowledgeRetrievalId,
    "Stable UUID v5 identifier for a reproducible retrieval read."
);
stable_id!(
    KnowledgeCitationCapabilityId,
    "Stable UUID v5 identifier for a redacted citation capability record."
);
stable_id!(
    KnowledgeLocalCitationProjectionId,
    "Stable UUID v5 identifier for a redacted local citation projection."
);

/// Scope boundary for knowledge ingestion and retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KnowledgeScope(Uuid);

impl KnowledgeScope {
    /// Derives a stable scope UUID v5 from an application-owned key.
    pub fn from_stable_key(value: impl AsRef<str>) -> Result<Self, KnowledgeError> {
        let value = validated_text(
            "knowledge_scope",
            value.as_ref().to_owned(),
            MAX_EXTERNAL_KEY_CHARS,
        )?;
        Ok(Self(Uuid::new_v5(
            &KNOWLEDGE_SCOPE_NAMESPACE,
            value.as_bytes(),
        )))
    }

    /// Wraps a persisted scope UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the scope UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl fmt::Display for KnowledgeScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Explicit source version supplied by ingestion callers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KnowledgeSourceVersion(String);

impl KnowledgeSourceVersion {
    /// Creates a non-empty source version.
    pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeError> {
        Ok(Self(validated_text(
            "knowledge_source_version",
            value.into(),
            MAX_LABEL_CHARS,
        )?))
    }

    /// Returns the version string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KnowledgeSourceVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Explicit deterministic algorithm version for text chunking.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChunkingVersion(String);

impl ChunkingVersion {
    /// Creates a chunking algorithm version.
    pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeError> {
        Ok(Self(validated_text(
            "chunking_version",
            value.into(),
            MAX_LABEL_CHARS,
        )?))
    }

    /// Returns the version string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explicit schema version that makes a citation record portable across capability consumers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CitationSchemaVersion(String);

impl CitationSchemaVersion {
    /// Creates a non-empty citation schema version.
    pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeError> {
        Ok(Self(validated_text(
            "citation_schema_version",
            value.into(),
            MAX_LABEL_CHARS,
        )?))
    }

    /// Returns the schema version text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact citation and embedding requirements declared by a capability consumer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationCompatibilityRequirement {
    citation_schema_version: CitationSchemaVersion,
    embedding_requirement: EmbeddingCapabilityRequirement,
}

impl CitationCompatibilityRequirement {
    /// Creates an exact requirement for redacted citation provenance.
    #[must_use]
    pub const fn new(
        citation_schema_version: CitationSchemaVersion,
        embedding_requirement: EmbeddingCapabilityRequirement,
    ) -> Self {
        Self {
            citation_schema_version,
            embedding_requirement,
        }
    }

    /// Returns the required citation schema version.
    #[must_use]
    pub const fn citation_schema_version(&self) -> &CitationSchemaVersion {
        &self.citation_schema_version
    }

    /// Returns the required embedding capability.
    #[must_use]
    pub const fn embedding_requirement(&self) -> &EmbeddingCapabilityRequirement {
        &self.embedding_requirement
    }
}

/// Redacted compatibility metadata for citations emitted by one retrieval capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationCompatibility {
    citation_schema_version: CitationSchemaVersion,
    embedding_capability: EmbeddingCapability,
}

impl CitationCompatibility {
    /// Returns the citation schema version.
    #[must_use]
    pub const fn citation_schema_version(&self) -> &CitationSchemaVersion {
        &self.citation_schema_version
    }

    /// Returns the embedding capability with no vector values or credentials.
    #[must_use]
    pub const fn embedding_capability(&self) -> &EmbeddingCapability {
        &self.embedding_capability
    }

    fn require_compatible(
        &self,
        requirement: &CitationCompatibilityRequirement,
    ) -> Result<(), KnowledgeError> {
        if self.citation_schema_version != requirement.citation_schema_version
            || self
                .embedding_capability
                .require_compatible(&requirement.embedding_requirement)
                .is_err()
        {
            return Err(KnowledgeError::CitationCompatibilityMismatch {
                expected: Box::new(requirement.clone()),
                actual: Box::new(self.clone()),
            });
        }
        Ok(())
    }
}

/// Deterministic character-boundary chunking policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkingPolicy {
    version: ChunkingVersion,
    maximum_characters: usize,
    overlap_characters: usize,
}

impl ChunkingPolicy {
    /// Creates a versioned chunking policy.
    pub fn new(
        version: impl Into<String>,
        maximum_characters: usize,
        overlap_characters: usize,
    ) -> Result<Self, KnowledgeError> {
        if maximum_characters == 0 || overlap_characters >= maximum_characters {
            return Err(KnowledgeError::InvalidChunkOverlap);
        }
        Ok(Self {
            version: ChunkingVersion::new(version)?,
            maximum_characters,
            overlap_characters,
        })
    }

    /// Returns the explicit chunking algorithm version.
    #[must_use]
    pub const fn version(&self) -> &ChunkingVersion {
        &self.version
    }

    /// Returns the maximum character count per chunk.
    #[must_use]
    pub const fn maximum_characters(&self) -> usize {
        self.maximum_characters
    }

    /// Returns overlap measured in Unicode scalar values.
    #[must_use]
    pub const fn overlap_characters(&self) -> usize {
        self.overlap_characters
    }

    fn fingerprint(&self) -> String {
        sha256_fingerprint(
            format!(
                "chunking-v1\\0{}\\0{}\\0{}",
                self.version.as_str(),
                self.maximum_characters,
                self.overlap_characters
            )
            .as_bytes(),
        )
    }
}

/// Input accepted at the private ingestion boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeIngestion {
    scope: KnowledgeScope,
    external_key: String,
    title: String,
    source_version: KnowledgeSourceVersion,
    content: String,
    chunking_policy: ChunkingPolicy,
}

impl KnowledgeIngestion {
    /// Creates a validated ingestion request with an explicit source and chunking version.
    pub fn new(
        scope: KnowledgeScope,
        external_key: impl Into<String>,
        title: impl Into<String>,
        source_version: impl Into<String>,
        content: impl Into<String>,
        chunking_policy: ChunkingPolicy,
    ) -> Result<Self, KnowledgeError> {
        let content = content.into();
        if content.trim().is_empty() {
            return Err(KnowledgeError::Empty {
                field: "knowledge_content",
            });
        }
        if content.chars().count() > MAX_CONTENT_CHARS {
            return Err(KnowledgeError::TooLong {
                field: "knowledge_content",
                max: MAX_CONTENT_CHARS,
            });
        }
        Ok(Self {
            scope,
            external_key: validated_text(
                "knowledge_external_key",
                external_key.into(),
                MAX_EXTERNAL_KEY_CHARS,
            )?,
            title: validated_text("knowledge_title", title.into(), MAX_LABEL_CHARS)?,
            source_version: KnowledgeSourceVersion::new(source_version)?,
            content,
            chunking_policy,
        })
    }

    /// Returns the ingestion scope.
    #[must_use]
    pub const fn scope(&self) -> KnowledgeScope {
        self.scope
    }
}

/// Query input accepted at the private retrieval boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeQuery(String);

impl KnowledgeQuery {
    /// Creates a validated retrieval query.
    pub fn new(value: impl Into<String>) -> Result<Self, KnowledgeError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(KnowledgeError::Empty {
                field: "knowledge_query",
            });
        }
        if value.chars().count() > MAX_CONTENT_CHARS {
            return Err(KnowledgeError::TooLong {
                field: "knowledge_query",
                max: MAX_CONTENT_CHARS,
            });
        }
        Ok(Self(value))
    }

    fn fingerprint(&self) -> String {
        sha256_fingerprint(self.0.as_bytes())
    }

    fn as_embedding_text(&self) -> Result<EmbeddingText, KnowledgeError> {
        Ok(EmbeddingText::new(self.0.clone())?)
    }
}

/// Bounded, versioned request for deterministic knowledge retrieval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrievalRequest {
    query: KnowledgeQuery,
    limit: usize,
    retrieval_version: String,
    scope: Option<KnowledgeScope>,
}

impl RetrievalRequest {
    /// Creates a retrieval request that searches every accessible scope.
    pub fn new(
        query: KnowledgeQuery,
        limit: usize,
        retrieval_version: impl Into<String>,
    ) -> Result<Self, KnowledgeError> {
        if limit == 0 || limit > MAX_RESULT_LIMIT {
            return Err(KnowledgeError::InvalidRetrievalLimit {
                max: MAX_RESULT_LIMIT,
            });
        }
        Ok(Self {
            query,
            limit,
            retrieval_version: validated_text(
                "retrieval_version",
                retrieval_version.into(),
                MAX_LABEL_CHARS,
            )?,
            scope: None,
        })
    }

    /// Narrows the request to one knowledge scope.
    #[must_use]
    pub fn for_scope(mut self, scope: KnowledgeScope) -> Self {
        self.scope = Some(scope);
        self
    }
}

/// Byte range in the original UTF-8 ingestion document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CitationRange {
    start_byte: usize,
    end_byte: usize,
}

impl CitationRange {
    /// Returns the inclusive start byte offset.
    #[must_use]
    pub const fn start_byte(self) -> usize {
        self.start_byte
    }

    /// Returns the exclusive end byte offset.
    #[must_use]
    pub const fn end_byte(self) -> usize {
        self.end_byte
    }
}

/// Verifiable citation metadata with no raw chunk text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeCitation {
    document_id: KnowledgeDocumentId,
    document_revision_id: KnowledgeDocumentRevisionId,
    chunk_id: KnowledgeChunkId,
    source_version: KnowledgeSourceVersion,
    chunking_version: ChunkingVersion,
    ordinal: u32,
    range: CitationRange,
    content_fingerprint: String,
}

impl KnowledgeCitation {
    /// Returns the stable document identifier.
    #[must_use]
    pub const fn document_id(&self) -> KnowledgeDocumentId {
        self.document_id
    }

    /// Returns the immutable document revision identifier.
    #[must_use]
    pub const fn document_revision_id(&self) -> KnowledgeDocumentRevisionId {
        self.document_revision_id
    }

    /// Returns the stable cited chunk identifier.
    #[must_use]
    pub const fn chunk_id(&self) -> KnowledgeChunkId {
        self.chunk_id
    }

    /// Returns the caller-supplied source version.
    #[must_use]
    pub const fn source_version(&self) -> &KnowledgeSourceVersion {
        &self.source_version
    }

    /// Returns the explicit chunking version.
    #[must_use]
    pub const fn chunking_version(&self) -> &ChunkingVersion {
        &self.chunking_version
    }

    /// Returns deterministic chunk ordinal within the document revision.
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    /// Returns the source byte range.
    #[must_use]
    pub const fn range(&self) -> CitationRange {
        self.range
    }

    /// Returns the chunk SHA-256 fingerprint.
    #[must_use]
    pub fn content_fingerprint(&self) -> &str {
        &self.content_fingerprint
    }
}

/// Provenance for one cited source without a raw document chunk or embedding vector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeSourceProvenance {
    citation: KnowledgeCitation,
    embedding_provenance: EmbeddingProvenance,
}

impl KnowledgeSourceProvenance {
    /// Returns citation identity, version, range, and content fingerprint metadata.
    #[must_use]
    pub const fn citation(&self) -> &KnowledgeCitation {
        &self.citation
    }

    /// Returns embedding provenance without an embedding vector or source content.
    #[must_use]
    pub const fn embedding_provenance(&self) -> &EmbeddingProvenance {
        &self.embedding_provenance
    }

    /// Confirms that source provenance excludes raw knowledge chunk content.
    #[must_use]
    pub const fn contains_raw_knowledge_chunk(&self) -> bool {
        false
    }
}

/// Immutable receipt produced by successful ingestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeIngestionReceipt {
    document_id: KnowledgeDocumentId,
    document_revision_id: KnowledgeDocumentRevisionId,
    document_version: KnowledgeSourceVersion,
    chunking_version: ChunkingVersion,
    chunk_count: usize,
}

impl KnowledgeIngestionReceipt {
    /// Returns the stable logical document identity.
    #[must_use]
    pub const fn document_id(&self) -> KnowledgeDocumentId {
        self.document_id
    }

    /// Returns the immutable revision identity.
    #[must_use]
    pub const fn document_revision_id(&self) -> KnowledgeDocumentRevisionId {
        self.document_revision_id
    }

    /// Returns the explicit source version.
    #[must_use]
    pub const fn document_version(&self) -> &KnowledgeSourceVersion {
        &self.document_version
    }

    /// Returns the chunking version used for this revision.
    #[must_use]
    pub const fn chunking_version(&self) -> &ChunkingVersion {
        &self.chunking_version
    }

    /// Returns the number of persisted chunks.
    #[must_use]
    pub const fn chunk_count(&self) -> usize {
        self.chunk_count
    }
}

/// Privacy-safe record of one deterministic retrieval read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeReadRecord {
    retrieval_id: KnowledgeRetrievalId,
    retrieval_version: String,
    scope: Option<KnowledgeScope>,
    query_fingerprint: String,
    cited_chunk_ids: Vec<KnowledgeChunkId>,
}

impl KnowledgeReadRecord {
    /// Returns the stable retrieval record identifier.
    #[must_use]
    pub const fn retrieval_id(&self) -> KnowledgeRetrievalId {
        self.retrieval_id
    }

    /// Returns the explicit retrieval algorithm version.
    #[must_use]
    pub fn retrieval_version(&self) -> &str {
        &self.retrieval_version
    }

    /// Returns the optional scope boundary applied to this retrieval.
    #[must_use]
    pub const fn scope(&self) -> Option<KnowledgeScope> {
        self.scope
    }

    /// Returns only the query fingerprint, never raw query text.
    #[must_use]
    pub fn query_fingerprint(&self) -> &str {
        &self.query_fingerprint
    }

    /// Returns cited chunk identities without chunk bodies.
    #[must_use]
    pub fn cited_chunk_ids(&self) -> &[KnowledgeChunkId] {
        &self.cited_chunk_ids
    }

    /// Confirms that this record has no raw document body by construction.
    #[must_use]
    pub const fn contains_raw_content(&self) -> bool {
        false
    }

    /// Confirms that this record has no raw query text by construction.
    #[must_use]
    pub const fn contains_raw_query(&self) -> bool {
        false
    }
}

/// Canonically ordered, provider-neutral citation capability result.
///
/// Sources are sorted by ascending `KnowledgeChunkId`; retrieval relevance remains
/// bound by the embedded read record and is not exposed as a score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeCitationCapability {
    id: KnowledgeCitationCapabilityId,
    schema_version: CitationSchemaVersion,
    compatibility: CitationCompatibility,
    read_record: KnowledgeReadRecord,
    sources: Vec<KnowledgeSourceProvenance>,
}

/// Canonically ordered local citation facts with no provider or vector data.
///
/// This V1 projection is suitable for a local read adapter that needs stable
/// citation provenance and retrieval fingerprints without re-exposing the
/// original query, source chunks, embeddings, or provider configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeLocalCitationProjectionV1 {
    id: KnowledgeLocalCitationProjectionId,
    schema_version: CitationSchemaVersion,
    retrieval_id: KnowledgeRetrievalId,
    retrieval_version: String,
    scope: Option<KnowledgeScope>,
    query_fingerprint: String,
    citations: Vec<KnowledgeCitation>,
}

impl KnowledgeLocalCitationProjectionV1 {
    fn from_retrieval(retrieval: KnowledgeRetrieval) -> Self {
        let KnowledgeRetrieval {
            mut citations,
            read_record,
        } = retrieval;
        citations.sort_by_key(KnowledgeCitation::chunk_id);
        let cited_chunk_ids = citations
            .iter()
            .map(|citation| citation.chunk_id().to_string())
            .collect::<Vec<_>>()
            .join(",");
        let identity = format!(
            "knowledge-local-citation-projection-v1\\0{}\\0{}\\0{}",
            read_record.retrieval_id, read_record.query_fingerprint, cited_chunk_ids,
        );

        Self {
            id: KnowledgeLocalCitationProjectionId(Uuid::new_v5(
                &KNOWLEDGE_LOCAL_CITATION_PROJECTION_NAMESPACE,
                identity.as_bytes(),
            )),
            schema_version: CitationSchemaVersion(
                KNOWLEDGE_LOCAL_CITATION_PROJECTION_SCHEMA_VERSION.to_owned(),
            ),
            retrieval_id: read_record.retrieval_id,
            retrieval_version: read_record.retrieval_version,
            scope: read_record.scope,
            query_fingerprint: read_record.query_fingerprint,
            citations,
        }
    }

    /// Returns the stable local projection identifier.
    #[must_use]
    pub const fn id(&self) -> KnowledgeLocalCitationProjectionId {
        self.id
    }

    /// Returns the explicit V1 projection schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &CitationSchemaVersion {
        &self.schema_version
    }

    /// Returns the redacted retrieval record identity.
    #[must_use]
    pub const fn retrieval_id(&self) -> KnowledgeRetrievalId {
        self.retrieval_id
    }

    /// Returns the explicit retrieval algorithm version.
    #[must_use]
    pub fn retrieval_version(&self) -> &str {
        &self.retrieval_version
    }

    /// Returns the optional scope boundary applied to the retrieval.
    #[must_use]
    pub const fn scope(&self) -> Option<KnowledgeScope> {
        self.scope
    }

    /// Returns the SHA-256 fingerprint of the private retrieval query.
    #[must_use]
    pub fn query_fingerprint(&self) -> &str {
        &self.query_fingerprint
    }

    /// Returns citations in ascending stable chunk-identifier order.
    #[must_use]
    pub fn citations(&self) -> &[KnowledgeCitation] {
        &self.citations
    }

    /// Confirms that raw knowledge chunks are absent by construction.
    #[must_use]
    pub const fn contains_raw_knowledge_chunks(&self) -> bool {
        false
    }

    /// Confirms that the private retrieval query is absent by construction.
    #[must_use]
    pub const fn contains_raw_retrieval_query(&self) -> bool {
        false
    }

    /// Confirms that embedding vector values are absent by construction.
    #[must_use]
    pub const fn contains_raw_vectors(&self) -> bool {
        false
    }

    /// Confirms that provider credentials are absent by construction.
    #[must_use]
    pub const fn contains_provider_secrets(&self) -> bool {
        false
    }
}

impl KnowledgeCitationCapability {
    fn new(
        schema_version: CitationSchemaVersion,
        compatibility: CitationCompatibility,
        read_record: KnowledgeReadRecord,
        mut sources: Vec<KnowledgeSourceProvenance>,
        requirement: &CitationCompatibilityRequirement,
    ) -> Result<Self, KnowledgeError> {
        compatibility.require_compatible(requirement)?;
        sources.sort_by_key(|source| source.citation.chunk_id());
        for source in &sources {
            if source.embedding_provenance.model() != compatibility.embedding_capability.model()
                || source.embedding_provenance.dimension()
                    != compatibility.embedding_capability.dimension()
            {
                return Err(KnowledgeError::CitationSourceEmbeddingMismatch {
                    chunk_id: source.citation.chunk_id(),
                });
            }
        }
        let source_ids = sources
            .iter()
            .map(|source| source.citation.chunk_id().to_string())
            .collect::<Vec<_>>()
            .join(",");
        let identity = format!(
            "knowledge-citation-capability-v1\\0{}\\0{}\\0{}\\0{}",
            schema_version.as_str(),
            read_record.retrieval_id(),
            compatibility.embedding_capability.id(),
            source_ids,
        );
        Ok(Self {
            id: KnowledgeCitationCapabilityId(Uuid::new_v5(
                &KNOWLEDGE_CITATION_CAPABILITY_NAMESPACE,
                identity.as_bytes(),
            )),
            schema_version,
            compatibility,
            read_record,
            sources,
        })
    }

    /// Returns the stable capability record identifier.
    #[must_use]
    pub const fn id(&self) -> KnowledgeCitationCapabilityId {
        self.id
    }

    /// Returns the explicit capability record schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &CitationSchemaVersion {
        &self.schema_version
    }

    /// Returns citation and embedding compatibility metadata.
    #[must_use]
    pub const fn compatibility(&self) -> &CitationCompatibility {
        &self.compatibility
    }

    /// Returns the redacted retrieval receipt that binds the retrieval provenance.
    #[must_use]
    pub const fn read_record(&self) -> &KnowledgeReadRecord {
        &self.read_record
    }

    /// Returns source provenance in ascending stable chunk-identifier order.
    #[must_use]
    pub fn sources(&self) -> &[KnowledgeSourceProvenance] {
        &self.sources
    }

    /// Confirms that the capability record cannot expose raw knowledge chunks.
    #[must_use]
    pub const fn contains_raw_knowledge_chunks(&self) -> bool {
        false
    }
}

/// Retrieval output containing citations and a privacy-safe read record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeRetrieval {
    citations: Vec<KnowledgeCitation>,
    read_record: KnowledgeReadRecord,
}

impl KnowledgeRetrieval {
    /// Returns citations ordered by descending score then stable source identifier.
    #[must_use]
    pub fn citations(&self) -> &[KnowledgeCitation] {
        &self.citations
    }

    /// Returns the privacy-safe read record.
    #[must_use]
    pub const fn read_record(&self) -> &KnowledgeReadRecord {
        &self.read_record
    }
}

/// Provider-free persistence and retrieval port.
pub trait KnowledgeRepository {
    /// Ingests a versioned document and returns deterministic receipt metadata.
    fn ingest(
        &self,
        ingestion: KnowledgeIngestion,
    ) -> Result<KnowledgeIngestionReceipt, KnowledgeError>;

    /// Retrieves ranked citations without returning raw document content.
    fn retrieve(&self, request: RetrievalRequest) -> Result<KnowledgeRetrieval, KnowledgeError>;
}

/// Provider-neutral port for citation-compatible, redacted knowledge retrieval.
pub trait KnowledgeCitationCapabilityRepository {
    /// Retrieves canonical source provenance after exact compatibility validation.
    fn retrieve_citation_capability(
        &self,
        request: RetrievalRequest,
        requirement: CitationCompatibilityRequirement,
    ) -> Result<KnowledgeCitationCapability, KnowledgeError>;
}

/// Provider-free port for redacted local citation projections.
pub trait KnowledgeLocalCitationProjectionRepository {
    /// Projects deterministic citation facts from one private retrieval request.
    fn project_local_citation_v1(
        &self,
        request: RetrievalRequest,
    ) -> Result<KnowledgeLocalCitationProjectionV1, KnowledgeError>;
}

#[derive(Debug, Clone)]
struct IndexedChunk {
    scope: KnowledgeScope,
    citation: KnowledgeCitation,
    embedding: Embedding,
}

/// Deterministic in-memory adapter for provider-free knowledge workflows.
#[derive(Debug)]
pub struct InMemoryKnowledgeRepository<E> {
    embedding_provider: E,
    chunks: RwLock<BTreeMap<KnowledgeChunkId, IndexedChunk>>,
}

impl<E> InMemoryKnowledgeRepository<E>
where
    E: EmbeddingProvider,
{
    /// Creates an empty deterministic local repository.
    #[must_use]
    pub fn new(embedding_provider: E) -> Self {
        Self {
            embedding_provider,
            chunks: RwLock::new(BTreeMap::new()),
        }
    }

    /// Ingests a document without retaining its raw body after embeddings are generated.
    pub fn ingest(
        &self,
        ingestion: KnowledgeIngestion,
    ) -> Result<KnowledgeIngestionReceipt, KnowledgeError> {
        let document_id = KnowledgeDocumentId(Uuid::new_v5(
            &KNOWLEDGE_DOCUMENT_NAMESPACE,
            format!("{}\\0{}", ingestion.scope, ingestion.external_key).as_bytes(),
        ));
        let content_fingerprint = sha256_fingerprint(ingestion.content.as_bytes());
        let policy_fingerprint = ingestion.chunking_policy.fingerprint();
        let document_revision_id = KnowledgeDocumentRevisionId(Uuid::new_v5(
            &KNOWLEDGE_REVISION_NAMESPACE,
            format!(
                "{}\\0{}\\0{}\\0{}",
                document_id, ingestion.source_version, content_fingerprint, policy_fingerprint
            )
            .as_bytes(),
        ));
        let raw_chunks = chunk_document(&ingestion.content, &ingestion.chunking_policy);
        let mut indexed_chunks = Vec::with_capacity(raw_chunks.len());
        for (ordinal, raw_chunk) in raw_chunks.iter().enumerate() {
            let ordinal =
                u32::try_from(ordinal).expect("document chunk count is bounded by input size");
            let chunk_fingerprint = sha256_fingerprint(raw_chunk.text.as_bytes());
            let chunk_id = KnowledgeChunkId(Uuid::new_v5(
                &KNOWLEDGE_CHUNK_NAMESPACE,
                format!(
                    "{}\\0{}\\0{}\\0{}\\0{}",
                    document_revision_id,
                    ordinal,
                    raw_chunk.range.start_byte,
                    raw_chunk.range.end_byte,
                    chunk_fingerprint
                )
                .as_bytes(),
            ));
            let citation = KnowledgeCitation {
                document_id,
                document_revision_id,
                chunk_id,
                source_version: ingestion.source_version.clone(),
                chunking_version: ingestion.chunking_policy.version.clone(),
                ordinal,
                range: raw_chunk.range,
                content_fingerprint: chunk_fingerprint,
            };
            let embedding = self.embedding_provider.embed(
                &chunk_id.to_string(),
                &EmbeddingText::new(raw_chunk.text.clone())?,
            )?;
            indexed_chunks.push((
                chunk_id,
                IndexedChunk {
                    scope: ingestion.scope,
                    citation,
                    embedding,
                },
            ));
        }

        let mut chunks = self
            .chunks
            .write()
            .map_err(|_| KnowledgeError::RepositoryUnavailable)?;
        for (chunk_id, chunk) in indexed_chunks {
            chunks.entry(chunk_id).or_insert(chunk);
        }
        Ok(KnowledgeIngestionReceipt {
            document_id,
            document_revision_id,
            document_version: ingestion.source_version,
            chunking_version: ingestion.chunking_policy.version,
            chunk_count: raw_chunks.len(),
        })
    }

    /// Retrieves deterministic citations with no document body exposure.
    pub fn retrieve(
        &self,
        request: RetrievalRequest,
    ) -> Result<KnowledgeRetrieval, KnowledgeError> {
        let chunks = self
            .chunks
            .read()
            .map_err(|_| KnowledgeError::RepositoryUnavailable)?;
        let available = chunks
            .values()
            .filter(|chunk| request.scope.is_none_or(|scope| scope == chunk.scope))
            .collect::<Vec<_>>();
        let embeddings = available
            .iter()
            .map(|chunk| chunk.embedding.clone())
            .collect::<Vec<_>>();
        let index = VectorIndex::from_embeddings(embeddings)?;
        let query_vector = self
            .embedding_provider
            .embed_query(&request.query.as_embedding_text()?)?;
        let rankings = index.search(query_vector, request.limit);
        let mut citations = Vec::with_capacity(rankings.len());
        for ranking in rankings {
            let chunk_id = Uuid::parse_str(ranking.source_key())
                .map(KnowledgeChunkId)
                .map_err(|_| KnowledgeError::UnknownIndexedChunk)?;
            let Some(chunk) = chunks.get(&chunk_id) else {
                return Err(KnowledgeError::UnknownIndexedChunk);
            };
            citations.push(chunk.citation.clone());
        }
        let cited_chunk_ids = citations
            .iter()
            .map(|citation| citation.chunk_id)
            .collect::<Vec<_>>();
        let identity = format!(
            "knowledge-retrieval-v1\\0{}\\0{}\\0{}\\0{}",
            request.retrieval_version,
            request.query.fingerprint(),
            request
                .scope
                .map_or_else(|| "all".to_owned(), |scope| scope.to_string()),
            cited_chunk_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        Ok(KnowledgeRetrieval {
            citations,
            read_record: KnowledgeReadRecord {
                retrieval_id: KnowledgeRetrievalId(Uuid::new_v5(
                    &KNOWLEDGE_RETRIEVAL_NAMESPACE,
                    identity.as_bytes(),
                )),
                retrieval_version: request.retrieval_version,
                scope: request.scope,
                query_fingerprint: request.query.fingerprint(),
                cited_chunk_ids,
            },
        })
    }

    /// Retrieves a redacted, citation-compatible capability record.
    pub fn retrieve_citation_capability(
        &self,
        request: RetrievalRequest,
        requirement: CitationCompatibilityRequirement,
    ) -> Result<KnowledgeCitationCapability, KnowledgeError>
    where
        E: EmbeddingCapabilityPort,
    {
        let retrieval = self.retrieve(request)?;
        let compatibility = CitationCompatibility {
            citation_schema_version: CitationSchemaVersion::new("citation-schema-v1")?,
            embedding_capability: self.embedding_provider.embedding_capability()?,
        };
        let chunks = self
            .chunks
            .read()
            .map_err(|_| KnowledgeError::RepositoryUnavailable)?;
        let mut sources = Vec::with_capacity(retrieval.citations.len());
        for citation in &retrieval.citations {
            let Some(chunk) = chunks.get(&citation.chunk_id) else {
                return Err(KnowledgeError::UnknownIndexedChunk);
            };
            sources.push(KnowledgeSourceProvenance {
                citation: citation.clone(),
                embedding_provenance: chunk.embedding.provenance(),
            });
        }
        KnowledgeCitationCapability::new(
            CitationSchemaVersion::new("knowledge-citation-capability-v1")?,
            compatibility,
            retrieval.read_record,
            sources,
            &requirement,
        )
    }

    /// Projects redacted, canonically ordered citation facts for a local read adapter.
    pub fn project_local_citation_v1(
        &self,
        request: RetrievalRequest,
    ) -> Result<KnowledgeLocalCitationProjectionV1, KnowledgeError> {
        Ok(KnowledgeLocalCitationProjectionV1::from_retrieval(
            self.retrieve(request)?,
        ))
    }
}

impl<E> KnowledgeRepository for InMemoryKnowledgeRepository<E>
where
    E: EmbeddingProvider,
{
    fn ingest(
        &self,
        ingestion: KnowledgeIngestion,
    ) -> Result<KnowledgeIngestionReceipt, KnowledgeError> {
        Self::ingest(self, ingestion)
    }

    fn retrieve(&self, request: RetrievalRequest) -> Result<KnowledgeRetrieval, KnowledgeError> {
        Self::retrieve(self, request)
    }
}

impl<E> KnowledgeCitationCapabilityRepository for InMemoryKnowledgeRepository<E>
where
    E: EmbeddingProvider + EmbeddingCapabilityPort,
{
    fn retrieve_citation_capability(
        &self,
        request: RetrievalRequest,
        requirement: CitationCompatibilityRequirement,
    ) -> Result<KnowledgeCitationCapability, KnowledgeError> {
        Self::retrieve_citation_capability(self, request, requirement)
    }
}

impl<E> KnowledgeLocalCitationProjectionRepository for InMemoryKnowledgeRepository<E>
where
    E: EmbeddingProvider,
{
    fn project_local_citation_v1(
        &self,
        request: RetrievalRequest,
    ) -> Result<KnowledgeLocalCitationProjectionV1, KnowledgeError> {
        Self::project_local_citation_v1(self, request)
    }
}

#[derive(Debug, Clone)]
struct RawChunk {
    text: String,
    range: CitationRange,
}

fn chunk_document(content: &str, policy: &ChunkingPolicy) -> Vec<RawChunk> {
    let character_boundaries = content
        .char_indices()
        .map(|(offset, _)| offset)
        .chain(std::iter::once(content.len()))
        .collect::<Vec<_>>();
    let character_count = character_boundaries.len().saturating_sub(1);
    let mut chunks = Vec::new();
    let mut start = 0_usize;

    while start < character_count {
        while start < character_count
            && character_at(content, &character_boundaries, start).is_whitespace()
        {
            start += 1;
        }
        if start >= character_count {
            break;
        }
        let maximum_end = (start + policy.maximum_characters).min(character_count);
        let mut end = maximum_end;
        if maximum_end < character_count {
            if let Some(boundary) = ((start + 1)..=maximum_end).rev().find(|candidate| {
                character_at(content, &character_boundaries, candidate - 1).is_whitespace()
            }) {
                end = boundary - 1;
            }
        }
        if end <= start {
            end = maximum_end;
        }
        let start_byte = character_boundaries[start];
        let mut end_byte = character_boundaries[end];
        let slice = &content[start_byte..end_byte];
        let trimmed = slice.trim();
        let leading_trim = slice.len() - slice.trim_start().len();
        end_byte -= slice.len() - slice.trim_end().len();
        if !trimmed.is_empty() {
            chunks.push(RawChunk {
                text: trimmed.to_owned(),
                range: CitationRange {
                    start_byte: start_byte + leading_trim,
                    end_byte,
                },
            });
        }
        if end == character_count {
            break;
        }
        start = end.saturating_sub(policy.overlap_characters);
    }
    chunks
}

fn character_at(content: &str, boundaries: &[usize], index: usize) -> char {
    content[boundaries[index]..boundaries[index + 1]]
        .chars()
        .next()
        .expect("character boundary must identify one character")
}

fn validated_text(
    field: &'static str,
    value: String,
    maximum_characters: usize,
) -> Result<String, KnowledgeError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(KnowledgeError::Empty { field });
    }
    if value.chars().count() > maximum_characters {
        return Err(KnowledgeError::TooLong {
            field,
            max: maximum_characters,
        });
    }
    Ok(value)
}

fn sha256_fingerprint(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    format!("sha256:{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_embedding::{DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig};

    #[test]
    fn chunking_preserves_nonempty_deterministic_ranges() {
        let chunks = chunk_document(
            "alpha beta gamma delta",
            &ChunkingPolicy::new("v1", 10, 2).expect("policy"),
        );

        assert!(!chunks.is_empty());
        assert!(chunks.iter().all(|chunk| !chunk.text.is_empty()));
        assert!(
            chunks
                .windows(2)
                .all(|pair| pair[0].range.start_byte < pair[1].range.end_byte)
        );
    }

    #[test]
    fn private_read_records_do_not_store_raw_query_or_content() {
        let repository = InMemoryKnowledgeRepository::new(DeterministicEmbeddingAdapter::new(
            DeterministicEmbeddingConfig::new("local", "1", 8).expect("config"),
        ));
        repository
            .ingest(
                KnowledgeIngestion::new(
                    KnowledgeScope::from_stable_key("scope").expect("scope"),
                    "document",
                    "Document",
                    "1",
                    "private source body",
                    ChunkingPolicy::new("v1", 64, 0).expect("policy"),
                )
                .expect("ingestion"),
            )
            .expect("stored");
        let result = repository
            .retrieve(
                RetrievalRequest::new(
                    KnowledgeQuery::new("private source").expect("query"),
                    1,
                    "retrieval-v1",
                )
                .expect("request"),
            )
            .expect("retrieval");

        assert!(!result.read_record().contains_raw_content());
        assert!(!result.read_record().contains_raw_query());
    }
}
