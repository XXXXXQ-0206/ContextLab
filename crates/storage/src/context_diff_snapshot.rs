//! Immutable exact-commit persistence for semantic, behavior, and evaluation diff inputs.

use async_trait::async_trait;
use chrono::{DateTime, Timelike, Utc};
use contextlab_diff_engine::{ContextDiffSnapshotV1, VersionedContextScopeV1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Error as SqlxError;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use uuid::Uuid;

/// The only diff-input schema accepted by this storage boundary.
pub const CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1: &str = "context-diff-snapshot-v1";

/// Immutable diff input materialized for one exact Context commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextDiffSnapshotV1Record {
    scope: VersionedContextScopeV1,
    schema_version: String,
    snapshot: ContextDiffSnapshotV1,
    captured_at: DateTime<Utc>,
    snapshot_digest: String,
}

impl ContextDiffSnapshotV1Record {
    pub(crate) fn new(
        scope: VersionedContextScopeV1,
        schema_version: &str,
        snapshot: ContextDiffSnapshotV1,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, ContextDiffSnapshotPersistenceError> {
        validate_scope(scope)?;
        if schema_version != CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1 {
            return Err(ContextDiffSnapshotPersistenceError::UnsupportedSchema {
                received: schema_version.to_owned(),
            });
        }
        let snapshot = validate_snapshot(snapshot)?;
        let snapshot_digest = snapshot_digest(&snapshot)?;
        Ok(Self {
            scope,
            schema_version: schema_version.to_owned(),
            snapshot,
            captured_at: normalize_capture_time(captured_at),
            snapshot_digest,
        })
    }

    /// Returns the exact project/Context/commit scope.
    #[must_use]
    pub const fn scope(&self) -> VersionedContextScopeV1 {
        self.scope
    }

    /// Returns the complete validated diff input.
    #[must_use]
    pub const fn snapshot(&self) -> &ContextDiffSnapshotV1 {
        &self.snapshot
    }

    /// Returns the stable storage schema identifier.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Returns the PostgreSQL-compatible capture timestamp.
    #[must_use]
    pub const fn captured_at(&self) -> DateTime<Utc> {
        self.captured_at
    }

    /// Returns the deterministic digest of the serialized V1 snapshot.
    #[must_use]
    pub fn snapshot_digest(&self) -> &str {
        &self.snapshot_digest
    }

    fn key(&self) -> (Uuid, Uuid, Uuid, String) {
        (
            self.scope.project_id().as_uuid(),
            self.scope.context_id().as_uuid(),
            self.scope.commit_id().as_uuid(),
            self.schema_version.clone(),
        )
    }
}

/// Command to persist one complete diff input at an exact commit.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistContextDiffSnapshotV1 {
    record: ContextDiffSnapshotV1Record,
}

impl PersistContextDiffSnapshotV1 {
    /// Creates a validated immutable persistence command.
    pub fn new(
        scope: VersionedContextScopeV1,
        schema_version: &str,
        snapshot: ContextDiffSnapshotV1,
        captured_at: DateTime<Utc>,
    ) -> Result<Self, ContextDiffSnapshotPersistenceError> {
        Ok(Self {
            record: ContextDiffSnapshotV1Record::new(scope, schema_version, snapshot, captured_at)?,
        })
    }

    /// Returns the immutable record carried by this command.
    #[must_use]
    pub const fn record(&self) -> &ContextDiffSnapshotV1Record {
        &self.record
    }
}

/// Two exact immutable diff snapshots loaded from one repository boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextDiffSnapshotV1Pair {
    source: ContextDiffSnapshotV1Record,
    target: ContextDiffSnapshotV1Record,
}

impl ContextDiffSnapshotV1Pair {
    /// Creates a pair of exact immutable diff snapshots.
    #[must_use]
    pub const fn new(
        source: ContextDiffSnapshotV1Record,
        target: ContextDiffSnapshotV1Record,
    ) -> Self {
        Self { source, target }
    }

    /// Returns the baseline snapshot.
    #[must_use]
    pub const fn source(&self) -> &ContextDiffSnapshotV1Record {
        &self.source
    }

    /// Returns the revised snapshot.
    #[must_use]
    pub const fn target(&self) -> &ContextDiffSnapshotV1Record {
        &self.target
    }
}

/// Whether an immutable source was newly stored or replayed unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDiffSnapshotWriteDisposition {
    /// A new exact source was stored.
    Created,
    /// An identical source already existed.
    Replayed,
}

/// Result of an immutable exact-commit write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextDiffSnapshotWriteResult {
    scope: VersionedContextScopeV1,
    disposition: ContextDiffSnapshotWriteDisposition,
}

impl ContextDiffSnapshotWriteResult {
    pub(crate) const fn new(
        scope: VersionedContextScopeV1,
        disposition: ContextDiffSnapshotWriteDisposition,
    ) -> Self {
        Self { scope, disposition }
    }

    /// Returns the scope written or replayed.
    #[must_use]
    pub const fn scope(self) -> VersionedContextScopeV1 {
        self.scope
    }

    /// Returns the write disposition.
    #[must_use]
    pub const fn disposition(self) -> ContextDiffSnapshotWriteDisposition {
        self.disposition
    }
}

/// Fail-closed errors from exact diff-input persistence and reads.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ContextDiffSnapshotPersistenceError {
    /// The storage schema is unsupported.
    #[error("unsupported Context diff snapshot schema: {received}")]
    UnsupportedSchema {
        /// Received schema identifier.
        received: String,
    },
    /// The snapshot could not be validated by the diff domain.
    #[error("invalid Context diff snapshot: {reason}")]
    InvalidSnapshot {
        /// Validation or serialization detail.
        reason: String,
    },
    /// A scope contains a nil or otherwise unusable identifier.
    #[error("invalid Context diff snapshot scope: {reason}")]
    InvalidScope {
        /// Stable validation reason.
        reason: &'static str,
    },
    /// A stored payload could not be serialized or decoded.
    #[error("Context diff snapshot payload is invalid")]
    SnapshotInvalid,
    /// An exact scope was reused with different immutable facts.
    #[error("Context diff snapshot conflicts at {scope:?}")]
    Conflict {
        /// Conflicting immutable scope.
        scope: VersionedContextScopeV1,
    },
    /// No source exists at the requested exact scope.
    #[error("Context diff snapshot is unavailable at {scope:?}")]
    NotFound {
        /// Missing exact scope.
        scope: VersionedContextScopeV1,
    },
    /// Stored payload or digest failed validation.
    #[error("stored Context diff snapshot is invalid")]
    StoredSnapshotInvalid,
    /// In-memory repository state could not be accessed safely.
    #[error("Context diff snapshot repository is unavailable")]
    RepositoryUnavailable,
    /// Database operation failed without exposing connection details.
    #[error("Context diff snapshot database operation failed: {message}")]
    Database {
        /// Redacted database failure message.
        message: String,
    },
}

/// Storage port for immutable exact-commit diff inputs.
#[async_trait]
pub trait ContextDiffSnapshotV1Repository: Send + Sync {
    /// Persists one validated source or replays an identical source.
    async fn persist_context_diff_snapshot(
        &self,
        command: PersistContextDiffSnapshotV1,
    ) -> Result<ContextDiffSnapshotWriteResult, ContextDiffSnapshotPersistenceError>;

    /// Reads one source at an exact project, Context, and commit scope.
    async fn read_context_diff_snapshot(
        &self,
        scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError>;
}

/// Storage port for one consistent source/target exact-snapshot read.
#[async_trait]
pub trait ContextDiffSnapshotV1PairRepository: Send + Sync {
    /// Reads source and target exact snapshots through one repository boundary.
    async fn read_context_diff_snapshot_pair(
        &self,
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Pair, ContextDiffSnapshotPersistenceError>;
}

/// Combined private repository capability for callers that need both exact single reads and pair reads.
pub trait ContextDiffSnapshotV1ReviewRepository:
    ContextDiffSnapshotV1Repository + ContextDiffSnapshotV1PairRepository
{
}

impl<T> ContextDiffSnapshotV1ReviewRepository for T where
    T: ContextDiffSnapshotV1Repository + ContextDiffSnapshotV1PairRepository
{
}

#[derive(Debug, Default)]
struct InMemoryContextDiffSnapshotState {
    records: BTreeMap<(Uuid, Uuid, Uuid, String), ContextDiffSnapshotV1Record>,
}

/// Deterministic in-memory adapter for local development and contract tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryContextDiffSnapshotV1Repository {
    state: Arc<RwLock<InMemoryContextDiffSnapshotState>>,
}

impl InMemoryContextDiffSnapshotV1Repository {
    /// Creates an empty immutable source store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ContextDiffSnapshotV1Repository for InMemoryContextDiffSnapshotV1Repository {
    async fn persist_context_diff_snapshot(
        &self,
        command: PersistContextDiffSnapshotV1,
    ) -> Result<ContextDiffSnapshotWriteResult, ContextDiffSnapshotPersistenceError> {
        let record = command.record().clone();
        let scope = record.scope();
        let mut state = self
            .state
            .write()
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;
        if let Some(existing) = state.records.get(&record.key()) {
            if existing == &record {
                return Ok(ContextDiffSnapshotWriteResult::new(
                    scope,
                    ContextDiffSnapshotWriteDisposition::Replayed,
                ));
            }
            return Err(ContextDiffSnapshotPersistenceError::Conflict { scope });
        }
        state.records.insert(record.key(), record);
        Ok(ContextDiffSnapshotWriteResult::new(
            scope,
            ContextDiffSnapshotWriteDisposition::Created,
        ))
    }

    async fn read_context_diff_snapshot(
        &self,
        scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError> {
        validate_scope(scope)?;
        let state = self
            .state
            .read()
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;
        let key = (
            scope.project_id().as_uuid(),
            scope.context_id().as_uuid(),
            scope.commit_id().as_uuid(),
            CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1.to_owned(),
        );
        let record = state
            .records
            .get(&key)
            .cloned()
            .ok_or(ContextDiffSnapshotPersistenceError::NotFound { scope })?;
        let restored = decode_context_diff_snapshot(
            scope,
            record.schema_version.clone(),
            serde_json::to_value(record.snapshot())
                .map_err(|_| ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid)?,
            record.snapshot_digest.clone(),
            record.captured_at,
        )?;
        if restored != record {
            return Err(ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid);
        }
        Ok(restored)
    }
}

#[async_trait]
impl ContextDiffSnapshotV1PairRepository for InMemoryContextDiffSnapshotV1Repository {
    async fn read_context_diff_snapshot_pair(
        &self,
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Result<ContextDiffSnapshotV1Pair, ContextDiffSnapshotPersistenceError> {
        validate_scope(source_scope)?;
        validate_scope(target_scope)?;
        let state = self
            .state
            .read()
            .map_err(|_| ContextDiffSnapshotPersistenceError::RepositoryUnavailable)?;

        let source = read_context_diff_snapshot_from_state(&state, source_scope)?;
        let target = read_context_diff_snapshot_from_state(&state, target_scope)?;
        Ok(ContextDiffSnapshotV1Pair::new(source, target))
    }
}

fn read_context_diff_snapshot_from_state(
    state: &InMemoryContextDiffSnapshotState,
    scope: VersionedContextScopeV1,
) -> Result<ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError> {
    let key = (
        scope.project_id().as_uuid(),
        scope.context_id().as_uuid(),
        scope.commit_id().as_uuid(),
        CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1.to_owned(),
    );
    let record = state
        .records
        .get(&key)
        .cloned()
        .ok_or(ContextDiffSnapshotPersistenceError::NotFound { scope })?;
    let restored = decode_context_diff_snapshot(
        scope,
        record.schema_version.clone(),
        serde_json::to_value(record.snapshot())
            .map_err(|_| ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid)?,
        record.snapshot_digest.clone(),
        record.captured_at,
    )?;
    if restored != record {
        return Err(ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid);
    }
    Ok(restored)
}

pub(crate) fn validate_scope(
    scope: VersionedContextScopeV1,
) -> Result<(), ContextDiffSnapshotPersistenceError> {
    if scope.project_id().as_uuid().is_nil() {
        return Err(ContextDiffSnapshotPersistenceError::InvalidScope {
            reason: "project identifier is nil",
        });
    }
    if scope.context_id().as_uuid().is_nil() {
        return Err(ContextDiffSnapshotPersistenceError::InvalidScope {
            reason: "Context identifier is nil",
        });
    }
    if scope.commit_id().as_uuid().is_nil() {
        return Err(ContextDiffSnapshotPersistenceError::InvalidScope {
            reason: "commit identifier is nil",
        });
    }
    Ok(())
}

pub(crate) fn decode_context_diff_snapshot(
    scope: VersionedContextScopeV1,
    schema_version: String,
    snapshot: serde_json::Value,
    expected_digest: String,
    captured_at: DateTime<Utc>,
) -> Result<ContextDiffSnapshotV1Record, ContextDiffSnapshotPersistenceError> {
    let snapshot: ContextDiffSnapshotV1 = serde_json::from_value(snapshot)
        .map_err(|_| ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid)?;
    let record = ContextDiffSnapshotV1Record::new(scope, &schema_version, snapshot, captured_at)?;
    if record.snapshot_digest() != expected_digest {
        return Err(ContextDiffSnapshotPersistenceError::StoredSnapshotInvalid);
    }
    Ok(record)
}

pub(crate) fn context_diff_snapshot_database_error(
    _error: SqlxError,
) -> ContextDiffSnapshotPersistenceError {
    ContextDiffSnapshotPersistenceError::Database {
        message: "database operation failed".to_owned(),
    }
}

fn validate_snapshot(
    snapshot: ContextDiffSnapshotV1,
) -> Result<ContextDiffSnapshotV1, ContextDiffSnapshotPersistenceError> {
    ContextDiffSnapshotV1::new(
        snapshot.semantic().clone(),
        snapshot.behavior().clone(),
        snapshot.evaluation().clone(),
    )
    .map_err(
        |error| ContextDiffSnapshotPersistenceError::InvalidSnapshot {
            reason: error.to_string(),
        },
    )
}

fn snapshot_digest(
    snapshot: &ContextDiffSnapshotV1,
) -> Result<String, ContextDiffSnapshotPersistenceError> {
    let bytes = serde_json::to_vec(snapshot).map_err(|error| {
        ContextDiffSnapshotPersistenceError::InvalidSnapshot {
            reason: format!("snapshot serialization failed: {error}"),
        }
    })?;
    let digest = Sha256::digest(bytes);
    Ok(format!("sha256:{digest:x}"))
}

fn normalize_capture_time(value: DateTime<Utc>) -> DateTime<Utc> {
    value
        .with_nanosecond(value.timestamp_subsec_micros() * 1_000)
        .expect("microsecond timestamp is always valid")
}
