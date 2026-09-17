//! Context Graph primitives for ContextLab.
//!
//! The graph crate models ContextLab resources as explicit nodes and
//! relationships. It is intentionally framework-independent so API, UI,
//! storage, evaluation, and workflow layers can share one graph contract.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Stable graph node identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GraphNodeId(String);

impl GraphNodeId {
    /// Creates a graph node id from a stable string.
    pub fn new(value: impl Into<String>) -> Result<Self, GraphValidationError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(GraphValidationError::EmptyNodeId);
        }
        Ok(Self(value))
    }

    /// Returns the node id string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A graph node label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphLabel(String);

impl GraphLabel {
    /// Creates a non-empty graph label.
    pub fn new(value: impl Into<String>) -> Result<Self, GraphValidationError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(GraphValidationError::EmptyLabel);
        }
        Ok(Self(value))
    }

    /// Returns the label string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Supported graph node kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    /// Workspace root.
    Workspace,
    /// Project inside a workspace.
    Project,
    /// Experiment tracking branch or run.
    Experiment,
    /// Context aggregate.
    Context,
    /// Context component.
    Component,
    /// Prompt component.
    Prompt,
    /// Memory component.
    Memory,
    /// Knowledge component.
    Knowledge,
    /// Tool or MCP server.
    Tool,
    /// Model configuration.
    Model,
    /// Evaluation dataset, suite, or result.
    Evaluation,
    /// Workflow node or definition.
    Workflow,
}

/// Supported relationship kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEdgeKind {
    /// Parent owns child.
    Owns,
    /// Context contains a component.
    Contains,
    /// Node configures another node.
    Configures,
    /// Node retrieves from another node.
    Retrieves,
    /// Node uses another node.
    Uses,
    /// Node evaluates another node.
    Evaluates,
    /// Node produced another node.
    Produces,
    /// Node belongs to an experiment.
    Tracks,
}

/// A resource node in the Context Graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    id: GraphNodeId,
    kind: GraphNodeKind,
    label: GraphLabel,
}

impl GraphNode {
    /// Creates a graph node.
    pub fn new(
        id: impl Into<String>,
        kind: GraphNodeKind,
        label: impl Into<String>,
    ) -> Result<Self, GraphValidationError> {
        Ok(Self {
            id: GraphNodeId::new(id)?,
            kind,
            label: GraphLabel::new(label)?,
        })
    }

    /// Returns the node id.
    #[must_use]
    pub const fn id(&self) -> &GraphNodeId {
        &self.id
    }

    /// Returns the node kind.
    #[must_use]
    pub const fn kind(&self) -> GraphNodeKind {
        self.kind
    }

    /// Returns the node label.
    #[must_use]
    pub const fn label(&self) -> &GraphLabel {
        &self.label
    }
}

/// A directed relationship in the Context Graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    source: GraphNodeId,
    target: GraphNodeId,
    kind: GraphEdgeKind,
}

impl GraphEdge {
    /// Creates a graph edge.
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        kind: GraphEdgeKind,
    ) -> Result<Self, GraphValidationError> {
        let source = GraphNodeId::new(source)?;
        let target = GraphNodeId::new(target)?;
        if source == target {
            return Err(GraphValidationError::SelfEdge { node_id: source });
        }

        Ok(Self {
            source,
            target,
            kind,
        })
    }

    /// Returns the edge source node id.
    #[must_use]
    pub const fn source(&self) -> &GraphNodeId {
        &self.source
    }

    /// Returns the edge target node id.
    #[must_use]
    pub const fn target(&self) -> &GraphNodeId {
        &self.target
    }

    /// Returns the edge kind.
    #[must_use]
    pub const fn kind(&self) -> GraphEdgeKind {
        self.kind
    }
}

/// A Context Graph aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextGraph {
    nodes: BTreeMap<GraphNodeId, GraphNode>,
    edges: Vec<GraphEdge>,
}

impl ContextGraph {
    /// Creates an empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    /// Adds a node.
    pub fn add_node(&mut self, node: GraphNode) -> Result<(), GraphValidationError> {
        if self.nodes.contains_key(node.id()) {
            return Err(GraphValidationError::DuplicateNode {
                node_id: node.id().clone(),
            });
        }
        self.nodes.insert(node.id().clone(), node);
        Ok(())
    }

    /// Adds an edge and validates that both endpoints exist.
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<(), GraphValidationError> {
        if !self.nodes.contains_key(edge.source()) {
            return Err(GraphValidationError::MissingNode {
                node_id: edge.source().clone(),
            });
        }
        if !self.nodes.contains_key(edge.target()) {
            return Err(GraphValidationError::MissingNode {
                node_id: edge.target().clone(),
            });
        }
        self.edges.push(edge);
        Ok(())
    }

    /// Returns nodes ordered by id.
    #[must_use]
    pub const fn nodes(&self) -> &BTreeMap<GraphNodeId, GraphNode> {
        &self.nodes
    }

    /// Returns graph edges in insertion order.
    #[must_use]
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// Builds the static preview graph used by early API and UI surfaces.
    pub fn context_engineering_preview() -> Result<Self, GraphValidationError> {
        let mut graph = Self::new();

        for node in [
            GraphNode::new(
                "workspace:default",
                GraphNodeKind::Workspace,
                "Default Workspace",
            )?,
            GraphNode::new(
                "project:support-ai",
                GraphNodeKind::Project,
                "Support AI Project",
            )?,
            GraphNode::new(
                "experiment:rag-v2",
                GraphNodeKind::Experiment,
                "RAG v2 Experiment",
            )?,
            GraphNode::new(
                "context:support-resolution-agent",
                GraphNodeKind::Context,
                "Support Resolution Agent",
            )?,
            GraphNode::new(
                "prompt:system-contract",
                GraphNodeKind::Prompt,
                "System Contract",
            )?,
            GraphNode::new("memory:timeline", GraphNodeKind::Memory, "Memory Timeline")?,
            GraphNode::new(
                "knowledge:refund-policy",
                GraphNodeKind::Knowledge,
                "Refund Policy Knowledge",
            )?,
            GraphNode::new("tool:mcp-search", GraphNodeKind::Tool, "MCP Search Tool")?,
            GraphNode::new(
                "model:deepseek",
                GraphNodeKind::Model,
                "DeepSeek Model Config",
            )?,
            GraphNode::new(
                "evaluation:safety-regression",
                GraphNodeKind::Evaluation,
                "Safety Regression Suite",
            )?,
        ] {
            graph.add_node(node)?;
        }

        for edge in [
            GraphEdge::new(
                "workspace:default",
                "project:support-ai",
                GraphEdgeKind::Owns,
            )?,
            GraphEdge::new(
                "project:support-ai",
                "experiment:rag-v2",
                GraphEdgeKind::Owns,
            )?,
            GraphEdge::new(
                "experiment:rag-v2",
                "context:support-resolution-agent",
                GraphEdgeKind::Tracks,
            )?,
            GraphEdge::new(
                "context:support-resolution-agent",
                "prompt:system-contract",
                GraphEdgeKind::Contains,
            )?,
            GraphEdge::new(
                "context:support-resolution-agent",
                "memory:timeline",
                GraphEdgeKind::Contains,
            )?,
            GraphEdge::new(
                "context:support-resolution-agent",
                "knowledge:refund-policy",
                GraphEdgeKind::Retrieves,
            )?,
            GraphEdge::new(
                "context:support-resolution-agent",
                "tool:mcp-search",
                GraphEdgeKind::Uses,
            )?,
            GraphEdge::new(
                "context:support-resolution-agent",
                "model:deepseek",
                GraphEdgeKind::Configures,
            )?,
            GraphEdge::new(
                "evaluation:safety-regression",
                "context:support-resolution-agent",
                GraphEdgeKind::Evaluates,
            )?,
        ] {
            graph.add_edge(edge)?;
        }

        Ok(graph)
    }
}

impl Default for ContextGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph validation errors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum GraphValidationError {
    /// Graph node id was empty.
    #[error("graph node id must not be empty")]
    EmptyNodeId,
    /// Graph label was empty.
    #[error("graph label must not be empty")]
    EmptyLabel,
    /// Node already exists.
    #[error("graph node {node_id:?} already exists")]
    DuplicateNode {
        /// Duplicated node id.
        node_id: GraphNodeId,
    },
    /// Edge references a missing node.
    #[error("graph node {node_id:?} does not exist")]
    MissingNode {
        /// Missing node id.
        node_id: GraphNodeId,
    },
    /// Edge source and target are the same node.
    #[error("graph edge cannot point {node_id:?} to itself")]
    SelfEdge {
        /// Self-referenced node id.
        node_id: GraphNodeId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_nodes_and_edges() {
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new("workspace:1", GraphNodeKind::Workspace, "Workspace").expect("node"),
            )
            .expect("insert workspace");
        graph
            .add_node(GraphNode::new("project:1", GraphNodeKind::Project, "Project").expect("node"))
            .expect("insert project");

        graph
            .add_edge(
                GraphEdge::new("workspace:1", "project:1", GraphEdgeKind::Owns).expect("edge"),
            )
            .expect("insert edge");

        assert_eq!(graph.nodes().len(), 2);
        assert_eq!(graph.edges().len(), 1);
    }

    #[test]
    fn rejects_duplicate_nodes() {
        let mut graph = ContextGraph::new();
        let node = GraphNode::new("context:1", GraphNodeKind::Context, "Context").expect("node");
        graph.add_node(node.clone()).expect("first insert");

        let error = graph.add_node(node).expect_err("duplicate must fail");

        assert!(matches!(error, GraphValidationError::DuplicateNode { .. }));
    }

    #[test]
    fn rejects_edges_with_missing_endpoints() {
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new("workspace:1", GraphNodeKind::Workspace, "Workspace").expect("node"),
            )
            .expect("insert workspace");

        let error = graph
            .add_edge(
                GraphEdge::new("workspace:1", "project:missing", GraphEdgeKind::Owns)
                    .expect("edge"),
            )
            .expect_err("missing target must fail");

        assert!(matches!(error, GraphValidationError::MissingNode { .. }));
    }

    #[test]
    fn preview_graph_contains_context_and_evaluation_relationships() {
        let graph = ContextGraph::context_engineering_preview().expect("valid preview graph");

        assert!(
            graph
                .nodes()
                .values()
                .any(|node| node.kind() == GraphNodeKind::Context)
        );
        assert!(
            graph
                .edges()
                .iter()
                .any(|edge| edge.kind() == GraphEdgeKind::Evaluates)
        );
    }
}
