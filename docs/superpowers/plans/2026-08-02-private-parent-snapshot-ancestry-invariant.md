# Private Parent Graph Snapshot Ancestry Invariant / 私有 Parent Graph Snapshot Ancestry 不变量

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly serves Criteria 1 (Context-first graph coverage), Criterion 2
(replayable version history), and Criterion 4 (Context Graph as the system skeleton). A child
Context commit must not become durable unless every declared parent commit has its immutable graph
snapshot materialized. This is also required by the charter's fail-closed, atomic writer boundary:
commit history and graph history must advance together. / 本增量直接服务条件 1（Context-first graph coverage）、条件 2
（可回放版本历史）与条件 4（Context Graph 作为系统骨架）。只有每个声明的 parent commit 都已有 immutable graph snapshot
时，Context child commit 才能持久化。这也是章程中 fail-closed、atomic writer boundary 的要求：commit history 与 graph history
必须同步推进。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The existing in-memory writer checks that parent commit rows exist, but it does not require a
materialized parent graph snapshot. Its PostgreSQL counterpart queries the same weaker predicate
through `PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL`. A malformed or partially migrated history could
therefore accept a child commit whose ancestry cannot be replayed as a complete Context Graph.
The missing evidence is a red regression for both adapters' contract shape and a green local
atomicity check proving that the child is absent after rejection. / 当前 Memory writer 只检查 parent commit row 存在，没有要求
parent graph snapshot 已 materialize；PostgreSQL 对应的 `PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL` 也使用相同的弱 predicate。因此
malformed 或 partially migrated history 可能接受一个无法 replay 为完整 Context Graph 的 child commit。当前缺口是两个 adapter
的 contract shape 红回归，以及 rejection 后 child 不存在的绿色 atomicity 证据。

### Why now / 为什么现在优先

Commit-associated graph snapshots, backend-owned history witnesses, and version-backed graph
review are already implemented and locally verified. This is the smallest dependency-ready fix
before branch-head and Context lifecycle consumers rely on ancestry completeness. Delaying it
would preserve a known write-path hole while adding more readers. / commit-associated graph snapshot、backend-owned history witness
与 version-backed graph review 已实现并完成本地验证。在 branch-head 与 Context lifecycle consumer 继续依赖 ancestry completeness
之前，这是最小的依赖就绪修复。延后会在增加更多 reader 的同时保留已知写入缺口。

### Explicit non-goals / 明确非目标

- No new public REST, OpenAPI, public SDK, Web mutation, operator transport, migration, provider,
  release, production, or external-evidence claim.
- No branch mutation, merge, rollback, lifecycle editor change, schema-version change, or second
  `GraphDiff` calculator.
- No secret access, Docker startup, PostgreSQL runtime claim, remote CI claim, or production
  connection. Existing valid replay, idempotency, RBAC, rate-limit, audit, and branch-head CAS
  semantics remain unchanged.

- 不新增 public REST、OpenAPI、public SDK、Web mutation、operator transport、migration、provider、release、production 或外部证据声明。
- 不新增 branch mutation、merge、rollback、lifecycle editor 改动、schema version 改动或第二个 `GraphDiff` calculator。
- 不读取 secret，不启动 Docker，不宣称 PostgreSQL runtime、remote CI，不连接 production。既有 replay、idempotency、RBAC、限流、审计
  与 branch-head CAS 语义保持不变。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

The implementation boundary is the private Rust storage writer contract: the in-memory
`persist_commit_snapshot` helper, the PostgreSQL parent-row query, their focused tests, and this
plan plus the bilingual roadmap receipts. No API, SDK, Web, migration, or design-system file is
needed. The error remains the existing structured `ScopeUnavailable` contract so adapters do not
invent divergent public semantics. / 实现边界是 private Rust storage writer contract：Memory 的
`persist_commit_snapshot` helper、PostgreSQL parent-row query、对应 focused tests，以及本计划与双语路线图回执。不需要 API、SDK、Web、
migration 或 design-system 文件。错误继续使用现有结构化 `ScopeUnavailable` contract，避免 adapter 发明分叉的 public semantics。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First observe a red Memory regression in which an existing parent commit without a graph snapshot
is rejected and the child is absent. Then make it green, add a PostgreSQL SQL-shape contract that
requires the parent query to join `context_commit_graph_snapshots`, and run the focused writer and
SQL tests. Before selecting another increment, observe fresh workspace Rust, format, strict
offline Clippy, locked Rust 1.85, Web checks/build, local contract verification, and the exact-one
`GraphDiff` source check. PostgreSQL live runtime, Docker, browser/visual smoke, Git, remote CI,
operator rehearsal, release, and production remain ignored, unobserved, or deferred. /
首先观测 Memory 红回归：已有 parent commit 但无 graph snapshot 时必须拒绝，且 child 不得存在。随后修复为 green，增加要求
parent query join `context_commit_graph_snapshots` 的 PostgreSQL SQL-shape contract，并运行 focused writer 与 SQL tests。选择下一增量
前必须取得 workspace Rust、format、strict offline Clippy、锁定 Rust 1.85、Web checks/build、local contract verification 与 exact-one
`GraphDiff` source check 的新鲜输出。PostgreSQL live runtime、Docker、browser/visual smoke、Git、remote CI、operator rehearsal、release
与 production 继续为 ignored、unobserved 或 deferred。

## Status / 状态

`completed / verified locally`; this plan is a bounded local implementation record and is not a
project completion claim. / `completed / verified locally`；本计划是有界本地实现记录，不构成项目完成声明。

## Completion Receipt / 收束回执

The private commit-snapshot writer now requires every declared parent commit to have a materialized
immutable graph snapshot before accepting a child snapshot. Memory checks the existing parent row
and `snapshot_exists_for_context_commit` while holding its single write guard; PostgreSQL uses the
same admission shape through `PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL`, which joins
`context_commit_graph_snapshots` and locks only `context_commits` with `FOR KEY SHARE`. Rejection
uses the existing structured `ScopeUnavailable` error before child persistence. / 私有 commit-snapshot writer 现在要求每个
声明的 parent commit 都已有 materialized immutable graph snapshot，才接受 child snapshot。Memory 在单一 write guard 内检查
parent row 与 `snapshot_exists_for_context_commit`；PostgreSQL 通过 `PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL` 采用相同准入形状，
join `context_commit_graph_snapshots`，并以 `FOR KEY SHARE` 只锁定 `context_commits`。拒绝继续使用现有结构化
`ScopeUnavailable` error，发生在 child 持久化前。

Fresh red/green evidence / 新鲜红绿证据:

- The new Memory regression failed before the snapshot guard by returning a child snapshot, then
  passed after the guard; writer rejection tests passed `5`.
- The PostgreSQL SQL-shape contract requiring the snapshot join, exact scope predicates, and lock
  clause passed `1`.
- `cargo test --workspace --quiet --offline -j 1` passed with API `222 passed` and storage
  `233 passed, 41 ignored`; `cargo fmt --all -- --check` passed; strict offline Clippy with
  `-D warnings` passed; and `cargo +1.85.0 check --workspace --all-targets --locked --offline`
  passed.
- `pnpm check:web` passed with public SDK `15`, local SDK `148`, Web `298`, and production build;
  `tests/contract/verify-local-contracts.test.ps1` passed; source inspection found one production
  `impl GraphDiff` and `9` current `GraphDiff::between` call sites.

- 新 Memory regression 在 snapshot guard 之前因返回 child snapshot 而失败，接入 guard 后通过；writer rejection tests `5` 项通过。
- 要求 snapshot join、exact scope predicate 与 lock clause 的 PostgreSQL SQL-shape contract `1` 项通过。
- `cargo test --workspace --quiet --offline -j 1` 通过，API `222 passed`、storage `233 passed, 41 ignored`；
  `cargo fmt --all -- --check` 通过；带 `-D warnings` 的 strict offline Clippy 通过；
  `cargo +1.85.0 check --workspace --all-targets --locked --offline` 通过。
- `pnpm check:web` 通过，public SDK `15`、local SDK `148`、Web `298` 与 production build 均通过；
  `tests/contract/verify-local-contracts.test.ps1` 通过；源码检查确认一个 production `impl GraphDiff` 与当前 `9` 个
  `GraphDiff::between` 调用点。

This receipt proves the private Rust writer admission boundary only. It does not prove arbitrary
direct SQL or operational bypasses cannot create malformed history. PostgreSQL runtime requiring
`CONTEXTLAB_TEST_DATABASE_URL`, Docker, authenticated browser/visual smoke, Git, remote CI,
operator rehearsal, release, production, and public protected-write readiness remain
`ignored`, `unobserved`, or `deferred`. No public REST/OpenAPI/SDK write, Web mutation, migration,
provider, secret access, operator transport, or second graph-diff calculator was added. Criteria 1,
2, and 4 are advanced but remain open, the long-term goal remains `active`, and the next increment
requires a fresh bilingual Necessity Record. / 本回执只证明 private Rust writer admission boundary；不证明任意 direct SQL 或运维
绕过无法制造 malformed history。需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime、Docker、authenticated browser/visual
smoke、Git、remote CI、operator rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、
`unobserved` 或 `deferred`。未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret access、operator
transport 或第二个 graph-diff calculator。条件 1、2、4 得到推进但仍开放，长期目标保持 `active`；下一项增量必须先有新的双语
Necessity Record。
