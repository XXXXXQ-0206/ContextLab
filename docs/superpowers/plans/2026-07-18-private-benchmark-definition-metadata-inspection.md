# Private Benchmark Definition Metadata Inspection Plan / 私有 Benchmark 定义元数据审阅计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` task-by-task. Preserve the existing private read-only path and checkbox evidence tracking.

**Goal / 目标：** Make the sealed suite and dataset metadata behind one exact local benchmark decision queryable and visible without exposing benchmark cases, inputs, expected outputs, run payloads, or a public contract.

**Architecture / 架构：** `contextlab-storage` owns a reusable summary projection that consumes the already-loaded sealed decision, then resolves only its project-scoped suite and declared datasets. The exact safe metadata shape is `definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }`; thresholds retain the suite's stable metric order and datasets are ordered by stable identifier. It returns no cases, inputs, expected outputs, run payloads, measurements, or policy output. The existing protected local decision GET reuses its current authentication, `ContextPermission::Read`, audit, rate-limit, and redaction boundary. The non-public local SDK, existing same-origin BFF, data/presenter, and design-system inspector transport and render the summary without fetching raw definitions or computing policy.

`contextlab-storage` 负责可复用 summary projection：它消费已经加载的 sealed decision，然后只解析其 project-scoped suite 与其声明的 dataset。精确的安全元数据形状为 `definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }`；threshold 保留 suite 稳定的 metric 顺序，dataset 按稳定 identifier 排序。它不返回 case、input、expected output、run payload、measurement 或 policy output。既有 protected local decision GET 复用当前 authentication、`ContextPermission::Read`、audit、rate-limit 与 redaction boundary。非公开 local SDK、既有同源 BFF、data/presenter 与 design-system inspector 会传输和呈现该 summary，而不获取 raw definition 或计算 policy。

**Tech Stack / 技术栈：** Rust 2024, `contextlab-storage`, Axum, Serde, `@contextlab/local-sdk`, Next.js, TypeScript, and `@contextlab/ui`.

---

## Necessity Record / 必要性记录

**Completion criteria / 服务条件：** Criterion 3 requires benchmark suites and datasets to be persisted, queryable, and visible in the Web workspace. Criterion 5 requires that visibility to remain in reusable data/presenter/design-system layers rather than page-local business logic.

条件 3 要求 benchmark suite 与 dataset 可持久化、可查询并在 Web workspace 中可见。条件 5 要求该可见性保持在可复用的 data/presenter/design-system 层，而不是页面局部业务逻辑中。

**Gap and priority / 缺口与优先级：** The current exact decision inspection exposes suite and dataset identifiers but cannot show their human-readable sealed metadata. Definitions are already immutable, exact-scope evidence exists, and the local protected reader, SDK/BFF, and inspector are verified. This is a smaller and more direct Criterion 3 convergence increment than Provider transport, execution HTTP, a dashboard, or component text diff.

当前的精确 decision inspection 会暴露 suite 与 dataset identifier，但不能显示其可读且已 seal 的元数据。definition 已不可变，精确 scope evidence 已存在，local protected reader、SDK/BFF 与 inspector 也已验证。它比 Provider transport、execution HTTP、dashboard 或 component text diff 更小、更直接地推进条件 3 收束。

**Scope / 范围：** Resolve only safe suite and dataset metadata from one already sealed exact project/Context/commit/decision scope through the existing private storage read boundary. The metadata contains only suite identifier/name/thresholds and dataset identifier/name/case count, with deterministic ordering and fail-closed membership checks.

仅通过既有私有 storage read boundary，从一条已经 seal 的精确 project/Context/commit/decision scope 解析安全的 suite 与 dataset 元数据。该元数据仅包含 suite identifier/name/threshold，以及 dataset identifier/name/case count，并具有确定性排序与 fail-closed membership check。

**Non-goals / 非目标：** No benchmark case, input, expected output, model output, per-run measurement, mutation, execution, provider call, new route, public REST/OpenAPI/public SDK method, Web mutation control, schema migration, GraphDiff change, Docker, secret access, release, production claim, external CI, or external/production receipt.

不包含 benchmark case、input、expected output、model output、逐 run measurement、mutation、execution、provider call、新 route、public REST/OpenAPI/public SDK method、Web mutation control、schema migration、GraphDiff 改动、Docker、密钥访问、release、production 声明、external CI 或 external/production 回执。

**Minimal boundary / 最小边界：** Add a storage-owned `BenchmarkDecisionDefinitionSummary` projection from an exact sealed decision plus its existing immutable definition reads; add a `definition` field only to the existing private `LocalBenchmarkDecisionResponse`; extend only its local SDK runtime parser and existing evidence data/presenter/inspector. Checked-in public route catalogs, public OpenAPI, public SDK, BFF route shape, and `diff-engine` remain unchanged.

最小边界是在 storage 中增加 `BenchmarkDecisionDefinitionSummary` projection，它从精确 sealed decision 及其既有不可变 definition read 构建；只向既有私有 `LocalBenchmarkDecisionResponse` 增加一个 `definition` 字段；只扩展其 local SDK runtime parser 和既有 evidence data/presenter/inspector。checked-in public route catalog、public OpenAPI、public SDK、BFF route shape 与 `diff-engine` 均保持不变。

**Fresh verification / 下一增量前验证：** Observe red then green tests for exact project/Context/commit/decision scope, suite/dataset membership mismatch or absence, deterministic dataset ordering, recursive raw-key rejection, existing auth/RBAC/rate-limit/public-404 behavior, local SDK exact-shape validation, BFF no-cookie/no-store preservation, bilingual presentation, TypeScript checks, Web build, `cargo fmt`, focused Clippy, workspace tests, and explicitly deferred PostgreSQL runtime.

下一增量前必须观察到：精确 project/Context/commit/decision scope、suite/dataset membership 不匹配或缺失、确定性 dataset 排序、递归 raw-key 拒绝、既有 auth/RBAC/rate-limit/public-404 行为、local SDK exact-shape validation、BFF no-cookie/no-store 保持、双语呈现、TypeScript check、Web build、`cargo fmt`、范围化 Clippy、workspace test，以及明确延期的 PostgreSQL runtime。

## Storage Red/Green Proof and Evidence Boundary / Storage 红绿证明与证据边界

**Red / 红：** The focused storage test was freshly observed red after it required `summarize(&evidence)`: the prior service required four scope arguments and failed with Rust `E0061`. The API red test then observed the absent `definition` response field, and the local SDK red tests observed acceptance of blank names and unsorted thresholds. These failures supplied the minimum corrections recorded below.

在 storage test 改为要求 `summarize(&evidence)` 后，聚焦命令新鲜观测到红态：旧 service 需要四个 scope 参数，产生 Rust `E0061`。随后 API 红测观测到缺失的 `definition` response field，local SDK 红测观测到空白 name 与无序 threshold 被接受。这些失败给出了下文记录的最小修复。

**Green / 绿：** `cargo test -p contextlab-storage --test benchmark_evidence definition_summary` now passes `2` focused in-memory tests. They prove the already-loaded evidence path, two-dataset identifier ordering, unavailable suite handling, and sealed-membership mismatch rejection without exposing case payloads. Focused API tests pass `10`; `pnpm check:web` passes the public SDK, local SDK, BFF, Web TypeScript/tests, and production Web build.

`cargo test -p contextlab-storage --test benchmark_evidence definition_summary` 现通过 `2` 个聚焦的内存 test。它们证明已加载 evidence 的路径、两个 dataset 的 identifier 排序、suite 不可用处理与 sealed-membership 不匹配拒绝，且不暴露 case payload。聚焦 API test 通过 `10` 项；`pnpm check:web` 通过 public SDK、local SDK、BFF、Web TypeScript/test 与 production Web build。

**Boundary / 边界：** Fresh local validation includes format, focused storage/API/SDK/Web checks, `cargo test --workspace --quiet`, and `pnpm check:web`. Strict Rust Clippy remains open only because the pre-existing Rust 1.85 MSRV lint in `crates/auth/src/authorization.rs:320` fails before the scoped storage/API crates complete. Docker remains disabled, so disposable PostgreSQL runtime, authenticated browser E2E, remote CI, and production evidence are unobserved and are not claimed.

新鲜本地验证包括 format、聚焦 storage/API/SDK/Web check、`cargo test --workspace --quiet` 与 `pnpm check:web`。严格 Rust Clippy 仍保持开放，唯一原因是既有 Rust 1.85 MSRV lint 在 `crates/auth/src/authorization.rs:320` 处失败，发生在范围化 storage/API crate 完成之前。Docker 仍关闭，因此 disposable PostgreSQL runtime、authenticated browser E2E、remote CI 与 production evidence 仍未观测，本文不作声称。

## File Map / 文件映射

- Modify: `crates/storage/src/benchmark_evidence.rs`, `crates/storage/src/lib.rs`
- Test: `crates/storage/tests/benchmark_evidence.rs`
- Modify: `server/api/src/routes.rs`, `server/api/src/lib.rs`
- Modify: `packages/local-sdk/src/types.ts`, `packages/local-sdk/src/client.test.ts`
- Modify: `apps/web/src/app/context-benchmark-evidence-data.ts`, `apps/web/src/app/context-benchmark-evidence-presenter.ts`, `apps/web/src/app/context-benchmark-evidence-inspector.tsx`, and their focused tests
- Modify: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and this plan

## Task 1: Project Safe Sealed Definition Metadata / 投影安全的已 Seal 定义元数据

- [x] **Step 1: Write failing storage tests.** Persist one decision containing two datasets, request its metadata summary at the exact project/Context/commit/decision scope, and assert suite name/thresholds plus datasets ordered by stable ID with `{ id, name, case_count }` only. Assert an absent suite, missing dataset, or summary membership mismatch fails closed without returning a partial summary.

- [x] **Step 2: Record the focused storage red condition.**

Run: `cargo test -p contextlab-storage --test benchmark_evidence definition_summary`

Expected before implementation: FAIL because the summary projection service does not exist. The historical terminal receipt is unavailable in this worktree; see the red/green evidence boundary above.

- [x] **Step 3: Implement the storage projection.** Add `BenchmarkDecisionDefinitionSummaryService` over the existing `BenchmarkEvidenceRepository`. It consumes the already-loaded exact decision, then reads its suite and each declared dataset from the project scope, verifies sealed suite/dataset membership, sorts summary datasets by ID, and returns no case payloads. It does not re-read the decision, re-evaluate runs, or write storage.

- [x] **Step 4: Run the focused storage test to verify green.**

Run: `cargo test -p contextlab-storage --test benchmark_evidence definition_summary`

Observed: PASS (`2 passed; 0 failed; 15 filtered out`).

## Task 2: Extend the Existing Private Decision Read / 扩展既有私有 Decision 读取

- [x] **Step 1: Write failing API tests.** Extend the existing exact-decision fixture to assert a `definition` response with suite name, deterministic dataset summaries, and threshold metadata; recursively assert no case/input/expected-output keys. Reassert public router `404`, authentication/RBAC/rate-limit behavior, and generic redaction for unavailable definition reads.

- [x] **Step 2: Run the focused API tests to verify red.**

Run: `cargo test -p contextlab-api local_benchmark_decision`

Observed: FAIL because the response had no `definition` projection.

- [x] **Step 3: Implement the narrow response extension.** Reuse the existing protected route and exact decision read; pass that already-loaded evidence to the storage summary only after authorization/rate limiting, map it into `LocalBenchmarkDecisionResponse`, and preserve generic error mapping. Do not install a new route or change checked-in public route/OpenAPI catalogs.

- [x] **Step 4: Run focused API tests to verify green.**

Run: `cargo test -p contextlab-api local_benchmark_decision`

Observed: PASS (`10 passed`) with public `404`, RBAC/rate-limit short-circuiting, generic definition failure redaction, and raw payload exclusion retained.

## Task 3: Extend Local SDK and Existing Inspector / 扩展 Local SDK 与既有 Inspector

- [x] **Step 1: Write failing local SDK and Web tests.** Require the new exact `definition` shape, reject unexpected/raw nested keys, preserve BFF no-cookie/private-no-store behavior, and render bilingual suite/dataset metadata in stable order through the existing evidence presenter and inspector.

- [x] **Step 2: Run focused tests to verify red.**

Run: `pnpm --filter @contextlab/local-sdk test`

Run: `pnpm --filter @contextlab/web test`

Observed: FAIL before parser/presentation support; follow-up parser red tests also rejected the missing nonblank-name and stable-threshold-order checks.

- [x] **Step 3: Implement only local read composition.** Extend `LocalBenchmarkDecision` and its strict parser; reuse existing BFF/data flow; add presenter models and a design-system metadata section using shared primitives. The browser never receives cases, inputs, expected outputs, runs, measurements, policy code, or mutation controls.

- [x] **Step 4: Run focused local SDK and Web tests to verify green.**

Run: `pnpm --filter @contextlab/local-sdk lint`

Run: `pnpm --filter @contextlab/local-sdk test`

Run: `pnpm --filter @contextlab/web lint`

Run: `pnpm --filter @contextlab/web test`

Observed: PASS; local SDK has `22` passing subtests and Web has `49` passing tests in the final `pnpm check:web` run.

## Task 4: Document and Verify / 文档与验证

- [x] **Step 1: Record bilingual architecture boundaries.** Document the exact sealed-definition projection, redaction boundary, existing private route reuse, unchanged public contracts, no GraphDiff change, and deferred PostgreSQL runtime.

- [x] **Step 2: Run final scope-matched validation.**

Run: `cargo fmt --all -- --check`

Run: `cargo clippy -p contextlab-storage -p contextlab-api --all-targets -- -D warnings`

Run: `cargo test --workspace`

Run: `pnpm check:web`

Observed: `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, and `pnpm check:web` pass. `cargo clippy -p contextlab-storage -p contextlab-api --all-targets -- -D warnings` remains blocked only by the pre-existing Rust 1.85 MSRV lint at `crates/auth/src/authorization.rs:320`. PostgreSQL runtime remains deferred while Docker is disabled.

- [x] **Step 3: Record exact evidence and retain the active long-term goal.** This plan records the fresh local storage/API/SDK/BFF/Web evidence and the Clippy limitation above. Docker/PostgreSQL runtime, browser E2E, remote CI, and production evidence remain unobserved; the long-term goal remains active.
