//! External V1 contract tests.

use contextlab_diff_engine::{
    BehaviorSnapshotV1, ContextDiffRequestV1, ContextDiffService, ContextDiffSnapshotV1,
    DiffContractVersion, EvaluationSnapshotV1, SemanticDocumentV1, SemanticSnapshotV1,
};
use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};

fn graph(nodes: &[(&str, GraphNodeKind, &str)]) -> ContextGraph {
    let mut graph = ContextGraph::new();

    for (id, kind, label) in nodes {
        graph
            .add_node(GraphNode::new(*id, *kind, *label).expect("valid graph node"))
            .expect("unique graph node");
    }

    graph
}

fn snapshot(graph: ContextGraph, documents: Vec<SemanticDocumentV1>) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(graph, documents).expect("valid semantic snapshot"),
        BehaviorSnapshotV1::new(Vec::new()).expect("valid empty behavior snapshot"),
        EvaluationSnapshotV1::new("suite:customer-support:v1", Vec::new())
            .expect("valid empty evaluation snapshot"),
    )
    .expect("valid diff snapshot")
}

#[test]
fn compares_semantic_documents_in_stable_order_and_delegates_graphs_to_graph_diff() {
    let original = snapshot(
        graph(&[
            ("context:support", GraphNodeKind::Context, "Support"),
            ("prompt:system", GraphNodeKind::Prompt, "System"),
        ]),
        vec![
            SemanticDocumentV1::new("prompt:system", "Answer concisely.")
                .expect("valid system prompt"),
            SemanticDocumentV1::new("memory:customer", "Customer is premium.")
                .expect("valid memory"),
        ],
    );
    let revised = snapshot(
        graph(&[
            ("context:support", GraphNodeKind::Context, "Support"),
            ("prompt:system", GraphNodeKind::Prompt, "System"),
            (
                "knowledge:refunds",
                GraphNodeKind::Knowledge,
                "Refund Policy",
            ),
        ]),
        vec![
            SemanticDocumentV1::new("knowledge:refunds", "Refunds require proof of purchase.")
                .expect("valid knowledge"),
            SemanticDocumentV1::new("prompt:system", "Answer with citations.")
                .expect("valid system prompt"),
        ],
    );

    let result = ContextDiffService::compare(
        ContextDiffRequestV1::new(original, revised).expect("valid comparison request"),
    )
    .expect("comparable snapshots");

    assert_eq!(result.contract_version(), DiffContractVersion::V1);
    assert_eq!(
        result
            .semantic()
            .graph_diff()
            .added_nodes()
            .iter()
            .map(|node| node.id().as_str())
            .collect::<Vec<_>>(),
        vec!["knowledge:refunds"]
    );
    assert_eq!(
        result
            .semantic()
            .document_changes()
            .iter()
            .map(|change| change.document_id().as_str())
            .collect::<Vec<_>>(),
        vec!["knowledge:refunds", "memory:customer", "prompt:system"]
    );
}
