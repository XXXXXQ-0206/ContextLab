# Protected Route Rate Limiting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add authentication-aware, bounded, fail-closed rate limiting to the private guarded Context commit route without changing public REST, OpenAPI, SDK, GraphDiff, or Web contracts.

**Architecture:** `contextlab-auth` owns a replaceable `ProtectedRouteRateLimiter` port and a bounded in-memory sliding-window adapter keyed by validated `PrincipalId` plus operation. `server/api` composes the limiter only for protected runtime state and checks it after JWT authentication but before authorization audit or guarded write execution. Public state has no limiter and never enters this middleware.

**Tech Stack:** Rust stable, async-trait, Tokio mutex/time, Axum middleware, Tower test utilities, Serde JSON, Cargo, pnpm.

---

### Task 1: Define the Rate-Limit Domain Contract

**Files:**
- Create: `crates/auth/src/rate_limit.rs`
- Modify: `crates/auth/src/lib.rs`

- [x] **Step 1: Write the failing contract tests**

Add tests that construct a validated policy and assert invalid ranges are rejected:

```rust
let policy = ProtectedRouteRateLimitPolicy::new(2, 60, 100).expect("policy");
assert_eq!(policy.max_requests(), 2);
assert!(ProtectedRouteRateLimitPolicy::new(0, 60, 100).is_err());
assert!(ProtectedRouteRateLimitPolicy::new(2, 0, 100).is_err());
assert!(ProtectedRouteRateLimitPolicy::new(2, 60, 0).is_err());
```

Define a key from `PrincipalId` and `ProtectedRouteOperation::ContextCommitWrite`; assert keys from different principals are unequal and contain no token or request body.

- [x] **Step 2: Verify the tests fail**

Run: `cargo test -p contextlab-auth rate_limit --lib`

Expected: compile failure because the rate-limit types do not exist.

- [x] **Step 3: Implement the minimal domain types**

Create these public contracts:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectedRouteOperation { ContextCommitWrite }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtectedRouteRateLimitKey {
    principal_id: PrincipalId,
    operation: ProtectedRouteOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitDecision {
    Allowed,
    Rejected { retry_after_seconds: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RateLimitError { Unavailable }

#[async_trait]
pub trait ProtectedRouteRateLimiter: Send + Sync {
    async fn check(
        &self,
        key: ProtectedRouteRateLimitKey,
    ) -> Result<RateLimitDecision, RateLimitError>;
}
```

Validate policy ranges: requests `1..=1000`, window seconds `1..=3600`, tracked principals `1..=100000`.

- [x] **Step 4: Verify the domain tests pass**

Run: `cargo test -p contextlab-auth rate_limit --lib`

Expected: policy and key contract tests pass without warnings.

### Task 2: Implement the Bounded Sliding-Window Adapter

**Files:**
- Modify: `crates/auth/src/rate_limit.rs`

- [x] **Step 1: Write failing behavior tests with a fake clock**

Use an injected monotonic clock and test:

```rust
assert_eq!(limiter.check(alex.clone()).await?, RateLimitDecision::Allowed);
assert_eq!(limiter.check(alex.clone()).await?, RateLimitDecision::Allowed);
assert_eq!(
    limiter.check(alex.clone()).await?,
    RateLimitDecision::Rejected { retry_after_seconds: 60 },
);
assert_eq!(limiter.check(blair).await?, RateLimitDecision::Allowed);
clock.advance(Duration::from_secs(60));
assert_eq!(limiter.check(alex).await?, RateLimitDecision::Allowed);
```

Add a separate test where `max_tracked_principals=1`: a second active principal returns `RateLimitError::Unavailable`; after the first window expires, stale state is pruned and the second principal is allowed.

- [x] **Step 2: Verify the behavior tests fail**

Run: `cargo test -p contextlab-auth rate_limit --lib`

Expected: compile failure because the in-memory adapter and clock port do not exist.

- [x] **Step 3: Implement bounded state and pruning**

Store `HashMap<ProtectedRouteRateLimitKey, VecDeque<Duration>>` behind `tokio::sync::Mutex`. On every check, remove timestamps at or before `now - window`, remove empty keys, reject a new active key when capacity remains full, and otherwise append `now`. Compute `retry_after_seconds` by ceiling the duration until the oldest accepted request exits the window. Map a poisoned/unavailable clock or state path to `RateLimitError::Unavailable`; never evict an active principal to admit another.

- [x] **Step 4: Verify adapter behavior**

Run: `cargo test -p contextlab-auth rate_limit --lib`

Expected: boundary, principal-isolation, expiry, and capacity tests pass.

### Task 3: Compose Limiting After Authentication

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `server/api/src/routes.rs`

- [x] **Step 1: Write failing protected-router tests**

Add explicit test limiters and assert:

```rust
// Missing/invalid bearer: 401 and limiter call count remains 0.
// First authenticated request reaches the existing handler.
// Rejected authenticated request: 429, JSON error `rate_limit_exceeded`, Retry-After header.
// Unavailable limiter: 503, JSON error `rate_limit_unavailable`.
// Rejected/unavailable requests create no authorization audit event and no commit.
```

Keep `public_router_keeps_commit_mutation_out_of_the_public_catalog` and `public_post_routes_match_openapi_contract` unchanged.

- [x] **Step 2: Verify router tests fail**

Run: `cargo test -p contextlab-api protected_rate_limit --lib`

Expected: compile failure because `AppState` and middleware have no limiter dependency.

- [x] **Step 3: Add explicit state and middleware composition**

Add `Option<Arc<dyn ProtectedRouteRateLimiter>>` to `AppState`. Extend protected dependency composition to require a limiter. In the existing protected middleware, authenticate first, construct a `ProtectedRouteRateLimitKey` for `ContextCommitWrite`, then:

```rust
match state.check_protected_rate_limit(key).await {
    Ok(RateLimitDecision::Allowed) => { /* insert principal and continue */ }
    Ok(RateLimitDecision::Rejected { retry_after_seconds }) => {
        return rate_limit_exceeded_response(retry_after_seconds);
    }
    Err(_) => return ApiError::RateLimitUnavailable.into_response(),
}
```

The `429` response includes integer `Retry-After`; `503` contains no internal state details. Limiting runs before handler authorization/audit/write logic.

- [x] **Step 4: Verify protected/public boundaries**

Run: `cargo test -p contextlab-api protected_rate_limit --lib`

Run: `cargo test -p contextlab-api public_post_routes_match_openapi_contract --lib`

Expected: authentication precedence, fail-closed rate limiting, no downstream effects, and unchanged public catalog all pass.

### Task 4: Add Protected Runtime Configuration

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `.env.example`

- [x] **Step 1: Write failing configuration tests**

For protected mode require:

```text
CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_REQUESTS
CONTEXTLAB_PROTECTED_RATE_LIMIT_WINDOW_SECONDS
CONTEXTLAB_PROTECTED_RATE_LIMIT_MAX_TRACKED_PRINCIPALS
```

Assert each missing value has a safe distinct configuration error, non-integers and out-of-range values fail startup, and public mode ignores these variables.

- [x] **Step 2: Verify configuration tests fail**

Run: `cargo test -p contextlab-api protected_rate_limit_configuration --lib`

Expected: compile failure because configuration types and errors do not exist.

- [x] **Step 3: Parse and compose the in-memory adapter**

Parse only inside `ApiRouteMode::Protected`. Build `ProtectedRouteRateLimitPolicy`, create `InMemoryProtectedRouteRateLimiter`, and inject it alongside the selected HMAC/OIDC authenticator. Never read raw tokens, provider secrets, or request bodies into limiter state.

- [x] **Step 4: Verify runtime configuration**

Run: `cargo test -p contextlab-api protected_runtime --lib`

Expected: HMAC and OIDC protected profiles require valid rate-limit configuration; public profile remains unchanged.

### Task 5: Document and Verify the Slice

**Files:**
- Modify: `README.md`
- Modify: `.env.example`
- Modify: `docs/adr/0002-guarded-context-commit-writes.md`
- Modify: `docs/roadmap/active-long-term-goal.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Update bilingual documentation**

Document the three variable names and ranges, per-principal/per-operation key, `429`/`503` behavior, bounded in-process scope, and the requirement for a shared atomic adapter before multi-replica/public deployment. State that no public OpenAPI/SDK operation is added.

- [x] **Step 2: Run release gates**

Run: `cargo fmt --all -- --check`

Run: `cargo test --workspace`

Run: `pnpm check:web`

Run with the Web dev server available: `python apps/web/verify-context-workspace.py`

Expected: all commands exit `0`; disposable PostgreSQL tests remain explicitly identified when their test database is unavailable.

- [x] **Step 3: Record the next long-term increment**

Update the bilingual roadmap to select either group-to-RBAC/membership management or authorization-audit retention/access governance based on dependency readiness. Do not mark the active long-term goal complete.
