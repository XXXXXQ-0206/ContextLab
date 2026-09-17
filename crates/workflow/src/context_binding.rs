//! Immutable Workflow provenance over one exact Context commit.

use crate::{WorkflowDefinition, WorkflowId, WorkflowRevision};
use contextlab_context_core::ContextId;
use contextlab_versioning::CommitId;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

/// Stable identifier for an immutable Context-to-Workflow source binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorkflowContextBindingId(Uuid);

impl WorkflowContextBindingId {
    /// Reconstructs a binding identifier from a UUID.
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

/// The exact immutable Context commit used as one Workflow definition's source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextCommitSource {
    context_id: ContextId,
    commit_id: CommitId,
}

impl ContextCommitSource {
    /// Creates a source reference from canonical Context and commit identifiers.
    #[must_use]
    pub const fn new(context_id: ContextId, commit_id: CommitId) -> Self {
        Self {
            context_id,
            commit_id,
        }
    }

    /// Returns the Context aggregate scope.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the immutable Context commit identity.
    #[must_use]
    pub const fn commit_id(self) -> CommitId {
        self.commit_id
    }
}

/// A sealed Workflow definition attached to one exact immutable Context commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkflowContextBinding {
    id: WorkflowContextBindingId,
    context_source: ContextCommitSource,
    workflow_definition: WorkflowDefinition,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowContextBindingWire {
    id: WorkflowContextBindingId,
    context_source: ContextCommitSource,
    workflow_definition: WorkflowDefinition,
}

impl<'de> Deserialize<'de> for WorkflowContextBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = WorkflowContextBindingWire::deserialize(deserializer)?;
        Ok(Self::new(
            wire.id,
            wire.workflow_definition,
            wire.context_source,
        ))
    }
}

impl WorkflowContextBinding {
    /// Seals one immutable Workflow definition to one exact immutable Context commit.
    #[must_use]
    pub const fn new(
        id: WorkflowContextBindingId,
        workflow_definition: WorkflowDefinition,
        context_source: ContextCommitSource,
    ) -> Self {
        Self {
            id,
            context_source,
            workflow_definition,
        }
    }

    /// Returns the stable source-binding identifier.
    #[must_use]
    pub const fn id(&self) -> WorkflowContextBindingId {
        self.id
    }

    /// Returns the exact immutable Context source.
    #[must_use]
    pub const fn context_source(&self) -> ContextCommitSource {
        self.context_source
    }

    /// Returns the sealed workflow definition used during replay.
    #[must_use]
    pub const fn workflow_definition(&self) -> &WorkflowDefinition {
        &self.workflow_definition
    }

    /// Returns the stable Workflow identity embedded in the sealed definition.
    #[must_use]
    pub const fn workflow_id(&self) -> WorkflowId {
        self.workflow_definition.id()
    }

    /// Returns the immutable Workflow revision embedded in the sealed definition.
    #[must_use]
    pub const fn workflow_revision(&self) -> WorkflowRevision {
        self.workflow_definition.revision()
    }
}
