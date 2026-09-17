# Private Replay and Context Graph Consistency / 私有回放与 Context Graph 一致性

## Necessity Record / 必要性记录

### Criterion and charter principle / 对应完成条件与宪章原则

- **Criterion 1 / 条件 1:** Context changes remain replayable, version-backed, and reviewable at an exact commit.
  Context 变更必须能在 exact commit 上重放，并可审阅版本历史。
- **Criterion 2 / 条件 2:** The reusable Rust core must preserve one authoritative Context Graph representation across storage and versioning.
  可复用 Rust 核心必须在 storage 与 versioning 之间保持唯一、权威的 Context Graph 表示。
- **Charter / 宪章:** Context is the primary abstraction; graph state and replay history are immutable, deterministic, and fail closed.
  Context 是首要抽象；图状态与回放历史必须不可变、确定性且 fail closed。

### Unmet dependency or evidence gap / 未满足依赖或证据缺口

`ReplayState` reconstructs the descriptor and `Uses` relationship state from commit history, while
`CommitGraphSnapshot` reconstructs the materialized graph for the same commit. The lifecycle read path
currently validates component witnesses independently but does not prove that these two authoritative
representations agree. A storage corruption or projection drift could therefore be returned as a
partially valid local Context state.

`ReplayState` 从 commit history 重建 descriptor 与 `Uses` relationship 状态，`CommitGraphSnapshot` 重建同一 commit
的 materialized graph。生命周期读取路径目前只独立校验 component witness，没有证明两份权威表示一致；因此 storage
损坏或投影漂移可能以“部分有效”的本地 Context 状态返回。

### Why this increment now / 为什么现在优先

The exact-commit replay adapter and immutable graph snapshot repository are already present and locally
tested. This validator is the smallest dependency-ready increment that turns those parallel contracts
into one fail-closed read invariant before adding another consumer, editor surface, or public contract.

exact-commit replay adapter 与 immutable graph snapshot repository 已存在并有本地测试。本校验器是当前最小、依赖已满足
的增量：在继续增加 consumer、编辑器或 public contract 前，把两条并行 contract 收束为一个 fail-closed read invariant。

### Minimal boundary and ownership / 最小边界与所有权

- Add one storage-private validator and focused storage tests.
  新增一个 storage-private validator 及 focused storage tests。
- Invoke it from `ContextLifecycleService::read_state_at_commit` after both exact-commit reads.
  在 `ContextLifecycleService::read_state_at_commit` 完成两次 exact-commit 读取后调用。
- Compare Context/commit identity, schema versions, the Context node, component node identity/kind/name,
  context-to-component containment edge, replayed `Uses` edges, duplicate edges, and unexpected graph nodes/edges.
  比较 Context/commit identity、schema version、Context node、component node identity/kind/name、Context 到 component
  的 containment edge、回放出的 `Uses` edge、重复 edge 以及多余 graph node/edge。
- Reuse existing `StoredComponentKind` mapping and graph types; do not calculate a diff.
  复用既有 `StoredComponentKind` 映射与 graph types，不计算 Diff。
- Files owned by this increment: `crates/storage/src/replay_graph_consistency.rs`,
  `crates/storage/src/lib.rs`, `crates/storage/src/context_lifecycle.rs`, and the focused storage test module.
  本增量所有权文件如上，禁止与其他 worker 共享核心文件编辑。

### Explicit non-goals / 明确非目标

- No REST, OpenAPI, SDK, BFF, Web, CLI, Desktop, or mutation changes.
  不新增 REST、OpenAPI、SDK、BFF、Web、CLI、Desktop 或 mutation。
- No commit writer, merge, branch mutation, rollback, migration, provider, or new persistence adapter.
  不新增 commit writer、merge、branch mutation、rollback、migration、provider 或新的 persistence adapter。
- No PostgreSQL runtime, Docker, remote CI, operator, release, production, or secret work.
  不进行 PostgreSQL runtime、Docker、remote CI、operator、release、production 或 secret 工作。
- `GraphDiff::between` remains the sole graph-diff calculator; this validator only checks equality of
  already materialized facts.
  `GraphDiff::between` 仍是唯一 graph-diff calculator；本校验器只检查已 materialize facts 的一致性。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

- Focused storage consistency tests cover the matching graph, scope/schema mismatch, missing/conflicting
  component nodes, missing/conflicting containment and `Uses` edges, duplicate edges, and unexpected elements.
  focused storage consistency tests 覆盖匹配图、scope/schema mismatch、缺失/冲突 component node、缺失/冲突 containment
  与 `Uses` edge、重复 edge 及多余元素。
- `cargo fmt --all -- --check`, focused storage tests, `cargo test --workspace --quiet`, strict offline
  Clippy, locked Rust check, and `pnpm check:web` are rerun with observed output.
  重新运行 format、focused storage、workspace tests、strict offline Clippy、锁定 Rust check 与 Web check，并记录真实输出。
- PostgreSQL/Docker runtime, authenticated browser, Git, remote CI, operator, release, and production
  evidence remain `unobserved` or `deferred`, never locally inferred.
  PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator、release 与 production evidence 继续标为
  `unobserved` 或 `deferred`，不得由本地结果推断。

## Implementation checklist / 实施清单

- [x] Add red tests for exact replay/snapshot consistency.
- [x] Implement the storage-private validator.
- [x] Wire validation into exact commit lifecycle reads.
- [x] Run focused and workspace verification.
- [x] Update the bilingual roadmap/audit and choose the next dependency-ready increment.

## Receipt and root-cause note / 回执与根因记录

The first workspace compile after wiring the new port exposed one real adapter gap:
server/api/src/lib.rs had not forwarded ContextReplayStateAtCommitRepository through
WorkspaceDataRepository. The Integration Lead added that disjoint adapter forwarding and
reran the affected API and workspace gates. An existing lifecycle test also showed that
component-specific graph errors must retain precedence; the validator therefore runs after
the existing component witness checks and adds only aggregate consistency failures.

接入新 port 后第一次 workspace compile 发现一个真实 adapter 缺口：server/api/src/lib.rs 的
WorkspaceDataRepository 没有转发 ContextReplayStateAtCommitRepository。Integration Lead 补齐了该不重叠的
adapter forwarding，并重跑受影响的 API 与 workspace gates。一个既有 lifecycle test 同时证明 component-specific
graph error 必须保持优先，因此 validator 在现有 component witness 校验之后运行，只新增 aggregate consistency failure。

Fresh receipts / 新鲜回执：focused consistency 6 passed；lifecycle 9 passed；storage
199 passed, 39 ignored；API 189 passed；workspace Rust passed；cargo fmt check；
strict offline Clippy；locked Rust 1.85.0 check；pnpm check:web public SDK 15、local SDK
99、Web 207、TypeScript/lint 与 production build；static GRAPH_DIFF_IMPL_COUNT=1。

PostgreSQL/Docker runtime, authenticated browser, Git, remote CI, operator rehearsal, release,
and production remain unobserved or deferred; no local result substitutes for them. The
long-term goal remains active.

PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator rehearsal、release 与 production 继续为
unobserved 或 deferred；任何本地结果都不替代这些证据。长期目标保持 active。
