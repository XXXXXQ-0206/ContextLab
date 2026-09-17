# Private Benchmark Definition Authoring / 私有 Benchmark 定义创作

## Boundary / 边界

This is a local-only, default-off mutation workflow. It is not part of the checked-in public OpenAPI contract, the public TypeScript SDK, public promotion, release readiness, or production rollout.

这是一条仅限本地、默认关闭的变更工作流。它不属于已检入的 public OpenAPI contract、public TypeScript SDK、public promotion、release readiness 或 production rollout。

The canonical protected local route is:

```text
POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings
```

The Web same-origin BFF mirrors the scope at:

```text
POST /api/local/projects/{projectId}/contexts/{contextId}/commits/{commitId}/benchmark-definition-bindings
```

旧的 context-only BFF path `/api/local/contexts/{contextId}/commits/{commitId}/benchmark-definitions` 已退役并返回 `410 benchmark_definition_route_gone`；它不再转发 upstream。唯一的 authoring transport 是 project-scoped route。

## Server Contract / 服务端契约

The Rust/storage command is schema version `1`. It requires stable UUIDs for the binding, suite, datasets, and cases; exact dataset membership; one exact immutable Context commit; the expected branch head; a validated principal identity; an idempotency key and digest; and deterministic ordering. Memory and PostgreSQL implement the same atomic writer contract. PostgreSQL rechecks active write authorization and branch-head equality in the transaction, and migration `0020` enforces append-only rows and composite Context/project foreign keys.

Rust/storage command 使用 schema version `1`。它要求 binding、suite、dataset 与 case 使用稳定 UUID；dataset membership 必须精确；必须绑定一条精确的不可变 Context commit；要求 expected branch head；要求经过校验的 principal identity；要求 idempotency key 与 digest；并要求确定性排序。Memory 与 PostgreSQL 实现相同的原子 writer contract。PostgreSQL 在 transaction 内重新检查 active write authorization 与 branch-head equality，迁移 `0020` 强制 append-only row 与 composite Context/project foreign key。

The response is a redacted immutable receipt with `created` or `replayed` disposition, exact project/Context/commit scope, branch, suite ID, dataset IDs, capture time, and bilingual server message. Raw cases, expected outputs, evaluator payloads, and internal traces never cross the API/BFF boundary. Typed storage conflicts map to stable fail-closed responses; the Web presents them without recalculating policy.

response 是脱敏的不可变 receipt，包含 `created` 或 `replayed` disposition、精确 project/Context/commit scope、branch、suite ID、dataset IDs、capture time 与双语 server message。raw case、expected output、evaluator payload 与 internal trace 绝不越过 API/BFF boundary。类型化 storage conflict 映射到稳定的 fail-closed response；Web 只呈现结果，不重新计算 policy。

## Security And Transport / 安全与传输

Authentication precedes the dedicated `BenchmarkDefinitionAuthoringWrite` quota. Authorization and audit are server-owned, and storage errors fail closed. The local SDK sends request-scoped Bearer credentials and `Idempotency-Key`, omits cookies, rejects scope drift and unknown fields, and freezes parsed responses. The BFF forwards only the Bearer header, uses `credentials: "omit"` and `cache: "no-store"`, strips unsafe upstream fields, and applies `Cache-Control: private, no-store` to success and error responses.

authentication 先于专用 `BenchmarkDefinitionAuthoringWrite` quota。authorization 与 audit 由服务端负责，storage error fail closed。local SDK 发送 request-scoped Bearer credential 与 `Idempotency-Key`，不携带 cookie，拒绝 scope drift 与 unknown field，并冻结 parsed response。BFF 只转发 Bearer header，使用 `credentials: "omit"` 与 `cache: "no-store"`，剥离不安全的 upstream field，并对成功与错误 response 应用 `Cache-Control: private, no-store`。

## Web Workflow / Web 工作流

The editor follows `data -> presenter -> screen -> editor` and uses shared design-system primitives. It exposes bilingual loading, error, empty, available, and unavailable states; exact target facts; field validation; pending disablement; and accessible status announcements. Mutation remains disabled unless the server-side local gate is explicitly enabled. The page does not implement authorization, idempotency, branch-head, sealing, benchmark policy, or graph-diff logic.

editor 遵循 `data -> presenter -> screen -> editor`，并使用共享 design-system primitive。它提供双语 loading、error、empty、available 与 unavailable state；展示精确 target fact；执行字段校验；pending 时禁用控件；并提供可访问的 status announcement。除非服务端 local gate 被显式开启，否则 mutation 保持禁用。页面不实现 authorization、idempotency、branch-head、sealing、benchmark policy 或 graph-diff logic。

`GraphDiff::between` remains the sole graph-diff calculator. This authoring route stores a version binding; it does not calculate semantic, behavior, evaluation, or graph diffs.

`GraphDiff::between` 仍是唯一 graph-diff calculator。本 authoring route 只存储 version binding，不计算 semantic、behavior、evaluation 或 graph diff。

## Binding Inspection / 绑定检查

The private read route is:

```text
GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings
```

It returns a redacted, deterministic list of immutable bindings at the exact commit. Each item
contains only binding/project/Context/commit IDs, branch, definition schema version, suite and
dataset metadata, and capture time. Cases, inputs, expected outputs, thresholds, request digests,
and internal traces are excluded. The route authenticates before the dedicated
`BenchmarkDefinitionBindingRead` quota, records the `ContextPermission::Read` audit decision, and
returns `private, no-store` responses. It is not in public OpenAPI or the public SDK.

private read route 为：

```text
GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings
```

它返回精确 commit 下不可变 binding 的脱敏、确定性列表。每一项只包含 binding/project/Context/commit ID、
branch、definition schema version、suite 与 dataset metadata，以及 capture time。case、input、expected output、
threshold、request digest 与 internal trace 均被排除。该 route 会先 authentication，再执行专用
`BenchmarkDefinitionBindingRead` quota，并记录 `ContextPermission::Read` audit decision；response 为
`private, no-store`。它不进入 public OpenAPI 或 public SDK。

The non-public local SDK, same-origin BFF, and Web inspector preserve the exact scope and stable
binding order. Web selection returns only the selected immutable binding ID; it never substitutes a
mutable latest binding and never calculates benchmark conclusions or graph diffs.

非公开 local SDK、同源 BFF 与 Web inspector 保持精确 scope 与稳定 binding order。Web selection 只返回选中的
不可变 binding ID；绝不替换为可变 latest binding，也不计算 benchmark conclusion 或 graph diff。

## Evidence Boundary / 证据边界

Local Rust, SDK, Web, and contract tests are scope-limited evidence. Docker/PostgreSQL runtime, authenticated browser-to-BFF-to-Axum runtime, Git binding, remote CI, operator rehearsal, release, and production evidence must be labeled `ignored`, `unobserved`, or `deferred` unless actually observed. Local evidence never impersonates external deployment proof.

本地 Rust、SDK、Web 与 contract test 只是范围受限的证据。Docker/PostgreSQL runtime、authenticated browser-to-BFF-to-Axum runtime、Git binding、remote CI、operator rehearsal、release 与 production evidence 必须在未实际观测时标记为 `ignored`、`unobserved` 或 `deferred`。本地证据绝不冒充外部部署证明。
## Execution Selection Boundary / 执行选择边界

The private core now consumes the selected immutable binding through a provider-free
execution-selection value. It constructs the existing deterministic `BenchmarkExecutionPlan` from
the binding's exact suite and dataset definitions, preserving project/Context/commit scope, stable
membership, and deterministic case count. It does not accept mutable latest state, invoke an
evaluator, or persist evidence; those actions remain owned by the existing execution service and
separately admitted workflow.

私有 core 现已通过 provider-free execution-selection value 消费已选定的不可变 binding。它从 binding 的精确
suite 与 dataset definition 构造现有确定性的 `BenchmarkExecutionPlan`，保持 project/Context/commit scope、
稳定 membership 与确定性 case count。它不接受 mutable latest state，不调用 evaluator，也不持久化 evidence；
这些动作仍由既有 execution service 负责，并须由单独准入的 workflow 决定。
