# Private Benchmark Evaluation Diff Implementation Plan / 私有 Benchmark Evaluation Diff 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` task-by-task. Steps use checkbox (`- [ ]`) tracking and preserve the local read-only boundary.

**Goal / 目标：** Compare two sealed benchmark decisions in a reusable evaluation-domain engine, then expose the result only through the existing authenticated local API, non-public SDK/BFF, and design-system Web workspace.

**Architecture / 架构：** `contextlab-evaluation` owns a pure comparison over normalized decision status and metric evidence; it must not depend on `contextlab-storage`, Axum, or React. `contextlab-storage` projects immutable `BenchmarkDecisionEvidence` into that input and retrieves both exact scopes before delegating to the sole comparison engine. The protected local route, local SDK, BFF, and Web presenter transport and render that result without recalculating thresholds, coverage, status, or GraphDiff.

**Tech Stack / 技术栈：** Rust stable, `contextlab-evaluation`, `contextlab-storage`, Axum, Serde, TypeScript, Next.js, `@contextlab/local-sdk`, and `@contextlab/ui`.

---

## Necessity Record / 必要性记录

**Criterion / 条件：** Completion criteria 2 and 3 require reusable evaluation diff workflows and benchmark decisions that are persisted, queryable, and visible in the Web workspace. The charter requires all Context changes and evaluation evidence to be comparable without page-local business logic.

完成条件 2 与 3 要求可复用的 evaluation diff workflow，以及可持久化、可查询且在 Web workspace 中可见的 benchmark decision。项目宪章要求所有 Context change 与 evaluation evidence 均可比较，且不得将业务逻辑放进页面。

**Gap and priority / 缺口与优先级：** One sealed decision can now be inspected safely at exact scope, but no reusable engine compares two immutable decision artifacts. This is the next dependency-ready choice because datasets, thresholds, exact-commit evidence, local protected inspection, response validation, and design-system presentation are verified. It precedes benchmark execution, dashboard expansion, A/B orchestration, semantic/behavior diff, and public promotion because those would otherwise duplicate or bypass a decision comparison contract.

一条 sealed decision 现已可在精确 scope 内被安全审阅，但尚无可复用 engine 比较两条不可变 decision artifact。由于 dataset、threshold、精确 commit evidence、local protected inspection、response validation 和 design-system presentation 均已验证，这是下一个依赖就绪的选择。它优先于 benchmark execution、dashboard 扩张、A/B orchestration、semantic/behavior diff 与 public promotion，以避免后续能力重复或绕过 decision comparison contract。

**Non-goals / 非目标：** No benchmark execution or provider call; no dataset, suite, run, or decision mutation; no public REST/OpenAPI/public SDK method; no public Web control; no dashboard/A-B workflow; no semantic/behavior diff; no GraphDiff change or second graph-diff calculator; no Docker, release, production, or external evidence claim.

不包含 benchmark execution 或 provider call；不包含 dataset、suite、run 或 decision mutation；不新增 public REST/OpenAPI/public SDK method；不新增 public Web control；不包含 dashboard/A-B workflow；不包含 semantic/behavior diff；不改变 GraphDiff 或新增第二个 graph-diff calculator；不涉及 Docker、release、production 或外部证据声明。

**Minimal boundary / 最小边界：** Add a pure `BenchmarkDecisionDiff` input/result contract in `contextlab-evaluation`; add one storage service that reads two exact project/Context/commit/decision scopes and delegates exactly once; add one authenticated protected local GET with explicit baseline/revised commit and decision query fields; add only matching local SDK/BFF/data/presenter/inspection composition. The Web layer receives no raw cases, inputs, expected outputs, per-run model payloads, or threshold policy implementation.

最小边界是在 `contextlab-evaluation` 中新增纯 `BenchmarkDecisionDiff` 输入/结果 contract；新增一项 storage service，用于读取两组精确 project/Context/commit/decision scope 并且只委托一次；新增一个 authenticated protected local GET，显式带 baseline/revised commit 与 decision query field；只新增对应的 local SDK/BFF/data/presenter/inspection 组合。Web 层不得接收 raw case、input、expected output、逐 run model payload 或 threshold policy implementation。

**Fresh verification before another increment / 下一增量前的新鲜验证：** Focused Rust red/green tests for status/metric additions, removals, value and coverage changes, and deterministic ordering; storage exact-scope and no-recalculation tests; protected API authentication/RBAC/rate-limit/redaction tests; local SDK/BFF shape/no-cookie/no-store tests; Web presenter and UI tests; `cargo fmt --all -- --check`, scoped Clippy, `cargo test --workspace`, package TypeScript checks, Web build, and explicit PostgreSQL runtime status.

下一增量前必须获得的新鲜验证包括：针对 status/metric 新增、删除、值与 coverage 变更及确定排序的 Rust 红绿测试；storage 精确 scope 与不重新计算测试；protected API authentication/RBAC/rate-limit/redaction 测试；local SDK/BFF shape/no-cookie/no-store 测试；Web presenter 与 UI 测试；`cargo fmt --all -- --check`、范围化 Clippy、`cargo test --workspace`、package TypeScript check、Web build，以及明确的 PostgreSQL runtime 状态。

**Blocked condition, minimal fix, and regression proof / 阻塞条件、最小修复与回归证明：** Preliminary storage audit found that independently reading two decisions can mix repository snapshots, and metric/status comparison without comparability fingerprint equality can compare different evaluator, model, suite, dataset, or Context conditions. The minimal fix is to add the immutable fingerprint to the pure comparison input, fail closed on mismatch, and add one pair-read port that obtains both exact scopes under one memory lock or one PostgreSQL repeatable-read transaction. Regression proof must cover equal fingerprints, mismatched fingerprints, exact commit/decision pairing, and a missing side without invoking `BenchmarkEvaluation::from_runs` on the read path.

**阻塞条件、最小修复与回归证明：** 初步 storage 审计发现，独立读取两条 decision 可能混合 repository snapshot；若不要求 comparability fingerprint 相等，则 metric/status 比较可能混入不同 evaluator、model、suite、dataset 或 Context 条件。最小修复是在纯 comparison input 中加入 immutable fingerprint、不匹配时 fail closed，并增加一个 pair-read port，使两条精确 scope 在同一 memory lock 或同一 PostgreSQL repeatable-read transaction 下取得。回归证明必须覆盖相同 fingerprint、不匹配 fingerprint、精确 commit/decision 配对以及缺失任一侧，且不得在 read path 调用 `BenchmarkEvaluation::from_runs`。

## Task 1: Define the Pure Evaluation-Diff Contract / 定义纯 Evaluation-Diff Contract

**Files / 文件：**
- Create: `crates/evaluation/src/decision_diff.rs`
- Modify: `crates/evaluation/src/lib.rs`
- Test: `crates/evaluation/tests/benchmark_decision_diff.rs`

- [x] **Step 1: Write failing comparison tests.** Add fixtures for a passed baseline and regressed revision with one removed metric, one added metric, one changed observed value, and one changed coverage fact. Assert the result contains a status change plus metric rows ordered by metric identifier.

```rust
let diff = BenchmarkDecisionDiff::between(baseline, revised);
assert_eq!(diff.status_change(), Some((RegressionDecisionStatus::Passed, RegressionDecisionStatus::Regressed)));
assert_eq!(diff.metric_changes()[0].metric(), MetricKind::Accuracy);
```

- [x] **Step 2: Run the new test to verify it fails.**

Run: `cargo test -p contextlab-evaluation --test benchmark_decision_diff`

Expected: FAIL because `BenchmarkDecisionDiff` and its input projection do not exist.

- [x] **Step 3: Implement the minimal domain value objects.** Define `BenchmarkDecisionComparisonInput` from status plus normalized metric evidence, `BenchmarkMetricComparisonInput`, typed additions/removals/changes, and `BenchmarkDecisionDiff::between`. Sort all metric identifiers by stable Rust ordering; reject duplicate metric inputs before comparison.

```rust
pub fn between(
    baseline: BenchmarkDecisionComparisonInput,
    revised: BenchmarkDecisionComparisonInput,
) -> Result<Self, BenchmarkDecisionDiffError>;
```

- [x] **Step 4: Run the focused domain test to verify it passes.**

Run: `cargo test -p contextlab-evaluation --test benchmark_decision_diff`

Expected: PASS with status, metric-add/remove, value, coverage, and deterministic-order assertions green.

## Task 2: Project Immutable Storage Evidence / 投影不可变 Storage Evidence

**Files / 文件：**
- Modify: `crates/storage/src/benchmark_evidence.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`
- Test: `crates/storage/tests/benchmark_evidence.rs`

- [x] **Step 1: Write failing storage tests.** Persist two decisions at distinct exact commits, request a comparison, assert each repository receives both complete scopes, and assert no storage adapter re-evaluates threshold policy.

```rust
let diff = repository
    .compare_benchmark_decisions(project_id, context_id, baseline, revised)
    .await?;
assert_eq!(diff.status_change(), Some((Passed, Regressed)));
```

- [x] **Step 2: Run the focused storage test to verify it fails.**

Run: `cargo test -p contextlab-storage --test benchmark_evidence comparison`

Expected: FAIL because the comparison repository/service does not exist.

- [x] **Step 3: Add the exact-scope comparison service.** Define `BenchmarkDecisionComparisonScope` with commit and decision identities, load each `BenchmarkDecisionEvidence` through the existing repository contract, project only status and metric evidence into the evaluation input, and call `BenchmarkDecisionDiff::between` once. Preserve generic storage errors and never deserialize cases or run measurement payloads.

```rust
async fn compare_benchmark_decisions(
    &self,
    project_id: ProjectId,
    context_id: ContextId,
    baseline: BenchmarkDecisionComparisonScope,
    revised: BenchmarkDecisionComparisonScope,
) -> Result<BenchmarkDecisionDiff, StorageRepositoryError>;
```

- [x] **Step 4: Run focused storage and migration checks.**

Run: `cargo test -p contextlab-storage --test benchmark_evidence`

Expected: PASS; existing persistence/replay tests and new exact-scope comparison tests remain green. PostgreSQL ignored tests remain explicitly unobserved unless a disposable database is supplied.

## Task 3: Add the Protected Local Read Contract / 新增 Protected Local 读取契约

**Files / 文件：**
- Modify: `server/api/src/lib.rs`
- Modify: `server/api/src/routes.rs`
- Modify: `crates/auth/src/rate_limit.rs`
- Modify: `crates/storage/src/protected_route_rate_limit.rs`
- Test: `server/api/src/lib.rs`

- [x] **Step 1: Write failing API tests.** Require a private-only route, bearer authentication, `ContextPermission::Read`, a dedicated comparison read rate-limit operation, both exact decision scopes, recursive redaction, and generic missing/storage errors.

```rust
let response = protected_router.oneshot(benchmark_decision_diff_request(
    project_id, context_id, baseline_commit, baseline_decision, revised_commit, revised_decision, token,
)).await?;
assert_eq!(response.status(), StatusCode::OK);
```

- [x] **Step 2: Run the focused API test to verify it fails.**

Run: `cargo test -p contextlab-api local_benchmark_decision_diff`

Expected: FAIL because the protected local route and comparison projection do not exist.

- [x] **Step 3: Implement the minimal private GET.** Add only `/api/v1/local/projects/{project_id}/contexts/{context_id}/benchmark-decision-diffs` with `baseline_commit_id`, `baseline_decision_id`, `revised_commit_id`, and `revised_decision_id` query fields. Reuse the protected router/middleware, exact-scope storage service, repository opt-in, generic errors, recursive response redaction, and a comparison-specific limiter operation. Keep checked-in OpenAPI and public route catalogs unchanged.

- [x] **Step 4: Run focused API tests to verify they pass.**

Run: `cargo test -p contextlab-api local_benchmark_decision_diff`

Expected: PASS; public router remains `404` and every rejection occurs before comparison loading.

## Task 4: Compose Local SDK, BFF, and Web Review / 组合 Local SDK、BFF 与 Web 审阅

**Files / 文件：**
- Modify: `packages/local-sdk/src/types.ts`
- Modify: `packages/local-sdk/src/client.ts`
- Modify: `packages/local-sdk/src/index.ts`
- Test: `packages/local-sdk/src/client.test.ts`
- Create: `apps/web/src/app/context-benchmark-decision-diff-data.ts`
- Create: `apps/web/src/app/context-benchmark-decision-diff-presenter.ts`
- Create: `apps/web/src/app/context-benchmark-decision-diff-inspector.tsx`
- Modify: `apps/web/src/app/context-lifecycle-proxy.ts`
- Create: `apps/web/src/app/api/local/projects/[projectId]/contexts/[contextId]/benchmark-decision-diffs/route.ts`
- Modify: `apps/web/src/app/context-workspace-screen.tsx`
- Modify: `apps/web/src/app/globals.css`
- Test: `apps/web/src/app/context-benchmark-decision-diff-*.test.ts(x)`

- [x] **Step 1: Write failing SDK/BFF/Web tests.** Test fully encoded baseline/revised query fields, request-scoped bearer forwarding, omitted cookies, `private, no-store` responses, recursive validation, disabled scope controls while loading, bilingual status/metric change labels, and deterministic order.

```ts
await client.getBenchmarkDecisionDiff(projectId, contextId, baseline, revised, { bearerToken });
assert.equal(response.headers.get("cache-control"), "private, no-store");
```

- [x] **Step 2: Run the focused tests to verify they fail.**

Run: `pnpm --filter @contextlab/local-sdk test` and `pnpm --filter @contextlab/web test`

Expected: FAIL because the local diff client, BFF proxy, presenter, and inspector do not exist.

- [x] **Step 3: Implement read-only composition.** Reuse the local SDK runtime validator pattern, same-origin BFF error/cache boundary, and shared `DefinitionGrid`, `StackTable`, `StatusPill`, `Input`, and `Select` primitives. The inspector must render only domain-produced changes; no threshold, status, coverage, or graph diff is recalculated in TypeScript.

- [x] **Step 4: Run focused checks to verify they pass.**

Run: `pnpm --filter @contextlab/local-sdk lint`, `pnpm --filter @contextlab/local-sdk test`, `pnpm --filter @contextlab/web lint`, and `pnpm --filter @contextlab/web test`

Expected: PASS with private-cache, raw-payload, scope-lock, bilingual, and deterministic-order coverage green.

## Task 5: Document and Verify / 文档与验证

**Files / 文件：**
- Modify: `ARCHITECTURE.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/active-long-term-goal.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/superpowers/plans/2026-07-18-private-benchmark-evaluation-diff.md`

- [x] **Step 1: Write bilingual boundary documentation.** Record the pure evaluation-domain owner, exact two-decision scope, local-only route/SDK/BFF/Web boundary, unchanged public API/SDK and `GraphDiff`, and any unobserved PostgreSQL runtime evidence.

- [x] **Step 2: Run final validation.**

Run: `cargo fmt --all -- --check`, `cargo clippy -p contextlab-evaluation -p contextlab-storage -p contextlab-api --all-targets -- -D warnings`, `cargo test --workspace`, and `pnpm check:web`.

Observed locally on 2026-07-18, without Docker, database provisioning, or secret access:

- `cargo fmt --all -- --check`: PASS.
- `cargo test --workspace`: PASS. The benchmark-diff coverage passed; storage reported `159 passed, 0 failed, 32 ignored`, including the PostgreSQL benchmark comparison runtime test as compiled/ignored.
- `pnpm check:web`: PASS. TypeScript SDK tests reported `14` passes, local SDK tests `8` passes, Web tests `47` passes, and the production Web build completed.
- `cargo clippy -p contextlab-evaluation -p contextlab-storage -p contextlab-api --all-targets -- -D warnings`: BLOCKED by the unrelated `contextlab-auth` MSRV lint at `crates/auth/src/authorization.rs:320`. The workspace MSRV is Rust `1.85.0`, while `self.0.is_empty()` in a `const` context requires Rust `1.87.0`; `-D warnings` promotes `clippy::incompatible-msrv` to an error. No source change was made in this validation pass.

The expected all-green final quality gate remains open until the auth MSRV blocker is resolved or explicitly waived. PostgreSQL runtime evidence is deferred: no disposable `CONTEXTLAB_TEST_DATABASE_URL` was supplied, so ignored PostgreSQL tests, including `postgres_benchmark_evidence_compares_two_exact_commit_scopes_in_one_consistent_read`, remain compiled/ignored/unobserved.

- [x] **Step 3: Record exact fresh evidence in this owned plan.** The observed local evidence and deferred PostgreSQL runtime status are recorded above. Roadmap handoff remains outside this plan-only ownership boundary; the active long-term goal is not claimed complete.
