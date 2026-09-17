//! Deterministic, provider-free replay of a linear Context history.

use crate::{CommitId, ContextChange, ContextChangeKind, ContextCommit, ContextMetadataPayload};
use contextlab_context_core::{
    ComponentId, ContentHash, ContextComponentKind, ContextId, NonEmptyString,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Schema version for the private deterministic replay state contract.
pub const REPLAY_STATE_SCHEMA_VERSION: u16 = 1;

/// The descriptor-only state of one active Context component at a commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayComponentState {
    component_id: ComponentId,
    kind: ContextComponentKind,
    name: NonEmptyString,
    metadata: Value,
    content_hash: ContentHash,
}

impl ReplayComponentState {
    /// Returns the stable component identifier.
    #[must_use]
    pub const fn component_id(&self) -> ComponentId {
        self.component_id
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

    /// Returns the descriptor metadata.
    #[must_use]
    pub const fn metadata(&self) -> &Value {
        &self.metadata
    }

    /// Returns the effective body hash.
    #[must_use]
    pub const fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
}

/// One active directed `Uses` relationship in replayed state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayRelationship {
    /// Source component.
    pub source_component_id: ComponentId,
    /// Target component.
    pub target_component_id: ComponentId,
}

/// The stable V1 wire envelope for one exact replay state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayStateSnapshotV1 {
    schema_version: u16,
    context_id: ContextId,
    initialized: bool,
    commit_id: Option<CommitId>,
    context_metadata: Option<ContextMetadataPayload>,
    components: Vec<ReplayComponentState>,
    relationships: Vec<ReplayRelationship>,
}

impl ReplayStateSnapshotV1 {
    /// Returns the explicit envelope schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the Context represented by this envelope.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the exact commit represented by this envelope, if any.
    #[must_use]
    pub const fn commit_id(&self) -> Option<CommitId> {
        self.commit_id
    }

    /// Returns whether the Context creation transition has been replayed.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns components in canonical component-ID order.
    #[must_use]
    pub fn components(&self) -> &[ReplayComponentState] {
        &self.components
    }

    /// Returns relationships in canonical endpoint order.
    #[must_use]
    pub fn relationships(&self) -> &[ReplayRelationship] {
        &self.relationships
    }
}

/// Errors that prevent a deterministic state replay.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReplayError {
    /// The serialized envelope uses an unsupported schema version.
    #[error("replay state schema version {actual} is unsupported; expected {expected}")]
    UnsupportedSchema {
        /// Schema version received from the envelope.
        actual: u16,
        /// Only schema version currently supported by this crate.
        expected: u16,
    },
    /// The serialized envelope violates the replay-state invariants.
    #[error("invalid replay state snapshot: {reason}")]
    InvalidSnapshot {
        /// Stable validation reason.
        reason: &'static str,
    },
    /// The commit belongs to another Context.
    #[error("replay commit belongs to Context {actual}, expected {expected}")]
    CrossContext {
        /// Context selected by the replay cursor.
        expected: ContextId,
        /// Context declared by the commit.
        actual: ContextId,
    },
    /// The commit does not extend the current linear replay cursor.
    #[error("replay parent does not match current cursor")]
    ParentMismatch {
        /// Current replay cursor.
        expected: Option<CommitId>,
        /// Parents supplied by the commit.
        actual: Vec<CommitId>,
    },
    /// Merge commits are outside this deterministic first-parent contract.
    #[error("replay accepts exactly one normal parent after initialization")]
    UnsupportedParentShape,
    /// A change payload does not carry enough information to rebuild state.
    #[error("change {change} lacks a complete replay payload")]
    MissingPayload {
        /// Change kind whose payload is incomplete.
        change: &'static str,
    },
    /// The requested transition is invalid for the current state.
    #[error("invalid {change} transition for component {component_id}")]
    InvalidTransition {
        /// Change kind being applied.
        change: &'static str,
        /// Component or relationship identity involved in the failure.
        component_id: String,
    },
    /// The change kind is not represented by this descriptor-only state.
    #[error("change {change} is not supported by descriptor replay")]
    UnsupportedChange {
        /// Unsupported change kind.
        change: &'static str,
    },
}

/// A deterministic descriptor and relationship state at one linear commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayState {
    context_id: ContextId,
    initialized: bool,
    last_commit_id: Option<CommitId>,
    context_metadata: Option<ContextMetadataPayload>,
    components: BTreeMap<String, ReplayComponentState>,
    relationships: BTreeSet<(String, String)>,
}

impl ReplayState {
    /// Creates an empty replay cursor for one Context.
    #[must_use]
    pub fn new(context_id: ContextId) -> Self {
        Self {
            context_id,
            initialized: false,
            last_commit_id: None,
            context_metadata: None,
            components: BTreeMap::new(),
            relationships: BTreeSet::new(),
        }
    }

    /// Replays an ordered normal-parent commit history to one exact state.
    pub fn from_commits(
        context_id: ContextId,
        commits: &[ContextCommit],
    ) -> Result<Self, ReplayError> {
        let mut state = Self::new(context_id);
        for commit in commits {
            state.apply_commit(commit)?;
        }
        Ok(state)
    }

    /// Returns the explicit schema version of this private projection.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        REPLAY_STATE_SCHEMA_VERSION
    }

    /// Returns the Context scope.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the commit represented by this state, if any.
    #[must_use]
    pub const fn commit_id(&self) -> Option<CommitId> {
        self.last_commit_id
    }

    /// Returns whether the Context creation transition has been replayed.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns the latest typed Context metadata, if replayed.
    #[must_use]
    pub const fn context_metadata(&self) -> Option<&ContextMetadataPayload> {
        self.context_metadata.as_ref()
    }

    /// Returns active components in stable component-ID order.
    pub fn components(&self) -> impl Iterator<Item = &ReplayComponentState> {
        self.components.values()
    }

    /// Finds an active component by exact stable ID.
    #[must_use]
    pub fn component(&self, component_id: ComponentId) -> Option<&ReplayComponentState> {
        self.components.get(&component_key(component_id))
    }

    /// Returns active relationships in stable endpoint order.
    pub fn relationships(&self) -> impl Iterator<Item = ReplayRelationship> + '_ {
        self.relationships
            .iter()
            .map(|(source, target)| ReplayRelationship {
                source_component_id: component_id_from_key(source),
                target_component_id: component_id_from_key(target),
            })
    }

    /// Converts this state to its canonical V1 serialization envelope.
    #[must_use]
    pub fn to_snapshot(&self) -> ReplayStateSnapshotV1 {
        ReplayStateSnapshotV1 {
            schema_version: REPLAY_STATE_SCHEMA_VERSION,
            context_id: self.context_id,
            initialized: self.initialized,
            commit_id: self.last_commit_id,
            context_metadata: self.context_metadata.clone(),
            components: self.components.values().cloned().collect(),
            relationships: self.relationships().collect(),
        }
    }

    /// Converts this state to a canonical JSON value.
    pub fn to_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::to_value(self.to_snapshot())
    }

    /// Restores a state from a validated V1 serialization envelope.
    pub fn from_snapshot(snapshot: ReplayStateSnapshotV1) -> Result<Self, ReplayError> {
        if snapshot.schema_version != REPLAY_STATE_SCHEMA_VERSION {
            return Err(ReplayError::UnsupportedSchema {
                actual: snapshot.schema_version,
                expected: REPLAY_STATE_SCHEMA_VERSION,
            });
        }
        if snapshot.context_id.as_uuid().is_nil() {
            return Err(ReplayError::InvalidSnapshot {
                reason: "Context identifier is nil",
            });
        }
        if snapshot
            .commit_id
            .is_some_and(|commit_id| commit_id.as_uuid().is_nil())
        {
            return Err(ReplayError::InvalidSnapshot {
                reason: "commit identifier is nil",
            });
        }

        let mut components = BTreeMap::new();
        for component in snapshot.components {
            if component.component_id().as_uuid().is_nil() {
                return Err(ReplayError::InvalidSnapshot {
                    reason: "component identifier is nil",
                });
            }
            let key = component_key(component.component_id());
            if components.insert(key, component).is_some() {
                return Err(ReplayError::InvalidSnapshot {
                    reason: "duplicate component identifier",
                });
            }
        }

        let mut relationships = BTreeSet::new();
        for relationship in snapshot.relationships {
            if relationship.source_component_id.as_uuid().is_nil()
                || relationship.target_component_id.as_uuid().is_nil()
            {
                return Err(ReplayError::InvalidSnapshot {
                    reason: "relationship endpoint identifier is nil",
                });
            }
            let source = component_key(relationship.source_component_id);
            let target = component_key(relationship.target_component_id);
            if !components.contains_key(&source) || !components.contains_key(&target) {
                return Err(ReplayError::InvalidSnapshot {
                    reason: "relationship endpoint is absent from components",
                });
            }
            if !relationships.insert((source, target)) {
                return Err(ReplayError::InvalidSnapshot {
                    reason: "duplicate relationship",
                });
            }
        }

        Ok(Self {
            context_id: snapshot.context_id,
            initialized: snapshot.initialized,
            last_commit_id: snapshot.commit_id,
            context_metadata: snapshot.context_metadata,
            components,
            relationships,
        })
    }

    /// Restores a state from a V1 JSON value and rejects schema drift.
    pub fn from_json(value: Value) -> Result<Self, ReplayError> {
        let snapshot = serde_json::from_value::<ReplayStateSnapshotV1>(value).map_err(|_| {
            ReplayError::InvalidSnapshot {
                reason: "invalid replay state JSON envelope",
            }
        })?;
        Self::from_snapshot(snapshot)
    }

    /// Applies one commit atomically to this replay cursor.
    pub fn apply_commit(&mut self, commit: &ContextCommit) -> Result<(), ReplayError> {
        if commit.context_id() != self.context_id {
            return Err(ReplayError::CrossContext {
                expected: self.context_id,
                actual: commit.context_id(),
            });
        }

        let expected_parent = self.last_commit_id;
        let actual_parents = commit.parent_ids();
        let parent_matches = match expected_parent {
            None => actual_parents.is_empty(),
            Some(expected) => actual_parents == [expected],
        };
        if !parent_matches {
            if expected_parent.is_some() && actual_parents.len() != 1 {
                return Err(ReplayError::UnsupportedParentShape);
            }
            return Err(ReplayError::ParentMismatch {
                expected: expected_parent,
                actual: actual_parents.to_vec(),
            });
        }

        let mut next = self.clone();
        for change in commit.changes() {
            next.apply_change(change)?;
        }
        next.last_commit_id = Some(commit.id());
        self.clone_from(&next);
        Ok(())
    }

    fn apply_change(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        match change.kind() {
            ContextChangeKind::CreatedContext => {
                if self.initialized {
                    return Err(ReplayError::InvalidTransition {
                        change: "created_context",
                        component_id: "context".to_owned(),
                    });
                }
                self.initialized = true;
            }
            ContextChangeKind::AddedComponent => self.add_component(change)?,
            ContextChangeKind::UpdatedComponent => self.update_content(change)?,
            ContextChangeKind::UpdatedComponentDescriptor => self.update_descriptor(change)?,
            ContextChangeKind::RemovedComponent => self.remove_component(change)?,
            ContextChangeKind::AddedUsesRelationship => self.add_relationship(change)?,
            ContextChangeKind::RemovedUsesRelationship => self.remove_relationship(change)?,
            ContextChangeKind::UpdatedMetadata => self.update_metadata(change)?,
        }
        Ok(())
    }

    fn add_component(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        let component_id = required_component_id(change, "added_component")?;
        let kind = required_kind(change, "added_component")?;
        let name = change
            .component_name()
            .cloned()
            .ok_or(ReplayError::MissingPayload {
                change: "added_component",
            })?;
        let metadata = change
            .component_metadata()
            .cloned()
            .ok_or(ReplayError::MissingPayload {
                change: "added_component",
            })?;
        let content_hash =
            change
                .resulting_content_hash()
                .cloned()
                .ok_or(ReplayError::MissingPayload {
                    change: "added_component",
                })?;
        let key = component_key(component_id);
        if self.components.contains_key(&key) {
            return Err(invalid_transition("added_component", component_id));
        }
        self.components.insert(
            key,
            ReplayComponentState {
                component_id,
                kind,
                name,
                metadata,
                content_hash,
            },
        );
        Ok(())
    }

    fn update_content(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        let component_id = required_component_id(change, "updated_component")?;
        let state = self
            .components
            .get_mut(&component_key(component_id))
            .ok_or_else(|| invalid_transition("updated_component", component_id))?;
        if Some(state.kind) != change.component_kind()
            || Some(&state.content_hash) != change.previous_content_hash()
        {
            return Err(invalid_transition("updated_component", component_id));
        }
        state.content_hash =
            change
                .resulting_content_hash()
                .cloned()
                .ok_or(ReplayError::MissingPayload {
                    change: "updated_component",
                })?;
        Ok(())
    }

    fn update_descriptor(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        let component_id = required_component_id(change, "updated_component_descriptor")?;
        let state = self
            .components
            .get_mut(&component_key(component_id))
            .ok_or_else(|| invalid_transition("updated_component_descriptor", component_id))?;
        if Some(state.kind) != change.component_kind() {
            return Err(invalid_transition(
                "updated_component_descriptor",
                component_id,
            ));
        }
        state.name = change
            .component_name()
            .cloned()
            .ok_or(ReplayError::MissingPayload {
                change: "updated_component_descriptor",
            })?;
        state.metadata =
            change
                .component_metadata()
                .cloned()
                .ok_or(ReplayError::MissingPayload {
                    change: "updated_component_descriptor",
                })?;
        Ok(())
    }

    fn update_metadata(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        self.context_metadata = Some(change.context_metadata().cloned().ok_or(
            ReplayError::MissingPayload {
                change: "updated_metadata",
            },
        )?);
        Ok(())
    }

    fn remove_component(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        let component_id = required_component_id(change, "removed_component")?;
        let key = component_key(component_id);
        let state = self
            .components
            .get(&key)
            .ok_or_else(|| invalid_transition("removed_component", component_id))?;
        if Some(state.kind) != change.component_kind()
            || Some(&state.content_hash) != change.previous_content_hash()
        {
            return Err(invalid_transition("removed_component", component_id));
        }
        self.components.remove(&key);
        self.relationships
            .retain(|(source, target)| source != &key && target != &key);
        Ok(())
    }

    fn add_relationship(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        let relationship = relationship(change, "added_uses_relationship")?;
        if !self.components.contains_key(&relationship.0)
            || !self.components.contains_key(&relationship.1)
            || !self.relationships.insert(relationship.clone())
        {
            return Err(ReplayError::InvalidTransition {
                change: "added_uses_relationship",
                component_id: format!("{} -> {}", relationship.0, relationship.1),
            });
        }
        Ok(())
    }

    fn remove_relationship(&mut self, change: &ContextChange) -> Result<(), ReplayError> {
        let relationship = relationship(change, "removed_uses_relationship")?;
        if !self.relationships.remove(&relationship) {
            return Err(ReplayError::InvalidTransition {
                change: "removed_uses_relationship",
                component_id: format!("{} -> {}", relationship.0, relationship.1),
            });
        }
        Ok(())
    }
}

fn required_component_id(
    change: &ContextChange,
    kind: &'static str,
) -> Result<ComponentId, ReplayError> {
    change
        .component_id()
        .ok_or(ReplayError::MissingPayload { change: kind })
}

fn required_kind(
    change: &ContextChange,
    kind: &'static str,
) -> Result<ContextComponentKind, ReplayError> {
    change
        .component_kind()
        .ok_or(ReplayError::MissingPayload { change: kind })
}

fn relationship(
    change: &ContextChange,
    kind: &'static str,
) -> Result<(String, String), ReplayError> {
    let source = change
        .source_component_id()
        .ok_or(ReplayError::MissingPayload { change: kind })?;
    let target = change
        .target_component_id()
        .ok_or(ReplayError::MissingPayload { change: kind })?;
    Ok((component_key(source), component_key(target)))
}

fn invalid_transition(change: &'static str, component_id: ComponentId) -> ReplayError {
    ReplayError::InvalidTransition {
        change,
        component_id: component_id.to_string(),
    }
}

fn component_key(component_id: ComponentId) -> String {
    component_id.to_string()
}

fn component_id_from_key(key: &str) -> ComponentId {
    let uuid = uuid::Uuid::parse_str(key).expect("ReplayState keys are validated UUID strings");
    ComponentId::from_uuid(uuid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BranchName;
    use chrono::Utc;
    use serde_json::json;

    fn commit(
        context_id: ContextId,
        parent_ids: Vec<CommitId>,
        changes: Vec<ContextChange>,
    ) -> ContextCommit {
        ContextCommit::new(
            context_id,
            BranchName::default(),
            "replay",
            parent_ids,
            changes,
            Utc::now(),
        )
        .expect("valid commit")
    }

    fn detailed_add(component_id: ComponentId, hash: &str) -> ContextChange {
        ContextChange::added_component_content_with_details(
            component_id,
            ContextComponentKind::Prompt,
            "Prompt",
            json!({"priority": 1}),
            ContentHash::new(hash).expect("hash"),
            "add",
        )
        .expect("valid change")
    }

    #[test]
    fn folds_detailed_component_and_relationship_history_deterministically() {
        let context_id = ContextId::new();
        let first = ComponentId::new();
        let second = ComponentId::new();
        let root = commit(
            context_id,
            Vec::new(),
            vec![
                ContextChange::created_context("root"),
                detailed_add(first, "sha256:first"),
            ],
        );
        let child = commit(
            context_id,
            vec![root.id()],
            vec![detailed_add(second, "sha256:second")],
        );
        let tip = commit(
            context_id,
            vec![child.id()],
            vec![
                ContextChange::added_uses_relationship(first, second, "wire").expect("relation"),
                ContextChange::updated_component_content(
                    first,
                    ContextComponentKind::Prompt,
                    ContentHash::new("sha256:first").expect("hash"),
                    ContentHash::new("sha256:updated").expect("hash"),
                    "update",
                ),
            ],
        );

        let tip_id = tip.id();
        let state = ReplayState::from_commits(context_id, &[root, child, tip])
            .expect("ordered history replay");

        assert_eq!(state.schema_version(), REPLAY_STATE_SCHEMA_VERSION);
        assert_eq!(state.commit_id(), Some(tip_id));
        assert_eq!(
            state
                .component(first)
                .expect("first")
                .content_hash()
                .as_str(),
            "sha256:updated"
        );
        assert_eq!(
            state
                .components()
                .map(|component| component.component_id())
                .collect::<Vec<_>>(),
            {
                let mut expected = vec![first, second];
                expected.sort_by_key(|id| id.to_string());
                expected
            }
        );
        assert_eq!(state.relationships().collect::<Vec<_>>().len(), 1);
    }

    #[test]
    fn rejects_non_replayable_legacy_add_and_keeps_state_unchanged() {
        let context_id = ContextId::new();
        let root = commit(
            context_id,
            Vec::new(),
            vec![ContextChange::created_context("root")],
        );
        let invalid = commit(
            context_id,
            vec![root.id()],
            vec![ContextChange::added_component(
                ComponentId::new(),
                ContextComponentKind::Prompt,
                "legacy",
            )],
        );
        let mut state = ReplayState::new(context_id);
        state.apply_commit(&root).expect("root replay");
        let before = state.clone();

        assert!(matches!(
            state.apply_commit(&invalid),
            Err(ReplayError::MissingPayload {
                change: "added_component"
            })
        ));
        assert_eq!(state, before);
    }

    #[test]
    fn rejects_cross_context_and_stale_parent() {
        let context_id = ContextId::new();
        let root = commit(
            context_id,
            Vec::new(),
            vec![ContextChange::created_context("root")],
        );
        let foreign = commit(
            ContextId::new(),
            Vec::new(),
            vec![ContextChange::created_context("foreign")],
        );
        let stale = commit(context_id, vec![CommitId::new()], Vec::new());
        let mut state = ReplayState::new(context_id);

        assert!(matches!(
            state.apply_commit(&foreign),
            Err(ReplayError::CrossContext { .. })
        ));
        state.apply_commit(&root).expect("root replay");
        assert!(matches!(
            state.apply_commit(&stale),
            Err(ReplayError::ParentMismatch { .. })
        ));
    }

    #[test]
    fn removal_cleans_incident_relationships() {
        let context_id = ContextId::new();
        let first = ComponentId::new();
        let second = ComponentId::new();
        let root = commit(
            context_id,
            Vec::new(),
            vec![
                ContextChange::created_context("root"),
                detailed_add(first, "sha256:first"),
                detailed_add(second, "sha256:second"),
            ],
        );
        let relation = commit(
            context_id,
            vec![root.id()],
            vec![ContextChange::added_uses_relationship(first, second, "wire").expect("relation")],
        );
        let removal = commit(
            context_id,
            vec![relation.id()],
            vec![ContextChange::removed_component(
                second,
                ContextComponentKind::Prompt,
                ContentHash::new("sha256:second").expect("hash"),
                "remove",
            )],
        );
        let mut state = ReplayState::new(context_id);
        state.apply_commit(&root).expect("root replay");
        state.apply_commit(&relation).expect("relation replay");
        state.apply_commit(&removal).expect("removal replay");

        assert!(state.component(second).is_none());
        assert_eq!(state.relationships().count(), 0);
    }

    #[test]
    fn parent_shape_rejects_merge_after_linear_history() {
        let context_id = ContextId::new();
        let root = commit(
            context_id,
            Vec::new(),
            vec![ContextChange::created_context("root")],
        );
        let merge = commit(context_id, vec![root.id(), CommitId::new()], Vec::new());
        let mut state = ReplayState::new(context_id);
        state.apply_commit(&root).expect("root replay");

        assert_eq!(
            state.apply_commit(&merge),
            Err(ReplayError::UnsupportedParentShape)
        );
    }

    #[test]
    fn descriptor_updates_preserve_body_hash() {
        let context_id = ContextId::new();
        let id = ComponentId::new();
        let root = commit(
            context_id,
            Vec::new(),
            vec![
                ContextChange::created_context("root"),
                detailed_add(id, "sha256:body"),
            ],
        );
        let update = commit(
            context_id,
            vec![root.id()],
            vec![
                ContextChange::updated_component_descriptor(
                    id,
                    ContextComponentKind::Prompt,
                    "Renamed",
                    json!({"priority": 2}),
                    "rename",
                )
                .expect("descriptor"),
            ],
        );
        let mut state = ReplayState::new(context_id);
        state.apply_commit(&root).expect("root replay");
        state.apply_commit(&update).expect("descriptor replay");

        let component = state.component(id).expect("component");
        assert_eq!(component.name().as_str(), "Renamed");
        assert_eq!(component.metadata(), &json!({"priority": 2}));
        assert_eq!(component.content_hash().as_str(), "sha256:body");
    }

    #[test]
    fn replays_context_metadata_and_keeps_failed_commit_atomic() {
        let context_id = ContextId::new();
        let root = commit(
            context_id,
            Vec::new(),
            vec![ContextChange::created_context("root")],
        );
        let metadata = contextlab_context_core::ContextMetadata::new(Utc::now());
        let update = commit(
            context_id,
            vec![root.id()],
            vec![ContextChange::updated_metadata(
                metadata.clone(),
                "metadata",
            )],
        );
        let invalid = commit(
            context_id,
            vec![update.id()],
            vec![
                ContextChange::updated_metadata(
                    contextlab_context_core::ContextMetadata::new(Utc::now()),
                    "next",
                ),
                ContextChange::added_component(
                    ComponentId::new(),
                    ContextComponentKind::Prompt,
                    "missing replay details",
                ),
            ],
        );
        let mut state = ReplayState::new(context_id);
        state.apply_commit(&root).expect("root");
        state.apply_commit(&update).expect("metadata update");
        assert_eq!(
            state.context_metadata().expect("metadata").metadata(),
            &metadata
        );
        let before = state.clone();

        assert!(matches!(
            state.apply_commit(&invalid),
            Err(ReplayError::MissingPayload {
                change: "added_component"
            })
        ));
        assert_eq!(state, before);
    }
}
