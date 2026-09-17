# Benchmark Regression Domain Implementation Plan / Benchmark 回归判定领域实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标：** Add the smallest reusable Rust domain contract for immutable benchmark datasets, suites, thresholds, and deterministic regression decisions.

**Architecture / 架构：** `contextlab-evaluation` remains framework-independent and owns all benchmark validation and decision policy. Benchmark records use stable typed identifiers and structured JSON case payloads; regression decisions consume the existing `Scorecard`, normalize threshold order, and expose explicit per-metric checks. Storage and presentation will consume this contract in a later separately admitted increment.

**Tech Stack / 技术栈：** Rust 2024, Serde, Serde JSON, UUID, existing ContextLab evaluation and context-core primitives.

---

## Necessity Record / 必要性记录

**Completion criteria and charter principle / 完成条件与宪章原则：** This increment directly serves completion criterion 3, `Benchmark-driven evaluation`, and criterion 1's dataset domain coverage. It reinforces the charter requirement that evaluation datasets be reusable and that regression detection live in reusable business logic rather than framework or page code.

本增量直接服务完成条件 3“Benchmark-driven evaluation”与完成条件 1 中的 dataset domain coverage，并落实宪章中 evaluation dataset 可复用、regression detection 必须属于可复用业务逻辑而非 framework 或页面代码的要求。

**Gap and risk / 缺口与风险：** The repository currently models evaluation runs, finite metric measurements, average scorecards, persisted run reads, and Web scorecard inspection. It has no benchmark dataset or suite identity, no persisted-policy-ready threshold direction, and no deterministic decision for missing or breached metrics. Extending storage or Web first would duplicate undefined policy across layers.

当前仓库已有 evaluation run、有限 metric measurement、平均 scorecard、持久化 run read 与 Web scorecard inspection，但没有 benchmark dataset 或 suite identity、适合后续持久化的 threshold direction，也没有针对缺失或越界 metric 的确定性 decision。若先扩展 storage 或 Web，会在多层重复尚未定义的 policy。

**Why now / 为什么现在优先：** The local Context lifecycle vertical slice is implemented and locally verified apart from explicitly deferred PostgreSQL lifecycle E2E and authenticated protected-browser smoke. Completion criterion 3 is the next dependency-ready core gap, and its framework-independent policy is the prerequisite for honest persistence, API, SDK, and dashboard work.

本地 Context lifecycle 垂直切片已经实现并完成本地验证，仅 lifecycle PostgreSQL E2E 与 authenticated protected-browser smoke 明确延期。完成条件 3 是下一个依赖就绪的核心缺口，其框架无关 policy 是后续诚实推进 persistence、API、SDK 与 dashboard 的前置。

**Explicit non-goals / 明确非目标：** No benchmark runner, provider call, evaluator plugin, public or private write API, migration, repository, OpenAPI/SDK method, Web control, A/B experiment, baseline delta, semantic/behavior/evaluation diff, Docker, remote CI, release, or production work. `GraphDiff` remains unchanged and the sole graph-diff calculator.

不包含 benchmark runner、provider call、evaluator plugin、public 或 private write API、migration、repository、OpenAPI/SDK method、Web control、A/B experiment、baseline delta、semantic/behavior/evaluation diff、Docker、remote CI、release 或 production 工作。`GraphDiff` 保持不变且继续是唯一 graph-diff calculator。

**Minimal boundary and bilingual documentation / 最小边界与双语文档：** Add focused evaluation tests, split benchmark/regression types into small modules under `crates/evaluation`, export them from the crate root, add only the required `serde_json` dependency, and update this design/plan plus roadmap evidence in English and Chinese.

增加聚焦 evaluation 测试，在 `crates/evaluation` 下用小模块承载 benchmark/regression 类型并从 crate root 导出，只增加必需的 `serde_json` dependency，同时用中英双语更新本设计、计划与路线图证据。

**Fresh verification before the next increment / 下一增量前的新鲜验证：** Observe the integration test fail because benchmark types do not exist. Then observe focused evaluation tests, formatting, and the complete Rust workspace pass. Search the change boundary to confirm no route, SDK, Web mutation, storage migration, or `GraphDiff` implementation was added. PostgreSQL and external evidence remain outside this pure-domain slice.

先观察 integration test 因 benchmark 类型不存在而失败，再观察聚焦 evaluation test、格式检查与完整 Rust workspace 通过。通过边界搜索确认没有新增 route、SDK、Web mutation、storage migration 或 `GraphDiff` 实现。PostgreSQL 与外部证据不属于本次纯领域切片。

## File Map / 文件映射

- Create: `crates/evaluation/tests/benchmark_regression.rs`
- Create: `crates/evaluation/src/benchmark.rs`
- Create: `crates/evaluation/src/regression.rs`
- Modify: `crates/evaluation/src/lib.rs`
- Modify: `crates/evaluation/Cargo.toml`
- Modify after verification: `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and this plan

## Task 1: Red Domain Contract / 领域契约红测

- [x] Add an integration test that imports the intended benchmark and regression public API.
- [x] Cover validated immutable records, empty and duplicate rejection, inclusive boundaries, stable ordering, pass/regressed/insufficient-data outcomes, and non-finite threshold rejection.
- [x] Run `cargo test -p contextlab-evaluation --test benchmark_regression` and record the expected unresolved-import failure.

## Task 2: Minimal Benchmark Model / 最小 Benchmark 模型

- [x] Add typed case, dataset, and suite identifiers with `new`, `from_uuid`, and `as_uuid` operations.
- [x] Implement validated immutable case and dataset records with deterministic case ordering and explicit oracle semantics.
- [x] Implement validated suite membership and unique threshold normalization.
- [x] Export the public model from `contextlab-evaluation` and add `serde_json` only for structured case payloads.

## Task 3: Deterministic Regression Decision / 确定性回归判定

- [x] Implement finite inclusive minimum/maximum thresholds.
- [x] Implement stable per-metric checks and overall `Passed`, `Regressed`, or `InsufficientData` status, including partial coverage, duplicate-metric pollution, stable aggregation, and large finite values.
- [x] Run the focused integration test and crate tests until green.

## Task 4: Evidence and Continuation / 证据与继续推进

- [x] Run `cargo fmt --all -- --check`, `cargo test -p contextlab-evaluation`, and `cargo test --workspace --quiet`.
- [x] Run boundary searches proving this slice added no route, SDK/Web mutation, storage migration, or second graph-diff calculator.
- [x] Record exact observed results in this plan and the bilingual roadmap without closing the active goal.
- [x] Admit the next dependency-ready criterion-3 increment only after reviewing persistence and read-only inspection boundaries.

## Observed Evidence / 已观察证据

The first focused run failed with unresolved imports for all benchmark/regression types and the missing `serde_json` dependency. Later red tests exposed partial metric coverage, run-order-dependent floating aggregation, large finite aggregation overflow, duplicate metrics masking missing runs, and ambiguous JSON-null oracle semantics. Review-driven coverage then added empty-name/threshold, typed-ID, membership-order, infinity, conflicting duplicate-threshold, three-value extreme, subnormal, and empty-scorecard cases. The final focused crate result is `21 passed` (`2` unit + `19` integration). `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings` passed. The fresh workspace result is API `121 passed`, auth `43 passed`, evaluation `21 passed`, storage `152 passed, 25 ignored`, and all remaining Rust crates green. Boundary search found no benchmark/regression types in storage, server, SDK, or Web, and production still contains exactly two `GraphDiff::between` call sites.

首次聚焦运行因全部 benchmark/regression 类型尚未导出且缺少 `serde_json` dependency 而失败。后续红测依次暴露 partial metric coverage、受 run 顺序影响的浮点聚合、大有限值聚合溢出、重复 metric 掩盖缺失 run，以及 JSON null oracle 语义含糊。审查驱动的补充覆盖还加入空名称/threshold、typed ID、membership 顺序、无穷值、冲突重复 threshold、三项极值、次正规数与空 scorecard case。最终聚焦 crate 结果为 `21 passed`（`2` unit + `19` integration）。`cargo clippy -p contextlab-evaluation --all-targets -- -D warnings` 已通过。新鲜 workspace 结果为 API `121 passed`、auth `43 passed`、evaluation `21 passed`、storage `152 passed, 25 ignored`，其余 Rust crate 全部通过。边界搜索未在 storage、server、SDK 或 Web 中发现 benchmark/regression 类型，production 仍只有两个 `GraphDiff::between` call site。
