# Private Deterministic Context Commit Replay / 私有确定性 Context Commit 回放

## Necessity Record / 必要性记录

**Criterion and charter principle / 完成条件与章程原则:** This increment directly advances Criterion 2,
the replayable version-history requirement, and Criterion 1's Context-first model. `ContextCommit`
already carries ordered, typed changes, but the reusable Rust core does not yet expose one validated
operation that projects a complete Context state at an exact commit. The result must preserve stable
UUID identity, explicit schema/version semantics, deterministic ordering, and fail-closed validation.

本增量直接推进条件 2 的可回放版本历史与条件 1 的 Context-first 模型。`ContextCommit` 已携带有序、类型化的
changes，但可复用 Rust core 尚未提供一个经过校验的 operation，将完整 Context state 投影到精确 commit。结果必须
保持稳定 UUID、显式 schema/version 语义、确定性排序与 fail-closed validation。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The commit model and
commit-associated graph snapshot repository are present, and benchmark workspace read lifecycle is
already recorded as locally complete at its private contract boundary. What remains unproven is
deterministic replay of a parent-to-child commit chain: component add/update/descriptor/removal,
relationship changes, metadata changes, invalid parent scope, duplicate operations, and missing
preconditions must produce either one exact state or a typed failure. Without this contract, a later
version-backed state read or review could reconstruct Context state differently in Rust, storage, or
Web adapters.

当前已有 commit model 与 commit-associated graph snapshot repository；benchmark workspace read lifecycle 也已在其
private contract boundary 记录为本地完成。尚未证明的是从 parent 到 child 的 commit chain 确定性 replay：component
add/update/descriptor/removal、relationship change、metadata change、invalid parent scope、duplicate operation 与
missing precondition 必须得到唯一 exact state 或 typed failure。没有该 contract，后续 version-backed state read 或 review
可能在 Rust、storage 与 Web adapter 中产生不同的 Context state。

**Why now / 为何现在优先:** This is the smallest dependency-ready local increment on the critical
Context/version spine. It consumes existing `ContextCommit` and domain models without adding a
transport or persistence schema, and it is required before exposing any commit-state inspection,
branch replay, or richer diff review. The benchmark alternative is lower priority because its
private workspace read, server-owned projection, Web integration, and execution projection producer
are already documented as complete locally.

这是当前 Context/version 主骨架上最小且依赖已满足的本地增量。它复用现有 `ContextCommit` 与 domain model，不新增
transport 或 persistence schema，并且是任何 commit-state inspection、branch replay 或更丰富 diff review 之前的必要
前置。Benchmark 方案优先级更低，因为其 private workspace read、server-owned projection、Web integration 与 execution
projection producer 已有本地完成记录。

**Explicit non-goals / 明确非目标:** No commit writer, component-content storage, branch create or
merge, rollback, public REST/OpenAPI/SDK method, local transport, Web mutation, provider call,
benchmark policy, migration, Docker/PostgreSQL runtime, external release evidence, secret access,
or second `GraphDiff` calculator. This plan must not expose raw component bodies; it projects only
the existing version-safe state representation and remains a private Rust contract.

不实现 commit writer、component-content storage、branch create 或 merge、rollback、public REST/OpenAPI/SDK method、
local transport、Web mutation、provider call、benchmark policy、migration、Docker/PostgreSQL runtime、external release
evidence、secret access 或第二个 `GraphDiff` calculator。本计划不得暴露 raw component body，只投影现有 version-safe state
representation，并保持为 private Rust contract。

**Smallest ownership boundary and bilingual documentation / 最小所有权边界与双语文档:** Own only
the replay/state-projection module and export in `crates/versioning`, its focused contract tests,
and this bilingual plan. Do not edit storage, API, SDK, Web, CLI, Desktop, roadmap, or migration
files in this increment. If the existing change payload cannot express a required invariant, record
that blocker in this plan before expanding ownership.

仅拥有 `crates/versioning` 内的 replay/state-projection module 与 export、其 focused contract tests，以及本双语计划。
本增量不得编辑 storage、API、SDK、Web、CLI、Desktop、roadmap 或 migration 文件。若现有 change payload 无法表达某项
必要 invariant，必须先在本计划中记录 blocker，之后才能扩大 ownership。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** First
write red tests for empty-root replay, ordered multi-commit replay, every supported change kind,
precondition and cross-Context rejection, deterministic serialization/order, and replay idempotence.
Then run the focused versioning tests, `cargo fmt --all -- --check`, workspace Rust tests, strict
offline Clippy, locked Rust `1.85.0` checks, and `pnpm check:web`; run static searches proving one
`GraphDiff` implementation and no public write or transport expansion. PostgreSQL/Docker, browser,
Git, remote CI, operator, release, and production evidence remain unobserved or deferred and are
not prerequisites for this local increment.

先为 empty-root replay、ordered multi-commit replay、所有支持的 change kind、precondition 与 cross-Context rejection、
deterministic serialization/order 以及 replay idempotence 编写 red tests。随后运行 focused versioning tests、
`cargo fmt --all -- --check`、workspace Rust tests、strict offline Clippy、锁定 Rust `1.85.0` checks 与 `pnpm check:web`；
并通过静态搜索证明只有一个 `GraphDiff` implementation，且没有扩大 public write 或 transport。PostgreSQL/Docker、
browser、Git、remote CI、operator、release 与 production evidence 继续为 unobserved 或 deferred，不是本地增量前置。

## Implementation Receipt / 实施回执

The proposed record was admitted only within the declared `crates/versioning` boundary. The
implementation adds `ReplayState`, `ReplayComponentState`, `ReplayRelationship`, and structured
`ReplayError`; it folds an ordered normal-parent history atomically, preserves descriptor and body
hash state, removes incident relationships, rejects cross-Context/stale/merge transitions, and
exposes `REPLAY_STATE_SCHEMA_VERSION` plus `ReplayState::from_commits`. It adds no storage, route,
SDK, Web, mutation, provider, migration, public surface, or second `GraphDiff` calculator.

该 proposed record 仅在声明的 `crates/versioning` 边界内准入。实现新增 `ReplayState`、`ReplayComponentState`、
`ReplayRelationship` 与结构化 `ReplayError`；以原子方式折叠有序 normal-parent history，保留 descriptor 与 body hash
state，清理 incident relationship，拒绝 cross-Context/stale/merge transition，并暴露
`REPLAY_STATE_SCHEMA_VERSION` 与 `ReplayState::from_commits`。未新增 storage、route、SDK、Web、mutation、provider、
migration、public surface 或第二个 `GraphDiff` calculator。

Observed local verification:

```text
cargo test -p contextlab-versioning --quiet                 # 37 passed
cargo fmt --package contextlab-versioning -- --check        # passed
cargo clippy -p contextlab-versioning --offline --all-targets -- -D warnings  # passed
```

本地新鲜验证：上述 versioning focused test `37 passed`，package format 与 strict offline Clippy 均通过。Git change-set、
PostgreSQL/Docker、browser、remote CI、operator、release 与 production evidence 未观测或延期；未读取 secrets。

The red-test output from the initial admission was not preserved, so this receipt claims only the
observed green contract evidence. The long-term goal remains active. The next implementation must
still begin with a new bilingual Necessity Record for the next dependency-ready local criterion.

首次准入时的 red-test 输出未被保留，因此本回执只声明已观测的 green contract evidence。长期目标保持 active；下一项
实现仍必须先为下一条依赖就绪的本地完成条件新增双语 Necessity Record。
