//! Protected-route rate limiting contracts and adapters.

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{PrincipalId, PrincipalIdentity};
use thiserror::Error;

/// Private protected-route operation used as a stable rate-limit dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectedRouteOperation {
    /// Read exact private Context lifecycle state at a materialized commit.
    ContextLifecycleRead,
    /// Discover durable branch heads for one private Context.
    ContextBranchHeadRead,
    /// Compare two immutable Context Graph snapshots at exact Context commits.
    ContextCommitGraphDiffRead,
    /// Read the private provider-free Knowledge/Memory projection at a Context commit.
    KnowledgeMemoryProjectionRead,
    /// Read one private Workflow execution status and replay provenance projection.
    WorkflowExecutionStatusRead,
    /// Read one private benchmark decision at an exact Context commit.
    BenchmarkDecisionRead,
    /// Compare two private benchmark decisions at exact Context commits.
    BenchmarkDecisionDiffRead,
    /// Read one private benchmark workspace projection at an exact receipt scope.
    BenchmarkWorkspaceRead,
    /// Append a guarded Context commit.
    ContextCommitWrite,
    /// Author an immutable benchmark definition bound to a Context commit.
    BenchmarkDefinitionAuthoringWrite,
    /// Read immutable benchmark-definition bindings at an exact Context commit.
    BenchmarkDefinitionBindingRead,
    /// Execute one private benchmark definition binding.
    BenchmarkExecutionWrite,
}

/// Validated principal and operation key for protected-route limiting.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtectedRouteRateLimitKey {
    principal_identity: PrincipalIdentity,
    operation: ProtectedRouteOperation,
}

impl ProtectedRouteRateLimitKey {
    /// Builds a key from a validated principal identity and protected operation.
    #[must_use]
    pub const fn new(
        principal_identity: PrincipalIdentity,
        operation: ProtectedRouteOperation,
    ) -> Self {
        Self {
            principal_identity,
            operation,
        }
    }

    /// Returns the issuer-scoped authenticated principal dimension.
    #[must_use]
    pub const fn principal_identity(&self) -> &PrincipalIdentity {
        &self.principal_identity
    }

    /// Returns the authenticated principal subject dimension.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        self.principal_identity.subject()
    }

    /// Returns the protected operation dimension.
    #[must_use]
    pub const fn operation(&self) -> ProtectedRouteOperation {
        self.operation
    }
}

/// Bounded sliding-window policy for one protected-route limiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectedRouteRateLimitPolicy {
    max_requests: u32,
    window_seconds: u64,
    max_tracked_principals: usize,
}

impl ProtectedRouteRateLimitPolicy {
    /// Validates request, window, and tracked-principal bounds.
    pub const fn new(
        max_requests: u32,
        window_seconds: u64,
        max_tracked_principals: usize,
    ) -> Result<Self, RateLimitPolicyError> {
        if max_requests == 0
            || max_requests > 1_000
            || window_seconds == 0
            || window_seconds > 3_600
            || max_tracked_principals == 0
            || max_tracked_principals > 100_000
        {
            return Err(RateLimitPolicyError::InvalidConfiguration);
        }

        Ok(Self {
            max_requests,
            window_seconds,
            max_tracked_principals,
        })
    }

    /// Returns the accepted requests per sliding window.
    #[must_use]
    pub const fn max_requests(self) -> u32 {
        self.max_requests
    }

    /// Returns the sliding-window duration in seconds.
    #[must_use]
    pub const fn window_seconds(self) -> u64 {
        self.window_seconds
    }

    /// Returns the maximum number of active principal-operation keys.
    #[must_use]
    pub const fn max_tracked_principals(self) -> usize {
        self.max_tracked_principals
    }
}

/// Errors produced while validating a protected-route rate-limit policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RateLimitPolicyError {
    /// One or more policy values are outside their supported bounds.
    #[error("protected-route rate-limit policy is invalid")]
    InvalidConfiguration,
}

/// Result of checking one protected-route request against its policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitDecision {
    /// The request is within its sliding-window allowance.
    Allowed,
    /// The request exceeds its allowance until the reported delay elapses.
    Rejected {
        /// Whole seconds clients should wait before retrying.
        retry_after_seconds: u64,
    },
}

/// Errors produced when rate-limit state cannot safely make a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RateLimitError {
    /// Rate-limit state or its monotonic clock is unavailable.
    #[error("protected-route rate-limit state is unavailable")]
    Unavailable,
}

/// Port for checking authenticated protected-route requests.
#[async_trait]
pub trait ProtectedRouteRateLimiter: Send + Sync {
    /// Checks and atomically records one request when capacity permits it.
    async fn check(
        &self,
        key: ProtectedRouteRateLimitKey,
    ) -> Result<RateLimitDecision, RateLimitError>;
}

trait MonotonicClock: Send + Sync {
    fn now(&self) -> Result<Duration, RateLimitError>;
}

#[derive(Debug)]
struct SystemMonotonicClock {
    origin: Instant,
}

impl Default for SystemMonotonicClock {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> Result<Duration, RateLimitError> {
        Ok(self.origin.elapsed())
    }
}

/// Bounded in-process sliding-window limiter for protected routes.
pub struct InMemoryProtectedRouteRateLimiter {
    policy: ProtectedRouteRateLimitPolicy,
    clock: Arc<dyn MonotonicClock>,
    state: Mutex<RateLimitState>,
}

#[derive(Debug, Default)]
struct RateLimitState {
    requests_by_key: HashMap<ProtectedRouteRateLimitKey, VecDeque<Duration>>,
    expiry_index: BTreeMap<Duration, HashSet<ProtectedRouteRateLimitKey>>,
    last_observed_at: Option<Duration>,
}

impl RateLimitState {
    fn prune_expired(&mut self, now: Duration) {
        while let Some((expiry, _)) = self.expiry_index.first_key_value() {
            if *expiry > now {
                break;
            }
            let Some((_, expired_keys)) = self.expiry_index.pop_first() else {
                break;
            };
            for expired_key in expired_keys {
                self.requests_by_key.remove(&expired_key);
            }
        }
    }

    fn remove_expiry(
        &mut self,
        key: &ProtectedRouteRateLimitKey,
        expiry: Duration,
    ) -> Result<(), RateLimitError> {
        let remove_bucket = {
            let keys = self
                .expiry_index
                .get_mut(&expiry)
                .ok_or(RateLimitError::Unavailable)?;
            if !keys.remove(key) {
                return Err(RateLimitError::Unavailable);
            }
            keys.is_empty()
        };

        if remove_bucket {
            self.expiry_index.remove(&expiry);
        }
        Ok(())
    }

    fn insert_expiry(&mut self, key: ProtectedRouteRateLimitKey, expiry: Duration) {
        self.expiry_index.entry(expiry).or_default().insert(key);
    }
}

impl InMemoryProtectedRouteRateLimiter {
    /// Creates an empty limiter that uses the process monotonic clock.
    #[must_use]
    pub fn new(policy: ProtectedRouteRateLimitPolicy) -> Self {
        Self {
            policy,
            clock: Arc::new(SystemMonotonicClock::default()),
            state: Mutex::new(RateLimitState::default()),
        }
    }

    #[cfg(test)]
    fn with_clock(policy: ProtectedRouteRateLimitPolicy, clock: Arc<dyn MonotonicClock>) -> Self {
        Self {
            policy,
            clock,
            state: Mutex::new(RateLimitState::default()),
        }
    }
}

#[async_trait]
impl ProtectedRouteRateLimiter for InMemoryProtectedRouteRateLimiter {
    async fn check(
        &self,
        key: ProtectedRouteRateLimitKey,
    ) -> Result<RateLimitDecision, RateLimitError> {
        let mut state = self.state.lock().await;
        let now = self.clock.now()?;
        if state
            .last_observed_at
            .is_some_and(|last_observed_at| now < last_observed_at)
        {
            return Err(RateLimitError::Unavailable);
        }
        state.last_observed_at = Some(now);
        let window = Duration::from_secs(self.policy.window_seconds());
        let new_expiry = now.checked_add(window).ok_or(RateLimitError::Unavailable)?;

        state.prune_expired(now);

        if let Some(requests) = state.requests_by_key.get_mut(&key) {
            if requests
                .back()
                .is_some_and(|latest_timestamp| *latest_timestamp > now)
            {
                return Err(RateLimitError::Unavailable);
            }
            if let Some(expiry_boundary) = now.checked_sub(window) {
                while requests
                    .front()
                    .is_some_and(|timestamp| *timestamp <= expiry_boundary)
                {
                    requests.pop_front();
                }
            }
            if requests.len() >= self.policy.max_requests() as usize {
                let oldest = *requests.front().ok_or(RateLimitError::Unavailable)?;
                let elapsed = now.checked_sub(oldest).ok_or(RateLimitError::Unavailable)?;
                let remaining = window
                    .checked_sub(elapsed)
                    .ok_or(RateLimitError::Unavailable)?;
                let retry_after_seconds = remaining
                    .as_secs()
                    .saturating_add(u64::from(remaining.subsec_nanos() > 0))
                    .max(1);

                return Ok(RateLimitDecision::Rejected {
                    retry_after_seconds,
                });
            }

            let old_expiry = requests
                .back()
                .and_then(|latest_timestamp| latest_timestamp.checked_add(window))
                .ok_or(RateLimitError::Unavailable)?;
            state.remove_expiry(&key, old_expiry)?;
            let requests = state
                .requests_by_key
                .get_mut(&key)
                .ok_or(RateLimitError::Unavailable)?;
            requests.push_back(now);
            state.insert_expiry(key, new_expiry);
            return Ok(RateLimitDecision::Allowed);
        }

        if state.requests_by_key.len() >= self.policy.max_tracked_principals() {
            return Err(RateLimitError::Unavailable);
        }

        state
            .requests_by_key
            .insert(key.clone(), VecDeque::from([now]));
        state.insert_expiry(key, new_expiry);
        Ok(RateLimitDecision::Allowed)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    use std::time::Duration;

    use super::*;

    #[derive(Debug, Default)]
    struct FakeMonotonicClock {
        elapsed_millis: AtomicU64,
    }

    impl FakeMonotonicClock {
        fn advance(&self, duration: Duration) {
            let millis = u64::try_from(duration.as_millis()).expect("test duration fits in u64");
            self.elapsed_millis.fetch_add(millis, Ordering::SeqCst);
        }

        fn set(&self, duration: Duration) {
            let millis = u64::try_from(duration.as_millis()).expect("test duration fits in u64");
            self.elapsed_millis.store(millis, Ordering::SeqCst);
        }
    }

    impl MonotonicClock for FakeMonotonicClock {
        fn now(&self) -> Result<Duration, RateLimitError> {
            Ok(Duration::from_millis(
                self.elapsed_millis.load(Ordering::SeqCst),
            ))
        }
    }

    fn key(principal_id: &str) -> ProtectedRouteRateLimitKey {
        ProtectedRouteRateLimitKey::new(
            PrincipalIdentity::new(
                crate::IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new(principal_id).expect("principal"),
            ),
            ProtectedRouteOperation::ContextCommitWrite,
        )
    }

    #[test]
    fn keys_isolate_equal_subjects_from_different_identity_sources() {
        let tenant_a = ProtectedRouteRateLimitKey::new(
            PrincipalIdentity::new(
                crate::IdentitySourceId::new("https://id.contextlab.test/tenant-a")
                    .expect("source"),
                PrincipalId::new("user:alex").expect("subject"),
            ),
            ProtectedRouteOperation::ContextCommitWrite,
        );
        let tenant_b = ProtectedRouteRateLimitKey::new(
            PrincipalIdentity::new(
                crate::IdentitySourceId::new("https://id.contextlab.test/tenant-b")
                    .expect("source"),
                PrincipalId::new("user:alex").expect("subject"),
            ),
            ProtectedRouteOperation::ContextCommitWrite,
        );

        assert_ne!(tenant_a, tenant_b);
    }

    #[test]
    fn policy_requires_bounded_positive_values() {
        let policy =
            ProtectedRouteRateLimitPolicy::new(2, 60, 100).expect("valid rate-limit policy");

        assert_eq!(policy.max_requests(), 2);
        assert_eq!(policy.window_seconds(), 60);
        assert_eq!(policy.max_tracked_principals(), 100);
        assert!(ProtectedRouteRateLimitPolicy::new(0, 60, 100).is_err());
        assert!(ProtectedRouteRateLimitPolicy::new(2, 0, 100).is_err());
        assert!(ProtectedRouteRateLimitPolicy::new(2, 60, 0).is_err());
        assert!(ProtectedRouteRateLimitPolicy::new(1_001, 60, 100).is_err());
        assert!(ProtectedRouteRateLimitPolicy::new(2, 3_601, 100).is_err());
        assert!(ProtectedRouteRateLimitPolicy::new(2, 60, 100_001).is_err());
    }

    #[test]
    fn keys_isolate_principals_for_one_protected_operation() {
        let alex = ProtectedRouteRateLimitKey::new(
            PrincipalIdentity::new(
                crate::IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new("user:alex").expect("principal"),
            ),
            ProtectedRouteOperation::ContextCommitWrite,
        );
        let blair = ProtectedRouteRateLimitKey::new(
            PrincipalIdentity::new(
                crate::IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new("user:blair").expect("principal"),
            ),
            ProtectedRouteOperation::ContextCommitWrite,
        );

        assert_ne!(alex, blair);
        assert_eq!(alex.principal_id().as_str(), "user:alex");
        assert_eq!(
            alex.operation(),
            ProtectedRouteOperation::ContextCommitWrite
        );
    }

    #[test]
    fn keys_isolate_distinct_context_read_operations() {
        let identity = PrincipalIdentity::new(
            crate::IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:alex").expect("principal"),
        );
        let lifecycle = ProtectedRouteRateLimitKey::new(
            identity.clone(),
            ProtectedRouteOperation::ContextLifecycleRead,
        );
        let graph_diff = ProtectedRouteRateLimitKey::new(
            identity,
            ProtectedRouteOperation::ContextCommitGraphDiffRead,
        );

        assert_ne!(lifecycle, graph_diff);
        assert_eq!(
            graph_diff.operation(),
            ProtectedRouteOperation::ContextCommitGraphDiffRead
        );

        let branch_heads = ProtectedRouteRateLimitKey::new(
            PrincipalIdentity::new(
                crate::IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
                PrincipalId::new("user:alex").expect("principal"),
            ),
            ProtectedRouteOperation::ContextBranchHeadRead,
        );
        assert_ne!(lifecycle, branch_heads);
        assert_ne!(graph_diff, branch_heads);
        assert_eq!(
            branch_heads.operation(),
            ProtectedRouteOperation::ContextBranchHeadRead
        );
    }

    #[tokio::test]
    async fn allows_exact_maximum_isolates_principals_and_resets_at_boundary() {
        let policy = ProtectedRouteRateLimitPolicy::new(2, 60, 100).expect("policy");
        let clock = Arc::new(FakeMonotonicClock::default());
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(policy, clock.clone());
        let alex = key("user:alex");

        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Rejected {
                retry_after_seconds: 60,
            })
        );
        assert_eq!(
            limiter.check(key("user:blair")).await,
            Ok(RateLimitDecision::Allowed)
        );

        clock.advance(Duration::from_secs(60));

        assert_eq!(limiter.check(alex).await, Ok(RateLimitDecision::Allowed));
    }

    #[tokio::test]
    async fn retry_after_rounds_up_and_never_falls_below_one_second() {
        let policy = ProtectedRouteRateLimitPolicy::new(1, 60, 100).expect("policy");
        let clock = Arc::new(FakeMonotonicClock::default());
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(policy, clock.clone());
        let alex = key("user:alex");

        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        clock.advance(Duration::from_millis(500));
        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Rejected {
                retry_after_seconds: 60,
            })
        );

        clock.advance(Duration::from_millis(59_499));
        assert_eq!(
            limiter.check(alex).await,
            Ok(RateLimitDecision::Rejected {
                retry_after_seconds: 1,
            })
        );
    }

    #[tokio::test]
    async fn clock_regression_fails_closed_for_existing_and_new_keys_without_state_changes() {
        let policy = ProtectedRouteRateLimitPolicy::new(2, 60, 10).expect("policy");
        let clock = Arc::new(FakeMonotonicClock::default());
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(policy, clock.clone());
        let alex = key("user:alex");
        let blair = key("user:blair");

        clock.set(Duration::from_secs(10));
        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        let state_before_regression = {
            let state = limiter.state.lock().await;
            (
                state.requests_by_key.len(),
                state.expiry_index.values().map(HashSet::len).sum::<usize>(),
            )
        };

        clock.set(Duration::from_secs(9));
        assert_eq!(limiter.check(alex).await, Err(RateLimitError::Unavailable));
        assert_eq!(limiter.check(blair).await, Err(RateLimitError::Unavailable));

        let state_after_regression = {
            let state = limiter.state.lock().await;
            (
                state.requests_by_key.len(),
                state.expiry_index.values().map(HashSet::len).sum::<usize>(),
            )
        };
        assert_eq!(state_after_regression, state_before_regression);
    }

    #[tokio::test]
    async fn full_capacity_fails_closed_without_evicting_active_keys_then_reclaims_expired_state() {
        let policy = ProtectedRouteRateLimitPolicy::new(2, 60, 1).expect("policy");
        let clock = Arc::new(FakeMonotonicClock::default());
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(policy, clock.clone());
        let alex = key("user:alex");
        let blair = key("user:blair");

        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        assert_eq!(
            limiter.check(blair.clone()).await,
            Err(RateLimitError::Unavailable)
        );
        assert_eq!(limiter.check(alex).await, Ok(RateLimitDecision::Allowed));

        clock.advance(Duration::from_secs(60));

        assert_eq!(limiter.check(blair).await, Ok(RateLimitDecision::Allowed));
    }

    #[tokio::test]
    async fn expiry_pruning_preserves_high_cardinality_active_keys() {
        let policy = ProtectedRouteRateLimitPolicy::new(2, 60, 600).expect("policy");
        let clock = Arc::new(FakeMonotonicClock::default());
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(policy, clock.clone());
        let expired = key("user:expired");

        assert_eq!(
            limiter.check(expired.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        clock.advance(Duration::from_secs(1));

        let active_keys = (0..512)
            .map(|index| key(&format!("user:active-{index}")))
            .collect::<Vec<_>>();
        for active_key in &active_keys {
            assert_eq!(
                limiter.check(active_key.clone()).await,
                Ok(RateLimitDecision::Allowed)
            );
        }

        clock.advance(Duration::from_secs(59));
        let newcomer = key("user:newcomer");
        assert_eq!(
            limiter.check(newcomer.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );

        let state = limiter.state.lock().await;
        assert_eq!(state.requests_by_key.len(), 513);
        assert!(!state.requests_by_key.contains_key(&expired));
        assert!(state.requests_by_key.contains_key(&newcomer));
        assert!(
            active_keys
                .iter()
                .all(|active_key| state.requests_by_key.contains_key(active_key))
        );
        assert!(
            state
                .expiry_index
                .first_key_value()
                .is_some_and(|(expiry, _)| *expiry > Duration::from_secs(60))
        );
    }

    #[tokio::test]
    async fn expiry_index_keeps_one_current_entry_per_active_key() {
        let policy = ProtectedRouteRateLimitPolicy::new(3, 60, 10).expect("policy");
        let clock = Arc::new(FakeMonotonicClock::default());
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(policy, clock.clone());
        let alex = key("user:alex");
        let blair = key("user:blair");

        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        clock.advance(Duration::from_secs(10));
        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        assert_eq!(
            limiter.check(blair.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        clock.advance(Duration::from_secs(10));
        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );

        let state = limiter.state.lock().await;
        let indexed_key_count = state.expiry_index.values().map(HashSet::len).sum::<usize>();
        let alex_index_entries = state
            .expiry_index
            .values()
            .filter(|keys| keys.contains(&alex))
            .count();
        let blair_index_entries = state
            .expiry_index
            .values()
            .filter(|keys| keys.contains(&blair))
            .count();

        assert_eq!(state.requests_by_key.len(), 2);
        assert_eq!(indexed_key_count, 2);
        assert_eq!(alex_index_entries, 1);
        assert_eq!(blair_index_entries, 1);
        assert!(!state.expiry_index.contains_key(&Duration::from_secs(60)));
        assert!(state.expiry_index[&Duration::from_secs(70)].contains(&blair));
        assert!(state.expiry_index[&Duration::from_secs(80)].contains(&alex));
    }

    #[tokio::test]
    async fn capacity_reclaim_uses_due_expiry_without_evicting_later_keys() {
        let policy = ProtectedRouteRateLimitPolicy::new(2, 60, 2).expect("policy");
        let clock = Arc::new(FakeMonotonicClock::default());
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(policy, clock.clone());
        let alex = key("user:alex");
        let blair = key("user:blair");
        let casey = key("user:casey");

        assert_eq!(
            limiter.check(alex.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        clock.advance(Duration::from_secs(10));
        assert_eq!(
            limiter.check(blair.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );
        clock.advance(Duration::from_secs(50));
        assert_eq!(
            limiter.check(casey.clone()).await,
            Ok(RateLimitDecision::Allowed)
        );

        let state = limiter.state.lock().await;
        assert_eq!(state.requests_by_key.len(), 2);
        assert!(!state.requests_by_key.contains_key(&alex));
        assert!(state.requests_by_key.contains_key(&blair));
        assert!(state.requests_by_key.contains_key(&casey));
        assert_eq!(
            state.expiry_index.values().map(HashSet::len).sum::<usize>(),
            2
        );
    }

    #[tokio::test]
    async fn request_queues_active_keys_and_expiry_entries_remain_bounded() {
        let policy = ProtectedRouteRateLimitPolicy::new(2, 60, 3).expect("policy");
        let limiter = InMemoryProtectedRouteRateLimiter::with_clock(
            policy,
            Arc::new(FakeMonotonicClock::default()),
        );
        let tracked_keys = [key("user:alex"), key("user:blair"), key("user:casey")];

        for tracked_key in &tracked_keys {
            assert_eq!(
                limiter.check(tracked_key.clone()).await,
                Ok(RateLimitDecision::Allowed)
            );
            assert_eq!(
                limiter.check(tracked_key.clone()).await,
                Ok(RateLimitDecision::Allowed)
            );
            assert!(matches!(
                limiter.check(tracked_key.clone()).await,
                Ok(RateLimitDecision::Rejected { .. })
            ));
        }
        assert_eq!(
            limiter.check(key("user:denied-by-capacity")).await,
            Err(RateLimitError::Unavailable)
        );

        let state = limiter.state.lock().await;
        assert_eq!(state.requests_by_key.len(), 3);
        assert!(
            state
                .requests_by_key
                .values()
                .all(|requests| requests.len() == 2)
        );
        assert_eq!(
            state.expiry_index.values().map(HashSet::len).sum::<usize>(),
            3
        );
        assert!(tracked_keys.iter().all(|tracked_key| {
            state
                .expiry_index
                .values()
                .filter(|keys| keys.contains(tracked_key))
                .count()
                == 1
        }));
    }

    #[tokio::test]
    async fn concurrent_checks_for_one_key_apply_the_limit_atomically() {
        let policy = ProtectedRouteRateLimitPolicy::new(1, 60, 100).expect("policy");
        let limiter = Arc::new(InMemoryProtectedRouteRateLimiter::with_clock(
            policy,
            Arc::new(FakeMonotonicClock::default()),
        ));
        let alex = key("user:alex");
        let mut checks = Vec::new();

        for _ in 0..16 {
            let limiter = limiter.clone();
            let alex = alex.clone();
            checks.push(tokio::spawn(async move { limiter.check(alex).await }));
        }

        let mut allowed = 0;
        let mut rejected = 0;
        for check in checks {
            match check.await.expect("check task") {
                Ok(RateLimitDecision::Allowed) => allowed += 1,
                Ok(RateLimitDecision::Rejected { .. }) => rejected += 1,
                Err(error) => panic!("unexpected limiter error: {error}"),
            }
        }

        assert_eq!(allowed, 1);
        assert_eq!(rejected, 15);
    }
}
