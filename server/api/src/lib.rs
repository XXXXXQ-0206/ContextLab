//! ContextLab REST API composition.

mod benchmark_definition_authoring;
mod benchmark_execution;
mod knowledge_memory;
mod local_branch_heads;
mod plugin_capability;
mod routes;
mod workflow_context_bindings;
mod workflow_execution;
mod workflow_status;

use axum::{Router, middleware};
use contextlab_auth::{
    AuthenticatedPrincipal, AuthenticationError, AuthorizationAuditError, AuthorizationAuditEvent,
    AuthorizationAuditSink, ContextAuthorizer, DenyAllContextAuthorizer, HmacJwtAuthenticator,
    HttpsJwksSource, InMemoryProtectedRouteRateLimiter, NoopAuthorizationAuditSink,
    OidcJwksAuthenticator, OidcJwksConfig, PrincipalAuthenticator, ProtectedRouteRateLimitKey,
    ProtectedRouteRateLimitPolicy, ProtectedRouteRateLimiter, RateLimitDecision, RateLimitError,
    RoleBasedContextAuthorizer,
};
use contextlab_model_gateway::ProviderRegistry;
use contextlab_storage::{
    BenchmarkDecisionComparisonScope, BenchmarkDecisionDiscoveryRepository, BenchmarkDecisionPair,
    BenchmarkDefinitionBindingCommand, BenchmarkDefinitionBindingRepository,
    BenchmarkDefinitionBindingWriteResult, BenchmarkDefinitionBindingWriter,
    BenchmarkEvidenceRepository, BenchmarkWorkspaceProjectionDecisionQuery,
    BenchmarkWorkspaceProjectionV1Reader, CommitGraphSnapshotRepository, CommitGraphSnapshotScope,
    ComponentContentRevisionRepository, ComponentStateAtCommitRepository, ContextBranchRepository,
    ContextCommitGraphRepository, ContextCommitHistoryRepository, ContextCommitRepository,
    ContextComponentRepository, ContextComponentStateSnapshotAtCommitRepository,
    ContextDiffSnapshotV1ReviewRepository, ContextGraphProjectionRepository,
    ContextGraphReviewWitnessRepository, ContextLifecycleReadFacts, ContextLifecycleReadRepository,
    ContextLifecycleRepository, ContextLifecycleRoot, ContextLifecycleRootRepository,
    ContextMergeInputScope, ContextMergeReviewWitness, ContextMergeReviewWitnessRepository,
    ContextMergeReviewWitnessRepositoryError, ContextMergeTipScope,
    ContextReplayStateAtCommitRepository, ContextRepository, ContextWorkflowBindingRepository,
    EvaluationRunRepository, ExperimentRepository, GuardedContextCommitWriter,
    InMemoryContextDiffSnapshotV1Repository, InMemoryContextGraphRepository,
    KnowledgeMemoryProjectionPersistenceError, KnowledgeMemoryProjectionScope,
    KnowledgeMemoryProjectionV1Repository, KnowledgeMemoryProjectionWriteResult,
    PersistKnowledgeMemoryProjectionV1, PostgresContextGraphRepository, ProjectRepository,
    StorageRepositoryError, WorkspaceRepository,
};
use plugin_capability::{
    EmptyPluginCapabilityAvailabilityRepository, PluginCapabilityAvailabilityRepository,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use thiserror::Error;

/// Graph-backed repositories required to assemble workspace application state.
pub struct WorkspaceGraphRepositories {
    workspace_graph_repository: Arc<dyn ContextGraphProjectionRepository>,
    commit_graph_snapshot_repository: Arc<dyn CommitGraphSnapshotRepository>,
    context_branch_repository: Arc<dyn ContextBranchRepository>,
    context_merge_review_witness_repository: Option<Arc<dyn ContextMergeReviewWitnessRepository>>,
}

impl WorkspaceGraphRepositories {
    /// Groups the graph projections used by workspace routes.
    #[must_use]
    pub fn new(
        workspace_graph_repository: impl ContextGraphProjectionRepository + 'static,
        commit_graph_snapshot_repository: impl CommitGraphSnapshotRepository + 'static,
        context_branch_repository: impl ContextBranchRepository + 'static,
    ) -> Self {
        Self {
            workspace_graph_repository: Arc::new(workspace_graph_repository),
            commit_graph_snapshot_repository: Arc::new(commit_graph_snapshot_repository),
            context_branch_repository: Arc::new(context_branch_repository),
            context_merge_review_witness_repository: None,
        }
    }

    /// Adds the private atomic server-owned merge review witness repository.
    #[must_use]
    pub fn with_context_merge_review_witness_repository(
        mut self,
        repository: impl ContextMergeReviewWitnessRepository + 'static,
    ) -> Self {
        self.context_merge_review_witness_repository = Some(Arc::new(repository));
        self
    }
}

/// List and detail repositories required to assemble workspace application state.
pub struct WorkspaceCatalogRepositories {
    workspace_repository: Arc<dyn WorkspaceRepository>,
    project_repository: Arc<dyn ProjectRepository>,
    experiment_repository: Arc<dyn ExperimentRepository>,
    context_repository: Arc<dyn ContextRepository>,
    commit_repository: Arc<dyn ContextCommitRepository>,
    commit_graph_repository: Arc<dyn ContextCommitGraphRepository>,
    component_repository: Arc<dyn ContextComponentRepository>,
    evaluation_run_repository: Arc<dyn EvaluationRunRepository>,
}

impl WorkspaceCatalogRepositories {
    /// Groups catalog repositories used by workspace read routes.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace_repository: impl WorkspaceRepository + 'static,
        project_repository: impl ProjectRepository + 'static,
        experiment_repository: impl ExperimentRepository + 'static,
        context_repository: impl ContextRepository + 'static,
        commit_repository: impl ContextCommitRepository + 'static,
        commit_graph_repository: impl ContextCommitGraphRepository + 'static,
        component_repository: impl ContextComponentRepository + 'static,
        evaluation_run_repository: impl EvaluationRunRepository + 'static,
    ) -> Self {
        Self {
            workspace_repository: Arc::new(workspace_repository),
            project_repository: Arc::new(project_repository),
            experiment_repository: Arc::new(experiment_repository),
            context_repository: Arc::new(context_repository),
            commit_repository: Arc::new(commit_repository),
            commit_graph_repository: Arc::new(commit_graph_repository),
            component_repository: Arc::new(component_repository),
            evaluation_run_repository: Arc::new(evaluation_run_repository),
        }
    }
}

/// Typed input for assembling workspace application state.
pub struct WorkspaceRepositories {
    graph: WorkspaceGraphRepositories,
    catalog: WorkspaceCatalogRepositories,
    context_commit_history_repository: Option<Arc<dyn ContextCommitHistoryRepository>>,
}

impl WorkspaceRepositories {
    /// Combines graph and catalog repository groups for application state construction.
    #[must_use]
    pub fn new(graph: WorkspaceGraphRepositories, catalog: WorkspaceCatalogRepositories) -> Self {
        Self {
            graph,
            catalog,
            context_commit_history_repository: None,
        }
    }

    /// Adds the backend-owned complete Context commit-history reader.
    #[must_use]
    pub fn with_context_commit_history_repository(
        mut self,
        repository: impl ContextCommitHistoryRepository + 'static,
    ) -> Self {
        self.context_commit_history_repository = Some(Arc::new(repository));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkflowExecutionStatusRepositoryBackend {
    Unavailable,
    Custom,
    StorageBacked,
}

#[derive(Clone, Copy)]
struct UnavailableContextCommitHistoryRepository;

#[async_trait::async_trait]
impl ContextCommitHistoryRepository for UnavailableContextCommitHistoryRepository {
    async fn load_context_commit_history(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<contextlab_versioning::CommitHistory, StorageRepositoryError> {
        Err(StorageRepositoryError::ScopeUnavailable {
            scope: format!("context_commit_history:{context_id}"),
        })
    }
}

/// Shared API application state.
#[derive(Clone)]
pub struct AppState {
    provider_registry: ProviderRegistry,
    preview_graph_repository: Arc<dyn ContextGraphProjectionRepository>,
    workspace_graph_repository: Arc<dyn ContextGraphProjectionRepository>,
    workspace_repository: Arc<dyn WorkspaceRepository>,
    project_repository: Arc<dyn ProjectRepository>,
    experiment_repository: Arc<dyn ExperimentRepository>,
    context_repository: Arc<dyn ContextRepository>,
    commit_repository: Arc<dyn ContextCommitRepository>,
    commit_history_repository: Arc<dyn ContextCommitHistoryRepository>,
    context_graph_review_witness_repository: Option<Arc<dyn ContextGraphReviewWitnessRepository>>,
    context_merge_review_witness_repository: Option<Arc<dyn ContextMergeReviewWitnessRepository>>,
    commit_graph_repository: Arc<dyn ContextCommitGraphRepository>,
    commit_graph_snapshot_repository: Arc<dyn CommitGraphSnapshotRepository>,
    context_branch_repository: Arc<dyn ContextBranchRepository>,
    context_diff_snapshot_repository: Option<Arc<dyn ContextDiffSnapshotV1ReviewRepository>>,
    component_repository: Arc<dyn ContextComponentRepository>,
    evaluation_run_repository: Arc<dyn EvaluationRunRepository>,
    context_authorizer: Arc<dyn ContextAuthorizer>,
    authorization_audit_sink: Arc<dyn AuthorizationAuditSink>,
    guarded_commit_writer: Arc<dyn GuardedContextCommitWriter>,
    context_lifecycle_repository: Option<Arc<dyn ContextLifecycleRepository>>,
    context_workflow_binding_repository: Option<Arc<dyn ContextWorkflowBindingRepository>>,
    workflow_execution_status_repository:
        Option<Arc<dyn workflow_execution::WorkflowExecutionStatusRepository>>,
    workflow_execution_status_repository_backend: WorkflowExecutionStatusRepositoryBackend,
    benchmark_evidence_repository: Option<Arc<dyn BenchmarkEvidenceRepository>>,
    benchmark_decision_discovery_repository: Option<Arc<dyn BenchmarkDecisionDiscoveryRepository>>,
    benchmark_workspace_projection_repository:
        Option<Arc<dyn BenchmarkWorkspaceProjectionV1Reader>>,
    benchmark_definition_binding_writer: Option<Arc<dyn BenchmarkDefinitionBindingWriter>>,
    benchmark_definition_binding_repository: Option<Arc<dyn BenchmarkDefinitionBindingRepository>>,
    benchmark_execution_adapter: Option<Arc<dyn benchmark_execution::BenchmarkExecutionAdapter>>,
    knowledge_memory_projection_repository:
        Option<Arc<dyn knowledge_memory::KnowledgeMemoryProjectionRepository>>,
    plugin_capability_availability_repository: Arc<dyn PluginCapabilityAvailabilityRepository>,
    jwt_authenticator: Option<Arc<dyn PrincipalAuthenticator>>,
    protected_rate_limiter: Option<Arc<dyn ProtectedRouteRateLimiter>>,
    postgres_runtime_repository: Option<PostgresContextGraphRepository>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppState")
            .field("provider_registry", &self.provider_registry)
            .field(
                "preview_graph_repository",
                &"ContextGraphProjectionRepository",
            )
            .field(
                "workspace_graph_repository",
                &"ContextGraphProjectionRepository",
            )
            .field("workspace_repository", &"WorkspaceRepository")
            .field("project_repository", &"ProjectRepository")
            .field("experiment_repository", &"ExperimentRepository")
            .field("context_repository", &"ContextRepository")
            .field("commit_repository", &"ContextCommitRepository")
            .field(
                "commit_history_repository",
                &"ContextCommitHistoryRepository",
            )
            .field(
                "context_graph_review_witness_repository",
                &self.context_graph_review_witness_repository.is_some(),
            )
            .field("commit_graph_repository", &"ContextCommitGraphRepository")
            .field(
                "commit_graph_snapshot_repository",
                &"CommitGraphSnapshotRepository",
            )
            .field("context_branch_repository", &"ContextBranchRepository")
            .field(
                "context_diff_snapshot_repository",
                &self.context_diff_snapshot_repository.is_some(),
            )
            .field("component_repository", &"ContextComponentRepository")
            .field("evaluation_run_repository", &"EvaluationRunRepository")
            .field("context_authorizer", &"ContextAuthorizer")
            .field("authorization_audit_sink", &"AuthorizationAuditSink")
            .field("guarded_commit_writer", &"GuardedContextCommitWriter")
            .field(
                "context_lifecycle_repository",
                &self.context_lifecycle_repository.is_some(),
            )
            .field(
                "context_workflow_binding_repository",
                &self.context_workflow_binding_repository.is_some(),
            )
            .field(
                "workflow_execution_status_repository",
                &self.workflow_execution_status_repository.is_some(),
            )
            .field(
                "workflow_execution_status_repository_backend",
                &self.workflow_execution_status_repository_backend,
            )
            .field(
                "benchmark_evidence_repository",
                &self.benchmark_evidence_repository.is_some(),
            )
            .field(
                "benchmark_decision_discovery_repository",
                &self.benchmark_decision_discovery_repository.is_some(),
            )
            .field(
                "benchmark_workspace_projection_repository",
                &self.benchmark_workspace_projection_repository.is_some(),
            )
            .field(
                "benchmark_definition_binding_writer",
                &self.benchmark_definition_binding_writer.is_some(),
            )
            .field(
                "benchmark_definition_binding_repository",
                &self.benchmark_definition_binding_repository.is_some(),
            )
            .field(
                "benchmark_execution_adapter",
                &self.benchmark_execution_adapter.is_some(),
            )
            .field(
                "knowledge_memory_projection_repository",
                &self.knowledge_memory_projection_repository.is_some(),
            )
            .field("plugin_capability_availability_repository", &true)
            .field("jwt_authenticator", &self.jwt_authenticator.is_some())
            .field(
                "protected_rate_limiter",
                &self.protected_rate_limiter.is_some(),
            )
            .field(
                "postgres_runtime_repository",
                &self.postgres_runtime_repository.is_some(),
            )
            .finish()
    }
}

impl AppState {
    /// Builds application state from the current process environment.
    #[must_use]
    pub fn from_current_env() -> Self {
        Self::try_from_current_env().expect("valid ContextLab API runtime configuration")
    }

    /// Tries to build application state from the current process environment.
    pub fn try_from_current_env() -> Result<Self, AppStateConfigError> {
        Self::try_from_env(std::env::vars())
    }

    /// Tries to build application state from explicit environment pairs.
    pub fn try_from_env<I, K, V>(env: I) -> Result<Self, AppStateConfigError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let env = env
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect::<Vec<_>>();
        let provider_registry = ProviderRegistry::from_env(env.iter().cloned());
        let workspace_data_repository = workspace_data_repository_from_env(&env)?;
        let authorization_audit_sink =
            workspace_data_repository.postgres_authorization_audit_sink();
        let postgres_runtime_repository = workspace_data_repository.postgres_runtime_repository();
        let context_diff_snapshot_repository =
            workspace_data_repository.context_diff_snapshot_repository();
        let knowledge_memory_repository = workspace_data_repository.clone();
        let lifecycle_repository = workspace_data_repository.clone();
        let workflow_binding_repository = workspace_data_repository.clone();
        let benchmark_evidence_repository = workspace_data_repository.clone();
        let benchmark_definition_binding_writer = workspace_data_repository.clone();
        let benchmark_definition_binding_repository = workspace_data_repository.clone();
        let commit_history_repository = workspace_data_repository.clone();
        let context_graph_review_witness_repository = workspace_data_repository.clone();
        let context_merge_review_witness_repository = workspace_data_repository.clone();
        let mut state = Self::with_workspace_repositories(
            provider_registry,
            WorkspaceRepositories::new(
                WorkspaceGraphRepositories::new(
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                )
                .with_context_merge_review_witness_repository(
                    context_merge_review_witness_repository,
                ),
                WorkspaceCatalogRepositories::new(
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                    workspace_data_repository.clone(),
                    workspace_data_repository,
                ),
            ),
        )
        .with_context_diff_snapshot_repository_arc(context_diff_snapshot_repository)
        .with_context_lifecycle_repository(lifecycle_repository)
        .with_context_workflow_binding_repository(workflow_binding_repository)
        .with_benchmark_evidence_repository(benchmark_evidence_repository)
        .with_benchmark_definition_binding_writer(benchmark_definition_binding_writer)
        .with_benchmark_definition_binding_repository(benchmark_definition_binding_repository)
        .with_context_commit_history_repository(commit_history_repository)
        .with_context_graph_review_witness_repository(context_graph_review_witness_repository);
        state.postgres_runtime_repository = postgres_runtime_repository;
        if let Some(repository) = state.postgres_runtime_repository.clone() {
            state = state.with_workflow_execution_status_storage_repository(repository.clone());
            state = state.with_benchmark_workspace_projection_repository(repository);
            state = state.with_knowledge_memory_projection_repository(
                knowledge_memory::StorageBackedKnowledgeMemoryProjectionRepository::new(
                    knowledge_memory_repository,
                ),
            );
        }

        Ok(match authorization_audit_sink {
            Some(audit_sink) => state.with_authorization_audit_sink(audit_sink),
            None => state,
        })
    }

    /// Builds application state from an explicit provider registry.
    #[must_use]
    pub fn new(provider_registry: ProviderRegistry) -> Self {
        Self::with_graph_repository(
            provider_registry,
            InMemoryContextGraphRepository::context_engineering_preview(),
        )
    }

    /// Builds application state from explicit infrastructure dependencies.
    #[must_use]
    pub fn with_graph_repository(
        provider_registry: ProviderRegistry,
        graph_repository: impl ContextGraphProjectionRepository + 'static,
    ) -> Self {
        let graph_repository = Arc::new(graph_repository);
        Self {
            provider_registry,
            preview_graph_repository: graph_repository.clone(),
            workspace_graph_repository: graph_repository,
            workspace_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            project_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            experiment_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            context_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            commit_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            commit_history_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            context_graph_review_witness_repository: None,
            context_merge_review_witness_repository: None,
            commit_graph_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            commit_graph_snapshot_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            context_branch_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            context_diff_snapshot_repository: Some(Arc::new(
                InMemoryContextDiffSnapshotV1Repository::new(),
            )),
            component_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            evaluation_run_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            context_authorizer: Arc::new(DenyAllContextAuthorizer),
            authorization_audit_sink: Arc::new(NoopAuthorizationAuditSink),
            guarded_commit_writer: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            context_lifecycle_repository: None,
            context_workflow_binding_repository: None,
            workflow_execution_status_repository: Some(Arc::new(
                workflow_execution::UnavailableWorkflowExecutionStatusAdapter,
            )),
            workflow_execution_status_repository_backend:
                WorkflowExecutionStatusRepositoryBackend::Unavailable,
            benchmark_evidence_repository: None,
            benchmark_decision_discovery_repository: None,
            benchmark_workspace_projection_repository: None,
            benchmark_definition_binding_writer: None,
            benchmark_definition_binding_repository: None,
            benchmark_execution_adapter: None,
            knowledge_memory_projection_repository: Some(Arc::new(
                knowledge_memory::InMemoryKnowledgeMemoryProjectionRepository::default(),
            )),
            plugin_capability_availability_repository: Arc::new(
                EmptyPluginCapabilityAvailabilityRepository,
            ),
            jwt_authenticator: None,
            protected_rate_limiter: None,
            postgres_runtime_repository: None,
        }
    }

    /// Builds application state with an explicit workspace repository and preview fixture.
    #[must_use]
    pub fn with_workspace_graph_repository(
        provider_registry: ProviderRegistry,
        workspace_graph_repository: impl ContextGraphProjectionRepository + 'static,
    ) -> Self {
        Self::with_workspace_repositories(
            provider_registry,
            WorkspaceRepositories::new(
                WorkspaceGraphRepositories::new(
                    workspace_graph_repository,
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                ),
                WorkspaceCatalogRepositories::new(
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                    InMemoryContextGraphRepository::context_engineering_preview(),
                ),
            ),
        )
    }

    /// Builds application state with explicit workspace graph and list repositories.
    #[must_use]
    pub fn with_workspace_repositories(
        provider_registry: ProviderRegistry,
        repositories: WorkspaceRepositories,
    ) -> Self {
        let WorkspaceRepositories {
            graph:
                WorkspaceGraphRepositories {
                    workspace_graph_repository,
                    commit_graph_snapshot_repository,
                    context_branch_repository,
                    context_merge_review_witness_repository,
                },
            catalog:
                WorkspaceCatalogRepositories {
                    workspace_repository,
                    project_repository,
                    experiment_repository,
                    context_repository,
                    commit_repository,
                    commit_graph_repository,
                    component_repository,
                    evaluation_run_repository,
                },
            context_commit_history_repository,
        } = repositories;

        let commit_history_repository: Arc<dyn ContextCommitHistoryRepository> =
            context_commit_history_repository
                .unwrap_or_else(|| Arc::new(UnavailableContextCommitHistoryRepository));
        Self {
            provider_registry,
            preview_graph_repository: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            workspace_graph_repository,
            workspace_repository,
            project_repository,
            experiment_repository,
            context_repository,
            commit_repository,
            commit_history_repository,
            context_graph_review_witness_repository: None,
            context_merge_review_witness_repository,
            commit_graph_repository,
            commit_graph_snapshot_repository,
            context_branch_repository,
            context_diff_snapshot_repository: Some(Arc::new(
                InMemoryContextDiffSnapshotV1Repository::new(),
            )),
            component_repository,
            evaluation_run_repository,
            context_authorizer: Arc::new(DenyAllContextAuthorizer),
            authorization_audit_sink: Arc::new(NoopAuthorizationAuditSink),
            guarded_commit_writer: Arc::new(
                InMemoryContextGraphRepository::context_engineering_preview(),
            ),
            context_lifecycle_repository: None,
            context_workflow_binding_repository: None,
            workflow_execution_status_repository: Some(Arc::new(
                workflow_execution::UnavailableWorkflowExecutionStatusAdapter,
            )),
            workflow_execution_status_repository_backend:
                WorkflowExecutionStatusRepositoryBackend::Unavailable,
            benchmark_evidence_repository: None,
            benchmark_decision_discovery_repository: None,
            benchmark_workspace_projection_repository: None,
            benchmark_definition_binding_writer: None,
            benchmark_definition_binding_repository: None,
            benchmark_execution_adapter: None,
            knowledge_memory_projection_repository: Some(Arc::new(
                knowledge_memory::InMemoryKnowledgeMemoryProjectionRepository::default(),
            )),
            plugin_capability_availability_repository: Arc::new(
                EmptyPluginCapabilityAvailabilityRepository,
            ),
            jwt_authenticator: None,
            protected_rate_limiter: None,
            postgres_runtime_repository: None,
        }
    }

    /// Adds explicit authentication, authorization, and guarded-write dependencies.
    #[must_use]
    pub fn with_protected_write_dependencies(
        self,
        context_authorizer: impl ContextAuthorizer + 'static,
        guarded_commit_writer: impl GuardedContextCommitWriter + 'static,
        jwt_authenticator: impl PrincipalAuthenticator + 'static,
        protected_rate_limiter: impl ProtectedRouteRateLimiter + 'static,
    ) -> Self {
        self.with_protected_write_dependencies_with_authenticator(
            context_authorizer,
            guarded_commit_writer,
            Arc::new(jwt_authenticator),
            Arc::new(protected_rate_limiter),
        )
    }

    /// Adds a reusable private Context lifecycle repository for protected local workflows.
    #[must_use]
    pub fn with_context_lifecycle_repository(
        mut self,
        repository: impl ContextLifecycleRepository + 'static,
    ) -> Self {
        self.context_lifecycle_repository = Some(Arc::new(repository));
        self
    }

    /// Adds the private exact-commit Context diff snapshot repository.
    #[must_use]
    pub fn with_context_diff_snapshot_repository(
        self,
        repository: impl ContextDiffSnapshotV1ReviewRepository + 'static,
    ) -> Self {
        self.with_context_diff_snapshot_repository_arc(Arc::new(repository))
    }

    /// Adds a private complete Context commit-history reader.
    #[must_use]
    pub fn with_context_commit_history_repository(
        mut self,
        repository: impl ContextCommitHistoryRepository + 'static,
    ) -> Self {
        self.commit_history_repository = Arc::new(repository);
        self
    }

    /// Adds the private atomic Context Graph review witness reader.
    #[must_use]
    pub fn with_context_graph_review_witness_repository(
        mut self,
        repository: impl ContextGraphReviewWitnessRepository + 'static,
    ) -> Self {
        self.context_graph_review_witness_repository = Some(Arc::new(repository));
        self
    }

    fn with_context_diff_snapshot_repository_arc(
        mut self,
        repository: Arc<dyn ContextDiffSnapshotV1ReviewRepository>,
    ) -> Self {
        self.context_diff_snapshot_repository = Some(repository);
        self
    }

    /// Adds a private Workflow source-binding repository for protected local reads.
    #[must_use]
    pub fn with_context_workflow_binding_repository(
        mut self,
        repository: impl ContextWorkflowBindingRepository + 'static,
    ) -> Self {
        self.context_workflow_binding_repository = Some(Arc::new(repository));
        self
    }

    /// Adds a private Workflow execution status/replay provenance reader.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn with_workflow_execution_status_repository(
        mut self,
        repository: impl workflow_execution::WorkflowExecutionStatusRepository + 'static,
    ) -> Self {
        self.workflow_execution_status_repository = Some(Arc::new(repository));
        self.workflow_execution_status_repository_backend =
            WorkflowExecutionStatusRepositoryBackend::Custom;
        self
    }

    /// Adds a private workflow execution status repository through the reusable storage contract.
    #[must_use]
    pub fn with_workflow_execution_status_storage_repository(
        mut self,
        repository: impl contextlab_storage::WorkflowExecutionStatusRepository + 'static,
    ) -> Self {
        self.workflow_execution_status_repository = Some(Arc::new(
            workflow_execution::StorageWorkflowExecutionStatusAdapter::new(repository),
        ));
        self.workflow_execution_status_repository_backend =
            WorkflowExecutionStatusRepositoryBackend::StorageBacked;
        self
    }

    /// Adds a private benchmark-evidence repository for protected local inspection.
    #[must_use]
    pub fn with_benchmark_evidence_repository(
        mut self,
        repository: impl BenchmarkEvidenceRepository + BenchmarkDecisionDiscoveryRepository + 'static,
    ) -> Self {
        let repository = Arc::new(repository);
        self.benchmark_evidence_repository = Some(repository.clone());
        self.benchmark_decision_discovery_repository = Some(repository);
        self
    }

    /// Adds a private benchmark-workspace projection reader for protected local inspection.
    #[must_use]
    pub fn with_benchmark_workspace_projection_repository(
        mut self,
        repository: impl BenchmarkWorkspaceProjectionV1Reader + 'static,
    ) -> Self {
        self.benchmark_workspace_projection_repository = Some(Arc::new(repository));
        self
    }

    /// Adds a private provider-free Knowledge/Memory projection reader.
    #[must_use]
    pub fn with_knowledge_memory_projection_repository(
        mut self,
        repository: impl knowledge_memory::KnowledgeMemoryProjectionRepository + 'static,
    ) -> Self {
        self.knowledge_memory_projection_repository = Some(Arc::new(repository));
        self
    }

    /// Adds the private, provider-free Plugin/MCP capability availability reader.
    #[must_use]
    pub fn with_plugin_capability_availability_repository(
        mut self,
        repository: impl PluginCapabilityAvailabilityRepository + 'static,
    ) -> Self {
        self.plugin_capability_availability_repository = Arc::new(repository);
        self
    }

    /// Adds only the private authoring writer for a locally composed route.
    #[must_use]
    pub fn with_benchmark_definition_binding_writer(
        mut self,
        writer: impl BenchmarkDefinitionBindingWriter + 'static,
    ) -> Self {
        self.benchmark_definition_binding_writer = Some(Arc::new(writer));
        self
    }

    /// Adds a private exact-scope benchmark-definition binding reader for local inspection.
    #[must_use]
    pub fn with_benchmark_definition_binding_repository(
        mut self,
        repository: impl BenchmarkDefinitionBindingRepository + 'static,
    ) -> Self {
        self.benchmark_definition_binding_repository = Some(Arc::new(repository));
        self
    }

    /// Adds an injected private benchmark execution adapter. The default runtime is fail-closed.
    #[must_use]
    #[allow(dead_code)]
    pub(crate) fn with_benchmark_execution_adapter(
        mut self,
        adapter: impl benchmark_execution::BenchmarkExecutionAdapter + 'static,
    ) -> Self {
        self.benchmark_execution_adapter = Some(Arc::new(adapter));
        self
    }

    fn with_protected_write_dependencies_with_authenticator(
        mut self,
        context_authorizer: impl ContextAuthorizer + 'static,
        guarded_commit_writer: impl GuardedContextCommitWriter + 'static,
        jwt_authenticator: Arc<dyn PrincipalAuthenticator>,
        protected_rate_limiter: Arc<dyn ProtectedRouteRateLimiter>,
    ) -> Self {
        self.context_authorizer = Arc::new(context_authorizer);
        self.guarded_commit_writer = Arc::new(guarded_commit_writer);
        self.jwt_authenticator = Some(jwt_authenticator);
        self.protected_rate_limiter = Some(protected_rate_limiter);
        self
    }

    /// Replaces the protected-route limiter for explicit composition and tests.
    #[must_use]
    pub fn with_protected_rate_limiter(
        mut self,
        protected_rate_limiter: impl ProtectedRouteRateLimiter + 'static,
    ) -> Self {
        self.protected_rate_limiter = Some(Arc::new(protected_rate_limiter));
        self
    }

    /// Adds an explicit authorization decision audit sink.
    #[must_use]
    pub fn with_authorization_audit_sink(
        mut self,
        audit_sink: impl AuthorizationAuditSink + 'static,
    ) -> Self {
        self.authorization_audit_sink = Arc::new(audit_sink);
        self
    }

    /// Returns the provider registry.
    #[must_use]
    pub const fn provider_registry(&self) -> &ProviderRegistry {
        &self.provider_registry
    }

    /// Returns the Context Graph projection repository.
    #[must_use]
    pub fn graph_repository(&self) -> &dyn ContextGraphProjectionRepository {
        self.preview_graph_repository.as_ref()
    }

    /// Returns the workspace Context Graph projection repository.
    #[must_use]
    pub fn workspace_graph_repository(&self) -> &dyn ContextGraphProjectionRepository {
        self.workspace_graph_repository.as_ref()
    }

    /// Returns the workspace list repository.
    #[must_use]
    pub fn workspace_repository(&self) -> &dyn WorkspaceRepository {
        self.workspace_repository.as_ref()
    }

    /// Returns the project list repository.
    #[must_use]
    pub fn project_repository(&self) -> &dyn ProjectRepository {
        self.project_repository.as_ref()
    }

    /// Returns the experiment list repository.
    #[must_use]
    pub fn experiment_repository(&self) -> &dyn ExperimentRepository {
        self.experiment_repository.as_ref()
    }

    /// Returns the context list repository.
    #[must_use]
    pub fn context_repository(&self) -> &dyn ContextRepository {
        self.context_repository.as_ref()
    }

    /// Returns the commit list repository.
    #[must_use]
    pub fn commit_repository(&self) -> &dyn ContextCommitRepository {
        self.commit_repository.as_ref()
    }

    /// Returns the private complete Context commit-history repository.
    #[must_use]
    pub fn context_commit_history_repository(&self) -> &dyn ContextCommitHistoryRepository {
        self.commit_history_repository.as_ref()
    }

    /// Returns the private backend-owned Context Graph review witness reader.
    #[must_use]
    pub(crate) fn context_graph_review_witness_repository(
        &self,
    ) -> Option<&dyn ContextGraphReviewWitnessRepository> {
        self.context_graph_review_witness_repository.as_deref()
    }

    /// Returns the private atomic server-owned merge review witness repository.
    #[must_use]
    pub(crate) fn context_merge_review_witness_repository(
        &self,
    ) -> Option<&dyn ContextMergeReviewWitnessRepository> {
        self.context_merge_review_witness_repository.as_deref()
    }

    /// Returns the exact Context commit-DAG repository for internal versioning consumers.
    #[must_use]
    pub fn commit_graph_repository(&self) -> &dyn ContextCommitGraphRepository {
        self.commit_graph_repository.as_ref()
    }

    /// Returns the immutable Context commit graph snapshot repository.
    #[must_use]
    pub fn commit_graph_snapshot_repository(&self) -> &dyn CommitGraphSnapshotRepository {
        self.commit_graph_snapshot_repository.as_ref()
    }

    /// Returns the private exact-commit Context diff snapshot repository, when composed.
    #[must_use]
    pub fn context_diff_snapshot_repository(
        &self,
    ) -> Option<&dyn ContextDiffSnapshotV1ReviewRepository> {
        self.context_diff_snapshot_repository.as_deref()
    }

    /// Returns the private durable Context branch-head repository.
    #[must_use]
    pub fn context_branch_repository(&self) -> &dyn ContextBranchRepository {
        self.context_branch_repository.as_ref()
    }

    /// Returns the component list repository.
    #[must_use]
    pub fn component_repository(&self) -> &dyn ContextComponentRepository {
        self.component_repository.as_ref()
    }

    /// Returns the evaluation run list repository.
    #[must_use]
    pub fn evaluation_run_repository(&self) -> &dyn EvaluationRunRepository {
        self.evaluation_run_repository.as_ref()
    }

    pub(crate) fn context_authorizer(&self) -> &dyn ContextAuthorizer {
        self.context_authorizer.as_ref()
    }

    pub(crate) fn guarded_commit_writer(&self) -> &dyn GuardedContextCommitWriter {
        self.guarded_commit_writer.as_ref()
    }

    pub(crate) fn context_lifecycle_repository(&self) -> Option<&dyn ContextLifecycleRepository> {
        self.context_lifecycle_repository.as_deref()
    }

    pub(crate) fn plugin_capability_availability_repository(
        &self,
    ) -> &dyn PluginCapabilityAvailabilityRepository {
        self.plugin_capability_availability_repository.as_ref()
    }

    pub(crate) fn context_workflow_binding_repository(
        &self,
    ) -> Option<&dyn ContextWorkflowBindingRepository> {
        self.context_workflow_binding_repository.as_deref()
    }

    pub(crate) fn workflow_execution_status_repository(
        &self,
    ) -> Option<&dyn workflow_execution::WorkflowExecutionStatusRepository> {
        self.workflow_execution_status_repository.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn workflow_execution_status_repository_backend(
        &self,
    ) -> WorkflowExecutionStatusRepositoryBackend {
        self.workflow_execution_status_repository_backend
    }

    pub(crate) fn benchmark_evidence_repository(&self) -> Option<&dyn BenchmarkEvidenceRepository> {
        self.benchmark_evidence_repository.as_deref()
    }

    pub(crate) fn benchmark_decision_discovery_repository(
        &self,
    ) -> Option<&dyn BenchmarkDecisionDiscoveryRepository> {
        self.benchmark_decision_discovery_repository.as_deref()
    }

    pub(crate) fn benchmark_workspace_projection_repository(
        &self,
    ) -> Option<&dyn BenchmarkWorkspaceProjectionV1Reader> {
        self.benchmark_workspace_projection_repository.as_deref()
    }

    pub(crate) fn knowledge_memory_projection_repository(
        &self,
    ) -> Option<&dyn knowledge_memory::KnowledgeMemoryProjectionRepository> {
        self.knowledge_memory_projection_repository.as_deref()
    }

    pub(crate) fn benchmark_definition_binding_writer(
        &self,
    ) -> Option<&dyn BenchmarkDefinitionBindingWriter> {
        self.benchmark_definition_binding_writer.as_deref()
    }

    pub(crate) fn benchmark_definition_binding_repository(
        &self,
    ) -> Option<&dyn BenchmarkDefinitionBindingRepository> {
        self.benchmark_definition_binding_repository.as_deref()
    }

    pub(crate) fn benchmark_execution_adapter(
        &self,
    ) -> Option<&dyn benchmark_execution::BenchmarkExecutionAdapter> {
        self.benchmark_execution_adapter.as_deref()
    }

    pub(crate) async fn record_authorization_decision(
        &self,
        event: AuthorizationAuditEvent,
    ) -> Result<(), AuthorizationAuditError> {
        self.authorization_audit_sink.record(event).await
    }

    pub(crate) async fn authenticate_principal(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<AuthenticatedPrincipal, AuthenticationError> {
        let authenticator = self
            .jwt_authenticator
            .as_ref()
            .ok_or(AuthenticationError::InvalidConfiguration)?;
        authenticator
            .authenticate_authorization_header(authorization_header)
            .await
    }

    pub(crate) async fn check_protected_rate_limit(
        &self,
        key: ProtectedRouteRateLimitKey,
    ) -> Result<RateLimitDecision, RateLimitError> {
        let limiter = self
            .protected_rate_limiter
            .as_ref()
            .ok_or(RateLimitError::Unavailable)?;
        limiter.check(key).await
    }
}

/// API state configuration errors.
#[derive(Debug, Error)]
pub enum AppStateConfigError {
    /// Graph repository mode is not supported.
    #[error("unsupported graph repository mode: {mode}")]
    UnsupportedGraphRepositoryMode {
        /// Raw mode value.
        mode: String,
    },
    /// PostgreSQL mode was requested without a database URL.
    #[error(
        "CONTEXTLAB_DATABASE_URL or DATABASE_URL is required when CONTEXTLAB_GRAPH_REPOSITORY=postgres"
    )]
    MissingDatabaseUrl,
    /// The requested API route mode is unsupported.
    #[error("unsupported API route mode: {mode}")]
    UnsupportedApiRouteMode {
        /// Raw mode value.
        mode: String,
    },
    /// Protected routes need PostgreSQL-backed authorization, audit, and write state.
    #[error("protected routes require CONTEXTLAB_GRAPH_REPOSITORY=postgres")]
    ProtectedRoutesRequirePostgres,
    /// Protected HMAC authentication needs a signing secret.
    #[error("CONTEXTLAB_AUTH_HS256_SECRET is required for protected routes")]
    MissingProtectedAuthSecret,
    /// Protected HMAC authentication needs an issuer restriction.
    #[error("CONTEXTLAB_AUTH_ISSUER is required for protected routes")]
    MissingProtectedAuthIssuer,
    /// Protected HMAC authentication needs an audience restriction.
    #[error("CONTEXTLAB_AUTH_AUDIENCE is required for protected routes")]
    MissingProtectedAuthAudience,
    /// Protected HMAC authentication settings were rejected by the authenticator.
    #[error("protected HMAC authentication configuration is invalid")]
    InvalidProtectedAuthConfiguration,
    /// Protected authentication mode is not supported.
    #[error("unsupported protected authentication mode: {mode}")]
    UnsupportedProtectedAuthMode {
        /// Raw mode value.
        mode: String,
    },
    /// Protected OIDC authentication needs an issuer restriction.
    #[error("CONTEXTLAB_OIDC_ISSUER is required for protected OIDC routes")]
    MissingOidcIssuer,
    /// Protected OIDC authentication needs an audience restriction.
    #[error("CONTEXTLAB_OIDC_AUDIENCE is required for protected OIDC routes")]
    MissingOidcAudience,
    /// Protected OIDC authentication needs an HTTPS JWKS endpoint.
    #[error("CONTEXTLAB_OIDC_JWKS_URL is required for protected OIDC routes")]
    MissingOidcJwksUrl,
    /// Protected OIDC authentication needs an explicit bounded cache TTL.
    #[error("CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS is required for protected OIDC routes")]
    MissingOidcCacheTtl,
    /// Protected OIDC authentication cache TTL is invalid.
    #[error("CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS must be an integer from 1 to 3600")]
    InvalidOidcCacheTtl,
    /// Protected OIDC group extraction needs an explicit top-level claim name.
    #[error("CONTEXTLAB_OIDC_GROUPS_CLAIM is required when OIDC group lifetime is configured")]
    MissingOidcGroupsClaim,
    /// Protected OIDC group extraction needs an explicit maximum token lifetime.
    #[error(
        "CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS is required when OIDC groups are configured"
    )]
    MissingOidcMaxTokenLifetime,
    /// Protected OIDC group token lifetime is invalid.
    #[error("CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS must be an integer from 1 to 3600")]
    InvalidOidcMaxTokenLifetime,
    /// Protected OIDC authentication settings were rejected by the authenticator.
    #[error("protected OIDC authentication configuration is invalid")]
    InvalidOidcConfiguration,
    /// Protected routes need an explicit request limit.
    #[error("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS is required for protected routes")]
    MissingProtectedRateLimitMaxRequests,
    /// Protected routes need an explicit sliding-window duration.
    #[error("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS is required for protected routes")]
    MissingProtectedRateLimitWindowSeconds,
    /// Protected routes need an explicit active-principal bound.
    #[error(
        "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS is required for protected routes"
    )]
    MissingProtectedRateLimitMaxTrackedPrincipals,
    /// Protected rate-limit settings are malformed or outside supported bounds.
    #[error("protected rate-limit configuration is invalid")]
    InvalidProtectedRateLimitConfiguration,
    /// Repository construction failed.
    #[error(transparent)]
    Storage(#[from] StorageRepositoryError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApiRouteMode {
    Public,
    Protected,
}

impl ApiRouteMode {
    fn from_env(env: &[(String, String)]) -> Result<Self, AppStateConfigError> {
        match env_value(env, "CONTEXTLAB_API_ROUTE_MODE")
            .unwrap_or("public")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "public" => Ok(Self::Public),
            "protected" => Ok(Self::Protected),
            mode => Err(AppStateConfigError::UnsupportedApiRouteMode {
                mode: mode.to_owned(),
            }),
        }
    }
}

enum ProtectedRuntimeAuthConfig {
    Hmac(ProtectedHmacAuthConfig),
    Oidc(OidcJwksConfig),
}

struct ProtectedHmacAuthConfig {
    secret: String,
    issuer: String,
    audience: String,
}

struct ProtectedRateLimitConfig {
    policy: ProtectedRouteRateLimitPolicy,
}

impl ProtectedRateLimitConfig {
    fn from_env(env: &[(String, String)]) -> Result<Self, AppStateConfigError> {
        let max_requests = required_protected_auth_value(
            env,
            "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS",
            AppStateConfigError::MissingProtectedRateLimitMaxRequests,
        )?
        .parse::<u32>()
        .map_err(|_| AppStateConfigError::InvalidProtectedRateLimitConfiguration)?;
        let window_seconds = required_protected_auth_value(
            env,
            "CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS",
            AppStateConfigError::MissingProtectedRateLimitWindowSeconds,
        )?
        .parse::<u64>()
        .map_err(|_| AppStateConfigError::InvalidProtectedRateLimitConfiguration)?;
        let max_tracked_principals = required_protected_auth_value(
            env,
            "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS",
            AppStateConfigError::MissingProtectedRateLimitMaxTrackedPrincipals,
        )?
        .parse::<usize>()
        .map_err(|_| AppStateConfigError::InvalidProtectedRateLimitConfiguration)?;
        let policy = ProtectedRouteRateLimitPolicy::new(
            max_requests,
            window_seconds,
            max_tracked_principals,
        )
        .map_err(|_| AppStateConfigError::InvalidProtectedRateLimitConfiguration)?;

        Ok(Self { policy })
    }
}

impl ProtectedRuntimeAuthConfig {
    fn from_env(env: &[(String, String)]) -> Result<Self, AppStateConfigError> {
        match env_value(env, "CONTEXTLAB_AUTH_MODE")
            .unwrap_or("hmac")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "hmac" => ProtectedHmacAuthConfig::from_env(env).map(Self::Hmac),
            "oidc" => oidc_jwks_config_from_env(env).map(Self::Oidc),
            mode => Err(AppStateConfigError::UnsupportedProtectedAuthMode {
                mode: mode.to_owned(),
            }),
        }
    }
}

impl ProtectedHmacAuthConfig {
    fn from_env(env: &[(String, String)]) -> Result<Self, AppStateConfigError> {
        Ok(Self {
            secret: required_protected_auth_value(
                env,
                "CONTEXTLAB_AUTH_HS256_SECRET",
                AppStateConfigError::MissingProtectedAuthSecret,
            )?,
            issuer: required_protected_auth_value(
                env,
                "CONTEXTLAB_AUTH_ISSUER",
                AppStateConfigError::MissingProtectedAuthIssuer,
            )?,
            audience: required_protected_auth_value(
                env,
                "CONTEXTLAB_AUTH_AUDIENCE",
                AppStateConfigError::MissingProtectedAuthAudience,
            )?,
        })
    }
}

fn oidc_jwks_config_from_env(
    env: &[(String, String)],
) -> Result<OidcJwksConfig, AppStateConfigError> {
    let issuer = required_protected_auth_value(
        env,
        "CONTEXTLAB_OIDC_ISSUER",
        AppStateConfigError::MissingOidcIssuer,
    )?;
    let audience = required_protected_auth_value(
        env,
        "CONTEXTLAB_OIDC_AUDIENCE",
        AppStateConfigError::MissingOidcAudience,
    )?;
    let jwks_url = required_protected_auth_value(
        env,
        "CONTEXTLAB_OIDC_JWKS_URL",
        AppStateConfigError::MissingOidcJwksUrl,
    )?;
    let cache_ttl_seconds = required_protected_auth_value(
        env,
        "CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS",
        AppStateConfigError::MissingOidcCacheTtl,
    )?
    .parse::<u64>()
    .map_err(|_| AppStateConfigError::InvalidOidcCacheTtl)?;
    if !(1..=3600).contains(&cache_ttl_seconds) {
        return Err(AppStateConfigError::InvalidOidcCacheTtl);
    }

    let configuration = OidcJwksConfig::new(&jwks_url, &issuer, &audience, cache_ttl_seconds)
        .map_err(|_| AppStateConfigError::InvalidOidcConfiguration)?;
    match (
        env_value(env, "CONTEXTLAB_OIDC_GROUPS_CLAIM"),
        env_value(env, "CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS"),
    ) {
        (None, None) => Ok(configuration),
        (Some(_), None) => Err(AppStateConfigError::MissingOidcMaxTokenLifetime),
        (None, Some(_)) => Err(AppStateConfigError::MissingOidcGroupsClaim),
        (Some(group_claim), Some(max_token_lifetime_seconds)) => {
            let max_token_lifetime_seconds = max_token_lifetime_seconds
                .parse::<u64>()
                .map_err(|_| AppStateConfigError::InvalidOidcMaxTokenLifetime)?;
            if !(1..=3600).contains(&max_token_lifetime_seconds) {
                return Err(AppStateConfigError::InvalidOidcMaxTokenLifetime);
            }
            configuration
                .with_group_claim(group_claim, max_token_lifetime_seconds)
                .map_err(|_| AppStateConfigError::InvalidOidcConfiguration)
        }
    }
}

/// Builds the API router.
pub fn build_router() -> Router {
    try_build_router_from_current_env().expect("valid ContextLab API runtime configuration")
}

/// Tries to build the API router from the current process environment.
pub fn try_build_router_from_current_env() -> Result<Router, AppStateConfigError> {
    try_build_router_from_env(std::env::vars())
}

/// Tries to build the API router from explicit environment pairs.
pub fn try_build_router_from_env<I, K, V>(env: I) -> Result<Router, AppStateConfigError>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let env = env
        .into_iter()
        .map(|(key, value)| (key.into(), value.into()))
        .collect::<Vec<_>>();
    let route_mode = ApiRouteMode::from_env(&env)?;
    let state = AppState::try_from_env(env.iter().cloned())?;

    match route_mode {
        ApiRouteMode::Public => Ok(build_router_with_state(state)),
        ApiRouteMode::Protected => {
            let repository = state
                .postgres_runtime_repository
                .clone()
                .ok_or(AppStateConfigError::ProtectedRoutesRequirePostgres)?;
            let authentication = ProtectedRuntimeAuthConfig::from_env(&env)?;
            let rate_limit = ProtectedRateLimitConfig::from_env(&env)?;
            let authenticator: Arc<dyn PrincipalAuthenticator> = match authentication {
                ProtectedRuntimeAuthConfig::Hmac(authentication) => Arc::new(
                    HmacJwtAuthenticator::new(
                        &authentication.secret,
                        &authentication.issuer,
                        &authentication.audience,
                    )
                    .map_err(|_| AppStateConfigError::InvalidProtectedAuthConfiguration)?,
                ),
                ProtectedRuntimeAuthConfig::Oidc(configuration) => {
                    Arc::new(OidcJwksAuthenticator::new(
                        HttpsJwksSource::new()
                            .map_err(|_| AppStateConfigError::InvalidOidcConfiguration)?,
                        configuration,
                    ))
                }
            };
            let protected_rate_limiter =
                Arc::new(InMemoryProtectedRouteRateLimiter::new(rate_limit.policy));

            Ok(build_protected_router_with_state(
                state.with_protected_write_dependencies_with_authenticator(
                    RoleBasedContextAuthorizer::new(repository.clone()),
                    repository,
                    authenticator,
                    protected_rate_limiter,
                ),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicGetRoute {
    Healthz,
    Meta,
    OpenApi,
    Providers,
    Workspaces,
    WorkspaceProjects,
    ProjectExperiments,
    ProjectContexts,
    ContextCommits,
    ContextCommit,
    ContextComponents,
    ContextComponent,
    ContextEvaluationRuns,
    ContextEvaluationScorecard,
    ContextEvaluationRun,
    ContextGraphPreview,
    WorkspaceContextGraph,
}

const PUBLIC_GET_ROUTES: &[PublicGetRoute] = &[
    PublicGetRoute::Healthz,
    PublicGetRoute::Meta,
    PublicGetRoute::OpenApi,
    PublicGetRoute::Providers,
    PublicGetRoute::Workspaces,
    PublicGetRoute::WorkspaceProjects,
    PublicGetRoute::ProjectExperiments,
    PublicGetRoute::ProjectContexts,
    PublicGetRoute::ContextCommits,
    PublicGetRoute::ContextCommit,
    PublicGetRoute::ContextComponents,
    PublicGetRoute::ContextComponent,
    PublicGetRoute::ContextEvaluationRuns,
    PublicGetRoute::ContextEvaluationScorecard,
    PublicGetRoute::ContextEvaluationRun,
    PublicGetRoute::ContextGraphPreview,
    PublicGetRoute::WorkspaceContextGraph,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublicPostRoute {
    GraphDiffs,
}

const PUBLIC_POST_ROUTES: &[PublicPostRoute] = &[PublicPostRoute::GraphDiffs];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectedPostRoute {
    ContextCommit,
    LocalComponentLifecycleCommit,
    LocalBenchmarkDefinitionAuthoring,
    LocalBenchmarkExecution,
}

const PROTECTED_POST_ROUTES: &[ProtectedPostRoute] = &[
    ProtectedPostRoute::ContextCommit,
    ProtectedPostRoute::LocalComponentLifecycleCommit,
];

const PROTECTED_BENCHMARK_DEFINITION_POST_ROUTES: &[ProtectedPostRoute] =
    &[ProtectedPostRoute::LocalBenchmarkDefinitionAuthoring];

const PROTECTED_BENCHMARK_EXECUTION_POST_ROUTES: &[ProtectedPostRoute] =
    &[ProtectedPostRoute::LocalBenchmarkExecution];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectedGetRoute {
    ContextBranchHeads,
    ContextCommitGraphDiff,
    LocalContextDiffReview,
    LocalContextMergeReview,
    ContextLifecycleState,
    KnowledgeMemoryProjection,
    WorkflowCapabilityStatus,
    WorkflowContextBindings,
    WorkflowExecutionStatus,
    PluginCapabilityAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectedBenchmarkGetRoute {
    List,
    Detail,
    RunDetails,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectedBenchmarkDefinitionGetRoute {
    List,
}

const PROTECTED_BENCHMARK_GET_ROUTES: &[ProtectedBenchmarkGetRoute] = &[
    ProtectedBenchmarkGetRoute::List,
    ProtectedBenchmarkGetRoute::Detail,
    ProtectedBenchmarkGetRoute::RunDetails,
];

const PROTECTED_BENCHMARK_DEFINITION_GET_ROUTES: &[ProtectedBenchmarkDefinitionGetRoute] =
    &[ProtectedBenchmarkDefinitionGetRoute::List];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectedBenchmarkWorkspaceGetRoute {
    Workspace,
    DecisionWorkspace,
}

const PROTECTED_BENCHMARK_WORKSPACE_GET_ROUTES: &[ProtectedBenchmarkWorkspaceGetRoute] = &[
    ProtectedBenchmarkWorkspaceGetRoute::Workspace,
    ProtectedBenchmarkWorkspaceGetRoute::DecisionWorkspace,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectedBenchmarkDiffGetRoute {
    LocalBenchmarkDecisionDiff,
}

const PROTECTED_BENCHMARK_DIFF_GET_ROUTES: &[ProtectedBenchmarkDiffGetRoute] =
    &[ProtectedBenchmarkDiffGetRoute::LocalBenchmarkDecisionDiff];

const PROTECTED_GET_ROUTES: &[ProtectedGetRoute] = &[
    ProtectedGetRoute::ContextLifecycleState,
    ProtectedGetRoute::WorkflowCapabilityStatus,
    ProtectedGetRoute::WorkflowContextBindings,
    ProtectedGetRoute::PluginCapabilityAvailability,
];

impl ProtectedPostRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::ContextCommit => "/api/v1/contexts/{context_id}/commits",
            Self::LocalComponentLifecycleCommit => {
                "/api/v1/local/contexts/{context_id}/component-lifecycle-commits"
            }
            Self::LocalBenchmarkDefinitionAuthoring => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings"
            }
            Self::LocalBenchmarkExecution => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-executions"
            }
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::ContextCommit => router.route(
                self.path(),
                axum::routing::post(routes::create_context_commit),
            ),
            Self::LocalComponentLifecycleCommit => router.route(
                self.path(),
                axum::routing::post(routes::create_local_component_lifecycle_commit),
            ),
            Self::LocalBenchmarkDefinitionAuthoring => router.route(
                self.path(),
                axum::routing::post(benchmark_definition_authoring::create),
            ),
            Self::LocalBenchmarkExecution => router.route(
                self.path(),
                axum::routing::post(benchmark_execution::create),
            ),
        }
    }
}

impl ProtectedGetRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::ContextBranchHeads => "/api/v1/local/contexts/{context_id}/branches",
            Self::ContextCommitGraphDiff => "/api/v1/local/contexts/{context_id}/graph-diff",
            Self::LocalContextDiffReview => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/diff-review"
            }
            Self::LocalContextMergeReview => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/merge-review"
            }
            Self::ContextLifecycleState => {
                "/api/v1/local/contexts/{context_id}/commits/{commit_id}/lifecycle-state"
            }
            Self::KnowledgeMemoryProjection => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/knowledge-memory-projection"
            }
            Self::WorkflowCapabilityStatus => {
                "/api/v1/local/contexts/{context_id}/workflow/capability-status"
            }
            Self::WorkflowContextBindings => {
                "/api/v1/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings"
            }
            Self::WorkflowExecutionStatus => {
                "/api/v1/local/contexts/{context_id}/workflow/runs/{run_id}/status"
            }
            Self::PluginCapabilityAvailability => {
                "/api/v1/local/contexts/{context_id}/plugins/capabilities"
            }
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::ContextBranchHeads => {
                router.route(self.path(), axum::routing::get(local_branch_heads::list))
            }
            Self::ContextCommitGraphDiff => router.route(
                self.path(),
                axum::routing::get(routes::context_commit_graph_diff),
            ),
            Self::LocalContextDiffReview => router.route(
                self.path(),
                axum::routing::get(routes::local_context_diff_review),
            ),
            Self::LocalContextMergeReview => router.route(
                self.path(),
                axum::routing::get(routes::local_context_merge_review),
            ),
            Self::ContextLifecycleState => router.route(
                self.path(),
                axum::routing::get(routes::local_context_lifecycle_state),
            ),
            Self::KnowledgeMemoryProjection => router.route(
                self.path(),
                axum::routing::get(routes::local_knowledge_memory_projection),
            ),
            Self::WorkflowCapabilityStatus => router.route(
                self.path(),
                axum::routing::get(routes::local_workflow_capability_status),
            ),
            Self::WorkflowContextBindings => router.route(
                self.path(),
                axum::routing::get(routes::local_workflow_context_bindings),
            ),
            Self::WorkflowExecutionStatus => router.route(
                self.path(),
                axum::routing::get(routes::local_workflow_execution_status),
            ),
            Self::PluginCapabilityAvailability => router.route(
                self.path(),
                axum::routing::get(routes::local_plugin_capability_availability),
            ),
        }
    }
}

impl ProtectedBenchmarkGetRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::List => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions"
            }
            Self::Detail => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}"
            }
            Self::RunDetails => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}/run-details"
            }
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::List => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_decision_list),
            ),
            Self::Detail => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_decision),
            ),
            Self::RunDetails => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_decision_run_details),
            ),
        }
    }
}

impl ProtectedBenchmarkDefinitionGetRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::List => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings"
            }
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::List => router.route(
                self.path(),
                axum::routing::get(benchmark_definition_authoring::list),
            ),
        }
    }
}

impl ProtectedBenchmarkWorkspaceGetRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::Workspace => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}"
            }
            Self::DecisionWorkspace => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}/workspace"
            }
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::Workspace => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_workspace),
            ),
            Self::DecisionWorkspace => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_workspace_by_decision),
            ),
        }
    }
}

impl ProtectedBenchmarkDiffGetRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::LocalBenchmarkDecisionDiff => {
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/benchmark-decision-diffs"
            }
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::LocalBenchmarkDecisionDiff => router.route(
                self.path(),
                axum::routing::get(routes::local_benchmark_decision_diff),
            ),
        }
    }
}

impl PublicPostRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::GraphDiffs => "/api/v1/graph-diffs",
        }
    }

    #[cfg(test)]
    const fn operation_id(self) -> &'static str {
        match self {
            Self::GraphDiffs => "compareGraphs",
        }
    }

    #[cfg(test)]
    const fn path_parameters(self) -> &'static [&'static str] {
        match self {
            Self::GraphDiffs => NO_PARAMETERS,
        }
    }

    #[cfg(test)]
    const fn query_parameters(self) -> &'static [&'static str] {
        match self {
            Self::GraphDiffs => NO_PARAMETERS,
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::GraphDiffs => router.route(self.path(), axum::routing::post(routes::graph_diffs)),
        }
    }
}

#[cfg(test)]
const NO_PARAMETERS: &[&str] = &[];
#[cfg(test)]
const CONTEXT_PATH_PARAMETERS: &[&str] = &["context_id"];
#[cfg(test)]
const CONTEXT_COMMIT_PATH_PARAMETERS: &[&str] = &["context_id", "commit_id"];
#[cfg(test)]
const CONTEXT_COMPONENT_PATH_PARAMETERS: &[&str] = &["context_id", "component_id"];
#[cfg(test)]
const CONTEXT_EVALUATION_RUN_PATH_PARAMETERS: &[&str] = &["context_id", "run_id"];
#[cfg(test)]
const PROJECT_PATH_PARAMETERS: &[&str] = &["project_id"];
#[cfg(test)]
const WORKSPACE_PATH_PARAMETERS: &[&str] = &["workspace_id"];

impl PublicGetRoute {
    const fn path(self) -> &'static str {
        match self {
            Self::Healthz => "/healthz",
            Self::Meta => "/api/v1/meta",
            Self::OpenApi => "/api/v1/openapi.json",
            Self::Providers => "/api/v1/providers",
            Self::Workspaces => "/api/v1/workspaces",
            Self::WorkspaceProjects => "/api/v1/workspaces/{workspace_id}/projects",
            Self::ProjectExperiments => "/api/v1/projects/{project_id}/experiments",
            Self::ProjectContexts => "/api/v1/projects/{project_id}/contexts",
            Self::ContextCommits => "/api/v1/contexts/{context_id}/commits",
            Self::ContextCommit => "/api/v1/contexts/{context_id}/commits/{commit_id}",
            Self::ContextComponents => "/api/v1/contexts/{context_id}/components",
            Self::ContextComponent => "/api/v1/contexts/{context_id}/components/{component_id}",
            Self::ContextEvaluationRuns => "/api/v1/contexts/{context_id}/evaluation-runs",
            Self::ContextEvaluationScorecard => {
                "/api/v1/contexts/{context_id}/evaluation-scorecard"
            }
            Self::ContextEvaluationRun => "/api/v1/contexts/{context_id}/evaluation-runs/{run_id}",
            Self::ContextGraphPreview => "/api/v1/context-graph/preview",
            Self::WorkspaceContextGraph => "/api/v1/workspaces/{workspace_id}/context-graph",
        }
    }

    #[cfg(test)]
    const fn operation_id(self) -> &'static str {
        match self {
            Self::Healthz => "healthz",
            Self::Meta => "getMeta",
            Self::OpenApi => "getOpenApiDocument",
            Self::Providers => "listProviders",
            Self::Workspaces => "listWorkspaces",
            Self::WorkspaceProjects => "listProjects",
            Self::ProjectExperiments => "listExperiments",
            Self::ProjectContexts => "listContexts",
            Self::ContextCommits => "listCommits",
            Self::ContextCommit => "getCommit",
            Self::ContextComponents => "listComponents",
            Self::ContextComponent => "getComponent",
            Self::ContextEvaluationRuns => "listEvaluationRuns",
            Self::ContextEvaluationScorecard => "getEvaluationScorecard",
            Self::ContextEvaluationRun => "getEvaluationRun",
            Self::ContextGraphPreview => "getContextGraphPreview",
            Self::WorkspaceContextGraph => "getWorkspaceContextGraph",
        }
    }

    #[cfg(test)]
    const fn path_parameters(self) -> &'static [&'static str] {
        match self {
            Self::WorkspaceProjects | Self::WorkspaceContextGraph => WORKSPACE_PATH_PARAMETERS,
            Self::ProjectExperiments | Self::ProjectContexts => PROJECT_PATH_PARAMETERS,
            Self::ContextCommit => CONTEXT_COMMIT_PATH_PARAMETERS,
            Self::ContextComponent => CONTEXT_COMPONENT_PATH_PARAMETERS,
            Self::ContextEvaluationRun => CONTEXT_EVALUATION_RUN_PATH_PARAMETERS,
            Self::ContextCommits
            | Self::ContextComponents
            | Self::ContextEvaluationRuns
            | Self::ContextEvaluationScorecard => CONTEXT_PATH_PARAMETERS,
            Self::Healthz
            | Self::Meta
            | Self::OpenApi
            | Self::Providers
            | Self::Workspaces
            | Self::ContextGraphPreview => NO_PARAMETERS,
        }
    }

    #[cfg(test)]
    const fn query_parameters(self) -> &'static [&'static str] {
        match self {
            Self::Workspaces | Self::WorkspaceProjects | Self::ProjectExperiments => {
                routes::DISCOVERY_LIST_QUERY_PARAMETERS
            }
            Self::ProjectContexts => routes::CONTEXT_LIST_QUERY_PARAMETERS,
            Self::ContextCommits => routes::COMMIT_LIST_QUERY_PARAMETERS,
            Self::ContextComponents => routes::COMPONENT_LIST_QUERY_PARAMETERS,
            Self::ContextEvaluationRuns => routes::EVALUATION_RUN_LIST_QUERY_PARAMETERS,
            Self::ContextEvaluationScorecard => routes::EVALUATION_SCORECARD_QUERY_PARAMETERS,
            Self::ContextCommit
            | Self::ContextComponent
            | Self::ContextEvaluationRun
            | Self::Healthz
            | Self::Meta
            | Self::OpenApi
            | Self::Providers
            | Self::ContextGraphPreview
            | Self::WorkspaceContextGraph => NO_PARAMETERS,
        }
    }

    fn install(self, router: Router<AppState>) -> Router<AppState> {
        match self {
            Self::Healthz => router.route(self.path(), axum::routing::get(routes::healthz)),
            Self::Meta => router.route(self.path(), axum::routing::get(routes::meta)),
            Self::OpenApi => router.route(self.path(), axum::routing::get(routes::openapi)),
            Self::Providers => router.route(self.path(), axum::routing::get(routes::providers)),
            Self::Workspaces => router.route(self.path(), axum::routing::get(routes::workspaces)),
            Self::WorkspaceProjects => {
                router.route(self.path(), axum::routing::get(routes::workspace_projects))
            }
            Self::ProjectExperiments => {
                router.route(self.path(), axum::routing::get(routes::project_experiments))
            }
            Self::ProjectContexts => {
                router.route(self.path(), axum::routing::get(routes::project_contexts))
            }
            Self::ContextCommits => {
                router.route(self.path(), axum::routing::get(routes::context_commits))
            }
            Self::ContextCommit => {
                router.route(self.path(), axum::routing::get(routes::context_commit))
            }
            Self::ContextComponents => {
                router.route(self.path(), axum::routing::get(routes::context_components))
            }
            Self::ContextComponent => {
                router.route(self.path(), axum::routing::get(routes::context_component))
            }
            Self::ContextEvaluationRuns => router.route(
                self.path(),
                axum::routing::get(routes::context_evaluation_runs),
            ),
            Self::ContextEvaluationScorecard => router.route(
                self.path(),
                axum::routing::get(routes::context_evaluation_scorecard),
            ),
            Self::ContextEvaluationRun => router.route(
                self.path(),
                axum::routing::get(routes::context_evaluation_run),
            ),
            Self::ContextGraphPreview => router.route(
                self.path(),
                axum::routing::get(routes::context_graph_preview),
            ),
            Self::WorkspaceContextGraph => router.route(
                self.path(),
                axum::routing::get(routes::workspace_context_graph),
            ),
        }
    }
}

/// Builds the API router with explicit application state.
pub fn build_router_with_state(state: AppState) -> Router {
    public_router().with_state(state)
}

/// Builds the public router plus an explicitly supplied protected commit route.
pub fn build_protected_router_with_state(state: AppState) -> Router {
    let mut protected_write_router = Router::<AppState>::new();
    for route in PROTECTED_POST_ROUTES {
        protected_write_router = route.install(protected_write_router);
    }
    let protected_write_router = protected_write_router.layer(middleware::from_fn_with_state(
        state.clone(),
        routes::authenticate_context_commit_write_request,
    ));
    let mut protected_benchmark_definition_write_router = Router::<AppState>::new();
    for route in PROTECTED_BENCHMARK_DEFINITION_POST_ROUTES {
        protected_benchmark_definition_write_router =
            route.install(protected_benchmark_definition_write_router);
    }
    let protected_benchmark_definition_write_router = protected_benchmark_definition_write_router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_benchmark_definition_authoring_write_request,
        ));
    let mut protected_benchmark_execution_write_router = Router::<AppState>::new();
    for route in PROTECTED_BENCHMARK_EXECUTION_POST_ROUTES {
        protected_benchmark_execution_write_router =
            route.install(protected_benchmark_execution_write_router);
    }
    let protected_benchmark_execution_write_router = protected_benchmark_execution_write_router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_benchmark_execution_write_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let mut protected_read_router = Router::<AppState>::new();
    for route in PROTECTED_GET_ROUTES {
        protected_read_router = route.install(protected_read_router);
    }
    let protected_read_router = protected_read_router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_context_lifecycle_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let protected_commit_graph_diff_read_router = ProtectedGetRoute::ContextCommitGraphDiff
        .install(Router::<AppState>::new())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_context_commit_graph_diff_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let protected_context_diff_review_read_router = ProtectedGetRoute::LocalContextDiffReview
        .install(Router::<AppState>::new())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_context_commit_graph_diff_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let protected_context_merge_review_read_router = ProtectedGetRoute::LocalContextMergeReview
        .install(Router::<AppState>::new())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_context_commit_graph_diff_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let protected_context_branch_heads_read_router = ProtectedGetRoute::ContextBranchHeads
        .install(Router::<AppState>::new())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_context_branch_head_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let protected_knowledge_memory_read_router = ProtectedGetRoute::KnowledgeMemoryProjection
        .install(Router::<AppState>::new())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_knowledge_memory_projection_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let protected_workflow_execution_status_read_router =
        ProtectedGetRoute::WorkflowExecutionStatus
            .install(Router::<AppState>::new())
            .layer(middleware::from_fn_with_state(
                state.clone(),
                routes::authenticate_workflow_execution_status_read_request,
            ))
            .layer(middleware::from_fn(routes::private_no_store_response));
    let mut protected_benchmark_read_router = Router::<AppState>::new();
    for route in PROTECTED_BENCHMARK_GET_ROUTES {
        protected_benchmark_read_router = route.install(protected_benchmark_read_router);
    }
    let protected_benchmark_read_router =
        protected_benchmark_read_router.layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_benchmark_evidence_read_request,
        ));
    let mut protected_benchmark_definition_read_router = Router::<AppState>::new();
    for route in PROTECTED_BENCHMARK_DEFINITION_GET_ROUTES {
        protected_benchmark_definition_read_router =
            route.install(protected_benchmark_definition_read_router);
    }
    let protected_benchmark_definition_read_router = protected_benchmark_definition_read_router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_benchmark_definition_binding_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));
    let mut protected_benchmark_diff_read_router = Router::<AppState>::new();
    for route in PROTECTED_BENCHMARK_DIFF_GET_ROUTES {
        protected_benchmark_diff_read_router = route.install(protected_benchmark_diff_read_router);
    }
    let protected_benchmark_diff_read_router =
        protected_benchmark_diff_read_router.layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_benchmark_decision_diff_read_request,
        ));
    let mut protected_benchmark_workspace_read_router = Router::<AppState>::new();
    for route in PROTECTED_BENCHMARK_WORKSPACE_GET_ROUTES {
        protected_benchmark_workspace_read_router =
            route.install(protected_benchmark_workspace_read_router);
    }
    let protected_benchmark_workspace_read_router = protected_benchmark_workspace_read_router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            routes::authenticate_benchmark_workspace_read_request,
        ))
        .layer(middleware::from_fn(routes::private_no_store_response));

    public_router()
        .merge(protected_write_router)
        .merge(protected_benchmark_definition_write_router)
        .merge(protected_benchmark_execution_write_router)
        .merge(protected_read_router)
        .merge(protected_context_branch_heads_read_router)
        .merge(protected_commit_graph_diff_read_router)
        .merge(protected_context_diff_review_read_router)
        .merge(protected_context_merge_review_read_router)
        .merge(protected_knowledge_memory_read_router)
        .merge(protected_workflow_execution_status_read_router)
        .merge(protected_benchmark_read_router)
        .merge(protected_benchmark_definition_read_router)
        .merge(protected_benchmark_diff_read_router)
        .merge(protected_benchmark_workspace_read_router)
        .with_state(state)
}

fn public_router() -> Router<AppState> {
    let mut router = Router::<AppState>::new();

    for route in PUBLIC_GET_ROUTES {
        router = route.install(router);
    }

    for route in PUBLIC_POST_ROUTES {
        router = route.install(router);
    }

    router
}

fn workspace_data_repository_from_env(
    env: &[(String, String)],
) -> Result<WorkspaceDataRepository, AppStateConfigError> {
    match env_value(env, "CONTEXTLAB_GRAPH_REPOSITORY")
        .unwrap_or("memory")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "memory" | "in_memory" | "preview" => Ok(WorkspaceDataRepository::Memory(
            InMemoryContextGraphRepository::context_engineering_preview(),
        )),
        "postgres" | "postgresql" => {
            let database_url = database_url_from_env(env)?;

            Ok(WorkspaceDataRepository::Postgres(
                PostgresContextGraphRepository::connect_lazy(database_url)?,
            ))
        }
        mode => Err(AppStateConfigError::UnsupportedGraphRepositoryMode {
            mode: mode.to_owned(),
        }),
    }
}

#[derive(Debug, Clone)]
enum WorkspaceDataRepository {
    Memory(InMemoryContextGraphRepository),
    Postgres(PostgresContextGraphRepository),
}

impl WorkspaceDataRepository {
    fn postgres_authorization_audit_sink(&self) -> Option<PostgresContextGraphRepository> {
        match self {
            Self::Memory(_) => None,
            Self::Postgres(repository) => Some(repository.clone()),
        }
    }

    fn postgres_runtime_repository(&self) -> Option<PostgresContextGraphRepository> {
        match self {
            Self::Memory(_) => None,
            Self::Postgres(repository) => Some(repository.clone()),
        }
    }

    fn context_diff_snapshot_repository(&self) -> Arc<dyn ContextDiffSnapshotV1ReviewRepository> {
        match self {
            Self::Memory(repository) => Arc::new(repository.clone()),
            Self::Postgres(repository) => Arc::new(repository.clone()),
        }
    }
}

#[async_trait::async_trait]
impl ContextGraphProjectionRepository for WorkspaceDataRepository {
    async fn load_context_graph_projection(
        &self,
        scope: contextlab_storage::GraphProjectionScope,
    ) -> Result<contextlab_storage::ContextGraphProjection, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.load_context_graph_projection(scope).await,
            Self::Postgres(repository) => repository.load_context_graph_projection(scope).await,
        }
    }
}

#[async_trait::async_trait]
impl BenchmarkWorkspaceProjectionV1Reader for WorkspaceDataRepository {
    async fn read_benchmark_workspace_projection(
        &self,
        query: contextlab_storage::BenchmarkWorkspaceProjectionV1Query,
    ) -> Result<
        contextlab_evaluation::BenchmarkWorkspaceProjectionV1,
        contextlab_storage::BenchmarkWorkspaceProjectionPersistenceError,
    > {
        match self {
            Self::Memory(repository) => repository.read_benchmark_workspace_projection(query).await,
            Self::Postgres(repository) => {
                repository.read_benchmark_workspace_projection(query).await
            }
        }
    }

    async fn resolve_benchmark_workspace_projection_scope(
        &self,
        query: BenchmarkWorkspaceProjectionDecisionQuery,
    ) -> Result<
        Option<contextlab_storage::BenchmarkWorkspaceProjectionReceiptScope>,
        contextlab_storage::BenchmarkWorkspaceProjectionPersistenceError,
    > {
        match self {
            Self::Memory(repository) => {
                repository
                    .resolve_benchmark_workspace_projection_scope(query)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .resolve_benchmark_workspace_projection_scope(query)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl KnowledgeMemoryProjectionV1Repository for WorkspaceDataRepository {
    async fn persist_knowledge_memory_projection(
        &self,
        command: PersistKnowledgeMemoryProjectionV1,
    ) -> Result<KnowledgeMemoryProjectionWriteResult, KnowledgeMemoryProjectionPersistenceError>
    {
        match self {
            Self::Memory(repository) => {
                repository
                    .persist_knowledge_memory_projection(command)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .persist_knowledge_memory_projection(command)
                    .await
            }
        }
    }

    async fn read_knowledge_memory_projection(
        &self,
        scope: KnowledgeMemoryProjectionScope,
    ) -> Result<
        contextlab_knowledge::KnowledgeMemoryContextProjectionV1,
        KnowledgeMemoryProjectionPersistenceError,
    > {
        match self {
            Self::Memory(repository) => repository.read_knowledge_memory_projection(scope).await,
            Self::Postgres(repository) => repository.read_knowledge_memory_projection(scope).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextLifecycleRootRepository for WorkspaceDataRepository {
    async fn get_context_lifecycle_root(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<ContextLifecycleRoot, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.get_context_lifecycle_root(context_id).await,
            Self::Postgres(repository) => repository.get_context_lifecycle_root(context_id).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextLifecycleReadRepository for WorkspaceDataRepository {
    async fn get_context_lifecycle_read_facts(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
    ) -> Result<ContextLifecycleReadFacts, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_context_lifecycle_read_facts(context_id, commit_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_context_lifecycle_read_facts(context_id, commit_id)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl ContextWorkflowBindingRepository for WorkspaceDataRepository {
    async fn persist_workflow_context_binding(
        &self,
        binding: contextlab_workflow::WorkflowContextBinding,
    ) -> Result<contextlab_storage::WorkflowContextBindingWriteResult, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.persist_workflow_context_binding(binding).await,
            Self::Postgres(repository) => {
                repository.persist_workflow_context_binding(binding).await
            }
        }
    }

    async fn get_workflow_context_binding(
        &self,
        workflow_id: contextlab_workflow::WorkflowId,
        workflow_revision: contextlab_workflow::WorkflowRevision,
    ) -> Result<Option<contextlab_workflow::WorkflowContextBinding>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_workflow_context_binding(workflow_id, workflow_revision)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_workflow_context_binding(workflow_id, workflow_revision)
                    .await
            }
        }
    }

    async fn list_workflow_context_bindings_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
    ) -> Result<Vec<contextlab_workflow::WorkflowContextBinding>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .list_workflow_context_bindings_at_commit(context_id, commit_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .list_workflow_context_bindings_at_commit(context_id, commit_id)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl BenchmarkDefinitionBindingRepository for WorkspaceDataRepository {
    async fn get_benchmark_definition_binding(
        &self,
        project_id: contextlab_context_core::ProjectId,
        context_id: contextlab_context_core::ContextId,
        context_commit_id: contextlab_versioning::CommitId,
        binding_id: contextlab_storage::BenchmarkDefinitionBindingId,
    ) -> Result<Option<contextlab_storage::BenchmarkDefinitionBinding>, StorageRepositoryError>
    {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_benchmark_definition_binding(
                        project_id,
                        context_id,
                        context_commit_id,
                        binding_id,
                    )
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_benchmark_definition_binding(
                        project_id,
                        context_id,
                        context_commit_id,
                        binding_id,
                    )
                    .await
            }
        }
    }

    async fn list_benchmark_definition_bindings_at_commit(
        &self,
        project_id: contextlab_context_core::ProjectId,
        context_id: contextlab_context_core::ContextId,
        context_commit_id: contextlab_versioning::CommitId,
    ) -> Result<Vec<contextlab_storage::BenchmarkDefinitionBinding>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .list_benchmark_definition_bindings_at_commit(
                        project_id,
                        context_id,
                        context_commit_id,
                    )
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .list_benchmark_definition_bindings_at_commit(
                        project_id,
                        context_id,
                        context_commit_id,
                    )
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl BenchmarkDefinitionBindingWriter for WorkspaceDataRepository {
    async fn persist_benchmark_definition_binding(
        &self,
        command: BenchmarkDefinitionBindingCommand,
    ) -> Result<BenchmarkDefinitionBindingWriteResult, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .persist_benchmark_definition_binding(command)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .persist_benchmark_definition_binding(command)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl WorkspaceRepository for WorkspaceDataRepository {
    async fn list_workspaces(
        &self,
        query: contextlab_storage::WorkspaceListQuery,
    ) -> Result<contextlab_storage::WorkspaceList, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.list_workspaces(query).await,
            Self::Postgres(repository) => repository.list_workspaces(query).await,
        }
    }
}

#[async_trait::async_trait]
impl ProjectRepository for WorkspaceDataRepository {
    async fn list_projects(
        &self,
        workspace_id: String,
        query: contextlab_storage::ProjectListQuery,
    ) -> Result<contextlab_storage::ProjectList, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.list_projects(workspace_id, query).await,
            Self::Postgres(repository) => repository.list_projects(workspace_id, query).await,
        }
    }
}

#[async_trait::async_trait]
impl ExperimentRepository for WorkspaceDataRepository {
    async fn list_experiments(
        &self,
        project_id: String,
        query: contextlab_storage::ExperimentListQuery,
    ) -> Result<contextlab_storage::ExperimentList, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.list_experiments(project_id, query).await,
            Self::Postgres(repository) => repository.list_experiments(project_id, query).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextRepository for WorkspaceDataRepository {
    async fn list_contexts(
        &self,
        project_id: String,
        query: contextlab_storage::ContextListQuery,
    ) -> Result<contextlab_storage::ContextList, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.list_contexts(project_id, query).await,
            Self::Postgres(repository) => repository.list_contexts(project_id, query).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextCommitRepository for WorkspaceDataRepository {
    async fn list_commits(
        &self,
        context_id: String,
        query: contextlab_storage::CommitListQuery,
    ) -> Result<contextlab_storage::CommitList, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.list_commits(context_id, query).await,
            Self::Postgres(repository) => repository.list_commits(context_id, query).await,
        }
    }

    async fn get_commit(
        &self,
        context_id: String,
        commit_id: String,
    ) -> Result<contextlab_storage::CommitDetail, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.get_commit(context_id, commit_id).await,
            Self::Postgres(repository) => repository.get_commit(context_id, commit_id).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextCommitHistoryRepository for WorkspaceDataRepository {
    async fn load_context_commit_history(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<contextlab_versioning::CommitHistory, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.load_context_commit_history(context_id).await,
            Self::Postgres(repository) => repository.load_context_commit_history(context_id).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextMergeReviewWitnessRepository for WorkspaceDataRepository {
    async fn load_context_merge_review_witness(
        &self,
        scope: ContextMergeTipScope,
    ) -> Result<ContextMergeReviewWitness, ContextMergeReviewWitnessRepositoryError> {
        match self {
            Self::Memory(repository) => repository.load_context_merge_review_witness(scope).await,
            Self::Postgres(repository) => repository.load_context_merge_review_witness(scope).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextGraphReviewWitnessRepository for WorkspaceDataRepository {
    async fn read_context_graph_review_witness(
        &self,
        source_scope: contextlab_storage::CommitGraphSnapshotScope,
        target_scope: contextlab_storage::CommitGraphSnapshotScope,
    ) -> Result<contextlab_storage::ContextGraphReviewWitness, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .read_context_graph_review_witness(source_scope, target_scope)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .read_context_graph_review_witness(source_scope, target_scope)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl ContextCommitGraphRepository for WorkspaceDataRepository {
    async fn load_context_commit_graph(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<contextlab_versioning::CommitGraph, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.load_context_commit_graph(context_id).await,
            Self::Postgres(repository) => repository.load_context_commit_graph(context_id).await,
        }
    }
}

#[async_trait::async_trait]
impl CommitGraphSnapshotRepository for WorkspaceDataRepository {
    async fn project_id_for_context(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<contextlab_context_core::ProjectId, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.project_id_for_context(context_id).await,
            Self::Postgres(repository) => repository.project_id_for_context(context_id).await,
        }
    }

    async fn scope_for_context_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
    ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .scope_for_context_commit(context_id, commit_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .scope_for_context_commit(context_id, commit_id)
                    .await
            }
        }
    }

    async fn get_commit_graph_snapshot(
        &self,
        scope: CommitGraphSnapshotScope,
    ) -> Result<Option<contextlab_storage::CommitGraphSnapshot>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.get_commit_graph_snapshot(scope).await,
            Self::Postgres(repository) => repository.get_commit_graph_snapshot(scope).await,
        }
    }

    async fn get_commit_graph_snapshot_batch(
        &self,
        scope: ContextMergeInputScope,
    ) -> Result<
        (
            contextlab_storage::CommitGraphSnapshot,
            contextlab_storage::CommitGraphSnapshot,
            contextlab_storage::CommitGraphSnapshot,
        ),
        StorageRepositoryError,
    > {
        match self {
            Self::Memory(repository) => repository.get_commit_graph_snapshot_batch(scope).await,
            Self::Postgres(repository) => repository.get_commit_graph_snapshot_batch(scope).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextBranchRepository for WorkspaceDataRepository {
    async fn list_context_branch_heads(
        &self,
        context_id: contextlab_context_core::ContextId,
    ) -> Result<
        Vec<contextlab_storage::ContextBranchHead>,
        contextlab_storage::ContextBranchRepositoryError,
    > {
        match self {
            Self::Memory(repository) => repository.list_context_branch_heads(context_id).await,
            Self::Postgres(repository) => repository.list_context_branch_heads(context_id).await,
        }
    }

    async fn get_context_branch_head(
        &self,
        context_id: contextlab_context_core::ContextId,
        branch: contextlab_versioning::BranchName,
    ) -> Result<
        contextlab_storage::ContextBranchHead,
        contextlab_storage::ContextBranchRepositoryError,
    > {
        match self {
            Self::Memory(repository) => {
                repository.get_context_branch_head(context_id, branch).await
            }
            Self::Postgres(repository) => {
                repository.get_context_branch_head(context_id, branch).await
            }
        }
    }
}

#[async_trait::async_trait]
impl ComponentContentRevisionRepository for WorkspaceDataRepository {
    async fn get_component_content_revision(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<contextlab_storage::ComponentContentRevision>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_component_content_revision(context_id, commit_id, component_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_component_content_revision(context_id, commit_id, component_id)
                    .await
            }
        }
    }

    async fn get_component_content_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<contextlab_storage::ComponentContentRevision>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_component_content_at_commit(context_id, commit_id, component_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_component_content_at_commit(context_id, commit_id, component_id)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl ComponentStateAtCommitRepository for WorkspaceDataRepository {
    async fn get_component_state_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
        component_id: contextlab_context_core::ComponentId,
    ) -> Result<Option<contextlab_storage::ComponentStateAtCommit>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_component_state_at_commit(context_id, commit_id, component_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_component_state_at_commit(context_id, commit_id, component_id)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl ContextReplayStateAtCommitRepository for WorkspaceDataRepository {
    async fn get_context_replay_state_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
    ) -> Result<contextlab_versioning::ReplayState, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_context_replay_state_at_commit(context_id, commit_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_context_replay_state_at_commit(context_id, commit_id)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl ContextComponentStateSnapshotAtCommitRepository for WorkspaceDataRepository {
    async fn get_context_component_state_snapshot_at_commit(
        &self,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
    ) -> Result<contextlab_storage::ContextComponentStateSnapshotAtCommit, StorageRepositoryError>
    {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_context_component_state_snapshot_at_commit(context_id, commit_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_context_component_state_snapshot_at_commit(context_id, commit_id)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl GuardedContextCommitWriter for WorkspaceDataRepository {
    async fn create_guarded_commit_snapshot(
        &self,
        command: contextlab_storage::GuardedContextCommitWrite,
    ) -> Result<contextlab_storage::GuardedCommitWriteResult, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.create_guarded_commit_snapshot(command).await,
            Self::Postgres(repository) => repository.create_guarded_commit_snapshot(command).await,
        }
    }
}

#[async_trait::async_trait]
impl ContextComponentRepository for WorkspaceDataRepository {
    async fn list_components(
        &self,
        context_id: String,
        query: contextlab_storage::ComponentListQuery,
    ) -> Result<contextlab_storage::ComponentList, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.list_components(context_id, query).await,
            Self::Postgres(repository) => repository.list_components(context_id, query).await,
        }
    }

    async fn get_component(
        &self,
        context_id: String,
        component_id: String,
    ) -> Result<contextlab_storage::ComponentDetail, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.get_component(context_id, component_id).await,
            Self::Postgres(repository) => repository.get_component(context_id, component_id).await,
        }
    }
}

#[async_trait::async_trait]
impl EvaluationRunRepository for WorkspaceDataRepository {
    async fn list_evaluation_runs(
        &self,
        context_id: String,
        query: contextlab_storage::EvaluationRunListQuery,
    ) -> Result<contextlab_storage::EvaluationRunList, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.list_evaluation_runs(context_id, query).await,
            Self::Postgres(repository) => repository.list_evaluation_runs(context_id, query).await,
        }
    }

    async fn get_evaluation_run(
        &self,
        context_id: String,
        run_id: String,
    ) -> Result<contextlab_storage::EvaluationRunDetail, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.get_evaluation_run(context_id, run_id).await,
            Self::Postgres(repository) => repository.get_evaluation_run(context_id, run_id).await,
        }
    }

    async fn get_evaluation_scorecard(
        &self,
        context_id: String,
        query: contextlab_storage::EvaluationScorecardQuery,
    ) -> Result<contextlab_storage::EvaluationScorecard, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository.get_evaluation_scorecard(context_id, query).await
            }
            Self::Postgres(repository) => {
                repository.get_evaluation_scorecard(context_id, query).await
            }
        }
    }
}

#[async_trait::async_trait]
impl BenchmarkEvidenceRepository for WorkspaceDataRepository {
    async fn get_benchmark_dataset(
        &self,
        project_id: contextlab_context_core::ProjectId,
        dataset_id: contextlab_evaluation::BenchmarkDatasetId,
    ) -> Result<Option<contextlab_evaluation::BenchmarkDataset>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_benchmark_dataset(project_id, dataset_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_benchmark_dataset(project_id, dataset_id)
                    .await
            }
        }
    }

    async fn get_benchmark_suite(
        &self,
        project_id: contextlab_context_core::ProjectId,
        suite_id: contextlab_evaluation::BenchmarkSuiteId,
    ) -> Result<Option<contextlab_evaluation::BenchmarkSuite>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => repository.get_benchmark_suite(project_id, suite_id).await,
            Self::Postgres(repository) => {
                repository.get_benchmark_suite(project_id, suite_id).await
            }
        }
    }

    async fn get_benchmark_run(
        &self,
        project_id: contextlab_context_core::ProjectId,
        context_id: contextlab_context_core::ContextId,
        context_commit_id: contextlab_versioning::CommitId,
        run_id: contextlab_evaluation::EvaluationRunId,
    ) -> Result<Option<contextlab_evaluation::EvaluationRun>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_benchmark_run(project_id, context_id, context_commit_id, run_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_benchmark_run(project_id, context_id, context_commit_id, run_id)
                    .await
            }
        }
    }

    async fn get_benchmark_decision(
        &self,
        project_id: contextlab_context_core::ProjectId,
        context_id: contextlab_context_core::ContextId,
        context_commit_id: contextlab_versioning::CommitId,
        decision_id: contextlab_storage::BenchmarkDecisionId,
    ) -> Result<Option<contextlab_storage::BenchmarkDecisionEvidence>, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_benchmark_decision(project_id, context_id, context_commit_id, decision_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_benchmark_decision(project_id, context_id, context_commit_id, decision_id)
                    .await
            }
        }
    }

    async fn get_benchmark_decision_pair(
        &self,
        project_id: contextlab_context_core::ProjectId,
        context_id: contextlab_context_core::ContextId,
        baseline: BenchmarkDecisionComparisonScope,
        revised: BenchmarkDecisionComparisonScope,
    ) -> Result<BenchmarkDecisionPair, StorageRepositoryError> {
        match self {
            Self::Memory(repository) => {
                repository
                    .get_benchmark_decision_pair(project_id, context_id, baseline, revised)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .get_benchmark_decision_pair(project_id, context_id, baseline, revised)
                    .await
            }
        }
    }
}

#[async_trait::async_trait]
impl BenchmarkDecisionDiscoveryRepository for WorkspaceDataRepository {
    async fn list_benchmark_decisions(
        &self,
        project_id: contextlab_context_core::ProjectId,
        context_id: contextlab_context_core::ContextId,
        context_commit_id: contextlab_versioning::CommitId,
    ) -> Result<Vec<contextlab_storage::BenchmarkDecisionDiscoverySummary>, StorageRepositoryError>
    {
        match self {
            Self::Memory(repository) => {
                repository
                    .list_benchmark_decisions(project_id, context_id, context_commit_id)
                    .await
            }
            Self::Postgres(repository) => {
                repository
                    .list_benchmark_decisions(project_id, context_id, context_commit_id)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod context_lifecycle_contract_tests {
    use super::WorkspaceDataRepository;
    use chrono::{TimeZone, Utc};
    use contextlab_auth::{
        AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity,
    };
    use contextlab_context_core::{ContextId, ContextMetadata, ProjectId};
    use contextlab_graph::ContextGraph;
    use contextlab_storage::{
        CommitGraphSnapshotRepository, ContextCommitGraphRepository, ContextLifecycleCommand,
        ContextLifecycleRepository, ContextLifecycleService, CreateContextCommitSnapshot,
        GuardedContextCommitWrite, GuardedContextCommitWriter, IdempotencyKey,
        InMemoryContextGraphRepository, PersistedContextDiffReviewService, RequestDigest,
    };
    use contextlab_versioning::{BranchName, ContextCommit, ExpectedBranchHead};
    use uuid::Uuid;

    #[test]
    fn workspace_data_repository_satisfies_the_private_lifecycle_contract() {
        fn assert_context_lifecycle_repository<Repository: ContextLifecycleRepository>() {}

        assert_context_lifecycle_repository::<WorkspaceDataRepository>();
    }

    #[test]
    fn workspace_data_repository_satisfies_the_typed_snapshot_contract() {
        fn assert_commit_graph_snapshot_repository<Repository: CommitGraphSnapshotRepository>() {}

        assert_commit_graph_snapshot_repository::<WorkspaceDataRepository>();
    }

    #[test]
    fn workspace_data_repository_satisfies_the_exact_commit_graph_contract() {
        fn assert_context_commit_graph_repository<Repository: ContextCommitGraphRepository>() {}

        assert_context_commit_graph_repository::<WorkspaceDataRepository>();
    }

    #[test]
    fn workspace_data_repository_exposes_the_private_diff_snapshot_adapter() {
        let memory = WorkspaceDataRepository::Memory(
            InMemoryContextGraphRepository::context_engineering_preview(),
        );

        let memory_adapter = memory.context_diff_snapshot_repository();
        assert_eq!(std::sync::Arc::strong_count(&memory_adapter), 1);
    }

    #[tokio::test]
    async fn memory_writer_state_is_readable_through_the_persisted_diff_snapshot_adapter() {
        let project_id = ProjectId::from_uuid(Uuid::from_u128(1));
        let context_id = ContextId::from_uuid(Uuid::from_u128(2));
        let mut projection =
            contextlab_storage::ContextGraphProjection::context_engineering_preview();
        projection.projects[0].id = project_id.to_string();
        projection.experiments[0].project_id = project_id.to_string();
        projection.contexts[0].id = context_id.to_string();
        projection.contexts[0].project_id = project_id.to_string();
        for component in &mut projection.components {
            component.context_id = context_id.to_string();
        }
        for evaluation_run in &mut projection.evaluation_runs {
            evaluation_run.context_id = context_id.to_string();
        }
        let commit = ContextCommit::new(
            context_id,
            BranchName::default(),
            "Persist Context snapshot",
            Vec::new(),
            Vec::new(),
            Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0)
                .single()
                .expect("timestamp"),
        )
        .expect("commit");
        let commit_id = commit.id();
        let snapshot_command = CreateContextCommitSnapshot::new(
            project_id,
            commit,
            ContextGraph::new(),
            Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 1)
                .single()
                .expect("snapshot timestamp"),
            1,
        )
        .expect("snapshot command");
        let guarded_command = GuardedContextCommitWrite::new(
            AuthenticatedPrincipal::new(PrincipalIdentity::new(
                IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new("user:memory-composition").expect("principal"),
            )),
            ExpectedBranchHead::Unborn,
            IdempotencyKey::new("memory-composition-request").expect("idempotency key"),
            RequestDigest::new("memory-composition-digest").expect("request digest"),
            snapshot_command,
        )
        .expect("guarded command");
        let memory =
            WorkspaceDataRepository::Memory(InMemoryContextGraphRepository::new(projection));

        memory
            .create_guarded_commit_snapshot(guarded_command)
            .await
            .expect("persisted commit snapshot");

        let scope =
            contextlab_diff_engine::VersionedContextScopeV1::new(project_id, context_id, commit_id);
        let persisted = memory
            .context_diff_snapshot_repository()
            .read_context_diff_snapshot(scope)
            .await
            .expect("persisted diff snapshot");

        assert_eq!(persisted.scope(), scope);
    }

    #[tokio::test]
    async fn memory_guarded_lifecycle_metadata_update_is_visible_in_persisted_diff_review() {
        let project_id = ProjectId::from_uuid(Uuid::from_u128(1));
        let context_id = ContextId::from_uuid(Uuid::from_u128(2));
        let mut projection =
            contextlab_storage::ContextGraphProjection::context_engineering_preview();
        projection.projects[0].id = project_id.to_string();
        projection.experiments[0].project_id = project_id.to_string();
        projection.contexts[0].id = context_id.to_string();
        projection.contexts[0].project_id = project_id.to_string();
        for component in &mut projection.components {
            component.context_id = context_id.to_string();
        }
        for evaluation_run in &mut projection.evaluation_runs {
            evaluation_run.context_id = context_id.to_string();
        }
        let memory =
            WorkspaceDataRepository::Memory(InMemoryContextGraphRepository::new(projection));
        let lifecycle = ContextLifecycleService::new(&memory);
        let branch = BranchName::default();
        let principal = AuthenticatedPrincipal::new(PrincipalIdentity::new(
            IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:memory-lifecycle-metadata").expect("principal"),
        ));

        let root = lifecycle
            .execute(ContextLifecycleCommand::initialize(
                principal.clone(),
                context_id,
                branch.clone(),
                IdempotencyKey::new("memory-lifecycle-root").expect("idempotency key"),
                RequestDigest::new("memory-lifecycle-root-digest").expect("request digest"),
                "Initialize Context lifecycle",
                Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 2)
                    .single()
                    .expect("root timestamp"),
            ))
            .await
            .expect("persist guarded lifecycle root");

        let mut initial_metadata = ContextMetadata::new(
            Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 3)
                .single()
                .expect("initial metadata timestamp"),
        );
        initial_metadata.set_label(
            "owner",
            "context-platform",
            Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 4)
                .single()
                .expect("initial metadata update timestamp"),
        );
        let initial = lifecycle
            .execute(ContextLifecycleCommand::update_metadata(
                principal.clone(),
                context_id,
                branch.clone(),
                root.commit_id(),
                IdempotencyKey::new("memory-lifecycle-metadata-initial").expect("idempotency key"),
                RequestDigest::new("memory-lifecycle-metadata-initial-digest")
                    .expect("request digest"),
                "Set initial Context metadata",
                initial_metadata.clone(),
                Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 4)
                    .single()
                    .expect("initial update timestamp"),
            ))
            .await
            .expect("persist initial guarded metadata update");

        let mut inherited_metadata = initial_metadata.clone();
        inherited_metadata.set_label(
            "reviewed",
            "true",
            Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 5)
                .single()
                .expect("successor metadata update timestamp"),
        );
        let successor = lifecycle
            .execute(ContextLifecycleCommand::update_metadata(
                principal,
                context_id,
                branch,
                initial.commit_id(),
                IdempotencyKey::new("memory-lifecycle-metadata-successor")
                    .expect("idempotency key"),
                RequestDigest::new("memory-lifecycle-metadata-successor-digest")
                    .expect("request digest"),
                "Review Context metadata",
                inherited_metadata.clone(),
                Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 5)
                    .single()
                    .expect("successor update timestamp"),
            ))
            .await
            .expect("persist inherited guarded metadata update");

        let adapter = memory.context_diff_snapshot_repository();
        let review = PersistedContextDiffReviewService::new(adapter.as_ref())
            .review(
                contextlab_diff_engine::VersionedContextScopeV1::new(
                    project_id,
                    context_id,
                    initial.commit_id(),
                ),
                contextlab_diff_engine::VersionedContextScopeV1::new(
                    project_id,
                    context_id,
                    successor.commit_id(),
                ),
            )
            .await
            .expect("review persisted lifecycle metadata diff");

        assert_eq!(
            review.diff().semantic().metadata_change(),
            Some(&contextlab_diff_engine::ContextMetadataChangeV1::Modified {
                original: initial_metadata,
                revised: inherited_metadata,
            })
        );
    }
}

fn env_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .find_map(|(candidate, value)| (candidate == key).then_some(value.as_str()))
}

fn required_protected_auth_value(
    env: &[(String, String)],
    key: &str,
    error: AppStateConfigError,
) -> Result<String, AppStateConfigError> {
    let value = env_value(env, key)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    value.map(str::to_owned).ok_or(error)
}

fn database_url_from_env(env: &[(String, String)]) -> Result<&str, AppStateConfigError> {
    env_value(env, "CONTEXTLAB_DATABASE_URL")
        .or_else(|| env_value(env, "DATABASE_URL"))
        .filter(|value| !value.trim().is_empty())
        .ok_or(AppStateConfigError::MissingDatabaseUrl)
}

/// Returns the API bind address from environment variables.
#[must_use]
pub fn api_address() -> SocketAddr {
    let host = std::env::var("CONTEXTLAB_API_HOST")
        .ok()
        .and_then(|value| value.parse::<IpAddr>().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

    let port = std::env::var("CONTEXTLAB_API_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3100);

    SocketAddr::new(host, port)
}

#[cfg(test)]
mod tests {
    use super::knowledge_memory::KnowledgeMemoryProjectionRepository;
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use chrono::{TimeZone, Utc};
    use contextlab_auth::{
        AuthenticatedPrincipal, AuthorizationAuditError, AuthorizationAuditEvent,
        AuthorizationAuditSink, AuthorizationDecision, AuthorizationError, ContextAuthorizer,
        ContextPermission, ProtectedRouteOperation, ProtectedRouteRateLimitKey,
        ProtectedRouteRateLimiter, RateLimitDecision, RateLimitError,
    };
    use contextlab_context_core::{ContextId, ProjectId};
    use contextlab_diff_engine::{
        BehaviorObservationV1, BehaviorOutcomeV1, BehaviorSnapshotV1, ContextDiffSnapshotV1,
        EvaluationMetricObservationV1, EvaluationSnapshotV1, SemanticDocumentV1,
        SemanticSnapshotV1, VersionedContextScopeV1,
    };
    use contextlab_evaluation::{
        BenchmarkCase, BenchmarkCaseExecutionResult, BenchmarkCaseId, BenchmarkDataset,
        BenchmarkDatasetId, BenchmarkEvaluation, BenchmarkExecutionPlan, BenchmarkExecutionReceipt,
        BenchmarkExpectedOutput, BenchmarkSuite, BenchmarkSuiteId, BenchmarkWorkspaceProjectionV1,
        EvaluationRun, MetricKind, MetricMeasurement, RegressionThreshold, ThresholdDirection,
    };
    use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
    use contextlab_model_gateway::ProviderRegistry;
    use contextlab_storage::{
        BenchmarkCaseEvaluationRequest, BenchmarkCaseEvaluator, BenchmarkCaseEvaluatorError,
        BenchmarkDecisionComparisonScope, BenchmarkDecisionDiscoveryRepository,
        BenchmarkDecisionEvidence, BenchmarkDecisionId, BenchmarkDecisionPair,
        BenchmarkEvidenceRepository, BenchmarkEvidenceWriter,
        BenchmarkWorkspaceProjectionDecisionQuery, BenchmarkWorkspaceProjectionPersistenceError,
        BenchmarkWorkspaceProjectionReceiptScope, BenchmarkWorkspaceProjectionV1Query,
        BenchmarkWorkspaceProjectionV1Reader, COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
        CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1, CommitGraphSnapshot, CommitGraphSnapshotScope,
        ContextCommitRecord, ContextDiffSnapshotV1Repository, ContextGraphProjection,
        ContextGraphReviewWitness, ContextMergeReviewWitness, ContextMergeReviewWitnessRepository,
        ContextMergeReviewWitnessRepositoryError, ContextMergeTipScope, ContextRecord,
        ContextWorkflowBindingRepository, InMemoryContextDiffSnapshotV1Repository,
        InMemoryContextGraphRepository, PersistBenchmarkEvaluationEvidence,
        PersistContextDiffSnapshotV1, ProjectRecord, StorageRepositoryError,
    };
    use contextlab_versioning::CommitId;
    use contextlab_workflow::{
        ContextCommitSource, WorkflowContextBinding, WorkflowContextBindingId, WorkflowDefinition,
        WorkflowEdge, WorkflowEdgeId, WorkflowId, WorkflowNode, WorkflowNodeId, WorkflowRevision,
        WorkflowRunId,
    };
    use http_body_util::BodyExt;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde_json::Value;
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;
    use uuid::Uuid;

    struct AllowContextWrites;

    #[async_trait::async_trait]
    impl ContextAuthorizer for AllowContextWrites {
        async fn authorize(
            &self,
            _principal: &AuthenticatedPrincipal,
            _context_id: ContextId,
            _permission: ContextPermission,
        ) -> Result<(), AuthorizationError> {
            Ok(())
        }
    }

    struct ReadOnlyContextAccess;

    #[async_trait::async_trait]
    impl ContextAuthorizer for ReadOnlyContextAccess {
        async fn authorize(
            &self,
            _principal: &AuthenticatedPrincipal,
            _context_id: ContextId,
            permission: ContextPermission,
        ) -> Result<(), AuthorizationError> {
            match permission {
                ContextPermission::Read => Ok(()),
                ContextPermission::Write => Err(AuthorizationError::Forbidden),
            }
        }
    }

    struct DenyContextReads;

    #[async_trait::async_trait]
    impl ContextAuthorizer for DenyContextReads {
        async fn authorize(
            &self,
            _principal: &AuthenticatedPrincipal,
            _context_id: ContextId,
            _permission: ContextPermission,
        ) -> Result<(), AuthorizationError> {
            Err(AuthorizationError::Forbidden)
        }
    }

    struct UnavailableContextWrites;

    #[async_trait::async_trait]
    impl ContextAuthorizer for UnavailableContextWrites {
        async fn authorize(
            &self,
            _principal: &AuthenticatedPrincipal,
            _context_id: ContextId,
            _permission: ContextPermission,
        ) -> Result<(), AuthorizationError> {
            Err(AuthorizationError::Unavailable)
        }
    }

    #[derive(Clone)]
    struct RecordingBenchmarkEvaluator {
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl BenchmarkCaseEvaluator for RecordingBenchmarkEvaluator {
        async fn evaluate_case(
            &self,
            request: BenchmarkCaseEvaluationRequest,
        ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            BenchmarkCaseExecutionResult::new(
                request.case().dataset_id(),
                request.case().case_id(),
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.95)
                        .map_err(|_| BenchmarkCaseEvaluatorError::new("invalid test metric"))?,
                ],
            )
            .map_err(|_| BenchmarkCaseEvaluatorError::new("invalid test result"))
        }
    }

    type BenchmarkDecisionReadCall = (ProjectId, ContextId, CommitId, BenchmarkDecisionId);
    type BenchmarkDecisionReadCalls = Arc<std::sync::Mutex<Vec<BenchmarkDecisionReadCall>>>;
    type BenchmarkDecisionComparisonCall = (
        ProjectId,
        ContextId,
        BenchmarkDecisionComparisonScope,
        BenchmarkDecisionComparisonScope,
    );
    type BenchmarkDecisionComparisonCalls =
        Arc<std::sync::Mutex<Vec<BenchmarkDecisionComparisonCall>>>;

    struct RecordingBenchmarkDecisionRepository {
        decision: Option<BenchmarkDecisionEvidence>,
        suite: Option<BenchmarkSuite>,
        datasets: Vec<BenchmarkDataset>,
        fail_reads: bool,
        calls: BenchmarkDecisionReadCalls,
    }

    struct RecordingBenchmarkDecisionComparisonRepository {
        decision: Option<BenchmarkDecisionEvidence>,
        fail_reads: bool,
        calls: BenchmarkDecisionComparisonCalls,
    }

    #[derive(Clone)]
    struct RecordingBenchmarkWorkspaceRepository {
        projection: BenchmarkWorkspaceProjectionV1,
        calls: Arc<std::sync::Mutex<Vec<BenchmarkWorkspaceProjectionV1Query>>>,
    }

    #[async_trait::async_trait]
    impl BenchmarkWorkspaceProjectionV1Reader for RecordingBenchmarkWorkspaceRepository {
        async fn read_benchmark_workspace_projection(
            &self,
            query: BenchmarkWorkspaceProjectionV1Query,
        ) -> Result<BenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError>
        {
            self.calls
                .lock()
                .expect("benchmark workspace calls lock")
                .push(query);
            Ok(self.projection.clone())
        }

        async fn resolve_benchmark_workspace_projection_scope(
            &self,
            query: BenchmarkWorkspaceProjectionDecisionQuery,
        ) -> Result<
            Option<BenchmarkWorkspaceProjectionReceiptScope>,
            BenchmarkWorkspaceProjectionPersistenceError,
        > {
            if query.decision_id().as_uuid() == Uuid::from_u128(404) {
                return Ok(None);
            }
            Ok(Some(BenchmarkWorkspaceProjectionReceiptScope::new(
                query.project_id(),
                query.context_id(),
                query.context_commit_id(),
                self.projection.receipt().cohort_id(),
            )))
        }
    }

    #[derive(Clone, Copy)]
    enum BenchmarkWorkspaceFailure {
        Missing,
        Comparison,
        StoredSource,
        Repository,
    }

    #[derive(Clone, Copy)]
    struct FailingBenchmarkWorkspaceRepository {
        failure: BenchmarkWorkspaceFailure,
    }

    #[async_trait::async_trait]
    impl BenchmarkWorkspaceProjectionV1Reader for FailingBenchmarkWorkspaceRepository {
        async fn read_benchmark_workspace_projection(
            &self,
            _query: BenchmarkWorkspaceProjectionV1Query,
        ) -> Result<BenchmarkWorkspaceProjectionV1, BenchmarkWorkspaceProjectionPersistenceError>
        {
            Err(match self.failure {
                BenchmarkWorkspaceFailure::Missing => {
                    BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable {
                        scope: BenchmarkWorkspaceProjectionReceiptScope::new(
                            ProjectId::from_uuid(Uuid::from_u128(111)),
                            ContextId::from_uuid(Uuid::from_u128(112)),
                            CommitId::from_uuid(Uuid::from_u128(113)),
                            contextlab_evaluation::BenchmarkExecutionCohortId::from_uuid(
                                Uuid::from_u128(114),
                            ),
                        ),
                    }
                }
                BenchmarkWorkspaceFailure::Comparison => {
                    BenchmarkWorkspaceProjectionPersistenceError::ComparisonPlanMismatch
                }
                BenchmarkWorkspaceFailure::StoredSource => {
                    BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid
                }
                BenchmarkWorkspaceFailure::Repository => {
                    BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable
                }
            })
        }
    }

    #[derive(Clone)]
    struct RecordingWorkflowBindingRepository {
        bindings: Vec<WorkflowContextBinding>,
        calls: Arc<std::sync::Mutex<Vec<(ContextId, CommitId)>>>,
        fail_reads: bool,
    }

    #[async_trait::async_trait]
    impl ContextWorkflowBindingRepository for RecordingWorkflowBindingRepository {
        async fn persist_workflow_context_binding(
            &self,
            _binding: WorkflowContextBinding,
        ) -> Result<contextlab_storage::WorkflowContextBindingWriteResult, StorageRepositoryError>
        {
            Err(StorageRepositoryError::InMemoryStateUnavailable)
        }

        async fn get_workflow_context_binding(
            &self,
            _workflow_id: WorkflowId,
            _workflow_revision: WorkflowRevision,
        ) -> Result<Option<WorkflowContextBinding>, StorageRepositoryError> {
            Err(StorageRepositoryError::InMemoryStateUnavailable)
        }

        async fn list_workflow_context_bindings_at_commit(
            &self,
            context_id: ContextId,
            commit_id: CommitId,
        ) -> Result<Vec<WorkflowContextBinding>, StorageRepositoryError> {
            self.calls
                .lock()
                .expect("workflow binding calls lock")
                .push((context_id, commit_id));
            if self.fail_reads {
                return Err(StorageRepositoryError::InMemoryStateUnavailable);
            }
            Ok(self.bindings.clone())
        }
    }

    #[async_trait::async_trait]
    impl BenchmarkEvidenceRepository for RecordingBenchmarkDecisionComparisonRepository {
        async fn get_benchmark_dataset(
            &self,
            _project_id: ProjectId,
            _dataset_id: contextlab_evaluation::BenchmarkDatasetId,
        ) -> Result<Option<BenchmarkDataset>, StorageRepositoryError> {
            Ok(None)
        }

        async fn get_benchmark_suite(
            &self,
            _project_id: ProjectId,
            _suite_id: contextlab_evaluation::BenchmarkSuiteId,
        ) -> Result<Option<BenchmarkSuite>, StorageRepositoryError> {
            Ok(None)
        }

        async fn get_benchmark_run(
            &self,
            _project_id: ProjectId,
            _context_id: ContextId,
            _context_commit_id: CommitId,
            _run_id: contextlab_evaluation::EvaluationRunId,
        ) -> Result<Option<EvaluationRun>, StorageRepositoryError> {
            Ok(None)
        }

        async fn get_benchmark_decision(
            &self,
            _project_id: ProjectId,
            _context_id: ContextId,
            _context_commit_id: CommitId,
            _decision_id: BenchmarkDecisionId,
        ) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError> {
            Ok(None)
        }

        async fn get_benchmark_decision_pair(
            &self,
            project_id: ProjectId,
            context_id: ContextId,
            baseline: BenchmarkDecisionComparisonScope,
            revised: BenchmarkDecisionComparisonScope,
        ) -> Result<BenchmarkDecisionPair, StorageRepositoryError> {
            self.calls
                .lock()
                .expect("benchmark decision comparison calls lock")
                .push((project_id, context_id, baseline, revised));
            if self.fail_reads {
                return Err(StorageRepositoryError::InMemoryStateUnavailable);
            }
            Ok(BenchmarkDecisionPair::new(
                self.decision.clone(),
                self.decision.clone(),
            ))
        }
    }

    #[async_trait::async_trait]
    impl BenchmarkDecisionDiscoveryRepository for RecordingBenchmarkDecisionComparisonRepository {
        async fn list_benchmark_decisions(
            &self,
            _project_id: ProjectId,
            _context_id: ContextId,
            _context_commit_id: CommitId,
        ) -> Result<
            Vec<contextlab_storage::BenchmarkDecisionDiscoverySummary>,
            StorageRepositoryError,
        > {
            Ok(Vec::new())
        }
    }

    #[async_trait::async_trait]
    impl BenchmarkEvidenceRepository for RecordingBenchmarkDecisionRepository {
        async fn get_benchmark_dataset(
            &self,
            _project_id: ProjectId,
            dataset_id: contextlab_evaluation::BenchmarkDatasetId,
        ) -> Result<Option<BenchmarkDataset>, StorageRepositoryError> {
            if self.fail_reads {
                return Err(StorageRepositoryError::InMemoryStateUnavailable);
            }
            Ok(self
                .datasets
                .iter()
                .find(|dataset| dataset.id() == dataset_id)
                .cloned())
        }

        async fn get_benchmark_suite(
            &self,
            _project_id: ProjectId,
            suite_id: contextlab_evaluation::BenchmarkSuiteId,
        ) -> Result<Option<BenchmarkSuite>, StorageRepositoryError> {
            if self.fail_reads {
                return Err(StorageRepositoryError::InMemoryStateUnavailable);
            }
            Ok(self
                .suite
                .as_ref()
                .filter(|suite| suite.id() == suite_id)
                .cloned())
        }

        async fn get_benchmark_run(
            &self,
            _project_id: ProjectId,
            _context_id: ContextId,
            _context_commit_id: CommitId,
            _run_id: contextlab_evaluation::EvaluationRunId,
        ) -> Result<Option<EvaluationRun>, StorageRepositoryError> {
            Ok(None)
        }

        async fn get_benchmark_decision(
            &self,
            project_id: ProjectId,
            context_id: ContextId,
            context_commit_id: CommitId,
            decision_id: BenchmarkDecisionId,
        ) -> Result<Option<BenchmarkDecisionEvidence>, StorageRepositoryError> {
            self.calls
                .lock()
                .expect("benchmark decision calls lock")
                .push((project_id, context_id, context_commit_id, decision_id));
            if self.fail_reads {
                return Err(StorageRepositoryError::InMemoryStateUnavailable);
            }
            Ok(self.decision.clone())
        }

        async fn get_benchmark_decision_pair(
            &self,
            project_id: ProjectId,
            context_id: ContextId,
            baseline: BenchmarkDecisionComparisonScope,
            revised: BenchmarkDecisionComparisonScope,
        ) -> Result<BenchmarkDecisionPair, StorageRepositoryError> {
            let mut calls = self.calls.lock().expect("benchmark decision calls lock");
            calls.push((
                project_id,
                context_id,
                baseline.context_commit_id(),
                baseline.decision_id(),
            ));
            calls.push((
                project_id,
                context_id,
                revised.context_commit_id(),
                revised.decision_id(),
            ));
            drop(calls);
            if self.fail_reads {
                return Err(StorageRepositoryError::InMemoryStateUnavailable);
            }
            Ok(BenchmarkDecisionPair::new(
                self.decision.clone(),
                self.decision.clone(),
            ))
        }
    }

    #[async_trait::async_trait]
    impl BenchmarkDecisionDiscoveryRepository for RecordingBenchmarkDecisionRepository {
        async fn list_benchmark_decisions(
            &self,
            _project_id: ProjectId,
            _context_id: ContextId,
            _context_commit_id: CommitId,
        ) -> Result<
            Vec<contextlab_storage::BenchmarkDecisionDiscoverySummary>,
            StorageRepositoryError,
        > {
            if self.fail_reads {
                return Err(StorageRepositoryError::InMemoryStateUnavailable);
            }
            Ok(Vec::new())
        }
    }

    #[derive(Clone)]
    struct RecordingAuditSink {
        events: std::sync::Arc<std::sync::Mutex<Vec<AuthorizationAuditEvent>>>,
    }

    #[async_trait::async_trait]
    impl AuthorizationAuditSink for RecordingAuditSink {
        async fn record(
            &self,
            event: AuthorizationAuditEvent,
        ) -> Result<(), AuthorizationAuditError> {
            self.events.lock().expect("audit events lock").push(event);
            Ok(())
        }
    }

    struct FailingAuditSink;

    #[async_trait::async_trait]
    impl AuthorizationAuditSink for FailingAuditSink {
        async fn record(
            &self,
            _event: AuthorizationAuditEvent,
        ) -> Result<(), AuthorizationAuditError> {
            Err(AuthorizationAuditError::Unavailable)
        }
    }

    #[derive(Clone)]
    struct StaticRateLimiter {
        result: Result<RateLimitDecision, RateLimitError>,
        calls: Arc<AtomicUsize>,
        keys: Arc<std::sync::Mutex<Vec<ProtectedRouteRateLimitKey>>>,
    }

    impl StaticRateLimiter {
        fn new(result: Result<RateLimitDecision, RateLimitError>) -> Self {
            Self {
                result,
                calls: Arc::new(AtomicUsize::new(0)),
                keys: Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }

        fn operations(&self) -> Vec<ProtectedRouteOperation> {
            self.keys
                .lock()
                .expect("rate-limit keys lock")
                .iter()
                .map(ProtectedRouteRateLimitKey::operation)
                .collect()
        }
    }

    #[async_trait::async_trait]
    impl ProtectedRouteRateLimiter for StaticRateLimiter {
        async fn check(
            &self,
            key: ProtectedRouteRateLimitKey,
        ) -> Result<RateLimitDecision, RateLimitError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.keys.lock().expect("rate-limit keys lock").push(key);
            self.result
        }
    }

    #[derive(serde::Serialize)]
    struct TestJwtClaims {
        sub: String,
        exp: usize,
        iss: String,
        aud: String,
    }

    const TEST_AUTH_ISSUER: &str = "https://issuer.contextlab.test";
    const TEST_AUTH_AUDIENCE: &str = "contextlab-web";

    fn test_rate_limiter() -> contextlab_auth::InMemoryProtectedRouteRateLimiter {
        contextlab_auth::InMemoryProtectedRouteRateLimiter::new(
            contextlab_auth::ProtectedRouteRateLimitPolicy::new(1_000, 3_600, 100)
                .expect("test rate-limit policy"),
        )
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RouteContract {
        operation_id: String,
        path_parameters: Vec<String>,
        query_parameters: Vec<String>,
    }

    fn test_router() -> Router {
        let registry = ProviderRegistry::from_env([
            ("DEEPSEEK_API_BASE_URL", "https://api.deepseek.com"),
            ("DEEPSEEK_API_KEY", "sk-deepseek-test-key"),
            ("OPENAI_COMPAT_API_BASE_URL", "https://codex.hiyo.top"),
            ("OPENAI_COMPAT_API_KEY", "sk-openai-compatible-key"),
        ]);

        build_router_with_state(AppState::new(registry))
    }

    #[tokio::test]
    async fn workspace_repository_bundle_preserves_public_workspace_route() {
        let repository = InMemoryContextGraphRepository::context_engineering_preview();
        let repositories = WorkspaceRepositories::new(
            WorkspaceGraphRepositories::new(
                repository.clone(),
                repository.clone(),
                repository.clone(),
            ),
            WorkspaceCatalogRepositories::new(
                repository.clone(),
                repository.clone(),
                repository.clone(),
                repository.clone(),
                repository.clone(),
                repository.clone(),
                repository.clone(),
                repository,
            ),
        );
        let state = AppState::with_workspace_repositories(
            ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>()),
            repositories,
        );

        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces")
                    .body(Body::empty())
                    .expect("workspace request"),
            )
            .await
            .expect("workspace response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::OK, "projection payload: {payload:?}");
    }

    const VERSION_BACKED_PROJECT_ID: &str = "11111111-1111-4111-8111-111111111111";
    const VERSION_BACKED_CONTEXT_ID: &str = "22222222-2222-4222-8222-222222222222";
    const VERSION_BACKED_ORIGINAL_COMMIT_ID: &str = "33333333-3333-4333-8333-333333333333";
    const VERSION_BACKED_REVISED_COMMIT_ID: &str = "44444444-4444-4444-8444-444444444444";
    const MERGE_REVIEW_RIGHT_COMMIT_ID: &str = "55555555-5555-4555-8555-555555555555";

    fn version_backed_graph_diff_state(
        snapshot_repository: impl CommitGraphSnapshotRepository
        + ContextGraphReviewWitnessRepository
        + Clone
        + 'static,
        context_authorizer: impl ContextAuthorizer + 'static,
    ) -> (AppState, String) {
        let registry = ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>());
        let mut projection = ContextGraphProjection::context_engineering_preview();

        projection.projects[0].id = VERSION_BACKED_PROJECT_ID.to_owned();
        projection.experiments[0].project_id = VERSION_BACKED_PROJECT_ID.to_owned();
        projection.contexts[0].id = VERSION_BACKED_CONTEXT_ID.to_owned();
        projection.contexts[0].project_id = VERSION_BACKED_PROJECT_ID.to_owned();
        for component in &mut projection.components {
            component.context_id = VERSION_BACKED_CONTEXT_ID.to_owned();
        }
        for evaluation_run in &mut projection.evaluation_runs {
            evaluation_run.context_id = VERSION_BACKED_CONTEXT_ID.to_owned();
        }
        projection.commits[0].id = VERSION_BACKED_ORIGINAL_COMMIT_ID.to_owned();
        projection.commits[0].context_id = VERSION_BACKED_CONTEXT_ID.to_owned();
        projection.commits[0].changes =
            serde_json::to_value(vec![contextlab_versioning::ContextChange::created_context(
                "Support Resolution Agent",
            )])
            .expect("serialized root change");
        projection.commits[0].change_count = 1;

        let mut revised_commit = projection.commits[0].clone();
        revised_commit.id = VERSION_BACKED_REVISED_COMMIT_ID.to_owned();
        revised_commit.parent_commit_ids = vec![VERSION_BACKED_ORIGINAL_COMMIT_ID.to_owned()];
        revised_commit.changes = serde_json::to_value(vec![
            contextlab_versioning::ContextChange::updated_metadata(
                contextlab_context_core::ContextMetadata::new(timestamp(2)),
                "Update support resolution context",
            ),
        ])
        .expect("serialized metadata change");
        revised_commit.change_count = 1;
        projection.commits.push(revised_commit);

        let repository = InMemoryContextGraphRepository::new(projection);

        let witness_repository = snapshot_repository.clone();
        let state = AppState::with_workspace_repositories(
            registry,
            WorkspaceRepositories::new(
                WorkspaceGraphRepositories::new(
                    repository.clone(),
                    snapshot_repository,
                    repository.clone(),
                ),
                WorkspaceCatalogRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                ),
            )
            .with_context_commit_history_repository(repository.clone()),
        )
        .with_context_graph_review_witness_repository(witness_repository)
        .with_protected_write_dependencies(
            context_authorizer,
            repository,
            contextlab_auth::HmacJwtAuthenticator::new(
                "test-secret",
                TEST_AUTH_ISSUER,
                TEST_AUTH_AUDIENCE,
            )
            .expect("authenticator"),
            test_rate_limiter(),
        )
        .with_context_lifecycle_repository(version_backed_graph_diff_snapshot_repository());
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestJwtClaims {
                sub: "user:alex".to_owned(),
                exp: (Utc::now().timestamp() + 300) as usize,
                iss: TEST_AUTH_ISSUER.to_owned(),
                aud: TEST_AUTH_AUDIENCE.to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");

        (state, token)
    }

    fn version_backed_graph_diff_snapshot_repository() -> InMemoryContextGraphRepository {
        let mut projection = ContextGraphProjection::context_engineering_preview();
        projection.projects[0].id = VERSION_BACKED_PROJECT_ID.to_owned();
        projection.experiments[0].project_id = VERSION_BACKED_PROJECT_ID.to_owned();
        projection.contexts[0].id = VERSION_BACKED_CONTEXT_ID.to_owned();
        projection.contexts[0].project_id = VERSION_BACKED_PROJECT_ID.to_owned();
        projection.commits[0].id = VERSION_BACKED_ORIGINAL_COMMIT_ID.to_owned();
        projection.commits[0].context_id = VERSION_BACKED_CONTEXT_ID.to_owned();
        projection.commits[0].changes =
            serde_json::to_value(vec![contextlab_versioning::ContextChange::created_context(
                "Support Resolution Agent",
            )])
            .expect("serialized root change");
        projection.commits[0].change_count = 1;
        let mut revised_commit = projection.commits[0].clone();
        revised_commit.id = VERSION_BACKED_REVISED_COMMIT_ID.to_owned();
        revised_commit.parent_commit_ids = vec![VERSION_BACKED_ORIGINAL_COMMIT_ID.to_owned()];
        revised_commit.changes = serde_json::to_value(vec![
            contextlab_versioning::ContextChange::updated_metadata(
                contextlab_context_core::ContextMetadata::new(timestamp(2)),
                "Update support resolution context",
            ),
        ])
        .expect("serialized metadata change");
        revised_commit.change_count = 1;
        projection.commits.push(revised_commit);

        let original_snapshot = CommitGraphSnapshot::new(
            CommitGraphSnapshotScope::new(
                ProjectId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_PROJECT_ID).expect("project id"),
                ),
                ContextId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_CONTEXT_ID).expect("context id"),
                ),
                CommitId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_ORIGINAL_COMMIT_ID).expect("original commit id"),
                ),
            ),
            graph_with_context_id_label(VERSION_BACKED_CONTEXT_ID, "Support Resolution Agent"),
            timestamp(1),
            1,
        )
        .expect("original snapshot");
        let revised_snapshot = CommitGraphSnapshot::new(
            CommitGraphSnapshotScope::new(
                ProjectId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_PROJECT_ID).expect("project id"),
                ),
                ContextId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_CONTEXT_ID).expect("context id"),
                ),
                CommitId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_REVISED_COMMIT_ID).expect("revised commit id"),
                ),
            ),
            graph_with_context_id_label(VERSION_BACKED_CONTEXT_ID, "Support Resolution Agent v2"),
            timestamp(2),
            1,
        )
        .expect("revised snapshot");
        InMemoryContextGraphRepository::with_commit_graph_snapshots(
            projection,
            [original_snapshot, revised_snapshot],
        )
        .expect("snapshot repository")
    }

    #[tokio::test]
    async fn workspace_repository_builder_fails_closed_without_explicit_history_repository() {
        let repository = version_backed_graph_diff_snapshot_repository();
        let state = AppState::with_workspace_repositories(
            ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>()),
            WorkspaceRepositories::new(
                WorkspaceGraphRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                ),
                WorkspaceCatalogRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository,
                ),
            ),
        );

        let error = state
            .context_commit_history_repository()
            .load_context_commit_history(ContextId::from_uuid(
                Uuid::parse_str(VERSION_BACKED_CONTEXT_ID).expect("context id"),
            ))
            .await
            .expect_err("history must not silently fall back to split ports");

        assert!(matches!(
            error,
            StorageRepositoryError::ScopeUnavailable { scope }
                if scope == format!("context_commit_history:{VERSION_BACKED_CONTEXT_ID}")
        ));
    }

    fn version_backed_graph_diff_request(
        context_id: &str,
        original_commit_id: &str,
        revised_commit_id: &str,
        token: Option<&str>,
    ) -> Request<Body> {
        let uri = format!(
            "/api/v1/local/contexts/{context_id}/graph-diff?original_commit_id={original_commit_id}&revised_commit_id={revised_commit_id}"
        );
        let mut builder = Request::builder().uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        builder.body(Body::empty()).expect("graph diff request")
    }

    async fn persisted_context_diff_snapshot_repository() -> InMemoryContextDiffSnapshotV1Repository
    {
        let repository = InMemoryContextDiffSnapshotV1Repository::new();
        let project_id = ProjectId::from_uuid(
            Uuid::parse_str(VERSION_BACKED_PROJECT_ID).expect("project identifier"),
        );
        let context_id = ContextId::from_uuid(
            Uuid::parse_str(VERSION_BACKED_CONTEXT_ID).expect("Context identifier"),
        );

        for (commit_id, graph, captured_at) in [
            (
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                graph_with_context_label("Support Resolution Agent"),
                1,
            ),
            (
                VERSION_BACKED_REVISED_COMMIT_ID,
                graph_with_context_label("Support Resolution Agent v2"),
                2,
            ),
        ] {
            let commit_id = CommitId::from_uuid(Uuid::parse_str(commit_id).expect("commit id"));
            let is_revised = commit_id.to_string() == VERSION_BACKED_REVISED_COMMIT_ID;
            let snapshot = ContextDiffSnapshotV1::new(
                SemanticSnapshotV1::new(graph, Vec::new()).expect("semantic snapshot"),
                BehaviorSnapshotV1::new(vec![
                    BehaviorObservationV1::new(
                        "case:context-review",
                        "input:context-review:v1",
                        if is_revised {
                            BehaviorOutcomeV1::failed("timeout").expect("failure outcome")
                        } else {
                            BehaviorOutcomeV1::succeeded("baseline")
                        },
                    )
                    .expect("behavior observation"),
                ])
                .expect("behavior snapshot"),
                EvaluationSnapshotV1::new(
                    "suite:context-review:v1",
                    vec![
                        EvaluationMetricObservationV1::new(
                            "accuracy",
                            if is_revised { 0.9 } else { 0.8 },
                            10,
                        )
                        .expect("evaluation metric"),
                    ],
                )
                .expect("evaluation snapshot"),
            )
            .expect("complete diff snapshot");
            let command = PersistContextDiffSnapshotV1::new(
                VersionedContextScopeV1::new(project_id, context_id, commit_id),
                CONTEXT_DIFF_SNAPSHOT_SCHEMA_V1,
                snapshot,
                timestamp(captured_at),
            )
            .expect("diff snapshot command");
            repository
                .persist_context_diff_snapshot(command)
                .await
                .expect("persist diff snapshot");
        }

        repository
    }

    fn context_diff_review_request(token: Option<&str>, query: &str) -> Request<Body> {
        let uri = format!(
            "/api/v1/local/projects/{VERSION_BACKED_PROJECT_ID}/contexts/{VERSION_BACKED_CONTEXT_ID}/diff-review?{query}"
        );
        let mut builder = Request::builder().uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        builder
            .body(Body::empty())
            .expect("context diff review request")
    }

    fn persisted_context_diff_review_request(
        project_id: &str,
        context_id: &str,
        source_commit_id: &str,
        target_commit_id: &str,
        token: Option<&str>,
    ) -> Request<Body> {
        let uri = format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/diff-review?source_commit_id={source_commit_id}&target_commit_id={target_commit_id}"
        );
        let mut builder = Request::builder().uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        builder
            .body(Body::empty())
            .expect("Context diff review request")
    }

    async fn persisted_context_diff_review_repository() -> InMemoryContextDiffSnapshotV1Repository {
        let repository = InMemoryContextDiffSnapshotV1Repository::new();
        let project_id =
            ProjectId::from_uuid(Uuid::parse_str(VERSION_BACKED_PROJECT_ID).expect("project id"));
        let context_id =
            ContextId::from_uuid(Uuid::parse_str(VERSION_BACKED_CONTEXT_ID).expect("context id"));
        let source_scope = VersionedContextScopeV1::new(
            project_id,
            context_id,
            CommitId::from_uuid(
                Uuid::parse_str(VERSION_BACKED_ORIGINAL_COMMIT_ID).expect("source commit id"),
            ),
        );
        let target_scope = VersionedContextScopeV1::new(
            project_id,
            context_id,
            CommitId::from_uuid(
                Uuid::parse_str(VERSION_BACKED_REVISED_COMMIT_ID).expect("target commit id"),
            ),
        );
        for (scope, documents) in [
            (source_scope, Vec::new()),
            (
                target_scope,
                vec![
                    SemanticDocumentV1::new("prompt:system", "revised").expect("semantic document"),
                ],
            ),
        ] {
            let is_revised = scope.commit_id() == target_scope.commit_id();
            let snapshot = ContextDiffSnapshotV1::new(
                SemanticSnapshotV1::new(ContextGraph::new(), documents).expect("semantic snapshot"),
                BehaviorSnapshotV1::new(vec![
                    BehaviorObservationV1::new(
                        "case:context-review",
                        "input:context-review:v1",
                        if is_revised {
                            BehaviorOutcomeV1::failed("timeout").expect("failure outcome")
                        } else {
                            BehaviorOutcomeV1::succeeded("baseline")
                        },
                    )
                    .expect("behavior observation"),
                ])
                .expect("behavior snapshot"),
                EvaluationSnapshotV1::new(
                    "suite:review",
                    vec![
                        EvaluationMetricObservationV1::new(
                            "accuracy",
                            if is_revised { 0.9 } else { 0.8 },
                            10,
                        )
                        .expect("evaluation metric"),
                    ],
                )
                .expect("evaluation snapshot"),
            )
            .expect("diff snapshot");
            repository
                .persist_context_diff_snapshot(
                    PersistContextDiffSnapshotV1::new(
                        scope,
                        "context-diff-snapshot-v1",
                        snapshot,
                        timestamp(1),
                    )
                    .expect("persist command"),
                )
                .await
                .expect("persist snapshot");
        }
        repository
    }

    #[derive(Clone)]
    struct RecordingCommitGraphSnapshotRepository {
        inner: InMemoryContextGraphRepository,
        calls: Arc<AtomicUsize>,
    }

    #[derive(Clone)]
    struct RecordingContextCommitGraphRepository {
        inner: InMemoryContextGraphRepository,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl ContextCommitGraphRepository for RecordingContextCommitGraphRepository {
        async fn load_context_commit_graph(
            &self,
            context_id: ContextId,
        ) -> Result<contextlab_versioning::CommitGraph, StorageRepositoryError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.load_context_commit_graph(context_id).await
        }
    }

    #[async_trait::async_trait]
    impl ContextMergeReviewWitnessRepository for RecordingContextCommitGraphRepository {
        async fn load_context_merge_review_witness(
            &self,
            scope: ContextMergeTipScope,
        ) -> Result<ContextMergeReviewWitness, ContextMergeReviewWitnessRepositoryError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.load_context_merge_review_witness(scope).await
        }
    }

    #[async_trait::async_trait]
    impl CommitGraphSnapshotRepository for RecordingCommitGraphSnapshotRepository {
        async fn project_id_for_context(
            &self,
            context_id: ContextId,
        ) -> Result<ProjectId, StorageRepositoryError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.project_id_for_context(context_id).await
        }

        async fn scope_for_context_commit(
            &self,
            context_id: ContextId,
            commit_id: CommitId,
        ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner
                .scope_for_context_commit(context_id, commit_id)
                .await
        }

        async fn get_commit_graph_snapshot(
            &self,
            scope: CommitGraphSnapshotScope,
        ) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.get_commit_graph_snapshot(scope).await
        }
    }

    #[async_trait::async_trait]
    impl ContextGraphReviewWitnessRepository for RecordingCommitGraphSnapshotRepository {
        async fn read_context_graph_review_witness(
            &self,
            source_scope: CommitGraphSnapshotScope,
            target_scope: CommitGraphSnapshotScope,
        ) -> Result<ContextGraphReviewWitness, StorageRepositoryError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner
                .read_context_graph_review_witness(source_scope, target_scope)
                .await
        }
    }

    #[derive(Clone)]
    struct DriftedCommitGraphSnapshotRepository {
        inner: InMemoryContextGraphRepository,
        returned_scope: CommitGraphSnapshotScope,
    }

    #[async_trait::async_trait]
    impl CommitGraphSnapshotRepository for DriftedCommitGraphSnapshotRepository {
        async fn project_id_for_context(
            &self,
            context_id: ContextId,
        ) -> Result<ProjectId, StorageRepositoryError> {
            self.inner.project_id_for_context(context_id).await
        }

        async fn scope_for_context_commit(
            &self,
            _context_id: ContextId,
            _commit_id: CommitId,
        ) -> Result<CommitGraphSnapshotScope, StorageRepositoryError> {
            Ok(self.returned_scope)
        }

        async fn get_commit_graph_snapshot(
            &self,
            scope: CommitGraphSnapshotScope,
        ) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError> {
            self.inner.get_commit_graph_snapshot(scope).await
        }
    }

    #[async_trait::async_trait]
    impl ContextGraphReviewWitnessRepository for DriftedCommitGraphSnapshotRepository {
        async fn read_context_graph_review_witness(
            &self,
            source_scope: CommitGraphSnapshotScope,
            target_scope: CommitGraphSnapshotScope,
        ) -> Result<ContextGraphReviewWitness, StorageRepositoryError> {
            Err(StorageRepositoryError::InvalidScope {
                scope: format!("context_graph_review:{source_scope}/{target_scope}"),
                reason: format!("resolver returned drifted scope {}", self.returned_scope),
            })
        }
    }

    #[tokio::test]
    async fn protected_router_requires_an_authorization_header() {
        let response = build_protected_router_with_state(AppState::new(
            ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>()),
        ))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn public_router_keeps_commit_mutation_out_of_the_public_catalog() {
        let response = build_router_with_state(AppState::new(ProviderRegistry::from_env(
            std::iter::empty::<(&str, &str)>(),
        )))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits")
                .body(Body::from("{}"))
                .expect("request"),
        )
        .await
        .expect("response");

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn protected_router_rejects_an_invalid_bearer_token() {
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 30,
        }));
        let state = AppState::new(ProviderRegistry::from_env(
            std::iter::empty::<(&str, &str)>(),
        ))
        .with_protected_write_dependencies(
            contextlab_auth::DenyAllContextAuthorizer,
            contextlab_storage::InMemoryContextGraphRepository::context_engineering_preview(),
            contextlab_auth::HmacJwtAuthenticator::new(
                "test-secret",
                TEST_AUTH_ISSUER,
                TEST_AUTH_AUDIENCE,
            )
            .expect("authenticator"),
            limiter.clone(),
        );
        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits")
                    .header("authorization", "Bearer invalid")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(limiter.calls(), 0);
    }

    #[tokio::test]
    async fn protected_router_denies_an_authenticated_principal_without_write_access() {
        let state = AppState::new(ProviderRegistry::from_env(
            std::iter::empty::<(&str, &str)>(),
        ))
        .with_protected_write_dependencies(
            contextlab_auth::DenyAllContextAuthorizer,
            InMemoryContextGraphRepository::context_engineering_preview(),
            contextlab_auth::HmacJwtAuthenticator::new(
                "test-secret",
                TEST_AUTH_ISSUER,
                TEST_AUTH_AUDIENCE,
            )
            .expect("authenticator"),
            test_rate_limiter(),
        );
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestJwtClaims {
                sub: "user:alex".to_owned(),
                exp: (Utc::now().timestamp() + 300) as usize,
                iss: TEST_AUTH_ISSUER.to_owned(),
                aud: TEST_AUTH_AUDIENCE.to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");
        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn protected_router_reports_authorization_unavailability_as_service_unavailable() {
        let state = AppState::new(ProviderRegistry::from_env(
            std::iter::empty::<(&str, &str)>(),
        ))
        .with_protected_write_dependencies(
            UnavailableContextWrites,
            InMemoryContextGraphRepository::context_engineering_preview(),
            contextlab_auth::HmacJwtAuthenticator::new(
                "test-secret",
                TEST_AUTH_ISSUER,
                TEST_AUTH_AUDIENCE,
            )
            .expect("authenticator"),
            test_rate_limiter(),
        );
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestJwtClaims {
                sub: "user:alex".to_owned(),
                exp: (Utc::now().timestamp() + 300) as usize,
                iss: TEST_AUTH_ISSUER.to_owned(),
                aud: TEST_AUTH_AUDIENCE.to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");
        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits")
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("request"),
            )
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "authorization_unavailable");
    }

    #[tokio::test]
    async fn protected_router_records_a_granted_authorization_decision() {
        let (state, context_id, token) = protected_test_state();
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state.with_authorization_audit_sink(RecordingAuditSink {
            events: events.clone(),
        });
        let response = build_protected_router_with_state(state)
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-audit-granted"),
                Body::from(protected_commit_body().to_string()),
            ))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::CREATED);
        let events = events.lock().expect("audit events lock");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].context_id().to_string(), context_id);
        assert_eq!(events[0].decision(), AuthorizationDecision::Granted);
    }

    #[tokio::test]
    async fn protected_router_fails_closed_when_authorization_audit_is_unavailable() {
        let (state, context_id, token) = protected_test_state();
        let state = state.with_authorization_audit_sink(FailingAuditSink);
        let response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-audit-failure"),
                Body::from(protected_commit_body().to_string()),
            ))
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "authorization_audit_unavailable");

        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn protected_router_creates_and_replays_a_guarded_commit() {
        let context_id = "11111111-1111-4111-8111-111111111111";
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: "22222222-2222-4222-8222-222222222222".to_owned(),
                workspace_id: "33333333-3333-4333-8333-333333333333".to_owned(),
                name: "Protected Project".to_owned(),
                slug: "protected-project".to_owned(),
                created_at: Utc::now(),
            }],
            contexts: vec![ContextRecord {
                id: context_id.to_owned(),
                project_id: "22222222-2222-4222-8222-222222222222".to_owned(),
                experiment_id: None,
                name: "Protected Context".to_owned(),
                description: None,
                created_at: Utc::now(),
            }],
            ..ContextGraphProjection::default()
        });
        let state = AppState::with_workspace_repositories(
            ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>()),
            WorkspaceRepositories::new(
                WorkspaceGraphRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                ),
                WorkspaceCatalogRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                ),
            ),
        )
        .with_protected_write_dependencies(
            AllowContextWrites,
            repository,
            contextlab_auth::HmacJwtAuthenticator::new(
                "test-secret",
                TEST_AUTH_ISSUER,
                TEST_AUTH_AUDIENCE,
            )
            .expect("authenticator"),
            test_rate_limiter(),
        );
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestJwtClaims {
                sub: "user:alex".to_owned(),
                exp: (Utc::now().timestamp() + 300) as usize,
                iss: TEST_AUTH_ISSUER.to_owned(),
                aud: TEST_AUTH_AUDIENCE.to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");
        let request_body = serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": null,
            "message": "Initial protected commit",
            "changes": [{
                "kind": "created_context",
                "component_id": null,
                "component_kind": null,
                "summary": "Initial protected context"
            }],
            "snapshot": {"nodes": [], "edges": []},
            "schema_version": 1
        });
        let first = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/contexts/{context_id}/commits"))
                    .header("authorization", format!("Bearer {token}"))
                    .header("idempotency-key", "request-001")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");
        let first_status = first.status();
        let first_body = first.into_body().collect().await.expect("body").to_bytes();

        assert_eq!(
            first_status,
            StatusCode::CREATED,
            "payload: {}",
            String::from_utf8_lossy(&first_body)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&first_body).expect("json")["disposition"],
            "created"
        );

        let replay = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/contexts/{context_id}/commits"))
                    .header("authorization", format!("Bearer {token}"))
                    .header("idempotency-key", "request-001")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(replay.status(), StatusCode::OK);

        let stale = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/contexts/{context_id}/commits"))
                    .header("authorization", format!("Bearer {token}"))
                    .header("idempotency-key", "request-stale")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");
        let stale_status = stale.status();
        let stale_body = stale.into_body().collect().await.expect("body").to_bytes();

        assert_eq!(stale_status, StatusCode::CONFLICT);
        assert_eq!(
            serde_json::from_slice::<Value>(&stale_body).expect("json")["error"],
            "storage_branch_head_conflict"
        );
    }

    #[tokio::test]
    async fn protected_router_rejects_a_missing_idempotency_key() {
        let (state, context_id, token) = protected_test_state();
        let response = build_protected_router_with_state(state)
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                None,
                Body::from(protected_commit_body().to_string()),
            ))
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_commit_request");
    }

    #[tokio::test]
    async fn protected_router_rejects_invalid_json_before_writing() {
        let (state, context_id, token) = protected_test_state();
        let response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-invalid-json"),
                Body::from("{"),
            ))
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_commit_request");

        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn protected_router_rejects_an_invalid_branch_name() {
        let (state, context_id, token) = protected_test_state();
        let mut body = protected_commit_body();
        body["branch_name"] = Value::String("feature with spaces".to_owned());
        let response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-invalid-branch"),
                Body::from(body.to_string()),
            ))
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_commit_request");

        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn protected_router_rejects_an_invalid_expected_head() {
        let (state, context_id, token) = protected_test_state();
        let mut body = protected_commit_body();
        body["expected_head_commit_id"] = Value::String("not-a-uuid".to_owned());
        let response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-invalid-head"),
                Body::from(body.to_string()),
            ))
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_commit_request");

        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn protected_router_rejects_an_invalid_snapshot_schema_version() {
        let (state, context_id, token) = protected_test_state();
        let mut body = protected_commit_body();
        body["schema_version"] = Value::from(0);
        let response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-invalid-schema"),
                Body::from(body.to_string()),
            ))
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_commit_request");

        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn protected_router_rejects_an_idempotency_key_reused_for_a_different_body() {
        let (state, context_id, token) = protected_test_state();
        let first_response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-digest-mismatch"),
                Body::from(protected_commit_body().to_string()),
            ))
            .await
            .expect("first response");
        assert_eq!(first_response.status(), StatusCode::CREATED);

        let mut changed = protected_commit_body();
        changed["message"] = Value::String("Different request body".to_owned());
        let replay = build_protected_router_with_state(state)
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-digest-mismatch"),
                Body::from(changed.to_string()),
            ))
            .await
            .expect("replay response");

        let (status, payload) = response_json(replay).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "storage_idempotency_conflict");
    }

    #[tokio::test]
    async fn protected_local_branch_heads_read_uses_exact_scope_and_private_headers() {
        let (state, context_id, token) =
            protected_test_state_with_authorizer(ReadOnlyContextAccess);
        let path = format!("/api/v1/local/contexts/{context_id}/branches");
        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("branch-head request"),
            )
            .await
            .expect("branch-head response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        let (_, payload) = response_json(response).await;
        assert_eq!(
            payload["schema_version"],
            "contextlab.local-context-branch-heads.v1"
        );
        assert_eq!(payload["context_id"], context_id);
        assert!(payload["branches"].is_array());
    }

    #[tokio::test]
    async fn protected_local_branch_heads_read_requires_auth_and_is_absent_from_public_router() {
        let (state, context_id, _) = protected_test_state_with_authorizer(ReadOnlyContextAccess);
        let path = format!("/api/v1/local/contexts/{context_id}/branches");
        let missing_auth = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("missing auth branch-head request"),
            )
            .await
            .expect("missing auth response");
        assert_eq!(missing_auth.status(), StatusCode::UNAUTHORIZED);

        let public = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public branch-head request"),
            )
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);
    }

    fn protected_test_state() -> (AppState, String, String) {
        protected_test_state_with_authorizer(AllowContextWrites)
    }

    fn protected_test_state_with_authorizer(
        context_authorizer: impl ContextAuthorizer + 'static,
    ) -> (AppState, String, String) {
        protected_test_state_with_repository(protected_test_repository(), context_authorizer)
    }

    fn protected_test_repository() -> InMemoryContextGraphRepository {
        let context_id = "11111111-1111-4111-8111-111111111111".to_owned();
        InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: "22222222-2222-4222-8222-222222222222".to_owned(),
                workspace_id: "33333333-3333-4333-8333-333333333333".to_owned(),
                name: "Protected Project".to_owned(),
                slug: "protected-project".to_owned(),
                created_at: Utc::now(),
            }],
            contexts: vec![ContextRecord {
                id: context_id.clone(),
                project_id: "22222222-2222-4222-8222-222222222222".to_owned(),
                experiment_id: None,
                name: "Protected Context".to_owned(),
                description: None,
                created_at: Utc::now(),
            }],
            ..ContextGraphProjection::default()
        })
    }

    fn protected_test_state_with_repository(
        repository: InMemoryContextGraphRepository,
        context_authorizer: impl ContextAuthorizer + 'static,
    ) -> (AppState, String, String) {
        let context_id = "11111111-1111-4111-8111-111111111111".to_owned();
        let state = AppState::with_workspace_repositories(
            ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>()),
            WorkspaceRepositories::new(
                WorkspaceGraphRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                ),
                WorkspaceCatalogRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                ),
            ),
        )
        .with_protected_write_dependencies(
            context_authorizer,
            repository.clone(),
            contextlab_auth::HmacJwtAuthenticator::new(
                "test-secret",
                TEST_AUTH_ISSUER,
                TEST_AUTH_AUDIENCE,
            )
            .expect("authenticator"),
            test_rate_limiter(),
        )
        .with_context_lifecycle_repository(repository.clone())
        .with_benchmark_definition_binding_writer(repository.clone())
        .with_benchmark_definition_binding_repository(repository);
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestJwtClaims {
                sub: "user:alex".to_owned(),
                exp: (Utc::now().timestamp() + 300) as usize,
                iss: TEST_AUTH_ISSUER.to_owned(),
                aud: TEST_AUTH_AUDIENCE.to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");

        (state, context_id, token)
    }

    fn protected_commit_body() -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": null,
            "message": "Protected commit",
            "changes": [{
                "kind": "created_context",
                "component_id": null,
                "component_kind": null,
                "summary": "Protected context"
            }],
            "snapshot": {"nodes": [], "edges": []},
            "schema_version": 1
        })
    }

    fn protected_commit_request(
        context_id: &str,
        token: &str,
        idempotency_key: Option<&str>,
        body: Body,
    ) -> Request<Body> {
        let mut builder = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/contexts/{context_id}/commits"))
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json");
        if let Some(idempotency_key) = idempotency_key {
            builder = builder.header("idempotency-key", idempotency_key);
        }
        builder.body(body).expect("request")
    }

    fn local_benchmark_definition_authoring_body(expected_head_commit_id: &str) -> Value {
        serde_json::json!({
            "schema_version": 1,
            "binding_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaa1",
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "datasets": [{
                "id": "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbb1",
                "name": "Core authoring dataset",
                "cases": [{
                    "id": "cccccccc-cccc-4ccc-8ccc-ccccccccccc1",
                    "name": "Exact structured answer",
                    "input": {"question": "What is Context Engineering?"},
                    "expected_output": {"mode": "exact", "value": {"answer": "Context first"}}
                }]
            }],
            "suite": {
                "id": "dddddddd-dddd-4ddd-8ddd-ddddddddddd1",
                "name": "Core authoring suite",
                "dataset_ids": ["bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbb1"],
                "thresholds": [{
                    "metric": "accuracy",
                    "direction": "minimum",
                    "value": 0.9
                }]
            }
        })
    }

    fn local_benchmark_definition_authoring_request(
        project_id: &str,
        context_id: &str,
        commit_id: &str,
        token: Option<&str>,
        idempotency_key: Option<&str>,
        body: Value,
    ) -> Request<Body> {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings"
            ))
            .header("content-type", "application/json");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        if let Some(idempotency_key) = idempotency_key {
            request = request.header("idempotency-key", idempotency_key);
        }
        request
            .body(Body::from(body.to_string()))
            .expect("local benchmark definition authoring request")
    }

    fn local_benchmark_execution_request(
        project_id: &str,
        context_id: &str,
        commit_id: &str,
        token: Option<&str>,
        idempotency_key: Option<&str>,
        body: Value,
    ) -> Request<Body> {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-executions"
            ))
            .header("content-type", "application/json");
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        if let Some(idempotency_key) = idempotency_key {
            request = request.header("idempotency-key", idempotency_key);
        }
        request
            .body(Body::from(body.to_string()))
            .expect("local benchmark execution request")
    }

    #[tokio::test]
    async fn local_benchmark_execution_is_private_and_fails_closed_without_an_evaluator() {
        let (state, context_id, token) = protected_test_state();
        let project_id = "22222222-2222-4222-8222-222222222222";
        let commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let authoring = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-authoring"),
                local_benchmark_definition_authoring_body(&commit_id),
            ))
            .await
            .expect("authoring response");
        assert_eq!(authoring.status(), StatusCode::CREATED);

        let body = serde_json::json!({
            "schema_version": 1,
            "binding_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaa1",
            "decision_id": "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeee1",
            "model_version": "local-test-model",
            "temperature": 0.0,
            "evaluator_key": "injected-local",
            "evaluator_version": "v1"
        });
        let public = build_router_with_state(state.clone())
            .oneshot(local_benchmark_execution_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-public"),
                body.clone(),
            ))
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let missing_auth = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_execution_request(
                project_id,
                &context_id,
                &commit_id,
                None,
                Some("benchmark-execution-missing-auth"),
                body.clone(),
            ))
            .await
            .expect("missing-auth response");
        assert_eq!(missing_auth.status(), StatusCode::UNAUTHORIZED);

        let unavailable = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_execution_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-unavailable"),
                body,
            ))
            .await
            .expect("unavailable response");
        let (status, payload) = response_json(unavailable).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "benchmark_execution_unavailable");

        let openapi: Value = serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("checked-in OpenAPI contract must parse");
        assert!(openapi["paths"]["/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-executions"].is_null());
    }

    #[tokio::test]
    async fn local_benchmark_execution_authorizes_write_before_parsing_body() {
        let (state, context_id, token) =
            protected_test_state_with_authorizer(ReadOnlyContextAccess);
        let response = build_protected_router_with_state(state)
            .oneshot(local_benchmark_execution_request(
                "22222222-2222-4222-8222-222222222222",
                &context_id,
                "33333333-3333-4333-8333-333333333333",
                Some(&token),
                Some("benchmark-execution-auth-order"),
                serde_json::json!({"unknown": true}),
            ))
            .await
            .expect("forbidden response");
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_write_forbidden");
    }

    #[tokio::test]
    async fn local_benchmark_execution_replays_and_rejects_changed_idempotency_payloads() {
        let repository = protected_test_repository();
        let (state, context_id, token) =
            protected_test_state_with_repository(repository.clone(), AllowContextWrites);
        let calls = Arc::new(AtomicUsize::new(0));
        let evaluator = RecordingBenchmarkEvaluator {
            calls: calls.clone(),
        };
        let state = state
            .with_benchmark_evidence_repository(repository.clone())
            .with_benchmark_workspace_projection_repository(repository.clone())
            .with_benchmark_execution_adapter(
                crate::benchmark_execution::StorageBenchmarkExecutionAdapter::new(
                    repository, evaluator,
                ),
            );
        let project_id = "22222222-2222-4222-8222-222222222222";
        let commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let authoring = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-binding"),
                local_benchmark_definition_authoring_body(&commit_id),
            ))
            .await
            .expect("authoring response");
        assert_eq!(authoring.status(), StatusCode::CREATED);

        let body = serde_json::json!({
            "schema_version": 1,
            "binding_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaa1",
            "decision_id": "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeee1",
            "model_version": "local-test-model",
            "temperature": 0.0,
            "evaluator_key": "injected-local",
            "evaluator_version": "v1"
        });
        let wrong_scope = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_execution_request(
                "33333333-3333-4333-8333-333333333333",
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-wrong-scope"),
                body.clone(),
            ))
            .await
            .expect("wrong-scope response");
        assert_eq!(wrong_scope.status(), StatusCode::NOT_FOUND);
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        let created = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_execution_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-replay"),
                body.clone(),
            ))
            .await
            .expect("created response");
        let (created_status, created_payload) = response_json(created).await;
        assert_eq!(created_status, StatusCode::CREATED);
        assert_eq!(created_payload["disposition"], "created");
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let replayed = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_execution_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-replay"),
                body,
            ))
            .await
            .expect("replayed response");
        let (replayed_status, replayed_payload) = response_json(replayed).await;
        assert_eq!(replayed_status, StatusCode::OK);
        assert_eq!(replayed_payload["disposition"], "replayed");
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let changed = serde_json::json!({
            "schema_version": 1,
            "binding_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaa1",
            "decision_id": "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeee1",
            "model_version": "different-model",
            "temperature": 0.0,
            "evaluator_key": "injected-local",
            "evaluator_version": "v1"
        });
        let conflict = build_protected_router_with_state(state)
            .oneshot(local_benchmark_execution_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-execution-replay"),
                changed,
            ))
            .await
            .expect("conflict response");
        let (conflict_status, conflict_payload) = response_json(conflict).await;
        assert_eq!(conflict_status, StatusCode::CONFLICT);
        assert_eq!(conflict_payload["error"], "benchmark_execution_conflict");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn local_benchmark_definition_authoring_is_private_and_requires_bearer_auth() {
        let (state, context_id, token) = protected_test_state();
        let project_id = "22222222-2222-4222-8222-222222222222";
        let commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let body = local_benchmark_definition_authoring_body(&commit_id);

        let public = build_router_with_state(state.clone())
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-authoring-private"),
                body.clone(),
            ))
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);
        let openapi: Value = serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("checked-in OpenAPI contract must parse");
        assert!(
            openapi["paths"]["/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings"]
                .is_null(),
            "private benchmark definition authoring must not be published in OpenAPI"
        );

        let protected = build_protected_router_with_state(state)
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &commit_id,
                None,
                Some("benchmark-authoring-private"),
                body,
            ))
            .await
            .expect("protected response");
        assert_eq!(protected.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn local_benchmark_definition_authoring_creates_and_replays_exact_scope() {
        let (state, context_id, token) = protected_test_state();
        let project_id = "22222222-2222-4222-8222-222222222222";
        let commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let body = local_benchmark_definition_authoring_body(&commit_id);

        let created = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-authoring-replay"),
                body.clone(),
            ))
            .await
            .expect("created response");
        assert_eq!(
            created
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        let (created_status, created_payload) = response_json(created).await;
        assert_eq!(
            created_status,
            StatusCode::CREATED,
            "payload: {created_payload}"
        );
        assert_eq!(
            created_payload["schema_version"],
            "contextlab.local-benchmark-definition-authoring.v1"
        );
        assert_eq!(created_payload["disposition"], "created");
        assert_eq!(created_payload["project_id"], project_id);
        assert_eq!(created_payload["context_id"], context_id);
        assert_eq!(created_payload["commit_id"], commit_id);
        assert_eq!(
            created_payload["message"]["en"],
            "Benchmark definitions authored."
        );
        assert_eq!(created_payload["message"]["zh"], "Benchmark 定义已创建。");
        assert_json_excludes_keys_recursively(
            &created_payload,
            &["cases", "input", "expected_output", "value", "raw"],
        );

        let replayed = build_protected_router_with_state(state)
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &commit_id,
                Some(&token),
                Some("benchmark-authoring-replay"),
                body,
            ))
            .await
            .expect("replayed response");
        let (replayed_status, replayed_payload) = response_json(replayed).await;
        assert_eq!(replayed_status, StatusCode::OK);
        assert_eq!(replayed_payload["disposition"], "replayed");
        assert_eq!(
            replayed_payload["binding_id"],
            created_payload["binding_id"]
        );
    }

    #[tokio::test]
    async fn local_benchmark_definition_authoring_fails_closed_on_schema_and_raw_fields() {
        let (state, context_id, token) = protected_test_state();
        let project_id = "22222222-2222-4222-8222-222222222222";
        let commit_id = create_materialized_protected_root(&state, &context_id, &token).await;

        for (key, mut body) in [
            (
                "schema",
                local_benchmark_definition_authoring_body(&commit_id),
            ),
            ("raw", local_benchmark_definition_authoring_body(&commit_id)),
        ] {
            if key == "schema" {
                body["schema_version"] = Value::from(2);
            } else {
                body["datasets"][0]["cases"][0]["raw_input"] =
                    Value::String("forbidden".to_owned());
            }
            let response = build_protected_router_with_state(state.clone())
                .oneshot(local_benchmark_definition_authoring_request(
                    project_id,
                    &context_id,
                    &commit_id,
                    Some(&token),
                    Some(&format!("benchmark-authoring-invalid-{key}")),
                    body,
                ))
                .await
                .expect("invalid response");
            let (status, payload) = response_json(response).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(
                payload["error"],
                "invalid_benchmark_definition_authoring_request"
            );
        }
    }

    #[tokio::test]
    async fn local_benchmark_definition_authoring_authorizes_write_before_body_parsing() {
        let (state, context_id, token) =
            protected_test_state_with_authorizer(ReadOnlyContextAccess);
        let project_id = "22222222-2222-4222-8222-222222222222";
        let response = build_protected_router_with_state(state)
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                "11111111-1111-4111-8111-111111111111",
                Some(&token),
                Some("benchmark-authoring-denied"),
                serde_json::json!({"raw": true}),
            ))
            .await
            .expect("forbidden response");
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_write_forbidden");
    }

    #[tokio::test]
    async fn local_benchmark_definition_authoring_rejects_digest_and_branch_head_conflicts() {
        let (state, context_id, token) = protected_test_state();
        let project_id = "22222222-2222-4222-8222-222222222222";
        let root_commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let body = local_benchmark_definition_authoring_body(&root_commit_id);
        let first = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &root_commit_id,
                Some(&token),
                Some("benchmark-authoring-conflict"),
                body.clone(),
            ))
            .await
            .expect("first response");
        assert_eq!(first.status(), StatusCode::CREATED);

        let mut changed = body.clone();
        changed["suite"]["name"] = Value::String("Changed suite".to_owned());
        let digest_conflict = build_protected_router_with_state(state.clone())
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &root_commit_id,
                Some(&token),
                Some("benchmark-authoring-conflict"),
                changed,
            ))
            .await
            .expect("digest conflict response");
        let (status, payload) = response_json(digest_conflict).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "storage_idempotency_conflict");

        let mut child_body = protected_commit_body();
        child_body["expected_head_commit_id"] = Value::String(root_commit_id.clone());
        let child = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("benchmark-authoring-advance-head"),
                Body::from(child_body.to_string()),
            ))
            .await
            .expect("advance head response");
        assert_eq!(child.status(), StatusCode::CREATED);

        let stale = build_protected_router_with_state(state)
            .oneshot(local_benchmark_definition_authoring_request(
                project_id,
                &context_id,
                &root_commit_id,
                Some(&token),
                Some("benchmark-authoring-stale"),
                body,
            ))
            .await
            .expect("stale response");
        let (status, payload) = response_json(stale).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "storage_branch_head_conflict");
    }

    #[tokio::test]
    async fn local_benchmark_definition_authoring_uses_guarded_write_rate_limit_operation() {
        let (state, context_id, token) = protected_test_state();
        let project_id = "22222222-2222-4222-8222-222222222222";
        let commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Allowed));
        let response =
            build_protected_router_with_state(state.with_protected_rate_limiter(limiter.clone()))
                .oneshot(local_benchmark_definition_authoring_request(
                    project_id,
                    &context_id,
                    &commit_id,
                    Some(&token),
                    Some("benchmark-authoring-rate-limit"),
                    local_benchmark_definition_authoring_body(&commit_id),
                ))
                .await
                .expect("rate-limited response");
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::BenchmarkDefinitionAuthoringWrite]
        );
    }

    #[tokio::test]
    async fn private_benchmark_definition_authoring_creates_an_exact_commit_binding() {
        let (state, context_id, token) = protected_test_state();
        let commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/v1/local/projects/22222222-2222-4222-8222-222222222222/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings"
                    ))
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .header("idempotency-key", "benchmark-authoring-red")
                    .body(Body::from(
                        serde_json::json!({
                            "schema_version": 1,
                            "binding_id": "33333333-3333-4333-8333-333333333333",
                            "branch_name": "main",
                            "expected_head_commit_id": commit_id,
                            "datasets": [{
                                "id": "44444444-4444-4444-8444-444444444444",
                                "name": "smoke",
                                "cases": [{
                                    "id": "55555555-5555-4555-8555-555555555555",
                                    "name": "case",
                                    "input": {"value": "hello"},
                                    "expected_output": {"mode": "unspecified"}
                                }]
                            }],
                            "suite": {
                                "id": "66666666-6666-4666-8666-666666666666",
                                "name": "smoke suite",
                                "dataset_ids": ["44444444-4444-4444-8444-444444444444"],
                                "thresholds": [{
                                    "metric": "accuracy",
                                    "direction": "minimum",
                                    "value": 0.5
                                }]
                            }
                        })
                        .to_string(),
                    ))
                    .expect("benchmark definition request"),
            )
            .await
            .expect("benchmark definition response");

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn protected_local_lifecycle_creates_replays_updates_removes_and_reads_historical_state()
    {
        let (state, context_id, token) = protected_test_state();
        let root_commit_id = create_materialized_protected_root(&state, &context_id, &token).await;

        let create_body = local_lifecycle_create_body(&root_commit_id);
        let created = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-create-001",
                create_body.clone(),
            ))
            .await
            .expect("created lifecycle response");
        let (created_status, created_payload) = response_json(created).await;
        assert_eq!(created_status, StatusCode::CREATED);
        assert_eq!(created_payload["disposition"], "created");
        let create_commit_id = created_payload["commit_id"]
            .as_str()
            .expect("created commit id")
            .to_owned();

        let replayed = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-create-001",
                create_body,
            ))
            .await
            .expect("replayed lifecycle response");
        let (replayed_status, replayed_payload) = response_json(replayed).await;
        assert_eq!(replayed_status, StatusCode::OK);
        assert_eq!(replayed_payload["disposition"], "replayed");
        assert_eq!(replayed_payload["commit_id"], create_commit_id);

        let created_state = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &create_commit_id,
                &token,
            ))
            .await
            .expect("created lifecycle state response");
        let (created_state_status, created_state_payload) = response_json(created_state).await;
        assert_eq!(created_state_status, StatusCode::OK);
        assert_eq!(
            created_state_payload["components"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            created_state_payload["components"][0]["content"],
            "You are a precise Context engineer."
        );
        let component_id = created_state_payload["components"][0]["component_id"]
            .as_str()
            .expect("created component id")
            .to_owned();

        let updated = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-update-001",
                local_lifecycle_update_body(&create_commit_id, &component_id),
            ))
            .await
            .expect("updated lifecycle response");
        let (updated_status, updated_payload) = response_json(updated).await;
        assert_eq!(updated_status, StatusCode::CREATED);
        let update_commit_id = updated_payload["commit_id"]
            .as_str()
            .expect("updated commit id")
            .to_owned();

        let removed = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-remove-001",
                local_lifecycle_remove_body(&update_commit_id, &component_id),
            ))
            .await
            .expect("removed lifecycle response");
        let (removed_status, removed_payload) = response_json(removed).await;
        assert_eq!(removed_status, StatusCode::CREATED);
        let removal_commit_id = removed_payload["commit_id"]
            .as_str()
            .expect("removal commit id")
            .to_owned();

        let historical = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &update_commit_id,
                &token,
            ))
            .await
            .expect("historical lifecycle state response");
        let (historical_status, historical_payload) = response_json(historical).await;
        assert_eq!(historical_status, StatusCode::OK);
        assert_eq!(
            historical_payload["components"][0]["content"],
            "Use the Context Graph before responding."
        );

        let removed_state = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &removal_commit_id,
                &token,
            ))
            .await
            .expect("removed lifecycle state response");
        let (removed_state_status, removed_state_payload) = response_json(removed_state).await;
        assert_eq!(removed_state_status, StatusCode::OK);
        assert_eq!(
            removed_state_payload["components"].as_array().map(Vec::len),
            Some(0)
        );

        let stale = build_protected_router_with_state(state)
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-stale-001",
                local_lifecycle_create_body(&root_commit_id),
            ))
            .await
            .expect("stale lifecycle response");
        let (stale_status, stale_payload) = response_json(stale).await;
        assert_eq!(stale_status, StatusCode::CONFLICT);
        assert_eq!(stale_payload["error"], "storage_branch_head_conflict");
    }

    #[tokio::test]
    async fn protected_local_lifecycle_updates_and_replays_context_metadata() {
        let (state, context_id, token) = protected_test_state();
        let root_commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let body = local_lifecycle_update_metadata_body(&root_commit_id);

        let created = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-metadata-001",
                body.clone(),
            ))
            .await
            .expect("metadata lifecycle response");
        let (created_status, created_payload) = response_json(created).await;
        assert_eq!(created_status, StatusCode::CREATED);
        assert_eq!(created_payload["disposition"], "created");
        let metadata_commit_id = created_payload["commit_id"]
            .as_str()
            .expect("metadata commit id")
            .to_owned();

        let replayed = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-metadata-001",
                body,
            ))
            .await
            .expect("metadata replay response");
        let (replayed_status, replayed_payload) = response_json(replayed).await;
        assert_eq!(replayed_status, StatusCode::OK);
        assert_eq!(replayed_payload["disposition"], "replayed");
        assert_eq!(replayed_payload["commit_id"], metadata_commit_id);

        let state_response = build_protected_router_with_state(state)
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &metadata_commit_id,
                &token,
            ))
            .await
            .expect("metadata lifecycle state response");
        let (state_status, state_payload) = response_json(state_response).await;
        assert_eq!(state_status, StatusCode::OK);
        assert_eq!(
            state_payload["metadata"],
            serde_json::json!({
                "created_at": "2026-07-30T00:00:00Z",
                "updated_at": "2026-07-30T01:00:00Z",
                "labels": {"owner": "luna"}
            })
        );
    }

    #[tokio::test]
    async fn protected_local_lifecycle_adds_replays_and_removes_a_uses_relationship() {
        let (state, context_id, token) = protected_test_state();
        let root_commit_id = create_materialized_protected_root(&state, &context_id, &token).await;

        let source_created = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-uses-source-create-001",
                local_lifecycle_create_body(&root_commit_id),
            ))
            .await
            .expect("source component response");
        let (source_status, source_payload) = response_json(source_created).await;
        assert_eq!(source_status, StatusCode::CREATED);
        let source_commit_id = source_payload["commit_id"]
            .as_str()
            .expect("source component commit id")
            .to_owned();

        let source_state = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &source_commit_id,
                &token,
            ))
            .await
            .expect("source component state response");
        let (source_state_status, source_state_payload) = response_json(source_state).await;
        assert_eq!(source_state_status, StatusCode::OK);
        let source_component_id = source_state_payload["components"][0]["component_id"]
            .as_str()
            .expect("source component id")
            .to_owned();

        let target_created = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-uses-target-create-001",
                local_lifecycle_create_body(&source_commit_id),
            ))
            .await
            .expect("target component response");
        let (target_status, target_payload) = response_json(target_created).await;
        assert_eq!(target_status, StatusCode::CREATED);
        let target_commit_id = target_payload["commit_id"]
            .as_str()
            .expect("target component commit id")
            .to_owned();

        let target_state = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &target_commit_id,
                &token,
            ))
            .await
            .expect("target component state response");
        let (target_state_status, target_state_payload) = response_json(target_state).await;
        assert_eq!(target_state_status, StatusCode::OK);
        let target_component_id = target_state_payload["components"]
            .as_array()
            .expect("target component list")
            .iter()
            .find_map(|component| {
                let component_id = component["component_id"].as_str()?;
                (component_id != source_component_id).then(|| component_id.to_owned())
            })
            .expect("target component id");

        let add_body = local_lifecycle_add_uses_relationship_body(
            &target_commit_id,
            &source_component_id,
            &target_component_id,
        );
        let added = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-uses-add-001",
                add_body.clone(),
            ))
            .await
            .expect("added relationship response");
        let (added_status, added_payload) = response_json(added).await;
        assert_eq!(added_status, StatusCode::CREATED);
        assert_eq!(added_payload["disposition"], "created");
        assert_eq!(
            added_payload["snapshot"]["graph"]["nodes"]
                .as_object()
                .map(|nodes| nodes.len()),
            Some(3)
        );
        let added_edges = added_payload["snapshot"]["graph"]["edges"]
            .as_array()
            .expect("added relationship edges");
        assert_eq!(added_edges.len(), 3);
        assert!(added_edges.iter().any(|edge| {
            edge == &serde_json::json!({
                "source": format!("component:{source_component_id}"),
                "target": format!("component:{target_component_id}"),
                "kind": "uses"
            })
        }));
        let added_commit_id = added_payload["commit_id"]
            .as_str()
            .expect("added relationship commit id")
            .to_owned();

        let added_commit = state
            .commit_repository()
            .get_commit(context_id.clone(), added_commit_id.clone())
            .await
            .expect("added relationship commit");
        let added_change =
            serde_json::to_value(added_commit).expect("serialized relationship commit");
        assert_eq!(
            added_change["changes"][0]["kind"],
            "added_uses_relationship"
        );
        assert_eq!(
            added_change["changes"][0]["source_component_id"],
            source_component_id
        );
        assert_eq!(
            added_change["changes"][0]["target_component_id"],
            target_component_id
        );

        let replayed = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-uses-add-001",
                add_body,
            ))
            .await
            .expect("replayed relationship response");
        let (replayed_status, replayed_payload) = response_json(replayed).await;
        assert_eq!(replayed_status, StatusCode::OK);
        assert_eq!(replayed_payload["disposition"], "replayed");
        assert_eq!(replayed_payload["commit_id"], added_commit_id);
        assert_eq!(replayed_payload["snapshot"], added_payload["snapshot"]);

        let removed = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-uses-remove-001",
                local_lifecycle_remove_uses_relationship_body(
                    &added_commit_id,
                    &source_component_id,
                    &target_component_id,
                ),
            ))
            .await
            .expect("removed relationship response");
        let (removed_status, removed_payload) = response_json(removed).await;
        assert_eq!(removed_status, StatusCode::CREATED);
        assert_eq!(removed_payload["disposition"], "created");
        let removed_edges = removed_payload["snapshot"]["graph"]["edges"]
            .as_array()
            .expect("removed relationship edges");
        assert_eq!(removed_edges.len(), 2);
        assert!(removed_edges.iter().all(|edge| edge["kind"] != "uses"));
        let removed_commit_id = removed_payload["commit_id"]
            .as_str()
            .expect("removed relationship commit id")
            .to_owned();

        let removed_commit = state
            .commit_repository()
            .get_commit(context_id, removed_commit_id)
            .await
            .expect("removed relationship commit");
        let removed_change =
            serde_json::to_value(removed_commit).expect("serialized removed relationship commit");
        assert_eq!(
            removed_change["changes"][0]["kind"],
            "removed_uses_relationship"
        );
        assert_eq!(
            removed_change["changes"][0]["source_component_id"],
            source_component_id
        );
        assert_eq!(
            removed_change["changes"][0]["target_component_id"],
            target_component_id
        );
    }

    #[tokio::test]
    async fn protected_local_context_lifecycle_updates_a_component_descriptor_without_rewriting_its_body()
     {
        let (state, context_id, token) = protected_test_state();
        let root_commit_id = create_materialized_protected_root(&state, &context_id, &token).await;

        let created = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-descriptor-create-001",
                local_lifecycle_create_body(&root_commit_id),
            ))
            .await
            .expect("created lifecycle response");
        let (created_status, created_payload) = response_json(created).await;
        assert_eq!(created_status, StatusCode::CREATED);
        let create_commit_id = created_payload["commit_id"]
            .as_str()
            .expect("created commit id")
            .to_owned();

        let created_state = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &create_commit_id,
                &token,
            ))
            .await
            .expect("created lifecycle state response");
        let (created_state_status, created_state_payload) = response_json(created_state).await;
        assert_eq!(created_state_status, StatusCode::OK);
        let component = &created_state_payload["components"][0];
        let component_id = component["component_id"]
            .as_str()
            .expect("created component id")
            .to_owned();
        let content = component["content"].clone();
        let content_hash = component["content_hash"].clone();
        let content_commit_id = component["content_commit_id"].clone();
        let descriptor_body =
            local_lifecycle_update_descriptor_body(&create_commit_id, &component_id);

        let descriptor_updated = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-update-descriptor-001",
                descriptor_body.clone(),
            ))
            .await
            .expect("descriptor update response");
        let (descriptor_status, descriptor_payload) = response_json(descriptor_updated).await;
        assert_eq!(descriptor_status, StatusCode::CREATED);
        assert_eq!(descriptor_payload["disposition"], "created");
        let descriptor_commit_id = descriptor_payload["commit_id"]
            .as_str()
            .expect("descriptor commit id")
            .to_owned();

        let replayed = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-update-descriptor-001",
                descriptor_body,
            ))
            .await
            .expect("descriptor replay response");
        let (replayed_status, replayed_payload) = response_json(replayed).await;
        assert_eq!(replayed_status, StatusCode::OK);
        assert_eq!(replayed_payload["disposition"], "replayed");
        assert_eq!(replayed_payload["commit_id"], descriptor_commit_id);

        let descriptor_state = build_protected_router_with_state(state)
            .oneshot(local_lifecycle_state_request(
                &context_id,
                &descriptor_commit_id,
                &token,
            ))
            .await
            .expect("descriptor lifecycle state response");
        let (descriptor_state_status, descriptor_state_payload) =
            response_json(descriptor_state).await;
        assert_eq!(descriptor_state_status, StatusCode::OK);
        let component = &descriptor_state_payload["components"][0];
        assert_eq!(component["name"], "System instruction");
        assert_eq!(
            component["metadata"],
            serde_json::json!({"language": "en", "scope": "system"})
        );
        assert_eq!(component["content"], content);
        assert_eq!(component["content_hash"], content_hash);
        assert_eq!(component["content_commit_id"], content_commit_id);
    }

    #[tokio::test]
    async fn protected_local_lifecycle_initializes_an_unborn_context_branch() {
        let (state, context_id, token) = protected_test_state();
        let path = format!("/api/v1/local/contexts/{context_id}/component-lifecycle-commits");
        let body = serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": null,
            "message": "Initialize Context lifecycle",
            "operation": { "kind": "initialize" }
        });

        let created = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-initialize-001",
                body.clone(),
            ))
            .await
            .expect("initialized lifecycle response");
        let (created_status, created_payload) = response_json(created).await;

        assert_eq!(created_status, StatusCode::CREATED);
        assert_eq!(created_payload["disposition"], "created");
        assert_eq!(created_payload["snapshot"]["context_id"], context_id);
        assert_eq!(
            created_payload["snapshot"]["graph"]["edges"],
            serde_json::json!([])
        );
        assert_eq!(
            created_payload["snapshot"]["graph"]["nodes"]
                .as_object()
                .map(|nodes| nodes.len()),
            Some(1)
        );

        let replayed = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-initialize-001",
                body,
            ))
            .await
            .expect("replayed lifecycle response");
        let (replayed_status, replayed_payload) = response_json(replayed).await;

        assert_eq!(replayed_status, StatusCode::OK);
        assert_eq!(replayed_payload["disposition"], "replayed");
        assert_eq!(replayed_payload["commit_id"], created_payload["commit_id"]);
        assert!(
            build_router_with_state(state)
                .oneshot(
                    Request::builder()
                        .uri(path)
                        .body(Body::empty())
                        .expect("request")
                )
                .await
                .expect("public router response")
                .status()
                .is_client_error()
        );
    }

    #[tokio::test]
    async fn protected_local_context_lifecycle_rejects_a_null_head_for_every_non_initialize_operation()
     {
        let (state, context_id, token) = protected_test_state();
        let expected_head_commit_id = "11111111-1111-4111-8111-111111111111";
        let component_id = "22222222-2222-4222-8222-222222222222";
        let mut create = local_lifecycle_create_body(expected_head_commit_id);
        let mut update = local_lifecycle_update_body(expected_head_commit_id, component_id);
        let mut update_descriptor =
            local_lifecycle_update_descriptor_body(expected_head_commit_id, component_id);
        let mut remove = local_lifecycle_remove_body(expected_head_commit_id, component_id);
        let mut add_uses_relationship = local_lifecycle_add_uses_relationship_body(
            expected_head_commit_id,
            component_id,
            "33333333-3333-4333-8333-333333333333",
        );
        let mut remove_uses_relationship = local_lifecycle_remove_uses_relationship_body(
            expected_head_commit_id,
            component_id,
            "33333333-3333-4333-8333-333333333333",
        );

        for body in [
            &mut create,
            &mut update,
            &mut update_descriptor,
            &mut remove,
            &mut add_uses_relationship,
            &mut remove_uses_relationship,
        ] {
            body["expected_head_commit_id"] = Value::Null;
        }

        for (idempotency_key, body) in [
            ("lifecycle-null-create-001", create),
            ("lifecycle-null-update-001", update),
            ("lifecycle-null-descriptor-001", update_descriptor),
            ("lifecycle-null-remove-001", remove),
            (
                "lifecycle-null-add-uses-relationship-001",
                add_uses_relationship,
            ),
            (
                "lifecycle-null-remove-uses-relationship-001",
                remove_uses_relationship,
            ),
        ] {
            let response = build_protected_router_with_state(state.clone())
                .oneshot(local_lifecycle_write_request(
                    &context_id,
                    &token,
                    idempotency_key,
                    body,
                ))
                .await
                .expect("invalid lifecycle response");
            let (status, payload) = response_json(response).await;

            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(payload["error"], "invalid_context_lifecycle_request");
            assert!(
                payload["message"]
                    .as_str()
                    .expect("error message")
                    .contains("expected_head_commit_id is required")
            );
        }

        assert!(
            state
                .commit_repository()
                .list_commits(context_id, contextlab_storage::CommitListQuery::default(),)
                .await
                .expect("list rejected lifecycle commits")
                .items
                .is_empty()
        );
    }

    #[tokio::test]
    async fn protected_local_context_lifecycle_strictly_decodes_initialize_only() {
        let (state, context_id, token) = protected_test_state();

        for (idempotency_key, field, value) in [
            (
                "lifecycle-initialize-component-field-001",
                "component_kind",
                Value::String("prompt".to_owned()),
            ),
            (
                "lifecycle-initialize-body-field-001",
                "content",
                Value::String("Must not become a root body revision.".to_owned()),
            ),
            (
                "lifecycle-initialize-descriptor-field-001",
                "metadata",
                serde_json::json!({"language": "en"}),
            ),
        ] {
            let mut body = serde_json::json!({
                "branch_name": "main",
                "expected_head_commit_id": null,
                "message": "Initialize Context lifecycle",
                "operation": { "kind": "initialize" }
            });
            body["operation"][field] = value;

            let response = build_protected_router_with_state(state.clone())
                .oneshot(local_lifecycle_write_request(
                    &context_id,
                    &token,
                    idempotency_key,
                    body,
                ))
                .await
                .expect("malformed initialization response");
            let (status, payload) = response_json(response).await;

            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(payload["error"], "invalid_context_lifecycle_request");
            assert!(
                payload["message"]
                    .as_str()
                    .expect("error message")
                    .contains(&format!("unknown field `{field}`"))
            );
        }

        assert!(
            state
                .commit_repository()
                .list_commits(context_id, contextlab_storage::CommitListQuery::default(),)
                .await
                .expect("list malformed initialization commits")
                .items
                .is_empty()
        );
    }

    #[tokio::test]
    async fn protected_local_context_lifecycle_rejects_unknown_top_level_fields() {
        let (state, context_id, token) = protected_test_state();
        let body = serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": null,
            "message": "Initialize Context lifecycle",
            "operation": { "kind": "initialize" },
            "unexpected_top_level_field": true
        });

        let response = build_protected_router_with_state(state)
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-initialize-top-level-field-001",
                body,
            ))
            .await
            .expect("invalid lifecycle response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_lifecycle_request");
    }

    #[tokio::test]
    async fn local_context_lifecycle_descriptor_updates_stay_private_and_protected() {
        let (state, context_id, token) = protected_test_state();
        let path = format!("/api/v1/local/contexts/{context_id}/component-lifecycle-commits");
        let body = local_lifecycle_update_descriptor_body(
            "11111111-1111-4111-8111-111111111111",
            "22222222-2222-4222-8222-222222222222",
        );

        let public = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&path)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("public request"),
            )
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let missing_bearer = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&path)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .expect("missing bearer request"),
            )
            .await
            .expect("missing bearer response");
        assert_eq!(missing_bearer.status(), StatusCode::UNAUTHORIZED);

        let rate_limited = build_protected_router_with_state(state.with_protected_rate_limiter(
            StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
                retry_after_seconds: 7,
            })),
        ))
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("rate limited request"),
        )
        .await
        .expect("rate limited response");
        assert_eq!(rate_limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            rate_limited
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok()),
            Some("7")
        );
    }

    #[tokio::test]
    async fn local_knowledge_memory_projection_is_authenticated_scoped_redacted_and_no_store() {
        let (state, context_id, token) = protected_test_state();
        let project_id = "22222222-2222-4222-8222-222222222222";
        let path = format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/11111111-1111-4111-8111-111111111111/knowledge-memory-projection"
        );

        let missing_bearer = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("missing bearer request"),
            )
            .await
            .expect("missing bearer response");
        assert_eq!(missing_bearer.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            missing_bearer
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );

        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("projection request"),
            )
            .await
            .expect("projection response");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        let (_, payload) = response_json(response).await;
        assert_eq!(
            payload["schema_version"],
            knowledge_memory::LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1
        );
        assert_eq!(payload["context_id"], context_id);
        assert_eq!(payload["project_id"], project_id);
        assert_eq!(payload["commit_id"], "11111111-1111-4111-8111-111111111111");
        assert_json_excludes_keys_recursively(
            &payload,
            &[
                "content",
                "query",
                "query_fingerprint",
                "embedding",
                "vector",
                "credentials",
                "provider",
            ],
        );
    }

    #[tokio::test]
    async fn local_knowledge_memory_projection_rejects_scope_and_schema_drift() {
        let (state, context_id, token) = protected_test_state();
        let project_id = ProjectId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project UUID"),
        );
        let commit_id = CommitId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("commit UUID"),
        );
        let context_uuid =
            ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context UUID"));
        let resource = knowledge_memory::InMemoryKnowledgeMemoryProjectionRepository::default()
            .project(project_id, context_uuid, commit_id)
            .await
            .expect("projection fixture");

        let mut scope_drift = resource.clone();
        scope_drift.context_id = "99999999-9999-4999-8999-999999999999".to_owned();
        let scope_state = state.clone().with_knowledge_memory_projection_repository(
            FixedKnowledgeMemoryProjectionRepository {
                resource: scope_drift,
            },
        );
        let path = format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/knowledge-memory-projection",
            project_id = project_id
        );
        let response = build_protected_router_with_state(scope_state)
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("scope drift request"),
            )
            .await
            .expect("scope drift response");
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "knowledge_memory_projection_invalid");

        let mut schema_drift = resource;
        schema_drift.schema_version = "contextlab.local-knowledge-memory-projection.v999";
        let schema_state = state.with_knowledge_memory_projection_repository(
            FixedKnowledgeMemoryProjectionRepository {
                resource: schema_drift,
            },
        );
        let response = build_protected_router_with_state(schema_state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("schema drift request"),
            )
            .await
            .expect("schema drift response");
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "knowledge_memory_projection_invalid");
    }

    #[tokio::test]
    async fn local_knowledge_memory_projection_enforces_context_read_and_own_quota() {
        let (state, context_id, token) = protected_test_state_with_authorizer(DenyContextReads);
        let path = format!(
            "/api/v1/local/projects/22222222-2222-4222-8222-222222222222/contexts/{context_id}/commits/11111111-1111-4111-8111-111111111111/knowledge-memory-projection"
        );
        let forbidden = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("forbidden request"),
            )
            .await
            .expect("forbidden response");
        let (status, payload) = response_json(forbidden).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_read_forbidden");

        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 5,
        }));
        let rate_limited = build_protected_router_with_state(
            protected_test_state()
                .0
                .with_protected_rate_limiter(limiter.clone()),
        )
        .oneshot(
            Request::builder()
                .uri(path)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .expect("rate-limited request"),
        )
        .await
        .expect("rate-limited response");
        assert_eq!(rate_limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::KnowledgeMemoryProjectionRead]
        );
    }

    #[derive(Clone)]
    struct FixedKnowledgeMemoryProjectionRepository {
        resource: knowledge_memory::KnowledgeMemoryProjectionResource,
    }

    #[async_trait::async_trait]
    impl knowledge_memory::KnowledgeMemoryProjectionRepository
        for FixedKnowledgeMemoryProjectionRepository
    {
        async fn project(
            &self,
            _project_id: ProjectId,
            _context_id: ContextId,
            _commit_id: CommitId,
        ) -> Result<
            knowledge_memory::KnowledgeMemoryProjectionResource,
            knowledge_memory::KnowledgeMemoryProjectionError,
        > {
            Ok(self.resource.clone())
        }
    }

    #[tokio::test]
    async fn local_lifecycle_routes_are_private_authenticated_and_rate_limited() {
        let (state, context_id, token) = protected_test_state();
        let path = format!(
            "/api/v1/local/contexts/{context_id}/commits/11111111-1111-4111-8111-111111111111/lifecycle-state"
        );

        let public = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public request"),
            )
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let missing_bearer = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("missing bearer request"),
            )
            .await
            .expect("missing bearer response");
        assert_eq!(missing_bearer.status(), StatusCode::UNAUTHORIZED);

        let rate_limited = build_protected_router_with_state(state.with_protected_rate_limiter(
            StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
                retry_after_seconds: 7,
            })),
        ))
        .oneshot(
            Request::builder()
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("rate limited request"),
        )
        .await
        .expect("rate limited response");
        assert_eq!(rate_limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            rate_limited
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok()),
            Some("7")
        );
    }

    #[tokio::test]
    async fn local_workflow_capability_status_is_private_authenticated_and_default_off() {
        let (state, context_id, token) = protected_test_state();
        let path = format!("/api/v1/local/contexts/{context_id}/workflow/capability-status");

        let public = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public request"),
            )
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let missing_bearer = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("missing bearer request"),
            )
            .await
            .expect("missing bearer response");
        assert_eq!(missing_bearer.status(), StatusCode::UNAUTHORIZED);

        let protected = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("protected request"),
            )
            .await
            .expect("protected response");
        let (status, payload) = response_json(protected).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            payload,
            serde_json::json!({
                "schema_version": "contextlab.local-workflow-capability-status.v1",
                "capability": "workflow",
                "enabled": false,
                "availability": "unavailable",
                "reason": "shared_integration_not_registered"
            })
        );
    }

    #[tokio::test]
    async fn local_workflow_execution_status_is_private_and_unavailable_without_a_reader() {
        let (state, context_id, token) = protected_test_state();
        let run_id = contextlab_workflow::WorkflowRunId::from_uuid(Uuid::from_u128(9_901));
        let path = format!(
            "/api/v1/local/contexts/{context_id}/workflow/runs/{}/status",
            run_id.as_uuid()
        );

        let public = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public request"),
            )
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let missing_bearer = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("missing bearer request"),
            )
            .await
            .expect("missing bearer response");
        assert_eq!(missing_bearer.status(), StatusCode::UNAUTHORIZED);

        let protected = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("protected request"),
            )
            .await
            .expect("protected response");
        let (status, payload) = response_json(protected).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "workflow_execution_status_unavailable");
    }

    #[tokio::test]
    async fn local_plugin_capability_availability_is_private_authenticated_and_empty_by_default() {
        let (state, context_id, token) = protected_test_state();
        let path = format!("/api/v1/local/contexts/{context_id}/plugins/capabilities");

        let public = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public request"),
            )
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let missing_bearer = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("missing bearer request"),
            )
            .await
            .expect("missing bearer response");
        assert_eq!(missing_bearer.status(), StatusCode::UNAUTHORIZED);

        let protected = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("protected request"),
            )
            .await
            .expect("protected response");
        let (status, payload) = response_json(protected).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            payload,
            serde_json::json!({
                "schema_version": "contextlab.local-plugin-capability-availability.v1",
                "context_id": context_id.to_string(),
                "entries": []
            })
        );
    }

    #[tokio::test]
    async fn local_workflow_bindings_denies_context_read_before_repository_access() {
        let (state, context_id, token) = protected_test_state_with_authorizer(DenyContextReads);
        let commit_id = CommitId::new();
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state =
            state.with_context_workflow_binding_repository(RecordingWorkflowBindingRepository {
                bindings: Vec::new(),
                calls: calls.clone(),
                fail_reads: false,
            });

        let response = build_protected_router_with_state(state)
            .oneshot(workflow_bindings_request(
                &context_id,
                commit_id,
                Some(&token),
            ))
            .await
            .expect("workflow binding response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_read_forbidden");
        assert!(
            calls
                .lock()
                .expect("workflow binding calls lock")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn local_workflow_bindings_fail_closed_when_repository_is_unavailable() {
        let (state, context_id, token) = protected_test_state();
        let commit_id = CommitId::new();

        let response = build_protected_router_with_state(state.clone())
            .oneshot(workflow_bindings_request(
                &context_id,
                commit_id,
                Some(&token),
            ))
            .await
            .expect("workflow binding response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "workflow_bindings_unavailable");

        let public = build_router_with_state(state)
            .oneshot(workflow_bindings_request(&context_id, commit_id, None))
            .await
            .expect("public workflow binding response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn local_workflow_bindings_forwards_exact_context_and_commit_scope() {
        let (state, context_id, token) = protected_test_state();
        let context_id = ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id"));
        let commit_id = CommitId::new();
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let binding = workflow_binding(context_id, commit_id, 1_001, 2_001, 1, 0);
        let state =
            state.with_context_workflow_binding_repository(RecordingWorkflowBindingRepository {
                bindings: vec![binding],
                calls: calls.clone(),
                fail_reads: false,
            });

        let response = build_protected_router_with_state(state)
            .oneshot(workflow_bindings_request(
                &context_id.to_string(),
                commit_id,
                Some(&token),
            ))
            .await
            .expect("workflow binding response");
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["context_id"], context_id.to_string());
        assert_eq!(payload["commit_id"], commit_id.to_string());
        assert_eq!(
            *calls.lock().expect("workflow binding calls lock"),
            vec![(context_id, commit_id)]
        );
    }

    #[tokio::test]
    async fn local_workflow_bindings_returns_only_safe_summary_counts() {
        let (state, context_id, token) = protected_test_state();
        let context_id = ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id"));
        let commit_id = CommitId::new();
        let binding_id = Uuid::from_u128(1_001);
        let workflow_id = Uuid::from_u128(2_001);
        let second_binding_id = Uuid::from_u128(1_002);
        let second_workflow_id = Uuid::from_u128(2_002);
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state =
            state.with_context_workflow_binding_repository(RecordingWorkflowBindingRepository {
                bindings: vec![
                    workflow_binding(context_id, commit_id, 1_001, 2_001, 2, 1),
                    workflow_binding(context_id, commit_id, 1_002, 2_002, 1, 0),
                ],
                calls,
                fail_reads: false,
            });

        let response = build_protected_router_with_state(state)
            .oneshot(workflow_bindings_request(
                &context_id.to_string(),
                commit_id,
                Some(&token),
            ))
            .await
            .expect("workflow binding response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            payload,
            serde_json::json!({
                "schema_version": "contextlab.local-workflow-context-bindings.v1",
                "context_id": context_id.to_string(),
                "commit_id": commit_id.to_string(),
                "bindings": [
                    {
                        "binding_id": binding_id.to_string(),
                        "workflow_id": workflow_id.to_string(),
                        "workflow_revision": 1,
                        "node_count": 2,
                        "edge_count": 1,
                    },
                    {
                        "binding_id": second_binding_id.to_string(),
                        "workflow_id": second_workflow_id.to_string(),
                        "workflow_revision": 1,
                        "node_count": 1,
                        "edge_count": 0,
                    }
                ]
            })
        );
        assert_json_excludes_keys_recursively(
            &payload,
            &[
                "context_source",
                "workflow_definition",
                "capability_requirements",
                "nodes",
                "edges",
            ],
        );
    }

    #[tokio::test]
    async fn local_benchmark_workspace_route_is_private_and_requires_opt_in_repository() {
        let (state, context_id, token) = protected_test_state();
        let project_id = ProjectId::from_uuid(Uuid::from_u128(71));
        let commit_id = CommitId::from_uuid(Uuid::from_u128(72));
        let cohort_id =
            contextlab_evaluation::BenchmarkExecutionCohortId::from_uuid(Uuid::from_u128(73));
        let path = format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{}",
            cohort_id.as_uuid()
        );

        let public = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public benchmark workspace request"),
            )
            .await
            .expect("public benchmark workspace response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let protected = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("protected benchmark workspace request"),
            )
            .await
            .expect("protected benchmark workspace response");
        let (status, payload) = response_json(protected).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "benchmark_workspace_unavailable");
    }

    #[tokio::test]
    async fn local_benchmark_workspace_authenticates_before_its_dedicated_quota_and_never_caches() {
        let (state, context_id, token) = protected_test_state();
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Allowed));
        let path = format!(
            "/api/v1/local/projects/{}/contexts/{context_id}/commits/{}/benchmark-workspace/{}",
            Uuid::from_u128(81),
            Uuid::from_u128(82),
            Uuid::from_u128(83),
        );

        let unauthenticated = build_protected_router_with_state(
            state.clone().with_protected_rate_limiter(limiter.clone()),
        )
        .oneshot(
            Request::builder()
                .uri(&path)
                .body(Body::empty())
                .expect("unauthenticated benchmark workspace request"),
        )
        .await
        .expect("unauthenticated benchmark workspace response");
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(limiter.calls(), 0);
        assert_eq!(
            unauthenticated
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );

        let unavailable =
            build_protected_router_with_state(state.with_protected_rate_limiter(limiter.clone()))
                .oneshot(
                    Request::builder()
                        .uri(path)
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::empty())
                        .expect("authenticated benchmark workspace request"),
                )
                .await
                .expect("authenticated benchmark workspace response");
        assert_eq!(unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            unavailable
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::BenchmarkWorkspaceRead]
        );
    }

    #[tokio::test]
    async fn local_benchmark_workspace_forwards_exact_comparison_scope_and_redacts_projection() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_workspace_fixture(&context_id);
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state.with_benchmark_workspace_projection_repository(
            RecordingBenchmarkWorkspaceRepository {
                projection: fixture.projection.clone(),
                calls: calls.clone(),
            },
        );
        let path = format!(
            "/api/v1/local/projects/{}/contexts/{}/commits/{}/benchmark-workspace/{}?baseline_commit_id={}&baseline_cohort_id={}",
            fixture.project_id,
            fixture.context_id,
            fixture.revised_commit_id,
            fixture.revised_cohort_id.as_uuid(),
            fixture.baseline_commit_id,
            fixture.baseline_cohort_id.as_uuid(),
        );

        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("benchmark workspace comparison request"),
            )
            .await
            .expect("benchmark workspace comparison response");
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            payload["schema_version"],
            "contextlab.local-benchmark-workspace.v1"
        );
        assert_eq!(payload["project_id"], fixture.project_id.to_string());
        assert_eq!(payload["context_id"], fixture.context_id.to_string());
        assert_eq!(
            payload["baseline"]["cohort_id"],
            fixture.baseline_cohort_id.as_uuid().to_string()
        );
        assert_eq!(
            payload["revised"]["cohort_id"],
            fixture.revised_cohort_id.as_uuid().to_string()
        );
        assert_eq!(payload["projection"]["schema_version"], 1);
        assert_json_excludes_keys_recursively(
            &payload,
            &["input", "expected_output", "model_output", "measurements"],
        );
        assert_eq!(
            *calls.lock().expect("benchmark workspace calls lock"),
            vec![BenchmarkWorkspaceProjectionV1Query::comparing(
                BenchmarkWorkspaceProjectionReceiptScope::new(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.baseline_commit_id,
                    fixture.baseline_cohort_id,
                ),
                BenchmarkWorkspaceProjectionReceiptScope::new(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.revised_commit_id,
                    fixture.revised_cohort_id,
                ),
            )]
        );
    }

    #[tokio::test]
    async fn local_benchmark_workspace_by_decision_resolves_exact_scope_and_stays_private() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_workspace_fixture(&context_id);
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state.with_benchmark_workspace_projection_repository(
            RecordingBenchmarkWorkspaceRepository {
                projection: fixture.revised_projection.clone(),
                calls: calls.clone(),
            },
        );
        let decision_id = Uuid::from_u128(74);
        let path = format!(
            "/api/v1/local/projects/{}/contexts/{}/commits/{}/benchmark-decisions/{decision_id}/workspace",
            fixture.project_id, fixture.context_id, fixture.revised_commit_id,
        );

        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("decision-bound benchmark workspace request"),
            )
            .await
            .expect("decision-bound benchmark workspace response");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            payload["revised"]["cohort_id"],
            fixture.revised_cohort_id.as_uuid().to_string()
        );
        assert_json_excludes_keys_recursively(
            &payload,
            &["input", "expected_output", "model_output", "measurements"],
        );
        assert_eq!(
            *calls.lock().expect("benchmark workspace calls lock"),
            vec![BenchmarkWorkspaceProjectionV1Query::single(
                BenchmarkWorkspaceProjectionReceiptScope::new(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.revised_commit_id,
                    fixture.revised_cohort_id,
                )
            )]
        );
    }

    #[tokio::test]
    async fn local_benchmark_workspace_by_decision_rejects_an_unresolved_decision() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_workspace_fixture(&context_id);
        let state = state.with_benchmark_workspace_projection_repository(
            RecordingBenchmarkWorkspaceRepository {
                projection: fixture.revised_projection,
                calls: Arc::new(std::sync::Mutex::new(Vec::new())),
            },
        );
        let path = format!(
            "/api/v1/local/projects/{}/contexts/{}/commits/{}/benchmark-decisions/{}/workspace",
            fixture.project_id,
            fixture.context_id,
            fixture.revised_commit_id,
            Uuid::from_u128(404),
        );

        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("unresolved decision request"),
            )
            .await
            .expect("unresolved decision response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(payload["error"], "benchmark_workspace_not_found");
    }

    #[tokio::test]
    async fn local_benchmark_workspace_rejects_repository_projection_scope_drift() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_workspace_fixture(&context_id);
        let requested_baseline_cohort_id =
            contextlab_evaluation::BenchmarkExecutionCohortId::from_uuid(Uuid::from_u128(99));
        let state = state.with_benchmark_workspace_projection_repository(
            RecordingBenchmarkWorkspaceRepository {
                projection: fixture.projection,
                calls: Arc::new(std::sync::Mutex::new(Vec::new())),
            },
        );
        let path = format!(
            "/api/v1/local/projects/{}/contexts/{}/commits/{}/benchmark-workspace/{}?baseline_commit_id={}&baseline_cohort_id={}",
            fixture.project_id,
            fixture.context_id,
            fixture.revised_commit_id,
            fixture.revised_cohort_id.as_uuid(),
            fixture.baseline_commit_id,
            requested_baseline_cohort_id.as_uuid(),
        );

        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("drifted benchmark workspace request"),
            )
            .await
            .expect("drifted benchmark workspace response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "benchmark_workspace_source_conflict");
    }

    #[tokio::test]
    async fn local_benchmark_workspace_rejects_partial_and_unknown_query_contracts() {
        let (state, context_id, token) = protected_test_state();
        let base_path = format!(
            "/api/v1/local/projects/{}/contexts/{context_id}/commits/{}/benchmark-workspace/{}",
            Uuid::from_u128(101),
            Uuid::from_u128(102),
            Uuid::from_u128(103),
        );

        for query in [
            "baseline_commit_id=11111111-1111-4111-8111-111111111111",
            "unexpected=true",
        ] {
            let response = build_protected_router_with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .uri(format!("{base_path}?{query}"))
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::empty())
                        .expect("invalid benchmark workspace query request"),
                )
                .await
                .expect("invalid benchmark workspace query response");
            assert_eq!(
                response
                    .headers()
                    .get(header::CACHE_CONTROL)
                    .and_then(|value| value.to_str().ok()),
                Some("private, no-store")
            );
            let (status, payload) = response_json(response).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(payload["error"], "invalid_benchmark_workspace_request");
        }
    }

    #[tokio::test]
    async fn local_benchmark_workspace_uses_its_own_malformed_scope_error() {
        let (state, context_id, token) = protected_test_state();
        let valid = [
            Uuid::from_u128(131).to_string(),
            context_id,
            Uuid::from_u128(132).to_string(),
            Uuid::from_u128(133).to_string(),
        ];

        for invalid_index in 0..valid.len() {
            let mut scope = valid.clone();
            scope[invalid_index] = "not-a-uuid".to_owned();
            let path = format!(
                "/api/v1/local/projects/{}/contexts/{}/commits/{}/benchmark-workspace/{}",
                scope[0], scope[1], scope[2], scope[3]
            );
            let response = build_protected_router_with_state(state.clone())
                .oneshot(
                    Request::builder()
                        .uri(path)
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::empty())
                        .expect("malformed benchmark workspace request"),
                )
                .await
                .expect("malformed benchmark workspace response");
            let (status, payload) = response_json(response).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(payload["error"], "invalid_benchmark_workspace_request");
        }
    }

    #[tokio::test]
    async fn local_benchmark_workspace_denies_context_read_before_repository_access() {
        let (state, context_id, token) = protected_test_state_with_authorizer(DenyContextReads);
        let fixture = benchmark_workspace_fixture(&context_id);
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state.with_benchmark_workspace_projection_repository(
            RecordingBenchmarkWorkspaceRepository {
                projection: fixture.projection,
                calls: calls.clone(),
            },
        );
        let path = format!(
            "/api/v1/local/projects/{}/contexts/{}/commits/{}/benchmark-workspace/{}",
            fixture.project_id,
            fixture.context_id,
            fixture.revised_commit_id,
            fixture.revised_cohort_id.as_uuid(),
        );

        let response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("denied benchmark workspace request"),
            )
            .await
            .expect("denied benchmark workspace response");
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        assert!(
            calls
                .lock()
                .expect("benchmark workspace calls lock")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn local_benchmark_workspace_maps_storage_failures_without_internal_details() {
        let (state, context_id, token) = protected_test_state();
        let path = format!(
            "/api/v1/local/projects/{}/contexts/{context_id}/commits/{}/benchmark-workspace/{}",
            Uuid::from_u128(121),
            Uuid::from_u128(122),
            Uuid::from_u128(123),
        );
        let cases = [
            (
                BenchmarkWorkspaceFailure::Missing,
                StatusCode::NOT_FOUND,
                "benchmark_workspace_not_found",
            ),
            (
                BenchmarkWorkspaceFailure::Comparison,
                StatusCode::CONFLICT,
                "benchmark_workspace_comparison_unavailable",
            ),
            (
                BenchmarkWorkspaceFailure::StoredSource,
                StatusCode::CONFLICT,
                "benchmark_workspace_source_conflict",
            ),
            (
                BenchmarkWorkspaceFailure::Repository,
                StatusCode::INTERNAL_SERVER_ERROR,
                "benchmark_workspace_unavailable",
            ),
        ];

        for (failure, expected_status, expected_code) in cases {
            let response = build_protected_router_with_state(
                state
                    .clone()
                    .with_benchmark_workspace_projection_repository(
                        FailingBenchmarkWorkspaceRepository { failure },
                    ),
            )
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("failing benchmark workspace request"),
            )
            .await
            .expect("failing benchmark workspace response");
            assert_eq!(response.status(), expected_status);
            assert_eq!(
                response
                    .headers()
                    .get(header::CACHE_CONTROL)
                    .and_then(|value| value.to_str().ok()),
                Some("private, no-store")
            );
            let (_, payload) = response_json(response).await;
            assert_eq!(payload["error"], expected_code);
            let serialized = payload.to_string().to_ascii_lowercase();
            assert!(!serialized.contains("sql"));
            assert!(!serialized.contains("digest"));
            assert!(!serialized.contains("receipt conflict"));
        }
    }

    #[tokio::test]
    async fn local_benchmark_evidence_route_is_private_and_requires_opt_in_composition() {
        let (state, context_id, token) = protected_test_state();
        let path = format!(
            "/api/v1/local/projects/22222222-2222-4222-8222-222222222222/contexts/{context_id}/commits/11111111-1111-4111-8111-111111111111/benchmark-decisions/33333333-3333-4333-8333-333333333333"
        );

        let public = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public request"),
            )
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let protected = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("protected request"),
            )
            .await
            .expect("protected response");
        let (status, payload) = response_json(protected).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "benchmark_evidence_unavailable");
    }

    #[tokio::test]
    async fn local_benchmark_evidence_route_uses_its_own_rate_limit_operation() {
        let (state, context_id, token) = protected_test_state();
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Allowed));
        let state = state.with_protected_rate_limiter(limiter.clone());
        let request = Request::builder()
            .uri(format!(
                "/api/v1/local/projects/22222222-2222-4222-8222-222222222222/contexts/{context_id}/commits/11111111-1111-4111-8111-111111111111/benchmark-decisions/33333333-3333-4333-8333-333333333333"
            ))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .expect("benchmark decision request");

        let response = build_protected_router_with_state(state)
            .oneshot(request)
            .await
            .expect("benchmark decision response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::BenchmarkDecisionRead]
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_diff_route_is_private_and_uses_its_own_rate_limit_operation()
    {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let request = benchmark_decision_diff_request(
            fixture.project_id,
            fixture.context_id,
            fixture.commit_id,
            fixture.decision_id,
            CommitId::new(),
            BenchmarkDecisionId::new(),
            Some(&token),
        );

        let public = build_router_with_state(state.clone())
            .oneshot(request)
            .await
            .expect("public benchmark decision diff response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);

        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Allowed));
        let response =
            build_protected_router_with_state(state.with_protected_rate_limiter(limiter.clone()))
                .oneshot(benchmark_decision_diff_request(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.commit_id,
                    fixture.decision_id,
                    CommitId::new(),
                    BenchmarkDecisionId::new(),
                    Some(&token),
                ))
                .await
                .expect("protected benchmark decision diff response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "benchmark_evidence_unavailable");
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::BenchmarkDecisionDiffRead]
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_diff_forwards_exact_scopes_and_redacts_case_payloads() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let revised_commit_id = CommitId::new();
        let revised_decision_id = BenchmarkDecisionId::new();
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state.with_benchmark_evidence_repository(
            RecordingBenchmarkDecisionComparisonRepository {
                decision: Some(fixture.evidence.clone()),
                fail_reads: false,
                calls: calls.clone(),
            },
        );

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_diff_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                revised_commit_id,
                revised_decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision diff response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["project_id"], fixture.project_id.to_string());
        assert_eq!(payload["context_id"], fixture.context_id.to_string());
        assert_eq!(
            payload["baseline"]["commit_id"],
            fixture.commit_id.to_string()
        );
        assert_eq!(
            payload["baseline"]["decision_id"],
            fixture.decision_id.to_string()
        );
        assert_eq!(
            payload["revised"]["commit_id"],
            revised_commit_id.to_string()
        );
        assert_eq!(
            payload["revised"]["decision_id"],
            revised_decision_id.to_string()
        );
        assert_json_excludes_keys_recursively(&payload, &["cases", "input", "expected_output"]);
        assert_eq!(
            *calls
                .lock()
                .expect("benchmark decision comparison calls lock"),
            vec![(
                fixture.project_id,
                fixture.context_id,
                BenchmarkDecisionComparisonScope::new(fixture.commit_id, fixture.decision_id),
                BenchmarkDecisionComparisonScope::new(revised_commit_id, revised_decision_id),
            )]
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_forwards_exact_scope_and_redacts_case_payloads() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state =
            state.with_benchmark_evidence_repository(RecordingBenchmarkDecisionRepository {
                decision: Some(fixture.evidence.clone()),
                suite: Some(fixture.suite.clone()),
                datasets: fixture.datasets.clone(),
                fail_reads: false,
                calls: calls.clone(),
            });

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["project_id"], fixture.project_id.to_string());
        assert_eq!(payload["context_id"], fixture.context_id.to_string());
        assert_eq!(payload["commit_id"], fixture.commit_id.to_string());
        assert_eq!(payload["decision_id"], fixture.decision_id.to_string());
        assert_eq!(
            payload["definition"]["suite"]["id"],
            fixture.suite.id().to_string()
        );
        assert_eq!(payload["definition"]["suite"]["name"], fixture.suite.name());
        assert_eq!(
            payload["definition"]["suite"]["thresholds"],
            serde_json::to_value(fixture.suite.thresholds()).expect("threshold JSON")
        );
        assert_eq!(
            payload["definition"]["datasets"],
            serde_json::json!([{
                "id": fixture.datasets[0].id().to_string(),
                "name": fixture.datasets[0].name(),
                "case_count": fixture.datasets[0].cases().len(),
            }])
        );
        assert_json_excludes_keys_recursively(
            &payload,
            &[
                "cases",
                "input",
                "expected_output",
                "runs",
                "measurements",
                "output",
                "model_output",
            ],
        );
        assert_eq!(
            *calls.lock().expect("benchmark decision calls lock"),
            vec![(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
            )]
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_list_projects_exact_scope_and_redacts_case_payloads() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let state = state.with_benchmark_evidence_repository(fixture.repository.clone());

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_list_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision list response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            payload["schema_version"],
            "contextlab.local-benchmark-decision-list.v1"
        );
        assert_eq!(payload["project_id"], fixture.project_id.to_string());
        assert_eq!(payload["context_id"], fixture.context_id.to_string());
        assert_eq!(payload["commit_id"], fixture.commit_id.to_string());
        assert_eq!(payload["decisions"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            payload["decisions"][0]["decision_id"],
            fixture.decision_id.to_string()
        );
        assert_eq!(
            payload["decisions"][0]["suite"],
            serde_json::json!({
                "id": fixture.suite.id().to_string(),
                "name": fixture.suite.name(),
            })
        );
        assert_eq!(
            payload["decisions"][0]["datasets"],
            serde_json::json!([{
                "id": fixture.datasets[0].id().to_string(),
                "name": fixture.datasets[0].name(),
                "case_count": fixture.datasets[0].cases().len(),
            }])
        );
        assert_eq!(
            payload["decisions"][0]["run_count"],
            fixture.evidence.run_ids().len()
        );
        assert_json_excludes_keys_recursively(
            &payload,
            &[
                "cases",
                "input",
                "expected_output",
                "runs",
                "measurements",
                "output",
                "model_output",
            ],
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_list_is_private_and_fails_closed() {
        let (state, context_id, token) = protected_test_state();
        let request = benchmark_decision_list_request(
            ProjectId::from_uuid(
                Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
            ),
            ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id")),
            CommitId::from_uuid(
                Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
            ),
            Some(&token),
        );
        let response = build_protected_router_with_state(state)
            .oneshot(request)
            .await
            .expect("benchmark decision list response");
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "benchmark_evidence_unavailable");

        let (state, context_id, token) = protected_test_state_with_authorizer(DenyContextReads);
        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_list_request(
                ProjectId::from_uuid(
                    Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
                ),
                ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id")),
                CommitId::from_uuid(
                    Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
                ),
                Some(&token),
            ))
            .await
            .expect("denied benchmark decision list response");
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_read_forbidden");
    }

    #[tokio::test]
    async fn local_benchmark_decision_list_is_absent_from_public_router_and_openapi() {
        let (state, context_id, _) = protected_test_state();
        let project_id = ProjectId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
        );
        let context_id = ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id"));
        let commit_id = CommitId::from_uuid(
            Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
        );
        let path = format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions"
        );

        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public benchmark decision list request"),
            )
            .await
            .expect("public benchmark decision list response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let openapi: Value = serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("checked-in OpenAPI contract must parse");
        let contract_path = ProtectedBenchmarkGetRoute::List.path();
        assert!(!public_get_route_contracts().contains_key(contract_path));
        assert!(
            openapi["paths"].get(contract_path).is_none(),
            "private benchmark decision list must not be published in OpenAPI"
        );
    }

    #[tokio::test]
    async fn local_benchmark_definition_binding_list_is_private_exact_scope_and_redacted() {
        let (state, context_id, token) = protected_test_state();
        let project_id = ProjectId::from_uuid(
            Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
        );
        let context_id = ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id"));
        let commit_id = CommitId::from_uuid(
            Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
        );
        let path = format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings"
        );

        let public_response = build_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .body(Body::empty())
                    .expect("public binding list request"),
            )
            .await
            .expect("public binding list response");
        assert_eq!(public_response.status(), StatusCode::NOT_FOUND);

        let protected_response = build_protected_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .expect("protected binding list request"),
            )
            .await
            .expect("protected binding list response");
        assert_eq!(protected_response.status(), StatusCode::OK);
        assert_eq!(
            protected_response.headers().get("cache-control").unwrap(),
            "private, no-store"
        );
        let (_, payload) = response_json(protected_response).await;
        assert_eq!(
            payload["schema_version"],
            "contextlab.local-benchmark-definition-binding-inspection.v1"
        );
        assert_eq!(payload["project_id"], project_id.to_string());
        assert_eq!(payload["context_id"], context_id.to_string());
        assert_eq!(payload["commit_id"], commit_id.to_string());
        assert_eq!(payload["bindings"], serde_json::json!([]));
        assert!(!payload.to_string().contains("raw_cases"));
    }

    #[tokio::test]
    async fn local_benchmark_decision_list_rejects_unauthenticated_requests_before_rate_limiting() {
        let (state, context_id, _) = protected_test_state();
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 17,
        }));
        let state = state.with_protected_rate_limiter(limiter.clone());
        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_list_request(
                ProjectId::from_uuid(
                    Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
                ),
                ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id")),
                CommitId::from_uuid(
                    Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
                ),
                None,
            ))
            .await
            .expect("unauthenticated benchmark decision list response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(payload["error"], "authentication_required");
        assert_eq!(limiter.calls(), 0);
        assert!(limiter.operations().is_empty());
    }

    #[tokio::test]
    async fn local_benchmark_decision_list_enforces_benchmark_decision_read_rate_limit() {
        let (state, context_id, token) = protected_test_state();
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 17,
        }));
        let state = state.with_protected_rate_limiter(limiter.clone());
        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_list_request(
                ProjectId::from_uuid(
                    Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
                ),
                ContextId::from_uuid(Uuid::parse_str(&context_id).expect("context id")),
                CommitId::from_uuid(
                    Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
                ),
                Some(&token),
            ))
            .await
            .expect("rate-limited benchmark decision list response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(payload["error"], "rate_limit_exceeded");
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::BenchmarkDecisionRead]
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_run_details_are_private_and_redacted() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let state = state.with_benchmark_evidence_repository(fixture.repository.clone());

        let response = build_protected_router_with_state(state.clone())
            .oneshot(benchmark_decision_run_details_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark run-detail response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["project_id"], fixture.project_id.to_string());
        assert_eq!(payload["context_id"], fixture.context_id.to_string());
        assert_eq!(payload["commit_id"], fixture.commit_id.to_string());
        assert_eq!(payload["decision_id"], fixture.decision_id.to_string());
        assert_eq!(payload["runs"].as_array().map(Vec::len), Some(1));
        assert_eq!(
            payload["runs"][0]["run_id"],
            fixture.evidence.run_ids()[0].as_uuid().to_string()
        );
        assert_eq!(payload["runs"][0]["model_version"], "model-a");
        assert_eq!(payload["runs"][0]["temperature"], 0.2);
        assert_eq!(
            payload["runs"][0]["metrics"],
            serde_json::json!([
                {"metric": "accuracy", "value": 0.95},
                {"metric": "latency_ms", "value": 700.0}
            ])
        );
        assert_json_excludes_keys_recursively(
            &payload,
            &[
                "cases",
                "input",
                "expected_output",
                "output",
                "model_output",
                "measurements",
            ],
        );

        let public = build_router_with_state(state)
            .oneshot(benchmark_decision_run_details_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                None,
            ))
            .await
            .expect("public benchmark run-detail response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn local_benchmark_decision_requires_authentication() {
        let (state, context_id, _) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                None,
            ))
            .await
            .expect("benchmark decision response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(payload["error"], "authentication_required");
    }

    #[tokio::test]
    async fn local_benchmark_decision_requests_context_read_permission() {
        let (state, context_id, token) =
            protected_test_state_with_authorizer(ReadOnlyContextAccess);
        let fixture = benchmark_decision_fixture(&context_id).await;
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state =
            state.with_benchmark_evidence_repository(RecordingBenchmarkDecisionRepository {
                decision: Some(fixture.evidence.clone()),
                suite: Some(fixture.suite.clone()),
                datasets: fixture.datasets.clone(),
                fail_reads: false,
                calls,
            });

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(payload["decision_id"], fixture.decision_id.to_string());
    }

    #[tokio::test]
    async fn local_benchmark_decision_denies_context_read_before_repository_access() {
        let (state, context_id, token) = protected_test_state_with_authorizer(DenyContextReads);
        let fixture = benchmark_decision_fixture(&context_id).await;
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state =
            state.with_benchmark_evidence_repository(RecordingBenchmarkDecisionRepository {
                decision: Some(fixture.evidence.clone()),
                suite: Some(fixture.suite.clone()),
                datasets: fixture.datasets.clone(),
                fail_reads: false,
                calls: calls.clone(),
            });

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_read_forbidden");
        assert!(
            calls
                .lock()
                .expect("benchmark decision calls lock")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_rate_limit_rejection_prevents_read() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state
            .with_benchmark_evidence_repository(RecordingBenchmarkDecisionRepository {
                decision: Some(fixture.evidence.clone()),
                suite: Some(fixture.suite.clone()),
                datasets: fixture.datasets.clone(),
                fail_reads: false,
                calls: calls.clone(),
            })
            .with_authorization_audit_sink(RecordingAuditSink {
                events: events.clone(),
            })
            .with_protected_rate_limiter(StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
                retry_after_seconds: 11,
            })));

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision response");
        let retry_after = response.headers()[header::RETRY_AFTER].clone();
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(retry_after, "11");
        assert_eq!(payload["error"], "rate_limit_exceeded");
        assert!(
            calls
                .lock()
                .expect("benchmark decision calls lock")
                .is_empty()
        );
        assert!(events.lock().expect("audit events lock").is_empty());
    }

    #[tokio::test]
    async fn local_benchmark_decision_redacts_storage_failures() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state =
            state.with_benchmark_evidence_repository(RecordingBenchmarkDecisionRepository {
                decision: None,
                suite: None,
                datasets: Vec::new(),
                fail_reads: true,
                calls,
            });

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(payload["error"], "benchmark_evidence_unavailable");
        assert_eq!(
            payload["message"],
            "benchmark evidence inspection is unavailable"
        );
    }

    #[tokio::test]
    async fn local_benchmark_decision_redacts_unavailable_definition_metadata() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let state =
            state.with_benchmark_evidence_repository(RecordingBenchmarkDecisionRepository {
                decision: Some(fixture.evidence.clone()),
                suite: None,
                datasets: Vec::new(),
                fail_reads: false,
                calls,
            });

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(payload["error"], "benchmark_evidence_not_found");
        assert_eq!(payload["message"], "benchmark decision was not found");
    }

    #[tokio::test]
    async fn local_benchmark_decision_redacts_definition_membership_mismatch() {
        let (state, context_id, token) = protected_test_state();
        let fixture = benchmark_decision_fixture(&context_id).await;
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mismatched_suite = BenchmarkSuite::with_id(
            fixture.suite.id(),
            "Mismatched suite",
            vec![contextlab_evaluation::BenchmarkDatasetId::new()],
            fixture.suite.thresholds().to_vec(),
        )
        .expect("mismatched suite fixture");
        let state =
            state.with_benchmark_evidence_repository(RecordingBenchmarkDecisionRepository {
                decision: Some(fixture.evidence.clone()),
                suite: Some(mismatched_suite),
                datasets: fixture.datasets.clone(),
                fail_reads: false,
                calls,
            });

        let response = build_protected_router_with_state(state)
            .oneshot(benchmark_decision_request(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
                Some(&token),
            ))
            .await
            .expect("benchmark decision response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(payload["error"], "benchmark_evidence_not_found");
        assert_eq!(payload["message"], "benchmark decision was not found");
    }

    #[tokio::test]
    async fn local_lifecycle_routes_use_operation_scoped_rate_limit_keys() {
        let (state, context_id, token) = protected_test_state();
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Allowed));
        let state = state.with_protected_rate_limiter(limiter.clone());

        let _read = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_state_request(
                &context_id,
                "11111111-1111-4111-8111-111111111111",
                &token,
            ))
            .await
            .expect("local lifecycle read response");
        let _write = build_protected_router_with_state(state)
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-operation-key-001",
                local_lifecycle_create_body("11111111-1111-4111-8111-111111111111"),
            ))
            .await
            .expect("local lifecycle write response");

        assert_eq!(
            limiter.operations(),
            vec![
                ProtectedRouteOperation::ContextLifecycleRead,
                ProtectedRouteOperation::ContextCommitWrite,
            ]
        );
    }

    #[tokio::test]
    async fn protected_local_lifecycle_records_a_forbidden_write_decision() {
        let (state, context_id, token) =
            protected_test_state_with_authorizer(ReadOnlyContextAccess);
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state.with_authorization_audit_sink(RecordingAuditSink {
            events: events.clone(),
        });

        let response = build_protected_router_with_state(state)
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-reader-denied-001",
                local_lifecycle_create_body("11111111-1111-4111-8111-111111111111"),
            ))
            .await
            .expect("forbidden lifecycle response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_write_forbidden");
        let events = events.lock().expect("audit events lock");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].permission(), ContextPermission::Write);
        assert_eq!(events[0].decision(), AuthorizationDecision::Forbidden);
    }

    #[tokio::test]
    async fn protected_local_lifecycle_rejects_client_owned_commit_fields_before_writing() {
        let (state, context_id, token) = protected_test_state();
        let root_commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let mut body = local_lifecycle_create_body(&root_commit_id);
        body["changes"] = serde_json::json!([{"kind": "created_context"}]);

        let response = build_protected_router_with_state(state.clone())
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-client-owned-fields-001",
                body,
            ))
            .await
            .expect("invalid lifecycle response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_lifecycle_request");
        assert!(
            payload["message"]
                .as_str()
                .expect("error message")
                .contains("unknown field `changes`")
        );
        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert_eq!(commits.items.len(), 1);
    }

    #[tokio::test]
    async fn protected_local_lifecycle_reports_an_unavailable_component_as_a_state_conflict() {
        let (state, context_id, token) = protected_test_state();
        let root_commit_id = create_materialized_protected_root(&state, &context_id, &token).await;
        let response = build_protected_router_with_state(state)
            .oneshot(local_lifecycle_write_request(
                &context_id,
                &token,
                "lifecycle-unavailable-component-001",
                local_lifecycle_update_body(
                    &root_commit_id,
                    "11111111-1111-4111-8111-111111111111",
                ),
            ))
            .await
            .expect("unavailable component lifecycle response");
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "context_lifecycle_state_conflict");
    }

    async fn create_materialized_protected_root(
        state: &AppState,
        context_id: &str,
        token: &str,
    ) -> String {
        let mut root_body = protected_commit_body();
        root_body["snapshot"] = serde_json::json!({
            "nodes": [{
                "id": format!("context:{context_id}"),
                "kind": "context",
                "label": "Protected Context"
            }],
            "edges": []
        });
        let root = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                context_id,
                token,
                Some("lifecycle-root-001"),
                Body::from(root_body.to_string()),
            ))
            .await
            .expect("materialized root response");
        let (status, payload) = response_json(root).await;
        assert_eq!(status, StatusCode::CREATED, "payload: {payload}");
        payload["snapshot"]["commit_id"]
            .as_str()
            .expect("materialized root commit id")
            .to_owned()
    }

    fn local_lifecycle_create_body(expected_head_commit_id: &str) -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "message": "Create instruction component",
            "operation": {
                "kind": "create",
                "component_kind": "prompt",
                "name": "Instruction",
                "metadata": {"language": "en"},
                "content": "You are a precise Context engineer."
            }
        })
    }

    fn local_lifecycle_update_body(expected_head_commit_id: &str, component_id: &str) -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "message": "Revise instruction component",
            "operation": {
                "kind": "update",
                "component_id": component_id,
                "content": "Use the Context Graph before responding."
            }
        })
    }

    fn local_lifecycle_update_descriptor_body(
        expected_head_commit_id: &str,
        component_id: &str,
    ) -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "message": "Rename instruction component",
            "operation": {
                "kind": "update_descriptor",
                "component_id": component_id,
                "name": "System instruction",
                "metadata": {"language": "en", "scope": "system"}
            }
        })
    }

    fn local_lifecycle_update_metadata_body(expected_head_commit_id: &str) -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "message": "Update Context metadata",
            "operation": {
                "kind": "update_metadata",
                "metadata": {
                    "created_at": "2026-07-30T00:00:00Z",
                    "updated_at": "2026-07-30T01:00:00Z",
                    "labels": {"owner": "luna"}
                }
            }
        })
    }

    fn local_lifecycle_remove_body(expected_head_commit_id: &str, component_id: &str) -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "message": "Remove instruction component",
            "operation": {
                "kind": "remove",
                "component_id": component_id
            }
        })
    }

    fn local_lifecycle_add_uses_relationship_body(
        expected_head_commit_id: &str,
        source_component_id: &str,
        target_component_id: &str,
    ) -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "message": "Add Uses relationship",
            "operation": {
                "kind": "add_uses_relationship",
                "source_component_id": source_component_id,
                "target_component_id": target_component_id
            }
        })
    }

    fn local_lifecycle_remove_uses_relationship_body(
        expected_head_commit_id: &str,
        source_component_id: &str,
        target_component_id: &str,
    ) -> Value {
        serde_json::json!({
            "branch_name": "main",
            "expected_head_commit_id": expected_head_commit_id,
            "message": "Remove Uses relationship",
            "operation": {
                "kind": "remove_uses_relationship",
                "source_component_id": source_component_id,
                "target_component_id": target_component_id
            }
        })
    }

    fn local_lifecycle_write_request(
        context_id: &str,
        token: &str,
        idempotency_key: &str,
        body: Value,
    ) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/v1/local/contexts/{context_id}/component-lifecycle-commits"
            ))
            .header("authorization", format!("Bearer {token}"))
            .header("idempotency-key", idempotency_key)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .expect("local lifecycle write request")
    }

    fn local_lifecycle_state_request(
        context_id: &str,
        commit_id: &str,
        token: &str,
    ) -> Request<Body> {
        Request::builder()
            .uri(format!(
                "/api/v1/local/contexts/{context_id}/commits/{commit_id}/lifecycle-state"
            ))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .expect("local lifecycle state request")
    }

    #[derive(Clone)]
    struct BenchmarkDecisionFixture {
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
        suite: BenchmarkSuite,
        datasets: Vec<BenchmarkDataset>,
        evidence: BenchmarkDecisionEvidence,
        repository: InMemoryContextGraphRepository,
    }

    #[derive(Clone)]
    struct BenchmarkWorkspaceFixture {
        project_id: ProjectId,
        context_id: ContextId,
        baseline_commit_id: CommitId,
        revised_commit_id: CommitId,
        baseline_cohort_id: contextlab_evaluation::BenchmarkExecutionCohortId,
        revised_cohort_id: contextlab_evaluation::BenchmarkExecutionCohortId,
        projection: BenchmarkWorkspaceProjectionV1,
        revised_projection: BenchmarkWorkspaceProjectionV1,
    }

    fn benchmark_workspace_fixture(context_id: &str) -> BenchmarkWorkspaceFixture {
        let project_id = ProjectId::from_uuid(Uuid::from_u128(91));
        let context_id = ContextId::from_uuid(Uuid::parse_str(context_id).expect("context id"));
        let baseline_commit_id = CommitId::from_uuid(Uuid::from_u128(92));
        let revised_commit_id = CommitId::from_uuid(Uuid::from_u128(93));
        let dataset_id = BenchmarkDatasetId::from_uuid(Uuid::from_u128(94));
        let case_id = BenchmarkCaseId::from_uuid(Uuid::from_u128(95));
        let suite_id = BenchmarkSuiteId::from_uuid(Uuid::from_u128(96));
        let dataset = BenchmarkDataset::with_id(
            dataset_id,
            "Safe support cases",
            vec![
                BenchmarkCase::with_id(
                    case_id,
                    "Refund eligibility",
                    serde_json::json!({"private_input": "never serialize"}),
                    BenchmarkExpectedOutput::Exact(serde_json::json!({"private_expected": true})),
                )
                .expect("benchmark workspace case"),
            ],
        )
        .expect("benchmark workspace dataset");
        let suite = BenchmarkSuite::with_id(
            suite_id,
            "Local regression gate",
            vec![dataset_id],
            vec![
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
            ],
        )
        .expect("benchmark workspace suite");
        let plan = BenchmarkExecutionPlan::new(suite, vec![dataset]).expect("workspace plan");
        let executed_at = Utc
            .with_ymd_and_hms(2026, 7, 23, 2, 0, 0)
            .single()
            .expect("workspace timestamp");
        let baseline = BenchmarkExecutionReceipt::from_plan(
            &plan,
            Uuid::from_u128(97),
            context_id,
            "model-v1",
            0.2,
            executed_at,
            "comparison-fingerprint-v1",
            vec![benchmark_workspace_case_result(
                dataset_id, case_id, 0.95, 700.0,
            )],
        )
        .expect("baseline workspace receipt");
        let revised = BenchmarkExecutionReceipt::from_plan(
            &plan,
            Uuid::from_u128(98),
            context_id,
            "model-v1",
            0.2,
            executed_at,
            "comparison-fingerprint-v1",
            vec![benchmark_workspace_case_result(
                dataset_id, case_id, 0.75, 900.0,
            )],
        )
        .expect("revised workspace receipt");
        let projection = BenchmarkWorkspaceProjectionV1::from_receipts(&baseline, &revised, &plan)
            .expect("benchmark workspace comparison projection");
        let revised_projection = BenchmarkWorkspaceProjectionV1::from_receipt(&revised, &plan)
            .expect("benchmark workspace single projection");

        BenchmarkWorkspaceFixture {
            project_id,
            context_id,
            baseline_commit_id,
            revised_commit_id,
            baseline_cohort_id: baseline.cohort_id(),
            revised_cohort_id: revised.cohort_id(),
            projection,
            revised_projection,
        }
    }

    fn benchmark_workspace_case_result(
        dataset_id: BenchmarkDatasetId,
        case_id: BenchmarkCaseId,
        accuracy: f64,
        latency_ms: f64,
    ) -> BenchmarkCaseExecutionResult {
        BenchmarkCaseExecutionResult::new(
            dataset_id,
            case_id,
            vec![
                MetricMeasurement::new(MetricKind::LatencyMs, latency_ms)
                    .expect("workspace latency"),
                MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("workspace accuracy"),
            ],
        )
        .expect("benchmark workspace result")
    }

    async fn benchmark_decision_fixture(context_id: &str) -> BenchmarkDecisionFixture {
        let project_id = ProjectId::from_uuid(
            uuid::Uuid::parse_str("22222222-2222-4222-8222-222222222222").expect("project id"),
        );
        let context_id =
            ContextId::from_uuid(uuid::Uuid::parse_str(context_id).expect("context id"));
        let commit_id = CommitId::from_uuid(
            uuid::Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("commit id"),
        );
        let decision_id = BenchmarkDecisionId::from_uuid(
            uuid::Uuid::parse_str("44444444-4444-4444-8444-444444444444").expect("decision id"),
        );
        let dataset = BenchmarkDataset::new(
            "Private support cases",
            vec![
                BenchmarkCase::new(
                    "Refund request",
                    serde_json::json!({"secret_input": "do not expose"}),
                    BenchmarkExpectedOutput::Exact(serde_json::json!({"secret_expected": true})),
                )
                .expect("benchmark case"),
            ],
        )
        .expect("benchmark dataset");
        let suite = BenchmarkSuite::new(
            "Private release gate",
            vec![dataset.id()],
            vec![
                RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                    .expect("accuracy threshold"),
                RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                    .expect("latency threshold"),
            ],
        )
        .expect("benchmark suite");
        let run = EvaluationRun::new(
            context_id,
            "model-a",
            0.2,
            vec![
                MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("accuracy metric"),
                MetricMeasurement::new(MetricKind::LatencyMs, 700.0).expect("latency metric"),
            ],
            Utc::now(),
        )
        .expect("evaluation run");
        let command = PersistBenchmarkEvaluationEvidence::new(
            decision_id,
            project_id,
            commit_id,
            vec![dataset.clone()],
            suite.clone(),
            vec![run.clone()],
            BenchmarkEvaluation::from_runs(&suite, &[run]),
            "contextlab.exact-match",
            "v1",
            Utc::now(),
        )
        .expect("benchmark evidence command");
        let repository = InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: project_id.to_string(),
                workspace_id: "55555555-5555-4555-8555-555555555555".to_owned(),
                name: "Benchmark project".to_owned(),
                slug: "benchmark-project".to_owned(),
                created_at: Utc::now(),
            }],
            contexts: vec![ContextRecord {
                id: context_id.to_string(),
                project_id: project_id.to_string(),
                experiment_id: None,
                name: "Benchmark context".to_owned(),
                description: None,
                created_at: Utc::now(),
            }],
            commits: vec![ContextCommitRecord {
                id: commit_id.to_string(),
                context_id: context_id.to_string(),
                branch_name: "main".to_owned(),
                message: "Benchmark commit".to_owned(),
                parent_commit_ids: Vec::new(),
                changes: serde_json::json!([]),
                change_count: 0,
                authored_at: Utc::now(),
                created_at: Utc::now(),
            }],
            ..ContextGraphProjection::default()
        });
        let evidence = repository
            .persist_benchmark_evaluation(command)
            .await
            .expect("persist benchmark evidence")
            .evidence()
            .clone();

        BenchmarkDecisionFixture {
            project_id,
            context_id,
            commit_id,
            decision_id,
            suite,
            datasets: vec![dataset],
            evidence,
            repository,
        }
    }

    fn benchmark_decision_request(
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
        token: Option<&str>,
    ) -> Request<Body> {
        let mut request = Request::builder().uri(format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}"
        ));
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        request
            .body(Body::empty())
            .expect("benchmark decision request")
    }

    fn benchmark_decision_list_request(
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
        token: Option<&str>,
    ) -> Request<Body> {
        let mut request = Request::builder().uri(format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions"
        ));
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        request
            .body(Body::empty())
            .expect("benchmark decision list request")
    }

    fn benchmark_decision_run_details_request(
        project_id: ProjectId,
        context_id: ContextId,
        commit_id: CommitId,
        decision_id: BenchmarkDecisionId,
        token: Option<&str>,
    ) -> Request<Body> {
        let mut request = Request::builder().uri(format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}/run-details"
        ));
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        request
            .body(Body::empty())
            .expect("benchmark decision run-details request")
    }

    fn benchmark_decision_diff_request(
        project_id: ProjectId,
        context_id: ContextId,
        baseline_commit_id: CommitId,
        baseline_decision_id: BenchmarkDecisionId,
        revised_commit_id: CommitId,
        revised_decision_id: BenchmarkDecisionId,
        token: Option<&str>,
    ) -> Request<Body> {
        let mut request = Request::builder().uri(format!(
            "/api/v1/local/projects/{project_id}/contexts/{context_id}/benchmark-decision-diffs?baseline_commit_id={baseline_commit_id}&baseline_decision_id={baseline_decision_id}&revised_commit_id={revised_commit_id}&revised_decision_id={revised_decision_id}"
        ));
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        request
            .body(Body::empty())
            .expect("benchmark decision diff request")
    }

    fn workflow_bindings_request(
        context_id: &str,
        commit_id: CommitId,
        token: Option<&str>,
    ) -> Request<Body> {
        let mut request = Request::builder().uri(format!(
            "/api/v1/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings"
        ));
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        request
            .body(Body::empty())
            .expect("workflow binding request")
    }

    fn workflow_binding(
        context_id: ContextId,
        commit_id: CommitId,
        binding_id: u128,
        workflow_id: u128,
        node_count: usize,
        edge_count: usize,
    ) -> WorkflowContextBinding {
        let revision = WorkflowRevision::new(1).expect("workflow revision");
        let nodes = (0..node_count)
            .map(|index| {
                WorkflowNode::new(
                    WorkflowNodeId::from_uuid(Uuid::from_u128(workflow_id + index as u128 + 100)),
                    revision,
                )
            })
            .collect::<Vec<_>>();
        let edges = (0..edge_count)
            .map(|index| {
                WorkflowEdge::new(
                    WorkflowEdgeId::from_uuid(Uuid::from_u128(workflow_id + index as u128 + 200)),
                    revision,
                    nodes[index].id(),
                    nodes[index + 1].id(),
                )
                .expect("workflow edge")
            })
            .collect::<Vec<_>>();
        let definition = WorkflowDefinition::new(
            WorkflowId::from_uuid(Uuid::from_u128(workflow_id)),
            revision,
            nodes,
            edges,
        )
        .expect("workflow definition");

        WorkflowContextBinding::new(
            WorkflowContextBindingId::from_uuid(Uuid::from_u128(binding_id)),
            definition,
            ContextCommitSource::new(context_id, commit_id),
        )
    }

    fn assert_json_excludes_keys_recursively(value: &Value, forbidden_keys: &[&str]) {
        match value {
            Value::Object(object) => {
                for (key, nested_value) in object {
                    assert!(
                        !forbidden_keys.contains(&key.as_str()),
                        "serialized response must not contain {key}"
                    );
                    assert_json_excludes_keys_recursively(nested_value, forbidden_keys);
                }
            }
            Value::Array(values) => {
                for nested_value in values {
                    assert_json_excludes_keys_recursively(nested_value, forbidden_keys);
                }
            }
            _ => {}
        }
    }

    async fn response_json(response: axum::response::Response) -> (StatusCode, Value) {
        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("json payload for {status}: {error}; body={:?}", body));
        (status, payload)
    }

    fn graph_with_context_label(label: &str) -> ContextGraph {
        graph_with_context_id_label("support-resolution-agent", label)
    }

    fn graph_with_context_id_label(context_id: &str, label: &str) -> ContextGraph {
        let mut graph = ContextGraph::new();
        graph
            .add_node(
                GraphNode::new(
                    format!("context:{context_id}"),
                    GraphNodeKind::Context,
                    label,
                )
                .expect("graph node"),
            )
            .expect("insert graph node");
        graph
    }

    fn timestamp(seconds: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 11, 0, 0, seconds)
            .single()
            .expect("timestamp")
    }

    #[test]
    fn public_get_routes_match_openapi_contract() {
        let openapi: Value = serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("checked-in OpenAPI contract must parse");
        let public_get_routes = public_get_route_contracts();

        assert_eq!(public_get_routes.len(), PUBLIC_GET_ROUTES.len());
        assert_eq!(openapi_get_route_contracts(&openapi), public_get_routes);
        assert!(
            !public_get_routes.contains_key("/api/v1/contexts/{context_id}/graph-diff"),
            "retired graph diff route must not return to the public catalog"
        );
        assert!(
            openapi["paths"]["/api/v1/contexts/{context_id}/graph-diff"].is_null(),
            "retired graph diff route must not return to public OpenAPI"
        );
        assert!(
            openapi["paths"]["/api/v1/local/contexts/{context_id}/graph-diff"].is_null(),
            "protected-local graph diff route must stay outside public OpenAPI"
        );
        assert!(
            openapi["paths"]["/api/v1/local/projects/{project_id}/contexts/{context_id}/diff-review"]
                .is_null(),
            "private persisted Context diff review must stay outside public OpenAPI"
        );
    }

    #[test]
    fn public_post_routes_match_openapi_contract() {
        let openapi: Value = serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("checked-in OpenAPI contract must parse");
        let public_post_routes = public_post_route_contracts();

        assert_eq!(public_post_routes.len(), PUBLIC_POST_ROUTES.len());
        assert_eq!(openapi_post_route_contracts(&openapi), public_post_routes);
    }

    fn public_get_route_contracts() -> BTreeMap<String, RouteContract> {
        PUBLIC_GET_ROUTES
            .iter()
            .map(|route| {
                (
                    route.path().to_owned(),
                    RouteContract {
                        operation_id: route.operation_id().to_owned(),
                        path_parameters: sorted_names(route.path_parameters()),
                        query_parameters: sorted_names(route.query_parameters()),
                    },
                )
            })
            .collect()
    }

    fn openapi_get_route_contracts(openapi: &Value) -> BTreeMap<String, RouteContract> {
        let paths = openapi
            .get("paths")
            .and_then(Value::as_object)
            .expect("OpenAPI paths object");

        paths
            .iter()
            .filter_map(|(path, path_item)| {
                path_item.get("get").map(|operation| {
                    (
                        path.clone(),
                        RouteContract {
                            operation_id: operation
                                .get("operationId")
                                .and_then(Value::as_str)
                                .expect("GET operationId")
                                .to_owned(),
                            path_parameters: openapi_parameter_names(openapi, operation, "path"),
                            query_parameters: openapi_parameter_names(openapi, operation, "query"),
                        },
                    )
                })
            })
            .collect()
    }

    fn public_post_route_contracts() -> BTreeMap<String, RouteContract> {
        PUBLIC_POST_ROUTES
            .iter()
            .map(|route| {
                (
                    route.path().to_owned(),
                    RouteContract {
                        operation_id: route.operation_id().to_owned(),
                        path_parameters: sorted_names(route.path_parameters()),
                        query_parameters: sorted_names(route.query_parameters()),
                    },
                )
            })
            .collect()
    }

    fn openapi_post_route_contracts(openapi: &Value) -> BTreeMap<String, RouteContract> {
        let paths = openapi
            .get("paths")
            .and_then(Value::as_object)
            .expect("OpenAPI paths object");

        paths
            .iter()
            .filter_map(|(path, path_item)| {
                path_item.get("post").map(|operation| {
                    (
                        path.clone(),
                        RouteContract {
                            operation_id: operation
                                .get("operationId")
                                .and_then(Value::as_str)
                                .expect("POST operationId")
                                .to_owned(),
                            path_parameters: openapi_parameter_names(openapi, operation, "path"),
                            query_parameters: openapi_parameter_names(openapi, operation, "query"),
                        },
                    )
                })
            })
            .collect()
    }

    fn openapi_parameter_names(openapi: &Value, operation: &Value, location: &str) -> Vec<String> {
        let mut names = operation
            .get("parameters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(|parameter| resolve_openapi_parameter(openapi, parameter))
            .filter(|parameter| parameter.get("in").and_then(Value::as_str) == Some(location))
            .map(|parameter| {
                parameter
                    .get("name")
                    .and_then(Value::as_str)
                    .expect("OpenAPI parameter name")
                    .to_owned()
            })
            .collect::<Vec<_>>();

        names.sort();
        names
    }

    fn resolve_openapi_parameter<'a>(openapi: &'a Value, parameter: &'a Value) -> &'a Value {
        let Some(reference) = parameter.get("$ref").and_then(Value::as_str) else {
            return parameter;
        };

        let parameter_name = reference
            .strip_prefix("#/components/parameters/")
            .expect("local OpenAPI parameter reference");

        openapi
            .get("components")
            .and_then(|components| components.get("parameters"))
            .and_then(|parameters| parameters.get(parameter_name))
            .unwrap_or_else(|| panic!("missing OpenAPI parameter reference {reference}"))
    }

    fn sorted_names(names: &[&str]) -> Vec<String> {
        let mut names = names.iter().map(ToString::to_string).collect::<Vec<_>>();
        names.sort();
        names
    }

    #[tokio::test]
    async fn health_route_reports_ok() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .expect("content-type")
                .to_str()
                .expect("content-type string")
                .starts_with("application/json")
        );

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["status"], "ok");
    }

    #[tokio::test]
    async fn meta_route_lists_core_capabilities() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/meta")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["product"], "ContextLab");
        assert_eq!(payload["primary_abstraction"], "context");
        assert!(
            payload["capabilities"]
                .as_array()
                .expect("capabilities")
                .contains(&Value::String("versioning".to_owned()))
        );
    }

    #[tokio::test]
    async fn openapi_route_returns_checked_in_contract() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/openapi.json")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert!(
            payload["openapi"]
                .as_str()
                .expect("openapi version")
                .starts_with("3.")
        );
        assert!(payload["paths"]["/api/v1/workspaces"]["get"].is_object());
        assert_eq!(
            payload["paths"]["/api/v1/openapi.json"]["get"]["operationId"],
            "getOpenApiDocument"
        );
    }

    #[tokio::test]
    async fn providers_route_reports_configured_providers_without_secrets() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/providers")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");
        let providers = payload["providers"].as_array().expect("providers");

        assert_eq!(providers.len(), 2);
        assert!(
            providers
                .iter()
                .all(|provider| provider["configured"] == true)
        );
        assert!(providers.iter().any(|provider| provider["id"] == "deepseek"
            && provider["api_key_fingerprint"] == "sk-...-key"));
        assert!(
            !serde_json::to_string(&payload)
                .expect("json")
                .contains("deepseek-test")
        );
    }

    #[tokio::test]
    async fn workspaces_route_lists_default_workspace() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "default");
        assert_eq!(payload["items"][0]["slug"], "default");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["pagination"]["page"], 1);
        assert_eq!(payload["pagination"]["per_page"], 20);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn workspaces_route_filters_and_paginates() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces?search=default&page=2&per_page=1&sort=-name")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"].as_array().expect("items").len(), 0);
        assert_eq!(payload["pagination"]["page"], 2);
        assert_eq!(payload["pagination"]["per_page"], 1);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn workspaces_route_accepts_created_at_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces?sort=created_at")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "default");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn workspaces_route_rejects_invalid_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces?sort=drop-table")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_workspace_sort");
    }

    #[tokio::test]
    async fn workspace_projects_route_lists_default_workspace_projects() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/default/projects")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "support-ai");
        assert_eq!(payload["items"][0]["workspace_id"], "default");
        assert_eq!(payload["items"][0]["slug"], "support-ai");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["pagination"]["page"], 1);
        assert_eq!(payload["pagination"]["per_page"], 20);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn workspace_projects_route_filters_and_paginates() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/workspaces/default/projects?search=support&page=2&per_page=1&sort=-name",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"].as_array().expect("items").len(), 0);
        assert_eq!(payload["pagination"]["page"], 2);
        assert_eq!(payload["pagination"]["per_page"], 1);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn workspace_projects_route_accepts_created_at_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/default/projects?sort=created_at")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "support-ai");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn workspace_projects_route_rejects_invalid_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/default/projects?sort=drop-table")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_project_sort");
    }

    #[tokio::test]
    async fn workspace_projects_route_reports_unknown_workspace() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/missing/projects")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn project_experiments_route_lists_default_project_experiments() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/experiments")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "rag-v2");
        assert_eq!(payload["items"][0]["project_id"], "support-ai");
        assert_eq!(payload["items"][0]["branch_name"], "experiment/rag-v2");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["pagination"]["page"], 1);
        assert_eq!(payload["pagination"]["per_page"], 20);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn project_experiments_route_filters_and_paginates() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/projects/support-ai/experiments?search=rag&page=2&per_page=1&sort=-name",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"].as_array().expect("items").len(), 0);
        assert_eq!(payload["pagination"]["page"], 2);
        assert_eq!(payload["pagination"]["per_page"], 1);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn project_experiments_route_accepts_created_at_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/experiments?sort=created_at")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "rag-v2");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn project_experiments_route_accepts_branch_name_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/experiments?sort=branch_name")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["branch_name"], "experiment/rag-v2");
    }

    #[tokio::test]
    async fn project_experiments_route_rejects_invalid_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/experiments?sort=drop-table")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_experiment_sort");
    }

    #[tokio::test]
    async fn project_experiments_route_reports_unknown_project() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/missing/experiments")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn project_contexts_route_lists_default_project_contexts() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/contexts")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "support-resolution-agent");
        assert_eq!(payload["items"][0]["project_id"], "support-ai");
        assert_eq!(payload["items"][0]["experiment_id"], "rag-v2");
        assert_eq!(payload["items"][0]["name"], "Support Resolution Agent");
        assert_eq!(
            payload["items"][0]["description"],
            "Production support context for resolving customer cases."
        );
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["pagination"]["page"], 1);
        assert_eq!(payload["pagination"]["per_page"], 20);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn project_contexts_route_filters_and_paginates() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/projects/support-ai/contexts?search=customer&page=2&per_page=1&sort=-name",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"].as_array().expect("items").len(), 0);
        assert_eq!(payload["pagination"]["page"], 2);
        assert_eq!(payload["pagination"]["per_page"], 1);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn project_contexts_route_filters_by_experiment_id() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/contexts?experiment_id=rag-v2")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "support-resolution-agent");
        assert_eq!(payload["pagination"]["total"], 1);

        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/contexts?experiment_id=other")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert!(payload["items"].as_array().expect("items").is_empty());
        assert_eq!(payload["pagination"]["total"], 0);
    }

    #[tokio::test]
    async fn project_contexts_route_accepts_created_at_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/contexts?sort=created_at")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "support-resolution-agent");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn project_contexts_route_rejects_invalid_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/support-ai/contexts?sort=drop-table")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_context_sort");
    }

    #[tokio::test]
    async fn project_contexts_route_reports_unknown_project() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/missing/contexts")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn context_commits_route_lists_default_context_commits() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/commits")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(
            payload["items"][0]["id"],
            "support-resolution-agent-initial"
        );
        assert_eq!(
            payload["items"][0]["context_id"],
            "support-resolution-agent"
        );
        assert_eq!(payload["items"][0]["branch_name"], "main");
        assert_eq!(
            payload["items"][0]["message"],
            "Create support resolution context"
        );
        assert!(
            payload["items"][0]["parent_commit_ids"]
                .as_array()
                .expect("parent ids")
                .is_empty()
        );
        assert_eq!(payload["items"][0]["change_count"], 1);
        assert_eq!(payload["items"][0]["authored_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["pagination"]["page"], 1);
        assert_eq!(payload["pagination"]["per_page"], 20);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn context_commits_route_filters_and_paginates() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/commits?search=support&page=2&per_page=1&sort=branch_name",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"].as_array().expect("items").len(), 0);
        assert_eq!(payload["pagination"]["page"], 2);
        assert_eq!(payload["pagination"]["per_page"], 1);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn context_commits_route_filters_by_branch_name() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/commits?branch_name=main")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(
            payload["items"][0]["id"],
            "support-resolution-agent-initial"
        );
        assert_eq!(payload["pagination"]["total"], 1);

        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/commits?branch_name=experiment/rag-v2",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert!(payload["items"].as_array().expect("items").is_empty());
        assert_eq!(payload["pagination"]["total"], 0);
    }

    #[tokio::test]
    async fn context_commits_route_accepts_authored_at_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/commits?sort=authored_at")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(
            payload["items"][0]["id"],
            "support-resolution-agent-initial"
        );
        assert_eq!(payload["items"][0]["authored_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn context_commits_route_rejects_invalid_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/commits?sort=message")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_commit_sort");
    }

    #[tokio::test]
    async fn context_commits_route_reports_unknown_context() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/missing/commits")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn context_commit_route_returns_commit_detail() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/commits/support-resolution-agent-initial")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["id"], "support-resolution-agent-initial");
        assert_eq!(payload["context_id"], "support-resolution-agent");
        assert_eq!(payload["branch_name"], "main");
        assert_eq!(payload["message"], "Create support resolution context");
        assert_eq!(payload["change_count"], 1);
        assert_eq!(payload["changes"].as_array().expect("changes").len(), 1);
        assert_eq!(payload["authored_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["created_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn context_commit_route_reports_unknown_commit() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/commits/missing")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn context_components_route_lists_default_context_components() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/components")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "refund-policy");
        assert_eq!(
            payload["items"][0]["context_id"],
            "support-resolution-agent"
        );
        assert_eq!(payload["items"][0]["kind"], "knowledge");
        assert_eq!(payload["items"][0]["name"], "Refund Policy Knowledge");
        assert_eq!(
            payload["items"][0]["content_hash"],
            "sha256:preview-refund-policy"
        );
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["pagination"]["page"], 1);
        assert_eq!(payload["pagination"]["per_page"], 20);
        assert_eq!(payload["pagination"]["total"], 5);
    }

    #[tokio::test]
    async fn context_components_route_filters_by_kind_and_paginates() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/components?kind=memory&search=timeline&page=1&per_page=1&sort=name",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "timeline");
        assert_eq!(payload["items"][0]["kind"], "memory");
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn context_components_route_rejects_invalid_kind() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/components?kind=artifact")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_component_kind");
    }

    #[tokio::test]
    async fn context_components_route_rejects_invalid_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/components?sort=content_hash")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_component_sort");
    }

    #[tokio::test]
    async fn context_components_route_reports_unknown_context() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/missing/components")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn context_component_route_returns_component_detail() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/components/refund-policy")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["id"], "refund-policy");
        assert_eq!(payload["context_id"], "support-resolution-agent");
        assert_eq!(payload["kind"], "knowledge");
        assert_eq!(payload["name"], "Refund Policy Knowledge");
        assert_eq!(payload["content_hash"], "sha256:preview-refund-policy");
        assert_eq!(payload["metadata"]["source"], "policy-handbook");
        assert_eq!(payload["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["updated_at"], "2026-07-09T00:00:00Z");
        assert!(
            payload.get("content").is_none(),
            "component body content is not persisted yet"
        );
    }

    #[tokio::test]
    async fn context_component_route_reports_unknown_component() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/components/missing")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn context_evaluation_runs_route_lists_default_context_runs() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/evaluation-runs")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "safety-regression");
        assert_eq!(
            payload["items"][0]["context_id"],
            "support-resolution-agent"
        );
        assert_eq!(payload["items"][0]["suite_name"], "Safety Regression Suite");
        assert_eq!(payload["items"][0]["model_version"], "deepseek-chat");
        assert_eq!(payload["items"][0]["temperature"], 0.2);
        assert_eq!(payload["items"][0]["metric_count"], 2);
        assert_eq!(payload["items"][0]["executed_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["items"][0]["created_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["pagination"]["page"], 1);
        assert_eq!(payload["pagination"]["per_page"], 20);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn context_evaluation_run_route_returns_evaluation_run_detail() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/evaluation-runs/safety-regression")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["id"], "safety-regression");
        assert_eq!(payload["context_id"], "support-resolution-agent");
        assert_eq!(payload["suite_name"], "Safety Regression Suite");
        assert_eq!(payload["model_version"], "deepseek-chat");
        assert_eq!(payload["temperature"], 0.2);
        assert_eq!(payload["metric_count"], 2);
        assert_eq!(payload["metrics"]["accuracy"], 0.92);
        assert_eq!(payload["metrics"]["latency_ms"], 820);
        assert_eq!(payload["executed_at"], "2026-07-09T00:00:00Z");
        assert_eq!(payload["created_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn context_evaluation_scorecard_route_returns_metric_averages() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/evaluation-scorecard")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["context_id"], "support-resolution-agent");
        assert_eq!(payload["run_count"], 1);
        assert_eq!(payload["metrics"][0]["name"], "accuracy");
        assert_eq!(payload["metrics"][0]["average"], 0.92);
        assert_eq!(payload["metrics"][0]["sample_count"], 1);
        assert_eq!(payload["metrics"][1]["name"], "latency_ms");
        assert_eq!(payload["metrics"][1]["average"], 820.0);
        assert_eq!(payload["metrics"][1]["sample_count"], 1);
    }

    #[tokio::test]
    async fn context_evaluation_scorecard_route_filters_by_suite_and_model() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/evaluation-scorecard?suite_name=Safety%20Regression%20Suite&model_version=other-model",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["context_id"], "support-resolution-agent");
        assert_eq!(payload["run_count"], 0);
        assert!(payload["metrics"].as_array().expect("metrics").is_empty());
    }

    #[tokio::test]
    async fn context_evaluation_run_route_reports_unknown_run() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/evaluation-runs/missing")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn context_evaluation_runs_route_filters_and_paginates() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/evaluation-runs?search=safety&page=2&per_page=1&sort=model_version",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"].as_array().expect("items").len(), 0);
        assert_eq!(payload["pagination"]["page"], 2);
        assert_eq!(payload["pagination"]["per_page"], 1);
        assert_eq!(payload["pagination"]["total"], 1);
    }

    #[tokio::test]
    async fn context_evaluation_runs_route_filters_by_suite_and_model() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/evaluation-runs?suite_name=Safety%20Regression%20Suite&model_version=deepseek-chat",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "safety-regression");
        assert_eq!(payload["pagination"]["total"], 1);

        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/evaluation-runs?suite_name=Safety%20Regression%20Suite&model_version=other-model",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert!(payload["items"].as_array().expect("items").is_empty());
        assert_eq!(payload["pagination"]["total"], 0);
    }

    #[tokio::test]
    async fn context_evaluation_runs_route_accepts_executed_at_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/support-resolution-agent/evaluation-runs?sort=executed_at",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["items"][0]["id"], "safety-regression");
        assert_eq!(payload["items"][0]["executed_at"], "2026-07-09T00:00:00Z");
    }

    #[tokio::test]
    async fn context_evaluation_runs_route_rejects_invalid_sort() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/support-resolution-agent/evaluation-runs?sort=metrics")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "invalid_evaluation_run_sort");
    }

    #[tokio::test]
    async fn context_evaluation_runs_route_reports_unknown_context() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/missing/evaluation-runs")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn context_graph_preview_route_returns_graph() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/context-graph/preview")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");
        let nodes = payload["graph"]["nodes"].as_object().expect("nodes object");
        let edges = payload["graph"]["edges"].as_array().expect("edges");

        assert!(nodes.values().any(|node| node["kind"] == "context"));
        assert!(nodes.values().any(|node| node["kind"] == "evaluation"));
        assert!(!edges.is_empty());
    }

    #[tokio::test]
    async fn graph_diffs_route_compares_validated_graph_snapshots() {
        let payload = serde_json::json!({
            "original": {
                "nodes": [
                    { "id": "context:agent", "kind": "context", "label": "Support Agent" },
                    { "id": "prompt:system", "kind": "prompt", "label": "System Prompt" }
                ],
                "edges": [
                    { "source": "context:agent", "target": "prompt:system", "kind": "contains" }
                ]
            },
            "revised": {
                "nodes": [
                    { "id": "context:agent", "kind": "context", "label": "Support Agent v2" },
                    { "id": "knowledge:policy", "kind": "knowledge", "label": "Refund Policy" }
                ],
                "edges": [
                    { "source": "context:agent", "target": "knowledge:policy", "kind": "retrieves" }
                ]
            }
        });
        let response = test_router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/graph-diffs")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let diff: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(diff["added_nodes"][0]["id"], "knowledge:policy");
        assert_eq!(diff["removed_nodes"][0]["id"], "prompt:system");
        assert_eq!(diff["modified_nodes"][0]["node_id"], "context:agent");
        assert_eq!(diff["modified_nodes"][0]["original_label"], "Support Agent");
        assert_eq!(
            diff["modified_nodes"][0]["revised_label"],
            "Support Agent v2"
        );
        assert_eq!(diff["added_edges"][0]["kind"], "retrieves");
        assert_eq!(diff["removed_edges"][0]["kind"], "contains");
    }

    #[tokio::test]
    async fn version_backed_graph_diff_is_absent_from_the_public_catalog() {
        let (state, _) = version_backed_graph_diff_state(
            version_backed_graph_diff_snapshot_repository(),
            AllowContextWrites,
        );
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/contexts/{VERSION_BACKED_CONTEXT_ID}/graph-diff?original_commit_id={VERSION_BACKED_ORIGINAL_COMMIT_ID}&revised_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}"))
                    .body(Body::empty())
                    .expect("public graph diff request"),
            )
            .await
            .expect("public graph diff response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn private_context_diff_review_returns_the_exact_persisted_projection() {
        let graph_repository = version_backed_graph_diff_snapshot_repository();
        let diff_repository = persisted_context_diff_snapshot_repository().await;
        let (state, token) = version_backed_graph_diff_state(graph_repository, AllowContextWrites);
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Allowed));
        let response = build_protected_router_with_state(
            state
                .with_context_diff_snapshot_repository(diff_repository)
                .with_protected_rate_limiter(limiter.clone()),
        )
        .oneshot(context_diff_review_request(
            Some(&token),
            &format!(
                "source_commit_id={VERSION_BACKED_ORIGINAL_COMMIT_ID}&target_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}"
            ),
        ))
        .await
        .expect("context diff review response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let (_, payload) = response_json(response).await;
        assert_eq!(payload["schema_version"], "v1");
        assert_eq!(
            payload["source_scope"]["project_id"],
            VERSION_BACKED_PROJECT_ID
        );
        assert_eq!(
            payload["source_scope"]["context_id"],
            VERSION_BACKED_CONTEXT_ID
        );
        assert_eq!(
            payload["source_scope"]["commit_id"],
            VERSION_BACKED_ORIGINAL_COMMIT_ID
        );
        assert_eq!(
            payload["target_scope"]["commit_id"],
            VERSION_BACKED_REVISED_COMMIT_ID
        );
        assert!(payload["diff"].is_object());
        assert!(
            !payload["diff"]["semantic"]["graph_diff"]["modified_nodes"]
                .as_array()
                .expect("semantic graph changes")
                .is_empty()
        );
        assert!(
            !payload["diff"]["behavior"]["case_changes"]
                .as_array()
                .expect("behavior case changes")
                .is_empty()
        );
        assert!(
            !payload["diff"]["evaluation"]["metric_changes"]
                .as_array()
                .expect("evaluation metric changes")
                .is_empty()
        );
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::ContextCommitGraphDiffRead]
        );
    }

    #[tokio::test]
    async fn private_context_diff_review_requires_exact_query_fields_and_rejects_unknown_fields() {
        let graph_repository = version_backed_graph_diff_snapshot_repository();
        let diff_repository = persisted_context_diff_snapshot_repository().await;
        let (state, token) = version_backed_graph_diff_state(graph_repository, AllowContextWrites);
        let state = state.with_context_diff_snapshot_repository(diff_repository);

        let missing = build_protected_router_with_state(state.clone())
            .oneshot(context_diff_review_request(
                Some(&token),
                "target_commit_id=missing",
            ))
            .await
            .expect("missing query response");
        assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            missing.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );

        let unknown = build_protected_router_with_state(state)
            .oneshot(context_diff_review_request(
                Some(&token),
                &format!(
                    "source_commit_id={VERSION_BACKED_ORIGINAL_COMMIT_ID}&target_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&unexpected=true"
                ),
            ))
            .await
            .expect("unknown query response");
        assert_eq!(unknown.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            unknown.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
    }

    #[tokio::test]
    async fn private_context_diff_review_is_absent_from_the_public_router() {
        let state = AppState::new(ProviderRegistry::from_env(
            std::iter::empty::<(&str, &str)>(),
        ));
        let response = build_router_with_state(state)
            .oneshot(context_diff_review_request(
                None,
                "source_commit_id=1&target_commit_id=2",
            ))
            .await
            .expect("public context diff review response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn protected_graph_diff_authenticates_before_quota_or_snapshot_access() {
        let calls = Arc::new(AtomicUsize::new(0));
        let snapshot_repository = RecordingCommitGraphSnapshotRepository {
            inner: version_backed_graph_diff_snapshot_repository(),
            calls: calls.clone(),
        };
        let (state, _) = version_backed_graph_diff_state(snapshot_repository, AllowContextWrites);
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 9,
        }));
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state
            .with_protected_rate_limiter(limiter.clone())
            .with_authorization_audit_sink(RecordingAuditSink {
                events: events.clone(),
            });

        let response = build_protected_router_with_state(state)
            .oneshot(version_backed_graph_diff_request(
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_REVISED_COMMIT_ID,
                None,
            ))
            .await
            .expect("protected graph diff response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        assert_eq!(limiter.calls(), 0);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(events.lock().expect("audit events lock").is_empty());
    }

    #[tokio::test]
    async fn protected_graph_diff_audits_a_forbidden_read_before_snapshot_access() {
        let calls = Arc::new(AtomicUsize::new(0));
        let snapshot_repository = RecordingCommitGraphSnapshotRepository {
            inner: version_backed_graph_diff_snapshot_repository(),
            calls: calls.clone(),
        };
        let (state, token) = version_backed_graph_diff_state(snapshot_repository, DenyContextReads);
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state.with_authorization_audit_sink(RecordingAuditSink {
            events: events.clone(),
        });

        let response = build_protected_router_with_state(state)
            .oneshot(version_backed_graph_diff_request(
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_REVISED_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("forbidden graph diff response");
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(payload["error"], "context_read_forbidden");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let events = events.lock().expect("audit events lock");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].decision(), AuthorizationDecision::Forbidden);
    }

    #[tokio::test]
    async fn protected_graph_diff_preserves_exact_scope_and_uses_its_own_quota() {
        let calls = Arc::new(AtomicUsize::new(0));
        let snapshot_repository = RecordingCommitGraphSnapshotRepository {
            inner: version_backed_graph_diff_snapshot_repository(),
            calls: calls.clone(),
        };
        let (state, token) =
            version_backed_graph_diff_state(snapshot_repository, AllowContextWrites);
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Allowed));
        let response =
            build_protected_router_with_state(state.with_protected_rate_limiter(limiter.clone()))
                .oneshot(version_backed_graph_diff_request(
                    VERSION_BACKED_CONTEXT_ID,
                    VERSION_BACKED_ORIGINAL_COMMIT_ID,
                    VERSION_BACKED_REVISED_COMMIT_ID,
                    Some(&token),
                ))
                .await
                .expect("allowed graph diff response");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let (_, payload) = response_json(response).await;

        assert_eq!(payload["context_id"], VERSION_BACKED_CONTEXT_ID);
        assert_eq!(
            payload["original"]["commit_id"],
            VERSION_BACKED_ORIGINAL_COMMIT_ID
        );
        assert_eq!(
            payload["revised"]["commit_id"],
            VERSION_BACKED_REVISED_COMMIT_ID
        );
        assert_eq!(
            payload["diff"]["modified_nodes"][0]["node_id"],
            format!("context:{VERSION_BACKED_CONTEXT_ID}")
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::ContextCommitGraphDiffRead]
        );
    }

    #[tokio::test]
    async fn protected_graph_diff_rejects_resolver_scope_drift_before_review() {
        let inner = version_backed_graph_diff_snapshot_repository();
        let drifted_repository = DriftedCommitGraphSnapshotRepository {
            inner,
            returned_scope: CommitGraphSnapshotScope::new(
                ProjectId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_PROJECT_ID).expect("project id"),
                ),
                ContextId::from_uuid(
                    Uuid::parse_str(VERSION_BACKED_CONTEXT_ID).expect("context id"),
                ),
                CommitId::from_uuid(Uuid::from_u128(900)),
            ),
        };
        let (state, token) =
            version_backed_graph_diff_state(drifted_repository, AllowContextWrites);

        let response = build_protected_router_with_state(state)
            .oneshot(version_backed_graph_diff_request(
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_REVISED_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("drifted graph diff response");
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(payload["error"], "commit_graph_diff_unavailable");
        assert_eq!(
            payload["message"],
            "version-backed Context Graph diff review is unavailable"
        );
    }

    #[tokio::test]
    async fn protected_graph_diff_rejects_identical_commit_scopes_before_snapshot_access() {
        let calls = Arc::new(AtomicUsize::new(0));
        let snapshot_repository = RecordingCommitGraphSnapshotRepository {
            inner: version_backed_graph_diff_snapshot_repository(),
            calls: calls.clone(),
        };
        let (state, token) =
            version_backed_graph_diff_state(snapshot_repository, AllowContextWrites);

        let response = build_protected_router_with_state(state)
            .oneshot(version_backed_graph_diff_request(
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("identical graph diff response");
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let (status, payload) = response_json(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_commit_graph_diff_query");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn protected_persisted_context_diff_review_delegates_and_preserves_exact_scope() {
        let repository = persisted_context_diff_review_repository().await;
        let (state, token) = version_backed_graph_diff_state(
            version_backed_graph_diff_snapshot_repository(),
            AllowContextWrites,
        );
        let state = state.with_context_diff_snapshot_repository(repository);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_diff_review_request(
                VERSION_BACKED_PROJECT_ID,
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_REVISED_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("persisted Context diff review response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let (_, payload) = response_json(response).await;
        assert_eq!(
            payload["source_scope"]["project_id"],
            VERSION_BACKED_PROJECT_ID
        );
        assert_eq!(
            payload["source_scope"]["context_id"],
            VERSION_BACKED_CONTEXT_ID
        );
        assert_eq!(
            payload["source_scope"]["commit_id"],
            VERSION_BACKED_ORIGINAL_COMMIT_ID
        );
        assert_eq!(
            payload["target_scope"]["commit_id"],
            VERSION_BACKED_REVISED_COMMIT_ID
        );
        assert_eq!(payload["schema_version"], "v1");
        assert!(
            !payload["diff"]["semantic"]["document_changes"]
                .as_array()
                .expect("semantic document changes")
                .is_empty()
        );
        assert!(
            !payload["diff"]["behavior"]["case_changes"]
                .as_array()
                .expect("behavior case changes")
                .is_empty()
        );
        assert!(
            !payload["diff"]["evaluation"]["metric_changes"]
                .as_array()
                .expect("evaluation metric changes")
                .is_empty()
        );
    }

    fn server_owned_merge_review_state(
        include_right_snapshot: bool,
    ) -> (AppState, String, Arc<AtomicUsize>) {
        server_owned_merge_review_state_with_witness(include_right_snapshot, true)
    }

    fn server_owned_merge_review_state_with_witness(
        include_right_snapshot: bool,
        include_witness: bool,
    ) -> (AppState, String, Arc<AtomicUsize>) {
        let project_id = ProjectId::from_uuid(
            Uuid::parse_str(VERSION_BACKED_PROJECT_ID).expect("merge review project id"),
        );
        let context_id = ContextId::from_uuid(
            Uuid::parse_str(VERSION_BACKED_CONTEXT_ID).expect("merge review context id"),
        );
        let base_commit_id = CommitId::from_uuid(
            Uuid::parse_str(VERSION_BACKED_ORIGINAL_COMMIT_ID).expect("merge review base id"),
        );
        let left_commit_id = CommitId::from_uuid(
            Uuid::parse_str(VERSION_BACKED_REVISED_COMMIT_ID).expect("merge review left id"),
        );
        let right_commit_id = CommitId::from_uuid(
            Uuid::parse_str(MERGE_REVIEW_RIGHT_COMMIT_ID).expect("merge review right id"),
        );
        let commit = |id: CommitId, parents: Vec<CommitId>| ContextCommitRecord {
            id: id.to_string(),
            context_id: context_id.to_string(),
            branch_name: "merge-review".to_owned(),
            message: "merge review fixture".to_owned(),
            parent_commit_ids: parents
                .into_iter()
                .map(|parent| parent.to_string())
                .collect(),
            changes: serde_json::json!([]),
            change_count: 0,
            authored_at: timestamp(1),
            created_at: timestamp(1),
        };
        let projection = ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: project_id.to_string(),
                workspace_id: "33333333-3333-4333-8333-333333333333".to_owned(),
                name: "Merge review project".to_owned(),
                slug: "merge-review".to_owned(),
                created_at: timestamp(1),
            }],
            contexts: vec![ContextRecord {
                id: context_id.to_string(),
                project_id: project_id.to_string(),
                experiment_id: None,
                name: "Merge review context".to_owned(),
                description: None,
                created_at: timestamp(1),
            }],
            commits: vec![
                commit(base_commit_id, Vec::new()),
                commit(left_commit_id, vec![base_commit_id]),
                commit(right_commit_id, vec![base_commit_id]),
            ],
            ..ContextGraphProjection::default()
        };
        let snapshots = [
            CommitGraphSnapshot::new(
                CommitGraphSnapshotScope::new(project_id, context_id, base_commit_id),
                graph_with_context_label("base"),
                timestamp(1),
                COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
            )
            .expect("base snapshot"),
            CommitGraphSnapshot::new(
                CommitGraphSnapshotScope::new(project_id, context_id, left_commit_id),
                graph_with_context_label("left"),
                timestamp(1),
                COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
            )
            .expect("left snapshot"),
        ]
        .into_iter()
        .chain(include_right_snapshot.then(|| {
            CommitGraphSnapshot::new(
                CommitGraphSnapshotScope::new(project_id, context_id, right_commit_id),
                graph_with_context_label("right"),
                timestamp(1),
                COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
            )
            .expect("right snapshot")
        }))
        .collect::<Vec<_>>();
        let repository =
            InMemoryContextGraphRepository::with_commit_graph_snapshots(projection, snapshots)
                .expect("merge review repository");
        let graph_reads = Arc::new(AtomicUsize::new(0));
        let graph_repository = RecordingContextCommitGraphRepository {
            inner: repository.clone(),
            calls: graph_reads.clone(),
        };
        let merge_witness_repository = graph_repository.clone();
        let graph_repositories = WorkspaceGraphRepositories::new(
            repository.clone(),
            repository.clone(),
            repository.clone(),
        );
        let graph_repositories = if include_witness {
            graph_repositories
                .with_context_merge_review_witness_repository(merge_witness_repository)
        } else {
            graph_repositories
        };
        let state = AppState::with_workspace_repositories(
            ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>()),
            WorkspaceRepositories::new(
                graph_repositories,
                WorkspaceCatalogRepositories::new(
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    repository.clone(),
                    graph_repository,
                    repository.clone(),
                    repository.clone(),
                ),
            ),
        )
        .with_protected_write_dependencies(
            AllowContextWrites,
            repository,
            contextlab_auth::HmacJwtAuthenticator::new(
                "test-secret",
                TEST_AUTH_ISSUER,
                TEST_AUTH_AUDIENCE,
            )
            .expect("authenticator"),
            test_rate_limiter(),
        );
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestJwtClaims {
                sub: "user:alex".to_owned(),
                exp: (Utc::now().timestamp() + 300) as usize,
                iss: TEST_AUTH_ISSUER.to_owned(),
                aud: TEST_AUTH_AUDIENCE.to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");
        (state, token, graph_reads)
    }

    fn persisted_context_merge_review_request(query: &str, token: Option<&str>) -> Request<Body> {
        let mut request = Request::builder().uri(format!(
            "/api/v1/local/projects/{VERSION_BACKED_PROJECT_ID}/contexts/{VERSION_BACKED_CONTEXT_ID}/merge-review?{query}"
        ));
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        request
            .body(Body::empty())
            .expect("Context merge review request")
    }

    #[tokio::test]
    async fn protected_persisted_context_merge_review_returns_server_owned_v1_projection() {
        let (state, token, _) = server_owned_merge_review_state(true);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id={MERGE_REVIEW_RIGHT_COMMIT_ID}"
                ),
                Some(&token),
            ))
            .await
            .expect("Context merge review response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let (_, payload) = response_json(response).await;
        assert_eq!(payload["schema_version"], "v1");
        assert_eq!(
            payload["plan"]["ThreeWay"]["base"],
            VERSION_BACKED_ORIGINAL_COMMIT_ID
        );
        assert_eq!(
            payload["plan"]["ThreeWay"]["left"],
            VERSION_BACKED_REVISED_COMMIT_ID
        );
        assert_eq!(
            payload["plan"]["ThreeWay"]["right"],
            MERGE_REVIEW_RIGHT_COMMIT_ID
        );
        assert_eq!(
            payload["base_scope"]["project_id"],
            VERSION_BACKED_PROJECT_ID
        );
        assert_eq!(
            payload["base_scope"]["context_id"],
            VERSION_BACKED_CONTEXT_ID
        );
        assert_eq!(
            payload["base_scope"]["commit_id"],
            VERSION_BACKED_ORIGINAL_COMMIT_ID
        );
        assert_eq!(
            payload["left_scope"]["project_id"],
            VERSION_BACKED_PROJECT_ID
        );
        assert_eq!(
            payload["left_scope"]["context_id"],
            VERSION_BACKED_CONTEXT_ID
        );
        assert_eq!(
            payload["left_scope"]["commit_id"],
            VERSION_BACKED_REVISED_COMMIT_ID
        );
        assert_eq!(
            payload["right_scope"]["project_id"],
            VERSION_BACKED_PROJECT_ID
        );
        assert_eq!(
            payload["right_scope"]["context_id"],
            VERSION_BACKED_CONTEXT_ID
        );
        assert_eq!(
            payload["right_scope"]["commit_id"],
            MERGE_REVIEW_RIGHT_COMMIT_ID
        );
        assert_eq!(
            payload["classification"]["Conflict"]["conflicts"][0]["Node"]["node_id"],
            "context:support-resolution-agent"
        );
    }

    #[tokio::test]
    async fn protected_persisted_context_merge_review_fails_closed_without_a_witness_repository() {
        let (state, token, graph_reads) = server_owned_merge_review_state_with_witness(true, false);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id={MERGE_REVIEW_RIGHT_COMMIT_ID}"
                ),
                Some(&token),
            ))
            .await
            .expect("missing witness response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "context_merge_review_unavailable");
        assert_eq!(graph_reads.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn protected_persisted_context_merge_review_rejects_caller_owned_plan_fields() {
        let (state, token, _) = server_owned_merge_review_state(true);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id={MERGE_REVIEW_RIGHT_COMMIT_ID}&base_commit_id={VERSION_BACKED_ORIGINAL_COMMIT_ID}"
                ),
                Some(&token),
            ))
            .await
            .expect("caller plan response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_context_merge_review_query");
    }

    #[tokio::test]
    async fn protected_persisted_context_merge_review_fails_closed_without_an_exact_snapshot() {
        let (state, token, _) = server_owned_merge_review_state(false);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id={MERGE_REVIEW_RIGHT_COMMIT_ID}"
                ),
                Some(&token),
            ))
            .await
            .expect("missing snapshot response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "context_merge_review_unavailable");
        assert!(!payload.to_string().contains(MERGE_REVIEW_RIGHT_COMMIT_ID));
    }

    #[tokio::test]
    async fn persisted_context_merge_review_is_private_to_the_protected_router() {
        let (state, token, _) = server_owned_merge_review_state(true);
        let missing_auth = build_protected_router_with_state(state.clone())
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id={MERGE_REVIEW_RIGHT_COMMIT_ID}"
                ),
                None,
            ))
            .await
            .expect("missing auth response");
        assert_eq!(missing_auth.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            missing_auth.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );

        let public = build_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id={MERGE_REVIEW_RIGHT_COMMIT_ID}"
                ),
                Some(&token),
            ))
            .await
            .expect("public response");
        assert_eq!(public.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn protected_persisted_context_merge_review_reads_server_owned_graph_once() {
        let (state, token, graph_reads) = server_owned_merge_review_state(true);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id={MERGE_REVIEW_RIGHT_COMMIT_ID}"
                ),
                Some(&token),
            ))
            .await
            .expect("single graph read response");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(graph_reads.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn protected_persisted_context_merge_review_rejects_unknown_tip() {
        let (state, token, _) = server_owned_merge_review_state(true);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}&right_commit_id=66666666-6666-4666-8666-666666666666"
                ),
                Some(&token),
            ))
            .await
            .expect("unknown tip response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "context_merge_review_unavailable");
        assert!(
            !payload
                .to_string()
                .contains("66666666-6666-4666-8666-666666666666")
        );
    }

    #[tokio::test]
    async fn protected_persisted_context_merge_review_rejects_fast_forward_plan() {
        let (state, token, _) = server_owned_merge_review_state(true);
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_merge_review_request(
                &format!(
                    "left_commit_id={VERSION_BACKED_ORIGINAL_COMMIT_ID}&right_commit_id={VERSION_BACKED_REVISED_COMMIT_ID}"
                ),
                Some(&token),
            ))
            .await
            .expect("fast-forward response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "context_merge_review_unavailable");
    }

    #[tokio::test]
    async fn protected_persisted_context_diff_review_reports_missing_exact_input() {
        let (state, token) = version_backed_graph_diff_state(
            version_backed_graph_diff_snapshot_repository(),
            AllowContextWrites,
        );
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_diff_review_request(
                VERSION_BACKED_PROJECT_ID,
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_REVISED_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("missing persisted Context diff review response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(payload["error"], "context_diff_review_snapshot_missing");
    }

    #[tokio::test]
    async fn protected_persisted_context_diff_review_requires_auth_and_rejects_identical_commits() {
        let state = AppState::new(ProviderRegistry::from_env(
            std::iter::empty::<(&str, &str)>(),
        ));
        let missing_auth = build_protected_router_with_state(state.clone())
            .oneshot(persisted_context_diff_review_request(
                VERSION_BACKED_PROJECT_ID,
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_REVISED_COMMIT_ID,
                None,
            ))
            .await
            .expect("missing auth response");
        assert_eq!(missing_auth.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            missing_auth.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );

        let (state, token) = version_backed_graph_diff_state(
            version_backed_graph_diff_snapshot_repository(),
            AllowContextWrites,
        );
        let response = build_protected_router_with_state(state)
            .oneshot(persisted_context_diff_review_request(
                VERSION_BACKED_PROJECT_ID,
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("identical persisted Context diff review response");
        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(payload["error"], "invalid_commit_graph_diff_query");
    }

    #[tokio::test]
    async fn protected_graph_diff_rejects_quota_exhaustion_before_authorization_or_reads() {
        let calls = Arc::new(AtomicUsize::new(0));
        let snapshot_repository = RecordingCommitGraphSnapshotRepository {
            inner: version_backed_graph_diff_snapshot_repository(),
            calls: calls.clone(),
        };
        let (state, token) =
            version_backed_graph_diff_state(snapshot_repository, AllowContextWrites);
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 7,
        }));
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let state = state
            .with_protected_rate_limiter(limiter.clone())
            .with_authorization_audit_sink(RecordingAuditSink {
                events: events.clone(),
            });

        let response = build_protected_router_with_state(state)
            .oneshot(version_backed_graph_diff_request(
                VERSION_BACKED_CONTEXT_ID,
                VERSION_BACKED_ORIGINAL_COMMIT_ID,
                VERSION_BACKED_REVISED_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("rate limited graph diff response");

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        assert_eq!(response.headers()[header::RETRY_AFTER], "7");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(events.lock().expect("audit events lock").is_empty());
        assert_eq!(
            limiter.operations(),
            vec![ProtectedRouteOperation::ContextCommitGraphDiffRead]
        );
    }

    #[tokio::test]
    async fn protected_graph_diff_rejects_post_requests() {
        let (state, token) = version_backed_graph_diff_state(
            version_backed_graph_diff_snapshot_repository(),
            AllowContextWrites,
        );
        let mut request = version_backed_graph_diff_request(
            VERSION_BACKED_CONTEXT_ID,
            VERSION_BACKED_ORIGINAL_COMMIT_ID,
            VERSION_BACKED_REVISED_COMMIT_ID,
            Some(&token),
        );
        *request.method_mut() = axum::http::Method::POST;
        let response = build_protected_router_with_state(state)
            .oneshot(request)
            .await
            .expect("post graph diff response");

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
    }

    #[tokio::test]
    async fn graph_diffs_route_rejects_invalid_graph_snapshots() {
        let payload = serde_json::json!({
            "original": {
                "nodes": [
                    { "id": "context:agent", "kind": "context", "label": "Support Agent" },
                    { "id": "context:agent", "kind": "context", "label": "Duplicate Agent" }
                ],
                "edges": []
            },
            "revised": { "nodes": [], "edges": [] }
        });
        let response = test_router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/graph-diffs")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let error: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(error["error"], "invalid_graph_snapshot");
    }

    #[tokio::test]
    async fn graph_diffs_route_rejects_invalid_snapshot_json() {
        let payload = serde_json::json!({
            "original": {
                "nodes": [
                    { "id": "context:agent", "kind": "not_a_graph_kind", "label": "Support Agent" }
                ],
                "edges": []
            },
            "revised": { "nodes": [], "edges": [] }
        });
        let response = test_router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/graph-diffs")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let error: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(error["error"], "invalid_graph_snapshot");
    }

    #[tokio::test]
    async fn context_graph_preview_route_reports_projection_errors() {
        let registry = ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>());
        let created_at =
            ContextGraphProjection::context_engineering_preview().projects[0].created_at;
        let projection = ContextGraphProjection {
            projects: vec![ProjectRecord {
                id: "orphan".to_owned(),
                workspace_id: "missing".to_owned(),
                name: "Orphan Project".to_owned(),
                slug: "orphan".to_owned(),
                created_at,
            }],
            ..ContextGraphProjection::default()
        };
        let router = build_router_with_state(AppState::with_graph_repository(
            registry,
            InMemoryContextGraphRepository::new(projection),
        ));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/v1/context-graph/preview")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "graph_projection_failed");
    }

    #[tokio::test]
    async fn workspace_context_graph_route_returns_default_workspace_graph() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/default/context-graph")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");
        let nodes = payload["graph"]["nodes"].as_object().expect("nodes object");

        assert!(nodes.values().any(|node| node["kind"] == "workspace"));
        assert!(nodes.values().any(|node| node["kind"] == "context"));
    }

    #[tokio::test]
    async fn workspace_context_graph_route_reports_unknown_workspace() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/missing/context-graph")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_unavailable");
    }

    #[tokio::test]
    async fn postgres_workspace_context_graph_route_rejects_invalid_workspace_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/not-a-uuid/context-graph")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_workspace_projects_route_rejects_invalid_workspace_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/workspaces/not-a-uuid/projects")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_project_experiments_route_rejects_invalid_project_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/not-a-uuid/experiments")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_project_contexts_route_rejects_invalid_project_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/projects/not-a-uuid/contexts")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_project_contexts_route_rejects_invalid_experiment_filter_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/projects/11111111-1111-4111-8111-111111111111/contexts?experiment_id=not-a-uuid",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_context_commits_route_rejects_invalid_context_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/not-a-uuid/commits")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_context_commit_route_rejects_invalid_commit_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits/not-a-uuid")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_context_components_route_rejects_invalid_context_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/not-a-uuid/components")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_context_component_route_rejects_invalid_component_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/11111111-1111-4111-8111-111111111111/components/not-a-uuid",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_context_evaluation_runs_route_rejects_invalid_context_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/not-a-uuid/evaluation-runs")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_context_evaluation_scorecard_route_rejects_invalid_context_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/contexts/not-a-uuid/evaluation-scorecard")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn postgres_context_evaluation_run_route_rejects_invalid_run_id() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/contexts/11111111-1111-4111-8111-111111111111/evaluation-runs/not-a-uuid",
                    )
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let payload: Value = serde_json::from_slice(&body).expect("json payload");

        assert_eq!(payload["error"], "storage_scope_invalid");
    }

    #[tokio::test]
    async fn preview_route_stays_available_when_workspace_repository_is_postgres() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("state");
        let response = build_router_with_state(state)
            .oneshot(
                Request::builder()
                    .uri("/api/v1/context-graph/preview")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn contextlab_database_url_takes_precedence_for_postgres_repository() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
            ("DATABASE_URL", "not-a-valid-postgres-url"),
        ]);

        assert!(state.is_ok());
    }

    #[test]
    fn protected_runtime_requires_postgres_storage() {
        let result = try_build_router_from_env([
            ("CONTEXTLAB_API_ROUTE_MODE", "protected"),
            ("CONTEXTLAB_AUTH_HS256_SECRET", "test-secret"),
            ("CONTEXTLAB_AUTH_ISSUER", "https://issuer.contextlab.test"),
            ("CONTEXTLAB_AUTH_AUDIENCE", "contextlab-web"),
        ]);

        assert!(matches!(
            result,
            Err(AppStateConfigError::ProtectedRoutesRequirePostgres)
        ));
    }

    #[tokio::test]
    async fn protected_runtime_requires_complete_hmac_configuration() {
        let base = [
            ("CONTEXTLAB_API_ROUTE_MODE", "protected"),
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ];

        assert!(matches!(
            try_build_router_from_env(base),
            Err(AppStateConfigError::MissingProtectedAuthSecret)
        ));
        assert!(matches!(
            try_build_router_from_env(
                base.into_iter()
                    .chain([("CONTEXTLAB_AUTH_HS256_SECRET", "test-secret",)])
            ),
            Err(AppStateConfigError::MissingProtectedAuthIssuer)
        ));
        assert!(matches!(
            try_build_router_from_env(base.into_iter().chain([
                ("CONTEXTLAB_AUTH_HS256_SECRET", "test-secret"),
                ("CONTEXTLAB_AUTH_ISSUER", "https://issuer.contextlab.test"),
            ])),
            Err(AppStateConfigError::MissingProtectedAuthAudience)
        ));
    }

    #[tokio::test]
    async fn protected_runtime_requires_complete_safe_oidc_configuration() {
        let base = [
            ("CONTEXTLAB_API_ROUTE_MODE", "protected"),
            ("CONTEXTLAB_AUTH_MODE", "oidc"),
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ];

        assert!(matches!(
            try_build_router_from_env(base),
            Err(AppStateConfigError::MissingOidcIssuer)
        ));
        assert!(matches!(
            try_build_router_from_env(
                base.into_iter()
                    .chain([("CONTEXTLAB_OIDC_ISSUER", "https://issuer.contextlab.test",)])
            ),
            Err(AppStateConfigError::MissingOidcAudience)
        ));
        assert!(matches!(
            try_build_router_from_env(base.into_iter().chain([
                ("CONTEXTLAB_OIDC_ISSUER", "https://issuer.contextlab.test"),
                ("CONTEXTLAB_OIDC_AUDIENCE", "contextlab-web"),
            ])),
            Err(AppStateConfigError::MissingOidcJwksUrl)
        ));
        assert!(matches!(
            try_build_router_from_env(base.into_iter().chain([
                ("CONTEXTLAB_OIDC_ISSUER", "https://issuer.contextlab.test"),
                ("CONTEXTLAB_OIDC_AUDIENCE", "contextlab-web"),
                (
                    "CONTEXTLAB_OIDC_JWKS_URL",
                    "http://issuer.contextlab.test/keys"
                ),
                ("CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS", "60"),
            ])),
            Err(AppStateConfigError::InvalidOidcConfiguration)
        ));
        assert!(matches!(
            try_build_router_from_env(base.into_iter().chain([
                ("CONTEXTLAB_OIDC_ISSUER", "https://issuer.contextlab.test"),
                ("CONTEXTLAB_OIDC_AUDIENCE", "contextlab-web"),
                (
                    "CONTEXTLAB_OIDC_JWKS_URL",
                    "https://issuer.contextlab.test/keys",
                ),
                ("CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS", "0"),
            ])),
            Err(AppStateConfigError::InvalidOidcCacheTtl)
        ));
    }

    #[test]
    fn oidc_group_configuration_requires_a_claim_and_bounded_token_lifetime_together() {
        let base = vec![
            (
                "CONTEXTLAB_OIDC_ISSUER".to_owned(),
                "https://issuer.contextlab.test".to_owned(),
            ),
            (
                "CONTEXTLAB_OIDC_AUDIENCE".to_owned(),
                "contextlab-web".to_owned(),
            ),
            (
                "CONTEXTLAB_OIDC_JWKS_URL".to_owned(),
                "https://issuer.contextlab.test/keys".to_owned(),
            ),
            (
                "CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS".to_owned(),
                "60".to_owned(),
            ),
        ];
        assert!(oidc_jwks_config_from_env(&base).is_ok());
        assert!(matches!(
            oidc_jwks_config_from_env(
                &base
                    .iter()
                    .cloned()
                    .chain([(
                        "CONTEXTLAB_OIDC_GROUPS_CLAIM".to_owned(),
                        "groups".to_owned(),
                    )])
                    .collect::<Vec<_>>(),
            ),
            Err(AppStateConfigError::MissingOidcMaxTokenLifetime)
        ));
        assert!(matches!(
            oidc_jwks_config_from_env(
                &base
                    .iter()
                    .cloned()
                    .chain([(
                        "CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS".to_owned(),
                        "300".to_owned(),
                    )])
                    .collect::<Vec<_>>(),
            ),
            Err(AppStateConfigError::MissingOidcGroupsClaim)
        ));
        assert!(matches!(
            oidc_jwks_config_from_env(
                &base
                    .iter()
                    .cloned()
                    .chain([
                        (
                            "CONTEXTLAB_OIDC_GROUPS_CLAIM".to_owned(),
                            "groups".to_owned()
                        ),
                        (
                            "CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS".to_owned(),
                            "0".to_owned(),
                        ),
                    ])
                    .collect::<Vec<_>>(),
            ),
            Err(AppStateConfigError::InvalidOidcMaxTokenLifetime)
        ));
        assert!(
            oidc_jwks_config_from_env(
                &base
                    .into_iter()
                    .chain([
                        (
                            "CONTEXTLAB_OIDC_GROUPS_CLAIM".to_owned(),
                            "groups".to_owned()
                        ),
                        (
                            "CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS".to_owned(),
                            "300".to_owned(),
                        ),
                    ])
                    .collect::<Vec<_>>(),
            )
            .is_ok()
        );
    }

    #[tokio::test]
    async fn protected_rate_limit_configuration_is_required_and_bounded() {
        let base = [
            ("CONTEXTLAB_API_ROUTE_MODE", "protected"),
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
            ("CONTEXTLAB_AUTH_HS256_SECRET", "test-secret"),
            ("CONTEXTLAB_AUTH_ISSUER", "https://issuer.contextlab.test"),
            ("CONTEXTLAB_AUTH_AUDIENCE", "contextlab-web"),
        ];

        assert!(matches!(
            try_build_router_from_env(base),
            Err(AppStateConfigError::MissingProtectedRateLimitMaxRequests)
        ));
        assert!(matches!(
            try_build_router_from_env(
                base.into_iter()
                    .chain([("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "10",)])
            ),
            Err(AppStateConfigError::MissingProtectedRateLimitWindowSeconds)
        ));
        assert!(matches!(
            try_build_router_from_env(base.into_iter().chain([
                ("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "10"),
                ("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS", "60"),
            ])),
            Err(AppStateConfigError::MissingProtectedRateLimitMaxTrackedPrincipals)
        ));

        let mut complete = base.to_vec();
        complete.extend([
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "10"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS", "60"),
            (
                "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS",
                "100",
            ),
        ]);
        for (name, value) in [
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "0"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "1001"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "invalid"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS", "0"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS", "3601"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS", "invalid"),
            (
                "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS",
                "0",
            ),
            (
                "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS",
                "100001",
            ),
            (
                "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS",
                "invalid",
            ),
        ] {
            let mut invalid = complete.clone();
            invalid
                .iter_mut()
                .find(|(candidate, _)| *candidate == name)
                .expect("rate-limit setting")
                .1 = value;
            let result = try_build_router_from_env(invalid);
            assert!(matches!(
                result,
                Err(AppStateConfigError::InvalidProtectedRateLimitConfiguration)
            ));
        }
    }

    #[test]
    fn public_runtime_ignores_protected_rate_limit_configuration() {
        assert!(
            try_build_router_from_env([
                ("CONTEXTLAB_API_ROUTE_MODE", "public"),
                ("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "invalid"),
                ("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS", "invalid"),
                (
                    "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS",
                    "invalid",
                ),
            ])
            .is_ok()
        );
    }

    #[test]
    fn public_runtime_ignores_protected_oidc_group_configuration() {
        assert!(
            try_build_router_from_env([
                ("CONTEXTLAB_API_ROUTE_MODE", "public"),
                ("CONTEXTLAB_AUTH_MODE", "oidc"),
                ("CONTEXTLAB_OIDC_GROUPS_CLAIM", "not a valid claim name"),
                ("CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS", "0"),
            ])
            .is_ok()
        );
    }

    #[tokio::test]
    async fn protected_runtime_installs_authentication_middleware() {
        let router = try_build_router_from_env([
            ("CONTEXTLAB_API_ROUTE_MODE", "protected"),
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
            ("CONTEXTLAB_AUTH_HS256_SECRET", "test-secret"),
            ("CONTEXTLAB_AUTH_ISSUER", "https://issuer.contextlab.test"),
            ("CONTEXTLAB_AUTH_AUDIENCE", "contextlab-web"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS", "10"),
            ("CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS", "60"),
            (
                "CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS",
                "100",
            ),
        ])
        .expect("protected runtime router");
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits")
                    .body(Body::from("{}"))
                    .expect("request"),
            )
            .await
            .expect("response");

        let (status, payload) = response_json(response).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(payload["error"], "authentication_required");
    }

    #[tokio::test]
    async fn protected_rate_limit_authentication_precedes_quota_consumption() {
        let (state, context_id, _) = protected_test_state();
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 30,
        }));
        let state = state
            .with_authorization_audit_sink(RecordingAuditSink {
                events: events.clone(),
            })
            .with_protected_rate_limiter(limiter.clone());

        let response = build_protected_router_with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/contexts/{context_id}/commits"))
                    .header("content-type", "application/json")
                    .body(Body::from(protected_commit_body().to_string()))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(limiter.calls(), 0);
        assert!(events.lock().expect("audit events lock").is_empty());
        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn protected_rate_limit_rejection_returns_retry_after_without_downstream_effects() {
        let (state, context_id, token) = protected_test_state();
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let limiter = StaticRateLimiter::new(Ok(RateLimitDecision::Rejected {
            retry_after_seconds: 17,
        }));
        let state = state
            .with_authorization_audit_sink(RecordingAuditSink {
                events: events.clone(),
            })
            .with_protected_rate_limiter(limiter.clone());

        let response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-rate-limited"),
                Body::from(protected_commit_body().to_string()),
            ))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], "17");
        let (_, payload) = response_json(response).await;
        assert_eq!(payload["error"], "rate_limit_exceeded");
        assert_eq!(limiter.calls(), 1);
        assert!(events.lock().expect("audit events lock").is_empty());
        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn protected_rate_limit_unavailability_fails_closed_without_downstream_effects() {
        let (state, context_id, token) = protected_test_state();
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let limiter = StaticRateLimiter::new(Err(RateLimitError::Unavailable));
        let state = state
            .with_authorization_audit_sink(RecordingAuditSink {
                events: events.clone(),
            })
            .with_protected_rate_limiter(limiter.clone());

        let response = build_protected_router_with_state(state.clone())
            .oneshot(protected_commit_request(
                &context_id,
                &token,
                Some("request-rate-limit-unavailable"),
                Body::from(protected_commit_body().to_string()),
            ))
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(!response.headers().contains_key(header::RETRY_AFTER));
        let (_, payload) = response_json(response).await;
        assert_eq!(payload["error"], "rate_limit_unavailable");
        assert_eq!(limiter.calls(), 1);
        assert!(events.lock().expect("audit events lock").is_empty());
        let commits = state
            .commit_repository()
            .list_commits(context_id, contextlab_storage::CommitListQuery::default())
            .await
            .expect("list commits");
        assert!(commits.items.is_empty());
    }

    #[tokio::test]
    async fn postgres_workspace_data_exposes_a_durable_authorization_audit_sink() {
        let postgres = WorkspaceDataRepository::Postgres(
            PostgresContextGraphRepository::connect_lazy(
                "postgres://contextlab:contextlab@localhost/contextlab",
            )
            .expect("valid lazy repository"),
        );
        let memory = WorkspaceDataRepository::Memory(
            InMemoryContextGraphRepository::context_engineering_preview(),
        );

        assert!(postgres.postgres_authorization_audit_sink().is_some());
        assert!(memory.postgres_authorization_audit_sink().is_none());
    }

    #[tokio::test]
    async fn postgres_mode_wires_a_storage_backed_workflow_execution_status_reader() {
        let state = AppState::try_from_env([
            ("CONTEXTLAB_GRAPH_REPOSITORY", "postgres"),
            (
                "CONTEXTLAB_DATABASE_URL",
                "postgres://contextlab:contextlab@localhost/contextlab",
            ),
        ])
        .expect("postgres mode state");

        assert!(state.workflow_execution_status_repository().is_some());
        assert_eq!(
            state.workflow_execution_status_repository_backend(),
            WorkflowExecutionStatusRepositoryBackend::StorageBacked
        );
    }

    #[tokio::test]
    async fn memory_mode_keeps_workflow_execution_status_unavailable() {
        let state = AppState::try_from_env([("CONTEXTLAB_GRAPH_REPOSITORY", "memory")])
            .expect("memory mode state");
        let reader = state
            .workflow_execution_status_repository()
            .expect("default status reader");

        assert_eq!(
            state.workflow_execution_status_repository_backend(),
            WorkflowExecutionStatusRepositoryBackend::Unavailable
        );
        let result = reader
            .read(
                ContextId::from_uuid(Uuid::from_u128(9_902)),
                WorkflowRunId::from_uuid(Uuid::from_u128(9_903)),
            )
            .await;

        assert!(matches!(
            result,
            Err(workflow_execution::WorkflowExecutionStatusStorageError::Unavailable)
        ));
    }

    #[tokio::test]
    async fn workflow_execution_status_backend_tracks_builder_overrides() {
        let state = AppState::try_from_env([("CONTEXTLAB_GRAPH_REPOSITORY", "memory")])
            .expect("memory mode state")
            .with_workflow_execution_status_repository(
                workflow_execution::UnavailableWorkflowExecutionStatusAdapter,
            )
            .with_workflow_execution_status_storage_repository(
                contextlab_storage::InMemoryWorkflowExecutionStatusRepository::default(),
            );

        assert_eq!(
            state.workflow_execution_status_repository_backend(),
            WorkflowExecutionStatusRepositoryBackend::StorageBacked
        );
    }

    #[test]
    fn postgres_graph_repository_requires_database_url() {
        let error = AppState::try_from_env([("CONTEXTLAB_GRAPH_REPOSITORY", "postgres")])
            .expect_err("postgres mode requires database url");

        assert!(matches!(error, AppStateConfigError::MissingDatabaseUrl));
    }

    #[test]
    fn rejects_unknown_graph_repository_mode() {
        let error = AppState::try_from_env([("CONTEXTLAB_GRAPH_REPOSITORY", "unsupported")])
            .expect_err("unknown mode should fail");

        assert!(matches!(
            error,
            AppStateConfigError::UnsupportedGraphRepositoryMode { .. }
        ));
    }
}
