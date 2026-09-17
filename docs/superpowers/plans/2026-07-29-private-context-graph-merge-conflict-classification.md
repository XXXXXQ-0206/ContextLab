# Private ContextGraph Three-Way Conflict Classification / 私有 ContextGraph 三路冲突分类

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
the replayable version-history and graph-diff portions of Criteria 2 and 4. ContextLab must be able
to inspect a future merge safely from immutable Context Graph snapshots while keeping Context as the
primary abstraction, reusable Rust domain logic as the source of truth, and `GraphDiff::between` as
the sole graph-diff calculator.

本增量直接推进条件 2 与条件 4 中的可回放版本历史和 graph-diff 部分。ContextLab 必须能基于不可变 Context Graph
snapshot 安全审阅未来 merge，同时保持 Context-first、可复用 Rust domain logic 为唯一事实来源，并保持
`GraphDiff::between` 为唯一 graph-diff calculator。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** `MergePlan` now
resolves ancestry and a candidate base, but it does not compare the base-to-left and base-to-right
graph changes. A future merge writer must not accept a caller-built result without a deterministic,
read-only classification of disjoint, equivalent, and conflicting node/edge changes. Storage does
not yet bind this classification to a durable merge write, so this plan intentionally stops before
persistence and mutation.

当前 `MergePlan` 已能解析 ancestry 与 candidate base，但还没有比较 base-to-left 与 base-to-right 的 graph change。
未来 merge writer 在没有确定性的 node/edge disjoint、equivalent、conflicting read-only classification 前不得接受
caller-built result。storage 还没有把 classification 绑定到 durable merge write，因此本计划明确停在 persistence 与
mutation 之前。

**Why now / 为何现在优先:** Branch-head discovery and merge-base resolution are locally verified,
making this the smallest dependency-ready private core increment before any branch merge/rollback
writer. It closes a named conflict-safety gap without requiring PostgreSQL runtime, Docker, public
transport, UI, provider access, or external release evidence.

branch-head discovery 与 merge-base resolution 已完成本地验证，因此这是任何 branch merge/rollback writer 之前最小的、
依赖已满足的 private core 增量。它直接收束 conflict-safety 缺口，不依赖 PostgreSQL runtime、Docker、public
transport、UI、provider access 或 external release evidence。

**Contract / 契约:** Accept an exact `ThreeWay { base, left, right }` `MergePlan`, one Context scope,
and three validated `ContextGraph` snapshots. Reuse `GraphDiff::between(base, left)` and
`GraphDiff::between(base, right)`; return deterministic `Clean`, `Equivalent`, or `Conflict` with
stable node/edge conflict keys. Reject non-three-way plans, plan/scope identity drift, and invalid
input before classification. Never produce a merged graph.

**Boundary and bilingual documentation / 边界与双语文档:** Own only
`crates/diff-engine/src/graph_merge_conflict.rs`, its `lib.rs` export, focused contract tests, this
plan, and the later bilingual architecture/roadmap receipts. No storage, migration, server/api,
OpenAPI, SDK, Web, CLI, Desktop, operator, release, or production files are in scope.

**Explicit non-goals / 明确非目标:**

- No merge writer, branch mutation, rollback, persistence, migration, or branch-head CAS.
- No public REST/OpenAPI/SDK route or method, Web mutation, provider call, Docker/PostgreSQL runtime,
  secret access, remote CI, operator rehearsal, release, or production claim.
- No second graph-diff implementation and no content merge result, conflict auto-resolution, or
  caller-supplied merged snapshot.

- 不实现 merge writer、branch mutation、rollback、persistence、migration 或 branch-head CAS。
- 不新增 public REST/OpenAPI/SDK route 或 method、Web mutation、provider、Docker/PostgreSQL runtime、secret、
  remote CI、operator rehearsal、release 或 production claim。
- 不新增第二个 graph-diff implementation，不生成 content merge result、自动冲突解决或 caller-supplied merged snapshot。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Add
red/green tests for exact three-way applicability, plan/scope drift, disjoint node/edge edits,
equivalent edits, divergent node edits, add/remove collisions, edge collisions, deterministic
ordering, and unchanged GraphDiff singularity. Then run focused diff-engine tests, format, workspace
tests, strict offline Clippy, locked Rust `1.85.0`, Web checks, and static public-surface/GraphDiff
searches. Runtime, browser, Git, remote, operator, release, production, Docker, and secrets remain
unobserved/deferred.

## Implementation Checklist / 实施清单

- [x] Add the pure three-way classification contract and stable conflict identifiers.
- [x] Add focused red/green tests for clean, equivalent, conflicting, and fail-closed inputs.
- [x] Export the contract without adding a second GraphDiff calculator or transport surface.
- [x] Record fresh local verification and bilingual architecture/roadmap receipts; keep the long-term
  goal active.

- [x] 增加 pure three-way classification contract 与稳定 conflict identifier。
- [x] 增加 clean、equivalent、conflicting 与 fail-closed input 的 focused red/green tests。
- [x] 导出契约但不增加第二个 GraphDiff calculator 或 transport surface。
- [x] 记录新鲜本地验证与双语 architecture/roadmap 回执；保持长期目标 active。

## Implementation and Fresh Verification Receipt / 实现与新鲜验证回执

`GraphSnapshotRef` binds each immutable Context Graph to exact `(ProjectId, ContextId, CommitId)`.
`GraphMergeConflictClassifier` accepts only an exact `MergePlan::ThreeWay`, rejects scope or plan
identity drift, invokes `GraphDiff::between` exactly for base-to-left and base-to-right, and returns
deterministic `Clean`, `Equivalent`, or `Conflict` classifications for node and multiset edge
changes. It never creates a merged graph or writes a branch.

`GraphSnapshotRef` 将每个不可变 Context Graph 绑定到 exact `(ProjectId, ContextId, CommitId)`。
`GraphMergeConflictClassifier` 只接受 exact `MergePlan::ThreeWay`，拒绝 scope 或 plan identity drift，只对
base-to-left 与 base-to-right 各调用一次 `GraphDiff::between`，并为 node 与 multiset edge change 返回确定性的
`Clean`、`Equivalent` 或 `Conflict`。它不会创建 merged graph，也不会写入 branch。

Fresh local evidence passed: diff-engine focused targets `6`, workspace Rust `186 passed, 39
ignored`, `cargo fmt --all -- --check`, strict offline workspace Clippy, locked Rust `1.85.0` check,
`pnpm check:web` with public SDK `15`, local SDK `92`, Web `185`, and production build, plus static
`impl GraphDiff count=1`. PostgreSQL/Docker runtime, authenticated browser, Git, remote CI, operator
rehearsal, release, and production remain `unobserved` or `deferred`; no secrets were read.

新鲜本地证据为 diff-engine focused targets `6`、workspace Rust `186 passed, 39 ignored`、
`cargo fmt --all -- --check`、strict offline workspace Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`（public SDK `15`、local SDK `92`、Web `185` 与 production build），以及静态
`impl GraphDiff count=1`。PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`；未读取 secrets。
