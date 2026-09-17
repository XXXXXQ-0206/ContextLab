//! Private local transport for exact benchmark-definition execution.

use crate::AppState;
use crate::routes::{ApiError, authorize_context_request};
use async_trait::async_trait;
use axum::{
    Json,
    extract::{Extension, Path, State, rejection::JsonRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response as HttpResponse},
};
use chrono::Utc;
use contextlab_auth::ContextPermission;
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_storage::{
    BenchmarkCaseEvaluator, BenchmarkDecisionId, BenchmarkDefinitionBindingId,
    BenchmarkEvidenceRepository, BenchmarkEvidenceWriter, BenchmarkExecutionDisposition,
    BenchmarkExecutionRequest, BenchmarkExecutionResult, BenchmarkExecutionService,
    BenchmarkExecutionServiceError, BenchmarkWorkspaceProjectionV1Writer,
    BenchmarkWorkspaceProjectionWriteDisposition, IdempotencyKey, RequestDigest,
    StorageRepositoryError,
};
use contextlab_versioning::CommitId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const REQUEST_SCHEMA_VERSION: u16 = 1;
const RESPONSE_SCHEMA: &str = "contextlab.local-benchmark-execution.v1";

/// Injected application port for a protected benchmark execution request.
#[async_trait]
pub(crate) trait BenchmarkExecutionAdapter: Send + Sync {
    async fn execute(
        &self,
        request: BenchmarkExecutionRequest,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
    ) -> Result<BenchmarkExecutionResult, BenchmarkExecutionServiceError>;
}

/// Adapter that delegates the protected transport to the reusable Rust execution service.
#[allow(dead_code)]
pub(crate) struct StorageBenchmarkExecutionAdapter<Repository, Evaluator> {
    repository: Repository,
    evaluator: Evaluator,
}

#[allow(dead_code)]
impl<Repository, Evaluator> StorageBenchmarkExecutionAdapter<Repository, Evaluator> {
    pub(crate) const fn new(repository: Repository, evaluator: Evaluator) -> Self {
        Self {
            repository,
            evaluator,
        }
    }
}

#[async_trait]
impl<Repository, Evaluator> BenchmarkExecutionAdapter
    for StorageBenchmarkExecutionAdapter<Repository, Evaluator>
where
    Repository: BenchmarkEvidenceRepository
        + BenchmarkEvidenceWriter
        + BenchmarkWorkspaceProjectionV1Writer
        + Send
        + Sync,
    Evaluator: BenchmarkCaseEvaluator,
{
    async fn execute(
        &self,
        request: BenchmarkExecutionRequest,
        idempotency_key: IdempotencyKey,
        request_digest: RequestDigest,
    ) -> Result<BenchmarkExecutionResult, BenchmarkExecutionServiceError> {
        let request = request.with_idempotency(idempotency_key, request_digest);
        BenchmarkExecutionService::new(&self.repository, &self.evaluator)
            .execute(request)
            .await
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    schema_version: u16,
    binding_id: Uuid,
    decision_id: Uuid,
    model_version: String,
    temperature: f32,
    evaluator_key: String,
    evaluator_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Response {
    schema_version: &'static str,
    disposition: &'static str,
    projection_disposition: &'static str,
    project_id: String,
    context_id: String,
    commit_id: String,
    binding_id: String,
    decision_id: String,
    suite_id: String,
    dataset_ids: Vec<String>,
    cohort_id: String,
}

pub(crate) enum Error {
    Authorization(ApiError),
    InvalidRequest,
    Unavailable,
    BindingNotFound,
    Storage(StorageRepositoryError),
    Request,
    Execution(BenchmarkExecutionServiceError),
}

impl IntoResponse for Error {
    fn into_response(self) -> HttpResponse {
        if let Self::Authorization(error) = self {
            return error.into_response();
        }

        let (status, error, message) = match self {
            Self::InvalidRequest | Self::Request => (
                StatusCode::BAD_REQUEST,
                "invalid_benchmark_execution_request",
                "benchmark execution request is invalid",
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "benchmark_execution_unavailable",
                "benchmark execution is unavailable",
            ),
            Self::BindingNotFound => (
                StatusCode::NOT_FOUND,
                "benchmark_definition_binding_not_found",
                "the exact benchmark definition binding was not found",
            ),
            Self::Storage(error) => storage_error_response(error),
            Self::Execution(error) => execution_error_response(error),
            Self::Authorization(_) => unreachable!("authorization errors return above"),
        };
        let mut response = (status, Json(ErrorResponse { error, message })).into_response();
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("private, no-store"),
        );
        response
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: &'static str,
}

/// Executes one exact immutable benchmark definition through the injected private adapter.
pub(crate) async fn create(
    State(state): State<AppState>,
    Extension(_principal): Extension<contextlab_auth::AuthenticatedPrincipal>,
    Path((raw_project_id, raw_context_id, raw_commit_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    request: Result<Json<Request>, JsonRejection>,
) -> Result<(StatusCode, HeaderMap, Json<Response>), Error> {
    let project_id = parse_project_id(&raw_project_id)?;
    let context_id = parse_context_id(&raw_context_id)?;
    let commit_id = parse_commit_id(&raw_commit_id)?;
    authorize_context_request(&state, &_principal, context_id, ContextPermission::Write)
        .await
        .map_err(Error::Authorization)?;

    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .ok_or(Error::InvalidRequest)
        .and_then(|value| IdempotencyKey::new(value).map_err(|_| Error::InvalidRequest))?;
    let Json(request) = request.map_err(|_| Error::InvalidRequest)?;
    if request.schema_version != REQUEST_SCHEMA_VERSION {
        return Err(Error::InvalidRequest);
    }
    let request_digest = RequestDigest::new(crate::routes::sha256_digest(&request))
        .map_err(|_| Error::InvalidRequest)?;

    let binding_id = BenchmarkDefinitionBindingId::from_uuid(request.binding_id);
    let binding = state
        .benchmark_definition_binding_repository()
        .ok_or(Error::Unavailable)?
        .get_benchmark_definition_binding(project_id, context_id, commit_id, binding_id)
        .await
        .map_err(Error::Storage)?
        .ok_or(Error::BindingNotFound)?;
    let execution_request = BenchmarkExecutionRequest::from_definition_binding(
        BenchmarkDecisionId::from_uuid(request.decision_id),
        &binding,
        project_id,
        context_id,
        commit_id,
        request.model_version,
        request.temperature,
        request.evaluator_key,
        request.evaluator_version,
        Utc::now(),
    )
    .map_err(|_| Error::Request)?;
    let result = state
        .benchmark_execution_adapter()
        .ok_or(Error::Unavailable)?
        .execute(execution_request, idempotency_key, request_digest)
        .await
        .map_err(Error::Execution)?;

    let evidence = result.evidence();
    let scope = result.workspace_projection_scope();
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok((
        match result.disposition() {
            BenchmarkExecutionDisposition::Created => StatusCode::CREATED,
            BenchmarkExecutionDisposition::Replayed => StatusCode::OK,
        },
        response_headers,
        Json(Response {
            schema_version: RESPONSE_SCHEMA,
            disposition: execution_disposition_label(result.disposition()),
            projection_disposition: projection_disposition_label(
                result.workspace_projection_disposition(),
            ),
            project_id: evidence.project_id().to_string(),
            context_id: evidence.context_id().to_string(),
            commit_id: evidence.context_commit_id().to_string(),
            binding_id: binding.id().to_string(),
            decision_id: evidence.decision_id().to_string(),
            suite_id: evidence.suite_id().to_string(),
            dataset_ids: evidence
                .dataset_ids()
                .iter()
                .map(ToString::to_string)
                .collect(),
            cohort_id: scope.cohort_id().as_uuid().to_string(),
        }),
    ))
}

fn execution_disposition_label(disposition: BenchmarkExecutionDisposition) -> &'static str {
    match disposition {
        BenchmarkExecutionDisposition::Created => "created",
        BenchmarkExecutionDisposition::Replayed => "replayed",
    }
}

fn projection_disposition_label(
    disposition: BenchmarkWorkspaceProjectionWriteDisposition,
) -> &'static str {
    match disposition {
        BenchmarkWorkspaceProjectionWriteDisposition::Created => "created",
        BenchmarkWorkspaceProjectionWriteDisposition::Replayed => "replayed",
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

fn storage_error_response(
    error: StorageRepositoryError,
) -> (StatusCode, &'static str, &'static str) {
    match error {
        StorageRepositoryError::ScopeUnavailable { .. } => (
            StatusCode::NOT_FOUND,
            "benchmark_execution_scope_not_found",
            "benchmark execution scope was not found",
        ),
        StorageRepositoryError::IdempotencyKeyReused { .. }
        | StorageRepositoryError::BranchHeadConflict { .. }
        | StorageRepositoryError::BenchmarkDefinitionBindingConflict { .. }
        | StorageRepositoryError::BenchmarkEvidenceDigestConflict { .. }
        | StorageRepositoryError::BenchmarkDefinitionConflict { .. } => (
            StatusCode::CONFLICT,
            "benchmark_execution_conflict",
            "benchmark execution conflicts with immutable state",
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
            "benchmark_execution_unavailable",
            "benchmark execution is unavailable",
        ),
    }
}

fn execution_error_response(
    error: BenchmarkExecutionServiceError,
) -> (StatusCode, &'static str, &'static str) {
    match error {
        BenchmarkExecutionServiceError::SuiteUnavailable
        | BenchmarkExecutionServiceError::DatasetUnavailable => (
            StatusCode::NOT_FOUND,
            "benchmark_definition_not_found",
            "the bound benchmark definition is unavailable",
        ),
        BenchmarkExecutionServiceError::Plan(_)
        | BenchmarkExecutionServiceError::BindingSelection(_) => (
            StatusCode::BAD_REQUEST,
            "invalid_benchmark_execution_request",
            "benchmark execution request is invalid",
        ),
        BenchmarkExecutionServiceError::StoredEvidenceMismatch => (
            StatusCode::CONFLICT,
            "benchmark_execution_conflict",
            "benchmark execution conflicts with immutable evidence",
        ),
        BenchmarkExecutionServiceError::Storage(error) => storage_error_response(error),
        BenchmarkExecutionServiceError::Evaluator(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "benchmark_evaluator_unavailable",
            "the configured benchmark evaluator is unavailable",
        ),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            "benchmark_execution_unavailable",
            "benchmark execution is unavailable",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_model_gateway::ProviderRegistry;

    struct RecordingAdapter;

    #[async_trait]
    impl BenchmarkExecutionAdapter for RecordingAdapter {
        async fn execute(
            &self,
            _request: BenchmarkExecutionRequest,
            _idempotency_key: IdempotencyKey,
            _request_digest: RequestDigest,
        ) -> Result<BenchmarkExecutionResult, BenchmarkExecutionServiceError> {
            Err(BenchmarkExecutionServiceError::StoredEvidenceMismatch)
        }
    }

    #[test]
    fn injected_execution_adapter_is_explicit_and_available_only_when_configured() {
        let state = crate::AppState::new(ProviderRegistry::from_env(std::iter::empty::<(
            &str,
            &str,
        )>()));
        assert!(state.benchmark_execution_adapter().is_none());
        let state = state.with_benchmark_execution_adapter(RecordingAdapter);
        assert!(state.benchmark_execution_adapter().is_some());
    }

    #[test]
    fn request_rejects_unknown_fields_and_wrong_schema() {
        let value = serde_json::json!({
            "schema_version": 2,
            "binding_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
            "decision_id": "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            "model_version": "model",
            "temperature": 0.0,
            "evaluator_key": "local",
            "evaluator_version": "v1",
            "cases": []
        });
        assert!(serde_json::from_value::<Request>(value).is_err());
    }

    #[test]
    fn response_schema_and_dispositions_are_stable_and_redacted() {
        assert_eq!(RESPONSE_SCHEMA, "contextlab.local-benchmark-execution.v1");
        assert_eq!(
            execution_disposition_label(BenchmarkExecutionDisposition::Created),
            "created"
        );
        assert_eq!(
            projection_disposition_label(BenchmarkWorkspaceProjectionWriteDisposition::Replayed),
            "replayed"
        );
    }
}
