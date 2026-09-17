//! Focused private benchmark breadth receipt across authoring, execution, and workspace reads.

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use contextlab_api::{
    AppState, WorkspaceCatalogRepositories, WorkspaceGraphRepositories, WorkspaceRepositories,
    build_protected_router_with_state,
};
use contextlab_auth::{
    AuthenticatedPrincipal, AuthorizationError, ContextAuthorizer, ContextPermission,
    HmacJwtAuthenticator, InMemoryProtectedRouteRateLimiter, ProtectedRouteRateLimitPolicy,
};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{BenchmarkCaseExecutionResult, MetricKind, MetricMeasurement};
use contextlab_model_gateway::ProviderRegistry;
use contextlab_storage::{
    BenchmarkCaseEvaluationRequest, BenchmarkCaseEvaluator, BenchmarkCaseEvaluatorError,
    BenchmarkDefinitionBinding, BenchmarkDefinitionBindingId, BenchmarkDefinitionBindingRepository,
    BenchmarkExecutionDisposition, BenchmarkExecutionRequest, BenchmarkExecutionResult,
    BenchmarkExecutionService, BenchmarkWorkspaceProjectionWriteDisposition, ContextCommitRecord,
    ContextGraphProjection, ContextRecord, IdempotencyKey, InMemoryContextGraphRepository,
    ProjectRecord, RequestDigest,
};
use contextlab_versioning::CommitId;
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tower::ServiceExt;
use uuid::Uuid;

const PROJECT_ID: &str = "22222222-2222-4222-8222-222222222222";
const CONTEXT_ID: &str = "33333333-3333-4333-8333-333333333333";
const BASELINE_COMMIT_ID: &str = "44444444-4444-4444-8444-444444444444";
const REVISED_COMMIT_ID: &str = "44444444-4444-4444-8444-444444444445";
const BASELINE_BINDING_ID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaa1";
const REVISED_BINDING_ID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaa2";
const BASELINE_DECISION_ID: &str = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeee1";
const REVISED_DECISION_ID: &str = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeee2";
const DATASET_A_ID: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbb1";
const DATASET_B_ID: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbb2";
const CASE_A1_ID: &str = "cccccccc-cccc-4ccc-8ccc-ccccccccccc1";
const CASE_A2_ID: &str = "cccccccc-cccc-4ccc-8ccc-ccccccccccc2";
const CASE_B1_ID: &str = "cccccccc-cccc-4ccc-8ccc-ccccccccccc3";
const CASE_B2_ID: &str = "cccccccc-cccc-4ccc-8ccc-ccccccccccc4";
const SUITE_ID: &str = "dddddddd-dddd-4ddd-8ddd-ddddddddddd1";
const AUTH_ISSUER: &str = "https://issuer.contextlab.test";
const AUTH_AUDIENCE: &str = "contextlab-web";

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("fixture UUID")
}

#[derive(Serialize)]
struct TestJwtClaims {
    sub: String,
    exp: usize,
    iss: String,
    aud: String,
}

struct AllowContextAccess;

#[async_trait]
impl ContextAuthorizer for AllowContextAccess {
    async fn authorize(
        &self,
        _principal: &AuthenticatedPrincipal,
        _context_id: ContextId,
        _permission: ContextPermission,
    ) -> Result<(), AuthorizationError> {
        Ok(())
    }
}

#[derive(Clone)]
struct CountingEvaluator {
    calls: Arc<AtomicUsize>,
    revised_commit: CommitId,
}

#[async_trait]
impl BenchmarkCaseEvaluator for CountingEvaluator {
    async fn evaluate_case(
        &self,
        request: BenchmarkCaseEvaluationRequest,
    ) -> Result<BenchmarkCaseExecutionResult, BenchmarkCaseEvaluatorError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let accuracy = if request.context_commit_id() == self.revised_commit {
            0.75
        } else {
            0.95
        };
        BenchmarkCaseExecutionResult::new(
            request.case().dataset_id(),
            request.case().case_id(),
            vec![
                MetricMeasurement::new(MetricKind::Accuracy, accuracy)
                    .map_err(|_| BenchmarkCaseEvaluatorError::new("metric"))?,
            ],
        )
        .map_err(|_| BenchmarkCaseEvaluatorError::new("result"))
    }
}

fn repository() -> InMemoryContextGraphRepository {
    InMemoryContextGraphRepository::new(ContextGraphProjection {
        projects: vec![ProjectRecord {
            id: PROJECT_ID.to_owned(),
            workspace_id: "11111111-1111-4111-8111-111111111111".to_owned(),
            name: "Benchmark breadth project".to_owned(),
            slug: "benchmark-breadth".to_owned(),
            created_at: Utc::now(),
        }],
        contexts: vec![ContextRecord {
            id: CONTEXT_ID.to_owned(),
            project_id: PROJECT_ID.to_owned(),
            experiment_id: None,
            name: "Benchmark breadth context".to_owned(),
            description: None,
            created_at: Utc::now(),
        }],
        commits: vec![
            ContextCommitRecord {
                id: BASELINE_COMMIT_ID.to_owned(),
                context_id: CONTEXT_ID.to_owned(),
                branch_name: "main".to_owned(),
                message: "Benchmark breadth baseline".to_owned(),
                parent_commit_ids: Vec::new(),
                changes: serde_json::json!([]),
                change_count: 0,
                authored_at: Utc::now(),
                created_at: Utc::now(),
            },
            ContextCommitRecord {
                id: REVISED_COMMIT_ID.to_owned(),
                context_id: CONTEXT_ID.to_owned(),
                branch_name: "main".to_owned(),
                message: "Benchmark breadth revised".to_owned(),
                parent_commit_ids: vec![BASELINE_COMMIT_ID.to_owned()],
                changes: serde_json::json!([]),
                change_count: 0,
                authored_at: Utc::now(),
                created_at: Utc::now(),
            },
        ],
        ..ContextGraphProjection::default()
    })
}

fn state(repository: &InMemoryContextGraphRepository) -> AppState {
    let repositories = WorkspaceRepositories::new(
        WorkspaceGraphRepositories::new(repository.clone(), repository.clone(), repository.clone()),
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
    );
    AppState::with_workspace_repositories(
        ProviderRegistry::from_env(std::iter::empty::<(&str, &str)>()),
        repositories,
    )
    .with_protected_write_dependencies(
        AllowContextAccess,
        repository.clone(),
        HmacJwtAuthenticator::new("test-secret", AUTH_ISSUER, AUTH_AUDIENCE)
            .expect("test authenticator"),
        InMemoryProtectedRouteRateLimiter::new(
            ProtectedRouteRateLimitPolicy::new(1_000, 3_600, 100).expect("test rate limit"),
        ),
    )
    .with_benchmark_definition_binding_writer(repository.clone())
    .with_benchmark_definition_binding_repository(repository.clone())
    .with_benchmark_evidence_repository(repository.clone())
    .with_benchmark_workspace_projection_repository(repository.clone())
}

fn token() -> String {
    encode(
        &Header::new(Algorithm::HS256),
        &TestJwtClaims {
            sub: "user:benchmark-breadth".to_owned(),
            exp: (Utc::now().timestamp() + 300) as usize,
            iss: AUTH_ISSUER.to_owned(),
            aud: AUTH_AUDIENCE.to_owned(),
        },
        &EncodingKey::from_secret(b"test-secret"),
    )
    .expect("test token")
}

fn authoring_body(binding_id: &str, commit_id: &str) -> Value {
    serde_json::json!({
        "schema_version": 1,
        "binding_id": binding_id,
        "branch_name": "main",
        "expected_head_commit_id": commit_id,
        "datasets": [
            {
                "id": DATASET_A_ID,
                "name": "Support cases",
                "cases": [
                    {"id": CASE_A2_ID, "name": "Second support case", "input": {"secret": "a2"}, "expected_output": {"mode": "exact", "value": {"answer": "a2"}}},
                    {"id": CASE_A1_ID, "name": "First support case", "input": {"secret": "a1"}, "expected_output": {"mode": "exact", "value": {"answer": "a1"}}}
                ]
            },
            {
                "id": DATASET_B_ID,
                "name": "Billing cases",
                "cases": [
                    {"id": CASE_B2_ID, "name": "Second billing case", "input": {"secret": "b2"}, "expected_output": {"mode": "exact", "value": {"answer": "b2"}}},
                    {"id": CASE_B1_ID, "name": "First billing case", "input": {"secret": "b1"}, "expected_output": {"mode": "exact", "value": {"answer": "b1"}}}
                ]
            }
        ],
        "suite": {
            "id": SUITE_ID,
            "name": "Support and billing gate",
            "dataset_ids": [DATASET_B_ID, DATASET_A_ID],
            "thresholds": [{"metric": "accuracy", "direction": "minimum", "value": 0.9}]
        }
    })
}

fn authoring_request(
    token: &str,
    commit_id: &str,
    idempotency_key: &str,
    body: Value,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!(
            "/api/v1/local/projects/{PROJECT_ID}/contexts/{CONTEXT_ID}/commits/{commit_id}/benchmark-definition-bindings"
        ))
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .header("idempotency-key", idempotency_key)
        .body(Body::from(body.to_string()))
        .expect("authoring request")
}

fn decision_diff_request(token: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/local/projects/{PROJECT_ID}/contexts/{CONTEXT_ID}/benchmark-decision-diffs?baseline_commit_id={BASELINE_COMMIT_ID}&baseline_decision_id={BASELINE_DECISION_ID}&revised_commit_id={REVISED_COMMIT_ID}&revised_decision_id={REVISED_DECISION_ID}"
        ))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("decision diff request")
}

async fn execute(
    repository: &InMemoryContextGraphRepository,
    binding: &BenchmarkDefinitionBinding,
    decision_id: &str,
    commit_id: &str,
    evaluator: &CountingEvaluator,
    idempotency_key: &str,
    request_digest: &str,
) -> BenchmarkExecutionResult {
    let request = BenchmarkExecutionRequest::from_definition_binding(
        contextlab_storage::BenchmarkDecisionId::from_uuid(uuid(decision_id)),
        binding,
        ProjectId::from_uuid(uuid(PROJECT_ID)),
        ContextId::from_uuid(uuid(CONTEXT_ID)),
        CommitId::from_uuid(uuid(commit_id)),
        "local-breadth-model",
        0.0,
        "breadth-evaluator",
        "v1",
        Utc::now(),
    )
    .expect("execution request")
    .with_idempotency(
        IdempotencyKey::new(idempotency_key).expect("idempotency key"),
        RequestDigest::new(request_digest).expect("request digest"),
    );

    BenchmarkExecutionService::new(repository, evaluator)
        .execute(request)
        .await
        .expect("execution result")
}

async fn response_json(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("response body")
        .to_bytes();
    (
        status,
        serde_json::from_slice(&body).expect("JSON response"),
    )
}

fn assert_redacted(value: &Value) {
    match value {
        Value::Object(map) => {
            for forbidden in [
                "cases",
                "input",
                "expected_output",
                "raw",
                "secret",
                "model_output",
                "measurements",
                "output",
                "model_version",
                "temperature",
            ] {
                assert!(!map.contains_key(forbidden), "response exposed {forbidden}");
            }
            for nested in map.values() {
                assert_redacted(nested);
            }
        }
        Value::Array(values) => {
            for nested in values {
                assert_redacted(nested);
            }
        }
        _ => {}
    }
}

#[tokio::test]
async fn private_benchmark_path_preserves_multi_dataset_case_breadth_and_scope() {
    let repository = repository();
    let state = state(&repository);
    let token = token();

    let baseline_authored = build_protected_router_with_state(state.clone())
        .oneshot(authoring_request(
            &token,
            BASELINE_COMMIT_ID,
            "benchmark-breadth-authoring-baseline",
            authoring_body(BASELINE_BINDING_ID, BASELINE_COMMIT_ID),
        ))
        .await
        .expect("baseline authoring response");
    let (status, baseline_authored_payload) = response_json(baseline_authored).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(baseline_authored_payload["project_id"], PROJECT_ID);
    assert_eq!(baseline_authored_payload["context_id"], CONTEXT_ID);
    assert_eq!(baseline_authored_payload["commit_id"], BASELINE_COMMIT_ID);
    assert_eq!(
        baseline_authored_payload["dataset_ids"],
        serde_json::json!([DATASET_A_ID, DATASET_B_ID])
    );
    assert_redacted(&baseline_authored_payload);

    let revised_authored = build_protected_router_with_state(state.clone())
        .oneshot(authoring_request(
            &token,
            REVISED_COMMIT_ID,
            "benchmark-breadth-authoring-revised",
            authoring_body(REVISED_BINDING_ID, REVISED_COMMIT_ID),
        ))
        .await
        .expect("revised authoring response");
    let (status, revised_authored_payload) = response_json(revised_authored).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(revised_authored_payload["project_id"], PROJECT_ID);
    assert_eq!(revised_authored_payload["context_id"], CONTEXT_ID);
    assert_eq!(revised_authored_payload["commit_id"], REVISED_COMMIT_ID);
    assert_eq!(
        revised_authored_payload["dataset_ids"],
        serde_json::json!([DATASET_A_ID, DATASET_B_ID])
    );
    assert_redacted(&revised_authored_payload);

    let baseline_binding = repository
        .get_benchmark_definition_binding(
            ProjectId::from_uuid(uuid(PROJECT_ID)),
            ContextId::from_uuid(uuid(CONTEXT_ID)),
            CommitId::from_uuid(uuid(BASELINE_COMMIT_ID)),
            BenchmarkDefinitionBindingId::from_uuid(uuid(BASELINE_BINDING_ID)),
        )
        .await
        .expect("baseline binding read")
        .expect("baseline authored binding");
    let revised_binding = repository
        .get_benchmark_definition_binding(
            ProjectId::from_uuid(uuid(PROJECT_ID)),
            ContextId::from_uuid(uuid(CONTEXT_ID)),
            CommitId::from_uuid(uuid(REVISED_COMMIT_ID)),
            BenchmarkDefinitionBindingId::from_uuid(uuid(REVISED_BINDING_ID)),
        )
        .await
        .expect("revised binding read")
        .expect("revised authored binding");
    let calls = Arc::new(AtomicUsize::new(0));
    let evaluator = CountingEvaluator {
        calls: calls.clone(),
        revised_commit: CommitId::from_uuid(uuid(REVISED_COMMIT_ID)),
    };
    let baseline_result = execute(
        &repository,
        &baseline_binding,
        BASELINE_DECISION_ID,
        BASELINE_COMMIT_ID,
        &evaluator,
        "benchmark-breadth-execution-baseline",
        "benchmark-breadth-request-baseline",
    )
    .await;
    let revised_result = execute(
        &repository,
        &revised_binding,
        REVISED_DECISION_ID,
        REVISED_COMMIT_ID,
        &evaluator,
        "benchmark-breadth-execution-revised",
        "benchmark-breadth-request-revised",
    )
    .await;
    assert_eq!(
        baseline_result.disposition(),
        BenchmarkExecutionDisposition::Created
    );
    assert_eq!(
        revised_result.disposition(),
        BenchmarkExecutionDisposition::Created
    );
    assert_eq!(calls.load(Ordering::SeqCst), 8);
    assert_eq!(baseline_result.evidence().dataset_ids().len(), 2);
    assert_eq!(baseline_result.evidence().run_ids().len(), 4);
    assert_eq!(revised_result.evidence().dataset_ids().len(), 2);
    assert_eq!(revised_result.evidence().run_ids().len(), 4);
    assert_eq!(
        baseline_result.evidence().context_commit_id().to_string(),
        BASELINE_COMMIT_ID
    );
    assert_eq!(
        revised_result.evidence().context_commit_id().to_string(),
        REVISED_COMMIT_ID
    );

    let baseline_replay = execute(
        &repository,
        &baseline_binding,
        BASELINE_DECISION_ID,
        BASELINE_COMMIT_ID,
        &evaluator,
        "benchmark-breadth-execution-baseline",
        "benchmark-breadth-request-baseline",
    )
    .await;
    let revised_replay = execute(
        &repository,
        &revised_binding,
        REVISED_DECISION_ID,
        REVISED_COMMIT_ID,
        &evaluator,
        "benchmark-breadth-execution-revised",
        "benchmark-breadth-request-revised",
    )
    .await;
    assert_eq!(
        baseline_replay.disposition(),
        BenchmarkExecutionDisposition::Replayed
    );
    assert_eq!(
        revised_replay.disposition(),
        BenchmarkExecutionDisposition::Replayed
    );
    assert_eq!(baseline_replay.evidence(), baseline_result.evidence());
    assert_eq!(revised_replay.evidence(), revised_result.evidence());
    assert_eq!(
        baseline_replay.workspace_projection_disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Replayed
    );
    assert_eq!(
        revised_replay.workspace_projection_disposition(),
        BenchmarkWorkspaceProjectionWriteDisposition::Replayed
    );
    assert_eq!(calls.load(Ordering::SeqCst), 8);

    let direct_diff = build_protected_router_with_state(state.clone())
        .oneshot(decision_diff_request(&token))
        .await
        .expect("direct decision diff response");
    let (status, direct_diff_payload) = response_json(direct_diff).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(direct_diff_payload["project_id"], PROJECT_ID);
    assert_eq!(direct_diff_payload["context_id"], CONTEXT_ID);
    assert_eq!(
        direct_diff_payload["baseline"]["commit_id"],
        BASELINE_COMMIT_ID
    );
    assert_eq!(
        direct_diff_payload["baseline"]["decision_id"],
        BASELINE_DECISION_ID
    );
    assert_eq!(
        direct_diff_payload["revised"]["commit_id"],
        REVISED_COMMIT_ID
    );
    assert_eq!(
        direct_diff_payload["revised"]["decision_id"],
        REVISED_DECISION_ID
    );
    assert_eq!(
        direct_diff_payload["status_change"],
        serde_json::json!({
            "baseline": "passed",
            "revised": "regressed"
        })
    );
    assert_eq!(
        direct_diff_payload["metric_changes"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    let accuracy_change = &direct_diff_payload["metric_changes"][0];
    assert_eq!(accuracy_change["kind"], "modified");
    assert_eq!(accuracy_change["metric"], "accuracy");
    for side in ["baseline", "revised"] {
        assert_eq!(accuracy_change[side]["sample_count"], 4);
        assert_eq!(accuracy_change[side]["required_sample_count"], 4);
    }
    assert_redacted(&direct_diff_payload);

    let direct_diff_replay = build_protected_router_with_state(state.clone())
        .oneshot(decision_diff_request(&token))
        .await
        .expect("direct decision diff replay response");
    let (status, direct_diff_replay_payload) = response_json(direct_diff_replay).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(direct_diff_replay_payload, direct_diff_payload);
    assert_redacted(&direct_diff_replay_payload);

    let baseline_cohort_id = baseline_result
        .workspace_projection_scope()
        .cohort_id()
        .as_uuid();
    let revised_cohort_id = revised_result
        .workspace_projection_scope()
        .cohort_id()
        .as_uuid();
    let workspace = build_protected_router_with_state(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/local/projects/{PROJECT_ID}/contexts/{CONTEXT_ID}/commits/{REVISED_COMMIT_ID}/benchmark-workspace/{revised_cohort_id}?baseline_commit_id={BASELINE_COMMIT_ID}&baseline_cohort_id={baseline_cohort_id}"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("workspace request"),
        )
        .await
        .expect("workspace response");
    let (status, workspace_payload) = response_json(workspace).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(workspace_payload["project_id"], PROJECT_ID);
    assert_eq!(workspace_payload["context_id"], CONTEXT_ID);
    assert!(
        workspace_payload.get("decision_pair_witness").is_none(),
        "cohort-bound comparison must not invent decision identity"
    );
    assert_eq!(
        workspace_payload["baseline"]["commit_id"],
        BASELINE_COMMIT_ID
    );
    assert_eq!(
        workspace_payload["baseline"]["cohort_id"],
        baseline_cohort_id.to_string()
    );
    assert_eq!(workspace_payload["revised"]["commit_id"], REVISED_COMMIT_ID);
    assert_eq!(
        workspace_payload["revised"]["cohort_id"],
        revised_cohort_id.to_string()
    );
    assert_eq!(
        workspace_payload["projection"]["receipt"]["cohort_id"],
        revised_cohort_id.to_string()
    );
    assert_eq!(
        workspace_payload["projection"]["datasets"],
        serde_json::json!([
            {"id": DATASET_A_ID, "name": "Support cases", "case_count": 2},
            {"id": DATASET_B_ID, "name": "Billing cases", "case_count": 2}
        ])
    );
    assert_eq!(
        workspace_payload["projection"]["runs"],
        serde_json::json!([
            {"dataset_id": DATASET_A_ID, "case_id": CASE_A1_ID, "metric_count": 1},
            {"dataset_id": DATASET_A_ID, "case_id": CASE_A2_ID, "metric_count": 1},
            {"dataset_id": DATASET_B_ID, "case_id": CASE_B1_ID, "metric_count": 1},
            {"dataset_id": DATASET_B_ID, "case_id": CASE_B2_ID, "metric_count": 1}
        ])
    );
    assert_eq!(workspace_payload["projection"]["scorecard"]["run_count"], 4);
    assert_eq!(
        workspace_payload["projection"]["scorecard"]["metrics"][0]["sample_count"],
        4
    );
    assert_eq!(
        workspace_payload["projection"]["scorecard"]["metrics"][0]["required_sample_count"],
        4
    );
    assert_eq!(
        workspace_payload["projection"]["scorecard"]["metrics"][0]["has_complete_coverage"],
        true
    );
    assert_eq!(
        workspace_payload["projection"]["regression_status"],
        "regressed"
    );
    assert_eq!(
        workspace_payload["projection"]["evaluation_diff"]["baseline_cohort_id"],
        baseline_cohort_id.to_string()
    );
    assert_eq!(
        workspace_payload["projection"]["evaluation_diff"]["revised_cohort_id"],
        revised_cohort_id.to_string()
    );
    assert_eq!(
        workspace_payload["projection"]["evaluation_diff"]["status_change"],
        serde_json::json!(["passed", "regressed"])
    );
    assert_eq!(
        workspace_payload["projection"]["evaluation_diff"]["metric_changes"][0]["metric"],
        "accuracy"
    );
    assert_eq!(
        workspace_payload["projection"]["evaluation_diff"]["metric_changes"][0]["change_kind"],
        "modified"
    );
    assert_eq!(
        workspace_payload["projection"]["evaluation_diff"]["metric_changes"][0]["baseline"]["sample_count"],
        4
    );
    assert_eq!(
        workspace_payload["projection"]["evaluation_diff"]["metric_changes"][0]["revised"]["sample_count"],
        4
    );
    assert_redacted(&workspace_payload);

    let workspace_single = build_protected_router_with_state(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/local/projects/{PROJECT_ID}/contexts/{CONTEXT_ID}/commits/{REVISED_COMMIT_ID}/benchmark-workspace/{revised_cohort_id}"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("single workspace request"),
        )
        .await
        .expect("single workspace response");
    let (status, workspace_single_payload) = response_json(workspace_single).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(workspace_single_payload["baseline"], Value::Null);
    assert!(
        workspace_single_payload
            .get("decision_pair_witness")
            .is_none(),
        "single cohort scope must not expose decision identity"
    );
    assert_redacted(&workspace_single_payload);

    let decision_workspace = build_protected_router_with_state(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/local/projects/{PROJECT_ID}/contexts/{CONTEXT_ID}/commits/{REVISED_COMMIT_ID}/benchmark-decisions/{REVISED_DECISION_ID}/workspace?baseline_commit_id={BASELINE_COMMIT_ID}&baseline_decision_id={BASELINE_DECISION_ID}"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("decision workspace request"),
        )
        .await
        .expect("decision workspace response");
    let (status, decision_workspace_payload) = response_json(decision_workspace).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(decision_workspace_payload["project_id"], PROJECT_ID);
    assert_eq!(decision_workspace_payload["context_id"], CONTEXT_ID);
    assert_eq!(
        decision_workspace_payload["revised"]["commit_id"],
        REVISED_COMMIT_ID
    );
    assert_eq!(
        decision_workspace_payload["revised"]["cohort_id"],
        revised_cohort_id.to_string()
    );
    assert_eq!(
        decision_workspace_payload["baseline"]["commit_id"],
        BASELINE_COMMIT_ID
    );
    assert_eq!(
        decision_workspace_payload["baseline"]["cohort_id"],
        baseline_cohort_id.to_string()
    );
    assert_eq!(
        decision_workspace_payload["decision_pair_witness"],
        serde_json::json!({
            "schema_version": 1,
            "project_id": PROJECT_ID,
            "context_id": CONTEXT_ID,
            "baseline": {
                "commit_id": BASELINE_COMMIT_ID,
                "decision_id": BASELINE_DECISION_ID
            },
            "revised": {
                "commit_id": REVISED_COMMIT_ID,
                "decision_id": REVISED_DECISION_ID
            }
        })
    );
    assert_eq!(
        decision_workspace_payload["projection"]["evaluation_diff"]["status_change"],
        serde_json::json!(["passed", "regressed"])
    );
    assert_redacted(&decision_workspace_payload);

    let decision_workspace_single = build_protected_router_with_state(state.clone())
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/local/projects/{PROJECT_ID}/contexts/{CONTEXT_ID}/commits/{REVISED_COMMIT_ID}/benchmark-decisions/{REVISED_DECISION_ID}/workspace"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("single decision workspace request"),
        )
        .await
        .expect("single decision workspace response");
    let (status, decision_workspace_single_payload) =
        response_json(decision_workspace_single).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(decision_workspace_single_payload["baseline"], Value::Null);
    assert!(
        decision_workspace_single_payload
            .get("decision_pair_witness")
            .is_none(),
        "single decision scope does not need a pair witness"
    );
    assert_redacted(&decision_workspace_single_payload);

    let drifted_decision = build_protected_router_with_state(state)
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/local/projects/{PROJECT_ID}/contexts/{CONTEXT_ID}/commits/{REVISED_COMMIT_ID}/benchmark-decisions/{BASELINE_DECISION_ID}/workspace"
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .expect("drifted decision workspace request"),
        )
        .await
        .expect("drifted decision workspace response");
    let (status, drifted_decision_payload) = response_json(drifted_decision).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        drifted_decision_payload["error"],
        "benchmark_workspace_not_found"
    );
}
