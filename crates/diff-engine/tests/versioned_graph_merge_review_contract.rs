//! Contract tests for the private version-bound Context Graph merge review.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::{
    GraphMergeClassification, GraphMergeClassificationError, GraphMergeSnapshotSide,
    VersionedContextGraphMergeReviewError, VersionedContextGraphMergeReviewRequestV1,
    VersionedContextGraphMergeReviewService, VersionedContextGraphSnapshotV1,
    VersionedContextScopeV1,
};
use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
use contextlab_versioning::{CommitId, MergePlan};
use serde_json::json;
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn project_id() -> ProjectId {
    ProjectId::from_uuid(id(30_000))
}

fn context_id() -> ContextId {
    ContextId::from_uuid(id(30_001))
}

fn commit_id(value: u128) -> CommitId {
    CommitId::from_uuid(id(value))
}

fn graph(label: &str) -> ContextGraph {
    let mut graph = ContextGraph::new();
    graph
        .add_node(GraphNode::new("context:versioned", GraphNodeKind::Context, label).expect("node"))
        .expect("insert node");
    graph
}

fn snapshot(commit_id: CommitId, label: &str) -> VersionedContextGraphSnapshotV1 {
    VersionedContextGraphSnapshotV1::new(
        VersionedContextScopeV1::new(project_id(), context_id(), commit_id),
        graph(label),
    )
}

fn request() -> VersionedContextGraphMergeReviewRequestV1 {
    VersionedContextGraphMergeReviewRequestV1::new(
        MergePlan::ThreeWay {
            base: commit_id(30_010),
            left: commit_id(30_011),
            right: commit_id(30_012),
        },
        snapshot(commit_id(30_010), "base"),
        snapshot(commit_id(30_011), "left"),
        snapshot(commit_id(30_012), "right"),
    )
}

#[test]
fn projects_exact_version_scopes_through_the_existing_classifier() {
    let projection = VersionedContextGraphMergeReviewService::review(request()).expect("review");

    assert!(matches!(
        projection.classification(),
        GraphMergeClassification::Conflict { .. }
    ));
    assert_eq!(projection.base_scope().commit_id(), commit_id(30_010));
    assert_eq!(projection.left_scope().commit_id(), commit_id(30_011));
    assert_eq!(projection.right_scope().commit_id(), commit_id(30_012));
}

#[test]
fn serialized_request_is_v1_and_rejects_schema_or_shape_drift() {
    let value = serde_json::to_value(request()).expect("serialize request");
    assert_eq!(value["schema_version"], json!("v1"));
    assert!(
        serde_json::from_value::<VersionedContextGraphMergeReviewRequestV1>(value.clone()).is_ok()
    );

    let mut unsupported = value.clone();
    unsupported["schema_version"] = json!("v2");
    assert!(
        serde_json::from_value::<VersionedContextGraphMergeReviewRequestV1>(unsupported).is_err()
    );

    let mut unknown = value;
    unknown["unexpected"] = json!(true);
    assert!(serde_json::from_value::<VersionedContextGraphMergeReviewRequestV1>(unknown).is_err());
}

#[test]
fn fails_closed_for_plan_identity_and_cross_scope_drift() {
    let wrong_plan = VersionedContextGraphMergeReviewRequestV1::new(
        MergePlan::ThreeWay {
            base: commit_id(30_010),
            left: commit_id(30_011),
            right: commit_id(30_099),
        },
        snapshot(commit_id(30_010), "base"),
        snapshot(commit_id(30_011), "left"),
        snapshot(commit_id(30_012), "right"),
    );
    assert!(matches!(
        VersionedContextGraphMergeReviewService::review(wrong_plan),
        Err(
            VersionedContextGraphMergeReviewError::ClassificationFailed {
                source: GraphMergeClassificationError::PlanIdentityMismatch { .. }
            }
        )
    ));

    let other_context = VersionedContextGraphSnapshotV1::new(
        VersionedContextScopeV1::new(
            project_id(),
            ContextId::from_uuid(id(30_099)),
            commit_id(30_012),
        ),
        graph("right"),
    );
    let drifted = VersionedContextGraphMergeReviewRequestV1::new(
        MergePlan::ThreeWay {
            base: commit_id(30_010),
            left: commit_id(30_011),
            right: commit_id(30_012),
        },
        snapshot(commit_id(30_010), "base"),
        snapshot(commit_id(30_011), "left"),
        other_context,
    );
    assert!(matches!(
        VersionedContextGraphMergeReviewService::review(drifted),
        Err(
            VersionedContextGraphMergeReviewError::ClassificationFailed {
                source: GraphMergeClassificationError::ScopeMismatch { .. }
            }
        )
    ));
}

#[test]
fn fails_closed_for_duplicate_or_nil_snapshot_identity() {
    let duplicate = VersionedContextGraphMergeReviewRequestV1::new(
        MergePlan::ThreeWay {
            base: commit_id(30_010),
            left: commit_id(30_010),
            right: commit_id(30_012),
        },
        snapshot(commit_id(30_010), "base"),
        snapshot(commit_id(30_010), "left"),
        snapshot(commit_id(30_012), "right"),
    );
    assert!(matches!(
        VersionedContextGraphMergeReviewService::review(duplicate),
        Err(
            VersionedContextGraphMergeReviewError::DuplicateCommitIdentity {
                side: GraphMergeSnapshotSide::Left,
                ..
            }
        )
    ));

    let nil_scope = VersionedContextGraphMergeReviewRequestV1::new(
        MergePlan::ThreeWay {
            base: commit_id(30_010),
            left: commit_id(30_011),
            right: commit_id(30_012),
        },
        VersionedContextGraphSnapshotV1::new(
            VersionedContextScopeV1::new(
                ProjectId::from_uuid(Uuid::nil()),
                context_id(),
                commit_id(30_010),
            ),
            graph("base"),
        ),
        snapshot(commit_id(30_011), "left"),
        snapshot(commit_id(30_012), "right"),
    );
    assert!(matches!(
        VersionedContextGraphMergeReviewService::review(nil_scope),
        Err(
            VersionedContextGraphMergeReviewError::InvalidSnapshotScope {
                side: GraphMergeSnapshotSide::Base
            }
        )
    ));
}
