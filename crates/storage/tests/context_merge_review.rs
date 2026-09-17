//! Contract tests for the private persisted three-way Context Graph review.

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
use contextlab_storage::{
    CommitGraphSnapshot, CommitGraphSnapshotRepository, CommitGraphSnapshotScope,
    ContextCommitRecord, ContextGraphProjection, ContextMergeInputScope, ContextMergeReviewWitness,
    ContextMergeReviewWitnessRepository, ContextMergeReviewWitnessRepositoryError,
    ContextMergeTipScope, ContextRecord, InMemoryContextGraphRepository,
    PersistedContextGraphMergeReviewError, PersistedContextGraphMergeReviewService,
    PersistedContextGraphMergeReviewSide, ProjectRecord, StorageRepositoryError,
};
use contextlab_versioning::{CommitId, MergePlan};
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn project_id() -> ProjectId {
    ProjectId::from_uuid(id(20_000))
}

fn context_id() -> ContextId {
    ContextId::from_uuid(id(20_001))
}

fn commit_id(value: u128) -> CommitId {
    CommitId::from_uuid(id(value))
}

fn timestamp(seconds: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 30, 0, 0, seconds)
        .single()
        .expect("timestamp")
}

fn graph(label: &str) -> ContextGraph {
    let mut graph = ContextGraph::new();
    graph
        .add_node(GraphNode::new("context:merge", GraphNodeKind::Context, label).expect("node"))
        .expect("insert node");
    graph
}

fn projection() -> ContextGraphProjection {
    let context_id = context_id();
    let commits = [20_010_u128, 20_011, 20_012]
        .into_iter()
        .map(|value| ContextCommitRecord {
            id: commit_id(value).to_string(),
            context_id: context_id.to_string(),
            branch_name: "merge-review".to_owned(),
            message: "merge review fixture".to_owned(),
            parent_commit_ids: if value == 20_010 {
                Vec::new()
            } else {
                vec![commit_id(20_010).to_string()]
            },
            changes: json!([]),
            change_count: 0,
            authored_at: timestamp(1),
            created_at: timestamp(1),
        })
        .collect();

    ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: project_id().to_string(),
            workspace_id: "merge-workspace".to_owned(),
            name: "Merge review project".to_owned(),
            slug: "merge-review".to_owned(),
            created_at: timestamp(0),
        }],
        contexts: vec![ContextRecord {
            id: context_id.to_string(),
            project_id: project_id().to_string(),
            experiment_id: None,
            name: "Merge review Context".to_owned(),
            description: None,
            created_at: timestamp(0),
        }],
        commits,
        ..ContextGraphProjection::default()
    }
}

fn snapshot(commit_id: CommitId, graph: ContextGraph) -> CommitGraphSnapshot {
    CommitGraphSnapshot::new(
        CommitGraphSnapshotScope::new(project_id(), context_id(), commit_id),
        graph,
        timestamp(2),
        1,
    )
    .expect("snapshot")
}

fn scope() -> ContextMergeInputScope {
    ContextMergeInputScope::new(
        project_id(),
        context_id(),
        commit_id(20_010),
        commit_id(20_011),
        commit_id(20_012),
    )
    .expect("scope")
}

fn make_repository(include_right: bool) -> InMemoryContextGraphRepository {
    let base_id = commit_id(20_010);
    let left_id = commit_id(20_011);
    let right_id = commit_id(20_012);
    let mut snapshots = vec![
        snapshot(base_id, graph("Context")),
        snapshot(left_id, graph("v2")),
    ];
    if include_right {
        snapshots.push(snapshot(right_id, graph("v2")));
    }
    InMemoryContextGraphRepository::with_commit_graph_snapshots(projection(), snapshots)
        .expect("repository")
}

#[tokio::test]
async fn reads_three_exact_snapshots_and_delegates_equivalent_review() {
    let repository = make_repository(true);
    let service = PersistedContextGraphMergeReviewService::new(&repository);

    let result = service
        .review(
            scope(),
            &MergePlan::ThreeWay {
                base: commit_id(20_010),
                left: commit_id(20_011),
                right: commit_id(20_012),
            },
        )
        .await
        .expect("review");

    assert!(matches!(
        result,
        contextlab_diff_engine::GraphMergeClassification::Equivalent { .. }
    ));
}

#[tokio::test]
async fn server_owned_review_resolves_three_way_plan_from_exact_context_dag() {
    let repository = make_repository(true);
    let service = PersistedContextGraphMergeReviewService::new(&repository);
    let tip_scope = ContextMergeTipScope::new(
        project_id(),
        context_id(),
        commit_id(20_011),
        commit_id(20_012),
    )
    .expect("tip scope");

    let result = service
        .review_server_owned(tip_scope)
        .await
        .expect("server-owned review");

    assert!(matches!(
        result,
        contextlab_diff_engine::GraphMergeClassification::Equivalent { .. }
    ));
}

#[derive(Clone)]
struct CountingServerOwnedWitnessRepository {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl ContextMergeReviewWitnessRepository for CountingServerOwnedWitnessRepository {
    async fn load_context_merge_review_witness(
        &self,
        _scope: ContextMergeTipScope,
    ) -> Result<ContextMergeReviewWitness, ContextMergeReviewWitnessRepositoryError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        ContextMergeReviewWitness::new(
            MergePlan::ThreeWay {
                base: commit_id(20_010),
                left: commit_id(20_011),
                right: commit_id(20_012),
            },
            snapshot(commit_id(20_010), graph("Context")),
            snapshot(commit_id(20_011), graph("v2")),
            snapshot(commit_id(20_012), graph("v2")),
        )
        .map_err(ContextMergeReviewWitnessRepositoryError::InvalidWitness)
    }
}

#[tokio::test]
async fn server_owned_review_uses_one_atomic_plan_snapshot_witness_port() {
    let calls = Arc::new(AtomicUsize::new(0));
    let repository = CountingServerOwnedWitnessRepository {
        calls: Arc::clone(&calls),
    };
    let service = PersistedContextGraphMergeReviewService::new(&repository);
    let tip_scope = ContextMergeTipScope::new(
        project_id(),
        context_id(),
        commit_id(20_011),
        commit_id(20_012),
    )
    .expect("tip scope");

    let result = service
        .review_server_owned(tip_scope)
        .await
        .expect("atomic witness review");

    assert!(matches!(
        result,
        contextlab_diff_engine::GraphMergeClassification::Equivalent { .. }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn server_owned_review_rejects_a_witness_outside_the_requested_tip_scope() {
    let repository = CountingServerOwnedWitnessRepository {
        calls: Arc::new(AtomicUsize::new(0)),
    };
    let service = PersistedContextGraphMergeReviewService::new(&repository);
    let requested_scope = ContextMergeTipScope::new(
        ProjectId::from_uuid(id(20_099)),
        context_id(),
        commit_id(20_011),
        commit_id(20_012),
    )
    .expect("tip scope");

    let error = service
        .review_server_owned(requested_scope)
        .await
        .expect_err("out-of-scope witness");

    assert!(matches!(
        error,
        PersistedContextGraphMergeReviewError::RequestedTipScopeMismatch
    ));
}

#[test]
fn merge_review_witness_rejects_non_three_way_plans() {
    let error = ContextMergeReviewWitness::new(
        MergePlan::NoOp {
            commit_id: commit_id(20_010),
        },
        snapshot(commit_id(20_010), graph("Context")),
        snapshot(commit_id(20_011), graph("v2")),
        snapshot(commit_id(20_012), graph("v2")),
    )
    .expect_err("non-three-way witness");

    assert!(matches!(
        error,
        contextlab_storage::ContextMergeReviewWitnessError::NonThreeWay
    ));
}

#[test]
fn merge_review_witness_rejects_snapshot_scope_drift() {
    let error = ContextMergeReviewWitness::new(
        MergePlan::ThreeWay {
            base: commit_id(20_010),
            left: commit_id(20_011),
            right: commit_id(20_012),
        },
        snapshot(commit_id(20_010), graph("Context")),
        snapshot(commit_id(20_099), graph("drifted")),
        snapshot(commit_id(20_012), graph("v2")),
    )
    .expect_err("drifted witness");

    assert!(matches!(
        error,
        contextlab_storage::ContextMergeReviewWitnessError::SnapshotScopeMismatch {
            side: PersistedContextGraphMergeReviewSide::Left,
            ..
        }
    ));
}

#[tokio::test]
async fn server_owned_projection_preserves_resolved_plan_and_exact_scopes() {
    let repository = make_repository(true);
    let service = PersistedContextGraphMergeReviewService::new(&repository);
    let tip_scope = ContextMergeTipScope::new(
        project_id(),
        context_id(),
        commit_id(20_011),
        commit_id(20_012),
    )
    .expect("tip scope");

    let projection = service
        .review_server_owned_projection(tip_scope)
        .await
        .expect("server-owned projection");
    let expected_plan = MergePlan::ThreeWay {
        base: commit_id(20_010),
        left: commit_id(20_011),
        right: commit_id(20_012),
    };

    assert_eq!(projection.plan(), &expected_plan);
    assert_eq!(projection.base_scope().project_id(), project_id());
    assert_eq!(projection.base_scope().context_id(), context_id());
    assert_eq!(projection.base_scope().commit_id(), commit_id(20_010));
    assert_eq!(projection.left_scope().commit_id(), commit_id(20_011));
    assert_eq!(projection.right_scope().commit_id(), commit_id(20_012));

    let encoded = serde_json::to_value(&projection).expect("projection JSON");
    assert_eq!(
        encoded["plan"],
        serde_json::to_value(expected_plan).expect("plan JSON")
    );
    assert_eq!(
        encoded["base_scope"]["commit_id"],
        commit_id(20_010).to_string()
    );
}

#[tokio::test]
async fn server_owned_review_rejects_unknown_tip_before_snapshot_read() {
    let repository = make_repository(true);
    let service = PersistedContextGraphMergeReviewService::new(&repository);
    let tip_scope = ContextMergeTipScope::new(
        project_id(),
        context_id(),
        commit_id(20_011),
        commit_id(20_099),
    )
    .expect("tip scope");

    let error = service
        .review_server_owned(tip_scope)
        .await
        .expect_err("unknown tip");

    assert!(matches!(
        error,
        PersistedContextGraphMergeReviewError::PlanResolutionFailed {
            source: contextlab_versioning::MergePlanError::UnknownCommit { .. }
        }
    ));
}

#[tokio::test]
async fn server_owned_review_rejects_fast_forward_as_non_three_way() {
    let repository = make_repository(true);
    let service = PersistedContextGraphMergeReviewService::new(&repository);
    let tip_scope = ContextMergeTipScope::new(
        project_id(),
        context_id(),
        commit_id(20_011),
        commit_id(20_010),
    )
    .expect("tip scope");

    let error = service
        .review_server_owned(tip_scope)
        .await
        .expect_err("fast-forward");

    assert!(matches!(
        error,
        PersistedContextGraphMergeReviewError::ClassificationFailed {
            source: contextlab_diff_engine::GraphMergeClassificationError::NotThreeWay
        }
    ));
}

#[tokio::test]
async fn batch_read_preserves_base_left_right_order() {
    let repository = make_repository(true);

    let (base, left, right) = repository
        .get_commit_graph_snapshot_batch(scope())
        .await
        .expect("batch snapshots");

    assert_eq!(base.commit_id(), commit_id(20_010));
    assert_eq!(left.commit_id(), commit_id(20_011));
    assert_eq!(right.commit_id(), commit_id(20_012));
}

#[tokio::test]
async fn fails_closed_for_missing_right_snapshot_and_non_three_way_plan() {
    let repository = make_repository(false);
    let service = PersistedContextGraphMergeReviewService::new(&repository);

    let error = service
        .review(
            scope(),
            &MergePlan::ThreeWay {
                base: commit_id(20_010),
                left: commit_id(20_011),
                right: commit_id(20_012),
            },
        )
        .await
        .expect_err("missing right snapshot");
    assert!(matches!(
        error,
        PersistedContextGraphMergeReviewError::SnapshotUnavailable {
            side: PersistedContextGraphMergeReviewSide::Right,
            ..
        }
    ));

    let repository = make_repository(true);
    let service = PersistedContextGraphMergeReviewService::new(&repository);
    let error = service
        .review(
            scope(),
            &MergePlan::NoOp {
                commit_id: commit_id(20_010),
            },
        )
        .await
        .expect_err("non-three-way plan");
    assert!(matches!(
        error,
        PersistedContextGraphMergeReviewError::ClassificationFailed {
            source: contextlab_diff_engine::GraphMergeClassificationError::NotThreeWay
        }
    ));
}

#[tokio::test]
async fn batch_read_fails_closed_for_missing_right_snapshot() {
    let repository = make_repository(false);

    let error = repository
        .get_commit_graph_snapshot_batch(scope())
        .await
        .expect_err("missing right snapshot");

    assert!(matches!(
        error,
        StorageRepositoryError::ScopeUnavailable { scope: missing }
            if missing.ends_with(&format!("/commit:{}", commit_id(20_012)))
    ));
}

#[test]
fn rejects_duplicate_commit_scope_inputs() {
    let error = ContextMergeInputScope::new(
        project_id(),
        context_id(),
        commit_id(20_010),
        commit_id(20_010),
        commit_id(20_012),
    )
    .expect_err("duplicate commit");
    assert!(matches!(
        error,
        contextlab_storage::ContextMergeInputScopeError::DuplicateCommit {
            side: PersistedContextGraphMergeReviewSide::Left,
            ..
        }
    ));
}

#[test]
fn deserialization_revalidates_duplicate_commit_scope_inputs() {
    let mut encoded = serde_json::to_value(scope()).expect("serialize scope");
    encoded["left_commit_id"] = encoded["base_commit_id"].clone();

    let error = serde_json::from_value::<ContextMergeInputScope>(encoded)
        .expect_err("deserialization must preserve scope invariants");
    assert!(error.to_string().contains("repeats"));
}

#[derive(Clone)]
struct DriftRepository {
    snapshot: CommitGraphSnapshot,
}

#[derive(Clone)]
struct CountingBatchRepository {
    snapshots: [CommitGraphSnapshot; 3],
    batch_calls: Arc<AtomicUsize>,
    single_calls: Arc<AtomicUsize>,
}

#[async_trait]
impl CommitGraphSnapshotRepository for CountingBatchRepository {
    async fn project_id_for_context(
        &self,
        _context_id: ContextId,
    ) -> Result<ProjectId, StorageRepositoryError> {
        Ok(project_id())
    }

    async fn scope_for_context_commit(
        &self,
        _context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError> {
        Ok(CommitGraphSnapshotScope::new(
            project_id(),
            context_id(),
            commit_id,
        ))
    }

    async fn get_commit_graph_snapshot(
        &self,
        _scope: CommitGraphSnapshotScope,
    ) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
        self.single_calls.fetch_add(1, Ordering::SeqCst);
        panic!("merge review must use the batch read port");
    }

    async fn get_commit_graph_snapshot_batch(
        &self,
        _scope: ContextMergeInputScope,
    ) -> Result<
        (
            CommitGraphSnapshot,
            CommitGraphSnapshot,
            CommitGraphSnapshot,
        ),
        StorageRepositoryError,
    > {
        self.batch_calls.fetch_add(1, Ordering::SeqCst);
        let [base, left, right] = self.snapshots.clone();
        Ok((base, left, right))
    }
}

#[tokio::test]
async fn review_invokes_the_batch_read_port_exactly_once() {
    let batch_calls = Arc::new(AtomicUsize::new(0));
    let single_calls = Arc::new(AtomicUsize::new(0));
    let repository = CountingBatchRepository {
        snapshots: [
            snapshot(commit_id(20_010), graph("Context")),
            snapshot(commit_id(20_011), graph("v2")),
            snapshot(commit_id(20_012), graph("v2")),
        ],
        batch_calls: Arc::clone(&batch_calls),
        single_calls: Arc::clone(&single_calls),
    };
    let service = PersistedContextGraphMergeReviewService::new(&repository);

    service
        .review(
            scope(),
            &MergePlan::ThreeWay {
                base: commit_id(20_010),
                left: commit_id(20_011),
                right: commit_id(20_012),
            },
        )
        .await
        .expect("review");

    assert_eq!(batch_calls.load(Ordering::SeqCst), 1);
    assert_eq!(single_calls.load(Ordering::SeqCst), 0);
}

#[async_trait]
impl CommitGraphSnapshotRepository for DriftRepository {
    async fn project_id_for_context(
        &self,
        _context_id: ContextId,
    ) -> Result<ProjectId, StorageRepositoryError> {
        Ok(self.snapshot.project_id())
    }

    async fn scope_for_context_commit(
        &self,
        _context_id: ContextId,
        _commit_id: CommitId,
    ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError> {
        Ok(self.snapshot.scope())
    }

    async fn get_commit_graph_snapshot(
        &self,
        _scope: CommitGraphSnapshotScope,
    ) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
        Ok(Some(self.snapshot.clone()))
    }
}

#[tokio::test]
async fn rejects_repository_scope_drift_before_classification() {
    let drifted_scope = CommitGraphSnapshotScope::new(
        ProjectId::from_uuid(id(20_100)),
        context_id(),
        commit_id(20_010),
    );
    let repository = DriftRepository {
        snapshot: CommitGraphSnapshot::new(drifted_scope, graph("drifted"), timestamp(2), 1)
            .expect("snapshot"),
    };
    let service = PersistedContextGraphMergeReviewService::new(&repository);

    let error = service
        .review(
            scope(),
            &MergePlan::ThreeWay {
                base: commit_id(20_010),
                left: commit_id(20_011),
                right: commit_id(20_012),
            },
        )
        .await
        .expect_err("scope drift");
    assert!(matches!(
        error,
        PersistedContextGraphMergeReviewError::StoredScopeMismatch {
            side: PersistedContextGraphMergeReviewSide::Base,
            ..
        }
    ));
}
