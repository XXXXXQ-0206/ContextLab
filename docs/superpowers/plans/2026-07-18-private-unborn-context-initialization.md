# Private Unborn-Branch Context Initialization Implementation Plan / 私有未出生分支 Context 初始化实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` task-by-task. Preserve the existing guarded local lifecycle boundary and checkbox evidence tracking.

**Goal / 目标：** Let a local authorized developer initialize one already persisted Context on an unborn branch through the existing private lifecycle workflow, producing one replayable root commit and one root graph snapshot before component editing begins.

**Architecture / 架构：** Add an `initialize` operation to the reusable lifecycle application service. It is the only operation allowed to use `ExpectedBranchHead::Unborn`; it derives a one-node Context graph and one `CreatedContext` change from the persisted Context record, then delegates atomically to the sole guarded writer. The existing non-public local route, local SDK, same-origin BFF, and design-system editor transport and present the tagged intent without adding a public mutation contract.

**Tech Stack / 技术栈：** Rust 2024, `contextlab-versioning`, `contextlab-storage`, Axum, `@contextlab/local-sdk`, Next.js, TypeScript, and `@contextlab/ui`.

---

## Necessity Record / 必要性记录

**Completion criteria / 服务条件：** This directly advances Criterion 1 (Context-first component coverage), Criterion 2 (replayable version history and branches), Criterion 4 (a commit graph records the same Context state), and Criterion 5 (the local editor remains a design-system composition).

本增量直接推进条件 1（以 Context 为核心的 component 覆盖）、条件 2（可回放的版本历史与分支）、条件 4（commit graph 记录同一 Context state）与条件 5（local editor 保持为 design-system composition）。

**Gap and priority / 缺口与优先级：** The current local lifecycle requires a materialized head and cannot create a Context root on an unborn branch. Existing `ExpectedBranchHead::Unborn`, guarded commit/snapshot persistence, root `CreatedContext` changes, branch-scoped idempotency, local transport, and design-system editor dependencies already exist. Initialization is therefore the smallest missing precondition for a usable Context lifecycle and is more direct than arbitrary graph editing, merge policy, semantic diff, or new benchmark presentation.

当前 local lifecycle 要求 materialized head，无法在未出生分支上创建 Context root。既有的 `ExpectedBranchHead::Unborn`、guarded commit/snapshot persistence、root `CreatedContext` change、branch-scoped idempotency、local transport 与 design-system editor 依赖均已存在。因此，初始化是可用 Context lifecycle 缺失的最小前置，且比任意 graph editing、merge policy、semantic diff 或新的 benchmark presentation 更直接。

**Explicit non-goals / 明确非目标：** No new Context record creation, component creation in the root commit, body revision, public REST/OpenAPI/public SDK mutation, public Web mutation control, branch creation UI, merge/rollback semantics, arbitrary graph editing, operator transport, provider call, Docker/PostgreSQL runtime claim, authenticated browser-to-BFF-to-protected-Axum mutation-smoke claim, release/production claim, or additional graph-diff calculator.

不包含新的 Context record creation、root commit 中的 component creation、body revision、public REST/OpenAPI/public SDK mutation、public Web mutation control、branch creation UI、merge/rollback 语义、任意 graph editing、operator transport、provider call、Docker/PostgreSQL runtime 声明、已认证 browser-to-BFF-to-protected-Axum mutation smoke 声明、release/production 声明或额外 graph-diff calculator。

**Minimal boundary and bilingual documentation / 最小边界与双语文档：** Modify the lifecycle command/service and its existing storage tests; extend only the existing local request parser/handler/client/presenter/editor to tag `initialize`; document the local-only root semantics in architecture, storage, API, roadmap, and completion criteria. Keep public routes, checked-in OpenAPI, `@contextlab/ts-sdk`, and `GraphDiff::between` unchanged.

只修改 lifecycle command/service 及其既有 storage test；只扩展既有 local request parser/handler/client/presenter/editor 以标记 `initialize`；在 architecture、storage、API、roadmap 与 completion criteria 中记录仅本地的 root 语义。public route、已检入 OpenAPI、`@contextlab/ts-sdk` 与 `GraphDiff::between` 保持不变。

**Fresh verification before another increment / 下一增量前的新鲜验证：** Observe red then green domain/service, storage memory, compiled ignored PostgreSQL, protected API, local SDK, BFF/presenter/editor tests; then run `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, scoped Clippy with any pre-existing baseline gaps explicitly recorded, and `pnpm check:web`. Docker-disabled PostgreSQL runtime and authenticated browser runtime stay unobserved.

下一增量前必须观察到 domain/service、storage memory、已编译 ignored PostgreSQL、protected API、local SDK、BFF/presenter/editor 测试的红绿过程；随后运行 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`、范围化 Clippy（明确记录任何既有 baseline 缺口）与 `pnpm check:web`。Docker 关闭下的 PostgreSQL runtime 与 authenticated browser runtime 保持未观测。

## File Map / 文件映射

- Modify: `crates/storage/src/context_lifecycle.rs`, `crates/storage/src/lib.rs`, and focused lifecycle tests.
- Modify: `server/api/src/routes.rs`, `server/api/src/lib.rs`, and focused protected-route tests.
- Modify: `packages/local-sdk/src/types.ts`, `packages/local-sdk/src/client.test.ts`.
- Modify: `apps/web/src/app/context-lifecycle-presenter.ts`, `apps/web/src/app/context-lifecycle-editor.tsx`, focused Web tests, and only existing local BFF/data modules if nullable-head transport needs type alignment.
- Modify: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/api/local-context-lifecycle.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and this plan.

## Task 1: Define and Test the Unborn Root Transition / 定义并测试未出生根 Transition

- [x] **Step 1: Write failing lifecycle service tests.** Add tests in `crates/storage/src/context_lifecycle.rs` that call a new initialize command for an existing Context and assert exactly one parentless `CreatedContext` change, exactly one `context:{context_id}` node with the persisted Context name, no component attachment, a created branch head, and an idempotent same-branch replay. Add rejection tests for initialization with an existing head and non-initialize operations with an unborn head.

- [x] **Step 2: Run the focused red tests.**

Run: `cargo test -p contextlab-storage unborn_context_initialization`

Expected: FAIL because `ContextLifecycleOperation::Initialize` and an unborn lifecycle command do not exist.

- [x] **Step 3: Implement the minimal reusable command composition.** Change `ContextLifecycleCommand` to retain `ExpectedBranchHead`; keep existing create/update/descriptor/remove constructors requiring `CommitId`, and add one initialize constructor with `ExpectedBranchHead::Unborn`. Make `ContextLifecycleService::execute` branch before parent-state loading: initialize reads only the persisted Context record, builds the one-node root graph and `CreatedContext` change, creates no attachment, and forwards `ExpectedBranchHead::Unborn`. All other operations retain their existing exact-head behavior.

- [x] **Step 4: Run focused green tests.**

Run: `cargo test -p contextlab-storage unborn_context_initialization`

Expected: PASS with root graph, branch-head, replay, and rejection coverage.

## Task 2: Prove Guarded Persistence Parity / 证明 Guarded 持久化一致性

- [x] **Step 1: Write failing memory and PostgreSQL-compiled tests.** Prove initialization atomically persists commit, snapshot, unborn-branch head, and branch-scoped receipt without component rows or revisions; prove same digest/key replays after a later head advance and a different branch can independently initialize. Add a PostgreSQL counterpart marked ignored with the existing disposable-database annotation.

- [x] **Step 2: Run the focused red tests.**

Run: `cargo test -p contextlab-storage unborn_context_initialization`

Expected: FAIL because the writer does not yet receive a lifecycle-built unborn root command.

- [x] **Step 3: Implement only required adapter integration.** Reuse `GuardedContextCommitWriter` without a new writer or migration. Ensure the service passes no component mutation attachment and that tests verify the existing writer's parentless branch path; do not change GraphDiff or public persistence contracts.

- [x] **Step 4: Run focused green tests.**

Run: `cargo test -p contextlab-storage unborn_context_initialization`

Expected: PASS; PostgreSQL test compiles and remains ignored while Docker is disabled.

## Task 3: Extend the Existing Private Local Contract / 扩展既有私有本地契约

- [x] **Step 1: Write failing API and local SDK tests.** Require a strict `initialize` operation with `expected_head_commit_id: null`; reject a null head for create/update/descriptor/remove and reject any additional initialize fields. Assert protected authentication, RBAC, audit, rate-limit, idempotency headers, replay status, and absence from the public router/OpenAPI/public SDK.

- [x] **Step 2: Run red tests.**

Run: `cargo test -p contextlab-api local_context_lifecycle`

Run: `pnpm --filter @contextlab/local-sdk test`

Expected: FAIL because current request types require a non-null head and do not recognize `initialize`.

- [x] **Step 3: Implement strict tagged decoding and transport.** Make only the private local write request accept an optional head; map null exclusively to initialize and preserve exact non-null head parsing for every other tag. Extend the non-public local SDK discriminated union and parser; preserve caller-supplied Bearer/idempotency headers, no cookies, no token persistence, and private/no-store responses.

- [x] **Step 4: Run green tests.**

Run: `cargo test -p contextlab-api local_context_lifecycle`

Run: `pnpm --filter @contextlab/local-sdk lint`

Run: `pnpm --filter @contextlab/local-sdk test`

Expected: PASS while the public SDK/OpenAPI mutation boundary remains unchanged.

## Task 4: Complete the Local Editor Workflow / 完成本地编辑器工作流

- [x] **Step 1: Write failing presenter, BFF, and editor tests.** Require an Initialize Context / 初始化 Context choice to build only the null-head tagged command, allow it without a selected commit, retain selected-head requirements for the other modes, and render bilingual validation/error states using existing shared primitives.

- [x] **Step 2: Run red tests.**

Run: `pnpm --filter @contextlab/web test`

Expected: FAIL because the presenter and submit gating require a selected non-null head for every operation.

- [x] **Step 3: Implement the smallest presentation change.** Add the initialize option to the existing operation union and presenter; make submit eligibility depend on a selected commit only outside initialize mode. Reuse the same BFF, data client, refresh behavior, `Input`, `Textarea`, `Select`, and `Button` primitives; do not add page-local lifecycle logic or a new page.

- [x] **Step 4: Run green tests.**

Run: `pnpm --filter @contextlab/web lint`

Run: `pnpm --filter @contextlab/web test`

Expected: PASS with the same-origin private transport and bilingual editor behavior.

## Task 5: Document and Verify / 文档与验证

- [x] **Step 1: Update bilingual architecture and operational boundaries.** Record server-owned root graph construction, unborn-only initialization, no component/body attachment, branch-scoped idempotency, existing guarded writer reuse, public-contract exclusion, and `GraphDiff` single-calculator preservation.

- [x] **Step 2: Run complete local verification.**

Run: `cargo fmt --all -- --check`

Run: `cargo clippy -p contextlab-versioning -p contextlab-storage -p contextlab-api --all-targets -- -D warnings`

Run: `cargo test --workspace --quiet`

Run: `pnpm check:web`

Expected: record exact command output. Do not claim clean Clippy unless the command completes; label Docker-disabled PostgreSQL runtime and authenticated browser runtime unobserved.

- [x] **Step 3: Keep the long-term goal active.** Update active-goal and completion-criteria evidence, then select the next dependency-ready local core increment only after fresh results.

## Completion Evidence / 完成证据

**Historical red phases / 历史红阶段：** The original operation, private transport, and editor red phases occurred before the implementation. During review hardening, the API strict-decoding red phase failed `3 passed, 1 failed` before the empty `initialize` variant was added; the bilingual disabled-gate red phase failed `11 passed, 2 failed` before its presentation mapping was added; and the direct-memory receipt red phase failed with missing test-only readers before they were implemented. These are development records, not substitutes for the fresh green receipt below.

原始 operation、私有 transport 与 editor 的红阶段发生在实现之前。审查加固期间，API strict-decoding 红阶段在加入空的 `initialize` variant 前为 `3 passed, 1 failed`；双语 disabled-gate 红阶段在加入其 presentation mapping 前为 `11 passed, 2 failed`；direct-memory receipt 红阶段在实现 test-only reader 前因缺少 reader 而失败。这些是开发记录，不替代下方的新鲜绿灯回执。

**Fresh local receipt, 2026-07-18 / 2026-07-18 新鲜本地回执：** `cargo fmt --all -- --check` exited `0`; `cargo test --workspace --quiet` exited `0` with API `140 passed`, auth `43 passed`, storage `165 passed, 34 ignored`, and zero failures in every remaining group; `pnpm check:web` exited `0` with public SDK `14`, local SDK `24`, Web `57`, TypeScript checks, and a production build. The named PostgreSQL initialization test compiles and is `1 ignored` because Docker is disabled. The scoped Clippy command exits `1` only at the existing Rust 1.85 MSRV lint in `crates/auth/src/authorization.rs:320`; this is not a clean Clippy receipt. The lifecycle editor and mutation BFF are default-deny and require the exact server-owned `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE=true`; an authenticated browser-to-BFF-to-protected-Axum mutation smoke remains unobserved.

`cargo fmt --all -- --check` 返回 `0`；`cargo test --workspace --quiet` 返回 `0`，API `140 passed`、auth `43 passed`、storage `165 passed, 34 ignored`，其余所有 test group 均为零失败；`pnpm check:web` 返回 `0`，public SDK `14`、local SDK `24`、Web `57`、TypeScript check 与 production build 均通过。指定的 PostgreSQL initialization test 已编译但因 Docker 关闭保持 `1 ignored`。范围化 Clippy command 只在 `crates/auth/src/authorization.rs:320` 的已有 Rust 1.85 MSRV lint 处返回 `1`，不将其表述为 clean Clippy receipt。lifecycle editor 与 mutation BFF 默认拒绝，只有精确的服务端 `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE=true` 才可启用；authenticated browser-to-BFF-to-protected-Axum mutation smoke 仍未观测。
