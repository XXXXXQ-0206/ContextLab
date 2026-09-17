//! Context component model.

use crate::{ComponentId, DomainValidationError, NonEmptyString};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hash of component content stored outside this aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(NonEmptyString);

impl ContentHash {
    /// Creates a content hash from a digest string.
    pub fn new(value: impl Into<String>) -> Result<Self, DomainValidationError> {
        Ok(Self(NonEmptyString::new("content_hash", value)?))
    }

    /// Returns the digest string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// UTF-8 body content for a Context component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentContent(String);

impl ComponentContent {
    /// Creates component body content without interpreting its component kind.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the exact UTF-8 body content.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the deterministic SHA-256 fingerprint of this content.
    #[must_use]
    pub fn content_hash(&self) -> ContentHash {
        let digest = Sha256::digest(self.0.as_bytes());
        ContentHash::new(format!("sha256:{digest:x}"))
            .expect("SHA-256 fingerprints are always non-empty")
    }
}

/// A first-class kind of data that can influence an AI model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextComponentKind {
    /// User or developer prompt content.
    Prompt,
    /// System-level instruction content.
    SystemPrompt,
    /// Durable memory or time-scoped memory.
    Memory,
    /// Knowledge-base document, chunk, citation, or reference.
    Knowledge,
    /// Retrieval configuration or query source.
    Retrieval,
    /// Embedding configuration or vector references.
    Embedding,
    /// Model provider and inference parameters.
    ModelConfiguration,
    /// Tool definition or execution affordance.
    Tool,
    /// MCP server definition.
    McpServer,
    /// Runtime variable.
    Variable,
    /// Structured output contract.
    OutputSchema,
    /// Workflow graph or executable step.
    Workflow,
    /// Conversation transcript or replay artifact.
    Conversation,
    /// Evaluation dataset, suite, metric, or run.
    Evaluation,
}

/// A named component attached to a Context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextComponent {
    id: ComponentId,
    kind: ContextComponentKind,
    name: NonEmptyString,
    content_hash: ContentHash,
}

impl ContextComponent {
    /// Creates a component with a generated identifier.
    pub fn new(
        kind: ContextComponentKind,
        name: impl Into<String>,
        content_hash: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        Self::with_id(ComponentId::new(), kind, name, content_hash)
    }

    /// Rehydrates a component that already has a stable version-owned identifier.
    pub fn with_id(
        id: ComponentId,
        kind: ContextComponentKind,
        name: impl Into<String>,
        content_hash: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        Ok(Self {
            id,
            kind,
            name: NonEmptyString::new("component.name", name)?,
            content_hash: ContentHash::new(content_hash)?,
        })
    }

    /// Returns the component identifier.
    #[must_use]
    pub const fn id(&self) -> ComponentId {
        self.id
    }

    /// Returns the component kind.
    #[must_use]
    pub const fn kind(&self) -> ContextComponentKind {
        self.kind
    }

    /// Returns the component name.
    #[must_use]
    pub fn name(&self) -> &NonEmptyString {
        &self.name
    }

    /// Returns the content hash.
    #[must_use]
    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_prompt_component() {
        let component = ContextComponent::new(
            ContextComponentKind::Prompt,
            "Instruction Prompt",
            "sha256:abc123",
        )
        .expect("valid component");

        assert_eq!(component.kind(), ContextComponentKind::Prompt);
        assert_eq!(component.name().as_str(), "Instruction Prompt");
        assert_eq!(component.content_hash().as_str(), "sha256:abc123");
    }

    #[test]
    fn fingerprints_utf8_component_content_deterministically() {
        let content = ComponentContent::new("system instruction: return JSON");
        let same_content = ComponentContent::new("system instruction: return JSON");
        let changed_content = ComponentContent::new("system instruction: return text");

        assert_eq!(content.as_str(), "system instruction: return JSON");
        assert_eq!(
            content.content_hash().as_str(),
            "sha256:d6b60502a94547e6b5c4723afcd292407b0f8dbe74323690a5b8f07263958d68"
        );
        assert_eq!(content.content_hash(), same_content.content_hash());
        assert_ne!(content.content_hash(), changed_content.content_hash());
    }
}
