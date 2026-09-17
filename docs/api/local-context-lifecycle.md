# Local Context Lifecycle API / 本地 Context 生命周期 API

## Scope / 范围

`server/api` provides two explicitly protected, local-development lifecycle routes. They are not public REST operations: they are absent from the default public router, `docs/api/openapi.json`, and `@contextlab/ts-sdk`. They make no release, public-promotion, remote-CI, or production-readiness claim.

`server/api` 提供两条显式 protected 的本地开发生命周期 route。它们不是 public REST operation：默认 public router、`docs/api/openapi.json` 与 `@contextlab/ts-sdk` 均不包含它们。它们不构成 release、public promotion、remote CI 或 production readiness 声明。

The server defaults to loopback binding, but “local” here describes the non-public contract and local-development workflow, not an independent network-reachability guarantee when an operator deliberately overrides the bind address. Deployments must keep the protected router and its credentials inside an appropriate trust boundary.

服务默认绑定 loopback，但此处“本地”描述的是非公开 contract 与本地开发 workflow；当 operator 有意覆盖绑定地址时，它不单独保证网络不可达。部署必须将 protected router 与其凭据放在适当的信任边界内。

## Protected Routes / 受保护路由

- `GET /api/v1/local/contexts/{context_id}/commits/{commit_id}/lifecycle-state` returns one exact materialized Context state: stable component descriptors, metadata, immutable effective bodies and hashes, creation/content provenance, and the same commit's immutable graph snapshot.
- `POST /api/v1/local/contexts/{context_id}/component-lifecycle-commits` accepts exactly one tagged `initialize`, `create`, `update`, `update_descriptor`, `remove`, `add_uses_relationship`, or `remove_uses_relationship` intent and creates or replays one guarded Context commit. `initialize` constructs the root Context graph state only for an unborn branch; `update_descriptor` replaces only the component name and JSON metadata while preserving the immutable body hash and effective content commit; the two Uses intents carry only source and target component identifiers.

- `GET /api/v1/local/contexts/{context_id}/commits/{commit_id}/lifecycle-state` 返回一个精确的 materialized Context state：稳定的 component descriptor、metadata、不可变有效正文与 hash、creation/content provenance，以及同一 commit 的不可变 graph snapshot。
- `POST /api/v1/local/contexts/{context_id}/component-lifecycle-commits` 只接受一个带标签的 `initialize`、`create`、`update`、`update_descriptor`、`remove`、`add_uses_relationship` 或 `remove_uses_relationship` intent，并创建或 replay 一个 guarded Context commit。`initialize` 只为 unborn branch 构造 root Context graph state；`update_descriptor` 只替换 component name 与 JSON metadata，保留不可变 body hash 与有效 content commit；两个 Uses intent 只携带 source/target component identifier。

`initialize` alone requires `expected_head_commit_id: null`; every other mutation requires a non-null materialized normal first-parent head, a valid branch name, message, and `Idempotency-Key`. The request has strict decoding: callers cannot supply Context changes, graph snapshots, component IDs for creation, hashes, capture timestamps, principals, or request digests. The server constructs those facts through `ContextLifecycleService` and the sole `GuardedContextCommitWriter`.

Idempotency replay is scoped to the authenticated identity, Context, branch, and key. The same key may independently create one commit on a different branch, while a same-branch retry always returns its original receipt even if that branch head has advanced; the receipt branch must also match its referenced immutable commit before replay.

只有 `initialize` 要求 `expected_head_commit_id: null`；其他写入仍必须提供非空且已 materialize 的 normal first-parent head、有效 branch name、message 与 `Idempotency-Key`。请求采用严格解码：调用方不能提供 Context change、graph snapshot、创建时的 component ID、hash、capture timestamp、principal 或 request digest。服务端通过 `ContextLifecycleService` 与唯一的 `GuardedContextCommitWriter` 构造这些事实。

idempotency replay 的作用域是 authenticated identity、Context、branch 与 key。相同 key 可在另一个 branch 上独立创建一个 commit；同一 branch 的 retry 即使发生在 branch head 已推进之后，也只会返回其原始 receipt；在 replay 前 receipt 的 branch 还必须与其引用的不可变 commit 相匹配。

For `add_uses_relationship` and `remove_uses_relationship`, the non-null expected head identifies the exact server-owned graph and component-state reconstruction point. The server requires two distinct endpoints, proves both are active components in the same Context, validates their graph nodes and structural Context relationships, and derives the successor graph itself. Callers cannot submit an edge kind or graph payload: the scope is exactly one directed `Uses` edge. Duplicate addition and missing removal fail closed; an identical guarded retry replays the original receipt without a second graph mutation.

对于 `add_uses_relationship` 与 `remove_uses_relationship`，非空 expected head 会指定由服务端拥有的精确 graph 与 component-state 重建点。服务端要求两个 endpoint 不同，确认二者都是同一 Context 中的 active component，校验其 graph node 与结构性 Context relationship，并自行派生 successor graph。调用方不能提交 edge kind 或 graph payload：范围严格限定为一条有向 `Uses` edge。重复新增和移除不存在的关系都会 fail closed；相同的 guarded retry 只 replay 原始 receipt，不会再次修改 graph。

## Access And Failure Semantics / 访问与失败语义

Authentication runs before rate limiting. Lifecycle-state reads use `ProtectedRouteOperation::ContextLifecycleRead` and require Context `Read`; lifecycle mutations use `ContextCommitWrite` and require Context `Write`. Every authorization decision is recorded before the handler proceeds, and an unavailable audit sink fails closed. A reader can receive component bodies from the protected lifecycle-state route, so memberships for a local developer session must be chosen accordingly.

authentication 先于 rate limit 执行。生命周期 state read 使用 `ProtectedRouteOperation::ContextLifecycleRead` 并要求 Context `Read`；生命周期 mutation 使用 `ContextCommitWrite` 并要求 Context `Write`。每个 authorization decision 都会在 handler 继续前被记录；audit sink 不可用时会 fail closed。reader 能从 protected lifecycle-state route 得到 component body，因此本地开发会话的 membership 必须据此配置。

Malformed commands return `400 invalid_context_lifecycle_request`. A valid command whose materialized history cannot provide a component, body, descriptor witness, or matching graph node/edge returns `409 context_lifecycle_state_conflict`; branch-head and idempotency conflicts retain their existing guarded-writer storage errors. Aggregate reads validate body, descriptor, and graph correspondence before returning any state.

格式错误的 command 返回 `400 invalid_context_lifecycle_request`。合法 command 若其 materialized history 无法提供 component、body、descriptor witness 或匹配的 graph node/edge，则返回 `409 context_lifecycle_state_conflict`；branch-head 与 idempotency conflict 保持既有 guarded-writer storage error。聚合 read 会在返回任何 state 前验证 body、descriptor 与 graph 的一致性。

## Deliberate Boundaries / 明确边界

The routes neither add an OpenAPI operation nor extend the public TypeScript client. They do not create Context records, implement arbitrary graph editing, add branch/merge UI, semantic/behavior/evaluation diff, benchmark execution, operator transport, or a second graph-diff calculator. Version comparison continues to delegate exclusively to `GraphDiff`.

这些 route 不新增 OpenAPI operation，也不扩展 public TypeScript client。它们不创建 Context 记录，也不实现任意 graph editing、branch/merge UI、semantic/behavior/evaluation diff、benchmark execution、operator transport 或第二个 graph-diff calculator。版本比较仍只委托给 `GraphDiff`。

`@contextlab/local-sdk` is a separate private workspace package for this contract. `ContextLabLocalClient` stores only its base URL and fetch implementation. Each state read receives a caller-supplied `bearerToken`; each mutation also receives a caller-supplied `idempotencyKey`. It never persists, logs, issues, refreshes, or forwards those credentials to a public endpoint.

`@contextlab/local-sdk` 是该 contract 的独立私有 workspace package。`ContextLabLocalClient` 只保存 base URL 与 fetch implementation。每次 state read 都接收调用方提供的 `bearerToken`；每次 mutation 还接收调用方提供的 `idempotencyKey`。它绝不持久化、记录、签发、刷新这些凭据，也不会将它们转发给 public endpoint。

## Local Web Workflow / 本地 Web 工作流

The Web workspace exposes lifecycle mutation only when the server-owned `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` value is exactly `true`. It otherwise omits the editor and the mutation BFF fails closed with `403 local_lifecycle_disabled` before parsing a body or forwarding credentials. When enabled, the browser supplies a bearer token from in-memory form state and a per-mutation idempotency key; its fetch requests explicitly omit cookies. The BFF forwards only those request-scoped headers to the explicit local SDK route, never persists or logs credentials, and preserves structured errors plus `Retry-After` guidance for rate-limited requests.

Web 工作台只有在服务端 `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` 精确为 `true` 时才暴露 lifecycle mutation；否则它不渲染 editor，mutation BFF 会在解析 body 或转发 credential 前以 `403 local_lifecycle_disabled` fail closed。启用后，浏览器从内存表单状态提供 bearer token，并为每次 mutation 提供 idempotency key；其 fetch 请求会显式省略 cookie。BFF 只把这些 request-scoped header 转发给明确的 local SDK route，绝不持久化或记录凭据，并会保留结构化错误及限流请求的 `Retry-After` 指引。

`ContextLifecycleEditor` is a design-system composition, not a second lifecycle implementation. It reads an explicitly selected materialized commit, builds exactly one tagged lifecycle intent through the client presenter, and after a successful guarded commit reloads the new aggregate state before refreshing the workspace's existing version-backed `GraphDiff` review. Its browser controls remain a local-development workflow and do not add a public Web mutation surface.

`ContextLifecycleEditor` 是 design-system 的组合，而不是第二套生命周期实现。它读取显式选中的已物化 commit，经客户端 presenter 构造唯一的带标签 lifecycle intent，并在 guarded commit 成功后重新读取新聚合状态，再刷新工作台既有的基于版本的 `GraphDiff` 审阅。它的浏览器控件仍是本地开发 workflow，不新增 public Web mutation surface。

Unit, component, local-SDK, and BFF transport checks are present, and preview-mode desktop/mobile browser smoke has been observed. No browser-to-BFF-to-protected-Axum mutation smoke is claimed because no authenticated local protected runtime was used in this increment.

已有 unit、component、local SDK 与 BFF transport 检查，并已观察 preview mode 的 desktop/mobile browser smoke。本增量未使用已认证的本地 protected runtime，因此不声称完成 browser-to-BFF-to-protected-Axum mutation smoke。

For the typed Uses relationship increment specifically, Docker-backed PostgreSQL runtime and authenticated browser-to-BFF-to-protected-Axum mutation runtime remain unobserved. The server-owned default-deny gate is unchanged, public transport is unchanged, and version comparison still delegates exclusively to `GraphDiff::between`.

仅就类型化 Uses relationship 增量而言，Docker-backed PostgreSQL runtime 与 authenticated browser-to-BFF-to-protected-Axum mutation runtime 仍未观测。服务端拥有的 default-deny gate 保持不变，public transport 保持不变，版本比较仍只委托给 `GraphDiff::between`。

## Private Sealed Benchmark Run Details Read / 私有已封存 Benchmark Run 明细读取

The admitted decision run-details workflow is a separate private local read path layered on the existing exact sealed-decision inspection boundary. After the existing authentication, `ContextPermission::Read`, audit, and benchmark-specific rate-limit checks, it resolves only the sealed decision's ordered run membership and returns a fail-closed redacted summary at exact project/Context/commit/run scope. Missing or out-of-scope members are unavailable rather than substituted by the generic public evaluation-run reader.

准入的 decision run-details workflow 是叠加在既有精确 sealed-decision inspection boundary 上的独立私有 local read path。它只有在既有 authentication、`ContextPermission::Read`、audit 与 benchmark 专用 rate-limit 检查完成后，才解析 sealed decision 的有序 run membership，并按精确的 project/Context/commit/run scope 返回 fail-closed 的 redacted summary。缺失或越界 member 会保持 unavailable，不会用 generic public evaluation-run reader 替代。

The local GET, non-public SDK, and same-origin BFF add no public REST, OpenAPI, or public SDK write surface, do not expose raw benchmark payloads, and do not recalculate policy or `GraphDiff`. The BFF preserves request-scoped credentials, omits cookies, and keeps `private, no-store` responses. Focused API/local-SDK/Web tests and the full `pnpm check:web` production build are freshly observed. Docker/PostgreSQL runtime and authenticated browser-to-BFF-to-protected-Axum E2E remain unobserved.

local GET、非公开 SDK 与同源 BFF 不新增 public REST、OpenAPI 或 public SDK write surface，不暴露 raw benchmark payload，也不重新计算 policy 或 `GraphDiff`。BFF 保持 request-scoped credential、忽略 cookie，并维持 `private, no-store` response。聚焦 API/local SDK/Web 测试与完整 `pnpm check:web` production build 已获得新鲜观测。Docker/PostgreSQL runtime 与 authenticated browser-to-BFF-to-protected-Axum E2E 仍未观测。

## Private Workflow Binding Read Boundary / 私有 Workflow Binding 读取边界

The private read contract is scoped by both the path `ContextId` and one exact `commit_id`. Its API, non-public local SDK, same-origin BFF, and Web consumer are implemented as a local read-only vertical slice with focused receipts. This does not promote the route into the public REST/OpenAPI/public SDK catalog.

该私有 read contract 同时以 path `ContextId` 与一个精确的 `commit_id` 作为 scope。API、非公开 local SDK、同源 BFF 与 Web consumer 已作为本地只读垂直切片实现并取得聚焦回执。本路由不会因此进入 public REST/OpenAPI/public SDK catalog。

```text
GET /api/v1/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings
```

The same-origin Web BFF uses the corresponding browser path below and must keep the path scope exact:

同源 Web BFF 使用下面对应的 browser path，并必须保持 path scope 精确：

```text
GET /api/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings
```

The checked `server/api` route authenticates and authorizes the path Context with `ContextPermission::Read` before it accesses the optional binding repository, passes the same Context and commit scope unchanged, and returns typed unavailable/storage failures. Its focused test command, `cargo test -p contextlab-api local_workflow_bindings --quiet`, returned `4 passed`. A missing repository is a typed `unavailable` result; the route must not read a preview/current head or resolve a binding by Workflow identifier alone.

检查到的 `server/api` route 会先完成 authentication，并在访问 optional binding repository 前使用 `ContextPermission::Read` 对 path Context 执行 authorization，随后将相同的 Context 与 commit scope 原样传递，并返回 typed unavailable/storage failure。其聚焦 test command 返回 `4 passed`。repository 缺失时返回类型化 `unavailable`；route 不得读取 preview/current head，也不得只凭 Workflow identifier 解析 binding。

The versioned response schema is `contextlab.local-workflow-context-bindings.v1`:

版本化 response schema 为 `contextlab.local-workflow-context-bindings.v1`：

```json
{
  "schema_version": "contextlab.local-workflow-context-bindings.v1",
  "context_id": "<path context_id>",
  "commit_id": "<path commit_id>",
  "bindings": [
    {
      "binding_id": "<stable id>",
      "workflow_id": "<stable id>",
      "workflow_revision": 1,
      "node_count": 0,
      "edge_count": 0
    }
  ]
}
```

`bindings` keeps the storage-defined `(workflow_id, workflow_revision, binding_id)` order. The response is a redacted summary only: it excludes workflow definitions, node/edge payloads, credentials, provider configuration, and operational secrets. The non-public local SDK parser must reject an unknown schema, mismatched scope, unsupported fields, raw fields, or inconsistent counts rather than guessing a compatible shape.

`bindings` 保持 storage 定义的 `(workflow_id, workflow_revision, binding_id)` 顺序。response 仅是脱敏 summary：排除 Workflow definition、node/edge payload、credential、provider configuration 与 operational secret。非公开 local SDK parser 遇到未知 schema、scope 不匹配、不支持字段、raw field 或不一致 count 时必须拒绝，而不是猜测兼容 shape。

The non-public `@contextlab/local-sdk` parser/client is present. The fresh focused workflow-binding run returned `7 passed`; the Web inspector and its `data -> presenter -> screen` files call the same-origin BFF path below, and the focused Web binding/presenter/inspector run returned `8 passed`. The nested BFF route test returned `9 passed`, including raw-field, scope, authentication, authorization, rate-limit, and unavailable fail-closed behavior. `pnpm check:web` also passed with local SDK `70` and Web `160`, including the production build. These are local implementation receipts, not authenticated runtime or production receipts.

非公开 `@contextlab/local-sdk` parser/client 已存在。新鲜聚焦 workflow-binding run 返回 `7 passed`；Web inspector 及其 `data -> presenter -> screen` 文件调用下方同源 BFF path，聚焦 Web binding/presenter/inspector run 返回 `8 passed`。嵌套 BFF route test 返回 `9 passed`，覆盖 raw-field、scope、authentication、authorization、rate-limit 与 unavailable fail-closed 行为。`pnpm check:web` 也通过，local SDK `70`、Web `160`，并完成 production build。这些是本地 implementation receipt，不是 authenticated runtime 或 production receipt。

This remains a private contract note and partial implementation ledger, not an authenticated transport receipt. It adds no OpenAPI operation or public SDK method. The BFF must forward only the request-scoped bearer token, omit cookies, use `private, no-store`, preserve structured `401`/`403`/`429`/`503` failures, and never substitute preview/current-head data. `GraphDiff::between` remains the sole graph-diff calculator.

这仍是私有 contract 说明与部分实现台账，不是 authenticated transport 回执。不新增 OpenAPI operation 或 public SDK method。BFF 必须只转发 request-scoped bearer token、忽略 cookie、使用 `private, no-store`、保留结构化 `401`/`403`/`429`/`503` failure，并绝不替换为 preview/current-head data。`GraphDiff::between` 仍是唯一的 graph-diff calculator。

这是私有 contract 说明与本地实现证据记录，不是 authenticated transport runtime 回执。它不新增 OpenAPI operation 或 public SDK method；本次验证确认源码、parser、BFF 与 Web consumer 存在并通过聚焦本地测试，但不把它们提升为 PostgreSQL-backed authenticated runtime、browser/visual、release 或 production evidence。
