//! Private local transport for immutable benchmark-definition authoring.

use crate::AppState;
use crate::routes::{ApiError, authorize_context_request};
use axum::{
    Json,
    extract::{Extension, Path, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, Uri, header},
    response::{IntoResponse, Response as HttpResponse},
};
use chrono::Utc;
use contextlab_auth::{
    AuthenticatedPrincipal, AuthorizationAuditEvent, AuthorizationDecision, AuthorizationError,
    ContextPermission,
};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkCaseId, BenchmarkDataset, BenchmarkDatasetId, BenchmarkExpectedOutput,
    BenchmarkSuite, BenchmarkSuiteId, MetricKind, RegressionThreshold, ThresholdDirection,
};
use contextlab_storage::{
    BenchmarkDefinitionBindingCommand, BenchmarkDefinitionBindingSummary,
    BenchmarkDefinitionBindingWriteDisposition, StorageRepositoryError,
};
use contextlab_versioning::{BranchName, CommitId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const RESPONSE_SCHEMA: &str = "contextlab.local-benchmark-definition-authoring.v1";
const INSPECTION_RESPONSE_SCHEMA: &str =
    "contextlab.local-benchmark-definition-binding-inspection.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BindingListResponse {
    schema_version: &'static str,
    project_id: String,
    context_id: String,
    commit_id: String,
    bindings: Vec<BindingSummaryResponse>,
}

#[derive(Debug, Clone, Serialize)]
struct BindingSummaryResponse {
    binding_id: String,
    project_id: String,
    context_id: String,
    commit_id: String,
    branch_name: String,
    definition_schema_version: u16,
    suite_id: String,
    suite_name: String,
    dataset_ids: Vec<String>,
    dataset_names: Vec<String>,
    captured_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    schema_version: u16,
    binding_id: Uuid,
    branch_name: String,
    expected_head_commit_id: Uuid,
    datasets: Vec<DatasetRequest>,
    suite: SuiteRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DatasetRequest {
    id: Uuid,
    name: String,
    cases: Vec<CaseRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CaseRequest {
    id: Uuid,
    name: String,
    input: Value,
    expected_output: ExpectedOutputRequest,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(
    tag = "mode",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum ExpectedOutputRequest {
    Unspecified,
    Exact(Value),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SuiteRequest {
    id: Uuid,
    name: String,
    dataset_ids: Vec<Uuid>,
    thresholds: Vec<ThresholdRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ThresholdRequest {
    metric: MetricKind,
    direction: ThresholdDirection,
    value: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Response {
    schema_version: &'static str,
    disposition: &'static str,
    message: BilingualText,
    project_id: String,
    context_id: String,
    commit_id: String,
    binding_id: String,
    branch_name: String,
    definition_schema_version: u16,
    dataset_ids: Vec<String>,
    suite_id: String,
    captured_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct BilingualText {
    en: &'static str,
    zh: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: &'static str,
}

pub(crate) enum Error {
    InvalidRequest,
    Unavailable,
    Storage(StorageRepositoryError),
    WriteForbidden,
    AuthorizationUnavailable,
    AuthorizationAuditUnavailable,
}

impl IntoResponse for Error {
    fn into_response(self) -> HttpResponse {
        let (status, error, message) = match self {
            Self::InvalidRequest => (
                StatusCode::BAD_REQUEST,
                "invalid_benchmark_definition_authoring_request",
                "benchmark definition authoring request is invalid",
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "benchmark_definition_authoring_unavailable",
                "benchmark definition authoring is unavailable",
            ),
            Self::Storage(error) => match error {
                StorageRepositoryError::ScopeUnavailable { .. } => (
                    StatusCode::NOT_FOUND,
                    "benchmark_definition_scope_not_found",
                    "benchmark definition authoring scope was not found",
                ),
                StorageRepositoryError::InvalidScope { .. } => (
                    StatusCode::BAD_REQUEST,
                    "invalid_benchmark_definition_authoring_request",
                    "benchmark definition authoring request is invalid",
                ),
                StorageRepositoryError::IdempotencyKeyReused { .. } => (
                    StatusCode::CONFLICT,
                    "storage_idempotency_conflict",
                    "benchmark definition authoring request conflicts with an existing request",
                ),
                StorageRepositoryError::BranchHeadConflict { .. } => (
                    StatusCode::CONFLICT,
                    "storage_branch_head_conflict",
                    "benchmark definition authoring branch head has changed",
                ),
                StorageRepositoryError::BenchmarkDefinitionConflict { .. }
                | StorageRepositoryError::BenchmarkDefinitionBindingConflict { .. } => (
                    StatusCode::CONFLICT,
                    "storage_benchmark_definition_conflict",
                    "benchmark definition authoring conflicts with immutable definitions",
                ),
                StorageRepositoryError::GuardedWriteForbidden => (
                    StatusCode::FORBIDDEN,
                    "context_write_forbidden",
                    "context write permission is forbidden",
                ),
                StorageRepositoryError::GuardedWriteAuthorizationUnavailable => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "authorization_unavailable",
                    "authorization state is unavailable",
                ),
                _ => (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "benchmark_definition_authoring_unavailable",
                    "benchmark definition authoring is unavailable",
                ),
            },
            Self::WriteForbidden => (
                StatusCode::FORBIDDEN,
                "context_write_forbidden",
                "context write permission is forbidden",
            ),
            Self::AuthorizationUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "authorization_unavailable",
                "authorization state is unavailable",
            ),
            Self::AuthorizationAuditUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "authorization_audit_unavailable",
                "authorization audit storage is unavailable",
            ),
        };
        (status, Json(ErrorResponse { error, message })).into_response()
    }
}

/// Persists one complete private benchmark-definition binding at an exact Context commit.
pub(crate) async fn create(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    request: Result<Json<Request>, JsonRejection>,
) -> Result<(StatusCode, HeaderMap, Json<Response>), Error> {
    let project_id = parse_project_id(&raw_project_id)?;
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id)?;
    authorize_context_write(&state, &principal, context_id).await?;

    let Json(request) = request.map_err(|_| Error::InvalidRequest)?;
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .ok_or(Error::InvalidRequest)?;
    let request_digest = request_digest(&request);
    let branch = BranchName::new(request.branch_name.clone()).map_err(|_| Error::InvalidRequest)?;
    let expected_head = CommitId::from_uuid(request.expected_head_commit_id);
    let schema_version = request.schema_version;
    let binding_id = request.binding_id;
    let (datasets, suite) = request.into_domain()?;
    let command = BenchmarkDefinitionBindingCommand::new(
        principal,
        binding_id,
        project_id,
        context_id,
        commit_id,
        branch,
        expected_head,
        idempotency_key,
        request_digest,
        datasets,
        suite,
        Utc::now(),
        schema_version,
    )
    .map_err(|_| Error::InvalidRequest)?;
    let result = state
        .benchmark_definition_binding_writer()
        .ok_or(Error::Unavailable)?
        .persist_benchmark_definition_binding(command)
        .await
        .map_err(Error::Storage)?;
    let binding = result.binding();
    let (status, disposition, message) = match result.disposition() {
        BenchmarkDefinitionBindingWriteDisposition::Created => (
            StatusCode::CREATED,
            "created",
            BilingualText {
                en: "Benchmark definitions authored.",
                zh: "Benchmark 定义已创建。",
            },
        ),
        BenchmarkDefinitionBindingWriteDisposition::Replayed => (
            StatusCode::OK,
            "replayed",
            BilingualText {
                en: "Benchmark definition authoring request replayed.",
                zh: "Benchmark 定义创建请求已重放。",
            },
        ),
    };
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );

    Ok((
        status,
        response_headers,
        Json(Response {
            schema_version: RESPONSE_SCHEMA,
            disposition,
            message,
            project_id: binding.project_id().to_string(),
            context_id: binding.context_id().to_string(),
            commit_id: binding.context_commit_id().to_string(),
            binding_id: binding.id().to_string(),
            branch_name: binding.branch().as_str().to_owned(),
            definition_schema_version: binding.schema_version(),
            dataset_ids: binding
                .dataset_ids()
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            suite_id: binding.suite().id().to_string(),
            captured_at: binding.captured_at().to_rfc3339(),
        }),
    ))
}

/// Lists redacted benchmark-definition bindings at one exact immutable Context commit.
pub(crate) async fn list(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id)): Path<(String, String, String)>,
    uri: Uri,
) -> Result<Json<BindingListResponse>, ApiError> {
    if uri.query().is_some() {
        return Err(ApiError::InvalidBenchmarkDefinitionBindingRequest(
            "query parameters are not supported".to_owned(),
        ));
    }
    let project_id = parse_binding_project_id(&raw_project_id)?;
    let context_id = parse_binding_context_id(&raw_context_id)?;
    let commit_id = parse_binding_commit_id(&raw_commit_id)?;
    authorize_context_request(&state, &principal, context_id, ContextPermission::Read).await?;
    let reader = state
        .benchmark_definition_binding_repository()
        .ok_or(ApiError::BenchmarkDefinitionBindingUnavailable)?;
    let bindings = reader
        .list_benchmark_definition_bindings_at_commit(project_id, context_id, commit_id)
        .await
        .map_err(ApiError::BenchmarkDefinitionBindingStorage)?;
    Ok(Json(BindingListResponse {
        schema_version: INSPECTION_RESPONSE_SCHEMA,
        project_id: project_id.to_string(),
        context_id: context_id.to_string(),
        commit_id: commit_id.to_string(),
        bindings: bindings
            .iter()
            .map(|binding| {
                BindingSummaryResponse::from_summary(
                    &BenchmarkDefinitionBindingSummary::from_binding(binding),
                )
            })
            .collect(),
    }))
}

impl BindingSummaryResponse {
    fn from_summary(summary: &BenchmarkDefinitionBindingSummary) -> Self {
        Self {
            binding_id: summary.binding_id().to_string(),
            project_id: summary.project_id().to_string(),
            context_id: summary.context_id().to_string(),
            commit_id: summary.context_commit_id().to_string(),
            branch_name: summary.branch().as_str().to_owned(),
            definition_schema_version: summary.schema_version(),
            suite_id: summary.suite_id().to_string(),
            suite_name: summary.suite_name().to_owned(),
            dataset_ids: summary
                .dataset_ids()
                .iter()
                .map(ToString::to_string)
                .collect(),
            dataset_names: summary.dataset_names().to_vec(),
            captured_at: summary.captured_at().to_rfc3339(),
        }
    }
}

fn parse_binding_project_id(value: &str) -> Result<ProjectId, ApiError> {
    Uuid::parse_str(value)
        .map(ProjectId::from_uuid)
        .map_err(|_| {
            ApiError::InvalidBenchmarkDefinitionBindingRequest(
                "project_id must be a UUID".to_owned(),
            )
        })
}

fn parse_binding_context_id(value: &str) -> Result<ContextId, ApiError> {
    Uuid::parse_str(value)
        .map(ContextId::from_uuid)
        .map_err(|_| {
            ApiError::InvalidBenchmarkDefinitionBindingRequest(
                "context_id must be a UUID".to_owned(),
            )
        })
}

fn parse_binding_commit_id(value: &str) -> Result<CommitId, ApiError> {
    Uuid::parse_str(value)
        .map(CommitId::from_uuid)
        .map_err(|_| {
            ApiError::InvalidBenchmarkDefinitionBindingRequest(
                "commit_id must be a UUID".to_owned(),
            )
        })
}

impl Request {
    fn into_domain(self) -> Result<(Vec<BenchmarkDataset>, BenchmarkSuite), Error> {
        let datasets = self
            .datasets
            .into_iter()
            .map(DatasetRequest::into_domain)
            .collect::<Result<Vec<_>, _>>()?;
        let thresholds = self
            .suite
            .thresholds
            .into_iter()
            .map(|threshold| {
                RegressionThreshold::new(threshold.metric, threshold.direction, threshold.value)
                    .map_err(|_| Error::InvalidRequest)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let suite = BenchmarkSuite::with_id(
            BenchmarkSuiteId::from_uuid(self.suite.id),
            self.suite.name,
            self.suite
                .dataset_ids
                .into_iter()
                .map(BenchmarkDatasetId::from_uuid)
                .collect(),
            thresholds,
        )
        .map_err(|_| Error::InvalidRequest)?;
        Ok((datasets, suite))
    }
}

impl DatasetRequest {
    fn into_domain(self) -> Result<BenchmarkDataset, Error> {
        let cases = self
            .cases
            .into_iter()
            .map(CaseRequest::into_domain)
            .collect::<Result<Vec<_>, _>>()?;
        BenchmarkDataset::with_id(BenchmarkDatasetId::from_uuid(self.id), self.name, cases)
            .map_err(|_| Error::InvalidRequest)
    }
}

impl CaseRequest {
    fn into_domain(self) -> Result<BenchmarkCase, Error> {
        let expected_output = match self.expected_output {
            ExpectedOutputRequest::Unspecified => BenchmarkExpectedOutput::Unspecified,
            ExpectedOutputRequest::Exact(value) => BenchmarkExpectedOutput::Exact(value),
        };
        BenchmarkCase::with_id(
            BenchmarkCaseId::from_uuid(self.id),
            self.name,
            self.input,
            expected_output,
        )
        .map_err(|_| Error::InvalidRequest)
    }
}

async fn authorize_context_write(
    state: &AppState,
    principal: &AuthenticatedPrincipal,
    context_id: ContextId,
) -> Result<(), Error> {
    let decision = match state
        .context_authorizer()
        .authorize(principal, context_id, ContextPermission::Write)
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
            ContextPermission::Write,
            decision,
        ))
        .await
        .map_err(|_| Error::AuthorizationAuditUnavailable)?;
    match decision {
        AuthorizationDecision::Granted => Ok(()),
        AuthorizationDecision::Forbidden => Err(Error::WriteForbidden),
        AuthorizationDecision::Unavailable => Err(Error::AuthorizationUnavailable),
    }
}

fn parse_project_id(value: &str) -> Result<ProjectId, Error> {
    Uuid::parse_str(value)
        .map(ProjectId::from_uuid)
        .map_err(|_| Error::InvalidRequest)
}

fn parse_context_id(value: &str) -> Result<ContextId, Error> {
    Uuid::parse_str(value)
        .map(ContextId::from_uuid)
        .map_err(|_| Error::InvalidRequest)
}

fn parse_commit_id(value: &str) -> Result<CommitId, Error> {
    Uuid::parse_str(value)
        .map(CommitId::from_uuid)
        .map_err(|_| Error::InvalidRequest)
}

fn request_digest(request: &Request) -> String {
    let canonical = serde_json::to_vec(request).expect("authoring request is serializable");
    format!("sha256:{:x}", Sha256::digest(canonical))
}
