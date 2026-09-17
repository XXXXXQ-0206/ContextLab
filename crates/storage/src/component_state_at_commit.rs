//! Private replayable component state at a Context commit.

use crate::{ComponentContentRevision, StorageRepositoryError};
use async_trait::async_trait;
use contextlab_context_core::{
    ComponentId, ContentHash, ContextComponent, ContextComponentKind, ContextId,
    DomainValidationError,
};
use contextlab_versioning::{CommitId, ContextChange, ContextChangeKind};
use serde_json::Value;
use std::collections::BTreeMap;

/// One component's replayable state at a target Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentStateAtCommit {
    context_id: ContextId,
    target_commit_id: CommitId,
    component: ContextComponent,
    metadata: Value,
    creation_commit_id: CommitId,
    content_commit_id: CommitId,
}

impl ComponentStateAtCommit {
    #[must_use]
    pub(crate) const fn new(
        context_id: ContextId,
        target_commit_id: CommitId,
        component: ContextComponent,
        metadata: Value,
        creation_commit_id: CommitId,
        content_commit_id: CommitId,
    ) -> Self {
        Self {
            context_id,
            target_commit_id,
            component,
            metadata,
            creation_commit_id,
            content_commit_id,
        }
    }

    /// Returns the Context that owns the replayed state.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the commit at which the state was requested.
    #[must_use]
    pub const fn target_commit_id(&self) -> CommitId {
        self.target_commit_id
    }

    /// Returns the replayed component descriptor and effective content hash.
    #[must_use]
    pub const fn component(&self) -> &ContextComponent {
        &self.component
    }

    /// Returns the metadata recorded by the replayable component creation.
    #[must_use]
    pub const fn metadata(&self) -> &Value {
        &self.metadata
    }

    /// Returns the commit that created the component descriptor.
    #[must_use]
    pub const fn creation_commit_id(&self) -> CommitId {
        self.creation_commit_id
    }

    /// Returns the commit that last changed the effective content hash.
    #[must_use]
    pub const fn content_commit_id(&self) -> CommitId {
        self.content_commit_id
    }
}

/// The complete descriptor-only component inventory replayed at one Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextComponentStateSnapshotAtCommit {
    context_id: ContextId,
    target_commit_id: CommitId,
    components: Vec<ComponentStateAtCommit>,
}

impl ContextComponentStateSnapshotAtCommit {
    #[must_use]
    pub(crate) const fn new(
        context_id: ContextId,
        target_commit_id: CommitId,
        components: Vec<ComponentStateAtCommit>,
    ) -> Self {
        Self {
            context_id,
            target_commit_id,
            components,
        }
    }

    /// Returns the Context that owns the replayed component inventory.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the commit at which the inventory was requested.
    #[must_use]
    pub const fn target_commit_id(&self) -> CommitId {
        self.target_commit_id
    }

    /// Returns component states ordered by their stable component identifier.
    #[must_use]
    pub fn components(&self) -> &[ComponentStateAtCommit] {
        &self.components
    }
}

/// Private read access to replayable component state at a commit.
#[async_trait]
pub trait ComponentStateAtCommitRepository: Send + Sync {
    /// Reconstructs a component state from a target commit's normal first-parent ancestry.
    async fn get_component_state_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
        component_id: ComponentId,
    ) -> Result<Option<ComponentStateAtCommit>, StorageRepositoryError>;
}

/// Private read access to a replayable Context component inventory at a commit.
#[async_trait]
pub trait ContextComponentStateSnapshotAtCommitRepository: Send + Sync {
    /// Reconstructs all replayable component descriptors from a target commit's normal first-parent ancestry.
    async fn get_context_component_state_snapshot_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<ContextComponentStateSnapshotAtCommit, StorageRepositoryError>;
}

/// One validated ancestry commit supplied to the private state reducer.
#[derive(Debug, Clone)]
pub(crate) struct ComponentStateReplayStep {
    commit_id: CommitId,
    changes: Vec<ContextChange>,
    revisions: BTreeMap<String, ComponentContentRevision>,
}

impl ComponentStateReplayStep {
    #[must_use]
    pub(crate) fn new(
        commit_id: CommitId,
        changes: Vec<ContextChange>,
        revisions: impl IntoIterator<Item = ComponentContentRevision>,
    ) -> Self {
        Self {
            commit_id,
            changes,
            revisions: revisions
                .into_iter()
                .map(|revision| (revision.component_id().to_string(), revision))
                .collect(),
        }
    }

    fn revision(&self, component_id: ComponentId) -> Option<&ComponentContentRevision> {
        self.revisions.get(&component_id.to_string())
    }
}

pub(crate) fn replay_component_state(
    context_id: ContextId,
    target_commit_id: CommitId,
    component_id: ComponentId,
    steps_from_root: impl IntoIterator<Item = ComponentStateReplayStep>,
) -> Result<Option<ComponentStateAtCommit>, StorageRepositoryError> {
    let mut state = None;

    for step in steps_from_root {
        for change in step
            .changes
            .iter()
            .filter(|change| change.component_id() == Some(component_id))
        {
            state = replay_component_state_transition(
                context_id,
                target_commit_id,
                component_id,
                state,
                change,
                &step,
            )?;
        }
    }

    Ok(state)
}

pub(crate) fn replay_context_component_state_snapshot(
    context_id: ContextId,
    target_commit_id: CommitId,
    steps_from_root: impl IntoIterator<Item = ComponentStateReplayStep>,
) -> Result<ContextComponentStateSnapshotAtCommit, StorageRepositoryError> {
    let mut states = BTreeMap::new();

    for step in steps_from_root {
        for change in &step.changes {
            let Some(component_id) = change.component_id() else {
                continue;
            };
            let key = component_id.to_string();
            let current_state = states.remove(&key);
            let next_state = replay_component_state_transition(
                context_id,
                target_commit_id,
                component_id,
                current_state,
                change,
                &step,
            )?;
            if let Some(next_state) = next_state {
                states.insert(key, next_state);
            }
        }
    }

    Ok(ContextComponentStateSnapshotAtCommit::new(
        context_id,
        target_commit_id,
        states.into_values().collect(),
    ))
}

fn replay_component_state_transition(
    context_id: ContextId,
    target_commit_id: CommitId,
    component_id: ComponentId,
    state: Option<ComponentStateAtCommit>,
    change: &ContextChange,
    step: &ComponentStateReplayStep,
) -> Result<Option<ComponentStateAtCommit>, StorageRepositoryError> {
    match change.kind() {
        ContextChangeKind::AddedComponent => {
            if state.is_some() {
                return Err(component_state_conflict(
                    "component state replay encountered a duplicate component creation",
                ));
            }
            let component_kind = required_component_kind(change)?;
            let component_name = change.component_name().ok_or_else(|| {
                component_state_conflict(
                    "component state replay requires a detailed component creation name",
                )
            })?;
            let metadata = change.component_metadata().cloned().ok_or_else(|| {
                component_state_conflict(
                    "component state replay requires detailed component creation metadata",
                )
            })?;
            let resulting_content_hash = required_resulting_content_hash(change)?;
            let revision = matching_revision(
                step.revision(component_id),
                component_id,
                component_kind,
                None,
                resulting_content_hash,
            )?;
            let component = ContextComponent::with_id(
                component_id,
                component_kind,
                component_name.as_str(),
                revision.resulting_content_hash().as_str(),
            )
            .map_err(invalid_component_state)?;

            Ok(Some(ComponentStateAtCommit::new(
                context_id,
                target_commit_id,
                component,
                metadata,
                step.commit_id,
                step.commit_id,
            )))
        }
        ContextChangeKind::UpdatedComponent => {
            let state = state.ok_or_else(|| {
                component_state_conflict(
                    "component state replay encountered an update before replayable creation",
                )
            })?;
            let component_kind = required_component_kind(change)?;
            if state.component.kind() != component_kind {
                return Err(component_state_conflict(
                    "component state replay encountered a component kind transition",
                ));
            }
            let previous_content_hash = change.previous_content_hash().ok_or_else(|| {
                component_state_conflict(
                    "component state replay requires an update previous content hash",
                )
            })?;
            if state.component.content_hash() != previous_content_hash {
                return Err(component_state_conflict(
                    "component state replay encountered a discontinuous content hash",
                ));
            }
            let resulting_content_hash = required_resulting_content_hash(change)?;
            let revision = matching_revision(
                step.revision(component_id),
                component_id,
                component_kind,
                Some(previous_content_hash),
                resulting_content_hash,
            )?;
            let component = ContextComponent::with_id(
                component_id,
                component_kind,
                state.component.name().as_str(),
                revision.resulting_content_hash().as_str(),
            )
            .map_err(invalid_component_state)?;

            Ok(Some(ComponentStateAtCommit::new(
                context_id,
                target_commit_id,
                component,
                state.metadata,
                state.creation_commit_id,
                step.commit_id,
            )))
        }
        ContextChangeKind::UpdatedComponentDescriptor => {
            let state = state.ok_or_else(|| {
                component_state_conflict(
                    "component state replay encountered a descriptor update before replayable creation",
                )
            })?;
            let component_kind = required_component_kind(change)?;
            if state.component.kind() != component_kind {
                return Err(component_state_conflict(
                    "component state replay encountered a component kind transition",
                ));
            }
            let component_name = change.component_name().ok_or_else(|| {
                component_state_conflict("component state replay requires a descriptor update name")
            })?;
            let metadata = change.component_metadata().cloned().ok_or_else(|| {
                component_state_conflict(
                    "component state replay requires descriptor update metadata",
                )
            })?;
            if change.previous_content_hash().is_some() || change.resulting_content_hash().is_some()
            {
                return Err(component_state_conflict(
                    "component state replay encountered descriptor body hashes",
                ));
            }
            let component = ContextComponent::with_id(
                component_id,
                component_kind,
                component_name.as_str(),
                state.component.content_hash().as_str(),
            )
            .map_err(invalid_component_state)?;

            Ok(Some(ComponentStateAtCommit::new(
                context_id,
                target_commit_id,
                component,
                metadata,
                state.creation_commit_id,
                state.content_commit_id,
            )))
        }
        ContextChangeKind::RemovedComponent => {
            let state = state.ok_or_else(|| {
                component_state_conflict(
                    "component state replay encountered a removal before replayable creation",
                )
            })?;
            let component_kind = required_component_kind(change)?;
            if state.component.kind() != component_kind {
                return Err(component_state_conflict(
                    "component state replay encountered a component kind transition",
                ));
            }
            let previous_content_hash = change.previous_content_hash().ok_or_else(|| {
                component_state_conflict(
                    "component state replay requires a removal previous content hash",
                )
            })?;
            if state.component.content_hash() != previous_content_hash {
                return Err(component_state_conflict(
                    "component state replay encountered a discontinuous removal content hash",
                ));
            }
            if change.resulting_content_hash().is_some()
                || change.component_name().is_some()
                || change.component_metadata().is_some()
            {
                return Err(component_state_conflict(
                    "component state replay encountered an invalid removal payload",
                ));
            }

            Ok(None)
        }
        ContextChangeKind::AddedUsesRelationship
        | ContextChangeKind::RemovedUsesRelationship
        | ContextChangeKind::CreatedContext
        | ContextChangeKind::UpdatedMetadata => Ok(state),
    }
}

fn matching_revision<'a>(
    revision: Option<&'a ComponentContentRevision>,
    component_id: ComponentId,
    component_kind: ContextComponentKind,
    previous_content_hash: Option<&ContentHash>,
    resulting_content_hash: &ContentHash,
) -> Result<&'a ComponentContentRevision, StorageRepositoryError> {
    let revision = revision.ok_or_else(|| {
        component_state_conflict(
            "component state replay requires an immutable revision for every component transition",
        )
    })?;
    if revision.component_id() != component_id
        || revision.component_kind() != component_kind
        || revision.previous_content_hash() != previous_content_hash
        || revision.resulting_content_hash() != resulting_content_hash
    {
        return Err(component_state_conflict(
            "component state replay transition does not match its immutable revision",
        ));
    }

    Ok(revision)
}

fn required_component_kind(
    change: &ContextChange,
) -> Result<ContextComponentKind, StorageRepositoryError> {
    change
        .component_kind()
        .ok_or_else(|| component_state_conflict("component state replay requires a component kind"))
}

fn required_resulting_content_hash(
    change: &ContextChange,
) -> Result<&ContentHash, StorageRepositoryError> {
    change.resulting_content_hash().ok_or_else(|| {
        component_state_conflict("component state replay requires a resulting content hash")
    })
}

fn invalid_component_state(_: DomainValidationError) -> StorageRepositoryError {
    component_state_conflict("component state replay contains an invalid component descriptor")
}

pub(crate) fn component_state_conflict(reason: impl Into<String>) -> StorageRepositoryError {
    StorageRepositoryError::ComponentStateReplayConflict {
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentStateReplayStep, replay_context_component_state_snapshot};
    use crate::{
        ComponentContentRevision, StorageRepositoryError,
        component_content_revision::PersistedComponentContentRevision,
    };
    use chrono::{TimeZone, Utc};
    use contextlab_context_core::{ComponentContent, ComponentId, ContextComponentKind, ContextId};
    use contextlab_versioning::{CommitId, ContextChange};

    #[test]
    fn replays_a_deterministically_ordered_context_component_inventory() {
        let context_id = ContextId::new();
        let first_component_id = ComponentId::new();
        let second_component_id = ComponentId::new();
        let root_commit_id = CommitId::new();
        let revised_commit_id = CommitId::new();
        let created_at = Utc
            .timestamp_opt(1_740_000_000, 0)
            .single()
            .expect("fixed timestamp");
        let initial_first_content = ComponentContent::new("first initial body");
        let revised_first_content = ComponentContent::new("first revised body");
        let second_content = ComponentContent::new("second initial body");

        let snapshot = replay_context_component_state_snapshot(
            context_id,
            revised_commit_id,
            [
                ComponentStateReplayStep::new(
                    root_commit_id,
                    vec![
                        ContextChange::added_component_content_with_details(
                            first_component_id,
                            ContextComponentKind::Prompt,
                            "First prompt",
                            serde_json::json!({ "order": 1 }),
                            initial_first_content.content_hash(),
                            "Create first prompt",
                        )
                        .expect("first creation change"),
                        ContextChange::added_component_content_with_details(
                            second_component_id,
                            ContextComponentKind::Knowledge,
                            "Second knowledge",
                            serde_json::json!({ "order": 2 }),
                            second_content.content_hash(),
                            "Create second knowledge",
                        )
                        .expect("second creation change"),
                    ],
                    vec![
                        ComponentContentRevision::from_persisted(
                            PersistedComponentContentRevision {
                                context_id,
                                commit_id: root_commit_id,
                                component_id: first_component_id,
                                component_kind: ContextComponentKind::Prompt,
                                previous_content_hash: None,
                                content: initial_first_content.clone(),
                                resulting_content_hash: initial_first_content.content_hash(),
                                captured_at: created_at,
                            },
                        ),
                        ComponentContentRevision::from_persisted(
                            PersistedComponentContentRevision {
                                context_id,
                                commit_id: root_commit_id,
                                component_id: second_component_id,
                                component_kind: ContextComponentKind::Knowledge,
                                previous_content_hash: None,
                                content: second_content.clone(),
                                resulting_content_hash: second_content.content_hash(),
                                captured_at: created_at,
                            },
                        ),
                    ],
                ),
                ComponentStateReplayStep::new(
                    revised_commit_id,
                    vec![ContextChange::updated_component_content(
                        first_component_id,
                        ContextComponentKind::Prompt,
                        initial_first_content.content_hash(),
                        revised_first_content.content_hash(),
                        "Revise first prompt",
                    )],
                    vec![ComponentContentRevision::from_persisted(
                        PersistedComponentContentRevision {
                            context_id,
                            commit_id: revised_commit_id,
                            component_id: first_component_id,
                            component_kind: ContextComponentKind::Prompt,
                            previous_content_hash: Some(initial_first_content.content_hash()),
                            content: revised_first_content.clone(),
                            resulting_content_hash: revised_first_content.content_hash(),
                            captured_at: created_at,
                        },
                    )],
                ),
            ],
        )
        .expect("replay Context component inventory");

        let component_ids = snapshot
            .components()
            .iter()
            .map(|state| state.component().id().to_string())
            .collect::<Vec<_>>();
        let mut sorted_component_ids = component_ids.clone();
        sorted_component_ids.sort();
        assert_eq!(component_ids, sorted_component_ids);
        assert_eq!(snapshot.components().len(), 2);

        let first_state = snapshot
            .components()
            .iter()
            .find(|state| state.component().id() == first_component_id)
            .expect("first component state");
        let second_state = snapshot
            .components()
            .iter()
            .find(|state| state.component().id() == second_component_id)
            .expect("second component state");
        assert_eq!(
            first_state.component().content_hash(),
            &revised_first_content.content_hash()
        );
        assert_eq!(first_state.creation_commit_id(), root_commit_id);
        assert_eq!(first_state.content_commit_id(), revised_commit_id);
        assert_eq!(
            second_state.component().content_hash(),
            &second_content.content_hash()
        );
        assert_eq!(second_state.creation_commit_id(), root_commit_id);
        assert_eq!(second_state.content_commit_id(), root_commit_id);
    }

    #[test]
    fn removes_a_component_from_a_later_context_inventory() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let root_commit_id = CommitId::new();
        let removal_commit_id = CommitId::new();
        let captured_at = Utc
            .timestamp_opt(1_740_000_000, 0)
            .single()
            .expect("fixed timestamp");
        let content = ComponentContent::new("removable prompt body");

        let snapshot = replay_context_component_state_snapshot(
            context_id,
            removal_commit_id,
            [
                ComponentStateReplayStep::new(
                    root_commit_id,
                    vec![
                        ContextChange::added_component_content_with_details(
                            component_id,
                            ContextComponentKind::Prompt,
                            "Removable prompt",
                            serde_json::json!({}),
                            content.content_hash(),
                            "Create removable prompt",
                        )
                        .expect("creation change"),
                    ],
                    vec![ComponentContentRevision::from_persisted(
                        PersistedComponentContentRevision {
                            context_id,
                            commit_id: root_commit_id,
                            component_id,
                            component_kind: ContextComponentKind::Prompt,
                            previous_content_hash: None,
                            content: content.clone(),
                            resulting_content_hash: content.content_hash(),
                            captured_at,
                        },
                    )],
                ),
                ComponentStateReplayStep::new(
                    removal_commit_id,
                    vec![ContextChange::removed_component(
                        component_id,
                        ContextComponentKind::Prompt,
                        content.content_hash(),
                        "Remove prompt",
                    )],
                    Vec::new(),
                ),
            ],
        )
        .expect("replay removal");

        assert!(snapshot.components().is_empty());
    }

    #[test]
    fn rejects_an_update_after_component_removal() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let root_commit_id = CommitId::new();
        let removal_commit_id = CommitId::new();
        let update_commit_id = CommitId::new();
        let captured_at = Utc
            .timestamp_opt(1_740_000_000, 0)
            .single()
            .expect("fixed timestamp");
        let content = ComponentContent::new("removed prompt body");
        let later_content = ComponentContent::new("invalid later body");

        let error = replay_context_component_state_snapshot(
            context_id,
            update_commit_id,
            [
                ComponentStateReplayStep::new(
                    root_commit_id,
                    vec![
                        ContextChange::added_component_content_with_details(
                            component_id,
                            ContextComponentKind::Prompt,
                            "Removed prompt",
                            serde_json::json!({}),
                            content.content_hash(),
                            "Create prompt",
                        )
                        .expect("creation change"),
                    ],
                    vec![ComponentContentRevision::from_persisted(
                        PersistedComponentContentRevision {
                            context_id,
                            commit_id: root_commit_id,
                            component_id,
                            component_kind: ContextComponentKind::Prompt,
                            previous_content_hash: None,
                            content: content.clone(),
                            resulting_content_hash: content.content_hash(),
                            captured_at,
                        },
                    )],
                ),
                ComponentStateReplayStep::new(
                    removal_commit_id,
                    vec![ContextChange::removed_component(
                        component_id,
                        ContextComponentKind::Prompt,
                        content.content_hash(),
                        "Remove prompt",
                    )],
                    Vec::new(),
                ),
                ComponentStateReplayStep::new(
                    update_commit_id,
                    vec![ContextChange::updated_component_content(
                        component_id,
                        ContextComponentKind::Prompt,
                        content.content_hash(),
                        later_content.content_hash(),
                        "Invalid post-removal update",
                    )],
                    Vec::new(),
                ),
            ],
        )
        .expect_err("updates after removal must fail closed");

        assert!(matches!(
            error,
            StorageRepositoryError::ComponentStateReplayConflict { .. }
        ));
    }
}
