# Private Versioned ContextGraph Merge Review / 私有版本化 ContextGraph Merge Review

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criteria 2 and 4: replayable version history and reusable semantic/graph diff boundaries must
represent a graph review by exact version identity, not by an unversioned graph tuple. The
version-bound contract must preserve Context as the primary abstraction and keep
`GraphDiff::between` behind the existing graph classifier.

本增量直接推进条件 2 与条件 4：可回放版本历史与可复用 semantic/graph diff boundary 必须以 exact version identity
表达 graph review，不能停留在无版本的 graph tuple。version-bound contract 必须保持 Context 为首要抽象，并让
`GraphDiff::between` 继续隐藏在既有 graph classifier 之后。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The repository bridge now
reads exact base/left/right persisted snapshots and the pure classifier already enforces a
matching three-way ancestry plan, but the application boundary does not yet own a stable V1
request/projection for version-bound graph review. Without it, a future local adapter could
reconstruct graph identities ad hoc or expose an unversioned classification result.

当前 repository bridge 已读取 exact base/left/right persisted snapshot，pure classifier 也已强制匹配 three-way ancestry
plan，但 application boundary 还没有拥有 version-bound graph review 的稳定 V1 request/projection。缺少它时，后续 local
adapter 可能各自重建 graph identity，或暴露无版本绑定的 classification result。

**Why now / 为何现在优先:** Merge-base validation, graph conflict classification, and the
persisted exact-scope review bridge are all locally verified. This is the smallest dependency-ready
application contract that connects those completed layers before any merge writer, replay command,
or transport. It is closer to the named version/Diff criteria than a new UI or provider feature.

merge-base validation、graph conflict classification 与 persisted exact-scope review bridge 均已完成本地验证。这是连接
这些已完成层、并先于任何 merge writer、replay command 或 transport 的最小依赖就绪 application contract；相比新增 UI
或 provider，它更直接服务已命名的 version/Diff 条件。

**Smallest boundary and bilingual documentation / 最小边界与双语文档:** Own only
`crates/diff-engine/src/versioned_graph_merge_review.rs`, its `lib.rs` export, focused diff-engine
contract tests, the storage delegation change in
`crates/storage/src/context_merge_review.rs`, and this plan plus bilingual architecture/roadmap
receipts. The storage adapter remains the repository boundary; the versioned service owns only
typed request validation and projection.

仅负责 `crates/diff-engine/src/versioned_graph_merge_review.rs`、其 `lib.rs` 导出、focused diff-engine contract test、
`crates/storage/src/context_merge_review.rs` 的 delegation change，以及本计划与双语 architecture/roadmap 回执。
storage adapter 继续是 repository boundary；versioned service 只负责 typed request validation 与 projection。

**Explicit non-goals / 明确非目标:** No merged graph, conflict resolution, branch mutation,
rollback, persistence schema, PostgreSQL transaction, REST/OpenAPI/SDK method, Web/CLI/Desktop
surface, provider call, raw private-content projection, public write, Docker/runtime, browser,
secret, remote CI, operator, release, or production claim. No second graph-diff calculator and no
direct graph-diff call outside the existing classifier.

不实现 merged graph、conflict resolution、branch mutation、rollback、persistence schema、PostgreSQL transaction、
REST/OpenAPI/SDK method、Web/CLI/Desktop surface、provider call、raw private-content projection、public write、
Docker/runtime、browser、secret、remote CI、operator、release 或 production claim。不新增第二个 graph-diff calculator，
也不允许绕过既有 classifier 直接调用 graph diff。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** First add
red tests for schema/version rejection, exact scope/plan mismatch, deterministic serialized
projection, and storage delegation. Then run focused diff-engine and storage tests, format, workspace
Rust tests, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, and the static
singularity/public-surface searches. PostgreSQL runtime remains unobserved and external release
evidence remains deferred.

先增加 schema/version rejection、exact scope/plan mismatch、确定性 serialized projection 与 storage delegation 的红测，
再运行 focused diff-engine/storage test、format、workspace Rust test、strict offline Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web` 与 static singularity/public-surface search。PostgreSQL runtime 继续 unobserved，外部 release evidence
继续 deferred。

## Implementation Checklist / 实施清单

- [x] Add the V1 version-bound graph snapshot/request/projection contract.
- [x] Delegate through the existing `GraphMergeConflictClassifier` only.
- [x] Adapt the private storage review service without changing its repository or transport boundary.
- [x] Record fresh bilingual architecture/roadmap evidence and select the next dependency-ready increment.

- [x] 增加 V1 version-bound graph snapshot/request/projection contract。
- [x] 只通过既有 `GraphMergeConflictClassifier` 完成 delegation。
- [x] 适配 private storage review service，但不改变 repository 或 transport boundary。
- [x] 记录新鲜双语 architecture/roadmap evidence，并选择下一项依赖就绪增量。

## Implementation Receipt / 实施回执

`crates/diff-engine/src/versioned_graph_merge_review.rs` now owns the private V1 application
contract. It serializes an explicit schema version, exact base/left/right
`(ProjectId, ContextId, CommitId)` scopes, owned graph snapshots, the matching `MergePlan`, and a
deterministic classification projection. Request deserialization rejects unknown shape or schema
drift; review rejects nil or duplicate snapshot identities, then delegates only to
`GraphMergeConflictClassifier`.

`crates/diff-engine/src/versioned_graph_merge_review.rs` 现负责 private V1 application contract。它序列化显式 schema
version、exact base/left/right `(ProjectId, ContextId, CommitId)` scope、owned graph snapshot、匹配的 `MergePlan` 与确定性
classification projection。request deserialization 会拒绝 unknown shape 或 schema drift；review 会拒绝 nil 或重复
snapshot identity，然后只委托 `GraphMergeConflictClassifier`。

`crates/storage/src/context_merge_review.rs` now constructs the version-bound request from its
exact persisted reads and extracts the already-computed classification for the existing private
storage port. It does not call `GraphDiff` directly. The focused diff-engine contract passed `4`
tests; the focused storage contract passed `5` tests. A first integration compile failure exposed
only the projection-to-legacy-port adapter mismatch; extracting the projection classification was
the minimal behavior-preserving fix.

`crates/storage/src/context_merge_review.rs` 现从 exact persisted read 构造 version-bound request，并从已计算的 projection
提取 classification 以保持既有 private storage port。它不直接调用 `GraphDiff`。focused diff-engine contract 通过 `4`
项测试；focused storage contract 通过 `5` 项测试。首次联调编译失败只暴露 projection 到 legacy port 的 adapter 类型
不匹配；提取 projection classification 是保持行为不变的最小修复。

Fresh verification passed: `cargo fmt --all -- --check`; `cargo test --workspace --quiet` with
`186 passed, 39 ignored`; strict offline workspace Clippy; `cargo +1.85.0 check --workspace
--all-targets --locked`; `pnpm check:web` with public SDK `15`, local SDK `92`, Web `185`, and the
production build; and static `impl GraphDiff count=1`. PostgreSQL/Docker runtime, browser, Git,
remote CI, operator, release, and production remain `unobserved` or `deferred`. The next
increment requires a new bilingual Necessity Record.

新鲜验证已通过：`cargo fmt --all -- --check`；`cargo test --workspace --quiet`（`186 passed, 39 ignored`）；strict offline
workspace Clippy；`cargo +1.85.0 check --workspace --all-targets --locked`；`pnpm check:web`（public SDK `15`、local SDK `92`、
Web `185` 与 production build）；以及 static `impl GraphDiff count=1`。PostgreSQL/Docker runtime、browser、Git、remote CI、
operator、release 与 production 仍为 `unobserved` 或 `deferred`。下一增量必须先新增双语 Necessity Record。
