# Commit-Associated Context Graph Snapshots / 与 Commit 关联的 Context Graph Snapshot

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criterion 1 (Context-first graph coverage), Criterion 2 (replayable version history), and Criterion
4 (semantic reviewability). A graph snapshot must be a durable, exact-commit fact rather than a
preview or mutable current-head projection before version-backed graph comparison can be trusted.

本增量直接推进条件 1（Context-first graph coverage）、条件 2（可回放版本历史）与条件 4（可进行 semantic
review）。在可信地接入 version-backed graph comparison 前，graph snapshot 必须是持久化且精确绑定 commit
的事实，而不是 preview 或可变 current-head projection。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** Existing
`CommitGraphSnapshot` and `ContextGraphProjectionRepository` contracts can read a snapshot, while
`GraphDiff::between` can compare two graphs, but their exact project/Context/commit binding and
repository error behavior are not yet a single reusable domain/storage contract. A future
comparison route must not reconstruct graphs from mutable preview state or calculate a second diff.

现有 `CommitGraphSnapshot` 与 `ContextGraphProjectionRepository` contract 已能读取 snapshot，
`GraphDiff::between` 也能比较两个 graph，但 exact project/Context/commit binding 与 repository error
语义尚未形成一份可复用的统一 domain/storage contract。未来 comparison route 不得从可变 preview state
重建 graph，也不得计算第二套 diff。

**Why now / 为什么现在优先:** The exact-commit Knowledge/Memory persistence boundary is now
green and establishes the required storage/replay discipline. The next directly dependent local
gap is the graph counterpart: bind materialized graph facts to immutable commits before exposing
any version-backed comparison adapter.

exact-commit Knowledge/Memory persistence boundary 已通过验证并建立所需 storage/replay discipline。下一个
直接依赖且服务于核心平台的本地缺口是 graph counterpart：在接入任何 version-backed comparison adapter 前，
先将 materialized graph facts 绑定到不可变 commit。

**Minimal boundary / 最小边界:** Define or tighten one Rust domain/storage contract for a redacted,
immutable snapshot keyed by exact project, Context, and commit; implement Memory parity and the
existing PostgreSQL repository path; add focused scope, replay, missing-source, and conflict tests.
Only after that contract is green may the existing version-backed diff application path consume it.

定义或收紧一份 Rust domain/storage contract，以 exact project、Context、commit 为 key 保存脱敏且不可变的
snapshot；实现 Memory parity 与现有 PostgreSQL repository path；补齐 scope、replay、missing-source 与
conflict focused test。只有该 contract 全绿后，现有 version-backed diff application path 才可消费它。

**Explicit non-goals / 明确非目标:** No new public write route, no public OpenAPI/SDK method, no Web
mutation or new UI, no graph editing, no branch/merge mutation, no provider call, no raw private
content, no new GraphDiff calculator, no Docker/production claim, and no external release evidence.
`GraphDiff::between` remains the sole graph-diff calculator.

不新增 public write route、public OpenAPI/SDK method、Web mutation 或新 UI；不做 graph editing、branch/merge
mutation、provider call、raw private content、第二个 GraphDiff calculator、Docker/production claim 或外部
release evidence。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Expected bilingual documentation / 预期双语文档:** Update this plan, the active long-term goal,
completion criteria, and parallel-development ledger with the contract, evidence, and next pointer.

更新本计划、active long-term goal、completion criteria 与 parallel-development ledger，记录 contract、证据与下一指针。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Focused Rust
tests must prove exact scope binding, immutable replay/conflict semantics, deterministic ordering,
missing snapshot fail-closed behavior, and one `GraphDiff::between` integration assertion. Then run
`cargo fmt --all -- --check`, workspace Rust tests, strict offline Clippy, and `pnpm check:web`.
PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and
production remain separately labeled by observed evidence.

聚焦 Rust test 必须证明 exact scope binding、immutable replay/conflict 语义、确定性排序、missing snapshot
fail-closed 行为，以及一条 `GraphDiff::between` integration assertion。随后运行 `cargo fmt --all -- --check`、
workspace Rust test、strict offline Clippy 与 `pnpm check:web`。PostgreSQL runtime、authenticated browser、Git、
remote CI、operator rehearsal、release 与 production 仍按真实观测单独标记。

## Ownership / 所有权

- Rust domain/storage: existing `crates/storage` snapshot/repository modules and focused tests only.
- Version-backed comparison: only after the storage contract is green, in the existing diff
  application boundary; no new calculator or public transport.
- Documentation: this plan plus the four roadmap ledgers; no SDK/Web changes in the first wave.

- Rust domain/storage：仅限现有 `crates/storage` snapshot/repository module 与聚焦测试。
- Version-backed comparison：storage contract 全绿后，才在现有 diff application boundary 接入；不新增
  calculator 或 public transport。
- 文档：本计划及四份 roadmap ledger；第一波不修改 SDK/Web。

## Implementation and verification receipt / 实现与验证回执

The contract is now implemented. `CommitGraphSnapshotScope` binds a snapshot to the exact
`(ProjectId, ContextId, CommitId)` tuple. `CommitGraphSnapshotRepository` resolves a durable
`ProjectId` from Context ownership for guarded creation, resolves an existing commit scope for
reads, and reads immutable snapshots by that typed key. Memory and PostgreSQL adapters preserve the
same fail-closed unknown-scope behavior; the V1 JSON shape flattens the typed scope to preserve the
existing response fields while adding `project_id`.

该 contract 已完成实现。`CommitGraphSnapshotScope` 将 snapshot 精确绑定到
`(ProjectId, ContextId, CommitId)` tuple。`CommitGraphSnapshotRepository` 会在 guarded creation 时从 Context
ownership 解析 durable `ProjectId`，在 read 时解析既有 commit scope，并按 typed key 读取不可变 snapshot。
Memory 与 PostgreSQL adapter 保持相同的 unknown-scope fail-closed 语义；V1 JSON shape 将 typed scope flatten，
从而保留既有 response field，同时增加 `project_id`。

The existing API graph-diff route now parses typed UUIDs, resolves both exact scopes server-side,
loads both snapshots, and delegates comparison only to `GraphDiff::between`. The existing guarded
commit route obtains project ownership from the same repository boundary before constructing its
snapshot command. No route path, public OpenAPI/SDK method, Web mutation, or second diff calculator
was added.

现有 API graph-diff route 现会解析 typed UUID，在服务端解析两个 exact scope，读取两个 snapshot，并且只委托
`GraphDiff::between` 完成比较。现有 guarded commit route 会在构造 snapshot command 前从同一 repository boundary
取得 project ownership。没有新增 route path、public OpenAPI/SDK method、Web mutation 或第二个 diff calculator。

Fresh local receipts: `cargo fmt --all -- --check`; `cargo test --workspace --quiet` with API
`184 passed` and storage `179 passed, 39 ignored`; `cargo clippy --workspace --all-targets --offline -- -D warnings`;
the API typed-scope integration test `2 passed`; the storage snapshot repository contract `5 passed`;
auth `44 passed`; and `pnpm check:web` including public SDK, local SDK, Web tests, and production build.
PostgreSQL runtime, `psql`, Docker/virtualization, authenticated browser runtime, Git change-set,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

新鲜本地回执包括：`cargo fmt --all -- --check`；`cargo test --workspace --quiet`（API `184 passed`、storage
`179 passed, 39 ignored`）；`cargo clippy --workspace --all-targets --offline -- -D warnings`；API typed-scope
integration test `2 passed`；storage snapshot repository contract `5 passed`；auth `44 passed`；以及包含 public SDK、
local SDK、Web test 与 production build 的 `pnpm check:web`。PostgreSQL runtime、`psql`、Docker/virtualization、
authenticated browser runtime、Git change-set、remote CI、operator rehearsal、release 与 production 仍为
`unobserved` 或 `deferred`。

This receipt advances the Context-first graph, replayable history, and semantic review criteria but
does not close the long-term goal. The next admitted work requires a new bilingual Necessity Record;
the current pointer is a private semantic/behavior/evaluation diff contract, with no public write or
production claim authorized.

本回执推进 Context-first graph、可回放历史与 semantic review 条件，但不关闭长期目标。下一项工作必须先新增双语
Necessity Record；当前指针为 private semantic/behavior/evaluation diff contract，未授权 public write 或 production
声明。
