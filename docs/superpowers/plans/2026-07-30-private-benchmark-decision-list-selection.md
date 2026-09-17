# Private Benchmark Decision-List Selection Binding / 私有 Benchmark Decision 列表选择绑定

## Necessity Record / 必要性记录

**Criterion served / 服务条件：** Criterion 3 and the Context-first workflow condition:
the local benchmark workspace must be usable from the server-owned sealed decision list,
with an exact Context commit/decision binding and no manual identifier substitution in the
Web flow.

**Unmet dependency or evidence gap / 未满足依赖或证据缺口：** The protected
decision-keyed workspace route, local SDK, and BFF now exist, but the Web inspector still
accepts a free-form decision ID. The existing decision discovery list already provides
redacted, ordered sealed decisions for the selected commit, yet it is not connected to the
workspace target state. This leaves a user-visible gap between decision discovery and the
workspace projection and permits a listed commit to be paired with an unrelated typed ID.

**Why now / 为什么现在优先：** The exact decision resolver and server-owned decision list
are already verified, so this is the smallest remaining read-only bridge for the benchmark
vertical slice. It directly improves Criterion 3 usability without adding evaluation policy,
transport, mutation, provider, or persistence behavior.

**Non-goals / 明确非目标：** No benchmark authoring/execution, provider call, scorecard or
regression algorithm, cohort derivation, public REST/OpenAPI/public SDK write, operator
transport, migration, Docker/PostgreSQL runtime claim, secret access, release, or production
claim. Existing cohort-keyed route remains compatible and Web remains a presenter of server
owned redacted data.

**Smallest boundary and bilingual documentation / 最小边界与双语文档：** Reuse the existing
local SDK decision-list parser/client and `data -> presenter -> screen` primitives. Add only
the selection state/callback contract needed for the workspace inspector to consume an exact
listed decision at the already selected commit, plus focused SDK/Web regressions. Keep Rust
domain/storage, API route, public OpenAPI/SDK, and evaluation policy untouched. Update this
plan and the bilingual parallel/active/completion records after fresh verification.

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Focused local SDK
decision-list and Web selection/invalidation tests; `pnpm check:web`; then workspace Rust,
`cargo fmt --all -- --check`, strict offline Clippy, locked Rust 1.85 check, and
`GRAPH_DIFF_IMPL_COUNT=1`. Authenticated browser, PostgreSQL/Docker runtime, Git, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`.

## Tasks / 任务

- [x] Define the exact listed-decision selection DTO/state without duplicating parser policy.
- [x] Bind selected decision identity to the current commit and exact workspace loader.
- [x] Remove the free-form decision-ID path from the normal workspace UI while preserving the cohort compatibility path.
- [x] Add selection, commit-change invalidation, empty/loading/error, and scope-drift regressions.
- [x] Record fresh local evidence and update bilingual roadmap/audit documents.

## Current Review Ledger / 当前审计台账

**Status / 状态:** `completed / verified locally` / `completed / 本地已验证`. The shared
workspace contains the local SDK selection resource/loader and Web decision discovery/select
wiring. The normal Web controls are selects populated from the exact commit list, while the
cohort-keyed route remains available for compatibility. The full Web suite and `pnpm check:web`
are green.

本轮仅审阅并修改文档，未修改代码；共享工作区现已包含 local SDK selection resource/loader 与 Web
decision discovery/select wiring。normal Web controls 已由 exact commit list 提供 select，cohort-keyed
route 继续作为兼容路径。完整 Web suite 与 `pnpm check:web` 均通过。

**Completed in the shared increment / 共享增量已完成:** The local SDK now owns the frozen
listed-decision selection resource, exact `(project, Context, commit)` scope check, selected-ID
membership check, and decision-keyed workspace loader. Web loads revised/baseline discovery lists,
clears selections on commit changes, and renders list options. Current evidence includes local SDK
`104 passed`, workspace selection focused `16 passed`, and discovery data/presenter `8 passed`.

**共享增量已完成：** local SDK 现拥有 frozen listed-decision selection resource、精确
`(project, Context, commit)` scope check、selected-ID membership check 与 decision-keyed workspace
loader。Web 会加载 revised/baseline discovery list，在 commit 变化时清空 selection，并渲染 list option。
当前证据包含 local SDK `104 passed`、workspace selection focused `16 passed` 与 discovery data/presenter
`8 passed`，已覆盖新的 selection lifecycle。

**Pending / 未完成:** No local implementation task remains in this plan. PostgreSQL, browser,
visual, remote CI, Git change-set, release, and production evidence were not run or observed in
this review and remain outside this local receipt.

**未完成：** 本计划没有剩余的本地实现任务。本文审计未运行也未观测 PostgreSQL、browser、visual、remote CI、Git
change-set、release 或 production 证据；它们仍在本地回执范围之外。

## Evidence Provenance / 证据 provenance

| Command / 命令 | Result / 结果 | Provenance and boundary / provenance 与边界 |
| --- | --- | --- |
| `pnpm --filter @contextlab/local-sdk test` | `104 passed, 0 failed` | Current local SDK process; includes selection resource, scope, membership, and selected-loader tests. / 当前 local SDK 进程；包含 selection resource、scope、membership 与 selected-loader tests。 |
| `pnpm --filter @contextlab/web exec tsx --test src/app/local-benchmark-workspace-inspector.test.tsx src/app/local-benchmark-workspace-data.test.ts src/app/local-benchmark-workspace-presenter.test.ts src/app/local-benchmark-workspace-screen.test.tsx` | `16 passed, 0 failed` | Workspace selection, commit invalidation, state handling, scope rejection, and screen semantics. / 覆盖 workspace selection、commit invalidation、状态处理、scope rejection 与 screen 语义。 |
| `pnpm --filter @contextlab/web exec tsx --test src/app/context-benchmark-decision-discovery-data.test.ts src/app/context-benchmark-decision-discovery-presenter.test.tsx` | `8 passed, 0 failed` | Current discovery parser/presenter tests; no workspace selection wiring is asserted. / 当前 discovery parser/presenter 测试；未断言 workspace selection wiring。 |
| `pnpm --filter @contextlab/web test` | `208 passed, 0 failed` | Current Web suite; includes exact-list selection, empty/loading/error/unavailable states, scope rejection, and the updated inspector lifecycle. / 当前 Web suite；包含 exact-list selection、empty/loading/error/unavailable state、scope rejection 与更新后的 inspector lifecycle。 |
| `cargo fmt --all -- --check`; `cargo test --workspace --quiet --no-fail-fast`; `cargo clippy --workspace --all-targets --offline -- -D warnings`; `cargo +1.85.0 check --workspace --all-targets --locked --offline` | `passed` | Workspace Rust, format, strict offline Clippy, and locked MSRV check; storage `204 passed, 39 ignored`, API `191 passed`. / workspace Rust、format、strict offline Clippy 与锁定 MSRV check；storage `204 passed, 39 ignored`、API `191 passed`。 |
| `GRAPH_DIFF_IMPL_COUNT=1` static search | `passed` | Exactly one `impl GraphDiff`; no second calculator was introduced. / 仅有一个 `impl GraphDiff`，未引入第二个计算器。 |

`pnpm check:web` passed with public SDK `15`, local SDK `104`, Web `208`, TypeScript/lint, and
production build. Workspace Rust tests, `cargo fmt --all -- --check`, strict offline Clippy, locked
Rust check, and `GRAPH_DIFF_IMPL_COUNT=1` also passed. Authenticated browser, PostgreSQL/Docker runtime,
visual, remote CI, operator rehearsal, Git change-set, release, and production remain `unobserved` or
`deferred`.

`pnpm check:web` 已通过，public SDK `15`、local SDK `104`、Web `208`、TypeScript/lint 与 production build 均完成。
workspace Rust tests、`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust check 与
`GRAPH_DIFF_IMPL_COUNT=1` 也已通过。authenticated browser、PostgreSQL/Docker、visual、remote CI、operator、Git
change-set、release 与 production 继续为 `unobserved` 或 `deferred`。
