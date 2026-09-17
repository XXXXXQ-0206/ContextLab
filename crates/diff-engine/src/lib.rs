//! Diff primitives for ContextLab.
//!
//! The first implementation is a deterministic line-oriented text diff. It is
//! intentionally small and dependency-light so semantic, behavior, and
//! evaluation diffs can layer on top of a stable contract.

use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single line in a text diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "text")]
pub enum DiffLine {
    /// The line appears in both inputs.
    Unchanged(String),
    /// The line appears only in the revised input.
    Added(String),
    /// The line appears only in the original input.
    Removed(String),
}

/// A deterministic line-oriented diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDiff {
    lines: Vec<DiffLine>,
}

/// A changed graph node with its original and revised attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeChange {
    node_id: String,
    original_kind: GraphNodeKind,
    revised_kind: GraphNodeKind,
    original_label: String,
    revised_label: String,
}

impl GraphNodeChange {
    /// Returns the stable node identifier.
    #[must_use]
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Returns the original node kind.
    #[must_use]
    pub const fn original_kind(&self) -> GraphNodeKind {
        self.original_kind
    }

    /// Returns the revised node kind.
    #[must_use]
    pub const fn revised_kind(&self) -> GraphNodeKind {
        self.revised_kind
    }

    /// Returns the original node label.
    #[must_use]
    pub fn original_label(&self) -> &str {
        &self.original_label
    }

    /// Returns the revised node label.
    #[must_use]
    pub fn revised_label(&self) -> &str {
        &self.revised_label
    }
}

/// A deterministic structural diff between two Context Graph snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDiff {
    added_nodes: Vec<GraphNode>,
    removed_nodes: Vec<GraphNode>,
    modified_nodes: Vec<GraphNodeChange>,
    added_edges: Vec<GraphEdge>,
    removed_edges: Vec<GraphEdge>,
}

impl GraphDiff {
    /// Compares two graph snapshots without relying on insertion order.
    #[must_use]
    pub fn between(original: &ContextGraph, revised: &ContextGraph) -> Self {
        let mut added_nodes = Vec::new();
        let mut removed_nodes = Vec::new();
        let mut modified_nodes = Vec::new();

        for (node_id, original_node) in original.nodes() {
            match revised.nodes().get(node_id) {
                None => removed_nodes.push(original_node.clone()),
                Some(revised_node)
                    if original_node.kind() != revised_node.kind()
                        || original_node.label() != revised_node.label() =>
                {
                    modified_nodes.push(GraphNodeChange {
                        node_id: node_id.as_str().to_owned(),
                        original_kind: original_node.kind(),
                        revised_kind: revised_node.kind(),
                        original_label: original_node.label().as_str().to_owned(),
                        revised_label: revised_node.label().as_str().to_owned(),
                    });
                }
                Some(_) => {}
            }
        }

        for (node_id, revised_node) in revised.nodes() {
            if !original.nodes().contains_key(node_id) {
                added_nodes.push(revised_node.clone());
            }
        }

        Self {
            added_nodes,
            removed_nodes,
            modified_nodes,
            added_edges: unmatched_edges(revised.edges(), original.edges()),
            removed_edges: unmatched_edges(original.edges(), revised.edges()),
        }
    }

    /// Returns nodes present only in the revised graph.
    #[must_use]
    pub fn added_nodes(&self) -> &[GraphNode] {
        &self.added_nodes
    }

    /// Returns nodes present only in the original graph.
    #[must_use]
    pub fn removed_nodes(&self) -> &[GraphNode] {
        &self.removed_nodes
    }

    /// Returns nodes whose kind or label changed.
    #[must_use]
    pub fn modified_nodes(&self) -> &[GraphNodeChange] {
        &self.modified_nodes
    }

    /// Returns edges present only in the revised graph.
    #[must_use]
    pub fn added_edges(&self) -> &[GraphEdge] {
        &self.added_edges
    }

    /// Returns edges present only in the original graph.
    #[must_use]
    pub fn removed_edges(&self) -> &[GraphEdge] {
        &self.removed_edges
    }

    /// Returns true when both graph structures are identical.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added_nodes.is_empty()
            && self.removed_nodes.is_empty()
            && self.modified_nodes.is_empty()
            && self.added_edges.is_empty()
            && self.removed_edges.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GraphEdgeKey {
    source: String,
    target: String,
    kind: GraphEdgeKind,
}

fn unmatched_edges(left: &[GraphEdge], right: &[GraphEdge]) -> Vec<GraphEdge> {
    let mut right_counts = BTreeMap::<GraphEdgeKey, usize>::new();
    for edge in right {
        *right_counts.entry(graph_edge_key(edge)).or_default() += 1;
    }

    let mut unmatched = Vec::new();
    for edge in left {
        let key = graph_edge_key(edge);
        match right_counts.get_mut(&key) {
            Some(count) if *count > 0 => *count -= 1,
            _ => unmatched.push(edge.clone()),
        }
    }

    unmatched.sort_by_key(graph_edge_key);
    unmatched
}

fn graph_edge_key(edge: &GraphEdge) -> GraphEdgeKey {
    GraphEdgeKey {
        source: edge.source().as_str().to_owned(),
        target: edge.target().as_str().to_owned(),
        kind: edge.kind(),
    }
}

impl TextDiff {
    /// Builds a text diff from two strings.
    #[must_use]
    pub fn between(original: &str, revised: &str) -> Self {
        let original_lines = split_lines(original);
        let revised_lines = split_lines(revised);
        let mut lines = diff_lines(&original_lines, &revised_lines);

        // `str::lines` intentionally normalizes line endings. Preserve an
        // exact text change whenever that normalization would hide it,
        // including when a line-ending change is mixed with content changes.
        let line_endings_changed = line_endings(original) != line_endings(revised);
        let normalized_diff_hides_change = lines
            .iter()
            .all(|line| matches!(line, DiffLine::Unchanged(_)));
        if original != revised
            && (!only_eof_newline_change(original, revised, &original_lines, &revised_lines)
                && (line_endings_changed || normalized_diff_hides_change))
        {
            lines = vec![
                DiffLine::Removed(original.to_owned()),
                DiffLine::Added(revised.to_owned()),
            ];
        }

        Self { lines }
    }

    /// Returns diff lines.
    #[must_use]
    pub fn lines(&self) -> &[DiffLine] {
        &self.lines
    }

    /// Returns true when no additions or removals were found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines
            .iter()
            .all(|line| matches!(line, DiffLine::Unchanged(_)))
    }
}

fn split_lines(input: &str) -> Vec<String> {
    let mut lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    if !input.is_empty() && (input.ends_with('\n') || input.ends_with('\r')) {
        // `str::lines` intentionally discards the final terminator. Keep an
        // explicit empty line so a semantic review cannot hide an EOF change.
        lines.push(String::new());
    }
    lines
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineEnding {
    CarriageReturn,
    LineFeed,
    CarriageReturnLineFeed,
}

fn line_endings(input: &str) -> Vec<LineEnding> {
    let bytes = input.as_bytes();
    let mut endings = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let ending = match bytes[index] {
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                index += 2;
                LineEnding::CarriageReturnLineFeed
            }
            b'\r' => {
                index += 1;
                LineEnding::CarriageReturn
            }
            b'\n' => {
                index += 1;
                LineEnding::LineFeed
            }
            _ => {
                index += 1;
                continue;
            }
        };
        endings.push(ending);
    }

    endings
}

fn only_eof_newline_change(
    original: &str,
    revised: &str,
    original_lines: &[String],
    revised_lines: &[String],
) -> bool {
    has_eof_newline(original) != has_eof_newline(revised)
        && lines_without_eof_marker(original_lines) == lines_without_eof_marker(revised_lines)
        && line_endings_before_eof_match(original, revised)
}

fn has_eof_newline(input: &str) -> bool {
    input.ends_with('\n') || input.ends_with('\r')
}

fn lines_without_eof_marker(lines: &[String]) -> &[String] {
    match lines.last() {
        Some(line) if line.is_empty() => &lines[..lines.len() - 1],
        _ => lines,
    }
}

fn line_endings_before_eof_match(original: &str, revised: &str) -> bool {
    let mut original_endings = line_endings(original);
    let mut revised_endings = line_endings(revised);

    if has_eof_newline(original) {
        original_endings.pop();
    }
    if has_eof_newline(revised) {
        revised_endings.pop();
    }

    original_endings == revised_endings
}

fn diff_lines(original: &[String], revised: &[String]) -> Vec<DiffLine> {
    let rows = original.len();
    let columns = revised.len();
    let mut lengths = vec![vec![0usize; columns + 1]; rows + 1];

    for row in (0..rows).rev() {
        for column in (0..columns).rev() {
            lengths[row][column] = if original[row] == revised[column] {
                lengths[row + 1][column + 1] + 1
            } else {
                lengths[row + 1][column].max(lengths[row][column + 1])
            };
        }
    }

    let mut row = 0;
    let mut column = 0;
    let mut lines = Vec::new();

    while row < rows && column < columns {
        if original[row] == revised[column] {
            lines.push(DiffLine::Unchanged(original[row].clone()));
            row += 1;
            column += 1;
        } else if lengths[row + 1][column] >= lengths[row][column + 1] {
            lines.push(DiffLine::Removed(original[row].clone()));
            row += 1;
        } else {
            lines.push(DiffLine::Added(revised[column].clone()));
            column += 1;
        }
    }

    while row < rows {
        lines.push(DiffLine::Removed(original[row].clone()));
        row += 1;
    }

    while column < columns {
        lines.push(DiffLine::Added(revised[column].clone()));
        column += 1;
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(
        nodes: &[(&str, GraphNodeKind, &str)],
        edges: &[(&str, &str, GraphEdgeKind)],
    ) -> ContextGraph {
        let mut graph = ContextGraph::new();

        for (id, kind, label) in nodes {
            graph
                .add_node(GraphNode::new(*id, *kind, *label).expect("node"))
                .expect("insert node");
        }

        for (source, target, kind) in edges {
            graph
                .add_edge(GraphEdge::new(*source, *target, *kind).expect("edge"))
                .expect("insert edge");
        }

        graph
    }

    #[test]
    fn detects_line_additions_and_removals() {
        let diff = TextDiff::between("system\nprompt\nmemory", "system\nprompt v2\nmemory");

        assert_eq!(
            diff.lines(),
            &[
                DiffLine::Unchanged("system".to_owned()),
                DiffLine::Removed("prompt".to_owned()),
                DiffLine::Added("prompt v2".to_owned()),
                DiffLine::Unchanged("memory".to_owned())
            ]
        );
    }

    #[test]
    fn identical_text_is_empty_diff() {
        let diff = TextDiff::between("a\nb", "a\nb");

        assert!(diff.is_empty());
    }

    #[test]
    fn preserves_an_end_of_file_newline_change() {
        let diff = TextDiff::between("policy", "policy\n");

        assert_eq!(
            diff.lines(),
            &[
                DiffLine::Unchanged("policy".to_owned()),
                DiffLine::Added(String::new()),
            ]
        );
        assert!(!diff.is_empty());
    }

    #[test]
    fn preserves_line_ending_changes() {
        let diff = TextDiff::between("policy\r\nmemory", "policy\nmemory");

        assert!(!diff.is_empty());
    }

    #[test]
    fn preserves_mixed_line_ending_changes_with_content_changes() {
        let original = "policy\r\nmemory";
        let revised = "policy\nmemory v2";

        assert_eq!(
            TextDiff::between(original, revised).lines(),
            &[
                DiffLine::Removed(original.to_owned()),
                DiffLine::Added(revised.to_owned()),
            ]
        );
    }

    #[test]
    fn preserves_eof_newline_changes_with_content_changes() {
        let original = "policy\nmemory";
        let revised = "policy\nmemory v2\n";

        assert_eq!(
            TextDiff::between(original, revised).lines(),
            &[
                DiffLine::Removed(original.to_owned()),
                DiffLine::Added(revised.to_owned()),
            ]
        );
    }

    #[test]
    fn preserves_line_ending_changes_when_an_eof_newline_is_added() {
        let original = "policy\r\nmemory";
        let revised = "policy\nmemory\n";

        assert_eq!(
            TextDiff::between(original, revised).lines(),
            &[
                DiffLine::Removed(original.to_owned()),
                DiffLine::Added(revised.to_owned()),
            ]
        );
    }

    #[test]
    fn graph_diff_reports_stable_structural_changes() {
        let original = graph(
            &[
                ("context:agent", GraphNodeKind::Context, "Support Agent"),
                ("prompt:system", GraphNodeKind::Prompt, "System Prompt"),
                ("tool:search", GraphNodeKind::Tool, "Search Tool"),
            ],
            &[
                ("context:agent", "prompt:system", GraphEdgeKind::Contains),
                ("context:agent", "tool:search", GraphEdgeKind::Uses),
            ],
        );
        let revised = graph(
            &[
                ("context:agent", GraphNodeKind::Workflow, "Support Agent v2"),
                (
                    "knowledge:policy",
                    GraphNodeKind::Knowledge,
                    "Refund Policy",
                ),
                ("tool:search", GraphNodeKind::Tool, "Search Tool"),
            ],
            &[
                (
                    "context:agent",
                    "knowledge:policy",
                    GraphEdgeKind::Retrieves,
                ),
                ("context:agent", "tool:search", GraphEdgeKind::Uses),
            ],
        );

        let diff = GraphDiff::between(&original, &revised);

        assert_eq!(diff.added_nodes().len(), 1);
        assert_eq!(diff.added_nodes()[0].id().as_str(), "knowledge:policy");
        assert_eq!(diff.removed_nodes().len(), 1);
        assert_eq!(diff.removed_nodes()[0].id().as_str(), "prompt:system");
        assert_eq!(diff.modified_nodes().len(), 1);
        assert_eq!(diff.modified_nodes()[0].node_id(), "context:agent");
        assert_eq!(
            diff.modified_nodes()[0].original_kind(),
            GraphNodeKind::Context
        );
        assert_eq!(
            diff.modified_nodes()[0].revised_kind(),
            GraphNodeKind::Workflow
        );
        assert_eq!(diff.modified_nodes()[0].original_label(), "Support Agent");
        assert_eq!(diff.modified_nodes()[0].revised_label(), "Support Agent v2");
        assert_eq!(diff.added_edges().len(), 1);
        assert_eq!(diff.added_edges()[0].target().as_str(), "knowledge:policy");
        assert_eq!(diff.removed_edges().len(), 1);
        assert_eq!(diff.removed_edges()[0].target().as_str(), "prompt:system");
        assert!(!diff.is_empty());
    }

    #[test]
    fn identical_graph_is_an_empty_graph_diff() {
        let graph = graph(
            &[("context:agent", GraphNodeKind::Context, "Support Agent")],
            &[],
        );

        assert!(GraphDiff::between(&graph, &graph).is_empty());
    }

    #[test]
    fn graph_diff_uses_a_stable_edge_multiset() {
        let original = graph(
            &[
                ("context:agent", GraphNodeKind::Context, "Support Agent"),
                ("tool:alpha", GraphNodeKind::Tool, "Alpha Tool"),
                ("tool:beta", GraphNodeKind::Tool, "Beta Tool"),
            ],
            &[
                ("context:agent", "tool:beta", GraphEdgeKind::Uses),
                ("context:agent", "tool:alpha", GraphEdgeKind::Uses),
                ("context:agent", "tool:alpha", GraphEdgeKind::Uses),
                ("context:agent", "tool:alpha", GraphEdgeKind::Uses),
            ],
        );
        let revised = graph(
            &[
                ("context:agent", GraphNodeKind::Context, "Support Agent"),
                ("tool:alpha", GraphNodeKind::Tool, "Alpha Tool"),
                ("tool:beta", GraphNodeKind::Tool, "Beta Tool"),
            ],
            &[
                ("context:agent", "tool:alpha", GraphEdgeKind::Uses),
                ("context:agent", "tool:beta", GraphEdgeKind::Uses),
                ("context:agent", "tool:beta", GraphEdgeKind::Uses),
                ("context:agent", "tool:beta", GraphEdgeKind::Uses),
            ],
        );

        let diff = GraphDiff::between(&original, &revised);

        assert_eq!(
            diff.removed_edges()
                .iter()
                .map(|edge| edge.target().as_str())
                .collect::<Vec<_>>(),
            vec!["tool:alpha", "tool:alpha"]
        );
        assert_eq!(
            diff.added_edges()
                .iter()
                .map(|edge| edge.target().as_str())
                .collect::<Vec<_>>(),
            vec!["tool:beta", "tool:beta"]
        );
    }
}

mod application;
mod contract;
mod graph_merge_conflict;
mod review;
mod versioned_graph_diff_review;
mod versioned_graph_merge_review;

pub use application::ContextDiffService;
pub use contract::{
    BehaviorCaseChangeV1, BehaviorDiffV1, BehaviorObservationV1, BehaviorOutcomeV1,
    BehaviorSnapshotV1, ContextDiffError, ContextDiffRequestV1, ContextDiffResultV1,
    ContextDiffSnapshotV1, ContextMetadataChangeV1, DiffContractVersion, DiffEntityId,
    DiffInputError, DiffSnapshotSide, EvaluationDiffV1, EvaluationMetricChangeV1,
    EvaluationMetricObservationV1, EvaluationSnapshotV1, SemanticDiffV1, SemanticDocumentChangeV1,
    SemanticDocumentV1, SemanticSnapshotV1,
};
pub use graph_merge_conflict::{
    GraphMergeChange, GraphMergeClassification, GraphMergeClassificationError, GraphMergeConflict,
    GraphMergeConflictClassifier, GraphMergeSnapshotSide, GraphSnapshotRef,
};
pub use review::{
    DiffReviewProjectionSchemaVersion, VersionedContextDiffReviewError,
    VersionedContextDiffReviewProjectionV1, VersionedContextDiffReviewRequestV1,
    VersionedContextDiffReviewService, VersionedContextScopeV1,
};
pub use versioned_graph_diff_review::{
    VersionedContextGraphDiffReviewError, VersionedContextGraphDiffReviewIdentityWitnessV1,
    VersionedContextGraphDiffReviewProjectionV1, VersionedContextGraphDiffReviewRequestV1,
    VersionedContextGraphDiffReviewService, VersionedGraphDiffReviewSchemaVersion,
    VersionedGraphDiffReviewSide,
};
pub use versioned_graph_merge_review::{
    VersionedContextGraphMergeReviewError, VersionedContextGraphMergeReviewProjectionV1,
    VersionedContextGraphMergeReviewRequestV1, VersionedContextGraphMergeReviewService,
    VersionedContextGraphSnapshotV1, VersionedGraphMergeReviewSchemaVersion,
};
