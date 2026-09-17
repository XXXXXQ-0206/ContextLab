//! Immutable Context Graph snapshots associated with Context commits.

use crate::StorageRepositoryError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
use contextlab_versioning::CommitId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use thiserror::Error;

/// The only supported persisted Context Graph snapshot schema.
pub const COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1: u16 = 1;

/// An exact, immutable project/Context/commit key for a materialized graph.
///
/// The project binding is deliberately part of the value rather than an
/// adapter-only query condition: a commit graph must never be read or replayed
/// outside the project that owns its Context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommitGraphSnapshotScope {
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
}

impl CommitGraphSnapshotScope {
    /// Creates an exact project/Context/commit snapshot key.
    #[must_use]
    pub const fn new(project_id: ProjectId, context_id: ContextId, commit_id: CommitId) -> Self {
        Self {
            project_id,
            context_id,
            commit_id,
        }
    }

    /// Returns the project that owns the Context.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the Context that owns the commit.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the commit whose graph was materialized.
    #[must_use]
    pub const fn commit_id(self) -> CommitId {
        self.commit_id
    }
}

impl fmt::Display for CommitGraphSnapshotScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "project:{}/context:{}/commit:{}",
            self.project_id, self.context_id, self.commit_id
        )
    }
}

/// An immutable Context Graph snapshot materialized for one Context commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitGraphSnapshot {
    #[serde(flatten)]
    scope: CommitGraphSnapshotScope,
    graph: ContextGraph,
    captured_at: DateTime<Utc>,
    schema_version: u16,
}

impl CommitGraphSnapshot {
    /// Creates a V1 snapshot associated with one exact Context commit.
    pub fn new(
        scope: CommitGraphSnapshotScope,
        graph: ContextGraph,
        captured_at: DateTime<Utc>,
        schema_version: u16,
    ) -> Result<Self, CommitGraphSnapshotError> {
        ensure_valid_scope(scope)?;
        ensure_supported_schema(schema_version)?;
        let graph = GraphSnapshotPayload::from(&graph).into_context_graph()?;

        Ok(Self {
            scope,
            graph,
            captured_at,
            schema_version,
        })
    }

    /// Returns the immutable exact scope that owns this snapshot.
    #[must_use]
    pub const fn scope(&self) -> CommitGraphSnapshotScope {
        self.scope
    }

    /// Returns the project identifier that owns the captured Context.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.scope.project_id()
    }

    /// Returns the Context identifier that owns the commit.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.scope.context_id()
    }

    /// Returns the associated commit identifier.
    #[must_use]
    pub const fn commit_id(&self) -> CommitId {
        self.scope.commit_id()
    }

    /// Returns the immutable graph aggregate captured for the commit.
    #[must_use]
    pub const fn graph(&self) -> &ContextGraph {
        &self.graph
    }

    /// Returns when this graph state was captured.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }

    /// Returns the snapshot payload schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Classifies a repeat write against this immutable fact.
    ///
    /// Adapters use this to implement idempotent replay without allowing an
    /// existing scope to be replaced by a graph with different content.
    #[must_use]
    pub fn replay_outcome(&self, candidate: &Self) -> CommitGraphSnapshotReplay {
        if self == candidate {
            CommitGraphSnapshotReplay::Replayed
        } else {
            CommitGraphSnapshotReplay::Conflict
        }
    }

    pub(crate) fn graph_payload(&self) -> Value {
        serde_json::to_value(GraphSnapshotPayload::from(self.graph())).expect("graph payload")
    }

    pub(crate) fn from_graph_payload(
        scope: CommitGraphSnapshotScope,
        graph: Value,
        captured_at: DateTime<Utc>,
        schema_version: u16,
    ) -> Result<Self, CommitGraphSnapshotError> {
        ensure_supported_schema(schema_version)?;
        let payload = serde_json::from_value::<GraphSnapshotPayload>(graph)
            .map_err(|_| CommitGraphSnapshotError::InvalidGraphPayload)?;
        Self::new(
            scope,
            payload.into_context_graph()?,
            captured_at,
            schema_version,
        )
    }
}

/// The result of attempting to store a snapshot at an already materialized scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitGraphSnapshotReplay {
    /// The candidate is byte-for-byte equivalent at the immutable scope.
    Replayed,
    /// The candidate would replace or alter an immutable snapshot.
    Conflict,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphSnapshotPayload {
    nodes: Vec<GraphNodePayload>,
    edges: Vec<GraphEdgePayload>,
}

impl From<&ContextGraph> for GraphSnapshotPayload {
    fn from(graph: &ContextGraph) -> Self {
        let mut edges = graph
            .edges()
            .iter()
            .map(GraphEdgePayload::from)
            .collect::<Vec<_>>();
        edges.sort_unstable_by(|left, right| {
            (&left.source, &left.target, left.kind).cmp(&(&right.source, &right.target, right.kind))
        });

        Self {
            nodes: graph.nodes().values().map(GraphNodePayload::from).collect(),
            edges,
        }
    }
}

impl GraphSnapshotPayload {
    fn into_context_graph(mut self) -> Result<ContextGraph, CommitGraphSnapshotError> {
        let mut graph = ContextGraph::new();
        for node in self.nodes {
            graph
                .add_node(
                    GraphNode::new(node.id, node.kind, node.label)
                        .map_err(|_| CommitGraphSnapshotError::InvalidGraphPayload)?,
                )
                .map_err(|_| CommitGraphSnapshotError::InvalidGraphPayload)?;
        }
        self.edges.sort_unstable_by(|left, right| {
            (&left.source, &left.target, left.kind).cmp(&(&right.source, &right.target, right.kind))
        });
        for edge in self.edges {
            graph
                .add_edge(
                    GraphEdge::new(edge.source, edge.target, edge.kind)
                        .map_err(|_| CommitGraphSnapshotError::InvalidGraphPayload)?,
                )
                .map_err(|_| CommitGraphSnapshotError::InvalidGraphPayload)?;
        }
        Ok(graph)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphNodePayload {
    id: String,
    kind: GraphNodeKind,
    label: String,
}
impl From<&GraphNode> for GraphNodePayload {
    fn from(node: &GraphNode) -> Self {
        Self {
            id: node.id().as_str().to_owned(),
            kind: node.kind(),
            label: node.label().as_str().to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphEdgePayload {
    source: String,
    target: String,
    kind: GraphEdgeKind,
}
impl From<&GraphEdge> for GraphEdgePayload {
    fn from(edge: &GraphEdge) -> Self {
        Self {
            source: edge.source().as_str().to_owned(),
            target: edge.target().as_str().to_owned(),
            kind: edge.kind(),
        }
    }
}

fn ensure_supported_schema(schema_version: u16) -> Result<(), CommitGraphSnapshotError> {
    if schema_version == COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1 {
        Ok(())
    } else {
        Err(CommitGraphSnapshotError::UnsupportedSchemaVersion { schema_version })
    }
}

fn ensure_valid_scope(scope: CommitGraphSnapshotScope) -> Result<(), CommitGraphSnapshotError> {
    if scope.project_id().as_uuid().is_nil()
        || scope.context_id().as_uuid().is_nil()
        || scope.commit_id().as_uuid().is_nil()
    {
        return Err(CommitGraphSnapshotError::InvalidScope { scope });
    }
    Ok(())
}

/// Errors returned while constructing snapshot records.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommitGraphSnapshotError {
    /// A snapshot schema version was not supported.
    #[error("unsupported commit graph snapshot schema version: {schema_version}")]
    UnsupportedSchemaVersion {
        /// Unsupported persisted schema version.
        schema_version: u16,
    },
    /// The immutable project/Context/commit scope contained a nil identity.
    #[error("commit graph snapshot scope contains a nil identity: {scope}")]
    InvalidScope {
        /// Invalid exact immutable scope.
        scope: CommitGraphSnapshotScope,
    },
    /// The persisted graph payload could not be validated.
    #[error("commit graph snapshot payload is invalid")]
    InvalidGraphPayload,
    /// More than one snapshot used the same immutable scope.
    #[error("duplicate commit graph snapshot for {scope}")]
    Duplicate {
        /// Exact immutable project/Context/commit scope.
        scope: CommitGraphSnapshotScope,
    },
}

/// Reads materialized Context Graph snapshots for existing Context commits.
#[async_trait]
pub trait CommitGraphSnapshotRepository: Send + Sync {
    /// Resolves the durable project owner for an existing Context.
    ///
    /// The project is intentionally derived from Context ownership rather than
    /// accepted from a caller, so guarded writes can bind new commits without
    /// trusting request scope.
    async fn project_id_for_context(
        &self,
        context_id: ContextId,
    ) -> Result<ProjectId, StorageRepositoryError>;

    /// Resolves the durable project owner for an existing Context commit.
    ///
    /// The project is intentionally derived from Context ownership rather than
    /// accepted from a caller, so adapters fail closed on unknown commits and
    /// cross-project scope assumptions.
    async fn scope_for_context_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError>;

    /// Returns a snapshot when the exact project/Context/commit exists and was materialized.
    async fn get_commit_graph_snapshot(
        &self,
        scope: CommitGraphSnapshotScope,
    ) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError>;

    /// Reads the complete base/left/right set through one repository boundary.
    ///
    /// The default keeps existing repository doubles and other adapters source
    /// compatible. Concrete repositories with a consistent read primitive must
    /// override it so all three reads share one guard or transaction.
    async fn get_commit_graph_snapshot_batch(
        &self,
        scope: crate::context_merge_review::ContextMergeInputScope,
    ) -> Result<
        (
            CommitGraphSnapshot,
            CommitGraphSnapshot,
            CommitGraphSnapshot,
        ),
        StorageRepositoryError,
    > {
        let scopes = [
            CommitGraphSnapshotScope::new(
                scope.project_id(),
                scope.context_id(),
                scope.base_commit_id(),
            ),
            CommitGraphSnapshotScope::new(
                scope.project_id(),
                scope.context_id(),
                scope.left_commit_id(),
            ),
            CommitGraphSnapshotScope::new(
                scope.project_id(),
                scope.context_id(),
                scope.right_commit_id(),
            ),
        ];
        let mut snapshots = Vec::with_capacity(scopes.len());
        for expected in scopes {
            let snapshot = self
                .get_commit_graph_snapshot(expected)
                .await?
                .ok_or_else(|| StorageRepositoryError::ScopeUnavailable {
                    scope: expected.to_string(),
                })?;
            snapshots.push(snapshot);
        }
        let [base, left, right] = snapshots
            .try_into()
            .expect("the batch scope always contains three snapshots");
        Ok((base, left, right))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use contextlab_graph::{GraphEdge, GraphNode, GraphNodeKind};

    fn scope() -> CommitGraphSnapshotScope {
        CommitGraphSnapshotScope::new(ProjectId::new(), ContextId::new(), CommitId::new())
    }

    fn captured_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 28, 8, 0, 0)
            .single()
            .expect("timestamp")
    }

    fn graph() -> ContextGraph {
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new("context:agent", GraphNodeKind::Context, "Support Agent")
                    .expect("node"),
            )
            .expect("insert node");
        graph
    }

    #[test]
    fn preserves_exact_project_context_commit_scope_and_graph_state() {
        let scope = scope();
        let snapshot = CommitGraphSnapshot::new(
            scope,
            graph(),
            captured_at(),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("snapshot");

        assert_eq!(snapshot.scope(), scope);
        assert_eq!(snapshot.project_id(), scope.project_id());
        assert_eq!(snapshot.context_id(), scope.context_id());
        assert_eq!(snapshot.commit_id(), scope.commit_id());
        assert_eq!(snapshot.graph().nodes().len(), 1);
        assert_eq!(snapshot.schema_version(), COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1);
    }

    #[test]
    fn rejects_nil_scope_identities_before_persistence() {
        let scopes = [
            CommitGraphSnapshotScope::new(
                ProjectId::from_uuid(uuid::Uuid::nil()),
                ContextId::new(),
                CommitId::new(),
            ),
            CommitGraphSnapshotScope::new(
                ProjectId::new(),
                ContextId::from_uuid(uuid::Uuid::nil()),
                CommitId::new(),
            ),
            CommitGraphSnapshotScope::new(
                ProjectId::new(),
                ContextId::new(),
                CommitId::from_uuid(uuid::Uuid::nil()),
            ),
        ];

        for scope in scopes {
            assert!(matches!(
                CommitGraphSnapshot::new(
                    scope,
                    ContextGraph::new(),
                    captured_at(),
                    COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
                ),
                Err(CommitGraphSnapshotError::InvalidScope { scope: actual }) if actual == scope
            ));
        }
    }

    #[test]
    fn rejects_unknown_schema_versions_fail_closed() {
        let scope = scope();
        let payload = serde_json::json!({ "nodes": [], "edges": [] });

        for schema_version in [0, 2, u16::MAX] {
            assert!(matches!(
                CommitGraphSnapshot::new(scope, graph(), captured_at(), schema_version),
                Err(CommitGraphSnapshotError::UnsupportedSchemaVersion { .. })
            ));
            assert!(matches!(
                CommitGraphSnapshot::from_graph_payload(
                    scope,
                    payload.clone(),
                    captured_at(),
                    schema_version,
                ),
                Err(CommitGraphSnapshotError::UnsupportedSchemaVersion { .. })
            ));
        }
    }

    #[test]
    fn graph_payload_is_deterministic_and_preserves_empty_graphs() {
        let scope = scope();
        let mut first = ContextGraph::new();
        let mut second = ContextGraph::new();

        for graph in [&mut first, &mut second] {
            graph
                .add_node(GraphNode::new("context:a", GraphNodeKind::Context, "A").expect("node"))
                .expect("node");
            graph
                .add_node(
                    GraphNode::new("component:b", GraphNodeKind::Component, "B").expect("node"),
                )
                .expect("node");
            graph
                .add_node(GraphNode::new("prompt:c", GraphNodeKind::Prompt, "C").expect("node"))
                .expect("node");
        }
        first
            .add_edge(
                GraphEdge::new("prompt:c", "context:a", GraphEdgeKind::Configures).expect("edge"),
            )
            .expect("edge");
        first
            .add_edge(
                GraphEdge::new("context:a", "component:b", GraphEdgeKind::Contains).expect("edge"),
            )
            .expect("edge");
        second
            .add_edge(
                GraphEdge::new("context:a", "component:b", GraphEdgeKind::Contains).expect("edge"),
            )
            .expect("edge");
        second
            .add_edge(
                GraphEdge::new("prompt:c", "context:a", GraphEdgeKind::Configures).expect("edge"),
            )
            .expect("edge");

        let first =
            CommitGraphSnapshot::new(scope, first, captured_at(), COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1)
                .expect("snapshot");
        let second = CommitGraphSnapshot::new(
            scope,
            second,
            captured_at(),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("snapshot");
        let empty = CommitGraphSnapshot::new(
            scope,
            ContextGraph::new(),
            captured_at(),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("empty snapshot");

        assert_eq!(first.graph_payload(), second.graph_payload());
        assert_eq!(
            empty.graph_payload(),
            serde_json::json!({ "nodes": [], "edges": [] })
        );

        let restored = CommitGraphSnapshot::from_graph_payload(
            scope,
            first.graph_payload(),
            captured_at(),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("restored snapshot");
        assert_eq!(restored.graph_payload(), first.graph_payload());
    }

    #[test]
    fn immutable_replay_accepts_only_an_identical_snapshot() {
        let scope = scope();
        let snapshot = CommitGraphSnapshot::new(
            scope,
            graph(),
            captured_at(),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("snapshot");
        let identical = snapshot.clone();
        let mut altered_graph = graph();
        altered_graph
            .add_node(
                GraphNode::new("component:policy", GraphNodeKind::Component, "Policy")
                    .expect("node"),
            )
            .expect("node");
        let altered = CommitGraphSnapshot::new(
            scope,
            altered_graph,
            captured_at(),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("altered snapshot");
        let different_scope = CommitGraphSnapshot::new(
            CommitGraphSnapshotScope::new(scope.project_id(), scope.context_id(), CommitId::new()),
            graph(),
            captured_at(),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("different scope snapshot");

        assert_eq!(
            snapshot.replay_outcome(&identical),
            CommitGraphSnapshotReplay::Replayed
        );
        assert_eq!(
            snapshot.replay_outcome(&altered),
            CommitGraphSnapshotReplay::Conflict
        );
        assert_eq!(
            snapshot.replay_outcome(&different_scope),
            CommitGraphSnapshotReplay::Conflict
        );
    }

    #[test]
    fn rejects_graph_payloads_with_unknown_fields() {
        let payload = serde_json::json!({
            "nodes": [],
            "edges": [],
            "untrusted": true,
        });

        assert!(matches!(
            CommitGraphSnapshot::from_graph_payload(
                scope(),
                payload,
                captured_at(),
                COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
            ),
            Err(CommitGraphSnapshotError::InvalidGraphPayload)
        ));
    }
}
