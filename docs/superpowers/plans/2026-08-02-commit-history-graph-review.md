# Commit History to Context Graph Review / 提交历史到 Context Graph 审阅

## Necessity Record / 必要性记录

**Primary criterion / 主要条件:** **Criterion 2 - replayable version history.** The exact
`CommitHistory` and read-only `BranchHeads` must be usable as one deterministic review scope.
This increment advances Criterion 2 only; it does not close it or the long-term goal.

**Secondary criterion / 次要条件:** **Criterion 4 - reviewable semantic/versioned change.** The
existing version-backed Context Graph review must receive a server-owned baseline/revised pair
selected from the same Context history, while `GraphDiff::between` remains the sole calculator.

**Gap / 缺口:** The versioning crate already validates `CommitHistory` and explicit branch heads,
and the private graph review already accepts an exact commit pair. The remaining gap is the
read-only composition between them: no single review input currently binds the selected
`BranchHeads` result, deterministic `CommitHistory`, and the existing pair witness. Without that
binding, history inspection and graph review can drift or require manually repeated commit IDs.

**Why now / 为什么现在优先:** The dependency contracts are already present and locally tested:
history validation, branch-head discovery, exact-scope snapshot reads, and version-backed graph
diff review. This is the smallest next increment that makes those facts useful together before
any branch mutation, merge, rollback, provider, benchmark, or new public consumer is considered.

**Explicit non-goals / 明确非目标:**

- No public REST route, OpenAPI operation, or public SDK method.
  / 不新增 public REST route、OpenAPI operation 或 public SDK method。
- No Web mutation, commit creation, branch-head mutation, merge, rollback, or persistence
  migration. / 不新增 Web mutation、commit creation、branch-head mutation、merge、rollback 或 persistence migration。
- No provider call, secret or credential access, or raw content exposure.
  / 不新增 provider call、secret 或 credential access，也不暴露 raw content。
- No Docker/PostgreSQL runtime claim, operator rehearsal, release, production, remote CI, or
  authenticated-browser claim. / 不声称 Docker/PostgreSQL runtime、operator rehearsal、release、production、remote CI 或 authenticated-browser evidence。
- No second `GraphDiff` implementation. `GraphDiff::between` remains the sole graph-diff
  calculator. / 不新增第二个 `GraphDiff` 实现；`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Minimal ownership / 最小 ownership:** The implementation, if separately admitted, owns only
the existing Rust versioning/storage read boundary and its focused contract tests, plus the
existing private read-only Web graph-review composition and tests needed to pass an exact selected
head. Reuse `CommitHistory`, `ContextBranchHead`/`BranchHeads`, the existing version-backed review
service, pair witness, transport/parser, and shared review primitives. This worker owns only this
bilingual plan file and must not edit those implementation files.

## Implementation Checklist / 实施清单

- [x] Read one exact `(project, context)` scope and compose deterministic `CommitHistory` with
      read-only `BranchHeads`; fail closed for foreign scope, unknown head, incomplete ancestry,
      duplicate branch, or invalid pair.
      / 读取一个精确的 `(project, context)` scope，将确定性的 `CommitHistory` 与只读 `BranchHeads` 组合；对跨 scope、未知 head、缺失 ancestry、重复 branch 或无效 pair fail closed。
- [~] Map a selected non-null branch head to the existing server-owned revised commit candidate;
      preserve unborn/null-head behavior and require a distinct exact baseline commit. This
      explicit branch-selection policy is **resolved as deferred** rather than implemented on the
      transport: the reusable storage primitive `review_branch_head` now resolves a branch name to
      its exact head with `BranchUnknown`/`BranchUnborn`/`TargetNotBranchHead` fail-closed errors,
      while the protected graph-diff route deliberately keeps requiring explicit
      `original_commit_id` and `revised_commit_id` so no client can infer a review pair from a
      moving head.
      / 将选中的非 null branch head 映射为既有 server-owned revised commit candidate；保留 unborn/null head 行为，并要求不同的 exact baseline commit。该显式分支选择策略**结案为 deliberate deferral，而非在 transport 上实现**：可复用 storage primitive `review_branch_head` 现已按 branch name 解析精确 head，并以 `BranchUnknown`/`BranchUnborn`/`TargetNotBranchHead` fail closed；同时 protected graph-diff route 仍刻意要求显式 `original_commit_id` 与 `revised_commit_id`，使任何 client 都无法从会移动的 head 推断 review pair。
      Evidence / 证据：`crates/storage/src/context_graph_history_review.rs:674` owns
      `review_branch_head`; `server/api/src/routes.rs:2364` keeps the explicit commit-pair query
      and rejects an identical pair before authorization. / `crates/storage/src/context_graph_history_review.rs:674` 拥有
      `review_branch_head`；`server/api/src/routes.rs:2364` 保留显式 commit-pair query，并在 authorization 前拒绝相同 pair。
- [x] Carry the validated history and exact pair identity into the existing version-backed
      Context Graph review without recomputing graph changes in storage, SDK, or Web.
      / 将 validated history 与 exact pair identity 送入既有 version-backed Context Graph review，不在 storage、SDK 或 Web 中重算 graph change。
- [x] Keep the read path private and read-only, reuse existing `data -> presenter -> screen`
      boundaries, and preserve redacted errors, request-memory credentials, and no-store behavior.
      / 保持 path private 且只读，复用既有 `data -> presenter -> screen` boundary，并保持 redacted errors、request-memory credentials 与 no-store 行为。
- [x] Add focused contract coverage for deterministic ordering, complete branch-head/history
      validation, exact pair binding,
      cross-scope rejection, unborn heads, stale/missing commits, and delegation to the sole
      `GraphDiff::between` implementation.
      / 增加 focused contract coverage，覆盖确定性排序、branch-head 到 pair 的绑定、跨 scope 拒绝、unborn head、stale/missing commit，以及委托给唯一 `GraphDiff::between` 实现。

## Fresh Verification / 新鲜验证

- [x] Run the focused versioning history, branch-head, storage pair-review, and Web composition
      tests from the current worktree.
      / 在当前 worktree 运行 focused versioning history、branch-head、storage pair-review 与 Web composition tests。
- [x] Run `cargo fmt --all -- --check`, offline workspace tests, strict offline Clippy, and the
      locked Rust toolchain check; run `pnpm check:web` including TypeScript, Web tests, and build.
      / 运行 Rust format、offline workspace tests、strict offline Clippy、锁定 Rust toolchain check，以及包含 TypeScript、Web tests 与 build 的 `pnpm check:web`。
- [x] Inspect the source boundary: exactly one production `impl GraphDiff`, no public
      REST/OpenAPI/SDK surface for this read, and no mutation path added.
      / 检查 source boundary：production `impl GraphDiff` 恰为一个，本 read 不产生 public REST/OpenAPI/SDK surface，也不新增 mutation path。
- [x] Record exact commands, counts, timestamp, and worktree boundary. Mark unavailable or
      timed-out checks as `unobserved`; do not turn local fixtures into runtime or release evidence.
      / 记录确切命令、数量、时间戳与 worktree boundary；不可用或 timeout 的检查标为 `unobserved`，不得把 local fixture 变成 runtime 或 release evidence。

## Evidence Boundary / 证据边界

Passing focused tests and local workspace checks prove only local contract behavior and source
ownership. They do not prove live Docker/PostgreSQL persistence, authenticated browser behavior,
remote CI, operator approval, release readiness, production safety, or public API compatibility.
The increment remains a private read-only integration and does not close Criterion 2, Criterion 4,
or the active long-term ContextLab goal.

通过 focused tests 与 local workspace checks 只能证明本地 contract behavior 与 source ownership；不能证明 live Docker/PostgreSQL persistence、authenticated browser behavior、remote CI、operator approval、release readiness、production safety 或 public API compatibility。本增量仍是 private、read-only integration，不关闭条件 2、条件 4 或 active long-term ContextLab goal。

## Fresh Receipt / 新鲜回执

Observed on 2026-08-02 in the current worktree: storage history composition `2 passed`,
history-bound graph review `3 passed`, independent storage history/snapshot contract
`2 passed`, workspace Rust `227 passed, 41 ignored`, `cargo fmt --all -- --check`, strict
offline Clippy, locked Rust `1.85.0` check, `pnpm check:web` (`15` public SDK, `148`
local SDK, `298` Web tests, production build), local contract fixture verification, and
`GRAPH_DIFF_IMPL_COUNT=1` all passed. PostgreSQL/Docker runtime, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, and production remain unobserved or
deferred.

2026-08-02 当前 worktree 已观测：storage history composition `2 passed`、history-bound graph
review `3 passed`、独立 storage history/snapshot contract `2 passed`、workspace Rust
`227 passed, 41 ignored`、`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust
`1.85.0` check、`pnpm check:web`（public SDK `15`、local SDK `148`、Web `298`、production
build）、local contract fixture verification 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。PostgreSQL/Docker
runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与
production 仍为 unobserved 或 deferred。
