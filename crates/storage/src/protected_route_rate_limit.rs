//! PostgreSQL-backed private protected-route rate limiting.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use contextlab_auth::{
    ProtectedRouteOperation, ProtectedRouteRateLimitKey, ProtectedRouteRateLimitPolicy,
    ProtectedRouteRateLimiter, RateLimitDecision, RateLimitError,
};
use sqlx::{PgPool, Postgres, Transaction};

const LOCK_SHARED_RATE_LIMIT_SQL: &str = r#"
SELECT pg_advisory_xact_lock(hashtextextended('contextlab.protected_route_rate_limit.v1', 0))
"#;

const LOCK_KEY_SQL: &str = r#"
SELECT pg_advisory_xact_lock(
    hashtextextended($1 || E'\x1F' || $2 || E'\x1F' || $3, 0)
)
"#;

const SELECT_DATABASE_TIME_SQL: &str = "SELECT clock_timestamp()";

const SELECT_CONFIGURATION_SQL: &str = r#"
SELECT max_requests, window_seconds, max_tracked_keys
FROM public.protected_route_rate_limit_configurations
WHERE singleton = TRUE
"#;

const INSERT_CONFIGURATION_SQL: &str = r#"
INSERT INTO public.protected_route_rate_limit_configurations
    (singleton, max_requests, window_seconds, max_tracked_keys)
VALUES (TRUE, $1, $2, $3)
ON CONFLICT (singleton) DO NOTHING
"#;

const DELETE_EXPIRED_STATES_SQL: &str = r#"
DELETE FROM public.protected_route_rate_limit_states
WHERE expires_at <= $1
"#;

const SELECT_STATE_FOR_UPDATE_SQL: &str = r#"
SELECT request_timestamps
FROM public.protected_route_rate_limit_states
WHERE identity_source = $1
  AND principal_id = $2
  AND operation = $3
FOR UPDATE
"#;

const COUNT_ACTIVE_STATES_SQL: &str = r#"
SELECT COUNT(*)
FROM public.protected_route_rate_limit_states
"#;

const INSERT_STATE_SQL: &str = r#"
INSERT INTO public.protected_route_rate_limit_states
    (identity_source, principal_id, operation, request_timestamps, expires_at)
VALUES ($1, $2, $3, $4, $5)
"#;

const UPDATE_STATE_SQL: &str = r#"
UPDATE public.protected_route_rate_limit_states
SET request_timestamps = $4,
    expires_at = $5
WHERE identity_source = $1
  AND principal_id = $2
  AND operation = $3
"#;

/// Private PostgreSQL adapter shared by protected-route API replicas.
#[derive(Debug, Clone)]
pub struct PostgresProtectedRouteRateLimiter {
    pool: PgPool,
    policy: ProtectedRouteRateLimitPolicy,
}

impl PostgresProtectedRouteRateLimiter {
    /// Creates an adapter from a PostgreSQL pool and the deployment-wide policy.
    #[must_use]
    pub const fn from_pool(pool: PgPool, policy: ProtectedRouteRateLimitPolicy) -> Self {
        Self { pool, policy }
    }

    fn operation_name(operation: ProtectedRouteOperation) -> &'static str {
        match operation {
            ProtectedRouteOperation::ContextLifecycleRead => "context_lifecycle_read",
            ProtectedRouteOperation::ContextBranchHeadRead => "context_branch_head_read",
            ProtectedRouteOperation::ContextCommitGraphDiffRead => "context_commit_graph_diff_read",
            ProtectedRouteOperation::KnowledgeMemoryProjectionRead => {
                "knowledge_memory_projection_read"
            }
            ProtectedRouteOperation::WorkflowExecutionStatusRead => {
                "workflow_execution_status_read"
            }
            ProtectedRouteOperation::BenchmarkDecisionRead => "benchmark_decision_read",
            ProtectedRouteOperation::BenchmarkDecisionDiffRead => "benchmark_decision_diff_read",
            ProtectedRouteOperation::BenchmarkWorkspaceRead => "benchmark_workspace_read",
            ProtectedRouteOperation::ContextCommitWrite => "context_commit_write",
            ProtectedRouteOperation::BenchmarkDefinitionAuthoringWrite => {
                "benchmark_definition_authoring_write"
            }
            ProtectedRouteOperation::BenchmarkDefinitionBindingRead => {
                "benchmark_definition_binding_read"
            }
            ProtectedRouteOperation::BenchmarkExecutionWrite => "benchmark_execution_write",
        }
    }

    fn policy_values(&self) -> (i32, i32, i32) {
        (
            self.policy.max_requests() as i32,
            self.policy.window_seconds() as i32,
            self.policy.max_tracked_principals() as i32,
        )
    }

    fn matches_configuration(&self, configuration: (i32, i32, i32)) -> bool {
        let (max_requests, window_seconds, max_tracked_keys) = configuration;
        (max_requests, window_seconds, max_tracked_keys) == self.policy_values()
    }

    fn retry_after_seconds(
        oldest_timestamp: DateTime<Utc>,
        now: DateTime<Utc>,
        window: Duration,
    ) -> Result<u64, RateLimitError> {
        let retry_at = oldest_timestamp
            .checked_add_signed(window)
            .ok_or(RateLimitError::Unavailable)?;
        let remaining = retry_at.signed_duration_since(now);
        let remaining_nanoseconds = remaining
            .num_nanoseconds()
            .ok_or(RateLimitError::Unavailable)?;
        if remaining_nanoseconds <= 0 {
            return Ok(1);
        }

        let rounded_seconds = (u64::try_from(remaining_nanoseconds)
            .map_err(|_| RateLimitError::Unavailable)?
            .saturating_add(999_999_999))
            / 1_000_000_000;
        Ok(rounded_seconds.max(1))
    }

    async fn check_in_transaction(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        key: &ProtectedRouteRateLimitKey,
    ) -> Result<RateLimitDecision, RateLimitError> {
        let identity_source = key.principal_identity().source().as_str();
        let principal_id = key.principal_id().as_str();
        let operation = Self::operation_name(key.operation());

        sqlx::query(LOCK_KEY_SQL)
            .bind(identity_source)
            .bind(principal_id)
            .bind(operation)
            .execute(&mut **transaction)
            .await
            .map_err(|_| RateLimitError::Unavailable)?;

        let policy_values = self.policy_values();
        sqlx::query(INSERT_CONFIGURATION_SQL)
            .bind(policy_values.0)
            .bind(policy_values.1)
            .bind(policy_values.2)
            .execute(&mut **transaction)
            .await
            .map_err(|_| RateLimitError::Unavailable)?;
        let configuration = sqlx::query_as::<_, (i32, i32, i32)>(SELECT_CONFIGURATION_SQL)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|_| RateLimitError::Unavailable)?;
        if !self.matches_configuration(configuration) {
            return Err(RateLimitError::Unavailable);
        }

        let window = Duration::seconds(self.policy.window_seconds() as i64);
        let mut now = sqlx::query_scalar::<_, DateTime<Utc>>(SELECT_DATABASE_TIME_SQL)
            .fetch_one(&mut **transaction)
            .await
            .map_err(|_| RateLimitError::Unavailable)?;
        let mut state = sqlx::query_scalar::<_, Vec<DateTime<Utc>>>(SELECT_STATE_FOR_UPDATE_SQL)
            .bind(identity_source)
            .bind(principal_id)
            .bind(operation)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(|_| RateLimitError::Unavailable)?;

        if state.is_none() {
            sqlx::query(LOCK_SHARED_RATE_LIMIT_SQL)
                .execute(&mut **transaction)
                .await
                .map_err(|_| RateLimitError::Unavailable)?;
            now = sqlx::query_scalar::<_, DateTime<Utc>>(SELECT_DATABASE_TIME_SQL)
                .fetch_one(&mut **transaction)
                .await
                .map_err(|_| RateLimitError::Unavailable)?;
            sqlx::query(DELETE_EXPIRED_STATES_SQL)
                .bind(now)
                .execute(&mut **transaction)
                .await
                .map_err(|_| RateLimitError::Unavailable)?;
            state = sqlx::query_scalar::<_, Vec<DateTime<Utc>>>(SELECT_STATE_FOR_UPDATE_SQL)
                .bind(identity_source)
                .bind(principal_id)
                .bind(operation)
                .fetch_optional(&mut **transaction)
                .await
                .map_err(|_| RateLimitError::Unavailable)?;
        }

        let cutoff = now
            .checked_sub_signed(window)
            .ok_or(RateLimitError::Unavailable)?;
        let Some(raw_timestamps) = state else {
            let active_state_count = sqlx::query_scalar::<_, i64>(COUNT_ACTIVE_STATES_SQL)
                .fetch_one(&mut **transaction)
                .await
                .map_err(|_| RateLimitError::Unavailable)?;
            if active_state_count >= i64::from(policy_values.2) {
                return Err(RateLimitError::Unavailable);
            }
            let expires_at = now
                .checked_add_signed(window)
                .ok_or(RateLimitError::Unavailable)?;
            sqlx::query(INSERT_STATE_SQL)
                .bind(identity_source)
                .bind(principal_id)
                .bind(operation)
                .bind(vec![now])
                .bind(expires_at)
                .execute(&mut **transaction)
                .await
                .map_err(|_| RateLimitError::Unavailable)?;
            return Ok(RateLimitDecision::Allowed);
        };

        if raw_timestamps
            .windows(2)
            .any(|timestamps| timestamps[0] > timestamps[1])
            || raw_timestamps
                .last()
                .is_some_and(|timestamp| *timestamp > now)
        {
            return Err(RateLimitError::Unavailable);
        }
        let mut timestamps = raw_timestamps
            .into_iter()
            .filter(|timestamp| *timestamp > cutoff)
            .collect::<Vec<_>>();

        if !timestamps.is_empty() && timestamps.len() >= self.policy.max_requests() as usize {
            let retry_after_seconds = Self::retry_after_seconds(
                *timestamps.first().ok_or(RateLimitError::Unavailable)?,
                now,
                window,
            )?;
            let expires_at = timestamps
                .last()
                .and_then(|timestamp| timestamp.checked_add_signed(window))
                .ok_or(RateLimitError::Unavailable)?;
            sqlx::query(UPDATE_STATE_SQL)
                .bind(identity_source)
                .bind(principal_id)
                .bind(operation)
                .bind(timestamps)
                .bind(expires_at)
                .execute(&mut **transaction)
                .await
                .map_err(|_| RateLimitError::Unavailable)?;

            return Ok(RateLimitDecision::Rejected {
                retry_after_seconds,
            });
        }

        timestamps.push(now);
        let expires_at = now
            .checked_add_signed(window)
            .ok_or(RateLimitError::Unavailable)?;
        sqlx::query(UPDATE_STATE_SQL)
            .bind(identity_source)
            .bind(principal_id)
            .bind(operation)
            .bind(timestamps)
            .bind(expires_at)
            .execute(&mut **transaction)
            .await
            .map_err(|_| RateLimitError::Unavailable)?;

        Ok(RateLimitDecision::Allowed)
    }
}

#[async_trait]
impl ProtectedRouteRateLimiter for PostgresProtectedRouteRateLimiter {
    async fn check(
        &self,
        key: ProtectedRouteRateLimitKey,
    ) -> Result<RateLimitDecision, RateLimitError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|_| RateLimitError::Unavailable)?;
        let decision = self.check_in_transaction(&mut transaction, &key).await?;
        transaction
            .commit()
            .await
            .map_err(|_| RateLimitError::Unavailable)?;
        Ok(decision)
    }
}
