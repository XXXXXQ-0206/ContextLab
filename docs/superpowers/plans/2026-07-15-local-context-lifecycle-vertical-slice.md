# Local Context Lifecycle Vertical Slice / 本地 Context 生命周期垂直切片实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标：** Deliver a local, authenticated Context component lifecycle workflow that creates, revises, removes, replays, and graph-compares component state through the existing guarded commit boundary without making a public-release claim.

**Architecture / 架构：** The reusable `contextlab-storage` lifecycle application boundary prepares typed component lifecycle commits from an exact materialized head; it owns change construction and graph-snapshot transformations while retaining the existing guarded persistence and replay contracts. Protected API routes authenticate, rate-limit, authorize, audit, and delegate; a non-public local SDK and a design-system-based Web workflow only transport typed commands and render presenter models. `GraphDiff::between` remains the only graph-diff calculation.

**Tech Stack / 技术栈：** Rust 2024, Tokio, Axum, Serde, SQLx/PostgreSQL, Next.js, TypeScript, React, workspace design system.

---

## Necessity Record / 必要性记录

**Completion criteria served / 服务的完成条件：** Criterion 1, Context-first platform coverage; criterion 2, versioning and diff workflows; criterion 4, Context Graph editing and diffing; and criterion 5, design-system-first interfaces. The slice turns verified private component lifecycle storage into a local developer workflow with replay and existing version-backed `GraphDiff` review.

**服务的完成条件：** 本切片服务于完成条件 1“以 Context 为核心的平台覆盖”、条件 2“版本与 Diff 工作流”、条件 4“Context Graph 编辑与比较”以及条件 5“设计系统优先界面”。它将已验证的私有 component 生命周期存储能力变为带回放与既有 version-backed `GraphDiff` 审阅的本地开发工作流。

**Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口：** The guarded writer can carry exactly one typed creation, revision, or removal attachment, but the protected API only constructs an unattached command. Public reads expose only component metadata and hashes; replayed bodies and a commit's complete component/graph state have no aggregate transport. The Web has no protected lifecycle transport or reusable text-entry primitive. Implementing controls without these contracts would duplicate lifecycle semantics in UI code, disclose bodies through public reads, or bypass authorization, audit, rate limiting, idempotency, branch-head CAS, and the atomic writer.

**未满足依赖、风险与证据缺口：** guarded writer 已能承载恰好一个 typed creation、revision 或 removal attachment，但 protected API 当前只构造未附带 attachment 的 command。public read 只暴露 component metadata 与 hash；回放正文和某个 commit 的完整 component/graph state 尚无聚合 transport。Web 没有 protected lifecycle transport 或可复用文本输入 primitive。若在这些 contract 缺失时直接添加控件，就会在 UI 复制生命周期语义、经 public read 泄露正文，或绕过 authorization、audit、rate limit、idempotency、branch-head CAS 与原子 writer。

**Why now / 为什么现在优先：** Creation, content revision, and typed removal now have fresh memory/PostgreSQL evidence and replay semantics, including removal-to-absence. The next named gap is usability of that Context lifecycle, not another field-level private storage increment. This is the smallest end-to-end increment that directly converts the existing Context-first core into a locally demonstrable, replayable workflow.

**为什么现在优先：** creation、content revision 与 typed removal 现已拥有新鲜的 memory/PostgreSQL 证据和 replay semantics，其中包括 removal-to-absence。下一个已命名缺口是 Context 生命周期的可用性，而不是另一个字段级私有 storage 增量。这是把现有 Context-first 核心直接转化为本地可演示、可回放 workflow 的最小端到端增量。

**Explicit non-goals / 明确非目标：** No public write promotion, public OpenAPI operation, mutation on the public `ContextLabClient`, release or production-readiness claim, token issuance, OAuth login, persistent browser credential storage, Context initialization, branch/merge UI, arbitrary graph editing, semantic/behavior/evaluation diff, benchmark execution, operator transport, or second graph-diff implementation. A local lifecycle command requires an existing materialized branch head and does not create an unborn Context root.

**明确非目标：** 不包含 public write promotion、public OpenAPI operation、在 public `ContextLabClient` 上添加 mutation、release 或 production-readiness 声明、token issuance、OAuth 登录、持久化浏览器 credential storage、Context 初始化、branch/merge UI、任意图编辑、semantic/behavior/evaluation diff、benchmark 执行、operator transport 或第二个 graph-diff 实现。本地 lifecycle command 要求已有 materialized branch head，不创建 unborn Context root。

**Minimal affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Extend the existing reusable `crates/storage/src/context_lifecycle.rs` application boundary with a private aggregate state-at-commit contract and memory/PostgreSQL adapters; add protected local API routes and operation-scoped middleware; add a non-public local SDK package and Web proxy/data/presenter/control modules; add `Input` and `Textarea` primitives to `packages/ui`; update architecture, API boundary, lifecycle workflow, and roadmap evidence in English and Chinese. Public route catalogs, checked-in OpenAPI, public SDK exports, and `diff-engine` stay unchanged.

**最小受影响边界与双语文档：** 扩展既有 `crates/storage/src/context_lifecycle.rs` application boundary，通过组合现有 snapshot、inventory replay、nearest-body replay 与 guarded-writer port 提供私有 aggregate state-at-commit 能力；新增 protected local API route 与按 operation 区分的 middleware；新增非公开 local SDK package 和 Web proxy/data/presenter/control module；在 `packages/ui` 中添加 `Input` 与 `Textarea` primitive；用中英双语更新 architecture、API boundary、lifecycle workflow 与 roadmap evidence。public route catalog、已检入 OpenAPI、public SDK export 与 `diff-engine` 均保持不变。

**Verification target and final boundary / 验证目标与最终边界：** The slice must prove aggregate replay, lifecycle command construction, protected authorization/rate-limit behavior, local SDK transport, Web interaction, and preview accessibility without expanding the public contract. Final local receipts cover those implementation boundaries. The dedicated lifecycle PostgreSQL end-to-end case and the authenticated browser-to-BFF-to-protected-Axum smoke remain deferred and unobserved; the dated 25-case PostgreSQL run is only an underlying storage baseline and cannot satisfy either lifecycle gate.

**验证目标与最终边界：** 本切片必须证明 aggregate replay、lifecycle command construction、protected authorization/rate-limit 行为、local SDK transport、Web interaction 与 preview accessibility，同时不得扩张 public contract。最终本地回执已覆盖这些实现边界。专用 lifecycle PostgreSQL 端到端 case 与 authenticated browser-to-BFF-to-protected-Axum smoke 仍为 deferred/unobserved；带日期的 25-case PostgreSQL 运行仅是底层 storage baseline，不能替代任一 lifecycle 门禁。

## File Map / 文件映射

- Modify: `crates/storage/src/context_lifecycle.rs`, `crates/storage/src/lib.rs`, `crates/storage/src/memory.rs`, `crates/storage/src/postgres.rs`, `crates/graph/src/lib.rs`
- Modify: `server/api/src/lib.rs`, `server/api/src/routes.rs`
- Create: `packages/local-sdk/package.json`, `packages/local-sdk/tsconfig.json`, `packages/local-sdk/src/index.ts`, `packages/local-sdk/src/client.ts`, `packages/local-sdk/src/types.ts`, `packages/local-sdk/src/client.test.ts`
- Modify: `apps/web/package.json`, `packages/ui/src/index.ts`, `packages/ui/src/styles.css`
- Create: `packages/ui/src/primitives/input.tsx`, `packages/ui/src/primitives/textarea.tsx`, `apps/web/src/app/context-lifecycle-data.ts`, `apps/web/src/app/context-lifecycle-presenter.ts`, `apps/web/src/app/context-lifecycle-editor.tsx`, `apps/web/src/app/context-lifecycle-proxy.ts`, `apps/web/src/app/api/local/contexts/[contextId]/commits/[commitId]/lifecycle-state/route.ts`, `apps/web/src/app/api/local/contexts/[contextId]/component-lifecycle-commits/route.ts`
- Modify: `apps/web/src/app/context-workspace-screen.tsx`, `apps/web/src/app/context-workspace-data.ts`, and focused Web tests
- Modify: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and a bilingual local lifecycle workflow document

## Task 1: Aggregate Immutable Lifecycle State / 聚合不可变生命周期状态

**Files:** Modify `crates/storage/src/context_lifecycle.rs`, storage exports, memory, PostgreSQL, and focused storage tests.

- [x] Write and observe failing in-memory service tests for a target commit whose aggregate contains its immutable graph snapshot plus deterministic component descriptor, metadata, body, hash, and provenance; assert graph/body/descriptor disagreement fails without a partial aggregate.
- [x] Implement immutable lifecycle response records in `ContextLifecycleService` by composing the existing snapshot, inventory replay, and nearest-body replay ports; require a body for every surviving component and preserve removal as absence.
- [x] Reuse the established memory and PostgreSQL repository ports without reading current component projection as history; keep the same fail-closed storage errors.
- [x] Re-run the five focused `contextlab-storage` lifecycle service tests and the workspace suite until green.
- [ ] Add and observe a dedicated PostgreSQL lifecycle aggregate end-to-end case; deferred/unobserved, and not represented by the dated 25-case underlying storage baseline.

## Task 2: Framework-Independent Lifecycle Command Builder / 与框架无关的生命周期命令构建器

**Files:** Modify `crates/storage/src/context_lifecycle.rs` and the graph taxonomy API; add unit tests.

- [x] Write and observe failing service tests for `create`, `update`, and `remove` against a materialized parent state, asserting typed changes, exact attachments, graph insertion/update/removal, replay, stale-head rejection, and merge-head rejection.
- [x] Implement framework-independent command composition inside `ContextLifecycleService`, accepting validated lifecycle intent, exact head state, principal, idempotency key, request digest, branch, and message before producing one `GuardedContextCommitWrite`.
- [x] Reuse the graph taxonomy API from storage validation and lifecycle command composition while preserving all node and relationship kinds.
- [x] Re-run the focused lifecycle service and storage unit tests until green.

## Task 3: Protected Local API Contracts / 受保护的本地 API 契约

**Files:** Modify `server/api/src/lib.rs`, `server/api/src/routes.rs`, and API tests.

- [x] Write failing protected-router tests for local lifecycle state reads and each lifecycle mutation: missing/invalid bearer, reader denial with an audit event, rate-limit rejection, create/replay, update, removal, stale branch head, and public-router absence.
- [x] Run the targeted API tests and observe the routes and operation-scoped middleware are absent.
- [x] Add non-OpenAPI `GET /api/v1/local/contexts/{context_id}/commits/{commit_id}/lifecycle-state` and `POST /api/v1/local/contexts/{context_id}/component-lifecycle-commits`; require a non-null materialized expected head for mutations.
- [x] Split protected middleware by `ContextLifecycleRead` and `ContextCommitWrite` operations, preserve bearer authentication, RBAC, durable authorization auditing, bounded rate limits, and fail-closed errors; delegate all command semantics to `context-lifecycle` and persistence to the guarded writer.
- [x] Re-run targeted API tests and the public-route/OpenAPI contract tests until green.

**Observed evidence, 2026-07-15 / 已观察证据，2026-07-15：** Red tests first observed the missing local routes, a graph/descriptor aggregate disagreement, silent acceptance of client-owned `changes`, and incorrect `400` classification for an unavailable component. Green evidence includes the six focused protected lifecycle API cases, the aggregate graph-consistency regression, and the fresh workspace/Web gates recorded below. Boundary searches observed neither local path in checked-in OpenAPI nor local method in the public SDK, and found only the existing two API uses of `GraphDiff::between`. The dated 25-case PostgreSQL result is an underlying storage baseline, not lifecycle end-to-end evidence. Docker and `psql` were not used, so lifecycle-specific PostgreSQL, browser protected-runtime, remote CI, and production evidence remain unobserved and are not claimed.

红测先观察到 local route 缺失、graph/descriptor aggregate 不一致、静默接受调用方拥有的 `changes`，以及 unavailable component 被错误归类为 `400`。绿测证据包括 6 个聚焦 protected lifecycle API case、aggregate graph-consistency 回归，以及下文记录的新鲜 workspace/Web 门禁。边界搜索确认 checked-in OpenAPI 不含 local path，public SDK 不含 local method，且 `GraphDiff::between` 只保留现有两个 API 调用。带日期的 25-case PostgreSQL 结果是底层 storage baseline，不是 lifecycle 端到端证据。本轮未使用 Docker 与 `psql`，因此 lifecycle-specific PostgreSQL、browser protected-runtime、remote CI 与 production evidence 仍未观察且不作声称。

## Task 4: Non-Public Local SDK / 非公开本地 SDK

**Files:** Create `packages/local-sdk`; modify workspace manifests and Web dependency; add tests.

- [x] Write failing transport tests that send bearer and idempotency headers only to explicit local protected endpoints, preserve structured non-2xx errors, and never call public endpoints.
- [x] Run the package tests and observe the local SDK package is absent.
- [x] Implement `ContextLabLocalClient` with typed lifecycle state and mutation methods; require caller-supplied bearer tokens and retain no token storage.
- [x] Re-run local SDK tests, existing public SDK tests, and an OpenAPI boundary test proving `ContextLabClient` still has no mutation method.

**Observed evidence, 2026-07-15 / 已观察证据，2026-07-15：** `pnpm --filter @contextlab/local-sdk test` first failed after the package shell existed because `src/client` was absent; later red tests caught generic-JSON metadata drift, missing `Retry-After`, and ambient-cookie forwarding. After implementation, the local package reported 4 passing transport/credential/error tests and `tsc --noEmit` passed. `packages/ts-sdk/src/openapi-contract.test.ts` continues to prove both local paths and both hypothetical local methods are absent from the public contract.

`pnpm --filter @contextlab/local-sdk test` 在 package shell 已存在但 `src/client` 缺失时先失败；后续红测又捕获 generic-JSON metadata drift、缺失 `Retry-After` 与 ambient-cookie forwarding。实现后，local package 的 transport/credential/error 测试 4 项通过，`tsc --noEmit` 通过。`packages/ts-sdk/src/openapi-contract.test.ts` 继续证明两个 local path 和两个假设的 local method 都不在 public contract 中。

## Task 5: Design-System Local Web Workflow / 设计系统本地 Web 工作流

**Files:** Add UI primitives; create Web proxy/data/presenter/editor modules and route handlers; modify the workspace screen and tests.

- [x] Write and observe failing primitive, transport, presenter, editor, duplicate-submit, refresh-failure, protected-error, destructive-confirmation, and accessibility tests.
- [x] Add shared `Input` and `Textarea` primitives with described-by wiring; add same-origin route handlers that forward only request-scoped Authorization and Idempotency-Key headers without logging or persistence.
- [x] Implement data and presenter modules outside the React screen; render a compact bilingual lifecycle editor with memory-only bearer state, command-validity gating, explicit removal confirmation, and preserved client state across `router.refresh()`.
- [x] Re-run focused Web tests, `pnpm check:web`, and preview desktop/mobile Playwright accessibility/interaction checks.
- [ ] Run authenticated browser-to-BFF-to-protected-Axum lifecycle mutation smoke; deferred/unobserved because no authenticated local protected runtime was used.

## Task 6: Evidence and Documentation / 证据与文档

**Files:** Modify bilingual architecture, storage, workflow, active-goal, and completion-criteria documents; update this plan checklist.

- [x] Record the exact local-only boundary, required pre-existing materialized head, no-body historical failure behavior, public-contract exclusion, and GraphDiff single-calculator rule in English and Chinese.
- [x] Keep the existing 25-case disposable PostgreSQL storage baseline registered and statically checked; no lifecycle end-to-end PostgreSQL case is represented by that baseline.
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, boundary searches for public writes and `GraphDiff::between`, and preview desktop/mobile browser accessibility/interaction smoke.
- [ ] Run lifecycle-specific PostgreSQL E2E and authenticated browser-to-BFF-to-protected-Axum smoke; deferred/unobserved and unclaimed because Docker and `psql` were not used.
- [x] Update this plan with actual observed commands and results; then update roadmap evidence while keeping the long-term goal active.

## Final Fresh Local Receipts / 最终新鲜本地回执

The final current-worktree receipts are: API `121 passed`; storage `152 passed, 25 ignored`; public SDK `14 passed`; local SDK `4 passed`; and Web `33 passed`. Task 5 implementation and desktop/mobile preview accessibility/interaction QA are complete. Public OpenAPI and the public SDK remain mutation-free, and `GraphDiff::between` remains the sole graph-diff calculator.

当前工作树的最终新鲜回执为：API `121 passed`；storage `152 passed, 25 ignored`；public SDK `14 passed`；local SDK `4 passed`；Web `33 passed`。Task 5 的实现以及 desktop/mobile preview accessibility/interaction QA 均已完成。public OpenAPI 与 public SDK 仍不包含 mutation，`GraphDiff::between` 仍是唯一 graph-diff calculator。

These receipts are local implementation and preview evidence only. The 25-case PostgreSQL result remains an underlying storage baseline, not lifecycle PostgreSQL E2E evidence. Lifecycle PostgreSQL E2E, authenticated browser-to-BFF-to-protected-Axum smoke, remote CI, operator approval, public promotion, release, and production rollout remain deferred/unobserved; no publication or release is claimed.

这些回执仅证明本地实现与 preview 边界。25-case PostgreSQL 结果仍只是底层 storage baseline，不是 lifecycle PostgreSQL E2E 证据。lifecycle PostgreSQL E2E、authenticated browser-to-BFF-to-protected-Axum smoke、remote CI、operator approval、public promotion、release 与 production rollout 仍为 deferred/unobserved；本文不声称已经发布或上线。
