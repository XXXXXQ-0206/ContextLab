# Private Context Graph Review Witness / 私有 Context Graph 审查见证

## Scope / 范围

`ContextGraphReviewWitnessRepository` is a private Rust storage contract for the local protected
version-backed graph-diff read. It returns one validated aggregate containing the complete
`CommitHistory` (including branch heads) and the exact source and target
`CommitGraphSnapshot` values. Its scope is `(ProjectId, ContextId, sourceCommitId, targetCommitId)`;
nil, mixed, repeated, missing, or schema-drifted scopes fail closed.

`ContextGraphReviewWitnessRepository` 是本地 protected version-backed graph-diff read 使用的 private Rust storage contract。
它返回一个经过校验的 aggregate，包含完整 `CommitHistory`（包括 branch heads）以及 exact source/target
`CommitGraphSnapshot`。范围是 `(ProjectId, ContextId, sourceCommitId, targetCommitId)`；nil、混合、重复、缺失或 schema drift
scope 均 fail closed。

## Consistency / 一致性

- Memory collects commit records, branch heads, and both snapshots under one `RwLock` read guard.
- PostgreSQL collects history rows and both exact snapshot rows under one `REPEATABLE READ READ ONLY` transaction.
- The repository reuses `CommitHistory::try_from_parts` and `CommitGraphSnapshot` validation; it does not calculate a diff.

- Memory 在同一个 `RwLock` read guard 下收集 commit record、branch head 与两份 snapshot。
- PostgreSQL 在同一个 `REPEATABLE READ READ ONLY` transaction 下读取 history row 与两份 exact snapshot row。
- repository 复用 `CommitHistory::try_from_parts` 与 `CommitGraphSnapshot` validation；不计算 Diff。

## Review Ownership / 审阅所有权

The witness is read first. `PersistedContextGraphWitnessReviewService` then delegates the snapshot
pair to the existing `VersionedContextGraphDiffReviewService`, where the sole `GraphDiff::between`
implementation remains owned by `contextlab-diff-engine`. API, local SDK, BFF, and Web continue to
adapt or present the existing response and do not expose the complete history or recompute Diff.

先读取 witness，再由 `PersistedContextGraphWitnessReviewService` 将 snapshot pair 委托给既有
`VersionedContextGraphDiffReviewService`；唯一的 `GraphDiff::between` implementation 仍由 `contextlab-diff-engine` 所有。
API、local SDK、BFF 与 Web 继续适配或呈现既有 response，不暴露完整 history，也不重新计算 Diff。

The environment-backed API state injects this witness repository into the existing protected local
GET. The route fails closed when the dependency is absent. The older split-port adapter remains
available only to direct unit/custom-fixture composition and is not a route fallback or an
environment-backed runtime contract.

environment-backed API state 将 witness repository 注入既有 protected local GET。依赖缺失时 route 直接 fail closed。旧
split-port adapter 仅保留给直接 unit/custom-fixture 组合，不是 route fallback，也不是 environment-backed runtime contract。

## Evidence Boundary / 证据边界

The local Memory tests, compile checks, and PostgreSQL SQL-shape tests do not prove a live PostgreSQL
runtime. Docker, `CONTEXTLAB_TEST_DATABASE_URL`, authenticated browser/visual smoke, Git, remote CI,
operator rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred`.

本地 Memory tests、compile checks 与 PostgreSQL SQL-shape tests 不证明 live PostgreSQL runtime。Docker、
`CONTEXTLAB_TEST_DATABASE_URL`、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与
production 继续为 `ignored`、`unobserved` 或 `deferred`。

No public REST/OpenAPI/public SDK method, write route, Web mutation, migration, provider, secret
access, operator transport, or second graph-diff calculator is added by this boundary.

本边界未新增 public REST/OpenAPI/public SDK method、写入 route、Web mutation、migration、provider、secret access、
operator transport 或第二个 graph-diff calculator。

## Atomic Merge Review Witness / 原子 Merge Review Witness

The private `ContextMergeReviewWitnessRepository` resolves the server-owned `MergePlan` and
loads its immutable base/left/right `CommitGraphSnapshot` values from one backend-owned
observation. Memory holds one `RwLock` read guard for the complete operation. PostgreSQL uses one
`REPEATABLE READ READ ONLY` transaction, constructs and validates the witness before committing,
and then returns the validated aggregate. The service rechecks that the returned witness matches
the requested project, Context, and left/right tips; a faulty repository cannot substitute another
scope. / 私有 `ContextMergeReviewWitnessRepository` 在一个 backend-owned observation 中解析 server-owned `MergePlan`，并读取其
immutable base/left/right `CommitGraphSnapshot`。Memory 在整个操作期间持有一个 `RwLock` read guard。PostgreSQL 使用单一
`REPEATABLE READ READ ONLY` transaction，在 commit 前构造并校验 witness，再返回已验证 aggregate。service 还会重新校验返回的 witness
是否匹配请求的 project、Context 与 left/right tips；错误 repository 不能替换为其他 scope。

`AppState` does not fall back to preview data when the private witness dependency is absent. The
protected route returns the existing safe unavailable response before repository access. The
application delegates classification to the existing versioned review path; `GraphDiff::between`
remains the sole graph-diff calculator. / 当 private witness dependency 缺失时，`AppState` 不回退到 preview data。protected route 在访问
repository 前返回既有安全 unavailable response。application 继续委托既有 versioned review path；`GraphDiff::between` 仍是唯一
graph-diff calculator。

This is local contract evidence, not live PostgreSQL evidence. No public REST/OpenAPI/public SDK method,
write route, Web mutation, migration, provider, secret access, operator transport, release, or
production claim is added. / 这是本地 contract evidence，不是 live PostgreSQL evidence。未新增 public REST/OpenAPI/public SDK method、write
route、Web mutation、migration、provider、secret access、operator transport、release 或 production 声明。
