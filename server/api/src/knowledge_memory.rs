//! Private, provider-free Knowledge/Memory inspection composition.
//!
//! This module adapts the reusable Knowledge and Memory contracts into one
//! deterministic read resource. It intentionally keeps raw bodies inside the
//! domain adapters and exposes only the redacted replay projection.

use contextlab_context_core::{ContextId, ProjectId};
use contextlab_embedding::{DeterministicEmbeddingAdapter, DeterministicEmbeddingConfig};
use contextlab_knowledge::{
    ChunkingPolicy, InMemoryKnowledgeRepository, KnowledgeError, KnowledgeIngestion,
    KnowledgeMemoryContextProjectionBridge, KnowledgeMemoryContextProjectionError,
    KnowledgeMemoryReplayBridgeError, KnowledgeMemoryReplayProjection, KnowledgeQuery,
    KnowledgeScope, RetrievalRequest,
};
use contextlab_memory::{
    Importance, InMemoryMemoryTimeline, MemoryCapabilitySchemaVersion,
    MemoryRetentionCapabilityRequest, MemoryRetentionCapabilityRequirement, MemoryWrite,
    RetentionPolicy,
};
use contextlab_storage::{
    KnowledgeMemoryProjectionPersistenceError, KnowledgeMemoryProjectionScope,
    KnowledgeMemoryProjectionV1Repository, PersistKnowledgeMemoryProjectionV1,
};
use contextlab_versioning::CommitId;
use serde::Serialize;
use thiserror::Error;

/// Stable private local resource schema.
pub const LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1: &str =
    "contextlab.local-knowledge-memory-projection.v1";

/// Errors from the provider-free local Knowledge/Memory projection adapter.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KnowledgeMemoryProjectionError {
    /// A domain contract rejected the deterministic fixture composition.
    #[error("knowledge projection domain contract failed: {0}")]
    Knowledge(String),
    /// A memory contract rejected the deterministic fixture composition.
    #[error("memory projection domain contract failed: {0}")]
    Memory(String),
    /// The existing Knowledge-to-Memory bridge rejected incompatible source facts.
    #[error(transparent)]
    Bridge(#[from] KnowledgeMemoryReplayBridgeError),
    /// A repository returned an invalid or incompatible resource.
    #[error("knowledge/memory projection is invalid: {0}")]
    Invalid(String),
    /// The Context-scoped bridge rejected an exact Context binding.
    #[error(transparent)]
    Context(#[from] KnowledgeMemoryContextProjectionError),
    /// The storage-backed exact-commit source was unavailable or invalid.
    #[error("knowledge/memory projection storage failed: {0}")]
    Storage(String),
}

/// Exact Context/commit scope plus the reusable redacted Knowledge/Memory facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnowledgeMemoryProjectionResource {
    /// Transport schema for the private local read.
    pub schema_version: &'static str,
    /// Exact Context UUID from the request path.
    pub project_id: String,
    /// Exact Context UUID from the request path.
    pub context_id: String,
    /// Exact materialized commit UUID from the request path.
    pub commit_id: String,
    /// Exact project UUID proven by the repository's nested source projection.
    pub source_project_id: String,
    /// Exact materialized commit UUID proven by the repository's nested source projection.
    pub source_commit_id: String,
    /// Domain-owned redacted Knowledge/Memory replay projection.
    pub projection: KnowledgeMemoryReplayProjection,
}

impl KnowledgeMemoryProjectionResource {
    /// Validates the immutable transport envelope against the requested scope.
    pub fn validate_for_scope(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<(), KnowledgeMemoryProjectionError> {
        if self.schema_version != LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1 {
            return Err(KnowledgeMemoryProjectionError::Invalid(
                "unsupported local projection schema".to_owned(),
            ));
        }
        if self.project_id != project_id.as_uuid().to_string()
            || self.context_id != context_id.as_uuid().to_string()
            || self.commit_id != commit_id.as_uuid().to_string()
            || self.source_project_id != project_id.as_uuid().to_string()
            || self.source_commit_id != commit_id.as_uuid().to_string()
        {
            return Err(KnowledgeMemoryProjectionError::Invalid(
                "projection scope does not match request".to_owned(),
            ));
        }
        let expected_knowledge_scope =
            KnowledgeScope::from_stable_key(format!("context:{context_id}"))
                .map_err(knowledge_error)?;
        let expected_memory_scope =
            contextlab_memory::MemoryScope::from_stable_key(format!("context:{context_id}"))
                .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?;
        if self.projection.schema_version().as_str() != "knowledge-memory-local-replay-v2"
            || self.projection.knowledge_scope() != expected_knowledge_scope
            || self.projection.memory_scope() != expected_memory_scope
        {
            return Err(KnowledgeMemoryProjectionError::Invalid(
                "projection domain scope or schema is inconsistent".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Repository boundary used by the private protected GET route.
#[async_trait::async_trait]
pub trait KnowledgeMemoryProjectionRepository: Send + Sync {
    /// Reads one redacted projection for the exact immutable scope.
    async fn project(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<KnowledgeMemoryProjectionResource, KnowledgeMemoryProjectionError>;
}

/// Provider-free local adapter for offline development and deterministic tests.
#[derive(Debug, Clone, Default)]
pub struct InMemoryKnowledgeMemoryProjectionRepository {
    store: contextlab_storage::InMemoryKnowledgeMemoryProjectionV1Repository,
}

#[async_trait::async_trait]
impl KnowledgeMemoryProjectionRepository for InMemoryKnowledgeMemoryProjectionRepository {
    async fn project(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<KnowledgeMemoryProjectionResource, KnowledgeMemoryProjectionError> {
        let scope = KnowledgeMemoryProjectionScope::new(project_id, context_id, commit_id);
        let context_projection = match self.store.read_knowledge_memory_projection(scope).await {
            Ok(projection) => projection,
            Err(KnowledgeMemoryProjectionPersistenceError::NotFound { .. }) => {
                let projection = build_context_projection(project_id, context_id, commit_id)?;
                let command = PersistKnowledgeMemoryProjectionV1::new(scope, projection)
                    .map_err(|error| KnowledgeMemoryProjectionError::Storage(error.to_string()))?;
                self.store
                    .persist_knowledge_memory_projection(command)
                    .await
                    .map_err(|error| KnowledgeMemoryProjectionError::Storage(error.to_string()))?;
                self.store
                    .read_knowledge_memory_projection(scope)
                    .await
                    .map_err(|error| KnowledgeMemoryProjectionError::Storage(error.to_string()))?
            }
            Err(error) => return Err(KnowledgeMemoryProjectionError::Storage(error.to_string())),
        };
        Ok(resource_from_context_projection(
            project_id,
            context_id,
            commit_id,
            context_projection,
        ))
    }
}

/// Adapter that exposes the reusable storage read port through the private API boundary.
#[derive(Debug, Clone)]
pub struct StorageBackedKnowledgeMemoryProjectionRepository<R> {
    repository: R,
}

impl<R> StorageBackedKnowledgeMemoryProjectionRepository<R> {
    /// Wraps a storage repository without changing its ownership or write surface.
    #[must_use]
    pub const fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl<R> KnowledgeMemoryProjectionRepository for StorageBackedKnowledgeMemoryProjectionRepository<R>
where
    R: KnowledgeMemoryProjectionV1Repository,
{
    async fn project(
        &self,
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<KnowledgeMemoryProjectionResource, KnowledgeMemoryProjectionError> {
        let scope = KnowledgeMemoryProjectionScope::new(project_id, context_id, commit_id);
        let projection = self
            .repository
            .read_knowledge_memory_projection(scope)
            .await
            .map_err(|error| KnowledgeMemoryProjectionError::Storage(error.to_string()))?;
        Ok(resource_from_context_projection(
            project_id, context_id, commit_id, projection,
        ))
    }
}

fn resource_from_context_projection(
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
    projection: contextlab_knowledge::KnowledgeMemoryContextProjectionV1,
) -> KnowledgeMemoryProjectionResource {
    let context_key = context_id.as_uuid().to_string();
    let commit_key = commit_id.as_uuid().to_string();
    let project_key = project_id.as_uuid().to_string();
    KnowledgeMemoryProjectionResource {
        schema_version: LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1,
        project_id: project_key.clone(),
        context_id: context_key,
        commit_id: commit_key.clone(),
        source_project_id: project_key,
        source_commit_id: commit_key,
        projection: projection.replay_projection().clone(),
    }
}

fn build_context_projection(
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
) -> Result<contextlab_knowledge::KnowledgeMemoryContextProjectionV1, KnowledgeMemoryProjectionError>
{
    let project_key = project_id.as_uuid().to_string();
    let context_key = context_id.as_uuid().to_string();
    let commit_key = commit_id.as_uuid().to_string();
    let knowledge_scope = KnowledgeScope::from_stable_key(format!("context:{context_key}"))
        .map_err(knowledge_error)?;
    let embedding = DeterministicEmbeddingAdapter::new(
        DeterministicEmbeddingConfig::new("contextlab-local-hash", "1", 8)
            .map_err(|error| KnowledgeMemoryProjectionError::Knowledge(error.to_string()))?,
    );
    let knowledge = InMemoryKnowledgeRepository::new(embedding);
    knowledge
        .ingest(
            KnowledgeIngestion::new(
                knowledge_scope,
                format!(
                    "project:{project_key}:context:{context_key}:commit:{commit_key}:knowledge"
                ),
                "Local Context knowledge source",
                "local-v1",
                "Private source content remains inside the provider-free local adapter.",
                ChunkingPolicy::new("characters-v1", 128, 0).map_err(knowledge_error)?,
            )
            .map_err(knowledge_error)?,
        )
        .map_err(knowledge_error)?;
    let citation_projection = knowledge
        .project_local_citation_v1(
            RetrievalRequest::new(
                KnowledgeQuery::new("local context inspection").map_err(knowledge_error)?,
                1,
                "retrieval-v1",
            )
            .map_err(knowledge_error)?
            .for_scope(knowledge_scope),
        )
        .map_err(knowledge_error)?;

    let memory_scope =
        contextlab_memory::MemoryScope::from_stable_key(format!("context:{context_key}"))
            .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?;
    let memory = InMemoryMemoryTimeline::new();
    let created = memory
        .append(
            MemoryWrite::new(
                memory_scope,
                format!("commit:{commit_key}:memory"),
                "Private memory body remains inside the provider-free local adapter.",
                Importance::new(80)
                    .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?,
                true,
            )
            .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?,
        )
        .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?;
    let policy = RetentionPolicy::new(
        "retention-v1",
        100,
        Importance::new(40)
            .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?,
    )
    .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?;
    let requirement = MemoryRetentionCapabilityRequirement::new(
        MemoryCapabilitySchemaVersion::new("memory-retention-capability-v1")
            .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?,
        "retention-v1",
    )
    .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?;
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
            .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?,
        )
        .map_err(|error| KnowledgeMemoryProjectionError::Memory(error.to_string()))?;

    KnowledgeMemoryContextProjectionBridge::project(context_id, &citation_projection, &retention)
        .map_err(KnowledgeMemoryProjectionError::from)
}

fn knowledge_error(error: KnowledgeError) -> KnowledgeMemoryProjectionError {
    KnowledgeMemoryProjectionError::Knowledge(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_value;
    use uuid::Uuid;

    #[tokio::test]
    async fn local_projection_is_deterministic_and_redacted() {
        let repository = InMemoryKnowledgeMemoryProjectionRepository::default();
        let project_id = ProjectId::from_uuid(Uuid::from_u128(303));
        let context_id = ContextId::from_uuid(Uuid::from_u128(101));
        let commit_id = CommitId::from_uuid(Uuid::from_u128(202));
        let first = repository
            .project(project_id, context_id, commit_id)
            .await
            .expect("projection");
        let second = repository
            .project(project_id, context_id, commit_id)
            .await
            .expect("projection");

        assert_eq!(first, second);
        assert_eq!(
            first.schema_version,
            LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1
        );
        assert_eq!(first.project_id, project_id.as_uuid().to_string());
        assert_eq!(first.context_id, context_id.as_uuid().to_string());
        assert_eq!(first.commit_id, commit_id.as_uuid().to_string());
        assert!(!first.projection.contains_raw_knowledge_content());
        assert!(!first.projection.contains_raw_memory_content());
        assert!(!first.projection.contains_raw_vectors());
        assert!(!first.projection.contains_raw_query());

        let json = to_value(&first).expect("serializable projection");
        let text = json.to_string();
        assert!(!text.contains("Private source content"));
        assert!(!text.contains("Private memory body"));
    }

    #[tokio::test]
    async fn local_projection_serializes_exact_source_identity() {
        let repository = InMemoryKnowledgeMemoryProjectionRepository::default();
        let project_id = ProjectId::from_uuid(Uuid::from_u128(303));
        let context_id = ContextId::from_uuid(Uuid::from_u128(101));
        let commit_id = CommitId::from_uuid(Uuid::from_u128(202));
        let projection = repository
            .project(project_id, context_id, commit_id)
            .await
            .expect("projection");

        let json = to_value(&projection).expect("serializable projection");
        assert_eq!(json["source_project_id"], project_id.as_uuid().to_string());
        assert_eq!(json["source_commit_id"], commit_id.as_uuid().to_string());
    }

    #[tokio::test]
    async fn local_projection_rejects_mismatched_source_project_or_commit_identity() {
        let repository = InMemoryKnowledgeMemoryProjectionRepository::default();
        let project_id = ProjectId::from_uuid(Uuid::from_u128(303));
        let context_id = ContextId::from_uuid(Uuid::from_u128(101));
        let commit_id = CommitId::from_uuid(Uuid::from_u128(202));

        let mut mismatched_project = repository
            .project(project_id, context_id, commit_id)
            .await
            .expect("projection");
        mismatched_project.source_project_id = Uuid::from_u128(404).to_string();
        assert!(matches!(
            mismatched_project.validate_for_scope(project_id, context_id, commit_id),
            Err(KnowledgeMemoryProjectionError::Invalid(message))
                if message == "projection scope does not match request"
        ));

        let mut mismatched_commit = repository
            .project(project_id, context_id, commit_id)
            .await
            .expect("projection");
        mismatched_commit.source_commit_id = Uuid::from_u128(505).to_string();
        assert!(matches!(
            mismatched_commit.validate_for_scope(project_id, context_id, commit_id),
            Err(KnowledgeMemoryProjectionError::Invalid(message))
                if message == "projection scope does not match request"
        ));
    }
}
