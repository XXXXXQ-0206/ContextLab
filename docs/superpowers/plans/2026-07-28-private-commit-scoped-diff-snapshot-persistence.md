# Private Commit-Scoped Diff Snapshot Persistence / 私有按 Commit 绑定的 Diff Snapshot 持久化

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criterion 2 (replayable version history) and Criterion 4 (Context Graph and reviewability as a
system backbone). Semantic, behavior, and evaluation review inputs must be reproducible facts of
one immutable Context commit, not caller-supplied data that merely carries a matching label. The
work also follows the charter's Context-first, reusable Rust core, stable contract, and clean
architecture principles.

本增量直接推进条件 2（可回放版本历史）与条件 4（Context Graph 与 reviewability 作为系统骨架）。Semantic、
behavior 与 evaluation review input 必须是一个不可变 Context commit 的可重现事实，而不能只是带有匹配标签的
调用方输入。本工作同时遵循章程中的 Context-first、可复用 Rust core、稳定契约与 clean architecture 原则。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The diff engine already
validates `ContextDiffSnapshotV1`, `ContextDiffService` is the unified comparison path, and the
versioned review contract binds each side to `(ProjectId, ContextId, CommitId)`. The remaining
gap is storage: no immutable repository binds the complete semantic/behavior/evaluation snapshot
payload and its schema/digest to that exact commit. Without it, a future version-backed review can
still compare caller-provided or mixed-scope snapshots.

diff engine 已校验 `ContextDiffSnapshotV1`，`ContextDiffService` 是统一 comparison path，versioned review contract
也已将每一侧绑定到 `(ProjectId, ContextId, CommitId)`。剩余缺口在 storage：尚无不可变 repository 将完整的
semantic/behavior/evaluation snapshot payload 及其 schema/digest 绑定到该 exact commit。缺少它，未来
version-backed review 仍可能比较调用方注入或跨 scope 混合的 snapshot。

**Why now / 为什么现在优先:** The commit-associated ContextGraph snapshot contract and protected-local
graph-diff read boundary are green, and the adjacent typed review contract is already tested. The
next dependency-ready local gap is the persistence bridge that makes semantic/behavior/evaluation
review replayable before any private adapter or UI consumes it. This closes a named versioning/diff
criterion gap without expanding a public surface.

commit-associated ContextGraph snapshot contract 与 protected-local graph-diff read boundary 已通过验证，邻近的
typed review contract 也已有测试。当前下一项依赖就绪的本地缺口，是让 semantic/behavior/evaluation review 在任何
private adapter 或 UI 消费前具备可回放 persistence bridge。本增量直接收束 versioning/diff 条件缺口，且不扩大
public surface。

**Minimal affected boundary / 最小受影响边界:** Add one storage-owned immutable projection keyed by
exact `(ProjectId, ContextId, CommitId, schema_version)` and carrying the validated V1 snapshot,
deterministic digest, and capture metadata. Implement one repository port with Memory and existing
PostgreSQL parity, fail-closed exact-scope reads, immutable create/replay/conflict semantics, and
focused migration/contract tests. Reuse the existing diff-engine DTO and validation; do not add an
application comparison path in this first wave.

新增一份 storage-owned immutable projection，以 exact `(ProjectId, ContextId, CommitId, schema_version)` 为 key，
保存已校验的 V1 snapshot、确定性 digest 与 capture metadata。实现一份 repository port，并提供 Memory 与现有
PostgreSQL parity、exact-scope fail-closed read、immutable create/replay/conflict 语义以及聚焦 migration/contract
test。第一波复用既有 diff-engine DTO 与 validation，不新增 application comparison path。

**Explicit non-goals / 明确非目标:** No public REST route, public OpenAPI/SDK method, Web mutation,
operator transport, provider/evaluator call, raw private-content read path, graph editing,
branch/merge/rollback, Docker/runtime setup, release, production claim, or second graph-diff
calculator. `GraphDiff::between` remains the sole graph-diff calculator; a later private read adapter
must delegate to `ContextDiffService::compare`.

不新增 public REST route、public OpenAPI/SDK method、Web mutation、operator transport、provider/evaluator call、
raw private-content read path、graph editing、branch/merge/rollback、Docker/runtime setup、release、production claim
或第二个 graph-diff calculator。`GraphDiff::between` 仍是唯一 graph-diff calculator；后续 private read adapter 必须
委托 `ContextDiffService::compare`。

**Ownership and bilingual documentation / 所有权与双语文档:** The first wave owns only
`crates/storage` snapshot projection/repository modules, migration `0023`, and focused storage
tests. The Integration Lead owns this plan and the roadmap ledgers. `crates/diff-engine`,
`server/api`, `packages/*`, `apps/*`, OpenAPI, and public contracts are out of scope until the
storage contract is green and separately admitted.

第一波只拥有 `crates/storage` snapshot projection/repository module、migration `0023` 与聚焦 storage test。
Integration Lead 负责本计划与 roadmap ledger。`crates/diff-engine`、`server/api`、`packages/*`、`apps/*`、
OpenAPI 与 public contract 在 storage contract 全绿且单独准入前均不在范围内。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Focused
Memory tests must prove exact scope round-trip, immutable replay, changed-payload conflict, missing
scope, deterministic digest, and schema rejection. PostgreSQL contract tests must prove matching
composite scope, append-only behavior, and parity with Memory when the local runtime is available.
Then run `cargo fmt --all -- --check`, workspace Rust tests, strict offline Clippy, locked Rust
`1.85.0` check, and `pnpm check:web`; record PostgreSQL, browser, Git, remote, operator, release,
and production only when observed.

聚焦 Memory test 必须证明 exact scope round-trip、immutable replay、changed-payload conflict、missing scope、确定性
digest 与 schema rejection。PostgreSQL contract test 在本地 runtime 可用时必须证明 matching composite scope、
append-only behavior 与 Memory parity。随后运行 `cargo fmt --all -- --check`、workspace Rust test、strict offline
Clippy、锁定 Rust `1.85.0` check 与 `pnpm check:web`；PostgreSQL、browser、Git、remote、operator、release 与
production 只在实际观测时记录。

## Implementation Steps / 实施步骤

- [x] Add the immutable exact-commit projection and repository port with Memory parity.
- [x] Add the PostgreSQL migration/adapter and fail-closed scope constraints.
- [x] Add red/green tests for replay, conflict, missing scope, schema, digest, and append-only behavior.
- [x] Run fresh focused and workspace verification; record only observed evidence.

- [x] 增加 immutable exact-commit projection 与 repository port，并提供 Memory parity。
- [x] 增加 PostgreSQL migration/adapter 与 fail-closed scope constraint。
- [x] 增加 replay、conflict、missing scope、schema、digest 与 append-only behavior 的红绿测试。
- [x] 运行新鲜 focused 与 workspace verification，并只记录已观测 evidence。

## Boundary at Admission / 准入时边界

This plan is admitted for local core development only. It does not authorize a version-backed
semantic/behavior/evaluation API, SDK, Web inspector, or public promotion. Those decisions require
a separate receipt after this repository contract is green.

本计划仅准入本地核心开发，不授权 version-backed semantic/behavior/evaluation API、SDK、Web inspector 或 public
promotion。这些决策必须等待本 repository contract 通过后，以独立回执再次准入。

## Implementation and Fresh Verification Receipt / 实现与新鲜验证回执

The storage contract is implemented without changing the diff algorithm or any transport surface.
`ContextDiffSnapshotV1Record` binds the validated `ContextDiffSnapshotV1` to an exact
`(ProjectId, ContextId, CommitId, schema_version)` through the existing
`VersionedContextScopeV1`. It stores a deterministic `sha256:` digest and microsecond-normalized
capture time. The Memory and PostgreSQL adapters share immutable create/replay/conflict semantics;
reads validate the exact scope, schema, payload, and digest before returning a record. Migration
`0023_context_diff_snapshots.sql` adds composite foreign keys, a fixed schema check, and an
append-only trigger.

该 storage contract 已完成实现，没有改变 diff algorithm 或任何 transport surface。`ContextDiffSnapshotV1Record`
通过既有 `VersionedContextScopeV1` 将已校验的 `ContextDiffSnapshotV1` 绑定到 exact
`(ProjectId, ContextId, CommitId, schema_version)`。它保存确定性的 `sha256:` digest 与规范化到微秒的 capture time。
Memory 与 PostgreSQL adapter 共享 immutable create/replay/conflict 语义；read 在返回 record 前校验 exact scope、
schema、payload 与 digest。migration `0023_context_diff_snapshots.sql` 增加 composite foreign key、固定 schema check
与 append-only trigger。

Fresh local evidence passed: `cargo test -p contextlab-storage --test context_diff_snapshot_repository --quiet`
(`4 passed`); migration contract (`2 passed`); PostgreSQL adapter contract (`1 passed, 1 ignored`, with the
runtime case ignored because no disposable PostgreSQL service was started); `cargo fmt --all -- --check`;
`cargo test --workspace --quiet` with API `183 passed` and storage `179 passed, 39 ignored`;
`cargo clippy --workspace --all-targets --offline -- -D warnings`; `cargo +1.85.0 check --workspace --all-targets --locked`;
and `pnpm check:web` with public SDK `15`, local SDK `92`, Web `185`, and a successful production build.

新鲜本地证据均已通过：`cargo test -p contextlab-storage --test context_diff_snapshot_repository --quiet`（`4 passed`）；
migration contract（`2 passed`）；PostgreSQL adapter contract（`1 passed, 1 ignored`，runtime case 因未启动 disposable
PostgreSQL service 而 ignored）；`cargo fmt --all -- --check`；`cargo test --workspace --quiet`（API `183 passed`、
storage `179 passed, 39 ignored`）；`cargo clippy --workspace --all-targets --offline -- -D warnings`；
`cargo +1.85.0 check --workspace --all-targets --locked`；以及 `pnpm check:web`（public SDK `15`、local SDK `92`、
Web `185`，production build 成功）。

PostgreSQL runtime, authenticated browser, Git binding/change-set, remote CI, operator rehearsal, release, and
production evidence remain `unobserved` or `deferred`. No secrets were read, no Docker was started, no provider was
called, no public REST/OpenAPI/SDK/Web mutation was added, and `GraphDiff::between` remains the sole graph-diff
calculator. The storage contract advances Criteria 2 and 4 but closes neither; the long-term goal remains active.

PostgreSQL runtime、authenticated browser、Git binding/change-set、remote CI、operator rehearsal、release 与 production
evidence 仍为 `unobserved` 或 `deferred`。未读取 secrets、未启动 Docker、未调用 provider、未新增 public
REST/OpenAPI/SDK/Web mutation，且 `GraphDiff::between` 仍是唯一 graph-diff calculator。本 storage contract 推进条件
2 与 4，但不关闭其中任何一项；长期目标保持 active。
