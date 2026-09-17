//! Static and compile-only contracts for the private PostgreSQL graph-review witness.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_storage::{
    CommitGraphSnapshotScope, ContextGraphBranchHeadReviewWitnessRepository,
    ContextGraphReviewWitnessRepository, PostgresContextGraphRepository,
};
use contextlab_versioning::{BranchName, CommitId};
use uuid::Uuid;

const POSTGRES_ADAPTER: &str = include_str!("../src/postgres.rs");

#[test]
fn witness_queries_share_one_transaction_and_two_exact_snapshot_reads() {
    let normalized = POSTGRES_ADAPTER.split_whitespace().collect::<String>();
    assert!(normalized.contains(
        "letmuttransaction=begin_consistent_read_transaction(&self.pool).await?;lethistory=load_context_commit_history_in_transaction("
    ));
    let implementation = POSTGRES_ADAPTER
        .split("impl ContextGraphReviewWitnessRepository")
        .nth(1)
        .expect("exact-pair witness implementation")
        .split("impl ContextGraphBranchHeadReviewWitnessRepository")
        .next()
        .expect("exact-pair witness implementation section")
        .split("async fn load_context_commit_history_in_transaction")
        .next()
        .expect("exact-pair witness implementation body");
    assert_eq!(
        implementation
            .matches("load_commit_graph_snapshot_in_transaction(&mut transaction")
            .count(),
        2,
        "the witness must read source and target through the same transaction"
    );
    assert!(POSTGRES_ADAPTER.contains(
        "async fn load_context_commit_history_in_transaction(\n    transaction: &mut Transaction<'_, Postgres>"
    ));
}

#[test]
fn branch_head_witness_selects_head_and_snapshots_inside_one_transaction() {
    let implementation = POSTGRES_ADAPTER
        .split("impl ContextGraphBranchHeadReviewWitnessRepository")
        .nth(1)
        .expect("branch-head witness implementation");
    let implementation = implementation
        .split("async fn load_context_commit_history_in_transaction")
        .next()
        .expect("branch-head witness implementation body");
    let normalized = implementation.split_whitespace().collect::<String>();

    assert!(normalized.contains(
        "letmuttransaction=begin_consistent_read_transaction(&self.pool).await?;lethistory=load_context_commit_history_in_transaction("
    ));
    assert!(normalized.contains("select_branch_head(&history,&branch)?"));
    assert_eq!(
        implementation
            .matches("load_commit_graph_snapshot_in_transaction(&mut transaction")
            .count(),
        2,
        "branch-head witness must read source and target through the same transaction"
    );
}

#[tokio::test]
async fn postgres_witness_repository_is_lazy_and_implements_the_contract() {
    let repository = PostgresContextGraphRepository::connect_lazy(
        "postgres://contextlab:contextlab@localhost/contextlab",
    )
    .expect("valid lazy PostgreSQL configuration");
    fn assert_repository<T: ContextGraphReviewWitnessRepository>() {}
    assert_repository::<PostgresContextGraphRepository>();
    fn assert_branch_repository<T: ContextGraphBranchHeadReviewWitnessRepository>() {}
    assert_branch_repository::<PostgresContextGraphRepository>();

    let source = CommitGraphSnapshotScope::new(
        ProjectId::from_uuid(Uuid::from_u128(800)),
        ContextId::from_uuid(Uuid::from_u128(801)),
        CommitId::from_uuid(Uuid::from_u128(802)),
    );
    let target = CommitGraphSnapshotScope::new(
        source.project_id(),
        source.context_id(),
        CommitId::from_uuid(Uuid::from_u128(803)),
    );
    let _ = (repository, source, target);
    let _ = BranchName::default();
}

#[tokio::test]
#[ignore = "requires an empty disposable PostgreSQL database; live witness transaction evidence is deferred"]
async fn postgres_witness_runtime_requires_a_disposable_database() {
    let repository = PostgresContextGraphRepository::connect_lazy(
        "postgres://contextlab:contextlab@localhost/contextlab",
    )
    .expect("valid lazy PostgreSQL configuration");
    let scope = CommitGraphSnapshotScope::new(
        ProjectId::from_uuid(Uuid::from_u128(810)),
        ContextId::from_uuid(Uuid::from_u128(811)),
        CommitId::from_uuid(Uuid::from_u128(812)),
    );
    let _ = repository
        .read_context_graph_review_witness(
            scope,
            CommitGraphSnapshotScope::new(
                scope.project_id(),
                scope.context_id(),
                CommitId::from_uuid(Uuid::from_u128(813)),
            ),
        )
        .await;
}
