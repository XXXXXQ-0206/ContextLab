# Private Context Replay Metadata / 私有 Context 回放元数据

## Necessity Record / 必要性记录

**Criterion and charter principle / 完成条件与章程原则:** This increment directly advances Criterion 1,
the Context-first model, and Criterion 2, replayable version history. The versioning replay projection
must represent every typed Context change that the domain model already names, including Context metadata,
rather than silently producing an incomplete state at a commit. It preserves the reusable Rust core,
explicit contracts, deterministic replay, and bilingual documentation principles.

本增量直接推进条件 1 的 Context-first 模型与条件 2 的可回放版本历史。versioning replay projection 必须表示 domain
model 已命名的每一种 typed Context change，包括 Context metadata，不能在某个 commit 静默产出不完整 state。本增量
保持可复用 Rust core、显式契约、确定性 replay 与双语文档原则。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** `ContextChangeKind::UpdatedMetadata`
exists, but its serialized payload has no Context-metadata field and `ReplayState` currently rejects it as
unsupported. A caller can therefore persist a named change that the reusable replay contract cannot project.
Adding a payload in a transport layer would duplicate domain semantics; leaving it unsupported makes exact
commit state incomplete.

`ContextChangeKind::UpdatedMetadata` 已存在，但其 serialized payload 没有 Context metadata field，`ReplayState` 当前也将
其作为 unsupported 拒绝。因此 caller 可以持久化一个已命名却无法由可复用 replay contract 投影的 change。在 transport layer
增加 payload 会复制 domain semantics；继续 unsupported 则会使 exact commit state 不完整。

**Why now / 为何现在优先:** The dependency-ready `ReplayState` contract is now locally green and exposes the
precise missing domain field. Completing metadata replay is the smallest direct follow-up before any storage
consumer, commit-state transport, or richer version review depends on an incomplete projection.

当前依赖就绪的 `ReplayState` contract 已本地转绿，并暴露了明确缺失的 domain field。在任何 storage consumer、commit-state
transport 或更丰富的 version review 依赖不完整 projection 之前，补齐 metadata replay 是最小且直接的后续增量。

**Explicit non-goals / 明确非目标:** No storage adapter, commit writer, migration, REST/OpenAPI/SDK method,
local transport, Web mutation, branch/merge/rollback, provider, Docker/PostgreSQL runtime, external release
evidence, secret access, or second `GraphDiff` calculator. The metadata remains a typed Rust-domain value and
must not expose raw component bodies.

不实现 storage adapter、commit writer、migration、REST/OpenAPI/SDK method、local transport、Web mutation、branch/merge/rollback、
provider、Docker/PostgreSQL runtime、external release evidence、secret access 或第二个 `GraphDiff` calculator。metadata 仍是
typed Rust-domain value，不得暴露 raw component body。

**Smallest ownership boundary and bilingual documentation / 最小所有权边界与双语文档:** Own only
`crates/versioning/src/change.rs`, `crates/versioning/src/replay.rs`, `crates/versioning/src/lib.rs`, their
focused tests, and this plan. No other crate or roadmap file is edited by this increment.

仅拥有 `crates/versioning/src/change.rs`、`crates/versioning/src/replay.rs`、`crates/versioning/src/lib.rs`、其 focused test
与本计划。本增量不编辑其他 crate 或 roadmap 文件。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** First add focused
contract tests for serialized metadata payload validation, exact replay projection, malformed/missing payload
rejection, and atomic failure. Then run `cargo test -p contextlab-versioning --quiet`, package format, strict offline
Clippy, workspace Rust tests, locked Rust 1.85 checks, and `pnpm check:web`. Static searches must still show one
`GraphDiff` implementation and no public write/transport expansion; PostgreSQL/Docker, browser, Git, remote CI,
operator, release, and production remain unobserved or deferred.

先增加 serialized metadata payload validation、exact replay projection、malformed/missing payload rejection 与 failure atomicity
的 focused contract test。随后运行 `cargo test -p contextlab-versioning --quiet`、package format、strict offline Clippy、workspace
Rust tests、锁定 Rust 1.85 checks 与 `pnpm check:web`。静态搜索仍须显示只有一个 `GraphDiff` implementation 且没有扩大 public
write/transport；PostgreSQL/Docker、browser、Git、remote CI、operator、release 与 production 继续为 unobserved 或 deferred。

## Implementation Status and Fresh Evidence / 实施状态与新鲜证据

`completed` / `verified locally`. A bounded `gpt-5.6-luna` versioning worker implemented the typed
`ContextMetadataPayload` and `ReplayState` projection. Integration review found one direct
fail-closed gap: the outer `ContextChangeWire` accepted unknown fields. The minimal repair adds
`serde(deny_unknown_fields)` and a regression proving schema drift is rejected instead of dropped.

`completed` / `verified locally`。有界 `gpt-5.6-luna` versioning worker 已实现 typed
`ContextMetadataPayload` 与 `ReplayState` projection。Integration review 发现一个直接的 fail-closed 缺口：外层
`ContextChangeWire` 会接受未知字段。最小修复增加 `serde(deny_unknown_fields)`，并加入回归证明 schema drift 会被拒绝而不是静默丢弃。

Fresh local receipts after the repair: `cargo test -p contextlab-versioning --quiet` passed `41`;
`cargo fmt --all -- --check` passed; package and workspace strict offline Clippy passed;
`cargo test --workspace --quiet` passed with API `183` and storage `186 passed, 39 ignored`;
`cargo +1.85.0 check --workspace --all-targets --locked` passed; and `pnpm check:web` passed
public SDK `15`, local SDK `92`, Web `189`, TypeScript/lint, and the local production build.
Static inspection observed exactly one `impl GraphDiff` (`GRAPH_DIFF_IMPL_COUNT=1`).

修复后的新鲜本地回执：`cargo test -p contextlab-versioning --quiet` 通过 `41` 项；`cargo fmt --all -- --check` 通过；
package 与 workspace strict offline Clippy 通过；`cargo test --workspace --quiet` 通过，其中 API `183`、storage `186 passed, 39 ignored`；
`cargo +1.85.0 check --workspace --all-targets --locked` 通过；`pnpm check:web` 通过 public SDK `15`、local SDK `92`、
Web `189`、TypeScript/lint 与本地 production build。静态检查观测到唯一的 `impl GraphDiff`（`GRAPH_DIFF_IMPL_COUNT=1`）。

This remains a private Rust in-memory contract. ReplayState serialization, descriptor stale-write
preconditions, and component-metadata content limits are recorded as later Necessity Records before
storage, user-input, or wire expansion. No storage adapter, transport, public write, Web mutation,
migration, Docker/PostgreSQL runtime, secret, external receipt, release, or production claim was added.
PostgreSQL/Docker runtime, authenticated browser, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`; the long-term goal remains active.

本增量仍是 private Rust in-memory contract。ReplayState serialization、descriptor stale-write precondition 与
component-metadata content limit 将作为后续 Necessity Record，在 storage、user-input 或 wire 扩展前单独准入。未新增
storage adapter、transport、public write、Web mutation、migration、Docker/PostgreSQL runtime、secret、external receipt、
release 或 production claim。PostgreSQL/Docker runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`；长期目标保持 active。

The next increment requires a new bilingual Necessity Record for the private storage replay adapter;
it must reuse `ReplayState::from_commits`, keep the immutable content-revision repository separate,
and add Memory/PostgreSQL parity evidence without adding transport or public surface.

下一增量必须先新增 private storage replay adapter 的双语 Necessity Record；实现必须复用 `ReplayState::from_commits`，
保持 immutable content-revision repository 独立，并增加 Memory/PostgreSQL parity evidence，同时不新增 transport 或 public surface。
