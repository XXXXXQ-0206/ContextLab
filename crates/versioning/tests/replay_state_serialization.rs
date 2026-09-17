#![allow(missing_docs)]

use chrono::{Duration, TimeZone, Utc};
use contextlab_context_core::{
    ComponentId, ContentHash, ContextComponentKind, ContextId, ContextMetadata,
};
use contextlab_versioning::{
    BranchName, CommitId, ContextChange, ContextCommit, ReplayError, ReplayState,
};
use serde_json::{Value, json};
use uuid::Uuid;

fn fixture_time() -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_735_689_600, 0)
        .single()
        .expect("valid fixture timestamp")
}

fn fixture_commit(
    id: u128,
    context_id: ContextId,
    parent_ids: Vec<CommitId>,
    changes: Vec<ContextChange>,
) -> ContextCommit {
    ContextCommit::from_persisted(
        CommitId::from_uuid(Uuid::from_u128(id)),
        context_id,
        BranchName::default(),
        "replay state serialization fixture",
        parent_ids,
        changes,
        fixture_time(),
    )
    .expect("valid fixture commit")
}

fn fixture_state() -> (ReplayState, ComponentId, ComponentId) {
    let context_id = ContextId::from_uuid(Uuid::from_u128(1));
    let source_id = ComponentId::from_uuid(Uuid::from_u128(2));
    let target_id = ComponentId::from_uuid(Uuid::from_u128(3));
    let mut metadata = ContextMetadata::new(fixture_time());
    metadata.set_label("owner", "versioning", fixture_time() + Duration::seconds(1));

    let root = fixture_commit(
        4,
        context_id,
        Vec::new(),
        vec![
            ContextChange::created_context("root"),
            ContextChange::added_component_content_with_details(
                source_id,
                ContextComponentKind::Prompt,
                "System prompt",
                json!({"role": "system", "priority": 1}),
                ContentHash::new("sha256:source").expect("valid source hash"),
                "add source",
            )
            .expect("valid source component change"),
            ContextChange::added_component_content_with_details(
                target_id,
                ContextComponentKind::Knowledge,
                "Knowledge source",
                json!({"locale": "en-US", "retention_days": 30}),
                ContentHash::new("sha256:target").expect("valid target hash"),
                "add target",
            )
            .expect("valid target component change"),
            ContextChange::updated_metadata(metadata, "set context metadata"),
        ],
    );
    let child = fixture_commit(
        5,
        context_id,
        vec![root.id()],
        vec![
            ContextChange::added_uses_relationship(source_id, target_id, "wire Uses")
                .expect("valid Uses relationship change"),
        ],
    );

    (
        ReplayState::from_commits(context_id, &[root, child]).expect("replay fixture history"),
        source_id,
        target_id,
    )
}

fn fixture_json() -> Value {
    fixture_state()
        .0
        .to_json()
        .expect("serialize replay state fixture")
}

#[test]
fn canonical_json_round_trip_preserves_replay_state_facts() {
    let (state, source_id, target_id) = fixture_state();
    assert!(state.is_initialized());
    assert_eq!(state.components().count(), 2);
    assert_eq!(state.relationships().count(), 1);
    assert_eq!(
        state
            .component(source_id)
            .expect("source component")
            .metadata(),
        &json!({"role": "system", "priority": 1})
    );
    assert_eq!(
        state
            .context_metadata()
            .expect("context metadata")
            .metadata()
            .labels()
            .get("owner"),
        Some(&"versioning".to_owned())
    );
    assert_eq!(
        state.relationships().collect::<Vec<_>>()[0].source_component_id,
        source_id
    );
    assert_eq!(
        state.relationships().collect::<Vec<_>>()[0].target_component_id,
        target_id
    );

    let canonical_json = state.to_json().expect("canonical JSON");
    let canonical_text = serde_json::to_string(&canonical_json).expect("canonical JSON text");
    let restored = ReplayState::from_json(
        serde_json::from_str(&canonical_text).expect("parse canonical JSON"),
    )
    .expect("restore canonical JSON");

    assert_eq!(restored, state);
    assert_eq!(
        serde_json::to_string(&restored.to_json().expect("re-serialize restored state"))
            .expect("restored canonical JSON text"),
        canonical_text
    );
}

#[test]
fn v1_snapshot_serializes_with_schema_version_one() {
    let value = fixture_json();

    assert_eq!(value["schema_version"], json!(1));
}

#[test]
fn snapshot_rejects_unknown_outer_fields() {
    let mut value = fixture_json();
    value["unexpected"] = json!(true);

    assert!(matches!(
        ReplayState::from_json(value),
        Err(ReplayError::InvalidSnapshot {
            reason: "invalid replay state JSON envelope"
        })
    ));
}

#[test]
fn snapshot_rejects_unknown_nested_fields() {
    let mut value = fixture_json();
    value["components"][0]["unexpected"] = json!(true);

    assert!(matches!(
        ReplayState::from_json(value),
        Err(ReplayError::InvalidSnapshot {
            reason: "invalid replay state JSON envelope"
        })
    ));
}

#[test]
fn snapshot_restore_rejects_duplicate_component() {
    let mut value = fixture_json();
    let first_component = value["components"][0].clone();
    value["components"]
        .as_array_mut()
        .expect("components array")
        .push(first_component);

    assert!(matches!(
        ReplayState::from_json(value),
        Err(ReplayError::InvalidSnapshot {
            reason: "duplicate component identifier"
        })
    ));
}

#[test]
fn snapshot_restore_rejects_duplicate_relationship() {
    let mut value = fixture_json();
    let first_relationship = value["relationships"][0].clone();
    value["relationships"]
        .as_array_mut()
        .expect("relationships array")
        .push(first_relationship);

    assert!(matches!(
        ReplayState::from_json(value),
        Err(ReplayError::InvalidSnapshot {
            reason: "duplicate relationship"
        })
    ));
}

#[test]
fn snapshot_restore_rejects_missing_relationship_endpoint() {
    let mut value = fixture_json();
    value["relationships"][0]["target_component_id"] =
        json!(ComponentId::from_uuid(Uuid::from_u128(99)).to_string());

    assert!(matches!(
        ReplayState::from_json(value),
        Err(ReplayError::InvalidSnapshot {
            reason: "relationship endpoint is absent from components"
        })
    ));
}

#[test]
fn snapshot_restore_rejects_unsupported_schema_version() {
    let mut value = fixture_json();
    value["schema_version"] = json!(2);

    assert!(matches!(
        ReplayState::from_json(value),
        Err(ReplayError::UnsupportedSchema {
            actual: 2,
            expected: 1
        })
    ));
}
