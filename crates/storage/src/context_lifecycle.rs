//! Private Context lifecycle application contracts.

use crate::{
    CommitGraphSnapshot, CommitGraphSnapshotRepository, CommitSnapshotWriteError,
    ComponentContentCreationWrite, ComponentContentRevision, ComponentContentRevisionRepository,
    ComponentContentRevisionWrite, ComponentDescriptorRevisionWrite, ComponentRemovalWrite,
    ComponentStateAtCommit, ContextComponentStateSnapshotAtCommit,
    ContextComponentStateSnapshotAtCommitRepository, ContextReplayStateAtCommitRepository,
    CreateContextCommitSnapshot, GuardedCommitWriteDisposition, GuardedCommitWriteError,
    GuardedContextCommitWrite, GuardedContextCommitWriter, IdempotencyKey, RequestDigest,
    StorageRepositoryError, StoredComponentKind,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_auth::AuthenticatedPrincipal;
use contextlab_context_core::{
    ComponentContent, ComponentId, ContextComponent, ContextComponentKind, ContextId,
    ContextMetadata, DomainValidationError, ProjectId,
};
use contextlab_graph::{
    ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind, GraphValidationError,
};
use contextlab_versioning::{
    BranchName, CommitId, ContextChange, ContextCommit, ExpectedBranchHead, ReplayState,
    VersioningError,
};
use serde_json::Value;
use thiserror::Error;

/// A high-level lifecycle operation that is translated into one guarded Context commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextLifecycleOperation {
    /// Creates the initial parentless commit and graph snapshot for an unborn Context branch.
    Initialize,
    /// Replaces the typed metadata of an existing Context.
    UpdateMetadata {
        /// The replacement Context metadata.
        metadata: ContextMetadata,
    },
    /// Creates a component and its immutable initial body revision.
    Create {
        /// The shared Context component taxonomy for the new component.
        component_kind: ContextComponentKind,
        /// The replayable component display name.
        name: String,
        /// Flexible component metadata preserved with the creation transition.
        metadata: Value,
        /// The immutable initial component body.
        content: ComponentContent,
    },
    /// Updates the immutable body revision of an existing component.
    Update {
        /// The existing component to revise.
        component_id: ComponentId,
        /// The immutable replacement body.
        content: ComponentContent,
    },
    /// Revises an existing component display name and metadata without replacing its body.
    UpdateDescriptor {
        /// The existing component to revise.
        component_id: ComponentId,
        /// The replacement replayable display name.
        name: String,
        /// The replacement flexible metadata.
        metadata: Value,
    },
    /// Removes an existing component from later Context versions.
    Remove {
        /// The existing component to remove.
        component_id: ComponentId,
    },
    /// Adds one directed `Uses` relationship between two active components.
    AddUsesRelationship {
        /// The component that uses the target component.
        source_component_id: ComponentId,
        /// The component used by the source component.
        target_component_id: ComponentId,
    },
    /// Removes one directed `Uses` relationship between two active components.
    RemoveUsesRelationship {
        /// The component that uses the target component.
        source_component_id: ComponentId,
        /// The component used by the source component.
        target_component_id: ComponentId,
    },
}

/// A framework-independent request for one local Context lifecycle transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLifecycleCommand {
    principal: AuthenticatedPrincipal,
    context_id: ContextId,
    branch: BranchName,
    expected_head: ExpectedBranchHead,
    idempotency_key: IdempotencyKey,
    request_digest: RequestDigest,
    message: String,
    operation: ContextLifecycleOperation,
    captured_at: DateTime<Utc>,
}

impl ContextLifecycleCommand {
    /// Builds a local lifecycle command from an already validated typed operation.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn from_operation(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        operation: ContextLifecycleOperation,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            principal,
            context_id,
            branch,
            expected_head: ExpectedBranchHead::Commit(expected_head),
            idempotency_key,
            request_digest,
            message: message.into(),
            operation,
            captured_at,
        }
    }

    /// Builds a local Context-root command for an unborn branch.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn initialize(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            principal,
            context_id,
            branch,
            expected_head: ExpectedBranchHead::Unborn,
            idempotency_key,
            request_digest,
            message: message.into(),
            operation: ContextLifecycleOperation::Initialize,
            captured_at,
        }
    }

    /// Builds a local component-creation command from an existing materialized branch head.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn create(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        component_kind: ContextComponentKind,
        name: impl Into<String>,
        metadata: Value,
        content: ComponentContent,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            principal,
            context_id,
            branch,
            expected_head: ExpectedBranchHead::Commit(expected_head),
            idempotency_key,
            request_digest,
            message: message.into(),
            operation: ContextLifecycleOperation::Create {
                component_kind,
                name: name.into(),
                metadata,
                content,
            },
            captured_at,
        }
    }

    /// Builds a typed Context metadata update from an existing materialized branch head.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn update_metadata(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        metadata: ContextMetadata,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self::from_operation(
            principal,
            context_id,
            branch,
            expected_head,
            idempotency_key,
            request_digest,
            message,
            ContextLifecycleOperation::UpdateMetadata { metadata },
            captured_at,
        )
    }

    /// Builds a local component-content update command from an existing materialized branch head.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn update(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        component_id: ComponentId,
        content: ComponentContent,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            principal,
            context_id,
            branch,
            expected_head: ExpectedBranchHead::Commit(expected_head),
            idempotency_key,
            request_digest,
            message: message.into(),
            operation: ContextLifecycleOperation::Update {
                component_id,
                content,
            },
            captured_at,
        }
    }

    /// Builds a local component descriptor update from an existing materialized branch head.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn update_descriptor(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        component_id: ComponentId,
        name: impl Into<String>,
        metadata: Value,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            principal,
            context_id,
            branch,
            expected_head: ExpectedBranchHead::Commit(expected_head),
            idempotency_key,
            request_digest,
            message: message.into(),
            operation: ContextLifecycleOperation::UpdateDescriptor {
                component_id,
                name: name.into(),
                metadata,
            },
            captured_at,
        }
    }

    /// Builds a local component-removal command from an existing materialized branch head.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn remove(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        component_id: ComponentId,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self {
            principal,
            context_id,
            branch,
            expected_head: ExpectedBranchHead::Commit(expected_head),
            idempotency_key,
            request_digest,
            message: message.into(),
            operation: ContextLifecycleOperation::Remove { component_id },
            captured_at,
        }
    }

    /// Builds a local component `Uses` relationship-add command.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn add_uses_relationship(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        source_component_id: ComponentId,
        target_component_id: ComponentId,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self::from_operation(
            principal,
            context_id,
            branch,
            expected_head,
            idempotency_key,
            request_digest,
            message,
            ContextLifecycleOperation::AddUsesRelationship {
                source_component_id,
                target_component_id,
            },
            captured_at,
        )
    }

    /// Builds a local component `Uses` relationship-remove command.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn remove_uses_relationship(
        principal: AuthenticatedPrincipal,
        context_id: ContextId,
        branch: BranchName,
        expected_head: CommitId,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        message: impl Into<String>,
        source_component_id: ComponentId,
        target_component_id: ComponentId,
        captured_at: DateTime<Utc>,
    ) -> Self {
        Self::from_operation(
            principal,
            context_id,
            branch,
            expected_head,
            idempotency_key,
            request_digest,
            message,
            ContextLifecycleOperation::RemoveUsesRelationship {
                source_component_id,
                target_component_id,
            },
            captured_at,
        )
    }
}

/// The outcome of one guarded local lifecycle transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLifecycleWriteResult {
    commit_id: CommitId,
    disposition: GuardedCommitWriteDisposition,
    snapshot: CommitGraphSnapshot,
}

impl ContextLifecycleWriteResult {
    /// Returns the commit that records the created or replayed transition.
    #[must_use]
    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    /// Returns whether this call created or replayed the guarded write.
    #[must_use]
    pub const fn disposition(&self) -> GuardedCommitWriteDisposition {
        self.disposition
    }

    /// Returns the graph snapshot atomically associated with the transition.
    #[must_use]
    pub const fn snapshot(&self) -> &CommitGraphSnapshot {
        &self.snapshot
    }
}

/// One replayed component descriptor paired with its immutable effective body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLifecycleComponentState {
    state: ComponentStateAtCommit,
    content: ComponentContentRevision,
}

impl ContextLifecycleComponentState {
    /// Returns the replayed component descriptor state.
    #[must_use]
    pub const fn state(&self) -> &ComponentStateAtCommit {
        &self.state
    }

    /// Returns the immutable effective body revision.
    #[must_use]
    pub const fn content(&self) -> &ComponentContentRevision {
        &self.content
    }
}

/// The complete local Context state at one materialized commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLifecycleStateAtCommit {
    scope: crate::CommitGraphSnapshotScope,
    components: Vec<ContextLifecycleComponentState>,
    metadata: Option<ContextMetadata>,
    graph_snapshot: CommitGraphSnapshot,
    replay_state: ReplayState,
}

impl ContextLifecycleStateAtCommit {
    /// Returns the exact project/Context/commit scope represented by this state.
    #[must_use]
    pub const fn scope(&self) -> crate::CommitGraphSnapshotScope {
        self.scope
    }

    /// Returns replayed components in stable component-identifier order.
    #[must_use]
    pub fn components(&self) -> &[ContextLifecycleComponentState] {
        &self.components
    }

    /// Returns the typed Context metadata reconstructed at the requested commit.
    #[must_use]
    pub const fn metadata(&self) -> Option<&ContextMetadata> {
        self.metadata.as_ref()
    }

    /// Returns the immutable graph snapshot at the requested commit.
    #[must_use]
    pub const fn graph_snapshot(&self) -> &CommitGraphSnapshot {
        &self.graph_snapshot
    }

    /// Returns the replay state used to derive metadata and component facts.
    #[must_use]
    pub const fn replay_state(&self) -> &ReplayState {
        &self.replay_state
    }
}

/// Facts collected by one repository boundary for one exact Context commit.
///
/// The aggregate keeps the project binding alongside the replay, component,
/// content, and graph facts. Leaf repositories remain available for writes and
/// narrower reads, but callers must use this port when they need one coherent
/// lifecycle view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLifecycleReadFacts {
    scope: crate::CommitGraphSnapshotScope,
    inventory: ContextComponentStateSnapshotAtCommit,
    contents: Vec<ComponentContentRevision>,
    replay_state: ReplayState,
    graph_snapshot: CommitGraphSnapshot,
}

impl ContextLifecycleReadFacts {
    pub(crate) fn from_parts(
        scope: crate::CommitGraphSnapshotScope,
        inventory: ContextComponentStateSnapshotAtCommit,
        contents: Vec<ComponentContentRevision>,
        replay_state: ReplayState,
        graph_snapshot: CommitGraphSnapshot,
    ) -> Result<Self, StorageRepositoryError> {
        if graph_snapshot.scope() != scope {
            return Err(StorageRepositoryError::InvalidScope {
                scope: scope.to_string(),
                reason: format!("stored graph snapshot scope is {}", graph_snapshot.scope()),
            });
        }
        if inventory.context_id() != scope.context_id()
            || inventory.target_commit_id() != scope.commit_id()
        {
            return Err(StorageRepositoryError::InvalidScope {
                scope: scope.to_string(),
                reason: "component inventory scope does not match the requested commit".to_owned(),
            });
        }
        if replay_state.context_id() != scope.context_id()
            || replay_state.commit_id() != Some(scope.commit_id())
        {
            return Err(StorageRepositoryError::InvalidScope {
                scope: scope.to_string(),
                reason: "replay state scope does not match the requested commit".to_owned(),
            });
        }
        if contents
            .iter()
            .any(|content| content.context_id() != scope.context_id())
        {
            return Err(StorageRepositoryError::InvalidScope {
                scope: scope.to_string(),
                reason: "component content scope does not match the requested Context".to_owned(),
            });
        }

        Ok(Self {
            scope,
            inventory,
            contents,
            replay_state,
            graph_snapshot,
        })
    }

    pub(crate) const fn scope(&self) -> crate::CommitGraphSnapshotScope {
        self.scope
    }

    pub(crate) const fn inventory(&self) -> &ContextComponentStateSnapshotAtCommit {
        &self.inventory
    }

    pub(crate) fn contents(&self) -> &[ComponentContentRevision] {
        &self.contents
    }

    /// Validates that all exact-commit lifecycle witnesses agree before a
    /// consumer uses the aggregate as a complete Context state.
    pub(crate) fn validate_consistency(&self) -> Result<(), ContextLifecycleError> {
        for state in self.inventory.components() {
            let component = state.component();
            let content = self
                .contents
                .iter()
                .find(|content| {
                    content.context_id() == self.scope.context_id()
                        && content.component_id() == component.id()
                        && content.commit_id() == state.content_commit_id()
                })
                .ok_or(ContextLifecycleError::ComponentContentMissing {
                    component_id: component.id(),
                    commit_id: self.scope.commit_id(),
                })?;
            validate_component_content_witness(state, content)?;
            validate_component_graph(
                self.graph_snapshot.graph(),
                self.scope.context_id(),
                component,
            )?;
        }

        crate::replay_graph_consistency::validate_replay_graph_consistency(
            &self.replay_state,
            &self.graph_snapshot,
        )
        .map_err(|error| ContextLifecycleError::ReplayGraphConsistency {
            reason: error.to_string(),
        })
    }

    pub(crate) const fn replay_state(&self) -> &ReplayState {
        &self.replay_state
    }

    pub(crate) const fn graph_snapshot(&self) -> &CommitGraphSnapshot {
        &self.graph_snapshot
    }
}

/// Errors produced while composing the local lifecycle workflow from storage contracts.
#[derive(Debug, Error)]
pub enum ContextLifecycleError {
    /// Context initialization can only target an unborn branch.
    #[error("Context initialization requires an unborn branch")]
    InitializationRequiresUnbornBranch,
    /// A required materialized graph snapshot is absent for the supplied commit.
    #[error("Context lifecycle requires a materialized graph snapshot at commit {commit_id}")]
    MaterializedSnapshotMissing {
        /// The commit whose graph snapshot is required.
        commit_id: CommitId,
    },
    /// The requested component cannot be reconstructed at the supplied branch head.
    #[error("component {component_id} is unavailable at commit {commit_id}")]
    ComponentStateMissing {
        /// The component requested by the transition.
        component_id: ComponentId,
        /// The materialized branch head used for replay.
        commit_id: CommitId,
    },
    /// A descriptor has no matching immutable body witness at the supplied commit.
    #[error("component {component_id} has no immutable body witness at commit {commit_id}")]
    ComponentContentMissing {
        /// The component whose immutable body witness is absent.
        component_id: ComponentId,
        /// The target commit whose replay requires the witness.
        commit_id: CommitId,
    },
    /// Immutable descriptor and body witnesses disagree.
    #[error("component {component_id} has conflicting descriptor and body witnesses")]
    ComponentWitnessConflict {
        /// The component with conflicting durable witnesses.
        component_id: ComponentId,
    },
    /// The materialized graph is missing the component required by an update or removal.
    #[error("materialized graph is missing component node {component_id}")]
    ComponentGraphNodeMissing {
        /// The component absent from the materialized graph.
        component_id: ComponentId,
    },
    /// A materialized graph component node does not agree with replayed descriptor facts.
    #[error("materialized graph component node conflicts with descriptor {component_id}")]
    ComponentGraphNodeConflict {
        /// The component whose graph node conflicts with replayed state.
        component_id: ComponentId,
    },
    /// Exact replay state and its materialized graph snapshot disagree.
    #[error("replay state and graph snapshot conflict: {reason}")]
    ReplayGraphConsistency {
        /// Stable storage consistency failure.
        reason: String,
    },
    /// The requested `Uses` relationship already exists at the materialized head.
    #[error("Uses relationship from {source_component_id} to {target_component_id} already exists")]
    UsesRelationshipAlreadyExists {
        /// Source component identifier.
        source_component_id: ComponentId,
        /// Target component identifier.
        target_component_id: ComponentId,
    },
    /// The requested `Uses` relationship is absent at the materialized head.
    #[error("Uses relationship from {source_component_id} to {target_component_id} is absent")]
    UsesRelationshipMissing {
        /// Source component identifier.
        source_component_id: ComponentId,
        /// Target component identifier.
        target_component_id: ComponentId,
    },
    /// The persisted snapshot identifier is not a valid Context commit identifier.
    #[error("materialized graph snapshot contains an invalid commit identifier")]
    InvalidSnapshotCommitIdentifier,
    /// A reusable storage contract rejected the operation.
    #[error(transparent)]
    Storage(#[from] StorageRepositoryError),
    /// The lifecycle command failed domain validation.
    #[error(transparent)]
    Domain(#[from] DomainValidationError),
    /// The lifecycle command could not create a normal Context commit.
    #[error(transparent)]
    Versioning(#[from] VersioningError),
    /// The successor graph is invalid.
    #[error(transparent)]
    Graph(#[from] GraphValidationError),
    /// The commit snapshot could not be constructed.
    #[error(transparent)]
    Snapshot(#[from] CommitSnapshotWriteError),
    /// The guarded writer rejected the composed transition before persistence.
    #[error(transparent)]
    GuardedWrite(#[from] GuardedCommitWriteError),
}

/// Private repository bundle required by the framework-independent lifecycle service.
pub trait ContextLifecycleRepository:
    ContextLifecycleReadRepository
    + CommitGraphSnapshotRepository
    + ContextComponentStateSnapshotAtCommitRepository
    + ContextReplayStateAtCommitRepository
    + ComponentContentRevisionRepository
    + crate::ComponentStateAtCommitRepository
    + ContextLifecycleRootRepository
    + GuardedContextCommitWriter
{
}

impl<Repository> ContextLifecycleRepository for Repository where
    Repository: ContextLifecycleReadRepository
        + CommitGraphSnapshotRepository
        + ContextComponentStateSnapshotAtCommitRepository
        + ContextReplayStateAtCommitRepository
        + ComponentContentRevisionRepository
        + crate::ComponentStateAtCommitRepository
        + ContextLifecycleRootRepository
        + GuardedContextCommitWriter
        + ?Sized
{
}

/// Reads one coherent lifecycle aggregate at an exact project/Context/commit scope.
#[async_trait]
pub trait ContextLifecycleReadRepository: Send + Sync {
    /// Loads replay, component, content, metadata, and graph facts through one
    /// backend-owned read boundary.
    async fn get_context_lifecycle_read_facts(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<ContextLifecycleReadFacts, StorageRepositoryError>;
}

/// Immutable Context facts resolved by storage before a lifecycle transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextLifecycleRoot {
    project_id: ProjectId,
    name: String,
}

impl ContextLifecycleRoot {
    /// Creates the active Context facts required by lifecycle composition.
    #[must_use]
    pub fn new(project_id: ProjectId, name: impl Into<String>) -> Self {
        Self {
            project_id,
            name: name.into(),
        }
    }

    /// Returns the project that owns the Context.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the Context display name used by its initial graph node.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Private storage port for the Context facts needed by lifecycle composition.
#[async_trait]
pub trait ContextLifecycleRootRepository: Send + Sync {
    /// Returns the active Context project binding and display name.
    async fn get_context_lifecycle_root(
        &self,
        context_id: ContextId,
    ) -> Result<ContextLifecycleRoot, StorageRepositoryError>;
}

/// Composes local lifecycle requests from reusable storage contracts without framework dependencies.
pub struct ContextLifecycleService<'repository, Repository: ?Sized> {
    repository: &'repository Repository,
}

impl<'repository, Repository: ?Sized> ContextLifecycleService<'repository, Repository> {
    /// Creates a lifecycle service over one repository implementation.
    #[must_use]
    pub const fn new(repository: &'repository Repository) -> Self {
        Self { repository }
    }
}

impl<Repository: ?Sized> ContextLifecycleService<'_, Repository>
where
    Repository: ContextLifecycleReadRepository
        + CommitGraphSnapshotRepository
        + ContextComponentStateSnapshotAtCommitRepository
        + ContextReplayStateAtCommitRepository
        + ComponentContentRevisionRepository
        + crate::ComponentStateAtCommitRepository
        + ContextLifecycleRootRepository
        + GuardedContextCommitWriter
        + Sync,
{
    /// Executes one local lifecycle operation through the existing guarded writer.
    pub async fn execute(
        &self,
        command: ContextLifecycleCommand,
    ) -> Result<ContextLifecycleWriteResult, ContextLifecycleError> {
        let ContextLifecycleCommand {
            principal,
            context_id,
            branch,
            expected_head,
            idempotency_key,
            request_digest,
            message,
            operation,
            captured_at,
        } = command;
        let root = self
            .repository
            .get_context_lifecycle_root(context_id)
            .await?;
        let project_id = root.project_id();
        let (change, graph, attachment, parent_ids, metadata) = match operation {
            ContextLifecycleOperation::Initialize => {
                if expected_head != ExpectedBranchHead::Unborn {
                    return Err(ContextLifecycleError::InitializationRequiresUnbornBranch);
                }
                (
                    ContextChange::created_context(&message),
                    graph_for_initialized_context(context_id, root.name())?,
                    None,
                    Vec::new(),
                    None,
                )
            }
            operation => {
                let ExpectedBranchHead::Commit(expected_head) = expected_head else {
                    return Err(ContextLifecycleError::InitializationRequiresUnbornBranch);
                };
                let parent_snapshot = self
                    .materialized_snapshot(project_id, context_id, expected_head)
                    .await?;
                self.repository
                    .get_context_component_state_snapshot_at_commit(context_id, expected_head)
                    .await?;
                let parent_metadata = self
                    .repository
                    .get_context_replay_state_at_commit(context_id, expected_head)
                    .await?
                    .context_metadata()
                    .map(|payload| payload.metadata().clone());
                let (change, graph, attachment, metadata) = match operation {
                    ContextLifecycleOperation::UpdateMetadata { metadata } => {
                        metadata.validate()?;
                        if let Some(parent_metadata) = parent_metadata.as_ref() {
                            parent_metadata.validate()?;
                            if metadata.created_at() != parent_metadata.created_at() {
                                return Err(DomainValidationError::MetadataCreatedAtChanged {
                                    parent_created_at: parent_metadata.created_at(),
                                    successor_created_at: metadata.created_at(),
                                }
                                .into());
                            }
                        }
                        let resulting_metadata = metadata.clone();
                        let change = ContextChange::updated_metadata(metadata, &message);
                        (
                            change,
                            parent_snapshot.graph().clone(),
                            LifecycleAttachment::NoAttachment,
                            Some(resulting_metadata),
                        )
                    }
                    ContextLifecycleOperation::Create {
                        component_kind,
                        name,
                        metadata,
                        content,
                    } => {
                        let creation = ComponentContentCreationWrite::new(
                            ComponentId::new(),
                            component_kind,
                            name,
                            metadata,
                            content,
                            captured_at,
                        )?;
                        let change = ContextChange::added_component_content_with_details(
                            creation.component().id(),
                            creation.component().kind(),
                            creation.component().name().as_str(),
                            creation.metadata().clone(),
                            creation.resulting_content_hash(),
                            &message,
                        )?;
                        let graph = graph_with_created_component(
                            parent_snapshot.graph(),
                            context_id,
                            creation.component(),
                        )?;
                        (
                            change,
                            graph,
                            LifecycleAttachment::Creation(creation),
                            parent_metadata.clone(),
                        )
                    }
                    ContextLifecycleOperation::Update {
                        component_id,
                        content,
                    } => {
                        let state = self
                            .component_state_at_head(context_id, expected_head, component_id)
                            .await?;
                        validate_component_graph(
                            parent_snapshot.graph(),
                            context_id,
                            state.component(),
                        )?;
                        let revision = ComponentContentRevisionWrite::new(
                            component_id,
                            state.component().kind(),
                            state.component().content_hash().clone(),
                            content,
                            captured_at,
                        );
                        let change = ContextChange::updated_component_content(
                            component_id,
                            state.component().kind(),
                            state.component().content_hash().clone(),
                            revision.resulting_content_hash(),
                            &message,
                        );
                        (
                            change,
                            parent_snapshot.graph().clone(),
                            LifecycleAttachment::Revision(revision),
                            parent_metadata.clone(),
                        )
                    }
                    ContextLifecycleOperation::UpdateDescriptor {
                        component_id,
                        name,
                        metadata,
                    } => {
                        let state = self
                            .component_state_at_head(context_id, expected_head, component_id)
                            .await?;
                        validate_component_graph(
                            parent_snapshot.graph(),
                            context_id,
                            state.component(),
                        )?;
                        let component = ContextComponent::with_id(
                            component_id,
                            state.component().kind(),
                            &name,
                            state.component().content_hash().as_str(),
                        )?;
                        let change = ContextChange::updated_component_descriptor(
                            component_id,
                            component.kind(),
                            component.name().as_str(),
                            metadata.clone(),
                            &message,
                        )?;
                        let graph = graph_with_updated_component_descriptor(
                            parent_snapshot.graph(),
                            context_id,
                            &component,
                        )?;
                        (
                            change,
                            graph,
                            LifecycleAttachment::Descriptor(ComponentDescriptorRevisionWrite::new(
                                component,
                                metadata,
                                captured_at,
                            )),
                            parent_metadata.clone(),
                        )
                    }
                    ContextLifecycleOperation::Remove { component_id } => {
                        let state = self
                            .component_state_at_head(context_id, expected_head, component_id)
                            .await?;
                        validate_component_graph(
                            parent_snapshot.graph(),
                            context_id,
                            state.component(),
                        )?;
                        let removal = ComponentRemovalWrite::new(
                            component_id,
                            state.component().kind(),
                            state.component().content_hash().clone(),
                        );
                        let change = ContextChange::removed_component(
                            component_id,
                            state.component().kind(),
                            state.component().content_hash().clone(),
                            &message,
                        );
                        let graph = graph_without_component(parent_snapshot.graph(), component_id)?;
                        (
                            change,
                            graph,
                            LifecycleAttachment::Removal(removal),
                            parent_metadata.clone(),
                        )
                    }
                    ContextLifecycleOperation::AddUsesRelationship {
                        source_component_id,
                        target_component_id,
                    } => {
                        self.validate_uses_endpoints(
                            context_id,
                            expected_head,
                            parent_snapshot.graph(),
                            source_component_id,
                            target_component_id,
                        )
                        .await?;
                        let change = ContextChange::added_uses_relationship(
                            source_component_id,
                            target_component_id,
                            &message,
                        )?;
                        let graph = graph_with_added_uses_relationship(
                            parent_snapshot.graph(),
                            source_component_id,
                            target_component_id,
                        )?;
                        (
                            change,
                            graph,
                            LifecycleAttachment::NoAttachment,
                            parent_metadata.clone(),
                        )
                    }
                    ContextLifecycleOperation::RemoveUsesRelationship {
                        source_component_id,
                        target_component_id,
                    } => {
                        self.validate_uses_endpoints(
                            context_id,
                            expected_head,
                            parent_snapshot.graph(),
                            source_component_id,
                            target_component_id,
                        )
                        .await?;
                        let change = ContextChange::removed_uses_relationship(
                            source_component_id,
                            target_component_id,
                            &message,
                        )?;
                        let graph = graph_without_uses_relationship(
                            parent_snapshot.graph(),
                            source_component_id,
                            target_component_id,
                        )?;
                        (
                            change,
                            graph,
                            LifecycleAttachment::NoAttachment,
                            parent_metadata,
                        )
                    }
                    ContextLifecycleOperation::Initialize => {
                        unreachable!("handled before parent replay")
                    }
                };
                (
                    change,
                    graph,
                    Some(attachment),
                    vec![expected_head],
                    metadata,
                )
            }
        };
        let commit = ContextCommit::new(
            context_id,
            branch,
            message,
            parent_ids,
            vec![change],
            captured_at,
        )?;
        let snapshot_command = CreateContextCommitSnapshot::new_with_metadata(
            project_id,
            commit,
            graph,
            captured_at,
            1,
            metadata,
        )?;
        let guarded = GuardedContextCommitWrite::new(
            principal,
            expected_head,
            idempotency_key,
            request_digest,
            snapshot_command,
        )?;
        let guarded = match attachment {
            Some(LifecycleAttachment::Creation(creation)) => {
                guarded.with_component_content_creation(creation)?
            }
            Some(LifecycleAttachment::Revision(revision)) => {
                guarded.with_component_content_revision(revision)?
            }
            Some(LifecycleAttachment::Removal(removal)) => {
                guarded.with_component_removal(removal)?
            }
            Some(LifecycleAttachment::Descriptor(descriptor)) => {
                guarded.with_component_descriptor_revision(descriptor)?
            }
            Some(LifecycleAttachment::NoAttachment) => guarded,
            None => guarded,
        };
        let result = self
            .repository
            .create_guarded_commit_snapshot(guarded)
            .await?;
        let commit_id = result.snapshot.commit_id();

        Ok(ContextLifecycleWriteResult {
            commit_id,
            disposition: result.disposition,
            snapshot: result.snapshot,
        })
    }

    /// Returns exact replayed Context state and graph facts at a materialized commit.
    pub async fn read_state_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<ContextLifecycleStateAtCommit, ContextLifecycleError> {
        let facts = self
            .repository
            .get_context_lifecycle_read_facts(context_id, commit_id)
            .await?;
        if facts.scope().context_id() != context_id || facts.scope().commit_id() != commit_id {
            return Err(ContextLifecycleError::Storage(
                StorageRepositoryError::InvalidScope {
                    scope: format!("context:{context_id}/commit:{commit_id}"),
                    reason: format!("repository returned lifecycle facts for {}", facts.scope()),
                },
            ));
        }
        let graph_snapshot = facts.graph_snapshot().clone();
        let inventory = facts.inventory();
        facts.validate_consistency()?;
        let mut components = Vec::with_capacity(inventory.components().len());

        for state in inventory.components() {
            let component = state.component();
            let content = facts
                .contents()
                .iter()
                .find(|content| {
                    content.context_id() == context_id
                        && content.component_id() == component.id()
                        && content.commit_id() == state.content_commit_id()
                })
                .cloned()
                .ok_or(ContextLifecycleError::ComponentContentMissing {
                    component_id: component.id(),
                    commit_id,
                })?;
            components.push(ContextLifecycleComponentState {
                state: state.clone(),
                content,
            });
        }
        let replay_state = facts.replay_state();
        let metadata = replay_state
            .context_metadata()
            .map(|payload| payload.metadata().clone());

        Ok(ContextLifecycleStateAtCommit {
            scope: facts.scope(),
            components,
            metadata,
            graph_snapshot,
            replay_state: replay_state.clone(),
        })
    }

    async fn materialized_snapshot(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<CommitGraphSnapshot, ContextLifecycleError> {
        self.repository
            .get_commit_graph_snapshot(crate::CommitGraphSnapshotScope::new(
                project_id, context_id, commit_id,
            ))
            .await?
            .ok_or(ContextLifecycleError::MaterializedSnapshotMissing { commit_id })
    }

    async fn component_state_at_head(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
        component_id: ComponentId,
    ) -> Result<ComponentStateAtCommit, ContextLifecycleError> {
        self.repository
            .get_component_state_at_commit(context_id, commit_id, component_id)
            .await?
            .ok_or(ContextLifecycleError::ComponentStateMissing {
                component_id,
                commit_id,
            })
    }

    async fn validate_uses_endpoints(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
        graph: &ContextGraph,
        source_component_id: ComponentId,
        target_component_id: ComponentId,
    ) -> Result<(), ContextLifecycleError> {
        let source = self
            .component_state_at_head(context_id, commit_id, source_component_id)
            .await?;
        let target = self
            .component_state_at_head(context_id, commit_id, target_component_id)
            .await?;
        validate_component_graph(graph, context_id, source.component())?;
        validate_component_graph(graph, context_id, target.component())?;
        Ok(())
    }
}

fn validate_component_content_witness(
    state: &ComponentStateAtCommit,
    content: &ComponentContentRevision,
) -> Result<(), ContextLifecycleError> {
    let component = state.component();
    if content.context_id() != state.context_id()
        || content.component_id() != component.id()
        || content.commit_id() != state.content_commit_id()
        || content.component_kind() != component.kind()
        || content.resulting_content_hash() != component.content_hash()
    {
        return Err(ContextLifecycleError::ComponentWitnessConflict {
            component_id: component.id(),
        });
    }

    Ok(())
}

enum LifecycleAttachment {
    Creation(ComponentContentCreationWrite),
    Revision(ComponentContentRevisionWrite),
    Descriptor(ComponentDescriptorRevisionWrite),
    Removal(ComponentRemovalWrite),
    NoAttachment,
}

fn graph_with_created_component(
    parent_graph: &ContextGraph,
    context_id: ContextId,
    component: &ContextComponent,
) -> Result<ContextGraph, ContextLifecycleError> {
    let mut graph = parent_graph.clone();
    let stored_kind = StoredComponentKind::from_context_component_kind(component.kind());
    let component_node_id = format!("component:{}", component.id());
    graph.add_node(GraphNode::new(
        &component_node_id,
        stored_kind.graph_node_kind(),
        component.name().as_str(),
    )?)?;
    graph.add_edge(GraphEdge::new(
        format!("context:{context_id}"),
        component_node_id,
        stored_kind.graph_edge_kind(),
    )?)?;
    Ok(graph)
}

fn validate_component_graph(
    graph: &ContextGraph,
    context_id: ContextId,
    component: &ContextComponent,
) -> Result<(), ContextLifecycleError> {
    let component_node_id = format!("component:{}", component.id());
    let stored_kind = StoredComponentKind::from_context_component_kind(component.kind());
    let node = graph
        .nodes()
        .values()
        .find(|node| node.id().as_str() == component_node_id)
        .ok_or(ContextLifecycleError::ComponentGraphNodeMissing {
            component_id: component.id(),
        })?;
    if node.kind() != stored_kind.graph_node_kind()
        || node.label().as_str() != component.name().as_str()
        || !graph.edges().iter().any(|edge| {
            edge.source().as_str() == format!("context:{context_id}")
                && edge.target().as_str() == component_node_id
                && edge.kind() == stored_kind.graph_edge_kind()
        })
    {
        return Err(ContextLifecycleError::ComponentGraphNodeConflict {
            component_id: component.id(),
        });
    }
    Ok(())
}

fn graph_without_component(
    parent_graph: &ContextGraph,
    component_id: ComponentId,
) -> Result<ContextGraph, ContextLifecycleError> {
    let component_node_id = format!("component:{component_id}");
    let mut graph = ContextGraph::new();
    let mut found = false;

    for node in parent_graph.nodes().values() {
        if node.id().as_str() == component_node_id {
            found = true;
        } else {
            graph.add_node(node.clone())?;
        }
    }
    if !found {
        return Err(ContextLifecycleError::ComponentGraphNodeMissing { component_id });
    }
    for edge in parent_graph.edges() {
        if edge.source().as_str() != component_node_id
            && edge.target().as_str() != component_node_id
        {
            graph.add_edge(edge.clone())?;
        }
    }
    Ok(graph)
}

fn graph_with_added_uses_relationship(
    parent_graph: &ContextGraph,
    source_component_id: ComponentId,
    target_component_id: ComponentId,
) -> Result<ContextGraph, ContextLifecycleError> {
    let source_node_id = format!("component:{source_component_id}");
    let target_node_id = format!("component:{target_component_id}");
    if parent_graph.edges().iter().any(|edge| {
        edge.source().as_str() == source_node_id
            && edge.target().as_str() == target_node_id
            && edge.kind() == GraphEdgeKind::Uses
    }) {
        return Err(ContextLifecycleError::UsesRelationshipAlreadyExists {
            source_component_id,
            target_component_id,
        });
    }

    let mut graph = parent_graph.clone();
    graph.add_edge(GraphEdge::new(
        source_node_id,
        target_node_id,
        GraphEdgeKind::Uses,
    )?)?;
    Ok(graph)
}

fn graph_without_uses_relationship(
    parent_graph: &ContextGraph,
    source_component_id: ComponentId,
    target_component_id: ComponentId,
) -> Result<ContextGraph, ContextLifecycleError> {
    let source_node_id = format!("component:{source_component_id}");
    let target_node_id = format!("component:{target_component_id}");
    let mut graph = ContextGraph::new();
    let mut removed = false;

    for node in parent_graph.nodes().values() {
        graph.add_node(node.clone())?;
    }
    for edge in parent_graph.edges() {
        if edge.source().as_str() == source_node_id
            && edge.target().as_str() == target_node_id
            && edge.kind() == GraphEdgeKind::Uses
        {
            removed = true;
        } else {
            graph.add_edge(edge.clone())?;
        }
    }
    if !removed {
        return Err(ContextLifecycleError::UsesRelationshipMissing {
            source_component_id,
            target_component_id,
        });
    }
    Ok(graph)
}

fn graph_for_initialized_context(
    context_id: ContextId,
    context_name: &str,
) -> Result<ContextGraph, ContextLifecycleError> {
    let mut graph = ContextGraph::new();
    graph.add_node(GraphNode::new(
        format!("context:{context_id}"),
        GraphNodeKind::Context,
        context_name,
    )?)?;
    Ok(graph)
}

fn graph_with_updated_component_descriptor(
    parent_graph: &ContextGraph,
    context_id: ContextId,
    component: &ContextComponent,
) -> Result<ContextGraph, ContextLifecycleError> {
    let component_node_id = format!("component:{}", component.id());
    let stored_kind = StoredComponentKind::from_context_component_kind(component.kind());
    let mut graph = ContextGraph::new();
    let mut found = false;

    for node in parent_graph.nodes().values() {
        if node.id().as_str() == component_node_id {
            if node.kind() != stored_kind.graph_node_kind() {
                return Err(ContextLifecycleError::ComponentGraphNodeConflict {
                    component_id: component.id(),
                });
            }
            graph.add_node(GraphNode::new(
                &component_node_id,
                stored_kind.graph_node_kind(),
                component.name().as_str(),
            )?)?;
            found = true;
        } else {
            graph.add_node(node.clone())?;
        }
    }
    if !found {
        return Err(ContextLifecycleError::ComponentGraphNodeMissing {
            component_id: component.id(),
        });
    }
    for edge in parent_graph.edges() {
        graph.add_edge(edge.clone())?;
    }
    validate_component_graph(&graph, context_id, component)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use contextlab_auth::{
        AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity,
    };
    use contextlab_context_core::{
        ComponentContent, ComponentId, ContextComponent, ContextComponentKind, ContextId,
        ContextMetadata, DomainValidationError, ProjectId,
    };
    use contextlab_diff_engine::{ContextMetadataChangeV1, VersionedContextScopeV1};
    use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
    use contextlab_versioning::{
        BranchName, ContextChange, ContextChangeKind, ContextCommit, ExpectedBranchHead,
    };
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        ContextLifecycleCommand, ContextLifecycleError, ContextLifecycleOperation,
        ContextLifecycleReadFacts, ContextLifecycleService,
    };
    use crate::{
        CommitGraphSnapshot, CommitGraphSnapshotRepository, CommitGraphSnapshotScope,
        ComponentContentRevision, ComponentListQuery, ComponentStateAtCommit,
        ContextCommitRepository, ContextCommitSnapshotWriter, ContextComponentRepository,
        ContextComponentStateSnapshotAtCommit, ContextDiffSnapshotV1PairRepository,
        ContextGraphProjection, ContextGraphProjectionRepository, ContextRecord,
        CreateContextCommitSnapshot, GraphProjectionScope, GuardedContextCommitWrite,
        GuardedContextCommitWriter, IdempotencyKey, InMemoryContextGraphRepository,
        PersistedContextDiffReviewService, ProjectRecord, RequestDigest, StorageRepositoryError,
    };

    fn lifecycle_project_id() -> ProjectId {
        ProjectId::from_uuid(Uuid::from_u128(1))
    }

    fn lifecycle_projection(mut projection: ContextGraphProjection) -> ContextGraphProjection {
        let project_id = lifecycle_project_id();
        for context in &mut projection.contexts {
            context.project_id = project_id.to_string();
        }
        if !projection
            .projects
            .iter()
            .any(|project| project.id == project_id.to_string())
        {
            projection.projects.push(ProjectRecord {
                id: project_id.to_string(),
                workspace_id: "lifecycle-workspace".to_owned(),
                name: "Lifecycle Test Project".to_owned(),
                slug: "lifecycle-test-project".to_owned(),
                created_at: timestamp(0),
            });
        }
        projection
    }

    #[tokio::test]
    async fn unborn_context_initialization_creates_a_replayable_root_snapshot() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Initialization test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let lifecycle = ContextLifecycleService::new(&repository);
        let branch = BranchName::new("initialize-root").expect("branch");
        let principal = test_principal();
        let idempotency_key =
            IdempotencyKey::new("lifecycle-initialize-001").expect("idempotency key");
        let command = ContextLifecycleCommand::initialize(
            principal.clone(),
            context_id,
            branch.clone(),
            idempotency_key.clone(),
            RequestDigest::new("sha256:lifecycle-initialize-001").expect("request digest"),
            "Initialize Context lifecycle",
            timestamp(1),
        );

        let created = lifecycle
            .execute(command.clone())
            .await
            .expect("initialize unborn Context branch");
        let persisted_root = repository
            .get_commit(context_id.to_string(), created.commit_id().to_string())
            .await
            .expect("read persisted initialization root");
        let root_changes: Vec<ContextChange> =
            serde_json::from_value(persisted_root.changes).expect("decode root changes");
        let root_change = root_changes.first().expect("one root change");
        let persisted_snapshot = repository
            .get_commit_graph_snapshot(crate::CommitGraphSnapshotScope::new(
                lifecycle_project_id(),
                context_id,
                created.commit_id(),
            ))
            .await
            .expect("read persisted root snapshot");

        assert_eq!(
            repository.persisted_branch_head(context_id, &branch),
            Some(created.commit_id())
        );
        assert_eq!(
            repository.persisted_idempotency_receipt_commit_id(
                principal.identity().source(),
                principal.id(),
                context_id,
                &branch,
                &idempotency_key,
            ),
            Some(created.commit_id())
        );

        assert!(persisted_root.parent_commit_ids.is_empty());
        assert_eq!(persisted_root.change_count, 1);
        assert_eq!(root_changes.len(), 1);
        assert_eq!(root_change.kind(), ContextChangeKind::CreatedContext);
        assert_eq!(root_change.component_id(), None);
        assert_eq!(root_change.component_kind(), None);
        assert_eq!(root_change.component_name(), None);
        assert_eq!(root_change.component_metadata(), None);
        assert_eq!(root_change.previous_content_hash(), None);
        assert_eq!(root_change.resulting_content_hash(), None);
        assert_eq!(persisted_snapshot, Some(created.snapshot().clone()));
        assert!(
            repository
                .list_components(context_id.to_string(), ComponentListQuery::default())
                .await
                .expect("list root components")
                .items
                .is_empty()
        );
        assert!(
            lifecycle
                .read_state_at_commit(context_id, created.commit_id())
                .await
                .expect("read initialized Context state")
                .components()
                .is_empty()
        );
        let advanced = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                branch.clone(),
                created.commit_id(),
                IdempotencyKey::new("lifecycle-initialize-advance-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-initialize-advance-001")
                    .expect("request digest"),
                "Advance initialized Context",
                ContextComponentKind::Prompt,
                "Initialization follow-up",
                json!({}),
                ComponentContent::new("A committed follow-up body."),
                timestamp(2),
            ))
            .await
            .expect("advance initialized Context branch");
        let replayed = lifecycle
            .execute(command)
            .await
            .expect("replay initialized Context branch after a head advance");
        let other_branch = lifecycle
            .execute(ContextLifecycleCommand::initialize(
                test_principal(),
                context_id,
                BranchName::new("initialize-feature").expect("branch"),
                IdempotencyKey::new("lifecycle-initialize-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-initialize-001").expect("request digest"),
                "Initialize Context lifecycle",
                timestamp(3),
            ))
            .await
            .expect("initialize an independent unborn branch");

        assert_eq!(
            created.disposition(),
            crate::GuardedCommitWriteDisposition::Created
        );
        assert_eq!(
            replayed.disposition(),
            crate::GuardedCommitWriteDisposition::Replayed
        );
        assert_eq!(created.commit_id(), replayed.commit_id());
        assert_ne!(advanced.commit_id(), created.commit_id());
        assert_eq!(
            other_branch.disposition(),
            crate::GuardedCommitWriteDisposition::Created
        );
        assert_ne!(other_branch.commit_id(), created.commit_id());
        assert_eq!(created.snapshot().graph().nodes().len(), 1);
        let context_node_id = contextlab_graph::GraphNodeId::new(format!("context:{context_id}"))
            .expect("context node id");
        let node = created
            .snapshot()
            .graph()
            .nodes()
            .get(&context_node_id)
            .expect("root Context node");
        assert_eq!(node.kind(), GraphNodeKind::Context);
        assert_eq!(node.label().as_str(), "Initialization test Context");
    }

    #[tokio::test]
    async fn lifecycle_initialization_rejects_a_materialized_branch_head() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Initialization conflict Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;

        let error = ContextLifecycleService::new(&repository)
            .execute(ContextLifecycleCommand::from_operation(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-initialize-headed-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-initialize-headed-001")
                    .expect("request digest"),
                "Reject headed Context initialization",
                ContextLifecycleOperation::Initialize,
                timestamp(2),
            ))
            .await
            .expect_err("initialization must target an unborn branch");

        assert!(matches!(
            error,
            ContextLifecycleError::InitializationRequiresUnbornBranch
        ));
    }

    #[tokio::test]
    async fn lifecycle_create_returns_replayable_descriptor_body_and_graph_at_the_successor_commit()
    {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Lifecycle test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let lifecycle = ContextLifecycleService::new(&repository);

        let result = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-create-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-create-001").expect("request digest"),
                "Create support prompt",
                ContextComponentKind::Prompt,
                "Support prompt",
                json!({"locale": "bilingual"}),
                ComponentContent::new("Return a concise answer."),
                timestamp(2),
            ))
            .await
            .expect("create lifecycle component");
        let state = lifecycle
            .read_state_at_commit(context_id, result.commit_id())
            .await
            .expect("read lifecycle state");

        assert_eq!(state.components().len(), 1);
        assert_eq!(
            state.scope(),
            CommitGraphSnapshotScope::new(lifecycle_project_id(), context_id, result.commit_id())
        );
        assert_eq!(state.replay_state().context_id(), context_id);
        assert_eq!(state.replay_state().commit_id(), Some(result.commit_id()));
        assert_eq!(
            state.components()[0].state().component().name().as_str(),
            "Support prompt"
        );
        assert_eq!(
            state.components()[0].content().content().as_str(),
            "Return a concise answer."
        );
        assert!(
            state.graph_snapshot().graph().nodes().contains_key(
                &contextlab_graph::GraphNodeId::new(format!(
                    "component:{}",
                    state.components()[0].state().component().id()
                ))
                .expect("component node id")
            )
        );
    }

    #[test]
    fn lifecycle_read_rejects_a_body_witness_from_a_different_content_commit() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let creation_commit_id = contextlab_versioning::CommitId::new();
        let target_commit_id = contextlab_versioning::CommitId::new();
        let wrong_content_commit_id = contextlab_versioning::CommitId::new();
        let content = ComponentContent::new("body recorded elsewhere");
        let component = ContextComponent::with_id(
            component_id,
            ContextComponentKind::Prompt,
            "Prompt",
            content.content_hash().as_str(),
        )
        .expect("component");
        let state = ComponentStateAtCommit::new(
            context_id,
            target_commit_id,
            component,
            json!({}),
            creation_commit_id,
            target_commit_id,
        );
        let body = ComponentContentRevision::from_persisted(
            crate::component_content_revision::PersistedComponentContentRevision {
                context_id,
                commit_id: wrong_content_commit_id,
                component_id,
                component_kind: ContextComponentKind::Prompt,
                previous_content_hash: None,
                content: content.clone(),
                resulting_content_hash: content.content_hash(),
                captured_at: timestamp(1),
            },
        );

        let error = super::validate_component_content_witness(&state, &body)
            .expect_err("a body witness from another commit must fail closed");

        assert!(matches!(
            error,
            ContextLifecycleError::ComponentWitnessConflict { component_id: actual }
                if actual == component_id
        ));
    }

    #[test]
    fn lifecycle_read_facts_reject_a_graph_from_a_different_exact_scope() {
        let context_id = ContextId::new();
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create exact-scope read fixture",
            Vec::new(),
            vec![ContextChange::created_context(
                "Create exact-scope read fixture",
            )],
            timestamp(1),
        )
        .expect("commit");
        let commit_id = commit.id();
        let scope = CommitGraphSnapshotScope::new(lifecycle_project_id(), context_id, commit_id);
        let wrong_scope = CommitGraphSnapshotScope::new(ProjectId::new(), context_id, commit_id);
        let graph_snapshot = CommitGraphSnapshot::new(
            wrong_scope,
            ContextGraph::new(),
            timestamp(1),
            crate::COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("graph snapshot");
        let inventory = ContextComponentStateSnapshotAtCommit::new(context_id, commit_id, vec![]);
        let replay_state = contextlab_versioning::ReplayState::from_commits(context_id, &[commit])
            .expect("replay state");

        let error = ContextLifecycleReadFacts::from_parts(
            scope,
            inventory,
            Vec::new(),
            replay_state,
            graph_snapshot,
        )
        .expect_err("facts from another project must fail closed");

        assert!(matches!(error, StorageRepositoryError::InvalidScope { .. }));
    }

    #[tokio::test]
    async fn lifecycle_state_rejects_a_materialized_snapshot_that_omits_a_replayed_component_node()
    {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Lifecycle test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let lifecycle = ContextLifecycleService::new(&repository);
        let created = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-graph-integrity-create-001")
                    .expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-graph-integrity-create-001")
                    .expect("request digest"),
                "Create graph-integrity prompt",
                ContextComponentKind::Prompt,
                "Graph-integrity prompt",
                json!({}),
                ComponentContent::new("Require matching graph state."),
                timestamp(2),
            ))
            .await
            .expect("create lifecycle component");
        let component_id = lifecycle
            .read_state_at_commit(context_id, created.commit_id())
            .await
            .expect("read created lifecycle state")
            .components()[0]
            .state()
            .component()
            .id();
        let mut inconsistent_graph = ContextGraph::new();
        inconsistent_graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Lifecycle test Context",
                )
                .expect("context node"),
            )
            .expect("add context node");
        let inconsistent_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Materialize inconsistent generic graph",
            vec![created.commit_id()],
            vec![ContextChange::updated_metadata(
                ContextMetadata::new(timestamp(3)),
                "Materialize inconsistent generic graph",
            )],
            timestamp(3),
        )
        .expect("generic commit");
        let inconsistent_commit_id = inconsistent_commit.id();
        repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    test_principal(),
                    ExpectedBranchHead::Commit(created.commit_id()),
                    IdempotencyKey::new("lifecycle-graph-integrity-generic-001")
                        .expect("idempotency key"),
                    RequestDigest::new("sha256:lifecycle-graph-integrity-generic-001")
                        .expect("request digest"),
                    CreateContextCommitSnapshot::new(
                        lifecycle_project_id(),
                        inconsistent_commit,
                        inconsistent_graph,
                        timestamp(3),
                        1,
                    )
                    .expect("inconsistent snapshot"),
                )
                .expect("guarded generic commit"),
            )
            .await
            .expect("materialize inconsistent generic graph");

        let error = lifecycle
            .read_state_at_commit(context_id, inconsistent_commit_id)
            .await
            .expect_err("aggregate lifecycle state must reject graph-descriptor disagreement");
        assert!(matches!(
            error,
            ContextLifecycleError::ComponentGraphNodeMissing {
                component_id: actual_component_id
            } if actual_component_id == component_id
        ));
    }

    #[tokio::test]
    async fn lifecycle_update_and_removal_preserve_replay_and_remove_the_graph_node() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Lifecycle test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let lifecycle = ContextLifecycleService::new(&repository);
        let created = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-update-create-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-update-create-001").expect("request digest"),
                "Create mutable support prompt",
                ContextComponentKind::Prompt,
                "Mutable support prompt",
                json!({}),
                ComponentContent::new("Initial answer."),
                timestamp(2),
            ))
            .await
            .expect("create lifecycle component");
        let created_state = lifecycle
            .read_state_at_commit(context_id, created.commit_id())
            .await
            .expect("read created state");
        let component_id = created_state.components()[0].state().component().id();
        let updated = lifecycle
            .execute(ContextLifecycleCommand::update(
                test_principal(),
                context_id,
                BranchName::default(),
                created.commit_id(),
                IdempotencyKey::new("lifecycle-update-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-update-001").expect("request digest"),
                "Update mutable support prompt",
                component_id,
                ComponentContent::new("Revised answer."),
                timestamp(3),
            ))
            .await
            .expect("update lifecycle component");
        let updated_state = lifecycle
            .read_state_at_commit(context_id, updated.commit_id())
            .await
            .expect("read updated state");

        assert_eq!(updated_state.components().len(), 1);
        assert_eq!(
            updated_state.components()[0].content().content().as_str(),
            "Revised answer."
        );
        assert_eq!(
            lifecycle
                .read_state_at_commit(context_id, created.commit_id())
                .await
                .expect("read created history")
                .components()[0]
                .content()
                .content()
                .as_str(),
            "Initial answer."
        );

        let removed = lifecycle
            .execute(ContextLifecycleCommand::remove(
                test_principal(),
                context_id,
                BranchName::default(),
                updated.commit_id(),
                IdempotencyKey::new("lifecycle-remove-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-remove-001").expect("request digest"),
                "Remove mutable support prompt",
                component_id,
                timestamp(4),
            ))
            .await
            .expect("remove lifecycle component");
        let removed_state = lifecycle
            .read_state_at_commit(context_id, removed.commit_id())
            .await
            .expect("read removed state");

        assert!(removed_state.components().is_empty());
        assert!(
            !removed_state.graph_snapshot().graph().nodes().contains_key(
                &contextlab_graph::GraphNodeId::new(format!("component:{component_id}"))
                    .expect("component node id")
            )
        );
    }

    #[tokio::test]
    async fn lifecycle_replays_typed_uses_relationship_addition_and_removal() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Uses lifecycle Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let lifecycle = ContextLifecycleService::new(&repository);
        let first = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("uses-create-source-001").expect("idempotency key"),
                RequestDigest::new("sha256:uses-create-source-001").expect("request digest"),
                "Create Uses source",
                ContextComponentKind::Prompt,
                "Uses source",
                json!({}),
                ComponentContent::new("Source body."),
                timestamp(2),
            ))
            .await
            .expect("create source component");
        let second = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                first.commit_id(),
                IdempotencyKey::new("uses-create-target-001").expect("idempotency key"),
                RequestDigest::new("sha256:uses-create-target-001").expect("request digest"),
                "Create Uses target",
                ContextComponentKind::Knowledge,
                "Uses target",
                json!({}),
                ComponentContent::new("Target body."),
                timestamp(3),
            ))
            .await
            .expect("create target component");
        let state = lifecycle
            .read_state_at_commit(context_id, second.commit_id())
            .await
            .expect("read component state");
        let source_component_id = state.components()[0].state().component().id();
        let target_component_id = state.components()[1].state().component().id();
        let command = ContextLifecycleCommand::add_uses_relationship(
            test_principal(),
            context_id,
            BranchName::default(),
            second.commit_id(),
            IdempotencyKey::new("uses-add-001").expect("idempotency key"),
            RequestDigest::new("sha256:uses-add-001").expect("request digest"),
            "Add Uses relationship",
            source_component_id,
            target_component_id,
            timestamp(4),
        );
        let added = lifecycle
            .execute(command.clone())
            .await
            .expect("add Uses relationship");
        let replayed = lifecycle
            .execute(command)
            .await
            .expect("replay Uses relationship");

        assert_eq!(replayed.commit_id(), added.commit_id());
        assert_eq!(
            replayed.disposition(),
            crate::GuardedCommitWriteDisposition::Replayed
        );
        assert!(added.snapshot().graph().edges().iter().any(|edge| {
            edge.source().as_str() == format!("component:{source_component_id}")
                && edge.target().as_str() == format!("component:{target_component_id}")
                && edge.kind() == contextlab_graph::GraphEdgeKind::Uses
        }));
        let duplicate = lifecycle
            .execute(ContextLifecycleCommand::add_uses_relationship(
                test_principal(),
                context_id,
                BranchName::default(),
                added.commit_id(),
                IdempotencyKey::new("uses-add-duplicate-001").expect("idempotency key"),
                RequestDigest::new("sha256:uses-add-duplicate-001").expect("request digest"),
                "Reject duplicate Uses relationship",
                source_component_id,
                target_component_id,
                timestamp(5),
            ))
            .await
            .expect_err("duplicate Uses relationship must fail");
        assert!(matches!(
            duplicate,
            ContextLifecycleError::UsesRelationshipAlreadyExists { .. }
        ));
        let removed = lifecycle
            .execute(ContextLifecycleCommand::remove_uses_relationship(
                test_principal(),
                context_id,
                BranchName::default(),
                added.commit_id(),
                IdempotencyKey::new("uses-remove-001").expect("idempotency key"),
                RequestDigest::new("sha256:uses-remove-001").expect("request digest"),
                "Remove Uses relationship",
                source_component_id,
                target_component_id,
                timestamp(5),
            ))
            .await
            .expect("remove Uses relationship");
        assert!(!removed.snapshot().graph().edges().iter().any(|edge| {
            edge.source().as_str() == format!("component:{source_component_id}")
                && edge.target().as_str() == format!("component:{target_component_id}")
                && edge.kind() == contextlab_graph::GraphEdgeKind::Uses
        }));
        let missing = lifecycle
            .execute(ContextLifecycleCommand::remove_uses_relationship(
                test_principal(),
                context_id,
                BranchName::default(),
                removed.commit_id(),
                IdempotencyKey::new("uses-remove-missing-001").expect("idempotency key"),
                RequestDigest::new("sha256:uses-remove-missing-001").expect("request digest"),
                "Reject missing Uses relationship",
                source_component_id,
                target_component_id,
                timestamp(6),
            ))
            .await
            .expect_err("missing Uses relationship must fail");
        assert!(matches!(
            missing,
            ContextLifecycleError::UsesRelationshipMissing { .. }
        ));
    }

    #[tokio::test]
    async fn lifecycle_descriptor_revision_replays_metadata_and_graph_label_without_body_rewrite() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Lifecycle test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let lifecycle = ContextLifecycleService::new(&repository);
        let created = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-descriptor-create-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-descriptor-create-001")
                    .expect("request digest"),
                "Create descriptor prompt",
                ContextComponentKind::SystemPrompt,
                "Initial policy",
                json!({"locale": "en-US"}),
                ComponentContent::new("Follow the initial policy."),
                timestamp(2),
            ))
            .await
            .expect("create lifecycle component");
        let created_state = lifecycle
            .read_state_at_commit(context_id, created.commit_id())
            .await
            .expect("read created state");
        let component_id = created_state.components()[0].state().component().id();
        let original_hash = created_state.components()[0]
            .state()
            .component()
            .content_hash()
            .clone();

        let descriptor_command = ContextLifecycleCommand::update_descriptor(
            test_principal(),
            context_id,
            BranchName::default(),
            created.commit_id(),
            IdempotencyKey::new("lifecycle-descriptor-revise-001").expect("idempotency key"),
            RequestDigest::new("sha256:lifecycle-descriptor-revise-001").expect("request digest"),
            "Correct descriptor metadata",
            component_id,
            "Localized policy",
            json!({"locale": "zh-CN", "reviewed": true}),
            timestamp(3),
        );
        let revised = lifecycle
            .execute(descriptor_command.clone())
            .await
            .expect("revise lifecycle descriptor");
        let replayed = lifecycle
            .execute(descriptor_command)
            .await
            .expect("replay descriptor lifecycle revision");

        assert_eq!(replayed.commit_id(), revised.commit_id());
        assert_eq!(
            replayed.disposition(),
            crate::GuardedCommitWriteDisposition::Replayed
        );

        let current_detail = repository
            .get_component(context_id.to_string(), component_id.to_string())
            .await
            .expect("read current descriptor projection");
        assert_eq!(current_detail.name, "Localized policy");
        assert_eq!(
            current_detail.metadata,
            json!({"locale": "zh-CN", "reviewed": true})
        );
        let current_list = repository
            .list_components(context_id.to_string(), ComponentListQuery::default())
            .await
            .expect("list current descriptor projection");
        assert_eq!(current_list.items[0].name, "Localized policy");
        let current_projection = repository
            .load_context_graph_projection(GraphProjectionScope::Preview)
            .await
            .expect("load current graph projection");
        assert_eq!(
            current_projection
                .components
                .iter()
                .find(|component| component.id == component_id.to_string())
                .expect("current component graph projection record")
                .name,
            "Localized policy"
        );
        let revised_state = lifecycle
            .read_state_at_commit(context_id, revised.commit_id())
            .await
            .expect("read revised state");
        let revised_component = revised_state.components()[0].state().component();

        assert_eq!(revised_component.name().as_str(), "Localized policy");
        assert_eq!(
            revised_state.components()[0].state().metadata(),
            &json!({"locale": "zh-CN", "reviewed": true})
        );
        assert_eq!(revised_component.content_hash(), &original_hash);
        assert_eq!(
            revised_state.components()[0].content().content().as_str(),
            "Follow the initial policy."
        );
        assert_eq!(
            revised_state.components()[0].state().content_commit_id(),
            created.commit_id()
        );
        assert_eq!(
            revised_state
                .graph_snapshot()
                .graph()
                .nodes()
                .get(
                    &contextlab_graph::GraphNodeId::new(format!("component:{component_id}"))
                        .expect("component node id")
                )
                .expect("revised graph node")
                .label()
                .as_str(),
            "Localized policy"
        );

        let historical_state = lifecycle
            .read_state_at_commit(context_id, created.commit_id())
            .await
            .expect("read historical state");
        let historical_component = historical_state.components()[0].state();
        assert_eq!(
            historical_component.component().name().as_str(),
            "Initial policy"
        );
        assert_eq!(historical_component.metadata(), &json!({"locale": "en-US"}));
        assert_eq!(
            historical_component.component().content_hash(),
            &original_hash
        );
    }

    #[tokio::test]
    async fn lifecycle_update_metadata_replays_typed_metadata_through_the_guarded_writer() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Metadata lifecycle test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let root_snapshot = repository
            .get_commit_graph_snapshot(crate::CommitGraphSnapshotScope::new(
                lifecycle_project_id(),
                context_id,
                root,
            ))
            .await
            .expect("read root snapshot")
            .expect("materialized root snapshot");
        let lifecycle = ContextLifecycleService::new(&repository);
        let mut metadata = ContextMetadata::new(timestamp(2));
        metadata.set_label("owner", "luna", timestamp(3));
        let command = ContextLifecycleCommand::update_metadata(
            test_principal(),
            context_id,
            BranchName::default(),
            root,
            IdempotencyKey::new("lifecycle-metadata-001").expect("idempotency key"),
            RequestDigest::new("sha256:lifecycle-metadata-001").expect("request digest"),
            "Update Context metadata",
            metadata.clone(),
            timestamp(3),
        );

        let updated = lifecycle
            .execute(command.clone())
            .await
            .expect("update Context metadata");
        let replayed = lifecycle
            .execute(command)
            .await
            .expect("replay Context metadata update");
        let persisted = repository
            .get_commit(context_id.to_string(), updated.commit_id().to_string())
            .await
            .expect("read persisted metadata commit");
        let changes: Vec<ContextChange> =
            serde_json::from_value(persisted.changes).expect("decode metadata change");
        let state = lifecycle
            .read_state_at_commit(context_id, updated.commit_id())
            .await
            .expect("read metadata lifecycle state");
        let review = PersistedContextDiffReviewService::new(&repository)
            .review(
                VersionedContextScopeV1::new(lifecycle_project_id(), context_id, root),
                VersionedContextScopeV1::new(
                    lifecycle_project_id(),
                    context_id,
                    updated.commit_id(),
                ),
            )
            .await
            .expect("review persisted metadata transition");

        assert_eq!(updated.snapshot().graph(), root_snapshot.graph());
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].kind(), ContextChangeKind::UpdatedMetadata);
        assert_eq!(
            changes[0]
                .context_metadata()
                .expect("metadata payload")
                .metadata(),
            &metadata
        );
        assert_eq!(
            repository.persisted_branch_head(context_id, &BranchName::default()),
            Some(updated.commit_id())
        );
        assert_eq!(
            repository.persisted_idempotency_receipt_commit_id(
                test_principal().identity().source(),
                test_principal().id(),
                context_id,
                &BranchName::default(),
                &IdempotencyKey::new("lifecycle-metadata-001").expect("idempotency key"),
            ),
            Some(updated.commit_id())
        );
        assert_eq!(state.metadata(), Some(&metadata));
        assert_eq!(
            review.diff().semantic().metadata_change(),
            Some(&ContextMetadataChangeV1::Added {
                revised: metadata.clone(),
            })
        );

        let successor = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                updated.commit_id(),
                IdempotencyKey::new("lifecycle-metadata-successor-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-metadata-successor-001")
                    .expect("request digest"),
                "Create metadata-bearing successor",
                ContextComponentKind::Prompt,
                "Metadata-bearing prompt",
                json!({"locale": "en-US"}),
                ComponentContent::new("Retain the Context metadata."),
                timestamp(4),
            ))
            .await
            .expect("create metadata-bearing successor");
        let successor_review = PersistedContextDiffReviewService::new(&repository)
            .review(
                VersionedContextScopeV1::new(
                    lifecycle_project_id(),
                    context_id,
                    updated.commit_id(),
                ),
                VersionedContextScopeV1::new(
                    lifecycle_project_id(),
                    context_id,
                    successor.commit_id(),
                ),
            )
            .await
            .expect("review persisted metadata-bearing successor");
        let persisted_pair = repository
            .read_context_diff_snapshot_pair(
                VersionedContextScopeV1::new(
                    lifecycle_project_id(),
                    context_id,
                    updated.commit_id(),
                ),
                VersionedContextScopeV1::new(
                    lifecycle_project_id(),
                    context_id,
                    successor.commit_id(),
                ),
            )
            .await
            .expect("read persisted metadata-bearing successor pair");

        assert_eq!(successor_review.diff().semantic().metadata_change(), None);
        assert_eq!(
            persisted_pair.source().snapshot().semantic().metadata(),
            Some(&metadata)
        );
        assert_eq!(
            persisted_pair.target().snapshot().semantic().metadata(),
            Some(&metadata)
        );
        assert_eq!(replayed.commit_id(), updated.commit_id());
        assert_eq!(
            replayed.disposition(),
            crate::GuardedCommitWriteDisposition::Replayed
        );
    }

    #[tokio::test]
    async fn lifecycle_update_metadata_rejects_invalid_timestamps_before_the_guarded_writer() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Invalid metadata Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let metadata: ContextMetadata = serde_json::from_value(json!({
            "created_at": timestamp(3),
            "updated_at": timestamp(2),
            "labels": {},
        }))
        .expect("metadata shape");

        let error = ContextLifecycleService::new(&repository)
            .execute(ContextLifecycleCommand::update_metadata(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-invalid-metadata-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-invalid-metadata-001")
                    .expect("request digest"),
                "Reject invalid metadata timestamps",
                metadata,
                timestamp(4),
            ))
            .await
            .expect_err("invalid metadata timestamps must fail before the guarded writer");

        assert!(matches!(
            error,
            ContextLifecycleError::Domain(
                DomainValidationError::MetadataTimestampsOutOfOrder { .. }
            )
        ));
        assert_eq!(
            repository.persisted_branch_head(context_id, &BranchName::default()),
            Some(root)
        );
    }

    #[tokio::test]
    async fn lifecycle_update_metadata_rejects_created_at_drift_before_the_guarded_writer() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Metadata drift Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let lifecycle = ContextLifecycleService::new(&repository);
        let initial_metadata = ContextMetadata::new(timestamp(2));
        let initial = lifecycle
            .execute(ContextLifecycleCommand::update_metadata(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-metadata-drift-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-metadata-drift-001").expect("request digest"),
                "Set initial metadata",
                initial_metadata,
                timestamp(2),
            ))
            .await
            .expect("initial metadata update");

        let error = lifecycle
            .execute(ContextLifecycleCommand::update_metadata(
                test_principal(),
                context_id,
                BranchName::default(),
                initial.commit_id(),
                IdempotencyKey::new("lifecycle-metadata-drift-002").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-metadata-drift-002").expect("request digest"),
                "Reject metadata creation-time drift",
                ContextMetadata::new(timestamp(3)),
                timestamp(4),
            ))
            .await
            .expect_err("metadata created_at drift must fail before the guarded writer");

        assert!(matches!(
            error,
            ContextLifecycleError::Domain(DomainValidationError::MetadataCreatedAtChanged { .. })
        ));
        assert_eq!(
            repository.persisted_branch_head(context_id, &BranchName::default()),
            Some(initial.commit_id())
        );
    }

    #[test]
    fn context_metadata_deserialization_rejects_unknown_fields() {
        let error = serde_json::from_value::<ContextMetadata>(json!({
            "created_at": timestamp(1),
            "updated_at": timestamp(1),
            "labels": {},
            "unexpected": true,
        }))
        .expect_err("unknown metadata fields must be rejected");

        assert!(error.to_string().contains("unknown field"));
    }

    #[tokio::test]
    async fn lifecycle_replays_an_identical_request_and_rejects_a_stale_materialized_head() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Lifecycle test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let lifecycle = ContextLifecycleService::new(&repository);
        let command = ContextLifecycleCommand::create(
            test_principal(),
            context_id,
            BranchName::default(),
            root,
            IdempotencyKey::new("lifecycle-replay-001").expect("idempotency key"),
            RequestDigest::new("sha256:lifecycle-replay-001").expect("request digest"),
            "Create replayable support prompt",
            ContextComponentKind::Prompt,
            "Replayable support prompt",
            json!({}),
            ComponentContent::new("Replay the same answer."),
            timestamp(2),
        );
        let created = lifecycle
            .execute(command.clone())
            .await
            .expect("create lifecycle component");
        let replayed = lifecycle
            .execute(command)
            .await
            .expect("replay lifecycle component");

        assert_eq!(replayed.commit_id(), created.commit_id());
        assert_eq!(
            replayed.disposition(),
            crate::GuardedCommitWriteDisposition::Replayed
        );

        let stale = lifecycle
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                root,
                IdempotencyKey::new("lifecycle-stale-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-stale-001").expect("request digest"),
                "Reject stale support prompt",
                ContextComponentKind::Prompt,
                "Stale support prompt",
                json!({}),
                ComponentContent::new("Do not persist."),
                timestamp(3),
            ))
            .await
            .expect_err("stale lifecycle write must fail");

        assert!(matches!(
            stale,
            ContextLifecycleError::Storage(StorageRepositoryError::BranchHeadConflict { .. })
        ));
    }

    #[tokio::test]
    async fn lifecycle_create_rejects_a_materialized_merge_head_before_the_guarded_write() {
        let context_id = ContextId::new();
        let repository =
            InMemoryContextGraphRepository::new(lifecycle_projection(ContextGraphProjection {
                contexts: vec![ContextRecord {
                    id: context_id.to_string(),
                    project_id: "project".to_owned(),
                    experiment_id: None,
                    name: "Lifecycle test Context".to_owned(),
                    description: None,
                    created_at: timestamp(0),
                }],
                ..ContextGraphProjection::default()
            }));
        let root = materialized_root(&repository, context_id).await;
        let root_graph = repository
            .get_commit_graph_snapshot(crate::CommitGraphSnapshotScope::new(
                lifecycle_project_id(),
                context_id,
                root,
            ))
            .await
            .expect("read root snapshot")
            .expect("materialized root snapshot")
            .graph()
            .clone();
        let side_commit = ContextCommit::new(
            context_id,
            BranchName::new("side").expect("side branch"),
            "Create side history",
            vec![root],
            vec![ContextChange::created_context("Create side history")],
            timestamp(2),
        )
        .expect("side commit");
        let side_commit_id = side_commit.id();
        repository
            .create_commit_snapshot(
                CreateContextCommitSnapshot::new(
                    lifecycle_project_id(),
                    side_commit,
                    root_graph.clone(),
                    timestamp(2),
                    1,
                )
                .expect("side snapshot"),
            )
            .await
            .expect("materialize side history");
        let merge_commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Materialize unsupported merge",
            vec![root, side_commit_id],
            vec![ContextChange::created_context(
                "Materialize unsupported merge",
            )],
            timestamp(3),
        )
        .expect("merge commit");
        let merge_commit_id = merge_commit.id();
        repository
            .create_commit_snapshot(
                CreateContextCommitSnapshot::new(
                    lifecycle_project_id(),
                    merge_commit,
                    root_graph,
                    timestamp(3),
                    1,
                )
                .expect("merge snapshot"),
            )
            .await
            .expect("materialize merge history");

        let error = ContextLifecycleService::new(&repository)
            .execute(ContextLifecycleCommand::create(
                test_principal(),
                context_id,
                BranchName::default(),
                merge_commit_id,
                IdempotencyKey::new("lifecycle-merge-001").expect("idempotency key"),
                RequestDigest::new("sha256:lifecycle-merge-001").expect("request digest"),
                "Reject merge-head component",
                ContextComponentKind::Prompt,
                "Merge-head component",
                json!({}),
                ComponentContent::new("Do not write after a merge."),
                timestamp(4),
            ))
            .await
            .expect_err("merge head must fail before the guarded writer");

        assert!(matches!(
            error,
            ContextLifecycleError::Storage(
                StorageRepositoryError::ComponentStateReplayConflict { .. }
            )
        ));
    }

    async fn materialized_root(
        repository: &InMemoryContextGraphRepository,
        context_id: ContextId,
    ) -> contextlab_versioning::CommitId {
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Lifecycle test Context",
                )
                .expect("context node"),
            )
            .expect("add context node");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create lifecycle test Context",
            Vec::new(),
            vec![ContextChange::created_context(
                "Create lifecycle test Context",
            )],
            timestamp(1),
        )
        .expect("root commit");
        let root = commit.id();
        repository
            .create_guarded_commit_snapshot(
                GuardedContextCommitWrite::new(
                    test_principal(),
                    ExpectedBranchHead::Unborn,
                    IdempotencyKey::new("lifecycle-root-001").expect("idempotency key"),
                    RequestDigest::new("sha256:lifecycle-root-001").expect("request digest"),
                    CreateContextCommitSnapshot::new(
                        lifecycle_project_id(),
                        commit,
                        graph,
                        timestamp(1),
                        1,
                    )
                    .expect("root snapshot"),
                )
                .expect("guarded root"),
            )
            .await
            .expect("materialize root");
        root
    }

    fn test_principal() -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(PrincipalIdentity::new(
            IdentitySourceId::new("https://id.contextlab.test").expect("identity source"),
            PrincipalId::new("lifecycle-tester").expect("principal id"),
        ))
    }

    fn timestamp(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 15, 9, 0, seconds as u32)
            .single()
            .expect("timestamp")
    }
}
