# Private Memory Writer-to-Review Composition / 私有 Memory Writer-to-Review 组合收束

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与宪章原则

- **Criterion 1 / 条件 1:** a Context commit must atomically preserve its commit, graph snapshot, branch head, idempotency, and derived diff state across the reusable storage boundary.
  / 条件 1：Context commit 必须在可复用 storage boundary 中原子保留 commit、graph snapshot、branch head、idempotency 与派生 diff state。
- **Criterion 2 / 条件 2:** version-backed Context history and diff review must replay the same exact commit state rather than a separately assembled fixture store.
  / 条件 2：版本化 Context history 与 diff review 必须回放同一份精确 commit state，而不是分别组装的 fixture store。
- **Criterion 4 / 条件 4:** the Context Graph and reusable Rust storage contract remain the system backbone; `GraphDiff::between` remains the sole calculator.
  / 条件 4：Context Graph 与可复用 Rust storage contract 继续作为系统骨架；`GraphDiff::between` 继续是唯一 calculator。

### Gap and ready dependencies / 缺口与已满足依赖

The Memory `InMemoryContextGraphRepository` already owns `commit_snapshot_state.diff_snapshots` and implements the diff snapshot port. `try_from_env` also shares one cloned Memory graph repository across the workspace repositories and guarded writer. However, `WorkspaceDataRepository::context_diff_snapshot_repository` currently creates a fresh empty `InMemoryContextDiffSnapshotV1Repository` for Memory, so a commit written through the local writer cannot be read by the persisted diff-review route in the same application composition. Existing storage writer/replay, protected route, and exact-scope contracts are available; the missing evidence is repository identity and one writer-to-review regression.

Memory `InMemoryContextGraphRepository` 已拥有 `commit_snapshot_state.diff_snapshots` 并实现 diff snapshot port。`try_from_env` 也会在 workspace repositories 与 guarded writer 之间共享同一份 cloned Memory graph repository。但 `WorkspaceDataRepository::context_diff_snapshot_repository` 当前为 Memory 新建空的 `InMemoryContextDiffSnapshotV1Repository`，因此同一 application composition 中由 local writer 写入的 commit 无法被 persisted diff-review route 读回。既有 storage writer/replay、protected route 与 exact-scope contract 已满足；缺口是 repository identity 与一条 writer-to-review regression 证据。

### Why now / 为何现在优先

This is the smallest root-cause fix directly exposed by the newly completed boundary witness and the persisted snapshot writer. Without it, the local Context lifecycle claims remain fixture-backed even though the underlying Memory repository already has the correct shared state. Fixing composition now is required before adding another consumer or claiming a usable local replay workflow; it is narrower and earlier than new benchmark, workflow execution, or public transport work.

这是新完成的 boundary witness 与 persisted snapshot writer 直接暴露的最小根因修复。若不修复，底层 Memory repository 虽已有正确共享 state，本地 Context lifecycle 仍只是 fixture-backed。必须先修复组合，才能增加新的 consumer 或声称 local replay workflow 可用；它比新增 benchmark、workflow execution 或 public transport 更窄、更优先。

### Explicit non-goals / 明确非目标

- No new route, public REST/OpenAPI/public SDK write method, Web mutation, migration, provider, secret access, operator transport, PostgreSQL runtime, browser E2E, release, or production claim.
  / 不新增 route、public REST/OpenAPI/public SDK write method、Web mutation、migration、provider、secret access、operator transport、PostgreSQL runtime、browser E2E、release 或 production 声明。
- No new repository abstraction or second diff calculator. Reuse `InMemoryContextGraphRepository` and existing `ContextDiffSnapshotV1Repository`; `GraphDiff::between` remains sole calculator.
  / 不新增 repository abstraction 或第二个 diff calculator。复用 `InMemoryContextGraphRepository` 与既有 `ContextDiffSnapshotV1Repository`；`GraphDiff::between` 仍是唯一 calculator。
- Do not manufacture behavior/evaluation evidence; empty producer collections remain empty unless a real producer supplies redacted facts.
  / 不制造 behavior/evaluation evidence；没有真实 producer 提供脱敏 facts 时，producer collection 继续为空。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

Only `server/api/src/lib.rs`, its focused composition/API tests, this plan, the active goal, completion criteria, parallel ledger, and one bilingual verification receipt are in scope. The storage port, migrations, public API/SDK, Web, OpenAPI, and production configuration remain unchanged.

仅影响 `server/api/src/lib.rs`、其 focused composition/API test、本计划、active goal、completion criteria、parallel ledger 与一份双语验证回执。storage port、migration、public API/SDK、Web、OpenAPI 与 production configuration 保持不变。

### Fresh verification required / 下一增量前的新鲜验证

First observe a red regression proving a commit written through the Memory composition is readable through the persisted diff-review repository, then implement the one-line adapter identity fix and observe green focused API/storage tests. Afterward run `cargo fmt --all -- --check`, offline workspace tests, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, the local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL/Docker runtime, browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

先观察一条 red regression，证明 Memory composition 写入的 commit 能通过 persisted diff-review repository 读回；再实现唯一的 adapter identity 修复并观察 focused API/storage tests 变绿。之后运行 `cargo fmt --all -- --check`、offline workspace tests、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL/Docker runtime、browser、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

## Execution Checklist / 执行清单

- [x] Add the red Memory writer-to-review composition regression.
  / 增加 red Memory writer-to-review composition regression。
- [x] Reuse the shared `InMemoryContextGraphRepository` for the Memory diff snapshot adapter.
  / 让 Memory diff snapshot adapter 复用共享的 `InMemoryContextGraphRepository`。
- [x] Run focused and full local verification and record the evidence boundary.
  / 运行 focused 与完整本地验证并记录证据边界。
