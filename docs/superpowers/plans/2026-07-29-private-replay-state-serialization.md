# Private ReplayState Serialization Envelope / 私有 ReplayState 序列化信封

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criteria 1 and 2: Context remains the primary state model, and a replayed state at an immutable
commit must have a stable, versioned representation for local persistence and later review. The
contract stays in reusable Rust versioning core and preserves deterministic ordering, stable IDs,
fail-closed validation, and the single `GraphDiff::between` boundary.

本增量直接推进条件 1 与 2：Context 仍是首要 state model；不可变 commit 上的 replay state 必须具备稳定、带版本的
表示，才能支持本地持久化与后续 review。本 contract 继续位于可复用 Rust versioning core，保持确定性排序、稳定 ID、
fail-closed 校验与唯一的 `GraphDiff::between` boundary。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** `ReplayState` can be
constructed and consumed by Memory/PostgreSQL through `ReplayState::from_commits`, but its private
fields have no explicit V1 wire envelope. Consumers would otherwise serialize implementation
details independently, risking schema drift, unstable ordering, duplicate relationships, or a
state that names components absent from the replayed Context.

`ReplayState` 现可由 Memory/PostgreSQL 通过 `ReplayState::from_commits` 构造和消费，但其 private fields 尚无明确的 V1
wire envelope。否则各 consumer 可能自行序列化 implementation details，导致 schema drift、排序不稳定、relationship 重复，
或出现引用不存在 component 的 state。

**Why now / 为何现在优先:** The storage adapter and Memory/PostgreSQL parity are freshly verified,
so the next dependency-ready gap is the reusable serialization boundary that both adapters can
eventually share. It is closer to replayable version history than new transport, UI, provider, or
workflow work, and it does not require a live database.

storage adapter 与 Memory/PostgreSQL parity 刚取得新鲜验证，因此下一项依赖已满足的缺口是两端未来可共享的可复用
serialization boundary。它比新增 transport、UI、provider 或 workflow 更直接服务可回放版本历史，且不需要 live database。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:** Own
`crates/versioning/src/replay.rs`, its `lib.rs` re-export if needed, and one integration-test file
under `crates/versioning/tests/`. The envelope is a private Rust contract; update this plan and
the active-long-term-goal, completion-criteria, and parallel-development roadmap receipts. Do not
touch storage adapters, API, OpenAPI, SDK, Web, migrations, or provider code.

仅拥有 `crates/versioning/src/replay.rs`、必要时的 `lib.rs` re-export，以及 `crates/versioning/tests/` 下一个
integration-test 文件。该 envelope 是 private Rust contract；同步更新本计划、active-long-term-goal、completion-criteria
与 parallel-development roadmap 回执。不修改 storage adapter、API、OpenAPI、SDK、Web、migration 或 provider code。

**Explicit non-goals / 明确非目标:** No replay writer, commit mutation, graph mutation, merge or
rollback, storage serialization adapter, migration, REST/OpenAPI/SDK/Web/CLI/Desktop surface,
raw component body, metadata content policy, stale-write precondition, provider call, Docker or
production claim, external release evidence, or second diff calculator. `GraphDiff::between`
remains the sole graph-diff calculator.

不实现 replay writer、commit mutation、graph mutation、merge 或 rollback、storage serialization adapter、migration、
REST/OpenAPI/SDK/Web/CLI/Desktop surface、raw component body、metadata content policy、stale-write precondition、provider
call、Docker 或 production claim、external release evidence 或第二个 diff calculator。`GraphDiff::between` 仍是唯一
graph-diff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Add
red/green tests for canonical JSON round-trip, explicit schema version, unknown-field rejection,
duplicate component/relationship rejection, missing relationship endpoint rejection, and state
equality after restore. Then run focused versioning tests, workspace Rust tests, format, strict
offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, and static checks for one
`GraphDiff` implementation and no public-surface expansion. PostgreSQL runtime, browser, Git,
remote CI, operator rehearsal, release, and production remain separately labeled.

先增加 canonical JSON round-trip、显式 schema version、unknown-field rejection、duplicate component/relationship rejection、
missing relationship endpoint rejection 与 restore 后 state equality 的红绿测试。随后运行 focused versioning test、workspace
Rust test、format、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`，并静态检查只有一个 `GraphDiff`
implementation 且没有 public-surface expansion。PostgreSQL runtime、browser、Git、remote CI、operator rehearsal、release
与 production 继续单独标记。

## Ownership and admission / 所有权与准入

The increment is admitted as one bounded `gpt-5.6-luna` versioning wave. The Integration Lead owns
the domain implementation in `replay.rs`; a separate Luna worker may own only the integration test
file. The long-term goal remains active and this plan does not authorize any public or production
claim.

本增量准入为一个有界的 `gpt-5.6-luna` versioning wave。Integration Lead 负责 `replay.rs` domain implementation；独立
Luna worker 只能负责 integration test 文件。长期目标保持 active，本计划不授权任何 public 或 production claim。

## Implementation and verification receipt / 实施与验证回执

`ReplayStateSnapshotV1` now provides an explicit, opaque V1 envelope for exact replay state. It
serializes stable Context/commit identity, metadata, descriptor-only component state, and Uses
relationships in canonical order. `ReplayState::from_json` rejects unsupported schema versions,
unknown outer or nested fields, duplicate components or relationships, nil identities, and
relationships whose endpoints are absent. Restore rebuilds the same in-memory state and never
includes component bodies.

`ReplayStateSnapshotV1` 现提供明确且 opaque 的 V1 replay state envelope。它按 canonical order 序列化稳定 Context/commit
identity、metadata、descriptor-only component state 与 Uses relationship。`ReplayState::from_json` 会拒绝 unsupported schema
version、outer 或 nested unknown field、重复 component 或 relationship、nil identity，以及 endpoint 不存在于 component
集合中的 relationship。restore 会重建相同的 in-memory state，且永不包含 component body。

Fresh local evidence / 新鲜本地证据:

- `cargo test -p contextlab-versioning --test replay_state_serialization --quiet --no-fail-fast`: `8 passed`.
- `cargo test -p contextlab-versioning --quiet --no-fail-fast`: `42` unit tests and `8` serialization integration tests passed.
- `cargo test --workspace --quiet --no-fail-fast`: API `183`, storage `193 passed, 39 ignored`, versioning and all other targets passed.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --offline -- -D warnings`, and `cargo +1.85.0 check --workspace --all-targets --locked`: passed.
- `pnpm check:web`: public SDK `15`, local SDK `92`, Web `189`, TypeScript/lint, and local production build passed.

- `cargo test -p contextlab-versioning --test replay_state_serialization --quiet --no-fail-fast`：`8 passed`。
- `cargo test -p contextlab-versioning --quiet --no-fail-fast`：`42` 个 unit test 与 `8` 个 serialization integration test 通过。
- `cargo test --workspace --quiet --no-fail-fast`：API `183`、storage `193 passed, 39 ignored`，versioning 与其余 target 均通过。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --offline -- -D warnings` 与 `cargo +1.85.0 check --workspace --all-targets --locked`：通过。
- `pnpm check:web`：public SDK `15`、local SDK `92`、Web `189`、TypeScript/lint 与本地 production build 通过。

No storage/API/SDK/Web/migration surface changed, no secrets or provider were accessed, and
`GraphDiff::between` remains the sole graph-diff calculator. PostgreSQL runtime, authenticated
browser, Git change-set, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`. The long-term goal remains active.

未修改 storage/API/SDK/Web/migration surface，未访问 secrets 或 provider，`GraphDiff::between` 仍是唯一 graph-diff
calculator。PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production
继续为 `unobserved` 或 `deferred`。长期目标保持 active。

## Next increment / 下一增量

This slice advances Criteria 1 and 2 without closing them. The next implementation requires a new
bilingual Necessity Record for a dependency-ready Context editing, benchmark evidence, or replay
consumer increment; serialization does not authorize a storage writer or public transport.

本切片推进条件 1 与 2，但不关闭它们。下一项实现必须先为依赖就绪的 Context editing、benchmark evidence 或 replay
consumer 增量新增双语 Necessity Record；serialization 不授权 storage writer 或 public transport。
