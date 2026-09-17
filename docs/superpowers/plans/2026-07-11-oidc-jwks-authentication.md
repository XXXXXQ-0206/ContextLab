# OIDC JWKS Authentication Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a fail-closed RS256 OIDC JWT verifier with a replaceable JWKS source, bounded cache, and protected-runtime configuration while keeping Context commit mutation out of public REST/OpenAPI/SDK contracts.

**Architecture:** `contextlab-auth` changes `PrincipalAuthenticator` to an async port, preserving HMAC as zero-I/O and adding `OidcJwksAuthenticator<S>`. A `JwksSource` port supplies `jsonwebtoken::jwk::JwkSet`; a private HTTPS `reqwest` adapter is injected only by the API runtime. The verifier accepts exactly RS256 RSA signing JWKs, validates issuer/audience, uses a mutex-protected cache, and forces one refresh for an unknown `kid` before rejecting.

**Tech Stack:** Rust stable, async-trait, Tokio sync/time, `jsonwebtoken` 9.3.1 JWK support, `reqwest` with Rustls TLS, Axum, URL.

---

## Task 1: Make Authentication Explicitly Asynchronous

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/auth/src/authentication.rs`
- Modify: `server/api/src/lib.rs`

- [ ] **Step 1: Write the failing async-port test**

Change the existing HMAC port test to await authentication:

```rust
let principal = authenticator
    .authenticate_authorization_header(Some(&format!("Bearer {token}")))
    .await
    .expect("principal");
```

- [ ] **Step 2: Run focused auth tests**

Run: `cargo test -p contextlab-auth authentication --lib`

Expected: compile failure because `PrincipalAuthenticator` remains synchronous.

- [ ] **Step 3: Change only the port and existing implementations**

```rust
#[async_trait]
pub trait PrincipalAuthenticator: Send + Sync {
    async fn authenticate_authorization_header(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<AuthenticatedPrincipal, AuthenticationError>;
}
```

Mark the HMAC implementation async without introducing I/O. Await the port in `routes::authenticate_request` and update test-only authenticators.

- [ ] **Step 4: Verify async port consumers**

Run: `cargo test -p contextlab-auth`

Run: `cargo test -p contextlab-api protected_router --lib`

Expected: HMAC and existing protected-route behavior remain green.

## Task 2: Define and Test JWKS Selection and Cache Contracts

**Files:**
- Create: `crates/auth/src/oidc.rs`
- Modify: `crates/auth/src/lib.rs`
- Modify: `crates/auth/Cargo.toml`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write failing unit tests with an in-memory source**

Define a test `CountingJwksSource` and assert the following without HTTP:

```rust
assert_eq!(source.fetch_count(), 1); // initial matching kid
assert_eq!(source.fetch_count(), 1); // cache hit
assert_eq!(source.fetch_count(), 2); // unknown kid forces exactly one refresh
```

Add negative tests for missing `kid`, duplicate matching keys, non-RSA JWK, `use = enc`, JWK `alg != RS256`, expired cache after fetch failure, invalid issuer, invalid audience, and an HS256 token signed with an RSA-looking key id.

- [ ] **Step 2: Run focused OIDC tests**

Run: `cargo test -p contextlab-auth oidc --lib`

Expected: compile failure because `JwksSource` and `OidcJwksAuthenticator` do not exist.

- [ ] **Step 3: Add domain types and safe errors**

```rust
#[async_trait]
pub trait JwksSource: Send + Sync {
    async fn fetch(&self, jwks_url: &Url) -> Result<JwkSet, JwksSourceError>;
}

pub struct OidcJwksAuthenticator<S> {
    source: S,
    config: OidcJwksConfig,
    cache: tokio::sync::Mutex<Option<CachedJwks>>,
}
```

`OidcJwksConfig` validates HTTPS URL, non-empty issuer/audience, and cache TTL within `1..=3600` seconds. Use `decode_header`, require `Algorithm::RS256`, select exactly one eligible JWK, construct `DecodingKey::from_jwk`, then decode with `Validation::new(Algorithm::RS256)` plus issuer/audience. Map all source, header, JWK, signature, and claim failures to existing safe authentication error forms or new value-free OIDC errors.

- [ ] **Step 4: Implement cache refresh semantics**

Hold the async mutex across a refresh decision so concurrent unknown `kid` requests share one fetch. Reuse an unexpired cache for known keys. On an unknown key, force one fetch and retry selection. On expiry, fetch before selection; if that fetch fails, reject without stale-key verification.

- [ ] **Step 5: Verify auth contract**

Run: `cargo test -p contextlab-auth`

Expected: all auth tests pass, and the in-memory source proves cache and failure semantics.

## Task 3: Add the HTTPS JWKS Infrastructure Adapter

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/auth/Cargo.toml`
- Modify: `crates/auth/src/oidc.rs`

- [ ] **Step 1: Add a failing adapter parse test**

Use a local test double response body, not a network endpoint, to prove malformed JSON maps to `JwksSourceError::Unavailable` and accepted JSON deserializes to `JwkSet`.

- [ ] **Step 2: Add minimal dependencies**

Add workspace `reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }` and enable Tokio `sync` and `time`. Add `reqwest.workspace = true` and `tokio.workspace = true` to `crates/auth/Cargo.toml`.

- [ ] **Step 3: Implement the private adapter**

```rust
pub struct HttpsJwksSource { client: reqwest::Client }

#[async_trait]
impl JwksSource for HttpsJwksSource {
    async fn fetch(&self, jwks_url: &Url) -> Result<JwkSet, JwksSourceError> {
        let response = self.client.get(jwks_url.clone()).send().await
            .map_err(|_| JwksSourceError::Unavailable)?;
        if !response.status().is_success() {
            return Err(JwksSourceError::Unavailable);
        }
        response.json().await.map_err(|_| JwksSourceError::Unavailable)
    }
}
```

Construct the client with an explicit timeout, HTTPS-only policy, redirect limit, and a bounded body strategy. Do not log URL query values or response bodies.

- [ ] **Step 4: Verify adapter tests**

Run: `cargo test -p contextlab-auth oidc --lib`

Expected: all OIDC source and verifier tests pass without external network access.

## Task 4: Compose OIDC Only in Private Protected Runtime

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `.env.example`
- Modify: `README.md`

- [ ] **Step 1: Write failing runtime configuration tests**

Assert that protected `CONTEXTLAB_AUTH_MODE=oidc` rejects missing issuer, audience, JWKS URL, invalid non-HTTPS JWKS URL, invalid cache TTL, and memory storage. Assert `hmac` retains the existing configuration behavior.

- [ ] **Step 2: Run focused API tests**

Run: `cargo test -p contextlab-api oidc_runtime --lib`

Expected: compile failure because auth mode parsing and OIDC runtime configuration do not exist.

- [ ] **Step 3: Add runtime selection without public promotion**

Select `HmacJwtAuthenticator` or `OidcJwksAuthenticator<HttpsJwksSource>` only inside `ApiRouteMode::Protected`. Keep `PUBLIC_POST_ROUTES`, OpenAPI, SDK, and `build_router()` unchanged. Map all configuration failures to safe `AppStateConfigError` variants.

- [ ] **Step 4: Document bilingual configuration**

List only variable names: `CONTEXTLAB_AUTH_MODE`, `CONTEXTLAB_OIDC_ISSUER`, `CONTEXTLAB_OIDC_AUDIENCE`, `CONTEXTLAB_OIDC_JWKS_URL`, and `CONTEXTLAB_OIDC_JWKS_CACHE_TTL_SECONDS`. State that no production identity provider or secret is used by tests.

- [ ] **Step 5: Verify private/public router boundaries**

Run: `cargo test -p contextlab-api protected_runtime --lib`

Run: `cargo test -p contextlab-api public_post_routes_match_openapi_contract --lib`

Expected: private OIDC configuration is fail-closed; public OpenAPI remains mutation-free.

## Task 5: Run Release Gates

- [ ] **Step 1: Format and test Rust workspace**

Run: `cargo fmt --all -- --check`

Run: `cargo test --workspace`

- [ ] **Step 2: Verify Web workspace is unchanged**

Run: `pnpm check:web`

Run: `python apps/web/verify-context-workspace.py`

Expected: all commands exit `0`.

## Completion Notes / 收束说明

Do not read or print `.env`, JWTs, JWKS response bodies, database URLs, or provider credentials. Do not accept HTTP JWKS URLs, HS/PS/ES/EdDSA JWTs, missing/duplicate key identifiers, or stale keys after cache expiry. This slice does not promote public mutations or complete OIDC discovery, group RBAC mapping, rate limiting, audit retention, CI release gates, or the long-term project goal.
