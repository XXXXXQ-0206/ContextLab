//! Contract tests for deterministic embedding behavior.

use contextlab_embedding::{
    DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig, EmbeddingCapabilityPort,
    EmbeddingCapabilityRequirement, EmbeddingError, EmbeddingText, VectorIndex,
};

fn adapter() -> DeterministicEmbeddingAdapter {
    DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("local-hash", "1", 16)
            .expect("valid deterministic embedding configuration"),
    )
}

#[test]
fn deterministic_embeddings_have_stable_ids_and_values() {
    let adapter = adapter();
    let text = EmbeddingText::new("Deterministic retrieval needs stable vectors.")
        .expect("valid embedding text");

    let first = adapter.embed("document:alpha", &text).expect("embedding");
    let second = adapter.embed("document:alpha", &text).expect("embedding");

    assert_eq!(first.id(), second.id());
    assert_eq!(first.vector(), second.vector());
    assert_eq!(first.model().version().as_str(), "1");
}

#[test]
fn vector_index_breaks_equal_scores_by_stable_source_id() {
    let adapter = adapter();
    let query = EmbeddingText::new("same query").expect("valid query");
    let alpha = adapter
        .embed("chunk:alpha", &query)
        .expect("alpha embedding");
    let beta = adapter.embed("chunk:beta", &query).expect("beta embedding");
    let index = VectorIndex::from_embeddings(vec![beta, alpha]).expect("valid index");

    let results = index.search(adapter.embed_query(&query).expect("query embedding"), 2);

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].source_key(), "chunk:alpha");
    assert_eq!(results[1].source_key(), "chunk:beta");
}

#[test]
fn embedding_text_rejects_blank_input() {
    let error = EmbeddingText::new(" \t ").expect_err("blank embedding text must fail");

    assert!(error.to_string().contains("must not be empty"));
}

#[test]
fn capability_is_deterministic_and_fails_closed_for_incompatible_consumers() {
    let adapter = adapter();
    let first = adapter.embedding_capability().expect("capability");
    let second = adapter.embedding_capability().expect("capability");

    assert_eq!(first.id(), second.id());
    assert_eq!(first.schema_version().as_str(), "embedding-capability-v1");
    assert_eq!(first.dimension(), 16);

    let incompatible =
        EmbeddingCapabilityRequirement::new("embedding-capability-v1", "local-hash", "2", 16)
            .expect("requirement");
    assert!(matches!(
        first.require_compatible(&incompatible),
        Err(EmbeddingError::CapabilityMismatch { .. })
    ));
}
