//! Provider-free embedding ports and deterministic local adapters.
//!
//! This crate deliberately contains no provider credentials, network clients, or
//! mutable global state. It offers a deterministic local adapter for fixtures,
//! replay, and conformance tests, while production integrations can implement
//! the EmbeddingProvider trait.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

const EMBEDDING_ID_NAMESPACE: Uuid = Uuid::from_u128(0x8187_a0ec_5755_50d5_8ef7_b590_00f5_2893);
const EMBEDDING_CAPABILITY_NAMESPACE: Uuid =
    Uuid::from_u128(0x9d8c_7395_11f8_575e_a64d_f3a8_3c51_91c2);
const MAX_TEXT_CHARS: usize = 1_000_000;
const MAX_MODEL_CHARS: usize = 240;
const MAX_SOURCE_KEY_CHARS: usize = 1_024;
const MAX_DIMENSION: usize = 4_096;

/// Errors produced by embedding ports, adapters, and vector indices.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EmbeddingError {
    /// A required text field was blank after trimming.
    #[error("{field} must not be empty")]
    Empty {
        /// The invalid field name.
        field: &'static str,
    },
    /// A text field exceeded its bounded length.
    #[error("{field} must be at most {max} characters")]
    TooLong {
        /// The invalid field name.
        field: &'static str,
        /// Maximum accepted Unicode scalar count.
        max: usize,
    },
    /// A vector dimension was outside the supported deterministic range.
    #[error("embedding dimension must be between 1 and {max}")]
    InvalidDimension {
        /// Maximum accepted dimension.
        max: usize,
    },
    /// A vector contained a NaN or infinite component.
    #[error("embedding vector values must be finite")]
    NonFiniteVector,
    /// A vector had no directional information and cannot be normalized.
    #[error("embedding vector must not have zero norm")]
    ZeroNorm,
    /// Vector dimensions did not match for a similarity operation.
    #[error("embedding dimensions must match")]
    DimensionMismatch,
    /// A vector index received duplicate source keys.
    #[error("embedding source key must be unique: {source_key}")]
    DuplicateSourceKey {
        /// Conflicting deterministic source key.
        source_key: String,
    },
    /// A vector index mixed embeddings from different model versions.
    #[error("all embeddings in an index must use the same model version")]
    ModelMismatch,
    /// A capability requirement did not exactly match the available deterministic capability.
    #[error("embedding capability does not satisfy the required schema, model, or dimension")]
    CapabilityMismatch {
        /// Consumer-declared compatibility requirement.
        expected: Box<EmbeddingCapabilityRequirement>,
        /// Available capability that failed the requirement.
        actual: Box<EmbeddingCapability>,
    },
}

/// A validated, explicit model name and version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EmbeddingModel {
    name: String,
    version: EmbeddingModelVersion,
}

impl EmbeddingModel {
    /// Creates a model identity with an explicit version.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Result<Self, EmbeddingError> {
        Ok(Self {
            name: validated_text("embedding_model.name", name.into(), MAX_MODEL_CHARS)?,
            version: EmbeddingModelVersion::new(version)?,
        })
    }

    /// Returns the stable model name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the explicit model version.
    #[must_use]
    pub const fn version(&self) -> &EmbeddingModelVersion {
        &self.version
    }
}

/// A validated embedding model version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EmbeddingModelVersion(String);

impl EmbeddingModelVersion {
    /// Creates a model version.
    pub fn new(value: impl Into<String>) -> Result<Self, EmbeddingError> {
        Ok(Self(validated_text(
            "embedding_model.version",
            value.into(),
            MAX_MODEL_CHARS,
        )?))
    }

    /// Returns the version text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmbeddingModelVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Explicit schema version for a provider-neutral embedding capability record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EmbeddingCapabilitySchemaVersion(String);

impl EmbeddingCapabilitySchemaVersion {
    /// Creates a non-empty embedding capability schema version.
    pub fn new(value: impl Into<String>) -> Result<Self, EmbeddingError> {
        Ok(Self(validated_text(
            "embedding_capability.schema_version",
            value.into(),
            MAX_MODEL_CHARS,
        )?))
    }

    /// Returns the schema version text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable UUID v5 identifier for an embedding capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EmbeddingCapabilityId(Uuid);

impl EmbeddingCapabilityId {
    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl fmt::Display for EmbeddingCapabilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Exact capability requirements declared by a provider-neutral consumer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingCapabilityRequirement {
    schema_version: EmbeddingCapabilitySchemaVersion,
    model: EmbeddingModel,
    dimension: usize,
}

impl EmbeddingCapabilityRequirement {
    /// Creates an exact schema, model, and vector-dimension requirement.
    pub fn new(
        schema_version: impl Into<String>,
        model_name: impl Into<String>,
        model_version: impl Into<String>,
        dimension: usize,
    ) -> Result<Self, EmbeddingError> {
        validate_dimension(dimension)?;
        Ok(Self {
            schema_version: EmbeddingCapabilitySchemaVersion::new(schema_version)?,
            model: EmbeddingModel::new(model_name, model_version)?,
            dimension,
        })
    }

    /// Returns the required capability schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &EmbeddingCapabilitySchemaVersion {
        &self.schema_version
    }

    /// Returns the required model identity.
    #[must_use]
    pub const fn model(&self) -> &EmbeddingModel {
        &self.model
    }

    /// Returns the required vector dimension.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Provider-neutral, versioned embedding capability metadata with no vector values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingCapability {
    id: EmbeddingCapabilityId,
    schema_version: EmbeddingCapabilitySchemaVersion,
    model: EmbeddingModel,
    dimension: usize,
}

impl EmbeddingCapability {
    fn new(
        schema_version: EmbeddingCapabilitySchemaVersion,
        model: EmbeddingModel,
        dimension: usize,
    ) -> Result<Self, EmbeddingError> {
        validate_dimension(dimension)?;
        let identity = format!(
            "embedding-capability-v1\\0{}\\0{}\\0{}\\0{}",
            schema_version.as_str(),
            model.name(),
            model.version(),
            dimension
        );
        Ok(Self {
            id: EmbeddingCapabilityId(Uuid::new_v5(
                &EMBEDDING_CAPABILITY_NAMESPACE,
                identity.as_bytes(),
            )),
            schema_version,
            model,
            dimension,
        })
    }

    /// Returns the stable capability identifier.
    #[must_use]
    pub const fn id(&self) -> EmbeddingCapabilityId {
        self.id
    }

    /// Returns the explicit record schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &EmbeddingCapabilitySchemaVersion {
        &self.schema_version
    }

    /// Returns the model identity without any provider credentials.
    #[must_use]
    pub const fn model(&self) -> &EmbeddingModel {
        &self.model
    }

    /// Returns the fixed vector dimension without vector values.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Fails closed unless a consumer requirement exactly matches this capability.
    pub fn require_compatible(
        &self,
        requirement: &EmbeddingCapabilityRequirement,
    ) -> Result<(), EmbeddingError> {
        if self.schema_version == requirement.schema_version
            && self.model == requirement.model
            && self.dimension == requirement.dimension
        {
            return Ok(());
        }
        Err(EmbeddingError::CapabilityMismatch {
            expected: Box::new(requirement.clone()),
            actual: Box::new(self.clone()),
        })
    }

    /// Confirms that capability metadata contains no raw vector values.
    #[must_use]
    pub const fn contains_raw_vector(&self) -> bool {
        false
    }
}

/// Provider-neutral port for advertising deterministic embedding compatibility.
pub trait EmbeddingCapabilityPort: Send + Sync {
    /// Returns the versioned capability without exposing vectors or credentials.
    fn embedding_capability(&self) -> Result<EmbeddingCapability, EmbeddingError>;
}

/// Input text to be embedded by a provider port.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingText(String);

impl EmbeddingText {
    /// Creates non-empty embedding text without modifying its semantic content.
    pub fn new(value: impl Into<String>) -> Result<Self, EmbeddingError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(EmbeddingError::Empty {
                field: "embedding_text",
            });
        }
        if value.chars().count() > MAX_TEXT_CHARS {
            return Err(EmbeddingError::TooLong {
                field: "embedding_text",
                max: MAX_TEXT_CHARS,
            });
        }
        Ok(Self(value))
    }

    /// Returns the supplied text for a provider invocation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a deterministic SHA-256 content fingerprint without retaining it globally.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        sha256_fingerprint(self.0.as_bytes())
    }
}

/// Stable UUID v5 identifier for an embedding artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EmbeddingId(Uuid);

impl EmbeddingId {
    /// Wraps an existing UUID.
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

impl fmt::Display for EmbeddingId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// A finite normalized embedding vector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingVector(Vec<f32>);

impl EmbeddingVector {
    /// Validates and normalizes a vector using a deterministic f64 accumulation path.
    pub fn normalized(values: Vec<f32>) -> Result<Self, EmbeddingError> {
        if values.is_empty() || values.len() > MAX_DIMENSION {
            return Err(EmbeddingError::InvalidDimension { max: MAX_DIMENSION });
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(EmbeddingError::NonFiniteVector);
        }

        let squared_norm = values.iter().fold(0.0_f64, |sum, value| {
            let value = f64::from(*value);
            sum + value * value
        });
        if !squared_norm.is_finite() || squared_norm <= 0.0 {
            return Err(EmbeddingError::ZeroNorm);
        }

        let norm = squared_norm.sqrt();
        let normalized = values
            .into_iter()
            .map(|value| (f64::from(value) / norm) as f32)
            .collect();
        Ok(Self(normalized))
    }

    /// Returns the vector dimension.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.0.len()
    }

    /// Returns normalized vector components.
    #[must_use]
    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }

    /// Computes cosine similarity after checking dimensions.
    pub fn cosine_similarity(&self, other: &Self) -> Result<f32, EmbeddingError> {
        if self.dimension() != other.dimension() {
            return Err(EmbeddingError::DimensionMismatch);
        }
        let score = self
            .0
            .iter()
            .zip(&other.0)
            .fold(0.0_f64, |sum, (left, right)| {
                sum + f64::from(*left) * f64::from(*right)
            });
        Ok(score.clamp(-1.0, 1.0) as f32)
    }
}

/// A generated embedding associated with a stable source key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embedding {
    id: EmbeddingId,
    source_key: String,
    model: EmbeddingModel,
    content_fingerprint: String,
    vector: EmbeddingVector,
}

impl Embedding {
    /// Returns the deterministic embedding identifier.
    #[must_use]
    pub const fn id(&self) -> EmbeddingId {
        self.id
    }

    /// Returns the stable source key used for ranking tie-breaks.
    #[must_use]
    pub fn source_key(&self) -> &str {
        &self.source_key
    }

    /// Returns the model identity.
    #[must_use]
    pub const fn model(&self) -> &EmbeddingModel {
        &self.model
    }

    /// Returns the SHA-256 content fingerprint.
    #[must_use]
    pub fn content_fingerprint(&self) -> &str {
        &self.content_fingerprint
    }

    /// Returns normalized vector values.
    #[must_use]
    pub const fn vector(&self) -> &EmbeddingVector {
        &self.vector
    }

    /// Returns source provenance without exposing the embedding vector.
    #[must_use]
    pub fn provenance(&self) -> EmbeddingProvenance {
        EmbeddingProvenance {
            embedding_id: self.id,
            model: self.model.clone(),
            dimension: self.vector.dimension(),
            content_fingerprint: self.content_fingerprint.clone(),
        }
    }
}

/// Source provenance for an embedding that omits the raw vector and input text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingProvenance {
    embedding_id: EmbeddingId,
    model: EmbeddingModel,
    dimension: usize,
    content_fingerprint: String,
}

impl EmbeddingProvenance {
    /// Returns the stable embedding identifier.
    #[must_use]
    pub const fn embedding_id(&self) -> EmbeddingId {
        self.embedding_id
    }

    /// Returns the model identity.
    #[must_use]
    pub const fn model(&self) -> &EmbeddingModel {
        &self.model
    }

    /// Returns the vector dimension without exposing values.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the SHA-256 content fingerprint.
    #[must_use]
    pub fn content_fingerprint(&self) -> &str {
        &self.content_fingerprint
    }

    /// Confirms that provenance contains no raw vector or input text.
    #[must_use]
    pub const fn contains_raw_content(&self) -> bool {
        false
    }
}

/// A provider-free port for producing document and query embeddings.
pub trait EmbeddingProvider: Send + Sync {
    /// Returns the explicit model identity used by this provider.
    fn model(&self) -> &EmbeddingModel;

    /// Produces an embedding for a stable source key.
    fn embed(&self, source_key: &str, text: &EmbeddingText) -> Result<Embedding, EmbeddingError>;

    /// Produces a query vector using the same model and dimension as documents.
    fn embed_query(&self, text: &EmbeddingText) -> Result<EmbeddingVector, EmbeddingError>;
}

/// Configuration for the deterministic local adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicEmbeddingConfig {
    model: EmbeddingModel,
    dimension: usize,
}

impl DeterministicEmbeddingConfig {
    /// Creates deterministic adapter configuration with an explicit model version.
    pub fn new(
        model_name: impl Into<String>,
        model_version: impl Into<String>,
        dimension: usize,
    ) -> Result<Self, EmbeddingError> {
        if dimension == 0 || dimension > MAX_DIMENSION {
            return Err(EmbeddingError::InvalidDimension { max: MAX_DIMENSION });
        }
        Ok(Self {
            model: EmbeddingModel::new(model_name, model_version)?,
            dimension,
        })
    }

    /// Returns the model identity.
    #[must_use]
    pub const fn model(&self) -> &EmbeddingModel {
        &self.model
    }

    /// Returns the fixed vector dimension.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Deterministic local adapter for tests, fixtures, replay, and offline development.
#[derive(Debug, Clone)]
pub struct DeterministicEmbeddingAdapter {
    config: DeterministicEmbeddingConfig,
}

impl DeterministicEmbeddingAdapter {
    /// Creates a local deterministic embedding adapter.
    #[must_use]
    pub const fn new(config: DeterministicEmbeddingConfig) -> Self {
        Self { config }
    }

    /// Embeds one stable source with deterministic UUID v5 identity and vector values.
    pub fn embed(
        &self,
        source_key: &str,
        text: &EmbeddingText,
    ) -> Result<Embedding, EmbeddingError> {
        let source_key = validated_text(
            "embedding.source_key",
            source_key.to_owned(),
            MAX_SOURCE_KEY_CHARS,
        )?;
        let content_fingerprint = text.fingerprint();
        let identity_name = format!(
            "embedding-v1\\0{}\\0{}\\0{}\\0{}",
            self.config.model.name(),
            self.config.model.version(),
            source_key,
            content_fingerprint
        );
        let id = EmbeddingId(Uuid::new_v5(
            &EMBEDDING_ID_NAMESPACE,
            identity_name.as_bytes(),
        ));
        Ok(Embedding {
            id,
            source_key,
            model: self.config.model.clone(),
            content_fingerprint,
            vector: self.embed_query(text)?,
        })
    }

    /// Produces a deterministic normalized query vector.
    pub fn embed_query(&self, text: &EmbeddingText) -> Result<EmbeddingVector, EmbeddingError> {
        let mut values = vec![0.0_f32; self.config.dimension];
        let mut token_count = 0_u64;
        for token in normalized_tokens(text.as_str()) {
            token_count += 1;
            let mut hasher = Sha256::new();
            hasher.update(b"contextlab-deterministic-embedding-v1\\0");
            hasher.update(self.config.model.name().as_bytes());
            hasher.update(b"\\0");
            hasher.update(self.config.model.version().as_str().as_bytes());
            hasher.update(b"\\0");
            hasher.update(token.as_bytes());
            let digest = hasher.finalize();
            let index = usize::from_be_bytes([
                digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6],
                digest[7],
            ]) % self.config.dimension;
            let magnitude = 1.0 + f32::from(digest[8]) / 255.0;
            let sign = if digest[9] & 1 == 0 { 1.0 } else { -1.0 };
            values[index] += sign * magnitude;
        }
        if token_count == 0 {
            return Err(EmbeddingError::Empty {
                field: "embedding_text",
            });
        }
        EmbeddingVector::normalized(values)
    }

    /// Returns the deterministic adapter configuration.
    #[must_use]
    pub const fn config(&self) -> &DeterministicEmbeddingConfig {
        &self.config
    }
}

impl EmbeddingProvider for DeterministicEmbeddingAdapter {
    fn model(&self) -> &EmbeddingModel {
        self.config.model()
    }

    fn embed(&self, source_key: &str, text: &EmbeddingText) -> Result<Embedding, EmbeddingError> {
        Self::embed(self, source_key, text)
    }

    fn embed_query(&self, text: &EmbeddingText) -> Result<EmbeddingVector, EmbeddingError> {
        Self::embed_query(self, text)
    }
}

impl EmbeddingCapabilityPort for DeterministicEmbeddingAdapter {
    fn embedding_capability(&self) -> Result<EmbeddingCapability, EmbeddingError> {
        EmbeddingCapability::new(
            EmbeddingCapabilitySchemaVersion::new("embedding-capability-v1")?,
            self.config.model.clone(),
            self.config.dimension,
        )
    }
}

/// A deterministic ranked search result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankedEmbedding {
    source_key: String,
    embedding_id: EmbeddingId,
    score: f32,
}

impl RankedEmbedding {
    /// Returns the stable source key.
    #[must_use]
    pub fn source_key(&self) -> &str {
        &self.source_key
    }

    /// Returns the stable embedding identifier.
    #[must_use]
    pub const fn embedding_id(&self) -> EmbeddingId {
        self.embedding_id
    }

    /// Returns cosine similarity score.
    #[must_use]
    pub const fn score(&self) -> f32 {
        self.score
    }
}

/// A deterministic in-memory vector index.
#[derive(Debug, Clone)]
pub struct VectorIndex {
    model: EmbeddingModel,
    dimension: usize,
    embeddings: Vec<Embedding>,
}

impl VectorIndex {
    /// Creates an index after enforcing a single model, dimension, and source key per entry.
    pub fn from_embeddings(mut embeddings: Vec<Embedding>) -> Result<Self, EmbeddingError> {
        embeddings.sort_by(|left, right| left.source_key.cmp(&right.source_key));
        let Some(first) = embeddings.first() else {
            return Ok(Self {
                model: EmbeddingModel::new("empty-index", "v1")?,
                dimension: 0,
                embeddings,
            });
        };
        let model = first.model.clone();
        let dimension = first.vector.dimension();
        let mut source_keys = BTreeSet::new();
        for embedding in &embeddings {
            if !source_keys.insert(embedding.source_key.clone()) {
                return Err(EmbeddingError::DuplicateSourceKey {
                    source_key: embedding.source_key.clone(),
                });
            }
            if embedding.model != model {
                return Err(EmbeddingError::ModelMismatch);
            }
            if embedding.vector.dimension() != dimension {
                return Err(EmbeddingError::DimensionMismatch);
            }
        }
        Ok(Self {
            model,
            dimension,
            embeddings,
        })
    }

    /// Searches with deterministic descending score and ascending source-key tie-breaks.
    #[must_use]
    pub fn search(&self, query: EmbeddingVector, limit: usize) -> Vec<RankedEmbedding> {
        if limit == 0 || self.dimension == 0 || query.dimension() != self.dimension {
            return Vec::new();
        }
        let mut ranked = self
            .embeddings
            .iter()
            .filter_map(|embedding| {
                embedding
                    .vector
                    .cosine_similarity(&query)
                    .ok()
                    .map(|score| RankedEmbedding {
                        source_key: embedding.source_key.clone(),
                        embedding_id: embedding.id,
                        score,
                    })
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.source_key.cmp(&right.source_key))
                .then_with(|| left.embedding_id.cmp(&right.embedding_id))
        });
        ranked.truncate(limit);
        ranked
    }

    /// Returns the model identity accepted by the index.
    #[must_use]
    pub const fn model(&self) -> &EmbeddingModel {
        &self.model
    }
}

fn validated_text(
    field: &'static str,
    value: String,
    maximum_characters: usize,
) -> Result<String, EmbeddingError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(EmbeddingError::Empty { field });
    }
    if value.chars().count() > maximum_characters {
        return Err(EmbeddingError::TooLong {
            field,
            max: maximum_characters,
        });
    }
    Ok(value)
}

fn validate_dimension(dimension: usize) -> Result<(), EmbeddingError> {
    if dimension == 0 || dimension > MAX_DIMENSION {
        return Err(EmbeddingError::InvalidDimension { max: MAX_DIMENSION });
    }
    Ok(())
}

fn normalized_tokens(input: &str) -> Vec<String> {
    input
        .split_whitespace()
        .map(|token| token.trim_matches(|character: char| !character.is_alphanumeric()))
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn sha256_fingerprint(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    format!("sha256:{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_similarity_rejects_dimension_mismatch() {
        let left = EmbeddingVector::normalized(vec![1.0, 0.0]).expect("valid vector");
        let right = EmbeddingVector::normalized(vec![1.0]).expect("valid vector");

        assert_eq!(
            left.cosine_similarity(&right),
            Err(EmbeddingError::DimensionMismatch)
        );
    }

    #[test]
    fn same_content_and_model_produce_same_embedding_id() {
        let adapter = DeterministicEmbeddingAdapter::new(
            DeterministicEmbeddingConfig::new("local", "1", 8).expect("configuration"),
        );
        let content = EmbeddingText::new("refund policy").expect("content");

        assert_eq!(
            adapter.embed("chunk:1", &content).expect("first").id(),
            adapter.embed("chunk:1", &content).expect("second").id()
        );
    }
}
