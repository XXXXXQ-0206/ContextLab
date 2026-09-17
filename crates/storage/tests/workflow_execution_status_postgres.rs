//! Opt-in PostgreSQL evidence for the private workflow execution status repository.

use contextlab_context_core::ContextId;
use contextlab_storage::{
    CONTEXT_PLATFORM_MIGRATION, PersistWorkflowExecutionStatusV1, PostgresContextGraphRepository,
    WORKSPACE_GRAPH_SEED, WorkflowExecutionStatusPersistenceError,
    WorkflowExecutionStatusRepository, WorkflowExecutionStatusWriteDisposition,
};
use contextlab_versioning::CommitId;
use contextlab_workflow::{
    ContextCommitSource, WorkflowCapabilitySnapshot, WorkflowContextBinding,
    WorkflowContextBindingId, WorkflowDefinition, WorkflowExecution,
    WorkflowExecutionStatusProjectionV1, WorkflowId, WorkflowNode, WorkflowNodeId,
    WorkflowRevision, WorkflowRunId,
};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

const SEED_CONTEXT_ID: Uuid = Uuid::from_u128(0x44444444444444448444444444444444);
const SEED_COMMIT_ID: Uuid = Uuid::from_u128(0x77777777777747778777777777777777);

fn projection(
    workflow_id: u128,
    binding_id: u128,
    run_id: u128,
) -> WorkflowExecutionStatusProjectionV1 {
    let context_id = ContextId::from_uuid(SEED_CONTEXT_ID);
    let commit_id = CommitId::from_uuid(SEED_COMMIT_ID);
    let revision = WorkflowRevision::new(1).expect("revision");
    let binding = WorkflowContextBinding::new(
        WorkflowContextBindingId::from_uuid(Uuid::from_u128(binding_id)),
        WorkflowDefinition::new(
            WorkflowId::from_uuid(Uuid::from_u128(workflow_id)),
            revision,
            vec![WorkflowNode::new(
                WorkflowNodeId::from_uuid(Uuid::from_u128(workflow_id + 1)),
                revision,
            )],
            Vec::new(),
        )
        .expect("workflow definition"),
        ContextCommitSource::new(context_id, commit_id),
    );
    let execution = WorkflowExecution::start(
        binding.clone(),
        WorkflowRunId::from_uuid(Uuid::from_u128(run_id)),
        &WorkflowCapabilitySnapshot::default(),
    )
    .expect("workflow execution");
    WorkflowExecutionStatusProjectionV1::from_log(
        binding,
        &WorkflowCapabilitySnapshot::default(),
        execution.log().clone(),
    )
    .expect("workflow status projection")
}

#[tokio::test]
#[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
async fn postgres_repository_persists_replays_and_reloads_exact_projection_scope() {
    let database_url = std::env::var("CONTEXTLAB_TEST_DATABASE_URL")
        .expect("CONTEXTLAB_TEST_DATABASE_URL must name an empty disposable database");
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&database_url)
        .await
        .expect("connect to disposable PostgreSQL database");

    sqlx::raw_sql(CONTEXT_PLATFORM_MIGRATION)
        .execute(&pool)
        .await
        .expect("apply ContextLab migrations to an empty database");
    sqlx::raw_sql(WORKSPACE_GRAPH_SEED)
        .execute(&pool)
        .await
        .expect("apply deterministic workspace seed");

    let repository = PostgresContextGraphRepository::new(pool.clone());
    let first = projection(0x102, 0x103, 0x104);
    let command = PersistWorkflowExecutionStatusV1::new(first.clone());
    let created = repository
        .persist_workflow_execution_status(command.clone())
        .await
        .expect("create projection");
    assert_eq!(
        created.disposition(),
        WorkflowExecutionStatusWriteDisposition::Created
    );

    let replayed = repository
        .persist_workflow_execution_status(command)
        .await
        .expect("replay projection");
    assert_eq!(
        replayed.disposition(),
        WorkflowExecutionStatusWriteDisposition::Replayed
    );

    let second_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("open a second database connection");
    let second_repository = PostgresContextGraphRepository::new(second_pool.clone());
    let read = second_repository
        .read_workflow_execution_status(first.context_source().context_id(), first.run_id())
        .await
        .expect("read projection")
        .expect("projection exists on a new connection");
    assert_eq!(read, first);

    let conflicting = projection(0x202, 0x203, 0x104);
    let conflict = second_repository
        .persist_workflow_execution_status(PersistWorkflowExecutionStatusV1::new(conflicting))
        .await
        .expect_err("immutable run reuse must conflict");
    assert!(matches!(
        conflict,
        WorkflowExecutionStatusPersistenceError::Conflict { .. }
    ));

    second_pool.close().await;
    pool.close().await;
}
