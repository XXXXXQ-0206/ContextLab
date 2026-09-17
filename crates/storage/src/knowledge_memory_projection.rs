//! Immutable exact-commit persistence for redacted Knowledge/Memory projections.

use async_trait::async_trait;
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_knowledge::KnowledgeMemoryContextProjectionV1;
use contextlab_versioning::CommitId;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;
use uuid::Uuid;

/// The only context projection schema accepted by this storage boundary.
pub const KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_SCHEMA_V1: &str =
    "knowledge-memory-context-projection-v1";

/// Exact immutable project, Context, and commit scope for one projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgeMemoryProjectionScope {
    project_id: ProjectId,
    context_id: ContextId,
    context_commit_id: CommitId,
}

impl KnowledgeMemoryProjectionScope {
    /// Creates an exact immutable projection scope.
    #[must_use]
    pub const fn new(
        project_id: ProjectId,
        context_id: ContextId,
        context_commit_id: CommitId,
    ) -> Self {
        Self {
            project_id,
            context_id,
            context_commit_id,
        }
    }

    /// Returns the owning project.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the Context root.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the exact immutable Context commit.
    #[must_use]
    pub const fn context_commit_id(self) -> CommitId {
        self.context_commit_id
    }

    fn key(self) -> (Uuid, Uuid, Uuid) {
        (
            self.project_id.as_uuid(),
            self.context_id.as_uuid(),
            self.context_commit_id.as_uuid(),
        )
    }
}

/// Validated immutable source facts accepted by the writer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistKnowledgeMemoryProjectionV1 {
    scope: KnowledgeMemoryProjectionScope,
    projection: KnowledgeMemoryContextProjectionV1,
}

impl PersistKnowledgeMemoryProjectionV1 {
    /// Binds one redacted Context projection to its exact project and commit.
    pub fn new(
        scope: KnowledgeMemoryProjectionScope,
        projection: KnowledgeMemoryContextProjectionV1,
    ) -> Result<Self, KnowledgeMemoryProjectionPersistenceError> {
        if projection.schema_version().as_str() != KNOWLEDGE_MEMORY_CONTEXT_PROJECTION_SCHEMA_V1 {
            return Err(
                KnowledgeMemoryProjectionPersistenceError::UnsupportedSchema {
                    received: projection.schema_version().as_str().to_owned(),
                },
            );
        }
        if projection.context_id() != scope.context_id() {
            return Err(KnowledgeMemoryProjectionPersistenceError::ScopeMismatch {
                reason: "projection Context does not match the exact storage scope",
            });
        }
        if projection.contains_raw_knowledge_content()
            || projection.contains_raw_memory_content()
            || projection.contains_raw_vectors()
            || projection.contains_raw_query()
            || projection.contains_provider_secrets()
        {
            return Err(KnowledgeMemoryProjectionPersistenceError::RawContentRejected);
        }
        Ok(Self { scope, projection })
    }

    /// Returns the exact immutable scope.
    #[must_use]
    pub const fn scope(&self) -> KnowledgeMemoryProjectionScope {
        self.scope
    }

    /// Returns the validated redacted projection.
    #[must_use]
    pub const fn projection(&self) -> &KnowledgeMemoryContextProjectionV1 {
        &self.projection
    }
}

/// Whether an immutable source was created or replayed unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeMemoryProjectionWriteDisposition {
    /// A new exact source was stored.
    Created,
    /// An identical source already existed.
    Replayed,
}

/// Result of an immutable projection write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnowledgeMemoryProjectionWriteResult {
    scope: KnowledgeMemoryProjectionScope,
    disposition: KnowledgeMemoryProjectionWriteDisposition,
}

impl KnowledgeMemoryProjectionWriteResult {
    pub(crate) const fn new(
        scope: KnowledgeMemoryProjectionScope,
        disposition: KnowledgeMemoryProjectionWriteDisposition,
    ) -> Self {
        Self { scope, disposition }
    }

    /// Returns the exact scope written or replayed.
    #[must_use]
    pub const fn scope(&self) -> KnowledgeMemoryProjectionScope {
        self.scope
    }

    /// Returns the write disposition.
    #[must_use]
    pub const fn disposition(&self) -> KnowledgeMemoryProjectionWriteDisposition {
        self.disposition
    }
}

/// Fail-closed errors from exact Knowledge/Memory persistence and reads.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeMemoryProjectionPersistenceError {
    /// The projection schema is not supported by this storage contract.
    #[error("unsupported Knowledge/Memory projection schema: {received}")]
    UnsupportedSchema {
        /// Received schema version.
        received: String,
    },
    /// Source projection and exact path scope disagree.
    #[error("Knowledge/Memory projection scope mismatch: {reason}")]
    ScopeMismatch {
        /// Stable redacted reason.
        reason: &'static str,
    },
    /// A source attempted to cross the storage privacy boundary.
    #[error("raw Knowledge/Memory projection content is rejected")]
    RawContentRejected,
    /// An immutable scope was reused with different facts.
    #[error("Knowledge/Memory projection conflicts at {scope:?}")]
    Conflict {
        /// Conflicting exact scope.
        scope: KnowledgeMemoryProjectionScope,
    },
    /// No source exists at the requested exact scope.
    #[error("Knowledge/Memory projection is unavailable at {scope:?}")]
    NotFound {
        /// Missing exact scope.
        scope: KnowledgeMemoryProjectionScope,
    },
    /// Stored JSON or source facts failed validation.
    #[error("stored Knowledge/Memory projection is invalid")]
    StoredProjectionInvalid,
    /// The in-memory repository lock could not be trusted.
    #[error("Knowledge/Memory projection repository is unavailable")]
    RepositoryUnavailable,
    /// A database operation failed without exposing connection details.
    #[error("Knowledge/Memory projection database operation failed: {message}")]
    Database {
        /// Redacted database error message.
        message: String,
    },
}

/// Storage port for immutable exact-commit Knowledge/Memory projection sources.
#[async_trait]
pub trait KnowledgeMemoryProjectionV1Repository: Send + Sync {
    /// Persists one validated source or replays an identical source.
    async fn persist_knowledge_memory_projection(
        &self,
        command: PersistKnowledgeMemoryProjectionV1,
    ) -> Result<KnowledgeMemoryProjectionWriteResult, KnowledgeMemoryProjectionPersistenceError>;

    /// Reads one redacted source at an exact project, Context, and commit scope.
    async fn read_knowledge_memory_projection(
        &self,
        scope: KnowledgeMemoryProjectionScope,
    ) -> Result<KnowledgeMemoryContextProjectionV1, KnowledgeMemoryProjectionPersistenceError>;
}

#[derive(Debug, Default)]
struct InMemoryKnowledgeMemoryProjectionState {
    projections: BTreeMap<(Uuid, Uuid, Uuid), KnowledgeMemoryContextProjectionV1>,
}

/// Deterministic in-memory adapter used by local development and contract tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryKnowledgeMemoryProjectionV1Repository {
    state: Arc<RwLock<InMemoryKnowledgeMemoryProjectionState>>,
}

impl InMemoryKnowledgeMemoryProjectionV1Repository {
    /// Creates an empty immutable projection store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KnowledgeMemoryProjectionV1Repository for InMemoryKnowledgeMemoryProjectionV1Repository {
    async fn persist_knowledge_memory_projection(
        &self,
        command: PersistKnowledgeMemoryProjectionV1,
    ) -> Result<KnowledgeMemoryProjectionWriteResult, KnowledgeMemoryProjectionPersistenceError>
    {
        let scope = command.scope();
        let mut state = self
            .state
            .write()
            .map_err(|_| KnowledgeMemoryProjectionPersistenceError::RepositoryUnavailable)?;
        if let Some(existing) = state.projections.get(&scope.key()) {
            if existing == command.projection() {
                return Ok(KnowledgeMemoryProjectionWriteResult::new(
                    scope,
                    KnowledgeMemoryProjectionWriteDisposition::Replayed,
                ));
            }
            return Err(KnowledgeMemoryProjectionPersistenceError::Conflict { scope });
        }
        state
            .projections
            .insert(scope.key(), command.projection().clone());
        Ok(KnowledgeMemoryProjectionWriteResult::new(
            scope,
            KnowledgeMemoryProjectionWriteDisposition::Created,
        ))
    }

    async fn read_knowledge_memory_projection(
        &self,
        scope: KnowledgeMemoryProjectionScope,
    ) -> Result<KnowledgeMemoryContextProjectionV1, KnowledgeMemoryProjectionPersistenceError> {
        let state = self
            .state
            .read()
            .map_err(|_| KnowledgeMemoryProjectionPersistenceError::RepositoryUnavailable)?;
        let projection = state
            .projections
            .get(&scope.key())
            .cloned()
            .ok_or(KnowledgeMemoryProjectionPersistenceError::NotFound { scope })?;
        PersistKnowledgeMemoryProjectionV1::new(scope, projection)
            .map(|command| command.projection().clone())
            .map_err(|_| KnowledgeMemoryProjectionPersistenceError::StoredProjectionInvalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContextCommitRecord, ContextGraphProjection, ContextRecord, ProjectRecord};
    use contextlab_embedding::{DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig};
    use contextlab_knowledge::{
        ChunkingPolicy, InMemoryKnowledgeRepository, KnowledgeIngestion,
        KnowledgeMemoryContextProjectionBridge, KnowledgeQuery, KnowledgeScope, RetrievalRequest,
    };
    use contextlab_memory::{
        Importance, InMemoryMemoryTimeline, MemoryCapabilitySchemaVersion,
        MemoryRetentionCapabilityRequest, MemoryRetentionCapabilityRequirement, MemoryWrite,
        RetentionPolicy,
    };
    use uuid::Uuid;

    fn fixture(
        source_key: &str,
    ) -> (
        KnowledgeMemoryProjectionScope,
        PersistKnowledgeMemoryProjectionV1,
    ) {
        let project_id = ProjectId::from_uuid(Uuid::from_u128(303));
        let context_id = ContextId::from_uuid(Uuid::from_u128(101));
        let commit_id = CommitId::from_uuid(Uuid::from_u128(202));
        let scope = KnowledgeMemoryProjectionScope::new(project_id, context_id, commit_id);
        let knowledge_scope = KnowledgeScope::from_stable_key(format!("context:{context_id}"))
            .expect("knowledge scope");
        let embedding = DeterministicEmbeddingAdapter::new(
            DeterministicEmbeddingConfig::new("contextlab-local-hash", "1", 8)
                .expect("embedding config"),
        );
        let knowledge = InMemoryKnowledgeRepository::new(embedding);
        knowledge
            .ingest(
                KnowledgeIngestion::new(
                    knowledge_scope,
                    source_key,
                    "Projection source",
                    "local-v1",
                    "raw source remains private",
                    ChunkingPolicy::new("characters-v1", 128, 0).expect("chunking"),
                )
                .expect("ingestion"),
            )
            .expect("ingest");
        let citations = knowledge
            .project_local_citation_v1(
                RetrievalRequest::new(
                    KnowledgeQuery::new("inspection").expect("query"),
                    1,
                    "retrieval-v1",
                )
                .expect("request")
                .for_scope(knowledge_scope),
            )
            .expect("citations");
        let memory_scope =
            contextlab_memory::MemoryScope::from_stable_key(format!("context:{context_id}"))
                .expect("memory scope");
        let memory = InMemoryMemoryTimeline::new();
        let created = memory
            .append(
                MemoryWrite::new(
                    memory_scope,
                    "memory-key",
                    "raw memory remains private",
                    Importance::new(80).expect("importance"),
                    true,
                )
                .expect("memory write"),
            )
            .expect("append");
        let policy = RetentionPolicy::new(
            "retention-v1",
            100,
            Importance::new(40).expect("importance"),
        )
        .expect("policy");
        let requirement = MemoryRetentionCapabilityRequirement::new(
            MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1").expect("schema"),
            "retention-v1",
        )
        .expect("requirement");
        let retention = memory
            .record_retention_capability(
                MemoryRetentionCapabilityRequest::new(
                    created.memory_id(),
                    memory_scope,
                    2,
                    "context-build-v1",
                    policy,
                    requirement,
                )
                .expect("retention request"),
            )
            .expect("retention");
        let projection =
            KnowledgeMemoryContextProjectionBridge::project(context_id, &citations, &retention)
                .expect("context projection");
        let command =
            PersistKnowledgeMemoryProjectionV1::new(scope, projection).expect("persist command");
        (scope, command)
    }

    #[tokio::test]
    async fn in_memory_projection_is_exact_immutable_and_replayable() {
        let (scope, command) = fixture("projection-source");
        let repository = InMemoryKnowledgeMemoryProjectionV1Repository::new();
        let created = repository
            .persist_knowledge_memory_projection(command.clone())
            .await
            .expect("create");
        assert_eq!(
            created.disposition(),
            KnowledgeMemoryProjectionWriteDisposition::Created
        );
        let replay = repository
            .persist_knowledge_memory_projection(command.clone())
            .await
            .expect("replay");
        assert_eq!(
            replay.disposition(),
            KnowledgeMemoryProjectionWriteDisposition::Replayed
        );
        let loaded = repository
            .read_knowledge_memory_projection(scope)
            .await
            .expect("read");
        assert_eq!(loaded, *command.projection());
        assert_eq!(loaded.context_id(), scope.context_id());
        assert!(!loaded.contains_raw_knowledge_content());
        assert!(!loaded.contains_raw_memory_content());
        assert!(!loaded.contains_raw_vectors());
        assert!(!loaded.contains_raw_query());
        assert!(!loaded.contains_provider_secrets());
    }

    #[tokio::test]
    async fn in_memory_projection_rejects_scope_drift_and_immutable_conflict() {
        let (scope, command) = fixture("projection-source");
        let repository = InMemoryKnowledgeMemoryProjectionV1Repository::new();
        repository
            .persist_knowledge_memory_projection(command.clone())
            .await
            .expect("create");
        let other_scope = KnowledgeMemoryProjectionScope::new(
            scope.project_id(),
            scope.context_id(),
            CommitId::from_uuid(Uuid::from_u128(404)),
        );
        let drift =
            PersistKnowledgeMemoryProjectionV1::new(other_scope, command.projection().clone())
                .expect("same Context projection can be bound to a separate commit source");
        let conflict = repository
            .persist_knowledge_memory_projection(drift)
            .await
            .expect("different commit is a separate exact source");
        assert_eq!(
            conflict.disposition(),
            KnowledgeMemoryProjectionWriteDisposition::Created
        );
        let (_, altered) = fixture("different-projection-source");
        assert_eq!(
            repository
                .persist_knowledge_memory_projection(altered)
                .await
                .expect_err("immutable exact scope conflict")
                .to_string(),
            format!("Knowledge/Memory projection conflicts at {:?}", scope)
        );
    }

    #[tokio::test]
    async fn context_graph_repository_rejects_projection_for_unknown_commit_scope() {
        let (scope, command) = fixture("projection-source");
        let repository = crate::InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: scope.project_id().to_string(),
                workspace_id: "workspace".to_owned(),
                name: "Projection project".to_owned(),
                slug: "projection-project".to_owned(),
                created_at: chrono::Utc::now(),
            }],
            contexts: vec![ContextRecord {
                id: scope.context_id().to_string(),
                project_id: scope.project_id().to_string(),
                experiment_id: None,
                name: "Projection context".to_owned(),
                description: None,
                created_at: chrono::Utc::now(),
            }],
            commits: vec![ContextCommitRecord {
                id: scope.context_commit_id().to_string(),
                context_id: scope.context_id().to_string(),
                branch_name: "main".to_owned(),
                message: "Projection commit".to_owned(),
                parent_commit_ids: Vec::new(),
                changes: serde_json::json!([]),
                change_count: 0,
                authored_at: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
            }],
            ..ContextGraphProjection::default()
        });
        let unknown_scope = KnowledgeMemoryProjectionScope::new(
            scope.project_id(),
            scope.context_id(),
            CommitId::from_uuid(Uuid::from_u128(404)),
        );
        let command =
            PersistKnowledgeMemoryProjectionV1::new(unknown_scope, command.projection().clone())
                .expect("scope binding remains valid before repository lookup");

        assert!(matches!(
            repository
                .persist_knowledge_memory_projection(command)
                .await,
            Err(KnowledgeMemoryProjectionPersistenceError::NotFound { scope })
                if scope == unknown_scope
        ));
    }
}
