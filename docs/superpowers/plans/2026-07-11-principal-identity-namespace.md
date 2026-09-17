# Principal Identity Namespace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace bare-subject protected-write security keys with one validated issuer-scoped `PrincipalIdentity` across authentication, rate limiting, direct membership, authorization audit, idempotency, and guarded-write transactional checks.

**Architecture:** `contextlab-auth` owns opaque identity-source and subject values plus the composite identity. HMAC and OIDC construct the identity only after verifier configuration and token validation. `server/api` transports the typed identity, while `contextlab-storage` persists and queries both dimensions through an additive PostgreSQL migration. No public REST, OpenAPI, SDK, GraphDiff, or Web contract changes.

**Tech Stack:** Rust stable, async-trait, Serde, jsonwebtoken, Axum, SQLx/PostgreSQL, Cargo, pnpm.

---

### Task 1: Define the Issuer-Scoped Identity Domain

**Files:**
- Modify: `crates/auth/src/authorization.rs`
- Modify: `crates/auth/src/lib.rs`

- [x] **Step 1: Write failing identity-value tests**

Add tests proving that identity sources and subjects are opaque, case-sensitive, bounded values:

```rust
let source = IdentitySourceId::new("https://id.example.com/tenant-a").expect("source");
let subject = PrincipalId::new("User:Alex").expect("subject");
let identity = PrincipalIdentity::new(source.clone(), subject.clone());

assert_eq!(identity.source(), &source);
assert_eq!(identity.subject(), &subject);
assert_ne!(subject, PrincipalId::new("user:alex").expect("subject"));
assert!(IdentitySourceId::new("").is_err());
assert!(IdentitySourceId::new(" source ").is_err());
assert!(PrincipalId::new(" subject ").is_err());
assert!(PrincipalId::new("subject\n").is_err());
assert!(PrincipalId::new("x".repeat(513)).is_err());
```

Add a constructor test requiring `AuthenticatedPrincipal::new(identity)` and preserving the exact composite identity.

- [x] **Step 2: Verify the domain tests fail**

Run: `cargo test -p contextlab-auth identity --lib`

Expected: compile failure because `IdentitySourceId` and `PrincipalIdentity` do not exist and `AuthenticatedPrincipal` still accepts a bare `PrincipalId`.

- [x] **Step 3: Implement bounded opaque identity values**

Add these contracts in `authorization.rs`:

```rust
pub struct IdentitySourceId(String);
pub struct PrincipalId(String);

pub struct PrincipalIdentity {
    source: IdentitySourceId,
    subject: PrincipalId,
}

pub struct AuthenticatedPrincipal {
    identity: PrincipalIdentity,
}
```

Validation rejects empty values, surrounding whitespace, Unicode control characters, and values over 2,048 bytes for `IdentitySourceId` or 512 bytes for `PrincipalId`. Do not lowercase, trim, hash, or parse either value. Provide `as_str`, `Display`, `source`, `subject`, and `identity` accessors. Export all public values from `crates/auth/src/lib.rs`.

- [x] **Step 4: Update auth-domain fixtures and verify green**

Replace all auth-crate test principals with an explicit test identity source. Run:

`cargo test -p contextlab-auth --lib`

Expected: all auth tests pass without warnings.

### Task 2: Construct Composite Identities in Authenticators

**Files:**
- Modify: `crates/auth/src/authentication.rs`
- Modify: `crates/auth/src/oidc.rs`
- Modify: `crates/auth/src/lib.rs`
- Modify: `server/api/src/lib.rs`

- [x] **Step 1: Write failing HMAC and OIDC tests**

For HMAC, require a non-empty validated issuer and audience at construction, then assert the authenticated identity source is exactly the configured issuer and the subject is exactly the signed `sub`.

For OIDC, sign two valid tokens with the same `sub` under two valid verifier configurations and assert their `PrincipalIdentity` values differ because their sources differ. Add invalid tests for surrounding whitespace/control characters and oversized subjects.

- [x] **Step 2: Verify authenticator tests fail**

Run: `cargo test -p contextlab-auth authentication --lib`

Run: `cargo test -p contextlab-auth oidc --lib`

Expected: failures because authenticators still return bare-subject principals and HMAC issuer/audience remain optional.

- [x] **Step 3: Make HMAC issuer and audience explicit**

Change the HMAC constructor to require `secret`, `issuer`, and `audience` strings. Validate non-empty bounded issuer/audience values, preserve existing HS256 signature/issuer/audience checks, and construct:

```rust
AuthenticatedPrincipal::new(PrincipalIdentity::new(
    IdentitySourceId::new(claims.iss.expect("issuer was validated"))?,
    PrincipalId::new(claims.sub)?,
))
```

Map validation failures to existing safe `AuthenticationError` variants without including claim values.

- [x] **Step 4: Construct OIDC identities from the verified issuer**

After RS256 signature, issuer, audience, and expiry validation, construct the source from the configured exact issuer and the subject from `sub`. Do not trust an unverified claim value or expose issuer/subject in errors.

- [x] **Step 5: Update runtime and test composition**

Update every `HmacJwtAuthenticator::new` call in `server/api/src/lib.rs` and crate tests to supply explicit issuer/audience values. Protected runtime continues using `CONTEXTLAB_AUTH_ISSUER` and `CONTEXTLAB_AUTH_AUDIENCE`; no new public environment variable is introduced.

- [x] **Step 6: Verify authentication green**

Run: `cargo test -p contextlab-auth --lib`

Run: `cargo test -p contextlab-api protected_ --lib`

Expected: exact issuer scoping passes, existing authentication status/error behavior remains stable, and protected routes still require rate limiting.

### Task 3: Use Composite Identity for Rate Limiting and Audit

**Files:**
- Modify: `crates/auth/src/rate_limit.rs`
- Modify: `crates/auth/src/authorization.rs`
- Modify: `crates/auth/src/lib.rs`
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`

- [x] **Step 1: Write failing composite-key tests**

Add a rate-limit test where two principals share subject `user:alex` but use different identity sources. Each identity must receive its own allowance. Add an audit-event test that preserves both source and subject.

- [x] **Step 2: Verify composite-key tests fail**

Run: `cargo test -p contextlab-auth identity --lib`

Expected: failure because rate-limit keys and audit events currently store only `PrincipalId`.

- [x] **Step 3: Replace bare-subject fields**

Change `ProtectedRouteRateLimitKey` to store `PrincipalIdentity`. Change `AuthorizationAuditEvent` to store `PrincipalIdentity`. Preserve operation, context, permission, and decision fields. Update accessors and API middleware construction to pass `principal.identity().clone()`.

- [x] **Step 4: Verify middleware and side-effect boundaries**

Update API test doubles to inspect the composite key. Assert invalid credentials consume no quota, two sources with the same subject do not share quota, and rate-limit rejection still produces no authorization audit or commit write.

Run: `cargo test -p contextlab-api protected_rate_limit --lib`

Expected: all protected-rate-limit tests pass with unchanged `429/503` responses.

### Task 4: Add the PostgreSQL Identity Namespace Migration

**Files:**
- Create: `crates/storage/migrations/0006_principal_identity_namespace.sql`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Write failing migration asset tests**

Assert `CONTEXT_PLATFORM_MIGRATION` includes migration 0006 and contains:

```text
ALTER TABLE workspace_memberships ADD COLUMN identity_source
PRIMARY KEY (workspace_id, identity_source, principal_id)
ALTER TABLE context_commit_idempotency ADD COLUMN identity_source
PRIMARY KEY (identity_source, principal_id, context_id, idempotency_key)
ALTER TABLE context_authorization_audit_events ADD COLUMN identity_source
```

Also assert non-empty/no-surrounding-whitespace checks and source+subject lookup indexes exist.

- [x] **Step 2: Verify migration tests fail**

Run: `cargo test -p contextlab-storage migration --lib`

Expected: failure because migration 0006 is absent.

- [x] **Step 3: Implement additive upgrade SQL**

For each existing table, add `identity_source TEXT`, backfill existing rows with explicit `legacy`, set `NOT NULL`, add bounded/non-empty checks, replace affected primary keys, and add source+subject indexes. Do not drop data or rewrite existing subject values. Include migration 0006 after migration 0005.

- [x] **Step 4: Verify structural migration tests**

Run: `cargo test -p contextlab-storage migration --lib`

Expected: all migration asset tests pass.

### Task 5: Propagate Composite Keys Through PostgreSQL

**Files:**
- Modify: `crates/storage/src/postgres.rs`
- Modify: `crates/storage/src/guarded_commit_write.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Write failing resolver and writer tests**

Add structural query tests proving direct membership resolution binds context id, identity source, and subject. Add guarded-writer query tests proving transactional membership recheck and idempotency reads/inserts use both source and subject.

Add an ignored disposable PostgreSQL upgrade test that applies migrations, inserts two same-subject memberships from different sources, and proves only the matching source resolves. Add a guarded-write test proving same-subject idempotency keys do not collide across sources.

- [x] **Step 2: Verify storage tests fail**

Run: `cargo test -p contextlab-storage identity --lib`

Expected: failures because current SQL binds only `principal_id`.

- [x] **Step 3: Update membership resolution**

Update `CONTEXT_MEMBERSHIP_ROLE_SQL` and `CONTEXT_WRITE_MEMBERSHIP_FOR_UPDATE_SQL` to require exact `identity_source` and `principal_id`. Bind `principal.identity().source().as_str()` and `principal.identity().subject().as_str()` in the same order in preflight and transactional paths.

- [x] **Step 4: Update idempotency storage**

Update every context-commit idempotency query, advisory-lock key, insert, replay, and conflict path to include identity source. Use an unambiguous length-prefixed or tuple-derived advisory-lock key rather than raw colon concatenation. The in-memory guarded writer key must also use `PrincipalIdentity`.

- [x] **Step 5: Update authorization audit storage**

Bind identity source and subject separately into `context_authorization_audit_events`. Existing audit response codes and append-only behavior remain unchanged.

- [x] **Step 6: Verify storage green**

Run: `cargo test -p contextlab-storage --lib`

Expected: all default tests pass; disposable PostgreSQL tests remain explicit when no isolated test database is configured.

### Task 6: Document and Verify the Namespace Increment

**Files:**
- Modify: `.env.example`
- Modify: `README.md`
- Modify: `ARCHITECTURE.md`
- Modify: `docs/adr/0002-guarded-context-commit-writes.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/active-long-term-goal.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`

- [x] **Step 1: Update bilingual architecture and migration guidance**

Document exact issuer+subject identity scoping, the explicit `legacy` upgrade sentinel, operator migration responsibility, composite quota/audit/membership/idempotency behavior, and the fact that trusted group extraction/bindings remain the immediately following increment. Do not claim group-to-RBAC is complete.

- [x] **Step 2: Preserve public contract guards**

Keep explicit assertions that public OpenAPI has no Context commit POST and `ContextLabClient` has no commit-creation method. No OpenAPI or SDK operation is added.

- [ ] **Step 3: Run fresh release gates**

Run:

```text
cargo fmt --all -- --check
cargo test --workspace
pnpm check:web
python apps/web/verify-context-workspace.py
```

Expected: all non-disposable gates exit `0`; ignored PostgreSQL tests remain named with their database prerequisite; the Web verifier passes against the local server.

- [x] **Step 4: Record the next increment**

Set bounded OIDC group extraction and `workspace_external_group_role_bindings` as the next dependency-ready plan. Keep audit retention/access governance after group mapping. Do not mark the active long-term goal or production security completion criterion complete.
