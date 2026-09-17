//! Focused V1 Context metadata semantic-diff contract tests.

use chrono::{TimeZone, Utc};
use contextlab_context_core::ContextMetadata;
use contextlab_diff_engine::{
    BehaviorSnapshotV1, ContextDiffRequestV1, ContextDiffResultV1, ContextDiffService,
    ContextDiffSnapshotV1, ContextMetadataChangeV1, DiffInputError, EvaluationSnapshotV1,
    SemanticSnapshotV1,
};
use contextlab_graph::ContextGraph;
use serde_json::json;

fn metadata(updated_at_seconds: i64, label: &str) -> ContextMetadata {
    let created_at = Utc
        .timestamp_opt(1_751_500_000, 0)
        .single()
        .expect("valid timestamp");
    let updated_at = Utc
        .timestamp_opt(updated_at_seconds, 0)
        .single()
        .expect("valid timestamp");
    let mut metadata = ContextMetadata::new(created_at);
    metadata.set_label("owner", label, updated_at);
    metadata
}

fn snapshot(metadata: ContextMetadata) -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new_with_metadata(ContextGraph::new(), Vec::new(), metadata)
            .expect("valid semantic snapshot"),
        BehaviorSnapshotV1::new(Vec::new()).expect("valid behavior snapshot"),
        EvaluationSnapshotV1::new("suite:metadata:v1", Vec::new())
            .expect("valid evaluation snapshot"),
    )
    .expect("valid diff snapshot")
}

fn snapshot_without_metadata() -> ContextDiffSnapshotV1 {
    ContextDiffSnapshotV1::new(
        SemanticSnapshotV1::new(ContextGraph::new(), Vec::new()).expect("valid semantic snapshot"),
        BehaviorSnapshotV1::new(Vec::new()).expect("valid behavior snapshot"),
        EvaluationSnapshotV1::new("suite:metadata:v1", Vec::new())
            .expect("valid evaluation snapshot"),
    )
    .expect("valid diff snapshot")
}

fn assert_metadata_change_unknown_field_rejected(change: ContextMetadataChangeV1) {
    let mut encoded = serde_json::to_value(change).expect("serialize metadata change");
    encoded["unexpected"] = json!(true);

    assert!(
        serde_json::from_value::<ContextMetadataChangeV1>(encoded).is_err(),
        "metadata change variants must reject unknown fields"
    );
}

#[test]
fn metadata_only_change_is_non_empty_and_deterministic() {
    let request = ContextDiffRequestV1::new(
        snapshot(metadata(1_751_500_000, "support")),
        snapshot(metadata(1_751_500_010, "platform")),
    )
    .expect("valid comparison request");

    let first = ContextDiffService::compare(request.clone()).expect("comparable snapshots");
    let second = ContextDiffService::compare(request).expect("comparable snapshots");

    assert!(!first.semantic().is_empty());
    assert_eq!(first, second);
    assert!(matches!(
        first.semantic().metadata_change(),
        Some(ContextMetadataChangeV1::Modified { original, revised })
            if original.labels().get("owner").map(String::as_str) == Some("support")
                && revised.labels().get("owner").map(String::as_str) == Some("platform")
    ));
}

#[test]
fn nullable_prior_metadata_changes_use_explicit_kinds() {
    let added = ContextDiffService::compare(
        ContextDiffRequestV1::new(
            snapshot_without_metadata(),
            snapshot(metadata(1_751_500_010, "platform")),
        )
        .expect("valid comparison request"),
    )
    .expect("comparable snapshots");
    assert!(matches!(
        added.semantic().metadata_change(),
        Some(ContextMetadataChangeV1::Added { revised })
            if revised.labels().get("owner").map(String::as_str) == Some("platform")
    ));

    let removed = ContextDiffService::compare(
        ContextDiffRequestV1::new(
            snapshot(metadata(1_751_500_000, "support")),
            snapshot_without_metadata(),
        )
        .expect("valid comparison request"),
    )
    .expect("comparable snapshots");
    assert!(matches!(
        removed.semantic().metadata_change(),
        Some(ContextMetadataChangeV1::Removed { original })
            if original.labels().get("owner").map(String::as_str) == Some("support")
    ));
}

#[test]
fn unknown_metadata_fields_fail_closed_during_deserialization() {
    let mut malformed = serde_json::to_value(snapshot(metadata(1_751_500_000, "support")))
        .expect("serialize snapshot");
    malformed["semantic"]["metadata"]["unexpected"] = json!(true);

    assert!(serde_json::from_value::<ContextDiffSnapshotV1>(malformed).is_err());

    let mut snapshot_with_unknown_nested_field =
        serde_json::to_value(snapshot(metadata(1_751_500_000, "support")))
            .expect("serialize snapshot");
    snapshot_with_unknown_nested_field["semantic"]["unexpected"] = json!(true);
    assert!(
        serde_json::from_value::<ContextDiffSnapshotV1>(snapshot_with_unknown_nested_field)
            .is_err()
    );

    let result = ContextDiffService::compare(
        ContextDiffRequestV1::new(
            snapshot(metadata(1_751_500_000, "support")),
            snapshot(metadata(1_751_500_010, "platform")),
        )
        .expect("valid comparison request"),
    )
    .expect("comparable snapshots");
    let mut result_with_unknown_nested_field =
        serde_json::to_value(result).expect("serialize result");
    result_with_unknown_nested_field["semantic"]["unexpected"] = json!(true);
    assert!(
        serde_json::from_value::<ContextDiffResultV1>(result_with_unknown_nested_field).is_err()
    );
}

#[test]
fn metadata_change_variants_fail_closed_for_unknown_fields() {
    assert_metadata_change_unknown_field_rejected(ContextMetadataChangeV1::Added {
        revised: metadata(1_751_500_010, "platform"),
    });
    assert_metadata_change_unknown_field_rejected(ContextMetadataChangeV1::Removed {
        original: metadata(1_751_500_000, "support"),
    });
    assert_metadata_change_unknown_field_rejected(ContextMetadataChangeV1::Modified {
        original: metadata(1_751_500_000, "support"),
        revised: metadata(1_751_500_010, "platform"),
    });
}

#[test]
fn invalid_metadata_timestamp_order_fails_closed_before_comparison() {
    let original = snapshot(metadata(1_751_500_000, "support"));
    let mut revised = serde_json::to_value(snapshot(metadata(1_751_500_010, "platform")))
        .expect("serialize snapshot");
    revised["semantic"]["metadata"]["updated_at"] = json!("2025-07-01T00:00:00Z");

    let revised: ContextDiffSnapshotV1 =
        serde_json::from_value(revised).expect("timestamp shape remains deserializable");
    let error = ContextDiffRequestV1::new(original, revised)
        .expect_err("invalid metadata ordering must reject the request");

    assert!(matches!(
        error,
        DiffInputError::InvalidContextMetadata { .. }
    ));
}
