//! Private, commit-bound component body revision contracts.

use crate::StorageRepositoryError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_context_core::{
    ComponentContent, ComponentId, ContentHash, ContextComponent, ContextComponentKind, ContextId,
    DomainValidationError,
};
use contextlab_versioning::{CommitId, ContextChangeKind, ContextCommit};
use serde_json::Value;
use thiserror::Error;

/// A newly created component body that must be attached to a matching Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContentCreationWrite {
    component: ContextComponent,
    metadata: Value,
    content: ComponentContent,
    captured_at: DateTime<Utc>,
}

impl ComponentContentCreationWrite {
    /// Creates a private component creation command with immutable initial body content.
    pub fn new(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        name: impl Into<String>,
        metadata: Value,
        content: ComponentContent,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, DomainValidationError> {
        let component = ContextComponent::with_id(
            component_id,
            component_kind,
            name,
            content.content_hash().as_str(),
        )?;

        Ok(Self {
            component,
            metadata,
            content,
            captured_at,
        })
    }

    /// Returns the version-owned component identity and immutable projection data.
    #[must_use]
    pub const fn component(&self) -> &ContextComponent {
        &self.component
    }

    /// Returns flexible component metadata without interpreting it.
    #[must_use]
    pub const fn metadata(&self) -> &Value {
        &self.metadata
    }

    /// Returns the immutable initial body content.
    #[must_use]
    pub const fn content(&self) -> &ComponentContent {
        &self.content
    }

    /// Returns the deterministic initial body hash.
    #[must_use]
    pub fn resulting_content_hash(&self) -> ContentHash {
        self.content.content_hash()
    }

    /// Returns the body capture timestamp.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }

    /// Ensures this creation is represented by the supplied Context commit.
    pub fn validate_against(
        &self,
        commit: &ContextCommit,
    ) -> Result<(), ComponentContentCreationError> {
        let resulting_content_hash = self.resulting_content_hash();
        let matches_change = commit.changes().iter().any(|change| {
            change.kind() == ContextChangeKind::AddedComponent
                && change.component_id() == Some(self.component.id())
                && change.component_kind() == Some(self.component.kind())
                && change.component_name() == Some(self.component.name())
                && change.component_metadata() == Some(&self.metadata)
                && change.previous_content_hash().is_none()
                && change.resulting_content_hash() == Some(&resulting_content_hash)
        });

        if matches_change {
            Ok(())
        } else {
            Err(ComponentContentCreationError::CommitChangeMismatch)
        }
    }

    /// Binds this initial body to the Context commit that records it.
    #[must_use]
    pub fn into_revision(
        self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> ComponentContentRevision {
        let resulting_content_hash = self.content.content_hash();

        ComponentContentRevision {
            context_id,
            commit_id,
            component_id: self.component.id(),
            component_kind: self.component.kind(),
            previous_content_hash: None,
            content: self.content,
            resulting_content_hash,
            captured_at: self.captured_at,
        }
    }
}

/// A component body update that must be attached to a matching Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContentRevisionWrite {
    component_id: ComponentId,
    component_kind: ContextComponentKind,
    previous_content_hash: ContentHash,
    content: ComponentContent,
    captured_at: DateTime<Utc>,
}

impl ComponentContentRevisionWrite {
    /// Creates a private component body revision command.
    #[must_use]
    pub fn new(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        previous_content_hash: ContentHash,
        content: ComponentContent,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            component_id,
            component_kind,
            previous_content_hash,
            content,
            captured_at,
        }
    }

    /// Returns the component being revised.
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.component_id
    }

    /// Returns the component kind recorded with the revision.
    #[must_use]
    pub const fn component_kind(&self) -> ContextComponentKind {
        self.component_kind
    }

    /// Returns the projected content hash expected before this update.
    #[must_use]
    pub const fn previous_content_hash(&self) -> &ContentHash {
        &self.previous_content_hash
    }

    /// Returns the immutable body content.
    #[must_use]
    pub const fn content(&self) -> &ComponentContent {
        &self.content
    }

    /// Returns the deterministic resulting body hash.
    #[must_use]
    pub fn resulting_content_hash(&self) -> ContentHash {
        self.content.content_hash()
    }

    /// Returns the body capture timestamp.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }

    /// Ensures this body revision is represented by the supplied Context commit.
    pub fn validate_against(
        &self,
        commit: &ContextCommit,
    ) -> Result<(), ComponentContentRevisionError> {
        let resulting_content_hash = self.resulting_content_hash();
        let matches_change = commit.changes().iter().any(|change| {
            change.kind() == ContextChangeKind::UpdatedComponent
                && change.component_id() == Some(self.component_id)
                && change.component_kind() == Some(self.component_kind)
                && change.previous_content_hash() == Some(&self.previous_content_hash)
                && change.resulting_content_hash() == Some(&resulting_content_hash)
        });

        if matches_change {
            Ok(())
        } else {
            Err(ComponentContentRevisionError::CommitChangeMismatch)
        }
    }

    /// Binds this validated write to the Context and commit that record it.
    #[must_use]
    pub fn into_revision(
        self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> ComponentContentRevision {
        let resulting_content_hash = self.resulting_content_hash();

        ComponentContentRevision {
            context_id,
            commit_id,
            component_id: self.component_id,
            component_kind: self.component_kind,
            previous_content_hash: Some(self.previous_content_hash),
            content: self.content,
            resulting_content_hash,
            captured_at: self.captured_at,
        }
    }
}

/// Immutable component body revision stored with a Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContentRevision {
    context_id: ContextId,
    commit_id: CommitId,
    component_id: ComponentId,
    component_kind: ContextComponentKind,
    previous_content_hash: Option<ContentHash>,
    content: ComponentContent,
    resulting_content_hash: ContentHash,
    captured_at: DateTime<Utc>,
}

/// Verified storage fields required to rehydrate one immutable revision.
pub(crate) struct PersistedComponentContentRevision {
    pub(crate) context_id: ContextId,
    pub(crate) commit_id: CommitId,
    pub(crate) component_id: ComponentId,
    pub(crate) component_kind: ContextComponentKind,
    pub(crate) previous_content_hash: Option<ContentHash>,
    pub(crate) content: ComponentContent,
    pub(crate) resulting_content_hash: ContentHash,
    pub(crate) captured_at: DateTime<Utc>,
}

impl ComponentContentRevision {
    /// Rehydrates a stored immutable revision after its storage invariants are verified.
    #[must_use]
    pub(crate) fn from_persisted(input: PersistedComponentContentRevision) -> Self {
        Self {
            context_id: input.context_id,
            commit_id: input.commit_id,
            component_id: input.component_id,
            component_kind: input.component_kind,
            previous_content_hash: input.previous_content_hash,
            content: input.content,
            resulting_content_hash: input.resulting_content_hash,
            captured_at: input.captured_at,
        }
    }

    /// Returns the Context that owns this revision.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the commit that records this revision.
    #[must_use]
    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    /// Returns the component that owns this revision.
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.component_id
    }

    /// Returns the component kind recorded with this revision.
    #[must_use]
    pub const fn component_kind(&self) -> ContextComponentKind {
        self.component_kind
    }

    /// Returns the projected content hash before the revision.
    #[must_use]
    pub const fn previous_content_hash(&self) -> Option<&ContentHash> {
        self.previous_content_hash.as_ref()
    }

    /// Returns the immutable body content.
    #[must_use]
    pub const fn content(&self) -> &ComponentContent {
        &self.content
    }

    /// Returns the deterministic body hash after the revision.
    #[must_use]
    pub const fn resulting_content_hash(&self) -> &ContentHash {
        &self.resulting_content_hash
    }

    /// Returns the recorded capture timestamp.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }
}

/// Validation errors for private component body revision commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ComponentContentRevisionError {
    /// The commit does not contain the exact component hash transition.
    #[error("component content revision must match an updated component change")]
    CommitChangeMismatch,
}

/// Validation errors for private component creation commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ComponentContentCreationError {
    /// The commit does not contain the exact initial component body hash.
    #[error("component content creation must match an added component change")]
    CommitChangeMismatch,
    /// The commit snapshot does not project the component node or relationship being created.
    #[error("component content creation must match the commit snapshot graph")]
    SnapshotGraphMismatch,
}

pub(crate) const fn component_kind_storage_value(kind: ContextComponentKind) -> &'static str {
    match kind {
        ContextComponentKind::Prompt => "prompt",
        ContextComponentKind::SystemPrompt => "system_prompt",
        ContextComponentKind::Memory => "memory",
        ContextComponentKind::Knowledge => "knowledge",
        ContextComponentKind::Retrieval => "retrieval",
        ContextComponentKind::Embedding => "embedding",
        ContextComponentKind::ModelConfiguration => "model_configuration",
        ContextComponentKind::Tool => "tool",
        ContextComponentKind::McpServer => "mcp_server",
        ContextComponentKind::Variable => "variable",
        ContextComponentKind::OutputSchema => "output_schema",
        ContextComponentKind::Workflow => "workflow",
        ContextComponentKind::Conversation => "conversation",
        ContextComponentKind::Evaluation => "evaluation",
    }
}

pub(crate) fn parse_component_kind_storage_value(value: &str) -> Option<ContextComponentKind> {
    Some(match value {
        "prompt" => ContextComponentKind::Prompt,
        "system_prompt" => ContextComponentKind::SystemPrompt,
        "memory" => ContextComponentKind::Memory,
        "knowledge" => ContextComponentKind::Knowledge,
        "retrieval" => ContextComponentKind::Retrieval,
        "embedding" => ContextComponentKind::Embedding,
        "model_configuration" => ContextComponentKind::ModelConfiguration,
        "tool" => ContextComponentKind::Tool,
        "mcp_server" => ContextComponentKind::McpServer,
        "variable" => ContextComponentKind::Variable,
        "output_schema" => ContextComponentKind::OutputSchema,
        "workflow" => ContextComponentKind::Workflow,
        "conversation" => ContextComponentKind::Conversation,
        "evaluation" => ContextComponentKind::Evaluation,
        _ => return None,
    })
}

/// Private read access to component body revisions.
#[async_trait]
pub trait ComponentContentRevisionRepository: Send + Sync {
    /// Returns a body revision by its exact Context, commit, and component identity.
    async fn get_component_content_revision(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
        component_id: ComponentId,
    ) -> Result<Option<ComponentContentRevision>, StorageRepositoryError>;

    /// Resolves the nearest body revision on a target commit's normal first-parent ancestry.
    async fn get_component_content_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
        component_id: ComponentId,
    ) -> Result<Option<ComponentContentRevision>, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_versioning::{BranchName, ContextChange};

    #[test]
    fn revision_binds_the_matching_component_hash_transition() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let previous_content_hash = ContentHash::new("sha256:previous").expect("previous hash");
        let content = ComponentContent::new("new body");
        let resulting_content_hash = content.content_hash();
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Update prompt body",
            Vec::new(),
            vec![ContextChange::updated_component_content(
                component_id,
                ContextComponentKind::Prompt,
                previous_content_hash.clone(),
                resulting_content_hash.clone(),
                "Update prompt body",
            )],
            Utc::now(),
        )
        .expect("commit");
        let write = ComponentContentRevisionWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            previous_content_hash.clone(),
            content,
            Utc::now(),
        );

        write.validate_against(&commit).expect("matching change");
        let revision = write.into_revision(context_id, commit.id());

        assert_eq!(
            revision.previous_content_hash(),
            Some(&previous_content_hash)
        );
        assert_eq!(revision.resulting_content_hash(), &resulting_content_hash);
    }

    #[test]
    fn creation_binds_added_component_hash_to_initial_revision() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let content = ComponentContent::new("initial prompt body");
        let resulting_content_hash = content.content_hash();
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create initial prompt body",
            Vec::new(),
            vec![
                ContextChange::added_component_content_with_details(
                    component_id,
                    ContextComponentKind::Prompt,
                    "Initial Prompt",
                    serde_json::json!({"locale": "en"}),
                    resulting_content_hash.clone(),
                    "Create initial prompt body",
                )
                .expect("valid replayable component change"),
            ],
            Utc::now(),
        )
        .expect("commit");
        let creation = ComponentContentCreationWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            "Initial Prompt",
            serde_json::json!({"locale": "en"}),
            content,
            Utc::now(),
        )
        .expect("valid creation");

        creation
            .validate_against(&commit)
            .expect("matching added component change");
        let revision = creation.into_revision(context_id, commit.id());

        assert_eq!(revision.previous_content_hash(), None);
        assert_eq!(revision.resulting_content_hash(), &resulting_content_hash);
    }
}
