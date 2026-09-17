# Private Lifecycle Metadata Diff Propagation / 私有生命周期 Metadata Diff 传播

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与章程原则

- **Criterion 2 / 条件 2:** Version-backed Context history must preserve replayable state and
  produce a reviewable semantic diff for the exact commit pair.
  / **条件 2：** 版本化 Context history 必须保留可回放状态，并为 exact commit pair 产生可审阅的 semantic diff。
- **Context-first and reusable Rust core / Context-first 与可复用 Rust core：** Context metadata is
  part of the Context snapshot, not an incidental UI-only field; the existing storage writer and
  `ContextDiffSnapshotV1` contract remain the single composition path.
  / Context metadata 是 Context snapshot 的组成部分，不是仅供 UI 使用的附属字段；继续使用现有 storage writer 与 `ContextDiffSnapshotV1` contract 作为唯一组合路径。

### Gap, dependencies, and evidence / 缺口、依赖与证据

`ContextLifecycleService` correctly replays `UpdateMetadata` into `ReplayState`, but the guarded
commit snapshot writer always builds a graph-only `SemanticSnapshotV1` with no Context metadata.
Consequently the persisted exact-commit diff review can lose a real lifecycle metadata transition.
The typed `ContextMetadata` model, replay state, `SemanticSnapshotV1::new_with_metadata`, pair-read
repository, and `VersionedContextDiffReviewService` already exist. The missing evidence is a
writer-to-persisted-review red/green regression, including inheritance of metadata by a later
non-metadata commit.

`ContextLifecycleService` 正确地将 `UpdateMetadata` 回放到 `ReplayState`，但 guarded commit snapshot writer 始终构造不含 Context metadata 的 graph-only `SemanticSnapshotV1`。因此 persisted exact-commit diff review 可能丢失真实的 lifecycle metadata transition。typed `ContextMetadata` model、replay state、`SemanticSnapshotV1::new_with_metadata`、pair-read repository 与 `VersionedContextDiffReviewService` 都已存在；当前缺少的是 writer-to-persisted-review red/green regression，且必须覆盖后续 non-metadata commit 继承 metadata。

### Why now / 为何现在优先

This is the highest-priority dependency-ready local gap identified by two independent Luna
reviewers. It is a correctness defect in the Context-first version/diff spine, not a display
enhancement: the current metadata parser and manually seeded storage fixtures can pass while a
real guarded lifecycle commit still produces an incomplete diff snapshot. Fixing it now closes a
named Criterion 2 evidence gap before any new benchmark, workflow, or public-surface work.

这是两名独立 Luna reviewer 共同识别的最高优先级、依赖就绪本地缺口。它是 Context-first version/diff spine 的正确性缺陷，而不是展示增强：当前 metadata parser 与手工构造的 storage fixture 可以通过，但真实 guarded lifecycle commit 仍可能产生不完整 diff snapshot。现在修复它，可以在新增 benchmark、workflow 或 public surface 前收束命名的 Criterion 2 证据缺口。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/SDK write, operator transport, Web mutation, migration, provider,
  secret access, or second `GraphDiff` calculator.
  / 不新增 public REST/OpenAPI/SDK write、operator transport、Web mutation、migration、provider、secret access 或第二个 `GraphDiff` calculator。
- No new diff algorithm, behavior/evaluation producer, backfill, or mutable-current-row lookup.
  / 不新增 diff algorithm、behavior/evaluation producer、backfill，也不读取 mutable current row。
- PostgreSQL runtime, Docker, browser, Git, remote CI, release, and production evidence remain
  separately `unobserved` or `deferred`.
  / PostgreSQL runtime、Docker、browser、Git、remote CI、release 与 production evidence 继续分别标记为 `unobserved` 或 `deferred`。

### Smallest boundary and ownership / 最小边界与所有权

- **Luna storage owner / Luna storage 所有者:** `crates/storage/src/commit_snapshot_writer.rs`,
  `crates/storage/src/context_lifecycle.rs`, and their focused Rust tests. The writer gets an
  optional exact resulting metadata value; lifecycle passes revised metadata for `UpdateMetadata`
  and inherited parent metadata for other successor commits.
  / `crates/storage/src/commit_snapshot_writer.rs`、`crates/storage/src/context_lifecycle.rs` 及其 focused Rust tests。writer 接收可选 exact resulting metadata；lifecycle 为 `UpdateMetadata` 传 revised metadata，为其他 successor commit 传父 metadata。
- **Luna API regression owner / Luna API 回归所有者:** only the existing writer-to-review fixture
  in `server/api/src/lib.rs`; prove a real metadata transition through the persisted review adapter.
  / 仅修改 `server/api/src/lib.rs` 中既有 writer-to-review fixture，证明真实 metadata transition 穿过 persisted review adapter。
- Integration Lead owns this plan, roadmap receipts, integration review, and final verification.
  / Integration Lead 负责本计划、roadmap 回执、集成审查与最终验证。

### Fresh verification required / 所需新鲜验证

First observe a red assertion showing a real lifecycle `UpdateMetadata` pair produces no persisted
metadata transition. Then implement the smallest propagation change and observe green focused
writer/lifecycle/storage/API tests, including a later non-metadata commit inheriting metadata.
Run workspace Rust tests, format, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web`,
the scoped local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`.

先观测真实 lifecycle `UpdateMetadata` commit pair 没有 persisted metadata transition 的 red assertion，再实现最小传播修复，并观测 writer/lifecycle/storage/API focused tests 变绿，覆盖后续 non-metadata commit 继承 metadata。运行 workspace Rust tests、format、strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`、scoped local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。

## Execution Checklist / 执行清单

- [x] Add the writer-to-persisted-review regression and inheritance coverage.
  / 增加 writer-to-persisted-review red regression 与继承 coverage。
- [x] Propagate exact resulting/inherited Context metadata through the existing guarded writer。
  / 通过现有 guarded writer 传播 exact resulting/inherited Context metadata。
- [x] Run fresh cross-stack verification and record evidence without closing the long-term goal。
  / 运行新鲜 cross-stack verification 并记录证据，不关闭长期目标。

## Fresh Verification / 新鲜验证

The private writer now accepts optional exact commit metadata. `UpdateMetadata` clones the
resulting value before consuming it in the versioning change; later component and relationship
commits inherit the parent metadata. The existing memory/PostgreSQL writer path persists the same
`ContextDiffSnapshotV1`, and replay returns the immutable receipt without replacing it.

私有 writer 现接受 exact commit metadata（可选）。`UpdateMetadata` 在将值交给 versioning change 消费前先保留 resulting value；后续 component 与 relationship commit 继承 parent metadata。既有 memory/PostgreSQL writer path 持久化同一个 `ContextDiffSnapshotV1`，replay 返回不可变 receipt，不会替换它。

Fresh local evidence / 新鲜本地证据：

- `cargo test -p contextlab-storage lifecycle_update_metadata_replays_typed_metadata_through_the_guarded_writer --offline`: `1 passed`.
- `cargo test -p contextlab-api memory_guarded_lifecycle_metadata_update_is_visible_in_persisted_diff_review --offline`: `1 passed`.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage reports `212 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust `1.85.0` check: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `135`, Web `280`, TypeScript/lint and production build all passed.
- `pwsh -NoProfile -File .\\tests\\contract\\verify-local-contracts.test.ps1`: `verify-local-contracts fixture tests passed`; the deterministic safe diff branch reports `overall=passed`.
- Live local verifier: source, graph, safe DTO, protected route, and `graph_diff_application=passed count=1`; its `overall=unobserved` status is only because no live unified diff input was supplied.

The metadata transition, later non-metadata inheritance, exact persisted pair review, and
idempotent replay are now locally verified. PostgreSQL runtime, Docker, authenticated browser,
Git change-set, remote CI, operator rehearsal, release, and production evidence remain
`unobserved` or `deferred`. No public write, Web mutation, migration, provider, secret access,
operator transport, or second `GraphDiff` calculator was added. The long-term goal remains active.

metadata transition、后续 non-metadata 继承、exact persisted pair review 与幂等 replay 现已获得本地验证。PostgreSQL runtime、Docker、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production evidence 继续为 `unobserved` 或 `deferred`。未新增 public write、Web mutation、migration、provider、secret access、operator transport 或第二个 `GraphDiff` calculator。长期目标保持 active。
