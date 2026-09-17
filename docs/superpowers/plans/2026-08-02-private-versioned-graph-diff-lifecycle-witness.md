# Private Versioned Graph-Diff Lifecycle Witness / 私有版本化 Graph-Diff 生命周期见证

## Necessity Record / 必要性记录

**Named completion criteria and charter principles / 命名完成条件与章程原则:** This increment
directly serves Criterion 2 (replayable version history) and Criterion 4 (reviewable versioned
change). A version-backed graph comparison must prove that each graph snapshot belongs to a
complete, internally consistent Context lifecycle state before `GraphDiff::between` compares it.

本增量直接服务条件 2（可回放版本历史）与条件 4（可审阅的版本化变更）。version-backed graph comparison
只有在每一侧 graph snapshot 已被证明属于完整且内部一致的 Context lifecycle state 后，才可调用
`GraphDiff::between`。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The existing private
graph-diff adapter reads exact graph snapshots and checks their pair scope, but it does not consume
the existing `ContextLifecycleReadRepository`. A graph can therefore be present while its replay
state, component inventory, content revision/provenance, or graph relationships are absent or
mixed across commits. Independent review identified this as the smallest remaining integrity gap.

现有 private graph-diff adapter 会读取 exact graph snapshot 并校验 pair scope，但没有消费既有
`ContextLifecycleReadRepository`。因此 graph 存在时，replay state、component inventory、content revision/provenance
或 graph relationship 仍可能缺失或跨 commit 混用。独立审查确认这是当前最小的完整性缺口。

**Why now / 为什么现在优先:** The snapshot immutability and resolver-scope repairs are now
locally verified. Before more benchmark, workflow, or editor consumers rely on version-backed graph
review, the read path must reject incomplete evidence at its existing storage boundary. This closes
an already-named version/replay criterion without widening the API surface.

snapshot immutability 与 resolver-scope 修复现已取得本地新鲜验证。在更多 benchmark、workflow 或 editor consumer
依赖 version-backed graph review 前，读取路径必须在现有 storage boundary 拒绝不完整证据。本增量直接收束已命名的
version/replay 条件，不扩大 API surface。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Extend
the private graph-diff composition to load both exact `ContextLifecycleReadFacts` values, validate
the existing lifecycle consistency rules, then pass only the validated graph snapshots to the
existing versioned diff service. Add focused Memory/API regressions, this plan, and bilingual
roadmap/architecture receipts. Do not alter the response DTO or route shape.

最小边界是扩展 private graph-diff composition，读取两侧 exact `ContextLifecycleReadFacts`，复用既有 lifecycle
一致性校验，再只把已验证的 graph snapshot 交给现有 versioned diff service。新增 focused Memory/API regression、
本计划以及双语 roadmap/architecture 回执。不改变 response DTO 或 route shape。

**Explicit non-goals / 明确非目标:** No public REST/OpenAPI/SDK write, new route, Web mutation,
operator transport, migration, provider, raw private content, PostgreSQL runtime claim, external
release evidence, or second graph-diff calculator. `GraphDiff::between` remains the sole graph-diff
calculator, and the local protected path remains read-only and default-off.

不新增 public REST/OpenAPI/SDK write、新 route、Web mutation、operator transport、migration、provider、raw private
content、PostgreSQL runtime 声明或 external release evidence，也不新增第二个 graph-diff calculator。
`GraphDiff::between` 继续是唯一 graph-diff calculator，local protected path 继续只读且默认关闭。

**Fresh verification before the next increment / 下一增量前的新鲜验证:** First observe red
regressions for missing or mixed lifecycle witness, then green focused storage/API/diff tests;
`cargo fmt --all -- --check`; offline workspace Rust tests; strict offline Clippy; locked Rust
`1.85.0` check; `pnpm check:web`; the local contract verifier; and a source count proving one
`GraphDiff` implementation. PostgreSQL runtime, browser, Git, remote CI, operator, release, and
production remain `ignored`, `unobserved`, or `deferred` unless independently observed.

先观测 missing 或 mixed lifecycle witness 的红色 regression，再取得 storage/API/diff focused green test；随后运行
`cargo fmt --all -- --check`、offline workspace Rust tests、strict offline Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`、local contract verifier 与唯一 `GraphDiff` implementation source count。PostgreSQL runtime、browser、
Git、remote CI、operator、release 与 production 除非独立观测，否则继续标记为 `ignored`、`unobserved` 或 `deferred`。

## Implementation Receipt / 实现回执

- [x] Red/green focused regressions for incomplete or mixed lifecycle witness. The focused
  storage suite reports `4 passed`, including missing snapshot, mixed lifecycle scope, invalid
  pair, and exact-pair preservation cases; the red phase was observed before the implementation
  in the admitted execution record.
- [x] Private graph-diff composition consumes both exact lifecycle facts before comparison. The
  adapter validates both `ContextLifecycleReadFacts` values before delegating the graph pair to
  the existing versioned review service.
- [x] Fresh Rust/Web/contract verification and bilingual roadmap/architecture receipts. This run
  observed API scope `3 passed`, protected GraphDiff `7 passed`, workspace Rust `221 passed, 41
  ignored`, format, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web` with public SDK
  `15`, local SDK `148`, Web `298`, and production build, the fixture verifier, and exactly one
  `impl GraphDiff` source. PostgreSQL runtime, Docker, browser/visual, Git, remote CI, operator,
  release, and production evidence remain `ignored`, `unobserved`, or `deferred`.

- [x] 不完整或混合 lifecycle witness 的红绿 focused regression。focused storage suite 报告
  `4 passed`，覆盖 missing snapshot、mixed lifecycle scope、invalid pair 与 exact-pair
  preservation；准入执行记录保留了实现前的 red phase。
- [x] private graph-diff composition 在比较前消费两侧 exact lifecycle facts。adapter 会在将
  graph pair 委托给既有 versioned review service 前校验两份 `ContextLifecycleReadFacts`。
- [x] 新鲜 Rust/Web/contract 验证与双语 roadmap/architecture 回执。本轮观测到 API scope
  `3 passed`、protected GraphDiff `7 passed`、workspace Rust `221 passed, 41 ignored`、format、
  strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`（public SDK `15`、local SDK `148`、
  Web `298` 与 production build）、fixture verifier，以及唯一一个 `impl GraphDiff` source。
  PostgreSQL runtime、Docker、browser/visual、Git、remote CI、operator、release 与 production
  evidence 继续为 `ignored`、`unobserved` 或 `deferred`。
