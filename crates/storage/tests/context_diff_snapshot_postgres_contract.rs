//! Compile-only and ignored runtime contracts for the PostgreSQL adapter.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::VersionedContextScopeV1;
use contextlab_storage::{
    ContextDiffSnapshotV1PairRepository, ContextDiffSnapshotV1Repository,
    PostgresContextGraphRepository, StorageRepositoryError,
};
use contextlab_versioning::CommitId;
use uuid::Uuid;

#[tokio::test]
async fn postgres_adapter_is_available_without_eager_runtime_connection() {
    let repository = PostgresContextGraphRepository::connect_lazy(
        "postgres://contextlab:contextlab@localhost/contextlab",
    )
    .expect("valid lazy PostgreSQL configuration");
    fn assert_repository<T: ContextDiffSnapshotV1Repository>() {}
    fn assert_pair_repository<T: ContextDiffSnapshotV1PairRepository>() {}
    assert_repository::<PostgresContextGraphRepository>();
    assert_pair_repository::<PostgresContextGraphRepository>();
    let _ = repository;
}

#[tokio::test]
#[ignore = "requires an empty disposable PostgreSQL database and migration 0023"]
async fn postgres_exact_scope_replay_conflict_contract_requires_runtime() {
    let _scope = VersionedContextScopeV1::new(
        ProjectId::from_uuid(Uuid::from_u128(700)),
        ContextId::from_uuid(Uuid::from_u128(701)),
        CommitId::from_uuid(Uuid::from_u128(702)),
    );
    let _ = std::mem::size_of::<StorageRepositoryError>();
}

#[tokio::test]
#[ignore = "requires an empty disposable PostgreSQL database; pair read must observe one REPEATABLE READ READ ONLY transaction"]
async fn postgres_pair_read_consistency_contract_requires_runtime() {
    let repository = PostgresContextGraphRepository::connect_lazy(
        "postgres://contextlab:contextlab@localhost/contextlab",
    )
    .expect("valid lazy PostgreSQL configuration");
    let source_scope = VersionedContextScopeV1::new(
        ProjectId::from_uuid(Uuid::from_u128(710)),
        ContextId::from_uuid(Uuid::from_u128(711)),
        CommitId::from_uuid(Uuid::from_u128(712)),
    );
    let target_scope = VersionedContextScopeV1::new(
        source_scope.project_id(),
        source_scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(713)),
    );
    let _ = repository
        .read_context_diff_snapshot_pair(source_scope, target_scope)
        .await;
}
