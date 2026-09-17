//! Private read projection for durable Context branch heads.

use crate::{AppState, routes::ApiError};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use contextlab_auth::{AuthenticatedPrincipal, ContextPermission};
use contextlab_context_core::ContextId;
use contextlab_storage::{ContextBranchHead, ContextBranchRepositoryError};
use serde::Serialize;
use uuid::Uuid;

const SCHEMA_VERSION: &str = "contextlab.local-context-branch-heads.v1";

/// Private response containing the durable heads of every branch in one Context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalContextBranchHeadsResponse {
    /// Stable response schema identifier.
    pub schema_version: &'static str,
    /// Context whose branch heads were read.
    pub context_id: String,
    /// Branch heads in server-owned branch-name order.
    pub branches: Vec<LocalContextBranchHeadResponse>,
}

/// Safe branch-head response item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalContextBranchHeadResponse {
    /// Validated branch name.
    pub branch_name: String,
    /// Durable head commit, or `null` for an unborn branch.
    pub head_commit_id: Option<String>,
    /// Monotonic durable branch revision.
    pub revision: u64,
}

/// Lists durable branch heads for one exact, authorized Context.
pub async fn list(
    State(state): State<AppState>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(raw_context_id): Path<String>,
) -> Result<Json<LocalContextBranchHeadsResponse>, ApiError> {
    let context_id = parse_context_id(&raw_context_id)?;
    crate::routes::authorize_context_request(
        &state,
        &principal,
        context_id,
        ContextPermission::Read,
    )
    .await?;

    let repository = state.context_branch_repository();
    let heads = repository
        .list_context_branch_heads(context_id)
        .await
        .map_err(map_repository_error)?;

    Ok(Json(response_from_heads(context_id, heads)))
}

fn parse_context_id(value: &str) -> Result<ContextId, ApiError> {
    Uuid::parse_str(value)
        .map(ContextId::from_uuid)
        .map_err(|_| ApiError::InvalidContextBranchHeadsRequest("context_id is invalid".to_owned()))
}

fn response_from_heads(
    context_id: ContextId,
    mut heads: Vec<ContextBranchHead>,
) -> LocalContextBranchHeadsResponse {
    heads.sort_unstable_by(|left, right| left.branch().as_str().cmp(right.branch().as_str()));

    LocalContextBranchHeadsResponse {
        schema_version: SCHEMA_VERSION,
        context_id: context_id.to_string(),
        branches: heads
            .into_iter()
            .map(|head| LocalContextBranchHeadResponse {
                branch_name: head.branch().as_str().to_owned(),
                head_commit_id: head.head_commit_id().map(|commit_id| commit_id.to_string()),
                revision: head.revision(),
            })
            .collect(),
    }
}

fn map_repository_error(error: ContextBranchRepositoryError) -> ApiError {
    match error {
        ContextBranchRepositoryError::UnknownContext { .. } => {
            ApiError::ContextBranchHeadsUnavailable
        }
        ContextBranchRepositoryError::UnknownBranch { .. }
        | ContextBranchRepositoryError::InvalidStoredBranchName { .. }
        | ContextBranchRepositoryError::InvalidStoredRevision { .. }
        | ContextBranchRepositoryError::RevisionOverflow { .. }
        | ContextBranchRepositoryError::BranchHeadIntegrityViolation { .. }
        | ContextBranchRepositoryError::InMemoryStateUnavailable
        | ContextBranchRepositoryError::Database { .. } => ApiError::ContextBranchHeadsUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_versioning::{BranchName, CommitId};
    use serde_json::json;

    fn head(
        context_id: ContextId,
        branch: &str,
        commit_id: Option<CommitId>,
        revision: u64,
    ) -> ContextBranchHead {
        ContextBranchHead::new(
            context_id,
            BranchName::new(branch).expect("valid branch"),
            commit_id,
            revision,
        )
    }

    #[test]
    fn response_uses_contract_schema_and_server_owned_branch_order() {
        let context_id = ContextId::new();
        let commit_id = CommitId::new();
        let response = response_from_heads(
            context_id,
            vec![
                head(context_id, "release", Some(commit_id), 4),
                head(context_id, "main", None, 0),
            ],
        );

        assert_eq!(response.schema_version, SCHEMA_VERSION);
        assert_eq!(response.context_id, context_id.to_string());
        assert_eq!(response.branches[0].branch_name, "main");
        assert_eq!(response.branches[0].head_commit_id, None);
        assert_eq!(response.branches[0].revision, 0);
        assert_eq!(response.branches[1].branch_name, "release");
        assert_eq!(
            response.branches[1].head_commit_id,
            Some(commit_id.to_string())
        );
        assert_eq!(response.branches[1].revision, 4);
    }

    #[test]
    fn response_contains_no_private_storage_content() {
        let context_id = ContextId::new();
        let response = response_from_heads(context_id, Vec::new());
        let body = serde_json::to_value(response).expect("response serializes");

        assert_eq!(
            body,
            json!({
                "schema_version": SCHEMA_VERSION,
                "context_id": context_id.to_string(),
                "branches": []
            })
        );
    }

    #[test]
    fn context_id_parser_is_fail_closed() {
        assert!(parse_context_id("not-a-uuid").is_err());
        assert!(parse_context_id("").is_err());
        assert!(parse_context_id(&ContextId::new().to_string()).is_ok());
    }

    #[test]
    fn repository_errors_are_redacted_at_the_api_boundary() {
        let context_id = ContextId::new();
        let branch = BranchName::new("private/branch").expect("valid branch");
        let error =
            map_repository_error(ContextBranchRepositoryError::BranchHeadIntegrityViolation {
                context_id,
                branch,
                head_commit_id: CommitId::new(),
            });

        assert_eq!(
            error.to_string(),
            "local Context branch heads are unavailable"
        );
        assert!(!error.to_string().contains("private/branch"));
        assert!(!error.to_string().contains(&context_id.to_string()));
    }
}
