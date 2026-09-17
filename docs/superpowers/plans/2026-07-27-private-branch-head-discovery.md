# Private Typed Branch-Head Discovery / 私有 Typed Branch-Head Discovery

**Status / 状态:** completed and verified locally / 已完成，已取得本地验证

## Necessity Record / 必要性记录

### Criterion and charter principle / 完成条件与宪章原则

This increment serves the Context-first, replayable version-history, and future branch/merge/replay
completion criteria. It establishes a reusable Rust read contract for the durable fact already used by
guarded Context commits: a typed branch name, exact Context ownership, optional head commit, and
monotonic branch revision. It protects the charter's rule that domain and persistence semantics live
in reusable Rust rather than in UI or transport code.

本增量服务 Context-first、可回放版本历史与未来 branch/merge/replay 的完成条件。它为 guarded Context commit 已经
使用的 durable fact 建立可复用 Rust read contract：typed branch name、精确 Context ownership、可为空的 head commit
与单调 branch revision。它维护领域与持久化语义必须位于可复用 Rust，而不是 UI 或 transport 的宪章原则。

### Gap and dependencies / 缺口与依赖

The database migration `0003_guarded_context_commit_writes.sql` already stores
`context_branches(head_commit_id, revision)`, and migration `0004` enforces that a head belongs to
the same Context. The guarded writer already increments the database revision. The missing contract
is a typed read model and repository port with Memory/PostgreSQL parity, including distinct unknown
Context versus unknown branch failures, stable branch ordering, and fail-closed negative/overflowing
revision handling.

迁移 `0003_guarded_context_commit_writes.sql` 已保存 `context_branches(head_commit_id, revision)`，迁移 `0004` 已
强制 head 属于同一 Context，guarded writer 也已递增 database revision。当前缺口是 typed read model 与 Memory/PostgreSQL
parity repository port，包括可区分的 unknown Context 与 unknown branch failure、稳定 branch ordering，以及对负数/溢出
revision 的 fail-closed 处理。

### Why now / 为什么现在优先

Benchmark/evaluation closure is freshly verified. Branch-head discovery is the smallest dependency-
ready read-only predecessor for later fork/merge/replay work; it uses existing durable state and
does not require a new write policy. Merge itself is explicitly deferred because the guarded writer
rejects merge parents and the repository has no merge-base or conflict contract.

Benchmark/evaluation closure 已取得新鲜验证。Branch-head discovery 是后续 fork/merge/replay 最小的依赖就绪只读前置，
复用现有 durable state，不需要新增 write policy。Merge 本身明确后置，因为 guarded writer 拒绝 merge parents，且
repository 尚无 merge-base 或 conflict contract。

### Explicit non-goals / 明确非目标

- No branch create, fork, rename, delete, merge, rollback, or write operation.
- No migration change; reuse the existing `context_branches` table and constraints.
- No REST, OpenAPI, SDK, BFF, Web, CLI, Desktop, operator, release, production, Docker, or external receipt work.
- No change to guarded commit CAS, idempotency, authorization, audit, or branch-head update semantics.
- No second replay implementation and no second `GraphDiff` calculator.

- 不新增 branch create、fork、rename、delete、merge、rollback 或写入操作。
- 不修改 migration；复用既有 `context_branches` 表与约束。
- 不做 REST、OpenAPI、SDK、BFF、Web、CLI、Desktop、operator、release、production、Docker 或 external receipt 工作。
- 不改变 guarded commit CAS、idempotency、authorization、audit 或 branch-head update 语义。
- 不新增第二套 replay implementation，也不新增第二个 `GraphDiff` calculator。

### Minimal boundary and bilingual documentation / 最小边界与双语文档

Own only a new storage branch-head contract module, its `lib.rs` export, the smallest Memory state
extension needed to retain revision, the PostgreSQL read adapter, and focused storage tests. The
branch-head record must expose typed `ContextId`, `BranchName`, `Option<CommitId>`, and `u64 revision`;
list results must be sorted by `BranchName`. Existing writer tests must remain green.

只拥有新的 storage branch-head contract module、`lib.rs` export、为保存 revision 所需的最小 Memory state extension、
PostgreSQL read adapter 与聚焦 storage tests。branch-head record 必须暴露 typed `ContextId`、`BranchName`、
`Option<CommitId>` 与 `u64 revision`；list result 必须按 `BranchName` 排序。既有 writer test 必须继续通过。

The implementation plan and the active goal, completion criteria, and parallel-development ledger
will record the boundary and evidence bilingually. No API or product documentation is expanded
because this is a private Rust read contract only.

实施计划以及 active goal、completion criteria 与 parallel-development ledger 将以中英双语记录边界与证据。由于这是
private Rust read contract，不扩展 API 或 product documentation。

### Fresh verification required / 下一增量前的新鲜验证

Run focused branch-head Memory and PostgreSQL adapter-contract tests, existing guarded commit tests,
format, storage/full workspace tests, strict offline Clippy, locked Rust `1.85.0` checks, and the
existing Web check to prove no public surface drift. PostgreSQL runtime remains unobserved when Docker
and virtualization are unavailable.

运行 focused branch-head Memory 与 PostgreSQL adapter-contract tests、既有 guarded commit tests、format、storage/full
workspace tests、strict offline Clippy、锁定 Rust `1.85.0` check 与既有 Web check，以证明没有 public surface drift。
当 Docker 与 virtualization 不可用时，PostgreSQL runtime 继续保持 unobserved。

## Implementation Checklist / 实施清单

- [x] Add the typed branch-head domain record, read port, and structured errors.
- [x] Add Memory revision parity without changing guarded writer semantics.
- [x] Add PostgreSQL exact Context/branch reads with stable ordering and fail-closed conversion.
- [x] Add focused tests for known/unknown Context, known/unknown branch, unborn head, revision,
  same-Context head integrity, and deterministic ordering.
- [x] Run fresh verification and update bilingual roadmap evidence; keep the long-term goal active.

- [x] 增加 typed branch-head domain record、read port 与结构化错误。
- [x] 增加 Memory revision parity，不改变 guarded writer 语义。
- [x] 增加 PostgreSQL exact Context/branch read、稳定排序与 fail-closed conversion。
- [x] 增加 known/unknown Context、known/unknown branch、unborn head、revision、same-Context head integrity 与确定性
  ordering 的 focused tests。
- [x] 运行新鲜验证并更新双语路线图证据；保持长期目标 active。

## Local Receipt / 本地回执

The typed branch-head contract is locally verified. Memory now stores nullable heads so an unborn
branch can be represented like PostgreSQL, exact branch reads resolve only the requested row, and
both adapters sort rehydrated typed names deterministically. The writer still owns branch-head CAS
and revision updates; this increment adds no branch mutation or transport.

typed branch-head contract 已在本地验证。Memory 现以 nullable head 表达与 PostgreSQL 一致的 unborn branch，exact
branch read 只解析请求行，两种 adapter 对 rehydrated typed name 使用确定性排序。writer 仍唯一负责 branch-head CAS
与 revision 更新；本增量不新增 branch mutation 或 transport。

Fresh focused evidence includes integration `branch_head` (`2 passed`) and Memory adapter
regressions for populated/unborn ordering and revision parity (`1 passed`), exact-read isolation
(`1 passed`), cross-Context integrity and overflow (`1 passed`), plus `cargo fmt --all -- --check`.
Fresh full receipts are storage `186 passed, 39 ignored`, workspace Rust `186 passed, 39 ignored`,
strict offline Clippy, locked Rust `1.85.0` check, and `pnpm check:web` with public SDK `15`, local
SDK `92`, Web `185`, and production build passed.

新鲜全量回执为 storage `186 passed, 39 ignored`、workspace Rust `186 passed, 39 ignored`、strict offline Clippy、
锁定 Rust `1.85.0` check，以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185`、production build）通过。

新鲜 focused evidence 包括 integration `branch_head`（`2 passed`）、Memory adapter 对 populated/unborn ordering
与 revision parity 的回归（`1 passed`）、exact-read isolation（`1 passed`）、cross-Context integrity 与 overflow
（`1 passed`），以及 `cargo fmt --all -- --check`。后续 workspace verification 继续作为本计划的权威全量回执。

## Evidence Boundary / 证据边界

Local contract tests and builds are non-production evidence only. No secrets are read. PostgreSQL
runtime, authenticated browser, Git change-set, remote CI, operator rehearsal, release, and production
remain `unobserved` or `deferred` unless directly observed.

本地 contract tests 与 builds 仅是非生产证据。不读取 secrets。除非直接观测，PostgreSQL runtime、authenticated
browser、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。
