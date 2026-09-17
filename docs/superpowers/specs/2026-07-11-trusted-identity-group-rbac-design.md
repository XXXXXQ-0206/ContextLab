# Trusted Identity and Group RBAC Design / 可信身份与组 RBAC 设计

## Goal / 目标

Provide a production-oriented path from authenticated identities and trusted external groups to existing ContextLab workspace roles without moving permission policy out of `contextlab-auth`, weakening transactional guarded-write checks, or publishing the private Context commit mutation through OpenAPI or the TypeScript SDK.

建立从认证身份与可信外部组到 ContextLab 既有 workspace role 的生产级路径，同时保持 permission policy 归属 `contextlab-auth`、不削弱 guarded write 的事务内复核，也不通过 OpenAPI 或 TypeScript SDK 公开 private Context commit mutation。

## Decision Summary / 决策摘要

The work is split into two dependency-ordered increments.

工作拆为两个有明确依赖顺序的增量。

1. **Issuer-scoped identity namespace / issuer 作用域身份命名空间.** Replace bare-subject security keys with a validated `(identity_source, subject)` identity. HMAC uses its validated issuer as the identity source; OIDC uses the exact issuer already verified in the token. Rate limiting, direct workspace membership, guarded-write idempotency, and authorization audit all use the same composite identity.
2. **Trusted group-to-role mapping / 可信组到角色映射.** OIDC may extract one explicitly configured group claim only after signature, algorithm, issuer, audience, expiry, and token-lifetime validation. PostgreSQL stores workspace-scoped mappings keyed by the exact identity source and opaque group identifier. Direct managed membership overrides group grants; external groups may grant `reader` or `editor`, never `owner` in the first version.

1. **issuer 作用域身份命名空间。** 将裸 subject security key 替换为经过校验的 `(identity_source, subject)` identity。HMAC 使用已校验 issuer 作为 identity source；OIDC 使用 token 中已精确验证的 issuer。rate limit、direct workspace membership、guarded-write idempotency 与 authorization audit 使用同一复合身份。
2. **可信组到角色映射。** OIDC 仅在 signature、algorithm、issuer、audience、expiry 与 token lifetime 校验后，提取一个显式配置的 group claim。PostgreSQL 以精确 identity source 与 opaque group identifier 保存 workspace-scoped mapping。direct managed membership 覆盖 group grant；第一版 external group 只能授予 `reader` 或 `editor`，不能授予 `owner`。

The identity namespace is a mandatory prerequisite. Persisting group mappings against bare `sub` would let two identity providers collide and could incorrectly share quota, audit history, membership, or idempotency records.

身份命名空间是强制前置条件。若 external group mapping 继续绑定裸 `sub`，两个 identity provider 可能发生碰撞，并错误共享 quota、audit history、membership 或 idempotency record。

## Alternatives / 备选方案

### Selected: verified claims plus explicit database bindings / 采用：已验证 claim + 显式数据库绑定

The authenticator emits typed identity context and bounded group identifiers but never emits a role. PostgreSQL resolves assignments, while `contextlab-auth` applies direct-membership precedence and role-to-permission policy. The same principal reaches both preflight authorization and the guarded writer transaction.

authenticator 输出 typed identity context 与有界 group identifier，但绝不输出 role。PostgreSQL 解析 assignment，`contextlab-auth` 应用 direct-membership precedence 与 role-to-permission policy。同一个 principal 同时进入 preflight authorization 与 guarded writer transaction。

### Rejected: map JWT roles directly / 拒绝：直接映射 JWT role

Provider-specific `roles`, `scope`, or display-name claims are not stable ContextLab authorization contracts. Direct mapping would move policy into authentication and make revocation, workspace scoping, and auditability inconsistent.

provider-specific `roles`、`scope` 或 display-name claim 不是稳定的 ContextLab authorization contract。直接映射会把 policy 移入 authentication，并导致 revocation、workspace scope 与 auditability 不一致。

### Deferred: synchronized external directory / 延后：同步外部目录

A directory synchronization adapter can later materialize group membership with richer lifecycle metadata. It remains compatible with the same identity and workspace-binding contracts, but it is not required for the first OIDC-backed mapping path.

directory synchronization adapter 后续可用更丰富的 lifecycle metadata 物化 group membership，并复用同一 identity 与 workspace-binding contract；首个 OIDC-backed mapping path 不依赖它。

## Domain Contracts / 领域契约

`contextlab-auth` owns these framework-independent values:

- `IdentitySourceId`: opaque, case-sensitive, non-empty, no surrounding whitespace or control characters, at most 2,048 UTF-8 bytes.
- `PrincipalId`: opaque, case-sensitive subject, non-empty, no surrounding whitespace or control characters, at most 512 UTF-8 bytes.
- `PrincipalIdentity`: one `IdentitySourceId` plus one `PrincipalId`; this is the stable security key.
- `ExternalGroupId`: opaque, case-sensitive, non-empty, no surrounding whitespace or control characters, at most 512 UTF-8 bytes.
- `TrustedExternalGroups`: deduplicated and deterministically ordered, at most 128 groups and at most 16 KiB of identifier bytes in total.
- `AuthenticatedPrincipal`: one `PrincipalIdentity` plus `TrustedExternalGroups`; constructors require explicit identity source and never infer roles.
- `WorkspaceRoleAssignments`: optional direct role plus zero or more group roles. Direct membership is an explicit override. Without a direct role, authorization succeeds when any group role grants the requested permission.

`contextlab-auth` 拥有以上 framework-independent value。所有标识均为 case-sensitive opaque value；不得 lowercase、trim 后合并、使用 display name，或从 request header/body 推断。

`ContextRoleResolver` returns `WorkspaceRoleAssignments`, not one database-selected effective role. PostgreSQL finds assignments; the auth domain owns precedence and `WorkspaceRole::allows`.

`ContextRoleResolver` 返回 `WorkspaceRoleAssignments`，而不是由数据库预先挑选的单个 effective role。PostgreSQL 只查找 assignment；precedence 与 `WorkspaceRole::allows` 仍属于 auth domain。

## Authentication Flow / 认证流程

HMAC and OIDC both construct a `PrincipalIdentity` from the issuer accepted by their verifier and the validated subject. Existing callers must supply an explicit source when constructing test or internal principals; there is no silent `legacy` constructor.

HMAC 与 OIDC 都使用 verifier 已接受的 issuer 和经过校验的 subject 构造 `PrincipalIdentity`。test 或内部调用方构造 principal 时也必须显式提供 source，不提供静默 `legacy` constructor。

OIDC group extraction is disabled unless protected runtime config provides `CONTEXTLAB_OIDC_GROUPS_CLAIM`. The first version accepts a simple top-level claim name matching `[A-Za-z0-9_.-]{1,64}`. `exp`, `iss`, and `aud` are required; issuer and audience must exactly match protected configuration, including an audience array containing the configured client. A present `nbf` is checked with zero leeway. The verified group claim must be an array of strings. Missing group claim means no groups; wrong type, malformed element, excessive count, excessive item length, or excessive total bytes rejects authentication with the existing safe `401 authentication_failed` response. Values are never truncated. JWKS network I/O occurs outside the cache state mutex; a cached known key remains usable during an unknown-`kid` refresh. Unknown or failed refreshes use a bounded one-second per-process cooldown without retaining attacker-supplied key IDs.

只有 protected runtime 配置 `CONTEXTLAB_OIDC_GROUPS_CLAIM` 时才启用 OIDC group extraction。第一版只接受匹配 `[A-Za-z0-9_.-]{1,64}` 的顶层 claim name。`exp`、`iss` 与 `aud` 为必需项，issuer 与 audience 必须精确匹配 protected configuration，包括含有已配置 client 的 audience array；若存在 `nbf`，以零容差校验。已验证的 group claim 必须是 string array。group claim 缺失表示无 group；类型错误、元素非法、数量过多、单项过长或总字节超限均通过既有安全 `401 authentication_failed` 拒绝，绝不截断。JWKS 网络 I/O 在 cache state mutex 之外执行；缓存中已知的 key 在未知 `kid` refresh 期间仍可用。未知或失败 refresh 使用有界的一秒钟进程内冷却，且不保留攻击者提供的 key ID。

Group-derived authorization is enabled only when OIDC also requires `CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS` in `1..=3600`. Tokens must contain `iat`, and `exp - iat` must not exceed this bound. This limits the delay before IdP-side group revocation takes effect. HMAC remains managed-membership-only in this increment.

只有 OIDC 同时要求 `CONTEXTLAB_OIDC_MAX_TOKEN_LIFETIME_SECONDS`（`1..=3600`）时，group-derived authorization 才能启用。token 必须包含 `iat`，且 `exp - iat` 不得超过该上限，以限制 IdP-side group revocation 的生效延迟。本增量中 HMAC 仍仅使用 managed membership。

## Persistence / 持久化

Migration `0006_principal_identity_namespace.sql` adds `identity_source` to:

- `workspace_memberships`, changing the key to `(workspace_id, identity_source, principal_id)`;
- `context_commit_idempotency`, changing the key to `(identity_source, principal_id, context_id, idempotency_key)`;
- `context_authorization_audit_events`, preserving immutable source and subject evidence.

Upgrade rows receive the explicit sentinel `legacy` only during migration. Protected runtime never falls back from its configured issuer to `legacy`; operators must deliberately migrate old grants. This avoids invisible cross-issuer authorization.

upgrade row 仅在 migration 中获得显式 sentinel `legacy`。protected runtime 不会从配置 issuer 回退到 `legacy`；operator 必须显式迁移旧 grant，避免不可见的跨 issuer authorization。

Migration `0007_workspace_external_group_role_bindings.sql` adds:

```text
workspace_external_group_role_bindings
  workspace_id UUID FK
  identity_source TEXT
  external_group_id TEXT
  role TEXT CHECK reader|editor
  created_at / updated_at / deleted_at
  UNIQUE (workspace_id, identity_source, external_group_id)
    WHERE deleted_at IS NULL
```

The migration deliberately has no table-wide primary key for this binding. Its active partial unique index allows a revoked binding to remain as historical evidence while one later active binding with the same workspace, source, and group can be created. The database collation is `C` for both opaque identifiers.

该 binding 的迁移刻意不设置全表 primary key。其 active partial unique index 允许已撤销 binding 作为历史证据保留，同时允许之后以相同 workspace、source 与 group 创建一个新的 active binding。两个不透明标识都使用数据库 `C` collation。

The first version does not store token payloads or duplicate IdP group membership. Bindings are workspace-scoped grants. Soft-deleted workspaces and bindings never authorize. A later management/synchronization API must use a separate audited write contract and does not belong in the current public SDK.

第一版不保存 token payload，也不重复存储 IdP group membership。binding 是 workspace-scoped grant。soft-deleted workspace 与 binding 绝不授权。后续 management/synchronization API 必须使用独立且可审计的 write contract，不属于当前 public SDK。

## Resolution and Transaction Rules / 解析与事务规则

1. Resolve the active Context through active Project and Workspace rows.
2. Read the direct membership for the exact `PrincipalIdentity`.
3. If direct membership exists, return it as the override and do not elevate it with groups.
4. Otherwise, read active group bindings matching the exact identity source and one of the trusted group IDs.
5. Decode every stored role through `WorkspaceRole::from_storage`; unknown values fail closed as `AuthorizationError::Unavailable`.
6. The authorizer applies `WorkspaceRoleAssignments::allows`.

guarded writer 在 transaction 内执行同一 assignment 解析，并对命中的 workspace、direct membership 或 group binding 使用足以阻止 role change/soft delete 的 row lock。preflight grant 在 transaction 内被撤销时，写入必须失败，且不能留下 commit、snapshot、branch 或 idempotency partial state。

## API and SDK Boundary / API 与 SDK 边界

This work changes only private protected runtime composition and internal Rust contracts. It adds no public route, OpenAPI operation, TypeScript SDK method, GraphDiff behavior, or Web mutation control. Existing contract tests continue to assert that commit creation is absent from public OpenAPI and `ContextLabClient`.

本工作只改变 private protected runtime composition 与内部 Rust contract，不增加 public route、OpenAPI operation、TypeScript SDK method、GraphDiff behavior 或 Web mutation control。既有 contract test 继续断言 public OpenAPI 与 `ContextLabClient` 不存在 commit creation。

## Failure Semantics / 失败语义

- Invalid identity or group claim: `401 authentication_failed`.
- Valid identity with no matching assignment: `403 context_write_forbidden` after authorization audit.
- Assignment repository failure or malformed stored role: `503 authorization_unavailable` after fail-closed audit handling.
- Mapping revoked before transactional recheck: guarded writer returns the existing stable forbidden/unavailable storage mapping without partial writes.
- No error includes issuer, subject, group identifier, token, key, database URL, or credential material.

## Testing / 测试

Domain tests cover opaque identifier boundaries, case sensitivity, control characters, deterministic group deduplication, total bounds, direct-role precedence, group-role evaluation, and unavailable resolver behavior.

Authentication tests cover exact issuer scoping, same subject under different issuers, configured/disabled/missing/malformed group claims, count/item/total bounds, required `iat`, maximum token lifetime, and no group-derived role output.

Migration and PostgreSQL tests cover upgrade preservation, composite uniqueness, issuer isolation, direct membership compatibility, direct-reader override of group-editor, group-only reader/editor, owner rejection, soft deletes, unknown role failure, row-lock revocation races, and no partial guarded write. Disposable database tests remain explicit until CI supplies isolated databases.

API/SDK tests cover protected config validation, HMAC compatibility, public mode ignoring OIDC group settings, composite rate-limit isolation, audit identity preservation, and continued absence of public commit mutation.

## Rollout / 发布顺序

1. Implement and verify the issuer-scoped identity namespace end to end.
2. Document the migration procedure from `legacy` direct memberships.
3. Add bounded OIDC group claims and private runtime configuration.
4. Add workspace group-role binding persistence and transactional resolution.
5. Run disposable PostgreSQL upgrade/concurrency evidence before considering any public write contract.

The long-term goal remains open after every step. Audit retention/access governance, shared atomic multi-replica limiting, disposable PostgreSQL CI, and production migration evidence remain subsequent gates.

每一步完成后长期目标仍保持开放。audit retention/access governance、共享原子 multi-replica limiter、disposable PostgreSQL CI 与 production migration evidence 仍是后续门禁。
