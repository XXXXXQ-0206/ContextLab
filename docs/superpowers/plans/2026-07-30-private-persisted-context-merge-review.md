# Private Persisted Context Merge Review / 私有持久化 Context Merge Review

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criteria 1, 2, 4, and 5: Context Graph snapshots must be replayable from durable exact commit
identity, versioned comparisons must use reusable Rust application boundaries, and storage must
remain the source of truth rather than request-time fixtures. The review must preserve the sole
`GraphDiff::between` calculator and the Context-first graph backbone.

本增量直接推进条件 1、2、4、5：Context Graph snapshot 必须能从 durable exact commit identity 回放，versioned
comparison 必须通过可复用 Rust application boundary 完成，storage 必须是事实来源而非 request-time fixture。review
必须保持 `GraphDiff::between` 为唯一 calculator 与 Context-first graph backbone。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The diff-engine now
classifies an exact in-memory three-way set, while storage exposes only a single-snapshot read
port. There is no private service that reads the three exact persisted snapshots together, rejects
missing or scope-drifted records, and delegates the comparison without substituting current head
or caller-built graph data.

当前 diff-engine 已能 classification exact in-memory three-way set，但 storage 目前只有 single-snapshot read port。
尚无 private service 能一次读取三个 exact persisted snapshot，拒绝 missing 或 scope drift，并在不替换为 current head
或 caller-built graph data 的前提下委托 comparison。

**Why now / 为何现在优先:** The repository contract and pure classifier are both locally verified,
so this is the smallest dependency-ready bridge before any merge/rollback writer. It is read-only,
uses the existing Memory/PostgreSQL repository boundary, and closes a named persistence-to-review
gap without external runtime evidence.

repository contract 与 pure classifier 均已完成本地验证，因此这是任何 merge/rollback writer 前最小的依赖就绪 bridge。
它只读、复用现有 Memory/PostgreSQL repository boundary，并直接收束 persistence-to-review 缺口，不依赖 external runtime
evidence。

**Contract / 契约:** `ContextMergeInputScope` contains one ProjectId, one ContextId, and three
distinct base/left/right CommitIds. `PersistedContextGraphMergeReviewService` reads each exact
scope once, fails closed for missing or returned-scope drift, and delegates the exact snapshots to
`GraphMergeConflictClassifier`. No caller-supplied graph, current-head lookup, merged graph, or
write operation is accepted.

**Boundary and bilingual documentation / 边界与双语文档:** Own only
`crates/storage/src/context_merge_review.rs`, its `lib.rs` export, and focused storage tests. Update
this plan plus bilingual architecture, active-goal, completion-criteria, and parallel-development
receipts after fresh verification. Do not edit migrations, transport, OpenAPI, SDK, Web, CLI, Desktop,
or provider files.

**Explicit non-goals / 明确非目标:**

- No merge writer, rollback, branch-head mutation, CAS, idempotency, persistence schema, migration,
  or public/local transport.
- No caller-supplied snapshots, current-head substitution, content auto-resolution, merged graph,
  provider call, Docker/PostgreSQL runtime, browser, secrets, remote CI, operator, release, or production claim.
- No second GraphDiff calculator.

- 不实现 merge writer、rollback、branch-head mutation、CAS、idempotency、persistence schema、migration 或 transport。
- 不接受 caller-supplied snapshot，不替换为 current head，不做 content auto-resolution，不生成 merged graph，不调用
  provider，也不做 Docker/PostgreSQL runtime、browser、secrets、remote CI、operator、release 或 production claim。
- 不新增第二个 GraphDiff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** The
focused contract tests must cover exact three-side delegation, missing-side failure, returned
scope drift, duplicate commit rejection, classifier delegation, and non-three-way plan rejection.
Then run focused storage tests, format, workspace Rust, strict offline Clippy, locked Rust `1.85.0`,
Web checks, and static GraphDiff/public-surface searches. PostgreSQL runtime remains unobserved
while Docker/virtualization is unavailable. The implementation receipt below supersedes this
admission-time requirement.

## Implementation Checklist / 实施清单

- [x] Add typed exact three-snapshot scope and read-only review service.
- [x] Add focused Memory contract and fail-closed delegation tests.
- [x] Export only the private Rust contract; keep mutation and transport unchanged.
- [x] Record fresh bilingual architecture/roadmap evidence and choose the next dependency-ready increment.

- [x] 增加 typed exact three-snapshot scope 与只读 review service。
- [x] 增加 focused Memory contract 与 fail-closed delegation tests。
- [x] 仅导出 private Rust contract；保持 mutation 与 transport 不变。
- [x] 记录新鲜双语 architecture/roadmap evidence，并选择下一项依赖就绪增量。

## Implementation Receipt / 实施回执

The private bridge is implemented in `crates/storage/src/context_merge_review.rs`, exported
through the storage crate, and covered by `crates/storage/tests/context_merge_review.rs`. The
service validates one project/Context and three distinct commit identities, rejects non-three-way
plans before repository reads, reads the exact base/left/right snapshot scopes, rejects missing or
scope-drifted records, and delegates classification only to
`GraphMergeConflictClassifier`. The focused Memory contract has 5 passing tests covering exact
three-side delegation, missing-side failure, duplicate commit rejection, repository scope drift,
and non-three-way rejection. It remains a private read-only Rust boundary: no merge writer,
transport, migration, public SDK/OpenAPI route, Web mutation, or second `GraphDiff` calculator was
added.

Serde deserialization also routes through the same constructor, so JSON cannot bypass the
distinct/non-nil scope invariants; nil identifier errors identify the project, Context, or commit
family that failed. Repository reads short-circuit on the first failed side.

Serde deserialization 也会经过同一个 constructor，因此 JSON 不能绕过 distinct/non-nil scope invariant；nil identifier
error 会指出失败的 project、Context 或 commit family。repository read 会在第一个失败 side 处 short-circuit。

该 private bridge 已实现于 `crates/storage/src/context_merge_review.rs`，并通过 storage crate 导出，由
`crates/storage/tests/context_merge_review.rs` 覆盖。service 校验一个 project/Context 与三个不同 commit identity，
在 repository read 前拒绝非 three-way plan，读取 exact base/left/right snapshot scope，拒绝 missing 或 scope drift，
且只委托 `GraphMergeConflictClassifier` 完成 classification。focused Memory contract 共 5 项通过，覆盖三侧 exact
委托、missing-side failure、duplicate commit rejection、repository scope drift 与 non-three-way rejection。本增量仍是
private read-only Rust boundary；未新增 merge writer、transport、migration、public SDK/OpenAPI route、Web mutation 或
第二个 `GraphDiff` calculator。

The PostgreSQL adapter currently satisfies the same single-snapshot repository port, so this
service performs three separate exact reads rather than one atomic batch transaction. Cross-read
consistency and a future transactional batch port are explicitly deferred; this receipt does not
claim PostgreSQL runtime evidence or atomic multi-read semantics.

PostgreSQL adapter 当前满足同一个 single-snapshot repository port，因此本 service 执行三次独立的 exact read，而不是
一次 atomic batch transaction。跨 read consistency 与未来 transactional batch port 明确 deferred；本回执不声称已取得
PostgreSQL runtime evidence，也不声称具备 atomic multi-read 语义。
