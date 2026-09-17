//! Contract tests for the local version-bound diff review projection.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::{
    BehaviorObservationV1, BehaviorOutcomeV1, BehaviorSnapshotV1, ContextDiffError,
    ContextDiffSnapshotV1, DiffReviewProjectionSchemaVersion, EvaluationMetricObservationV1,
    EvaluationSnapshotV1, SemanticDocumentV1, SemanticSnapshotV1, VersionedContextDiffReviewError,
    VersionedContextDiffReviewRequestV1, VersionedContextDiffReviewService,
    VersionedContextScopeV1,
};
use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
use contextlab_versioning::CommitId;
use uuid::Uuid;

#[test]
fn projects_a_complete_diff_for_an_exact_ordered_version_pair() {
    let source_version_id = commit_id(1);
    let target_version_id = commit_id(2);
    let review = VersionedContextDiffReviewService::project(
        VersionedContextDiffReviewRequestV1::new(
            scope(source_version_id),
            scope(target_version_id),
            snapshot(
                vec![
                    SemanticDocumentV1::new("prompt:z", "Removed prompt").expect("prompt z"),
                    SemanticDocumentV1::new("prompt:a", "Old prompt").expect("prompt a"),
                ],
                vec![
                    behavior(
                        "case:z",
                        "sha256:z",
                        BehaviorOutcomeV1::succeeded("removed"),
                    ),
                    behavior("case:a", "sha256:a", BehaviorOutcomeV1::succeeded("old")),
                ],
                vec![metric("latency_ms", 120.0), metric("accuracy", 0.80)],
            ),
            snapshot(
                vec![
                    SemanticDocumentV1::new("prompt:b", "Added prompt").expect("prompt b"),
                    SemanticDocumentV1::new("prompt:a", "New prompt").expect("prompt a"),
                ],
                vec![
                    behavior("case:b", "sha256:b", BehaviorOutcomeV1::succeeded("added")),
                    behavior(
                        "case:z",
                        "sha256:z",
                        BehaviorOutcomeV1::failed("provider_timeout").expect("failed outcome"),
                    ),
                ],
                vec![metric("cost_usd", 0.25), metric("accuracy", 0.90)],
            ),
        )
        .expect("distinct version pair"),
    )
    .expect("comparable version-bound snapshots");

    assert_eq!(
        review.schema_version(),
        DiffReviewProjectionSchemaVersion::V1
    );
    assert_eq!(review.source_version_id(), source_version_id);
    assert_eq!(review.target_version_id(), target_version_id);
    assert_eq!(review.source_scope(), scope(source_version_id));
    assert_eq!(review.target_scope(), scope(target_version_id));
    assert_eq!(
        review
            .diff()
            .semantic()
            .document_changes()
            .iter()
            .map(|change| change.document_id().as_str())
            .collect::<Vec<_>>(),
        vec!["prompt:a", "prompt:b", "prompt:z"]
    );
    assert_eq!(
        review
            .diff()
            .behavior()
            .case_changes()
            .iter()
            .map(|change| change.case_id().as_str())
            .collect::<Vec<_>>(),
        vec!["case:a", "case:b", "case:z"]
    );
    assert_eq!(
        review
            .diff()
            .evaluation()
            .metric_changes()
            .iter()
            .map(|change| change.metric_id().as_str())
            .collect::<Vec<_>>(),
        vec!["accuracy", "cost_usd", "latency_ms"]
    );

    let serialized = serde_json::to_value(&review).expect("serialize review projection");
    assert_eq!(serialized["schema_version"], "v1");
    assert_eq!(
        serialized["source_scope"]["commit_id"],
        source_version_id.to_string()
    );
    assert_eq!(
        serialized["target_scope"]["commit_id"],
        target_version_id.to_string()
    );
}

#[test]
fn rejects_a_review_of_the_same_exact_version() {
    let version_id = commit_id(3);
    let error = VersionedContextDiffReviewRequestV1::new(
        scope(version_id),
        scope(version_id),
        empty_snapshot("suite:review:v1"),
        empty_snapshot("suite:review:v1"),
    )
    .expect_err("a review requires two distinct versions");

    assert!(matches!(
        error,
        VersionedContextDiffReviewError::IdenticalVersionScopes { scope: actual }
            if actual == scope(version_id)
    ));
}

#[test]
fn preserves_exact_versions_when_the_underlying_evaluation_diff_fails_closed() {
    let source_version_id = commit_id(4);
    let target_version_id = commit_id(5);
    let error = VersionedContextDiffReviewService::project(
        VersionedContextDiffReviewRequestV1::new(
            scope(source_version_id),
            scope(target_version_id),
            empty_snapshot("suite:review:v1"),
            empty_snapshot("suite:review:v2"),
        )
        .expect("distinct version pair"),
    )
    .expect_err("incomparable evaluation evidence must return no projection");

    assert!(matches!(
        error,
        VersionedContextDiffReviewError::ContextDiffFailed {
            source_version_id: actual_source,
            target_version_id: actual_target,
            source: ContextDiffError::EvaluationComparabilityMismatch { .. },
        } if actual_source == source_version_id && actual_target == target_version_id
    ));
}

#[test]
fn rejects_unknown_or_invalid_serialized_review_requests() {
    let request = VersionedContextDiffReviewRequestV1::new(
        scope(commit_id(6)),
        scope(commit_id(7)),
        empty_snapshot("suite:review:v1"),
        empty_snapshot("suite:review:v1"),
    )
    .expect("distinct version pair");
    let serialized = serde_json::to_value(&request).expect("serialize request");
    let restored: VersionedContextDiffReviewRequestV1 =
        serde_json::from_value(serialized.clone()).expect("round-trip request");
    assert_eq!(restored, request);

    let mut unknown = serialized.clone();
    unknown["unexpected"] = true.into();
    assert!(serde_json::from_value::<VersionedContextDiffReviewRequestV1>(unknown).is_err());

    let mut unsupported_schema = serialized.clone();
    unsupported_schema["schema_version"] = "v2".into();
    assert!(
        serde_json::from_value::<VersionedContextDiffReviewRequestV1>(unsupported_schema).is_err()
    );

    let mut identical_versions = serialized;
    identical_versions["target_scope"] = identical_versions["source_scope"].clone();
    assert!(
        serde_json::from_value::<VersionedContextDiffReviewRequestV1>(identical_versions).is_err()
    );
}

#[test]
fn rejects_a_review_that_crosses_projects_or_contexts() {
    let baseline = scope(commit_id(8));
    let different_project =
        VersionedContextScopeV1::new(ProjectId::new(), baseline.context_id(), commit_id(9));
    let error = VersionedContextDiffReviewRequestV1::new(
        baseline,
        different_project,
        empty_snapshot("suite:review:v1"),
        empty_snapshot("suite:review:v1"),
    )
    .expect_err("cross-project review must fail closed");

    assert!(matches!(
        error,
        VersionedContextDiffReviewError::MismatchedContextScope { .. }
    ));
}

#[test]
fn preserves_the_typed_scope_when_a_request_is_replayed() {
    let source_scope = scope(commit_id(10));
    let target_scope = scope(commit_id(11));
    let request = VersionedContextDiffReviewRequestV1::new(
        source_scope,
        target_scope,
        empty_snapshot("suite:review:v1"),
        empty_snapshot("suite:review:v1"),
    )
    .expect("scoped request");
    let serialized = serde_json::to_value(&request).expect("serialize scoped request");
    let restored: VersionedContextDiffReviewRequestV1 =
        serde_json::from_value(serialized).expect("replay scoped request");

    assert_eq!(restored, request);
    assert_eq!(restored.source_scope(), source_scope);
    assert_eq!(restored.target_scope(), target_scope);
}

#[test]
fn delegates_non_empty_graph_review_to_the_unified_graph_diff() {
    let source_graph = graph(&[("context:support", GraphNodeKind::Context, "Support")]);
    let target_graph = graph(&[
        ("context:support", GraphNodeKind::Context, "Support"),
        ("prompt:policy", GraphNodeKind::Prompt, "Policy"),
    ]);
    let review = VersionedContextDiffReviewService::project(
        VersionedContextDiffReviewRequestV1::new(
            scope(commit_id(12)),
            scope(commit_id(13)),
            snapshot_with_graph(source_graph, Vec::new(), Vec::new(), Vec::new()),
            snapshot_with_graph(target_graph, Vec::new(), Vec::new(), Vec::new()),
        )
        .expect("scoped graph request"),
    )
    .expect("review delegates to the unified diff");

    assert_eq!(
        review
            .diff()
            .semantic()
            .graph_diff()
            .added_nodes()
            .iter()
            .map(|node| node.id().as_str())
            .collect::<Vec<_>>(),
        vec!["prompt:policy"]
    );
}

fn snapshot(
    documents: Vec<SemanticDocumentV1>,
    behavior_cases: Vec<BehaviorObservationV1>,
    metrics: Vec<EvaluationMetricObservationV1>,
) -> ContextDiffSnapshotV1 {
    snapshot_with_graph(ContextGraph::new(), documents, behavior_cases, metrics)
}

fn snapshot_with_graph(
    graph: ContextGraph,
    documents: Vec<SemanticDocumentV1>,
    behavior_cases: Vec<BehaviorObservationV1>,
    metrics: Vec<EvaluationMetricObservationV1>,
) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(graph, documents).expect("semantic snapshot"),
        BehaviorSnapshotV1::new(behavior_cases).expect("behavior snapshot"),
        EvaluationSnapshotV1::new("suite:review:v1", metrics).expect("evaluation snapshot"),
    )
    .expect("complete snapshot")
}

fn graph(nodes: &[(&str, GraphNodeKind, &str)]) -> ContextGraph {
    let mut graph = ContextGraph::new();
    for (id, kind, label) in nodes {
        graph
            .add_node(GraphNode::new(*id, *kind, *label).expect("graph node"))
            .expect("unique graph node");
    }
    graph
}

fn empty_snapshot(fingerprint: &str) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(ContextGraph::new(), Vec::new()).expect("semantic snapshot"),
        BehaviorSnapshotV1::new(Vec::new()).expect("behavior snapshot"),
        EvaluationSnapshotV1::new(fingerprint, Vec::new()).expect("evaluation snapshot"),
    )
    .expect("complete snapshot")
}

fn behavior(
    case_id: &str,
    input_fingerprint: &str,
    outcome: BehaviorOutcomeV1,
) -> BehaviorObservationV1 {
    BehaviorObservationV1::new(case_id, input_fingerprint, outcome).expect("behavior case")
}

fn metric(metric_id: &str, value: f64) -> EvaluationMetricObservationV1 {
    EvaluationMetricObservationV1::new(metric_id, value, 3).expect("evaluation metric")
}

fn commit_id(value: u128) -> CommitId {
    CommitId::from_uuid(Uuid::from_u128(value))
}

fn scope(commit_id: CommitId) -> VersionedContextScopeV1 {
    VersionedContextScopeV1::new(
        ProjectId::from_uuid(Uuid::from_u128(100)),
        ContextId::from_uuid(Uuid::from_u128(101)),
        commit_id,
    )
}
