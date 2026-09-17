# Private Workflow Execution Status Application Composition / 私有 Workflow 执行状态应用组合

## Necessity Record / 必要性记录

### Named criterion and charter principle / 对应完成条件与宪章原则

This increment directly serves Criteria 1 and 8: a versioned Context workflow receipt must be
replayable through the reusable Rust storage boundary and consumable by the protected local read
path. The application composition must preserve the Context-first, server-owned, fail-closed
boundary.

本增量直接服务条件 1 与 8：版本化 Context workflow receipt 必须通过可复用 Rust storage boundary
可回放，并能被 protected local read path 消费。应用组合必须保持 Context-first、server-owned 与
fail-closed 边界。

### Gap, dependencies, and evidence / 缺口、依赖与证据

`PostgresContextGraphRepository` now implements the existing immutable
`WorkflowExecutionStatusRepository`, and the API has a redacted adapter plus an explicit
injection builder. However, `AppState::try_from_env` wires the PostgreSQL repository into other
private storage-backed readers while leaving workflow execution status on the default
`UnavailableWorkflowExecutionStatusAdapter`. A PostgreSQL-configured local application therefore
cannot read the durable status projection through its existing protected route. The dependency
is ready; only server composition and its regression evidence are missing.

`PostgresContextGraphRepository` 已实现既有不可变 `WorkflowExecutionStatusRepository`，API 也有脱敏
adapter 与显式 injection builder。但 `AppState::try_from_env` 虽将 PostgreSQL repository 接入其他
private storage-backed reader，却仍让 workflow execution status 使用默认
`UnavailableWorkflowExecutionStatusAdapter`。因此配置 PostgreSQL 的 local application 无法通过既有
protected route 读取 durable status projection。依赖已经就绪，缺口只在 server composition 与回归证据。

### Why now / 为何现在优先

The previous increment proved the storage contract against a fresh local PostgreSQL cluster. The
nearest remaining gap on the same vertical slice is the already-designed application composition;
leaving it unavailable would make the durable receipt unusable by the existing local read UI.
Wiring this path is more necessary than adding a new Workflow screen, scheduler, provider, or
benchmark surface.

上一增量已经在 fresh local PostgreSQL cluster 上证明 storage contract。当前同一 vertical slice 上最近的
缺口是已经设计好的 application composition；继续保持 unavailable 会使 durable receipt 无法被现有 local
read UI 使用。因此接线优先于新增 Workflow screen、scheduler、provider 或 benchmark surface。

### Explicit non-goals / 明确非目标

- No execution-start route, public REST/OpenAPI/public SDK write method, Web mutation, polling, scheduler, provider, operator transport, or production-readiness claim.
- No change to `WorkflowExecutionStatusRepository`, `GraphDiff::between`, projection schema, migration, or raw-event persistence.
- Memory mode remains explicitly unavailable for execution status unless a caller injects a repository; PostgreSQL mode alone gains the durable private reader.

- 不新增 execution-start route、public REST/OpenAPI/public SDK write method、Web mutation、polling、scheduler、provider、operator transport 或 production-readiness 声明。
- 不改变 `WorkflowExecutionStatusRepository`、`GraphDiff::between`、projection schema、migration 或 raw-event persistence。
- Memory mode 在调用方显式注入 repository 之前继续明确 unavailable；只有 PostgreSQL mode 获得 durable private reader。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

- `server/api/src/lib.rs` owns one composition assignment from the existing PostgreSQL runtime repository to the existing status storage adapter.
- `server/api` focused tests own the mode-specific injection and memory-unavailable regression.
- This plan, the active goal/completion/parallel receipts, and the existing private PostgreSQL architecture note are the required documentation; no API/OpenAPI/SDK/Web contract changes are in scope.

- `server/api/src/lib.rs` 只负责将既有 PostgreSQL runtime repository 接入既有 status storage adapter。
- `server/api` focused tests 负责 mode-specific injection 与 memory-unavailable regression。
- 本计划、active goal/completion/parallel receipt 与既有 private PostgreSQL architecture note 是必需文档；不修改 API/OpenAPI/SDK/Web contract。

### Fresh verification before the next increment / 下一增量前的新鲜验证

First observe a red API composition test showing PostgreSQL mode still reports the status reader
as unavailable. Then require the focused API regression, workspace Rust, format, strict offline
Clippy, locked Rust 1.85 check, `pnpm check:web`, local contract verifier, and
`graph_diff_application=passed count=1`. Re-run the protected read against the temporary local
PostgreSQL cluster only if the test can use the same non-production fixture; otherwise record it
as unobserved. Do not use or expose secrets.

先观测 API composition test 的红灯，证明 PostgreSQL mode 仍报告 status reader unavailable；再要求 focused
API regression、workspace Rust、format、strict offline Clippy、锁定 Rust 1.85 check、`pnpm check:web`、local
contract verifier 与 `graph_diff_application=passed count=1`。只有 test 能复用同一非生产 fixture 时才在临时
local PostgreSQL cluster 上重跑 protected read；否则记录为 unobserved。不得使用或暴露 secrets。

## Execution Checklist / 执行清单

- [x] Add the PostgreSQL-mode application composition assignment.
- [x] Add red/green API composition regression while preserving memory unavailable behavior.
- [x] Run the local quality matrix and update bilingual receipts; keep the long-term goal active.

- [x] 增加 PostgreSQL-mode application composition assignment。
- [x] 增加 red/green API composition regression，同时保持 memory unavailable behavior。
- [x] 运行本地质量矩阵并更新双语回执；保持长期目标 active。

## Completion receipt / 完成回执

The application composition is locally verified. `postgres_mode_wires_a_storage_backed_workflow_execution_status_reader`, `memory_mode_keeps_workflow_execution_status_unavailable`, and `workflow_execution_status_backend_tracks_builder_overrides` each passed as focused API tests. A review found that an independent boolean could drift from the repository trait object, so the smallest repair replaced it with one `Unavailable`/`Custom`/`StorageBacked` backend state. The PostgreSQL-mode assertion now checks that state rather than only `Option::is_some()`. The memory-mode test continues to exercise the typed unavailable reader and fail-closed error.

本应用组合已完成本地验证。`postgres_mode_wires_a_storage_backed_workflow_execution_status_reader`、`memory_mode_keeps_workflow_execution_status_unavailable` 与 `workflow_execution_status_backend_tracks_builder_overrides` 三个 focused API test 均通过。审查发现独立 boolean 可能与 repository trait object 漂移，因此最小修复改为单一 `Unavailable`/`Custom`/`StorageBacked` backend state。PostgreSQL-mode assertion 现在检查该状态，而不是仅检查 `Option::is_some()`；memory-mode test 继续覆盖 typed unavailable reader 与 fail-closed error。

Fresh commands / 新鲜命令：

- `cargo test -p contextlab-api postgres_mode_wires_a_storage_backed_workflow_execution_status_reader --lib --offline` -> `1 passed`.
- `cargo test -p contextlab-api memory_mode_keeps_workflow_execution_status_unavailable --lib --offline` -> `1 passed`.
- `cargo test -p contextlab-api workflow_execution_status_backend_tracks_builder_overrides --lib --offline` -> `1 passed`.

The focused tests prove composition and mode isolation only. A protected read against the same temporary live PostgreSQL fixture was not separately observed in this increment and remains `unobserved`; the lazy URL test must not be treated as a database-runtime receipt. The long-term goal remains `active`. No public REST/OpenAPI/SDK write, Web mutation, operator transport, secret access, second `GraphDiff` calculator, Docker, release, or production claim is made.

focused test 只证明 composition 与 mode isolation。本增量没有单独观测同一临时 live PostgreSQL fixture 上的 protected read，继续标记为 `unobserved`；lazy URL test 不得被当作 database-runtime 回执。长期目标保持 `active`。不声称新增 public REST/OpenAPI/SDK write、Web mutation、operator transport、secret access、第二个 `GraphDiff` calculator、Docker、release 或 production 能力。
