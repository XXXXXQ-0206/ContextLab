# Protected Runtime Authentication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a fail-closed opt-in PostgreSQL/HMAC protected router runtime profile without promoting the Context commit mutation to the public API contract.

**Architecture:** `server/api` parses route mode and protected HMAC configuration from explicit environment pairs. A private runtime-composition helper keeps one cloned PostgreSQL repository for every storage port, authorizer, guarded writer, and audit sink; public mode continues to build the existing public router. The existing `HmacJwtAuthenticator` validates issuer and audience.

**Tech Stack:** Rust stable, Axum, SQLx/PostgreSQL, `jsonwebtoken`, Tokio, existing ContextLab auth/storage crates.

---

## Task 1: Define Safe Runtime Configuration

**Files:**
- Modify: `server/api/src/lib.rs`

- [ ] **Step 1: Write failing configuration tests**

Add tests for `public` default, unsupported `CONTEXTLAB_API_ROUTE_MODE`, protected memory mode, and missing `CONTEXTLAB_AUTH_HS256_SECRET`, `CONTEXTLAB_AUTH_ISSUER`, or `CONTEXTLAB_AUTH_AUDIENCE`.

```rust
assert!(matches!(
    try_build_router_from_env([("CONTEXTLAB_API_ROUTE_MODE", "protected")]),
    Err(AppStateConfigError::ProtectedRoutesRequirePostgres)
));
```

- [ ] **Step 2: Run focused tests**

Run: `cargo test -p contextlab-api protected_runtime --lib`

Expected: compile failure because the runtime builder and configuration errors do not yet exist.

- [ ] **Step 3: Add private route-mode and HMAC configuration types**

```rust
enum ApiRouteMode { Public, Protected }

struct ProtectedRuntimeAuthConfig {
    secret: String,
    issuer: String,
    audience: String,
}
```

Parse only `public` and `protected`; require PostgreSQL and trim/reject empty authentication values. Add safe `AppStateConfigError` variants that name only missing variable names or unsupported modes.

- [ ] **Step 4: Run focused configuration tests**

Run: `cargo test -p contextlab-api protected_runtime --lib`

Expected: tests pass and no error contains the supplied secret.

## Task 2: Compose Protected Dependencies from One PostgreSQL Repository

**Files:**
- Modify: `server/api/src/lib.rs`

- [ ] **Step 1: Write a failing protected-router construction test**

Use explicit test environment pairs for PostgreSQL, the private protected route mode, HMAC secret, issuer, and audience. Build the router and issue a Context commit POST without an authorization header:

```rust
let response = try_build_router_from_env(protected_postgres_environment())?
    .oneshot(Request::builder()
        .method("POST")
        .uri("/api/v1/contexts/11111111-1111-4111-8111-111111111111/commits")
        .body(Body::from("{}"))?)
    .await?;
assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
```

- [ ] **Step 2: Run the construction test**

Run: `cargo test -p contextlab-api protected_runtime_installs_authentication_middleware --lib`

Expected: failure because the generic runtime builder still returns only the public router.

- [ ] **Step 3: Add private state composition and router builder**

Create a shared private environment-pair helper that returns `AppState` and an optional `PostgresContextGraphRepository`. In protected mode, use the repository clone for `ContextAuthorizer`, `GuardedContextCommitWriter`, and the existing PostgreSQL audit sink; construct `HmacJwtAuthenticator::new(&secret, Some(&issuer), Some(&audience))`; call `build_protected_router_with_state`.

- [ ] **Step 4: Run protected and public route tests**

Run: `cargo test -p contextlab-api protected_router --lib`

Run: `cargo test -p contextlab-api public_router_keeps_commit_mutation_out_of_the_public_catalog --lib`

Expected: protected mode returns `401` before database access, while public mode keeps the route absent.

## Task 3: Document the Private Runtime Profile

**Files:**
- Modify: `.env.example`
- Modify: `README.md`
- Modify: `docs/adr/0002-guarded-context-commit-writes.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [ ] **Step 1: Add bilingual safe configuration guidance**

Document route modes, required protected profile variables by name only, the PostgreSQL requirement, and the non-goals: no public write route, OAuth2/OIDC/JWKS, key rotation, rate limiting, or release promotion.

- [ ] **Step 2: Run contract and release gates**

Run: `cargo fmt --all -- --check`

Run: `cargo test --workspace`

Run: `pnpm check:web`

Run: `python apps/web/verify-context-workspace.py`

Expected: all commands exit `0`; `PUBLIC_POST_ROUTES` and checked-in OpenAPI remain unchanged.

## Completion Notes / 收束说明

Do not inspect or print `.env` values, JWTs, database URLs, or provider credentials. Do not initialize Git metadata. This profile is an authenticated private runtime composition, not a claim that OAuth2/OIDC, production key rotation, rate limiting, or public mutation release conditions are complete.

**Execution:** Completed on 2026-07-11 with safe configuration errors, PostgreSQL-only protected composition, issuer/audience enforcement, protected middleware evidence, bilingual documentation, and fresh Rust/Web verification. The next security increment is JWKS-backed OIDC validation with key rotation.
