# Private Workflow Execution Status PostgreSQL Repository / 私有 Workflow 执行状态 PostgreSQL Repository

## Necessity Record / 必要性记录

### Named criterion and charter principle / 对应完成条件与宪章原则

This increment directly serves Criteria 1 and 8: Context history and workflow execution
receipts must be replayable and backed by durable, scope-safe persistence. The reusable Rust
domain projection remains the source of truth; PostgreSQL is only its private storage adapter.

本增量直接服务条件 1 与 8：Context 历史与 Workflow execution receipt 必须可回放，并由持久化、范围安全的存储支撑。可复用 Rust domain projection 仍是唯一事实来源；PostgreSQL 只是其私有 storage adapter。

### Gap, dependencies, and evidence / 缺口、依赖与证据

`WorkflowExecutionStatusRepository` currently has only a deterministic in-memory implementation.
The local API/BFF/Web path can therefore read a fixture but cannot retain an execution status
across process restarts. `PostgresContextGraphRepository` already owns the SQLx pool and the
workflow binding persistence boundary, but it has no implementation for this exact projection.
The missing dependency is a schema-versioned, append-only table plus an adapter that validates
the exact `(ContextId, WorkflowRunId)` scope, preserves immutable create/replay/conflict
semantics, and rehydrates the same redacted projection without reading raw events.

当前 `WorkflowExecutionStatusRepository` 只有确定性的 in-memory 实现。因此 local API/BFF/Web path
只能读取 fixture，无法跨进程保留 execution status。`PostgresContextGraphRepository` 已经拥有 SQLx
pool 与 workflow binding persistence boundary，但尚未实现此 projection 的存储。缺失依赖是：一个
schema-versioned、append-only table，以及一个能校验 exact `(ContextId, WorkflowRunId)` scope、保持
immutable create/replay/conflict 语义，并在不读取 raw event 的情况下还原同一脱敏 projection 的 adapter。

### Why now / 为何现在优先

The private producer, API adapter, local SDK, BFF, and shared Web screen are already green and
the in-memory contract is complete. Durable status storage is the nearest dependency-ready gap
on that existing vertical slice and directly supplies the persistent-storage evidence needed by
the completion criteria. It is therefore admitted before new workflow UI, provider, scheduler,
benchmark, or plugin features.

私有 producer、API adapter、local SDK、BFF 与共享 Web screen 已通过验证，in-memory contract 也已完整。
durable status storage 是该既有 vertical slice 上最近且依赖已满足的缺口，直接补充 completion criteria
所需的 persistent-storage evidence。因此它优先于新增 workflow UI、provider、scheduler、benchmark 或 plugin 功能。

### Explicit non-goals / 明确非目标

- No execution-start route, public REST/OpenAPI/public SDK write method, Web mutation, scheduler, provider call, operator transport, release, or production-readiness claim.
- No raw execution-event or failure-payload table; only `WorkflowExecutionStatusProjectionV1` is persisted and read.
- No automatic migration runner, production migration rehearsal, or fabricated PostgreSQL/Docker receipt. A live database result is recorded only when actually observed.
- No second graph-diff calculator; `GraphDiff::between` remains the sole graph-diff calculator.

- 不新增 execution-start route、public REST/OpenAPI/public SDK write method、Web mutation、scheduler、provider call、operator transport、release 或 production-readiness 声明。
- 不新增 raw execution-event 或 failure-payload table；只持久化和读取 `WorkflowExecutionStatusProjectionV1`。
- 不新增 automatic migration runner、生产 migration rehearsal，也不伪造 PostgreSQL/Docker 回执；只有实际观测到的 live database 结果才会被记录。
- 不新增第二个 graph-diff calculator；`GraphDiff::between` 仍是唯一 graph-diff calculator。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

- `crates/storage/src/postgres_workflow_execution_status.rs` owns the SQLx adapter implementation; the existing workflow status trait and domain validation remain unchanged.
- `crates/storage/migrations/0024_workflow_execution_status.sql`, `crates/storage/src/lib.rs`, and a focused migration contract test own the schema asset and append-only invariants.
- `crates/storage` focused tests own exact-scope, create/replay/conflict, and projection rehydration coverage; ignored live PostgreSQL tests are evidence-gated and never treated as passed without output.
- This plan, `docs/architecture/private-workflow-execution-status-postgres.md`, and bilingual roadmap/completion receipts are required; no public API or SDK documentation changes are in scope.

- `crates/storage/src/postgres_workflow_execution_status.rs` 负责 SQLx adapter；既有 workflow status trait 与 domain validation 不变。
- `crates/storage/migrations/0024_workflow_execution_status.sql`、`crates/storage/src/lib.rs` 与 focused migration contract test 负责 schema asset 与 append-only invariants。
- `crates/storage` focused tests 负责 exact-scope、create/replay/conflict 与 projection rehydration；ignored live PostgreSQL test 只有在实际输出后才算 evidence。
- 本计划、`docs/architecture/private-workflow-execution-status-postgres.md` 与双语 roadmap/completion receipts 是必需文档；不修改 public API 或 SDK 文档。

### Fresh verification before the next increment / 下一增量前的新鲜验证

Observe a red compile/static contract test for the missing PostgreSQL adapter or migration first.
Then require green focused storage tests, `cargo fmt --all -- --check`, offline workspace tests,
strict offline Clippy, locked Rust 1.85 check, `pnpm check:web`, the scoped local contract
verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. Run the named PostgreSQL integration test only if a
non-Docker local PostgreSQL runtime is available; otherwise record it as `unobserved` and keep
the external release/production evidence deferred.

先观测 PostgreSQL adapter 或 migration 缺失导致的 focused compile/static contract 红灯；之后要求
storage focused tests、`cargo fmt --all -- --check`、offline workspace tests、strict offline Clippy、锁定
Rust 1.85 check、`pnpm check:web`、范围化 local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1` 全部通过。
只有本地存在非 Docker PostgreSQL runtime 时才运行指定 integration test；否则记录为 `unobserved`，外部
release/production evidence 继续延期。

### Root-cause correction observed during verification / 验证期间观测到的根因修复

The first fresh local PostgreSQL run reached the composed migration and failed before the new
table was exercised: migrations `0016` and `0022` both attempted to create
`uq_contexts_project_id_id`. The minimum correction is to keep the constraint in its original
owner migration `0016` and remove the duplicate declaration from `0022`; no data model or runtime
adapter behavior changes. A static count regression and a fresh empty-cluster rerun are required
before this increment can be accepted.

首次新鲜本地 PostgreSQL run 已进入 composed migration，但在新表执行前失败：`0016` 与 `0022` 都尝试创建
`uq_contexts_project_id_id`。最小修复是保留原 owner migration `0016` 中的 constraint，并从 `0022` 删除重复声明；
不改变 data model 或 runtime adapter behavior。必须增加静态计数回归，并在 fresh empty cluster 上重新运行后，才能接收本增量。

## Execution Checklist / 执行清单

- [x] Add the private PostgreSQL adapter without changing the domain contract.
- [x] Add migration `0024` and append-only/static schema tests.
- [x] Add focused adapter tests and observe live PostgreSQL evidence when the runtime is available.
- [x] Correct the duplicate migration constraint exposed by the first live run and rerun the fresh-cluster test.
- [x] Add bilingual architecture and roadmap receipts, then select the next dependency-ready increment while keeping the long-term goal active.

- [x] 增加私有 PostgreSQL adapter，不改变 domain contract。
- [x] 增加 `0024` migration 与 append-only/static schema tests。
- [x] 增加 focused adapter tests；runtime 可用时观测 live PostgreSQL evidence。
- [x] 修复首次 live run 暴露的重复 migration constraint，并在 fresh cluster 上重跑测试。
- [x] 增加双语架构与 roadmap 回执，然后选择下一条依赖就绪增量并保持长期目标 active。

## Observed Receipt / 已观测回执（2026-08-01）

The first fresh empty-cluster run was red because composed migrations `0016` and `0022` both
created `uq_contexts_project_id_id`. The minimum fix removed the duplicate from `0022`, added a
static one-owner regression, and preserved `0016` as the owner. The corrected empty PostgreSQL
16.14 cluster then ran the opt-in integration test with `1 passed`, covering migration and seed,
`Created`, identical `Replayed`, cross-connection read, and immutable conflict.

首次 fresh empty-cluster run 为红灯，因为 composed migrations `0016` 与 `0022` 都创建了
`uq_contexts_project_id_id`。最小修复从 `0022` 删除重复声明，增加 one-owner 静态回归，并保留 `0016`
作为 owner。修复后的 PostgreSQL 16.14 空集群重新运行 opt-in integration test，真实 `1 passed`，覆盖
migration/seed、`Created`、相同 `Replayed`、跨连接读取与 immutable conflict。

Focused local evidence / focused 本地证据：

- `cargo test -p contextlab-storage postgres_workflow_execution_status --offline`: `3 passed`.
- `cargo test -p contextlab-storage --test workflow_execution_status_migration --offline`: `3 passed`.
- `cargo test -p contextlab-storage --test workflow_execution_status_postgres --offline`: `1 ignored` without a configured database URL; the same test was separately run against the temporary loopback PostgreSQL 16.14 cluster and passed.
- `cargo fmt --all -- --check`: passed after the transaction rollback correction.

本地 focused 证据：

- `cargo test -p contextlab-storage postgres_workflow_execution_status --offline`：`3 passed`。
- `cargo test -p contextlab-storage --test workflow_execution_status_migration --offline`：`3 passed`。
- 未配置 database URL 时 `workflow_execution_status_postgres` 为 `1 ignored`；同一 test 随后在临时 loopback PostgreSQL 16.14 cluster 上独立运行并通过。
- transaction rollback correction 后 `cargo fmt --all -- --check` 通过。

This is local, private, non-Docker, provider-free evidence. Remote CI, operator rehearsal,
production migration, release, public write, and production-readiness remain unobserved or
deferred; the long-term goal remains active.

这是 local、private、non-Docker、provider-free 证据。remote CI、operator rehearsal、production migration、
release、public write 与 production-readiness 继续为 unobserved 或 deferred；长期目标保持 active。
