//! Pure three-way Context Graph conflict classification.

use crate::GraphDiff;
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
use contextlab_versioning::{CommitId, MergePlan};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// The exact immutable scope of one graph snapshot used by a private merge review.
#[derive(Debug, Clone, Copy)]
pub struct GraphSnapshotRef<'a> {
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
    graph: &'a ContextGraph,
}

impl<'a> GraphSnapshotRef<'a> {
    /// Binds a graph to its exact project, Context, and commit identity.
    #[must_use]
    pub const fn new(
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
        graph: &'a ContextGraph,
    ) -> Self {
        Self {
            project_id,
            context_id,
            commit_id,
            graph,
        }
    }

    /// Returns the project scope.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the Context scope.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the commit scope.
    #[must_use]
    pub const fn commit_id(&self) -> CommitId {
        self.commit_id
    }

    /// Returns the immutable graph reference.
    #[must_use]
    pub const fn graph(&self) -> &'a ContextGraph {
        self.graph
    }
}

/// Side of a three-way graph review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphMergeSnapshotSide {
    /// The common base snapshot.
    Base,
    /// The left branch snapshot.
    Left,
    /// The right branch snapshot.
    Right,
}

/// A stable node or edge identity changed by one or both branches.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GraphMergeChange {
    /// A node identity changed.
    Node {
        /// Stable graph node identifier.
        node_id: String,
    },
    /// An edge identity changed.
    Edge {
        /// Stable edge source.
        source: String,
        /// Stable edge target.
        target: String,
        /// Stable edge kind.
        kind: GraphEdgeKind,
    },
}

/// A conflicting node or edge identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GraphMergeConflict {
    /// Both branches changed one node incompatibly.
    Node {
        /// Stable graph node identifier.
        node_id: String,
    },
    /// Both branches changed one edge incompatibly.
    Edge {
        /// Stable edge source.
        source: String,
        /// Stable edge target.
        target: String,
        /// Stable edge kind.
        kind: GraphEdgeKind,
    },
}

/// Deterministic result of a read-only three-way graph review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphMergeClassification {
    /// Branches changed disjoint graph identities, or only one branch changed.
    Clean {
        /// All changed identities in stable order.
        changes: Vec<GraphMergeChange>,
    },
    /// Branches made the same changes, including the empty change set.
    Equivalent {
        /// Shared changed identities in stable order.
        changes: Vec<GraphMergeChange>,
    },
    /// Branches changed one or more identities incompatibly.
    Conflict {
        /// Conflicting identities in stable order.
        conflicts: Vec<GraphMergeConflict>,
    },
}

/// Fail-closed errors for exact-scope three-way graph review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphMergeClassificationError {
    /// The ancestry plan is not a three-way plan.
    NotThreeWay,
    /// A snapshot has a different identity than the ancestry plan.
    PlanIdentityMismatch {
        /// Snapshot side with drift.
        side: GraphMergeSnapshotSide,
        /// Commit expected by the ancestry plan.
        expected: CommitId,
        /// Commit supplied by the snapshot.
        actual: CommitId,
    },
    /// A snapshot has a different project or Context scope.
    ScopeMismatch {
        /// Snapshot side with drift.
        side: GraphMergeSnapshotSide,
    },
}

impl fmt::Display for GraphMergeClassificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotThreeWay => {
                formatter.write_str("graph merge review requires a three-way ancestry plan")
            }
            Self::PlanIdentityMismatch { side, .. } => {
                write!(
                    formatter,
                    "{side:?} snapshot commit does not match the merge plan"
                )
            }
            Self::ScopeMismatch { side } => {
                write!(
                    formatter,
                    "{side:?} snapshot scope does not match the other merge inputs"
                )
            }
        }
    }
}

impl std::error::Error for GraphMergeClassificationError {}

/// Classifies graph changes for an exact three-way ancestry plan.
pub struct GraphMergeConflictClassifier;

impl GraphMergeConflictClassifier {
    /// Compares base-to-left and base-to-right using the sole `GraphDiff` calculator.
    pub fn classify(
        plan: &MergePlan,
        base: GraphSnapshotRef<'_>,
        left: GraphSnapshotRef<'_>,
        right: GraphSnapshotRef<'_>,
    ) -> Result<GraphMergeClassification, GraphMergeClassificationError> {
        let MergePlan::ThreeWay {
            left: plan_left,
            right: plan_right,
            base: plan_base,
        } = plan
        else {
            return Err(GraphMergeClassificationError::NotThreeWay);
        };

        validate_scope(&base, &left, &right)?;
        validate_commit(GraphMergeSnapshotSide::Base, *plan_base, base.commit_id())?;
        validate_commit(GraphMergeSnapshotSide::Left, *plan_left, left.commit_id())?;
        validate_commit(
            GraphMergeSnapshotSide::Right,
            *plan_right,
            right.commit_id(),
        )?;

        let left_diff = GraphDiff::between(base.graph(), left.graph());
        let right_diff = GraphDiff::between(base.graph(), right.graph());
        Ok(classify_diffs(&left_diff, &right_diff))
    }
}

fn validate_scope(
    base: &GraphSnapshotRef<'_>,
    left: &GraphSnapshotRef<'_>,
    right: &GraphSnapshotRef<'_>,
) -> Result<(), GraphMergeClassificationError> {
    if base.project_id() != left.project_id() || base.context_id() != left.context_id() {
        return Err(GraphMergeClassificationError::ScopeMismatch {
            side: GraphMergeSnapshotSide::Left,
        });
    }
    if base.project_id() != right.project_id() || base.context_id() != right.context_id() {
        return Err(GraphMergeClassificationError::ScopeMismatch {
            side: GraphMergeSnapshotSide::Right,
        });
    }
    Ok(())
}

fn validate_commit(
    side: GraphMergeSnapshotSide,
    expected: CommitId,
    actual: CommitId,
) -> Result<(), GraphMergeClassificationError> {
    if expected != actual {
        return Err(GraphMergeClassificationError::PlanIdentityMismatch {
            side,
            expected,
            actual,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NodeMutation {
    Added {
        kind: GraphNodeKind,
        label: String,
    },
    Removed {
        kind: GraphNodeKind,
        label: String,
    },
    Modified {
        original_kind: GraphNodeKind,
        revised_kind: GraphNodeKind,
        original_label: String,
        revised_label: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct EdgeKey {
    source: String,
    target: String,
    kind: GraphEdgeKind,
}

fn classify_diffs(left: &GraphDiff, right: &GraphDiff) -> GraphMergeClassification {
    let left_nodes = node_mutations(left);
    let right_nodes = node_mutations(right);
    let left_edges = edge_deltas(left);
    let right_edges = edge_deltas(right);

    let mut changes = Vec::new();
    let mut conflicts = Vec::new();
    let mut shared_changes = Vec::new();
    let mut has_one_sided_change = false;

    for node_id in left_nodes
        .keys()
        .chain(right_nodes.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
    {
        match (left_nodes.get(&node_id), right_nodes.get(&node_id)) {
            (Some(left_change), Some(right_change)) if left_change == right_change => {
                shared_changes.push(GraphMergeChange::Node { node_id });
            }
            (Some(_), Some(_)) => conflicts.push(GraphMergeConflict::Node { node_id }),
            (Some(_), None) | (None, Some(_)) => {
                has_one_sided_change = true;
                changes.push(GraphMergeChange::Node { node_id });
            }
            (None, None) => {}
        }
    }

    for edge in left_edges
        .keys()
        .chain(right_edges.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
    {
        match (left_edges.get(&edge), right_edges.get(&edge)) {
            (Some(left_delta), Some(right_delta)) if left_delta == right_delta => {
                shared_changes.push(edge_change(&edge));
            }
            (Some(_), Some(_)) => conflicts.push(edge_conflict(&edge)),
            (Some(_), None) | (None, Some(_)) => {
                has_one_sided_change = true;
                changes.push(edge_change(&edge));
            }
            (None, None) => {}
        }
    }

    if !conflicts.is_empty() {
        conflicts.sort();
        return GraphMergeClassification::Conflict { conflicts };
    }

    if has_one_sided_change {
        changes.extend(shared_changes);
        changes.sort();
        return GraphMergeClassification::Clean { changes };
    }

    shared_changes.sort();
    GraphMergeClassification::Equivalent {
        changes: shared_changes,
    }
}

fn node_mutations(diff: &GraphDiff) -> BTreeMap<String, NodeMutation> {
    let mut changes = BTreeMap::new();
    for node in diff.added_nodes() {
        changes.insert(node.id().as_str().to_owned(), added_node(node));
    }
    for node in diff.removed_nodes() {
        changes.insert(node.id().as_str().to_owned(), removed_node(node));
    }
    for node in diff.modified_nodes() {
        changes.insert(
            node.node_id().to_owned(),
            NodeMutation::Modified {
                original_kind: node.original_kind(),
                revised_kind: node.revised_kind(),
                original_label: node.original_label().to_owned(),
                revised_label: node.revised_label().to_owned(),
            },
        );
    }
    changes
}

fn added_node(node: &GraphNode) -> NodeMutation {
    NodeMutation::Added {
        kind: node.kind(),
        label: node.label().as_str().to_owned(),
    }
}

fn removed_node(node: &GraphNode) -> NodeMutation {
    NodeMutation::Removed {
        kind: node.kind(),
        label: node.label().as_str().to_owned(),
    }
}

fn edge_deltas(diff: &GraphDiff) -> BTreeMap<EdgeKey, i32> {
    let mut deltas = BTreeMap::new();
    for edge in diff.added_edges() {
        *deltas.entry(edge_key(edge)).or_default() += 1;
    }
    for edge in diff.removed_edges() {
        *deltas.entry(edge_key(edge)).or_default() -= 1;
    }
    deltas.retain(|_, delta| *delta != 0);
    deltas
}

fn edge_key(edge: &GraphEdge) -> EdgeKey {
    EdgeKey {
        source: edge.source().as_str().to_owned(),
        target: edge.target().as_str().to_owned(),
        kind: edge.kind(),
    }
}

fn edge_change(edge: &EdgeKey) -> GraphMergeChange {
    GraphMergeChange::Edge {
        source: edge.source.clone(),
        target: edge.target.clone(),
        kind: edge.kind,
    }
}

fn edge_conflict(edge: &EdgeKey) -> GraphMergeConflict {
    GraphMergeConflict::Edge {
        source: edge.source.clone(),
        target: edge.target.clone(),
        kind: edge.kind,
    }
}
