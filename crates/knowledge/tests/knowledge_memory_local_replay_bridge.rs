//! Contract tests for the redacted Knowledge-to-Memory local replay bridge.

use contextlab_embedding::{DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig};
use contextlab_knowledge::{
    ChunkingPolicy, InMemoryKnowledgeRepository, KnowledgeIngestion,
    KnowledgeLocalCitationProjectionV1, KnowledgeMemoryReplayBridge,
    KnowledgeMemoryReplayProjection, KnowledgeQuery, KnowledgeScope, RetrievalRequest,
};
use contextlab_memory::{
    Importance, InMemoryMemoryTimeline, MemoryCapabilitySchemaVersion, MemoryRetentionCapability,
    MemoryRetentionCapabilityRequest, MemoryRetentionCapabilityRequirement, MemoryScope,
    MemoryWrite, RetentionDecision, RetentionPolicy,
};
use serde_json::json;

fn repository() -> InMemoryKnowledgeRepository<DeterministicEmbeddingAdapter> {
    InMemoryKnowledgeRepository::new(DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("knowledge-memory-test-model", "1", 4)
            .expect("adapter configuration"),
    ))
}

fn local_inputs() -> (
    KnowledgeLocalCitationProjectionV1,
    MemoryRetentionCapability,
) {
    let repository = repository();
    let knowledge_scope = KnowledgeScope::from_stable_key("workspace:alpha").expect("scope");
    repository
        .ingest(
            KnowledgeIngestion::new(
                knowledge_scope,
                "guide",
                "Private guide",
                "source-v7",
                "Private citation source body must not cross the replay bridge.",
                ChunkingPolicy::new("chunking-v1", 16, 0).expect("chunking"),
            )
            .expect("ingestion"),
        )
        .expect("ingest");
    repository
        .ingest(
            KnowledgeIngestion::new(
                knowledge_scope,
                "guide-supplement",
                "Private guide supplement",
                "source-v7",
                "A second private source ensures canonical ordering is exercised.",
                ChunkingPolicy::new("chunking-v1", 64, 0).expect("chunking"),
            )
            .expect("ingestion"),
        )
        .expect("ingest supplement");
    let citations = repository
        .project_local_citation_v1(
            RetrievalRequest::new(
                KnowledgeQuery::new("bridge source").expect("query"),
                10,
                "retrieval-v1",
            )
            .expect("retrieval")
            .for_scope(knowledge_scope),
        )
        .expect("citations");

    let timeline = InMemoryMemoryTimeline::new();
    let memory_scope = MemoryScope::from_stable_key("conversation:alpha").expect("scope");
    let created = timeline
        .append(
            MemoryWrite::new(
                memory_scope,
                "bridge-memory",
                "Private memory body must not cross the replay bridge.",
                Importance::new(80).expect("importance"),
                false,
            )
            .expect("memory write"),
        )
        .expect("create");
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
                .expect("policy"),
                MemoryRetentionCapabilityRequirement::new(
                    MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1")
                        .expect("schema"),
                    "retention-v1",
                )
                .expect("requirement"),
            )
            .expect("request"),
        )
        .expect("retention");

    (citations, retention)
}

#[test]
fn local_replay_bridge_is_deterministic_and_redacted() {
    let (citations, retention) = local_inputs();
    let first = KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");
    let second = KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");

    assert_eq!(first.id(), second.id());
    assert_eq!(first.citation_projection_id(), citations.id());
    assert_eq!(
        first.knowledge_scope(),
        citations.scope().expect("exact scope")
    );
    assert_eq!(first.retrieval_id(), citations.retrieval_id());
    assert_eq!(first.retrieval_version(), "retrieval-v1");
    assert_eq!(first.citations(), citations.citations());
    assert_eq!(first.memory_id(), retention.memory_id());
    assert_eq!(first.memory_scope(), retention.scope());
    assert_eq!(
        first.retention_decision(),
        RetentionDecision::RetainedImportant
    );
    assert_eq!(
        first.replay_state(),
        contextlab_memory::MemoryReplayState::Active
    );
    assert!(!first.contains_raw_knowledge_content());
    assert!(!first.contains_raw_memory_content());
    assert!(!first.contains_raw_vectors());
    assert!(!first.contains_raw_query());
    let decoded: KnowledgeMemoryReplayProjection =
        serde_json::from_value(serde_json::to_value(&first).expect("serialized replay projection"))
            .expect("validated replay projection round trip");
    assert_eq!(decoded, first);

    let debug = format!("{first:?}");
    assert!(!debug.contains("Private citation source body must not cross the replay bridge."));
    assert!(!debug.contains("Private memory body must not cross the replay bridge."));
}

#[test]
fn local_replay_bridge_fails_closed_for_an_unknown_citation_projection_schema() {
    let (citations, retention) = local_inputs();
    let mut encoded = serde_json::to_value(citations).expect("citation projection JSON");
    encoded["schema_version"] = json!("knowledge-local-citation-projection-v999");
    let incompatible = serde_json::from_value(encoded).expect("deserializable foreign projection");

    let error = KnowledgeMemoryReplayBridge::project(&incompatible, &retention)
        .expect_err("unknown citation projection schemas must fail closed");

    assert!(error.to_string().contains("citation projection schema"));
}

#[test]
fn local_replay_bridge_fails_closed_for_an_unknown_memory_capability_schema() {
    let (citations, retention) = local_inputs();
    let mut encoded = serde_json::to_value(retention).expect("retention capability JSON");
    encoded["schema_version"] = json!("memory-retention-capability-v999");
    let incompatible = serde_json::from_value(encoded).expect("deserializable foreign capability");

    let error = KnowledgeMemoryReplayBridge::project(&citations, &incompatible)
        .expect_err("unknown memory capability schemas must fail closed");

    assert!(error.to_string().contains("memory capability schema"));
}

#[test]
fn local_replay_bridge_requires_an_exact_knowledge_scope() {
    let (citations, retention) = local_inputs();
    let mut encoded = serde_json::to_value(citations).expect("citation projection JSON");
    encoded["scope"] = serde_json::Value::Null;
    let unscoped = serde_json::from_value(encoded).expect("unscoped citation projection");

    let error = KnowledgeMemoryReplayBridge::project(&unscoped, &retention)
        .expect_err("unscoped knowledge reads must fail closed");

    assert!(matches!(
        error,
        contextlab_knowledge::KnowledgeMemoryReplayBridgeError::CitationScopeRequired
    ));
}

#[test]
fn local_replay_projection_identity_includes_retention_outcome() {
    let (citations, retention) = local_inputs();
    let original =
        KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");

    let mut changed_decision = serde_json::to_value(&retention).expect("retention capability JSON");
    changed_decision["decision"] = json!("retained_fresh");
    let changed_decision =
        serde_json::from_value(changed_decision).expect("changed retention decision");
    let changed_decision = KnowledgeMemoryReplayBridge::project(&citations, &changed_decision)
        .expect("projection with changed decision");

    let mut changed_state = serde_json::to_value(&retention).expect("retention capability JSON");
    changed_state["replay_state"] = json!("forgotten");
    changed_state["decision"] = json!("expired_forgotten");
    let changed_state = serde_json::from_value(changed_state).expect("changed replay state");
    let changed_state = KnowledgeMemoryReplayBridge::project(&citations, &changed_state)
        .expect("projection with changed replay state");

    assert_ne!(original.id(), changed_decision.id());
    assert_ne!(original.id(), changed_state.id());
    assert_ne!(changed_decision.id(), changed_state.id());
}

#[test]
fn local_replay_projection_identity_includes_memory_capability_identity() {
    let (citations, retention) = local_inputs();
    let original =
        KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");

    let mut encoded = serde_json::to_value(&retention).expect("retention capability JSON");
    encoded["id"] = json!("00000000-0000-0000-0000-000000000001");
    encoded["evaluated_at_millis"] = json!(3_000);
    let changed_retention = serde_json::from_value(encoded).expect("changed retention capability");
    let changed = KnowledgeMemoryReplayBridge::project(&citations, &changed_retention)
        .expect("projection with another capability identity");

    assert_ne!(original.id(), changed.id());
}

#[test]
fn local_replay_projection_identity_includes_knowledge_scope() {
    let (citations, retention) = local_inputs();
    let original =
        KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");
    let other_scope = KnowledgeScope::from_stable_key("workspace:other").expect("scope");
    let mut encoded = serde_json::to_value(citations).expect("citation projection JSON");
    encoded["scope"] = serde_json::to_value(other_scope).expect("scope JSON");
    let other_citations = serde_json::from_value(encoded).expect("other scoped citations");
    let changed = KnowledgeMemoryReplayBridge::project(&other_citations, &retention)
        .expect("projection with another exact scope");

    assert_ne!(original.id(), changed.id());
    assert_ne!(original.knowledge_scope(), changed.knowledge_scope());
}

#[test]
fn local_replay_projection_rejects_an_unknown_schema_during_deserialization() {
    let (citations, retention) = local_inputs();
    let projection =
        KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");
    let mut encoded = serde_json::to_value(projection).expect("projection JSON");
    encoded["schema_version"] = json!("knowledge-memory-local-replay-v999");

    let decoded = serde_json::from_value::<KnowledgeMemoryReplayProjection>(encoded);

    assert!(decoded.is_err(), "unknown schemas must fail closed");
}

#[test]
fn local_replay_projection_rejects_noncanonical_citations_during_deserialization() {
    let (citations, retention) = local_inputs();
    let projection =
        KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");
    let mut encoded = serde_json::to_value(projection).expect("projection JSON");
    let encoded_citations = encoded["citations"].as_array_mut().expect("citation array");
    assert!(
        encoded_citations.len() > 1,
        "fixture must exercise ordering"
    );
    encoded_citations.reverse();

    let decoded = serde_json::from_value::<KnowledgeMemoryReplayProjection>(encoded);

    assert!(decoded.is_err(), "noncanonical citations must fail closed");
}

#[test]
fn local_replay_projection_rejects_source_inconsistency_during_deserialization() {
    let (citations, retention) = local_inputs();
    let projection =
        KnowledgeMemoryReplayBridge::project(&citations, &retention).expect("projection");
    let mut encoded = serde_json::to_value(projection).expect("projection JSON");
    encoded["retrieval_version"] = json!("tampered-retrieval-v999");

    let decoded = serde_json::from_value::<KnowledgeMemoryReplayProjection>(encoded);

    assert!(decoded.is_err(), "source facts must remain bound to the ID");
}
