# Private Workflow Execution Status Repository / 私有 Workflow 执行状态 Repository

## Necessity Record / 必要性记录

### Named criterion and charter principle / 对应完成条件与宪章原则

This increment directly serves Criteria 1 and 5: Context-first version history must be
replayable, and workflow execution state must remain a reusable, provider-free Rust contract.
The existing workflow domain already validates execution logs and replay provenance; the
read-only local API and Web path must be able to consume an exact stored projection without
exposing raw events or failure payloads.

本增量直接服务条件 1 与 5：Context-first 版本历史必须可回放，Workflow 执行状态必须保持可复用、无 provider 依赖的 Rust contract。现有 Workflow domain
已经校验 execution log 与 replay provenance；本地只读 API 与 Web path 需要能够消费精确存储的 projection，同时不暴露 raw event 或 failure payload。

### Gap, dependencies, and evidence / 缺口、依赖与证据

The workflow crate has `WorkflowExecutionStatusProjectionV1`, but `server/api` exposes only a
typed `UnavailableWorkflowExecutionStatusAdapter`. No storage crate repository contract or
idempotent in-memory producer exists, so the completed read path has no local execution fact to
read. The dependency-ready minimum is an immutable projection repository with exact
`(ContextId, WorkflowRunId)` scope, create/replay/conflict semantics, plus an API adapter. The
default application remains unavailable until an explicit repository is injected.

Workflow crate 已有 `WorkflowExecutionStatusProjectionV1`，但 `server/api` 目前只有 typed
`UnavailableWorkflowExecutionStatusAdapter`。storage crate 没有 repository contract 或幂等的内存 producer，因此已完成的 read path 没有本地 execution fact 可读。依赖就绪的最小增量是：提供 exact
`(ContextId, WorkflowRunId)` scope 的 immutable projection repository、create/replay/conflict 语义与 API adapter；默认应用在显式注入 repository 前继续返回 unavailable。

### Why now / 为何现在优先

The private status API, local SDK, BFF, and shared Web screen already exist, while workflow
execution domain and capability snapshot validation are already available. Completing their
missing repository boundary is the smallest direct path to a usable local read/replay witness,
and is more necessary than adding another UI surface, public transport, or unrelated roadmap
feature.

私有 status API、local SDK、BFF 与 shared Web screen 已经存在，Workflow execution domain 与 capability snapshot validation 也已具备。补齐缺失的 repository boundary 是形成可用本地 read/replay witness 的最小直接路径，优先级高于增加 UI surface、public transport 或无关路线图功能。

### Explicit non-goals / 明确非目标

- No execution start route, public REST/OpenAPI/public SDK write method, Web mutation, provider
  call, scheduler, PostgreSQL migration, operator transport, or production-readiness claim.
- No raw event or failure payload persistence/read surface; `WorkflowExecutionStatusProjectionV1`
  remains the only stored read shape.
- No second graph-diff calculator; `GraphDiff::between` remains the sole graph-diff calculator.

- 不新增 execution start route、public REST/OpenAPI/public SDK write method、Web mutation、provider call、scheduler、PostgreSQL migration、operator transport 或 production-readiness 声明。
- 不持久化或读取 raw event、failure payload；`WorkflowExecutionStatusProjectionV1` 仍是唯一存储读取形状。
- 不新增第二个 graph-diff calculator；`GraphDiff::between` 仍是唯一 graph-diff calculator。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

- `crates/storage/src/workflow_execution_status.rs` and focused repository tests own the
  framework-independent storage contract and deterministic in-memory implementation.
- `server/api/src/workflow_execution.rs` and focused API tests own the adapter from the storage
  projection to the existing redacted resource.
- This plan, `docs/architecture/private-workflow-execution-status.md`, and the bilingual
  roadmap/completion receipt are the required documentation; no public API or SDK documentation
  changes are in scope.

- `crates/storage/src/workflow_execution_status.rs` 及其 focused repository tests 负责 framework-independent storage contract 与确定性内存实现。
- `server/api/src/workflow_execution.rs` 及 focused API tests 负责从 storage projection 到既有脱敏 resource 的 adapter。
- 本计划、`docs/architecture/private-workflow-execution-status.md` 与双语 roadmap/completion receipt 是必需文档；不修改 public API 或 SDK 文档。

### Fresh verification before the next increment / 下一增量前的新鲜验证

First observe focused red tests showing the missing repository/adapter behavior. Then observe
green storage and API tests for create, identical replay, conflicting reuse, exact Context/run
scope rejection, and redacted serialization. Run format, the offline workspace test suite,
strict offline Clippy, locked Rust 1.85 check, `pnpm check:web`, the scoped local contract
verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. Docker/PostgreSQL runtime, browser, Git, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`.

先观察 focused red test 证明 repository/adapter behavior 缺失；再观察 storage 与 API green test 覆盖 create、相同 replay、冲突复用、exact Context/run scope rejection 与脱敏序列化。运行 format、offline workspace test、strict offline Clippy、锁定 Rust 1.85 check、`pnpm check:web`、范围化 local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。Docker/PostgreSQL runtime、browser、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

## Execution Checklist / 执行清单

- [x] Add the storage projection repository contract and deterministic in-memory implementation.
- [x] Add the API adapter while preserving default unavailable behavior and exact scope checks.
- [x] Run red/green focused tests and the complete local verification matrix.
- [x] Record bilingual receipts and select the next dependency-ready criterion; keep the goal active.

- [x] 增加 storage projection repository contract 与确定性内存实现。
- [x] 增加 API adapter，同时保持默认 unavailable 与 exact scope checks。
- [x] 运行红绿 focused tests 与完整本地验证矩阵。
- [x] 记录双语回执并选择下一项依赖就绪条件；保持长期目标 active。

## Observed Receipt / 已观测回执（2026-08-01）

The initial focused storage run was red: the newly admitted repository tests failed to compile
because the repository, command, error, and write-result types did not exist. The implementation
adds `WorkflowExecutionStatusRepository`, `InMemoryWorkflowExecutionStatusRepository`, and the
provider-free `WorkflowExecutionStatusService`. The service validates a root or replay log through
`WorkflowExecutionStatusProjectionV1` before persistence. Immutable same-scope writes return
`Created` or identical `Replayed`; conflicting reuse fails closed. The API adapter translates only
the redacted projection and preserves the default `Unavailable` adapter.

首次 focused storage run 为红灯：新增 repository tests 因 repository、command、error 与 write-result types 尚不存在而无法编译。实现现已增加
`WorkflowExecutionStatusRepository`、`InMemoryWorkflowExecutionStatusRepository` 与 provider-free `WorkflowExecutionStatusService`。service 在持久化前通过
`WorkflowExecutionStatusProjectionV1` 校验 root/replay log。同 scope 的不可变写入返回 `Created` 或相同的 `Replayed`，冲突复用 fail closed。API adapter 只转换脱敏 projection，并保持默认 `Unavailable` adapter。

Fresh local verification / 新鲜本地验证：

- Storage focused green: `3 passed`; API execution adapter focused green: `6 passed`.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage `215 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `135`, Web `284`, TypeScript/lint, and production build.
- `scripts/verify-local-contracts.ps1`: scoped source/graph/DTO/route checks passed; `overall=unobserved` without unified diff input.
- `GRAPH_DIFF_IMPL_COUNT=1`: passed.

新鲜本地验证：

- storage focused green `3 passed`；API execution adapter focused green `6 passed`。
- `cargo fmt --all -- --check` 通过。
- `cargo test --workspace --quiet --no-fail-fast --offline` 通过；storage `215 passed, 39 ignored`。
- `cargo clippy --workspace --all-targets --offline -- -D warnings` 通过。
- `cargo +1.85.0 check --workspace --all-targets --locked --offline` 通过。
- `pnpm check:web` 通过；public SDK `15`、local SDK `135`、Web `284`，TypeScript/lint 与 production build 均通过。
- `scripts/verify-local-contracts.ps1` 的 scoped source/graph/DTO/route checks 通过；因未提供 unified diff input，`overall=unobserved`。
- `GRAPH_DIFF_IMPL_COUNT=1` 通过。

The receipt is local, private, provider-free, and in-memory. It adds no execution start route,
public REST/OpenAPI/public SDK write, Web mutation, scheduler, migration, provider, secret access,
operator transport, or production claim. PostgreSQL/Docker runtime, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`. The long-term goal remains active; the next increment requires a fresh bilingual
Necessity Record for the next dependency-ready named criterion.

本回执是 local、private、provider-free 且 in-memory 的。未新增 execution start route、public REST/OpenAPI/public SDK write、Web mutation、scheduler、migration、provider、secret access、operator transport 或 production 声明。PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active；下一增量必须先为下一项依赖就绪的命名条件新增双语 Necessity Record。
