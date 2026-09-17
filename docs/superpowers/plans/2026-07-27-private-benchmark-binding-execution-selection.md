# Private Benchmark Binding Execution Selection / 私有 Benchmark 绑定执行选择

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则:** This increment directly
advances Criterion 3, which requires benchmark definitions to be reusable, version-bound, queryable,
and connected to evaluation workflows. It also reinforces the Context-first and replayable-version
principles by making the exact immutable Context commit the source of execution selection.

本增量直接推进条件 3：benchmark definition 必须可复用、绑定版本、可查询，并连接到评测 workflow；
同时强化 Context-first 与可回放版本原则，使精确不可变 Context commit 成为执行选择的来源。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The private authoring
writer and binding inspection now preserve a complete suite and dataset definition at an exact
project/Context/commit scope. The existing `BenchmarkExecutionService` still accepts a suite ID
directly, so a future caller could select a suite that was not the inspected immutable binding. A
provider-free selection value is the smallest bridge that closes this contract gap without expanding
transport or evaluator behavior.

私有 authoring writer 与 binding inspection 现已在精确 project/Context/commit scope 保存完整 suite 与
dataset definition。现有 `BenchmarkExecutionService` 仍直接接受 suite ID，因此未来 caller 可能选择
并非 inspected immutable binding 的 suite。增加 provider-free selection value 是收束该 contract 缺口的
最小桥接，不会扩大 transport 或 evaluator 行为。

**Why now / 为什么现在优先:** The binding inspection increment and existing deterministic
`BenchmarkExecutionPlan` are freshly verified. The selection bridge can be built entirely from those
stable contracts, before any provider, benchmark mutation, public route, or Web execution control is
admitted. It is closer to Criterion 3 than unrelated workflow, memory, or graph-editor expansion.

**为什么现在优先：** binding inspection 增量与现有确定性的 `BenchmarkExecutionPlan` 已获得新鲜验证。
selection bridge 可以完全建立在这些稳定 contract 之上，且早于任何 provider、benchmark mutation、
public route 或 Web execution control 的准入。它比无关的 workflow、memory 或 graph-editor 扩张更接近条件 3。

**Minimal affected boundary / 最小受影响边界:** Restrict implementation to
`crates/storage/src/benchmark_execution.rs` and its focused Rust tests, with only the existing
evaluation plan and binding accessors as inputs. Add a bilingual architecture/API/roadmap record.
No server route, OpenAPI, SDK, BFF, Web mutation, migration, or second graph-diff implementation.

实现严格限制在 `crates/storage/src/benchmark_execution.rs` 及其聚焦 Rust test，并只使用既有 evaluation
plan 与 binding accessor。增加双语 architecture/API/roadmap record。不增加 server route、OpenAPI、SDK、
BFF、Web mutation、migration 或第二个 graph-diff implementation。

**Explicit non-goals / 明确非目标:** No provider or evaluator invocation, benchmark evidence or
workspace projection persistence, raw case/input/output transport, mutable latest lookup, public
REST/OpenAPI/public SDK method, operator transport, production claim, or new graph-diff calculator.
`GraphDiff::between` remains the sole graph-diff calculator.

不调用 provider 或 evaluator，不持久化 benchmark evidence 或 workspace projection，不传输 raw case/input/output，
不解析 mutable latest，不新增 public REST/OpenAPI/public SDK method、operator transport、production 声明或新的
graph-diff calculator。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Fresh verification before the next increment / 下一增量前的新鲜验证:** Observe focused red/green
tests for exact binding scope, stable suite/dataset membership, deterministic case count, rejection
of mismatched scope or malformed binding, and no raw payload in the selection value. Then run
`cargo fmt --all -- --check`, focused and workspace Rust tests, strict workspace Clippy, the locked
Rust `1.85.0` check, the Wave 2 contract verifier, and `pnpm check:web`. PostgreSQL runtime,
authenticated browser runtime, Git, remote CI, operator rehearsal, release, and production remain
unobserved or deferred unless directly observed.

**下一增量前的新鲜验证：** 观察精确 binding scope、稳定 suite/dataset membership、确定性 case count、
scope mismatch 或 malformed binding 拒绝，以及 selection value 不包含 raw payload 的聚焦红绿测试。随后运行
`cargo fmt --all -- --check`、聚焦与 workspace Rust test、strict workspace Clippy、锁定 Rust `1.85.0` check、
Wave 2 contract verifier 与 `pnpm check:web`。除非真实观测，PostgreSQL runtime、authenticated browser runtime、
Git、remote CI、operator rehearsal、release 与 production 仍保持 unobserved 或 deferred。

## Implementation Tasks / 实施任务

- [x] Add failing tests for selection scope and definition membership before implementation.
- [x] Implement the immutable provider-free selection value and plan assembly using existing accessors.
- [x] Add replay/equality and malformed-input regressions without exposing raw cases.
- [x] Record fresh local verification and select the next increment; keep the long-term goal active.

## Fresh Verification Receipt / 新鲜验证回执

The red/green contract was observed: the new test initially failed because the selection type and
error contract did not exist, then passed as selection `2/2`. The execution integration regression
passed `11/11`. Fresh workspace evidence is `cargo fmt --all -- --check`, `cargo test --workspace --quiet`
with API `170 passed` and storage `167 passed, 39 ignored`, strict workspace Clippy,
`cargo +1.85.0 check --workspace --all-targets --locked`, and `pnpm check:web` with public SDK `14`,
local SDK `70`, Web `156`, and the production build. The graph boundary search found one application
`GraphDiff::between` implementation. No public route or SDK changed. PostgreSQL runtime,
authenticated browser runtime, Git binding, remote CI, operator rehearsal, release, and production
remain unobserved or deferred.

本次红绿 contract 已被观察：新 test 最初因 selection type 与 error contract 不存在而失败，随后 selection
聚焦测试 `2/2` 通过；execution integration regression `11/11` 通过。workspace 新鲜证据包括
`cargo fmt --all -- --check`、`cargo test --workspace --quiet`（API `170 passed`；storage `167 passed, 39 ignored`）、
strict workspace Clippy、`cargo +1.85.0 check --workspace --all-targets --locked`，以及 `pnpm check:web`
（public SDK `14`、local SDK `70`、Web `156`，并完成 production build）。graph boundary search 找到一个
application `GraphDiff::between` implementation。没有改变 public route 或 SDK。PostgreSQL runtime、authenticated
browser runtime、Git binding、remote CI、operator rehearsal、release 与 production 仍为 unobserved 或 deferred。
