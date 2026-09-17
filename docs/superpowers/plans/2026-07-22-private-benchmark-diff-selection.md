# Private Benchmark Diff Selection Plan / 私有 Benchmark Diff Selection 计划

## Necessity Record / 必要性记录

**Completion criteria and charter principle / 完成条件与宪章原则：** This increment directly serves
Criterion 3, `Benchmark-driven evaluation`, and the charter requirement that versioned Context
history and evaluation evidence be replayable and comparable. The existing private evaluation-diff
route and `GraphDiff::between` boundary are already implemented; this increment only makes the two
sealed decision identities discoverable at their selected commit scopes before comparison.

本增量直接服务条件 3“Benchmark-driven evaluation”，以及宪章关于 versioned Context history 与
evaluation evidence 必须可回放、可比较的要求。现有 private evaluation-diff route 与
`GraphDiff::between` boundary 已实现；本增量只在 comparison 前让两个 sealed decision identity
在各自 selected commit scope 下可发现。

**Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口：** The preceding private
decision-discovery slice supplies the exact-scope redacted list, but its current safe-summary
hardening still requires fresh verification. The former Web diff inspector required manual baseline
and revised `decision_id` entry. The replacement selection wiring is now implemented; its remaining
risk is proving exact dual scope and stale-selection clearing through fresh focused and cross-stack
checks.

前一条私有 decision-discovery slice 提供 exact-scope 的脱敏 list，但其当前 safe-summary hardening
仍需新鲜验证。旧的 Web diff inspector 要求手工填写 baseline 与 revised `decision_id`。替代的
selection wiring 现已实现；剩余风险是通过新鲜的聚焦与 cross-stack 检查证明 exact dual scope 与
stale-selection clearing。

**Why now / 为什么现在优先：** The exact-scope discovery repository, redacted list schema, protected
local transport, and existing evaluation-diff read are available. The selection adapter is the
smallest step that connects these contracts into the named benchmark comparison criterion; it does
not require benchmark writes, provider execution, or policy changes. Current implementation must
be verified after the discovery hardening rather than relying on the earlier green snapshot.

exact-scope discovery repository、脱敏 list schema、protected local transport 与既有 evaluation-diff
read 均已存在。selection adapter 是连接这些契约与 benchmark comparison 条件的最小步骤，不需要
benchmark write、provider execution 或 policy change。当前实现必须在 discovery hardening 后完成验证，
不能继续依赖此前的 green snapshot。

**Explicit non-goals / 明确非目标：** No benchmark mutation, provider/evaluator invocation,
threshold/regression recomputation, raw case/run payload, public REST/OpenAPI/public SDK method,
Web mutation, operator transport, Docker/PostgreSQL runtime, browser E2E, release, production, or
new graph-diff calculator. The selection flow must not call `GraphDiff::between`; the existing
version-backed evaluation diff route remains the only comparison path.

不包含 benchmark mutation、provider/evaluator invocation、threshold/regression recomputation、raw
case/run payload、public REST/OpenAPI/public SDK method、Web mutation、operator transport、
Docker/PostgreSQL runtime、browser E2E、release、production 或新的 graph-diff calculator。selection
flow 不得调用 `GraphDiff::between`；既有 version-backed evaluation diff route 仍是唯一 comparison
path。

**Smallest affected boundary and bilingual docs / 最小受影响边界与双语文档：** Add only a Web
selection data adapter/presenter and the existing diff inspector's screen wiring. Reuse the existing
private discovery client and parser, preserve the exact project/Context/commit target for each side,
and keep all domain/application policy in Rust. Update this plan, the active goal, completion audit,
architecture note, and Wave 3 verification matrix with observed evidence only.

最小边界只增加 Web selection data adapter/presenter 与既有 diff inspector 的 screen wiring。复用
现有 private discovery client/parser，保持两侧各自 exact project/Context/commit target，并将所有
domain/application policy 留在 Rust。只用已观察证据更新本计划、active goal、completion audit、
architecture note 与 Wave 3 verification matrix。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Red/green
tests must prove both commit scopes are fetched, missing or empty sealed lists disable comparison,
stale selections are cleared on commit changes, request-scoped credentials omit cookies, and the
existing diff request receives exactly the selected dual scope. Then run `cargo fmt --all -- --check`,
`cargo test --workspace --quiet`, `pnpm check:web`, focused Web selection/diff tests, and public-surface
and `GraphDiff` searches. Docker/PostgreSQL runtime, browser, remote, release, and production evidence
remain unobserved or deferred.

下一增量前必须取得新鲜验证：red/green test 证明两侧 commit scope 都被读取、missing 或 empty sealed
list 会禁用 comparison、commit change 会清除 stale selection、request-scoped credential 不携带
cookie，以及既有 diff request 只收到选定的 dual scope。随后运行 `cargo fmt --all -- --check`、
`cargo test --workspace --quiet`、`pnpm check:web`、Web selection/diff 聚焦 test 与 public-surface/
`GraphDiff` search。Docker/PostgreSQL runtime、browser、remote、release 与 production evidence 仍为
未观测或延期。

## Status / 状态

- [x] Add focused tests for dual exact-scope selection and stale-selection clearing.
- [x] Implement data/presenter/screen selection wiring through the existing private discovery contract.
- [ ] Run fresh cross-stack verification and update bilingual evidence.
- [ ] Keep the long-term goal active and select the next dependency-ready increment.

## Current Implementation State / 当前实现状态

The private selection data adapter, presenter, and diff-inspector screen wiring are present. This
record includes one worker-observed focused receipt: selection tests `5 passed`. The main
integration thread must still run the full Web check, Rust formatting and workspace checks, and the
public-surface/`GraphDiff` boundary searches before marking the increment freshly green.

私有 selection data adapter、presenter 与 diff-inspector screen wiring 已存在。本记录刻意不作
cross-stack 通过声明；worker 已观察到一项聚焦回执：selection test `5 passed`。main integration
thread 仍必须运行完整 Web check、Rust formatting 与 workspace check，以及 public-surface/
`GraphDiff` boundary search，之后才能把本增量标记为新鲜 green。
