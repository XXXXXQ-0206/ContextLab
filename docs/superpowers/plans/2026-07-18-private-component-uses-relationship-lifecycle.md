# Private Typed Component Uses Relationship Lifecycle Implementation Plan / 私有类型化组件 Uses 关系生命周期实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` task-by-task. Preserve the guarded local lifecycle and default-deny Web boundary.

**Goal / 目标：** Let an authorized local developer add or remove one directed `Uses` relationship between two active components at a materialized Context branch head, as a replayable commit and server-built graph snapshot.

**Architecture / 架构：** Add typed relationship changes to `contextlab-versioning`; the lifecycle service reconstructs the current graph, validates both component endpoints in the same Context, and derives the successor graph itself. The existing guarded writer atomically persists the commit, snapshot, head, and branch-scoped receipt. Existing private local transport may carry only the tagged component identifiers; the Web surface remains hidden and the BFF rejects writes unless `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE=true`.

**Tech Stack / 技术栈：** Rust 2024, `contextlab-versioning`, `contextlab-graph`, `contextlab-storage`, Axum, `@contextlab/local-sdk`, Next.js, TypeScript, and `@contextlab/ui`.

## Documentation Closure / 文档收束

The current implementation is limited to the private typed `Uses` relationship lifecycle. The protected local command accepts only `add_uses_relationship` or `remove_uses_relationship` with two distinct component identifiers and a non-null exact materialized head. The server owns endpoint reconstruction and validation: both endpoints must still be active components in the same Context and must agree with their materialized graph facts. It then changes exactly one directed `Uses` edge, rejects duplicate addition or missing removal, and preserves every unrelated graph fact. Component removal continues to remove all incident edges, while relationship replay leaves component descriptor and body state unchanged.

当前实现严格限定为私有类型化 `Uses` relationship lifecycle。protected local command 只接受带两个不同 component identifier 和非空精确 materialized head 的 `add_uses_relationship` 或 `remove_uses_relationship`。endpoint 重建与校验归服务端所有：两个 endpoint 必须仍是同一 Context 中的 active component，并且必须与已 materialize 的 graph fact 一致。随后服务端只修改一条有向 `Uses` edge，拒绝重复新增或移除不存在的关系，并保留所有无关 graph fact。component removal 仍会删除全部 incident edge，而 relationship replay 不会改变 component descriptor 或正文状态。

The relationship-only commit deliberately carries no component mutation attachment. It reuses the existing guarded writer for exact-head compare-and-swap, branch-scoped idempotency replay, and atomic commit/snapshot/head/receipt persistence. The server-owned `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` gate keeps the local Web editor and mutation BFF default-deny. Public REST/OpenAPI/public SDK remain unchanged, and `GraphDiff::between` remains the sole graph-diff calculator. Docker-backed PostgreSQL runtime and authenticated browser mutation runtime are unobserved for this increment. The fresh verification receipt below records only observed local evidence; selecting the following increment remains governed separately by the Necessity Record rule.

仅包含 relationship 的 commit 按设计不携带 component mutation attachment。它复用既有 guarded writer 完成 exact-head compare-and-swap、branch-scoped idempotency replay，以及 commit/snapshot/head/receipt 的原子持久化。由服务端拥有的 `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` gate 使 local Web editor 与 mutation BFF 保持默认拒绝。public REST/OpenAPI/public SDK 保持不变，`GraphDiff::between` 仍是唯一 graph-diff calculator。本增量的 Docker-backed PostgreSQL runtime 与 authenticated browser mutation runtime 均未观测。下方的新鲜验证记录只陈述已观察到的本地证据；后续增量的选择仍必须单独遵守 Necessity Record 规则。

---

## Necessity Record / 必要性记录

**Completion criteria / 服务条件：** Advances Criterion 1 (Context-first coverage), Criterion 2 (replayable version history), Criterion 4 (explicit Context Graph editing and review), and Criterion 5 (shared design-system composition).

本增量推进条件 1（Context-first 覆盖）、条件 2（可回放版本历史）、条件 4（显式 Context Graph 编辑与审阅）和条件 5（共享 design-system 组合）。

**Gap and priority / 缺口与优先级：** The current lifecycle creates structural Context-to-component edges but cannot record an explicit component-to-component relationship. `GraphEdgeKind::Uses`, endpoint validation, initialized roots, materialized snapshots, guarded CAS/idempotency, and the default-deny private workflow already exist. This is the smallest dependency-ready graph-editing gap before arbitrary graph payloads, branch/merge policy, semantic diff, or benchmark UI expansion.

当前 lifecycle 只能创建结构性的 Context-to-component edge，不能记录显式 component-to-component relationship。`GraphEdgeKind::Uses`、端点校验、初始化 root、materialized snapshot、guarded CAS/idempotency 与 default-deny 私有工作流均已存在。因此，在任意 graph payload、branch/merge policy、semantic diff 或 benchmark UI 扩张前，这是依赖已满足的最小 graph-editing 缺口。

**Explicit non-goals / 明确非目标：** No arbitrary nodes or edges, cross-Context endpoints, `Contains`/`Owns` mutation, new edge kinds, component body/descriptor edits, Context record creation, branch/merge/rollback, public REST/OpenAPI/public SDK/public Web writes, schema migration, Docker/browser/remote/release claims, or another graph-diff calculator.

不包含任意 node 或 edge、跨 Context endpoint、`Contains`/`Owns` mutation、新 edge kind、component body/descriptor 编辑、Context record creation、branch/merge/rollback、public REST/OpenAPI/public SDK/public Web 写入、schema migration、Docker/browser/remote/release 声明或额外 graph-diff calculator。

**Minimal boundary and documentation / 最小边界与文档：** Modify `crates/versioning/src/change.rs`, `crates/storage/src/context_lifecycle.rs`, guarded-write validation only if the typed change needs it, existing private request/SDK/presenter/editor unions, focused tests, and bilingual architecture/storage/API/roadmap records. PostgreSQL parity is compiled/ignored while Docker stays disabled.

**Fresh verification before another increment / 下一增量前的新鲜验证：** Observe red/green versioning, lifecycle-memory, compiled ignored PostgreSQL, protected API, local SDK, BFF/presenter/editor tests; run `cargo fmt --all -- --check`, scoped Clippy with baseline gaps recorded, `cargo test --workspace --quiet`, `pnpm --filter @contextlab/ts-sdk test`, and `pnpm check:web`.

## File Map / 文件映射

- Modify: `crates/versioning/src/change.rs` and domain tests for `AddedUsesRelationship` / `RemovedUsesRelationship`.
- Modify: `crates/storage/src/context_lifecycle.rs`, `crates/storage/src/guarded_commit_write.rs` only if typed-change validation requires it, plus memory/PostgreSQL-focused tests.
- Modify: `server/api/src/routes.rs`, `server/api/src/lib.rs`, `packages/local-sdk/src/types.ts`, and tests for strict private tags.
- Modify: existing lifecycle presenter/editor/BFF tests only; retain the default-deny Web gate.
- Modify: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/api/local-context-lifecycle.md`, `docs/roadmap/active-long-term-goal.md`, and `docs/roadmap/completion-criteria.md`.

## Task 1: Define Replayable Relationship Changes / 定义可回放关系 Change

- [ ] **Step 1: Write failing versioning tests.** Require add/remove `Uses` changes to carry two distinct component identifiers, reject self-relationships, and serialize without content hashes or descriptors.
- [ ] **Step 2: Run red tests.** Run `cargo test -p contextlab-versioning uses_relationship` and observe missing typed changes.
- [x] **Step 3: Implement the minimal typed domain change.** Add explicit add/remove `Uses` variants and accessors; do not embed graph payloads.
- [x] **Step 4: Run green tests.** Fresh `cargo test -p contextlab-versioning --quiet` observed `23 passed`.

## Task 2: Derive and Persist Successor Graphs / 派生并持久化后继图谱

- [ ] **Step 1: Write failing lifecycle tests.** From a materialized Context state with two active components, require add/remove to create one exact normal-parent commit, change only one `Uses` edge, replay idempotently, reject missing, cross-Context, duplicate-add, and missing-remove endpoints.
- [ ] **Step 2: Run red tests.** Run `cargo test -p contextlab-storage uses_relationship`.
- [x] **Step 3: Implement server-built graph composition.** Add lifecycle operations that validate active component state and modify only the typed `Uses` edge before delegating to the existing guarded writer; add no attachment or migration.
- [x] **Step 4: Run green tests.** Fresh memory storage tests observed `166 passed, 35 ignored`; the named PostgreSQL parity test compiled with `--no-run` and remains unobserved at runtime.

## Task 3: Extend Only the Private Workflow / 仅扩展私有工作流

- [ ] **Step 1: Write failing private contract tests.** Require strict add/remove tags with non-null exact heads and two component IDs; reject client graph payloads, null heads, extra fields, and disabled BFF mutation.
- [ ] **Step 2: Run red tests.** Run focused API, local SDK, and Web tests.
- [x] **Step 3: Implement tagged transport and shared-primitive presentation.** Extend only existing protected local parser, local SDK union, presenter/editor, and enabled BFF forwarding; preserve default denial, RBAC, audit, rate limit, no cookies, and private/no-store.
- [x] **Step 4: Run green tests.** Fresh protected API tests observed `10 passed`; local SDK observed `25 passed`; Web observed `59 passed` and a production build.

## Task 4: Document and Verify / 文档与验证

- [x] **Step 1: Update bilingual architecture and boundaries.** Document server-owned endpoint validation, replay semantics, `Uses`-only scope, default-deny Web behavior, and `GraphDiff::between` preservation.
- [x] **Step 2: Run full local verification.** `cargo fmt --all -- --check` and `cargo test --workspace --quiet` passed (including API `141 passed`, storage `166 passed, 35 ignored`, and versioning `23 passed`); public SDK observed `14 passed`, local SDK `25 passed`, and `pnpm check:web` observed Web `59 passed` plus a production build. The scoped Clippy command stops only at the pre-existing Rust 1.85 MSRV lint in `crates/auth/src/authorization.rs:320`; Docker-backed PostgreSQL runtime and authenticated browser mutation remain unobserved.
- [ ] **Step 3: Keep the long-term goal active.** Select the next dependency-ready core increment only after fresh evidence.

## Verification Recovery Note / 验证恢复说明

The typed domain and storage implementation already existed when this execution environment resumed. The original red-state receipts for Tasks 1–3 were not observed in this session and are deliberately left unchecked rather than reconstructed by deleting working production code. The fresh regression, API, SDK, Web, workspace, formatting, and compiled-PostgreSQL evidence above establishes the current behavior without manufacturing a historical TDD receipt.

本执行环境恢复时，类型化 domain 与 storage 实现已经存在。Tasks 1–3 的原始红灯回执未在本会话中观察到，因此刻意保持未勾选；不会为了补造历史 TDD 回执而删除正常工作的生产代码。上方的新鲜 regression、API、SDK、Web、workspace、格式化以及已编译 PostgreSQL 证据证明当前行为，但不伪造历史红灯记录。
