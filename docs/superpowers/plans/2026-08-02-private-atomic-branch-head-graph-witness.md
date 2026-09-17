# Private Atomic Branch-Head Graph Witness / 私有原子 Branch-Head Graph Witness

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly advances Criterion 2 (replayable version history) and Criterion 4
(Context Graph as the system skeleton). A server-owned `BranchName` must resolve to the exact
revised commit, complete validated history, and both immutable graph snapshots at one backend-owned
observation point before version-backed graph comparison is projected. / 本增量直接推进条件 2（可回放版本历史）与
条件 4（Context Graph 作为系统骨架）。在投影版本化 graph comparison 之前，server-owned `BranchName` 必须在一个
backend-owned observation point 内解析为 exact revised commit，并与完整的 validated history 及两份 immutable graph
snapshot 绑定。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The existing `review_branch_head` reads `CommitHistory` through one repository boundary, selects
the head, and then reads graph snapshots through another boundary. A concurrent head advance can
therefore mix an old history/head with a new snapshot. The existing exact-pair witness is atomic,
but it accepts a caller-supplied target commit and does not prove that the target is the selected
branch head. / 现有 `review_branch_head` 通过一个 repository boundary 读取 `CommitHistory` 并选择 head，随后通过另一个
boundary 读取 graph snapshot；并发 head 推进可能造成旧 history/head 与新 snapshot 混合。现有 exact-pair witness 虽然是原子的，
但接受调用方传入的 target commit，不能证明 target 就是所选 branch head。

### Why now / 为什么现在优先

Complete history, exact snapshot witness, `CommitHistory::head`, and the existing graph-review
projection are already locally verified. This private storage contract is the smallest dependency-
ready correction before another Context consumer or any branch-selection transport. It keeps branch
policy in reusable Rust storage boundaries instead of duplicating it in API, SDK, or Web code. /
完整 history、exact snapshot witness、`CommitHistory::head` 与既有 graph-review projection 已经完成本地验证。这个 private
storage contract 是在增加其他 Context consumer 或 branch-selection transport 之前，当前依赖已满足的最小修正；它将 branch
policy 保留在可复用的 Rust storage boundary 中，不在 API、SDK 或 Web 中复制。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/SDK route or method, no Web mutation, no branch mutation, merge, rollback, migration, provider, operator transport, release, or production claim.
- No live PostgreSQL/Docker receipt is manufactured; SQL-shape evidence remains separate from runtime evidence.
- No second graph-diff calculator. `GraphDiff::between` remains the sole calculator, reached through the existing versioned review projection.

- 不新增 public REST/OpenAPI/SDK route 或 method，不新增 Web mutation、branch mutation、merge、rollback、migration、provider、operator transport、release 或生产声明。
- 不伪造 live PostgreSQL/Docker 回执；SQL-shape evidence 与 runtime evidence 保持分离。
- 不新增第二个 graph-diff calculator；`GraphDiff::between` 仍是唯一 calculator，并继续通过既有 versioned review projection 调用。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

The implementation boundary is the private `crates/storage` branch-bound witness contract and its
Memory/PostgreSQL adapters, the existing private witness review adapter, focused storage tests, and
this bilingual plan plus roadmap receipts. The existing public exact-pair witness trait remains
unchanged; no API, SDK, BFF, Web, versioning, or migration file is needed. / 实现边界限定为 private `crates/storage` branch-bound
witness contract、Memory/PostgreSQL adapter、既有 private witness review adapter、storage focused tests，以及本双语计划和
roadmap 回执。既有 public exact-pair witness trait 保持不变；不需要修改 API、SDK、BFF、Web、versioning 或 migration。

### Fresh verification before the next increment / 下一增量前的新鲜验证

First observe a red contract for branch selection that cannot be satisfied by the old split-read
service. Then obtain green Memory branch-bound tests, PostgreSQL SQL-shape/static tests, and review
projection tests. Before the next increment, run focused storage/API tests, workspace Rust, format,
strict offline Clippy, locked Rust 1.85, Web/SDK checks, the local contract verifier, and the exact-
one `GraphDiff` source check. Live PostgreSQL, Docker, browser/visual, Git, remote CI, operator,
release, and production remain unobserved or deferred. / 先观测旧 split-read service 无法满足的 branch selection 红 contract，
再取得 Memory branch-bound、PostgreSQL SQL-shape/static 与 review projection 绿灯。下一增量前运行 storage/API focused、workspace Rust、
format、strict offline Clippy、锁定 Rust 1.85、Web/SDK、local contract verifier 与 exact-one `GraphDiff` source check。live PostgreSQL、Docker、
browser/visual、Git、remote CI、operator、release 与 production 继续为 unobserved 或 deferred。

## Implementation Boundary / 实施边界

Add a private repository contract that accepts `source_scope` and a typed `BranchName`, selects the
branch head, reads complete history and both exact snapshots at one backend-owned observation point,
and returns an immutable branch-bound witness. The witness must retain the selected branch and exact
target commit, reject unknown/unborn branches, reject scope drift, and fail closed on missing target
snapshots. The application adapter delegates the resulting exact pair to the existing
`PersistedContextGraphDiffReviewService::project_snapshots`; storage does not calculate a diff. /
新增 private repository contract，接收 `source_scope` 与 typed `BranchName`，在一个 backend-owned observation point 内选择
branch head、读取完整 history 与两份 exact snapshot，并返回 immutable branch-bound witness。witness 必须保留 selected branch 与 exact
target commit，拒绝 unknown/unborn branch，拒绝 scope drift，并在 target snapshot 缺失时 fail closed。application adapter 将 exact pair
委托给既有 `PersistedContextGraphDiffReviewService::project_snapshots`；storage 不计算 Diff。

## Status / 状态

`completed / verified locally`; the long-term goal remains `active`. This plan is not a project
completion claim. / `completed / verified locally`；长期目标保持 `active`。本计划不构成项目完成声明。

## Gate Correction / 门禁修正

The boundary audit confirms that the split `PersistedContextGraphHistoryReviewService` exposes only
explicit commit-scope review; it has no callable `review_branch_head` method. The only
`review_branch_head` method is on `PersistedContextGraphWitnessReviewService`, whose repository
constraint is `ContextGraphBranchHeadReviewWitnessRepository` and whose branch selection is atomic.
This distinction prevents a split-read branch selector from being mistaken for the admitted witness
path. / 边界审计确认，split `PersistedContextGraphHistoryReviewService` 只暴露 explicit commit-scope review，不存在可调用的
`review_branch_head` 方法。唯一的 `review_branch_head` 方法位于 `PersistedContextGraphWitnessReviewService`，其 repository
constraint 是 `ContextGraphBranchHeadReviewWitnessRepository`，branch selection 具有 atomic 语义。该区分防止将 split-read
branch selector 误认为已准入的 witness path。

The correction must be proven by a source-boundary search showing no callable branch-head method on
the split service and exactly one branch selector on the witness service, plus the existing six
Memory branch-bound tests, PostgreSQL SQL-shape contract, API error mapping, workspace Rust, format,
strict Clippy, locked Rust 1.85, Web/SDK, and local contract checks. / 本修正必须由 source-boundary search 证明 split service
不存在可调用的 branch-head method，且 witness service 只有一个 branch selector；同时通过既有六项 Memory branch-bound tests、
PostgreSQL SQL-shape contract、API error mapping、workspace Rust、format、strict Clippy、锁定 Rust 1.85、Web/SDK 与 local
contract checks。

## Completion Receipt / 收束回执

No split-read branch-head method is exposed by `PersistedContextGraphHistoryReviewService`.
`ContextGraphBranchHeadReviewWitness` now owns branch selection and exact snapshot binding; Memory holds one read guard and PostgreSQL uses one
`REPEATABLE READ READ ONLY` transaction. The application layer delegates comparison to the existing
versioned review projection, and `GraphDiff::between` remains the sole graph-diff calculator. /
旧的 split-read branch-head method 已删除。`ContextGraphBranchHeadReviewWitness` 现在负责 branch selection 与 exact snapshot
binding；Memory 持有一个 read guard，PostgreSQL 使用一个 `REPEATABLE READ READ ONLY` transaction。application layer 继续委托
既有 versioned review projection，`GraphDiff::between` 仍是唯一 graph-diff calculator。

Fresh local verification / 新鲜本地验证:

- Memory branch-bound focused tests: `6 passed`.
- PostgreSQL SQL contract: `3 passed, 1 ignored` (runtime remains unobserved without a test database URL).
- API library: `222 passed`; workspace Rust: storage `230 passed, 41 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust `1.85.0` checks passed.
- `pnpm check:web`: public SDK `15`, local SDK `148`, Web `298`, and production build passed.
- `tests/contract/verify-local-contracts.test.ps1` passed; source inspection found one
  production `impl GraphDiff` and `10` `GraphDiff::between` call sites.

- Memory branch-bound focused tests：`6 passed`。
- PostgreSQL SQL contract：`3 passed, 1 ignored`（没有 test database URL 时 runtime 继续未观测）。
- API library：`222 passed`；workspace Rust：storage `230 passed, 41 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0` 检查通过。
- `pnpm check:web`：public SDK `15`、local SDK `148`、Web `298` 与 production build 通过。
- `tests/contract/verify-local-contracts.test.ps1` 通过；源码检查发现一个 production `impl GraphDiff` 与
  `10` 个 `GraphDiff::between` 调用点。

PostgreSQL live runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator
rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred`. No public
REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, or operator transport
was added. The next increment requires a new bilingual Necessity Record. /
PostgreSQL live runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production
继续为 `ignored`、`unobserved` 或 `deferred`。未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret
access 或 operator transport。下一项增量必须新增双语 Necessity Record。
