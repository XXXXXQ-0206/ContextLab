//! Contract tests for the provider-free Knowledge-to-Plugin citation bridge.

use contextlab_embedding::{
    DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig, EmbeddingCapabilityRequirement,
};
use contextlab_knowledge::{
    ChunkingPolicy, CitationCompatibilityRequirement, CitationSchemaVersion,
    InMemoryKnowledgeRepository, KnowledgeIngestion, KnowledgePluginCitationBridge,
    KnowledgePluginCitationBridgeError, KnowledgePluginCitationRequirement, KnowledgeQuery,
    KnowledgeScope, RetrievalRequest,
};
use contextlab_mcp::{
    CapabilityDescriptor, CapabilityKind, CapabilityRegistry, LifecycleContract, LifecyclePhase,
    PluginId, PluginManifest, Version,
};
use serde_json::Value;

const CAPABILITY_ID: &str = "knowledge.citations";

fn citation_requirement(schema: &str) -> CitationCompatibilityRequirement {
    CitationCompatibilityRequirement::new(
        CitationSchemaVersion::new(schema).expect("citation schema"),
        EmbeddingCapabilityRequirement::new("embedding-capability-v1", "local-hash", "1", 16)
            .expect("embedding requirement"),
    )
}

fn registry(kind: CapabilityKind, version: Version) -> CapabilityRegistry {
    let descriptor =
        CapabilityDescriptor::new(CAPABILITY_ID, kind, version).expect("capability descriptor");
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
        PluginId::new("knowledge.plugin").expect("plugin id"),
        "Knowledge citations",
        Version::new(1, 0, 0),
        vec![descriptor],
        lifecycle,
    )
    .expect("manifest");
    let mut registry = CapabilityRegistry::new();
    registry.register(manifest).expect("register plugin");
    registry
}

fn citation_capability() -> contextlab_knowledge::KnowledgeCitationCapability {
    let repository = InMemoryKnowledgeRepository::new(DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("local-hash", "1", 16).expect("embedding config"),
    ));
    let scope = KnowledgeScope::from_stable_key("workspace:plugin-bridge").expect("scope");
    for (key, body) in [
        (
            "zeta",
            "Private zeta source must remain inside the repository.",
        ),
        (
            "alpha",
            "Private alpha source must remain inside the repository.",
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
            citation_requirement("citation-schema-v1"),
        )
        .expect("citation capability")
}

fn plugin_requirement(
    version: Version,
    citation_schema: &str,
) -> KnowledgePluginCitationRequirement {
    KnowledgePluginCitationRequirement::new(
        CAPABILITY_ID,
        version,
        citation_requirement(citation_schema),
    )
    .expect("plugin citation requirement")
}

fn assert_no_private_keys(value: &Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                let key = key.to_ascii_lowercase();
                assert!(
                    !matches!(
                        key.as_str(),
                        "content"
                            | "raw_content"
                            | "document_body"
                            | "chunk_body"
                            | "chunk_text"
                            | "source_text"
                    ),
                    "private content key leaked: {key}"
                );
                assert!(
                    !matches!(key.as_str(), "query" | "query_fingerprint"),
                    "private query key leaked: {key}"
                );
                assert!(
                    !matches!(key.as_str(), "vector" | "embedding_vector" | "vectors"),
                    "embedding vector key leaked: {key}"
                );
                assert!(!key.contains("secret"), "provider secret key leaked: {key}");
                assert!(!key.contains("credential"), "credential key leaked: {key}");
                assert_no_private_keys(value);
            }
        }
        Value::Array(values) => values.iter().for_each(assert_no_private_keys),
        _ => {}
    }
}

#[test]
fn bridge_resolves_resource_capability_and_projects_only_stable_redacted_metadata() {
    let registry = registry(CapabilityKind::Resource, Version::new(1, 2, 0));
    let citation_capability = citation_capability();

    let resolved = KnowledgePluginCitationBridge::resolve(
        &registry,
        &citation_capability,
        &plugin_requirement(Version::new(1, 1, 0), "citation-schema-v1"),
    )
    .expect("compatible bridge");

    assert_eq!(resolved.plugin_id(), "knowledge.plugin");
    assert_eq!(resolved.capability_id(), CAPABILITY_ID);
    assert_eq!(resolved.capability_kind(), CapabilityKind::Resource);
    assert_eq!(resolved.capability_version(), Version::new(1, 2, 0));
    assert_eq!(resolved.citation_capability_id(), citation_capability.id());
    assert_eq!(
        resolved.citation_schema_version().as_str(),
        "citation-schema-v1"
    );
    assert_eq!(
        resolved.retrieval_id(),
        citation_capability.read_record().retrieval_id()
    );
    assert!(
        resolved
            .citations()
            .windows(2)
            .all(|pair| pair[0].chunk_id() < pair[1].chunk_id())
    );
    assert!(!resolved.contains_raw_private_content());
    assert!(!resolved.contains_raw_vectors());
    assert!(!resolved.contains_raw_query());
    assert!(!resolved.contains_provider_secrets());

    let debug = format!("{resolved:?}");
    assert!(!debug.contains("Private alpha source"));
    assert!(!debug.contains("private source query"));
    assert!(!debug.contains("provider_secret"));

    let encoded = serde_json::to_value(&resolved).expect("projection serializes");
    assert_no_private_keys(&encoded);
}

#[test]
fn bridge_fails_closed_for_missing_incompatible_and_wrong_kind_capabilities() {
    let citation_capability = citation_capability();
    let missing = KnowledgePluginCitationRequirement::new(
        "knowledge.missing",
        Version::new(1, 0, 0),
        citation_requirement("citation-schema-v1"),
    )
    .expect("missing requirement");
    let error = KnowledgePluginCitationBridge::resolve(
        &CapabilityRegistry::new(),
        &citation_capability,
        &missing,
    )
    .expect_err("missing capability must fail closed");
    assert!(matches!(
        error,
        KnowledgePluginCitationBridgeError::CapabilityResolution(_)
    ));

    let incompatible = KnowledgePluginCitationBridge::resolve(
        &registry(CapabilityKind::Resource, Version::new(1, 0, 0)),
        &citation_capability,
        &plugin_requirement(Version::new(1, 1, 0), "citation-schema-v1"),
    )
    .expect_err("old capability must fail closed");
    assert!(matches!(
        incompatible,
        KnowledgePluginCitationBridgeError::CapabilityResolution(_)
    ));

    let wrong_kind = KnowledgePluginCitationBridge::resolve(
        &registry(CapabilityKind::Tool, Version::new(1, 2, 0)),
        &citation_capability,
        &plugin_requirement(Version::new(1, 0, 0), "citation-schema-v1"),
    )
    .expect_err("wrong capability kind must fail closed");
    assert!(matches!(
        wrong_kind,
        KnowledgePluginCitationBridgeError::WrongCapabilityKind {
            expected: CapabilityKind::Resource,
            actual: CapabilityKind::Tool,
            ..
        }
    ));
}

#[test]
fn bridge_fails_closed_when_citation_compatibility_is_not_exact() {
    let error = KnowledgePluginCitationBridge::resolve(
        &registry(CapabilityKind::Resource, Version::new(1, 0, 0)),
        &citation_capability(),
        &plugin_requirement(Version::new(1, 0, 0), "citation-schema-v2"),
    )
    .expect_err("citation schema mismatch must fail closed");

    assert!(matches!(
        error,
        KnowledgePluginCitationBridgeError::CitationCompatibility(_)
    ));
}

#[test]
fn bridge_projection_rejects_tampered_contract_fields_during_deserialization() {
    let registry = registry(CapabilityKind::Resource, Version::new(1, 2, 0));
    let projection = KnowledgePluginCitationBridge::resolve(
        &registry,
        &citation_capability(),
        &plugin_requirement(Version::new(1, 0, 0), "citation-schema-v1"),
    )
    .expect("compatible bridge");
    let mut encoded = serde_json::to_value(projection).expect("projection JSON");
    encoded["schema_version"] = serde_json::json!("2.0.0");
    encoded["capability_kind"] = serde_json::json!("tool");
    encoded["citations"]
        .as_array_mut()
        .expect("citations")
        .reverse();

    let decoded =
        serde_json::from_value::<contextlab_knowledge::KnowledgePluginCitationProjection>(encoded);

    assert!(
        decoded.is_err(),
        "tampered bridge contracts must fail closed"
    );
}
