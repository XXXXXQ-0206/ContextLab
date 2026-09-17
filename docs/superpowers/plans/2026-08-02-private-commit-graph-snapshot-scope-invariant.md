# Private Commit Graph Snapshot Scope Invariant / 私有 Commit Graph Snapshot Scope 不变量

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly serves Criteria 1 (Context-first graph coverage), Criterion 2
(replayable version history), and the charter's fail-closed stable-UUID rule. A
`CommitGraphSnapshot` is a commit-associated immutable fact and must not be constructible with a
nil project, Context, or commit identity. / 本增量直接服务条件 1（Context-first graph coverage）、条件 2（可回放版本历史）
以及章程中的 fail-closed stable-UUID 原则。`CommitGraphSnapshot` 是与 commit 关联的 immutable fact，不应允许使用 nil
project、Context 或 commit identity 构造。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The existing scope key is typed, but `CommitGraphSnapshot::new` currently validates only schema and
graph payload. Review adapters reject nil scopes later, while a caller can still construct an
invalid snapshot command before reaching a repository. This leaves the domain boundary weaker than
the already-verified Memory/PostgreSQL and version-backed review contracts. / 现有 scope key 已 typed，但
`CommitGraphSnapshot::new` 当前只校验 schema 与 graph payload。review adapter 会在后续拒绝 nil scope，但调用方仍可在进入
repository 前构造 invalid snapshot command，使 domain boundary 弱于已经验证的 Memory/PostgreSQL 与 version-backed review contract。

### Why now / 为什么现在优先

The commit-associated snapshot domain and repository path are already implemented and connected to
version-backed review. Tightening this constructor is the smallest dependency-ready correction
before another Context consumer relies on the snapshot type; it changes no transport and does not
duplicate graph comparison. / commit-associated snapshot domain 与 repository path 已实现并连接到 version-backed review。
在其他 Context consumer 依赖该 snapshot type 前收紧 constructor，是当前最小的依赖就绪修复；不改变 transport，也不重复 graph comparison。

### Explicit non-goals / 明确非目标

- No new REST, OpenAPI, public SDK, Web mutation, operator transport, migration, provider, or release/production claim.
- No writer redesign, project-ownership policy change, snapshot schema change, or second `GraphDiff` calculator.
- Existing valid UUID, schema-V1, replay, Memory/PostgreSQL, and version-backed review semantics remain unchanged.

- 不新增 REST、OpenAPI、public SDK、Web mutation、operator transport、migration、provider 或 release/production 声明。
- 不重设计 writer，不改变 project-ownership policy、snapshot schema 或新增第二个 `GraphDiff` calculator。
- 现有 valid UUID、schema-V1、replay、Memory/PostgreSQL 与 version-backed review 语义保持不变。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

Only `crates/storage/src/commit_graph_snapshot.rs`, its focused unit tests, this plan, and the
roadmap receipts are in scope. The implementation adds one fail-closed scope invariant and its
domain error; no API/SDK/Web or migration file is needed. / 范围仅包括
`crates/storage/src/commit_graph_snapshot.rs`、其 focused unit tests、本计划与路线图回执。实现只新增一个 fail-closed
scope invariant 及其 domain error；不需要 API/SDK/Web 或 migration 文件。

### Fresh verification before the next increment / 下一增量前的新鲜验证

Observe a red test that constructs nil project, Context, and commit scopes, then make it green.
Run the focused snapshot tests, workspace Rust, `cargo fmt --all -- --check`, strict offline
Clippy, locked Rust `1.85.0` check, `pnpm check:web`, the local contract verifier, and the exact
one `GraphDiff` source check. PostgreSQL live runtime, Docker, browser/visual, Git, remote CI,
operator rehearsal, release, and production remain unobserved or deferred. / 先观测 nil project、Context 与
commit scope 构造 red test，再修复为 green。运行 snapshot focused tests、workspace Rust、`cargo fmt --all -- --check`、strict
offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`、local contract verifier 与 exact-one `GraphDiff` source check。
PostgreSQL live runtime、Docker、browser/visual、Git、remote CI、operator rehearsal、release 与 production 继续为
unobserved 或 deferred。

## Status / 状态

`completed / verified locally`; the long-term goal remains `active`. This plan is not a project
completion claim. / `completed / verified locally`；长期目标保持 `active`。本计划不构成项目完成声明。

## Completion Receipt / 收束回执

`CommitGraphSnapshot::new` now rejects a nil project, Context, or commit UUID with the structured
`CommitGraphSnapshotError::InvalidScope` before graph materialization or persistence. Existing
schema-V1, deterministic payload, replay/conflict, Memory/PostgreSQL, and version-backed review
contracts remain unchanged; `GraphDiff::between` is still the sole graph-diff calculator. /
`CommitGraphSnapshot::new` 现在会在 graph materialization 或 persistence 前，以结构化
`CommitGraphSnapshotError::InvalidScope` 拒绝 nil project、Context 或 commit UUID。既有 schema-V1、确定性 payload、
replay/conflict、Memory/PostgreSQL 与 version-backed review contract 保持不变；`GraphDiff::between` 仍是唯一 graph-diff calculator。

Fresh local verification / 新鲜本地验证:

- Red regression observed: nil-scope test failed before constructor validation; green focused snapshot
  unit tests: `6 passed`.
- Snapshot repository contract: `5 passed`; workspace Rust had no failures, with API `222 passed`
  and storage `230 passed, 41 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust `1.85.0` check passed.
- `pnpm check:web`: public SDK `15`, local SDK `148`, Web `298`, production build passed.
- `tests/contract/verify-local-contracts.test.ps1` passed; source inspection found one production
  `impl GraphDiff` and ten `GraphDiff::between` call sites.

- 已观测红回归：constructor validation 之前 nil-scope test 失败；绿色 focused snapshot unit tests：`6 passed`。
- Snapshot repository contract：`5 passed`；workspace Rust 无失败，其中 API `222 passed`、storage `230 passed, 41 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- `pnpm check:web`：public SDK `15`、local SDK `148`、Web `298`、production build 通过。
- `tests/contract/verify-local-contracts.test.ps1` 通过；源码检查发现一个 production `impl GraphDiff` 与十个
  `GraphDiff::between` 调用点。

PostgreSQL live runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator
rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred`. No public
REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, or operator transport
was added. The next admitted increment is the separately recorded parent-snapshot ancestry
invariant; it requires its own bilingual Necessity Record. / PostgreSQL live runtime、Docker、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。未新增 public REST/OpenAPI/SDK write、
Web mutation、migration、provider、secret access 或 operator transport。下一项准入增量是单独记录的 parent-snapshot ancestry
invariant，必须拥有自己的双语 Necessity Record。
