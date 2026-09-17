# Private Unborn-Branch Context Initialization Design / 私有未出生分支 Context 初始化设计

## Decision / 决策

ContextLab will add one `initialize` intent to the existing private local component-lifecycle contract. It creates the first immutable Context commit and graph snapshot for an already persisted Context on an unborn branch. The command is replayable through the sole guarded writer and does not create a component, body revision, public route, OpenAPI operation, public SDK method, or new graph-diff calculator.

ContextLab 将在既有私有 local component-lifecycle contract 中增加一个 `initialize` intent。它会为已持久化的 Context 在未出生分支上创建第一条不可变 Context commit 与 graph snapshot。该 command 通过唯一的 guarded writer 支持 replay，不创建 component、body revision、public route、OpenAPI operation、public SDK method 或新的 graph-diff calculator。

## Problem / 问题

The existing lifecycle service and local request require a non-null materialized branch head. Consequently, a local developer can create, revise, rename, remove, replay, and compare components only after another mechanism has already created a Context root. This prevents the local lifecycle workflow from forming a complete Context-first path from an empty branch.

现有 lifecycle service 与 local request 都要求非空、已 materialize 的 branch head。因此，本地开发者只能在另一个机制已创建 Context root 后，才能创建、修订、重命名、移除、回放和比较 component。这使本地 lifecycle workflow 无法从空分支形成完整的 Context-first 路径。

## Chosen Shape / 选定形态

`ContextLifecycleOperation::Initialize` has no caller-controlled graph, Context change, component, body, timestamp, principal, or digest fields. The existing Context record supplies the root graph label. The application service constructs exactly one `CreatedContext` change and a one-node `context:{context_id}` `ContextGraph`, uses `ExpectedBranchHead::Unborn`, and passes that state to `GuardedContextCommitWriter`.

`ContextLifecycleOperation::Initialize` 不包含调用方可控制的 graph、Context change、component、body、timestamp、principal 或 digest 字段。既有 Context record 提供 root graph label。application service 构造恰好一个 `CreatedContext` change 和只含 `context:{context_id}` 节点的 `ContextGraph`，使用 `ExpectedBranchHead::Unborn`，再将该状态交给 `GuardedContextCommitWriter`。

The existing local request keeps one tagged operation. Its `expected_head_commit_id` is `null` only for `initialize`; every create, update, descriptor update, and removal still requires an exact non-null materialized head. The existing authorization, audit, `ContextCommitWrite` rate-limit operation, branch-scoped idempotency, branch-head compare-and-swap, request digest, same-origin BFF, and private/no-store behavior are reused without a bypass.

既有 local request 保持一个带标签的 operation。`expected_head_commit_id` 只允许在 `initialize` 时为 `null`；所有 create、update、descriptor update 与 removal 仍必须提供精确的非空 materialized head。既有 authorization、audit、`ContextCommitWrite` rate-limit operation、branch-scoped idempotency、branch-head compare-and-swap、request digest、同源 BFF 与 private/no-store 行为全部复用，不增加绕过路径。

## Rejected Alternatives / 已拒绝方案

1. Reuse the generic protected commit endpoint: it accepts client-provided changes and graph snapshots, which violates the lifecycle service's server-owned Context state boundary.
2. Treat a first component creation as implicit initialization: it couples Context root creation to one component type, prevents an empty initial state, and makes replay semantics less explicit.
3. Add a public initialization route or public SDK method: public protected-write promotion is outside the local delivery scope.

1. 复用通用 protected commit endpoint：它接受调用方提供的 changes 与 graph snapshots，违反 lifecycle service 对服务器拥有 Context state 的边界。
2. 将首次 component creation 视为隐式初始化：这会把 Context root creation 与某个 component type 耦合，无法形成空初始状态，也会让 replay 语义不够明确。
3. 新增 public initialization route 或 public SDK method：public protected-write promotion 不属于本地交付范围。

## Verification / 验证

Tests must first fail for root initialization through an unborn branch, graph-root construction, creation replay, and rejection of initialize-on-headed branches or non-initialize null heads. They then prove atomic commit/snapshot/head/receipt persistence in memory, compiled ignored PostgreSQL parity, protected local API authorization and public-contract absence, strict local SDK parsing, and bilingual design-system editor behavior. Docker remains disabled, so PostgreSQL runtime and authenticated browser-to-BFF-to-Axum runtime smoke stay explicitly unobserved.

测试必须先对未出生分支 root initialization、graph root 构造、creation replay，以及 headed branch 初始化或非初始化 null head 的拒绝产生失败；随后证明 memory 中 commit/snapshot/head/receipt 的原子持久化、已编译且 ignored 的 PostgreSQL parity、protected local API authorization 与 public contract 缺席、严格 local SDK parsing，以及双语 design-system editor 行为。Docker 保持关闭，因此 PostgreSQL runtime 与 authenticated browser-to-BFF-to-Axum runtime smoke 仍明确为未观测。
