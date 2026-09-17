//! Storage-private consistency checks for exact replay and graph snapshots.

use crate::{CommitGraphSnapshot, StoredComponentKind};
use contextlab_context_core::{ComponentId, ContextId};
use contextlab_graph::{GraphEdgeKind, GraphNodeKind};
use contextlab_versioning::{CommitId, REPLAY_STATE_SCHEMA_VERSION, ReplayState};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Structured failure for the private exact-replay/graph consistency contract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum ReplayGraphConsistencyError {
    /// The replay state uses a schema that this storage contract cannot compare.
    #[error("replay state schema version {actual} is unsupported; expected {expected}")]
    ReplaySchemaVersionMismatch { actual: u16, expected: u16 },
    /// The materialized graph uses a schema that this storage contract cannot compare.
    #[error("commit graph snapshot schema version {actual} is unsupported; expected {expected}")]
    GraphSchemaVersionMismatch { actual: u16, expected: u16 },
    /// The replay state and graph snapshot belong to different Contexts.
    #[error("replay Context {replayed} does not match graph Context {snapshot}")]
    ContextIdMismatch {
        replayed: ContextId,
        snapshot: ContextId,
    },
    /// The replay state does not represent the exact graph commit.
    #[error("replay commit {replayed:?} does not match graph commit {snapshot}")]
    CommitIdMismatch {
        replayed: Option<CommitId>,
        snapshot: CommitId,
    },
    /// The graph is missing its Context root node.
    #[error("graph snapshot is missing Context node {node_id}")]
    ContextNodeMissing { node_id: String },
    /// The Context root node has the wrong kind.
    #[error("graph Context node {node_id} has an unexpected kind")]
    ContextNodeMismatch { node_id: String },
    /// A replayed component is absent from the graph.
    #[error("graph snapshot is missing component node {component_id}")]
    ComponentNodeMissing { component_id: ComponentId },
    /// A graph component node disagrees with replayed kind or name.
    #[error("graph component node {component_id} disagrees with replayed descriptor")]
    ComponentNodeMismatch { component_id: ComponentId },
    /// An expected graph edge is absent.
    #[error("graph snapshot is missing {kind:?} edge {source_id}->{target_id}")]
    MissingEdge {
        source_id: String,
        target_id: String,
        kind: GraphEdgeKind,
    },
    /// A graph edge appears more than once in the immutable payload.
    #[error("graph snapshot contains duplicate {kind:?} edge {source_id}->{target_id}")]
    DuplicateEdge {
        source_id: String,
        target_id: String,
        kind: GraphEdgeKind,
    },
    /// A graph node is not represented by replayed state.
    #[error("graph snapshot contains unexpected node {node_id}")]
    UnexpectedNode { node_id: String },
    /// A graph edge is not represented by replayed state.
    #[error("graph snapshot contains unexpected {kind:?} edge {source_id}->{target_id}")]
    UnexpectedEdge {
        source_id: String,
        target_id: String,
        kind: GraphEdgeKind,
    },
}

/// Validates that one exact replay state and graph snapshot describe the same Context state.
pub(crate) fn validate_replay_graph_consistency(
    replay: &ReplayState,
    snapshot: &CommitGraphSnapshot,
) -> Result<(), ReplayGraphConsistencyError> {
    if replay.schema_version() != REPLAY_STATE_SCHEMA_VERSION {
        return Err(ReplayGraphConsistencyError::ReplaySchemaVersionMismatch {
            actual: replay.schema_version(),
            expected: REPLAY_STATE_SCHEMA_VERSION,
        });
    }
    if snapshot.schema_version() != crate::COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1 {
        return Err(ReplayGraphConsistencyError::GraphSchemaVersionMismatch {
            actual: snapshot.schema_version(),
            expected: crate::COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        });
    }
    if replay.context_id() != snapshot.context_id() {
        return Err(ReplayGraphConsistencyError::ContextIdMismatch {
            replayed: replay.context_id(),
            snapshot: snapshot.context_id(),
        });
    }
    if replay.commit_id() != Some(snapshot.commit_id()) {
        return Err(ReplayGraphConsistencyError::CommitIdMismatch {
            replayed: replay.commit_id(),
            snapshot: snapshot.commit_id(),
        });
    }

    let context_node_id = format!("context:{}", replay.context_id());
    let mut expected_nodes = BTreeSet::from([context_node_id.clone()]);
    let mut expected_component_nodes = BTreeMap::new();
    for component in replay.components() {
        let component_node_id = format!("component:{}", component.component_id());
        expected_nodes.insert(component_node_id.clone());
        expected_component_nodes.insert(
            component_node_id,
            (
                component.component_id(),
                StoredComponentKind::from_context_component_kind(component.kind())
                    .graph_node_kind(),
                component.name().as_str().to_owned(),
            ),
        );
    }

    for node_id in &expected_nodes {
        let Some(node) = snapshot
            .graph()
            .nodes()
            .values()
            .find(|node| node.id().as_str() == node_id)
        else {
            if node_id == &context_node_id {
                return Err(ReplayGraphConsistencyError::ContextNodeMissing {
                    node_id: node_id.clone(),
                });
            }
            let component_id = expected_component_nodes
                .get(node_id)
                .expect("non-context expected node has a component identity")
                .0;
            return Err(ReplayGraphConsistencyError::ComponentNodeMissing { component_id });
        };

        if node_id == &context_node_id {
            if node.kind() != GraphNodeKind::Context {
                return Err(ReplayGraphConsistencyError::ContextNodeMismatch {
                    node_id: node_id.clone(),
                });
            }
            continue;
        }

        let (component_id, expected_kind, expected_label) = expected_component_nodes
            .get(node_id)
            .expect("component node is in expected map");
        if node.kind() != *expected_kind || node.label().as_str() != expected_label {
            return Err(ReplayGraphConsistencyError::ComponentNodeMismatch {
                component_id: *component_id,
            });
        }
    }

    for node in snapshot.graph().nodes().values() {
        if !expected_nodes.contains(node.id().as_str()) {
            return Err(ReplayGraphConsistencyError::UnexpectedNode {
                node_id: node.id().as_str().to_owned(),
            });
        }
    }

    let mut expected_edges = BTreeSet::new();
    for component in replay.components() {
        let stored_kind = StoredComponentKind::from_context_component_kind(component.kind());
        expected_edges.insert((
            context_node_id.clone(),
            format!("component:{}", component.component_id()),
            stored_kind.graph_edge_kind(),
        ));
    }
    for relationship in replay.relationships() {
        expected_edges.insert((
            format!("component:{}", relationship.source_component_id),
            format!("component:{}", relationship.target_component_id),
            GraphEdgeKind::Uses,
        ));
    }

    let mut actual_edges = BTreeSet::new();
    for edge in snapshot.graph().edges() {
        let key = (
            edge.source().as_str().to_owned(),
            edge.target().as_str().to_owned(),
            edge.kind(),
        );
        if !actual_edges.insert(key.clone()) {
            return Err(ReplayGraphConsistencyError::DuplicateEdge {
                source_id: key.0,
                target_id: key.1,
                kind: key.2,
            });
        }
        if !expected_edges.contains(&key) {
            return Err(ReplayGraphConsistencyError::UnexpectedEdge {
                source_id: key.0,
                target_id: key.1,
                kind: key.2,
            });
        }
    }
    for (source, target, kind) in expected_edges {
        if !actual_edges.contains(&(source.clone(), target.clone(), kind)) {
            return Err(ReplayGraphConsistencyError::MissingEdge {
                source_id: source,
                target_id: target,
                kind,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ReplayGraphConsistencyError, validate_replay_graph_consistency};
    use crate::{COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1, CommitGraphSnapshot, CommitGraphSnapshotScope};
    use chrono::{TimeZone, Utc};
    use contextlab_context_core::{
        ComponentId, ContentHash, ContextComponentKind, ContextId, ProjectId,
    };
    use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
    use contextlab_versioning::{BranchName, ContextChange, ContextCommit, ReplayState};
    use serde_json::json;
    use uuid::Uuid;

    fn timestamp(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(1_754_000_000 + seconds, 0)
            .single()
            .expect("valid timestamp")
    }

    fn fixture() -> (
        ReplayState,
        CommitGraphSnapshot,
        ContextId,
        ComponentId,
        ComponentId,
    ) {
        let context_id = ContextId::from_uuid(Uuid::from_u128(10));
        let prompt_id = ComponentId::from_uuid(Uuid::from_u128(11));
        let memory_id = ComponentId::from_uuid(Uuid::from_u128(12));
        let root = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Create context",
            Vec::new(),
            vec![
                ContextChange::created_context("Context"),
                ContextChange::added_component_content_with_details(
                    prompt_id,
                    ContextComponentKind::Prompt,
                    "Prompt",
                    json!({"role": "system"}),
                    ContentHash::new("sha256:prompt").expect("hash"),
                    "Add prompt",
                )
                .expect("prompt change"),
                ContextChange::added_component_content_with_details(
                    memory_id,
                    ContextComponentKind::Memory,
                    "Memory",
                    json!({"retention": "session"}),
                    ContentHash::new("sha256:memory").expect("hash"),
                    "Add memory",
                )
                .expect("memory change"),
            ],
            timestamp(0),
        )
        .expect("root commit");
        let relation = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Connect memory",
            vec![root.id()],
            vec![
                ContextChange::added_uses_relationship(prompt_id, memory_id, "Prompt uses memory")
                    .expect("relationship change"),
            ],
            timestamp(1),
        )
        .expect("relation commit");
        let commit_id = relation.id();
        let state = ReplayState::from_commits(context_id, &[root, relation]).expect("replay state");

        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Context",
                )
                .expect("context node"),
            )
            .expect("context node insert");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{prompt_id}"),
                    GraphNodeKind::Prompt,
                    "Prompt",
                )
                .expect("prompt node"),
            )
            .expect("prompt node insert");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{memory_id}"),
                    GraphNodeKind::Memory,
                    "Memory",
                )
                .expect("memory node"),
            )
            .expect("memory node insert");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    format!("component:{prompt_id}"),
                    GraphEdgeKind::Contains,
                )
                .expect("prompt edge"),
            )
            .expect("prompt edge insert");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    format!("component:{memory_id}"),
                    GraphEdgeKind::Contains,
                )
                .expect("memory edge"),
            )
            .expect("memory edge insert");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("component:{prompt_id}"),
                    format!("component:{memory_id}"),
                    GraphEdgeKind::Uses,
                )
                .expect("uses edge"),
            )
            .expect("uses edge insert");

        let snapshot = CommitGraphSnapshot::new(
            CommitGraphSnapshotScope::new(
                ProjectId::from_uuid(Uuid::from_u128(9)),
                context_id,
                commit_id,
            ),
            graph,
            timestamp(2),
            COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        )
        .expect("graph snapshot");
        (state, snapshot, context_id, prompt_id, memory_id)
    }

    #[test]
    fn accepts_matching_replay_and_graph_snapshot() {
        let (state, snapshot, _, _, _) = fixture();

        validate_replay_graph_consistency(&state, &snapshot).expect("matching state");
    }

    #[test]
    fn rejects_context_scope_drift() {
        let (state, snapshot, _, _, _) = fixture();
        let drifted = CommitGraphSnapshot::new(
            CommitGraphSnapshotScope::new(
                snapshot.project_id(),
                ContextId::from_uuid(Uuid::from_u128(99)),
                snapshot.commit_id(),
            ),
            snapshot.graph().clone(),
            snapshot.captured_at(),
            snapshot.schema_version(),
        )
        .expect("drifted snapshot");

        assert!(matches!(
            validate_replay_graph_consistency(&state, &drifted),
            Err(ReplayGraphConsistencyError::ContextIdMismatch { .. })
        ));
    }

    #[test]
    fn rejects_component_node_kind_or_name_drift() {
        let (state, snapshot, context_id, prompt_id, memory_id) = fixture();
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    "Context",
                )
                .expect("context node"),
            )
            .expect("context insert");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{prompt_id}"),
                    GraphNodeKind::Memory,
                    "Wrong prompt",
                )
                .expect("drifted node"),
            )
            .expect("prompt insert");
        graph
            .add_node(
                GraphNode::new(
                    format!("component:{memory_id}"),
                    GraphNodeKind::Memory,
                    "Memory",
                )
                .expect("memory node"),
            )
            .expect("memory insert");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    format!("component:{prompt_id}"),
                    GraphEdgeKind::Contains,
                )
                .expect("prompt edge"),
            )
            .expect("prompt edge insert");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    format!("component:{memory_id}"),
                    GraphEdgeKind::Contains,
                )
                .expect("memory edge"),
            )
            .expect("memory edge insert");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("component:{prompt_id}"),
                    format!("component:{memory_id}"),
                    GraphEdgeKind::Uses,
                )
                .expect("uses edge"),
            )
            .expect("uses edge insert");
        let drifted = CommitGraphSnapshot::new(
            snapshot.scope(),
            graph,
            snapshot.captured_at(),
            snapshot.schema_version(),
        )
        .expect("drifted snapshot");

        assert!(matches!(
            validate_replay_graph_consistency(&state, &drifted),
            Err(ReplayGraphConsistencyError::ComponentNodeMismatch { .. })
        ));
    }

    #[test]
    fn rejects_missing_uses_edge() {
        let (state, snapshot, context_id, prompt_id, memory_id) = fixture();
        let mut graph = snapshot.graph().clone();
        graph = graph_without_edge(
            &graph,
            format!("component:{prompt_id}"),
            format!("component:{memory_id}"),
            GraphEdgeKind::Uses,
            context_id,
        );
        let drifted = CommitGraphSnapshot::new(
            snapshot.scope(),
            graph,
            snapshot.captured_at(),
            snapshot.schema_version(),
        )
        .expect("drifted snapshot");

        assert!(matches!(
            validate_replay_graph_consistency(&state, &drifted),
            Err(ReplayGraphConsistencyError::MissingEdge { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_edges() {
        let (state, snapshot, _, _, _) = fixture();
        let mut graph = snapshot.graph().clone();
        graph
            .add_edge(graph.edges()[0].clone())
            .expect("duplicate edge is a graph-level input");
        let drifted = CommitGraphSnapshot::new(
            snapshot.scope(),
            graph,
            snapshot.captured_at(),
            snapshot.schema_version(),
        )
        .expect("drifted snapshot");

        assert!(matches!(
            validate_replay_graph_consistency(&state, &drifted),
            Err(ReplayGraphConsistencyError::DuplicateEdge { .. })
        ));
    }

    #[test]
    fn rejects_unexpected_nodes_and_edges() {
        let (state, snapshot, context_id, _, _) = fixture();
        let mut graph = snapshot.graph().clone();
        graph
            .add_node(
                GraphNode::new("unexpected", GraphNodeKind::Tool, "Unexpected").expect("node"),
            )
            .expect("unexpected node insert");
        graph
            .add_edge(
                GraphEdge::new(
                    format!("context:{context_id}"),
                    "unexpected",
                    GraphEdgeKind::Uses,
                )
                .expect("edge"),
            )
            .expect("unexpected edge insert");
        let drifted = CommitGraphSnapshot::new(
            snapshot.scope(),
            graph,
            snapshot.captured_at(),
            snapshot.schema_version(),
        )
        .expect("drifted snapshot");

        assert!(matches!(
            validate_replay_graph_consistency(&state, &drifted),
            Err(ReplayGraphConsistencyError::UnexpectedNode { .. })
        ));
    }

    fn graph_without_edge(
        graph: &ContextGraph,
        source: String,
        target: String,
        kind: GraphEdgeKind,
        context_id: ContextId,
    ) -> ContextGraph {
        let mut result = ContextGraph::new();
        for node in graph.nodes().values() {
            result.add_node(node.clone()).expect("copy node");
        }
        for edge in graph.edges() {
            if edge.source().as_str() == source
                && edge.target().as_str() == target
                && edge.kind() == kind
            {
                continue;
            }
            result.add_edge(edge.clone()).expect("copy edge");
        }
        let _ = context_id;
        result
    }
}
