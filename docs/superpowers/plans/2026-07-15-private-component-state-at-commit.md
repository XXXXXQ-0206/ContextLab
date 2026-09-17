# Private Component State at Commit Plan / 私有 Component 提交状态计划

**Goal / 目标：** Add a private reusable storage contract that reconstructs one component's replayable state at a target normal first-parent commit: existence, kind, name, metadata, current content hash, creation commit, and most recent content-change commit.

**Architecture / 架构：** Reuse the existing commit history and guarded-write payloads as the source of truth. `AddedComponent` already records the descriptor and initial hash; `UpdatedComponent` records an auditable hash transition. Memory and PostgreSQL must validate the whole normal first-parent ancestry before they fold those changes from root to target. `GraphDiff` stays the sole graph-diff calculator.

**Tech Stack / 技术栈：** Rust stable, existing `context-core`, `versioning`, `contextlab-storage` repository ports, memory/PostgreSQL adapters, the loopback disposable PostgreSQL 16 harness, and bilingual documentation.

---

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** This increment directly advances completion criteria 1, `Context-first platform coverage`, and 2, `Versioning and diff workflows`. A component body revision can already be resolved at a commit, but a Context-first replay consumer still cannot ask whether that component exists there or obtain its replayable descriptor and effective hash.

**服务条件：** 本增量直接推进完成条件 1“以 Context 为核心的平台覆盖”与条件 2“版本与 Diff 工作流”。当前已能在 commit 解析 component body revision，但 Context-first replay consumer 仍无法查询该 component 在该处是否存在，也无法取得可回放 descriptor 与有效 hash。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口：** `ContextComponentRepository::get_component` returns the current projection, not a historical state. Deriving a target state from that current row would silently leak later name, metadata, or hash values into replay. The completed `0015` parent-scope invariant and body-revision resolver establish a trusted normal first-parent chain, but no reusable adapter contract folds `AddedComponent` and `UpdatedComponent` across it.

**未满足依赖、风险或证据缺口：** `ContextComponentRepository::get_component` 返回的是当前 projection，而不是历史状态。若从当前 row 推导目标状态，会把较晚的 name、metadata 或 hash 静默泄漏到 replay。已完成的 `0015` parent-scope 不变量和 body-revision resolver 已建立可信 normal first-parent chain，但尚无可复用 adapter contract 会沿该链折叠 `AddedComponent` 与 `UpdatedComponent`。

**Why now / 为什么现在优先：** Parent-scope integrity, private component creation, immutable revisions, and fail-closed ancestry validation are verified local prerequisites. This is the smallest dependency-ready follow-up that turns those isolated facts into a usable Context version state without opening an editing or transport surface. It is more direct than benchmark, graph-editor, or public write work because those features need trustworthy historical component state.

**为什么现在优先：** parent-scope integrity、私有 component creation、不可变 revision 与 fail-closed ancestry validation 已具备经过验证的本地前置。该增量是最小的依赖就绪后续项，能把这些孤立事实组合成可用的 Context version state，而不打开编辑或传输表面。它比 benchmark、graph editor 或 public write 更直接，因为这些能力都需要可信的历史 component state。

**Explicit non-goals / 明确非目标：** No public REST route, protected-router promotion, OpenAPI operation, SDK method, Web control, operator transport, migration, component removal write workflow, merge policy, branch UI, body exposure, benchmark feature, or additional graph-diff calculator. Legacy components without a reachable detailed `AddedComponent` are not reconstructed from the current projection; they return no replayable state. External deployment work remains deferred and is not audited or awaited.

**明确非目标：** 不新增 public REST route、protected-router promotion、OpenAPI operation、SDK method、Web control、operator transport、migration、component removal write workflow、merge policy、branch UI、body exposure、benchmark feature 或额外 graph-diff calculator。没有可达 detailed `AddedComponent` 的 legacy component 不会从当前 projection 反推，而是返回没有 replayable state。外部部署工作保持延期，不被审计或等待。

**Minimal affected boundary and bilingual docs / 最小受影响边界与双语文档：** Restrict changes to a storage-private state value/port, `memory.rs`, `postgres.rs`, storage exports/errors and tests, the existing disposable PostgreSQL replay test when it can exercise the same contract, this plan, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, and active roadmap evidence. `server/api`, OpenAPI, TypeScript SDK, Web, and `crates/diff-engine` remain unchanged.

**最小受影响边界与双语文档：** 修改范围仅限 storage-private state value/port、`memory.rs`、`postgres.rs`、storage export/error 与 test；若既有 disposable PostgreSQL replay test 能覆盖同一 contract 则复用它；并更新本计划、`ARCHITECTURE.md`、`docs/storage/persistence-foundation.md` 与活跃路线图证据。`server/api`、OpenAPI、TypeScript SDK、Web 与 `crates/diff-engine` 保持不变。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** First observe a focused memory test fail because the state port does not exist. Then prove root-to-target creation/update folding, a target before creation returning `None`, no current-projection leakage, missing/malformed descriptors failing closed, unknown Context/commit and merge/cycle/cross-Context ancestry rejection, and memory/PostgreSQL parity. Run the registered isolated PostgreSQL case after a loopback schema reset, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, shell guards, and public-surface/`GraphDiff` boundary searches. All evidence remains local and non-production.

**下一增量前的新鲜验证：** 先观察聚焦 memory test 因 state port 不存在而失败。随后证明从 root 到 target 的 creation/update folding、creation 前目标返回 `None`、不会泄漏当前 projection、缺失或损坏 descriptor 会 fail closed，以及 unknown Context/commit 与 merge/cycle/cross-Context ancestry 会被拒绝，并验证 memory/PostgreSQL parity。运行已登记的 isolated PostgreSQL case（在 loopback schema reset 后）、`cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web`、shell guard 与 public-surface/`GraphDiff` boundary search。所有证据仍仅限本地、非生产环境。

## Tasks / 任务

### Task 1: Define replayable component state / 定义可回放 component 状态

- [x] Add a private `ComponentStateAtCommit` value and storage port that preserves target, creation, and last-content-change commit identities.
- [x] 增加私有 `ComponentStateAtCommit` value 与 storage port，保留 target、creation 与最近 content-change commit identity。
- [x] Write a failing domain/storage test for root-to-target folding before implementation.
- [x] 在实现前编写 root-to-target folding 的失败 domain/storage test。

### Task 2: Implement fail-closed adapters / 实现 fail-closed adapter

- [x] Reuse or extract normal first-parent ancestry validation in the memory adapter before folding changes.
- [x] 在 folding change 前复用或提取 memory adapter 中的 normal first-parent ancestry validation。
- [x] Add PostgreSQL replay through the same validated recursive-history semantics and parse only stored change payloads.
- [x] 通过同一已验证的 recursive-history 语义增加 PostgreSQL replay，并且只解析已存储的 change payload。
- [x] Preserve `None` for a known target without a reachable detailed creation and reject malformed transitions without guessing.
- [x] 已知 target 没有可达 detailed creation 时保持 `None`，并拒绝损坏 transition，不进行猜测。

### Task 3: Prove parity and preserve boundaries / 证明一致性并保持边界

- [x] Extend the existing local disposable replay case only if it directly covers the new PostgreSQL contract; otherwise register one distinct reset-per-test case.
- [x] 仅当既有本地 disposable replay case 能直接覆盖新 PostgreSQL contract 时扩展它；否则登记一个独立的 reset-per-test case。
- [x] Document the private historical-state boundary in English and Chinese.
- [x] 以中英双语记录私有历史状态边界。
- [x] Run all fresh verification and update the active roadmap without treating the slice as project completion.
- [x] 运行全部新鲜验证并更新活跃路线图，不把该切片当作项目完成。

## Fresh Local Evidence / 新鲜本地证据

Observed on 2026-07-15: the focused route-classification and in-memory parity tests ran red before their minimal fixes and green afterwards. `cargo fmt --all -- --check`, `cargo test --workspace`, and `pnpm check:web` passed. The disposable-storage shell guard passed, and a new isolated local PostgreSQL 16 container completed all 25 reset-per-test cases, including `postgres_component_content_at_commit_replays_nearest_normal_parent_revision`, which exercises the component-state contract. A boundary search found no component-state route, OpenAPI, SDK, or Web entrypoint, and `GraphDiff::between` remains the only graph-diff calculation entrypoint. This is local non-production evidence only; no remote CI, operator rehearsal, public promotion, release, or production claim is made.

2026-07-15 已观察到：聚焦的 route-classification 与内存 parity test 都在最小修复前出现红灯，并在修复后转绿。`cargo fmt --all -- --check`、`cargo test --workspace` 与 `pnpm check:web` 均已通过。disposable-storage shell guard 已通过；一个新的隔离本地 PostgreSQL 16 container 完成了全部 25 个逐例 reset case，其中 `postgres_component_content_at_commit_replays_nearest_normal_parent_revision` 覆盖 component-state contract。边界搜索未发现 component-state route、OpenAPI、SDK 或 Web entrypoint，且 `GraphDiff::between` 仍是唯一的 graph-diff calculation entrypoint。这仅是本地非生产证据；未声明 remote CI、operator rehearsal、public promotion、release 或生产环境已通过。
