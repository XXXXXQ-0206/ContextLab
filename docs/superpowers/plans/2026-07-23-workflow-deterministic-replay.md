# Workflow Deterministic Execution and Replay Implementation Plan / Workflow 确定性执行与回放实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:test-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal / 目标：** Add one provider-free Workflow core increment that executes the existing typed graph through its existing scheduler and reconstructs the exact run from a canonical, versioned event log.

新增一个不依赖 provider 的 Workflow core 增量：通过既有 scheduler 执行现有类型化图，并从规范、有版本的事件日志精确重建 run。

**Architecture / 架构：** A crate-local execution state machine owns one existing immutable `WorkflowContextBinding`, delegates capability validation and node transitions to `WorkflowScheduler`, and appends canonical events after successful transitions. Replay accepts the same binding and existing `WorkflowCapabilitySnapshot`, validates the schema and exact binding/run metadata, then re-applies events through the scheduler while rejecting gaps, reordered events, events after a terminal marker, and terminal-marker mismatches.

crate 内执行状态机持有一份既有不可变 `WorkflowContextBinding`，将 capability 校验与节点转换委托给 `WorkflowScheduler`，并仅在转换成功后追加规范事件。回放接收同一 binding 与既有 `WorkflowCapabilitySnapshot`，校验 schema 和精确 binding/run metadata，再通过 scheduler 重放事件；序号缺口、事件乱序、terminal marker 后事件及 terminal marker 不一致均 fail closed。

**Tech Stack / 技术栈：** Rust, Serde, thiserror, existing `contextlab-workflow` domain contracts, `cargo test`, strict Clippy.

---

## Necessity Record / 必要性记录

**Criterion and charter principle / 完成条件与宪章原则：** This increment advances Criterion 1, Context-first Workflow platform coverage; Criterion 2, replayable versioning; Criterion 7, provider/plugin extensibility without changing core business logic; and the charter principles that every change is reproducible and business logic remains reusable Rust infrastructure.

本增量推进条件 1“以 Context 为核心的 Workflow 平台覆盖”、条件 2“可回放版本”、条件 7“无需修改核心业务逻辑的 provider/plugin 扩展”，并落实“每次变更均可复现、业务逻辑属于可复用 Rust 基础设施”的宪章原则。

**Unmet gap and risk / 未满足缺口与风险：** The current scheduler deterministically claims nodes and supports creating a fresh run from a terminal scheduler, while source binding and plugin-to-capability adaptation already exist. It does not preserve a schema-versioned event boundary from which the exact execution can be reconstructed and validated. Without that boundary, persistence or transport layers would have to infer event order, terminal failure, Context source identity, and replay consistency.

当前 scheduler 已能确定性 claim node，并可从 terminal scheduler 创建新 run；source binding 与 plugin-to-capability adapter 也已存在。但目前没有可用于精确重建和校验执行过程的 schema-versioned event boundary。缺少该边界时，后续 persistence 或 transport 层将被迫自行推断事件顺序、terminal failure、Context source identity 与 replay consistency。

**Why this is next / 为什么现在做：** The immutable typed node/edge definition, deterministic scheduler, terminal replay rule, exact Context binding, and capability snapshot/bridge are all dependency-ready. A crate-local event state machine is the smallest next step that composes those contracts and prevents later storage or API work from inventing execution semantics.

不可变类型化 node/edge definition、确定性 scheduler、terminal replay rule、精确 Context binding 以及 capability snapshot/bridge 均已依赖就绪。crate 内事件状态机是组合这些契约的最小下一步，并可防止后续 storage 或 API 自行发明执行语义。

**Explicit non-goals / 明确非目标：** No provider or tool invocation, async orchestration, retries, concurrency, persistence, database or Docker work, public/private route, OpenAPI or SDK contract, Web UI, MCP/plugin modification, secret read, branch policy, or scheduling beyond the existing serial scheduler. This increment does not expose raw execution logs through the existing redacted status projection.

不包含 provider 或 tool 调用、异步编排、重试、并发、持久化、数据库或 Docker、public/private route、OpenAPI 或 SDK contract、Web UI、MCP/plugin 修改、secret 读取、branch policy，也不扩展既有串行 scheduler 之外的调度。本增量不会通过既有脱敏 status projection 暴露原始执行日志。

**Smallest affected boundary and bilingual documentation / 最小边界与双语文档：** Only `crates/workflow/src`, focused `crates/workflow/tests`, and this bilingual plan change. The state machine consumes `WorkflowContextBinding` and `WorkflowCapabilitySnapshot` directly, delegates all transitions to `WorkflowScheduler`, and introduces one V1 event-log schema plus structured replay/execution errors.

仅修改 `crates/workflow/src`、聚焦的 `crates/workflow/tests` 与本双语计划。状态机直接消费 `WorkflowContextBinding` 和 `WorkflowCapabilitySnapshot`，将全部转换委托给 `WorkflowScheduler`，并新增一个 V1 event-log schema 与结构化 replay/execution error。

**Fresh verification / 新鲜验证：** Observe a focused compile-time RED proving the state-machine contract is absent; then pass focused tests for success/failure event order, exact replay, schema/scope/order rejection, terminal replay, and adapter reuse. Before completion, run `cargo test -p contextlab-workflow`, `cargo clippy -p contextlab-workflow --all-targets -- -D warnings`, and `cargo fmt --all -- --check`. Provider, transport, persistence, Docker/PostgreSQL, browser, remote CI, release, and production evidence remain unobserved and must not be inferred.

先观察一次聚焦编译 RED，证明状态机契约尚不存在；随后通过 success/failure event order、精确回放、schema/scope/order 拒绝、terminal replay 与 adapter 复用测试。完成前运行 `cargo test -p contextlab-workflow`、`cargo clippy -p contextlab-workflow --all-targets -- -D warnings` 与 `cargo fmt --all -- --check`。不得据此推断 provider、transport、persistence、Docker/PostgreSQL、browser、remote CI、release 或 production 证据。

## File Structure / 文件结构

- Create `crates/workflow/src/execution.rs`: versioned event log, state-machine composition, replay validation, and structured errors.
- Modify `crates/workflow/src/lib.rs`: register and re-export the execution contract only.
- Create `crates/workflow/tests/workflow_deterministic_replay.rs`: behavior-first contract and regression coverage.
- Update this plan with exact RED/GREEN receipts and the final contract boundary.

## Tasks / 任务

### Task 1: Freeze the wished-for execution contract

- [x] Add a focused integration test that starts from `WorkflowContextBinding` plus `WorkflowCapabilitySnapshot`, executes success and failure paths, and asserts the V1 canonical event sequence.
- [x] Run `cargo test -p contextlab-workflow --test workflow_deterministic_replay` and record the expected missing-symbol compile RED.

### Task 2: Implement minimal execution and replay

- [x] Add the V1 log/event schema and state machine that delegates live transitions to `WorkflowScheduler`.
- [x] Add deterministic `from_log` reconstruction with sequence, metadata, transition, and terminal-marker validation.
- [x] Add fresh terminal-run replay using the scheduler's existing replay operation.
- [x] Run the focused suite until GREEN without changing unrelated crates.

### Task 3: Add focused regressions and quality gates

- [x] Cover wrong schema, wrong binding, sequence gaps/reordering, terminal marker mismatch, events after terminal state, and structured transition errors.
- [x] Run the full Workflow crate tests, strict crate Clippy, and crate formatting check.
- [x] Record exact evidence, contract boundary, and residual limitations below.

## Evidence Record / 证据记录

### Integration-review root cause / 集成审阅根因

The independent integration review found that `from_log` accepted an unverified `replay_of`
identifier directly from serialized input. A self-referential or unrelated run could therefore be
presented as replay provenance even though `replay_as` only creates replays from a distinct terminal
source. The blocked condition is trustworthy replay semantics under Criteria 1 and 2. The minimum
repair is to reject any replay log at the root reconstruction boundary unless a validated terminal
source execution is supplied, then require exact source run, binding, Workflow revision, and Context
commit identity before reconstructing it. Focused regressions must prove self-reference rejection,
missing-source rejection, source mismatch rejection, and successful reconstruction from the exact
terminal source before broader verification resumes.

独立集成审阅发现，`from_log` 会直接接受序列化输入中的未验证 `replay_of` 标识。因此，自引用或无关
run 都可能被伪装成 replay provenance，尽管 `replay_as` 只允许从不同的 terminal source 创建回放。
被阻塞的是条件 1 与条件 2 所要求的可信回放语义。最小修复是在 root reconstruction boundary 对任何
缺少已验证 terminal source execution 的 replay log 执行 fail closed，并在重建前要求 source run、binding、
Workflow revision 与 Context commit identity 精确一致。恢复更广验证前，聚焦回归必须证明 self-reference、
missing source、source mismatch 会被拒绝，且精确 terminal source 可以成功重建。

**RED / 红测：** `cargo test -p contextlab-workflow --test workflow_deterministic_replay` first failed with `E0432` because `WorkflowExecution` and `WorkflowExecutionEventV1` did not exist. The next contract expansion failed with missing `WorkflowExecutionReplayError`, `from_log`, `run_state`, `node_state`, `replay_as`, and `Deserialize` for `WorkflowExecutionLogV1`. The schema-boundary regression then failed because an unknown tagged-event field was accepted. The terminal-read regression failed with an observed duplicate `RunSucceeded { sequence: 5 }` after the canonical terminal event at sequence 4.

首次运行 `cargo test -p contextlab-workflow --test workflow_deterministic_replay` 因 `WorkflowExecution` 与 `WorkflowExecutionEventV1` 不存在而以 `E0432` 失败。下一次扩展契约时，缺少 `WorkflowExecutionReplayError`、`from_log`、`run_state`、`node_state`、`replay_as` 与 `WorkflowExecutionLogV1` 的 `Deserialize`。随后 schema-boundary regression 因未知 tagged-event field 被接受而失败。terminal-read regression 观测到 canonical terminal event（sequence 4）后额外追加了 `RunSucceeded { sequence: 5 }`。

**GREEN / 绿测：** The focused command now passes `11 passed; 0 failed`: canonical success and failure order, exact deterministic reconstruction, fresh terminal replay provenance, unverified/self-referential/wrong/non-terminal source rejection, Context-source rejection, sequence-gap and post-terminal rejection, unknown schema/event-field and blank-failure rejection, and no duplicate terminal event on a terminal read.

当前聚焦命令已通过，结果为 `11 passed; 0 failed`：覆盖 canonical success/failure order、精确确定性重建、fresh terminal replay provenance、未验证/self-referential/wrong/non-terminal source 拒绝、Context source 拒绝、sequence gap 与 post-terminal 拒绝、unknown schema/event field 与 blank failure 拒绝，以及 terminal read 不重复追加 terminal event。

**Verification / 验证：** `cargo fmt --all -- --check` passed. `cargo test -p contextlab-workflow` passes `36` tests across 2 binding, 11 deterministic replay, 7 capability-bridge, 15 scheduler, and 1 status-projection integration tests. Workspace strict Clippy and the Rust `1.85.0` workspace check pass. No compile failure remains in the admitted Workflow scope.

`cargo fmt --all -- --check` 已通过。`cargo test -p contextlab-workflow` 在 2 个 binding、11 个 deterministic replay、7 个 capability-bridge、15 个 scheduler 与 1 个 status-projection integration test 中共通过 `36` 项。workspace strict Clippy 与 Rust `1.85.0` workspace check 均通过。已准入 Workflow 范围内没有残留 compile failure。

## Contract Boundary / 契约边界

- `WorkflowExecution` consumes the existing immutable `WorkflowContextBinding` and `WorkflowCapabilitySnapshot`; it does not resolve Context source or capability versions itself.
- All live and reconstructed node transitions use the existing `WorkflowScheduler`; the new state machine only records and verifies canonical events.
- `WorkflowExecutionLogV1` is a strict Serde V1 boundary with exact binding/workflow/context/run metadata, contiguous sequence numbers, explicit `run_started`, and explicit `run_succeeded` or `run_failed` terminal markers.
- `WorkflowExecutionReplayError` returns structured scope, sequence, claim, attempt, scheduler-transition, terminal, and source-replay errors. The existing redacted status projection remains the read boundary and does not expose core log failure detail.

- `WorkflowExecution` 消费既有不可变 `WorkflowContextBinding` 和 `WorkflowCapabilitySnapshot`；它不自行解析 Context source 或 capability version。
- 所有 live/reconstructed node transition 均复用既有 `WorkflowScheduler`；新状态机只记录并校验 canonical event。
- `WorkflowExecutionLogV1` 是严格的 Serde V1 边界，包含精确 binding/workflow/context/run metadata、连续 sequence number、显式 `run_started` 及显式 `run_succeeded` 或 `run_failed` terminal marker。
- `WorkflowExecutionReplayError` 返回结构化的 scope、sequence、claim、attempt、scheduler-transition、terminal 与 source-replay error。既有脱敏 status projection 仍是 read boundary，不会暴露 core log 的 failure detail。

## Residual Limitations / 剩余限制

This is an in-memory, serial, provider-free domain state machine only. It adds no provider or tool execution, retries, concurrency, persistence, database/Docker, public or local transport, SDK, UI, secret access, or raw-log inspection. Plugin capability resolution remains owned by the existing bridge; this state machine accepts its existing snapshot output. Broader workspace, PostgreSQL, browser, remote CI, release, and production evidence were intentionally not run or inferred.

这仅是内存中、串行、provider-free 的 domain state machine。它不新增 provider 或 tool execution、retries、concurrency、persistence、database/Docker、public 或 local transport、SDK、UI、secret access 或 raw-log inspection。plugin capability resolution 仍由既有 bridge 拥有；本状态机只接收其既有 snapshot output。未运行也不得推断 workspace、PostgreSQL、browser、remote CI、release 或 production 证据。
