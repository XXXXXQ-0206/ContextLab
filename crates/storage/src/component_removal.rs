//! Private component-removal transition contracts.

use contextlab_context_core::{ComponentId, ContentHash, ContextComponentKind};
use contextlab_versioning::{ContextChangeKind, ContextCommit};
use thiserror::Error;

/// A component removal that must be attached to a matching Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentRemovalWrite {
    component_id: ComponentId,
    component_kind: ContextComponentKind,
    previous_content_hash: ContentHash,
}

impl ComponentRemovalWrite {
    /// Creates a private component-removal command with its final content hash precondition.
    #[must_use]
    pub const fn new(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        previous_content_hash: ContentHash,
    ) -> Self {
        Self {
            component_id,
            component_kind,
            previous_content_hash,
        }
    }

    /// Returns the component being removed.
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.component_id
    }

    /// Returns the component kind that must still be active.
    #[must_use]
    pub const fn component_kind(&self) -> ContextComponentKind {
        self.component_kind
    }

    /// Returns the final projected content hash expected before removal.
    #[must_use]
    pub const fn previous_content_hash(&self) -> &ContentHash {
        &self.previous_content_hash
    }

    /// Ensures this removal is represented by the supplied Context commit.
    pub fn validate_against(&self, commit: &ContextCommit) -> Result<(), ComponentRemovalError> {
        let matches_change = commit.changes().iter().any(|change| {
            change.kind() == ContextChangeKind::RemovedComponent
                && change.component_id() == Some(self.component_id)
                && change.component_kind() == Some(self.component_kind)
                && change.previous_content_hash() == Some(&self.previous_content_hash)
                && change.resulting_content_hash().is_none()
                && change.component_name().is_none()
                && change.component_metadata().is_none()
        });

        if matches_change {
            Ok(())
        } else {
            Err(ComponentRemovalError::CommitChangeMismatch)
        }
    }
}

/// Validation errors for a private component-removal command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ComponentRemovalError {
    /// The commit does not contain the exact typed removal transition.
    #[error("component removal does not match the Context commit change")]
    CommitChangeMismatch,
    /// The commit graph snapshot still contains the removed component node.
    #[error("component removal snapshot still contains the component node")]
    SnapshotGraphMismatch,
}
