# Audit Retention and Access Governance Implementation Plan / 授权审计留存与访问治理实施计划

> **For agentic workers / 面向智能体执行者：** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` for independent implementation and two-stage review. Steps use checkbox syntax for traceable delivery.

**Goal / 目标：** Add a private, owner-governed foundation for reviewing and retaining authorization-decision evidence without exposing identities, credentials, commit content, or a public audit API.

**Architecture / 架构：** `contextlab-auth` owns review-access policy and requires a direct workspace `Owner`; group roles and Context read/write permissions never imply audit-review access. `contextlab-storage` owns direct-role resolution plus the first storage-only retention foundation: immutable policy revisions, workspace active-policy scopes, hold-by-default event metadata, and append-only purge manifests. A restricted purge executor is deliberately deferred until an isolated runtime database-role contract and disposable PostgreSQL evidence exist. The API, OpenAPI, TypeScript SDK, and Web runtime remain unchanged until a separately reviewed private operator boundary exists.

**Tech Stack / 技术栈：** Rust stable, Tokio, SQLx/PostgreSQL, Axum composition tests, Cargo tests.

---

### Task 1: Define owner-only audit-review access / 定义仅 owner 的审计审查访问

**Files / 文件：**
- Modify / 修改：`crates/auth/src/authorization.rs`, `crates/auth/src/lib.rs`
- Test / 测试：`crates/auth/src/lib.rs`

- [x] Write failing domain tests where a direct `Owner` can review one `WorkspaceId`, while direct `Editor`, direct `Reader`, an absent role, and an unavailable resolver cannot.
- [x] Add the framework-independent port and policy:

```rust
#[async_trait]
pub trait WorkspaceRoleResolver: Send + Sync {
    async fn resolve_workspace_role(
        &self,
        principal: &AuthenticatedPrincipal,
        workspace_id: WorkspaceId,
    ) -> Result<Option<WorkspaceRole>, AuthorizationError>;
}

pub struct DirectOwnerAuthorizationAuditReviewAuthorizer<R> {
    resolver: R,
}
```

- [x] Map only `Some(WorkspaceRole::Owner)` to `Ok(())`; map other active direct roles and no role to `AuthorizationError::Forbidden`, and infrastructure failures to `AuthorizationError::Unavailable`.
- [x] Re-export the port and authorizer from `contextlab-auth`; do not create a REST route, DTO, SDK method, or mutable management control.
- [x] Implement `WorkspaceRoleResolver` for `PostgresContextGraphRepository` with active-workspace, exact-source, exact-subject direct membership resolution only; group bindings remain excluded from audit-review authorization.

### Task 2: Add workspace retention-policy persistence / 增加 workspace 留存策略持久化

**Files / 文件：**
- Create / 新建：`crates/storage/migrations/0008_context_authorization_audit_governance.sql`
- Modify / 修改：`crates/storage/src/postgres.rs`
- Test / 测试：`crates/storage/src/postgres.rs`

- [x] Write migration-composition tests for workspace-scoped policy revisions, default `hold` disposition on existing events, `purge_eligible_at`, and append-only purge manifests. An ignored disposable-PostgreSQL upgrade test also checks default hold and ordinary mutation rejection; versioning rows remain outside this migration.
- [x] Create `context_authorization_audit_retention_policies` with immutable revision identity, an independently mutable workspace active-policy scope, non-negative retention duration, and database timestamps. Events without an explicit policy remain on hold and are never automatically deleted.
- [x] Add immutable event disposition fields, Context-workspace policy validation, and an expiry-selection index. Preserve the existing append-only event trigger for ordinary `UPDATE` and `DELETE`.
- [ ] Defer a restricted purge executor. Do not add `SECURITY DEFINER` or an ordinary delete exception until isolated runtime database roles, executor ownership, and disposable PostgreSQL evidence prove the restricted path is enforceable.

### Task 3: Add redacted private review contracts / 增加去标识化 private 审查契约

**Files / 文件：**
- Create / 新建：`crates/storage/src/authorization_audit_review.rs`
- Modify / 修改：`crates/storage/src/lib.rs`, `crates/storage/src/postgres.rs`
- Test / 测试：`crates/storage/src/authorization_audit_review.rs`, `crates/storage/src/postgres.rs`

- [x] Define a workspace-scoped `AuthorizationAuditReviewRepository` that returns only event timestamp, Context identifier, permission, decision, retention disposition, and policy revision. Its DTO excludes `identity_source`, `principal_id`, external group identifiers, bearer tokens, request bodies, idempotency keys, digests, commit IDs, graph snapshots, and database errors.
- [x] Add stable cursor pagination ordered by `(recorded_at DESC, id DESC)` only after the caller has passed the direct-owner access boundary; the audit event ID stays inside the opaque cursor.
- [x] Implement the PostgreSQL adapter with active-workspace filtering in SQL before pagination and map unknown/deleted workspaces to an unavailable scope without cross-workspace enumeration.
- [ ] Added an ignored disposable-PostgreSQL redaction/pagination test and an ignored upgrade test for hold-by-default/ordinary mutation rejection. Run them against a fresh disposable database; eligible-purge and untouched versioning assertions remain deferred with the restricted purge executor.

### Task 4: Preserve non-public operation and evidence / 保持非公开操作与证据

**Files / 文件：**
- Modify / 修改：`README.md`, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/adr/0002-guarded-context-commit-writes.md`, `docs/roadmap/*.md`
- Test / 测试：`server/api/src/lib.rs`, `packages/ts-sdk/src/*.test.ts`

- [x] Document that an authorization `granted` event records a pre-write decision, not a commit-creation outcome; a future outcome ledger needs a separate transaction contract.
- [x] Document direct-owner-only review, redacted fields, hold-by-default retention, purge manifests, and the lack of public routes in English and Chinese.
- [ ] Keep public route-catalog, OpenAPI, SDK, and Web tests asserting no audit-review mutation or discovery operation exists.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, visual workspace QA, and disposable PostgreSQL evidence when Docker is available; never read `.env` or connect to an unspecified database.
