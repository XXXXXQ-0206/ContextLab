# Guarded Context Commit Write Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first authenticated, idempotent, optimistic-concurrency-safe Context commit write path without exposing graph editing, merge, or arbitrary parent selection.

**Architecture:** Follow ADR 0002. Build reusable authorization and guarded-write ports below Axum; persist idempotency and branch-head state in the same storage transaction that creates a commit and immutable graph snapshot. The REST layer first exposes an explicitly configured protected router while the public route catalog and OpenAPI/SDK mutation surface remain closed until production authorization and disposable PostgreSQL evidence pass.

**Tech Stack:** Rust stable, Axum, SQLx/PostgreSQL, Serde, existing `contextlab-versioning`, `contextlab-graph`, `contextlab-storage`, REST/OpenAPI, TypeScript SDK.

---

## File Responsibilities

- `docs/adr/0002-guarded-context-commit-writes.md`: accepted decision and scope boundary.
- `crates/auth`: authenticated principal, workspace membership/role contracts, and `context:write` authorization port, implemented deny-first for production and explicitly permissive only in tests.
- `crates/storage`: guarded write command, idempotency and branch-head repository contracts, migrations, in-memory implementation, PostgreSQL transaction implementation, and conflict errors.
- `crates/versioning`: branch-head and expected-head validation types; it remains independent of graph snapshots and persistence.
- `server/api`: authenticated request extraction, route/catalog registration, DTO validation, error mapping, and route tests.
- `docs/api/openapi.json` and `packages/ts-sdk`: remain synchronized with the public read-only surface; a public write contract is deferred until the promotion gate passes.

## Task 1: Define Guarded-Write Domain Contracts

- [x] Add failing unit tests for an expected head that is absent, current, stale, and invalid for initial branch creation.
- [x] Add a small versioning model that selects a normal commit parent from an expected and actual branch head; graph snapshots, idempotency, and persistence remain outside this crate.
- [x] Add a principal/permission port that expresses `context:write` without coupling domain code to JWT or Axum.
- [x] Run focused crate tests until green.

## Task 2: Persist Idempotency and Branch Heads Atomically

- [x] Add a migration for `context_branches`, workspace membership/role records, and idempotency records with ownership, request digest, result reference, timestamps, revision, and necessary unique indexes.
- [x] Add failing in-memory tests for identical replay, key/digest mismatch, stale head, first branch commit, and atomic rollback on snapshot failure.
- [x] Extend storage errors with structured conflict variants that preserve safe current-head metadata.
- [x] Implement a single guarded writer operation in memory and PostgreSQL; lock idempotency and branch-head state in a deterministic order.
- [x] Add ignored disposable PostgreSQL tests for concurrent stale-head writers, concurrent idempotent replay, and same-Context foreign-key enforcement using a two-connection pool; fresh execution remains a release-gate task.

## Task 3: Authenticate and Expose the REST Contract

The repository now has bearer extraction, a replaceable `PrincipalAuthenticator` port, HMAC JWT verification, explicit `Forbidden`/`Unavailable` authorization semantics, protected-router middleware, a PostgreSQL membership authorizer, a PostgreSQL-backed fail-closed `AuthorizationAuditSink` adapter, and an explicitly opt-in protected route. Fresh local disposable PostgreSQL runs have exercised guarded concurrency and audit persistence. Production runtime auth/OAuth2-OIDC JWKS configuration, broader RBAC, rate limiting, audit retention/access policy, disposable-database CI, and the public mutation contract remain pending.

仓库现在已有 Bearer 提取、可替换的 `PrincipalAuthenticator` port、HMAC JWT 校验、明确区分 `Forbidden`/`Unavailable` 的 authorization semantics、protected-router middleware、PostgreSQL membership authorizer、PostgreSQL-backed 且 fail-closed 的 `AuthorizationAuditSink` adapter，以及显式 opt-in protected route。多次新鲜的本地 disposable PostgreSQL run 已覆盖 guarded concurrency 与 audit persistence。production runtime auth/OAuth2-OIDC JWKS configuration、更广泛的 RBAC、rate limiting、audit retention/access policy、disposable-database CI 与 public mutation contract 仍待完成。

- [x] Add an explicitly opt-in protected router with bearer/JWT authentication, authorization, creation, replay, and stale-head tests; keep `PUBLIC_POST_ROUTES` unchanged.
- [ ] Promote the protected route only after production authorization configuration and disposable PostgreSQL concurrency evidence pass; then update OpenAPI and the TypeScript SDK.

- [x] Add Axum tests for missing identity, denied identity, invalid request, successful creation, replay, stale head, and no partial persistence.
- [x] Register `POST /api/v1/contexts/{context_id}/commits` only in a protected route catalog behind authenticated state configuration; preview mode returns no public write route.
- [x] Add request/response DTOs that reject client-provided commit ids, parents, timestamps, and author identities.
- [ ] Update the public POST catalog and OpenAPI drift tests only when the production promotion gate passes.
- [x] Run focused API tests until green.

## Task 4: Publish the SDK and Documentation Boundary

- [ ] Add a TypeScript SDK method that requires the idempotency key and expected-head field.
- [ ] Add SDK URL/body/error tests and update OpenAPI contract tests.
- [x] Update bilingual REST, storage, SDK, and roadmap documentation with explicit non-goals: no merge, rollback, replay execution, or Web editor.
- [ ] Keep the public OpenAPI and TypeScript SDK mutation surface unchanged until production authorization and disposable PostgreSQL evidence pass.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm --filter @contextlab/ts-sdk test`, `pnpm check:web`, and the disposable PostgreSQL integration suite.
