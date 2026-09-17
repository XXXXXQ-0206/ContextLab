# Private Persisted Diff Review / 私有持久化 Diff Review

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** The service is complete when one
local, private Rust application boundary accepts two distinct exact
`(ProjectId, ContextId, CommitId)` scopes, reads both immutable
`ContextDiffSnapshotV1Record` values through `ContextDiffSnapshotV1Repository`, rejects any missing or
cross-scope fact without a partial result, and delegates the complete comparison to the existing
`VersionedContextDiffReviewService`. The returned projection must preserve both exact scopes and all
semantic, behavior, and evaluation results. It must not accept caller-supplied snapshots.

本服务的完成条件是：建立一个 local、private 的 Rust application boundary，接收两个不同的 exact
`(ProjectId, ContextId, CommitId)` scope，通过 `ContextDiffSnapshotV1Repository` 读取两份 immutable
`ContextDiffSnapshotV1Record`；对缺失或 scope 不一致的事实 fail closed，绝不返回 partial result，并将完整比较
委托给既有 `VersionedContextDiffReviewService`。返回 projection 必须保留两个 exact scope 以及 semantic、behavior、
evaluation 全部结果，且不得接受调用方自带 snapshot。

This directly advances Completion Criterion 2, replayable version history, and Criterion 4, Context
Graph and reviewability as a system backbone. It follows the charter's Context-first, reusable Rust
core, explicit contract, clean architecture, and secure-by-default principles. It does not claim
Criterion 6 access governance until a separately admitted authenticated boundary exists.

本增量直接推进完成条件 2“可回放版本历史”和完成条件 4“Context Graph 与 reviewability 作为系统骨架”。它遵守
章程中的 Context-first、可复用 Rust core、显式契约、clean architecture 与 secure-by-default 原则。在单独准入的
authenticated boundary 建立前，本增量不声称已经满足条件 6 的 access governance。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The typed version-bound review contract
and the immutable exact-commit snapshot repository are adjacent prerequisites, but no service yet
composes them into one persisted-input review operation. Without this composition, a private caller
can still bypass persistence, mix snapshots from different Context scopes, or confuse a storage
read with a trustworthy review. The planned service also depends on the repository's exact-scope
read, schema, payload, and digest validation remaining fail closed.

typed version-bound review contract 与 immutable exact-commit snapshot repository 是相邻前置依赖，但当前还没有一个
service 将它们组合为一次只使用 persisted input 的 review operation。缺少这一层时，private caller 仍可能绕过 persistence、
混用不同 Context scope 的 snapshot，或把一次 storage read 误认为可信 review。该服务还依赖 repository 持续对 exact
scope、schema、payload 与 digest 做 fail-closed 校验。

The composition contract, its focused tests, a disposable PostgreSQL runtime, authenticated
browser behavior, remote CI, Git change-set evidence, operator rehearsal, release receipt, and
production receipt are not evidence for this plan yet. Existing neighboring-plan receipts may be
used as dependency context, not as a substitute for fresh verification of this service. Release and
production receipts are explicitly **deferred/unobserved**; none may be inferred from local tests,
build output, or a plan file.

本组合 contract、其 focused tests、disposable PostgreSQL runtime、authenticated browser behavior、remote CI、Git
change-set evidence、operator rehearsal、release receipt 与 production receipt 目前都不是本计划的证据。相邻计划已有的
回执只能作为 dependency context，不能替代本服务的新鲜验证。release 与 production receipt 明确标记为
**deferred/unobserved**；不得从 local test、build output 或计划文件推断出来。

**Why prioritize now / 为什么现在优先:** Immutable commit-scoped inputs and the pure typed review projection are now
the smallest useful adjacent contracts. The next risk is not another diff feature; it is provenance:
proving that a private review result was calculated from the two stored commit facts named by the
review. Completing this local composition before any adapter or UI prevents caller-payload drift,
keeps replay deterministic, and closes the persistence-to-review gap without expanding a public
surface or introducing a second comparison path.

immutable commit-scoped input 与纯 typed review projection 已经是当前最小且有用的相邻 contract。下一项风险不是增加另一
个 diff feature，而是 provenance：证明 private review result 确实由 review 指定的两份已存储 commit fact 计算而来。
在任何 adapter 或 UI 之前完成这一 local composition，可以防止 caller payload 漂移、保持 replay deterministic，并在不扩展
public surface、也不引入第二条 comparison path 的前提下收束 persistence-to-review 缺口。

**Explicit non-goals / 明确非目标:**

- No public REST route, public OpenAPI operation, or public SDK method.
- No Web mutation, Web inspector, server-rendered public fallback, or UI work.
- No operator transport, operator workflow, or production deployment/configuration.
- No release, release receipt, production claim, production receipt, or external promotion.
- No provider/evaluator/model call, recomputation of evaluation facts, or raw private-content read path.
- No storage migration, write path, commit/branch/merge/rollback policy, graph editing, or auth/RBAC transport policy.
- No second graph-diff calculator. `GraphDiff::between` remains the sole graph-diff calculation used by
  the existing unified diff path; this service only loads persisted facts and delegates to the existing review service.

- 不新增 public REST route、public OpenAPI operation 或 public SDK method。
- 不新增 Web mutation、Web inspector、server-rendered public fallback 或 UI 工作。
- 不新增 operator transport、operator workflow 或 production deployment/configuration。
- 不新增 release、release receipt、production claim、production receipt 或 external promotion。
- 不调用 provider/evaluator/model，不重新计算 evaluation fact，不增加 raw private-content read path。
- 不增加 storage migration、write path、commit/branch/merge/rollback policy、graph editing 或 auth/RBAC transport policy。
- 不增加第二个 graph-diff calculator。`GraphDiff::between` 仍是既有 unified diff path 的唯一 graph-diff calculation；本服务
  只读取 persisted fact，并委托既有 review service。

**Minimal boundary / 最小边界:** Add one storage-owned application service, for example
`PersistedContextDiffReviewService<R>`, in `crates/storage/src/context_diff_review.rs`. Its only
operation accepts an ordered source scope and target scope, validates that both belong to the same
project and Context and are not identical, reads each scope through the repository port, verifies
the returned record still has the requested scope, clones only the validated stored snapshots,
constructs `VersionedContextDiffReviewRequestV1`, and calls
`VersionedContextDiffReviewService::project`. Map source/target read failures to structured,
redacted service errors; never expose database details or return one-sided output.

在 `crates/storage/src/context_diff_review.rs` 增加一份 storage-owned application service，例如
`PersistedContextDiffReviewService<R>`。它只提供一个 operation：接收有序 source scope 与 target scope，校验两者属于同一
project 与 Context 且不相同；通过 repository port 读取每个 scope；再次验证返回 record 仍匹配请求 scope；只 clone 已校验的
stored snapshot；构造 `VersionedContextDiffReviewRequestV1`；最后调用 `VersionedContextDiffReviewService::project`。
将 source/target read failure 映射为结构化且脱敏的 service error；不得暴露 database detail，也不得返回单侧 output。

The only owned source changes are the new service module, its `crates/storage/src/lib.rs` export,
and `crates/storage/tests/context_diff_review.rs`. No schema or migration changes are
needed. `crates/diff-engine` remains unchanged and owns comparison semantics; `server/api`,
`apps/*`, `packages/*`, OpenAPI, SDK, and roadmap documents are outside this worker boundary. This
file is the complete bilingual documentation boundary for the plan; no other docs file is to be
modified.

本计划只拥有新增 service module、`crates/storage/src/lib.rs` export 与
`crates/storage/tests/context_diff_review.rs`。不需要 schema 或 migration 变更。
`crates/diff-engine` 继续拥有 comparison semantics 且不修改；`server/api`、`apps/*`、`packages/*`、OpenAPI、SDK 与
roadmap document 均在本 worker 边界之外。本文件是该计划完整的双语文档边界；不得修改其他 docs 文件。

## Contract Shape / 契约形状

The service input is two scopes only: `(source_scope, target_scope)`. Snapshot payloads are loaded
internally from the immutable repository. The success value is the existing
`VersionedContextDiffReviewProjectionV1`, including its explicit schema version, ordered scopes,
and complete semantic/behavior/evaluation diff. The service must preserve the existing error
boundary for invalid diff input and comparison failure while adding enough side/scope information
to diagnose a missing persisted input without leaking private content.

service input 只有两个 scope：`(source_scope, target_scope)`。snapshot payload 必须在 service 内部从 immutable repository
读取，不能由 caller 传入。成功值复用既有 `VersionedContextDiffReviewProjectionV1`，包含显式 schema version、有序 scope
以及完整 semantic/behavior/evaluation diff。service 必须保留既有 invalid diff input 与 comparison failure error boundary，
并补充足以定位 missing persisted input 的 side/scope 信息，但不得泄露 private content。

The service must perform identity checks before repository lookup, read source and target exactly
once, and produce no result if either read fails. It must not invoke `ContextDiffService::compare`
directly or inspect graph fields; the existing `VersionedContextDiffReviewService` remains the one
composition point into the unified diff engine, and `GraphDiff::between` remains unique there.

service 必须在 repository lookup 前完成 identity check，source 与 target 各读取一次；任一 read 失败都不得产生 result。它不得
直接调用 `ContextDiffService::compare`，也不得读取 graph field；既有 `VersionedContextDiffReviewService` 仍是进入 unified
diff engine 的唯一 composition point，`GraphDiff::between` 仍保持唯一性。

## Implementation Plan / 实施计划

This plan is now implemented locally. The service remains private and storage-owned; no public
transport or write surface was added.

本计划已在本地实现。service 仍是 private、storage-owned；没有新增 public transport 或写入面。

### Task 1: Establish the persisted-input contract / 建立 persisted-input contract

**Files / 文件:**

- Create / 新增: `crates/storage/tests/context_diff_review.rs`
- Modify / 修改: none initially / 初始不修改其他文件

- [x] Add focused tests for same-project/same-Context distinct scopes, ordered provenance, and a complete three-dimension projection.
- [x] Add tests proving caller code supplies scopes only and that stored source/target payloads are the facts used for review.
- [x] Add tests for identical scope, cross-Context scope, nil scope, missing source, and deterministic repeated review.
- [x] Add side-aware read failures and out-of-scope/schema checks in the adapter contract.

- [x] 增加 same-project/same-Context distinct scope、ordered provenance 与完整三维 projection 的 focused test。
- [x] 增加 test 证明 caller 只提供 scope，review 使用的是 storage 中的 source/target payload fact。
- [x] 增加 identical scope、cross-Context scope、nil scope、missing source 与 deterministic repeated review 的 test。
- [x] 在 adapter contract 中增加 side-aware read failure 与 out-of-scope/schema 校验。

### Task 2: Add the storage-owned composition service / 增加 storage-owned composition service

**Files / 文件:**

- Create / 新增: `crates/storage/src/context_diff_review.rs`
- Modify / 修改: `crates/storage/src/lib.rs`
- Test / 测试: `crates/storage/tests/context_diff_review.rs`

- [x] Implement the scope-only service over `ContextDiffSnapshotV1Repository` as
  `PersistedContextDiffReviewService<'repository, R>`.
- [x] Keep repository reads exact, fail closed, side-aware, redacted, and free of raw payloads in errors.
- [x] Construct the existing versioned review request from cloned validated records and delegate exactly once to `VersionedContextDiffReviewService::project`.
- [x] Export only the reusable service, compatibility adapter alias, side type, and structured error; do not add any server, SDK, Web, or public contract.

- [x] 基于 `ContextDiffSnapshotV1Repository` 实现只接受 scope 的
  `PersistedContextDiffReviewService<'repository, R>`。
- [x] 保持 repository read exact、fail closed、带 side 信息、脱敏，且 error 不包含 raw payload。
- [x] 从 clone 后的已校验 record 构造既有 versioned review request，并且只委托一次给 `VersionedContextDiffReviewService::project`。
- [x] 只 export 可复用 service、兼容 adapter alias、side type 与结构化 error；不增加 server、SDK、Web 或 public contract。

### Task 3: Verify the local boundary / 验证 local boundary

**Files / 文件:**

- Modify / 修改: this plan only for observed local receipts / 仅在有实际 local receipt 时修改本计划

- [x] Run the focused storage test and the full `contextlab-storage` test suite.
- [x] Run the existing `contextlab-diff-engine` review tests to protect the delegated contract.
- [x] Run formatting, workspace Rust tests, strict offline Clippy, locked Rust `1.85.0` checks, and `pnpm check:web`.
- [x] Inspect the owned scope for new transport, public surface, raw private-content exposure, or a second `GraphDiff::between` path.
- [x] Record only observed local evidence; leave PostgreSQL runtime, browser, Git/remote, operator, release, and production as `unobserved` or `deferred` when not directly observed.

- [x] 运行 focused storage test 与完整 `contextlab-storage` test suite。
- [x] 运行既有 `contextlab-diff-engine` review test，保护被委托的 contract。
- [x] 运行 formatting、workspace Rust tests、strict offline Clippy、锁定 Rust `1.85.0` check 与 `pnpm check:web`。
- [x] 检查 owned scope 中是否出现新的 transport、public surface、raw private-content exposure 或第二条 `GraphDiff::between` path。
- [x] 只记录实际观测到的 local evidence；未直接观测时，将 PostgreSQL runtime、browser、Git/remote、operator、release 与 production 保持为 `unobserved` 或 `deferred`。

## Fresh Verification Gate Before the Next Increment / 下一增量前的新鲜验证门槛

### Observed local receipt / 已观测本地回执 (2026-07-29)

The admitted service boundary is locally green. `cargo test -p contextlab-storage --test
context_diff_review --quiet` passed `6`; the snapshot repository contract passed `4`, the migration
contract passed `2`, and `cargo test -p contextlab-diff-engine --all-targets --quiet` passed all
targets. Fresh `cargo test --workspace --quiet` passed API `183` and storage `179` with `39`
ignored; `cargo fmt --all -- --check`, strict offline workspace Clippy, and locked Rust `1.85.0`
workspace check passed. `pnpm check:web` passed public SDK `15`, local SDK `92`, Web `185`, and the
production build. Scope inspection found one `impl GraphDiff`, one delegated
`VersionedContextDiffReviewService::project` call, and no public version-backed graph-diff route in
the checked-in OpenAPI document. `.git\HEAD` and `.git\config` are absent, so Git change-set evidence
is `unobserved`.

本 service boundary 已取得 local green。`cargo test -p contextlab-storage --test context_diff_review --quiet` 通过 `6` 项；
snapshot repository contract 通过 `4` 项，migration contract 通过 `2` 项，`cargo test -p contextlab-diff-engine --all-targets
--quiet` 的全部 target 通过。新鲜 `cargo test --workspace --quiet` 通过 API `183`、storage `179`，其中 `39` 项 ignored；
`cargo fmt --all -- --check`、strict offline workspace Clippy 与锁定 Rust `1.85.0` workspace check 通过。`pnpm check:web`
通过 public SDK `15`、local SDK `92`、Web `185` 并完成 production build。scope inspection 发现一个 `impl GraphDiff`、
一次委托 `VersionedContextDiffReviewService::project`，且 checked-in OpenAPI 中没有 public version-backed graph-diff route。
`.git\HEAD` 与 `.git\config` 不存在，因此 Git change-set evidence 为 `unobserved`。

PostgreSQL runtime, Docker/virtualization, authenticated browser, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`; no secrets were read and no external
receipt is inferred from these local commands.

PostgreSQL runtime、Docker/virtualization、authenticated browser、remote CI、operator rehearsal、release 与 production 仍为
`unobserved` 或 `deferred`；未读取 secrets，也没有从这些 local command 推断任何 external receipt。

This increment's local gate is now satisfied by the observed receipt above. Any next increment still
requires its own bilingual Necessity Record and fresh, scope-matched verification. The receipt must
cover exact scope preservation, no caller payload injection, fail-closed missing/mismatched reads,
no partial output, deterministic semantic/behavior/evaluation results, and one delegated review path.
The commands used were:

本增量的 local gate 已由上方已观测回执满足。任何下一增量仍必须拥有自己的双语 Necessity Record 与范围匹配的新鲜验证。
回执必须覆盖 exact scope preservation、无 caller payload injection、missing/mismatched read 的 fail-closed、无 partial
output、semantic/behavior/evaluation 结果 deterministic，以及只有一条 delegated review path。实际使用的命令为：

```powershell
cargo test -p contextlab-storage --test context_diff_review
cargo test -p contextlab-storage
cargo test -p contextlab-diff-engine --all-targets
cargo fmt --all -- --check
cargo test --workspace --quiet
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo +1.85.0 check --workspace --all-targets --locked
pnpm check:web
rg -n "impl GraphDiff|GraphDiff::between|VersionedContextDiffReviewService::project" crates\storage\src crates\diff-engine\src
```

Expected local evidence is limited to the commands actually run. PostgreSQL service availability,
authenticated browser behavior, Git binding/change-set state, remote CI, operator rehearsal,
release, and production remain **deferred/unobserved** unless independently observed and recorded.
No release or production receipt is implied by a passing local test, a build, or this plan.

预期 local evidence 仅限于实际运行过的 command。除非独立观测并记录，PostgreSQL service availability、authenticated
browser behavior、Git binding/change-set state、remote CI、operator rehearsal、release 与 production 均保持
**deferred/unobserved**。local test、build 或本计划通过都不意味着存在 release 或 production receipt。

## Admission Boundary / 准入边界

This is now the implementation receipt for a local private composition service, not a deployment
authorization. It advances Criteria 2 and 4 with the observed local evidence above; it does not
close the long-term ContextLab vision, certify public compatibility, or authorize a public/private
transport decision. The Integration Lead separately synchronized the roadmap and architecture
ledgers; no public contract was changed.

本文件现是 local private composition service 的实现回执，不是部署授权。上述新鲜 local evidence 证明它推进了条件 2 与
4；但它不会关闭 ContextLab 的长期愿景，不会认证 public compatibility，也不会授权 public/private transport decision。
Integration Lead 已单独同步 roadmap 与 architecture ledger；没有改变 public contract。
