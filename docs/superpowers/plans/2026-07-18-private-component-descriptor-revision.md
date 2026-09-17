# Private Component Descriptor Revision Implementation Plan / 私有组件描述符修订实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` task-by-task. Preserve the guarded local lifecycle boundary and checkbox evidence tracking.

**Goal / 目标：** Let a local editor rename one existing Context component and replace its JSON metadata through one guarded, replayable commit while preserving the immutable body and updating the matching commit graph snapshot label.

**Architecture / 架构：** Add a distinct replayable descriptor transition rather than overloading the body-hash `UpdatedComponent` change. The existing lifecycle service will validate the active descriptor at the expected branch head, create one guarded commit with no body-revision attachment, and advance its graph snapshot, branch head, and idempotency receipt atomically. The existing private protected route, local SDK, same-origin BFF, and design-system editor will expose that one operation only; public REST/OpenAPI/public SDK remain unchanged.

**Tech Stack / 技术栈：** Rust 2024, `contextlab-versioning`, `contextlab-storage`, Axum, `@contextlab/local-sdk`, Next.js, TypeScript, and `@contextlab/ui`.

---

## Necessity Record / 必要性记录

**Completion criteria / 服务条件：** This directly advances Criterion 1 (Context components have domain/persistence/API/SDK/UI coverage), Criterion 2 (descriptor changes replay through commits), Criterion 4 (the committed graph reflects the same Context state), and Criterion 5 (the editor remains a shared-primitive composition).

本增量直接推进条件 1（Context component 具备 domain/persistence/API/SDK/UI 覆盖）、条件 2（描述符变更可通过 commit 回放）、条件 4（已提交图谱反映同一 Context 状态）与条件 5（editor 保持为共享 primitive 的组合）。

**Gap and priority / 缺口与优先级：** The completed lifecycle covers create, immutable body update, and removal, but neither a display-name correction nor metadata revision is a replayable transition. This is the smallest unresolved Context editing workflow after lifecycle and benchmark slices: it reuses the existing guarded writer, normal-first-parent replay, graph snapshot, RBAC, idempotency, local SDK/BFF, and Web editor. It is lower risk and more directly dependency-ready than branch creation, merge/rollback policy, arbitrary graph editing, or a dashboard.

已完成的 lifecycle 覆盖 create、不可变正文 update 与 removal，但 display-name 更正和 metadata revision 尚不是可回放 transition。这是 lifecycle 与 benchmark 切片之后最小的未解决 Context editing workflow：它复用既有 guarded writer、normal-first-parent replay、graph snapshot、RBAC、idempotency、local SDK/BFF 与 Web editor。它比 branch creation、merge/rollback policy、任意 graph editing 或 dashboard 风险更低且依赖更完备。

**Explicit non-goals / 明确非目标：** No body rewrite, component-kind change, branch creation, merge/rollback semantics, arbitrary graph edge editing, public route/OpenAPI/public SDK change, Web public mutation control, schema migration, benchmark work, provider call, GraphDiff calculator change, Docker/PostgreSQL runtime claim, secret access, or release/production claim.

不包含 body rewrite、component-kind change、branch creation、merge/rollback 语义、任意 graph edge editing、public route/OpenAPI/public SDK 变更、Web public mutation control、schema migration、benchmark 工作、provider call、GraphDiff calculator 变更、Docker/PostgreSQL runtime 声明、密钥访问或 release/production 声明。

**Minimal boundary / 最小边界：** Introduce `UpdatedComponentDescriptor` in the reusable versioning domain; add `ContextLifecycleOperation::UpdateDescriptor { component_id, name, metadata }`; extend only the existing guarded storage transaction and same private local route/SDK/BFF/editor. Update the component state reducer and successor graph snapshot from the same transition. Add bilingual architecture/storage/roadmap evidence.

在可复用 versioning domain 中引入 `UpdatedComponentDescriptor`；增加 `ContextLifecycleOperation::UpdateDescriptor { component_id, name, metadata }`；只扩展既有 guarded storage transaction 与同一 private local route/SDK/BFF/editor。由同一 transition 更新 component state reducer 与 successor graph snapshot。增加中英双语 architecture/storage/roadmap 证据。

**Fresh verification before another increment / 下一增量前的新鲜验证：** Observe red then green unit/integration tests for descriptor validation, immutable historical replay, body-hash preservation, graph-label projection, atomic/idempotent guarded writes, stale/removed/kind-mismatch rejection, protected-route authentication/RBAC/rate-limit behavior, strict local SDK/BFF transport, bilingual editor rendering, `cargo fmt --all -- --check`, scoped Clippy, `cargo test --workspace --quiet`, and `pnpm check:web`. PostgreSQL runtime remains explicitly unobserved while Docker is disabled.

下一增量前必须观察到 descriptor validation、不可变历史 replay、body-hash 保持、graph-label projection、原子/idempotent guarded write、stale/removed/kind-mismatch 拒绝、protected-route authentication/RBAC/rate-limit 行为、严格 local SDK/BFF transport、双语 editor 呈现的红绿测试，以及 `cargo fmt --all -- --check`、范围化 Clippy、`cargo test --workspace --quiet` 与 `pnpm check:web`。Docker 关闭时 PostgreSQL runtime 仍明确为未观测。

### Verification Root-Cause Addendum / 验证根因补充

**Blocked conditions / 被阻塞条件：** Criterion 1 requires current component inspection to consume the Context state written by the lifecycle workflow; Criterion 2 requires version/branch behavior to be replayable and unambiguous; Criterion 4 requires current projections and commit graph snapshots to describe the same Context state. Independent review found that a descriptor-only commit changed the immutable snapshot but had no private mutation attachment to update the current component projection, and that guarded idempotency receipts were scoped by principal and Context but not branch.

条件 1 要求当前 component inspection 消费 lifecycle workflow 写入的 Context state；条件 2 要求 version/branch 行为可回放且无歧义；条件 4 要求当前 projection 与 commit graph snapshot 描述同一 Context state。独立审阅发现，descriptor-only commit 虽会改变不可变 snapshot，却没有用于更新当前 component projection 的私有 mutation attachment；guarded idempotency receipt 也只按 principal 与 Context 作用域划分，没有按 branch 隔离。

**Minimal repair / 最小修复：** Add one private descriptor-revision attachment to the existing guarded writer. It validates the one typed descriptor transition and successor graph node, then updates only the mutable component name, metadata, and timestamp in the same memory overlay or PostgreSQL transaction that persists the commit, snapshot, branch head, and receipt. Scope memory receipts and PostgreSQL lookup/lock/uniqueness to `(identity source, principal, Context, branch, key)` through a forward-only migration that backfills the immutable branch from each referenced commit. Do not change body revisions, historical replay, public REST/OpenAPI/public SDK, Web mutation surface, operator transport, or `GraphDiff`.

在既有 guarded writer 中增加一个私有 descriptor-revision attachment。它会校验唯一的 typed descriptor transition 与 successor graph node，然后只在持久化 commit、snapshot、branch head 与 receipt 的同一 memory overlay 或 PostgreSQL transaction 中更新可变 component 的 name、metadata 与 timestamp。通过仅向前的 migration，从每个被引用 commit 回填不可变 branch，并将 memory receipt 和 PostgreSQL lookup/lock/uniqueness 的作用域调整为 `(identity source, principal, Context, branch, key)`。不改变 body revision、历史 replay、public REST/OpenAPI/public SDK、Web mutation surface、operator transport 或 `GraphDiff`。

**Regression proof / 回归证明：** First observe tests that fail because descriptor commits leave current component list/detail/graph projections stale and because the same key can cross branches. Green tests must prove descriptor writes atomically update current projections, a same-branch retry after a later head advance returns only the original commit, and identical keys on separate branches create/replay only within their own branch. PostgreSQL tests remain compiled/ignored until Docker is available; local evidence must label that boundary explicitly.

先观察会失败的测试：descriptor commit 会使当前 component list/detail/graph projection 陈旧，同一个 key 还能跨 branch 重放。绿测必须证明 descriptor write 原子更新当前 projection；同一 branch 在后续 head 推进后的 retry 只返回原始 commit；相同 key 在不同 branch 只能在各自 branch 内创建或重放。Docker 可用前 PostgreSQL test 仍为 compiled/ignored；本地证据必须明确标注该边界。

## File Map / 文件映射

- Modify: `crates/versioning/src/change.rs`
- Modify: `crates/storage/src/context_lifecycle.rs`, `crates/storage/src/component_state_at_commit.rs`, `crates/storage/src/guarded_commit_write.rs`, `crates/storage/src/memory.rs`, `crates/storage/src/postgres.rs`
- Test: existing focused versioning/storage lifecycle and replay tests
- Modify: `server/api/src/routes.rs`, `server/api/src/lib.rs`
- Modify: `packages/local-sdk/src/types.ts`, `packages/local-sdk/src/client.ts`, `packages/local-sdk/src/client.test.ts`
- Modify: `apps/web/src/app/context-lifecycle-data.ts`, `apps/web/src/app/context-lifecycle-presenter.ts`, `apps/web/src/app/context-lifecycle-editor.tsx`, and focused tests
- Modify: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and this plan

## Task 1: Define the Replayable Descriptor Transition / 定义可回放描述符 Transition

- [x] **Step 1: Write failing versioning and reducer tests.** Require a descriptor change with a nonblank new name and JSON metadata to preserve `component_kind`, `previous_content_hash`, and `resulting_content_hash`; replay the creation commit and successor commit to prove old and new name/metadata remain isolated. Require a missing descriptor payload or a body-hash mutation to fail.

- [x] **Step 2: Run focused red tests.**

Run: `cargo test -p contextlab-versioning updated_component_descriptor`

Run: `cargo test -p contextlab-storage component_descriptor`

Observed: FAIL because the descriptor transition and lifecycle command did not exist; the guarded attachment gate then rejected the initially un-attached descriptor transition.

- [x] **Step 3: Implement the minimal domain and replay state.** Add `ContextChangeKind::UpdatedComponentDescriptor` and a constructor that stores component ID, unchanged kind, validated replacement name, replacement metadata, and no body hash transition. Extend `replay_component_state_transition` to require an existing matching-kind state, retain its content commit/hash, and replace only descriptor fields.

- [x] **Step 4: Run focused green tests.**

Run: `cargo test -p contextlab-versioning updated_component_descriptor`

Run: `cargo test -p contextlab-storage component_descriptor`

Observed: PASS with immutable historical descriptors and unchanged body hashes.

## Task 2: Persist One Atomic Guarded Descriptor Revision / 原子持久化一次 Guarded 描述符修订

- [x] **Step 1: Write failing memory and PostgreSQL-compiled contract tests.** Require a descriptor revision to produce one successor commit, one matching graph snapshot with the new node label, one branch-head advance, and one idempotency replay. Require stale heads, removed components, kind mismatch, and malformed descriptor changes to leave no partial state.

- [x] **Step 2: Run focused red tests.**

Run: `cargo test -p contextlab-storage descriptor_revision`

Observed: FAIL because the guarded writer initially required a body mutation attachment for every component change.

- [x] **Step 3: Implement guarded lifecycle composition.** Add `UpdateDescriptor` to `ContextLifecycleOperation`; resolve the active component state at the expected head; construct the descriptor change; apply it to the successor graph snapshot; and use the existing guarded commit transaction so commit, snapshot, branch head, and idempotency receipt advance together without a component body revision write.

- [x] **Step 4: Run focused green tests.**

Run: `cargo test -p contextlab-storage descriptor_revision`

Observed: PASS for memory. PostgreSQL tests compile and remain ignored unless an explicitly disposable database is available.

## Task 3: Extend Only the Existing Private Local Workflow / 仅扩展既有私有本地工作流

- [x] **Step 1: Write failing API, local SDK, BFF, and Web tests.** Require `update_descriptor` on the existing protected lifecycle route; assert request-scoped Bearer and idempotency transport, no cookies, private/no-store, generic protected errors, and a bilingual editor command that preserves the component body field.

- [x] **Step 2: Run focused red tests.**

Run: `cargo test -p contextlab-api local_context_lifecycle`

Run: `pnpm --filter @contextlab/local-sdk test`

Run: `pnpm --filter @contextlab/web test`

Observed: FAIL before the discriminated operation, strict parser, and editor mode were implemented.

- [x] **Step 3: Implement narrow transport and presentation.** Add only the new local operation variant to the existing route request, local SDK parser/client, BFF payload flow, presenter, and shared-primitive editor controls. Reuse the existing lifecycle refresh and graph review; do not add a route, public contract, or page-local business rule.

- [x] **Step 4: Run focused green tests.**

Run: `cargo test -p contextlab-api local_context_lifecycle`

Run: `pnpm --filter @contextlab/local-sdk lint`

Run: `pnpm --filter @contextlab/local-sdk test`

Run: `pnpm --filter @contextlab/web lint`

Run: `pnpm --filter @contextlab/web test`

Observed: PASS with public route/OpenAPI/public SDK absence retained.

## Task 4: Document and Verify / 文档与验证

- [x] **Step 1: Record bilingual boundaries.** Document descriptor-only revision semantics, historical replay, graph-label consistency, single guarded transaction, unchanged public contracts, unchanged GraphDiff, and deferred PostgreSQL runtime.

- [x] **Step 2: Run scope-matched verification.**

Run: `cargo fmt --all -- --check`

Run: `cargo clippy -p contextlab-versioning -p contextlab-storage -p contextlab-api --all-targets -- -D warnings`

Run: `cargo test --workspace --quiet`

Run: `pnpm check:web`

Observed (fresh 2026-07-18): `cargo fmt --all -- --check` passed. The first full `cargo test --workspace --quiet` exposed one migration-contract regression: `benchmark_migration` incorrectly required immutable migration `0016` to remain the last ledger entry after `0017`. The test was corrected to require inclusion, then `cargo test -p contextlab-storage --test benchmark_migration --quiet` passed `5`; a second full workspace run passed, including API `136 passed` and storage `163 passed, 33 ignored`. `pnpm check:web` passed: public SDK `14 passed`, local SDK `23 passed`, Web `51 passed`, TypeScript checks, and the production Web build. Strict scoped Clippy remains an open repository baseline: its first run stops at the unrelated Rust 1.85 MSRV lint in `contextlab-auth/src/authorization.rs:320`; allowing that lint exposes existing storage/API warnings. It is therefore not claimed as a clean descriptor-slice receipt. PostgreSQL runtime remains compiled/ignored and unobserved while Docker is disabled.

观察结果（2026-07-18 新鲜）：`cargo fmt --all -- --check` 已通过。第一次完整 `cargo test --workspace --quiet` 暴露一项 migration-contract regression：`benchmark_migration` 在 `0017` 已追加后仍错误要求不可变 migration `0016` 必须是 ledger 的最后一项。测试改为要求被包含后，`cargo test -p contextlab-storage --test benchmark_migration --quiet` 通过 `5`；第二次完整 workspace run 通过，其中 API 为 `136 passed`，storage 为 `163 passed, 33 ignored`。`pnpm check:web` 已通过：public SDK `14 passed`、local SDK `23 passed`、Web `51 passed`，以及 TypeScript 检查和 production Web build。范围化 strict Clippy 仍是开放的 repository baseline：首次运行停在 `contextlab-auth/src/authorization.rs:320` 无关的 Rust 1.85 MSRV lint；放行该 lint 后会暴露已有 storage/API warning。因此不将它表述为干净的 descriptor-slice receipt。Docker 关闭时 PostgreSQL runtime 仍为 compiled/ignored 且未观测。

- [x] **Step 3: Record exact evidence and retain the active long-term goal.** The long-term goal remains active; local PostgreSQL runtime, browser E2E, remote CI, and production evidence remain unobserved.

## Kickoff Evidence / 启动证据

The first domain-only red test was observed with `cargo test -p contextlab-versioning updated_component_descriptor`: `ContextChange::updated_component_descriptor` and `UpdatedComponentDescriptor` did not exist. After adding the distinct no-body-hash transition, the same command passed one test. Compiling storage then exposed the required non-exhaustive state-replay branch; `cargo test -p contextlab-storage --lib --quiet` passed `159` tests with `32` explicitly ignored after the descriptor replay branch was added. This is historical Task 1 evidence only; the completed guarded persistence, protected transport, SDK, BFF, Web, and final local verification are recorded above. PostgreSQL runtime remains explicitly unobserved while Docker is disabled.

第一个仅领域红测通过 `cargo test -p contextlab-versioning updated_component_descriptor` 被实际观测：`ContextChange::updated_component_descriptor` 与 `UpdatedComponentDescriptor` 尚不存在。加入独立且无 body-hash 的 transition 后，同一命令通过了 1 个 test。随后编译 storage 暴露了必须补齐的非穷尽 state-replay 分支；加入 descriptor replay 分支后，`cargo test -p contextlab-storage --lib --quiet` 通过 `159` 个 test，并有 `32` 个显式 ignored。这只是历史 Task 1 证据；已完成的 guarded persistence、protected transport、SDK、BFF、Web 与最终本地验证均已在上文记录。Docker 关闭时 PostgreSQL runtime 仍明确为未观测。
