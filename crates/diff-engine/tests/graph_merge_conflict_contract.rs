//! Contract tests for private Context Graph three-way conflict classification.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::{
    GraphMergeClassification, GraphMergeClassificationError, GraphMergeConflict,
    GraphMergeConflictClassifier, GraphMergeSnapshotSide, GraphSnapshotRef,
};
use contextlab_graph::{ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
use contextlab_versioning::{CommitId, MergePlan};

fn snapshot(
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
    graph: &ContextGraph,
) -> GraphSnapshotRef<'_> {
    GraphSnapshotRef::new(project_id, context_id, commit_id, graph)
}

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

fn plan(base: CommitId, left: CommitId, right: CommitId) -> MergePlan {
    MergePlan::ThreeWay { base, left, right }
}

#[test]
fn classifies_disjoint_node_changes_as_clean() {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let base_id = CommitId::new();
    let left_id = CommitId::new();
    let right_id = CommitId::new();
    let base = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("prompt:1", GraphNodeKind::Prompt, "Prompt"),
        ],
        &[],
    );
    let left = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("prompt:1", GraphNodeKind::Prompt, "Prompt v2"),
        ],
        &[],
    );
    let right = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("tool:1", GraphNodeKind::Tool, "Search"),
            ("prompt:1", GraphNodeKind::Prompt, "Prompt"),
        ],
        &[],
    );

    let result = GraphMergeConflictClassifier::classify(
        &plan(base_id, left_id, right_id),
        snapshot(project_id, context_id, base_id, &base),
        snapshot(project_id, context_id, left_id, &left),
        snapshot(project_id, context_id, right_id, &right),
    )
    .expect("clean classification");

    assert!(matches!(result, GraphMergeClassification::Clean { .. }));
}

#[test]
fn classifies_identical_node_changes_as_equivalent() {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let base_id = CommitId::new();
    let left_id = CommitId::new();
    let right_id = CommitId::new();
    let base = graph(&[("context:1", GraphNodeKind::Context, "Context")], &[]);
    let changed = graph(&[("context:1", GraphNodeKind::Context, "Context v2")], &[]);

    let result = GraphMergeConflictClassifier::classify(
        &plan(base_id, left_id, right_id),
        snapshot(project_id, context_id, base_id, &base),
        snapshot(project_id, context_id, left_id, &changed),
        snapshot(project_id, context_id, right_id, &changed),
    )
    .expect("equivalent classification");

    assert!(matches!(
        result,
        GraphMergeClassification::Equivalent { .. }
    ));
}

#[test]
fn classifies_divergent_node_changes_and_edge_count_changes_as_conflicts() {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let base_id = CommitId::new();
    let left_id = CommitId::new();
    let right_id = CommitId::new();
    let base = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("tool:1", GraphNodeKind::Tool, "Search"),
        ],
        &[],
    );
    let left = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Left"),
            ("tool:1", GraphNodeKind::Tool, "Search"),
        ],
        &[("context:1", "tool:1", GraphEdgeKind::Uses)],
    );
    let right = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Right"),
            ("tool:1", GraphNodeKind::Tool, "Search"),
        ],
        &[
            ("context:1", "tool:1", GraphEdgeKind::Uses),
            ("context:1", "tool:1", GraphEdgeKind::Uses),
        ],
    );

    let result = GraphMergeConflictClassifier::classify(
        &plan(base_id, left_id, right_id),
        snapshot(project_id, context_id, base_id, &base),
        snapshot(project_id, context_id, left_id, &left),
        snapshot(project_id, context_id, right_id, &right),
    )
    .expect("conflict classification");

    let GraphMergeClassification::Conflict { conflicts } = result else {
        panic!("expected conflicts");
    };
    assert!(conflicts.contains(&GraphMergeConflict::Node {
        node_id: "context:1".to_owned()
    }));
    assert!(conflicts.contains(&GraphMergeConflict::Edge {
        source: "context:1".to_owned(),
        target: "tool:1".to_owned(),
        kind: GraphEdgeKind::Uses,
    }));
}

#[test]
fn rejects_non_three_way_and_scope_or_plan_drift() {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let other_context_id = ContextId::new();
    let base_id = CommitId::new();
    let left_id = CommitId::new();
    let right_id = CommitId::new();
    let base = graph(&[("context:1", GraphNodeKind::Context, "Context")], &[]);

    let base_ref = snapshot(project_id, context_id, base_id, &base);
    let left_ref = snapshot(project_id, context_id, left_id, &base);
    let right_ref = snapshot(project_id, context_id, right_id, &base);
    assert!(matches!(
        GraphMergeConflictClassifier::classify(
            &MergePlan::NoOp { commit_id: base_id },
            base_ref,
            left_ref,
            right_ref,
        ),
        Err(GraphMergeClassificationError::NotThreeWay)
    ));

    let wrong_left = snapshot(project_id, other_context_id, left_id, &base);
    assert!(matches!(
        GraphMergeConflictClassifier::classify(
            &plan(base_id, left_id, right_id),
            base_ref,
            wrong_left,
            right_ref,
        ),
        Err(GraphMergeClassificationError::ScopeMismatch {
            side: GraphMergeSnapshotSide::Left
        })
    ));

    let wrong_commit = snapshot(project_id, context_id, CommitId::new(), &base);
    assert!(matches!(
        GraphMergeConflictClassifier::classify(
            &plan(base_id, left_id, right_id),
            base_ref,
            wrong_commit,
            right_ref,
        ),
        Err(GraphMergeClassificationError::PlanIdentityMismatch {
            side: GraphMergeSnapshotSide::Left,
            ..
        })
    ));
}

#[test]
fn classifies_add_add_and_remove_modify_collisions_as_conflicts() {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let base_id = CommitId::new();
    let left_id = CommitId::new();
    let right_id = CommitId::new();
    let base = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("prompt:1", GraphNodeKind::Prompt, "Prompt"),
        ],
        &[],
    );
    let left = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("tool:1", GraphNodeKind::Tool, "Left tool"),
        ],
        &[],
    );
    let right = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("prompt:1", GraphNodeKind::Prompt, "Prompt revised"),
            ("tool:1", GraphNodeKind::Tool, "Right tool"),
        ],
        &[],
    );

    let result = GraphMergeConflictClassifier::classify(
        &plan(base_id, left_id, right_id),
        snapshot(project_id, context_id, base_id, &base),
        snapshot(project_id, context_id, left_id, &left),
        snapshot(project_id, context_id, right_id, &right),
    )
    .expect("collision classification");

    let GraphMergeClassification::Conflict { conflicts } = result else {
        panic!("expected add/add and remove/modify conflicts");
    };
    assert_eq!(
        conflicts,
        vec![
            GraphMergeConflict::Node {
                node_id: "prompt:1".to_owned(),
            },
            GraphMergeConflict::Node {
                node_id: "tool:1".to_owned(),
            },
        ]
    );
}

#[test]
fn preserves_deterministic_edge_identity_and_equivalence() {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let base_id = CommitId::new();
    let left_id = CommitId::new();
    let right_id = CommitId::new();
    let base = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("tool:1", GraphNodeKind::Tool, "Tool"),
        ],
        &[],
    );
    let revised = graph(
        &[
            ("context:1", GraphNodeKind::Context, "Context"),
            ("tool:1", GraphNodeKind::Tool, "Tool"),
        ],
        &[("context:1", "tool:1", GraphEdgeKind::Uses)],
    );

    let result = GraphMergeConflictClassifier::classify(
        &plan(base_id, left_id, right_id),
        snapshot(project_id, context_id, base_id, &base),
        snapshot(project_id, context_id, left_id, &revised),
        snapshot(project_id, context_id, right_id, &revised),
    )
    .expect("equivalent edge classification");

    assert_eq!(
        result,
        GraphMergeClassification::Equivalent {
            changes: vec![contextlab_diff_engine::GraphMergeChange::Edge {
                source: "context:1".to_owned(),
                target: "tool:1".to_owned(),
                kind: GraphEdgeKind::Uses,
            }]
        }
    );
}
