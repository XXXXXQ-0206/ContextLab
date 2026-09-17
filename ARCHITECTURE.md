# ContextLab 系统架构（Architecture）

## 一、总体目标

ContextLab 是一个面向 AI Context Engineering 的开源平台。

它不是聊天软件，不是 Prompt 管理器，而是一套完整的 AI 工作流基础设施。

整个系统采用模块化设计，每个模块职责单一、边界清晰，可独立开发、测试、部署和扩展。

任何新增功能都必须归属于已有模块，或新增独立模块，禁止跨模块堆积业务逻辑。

---

# 二、总体架构

```
┌─────────────────────────────────────────────┐
│                 Web / Desktop               │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│        SDK / Client Boundary                │
│        SDK / 客户端边界                      │
└─────────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────────┐
│                 API Gateway                 │
└─────────────────────────────────────────────┘
                    │
──────────────────────────────────────────────────
                    │
        ContextLab Core Platform
                    │
──────────────────────────────────────────────────

Context Engine

Version Engine

Diff Engine

Workflow Engine

Evaluation Engine

Memory Engine

Knowledge Engine

Embedding Engine

Context Graph

Plugin System

Model Gateway

MCP Gateway

Authentication

Workspace

Storage

Search

Telemetry

Scheduler

Notification

Audit Log

──────────────────────────────────────────────────
                    │
                    ▼
PostgreSQL
Redis
MinIO
pgvector
```

---

# 三、模块划分

## 1. Workspace

负责：

用户

组织

工作区

项目

权限

成员管理

这是所有资源的根节点。

---

## 2. Context Engine

整个系统最核心模块。

负责管理：

Prompt

Memory

Knowledge

Conversation

Variables

Output Schema

Model Config

Retrieval

Context 是所有 AI 能力的统一抽象。

---

## 3. Version Engine

负责：

Commit

Branch

Merge

History

Fork

Rollback

Replay

Tag

Snapshot

任何 Context 修改都必须经过 Version Engine。

---

## 4. Diff Engine

负责：

文本 Diff

语义 Diff

行为 Diff

Benchmark Diff

Evaluation Diff

Graph Diff

当前基础能力：`diff-engine` 提供确定性的 `GraphDiff`，用于比较两个显式 `ContextGraph` 快照中的节点新增、删除、类型或标签变更，以及关系的新增和删除。它按稳定节点标识和关系键输出结果，不依赖插入顺序。

The public graph-diff catalog is retired: `POST /api/v1/graph-diffs` and the corresponding public SDK operation are no longer public contracts. Graph diff is available only through the protected local read path `GET /api/v1/local/contexts/{context_id}/graph-diff`, which authenticates first, applies Context-scoped RBAC, records an authorization audit event, enforces the dedicated rate limit, and returns `Cache-Control: private, no-store` on both success and failure.

graph-diff 的 public catalog 已退役：`POST /api/v1/graph-diffs` 及对应的 public SDK operation 不再是 public contract。Graph diff 只通过 protected local read path `GET /api/v1/local/contexts/{context_id}/graph-diff` 提供；该 path 先完成 authentication，再执行 Context-scoped RBAC、记录 authorization audit event、执行专用 rate limit，并在成功与失败响应中都返回 `Cache-Control: private, no-store`。

The non-public local SDK and same-origin BFF are transport adapters for this GET only. They forward the exact Context scope and redacted response, do not implement authorization, audit, rate-limit, caching policy, or graph comparison, and do not restore the retired public catalog. Both layers preserve the private no-store boundary and keep credentials in request memory.

非公开 local SDK 与同源 BFF 仅作为该 GET 的 transport adapter。它们转发精确 Context scope 与脱敏 response，不实现 authorization、audit、rate-limit、缓存策略或 graph comparison，也不恢复已退役的 public catalog。两层都保持 private no-store boundary，并只在 request memory 中处理 credentials。

Every route, SDK, BFF, and Web adapter delegates graph comparison to the single domain calculator `GraphDiff::between`; no transport or presentation layer may recalculate graph differences.

每个 route、SDK、BFF 与 Web adapter 都必须委托唯一的 domain calculator `GraphDiff::between` 执行 graph comparison；transport 或 presentation layer 不得重新计算图差异。

This is a pure supplied-snapshot calculation, not a commit diff, persisted version snapshot, graph editing flow, or Web review workflow. Future versioning, storage, and UI work must reuse this contract rather than recalculating graph changes locally.

该能力仍只处理调用方提供的图快照；它不是 commit diff、持久化版本快照、图编辑或 Web review workflow。后续 versioning、storage 与 UI 工作必须复用这一契约，而不能在各层重新计算图差异。

The storage boundary defines `CommitGraphSnapshot` for associating an already-validated `ContextGraph` with one Context commit. It now also provides `CreateContextCommitSnapshot` and `ContextCommitSnapshotWriter`: a storage-only command and write boundary that persist one `ContextCommit`, its ordered same-Context parents, and exactly one immutable snapshot together. The in-memory adapter uses one synchronized overlay; PostgreSQL validates the active Context and all parents, then inserts the commit, parent positions, and JSONB snapshot in one transaction.

storage 边界定义了 `CommitGraphSnapshot`，用于将已验证的 `ContextGraph` 关联到单个 Context commit。现在还提供 `CreateContextCommitSnapshot` 与 `ContextCommitSnapshotWriter`：仅限 storage 的 command 和写入边界，将一个 `ContextCommit`、其有序且同 Context 的 parent，以及唯一的不可变 snapshot 一起持久化。内存 adapter 使用单个同步 overlay；PostgreSQL 会验证活跃 Context 和全部 parent，再在一个 transaction 内插入 commit、parent position 与 JSONB snapshot。

The local API adapter uses this read boundary to compare materialized commit snapshots from one Context through version-backed GraphDiff. Separately, storage and the API may provide guarded write boundaries with authorization, idempotency, and branch-head compare-and-swap behind explicitly configured protected routers; those writes are unrelated to the local graph-diff GET. This is a local development capability only. It is not a public-write, public-release, release-readiness, or production-readiness decision; any promotion requires a separate deployment and governance decision. Merge policy and graph editing remain separate collaboration increments.

local API adapter 通过该 read boundary 对同一 Context 的 materialized commit snapshot 执行 version-backed GraphDiff。storage 与 API 可以在显式配置的 protected router 后提供带 authorization、idempotency 与 branch-head compare-and-swap 的 guarded write boundary；这些 write 与 local graph-diff GET 无关。本能力仅用于本地开发，不等于 public write、public release、release readiness 或 production readiness；任何推广都必须经过独立的 deployment 与 governance decision。merge policy 与 graph editing 仍属于独立的 collaboration 增量。

The contract persists, reads, and captures immutable snapshots during commit creation in PostgreSQL through the same validated graph payload boundary. The protected local API, local SDK, and Web workspace expose read-only version-backed GraphDiff over two materialized commit snapshots; snapshot detail APIs, graph editing, and merge policy remain open.

该 contract 已通过同一验证式图载荷边界在 PostgreSQL 中持久化、读取并在 commit 创建时捕获不可变 snapshot。protected local API、local SDK 与 Web workspace 已对两个 materialized commit snapshot 暴露只读 version-backed GraphDiff；snapshot detail API、graph editing 与 merge policy 仍未完成。

`contextlab-diff-engine` also owns a private `VersionedContextScopeV1` contract for complete
semantic, behavior, and evaluation reviews. Each review side is bound to exact
`(ProjectId, ContextId, CommitId)` identity; cross-project, cross-Context, and self-comparisons
fail closed before the unified `ContextDiffService` runs. This contract reuses the existing
`SemanticSnapshotV1`, `BehaviorSnapshotV1`, `EvaluationSnapshotV1`, and the sole
`GraphDiff::between` calculation. It is not a persistence bridge: exact semantic sources, sealed
behavior outcomes, and fully observed evaluation evidence must exist independently before any
storage or transport adapter can materialize a complete review.

`contextlab-diff-engine` 还拥有一份用于完整 semantic、behavior 与 evaluation review 的 private
`VersionedContextScopeV1` contract。每个 review side 都绑定精确的 `(ProjectId, ContextId, CommitId)` identity；
cross-project、cross-Context 与 self-comparison 会在统一 `ContextDiffService` 运行前 fail closed。该 contract 复用既有的
`SemanticSnapshotV1`、`BehaviorSnapshotV1`、`EvaluationSnapshotV1` 与唯一的 `GraphDiff::between` calculation。它不是
persistence bridge：exact semantic source、sealed behavior outcome 与 fully observed evaluation evidence 必须独立存在，
之后 storage 或 transport adapter 才能 materialize complete review。

The private storage bridge now persists a validated `ContextDiffSnapshotV1` record against the exact
`(ProjectId, ContextId, CommitId, schema_version)` scope through `VersionedContextScopeV1`. Its
Memory and PostgreSQL repository implementations share immutable create/replay/conflict semantics,
microsecond capture normalization, deterministic `sha256:` digest validation, and fail-closed
schema/scope reads. Migration `0023_context_diff_snapshots.sql` uses composite foreign keys and an
append-only trigger. The private `PersistedContextDiffReviewService` now consumes two exact records
through this port, validates scope/schema again at the application boundary, and delegates the
complete comparison to `VersionedContextDiffReviewService`. It is a local Rust composition contract
only; no semantic review transport or Web mutation consumes it, and `GraphDiff::between` remains
the sole graph-diff calculator.

私有 storage bridge 现已通过 `VersionedContextScopeV1` 将已校验的 `ContextDiffSnapshotV1` record 持久化到 exact
`(ProjectId, ContextId, CommitId, schema_version)` scope。Memory 与 PostgreSQL repository 实现共享 immutable
create/replay/conflict 语义、微秒 capture normalization、确定性的 `sha256:` digest 校验以及 fail-closed 的
schema/scope read。migration `0023_context_diff_snapshots.sql` 使用 composite foreign key 与 append-only trigger。
私有 `PersistedContextDiffReviewService` 现通过该 port 消费两个 exact record，在 application boundary 再次校验
scope/schema，并将完整比较委托给 `VersionedContextDiffReviewService`。它仍只是本地 Rust composition contract；没有
semantic review transport 或 Web mutation 消费它，且 `GraphDiff::between` 仍是唯一 graph-diff calculator。

### Trusted Principal Namespace / 可信主体命名空间

`contextlab-auth` owns `IdentitySourceId`, `PrincipalId`, and their `PrincipalIdentity` composition. Both dimensions are opaque, case-sensitive validated values: the HMAC profile uses its configured validated issuer as the source, and the OIDC profile uses its exact validated issuer. `server/api` transports that typed identity only after authentication; it does not derive authorization, rate-limit, audit, or idempotency keys from a bare subject.

`contextlab-auth` 负责 `IdentitySourceId`、`PrincipalId` 及其组合 `PrincipalIdentity`。两个维度都是经过校验的不透明、大小写敏感值：HMAC profile 使用已配置并验证的 issuer 作为 source，OIDC profile 使用精确匹配的已验证 issuer。`server/api` 只会在 authentication 后传递这个类型化身份，不会再从裸 subject 推导 authorization、rate-limit、audit 或 idempotency key。

`contextlab-storage` persists and queries the exact source-and-subject pair for workspace membership, authorization audit records, and commit idempotency. The bounded in-process protected-route quota remains an in-memory `contextlab-auth` concern. Migration `0006_principal_identity_namespace.sql` is additive: it preserves pre-existing rows under the explicit reserved `legacy` source sentinel, uses byte limits and `C` collation for new identity values, replaces affected composite keys, and adds source-and-subject indexes. Existing invalid historical subjects remain readable through `NOT VALID` constraints, while new writes are checked. Operators own trusted legacy-row reconciliation; the migration never guesses an issuer and authenticators never accept `legacy` as a source. Migration `0007_workspace_external_group_role_bindings.sql` adds soft-deletable, workspace-scoped `reader`/`editor` grants keyed by exact identity source and opaque external group ID. `contextlab-auth` validates and canonically bounds groups only after optional OIDC claim verification, applies direct-membership precedence, and never derives `owner` from a group. PostgreSQL repeats that assignment check under `FOR UPDATE` before a guarded write. `GraphDiff` remains the sole graph-diff calculator.

`contextlab-storage` 会为 workspace membership、authorization audit record 与 commit idempotency 持久化并查询精确的 source-and-subject 组合。有界的进程内 protected-route quota 仍归属于内存中的 `contextlab-auth`。迁移 `0006_principal_identity_namespace.sql` 是增量式的：它把既有行保留在显式且保留的 `legacy` source sentinel 下，为新的 identity value 使用字节上限与 `C` collation，替换受影响的组合 key，并添加 source-and-subject index。既有的不规范历史 subject 会通过 `NOT VALID` constraint 保持可读，而新写入仍会被校验。可信 legacy 行的对齐由 operator 负责，迁移不会猜测 issuer，authenticator 也绝不接受 `legacy` 作为 source。迁移 `0007_workspace_external_group_role_bindings.sql` 增加按 workspace、精确 identity source 与不透明 external group ID 分区、可 soft-delete 的 `reader`/`editor` grant。`contextlab-auth` 只会在可选 OIDC claim 校验后验证并规范化限制 group，应用 direct-membership precedence，且绝不从 group 推导 `owner`。PostgreSQL 会在 guarded write 前以 `FOR UPDATE` 重复该 assignment 检查。`GraphDiff` 继续是唯一的图差异计算器。

Migration `0010_shared_protected_route_rate_limits.sql` supplies the next private persistence adapter without changing the auth port. A singleton configuration row pins the deployment-wide policy and last observed database time; each case-sensitive `(identity_source, principal_id, operation)` state row owns only a bounded timestamp queue. `PostgresProtectedRouteRateLimiter` holds a transaction-scoped PostgreSQL advisory lock while it validates the policy, rejects clock regression, reclaims expired state, enforces active-key capacity, locks the requested state row, and updates or rejects the sliding window. Any database error or policy drift maps to `RateLimitError::Unavailable`. The adapter is deliberately not composed into an HTTP route, OpenAPI, SDK, or Web surface, so it is evidence for a future multi-replica decision rather than public-write readiness.

迁移 `0010_shared_protected_route_rate_limits.sql` 在不改变 auth port 的前提下提供下一层私有 persistence adapter。singleton configuration row 固定 deployment-wide policy 与最后一次观察到的 database time；每个大小写敏感的 `(identity_source, principal_id, operation)` state row 只拥有一个有界 timestamp queue。`PostgresProtectedRouteRateLimiter` 在 transaction-scoped PostgreSQL advisory lock 内验证 policy、拒绝 clock regression、回收过期 state、执行 active-key capacity、锁定请求 state row，并更新或拒绝 sliding window。任意 database error 或 policy drift 都映射为 `RateLimitError::Unavailable`。该 adapter 刻意不组合进 HTTP route、OpenAPI、SDK 或 Web surface，因此它是未来 multi-replica 决策的证据，而不是 public-write readiness。

Migration `0011_shared_protected_route_rate_limit_hardening.sql` narrows the lock boundary after a concurrency review. The configuration row is immutable and stores the policy for active identity-operation keys. Every check takes a deterministic advisory lock only for its exact `(identity_source, principal_id, operation)` state; only a previously absent key additionally takes the global admission lock to reclaim expired rows, count capacity, and insert. A future or unordered timestamp is rejected as unavailable rather than being appended out of order. This keeps different active keys independently schedulable while preserving one atomic global admission decision.

迁移 `0011_shared_protected_route_rate_limit_hardening.sql` 在并发审阅后收窄锁边界。configuration row 不可变，并存储 active identity-operation key 的 policy。每次检查只为其精确的 `(identity_source, principal_id, operation)` state 获取确定性 advisory lock；只有此前不存在的 key 才会额外获取 global admission lock，以回收过期 row、统计 capacity 并插入。future 或无序 timestamp 会被作为 unavailable 拒绝，而不会乱序追加。这让不同 active key 可以独立调度，同时保持全局 admission decision 的原子性。

Migration `0008_context_authorization_audit_governance.sql` is a private storage foundation, not an operator endpoint. It adds immutable workspace-owned retention-policy revisions, mutable per-workspace active-policy pointers, and default `hold` disposition fields to existing and newly appended authorization-audit events. A non-hold event must carry a policy revision from the Context's workspace; the insertion trigger derives `purge_eligible_at` from the database-recorded timestamp plus its immutable retention duration. Purge-selection manifests are append-only and record a workspace, revision, bounded cutoff, and selected count, but no procedure can delete audit rows yet. The migration intentionally has no `SECURITY DEFINER` function because runtime database roles and restricted executor evidence are not established. Direct-owner review authorization remains a framework-independent domain policy; `contextlab-storage` now provides a redacted cursor-paginated repository after that boundary, while private operator transport, public API/SDK/Web changes, and GraphDiff changes remain out of scope.

迁移 `0008_context_authorization_audit_governance.sql` 是私有的存储基础，不是 operator endpoint。它增加不可变的 workspace 所有留存策略修订、可替换的按 workspace active-policy 指针，并为既有和新追加的 authorization-audit event 增加默认 `hold` disposition 字段。非 `hold` event 必须带有属于该 Context workspace 的 policy revision；insertion trigger 会用数据库记录的 timestamp 加上其不可变 retention duration 推导 `purge_eligible_at`。purge-selection manifest 是追加式的，记录 workspace、revision、有界 cutoff 与 selected count，但目前尚没有可删除 audit row 的 procedure。该迁移刻意不含 `SECURITY DEFINER` function，因为运行时 database role 与受限 executor evidence 尚未建立。direct-owner review authorization 仍是与框架无关的 domain policy；`contextlab-storage` 现在在该 boundary 后提供 redacted cursor-paginated repository，而 private operator transport、public API/SDK/Web 变更和 GraphDiff 变更仍不在范围内。

The disposable PostgreSQL verifier validates named storage contracts in an empty, loopback-only test service with an explicit database name, dedicated test role, and reset opt-in. The current local delivery scope requires fresh per-case reset evidence; remote CI is an unavailable, deferred future deployment prerequisite rather than a current core-development gate. Local evidence is not production migration evidence, does not establish a public audit transport, and does not change graph-diff semantics; `GraphDiff` remains the sole graph-diff calculator.

disposable PostgreSQL verifier 会在空的、仅限 loopback 的测试服务中验证指定存储契约，并要求显式数据库名称、专用测试角色与 reset 确认。当前本地交付范围要求新鲜的逐例 reset 证据；远端 CI 是外部条件不可用、明确延期的未来部署前置，而不是当前核心开发门禁。本地证据不是生产迁移证据，不建立公开审计传输，也不改变图差异语义；`GraphDiff` 仍是唯一的图差异计算器。

The private audit purge executor is a database-only boundary. An operator-provisioned non-login definer owns one `SECURITY DEFINER` procedure, while a dedicated runtime pool has only `EXECUTE` and cannot read or mutate audit tables directly. The procedure locks one workspace/policy batch, records redacted immutable manifest items, and deletes only entries listed by its transaction-local marker; it adds no HTTP, OpenAPI, SDK, Web, scheduler, or public write surface.

私有 audit purge executor 是仅数据库边界。由 operator provision 的无登录 definer 拥有唯一的 `SECURITY DEFINER` procedure，而专用 runtime pool 只有 `EXECUTE`，不能直接读取或修改 audit table。该 procedure 锁定一个 workspace/policy batch，记录去标识化且不可变的 manifest item，并只删除 transaction-local marker 列出的 event；它不新增 HTTP、OpenAPI、SDK、Web、scheduler 或 public write 表面。

未来支持 AI 自动解释：

为什么两个 Context 存在差异。

---

### Private Unborn-Branch Context Initialization / 私有未出生分支 Context 初始化

`ContextLifecycleOperation::Initialize` is the only private local lifecycle operation that targets `ExpectedBranchHead::Unborn`. For an already persisted active Context, `ContextLifecycleRootRepository` supplies only the Context display name; the framework-independent lifecycle service constructs exactly one parentless `CreatedContext` change and a one-node `context:{context_id}` graph snapshot. It creates no component, body revision, migration, or caller-owned graph payload. The existing guarded writer atomically persists the root commit, snapshot, branch head, and branch-scoped idempotency receipt; a same-branch retry returns the original root after later head advancement, while the same key may initialize a different unborn branch independently.

`ContextLifecycleOperation::Initialize` 是唯一以 `ExpectedBranchHead::Unborn` 为目标的私有 local lifecycle operation。对于已持久化且 active 的 Context，`ContextLifecycleRootRepository` 只提供 Context display name；与框架无关的 lifecycle service 会构造恰好一条无 parent 的 `CreatedContext` change 和只含 `context:{context_id}` 节点的 graph snapshot。它不创建 component、body revision、migration 或调用方拥有的 graph payload。既有 guarded writer 会原子持久化 root commit、snapshot、branch head 与 branch-scoped idempotency receipt；同一 branch 的 retry 即使发生在后续 head 推进后也会返回原 root，而相同 key 可以独立初始化另一个 unborn branch。

The private protected local route, non-public local SDK, same-origin BFF, and shared-primitive editor transport the tagged `initialize` intent with `expected_head_commit_id: null`. Every other lifecycle operation still requires an exact materialized head. Public REST/OpenAPI/public SDK/write controls and `GraphDiff::between` remain unchanged; Docker-disabled PostgreSQL runtime and authenticated browser runtime are unobserved.

私有 protected local route、非公开 local SDK、同源 BFF 与 shared-primitive editor 会以 `expected_head_commit_id: null` 传输带标签的 `initialize` intent。所有其他 lifecycle operation 仍要求精确的 materialized head。public REST/OpenAPI/public SDK/write control 与 `GraphDiff::between` 保持不变；Docker 关闭时 PostgreSQL runtime 与 authenticated browser runtime 未观测。

### Private Typed Component Uses Relationship Lifecycle / 私有类型化 Component Uses 关系生命周期

`ContextLifecycleOperation::AddUsesRelationship` and `RemoveUsesRelationship` accept only two distinct component identifiers at an exact materialized branch head. The server-owned lifecycle service reconstructs component state at that commit, requires both endpoints to be active in the same Context, and verifies each endpoint's graph node and taxonomy-defined Context-to-component relationship before deriving the successor graph. It can add or remove only one directed `GraphEdgeKind::Uses` edge: a duplicate add and a missing remove fail closed, while unrelated nodes and edges are preserved. The typed `ContextChange` stores only the endpoint identities, so relationship replay does not mutate component descriptor or body state; a later component removal drops every incident edge from its successor snapshot.

`ContextLifecycleOperation::AddUsesRelationship` 与 `RemoveUsesRelationship` 只接受两个不同的 component identifier，并且必须以精确的已 materialize branch head 为基点。由服务端拥有的 lifecycle service 会在该 commit 重建 component state，要求两个 endpoint 都是同一 Context 中的 active component，并在派生 successor graph 前校验各 endpoint 的 graph node 与由 taxonomy 定义的 Context-to-component relationship。它只能新增或移除一条有向 `GraphEdgeKind::Uses` edge：重复新增和移除不存在的关系都会 fail closed，其余 node 与 edge 保持不变。类型化 `ContextChange` 只记录 endpoint identity，因此 relationship replay 不会改动 component descriptor 或正文状态；后续 component removal 会从 successor snapshot 中删除与该 component 相连的所有 edge。

Relationship commits reuse the existing attachment-free guarded path: one exact normal parent, branch-head compare-and-swap, branch-scoped idempotency replay, and atomic persistence of the immutable commit, derived graph snapshot, head, and receipt. The protected local route, non-public local SDK, same-origin BFF, and shared-primitive editor carry only the tagged intent and endpoint identifiers. The local Web editor and mutation BFF remain default-deny unless the server-owned `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` value is exactly `true`; public REST/OpenAPI/public SDK remain unchanged. `GraphDiff::between` remains the sole graph-diff calculator. Docker-backed PostgreSQL runtime and authenticated browser mutation runtime for this Uses increment are unobserved.

relationship commit 复用既有、无需 component mutation attachment 的 guarded path：它具有一个精确的 normal parent、branch-head compare-and-swap、按 branch 划分的 idempotency replay，并原子持久化不可变 commit、派生 graph snapshot、head 与 receipt。protected local route、非公开 local SDK、同源 BFF 与 shared-primitive editor 只传输带标签 intent 和 endpoint identifier。除非服务端拥有的 `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` 值精确为 `true`，local Web editor 与 mutation BFF 始终默认拒绝；public REST/OpenAPI/public SDK 保持不变。`GraphDiff::between` 仍是唯一 graph-diff calculator。本次 Uses 增量的 Docker-backed PostgreSQL runtime 与 authenticated browser mutation runtime 均未观测。

### Private Component Content Creation / 私有 Component 正文创建

`contextlab-storage` now provides a private `ComponentContentCreationWrite` attachment on the existing guarded commit boundary. The attachment is mutually exclusive with an existing-component revision. It carries the commit-owned component UUID, kind, non-empty name, JSON metadata, opaque UTF-8 body, and capture time. `ContextChange::added_component_content_with_details` records the same descriptor and deterministic SHA-256 result hash, so replay preserves the component identity and content transition rather than generating a fresh ID.

`contextlab-storage` 现在在既有 guarded commit 边界上提供私有的 `ComponentContentCreationWrite` 附件。该附件与既有 component revision 互斥；它携带由 commit 所有的 component UUID、kind、非空 name、JSON metadata、不透明 UTF-8 正文与 capture time。`ContextChange::added_component_content_with_details` 会记录同一 descriptor 与确定性 SHA-256 result hash，因此 replay 保留 component identity 和 content transition，而不会生成新的 ID。

Before either adapter persists data, the guarded command requires exactly one matching private attachment for a body creation, body revision, or removal; one `UpdatedComponentDescriptor` transition instead requires exactly one private `ComponentDescriptorRevisionWrite`, never a body attachment or a mixed attachment set. That attachment validates the successor graph node and updates only the current component name, metadata, and timestamp in the same overlay or PostgreSQL transaction that commits the immutable commit, graph snapshot, branch head, and receipt. Idempotency is scoped exactly to `(identity source, principal, Context, branch, key)` in memory and PostgreSQL advisory-lock/query/uniqueness paths; migration `0017_branch_scoped_commit_idempotency.sql` backfills the branch from each referenced commit and extends the receipt primary key. PostgreSQL also verifies a replay receipt branch against its referenced commit before returning it. Creation still requires the replayable `AddedComponent` descriptor and the commit snapshot's `component:{id}` node plus its taxonomy-defined relationship from `context:{context_id}`. `StoredComponentKind` remains the shared source for projected graph node and edge kinds. Memory writes one synchronized overlay and projects current revision hash/timestamp over seeded and dynamic components; PostgreSQL replays idempotency before component existence checks, serializes new IDs with a transaction advisory lock, and commits the commit, graph snapshot, component, initial `NULL`-prior revision, branch head, and idempotency receipt together. Forward migration `0014` makes the `NULL` prior a database-enforced initial-revision state. Descriptor PostgreSQL tests compile but remain ignored and unobserved while Docker is disabled. `GraphDiff` remains the only graph-diff calculator.

在任一 adapter 持久化数据前，guarded command 对 body creation、body revision 或 removal 都要求恰好一个匹配的私有 attachment；单个 `UpdatedComponentDescriptor` transition 则必须恰好拥有一个私有 `ComponentDescriptorRevisionWrite`，绝不能使用 body attachment 或混合 attachment 集合。该 attachment 会校验 successor graph node，并仅在提交不可变 commit、graph snapshot、branch head 与 receipt 的同一 overlay 或 PostgreSQL transaction 中更新当前 component 的 name、metadata 与 timestamp。memory 与 PostgreSQL advisory-lock/query/uniqueness path 都将 idempotency 精确作用域限定为 `(identity source, principal, Context, branch, key)`；迁移 `0017_branch_scoped_commit_idempotency.sql` 从每个被引用 commit 回填 branch，并扩展 receipt primary key。PostgreSQL 还会在 replay 前验证 receipt branch 与其被引用 commit 一致。创建仍要求可回放的 `AddedComponent` descriptor，以及 commit snapshot 中的 `component:{id}` 节点和从 `context:{context_id}` 出发、由 taxonomy 定义的 relationship。`StoredComponentKind` 仍是投影 graph node 与 edge kind 的共享来源。Memory 在一个同步 overlay 中写入，并为 seeded 与 dynamic component 投影当前 revision hash/timestamp；PostgreSQL 会在 component existence check 前回放 idempotency，使用 transaction advisory lock 串行化新 ID，并将 commit、graph snapshot、component、初始 `NULL`-prior revision、branch head 与 idempotency receipt 一起提交。前向迁移 `0014` 使 `NULL` prior 成为由数据库约束的 initial-revision state。Docker 关闭时 descriptor PostgreSQL test 已编译但保持 ignored/未观测。`GraphDiff` 仍是唯一的 graph-diff 计算器。

### Private Component Content Replay / 私有 Component 正文回放

`ComponentContentRevisionRepository::get_component_content_at_commit` provides the private replay boundary for component bodies. It validates a target Context commit's complete normal first-parent chain before returning the nearest immutable revision, so a nearer captured body cannot hide unsupported history. It returns the revision's source commit identity, distinguishes a known no-body result, unknown Context, and unknown commit, and rejects merge ancestry, cycles, or malformed cross-Context parent edges. The in-memory implementation walks `ContextCommitRecord.parent_commit_ids`; PostgreSQL uses a parameterized recursive CTE with UUID path and parent-scope detection. This keeps ancestry semantics in reusable storage code and leaves public body reads, graph editing, merge policy, API/SDK/Web surfaces, operator transport, and `GraphDiff` unchanged.

`ComponentContentRevisionRepository::get_component_content_at_commit` 提供 component 正文的私有 replay boundary。它会在返回最近的不可变 revision 前验证 target Context commit 的完整 normal first-parent chain，因此较近的 captured body 不能掩盖不受支持的历史。它返回 revision 的 source commit identity，区分已知但无正文、不存在的 Context 与 unknown commit，并拒绝 merge ancestry、cycle 或不合规的跨 Context parent edge。内存实现遍历 `ContextCommitRecord.parent_commit_ids`；PostgreSQL 使用带 UUID path 和 parent-scope detection 的参数化 recursive CTE。这样 ancestry semantics 保持在可复用 storage code 中，而 public body read、graph editing、merge policy、API/SDK/Web surface、operator transport 与 `GraphDiff` 均保持不变。

### Private Component State at Commit Replay / 私有 Component 提交状态回放

`ComponentStateAtCommitRepository::get_component_state_at_commit` is a storage-only replay boundary for the descriptor state of one component at a target commit. It validates the complete normal first-parent ancestry before folding detailed `AddedComponent`, body-hash `UpdatedComponent`, descriptor-only `UpdatedComponentDescriptor`, and typed `RemovedComponent` changes from root to target. Creations and body updates cross-check their immutable revision witness; descriptor updates require a matching active kind, validated name/metadata, and no body hashes, retaining the existing content witness and content commit. A valid removal must match the effective component kind and prior hash, then makes later state absent. The result contains the descriptor, metadata, effective content hash, target commit, creation commit, and last content-change commit when state exists, but never exposes the component body. Neither adapter derives past state from the current component projection.

`ComponentStateAtCommitRepository::get_component_state_at_commit` 是一个仅限 storage 的 replay boundary，用于在目标 commit 重建单个 component 的 descriptor state。它会先验证完整的 normal first-parent ancestry，再从 root 到 target 折叠带详情的 `AddedComponent`、body-hash `UpdatedComponent`、descriptor-only `UpdatedComponentDescriptor` 与 typed `RemovedComponent` change。creation 与 body update 会和对应的不可变 revision witness 交叉校验；descriptor update 必须匹配有效 kind、经过校验的 name/metadata 且不携带 body hash，并保留已有 content witness 与 content commit。合法 removal 必须匹配有效 component kind 与 prior hash，随后使较晚 state 缺席。state 存在时结果包含 descriptor、metadata、有效 content hash、target commit、creation commit 与最近 content-change commit，但绝不暴露 component body。两个 adapter 都不会从当前 component projection 反推历史状态。

This remains private to reusable Rust storage contracts: it adds no REST route, OpenAPI operation, TypeScript SDK method, Web control, operator transport, or graph-diff implementation. `GraphDiff` remains the sole graph-diff calculator.

这仍是可复用 Rust storage contract 内部的能力：不新增 REST route、OpenAPI operation、TypeScript SDK method、Web control、operator transport 或 graph-diff implementation。`GraphDiff` 仍是唯一的 graph-diff calculator。

### Private Context Component-State Snapshot / 私有 Context Component-State Snapshot

`ContextComponentStateSnapshotAtCommitRepository::get_context_component_state_snapshot_at_commit` extends the same private replay boundary from one supplied component to the whole Context. It validates a target's complete normal first-parent history once, folds detailed component creation, update, and typed removal changes from root to target, and returns descriptor-only component states in stable component-identifier order. Each valid removal removes that component from later inventory; each remaining state retains its metadata, effective hash, creation commit, and last content-change commit; no returned value exposes a component body. The adapters never enumerate from the current component projection, so a later addition, revision, or removal cannot leak into an earlier inventory. Unknown scope, malformed, stale, repeated, or post-removal transitions, missing revision witnesses, merge ancestry, cycles, and cross-Context parents fail closed.

`ContextComponentStateSnapshotAtCommitRepository::get_context_component_state_snapshot_at_commit` 将同一私有 replay boundary 从一个外部提供的 component 扩展到整个 Context。它会一次性验证 target 的完整 normal first-parent history，再从 root 到 target 折叠带详情的 component creation、update 与 typed removal change，并按稳定的 component identifier 顺序返回仅含 descriptor 的 component state。每次合法 removal 都会使该 component 在较晚 inventory 中缺席；保留下来的 state 仍保留 metadata、有效 hash、creation commit 与最近 content-change commit；任何返回值都不会暴露 component body。adapter 绝不会从当前 component projection 枚举，因此较晚的新增、revision 或 removal 不会泄漏进较早的 inventory。unknown scope、损坏、陈旧、重复或 removal 后的 transition、缺失 revision witness、merge ancestry、cycle 与跨 Context parent 都会 fail closed。

This is still a reusable Rust storage contract only. It adds no REST/OpenAPI/SDK/Web surface, mutation capability, migration, operator transport, merge policy, release claim, or new graph-diff calculation; `GraphDiff` remains the sole graph-diff calculator.

这仍只是可复用 Rust storage contract。不新增 REST/OpenAPI/SDK/Web surface、mutation capability、migration、operator transport、merge policy、release 声明或新的 graph-diff calculation；`GraphDiff` 仍是唯一的 graph-diff calculator。

### Durable Context Commit Parent Scope / 持久化 Context Commit 父范围

Migration `0015_context_commit_parent_scope_integrity.sql` records the owning Context on every durable commit-parent link, backfills valid history from each child commit, and uses composite foreign keys for both child and parent endpoints. It keeps child deletion cascading to its links and parent deletion restricted while referenced. A malformed historical cross-Context edge fails the forward migration rather than being repaired or silently accepted; the replay resolver remains a separate fail-closed defense for deliberately isolated legacy-corruption tests. This changes neither public transport nor `GraphDiff`.

迁移 `0015_context_commit_parent_scope_integrity.sql` 会在每条持久化 commit-parent link 上记录所属 Context，从每个 child commit 回填合法历史，并为 child 与 parent 两端使用复合外键。它保持 child 删除时级联删除 link，并在 parent 仍被引用时限制删除。损坏的历史跨 Context edge 会使前向迁移失败，而不会被修复或静默接受；replay resolver 仍作为独立的 fail-closed 防线，用于刻意隔离的 legacy-corruption 测试。本迁移不改变 public transport 或 `GraphDiff`。

This is not a public mutation contract: the default public router, OpenAPI operation set, public SDK, public Web mutation controls, and operator transport do not expose component-body creation or retrieval. The explicitly composed protected local router provides one authenticated `Read` lifecycle-state route and one authenticated `Write` lifecycle-command route. A separate local-only Web editor reaches those routes only through same-origin BFF handlers, with memory-only bearer input, request-scoped idempotency, cookie-omitting browser fetches, and no credential persistence. Both paths reuse the guarded writer, durable authorization audit, operation-scoped rate limits, materialized normal first-parent replay, and one immutable graph snapshot per successor commit; the read fails closed unless descriptor, body, and graph node/edge facts agree. The local contract does not establish public promotion, release, production readiness, or a second graph-diff calculation; `GraphDiff` remains the sole calculator. Local disposable PostgreSQL verification is bounded non-production evidence and does not decide public-write or release promotion.

这不是 public mutation contract：默认 public router、OpenAPI operation 集、public SDK、public Web mutation control 与 operator transport 都不暴露 component-body creation 或 retrieval。显式组合的 protected local router 提供一条经认证的 `Read` lifecycle-state route 和一条经认证的 `Write` lifecycle-command route。独立的仅本地 Web editor 只能通过同源 BFF handler 访问这些 route，并使用仅内存 bearer input、request-scoped idempotency、显式省略 cookie 的浏览器 fetch，且不持久化 credential。两条路径均复用 guarded writer、持久 authorization audit、按 operation 划分的 rate limit、已 materialize 的 normal first-parent replay，以及每个 successor commit 的一个不可变 graph snapshot；除非 descriptor、body 与 graph node/edge 事实一致，read 会 fail closed。local contract 不等于 public promotion、release、production readiness 或第二个 graph-diff calculation；`GraphDiff` 仍是唯一 calculator。本地 disposable PostgreSQL 验证是有界的 non-production 证据，不决定 public-write 或 release promotion。

## 5. Workflow Engine

负责：

工作流

节点

执行

调度

依赖关系

未来支持可视化 Workflow 编辑器。

The private source-binding boundary now seals one complete provider-free `WorkflowDefinition` revision to one exact `ContextCommitSource` (`ContextId` plus immutable `CommitId`). `contextlab-workflow` owns the typed immutable binding record and preserves the full definition for replay; `contextlab-storage` owns the append-only `ContextWorkflowBindingRepository`, whose memory and PostgreSQL adapters accept a binding only when the exact Context commit already has a materialized graph snapshot. Identical writes replay, while a different source for the same Workflow identity/revision fails closed. Reads at a commit are exact and deterministically ordered; they never substitute the current branch head or resolve a Workflow by ID alone.

当前私有 source-binding boundary 会将一份完整的 provider-free `WorkflowDefinition` revision 封存到一个精确的 `ContextCommitSource`（`ContextId` 加不可变 `CommitId`）。`contextlab-workflow` 拥有类型化不可变 binding record，并保留完整 definition 供 replay；`contextlab-storage` 拥有 append-only `ContextWorkflowBindingRepository`，其 memory 与 PostgreSQL adapter 只有在精确 Context commit 已具备 materialized graph snapshot 时才接受 binding。相同写入会 replay，而同一 Workflow identity/revision 指向不同 source 时会 fail closed。按 commit 的读取是精确且确定性排序的；它绝不替换为当前 branch head，也不只凭 Workflow ID 解析 source。

This is a private domain/storage contract with a protected local read adapter, not Workflow execution transport. The existing read adapter authenticates and authorizes the canonical path `ContextId` before repository access, remains default-deny, preserves the same scope in every repository call, and exposes only the redacted exact-commit summary. It does not add a public route, OpenAPI/SDK method, Web mutation, provider call, branch merge policy, detach/rebind lifecycle, or a second graph-diff implementation. Any future Workflow execution or mutation transport requires a separate contract and gate. `GraphDiff::between` remains the sole graph-diff calculator.

这仍是带有 protected local read adapter 的私有 domain/storage contract，而不是 Workflow execution transport。现有 read adapter 会在 repository access 前对 canonical path `ContextId` 执行 authentication 与 authorization，保持 default-deny，在每次 repository call 中保留同一 scope，并且只暴露脱敏的 exact-commit summary。它不新增 public route、OpenAPI/SDK method、Web mutation、provider call、branch merge policy、detach/rebind lifecycle 或第二个 graph-diff implementation。未来 Workflow execution 或 mutation transport 必须另有独立 contract 与 gate。`GraphDiff::between` 仍是唯一的 graph-diff calculator。

---

## 6. Evaluation Engine

负责：

Benchmark

Dataset

A/B Test

Regression Test

Scoring

Metrics

自动生成评测报告。

`contextlab-evaluation` now owns the first framework-independent benchmark policy contract. Immutable benchmark cases distinguish an unspecified oracle from an exact structured output, including exact JSON null. Datasets and suites use stable typed identities, reject empty or duplicate membership, and normalize case, dataset, threshold, and metric ordering. A suite applies unique inclusive minimum/maximum metric thresholds to a `Scorecard` and returns deterministic `Passed`, `Regressed`, or `InsufficientData` decisions with per-metric evidence.

`contextlab-evaluation` 现拥有第一版与框架无关的 benchmark policy contract。不可变 benchmark case 会区分未指定 oracle 与精确 structured output，包括精确 JSON null。dataset 与 suite 使用稳定的类型化 identity，拒绝空或重复 membership，并规范化 case、dataset、threshold 与 metric 顺序。suite 会把 metric 唯一、包含边界的 minimum/maximum threshold 应用于 `Scorecard`，返回确定性的 `Passed`、`Regressed` 或 `InsufficientData` decision 以及逐 metric 证据。

Scorecards retain per-metric sample counts, normalize floating aggregation order, avoid large-finite summation overflow, and fail coverage closed when a run repeats one metric or when any run omits it. `EvaluationRun` rejects blank model identities, non-finite temperatures, and temperatures outside the inclusive `0.0..=2.0` range before storage; migration `0016_benchmark_definition_decision_evidence.sql` enforces the same database range. Validated benchmark, run, threshold, and scorecard aggregates are not directly deserialized around their constructors. The private evidence boundary persists immutable project-scoped dataset and suite definitions, exact project/Context/commit-bound runs, and deterministic decision evidence atomically through memory or PostgreSQL repositories. Decisions, their membership rows, metric rows, and seals are all keyed by the exact Context commit, so a repeated decision ID is isolated per commit. Deferred completeness checks seal every aggregate, later child inserts are rejected, and metric evidence stores its coverage fact so duplicate input remains replayable as `InsufficientData`. PostgreSQL timestamp normalization rebuilds `BenchmarkEvaluation` from the normalized run payloads before verifying the immutable evidence command. Storage validates scope, identity, membership, counts, and preserved policy output; `contextlab-evaluation` remains the only threshold and decision calculator.

Scorecard 会保留逐 metric sample count、规范化浮点聚合顺序、避免大有限值求和溢出，并在单个 run 重复 metric 或任一 run 缺失时 fail coverage closed。`EvaluationRun` 会在进入 storage 前拒绝空白 model identity、非有限 temperature，以及超出包含边界 `0.0..=2.0` 的 temperature；迁移 `0016_benchmark_definition_decision_evidence.sql` 也会施加相同的数据库范围。经过校验的 benchmark、run、threshold 与 scorecard aggregate 不允许绕过 constructor 直接反序列化。私有 evidence boundary 会经由 memory 或 PostgreSQL repository 原子持久化不可变的 project-scoped dataset/suite definition、精确绑定 project/Context/commit 的 run，以及确定性的 decision evidence。decision、其 membership row、metric row 与 seal 都以精确的 Context commit 为键，因此可在不同 commit 中隔离重复的 decision ID。延迟完整性检查会 seal 每个 aggregate，之后的 child insert 会被拒绝；metric evidence 会保存 coverage fact，因此重复输入仍可作为 `InsufficientData` 被 replay。PostgreSQL 的 timestamp normalization 会在验证不可变 evidence command 前，从规范化后的 run payload 重建 `BenchmarkEvaluation`。storage 只验证 scope、identity、membership、count 和已保留的 policy output；`contextlab-evaluation` 仍是唯一的 threshold 与 decision calculator。

The repository ports themselves are not public REST, OpenAPI, or public SDK contracts. A separate protected local inspection layer can read one sealed decision only at its exact project/Context/commit/decision scope after `ContextPermission::Read` authorization and a benchmark-specific rate-limit operation. Its non-public local SDK validates the full narrow projection and rejects raw case/oracle keys at every nesting level; the same-origin BFF forwards only a request-scoped Bearer token, omits cookies, and returns `Cache-Control: private, no-store`. The design-system Web inspector renders identifiers, comparability, coverage, metric outcomes, and sealed definition metadata through shared primitives. A storage-owned sealed-definition summary consumes the already-loaded exact decision, verifies its suite and dataset membership against the seal, then reads only immutable project-scoped definitions and exposes `definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }`. It orders thresholds by stable metric and datasets by identifier, and excludes cases, inputs, expected outputs, run payloads, measurements, and policy output. The summary adds no route or public contract. Fresh local storage/API/SDK/BFF/Web tests verify this narrow path; it does not add benchmark execution, a dashboard, public inspection, or any write path. PostgreSQL adapter tests for this earlier sealed-decision inspection path compile and remain ignored disposable-database tests; their runtime execution is unobserved while Docker is disabled.

`BenchmarkDecisionDiff` is a separate pure evaluation-domain comparison over sealed status and metric evidence. It requires equal immutable comparability fingerprints and does not call `BenchmarkEvaluation::from_runs` or recalculate thresholds. `BenchmarkEvidenceRepository::get_benchmark_decision_pair` reads both exact project/Context/commit/decision scopes under one memory lock or one PostgreSQL `REPEATABLE READ`, read-only transaction. The protected local diff route, non-public local SDK, same-origin BFF, and design-system comparison inspector only transport and render that domain result; they expose no raw cases, inputs, expected outputs, run payloads, or public REST/OpenAPI/public SDK contract. `GraphDiff::between` remains the sole graph-diff calculator.

`BenchmarkDecisionDiff` 是独立、纯粹的 evaluation-domain 比较：它只比较已 seal 的 status 与 metric evidence，要求不可变 comparability fingerprint 相同，既不调用 `BenchmarkEvaluation::from_runs`，也不重新计算 threshold。`BenchmarkEvidenceRepository::get_benchmark_decision_pair` 会在单个 memory lock 或单个 PostgreSQL `REPEATABLE READ`、只读 transaction 内读取两组精确的 project/Context/commit/decision scope。protected local diff route、非公开 local SDK、同源 BFF 与 design-system comparison inspector 只传输和呈现这一 domain result；它们不暴露 raw case、input、expected output、run payload，也不新增 public REST/OpenAPI/public SDK contract。`GraphDiff::between` 仍是唯一的 graph-diff calculator。

Private benchmark execution now remains inside the reusable Rust core. `BenchmarkExecutionPlan` loads only sealed suite membership and creates an ordered composite `(dataset_id, case_id)` schedule. A trusted injected evaluator receives that internal case value and returns finite, duplicate-metric-free measurements; no route, SDK, page, or caller provides measurements. `BenchmarkExecutionCohort` retains every case-to-run binding and derives each `EvaluationRunId` with UUIDv5 from the immutable decision UUID plus the composite case key. The storage service validates model/evaluator configuration before any evaluator call, returns an existing exact decision without re-executing, evaluates the cohort once through `BenchmarkSuite::evaluate_runs`, and makes one existing evidence-writer call. The resulting deterministic run identities are already retained by the sealed evidence aggregate, so this adds no second persistence contract, policy calculator, provider transport, or public write surface.

私有 benchmark execution 现仍位于可复用的 Rust core 内。`BenchmarkExecutionPlan` 只加载已 seal 的 suite membership，并创建有序的复合 `(dataset_id, case_id)` schedule。可信的注入式 evaluator 接收该内部 case value，并返回有限且不含重复 metric 的 measurement；route、SDK、页面和调用方都不能提供 measurement。`BenchmarkExecutionCohort` 保留每条 case-to-run binding，并使用不可变 decision UUID 加复合 case key 通过 UUIDv5 派生每个 `EvaluationRunId`。storage service 会在任何 evaluator call 前校验 model/evaluator configuration；精确 decision 已存在时直接返回而不重新执行；它只经 `BenchmarkSuite::evaluate_runs` 评测 cohort 一次，并只调用既有 evidence writer 一次。产生的确定性 run identity 已由 sealed evidence aggregate 保留，因此不新增第二套 persistence contract、policy calculator、provider transport 或 public write surface。

这些 repository port 本身不属于 public REST、OpenAPI 或 public SDK contract。独立的 protected local inspection 层只能在 `ContextPermission::Read` authorization 与 benchmark 专用 rate-limit operation 之后，按精确的 project/Context/commit/decision scope 读取一条已 seal 的 decision。非公开 local SDK 会校验完整且狭窄的 projection，并在任何嵌套层级拒绝 raw case/oracle key；同源 BFF 只转发 request-scoped Bearer token、忽略 cookie，并返回 `Cache-Control: private, no-store`。design-system Web inspector 通过共享 primitive 呈现 identity、comparability、coverage、metric outcome 与 sealed definition metadata。storage 所有的 sealed-definition summary 会消费已经加载的精确 decision、按其 seal 校验 suite 与 dataset membership，然后只读取不可变 project-scoped definition，并严格限定为 `definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }`。它按稳定 metric 排序 threshold、按 identifier 排序 dataset，并排除 case、input、expected output、run payload、measurement 与 policy output。该 summary 本身不新增 route 或 public contract。新鲜本地 storage/API/SDK/BFF/Web test 已验证这条狭窄路径；它不新增 benchmark execution、dashboard、public inspection 或任何 write path。此前 sealed-decision inspection path 的 PostgreSQL adapter test 已完成编译并登记为 ignored disposable-database test；Docker 关闭时其 runtime execution 仍未观测。

---

## 7. Memory Engine

负责：

长期记忆

短期记忆

重要度

时间线

遗忘策略

Memory Replay

---

## 8. Knowledge Engine

负责：

知识库

文档

Chunk

Embedding

引用关系

知识检索

---

## 9. Embedding Engine

统一管理：

Embedding Provider

向量索引

相似度搜索

重建

缓存

---

## 10. Context Graph

整个系统所有对象形成 Graph。

例如：

Workspace

↓

Project

↓

Experiment

↓

Context

↓

Memory

↓

Knowledge

↓

Prompt

↓

Evaluation

↓

Result

任何节点都能可视化。

---

## 11. Model Gateway

统一接入：

OpenAI

Anthropic

Google

DeepSeek

Qwen

Ollama

OpenRouter

未来新增 Provider 不影响业务层。

---

## 12. MCP Gateway

统一管理：

MCP Server

Tool Discovery

Tool Execution

权限

日志

状态

---

## 13. Plugin System

支持动态扩展：

LLM

Tool

Provider

Storage

Authentication

Importer

Exporter

Renderer

Workflow

Evaluator

任何新增能力均采用插件实现。

---

## 14. Search

统一全文检索：

Prompt

Memory

Knowledge

Conversation

Project

Evaluation

未来支持混合检索。

---

## 15. Storage

统一对象存储。

负责：

文件

图片

PDF

附件

缓存

版本文件

---

## 16. Scheduler

负责：

后台任务

Benchmark

Embedding

定时任务

同步

异步执行

---

## 17. Telemetry

负责：

性能监控

API 调用

耗时

Token

成本

错误率

资源占用

---

## 18. Audit Log

所有重要操作永久记录。

支持：

时间线

回放

审计

恢复

---

## 19. SDK / Client Boundary

负责：

REST Contract

TypeScript DTO

Client SDK

Route Construction

Error Model

Web Live Data Boundary

SDK 层只表达公开 API 合约，不承载业务逻辑。

未来 OpenAPI 生成器接入后，手写 SDK 合约测试仍作为兼容性基准。

---

# 四、数据流

```
用户

↓

编辑 Context

↓

Version Engine

↓

Diff Engine

↓

Evaluation Engine

↓

Context Graph

↓

Storage

↓

Search Index

↓

UI 实时更新
```

---

# 五、设计原则

所有模块：

单一职责。

低耦合。

高内聚。

接口稳定。

依赖倒置。

领域驱动。

禁止循环依赖。

禁止跨模块直接访问数据库。

所有模块只能通过公开接口通信。

---

# 六、未来规划

第一阶段：

Context 管理

Prompt 编辑

版本控制

Diff

Design System

基础评测

第二阶段：

Memory

Knowledge

Embedding

Graph

Workflow

第三阶段：

Plugin Marketplace

MCP

AI 自动优化

团队协作

云同步

第四阶段：

开放平台

SDK

生态系统

社区插件

第三方集成

最终形成完整的 AI Context Engineering Infrastructure。

## Private Sealed Benchmark Decision Run Details / 私有已封存 Benchmark Decision Run 明细

The admitted sealed-decision run-details increment is a private local read path. It starts from an already authorized `BenchmarkDecisionEvidence`, resolves exactly its ordered `run_ids` at their exact project/Context/commit/run scope, and exposes only the safe run summary required for inspection. A missing member, scope mismatch, or raw nested benchmark payload must fail closed; cases, inputs, expected outputs, model outputs, and evaluator diagnostics remain outside the projection.

本次准入的 sealed decision run-details 增量是一条私有 local read path。它从已经完成授权的 `BenchmarkDecisionEvidence` 开始，严格按照其中保存的有序 `run_ids`，在精确的 project/Context/commit/run scope 内解析，并且只暴露审阅所需的安全 run summary。缺失 member、scope mismatch 或 raw nested benchmark payload 都必须 fail closed；case、input、expected output、model output 与 evaluator diagnostic 仍不属于 projection。

The path adds no public REST, OpenAPI, or public SDK write contract and does not change `GraphDiff::between` or introduce another graph-diff calculator. At this increment's receipt time, the observed local evidence was `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, `pnpm check:web`, focused storage/API/local-SDK/Web tests, and compile-only coverage for its ignored PostgreSQL test; scoped strict Clippy was still blocked by the then-existing Rust 1.85 MSRV lint in `crates/auth/src/authorization.rs:320`. That Clippy statement is historical and is superseded by the later closure receipt under "Same-Origin Benchmark Web Workspace", where strict workspace Clippy and the locked Rust 1.85 workspace check passed. Runtime for this increment's ignored PostgreSQL run-details test and authenticated browser E2E remain unobserved.

该 path 不新增 public REST、OpenAPI 或 public SDK write contract，也不改变 `GraphDiff::between`，不会引入第二个 graph-diff calculator。在本增量形成回执时，已观测本地证据为 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`、`pnpm check:web`、聚焦 storage/API/local SDK/Web 测试，以及本增量 ignored PostgreSQL 测试的仅编译覆盖；范围化 strict Clippy 当时仍受 `crates/auth/src/authorization.rs:320` 中既有 Rust 1.85 MSRV lint 阻断。该 Clippy 说法属于历史状态，已由后文“Same-Origin Benchmark Web Workspace”的收束回执取代；后者记录了严格 workspace Clippy 与锁定 Rust 1.85 workspace check 通过。本增量 ignored PostgreSQL run-details test 的 runtime 与 authenticated browser E2E 仍未观测。

## Private Benchmark Decision Discovery / 私有 Benchmark Decision 发现

The private benchmark decision discovery path is an exact-scope, read-only composition over the
sealed evidence model. `BenchmarkDecisionDiscoveryRepository` is intentionally separate from
`BenchmarkEvidenceRepository`: its contract returns only safe sealed-decision summaries and never
hydrates raw `BenchmarkDecisionEvidence`. `BenchmarkDecisionDiscoveryService` consumes only that
discovery port, checks every returned row against project, Context, and commit identity, and orders
the summaries by `recorded_at DESC` followed by `decision_id ASC`.

私有 benchmark decision discovery path 是基于 sealed evidence model 的 exact-scope、只读
composition。`BenchmarkDecisionDiscoveryRepository` 有意与 `BenchmarkEvidenceRepository` 分离：
它的 contract 只返回安全的 sealed-decision summary，绝不 hydrate 原始
`BenchmarkDecisionEvidence`。`BenchmarkDecisionDiscoveryService` 只消费该 discovery port，
对每一条返回 row 校验 project、Context 与 commit identity，并按 `recorded_at DESC`、再按
`decision_id ASC` 排序 summary。

The protected local route is server-owned and remains outside the public REST/OpenAPI/public SDK
catalog. It reuses authentication, `ContextPermission::Read`, authorization audit, rate limiting,
private/no-store responses, and the existing `data -> presenter -> screen` Web boundary. The V1
response contains only stable decision/suite/dataset metadata, status, recording time, and run
count. Local SDK and compatibility Web parsers reject scope drift, duplicate identities, invalid
timestamps, unstable ordering, and raw case/input/output/measurement fields. No benchmark policy
or graph diff is calculated in Web; `GraphDiff::between` remains the sole graph-diff calculator.

该 protected local route 由 server 拥有，仍不属于 public REST/OpenAPI/public SDK catalog。它复用
authentication、`ContextPermission::Read`、authorization audit、rate limiting、private/no-store
response 与既有 `data -> presenter -> screen` Web boundary。V1 response 只包含稳定的
decision/suite/dataset metadata、status、recording time 与 run count。local SDK 与兼容 Web parser
都会拒绝 scope drift、duplicate identity、invalid timestamp、unstable ordering 以及 raw
case/input/output/measurement field。Web 不计算 benchmark policy 或 graph diff；`GraphDiff::between`
仍是唯一 graph-diff calculator。

The earlier cross-stack receipts predate the safe-summary and PostgreSQL aggregate-query hardening,
so they remain historical rather than fresh evidence for this contract. The later repository-wide
closure receipt under "Same-Origin Benchmark Web Workspace" supersedes the general focused and
cross-stack rerun requirement. It does not establish PostgreSQL runtime for this discovery aggregate
query or authenticated browser E2E; those remain unobserved, while remote CI, operator rehearsal,
release, and production evidence remain deferred. This protected local read path is progress
evidence and does not close those external completion criteria.

先前的 cross-stack 回执早于 safe-summary 与 PostgreSQL aggregate-query hardening，因此对本
contract 而言仍是历史记录，不能当作新鲜证据。后文“Same-Origin Benchmark Web Workspace”的
repository-wide 收束回执已取代一般性的聚焦与 cross-stack 重跑要求，但它不证明本 discovery
aggregate query 的 PostgreSQL runtime 或 authenticated browser E2E；这两项仍未观测，remote CI、
operator rehearsal、release 与 production evidence 仍为 deferred。本 protected local read path 是
进展证据，不关闭这些外部 completion criteria。

## Durable Benchmark Workspace Projection And Protected Read / 持久 Benchmark Workspace Projection 与受保护读取

Migration `0019_benchmark_workspace_projection_receipts.sql` adds the durable source for the
redacted `BenchmarkWorkspaceProjectionV1`. One receipt is immutable at exact project, Context,
Context commit, decision, and execution-cohort scope. Its ordered child rows retain every composite
`(dataset_id, case_id) -> run_id` link; foreign keys require the exact project dataset case and the
exact project/Context/commit/decision run. The receipt also carries the canonical decision-evidence
digest through a composite foreign key to `benchmark_decision_evidence` and requires the exact
decision seal. This persists provenance, not raw benchmark content. Cases, inputs, expected outputs,
model outputs, evaluator diagnostics, and provider payloads remain outside the projection tables.

Migration `0019_benchmark_workspace_projection_receipts.sql` 为脱敏的
`BenchmarkWorkspaceProjectionV1` 增加持久 source。每份 receipt 在精确 project、Context、Context
commit、decision 与 execution cohort scope 内不可变；其有序 child row 会保留每条复合
`(dataset_id, case_id) -> run_id` link。foreign key 要求精确的 project dataset case，以及精确的
project/Context/commit/decision run。receipt 还通过复合 foreign key 携带并绑定
`benchmark_decision_evidence` 中的 canonical decision-evidence digest，同时要求精确 decision seal。
该结构只持久化 provenance，不持久化 raw benchmark content；case、input、expected output、model
output、evaluator diagnostic 与 provider payload 均不进入 projection table。

Completeness is enforced by `DEFERRABLE INITIALLY DEFERRED` constraint triggers. Before a write can
commit, PostgreSQL proves a positive count, contiguous positions, exact suite dataset/case
membership, and exact decision run membership, then creates one automatic receipt seal. Receipt,
case-link, and seal rows reject update/delete, and a sealed receipt rejects later case-link inserts.
An incomplete transaction fails validation and leaves no receipt. The writer makes constraints
immediate before its final verification, reloads the stored source, and commits only if the complete
reconstructed `PersistBenchmarkWorkspaceProjectionV1` equals the submitted command.

完整性由 `DEFERRABLE INITIALLY DEFERRED` constraint trigger 强制执行。write 提交前，PostgreSQL 会
验证正数 count、连续 position、精确 suite dataset/case membership，以及精确 decision run
membership，随后自动创建唯一 receipt seal。receipt、case-link 与 seal row 均拒绝 update/delete，
已 seal receipt 也拒绝后续 case-link insert。不完整 transaction 会在校验时失败，且不留下 receipt。
writer 会在最终验证前把 constraint 切为 immediate，重新加载持久 source，并且只有完整重建的
`PersistBenchmarkWorkspaceProjectionV1` 与提交 command 相等时才 commit。

An existing cohort is not classified as `Replayed` by key equality. PostgreSQL reloads its seal,
ordered provenance, decision evidence, suite, datasets, and exact runs; rebuilds the execution plan
and receipt; and checks evidence digest, cohort, case/run identities, and the complete persistence
command before returning `Replayed`. A mismatch fails closed. Reads use one PostgreSQL transaction
set to both `REPEATABLE READ` and `READ ONLY`; the revised source and optional baseline source are
fully reconstructed in that snapshot before the existing safe projection or evaluation diff is
returned.

既有 cohort 不会只因 key 相等就被判为 `Replayed`。PostgreSQL 会重新加载 seal、有序 provenance、
decision evidence、suite、dataset 与精确 run，重建 execution plan 和 receipt，再校验 evidence
digest、cohort、case/run identity 与完整 persistence command；任意不一致都会 fail closed。read 会
使用同一个同时设为 `REPEATABLE READ` 与 `READ ONLY` 的 PostgreSQL transaction；在返回既有安全
projection 或 evaluation diff 前，会在该 snapshot 内完整重建 revised source 与可选 baseline
source。

The server-owned GET is
`/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}`.
`baseline_commit_id` and `baseline_cohort_id` are optional only as a pair, and unknown query keys are
rejected. The response schema is `contextlab.local-benchmark-workspace.v1`. Its protected-router
composition authenticates before checking the dedicated `BenchmarkWorkspaceRead` quota, then the
handler records a `ContextPermission::Read` RBAC/audit decision before touching the optional
`BenchmarkWorkspaceProjectionV1Reader`. All success and failure responses are `Cache-Control:
private, no-store`. Stable fail-closed mappings distinguish invalid requests (`400`), missing exact
receipts (`404`), comparison/source conflicts (`409`), absent composition or shared auth/rate/RBAC
dependencies (`401`/`403`/`429`/`503`), and repository failure (`500`) without exposing internals.

server 所有的 GET 为
`/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}`。
`baseline_commit_id` 与 `baseline_cohort_id` 只能成对省略或提供，unknown query key 会被拒绝。
response schema 为 `contextlab.local-benchmark-workspace.v1`。protected router 会先 authentication，
再检查独立的 `BenchmarkWorkspaceRead` quota；handler 随后记录 `ContextPermission::Read` RBAC/audit
decision，最后才访问 optional `BenchmarkWorkspaceProjectionV1Reader`。所有成功和失败 response 都
带有 `Cache-Control: private, no-store`。稳定的 fail-closed mapping 会区分 invalid request（`400`）、
精确 receipt 缺失（`404`）、comparison/source conflict（`409`）、composition 或共享 auth/rate/RBAC
依赖缺失（`401`/`403`/`429`/`503`），以及不暴露内部细节的 repository failure（`500`）。

The strict client and parser live only in `packages/local-sdk`. They send a request-scoped Bearer
token with cookies and browser credentials omitted; require exact echoed project, Context, commit,
cohort, and optional baseline scope; and reject unknown keys, raw fields, schema drift, unstable or
duplicate order, count/coverage inconsistency, and receipt/diff scope drift. The public router,
checked-in OpenAPI, and `packages/ts-sdk` remain unchanged. Benchmark projection and evaluation-diff
code do not introduce graph semantics; `GraphDiff::between` remains the sole graph-diff calculator.

严格 client 与 parser 只位于 `packages/local-sdk`。它们只发送 request-scoped Bearer token，省略
cookie 与 browser credential；要求回显的 project、Context、commit、cohort 与可选 baseline scope
完全一致；并拒绝 unknown key、raw field、schema drift、不稳定或重复排序、count/coverage
inconsistency，以及 receipt/diff scope drift。public router、已检入 OpenAPI 与 `packages/ts-sdk` 均未
改变。Benchmark projection 与 evaluation-diff code 不引入 graph semantics；`GraphDiff::between`
仍是唯一 graph-diff calculator。

### Execution-Owned Projection Materialization / 执行服务拥有的投影物化

`BenchmarkExecutionService` is now the normal private producer for
`PersistBenchmarkWorkspaceProjectionV1`. After either newly created evidence or an exact evidence
replay, it reloads the sealed suite, ordered datasets, and every exact sealed run; rebuilds the
execution plan; maps each `(dataset_id, case_id)` to its domain-owned UUID v5 run identity under the
decision namespace; reconstructs the receipt; and delegates persistence to the existing
`BenchmarkWorkspaceProjectionV1Writer`. The in-memory repository and
`PostgresContextGraphRepository` therefore share one producer contract and one projection algorithm.

`BenchmarkExecutionService` 现在是 `PersistBenchmarkWorkspaceProjectionV1` 的常规私有 producer。
无论 evidence 是新建还是精确 replay，service 都会重新加载已封存 suite、有序 dataset 与每条精确
sealed run，重建 execution plan，以 decision namespace 下由 domain 拥有的 UUID v5 identity 映射每个
`(dataset_id, case_id)`，再重建 receipt，并委托既有 `BenchmarkWorkspaceProjectionV1Writer` 持久化。
因此 in-memory repository 与 `PostgresContextGraphRepository` 共享同一 producer contract 和同一
projection algorithm。

Evidence and projection writes remain two fail-closed persistence steps. If the projection write
fails after evidence is sealed, the next exact execution replay reconstructs the missing projection
from stored definitions and runs without invoking the evaluator again. Replay identity compares
timestamps at PostgreSQL's microsecond precision and uses the sealed run timestamp when rebuilding
the receipt, so sub-microsecond adapter normalization does not create a false mismatch. The service
returns both the evidence execution disposition and the exact project/Context/commit/cohort
projection scope plus its independent `Created` or `Replayed` disposition.

evidence write 与 projection write 仍是两个 fail-closed persistence step。若 evidence 已封存而
projection write 失败，下一次精确 execution replay 会从已存 definition 与 run 重建缺失 projection，
且不再次调用 evaluator。replay identity 按 PostgreSQL 微秒精度比较 timestamp，并在重建 receipt 时
使用 sealed run timestamp，因此 adapter 的亚微秒 normalization 不会制造错误 mismatch。service 会
同时返回 evidence execution disposition，以及精确 project/Context/commit/cohort projection scope 和
其独立的 `Created` 或 `Replayed` disposition。

### Same-Origin Benchmark Web Workspace / 同源 Benchmark Web 工作台

The real local Web path is now protected Axum GET -> non-public local SDK -> same-origin BFF -> Web
data -> presenter -> screen, composed by the inspector. The BFF accepts one exact revised scope and
an optional baseline only as an exact commit/cohort pair. Bearer input remains in request memory;
both browser-to-BFF and BFF-to-Axum fetches use `credentials: "omit"` and no-store behavior, and the
BFF returns `Cache-Control: private, no-store` for success and failure. The Web data adapter parses
the BFF response again with the strict local-SDK parser, requires exact scope echo, and recursively
freezes the accepted payload.

真实 local Web path 现为 protected Axum GET -> 非公开 local SDK -> 同源 BFF -> Web data ->
presenter -> screen，并由 inspector 完成组合。BFF 接受一个精确 revised scope；可选 baseline 只能以
精确 commit/cohort 参数对提供。Bearer input 只保留在 request memory；browser-to-BFF 与
BFF-to-Axum fetch 均使用 `credentials: "omit"` 和 no-store 行为，BFF 对成功与失败都返回
`Cache-Control: private, no-store`。Web data adapter 会再次使用严格 local-SDK parser 解析 BFF
response，要求精确 scope echo，并递归冻结通过校验的 payload。

The presenter and screen display only the server-owned redacted scorecard, regression status,
provenance, and optional evaluation diff. They never recalculate thresholds, policy, regression, or
diffs. The inspector exposes five explicit states (`loading`, `error`, `empty`, `available`, and
`unavailable`), uses shared `@contextlab/ui` primitives with bilingual labels and accessibility
names, and preserves the existing decision-evidence and decision-diff inspectors below the new
workspace. `ContextWorkspaceScreen` keys this inspector by project, Context, and the complete
candidate-commit identity list, forcing memory-only credentials and scope-local state to remount
when any owning scope changes. Public OpenAPI and the public SDK remain unchanged.

presenter 与 screen 只显示服务端拥有的脱敏 scorecard、regression status、provenance 与可选
evaluation diff，绝不重新计算 threshold、policy、regression 或 diff。inspector 暴露五种明确状态
（`loading`、`error`、`empty`、`available`、`unavailable`），使用共享 `@contextlab/ui` primitive、
双语 label 与 accessibility name，并在新 workspace 下保留既有 decision-evidence 与 decision-diff
inspector。`ContextWorkspaceScreen` 以 project、Context 和完整 candidate-commit identity list 为该
inspector 生成 key；任一所属 scope 改变时都会 remount，从而清除仅内存 credential 与局部 scope
state。public OpenAPI 与 public SDK 保持不变。

Fresh local closure evidence passed focused evaluation tests `15/15`, focused storage tests `38/38`
after the timestamp-normalization regression, formatting, strict workspace Clippy, the Rust `1.85`
workspace check, and full workspace tests with API `162` and storage `166 passed, 38 ignored`.
`pnpm check:web` passed public SDK `14`, local SDK `59`, Web `138`, and the production build; the
static contract verifiers and updated desktop/mobile Playwright smoke also passed with no horizontal
overflow or console errors. One exact ignored producer runtime test passed (`1 passed`) against a
disposable loopback PostgreSQL `16.14` service, and the server was stopped afterward.

新鲜本地收束证据已通过 evaluation 聚焦测试 `15/15`、timestamp-normalization regression 后的 storage
聚焦测试 `38/38`、格式检查、严格 workspace Clippy、Rust `1.85` workspace check，以及 workspace
全量测试（API `162`；storage `166 passed, 38 ignored`）。`pnpm check:web` 通过 public SDK `14`、
local SDK `59`、Web `138` 与 production build；静态 contract verifier 和更新后的桌面/移动端
Playwright smoke 也已通过，且无横向 overflow 或 console error。一项精确选择的 ignored producer
runtime test 在 disposable、loopback-only PostgreSQL `16.14` service 上以 `1 passed` 通过，随后
server 已停止。

That disposable database used `SQL_ASCII`; this bounded producer receipt is not production
encoding/Unicode readiness, migration safety, or operational evidence. Authenticated
browser-to-BFF-to-Axum runtime and Git change-set evidence remain unobserved. Remote CI, operator
rehearsal, public promotion, release, and production behavior remain deferred.

该 disposable database 使用 `SQL_ASCII`；这份有界 producer 回执不能证明 production encoding/
Unicode readiness、migration safety 或 operational readiness。authenticated
browser-to-BFF-to-Axum runtime 与 Git change-set evidence 仍未观测；remote CI、operator rehearsal、
public promotion、release 与 production behavior 仍为 deferred。

### Private Benchmark Definition Authoring / 私有 Benchmark 定义创作

Private benchmark authoring now has a reusable Rust command and storage port. The command validates
schema version 1, complete suite membership, deterministic dataset ordering, an exact immutable
project/context/commit source, an equal expected branch head, and replay metadata. Memory and
PostgreSQL persist immutable project-scoped dataset/suite definitions plus an append-only
Context-commit binding. PostgreSQL locks the project, Context, source commit, write membership or
trusted group role, idempotency advisory key, and branch row in one transaction before inserting
anything; the branch is not advanced. Identical request digests replay the binding, while changed
digests, binding identities, suite membership, or definitions fail closed without partial writes.
The adapter normalizes capture timestamps to PostgreSQL microseconds, matching existing evidence
replay semantics. Migration 0020 uses composite foreign keys for project/Context, Context/commit,
and project/suite, issuer-scoped principals, branch-scoped idempotency, schema checks, and
append-only triggers. This is a private core/storage contract; public REST/OpenAPI/public SDK
writes, Web mutation, provider calls, Context commit mutation, and release readiness remain out of
scope. GraphDiff::between remains the sole graph-diff calculator.

私有 benchmark authoring 现已具备可复用 Rust command 与 storage port。command 校验 schema version 1、
完整 suite membership、确定性 dataset ordering、精确不可变 project/context/commit source、
相等的 expected branch head 与 replay metadata。Memory 与 PostgreSQL 原子持久化 project-scoped
不可变 dataset/suite definition 以及 append-only Context-commit binding。PostgreSQL 在插入任何
内容前，于同一 transaction 内锁定 project、Context、source commit、write membership 或可信
group role、idempotency advisory key 与 branch row；branch 本身不推进。相同 request digest
replay binding，digest、binding identity、suite membership 或 definition 改变则 fail closed 且
不产生部分写入。adapter 将 capture timestamp 规范化为 PostgreSQL 微秒精度，与既有 evidence
replay 语义一致。迁移 0020 使用 project/Context、Context/commit、project/suite 复合外键、
issuer-scoped principal、branch-scoped idempotency、schema check 与 append-only trigger。本
增量是 private core/storage contract；public REST/OpenAPI/public SDK write、Web mutation、
provider call、Context commit mutation 与 release readiness 均不在范围内。GraphDiff::between
仍是唯一 graph-diff calculator。

Fresh evidence for this wave is focused authoring 3/3, migration contract 6/6, storage workspace
tests 166 passed and 38 ignored, strict storage Clippy, and an isolated native PostgreSQL 16.14
UTF-8 loopback run of all 26 named disposable storage tests (26/26 passed). The cluster and
installed PostgreSQL service were stopped afterward. This is local non-production evidence;
authenticated browser-to-BFF-to-Axum, Git, remote CI, operator rehearsal, release, public
promotion, and production evidence remain unobserved or deferred.

本波新鲜证据包括 authoring 聚焦测试 3/3、migration contract 6/6、storage workspace
166 passed and 38 ignored、严格 storage Clippy，以及 native PostgreSQL 16.14 UTF-8 loopback
disposable run 的 26 个指定 storage test 全部通过（26/26 passed）。cluster 与安装的
PostgreSQL service 随后均已停止。这是本地非生产证据；authenticated browser-to-BFF-to-Axum、
Git、remote CI、operator rehearsal、release、public promotion 与 production evidence 仍为
unobserved 或 deferred。

### Private Benchmark Definition Authoring / 私有 Benchmark 定义创作

The private authoring vertical slice now crosses the reusable Rust/storage core, the explicitly
composed protected local Axum route, the non-public local SDK, the same-origin Web BFF, and the
shared design-system editor. The canonical write path is
`POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`.
It accepts one schema-1 immutable dataset/suite definition and binds it to one exact Context commit;
the server owns authorization, audit, the dedicated write quota, idempotency, branch-head equality,
and atomic persistence. The response is a redacted created/replayed receipt; raw cases and evaluator
payloads do not cross the transport boundary.

私有 authoring vertical slice 现已贯通可复用 Rust/storage core、显式组合的 protected local Axum route、
非公开 local SDK、同源 Web BFF 与共享 design-system editor。canonical write path 为
`POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`。
它接收一条 schema-1 不可变 dataset/suite definition，并绑定到一条精确 Context commit；authorization、
audit、专用 write quota、idempotency、branch-head equality 与原子持久化均由服务端负责。response 是脱敏的
created/replayed receipt；raw case 与 evaluator payload 不越过 transport boundary。

The browser stores the Bearer token only in request memory, omits cookies, uses `credentials: "omit"`,
and receives `private, no-store` responses. The editor uses `data -> presenter -> screen -> editor`,
shared primitives, bilingual state copy, and fail-closed validation; it does not implement storage,
authorization, benchmark policy, or diff logic. The former context-only BFF route is retired with a
`410 benchmark_definition_route_gone` response and never contacts an upstream service. The public
router, checked-in OpenAPI, public SDK, and `GraphDiff::between` remain unchanged.

浏览器只在请求内存中保存 Bearer token，不携带 cookie，使用 `credentials: "omit"`，并接收 `private, no-store`
response。editor 遵循 `data -> presenter -> screen -> editor`，使用共享 primitive、双语 state copy 与 fail-closed
validation；它不实现 storage、authorization、benchmark policy 或 diff logic。旧的 context-only BFF route 已退役，
返回 `410 benchmark_definition_route_gone`，绝不触达 upstream service。public router、已检入 OpenAPI、public SDK
与 `GraphDiff::between` 保持不变。

This is a private local development capability, not a public-write or production-readiness decision.
Local test receipts are recorded in the roadmap and authoring plan; authenticated browser runtime,
Git binding, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or
`deferred`.

这是 private local development capability，不构成 public-write 或 production-readiness decision。本地 test receipt
记录在 roadmap 与 authoring plan 中；authenticated browser runtime、Git binding、remote CI、operator rehearsal、
release 与 production evidence 仍为 `unobserved` 或 `deferred`。

### Private Benchmark Binding Inspection / 私有 Benchmark 绑定检查

The exact binding read path is a protected local GET at
`/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`.
It reuses `BenchmarkDefinitionBindingRepository`, converts complete persisted bindings into a
redacted `BenchmarkDefinitionBindingSummary`, and preserves storage ordering without exposing case
payloads, expected outputs, thresholds, request digests, or internal traces. Authentication precedes
the dedicated `BenchmarkDefinitionBindingRead` quota; the handler then records Context read
authorization/audit and applies `private, no-store`.

精确 binding read path 是 protected local GET：
`/api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`。
它复用 `BenchmarkDefinitionBindingRepository`，将完整持久 binding 转换为脱敏的
`BenchmarkDefinitionBindingSummary`，保持 storage order，但不暴露 case payload、expected output、threshold、
request digest 或 internal trace。authentication 先于专用 `BenchmarkDefinitionBindingRead` quota；handler 随后
记录 Context read authorization/audit，并应用 `private, no-store`。

The local SDK, same-origin BFF, and Web inspector use the explicit inspection schema
`contextlab.local-benchmark-definition-binding-inspection.v1`. The Web selection stores only the
exact immutable binding ID and commit scope; it does not implement latest resolution, benchmark
policy, or graph-diff logic. Public OpenAPI/public SDK remain unchanged, and `GraphDiff::between`
remains the sole graph-diff calculator.

local SDK、同源 BFF 与 Web inspector 使用显式 inspection schema
`contextlab.local-benchmark-definition-binding-inspection.v1`。Web selection 只保存精确不可变 binding ID 与
commit scope；不实现 latest resolution、benchmark policy 或 graph-diff logic。public OpenAPI/public SDK 保持不变，
`GraphDiff::between` 仍是唯一 graph-diff calculator。
The private benchmark boundary now includes provider-free execution selection. It consumes one already
selected immutable `BenchmarkDefinitionBinding`, reconstructs the existing deterministic
`BenchmarkExecutionPlan` from its exact suite and dataset definitions, and returns only stable scope,
definition identity, membership, and case-count facts to the execution layer. It does not accept a
mutable latest binding, call a provider/evaluator, persist evidence, or cross a REST/OpenAPI/SDK/Web
transport boundary. `GraphDiff::between` remains the sole graph-diff calculator.

私有 benchmark 边界现已包含 provider-free execution selection。它消费一条已经选定的不可变
`BenchmarkDefinitionBinding`，从其精确 suite 与 dataset definition 重建现有确定性的
`BenchmarkExecutionPlan`，并只向 execution layer 返回稳定 scope、definition identity、membership 与
case-count fact。它不接受 mutable latest binding，不调用 provider/evaluator，不持久化 evidence，也不跨越
REST/OpenAPI/SDK/Web transport boundary。`GraphDiff::between` 仍是唯一 graph-diff calculator。

## Typed Branch-Head Read Boundary / Typed Branch-Head 读取边界

`contextlab-storage::ContextBranchRepository` is a private, read-only repository port for the
durable `context_branches` fact. `ContextBranchHead` binds a typed `ContextId` and `BranchName` to
an optional `CommitId` and a monotonic revision. The optional head is intentional: PostgreSQL can
persist an unborn branch row with `head_commit_id = NULL`, so the Memory adapter must preserve the
same state shape rather than silently dropping the branch.

`contextlab-storage::ContextBranchRepository` 是 durable `context_branches` fact 的 private、read-only repository
port。`ContextBranchHead` 将 typed `ContextId` 与 `BranchName` 绑定到可空 `CommitId` 和单调 revision。可空 head 是
有意设计：PostgreSQL 可以持久化 `head_commit_id = NULL` 的 unborn branch row，因此 Memory adapter 必须保留相同
state shape，不能静默丢弃该 branch。

The repository has two deliberately different read paths. Listing validates and deterministically
sorts every rehydrated typed row. Exact lookup first validates Context ownership, then reads only
the requested `(ContextId, BranchName)` row so an unrelated malformed or overflowing row cannot
poison a valid read. Stored signed revisions, invalid branch names, and heads outside Context scope
fail closed. Branch-head CAS, revision increments, idempotency, authorization, and audit remain in
the existing guarded writer; this port does not create, rename, merge, or delete branches.

repository 有两个刻意不同的 read path。list 会验证并对所有 rehydrated typed row 做确定性排序；exact lookup 先验证
Context ownership，再只读取请求的 `(ContextId, BranchName)` row，因此无关 malformed 或 overflowing row 不会污染
有效读取。stored signed revision、invalid branch name 与越出 Context scope 的 head 均 fail closed。Branch-head CAS、
revision increment、idempotency、authorization 与 audit 仍由既有 guarded writer 负责；本 port 不创建、重命名、合并
或删除 branch。

## Commit-Associated ContextGraph Snapshots / Commit 关联 ContextGraph Snapshot

`contextlab-storage` owns the immutable `CommitGraphSnapshot` fact and its typed
`CommitGraphSnapshotScope(ProjectId, ContextId, CommitId)`. The repository port has two distinct
ownership operations: `project_id_for_context` derives the project from durable Context ownership for
a new guarded commit, while `scope_for_context_commit` requires an existing Context commit before a
version-backed read. Both Memory and PostgreSQL adapters validate the same scope and schema-V1
semantics; V1 serialization flattens the scope to preserve the established response shape.

`contextlab-storage` 负责不可变的 `CommitGraphSnapshot` fact 及 typed
`CommitGraphSnapshotScope(ProjectId, ContextId, CommitId)`。repository port 明确区分两种 ownership operation：
`project_id_for_context` 在新 guarded commit 时从 durable Context ownership 推导 project，
`scope_for_context_commit` 则要求 Context commit 已存在后才能进行 version-backed read。Memory 与 PostgreSQL adapter
共享相同的 scope 与 schema-V1 validation；V1 serialization flatten scope，以保持既有 response shape。

The API is an adapter only. It parses typed identifiers, resolves exact scopes server-side, loads
immutable snapshots, and calls `GraphDiff::between`; no presentation layer or second algorithm compares
graphs. The route path, query, public OpenAPI/SDK surface, RBAC/write guards, idempotency, branch-head
CAS, and Web mutation boundary are unchanged.

API 仅是 adapter：解析 typed identifier，在服务端解析 exact scope，读取不可变 snapshot，并调用 `GraphDiff::between`；
没有 presentation layer 或第二套 algorithm 比较 graph。route path、query、public OpenAPI/SDK surface、RBAC/write guard、
idempotency、branch-head CAS 与 Web mutation boundary 均未改变。

## Private Merge-Base and Ancestry Conflict Contract / 私有 Merge-Base 与 Ancestry Conflict 契约

`contextlab-versioning` now owns the framework-independent `CommitGraph` validator and
`MergePlan::resolve`. The validator rejects duplicate commits, duplicate parents, missing
parents, cross-Context parent edges, disconnected nodes outside the graph Context, and cycles.
It produces deterministic ancestry-only outcomes for no-op, fast-forward, one-base three-way,
no-common-ancestor, and ambiguous maximal-base histories. `CrossContextTips` remains a defensive
error classification, while validated graphs themselves have one Context scope.

`contextlab-versioning` 现负责 framework-independent 的 `CommitGraph` validator 与
`MergePlan::resolve`。validator 会拒绝 duplicate commit、duplicate parent、missing parent、跨 Context parent edge、
不属于 graph Context 的断开节点与 cycle，并以确定性结果区分 no-op、fast-forward、single-base three-way、
no-common-ancestor 与 ambiguous maximal-base history。`CrossContextTips` 保留为防御性 error classification；validated
graph 本身只允许一个 Context scope。

This is an ancestry contract only. It does not resolve component/content conflicts, persist a
merge, mutate a branch, expose REST/OpenAPI/SDK/Web/CLI/Desktop transport, or calculate a graph
diff. `GraphDiff::between` remains the sole graph-diff calculator. The local receipt is limited to
focused versioning tests and static/local workspace verification; PostgreSQL/Docker runtime,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
unobserved or deferred.

本契约只处理 ancestry，不解决 component/content conflict，不持久化 merge、不修改 branch、不暴露
REST/OpenAPI/SDK/Web/CLI/Desktop transport，也不计算 graph diff。`GraphDiff::between` 仍是唯一 graph-diff calculator。
本地回执仅覆盖 focused versioning tests 与静态/local workspace verification；PostgreSQL/Docker runtime、authenticated
browser、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。

## Private ContextGraph Three-Way Conflict Classification / 私有 ContextGraph 三路冲突分类

`contextlab-diff-engine` now owns a read-only `GraphMergeConflictClassifier`. Its
`GraphSnapshotRef` binds each graph to `(ProjectId, ContextId, CommitId)`, and classification
accepts only a matching `MergePlan::ThreeWay`. The classifier calls the existing
`GraphDiff::between` once for each branch from the base and returns deterministic Clean,
Equivalent, or Conflict results for node and edge changes. It never creates a merged graph or
mutates a branch.

`contextlab-diff-engine` 现负责只读的 `GraphMergeConflictClassifier`。其 `GraphSnapshotRef` 将每个 graph 绑定到
`(ProjectId, ContextId, CommitId)`，classification 只接受匹配的 `MergePlan::ThreeWay`。classifier 对每个 branch
从 base 调用一次既有 `GraphDiff::between`，并为 node 与 edge change 返回确定性的 Clean、Equivalent 或 Conflict 结果。
它不会创建 merged graph，也不会修改 branch。

This is a private pure-Rust boundary. No persistence, merge writer, rollback, REST/OpenAPI/SDK/Web
mutation, provider, Docker/PostgreSQL runtime, or second graph-diff calculator was added. Fresh
workspace Rust, format, strict offline Clippy, locked Rust `1.85.0`, Web, and static singularity
checks passed; PostgreSQL/Docker runtime, browser, Git, remote CI, operator rehearsal, release, and
production remain unobserved or deferred.

这是 private pure-Rust boundary。不新增 persistence、merge writer、rollback、REST/OpenAPI/SDK/Web mutation、provider、
Docker/PostgreSQL runtime 或第二个 graph-diff calculator。workspace Rust、format、strict offline Clippy、锁定 Rust
`1.85.0`、Web 与 static singularity checks 均新鲜通过；PostgreSQL/Docker runtime、browser、Git、remote CI、operator
rehearsal、release 与 production 仍为 unobserved 或 deferred。

## Private Persisted ContextGraph Merge Review / 私有持久化 ContextGraph Merge Review

`contextlab-storage` now owns the private `ContextMergeInputScope` and
`PersistedContextGraphMergeReviewService`. The scope binds one project/Context to distinct
base/left/right commit identities. The service rejects non-three-way and plan-identity drift
before repository reads, reads exact persisted snapshot scopes, fails closed for missing or
returned-scope-drifted snapshots, and delegates classification only to the existing
`GraphMergeConflictClassifier`. The classifier remains the only path that calls
`GraphDiff::between`; this storage adapter does not calculate, resolve, or write a merge.

`contextlab-storage` 现负责 private `ContextMergeInputScope` 与
`PersistedContextGraphMergeReviewService`。scope 将一个 project/Context 绑定到不同的 base/left/right commit identity。
service 在 repository read 前拒绝 non-three-way 与 plan-identity drift，读取 exact persisted snapshot scope，遇到 missing
或 returned-scope-drifted snapshot 时 fail closed，并只委托既有 `GraphMergeConflictClassifier`。classifier 仍是唯一调用
`GraphDiff::between` 的路径；该 storage adapter 不计算、不 resolve 也不写入 merge。

The focused Memory contract has five passing tests for exact three-side delegation, missing-side
failure, duplicate commit rejection, repository scope drift, and non-three-way rejection. The
PostgreSQL adapter currently exposes the same single-snapshot repository port, so the service
performs three separate exact reads rather than one atomic batch transaction. Cross-read
consistency and a future transactional batch port are deferred. No merge writer, migration,
transport, OpenAPI/SDK route, Web mutation, secret, provider, or production claim was added.

focused Memory contract 共 5 项通过，覆盖 exact three-side delegation、missing-side failure、duplicate commit rejection、
repository scope drift 与 non-three-way rejection。PostgreSQL adapter 当前暴露相同的 single-snapshot repository port，
因此 service 执行三次独立 exact read，而不是一次 atomic batch transaction。跨 read consistency 与未来 transactional
batch port deferred。未新增 merge writer、migration、transport、OpenAPI/SDK route、Web mutation、secret、provider 或
production claim。

## Private Versioned ContextGraph Merge Review / 私有版本化 ContextGraph Merge Review

`contextlab-diff-engine` now owns the private V1 application contract for a three-way graph review.
`VersionedContextGraphSnapshotV1` binds an owned graph to the existing
`VersionedContextScopeV1`; the request and projection carry an explicit schema version, the
matching `MergePlan`, exact base/left/right scopes, and the deterministic classification. The
versioned service rejects nil or duplicate snapshot identities and delegates only to
`GraphMergeConflictClassifier`. It never calls `GraphDiff::between` directly and never creates a
merged graph.

`contextlab-diff-engine` 现负责 private V1 three-way graph review application contract。
`VersionedContextGraphSnapshotV1` 将 owned graph 绑定到既有 `VersionedContextScopeV1`；request 与 projection 携带显式
schema version、匹配的 `MergePlan`、exact base/left/right scope 与确定性的 classification。versioned service 拒绝 nil 或
duplicate snapshot identity，并且只委托 `GraphMergeConflictClassifier`。它不会直接调用 `GraphDiff::between`，也不会创建
merged graph。

`contextlab-storage::PersistedContextGraphMergeReviewService` is now a thin adapter: it converts
the three exact persisted reads into the V1 request and returns the existing classification shape
without recalculating. The storage repository still performs three separate exact reads; atomic
batch consistency remains deferred. Focused versioned review tests passed `4`, storage review tests
passed `5`, workspace Rust passed `186` with `39 ignored`, strict offline Clippy, locked Rust
`1.85.0`, formatting, Web `15/92/185` plus production build, and static singularity `count=1`
all passed. No public transport, write, migration, Web mutation, secret, provider, or production
claim was added.

`contextlab-storage::PersistedContextGraphMergeReviewService` 现是薄 adapter：它将三次 exact persisted read 转换为 V1
request，并在不重新计算的情况下返回既有 classification shape。storage repository 仍执行三次独立 exact read；atomic
batch consistency 继续 deferred。focused versioned review `4` 项通过、storage review `5` 项通过、workspace Rust `186`
项通过且 `39 ignored`，strict offline Clippy、锁定 Rust `1.85.0`、formatting、Web `15/92/185` 与 production build，以及
static singularity `count=1` 均通过。未新增 public transport、write、migration、Web mutation、secret、provider 或
production claim。

## Exact Context Commit DAG Ownership / Exact Context Commit DAG 归属

`contextlab-storage::ContextCommitGraphRepository` is the reusable boundary for loading one
complete exact-Context commit DAG. Memory and PostgreSQL adapters convert persisted commit and
parent identities into `contextlab-versioning::CommitGraphNode` values and fail closed on invalid,
missing, duplicate, cross-Context, disconnected, or cyclic history. `WorkspaceDataRepository` and
`AppState` delegate this port internally; no route or transport is implied.

`contextlab-storage::ContextCommitGraphRepository` 是加载单一 exact-Context commit DAG 的可复用 boundary。Memory 与 PostgreSQL adapter
将持久化 commit/parent identity 转换为 `contextlab-versioning::CommitGraphNode`，并对 invalid、missing、duplicate、cross-Context、
disconnected 或 cyclic history fail closed。`WorkspaceDataRepository` 与 `AppState` 仅在内部 delegation；这不意味着新增 route 或 transport。

`PersistedContextGraphMergeReviewService::review_server_owned` consumes a typed Context plus two
branch tips, resolves `MergePlan` from the server-owned DAG, and reuses the existing exact
snapshot batch and versioned classifier. Caller-supplied review remains a compatibility method
until a separate private transport decision. `GraphDiff::between` remains the sole graph-diff
calculator; no public API/SDK/Web write, merge writer, or production readiness is claimed.

`PersistedContextGraphMergeReviewService::review_server_owned` 接受 typed Context 与两个 branch tip，从 server-owned DAG 解析
`MergePlan`，并复用既有 exact snapshot batch 与 versioned classifier。caller-supplied review 在单独 private transport 决策前作为兼容
method 保留。`GraphDiff::between` 仍是唯一 graph-diff calculator；不声称新增 public API/SDK/Web write、merge writer 或 production readiness。

## Exact-Commit Lifecycle Witness for GraphDiff / GraphDiff 的 Exact-Commit Lifecycle Witness

The private `PersistedContextGraphDiffReviewService` now requires a complete
`ContextLifecycleReadFacts` witness for each exact commit before comparison. The witness
validates the requested project/Context/commit scope across component content and provenance,
graph nodes and edges, replay state, Context identity, and commit identity. Missing facts,
mixed scope, or inconsistent lifecycle state fail closed and are mapped by the API adapter to a
safe unavailable response; the adapter does not leak repository or lifecycle details.

私有 `PersistedContextGraphDiffReviewService` 现在要求每个 exact commit 在比较前提供完整的
`ContextLifecycleReadFacts` witness。该 witness 会对 component content 与 provenance、graph nodes 与 edges、replay state、
Context identity 以及 commit identity 的 project/Context/commit scope 进行校验。缺失 facts、混合 scope 或不一致的 lifecycle
state 会 fail closed，并由 API adapter 映射为安全的 unavailable response；adapter 不泄露 repository 或 lifecycle 细节。

The service delegates the actual graph comparison to the existing `GraphDiff::between`; no
transport, SDK, or presentation layer calculates a second graph diff. This is a private,
version-backed local read contract. It adds no public REST/OpenAPI/SDK write, Web mutation,
operator transport, migration, provider, secret access, Docker/PostgreSQL runtime claim, or
production-readiness decision.

service 将实际 graph comparison 委托给既有的 `GraphDiff::between`；transport、SDK 与 presentation layer 都不会计算第二个
graph diff。这是一份 private、version-backed local read contract，不新增 public REST/OpenAPI/SDK write、Web mutation、
operator transport、migration、provider、secret access、Docker/PostgreSQL runtime 声明或 production-readiness decision。

Fresh verification / 新鲜验证：focused storage lifecycle-witness GraphDiff review tests `4 passed`;
focused API GraphDiff tests `13 passed`; API integration contract `3 passed`; workspace Rust
`221 passed, 41 ignored`; `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0`
check; `pnpm check:web` with public SDK `15`, local SDK `148`, Web `298`, and production Web build;
and `tests/contract/verify-local-contracts.test.ps1` fixture verification all passed. PostgreSQL
runtime, authenticated browser/visual smoke, remote CI, operator rehearsal, Docker, Git, release,
and production evidence remain unobserved or deferred.

新鲜验证：focused storage lifecycle-witness GraphDiff review tests `4 passed`；focused API GraphDiff tests `13 passed`；API
integration contract `3 passed`；workspace Rust `221 passed, 41 ignored`；`cargo fmt --all -- --check`；strict offline Clippy；锁定
Rust `1.85.0` check；`pnpm check:web`（public SDK `15`、local SDK `148`、Web `298` 与 production Web build）；以及
`tests/contract/verify-local-contracts.test.ps1` fixture verification 均通过。PostgreSQL runtime、authenticated browser/visual
smoke、remote CI、operator rehearsal、Docker、Git、release 与 production evidence 继续为 `unobserved` 或 `deferred`。

## Guarded Lifecycle to Versioned GraphDiff Integration / Guarded Lifecycle 到版本化 GraphDiff 集成

The private storage integration contract now exercises the actual `ContextLifecycleService` guarded
writer rather than synthetic lifecycle facts. The in-memory path creates a Context root, creates a
component, records an immutable content revision, creates a second component, and adds/removes a
typed `Uses` relationship. The same repository then supplies exact lifecycle facts and graph
snapshots to `PersistedContextGraphDiffReviewService`, which returns the expected edge additions
and removals with exact source/target commit scopes. Missing and mismatched scopes fail closed.

私有 storage integration contract 现直接执行 `ContextLifecycleService` guarded writer，不再以合成 lifecycle facts 作为成功路径。
in-memory path 会创建 Context root、创建 component、记录 immutable content revision、创建第二个 component，并添加/移除 typed
`Uses` relationship。随后同一 repository 将 exact lifecycle facts 与 graph snapshot 提供给
`PersistedContextGraphDiffReviewService`，返回符合预期的 edge addition/removal 与 exact source/target commit scope；missing
或 mismatched scope 会 fail closed。

This is test-only evidence at the reusable storage boundary. It does not add a route, SDK surface,
Web mutation, migration, provider, secret access, or operator transport. The versioned review path
still delegates the calculation to the sole `GraphDiff::between` implementation.

本回执仅是 reusable storage boundary 的 test-only evidence，不新增 route、SDK surface、Web mutation、migration、provider、
secret access 或 operator transport。版本化 review path 仍将 calculation 委托给唯一的 `GraphDiff::between` implementation。

## Private Commit-History Replay Contract / 私有提交历史回放契约

The reusable versioning core now exposes a read-only `CommitHistory` aggregate over an immutable
commit set and explicit `BranchHead` values. Construction validates one Context scope, unique
commit and branch identities, known heads, complete parent ancestry, and the existing commit-graph
invariants. Branch replay is deterministic root-to-head traversal; linear replay rejects merge
ancestry, while `merge_plan` delegates ancestry classification to the existing versioning domain.

可复用 versioning core 现提供只读 `CommitHistory` aggregate，组合不可变 commit set 与显式 `BranchHead`。构造时校验
单一 Context scope、唯一 commit/branch identity、已知 head、完整 parent ancestry 与既有 commit-graph invariant。
Branch replay 按确定性的 root-to-head 顺序读取；linear replay 拒绝 merge ancestry，`merge_plan` 委托既有 versioning
domain 完成 ancestry classification。

This boundary performs no commit creation, branch mutation, merge execution, persistence, or
transport. It is intended to be consumed by version-backed Context replay and review adapters;
the graph calculation remains exclusively in `GraphDiff::between`. Focused history, Diff, full Rust,
Rust 1.85, Clippy, Web, and local contract evidence is recorded in the corresponding bilingual
plan and roadmap receipt. PostgreSQL, Docker, browser/visual, Git, remote, operator, release, and
production evidence remains outside this local contract.

本边界不创建 commit、不修改 branch、不执行 merge、不持久化数据，也不提供 transport。它供 version-backed Context
replay 与 review adapter 消费；graph calculation 仍只由 `GraphDiff::between` 负责。focused history、Diff、full Rust、
Rust 1.85、Clippy、Web 与 local contract 证据已记录在对应双语计划和路线图回执中。PostgreSQL、Docker、browser/visual、
Git、remote、operator、release 与 production evidence 仍不属于本地 contract。

## Private History-Bound Graph Review / 私有 History-Bound Graph Review

`contextlab-storage::ContextCommitHistoryRepository` is the read-only application composition
between the existing paginated commit list/detail port and typed branch-head port. It rehydrates
`ContextCommit` through the existing persistence decoder, then calls
`contextlab-versioning::CommitHistory::try_from_parts`; complete parent ancestry, Context scope,
duplicate identities, explicit born/unborn heads, and malformed changes therefore fail closed in
one reusable versioning boundary. `ContextCommitHistoryRepositoryAdapter` composes separately owned
repository trait objects for API state without moving domain ownership into storage.

`PersistedContextGraphHistoryReviewService` accepts an exact snapshot repository and a history
repository, validates both requested commit identities against the complete history, and delegates
the actual snapshot review to the existing `PersistedContextGraphDiffReviewService`. The protected
local graph-diff GET uses this composition while retaining its response schema, auth/RBAC/audit/
rate-limit/no-store boundaries. No API, SDK, OpenAPI, Web, or mutation contract was added. The
branch-head selection policy and atomic batch read across the separately owned ports remain explicit
follow-up work; this composition does not claim PostgreSQL runtime atomicity.

`contextlab-storage::ContextCommitHistoryRepository` 是既有分页 commit list/detail port 与 typed branch-head port 之间的
只读 application composition。它通过既有 persistence decoder 重建 `ContextCommit`，再调用
`contextlab-versioning::CommitHistory::try_from_parts`；完整 parent ancestry、Context scope、duplicate identity、显式
born/unborn head 与 malformed changes 因而在一个可复用 versioning boundary 中 fail closed。
`ContextCommitHistoryRepositoryAdapter` 为 API state 组合分离拥有的 repository trait object，同时不把 domain ownership
移入 storage。

`PersistedContextGraphHistoryReviewService` 接收 exact snapshot repository 与 history repository，先将请求的两个 commit
identity 对照完整 history 校验，再委托既有 `PersistedContextGraphDiffReviewService` 执行 snapshot review。protected local
graph-diff GET 使用该 composition，同时保留 response schema、auth/RBAC/audit/rate-limit/no-store boundary。未新增 API、SDK、
OpenAPI、Web 或 mutation contract。branch-head selection policy 与跨分离 port 的 atomic batch read 仍是显式 follow-up；本
composition 不声称 PostgreSQL runtime atomicity。

### Branch-Head-Bound Graph Review / Branch-Head-Bound Graph Review

The private `PersistedContextGraphHistoryReviewService` accepts explicit exact commit scopes and
enforces history Context ownership before commit membership or snapshot reads proceed. Branch-name
selection is no longer available on this split-port service; it is exclusively provided by
`PersistedContextGraphWitnessReviewService::review_branch_head`, which consumes the atomic
branch-bound witness repository. A cross-Context history collision therefore fails closed, and the
API maps the domain error to the existing private unavailable response rather than exposing history
diagnostics. / 私有 `PersistedContextGraphHistoryReviewService` 接收显式 exact commit scope，并在 commit membership 或
snapshot read 前强制 history Context ownership。该 split-port service 不再提供 branch-name selection；branch-name selection 只由
消费 atomic branch-bound witness repository 的 `PersistedContextGraphWitnessReviewService::review_branch_head` 提供。因此
cross-Context history collision 会 fail closed，API 将 domain error 映射到既有 private unavailable response，不暴露 history diagnostics。

This is a read-only storage/application hardening increment. It does not add transport or mutation,
does not claim atomicity across separately owned ports or PostgreSQL runtime behavior, and continues
to delegate graph calculation to the sole `GraphDiff::between`. / 这是只读 storage/application 硬化增量，不新增 transport 或
mutation，不声称分离 port 之间的 atomicity 或 PostgreSQL runtime behavior，并继续将 graph calculation 委托给唯一的
`GraphDiff::between`。

### Consistent Context Commit-History Read / 一致的 Context 提交历史读取

The private `ContextCommitHistoryRepository` now has a backend-owned concrete path. Memory holds one
`RwLock` read guard while collecting static/dynamic commit records and branch heads. PostgreSQL uses
one `REPEATABLE READ READ ONLY` transaction for Context existence, complete commit rows, parent order,
and branch-head rows. Both paths reuse the storage decoder and `CommitHistory::try_from_parts`; neither
creates commits or calculates graph diffs.

私有 `ContextCommitHistoryRepository` 现在拥有 backend-owned concrete path。Memory 在收集 static/dynamic commit record 与
branch head 时持有一个 `RwLock` read guard。PostgreSQL 使用一个 `REPEATABLE READ READ ONLY` transaction 完成 Context
existence、完整 commit row、parent order 与 branch-head row 读取。两条路径复用 storage decoder 与
`CommitHistory::try_from_parts`；都不创建 commit，也不计算 graph diff。

The environment-backed `WorkspaceDataRepository` is injected into `AppState`, and the existing
protected graph-diff GET consumes that concrete history contract. `WorkspaceRepositories` requires
an explicit history repository for custom composition; when omitted, `AppState::with_workspace_repositories`
installs a fail-closed unavailable reader instead of silently composing independent commit and
branch-head ports. The generic storage adapter remains available only when a test explicitly asks
for that legacy split-port behavior. This is local read-only evidence and does not claim live
PostgreSQL, Docker, public expansion, production readiness, or release. / environment-backed
`WorkspaceDataRepository` 注入 `AppState`，现有 protected graph-diff GET 使用该 concrete history contract。
`WorkspaceRepositories` 要求 custom composition 显式提供 history repository；缺失时，
`AppState::with_workspace_repositories` 安装 fail-closed unavailable reader，不再静默组合独立 commit 与 branch-head port。
generic storage adapter 仅在 test 明确要求 legacy split-port behavior 时可用。这是本地只读证据，不声称 live PostgreSQL、
Docker、public 扩展、生产就绪或 release。

### Atomic Branch-Head Graph Witness / 原子 Branch-Head Graph Witness

The private `ContextGraphBranchHeadReviewWitnessRepository` accepts an exact source scope and a
server-owned typed `BranchName`. Memory selects the head and reads complete history plus source and
selected-target graph snapshots under one `RwLock` read guard. PostgreSQL performs the same work in
one `REPEATABLE READ READ ONLY` transaction. The immutable witness retains the selected branch and
exact target commit, rejects unknown or unborn branches, and rejects any target that is not the
validated branch head. It does not call the split-port branch-head adapter or accept a caller-
invented revised commit.

私有 `ContextGraphBranchHeadReviewWitnessRepository` 接收 exact source scope 与 server-owned typed `BranchName`。Memory 在一个
`RwLock` read guard 下选择 head，并读取完整 history、source 与 selected-target graph snapshot。PostgreSQL 在一个
`REPEATABLE READ READ ONLY` transaction 内完成相同工作。immutable witness 保留 selected branch 与 exact target commit，拒绝
unknown/unborn branch，也拒绝不是 validated branch head 的 target。它不调用 split-port branch-head adapter，也不接受调用方臆造的
revised commit。

`PersistedContextGraphWitnessReviewService::review_branch_head` passes the branch-bound immutable
pair to the existing versioned graph-review projection. The storage contract only selects and
validates; `GraphDiff::between` remains the sole graph-diff calculator. This is a private local Rust
read boundary: no public REST/OpenAPI/SDK/Web method, mutation, migration, provider, operator
transport, release, or production claim is added.

`PersistedContextGraphWitnessReviewService::review_branch_head` 将 branch-bound immutable pair 传给既有 versioned graph-review
projection。storage contract 只负责 selection 与 validation；`GraphDiff::between` 仍是唯一 graph-diff calculator。本边界是 private
local Rust read boundary：不新增 public REST/OpenAPI/SDK/Web method、mutation、migration、provider、operator transport、release
或生产声明。

## Normal First-Parent Versioned Graph Review / Normal First-Parent 版本化图审阅

Version-backed graph review now has an explicit reusable ancestry boundary. The versioning core's
`CommitHistory::normal_first_parent_path` walks from the requested target through its sole parent
until the requested source is reached, returns a deterministic inclusive path, and rejects unknown,
reversed, unrelated, or merge-containing ranges. This is a read-only validation contract; it does
not create commits, move branch heads, execute merges, or infer ancestry from caller-provided text.

版本化 graph review 现拥有明确的可复用 ancestry boundary。versioning core 的 `CommitHistory::normal_first_parent_path` 从 target
沿 sole parent 回溯，直到到达 source，返回确定性的 inclusive path，并拒绝 unknown、reversed、unrelated 或包含 merge 的 range。
这是只读 validation contract；不创建 commit、不移动 branch head、不执行 merge，也不从调用方文本推断 ancestry。

`PersistedContextGraphHistoryReviewService` and the atomic `ContextGraphReviewWitness` both enforce
the same contract before snapshot projection. Merge ancestry is intentionally reserved for the
existing merge-review path. The graph review continues to call the sole `GraphDiff::between`; no
REST/OpenAPI/SDK/Web surface, public write path, migration, provider, or operator transport is
introduced. / `PersistedContextGraphHistoryReviewService` 与 atomic `ContextGraphReviewWitness` 均在 snapshot projection 前执行同一
contract。merge ancestry 明确保留给既有 merge-review path。graph review 继续调用唯一 `GraphDiff::between`；不新增 REST/OpenAPI/SDK/Web
surface、public write path、migration、provider 或 operator transport。

## Private Atomic Merge Review Witness / 私有原子 Merge Review Witness

The server-owned merge review read is an atomic storage/application contract. A
`ContextMergeReviewWitnessRepository` resolves the exact `MergePlan` and binds its base, left,
and right immutable graph snapshots inside one consistent observation. Memory uses one `RwLock`
read guard; PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction and constructs the validated
witness before commit. The application service verifies that the returned witness still matches
the requested `(ProjectId, ContextId, leftCommitId, rightCommitId)` scope before delegating to the
existing classifier. / server-owned merge review read 是 atomic storage/application contract。
`ContextMergeReviewWitnessRepository` 在一个一致 observation 内解析 exact `MergePlan`，并绑定 base、left、right immutable graph
snapshot。Memory 使用一个 `RwLock` read guard；PostgreSQL 使用一个 `REPEATABLE READ READ ONLY` transaction，并在 commit 前构造已验证
witness。application service 在委托既有 classifier 前，重新确认返回 witness 匹配请求的 `(ProjectId, ContextId, leftCommitId,
rightCommitId)` scope。

Missing witness wiring is fail-closed: `AppState` stores an optional private dependency and the
protected route returns the existing unavailable response instead of preview fallback. Storage
does not calculate a graph diff; `GraphDiff::between` remains the only graph-diff calculator.
The local tests prove this contract, but do not prove a live PostgreSQL runtime or any release or
production property. / witness wiring 缺失时 fail closed：`AppState` 保存 optional private dependency，protected route 返回既有
unavailable response，而不是回退到 preview。storage 不计算 graph diff；`GraphDiff::between` 仍是唯一 graph-diff calculator。本地测试
证明该 contract，但不证明 live PostgreSQL runtime，也不证明 release 或 production 属性。
