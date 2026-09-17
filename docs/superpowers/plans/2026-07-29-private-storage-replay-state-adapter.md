# Private Storage ReplayState Adapter / 私有 Storage ReplayState Adapter

## Necessity Record / 必要性记录

**Criterion and charter principle / 完成条件与章程原则:** This increment directly advances Criteria 1 and 2:
Context remains the primary, replayable state model and the reusable Rust versioning contract must be
consumed by persistence rather than allowing Memory and PostgreSQL to maintain a second descriptor
transition policy. It preserves stable IDs, deterministic ordering, exact scope validation, and the
Rust-core/domain boundary.

本增量直接推进条件 1 与 2：Context 必须是首要且可回放的 state model；持久化层应消费可复用的 Rust versioning contract，
而不是让 Memory 与 PostgreSQL 继续各自维护第二套 descriptor transition policy。本增量保持稳定 ID、确定性排序、exact scope
validation 与 Rust-core/domain boundary。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** `ReplayState::from_commits`
now has fresh local evidence, but storage does not yet reconstruct persisted `ContextCommitRecord`
values into that domain type. Existing component replay remains useful for immutable content-revision
witnesses, but it does not project Context metadata or `Uses` relationships and can drift from the
versioning contract.

`ReplayState::from_commits` 已取得新鲜本地证据，但 storage 仍未把持久化的 `ContextCommitRecord` 重建为该 domain type。
现有 component replay 仍负责 immutable content-revision witness，但不会投影 Context metadata 或 `Uses` relationship，且
可能与 versioning contract 漂移。

**Why now / 为何现在优先:** The metadata replay contract is now verified and the existing Memory/PostgreSQL
normal-parent history queries already provide the exact ordered ancestry needed by the adapter. This
is the nearest dependency-ready consumer before exposing richer commit-state review or adding any
public/local transport.

metadata replay contract 已完成验证；现有 Memory/PostgreSQL normal-parent history query 已提供 adapter 所需的 exact ordered
ancestry。因此在增加更丰富 commit-state review 或任何 public/local transport 前，这是最近且依赖已满足的 consumer 增量。

**Explicit non-goals / 明确非目标:** No commit writer, merge/rollback, branch mutation, content-body
repository, migration, REST/OpenAPI/SDK method, local transport, Web mutation, provider call,
Docker/PostgreSQL runtime execution, external release evidence, secret access, descriptor stale-write
policy, component-metadata content policy, ReplayState serialization, or second `GraphDiff` calculator.

不实现 commit writer、merge/rollback、branch mutation、content-body repository、migration、REST/OpenAPI/SDK method、local
transport、Web mutation、provider call、Docker/PostgreSQL runtime execution、external release evidence、secret access、descriptor
stale-write policy、component-metadata content policy、ReplayState serialization 或第二个 `GraphDiff` calculator。

**Smallest ownership boundary and bilingual documentation / 最小所有权边界与双语文档:** Own
`crates/versioning/src/commit.rs` and focused tests for persisted commit reconstruction; a new private
`crates/storage/src/replay_state_at_commit.rs`; the smallest Memory/PostgreSQL adapter additions and
focused storage tests; this plan and the three roadmap receipts. Do not edit API, SDK, Web, OpenAPI,
or migration files.

仅拥有 `crates/versioning/src/commit.rs` 及其 focused tests、私有的 `crates/storage/src/replay_state_at_commit.rs`、Memory/PostgreSQL
adapter 的最小新增与 focused storage tests、本计划及三份路线图回执。不编辑 API、SDK、Web、OpenAPI 或 migration 文件。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Focused
versioning reconstruction and storage adapter tests, Memory parity tests, PostgreSQL SQL/contract tests
that do not require a live runtime, `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, strict
offline workspace Clippy, locked Rust `1.85.0` check, `pnpm check:web`, and static checks for one
`GraphDiff` implementation and no public transport/write expansion. Docker/PostgreSQL runtime,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`.

先运行 focused versioning reconstruction 与 storage adapter tests、Memory parity tests、无需 live runtime 的 PostgreSQL SQL/contract
tests，再运行 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`、strict offline workspace Clippy、锁定 Rust `1.85.0`
check、`pnpm check:web`，并静态检查只有一个 `GraphDiff` implementation 且没有 public transport/write 扩张。Docker/PostgreSQL
runtime、authenticated browser、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

## Implementation and verification receipt / 实施与验证回执

`completed` / `verified locally`. The private adapter reconstructs persisted
`ContextCommitRecord` values without generating replacement identities, and both Memory and
PostgreSQL route the projection through the same `ReplayState::from_commits` policy. Memory walks
the validated normal-parent history; PostgreSQL uses the shared recursive history query and
rejects missing targets, merge ancestry, cycles, cross-Context parents, and malformed changes
before replay. No public surface was expanded.

`completed` / `verified locally`。私有 adapter 会在不生成替代 identity 的前提下重建持久化
`ContextCommitRecord`，Memory 与 PostgreSQL 均通过同一个 `ReplayState::from_commits` policy 完成 projection。Memory
读取经校验的 normal-parent history；PostgreSQL 使用共享递归 history query，并在 replay 前拒绝 missing target、merge
ancestry、cycle、cross-Context parent 与 malformed changes。本增量未扩大 public surface。

Fresh local evidence / 新鲜本地证据:

- `cargo test -p contextlab-storage memory::tests::in_memory_replay_state_repository_matches_normal_parent_and_missing_commit_contract --no-fail-fast`: `1 passed`.
- `cargo test -p contextlab-storage replay_state_history --quiet --no-fail-fast`: `4 passed` for the replay-history SQL/row contract tests.
- `cargo test -p contextlab-storage --quiet --no-fail-fast`: storage library `193 passed, 39 ignored`; all auxiliary storage targets passed.
- `cargo fmt --all -- --check`: passed.

- `cargo test -p contextlab-storage memory::tests::in_memory_replay_state_repository_matches_normal_parent_and_missing_commit_contract --no-fail-fast`：`1 passed`。
- `cargo test -p contextlab-storage replay_state_history --quiet --no-fail-fast`：replay-history SQL/row contract `4 passed`。
- `cargo test -p contextlab-storage --quiet --no-fail-fast`：storage library `193 passed, 39 ignored`；所有 auxiliary storage target 均通过。
- `cargo fmt --all -- --check`：通过。

The final workspace gate also passed `cargo test --workspace --quiet --no-fail-fast` (API `183`,
versioning `45`, storage `193 passed, 39 ignored`, with all other targets passing),
`cargo clippy --workspace --all-targets --offline -- -D warnings`,
`cargo +1.85.0 check --workspace --all-targets --locked`, and `pnpm check:web` (public SDK `15`,
local SDK `92`, Web `189`, TypeScript/lint, and local production build). Static inspection observed
`GRAPH_DIFF_IMPL_COUNT=1`.

最终 workspace gate 还通过了 `cargo test --workspace --quiet --no-fail-fast`（API `183`、versioning `45`、storage `193 passed, 39 ignored`，
其余 target 均通过）、`cargo clippy --workspace --all-targets --offline -- -D warnings`、
`cargo +1.85.0 check --workspace --all-targets --locked` 与 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `189`、
TypeScript/lint 与本地 production build）。静态检查观测到 `GRAPH_DIFF_IMPL_COUNT=1`。

The PostgreSQL worker's endpoint returned `502`; this is recorded as a worker handoff, not as
verification. The Integration Lead ran the bounded PostgreSQL checks serially after takeover.
Docker/PostgreSQL runtime, authenticated browser, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`; no secrets were read. The long-term
goal remains active.

PostgreSQL worker 的 endpoint 返回 `502`；该事实记录为 worker handoff，不作为验证结果。Integration Lead 接管后以串行
方式运行了有界 PostgreSQL checks。Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator
rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未读取 secrets。长期目标保持 active。

## Next increment / 下一增量

This slice advances Criteria 1 and 2 but does not close either criterion. Before any replay
serialization, descriptor stale-write precondition, content policy, transport, or public write,
add a new bilingual Necessity Record and obtain fresh red/green evidence for the next
dependency-ready Context editing, benchmark, or replay consumer increment.

本切片推进条件 1 与 2，但不关闭任一条件。在推进 ReplayState serialization、descriptor stale-write precondition、content
policy、transport 或 public write 前，必须为下一项依赖就绪的 Context editing、benchmark 或 replay consumer 增量新增双语
Necessity Record，并取得新鲜 red/green 证据。
