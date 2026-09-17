//! Guarded commit write command contracts.

use crate::{
    CommitGraphSnapshot, ComponentContentCreationError, ComponentContentCreationWrite,
    ComponentContentRevisionWrite, ComponentDescriptorRevisionError,
    ComponentDescriptorRevisionWrite, ComponentRemovalError, ComponentRemovalWrite,
    CreateContextCommitSnapshot, StorageRepositoryError, StoredComponentKind,
};
use async_trait::async_trait;
use contextlab_auth::AuthenticatedPrincipal;
use contextlab_versioning::{ContextChangeKind, ContextCommit, ExpectedBranchHead};
use std::fmt;
use thiserror::Error;

/// An opaque client-generated key for replaying one guarded write safely.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Validates an idempotency key.
    pub fn new(value: impl Into<String>) -> Result<Self, GuardedCommitWriteError> {
        let value = value.into().trim().to_owned();

        if value.is_empty() {
            return Err(GuardedCommitWriteError::EmptyIdempotencyKey);
        }

        Ok(Self(value))
    }

    /// Returns the validated key value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A canonical digest for the mutation payload protected by an idempotency key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestDigest(String);

impl RequestDigest {
    /// Validates a non-empty canonical request digest.
    pub fn new(value: impl Into<String>) -> Result<Self, GuardedCommitWriteError> {
        let value = value.into().trim().to_owned();

        if value.is_empty() {
            return Err(GuardedCommitWriteError::EmptyRequestDigest);
        }

        Ok(Self(value))
    }

    /// Returns the validated digest value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The single private component mutation attached to a guarded commit write.
#[derive(Debug, Clone)]
pub(crate) enum ComponentContentMutationWrite {
    /// Revises an existing component body.
    Revision(ComponentContentRevisionWrite),
    /// Creates a component with its initial body revision.
    Creation(ComponentContentCreationWrite),
    /// Soft-removes an existing component after checking its final content hash.
    Removal(ComponentRemovalWrite),
    /// Replaces an existing component descriptor without changing its body.
    Descriptor(ComponentDescriptorRevisionWrite),
}

/// Input for a commit write guarded by authorization, idempotency, and a branch head.
#[derive(Debug, Clone)]
pub struct GuardedContextCommitWrite {
    principal: AuthenticatedPrincipal,
    expected_branch_head: ExpectedBranchHead,
    idempotency_key: IdempotencyKey,
    request_digest: RequestDigest,
    snapshot_command: CreateContextCommitSnapshot,
    component_content_mutation: Option<ComponentContentMutationWrite>,
}

impl GuardedContextCommitWrite {
    /// Creates a guarded normal-commit command.
    pub fn new(
        principal: AuthenticatedPrincipal,
        expected_branch_head: ExpectedBranchHead,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
        snapshot_command: CreateContextCommitSnapshot,
    ) -> Result<Self, GuardedCommitWriteError> {
        if snapshot_command.commit().parent_ids().len() > 1 {
            return Err(GuardedCommitWriteError::MergeParentsUnsupported);
        }

        Ok(Self {
            principal,
            expected_branch_head,
            idempotency_key,
            request_digest,
            snapshot_command,
            component_content_mutation: None,
        })
    }

    /// Attaches one validated private component body revision to this guarded write.
    pub fn with_component_content_revision(
        mut self,
        component_content_revision: ComponentContentRevisionWrite,
    ) -> Result<Self, GuardedCommitWriteError> {
        if self.component_content_mutation.is_some() {
            return Err(GuardedCommitWriteError::ComponentContentMutationAlreadyAttached);
        }
        component_content_revision.validate_against(self.snapshot_command.commit())?;
        self.component_content_mutation = Some(ComponentContentMutationWrite::Revision(
            component_content_revision,
        ));
        Ok(self)
    }

    /// Attaches one validated private component creation to this guarded write.
    pub fn with_component_content_creation(
        mut self,
        component_content_creation: ComponentContentCreationWrite,
    ) -> Result<Self, GuardedCommitWriteError> {
        if self.component_content_mutation.is_some() {
            return Err(GuardedCommitWriteError::ComponentContentMutationAlreadyAttached);
        }
        component_content_creation.validate_against(self.snapshot_command.commit())?;
        validate_component_content_creation_snapshot(
            &component_content_creation,
            &self.snapshot_command,
        )?;
        self.component_content_mutation = Some(ComponentContentMutationWrite::Creation(
            component_content_creation,
        ));
        Ok(self)
    }

    /// Attaches one validated private component removal to this guarded write.
    pub fn with_component_removal(
        mut self,
        component_removal: ComponentRemovalWrite,
    ) -> Result<Self, GuardedCommitWriteError> {
        if self.component_content_mutation.is_some() {
            return Err(GuardedCommitWriteError::ComponentContentMutationAlreadyAttached);
        }
        component_removal.validate_against(self.snapshot_command.commit())?;
        validate_component_removal_snapshot(&component_removal, &self.snapshot_command)?;
        self.component_content_mutation =
            Some(ComponentContentMutationWrite::Removal(component_removal));
        Ok(self)
    }

    /// Attaches one validated private component descriptor revision to this guarded write.
    pub fn with_component_descriptor_revision(
        mut self,
        descriptor: ComponentDescriptorRevisionWrite,
    ) -> Result<Self, GuardedCommitWriteError> {
        if self.component_content_mutation.is_some() {
            return Err(GuardedCommitWriteError::ComponentContentMutationAlreadyAttached);
        }
        descriptor.validate_against(self.snapshot_command.commit())?;
        validate_component_descriptor_snapshot(&descriptor, &self.snapshot_command)?;
        self.component_content_mutation =
            Some(ComponentContentMutationWrite::Descriptor(descriptor));
        Ok(self)
    }

    /// Returns the authenticated principal requesting the mutation.
    #[must_use]
    pub const fn principal(&self) -> &AuthenticatedPrincipal {
        &self.principal
    }

    /// Returns the branch head observed by the client.
    #[must_use]
    pub const fn expected_branch_head(&self) -> ExpectedBranchHead {
        self.expected_branch_head
    }

    /// Returns the idempotency key for this mutation.
    #[must_use]
    pub const fn idempotency_key(&self) -> &IdempotencyKey {
        &self.idempotency_key
    }

    /// Returns the canonical request digest protected by the idempotency key.
    #[must_use]
    pub const fn request_digest(&self) -> &RequestDigest {
        &self.request_digest
    }

    /// Returns the atomically persisted commit snapshot command.
    #[must_use]
    pub const fn snapshot_command(&self) -> &CreateContextCommitSnapshot {
        &self.snapshot_command
    }

    /// Returns the optional private component body revision.
    #[must_use]
    pub const fn component_content_revision(&self) -> Option<&ComponentContentRevisionWrite> {
        match self.component_content_mutation.as_ref() {
            Some(ComponentContentMutationWrite::Revision(revision)) => Some(revision),
            Some(ComponentContentMutationWrite::Creation(_))
            | Some(ComponentContentMutationWrite::Removal(_))
            | Some(ComponentContentMutationWrite::Descriptor(_))
            | None => None,
        }
    }

    /// Returns the optional private component creation payload.
    #[must_use]
    pub const fn component_content_creation(&self) -> Option<&ComponentContentCreationWrite> {
        match self.component_content_mutation.as_ref() {
            Some(ComponentContentMutationWrite::Creation(creation)) => Some(creation),
            Some(ComponentContentMutationWrite::Revision(_))
            | Some(ComponentContentMutationWrite::Removal(_))
            | Some(ComponentContentMutationWrite::Descriptor(_))
            | None => None,
        }
    }

    /// Returns the optional private component removal payload.
    #[must_use]
    pub const fn component_removal(&self) -> Option<&ComponentRemovalWrite> {
        match self.component_content_mutation.as_ref() {
            Some(ComponentContentMutationWrite::Removal(removal)) => Some(removal),
            Some(ComponentContentMutationWrite::Revision(_))
            | Some(ComponentContentMutationWrite::Creation(_))
            | Some(ComponentContentMutationWrite::Descriptor(_))
            | None => None,
        }
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        AuthenticatedPrincipal,
        ExpectedBranchHead,
        IdempotencyKey,
        RequestDigest,
        CreateContextCommitSnapshot,
        Option<ComponentContentMutationWrite>,
    ) {
        (
            self.principal,
            self.expected_branch_head,
            self.idempotency_key,
            self.request_digest,
            self.snapshot_command,
            self.component_content_mutation,
        )
    }
}

fn validate_component_content_creation_snapshot(
    creation: &ComponentContentCreationWrite,
    snapshot_command: &CreateContextCommitSnapshot,
) -> Result<(), ComponentContentCreationError> {
    let context_id = snapshot_command.commit().context_id();
    let component_id = creation.component().id();
    let component_node_id = format!("component:{component_id}");
    let context_node_id = format!("context:{context_id}");
    let stored_kind = StoredComponentKind::from_context_component_kind(creation.component().kind());
    let component_node = snapshot_command
        .snapshot()
        .graph()
        .nodes()
        .values()
        .find(|node| node.id().as_str() == component_node_id)
        .ok_or(ComponentContentCreationError::SnapshotGraphMismatch)?;

    if component_node.kind() != stored_kind.graph_node_kind()
        || component_node.label().as_str() != creation.component().name().as_str()
    {
        return Err(ComponentContentCreationError::SnapshotGraphMismatch);
    }

    let has_relationship = snapshot_command
        .snapshot()
        .graph()
        .edges()
        .iter()
        .any(|edge| {
            edge.source().as_str() == context_node_id
                && edge.target().as_str() == component_node_id
                && edge.kind() == stored_kind.graph_edge_kind()
        });
    if !has_relationship {
        return Err(ComponentContentCreationError::SnapshotGraphMismatch);
    }

    Ok(())
}

fn validate_component_removal_snapshot(
    removal: &ComponentRemovalWrite,
    snapshot_command: &CreateContextCommitSnapshot,
) -> Result<(), ComponentRemovalError> {
    let component_node_id = format!("component:{}", removal.component_id());
    let contains_component_node = snapshot_command
        .snapshot()
        .graph()
        .nodes()
        .values()
        .any(|node| node.id().as_str() == component_node_id);

    if contains_component_node {
        Err(ComponentRemovalError::SnapshotGraphMismatch)
    } else {
        Ok(())
    }
}

fn validate_component_descriptor_snapshot(
    descriptor: &ComponentDescriptorRevisionWrite,
    snapshot_command: &CreateContextCommitSnapshot,
) -> Result<(), ComponentDescriptorRevisionError> {
    let component_node_id = format!("component:{}", descriptor.component_id());
    let Some(node) = snapshot_command
        .snapshot()
        .graph()
        .nodes()
        .values()
        .find(|node| node.id().as_str() == component_node_id)
    else {
        return Err(ComponentDescriptorRevisionError::SnapshotGraphMismatch);
    };

    if node.kind()
        != StoredComponentKind::from_context_component_kind(descriptor.component_kind())
            .graph_node_kind()
        || node.label().as_str() != descriptor.component().name().as_str()
    {
        return Err(ComponentDescriptorRevisionError::SnapshotGraphMismatch);
    }

    Ok(())
}

/// Ensures every component transition has its matching private mutation attachment.
pub(crate) fn validate_component_content_attachment(
    commit: &ContextCommit,
    component_content_mutation: Option<&ComponentContentMutationWrite>,
) -> Result<(), GuardedCommitWriteError> {
    let component_content_change_count = commit
        .changes()
        .iter()
        .filter(|change| {
            matches!(
                change.kind(),
                ContextChangeKind::AddedComponent
                    | ContextChangeKind::UpdatedComponent
                    | ContextChangeKind::RemovedComponent
            )
        })
        .count();
    let descriptor_change_count = commit
        .changes()
        .iter()
        .filter(|change| change.kind() == ContextChangeKind::UpdatedComponentDescriptor)
        .count();

    if descriptor_change_count > 0 {
        return if descriptor_change_count == 1
            && component_content_change_count == 0
            && matches!(
                component_content_mutation,
                Some(ComponentContentMutationWrite::Descriptor(_))
            ) {
            Ok(())
        } else {
            Err(GuardedCommitWriteError::ComponentContentMutationCountMismatch)
        };
    }

    match component_content_mutation {
        None if component_content_change_count == 0 => Ok(()),
        None => Err(GuardedCommitWriteError::ComponentCreationAttachmentRequired),
        Some(ComponentContentMutationWrite::Revision(revision))
            if component_content_change_count == 1 =>
        {
            revision.validate_against(commit)?;
            Ok(())
        }
        Some(ComponentContentMutationWrite::Creation(creation))
            if component_content_change_count == 1 =>
        {
            creation.validate_against(commit)?;
            Ok(())
        }
        Some(ComponentContentMutationWrite::Removal(removal))
            if component_content_change_count == 1 =>
        {
            removal.validate_against(commit)?;
            Ok(())
        }
        Some(ComponentContentMutationWrite::Descriptor(descriptor))
            if component_content_change_count == 0 =>
        {
            descriptor.validate_against(commit)?;
            Ok(())
        }
        Some(_) => Err(GuardedCommitWriteError::ComponentContentMutationCountMismatch),
    }
}

/// Whether a guarded write created a commit or replayed an existing result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardedCommitWriteDisposition {
    /// The command created a new commit, snapshot, and branch head.
    Created,
    /// An identical idempotency key returned its existing commit result.
    Replayed,
}

/// Immutable result of a successful guarded commit write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedCommitWriteResult {
    /// Snapshot captured with the resulting commit.
    pub snapshot: CommitGraphSnapshot,
    /// Indicates whether this call created or replayed the result.
    pub disposition: GuardedCommitWriteDisposition,
}

/// Atomically persists a guarded normal Context commit and advances its branch.
#[async_trait]
pub trait GuardedContextCommitWriter: Send + Sync {
    /// Creates a commit or returns an idempotent replay result.
    async fn create_guarded_commit_snapshot(
        &self,
        command: GuardedContextCommitWrite,
    ) -> Result<GuardedCommitWriteResult, StorageRepositoryError>;
}

/// Validation errors for guarded commit write commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GuardedCommitWriteError {
    /// The client did not provide a usable idempotency key.
    #[error("idempotency key must not be empty")]
    EmptyIdempotencyKey,
    /// The request digest was absent after trimming.
    #[error("request digest must not be empty")]
    EmptyRequestDigest,
    /// A merge requires a dedicated future workflow with explicit semantics.
    #[error("guarded normal commits cannot have more than one parent")]
    MergeParentsUnsupported,
    /// The attached component revision does not match the commit's content hash transition.
    #[error(transparent)]
    ComponentContentRevision(#[from] crate::ComponentContentRevisionError),
    /// The attached component creation does not match the commit or graph snapshot.
    #[error(transparent)]
    ComponentContentCreation(#[from] crate::ComponentContentCreationError),
    /// The attached component removal does not match the commit or graph snapshot.
    #[error(transparent)]
    ComponentRemoval(#[from] crate::ComponentRemovalError),
    /// The attached component descriptor revision does not match the commit or graph snapshot.
    #[error(transparent)]
    ComponentDescriptorRevision(#[from] crate::ComponentDescriptorRevisionError),
    /// A guarded commit accepts at most one private component mutation.
    #[error("guarded commit write accepts at most one component content mutation")]
    ComponentContentMutationAlreadyAttached,
    /// A component change needs its matching private creation or revision attachment.
    #[error("guarded component changes require exactly one private mutation attachment")]
    ComponentCreationAttachmentRequired,
    /// The guarded write contains more component changes than its one attachment can persist.
    #[error("guarded component write contains unsupported multiple component changes")]
    ComponentContentMutationCountMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use contextlab_auth::{
        AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity,
    };
    use contextlab_context_core::{
        ComponentContent, ComponentId, ContextComponentKind, ContextId, ProjectId,
    };
    use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
    use contextlab_versioning::{
        BranchName, CommitId, ContextChange, ContextCommit, ExpectedBranchHead,
    };

    use crate::{
        ComponentContentCreationWrite, ComponentRemovalWrite, CreateContextCommitSnapshot,
    };

    fn component_content_creation(component_id: ComponentId) -> ComponentContentCreationWrite {
        ComponentContentCreationWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            "Initial Prompt",
            serde_json::json!({"locale": "en"}),
            ComponentContent::new("initial prompt body"),
            Utc::now(),
        )
        .expect("valid creation")
    }

    fn creation_snapshot(
        context_id: ContextId,
        creation: &ComponentContentCreationWrite,
        graph: ContextGraph,
    ) -> CreateContextCommitSnapshot {
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create initial prompt",
            Vec::new(),
            vec![
                ContextChange::added_component_content_with_details(
                    creation.component().id(),
                    creation.component().kind(),
                    creation.component().name().as_str(),
                    creation.metadata().clone(),
                    creation.resulting_content_hash(),
                    "Create initial prompt",
                )
                .expect("valid replayable creation"),
            ],
            Utc::now(),
        )
        .expect("commit");

        CreateContextCommitSnapshot::new(ProjectId::new(), commit, graph, Utc::now(), 1)
            .expect("snapshot")
    }

    fn guarded_creation(snapshot: CreateContextCommitSnapshot) -> GuardedContextCommitWrite {
        GuardedContextCommitWrite::new(
            AuthenticatedPrincipal::new(PrincipalIdentity::new(
                IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new("user:alex").expect("principal"),
            )),
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("creation-request-001").expect("key"),
            RequestDigest::new("sha256:creation-request-001").expect("digest"),
            snapshot,
        )
        .expect("guarded command")
    }

    #[test]
    fn rejects_an_empty_idempotency_key() {
        assert!(matches!(
            IdempotencyKey::new("  "),
            Err(GuardedCommitWriteError::EmptyIdempotencyKey)
        ));
    }

    #[test]
    fn rejects_a_merge_commit_before_it_reaches_the_guarded_writer() {
        let commit = ContextCommit::new(
            ContextId::new(),
            BranchName::default(),
            "Merge context changes",
            vec![CommitId::new(), CommitId::new()],
            vec![ContextChange::created_context("Support agent")],
            Utc::now(),
        )
        .expect("commit");
        let snapshot = CreateContextCommitSnapshot::new(
            ProjectId::new(),
            commit,
            ContextGraph::new(),
            Utc::now(),
            1,
        )
        .expect("snapshot");
        let principal = AuthenticatedPrincipal::new(PrincipalIdentity::new(
            IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:alex").expect("principal"),
        ));

        let error = GuardedContextCommitWrite::new(
            principal,
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("request-001").expect("key"),
            RequestDigest::new("sha256:fixture").expect("digest"),
            snapshot,
        )
        .expect_err("merge parents belong to a later explicit workflow");

        assert_eq!(error, GuardedCommitWriteError::MergeParentsUnsupported);
    }

    #[test]
    fn component_content_revision_requires_a_matching_updated_component_change() {
        let component_id = ComponentId::new();
        let revision = ComponentContentRevisionWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            contextlab_context_core::ContentHash::new("sha256:previous").expect("previous hash"),
            ComponentContent::new("new prompt body"),
            Utc::now(),
        );
        let commit = ContextCommit::new(
            ContextId::new(),
            BranchName::default(),
            "Create context",
            Vec::new(),
            vec![ContextChange::created_context("Initial context")],
            Utc::now(),
        )
        .expect("commit");

        assert!(revision.validate_against(&commit).is_err());
    }

    #[test]
    fn component_removal_requires_a_matching_removed_component_change() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let previous_content_hash =
            contextlab_context_core::ContentHash::new("sha256:previous").expect("hash");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Remove prompt",
            Vec::new(),
            vec![ContextChange::removed_component(
                component_id,
                ContextComponentKind::Prompt,
                previous_content_hash.clone(),
                "Remove prompt",
            )],
            Utc::now(),
        )
        .expect("commit");
        let snapshot = CreateContextCommitSnapshot::new(
            ProjectId::new(),
            commit,
            ContextGraph::new(),
            Utc::now(),
            1,
        )
        .expect("removal snapshot");

        let command = GuardedContextCommitWrite::new(
            AuthenticatedPrincipal::new(PrincipalIdentity::new(
                IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new("user:alex").expect("principal"),
            )),
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("removal-request-001").expect("key"),
            RequestDigest::new("sha256:removal-request-001").expect("digest"),
            snapshot,
        )
        .expect("guarded command")
        .with_component_removal(ComponentRemovalWrite::new(
            component_id,
            ContextComponentKind::Prompt,
            previous_content_hash,
        ))
        .expect("matching removal attachment");

        assert!(command.component_removal().is_some());
    }

    #[test]
    fn component_removal_rejects_a_snapshot_that_retains_the_component_node() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let previous_content_hash =
            contextlab_context_core::ContentHash::new("sha256:previous").expect("hash");
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Remove prompt",
            Vec::new(),
            vec![ContextChange::removed_component(
                component_id,
                ContextComponentKind::Prompt,
                previous_content_hash.clone(),
                "Remove prompt",
            )],
            Utc::now(),
        )
        .expect("commit");
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{component_id}"),
                    GraphNodeKind::Prompt,
                    "Retained prompt",
                )
                .expect("component node"),
            )
            .expect("add component node");
        let snapshot =
            CreateContextCommitSnapshot::new(ProjectId::new(), commit, graph, Utc::now(), 1)
                .expect("removal snapshot");
        let command = GuardedContextCommitWrite::new(
            AuthenticatedPrincipal::new(PrincipalIdentity::new(
                IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new("user:alex").expect("principal"),
            )),
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("removal-node-request-001").expect("key"),
            RequestDigest::new("sha256:removal-node-request-001").expect("digest"),
            snapshot,
        )
        .expect("guarded command");

        assert!(matches!(
            command.with_component_removal(ComponentRemovalWrite::new(
                component_id,
                ContextComponentKind::Prompt,
                previous_content_hash,
            )),
            Err(GuardedCommitWriteError::ComponentRemoval(
                ComponentRemovalError::SnapshotGraphMismatch
            ))
        ));
    }

    #[test]
    fn component_content_creation_rejects_a_missing_snapshot_component_node() {
        let context_id = ContextId::new();
        let creation = component_content_creation(ComponentId::new());
        let snapshot = creation_snapshot(context_id, &creation, ContextGraph::new());

        assert!(
            guarded_creation(snapshot)
                .with_component_content_creation(creation)
                .is_err()
        );
    }

    #[test]
    fn added_component_changes_require_a_private_creation_attachment() {
        let commit = ContextCommit::new(
            ContextId::new(),
            BranchName::default(),
            "Add a component",
            Vec::new(),
            vec![ContextChange::added_component(
                ComponentId::new(),
                ContextComponentKind::Prompt,
                "Add prompt",
            )],
            Utc::now(),
        )
        .expect("commit");

        let error = validate_component_content_attachment(&commit, None)
            .expect_err("component additions cannot persist without a creation attachment");

        assert_eq!(
            error,
            GuardedCommitWriteError::ComponentCreationAttachmentRequired
        );
    }

    #[test]
    fn multiple_component_changes_require_one_attachment_per_supported_write() {
        let first_component_id = ComponentId::new();
        let second_component_id = ComponentId::new();
        let commit = ContextCommit::new(
            ContextId::new(),
            BranchName::default(),
            "Add prompts",
            Vec::new(),
            vec![
                ContextChange::added_component(
                    first_component_id,
                    ContextComponentKind::Prompt,
                    "First prompt",
                ),
                ContextChange::added_component(
                    second_component_id,
                    ContextComponentKind::Prompt,
                    "Second prompt",
                ),
            ],
            Utc::now(),
        )
        .expect("commit");
        let creation = component_content_creation(first_component_id);

        let error = validate_component_content_attachment(
            &commit,
            Some(&ComponentContentMutationWrite::Creation(creation)),
        )
        .expect_err("one private attachment cannot persist two component transitions");

        assert_eq!(
            error,
            GuardedCommitWriteError::ComponentContentMutationCountMismatch
        );
    }

    #[test]
    fn component_content_creation_rejects_a_mismatched_snapshot_component_node() {
        let context_id = ContextId::new();
        let creation = component_content_creation(ComponentId::new());
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Support Context",
                )
                .expect("context node"),
            )
            .expect("add context node");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{}", creation.component().id()),
                    GraphNodeKind::Prompt,
                    "Wrong Label",
                )
                .expect("component node"),
            )
            .expect("add component node");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    format!("component:{}", creation.component().id()),
                    GraphEdgeKind::Contains,
                )
                .expect("relationship"),
            )
            .expect("add relationship");
        let snapshot = creation_snapshot(context_id, &creation, graph);

        assert!(
            guarded_creation(snapshot)
                .with_component_content_creation(creation)
                .is_err()
        );
    }

    #[test]
    fn component_content_creation_rejects_a_missing_snapshot_relationship() {
        let context_id = ContextId::new();
        let creation = component_content_creation(ComponentId::new());
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Support Context",
                )
                .expect("context node"),
            )
            .expect("add context node");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{}", creation.component().id()),
                    GraphNodeKind::Prompt,
                    creation.component().name().as_str(),
                )
                .expect("component node"),
            )
            .expect("add component node");
        let snapshot = creation_snapshot(context_id, &creation, graph);

        assert!(
            guarded_creation(snapshot)
                .with_component_content_creation(creation)
                .is_err()
        );
    }

    #[test]
    fn guarded_component_content_mutation_is_mutually_exclusive() {
        let context_id = ContextId::new();
        let component_id = ComponentId::new();
        let creation = component_content_creation(component_id);
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Support Context",
                )
                .expect("context node"),
            )
            .expect("add context node");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{component_id}"),
                    GraphNodeKind::Prompt,
                    "Initial Prompt",
                )
                .expect("component node"),
            )
            .expect("add component node");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    format!("component:{component_id}"),
                    GraphEdgeKind::Contains,
                )
                .expect("relationship"),
            )
            .expect("add relationship");
        let command = guarded_creation(creation_snapshot(context_id, &creation, graph))
            .with_component_content_creation(creation)
            .expect("matching creation");

        let error = command
            .with_component_content_revision(ComponentContentRevisionWrite::new(
                component_id,
                ContextComponentKind::Prompt,
                contextlab_context_core::ContentHash::new("sha256:previous")
                    .expect("previous hash"),
                ComponentContent::new("updated body"),
                Utc::now(),
            ))
            .expect_err("a guarded write cannot attach both creation and revision");

        assert_eq!(
            error,
            GuardedCommitWriteError::ComponentContentMutationAlreadyAttached
        );
    }
}
