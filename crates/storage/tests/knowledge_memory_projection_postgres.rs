//! Opt-in PostgreSQL evidence for immutable private Knowledge/Memory projections.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_embedding::{DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig};
use contextlab_knowledge::{
    ChunkingPolicy, InMemoryKnowledgeRepository, KnowledgeIngestion,
    KnowledgeMemoryContextProjectionBridge, KnowledgeQuery, KnowledgeScope, RetrievalRequest,
};
use contextlab_memory::{
    Importance, InMemoryMemoryTimeline, MemoryCapabilitySchemaVersion,
    MemoryRetentionCapabilityRequest, MemoryRetentionCapabilityRequirement, MemoryScope,
    MemoryWrite, RetentionPolicy,
};
use contextlab_storage::{
    CONTEXT_PLATFORM_MIGRATION, KnowledgeMemoryProjectionPersistenceError,
    KnowledgeMemoryProjectionScope, KnowledgeMemoryProjectionV1Repository,
    KnowledgeMemoryProjectionWriteDisposition, PersistKnowledgeMemoryProjectionV1,
    PostgresContextGraphRepository, WORKSPACE_GRAPH_SEED,
};
use contextlab_versioning::CommitId;
use serde_json::{Value, json};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use uuid::Uuid;

const SEED_PROJECT_ID: Uuid = Uuid::from_u128(0x22222222222242228222222222222222);
const SEED_CONTEXT_ID: Uuid = Uuid::from_u128(0x44444444444444448444444444444444);
const SEED_COMMIT_ID: Uuid = Uuid::from_u128(0x77777777777747778777777777777777);
const SECOND_SEED_COMMIT_ID: Uuid = Uuid::from_u128(0x77777777777747778777777777777778);

const PRIVATE_KNOWLEDGE_CONTENT: &str =
    "private Knowledge body must remain outside the stored projection";
const PRIVATE_MEMORY_CONTENT: &str =
    "private Memory body must remain outside the stored projection";

fn seed_scope() -> KnowledgeMemoryProjectionScope {
    KnowledgeMemoryProjectionScope::new(
        ProjectId::from_uuid(SEED_PROJECT_ID),
        ContextId::from_uuid(SEED_CONTEXT_ID),
        CommitId::from_uuid(SEED_COMMIT_ID),
    )
}

fn projection_command(source_key: &str) -> PersistKnowledgeMemoryProjectionV1 {
    let scope = seed_scope();
    let context_id = scope.context_id();
    let knowledge_scope =
        KnowledgeScope::from_stable_key(format!("context:{context_id}")).expect("knowledge scope");
    let knowledge = InMemoryKnowledgeRepository::new(DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("contextlab-local-hash", "1", 8)
            .expect("embedding config"),
    ));
    knowledge
        .ingest(
            KnowledgeIngestion::new(
                knowledge_scope,
                source_key,
                "Projection source",
                "local-v1",
                PRIVATE_KNOWLEDGE_CONTENT,
                ChunkingPolicy::new("characters-v1", 128, 0).expect("chunking"),
            )
            .expect("ingestion"),
        )
        .expect("ingest");
    let citations = knowledge
        .project_local_citation_v1(
            RetrievalRequest::new(
                KnowledgeQuery::new("inspection").expect("query"),
                1,
                "retrieval-v1",
            )
            .expect("request")
            .for_scope(knowledge_scope),
        )
        .expect("citations");

    let memory_scope =
        MemoryScope::from_stable_key(format!("context:{context_id}")).expect("memory scope");
    let memory = InMemoryMemoryTimeline::new();
    let created = memory
        .append(
            MemoryWrite::new(
                memory_scope,
                "memory-key",
                PRIVATE_MEMORY_CONTENT,
                Importance::new(80).expect("importance"),
                true,
            )
            .expect("memory write"),
        )
        .expect("append");
    let policy = RetentionPolicy::new(
        "retention-v1",
        100,
        Importance::new(40).expect("importance"),
    )
    .expect("policy");
    let requirement = MemoryRetentionCapabilityRequirement::new(
        MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1").expect("schema"),
        "retention-v1",
    )
    .expect("requirement");
    let retention = memory
        .record_retention_capability(
            MemoryRetentionCapabilityRequest::new(
                created.memory_id(),
                memory_scope,
                2,
                "context-build-v1",
                policy,
                requirement,
            )
            .expect("retention request"),
        )
        .expect("retention");
    let projection =
        KnowledgeMemoryContextProjectionBridge::project(context_id, &citations, &retention)
            .expect("context projection");
    PersistKnowledgeMemoryProjectionV1::new(scope, projection).expect("persist command")
}

#[tokio::test]
#[ignore = "requires an empty disposable PostgreSQL database via CONTEXTLAB_TEST_DATABASE_URL"]
async fn postgres_projection_runtime_receipt_covers_exact_immutable_redacted_replay() {
    let database_url = std::env::var("CONTEXTLAB_TEST_DATABASE_URL")
        .expect("CONTEXTLAB_TEST_DATABASE_URL must name an empty disposable database");
    let connect_options = database_url
        .parse::<PgConnectOptions>()
        .expect("CONTEXTLAB_TEST_DATABASE_URL must be a valid PostgreSQL URL");
    assert!(
        matches!(
            connect_options.get_host(),
            "localhost" | "127.0.0.1" | "::1"
        ),
        "the disposable PostgreSQL fixture only permits loopback hosts"
    );
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(connect_options.clone())
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
    let command = projection_command("projection-source");
    let scope = command.scope();
    let created = repository
        .persist_knowledge_memory_projection(command.clone())
        .await
        .expect("create projection");
    assert_eq!(
        created.disposition(),
        KnowledgeMemoryProjectionWriteDisposition::Created
    );

    let stored_projection: (Value,) = sqlx::query_as(
        "SELECT projection FROM knowledge_memory_context_projections WHERE project_id = $1 AND context_id = $2 AND context_commit_id = $3",
    )
    .bind(scope.project_id().as_uuid())
    .bind(scope.context_id().as_uuid())
    .bind(scope.context_commit_id().as_uuid())
    .fetch_one(&pool)
    .await
    .expect("read stored projection JSON");
    let stored_json = stored_projection.0.to_string();
    assert!(!stored_json.contains(PRIVATE_KNOWLEDGE_CONTENT));
    assert!(!stored_json.contains(PRIVATE_MEMORY_CONTENT));

    let replayed = repository
        .persist_knowledge_memory_projection(command.clone())
        .await
        .expect("replay projection");
    assert_eq!(
        replayed.disposition(),
        KnowledgeMemoryProjectionWriteDisposition::Replayed
    );

    let second_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect_with(connect_options)
        .await
        .expect("open a second database connection");
    let second_repository = PostgresContextGraphRepository::new(second_pool.clone());
    let loaded = second_repository
        .read_knowledge_memory_projection(scope)
        .await
        .expect("read projection from second connection");
    assert_eq!(loaded, *command.projection());
    assert!(!loaded.contains_raw_knowledge_content());
    assert!(!loaded.contains_raw_memory_content());
    assert!(!loaded.contains_raw_vectors());
    assert!(!loaded.contains_raw_query());
    assert!(!loaded.contains_provider_secrets());

    let conflicting = projection_command("different-projection-source");
    let conflict = second_repository
        .persist_knowledge_memory_projection(conflicting)
        .await
        .expect_err("immutable exact scope conflict");
    assert!(matches!(
        conflict,
        KnowledgeMemoryProjectionPersistenceError::Conflict { scope: conflict_scope }
            if conflict_scope == scope
    ));

    let project_mismatch = KnowledgeMemoryProjectionScope::new(
        ProjectId::from_uuid(Uuid::from_u128(0x99999999999949999999999999999999)),
        scope.context_id(),
        scope.context_commit_id(),
    );
    let project_mismatch_command =
        PersistKnowledgeMemoryProjectionV1::new(project_mismatch, command.projection().clone())
            .expect("project scope is enforced by the PostgreSQL composite foreign key");
    assert!(matches!(
        second_repository
            .persist_knowledge_memory_projection(project_mismatch_command)
            .await
            .expect_err("foreign project scope must be rejected"),
        KnowledgeMemoryProjectionPersistenceError::Database { .. }
    ));

    let context_mismatch = KnowledgeMemoryProjectionScope::new(
        scope.project_id(),
        ContextId::from_uuid(Uuid::from_u128(0x88888888888848888888888888888888)),
        scope.context_commit_id(),
    );
    assert!(matches!(
        PersistKnowledgeMemoryProjectionV1::new(context_mismatch, command.projection().clone(),),
        Err(KnowledgeMemoryProjectionPersistenceError::ScopeMismatch { .. })
    ));

    let commit_mismatch = KnowledgeMemoryProjectionScope::new(
        scope.project_id(),
        scope.context_id(),
        CommitId::from_uuid(Uuid::from_u128(0xaaaaaaaaaaaa4aaaaaaaaaaaaaaaaaaa)),
    );
    let commit_mismatch_command =
        PersistKnowledgeMemoryProjectionV1::new(commit_mismatch, command.projection().clone())
            .expect("commit scope is enforced by the PostgreSQL composite foreign key");
    assert!(matches!(
        second_repository
            .persist_knowledge_memory_projection(commit_mismatch_command)
            .await
            .expect_err("foreign commit scope must be rejected"),
        KnowledgeMemoryProjectionPersistenceError::Database { .. }
    ));

    let malformed_scope = KnowledgeMemoryProjectionScope::new(
        scope.project_id(),
        scope.context_id(),
        CommitId::from_uuid(SECOND_SEED_COMMIT_ID),
    );
    let mut raw_projection = serde_json::to_value(command.projection()).expect("projection JSON");
    raw_projection["raw_private_content"] = json!("synthetic sentinel");
    sqlx::query(
        "INSERT INTO knowledge_memory_context_projections (project_id, context_id, context_commit_id, schema_version, projection) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(malformed_scope.project_id().as_uuid())
    .bind(malformed_scope.context_id().as_uuid())
    .bind(malformed_scope.context_commit_id().as_uuid())
    .bind("knowledge-memory-context-projection-v1")
    .bind(raw_projection)
    .execute(&pool)
    .await
    .expect("insert synthetic malformed projection fixture");
    assert!(matches!(
        second_repository
            .read_knowledge_memory_projection(malformed_scope)
            .await
            .expect_err("raw private projection shape must fail closed"),
        KnowledgeMemoryProjectionPersistenceError::StoredProjectionInvalid
    ));

    second_pool.close().await;
    pool.close().await;
}
