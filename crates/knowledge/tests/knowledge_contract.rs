//! Contract tests for deterministic knowledge ingestion and retrieval.

use contextlab_embedding::{
    DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig, EmbeddingCapabilityRequirement,
};
use contextlab_knowledge::{
    ChunkingPolicy, CitationCompatibilityRequirement, CitationSchemaVersion,
    InMemoryKnowledgeRepository, KnowledgeError, KnowledgeIngestion, KnowledgeQuery,
    KnowledgeScope, RetrievalRequest,
};

fn repository() -> InMemoryKnowledgeRepository<DeterministicEmbeddingAdapter> {
    let embedding = DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("local-hash", "1", 16)
            .expect("valid deterministic embedding configuration"),
    );
    InMemoryKnowledgeRepository::new(embedding)
}

#[test]
fn ingestion_chunks_and_retrieves_with_citations_without_raw_document_content() {
    let repository = repository();
    let ingestion = KnowledgeIngestion::new(
        KnowledgeScope::from_stable_key("workspace:alpha").expect("scope"),
        "refund-policy",
        "Refund policy",
        "v1",
        "Refunds are available within thirty days. Contact support for an exception.",
        ChunkingPolicy::new("word-boundary-v1", 42, 8).expect("chunking policy"),
    )
    .expect("valid ingestion");

    let receipt = repository.ingest(ingestion).expect("ingestion");
    let results = repository
        .retrieve(
            RetrievalRequest::new(
                KnowledgeQuery::new("refund exception").expect("query"),
                2,
                "retrieval-v1",
            )
            .expect("request"),
        )
        .expect("retrieval");

    assert_eq!(receipt.document_version().as_str(), "v1");
    assert!(!results.citations().is_empty());
    assert_eq!(results.citations()[0].source_version().as_str(), "v1");
    assert!(!results.read_record().contains_raw_content());
    assert!(!results.read_record().contains_raw_query());
}

#[test]
fn chunking_and_retrieval_are_deterministic_across_insertion_order() {
    let first = repository();
    let second = repository();
    let policy = ChunkingPolicy::new("word-boundary-v1", 64, 0).expect("policy");
    let scope = KnowledgeScope::from_stable_key("workspace:alpha").expect("scope");

    let alpha = KnowledgeIngestion::new(
        scope,
        "alpha",
        "Alpha",
        "v1",
        "same ranking text",
        policy.clone(),
    )
    .expect("alpha");
    let beta = KnowledgeIngestion::new(scope, "beta", "Beta", "v1", "same ranking text", policy)
        .expect("beta");

    first.ingest(beta.clone()).expect("beta first");
    first.ingest(alpha.clone()).expect("alpha second");
    second.ingest(alpha).expect("alpha first");
    second.ingest(beta).expect("beta second");

    let request = RetrievalRequest::new(
        KnowledgeQuery::new("same ranking text").expect("query"),
        2,
        "retrieval-v1",
    )
    .expect("request");
    let first_ids = first
        .retrieve(request.clone())
        .expect("first retrieval")
        .citations()
        .iter()
        .map(|citation| citation.chunk_id())
        .collect::<Vec<_>>();
    let second_ids = second
        .retrieve(request)
        .expect("second retrieval")
        .citations()
        .iter()
        .map(|citation| citation.chunk_id())
        .collect::<Vec<_>>();

    assert_eq!(first_ids, second_ids);
}

#[test]
fn citation_capability_records_canonical_provenance_without_raw_chunks() {
    let repository = repository();
    let scope = KnowledgeScope::from_stable_key("workspace:capability").expect("scope");
    let policy = ChunkingPolicy::new("word-boundary-v1", 64, 0).expect("policy");
    repository
        .ingest(
            KnowledgeIngestion::new(
                scope,
                "alpha",
                "Alpha",
                "v1",
                "Private alpha source document.",
                policy.clone(),
            )
            .expect("ingestion"),
        )
        .expect("alpha ingestion");
    repository
        .ingest(
            KnowledgeIngestion::new(
                scope,
                "beta",
                "Beta",
                "v1",
                "Private beta source document.",
                policy,
            )
            .expect("ingestion"),
        )
        .expect("beta ingestion");

    let compatibility = CitationCompatibilityRequirement::new(
        CitationSchemaVersion::new("citation-schema-v1").expect("citation schema"),
        EmbeddingCapabilityRequirement::new("embedding-capability-v1", "local-hash", "1", 16)
            .expect("embedding requirement"),
    );
    let request = RetrievalRequest::new(
        KnowledgeQuery::new("private source").expect("query"),
        2,
        "retrieval-v1",
    )
    .expect("request")
    .for_scope(scope);
    let capability = repository
        .retrieve_citation_capability(request.clone(), compatibility.clone())
        .expect("capability");

    assert_eq!(
        capability.schema_version().as_str(),
        "knowledge-citation-capability-v1"
    );
    assert_eq!(
        capability
            .compatibility()
            .citation_schema_version()
            .as_str(),
        "citation-schema-v1"
    );
    assert!(
        capability
            .sources()
            .windows(2)
            .all(|pair| pair[0].citation().chunk_id() < pair[1].citation().chunk_id())
    );
    assert_eq!(
        capability.sources()[0].embedding_provenance().dimension(),
        16
    );
    assert!(!capability.contains_raw_knowledge_chunks());
    assert!(!format!("{capability:?}").contains("Private alpha source document."));

    let incompatible = CitationCompatibilityRequirement::new(
        CitationSchemaVersion::new("citation-schema-v2").expect("citation schema"),
        compatibility.embedding_requirement().clone(),
    );
    assert!(matches!(
        repository.retrieve_citation_capability(request, incompatible),
        Err(KnowledgeError::CitationCompatibilityMismatch { .. })
    ));
}

#[test]
fn local_citation_projection_v1_is_canonical_and_redacted() {
    let repository = repository();
    let scope = KnowledgeScope::from_stable_key("workspace:local-projection").expect("scope");
    let policy = ChunkingPolicy::new("word-boundary-v1", 64, 0).expect("policy");
    repository
        .ingest(
            KnowledgeIngestion::new(
                scope,
                "alpha",
                "Alpha",
                "v1",
                "Local-only source content must remain private.",
                policy.clone(),
            )
            .expect("ingestion"),
        )
        .expect("alpha ingestion");
    repository
        .ingest(
            KnowledgeIngestion::new(
                scope,
                "beta",
                "Beta",
                "v1",
                "A second local-only source keeps ordering observable.",
                policy,
            )
            .expect("ingestion"),
        )
        .expect("beta ingestion");

    let request = RetrievalRequest::new(
        KnowledgeQuery::new("local-only retrieval query").expect("query"),
        2,
        "retrieval-v1",
    )
    .expect("request")
    .for_scope(scope);
    let first = repository
        .project_local_citation_v1(request.clone())
        .expect("projection");
    let second = repository
        .project_local_citation_v1(request)
        .expect("projection");

    assert_eq!(first.id(), second.id());
    assert_eq!(
        first.schema_version().as_str(),
        "knowledge-local-citation-projection-v1"
    );
    assert_eq!(first.scope(), Some(scope));
    assert!(
        first
            .citations()
            .windows(2)
            .all(|pair| pair[0].chunk_id() < pair[1].chunk_id())
    );
    assert!(!first.contains_raw_knowledge_chunks());
    assert!(!first.contains_raw_retrieval_query());
    assert!(!first.contains_raw_vectors());
    assert!(!first.contains_provider_secrets());

    let debug = format!("{first:?}");
    assert!(!debug.contains("Local-only source content must remain private."));
    assert!(!debug.contains("local-only retrieval query"));
}
