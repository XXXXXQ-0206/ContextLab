# Private Component Content Replay Witness Integrity
# 私有组件正文回放 Witness 一致性

## Necessity Record / 必要性记录

### Criterion and charter principle / 对应条件与宪章原则

- Criterion 1, Context-first platform coverage: a Context component must have a
  durable domain/repository contract and a verifiable inspection path at an
  exact commit.
- Criterion 2, Versioning and diff workflows: the effective component body and
  its version history must be replayable without accepting contradictory
  witnesses.
- Charter boundary: reusable Rust domain/storage contracts remain the source
  of truth; Web and adapters must not reconstruct lifecycle semantics.

- 条件 1，以 Context 为核心的平台覆盖：Context component 必须具备持久化
  domain/repository contract，并能在 exact commit 上获得可验证的 inspection path。
- 条件 2，版本与 Diff 工作流：组件生效正文及其版本历史必须可回放，不能接受
  相互矛盾的 witness。
- 宪章边界：可复用 Rust domain/storage contract 是唯一事实源；Web 与 adapter
  不得重新构造生命周期语义。

### Gap and dependency / 缺口与依赖

The existing guarded lifecycle already persists immutable component body
revisions, replays `ReplayState`, and reads exact graph snapshots. The exact
commit read checks the body hash and component kind, but does not explicitly
prove that the body revision's recording commit equals the replayed state's
`content_commit_id`. A corrupted or mismatched repository adapter could
therefore satisfy the current hash check while returning a body from another
commit. The required dependencies are already present: `ComponentContentRevision`,
`ComponentStateAtCommit`, `ContextLifecycleService::read_state_at_commit`, and
Memory/PostgreSQL repository contracts.

现有 guarded lifecycle 已持久化 immutable component body revision、回放
`ReplayState` 并读取 exact graph snapshot。exact commit read 已检查 body hash 与
component kind，但尚未显式证明 body revision 的记录 commit 等于 replay state 的
`content_commit_id`。因此损坏或漂移的 repository adapter 可能在 hash 检查通过时
返回另一个 commit 的正文。所需依赖已具备：`ComponentContentRevision`、
`ComponentStateAtCommit`、`ContextLifecycleService::read_state_at_commit` 与
Memory/PostgreSQL repository contract。

### Why now / 当前优先级

This is the smallest dependency-ready increment directly closing the named
Context-first/replay integrity gap identified by the 2026-07-30 Criterion 1
audit. It strengthens an existing local read contract before adding another
consumer surface, and it provides evidence needed before claiming component
content update/replay is trustworthy.

这是 2026-07-30 条件 1 审计识别出的、依赖就绪且最小的 Context-first/replay
完整性增量。它先加强既有 local read contract，再增加新的 consumer surface，
并补齐判断 component content update/replay 是否可信所需的证据。

### Non-goals / 明确非目标

- No new writer, mutation route, public REST/OpenAPI/public SDK method, or Web
  mutation control.
- No PostgreSQL runtime, Docker, production, release, remote CI, operator, or
  external receipt work.
- No new graph-diff implementation; `GraphDiff::between` remains the sole
  graph-diff calculator.
- No change to component content policy, merge semantics, or provider behavior.

- 不新增 writer、mutation route、public REST/OpenAPI/public SDK method 或 Web
  mutation control。
- 不进行 PostgreSQL runtime、Docker、production、release、remote CI、operator 或
  external receipt 工作。
- 不新增 graph-diff 实现；`GraphDiff::between` 仍是唯一 graph-diff calculator。
- 不改变 component content policy、merge semantics 或 provider 行为。

### Minimal boundary and bilingual documentation / 最小边界与双语文档

Implementation is limited to the Rust lifecycle read composition and its
focused regression tests in `crates/storage/src/context_lifecycle.rs`.
The bilingual plan is the only new documentation surface; after fresh
verification, the roadmap and parallel-development ledger will receive a
receipt with exact commands and evidence classifications.

实现仅限 `crates/storage/src/context_lifecycle.rs` 的 Rust lifecycle read
composition 与 focused regression tests。双语 plan 是唯一新增文档面；取得新鲜
验证后，再在 roadmap 与 parallel-development ledger 中记录精确命令及证据分类。

### Fresh verification required before the next increment / 下一增量前的新鲜验证

The change is admitted only after:

1. A red regression demonstrates that a body revision from the wrong commit is
   rejected even when its hash and kind match.
2. The focused lifecycle/storage tests pass, including normal create/update/replay
   behavior.
3. `cargo fmt --all -- --check`, workspace Rust tests, strict offline Clippy,
   locked Rust 1.85 check, `pnpm check:web`, and the sole-`GraphDiff` static
   check are freshly observed.
4. Docker/PostgreSQL runtime, browser, Git, remote, operator, release, and
   production evidence remain explicitly `unobserved` or `deferred`.

本增量只有在以下条件满足后才允许选择下一项工作：

1. 先由红色回归证明：即使 hash 与 kind 匹配，来自错误 commit 的 body revision
   也会被拒绝。
2. lifecycle/storage focused tests 通过，并覆盖正常 create/update/replay 行为。
3. 新鲜观察到 `cargo fmt --all -- --check`、workspace Rust tests、strict offline
   Clippy、锁定 Rust 1.85 check、`pnpm check:web` 与唯一 `GraphDiff` 静态检查通过。
4. Docker/PostgreSQL runtime、browser、Git、remote、operator、release 与 production
   evidence 继续明确标记为 `unobserved` 或 `deferred`。

## Implementation checklist / 实施清单

- [x] Add the focused red/green cross-witness regression.
- [x] Enforce the commit-identity invariant in the shared lifecycle read path.
- [x] Run focused and full local verification.
- [x] Record the fresh bilingual receipt and keep the long-term goal active.

- [x] 增加 focused red/green cross-witness regression。
- [x] 在共享 lifecycle read path 强制 commit identity invariant。
- [x] 运行 focused 与完整本地验证。
- [x] 记录新鲜双语回执并保持长期目标 active。
