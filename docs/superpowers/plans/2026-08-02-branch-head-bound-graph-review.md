# Branch Head to Graph Review / Branch Head 到 Graph Review

## Necessity Record / 必要性记录

**Primary criterion / 主要条件:** Criterion 2 - replayable version history. A server-owned
branch head must be a deterministic, typed revision candidate rather than a caller-invented
commit pair. This increment advances the criterion but does not close it. / 条件 2 - 可回放版本历史：
server-owned branch head 必须是确定性、typed 的 revision candidate，而不是调用方臆造的 commit pair。本增量推进
该条件，但不关闭它。

**Secondary criterion / 次要条件:** Criterion 4 - Context Graph as the system backbone. The
existing version-backed graph review should be able to derive its revised side from the validated
history witness while preserving exact project/Context scope and the sole `GraphDiff::between` path. /
条件 4 - Context Graph 作为系统骨架：现有 version-backed graph review 应能从 validated history witness 推导
revised side，同时保留 exact project/Context scope 与唯一 `GraphDiff::between` 路径。

**Gap / 缺口:** The preceding increment loads complete `CommitHistory` and explicit branch heads,
but its review entry point still accepts two caller-supplied commit scopes. The branch selection
policy is therefore not yet represented by a reusable Rust contract. / 前一增量已加载完整 `CommitHistory` 与显式
branch heads，但 review entry point 仍接受调用方提供的两个 commit scope，因此 branch selection policy 尚未由可复用
Rust contract 表达。

**Why now / 为什么现在优先:** The required history and graph-review contracts are now locally
verified and already meet at one storage service. Adding the branch-head selection method is the
smallest direct closure of the remaining named item before any transport or mutation work. / 现在优先是因为：所需
history 与 graph-review contract 已在本地验证，并已汇合到一个 storage service；在任何 transport 或 mutation 之前，
增加 branch-head selection method 是直接收束剩余命名项的最小增量。

**Explicit non-goals / 明确非目标:** No new public REST/OpenAPI/SDK route or method, no Web mutation,
no branch mutation, merge, rollback, migration, provider, Docker/PostgreSQL runtime claim,
authenticated-browser claim, secret access, operator transport, release/production claim, or
second graph-diff calculator. / 不新增 public REST/OpenAPI/SDK route 或 method、Web mutation、branch
mutation、merge、rollback、migration、provider、Docker/PostgreSQL runtime 声明、authenticated-browser 声明、
secret access、operator transport、release/production 声明或第二个 graph-diff calculator。

**Minimal ownership / 最小 ownership:** `crates/storage/src/context_graph_history_review.rs`, the
storage scope-validation helper in `crates/storage/src/context_graph_diff_review.rs`, the existing
safe error mapping in `server/api/src/routes.rs`, and focused tests only. Reuse
`CommitHistory::head`, `BranchName`, exact `CommitGraphSnapshotScope`, and
`PersistedContextGraphDiffReviewService`; do not move policy into SDK, Web, or versioning
persistence. / 最小 ownership 为 `context_graph_history_review.rs`、`context_graph_diff_review.rs` 的 storage scope-validation
helper、`server/api/src/routes.rs` 中既有的安全 error mapping 与 focused tests。复用上述既有 contract，不把 policy 移入
SDK、Web 或 versioning persistence。

**Fresh verification before the next increment / 下一增量前的新鲜验证:** Focused branch-head
selection tests, storage history/review tests, API crate tests, workspace Rust, format, strict
offline Clippy, locked Rust `1.85.0`, Web/SDK checks, local contract fixture, and the exact-one
`GraphDiff` source check. For this branch-head increment, PostgreSQL/Docker runtime, browser, Git,
remote, operator, release, and production evidence remain unobserved or deferred. / 下一增量前需取得 focused
branch-head selection、storage history/review、API、workspace Rust、format、strict offline Clippy、锁定 Rust `1.85.0`、
Web/SDK、local contract fixture 与 exact-one `GraphDiff` source check 的新鲜结果。对于本 branch-head 增量，
PostgreSQL/Docker runtime、browser、Git、remote、operator、release 与 production evidence 仍为 unobserved 或 deferred。

## Evidence Boundary / 证据边界

The branch-head method is a private read-only domain/storage adapter. A passing local test proves
only deterministic selection and delegation; it does not prove database snapshot atomicity or
release readiness.

branch-head method 是 private、read-only domain/storage adapter。local test 通过只证明确定性 selection 与 delegation；
不证明 database snapshot atomicity 或 release readiness。

## Implementation and verification receipt / 实施与验证回执

The implementation is complete for this bounded increment. `review_branch_head` now validates the
source snapshot scope before reading the history repository; the loaded `CommitHistory` must also
own the same Context as both requested snapshot scopes. The API maps the new fail-closed history
scope error to the existing unavailable response, without changing any transport contract. / 本有界增量的实现已完成。
`review_branch_head` 现在会在读取 history repository 前校验 source snapshot scope；加载的 `CommitHistory` 还必须与两侧
requested snapshot scope 属于同一 Context。API 将新增的 fail-closed history scope error 映射到既有 unavailable response，
未改变任何 transport contract。

The red-to-green focused suite is `7 passed`. API graph-diff focused tests are `13 passed`; full
workspace Rust passed with API `223 passed` and storage `231 passed, 41 ignored`; format, strict
offline Clippy, locked Rust `1.85.0`, the local contract verifier, Web/SDK checks (`15` public SDK,
`148` local SDK, `298` Web tests, production build), and `GRAPH_DIFF_IMPL_COUNT=1` / call count `10`
passed. / 红到绿 focused suite 为 `7 passed`。API graph-diff focused tests 为 `13 passed`；完整 workspace Rust 通过，
其中 API `223 passed`、storage `231 passed, 41 ignored`；format、strict offline Clippy、锁定 Rust `1.85.0`、local contract
verifier、Web/SDK checks（public SDK `15`、local SDK `148`、Web tests `298`、production build）以及
`GRAPH_DIFF_IMPL_COUNT=1` / call count `10` 均通过。

PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, and production remain unobserved or deferred for this increment. The long-term goal remains
active; atomic multi-port reads and any future public promotion still require separate evidence and
their own bilingual Necessity Record. / 对本增量而言，PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release 与 production 仍为 unobserved 或 deferred。长期目标保持 active；atomic multi-port
reads 与未来任何 public promotion 仍需独立证据及各自的双语 Necessity Record。
