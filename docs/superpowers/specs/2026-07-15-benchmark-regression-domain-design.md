# Benchmark Regression Domain Design / Benchmark 回归判定领域设计

**Status / 状态:** Approved for autonomous implementation under the active long-term goal / 已按当前长期目标准入自主实施

## Goal / 目标

Add a framework-independent evaluation contract for reusable immutable benchmark datasets, suites, metric thresholds, and deterministic regression decisions. This is the smallest domain foundation needed before ContextLab can persist or present benchmark-driven evaluation without inventing policy in storage, API, SDK, or Web code.

为 ContextLab 增加与框架无关的评测契约，覆盖可复用的不可变 benchmark dataset、suite、metric threshold 与确定性 regression decision。这是在持久化或呈现 benchmark-driven evaluation 前所需的最小领域基础，避免在 storage、API、SDK 或 Web 中另行发明策略。

## Domain Boundary / 领域边界

- A `BenchmarkCase` has a stable identifier, a validated name, a structured JSON input, and an explicit `Unspecified` or `Exact(Value)` oracle so exact JSON null is not confused with no oracle.
- A `BenchmarkDataset` has a stable identifier, a validated name, at least one case, and no duplicate case identifiers.
- A `BenchmarkSuite` has a stable identifier, a validated name, at least one dataset membership, no duplicate dataset identifiers, and at least one unique metric threshold.
- A `RegressionThreshold` names one `MetricKind` and an explicit inclusive `Minimum` or `Maximum` finite boundary.
- A `RegressionDecision` evaluates a `Scorecard` in stable metric order. Any completely observed threshold breach produces `Regressed`; otherwise any missing, partial, duplicated, or invalid required metric produces `InsufficientData`; all satisfied thresholds produce `Passed`.

- `BenchmarkCase` 具有稳定标识、经校验的名称、结构化 JSON input，并使用显式 `Unspecified` 或 `Exact(Value)` oracle，避免把精确 JSON null 与没有 oracle 混淆。
- `BenchmarkDataset` 具有稳定标识、经校验的名称、至少一个 case，且不得出现重复 case identifier。
- `BenchmarkSuite` 具有稳定标识、经校验的名称、至少一个 dataset membership，不得出现重复 dataset identifier，并包含至少一个 metric 唯一的 threshold。
- `RegressionThreshold` 指向一个 `MetricKind`，并使用显式、包含边界值且有限的 `Minimum` 或 `Maximum` 边界。
- `RegressionDecision` 按稳定 metric 顺序评估 `Scorecard`。任一具有完整证据的 threshold 越界时返回 `Regressed`；否则，任一必需 metric 缺失、部分覆盖、重复或无效时返回 `InsufficientData`；全部满足时返回 `Passed`。

## Determinism and Validation / 确定性与校验

Constructors reject empty names through the shared `NonEmptyString` primitive, empty datasets or suites, duplicate memberships, duplicate metric thresholds, and non-finite threshold values. Collections are normalized by stable identifiers or `MetricKind`; scorecard samples are normalized before aggregation. Duplicate metrics inside one run invalidate that metric's coverage, and validated aggregate objects are serialization-only rather than constructor-bypassing deserialization targets. Threshold equality passes because regression boundaries are inclusive.

Constructor 通过共享 `NonEmptyString` primitive 拒绝空名称，并拒绝空 dataset 或 suite、重复 membership、重复 metric threshold 与非有限阈值。集合按稳定 identifier 或 `MetricKind` 规范化，scorecard sample 也会在聚合前规范化。单个 run 内的重复 metric 会使该 metric coverage 无效；经过校验的 aggregate object 只支持序列化，不提供绕过 constructor 的直接反序列化。阈值等值视为通过，因为 regression boundary 是包含式边界。

## Data Flow / 数据流

The existing `EvaluationRun -> Scorecard` boundary remains the sole aggregation path, but it now preserves per-metric coverage evidence, rejects duplicate-run evidence for decision purposes, normalizes floating input order, and avoids large-finite summation overflow. Callers construct or rehydrate one validated suite, build a scorecard from matching runs, and ask the suite for a decision. The returned decision contains one check per threshold, including the observed value when present, so later persistence and read-only presentation can transport the result without recalculating it.

既有 `EvaluationRun -> Scorecard` 边界继续是唯一聚合路径，但现会保留逐 metric coverage 证据、在 decision 中拒绝重复 run evidence、规范化浮点输入顺序并避免大有限值求和溢出。调用方构造或恢复一个经过校验的 suite，从匹配 run 构建 scorecard，再由 suite 生成 decision。返回结果为每个 threshold 保留一项 check，并在 metric 存在时包含 observed value，使后续 persistence 与只读呈现能够直接传输结果，而不重新计算。

## Explicit Non-Goals / 明确非目标

This increment does not add benchmark execution, provider calls, evaluator plugins, dataset mutation, storage migrations, repository ports, REST/OpenAPI/SDK methods, Web UI, A/B experiments, baseline-relative deltas, semantic/behavior/evaluation diff engines, release evidence, or production claims. It does not change `GraphDiff`.

本增量不新增 benchmark execution、provider 调用、evaluator plugin、dataset mutation、storage migration、repository port、REST/OpenAPI/SDK method、Web UI、A/B experiment、相对 baseline delta、semantic/behavior/evaluation diff engine、release evidence 或 production 声明，也不修改 `GraphDiff`。

## Verification / 验证

Focused tests must first fail because the benchmark types are absent, then pass after implementation. Tests cover immutable identity/data access, duplicate and empty collection rejection, non-finite thresholds, inclusive minimum/maximum boundaries, stable ordering, pass, regression, and missing-metric decisions. The fresh gate is `cargo fmt --all -- --check`, `cargo test -p contextlab-evaluation`, and `cargo test --workspace --quiet`; external and PostgreSQL evidence are not required for this pure domain increment.

聚焦测试必须先因 benchmark 类型缺失而失败，再在实现后通过。测试覆盖不可变 identity/data access、重复与空集合拒绝、非有限阈值、包含式 minimum/maximum 边界、稳定排序、通过、回归与缺失 metric 判定。新鲜门禁为 `cargo fmt --all -- --check`、`cargo test -p contextlab-evaluation` 与 `cargo test --workspace --quiet`；本次纯领域增量不需要外部或 PostgreSQL 证据。
