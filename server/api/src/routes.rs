//! API route handlers.

use crate::{
    AppState, knowledge_memory::KnowledgeMemoryProjectionResource,
    plugin_capability::PluginCapabilityAvailabilityResource,
    workflow_context_bindings::LocalWorkflowContextBindingsResponse,
    workflow_execution::LocalWorkflowExecutionStatusResource,
    workflow_status::UnavailableWorkflowCapabilityAdapter,
};
use axum::{
    Json,
    extract::{
        Extension, Path, Query, Request, State,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use contextlab_auth::{
    AuthorizationAuditEvent, AuthorizationDecision, AuthorizationError, ContextPermission,
    ProtectedRouteOperation, ProtectedRouteRateLimitKey, RateLimitDecision,
};
use contextlab_context_core::{
    ComponentContent, ComponentId, ContextComponentKind, ContextId, ContextMetadata, ProjectId,
};
use contextlab_diff_engine::{
    GraphDiff, VersionedContextGraphMergeReviewProjectionV1, VersionedContextScopeV1,
};
use contextlab_evaluation::{BenchmarkExecutionCohortId, BenchmarkWorkspaceProjectionV1};
use contextlab_graph::{
    ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind, GraphValidationError,
};
use contextlab_model_gateway::PublicProviderStatus;
use contextlab_storage::{
    BenchmarkDecisionComparisonScope, BenchmarkDecisionComparisonService,
    BenchmarkDecisionComparisonServiceError, BenchmarkDecisionDefinitionSummary,
    BenchmarkDecisionDefinitionSummaryError, BenchmarkDecisionDefinitionSummaryService,
    BenchmarkDecisionDiscoveryService, BenchmarkDecisionDiscoveryServiceError,
    BenchmarkDecisionDiscoverySummary, BenchmarkDecisionEvidence, BenchmarkDecisionId,
    BenchmarkDecisionRunDetails, BenchmarkDecisionRunDetailsError,
    BenchmarkDecisionRunDetailsService, BenchmarkWorkspaceProjectionDecisionQuery,
    BenchmarkWorkspaceProjectionPersistenceError, BenchmarkWorkspaceProjectionReceiptScope,
    BenchmarkWorkspaceProjectionV1Query, CommitDetail, CommitGraphSnapshot,
    CommitGraphSnapshotScope, CommitList, CommitListQuery, CommitSort, CommitSortParseError,
    ComponentDetail, ComponentList, ComponentListQuery, ComponentSort, ComponentSortParseError,
    ContextLifecycleCommand, ContextLifecycleError, ContextLifecycleOperation,
    ContextLifecycleService, ContextLifecycleStateAtCommit, ContextList, ContextListQuery,
    ContextMergeTipScope, ContextSort, ContextSortParseError, CreateContextCommitSnapshot,
    EvaluationRunDetail, EvaluationRunList, EvaluationRunListQuery, EvaluationRunSort,
    EvaluationRunSortParseError, EvaluationScorecard, EvaluationScorecardQuery, ExperimentList,
    ExperimentListQuery, ExperimentSort, ExperimentSortParseError, GraphProjectionScope,
    GuardedCommitWriteDisposition, GuardedContextCommitWrite, IdempotencyKey,
    PersistedContextDiffReviewError, PersistedContextDiffReviewService,
    PersistedContextGraphDiffReviewError, PersistedContextGraphHistoryReviewError,
    PersistedContextGraphMergeReviewError, PersistedContextGraphMergeReviewService,
    PersistedContextGraphWitnessReviewService, ProjectList, ProjectListQuery, ProjectSort,
    ProjectSortParseError, RequestDigest, StorageProjectionError, StorageRepositoryError,
    StoredComponentKind, StoredComponentKindParseError, WorkspaceList, WorkspaceListQuery,
    WorkspaceSort, WorkspaceSortParseError,
};
use contextlab_versioning::{
    BranchName, CommitId, ContextChange, ContextCommit, ExpectedBranchHead,
};
use contextlab_workflow::WorkflowRunId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use thiserror::Error;
use uuid::Uuid;

/// Health response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

/// Platform metadata response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MetaResponse {
    product: &'static str,
    primary_abstraction: &'static str,
    bilingual: bool,
    context_component_kinds: Vec<ContextComponentKind>,
    capabilities: Vec<&'static str>,
}

/// Provider registry response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvidersResponse {
    providers: Vec<PublicProviderStatus>,
}

/// Context Graph response body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextGraphResponse {
    graph: ContextGraph,
}

/// Graph comparison request body.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct GraphDiffRequest {
    original: GraphSnapshotRequest,
    revised: GraphSnapshotRequest,
}

/// Explicit graph snapshot input used for comparison.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct GraphSnapshotRequest {
    nodes: Vec<GraphNodeRequest>,
    edges: Vec<GraphEdgeRequest>,
}

/// Explicit graph node input used for comparison.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct GraphNodeRequest {
    id: String,
    kind: GraphNodeKind,
    label: String,
}

/// Explicit graph edge input used for comparison.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct GraphEdgeRequest {
    source: String,
    target: String,
    kind: GraphEdgeKind,
}

/// Request body for the explicitly protected normal-commit route.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct ContextCommitWriteRequest {
    /// Branch name receiving the commit.
    branch_name: String,
    /// Client-observed head; omitted or null means an unborn branch.
    expected_head_commit_id: Option<String>,
    /// Commit message.
    message: String,
    /// Ordered replayable Context changes.
    changes: Vec<ContextChange>,
    /// Graph state captured by the commit.
    snapshot: GraphSnapshotRequest,
    /// Graph snapshot payload schema version.
    schema_version: u16,
}

/// Response body for a protected normal commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextCommitWriteResponse {
    /// Created or replayed disposition.
    disposition: &'static str,
    /// Immutable graph snapshot associated with the commit.
    snapshot: CommitGraphSnapshot,
}

/// Request body for a local-only, guarded component lifecycle transition.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalContextLifecycleWriteRequest {
    /// Branch name receiving the successor commit.
    branch_name: String,
    /// Null only when initializing an unborn branch; otherwise the materialized normal head observed by the caller.
    expected_head_commit_id: Option<String>,
    /// Commit message for the replayable lifecycle transition.
    message: String,
    /// Typed component lifecycle intent. Server-side code constructs the change and graph snapshot.
    operation: LocalContextLifecycleOperationRequest,
}

/// A local-only lifecycle intent transported without client-built commits or graph snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum LocalContextLifecycleOperationRequest {
    /// Initialize one existing Context on an unborn branch without creating a component.
    Initialize {},
    /// Create one named component and its immutable initial body.
    Create {
        component_kind: ContextComponentKind,
        name: String,
        metadata: Value,
        content: ComponentContent,
    },
    /// Revise one component's immutable body.
    Update {
        component_id: String,
        content: ComponentContent,
    },
    /// Revise one component's display name and metadata without changing its body.
    UpdateDescriptor {
        component_id: String,
        name: String,
        metadata: Value,
    },
    /// Replace the replayable metadata of the Context without changing components.
    UpdateMetadata { metadata: ContextMetadata },
    /// Remove one component from successor Context states.
    Remove { component_id: String },
    /// Add one directed `Uses` relationship between two components.
    AddUsesRelationship {
        source_component_id: String,
        target_component_id: String,
    },
    /// Remove one directed `Uses` relationship between two components.
    RemoveUsesRelationship {
        source_component_id: String,
        target_component_id: String,
    },
}

const LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1: &str =
    "contextlab.local-component-lifecycle-commit.v1";
const LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1: &str = "contextlab.local-context-lifecycle-state.v1";

/// Created or replayed local lifecycle commit facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalContextLifecycleWriteResponse {
    /// Explicit version for this private response envelope.
    schema_version: &'static str,
    /// Whether the guarded writer created or replayed the transition.
    disposition: &'static str,
    /// Immutable commit identifier for the lifecycle transition.
    commit_id: String,
    /// Immutable graph snapshot atomically associated with the transition.
    snapshot: CommitGraphSnapshot,
}

/// Exact private lifecycle state at a materialized Context commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalContextLifecycleStateResponse {
    /// Explicit version for this private response envelope.
    schema_version: &'static str,
    /// Context owning the historical state.
    context_id: String,
    /// Materialized commit represented by this response.
    commit_id: String,
    /// Replayed typed Context metadata at the materialized commit.
    metadata: Option<ContextMetadata>,
    /// Replayable component descriptors with their immutable effective bodies.
    components: Vec<LocalContextLifecycleComponentResponse>,
    /// Immutable graph snapshot from the same commit.
    graph_snapshot: CommitGraphSnapshot,
}

/// Exact private benchmark decision evidence without benchmark case payloads.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalBenchmarkDecisionResponse {
    project_id: String,
    context_id: String,
    commit_id: String,
    decision_id: String,
    suite_id: String,
    dataset_ids: Vec<String>,
    definition: LocalBenchmarkDecisionDefinitionResponse,
    run_ids: Vec<String>,
    comparability: LocalBenchmarkComparabilityResponse,
    evidence_digest: String,
    status: contextlab_evaluation::RegressionDecisionStatus,
    recorded_at: chrono::DateTime<Utc>,
    metrics: Vec<LocalBenchmarkMetricEvidenceResponse>,
}

/// Exact private list of sealed benchmark decisions without raw benchmark payloads.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalBenchmarkDecisionListResponse {
    schema_version: &'static str,
    project_id: String,
    context_id: String,
    commit_id: String,
    decisions: Vec<LocalBenchmarkDecisionListItemResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct LocalBenchmarkDecisionListItemResponse {
    decision_id: String,
    suite: LocalBenchmarkDecisionListSuiteResponse,
    datasets: Vec<LocalBenchmarkDecisionListDatasetResponse>,
    status: contextlab_evaluation::RegressionDecisionStatus,
    recorded_at: chrono::DateTime<Utc>,
    run_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkDecisionListSuiteResponse {
    id: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkDecisionListDatasetResponse {
    id: String,
    name: String,
    case_count: usize,
}

/// Exact private redacted run cohort behind one sealed benchmark decision.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalBenchmarkDecisionRunDetailsResponse {
    project_id: String,
    context_id: String,
    commit_id: String,
    decision_id: String,
    runs: Vec<LocalBenchmarkDecisionRunResponse>,
}

/// One safe persisted benchmark run without case or model payloads.
#[derive(Debug, Clone, PartialEq, Serialize)]
struct LocalBenchmarkDecisionRunResponse {
    run_id: String,
    model_version: String,
    temperature: f32,
    metrics: Vec<LocalBenchmarkDecisionRunMetricResponse>,
    executed_at: chrono::DateTime<Utc>,
}

/// One safe numeric measurement from a sealed run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
struct LocalBenchmarkDecisionRunMetricResponse {
    metric: contextlab_evaluation::MetricKind,
    value: f64,
}

/// Safe immutable benchmark definitions bound to one exact private decision.
#[derive(Debug, Clone, PartialEq, Serialize)]
struct LocalBenchmarkDecisionDefinitionResponse {
    suite: LocalBenchmarkDecisionSuiteResponse,
    datasets: Vec<LocalBenchmarkDecisionDatasetResponse>,
}

/// Safe immutable benchmark suite metadata without policy execution details.
#[derive(Debug, Clone, PartialEq, Serialize)]
struct LocalBenchmarkDecisionSuiteResponse {
    id: String,
    name: String,
    thresholds: Vec<LocalBenchmarkThresholdResponse>,
}

/// One validated benchmark threshold in stable metric order.
#[derive(Debug, Clone, PartialEq, Serialize)]
struct LocalBenchmarkThresholdResponse {
    metric: contextlab_evaluation::MetricKind,
    direction: contextlab_evaluation::ThresholdDirection,
    value: f64,
}

/// Safe immutable benchmark dataset metadata without its case payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkDecisionDatasetResponse {
    id: String,
    name: String,
    case_count: usize,
}

/// Exact private comparison of two sealed benchmark decisions without raw benchmark payloads.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalBenchmarkDecisionDiffResponse {
    project_id: String,
    context_id: String,
    baseline: LocalBenchmarkDecisionScopeResponse,
    revised: LocalBenchmarkDecisionScopeResponse,
    status_change: Option<LocalBenchmarkDecisionStatusChangeResponse>,
    metric_changes: Vec<LocalBenchmarkDecisionMetricDiffResponse>,
}

/// Safe benchmark workspace projection at one exact private Context commit and cohort scope.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocalBenchmarkWorkspaceResponse {
    schema_version: &'static str,
    project_id: String,
    context_id: String,
    revised: LocalBenchmarkWorkspaceScopeResponse,
    baseline: Option<LocalBenchmarkWorkspaceScopeResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision_pair_witness: Option<LocalBenchmarkDecisionPairWitnessResponse>,
    projection: BenchmarkWorkspaceProjectionV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkWorkspaceScopeResponse {
    commit_id: String,
    cohort_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkDecisionScopeResponse {
    commit_id: String,
    decision_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkDecisionPairWitnessResponse {
    schema_version: u16,
    project_id: String,
    context_id: String,
    baseline: LocalBenchmarkDecisionScopeResponse,
    revised: LocalBenchmarkDecisionScopeResponse,
}

impl LocalBenchmarkDecisionPairWitnessResponse {
    fn from_resolved_scopes(
        project_id: ProjectId,
        context_id: ContextId,
        baseline: BenchmarkWorkspaceProjectionReceiptScope,
        baseline_decision_id: BenchmarkDecisionId,
        revised: BenchmarkWorkspaceProjectionReceiptScope,
        revised_decision_id: BenchmarkDecisionId,
    ) -> Self {
        Self {
            schema_version: 1,
            project_id: project_id.to_string(),
            context_id: context_id.to_string(),
            baseline: LocalBenchmarkDecisionScopeResponse {
                commit_id: baseline.context_commit_id().to_string(),
                decision_id: baseline_decision_id.to_string(),
            },
            revised: LocalBenchmarkDecisionScopeResponse {
                commit_id: revised.context_commit_id().to_string(),
                decision_id: revised_decision_id.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkDecisionStatusChangeResponse {
    baseline: contextlab_evaluation::RegressionDecisionStatus,
    revised: contextlab_evaluation::RegressionDecisionStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum LocalBenchmarkDecisionMetricDiffResponse {
    Added {
        metric: contextlab_evaluation::MetricKind,
        revised: LocalBenchmarkMetricEvidenceResponse,
    },
    Removed {
        metric: contextlab_evaluation::MetricKind,
        baseline: LocalBenchmarkMetricEvidenceResponse,
    },
    Modified {
        metric: contextlab_evaluation::MetricKind,
        baseline: LocalBenchmarkMetricEvidenceResponse,
        revised: LocalBenchmarkMetricEvidenceResponse,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalBenchmarkComparabilityResponse {
    evaluator_key: String,
    evaluator_version: String,
    fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct LocalBenchmarkMetricEvidenceResponse {
    metric: contextlab_evaluation::MetricKind,
    threshold_direction: contextlab_evaluation::ThresholdDirection,
    threshold_value: f64,
    observed: Option<f64>,
    sample_count: usize,
    required_sample_count: usize,
    has_complete_coverage: bool,
    outcome: contextlab_evaluation::RegressionCheckStatus,
}

/// One component's exact private descriptor and effective immutable body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LocalContextLifecycleComponentResponse {
    component_id: String,
    component_kind: ContextComponentKind,
    name: String,
    metadata: Value,
    content: String,
    content_hash: String,
    creation_commit_id: String,
    content_commit_id: String,
}

impl LocalContextLifecycleOperationRequest {
    fn into_lifecycle_operation(self) -> Result<ContextLifecycleOperation, ApiError> {
        match self {
            Self::Initialize {} => Ok(ContextLifecycleOperation::Initialize),
            Self::Create {
                component_kind,
                name,
                metadata,
                content,
            } => Ok(ContextLifecycleOperation::Create {
                component_kind,
                name,
                metadata,
                content,
            }),
            Self::Update {
                component_id,
                content,
            } => Ok(ContextLifecycleOperation::Update {
                component_id: parse_component_id(&component_id)?,
                content,
            }),
            Self::UpdateDescriptor {
                component_id,
                name,
                metadata,
            } => Ok(ContextLifecycleOperation::UpdateDescriptor {
                component_id: parse_component_id(&component_id)?,
                name,
                metadata,
            }),
            Self::UpdateMetadata { metadata } => {
                Ok(ContextLifecycleOperation::UpdateMetadata { metadata })
            }
            Self::Remove { component_id } => Ok(ContextLifecycleOperation::Remove {
                component_id: parse_component_id(&component_id)?,
            }),
            Self::AddUsesRelationship {
                source_component_id,
                target_component_id,
            } => Ok(ContextLifecycleOperation::AddUsesRelationship {
                source_component_id: parse_component_id(&source_component_id)?,
                target_component_id: parse_component_id(&target_component_id)?,
            }),
            Self::RemoveUsesRelationship {
                source_component_id,
                target_component_id,
            } => Ok(ContextLifecycleOperation::RemoveUsesRelationship {
                source_component_id: parse_component_id(&source_component_id)?,
                target_component_id: parse_component_id(&target_component_id)?,
            }),
        }
    }
}

impl LocalContextLifecycleStateResponse {
    fn from_state(
        context_id: ContextId,
        commit_id: CommitId,
        state: ContextLifecycleStateAtCommit,
    ) -> Self {
        Self {
            schema_version: LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1,
            context_id: context_id.to_string(),
            commit_id: commit_id.to_string(),
            metadata: state.metadata().cloned(),
            components: state
                .components()
                .iter()
                .map(|component_state| {
                    let descriptor = component_state.state();
                    let component = descriptor.component();
                    let content = component_state.content();
                    LocalContextLifecycleComponentResponse {
                        component_id: component.id().to_string(),
                        component_kind: component.kind(),
                        name: component.name().as_str().to_owned(),
                        metadata: descriptor.metadata().clone(),
                        content: content.content().as_str().to_owned(),
                        content_hash: content.resulting_content_hash().as_str().to_owned(),
                        creation_commit_id: descriptor.creation_commit_id().to_string(),
                        content_commit_id: descriptor.content_commit_id().to_string(),
                    }
                })
                .collect(),
            graph_snapshot: state.graph_snapshot().clone(),
        }
    }
}

impl LocalBenchmarkDecisionResponse {
    fn from_evidence(
        evidence: BenchmarkDecisionEvidence,
        definition: BenchmarkDecisionDefinitionSummary,
    ) -> Self {
        let comparability = evidence.comparability();
        Self {
            project_id: evidence.project_id().to_string(),
            context_id: evidence.context_id().to_string(),
            commit_id: evidence.context_commit_id().to_string(),
            decision_id: evidence.decision_id().to_string(),
            suite_id: evidence.suite_id().to_string(),
            dataset_ids: evidence
                .dataset_ids()
                .iter()
                .map(ToString::to_string)
                .collect(),
            definition: LocalBenchmarkDecisionDefinitionResponse {
                suite: LocalBenchmarkDecisionSuiteResponse {
                    id: definition.suite().id().to_string(),
                    name: definition.suite().name().to_owned(),
                    thresholds: definition
                        .suite()
                        .thresholds()
                        .iter()
                        .map(|threshold| LocalBenchmarkThresholdResponse {
                            metric: threshold.metric(),
                            direction: threshold.direction(),
                            value: threshold.value(),
                        })
                        .collect(),
                },
                datasets: definition
                    .datasets()
                    .iter()
                    .map(|dataset| LocalBenchmarkDecisionDatasetResponse {
                        id: dataset.id().to_string(),
                        name: dataset.name().to_owned(),
                        case_count: dataset.case_count(),
                    })
                    .collect(),
            },
            run_ids: evidence
                .run_ids()
                .iter()
                .map(|run_id| run_id.as_uuid().to_string())
                .collect(),
            comparability: LocalBenchmarkComparabilityResponse {
                evaluator_key: comparability.evaluator_key().to_owned(),
                evaluator_version: comparability.evaluator_version().to_owned(),
                fingerprint: comparability.fingerprint().to_owned(),
            },
            evidence_digest: evidence.evidence_digest().to_owned(),
            status: evidence.status(),
            recorded_at: evidence.recorded_at(),
            metrics: evidence
                .metric_results()
                .iter()
                .map(|metric| LocalBenchmarkMetricEvidenceResponse {
                    metric: metric.metric(),
                    threshold_direction: metric.threshold().direction(),
                    threshold_value: metric.threshold().value(),
                    observed: metric.observed(),
                    sample_count: metric.sample_count(),
                    required_sample_count: metric.required_sample_count(),
                    has_complete_coverage: metric.has_complete_coverage(),
                    outcome: metric.outcome(),
                })
                .collect(),
        }
    }
}

impl LocalBenchmarkDecisionListResponse {
    fn from_summaries(
        project_id: contextlab_context_core::ProjectId,
        context_id: contextlab_context_core::ContextId,
        commit_id: contextlab_versioning::CommitId,
        decisions: Vec<BenchmarkDecisionDiscoverySummary>,
    ) -> Self {
        Self {
            schema_version: "contextlab.local-benchmark-decision-list.v1",
            project_id: project_id.to_string(),
            context_id: context_id.to_string(),
            commit_id: commit_id.to_string(),
            decisions: decisions
                .into_iter()
                .map(|summary| LocalBenchmarkDecisionListItemResponse {
                    decision_id: summary.decision_id().to_string(),
                    suite: LocalBenchmarkDecisionListSuiteResponse {
                        id: summary.suite().id().to_string(),
                        name: summary.suite().name().to_owned(),
                    },
                    datasets: summary
                        .datasets()
                        .iter()
                        .map(|dataset| LocalBenchmarkDecisionListDatasetResponse {
                            id: dataset.id().to_string(),
                            name: dataset.name().to_owned(),
                            case_count: dataset.case_count(),
                        })
                        .collect(),
                    status: summary.status(),
                    recorded_at: summary.recorded_at(),
                    run_count: summary.run_count(),
                })
                .collect(),
        }
    }
}

impl LocalBenchmarkDecisionRunDetailsResponse {
    fn from_summary(
        evidence: &BenchmarkDecisionEvidence,
        summary: BenchmarkDecisionRunDetails,
    ) -> Self {
        Self {
            project_id: evidence.project_id().to_string(),
            context_id: evidence.context_id().to_string(),
            commit_id: evidence.context_commit_id().to_string(),
            decision_id: evidence.decision_id().to_string(),
            runs: summary
                .runs()
                .iter()
                .map(|run| LocalBenchmarkDecisionRunResponse {
                    run_id: run.id().as_uuid().to_string(),
                    model_version: run.model_version().to_owned(),
                    temperature: run.temperature(),
                    metrics: run
                        .measurements()
                        .iter()
                        .map(|measurement| LocalBenchmarkDecisionRunMetricResponse {
                            metric: measurement.metric(),
                            value: measurement.value(),
                        })
                        .collect(),
                    executed_at: run.executed_at(),
                })
                .collect(),
        }
    }
}

impl LocalBenchmarkDecisionDiffResponse {
    fn from_diff(
        project_id: ProjectId,
        context_id: ContextId,
        baseline: BenchmarkDecisionComparisonScope,
        revised: BenchmarkDecisionComparisonScope,
        diff: contextlab_evaluation::BenchmarkDecisionDiff,
    ) -> Self {
        Self {
            project_id: project_id.to_string(),
            context_id: context_id.to_string(),
            baseline: LocalBenchmarkDecisionScopeResponse {
                commit_id: baseline.context_commit_id().to_string(),
                decision_id: baseline.decision_id().to_string(),
            },
            revised: LocalBenchmarkDecisionScopeResponse {
                commit_id: revised.context_commit_id().to_string(),
                decision_id: revised.decision_id().to_string(),
            },
            status_change: diff.status_change().map(|(baseline, revised)| {
                LocalBenchmarkDecisionStatusChangeResponse { baseline, revised }
            }),
            metric_changes: diff
                .metric_changes()
                .iter()
                .map(|change| match change {
                    contextlab_evaluation::BenchmarkDecisionMetricDiff::Added {
                        metric,
                        revised,
                    } => LocalBenchmarkDecisionMetricDiffResponse::Added {
                        metric: *metric,
                        revised: LocalBenchmarkMetricEvidenceResponse::from_decision_input(revised),
                    },
                    contextlab_evaluation::BenchmarkDecisionMetricDiff::Removed {
                        metric,
                        baseline,
                    } => LocalBenchmarkDecisionMetricDiffResponse::Removed {
                        metric: *metric,
                        baseline: LocalBenchmarkMetricEvidenceResponse::from_decision_input(
                            baseline,
                        ),
                    },
                    contextlab_evaluation::BenchmarkDecisionMetricDiff::Modified {
                        metric,
                        baseline,
                        revised,
                    } => LocalBenchmarkDecisionMetricDiffResponse::Modified {
                        metric: *metric,
                        baseline: LocalBenchmarkMetricEvidenceResponse::from_decision_input(
                            baseline,
                        ),
                        revised: LocalBenchmarkMetricEvidenceResponse::from_decision_input(revised),
                    },
                })
                .collect(),
        }
    }
}

impl LocalBenchmarkMetricEvidenceResponse {
    fn from_decision_input(input: &contextlab_evaluation::BenchmarkDecisionMetricInput) -> Self {
        Self {
            metric: input.metric(),
            threshold_direction: input.threshold().direction(),
            threshold_value: input.threshold().value(),
            observed: input.observed(),
            sample_count: input.sample_count(),
            required_sample_count: input.required_sample_count(),
            has_complete_coverage: input.has_complete_coverage(),
            outcome: input.outcome(),
        }
    }
}

impl GraphSnapshotRequest {
    fn into_context_graph(self) -> Result<ContextGraph, GraphValidationError> {
        let mut graph = ContextGraph::new();

        for node in self.nodes {
            graph.add_node(GraphNode::new(node.id, node.kind, node.label)?)?;
        }

        for edge in self.edges {
            graph.add_edge(GraphEdge::new(edge.source, edge.target, edge.kind)?)?;
        }

        Ok(graph)
    }
}

/// Structured graph comparison response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphDiffResponse {
    added_nodes: Vec<GraphNodeResponse>,
    removed_nodes: Vec<GraphNodeResponse>,
    modified_nodes: Vec<GraphNodeChangeResponse>,
    added_edges: Vec<GraphEdgeResponse>,
    removed_edges: Vec<GraphEdgeResponse>,
}

/// Version-backed graph comparison response for two Context commits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitGraphDiffResponse {
    context_id: String,
    pair_witness: CommitGraphDiffPairWitness,
    original: CommitGraphSnapshotReference,
    revised: CommitGraphSnapshotReference,
    diff: GraphDiffResponse,
}

/// Server-owned identity for the ordered pair used by a local graph review.
///
/// The witness is derived from durable snapshot ownership after authorization;
/// callers cannot supply or widen any of these identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CommitGraphDiffPairWitness {
    schema_version: u16,
    project_id: String,
    context_id: String,
    baseline_commit_id: String,
    revised_commit_id: String,
}

impl CommitGraphDiffPairWitness {
    fn from_snapshots(source: &CommitGraphSnapshot, target: &CommitGraphSnapshot) -> Self {
        Self {
            schema_version: 1,
            project_id: source.project_id().to_string(),
            context_id: source.context_id().to_string(),
            baseline_commit_id: source.commit_id().to_string(),
            revised_commit_id: target.commit_id().to_string(),
        }
    }
}

/// Immutable snapshot metadata included with a version-backed graph diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct CommitGraphSnapshotReference {
    commit_id: String,
    captured_at: String,
    schema_version: u16,
}

impl From<&CommitGraphSnapshot> for CommitGraphSnapshotReference {
    fn from(snapshot: &CommitGraphSnapshot) -> Self {
        Self {
            commit_id: snapshot.commit_id().to_string(),
            captured_at: snapshot.captured_at().to_rfc3339(),
            schema_version: snapshot.schema_version(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GraphNodeResponse {
    id: String,
    kind: GraphNodeKind,
    label: String,
}

impl From<&GraphNode> for GraphNodeResponse {
    fn from(node: &GraphNode) -> Self {
        Self {
            id: node.id().as_str().to_owned(),
            kind: node.kind(),
            label: node.label().as_str().to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GraphNodeChangeResponse {
    node_id: String,
    original_kind: GraphNodeKind,
    revised_kind: GraphNodeKind,
    original_label: String,
    revised_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct GraphEdgeResponse {
    source: String,
    target: String,
    kind: GraphEdgeKind,
}

impl From<&GraphEdge> for GraphEdgeResponse {
    fn from(edge: &GraphEdge) -> Self {
        Self {
            source: edge.source().as_str().to_owned(),
            target: edge.target().as_str().to_owned(),
            kind: edge.kind(),
        }
    }
}

impl From<GraphDiff> for GraphDiffResponse {
    fn from(diff: GraphDiff) -> Self {
        Self {
            added_nodes: diff
                .added_nodes()
                .iter()
                .map(GraphNodeResponse::from)
                .collect(),
            removed_nodes: diff
                .removed_nodes()
                .iter()
                .map(GraphNodeResponse::from)
                .collect(),
            modified_nodes: diff
                .modified_nodes()
                .iter()
                .map(|node| GraphNodeChangeResponse {
                    node_id: node.node_id().to_owned(),
                    original_kind: node.original_kind(),
                    revised_kind: node.revised_kind(),
                    original_label: node.original_label().to_owned(),
                    revised_label: node.revised_label().to_owned(),
                })
                .collect(),
            added_edges: diff
                .added_edges()
                .iter()
                .map(GraphEdgeResponse::from)
                .collect(),
            removed_edges: diff
                .removed_edges()
                .iter()
                .map(GraphEdgeResponse::from)
                .collect(),
        }
    }
}

/// Structured API error response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorResponse {
    error: &'static str,
    message: String,
}

/// API route errors.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Storage repository failed.
    #[error(transparent)]
    StorageRepository(#[from] StorageRepositoryError),
    /// Projection into graph contract failed.
    #[error(transparent)]
    StorageProjection(#[from] StorageProjectionError),
    /// Request query parameters are invalid.
    #[error(transparent)]
    BadRequest(#[from] RequestQueryError),
    /// Graph comparison snapshot failed validation.
    #[error(transparent)]
    InvalidGraphSnapshot(#[from] GraphValidationError),
    /// Graph comparison request body failed JSON validation.
    #[error("invalid graph snapshot request: {0}")]
    InvalidGraphSnapshotRequest(String),
    /// Version-backed graph diff query parameters are missing or empty.
    #[error("invalid commit graph diff query: {0}")]
    InvalidCommitGraphDiffQuery(String),
    /// A known Context commit has no materialized graph snapshot.
    #[error("commit graph snapshot is not materialized: {context_id}/{commit_id}")]
    CommitGraphSnapshotMissing {
        /// Context identifier.
        context_id: String,
        /// Commit identifier.
        commit_id: String,
    },
    /// The private version-backed graph review contract could not produce a complete result.
    #[error("version-backed Context Graph diff review is unavailable")]
    CommitGraphDiffReviewUnavailable,
    /// The private persisted Context diff review repository is unavailable for this runtime.
    #[error("persisted Context diff review is unavailable")]
    ContextDiffReviewUnavailable,
    /// The private Context merge review query is invalid.
    #[error("invalid Context merge review query: {0}")]
    InvalidContextMergeReviewQuery(String),
    /// The private Context merge review could not produce a coherent input.
    #[error("Context merge review input is unavailable")]
    ContextMergeReviewUnavailable,
    /// The private Context merge review storage boundary is unavailable.
    #[error("Context merge review storage is unavailable")]
    ContextMergeReviewStorageUnavailable,
    /// A required persisted Context diff input is missing at its exact commit scope.
    #[error("persisted Context diff input is missing: {project_id}/{context_id}/{commit_id}")]
    ContextDiffReviewSnapshotMissing {
        /// Project identifier.
        project_id: String,
        /// Context identifier.
        context_id: String,
        /// Commit identifier.
        commit_id: String,
    },
    /// No authenticated identity was supplied.
    #[error("authentication is required")]
    AuthenticationRequired,
    /// The supplied identity could not be verified.
    #[error("authentication failed")]
    AuthenticationFailed,
    /// The authenticated principal lacks Context write permission.
    #[error("context write permission is forbidden")]
    ContextWriteForbidden,
    /// The authenticated principal lacks Context read permission.
    #[error("context read permission is forbidden")]
    ContextReadForbidden,
    /// Authorization state could not be read reliably.
    #[error("authorization state is unavailable")]
    AuthorizationUnavailable,
    /// Authorization decision audit storage could not accept the record.
    #[error("authorization audit sink is unavailable")]
    AuthorizationAuditUnavailable,
    /// The authenticated principal exhausted the protected-route quota.
    #[error("protected route rate limit exceeded")]
    RateLimitExceeded {
        /// Whole seconds until the next request can be accepted.
        retry_after_seconds: u64,
    },
    /// Protected-route rate-limit state could not be checked reliably.
    #[error("protected route rate limiter is unavailable")]
    RateLimitUnavailable,
    /// The protected commit request is invalid.
    #[error("invalid context commit request: {0}")]
    InvalidContextCommitRequest(String),
    /// The private local lifecycle route is unavailable for this runtime composition.
    #[error("local context lifecycle is unavailable")]
    ContextLifecycleUnavailable,
    /// The private local branch-head route is unavailable for this runtime composition.
    #[error("local Context branch heads are unavailable")]
    ContextBranchHeadsUnavailable,
    /// A lifecycle operation reached storage and must retain its stable status without exposing internals.
    #[error("context lifecycle storage operation failed")]
    ContextLifecycleStorage(StorageRepositoryError),
    /// The local Context lifecycle request is invalid or violates lifecycle invariants.
    #[error("invalid context lifecycle request: {0}")]
    InvalidContextLifecycleRequest(String),
    /// The local Context branch-head request is invalid.
    #[error("invalid Context branch-head request: {0}")]
    InvalidContextBranchHeadsRequest(String),
    /// The requested materialized Context history cannot supply a coherent lifecycle state.
    #[error("context lifecycle state conflict: {0}")]
    ContextLifecycleStateConflict(String),
    /// The private benchmark-evidence route is unavailable for this runtime composition.
    #[error("benchmark evidence inspection is unavailable")]
    BenchmarkEvidenceUnavailable,
    /// The requested private benchmark decision does not exist in its exact scope.
    #[error("benchmark decision was not found")]
    BenchmarkEvidenceNotFound,
    /// A benchmark-evidence storage operation failed without exposing internals.
    #[error("benchmark evidence storage operation failed")]
    BenchmarkEvidenceStorage(StorageRepositoryError),
    /// Two sealed benchmark decisions cannot be safely compared.
    #[error("benchmark decisions cannot be compared")]
    BenchmarkDecisionComparisonUnavailable,
    /// The private benchmark-workspace reader is unavailable for this runtime composition.
    #[error("benchmark workspace inspection is unavailable")]
    BenchmarkWorkspaceUnavailable,
    /// The benchmark-workspace read request has invalid exact-scope parameters.
    #[error("invalid benchmark workspace request: {0}")]
    InvalidBenchmarkWorkspaceRequest(String),
    /// A projection source does not exist at the exact requested scope.
    #[error("benchmark workspace projection was not found")]
    BenchmarkWorkspaceNotFound,
    /// Benchmark-workspace persistence failed without exposing source internals.
    #[error("benchmark workspace projection storage operation failed")]
    BenchmarkWorkspaceStorage(BenchmarkWorkspaceProjectionPersistenceError),
    /// The private benchmark-definition binding reader is unavailable for this runtime composition.
    #[error("benchmark definition binding inspection is unavailable")]
    BenchmarkDefinitionBindingUnavailable,
    /// The benchmark-definition binding read request has invalid exact-scope parameters.
    #[error("invalid benchmark definition binding request: {0}")]
    InvalidBenchmarkDefinitionBindingRequest(String),
    /// Benchmark-definition binding storage failed without exposing internals.
    #[error("benchmark definition binding storage operation failed")]
    BenchmarkDefinitionBindingStorage(StorageRepositoryError),
    /// The private Workflow source-binding repository is unavailable.
    #[error("workflow Context binding inspection is unavailable")]
    WorkflowContextBindingsUnavailable,
    /// A private Workflow source-binding storage operation failed.
    #[error("workflow Context binding storage operation failed")]
    WorkflowContextBindingsStorage(StorageRepositoryError),
    /// The private Knowledge/Memory projection repository is unavailable.
    #[error("Knowledge/Memory projection inspection is unavailable")]
    KnowledgeMemoryProjectionUnavailable,
    /// The private Knowledge/Memory projection violated its exact-scope or schema contract.
    #[error("Knowledge/Memory projection is inconsistent")]
    KnowledgeMemoryProjectionInvalid,
    /// The private Plugin/MCP capability projection is unavailable.
    #[error("Plugin/MCP capability availability is unavailable")]
    PluginCapabilityAvailabilityUnavailable,
    /// The private Plugin/MCP capability projection violated its scope contract.
    #[error("Plugin/MCP capability availability is inconsistent")]
    PluginCapabilityAvailabilityInvalid,
    /// The private Workflow execution status repository is unavailable.
    #[error("Workflow execution status inspection is unavailable")]
    WorkflowExecutionStatusUnavailable,
    /// The requested Workflow execution status was not found in its exact scope.
    #[error("Workflow execution status was not found")]
    WorkflowExecutionStatusNotFound,
    /// The private Workflow execution status projection violated its exact-scope contract.
    #[error("Workflow execution status is inconsistent")]
    WorkflowExecutionStatusInvalid,
    /// The Workflow execution status request contains an invalid identifier.
    #[error("invalid Workflow execution status request: {0}")]
    InvalidWorkflowExecutionStatusRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error, message, retry_after_seconds) = match self {
            Self::StorageRepository(error) => (
                storage_status(&error),
                storage_error_code(&error),
                error.to_string(),
                None,
            ),
            Self::StorageProjection(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "graph_projection_failed",
                error.to_string(),
                None,
            ),
            Self::BadRequest(error) => (
                StatusCode::BAD_REQUEST,
                error.code(),
                error.to_string(),
                None,
            ),
            Self::InvalidGraphSnapshot(error) => (
                StatusCode::BAD_REQUEST,
                "invalid_graph_snapshot",
                error.to_string(),
                None,
            ),
            Self::InvalidGraphSnapshotRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_graph_snapshot",
                message,
                None,
            ),
            Self::InvalidCommitGraphDiffQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_commit_graph_diff_query",
                message,
                None,
            ),
            Self::CommitGraphSnapshotMissing {
                context_id,
                commit_id,
            } => (
                StatusCode::CONFLICT,
                "commit_graph_snapshot_missing",
                format!("commit graph snapshot is not materialized: {context_id}/{commit_id}"),
                None,
            ),
            Self::CommitGraphDiffReviewUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "commit_graph_diff_unavailable",
                "version-backed Context Graph diff review is unavailable".to_owned(),
                None,
            ),
            Self::ContextDiffReviewUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "context_diff_review_unavailable",
                "persisted Context diff review is unavailable".to_owned(),
                None,
            ),
            Self::ContextDiffReviewSnapshotMissing {
                project_id,
                context_id,
                commit_id,
            } => (
                StatusCode::CONFLICT,
                "context_diff_review_snapshot_missing",
                format!(
                    "persisted Context diff input is not materialized: {project_id}/{context_id}/{commit_id}"
                ),
                None,
            ),
            Self::InvalidContextMergeReviewQuery(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_context_merge_review_query",
                message,
                None,
            ),
            Self::ContextMergeReviewUnavailable => (
                StatusCode::CONFLICT,
                "context_merge_review_unavailable",
                "Context merge review input is unavailable".to_owned(),
                None,
            ),
            Self::ContextMergeReviewStorageUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "context_merge_review_storage_unavailable",
                "Context merge review storage is unavailable".to_owned(),
                None,
            ),
            Self::AuthenticationRequired => (
                StatusCode::UNAUTHORIZED,
                "authentication_required",
                "authentication is required".to_owned(),
                None,
            ),
            Self::AuthenticationFailed => (
                StatusCode::UNAUTHORIZED,
                "authentication_failed",
                "authentication failed".to_owned(),
                None,
            ),
            Self::ContextWriteForbidden => (
                StatusCode::FORBIDDEN,
                "context_write_forbidden",
                "context write permission is forbidden".to_owned(),
                None,
            ),
            Self::ContextReadForbidden => (
                StatusCode::FORBIDDEN,
                "context_read_forbidden",
                "context read permission is forbidden".to_owned(),
                None,
            ),
            Self::AuthorizationUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "authorization_unavailable",
                "authorization state is unavailable".to_owned(),
                None,
            ),
            Self::AuthorizationAuditUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "authorization_audit_unavailable",
                "authorization audit storage is unavailable".to_owned(),
                None,
            ),
            Self::RateLimitExceeded {
                retry_after_seconds,
            } => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limit_exceeded",
                "protected route rate limit exceeded".to_owned(),
                Some(retry_after_seconds.max(1)),
            ),
            Self::RateLimitUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "rate_limit_unavailable",
                "protected route rate limiter is unavailable".to_owned(),
                None,
            ),
            Self::InvalidContextCommitRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_context_commit_request",
                message,
                None,
            ),
            Self::ContextLifecycleUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "context_lifecycle_unavailable",
                "local context lifecycle is unavailable".to_owned(),
                None,
            ),
            Self::ContextBranchHeadsUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "context_branch_heads_unavailable",
                "local Context branch heads are unavailable".to_owned(),
                None,
            ),
            Self::ContextLifecycleStorage(error) => {
                let status = storage_status(&error);
                let (error, message) = if status == StatusCode::INTERNAL_SERVER_ERROR {
                    (
                        "context_lifecycle_storage_error",
                        "context lifecycle storage is unavailable".to_owned(),
                    )
                } else {
                    (
                        storage_error_code(&error),
                        "context lifecycle operation could not be completed".to_owned(),
                    )
                };

                (status, error, message, None)
            }
            Self::InvalidContextLifecycleRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_context_lifecycle_request",
                message,
                None,
            ),
            Self::InvalidContextBranchHeadsRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_context_branch_heads_request",
                message,
                None,
            ),
            Self::ContextLifecycleStateConflict(message) => (
                StatusCode::CONFLICT,
                "context_lifecycle_state_conflict",
                message,
                None,
            ),
            Self::BenchmarkEvidenceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "benchmark_evidence_unavailable",
                "benchmark evidence inspection is unavailable".to_owned(),
                None,
            ),
            Self::BenchmarkEvidenceNotFound => (
                StatusCode::NOT_FOUND,
                "benchmark_evidence_not_found",
                "benchmark decision was not found".to_owned(),
                None,
            ),
            Self::BenchmarkEvidenceStorage(error) => {
                let status = storage_status(&error);
                let (error, message) = if status == StatusCode::INTERNAL_SERVER_ERROR {
                    (
                        "benchmark_evidence_unavailable",
                        "benchmark evidence inspection is unavailable".to_owned(),
                    )
                } else {
                    (
                        "benchmark_evidence_not_found",
                        "benchmark decision was not found".to_owned(),
                    )
                };
                (status, error, message, None)
            }
            Self::BenchmarkDecisionComparisonUnavailable => (
                StatusCode::CONFLICT,
                "benchmark_decision_comparison_unavailable",
                "benchmark decisions cannot be compared".to_owned(),
                None,
            ),
            Self::BenchmarkWorkspaceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "benchmark_workspace_unavailable",
                "benchmark workspace inspection is unavailable".to_owned(),
                None,
            ),
            Self::InvalidBenchmarkWorkspaceRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_benchmark_workspace_request",
                message,
                None,
            ),
            Self::BenchmarkWorkspaceNotFound => (
                StatusCode::NOT_FOUND,
                "benchmark_workspace_not_found",
                "benchmark workspace projection was not found".to_owned(),
                None,
            ),
            Self::BenchmarkWorkspaceStorage(error) => match error {
                BenchmarkWorkspaceProjectionPersistenceError::ComparisonScopeMismatch
                | BenchmarkWorkspaceProjectionPersistenceError::ComparisonPlanMismatch
                | BenchmarkWorkspaceProjectionPersistenceError::Projection(_) => (
                    StatusCode::CONFLICT,
                    "benchmark_workspace_comparison_unavailable",
                    "benchmark workspace projections cannot be compared".to_owned(),
                    None,
                ),
                BenchmarkWorkspaceProjectionPersistenceError::ReceiptConflict { .. }
                | BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid => (
                    StatusCode::CONFLICT,
                    "benchmark_workspace_source_conflict",
                    "benchmark workspace projection source is inconsistent".to_owned(),
                    None,
                ),
                BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { .. } => (
                    StatusCode::NOT_FOUND,
                    "benchmark_workspace_not_found",
                    "benchmark workspace projection was not found".to_owned(),
                    None,
                ),
                BenchmarkWorkspaceProjectionPersistenceError::RepositoryUnavailable => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "benchmark_workspace_unavailable",
                    "benchmark workspace inspection is unavailable".to_owned(),
                    None,
                ),
            },
            Self::BenchmarkDefinitionBindingUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "benchmark_definition_binding_unavailable",
                "benchmark definition binding inspection is unavailable".to_owned(),
                None,
            ),
            Self::InvalidBenchmarkDefinitionBindingRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_benchmark_definition_binding_request",
                message,
                None,
            ),
            Self::BenchmarkDefinitionBindingStorage(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "benchmark_definition_binding_unavailable",
                "benchmark definition binding inspection is unavailable".to_owned(),
                None,
            ),
            Self::WorkflowContextBindingsUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "workflow_bindings_unavailable",
                "workflow Context binding inspection is unavailable".to_owned(),
                None,
            ),
            Self::WorkflowContextBindingsStorage(error) => {
                let status = storage_status(&error);
                let (error, message) = if status == StatusCode::INTERNAL_SERVER_ERROR {
                    (
                        "workflow_bindings_unavailable",
                        "workflow Context binding inspection is unavailable".to_owned(),
                    )
                } else {
                    (
                        storage_error_code(&error),
                        "workflow Context binding inspection could not be completed".to_owned(),
                    )
                };
                (status, error, message, None)
            }
            Self::KnowledgeMemoryProjectionUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "knowledge_memory_projection_unavailable",
                "Knowledge/Memory projection inspection is unavailable".to_owned(),
                None,
            ),
            Self::KnowledgeMemoryProjectionInvalid => (
                StatusCode::CONFLICT,
                "knowledge_memory_projection_invalid",
                "Knowledge/Memory projection is inconsistent".to_owned(),
                None,
            ),
            Self::PluginCapabilityAvailabilityUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "plugin_capability_availability_unavailable",
                "Plugin/MCP capability availability is unavailable".to_owned(),
                None,
            ),
            Self::PluginCapabilityAvailabilityInvalid => (
                StatusCode::CONFLICT,
                "plugin_capability_availability_invalid",
                "Plugin/MCP capability availability is inconsistent".to_owned(),
                None,
            ),
            Self::WorkflowExecutionStatusUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "workflow_execution_status_unavailable",
                "Workflow execution status inspection is unavailable".to_owned(),
                None,
            ),
            Self::WorkflowExecutionStatusNotFound => (
                StatusCode::NOT_FOUND,
                "workflow_execution_status_not_found",
                "Workflow execution status was not found".to_owned(),
                None,
            ),
            Self::WorkflowExecutionStatusInvalid => (
                StatusCode::CONFLICT,
                "workflow_execution_status_invalid",
                "Workflow execution status is inconsistent".to_owned(),
                None,
            ),
            Self::InvalidWorkflowExecutionStatusRequest(message) => (
                StatusCode::BAD_REQUEST,
                "invalid_workflow_execution_status_request",
                message,
                None,
            ),
        };

        let mut response = (status, Json(ErrorResponse { error, message })).into_response();
        if let Some(retry_after_seconds) = retry_after_seconds {
            let value = HeaderValue::from_str(&retry_after_seconds.to_string())
                .expect("retry-after seconds are a valid header value");
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
        response
    }
}

/// Authenticates guarded Context-commit writes before they enter the protected route catalog.
pub(crate) async fn authenticate_context_commit_write_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::ContextCommitWrite,
    )
    .await
}

/// Authenticates private lifecycle-state reads before they enter the protected route catalog.
pub(crate) async fn authenticate_context_lifecycle_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::ContextLifecycleRead,
    )
    .await
}

/// Authenticates private durable Context branch-head discovery before its quota check.
pub(crate) async fn authenticate_context_branch_head_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::ContextBranchHeadRead,
    )
    .await
}

/// Authenticates private commit-associated Context Graph diff reads before quota enforcement.
pub(crate) async fn authenticate_context_commit_graph_diff_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::ContextCommitGraphDiffRead,
    )
    .await
}

/// Authenticates the private Knowledge/Memory projection read before its quota check.
pub(crate) async fn authenticate_knowledge_memory_projection_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::KnowledgeMemoryProjectionRead,
    )
    .await
}

/// Authenticates private Workflow execution status reads before their quota check.
pub(crate) async fn authenticate_workflow_execution_status_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::WorkflowExecutionStatusRead,
    )
    .await
}

/// Authenticates private benchmark-definition authoring before its dedicated write quota.
pub(crate) async fn authenticate_benchmark_definition_authoring_write_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::BenchmarkDefinitionAuthoringWrite,
    )
    .await
}

/// Authenticates private benchmark execution before its dedicated write quota.
pub(crate) async fn authenticate_benchmark_execution_write_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::BenchmarkExecutionWrite,
    )
    .await
}

/// Authenticates private benchmark-definition binding reads before their dedicated quota.
pub(crate) async fn authenticate_benchmark_definition_binding_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::BenchmarkDefinitionBindingRead,
    )
    .await
}

/// Authenticates private benchmark-decision reads before their protected route catalog.
pub(crate) async fn authenticate_benchmark_evidence_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::BenchmarkDecisionRead,
    )
    .await
}

/// Authenticates private benchmark-decision comparison reads before their protected route catalog.
pub(crate) async fn authenticate_benchmark_decision_diff_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::BenchmarkDecisionDiffRead,
    )
    .await
}

/// Authenticates private benchmark-workspace reads before their protected route catalog.
pub(crate) async fn authenticate_benchmark_workspace_read_request(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    authenticate_request_for_operation(
        state,
        request,
        next,
        ProtectedRouteOperation::BenchmarkWorkspaceRead,
    )
    .await
}

/// Prevents protected local read responses, including failures, from entering shared caches.
pub(crate) async fn private_no_store_response(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
}

async fn authenticate_request_for_operation(
    state: AppState,
    mut request: Request,
    next: Next,
    operation: ProtectedRouteOperation,
) -> Response {
    let authorization_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    let principal = match state.authenticate_principal(authorization_header).await {
        Ok(principal) => principal,
        Err(error) => return authentication_error(error).into_response(),
    };

    let rate_limit_key = ProtectedRouteRateLimitKey::new(principal.identity().clone(), operation);
    match state.check_protected_rate_limit(rate_limit_key).await {
        Ok(RateLimitDecision::Allowed) => {}
        Ok(RateLimitDecision::Rejected {
            retry_after_seconds,
        }) => {
            return ApiError::RateLimitExceeded {
                retry_after_seconds,
            }
            .into_response();
        }
        Err(_) => return ApiError::RateLimitUnavailable.into_response(),
    }

    request.extensions_mut().insert(principal);
    next.run(request).await
}

fn authentication_error(error: contextlab_auth::AuthenticationError) -> ApiError {
    match error {
        contextlab_auth::AuthenticationError::MissingCredentials => {
            ApiError::AuthenticationRequired
        }
        _ => ApiError::AuthenticationFailed,
    }
}

/// Query parsing errors.
#[derive(Debug, Error)]
pub enum RequestQueryError {
    /// Sort value is unsupported.
    #[error("invalid sort parameter: {0}")]
    WorkspaceSort(#[from] WorkspaceSortParseError),
    /// Project sort value is unsupported.
    #[error("invalid sort parameter: {0}")]
    ProjectSort(#[from] ProjectSortParseError),
    /// Experiment sort value is unsupported.
    #[error("invalid sort parameter: {0}")]
    ExperimentSort(#[from] ExperimentSortParseError),
    /// Context sort value is unsupported.
    #[error("invalid sort parameter: {0}")]
    ContextSort(#[from] ContextSortParseError),
    /// Commit sort value is unsupported.
    #[error("invalid sort parameter: {0}")]
    CommitSort(#[from] CommitSortParseError),
    /// Component kind value is unsupported.
    #[error("invalid component kind parameter: {0}")]
    ComponentKind(#[from] StoredComponentKindParseError),
    /// Component sort value is unsupported.
    #[error("invalid sort parameter: {0}")]
    ComponentSort(#[from] ComponentSortParseError),
    /// Evaluation run sort value is unsupported.
    #[error("invalid sort parameter: {0}")]
    EvaluationRunSort(#[from] EvaluationRunSortParseError),
}

impl RequestQueryError {
    fn code(&self) -> &'static str {
        match self {
            Self::WorkspaceSort(_) => "invalid_workspace_sort",
            Self::ProjectSort(_) => "invalid_project_sort",
            Self::ExperimentSort(_) => "invalid_experiment_sort",
            Self::ContextSort(_) => "invalid_context_sort",
            Self::CommitSort(_) => "invalid_commit_sort",
            Self::ComponentKind(_) => "invalid_component_kind",
            Self::ComponentSort(_) => "invalid_component_sort",
            Self::EvaluationRunSort(_) => "invalid_evaluation_run_sort",
        }
    }
}

/// Workspace list query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WorkspaceListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    sort: Option<String>,
}

#[cfg(test)]
pub(crate) const DISCOVERY_LIST_QUERY_PARAMETERS: &[&str] = &["page", "per_page", "search", "sort"];

impl TryFrom<WorkspaceListParams> for WorkspaceListQuery {
    type Error = RequestQueryError;

    fn try_from(params: WorkspaceListParams) -> Result<Self, Self::Error> {
        let sort = params
            .sort
            .as_deref()
            .unwrap_or(WorkspaceSort::default().as_query_value())
            .parse::<WorkspaceSort>()?;

        Ok(Self::new(params.page, params.per_page, params.search, sort))
    }
}

/// Workspace-scoped project list query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProjectListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    sort: Option<String>,
}

impl TryFrom<ProjectListParams> for ProjectListQuery {
    type Error = RequestQueryError;

    fn try_from(params: ProjectListParams) -> Result<Self, Self::Error> {
        let sort = params
            .sort
            .as_deref()
            .unwrap_or(ProjectSort::default().as_query_value())
            .parse::<ProjectSort>()?;

        Ok(Self::new(params.page, params.per_page, params.search, sort))
    }
}

/// Project-scoped experiment list query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ExperimentListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    sort: Option<String>,
}

impl TryFrom<ExperimentListParams> for ExperimentListQuery {
    type Error = RequestQueryError;

    fn try_from(params: ExperimentListParams) -> Result<Self, Self::Error> {
        let sort = params
            .sort
            .as_deref()
            .unwrap_or(ExperimentSort::default().as_query_value())
            .parse::<ExperimentSort>()?;

        Ok(Self::new(params.page, params.per_page, params.search, sort))
    }
}

/// Project-scoped context list query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ContextListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    experiment_id: Option<String>,
    sort: Option<String>,
}

#[cfg(test)]
pub(crate) const CONTEXT_LIST_QUERY_PARAMETERS: &[&str] =
    &["page", "per_page", "search", "experiment_id", "sort"];

impl TryFrom<ContextListParams> for ContextListQuery {
    type Error = RequestQueryError;

    fn try_from(params: ContextListParams) -> Result<Self, Self::Error> {
        let sort = params
            .sort
            .as_deref()
            .unwrap_or(ContextSort::default().as_query_value())
            .parse::<ContextSort>()?;

        Ok(Self::new(
            params.page,
            params.per_page,
            params.search,
            params.experiment_id,
            sort,
        ))
    }
}

/// Context-scoped commit list query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CommitListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    branch_name: Option<String>,
    sort: Option<String>,
}

/// Context-scoped commit graph diff query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CommitGraphDiffParams {
    original_commit_id: Option<String>,
    revised_commit_id: Option<String>,
}

/// Exact source and target commits for a persisted Context diff review.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextDiffReviewParams {
    source_commit_id: Option<String>,
    target_commit_id: Option<String>,
}

/// Exact server-owned branch tips for a private Context merge review.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextMergeReviewParams {
    left_commit_id: Option<String>,
    right_commit_id: Option<String>,
}

/// Exact private benchmark decision identities for one read-only comparison.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct BenchmarkDecisionDiffParams {
    baseline_commit_id: String,
    baseline_decision_id: String,
    revised_commit_id: String,
    revised_decision_id: String,
}

/// Optional exact baseline scope for a private benchmark workspace projection.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalBenchmarkWorkspaceParams {
    baseline_commit_id: Option<String>,
    baseline_cohort_id: Option<String>,
}

/// Optional exact baseline decision for a private decision-bound workspace read.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalBenchmarkWorkspaceDecisionParams {
    baseline_commit_id: Option<String>,
    baseline_decision_id: Option<String>,
}

#[cfg(test)]
pub(crate) const COMMIT_LIST_QUERY_PARAMETERS: &[&str] =
    &["page", "per_page", "search", "branch_name", "sort"];

impl TryFrom<CommitListParams> for CommitListQuery {
    type Error = RequestQueryError;

    fn try_from(params: CommitListParams) -> Result<Self, Self::Error> {
        let sort = params
            .sort
            .as_deref()
            .unwrap_or(CommitSort::default().as_query_value())
            .parse::<CommitSort>()?;

        Ok(Self::new(
            params.page,
            params.per_page,
            params.search,
            params.branch_name,
            sort,
        ))
    }
}

/// Context-scoped component list query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ComponentListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    kind: Option<String>,
    sort: Option<String>,
}

#[cfg(test)]
pub(crate) const COMPONENT_LIST_QUERY_PARAMETERS: &[&str] =
    &["page", "per_page", "search", "kind", "sort"];

impl TryFrom<ComponentListParams> for ComponentListQuery {
    type Error = RequestQueryError;

    fn try_from(params: ComponentListParams) -> Result<Self, Self::Error> {
        let sort = params
            .sort
            .as_deref()
            .unwrap_or(ComponentSort::default().as_query_value())
            .parse::<ComponentSort>()?;
        let kind = params
            .kind
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(str::parse::<StoredComponentKind>)
            .transpose()?;

        Ok(Self::new(
            params.page,
            params.per_page,
            params.search,
            kind,
            sort,
        ))
    }
}

/// Context-scoped evaluation run list query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluationRunListParams {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    suite_name: Option<String>,
    model_version: Option<String>,
    sort: Option<String>,
}

#[cfg(test)]
pub(crate) const EVALUATION_RUN_LIST_QUERY_PARAMETERS: &[&str] = &[
    "page",
    "per_page",
    "search",
    "suite_name",
    "model_version",
    "sort",
];

impl TryFrom<EvaluationRunListParams> for EvaluationRunListQuery {
    type Error = RequestQueryError;

    fn try_from(params: EvaluationRunListParams) -> Result<Self, Self::Error> {
        let sort = params
            .sort
            .as_deref()
            .unwrap_or(EvaluationRunSort::default().as_query_value())
            .parse::<EvaluationRunSort>()?;

        Ok(Self::new(
            params.page,
            params.per_page,
            params.search,
            params.suite_name,
            params.model_version,
            sort,
        ))
    }
}

/// Context-scoped evaluation scorecard query parameters.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EvaluationScorecardParams {
    search: Option<String>,
    suite_name: Option<String>,
    model_version: Option<String>,
}

#[cfg(test)]
pub(crate) const EVALUATION_SCORECARD_QUERY_PARAMETERS: &[&str] =
    &["search", "suite_name", "model_version"];

impl From<EvaluationScorecardParams> for EvaluationScorecardQuery {
    fn from(params: EvaluationScorecardParams) -> Self {
        Self::new(params.search, params.suite_name, params.model_version)
    }
}

fn storage_status(error: &StorageRepositoryError) -> StatusCode {
    match error {
        StorageRepositoryError::ScopeUnavailable { .. } => StatusCode::NOT_FOUND,
        StorageRepositoryError::InvalidScope { .. } => StatusCode::BAD_REQUEST,
        StorageRepositoryError::CommitAlreadyExists { .. }
        | StorageRepositoryError::IdempotencyKeyReused { .. }
        | StorageRepositoryError::BranchHeadConflict { .. }
        | StorageRepositoryError::CommitParentMismatch { .. }
        | StorageRepositoryError::ComponentContentRevisionConflict { .. }
        | StorageRepositoryError::ComponentStateReplayConflict { .. }
        | StorageRepositoryError::BenchmarkDefinitionConflict { .. }
        | StorageRepositoryError::BenchmarkDefinitionBindingConflict { .. }
        | StorageRepositoryError::BenchmarkEvidenceDigestConflict { .. }
        | StorageRepositoryError::WorkflowContextBindingConflict { .. } => StatusCode::CONFLICT,
        StorageRepositoryError::GuardedWriteForbidden => StatusCode::FORBIDDEN,
        StorageRepositoryError::GuardedWriteAuthorizationUnavailable => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        StorageRepositoryError::InvalidStoredComponentKind { .. }
        | StorageRepositoryError::Database { .. }
        | StorageRepositoryError::InMemoryStateUnavailable => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn storage_error_code(error: &StorageRepositoryError) -> &'static str {
    match error {
        StorageRepositoryError::ScopeUnavailable { .. } => "storage_scope_unavailable",
        StorageRepositoryError::InvalidScope { .. } => "storage_scope_invalid",
        StorageRepositoryError::CommitAlreadyExists { .. } => "storage_commit_conflict",
        StorageRepositoryError::IdempotencyKeyReused { .. } => "storage_idempotency_conflict",
        StorageRepositoryError::BranchHeadConflict { .. } => "storage_branch_head_conflict",
        StorageRepositoryError::CommitParentMismatch { .. } => "storage_commit_parent_conflict",
        StorageRepositoryError::ComponentContentRevisionConflict { .. } => {
            "storage_component_content_revision_conflict"
        }
        StorageRepositoryError::ComponentStateReplayConflict { .. } => {
            "storage_component_state_replay_conflict"
        }
        StorageRepositoryError::BenchmarkDefinitionConflict { .. } => {
            "storage_benchmark_definition_conflict"
        }
        StorageRepositoryError::BenchmarkDefinitionBindingConflict { .. } => {
            "storage_benchmark_definition_binding_conflict"
        }
        StorageRepositoryError::BenchmarkEvidenceDigestConflict { .. } => {
            "storage_benchmark_evidence_digest_conflict"
        }
        StorageRepositoryError::WorkflowContextBindingConflict { .. } => {
            "storage_workflow_context_binding_conflict"
        }
        StorageRepositoryError::GuardedWriteForbidden => "context_write_forbidden",
        StorageRepositoryError::GuardedWriteAuthorizationUnavailable => "authorization_unavailable",
        StorageRepositoryError::InvalidStoredComponentKind { .. } => "storage_contract_invalid",
        StorageRepositoryError::Database { .. } => "storage_database_error",
        StorageRepositoryError::InMemoryStateUnavailable => "storage_memory_state_unavailable",
    }
}

#[cfg(test)]
mod storage_error_tests {
    use super::{storage_error_code, storage_status};
    use axum::http::StatusCode;
    use contextlab_storage::StorageRepositoryError;

    #[test]
    fn maps_commit_conflicts_and_in_memory_state_failures() {
        let duplicate = StorageRepositoryError::CommitAlreadyExists {
            context_id: "context-1".to_owned(),
            commit_id: "commit-1".to_owned(),
        };

        assert_eq!(storage_status(&duplicate), StatusCode::CONFLICT);
        assert_eq!(storage_error_code(&duplicate), "storage_commit_conflict");
        assert_eq!(
            storage_status(&StorageRepositoryError::InMemoryStateUnavailable),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            storage_error_code(&StorageRepositoryError::InMemoryStateUnavailable),
            "storage_memory_state_unavailable"
        );
    }

    #[test]
    fn maps_private_benchmark_evidence_conflicts_without_exposing_a_transport() {
        let definition = StorageRepositoryError::BenchmarkDefinitionConflict {
            definition_kind: "dataset",
            definition_id: "dataset-1".to_owned(),
        };
        let digest = StorageRepositoryError::BenchmarkEvidenceDigestConflict {
            run_id: "decision-1".to_owned(),
        };

        assert_eq!(storage_status(&definition), StatusCode::CONFLICT);
        assert_eq!(
            storage_error_code(&definition),
            "storage_benchmark_definition_conflict"
        );
        assert_eq!(storage_status(&digest), StatusCode::CONFLICT);
        assert_eq!(
            storage_error_code(&digest),
            "storage_benchmark_evidence_digest_conflict"
        );
    }

    #[test]
    fn maps_guarded_commit_conflicts_to_stable_conflict_codes() {
        let idempotency = StorageRepositoryError::IdempotencyKeyReused {
            context_id: "context-1".to_owned(),
            principal_id: "user:alex".to_owned(),
            idempotency_key: "request-1".to_owned(),
        };
        let branch_head = StorageRepositoryError::BranchHeadConflict {
            expected: None,
            actual: Some("commit-1".to_owned()),
        };
        let parent = StorageRepositoryError::CommitParentMismatch {
            expected_parent_id: Some("commit-1".to_owned()),
            actual_parent_ids: Vec::new(),
        };
        let component_content = StorageRepositoryError::ComponentContentRevisionConflict {
            reason: "component hash transition does not match".to_owned(),
        };
        let component_state = StorageRepositoryError::ComponentStateReplayConflict {
            reason: "component state history cannot be replayed".to_owned(),
        };

        assert_eq!(storage_status(&idempotency), StatusCode::CONFLICT);
        assert_eq!(
            storage_error_code(&idempotency),
            "storage_idempotency_conflict"
        );
        assert_eq!(storage_status(&branch_head), StatusCode::CONFLICT);
        assert_eq!(
            storage_error_code(&branch_head),
            "storage_branch_head_conflict"
        );
        assert_eq!(storage_status(&parent), StatusCode::CONFLICT);
        assert_eq!(
            storage_error_code(&parent),
            "storage_commit_parent_conflict"
        );
        assert_eq!(storage_status(&component_content), StatusCode::CONFLICT);
        assert_eq!(
            storage_error_code(&component_content),
            "storage_component_content_revision_conflict"
        );
        assert_eq!(storage_status(&component_state), StatusCode::CONFLICT);
        assert_eq!(
            storage_error_code(&component_state),
            "storage_component_state_replay_conflict"
        );
    }

    #[test]
    fn maps_transactional_guarded_write_authorization_failures() {
        assert_eq!(
            storage_status(&StorageRepositoryError::GuardedWriteForbidden),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            storage_error_code(&StorageRepositoryError::GuardedWriteForbidden),
            "context_write_forbidden"
        );
        assert_eq!(
            storage_status(&StorageRepositoryError::GuardedWriteAuthorizationUnavailable),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            storage_error_code(&StorageRepositoryError::GuardedWriteAuthorizationUnavailable),
            "authorization_unavailable"
        );
    }
}

/// Returns API health.
pub async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// Returns platform-level metadata for clients and SDKs.
pub async fn meta() -> Json<MetaResponse> {
    Json(MetaResponse {
        product: "ContextLab",
        primary_abstraction: "context",
        bilingual: true,
        context_component_kinds: vec![
            ContextComponentKind::Prompt,
            ContextComponentKind::SystemPrompt,
            ContextComponentKind::Memory,
            ContextComponentKind::Knowledge,
            ContextComponentKind::Retrieval,
            ContextComponentKind::Embedding,
            ContextComponentKind::ModelConfiguration,
            ContextComponentKind::Tool,
            ContextComponentKind::McpServer,
            ContextComponentKind::Variable,
            ContextComponentKind::OutputSchema,
            ContextComponentKind::Workflow,
            ContextComponentKind::Conversation,
            ContextComponentKind::Evaluation,
        ],
        capabilities: vec![
            "context_core",
            "versioning",
            "text_diff",
            "evaluation_scorecards",
            "context_graph",
            "model_gateway",
            "rest_api",
        ],
    })
}

/// Returns the checked-in OpenAPI contract.
pub async fn openapi() -> Json<Value> {
    Json(
        serde_json::from_str(include_str!("../../../docs/api/openapi.json"))
            .expect("checked-in OpenAPI contract must be valid JSON"),
    )
}

/// Lists workspaces with pagination, filtering, and sorting.
pub async fn workspaces(
    State(state): State<AppState>,
    Query(params): Query<WorkspaceListParams>,
) -> Result<Json<WorkspaceList>, ApiError> {
    let query = WorkspaceListQuery::try_from(params)?;
    let workspaces = state.workspace_repository().list_workspaces(query).await?;

    Ok(Json(workspaces))
}

/// Lists projects for one workspace with pagination, filtering, and sorting.
pub async fn workspace_projects(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
    Query(params): Query<ProjectListParams>,
) -> Result<Json<ProjectList>, ApiError> {
    let query = ProjectListQuery::try_from(params)?;
    let projects = state
        .project_repository()
        .list_projects(workspace_id, query)
        .await?;

    Ok(Json(projects))
}

/// Lists experiments for one project with pagination, filtering, and sorting.
pub async fn project_experiments(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(params): Query<ExperimentListParams>,
) -> Result<Json<ExperimentList>, ApiError> {
    let query = ExperimentListQuery::try_from(params)?;
    let experiments = state
        .experiment_repository()
        .list_experiments(project_id, query)
        .await?;

    Ok(Json(experiments))
}

/// Lists contexts for one project with pagination, filtering, and sorting.
pub async fn project_contexts(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Query(params): Query<ContextListParams>,
) -> Result<Json<ContextList>, ApiError> {
    let query = ContextListQuery::try_from(params)?;
    let contexts = state
        .context_repository()
        .list_contexts(project_id, query)
        .await?;

    Ok(Json(contexts))
}

/// Lists commits for one context with pagination, filtering, and sorting.
pub async fn context_commits(
    State(state): State<AppState>,
    Path(context_id): Path<String>,
    Query(params): Query<CommitListParams>,
) -> Result<Json<CommitList>, ApiError> {
    let query = CommitListQuery::try_from(params)?;
    let commits = state
        .commit_repository()
        .list_commits(context_id, query)
        .await?;

    Ok(Json(commits))
}

/// Returns one commit for one context.
pub async fn context_commit(
    State(state): State<AppState>,
    Path((context_id, commit_id)): Path<(String, String)>,
) -> Result<Json<CommitDetail>, ApiError> {
    let commit = state
        .commit_repository()
        .get_commit(context_id, commit_id)
        .await?;

    Ok(Json(commit))
}

/// Compares two materialized Context commit graph snapshots.
pub async fn context_commit_graph_diff(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path(raw_context_id): Path<String>,
    Query(params): Query<CommitGraphDiffParams>,
) -> Result<Json<CommitGraphDiffResponse>, ApiError> {
    let context_id = parse_commit_graph_diff_context_id(&raw_context_id)?;
    let original_commit_id =
        required_commit_graph_diff_id(params.original_commit_id, "original_commit_id")?;
    let revised_commit_id =
        required_commit_graph_diff_id(params.revised_commit_id, "revised_commit_id")?;
    if original_commit_id == revised_commit_id {
        return Err(ApiError::InvalidCommitGraphDiffQuery(
            "original_commit_id and revised_commit_id must differ".to_owned(),
        ));
    }
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let project_id = state
        .commit_graph_snapshot_repository()
        .project_id_for_context(context_id)
        .await?;
    let original_scope = CommitGraphSnapshotScope::new(project_id, context_id, original_commit_id);
    let revised_scope = CommitGraphSnapshotScope::new(project_id, context_id, revised_commit_id);
    let witness_repository = state
        .context_graph_review_witness_repository()
        .ok_or(ApiError::CommitGraphDiffReviewUnavailable)?;
    let review = PersistedContextGraphWitnessReviewService::new(witness_repository)
        .review(original_scope, revised_scope)
        .await
        .map_err(map_persisted_context_graph_history_review_error)?;
    let graph_review = review.graph_review();

    Ok(Json(CommitGraphDiffResponse {
        context_id: context_id.to_string(),
        pair_witness: CommitGraphDiffPairWitness::from_snapshots(
            graph_review.source(),
            graph_review.target(),
        ),
        original: CommitGraphSnapshotReference::from(graph_review.source()),
        revised: CommitGraphSnapshotReference::from(graph_review.target()),
        diff: GraphDiffResponse::from(graph_review.review().diff().clone()),
    }))
}

/// Reviews two immutable persisted Context diff inputs without calculating in the API layer.
pub async fn local_context_diff_review(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id)): Path<(String, String)>,
    Query(params): Query<ContextDiffReviewParams>,
) -> Result<Json<contextlab_diff_engine::VersionedContextDiffReviewProjectionV1>, ApiError> {
    let project_id = parse_context_diff_review_project_id(&raw_project_id)?;
    let context_id = parse_context_diff_review_context_id(&raw_context_id)?;
    let source_commit_id =
        required_commit_graph_diff_id(params.source_commit_id, "source_commit_id")?;
    let target_commit_id =
        required_commit_graph_diff_id(params.target_commit_id, "target_commit_id")?;
    if source_commit_id == target_commit_id {
        return Err(ApiError::InvalidCommitGraphDiffQuery(
            "source_commit_id and target_commit_id must differ".to_owned(),
        ));
    }
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .context_diff_snapshot_repository()
        .ok_or(ApiError::ContextDiffReviewUnavailable)?;
    let source_scope = VersionedContextScopeV1::new(project_id, context_id, source_commit_id);
    let target_scope = VersionedContextScopeV1::new(project_id, context_id, target_commit_id);
    let projection = PersistedContextDiffReviewService::new(repository)
        .review(source_scope, target_scope)
        .await
        .map_err(map_persisted_context_diff_review_error)?;

    Ok(Json(projection))
}

/// Reviews two server-owned Context tips through the exact persisted commit DAG.
pub async fn local_context_merge_review(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id)): Path<(String, String)>,
    query: Result<Query<ContextMergeReviewParams>, QueryRejection>,
) -> Result<Json<VersionedContextGraphMergeReviewProjectionV1>, ApiError> {
    let Query(params) = query.map_err(|_| {
        ApiError::InvalidContextMergeReviewQuery(
            "left_commit_id and right_commit_id are the only supported query fields".to_owned(),
        )
    })?;
    let project_id = parse_context_merge_review_project_id(&raw_project_id)?;
    let context_id = parse_context_merge_review_context_id(&raw_context_id)?;
    let left_commit_id =
        required_context_merge_review_commit_id(params.left_commit_id, "left_commit_id")?;
    let right_commit_id =
        required_context_merge_review_commit_id(params.right_commit_id, "right_commit_id")?;
    if left_commit_id == right_commit_id {
        return Err(ApiError::InvalidContextMergeReviewQuery(
            "left_commit_id and right_commit_id must differ".to_owned(),
        ));
    }
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let scope = ContextMergeTipScope::new(project_id, context_id, left_commit_id, right_commit_id)
        .map_err(|_| {
            ApiError::InvalidContextMergeReviewQuery(
                "left_commit_id and right_commit_id must identify distinct non-nil commits"
                    .to_owned(),
            )
        })?;
    let repository = state
        .context_merge_review_witness_repository()
        .ok_or(ApiError::ContextMergeReviewUnavailable)?;
    let projection = PersistedContextGraphMergeReviewService::new(repository)
        .review_server_owned_projection(scope)
        .await
        .map_err(map_persisted_context_graph_merge_review_error)?;
    Ok(Json(projection))
}

fn map_persisted_context_graph_merge_review_error(
    error: PersistedContextGraphMergeReviewError,
) -> ApiError {
    match error {
        PersistedContextGraphMergeReviewError::CommitGraphRead { .. }
        | PersistedContextGraphMergeReviewError::SnapshotRead { .. } => {
            ApiError::ContextMergeReviewStorageUnavailable
        }
        PersistedContextGraphMergeReviewError::PlanResolutionFailed { .. }
        | PersistedContextGraphMergeReviewError::InvalidInputScope { .. }
        | PersistedContextGraphMergeReviewError::SnapshotUnavailable { .. }
        | PersistedContextGraphMergeReviewError::StoredScopeMismatch { .. }
        | PersistedContextGraphMergeReviewError::RequestedTipScopeMismatch
        | PersistedContextGraphMergeReviewError::ClassificationFailed { .. } => {
            ApiError::ContextMergeReviewUnavailable
        }
    }
}

fn required_context_merge_review_commit_id(
    value: Option<String>,
    parameter: &'static str,
) -> Result<CommitId, ApiError> {
    let value = value.unwrap_or_default().trim().to_owned();
    if value.is_empty() {
        return Err(ApiError::InvalidContextMergeReviewQuery(format!(
            "{parameter} must not be empty"
        )));
    }

    Uuid::parse_str(&value)
        .map(CommitId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextMergeReviewQuery(format!("{parameter} must be a UUID: {error}"))
        })
}

fn parse_context_merge_review_project_id(value: &str) -> Result<ProjectId, ApiError> {
    Uuid::parse_str(value)
        .map(ProjectId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextMergeReviewQuery(format!("project_id must be a UUID: {error}"))
        })
}

fn parse_context_merge_review_context_id(value: &str) -> Result<ContextId, ApiError> {
    Uuid::parse_str(value)
        .map(ContextId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextMergeReviewQuery(format!("context_id must be a UUID: {error}"))
        })
}

fn map_persisted_context_diff_review_error(error: PersistedContextDiffReviewError) -> ApiError {
    match error {
        PersistedContextDiffReviewError::InvalidScope { side, .. } => {
            ApiError::InvalidCommitGraphDiffQuery(format!("{side:?} commit scope is invalid"))
        }
        PersistedContextDiffReviewError::MismatchedContextScope { .. } => {
            ApiError::InvalidCommitGraphDiffQuery(
                "source_commit_id and target_commit_id must belong to one project and Context"
                    .to_owned(),
            )
        }
        PersistedContextDiffReviewError::IdenticalVersionScopes { .. } => {
            ApiError::InvalidCommitGraphDiffQuery(
                "source_commit_id and target_commit_id must differ".to_owned(),
            )
        }
        PersistedContextDiffReviewError::SnapshotRead { scope, source, .. } => match source {
            contextlab_storage::ContextDiffSnapshotPersistenceError::NotFound { .. } => {
                ApiError::ContextDiffReviewSnapshotMissing {
                    project_id: scope.project_id().to_string(),
                    context_id: scope.context_id().to_string(),
                    commit_id: scope.commit_id().to_string(),
                }
            }
            _ => ApiError::ContextDiffReviewUnavailable,
        },
        PersistedContextDiffReviewError::SnapshotPairRead { source, .. } => match *source {
            contextlab_storage::ContextDiffSnapshotPersistenceError::NotFound { scope } => {
                ApiError::ContextDiffReviewSnapshotMissing {
                    project_id: scope.project_id().to_string(),
                    context_id: scope.context_id().to_string(),
                    commit_id: scope.commit_id().to_string(),
                }
            }
            _ => ApiError::ContextDiffReviewUnavailable,
        },
        PersistedContextDiffReviewError::StoredScopeMismatch { .. }
        | PersistedContextDiffReviewError::StoredSchemaMismatch { .. }
        | PersistedContextDiffReviewError::ReviewFailed { .. } => {
            ApiError::ContextDiffReviewUnavailable
        }
    }
}

fn map_persisted_context_graph_diff_review_error(
    error: PersistedContextGraphDiffReviewError,
) -> ApiError {
    match error {
        PersistedContextGraphDiffReviewError::InvalidScope { side, .. } => {
            ApiError::InvalidCommitGraphDiffQuery(format!("{side:?} commit graph scope is invalid"))
        }
        PersistedContextGraphDiffReviewError::MismatchedContextScope { .. } => {
            ApiError::InvalidCommitGraphDiffQuery(
                "original_commit_id and revised_commit_id must belong to one Context".to_owned(),
            )
        }
        PersistedContextGraphDiffReviewError::IdenticalVersionScopes { .. } => {
            ApiError::InvalidCommitGraphDiffQuery(
                "original_commit_id and revised_commit_id must differ".to_owned(),
            )
        }
        PersistedContextGraphDiffReviewError::SnapshotMissing { side, scope } => {
            let commit_id = match side {
                contextlab_storage::PersistedContextGraphDiffReviewSide::Source
                | contextlab_storage::PersistedContextGraphDiffReviewSide::Target => {
                    scope.commit_id()
                }
            };
            ApiError::CommitGraphSnapshotMissing {
                context_id: scope.context_id().to_string(),
                commit_id: commit_id.to_string(),
            }
        }
        PersistedContextGraphDiffReviewError::SnapshotRead { source, .. } => {
            ApiError::StorageRepository(source)
        }
        PersistedContextGraphDiffReviewError::LifecycleRead { .. }
        | PersistedContextGraphDiffReviewError::LifecycleMissing { .. } => {
            ApiError::CommitGraphDiffReviewUnavailable
        }
        PersistedContextGraphDiffReviewError::StoredScopeMismatch { .. }
        | PersistedContextGraphDiffReviewError::StoredSchemaMismatch { .. }
        | PersistedContextGraphDiffReviewError::ReviewFailed { .. } => {
            ApiError::CommitGraphDiffReviewUnavailable
        }
    }
}

fn map_persisted_context_graph_history_review_error(
    error: PersistedContextGraphHistoryReviewError,
) -> ApiError {
    match error {
        PersistedContextGraphHistoryReviewError::WitnessRead { .. }
        | PersistedContextGraphHistoryReviewError::BranchWitnessRead { .. } => {
            ApiError::CommitGraphDiffReviewUnavailable
        }
        PersistedContextGraphHistoryReviewError::GraphReview { source } => {
            map_persisted_context_graph_diff_review_error(source)
        }
        PersistedContextGraphHistoryReviewError::HistoryRead { .. }
        | PersistedContextGraphHistoryReviewError::CommitMissing { .. }
        | PersistedContextGraphHistoryReviewError::HistoryContextMismatch { .. }
        | PersistedContextGraphHistoryReviewError::ReplayPathInvalid { .. } => {
            ApiError::CommitGraphDiffReviewUnavailable
        }
    }
}

fn required_commit_graph_diff_id(
    value: Option<String>,
    parameter: &'static str,
) -> Result<CommitId, ApiError> {
    let value = value.unwrap_or_default().trim().to_owned();
    if value.is_empty() {
        return Err(ApiError::InvalidCommitGraphDiffQuery(format!(
            "{parameter} must not be empty"
        )));
    }

    Uuid::parse_str(&value)
        .map(CommitId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidCommitGraphDiffQuery(format!("{parameter} must be a UUID: {error}"))
        })
}

fn parse_commit_graph_diff_context_id(value: &str) -> Result<ContextId, ApiError> {
    Uuid::parse_str(value)
        .map(ContextId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidCommitGraphDiffQuery(format!("context_id must be a UUID: {error}"))
        })
}

fn parse_context_diff_review_project_id(value: &str) -> Result<ProjectId, ApiError> {
    Uuid::parse_str(value)
        .map(ProjectId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidCommitGraphDiffQuery(format!("project_id must be a UUID: {error}"))
        })
}

fn parse_context_diff_review_context_id(value: &str) -> Result<ContextId, ApiError> {
    Uuid::parse_str(value)
        .map(ContextId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidCommitGraphDiffQuery(format!("context_id must be a UUID: {error}"))
        })
}

/// Lists components for one context with pagination, filtering, and sorting.
pub async fn context_components(
    State(state): State<AppState>,
    Path(context_id): Path<String>,
    Query(params): Query<ComponentListParams>,
) -> Result<Json<ComponentList>, ApiError> {
    let query = ComponentListQuery::try_from(params)?;
    let components = state
        .component_repository()
        .list_components(context_id, query)
        .await?;

    Ok(Json(components))
}

/// Returns one component for one context.
pub async fn context_component(
    State(state): State<AppState>,
    Path((context_id, component_id)): Path<(String, String)>,
) -> Result<Json<ComponentDetail>, ApiError> {
    let component = state
        .component_repository()
        .get_component(context_id, component_id)
        .await?;

    Ok(Json(component))
}

/// Lists evaluation runs for one context with pagination, filtering, and sorting.
pub async fn context_evaluation_runs(
    State(state): State<AppState>,
    Path(context_id): Path<String>,
    Query(params): Query<EvaluationRunListParams>,
) -> Result<Json<EvaluationRunList>, ApiError> {
    let query = EvaluationRunListQuery::try_from(params)?;
    let evaluation_runs = state
        .evaluation_run_repository()
        .list_evaluation_runs(context_id, query)
        .await?;

    Ok(Json(evaluation_runs))
}

/// Returns one evaluation run for one context.
pub async fn context_evaluation_run(
    State(state): State<AppState>,
    Path((context_id, run_id)): Path<(String, String)>,
) -> Result<Json<EvaluationRunDetail>, ApiError> {
    let evaluation_run = state
        .evaluation_run_repository()
        .get_evaluation_run(context_id, run_id)
        .await?;

    Ok(Json(evaluation_run))
}

/// Returns a context-level evaluation scorecard built from persisted metrics.
pub async fn context_evaluation_scorecard(
    State(state): State<AppState>,
    Path(context_id): Path<String>,
    Query(params): Query<EvaluationScorecardParams>,
) -> Result<Json<EvaluationScorecard>, ApiError> {
    let query = EvaluationScorecardQuery::from(params);
    let scorecard = state
        .evaluation_run_repository()
        .get_evaluation_scorecard(context_id, query)
        .await?;

    Ok(Json(scorecard))
}

/// Returns a deterministic Context Graph preview for early clients.
pub async fn context_graph_preview(
    State(state): State<AppState>,
) -> Result<Json<ContextGraphResponse>, ApiError> {
    let projection = state
        .graph_repository()
        .load_context_graph_projection(GraphProjectionScope::Preview)
        .await?;

    Ok(Json(ContextGraphResponse {
        graph: projection.project()?,
    }))
}

/// Returns a Context Graph for one workspace.
pub async fn workspace_context_graph(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<Json<ContextGraphResponse>, ApiError> {
    let projection = state
        .workspace_graph_repository()
        .load_context_graph_projection(GraphProjectionScope::Workspace { workspace_id })
        .await?;

    Ok(Json(ContextGraphResponse {
        graph: projection.project()?,
    }))
}

/// Compares two validated Context Graph snapshots without persisting either snapshot.
pub async fn graph_diffs(
    request: Result<Json<GraphDiffRequest>, JsonRejection>,
) -> Result<Json<GraphDiffResponse>, ApiError> {
    let Json(request) =
        request.map_err(|error| ApiError::InvalidGraphSnapshotRequest(error.to_string()))?;
    let original = request.original.into_context_graph()?;
    let revised = request.revised.into_context_graph()?;

    Ok(Json(GraphDiffResponse::from(GraphDiff::between(
        &original, &revised,
    ))))
}

/// Creates a normal Context commit through an explicitly protected router.
pub async fn create_context_commit(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path(raw_context_id): Path<String>,
    headers: HeaderMap,
    request: Result<Json<ContextCommitWriteRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<ContextCommitWriteResponse>), ApiError> {
    let context_uuid = Uuid::parse_str(&raw_context_id).map_err(|error| {
        ApiError::InvalidContextCommitRequest(format!("context_id is invalid: {error}"))
    })?;
    let context_id = ContextId::from_uuid(context_uuid);

    authorize_context_request(&state, &principal, context_id, ContextPermission::Write).await?;

    let Json(request) =
        request.map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))?;
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::InvalidContextCommitRequest("Idempotency-Key is required".to_owned())
        })
        .and_then(|value| {
            IdempotencyKey::new(value)
                .map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))
        })?;
    let request_digest = RequestDigest::new(sha256_digest(&request))
        .map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))?;
    let expected_branch_head =
        parse_expected_branch_head(request.expected_head_commit_id.as_deref())?;
    let parent_ids = match expected_branch_head {
        ExpectedBranchHead::Unborn => Vec::new(),
        ExpectedBranchHead::Commit(commit_id) => vec![commit_id],
    };
    let branch = BranchName::new(request.branch_name)
        .map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))?;
    let graph = request
        .snapshot
        .into_context_graph()
        .map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))?;
    let captured_at = Utc::now();
    let commit = ContextCommit::new(
        context_id,
        branch,
        request.message,
        parent_ids,
        request.changes,
        captured_at,
    )
    .map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))?;
    let project_id = state
        .commit_graph_snapshot_repository()
        .project_id_for_context(context_id)
        .await?;
    let snapshot_command = CreateContextCommitSnapshot::new(
        project_id,
        commit,
        graph,
        captured_at,
        request.schema_version,
    )
    .map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))?;
    let result = state
        .guarded_commit_writer()
        .create_guarded_commit_snapshot(
            GuardedContextCommitWrite::new(
                principal,
                expected_branch_head,
                idempotency_key,
                request_digest,
                snapshot_command,
            )
            .map_err(|error| ApiError::InvalidContextCommitRequest(error.to_string()))?,
        )
        .await?;
    let status = match result.disposition {
        GuardedCommitWriteDisposition::Created => StatusCode::CREATED,
        GuardedCommitWriteDisposition::Replayed => StatusCode::OK,
    };
    let disposition = match result.disposition {
        GuardedCommitWriteDisposition::Created => "created",
        GuardedCommitWriteDisposition::Replayed => "replayed",
    };

    Ok((
        status,
        Json(ContextCommitWriteResponse {
            disposition,
            snapshot: result.snapshot,
        }),
    ))
}

/// Reads complete private component lifecycle state for one materialized Context commit.
pub async fn local_context_lifecycle_state(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_context_id, raw_commit_id)): Path<(String, String)>,
) -> Result<Json<LocalContextLifecycleStateResponse>, ApiError> {
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id, "commit_id")?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .context_lifecycle_repository()
        .ok_or(ApiError::ContextLifecycleUnavailable)?;
    let lifecycle_state = ContextLifecycleService::new(repository)
        .read_state_at_commit(context_id, commit_id)
        .await
        .map_err(map_context_lifecycle_error)?;

    Ok(Json(LocalContextLifecycleStateResponse::from_state(
        context_id,
        commit_id,
        lifecycle_state,
    )))
}

/// Reads the provider-free redacted Knowledge/Memory projection for one exact Context commit.
pub async fn local_knowledge_memory_projection(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id)): Path<(String, String, String)>,
) -> Result<Json<KnowledgeMemoryProjectionResource>, ApiError> {
    let project_id = parse_project_id(&raw_project_id)?;
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id, "commit_id")?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .knowledge_memory_projection_repository()
        .ok_or(ApiError::KnowledgeMemoryProjectionUnavailable)?;
    let projection = repository
        .project(project_id, context_id, commit_id)
        .await
        .map_err(|_| ApiError::KnowledgeMemoryProjectionUnavailable)?;
    projection
        .validate_for_scope(project_id, context_id, commit_id)
        .map_err(|_| ApiError::KnowledgeMemoryProjectionInvalid)?;
    if projection.projection.contains_raw_knowledge_content()
        || projection.projection.contains_raw_memory_content()
        || projection.projection.contains_raw_vectors()
        || projection.projection.contains_raw_query()
    {
        return Err(ApiError::KnowledgeMemoryProjectionInvalid);
    }
    Ok(Json(projection))
}

/// Reads one sealed private benchmark decision for an exact project, Context, and commit scope.
pub async fn local_benchmark_decision(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id, raw_decision_id)): Path<(
        String,
        String,
        String,
        String,
    )>,
) -> Result<Json<LocalBenchmarkDecisionResponse>, ApiError> {
    let project_id = parse_project_id(&raw_project_id)?;
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id, "commit_id")?;
    let decision_id = parse_benchmark_decision_id(&raw_decision_id)?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .benchmark_evidence_repository()
        .ok_or(ApiError::BenchmarkEvidenceUnavailable)?;
    let evidence = repository
        .get_benchmark_decision(project_id, context_id, commit_id, decision_id)
        .await
        .map_err(ApiError::BenchmarkEvidenceStorage)?
        .ok_or(ApiError::BenchmarkEvidenceNotFound)?;
    let definition = BenchmarkDecisionDefinitionSummaryService::new(repository)
        .summarize(&evidence)
        .await
        .map_err(map_benchmark_decision_definition_summary_error)?;

    Ok(Json(LocalBenchmarkDecisionResponse::from_evidence(
        evidence, definition,
    )))
}

/// Lists sealed private benchmark decisions for one exact project, Context, and commit scope.
pub async fn local_benchmark_decision_list(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id)): Path<(String, String, String)>,
) -> Result<Json<LocalBenchmarkDecisionListResponse>, ApiError> {
    let project_id = parse_project_id(&raw_project_id)?;
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id, "commit_id")?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let discovery_repository = state
        .benchmark_decision_discovery_repository()
        .ok_or(ApiError::BenchmarkEvidenceUnavailable)?;
    let decisions = BenchmarkDecisionDiscoveryService::new(discovery_repository)
        .summarize(project_id, context_id, commit_id)
        .await
        .map_err(map_benchmark_decision_discovery_error)?;

    Ok(Json(LocalBenchmarkDecisionListResponse::from_summaries(
        project_id, context_id, commit_id, decisions,
    )))
}

/// Reads the private local workflow capability status.
pub async fn local_workflow_capability_status(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path(raw_context_id): Path<String>,
) -> Result<Json<crate::workflow_status::WorkflowCapabilityStatusV1>, ApiError> {
    let context_id = parse_context_id(&raw_context_id)?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    Ok(Json(UnavailableWorkflowCapabilityAdapter.status()))
}

/// Reads the private, provider-free Plugin/MCP capability availability projection.
pub async fn local_plugin_capability_availability(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path(raw_context_id): Path<String>,
) -> Result<Json<PluginCapabilityAvailabilityResource>, ApiError> {
    let context_id = parse_context_id(&raw_context_id)?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let resource = state
        .plugin_capability_availability_repository()
        .read(context_id)
        .await
        .map_err(|_| ApiError::PluginCapabilityAvailabilityUnavailable)?;
    resource
        .validate_for_scope(context_id)
        .map_err(|_| ApiError::PluginCapabilityAvailabilityInvalid)?;
    Ok(Json(resource))
}

/// Reads immutable Workflow source bindings for one exact Context commit.
pub async fn local_workflow_context_bindings(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_context_id, raw_commit_id)): Path<(String, String)>,
) -> Result<Json<LocalWorkflowContextBindingsResponse>, ApiError> {
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id, "commit_id")?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .context_workflow_binding_repository()
        .ok_or(ApiError::WorkflowContextBindingsUnavailable)?;
    let bindings = repository
        .list_workflow_context_bindings_at_commit(context_id, commit_id)
        .await
        .map_err(ApiError::WorkflowContextBindingsStorage)?;

    Ok(Json(LocalWorkflowContextBindingsResponse::from_bindings(
        context_id, commit_id, bindings,
    )))
}

/// Reads a redacted Workflow execution status and replay provenance projection.
pub async fn local_workflow_execution_status(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_context_id, raw_run_id)): Path<(String, String)>,
) -> Result<Json<LocalWorkflowExecutionStatusResource>, ApiError> {
    let context_id = parse_context_id(&raw_context_id)?;
    let run_id = Uuid::parse_str(&raw_run_id)
        .map(WorkflowRunId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidWorkflowExecutionStatusRequest(format!("run_id is invalid: {error}"))
        })?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .workflow_execution_status_repository()
        .ok_or(ApiError::WorkflowExecutionStatusUnavailable)?;
    let resource = repository
        .read_scoped(context_id, run_id)
        .await
        .map_err(|error| match error {
            crate::workflow_execution::WorkflowExecutionStatusStorageError::Unavailable => {
                ApiError::WorkflowExecutionStatusUnavailable
            }
            crate::workflow_execution::WorkflowExecutionStatusStorageError::InvalidScope => {
                ApiError::WorkflowExecutionStatusInvalid
            }
        })?
        .ok_or(ApiError::WorkflowExecutionStatusNotFound)?;

    Ok(Json(resource))
}

/// Reads safe ordered run details behind one exact sealed private benchmark decision.
pub async fn local_benchmark_decision_run_details(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id, raw_decision_id)): Path<(
        String,
        String,
        String,
        String,
    )>,
) -> Result<Json<LocalBenchmarkDecisionRunDetailsResponse>, ApiError> {
    let project_id = parse_project_id(&raw_project_id)?;
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id, "commit_id")?;
    let decision_id = parse_benchmark_decision_id(&raw_decision_id)?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .benchmark_evidence_repository()
        .ok_or(ApiError::BenchmarkEvidenceUnavailable)?;
    let evidence = repository
        .get_benchmark_decision(project_id, context_id, commit_id, decision_id)
        .await
        .map_err(ApiError::BenchmarkEvidenceStorage)?
        .ok_or(ApiError::BenchmarkEvidenceNotFound)?;
    let summary = BenchmarkDecisionRunDetailsService::new(repository)
        .summarize(&evidence)
        .await
        .map_err(map_benchmark_decision_run_details_error)?;

    Ok(Json(
        LocalBenchmarkDecisionRunDetailsResponse::from_summary(&evidence, summary),
    ))
}

/// Compares two sealed private benchmark decisions at explicit project and Context scope.
pub async fn local_benchmark_decision_diff(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id)): Path<(String, String)>,
    Query(params): Query<BenchmarkDecisionDiffParams>,
) -> Result<Json<LocalBenchmarkDecisionDiffResponse>, ApiError> {
    let project_id = parse_project_id(&raw_project_id)?;
    let context_id = parse_context_id(&raw_context_id)?;
    let baseline = BenchmarkDecisionComparisonScope::new(
        parse_commit_id(&params.baseline_commit_id, "baseline_commit_id")?,
        parse_benchmark_decision_id(&params.baseline_decision_id)?,
    );
    let revised = BenchmarkDecisionComparisonScope::new(
        parse_commit_id(&params.revised_commit_id, "revised_commit_id")?,
        parse_benchmark_decision_id(&params.revised_decision_id)?,
    );
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .benchmark_evidence_repository()
        .ok_or(ApiError::BenchmarkEvidenceUnavailable)?;
    let diff = BenchmarkDecisionComparisonService::new(repository)
        .compare(project_id, context_id, baseline, revised)
        .await
        .map_err(map_benchmark_decision_comparison_error)?
        .ok_or(ApiError::BenchmarkEvidenceNotFound)?;

    Ok(Json(LocalBenchmarkDecisionDiffResponse::from_diff(
        project_id, context_id, baseline, revised, diff,
    )))
}

/// Reads one redacted benchmark workspace projection at an exact immutable receipt scope.
pub async fn local_benchmark_workspace(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id, raw_cohort_id)): Path<(
        String,
        String,
        String,
        String,
    )>,
    params: Result<Query<LocalBenchmarkWorkspaceParams>, QueryRejection>,
) -> Result<(HeaderMap, Json<LocalBenchmarkWorkspaceResponse>), ApiError> {
    let Query(params) = params.map_err(|_| {
        ApiError::InvalidBenchmarkWorkspaceRequest(
            "query parameters do not match the V1 contract".to_owned(),
        )
    })?;
    let project_id = ProjectId::from_uuid(parse_benchmark_workspace_uuid(
        &raw_project_id,
        "project_id",
    )?);
    let context_id = ContextId::from_uuid(parse_benchmark_workspace_uuid(
        &raw_context_id,
        "context_id",
    )?);
    let commit_id =
        CommitId::from_uuid(parse_benchmark_workspace_uuid(&raw_commit_id, "commit_id")?);
    let cohort_id = parse_benchmark_execution_cohort_id(&raw_cohort_id, "cohort_id")?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let revised_scope =
        BenchmarkWorkspaceProjectionReceiptScope::new(project_id, context_id, commit_id, cohort_id);
    let (query, baseline, expected_baseline_cohort_id) = match (
        params.baseline_commit_id.as_deref(),
        params.baseline_cohort_id.as_deref(),
    ) {
        (None, None) => (
            BenchmarkWorkspaceProjectionV1Query::single(revised_scope),
            None,
            None,
        ),
        (Some(raw_baseline_commit_id), Some(raw_baseline_cohort_id)) => {
            let baseline_commit_id = CommitId::from_uuid(parse_benchmark_workspace_uuid(
                raw_baseline_commit_id,
                "baseline_commit_id",
            )?);
            let baseline_cohort_id =
                parse_benchmark_execution_cohort_id(raw_baseline_cohort_id, "baseline_cohort_id")?;
            let baseline_scope = BenchmarkWorkspaceProjectionReceiptScope::new(
                project_id,
                context_id,
                baseline_commit_id,
                baseline_cohort_id,
            );
            (
                BenchmarkWorkspaceProjectionV1Query::comparing(baseline_scope, revised_scope),
                Some(LocalBenchmarkWorkspaceScopeResponse {
                    commit_id: baseline_commit_id.to_string(),
                    cohort_id: baseline_cohort_id.as_uuid().to_string(),
                }),
                Some(baseline_cohort_id),
            )
        }
        _ => {
            return Err(ApiError::InvalidBenchmarkWorkspaceRequest(
                "baseline_commit_id and baseline_cohort_id must be supplied together".to_owned(),
            ));
        }
    };

    let repository = state
        .benchmark_workspace_projection_repository()
        .ok_or(ApiError::BenchmarkWorkspaceUnavailable)?;
    let projection = repository
        .read_benchmark_workspace_projection(query)
        .await
        .map_err(map_benchmark_workspace_projection_error)?;
    let diff_scope_matches = match (expected_baseline_cohort_id, projection.evaluation_diff()) {
        (None, None) => true,
        (Some(expected_baseline), Some(diff)) => {
            diff.baseline_cohort_id() == expected_baseline && diff.revised_cohort_id() == cohort_id
        }
        _ => false,
    };
    if projection.receipt().cohort_id() != cohort_id || !diff_scope_matches {
        return Err(ApiError::BenchmarkWorkspaceStorage(
            BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid,
        ));
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok((
        headers,
        Json(LocalBenchmarkWorkspaceResponse {
            schema_version: "contextlab.local-benchmark-workspace.v1",
            project_id: project_id.to_string(),
            context_id: context_id.to_string(),
            revised: LocalBenchmarkWorkspaceScopeResponse {
                commit_id: commit_id.to_string(),
                cohort_id: cohort_id.as_uuid().to_string(),
            },
            baseline,
            decision_pair_witness: None,
            projection,
        }),
    ))
}

/// Reads the existing redacted workspace projection through an exact sealed decision binding.
pub async fn local_benchmark_workspace_by_decision(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id, raw_decision_id)): Path<(
        String,
        String,
        String,
        String,
    )>,
    params: Result<Query<LocalBenchmarkWorkspaceDecisionParams>, QueryRejection>,
) -> Result<(HeaderMap, Json<LocalBenchmarkWorkspaceResponse>), ApiError> {
    let Query(params) = params.map_err(|_| {
        ApiError::InvalidBenchmarkWorkspaceRequest(
            "query parameters do not match the decision-bound V1 contract".to_owned(),
        )
    })?;
    let project_id = ProjectId::from_uuid(parse_benchmark_workspace_uuid(
        &raw_project_id,
        "project_id",
    )?);
    let context_id = ContextId::from_uuid(parse_benchmark_workspace_uuid(
        &raw_context_id,
        "context_id",
    )?);
    let commit_id =
        CommitId::from_uuid(parse_benchmark_workspace_uuid(&raw_commit_id, "commit_id")?);
    let decision_id = BenchmarkDecisionId::from_uuid(parse_benchmark_workspace_uuid(
        &raw_decision_id,
        "decision_id",
    )?);
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;

    let repository = state
        .benchmark_workspace_projection_repository()
        .ok_or(ApiError::BenchmarkWorkspaceUnavailable)?;
    let revised_scope = repository
        .resolve_benchmark_workspace_projection_scope(
            BenchmarkWorkspaceProjectionDecisionQuery::new(
                project_id,
                context_id,
                commit_id,
                decision_id,
            ),
        )
        .await
        .map_err(map_benchmark_workspace_projection_error)?
        .ok_or(ApiError::BenchmarkWorkspaceNotFound)?;
    validate_decision_workspace_scope(revised_scope, project_id, context_id, commit_id)?;

    let (query, baseline, decision_pair_witness) = match (
        params.baseline_commit_id.as_deref(),
        params.baseline_decision_id.as_deref(),
    ) {
        (None, None) => (
            BenchmarkWorkspaceProjectionV1Query::single(revised_scope),
            None,
            None,
        ),
        (Some(raw_baseline_commit_id), Some(raw_baseline_decision_id)) => {
            let baseline_commit_id = CommitId::from_uuid(parse_benchmark_workspace_uuid(
                raw_baseline_commit_id,
                "baseline_commit_id",
            )?);
            let baseline_decision_id = BenchmarkDecisionId::from_uuid(
                parse_benchmark_workspace_uuid(raw_baseline_decision_id, "baseline_decision_id")?,
            );
            let baseline_scope = repository
                .resolve_benchmark_workspace_projection_scope(
                    BenchmarkWorkspaceProjectionDecisionQuery::new(
                        project_id,
                        context_id,
                        baseline_commit_id,
                        baseline_decision_id,
                    ),
                )
                .await
                .map_err(map_benchmark_workspace_projection_error)?
                .ok_or(ApiError::BenchmarkWorkspaceNotFound)?;
            validate_decision_workspace_scope(
                baseline_scope,
                project_id,
                context_id,
                baseline_commit_id,
            )?;
            let decision_pair_witness =
                LocalBenchmarkDecisionPairWitnessResponse::from_resolved_scopes(
                    project_id,
                    context_id,
                    baseline_scope,
                    baseline_decision_id,
                    revised_scope,
                    decision_id,
                );
            (
                BenchmarkWorkspaceProjectionV1Query::comparing(baseline_scope, revised_scope),
                Some(LocalBenchmarkWorkspaceScopeResponse {
                    commit_id: baseline_commit_id.to_string(),
                    cohort_id: baseline_scope.cohort_id().as_uuid().to_string(),
                }),
                Some(decision_pair_witness),
            )
        }
        _ => {
            return Err(ApiError::InvalidBenchmarkWorkspaceRequest(
                "baseline_commit_id and baseline_decision_id must be supplied together".to_owned(),
            ));
        }
    };

    let projection = repository
        .read_benchmark_workspace_projection(query)
        .await
        .map_err(map_benchmark_workspace_projection_error)?;
    let diff_scope_matches = match (&baseline, projection.evaluation_diff()) {
        (None, None) => true,
        (Some(expected_baseline), Some(diff)) => {
            diff.baseline_cohort_id().as_uuid().to_string() == expected_baseline.cohort_id
                && diff.revised_cohort_id() == revised_scope.cohort_id()
        }
        _ => false,
    };
    if projection.receipt().cohort_id() != revised_scope.cohort_id() || !diff_scope_matches {
        return Err(ApiError::BenchmarkWorkspaceStorage(
            BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid,
        ));
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok((
        headers,
        Json(LocalBenchmarkWorkspaceResponse {
            schema_version: "contextlab.local-benchmark-workspace.v1",
            project_id: project_id.to_string(),
            context_id: context_id.to_string(),
            revised: LocalBenchmarkWorkspaceScopeResponse {
                commit_id: commit_id.to_string(),
                cohort_id: revised_scope.cohort_id().as_uuid().to_string(),
            },
            baseline,
            decision_pair_witness,
            projection,
        }),
    ))
}

/// Creates, updates, or removes one component through the guarded local lifecycle service.
pub async fn create_local_component_lifecycle_commit(
    State(state): State<AppState>,
    Extension(principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path(raw_context_id): Path<String>,
    headers: HeaderMap,
    request: Result<Json<LocalContextLifecycleWriteRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<LocalContextLifecycleWriteResponse>), ApiError> {
    let context_id = parse_context_id(&raw_context_id)?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Write).await?;

    let Json(request) =
        request.map_err(|error| ApiError::InvalidContextLifecycleRequest(error.to_string()))?;
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::InvalidContextLifecycleRequest("Idempotency-Key is required".to_owned())
        })
        .and_then(|value| {
            IdempotencyKey::new(value)
                .map_err(|error| ApiError::InvalidContextLifecycleRequest(error.to_string()))
        })?;
    let request_digest = RequestDigest::new(sha256_digest(&request))
        .map_err(|error| ApiError::InvalidContextLifecycleRequest(error.to_string()))?;
    let branch = BranchName::new(request.branch_name)
        .map_err(|error| ApiError::InvalidContextLifecycleRequest(error.to_string()))?;
    let operation = request.operation.into_lifecycle_operation()?;
    let repository = state
        .context_lifecycle_repository()
        .ok_or(ApiError::ContextLifecycleUnavailable)?;
    let command = match operation {
        ContextLifecycleOperation::Initialize => {
            if request.expected_head_commit_id.is_some() {
                return Err(ApiError::InvalidContextLifecycleRequest(
                    "initialize requires expected_head_commit_id to be null".to_owned(),
                ));
            }
            ContextLifecycleCommand::initialize(
                principal,
                context_id,
                branch,
                idempotency_key,
                request_digest,
                request.message,
                Utc::now(),
            )
        }
        operation => {
            let expected_head = request
                .expected_head_commit_id
                .as_deref()
                .ok_or_else(|| {
                    ApiError::InvalidContextLifecycleRequest(
                        "expected_head_commit_id is required outside initialize".to_owned(),
                    )
                })
                .and_then(|value| parse_commit_id(value, "expected_head_commit_id"))?;
            ContextLifecycleCommand::from_operation(
                principal,
                context_id,
                branch,
                expected_head,
                idempotency_key,
                request_digest,
                request.message,
                operation,
                Utc::now(),
            )
        }
    };
    let result = ContextLifecycleService::new(repository)
        .execute(command)
        .await
        .map_err(map_context_lifecycle_error)?;
    let disposition = match result.disposition() {
        GuardedCommitWriteDisposition::Created => "created",
        GuardedCommitWriteDisposition::Replayed => "replayed",
    };
    let status = match result.disposition() {
        GuardedCommitWriteDisposition::Created => StatusCode::CREATED,
        GuardedCommitWriteDisposition::Replayed => StatusCode::OK,
    };

    Ok((
        status,
        Json(LocalContextLifecycleWriteResponse {
            schema_version: LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1,
            disposition,
            commit_id: result.commit_id().to_string(),
            snapshot: result.snapshot().clone(),
        }),
    ))
}

pub(crate) async fn authorize_context_request(
    state: &AppState,
    principal: &contextlab_auth::AuthenticatedPrincipal,
    context_id: ContextId,
    permission: ContextPermission,
) -> Result<(), ApiError> {
    let authorization_decision = match state
        .context_authorizer()
        .authorize(principal, context_id, permission)
        .await
    {
        Ok(()) => AuthorizationDecision::Granted,
        Err(AuthorizationError::Forbidden) => AuthorizationDecision::Forbidden,
        Err(AuthorizationError::Unavailable) => AuthorizationDecision::Unavailable,
    };
    state
        .record_authorization_decision(AuthorizationAuditEvent::new(
            principal.identity().clone(),
            context_id,
            permission,
            authorization_decision,
        ))
        .await
        .map_err(|_| ApiError::AuthorizationAuditUnavailable)?;

    match authorization_decision {
        AuthorizationDecision::Granted => Ok(()),
        AuthorizationDecision::Forbidden => match permission {
            ContextPermission::Read => Err(ApiError::ContextReadForbidden),
            ContextPermission::Write => Err(ApiError::ContextWriteForbidden),
        },
        AuthorizationDecision::Unavailable => Err(ApiError::AuthorizationUnavailable),
    }
}

fn validate_decision_workspace_scope(
    scope: BenchmarkWorkspaceProjectionReceiptScope,
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
) -> Result<(), ApiError> {
    if scope.project_id() != project_id
        || scope.context_id() != context_id
        || scope.context_commit_id() != commit_id
    {
        return Err(ApiError::BenchmarkWorkspaceStorage(
            BenchmarkWorkspaceProjectionPersistenceError::StoredSourceInvalid,
        ));
    }
    Ok(())
}

fn parse_context_id(value: &str) -> Result<ContextId, ApiError> {
    Uuid::parse_str(value)
        .map(ContextId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextLifecycleRequest(format!("context_id is invalid: {error}"))
        })
}

fn parse_commit_id(value: &str, field: &str) -> Result<CommitId, ApiError> {
    Uuid::parse_str(value)
        .map(CommitId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextLifecycleRequest(format!("{field} is invalid: {error}"))
        })
}

fn parse_project_id(value: &str) -> Result<ProjectId, ApiError> {
    Uuid::parse_str(value)
        .map(ProjectId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextLifecycleRequest(format!("project_id is invalid: {error}"))
        })
}

fn parse_benchmark_decision_id(value: &str) -> Result<BenchmarkDecisionId, ApiError> {
    Uuid::parse_str(value)
        .map(BenchmarkDecisionId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextLifecycleRequest(format!("decision_id is invalid: {error}"))
        })
}

fn parse_benchmark_execution_cohort_id(
    value: &str,
    field: &str,
) -> Result<BenchmarkExecutionCohortId, ApiError> {
    Uuid::parse_str(value)
        .map(BenchmarkExecutionCohortId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidBenchmarkWorkspaceRequest(format!("{field} is invalid: {error}"))
        })
}

fn parse_benchmark_workspace_uuid(value: &str, field: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|error| {
        ApiError::InvalidBenchmarkWorkspaceRequest(format!("{field} is invalid: {error}"))
    })
}

fn map_benchmark_workspace_projection_error(
    error: BenchmarkWorkspaceProjectionPersistenceError,
) -> ApiError {
    match error {
        BenchmarkWorkspaceProjectionPersistenceError::ReceiptUnavailable { .. } => {
            ApiError::BenchmarkWorkspaceNotFound
        }
        other => ApiError::BenchmarkWorkspaceStorage(other),
    }
}

fn parse_component_id(value: &str) -> Result<ComponentId, ApiError> {
    Uuid::parse_str(value)
        .map(ComponentId::from_uuid)
        .map_err(|error| {
            ApiError::InvalidContextLifecycleRequest(format!("component_id is invalid: {error}"))
        })
}

fn map_context_lifecycle_error(error: ContextLifecycleError) -> ApiError {
    match error {
        ContextLifecycleError::Storage(error) => {
            tracing::error!(error = %error, "context lifecycle storage operation failed");
            ApiError::ContextLifecycleStorage(error)
        }
        ContextLifecycleError::MaterializedSnapshotMissing { .. }
        | ContextLifecycleError::ComponentStateMissing { .. }
        | ContextLifecycleError::ComponentContentMissing { .. }
        | ContextLifecycleError::ComponentWitnessConflict { .. }
        | ContextLifecycleError::ComponentGraphNodeMissing { .. }
        | ContextLifecycleError::ComponentGraphNodeConflict { .. }
        | ContextLifecycleError::InvalidSnapshotCommitIdentifier => {
            tracing::warn!(error = %error, "context lifecycle state invariant failed");
            ApiError::ContextLifecycleStateConflict(
                "context lifecycle state is inconsistent".to_owned(),
            )
        }
        other => ApiError::InvalidContextLifecycleRequest(other.to_string()),
    }
}

fn map_benchmark_decision_comparison_error(
    error: BenchmarkDecisionComparisonServiceError,
) -> ApiError {
    match error {
        BenchmarkDecisionComparisonServiceError::Storage(error) => {
            tracing::error!(error = %error, "benchmark decision comparison storage operation failed");
            ApiError::BenchmarkEvidenceStorage(error)
        }
        BenchmarkDecisionComparisonServiceError::Comparison(error) => {
            tracing::warn!(error = %error, "benchmark decision comparison rejected");
            ApiError::BenchmarkDecisionComparisonUnavailable
        }
    }
}

fn map_benchmark_decision_definition_summary_error(
    error: BenchmarkDecisionDefinitionSummaryError,
) -> ApiError {
    match error {
        BenchmarkDecisionDefinitionSummaryError::Storage(error) => {
            tracing::error!(error = %error, "benchmark definition summary storage operation failed");
            ApiError::BenchmarkEvidenceStorage(error)
        }
        BenchmarkDecisionDefinitionSummaryError::DefinitionUnavailable
        | BenchmarkDecisionDefinitionSummaryError::DefinitionMembershipMismatch => {
            tracing::warn!(error = %error, "benchmark definition summary was unavailable");
            ApiError::BenchmarkEvidenceNotFound
        }
    }
}

fn map_benchmark_decision_discovery_error(
    error: BenchmarkDecisionDiscoveryServiceError,
) -> ApiError {
    match error {
        BenchmarkDecisionDiscoveryServiceError::Storage(error) => {
            tracing::error!(error = %error, "benchmark decision discovery storage operation failed");
            ApiError::BenchmarkEvidenceStorage(error)
        }
        BenchmarkDecisionDiscoveryServiceError::Definition(error) => {
            map_benchmark_decision_definition_summary_error(error)
        }
        BenchmarkDecisionDiscoveryServiceError::ScopeMismatch => {
            tracing::warn!("benchmark decision discovery returned an out-of-scope decision");
            ApiError::BenchmarkEvidenceNotFound
        }
    }
}

fn map_benchmark_decision_run_details_error(error: BenchmarkDecisionRunDetailsError) -> ApiError {
    match error {
        BenchmarkDecisionRunDetailsError::Storage(error) => {
            ApiError::BenchmarkEvidenceStorage(error)
        }
        BenchmarkDecisionRunDetailsError::RunUnavailable
        | BenchmarkDecisionRunDetailsError::RunMembershipMismatch => {
            tracing::warn!(error = %error, "benchmark decision run details were unavailable");
            ApiError::BenchmarkEvidenceNotFound
        }
    }
}

#[cfg(test)]
mod context_lifecycle_error_tests {
    use super::map_context_lifecycle_error;
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};
    use contextlab_context_core::ComponentId;
    use contextlab_storage::{ContextLifecycleError, StorageRepositoryError};
    use serde_json::json;
    use uuid::Uuid;

    #[tokio::test]
    async fn hides_internal_storage_causes_from_lifecycle_responses() {
        let response = map_context_lifecycle_error(ContextLifecycleError::Storage(
            StorageRepositoryError::Database {
                message: "database password leaked from storage".to_owned(),
            },
        ))
        .into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body is readable");
        let body =
            serde_json::from_slice::<serde_json::Value>(&body).expect("response body is JSON");
        assert_eq!(
            body,
            json!({
                "error": "context_lifecycle_storage_error",
                "message": "context lifecycle storage is unavailable",
            })
        );
        assert!(!body.to_string().contains("database password leaked"));
    }

    #[tokio::test]
    async fn preserves_lifecycle_conflict_status_without_exposing_storage_details() {
        let response = map_context_lifecycle_error(ContextLifecycleError::Storage(
            StorageRepositoryError::BranchHeadConflict {
                expected: Some("expected-commit".to_owned()),
                actual: Some("actual-commit".to_owned()),
            },
        ))
        .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body");
        let body =
            serde_json::from_slice::<serde_json::Value>(&body).expect("response body is JSON");
        assert_eq!(body["error"], "storage_branch_head_conflict");
        assert!(!body.to_string().contains("expected-commit"));
        assert!(!body.to_string().contains("actual-commit"));
    }

    #[tokio::test]
    async fn hides_lifecycle_state_invariant_details_from_conflict_responses() {
        let component_id = ComponentId::from_uuid(
            Uuid::parse_str("11111111-1111-4111-8111-111111111111")
                .expect("component fixture identifier"),
        );
        let response =
            map_context_lifecycle_error(ContextLifecycleError::ComponentGraphNodeConflict {
                component_id,
            })
            .into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body is readable");
        let body =
            serde_json::from_slice::<serde_json::Value>(&body).expect("response body is JSON");
        assert_eq!(
            body,
            json!({
                "error": "context_lifecycle_state_conflict",
                "message": "context lifecycle state is inconsistent",
            })
        );
        assert!(!body.to_string().contains(component_id.to_string().as_str()));
    }
}

fn parse_expected_branch_head(value: Option<&str>) -> Result<ExpectedBranchHead, ApiError> {
    let Some(value) = value else {
        return Ok(ExpectedBranchHead::Unborn);
    };
    let commit_uuid = Uuid::parse_str(value).map_err(|error| {
        ApiError::InvalidContextCommitRequest(format!(
            "expected_head_commit_id is invalid: {error}"
        ))
    })?;

    Ok(ExpectedBranchHead::Commit(CommitId::from_uuid(commit_uuid)))
}

pub(crate) fn sha256_digest<Request: Serialize>(request: &Request) -> String {
    let canonical = serde_json::to_vec(request).expect("commit write request is serializable");
    let digest = Sha256::digest(canonical);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }

    format!("sha256:{encoded}")
}

/// Returns configured model provider status without exposing secrets.
pub async fn providers(State(state): State<AppState>) -> Json<ProvidersResponse> {
    Json(ProvidersResponse {
        providers: state.provider_registry().public_statuses(),
    })
}
