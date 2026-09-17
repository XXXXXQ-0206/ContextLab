//! Provider-free memory timelines, retention, privacy-safe reads, and replay.
//!
//! Memory bodies are accepted at write time only. Public events, replays, and
//! read records carry fingerprints and versioned metadata rather than raw
//! private content, queries, credentials, or provider state.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;

const MEMORY_SCOPE_NAMESPACE: Uuid = Uuid::from_u128(0x16a0_b1b1_6f04_5821_a685_30f5_3ec9_410b);
const MEMORY_ID_NAMESPACE: Uuid = Uuid::from_u128(0x8e91_8ce8_7c73_5126_b4b8_4211_7c59_4d4d);
const MEMORY_EVENT_NAMESPACE: Uuid = Uuid::from_u128(0x39f2_0bd2_0e5c_5a7a_898c_84f2_3670_8325);
const MEMORY_READ_NAMESPACE: Uuid = Uuid::from_u128(0x73fc_6ffb_33e6_5475_ba05_7c5e_92ec_9e07);
const MEMORY_RETENTION_CAPABILITY_NAMESPACE: Uuid =
    Uuid::from_u128(0x9c62_2fb0_d4d1_5caa_9d58_b4af_63d1_11ab);
const MEMORY_RETENTION_CAPABILITY_SCHEMA_VERSION: &str = "memory-retention-capability-v1";
const MAX_KEY_CHARS: usize = 512;
const MAX_CONTENT_CHARS: usize = 1_000_000;
const MAX_VERSION_CHARS: usize = 240;

/// Errors produced by memory domain constructors, timelines, retention, and replay.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MemoryError {
    /// A required field was blank after trimming.
    #[error("{field} must not be empty")]
    Empty {
        /// Stable field name.
        field: &'static str,
    },
    /// A bounded field exceeded its maximum length.
    #[error("{field} must be at most {max} characters")]
    TooLong {
        /// Stable field name.
        field: &'static str,
        /// Maximum accepted character length.
        max: usize,
    },
    /// Importance was outside the inclusive zero-to-one-hundred range.
    #[error("importance must be between 0 and 100")]
    InvalidImportance,
    /// A timeline version must begin at one.
    #[error("timeline version must be at least 1")]
    InvalidTimelineVersion,
    /// A create attempted to overwrite an existing memory identity.
    #[error("memory already exists: {memory_id}")]
    MemoryAlreadyExists {
        /// Conflicting memory identity.
        memory_id: MemoryId,
    },
    /// An update or read referenced no known memory.
    #[error("memory does not exist: {memory_id}")]
    MissingMemory {
        /// Missing memory identity.
        memory_id: MemoryId,
    },
    /// A request attempted to cross a memory scope boundary.
    #[error("memory scope does not match requested scope")]
    ScopeMismatch,
    /// A replay sequence had an unexpected timeline version.
    #[error("memory replay expected version {expected} but found {found}")]
    InvalidReplayVersion {
        /// Expected version.
        expected: TimelineVersion,
        /// Observed version.
        found: TimelineVersion,
    },
    /// A replay sequence started with an invalid operation.
    #[error("memory replay must begin with a create event")]
    ReplayMustStartWithCreate,
    /// A replay attempted to mutate a forgotten memory.
    #[error("memory replay cannot mutate a forgotten memory")]
    MutationAfterForget,
    /// A timeline lock was poisoned and state cannot be trusted.
    #[error("memory timeline state is unavailable")]
    TimelineUnavailable,
    /// A retention capability request declared a policy incompatible with its policy artifact.
    #[error("retention capability requirement does not match the retention policy")]
    RetentionCapabilityMismatch {},
}

macro_rules! stable_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(Uuid);

        impl $name {
            /// Wraps an existing UUID.
            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            /// Returns the underlying UUID.
            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

stable_id!(MemoryId, "Stable UUID v5 identity for one memory timeline.");
stable_id!(
    MemoryEventId,
    "Stable UUID v5 identity for one immutable memory event."
);
stable_id!(
    MemoryReadId,
    "Stable UUID v5 identity for a privacy-safe memory read record."
);
stable_id!(
    MemoryRetentionCapabilityId,
    "Stable UUID v5 identity for one privacy-safe retention capability record."
);

/// Scope boundary for a memory timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoryScope(Uuid);

impl MemoryScope {
    /// Derives a stable scope UUID v5 from an application-owned key.
    pub fn from_stable_key(value: impl AsRef<str>) -> Result<Self, MemoryError> {
        let value = validated_text("memory_scope", value.as_ref().to_owned(), MAX_KEY_CHARS)?;
        Ok(Self(Uuid::new_v5(
            &MEMORY_SCOPE_NAMESPACE,
            value.as_bytes(),
        )))
    }

    /// Wraps an existing persisted scope UUID.
    #[must_use]
    pub const fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Returns the underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl fmt::Display for MemoryScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Monotonic, one-based logical version in a memory timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TimelineVersion(u64);

impl TimelineVersion {
    /// Creates a one-based timeline version.
    pub fn new(value: u64) -> Result<Self, MemoryError> {
        if value == 0 {
            return Err(MemoryError::InvalidTimelineVersion);
        }
        Ok(Self(value))
    }

    /// Returns the raw logical version number.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TimelineVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Compact deterministic importance score from zero through one hundred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Importance(u8);

impl Importance {
    /// Creates an importance score.
    pub fn new(value: u8) -> Result<Self, MemoryError> {
        if value > 100 {
            return Err(MemoryError::InvalidImportance);
        }
        Ok(Self(value))
    }

    /// Returns the score value.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Explicit retention-policy version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RetentionPolicyVersion(String);

impl RetentionPolicyVersion {
    /// Creates a non-empty retention-policy version.
    pub fn new(value: impl Into<String>) -> Result<Self, MemoryError> {
        Ok(Self(validated_text(
            "retention_policy_version",
            value.into(),
            MAX_VERSION_CHARS,
        )?))
    }

    /// Returns the version string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Explicit schema version for a privacy-safe retention capability record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoryCapabilitySchemaVersion(String);

impl MemoryCapabilitySchemaVersion {
    /// Creates a non-empty capability schema version.
    pub fn new(value: impl Into<String>) -> Result<Self, MemoryError> {
        Ok(Self(validated_text(
            "memory_capability_schema_version",
            value.into(),
            MAX_VERSION_CHARS,
        )?))
    }

    /// Returns the exact capability schema version.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact schema and retention-policy compatibility required by one local consumer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRetentionCapabilityRequirement {
    schema_version: MemoryCapabilitySchemaVersion,
    policy_version: RetentionPolicyVersion,
}

impl MemoryRetentionCapabilityRequirement {
    /// Creates a requirement for one retention capability schema and policy version.
    pub fn new(
        schema_version: MemoryCapabilitySchemaVersion,
        policy_version: impl Into<String>,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            schema_version,
            policy_version: RetentionPolicyVersion::new(policy_version)?,
        })
    }

    /// Returns the required capability schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &MemoryCapabilitySchemaVersion {
        &self.schema_version
    }

    /// Returns the required retention policy version.
    #[must_use]
    pub const fn policy_version(&self) -> &RetentionPolicyVersion {
        &self.policy_version
    }
}

/// A versioned deterministic memory-retention policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionPolicy {
    version: RetentionPolicyVersion,
    maximum_age_millis: u64,
    minimum_importance_after_expiry: Importance,
}

impl RetentionPolicy {
    /// Creates a retention policy with explicit version and expiry threshold.
    pub fn new(
        version: impl Into<String>,
        maximum_age_millis: u64,
        minimum_importance_after_expiry: Importance,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            version: RetentionPolicyVersion::new(version)?,
            maximum_age_millis,
            minimum_importance_after_expiry,
        })
    }

    /// Returns the explicit policy version.
    #[must_use]
    pub const fn version(&self) -> &RetentionPolicyVersion {
        &self.version
    }

    /// Evaluates deterministic retention without reading raw memory content.
    #[must_use]
    pub fn decision(&self, event: &MemoryEvent, now_millis: u64) -> RetentionDecision {
        if event.kind == MemoryEventKind::Forgotten {
            return RetentionDecision::ExpiredForgotten;
        }
        if event.pinned {
            return RetentionDecision::RetainedPinned;
        }
        let age = now_millis.saturating_sub(event.occurred_at_millis);
        if age <= self.maximum_age_millis {
            return RetentionDecision::RetainedFresh;
        }
        if event.importance >= self.minimum_importance_after_expiry {
            RetentionDecision::RetainedImportant
        } else {
            RetentionDecision::ExpiredLowImportance
        }
    }

    /// Returns whether retention policy keeps the event.
    #[must_use]
    pub fn should_retain(&self, event: &MemoryEvent, now_millis: u64) -> bool {
        self.decision(event, now_millis).is_retained()
    }
}

/// Deterministic outcome of a retention decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionDecision {
    /// Memory remains because it is pinned.
    RetainedPinned,
    /// Memory remains because it is within its retention age.
    RetainedFresh,
    /// Memory remains because importance meets the configured threshold.
    RetainedImportant,
    /// Memory is expired because it was explicitly forgotten.
    ExpiredForgotten,
    /// Memory is expired because it is stale and below the importance threshold.
    ExpiredLowImportance,
}

impl RetentionDecision {
    /// Returns whether the decision retains the memory.
    #[must_use]
    pub const fn is_retained(self) -> bool {
        matches!(
            self,
            Self::RetainedPinned | Self::RetainedFresh | Self::RetainedImportant
        )
    }
}

/// Validated input for one deterministic, privacy-safe retention capability projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRetentionCapabilityRequest {
    memory_id: MemoryId,
    scope: MemoryScope,
    evaluated_at_millis: u64,
    context_build_version: String,
    policy: RetentionPolicy,
    requirement: MemoryRetentionCapabilityRequirement,
}

impl MemoryRetentionCapabilityRequest {
    /// Creates a request bound to one memory, scope, policy, and capability requirement.
    pub fn new(
        memory_id: MemoryId,
        scope: MemoryScope,
        evaluated_at_millis: u64,
        context_build_version: impl Into<String>,
        policy: RetentionPolicy,
        requirement: MemoryRetentionCapabilityRequirement,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            memory_id,
            scope,
            evaluated_at_millis,
            context_build_version: validated_text(
                "memory_context_build_version",
                context_build_version.into(),
                MAX_VERSION_CHARS,
            )?,
            policy,
            requirement,
        })
    }
}

/// Requested mutation kind for an append-only memory timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MemoryWriteKind {
    Create { stable_key: String },
    Update { memory_id: MemoryId },
    Forget { memory_id: MemoryId },
}

/// Private write request containing raw content only during append.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWrite {
    kind: MemoryWriteKind,
    scope: MemoryScope,
    content: String,
    importance: Importance,
    pinned: bool,
}

impl MemoryWrite {
    /// Creates a new memory with a caller-owned stable key.
    pub fn new(
        scope: MemoryScope,
        stable_key: impl Into<String>,
        content: impl Into<String>,
        importance: Importance,
        pinned: bool,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            kind: MemoryWriteKind::Create {
                stable_key: validated_text("memory_stable_key", stable_key.into(), MAX_KEY_CHARS)?,
            },
            scope,
            content: validated_content(content.into())?,
            importance,
            pinned,
        })
    }

    /// Updates an existing memory with a new immutable event.
    pub fn update(
        memory_id: MemoryId,
        scope: MemoryScope,
        content: impl Into<String>,
        importance: Importance,
        pinned: bool,
    ) -> Self {
        Self {
            kind: MemoryWriteKind::Update { memory_id },
            scope,
            content: content.into(),
            importance,
            pinned,
        }
    }

    /// Appends a tombstone event that prevents later mutations during replay.
    pub fn forget(memory_id: MemoryId, scope: MemoryScope) -> Self {
        Self {
            kind: MemoryWriteKind::Forget { memory_id },
            scope,
            content: String::new(),
            importance: Importance(0),
            pinned: false,
        }
    }

    fn validate(&self) -> Result<(), MemoryError> {
        match self.kind {
            MemoryWriteKind::Forget { .. } => Ok(()),
            MemoryWriteKind::Create { .. } | MemoryWriteKind::Update { .. } => {
                validated_content(self.content.clone()).map(|_| ())
            }
        }
    }
}

/// Immutable event kind stored in an append-only timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryEventKind {
    /// First event in a memory timeline.
    Created,
    /// Content or retention attributes changed.
    Updated,
    /// Timeline was explicitly tombstoned.
    Forgotten,
}

/// Immutable public event that never exposes the raw memory body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryEvent {
    id: MemoryEventId,
    memory_id: MemoryId,
    scope: MemoryScope,
    version: TimelineVersion,
    schema_version: String,
    kind: MemoryEventKind,
    content_fingerprint: String,
    importance: Importance,
    pinned: bool,
    occurred_at_millis: u64,
}

impl MemoryEvent {
    /// Creates a deterministic standalone event for adapter conformance fixtures.
    pub fn created_for_test(
        stable_memory_key: impl AsRef<str>,
        stable_scope_key: impl AsRef<str>,
        content: impl AsRef<str>,
        importance: Importance,
        pinned: bool,
        occurred_at_millis: u64,
    ) -> Result<Self, MemoryError> {
        let scope = MemoryScope::from_stable_key(stable_scope_key)?;
        let stable_memory_key = validated_text(
            "memory_stable_key",
            stable_memory_key.as_ref().to_owned(),
            MAX_KEY_CHARS,
        )?;
        let content = validated_content(content.as_ref().to_owned())?;
        let memory_id = memory_id_for(scope, &stable_memory_key);
        Ok(event_for(
            memory_id,
            scope,
            TimelineVersion::new(1)?,
            EventPayload {
                kind: MemoryEventKind::Created,
                content: &content,
                importance,
                pinned,
                occurred_at_millis,
            },
        ))
    }

    /// Returns the stable event identity.
    #[must_use]
    pub const fn id(&self) -> MemoryEventId {
        self.id
    }

    /// Returns the memory timeline identity.
    #[must_use]
    pub const fn memory_id(&self) -> MemoryId {
        self.memory_id
    }

    /// Returns the scope boundary.
    #[must_use]
    pub const fn scope(&self) -> MemoryScope {
        self.scope
    }

    /// Returns the event logical version.
    #[must_use]
    pub const fn version(&self) -> TimelineVersion {
        self.version
    }

    /// Returns the explicit event schema version.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Returns the event kind.
    #[must_use]
    pub const fn kind(&self) -> MemoryEventKind {
        self.kind
    }

    /// Returns a SHA-256 fingerprint instead of raw memory content.
    #[must_use]
    pub fn content_fingerprint(&self) -> &str {
        &self.content_fingerprint
    }

    /// Returns deterministic importance metadata.
    #[must_use]
    pub const fn importance(&self) -> Importance {
        self.importance
    }

    /// Returns whether the memory is pinned against normal expiry.
    #[must_use]
    pub const fn pinned(&self) -> bool {
        self.pinned
    }

    /// Returns the caller-owned logical timestamp.
    #[must_use]
    pub const fn occurred_at_millis(&self) -> u64 {
        self.occurred_at_millis
    }
}

/// Replay result for a validated memory timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryReplay {
    memory_id: MemoryId,
    scope: MemoryScope,
    version: TimelineVersion,
    state: MemoryReplayState,
    content_fingerprint: Option<String>,
    importance: Option<Importance>,
    pinned: bool,
}

impl MemoryReplay {
    /// Returns the memory identity.
    #[must_use]
    pub const fn memory_id(&self) -> MemoryId {
        self.memory_id
    }

    /// Returns the scope boundary.
    #[must_use]
    pub const fn scope(&self) -> MemoryScope {
        self.scope
    }

    /// Returns the last applied event version.
    #[must_use]
    pub const fn version(&self) -> TimelineVersion {
        self.version
    }

    /// Returns the current replay state.
    #[must_use]
    pub const fn state(&self) -> MemoryReplayState {
        self.state
    }

    /// Returns the active content fingerprint when the memory is not forgotten.
    #[must_use]
    pub fn content_fingerprint(&self) -> Option<&str> {
        self.content_fingerprint.as_deref()
    }

    /// Returns current importance if the memory remains active.
    #[must_use]
    pub const fn importance(&self) -> Option<Importance> {
        self.importance
    }

    /// Returns whether the active memory is pinned.
    #[must_use]
    pub const fn pinned(&self) -> bool {
        self.pinned
    }
}

/// Current state after deterministic replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryReplayState {
    /// The timeline resolves to an active memory.
    Active,
    /// The timeline resolves to a tombstone.
    Forgotten,
}

/// Privacy-safe read record containing fingerprints and versions only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryReadRecord {
    id: MemoryReadId,
    memory_id: MemoryId,
    scope: MemoryScope,
    timeline_version: TimelineVersion,
    read_at_millis: u64,
    read_purpose_version: String,
    content_fingerprint: Option<String>,
}

impl MemoryReadRecord {
    /// Returns the stable read record identity.
    #[must_use]
    pub const fn id(&self) -> MemoryReadId {
        self.id
    }

    /// Returns the read memory identity.
    #[must_use]
    pub const fn memory_id(&self) -> MemoryId {
        self.memory_id
    }

    /// Returns the replayed timeline version.
    #[must_use]
    pub const fn timeline_version(&self) -> TimelineVersion {
        self.timeline_version
    }

    /// Returns the caller-provided read timestamp.
    #[must_use]
    pub const fn read_at_millis(&self) -> u64 {
        self.read_at_millis
    }

    /// Returns the explicit read-purpose version.
    #[must_use]
    pub fn read_purpose_version(&self) -> &str {
        &self.read_purpose_version
    }

    /// Returns only an optional fingerprint, never a raw memory body.
    #[must_use]
    pub fn content_fingerprint(&self) -> Option<&str> {
        self.content_fingerprint.as_deref()
    }

    /// Confirms that raw content is absent by construction.
    #[must_use]
    pub const fn contains_raw_content(&self) -> bool {
        false
    }

    /// Confirms that raw query text is absent by construction.
    #[must_use]
    pub const fn contains_raw_query(&self) -> bool {
        false
    }
}

/// Privacy-safe retention and replay capability record with no retained memory content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRetentionCapability {
    id: MemoryRetentionCapabilityId,
    schema_version: MemoryCapabilitySchemaVersion,
    memory_id: MemoryId,
    scope: MemoryScope,
    timeline_version: TimelineVersion,
    evaluated_at_millis: u64,
    context_build_version: String,
    policy_version: RetentionPolicyVersion,
    decision: RetentionDecision,
    replay_state: MemoryReplayState,
    content_fingerprint: Option<String>,
    importance: Option<Importance>,
    pinned: bool,
}

impl MemoryRetentionCapability {
    /// Returns the deterministic capability record identity.
    #[must_use]
    pub const fn id(&self) -> MemoryRetentionCapabilityId {
        self.id
    }

    /// Returns the explicit capability schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &MemoryCapabilitySchemaVersion {
        &self.schema_version
    }

    /// Returns the stable memory timeline identity.
    #[must_use]
    pub const fn memory_id(&self) -> MemoryId {
        self.memory_id
    }

    /// Returns the memory scope boundary.
    #[must_use]
    pub const fn scope(&self) -> MemoryScope {
        self.scope
    }

    /// Returns the replayed timeline version used for this projection.
    #[must_use]
    pub const fn timeline_version(&self) -> TimelineVersion {
        self.timeline_version
    }

    /// Returns the logical time at which retention was evaluated.
    #[must_use]
    pub const fn evaluated_at_millis(&self) -> u64 {
        self.evaluated_at_millis
    }

    /// Returns the explicit local context-build version.
    #[must_use]
    pub fn context_build_version(&self) -> &str {
        &self.context_build_version
    }

    /// Returns the retention policy version used for this projection.
    #[must_use]
    pub const fn policy_version(&self) -> &RetentionPolicyVersion {
        &self.policy_version
    }

    /// Returns the deterministic retention outcome.
    #[must_use]
    pub const fn decision(&self) -> RetentionDecision {
        self.decision
    }

    /// Returns the current replay state used for the retention decision.
    #[must_use]
    pub const fn replay_state(&self) -> MemoryReplayState {
        self.replay_state
    }

    /// Returns only the replayed content fingerprint, never a raw memory body.
    #[must_use]
    pub fn content_fingerprint(&self) -> Option<&str> {
        self.content_fingerprint.as_deref()
    }

    /// Returns replayed importance when the memory remains active.
    #[must_use]
    pub const fn importance(&self) -> Option<Importance> {
        self.importance
    }

    /// Returns whether the replayed active memory was pinned.
    #[must_use]
    pub const fn pinned(&self) -> bool {
        self.pinned
    }

    /// Confirms raw memory content is absent by construction.
    #[must_use]
    pub const fn contains_raw_content(&self) -> bool {
        false
    }
}

/// Provider-free port for append-only timeline storage and replay.
pub trait MemoryTimelineRepository {
    /// Appends a validated immutable memory event.
    fn append(&self, write: MemoryWrite) -> Result<MemoryEvent, MemoryError>;

    /// Replays a memory timeline inside the requested scope boundary.
    fn replay(&self, memory_id: MemoryId, scope: MemoryScope) -> Result<MemoryReplay, MemoryError>;

    /// Records a privacy-safe memory read without raw content or query text.
    fn record_read(
        &self,
        memory_id: MemoryId,
        scope: MemoryScope,
        read_at_millis: u64,
        read_purpose_version: impl Into<String>,
    ) -> Result<MemoryReadRecord, MemoryError>;
}

/// Provider-free port for deterministic retention capability projections.
pub trait MemoryRetentionCapabilityRepository {
    /// Records a privacy-safe capability projection for one exact memory timeline.
    fn record_retention_capability(
        &self,
        request: MemoryRetentionCapabilityRequest,
    ) -> Result<MemoryRetentionCapability, MemoryError>;
}

/// Deterministic in-memory adapter for append-only memory timelines.
#[derive(Debug, Default)]
pub struct InMemoryMemoryTimeline {
    events: RwLock<BTreeMap<MemoryId, Vec<MemoryEvent>>>,
}

impl InMemoryMemoryTimeline {
    /// Creates an empty timeline adapter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one immutable event with the next logical version.
    pub fn append(&self, write: MemoryWrite) -> Result<MemoryEvent, MemoryError> {
        write.validate()?;
        let mut events = self
            .events
            .write()
            .map_err(|_| MemoryError::TimelineUnavailable)?;
        let memory_id = match &write.kind {
            MemoryWriteKind::Create { stable_key } => memory_id_for(write.scope, stable_key),
            MemoryWriteKind::Update { memory_id } | MemoryWriteKind::Forget { memory_id } => {
                *memory_id
            }
        };
        let timeline = events.entry(memory_id).or_default();
        match write.kind {
            MemoryWriteKind::Create { .. } if !timeline.is_empty() => {
                return Err(MemoryError::MemoryAlreadyExists { memory_id });
            }
            MemoryWriteKind::Create { .. } => {}
            MemoryWriteKind::Update { .. } | MemoryWriteKind::Forget { .. }
                if timeline.is_empty() =>
            {
                return Err(MemoryError::MissingMemory { memory_id });
            }
            MemoryWriteKind::Update { .. } | MemoryWriteKind::Forget { .. } => {
                let replay = replay_events(memory_id, write.scope, timeline)?;
                if replay.state == MemoryReplayState::Forgotten {
                    return Err(MemoryError::MutationAfterForget);
                }
            }
        }
        let version = TimelineVersion::new(
            u64::try_from(timeline.len()).expect("timeline length fits u64") + 1,
        )?;
        let (kind, content, importance, pinned) = match write.kind {
            MemoryWriteKind::Create { .. } => (
                MemoryEventKind::Created,
                write.content,
                write.importance,
                write.pinned,
            ),
            MemoryWriteKind::Update { .. } => (
                MemoryEventKind::Updated,
                write.content,
                write.importance,
                write.pinned,
            ),
            MemoryWriteKind::Forget { .. } => (
                MemoryEventKind::Forgotten,
                String::new(),
                Importance(0),
                false,
            ),
        };
        let event = event_for(
            memory_id,
            write.scope,
            version,
            EventPayload {
                kind,
                content: &content,
                importance,
                pinned,
                occurred_at_millis: version.get(),
            },
        );
        timeline.push(event.clone());
        Ok(event)
    }

    /// Replays a timeline inside the provided scope boundary.
    pub fn replay(
        &self,
        memory_id: MemoryId,
        scope: MemoryScope,
    ) -> Result<MemoryReplay, MemoryError> {
        let events = self
            .events
            .read()
            .map_err(|_| MemoryError::TimelineUnavailable)?;
        let Some(timeline) = events.get(&memory_id) else {
            return Err(MemoryError::MissingMemory { memory_id });
        };
        replay_events(memory_id, scope, timeline)
    }

    /// Records one privacy-safe read after fail-closed replay validation.
    pub fn record_read(
        &self,
        memory_id: MemoryId,
        scope: MemoryScope,
        read_at_millis: u64,
        read_purpose_version: impl Into<String>,
    ) -> Result<MemoryReadRecord, MemoryError> {
        let read_purpose_version = validated_text(
            "memory_read_purpose_version",
            read_purpose_version.into(),
            MAX_VERSION_CHARS,
        )?;
        let replay = self.replay(memory_id, scope)?;
        let identity = format!(
            "memory-read-v1\\0{}\\0{}\\0{}\\0{}\\0{}",
            memory_id,
            replay.version,
            read_at_millis,
            read_purpose_version,
            replay.content_fingerprint.as_deref().unwrap_or("forgotten")
        );
        Ok(MemoryReadRecord {
            id: MemoryReadId(Uuid::new_v5(&MEMORY_READ_NAMESPACE, identity.as_bytes())),
            memory_id,
            scope,
            timeline_version: replay.version,
            read_at_millis,
            read_purpose_version,
            content_fingerprint: replay.content_fingerprint,
        })
    }

    /// Records a deterministic retention capability without exposing raw memory content.
    pub fn record_retention_capability(
        &self,
        request: MemoryRetentionCapabilityRequest,
    ) -> Result<MemoryRetentionCapability, MemoryError> {
        if request.requirement.schema_version.as_str() != MEMORY_RETENTION_CAPABILITY_SCHEMA_VERSION
            || request.requirement.policy_version != *request.policy.version()
        {
            return Err(MemoryError::RetentionCapabilityMismatch {});
        }

        let events = self
            .events
            .read()
            .map_err(|_| MemoryError::TimelineUnavailable)?;
        let Some(timeline) = events.get(&request.memory_id) else {
            return Err(MemoryError::MissingMemory {
                memory_id: request.memory_id,
            });
        };
        let replay = replay_events(request.memory_id, request.scope, timeline)?;
        let event = timeline
            .last()
            .expect("a timeline with a replay result always has an event");
        let decision = request.policy.decision(event, request.evaluated_at_millis);
        let identity = format!(
            "memory-retention-capability-v1\\0{}\\0{}\\0{}\\0{}\\0{}\\0{}\\0{:?}",
            request.memory_id,
            replay.version,
            request.requirement.schema_version.as_str(),
            request.policy.version().as_str(),
            request.evaluated_at_millis,
            request.context_build_version,
            decision,
        );

        Ok(MemoryRetentionCapability {
            id: MemoryRetentionCapabilityId(Uuid::new_v5(
                &MEMORY_RETENTION_CAPABILITY_NAMESPACE,
                identity.as_bytes(),
            )),
            schema_version: request.requirement.schema_version,
            memory_id: request.memory_id,
            scope: request.scope,
            timeline_version: replay.version,
            evaluated_at_millis: request.evaluated_at_millis,
            context_build_version: request.context_build_version,
            policy_version: request.policy.version,
            decision,
            replay_state: replay.state,
            content_fingerprint: replay.content_fingerprint,
            importance: replay.importance,
            pinned: replay.pinned,
        })
    }
}

impl MemoryTimelineRepository for InMemoryMemoryTimeline {
    fn append(&self, write: MemoryWrite) -> Result<MemoryEvent, MemoryError> {
        Self::append(self, write)
    }

    fn replay(&self, memory_id: MemoryId, scope: MemoryScope) -> Result<MemoryReplay, MemoryError> {
        Self::replay(self, memory_id, scope)
    }

    fn record_read(
        &self,
        memory_id: MemoryId,
        scope: MemoryScope,
        read_at_millis: u64,
        read_purpose_version: impl Into<String>,
    ) -> Result<MemoryReadRecord, MemoryError> {
        Self::record_read(self, memory_id, scope, read_at_millis, read_purpose_version)
    }
}

impl MemoryRetentionCapabilityRepository for InMemoryMemoryTimeline {
    fn record_retention_capability(
        &self,
        request: MemoryRetentionCapabilityRequest,
    ) -> Result<MemoryRetentionCapability, MemoryError> {
        Self::record_retention_capability(self, request)
    }
}

fn memory_id_for(scope: MemoryScope, stable_key: &str) -> MemoryId {
    MemoryId(Uuid::new_v5(
        &MEMORY_ID_NAMESPACE,
        format!("{}\\0{}", scope, stable_key).as_bytes(),
    ))
}

struct EventPayload<'a> {
    kind: MemoryEventKind,
    content: &'a str,
    importance: Importance,
    pinned: bool,
    occurred_at_millis: u64,
}

fn event_for(
    memory_id: MemoryId,
    scope: MemoryScope,
    version: TimelineVersion,
    payload: EventPayload<'_>,
) -> MemoryEvent {
    let content_fingerprint = if payload.kind == MemoryEventKind::Forgotten {
        String::new()
    } else {
        sha256_fingerprint(payload.content.as_bytes())
    };
    let identity = format!(
        "memory-event-v1\\0{}\\0{}\\0{:?}\\0{}\\0{}\\0{}\\0{}",
        memory_id,
        version,
        payload.kind,
        content_fingerprint,
        payload.importance.get(),
        payload.pinned,
        payload.occurred_at_millis
    );
    MemoryEvent {
        id: MemoryEventId(Uuid::new_v5(&MEMORY_EVENT_NAMESPACE, identity.as_bytes())),
        memory_id,
        scope,
        version,
        schema_version: "memory-event-v1".to_owned(),
        kind: payload.kind,
        content_fingerprint,
        importance: payload.importance,
        pinned: payload.pinned,
        occurred_at_millis: payload.occurred_at_millis,
    }
}

fn replay_events(
    memory_id: MemoryId,
    scope: MemoryScope,
    events: &[MemoryEvent],
) -> Result<MemoryReplay, MemoryError> {
    let Some(first) = events.first() else {
        return Err(MemoryError::MissingMemory { memory_id });
    };
    if first.scope != scope {
        return Err(MemoryError::ScopeMismatch);
    }
    let mut expected = TimelineVersion::new(1)?;
    let mut state = MemoryReplayState::Active;
    let mut content_fingerprint = None;
    let mut importance = None;
    let mut pinned = false;

    for event in events {
        if event.scope != scope {
            return Err(MemoryError::ScopeMismatch);
        }
        if event.version != expected {
            return Err(MemoryError::InvalidReplayVersion {
                expected,
                found: event.version,
            });
        }
        if event.version.get() == 1 && event.kind != MemoryEventKind::Created {
            return Err(MemoryError::ReplayMustStartWithCreate);
        }
        if state == MemoryReplayState::Forgotten {
            return Err(MemoryError::MutationAfterForget);
        }
        match event.kind {
            MemoryEventKind::Created | MemoryEventKind::Updated => {
                content_fingerprint = Some(event.content_fingerprint.clone());
                importance = Some(event.importance);
                pinned = event.pinned;
            }
            MemoryEventKind::Forgotten => {
                state = MemoryReplayState::Forgotten;
                content_fingerprint = None;
                importance = None;
                pinned = false;
            }
        }
        expected = TimelineVersion::new(event.version.get() + 1)?;
    }
    let version = events.last().expect("non-empty timeline").version;
    Ok(MemoryReplay {
        memory_id,
        scope,
        version,
        state,
        content_fingerprint,
        importance,
        pinned,
    })
}

fn validated_text(
    field: &'static str,
    value: String,
    maximum_characters: usize,
) -> Result<String, MemoryError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(MemoryError::Empty { field });
    }
    if value.chars().count() > maximum_characters {
        return Err(MemoryError::TooLong {
            field,
            max: maximum_characters,
        });
    }
    Ok(value)
}

fn validated_content(value: String) -> Result<String, MemoryError> {
    if value.trim().is_empty() {
        return Err(MemoryError::Empty {
            field: "memory_content",
        });
    }
    if value.chars().count() > MAX_CONTENT_CHARS {
        return Err(MemoryError::TooLong {
            field: "memory_content",
            max: MAX_CONTENT_CHARS,
        });
    }
    Ok(value)
}

fn sha256_fingerprint(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    format!("sha256:{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_ids_are_stable_for_equal_immutable_inputs() {
        let first = MemoryEvent::created_for_test(
            "memory",
            "scope",
            "content",
            Importance::new(50).expect("importance"),
            false,
            12,
        )
        .expect("event");
        let second = MemoryEvent::created_for_test(
            "memory",
            "scope",
            "content",
            Importance::new(50).expect("importance"),
            false,
            12,
        )
        .expect("event");

        assert_eq!(first.id(), second.id());
    }

    #[test]
    fn replay_rejects_a_timeline_with_a_missing_version() {
        let scope = MemoryScope::from_stable_key("scope").expect("scope");
        let event = MemoryEvent::created_for_test(
            "memory",
            "scope",
            "content",
            Importance::new(50).expect("importance"),
            false,
            1,
        )
        .expect("event");
        let invalid = MemoryEvent {
            version: TimelineVersion::new(3).expect("version"),
            ..event.clone()
        };

        let error = replay_events(event.memory_id(), scope, &[event, invalid])
            .expect_err("gap must fail closed");
        assert!(matches!(error, MemoryError::InvalidReplayVersion { .. }));
    }
}
