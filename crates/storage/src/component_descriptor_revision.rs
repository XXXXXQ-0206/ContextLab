//! Private, commit-bound component descriptor revision contracts.

use crate::StorageRepositoryError;
use chrono::{DateTime, Utc};
use contextlab_context_core::{ComponentId, ContextComponent, ContextComponentKind};
use contextlab_versioning::{ContextChangeKind, ContextCommit};
use serde_json::Value;
use thiserror::Error;

/// A component descriptor replacement attached to a matching Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDescriptorRevisionWrite {
    component: ContextComponent,
    metadata: Value,
    captured_at: DateTime<Utc>,
}

impl ComponentDescriptorRevisionWrite {
    /// Creates a private descriptor revision from the replacement component state.
    #[must_use]
    pub const fn new(
        component: ContextComponent,
        metadata: Value,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            component,
            metadata,
            captured_at,
        }
    }

    /// Returns the replacement component descriptor and body hash witness.
    #[must_use]
    pub const fn component(&self) -> &ContextComponent {
        &self.component
    }

    /// Returns the replacement metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Value {
        &self.metadata
    }

    /// Returns the descriptor capture timestamp.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }

    /// Ensures this descriptor replacement is represented by the supplied commit.
    pub fn validate_against(
        &self,
        commit: &ContextCommit,
    ) -> Result<(), ComponentDescriptorRevisionError> {
        let matches_change = commit.changes().iter().any(|change| {
            change.kind() == ContextChangeKind::UpdatedComponentDescriptor
                && change.component_id() == Some(self.component.id())
                && change.component_kind() == Some(self.component.kind())
                && change.component_name() == Some(self.component.name())
                && change.component_metadata() == Some(&self.metadata)
                && change.previous_content_hash().is_none()
                && change.resulting_content_hash().is_none()
        });

        if matches_change {
            Ok(())
        } else {
            Err(ComponentDescriptorRevisionError::CommitChangeMismatch)
        }
    }
}

/// Maps descriptor mutation failures to the guarded storage boundary.
impl From<ComponentDescriptorRevisionError> for StorageRepositoryError {
    fn from(error: ComponentDescriptorRevisionError) -> Self {
        Self::ComponentContentRevisionConflict {
            reason: error.to_string(),
        }
    }
}

/// Validation failures for a private descriptor revision attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ComponentDescriptorRevisionError {
    /// The commit does not contain the exact descriptor replacement.
    #[error("component descriptor revision must match an updated descriptor change")]
    CommitChangeMismatch,
    /// The commit snapshot does not project the replacement component node.
    #[error("component descriptor revision must match the commit snapshot graph")]
    SnapshotGraphMismatch,
}

impl ComponentDescriptorRevisionWrite {
    /// Returns the component identity.
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.component.id()
    }

    /// Returns the component taxonomy kind.
    #[must_use]
    pub const fn component_kind(&self) -> ContextComponentKind {
        self.component.kind()
    }
}
