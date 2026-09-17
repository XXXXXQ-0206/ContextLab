//! Cross-domain proof that Workflow and Knowledge consume one immutable registry snapshot.

use contextlab_embedding::{
    DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig, EmbeddingCapabilityRequirement,
};
use contextlab_knowledge::{
    ChunkingPolicy, CitationCompatibilityRequirement, CitationSchemaVersion,
    InMemoryKnowledgeRepository, KnowledgeIngestion, KnowledgePluginCitationBridge,
    KnowledgePluginCitationRequirement, KnowledgeQuery, KnowledgeScope, RetrievalRequest,
};
use contextlab_mcp::{
    CapabilityDescriptor, CapabilityKind, CapabilityRegistry, LifecycleContract, LifecyclePhase,
    PluginId, PluginManifest, Version,
};
use contextlab_workflow::{
    WorkflowCapabilityId, WorkflowCapabilityRequirement, WorkflowCapabilityVersion,
    WorkflowPluginCapabilityBridge, WorkflowPluginCapabilityRequirement,
};
use serde_json::Value;

const WORKFLOW_CAPABILITY: &str = "workflow.context-runner";
const KNOWLEDGE_CAPABILITY: &str = "knowledge.citations";

fn registry() -> CapabilityRegistry {
    let lifecycle = LifecycleContract::new(
        Version::new(1, 0, 0),
        [
            LifecyclePhase::Initialize,
            LifecyclePhase::Activate,
            LifecyclePhase::Deactivate,
            LifecyclePhase::Shutdown,
        ],
    )
    .expect("lifecycle");
    let manifest = PluginManifest::new(
        1,
        PluginId::new("context.plugin").expect("plugin id"),
        "Context plugin",
        Version::new(1, 0, 0),
        vec![
            CapabilityDescriptor::new(
                WORKFLOW_CAPABILITY,
                CapabilityKind::Tool,
                Version::new(1, 3, 0),
            )
            .expect("workflow capability"),
            CapabilityDescriptor::new(
                KNOWLEDGE_CAPABILITY,
                CapabilityKind::Resource,
                Version::new(1, 2, 0),
            )
            .expect("knowledge capability"),
        ],
        lifecycle,
    )
    .expect("manifest");
    let mut registry = CapabilityRegistry::new();
    registry.register(manifest).expect("registry");
    registry
}

fn citation_capability() -> contextlab_knowledge::KnowledgeCitationCapability {
    let repository = InMemoryKnowledgeRepository::new(DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("local-hash", "1", 16).expect("embedding config"),
    ));
    let scope = KnowledgeScope::from_stable_key("context:cross-domain").expect("scope");
    for (key, body) in [
        (
            "zeta",
            "Private zeta source must stay inside the repository.",
        ),
        (
            "alpha",
            "Private alpha source must stay inside the repository.",
        ),
    ] {
        repository
            .ingest(
                KnowledgeIngestion::new(
                    scope,
                    key,
                    key,
                    "v1",
                    body,
                    ChunkingPolicy::new("word-boundary-v1", 128, 0).expect("policy"),
                )
                .expect("ingestion"),
            )
            .expect("stored");
    }
    repository
        .retrieve_citation_capability(
            RetrievalRequest::new(
                KnowledgeQuery::new("private source query").expect("query"),
                2,
                "retrieval-v1",
            )
            .expect("request")
            .for_scope(scope),
            CitationCompatibilityRequirement::new(
                CitationSchemaVersion::new("citation-schema-v1").expect("citation schema"),
                EmbeddingCapabilityRequirement::new(
                    "embedding-capability-v1",
                    "local-hash",
                    "1",
                    16,
                )
                .expect("embedding requirement"),
            ),
        )
        .expect("citation capability")
}

fn assert_no_private_keys(value: &Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let key = key.to_ascii_lowercase();
                assert!(!matches!(
                    key.as_str(),
                    "content"
                        | "raw_content"
                        | "document_body"
                        | "chunk_body"
                        | "chunk_text"
                        | "source_text"
                ));
                assert!(!matches!(key.as_str(), "query" | "query_fingerprint"));
                assert!(!matches!(
                    key.as_str(),
                    "vector" | "embedding_vector" | "vectors"
                ));
                assert!(!key.contains("secret"));
                assert!(!key.contains("credential"));
                assert_no_private_keys(value);
            }
        }
        Value::Array(values) => values.iter().for_each(assert_no_private_keys),
        _ => {}
    }
}

#[test]
fn one_immutable_registry_snapshot_composes_workflow_and_knowledge_reads() {
    let snapshot = registry().snapshot();
    let workflow_id = WorkflowCapabilityId::new(WORKFLOW_CAPABILITY).expect("workflow ID");
    let workflow = WorkflowPluginCapabilityBridge::snapshot(
        snapshot.registry(),
        [WorkflowPluginCapabilityRequirement::new(
            WorkflowCapabilityRequirement::new(
                WORKFLOW_CAPABILITY,
                WorkflowCapabilityVersion::new(1, 1, 0),
            )
            .expect("workflow requirement"),
            CapabilityKind::Tool,
        )],
    )
    .expect("workflow snapshot");

    let knowledge = KnowledgePluginCitationBridge::resolve(
        snapshot.registry(),
        &citation_capability(),
        &KnowledgePluginCitationRequirement::new(
            KNOWLEDGE_CAPABILITY,
            Version::new(1, 1, 0),
            CitationCompatibilityRequirement::new(
                CitationSchemaVersion::new("citation-schema-v1").expect("citation schema"),
                EmbeddingCapabilityRequirement::new(
                    "embedding-capability-v1",
                    "local-hash",
                    "1",
                    16,
                )
                .expect("embedding requirement"),
            ),
        )
        .expect("knowledge requirement"),
    )
    .expect("knowledge projection");

    assert_eq!(snapshot.availability().entries().len(), 2);
    assert_eq!(
        workflow.version(&workflow_id),
        Some(WorkflowCapabilityVersion::new(1, 3, 0))
    );
    assert_eq!(knowledge.capability_id(), KNOWLEDGE_CAPABILITY);
    assert_eq!(knowledge.capability_kind(), CapabilityKind::Resource);
    assert_eq!(knowledge.capability_version(), Version::new(1, 2, 0));
    assert!(!knowledge.contains_raw_private_content());
    assert_no_private_keys(&serde_json::to_value(&knowledge).expect("redacted projection"));
}
