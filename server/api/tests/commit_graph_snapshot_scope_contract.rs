//! Black-box contract coverage for typed commit-associated Context Graph snapshots.

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{TimeZone, Utc};
use contextlab_api::{
    AppState, WorkspaceCatalogRepositories, WorkspaceGraphRepositories, WorkspaceRepositories,
    build_protected_router_with_state, build_router_with_state,
};
use contextlab_auth::{
    AuthenticatedPrincipal, AuthorizationError, ContextAuthorizer, ContextPermission,
    HmacJwtAuthenticator, InMemoryProtectedRouteRateLimiter, ProtectedRouteRateLimitPolicy,
};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_diff_engine::GraphDiff;
use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
use contextlab_model_gateway::ProviderRegistry;
use contextlab_storage::{
    COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1, CommitGraphSnapshot, CommitGraphSnapshotScope,
    ContextCommitRecord, ContextGraphProjection, ContextRecord, InMemoryContextGraphRepository,
    ProjectRecord, WorkspaceRecord,
};
use contextlab_versioning::CommitId;
use http_body_util::BodyExt;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

const WORKSPACE_ID: &str = "11111111-1111-4111-8111-111111111111";
const PROJECT_ID: &str = "22222222-2222-4222-8222-222222222222";
const CONTEXT_ID: &str = "33333333-3333-4333-8333-333333333333";
const ORIGINAL_COMMIT_ID: &str = "44444444-4444-4444-8444-444444444444";
const REVISED_COMMIT_ID: &str = "55555555-5555-4555-8555-555555555555";
const AUTH_ISSUER: &str = "https://issuer.contextlab.test";
const AUTH_AUDIENCE: &str = "contextlab-web";

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).expect("fixture UUID")
}

fn timestamp(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(seconds, 0)
        .single()
        .expect("fixture timestamp")
}

fn graph(label: &str) -> ContextGraph {
    let mut graph = ContextGraph::new();
    graph
        .add_node(
            GraphNode::new(
                format!("context:{CONTEXT_ID}"),
                GraphNodeKind::Context,
                label,
            )
            .expect("graph node"),
        )
        .expect("insert graph node");
    graph
}

fn projection() -> ContextGraphProjection {
    ContextGraphProjection {
        workspaces: vec![WorkspaceRecord {
            id: WORKSPACE_ID.to_owned(),
            name: "Typed Scope Workspace".to_owned(),
            slug: "typed-scope".to_owned(),
            created_at: timestamp(1),
        }],
        projects: vec![ProjectRecord {
            id: PROJECT_ID.to_owned(),
            workspace_id: WORKSPACE_ID.to_owned(),
            name: "Typed Scope Project".to_owned(),
            slug: "typed-scope".to_owned(),
            created_at: timestamp(1),
        }],
        contexts: vec![ContextRecord {
            id: CONTEXT_ID.to_owned(),
            project_id: PROJECT_ID.to_owned(),
            experiment_id: None,
            name: "Typed Scope Context".to_owned(),
            description: None,
            created_at: timestamp(1),
        }],
        commits: vec![
            ContextCommitRecord {
                id: ORIGINAL_COMMIT_ID.to_owned(),
                context_id: CONTEXT_ID.to_owned(),
                branch_name: "main".to_owned(),
                message: "Original typed snapshot".to_owned(),
                parent_commit_ids: Vec::new(),
                changes: serde_json::to_value(vec![
                    contextlab_versioning::ContextChange::created_context("Typed Scope Context"),
                ])
                .expect("root change"),
                change_count: 1,
                authored_at: timestamp(1),
                created_at: timestamp(1),
            },
            ContextCommitRecord {
                id: REVISED_COMMIT_ID.to_owned(),
                context_id: CONTEXT_ID.to_owned(),
                branch_name: "main".to_owned(),
                message: "Revised typed snapshot".to_owned(),
                parent_commit_ids: vec![ORIGINAL_COMMIT_ID.to_owned()],
                changes: serde_json::to_value(vec![
                    contextlab_versioning::ContextChange::updated_metadata(
                        contextlab_context_core::ContextMetadata::new(timestamp(2)),
                        "Update typed scope Context",
                    ),
                ])
                .expect("metadata change"),
                change_count: 1,
                authored_at: timestamp(2),
                created_at: timestamp(2),
            },
        ],
        ..ContextGraphProjection::default()
    }
}

fn snapshot_repository() -> InMemoryContextGraphRepository {
    let scope = |commit_id: &str| {
        CommitGraphSnapshotScope::new(
            ProjectId::from_uuid(uuid(PROJECT_ID)),
            ContextId::from_uuid(uuid(CONTEXT_ID)),
            CommitId::from_uuid(uuid(commit_id)),
        )
    };
    InMemoryContextGraphRepository::with_commit_graph_snapshots(
        projection(),
        [
            CommitGraphSnapshot::new(
                scope(ORIGINAL_COMMIT_ID),
                graph("Original typed graph"),
                timestamp(1),
                COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
            )
            .expect("original snapshot"),
            CommitGraphSnapshot::new(
                scope(REVISED_COMMIT_ID),
                graph("Revised typed graph"),
                timestamp(2),
                COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1,
            )
            .expect("revised snapshot"),
        ],
    )
    .expect("snapshot repository")
}

fn version_backed_graph_diff_router() -> Router {
    let repository = snapshot_repository();
    build_router_with_state(AppState::with_workspace_repositories(
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
        )
        .with_context_commit_history_repository(repository.clone()),
    ))
}

struct AllowContextReads;

#[async_trait]
impl ContextAuthorizer for AllowContextReads {
    async fn authorize(
        &self,
        _principal: &AuthenticatedPrincipal,
        _context_id: ContextId,
        _permission: ContextPermission,
    ) -> Result<(), AuthorizationError> {
        Ok(())
    }
}

#[derive(Serialize)]
struct TestJwtClaims {
    sub: String,
    exp: usize,
    iss: String,
    aud: String,
}

fn protected_graph_diff_router() -> (Router, String) {
    protected_graph_diff_router_with_witness(true)
}

fn protected_graph_diff_router_without_witness() -> (Router, String) {
    protected_graph_diff_router_with_witness(false)
}

fn protected_graph_diff_router_with_witness(include_witness: bool) -> (Router, String) {
    let repository = snapshot_repository();
    let mut state = AppState::with_workspace_repositories(
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
        )
        .with_context_commit_history_repository(repository.clone()),
    )
    .with_protected_write_dependencies(
        AllowContextReads,
        snapshot_repository(),
        HmacJwtAuthenticator::new("test-secret", AUTH_ISSUER, AUTH_AUDIENCE)
            .expect("test authenticator"),
        InMemoryProtectedRouteRateLimiter::new(
            ProtectedRouteRateLimitPolicy::new(100, 3_600, 100).expect("test rate limit"),
        ),
    )
    .with_context_lifecycle_repository(snapshot_repository());
    if include_witness {
        state = state.with_context_graph_review_witness_repository(repository);
    }
    let token = encode(
        &Header::new(Algorithm::HS256),
        &TestJwtClaims {
            sub: "user:commit-graph-scope".to_owned(),
            exp: (Utc::now().timestamp() + 300) as usize,
            iss: AUTH_ISSUER.to_owned(),
            aud: AUTH_AUDIENCE.to_owned(),
        },
        &EncodingKey::from_secret(b"test-secret"),
    )
    .expect("test token");

    (build_protected_router_with_state(state), token)
}

fn protected_graph_diff_request(
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

#[tokio::test]
async fn public_version_backed_graph_diff_route_is_retired() {
    let uri = format!(
        "/api/v1/contexts/{CONTEXT_ID}/graph-diff?original_commit_id={ORIGINAL_COMMIT_ID}&revised_commit_id={REVISED_COMMIT_ID}"
    );
    let response = version_backed_graph_diff_router()
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("graph diff request"),
        )
        .await
        .expect("graph diff response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn protected_version_backed_graph_diff_is_authenticated_and_exactly_scoped() {
    let (router, token) = protected_graph_diff_router();

    let missing_auth = router
        .clone()
        .oneshot(protected_graph_diff_request(
            CONTEXT_ID,
            ORIGINAL_COMMIT_ID,
            REVISED_COMMIT_ID,
            None,
        ))
        .await
        .expect("missing auth response");
    assert_eq!(missing_auth.status(), StatusCode::UNAUTHORIZED);

    let response = router
        .clone()
        .oneshot(protected_graph_diff_request(
            CONTEXT_ID,
            ORIGINAL_COMMIT_ID,
            REVISED_COMMIT_ID,
            Some(&token),
        ))
        .await
        .expect("protected graph diff response");
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "private, no-store"
    );
    let (status, payload) = response_json(response).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload["context_id"], CONTEXT_ID);
    assert_eq!(payload["pair_witness"]["schema_version"], 1);
    assert_eq!(payload["pair_witness"]["project_id"], PROJECT_ID);
    assert_eq!(payload["pair_witness"]["context_id"], CONTEXT_ID);
    assert_eq!(
        payload["pair_witness"]["baseline_commit_id"],
        ORIGINAL_COMMIT_ID
    );
    assert_eq!(
        payload["pair_witness"]["revised_commit_id"],
        REVISED_COMMIT_ID
    );
    assert_eq!(payload["original"]["commit_id"], ORIGINAL_COMMIT_ID);
    assert_eq!(payload["original"]["schema_version"], 1);
    assert_eq!(payload["revised"]["commit_id"], REVISED_COMMIT_ID);
    assert_eq!(payload["revised"]["schema_version"], 1);
    assert_eq!(
        payload["diff"]["modified_nodes"][0]["node_id"],
        format!("context:{CONTEXT_ID}")
    );
    assert_eq!(
        payload["diff"]["modified_nodes"][0]["original_label"],
        "Original typed graph"
    );
    assert_eq!(
        payload["diff"]["modified_nodes"][0]["revised_label"],
        "Revised typed graph"
    );
    assert!(
        payload["diff"]["modified_nodes"]
            .as_array()
            .is_some_and(|nodes| !nodes.is_empty())
    );

    let (status, payload) = response_json(
        router
            .oneshot(protected_graph_diff_request(
                CONTEXT_ID,
                ORIGINAL_COMMIT_ID,
                ORIGINAL_COMMIT_ID,
                Some(&token),
            ))
            .await
            .expect("identical commit response"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(payload["error"], "invalid_commit_graph_diff_query");
}

#[tokio::test]
async fn protected_version_backed_graph_diff_fails_closed_without_witness() {
    let (router, token) = protected_graph_diff_router_without_witness();
    let response = router
        .oneshot(protected_graph_diff_request(
            CONTEXT_ID,
            ORIGINAL_COMMIT_ID,
            REVISED_COMMIT_ID,
            Some(&token),
        ))
        .await
        .expect("missing witness response");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn fixture_changes_are_graph_diff_derived() {
    let original = graph("Original typed graph");
    let revised = graph("Revised typed graph");
    let diff = GraphDiff::between(&original, &revised);

    assert_eq!(diff.modified_nodes().len(), 1);
    assert_eq!(
        diff.modified_nodes()[0].node_id(),
        format!("context:{CONTEXT_ID}")
    );
}
