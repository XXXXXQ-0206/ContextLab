# Private Context Graph Review / 私有 Context Graph 审阅

## Boundary / 边界

The local private review path is layered as `persisted snapshots -> storage batch port ->
versioned graph review contract -> GraphMergeConflictClassifier`. A review is identified by one
`ProjectId`, one `ContextId`, and distinct base/left/right `CommitId` values. The storage boundary
returns the three immutable snapshots in that fixed order and rejects missing records or scope
drift before the application projection is built.

本地私有 review path 分层为 `persisted snapshots -> storage batch port -> versioned graph review contract ->
GraphMergeConflictClassifier`。一次 review 由一个 `ProjectId`、一个 `ContextId` 与三个不同的 base/left/right
`CommitId` 标识。storage boundary 以固定顺序返回三份 immutable snapshot，并在构建 application projection 前拒绝
missing record 或 scope drift。

## Consistency / 一致性

The in-memory adapter reads the complete set under one read guard. The PostgreSQL adapter reads the
same set inside one `REPEATABLE READ READ ONLY` transaction. Existing repository doubles keep the
compatible default batch implementation, which preserves the single-snapshot port for unrelated
readers. This is a locally verified contract; it is not PostgreSQL runtime, production, or release
evidence.

Memory adapter 在一个 read guard 下读取完整集合。PostgreSQL adapter 在一个 `REPEATABLE READ READ ONLY` transaction
内读取相同集合。既有 repository double 通过兼容的 default batch implementation 保持可用，其他 reader 的
single-snapshot port 不受影响。这是已完成本地验证的 contract，不是 PostgreSQL runtime、production 或 release evidence。

## Diff Ownership / Diff 所有权

`PersistedContextGraphMergeReviewService` invokes the batch port once and delegates the versioned
request to `GraphMergeConflictClassifier`. Only that classifier calls `GraphDiff::between`; storage,
API, SDK, and Web do not calculate graph differences. The review is read-only and produces no
merged graph, branch mutation, rollback, public route, OpenAPI/SDK method, or Web mutation.

`PersistedContextGraphMergeReviewService` 只调用一次 batch port，并将 versioned request 委托给
`GraphMergeConflictClassifier`。只有该 classifier 调用 `GraphDiff::between`；storage、API、SDK 与 Web 不计算 graph
差异。该 review 是只读的，不生成 merged graph，不执行 branch mutation 或 rollback，也不新增 public route、OpenAPI/SDK
method 或 Web mutation。

## Evidence Boundary / 证据边界

Fresh local evidence includes focused storage `8 passed`, workspace API `183 passed`, storage
`186 passed, 39 ignored`, format, strict offline Clippy, locked Rust `1.85.0`, public SDK `15`,
local SDK `92`, Web `188`, and the production Web build. PostgreSQL runtime, authenticated browser,
Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`. No secrets were read.

新鲜本地证据包括 focused storage `8 passed`、workspace API `183 passed`、storage `186 passed, 39 ignored`、format、
strict offline Clippy、锁定 Rust `1.85.0`、public SDK `15`、local SDK `92`、Web `188` 与 production Web build。
PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production 仍为
`unobserved` 或 `deferred`。未读取 secrets。

## Two-Way Versioned Graph Review / 两路版本化 Graph Review

The private two-way review path is layered as `exact commit scopes ->
PersistedContextGraphDiffReviewService -> VersionedContextGraphDiffReviewService ->
GraphDiff::between`. The storage adapter reads exactly two immutable
`CommitGraphSnapshot` records, preserves their capture metadata, rejects nil, identical,
cross-Context, missing, out-of-scope, and unsupported-schema records, and delegates the graph
comparison to the reusable diff-engine contract. The API handler retains authentication,
authorization, rate limiting, exact commit scope resolution, and response mapping only; it no
longer owns comparison orchestration.

私有两路 review path 分层为 `exact commit scopes -> PersistedContextGraphDiffReviewService ->
VersionedContextGraphDiffReviewService -> GraphDiff::between`。storage adapter 只读取两份 immutable
`CommitGraphSnapshot`，保留 capture metadata，并拒绝 nil、identical、跨 Context、missing、越界与不支持的
schema record，再将 graph comparison 委托给可复用的 diff-engine contract。API handler 只保留 authentication、
authorization、rate limit、exact commit scope resolution 与 response mapping，不再拥有 comparison orchestration。

`VersionedContextGraphDiffReviewRequestV1` is a provider-free, schema-versioned pair contract.
Its scope validation is the single source for nil, mixed-scope, and self-comparison policy;
storage maps errors without reimplementing the policy. `VersionedContextGraphDiffReviewService`
is the only new application boundary and calls the existing `GraphDiff::between`; no second
algorithm or persisted diff calculation exists. The route path, response DTO, OpenAPI, public
SDK, local SDK, Web, and mutation boundaries are unchanged.

`VersionedContextGraphDiffReviewRequestV1` 是 provider-free、带 schema version 的 pair contract。其 scope validation
是 nil、mixed-scope 与 self-comparison policy 的唯一来源；storage 只做错误映射，不重新实现 policy。
`VersionedContextGraphDiffReviewService` 是唯一新增的 application boundary，并调用既有 `GraphDiff::between`；
不存在第二套 algorithm 或 persisted diff calculation。route path、response DTO、OpenAPI、public SDK、local SDK、
Web 与 mutation boundary 均未改变。

Fresh local evidence: focused diff-engine `3 passed`, storage `3 passed`, protected API graph-diff
`6 passed`, full workspace Rust, strict offline Clippy, locked Rust `1.85.0`, format, and Web
`15/99/207 + production build` passed; static `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL/Docker runtime,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`.

新鲜本地证据：focused diff-engine `3 passed`、storage `3 passed`、protected API graph-diff `6 passed`、
workspace Rust、strict offline Clippy、锁定 Rust `1.85.0`、format 与 Web `15/99/207 + production build` 均通过；
static `GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`。
