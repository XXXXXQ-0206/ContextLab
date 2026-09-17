# Private Benchmark Regression, Scorecard, and Evaluation-Diff Closure / 私有 Benchmark 回归、Scorecard 与 Evaluation-Diff 收束

**Status / 状态:** completed / verified locally / 已完成，本地验证通过

## Necessity Record / 必要性记录

### Criterion and charter principle / 完成条件与宪章原则

This increment serves the local benchmark-driven evaluation and replayable-history criteria,
especially the requirement that a Context change can be inspected through deterministic scorecard,
regression, and evaluation-diff evidence. It also protects the charter principles that Context is
the primary abstraction, reusable Rust owns policy, transport remains an adapter, and the product is
bilingual and design-system-first.

本增量服务于本地 benchmark 驱动评测与可回放版本历史完成条件，重点证明 Context 变化可以通过确定性的
scorecard、regression 与 evaluation-diff evidence 被审阅。同时维护 Context-first、可复用 Rust 承载策略、
transport 仅作 adapter，以及中英双语、design-system-first 的宪章原则。

### Gap and dependency / 缺口与依赖

The domain policy (`BenchmarkSuite -> Scorecard -> RegressionDecision`), immutable exact-commit
decision/evidence persistence, workspace projection, private local route, local SDK parser, BFF,
and Web `data -> presenter -> screen` boundary already exist. The remaining gap is a fresh,
scope-matched closure receipt proving that these layers preserve sealed evidence, threshold coverage,
deterministic ordering, exact baseline/revised scope, replay semantics, redaction, and one evaluation
diff policy path. Branch/merge/replay expansion is not an equivalent prerequisite: the guarded writer
still rejects merge parents and has no merge-base or conflict contract.

当前已有 `BenchmarkSuite -> Scorecard -> RegressionDecision` domain policy、exact-commit immutable
decision/evidence persistence、workspace projection、private local route、local SDK parser、BFF 与 Web
`data -> presenter -> screen` 边界。剩余缺口是取得一份范围匹配的新鲜收束回执，证明这些层保持 sealed evidence、
threshold coverage、确定性排序、exact baseline/revised scope、replay 语义、脱敏，以及唯一的 evaluation-diff
policy path。branch/merge/replay 扩展不是等价前置：guarded writer 仍拒绝 merge parents，且尚无 merge-base 或冲突契约。

### Why now / 为什么现在优先

This is the highest-priority dependency-ready local increment after commit-scoped diff review:
its implementation dependencies are present and its evidence gap can be closed without Docker,
PostgreSQL runtime, provider calls, public transport, or external receipts. It directly advances
Criteria 3 and 4; branch/merge would require new write semantics and is therefore later.

这是 commit-scoped diff review 之后最高优先级且依赖已满足的本地增量：实现依赖已经具备，证据缺口无需 Docker、
PostgreSQL runtime、provider call、public transport 或 external receipt 即可收束。它直接推进条件 3 与 4；
branch/merge 需要新增写入语义，因此后置。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK expansion, public write, or Web mutation.
- No provider/evaluator/model call, raw case/output exposure, or new persistence migration.
- No branch create/fork/merge/rollback mutation and no authenticated browser, Docker, production,
  remote CI, operator, release, or external-receipt work.
- No second scorecard/regression/evaluation-diff policy path and no second `GraphDiff` calculator.

- 不扩展 public REST/OpenAPI/public SDK，不增加 public write 或 Web mutation。
- 不调用 provider/evaluator/model，不暴露 raw case/output，不新增 persistence migration。
- 不做 branch create/fork/merge/rollback mutation，不做 authenticated browser、Docker、production、
  remote CI、operator、release 或 external receipt 工作。
- 不增加第二套 scorecard/regression/evaluation-diff policy path，也不增加第二个 `GraphDiff` calculator。

### Minimal boundary and bilingual documentation / 最小边界与双语文档

The implementation boundary is limited to existing evaluation/storage contract tests and their
smallest necessary Rust or adapter fix, plus this plan and the matching roadmap evidence entry.
Existing local SDK/Web files may only change if a contract test demonstrates an actual parser,
scope, redaction, or presenter defect. No unrelated module is owned by this increment.

实现边界仅限于现有 evaluation/storage contract tests 及其必要的最小 Rust 或 adapter 修复，以及本计划和对应的
路线图证据条目。只有 contract test 证明存在 parser、scope、redaction 或 presenter defect 时，才可修改现有
local SDK/Web 文件；本增量不拥有无关模块。

### Fresh verification required / 下一增量前的新鲜验证

At minimum, run the focused evaluation, storage, API, local SDK, and Web checks; then run format,
workspace tests, strict offline Clippy, locked Rust `1.85.0` checks, and static scope searches for
public-surface expansion and duplicate diff/policy calculators. Record only observed results.

至少运行 evaluation、storage、API、local SDK 与 Web 聚焦检查；随后运行 format、workspace tests、strict offline
Clippy、锁定 Rust `1.85.0` 检查，并静态搜索 public-surface expansion 与重复 diff/policy calculator。只记录实际观测结果。

## Implementation and Verification / 实施与验证

- [x] Add or repair only a contract test exposed by the baseline audit. Storage tests now cover
  same-commit exact decision comparison and complete workspace projection scope/redaction/diff facts;
  no production implementation defect was found.
- [x] Preserve server-owned policy, exact scope, immutable replay/conflict, redaction, and deterministic ordering.
- [x] Run the full fresh verification gate and inspect the owned scope.
- [x] Update the bilingual roadmap receipt without closing the long-term goal.

- [x] 仅增加或修复基线审计暴露的 contract test。Storage test 现覆盖 same-commit exact decision
  comparison 与完整 workspace projection scope/redaction/diff fact；未发现 production implementation defect。
- [x] 保持服务端 policy、exact scope、immutable replay/conflict、脱敏与确定性排序。
- [x] 运行完整新鲜验证门槛并检查受影响边界。
- [x] 更新双语路线图回执，但不关闭长期目标。

### Observed receipt / 已观测回执

The bounded `gpt-5.6-luna` reviews found no evaluation or local SDK/Web implementation gap. The
storage review added only the missing contract assertions in
`crates/storage/tests/benchmark_evidence.rs` and
`crates/storage/tests/benchmark_workspace_projection.rs`. The completed local workflow is
decision-list discovery -> exact decision-bound workspace -> server-owned run details, scorecard,
regression, and evaluation-diff evidence -> Web presentation; the route remains private and
read-only, and the existing cohort-keyed compatibility path remains intact. Fresh focused receipts
are evaluation `45 passed`, storage evidence `23 passed`, workspace projection `7 passed`,
execution `11 passed`, API `191 passed`, local SDK `104 passed`, and the Web presenter/editor
checks `22 passed`. The full gate is format passed, workspace Rust storage `204 passed, 39 ignored`,
strict offline workspace Clippy passed, locked Rust `1.85.0` check passed, and `pnpm check:web`
passed public SDK `15`, local SDK `104`, Web `213`, plus production build.
Static scope inspection observed one `impl GraphDiff`; the public SDK/OpenAPI retirement checks also
passed inside `pnpm check:web`.

有界 `gpt-5.6-luna` review 未发现 evaluation 或 local SDK/Web implementation gap。Storage review 只在
`crates/storage/tests/benchmark_evidence.rs` 与 `crates/storage/tests/benchmark_workspace_projection.rs` 补充缺失的
contract assertion。已完成的本地 workflow 为 decision-list discovery -> exact decision-bound workspace -> server-owned
run details、scorecard、regression 与 evaluation-diff evidence -> Web presentation；route 仍是 private、read-only，既有
cohort-keyed compatibility path 保持不变。新鲜 focused receipt 为 evaluation `45 passed`、storage evidence `23 passed`、
workspace projection `7 passed`、execution `11 passed`、API `191 passed`、local SDK `104 passed` 与 Web presenter/editor
`22 passed`。完整门禁为 format 通过、workspace Rust storage `204 passed, 39 ignored`、strict offline workspace Clippy 通过、
锁定 Rust `1.85.0` check 通过，以及 `pnpm check:web` 通过 public SDK `15`、local SDK `104`、Web `213` 并完成 production build。
静态 scope inspection 观测到唯一 `impl GraphDiff`；public SDK/OpenAPI retirement check 也在 `pnpm check:web` 内通过。

The direct ad hoc Web Vitest command was `unobserved` because the workspace does not expose a direct
`vitest` binary; the repository-owned `pnpm check:web` is the authoritative Web command and passed.
PostgreSQL runtime, authenticated browser runtime, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. No secret, Docker runtime, provider call,
public write, migration, or public surface was added.

直接调用 Web Vitest 的命令因 workspace 未暴露直接 `vitest` binary 而为 `unobserved`；仓库既有的
`pnpm check:web` 是权威 Web command，且已通过。PostgreSQL runtime、authenticated browser runtime、Git change-set、
remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。没有新增 secret、Docker
runtime、provider call、public write、migration 或 public surface。

## Evidence Boundary / 证据边界

Local tests and builds are local non-production evidence only. PostgreSQL runtime, authenticated
browser, Git change-set, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred` unless independently observed; no secret is read and no external receipt
is inferred.

本地 tests 与 builds 仅是本地非生产证据。除非独立观测，PostgreSQL runtime、authenticated browser、Git
change-set、remote CI、operator rehearsal、release 与 production 继续标记为 `unobserved` 或 `deferred`；不读取
secret，也不从本地结果推断 external receipt。
