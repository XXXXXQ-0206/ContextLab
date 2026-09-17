# Benchmark Workspace Projection Plan / Benchmark Workspace 投影计划

**Goal / 目标：** Admit one provider-free, deterministic, redacted V1 application DTO that composes an existing sealed benchmark receipt, its ordered run provenance, scorecard evidence, regression decision, and the existing benchmark evaluation diff.

**Architecture / 架构：** `contextlab-evaluation` remains the only affected code boundary. `BenchmarkWorkspaceProjectionV1` projects already-calculated receipt facts and, when a baseline receipt is supplied, delegates comparison exclusively to `BenchmarkDecisionDiff::between`. The projection validates exact suite scope, preserves UUID identities, orders datasets, runs, and metric changes by existing domain order, and serializes no cases, inputs, expected outputs, provider outputs, model configuration, or measurements.

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This increment directly advances completion criterion 3, which requires reusable benchmark suites, run details, numeric scorecards, regression decisions, and evaluation diff views. It also follows the charter requirement that reusable domain/application contracts live in Rust rather than UI components.

本增量直接推进收束条件 3，其中要求可复用的 benchmark suite、run detail、数值 scorecard、regression decision 与 evaluation diff view。它同时遵循项目宪章：可复用的 domain/application contract 必须位于 Rust 层，而不是 UI component 中。

**Unmet gap and risk / 未满足缺口与风险：** The existing sealed execution receipt already composes deterministic runs, a scorecard, a regression decision, and comparison input, while the current workspace projection exposes only the single-receipt facts. There is no single safe DTO that carries the existing evaluation diff. A future consumer would otherwise have to compose or recalculate comparison behavior outside the evaluation crate.

既有 sealed execution receipt 已组合确定性 run、scorecard、regression decision 与 comparison input，但当前 workspace projection 只暴露单条 receipt 的事实。当前没有一个安全 DTO 能携带既有 evaluation diff；未来 consumer 因此可能在 evaluation crate 之外自行拼装或重新计算 comparison behavior。

**Reviewed root cause / 复核根因：** `BenchmarkWorkspaceProjectionV1` authenticated the caller-supplied plan only by comparing its suite UUID with the receipt suite UUID, then copied the suite name, dataset names, dataset membership, and case counts from that plan. `BenchmarkSuite::with_id` legitimately rehydrates any validated definition under a supplied UUID, so UUID equality did not prove definition equality and allowed sealed receipt evidence to be combined with unrelated caller metadata. The repair seals redacted, immutable workspace-definition facts in the receipt, compares those facts exactly before projection or diff composition, and projects metadata from the receipt-owned facts.

`BenchmarkWorkspaceProjectionV1` 过去只比较 caller-supplied plan 与 receipt 的 suite UUID，随后直接从该 plan 复制 suite name、dataset name、dataset membership 与 case count。`BenchmarkSuite::with_id` 可在给定 UUID 下合法重建任意通过校验的 definition，因此 UUID 相等并不能证明 definition 相等，sealed receipt evidence 可能与无关 caller metadata 被拼接。修复方案是在 receipt 中封存经过脱敏的 immutable workspace-definition facts，在 projection 或 diff composition 前执行精确比较，并只从 receipt-owned facts 投影 metadata。

**Why this is next / 为什么现在优先：** All required inputs and the sole evaluation-diff calculator already exist inside `contextlab-evaluation`, so the increment has no external dependency. It closes a direct Criterion 3 composition gap with a smaller boundary than persistence, transport, provider, or UI work.

所需输入与唯一的 evaluation-diff calculator 已全部存在于 `contextlab-evaluation`，因此本增量没有外部依赖。它以小于 persistence、transport、provider 或 UI 工作的边界，直接收束条件 3 的 composition 缺口。

**Explicit non-goals / 明确非目标：** No raw cases, case inputs, expected outputs, provider or model outputs, measurement vectors, provider calls, policy or scorecard recomputation, persistence, public REST/OpenAPI/public SDK writes, Web changes, Docker, PostgreSQL, secret access, environment-file reads, release claim, or production claim. `GraphDiff::between` is untouched and remains the sole graph-diff calculator.

不包含 raw case、case input、expected output、provider 或 model output、measurement vector、provider call、policy 或 scorecard 重算、persistence、public REST/OpenAPI/public SDK write、Web 改动、Docker、PostgreSQL、secret 访问、environment file 读取、release 声明或 production 声明。`GraphDiff::between` 保持不变，仍是唯一的 graph-diff calculator。

**Minimal boundary and bilingual documentation / 最小边界与双语文档：** Only `crates/evaluation/**` and this bilingual plan may change. The V1 DTO uses existing typed UUIDs, an explicit numeric schema version, deterministic domain ordering, safe optional baseline/revised metric summaries, and structured fail-closed scope/comparability errors.

只允许修改 `crates/evaluation/**` 与本双语计划。V1 DTO 使用既有 typed UUID、显式数值 schema version、确定性的 domain ordering、安全的可选 baseline/revised metric summary，以及结构化的 scope/comparability fail-closed error。

**Fresh evidence required / 所需新鲜证据：** Observe one focused RED test before implementation; then run the focused projection test, all `contextlab-evaluation` tests, `cargo fmt --all -- --check`, and `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings`. Record Docker/PostgreSQL, public transport, browser, remote CI, release, and production evidence as unobserved rather than inferred.

实现前必须观察到一个聚焦 RED test；随后运行聚焦 projection test、全部 `contextlab-evaluation` tests、`cargo fmt --all -- --check` 与 `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings`。Docker/PostgreSQL、public transport、browser、remote CI、release 与 production evidence 必须记为 unobserved，不能推断为已验证。

## TDD Steps / TDD 步骤

- [x] Add one focused test for safe evaluation-diff composition, deterministic metric ordering, and redaction.
- [x] Run the focused test and observe the expected missing-contract failure.
- [x] Minimally extend `BenchmarkWorkspaceProjectionV1` to delegate to `BenchmarkDecisionDiff::between` and project only safe evidence.
- [x] Add focused fail-closed comparability coverage and retain existing scope/ordering/redaction coverage.
- [x] Run focused tests, all crate tests, formatting, and strict crate Clippy.
- [x] Add a same-suite-ID definition-drift regression and observe the unrelated plan metadata being admitted.
- [x] Seal redacted plan-definition facts into receipts and fail closed on exact definition mismatch.

## Evidence Record / 证据记录

Observed locally on 2026-07-23 without Docker, PostgreSQL provisioning, provider calls, secret access, or environment-file reads:

- RED: `cargo test -p contextlab-evaluation --test benchmark_workspace_projection` failed as expected because `BenchmarkEvaluationMetricChangeKindV1` and `BenchmarkWorkspaceProjectionV1::from_receipts` did not exist.
- GREEN: the same focused command passed with `4 passed` after the minimal diff projection was added.
- RED: the focused command then failed with `4 passed; 1 failed` because a revised receipt with the wrong suite reached comparison before scope validation.
- GREEN: after validating both receipt suites before comparison, the focused command passed with `5 passed`.
- `cargo test -p contextlab-evaluation`: passed with `42 passed`.
- `cargo fmt -p contextlab-evaluation -- --check`: passed.
- `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check` was run but remains blocked by unrelated formatting drift in CLI, diff-engine, knowledge, and workflow files outside this increment's write ownership. No outside-owned file was formatted or changed.
- Docker/PostgreSQL runtime, public transport, browser E2E, remote CI, release, and production evidence remain unobserved. This local DTO increment does not close Criterion 3.

2026-07-23 在未使用 Docker、未配置 PostgreSQL、未调用 provider、未访问 secret 或 environment file 的条件下，本地观察到：

- RED：`cargo test -p contextlab-evaluation --test benchmark_workspace_projection` 按预期失败，因为 `BenchmarkEvaluationMetricChangeKindV1` 与 `BenchmarkWorkspaceProjectionV1::from_receipts` 尚不存在。
- GREEN：加入最小 diff projection 后，同一聚焦命令以 `4 passed` 通过。
- RED：聚焦命令随后以 `4 passed; 1 failed` 失败，因为错误 suite 的 revised receipt 在 scope validation 前进入 comparison。
- GREEN：在 comparison 前校验两条 receipt 的 suite 后，聚焦命令以 `5 passed` 通过。
- `cargo test -p contextlab-evaluation`：以 `42 passed` 通过。
- `cargo fmt -p contextlab-evaluation -- --check`：通过。
- `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings`：通过。
- 已运行 `cargo fmt --all -- --check`，但它仍被 CLI、diff-engine、knowledge 与 workflow 中不属于本增量 write ownership 的 formatting drift 阻塞；没有格式化或修改任何外部 ownership 的文件。
- Docker/PostgreSQL runtime、public transport、browser E2E、remote CI、release 与 production evidence 仍未观测。本地 DTO 增量不关闭条件 3。

### Reviewed projection-integrity repair / 已复核的投影完整性修复

Observed locally on 2026-07-23 after the reviewed same-suite-ID definition-drift finding:

- RED: `cargo test -p contextlab-evaluation --test benchmark_workspace_projection projection_fails_closed_when_same_suite_id_has_definition_drift -- --exact --nocapture` failed with `0 passed; 1 failed`. The panic showed that a receipt sealed for `Release gate` produced a successful projection named `Unrelated release gate` with `Unrelated first` and `Unrelated second` datasets.
- GREEN: the same focused command passed with `1 passed` after receipts sealed redacted suite name, thresholds, dataset identity/name, and case identity facts and the projection compared those facts exactly.
- Focused projection contract: `cargo test -p contextlab-evaluation --test benchmark_workspace_projection` passed with `6 passed`.
- Full evaluation crate: `cargo test -p contextlab-evaluation` passed with 43 tests across unit and integration targets; doc-tests contained zero tests and passed.
- Crate formatting: `cargo fmt -p contextlab-evaluation -- --check` passed.
- Strict crate Clippy: `cargo clippy -p contextlab-evaluation --all-targets -- -D warnings` passed.
- Rust 1.85 MSRV: installed toolchain `1.85.0-x86_64-pc-windows-msvc`; `cargo +1.85.0 check -p contextlab-evaluation --all-targets` passed.
- Repository formatting: `cargo fmt --all -- --check` remains blocked by out-of-scope formatting drift in `crates/storage/tests/benchmark_workspace_projection.rs` and `crates/workflow/tests/workflow_deterministic_replay.rs`. Neither outside-owned file was formatted or changed.
- No policy or scorecard calculation was added, no raw case content was exposed, and `GraphDiff` was not changed.

2026-07-23，在复核 same-suite-ID definition-drift finding 后，本地观察到：

- RED：`cargo test -p contextlab-evaluation --test benchmark_workspace_projection projection_fails_closed_when_same_suite_id_has_definition_drift -- --exact --nocapture` 以 `0 passed; 1 failed` 失败。panic 显示，为 `Release gate` 封存的 receipt 成功生成了名为 `Unrelated release gate` 的 projection，并携带 `Unrelated first` 与 `Unrelated second` dataset。
- GREEN：receipt 封存脱敏的 suite name、threshold、dataset identity/name 与 case identity facts，且 projection 对这些 facts 做精确比较后，同一聚焦命令以 `1 passed` 通过。
- 聚焦 projection contract：`cargo test -p contextlab-evaluation --test benchmark_workspace_projection` 以 `6 passed` 通过。
- 完整 evaluation crate：`cargo test -p contextlab-evaluation` 的 unit 与 integration target 共 43 个测试全部通过；doc-test 为 0 个并通过。
- Crate formatting：`cargo fmt -p contextlab-evaluation -- --check` 通过。
- 严格 crate Clippy：`cargo clippy -p contextlab-evaluation --all-targets -- -D warnings` 通过。
- Rust 1.85 MSRV：已安装 `1.85.0-x86_64-pc-windows-msvc`；`cargo +1.85.0 check -p contextlab-evaluation --all-targets` 通过。
- Repository formatting：`cargo fmt --all -- --check` 仍被 `crates/storage/tests/benchmark_workspace_projection.rs` 与 `crates/workflow/tests/workflow_deterministic_replay.rs` 中的 scope 外 formatting drift 阻塞；未格式化或修改任何外部 ownership 文件。
- 未新增 policy 或 scorecard calculation，未暴露 raw case content，且未修改 `GraphDiff`。
