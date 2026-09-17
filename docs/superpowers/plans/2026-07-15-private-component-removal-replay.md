# Private Component Removal Replay Plan / 私有 Component Removal Replay 计划

**Goal / 目标：** Add a private, replayable component-removal transition so a Context component-state snapshot can prove absence after a durable removal instead of rejecting otherwise valid lifecycle history.

**Architecture / 架构：** Let `contextlab-versioning` own the typed removal payload and its preconditions. Reuse guarded commit attachments, normal first-parent replay, immutable component revision history, and Context component-state reducers. A removal changes existence only: it must not expose a body or create another graph-diff engine.

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** Completion criteria 1, `Context-first platform coverage`, and 2, `Versioning and diff workflows`. A replayable Context inventory must model both presence and durable absence to represent an actual Context version.

**服务条件：** 服务完成条件 1“以 Context 为核心的平台覆盖”与条件 2“版本与 Diff 工作流”。可回放的 Context inventory 必须同时表示 presence 与持久化 absence，才能代表真实的 Context version。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口：** `RemovedComponent` currently has no typed kind/prior-hash transition contract, and the verified state/inventory reducers correctly fail closed rather than infer deletion from the current projection. Without a durable precondition, an update-after-removal or stale removal cannot be distinguished from corrupted history.

**未满足依赖、风险或证据缺口：** `RemovedComponent` 当前没有 typed kind/prior-hash transition contract；已验证的 state/inventory reducer 正确地 fail closed，而不会从当前 projection 推断删除。没有持久化 precondition 时，无法区分 update-after-removal、stale removal 与损坏历史。

**Why now / 为什么现在优先：** The completed Context inventory gives a direct, evidence-backed consumer of removal semantics. This is more immediate than public editing, benchmarks, workflow, or UI work because those capabilities require correct historical component existence.

**为什么现在优先：** 已完成的 Context inventory 提供了 removal semantics 的直接、证据充分的 consumer。它比 public editing、benchmark、workflow 或 UI 工作更迫切，因为这些能力需要正确的历史 component existence。

**Explicit non-goals / 明确非目标：** No public REST route, OpenAPI operation, SDK method, Web control, operator transport, migration, merge policy, branch UI, body exposure, physical deletion, release work, or additional graph-diff calculator. A first iteration may keep active current-projection deletion private to guarded storage while replay never infers deletion from that projection.

**明确非目标：** 不新增 public REST route、OpenAPI operation、SDK method、Web control、operator transport、migration、merge policy、branch UI、body exposure、物理删除、release 工作或额外 graph-diff calculator。第一轮可以把 active current-projection deletion 保持在 guarded storage 私有边界内，但 replay 绝不从该 projection 推断删除。

**Minimal affected boundary and bilingual docs / 最小受影响边界与双语文档：** Restrict changes to `crates/versioning`, `crates/storage` guarded-write/replay adapters and focused tests, plus this plan, `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, and roadmap evidence. `server/api`, OpenAPI, TypeScript SDK, Web, and `crates/diff-engine` remain unchanged.

**最小受影响边界与双语文档：** 修改范围仅限 `crates/versioning`、`crates/storage` guarded-write/replay adapter 与聚焦 test，并更新本计划、`ARCHITECTURE.md`、`docs/storage/persistence-foundation.md` 和路线图证据。`server/api`、OpenAPI、TypeScript SDK、Web 与 `crates/diff-engine` 保持不变。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** First observe typed-removal and guarded-write tests fail. Then prove removal after creation returns no state in later snapshots, stale/mismatched or duplicate removal fails without partial persistence, update-after-removal fails, and memory/PostgreSQL parity holds under isolated schema resets. Run the full scoped formatting, workspace, Web/SDK, shell-guard, boundary-search, and disposable PostgreSQL verification.

**下一增量前的新鲜验证：** 先观察 typed-removal 与 guarded-write test 失败。随后证明 creation 后 removal 会使较晚 snapshot 不含该 state，stale/mismatched 或 duplicate removal 会在无 partial persistence 的前提下失败，update-after-removal 失败，并在隔离 schema reset 下验证 memory/PostgreSQL parity。运行完整的格式、workspace、Web/SDK、shell-guard、boundary-search 与 disposable PostgreSQL 验证。

## Completion Evidence / 完成证据

- [x] Added a typed `RemovedComponent` with component kind and prior-hash preconditions, plus matching private guarded-write attachment validation.
- [x] 增加带 component kind 与 prior-hash 前置条件的 typed `RemovedComponent`，并实现匹配的私有 guarded-write attachment 校验。
- [x] Proved valid removal omits the component from later state and inventory, while stale, repeated, and post-removal transitions fail closed without partial persistence.
- [x] 证明合法 removal 会使 component 在较晚 state 与 inventory 中缺席，而陈旧、重复与 removal 后的 transition 会在无部分持久化的前提下 fail closed。
- [x] Preserved idempotent replay and Memory/PostgreSQL parity; PostgreSQL soft-deletes only the mutable current projection while immutable revisions remain replay witnesses.
- [x] 保持 idempotent replay 与 Memory/PostgreSQL 一致性；PostgreSQL 仅 soft-delete 可变 current projection，不可变 revision 仍作为 replay witness 保留。

Observed locally on 2026-07-15: focused removal tests, `cargo fmt --all -- --check`, `cargo test --workspace` (storage: `147 passed, 25 ignored`), and `pnpm check:web` passed. A loopback-only disposable PostgreSQL 16 container completed all 25 reset-per-test cases through `scripts/verify-disposable-postgres-storage.sh`; the existing creation lifecycle case exercises the removal path, so no 26th case is claimed. The initial script invocation exposed a missing host `psql` command; the minimal remediation was a temporary Docker-backed local client shim against a synthetic disposable database, removed immediately after the successful rerun. A boundary search found no removal REST/OpenAPI/SDK/Web transport and no second graph-diff calculator; `GraphDiff::between` remains the sole calculator. This is local non-production evidence only. No preserved red-test output is claimed, and no remote CI, operator approval, release, public promotion, or production result is inferred.

2026-07-15 已在本地观察到：聚焦 removal test、`cargo fmt --all -- --check`、`cargo test --workspace`（storage：`147 passed, 25 ignored`）与 `pnpm check:web` 均通过。仅限 loopback 的 disposable PostgreSQL 16 container 通过 `scripts/verify-disposable-postgres-storage.sh` 完成全部 25 个逐例 reset case；既有 creation lifecycle case 已覆盖 removal 路径，因此不声称新增第 26 项。初次脚本调用暴露本机缺少 `psql` 命令；最小修复是针对合成 disposable 数据库使用临时 Docker-backed 本地 client shim，并在成功重跑后立即删除。边界搜索未发现 removal REST/OpenAPI/SDK/Web transport 或第二个 graph-diff calculator；`GraphDiff::between` 仍是唯一计算器。这仅是本地非生产证据；不声称保存了 red-test 输出，也不推断 remote CI、operator 批准、release、public promotion 或 production 结果。
