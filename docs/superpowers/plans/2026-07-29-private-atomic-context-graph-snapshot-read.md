# Private Atomic ContextGraph Snapshot Read / 私有原子 ContextGraph Snapshot 读取

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** The private persisted
three-way ContextGraph review must read base, left, and right snapshots from one consistent
repository boundary before version-backed classification. This directly advances Criteria 2 and 4:
replayable version history and one shared Context Graph contract must not silently combine facts from
different database views. It preserves the charter's reusable Rust core, exact immutable scope,
fail-closed storage, and `GraphDiff::between` as the sole graph-diff calculator.

**完成条件与章程原则：** 私有持久化三路 ContextGraph review 在进行 version-backed classification 前，必须从一个一致的
repository boundary 读取 base、left、right snapshot。本增量直接推进条件 2 与 4：可回放版本历史与统一 Context Graph
contract 不能静默组合来自不同数据库视图的事实。它保持章程要求的可复用 Rust core、精确 immutable scope、fail-closed
storage，以及 `GraphDiff::between` 作为唯一 graph-diff calculator。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The exact snapshot domain,
merge-base plan, conflict classifier, version-bound review contract, and storage review bridge are
already locally verified. The bridge still invokes the single-snapshot repository method three
times; each PostgreSQL call has its own read transaction. A concurrent write or snapshot refresh
could therefore produce a mixed base/left/right view. No batch read port currently expresses or tests
this consistency boundary.

**未满足依赖、风险或证据缺口：** exact snapshot domain、merge-base plan、conflict classifier、version-bound review contract
与 storage review bridge 均已在本地验证。当前 bridge 仍调用三次 single-snapshot repository method；每次 PostgreSQL call
都有自己的 read transaction。因此并发写入或 snapshot refresh 可能产生混合的 base/left/right view。当前没有 batch read
port 来表达或测试这一一致性 boundary。

**Why now / 为何现在优先:** This is the closest remaining local reliability gap in the already
admitted ContextGraph comparison path. It is smaller and more necessary than a merge writer,
rollback, public transport, or UI mutation, all of which depend on a trustworthy read set. It also
closes an explicitly documented deferred consistency caveat without requiring Docker, PostgreSQL
runtime, external evidence, or production access.

**为何现在优先：** 这是现有已准入 ContextGraph comparison path 中最近且最直接的 local reliability gap。相比 merge writer、
rollback、public transport 或 UI mutation，它更小且更必要，因为这些能力都依赖可信的 read set。它还可以收束文档中
明确记录的 deferred consistency caveat，而无需 Docker、PostgreSQL runtime、external evidence 或 production access。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Own only
`crates/storage/src/commit_graph_snapshot.rs`, `crates/storage/src/context_merge_review.rs`, the
Memory/PostgreSQL repository implementations, focused storage contract tests, this plan, and the
bilingual roadmap/architecture receipt. Keep the `versioning`, `diff-engine`, API, SDK, Web, CLI,
Desktop, migration, and public OpenAPI boundaries unchanged.

**明确非目标：** No merge result, conflict resolution, branch mutation, rollback, schema migration,
REST/OpenAPI/SDK method, Web mutation, provider, raw graph-content transport, Docker/runtime,
production claim, secret access, or second GraphDiff calculator. The batch port improves read
consistency only; it is not PostgreSQL runtime evidence or production readiness.

**明确非目标：** 不实现 merge result、conflict resolution、branch mutation、rollback、schema migration、REST/OpenAPI/SDK
method、Web mutation、provider、raw graph-content transport、Docker/runtime、production claim、secret access 或第二个
GraphDiff calculator。batch port 只改善 read consistency，不是 PostgreSQL runtime evidence 或 production readiness。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Add red
tests for typed read-set identity/order, Memory single-guard behavior, missing-side fail-closed
mapping, and a repository double proving the review service invokes the batch method exactly once.
Add a PostgreSQL ignored integration test only if its existing harness supports it, and label it
unobserved when Docker/psql are unavailable. Then run focused storage/diff tests, format, workspace
Rust tests, strict offline Clippy, locked Rust check, Web checks, and the static GraphDiff/public
surface searches.

**下一增量前的新鲜验证：** 先增加 typed read-set identity/order、Memory single-guard behavior、missing-side fail-closed
mapping，以及 repository double 证明 review service 只调用一次 batch method 的红测。只有现有 harness 支持时才增加
PostgreSQL ignored integration test；Docker/psql 不可用时明确标记 unobserved。随后运行 focused storage/diff test、format、
workspace Rust test、strict offline Clippy、锁定 Rust check、Web check 与 GraphDiff/public surface static search。

## Implementation Checklist / 实施清单

- [x] Add a validated base/left/right read-set and complete-result contract with deterministic side order.
- [x] Implement one-guard Memory and one-transaction PostgreSQL reads; preserve existing single-read compatibility.
- [x] Make persisted merge review use the batch contract exactly once and keep side-aware redacted errors.
- [x] Record fresh local evidence without claiming PostgreSQL runtime or production readiness.

- [x] 增加带校验的 base/left/right read-set 与 complete-result contract，并保持确定性的 side order。
- [x] 实现 one-guard Memory 与 one-transaction PostgreSQL read；保持既有 single-read compatibility。
- [x] 让 persisted merge review 只调用一次 batch contract，并保持 side-aware 脱敏 error。
- [x] 记录新鲜 local evidence，但不声称 PostgreSQL runtime 或 production readiness。

## Implementation Receipt / 实施回执

The batch read contract is implemented without widening the transport boundary. The storage
repository port now accepts a validated `ContextMergeInputScope` and returns base/left/right in a
fixed tuple order. Existing repository doubles remain compatible through the default implementation;
the in-memory adapter reads the three snapshots under one guard, while PostgreSQL reads them in one
`REPEATABLE READ READ ONLY` transaction. `PersistedContextGraphMergeReviewService` invokes the
batch port exactly once, validates every returned scope, maps missing sides to typed redacted errors,
and delegates the version-bound comparison to the existing classifier.

该 batch read contract 已在不扩大 transport boundary 的前提下实现。storage repository port 接收经过校验的
`ContextMergeInputScope`，并以固定 tuple 顺序返回 base/left/right。通过 default implementation 保持既有 repository
double 兼容；Memory adapter 在一个 guard 下读取三份 snapshot，PostgreSQL 在一个
`REPEATABLE READ READ ONLY` transaction 中读取三份 snapshot。`PersistedContextGraphMergeReviewService` 只调用一次
batch port，校验每个返回 scope，将 missing side 映射为 typed redacted error，并委托既有 classifier 完成 version-bound
comparison。

Fresh local verification observed focused merge-review storage tests `8 passed`, workspace Rust
tests with API `183 passed` and storage `186 passed, 39 ignored`, `cargo fmt --all -- --check`,
strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web` with public SDK `15`, local
SDK `92`, Web `188`, and the production Web build, plus static `impl GraphDiff` count `1`.
PostgreSQL runtime, authenticated browser, Git change-set, remote CI, operator rehearsal, release,
and production remain `unobserved` or `deferred`; no secret was read and no public API, SDK,
OpenAPI, Web mutation, or second GraphDiff calculator was added. The long-term goal remains active.

新鲜本地验证观测到 focused merge-review storage test `8 passed`、workspace Rust test（API `183 passed`、storage
`186 passed, 39 ignored`）、`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`（public SDK `15`、local SDK `92`、Web `188` 与 production Web build），以及 static `impl GraphDiff`
count `1`。PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与
production 仍为 `unobserved` 或 `deferred`；未读取 secret，也未新增 public API、SDK、OpenAPI、Web mutation 或第二个
GraphDiff calculator。长期目标保持 active。

The next increment must begin with its own bilingual Necessity Record. The version-bound
ContextGraph review/application contract is already recorded separately; the next candidate must
be selected from the remaining dependency-ready private Context editing, benchmark, or replay gaps.
No merge writer, rollback, public transport, or production claim is admitted by this receipt.

下一增量必须先拥有自己的双语 Necessity Record。version-bound ContextGraph review/application contract 已在独立回执中
记录；下一候选必须从剩余且依赖就绪的 private Context editing、benchmark 或 replay 缺口中选择。本回执不准入 merge
writer、rollback、public transport 或 production claim。
