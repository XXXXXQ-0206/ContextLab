//! Contract tests for the server-owned ordered graph review pair identity.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::{
    VersionedContextGraphDiffReviewError, VersionedContextGraphDiffReviewIdentityWitnessV1,
    VersionedContextGraphDiffReviewRequestV1, VersionedContextGraphDiffReviewService,
    VersionedContextGraphSnapshotV1, VersionedContextScopeV1,
};
use contextlab_graph::ContextGraph;
use contextlab_versioning::CommitId;
use serde_json::json;
use uuid::Uuid;

#[test]
fn projection_serializes_the_exact_ordered_pair_identity_witness() {
    let source_scope = scope(10, 20, 30);
    let target_scope = scope(10, 20, 31);
    let projection = VersionedContextGraphDiffReviewService::project(
        VersionedContextGraphDiffReviewRequestV1::new(
            snapshot(source_scope),
            snapshot(target_scope),
        )
        .expect("distinct graph review pair"),
    );

    let serialized = serde_json::to_value(&projection).expect("serialize graph review projection");
    assert_eq!(
        serialized["identity_witness"],
        json!({
            "project_id": source_scope.project_id().as_uuid().to_string(),
            "context_id": source_scope.context_id().as_uuid().to_string(),
            "baseline_commit_id": source_scope.commit_id().as_uuid().to_string(),
            "revised_commit_id": target_scope.commit_id().as_uuid().to_string(),
        })
    );
    assert_eq!(
        projection.identity_witness().source_commit_id(),
        source_scope.commit_id()
    );
    assert_eq!(
        projection.identity_witness().target_commit_id(),
        target_scope.commit_id()
    );
    assert_ne!(
        projection.identity_witness().source_commit_id(),
        projection.identity_witness().target_commit_id()
    );
}

#[test]
fn identity_witness_round_trip_is_strict_and_fail_closed() {
    let source_scope = scope(11, 21, 40);
    let target_scope = scope(11, 21, 41);
    let witness = VersionedContextGraphDiffReviewIdentityWitnessV1::new(source_scope, target_scope)
        .expect("valid ordered pair");
    let serialized = serde_json::to_value(witness).expect("serialize identity witness");
    let restored: VersionedContextGraphDiffReviewIdentityWitnessV1 =
        serde_json::from_value(serialized.clone()).expect("round-trip identity witness");
    assert_eq!(restored, witness);
    assert_eq!(restored.project_id(), source_scope.project_id());
    assert_eq!(restored.context_id(), source_scope.context_id());
    assert_eq!(restored.baseline_commit_id(), source_scope.commit_id());
    assert_eq!(restored.revised_commit_id(), target_scope.commit_id());

    let mut unknown = serialized.clone();
    unknown["unexpected"] = true.into();
    assert!(
        serde_json::from_value::<VersionedContextGraphDiffReviewIdentityWitnessV1>(unknown)
            .is_err()
    );

    let mut nil_project = serialized.clone();
    nil_project["project_id"] = Uuid::nil().to_string().into();
    assert!(
        serde_json::from_value::<VersionedContextGraphDiffReviewIdentityWitnessV1>(nil_project)
            .is_err()
    );

    let mismatched =
        VersionedContextGraphDiffReviewIdentityWitnessV1::new(source_scope, scope(12, 21, 41))
            .expect_err("cross-project pair must fail closed");
    assert!(matches!(
        mismatched,
        VersionedContextGraphDiffReviewError::MismatchedContextScope { .. }
    ));

    let identical =
        VersionedContextGraphDiffReviewIdentityWitnessV1::new(source_scope, source_scope)
            .expect_err("identical pair must fail closed");
    assert!(matches!(
        identical,
        VersionedContextGraphDiffReviewError::IdenticalVersionScopes { .. }
    ));
}

fn snapshot(scope: VersionedContextScopeV1) -> VersionedContextGraphSnapshotV1 {
    VersionedContextGraphSnapshotV1::new(scope, ContextGraph::new())
}

fn scope(project: u128, context: u128, commit: u128) -> VersionedContextScopeV1 {
    VersionedContextScopeV1::new(
        ProjectId::from_uuid(Uuid::from_u128(project)),
        ContextId::from_uuid(Uuid::from_u128(context)),
        CommitId::from_uuid(Uuid::from_u128(commit)),
    )
}
