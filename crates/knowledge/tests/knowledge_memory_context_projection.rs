//! Contract tests for the exact Context-scoped Knowledge/Memory projection.

use contextlab_context_core::ContextId;
use contextlab_embedding::{DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig};
use contextlab_knowledge::{
    ChunkingPolicy, InMemoryKnowledgeRepository, KnowledgeIngestion,
    KnowledgeMemoryContextProjectionBridge, KnowledgeMemoryContextProjectionError,
    KnowledgeMemoryContextProjectionV1, KnowledgeMemoryReplayBridgeError, KnowledgeQuery,
    KnowledgeScope, RetrievalRequest,
};
use contextlab_memory::{
    Importance, InMemoryMemoryTimeline, MemoryCapabilitySchemaVersion,
    MemoryRetentionCapabilityRequest, MemoryRetentionCapabilityRequirement, MemoryScope,
    MemoryWrite, RetentionPolicy,
};
use serde_json::json;
use uuid::Uuid;

fn context_id(value: u128) -> ContextId {
    ContextId::from_uuid(Uuid::from_u128(value))
}

fn local_inputs(
    context_id: ContextId,
) -> (
    contextlab_knowledge::KnowledgeLocalCitationProjectionV1,
    contextlab_memory::MemoryRetentionCapability,
) {
    let knowledge_scope = KnowledgeScope::from_stable_key(format!("context:{context_id}"))
        .expect("knowledge context scope");
    let repository = InMemoryKnowledgeRepository::new(DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("context-projection-test-model", "1", 4)
            .expect("embedding configuration"),
    ));
    let policy = ChunkingPolicy::new("chunking-v1", 64, 0).expect("chunking policy");
    repository
        .ingest(
            KnowledgeIngestion::new(
                knowledge_scope,
                "alpha",
                "Private alpha",
                "source-v1",
                "Private alpha content must not cross the projection boundary.",
                policy.clone(),
            )
            .expect("knowledge ingestion"),
        )
        .expect("alpha ingestion");
    repository
        .ingest(
            KnowledgeIngestion::new(
                knowledge_scope,
                "beta",
                "Private beta",
                "source-v1",
                "Private beta content keeps canonical ordering observable.",
                policy,
            )
            .expect("knowledge ingestion"),
        )
        .expect("beta ingestion");
    let citations = repository
        .project_local_citation_v1(
            RetrievalRequest::new(
                KnowledgeQuery::new("private projection query").expect("query"),
                10,
                "retrieval-v1",
            )
            .expect("retrieval request")
            .for_scope(knowledge_scope),
        )
        .expect("citation projection");

    let memory_scope = MemoryScope::from_stable_key(format!("context:{context_id}"))
        .expect("memory context scope");
    let timeline = InMemoryMemoryTimeline::new();
    let created = timeline
        .append(
            MemoryWrite::new(
                memory_scope,
                "context-memory",
                "Private memory content must not cross the projection boundary.",
                Importance::new(80).expect("importance"),
                false,
            )
            .expect("memory write"),
        )
        .expect("memory append");
    let retention = timeline
        .record_retention_capability(
            MemoryRetentionCapabilityRequest::new(
                created.memory_id(),
                memory_scope,
                2_000,
                "context-build-v1",
                RetentionPolicy::new(
                    "retention-v1",
                    1_000,
                    Importance::new(60).expect("importance"),
                )
                .expect("retention policy"),
                MemoryRetentionCapabilityRequirement::new(
                    MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1")
                        .expect("memory schema"),
                    "retention-v1",
                )
                .expect("retention requirement"),
            )
            .expect("retention request"),
        )
        .expect("retention capability");

    (citations, retention)
}

#[test]
fn context_projection_is_deterministic_redacted_and_canonically_ordered() {
    let context = context_id(0x1111);
    let (citations, retention) = local_inputs(context);
    let first = KnowledgeMemoryContextProjectionBridge::project(context, &citations, &retention)
        .expect("projection");
    let second = KnowledgeMemoryContextProjectionBridge::project(context, &citations, &retention)
        .expect("projection");

    assert_eq!(first, second);
    assert_eq!(first.id(), second.id());
    assert_eq!(first.context_id(), context);
    assert_eq!(
        first.knowledge_scope(),
        KnowledgeScope::from_stable_key(format!("context:{context}")).expect("scope")
    );
    assert_eq!(
        first.memory_scope(),
        MemoryScope::from_stable_key(format!("context:{context}")).expect("scope")
    );
    assert!(
        first
            .citations()
            .windows(2)
            .all(|pair| pair[0].chunk_id() < pair[1].chunk_id())
    );
    assert!(!first.contains_raw_knowledge_content());
    assert!(!first.contains_raw_memory_content());
    assert!(!first.contains_raw_vectors());
    assert!(!first.contains_raw_query());
    assert!(!first.contains_provider_secrets());

    let serialized = serde_json::to_string(&first).expect("projection serialization");
    assert!(!serialized.contains("Private alpha content"));
    assert!(!serialized.contains("Private memory content"));
    assert!(!serialized.contains("private projection query"));
    let decoded: KnowledgeMemoryContextProjectionV1 =
        serde_json::from_str(&serialized).expect("validated projection round trip");
    assert_eq!(decoded, first);
}

#[test]
fn context_projection_rejects_knowledge_or_memory_from_another_context() {
    let context = context_id(0x2222);
    let other_context = context_id(0x3333);
    let (citations, retention) = local_inputs(context);

    let error =
        KnowledgeMemoryContextProjectionBridge::project(other_context, &citations, &retention)
            .expect_err("foreign Knowledge scope must fail closed");
    assert!(matches!(
        error,
        KnowledgeMemoryContextProjectionError::KnowledgeScopeMismatch { .. }
    ));

    let (_other_citations, foreign_retention) = local_inputs(other_context);
    let error =
        KnowledgeMemoryContextProjectionBridge::project(context, &citations, &foreign_retention)
            .expect_err("foreign Memory scope must fail closed");
    assert!(matches!(
        error,
        KnowledgeMemoryContextProjectionError::MemoryScopeMismatch { .. }
    ));
}

#[test]
fn context_projection_deserialization_is_versioned_and_fail_closed() {
    let context = context_id(0x4444);
    let (citations, retention) = local_inputs(context);
    let projection =
        KnowledgeMemoryContextProjectionBridge::project(context, &citations, &retention)
            .expect("projection");

    let mut incompatible = serde_json::to_value(&projection).expect("projection JSON");
    incompatible["schema_version"] = json!("knowledge-memory-context-projection-v999");
    assert!(serde_json::from_value::<KnowledgeMemoryContextProjectionV1>(incompatible).is_err());

    let mut foreign_scope = serde_json::to_value(&projection).expect("projection JSON");
    foreign_scope["context_id"] = json!(context_id(0x5555).to_string());
    assert!(serde_json::from_value::<KnowledgeMemoryContextProjectionV1>(foreign_scope).is_err());

    let mut unknown_field = serde_json::to_value(&projection).expect("projection JSON");
    unknown_field["unexpected"] = json!(true);
    assert!(serde_json::from_value::<KnowledgeMemoryContextProjectionV1>(unknown_field).is_err());
}

#[test]
fn context_projection_does_not_reinterpret_existing_domain_errors() {
    let context = context_id(0x6666);
    let (citations, retention) = local_inputs(context);
    let mut incompatible = serde_json::to_value(&citations).expect("citation JSON");
    incompatible["schema_version"] = json!("knowledge-local-citation-projection-v999");
    let incompatible = serde_json::from_value(incompatible).expect("source remains parseable");

    let error = KnowledgeMemoryContextProjectionBridge::project(context, &incompatible, &retention)
        .expect_err("source schema mismatch must remain fail closed");
    assert!(
        matches!(
            error,
            KnowledgeMemoryContextProjectionError::Replay(
                KnowledgeMemoryReplayBridgeError::CitationProjectionSchemaMismatch { .. }
            )
        ) || error.to_string().contains("citation projection schema")
    );
}
