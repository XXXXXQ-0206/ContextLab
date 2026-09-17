# Private Sealed Benchmark Decision Run Details Plan / 私有已封存 Benchmark Decision Run 明细计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` task-by-task. Preserve the existing protected local read path, no public-contract boundary, and `GraphDiff::between` as the sole graph-diff calculator.

**Goal / 目标：** Make the ordered, exact-decision-scoped run cohort behind one sealed benchmark decision queryable and visible through the existing private local inspection workflow without exposing cases, inputs, expected outputs, model outputs, or evaluator diagnostics.

**Architecture / 架构：** `BenchmarkDecisionEvidence` already seals ordered run identifiers and `BenchmarkEvidenceRepository` already resolves one run at an exact `(project, Context, commit, run)` scope. A storage-owned read service will consume the already-loaded sealed evidence, resolve every listed run in stored order, and fail closed if any run is absent or escapes that scope. A distinct protected local GET, non-public local SDK model, and same-origin BFF response will expose only the redacted run summary because the existing aggregate-decision parser deliberately rejects nested measurements; the existing inspector composes both exact reads without recomputing policy or `GraphDiff`.

**Tech Stack / 技术栈：** Rust 2024, `contextlab-evaluation`, `contextlab-storage`, Axum, Serde, `@contextlab/local-sdk`, Next.js, TypeScript, and `@contextlab/ui`.

---

## Necessity Record / 必要性记录

**Completion criteria and charter principles / 服务的完成条件与宪章原则：** This directly advances Criterion 3, which requires persisted, queryable, and Web-visible benchmark run details, numeric scorecards, thresholds, and evaluation-diff workflows. It also advances Criterion 5 by keeping projection and presentation in reusable storage/data/presenter/design-system layers rather than page-local business logic.

本增量直接推进条件 3：benchmark run detail、数值 scorecard、threshold 与 evaluation-diff workflow 必须可持久化、可查询并在 Web 中可见。它也推进条件 5：projection 与呈现应位于可复用的 storage/data/presenter/design-system 层，而不是页面局部业务逻辑。

**Gap, dependencies, and risk / 缺口、依赖与风险：** The immutable evidence records ordered `run_ids`, and the execution service persists/replays the cohort, but a caller can currently inspect only aggregate decision facts. `BenchmarkEvidenceRepository::get_benchmark_run` has no decision-membership projection, so the existing generic evaluation-run reader must not be used as a substitute. Definitions, sealed evidence, exact protected decision reads, strict local-SDK parsing, private/no-store BFF forwarding, and shared inspector primitives already exist. The risk is leaking raw benchmark payloads or returning a run that is not a member of the sealed decision; the new projection must start from the already-authorized evidence and resolve only its ordered membership.

不可变 evidence 会记录有序 `run_ids`，execution service 也会持久化或 replay cohort，但调用方目前只能审阅 aggregate decision fact。`BenchmarkEvidenceRepository::get_benchmark_run` 尚无按 decision membership 投影，因此不能把既有通用 evaluation-run reader 误当作替代。definition、sealed evidence、精确 protected decision read、严格 local-SDK parser、private/no-store BFF forwarding 与共享 inspector primitive 都已具备。风险在于泄露 raw benchmark payload 或返回并不属于 sealed decision 的 run；新 projection 必须从已完成授权的 evidence 出发，只解析其有序 membership。

**Why now / 为什么现在优先：** The prior lifecycle increment has fresh local evidence. The project already has benchmark definition metadata inspection, decision aggregate inspection, immutable decision diff, and provider-free execution orchestration. Showing the sealed cohort is the smallest missing vertical evidence needed to make those persisted benchmark runs inspectable before any definition editing, execution transport, provider integration, dashboard expansion, or public promotion.

前一生命周期增量已取得新鲜本地证据。项目已有 benchmark definition metadata inspection、decision aggregate inspection、不可变 decision diff 与无 Provider execution orchestration。展示 sealed cohort 是使已持久化 benchmark run 可审阅所需的最小缺口，优先于 definition editing、execution transport、Provider 集成、dashboard 扩张或 public promotion。

**Explicit non-goals / 明确非目标：** No evaluator invocation, benchmark definition mutation, persistence or migration change, scheduler, queue, provider/model call, secret access, caller-supplied measurements, policy or scorecard recalculation, cases, inputs, expected outputs, model outputs, evaluator diagnostics, public REST/OpenAPI/public SDK, Web mutation, `GraphDiff` change, Docker/PostgreSQL runtime, authenticated browser E2E, release, remote CI, or production claim.

不包含 evaluator 调用、benchmark definition mutation、持久化或 migration 变更、scheduler、queue、Provider/model 调用、密钥访问、调用方提供的 measurement、policy 或 scorecard 重算、case、input、expected output、model output、evaluator diagnostic、public REST/OpenAPI/public SDK、Web mutation、`GraphDiff` 改动、Docker/PostgreSQL runtime、authenticated browser E2E、release、remote CI 或 production 声明。

**Minimal affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Modify the existing benchmark-evidence read service and tests; add one protected local GET and same-origin BFF route; add one non-public local SDK exact parser/model; and compose its safe result in the existing benchmark-evidence data/presenter/inspector tests. Update bilingual architecture, storage, API, roadmap, completion, and this plan. Leave migration assets, checked-in public contracts, `model-gateway`, and all write routes unchanged.

最小受影响边界是既有 benchmark-evidence read service 与测试、既有 protected local decision response 与测试、非公开 local SDK exact parser 与测试，以及既有 benchmark-evidence data/presenter/inspector 测试。更新中英双语 architecture、storage、API、roadmap、completion 与本计划。migration 资产、public contract、`model-gateway` 和所有 write route 保持不变。

**Fresh verification before another increment / 下一增量前的新鲜验证：** Observe red then green storage/API/local-SDK/Web tests for ordered membership, missing member fail-closed behavior, raw-key exclusion, exact-scope authorization, public-router exclusion, no-cookie/private-no-store preservation, and bilingual rendering. Then run `cargo fmt --all -- --check`, focused Clippy with its existing unrelated MSRV blocker recorded, `cargo test --workspace --quiet`, `pnpm --filter @contextlab/ts-sdk test`, and `pnpm check:web`. Compile any PostgreSQL coverage without running ignored tests while Docker remains disabled.

下一增量前必须观察到：ordered membership、缺失 member fail-closed、raw-key exclusion、exact-scope authorization、public-router exclusion、no-cookie/private-no-store 保持与双语呈现的红绿 storage/API/local-SDK/Web 测试。随后运行 `cargo fmt --all -- --check`、范围化 Clippy（记录既有无关 MSRV blocker）、`cargo test --workspace --quiet`、`pnpm --filter @contextlab/ts-sdk test` 与 `pnpm check:web`。Docker 保持关闭时，只编译任何 PostgreSQL coverage，不运行 ignored test。

## File Map / 文件映射

- Modify: `crates/storage/src/benchmark_evidence.rs` and `crates/storage/src/lib.rs`.
- Test: `crates/storage/tests/benchmark_evidence.rs`.
- Modify: `server/api/src/routes.rs` and `server/api/src/lib.rs`.
- Modify: `packages/local-sdk/src/types.ts` and `packages/local-sdk/src/client.test.ts`.
- Modify: `apps/web/src/app/context-lifecycle-proxy.ts`, add the matching `apps/web/src/app/api/local/.../benchmark-decision-run-details/route.ts`, and modify `apps/web/src/app/context-benchmark-evidence-data.ts`, `apps/web/src/app/context-benchmark-evidence-presenter.ts`, `apps/web/src/app/context-benchmark-evidence-inspector.tsx`, and their existing tests.
- Modify: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/api/local-context-lifecycle.md`, `docs/roadmap/active-long-term-goal.md`, and `docs/roadmap/completion-criteria.md`.

## Task 1: Project Exact Sealed Run Membership / 投影精确的已封存 Run Membership

- [x] **Step 1: Write failing storage tests.** Require a summary service to consume one loaded `BenchmarkDecisionEvidence`, resolve its `run_ids` in stored order, return only run id/model version/temperature/executed time/numeric measurements, and fail when one sealed member is missing.

- [x] **Step 2: Run the red test.** Run `cargo test -p contextlab-storage --test benchmark_evidence decision_run_details` and observe the absent projection service.

- [x] **Step 3: Implement the minimal storage read service.** Add a `BenchmarkDecisionRunDetailsService` that accepts project and Context identifiers plus already-loaded evidence, verifies exact evidence scope, calls `get_benchmark_run` only for the evidence's ordered members, and returns a typed summary without recalculating evaluation policy or writing storage.

- [x] **Step 4: Run the green test.** Re-run the focused storage test and compile the named ignored PostgreSQL coverage without starting Docker.

## Task 2: Add a Distinct Private Run-Details Read / 增加独立的私有 Run 明细读取

- [x] **Step 1: Write failing protected API tests.** Require one exact private run-details GET to return ordered safe run summaries, reject unavailable members generically, reuse the existing decision-read authentication/RBAC/audit/rate-limit boundary, and remain absent from the public router.

- [x] **Step 2: Run the red API test.** Run `cargo test -p contextlab-api local_benchmark_decision_run_details` and observe the absent private route and response.

- [x] **Step 3: Implement the narrow protected route.** Load the exact decision after the existing protected read boundary, map storage summaries to a separate private response, and add no public router, OpenAPI operation, or public SDK method.

- [x] **Step 4: Run the green API test.** Re-run the focused protected API tests.

## Task 3: Compose the Separate Private SDK and Design-System Read Path / 组合独立的私有 SDK 与设计系统读取路径

- [x] **Step 1: Write failing local SDK and Web tests.** Require a separate exact safe run-details shape, rejection of raw/unknown keys without weakening the aggregate decision parser, BFF no-cookie/private-no-store preservation, and bilingual shared-primitive rendering in the existing inspector.

- [x] **Step 2: Run the red TypeScript tests.** Run the focused local SDK and Web tests and observe the missing parser/presenter fields.

- [x] **Step 3: Implement local read composition only.** Add the non-public parser/client method and same-origin BFF route, then extend existing data/presenter and shared-primitive inspector; do not add a mutation control or page-local evaluation logic.

- [x] **Step 4: Run the green TypeScript tests.** Re-run focused local SDK and Web tests.

## Task 4: Document and Verify / 文档与验证

- [x] **Step 1: Update bilingual architecture boundaries.** Document ordered sealed membership, redaction limits, unchanged public contracts, `GraphDiff::between` preservation, and deferred Docker/browser evidence.

- [x] **Step 2: Run full local verification.** Record only observed formatting, Clippy, workspace, SDK, Web, and compiled-PostgreSQL output.

- [x] **Step 3: Keep the long-term goal active.** Select the next dependency-ready core increment only after fresh evidence.

## Documentation Boundary and Evidence Status / 文档边界与证据状态

This plan admits a private local read path for one sealed benchmark decision. It resolves only the exact ordered `run_ids` recorded by the sealed evidence at project/Context/commit/run scope, preserves stored order, and exposes only a fail-closed redacted run summary. Missing or out-of-scope membership is unavailable; raw cases, inputs, expected outputs, model outputs, and evaluator diagnostics are never part of the response.

本计划准入一条针对单个 sealed benchmark decision 的私有 local read path。它只在 project/Context/commit/run scope 内解析 sealed evidence 记录的精确有序 `run_ids`，保留存储顺序，并只暴露 fail-closed 的 redacted run summary。缺失或越界 membership 会保持 unavailable；raw case、input、expected output、model output 与 evaluator diagnostic 永远不属于 response。

The plan adds no public REST, OpenAPI, or public SDK write contract and does not change `GraphDiff::between` or introduce another graph-diff calculator. Docker/PostgreSQL runtime and authenticated browser E2E remain unobserved. No implementation or verification result is claimed by this documentation update.

本计划不新增 public REST、OpenAPI 或 public SDK write contract，不改变 `GraphDiff::between`，也不引入第二个 graph-diff calculator。Docker/PostgreSQL runtime 与 authenticated browser E2E 仍未观测。本次文档更新不声称任何 implementation 或 verification result。

Fresh main-thread evidence / main-thread 新鲜证据：

- [x] Storage/API/local-SDK/Web red-green tests cover ordered membership, missing-member fail-closed behavior, exact scope, redaction, public-router exclusion, no-cookie/private-no-store behavior, and bilingual rendering. The storage order assertion was additionally rerun five times after resolving fixture data by the sealed ID.
- [x] `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, `pnpm --filter @contextlab/ts-sdk test`, and `pnpm check:web` passed. Focused strict Clippy reaches only the existing unrelated Rust 1.85 MSRV lint at `crates/auth/src/authorization.rs:320`.
- [x] PostgreSQL coverage compiled with `--no-run`; ignored runtime execution remains unobserved while Docker is disabled.
