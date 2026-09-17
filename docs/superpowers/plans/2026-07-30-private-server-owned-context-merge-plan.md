# Private Server-Owned Context Merge Plan / 私有服务端拥有的 Context Merge Plan

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与章程原则:** This increment directly advances
Criteria 2 and 4: versioned Context history must resolve ancestry from the server-owned exact
Context commit DAG before graph review. `GraphDiff::between` remains behind the existing graph
classifier and is not reimplemented.

本增量直接推进条件 2 与条件 4：版本化 Context history 必须在 graph review 前从服务端拥有的 exact
Context commit DAG 解析 ancestry。`GraphDiff::between` 继续隐藏在既有 graph classifier 后，不重新实现。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The reusable
`ContextCommitGraphRepository` now validates and loads the complete DAG, but the persisted merge
review service still accepts a caller-supplied `MergePlan`. That leaves a future consumer able to
pair exact snapshots with unverified ancestry.

可复用的 `ContextCommitGraphRepository` 已能校验并加载完整 DAG，但 persisted merge review service
仍接受 caller-supplied `MergePlan`。这会使未来 consumer 可能把 exact snapshot 与未经验证的 ancestry 配对。

**Why now / 为什么现在优先:** This is the smallest direct consumer of the just-verified ancestry
port and the existing version-bound classifier. It closes the nearest integration gap before any
transport, editor, merge writer, or benchmark breadth.

这是刚验证的 ancestry port 与既有 version-bound classifier 的最小直接 consumer。在任何 transport、editor、
merge writer 或 benchmark breadth 之前，它能收束最近的集成缺口。

**Smallest boundary and bilingual documentation / 最小受影响边界与双语文档:** Add a typed
Context-plus-two-tips scope and a private service method in `crates/storage/src/context_merge_review.rs`.
Resolve `MergePlan` through `ContextCommitGraphRepository`, then delegate the resulting exact
three-way review through the existing snapshot batch and `VersionedContextGraphMergeReviewService`.
Add focused storage contract tests and update this plan plus the bilingual roadmap receipts.

增加 typed Context-plus-two-tips scope，以及 `crates/storage/src/context_merge_review.rs` 中的私有 service method。
通过 `ContextCommitGraphRepository` 解析 `MergePlan`，再通过既有 snapshot batch 与
`VersionedContextGraphMergeReviewService` 委托 exact three-way review。增加 focused storage contract tests，
并更新本计划与双语 roadmap 回执。

**Explicit non-goals / 明确非目标:** No REST/OpenAPI/SDK method, Web/CLI/Desktop surface,
merge writer, branch mutation, rollback, persistence schema, provider call, raw private-content
projection, secret access, Docker/runtime, browser, release, production claim, or second
GraphDiff calculator.

不新增 REST/OpenAPI/SDK method、Web/CLI/Desktop surface、merge writer、branch mutation、rollback、persistence schema、
provider call、raw private-content projection、secret access、Docker/runtime、browser、release、production 声明或第二个
GraphDiff calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** First
observe a red focused test for the missing server-owned method. Then run focused storage merge-review
tests, workspace Rust tests, format, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`,
and `GRAPH_DIFF_IMPL_COUNT=1` plus public-surface checks. PostgreSQL runtime, browser, Git, remote CI,
operator rehearsal, release, and production remain separately unobserved or deferred.

先观察 focused test 因 server-owned method 缺失而红，再运行 storage merge-review focused tests、workspace Rust tests、format、
strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`、`GRAPH_DIFF_IMPL_COUNT=1` 与 public-surface checks。
PostgreSQL runtime、browser、Git、remote CI、operator rehearsal、release 与 production 继续单独标记为 unobserved 或 deferred。

## Implementation Checklist / 实施清单

- [x] Add the typed tip scope and server-owned plan resolution method.
- [x] Add focused red/green storage contract coverage for exact DAG ancestry and non-three-way outcomes.
- [x] Run fresh scope-matched verification and update bilingual roadmap receipts.

- [x] 增加 typed tip scope 与 server-owned plan resolution method。
- [x] 增加 exact DAG ancestry 与 non-three-way outcome 的 focused 红绿 storage contract coverage。
- [x] 运行范围匹配的新鲜验证并更新双语 roadmap 回执。

## Implementation Receipt / 实施回执

`ContextMergeTipScope` now binds one project, Context, and two distinct typed branch tips.
`PersistedContextGraphMergeReviewService::review_server_owned` loads the exact Context DAG,
resolves `MergePlan` server-side, rejects unknown tips and non-three-way outcomes, and then
reuses the existing exact snapshot batch plus `VersionedContextGraphMergeReviewService`.
The pre-existing caller-supplied review method remains intact for compatibility; no transport
consumes either method yet.

`ContextMergeTipScope` 现绑定一个 project、Context 与两个不同的 typed branch tip。
`PersistedContextGraphMergeReviewService::review_server_owned` 读取 exact Context DAG，在服务端解析 `MergePlan`，拒绝 unknown tip
与 non-three-way outcome，然后复用既有 exact snapshot batch 与 `VersionedContextGraphMergeReviewService`。既有 caller-supplied review
method 保持不变以兼容现有调用；目前没有 transport 消费任一方法。

Fresh local verification passed: the focused storage merge-review contract reported `11 passed`;
workspace Rust reported `208 passed, 39 ignored` in storage; format, strict offline Clippy, and
locked Rust `1.85.0` passed; `pnpm check:web` passed with public SDK `15`, local SDK `117`, Web
`245`, TypeScript/lint, and production build; the public OpenAPI exclusion test passed (`1`); and
`GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL runtime, browser, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`.

新鲜本地验证通过：storage merge-review focused contract 为 `11 passed`；workspace Rust storage 为 `208 passed, 39 ignored`；format、strict
offline Clippy 与锁定 Rust `1.85.0` 通过；`pnpm check:web` 通过（public SDK `15`、local SDK `117`、Web `245`、TypeScript/lint 与
production build）；public OpenAPI exclusion test 通过（`1`）；`GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL runtime、browser、Git、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。
