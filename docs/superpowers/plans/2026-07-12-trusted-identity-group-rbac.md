# Trusted Identity and Group RBAC Implementation Plan / 可信身份与组 RBAC 实施计划

> **For agentic workers / 面向智能体执行者：** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` for independent review and verification. Steps use checkbox syntax for traceable delivery.

**Goal / 目标：** Map verified bounded OIDC group identifiers to private workspace `reader` and `editor` grants while preserving issuer isolation, direct-membership precedence, and guarded-write rechecks.

**Architecture / 架构：** `contextlab-auth` owns opaque identities, bounded groups, role precedence, and permission checks. `contextlab-storage` owns PostgreSQL persistence and transaction-local assignment resolution; `server/api` only composes protected OIDC configuration. No public mutation route, OpenAPI operation, TypeScript SDK method, or GraphDiff behavior changes.

**Tech Stack / 技术栈：** Rust stable, Axum, SQLx/PostgreSQL, Serde JWT/OIDC validation, Cargo tests.

---

### Task 1: Define framework-independent authorization values / 定义无框架授权值

**Files / 文件：**
- Modify / 修改：`crates/auth/src/authorization.rs`
- Test / 测试：`crates/auth/src/authorization.rs`

- [x] Add `ExternalGroupId` plus `TrustedExternalGroups` with case-sensitive opaque validation, deterministic deduplication, a 128-item cap, and a 16 KiB total identifier budget.
- [x] Add `GroupWorkspaceRole` restricted to `reader` and `editor`, and make `WorkspaceRoleAssignments` choose a direct role before any group role.
- [x] Verify domain coverage with `cargo test -p contextlab-auth --lib` (42 passed in the latest delivery run).

### Task 2: Verify bounded OIDC group claims / 验证有界 OIDC group claim

**Files / 文件：**
- Modify / 修改：`crates/auth/src/oidc.rs`
- Test / 测试：`crates/auth/src/oidc.rs`

- [x] Add opt-in `.with_group_claim()` parsing only after normal signature, algorithm, issuer, audience, and expiry verification.
- [x] Reject invalid claim names, malformed arrays, invalid values, excessive counts or bytes, future `iat`, missing `iat`, and a lifetime exceeding the configured bound; HMAC principals retain no groups.
- [x] Require exact `iss` and `aud`, accept a validated audience array containing the configured client, reject every future `nbf` with zero leeway, and use single-flight JWKS refresh plus a bounded one-second unknown/failed-refresh cooldown to limit unauthenticated refresh pressure.
- [x] Keep failures at the existing safe authentication boundary without logging claim values or credentials.

### Task 3: Persist workspace group grants / 持久化 workspace group grant

**Files / 文件：**
- Create / 新建：`crates/storage/migrations/0007_workspace_external_group_role_bindings.sql`
- Modify / 修改：`crates/storage/src/postgres.rs`
- Test / 测试：`crates/storage/src/postgres.rs`

- [x] Create workspace/source/group scoped `reader` and `editor` bindings with `C` collation and byte-bounded opaque identifiers.
- [x] Use an active partial unique index on `(workspace_id, identity_source, external_group_id)` so soft-deleted rows remain historical evidence while a later active grant is valid.
- [x] Resolve direct membership first and re-resolve assignments inside the guarded commit transaction under `FOR UPDATE`.
- [x] Verify the storage unit suite with `cargo test -p contextlab-storage --lib` (114 passed, 9 PostgreSQL-gated tests ignored in the delivery run).

### Task 4: Compose private protected runtime configuration / 组合 private protected runtime 配置

**Files / 文件：**
- Modify / 修改：`server/api/src/lib.rs`
- Modify / 修改：`.env.example`
- Test / 测试：`server/api/src/lib.rs`

- [x] Read `CONTEXTLAB_OIDC_GROUPS_CLAIM` and `CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS` only for protected OIDC runtime configuration.
- [x] Keep public mode unchanged and keep the commit mutation absent from public OpenAPI and the TypeScript SDK.
- [x] Verify the API library suite with `cargo test -p contextlab-api --lib` (111 passed in the delivery run).

### Task 5: Record boundaries and remaining gates / 记录边界与剩余门禁

**Files / 文件：**
- Modify / 修改：`README.md`, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/adr/0002-guarded-context-commit-writes.md`, `docs/roadmap/*.md`

- [x] Document issuer isolation, direct-role override, no-group-owner rule, private-only API scope, and the partial-index soft-delete contract in English and Chinese.
- [x] Keep `GraphDiff` as the sole graph-diff calculator and leave its REST/SDK behavior unchanged.
- [ ] Run disposable PostgreSQL upgrade and revocation-race tests in CI and validate production migration rehearsal; these remain explicit release gates, not skipped evidence.
- [ ] Complete shared atomic multi-replica rate limiting and audit retention/access governance before evaluating public protected write publication.
