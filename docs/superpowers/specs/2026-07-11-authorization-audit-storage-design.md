# Authorization Audit Storage Design / 授权审计存储设计

## Goal / 目标

Persist every authorization decision emitted by the guarded Context commit route through the existing framework-independent `AuthorizationAuditSink` port. PostgreSQL runtime composition must use the durable adapter automatically, while preview and in-memory composition retain the explicit no-op sink. The protected mutation remains absent from the public REST, OpenAPI, and TypeScript SDK contracts.

通过既有、独立于框架的 `AuthorizationAuditSink` port 持久化受保护 Context commit route 发出的每条 authorization decision。PostgreSQL runtime composition 必须自动使用持久化 adapter；preview 与 in-memory composition 继续显式使用 no-op sink。protected mutation 仍不得进入 public REST、OpenAPI 与 TypeScript SDK contract。

## Decision / 决策

Use the existing `PostgresContextGraphRepository` as the infrastructure adapter. It already owns the SQLx pool and implements storage-facing authorization behavior, so implementing `AuthorizationAuditSink` there preserves the dependency direction `storage -> auth` and avoids an API handler that knows database tables.

使用既有的 `PostgresContextGraphRepository` 作为 infrastructure adapter。它已经持有 SQLx pool 并实现 storage-facing authorization 行为，因此在此实现 `AuthorizationAuditSink` 能保持 `storage -> auth` 的依赖方向，避免 API handler 直接了解数据库表。

### Alternatives considered / 备选方案

1. **Repository adapter (selected) / Repository adapter（采用）**: add a PostgreSQL migration and implement the existing port on `PostgresContextGraphRepository`. Runtime selects it only for PostgreSQL. This keeps domain contracts reusable and makes storage failures visible as the existing `AuthorizationAuditError::Unavailable`.
2. **Direct API SQL / API 直接执行 SQL**: rejected because Axum would own persistence details and duplicate the storage boundary.
3. **Event bus or transactional outbox / 事件总线或 transactional outbox**: deferred. They are appropriate for external audit replication, but add delivery, retry, and operational semantics before the first durable local audit record exists.

1. **Repository adapter（采用）**：增加 PostgreSQL migration，并在 `PostgresContextGraphRepository` 上实现既有 port。runtime 只在 PostgreSQL 模式选择它，从而保持 domain contract 可复用，并让 storage failure 通过既有 `AuthorizationAuditError::Unavailable` 可见。
2. **API 直接执行 SQL**：拒绝，因为 Axum 会持有 persistence detail，破坏并重复 storage boundary。
3. **事件总线或 transactional outbox**：延期。它们适合外部 audit replication，但在第一条本地持久化 audit record 尚不存在时就引入 delivery、retry 与运维语义，范围过大。

## Data Model / 数据模型

Migration `0005_context_authorization_audit_events.sql` creates append-only `context_authorization_audit_events`:

| Column | Constraint and purpose / 约束与用途 |
| --- | --- |
| `id` | UUID primary key, database generated / UUID 主键，由数据库生成 |
| `principal_id` | non-empty text subject, never a bearer credential / 非空主体文本，绝不保存 bearer credential |
| `context_id` | non-null FK to `contexts(id)` / 非空外键，指向 `contexts(id)` |
| `permission` | checked `read` or `write` value / 受检的 `read` 或 `write` |
| `decision` | checked `granted`, `forbidden`, or `unavailable` / 受检的三种决策值 |
| `recorded_at` | non-null database timestamp / 非空数据库时间戳 |

The table has an index on `(context_id, recorded_at DESC, id DESC)` for future scoped forensic inspection and an index on `(principal_id, recorded_at DESC, id DESC)` for future actor-scoped review. This slice intentionally adds no reader, public endpoint, SDK type, retention policy, or mutable metadata.

A database trigger rejects every `UPDATE` and `DELETE` against the table, so append-only is an enforced persistence invariant rather than an application convention. Retention is deferred to an explicit, separately reviewed governance workflow.

表为未来按 Context 的取证检查建立 `(context_id, recorded_at DESC, id DESC)` 索引，并为未来按主体审查建立 `(principal_id, recorded_at DESC, id DESC)` 索引。本切片有意不增加 reader、public endpoint、SDK type、retention policy 或可变 metadata。

数据库 trigger 会拒绝表上的所有 `UPDATE` 与 `DELETE`，因此 append-only 是受强制执行的 persistence invariant，而不是应用层约定。retention 延后到单独评审的 governance workflow。

## Runtime Composition / 运行时组合

`AppState::try_from_env` already chooses an in-memory or PostgreSQL `WorkspaceDataRepository`. After constructing the repository bundle, it will obtain an optional durable audit sink: PostgreSQL returns a clone of the repository; memory returns none. `AppState` installs that sink only when available, retaining `NoopAuthorizationAuditSink` otherwise. A protected router built from PostgreSQL state therefore fails closed when the audit insert cannot be accepted.

`AppState::try_from_env` 已会选择 in-memory 或 PostgreSQL `WorkspaceDataRepository`。构造 repository bundle 后，它将取得可选的 durable audit sink：PostgreSQL 返回 repository clone，memory 返回 none。只有 sink 存在时 `AppState` 才安装它，否则保留 `NoopAuthorizationAuditSink`。因此，由 PostgreSQL state 构造的 protected router 在 audit insert 无法被接受时会 fail closed。

The route continues to calculate the authorization result once, record the corresponding event before the guarded writer is invoked, and map any sink failure to `503 authorization_audit_unavailable`. The audit table stores no token, request body, idempotency key, graph payload, or provider secret.

route 仍只计算一次 authorization result，在调用 guarded writer 前记录对应 event，并把任何 sink failure 映射为 `503 authorization_audit_unavailable`。audit table 不保存 token、request body、idempotency key、graph payload 或 provider secret。

## Verification / 验证

1. Migration unit tests assert the table, all enum checks, foreign key, and both indexes are present in the composed migration asset.
2. A focused ignored disposable PostgreSQL test records granted, forbidden, and unavailable events through `PostgresContextGraphRepository`, rejects both mutation operations, then verifies the exact persisted safe fields and database-generated timestamps.
3. An API composition test proves the existing injected failing sink still returns `503` before any commit appears; existing public-route/OpenAPI tests prove no mutation is promoted.
4. Full release checks remain `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, and `python apps/web/verify-context-workspace.py`.

1. Migration unit test 断言 composed migration asset 包含表、所有 enum check、foreign key 与两个 index。
2. focused ignored disposable PostgreSQL test 通过 `PostgresContextGraphRepository` 记录 granted、forbidden 与 unavailable event，拒绝两种 mutation operation，再验证精确的安全字段与数据库生成时间戳。
3. API composition test 证明既有 injected failing sink 仍在任何 commit 出现前返回 `503`；既有 public-route/OpenAPI test 证明 mutation 未被提升。
4. 完整 release check 保持为 `cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web` 与 `python apps/web/verify-context-workspace.py`。
