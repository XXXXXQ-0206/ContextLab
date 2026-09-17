//! Contract tests for memory timeline, retention, and replay behavior.

use contextlab_memory::{
    Importance, InMemoryMemoryTimeline, MemoryCapabilitySchemaVersion, MemoryError, MemoryEvent,
    MemoryRetentionCapabilityRequest, MemoryRetentionCapabilityRequirement, MemoryScope,
    MemoryWrite, RetentionDecision, RetentionPolicy, TimelineVersion,
};

#[test]
fn timeline_replays_versioned_events_and_records_private_reads() {
    let timeline = InMemoryMemoryTimeline::new();
    let scope = MemoryScope::from_stable_key("conversation:alpha").expect("scope");
    let memory = MemoryWrite::new(
        scope,
        "user-preference",
        "User prefers concise answers.",
        Importance::new(80).expect("importance"),
        false,
    )
    .expect("memory write");

    let created = timeline.append(memory).expect("create event");
    let updated = timeline
        .append(MemoryWrite::update(
            created.memory_id(),
            scope,
            "User prefers concise and direct answers.",
            Importance::new(90).expect("importance"),
            false,
        ))
        .expect("update event");
    let replay = timeline
        .replay(created.memory_id(), scope)
        .expect("valid replay");
    let read = timeline
        .record_read(
            created.memory_id(),
            scope,
            1_725_000_000_000,
            "context-build-v1",
        )
        .expect("privacy-safe read record");

    assert_eq!(replay.version(), TimelineVersion::new(2).expect("version"));
    assert_eq!(
        replay.content_fingerprint(),
        Some(updated.content_fingerprint())
    );
    assert!(!read.contains_raw_content());
    assert!(!read.contains_raw_query());
}

#[test]
fn retention_preserves_pinned_memories_and_expires_low_importance_memories() {
    let policy = RetentionPolicy::new("retention-v1", 1_000, Importance::new(60).expect("score"))
        .expect("policy");
    let pinned = MemoryEvent::created_for_test(
        "memory:pinned",
        "scope:alpha",
        "pinned content",
        Importance::new(1).expect("score"),
        true,
        100,
    )
    .expect("event");
    let stale = MemoryEvent::created_for_test(
        "memory:stale",
        "scope:alpha",
        "stale content",
        Importance::new(1).expect("score"),
        false,
        100,
    )
    .expect("event");

    assert!(policy.should_retain(&pinned, 1_101));
    assert!(!policy.should_retain(&stale, 1_101));
}

#[test]
fn replay_fails_closed_for_scope_mismatch() {
    let timeline = InMemoryMemoryTimeline::new();
    let scope = MemoryScope::from_stable_key("conversation:alpha").expect("scope");
    let created = timeline
        .append(
            MemoryWrite::new(
                scope,
                "fact",
                "Private fact.",
                Importance::new(70).expect("importance"),
                false,
            )
            .expect("write"),
        )
        .expect("create event");

    let error = timeline
        .replay(
            created.memory_id(),
            MemoryScope::from_stable_key("conversation:beta").expect("scope"),
        )
        .expect_err("scope mismatch must fail closed");

    assert!(error.to_string().contains("scope"));
}

#[test]
fn retention_capability_is_deterministic_and_never_exposes_retained_content() {
    let timeline = InMemoryMemoryTimeline::new();
    let scope = MemoryScope::from_stable_key("conversation:capability").expect("scope");
    let created = timeline
        .append(
            MemoryWrite::new(
                scope,
                "short-lived-fact",
                "Only in short-lived memory.",
                Importance::new(20).expect("importance"),
                false,
            )
            .expect("write"),
        )
        .expect("create");
    let policy = RetentionPolicy::new(
        "retention-v1",
        1_000,
        Importance::new(60).expect("importance"),
    )
    .expect("policy");
    let requirement = MemoryRetentionCapabilityRequirement::new(
        MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1")
            .expect("schema version"),
        "retention-v1",
    )
    .expect("requirement");
    let request = MemoryRetentionCapabilityRequest::new(
        created.memory_id(),
        scope,
        2_000,
        "context-build-v1",
        policy,
        requirement.clone(),
    )
    .expect("request");

    let first = timeline
        .record_retention_capability(request.clone())
        .expect("capability");
    let second = timeline
        .record_retention_capability(request)
        .expect("capability");

    assert_eq!(first.id(), second.id());
    assert_eq!(
        first.schema_version().as_str(),
        "memory-retention-capability-v1"
    );
    assert_eq!(first.decision(), RetentionDecision::ExpiredLowImportance);
    assert!(!first.contains_raw_content());
    assert!(!format!("{first:?}").contains("Only in short-lived memory."));

    let incompatible = MemoryRetentionCapabilityRequirement::new(
        MemoryCapabilitySchemaVersion::new("memory-retention-capability-v2")
            .expect("schema version"),
        "retention-v1",
    )
    .expect("requirement");
    let request = MemoryRetentionCapabilityRequest::new(
        created.memory_id(),
        scope,
        2_000,
        "context-build-v1",
        RetentionPolicy::new(
            "retention-v1",
            1_000,
            Importance::new(60).expect("importance"),
        )
        .expect("policy"),
        incompatible,
    )
    .expect("request");
    assert!(matches!(
        timeline.record_retention_capability(request),
        Err(MemoryError::RetentionCapabilityMismatch { .. })
    ));
}

#[test]
fn retention_capability_v1_projects_redacted_replay_facts() {
    let timeline = InMemoryMemoryTimeline::new();
    let scope = MemoryScope::from_stable_key("conversation:replay-projection").expect("scope");
    let created = timeline
        .append(
            MemoryWrite::new(
                scope,
                "preference",
                "The raw memory body must never reach local readers.",
                Importance::new(40).expect("importance"),
                false,
            )
            .expect("write"),
        )
        .expect("create");
    let updated = timeline
        .append(MemoryWrite::update(
            created.memory_id(),
            scope,
            "The updated raw memory body must stay private too.",
            Importance::new(90).expect("importance"),
            true,
        ))
        .expect("update");
    let policy = RetentionPolicy::new(
        "retention-v1",
        1_000,
        Importance::new(60).expect("importance"),
    )
    .expect("policy");
    let requirement = MemoryRetentionCapabilityRequirement::new(
        MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1")
            .expect("schema version"),
        "retention-v1",
    )
    .expect("requirement");
    let capability = timeline
        .record_retention_capability(
            MemoryRetentionCapabilityRequest::new(
                created.memory_id(),
                scope,
                2_000,
                "context-build-v1",
                policy,
                requirement,
            )
            .expect("request"),
        )
        .expect("capability");

    assert_eq!(capability.memory_id(), created.memory_id());
    assert_eq!(capability.scope(), scope);
    assert_eq!(
        capability.timeline_version(),
        TimelineVersion::new(2).expect("version")
    );
    assert_eq!(
        capability.replay_state(),
        contextlab_memory::MemoryReplayState::Active
    );
    assert_eq!(
        capability.content_fingerprint(),
        Some(updated.content_fingerprint())
    );
    assert_eq!(
        capability.importance(),
        Some(Importance::new(90).expect("importance"))
    );
    assert!(capability.pinned());
    assert_eq!(capability.decision(), RetentionDecision::RetainedPinned);
    assert!(!capability.contains_raw_content());

    let debug = format!("{capability:?}");
    assert!(!debug.contains("The raw memory body must never reach local readers."));
    assert!(!debug.contains("The updated raw memory body must stay private too."));
}
