# Private Atomic Merge Review Witness / 私有原子 Merge Review Witness

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly serves Criteria 2 (replayable version history) and Criterion 4
(Context Graph as the system skeleton). A server-owned three-way merge review must bind the
resolved `MergePlan` and its immutable base/left/right Context Graph snapshots to one read
observation. `GraphDiff::between` remains the sole graph-diff calculator.

本增量直接服务条件 2（可回放版本历史）与条件 4（Context Graph 作为系统骨架）。server-owned three-way merge review 必须在同一个
read observation 中绑定已解析的 `MergePlan` 与 immutable base/left/right Context Graph snapshot。`GraphDiff::between` 仍是唯一的图差异计算器。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

`PersistedContextGraphMergeReviewService::review_server_owned` currently loads the complete
commit DAG and resolves `MergePlan` first, then performs a separate batch snapshot read. A
concurrent commit/snapshot change can therefore pair plan state N with snapshot state N+1. The
missing evidence is a private repository witness that returns the plan and all three snapshots
from one Memory read guard or one PostgreSQL read transaction, plus a regression proving the
server-owned service uses that port exactly once.

当前 `PersistedContextGraphMergeReviewService::review_server_owned` 先加载完整 commit DAG 并解析 `MergePlan`，再执行独立的 batch
snapshot read。并发 commit/snapshot 变化可能让 plan state N 与 snapshot state N+1 被错误配对。缺失证据是：private repository witness
必须在一次 Memory read guard 或一次 PostgreSQL read transaction 中返回 plan 与三份 snapshot，并通过 regression 证明 server-owned
service 只调用该 port 一次。

### Why now / 为什么现在优先

Normal first-parent ancestry, commit-associated snapshots, parent-snapshot admission, and the
atomic branch-head witness are already locally verified. This is the smallest remaining local
consistency gap before trustworthy merge review; it is more necessary than adding another
transport or UI surface and does not depend on deferred external release evidence.

normal first-parent ancestry、commit-associated snapshot、parent-snapshot admission 与 atomic branch-head witness 已完成本地验证。
这是可信 merge review 之前最小的本地一致性缺口，比增加 transport 或 UI surface 更必要，并且不依赖延期的外部发布证据。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/SDK route or method, Web/CLI/Desktop mutation, merge writer, branch mutation, rollback, migration, provider, or operator transport.
- No second graph-diff implementation, no change to the existing classifier, and no production or release-readiness claim.
- No Docker/PostgreSQL runtime, browser, remote CI, operator, Git, secret, release, or production evidence claim.

- 不新增 public REST/OpenAPI/SDK route 或 method、Web/CLI/Desktop mutation、merge writer、branch mutation、rollback、migration、provider 或 operator transport。
- 不新增第二个 graph-diff 实现，不改变既有 classifier，也不作 production 或 release-readiness 声明。
- 不宣称 Docker/PostgreSQL runtime、browser、remote CI、operator、Git、secret、release 或 production 证据。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

Add one private storage witness contract in `context_merge_review`, implement it in the existing
Memory and PostgreSQL repositories, refactor only the server-owned merge review adapter, add
focused storage tests, and record the result in this bilingual plan, architecture notes, and the
roadmap ledger. Existing explicit-scope batch review remains source-compatible.

只在 `context_merge_review` 增加一个 private storage witness contract，在现有 Memory 与 PostgreSQL repository 中实现它，只重构
server-owned merge review adapter，补 focused storage tests，并在本双语计划、架构说明与 roadmap ledger 记录结果。现有 explicit-scope
batch review 保持 source-compatible。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First observe a red contract test requiring the atomic witness port. Then run focused merge-review
tests, PostgreSQL SQL-shape tests without a live database claim, formatting, offline workspace
tests, strict offline Clippy, locked Rust 1.85 verification, local contract verification, and
the exact-one `GraphDiff` implementation check. Docker/PostgreSQL runtime, browser, Git, remote
CI, operator, release, and production remain ignored, unobserved, or deferred unless real new
evidence exists.

先观测要求 atomic witness port 的红测，再运行 merge-review focused tests、无 live database 声明的 PostgreSQL SQL-shape tests、格式检查、
offline workspace tests、strict offline Clippy、锁定 Rust 1.85 验证、local contract verification 与 exact-one `GraphDiff` implementation
check。除非出现真实新证据，Docker/PostgreSQL runtime、browser、Git、remote CI、operator、release 与 production 继续标记为 ignored、
unobserved 或 deferred。

## Implementation Checklist / 实施清单

- [x] Add the private plan-plus-three-snapshots witness contract and red/green regressions.
- [x] Implement one Memory read-guard witness and one PostgreSQL read-transaction witness.
- [x] Delegate server-owned merge review to the existing classifier without adding a diff path.
- [x] Run fresh verification and update the bilingual roadmap and architecture receipts.

- [x] 增加 private plan-plus-three-snapshots witness contract 与红绿 regression。
- [x] 实现一次 Memory read-guard witness 与一次 PostgreSQL read-transaction witness。
- [x] 让 server-owned merge review 委托既有 classifier，不增加 diff path。
- [x] 运行新鲜验证并更新双语 roadmap 与架构回执。

## Status / 状态

`completed / verified locally`; the long-term goal remains `active`. / `completed / verified locally`；长期目标保持 `active`。

The witness validates the request tip scope in the application service, constructs and validates
the PostgreSQL witness before committing its single read-only transaction, and keeps missing API
dependencies unavailable rather than falling back to preview data. The live PostgreSQL runtime
test remains ignored because no disposable database URL was provided. / witness 在 application service
重新校验请求 tip scope；PostgreSQL 在提交单一只读 transaction 前构造并校验 witness；API 依赖缺失时保持 unavailable，不回退到 preview
data。由于未提供 disposable database URL，live PostgreSQL runtime test 继续 ignored。
