# Private Benchmark Execution Orchestration Plan / 私有 Benchmark 执行编排计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) tracking and preserve the private, local-only boundary.

**Goal / 目标：** Execute a sealed benchmark suite through an injected local evaluator, create one deterministic run cohort, and atomically persist the resulting immutable decision evidence so existing protected inspection and diff paths can review it.

**Architecture / 架构：** `contextlab-evaluation` owns a pure, deterministic case-plan, composite case key, and provenance-preserving cohort assembly contract. Each run ID is UUIDv5-derived from the immutable decision identity plus `(dataset_id, case_id)`, so the existing immutable writer preserves a replayable case-to-run mapping without a second persistence path. `contextlab-storage` owns asynchronous evaluator invocation, exact project/Context/commit definition loading, existing-decision replay avoidance, domain policy evaluation, and the sole `BenchmarkEvidenceWriter` persistence call. No HTTP handler, provider client, browser control, public contract, or page-local evaluation policy is added in this increment.

**Tech Stack / 技术栈：** Rust 2024, `contextlab-evaluation`, `contextlab-storage`, Tokio test runtime, Serde, and existing immutable benchmark evidence contracts.

---

## Necessity Record / 必要性记录

**Completion criteria and charter principles / 服务的完成条件与宪章原则：** Criterion 3 requires benchmark suites, datasets, run details, numeric scorecards, regression thresholds, and evaluation diff views to be persisted, queryable, and visible. Criteria 1 and 2 require Context-bound artifacts to be reusable and replayable. The charter requires evaluation behavior to live in reusable Rust infrastructure rather than in routes or pages.

条件 3 要求 benchmark suite、dataset、run detail、数值 scorecard、regression threshold 与 evaluation diff view 可持久化、可查询且在 Web workspace 可见。条件 1 和 2 要求 Context 绑定 artifact 可复用、可回放。项目宪章要求 evaluation behavior 位于可复用的 Rust 基础设施中，而不是 route 或页面中。

**Gap, dependency, and risk / 缺口、依赖与风险：** The repository can persist sealed definitions and decisions, inspect one decision, and compare two decisions, but it cannot turn immutable cases into a run cohort. Existing storage supplies exact definition reads, decision identity, evidence sealing, and an atomic writer. The risk is accepting caller-supplied measurements or recomputing policy outside `BenchmarkEvaluation`; the service must obtain all case data from sealed storage and assemble the cohort only through the evaluation domain.

仓库已经能持久化已 seal 的 definition 与 decision、审阅单条 decision 并比较两条 decision，但还不能把不可变 case 转化为 run cohort。现有 storage 已提供精确的 definition read、decision identity、evidence seal 与原子 writer。风险在于接受调用方提供的 measurement，或在 `BenchmarkEvaluation` 之外重新计算 policy；因此服务必须只从已 seal storage 取得 case，并只经 evaluation domain 组装 cohort。

**Why now / 为什么现在优先：** The local Context lifecycle is complete, benchmark definitions/thresholds/evidence are persisted, and the decision inspection/diff workflow is verified. Execution is the direct missing link in Criterion 3 and is more convergent than another inspection panel, descriptor field, provider transport, or dashboard expansion.

本地 Context 生命周期已经完成，benchmark definition/threshold/evidence 已持久化，decision inspection/diff workflow 已验证。execution 是条件 3 直接缺失的一环，比继续新增 inspection panel、descriptor field、provider transport 或 dashboard 扩张更有助于收束。

**Explicit non-goals / 明确非目标：** No provider HTTP client, API-key access, model invocation, public REST/OpenAPI/public SDK/Web mutation, local API transport, browser control, dataset/suite editing, scheduler, queue, concurrent execution claim, dashboard expansion, GraphDiff change, second graph-diff calculator, Docker, PostgreSQL runtime, release, or production claim. A deterministic fixture evaluator exists only in tests; it is never configured as a user-facing source of truth.

不包含 provider HTTP client、API key 访问、model invocation、public REST/OpenAPI/public SDK/Web mutation、local API transport、浏览器控件、dataset/suite editing、scheduler、queue、并发执行声明、dashboard 扩张、GraphDiff 改动、第二个 graph-diff calculator、Docker、PostgreSQL runtime、release 或 production 声明。确定性的 fixture evaluator 仅存在于测试中，绝不作为面向用户的事实来源。

**Minimal affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Add the pure execution plan in `crates/evaluation`; add one private orchestration service and focused tests in `crates/storage`; re-export the contracts; document the provider-free boundary, no caller-supplied measurements rule, existing-decision behavior, and deferred PostgreSQL runtime in English and Chinese. Public API catalogs, both SDKs, Web code, `model-gateway`, database schema, and `diff-engine` remain unchanged.

在 `crates/evaluation` 中新增纯 execution plan；在 `crates/storage` 中新增一个私有 orchestration service 与聚焦测试；重新导出这些 contract；用中英双语记录无 Provider 的边界、禁止调用方提供 measurement、existing-decision 行为与延期的 PostgreSQL runtime。public API catalog、两个 SDK、Web code、`model-gateway`、database schema 与 `diff-engine` 均保持不变。

**Blocked condition, minimal fix, and regression proof / 阻塞条件、最小修复与回归证明：** The initial plan produced random `EvaluationRunId` values and discarded `(dataset_id, case_id)` after assembly; it also accepted duplicate metric kinds until scorecard construction. This blocked trustworthy replay because a sealed decision could not prove which immutable case produced a stored run. The minimal fix is a private `BenchmarkExecutionCaseKey` and `BenchmarkExecutionCohort`: derive every run ID with UUIDv5 from the decision UUID plus that composite key, retain the key beside the run during domain assembly, and reject duplicate metrics before any run exists. Regression proof must show identical decision/key inputs reproduce run IDs, a different decision changes them, duplicate metric results fail before cohort creation, and persisted evidence retains precisely those deterministic run IDs.

初始计划会生成随机 `EvaluationRunId`，并在组装后丢失 `(dataset_id, case_id)`；同时它会在 scorecard 构造前接受重复 metric kind。这会阻塞可信 replay，因为 sealed decision 无法证明哪条不可变 case 产生了已存储的 run。最小修复是私有的 `BenchmarkExecutionCaseKey` 与 `BenchmarkExecutionCohort`：用 decision UUID 与该复合 key 经 UUIDv5 派生每个 run ID，在 domain assembly 中保留 key 与 run 的对应关系，并在任何 run 产生前拒绝重复 metric。回归证明必须显示相同 decision/key 输入会重现 run ID、不同 decision 会改变它、重复 metric result 会在 cohort 创建前失败，并且 persisted evidence 恰好保留这些确定性 run ID。

**Fresh verification before another increment / 下一增量前的新鲜验证：** Observe red then green tests for exact case ordering, deterministic case-to-run provenance, duplicate metric rejection, unknown/missing/duplicate results, read-before-execute replay avoidance, evaluator error propagation without persistence, one domain evaluation and one writer call, immutable evidence inspection compatibility, `cargo fmt --all -- --check`, focused Clippy for changed crates, `cargo test --workspace`, and the explicit compiled/ignored PostgreSQL status. The known unrelated auth MSRV Clippy blocker must remain accurately recorded rather than masked.

下一增量前必须观察到：精确 case 排序、unknown/missing/duplicate result、read-before-execute replay avoidance、evaluator error 不持久化、只调用一次 domain evaluation 与 writer、不可变 evidence inspection compatibility 的红绿测试；以及 `cargo fmt --all -- --check`、已修改 crate 的聚焦 Clippy、`cargo test --workspace` 与明确的 compiled/ignored PostgreSQL 状态。已知的无关 auth MSRV Clippy blocker 必须继续如实记录，不能掩盖。

## File Map / 文件映射

- Create: `crates/evaluation/src/benchmark_execution.rs`
- Modify: `Cargo.toml`, `crates/evaluation/src/lib.rs`
- Test: `crates/evaluation/tests/benchmark_execution.rs`
- Create: `crates/storage/src/benchmark_execution.rs`
- Modify: `crates/storage/src/lib.rs`
- Test: `crates/storage/tests/benchmark_execution.rs`
- Modify: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and this plan

## Task 1: Define a Pure Sealed Case Plan / 定义纯粹的已 Seal Case Plan

- [x] **Step 1: Write the failing domain test.** Add a `BenchmarkExecutionPlan::new` test using two datasets supplied out of order. Assert that the plan emits a stable `BenchmarkExecutionCaseKey` sequence, UUIDv5-derived run identities are stable for the same decision UUID and different for a different decision UUID, incomplete dataset membership is rejected, duplicate metric kinds are rejected before cohort construction, and duplicated/unknown/missing result keys fail.

```rust
let cohort = plan.assemble_cohort(
    decision_id.as_uuid(), context_id, "model-a", 0.2, now, results,
).expect("cohort");
assert_eq!(cohort.entries()[0].key().dataset_id(), expected_dataset_id);
assert_eq!(cohort.entries()[0].run().id(), repeated.entries()[0].run().id());
```

- [x] **Step 2: Run the test to verify red.**

Run: `cargo test -p contextlab-evaluation --test benchmark_execution`

Expected: FAIL because the execution-plan types do not exist.

- [x] **Step 3: Implement the minimal domain contract.** Add `BenchmarkExecutionCaseKey`, `BenchmarkExecutionPlan`, `BenchmarkExecutionCase`, `BenchmarkCaseExecutionResult`, `BenchmarkExecutionCohort`, `BenchmarkExecutedCase`, and `BenchmarkExecutionError`. Validate exact suite membership, reject duplicate dataset IDs and duplicate metric kinds, sort datasets and cases by stable IDs, and turn exactly one finite measurement vector per planned case into a UUIDv5-stable `EvaluationRun` retained beside its composite key. The plan does not call an executor, read a clock, calculate a regression decision, depend on storage, or serialize raw cases.

```rust
pub fn assemble_cohort(
    &self,
    decision_namespace: Uuid,
    context_id: ContextId,
    model_version: &str,
    temperature: f32,
    executed_at: DateTime<Utc>,
    results: Vec<BenchmarkCaseExecutionResult>,
) -> Result<BenchmarkExecutionCohort, BenchmarkExecutionError>;
```

- [x] **Step 4: Run the focused domain test to verify green.**

Run: `cargo test -p contextlab-evaluation --test benchmark_execution`

Expected: PASS with deterministic ordering and invalid-result cases covered.

## Task 2: Orchestrate an Injected Local Evaluator / 编排注入的本地 Evaluator

- [x] **Step 1: Write the failing storage service test.** Create a recording evaluator that returns one declared measurement set per case and a recording repository/writer. Assert that execution loads the suite and all named datasets from storage, passes the exact Context commit and immutable plan case to the evaluator, evaluates the cohort with `suite.evaluate_runs`, and writes `PersistBenchmarkEvaluationEvidence` once.

```rust
let result = service.execute(request).await.expect("execution");
assert_eq!(result.disposition(), BenchmarkExecutionDisposition::Created);
assert_eq!(evaluator.calls(), plan.case_count());
assert_eq!(writer.calls(), 1);
```

- [x] **Step 2: Run the test to verify red.**

Run: `cargo test -p contextlab-storage --test benchmark_execution`

Expected: FAIL because `BenchmarkExecutionService` and the evaluator port do not exist.

- [x] **Step 3: Implement the private service.** Add `BenchmarkCaseEvaluator` as an async storage application port and `BenchmarkExecutionService<R, E>`, where `R` implements the existing `BenchmarkEvidenceRepository + BenchmarkEvidenceWriter` contracts. `BenchmarkExecutionRequest` holds only typed project/Context/commit/suite/decision/model/evaluator identities and timestamps; it never accepts datasets, cases, measurements, scorecards, or a decision status. It validates model/temperature and evaluator identity before evaluator invocation. First read the exact decision identity; if it exists, return a replayed outcome without evaluator invocation. Otherwise load the suite and datasets, build the pure plan, invoke the evaluator once per stable case, create a deterministic provenance cohort with the decision UUID, call `suite.evaluate_runs` exactly once over the cohort runs, build `PersistBenchmarkEvaluationEvidence`, and call the writer exactly once.

```rust
pub async fn execute(
    &self,
    request: BenchmarkExecutionRequest,
) -> Result<BenchmarkExecutionResult, StorageRepositoryError>;
```

- [x] **Step 4: Add failure and replay proofs.** Extend the focused test so missing suite/dataset, evaluator error, and malformed case result return an error before persistence; an existing exact decision returns `Replayed` without calling the evaluator; and a new decision can be loaded by the existing `get_benchmark_decision` repository method.

- [x] **Step 5: Run focused storage tests to verify green.**

Run: `cargo test -p contextlab-storage --test benchmark_execution`

Expected: PASS with created, replayed, read-failure, evaluator-failure, and persistence-boundary assertions green.

## Task 3: Export, Document, and Verify / 导出、文档与验证

- [x] **Step 1: Re-export only application contracts.** Export evaluation plan types and storage service/port from crate roots. Do not add an Axum route, OpenAPI entry, TypeScript SDK method, Web control, model-gateway client, or migration.

- [x] **Step 2: Write bilingual boundary documentation.** Update architecture, persistence foundation, active goal, and completion criteria to state that this is a private, injected-evaluator orchestration layer; provider transport and local UI invocation remain later decisions; existing protected decision inspection/diff is its only review surface; and PostgreSQL execution runtime remains unobserved without an explicitly supplied disposable database.

- [x] **Step 3: Run fresh scope-matched validation.**

Run: `cargo fmt --all -- --check`

Expected: PASS.

Run: `cargo clippy -p contextlab-evaluation -p contextlab-storage --all-targets -- -D warnings`

Expected: PASS for changed crates, or an exact recorded blocker unrelated to the change.

Run: `cargo test -p contextlab-evaluation --test benchmark_execution`

Expected: PASS.

Run: `cargo test -p contextlab-storage --test benchmark_execution`

Expected: PASS.

Run: `cargo test --workspace`

Expected: PASS; PostgreSQL runtime tests remain compiled/ignored unless a disposable database is explicitly available.

- [ ] **Step 4: Record observed evidence.** Mark only observed red/green steps complete, record command counts verbatim, retain the active long-term goal, and use the completion criteria to select the next dependency-ready increment.

## Fresh Verification Record / 新鲜验证记录

Observed locally on 2026-07-18, with Docker disabled and without database provisioning or secret access:

- `cargo test -p contextlab-evaluation --test benchmark_execution`: PASS (`4 passed`). The initial missing-symbol test state and the later cohort/provenance API red state were both observed before their implementations.
- `cargo test -p contextlab-storage --test benchmark_execution`: PASS (`5 passed`). It covers exact evaluator scope, UUIDv5 run provenance, created/replayed disposition, one write versus replay zero-write, missing suite/dataset, invalid request, evaluator error, unknown case result, and redacted adapter diagnostics.
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --quiet`: PASS. API `131 passed`; storage `159 passed, 32 ignored`; evaluation execution `4 passed`; storage execution `5 passed`.
- `cargo clippy -p contextlab-storage --all-targets -- -D warnings`: remains BLOCKED before this slice is linted by the existing Rust 1.85 MSRV error in `contextlab-auth/src/authorization.rs:320`. A supplemental run with only that lint allowed reached 12 pre-existing storage warnings outside the execution files, so it is not represented as a passing storage quality gate.
- PostgreSQL benchmark-execution runtime evidence is unobserved: no disposable database URL was supplied and Docker remains disabled. No local result is presented as PostgreSQL, remote CI, release, or production evidence.

2026-07-18 在 Docker 保持关闭、未配置数据库且未访问密钥的条件下已观察到：

- `cargo test -p contextlab-evaluation --test benchmark_execution`：通过（`4 passed`）。实现前已观察到初始 missing-symbol 红测以及后续 cohort/provenance API 红测。
- `cargo test -p contextlab-storage --test benchmark_execution`：通过（`5 passed`）。覆盖精确 evaluator scope、UUIDv5 run provenance、created/replayed disposition、一次写入与 replay 零写入、缺失 suite/dataset、非法 request、evaluator error、unknown case result 和脱敏 adapter diagnostics。
- `cargo fmt --all -- --check`：通过。
- `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings`：通过。
- `cargo test --workspace --quiet`：通过。API `131 passed`；storage `159 passed, 32 ignored`；evaluation execution `4 passed`；storage execution `5 passed`。
- `cargo clippy -p contextlab-storage --all-targets -- -D warnings`：在 lint 到本切片前，被 `contextlab-auth/src/authorization.rs:320` 中既有的 Rust 1.85 MSRV error 阻塞。仅放行该 lint 的补充运行到达了 execution 文件之外的 12 条既有 storage warning，因此不把它表示为通过的 storage quality gate。
- PostgreSQL benchmark-execution runtime evidence 仍未观测：未提供 disposable database URL，且 Docker 保持关闭。没有任何本地结果被表述为 PostgreSQL、remote CI、release 或 production 证据。
