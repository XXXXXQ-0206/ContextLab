# Private Context Component-State Snapshot Plan / 私有 Context Component-State Snapshot 计划

**Goal / 目标：** Add a private reusable storage contract that reconstructs the complete descriptor-only component inventory for one Context at a target normal first-parent commit.

**Architecture / 架构：** Reuse the completed detailed component-state reducer, immutable revision witnesses, and normal first-parent history validation. The snapshot must fold history from root to target exactly once and never enumerate from the current component projection. `GraphDiff` remains the sole graph-diff calculator.

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** This increment directly advances completion criterion 1, `Context-first platform coverage`, and criterion 2, `Versioning and diff workflows`. A Context consumer needs the component collection that existed at one commit, not only a lookup for a component identifier supplied from outside the historical state.

**服务条件：** 本增量直接推进完成条件 1“以 Context 为核心的平台覆盖”与条件 2“版本与 Diff 工作流”。Context consumer 需要得到某个 commit 实际存在的 component collection，而不只是查询外部提供 component identifier 的单项历史 state。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口：** `ContextComponentRepository::list_components` is a current projection. Reusing it would leak later additions, names, metadata, or hashes into an earlier Context replay. The verified single-component port can prove one state but cannot discover which component identifiers should be queried, so repeatedly calling it cannot form a trustworthy Context inventory.

**未满足依赖、风险或证据缺口：** `ContextComponentRepository::list_components` 是当前 projection。复用它会把后续新增的 component、name、metadata 或 hash 泄漏到较早的 Context replay。已验证的单 component port 可以证明一项 state，但无法发现应查询哪些 component identifier，因此反复调用它也不能形成可信的 Context inventory。

**Why now / 为什么现在优先：** Detailed creation payloads, immutable initial and update revisions, complete first-parent validation, and single-component replay are verified local prerequisites. A deterministic inventory is the smallest dependency-ready step that makes the Context, rather than an externally chosen component, the replay subject. It is more direct than public editing, benchmark, workflow, or graph-editor work because each depends on a faithful Context version.

**为什么现在优先：** 带详情的 creation payload、不可变 initial/update revision、完整 first-parent validation 和单 component replay 都是经过验证的本地前置。确定性的 inventory 是最小、依赖就绪的下一步，使 Context 而不是外部选定 component 成为 replay subject。它比 public editing、benchmark、workflow 或 graph-editor 工作更直接，因为这些能力都依赖忠实的 Context version。

**Explicit non-goals / 明确非目标：** No public REST route, OpenAPI operation, SDK method, Web control, component body exposure, guarded write, component-removal workflow, merge policy, migration, operator transport, release work, or additional graph-diff calculator. Targets whose history contains unsupported removal or malformed transitions fail closed; no current projection or graph snapshot is used to fill gaps.

**明确非目标：** 不新增 public REST route、OpenAPI operation、SDK method、Web control、component body exposure、guarded write、component-removal workflow、merge policy、migration、operator transport、release 工作或额外 graph-diff calculator。target 的 history 若包含不受支持的 removal 或损坏 transition，会 fail closed；不使用当前 projection 或 graph snapshot 填补缺口。

**Minimal affected boundary and bilingual docs / 最小受影响边界与双语文档：** Restrict implementation to the private `component_state_at_commit` storage value/port and its memory/PostgreSQL adapters plus focused tests. Update this plan, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/active-long-term-goal.md`, and completion evidence in both languages. `server/api`, OpenAPI, TypeScript SDK, Web, migrations, and `crates/diff-engine` remain unchanged.

**最小受影响边界与双语文档：** 实现范围仅限私有 `component_state_at_commit` storage value/port、其 memory/PostgreSQL adapter 与聚焦 test。以中英双语更新本计划、`ARCHITECTURE.md`、`docs/storage/persistence-foundation.md`、`docs/roadmap/active-long-term-goal.md` 与完成证据。`server/api`、OpenAPI、TypeScript SDK、Web、migration 和 `crates/diff-engine` 保持不变。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** First observe focused storage tests fail because no Context inventory port exists. Then prove root-to-target collection, deterministic ordering, updates without current-projection leakage, targets before creation, and fail-closed malformed add/update/removal, missing revisions, unknown scope, merge, cycle, and cross-Context ancestry in memory/PostgreSQL parity. Run the registered isolated disposable PostgreSQL case after a loopback schema reset, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, shell guards, and public-surface/`GraphDiff` boundary searches.

**下一增量前的新鲜验证：** 先观察聚焦 storage test 因没有 Context inventory port 而失败。随后证明 root-to-target collection、确定性排序、不会从当前 projection 泄漏更新、creation 前 target，以及内存/PostgreSQL 一致的 fail-closed：损坏 add/update/removal、缺失 revision、unknown scope、merge、cycle 与跨 Context ancestry。运行已登记的 isolated disposable PostgreSQL case（loopback schema reset 后）、`cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web`、shell guard 与 public-surface/`GraphDiff` boundary search。

## Tasks / 任务

### Task 1: Define a Context inventory port / 定义 Context 清单端口

- [x] Add a private descriptor-only Context component-state snapshot value and read port.
- [x] 增加私有、仅含 descriptor 的 Context component-state snapshot value 与 read port。
- [x] Write a failing root-to-target inventory test before implementation.
- [x] 在实现前编写失败的 root-to-target inventory test。

### Task 2: Fold validated durable history / 折叠已验证的持久化历史

- [x] Refactor the pure reducer so one validated root-to-target traversal can produce all component states deterministically.
- [x] 重构纯 reducer，使一次已验证的 root-to-target traversal 可确定性地产生全部 component state。
- [x] Implement memory and PostgreSQL adapters with no current-projection fallback.
- [x] 实现 memory 与 PostgreSQL adapter，不允许 current-projection fallback。
- [x] Reject unsupported removal and malformed state transitions without partial results.
- [x] 拒绝不受支持的 removal 与损坏 state transition，不返回部分结果。

### Task 3: Prove parity and preserve boundaries / 证明一致性并保持边界

- [x] Extend one existing reset-per-test PostgreSQL replay case when it directly covers the inventory contract.
- [x] 仅在既有 reset-per-test PostgreSQL replay case 直接覆盖 inventory contract 时扩展它。
- [x] Document the private Context inventory boundary in English and Chinese.
- [x] 以中英双语记录私有 Context inventory 边界。
- [x] Run fresh scoped verification and record the next dependency-ready increment.
- [x] 运行新鲜的范围验证并记录下一项依赖就绪增量。

## Fresh Local Evidence / 新鲜本地证据

Observed on 2026-07-15: focused pure, memory, and PostgreSQL tests first failed because the Context inventory port did not exist. They then passed with deterministic root-to-target replay and pre-update inventory assertions. `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, and the disposable-storage shell guard passed. A fresh local PostgreSQL 16 container completed all 25 reset-per-test cases, including the expanded component replay case. Boundary searches found no inventory REST/OpenAPI/SDK/Web entrypoint and no additional graph-diff calculator. This is local non-production evidence only; it does not claim remote CI, operator approval, public promotion, release, or production deployment.

2026-07-15 已观察到：聚焦的纯 reducer、memory 与 PostgreSQL test 最初都因 Context inventory port 不存在而失败。它们随后在确定性的 root-to-target replay 与更新前 inventory assertion 下通过。`cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web` 与 disposable-storage shell guard 均已通过。一个新的本地 PostgreSQL 16 container 完成了全部 25 个逐例 reset case，其中包括扩展后的 component replay case。边界搜索未发现 inventory REST/OpenAPI/SDK/Web entrypoint，也未发现额外 graph-diff calculator。这仅是本地非生产证据；不声明 remote CI、operator approval、public promotion、release 或生产部署已通过。
