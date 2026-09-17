# Completion Criteria / 收束条件

## 2026-08-01 Private Workflow Execution Status Application Composition / 2026-08-01 私有 Workflow 执行状态应用组合

This bounded local receipt advances Criteria 1 and 8 without closing them or the active
long-term goal. PostgreSQL-mode `AppState::try_from_env` now composes the existing
`PostgresContextGraphRepository` into the private workflow execution status reader through the
storage adapter, while memory mode remains explicitly unavailable. A review-found state-drift risk
was repaired by using one `Unavailable`/`Custom`/`StorageBacked` backend discriminator instead of
an independent boolean.

本有界 local receipt 推进条件 1 与 8，但不关闭它们或 active long-term goal。PostgreSQL-mode
`AppState::try_from_env` 现通过 storage adapter 将既有 `PostgresContextGraphRepository` 组合到私有
Workflow execution status reader；memory mode 继续明确 unavailable。审查发现的 state-drift 风险已修复：使用单一
`Unavailable`/`Custom`/`StorageBacked` backend discriminator，替代独立 boolean。

Fresh verification / 新鲜验证：

- PostgreSQL composition focused test: `1 passed`.
- Memory unavailable focused regression: `1 passed`.
- Builder override regression: `1 passed`.

新鲜验证：

- PostgreSQL composition focused test：`1 passed`。
- Memory unavailable focused regression：`1 passed`。
- Builder override regression：`1 passed`。

The receipt is composition evidence only. A protected read against the same temporary live
PostgreSQL fixture was not separately observed and remains `unobserved`; a lazy URL cannot prove
runtime database behavior. No public REST/OpenAPI/public SDK write, Web mutation, operator
transport, migration, provider, secret access, second `GraphDiff` calculator, release, or
production claim was added. The long-term goal remains active and the next increment requires a
fresh bilingual Necessity Record.

本回执只属于 composition evidence。没有单独观测同一临时 live PostgreSQL fixture 上的 protected read，继续为
`unobserved`；lazy URL 不能证明 runtime database behavior。未新增 public REST/OpenAPI/public SDK write、Web mutation、operator
transport、migration、provider、secret access、第二个 `GraphDiff` calculator、release 或 production 声明。长期目标保持 active，下一增量必须先有新的双语 Necessity Record。

## 2026-08-01 Private Branch-Head Error Redaction / 2026-08-01 私有 Branch-Head 错误脱敏

This bounded local security increment advances Criterion 4 without closing it or the active
long-term goal. The private branch-head Web data adapter preserves the upstream error code and
HTTP status while replacing unknown-status diagnostic messages with a stable bilingual local
message. Scope validation, request-memory credentials, retry behavior, and shared presentation
states remain unchanged.

本次有界 local security 增量推进条件 4，但不关闭条件 4 或 active long-term goal。私有 branch-head Web data adapter 保留 upstream error code 与 HTTP status，同时将 unknown-status diagnostic message 替换为稳定双语本地文案。Scope validation、request-memory credential、retry behavior 与 shared presentation state 保持不变。

Fresh red/green evidence / 新鲜红绿证据：old behavior `6 passed, 2 failed` exposed both raw
messages; repaired focused branch-head suite `8 passed`. `cargo fmt --all -- --check`, offline
workspace Rust (`storage 212 passed, 39 ignored`), strict offline Clippy, locked Rust `1.85.0`,
`pnpm check:web` (`15/135/284 + production build`), scoped local verifier, and
`GRAPH_DIFF_IMPL_COUNT=1` passed. The verifier remains `overall=unobserved` without unified diff
input.

新鲜红绿证据：旧行为 `6 passed, 2 failed` 暴露两个 raw message；修复后的 focused branch-head suite `8 passed`。`cargo fmt --all -- --check`、offline workspace Rust（storage `212 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`（`15/135/284 + production build`）、范围化 local verifier 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。verifier 因未提供 unified diff input 继续为 `overall=unobserved`。

No public route, OpenAPI/SDK method, Web mutation, Rust/API/storage change, migration, provider,
secret access, second `GraphDiff` calculator, Docker/PostgreSQL runtime, authenticated browser,
visual smoke, Git, remote CI, operator rehearsal, release, or production claim was added. Criterion
4 and the remaining convergence conditions stay open.

未新增 public route、OpenAPI/SDK method、Web mutation、Rust/API/storage change、migration、provider、secret access、第二个 `GraphDiff` calculator、Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。条件 4 与其余收束条件继续开放。

## 2026-08-01 Private Context Diff V1 Read-Scope Hardening / 2026-08-01 私有 Context Diff V1 读取范围硬化

This bounded storage contract increment advances Criterion 2 without closing it or the long-term
goal. The PostgreSQL V1 read explicitly binds `context-diff-snapshot-v1`, and the Memory contract
test independently rejects project, Context, and commit identity drift. This closes a local schema
and exact-scope evidence gap without adding a migration or runtime database claim.

本有界 storage contract 增量推进条件 2，但不关闭条件 2 或长期目标。PostgreSQL V1 read 现显式绑定 `context-diff-snapshot-v1`，Memory contract test
分别拒绝 project、Context 与 commit identity drift。在不新增 migration 或 runtime database 声明的前提下，本增量收束本地 schema 与 exact-scope 证据缺口。

Fresh evidence / 新鲜证据：focused Memory scope `4 passed`、PostgreSQL SQL contract `1 passed`；workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；format、
strict offline Clippy、locked Rust `1.85.0` check passed；`pnpm check:web` passed with public SDK `15`, local SDK `134`, Web `272`, TypeScript/lint and production build；
local verifier scoped checks and `GRAPH_DIFF_IMPL_COUNT=1` passed. The verifier remains `overall=unobserved` because no unified diff input was supplied.

新鲜证据：focused Memory scope `4 passed`、PostgreSQL SQL contract `1 passed`；workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；format、strict offline Clippy、
锁定 Rust `1.85.0` check 通过；`pnpm check:web` 通过（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）；local verifier scoped checks 与
`GRAPH_DIFF_IMPL_COUNT=1` 通过。verifier 因未提供 unified diff input 继续为 `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret, operator
transport, or second graph-diff calculator was added. PostgreSQL/Docker runtime, authenticated
browser, visual smoke, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`; the long-term goal remains active and the next increment requires a new
bilingual Necessity Record.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret、operator transport 或第二个 graph-diff calculator。PostgreSQL/Docker runtime、
authenticated browser、visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；长期目标保持 active，下一增量必须新增双语 Necessity Record。

## 2026-08-01 Private Persisted Context Diff Writer Evidence / 2026-08-01 私有持久化 Context Diff Writer 证据

This bounded receipt advances Criteria 1, 2, and 4 without closing them or the long-term goal.
The production commit path now derives `ContextDiffSnapshotV1` from the same graph and persists it
with the commit and graph snapshot through the existing Memory/PostgreSQL normal and guarded
transaction boundaries. Exact-scope Memory reads and idempotent replay are covered by regression
tests. API and BFF fixtures supply explicit redacted producer facts so semantic, behavior, and
evaluation sections are each non-empty; no runtime evaluation result is invented.

本有界回执推进条件 1、2、4，但不关闭这些条件或长期目标。production commit path 现从同一 graph 派生 `ContextDiffSnapshotV1`，并通过既有
Memory/PostgreSQL 普通与 guarded transaction boundary 与 commit、graph snapshot 一起持久化。回归测试覆盖 Memory exact-scope read 与幂等 replay。
API 与 BFF fixture 提供明确脱敏 producer facts，使 semantic、behavior、evaluation 三段均非空；不虚构运行时评测结果。

Fresh evidence / 新鲜证据：workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；storage/API focused regressions passed；strict offline
Clippy、locked Rust `1.85.0` check、format passed；`pnpm check:web` passed with public SDK `15`, local SDK `134`, Web `272`, TypeScript/lint and production
build；local verifier scoped checks passed and `GRAPH_DIFF_IMPL_COUNT=1` passed. The verifier reports `overall=unobserved` because no unified diff input was supplied.

新鲜证据：workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；storage/API focused regressions 通过；strict offline Clippy、锁定 Rust `1.85.0` check 与 format 通过；
`pnpm check:web` 通过（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）；local verifier scoped checks 通过，`GRAPH_DIFF_IMPL_COUNT=1` 通过。
verifier 因未提供 unified diff input 报告 `overall=unobserved`。

No public write, OpenAPI/public SDK write, Web mutation, operator transport, provider, secret,
migration, release, or production claim changed. PostgreSQL/Docker runtime, authenticated browser,
visual smoke, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`; the long-term goal remains active and the next increment requires a new bilingual
Necessity Record.

没有新增 public write、OpenAPI/public SDK write、Web mutation、operator transport、provider、secret、migration、release 或 production 声明。PostgreSQL/Docker runtime、
authenticated browser、visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；长期目标保持 active，下一增量必须新增双语 Necessity Record。

## 2026-08-01 Private Local Contract Verifier Route Catalog Repair / 2026-08-01 私有本地契约 Verifier 路由目录修复

This bounded local increment advances Criterion 8 only; it does not close Criterion 8 or the
long-term goal. The static contract verifier's previous `benchmark-workspace-route-method-count:2`
baseline was stale: the protected Benchmark workspace read catalog contains exactly two local GET
variants, one cohort-keyed and one decision-keyed. The verifier now checks the exact two paths,
methods, handlers, and catalog entries. Its fixture covers wrong methods, non-local paths,
duplicate/missing handlers, duplicate/missing catalog entries, and public-router leakage.

本地有界增量仅推进条件 8，不关闭条件 8 或长期目标。静态 contract verifier 之前的
`benchmark-workspace-route-method-count:2` baseline 已过时：受保护 Benchmark workspace read catalog 精确包含两个 local GET variant，
分别按 cohort 与 decision 定位。verifier 现检查精确 path、method、handler 与 catalog entry；fixture 覆盖错误 method、非 local path、重复或缺失 handler、
重复或缺失 catalog entry 及 public-router leakage。

Fresh local evidence / 新鲜本地证据：focused PowerShell fixture passed; live verifier reported
`local_contract_source=passed`, `benchmark_workspace_route=passed`,
`benchmark_workspace_public_surface=passed`, `benchmark_definition_schema=passed`,
`benchmark_definition_public_boundary=passed`, `graph_diff_application=passed count=1`, and
`overall=unobserved` because no diff input was supplied. `cargo fmt --all -- --check`, offline
workspace Rust tests with storage `212 passed, 39 ignored`, strict offline Clippy, locked Rust
`1.85.0` check, and `pnpm check:web` with public SDK `15`, local SDK `134`, Web `270`, and
production build passed.

新鲜本地证据：focused PowerShell fixture 通过；live verifier 报告 `local_contract_source=passed`、`benchmark_workspace_route=passed`、
`benchmark_workspace_public_surface=passed`、`benchmark_definition_schema=passed`、`benchmark_definition_public_boundary=passed`、
`graph_diff_application=passed count=1`；由于未提供 diff input，`overall=unobserved`。`cargo fmt --all -- --check`、Rust workspace（storage
`212 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0` check，以及 `pnpm check:web`（public SDK `15`、local SDK `134`、Web `270`、
production build）均通过。

No public route, OpenAPI/SDK method, Web mutation, Rust domain behavior, migration, provider,
secret, Docker/PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal,
release, or production claim was added. The multi-dataset benchmark breadth wording is stale: later
local receipts already cover the named breadth evidence. The current unmet local evidence boundary
is authenticated browser-to-BFF-to-protected-Axum lifecycle smoke, which is not observed in this
workspace and is not claimed as a dependency-ready pass; any replacement implementation must first
have its own bilingual Necessity Record. Deferred external deployment facts remain outside current
local completion work.

没有新增 public route、OpenAPI/SDK method、Web mutation、Rust domain behavior、migration、provider、secret、Docker/PostgreSQL runtime、authenticated browser、
Git、remote CI、operator rehearsal、release 或 production 声明。多 dataset benchmark breadth 的表述已过时：后续本地回执已经覆盖该具名 breadth evidence。
当前未收束的本地证据边界是 authenticated browser-to-BFF-to-protected-Axum lifecycle smoke；本工作区尚未观测到该证据，也不将其写成 dependency-ready pass；任何替代实现都必须先拥有自己的双语 Necessity Record。
延期的外部部署事实继续不属于当前本地完成工作。

## 2026-07-30 Private Workflow Execution Status and Replay Provenance Read Closure / 2026-07-30 私有 Workflow 执行状态与回放 provenance 读取收束

This fresh local receipt advances Criteria 1 and 5 without closing either criterion or the
long-term goal. A schema-versioned Rust projection validates the exact Context-to-Workflow
binding, immutable workflow revision, terminal/replay provenance, and canonical event counts
without exposing raw events, failure payloads, provider output, or credentials. The protected
local API, non-public local SDK, same-origin BFF, and shared Web `data -> presenter -> screen`
layers now consume this projection. The default runtime has no execution repository and returns
typed `unavailable`; no execution producer or persistence path is claimed.

本次新鲜本地回执推进条件 1 与 5，但不关闭其中任何条件或长期目标。schema-versioned Rust projection 会校验精确的
Context-to-Workflow binding、immutable workflow revision、terminal/replay provenance 与 canonical event counts，且不暴露 raw events、
failure payload、provider output 或 credentials。protected local API、非公开 local SDK、同源 BFF 与 shared Web `data -> presenter -> screen`
层现已消费该 projection。默认运行时没有 execution repository，因此返回 typed `unavailable`；不声称存在 execution producer 或持久化路径。

Fresh local evidence / 新鲜本地证据：Workflow projection `5 passed`；protected API route `1 passed`；local SDK `111 passed`；execution-status BFF
route `4 passed`；Web
data/presenter/screen `7 passed`；`pnpm check:web` passed with public SDK `15`, local SDK `111`, Web `229`, TypeScript/lint, and production
build；`cargo test --workspace --quiet --no-fail-fast --offline` passed with storage `204 passed, 39 ignored`；format、strict offline Clippy、
locked Rust `1.85.0` check、`GRAPH_DIFF_IMPL_COUNT=1` 与 public workflow/plugin capability surface hits `0` 均通过。

没有新增 public REST/OpenAPI/public SDK method、execution start route、Workflow mutation、provider、migration、operator transport、secret 或
第二个 `GraphDiff` calculator。Docker/PostgreSQL、authenticated browser、visual、Git、remote CI、operator rehearsal、release 与 production
仍为 `unobserved` 或 `deferred`；条件 1、5 与长期目标继续开放。下一增量必须先新增独立的双语 Necessity Record。

The broader `scripts/verify-local-contracts.ps1` baseline is not a green receipt for this slice: it
stops at the unrelated `benchmark-workspace-route-method-count:2` check. The narrower
Workflow/Plugin public-surface and GraphDiff checks passed.

更宽的 `scripts/verify-local-contracts.ps1` baseline 不是本切片的绿色回执：它在无关的
`benchmark-workspace-route-method-count:2` 检查处停止。更窄的 Workflow/Plugin public-surface 与 GraphDiff check 已通过。

## 2026-07-27 Local Evidence Update / 2026-07-27 本地证据更新

The private versioned Context diff contract adds exact `(ProjectId, ContextId, CommitId)` identity
to semantic/behavior/evaluation review input and output, with fail-closed mixed-scope and
self-comparison rejection. The existing unified diff service remains the only semantic/behavior/
evaluation calculator, and `GraphDiff::between` remains the sole graph-diff calculator. This is
fresh local evidence for Criteria 2 and 4 only; it does not close them, authorize public transport,
or replace the missing exact persistence artifacts for semantic, behavior, and evaluation inputs.

私有版本化 Context diff contract 为 semantic/behavior/evaluation review 的输入与输出增加了精确的
`(ProjectId, ContextId, CommitId)` identity，并对 mixed-scope 与 self-comparison 执行 fail-closed 拒绝。既有
unified diff service 仍是唯一 semantic/behavior/evaluation calculator，`GraphDiff::between` 仍是唯一 graph-diff
calculator。这只是条件 2 与条件 4 的新鲜本地证据；它不关闭这些条件、不授权 public transport，也不替代 semantic、
behavior 与 evaluation input 尚缺失的 exact persistence artifact。

Observed receipts: diff-engine `20/20`; workspace Rust API `184 passed`, storage `179 passed, 39
ignored`; formatting; strict offline Clippy; Rust `1.85.0` workspace check; and `pnpm check:web`
with public SDK `14`, local SDK `85`, Web `179`, and production build. Static contract verification
observed `graph_diff_application=passed count=1`, while Git, browser, and production evidence remain
`unobserved`. PostgreSQL runtime, Docker/virtualization, remote CI, operator rehearsal, release,
and production remain `unobserved` or `deferred`; no secret or external receipt is implied.

已观测回执：diff-engine `20/20`；workspace Rust API `184 passed`、storage `179 passed, 39 ignored`；formatting；
strict offline Clippy；Rust `1.85.0` workspace check；以及 `pnpm check:web`（public SDK `14`、local SDK `85`、Web `179`，
并完成 production build）。静态 contract verification 观测到 `graph_diff_application=passed count=1`，Git、browser 与
production evidence 仍为 `unobserved`。PostgreSQL runtime、Docker/virtualization、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`；不暗示任何 secret 或外部 receipt。

### 2026-07-28 Private Commit-Scoped Diff Snapshot Persistence / 2026-07-28 私有按 Commit 绑定的 Diff Snapshot 持久化

The private storage wave now persists validated semantic/behavior/evaluation `ContextDiffSnapshotV1`
inputs against exact `(ProjectId, ContextId, CommitId, schema_version)` scope through
`VersionedContextScopeV1`. Memory and PostgreSQL adapters share immutable create/replay/conflict
semantics; migration `0023_context_diff_snapshots.sql` enforces composite foreign keys, the fixed
V1 schema, digest shape, and append-only behavior. Focused Memory tests passed `4`, the migration
contract passed `2`, and the PostgreSQL adapter contract observed `1 passed, 1 ignored` with its
runtime case intentionally not run. Workspace Rust, format, strict offline Clippy, locked Rust
`1.85.0`, and `pnpm check:web` also passed with API `183`, storage `179 passed, 39 ignored`, public
SDK `15`, local SDK `92`, Web `185`, and a successful production build.

私有 storage wave 现已通过 `VersionedContextScopeV1` 将已校验的 semantic/behavior/evaluation
`ContextDiffSnapshotV1` input 持久化到 exact `(ProjectId, ContextId, CommitId, schema_version)` scope。Memory 与
PostgreSQL adapter 共享 immutable create/replay/conflict 语义；migration `0023_context_diff_snapshots.sql` 强制
composite foreign key、固定 V1 schema、digest shape 与 append-only behavior。focused Memory test `4` 项通过，migration
contract `2` 项通过，PostgreSQL adapter contract 观测到 `1 passed, 1 ignored`，其中 runtime case 明确未运行。Workspace
Rust、format、strict offline Clippy、锁定 Rust `1.85.0` 与 `pnpm check:web` 也通过，包含 API `183`、storage `179 passed,
39 ignored`、public SDK `15`、local SDK `92`、Web `185` 与成功的 production build。

This advances Criteria 2 and 4 but does not close either. The private
`PersistedContextDiffReviewService` now reads two exact persisted records, validates scope and schema,
and delegates only to `VersionedContextDiffReviewService`; its focused contract has no caller-supplied
snapshot path or partial result. PostgreSQL runtime, authenticated browser, Git binding/change-set,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; no public
transport, Web mutation, provider call, secret access, or second `GraphDiff::between` calculator was
added. The next increment must be selected by a new bilingual Necessity Record.

本增量推进条件 2 与 4，但不关闭其中任何一项。私有 `PersistedContextDiffReviewService` 现读取两个 exact persisted
record，校验 scope 与 schema，并且只委托 `VersionedContextDiffReviewService`；其 focused contract 不接受
caller-supplied snapshot，也不返回 partial result。PostgreSQL runtime、authenticated browser、Git binding/change-set、
remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`；没有新增 public transport、Web
mutation、provider call、secret access 或第二个 `GraphDiff::between` calculator。下一增量必须先由新的双语 Necessity
Record 选择。

### 2026-07-30 Private Context Lifecycle Graph-Diff Review / 2026-07-30 私有 Context 生命周期 Graph-Diff 审阅

This fresh local receipt advances Criteria 1, 2, 4, 5, and 9 without closing them. A successful
private lifecycle commit now retains the exact prior branch head and returned commit as the
version-backed graph-review pair for create, content update, removal, and relationship operations.
The existing graph-review composition owns candidate selection and the existing local graph-diff
read owns loading and calculation; the Web bridge adds no policy or diff algorithm. Same-commit
replay pairs are rejected, and stale-head/conflict failures do not issue a graph-diff request.

本次新鲜本地回执推进条件 1、2、4、5 与 9，但不关闭这些条件。private lifecycle commit 成功后，现会为 create、content update、
removal 与 relationship operation 保留精确的 prior branch head 与返回 commit，作为 version-backed graph-review pair。既有
graph-review composition 负责 candidate selection，既有 local graph-diff read 负责 loading 与 calculation；Web bridge 不增加策略或
diff algorithm。same-commit replay pair 会被拒绝，stale-head/conflict failure 不会发起 graph-diff request。

Observed local receipts: `context-lifecycle-editor.test.tsx` `7/7`; `local-branch-heads-graph-review.test.tsx` `5/5`; `pnpm check:web`
with public SDK `15`, local SDK `99`, Web `207`, TypeScript/lint, and production build; workspace Rust with storage
`193 passed, 39 ignored`; `cargo fmt --all -- --check`; strict offline workspace Clippy; locked Rust `1.85.0` check; and
static `impl GraphDiff count=1`. PostgreSQL/Docker runtime, authenticated browser, Git change-set, remote CI, operator
rehearsal, release, and production remain `unobserved` or `deferred`; no public write or deployment evidence is implied.

已观测本地回执：`context-lifecycle-editor.test.tsx` `7/7`；`local-branch-heads-graph-review.test.tsx` `5/5`；`pnpm check:web`（public SDK `15`、
local SDK `99`、Web `207`、TypeScript/lint 与 production build）；workspace Rust（storage `193 passed, 39 ignored`）；
`cargo fmt --all -- --check`；strict offline workspace Clippy；锁定 Rust `1.85.0` check；以及 static `impl GraphDiff count=1`。
PostgreSQL/Docker runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production 仍为
`unobserved` 或 `deferred`；不暗示 public write 或 deployment evidence。

### 2026-07-30 Private Context Metadata Lifecycle Closure / 2026-07-30 私有 Context Metadata 生命周期收束

| Ownership/status / 所有权与状态 | Boundary and evidence / 边界与证据 |
| --- | --- |
| `completed / verified locally` for the bounded private metadata lifecycle only; the long-term goal and all completion criteria remain open. / 仅对有界 private metadata lifecycle 标记 `completed / verified locally`；长期目标与所有收束条件仍开放。 | The existing versioned `UpdateMetadata` operation now has an observed guarded-writer/replay/API/local-SDK/Web contract path. This receipt records the current code and local tests; it adds no code, test, route, public write, migration, provider, secret access, or external integration. / 既有 versioned `UpdateMetadata` operation 已观察到 guarded-writer/replay/API/local-SDK/Web contract path。本回执记录当前代码与本地测试；未新增 code、test、route、public write、migration、provider、secret access 或 external integration。 |
| Fresh local verification / 新鲜本地验证 | Storage metadata filter: `3 passed`; API metadata update/replay: `1 passed`; lifecycle data/proxy focused command: `14 passed`; full local SDK: `131 passed`; full Web: `265 passed`. / storage `3 passed`；API `1 passed`；lifecycle data/proxy focused command `14`；完整 local SDK `131`；完整 Web `265`。 |
| Evidence boundary / 证据边界 | Full workspace/production checks, PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git change-set, remote CI, operator rehearsal, release, production, and public promotion remain `unobserved` or `deferred`. / 其余全量与外部证据继续为 `unobserved` 或 `deferred`。 |
| Next queue / 下一队列 | Keep the long-term goal active. Any next implementation requires its own bilingual Necessity Record; do not treat this private slice as project completion. / 保持长期目标 active；下一项实现必须有独立双语 Necessity Record，不得将本 private slice 视为项目完成。 |

### 2026-07-30 Private Context Metadata Semantic Diff QA Status (Historical Snapshot) / 2026-07-30 私有 Context Metadata Semantic Diff QA 状态（历史快照）

| Criterion/status / 条件与状态 | Evidence and boundary / 证据与边界 |
| --- | --- |
| Criteria 1 and 2 were only admitted/in progress at that time; neither was closed. The implementation was then blocked by a focused source-contract mismatch. / 当时条件 1 与 2 仅处于已准入/in progress；均未关闭；实现当时被 focused source-contract mismatch 阻塞。 | The 2026-07-30 test required exported `ContextMetadataChangeV1::Modified { original, revised }`, while that workspace inspection found `SemanticMetadataChangeV1` struct usage. No green receipt had been recorded at that time. / 2026-07-30 测试要求导出 `ContextMetadataChangeV1::Modified { original, revised }`，当时工作区检查发现 `SemanticMetadataChangeV1` 结构体；当时尚无 green 回执。 |
| Existing predecessor evidence / 既有前置证据 | Private metadata lifecycle/Web fixture predecessor remains storage `3 passed`, API `1 passed`, lifecycle data/proxy `14 passed`, local SDK `131 passed`, and Web `265 passed`; it is not sufficient to close Criteria 1 or 2. / private metadata lifecycle/Web fixture 前置仍为 storage `3 passed`、API `1 passed`、lifecycle data/proxy `14 passed`、local SDK `131 passed`、Web `265 passed`；不足以关闭条件 1 或 2。 |
| Completion gate / 收束门禁 | Before this increment can advance, observe the contract repair's focused red/green proof, then the plan-required storage/workspace, format, strict offline Clippy, locked Rust `1.85.0`, Web, public-surface, and sole-`GraphDiff` checks. This docs-only pass adds none of those receipts. / 在本增量推进前，必须先观察 contract repair 的 focused red/green proof，再完成计划要求的 storage/workspace、format、strict offline Clippy、锁定 Rust `1.85.0`、Web、public-surface 与唯一 `GraphDiff` checks；本 docs-only pass 未新增这些回执。 |
| Boundary/evidence state / 边界与证据状态 | No public write, REST/OpenAPI/SDK expansion, Web mutation, migration, provider, secret access, or second `GraphDiff` calculator. PostgreSQL/Docker runtime, authenticated browser, visual smoke, and Git change-set remain `unobserved`; remote CI, operator rehearsal, release, and production remain `deferred`. Keep the long-term goal active. / 不新增 public write、REST/OpenAPI/SDK 扩展、Web mutation、migration、provider、secret access 或第二个 `GraphDiff` calculator。PostgreSQL/Docker runtime、authenticated browser、visual smoke 与 Git change-set 继续为 `unobserved`；remote CI、operator rehearsal、release 与 production 继续为 `deferred`；保持长期目标 active。 |

ContextLab is a long-running open-source infrastructure project. This document defines how the repository decides whether the project is complete enough to close the active long-term goal. Passing one feature, page, API, or demo is not sufficient.

ContextLab 是一个长期演进的开源基础设施项目。本文档定义仓库如何判断当前长期目标是否真正收束。完成单个 feature、页面、API 或 demo 都不足以关闭长期目标。

## Current Local Delivery Scope / 当前本地交付范围

Remote disposable CI, operator-approved change rehearsal, and public protected-write, release, or production promotion are explicitly deferred future deployment prerequisites because their external operating conditions are unavailable. They are not current local completion work or a blocker for dependency-ready Context-first engineering; this repository does not claim they have passed, been published, or been deployed.

由于外部运行条件不可用，远端 disposable CI、operator 批准的变更演练，以及 public protected-write、release 或生产推广均是明确延期的未来部署前置。它们不属于当前本地完成工作，也不阻断依赖就绪的 Context-first 工程；本仓库不会声称它们已经通过、发布或上线。

## Current Completion Audit / 当前完成度审计

The current repository has a solid Phase 1 foundation and an early Phase 2 platform slice:

当前仓库已经具备较扎实的第一阶段地基，并形成了早期第二阶段平台切片：

- Rust crates exist for `context-core`, `versioning`, `diff-engine`, `evaluation`, `graph`, `storage`, `workflow`, `embedding`, `knowledge`, `memory`, `mcp`, `plugin-runtime`, and `model-gateway`.
- 已具备 `context-core`、`versioning`、`diff-engine`、`evaluation`、`graph`、`storage`、`workflow`、`embedding`、`knowledge`、`memory`、`mcp`、`plugin-runtime` 与 `model-gateway` Rust crates。
- The Axum API exposes REST discovery routes for workspaces, projects, experiments, contexts, commits, components, evaluation runs, evaluation scorecards, providers, OpenAPI, workspace context graph reads, pure supplied-snapshot GraphDiff computation, and the default-off protected-local version-backed commit graph comparison read.
- Axum API 已提供 workspace、project、experiment、context、commit、component、evaluation run、evaluation scorecard、provider、OpenAPI、workspace context graph read、纯 supplied-snapshot GraphDiff computation，以及默认关闭的 protected-local version-backed commit graph comparison read。
- OpenAPI, GET/POST route catalog tests, and the TypeScript SDK provide an explicit client boundary for the current public surface.
- OpenAPI、GET/POST route catalog test 与 TypeScript SDK 为当前 public surface 提供显式 client boundary。
- `diff-engine` provides a tested deterministic `GraphDiff` domain contract for supplied Context Graph snapshots, and `POST /api/v1/graph-diffs` plus the SDK expose it without persisting either input. Storage now has tested in-memory and PostgreSQL contracts that atomically capture a new commit, ordered same-Context parents, and one immutable graph snapshot; the protected-local API, non-public local SDK, and Web workspace can compare two materialized commit snapshots with an explicit unavailable state for missing snapshots.
- `diff-engine` 为给定的 Context Graph snapshot 提供经过测试的确定性 `GraphDiff` 领域契约，`POST /api/v1/graph-diffs` 与 SDK 已在不持久化任一输入的前提下暴露它。storage 现有经过测试的内存与 PostgreSQL contract，可原子捕获新 commit、有序且同 Context 的 parent 以及一个不可变 graph snapshot；protected-local API、非公开 local SDK 与 Web workspace 已能比较两个 materialized commit snapshot，并在快照缺失时明确显示不可用状态。
- The Web workspace follows `data -> presenter -> screen`, uses shared design-system primitives, and mirrors context, commit, component, workspace graph, evaluation run detail, and selected scorecard aggregate data with preview/live fallback.
- Web workspace 遵循 `data -> presenter -> screen`，使用共享 design-system primitive，并通过 preview/live fallback 对齐 context、commit、component、workspace graph、evaluation run detail 与 selected scorecard aggregate data。
- The current graph/scorecard front-end integration slice is a read-only inspection path: SDK/API payloads enter the Web data boundary, the presenter creates graph nodes, relationships, score facts, and scorecard rows, and the screen renders them with shared primitives.
- 当前 graph/scorecard 前端接入切片是 read-only inspection path：SDK/API payload 进入 Web data boundary，presenter 生成 graph node、relationship、score fact 与 scorecard row，screen 使用共享 primitive 渲染。
- Documentation is mostly bilingual for project vision, API shape, SDK boundary, design-system foundation, and roadmap.
- 项目愿景、API shape、SDK boundary、design-system foundation 与 roadmap 基本保持中英双语。

The project is still not complete:

项目仍未完成：

- Web now consumes the workspace context graph and protected-local version-backed GraphDiff payloads for inspection, and the default-off private lifecycle editor provides graph-backed Add/Remove Uses relationship editing. The public SDK continues to expose only pure supplied-snapshot GraphDiff, and that API/SDK operation remains distinct from the persisted commit-review workflow. The edge-aware editor is local progress, not closure of Criteria 1 or 4.
- Web 现在消费 workspace context graph 与 protected-local version-backed GraphDiff payload，用于 inspection；默认关闭的 private lifecycle editor 已提供由 graph-backed state 支撑的 Add/Remove Uses relationship editing。public SDK 仍只暴露 pure supplied-snapshot GraphDiff，该 API/SDK operation 仍与持久化 commit 审阅 workflow 区分开来。edge-aware editor 只是本地进展，不关闭条件 1 或 4。
- The guarded commit domain, storage boundary, JWT transport, membership authorizer, and explicitly opt-in protected REST route support in-memory and PostgreSQL creation, replay, denial, stale-head behavior, and fail-closed authorization-decision audit recording. The protected route authenticates before rate limiting and authorization; PostgreSQL repeats direct/group membership resolution inside the commit transaction. Migrations `0006–0015` cover identity namespaces, group bindings, audit governance, shared limiter state, immutable component-content revisions, and durable same-Context commit-parent integrity. The local-only lifecycle editor and `/api/local` BFF are implemented, and Task 5 desktop/mobile preview accessibility/interaction QA is complete. The default public Axum router, checked-in OpenAPI, public TypeScript SDK, operator transport, and `GraphDiff` behavior remain mutation-free.
- guarded commit domain、storage boundary、JWT transport、membership authorizer 与显式 opt-in protected REST route 已支持内存和 PostgreSQL 创建、replay、拒绝、stale-head 行为以及 fail-closed authorization-decision audit recording。protected route 会先完成 authentication，再执行 rate limiting 与 authorization；PostgreSQL 会在 commit transaction 内重复 direct/group membership resolution。迁移 `0006–0015` 覆盖 identity namespace、group binding、audit governance、共享 limiter state、不可变 component-content revision 与持久化同 Context commit-parent integrity。仅本地 lifecycle editor 与 `/api/local` BFF 已实现，Task 5 的 desktop/mobile preview accessibility/interaction QA 已完成。默认 public Axum router、已检入 OpenAPI、public TypeScript SDK、operator transport 与 `GraphDiff` behavior 仍无 mutation。
- Earlier 15- and 17-test disposable PostgreSQL results remain dated historical records. The dated 25-case loopback disposable PostgreSQL verifier is only an underlying storage baseline covering security/storage contracts plus component-content creation, update, removal replay, and commit-parent scope integrity. It is not lifecycle PostgreSQL E2E evidence and makes no remote CI, operator approval, release, or production claim.
- 较早的 15 项与 17 项 disposable PostgreSQL 结果保留为带日期的历史记录。带日期的 25-case loopback disposable PostgreSQL verifier 仅是底层 storage baseline，覆盖此前安全/存储 contract，以及 component-content creation、update、removal replay 与 commit-parent scope integrity。它不是 lifecycle PostgreSQL E2E 证据，也不声称远端 CI、operator 批准、release 或 production 事实。
- The private protected lifecycle API now creates, updates, removes, replays, and reads complete component state through the guarded commit boundary. Its separate non-public SDK and local design-system Web workflow use request-scoped credentials, same-origin BFF transport, and existing version-backed review refreshes; no public OpenAPI/SDK/Web mutation surface has been added. Final fresh receipts are API `121 passed`, storage `152 passed, 25 ignored`, public SDK `14 passed`, local SDK `4 passed`, and Web `33 passed`. Lifecycle PostgreSQL E2E and authenticated browser-to-BFF-to-protected-Axum smoke remain deferred/unobserved. Context/prompt/schema editing beyond that bounded lifecycle, merge/rollback/replay workflows, semantic diff, evaluation diff, regression decisions, dashboards, and benchmark dataset management remain incomplete.
- 私有 protected lifecycle API 现已通过 guarded commit 边界创建、更新、删除、replay 并读取完整 component state。独立的非公开 SDK 与本地 design-system Web workflow 使用 request-scoped credential、同源 BFF transport 与既有基于版本的 review refresh；没有新增 public OpenAPI/SDK/Web mutation surface。最终新鲜回执为 API `121 passed`、storage `152 passed, 25 ignored`、public SDK `14 passed`、local SDK `4 passed`、Web `33 passed`。lifecycle PostgreSQL E2E 与 authenticated browser-to-BFF-to-protected-Axum smoke 仍为 deferred/unobserved。超出该有界生命周期的 Context/prompt/schema editing、merge/rollback/replay workflow、semantic diff、evaluation diff、regression decision、dashboard 与 benchmark dataset management 仍未完成。
- `contextlab-evaluation` now defines immutable benchmark cases, datasets, suites, and the domain-owned `BenchmarkEvaluation` artifact for unique inclusive metric thresholds, scorecard aggregation, and deterministic regression decisions. `EvaluationRun` rejects blank models and temperatures outside finite `0.0..=2.0`; `contextlab-storage` and migration `0016` preserve the same invariant while persisting sealed project/Context/commit-scoped definitions, runs, coverage facts, and decisions through private memory/PostgreSQL repository ports. Decision identities, children, and seals are exact-commit-scoped; duplicate metric evidence remains durable and fail-closed as `InsufficientData`. A protected local inspection path now exposes one exact sealed decision through a non-public local SDK, same-origin BFF, and design-system Web panel with recursive raw-payload rejection and private/no-store responses. Public REST/OpenAPI/public SDK and benchmark execution remained absent at that historical boundary; the 2026-07-23 Benchmark workspace receipt below supersedes its PostgreSQL projection-runtime status and current totals.
- `contextlab-evaluation` 现已定义不可变 benchmark case、dataset、suite，以及用于 metric 唯一且包含边界的 threshold、scorecard aggregate 和确定性 regression decision 的 domain-owned `BenchmarkEvaluation` artifact。`EvaluationRun` 会拒绝空白 model 与有限 `0.0..=2.0` 之外的 temperature；`contextlab-storage` 与迁移 `0016` 会在通过私有 memory/PostgreSQL repository port 持久化已 seal、按 project/Context/commit 作用域划分的 definition、run、coverage fact 与 decision 时维持同一不变式。decision identity、child 与 seal 都按精确 commit 分区；重复 metric evidence 仍可持久化，并以 `InsufficientData` fail-closed。现在已有 protected local inspection path 可经由非公开 local SDK、同源 BFF 与 design-system Web panel 暴露一条精确的 sealed decision，并且会递归拒绝 raw payload、返回 private/no-store response。public REST/OpenAPI/public SDK 与 benchmark execution 在该历史边界仍不存在；下方 2026-07-23 Benchmark workspace 回执取代其 PostgreSQL projection runtime 状态与当前测试总数。
- Bearer extraction, shared-secret JWT verification, the replaceable `PrincipalAuthenticator` port, RS256 OIDC verification through an HTTPS JWKS source with bounded cache and unknown-`kid` refresh, issuer-scoped `PrincipalIdentity`, bounded trusted OIDC groups, reusable Context RBAC, PostgreSQL direct/group assignment resolution and audit persistence, plus private in-process and shared PostgreSQL protected-route limiters now form the auth boundary. OIDC discovery, public write promotion, remote CI and operator-approved production evidence, audit retention/access review, plugin loading, MCP gateway, workflow execution, search, embeddings, object storage, and production deployment remain future platform work.
- Bearer 提取、共享密钥 JWT 校验、可替换 `PrincipalAuthenticator` port、经 HTTPS JWKS source 进行的 RS256 OIDC 校验（含有界 cache 与未知 `kid` refresh）、按 issuer 分区的 `PrincipalIdentity`、有界可信 OIDC group、可复用 Context RBAC、PostgreSQL direct/group assignment resolution 与 audit persistence，以及 private in-process 与共享 PostgreSQL protected-route limiter 现已形成 auth boundary。OIDC discovery、public write promotion、远端 CI 与 operator 批准的 production evidence、audit retention/access review、plugin loading、MCP gateway、workflow execution、search、embedding、object storage 与 production deployment 仍属于后续平台工作。
- PostgreSQL integration remains opt-in for local development. The 25-case loopback disposable verifier remains the underlying storage baseline; it does not close the deferred/unobserved lifecycle PostgreSQL E2E gate. Remote CI and operator-approved deployment evidence are deferred future external prerequisites outside the current local open-source delivery scope.
- PostgreSQL integration 在本地开发中仍保持 opt-in。25-case loopback disposable verifier 仍是底层 storage baseline；它不能关闭 deferred/unobserved 的 lifecycle PostgreSQL E2E 门禁。远端 CI 与 operator 批准的部署证据属于当前本地开源交付范围之外的延期未来外部前置。
- CLI and Desktop now share a typed unavailable-capability staging DTO with the Web capability-state adapters, but shared-core command registrations, production Tauri packaging, docs app, generated SDKs, contribution workflow, and release automation remain incomplete.
- CLI 与 Desktop 现已通过类型化的 unavailable-capability staging DTO 和 Web capability-state adapter 共享状态，但 shared-core command registration、production Tauri packaging、docs app、generated SDK、contribution workflow 与 release automation 仍未完成。

## Completion Assessment / 完成度判断

These percentages are planning estimates, not release claims. They are used to keep the active goal honest and prevent one feature slice from being mistaken for project completion.

以下百分比是规划估算，不是发布声明。它们用于约束当前长期目标，避免把单个 feature slice 误判为项目完成。

- Overall repository completion is criterion-based; historical scalar snapshots such as `16%` or `28%` are stale and non-authoritative. The repository now has a credible Context-first foundation, guarded local lifecycle slices, REST/SDK contracts, version-backed graph comparison, benchmark evidence/diff foundations, deterministic workflow/knowledge/memory/plugin cores, and typed CLI/Desktop/Web availability adapters. Most collaboration, public transport, provider, plugin bridge, operational, release, and production gates remain incomplete.
- 仓库整体完成度按具名完成条件判断；`16%` 或 `28%` 等历史单一百分比快照已经陈旧，不再作为权威状态。当前已有可信的 Context-first 地基、guarded local lifecycle 切片、REST/SDK contract、基于版本的图谱比较、benchmark evidence/diff 基础、确定性的 workflow/knowledge/memory/plugin core，以及类型化的 CLI/Desktop/Web availability adapter。大部分协作、public transport、provider、plugin bridge、运维、release 与 production gate 仍未完成。

### 2026-07-19 F+G Local Capability Availability / 2026-07-19 F+G 本地能力可用性

`apps/cli/crates/contextlab-adapter-contract` defines a serializable `contextlab.local-capability-availability.v1` projection for the existing typed unavailable adapter response. CLI and Desktop carry that projection without duplicating core registration or availability logic. The Web data boundary fail-closes invalid string-schema payloads and the existing numeric `schema_version: 1` resource shape, freezes safe bilingual DTOs, and delegates all five presentation states to the shared capability-state presenter and screen. A follow-on Knowledge/Memory fixture uses the same `data -> presenter -> screen` boundary to expose only frozen redacted citation/retention capability metadata; it adds no API, SDK, provider, or mutation transport. Fresh local evidence is focused adapter/CLI/Desktop and Knowledge/Memory fixture tests, strict Clippy for the adapter/CLI/Desktop crates, `cargo fmt --all -- --check`, `cargo test --workspace --quiet` (storage `166 passed, 36 ignored`), and `pnpm check:web` (Web `72` passed with a production build). Docker/PostgreSQL runtime, browser visual smoke, remote CI, operator approval, release, and production evidence remain ignored, unobserved, or deferred as applicable. This local adapter slice does not close any repository convergence condition.

`apps/cli/crates/contextlab-adapter-contract` 为既有类型化 unavailable adapter response 定义了可序列化的 `contextlab.local-capability-availability.v1` 投影。CLI 与 Desktop 只携带该投影，不重复实现核心 registration 或 availability logic。Web data boundary 会对无效的字符串 schema payload 和既有数字 `schema_version: 1` resource shape fail closed，冻结安全的双语 DTO，并把全部五种 presentation state 委托给共享 capability-state presenter 和 screen。后续 Knowledge/Memory fixture 使用同一 `data -> presenter -> screen` boundary，只暴露冻结的、脱敏的 citation/retention capability metadata；它不新增 API、SDK、provider 或 mutation transport。新鲜本地证据包括聚焦 adapter/CLI/Desktop 与 Knowledge/Memory fixture 测试、adapter/CLI/Desktop crate 的 strict Clippy、`cargo fmt --all -- --check`、`cargo test --workspace --quiet`（storage `166 passed, 36 ignored`）以及 `pnpm check:web`（Web `72` 通过且完成 production build）。Docker/PostgreSQL runtime、browser visual smoke、remote CI、operator approval、release 与 production evidence 按其实际情况仍为 ignored、unobserved 或 deferred。本地 adapter 切片不关闭任何 repository convergence condition。

### 2026-07-19 Wave 3 Local Capability Bridges / 2026-07-19 Wave 3 本地能力桥接

Wave 3 advances Criteria 1 and 3 without closing either: the Rust cores add deterministic redacted V1 projections for sealed benchmark receipt identity, Workflow status/replay, Knowledge citation, Memory retention/replay, and plugin capability availability. Only the existing private Workflow availability read is composed into the non-public local SDK, same-origin BFF, and shared-state Web workspace inspector; it remains default unavailable until a server-owned integration is registered and does not serialize the detailed Workflow status projection. Benchmark, Knowledge/Memory, and Plugin/MCP new projections do not yet claim transport. Fresh local evidence is focused Core tests and strict Clippy, `cargo fmt --all -- --check`, `cargo test --workspace --quiet` (storage `166 passed, 36 ignored`), focused private Workflow API test, and `pnpm check:web` (public SDK `14`, local SDK `28`, Web `78`, production build). API-wide strict Clippy is blocked only by the existing Rust 1.85 MSRV lint at `crates/auth/src/authorization.rs:320`; Docker/PostgreSQL runtime, browser visual/authenticated E2E, remote, release, and production evidence remain unobserved or deferred. This is not project completion or public-readiness evidence.

Wave 3 推进条件 1 与条件 3，但不关闭其中任何一项：Rust core 为 sealed benchmark receipt identity、Workflow status/replay、Knowledge citation、Memory retention/replay 与 plugin capability availability 增加确定性、脱敏的 V1 projection。只有既有 private Workflow availability read 被组合到非公开 local SDK、同源 BFF 与共享 state 的 Web workspace inspector；在 server-owned integration 注册前它保持默认 unavailable，且不序列化详细 Workflow status projection。Benchmark、Knowledge/Memory 与 Plugin/MCP 的新 projection 尚不声称已有 transport。新鲜本地证据包括聚焦 Core test 与 strict Clippy、`cargo fmt --all -- --check`、`cargo test --workspace --quiet`（storage `166 passed, 36 ignored`）、聚焦 private Workflow API test，以及 `pnpm check:web`（public SDK `14`、local SDK `28`、Web `78`、production build）。API-wide strict Clippy 仅被既有 Rust 1.85 MSRV lint `crates/auth/src/authorization.rs:320` 阻塞；Docker/PostgreSQL runtime、browser visual/authenticated E2E、remote、release 与 production evidence 仍为 unobserved 或 deferred。这不是项目完成或 public-readiness evidence。
- Current Web graph/scorecard presentation slice: about 98% for read-only inspection. The SDK/API payload is connected through `data -> presenter -> screen`, scorecards are filtered to the selected suite/model slice, graph relationships are inspectable, and persisted commit graph differences support accessible comparison with explicit snapshot-unavailable states. The remaining scope is richer filters, retry UX, and separately validated editing workflows.
- 当前 Web graph/scorecard 呈现切片：按 read-only inspection 口径约 98%。SDK/API payload 已经通过 `data -> presenter -> screen` 接入，scorecard 按选中 suite/model 切片收敛，graph relationship 可检查；持久化 commit 的图谱差异支持可访问比较，并在快照不可用时明确提示。剩余工作是更丰富的筛选、重试体验，以及独立验证的编辑 workflow。

## Repository Convergence Conditions / 仓库收束条件

The long-term project goal may be marked complete only when all conditions below are true and freshly verified. The current guarded commit route is deliberately opt-in; its existence does not satisfy the production security or release gates by itself.

只有以下条件全部成立并经过新鲜验证后，长期项目目标才可以标记完成。当前 guarded commit route 仍然是显式 opt-in；仅有该 route 本身不能满足 production security 或 release gate。

1. **Context-first platform coverage.** Workspaces, projects, experiments, contexts, components, commits, branches, evaluation runs, scorecards, datasets, workflows, knowledge, memory, tools, models, and graph relationships have domain models, persistence contracts, API contracts, SDK contracts, and at least one UI inspection or editing path.
   **以 Context 为核心的平台覆盖。** workspace、project、experiment、context、component、commit、branch、evaluation run、scorecard、dataset、workflow、knowledge、memory、tool、model 与 graph relationship 都具备 domain model、persistence contract、API contract、SDK contract，以及至少一条 UI inspection 或 editing path。
2. **Versioning and diff workflows.** Context changes are replayable through commits/branches, and text, semantic, behavior, and evaluation diffs are represented by tested reusable engines rather than page-local logic.
   **版本与 Diff 工作流。** Context change 可通过 commit/branch 回放，text、semantic、behavior 与 evaluation diff 由经过测试的可复用 engine 表达，而不是页面局部逻辑。
3. **Benchmark-driven evaluation.** Benchmark suites, datasets, run details, numeric scorecards, regression thresholds, and evaluation diff views are persisted, queryable, and visible in the Web workspace.
   **Benchmark-driven evaluation。** benchmark suite、dataset、run detail、numeric scorecard、regression threshold 与 evaluation diff view 均可持久化、可查询，并在 Web workspace 中可见。
4. **Graph as the system backbone.** Web, API, SDK, and storage consume the same Context Graph contracts for workspace/project/context relationships, and graph visualization/editing/diffing are backed by those contracts rather than page-local reconstruction.
   **Graph 作为系统骨架。** Web、API、SDK 与 storage 消费同一套 Context Graph contract 来表达 workspace/project/context relationship，并且 graph visualization/editing/diffing 均由这些 contract 支撑，而不是页面局部重建。
5. **Design-system-first interface.** Product screens use shared tokens and primitives from `packages/design-system` and `packages/ui`; page-local UI is limited to composition and layout adapters.
   **Design-system-first interface。** 产品界面使用 `packages/design-system` 与 `packages/ui` 中的共享 token 和 primitive；页面局部 UI 只承担组合和布局适配。
6. **Production security and collaboration.** Authentication, OAuth2/JWT, RBAC, audit logs, secret redaction, rate limiting, and collaboration boundaries are implemented and tested.
   **生产级安全与协作。** authentication、OAuth2/JWT、RBAC、audit log、secret redaction、rate limiting 与 collaboration boundary 均已实现并测试。
7. **Provider and plugin extensibility.** Model providers, MCP servers, tools, evaluators, importers, exporters, renderers, and storage adapters can be extended without changing core business logic.
   **Provider 与 plugin 可扩展性。** model provider、MCP server、tool、evaluator、importer、exporter、renderer 与 storage adapter 可扩展，且不需要改动核心业务逻辑。
8. **Reliable release gates.** `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, PostgreSQL integration tests against a disposable database, accessibility checks, visual workspace QA, and documented smoke tests all pass.
   **可靠发布门禁。** `cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web`、基于 disposable database 的 PostgreSQL integration test、accessibility check、visual workspace QA 与 documented smoke test 全部通过。
9. **Bilingual documentation.** Architecture, API, SDK, design-system, setup, contribution, security, release, and user workflow documentation are maintained in English and Chinese.
   **中英双语文档。** architecture、API、SDK、design-system、setup、contribution、security、release 与 user workflow 文档均保持中英双语维护。

`external-release-evidence-protocol.md` is a future-deployment reference for public protected-write promotion, release, and production rollout. Those external prerequisites are not part of the current local open-source completion audit and must not be tracked, audited, or awaited until external operating conditions exist. Local evidence never substitutes for them.

`external-release-evidence-protocol.md` 是 public protected-write promotion、release 与生产推广的未来部署参考。这些外部前置不属于当前本地开源完成度审计；在外部运行条件具备前，不得追踪、审计或等待。本地证据绝不替代它们。

### Current local progress / 当前本地进展

### 2026-07-27 Wave 2 Authoring Transport Receipt / 2026-07-27 Wave 2 Authoring Transport 回执

The private Benchmark Definition Authoring transport and editor are locally verified. Fresh commands
observed `cargo fmt --all -- --check`; workspace Rust tests with storage `167 passed, 39 ignored`;
strict workspace Clippy; locked Rust `1.85.0` check; `pnpm check:web` with public SDK `14`, local SDK
`68`, Web `151`, and the production build; authoring data/presenter/screen focused tests `11/11`; the
full Web route suite including retired-route `410` and canonical project-scoped route tests; the
wave2 verifier self-test; and a live verifier with `wave2_local_contracts=passed` and
`graph_diff_calculators=passed count=1`. The live verifier's `overall=unobserved` is caused only by
unavailable Git change-set evidence. The local Playwright workspace verifier was not observed because
no Web server was running. Docker/PostgreSQL runtime, authenticated browser runtime, Git binding,
remote CI, operator rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred`.
Criterion 3 and the long-term goal remain open.

私有 Benchmark Definition Authoring transport 与 editor 已获得本地验证。新鲜命令观测到
`cargo fmt --all -- --check`；workspace Rust test（storage `167 passed, 39 ignored`）；strict workspace
Clippy；锁定 Rust `1.85.0` check；`pnpm check:web`（public SDK `14`、local SDK `68`、Web `151`，并完成
production build）；authoring data/presenter/screen 聚焦 `11/11`；包含退役 route `410` 与 canonical
project-scoped route test 的 Web 全量 suite；wave2 verifier 自测；以及实际 verifier 的
`wave2_local_contracts=passed` 与 `graph_diff_calculators=passed count=1`。实际 verifier 的
`overall=unobserved` 仅由 Git change-set evidence 不可用导致。local Playwright workspace verifier 因未启动
Web server 而未观测。Docker/PostgreSQL runtime、authenticated browser runtime、Git binding、remote CI、
operator rehearsal、release 与 production 仍为 `ignored`、`unobserved` 或 `deferred`。条件 3 与长期目标保持开放。

The 2026-07-22 private Context-to-Workflow source-binding increment supplies part of Criteria 1, 2, 4, and 9: a reusable Workflow domain binding, append-only memory/PostgreSQL repository contract, exact Context commit/snapshot scope, deterministic ordering, and bilingual architecture evidence. It does not supply the remaining Workflow API/SDK/UI inspection path, workflow execution transport, complete graph editing, or release evidence. Its fresh local receipts are focused Workflow (`2 passed`) and storage (`3 passed`) binding tests, storage full suite (`166 passed, 36 ignored`), and `cargo fmt --all -- --check`; PostgreSQL runtime and browser evidence remain `ignored`/`unobserved` while Docker is disabled.

2026-07-22 私有 Context 到 Workflow source-binding 增量为条件 1、2、4、9 提供部分收束证据：可复用 Workflow domain binding、append-only memory/PostgreSQL repository contract、精确 Context commit/snapshot scope、确定性排序与双语架构证据。它仍未提供剩余的 Workflow API/SDK/UI inspection path、Workflow execution transport、完整 graph editing 或 release evidence。新鲜本地回执为聚焦 Workflow（`2 passed`）与 storage（`3 passed`）binding test、storage 全量（`166 passed, 36 ignored`）以及 `cargo fmt --all -- --check`；Docker 关闭时 PostgreSQL runtime 与 browser evidence 仍为 `ignored`/`unobserved`。

### 2026-07-22 Private Workflow Binding Read / 2026-07-22 私有 Workflow Binding 读取

The follow-on private read now supplies additional local evidence for Criteria 1, 2, 4, 6, and 9. The protected API and redacted `contextlab.local-workflow-context-bindings.v1` response are consumed by a fail-closed non-public local SDK, a same-origin BFF, and a shared Web `data -> presenter -> screen` inspector anchored to the selected exact commit. Fresh local receipts are API binding `4 passed`, local SDK `35 passed`, Web `tsc --noEmit`, Web `84 passed`, nested BFF route `7 passed` including raw-field rejection, `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, and `pnpm check:web` with its production Web build. This does not close any repository convergence condition: graph editing, workflow execution, public transport, PostgreSQL runtime, authenticated browser evidence, release, and production gates remain incomplete, unobserved, or deferred as applicable. `GraphDiff::between` remains the sole graph-diff calculator.

后续私有 read 为条件 1、2、4、6 与 9 增加了本地收束证据。受保护 API 与脱敏的 `contextlab.local-workflow-context-bindings.v1` response 现由 fail-closed 的非公开 local SDK、同源 BFF 与共享 Web `data -> presenter -> screen` inspector 消费，并锚定选定的精确 commit。新鲜本地回执为 API binding `4 passed`、local SDK `35 passed`、Web `tsc --noEmit`、Web `84 passed`、包含 raw-field rejection 的嵌套 BFF route `7 passed`、`cargo fmt --all -- --check`、`cargo test --workspace --quiet` 与带 production Web build 的 `pnpm check:web`。这不关闭任何 repository convergence condition：graph editing、Workflow execution、public transport、PostgreSQL runtime、authenticated browser evidence、release 与 production gate 仍按实际情况为 incomplete、unobserved 或 deferred。`GraphDiff::between` 仍是唯一 graph-diff calculator。

## Near-Term Priority / 近期优先级

### 2026-07-13 Shared-Limiter Status / 2026-07-13 共享限流状态

This is a historical local snapshot that predates later storage increments. The private
`PostgresProtectedRouteRateLimiter` was locally proven across two pools for quota atomicity, identity
isolation, expiry recovery, policy drift, and closed-pool failure. It remained outside HTTP, OpenAPI,
SDK, Web, and `GraphDiff`. External deployment conditions are explicitly deferred and are not an
active continuation item.

这是早于后续 storage 增量的历史本地快照。私有 `PostgresProtectedRouteRateLimiter` 已通过两个 pool
在本地证明 quota atomicity、identity isolation、expiry recovery、policy drift 与 closed-pool failure；
它保持在 HTTP、OpenAPI、SDK、Web 与 `GraphDiff` 之外。外部部署条件已明确延期，不属于活跃续作。

### 2026-07-14 Concurrency-Hardening Update / 2026-07-14 并发硬化更新

Historical local regressions proved that an admission lock must not block an existing key and that a
future stored timestamp must fail closed. Migration `0011` made the policy immutable, introduced
deterministic per-key locking, and retained global locking only for first-key admission. The then-current
17-case disposable run and forward rehearsal are preserved as dated local evidence, not as remote,
operator-approved, release, or production evidence.

历史本地回归证明 admission lock 不能阻塞 existing key，future stored timestamp 必须 fail closed。迁移
`0011` 使 policy 不可变、使用确定性 per-key locking，并只为 first-key admission 保留 global locking。
当时的 17-case disposable run 与 forward rehearsal 作为带日期的本地证据保留，不代表远端、operator
批准、release 或 production 事实。

### 2026-07-14 Deferred External Release Evidence / 2026-07-14 延期外部发布证据

This section preserves the 2026-07-14 decision boundary as history: private auth, RBAC, audit, limiter,
purge, and migration-rehearsal assets had local evidence, but no public write or deployment claim was made.
The external-evidence protocol is now a dormant future-deployment reference. Its receipts are not tracked,
audited, requested, or awaited in the current local open-source delivery scope.

本节把 2026-07-14 的决策边界作为历史保留：private auth、RBAC、audit、limiter、purge 与 migration
rehearsal asset 当时已有本地证据，但没有 public write 或部署声明。外部证据协议现在是休眠的未来部署
参考；在当前本地开源交付范围内，不追踪、不审计、不索取也不等待其回执。

The checked-in workflow/evidence assets are historical repository shape only. They are not current execution work and establish no remote, approval, release, or production fact.

已检入的 workflow/evidence asset 仅作为历史 repository shape 保留；它们不是当前执行工作，也不建立远端、审批、release 或 production 事实。

Migration and rehearsal manifests remain dormant future-deployment inputs. They are not collected or reviewed during local core-platform increments.

Migration 与 rehearsal manifest 作为休眠的未来部署输入保留；本地核心平台增量不收集也不审阅它们。

### 2026-07-14 Private Component-Content Revision Update / 2026-07-14 私有 Component 正文修订更新

`context-core` now owns opaque UTF-8 component bodies with deterministic SHA-256 fingerprints. `versioning` records optional prior/resulting body hashes on `UpdatedComponent`, while `contextlab-storage` validates and atomically persists one immutable revision for an existing component through the guarded commit transaction. The current component hash projection updates in that transaction, and a private repository can read the exact `(context, commit, component)` revision. A fresh exact PostgreSQL 16.14 regression covers create, idempotent replay, private lookup, and projection update; the disposable-script selection now has 18 reset-per-test cases. This advances Context-first coverage and replayable versioning, but creates no public body read or write, REST/OpenAPI/SDK/Web mutation, operator transport, or `GraphDiff` change. Historical components without a captured revision remain body-unavailable.

`context-core` 现拥有不透明 UTF-8 component 正文及确定性的 SHA-256 fingerprint。`versioning` 会在 `UpdatedComponent` 上记录可选的前后正文 hash，`contextlab-storage` 则通过 guarded commit transaction 校验并原子持久化既有 component 的一条不可变 revision。当前 component hash projection 也在同一 transaction 内更新，私有 repository 可按精确的 `(context, commit, component)` 读取 revision。一项新鲜、精确的 PostgreSQL 16.14 regression 覆盖 create、idempotent replay、private lookup 与 projection update；disposable script 选择现有 18 个逐例 reset 的 case。这推进了 Context-first 覆盖与可回放版本化，但不新增 public 正文 read/write、REST/OpenAPI/SDK/Web mutation、operator transport 或 `GraphDiff` 改动。没有已捕获 revision 的历史 component 仍不可取得正文。

### 2026-07-15 Private Component-Content Creation Update / 2026-07-15 私有 Component 正文创建更新

`AddedComponent` now carries a replayable non-empty name, JSON metadata, and deterministic initial body hash for private body creation. `ComponentContentCreationWrite` binds that descriptor to the commit-owned component UUID and requires the same component node and taxonomy relationship in the commit graph snapshot. The guarded memory and PostgreSQL adapters accept exactly one matching existing-body revision or creation attachment for every component transition. A creation atomically persists its component row, immutable revision with `previous_content_hash = NULL`, commit/snapshot, branch head, and idempotency receipt; a duplicate component is rejected before those records can change. Memory graph projection overlays the current hash and last accepted revision timestamp for both seeded and dynamic components. Forward migrations `0013 -> 0014` preserve historical non-null priors while a migration-time history check, partial unique index, and trigger permit a `NULL` prior only for the initial revision at the component creation timestamp. Focused local PostgreSQL 16.14 tests cover creation, replay, body-revision lookup, projection update, duplicate rejection without partial writes, the `0012 -> 0014` upgrade path, and malformed `NULL` priors. The disposable script now selects 21 reset-per-test cases. This remains private storage evidence: no public REST/OpenAPI/SDK/Web write or body-read surface, operator transport, release receipt, or `GraphDiff` behavior changed.

`AddedComponent` 现在会携带用于私有正文创建的可回放非空 name、JSON metadata 与确定性初始 body hash。`ComponentContentCreationWrite` 会把该 descriptor 绑定到 commit 所有的 component UUID，并要求 commit graph snapshot 中存在同一个 component node 与 taxonomy relationship。对于每个 component transition，Guarded memory 和 PostgreSQL adapter 都只能接受恰好一个匹配的既有 body revision 或 creation attachment。Creation 会原子持久化 component row、带 `previous_content_hash = NULL` 的不可变 revision、commit/snapshot、branch head 与 idempotency receipt；重复 component 会在这些 record 变更前被拒绝。Memory graph projection 会为 seeded 与 dynamic component 覆盖当前 hash 和最后一次已接受 revision 的 timestamp。前向迁移 `0013 -> 0014` 会保留历史 non-null prior；migration-time history check、partial unique index 与 trigger 只允许 component creation timestamp 上第一条 revision 使用 `NULL` prior。聚焦的本地 PostgreSQL 16.14 test 覆盖 creation、replay、body-revision lookup、projection update、无 partial write 的 duplicate rejection、`0012 -> 0014` migration path，以及不合规的 `NULL` prior。disposable script 现选择 21 个逐例 reset case。这仍是私有 storage 证据：没有 public REST/OpenAPI/SDK/Web write 或 body-read surface、operator transport、release receipt 或 `GraphDiff` behavior 发生变化。

### 2026-07-15 Private Component-Content Replay Resolution / 2026-07-15 私有 Component 正文回放解析

`ComponentContentRevisionRepository::get_component_content_at_commit` now validates the complete normal first-parent ancestry of a target commit before resolving the nearest immutable component-body revision in both memory and PostgreSQL. It preserves the revision's source commit identity, returns `None` for a known commit with no captured body, distinguishes an unknown Context from an unknown commit, and rejects merge ancestry, cycles, and malformed cross-Context parent edges. Fresh focused memory and disposable PostgreSQL tests cover unchanged and revised descendants, no-body components, unknown scopes and targets, merge boundaries hidden behind a nearer revision, and malformed parent scope; the disposable selection contains 22 reset-per-test cases. This advances replayable versioning while remaining storage-only: no public body read/write, graph editing, merge policy, REST/OpenAPI/SDK/Web method, operator transport, release receipt, or `GraphDiff` behavior changed.

`ComponentContentRevisionRepository::get_component_content_at_commit` 现已在内存与 PostgreSQL 中先验证 target commit 的完整 normal first-parent ancestry，再解析最近的不可变 component-body revision。它保留 revision 的 source commit identity；已知但无 captured body 的 commit 返回 `None`；不存在的 Context 与 unknown commit 可区分，merge ancestry、cycle 与不合规的跨 Context parent edge 都会被拒绝。新鲜的 focused memory 与 disposable PostgreSQL test 覆盖 unchanged 与 revised descendant、无正文 component、unknown scope 与 target、被较近 revision 掩盖的 merge boundary，以及不合规的 parent scope；disposable selection 包含 22 个逐例 reset case。这推进了可回放版本化，但仍仅限 storage：没有 public body read/write、graph editing、merge policy、REST/OpenAPI/SDK/Web method、operator transport、release receipt 或 `GraphDiff` behavior 发生变化。

### 2026-07-15 Context Commit Parent Scope Integrity / 2026-07-15 Context Commit 父范围完整性

Migration `0015_context_commit_parent_scope_integrity.sql` moves the same-Context parent invariant into durable PostgreSQL storage. It preflights malformed history before schema changes, backfills valid links, requires the owning Context, and adds composite foreign keys for both child and parent commit endpoints. Valid history upgrades; malformed cross-Context history fails with SQLSTATE `23503` without partial catalog changes; direct mismatched child or parent inserts are rejected; child deletion cascades links; and a referenced parent stays restricted. The guarded writer binds the Context explicitly, and a parented idempotent replay leaves exactly one commit, parent link, graph snapshot, receipt, and matching branch head. At that historical checkpoint, all 25 reset-per-test PostgreSQL cases passed locally and the normal storage suite reported `142 passed, 25 ignored`; these are historical counts, superseded by the final fresh receipt above. This remains a local private-storage increment: no public write/read transport, release, production promotion, or `GraphDiff` behavior changed.

迁移 `0015_context_commit_parent_scope_integrity.sql` 将同 Context parent 不变量迁移到持久化 PostgreSQL storage。它会在 schema change 前 preflight 损坏历史、回填合法 link、要求所属 Context，并为 child 与 parent commit 两端增加复合外键。合法历史可以升级；损坏的跨 Context 历史会以 SQLSTATE `23503` 失败且不留下部分 catalog 变更；直接插入中 child 或 parent 的 Context 不匹配会被拒绝；child 删除会级联 link，而仍被引用的 parent 保持受限。guarded writer 显式绑定 Context，带 parent 的幂等 replay 只保留一个 commit、parent link、graph snapshot、receipt 与匹配的 branch head。在该历史检查点，25 个逐例 reset 的 PostgreSQL case 已在本地全部通过，普通 storage suite 报告 `142 passed, 25 ignored`；这些是已被上文最终新鲜回执取代的历史数字。这仍是本地私有 storage 增量：没有 public write/read transport、release、生产推广或 `GraphDiff` behavior 变化。

### 2026-07-15 Private Component State at Commit Replay / 2026-07-15 私有 Component 提交状态回放

`ComponentStateAtCommitRepository::get_component_state_at_commit` now reconstructs one component's descriptor, metadata, effective hash, creation commit, and last content-change commit at a known target commit. Memory and PostgreSQL validate complete normal first-parent ancestry before they fold detailed `AddedComponent`, `UpdatedComponent`, and typed `RemovedComponent` changes, cross-checking each creation or update against the immutable revision recorded at that commit and each removal against effective kind and hash. A known target without a reachable detailed creation or after a valid removal returns `None`; malformed, stale, repeated, or post-removal payloads, missing or mismatched revisions, discontinuous hashes, merge ancestry, cycles, and cross-Context parent corruption fail closed as the same component-state replay conflict in both adapters. Fresh local evidence includes `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, the shell guard, and all 25 isolated reset-per-test PostgreSQL cases. The returned state contains no component body and adds no REST/OpenAPI/SDK/Web route, control, or method, operator transport, release claim, or second graph-diff calculator.

`ComponentStateAtCommitRepository::get_component_state_at_commit` 现可在已知 target commit 重建单个 component 的 descriptor、metadata、有效 hash、creation commit 与最近 content-change commit。内存和 PostgreSQL 都会先验证完整 normal first-parent ancestry，再折叠带详情的 `AddedComponent`、`UpdatedComponent` 与 typed `RemovedComponent` change；每次 creation 或 update 都与同一 commit 记录的不可变 revision 交叉校验，每次 removal 都与有效 kind 和 hash 交叉校验。已知 target 没有可达 detailed creation 或位于合法 removal 之后时返回 `None`；损坏、陈旧、重复或 removal 后的 payload、缺失或不匹配的 revision、不连续 hash、merge ancestry、cycle 与跨 Context parent 损坏都会在两个 adapter 中以同一个 component-state replay conflict fail closed。新鲜本地证据包括 `cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web`、shell guard，以及全部 25 个隔离的 reset-per-test PostgreSQL case。返回 state 不包含 component body，也不新增 REST/OpenAPI/SDK/Web route、control 或 method、operator transport、release 声明或第二个 graph-diff calculator。

### 2026-07-15 Private Context Component-State Snapshot / 2026-07-15 私有 Context Component-State Snapshot

`ContextComponentStateSnapshotAtCommitRepository::get_context_component_state_snapshot_at_commit` now materializes the full descriptor-only component inventory at a target Context commit. Both adapters validate the complete normal first-parent ancestry, parse stored changes, and fold detailed creation, update, and typed removal transitions from root to target with immutable revision witnesses. Results are deterministically ordered by component identifier; a target before an update retains the original hash and a target after a valid removal omits that component, without leaking the current projection. Malformed, stale, repeated, or post-removal data, unknown scope, merge/cycle/cross-Context ancestry fail closed without a partial result. Fresh focused tests, `cargo fmt --all -- --check`, `cargo test --workspace`, `pnpm check:web`, the shell guard, and all 25 reset-per-test local PostgreSQL cases passed. No component body, public REST/OpenAPI/SDK/Web contract, mutation capability, operator transport, release claim, or additional `GraphDiff` calculator was added.

`ContextComponentStateSnapshotAtCommitRepository::get_context_component_state_snapshot_at_commit` 现可在 target Context commit materialize 完整的、仅含 descriptor 的 component inventory。两个 adapter 都会验证完整 normal first-parent ancestry、解析存储的 change，并借助不可变 revision witness 从 root 到 target 折叠带详情的 creation、update 与 typed removal transition。结果按 component identifier 确定性排序；位于 update 前的 target 会保留原始 hash，位于合法 removal 后的 target 不包含该 component，不会泄漏当前 projection。损坏、陈旧、重复或 removal 后的数据、unknown scope、merge/cycle/跨 Context ancestry 都会 fail closed，且不返回部分结果。新鲜的 focused test、`cargo fmt --all -- --check`、`cargo test --workspace`、`pnpm check:web`、shell guard 和全部 25 个 reset-per-test 本地 PostgreSQL case 均已通过。未新增 component body、public REST/OpenAPI/SDK/Web contract、mutation capability、operator transport、release 声明或额外 `GraphDiff` calculator。

### 2026-07-15 Private Component Removal Replay / 2026-07-15 私有 Component Removal Replay

Typed `RemovedComponent` transitions now bind the component identifier, kind, and effective prior content hash to one guarded commit. The graph snapshot must omit the removed node. Memory and PostgreSQL validate the active projection under the guarded transaction, preserve immutable revision history, advance the branch head and idempotency receipt atomically with the commit and snapshot, then hide the current component through soft deletion. Replay makes valid later component state and Context inventory absent without consulting that mutable projection. Stale, mismatched, repeated, or post-removal transitions fail closed without partial persistence; an identical idempotency replay returns the original outcome. Historical local evidence from that increment included focused removal tests, formatting, workspace/Web checks, the shell guard, the 25-case underlying PostgreSQL storage baseline, and a storage count of `147 passed, 25 ignored`; that count is historical and superseded by the final fresh receipt above. This does not add public REST/OpenAPI/SDK/Web mutation, operator transport, release/promotion evidence, or another graph-diff calculator.

Typed `RemovedComponent` transition 现将 component identifier、kind 与有效 prior content hash 绑定到一个 guarded commit。graph snapshot 必须不含已移除节点。memory 与 PostgreSQL 会在 guarded transaction 中验证 active projection，保留不可变 revision history，并将 commit、snapshot、branch head 与 idempotency receipt 原子推进，再通过 soft delete 隐藏当前 component。replay 不读取该可变 projection，合法的较晚 component state 与 Context inventory 均会缺席。陈旧、不匹配、重复或 removal 后的 transition 会 fail closed，且不留下部分持久化；相同的 idempotency replay 返回原有结果。该增量的历史本地证据包括聚焦 removal test、格式检查、workspace/Web 检查、shell guard、作为底层 storage baseline 的 25-case PostgreSQL 运行，以及 storage `147 passed, 25 ignored`；该数字是历史值，已被上文最终新鲜回执取代。这不新增 public REST/OpenAPI/SDK/Web mutation、operator transport、release/promotion 证据或额外 graph-diff calculator。

### 2026-07-16 Local Context Lifecycle Evidence Closure / 2026-07-16 本地 Context 生命周期证据收束

The local lifecycle vertical slice is implemented through the pure service, protected non-OpenAPI Axum routes, non-public local SDK, same-origin BFF, and design-system Web editor. Task 5 implementation and desktop/mobile preview accessibility/interaction QA are checked complete. Final fresh receipts are API `121 passed`, storage `152 passed, 25 ignored`, public SDK `14 passed`, local SDK `4 passed`, and Web `33 passed`. The public OpenAPI/SDK boundary remains mutation-free, and `GraphDiff::between` remains the sole graph-diff calculator.

本地 lifecycle 垂直切片已贯通 pure service、protected 非 OpenAPI Axum route、非公开 local SDK、同源 BFF 与 design-system Web editor。Task 5 的实现以及 desktop/mobile preview accessibility/interaction QA 已勾选完成。最终新鲜回执为 API `121 passed`、storage `152 passed, 25 ignored`、public SDK `14 passed`、local SDK `4 passed`、Web `33 passed`。public OpenAPI/SDK 边界仍不包含 mutation，`GraphDiff::between` 仍是唯一 graph-diff calculator。

The 25-case PostgreSQL run remains only an underlying storage baseline. Lifecycle PostgreSQL E2E and authenticated browser-to-BFF-to-protected-Axum smoke are deferred/unobserved. Remote CI, operator approval, public promotion, release, and production rollout remain deferred external evidence; this closure makes no publication or release claim.

25-case PostgreSQL 运行仍仅是底层 storage baseline。lifecycle PostgreSQL E2E 与 authenticated browser-to-BFF-to-protected-Axum smoke 仍为 deferred/unobserved。remote CI、operator approval、public promotion、release 与 production rollout 仍是延期外部证据；本次收束不声称已经发布或上线。

### 2026-07-18 Private Component Descriptor Revision / 2026-07-18 私有 Component 描述符修订

`UpdatedComponentDescriptor` extends the existing local lifecycle vertical slice with a replayable descriptor-only commit. It rejects blank names, kind changes, descriptor body hashes, missing prior state, and graph-kind conflicts; it preserves the effective body hash and content commit while replacing only name/metadata and the matching successor graph-node label. One private `ComponentDescriptorRevisionWrite` updates the current component projection atomically with commit, snapshot, branch head, and receipt. Idempotency receipts now use `(identity source, principal, Context, branch, key)` in memory and PostgreSQL, with forward migration `0017_branch_scoped_commit_idempotency.sql` backfilling the branch from the referenced immutable commit and replay verifying that match. The operation reuses the same protected local lifecycle POST, non-public SDK, same-origin BFF, shared-primitive Web editor, atomic branch-head/idempotency boundary, and `GraphDiff::between` remains unchanged. Fresh local verification passed `cargo fmt --all -- --check`, `cargo test --workspace --quiet` (API `136 passed`; storage `163 passed, 33 ignored`), and `pnpm check:web` (public SDK `14`, local SDK `23`, Web `51`, TypeScript checks, production build). Strict Clippy remains a repository-baseline gap because scoped lint reports pre-existing auth/storage/API warnings. PostgreSQL runtime and browser E2E remain unobserved while Docker is disabled, and no public REST/OpenAPI/public SDK mutation surface is added.

`UpdatedComponentDescriptor` 在既有 local lifecycle vertical slice 上增加可回放的 descriptor-only commit。它会拒绝空白 name、kind change、descriptor body hash、缺失 prior state 与 graph-kind conflict；在只替换 name/metadata 和匹配 successor graph-node label 的同时，保留有效 body hash 与 content commit。一个私有 `ComponentDescriptorRevisionWrite` 会将当前 component projection 与 commit、snapshot、branch head、receipt 原子更新。memory 与 PostgreSQL 的 idempotency receipt 现使用 `(identity source, principal, Context, branch, key)`；前向迁移 `0017_branch_scoped_commit_idempotency.sql` 会从被引用的不可变 commit 回填 branch，replay 时还会校验二者匹配。该 operation 复用同一 protected local lifecycle POST、非公开 SDK、同源 BFF、共享 primitive 的 Web editor、原子 branch-head/idempotency 边界，`GraphDiff::between` 保持不变。新鲜本地验证已通过 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`（API `136 passed`；storage `163 passed, 33 ignored`）以及 `pnpm check:web`（public SDK `14`、local SDK `23`、Web `51`、TypeScript check、production build）。strict Clippy 仍是 repository-baseline 缺口，因为范围化 lint 报告已有 auth/storage/API warning。Docker 关闭时 PostgreSQL runtime 与 browser E2E 仍未观测，且没有新增 public REST/OpenAPI/public SDK mutation surface。

### 2026-07-18 Private Unborn-Branch Context Initialization / 2026-07-18 私有未出生分支 Context 初始化

`ContextLifecycleOperation::Initialize` now closes the local lifecycle's root-state gap for an already persisted active Context. It is the only tagged local operation that accepts `expected_head_commit_id: null`; it reads only the Context name through a private root port, constructs one parentless `CreatedContext` commit and one-node Context graph snapshot, and reuses the sole guarded writer for atomic branch-head and branch-scoped idempotency receipt persistence. It creates no component, body revision, migration, client graph, or public write. Memory tests directly read the adapter's persisted branch head and scoped receipt, prove root replay after a later same-branch head advance, and prove independent initialization on another unborn branch with the same key/digest. Fresh evidence is API `140 passed`, storage `165 passed, 34 ignored`, public SDK `14`, local SDK `24`, Web `57`, TypeScript checks, and a production build. The editor and mutation BFF are default-deny unless the exact server-owned `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE=true` enables local development. One PostgreSQL parity test is compiled and ignored while Docker is disabled; authenticated browser mutation smoke remains unobserved. The protected local API, non-public local SDK, same-origin BFF, and shared-primitive editor preserve authentication, RBAC, audit, rate limiting, no cookies, private/no-store, and the public OpenAPI/SDK exclusion; `GraphDiff::between` remains the sole graph-diff calculator.

`ContextLifecycleOperation::Initialize` 现为已持久化且 active 的 Context 补齐 local lifecycle 的 root-state 缺口。它是唯一接受 `expected_head_commit_id: null` 的带标签 local operation；它通过私有 root port 只读取 Context name，构造一条无 parent 的 `CreatedContext` commit 和只含 Context 节点的 graph snapshot，并复用唯一 guarded writer 原子持久化 branch-head 与 branch-scoped idempotency receipt。它不创建 component、body revision、migration、客户端 graph 或 public write。memory test 直接读取 adapter 持久化的 branch head 与 scoped receipt，证明同一 branch 在后续 head 推进后仍会 replay root，并能以相同 key/digest 在另一 unborn branch 独立初始化。新鲜证据为 API `140 passed`、storage `165 passed, 34 ignored`、public SDK `14`、local SDK `24`、Web `57`、TypeScript check 与 production build。editor 与 mutation BFF 默认拒绝，只有精确的服务端 `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE=true` 才会启用本地开发。一条 PostgreSQL parity test 在 Docker 关闭时完成编译并保持 ignored；authenticated browser mutation smoke 仍未观测。protected local API、非公开 local SDK、同源 BFF 与 shared-primitive editor 保持 authentication、RBAC、audit、rate limiting、无 cookie、private/no-store 以及 public OpenAPI/SDK exclusion；`GraphDiff::between` 仍是唯一 graph-diff calculator。

### 2026-07-18 Private Typed Uses Relationship Lifecycle / 2026-07-18 私有类型化 Uses 关系生命周期

The current private lifecycle accepts typed `AddedUsesRelationship` and `RemovedUsesRelationship` transitions only through the existing protected local command. Each request carries two distinct component identifiers and a non-null exact materialized head; the server reconstructs both endpoint states, requires both components to be active in the same Context, validates their graph facts, and derives a successor snapshot that changes only one directed `Uses` edge. Duplicate addition and missing removal fail closed, unrelated graph facts are preserved, and component removal deletes incident edges. Relationship changes remain neutral to component descriptor/body replay.

当前私有 lifecycle 只通过既有 protected local command 接受类型化 `AddedUsesRelationship` 与 `RemovedUsesRelationship` transition。每个请求只携带两个不同的 component identifier 和非空的精确 materialized head；服务端重建两个 endpoint state，要求二者都是同一 Context 中的 active component，校验其 graph fact，并派生只修改一条有向 `Uses` edge 的 successor snapshot。重复新增和移除不存在的关系都会 fail closed，无关 graph fact 保持不变，component removal 则会删除 incident edge。relationship change 不影响 component descriptor/body replay。

The relationship-only commit uses no component mutation attachment and reuses the guarded writer's exact-head compare-and-swap, branch-scoped idempotency replay, and atomic commit/snapshot/head/receipt persistence. The server-owned `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` gate keeps the local editor and mutation BFF default-deny. Public REST/OpenAPI/public SDK and `GraphDiff::between` remain unchanged; `GraphDiff` is still the sole graph-diff calculator. Docker-backed PostgreSQL runtime and authenticated browser mutation runtime for this increment are unobserved. This record closes documentation only, reports no test totals, and does not begin the next increment.

仅包含 relationship 的 commit 不使用 component mutation attachment，并复用 guarded writer 的 exact-head compare-and-swap、branch-scoped idempotency replay，以及 commit/snapshot/head/receipt 的原子持久化。由服务端拥有的 `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE` gate 使 local editor 与 mutation BFF 保持默认拒绝。public REST/OpenAPI/public SDK 与 `GraphDiff::between` 保持不变；`GraphDiff` 仍是唯一 graph-diff calculator。本增量的 Docker-backed PostgreSQL runtime 与 authenticated browser mutation runtime 均未观测。本记录只收束文档，不报告测试总数，也不启动下一增量。

### 2026-07-18 Private Benchmark Evidence Inspection / 2026-07-18 私有 Benchmark Evidence 审阅

One sealed benchmark decision is now inspectable only through the protected local route at its exact project/Context/commit/decision scope. The route requires authentication, `ContextPermission::Read`, authorization auditing, and the dedicated `BenchmarkDecisionRead` rate-limit operation; it fails closed without its optional repository and returns only identifiers, comparability, digest, status, recorded time, and metric threshold/coverage/outcomes. The non-public local SDK and same-origin BFF validate the complete redacted shape, recursively reject case/input/oracle keys, forward only request-scoped Bearer credentials, omit cookies, and mark responses private/no-store. The design-system Web inspector holds scope controls during an in-flight read and renders deterministic identifier/metric review without local policy calculation. Fresh local receipts are API `5 passed` focused and `129 passed` workspace API, storage `159 passed, 31 ignored`, public SDK `14 passed`, local SDK `6 passed`, Web `41 passed`, TypeScript checks, and a production Web build. Public REST/OpenAPI/public SDK, mutation, benchmark execution, dashboards, release evidence, and `GraphDiff` behavior remain unchanged; PostgreSQL runtime and authenticated browser-to-BFF-to-Axum E2E remain unobserved in the non-Docker environment.

现在可以仅通过 protected local route，按精确 project/Context/commit/decision scope 审阅一条已 seal 的 benchmark decision。该 route 要求 authentication、`ContextPermission::Read`、authorization auditing 与独立的 `BenchmarkDecisionRead` rate-limit operation；可选 repository 缺失时会 fail closed，并且只返回 identifier、comparability、digest、status、recorded time 及 metric threshold/coverage/outcome。非公开 local SDK 与同源 BFF 会校验完整的 redacted shape、递归拒绝 case/input/oracle key、只转发 request-scoped Bearer credential、忽略 cookie，并将 response 标记为 private/no-store。design-system Web inspector 会在 read in-flight 时锁定 scope control，以确定性的 identifier/metric 审阅呈现结果，不进行本地 policy calculation。新鲜本地回执为 API 聚焦 `5 passed` 与工作区 API `129 passed`、storage `159 passed, 31 ignored`、public SDK `14 passed`、local SDK `6 passed`、Web `41 passed`、TypeScript check 及 production Web build。public REST/OpenAPI/public SDK、mutation、benchmark execution、dashboard、release evidence 与 `GraphDiff` behavior 均未改变；在非 Docker 环境中，PostgreSQL runtime 与 authenticated browser-to-BFF-to-Axum E2E 仍未观测。

The follow-on sealed-definition metadata inspection is now a complete private read vertical slice: `definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }`. Its storage summary consumes the already-loaded sealed decision, validates immutable suite/dataset membership, orders thresholds by stable metric and datasets by identifier, and excludes cases, inputs, expected outputs, run payloads, measurements, and policy output. The existing protected reader, local SDK, same-origin BFF, and shared-primitive Web inspector retain private/no-store and no-public-contract boundaries. Fresh evidence is focused storage `2 passed`, focused API `10 passed`, `cargo test --workspace --quiet` (API `134 passed`; storage `159 passed, 32 ignored`), and `pnpm check:web`. Scoped strict Clippy remains open only because of the unrelated Rust 1.85 MSRV lint at `crates/auth/src/authorization.rs:320`; Docker/PostgreSQL runtime, authenticated browser E2E, remote CI, and production remain unobserved.

后续的 sealed-definition metadata inspection 现已成为完整的私有读取垂直切片：`definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }`。其 storage summary 会消费已经加载的 sealed decision、校验不可变 suite/dataset membership、按稳定 metric 排序 threshold、按 identifier 排序 dataset，并排除 case、input、expected output、run payload、measurement 与 policy output。既有 protected reader、local SDK、同源 BFF 与共享 primitive 的 Web inspector 保持 private/no-store 与无 public contract 的边界。新鲜证据为聚焦 storage `2 passed`、聚焦 API `10 passed`、`cargo test --workspace --quiet`（API `134 passed`；storage `159 passed, 32 ignored`）以及 `pnpm check:web`。范围化 strict Clippy 仅因无关的 Rust 1.85 MSRV lint 在 `crates/auth/src/authorization.rs:320` 处保持开放；Docker/PostgreSQL runtime、authenticated browser E2E、remote CI 与 production 仍未观测。

### 2026-07-18 Private Benchmark Evaluation Diff / 2026-07-18 私有 Benchmark Evaluation Diff

Two sealed decisions can now be compared only through the pure `BenchmarkDecisionDiff` engine. The pair-read contract obtains both exact scopes atomically under one memory lock or one PostgreSQL repeatable-read snapshot, requires an identical comparability fingerprint, and projects stored status/metric evidence without reevaluating policy. A distinct authenticated local route uses `ContextPermission::Read`, authorization auditing, and `BenchmarkDecisionDiffRead` rate limiting. Its non-public SDK, same-origin BFF, and design-system Web inspector recursively reject raw payload keys, omit cookies, return private/no-store responses, and render bilingual status/metric changes without local threshold calculation. Fresh local verification passed `cargo fmt --all -- --check`, `cargo test --workspace`, and `pnpm check:web`; focused receipts are evaluation `3 passed`, storage `15 passed`, API `131 passed`, local SDK `8 passed`, Web `47 passed`, Web TypeScript check, and a successful production Web build. The scoped Clippy command remains open only because the unrelated Rust 1.85 `contextlab-auth` MSRV lint fails at `crates/auth/src/authorization.rs:320`. PostgreSQL pair-read runtime remains compiled and ignored/unobserved while Docker is disabled; no public REST/OpenAPI/public SDK/write path, benchmark executor, release claim, or additional graph-diff calculator was added.

现在可仅通过纯粹的 `BenchmarkDecisionDiff` engine 比较两条 sealed decision。pair-read contract 会在单个 memory lock 或单个 PostgreSQL repeatable-read snapshot 内原子取得两组精确 scope，要求相同的 comparability fingerprint，并只投影已存储的 status/metric evidence，不重新评测 policy。独立的 authenticated local route 使用 `ContextPermission::Read`、authorization auditing 与 `BenchmarkDecisionDiffRead` rate limiting。其非公开 SDK、同源 BFF 与 design-system Web inspector 会递归拒绝 raw payload key、忽略 cookie、返回 private/no-store response，并以双语呈现 status/metric change，而不在本地计算 threshold。新鲜本地验证已通过 `cargo fmt --all -- --check`、`cargo test --workspace` 与 `pnpm check:web`；聚焦回执为 evaluation `3 passed`、storage `15 passed`、API `131 passed`、local SDK `8 passed`、Web `47 passed`、Web TypeScript check 与成功的 production Web build。范围化 Clippy command 仅因无关的 Rust 1.85 `contextlab-auth` MSRV lint 在 `crates/auth/src/authorization.rs:320` 失败而保持开放。Docker 关闭时 PostgreSQL pair-read runtime 已编译但保持 ignored/unobserved；未新增 public REST/OpenAPI/public SDK/write path、benchmark executor、release claim 或额外 graph-diff calculator。

`docs/superpowers/plans/2026-07-18-private-benchmark-execution-orchestration.md` is implemented. Its private service derives deterministic UUIDv5 run identities from a decision plus composite dataset/case key, validates evaluator output before any evidence write, and reuses an existing exact decision without another evaluator call. The service has no public transport, provider call, secret access, schema migration, or page-local policy; existing private inspection/diff remains its read-only review surface. Fresh local tests are evaluation `4 passed`, storage `5 passed`, and workspace API/storage counts of `131 passed` and `159 passed, 32 ignored`. PostgreSQL runtime remains compiled/ignored and unobserved while Docker is disabled.

`docs/superpowers/plans/2026-07-18-private-benchmark-execution-orchestration.md` 已实现。其私有 service 从 decision 加复合 dataset/case key 派生确定性的 UUIDv5 run identity，在任何 evidence write 前校验 evaluator output，并复用已存在的精确 decision 而不再次调用 evaluator。该 service 不含 public transport、provider call、密钥访问、schema migration 或页面局部 policy；既有私有 inspection/diff 仍是它的只读审阅面。新鲜本地测试为 evaluation `4 passed`、storage `5 passed`，以及 workspace API/storage 的 `131 passed` 和 `159 passed, 32 ignored`。Docker 关闭时 PostgreSQL runtime 仍为 compiled/ignored 且未观测。

### 2026-07-18 Admitted Private Sealed Benchmark Run Details / 2026-07-18 准入的私有已封存 Benchmark Run 明细

The admitted run-details slice is a bounded Criterion 3 read path: it resolves one sealed decision's exact ordered run membership, preserves exact project/Context/commit/run scope, and fails closed with a redacted safe summary when membership is missing or out of scope. It is intended to make persisted benchmark run details queryable and Web-visible without exposing raw benchmark payloads. It adds no public REST, OpenAPI, or public SDK write contract and does not change `GraphDiff::between`.

准入的 run-details slice 是一个有界的条件 3 read path：它解析单个 sealed decision 的精确有序 run membership，保持精确的 project/Context/commit/run scope，并在 membership 缺失或越界时以 redacted safe summary fail closed。它旨在让已持久化的 benchmark run detail 可查询并在 Web 中可见，同时不暴露 raw benchmark payload。它不新增 public REST、OpenAPI 或 public SDK write contract，也不改变 `GraphDiff::between`。

This record does not close Criterion 3 or the repository convergence goal. Fresh local evidence covers five consecutive focused storage runs after fixing a fixture-order assertion, focused API/local-SDK/Web checks, `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, `pnpm check:web` (public SDK `14`, local SDK `26`, Web `64`, and production build), and the ignored PostgreSQL parity test compiled with `--no-run`. The scoped strict Clippy command reaches only the existing Rust 1.85 MSRV lint at `crates/auth/src/authorization.rs:320`; it is outside this increment. Docker/PostgreSQL runtime and authenticated browser E2E remain unobserved.

本记录不关闭条件 3，也不关闭仓库收束目标。新鲜本地证据包括：修复 fixture 顺序断言后连续五次通过的聚焦 storage 测试、聚焦 API/local SDK/Web 检查、`cargo fmt --all -- --check`、`cargo test --workspace --quiet`、`pnpm check:web`（public SDK `14`、local SDK `26`、Web `64` 和 production build），以及 ignored PostgreSQL parity 测试的 `--no-run` 编译。范围化 strict Clippy 只到达既有 Rust 1.85 MSRV lint `crates/auth/src/authorization.rs:320`，不属于本增量。Docker/PostgreSQL runtime 与 authenticated browser E2E 仍未观测。

### 2026-07-22 Private Benchmark Decision Discovery / 2026-07-22 私有 Benchmark Decision 发现

The private decision-discovery read is a protected local, read-only discovery path for sealed
decisions at one exact project/Context/commit scope. Its separate storage port returns only safe
summaries, and the application service does not hydrate raw decision evidence. The projection
exposes only stable identifiers, suite/dataset metadata, status, recorded time, and run count.
Memory and PostgreSQL adapters preserve deterministic `recorded_at DESC, decision_id ASC`
ordering; the service rejects out-of-scope rows; and the API, non-public local SDK, BFF, and Web
parser reject raw case/input/output/measurement fields. Discovery alone did not remove the
free-text dual-decision selection gap in the evaluation-diff inspector; the separately admitted
private selection flow now replaces it and is covered by the fresh verification below.

私有 decision-discovery read 是一条 protected local、只读 discovery path，只在一个精确的
project/Context/commit scope 下发现 sealed decision。独立 storage port 只返回安全 summary，
application service 不 hydrate 原始 decision evidence。projection 只暴露稳定 identifier、
suite/dataset metadata、status、recorded time 与 run count。Memory 与 PostgreSQL adapter 保持
确定性的 `recorded_at DESC, decision_id ASC` ordering；service 拒绝越界 row；API、非公开 local
SDK、BFF 与 Web parser 拒绝 raw case/input/output/measurement field。仅 discovery 本身并未消除
evaluation-diff inspector 的 free-text 双 decision selection gap；单独准入的私有 selection flow
现已替代该入口，并由下方新鲜验证覆盖。

The preceding cross-stack receipt snapshot is historical. The later 2026-07-23 Docs/QA closure below
supersedes the present-tense totals and strict-Clippy status in this paragraph. This recorded local verification receipt
covers the safe-summary, scope-echo, strict timestamp, aggregate-count, and private selection
hardening: `cargo fmt --all -- --check` passed; `cargo test --workspace --quiet` passed (API `153
passed`; storage `166 passed, 36 ignored`); and `pnpm check:web` passed (public SDK `14`, local SDK
`51`, Web `109`, and the production Web build). Focused checks also observed strict Web timestamp
parsing `5 passed`, nested BFF security regressions `4 passed`, public OpenAPI/SDK exclusion `8
passed`, and Web selection `5 passed`. The default recursive Web test glob now collects nested BFF
tests. This is fresh local product evidence, not Criterion 3 closure: Docker/PostgreSQL runtime,
browser visual/authenticated E2E, remote CI, operator rehearsal, release, production, and
public-write evidence remain unobserved or deferred. At that recorded snapshot, strict workspace Clippy failed only at
the Rust 1.85 `Vec::is_empty` const-context compatibility lint in
`crates/auth/src/authorization.rs:320`; its minimal quality-sidecar repair is in progress. No public
REST/OpenAPI/public SDK write, Web mutation, provider call, operator transport, migration, or second
graph-diff calculator was added; `GraphDiff::between` remains the sole graph-diff calculator.

前一份 cross-stack 回执快照现仅属于历史记录。下方 2026-07-23 Docs/QA 收束记录会取代本段的当前时态
测试总数与 strict-Clippy 状态。本段记录的本地验证回执已覆盖 safe-summary、
scope-echo、严格 timestamp、aggregate-count 与私有 selection hardening：
`cargo fmt --all -- --check` 通过；`cargo test --workspace --quiet` 通过（API `153 passed`；
storage `166 passed, 36 ignored`）；`pnpm check:web` 通过（public SDK `14`、local SDK `51`、
Web `109` 以及 production Web build）。聚焦检查还观察到严格 Web timestamp parser `5 passed`、
嵌套 BFF security regression `4 passed`、public OpenAPI/SDK exclusion `8 passed` 和 Web selection
`5 passed`。默认的递归 Web test glob 现会收集嵌套 BFF test。这是新鲜的本地产品证据，而非条件 3
的关闭证明：Docker/PostgreSQL runtime、browser visual/authenticated E2E、remote CI、operator
rehearsal、release、production 与 public-write evidence 仍为未观测或延期。在该历史快照中，严格 workspace Clippy
只在 `crates/auth/src/authorization.rs:320` 的 Rust 1.85 `Vec::is_empty` const-context 兼容性
lint 处失败；其最小质量侧车修复正在进行。没有新增 public REST/OpenAPI/public SDK write、Web
mutation、provider call、operator transport、migration 或第二个 graph-diff calculator；
`GraphDiff::between` 仍是唯一 graph-diff calculator。

## 2026-07-23 Integration Evidence Closure / 2026-07-23 集成证据收束

This Integration Lead closure reconciles the current ownership and verification records; it does not close any repository completion criterion. Focused local checks cover adapter/CLI/Desktop (`6`/`4`/`3`), evaluation projection `6`, Workflow replay `11`, MCP descriptor `3`, plugin negotiation `5`, Knowledge/Memory replay `7`, and storage workspace projection `4`. `cargo fmt --all -- --check`, `cargo test --workspace --quiet` (API `154`; storage `166 passed, 36 ignored`), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo +1.85.0 check --workspace --all-targets --locked`, and `pnpm check:web` (public SDK `14`, local SDK `51`, Web `114`, production build) passed. The independent review regressions and final direct reruns supersede the earlier concurrent-write snapshot.

本次 Integration Lead 收束对齐当前所有权与验证记录，但不关闭任何仓库完成条件。聚焦本地检查覆盖 adapter/CLI/Desktop（`6`/`4`/`3`）、evaluation projection `6`、Workflow replay `11`、MCP descriptor `3`、plugin negotiation `5`、Knowledge/Memory replay `7` 与 storage workspace projection `4`。`cargo fmt --all -- --check`、`cargo test --workspace --quiet`（API `154`；storage `166 passed, 36 ignored`）、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo +1.85.0 check --workspace --all-targets --locked` 与 `pnpm check:web`（public SDK `14`、local SDK `51`、Web `114`、production build）均通过。独立审阅回归与最终直接重跑取代此前的并发写入快照。

The unauthenticated preview smoke now passes at desktop and mobile sizes after its stale text and lifecycle-gate assumptions were aligned with the current UI. It verifies key interactions, default-deny lifecycle presentation, no horizontal overflow, no console errors, and writes `target/context-workspace-desktop.png` plus `target/context-workspace-mobile.png`. Docker/PostgreSQL runtime is `unobserved` because Docker is disabled and no database command ran. Authenticated browser-to-BFF-to-Axum E2E and Git change-set evidence are `unobserved`. Remote CI, operator rehearsal, public promotion, release, and production are `deferred`.

未认证 preview smoke 现已在桌面与移动尺寸通过；其陈旧文案和 lifecycle-gate 假设已与当前 UI 对齐。验证覆盖关键交互、默认拒绝的 lifecycle 呈现、无横向溢出、无 console error，并生成 `target/context-workspace-desktop.png` 与 `target/context-workspace-mobile.png`。Docker 已禁用且未运行数据库命令，所以 Docker/PostgreSQL runtime 为 `unobserved`。authenticated browser-to-BFF-to-Axum E2E 与 Git change-set evidence 为 `unobserved`。remote CI、operator rehearsal、public promotion、release 与 production 为 `deferred`。

The bilingual Necessity Record and exact command boundary are in [the Docs/QA evidence refresh plan](../superpowers/plans/2026-07-23-docs-qa-evidence-refresh.md).

双语必要性记录与精确命令边界见 [Docs/QA 证据刷新计划](../superpowers/plans/2026-07-23-docs-qa-evidence-refresh.md)。

## 2026-07-23 Benchmark Workspace PostgreSQL and Protected Local Read / 2026-07-23 Benchmark Workspace PostgreSQL 与受保护本地读取

This is the authoritative current local receipt and supersedes earlier present-tense test totals and next-increment labels while preserving those entries as history. Migration `0019_benchmark_workspace_projection_receipts.sql` durably binds every receipt to an exact project, Context, immutable Context commit, decision, and execution cohort, and stores exact dataset-case-to-run provenance. Its composite foreign key binds the receipt's decision-evidence digest to the matching sealed decision evidence. Deferred completeness checks, append-only triggers, and immutable receipt seals enforce complete provenance before sealing, identical replay, and conflict rejection without permitting sealed receipt or case mutation.

这是当前权威本地回执；它取代此前使用当前时态的测试总数与下一增量标签，同时保留原条目作为历史。迁移 `0019_benchmark_workspace_projection_receipts.sql` 将每份 receipt 持久绑定到精确 project、Context、不可变 Context commit、decision 与 execution cohort，并保存精确 dataset-case-to-run provenance。复合外键把 receipt 的 decision-evidence digest 绑定到匹配的已封存 decision evidence。延迟完整性检查、append-only trigger 与不可变 receipt seal 要求 provenance 完整后才能封存，允许相同 replay、拒绝冲突，且不允许修改已封存 receipt 或 case。

The protected local read is `GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}`, with an optional exact baseline pair, protected authentication/RBAC/audit/rate-limit ordering, private no-store responses, and a fail-closed non-public local SDK. The default public router, checked-in OpenAPI, and public TypeScript SDK contract are unchanged. No production application service currently calls `persist_benchmark_workspace_projection`; durable rows are infrastructure and test-seeded evidence until that producer is connected.

受保护本地读取为 `GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}`，支持可选的精确 baseline pair，并保持受保护的 authentication/RBAC/audit/rate-limit 顺序、private no-store response 与 fail-closed 的非公开 local SDK。默认 public router、已检入 OpenAPI 与 public TypeScript SDK contract 均未改变。当前没有 production application service 调用 `persist_benchmark_workspace_projection`；在 producer 接入前，持久化行仍只是基础设施与测试 seed 证据。

Fresh local commands passed: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace --quiet` with API `162 passed` and storage `166 passed, 37 ignored`; `cargo +1.85.0 check --workspace --all-targets --locked`; `pnpm check:web` with public SDK `14`, local SDK `59`, Web `114`, and the production build; both required contract verifiers; and `python apps/web/verify-context-workspace.py`. One disposable PostgreSQL 16.14 `SQL_ASCII` runtime test passed against a loopback-only server, and that server was stopped afterward. This is focused migration/adapter runtime evidence, not production encoding readiness. Git and authenticated browser-to-BFF-to-Axum evidence remain `unobserved`; remote CI, operator rehearsal, public promotion, release, and production remain `deferred`.

新鲜本地命令均已通过：`cargo fmt --all -- --check`；`cargo clippy --workspace --all-targets -- -D warnings`；`cargo test --workspace --quiet`，其中 API `162 passed`、storage `166 passed, 37 ignored`；`cargo +1.85.0 check --workspace --all-targets --locked`；`pnpm check:web`，其中 public SDK `14`、local SDK `59`、Web `114`，并完成 production build；两项所需 contract verifier；以及 `python apps/web/verify-context-workspace.py`。一项 disposable PostgreSQL 16.14 `SQL_ASCII` runtime test 已在仅 loopback 的 server 上通过，随后 server 已停止。这是聚焦的 migration/adapter runtime 证据，不代表 production encoding readiness。Git 与 authenticated browser-to-BFF-to-Axum evidence 仍为 `unobserved`；remote CI、operator rehearsal、public promotion、release 与 production 仍为 `deferred`。

The next increment is the real Benchmark workspace Web BFF plus `data -> presenter -> screen` integration. It is followed by the missing production projection producer that calls `persist_benchmark_workspace_projection`. Neither queue item closes Criterion 3. The long-term goal remains active.

下一增量是接入真实 Benchmark workspace Web BFF，并完成 `data -> presenter -> screen` 集成；随后补齐调用 `persist_benchmark_workspace_projection` 的 production projection producer。两项排队工作都不会关闭条件 3。长期目标保持 active。

## 2026-07-23 Benchmark Web and Durable Execution Projection Closure / 2026-07-23 Benchmark Web 与持久执行投影收束

This is the authoritative current Criterion 3 receipt. It supersedes the immediately preceding present-tense statements that the real Benchmark workspace Web integration and production application caller are next or missing; those statements remain preserved as historical records. The real protected Web BFF and `data -> presenter -> screen` workflow are complete as a private, local, read-only surface. The bearer token exists only in browser request memory and is forwarded to the same-origin BFF in the `Authorization` header; cookies are neither accepted as authentication nor forwarded upstream, upstream fetches use `credentials: "omit"`, and every BFF response is `Cache-Control: private, no-store`. The Web parses and presents the server-owned redacted projection without recomputing benchmark policy. The default public router, checked-in public OpenAPI, and public TypeScript SDK are unchanged, and `GraphDiff::between` remains the sole graph-diff calculator.

这是当前权威的条件 3 回执。它取代紧邻上方把真实 Benchmark workspace Web 集成与 production application caller 表述为“下一步”或“缺失”的当前时态判断，同时保留原文作为历史记录。真实 protected Web BFF 与 `data -> presenter -> screen` workflow 已完成，并严格限定为 private、local、read-only surface。Bearer token 只存在于浏览器请求内存中，并通过 `Authorization` header 转发给同源 BFF；cookie 既不作为认证凭据，也不会向上游转发，上游 fetch 使用 `credentials: "omit"`，所有 BFF response 均为 `Cache-Control: private, no-store`。Web 只解析和呈现 server-owned 的脱敏 projection，不重新计算 benchmark policy。默认 public router、已检入的 public OpenAPI 与 public TypeScript SDK 均未改变，`GraphDiff::between` 仍是唯一 graph-diff calculator。

`BenchmarkExecutionService` is now the durable projection producer for both created and replayed execution outcomes. The materialization path reloads the exact sealed suite/dataset definitions and sealed runs bound to the decision evidence, reconstructs the deterministic execution receipt, and delegates persistence to the existing `BenchmarkWorkspaceProjectionV1Writer` through `persist_benchmark_workspace_projection`. If the first projection write fails after decision evidence is sealed, an evidence-backed replay repairs the missing projection without another evaluator invocation. Replay identity uses PostgreSQL-compatible microsecond timestamp normalization, and the memory and PostgreSQL adapters preserve the same producer contract and projection semantics.

`BenchmarkExecutionService` 现已成为 created 与 replayed 两类 execution outcome 的持久投影 producer。物化路径会重新加载与 decision evidence 精确绑定的 sealed suite/dataset definition 和 sealed run，重建确定性的 execution receipt，并通过 `persist_benchmark_workspace_projection` 委托给既有 `BenchmarkWorkspaceProjectionV1Writer` 完成持久化。若 decision evidence 封存后首次 projection write 失败，基于既有 evidence 的 replay 会在不再次调用 evaluator 的情况下修复缺失 projection。replay identity 使用与 PostgreSQL 兼容的微秒级 timestamp normalization，memory 与 PostgreSQL adapter 保持相同的 producer contract 与 projection 语义。

Fresh local evidence is evaluation-focused `15/15`, storage-focused `38/38`, the confirmed full storage suite `166 passed, 38 ignored`, `pnpm check:web` with public SDK `14`, local SDK `59`, Web `138`, and the full production Web build, workspace API `162 passed`, strict workspace Clippy, the locked Rust `1.85.0` workspace check, both required contract verifiers, and Playwright desktop/mobile checks with no horizontal overflow or console errors. One PostgreSQL 16.14 `SQL_ASCII` producer runtime test passed against a loopback-only server, and that server was stopped afterward. This proves the focused producer runtime only; `SQL_ASCII` is not production encoding-readiness evidence.

新鲜本地证据包括 evaluation 聚焦测试 `15/15`、storage 聚焦测试 `38/38`、已确认的 storage 全量 `166 passed, 38 ignored`、`pnpm check:web`（public SDK `14`、local SDK `59`、Web `138`，并完成完整 production Web build）、workspace API `162 passed`、严格 workspace Clippy、锁定 Rust `1.85.0` 的 workspace check、两项所需 contract verifier，以及桌面/移动端 Playwright 检查，且无横向溢出或 console error。一项 PostgreSQL 16.14 `SQL_ASCII` producer runtime test 已在仅 loopback 的 server 上通过，随后 server 已停止。这只证明聚焦 producer runtime；`SQL_ASCII` 不构成 production encoding readiness 证据。

Authenticated browser-to-BFF-to-Axum runtime and Git change-set evidence remain `unobserved`; external deployment, public promotion, release, and production evidence remain `deferred`. Criterion 3 is advanced but not closed, and the long-term goal remains active. The current completion criteria expose no stricter dependency-ready local predecessor, so the next Criterion 3 increment is private benchmark dataset/suite authoring plus guarded version binding, admitted only after a new bilingual Necessity Record.

authenticated browser-to-BFF-to-Axum runtime 与 Git change-set evidence 仍为 `unobserved`；外部 deployment、public promotion、release 与 production evidence 仍为 `deferred`。条件 3 得到推进但尚未关闭，长期目标保持 active。当前收束条件没有显示更严格且依赖就绪的本地前置，因此下一项条件 3 增量确定为私有 benchmark dataset/suite authoring 加 guarded version binding，并且只有在新增一份双语 Necessity Record 后才可准入。
- 2026-07-27 authoritative Criterion 3 progress receipt: private benchmark definition authoring now has a validated schema-1 command, immutable exact project/Context/commit binding, memory/PostgreSQL atomic persistence, branch-head and transaction-local write authorization guards, issuer-scoped idempotency, exact read/list ports, and migration 0020 append-only/composite-FK enforcement. Fresh evidence is authoring 3/3, migration 6/6, strict storage Clippy, workspace API 162 and storage 166 passed/38 ignored, plus 26/26 named disposable storage tests on a native UTF-8 PostgreSQL 16.14 loopback cluster with per-test schema reset. The cluster and service were stopped. Wave 2 private local API/SDK/BFF/Web authoring transport remains open; public REST/OpenAPI/public SDK writes and Criterion 3 remain open. Browser-auth, Git, remote, operator, release, and production evidence remain unobserved or deferred. The long-term goal remains active.

- 2026-07-27 条件 3 权威进展回执：私有 benchmark definition authoring 现具备经过校验的 schema-1 command、不可变 exact project/Context/commit binding、Memory/PostgreSQL 原子持久化、branch-head 与 transaction-local write authorization guard、issuer-scoped idempotency、exact read/list port，以及迁移 0020 的 append-only/复合外键约束。新鲜证据为 authoring 3/3、migration 6/6、严格 storage Clippy、workspace API 162 与 storage 166 passed/38 ignored，以及每个 test 前重置 schema 后 native UTF-8 PostgreSQL 16.14 loopback 的 26/26 个指定 disposable storage test 全部通过。cluster 与 service 已停止。Wave 2 private local API/SDK/BFF/Web authoring transport 仍未完成；public REST/OpenAPI/public SDK write 与条件 3 仍开放。browser-auth、Git、remote、operator、release 与 production evidence 仍为 unobserved 或 deferred。长期目标保持 active。

- 2026-07-27 Criterion 3 Wave 2 closure receipt: the private authoring transport and editor are now complete. The canonical protected local route is `POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`; the non-public local SDK, same-origin BFF, and Web `data -> presenter -> screen -> editor` composition preserve its exact scope, request-memory Bearer behavior, cookie omission, `credentials: "omit"`, and `private, no-store` response contract. The context-only BFF route is retired with a fail-closed `410 benchmark_definition_route_gone` and no upstream call. Server-owned authentication, authorization/audit, dedicated quota, idempotency, branch-head checks, atomic storage, redacted responses, and bilingual UI states remain intact. Public REST/OpenAPI/public SDK writes, provider execution, and `GraphDiff::between` are unchanged. Criterion 3 is advanced but remains open because benchmark execution breadth, authenticated browser runtime, Git binding, external release evidence, and other completion items are not all closed.

- 2026-07-27 条件 3 Wave 2 收束回执：私有 authoring transport 与 editor 现已完成。canonical protected local route 为 `POST /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-definition-bindings`；非公开 local SDK、同源 BFF 与 Web `data -> presenter -> screen -> editor` composition 保持精确 scope、request-memory Bearer、cookie omission、`credentials: "omit"` 与 `private, no-store` response contract。context-only BFF route 已退役，fail-closed 返回 `410 benchmark_definition_route_gone` 且不调用 upstream。服务端 authentication、authorization/audit、专用 quota、idempotency、branch-head check、原子 storage、脱敏 response 与双语 UI state 保持不变。Public REST/OpenAPI/public SDK write、provider execution 与 `GraphDiff::between` 均未改变。条件 3 得到推进但仍开放，因为 benchmark execution breadth、authenticated browser runtime、Git binding、外部 release evidence 与其他 completion item 尚未全部收束。

### 2026-07-27 Binding Inspection Follow-on Receipt / 2026-07-27 Binding Inspection 后续回执

The private exact binding read and selection surface is freshly verified. `cargo fmt --all -- --check`
passed; focused storage tests passed with `167 passed, 39 ignored`; the Wave 2 verifier reported
`wave2_local_contracts=passed` and `graph_diff_calculators=passed count=1`; and `pnpm check:web`
passed with public SDK `14`, local SDK `70`, Web `156`, and the production build. The live verifier's
`overall=unobserved` is limited to unavailable Git change-set evidence. No Web server was running for
the local Playwright verifier. Docker/PostgreSQL runtime, authenticated browser runtime, Git binding,
remote CI, operator rehearsal, release, and production remain ignored, unobserved, or deferred. Six
requested `gpt-5.6-luna` review workers were rejected because the agent thread limit was full; that is
recorded as scheduling failure only, not product evidence. The next admitted local increment is the
provider-free exact binding execution-selection contract in
`docs/superpowers/plans/2026-07-27-private-benchmark-binding-execution-selection.md`.

私有 exact binding read 与 selection surface 已获得新鲜验证。`cargo fmt --all -- --check` 通过；storage 聚焦
测试通过，结果为 `167 passed, 39 ignored`；Wave 2 verifier 报告 `wave2_local_contracts=passed` 与
`graph_diff_calculators=passed count=1`；`pnpm check:web` 通过，其中 public SDK `14`、local SDK `70`、
Web `156`，并完成 production build。live verifier 的 `overall=unobserved` 仅限 Git change-set evidence
不可用。local Playwright verifier 未运行，因为没有启动 Web server。Docker/PostgreSQL runtime、authenticated
browser runtime、Git binding、remote CI、operator rehearsal、release 与 production 仍为 ignored、unobserved
或 deferred。请求的 6 个 `gpt-5.6-luna` review worker 因 agent thread limit 已满而被拒绝；这里只记录调度失败，
不作为产品证据。下一项准入的本地增量是
`docs/superpowers/plans/2026-07-27-private-benchmark-binding-execution-selection.md` 中的 provider-free
exact binding execution-selection contract。
### 2026-07-27 Private Binding Execution Selection / 2026-07-27 私有绑定执行选择

The exact immutable benchmark binding is now a provider-free execution input. The new
`BenchmarkDefinitionBindingExecutionSelection` validates project/Context/commit scope, preserves
stable suite and dataset membership, assembles the existing deterministic `BenchmarkExecutionPlan`,
and exposes only safe identity/count accessors. `BenchmarkExecutionRequest::from_definition_binding`
passes that selection into the existing private `BenchmarkExecutionService`; selected execution and
replay use the binding's plan rather than a caller-supplied mutable suite lookup. Its custom Debug
output omits case payloads and expected outputs. No API, SDK, BFF, Web, migration, provider, or public
surface changed, and `GraphDiff::between` remains the sole graph-diff calculator.

精确不可变 benchmark binding 现已成为 provider-free execution input。新的
`BenchmarkDefinitionBindingExecutionSelection` 校验 project/Context/commit scope，保持稳定的 suite 与 dataset
membership，组装既有确定性的 `BenchmarkExecutionPlan`，并只暴露安全的 identity/count accessor。
`BenchmarkExecutionRequest::from_definition_binding` 将 selection 传入既有私有 `BenchmarkExecutionService`；
selected execution 与 replay 使用 binding 自身的 plan，而不是 caller 提供的 mutable suite lookup。其 custom
Debug output 排除 case payload 与 expected output。没有改变 API、SDK、BFF、Web、migration、provider 或 public
surface，`GraphDiff::between` 仍是唯一 graph-diff calculator。

Fresh evidence is selection `2/2`, execution `11/11`, `cargo fmt --all -- --check`, workspace Rust with API
`170 passed` and storage `167 passed, 39 ignored`, strict workspace Clippy, locked Rust `1.85.0`, and
`pnpm check:web` with public SDK `14`, local SDK `70`, Web `156`, and production build. PostgreSQL runtime,
authenticated browser runtime, Git binding, remote CI, operator rehearsal, release, and production remain
unobserved or deferred. The next admitted local increment is the protected local Benchmark execution adapter
recorded in `docs/superpowers/plans/2026-07-27-private-benchmark-execution-adapter.md`; Criterion 3 and the
long-term goal remain open.

新鲜证据为 selection `2/2`、execution `11/11`、`cargo fmt --all -- --check`、workspace Rust（API `170 passed`；
storage `167 passed, 39 ignored`）、strict workspace Clippy、锁定 Rust `1.85.0`，以及 `pnpm check:web`（public SDK
`14`、local SDK `70`、Web `156`，并完成 production build）。PostgreSQL runtime、authenticated browser runtime、
Git binding、remote CI、operator rehearsal、release 与 production 仍为 unobserved 或 deferred。下一项准入的本地
增量是 `docs/superpowers/plans/2026-07-27-private-benchmark-execution-adapter.md` 记录的 protected local
Benchmark execution adapter；条件 3 与长期目标仍保持开放。

### 2026-07-27 Workflow Binding Read Revalidation / 2026-07-27 Workflow Binding Read 复核

The private Workflow context-binding read is the current local Web integration receipt: API `4 passed`, storage `3 passed`, local SDK `70 passed`, BFF route `9 passed`, focused Web binding/presenter/inspector `8 passed`, `cargo fmt --all -- --check`, and `pnpm check:web` with public SDK `14`, local SDK `70`, Web `160`, and a successful production build. The receipt covers exact Context/commit scope, redaction, deterministic ordering, request-memory Bearer forwarding, cookie omission, typed failures, `private, no-store`, and shared `data -> presenter -> screen` composition. No public REST/OpenAPI/public SDK write was added, and `GraphDiff::between` remains the sole graph-diff calculator. PostgreSQL-backed authenticated runtime, authenticated browser/visual E2E, Git binding, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; Criterion 3 and the long-term goal remain open. The next admitted local increment is the protected Benchmark execution adapter with its existing bilingual Necessity Record.

私有 Workflow context-binding read 是当前本地 Web 集成回执：API `4 passed`、storage `3 passed`、local SDK `70 passed`、BFF route `9 passed`、聚焦 Web binding/presenter/inspector `8 passed`、`cargo fmt --all -- --check`，以及 `pnpm check:web`（public SDK `14`、local SDK `70`、Web `160`，并成功完成 production build）。该回执覆盖精确 Context/commit scope、脱敏、确定性排序、request-memory Bearer 转发、cookie omission、typed failure、`private, no-store` 与共享 `data -> presenter -> screen` 组合。没有新增 public REST/OpenAPI/public SDK write，`GraphDiff::between` 仍是唯一 graph-diff calculator。PostgreSQL-backed authenticated runtime、authenticated browser/visual E2E、Git binding、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`；条件 3 与长期目标仍开放。下一项准入的本地增量是已有双语 Necessity Record 的 protected Benchmark execution adapter。

### 2026-07-27 Private Benchmark Execution Adapter Receipt / 2026-07-27 私有 Benchmark 执行适配器回执

The private local execution write boundary is now implemented without changing the public API
surface. The protected route authenticates and authorizes `ContextPermission::Write` before quota
and body parsing, loads one exact immutable binding scope, and delegates execution to the reusable
Rust service. Typed `IdempotencyKey` and canonical `RequestDigest` values are persisted by memory and
PostgreSQL through `benchmark_execution_idempotency` migration `0021`; identical retries replay
stored evidence and changed payloads conflict before evaluator invocation. Responses are redacted and
`private, no-store`; raw case/input/output, provider access, and client-side policy/Diff are excluded.

私有 local execution write boundary 现已实现，但没有改变 public API surface。protected route 在 quota 与 body
parsing 前完成 authentication 和 `ContextPermission::Write` authorization，加载一条精确 immutable binding scope，
并将 execution 委托给可复用 Rust service。typed `IdempotencyKey` 与 canonical `RequestDigest` 通过迁移 `0021`
的 `benchmark_execution_idempotency` 由 memory 与 PostgreSQL 持久化；相同 retry 回放已存储 evidence，不同
payload 会在 evaluator invocation 前返回 conflict。response 已脱敏并使用 `private, no-store`；raw case/input/output、
provider access 与 client-side policy/Diff 均排除。

Fresh evidence is API execution `3 passed`, storage `169 passed, 39 ignored`, storage/API compile,
and `cargo fmt --all -- --check`. PostgreSQL runtime, authenticated browser runtime, Git change-set,
remote CI, operator rehearsal, release, and production promotion remain `ignored`, `unobserved`, or
`deferred`; Criterion 3 is advanced but not closed and the long-term goal remains active.

新鲜证据为 API execution `3 passed`、storage `169 passed, 39 ignored`、storage/API compile 与
`cargo fmt --all -- --check`。PostgreSQL runtime、authenticated browser runtime、Git change-set、remote CI、
operator rehearsal、release 与 production promotion 仍为 `ignored`、`unobserved` 或 `deferred`；条件 3 得到推进
但尚未关闭，长期目标保持 active。

### 2026-07-27 Private Benchmark Execution Web Adapter / 2026-07-27 私有 Benchmark 执行 Web 适配器

The protected execution adapter now has a complete local SDK/BFF/Web composition. The local SDK
enforces canonical UUIDs, the shared `0.0..=2.0` temperature invariant, ordered-unique dataset
identifiers, request-memory Bearer credentials, no cookies, no-store transport, frozen redacted
receipts, and typed replay conflicts. The BFF validates the exact project/Context/commit path,
allowlists stable bilingual error messages, rejects raw or unstable responses, and never forwards
upstream diagnostics. The default-off Web execution inspector is mounted in the existing Benchmark
workspace region behind the server-controlled local gate; it presents receipt metadata only and
does not implement policy or Diff logic. Public REST/OpenAPI/public SDK write surfaces and
`GraphDiff::between` are unchanged.

受保护 execution adapter 现已具备完整的 local SDK/BFF/Web 组合。local SDK 强制 canonical UUID、共享的
`0.0..=2.0` temperature invariant、有序唯一 dataset identifier、request-memory Bearer credential、无 cookie、
no-store transport、冻结的脱敏 receipt 与 typed replay conflict。BFF 校验精确 project/Context/commit path，
只允许稳定双语 error message，拒绝 raw 或不稳定 response，且绝不转发 upstream diagnostic。默认关闭的 Web
execution inspector 已挂载到现有 Benchmark workspace region，并由服务端控制的 local gate 保护；它只呈现 receipt
metadata，不实现 policy 或 Diff logic。Public REST/OpenAPI/public SDK write surface 与 `GraphDiff::between` 均未改变。

Fresh evidence: `pnpm check:web` passed with public SDK `14`, local SDK `73`, Web `173`, and
the production build; local SDK and Web typechecks passed; `cargo fmt --all -- --check` passed;
`cargo test --workspace --quiet` passed with API `176` and storage `169 passed, 39 ignored`;
strict workspace Clippy passed; and locked Rust `1.85.0` workspace check passed. PostgreSQL
runtime, authenticated browser-to-BFF-to-Axum runtime, Git binding, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. Criterion 3 is advanced but not closed;
the long-term goal remains active.

新鲜证据为：`pnpm check:web` 通过（public SDK `14`、local SDK `73`、Web `173`，并完成 production build）；local SDK
与 Web typecheck 通过；`cargo fmt --all -- --check` 通过；`cargo test --workspace --quiet` 通过，其中 API `176`、
storage `169 passed, 39 ignored`；strict workspace Clippy 通过；锁定 Rust `1.85.0` workspace check 通过。
PostgreSQL runtime、authenticated browser-to-BFF-to-Axum runtime、Git binding、remote CI、operator rehearsal、release
与 production 仍为 `unobserved` 或 `deferred`。条件 3 得到推进但尚未关闭，长期目标保持 active。

### 2026-07-27 Knowledge/Memory Projection Boundary and Exact-Commit Persistence Gap / 2026-07-27 Knowledge/Memory 投影边界与精确提交持久化缺口

The private Knowledge/Memory projection is now a validated read-only contract, not a completion
claim for persisted knowledge or memory. The local SDK and same-origin Web adapter require a
canonical `project_id`, `context_id`, and `commit_id`; `source_project_id` and `source_commit_id`
must exactly echo that envelope; knowledge and memory scopes must resolve to the requested Context;
replay identity, citation order, schema versions, retention decision, and replay state are checked
deterministically; and responses remain redacted, private, and `no-store`. `content_fingerprint` is
metadata only and does not expose raw document, chunk, memory, input, or model content. The current
read path therefore advances Criteria 1 and 4 as a safe inspection/projection slice while keeping
provider calls, ingestion, retrieval execution, public writes, and mutation transport out of scope.

当前私有 Knowledge/Memory projection 已成为经过校验的只读 contract，但不代表知识或记忆数据已经完成持久化。
local SDK 与同源 Web adapter 要求 canonical `project_id`、`context_id`、`commit_id`；
`source_project_id` 与 `source_commit_id` 必须精确回显 envelope；knowledge 与 memory scope 必须解析到请求的
Context；replay identity、citation order、schema version、retention decision 与 replay state 均进行确定性校验；
response 继续保持脱敏、private 与 `no-store`。`content_fingerprint` 仅是 metadata，不暴露原始 document、chunk、
memory、input 或 model content。因此当前 read path 推进了条件 1 与条件 4 的安全 inspection/projection 切片，
但 provider call、ingestion、retrieval execution、public write 与 mutation transport 仍明确不在范围内。

Fresh local evidence for this boundary is `pnpm check:web` passed with public SDK `14`, local SDK `85`, Web `179`,
and the production build; Rust workspace tests passed with API `182` and storage `169 passed, 39 ignored`;
`cargo fmt --all -- --check` and strict offline Clippy passed; focused API projection tests passed `3`, Knowledge
Context projection tests passed `4`, and Knowledge replay bridge tests passed `9`. These receipts are local product
evidence only. Docker/PostgreSQL runtime, authenticated browser E2E, Git change-set, remote CI, operator rehearsal,
release, and production evidence remain `unobserved` or `deferred`; no external receipt is implied.

该边界的新鲜本地证据为：`pnpm check:web` 通过（public SDK `14`、local SDK `85`、Web `179`，并完成 production build）；
Rust workspace test 通过（API `182`、storage `169 passed, 39 ignored`）；`cargo fmt --all -- --check` 与 strict offline
Clippy 通过；聚焦 API projection test `3`、Knowledge Context projection test `4`、Knowledge replay bridge test `9`
通过。这些回执仅是本地产品证据。Docker/PostgreSQL runtime、authenticated browser E2E、Git change-set、remote CI、
operator rehearsal、release 与 production evidence 仍为 `unobserved` 或 `deferred`；不暗示任何外部 receipt。

The remaining exact-commit persistence gap is explicit: replace the current request-time/fixture-backed projection
source with a reusable storage projection repository that reads complete, redacted Knowledge/Memory state by the exact
`(project, Context, immutable commit)` scope, preserves deterministic replay identity and ordering, and provides
Memory/PostgreSQL parity. The repository must prove missing-scope and mismatched-source failures closed, without
returning raw content. This next increment is private read-only infrastructure and its tests only: it does not add
public REST/OpenAPI/SDK writes, Web mutation controls, provider access, Docker or production claims, or another
GraphDiff implementation. `GraphDiff::between` remains the sole graph-diff calculator. The Criterion 1/4 work and the
long-term goal remain active until the broader completion criteria have fresh, scope-matched evidence.

剩余的 exact-commit persistence gap 已明确：用可复用的 storage projection repository 替换当前 request-time/fixture-backed
projection source，按精确的 `(project, Context, immutable commit)` scope 读取完整但脱敏的 Knowledge/Memory state，保持
确定性的 replay identity 与 ordering，并提供 Memory/PostgreSQL parity。repository 必须证明 missing-scope 与
mismatched-source 会 fail closed，且不返回 raw content。下一增量仅限私有只读 infrastructure 与 tests：不新增 public
REST/OpenAPI/SDK write、Web mutation control、provider access、Docker 或 production claim，也不增加另一套
GraphDiff 实现。`GraphDiff::between` 仍是唯一 graph-diff calculator。在更广泛的 completion criteria 获得新鲜且范围匹配
的证据前，条件 1/4 工作与长期目标继续保持 active。

### 2026-07-27 Exact-Commit Knowledge/Memory Persistence Receipt / 2026-07-27 Knowledge/Memory 精确提交持久化回执

The exact-commit persistence gap has been closed at the local storage-contract boundary. The
reusable `KnowledgeMemoryProjectionV1Repository` persists a redacted
`KnowledgeMemoryContextProjectionV1` at exact `(project, Context, commit)` scope, provides
Memory/PostgreSQL implementations, and preserves immutable `Created`/`Replayed`/`Conflict` behavior.
Migration `0022` is append-only and uses composite foreign keys to prevent project, Context, and
commit scope drift. The private API uses the storage-backed adapter for PostgreSQL mode; no public
REST/OpenAPI/SDK write, Web mutation, provider call, raw content, or second GraphDiff calculator was
added. Criterion 1 and replayable-history evidence advance, but the broader criteria remain open.

本次 exact-commit persistence gap 已在本地 storage-contract boundary 收束。可复用的
`KnowledgeMemoryProjectionV1Repository` 在精确 `(project, Context, commit)` scope 保存脱敏
`KnowledgeMemoryContextProjectionV1`，提供 Memory/PostgreSQL 实现，并保持 immutable 的
`Created`/`Replayed`/`Conflict` 行为。迁移 `0022` 具备 append-only 保护，并使用 composite foreign key
防止 project、Context 与 commit scope drift。private API 在 PostgreSQL mode 使用 storage-backed adapter；没有
新增 public REST/OpenAPI/SDK write、Web mutation、provider call、raw content 或第二个 GraphDiff calculator。
条件 1 与可回放历史证据得到推进，但更广泛条件仍开放。

Fresh local evidence: focused storage `2/2`, focused API `6/6`, workspace Rust API `182 passed`,
storage `171 passed, 39 ignored`, `cargo fmt --all -- --check`, strict offline Clippy, and
`pnpm check:web` with public SDK `14`, local SDK `85`, Web `179`, and production build. `psql` is
unavailable and Docker/virtualization is disabled; PostgreSQL runtime, authenticated browser E2E,
Git, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or
`deferred`. These receipts are local non-production evidence only, and the long-term goal remains
active.

新鲜本地证据为 focused storage `2/2`、focused API `6/6`、workspace Rust API `182 passed`、storage
`171 passed, 39 ignored`、`cargo fmt --all -- --check`、strict offline Clippy，以及
`pnpm check:web`（public SDK `14`、local SDK `85`、Web `179`、production build）。`psql` 不可用且
Docker/virtualization 当前关闭；PostgreSQL runtime、authenticated browser E2E、Git、remote CI、operator
rehearsal、release 与 production evidence 仍为 `unobserved` 或 `deferred`。这些仅是本地非生产证据，长期目标
保持 active。

**Next criterion pointer / 下一条件指针:** Add a bilingual Necessity Record for commit-associated
ContextGraph snapshot domain/repository persistence. Only after its immutable commit binding,
replay, and focused contract tests pass may version-backed graph comparison be admitted. This
pointer does not authorize public writes or another graph-diff implementation.

**下一条件指针：** 为 commit-associated ContextGraph snapshot domain/repository persistence 增加双语
Necessity Record。只有在 immutable commit binding、replay 与聚焦 contract test 通过后，才准入 version-backed
graph comparison。该指针不授权 public write，也不授权新增另一套 graph-diff implementation。

## Branch-Head Read Contract / Branch-Head 读取契约

The private branch-head prerequisite is locally verified for the replayable version-history
criterion. `ContextBranchHead` and `ContextBranchRepository` preserve exact Context ownership,
nullable unborn heads, typed branch names, monotonic revisions, deterministic ordering, and
fail-closed malformed, overflow, and cross-Context handling across Memory and PostgreSQL adapters.
The guarded writer remains the only branch-head mutation path. This evidence does not close branch
creation, merge, rollback, public writes, release, or production criteria.

可回放版本历史条件所需的 private branch-head 前置已在本地验证。`ContextBranchHead` 与
`ContextBranchRepository` 在 Memory 与 PostgreSQL adapter 间保持 exact Context ownership、nullable unborn head、
typed branch name、单调 revision、确定性 ordering，以及 malformed、overflow 和 cross-Context 情况的 fail-closed
处理。Guarded writer 仍是唯一 branch-head mutation path。本证据不关闭 branch creation、merge、rollback、public
write、release 或 production 条件。

Observed local receipts are focused `branch_head` `2 passed`, Memory adapter regressions `3 passed`,
storage `186 passed, 39 ignored`, workspace Rust `186 passed, 39 ignored`, format, strict offline
Clippy, locked Rust `1.85.0` check, and Web `15` public SDK, `92` local SDK, `185` Web tests plus
production build. PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`; the long-term goal remains active.

新鲜本地回执为 focused `branch_head` `2 passed`、Memory adapter regression `3 passed`、storage `186 passed, 39 ignored`、
workspace Rust `186 passed, 39 ignored`、format、strict offline Clippy、锁定 Rust `1.85.0` check，以及 Web `15` 项
public SDK、`92` 项 local SDK、`185` 项 Web test 与 production build。PostgreSQL runtime、authenticated browser、Git、
remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`；长期目标保持 active。

## 2026-07-27 Commit-Associated ContextGraph Snapshot and Version-Backed Diff Receipt / 2026-07-27 Commit 关联 ContextGraph Snapshot 与版本图 Diff 回执

This supersedes the preceding pointer as the current local receipt. The reusable storage contract
now binds every immutable graph snapshot to `CommitGraphSnapshotScope(ProjectId, ContextId, CommitId)`.
The repository separately exposes durable Context ownership for guarded creation and strict existing
commit-scope resolution for reads. Memory and PostgreSQL implementations share deterministic keys,
schema-V1 validation, immutable replay/conflict classification, and fail-closed unknown-scope errors.

本节取代上方“下一项”指针，成为当前本地回执。可复用 storage contract 现将每个不可变 graph snapshot 绑定到
`CommitGraphSnapshotScope(ProjectId, ContextId, CommitId)`。repository 为 guarded creation 单独提供 durable Context
ownership，并为 read 提供严格的既有 commit-scope resolution。Memory 与 PostgreSQL 实现共享确定性 key、schema-V1
validation、immutable replay/conflict classification 与 unknown-scope fail-closed error。

The existing version-backed graph-diff API now consumes those exact scopes and delegates all graph
comparison to `GraphDiff::between`. Its public read surface remains exposed through the checked-in
OpenAPI and public SDK; no public write or mutation surface was added. API response compatibility is
preserved by flattening V1 scope fields, and the path, query names, and Web mutation boundary are
unchanged. Focused API scope coverage is `2 passed`, storage snapshot-repository coverage is `5
passed`, and the latest observed receipts are local API `184 passed`, storage `179 passed, 39 ignored`,
strict offline Clippy, format, and `pnpm check:web` with SDK/Web tests and production build.

现有 version-backed graph-diff API 现消费这些 exact scope，并将全部 graph comparison 委托给 `GraphDiff::between`。
其 public read surface 仍通过已检入的 OpenAPI 与 public SDK 暴露；没有新增 public write 或 mutation surface。通过
flatten V1 scope field 保持 API response compatibility，path、query name 与 Web mutation boundary 未改变。聚焦 API
scope coverage 为 `2 passed`，storage snapshot-repository coverage 为 `5 passed`；最近观测到的回执仅为本地 API
`184 passed`、storage `179 passed, 39 ignored`、strict offline Clippy、format，以及包含 SDK/Web test 与 production
build 的 `pnpm check:web`。

The protected-local read is a separate local inspection receipt. It advances only the related
criteria and does not close Criteria 1, 2, or 4, promote the read path to a public write/mutation
surface, or substitute for external evidence. The receipts above are local, non-production evidence;
release and production evidence remain deferred/unobserved.

受保护的 protected-local read 是独立的本地 inspection 回执，只推进相关条件，不能关闭条件 1、2、4，也不会把该
read path 提升为 public write/mutation surface，或替代外部证据。上方回执仅是本地、非生产证据；release 与 production
证据仍为 deferred/unobserved。

This advances Criteria 1, 2, and 4 but does not close them or the repository. PostgreSQL runtime is
`unobserved` because `psql` is unavailable and Docker/virtualization is disabled. Authenticated
browser runtime, Git change-set, remote CI, operator rehearsal, release, and production evidence are
`unobserved` or `deferred`. The long-term goal remains active. The next admitted local pointer is a
private semantic/behavior/evaluation diff contract, after a new bilingual Necessity Record.

本回执推进条件 1、2、4，但未关闭这些条件或整个仓库。由于 `psql` 不可用且 Docker/virtualization 已关闭，PostgreSQL
runtime 为 `unobserved`。authenticated browser runtime、Git change-set、remote CI、operator rehearsal、release 与
production evidence 为 `unobserved` 或 `deferred`。长期目标保持 active。下一项本地准入指针是 private
semantic/behavior/evaluation diff contract，开始前必须新增双语 Necessity Record。

## 2026-07-28 Private Commit-Scoped Diff Snapshot Persistence Admitted Increment / 2026-07-28 私有按 Commit 绑定 Diff Snapshot 持久化准入增量

This admitted increment points to [`docs/superpowers/plans/2026-07-28-private-commit-scoped-diff-snapshot-persistence.md`](../superpowers/plans/2026-07-28-private-commit-scoped-diff-snapshot-persistence.md). Existing local evidence covers the diff core and typed exact-scope contract, but the exact semantic/behavior/evaluation persistence bridge is still missing. Storage implementation is in progress and is not passed evidence; this entry must not be read as a green storage receipt.

本次准入增量指向 [`docs/superpowers/plans/2026-07-28-private-commit-scoped-diff-snapshot-persistence.md`](../superpowers/plans/2026-07-28-private-commit-scoped-diff-snapshot-persistence.md)。现有本地证据覆盖 diff core 与 typed exact-scope contract，但 exact semantic/behavior/evaluation persistence bridge 仍然缺失。storage implementation 正在进行中，不是 passed evidence；本条不得解读为 storage 已通过的回执。

This increment advances Criteria 2 and 4 only. It does not close either criterion or any completion condition. It adds no public transport, Web mutation, provider/evaluator call, or other public surface. `GraphDiff::between` remains the sole graph-diff calculator, and no second comparison implementation is admitted. PostgreSQL runtime, authenticated browser runtime, Git change-set, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or `deferred` according to the actual current state; no external, production, or secret receipt is implied.

本增量仅推进条件 2 与条件 4，不关闭任一条件或任何收束条件。不新增 public transport、Web mutation、provider/evaluator call 或其他 public surface。`GraphDiff::between` 仍是唯一 graph-diff calculator，不准入第二套 comparison implementation。PostgreSQL runtime、authenticated browser runtime、Git change-set、remote CI、operator rehearsal、release 与 production evidence 按当前真实状态仍为 `unobserved` 或 `deferred`；不暗示任何 external、production 或 secret receipt。

## Current Private Benchmark Regression/Scorecard/Evaluation-Diff Closure / 当前私有 Benchmark 回归、Scorecard 与 Evaluation-Diff 收束

The local benchmark closure is now fresh at the contract boundary. The existing evaluation domain
remains the sole `BenchmarkSuite -> Scorecard -> RegressionDecision` policy path. Storage contract
coverage now includes same-commit exact decision comparison and a complete redacted workspace
projection: all project/Context/commit/cohort scope dimensions fail closed, baseline/revised values
and statuses remain server-owned, ordering is deterministic, and raw benchmark content is absent.
No evaluation, local SDK, or Web implementation gap was found by the bounded reviews.

本地 benchmark 收束现已在 contract boundary 取得新鲜证据。现有 evaluation domain 继续是唯一的
`BenchmarkSuite -> Scorecard -> RegressionDecision` policy path。Storage contract coverage 现包含 same-commit exact
decision comparison 与完整脱敏 workspace projection：project/Context/commit/cohort 的所有 scope 维度均 fail closed，
baseline/revised value 与 status 仍由服务端拥有，排序确定，且不包含 raw benchmark content。bounded review 未发现
evaluation、local SDK 或 Web implementation gap。

Observed receipts are evaluation `45 passed`, storage benchmark evidence `23 passed`, workspace
projection `7 passed`, execution `11 passed`, API `183 passed`, workspace Rust `179 passed, 39 ignored`,
format passed, strict offline workspace Clippy passed, locked Rust `1.85.0` check passed, and
`pnpm check:web` passed public SDK `15`, local SDK `92`, Web `185`, and the production build. Static
inspection observed one `impl GraphDiff`; public SDK/OpenAPI retirement assertions passed as part of
the repository Web check. The direct ad hoc Vitest command was unobserved because no direct binary is
provided; it is not substituted for the passed repository command.

已观测回执为 evaluation `45 passed`、storage benchmark evidence `23 passed`、workspace projection `7 passed`、
execution `11 passed`、API `183 passed`、workspace Rust `179 passed, 39 ignored`、format 通过、strict offline workspace
Clippy 通过、锁定 Rust `1.85.0` check 通过，以及 `pnpm check:web` 通过 public SDK `15`、local SDK `92`、Web `185` 并
完成 production build。静态 inspection 观测到一个 `impl GraphDiff`；public SDK/OpenAPI retirement assertion 已在
仓库 Web check 中通过。直接调用 Vitest 因没有 direct binary 而未观测，不以此替代已通过的仓库 command。

This advances Criteria 3 and 4 only; it does not close the repository or authorize public writes,
release, or production. PostgreSQL runtime, authenticated browser runtime, Git change-set, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`. No secret, Docker
runtime, provider call, migration, public route, public SDK method, Web mutation, or second graph-diff
calculator was added. The long-term goal remains active. The next candidate is private typed branch-head
discovery, admitted only after a new bilingual Necessity Record; branch merge/rollback remains out of
scope until its own merge-base and conflict contracts exist.

本增量只推进条件 3 与 4，不关闭仓库，也不授权 public write、release 或 production。PostgreSQL runtime、authenticated
browser runtime、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或
`deferred`。没有新增 secret、Docker runtime、provider call、migration、public route、public SDK method、Web mutation
或第二个 graph-diff calculator。长期目标保持 active。下一候选是 private typed branch-head discovery，只有新增双语
Necessity Record 后才准入；在独立 merge-base 与 conflict contract 具备前，branch merge/rollback 继续不在范围内。

## 2026-07-29 Private ContextGraph Three-Way Conflict Receipt / 2026-07-29 私有 ContextGraph 三路冲突回执

The private diff-engine contract now binds base, left, and right Context Graph snapshots to exact
`(ProjectId, ContextId, CommitId)` identities and accepts only a matching `MergePlan::ThreeWay`.
It reuses `GraphDiff::between` for both base-to-branch comparisons and classifies deterministic
Clean, Equivalent, or Conflict node/edge changes. This advances Criteria 2 and 4, but does not
close either criterion or authorize a merge/rollback writer.

private diff-engine contract 现已将 base、left、right Context Graph snapshot 绑定到 exact `(ProjectId, ContextId, CommitId)`
identity，并且只接受匹配的 `MergePlan::ThreeWay`。它复用 `GraphDiff::between` 完成两次 base-to-branch comparison，并
对 node/edge change 返回确定性的 Clean、Equivalent 或 Conflict。本回执推进条件 2 与 4，但不关闭任一条件，也不授权
merge/rollback writer。

Observed local receipt: diff-engine focused targets `6`, workspace Rust `186 passed, 39 ignored`, format, strict
offline workspace Clippy, locked Rust `1.85.0`, and `pnpm check:web` with public SDK `15`, local SDK `92`, Web `185`,
and production build passed. Static singularity inspection observed one `impl GraphDiff`. No persistence, public write,
transport, Web mutation, provider, Docker runtime, or second calculator was added. PostgreSQL/Docker runtime,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`;
the long-term goal remains active.

已观测本地回执：diff-engine focused targets `6`、workspace Rust `186 passed, 39 ignored`、format、strict offline workspace
Clippy、锁定 Rust `1.85.0`，以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185` 与 production build）均通过。
静态 singularity inspection 观测到一个 `impl GraphDiff`。没有新增 persistence、public write、transport、Web mutation、
provider、Docker runtime 或第二个 calculator。PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator
rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；长期目标保持 active。

## 2026-07-30 Private Workflow Binding Availability Closure / 2026-07-30 私有 Workflow Binding 可用性收束

The private Workflow Context-binding read now exposes the existing five-state `data -> presenter ->
screen` contract at the actual inspector boundary. A typed upstream `503` reaches `unavailable`,
while auth, authorization, rate-limit, protocol, and transport failures remain `error`. The success
regression renders the selected redacted binding through the shared screen, and the shared local SDK
request helper declares `cache: "no-store"` with request-scoped Bearer and cookie omission. This
advances Criteria 1, 5, 6, and 9 without adding a route, public API/OpenAPI/SDK method, mutation,
provider, secret, or second graph-diff calculator.

私有 Workflow Context binding read 现已在实际 inspector boundary 暴露既有五态 `data -> presenter -> screen` contract。typed
upstream `503` 到达 `unavailable`，auth、authorization、rate-limit、protocol 与 transport failure 继续保持 `error`。
成功 regression 会通过 shared screen 渲染选定的脱敏 binding，shared local SDK request helper 也会在 request-scoped Bearer
与 cookie omission 之外声明 `cache: "no-store"`。本增量推进条件 1、5、6 与 9，未新增 route、public API/OpenAPI/SDK method、
mutation、provider、secret 或第二个 graph-diff calculator。

Fresh local receipts: focused Web Workflow `11/11`; focused local SDK Workflow `7/7`; `pnpm
check:web` with public SDK `15`, local SDK `99`, Web `202`, TypeScript/lint, and production build;
workspace Rust with storage `193 passed, 39 ignored`; API Workflow binding `4 passed`; format;
strict offline Clippy; and locked Rust `1.85.0` check. Docker/PostgreSQL runtime, authenticated
browser, Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved`
or `deferred`. Criteria 1, 5, 6, and 9 remain open; the long-term goal remains active.

新鲜本地回执：focused Web Workflow `11/11`；focused local SDK Workflow `7/7`；`pnpm check:web`（public SDK `15`、local SDK `99`、
Web `202`、TypeScript/lint 与 production build）；workspace Rust（storage `193 passed, 39 ignored`）；API Workflow binding `4 passed`；
format；strict offline Clippy；以及锁定 Rust `1.85.0` check。Docker/PostgreSQL runtime、authenticated browser、Git change-set、
remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。条件 1、5、6 与 9 仍开放；长期目标保持 active。

## 2026-07-30 Private Versioned ContextGraph Merge Review (recorded after persisted review) / 2026-07-30 私有版本化 ContextGraph Merge Review（在持久化审阅之后记录）

The private diff-engine application boundary now carries a V1 schema, exact owned base/left/right
graph snapshots, matching `MergePlan`, and deterministic `GraphMergeClassification` projection.
It rejects nil or duplicate version identities and delegates only through
`GraphMergeConflictClassifier`; `GraphDiff::between` remains encapsulated in the existing
classifier. The storage review adapter converts its exact persisted reads into this request and
extracts the already-computed classification without adding a second algorithm. This advances
Criteria 2 and 4 without closing either or authorizing transport.

private diff-engine application boundary 现携带 V1 schema、exact owned base/left/right graph snapshot、匹配的 `MergePlan` 与
确定性的 `GraphMergeClassification` projection。它拒绝 nil 或 duplicate version identity，并且只通过
`GraphMergeConflictClassifier` delegation；`GraphDiff::between` 继续封装在既有 classifier 内。storage review adapter 将
exact persisted read 转换为该 request，并在不新增第二套 algorithm 的情况下提取已计算的 classification。本增量推进
条件 2 与 4，但不关闭任一条件，也不授权 transport。

Observed local receipts: focused diff-engine `4 passed`; focused storage `8 passed`; workspace
Rust `186 passed, 39 ignored`; format; strict offline workspace Clippy; locked Rust `1.85.0`; and
`pnpm check:web` with public SDK `15`, local SDK `92`, Web `188`, and production build. Static
inspection observed one `impl GraphDiff`. The repository adapter now reads the three exact snapshots
through one batch contract, using one Memory guard and one PostgreSQL read transaction. PostgreSQL/Docker runtime, browser,
Git, remote CI, operator, release, and production remain `unobserved` or `deferred`; no public
write, migration, Web mutation, secret, provider, or production claim was added. The long-term
goal remains active and the next increment requires a new bilingual Necessity Record.

已观测本地回执：focused diff-engine `4 passed`；focused storage `8 passed`；workspace Rust `186 passed, 39 ignored`；format；
strict offline workspace Clippy；锁定 Rust `1.85.0`；以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `188` 与
production build）。静态 inspection 观测到一个 `impl GraphDiff`。repository adapter 现通过一个 batch contract 读取三份
exact snapshot，Memory 使用一个 guard，PostgreSQL 使用一个 read transaction。PostgreSQL/Docker runtime、browser、Git、
remote CI、operator、release 与 production 仍为 `unobserved` 或 `deferred`；未新增 public write、migration、Web mutation、
secret、provider 或 production claim。长期目标保持 active，下一增量必须先新增双语 Necessity Record。

## 2026-07-30 Private Persisted ContextGraph Merge Review / 2026-07-30 私有持久化 ContextGraph Merge Review

The storage-owned review bridge now supplies the missing repository contract for a private
three-way Context Graph review. `ContextMergeInputScope` binds one project/Context and three
distinct commit identities. `PersistedContextGraphMergeReviewService` validates the plan before
reading, requests exact base/left/right persisted snapshot scopes, fails closed for missing or
scope-drifted records, and delegates to `GraphMergeConflictClassifier`. It does not merge, write,
mutate a branch, or calculate a second diff. This is fresh local evidence for Criteria 2 and 4,
but it does not close either criterion or authorize public transport.

storage-owned review bridge 现已补齐 private three-way Context Graph review 所缺的 repository contract。
`ContextMergeInputScope` 绑定一个 project/Context 与三个不同 commit identity。
`PersistedContextGraphMergeReviewService` 在 read 前校验 plan，请求 exact base/left/right persisted snapshot scope，
对 missing 或 scope drifted record fail closed，并委托 `GraphMergeConflictClassifier`。它不 merge、不 write、不修改
branch，也不计算第二个 diff。本回执是条件 2 与 4 的新鲜本地证据，但不关闭任一条件，也不授权 public transport。

Observed local receipts: focused storage `8 passed`; workspace Rust `186 passed, 39 ignored`;
`cargo fmt --all -- --check`; strict offline workspace Clippy; locked Rust `1.85.0` workspace
check; `pnpm check:web` with public SDK `15`, local SDK `92`, Web `185`, and production build; and
static singularity inspection with one `impl GraphDiff`. The repository bridge now uses the batch
contract exactly once; Memory reads under one guard and PostgreSQL reads under one
`REPEATABLE READ READ ONLY` transaction. This closes the cross-read consistency gap at the local
contract boundary without claiming PostgreSQL runtime evidence.
PostgreSQL runtime, Docker/virtualization, authenticated browser, Git, remote CI, operator
rehearsal, release, and production remain `unobserved` or `deferred`; no secret or external receipt
is implied.

已观测本地回执：focused storage `8 passed`；workspace Rust `186 passed, 39 ignored`；`cargo fmt --all -- --check`；strict
offline workspace Clippy；锁定 Rust `1.85.0` workspace check；`pnpm check:web`（public SDK `15`、local SDK `92`、Web `188`
与 production build）；以及 static singularity inspection 观测到一个 `impl GraphDiff`。repository bridge 现只调用一次
batch contract；Memory 在一个 guard 下读取，PostgreSQL 在一个 `REPEATABLE READ READ ONLY` transaction 下读取。该修复在
local contract boundary 收束 cross-read consistency，但不代表 PostgreSQL runtime evidence。PostgreSQL runtime、Docker/virtualization、authenticated browser、Git、
remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`；不暗示任何 secret 或 external receipt。

This slice preserves `GraphDiff::between` as the sole graph-diff calculator and adds no public
write, OpenAPI/SDK method, Web mutation, migration, provider, or production-readiness claim. The
long-term goal remains active. The next increment must begin with a new bilingual Necessity Record
for the next dependency-ready private Context editing, benchmark, or replay contract.

本切片继续保持 `GraphDiff::between` 为唯一 graph-diff calculator，未新增 public write、OpenAPI/SDK method、Web
mutation、migration、provider 或 production-readiness claim。长期目标保持 active。下一增量必须先为下一项依赖就绪的
private Context editing、benchmark 或 replay contract 新增双语 Necessity Record。

## 2026-07-28 Private Merge-Base and Ancestry Conflict Receipt / 2026-07-28 私有 Merge-Base 与 Ancestry Conflict 回执

The reusable versioning core now has a validated, single-Context commit DAG and deterministic
ancestry planning. `CommitGraph` rejects duplicate commit/parent identities, missing parents,
cross-Context edges, disconnected nodes from another Context, and cycles. `MergePlan` covers
no-op, fast-forward, one-base three-way, no-common-ancestor, and ambiguous maximal-base outcomes.
This advances the replayable version-history portion of Criteria 2, but does not close Criteria 2,
branch merge/rollback, or the repository.

可复用 versioning core 现已具备 validated single-Context commit DAG 与确定性的 ancestry planning。`CommitGraph` 会拒绝
duplicate commit/parent identity、missing parent、跨 Context edge、来自另一个 Context 的断开节点与 cycle。`MergePlan`
覆盖 no-op、fast-forward、single-base three-way、no-common-ancestor 与 ambiguous maximal-base outcome。本回执只推进
条件 2 的可回放版本历史部分，不关闭条件 2、branch merge/rollback 或 repository。

Observed local receipt: `cargo test -p contextlab-versioning --quiet` passed `31` tests; focused
strict offline Clippy and format passed before the broader workspace verification. `GraphDiff::between`
remains the sole graph-diff calculator. The contract is pure ancestry: it adds no storage, migration,
transport, public write, Web mutation, or content-level conflict resolution. PostgreSQL/Docker runtime,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`; the long-term goal remains active.

已观测本地回执：`cargo test -p contextlab-versioning --quiet` 通过 `31` 项测试；focused strict offline Clippy 与 format
已在更大 workspace verification 前通过。`GraphDiff::between` 仍是唯一 graph-diff calculator。本契约只处理 pure ancestry，
不新增 storage、migration、transport、public write、Web mutation 或 content-level conflict resolution。PostgreSQL/Docker
runtime、authenticated browser、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；
长期目标保持 active。

### 2026-07-29 Workflow Binding Interaction Receipt / 2026-07-29 Workflow Binding 交互回执

The private Workflow binding read has fresh Web interaction evidence. Its inspector lifecycle tests
execute the control handler, preserve exact Context/commit selection, assert request-memory Bearer
transport with cookie omission and no-store caching, project the redacted ready state, and map a
protected 403 to a bilingual typed error without rendering the upstream diagnostic. The current
package test command expands to the Web suite and passed `188`; `pnpm check:web` passed public SDK
`15`, local SDK `92`, Web `188`, and the local production build. Rust workspace tests passed (`183`
API and `186 passed, 39 ignored` storage), the focused API hardening test passed `4`, strict offline
Clippy, locked Rust `1.85.0` check, and `cargo fmt --all -- --check` all passed. The hardening
repairs are implemented and the complete local gate is full-green. This closes only the local interaction-evidence gap; Criteria
1, 2, 4, 5, 6, and 9 remain open. PostgreSQL runtime, authenticated browser, Git, and remote
evidence remain `unobserved`; operator, release, and production evidence remain `deferred`.

2026-07-29 私有 Workflow binding read 获得了新鲜 Web interaction evidence。inspector lifecycle test 会执行 control handler，
保持精确 Context/commit selection，断言 request-memory Bearer transport、cookie omission 与 no-store caching，投影脱敏
ready state，并将 protected 403 映射为双语 typed error，且不渲染上游 diagnostic。当前 package test command 展开为 Web
suite 并通过 `188` 项；`pnpm check:web` 通过 public SDK `15`、local SDK `92`、Web `188` 与本地 production build。Rust
workspace test 通过（API `183`、storage `186 passed, 39 ignored`），API hardening focused test 通过 `4` 项，strict offline
Clippy、锁定 Rust `1.85.0` check 与 `cargo fmt --all -- --check` 均通过。hardening repair 已实现并完成 full-green local
verification。本回执只收束 local interaction-evidence gap；条件 1、2、4、5、6
与 9 仍开放。PostgreSQL runtime、authenticated browser、Git 与 remote evidence 仍为 `unobserved`；operator、release 与
production evidence 仍为 `deferred`。

The private atomic three-snapshot read contract in
`docs/superpowers/plans/2026-07-29-private-atomic-context-graph-snapshot-read.md` is now locally
verified. Its scope remains storage consistency for the existing private merge review: Memory uses
one guard, PostgreSQL uses one read transaction, and the review service calls the batch port once.
The next admitted increment must begin with a new bilingual Necessity Record for a dependency-ready
private Context editing, benchmark, or replay contract; no merge writer, public transport, API/SDK
method, Web mutation, or production claim is admitted by this receipt.

`docs/superpowers/plans/2026-07-29-private-atomic-context-graph-snapshot-read.md` 中的私有 atomic three-snapshot read
contract 现已完成本地验证：Memory 使用一个 guard，PostgreSQL 使用一个 read transaction，review service 只调用一次
batch port。下一项准入增量必须先为依赖就绪的 private Context editing、benchmark 或 replay contract 新增双语 Necessity
Record；本回执不准入 merge writer、public transport、API/SDK method、Web mutation 或 production claim。

## 2026-07-29 Private Context Commit Replay State / 2026-07-29 私有 Context Commit 回放状态

The private versioning core now has a deterministic replay projection for an ordered normal-parent
Context history. `ReplayState` folds typed component creation, content-hash update, descriptor update,
removal, and `Uses` relationship changes into exact descriptor-only state; invalid transitions are
atomic failures, removal cleans incident relationships, and cross-Context/stale/merge parent shapes
fail closed. The contract exposes `REPLAY_STATE_SCHEMA_VERSION` and
`ReplayState::from_commits`, while preserving `GraphDiff::between` as the sole graph-diff calculator.
This advances Criteria 1 and 2 only; it does not add persistence, transport, public writes, Web
mutation, merge/rollback, or production readiness.

私有 versioning core 现具备有序 normal-parent Context history 的确定性 replay projection。`ReplayState` 将 typed
component creation、content-hash update、descriptor update、removal 与 `Uses` relationship change 折叠为 exact
descriptor-only state；无效 transition 以原子失败返回，removal 会清理 incident relationship，cross-Context、stale 与
merge parent shape 会 fail closed。该 contract 暴露 `REPLAY_STATE_SCHEMA_VERSION` 与 `ReplayState::from_commits`，并继续
保持 `GraphDiff::between` 为唯一 graph-diff calculator。本增量只推进条件 1 与 2；不新增 persistence、transport、public
write、Web mutation、merge/rollback 或 production readiness。

Fresh local evidence is versioning `37 passed`, package formatting passed, strict offline package Clippy passed, and Web
`189 passed` with TypeScript/lint and production build passed after the exact benchmark workspace lifecycle sidecar. Git,
PostgreSQL/Docker runtime, authenticated browser, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`; no secret was read. The long-term goal remains active and the next increment requires a new
bilingual Necessity Record.

新鲜本地证据为 versioning `37 passed`、package formatting、strict offline package Clippy，以及 Web `189 passed`；exact benchmark
workspace lifecycle sidecar 后 TypeScript/lint 与 production build 通过。Git、PostgreSQL/Docker runtime、authenticated browser、
remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未读取 secret。长期目标保持 active，
下一增量必须先新增双语 Necessity Record。

## 2026-07-29 Metadata Replay Receipt / 2026-07-29 Metadata Replay 回执

The private metadata replay slice is `completed` / `verified locally`. It adds a typed schema-versioned
`ContextMetadataPayload`, deterministic `UpdatedMetadata` projection, malformed/missing payload rejection,
atomic failure behavior, and an outer `ContextChangeWire` unknown-field rejection regression. Fresh
verification passed versioning `41` tests, workspace Rust (API `183`; storage `186 passed, 39 ignored`),
format, strict offline Clippy, locked Rust `1.85.0`, and `pnpm check:web` (public SDK `15`, local SDK `92`,
Web `189`, TypeScript/lint, production build). Static inspection found one `impl GraphDiff`.

私有 metadata replay 切片现为 `completed` / `verified locally`。它增加 typed schema-versioned `ContextMetadataPayload`、
确定性的 `UpdatedMetadata` projection、malformed/missing payload rejection、失败原子性，以及 outer `ContextChangeWire`
unknown-field rejection regression。新鲜验证通过 versioning `41` 项、workspace Rust（API `183`；storage `186 passed, 39 ignored`）、
format、strict offline Clippy、锁定 Rust `1.85.0` 与 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `189`、TypeScript/lint、
production build）。静态检查发现一个 `impl GraphDiff`。

This evidence advances Criteria 1 and 2 without closing them. ReplayState serialization, descriptor
stale-write preconditions, and component metadata content limits remain future Necessity Records.
PostgreSQL/Docker runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and
production are still `unobserved` or `deferred`; no public write or transport was added. The long-term
goal remains active. / 本证据推进条件 1 与 2 但不关闭它们。ReplayState serialization、descriptor stale-write precondition 与
component metadata content limit 仍需未来 Necessity Record。PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未新增 public write 或 transport。长期目标保持 active。

### 2026-07-29 Private Storage ReplayState Adapter Receipt / 2026-07-29 私有 Storage ReplayState Adapter 回执

The private storage replay-state repository is locally verified. It preserves persisted commit
identity, reads only the exact Context/commit scope, follows the normal-parent chain, rejects
missing or malformed records, and delegates all state transitions to `ReplayState::from_commits`.
The Memory parity test passed `1`; PostgreSQL SQL/row contract tests passed `4`; the storage package
passed `193` library tests with `39 ignored` and all auxiliary targets passed; and
`cargo fmt --all -- --check` passed. This evidence advances Criteria 1 and 2 but does not close
them or authorize a public write/read surface. Docker/PostgreSQL runtime, authenticated browser,
Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

私有 storage replay-state repository 已完成本地验证。它保留持久化 commit identity，只读取精确 Context/commit scope，沿
normal-parent chain 读取，拒绝 missing 或 malformed record，并将所有 state transition 委托给 `ReplayState::from_commits`。
Memory parity test 通过 `1` 项；PostgreSQL SQL/row contract test 通过 `4` 项；storage package 通过 `193` 个 library test、
`39` 个 ignored，所有 auxiliary target 均通过；`cargo fmt --all -- --check` 通过。本证据推进条件 1 与 2，但不关闭它们，
也不授权 public write/read surface。Docker/PostgreSQL runtime、authenticated browser、Git、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`。

## 2026-08-02 Criterion 3 Local PostgreSQL Breadth Receipt / 2026-08-02 条件 3 本地 PostgreSQL 宽度回执

Necessity / 必要性：This receipt supplies fresh local evidence for Criterion 3, Benchmark-driven
evaluation. The earlier port `55439` test represents only direct `BenchmarkDecisionEvidence`
decision/run persistence. The corrected port `55441` test-only PostgreSQL breadth case is needed to
round-trip two datasets and four cases through the existing sealed benchmark workspace projection,
retaining exact baseline/revised commit scopes, dataset-case provenance, scorecard threshold and
coverage, evaluation-diff evidence, replay/readback, exact-scope rejection, and redacted payload
boundaries. It is evidence for the criterion, not closure of the criterion.

必要性：本回执为条件 3（Benchmark-driven evaluation）补充新鲜本地证据。早期 port `55439` test 只代表直接
`BenchmarkDecisionEvidence` decision/run persistence。修正后的 port `55441` test-only PostgreSQL breadth case
用于通过既有 sealed benchmark workspace projection 往返保存 two datasets 与 four cases，并保留 exact
baseline/revised commit scope、dataset-case provenance、scorecard threshold 与 coverage、evaluation-diff evidence、
replay/readback、exact-scope rejection 与 redacted payload boundary。本回执是条件证据，不是条件收束。

Fresh evidence / 新鲜证据：

- Focused breadth test without a configured URL: `1 ignored` by design.
- Fresh loopback PostgreSQL 16 cluster on port `55439`: early direct decision/run persistence `1 passed`; it is
  not the full projection/provenance/replay/scope/redaction receipt.
- Separate fresh loopback PostgreSQL 16 cluster on port `55441`: corrected test-only projection breadth `1 passed`,
  carrying the full projection/provenance/replay/scope/redaction receipt; both temporary clusters were stopped.
- Workspace Rust: `220 passed`, `40 ignored`; format, strict offline Clippy, and locked Rust `1.85.0` checks passed.
- Web: `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `284/284`, and production build.
- `tests/contract/verify-local-contracts.test.ps1` and `scripts/verify-local-contracts.ps1` passed.
- Temporary-directory cleanup remains `unobserved`; existing port `5432` was not inspected or used.

新鲜证据：

- 未配置 URL 的 focused breadth test：按设计为 `1 ignored`。
- port `55439` 的 fresh loopback PostgreSQL 16 cluster：早期 direct decision/run persistence `1 passed`；不属于完整
  projection/provenance/replay/scope/redaction receipt。
- 独立的 port `55441` fresh loopback PostgreSQL 16 cluster：修正后的 test-only projection breadth `1 passed`，承载完整
  projection/provenance/replay/scope/redaction receipt；两个临时 cluster 均已停止。
- Workspace Rust：`220 passed`、`40 ignored`；format、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- Web：`pnpm check:web` 通过，public SDK `15`、local SDK `135`、Web `284/284` 与 production build 均通过。
- `tests/contract/verify-local-contracts.test.ps1` 与 `scripts/verify-local-contracts.ps1` 通过。
- 临时目录 cleanup 继续为 `unobserved`；未检查或使用既有 port `5432`。

No public REST/OpenAPI/SDK write, Web mutation, provider, migration, secret, Docker, browser,
Git, remote CI, operator, release, or production evidence was added or claimed. Those boundaries,
along with filesystem cleanup, remain `unobserved` or `deferred`. Criteria 1-9 and the long-term
goal remain open until every criterion has fresh scope-matched evidence.

未新增或宣称 public REST/OpenAPI/SDK write、Web mutation、provider、migration、secret、Docker、browser、
Git、remote CI、operator、release 或 production evidence。上述边界及 filesystem cleanup 继续为 `unobserved`
或 `deferred`。条件 1-9 与长期目标仍保持 open，只有全部条件取得新鲜且范围匹配的证据后才可收束。

## 2026-08-02 Context Lifecycle PostgreSQL Runtime Audit / 2026-08-02 Context 生命周期 PostgreSQL 运行时审计

Criteria 1 and 2 receive fresh local runtime evidence but remain open. The existing PostgreSQL
Context lifecycle tests ran against separate fresh loopback PostgreSQL 16 clusters. The initialization
and replay test first exposed a stale fixture expectation: the seeded workspace contains six existing
components, so the observed count was `(1, 0, 1, 1, 1, 6, 0)`, not the old `(1, 0, 1, 1, 1, 0, 0)`. A
test-only expectation correction was made and the fresh rerun passed `1`; the typed `Uses` relationship
add/remove and exact-commit read/replay test passed `1` on another fresh cluster.

条件 1 与 2 获得新鲜 local runtime evidence，但仍保持开放。既有 PostgreSQL Context lifecycle test 分别在独立
fresh loopback PostgreSQL 16 cluster 上运行。初始化与 replay test 首次暴露陈旧 fixture expectation：seeded workspace
包含六个既有 component，因此实际 count 为 `(1, 0, 1, 1, 1, 6, 0)`，而不是旧值 `(1, 0, 1, 1, 1, 0, 0)`。完成
test-only expectation correction 后 fresh rerun 为 `1 passed`；typed `Uses` relationship add/remove 与 exact-commit
read/replay test 在另一个 fresh cluster 上为 `1 passed`。

Both temporary clusters were stopped. The pre-existing local PostgreSQL listener on port 5432 was
not used or inspected for application data; temporary-directory cleanup remains `unobserved` due
local tool policy. This is local non-production evidence only. Browser, Git, remote CI, operator
rehearsal, release, and production evidence remain `unobserved` or `deferred`; the long-term goal
remains `active`.

两个临时 cluster 均已停止。既有本地 5432 PostgreSQL listener 未被使用，也未读取其 application data；由于本地工具策略，
临时目录 cleanup 继续为 `unobserved`。本证据仅限本地非生产。Browser、Git、remote CI、operator rehearsal、release 与 production
evidence 继续为 `unobserved` 或 `deferred`；长期目标保持 `active`。

## 2026-08-01 Benchmark Persistence Runtime Audit / 2026-08-01 Benchmark 持久化运行时审计

Criterion 3 receives a fresh local evidence receipt but remains open. The existing private
Benchmark workspace projection PostgreSQL writer/reader was exercised against two separate fresh
loopback PostgreSQL 16 clusters: `postgres_benchmark_workspace_projection_creates_replays_and_reads_exact_scope`
passed `1`, and `postgres_benchmark_execution_materializes_and_replays_workspace_projection` passed `1`.
The focused projection contract passed `7`, and the final Web gate passed with public SDK `15`, local
SDK `135`, Web `284/284`, and a production build. This proves a local persistence boundary, not the
whole benchmark criterion, public write readiness, or production readiness.

条件 3 获得新鲜本地 evidence receipt，但仍保持开放。既有 private Benchmark workspace projection PostgreSQL
writer/reader 在两个独立 fresh loopback PostgreSQL 16 cluster 上运行：
`postgres_benchmark_workspace_projection_creates_replays_and_reads_exact_scope` 为 `1 passed`，
`postgres_benchmark_execution_materializes_and_replays_workspace_projection` 为 `1 passed`。focused projection
contract 为 `7 passed`，最终 Web gate 为 public SDK `15`、local SDK `135`、Web `284/284` 与 production build
通过。这只证明本地 persistence boundary，不证明完整 benchmark criterion、public write readiness 或 production readiness。

The two ignored tests must not share one database: the combined invocation failed at the test
harness boundary with `relation "workspaces" already exists` after the first migration. Separate
fresh databases are now the required reproducible invocation. Temporary processes were stopped;
recursive data-directory cleanup remains `unobserved` due local tool policy. PostgreSQL runtime is
local and non-production; browser, Git, remote CI, operator rehearsal, release, and production
evidence remain `unobserved` or `deferred`. The long-term goal remains `active`.

两个 ignored test 不得共用一个 database：组合运行在第一项 migration 后于 test harness 边界因
`relation "workspaces" already exists` 失败。现在要求使用独立 fresh database 进行可重复运行。临时 process 已
停止；由于本地工具策略，data-directory cleanup 继续为 `unobserved`。PostgreSQL runtime 证据仅限本地非生产；
browser、Git、remote CI、operator rehearsal、release 与 production evidence 继续为 `unobserved` 或 `deferred`。
长期目标保持 `active`。

The broader fresh local gate also passed workspace Rust tests (API `183`, versioning `45`, storage
`193 passed, 39 ignored`, all other targets passed), strict offline workspace Clippy,
`cargo +1.85.0 check --workspace --all-targets --locked`, and `pnpm check:web` (public SDK `15`,
local SDK `92`, Web `189`, TypeScript/lint, and local production build). Static inspection observed
one `impl GraphDiff`; this does not turn local evidence into PostgreSQL runtime, release, or
production evidence.

更大范围的新鲜本地 gate 还通过了 workspace Rust tests（API `183`、versioning `45`、storage `193 passed, 39 ignored`，其余
target 均通过）、strict offline workspace Clippy、`cargo +1.85.0 check --workspace --all-targets --locked` 与
`pnpm check:web`（public SDK `15`、local SDK `92`、Web `189`、TypeScript/lint 与本地 production build）。静态检查观测到唯一
一个 `impl GraphDiff`；这些本地证据不等同于 PostgreSQL runtime、release 或 production evidence。

The next admissible work still requires a new bilingual Necessity Record. ReplayState
serialization, descriptor stale-write preconditions, component metadata content policy, transport,
and public mutation remain unverified or out of scope; the long-term goal remains active.

下一项可准入工作仍必须新增双语 Necessity Record。ReplayState serialization、descriptor stale-write precondition、component
metadata content policy、transport 与 public mutation 仍未验证或不在范围内；长期目标保持 active。

### 2026-07-29 Private ReplayState Serialization Envelope Receipt / 2026-07-29 私有 ReplayState 序列化信封回执

The reusable versioning core now exposes `ReplayStateSnapshotV1` as a stable local serialization
boundary. Canonical JSON round-trip preserves the exact replay state, while schema drift, unknown
outer/nested fields, duplicate components or relationships, nil IDs, and missing relationship
endpoints fail closed. Fresh evidence is serialization integration `8 passed`, versioning `42` unit
plus `8` integration tests passed, workspace Rust (API `183`; storage `193 passed, 39 ignored`; all
other targets passed), format, strict offline Clippy, locked Rust `1.85.0`, and `pnpm check:web`
(public SDK `15`, local SDK `92`, Web `189`, TypeScript/lint, production build). This advances
Criteria 1 and 2 but does not close them or authorize storage mutation, public transport, or
production readiness.

可复用 versioning core 现暴露 `ReplayStateSnapshotV1` 作为稳定的本地 serialization boundary。canonical JSON round-trip 会
保留 exact replay state；schema drift、outer/nested unknown field、重复 component 或 relationship、nil ID 与缺失 relationship
endpoint 均会 fail closed。新鲜证据为 serialization integration `8 passed`、versioning `42` 个 unit 加 `8` 个 integration test
通过、workspace Rust（API `183`；storage `193 passed, 39 ignored`；其余 target 均通过）、format、strict offline Clippy、锁定
Rust `1.85.0` 与 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `189`、TypeScript/lint、production build）。本证据推进
条件 1 与 2，但不关闭它们，也不授权 storage mutation、public transport 或 production readiness。

No storage/API/SDK/Web/migration surface changed, `GraphDiff::between` remains the sole graph-diff
calculator, and PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. The long-term goal remains active.

未改变 storage/API/SDK/Web/migration surface，`GraphDiff::between` 仍是唯一 graph-diff calculator；PostgreSQL runtime、
authenticated browser、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active。

The next admissible increment requires a new bilingual Necessity Record for a dependency-ready
Context editing, benchmark evidence, or replay consumer boundary.

下一项可准入增量必须先为依赖就绪的 Context editing、benchmark evidence 或 replay consumer boundary 新增双语 Necessity Record。

### 2026-07-29 Private Branch-Head Discovery Receipt / 2026-07-29 私有 Branch-Head 发现回执

The local version-history coverage now includes a typed, private branch-head discovery path.
`ContextBranchRepository` remains the Rust source of truth; the protected API exposes only the
exact Context-scoped redacted projection, the non-public SDK validates the V1 envelope, and the
same-origin Web BFF and shared inspector preserve request-memory Bearer, cookie omission,
`credentials: "omit"`, `private, no-store`, and bilingual loading/error/empty/ready/unavailable
states. Web parser drift found during review was fixed by reusing the strict SDK parser; it now
rejects canonical UUID, legal branch-name, duplicate, unsorted, and unknown-field violations.

当前版本历史 coverage 已包含 typed、private branch-head discovery path。`ContextBranchRepository` 仍是 Rust source of
truth；protected API 只暴露 exact Context-scoped 脱敏 projection，非公开 SDK 校验 V1 envelope，同源 Web BFF 与 shared
inspector 保持 request-memory Bearer、cookie omission、`credentials: "omit"`、`private, no-store` 以及双语
loading/error/empty/ready/unavailable state。复核中发现的 Web parser drift 已通过复用 strict SDK parser 修复；现在会拒绝
canonical UUID、合法 branch-name、duplicate、unsorted 与 unknown-field violation。

Fresh local evidence is storage contract `7 passed`, API handler `6 passed`, protected router `2 passed`, local SDK
focused `7 passed` and full `99 passed`, BFF `3 passed`, Web branch-head `5 passed`, workspace Rust pass with storage
`193 passed, 39 ignored`, `cargo fmt --all -- --check`, strict offline workspace Clippy, locked offline check, and
`pnpm check:web` with public SDK `15`, local SDK `99`, Web `197`, TypeScript/lint, and production build. Static inspection
observed exactly one `impl GraphDiff`. This advances Criteria 1, 2, and 4 but does not close them or the repository.

新鲜本地证据为 storage contract `7 passed`、API handler `6 passed`、protected router `2 passed`、local SDK focused `7 passed` 与全包
`99 passed`、BFF `3 passed`、Web branch-head `5 passed`、workspace Rust 通过且 storage `193 passed, 39 ignored`、
`cargo fmt --all -- --check`、strict offline workspace Clippy、locked offline check，以及 `pnpm check:web`（public SDK `15`、
local SDK `99`、Web `197`、TypeScript/lint 与 production build）。静态检查观测到唯一一个 `impl GraphDiff`。本增量推进条件
1、2 与 4，但不关闭这些条件或仓库。

No branch creation, merge/rollback, public REST/OpenAPI/public SDK write, migration, operator
transport, provider call, secret access, or second graph-diff calculator was added. PostgreSQL/
Docker runtime, authenticated browser, and Git change-set remain `unobserved`; remote CI, operator
rehearsal, release, and production remain `deferred`. Criterion 1/2/4 and the long-term goal remain
open; the next increment requires a new bilingual Necessity Record.

没有新增 branch creation、merge/rollback、public REST/OpenAPI/public SDK write、migration、operator transport、provider call、
secret access 或第二个 graph-diff calculator。PostgreSQL/Docker runtime、authenticated browser 与 Git change-set 仍为
`unobserved`；remote CI、operator rehearsal、release 与 production 仍为 `deferred`。条件 1/2/4 与长期目标仍开放；下一项增量
必须先新增双语 Necessity Record。

## 2026-07-29 Private Branch-Head Graph Review Selection Receipt / 2026-07-29 私有 Branch-Head 图谱审阅选择回执

The private branch-head to graph-review composition is implemented and locally verified. The
existing branch-head inspector emits the exact server-owned non-null `head_commit_id`; the Web
composition adds it as the revised graph-review candidate without duplicates and leaves the
existing defaults unchanged for an unborn/null head. Its keyed review boundary resets the existing
review state when the Context or selected head changes. This is a local read-only composition over
the existing strict parser, protected graph-diff read, and `data -> presenter -> screen` boundary;
it does not calculate graph differences in Web.

私有 branch-head 到 graph-review 的 composition 已实现并完成本地验证。既有 branch-head inspector 发出 exact server-owned
非 null `head_commit_id`；Web composition 将其作为 revised graph-review candidate 加入且不重复，unborn/null head 则保持
既有 defaults。带 key 的 review boundary 会在 Context 或 selected head 变化时重置既有 review state。本增量是基于既有
strict parser、protected graph-diff read 与 `data -> presenter -> screen` boundary 的 local read-only composition；Web 不计算
graph difference。

Fresh evidence: focused branch-head/graph-review tests `8 passed`; `pnpm check:web` public SDK
`15`, local SDK `99`, Web `200`, TypeScript/lint, and production build; `cargo fmt --all --
--check`; `cargo test --workspace --quiet --no-fail-fast` with storage `193 passed, 39 ignored`;
strict offline Clippy; locked Rust `1.85.0` check; static `impl GraphDiff` count `1`; public SDK
branch-head hits `0`; retired public commit-graph-diff read hits `0`.

新鲜证据：focused branch-head/graph-review test `8 passed`；`pnpm check:web` public SDK `15`、local SDK `99`、Web `200`、
TypeScript/lint 与 production build；`cargo fmt --all -- --check`；`cargo test --workspace --quiet --no-fail-fast`（storage
`193 passed, 39 ignored`）；strict offline Clippy；锁定 Rust `1.85.0` check；static `impl GraphDiff` count `1`；public SDK
branch-head hit `0`；retired public commit-graph-diff read hit `0`。

No public REST/OpenAPI/public SDK method, branch mutation, merge/rollback, Web mutation, provider,
migration, secret, Docker/PostgreSQL runtime, authenticated browser, Git change-set, remote CI,
operator rehearsal, release, or production evidence was added. This advances Criteria 1, 2, and 4
without closing them; the long-term goal remains active and the next increment requires a new
bilingual Necessity Record. PostgreSQL/Docker runtime, browser, and Git remain `unobserved`; remote
CI, operator, release, and production remain `deferred`.

没有新增 public REST/OpenAPI/public SDK method、branch mutation、merge/rollback、Web mutation、provider、migration、secret、
Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 或 production
evidence。本增量推进条件 1、2 与 4 但不关闭它们；长期目标保持 active，下一项增量必须先新增双语 Necessity Record。
PostgreSQL/Docker runtime、browser 与 Git 继续为 `unobserved`；remote CI、operator、release 与 production 继续为 `deferred`。

## 2026-07-29 Private Capability Availability Schema Hardening Receipt / 2026-07-29 私有 Capability Availability Schema 硬化回执

The numeric V1 local capability-availability parser now fails closed on unknown outer fields and
unknown nested bilingual fields. The red test first reproduced `Missing expected exception`; the
focused green test passed `3`. The existing frozen DTO, bilingual capability-state presenter and
screen, and local transport contract remain unchanged.

numeric V1 local capability-availability parser 现会对 unknown outer field 与 unknown nested bilingual field fail closed。红测
先以 `Missing expected exception` 复现缺口；focused green test 通过 `3`。既有 frozen DTO、双语 capability-state presenter 与
screen，以及 local transport contract 均保持不变。

Fresh local evidence also passed `pnpm check:web` with public SDK `15`, local SDK `99`, Web `201`,
TypeScript/lint, and production build; Rust format; workspace tests with storage `193 passed, 39
ignored`; strict offline Clippy; locked Rust `1.85.0` check; static `impl GraphDiff` count `1`;
public SDK branch-head hits `0`; and retired public commit-graph-diff read hits `0`.

新鲜本地证据还通过 `pnpm check:web`（public SDK `15`、local SDK `99`、Web `201`、TypeScript/lint 与 production build）、Rust
format、workspace test（storage `193 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0` check、static
`impl GraphDiff` count `1`、public SDK branch-head hit `0` 与 retired public commit-graph-diff read hit `0`。

This advances the fail-closed contract portion of the local platform criteria without closing the
repository. No API/OpenAPI/SDK method, mutation, provider, migration, secret, Docker/PostgreSQL
runtime, browser, Git, remote CI, operator, release, or production boundary changed. The
long-term goal remains active and the next increment still requires a new bilingual Necessity
Record; runtime/release evidence remains `unobserved/deferred`.

本增量推进本地平台条件中的 fail-closed contract 部分，但不收束仓库。没有改变 API/OpenAPI/SDK method、mutation、provider、
migration、secret、Docker/PostgreSQL runtime、browser、Git、remote CI、operator、release 或 production boundary。长期目标
保持 active，下一项增量仍必须先新增双语 Necessity Record；runtime/release evidence 继续为 `unobserved/deferred`。

## 2026-07-30 Private Replay/ContextGraph Consistency / 2026-07-30 私有 Replay/ContextGraph 一致性

### Criterion mapping / 条件映射

This receipt advances Criteria 1 and 2: exact Context commit history is replayable and reviewable,
and the reusable Rust storage/versioning core now checks one authoritative graph representation
before returning a complete local lifecycle state. It is progress evidence, not repository closeout.

本回执推进条件 1 与条件 2：exact Context commit history 可回放、可审阅，可复用的 Rust storage/versioning core 在返回
完整本地 lifecycle state 前会校验唯一权威 graph representation。它是进展证据，不是仓库收束。

### Implemented contract / 已实现 contract

crates/storage/src/replay_graph_consistency.rs provides a storage-private fail-closed validator.
ContextLifecycleService::read_state_at_commit invokes it after existing component witness checks.
server/api/src/lib.rs forwards the new replay port through the internal WorkspaceDataRepository.
crates/versioning/src/lib.rs exports the existing replay schema constant without adding a transport
contract. Focused tests cover matching state, scope drift, component node drift, missing Uses,
duplicate edges, and unexpected graph elements.

crates/storage/src/replay_graph_consistency.rs 提供 storage-private fail-closed validator。ContextLifecycleService::read_state_at_commit
在既有 component witness 校验后调用它。server/api/src/lib.rs 通过内部 WorkspaceDataRepository 转发新 replay port。
crates/versioning/src/lib.rs 导出既有 replay schema constant，但没有新增 transport contract。focused tests 覆盖匹配状态、
scope drift、component node drift、缺失 Uses、重复 edge 与多余 graph element。

### Fresh evidence / 新鲜证据

Consistency focused 6 passed; lifecycle 9 passed; storage 199 passed, 39 ignored; API
189 passed; workspace Rust passed; cargo fmt --all -- --check; strict offline Clippy; locked
Rust 1.85.0 check; pnpm check:web public SDK 15, local SDK 99, Web 207, TypeScript/lint,
and production build; static GRAPH_DIFF_IMPL_COUNT=1.

一致性 focused 6 passed、lifecycle 9 passed、storage 199 passed, 39 ignored、API 189 passed、workspace Rust、
cargo fmt --all -- --check、strict offline Clippy、锁定 Rust 1.85.0 check，以及 pnpm check:web（public SDK 15、
local SDK 99、Web 207、TypeScript/lint 与 production build）均通过；static GRAPH_DIFF_IMPL_COUNT=1。

### Evidence boundary and next gate / 证据边界与下一门禁

No public REST/OpenAPI/SDK write, Web mutation, operator transport, migration, provider, secret,
Docker/PostgreSQL runtime, authenticated browser, Git, remote CI, operator, release, or production
claim is made. PostgreSQL/Docker runtime, authenticated browser, and Git remain unobserved;
remote CI, operator rehearsal, release, and production remain deferred. The long-term goal stays
active; the next increment needs a new bilingual Necessity Record.

不宣称 public REST/OpenAPI/SDK write、Web mutation、operator transport、migration、provider、secret、Docker/PostgreSQL runtime、
authenticated browser、Git、remote CI、operator、release 或 production 已完成。PostgreSQL/Docker runtime、authenticated browser 与
Git 继续为 unobserved；remote CI、operator rehearsal、release 与 production 继续为 deferred。长期目标保持 active；下一增量
必须新增双语 Necessity Record。

## 2026-07-30 Private Versioned ContextGraph Diff and Replay Consumer Evidence / 2026-07-30 私有版本化 ContextGraph Diff 与回放消费者证据

This receipt advances Criteria 1, 2, and 4. A reusable V1 two-way ContextGraph review now
validates exact version scopes in `contextlab-diff-engine`; the storage adapter reads two
immutable `CommitGraphSnapshot` records and preserves their metadata; the protected API route
delegates comparison without changing its transport contract. The only graph comparison call
path remains the existing `GraphDiff::between` implementation. Separately, CLI and Desktop
consume the existing `ReplayStateSnapshotV1` through a typed, canonical, fail-closed read-only
projection, proving local replay consumers do not reimplement versioning logic.

本回执推进条件 1、2 与 4。可复用的 V1 two-way ContextGraph review 现由 `contextlab-diff-engine` 校验 exact version scope；
storage adapter 读取两份 immutable `CommitGraphSnapshot` 并保留 metadata；protected API route 在不改变 transport contract
的前提下委托 comparison。唯一 graph comparison call path 仍是既有 `GraphDiff::between` implementation。另一方面，CLI 与
Desktop 通过 typed、canonical、fail-closed 的 read-only projection 消费既有 `ReplayStateSnapshotV1`，证明 local replay
consumer 不会复制 versioning logic。

Observed local receipts: diff-engine `3 passed`, storage `3 passed`, protected API `6 passed`,
adapter `8 passed`, CLI `5 passed`, Desktop `5 passed`; workspace Rust passed with storage
`202 passed, 39 ignored` and API `189`; format, full strict offline Clippy, locked Rust `1.85.0`,
and `pnpm check:web` passed with public SDK `15`, local SDK `99`, Web `207`, TypeScript/lint,
and production build. Static `GRAPH_DIFF_IMPL_COUNT=1`.

已观测本地回执：diff-engine `3 passed`、storage `3 passed`、protected API `6 passed`、adapter `8 passed`、CLI `5 passed`、
Desktop `5 passed`；workspace Rust 通过，其中 storage `202 passed, 39 ignored`、API `189`；format、完整 strict offline Clippy、
锁定 Rust `1.85.0` 与 `pnpm check:web` 通过（public SDK `15`、local SDK `99`、Web `207`、TypeScript/lint 与 production build）。
静态 `GRAPH_DIFF_IMPL_COUNT=1`。

No public REST/OpenAPI/SDK write, Web mutation, provider call, migration, operator transport,
secret, or external deployment claim is made. PostgreSQL/Docker runtime, Tauri runtime,
authenticated browser, and Git remain `unobserved`; remote CI, operator rehearsal, release, and
production remain `deferred`. These are local progress receipts, not closure of any criterion or
the long-term goal.

没有新增 public REST/OpenAPI/SDK write、Web mutation、provider call、migration、operator transport、secret 或 external deployment
声明。PostgreSQL/Docker runtime、Tauri runtime、authenticated browser 与 Git 继续为 `unobserved`；remote CI、operator rehearsal、
release 与 production 继续为 `deferred`。这些是本地进展回执，不关闭任何条件或长期目标。

## 2026-07-30 Private Benchmark Decision Workspace and Workflow Replay Provenance / 2026-07-30 私有 Benchmark Decision Workspace 与 Workflow Replay Provenance

This receipt advances Criteria 1, 2, 3, and 4 without closing any of them. The private benchmark
workspace reader now has an exact decision resolver over `(project, Context, commit, decision)`;
Memory and PostgreSQL retain the decision-to-cohort mapping with the immutable projection source.
The protected decision workspace route, local SDK, BFF, and Web adapter consume that resolver and
the existing sealed projection. Partial baseline pairs, unknown query fields, unresolved decisions,
scope drift, and stored projection mismatches fail closed. Existing cohort-keyed reads remain
compatible. No client-side scorecard, regression, or evaluation diff logic was added.

本回执推进条件 1、2、3 与 4，但不关闭任何条件。private benchmark workspace reader 现对 `(project, Context, commit, decision)` 提供
exact decision resolver；Memory 与 PostgreSQL 将 decision-to-cohort mapping 与 immutable projection source 一起保留。protected decision
workspace route、local SDK、BFF 与 Web adapter 消费该 resolver 与既有 sealed projection。partial baseline pair、unknown query field、
unresolved decision、scope drift 与 stored projection mismatch 均 fail closed。既有 cohort-keyed read 保持兼容，没有新增 client-side scorecard、
regression 或 evaluation diff logic。

Workflow replay now persists `WorkflowCapabilitySnapshotV1` and its canonical digest in
`WorkflowExecutionLogV1`. Restore and replay validate schema, ordering, digest, capability
identity/version, and exact source provenance. This remains provider-free and adds no registry
mutation or transport.

Workflow replay 现会在 `WorkflowExecutionLogV1` 中持久化 `WorkflowCapabilitySnapshotV1` 与其 canonical digest。restore 与 replay 校验 schema、
ordering、digest、capability identity/version 与 exact source provenance。该能力保持 provider-free，没有新增 registry mutation 或 transport。

Fresh verification / 新鲜验证: resolver `2` focused tests; protected decision workspace API `10`
focused tests; storage `204 passed, 39 ignored`; API `191 passed`; workflow replay `15 passed`;
workspace Rust; `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0` check;
local SDK `100 passed`; and `pnpm check:web` public SDK `15`, local SDK `100`, Web `207`,
TypeScript/lint, production build. Static `GRAPH_DIFF_IMPL_COUNT=1`.

新鲜验证：resolver focused test `2` 项、protected decision workspace API focused test `10` 项、storage `204 passed, 39 ignored`、API `191 passed`、
workflow replay `15 passed`、workspace Rust、`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、local SDK `100 passed` 与
`pnpm check:web`（public SDK `15`、local SDK `100`、Web `207`、TypeScript/lint、production build）均通过。静态 `GRAPH_DIFF_IMPL_COUNT=1`。

At the start of this increment, the local Web still exposed the exact decision input as a private
read control. The decision-list selection binding below now removes that manual-ID path while
preserving the cohort compatibility read. PostgreSQL/Docker runtime, authenticated browser, and Git
remain `unobserved`; remote CI, operator rehearsal, release, and production remain `deferred`. No
public write, OpenAPI/public SDK write, Web mutation, provider, migration, secret, or production claim
is made.

本增量开始时，local Web 仍以 private read control 暴露 exact decision input；下方的 decision-list selection binding 现已移除该手填 ID 路径，
同时保留 cohort compatibility read。PostgreSQL/Docker runtime、authenticated browser 与 Git 继续为 `unobserved`；remote CI、operator rehearsal、release 与
production 继续为 `deferred`。没有新增 public write、OpenAPI/public SDK write、Web mutation、provider、migration、secret，也不作 production 声明。

## 2026-07-30 Private Benchmark Decision-List Selection Binding Review / 2026-07-30 私有 Benchmark Decision 列表选择绑定审阅

### Status and criterion mapping / 状态与条件映射

`completed / verified locally` / `completed / 本地已验证`. This review records a local-only,
private, read-only Web increment for Criterion 3 usability; it advances the criterion but does not
close it. The local SDK selection resource/loader and Web discovery/select wiring are present, and
`pnpm check:web` is green with public SDK `15`, local SDK `104`, Web `208`, TypeScript/lint, and
production build.

`completed / 本地已验证`。本审阅记录一个面向条件 3 可用性的 local-only、private、read-only Web 增量；它推进该条件但不关闭
该条件。local SDK selection resource/loader 与 Web discovery/select wiring 已存在，`pnpm check:web` 已通过，public SDK `15`、
local SDK `104`、Web `208`、TypeScript/lint 与 production build 均完成。

### Completed prerequisites / 已完成前置

The local SDK now owns the frozen listed-decision selection resource, exact scope/membership
checks, and selected decision-keyed loader. Web loads revised/baseline discovery lists, clears
selections on commit changes, and renders select options; the cohort-keyed compatibility read
remains present. Current audit processes observed local SDK `104 passed`, workspace selection
focused `16 passed`, discovery data/presenter `8 passed`, and `pnpm check:web` with Web `208`.

local SDK 现拥有 frozen listed-decision selection resource、精确 scope/membership check 与 selected decision-keyed loader。
Web 会加载 revised/baseline discovery list，在 commit 变化时清空 selection 并渲染 select option；cohort-keyed compatibility read
继续存在。当前审计进程观测到 local SDK `104 passed`、workspace selection focused `16 passed`、discovery data/presenter `8 passed`，
以及 Web `208`。

### Uncompleted work / 未完成工作

- No local implementation item remains for this increment. PostgreSQL/Docker, browser, visual,
  remote, Git, release, and production evidence remain outside this receipt.

- 本增量没有剩余的本地实现项。PostgreSQL/Docker、browser、visual、remote、Git、release 与 production evidence
  仍不在本回执范围内；Rust 质量门禁已在下方新鲜通过。

### Verification and provenance / 验证与 provenance

| Command / 命令 | Status / 状态 | Evidence boundary / 证据边界 |
| --- | --- | --- |
| `pnpm --filter @contextlab/local-sdk test` | `passed: 104` | Includes selection resource, exact scope/membership, state, and selected-loader tests. / 包含 selection resource、精确 scope/membership、state 与 selected-loader tests。 |
| `pnpm --filter @contextlab/web exec tsx --test src/app/local-benchmark-workspace-inspector.test.tsx src/app/local-benchmark-workspace-data.test.ts src/app/local-benchmark-workspace-presenter.test.ts src/app/local-benchmark-workspace-screen.test.tsx` | `passed: 16` | Workspace selection, commit invalidation, state handling, scope rejection, and screen semantics. / 覆盖 workspace selection、commit invalidation、状态处理、scope rejection 与 screen 语义。 |
| `pnpm --filter @contextlab/web exec tsx --test src/app/context-benchmark-decision-discovery-data.test.ts src/app/context-benchmark-decision-discovery-presenter.test.tsx` | `passed: 8` | Existing discovery parser/presenter only; no workspace selection wiring. / 仅既有 discovery parser/presenter；没有 workspace selection wiring。 |
| `pnpm --filter @contextlab/web test` | `208 passed, 0 failed` | Current Web suite includes exact-list selection, state handling, scope rejection, and the updated inspector lifecycle. / 当前 Web suite 包含 exact-list selection、state handling、scope rejection 与更新后的 inspector lifecycle。 |
| `pnpm check:web` | `passed` | Public SDK `15`, local SDK `104`, Web `208`, TypeScript/lint, and production build. / public SDK `15`、local SDK `104`、Web `208`、TypeScript/lint 与 production build。 |
| `cargo fmt --all -- --check`; `cargo test --workspace --quiet --no-fail-fast`; `cargo clippy --workspace --all-targets --offline -- -D warnings`; `cargo +1.85.0 check --workspace --all-targets --locked --offline`; `GRAPH_DIFF_IMPL_COUNT=1` | `passed` | Rust workspace (storage `204 passed, 39 ignored`, API `191 passed`), format, strict offline Clippy, locked MSRV, and sole GraphDiff implementation. / Rust workspace（storage `204 passed, 39 ignored`、API `191 passed`）、format、strict offline Clippy、锁定 MSRV 与唯一 GraphDiff 实现。 |
| PostgreSQL/Docker, authenticated browser, visual, remote CI, Git change-set, operator, release, production | `unobserved` / `deferred` | Not run or observed for this local receipt. / 本地回执未运行或观测这些外部边界。 |

No external release or production evidence is claimed. This local increment is complete; future
increments require their own bilingual Necessity Record and fresh provenance.

不宣称任何 external release 或 production 证据。本地增量已完成；未来增量必须有各自双语 Necessity Record 与新鲜 provenance。

## 2026-07-30 Criterion 3 Benchmark Regression and Scorecard Closure / 2026-07-30 条件 3 Benchmark 回归与 Scorecard 收束

This is a fresh local receipt for Criterion 3, not a criterion or repository closeout. The
private, read-only workflow is now reconciled as decision-list discovery -> exact decision-bound
workspace -> server-owned run details, scorecard, regression, and evaluation-diff evidence -> Web
presentation. The cohort-keyed compatibility read remains intact; clients do not recalculate
benchmark policy, and `GraphDiff::between` remains the sole graph-diff calculator.

这是条件 3 的新鲜本地回执，不是条件或仓库收束。private、read-only workflow 已对账为 decision-list discovery -> exact
decision-bound workspace -> 服务端拥有的 run details、scorecard、regression 与 evaluation-diff evidence -> Web presentation。
cohort-keyed compatibility read 保持不变；客户端不重新计算 benchmark policy，`GraphDiff::between` 仍是唯一 graph-diff calculator。

Fresh local evidence is evaluation `45 passed`, storage evidence `23 passed`, workspace
projection `7 passed`, execution `11 passed`, API `191 passed`, local SDK `104 passed`, Web
presenter/editor `22 passed`, workspace storage `204 passed, 39 ignored`, `pnpm check:web` with
public SDK `15`, local SDK `104`, Web `213`, TypeScript/lint, and production build, plus format,
strict offline Clippy, locked Rust `1.85.0`, and `GRAPH_DIFF_IMPL_COUNT=1`.

新鲜本地 evidence 为 evaluation `45 passed`、storage evidence `23 passed`、workspace projection `7 passed`、execution `11 passed`、
API `191 passed`、local SDK `104 passed`、Web presenter/editor `22 passed`、workspace storage `204 passed, 39 ignored`、
`pnpm check:web`（public SDK `15`、local SDK `104`、Web `213`、TypeScript/lint 与 production build），以及 format、strict offline
Clippy、锁定 Rust `1.85.0` 与 `GRAPH_DIFF_IMPL_COUNT=1`。

Criterion 3 remains open because the local authenticated browser runtime, Git binding, and the
other completion conditions are not all closed. Later local receipts already cover the named
benchmark breadth; it is not the current local evidence gap. The current unmet local boundary is
authenticated browser-to-BFF-to-protected-Axum lifecycle smoke, which remains unobserved in this
workspace. PostgreSQL/Docker runtime, visual, and Git remain `unobserved`; remote CI, operator
rehearsal, release, and production remain `deferred`. No public REST/OpenAPI/public SDK write, Web
mutation, provider, migration, secret, or external deployment claim is made. The next local
increment requires a new bilingual Necessity Record and must be admitted only for a demonstrated
dependency-ready criterion.

条件 3 仍保持开放，因为 authenticated browser runtime、Git binding 与其他完成条件尚未全部收束。
后续本地回执已经覆盖具名的 benchmark breadth；它不是当前本地证据缺口。当前未收束的本地边界是
authenticated browser-to-BFF-to-protected-Axum lifecycle smoke，本工作区尚未观测到该证据。PostgreSQL/Docker runtime、visual 与 Git
仍为 `unobserved`；remote CI、operator rehearsal、release 与 production 仍为 `deferred`。没有新增 public REST/OpenAPI/public SDK write、Web
mutation、provider、migration、secret 或 external deployment 声明。下一项本地增量必须新增双语 Necessity Record，并且只能为已有真实依赖
就绪的完成条件准入。

## 2026-07-30 Private Plugin/MCP Capability Availability Read Admission / 2026-07-30 私有 Plugin/MCP 能力可用性读取准入

The next local increment is admitted, not completed. The Rust `contextlab-mcp` and
`contextlab-plugin-runtime` contracts already provide versioned manifests, compatibility,
lifecycle, registry, safe diagnostics, and deterministic `CapabilityAvailabilityProjection`; the
missing local convergence boundary is a private Context-scoped read through API, non-public SDK,
same-origin BFF, and shared capability-state Web composition. Its frozen response will expose only
stable plugin/capability IDs, semver, availability, compatibility, and nullable safe diagnostic
codes. Empty or unavailable data is required when no runtime is registered.

下一项本地增量已准入但尚未完成。Rust `contextlab-mcp` 与 `contextlab-plugin-runtime` contract 已提供 versioned manifest、
compatibility、lifecycle、registry、安全 diagnostic 与确定性的 `CapabilityAvailabilityProjection`；当前缺少的本地收束边界是经由
API、非公开 SDK、同源 BFF 与 shared capability-state Web composition 完成 private Context-scoped read。冻结 response 只暴露稳定
plugin/capability ID、semver、availability、compatibility 与可空安全 diagnostic code；没有注册 runtime 时必须返回 empty 或
unavailable data。

The admitted boundary is private and read-only: no public REST/OpenAPI/public SDK write, registry
mutation, dynamic loading, provider call, migration, secret access, operator transport, or second
`GraphDiff` calculator. Focused MCP/runtime/API/SDK/Web tests, `pnpm check:web`, workspace Rust,
format, strict offline Clippy, locked Rust `1.85.0`, and static surface checks are required before a
completion receipt can be written. Criterion 7 and the long-term goal remain open.

本准入边界为 private、read-only：不增加 public REST/OpenAPI/public SDK write、registry mutation、dynamic loading、provider call、
migration、secret access、operator transport 或第二个 `GraphDiff` calculator。完成回执前必须取得 focused MCP/runtime/API/SDK/Web tests、
`pnpm check:web`、workspace Rust、format、strict offline Clippy、锁定 Rust `1.85.0` 与 static surface checks 的新鲜结果。条件 7 与
长期目标保持开放。

## 2026-07-30 Private Plugin/MCP Capability Availability Read Closure / 2026-07-30 私有 Plugin/MCP 能力可用性读取收束

### Status / 状态

`completed / verified locally` / `completed / 本地已验证`. This local, private, read-only
increment advances Criterion 7 without closing Criterion 7 or the long-term goal. The existing
MCP/plugin-runtime projection is now consumed by the protected API, non-public local SDK,
same-origin BFF, and shared bilingual Web capability-state composition. The response remains
redacted, exact-scope, deterministic, provider-free, request-scoped, and `no-store`.

`completed / verified locally` / `completed / 本地已验证`。本地 private、read-only 增量推进条件 7，但不关闭条件 7 或长期目标。
既有 MCP/plugin-runtime projection 现由 protected API、非公开 local SDK、同源 BFF 与 shared 双语 Web capability-state composition
消费。response 继续保持脱敏、exact scope、确定性、provider-free、request-scoped 与 `no-store`。

### Verification and boundary / 验证与边界

| Command / 命令 | Status / 状态 | Receipt / 回执 |
| --- | --- | --- |
| Focused MCP/runtime/API/SDK/Web tests | `passed` | MCP `8`, plugin-runtime `9`, API `2`, local SDK `3`, Web BFF/inspector `5` passed. / 分别通过 `8`、`9`、`2`、`3`、`5` 项。 |
| `pnpm check:web` | `passed` | Public SDK `15`, local SDK `107`, Web `218`, TypeScript/lint, and production build. / public SDK `15`、local SDK `107`、Web `218`、TypeScript/lint 与 production build。 |
| `cargo test --workspace --quiet --no-fail-fast` | `passed` | Storage `204 passed, 39 ignored`. / storage `204 passed, 39 ignored`。 |
| `cargo +1.85.0 check --workspace --all-targets --locked --offline` | `passed` | Locked Rust `1.85.0` check; two unrelated `missing_docs` warnings remained. / 锁定 Rust `1.85.0` check；保留两个无关 `missing_docs` warning。 |
| `cargo fmt --all -- --check` | `failed` | Unrelated formatting drift in `crates/workflow/src/execution_status.rs`; outside this worker's ownership. / 无关 Workflow formatting drift，超出本 worker ownership。 |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | `failed` | Two unrelated `missing_docs` errors in the same Workflow file; not a Plugin/MCP product blocker. / 同一 Workflow 文件的两个无关 `missing_docs` error，不是 Plugin/MCP product blocker。 |
| `GRAPH_DIFF_IMPL_COUNT=1`; public Plugin/MCP capability surface hits `0` | `passed` | One GraphDiff implementation and no public Plugin/MCP capability route/method hits. / 唯一一个 GraphDiff 实现，且无 public Plugin/MCP capability route/method 命中。 |

No public REST/OpenAPI/public SDK method, registry mutation, dynamic loading, provider call,
secret, or second GraphDiff calculator was added. PostgreSQL/Docker runtime, browser, Git, remote
CI, operator, release, and production remain `unobserved` or `deferred`.

没有新增 public REST/OpenAPI/public SDK method、registry mutation、dynamic loading、provider call、secret 或第二个 GraphDiff
calculator。PostgreSQL/Docker runtime、browser、Git、remote CI、operator、release 与 production 继续为 `unobserved` 或 `deferred`。

Two failed coding-worker dispatches were observed before any worker patch was accepted. They are
execution provenance only and are not product blockers. / 两次 coding-worker dispatch 在接受任何 worker patch 前失败；它们仅是
execution provenance，不是 product blocker。

## 2026-07-30 Private Workflow Execution Status Inspector Mount / 2026-07-30 私有 Workflow 执行状态检查器挂载

| Status / 状态 | Criterion mapping and boundary / 条件映射与边界 |
| --- | --- |
| `completed / verified locally` / `completed / 本地已验证` | This local-only, private, read-only increment advances Criteria 1 and 5 by mounting the existing workflow execution-status projection into the Context workspace binding inspector. The user selects an exact redacted binding row and explicitly enters a canonical `run_id`; no evaluation ID, fixture, preview, or current-head inference is allowed. / 本地 private、read-only 增量通过将既有 workflow execution-status projection 挂载到 Context workspace binding inspector，推进条件 1 与 5。用户选择精确脱敏 binding row 并显式输入 canonical `run_id`；禁止从 evaluation ID、fixture、preview 或 current head 推导。 |
| Scope invariants / 范围不变量 | Context/commit remount key and render-time scope guard reject stale state; exact binding/workflow/revision/run scope is preserved; 404 maps to `empty`; BFF and Web data/presenter layers redact upstream messages; `run_id` is required and carries `aria-required`, `aria-invalid`, and an associated alert. Web TypeScript excludes only transient `.next/dev` generated types while retaining production `.next/types`. / Context/commit remount key 与渲染时 scope guard 拒绝旧状态；保留 exact binding/workflow/revision/run scope；404 映射为 `empty`；BFF 与 Web data/presenter layer 脱敏 upstream message；`run_id` required，并具备 `aria-required`、`aria-invalid` 与关联 alert。Web TypeScript 仅排除临时 `.next/dev` generated types，同时保留 production `.next/types`。 |
| Fresh verification / 新鲜验证 | Focused inspector `15`, workspace reachability `2`, execution-status data/presenter `7`; Web `310 passed`; `pnpm check:web` public SDK `15`, local SDK `148`, Web `310`, TypeScript/lint, and production build; Rust format, workspace tests (API `223 passed`, storage `239 passed, 41 ignored`), strict offline Clippy, locked Rust `1.85.0`, and exactly one production `impl GraphDiff` passed. / focused inspector `15`、workspace reachability `2`、execution-status data/presenter `7`；Web `310 passed`；完整 Web、Rust、format、Clippy、MSRV 与唯一 GraphDiff 检查均通过。 |
| Evidence boundary / 证据边界 | No producer, persistence, polling, run discovery, mutation, public REST/OpenAPI/SDK operation, provider, migration, secret, Docker/PostgreSQL runtime, authenticated browser, Git change-set, release, or production claim was added. Docker/PostgreSQL, browser, visual, and Git remain `unobserved`; remote CI, operator rehearsal, release, and production remain `deferred`. / 不新增 producer、persistence、polling、run discovery、mutation、public REST/OpenAPI/SDK operation、provider、migration、secret 或外部部署声明；相关证据按 `unobserved/deferred` 保持。 |
| Known baseline verifier gap / 已知基线 verifier 缺口 | `scripts/verify-local-contracts.ps1` still reports `benchmark-workspace-route-method-count:2` because the existing protected benchmark workspace has two GET route variants. This pre-existing verifier mismatch is recorded as baseline `blocked`, not silently converted to `passed`, and is outside this inspector increment. / `scripts/verify-local-contracts.ps1` 因既有 protected benchmark workspace 含两个 GET route variant 仍报告 `benchmark-workspace-route-method-count:2`。该既有 verifier mismatch 记录为 baseline `blocked`，不静默改写为 `passed`，且不属于本 inspector 增量。 |

This receipt advances the named local criteria but closes neither the criteria nor the long-term
goal. The next admitted work requires a new bilingual Necessity Record; the current candidate is a
documented CLI read-only smoke path for Criterion 8 after fresh executable/exit-code verification.

本回执推进已命名的本地条件，但不关闭条件或长期目标。下一项准入工作必须有新的双语 Necessity Record；当前候选是条件 8 的 CLI
只读 smoke path 文档，前提是先对既有 executable/exit-code 进行新鲜验证。

## 2026-07-30 Private CLI Read-only Smoke Receipt / 2026-07-30 私有 CLI 只读 Smoke 回执

| Status / 状态 | Criterion mapping and boundary / 条件映射与边界 |
| --- | --- |
| `completed / verified locally` / `completed / 本地已验证` | This documentation-only increment supplies Criterion 8's named documented CLI smoke evidence. It records existing read-only staging behavior; no CLI command or shared Rust domain contract changed. / 本 documentation-only 增量补齐条件 8 已命名的 CLI smoke 文档证据，只记录既有只读 staging behavior；没有改变 CLI command 或 shared Rust domain contract。 |
| Fresh binary evidence / 新鲜 binary 证据 | `cargo build --offline -p contextlab-cli` passed. Replay projection exited `0`; valid unavailable capability exited `2`; capability operation/integration drift exited `64`; workspace adapter unavailable exited `2`. / 构建通过，四个 binary probe 退出码均已真实观测。 |
| Focused quality evidence / 聚焦质量证据 | Adapter contract `8 passed`, CLI `5 passed`, Desktop staging `5 passed`; scoped strict Clippy, format, and locked Rust `1.85.0` passed. / adapter contract `8`、CLI `5`、Desktop staging `5` 项通过；Clippy、format 与 MSRV 通过。 |
| Evidence boundary / 证据边界 | No command, public transport, Web mutation, provider, secret, Tauri runtime, Docker/PostgreSQL, release, or production surface changed. Tauri runtime, Docker/PostgreSQL, authenticated browser, Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`. / 未新增产品或外部部署能力；对应证据继续为 `unobserved/deferred`。 |

This receipt advances Criterion 8 but does not close Criterion 8 or the long-term goal. The next
implementation requires a new bilingual Necessity Record and a bounded Criterion 1 audit for the
smallest dependency-ready Context-first gap, including component content update/replay coverage.

本回执推进条件 8，但不关闭条件 8 或长期目标。下一项 implementation 必须有新的双语 Necessity Record，并对最小依赖就绪的 Context-first
gap 做有界条件 1 审计，包含 component content update/replay coverage。

## 2026-07-30 Private Component Content Replay Witness Integrity / 2026-07-30 私有组件正文回放 Witness 一致性

| Criterion mapping / 条件映射 | Fresh local receipt / 新鲜本地回执 |
| --- | --- |
| Criteria 1 and 2 advanced; neither criterion is closed. / 条件 1 与 2 得到推进，但均未关闭。 | The Rust lifecycle read fails closed unless the exact Context, component, kind, resulting hash, and body revision recording commit agree with the replayed component state. The red compile phase for the missing helper was observed, followed by green regression `1`, lifecycle `10`, component-content `10 passed, 5 ignored`, replay-state `7`, and workspace Rust with storage `205 passed, 39 ignored`. / Rust lifecycle read 现在只有在 exact Context、component、kind、resulting hash 与 body revision recording commit 均与 replayed component state 一致时才接受；missing helper 的红阶段编译失败已观测，随后 regression `1`、lifecycle `10`、component-content `10 passed, 5 ignored`、replay-state `7` 与 workspace Rust storage `205 passed, 39 ignored` 通过。 |
| Release-quality evidence / 发布质量证据 | `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web` (public SDK `15`, local SDK `111`, Web `236`, TypeScript/lint, production build), and `GRAPH_DIFF_IMPL_COUNT=1` passed. / format、strict offline Clippy、锁定 Rust `1.85.0`、Web 全检查与唯一 GraphDiff 计数均通过。 |
| Boundary / 边界 | No public write, REST/OpenAPI/public SDK method, Web mutation, migration, provider, secret, operator transport, or second GraphDiff calculator was added. PostgreSQL/Docker runtime, authenticated browser, visual, Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; the long-term goal remains active. / 未新增 public write、REST/OpenAPI/public SDK method、Web mutation、migration、provider、secret、operator transport 或第二个 GraphDiff calculator；PostgreSQL/Docker runtime、authenticated browser、visual、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved/deferred`；长期目标保持 active。 |

This receipt strengthens the named replay integrity predecessor for Criteria 1 and 2; it does not
establish complete Context-first coverage, versioning/diff coverage, release readiness, or project
completion. The next implementation requires its own bilingual Necessity Record.

本回执强化条件 1 与 2 已命名的 replay integrity 前置，但不证明 Context-first coverage、版本/Diff coverage、release readiness 或
项目完成。下一项 implementation 必须具备独立的双语 Necessity Record。

## 2026-07-30 Private Persisted Context Diff Review Read / 2026-07-30 私有持久化 Context Diff Review 读取

| Criterion mapping / 条件映射 | Fresh local receipt / 新鲜本地回执 |
| --- | --- |
| Criteria 2 and 4 advanced; neither criterion is closed. / 条件 2 与 4 得到推进，均未关闭。 | The private exact-commit diff-review projection is now consumable through protected API, non-public local SDK, same-origin BFF, and shared Web `data -> presenter -> screen`. The client does not calculate semantic, behavior, evaluation, or graph diffs. / private exact-commit diff-review projection 现可经 protected API、非公开 local SDK、同源 BFF 与 shared Web `data -> presenter -> screen` 消费；client 不计算 semantic、behavior、evaluation 或 graph diff。 |
| Fresh verification / 新鲜验证 | Focused API diff-review `6 passed`; workspace Rust passed with storage `205 passed, 39 ignored`; format, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web` with public SDK `15`, local SDK `117`, Web `245`, TypeScript/lint and production build, `GRAPH_DIFF_IMPL_COUNT=1`, public OpenAPI diff-review hits `0`, and private handler hits `1`. / focused API `6 passed`；workspace Rust storage `205 passed, 39 ignored`；format、strict offline Clippy、锁定 Rust `1.85.0`、Web `15/117/245` 与 TypeScript/lint、production build、唯一 GraphDiff、public OpenAPI 命中 `0`、private handler 命中 `1` 均通过。 |
| Boundary / 边界 | No public REST/OpenAPI/public SDK method, write route, Web mutation, provider, migration, secret, operator transport, or second GraphDiff calculator was added. Docker/PostgreSQL runtime, authenticated browser, visual smoke, Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`. / 未新增 public REST/OpenAPI/public SDK method、写入 route、Web mutation、provider、migration、secret、operator transport 或第二个 GraphDiff calculator；Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved/deferred`。 |

This is a local convergence receipt only. Criteria 2 and 4 remain open because complete replayable
Context coverage, graph editing breadth, authenticated runtime evidence, Git binding, and the other
completion conditions are not closed. The long-term goal remains active. The next increment must
have its own bilingual Necessity Record and fresh scope-matched evidence.

本回执仅是本地收束证据。条件 2 与 4 仍开放，因为完整可回放 Context coverage、图谱编辑 breadth、authenticated runtime evidence、Git
binding 与其他完成条件均未关闭。长期目标保持 active；下一项增量必须拥有独立双语 Necessity Record 与范围匹配的新鲜证据。

## 2026-07-30 Private Context Commit Ancestry and Server-Owned Merge Plan / 2026-07-30 私有 Context Commit Ancestry 与服务端 Merge Plan

| Criterion mapping / 条件映射 | Fresh local receipt / 新鲜本地回执 |
| --- | --- |
| Criteria 2 and 4 advanced; neither criterion is closed. / 条件 2 与 4 得到推进，均未关闭。 | The exact Context commit-DAG port is implemented and wired through the internal API repository bundle. The private merge-review service resolves `MergePlan` from two typed tips through that DAG, rejects unknown tips and non-three-way ancestry, and delegates graph comparison only to the existing versioned classifier. / exact Context commit-DAG port 已实现并接入内部 API repository bundle。private merge-review service 通过该 DAG 从两个 typed tip 解析 `MergePlan`，拒绝 unknown tip 与 non-three-way ancestry，图比较只委托既有 versioned classifier。 |
| Verification / 验证 | Ancestry storage `2` integration + `13` unit, API repository contract `4`, server-owned merge-review `11`, workspace storage `208 passed, 39 ignored`; format, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web` public SDK `15`, local SDK `117`, Web `245`, production build, OpenAPI exclusion `1`, and `GRAPH_DIFF_IMPL_COUNT=1` passed. / 以上各项均取得新鲜通过回执。 |
| Boundary / 边界 | No public REST/OpenAPI/SDK method, Web/CLI/Desktop surface, mutation, merge writer, migration, provider, secret, operator transport, or second GraphDiff calculator was added. PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; the long-term goal remains active. / 未新增公共或写入表面；相关 runtime 与发布证据继续为 `unobserved/deferred`，长期目标保持 active。 |

This receipt closes neither Criterion 2 nor Criterion 4. A future transport may consume the
server-owned review only after a separate Necessity Record, contract decision, and scope-matched
API/SDK/Web verification; the current local increment deliberately stops at the reusable Rust
domain/storage boundary.

本回执不关闭条件 2 或条件 4。未来 transport 只有在独立 Necessity Record、契约决策与范围匹配的 API/SDK/Web 验证之后才能消费
server-owned review；当前本地增量明确停留在可复用 Rust domain/storage boundary。

## 2026-07-30 Private Context Merge Review Inspector Mount / 2026-07-30 私有 Context Merge Review Inspector 挂载

### Status / 状态

`completed / verified locally` / `completed / verified locally`. The existing private,
server-owned merge-review projection is now reachable from the mounted Context workspace through
the same-origin BFF and shared `data -> presenter -> screen` layers. The inspector is independently
gated by `CONTEXTLAB_ENABLE_LOCAL_CONTEXT_MERGE_REVIEW=true`, defaults off, uses request-memory
Bearer only, and binds every request to the selected project, Context, left commit, and right
commit. Context or candidate changes clear the previous selection and resource. The Web resource
boundary accepts only normalized V1 data or a structured wire record and never calculates a
graph diff.

`completed / verified locally` / `completed / 本地已验证`。已有的 private、server-owned merge-review projection 现在通过同源 BFF 与共享
`data -> presenter -> screen` 层从 mounted Context workspace 可达。inspector 使用独立的
`CONTEXTLAB_ENABLE_LOCAL_CONTEXT_MERGE_REVIEW=true` gate，默认关闭，仅使用 request-memory Bearer，并将每次请求绑定到 selected project、Context、
left commit 与 right commit。Context 或 candidate 变化会清除旧 selection 与 resource。Web resource boundary 只接受 normalized V1 data 或
structured wire record，不计算 graph diff。

### Fresh local evidence / 新鲜本地证据

- Red phase: missing inspector module was observed as `ERR_MODULE_NOT_FOUND`.
- `pnpm --filter @contextlab/web lint`: passed; full Web tests: `260 passed`.
- `pnpm check:web`: public SDK `15`, local SDK `126`, Web `260`, TypeScript/lint, and production build passed.
- `cargo test --workspace --quiet`: passed; storage `208 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust `1.85.0` check passed.
- Static checks: production `impl GraphDiff` count `1`; public OpenAPI/SDK merge-review hits `0`.

- 红阶段：缺失 inspector 模块被真实观察为 `ERR_MODULE_NOT_FOUND`。
- `pnpm --filter @contextlab/web lint` 通过；完整 Web tests：`260 passed`。
- `pnpm check:web` 通过：public SDK `15`、local SDK `126`、Web `260`、TypeScript/lint 与 production build。
- `cargo test --workspace --quiet` 通过；storage 为 `208 passed, 39 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- 静态检查：production `impl GraphDiff` 数量 `1`；public OpenAPI/SDK merge-review 命中 `0`。

This advances Criteria 2, 4, and 9 but closes none of them and does not close the long-term goal.
No public REST/OpenAPI/public SDK method, merge writer, branch mutation, Web mutation, migration,
provider, secret access, operator transport, or second `GraphDiff` calculator was added.
PostgreSQL/Docker runtime, authenticated browser, visual smoke, and Git remain `unobserved`;
remote CI, operator rehearsal, release, and production remain `deferred`.

本次推进条件 2、4 与 9，但不关闭其中任何条件，也不关闭长期目标。未新增 public REST/OpenAPI/public SDK method、merge writer、branch mutation、
Web mutation、migration、provider、secret access、operator transport 或第二个 `GraphDiff` calculator。PostgreSQL/Docker runtime、authenticated
browser、visual smoke 与 Git 继续为 `unobserved`；remote CI、operator rehearsal、release 与 production 继续为 `deferred`。

### Next queue / 下一队列

Keep the long-term goal active. Before the next implementation, perform a bounded audit of the
highest-priority dependency-ready local completion gap and write a new bilingual Necessity Record.
Do not add public transport or mutation work while the current local evidence boundary remains the
scope.

保持长期目标 active。下一项实现前，对最高优先级且依赖就绪的本地收束缺口做有界审计，并新增双语 Necessity Record。在当前本地证据边界内，
不得新增 public transport 或 mutation work。

### 2026-07-31 Private Context Metadata Semantic Diff Receipt / 2026-07-31 私有 Context Metadata Semantic Diff 回执

| Criterion/status / 条件与状态 | Evidence and boundary / 证据与边界 |
| --- | --- |
| Criteria 1, 2, 4, and 9 advanced locally; none is closed. The long-term goal remains active. / 条件 1、2、4、9 获得本地推进；均未关闭，长期目标保持 active。 | The current private contract carries optional Context metadata in `SemanticSnapshotV1`, returns typed `ContextMetadataChangeV1` `added`/`removed`/`modified` transitions, and preserves `GraphDiff::between` as the sole graph-diff calculator. / 当前 private contract 在 `SemanticSnapshotV1` 携带可选 Context metadata，返回 typed `ContextMetadataChangeV1` 三态 transition，并保持 `GraphDiff::between` 为唯一 graph-diff calculator。 |
| Fresh local verification / 新鲜本地验证 | Diff focused `4 passed`; storage review `7 passed`; `cargo test --workspace --quiet --no-fail-fast` passed with `40 ignored`; strict offline Clippy, locked Rust `1.85.0` check, fmt, `pnpm check:web` (`15` public SDK, `134` local SDK, `270` Web, TypeScript/lint and production build), and `impl GraphDiff=1` passed. / diff focused `4 passed`；storage review `7 passed`；workspace 通过且 `40 ignored`；Clippy、MSRV、fmt、`pnpm check:web`（public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build）及唯一 GraphDiff 通过。 |
| Open evidence / 开放证据 | Rust enum variant unknown-field rejection and storage exact-commit replay for `added`/`removed` metadata remain unobserved. PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`. / Rust enum variant unknown-field rejection 与 `added`/`removed` metadata storage exact-commit replay 尚未观测；PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。 |
| Scope gate / 范围门禁 | No public REST/OpenAPI/SDK write, Web mutation, migration, provider, operator transport, secret access, or second `GraphDiff` calculator was added. / 未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、operator transport、secret access 或第二个 `GraphDiff` calculator。 |
| Next necessity record / 下一项必要性记录 | The next admitted work is only focused Rust fail-closed and existing storage replay tests for the two missing transition receipts; it must be verified before choosing another increment. / 下一项仅准入 focused Rust fail-closed 与既有 storage replay tests，以补齐两个缺失 transition 回执；必须验证后才能选择下一增量。 |

### 2026-07-31 Metadata Transition Evidence Closure / 2026-07-31 Metadata Transition 证据收束

| Criterion/status / 条件与状态 | Evidence and boundary / 证据与边界 |
| --- | --- |
| Criteria 1, 2, 4, and 9 advanced; none is closed. The long-term goal remains active. / 条件 1、2、4、9 获得推进；均未关闭，长期目标保持 active。 | The real red phase found unknown metadata-variant fields were accepted. The minimal repair adds `deny_unknown_fields` to `ContextMetadataChangeV1`; storage review now proves exact-commit `added`, `removed`, and `modified` transitions. / 真实红阶段发现未知 metadata variant 字段会被接受；最小修复是在 `ContextMetadataChangeV1` 加入 `deny_unknown_fields`；storage review 现已证明 exact-commit `added`、`removed`、`modified` transition。 |
| Fresh verification / 新鲜验证 | Diff focused `5 passed`; storage review `8 passed`; workspace passed with storage `212 passed, 39 ignored`; strict offline Clippy, locked Rust `1.85.0` check, fmt, `pnpm check:web` (`15` public SDK, `134` local SDK, `270` Web, TypeScript/lint and production build), and `impl GraphDiff=1` passed. / diff focused `5 passed`；storage review `8 passed`；workspace storage `212 passed, 39 ignored`；Clippy、MSRV、fmt、`pnpm check:web`（public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build）与唯一 GraphDiff 通过。 |
| Scope gate / 范围门禁 | No public REST/OpenAPI/SDK write, Web mutation, migration, provider, operator transport, secret access, Docker/PostgreSQL runtime claim, external receipt, release, production, or second `GraphDiff` calculator was added. / 未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、operator transport、secret access、Docker/PostgreSQL runtime 声明、external receipt、release、production 或第二个 `GraphDiff` calculator。 |
| Next queue / 下一队列 | This closes only the bounded metadata evidence increment. The next increment requires a fresh bilingual Necessity Record and must serve a different dependency-ready named gap; do not close the long-term goal. / 本回执仅收束有界 metadata evidence 增量。下一项必须先有新的双语 Necessity Record，并服务另一个依赖就绪的命名缺口；不得关闭长期目标。 |

### 2026-08-01 Private Benchmark Multi-Dataset Breadth Receipt / 2026-08-01 私有 Benchmark 多 Dataset 宽度回执

This local evidence increment advances Criterion 3 without closing it or the long-term goal. The
storage fixture covers two datasets and four cases at baseline/revised Context commits,
deterministic ordering, replay, exact scope, persisted runs, scorecard/regression metadata,
evaluation-diff metadata, decision-bound workspace reads, and raw-payload-safe projections. The
API fixture covers two datasets and four cases at one revised exact commit through protected
authoring/workspace/decision-workspace routes. Its execution stage is a direct call to the real
`BenchmarkExecutionService`, not an execution POST, API-level baseline/revised comparison, or full
REST execution adapter receipt.

本地证据增量推进条件 3，但不关闭条件 3 或长期目标。storage fixture 覆盖 baseline/revised Context commit 上的两个 dataset、四个 case、
确定性排序、replay、精确 scope、持久化 run、scorecard/regression metadata、evaluation-diff metadata、decision-bound workspace read 与
raw-payload-safe projection。API fixture 在一个 revised 精确 commit 上通过受保护的 authoring/workspace/decision-workspace route 覆盖两个
dataset/四个 case。API execution 阶段直接调用真实 `BenchmarkExecutionService`，不是 execution POST、API-level baseline/revised comparison 或完整 REST execution adapter 回执。

Fresh local receipts / 新鲜本地回执：storage breadth `1 passed`; API breadth `1 passed`; `cargo fmt --all -- --check` passed;
`cargo test --workspace --quiet --no-fail-fast --offline` passed with storage `212 passed, 39 ignored`; strict offline Clippy passed;
locked Rust `1.85.0` check passed; `pnpm check:web` passed with public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production
build; local verifier source/graph/safe-DTO/protected-route checks passed with `overall=unobserved` because no unified diff input was supplied;
`GRAPH_DIFF_IMPL_COUNT=1`.

新鲜本地回执：storage breadth `1 passed`；API breadth `1 passed`；`cargo fmt --all -- --check` 通过；`cargo test --workspace --quiet --no-fail-fast --offline`
通过且 storage `212 passed, 39 ignored`；strict offline Clippy 通过；锁定 Rust `1.85.0` check 通过；`pnpm check:web` 通过（public SDK `15`、local SDK `134`、
Web `270`、TypeScript/lint 与 production build）；local verifier 的 source/graph/safe-DTO/protected-route checks 通过，但因未提供 unified diff input，
`overall=unobserved`；`GRAPH_DIFF_IMPL_COUNT=1`。

No public write, OpenAPI/SDK write, Web mutation, migration, provider, secret access, PostgreSQL/Docker runtime, authenticated browser,
visual smoke, Git change-set, remote CI, operator rehearsal, release, production claim, or second `GraphDiff` calculator was added. Criterion 3
is advanced but remains open, and all external/runtime facts remain `unobserved` or `deferred`.

未新增 public write、OpenAPI/SDK write、Web mutation、migration、provider、secret access、PostgreSQL/Docker runtime、authenticated browser、visual smoke、Git change-set、
remote CI、operator rehearsal、release、production 声明或第二个 `GraphDiff` calculator。条件 3 获得推进但仍开放，所有外部/runtime 事实继续为 `unobserved` 或 `deferred`。

### 2026-08-01 Private Benchmark Vertical Breadth Completion / 2026-08-01 私有 Benchmark 垂直宽度收束

This follow-up advances Criterion 3 without closing it. The API fixture now proves baseline/revised
in-memory execution and replay, protected workspace and decision-workspace comparisons, two
datasets/four cases, stable rows, exact scope, scorecard coverage, `passed -> regressed` evaluation
diff, and recursive redaction. It directly invokes the real `BenchmarkExecutionService`; it is not
an execution POST or full REST adapter receipt. The Web inspector fixture carries the matching
redacted projection through data -> presenter -> screen and proves four-run ordering/coverage,
regression/diff rendering, bilingual accessibility state, and raw-payload exclusion. It is a local
mocked Web contract receipt, not authenticated browser evidence.

本次后续推进条件 3，但不关闭条件 3。API fixture 现证明 baseline/revised in-memory execution 与 replay、protected workspace 与 decision-workspace comparison、
两个 dataset/四个 case、稳定 row、精确 scope、scorecard coverage、`passed -> regressed` evaluation diff 与递归脱敏。它直接调用真实
`BenchmarkExecutionService`，不是 execution POST 或完整 REST adapter 回执。Web inspector fixture 将匹配的脱敏 projection 穿过 data -> presenter -> screen，
证明四 run 的排序/coverage、regression/diff 渲染、双语可访问状态与 raw-payload 排除；这是本地 mocked Web contract 回执，不是 authenticated browser 证据。

Fresh verification / 新鲜验证：API focused `1 passed`; Web inspector focused `6 passed`; `cargo fmt --all -- --check` passed;
offline workspace Rust passed with storage `212 passed, 39 ignored`; strict offline Clippy passed; locked Rust `1.85.0` check passed;
`pnpm check:web` passed with public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build; local verifier source/graph/safe DTO/
protected route/catalog/public boundary checks passed with `overall=unobserved` because no unified diff input was supplied; `GRAPH_DIFF_IMPL_COUNT=1`.

新鲜验证：API focused `1 passed`；Web inspector focused `6 passed`；`cargo fmt --all -- --check` 通过；offline workspace Rust 通过且 storage `212 passed, 39 ignored`；
strict offline Clippy 通过；锁定 Rust `1.85.0` check 通过；`pnpm check:web` 通过（public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build）；
local verifier 的 source/graph/safe DTO/protected route/catalog/public boundary checks 通过，但因未提供 unified diff input，`overall=unobserved`；`GRAPH_DIFF_IMPL_COUNT=1`。

No public write, OpenAPI/SDK write, Web mutation, migration, provider, secret access, PostgreSQL/Docker runtime, authenticated browser/visual smoke,
Git change-set, remote CI, operator rehearsal, release, production claim, or second `GraphDiff` calculator was added. Criterion 3 and the long-term goal remain open.

未新增 public write、OpenAPI/SDK write、Web mutation、migration、provider、secret access、PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git change-set、
remote CI、operator rehearsal、release、production 声明或第二个 `GraphDiff` calculator。条件 3 与长期目标继续开放。

### 2026-08-01 Private CLI Read-only Smoke Receipt / 2026-08-01 私有 CLI 只读 Smoke 回执

This local process receipt advances Criterion 8 without closing it or the long-term goal. The
offline-built provider-free CLI observed the documented read-only exit semantics: valid replay `0`
with deterministic redacted identity/count/ID output; valid unavailable capability `2` with
bilingual output; capability operation/integration drift `64` with empty stdout and only the
bilingual invalid-contract stderr; and adapter-backed workspace `2` with an explicit unavailable
registration message. Adapter contract `8`, CLI `5`, Desktop staging `5`, format, and scoped strict
Clippy passed.

本次本地进程回执推进条件 8，但不关闭条件 8 或长期目标。离线构建的 provider-free CLI 观测到文档化的只读 exit 语义：valid replay `0`，输出确定性脱敏的
identity/count/ID；valid unavailable capability `2`，输出双语；capability operation/integration drift `64`，stdout 为空且 stderr 只有双语 invalid-contract；
adapter-backed workspace `2`，输出明确的 unavailable registration message。adapter contract `8`、CLI `5`、Desktop staging `5`、format 与 scoped strict Clippy 均通过。

Fresh local verification / 新鲜本地验证：`cargo build --offline -p contextlab-cli` passed; focused adapter `8 passed`, CLI `5 passed`, Desktop staging `5 passed`;
`cargo fmt --all -- --check` passed; scoped strict Clippy passed; the current locked Rust workspace check, local verifier, and `GRAPH_DIFF_IMPL_COUNT=1` passed.

新鲜本地验证：`cargo build --offline -p contextlab-cli` 通过；adapter `8 passed`、CLI `5 passed`、Desktop staging `5 passed`；`cargo fmt --all -- --check`、scoped strict
Clippy、当前锁定 Rust workspace check、local verifier 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。

No CLI write, Context mutation, provider/network/credential access, Desktop/Tauri runtime claim,
public REST/OpenAPI/SDK method, Web mutation, migration, PostgreSQL/Docker runtime, authenticated
browser, visual smoke, Git change-set, remote CI, operator rehearsal, release, production claim, or
second `GraphDiff` calculator was added. Criterion 8 and the long-term goal remain open.

未新增 CLI write、Context mutation、provider/network/credential access、Desktop/Tauri runtime 声明、public REST/OpenAPI/SDK method、Web mutation、migration、
PostgreSQL/Docker runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release、production 声明或第二个 `GraphDiff` calculator。
条件 8 与长期目标继续开放。

### 2026-08-01 Private Memory Writer-to-Review Composition / 2026-08-01 私有 Memory Writer-to-Review 组合

This local root-cause increment advances Criteria 1, 2, and 4 without closing them or the
long-term goal. The Memory `WorkspaceDataRepository` now reuses the shared
`InMemoryContextGraphRepository` clone for diff snapshot reads; a red-to-green regression proves a
guarded commit writes a derived exact-scope diff snapshot that the persisted-review adapter can
read back. No behavior/evaluation facts are invented when no producer supplies them.

本地根因增量推进条件 1、2 与 4，但不关闭这些条件或长期目标。Memory `WorkspaceDataRepository` 现为 diff snapshot read 复用共享的
`InMemoryContextGraphRepository` clone；red-to-green regression 证明 guarded commit 写入的派生 exact-scope diff snapshot 可由 persisted-review adapter 读回。没有 producer 时不制造 behavior/evaluation facts。

Fresh verification / 新鲜验证：API lib `215 passed`; storage snapshot repository `4 passed`; workspace Rust API `215 passed`, storage `212 passed, 39 ignored`; format; strict offline
Clippy; locked Rust `1.85.0`; `pnpm check:web` with public SDK `15`, local SDK `134`, Web `272`, TypeScript/lint and production build; local contract verifier scoped checks; and
`GRAPH_DIFF_IMPL_COUNT=1` passed. The verifier remains `overall=unobserved` because no unified diff
input was supplied.

新鲜验证：API lib `215 passed`；storage snapshot repository `4 passed`；workspace Rust API `215 passed`、storage `212 passed, 39 ignored`；format、strict offline Clippy、锁定 Rust `1.85.0`、
`pnpm check:web`（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）、local contract verifier scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。因未提供 unified diff input，verifier 整体仍为 `overall=unobserved`。

This receipt proves only local in-memory repository identity. The generic fixture constructor remains
test-composed and protected runtime uses PostgreSQL; PostgreSQL/Docker runtime, authenticated
browser/visual smoke, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`. No public write, OpenAPI/public SDK write method, Web mutation, migration,
provider, secret access, operator transport, or second `GraphDiff` calculator was added. The
long-term goal remains active; the next implementation requires its own bilingual Necessity Record.

本回执只证明 local in-memory repository identity。通用 fixture constructor 仍由 test 组合，protected runtime 使用 PostgreSQL；PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。
没有新增 public write、OpenAPI/public SDK write method、Web mutation、migration、provider、secret access、operator transport 或第二个 `GraphDiff` calculator。长期目标保持 active；下一项实现必须拥有自己的双语 Necessity Record。

### 2026-08-01 Private Versioned Context Diff Boundary Witness / 2026-08-01 私有版本化 Context Diff 边界见证

This local read-contract increment advances Criteria 2 and 4 without closing either or the
long-term goal. The independent protected-router contract now proves exact Context/source/target
commit scope, authentication, private no-store response, stable non-empty graph diff, public-route
retirement, and same-commit rejection. The local SDK parser matrix covers semantic document,
behavior case, and evaluation metric `added` and `removed` variants with fail-closed exact-shape
checks.

本地读取契约增量推进条件 2 与 4，但不关闭其中任何条件或长期目标。独立 protected-router contract 现证明精确 Context/source/target commit scope、authentication、private no-store response、
稳定非空 graph diff、public route retirement 与 same-commit rejection。local SDK parser matrix 覆盖 semantic document、behavior case 与 evaluation metric 的 `added`、`removed` variant，并以
fail-closed exact-shape check 约束。

Fresh verification / 新鲜验证：protected API `3 passed`; local SDK parser `10 passed`; workspace
Rust API `214 passed`, storage `212 passed, 39 ignored`; `cargo fmt --all -- --check`; strict offline
Clippy; locked Rust `1.85.0`; `pnpm check:web` with public SDK `15`, local SDK `134`, Web `272`,
TypeScript/lint and production build; local contract verifier scoped checks; and
`GRAPH_DIFF_IMPL_COUNT=1` passed. The verifier remains `overall=unobserved` because no unified diff
input was supplied.

新鲜验证：protected API `3 passed`；local SDK parser `10 passed`；workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0`、
`pnpm check:web`（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）、local contract verifier scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。因未提供 unified diff input，verifier 整体仍为
`overall=unobserved`。

These are fixture-backed local contract receipts. They do not prove Memory writer-to-review
repository identity, PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git change-set,
remote CI, operator rehearsal, release, or production promotion. No public write, OpenAPI/public SDK
write method, Web mutation, migration, provider, secret access, operator transport, or second
`GraphDiff` calculator was added. The long-term goal remains active; the next implementation must
first add a bilingual Necessity Record for the local repository-composition gap found during review.

这些是 fixture-backed local contract receipt，不能证明 Memory writer-to-review repository identity、PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git change-set、remote CI、operator rehearsal、
release 或 production promotion。没有新增 public write、OpenAPI/public SDK write method、Web mutation、migration、provider、secret access、operator transport 或第二个 `GraphDiff` calculator。长期目标保持 active；
下一项实现前必须先为审查发现的本地 repository-composition gap 增加双语 Necessity Record。

### 2026-08-01 Private Context Diff Pair Read Consistency / 2026-08-01 私有 Context Diff 成对读取一致性

This bounded local storage increment advances Criteria 2 and 4 without closing either criterion or the long-term goal. The Rust storage boundary now exposes a typed `ContextDiffSnapshotV1Pair` through the separate `ContextDiffSnapshotV1PairRepository` read port. `PersistedContextDiffReviewService` performs one pair read, validates both exact returned scopes and V1 schemas, and delegates only to `VersionedContextDiffReviewService`; `GraphDiff::between` remains the sole graph-diff calculator. The existing single-snapshot repository port remains compatible.

本次有界 local storage 增量推进条件 2 与 4，但不关闭任一条件或长期目标。Rust storage boundary 现通过独立的 `ContextDiffSnapshotV1PairRepository` read port 暴露 typed `ContextDiffSnapshotV1Pair`。`PersistedContextDiffReviewService` 只执行一次 pair read，校验两份返回记录的 exact scope 与 V1 schema，并且只委托 `VersionedContextDiffReviewService`；`GraphDiff::between` 仍是唯一 graph-diff calculator。既有 single-snapshot repository port 保持兼容。

Memory pair reads hold one read lock for both records, and PostgreSQL pair reads use one `REPEATABLE READ READ ONLY` transaction for both exact selects and fail-closed digest/schema decoding. The Integration Lead took over the two disjoint Memory/PostgreSQL implementation lanes after the assigned Luna workers stopped producing output; no conflicting patch was retained.

Memory pair read 对两份 record 持有一个 read lock；PostgreSQL pair read 对两次 exact select 与 fail-closed digest/schema decode 使用一个 `REPEATABLE READ READ ONLY` transaction。两名被分派的 Luna worker 停止产出后，Integration Lead 接管两个不重叠的 Memory/PostgreSQL implementation lane；没有保留冲突 patch。

Fresh evidence / 新鲜证据：`context_diff_review` `9 passed`; Memory repository `5 passed`; PostgreSQL contract `1 passed, 2 ignored`; storage library `212 passed, 39 ignored`; workspace tests, formatting, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web` (`15/134/272 + production build`), verifier fixture/live scoped checks, and `GRAPH_DIFF_IMPL_COUNT=1` passed. The live verifier remains `overall=unobserved` only because no unified diff input was supplied. The ignored PostgreSQL runtime cases are not passes; Docker/virtualization, browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

新鲜证据：`context_diff_review` `9 passed`；Memory repository `5 passed`；PostgreSQL contract `1 passed, 2 ignored`；storage library `212 passed, 39 ignored`；workspace tests、formatting、strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`（`15/134/272 + production build`）、verifier fixture/live scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。live verifier 仅因未提供 unified diff input 而保持 `overall=unobserved`。PostgreSQL ignored runtime case 不计为通过；Docker/virtualization、browser、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。

No public route, OpenAPI/SDK write method, Web mutation, migration, provider, secret access, operator transport, external receipt, release, production claim, or second graph-diff calculator was added. The active long-term goal remains open, and the next increment requires a new bilingual Necessity Record.

未新增 public route、OpenAPI/SDK write method、Web mutation、migration、provider、secret access、operator transport、external receipt、release、production 声明或第二个 graph-diff calculator。长期目标保持 active；下一增量必须先新增双语 Necessity Record。

### 2026-08-01 Private Exact-Commit Context Graph Relationship Inspector / 2026-08-01 私有精确提交 Context Graph 关系检查器

This bounded local read receipt advances Criteria 1, 2, and 4 but does not close them or the
repository convergence goal. The private local SDK now accepts and validates the Rust-shaped
`graph_snapshot.project_id`. The Web relationship model lives in the presenter, preserves all
supported graph edge kinds with deterministic ordering and exact commit scope, and the editor uses
only the presenter output with shared design-system primitives. The projection exposes stable node
and edge facts, not component content or private payloads.

本次有界 local read 回执推进条件 1、2、4，但不关闭这些条件或仓库收束目标。私有 local SDK 现接受并校验 Rust-shaped
`graph_snapshot.project_id`。Web relationship model 位于 presenter，按确定性排序保留全部支持的 graph edge kind 与 exact commit scope；editor 仅使用 presenter output 与 shared design-system primitive。
projection 只暴露稳定 node/edge fact，不暴露 component content 或 private payload。

Fresh evidence / 新鲜证据：focused Web presenter/editor `29 passed`; local SDK `135 passed`; `cargo fmt --all -- --check`; workspace Rust tests with storage `212 passed, 39 ignored`; strict offline Clippy; locked Rust `1.85.0`; `pnpm check:web` with public SDK `15`, local SDK `135`, Web `276`, and production build; scoped local contract verifier; and `GRAPH_DIFF_IMPL_COUNT=1` all passed. The verifier remains `overall=unobserved` because no unified diff input was supplied.

新鲜证据：Web presenter/editor 聚焦 `29 passed`；local SDK `135 passed`；`cargo fmt --all -- --check`；workspace Rust tests（storage `212 passed, 39 ignored`）；strict offline Clippy；锁定 Rust `1.85.0`；`pnpm check:web`（public SDK `15`、local SDK `135`、Web `276` 与 production build）；scoped local contract verifier；以及 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。因未提供 unified diff input，verifier 仍为 `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, or production claim was added. The goal remains active;
the next implementation must start with a new bilingual Necessity Record and a fresh dependency
audit. Deferred external release evidence is outside the current local queue.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、
authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。目标保持 active；下一项实现必须先新增双语 Necessity Record 并进行新鲜依赖审计。
延期的外部 release evidence 不属于当前本地队列。

### 2026-08-01 Private Graph Diff Error Redaction / 2026-08-01 私有 Graph Diff 错误脱敏

This bounded local security receipt advances Criterion 4 but does not close it or the repository
convergence goal. The Web local graph-diff adapter preserves HTTP status and structured error code
while replacing structured and malformed upstream messages with one stable bilingual local review
message. The red regression exposed a synthetic `sql://internal-db?token=secret` message before the
fix; the green regression proves it no longer crosses the adapter boundary.

本次有界 local security 回执推进条件 4，但不关闭条件 4 或仓库收束目标。Web local graph-diff adapter 保留 HTTP status 与 structured error code，同时将 structured
与 malformed upstream message 替换为稳定的双语 local review message。红回归在修复前暴露 synthetic `sql://internal-db?token=secret` message；绿回归证明它不再跨过 adapter boundary。

Fresh evidence / 新鲜证据：red focused adapter `2 passed, 1 failed`; green focused adapter `3 passed`; `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `277`, TypeScript/lint and production build; `cargo fmt --all -- --check`; workspace tests with storage `212 passed, 39 ignored`; strict offline Clippy; locked Rust `1.85.0`; scoped local verifier; and `GRAPH_DIFF_IMPL_COUNT=1`. The verifier is `overall=unobserved` because no unified diff input was supplied.

新鲜证据：红 focused adapter `2 passed, 1 failed`；绿 focused adapter `3 passed`；`pnpm check:web` 通过（public SDK `15`、local SDK `135`、Web `277`、TypeScript/lint 与 production build）；`cargo fmt --all -- --check`；workspace tests（storage `212 passed, 39 ignored`）；strict offline Clippy；锁定 Rust `1.85.0`；scoped local verifier；以及 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。因未提供 unified diff input，verifier 为 `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, or production claim was added. The long-term goal remains
active; the next implementation must begin with a new bilingual Necessity Record. Deferred
external release evidence remains outside the current local queue.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、
authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。长期目标保持 active；下一项实现必须先新增双语 Necessity Record。
延期的外部 release evidence 继续不进入当前本地队列。

### 2026-08-01 Private Persisted Context Diff Error Redaction / 2026-08-01 私有持久化 Context Diff 错误脱敏

This bounded local security receipt advances Criterion 4 but does not close it or the repository
convergence goal. The private persisted Context diff Web adapter now retains HTTP status and
structured error code while replacing structured or malformed upstream messages with the stable
bilingual unavailable message. The red regression proved the old adapter could expose a synthetic
`sql://internal-db?token=secret diagnostic payload`; the green regression proves it is excluded.

本次有界 local security 回执推进条件 4，但不关闭条件 4 或仓库收束目标。私有 persisted Context diff Web adapter 现保留 HTTP status 与 structured error code，
同时将 structured 或 malformed upstream message 替换为稳定的双语 unavailable message。红回归证明旧 adapter 可能暴露 synthetic
`sql://internal-db?token=secret diagnostic payload`；绿回归证明该内容已被排除。

Fresh evidence / 新鲜证据：red focused adapter `4 passed, 2 failed`; green focused adapter `6 passed`; `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `278`, TypeScript/lint and production build; `cargo fmt --all -- --check`; workspace tests with storage `212 passed, 39 ignored`; strict offline Clippy; locked Rust `1.85.0`; scoped local verifier; and `GRAPH_DIFF_IMPL_COUNT=1`. The verifier remains `overall=unobserved` because no unified diff input was supplied.

新鲜证据：红 focused adapter `4 passed, 2 failed`；绿 focused adapter `6 passed`；`pnpm check:web` 通过（public SDK `15`、local SDK `135`、Web `278`、TypeScript/lint 与 production build）；`cargo fmt --all -- --check`；workspace tests（storage `212 passed, 39 ignored`）；strict offline Clippy；锁定 Rust `1.85.0`；scoped local verifier；以及 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。因未提供 unified diff input，verifier 仍为 `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, or production claim was added. The long-term goal remains
active; the next implementation must start with a new bilingual Necessity Record. Deferred external
release evidence is outside the current local queue.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、
authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。长期目标保持 active；下一项实现必须先新增双语 Necessity Record。
延期的外部 release evidence 不属于当前本地队列。

### 2026-08-01 Local Workflow Read and Persisted Metadata Receipt / 2026-08-01 本地 Workflow Read 与持久化 Metadata 回执

This bounded local receipt advances Criteria 1, 2, 4, 6, and 9 but closes none of them. The
guarded lifecycle writer preserves exact resulting/inherited `ContextMetadata` in immutable
semantic snapshots, including later non-metadata successor inheritance and idempotent replay. The
private Workflow binding read has a complete BFF -> data -> presenter -> screen path at the selected
exact Context commit, with frozen redacted DTOs, deterministic ordering, shared bilingual state UI,
and no page-level business algorithm.

本次有界 local 回执推进条件 1、2、4、6、9，但不关闭任何条件。guarded lifecycle writer 将 exact resulting/inherited `ContextMetadata` 保存在不可变 semantic snapshot 中，覆盖后续 non-metadata successor 继承与幂等 replay。私有 Workflow binding read 在选定 exact Context commit 上形成完整 BFF -> data -> presenter -> screen path，使用 frozen 脱敏 DTO、确定性排序、共享双语状态 UI，页面不承载业务算法。

Fresh evidence / 新鲜证据：storage metadata `1 passed`; API writer-to-review metadata `1 passed`; workspace Rust passed with storage `212 passed, 39 ignored`; `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0`; BFF route `10 passed`; Workflow binding focused `17 passed`; and `pnpm check:web` (`15/135/280 + production build`) passed. The fixture verifier test passed with a deterministic safe diff and `overall=passed`. The live verifier's source/graph/safe-DTO/protected-route checks passed with `graph_diff_application=passed count=1`; live `overall=unobserved` only because no live unified diff input was supplied.

新鲜证据：storage metadata `1 passed`；API writer-to-review metadata `1 passed`；workspace Rust 通过且 storage 为 `212 passed, 39 ignored`；`cargo fmt --all -- --check`；严格 offline Clippy；锁定 Rust `1.85.0`；BFF route `10 passed`；Workflow binding focused `17 passed`；以及 `pnpm check:web`（`15/135/280 + production build`）通过。fixture verifier test 使用确定性安全 diff 并报告 `overall=passed`。live verifier 的 source/graph/safe-DTO/protected-route checks 通过，`graph_diff_application=passed count=1`；因未提供 live unified diff input，live `overall=unobserved`。

No public REST/OpenAPI/public SDK write, Web mutation, migration, provider, secret access,
operator transport, second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser,
visual smoke, Git change-set, remote CI, operator rehearsal, release, or production claim is made.
Those facts remain `unobserved` or `deferred`; deferred external evidence is not a local
development blocker. A fresh bilingual Necessity Record is required before the next implementation,
and the long-term goal remains active.

未新增 public REST/OpenAPI/public SDK write、Web mutation、migration、provider、secret access、operator transport、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。上述事实继续为 `unobserved` 或 `deferred`；延期外部证据不构成本地开发阻塞。下一项实现前必须新增双语 Necessity Record，长期目标保持 active。

## 2026-08-01 Private Cross-Domain Capability Registry Snapshot / 2026-08-01 私有跨域 Capability Registry Snapshot

This bounded local contract increment advances Criteria 1, 5, 7, 8, and 9 without closing any of
them or the active long-term goal. The reusable MCP core now provides an immutable
`CapabilityRegistrySnapshotV1` with an explicit schema version, canonical secret-free availability
projection, and deterministic SHA-256 fingerprint over the validated manifest set. A cross-domain
API fixture consumes one snapshot through the existing Workflow and Knowledge bridges, preserving
exact version/kind checks, deterministic ordering, and redacted citation replay. The snapshot is a
local Rust contract; it is not a plugin loader, execution producer, or public transport.

本次有界 local contract 增量推进条件 1、5、7、8 与 9，但不关闭其中任何条件或 active long-term goal。可复用 MCP core 现提供不可变的
`CapabilityRegistrySnapshotV1`，包含显式 schema version、canonical secret-free availability projection，以及对已校验 manifest set 计算的确定性 SHA-256 fingerprint。
跨域 API fixture 使用同一 snapshot 通过既有 Workflow 与 Knowledge bridge，保持精确 version/kind check、确定性排序与脱敏 citation replay。该 snapshot 只是本地 Rust contract，
不是 plugin loader、execution producer 或 public transport。

Fresh red/green and local evidence / 新鲜红绿与本地证据：

- mcp snapshot red compile failed on the missing type, schema constant, and `CapabilityRegistry::snapshot()`; green snapshot contract `2 passed`.
- Cross-domain Workflow + Knowledge composition `1 passed`.
- Knowledge/Memory replay identity red regression then `14 passed`; full Knowledge package tests passed.
- CLI/Desktop identity-drift regressions red then adapter `8`, CLI `5`, and Desktop staging `5 passed`.
- Web raw-field parser regression red then full `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `282`, TypeScript/lint, and production build.
- Workflow/Plugin, Diff/versioning, and Evaluation bounded audits found no defensible defect and made no speculative edits.

- mcp snapshot 红 compile 在缺少 type、schema constant 与 `CapabilityRegistry::snapshot()` 时失败；绿 snapshot contract `2 passed`。
- 跨域 Workflow + Knowledge composition `1 passed`。
- Knowledge/Memory replay identity 红 regression 后 `14 passed`；Knowledge package 全量通过。
- CLI/Desktop identity-drift 红 regression 后 adapter `8`、CLI `5`、Desktop staging `5 passed`。
- Web raw-field parser 红 regression 后 `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `282`、TypeScript/lint 与 production build。
- Workflow/Plugin、Diff/versioning 与 Evaluation 有界审查未发现可 defensibly 修复的缺陷，也未做 speculative edit。

The first Knowledge/Memory, Evaluation, CLI/Desktop, and Web Luna workers failed with transport
`502 Bad Gateway`; they were closed and immediately reassigned to replacement `gpt-5.6-luna`
workers. The replacement outputs above are product evidence; the transport failures are execution
provenance only. The final local checks are now observed passed: `cargo fmt --all -- --check`,
workspace Rust with storage `212 passed, 39 ignored`, strict offline Clippy, locked Rust `1.85.0`,
the scoped local verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. The live verifier remains
`overall=unobserved` only because no unified diff input was supplied.

首轮 Knowledge/Memory、Evaluation、CLI/Desktop 与 Web Luna worker 因 transport `502 Bad Gateway` 失败，已关闭并立即由 replacement `gpt-5.6-luna` worker 接管。上述 replacement 输出才是产品证据；transport failure 仅是执行 provenance。
本波次最终本地检查现已观测通过：`cargo fmt --all -- --check`、workspace Rust（storage `212 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0`、范围化 local verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。live verifier 仅因未提供 unified diff input 而保持 `overall=unobserved`。

No public REST/OpenAPI/public SDK write, Web mutation, operator transport, provider/network call,
migration, secret access, second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated
browser/visual smoke, Git change-set, remote CI, operator rehearsal, release, or production claim
was added. Those facts remain `unobserved` or `deferred`; the long-term goal remains active and the
next implementation requires a new bilingual Necessity Record.

未新增 public REST/OpenAPI/public SDK write、Web mutation、operator transport、provider/network call、migration、secret access、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。
上述事实继续为 `unobserved` 或 `deferred`；长期目标保持 active，下一项实现必须新增双语 Necessity Record。

## 2026-08-01 Private Workflow Execution Producer and Repository / 2026-08-01 私有 Workflow 执行 Producer 与 Repository

This bounded local evidence increment advances Criterion 1 without closing it or the active
long-term goal. The storage boundary now provides an immutable exact-scope execution-status
repository, a deterministic in-memory adapter, and a provider-free producer service that validates
root/replay logs into `WorkflowExecutionStatusProjectionV1` before writing. Same-scope identical
replay returns `Replayed`; conflicting immutable reuse is rejected. The API adapter consumes only
the redacted projection, while the application default remains unavailable until explicit
repository injection.

本次有界 local evidence 增量推进条件 1，但不关闭条件 1 或 active long-term goal。storage boundary 现提供不可变 exact-scope execution-status repository、确定性 in-memory adapter 与 provider-free producer service，在写入前将 root/replay log 校验为
`WorkflowExecutionStatusProjectionV1`。同 scope 相同 replay 返回 `Replayed`；冲突的不可变复用被拒绝。API adapter 只消费脱敏 projection，应用默认在显式 repository injection 前继续 unavailable。

Fresh evidence / 新鲜证据：storage producer/repository `3 passed`; API adapter `6 passed`; workspace Rust passed with storage `215 passed, 39 ignored`; format, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web` (`15/135/284 + production build`), scoped verifier checks, and `GRAPH_DIFF_IMPL_COUNT=1` passed. Verifier `overall=unobserved` reflects intentionally absent unified diff input.

新鲜证据：storage producer/repository `3 passed`；API adapter `6 passed`；workspace Rust 通过且 storage 为 `215 passed, 39 ignored`；format、strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`（`15/135/284 + production build`）、scoped verifier checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。verifier `overall=unobserved` 仅表示有意未提供 unified diff input。

No execution start route, public REST/OpenAPI/public SDK write, Web mutation, scheduler,
migration, provider, secret, operator transport, second GraphDiff calculator, or production claim
was added. PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git, remote CI, operator
rehearsal, release, and production remain `unobserved` or `deferred`; the long-term goal remains
active and the next implementation requires a new bilingual Necessity Record.

未新增 execution start route、public REST/OpenAPI/public SDK write、Web mutation、scheduler、migration、provider、secret、operator transport、第二个 GraphDiff calculator 或 production 声明。PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；长期目标保持 active，下一项实现必须新增双语 Necessity Record。

## 2026-08-01 Workflow Execution Status PostgreSQL Receipt / 2026-08-01 Workflow 执行状态 PostgreSQL 回执

Criteria 1 and 8 receive a fresh local persistence receipt, but remain open. The private
Workflow execution projection repository now has an append-only PostgreSQL adapter and migration
0024. A fresh temporary PostgreSQL 16.14 cluster passed the focused integration test (`1
passed`) for migration/seed, `Created`, identical `Replayed`, cross-connection read, and immutable
conflict. Static adapter and migration contracts passed (`3 passed` each). The first runtime run
found a duplicate constraint in migrations 0016/0022; removing the duplicate from 0022 and adding
a one-owner static regression fixed the root cause. No public write or production readiness is
claimed; remote CI/operator/release/production evidence remains unobserved or deferred.

条件 1 与 8 获得新鲜 local persistence receipt，但仍保持开放。私有 Workflow execution projection repository
现拥有 append-only PostgreSQL adapter 与 `0024` migration。临时 PostgreSQL 16.14 fresh cluster 通过 focused
integration test（`1 passed`），覆盖 migration/seed、`Created`、相同 `Replayed`、跨连接读取与 immutable conflict。
adapter 与 migration static contract 各自 `3 passed`。首次 runtime 发现 migration 0016/0022 重复 constraint；从
0022 删除重复项并增加 one-owner static regression 后根因修复。没有宣称 public write 或 production readiness；
remote CI/operator/release/production evidence 继续为 unobserved 或 deferred。

Final local matrix / 最终本地矩阵：workspace Rust storage `218 passed, 39 ignored`、format、strict offline
Clippy、locked Rust `1.85.0` check、`pnpm check:web` (`15/135/284 + production build`) 全部通过；local
verifier 的 `graph_diff_application=passed count=1` 与 `safe_local_dto_fields=passed` 通过，`overall=unobserved`
仅表示没有提供 unified diff input。

## 2026-08-01 Private Knowledge/Memory PostgreSQL Runtime Receipt / 2026-08-01 私有 Knowledge/Memory PostgreSQL 运行时回执

This bounded local receipt advances Criteria 1 and 8 without closing them or the active
long-term goal. The existing private Knowledge/Memory projection repository now has a fresh
loopback PostgreSQL runtime test. It applies the existing migration and deterministic seed, proves
immutable `Created`/`Replayed`, a second-connection read, stored JSON redaction, exact project,
Context, and commit scope enforcement, immutable conflict, and malformed stored projection
rejection. The fixture is opt-in, ignored without an explicit URL, and loopback-only.

本有界 local receipt 推进条件 1 与 8，但不关闭它们或 active long-term goal。既有 private Knowledge/Memory projection repository 现拥有新鲜 loopback PostgreSQL runtime test。
它应用既有 migration 与确定性 seed，证明 immutable `Created`/`Replayed`、第二连接读取、stored JSON redaction、exact project、Context 与 commit scope enforcement、immutable conflict 与 malformed stored projection rejection。fixture 是 opt-in、无显式 URL 时 ignored，且仅允许 loopback。

Fresh verification / 新鲜验证：

- Fresh temporary PostgreSQL 16 cluster: `postgres_projection_runtime_receipt_covers_exact_immutable_redacted_replay` -> `1 passed`.
- No-URL focused run: `1 ignored` by design.
- Full local Rust/storage: `218 passed, 39 ignored`; `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0`; `pnpm check:web` Web `284/284` plus production build; local contract verifier fixture passed.

新鲜验证：

- fresh temporary PostgreSQL 16 cluster：`postgres_projection_runtime_receipt_covers_exact_immutable_redacted_replay` -> `1 passed`。
- 无 URL focused run：按设计为 `1 ignored`。
- 完整 local Rust/storage：`218 passed, 39 ignored`；`cargo fmt --all -- --check`；strict offline Clippy；锁定 Rust `1.85.0`；`pnpm check:web` Web `284/284` 与 production build；local contract verifier fixture 通过。

The temporary database process was stopped. Recursive removal of the verified temporary data
directory was rejected by local tool policy, so filesystem cleanup remains `unobserved`; this
does not alter the test receipt and no database process remains running. The evidence is local and
non-production only. Remote CI, operator rehearsal, release, production, authenticated browser,
visual smoke, and Git evidence remain `unobserved` or `deferred`; no public write or readiness
claim is made and the long-term goal remains active.

临时 database process 已停止。工具策略拒绝删除已核验的临时 data directory，因此 filesystem cleanup 继续为 `unobserved`；这不改变 test receipt，且没有数据库进程继续运行。本证据仅限本地非生产环境。remote CI、operator rehearsal、release、production、authenticated browser、visual smoke 与 Git evidence 继续为 `unobserved` 或 `deferred`；不声称 public write 或 readiness，长期目标保持 active。

## 2026-08-01 Private Context Lifecycle Atomic Read / 2026-08-01 私有 Context 生命周期原子读取

This receipt advances Criteria 1 and 2 without closing them. `ContextLifecycleReadRepository` now
owns the exact-commit aggregate boundary for graph snapshot, replay state, component inventory,
immutable content witnesses, and derived project/context/commit scope. Memory uses one read guard;
PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction. The service fail-closes returned
Context/commit scope drift and preserves the existing API/SDK/Web response shape.

本回执推进条件 1 与 2，但不关闭它们。`ContextLifecycleReadRepository` 现拥有 graph snapshot、replay state、
component inventory、immutable content witness 与 project/context/commit scope 的 exact-commit aggregate
boundary。Memory 使用一个 read guard；PostgreSQL 使用一个 `REPEATABLE READ READ ONLY` transaction。service
对返回的 Context/commit scope drift fail closed，并保持既有 API/SDK/Web response shape。

Fresh local evidence / 新鲜本地证据：storage lifecycle `15 passed`; PostgreSQL SQL contract `1 passed`;
API lifecycle `5 passed`; workspace Rust storage `219 passed, 39 ignored`; `cargo fmt --all -- --check`;
strict offline Clippy; `cargo +1.85.0 check --workspace --locked --offline`; `pnpm check:web`
(`15/135/284 + production build`); scoped local verifier; and `GRAPH_DIFF_IMPL_COUNT=1`.

新鲜本地证据：storage lifecycle `15 passed`；PostgreSQL SQL contract `1 passed`；API lifecycle `5 passed`；
workspace Rust storage `219 passed, 39 ignored`；`cargo fmt --all -- --check`；strict offline Clippy；
`cargo +1.85.0 check --workspace --locked --offline`；`pnpm check:web`（`15/135/284 + production build`）；
范围化 local verifier；以及 `GRAPH_DIFF_IMPL_COUNT=1`。

No new route, write, migration, provider, raw-content surface, public REST/OpenAPI/public SDK write, Web
mutation, operator transport, or second GraphDiff calculator was added. `CONTEXTLAB_TEST_DATABASE_URL` was
not available for this new aggregate, so PostgreSQL runtime atomicity is `unobserved`, not passed. Docker,
browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

未新增 route、write、migration、provider、raw-content surface、public REST/OpenAPI/public SDK write、Web mutation、
operator transport 或第二个 GraphDiff calculator。本 aggregate 没有可用的 `CONTEXTLAB_TEST_DATABASE_URL`，因此
PostgreSQL runtime atomicity 为 `unobserved`，不是通过。Docker、browser、Git、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`。
## 2026-08-02 Criterion 1 Local Read Inspector Receipt / 2026-08-02 条件 1 本地只读检查器回执

This local receipt advances Criterion 1 only. The private Context Lifecycle Read Inspector consumes
the existing protected exact-commit lifecycle DTO through shared Web `data -> presenter -> screen`
layers and shared UI primitives. It exposes no mutation and does not calculate graph diffs in the Web layer.

本地回执仅推进条件 1。私有 Context 生命周期只读检查器通过共享 Web `data -> presenter -> screen` 层与共享 UI
primitive 消费既有受保护 exact-commit lifecycle DTO。它不暴露 mutation，也不在 Web 层计算图差异。

Fresh evidence / 新鲜证据：focused lifecycle tests `23 passed`; Web lint and full Web tests `290 passed`;
`pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `290`, and production build; Rust format,
workspace tests `220 passed, 41 ignored`, strict offline Clippy, and locked Rust `1.85.0` check passed.

新鲜证据：lifecycle 聚焦测试 `23 passed`；Web lint 与完整 Web tests `290 passed`；`pnpm check:web` 通过，public SDK
`15`、local SDK `135`、Web `290` 与 production build 均通过；Rust format、workspace tests `220 passed, 41 ignored`、
strict offline Clippy 与锁定 Rust `1.85.0` check 通过。

This is local static/rendered-test evidence, not authenticated browser, visual regression, PostgreSQL
runtime, remote CI, operator, release, or production evidence. Criterion 1 and all other criteria remain open.

本回执是本地静态/渲染测试证据，不是 authenticated browser、visual regression、PostgreSQL runtime、remote CI、operator、
release 或 production 证据。条件 1 与其余条件继续开放。
## 2026-08-02 Criterion 1 Lifecycle Read Transport Receipt / 2026-08-02 条件 1 生命周期读取传输回执

This local receipt advances Criterion 1 only. It fixes and tests the protected read transport's
explicit no-store cache behavior, exact `(Context, commit)` scope, Bearer-only request-memory
credentials, cookie omission, private response headers, 503/unavailable mapping, and accessible
bilingual status presentation. It does not close Criterion 1 or the repository.

本地回执仅推进条件 1。它修复并测试受保护读取传输的显式 no-store cache 行为、精确 `(Context, commit)` scope、
仅 Bearer 的请求内存凭据、cookie omission、private response header、503/unavailable mapping 与可访问双语 status
呈现。不关闭条件 1 或仓库。

Fresh evidence / 新鲜证据：focused lifecycle data/proxy/presenter/inspector `41 passed`; full Web `294 passed`;
`pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `294`, production build; Rust format, workspace
tests `220 passed, 41 ignored`, strict offline Clippy, locked Rust `1.85.0`, and local contract verifier passed.

新鲜证据：lifecycle data/proxy/presenter/inspector 聚焦测试 `41 passed`；完整 Web `294 passed`；`pnpm check:web` 通过，
public SDK `15`、local SDK `135`、Web `294`、production build 均通过；Rust format、workspace tests `220 passed, 41 ignored`、
strict offline Clippy、锁定 Rust `1.85.0` 与 local contract verifier 通过。

This remains local contract/rendered-test evidence. PostgreSQL runtime, authenticated browser, visual regression,
Git binding, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or `deferred`; all
criteria and the long-term goal remain open.

本回执仍是本地 contract/rendered-test evidence。PostgreSQL runtime、authenticated browser、visual regression、Git binding、
remote CI、operator rehearsal、release 与 production evidence 继续为 `unobserved` 或 `deferred`；全部条件与长期目标继续开放。

## 2026-08-02 Private Versioned Context Graph Pair Witness / 2026-08-02 私有版本化 Context Graph 成对见证

This bounded local receipt advances Criterion 2 without closing it. The diff domain now creates an immutable
server-owned ordered pair witness, and the protected local graph-diff response exposes the exact project, Context,
baseline commit, and revised commit identity under `pair_witness`. The local SDK enforces the exact response shape and
fails closed on missing, unknown, mixed-scope, schema-drifting, or self-pair witnesses. Web consumes the DTO through
the existing data adapter and does not calculate a second diff. `GraphDiff::between` remains the sole graph-diff
calculator.

本有界本地回执推进条件 2 但不关闭条件 2。diff domain 现生成不可变的 server-owned 有序 pair witness，受保护 local
graph-diff response 在 `pair_witness` 下暴露精确 project、Context、baseline commit 与 revised commit identity。local SDK
强制校验 exact response shape，并对缺失、未知字段、混合 scope、schema 漂移或 self-pair witness fail closed。Web 经由既有
data adapter 消费 DTO，不计算第二套 diff；`GraphDiff::between` 仍是唯一图差异 calculator。

Fresh evidence / 新鲜证据：diff-engine `40 passed`; API scope contract `3 passed`; workspace Rust `220 passed, 41 ignored`;
format; strict offline Clippy; locked Rust `1.85.0`; `pnpm check:web` public SDK `15`, local SDK `137`, Web `294`, production
build; fixture verifier passed; and live verifier source/graph/safe-DTO/protected-route checks passed with
`graph_diff_application=passed count=1`, while `overall=unobserved` remained due to intentionally absent unified diff input.

新鲜证据：diff-engine `40 passed`；API scope contract `3 passed`；workspace Rust `220 passed, 41 ignored`；format、strict offline
Clippy、锁定 Rust；`pnpm check:web` 的 public SDK `15`、local SDK `137`、Web `294` 与 production build；fixture verifier 通过；
live verifier 的 source/graph/safe-DTO/protected-route checks 与 `graph_diff_application=passed count=1` 通过，但由于有意未提供
unified diff input，`overall=unobserved`。

No public write, public REST/OpenAPI/public SDK write, migration, provider, secret, operator transport, second
GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, or production claim is made. These remain `unobserved` or `deferred`; the long-term goal remains active and
the next increment requires a new bilingual Necessity Record.

未新增 public write、public REST/OpenAPI/public SDK write、migration、provider、secret、operator transport、第二个 GraphDiff
calculator、Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 或 production
声明。上述事实继续为 `unobserved` 或 `deferred`；长期目标保持 active，下一项增量必须新增双语 Necessity Record。

## 2026-08-02 Pair Witness Contract Boundary Repair / 2026-08-02 成对见证契约边界修复

This follow-up receipt records the minimum integration repair discovered by independent review of the
pair-witness slice. The local SDK now accepts the API's actual UTC RFC3339 `Z` and `+00:00` forms while
rejecting non-UTC offsets and invalid dates; it preserves the server's safe
`commit_graph_diff_unavailable` and `storage_scope_unavailable` codes; and it rejects one node identity
appearing in multiple GraphDiff node categories. No graph calculation moved to the SDK or Web; the Rust
`GraphDiff::between` implementation remains the sole graph-diff calculator.

本跟进回执记录 pair-witness slice 经独立审查后发现的最小集成修复。local SDK 现接受 API 实际使用的 UTC RFC3339
`Z` 与 `+00:00` 形式，同时拒绝非 UTC offset 与非法日期；保留 server 的安全错误码
`commit_graph_diff_unavailable` 与 `storage_scope_unavailable`；并拒绝同一 node identity 同时出现在多个 GraphDiff
node category。没有把图计算移到 SDK 或 Web；Rust `GraphDiff::between` 仍是唯一 graph-diff calculator。

Fresh evidence / 新鲜证据：`pnpm --filter @contextlab/local-sdk lint` passed; local SDK `141/141` passed;
diff-engine identity witness `2 passed`; API scope contract `3 passed`; full workspace Rust `220 passed, 41 ignored`;
format; strict offline Clippy; locked Rust `1.85.0`; `pnpm check:web` public SDK `15`, local SDK `141`, Web `294`,
production build; and fixture contract verifier passed. The scope verifier retained
`graph_diff_application=passed count=1` and `overall=unobserved` without unified diff input.

新鲜证据：`pnpm --filter @contextlab/local-sdk lint` 通过；local SDK `141/141` 通过；diff-engine identity witness `2 passed`；
API scope contract `3 passed`；完整 workspace Rust `220 passed, 41 ignored`；format、strict offline Clippy、锁定 Rust `1.85.0`；
`pnpm check:web` 的 public SDK `15`、local SDK `141`、Web `294` 与 production build 通过；fixture contract verifier 通过。
范围 verifier 在未提供 unified diff input 时保留 `graph_diff_application=passed count=1` 与 `overall=unobserved`。

This remains local contract evidence only. PostgreSQL runtime, authenticated browser/visual smoke, Git binding, Docker,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; Criteria 2 and the remaining
criteria stay open. The next admitted local increment is a new bilingual Necessity Record for a server-owned benchmark
decision pair witness; it must not add public write or a second diff calculator.

本回执仍仅是 local contract evidence。PostgreSQL runtime、authenticated browser/visual smoke、Git binding、Docker、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；条件 2 与其余条件继续开放。下一项准入的本地增量
是为 server-owned benchmark decision pair witness 新建双语 Necessity Record；不得新增 public write 或第二个 diff calculator。

## 2026-08-02 Criterion 3 Direct Decision-Diff Multi-Dataset Receipt / 2026-08-02 条件 3 直接 Decision-Diff 多 Dataset 回执

This receipt advances Criterion 3's persisted/queryable/comparable evaluation evidence but does not close Criterion 3. The existing protected API integration test now exercises direct decision diff against the same sealed two-dataset/four-case fixture as the breadth path and verifies exact pair scope, `passed -> regressed`, modified accuracy, four samples and four required samples on both sides, deterministic replay, and recursive redaction. Dataset IDs and raw benchmark payloads remain outside the response by design; no public REST/OpenAPI/SDK surface was added.

本回执推进条件 3 的 persisted/queryable/comparable evaluation evidence，但不关闭条件 3。现有受保护 API integration test 现针对与 breadth path 相同的 two-dataset/four-case sealed fixture 直接执行 decision diff，并验证 exact pair scope、`passed -> regressed`、modified accuracy、两侧四个 sample 与四个 required sample、确定性 replay 及递归脱敏。Dataset ID 与 raw benchmark payload 按设计不进入 response；未新增 public REST/OpenAPI/SDK surface。

Fresh local verification / 新鲜本地验证：`cargo test -p contextlab-api --test benchmark_breadth --offline -- --nocapture` (`1 passed`), `cargo test -p contextlab-api --lib local_benchmark_decision_diff --offline -- --nocapture` (`2 passed`), `cargo test -p contextlab-storage --test benchmark_breadth --offline -- --nocapture` (`1 passed`), `cargo test -p contextlab-evaluation --test benchmark_decision_diff --offline -- --nocapture` (`3 passed`), and `cargo fmt --all -- --check` passed at `2026-08-02T03:45:26.6799857+08:00`.

新鲜本地验证：上述 API breadth `1 passed`、API direct diff `2 passed`、storage breadth `1 passed`、evaluation diff `3 passed`，以及 `cargo fmt --all -- --check` 于 `2026-08-02T03:45:26.6799857+08:00` 通过。

This is a local evidence receipt, not a criterion closeout. Full workspace/Web verification remains required separately; PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`. The long-term goal remains active.

这是本地 evidence receipt，不是条件收束。full workspace/Web verification 仍需单独完成；PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active。
## 2026-08-02 Criterion 2/3 Private Benchmark Decision-Pair Witness Receipt / 2026-08-02 条件 2/3 私有 Benchmark Decision-Pair Witness 回执

This receipt advances Criterion 2's exact version identity and Criterion 3's queryable/comparable benchmark workflow, but does not close either criterion. The protected decision-bound workspace carries optional server-owned `decision_pair_witness` with `schema_version=1`, exact project/Context, and ordered baseline/revised commit+decision identities; cohort-bound and single-scope reads omit it. The local SDK parser/client fail closed on missing, unknown, malformed, wrong-version, mixed-scope, mismatched, or self-pair values. Web uses the SDK parser and shared scope grid; no policy, evaluation Diff, or GraphDiff is recomputed outside the owning Rust boundary.

本回执推进条件 2 的精确版本 identity 与条件 3 的可查询/可比较 benchmark workflow，但不关闭任一条件。受保护 decision-bound workspace 携带可选的 server-owned `decision_pair_witness`，包含 `schema_version=1`、精确 project/Context 以及有序 baseline/revised commit+decision identity；cohort-bound 与 single-scope read 不返回它。local SDK parser/client 对缺失、未知、格式错误、错误版本、混合 scope、不匹配或 self-pair fail closed。Web 使用 SDK parser 与共享 scope grid；没有在 Rust owner boundary 之外重算 policy、evaluation Diff 或 GraphDiff。

Fresh evidence / 新鲜证据：API breadth `1 passed`; API workspace unit `10 passed`; local SDK `148 passed` and lint; Web focused `20 passed`; full Web `298 passed`; `pnpm check:web` public SDK `15`, local SDK `148`, Web `298`, production build passed; Rust workspace `220 passed, 41 ignored`, fmt, strict offline Clippy, locked Rust `1.85.0` check, and `verify-local-contracts.test.ps1` fixture tests passed.

新鲜证据：API breadth `1 passed`；API workspace unit `10 passed`；local SDK `148 passed` 且 lint 通过；Web focused `20 passed`；完整 Web `298 passed`；`pnpm check:web` 的 public SDK `15`、local SDK `148`、Web `298` 与 production build 通过；Rust workspace `220 passed, 41 ignored`、fmt、strict offline Clippy、锁定 Rust `1.85.0` check 与 `verify-local-contracts.test.ps1` fixture tests 通过。

Migration/API integrity verification is recorded as local evidence only; it validates the local migration and API contract wiring and does not establish PostgreSQL runtime or external release evidence.

Migration/API integrity verification 仅记录为 local evidence；它只验证本地 migration 与 API contract wiring，不构成 PostgreSQL runtime 或 external release evidence。

This remains local non-production contract evidence. PostgreSQL runtime tests requiring `CONTEXTLAB_TEST_DATABASE_URL` are ignored/unobserved; Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, production, and public protected-write readiness remain `unobserved` or `deferred`. The completion criteria and long-term goal remain open.

本回执仍是本地非生产 contract evidence。需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime tests 为 `ignored/unobserved`；Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release、production 与 public protected-write readiness 继续为 `unobserved` 或 `deferred`。全部 completion criteria 与长期目标继续开放。

## 2026-08-02 Private Lifecycle-Witness GraphDiff Receipt / 2026-08-02 私有生命周期见证 GraphDiff 回执

This bounded local receipt advances Criteria 2 and 4 but closes neither. Before comparing the
requested version pair, the private graph-diff composition now loads and validates exact
`ContextLifecycleReadFacts` for both commits, including Context/commit scope, component
content/provenance, graph nodes/edges, and replay state. Missing, mixed, or inconsistent lifecycle
witnesses fail closed; only validated graph snapshots reach the existing `GraphDiff::between` path.
No response shape, public write surface, or second diff calculator was added.

本有界本地回执推进条件 2 与 4，但不关闭任一条件。private graph-diff composition 现会在比较请求的版本 pair
前读取并校验两侧 exact `ContextLifecycleReadFacts`，覆盖 Context/commit scope、component content/provenance、
graph nodes/edges 与 replay state。缺失、混合或不一致的 lifecycle witness 均 fail closed；只有已验证的 graph
snapshot 才进入既有 `GraphDiff::between` 路径。未改变 response shape、public write surface，也未新增第二个 diff calculator。

Fresh local verification / 新鲜本地验证：storage GraphDiff review tests `4 passed`; protected API GraphDiff
focused tests `13 passed`; API integration contract `3 passed`; workspace Rust `221 passed, 41 ignored`;
`cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0` check; `pnpm check:web`
with public SDK `15`, local SDK `148`, Web `298`, and production build; and
`tests/contract/verify-local-contracts.test.ps1` passed.

新鲜本地验证：storage GraphDiff review tests `4 passed`；protected API GraphDiff focused tests `13 passed`；API
integration contract `3 passed`；workspace Rust `221 passed, 41 ignored`；`cargo fmt --all -- --check`；strict offline
Clippy；锁定 Rust `1.85.0` check；`pnpm check:web` 的 public SDK `15`、local SDK `148`、Web `298` 与 production
build；以及 `tests/contract/verify-local-contracts.test.ps1` 通过。

This is local non-production contract evidence only. PostgreSQL runtime remains `ignored/unobserved` without
`CONTEXTLAB_TEST_DATABASE_URL`; Docker, authenticated browser, visual smoke, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. The completion criteria and long-term goal remain open.

本回执仅是本地非生产 contract evidence。没有 `CONTEXTLAB_TEST_DATABASE_URL` 时 PostgreSQL runtime 继续为
`ignored/unobserved`；Docker、authenticated browser、visual smoke、Git、remote CI、operator rehearsal、release
与 production 继续为 `unobserved` 或 `deferred`。全部 completion criteria 与长期目标继续开放。

## 2026-08-02 Guarded Lifecycle to Versioned GraphDiff Integration / 2026-08-02 Guarded Lifecycle 到版本化 GraphDiff 集成

This test-only local receipt advances Criteria 2 and 4 without closing either. The storage
integration test drives the existing guarded lifecycle writer through Context initialization,
component creation, immutable content revision, a second component, and typed Uses add/remove. The
same in-memory repository feeds exact replay facts and graph snapshots into
`PersistedContextGraphDiffReviewService`; the resulting projection has exact source/target commit
scopes and the expected edge additions/removals, while missing and mixed scopes fail closed.

本 test-only 本地回执推进条件 2 与 4，但不关闭任一条件。storage integration test 通过既有 guarded lifecycle writer 执行
Context initialization、component creation、immutable content revision、第二个 component 以及 typed Uses 添加/移除。同一
in-memory repository 将 exact replay facts 与 graph snapshot 输入 `PersistedContextGraphDiffReviewService`；结果 projection
具有 exact source/target commit scope 与预期 edge addition/removal，missing 与 mixed scope 均 fail closed。

Fresh local evidence / 新鲜本地证据：integration `3 passed`; focused storage review `4 passed`; API scope contract `3 passed`;
workspace Rust `221 passed, 41 ignored`; format; strict offline Clippy; locked Rust `1.85.0`; `pnpm check:web` (`15/148/298` plus
production build); local contract fixture verifier; and one `impl GraphDiff` source. PostgreSQL runtime, Docker, authenticated
browser/visual smoke, Git, remote CI, operator rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred`.

新鲜本地证据：integration `3 passed`；focused storage review `4 passed`；API scope contract `3 passed`；workspace Rust `221 passed, 41
ignored`；format；strict offline Clippy；锁定 Rust `1.85.0`；`pnpm check:web`（`15/148/298` 与 production build）；local contract fixture
verifier；以及一个 `impl GraphDiff` source。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator
rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。

No public REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, operator transport, or second GraphDiff
calculator was added. Criteria 2 and 4 remain open, as do the other convergence conditions; the long-term goal remains active.

未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret access、operator transport 或第二个 GraphDiff
calculator。条件 2 与 4 以及其他收束条件继续开放；长期目标保持 active。

## 2026-08-02 Private Capability Snapshot and Memory Scope Hardening / 2026-08-02 私有能力快照与 Memory Scope 硬化

This local parallel receipt advances Criteria 1, 5, 6, and 7 but closes none of them or the active
long-term goal. Workflow/Plugin now resolves requirements from the immutable
`CapabilityRegistrySnapshotV1` through `snapshot_from_registry_snapshot`, preserving the existing
live-registry compatibility helper. The in-memory Context Graph repository now enforces exact
project/Context/commit membership before Knowledge/Memory projection persistence or reads, matching
the PostgreSQL composite foreign-key contract.

本地并行回执推进条件 1、5、6、7，但不关闭其中任何条件或 active long-term goal。Workflow/Plugin 现通过
`snapshot_from_registry_snapshot` 从 immutable `CapabilityRegistrySnapshotV1` 解析 requirement，同时保留既有 live-registry compatibility helper。
内存 Context Graph repository 现会在 Knowledge/Memory projection 持久化或读取前强制 exact project/Context/commit membership，
与 PostgreSQL composite foreign-key contract 对齐。

Fresh verification / 新鲜验证: Workflow bridge `8 passed`; MCP `10 passed`; Knowledge/Memory projection `3 passed`; embedding
`6 passed`; storage `222 passed, 41 ignored`; full workspace Rust; strict offline Clippy; locked Rust `1.85.0` check;
`cargo fmt --all -- --check`; `pnpm check:web` with public SDK `15`, Web `298`, and production build; local contract fixture
verifier; and `GRAPH_DIFF_IMPL_COUNT=1` all passed. PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`.

新鲜验证：Workflow bridge `8 passed`、MCP `10 passed`、Knowledge/Memory projection `3 passed`、embedding `6 passed`、storage
`222 passed, 41 ignored`、workspace Rust、strict offline Clippy、锁定 Rust `1.85.0` check、`cargo fmt --all -- --check`、`pnpm check:web`
（public SDK `15`、Web `298` 与 production build）、local contract fixture verifier 以及 `GRAPH_DIFF_IMPL_COUNT=1` 全部通过。PostgreSQL runtime、
Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

No public REST/OpenAPI/public SDK write, Web mutation, dynamic loading, provider, migration, secret,
operator transport, or second GraphDiff calculator was added. A benchmark worker dispatch error and
the CLI/Docs no-gap review are recorded as scheduling/audit evidence only, not product proof. The
long-term goal remains active; the next implementation requires a fresh bilingual Necessity Record.

未新增 public REST/OpenAPI/public SDK write、Web mutation、dynamic loading、provider、migration、secret、operator transport 或第二个
GraphDiff calculator。Benchmark worker 的调度错误与 CLI/Docs no-gap review 仅记录为调度/审计证据，不是产品通过证明。长期目标保持 active；
下一项 implementation 必须新增双语 Necessity Record。

## 2026-08-02 Private Commit-History Replay Receipt / 2026-08-02 私有提交历史回放回执

The local versioning boundary now has a fresh, scope-matched receipt for Criterion 2. The
read-only `CommitHistory` aggregate validates one Context, explicit branch heads, known commit
ancestry, deterministic replay order, and ancestry-only merge planning. Focused history tests
reported `8 passed`; full versioning tests reported `54 passed`. A narrow TextDiff newline
regression was also repaired and verified with `13 passed`; `GraphDiff::between` remains the sole
graph-diff calculator.

本地 versioning boundary 现取得与条件 2 范围匹配的新鲜回执。只读 `CommitHistory` aggregate 校验单一 Context、显式
branch head、已知 commit ancestry、确定性 replay order 与仅基于 ancestry 的 merge planning。focused history tests 报告
`8 passed`；完整 versioning tests 报告 `54 passed`。同时修复并验证了窄范围 TextDiff newline regression（`13 passed`）；
`GraphDiff::between` 继续是唯一 graph-diff calculator。

The integrated local gates also passed: full workspace Rust (storage `222 passed, 41 ignored`),
format, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web` with Web `298 passed` and
production build, and the local contract verifier. These are local evidence only. PostgreSQL
runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release,
and production remain `ignored`, `unobserved`, or `deferred`; this receipt does not close Criterion
2, any other criterion, or the long-term goal.

集成本地 gate 也已通过：full workspace Rust（storage `222 passed, 41 ignored`）、format、strict offline Clippy、锁定 Rust
`1.85.0`、`pnpm check:web`（Web `298 passed` 与 production build）以及 local contract verifier。这些仅是本地 evidence。
PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production
继续为 `ignored`、`unobserved` 或 `deferred`；本回执不关闭条件 2、任何其他条件或长期目标。

No public write, public REST/OpenAPI/SDK write, Web mutation, operator transport, migration,
provider, secret access, or second graph-diff calculator was added. The next local increment still
requires a bilingual Necessity Record; the benchmark decision-pair witness plan is a candidate for
the next dependency audit, not completion evidence.

未新增 public write、public REST/OpenAPI/SDK write、Web mutation、operator transport、migration、provider、secret access 或
第二个 graph-diff calculator。下一项本地增量仍需双语 Necessity Record；benchmark decision-pair witness 计划是下一轮依赖
审计候选，不是完成证据。

## 2026-08-02 Private CommitHistory/Graph Review Composition / 2026-08-02 私有 CommitHistory/Graph 审阅组合

Criterion 2 now has a fresh local storage/application receipt that binds complete commit history
and explicit branch heads to the existing version-backed Context Graph review. The storage port
rehydrates typed commits through the existing decoder and delegates graph comparison through the
unchanged review service; API only composes the existing protected read route. Criterion 4 is
supported because graph review is now preceded by validated history scope, while
`GraphDiff::between` remains the sole calculator.

条件 2 现取得新鲜的本地 storage/application 回执，将完整 commit history 与显式 branch head 绑定到既有
version-backed Context Graph review。storage port 通过既有 decoder 重建 typed commit，并委托未改变的 review
service 计算；API 只组合既有 protected read route。条件 4 得到支持，因为 graph review 前已有 validated history
scope，且 `GraphDiff::between` 仍是唯一 calculator。

Observed evidence / 已观测证据: storage history composition `2 passed`; history-bound graph review `3 passed`;
independent storage history/snapshot contract `2 passed`; API focused graph-diff route `1 passed`; API crate
`223 passed`; workspace Rust `227 passed, 41 ignored`; format; strict offline Clippy; locked Rust `1.85.0`;
`pnpm check:web` with public SDK `15`, local SDK `148`, Web `298`, and production build; local contract fixture
verification; and `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

新鲜证据：storage history composition `2 passed`；history-bound graph review `3 passed`；独立 storage
history/snapshot contract `2 passed`；API focused graph-diff route `1 passed`；API crate `223 passed`；workspace Rust
`227 passed, 41 ignored`；format；strict offline Clippy；锁定 Rust `1.85.0`；`pnpm check:web`（public SDK `15`、local
SDK `148`、Web `298` 与 production build）；local contract fixture verification；以及 `GRAPH_DIFF_IMPL_COUNT=1`。
PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与
production 仍为 `unobserved` 或 `deferred`。

This receipt does not close Criterion 2, Criterion 4, or the repository. It adds no public REST/OpenAPI/SDK write,
Web mutation, migration, provider, secret access, operator transport, or second graph-diff calculator. Explicit
branch-head selection and atomic multi-port reads remain open follow-up criteria and require their own bilingual
Necessity Record.

本回执不关闭条件 2、条件 4 或仓库整体；未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、
secret access、operator transport 或第二个 graph-diff calculator。显式 branch-head selection 与 atomic multi-port
read 仍是开放 follow-up criteria，必须各自先有双语 Necessity Record。

## 2026-08-02 Branch-Head-Bound Review Scope Integrity / 2026-08-02 Branch-Head-Bound Review Scope 完整性

Necessity Record / 必要性记录: This increment directly supplies Criterion 2 replayable version
history and Criterion 4 Context Graph scope evidence. The gap was that branch-head review could
read history before validating its source scope, and could accept a history owned by another
Context. The minimum fix was storage-side early scope validation, explicit history Context matching,
and API mapping to the existing safe unavailable error. / 本增量直接补充条件 2 的可回放版本历史与条件 4 的 Context Graph
scope 证据。缺口是 branch-head review 可能在校验 source scope 前读取 history，也可能接受属于其他 Context 的 history。最小
修复是 storage 侧 early scope validation、显式 history Context matching，以及 API 映射到既有安全 unavailable error。

Observed evidence / 已观测证据: focused branch-head/history review `7 passed`; API graph-diff
focused `13 passed`; workspace Rust with API `223 passed` and storage `231 passed, 41 ignored`;
format; strict offline Clippy; locked Rust `1.85.0`; local contract verifier; Web/SDK `15` public
SDK, `148` local SDK, `298` Web tests, and production build; `GRAPH_DIFF_IMPL_COUNT=1` and ten
`GraphDiff::between` call sites. / 新鲜证据：focused branch-head/history review `7 passed`；API graph-diff focused `13 passed`；
workspace Rust（API `223 passed`、storage `231 passed, 41 ignored`）；format；strict offline Clippy；锁定 Rust `1.85.0`；
local contract verifier；Web/SDK（public SDK `15`、local SDK `148`、Web tests `298` 与 production build）；
`GRAPH_DIFF_IMPL_COUNT=1` 以及十个 `GraphDiff::between` call site。

This receipt advances Criteria 2 and 4 but closes neither. It adds no public write, public REST/
OpenAPI/SDK write method, Web mutation, migration, provider, operator transport, secret access, or
second diff calculator. PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, and production remain ignored, unobserved, or deferred. /
本回执推进条件 2 与条件 4，但不关闭任一条件。未新增 public write、public REST/OpenAPI/SDK write method、Web mutation、
migration、provider、operator transport、secret access 或第二个 diff calculator。PostgreSQL runtime、Docker、authenticated
browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 ignored、unobserved 或 deferred。

## 2026-08-02 Consistent Context Commit-History Read / 2026-08-02 一致的 Context 提交历史读取

This bounded increment advances Criteria 2 and 4 by making the private history read backend-owned:
Memory uses one read guard and PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction for commit
rows, parents, and branch heads before reusing `CommitHistory::try_from_parts`. Focused evidence is
storage history `3 passed`, Memory `1 passed`, SQL-shape `1 passed`, protected API graph-diff `7 passed`,
storage/API checks, and formatting. It does not close either criterion or the repository; broader fresh
workspace/Web/Clippy/MSRV and PostgreSQL runtime evidence is still required. / 本有界增量通过 backend-owned private history
read 推进条件 2 与 4：Memory 使用单一 read guard，PostgreSQL 使用单一 `REPEATABLE READ READ ONLY` transaction 读取 commit
row、parent 与 branch head，随后复用 `CommitHistory::try_from_parts`。focused evidence 为 storage history `3 passed`、
Memory `1 passed`、SQL-shape `1 passed`、protected API graph-diff `7 passed`、storage/API check 与 format。它不关闭任一
条件或仓库整体；workspace/Web/Clippy/MSRV 与 PostgreSQL runtime 仍需更广范围的新鲜证据。

No public REST/OpenAPI/SDK/Web mutation, migration, provider, secret, operator transport, or second
GraphDiff calculator was added. / 未新增 public REST/OpenAPI/SDK/Web mutation、migration、provider、secret、operator
transport 或第二个 GraphDiff calculator。

## 2026-08-02 Private Context Commit-History Consistent Read: Full Local Receipt / 2026-08-02 私有 Context 提交历史一致读取：全量本地回执

This receipt advances Criteria 2 and 4 and closes the previously named atomic-read gap for the
specific Context commit-history path. Memory uses one read guard and PostgreSQL uses one
`REPEATABLE READ READ ONLY` transaction for commit rows, parents, and branch heads; the validated
aggregate is still rebuilt by `CommitHistory::try_from_parts` before the existing private graph
review delegates to `GraphDiff::between`. This does not close either criterion or repository
convergence. Broader multi-aggregate consistency remains future work and is not implied by this receipt.

本回执推进条件 2 与 4，并收束此前针对特定 Context 提交历史 path 命名的 atomic-read 缺口。Memory 使用单一
read guard，PostgreSQL 使用单一 `REPEATABLE READ READ ONLY` transaction 读取 commit row、parent 与 branch head；
validated aggregate 仍由 `CommitHistory::try_from_parts` 重建，随后既有 private graph review 委托
`GraphDiff::between`。本回执不关闭任一条件或仓库收束；更广泛的 multi-aggregate consistency 仍是未来工作，不能由本回执推断完成。

Fresh evidence / 新鲜证据: workspace Rust API `223 passed`, storage `233 passed, 41 ignored`; format;
strict offline Clippy; locked Rust `1.85.0` check; `pnpm check:web` with public SDK `15`, local SDK `148 passed`,
Web `298 passed`, and production build; `tests/contract/verify-local-contracts.test.ps1`; and source
inspection with exactly one production `impl GraphDiff` and `10` `GraphDiff::between` call sites all passed.

新鲜证据：workspace Rust API `223 passed`、storage `233 passed, 41 ignored`；format；strict offline Clippy；
锁定 Rust `1.85.0` check；`pnpm check:web`（public SDK `15`、local SDK `148 passed`、Web `298 passed` 与 production build）；
`tests/contract/verify-local-contracts.test.ps1`；以及源码检查确认一个 production `impl GraphDiff` 和 `10` 个
`GraphDiff::between` 调用点，均通过。

PostgreSQL runtime requiring `CONTEXTLAB_TEST_DATABASE_URL`, Docker, authenticated browser/visual smoke,
Git, remote CI, operator rehearsal, release, production, and public protected-write readiness remain
`ignored`, `unobserved`, or `deferred`. No public REST/OpenAPI/SDK write, Web mutation, migration,
provider, secret access, operator transport, or second graph-diff calculator was added. The long-term
goal remains active; the next increment requires its own bilingual Necessity Record.

需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、
operator rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`。
未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret access、operator transport 或第二个
graph-diff calculator。长期目标保持 active；下一项增量必须拥有自己的双语 Necessity Record。

## 2026-08-02 Context History Construction-Path Hardening / 2026-08-02 Context History Construction-Path 硬化

The local contract audit found a bypass in the custom `AppState` builder: without an explicit
history repository, it used the generic split-port adapter and could therefore undermine the
backend-owned consistent-read guarantee. A red regression observed the bypass by successfully
rehydrating a complete history from a builder that had not received a history repository. The
minimal green repair makes the history dependency explicit in `WorkspaceRepositories`, installs a
fail-closed unavailable reader when absent, and supplies the shared Memory repository in the
versioned graph-diff fixture. This advances Criteria 2 and 4 but closes neither.

本地 contract audit 发现 custom `AppState` builder 存在 bypass：未显式提供 history repository 时，它使用 generic split-port
adapter，可能削弱 backend-owned consistent-read guarantee。红色 regression 通过一个未接收 history repository 的 builder
成功重建完整 history，观测到该 bypass。最小绿色修复让 history dependency 在 `WorkspaceRepositories` 中显式存在，缺失时安装
fail-closed unavailable reader，并在 versioned graph-diff fixture 中提供共享 Memory repository。本修复推进条件 2 与 4，
但不关闭任一条件。

Fresh local evidence / 新鲜本地证据: red then green construction-path regression; API library
`224 passed`; protected graph-diff focused `7 passed`; public-catalog versioned graph-diff `1 passed`;
workspace Rust API `224 passed`, storage `233 passed, 41 ignored`; format; strict offline Clippy;
locked Rust `1.85.0`; `pnpm check:web` public SDK `15`, local SDK `148 passed`, Web `298 passed`,
production build; local contract verifier;
and exactly one production `impl GraphDiff`. No public REST/OpenAPI/SDK write, Web mutation,
migration, provider, secret access, operator transport, or second calculator was added.

新鲜本地证据：construction-path regression 先红后绿；API library `224 passed`；protected graph-diff focused `7 passed`；
public-catalog versioned graph-diff `1 passed`；workspace Rust API `224 passed`、storage `233 passed, 41 ignored`；format；
strict offline Clippy；锁定 Rust `1.85.0`；`pnpm check:web` public SDK `15`、local SDK `148 passed`、Web `298 passed`、
production build；local contract verifier；以及唯一一个 production `impl GraphDiff`。未新增 public REST/OpenAPI/SDK write、Web mutation、
migration、provider、secret access、operator transport 或第二个 calculator。

PostgreSQL runtime requiring `CONTEXTLAB_TEST_DATABASE_URL`, Docker, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, production, and public protected-write readiness
remain `ignored`, `unobserved`, or `deferred`. The long-term goal remains active; the next increment
requires a fresh bilingual Necessity Record.

需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、
operator rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`。
长期目标保持 active；下一项增量必须拥有新的双语 Necessity Record。

## 2026-08-02 Private Context Graph Review Witness: Full Local Receipt / 2026-08-02 私有 Context Graph 审查见证：全量本地回执

Necessity Record / 必要性记录: This increment directly advances Criteria 1, 2, and 4 by binding
complete commit history, branch heads, and both immutable graph snapshots to one backend-owned
read boundary. The protected route consumes the witness repository and fails closed when it is
absent; test fixtures inject it explicitly. / 本增量直接推进条件 1、2、4：将完整 commit history、branch heads 与两份
immutable graph snapshot 绑定到一个 backend-owned read boundary。protected route 只消费 witness repository，依赖缺失时
fail closed；测试 fixture 显式注入 witness。

Fresh local verification / 新鲜本地验证: storage composition `4 passed`; PostgreSQL SQL-shape/lazy contract
`2 passed, 1 ignored`; protected API graph-diff `7 passed`; API exact-scope integration `4 passed` including the
missing-witness fail-closed regression; `cargo test --workspace --quiet --offline -j 1` passed with storage `233 passed,
41 ignored`; `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0` check; `pnpm check:web` with
public SDK `15`, local SDK `148`, Web `298`, and production build; `tests/contract/verify-local-contracts.test.ps1`
fixture tests; and `GRAPH_DIFF_IMPL_COUNT=1` all passed.

新鲜本地验证：storage composition `4 passed`；PostgreSQL SQL-shape/lazy contract `2 passed, 1 ignored`；protected API
graph-diff `7 passed`；API exact-scope integration `4 passed`（包含 missing-witness fail-closed regression）；
`cargo test --workspace --quiet --offline -j 1` 通过且 storage 为 `233 passed, 41 ignored`；`cargo fmt --all -- --check`、
strict offline Clippy、锁定 Rust `1.85.0` check；`pnpm check:web` 的 public SDK `15`、local SDK `148`、Web `298` 与
production build；`tests/contract/verify-local-contracts.test.ps1` fixture tests；以及 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。

This receipt advances Criteria 1, 2, and 4 but closes neither them nor repository convergence. The local contract verifier
fixture command passed; broader unified-diff/static scan output was not treated as a full receipt. Live PostgreSQL requiring
`CONTEXTLAB_TEST_DATABASE_URL`, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release,
production, and public protected-write readiness remain `ignored`, `unobserved`, or `deferred`. No public REST/OpenAPI/SDK
write, Web mutation, migration, provider, secret access, operator transport, or second graph-diff calculator was added.
The long-term goal remains `active`; the next increment requires a fresh bilingual Necessity Record.

本回执推进条件 1、2、4，但不关闭这些条件或仓库整体收束。local contract verifier fixture command 已通过；更广泛的
unified-diff/static scan 输出未作为完整回执。需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 live PostgreSQL、Docker、authenticated
browser/visual smoke、Git、remote CI、operator rehearsal、release、production 与 public protected-write readiness 继续为
`ignored`、`unobserved` 或 `deferred`。未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret
access、operator transport 或第二个 graph-diff calculator。长期目标保持 `active`；下一项增量必须先有新的双语
Necessity Record。

## 2026-08-02 Private Atomic Branch-Head Graph Witness / 2026-08-02 私有原子 Branch-Head Graph Witness

Status / 状态: `completed / verified locally` for this named local increment; the long-term goal
remains `active`. / 本命名本地增量状态为 `completed / verified locally`；长期目标保持 `active`。

Necessity Record / 必要性记录: This increment supplies the Criteria 2 and 4 evidence that a
server-owned `BranchName`, selected exact head, complete replayable history, and immutable source /
target graph snapshots are bound at one backend-owned observation point. The prior gap was a
split-read branch selection risk. The minimum affected boundary is the private Rust storage witness
repository plus its Memory/PostgreSQL adapters, focused tests, and bilingual architecture/roadmap
records. It intentionally adds no public REST/OpenAPI/SDK write, Web mutation, branch mutation,
merge, rollback, migration, provider, operator transport, release, production, or live database
claim. / 本增量补充条件 2 与 4 所需证据：server-owned `BranchName`、selected exact head、完整可回放 history 与 immutable
source/target graph snapshot 在同一个 backend-owned observation point 绑定。此前缺口是 split-read branch selection risk。最小
受影响边界是 private Rust storage witness repository、Memory/PostgreSQL adapter、focused tests 与双语架构/路线图记录。明确不新增
public REST/OpenAPI/SDK write、Web mutation、branch mutation、merge、rollback、migration、provider、operator transport、release、
production 或 live database 声明。

The split `PersistedContextGraphHistoryReviewService` has no callable `review_branch_head` and
only handles explicit commit scopes. The sole branch selector is the atomic
`PersistedContextGraphWitnessReviewService` path backed by
`ContextGraphBranchHeadReviewWitnessRepository`; it rejects unknown, unborn, mismatched, and
non-head targets. Memory uses one read guard, PostgreSQL uses one `REPEATABLE READ READ ONLY`
transaction, and graph comparison remains delegated to the sole `GraphDiff::between`. / split
`PersistedContextGraphHistoryReviewService` 没有可调用的 `review_branch_head`，只处理 explicit commit scope。唯一 branch selector
是由 `ContextGraphBranchHeadReviewWitnessRepository` 支持的 atomic `PersistedContextGraphWitnessReviewService` path；它拒绝
unknown、unborn、mismatched 与非 head target。Memory 使用一个 read guard，PostgreSQL 使用一个 `REPEATABLE READ READ ONLY`
transaction，graph comparison 继续委托给唯一的 `GraphDiff::between`。

Fresh evidence / 新鲜证据: Memory branch-bound `6 passed`; PostgreSQL SQL contract `3 passed, 1
ignored`; API `222 passed`; storage `230 passed, 41 ignored`; workspace Rust had no failures;
`cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0` check;
`pnpm check:web` with public SDK `15`, local SDK `148`, Web `298`, and production build;
`tests/contract/verify-local-contracts.test.ps1`; source inspection with zero split-service branch
selectors, one witness-service branch selector, one production `impl GraphDiff`, and ten
`GraphDiff::between` call sites. / 新鲜证据：Memory branch-bound `6 passed`；PostgreSQL SQL contract `3 passed, 1 ignored`；API
`222 passed`；storage `230 passed, 41 ignored`；workspace Rust 无失败；`cargo fmt --all -- --check`；strict offline Clippy；锁定
Rust `1.85.0` check；`pnpm check:web`（public SDK `15`、local SDK `148`、Web `298` 与 production build）；
`tests/contract/verify-local-contracts.test.ps1`；源码检查显示 split service branch selector 为零、witness service branch selector
为一、一个 production `impl GraphDiff` 与十个 `GraphDiff::between` 调用点。

PostgreSQL live runtime requiring `CONTEXTLAB_TEST_DATABASE_URL`, Docker, authenticated
browser/visual smoke, Git, remote CI, operator rehearsal, release, production, and public
protected-write readiness remain `ignored`, `unobserved`, or `deferred`. This receipt advances but
does not close Criteria 2 or 4, any other criterion, or repository convergence. The next admitted
increment requires a fresh bilingual Necessity Record. / 需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL live runtime、Docker、
authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release、production 与 public protected-write readiness
继续为 `ignored`、`unobserved` 或 `deferred`。本回执推进但不关闭条件 2、条件 4、其他条件或仓库整体收束。下一项准入增量必须
拥有新的双语 Necessity Record。

## 2026-08-02 Private Commit-Graph Snapshot Scope Invariant / 2026-08-02 私有 Commit Graph Snapshot Scope 不变量

Status / 状态: `completed / verified locally` for this named local increment; the long-term goal
remains `active`. / 本命名本地增量状态为 `completed / verified locally`；长期目标保持 `active`。

This receipt advances Criteria 1 and 2 and the charter's fail-closed stable-UUID rule. The typed
`CommitGraphSnapshotScope` previously allowed a nil project, Context, or commit UUID to reach the
domain constructor; `CommitGraphSnapshot::new` now rejects it before graph materialization with
`CommitGraphSnapshotError::InvalidScope`. The existing snapshot writer, repository, schema-V1,
replay/conflict, and version-backed review contracts are unchanged. / 本回执推进条件 1、2 与章程的 fail-closed
stable-UUID 原则。此前 typed `CommitGraphSnapshotScope` 允许 nil project、Context 或 commit UUID 到达 domain constructor；
`CommitGraphSnapshot::new` 现在会在 graph materialization 前以 `CommitGraphSnapshotError::InvalidScope` 拒绝。既有 snapshot writer、
repository、schema-V1、replay/conflict 与 version-backed review contract 未改变。

Fresh red/green evidence / 新鲜红绿证据: the nil-scope regression failed before the constructor
guard and passed after it; focused snapshot tests `6 passed`; snapshot repository contract `5
passed`; workspace Rust had no failures with API `222 passed` and storage `230 passed, 41 ignored`;
`cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0` check,
`pnpm check:web` (`15/148/298` plus production build),
`tests/contract/verify-local-contracts.test.ps1`, and source inspection with one production
`impl GraphDiff` and ten `GraphDiff::between` call sites passed. / 新鲜红绿证据：nil-scope regression 在 constructor guard 前失败、
接入后通过；focused snapshot tests `6 passed`；snapshot repository contract `5 passed`；workspace Rust 无失败，API `222 passed`、
storage `230 passed, 41 ignored`；`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`（`15/148/298` 与 production build）、`tests/contract/verify-local-contracts.test.ps1`，以及源码检查（一个 production
`impl GraphDiff`、十个 `GraphDiff::between` 调用点）通过。

PostgreSQL live runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator
rehearsal, release, production, and public protected-write readiness remain `ignored`,
`unobserved`, or `deferred`. This receipt does not close Criteria 1 or 2, any other criterion, or
repository convergence. The next admitted increment is the parent-snapshot ancestry invariant:
an existing parent commit without a materialized graph snapshot must fail closed before a child
snapshot is persisted. / PostgreSQL live runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release、
production 与 public protected-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`。本回执不关闭条件 1、2、其他条件或仓库
整体收束。下一项准入增量是 parent-snapshot ancestry invariant：已有 parent commit 但缺少 materialized graph snapshot 时，必须在
child snapshot 持久化前 fail closed。

## 2026-08-02 Parent Graph Snapshot Ancestry Receipt / 2026-08-02 Parent Graph Snapshot Ancestry 回执

This local increment advances Criteria 1 (Context-first graph coverage), Criterion 2 (replayable
version history), and Criterion 4 (Context Graph skeleton). The private Rust commit-snapshot writer
now rejects a child when a declared parent commit row exists without a materialized immutable graph
snapshot. Memory enforces this under its single write guard; PostgreSQL enforces the same shape with
an inner join in `PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL`. The child is rejected with the existing
`ScopeUnavailable` contract before persistence. / 本地增量推进条件 1（Context-first graph coverage）、条件 2（可回放版本历史）与条件 4
（Context Graph 骨架）。当声明的 parent commit row 存在但没有 materialized immutable graph snapshot 时，private Rust
commit-snapshot writer 现在拒绝 child。Memory 在单一 write guard 下执行；PostgreSQL 在
`PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL` 中使用 inner join 执行相同形状。child 在持久化前以现有 `ScopeUnavailable` contract 被拒绝。

Fresh evidence / 新鲜证据: red/green Memory regression; writer rejection tests `5 passed`; PostgreSQL SQL-shape contract
`1 passed`; `cargo test --workspace --quiet --offline -j 1` with API `222 passed` and storage `233 passed, 41 ignored`;
`cargo fmt --all -- --check`; strict offline Clippy; `cargo +1.85.0 check --workspace --all-targets --locked --offline`;
`pnpm check:web` with public SDK `15`, local SDK `148`, Web `298`, and production build;
`tests/contract/verify-local-contracts.test.ps1`; and source inspection with one production `impl GraphDiff` and `9` current
`GraphDiff::between` call sites. / 新鲜证据：Memory 红绿 regression；writer rejection tests `5 passed`；PostgreSQL SQL-shape contract
`1 passed`；`cargo test --workspace --quiet --offline -j 1`（API `222 passed`、storage `233 passed, 41 ignored`）；
`cargo fmt --all -- --check`；strict offline Clippy；`cargo +1.85.0 check --workspace --all-targets --locked --offline`；
`pnpm check:web`（public SDK `15`、local SDK `148`、Web `298` 与 production build）；
`tests/contract/verify-local-contracts.test.ps1`；以及源码检查（一个 production `impl GraphDiff`、当前 `9` 个 `GraphDiff::between` 调用点）。

This receipt is local non-production evidence and does not close any criterion or repository
convergence. PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git, remote CI,
operator rehearsal, release, production, and public protected-write readiness remain
`ignored`, `unobserved`, or `deferred`; arbitrary direct SQL bypass is outside this receipt. No
public REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, operator transport,
or second graph-diff calculator was added. The increment is `completed / verified locally`; the
long-term goal remains `active`. / 本回执是本地非生产证据，不关闭任何 completion criterion 或仓库收束。PostgreSQL runtime、Docker、
authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release、production 与 public protected-write readiness
继续为 `ignored`、`unobserved` 或 `deferred`；arbitrary direct SQL bypass 不在本回执范围内。未新增 public REST/OpenAPI/SDK write、
Web mutation、migration、provider、secret access、operator transport 或第二个 graph-diff calculator。本增量为
`completed / verified locally`；长期目标保持 `active`。

## 2026-08-02 Private Normal First-Parent Graph Review Ancestry / 2026-08-02 私有 Normal First-Parent 图审阅 Ancestry

Necessity Record / 必要性记录: This local read-only increment serves Criteria 2 and 4. A
version-backed graph review must prove that its target is a normal first-parent descendant of its
source; commit membership alone is insufficient. The minimum boundary is the reusable versioning
method, storage review guard, focused tests, and bilingual receipts. No public transport, write
surface, migration, or second GraphDiff calculator is in scope. / 本地只读增量服务条件 2、4。版本化 graph review 必须证明 target
是 source 的 normal first-parent descendant；仅有 commit membership 不足。最小边界是 reusable versioning method、storage review guard、
focused tests 与双语回执；不包含 public transport、write surface、migration 或第二个 GraphDiff calculator。

Fresh red/green evidence / 新鲜红绿证据: the initial versioning contract stage failed because the
method and typed range error were absent. The implementation then passed versioning history
contract `11`, storage history-review `8`, workspace Rust API `222`, storage `237 passed, 41
ignored`, format, strict offline Clippy, locked Rust `1.85.0`, and `pnpm check:web` with public SDK
`15`, local SDK `148`, Web `298`, and production build. Fixture contract tests passed. Static
inspection found one production `impl GraphDiff` and `10` `GraphDiff::between` matches. / 初始 versioning
contract 红测因 method 与 typed range error 缺失而失败；实现后 versioning history contract `11`、storage history-review `8`、workspace
Rust API `222`、storage `237 passed, 41 ignored`、format、strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`（public SDK `15`、
local SDK `148`、Web `298` 与 production build）均通过；fixture contract tests 通过。静态检查确认一个 production `impl GraphDiff` 与
`10` 个 `GraphDiff::between` 匹配。

The direct full local verifier remains `overall=blocked` on the pre-existing
`LocalBenchmarkWorkspaceResponse` safe-DTO baseline; its fixture regression passes and this
increment does not relabel that baseline. PostgreSQL runtime, Docker, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, production, and public protected-write
readiness remain `ignored`, `unobserved`, or `deferred`. Criteria 1, 2, and 4 remain open and the
long-term goal remains active. / 直接运行完整 local verifier 仍因既有 `LocalBenchmarkWorkspaceResponse` safe-DTO baseline 报告
`overall=blocked`；fixture regression 通过，本增量不重新标记该基线。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`。
条件 1、2、4 仍开放，长期目标保持 active。

## 2026-08-02 Private Local Verifier Decision-Pair Witness Alignment / 2026-08-02 私有 Local Verifier Decision-Pair Witness 对齐

This local evidence-tool increment serves Criterion 8 and the charter's stable fail-closed
contract principle. The verifier previously rejected the approved optional
`LocalBenchmarkWorkspaceResponse.decision_pair_witness` because its allowlist described an older
six-field response. It now checks the optional root field and the exact nested witness fields while
retaining unknown/raw-field rejection. / 本地 evidence-tool 增量服务条件 8 与章程的稳定 fail-closed contract 原则。verifier 此前因 allowlist
仍描述旧六字段 response，而拒绝已批准的可选 `LocalBenchmarkWorkspaceResponse.decision_pair_witness`。现在它检查可选 root field 与
精确 nested witness field，同时保留 unknown/raw-field rejection。

Fresh red/green evidence / 新鲜红绿证据：修复前 direct verifier 为 `safe_local_dto_fields=blocked`、`overall=blocked`；修复后
`tests/contract/verify-local-contracts.test.ps1` 通过，direct verifier 为 `safe_local_dto_fields=passed`，local source/route/SDK/public-boundary
checks 通过，`public_write_additions=unobserved`，无 unified diff input 时 `overall=unobserved`。fixture 同时证明 unknown nested field 与 raw
`payload` nested field 继续被阻止。

No product behavior or public surface changed. PostgreSQL/Docker, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, production, and public-write readiness remain
`ignored`, `unobserved`, or `deferred`. This closes the named verifier evidence gap only; Criterion
8 and the long-term goal remain open and active. / 未改变产品 behavior 或 public surface。PostgreSQL/Docker、authenticated browser/visual
smoke、Git、remote CI、operator rehearsal、release、production 与 public-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`。
本回执只关闭具名 verifier evidence gap；条件 8 与长期目标仍开放且保持 active。

## Local Evidence Update: Atomic Merge Review Witness / 本地证据更新：原子 Merge Review Witness

The local versioning/graph convergence evidence now includes a server-owned merge-review witness:
one plan and three exact immutable Context Graph snapshots are read from one backend observation;
the service revalidates the requested tip scope; missing wiring is unavailable rather than preview
fallback; and classification continues through the sole `GraphDiff::between` path. This advances
Criteria 2 and 4 but does not close either criterion globally. / 本地 versioning/graph convergence evidence 现包含 server-owned
merge-review witness：一个 plan 与三份 exact immutable Context Graph snapshot 来自一个 backend observation；service 重新校验 requested
tip scope；wiring 缺失时返回 unavailable 而不是 preview fallback；classification 继续通过唯一 `GraphDiff::between` path。该增量推进
条件 2 与 4，但不在全局关闭任何一个条件。

Observed local gates / 已观察本地门禁：storage merge-review `16 passed`; PostgreSQL SQL-shape `3 passed, 1 ignored`; API `223 passed`;
workspace Rust storage `238 passed, 41 ignored`; format; strict offline Clippy; locked Rust `1.85.0`; Web `15/148/298` plus
production build; local contract fixture; GraphDiff implementation count `1`; public merge-review surface count `0`.
PostgreSQL live runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, production, and public
protected-write readiness remain `ignored`, `unobserved`, or `deferred`. / 已观察本地门禁如上；PostgreSQL live runtime、Docker、authenticated
browser/visual smoke、Git、remote CI、operator rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、
`unobserved` 或 `deferred`。

The completion criteria remain open and the long-term goal remains active. The next admitted local
increment requires a new bilingual Necessity Record and is the private lifecycle branch-head target
binding contract; this receipt does not authorize public writes or declare production readiness. /
完成条件继续开放，长期目标继续 active。下一项准入的本地增量必须新增双语 Necessity Record，目标为 private lifecycle branch-head target
binding contract；本回执不授权 public writes，也不声明 production readiness。

## 2026-08-02 Completion Audit Update: Lifecycle Branch-Head Binding / 2026-08-02 完成审计更新：Lifecycle Branch-Head Binding

This receipt advances Criteria 1 (Context-first graph coverage), Criterion 2 (replayable version
history), and Criterion 4 (Context Graph skeleton) for the local Web composition boundary. It does
not mark those criteria complete. The server-owned branch target is now shared by lifecycle editing
and graph review, and stale committed review pairs are rejected by target scope.

本回执在本地 Web composition 边界推进条件 1（Context-first graph coverage）、条件 2（可重放版本历史）与条件 4（Context Graph 骨架），
但不将这些条件标记为完成。server-owned branch target 现由 lifecycle editing 与 graph review 共享，stale committed review pair 会被
target scope 拒绝。

Fresh evidence / 新鲜证据: red `22 passed, 2 failed`; green focused Web `33/33`; `pnpm check:web`
passed with TS SDK `15`, local SDK `148`, Web `304`, and production build; Rust workspace passed
with storage `238 passed, 41 ignored`; fmt, strict offline Clippy, locked Rust `1.85.0`, local
contract verifier, and fixture verifier passed. Static checks found one production GraphDiff
implementation, ten `GraphDiff::between` call sites, and zero public merge-review surface hits.

新鲜证据：红测 `22 passed, 2 failed`；focused Web 绿测 `33/33`；`pnpm check:web` 通过（TS SDK `15`、local SDK `148`、Web `304` 与
production build）；Rust workspace 通过（storage `238 passed, 41 ignored`）；fmt、strict offline Clippy、锁定 Rust `1.85.0`、local contract
verifier 与 fixture verifier 通过。静态检查为一个 production GraphDiff implementation、十个 `GraphDiff::between` call site、public
merge-review surface hits 为零。

Evidence boundary / 证据边界: direct verifier `overall=unobserved` without unified diff input;
PostgreSQL runtime `ignored` without `CONTEXTLAB_TEST_DATABASE_URL`; Docker, authenticated
browser/visual, Git, remote CI, operator rehearsal, release, production, and public protected-write
readiness remain unobserved or deferred. The long-term goal remains active.

证据边界：未提供 unified diff input 时 direct verifier 为 `overall=unobserved`；缺少 `CONTEXTLAB_TEST_DATABASE_URL` 时 PostgreSQL runtime 为
`ignored`；Docker、authenticated browser/visual、Git、remote CI、operator rehearsal、release、production 与 public protected-write readiness
继续为 unobserved 或 deferred。长期目标保持 active。

## 2026-08-02 Private Context Lifecycle Error Redaction / 2026-08-02 私有 Context 生命周期错误脱敏

This bounded Web security receipt advances Criterion 6 (production security and collaboration) and
the charter's secure-by-default, secret-redaction, and fail-closed principles. The private Context
lifecycle data boundary now removes upstream diagnostic messages during proxy parsing and
`LocalLifecycleProxyError` construction; the existing bilingual presenter maps stable error codes
and statuses without reading the upstream message. Conflict, rate-limit, and unknown-status errors
are covered in regression tests, while the existing disabled-gate behavior, retry-after, exact
scope checks, request-memory Bearer handling, `credentials: "omit"`, and `cache: "no-store"` remain
unchanged.

本有界 Web 安全回执推进条件 6（生产级安全与协作）以及宪章的 secure-by-default、secret-redaction 与 fail-closed 原则。私有
Context lifecycle data boundary 现会在 proxy parsing 与 `LocalLifecycleProxyError` construction 阶段移除 upstream diagnostic message；
既有双语 presenter 依据 stable error code 与 status 映射文案，不读取 upstream message。regression tests 覆盖 conflict、rate-limit 与
unknown-status error；既有 disabled-gate behavior、retry-after、exact scope checks、request-memory Bearer handling、`credentials: "omit"`
与 `cache: "no-store"` 保持不变。

Fresh local verification / 新鲜本地验证：focused lifecycle data `9 passed` and editor `14 passed`; `pnpm check:web` passed with public
SDK `15`, local SDK `148`, Web `308` tests, TypeScript/lint, and production build; `cargo fmt --all -- --check`; offline workspace Rust with
API `223 passed` and storage `238 passed, 41 ignored`; strict offline Clippy; locked Rust `1.85.0` check; fixture contract verification; and
source inspection with `GRAPH_DIFF_IMPL_COUNT=1` and `GraphDiff::between` in `10` Rust matches.

新鲜本地验证：focused lifecycle data `9 passed`、editor `14 passed`；`pnpm check:web` 通过（public SDK `15`、local SDK `148`、Web `308` tests、
TypeScript/lint 与 production build）；`cargo fmt --all -- --check`；离线 workspace Rust 通过（API `223 passed`、storage `238 passed, 41 ignored`）；
strict offline Clippy；锁定 Rust `1.85.0` check；fixture contract verification；以及源码检查 `GRAPH_DIFF_IMPL_COUNT=1`、Rust 中
`GraphDiff::between` 共 `10` 个匹配。

No Rust/API/SDK/OpenAPI/public write, migration, provider, secret access, operator transport,
second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, or production claim was added. PostgreSQL runtime,
authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production
remain `ignored`, `unobserved`, or `deferred` as applicable. This receipt advances Criterion 6
only; it does not close Criterion 6, any other completion criterion, or the repository convergence,
and the long-term goal remains `active`. Any next implementation requires a fresh bilingual
Necessity Record.

未新增 Rust/API/SDK/OpenAPI/public write、migration、provider、secret access、operator transport、第二个 GraphDiff calculator、
Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。PostgreSQL
runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续按适用情况标记为 `ignored`、
`unobserved` 或 `deferred`。本回执仅推进条件 6；不关闭条件 6、任何其他 completion criterion 或仓库整体收束，长期目标保持 `active`。
任何下一项实现都必须先有新的双语 Necessity Record。

Documentation reconciliation / 文档收束对账：this appended receipt is the current bilingual
red/green evidence for the private lifecycle error-redaction increment. The completion-criteria
record now treats benchmark breadth as already evidenced by later local receipts and identifies
only the authenticated browser-to-BFF-to-protected-Axum lifecycle smoke as the current local
unobserved boundary. The active long-term goal remains active; no Docker/PostgreSQL runtime,
browser, Git, remote, operator, release, or production evidence is claimed. / 文档收束对账：本追加回执是
私有 Context 生命周期错误脱敏增量当前的双语红绿证据。completion-criteria 现将 benchmark breadth 视为已有后续本地回执支持，当前本地
未观测边界仅记录为 authenticated browser-to-BFF-to-protected-Axum lifecycle smoke。长期目标继续保持 active；不宣称 Docker/PostgreSQL
runtime、browser、Git、remote、operator、release 或 production evidence。

## 2026-08-30 Private ContextLab Browser Smoke Evidence / 2026-08-30 私有 ContextLab 浏览器 Smoke 证据

This bounded evidence receipt advances Criteria 1 and 8. The current production Next build was checked through a black-box Playwright harness at desktop (1440x1100) and mobile (390x844) viewports. Both renders exposed the ContextLab workspace and the mounted private Workflow binding controls, with no page errors, unexpected console errors, failed HTTP responses, or horizontal overflow. The desktop interaction filled the memory-only Bearer field and clicked the actual private binding inspector; preview mode returned the expected `503` unavailable response, one local notice, and no upstream diagnostic text.

本有界证据回执推进条件 1 与 8。当前 production Next build 已通过 black-box Playwright harness 在 desktop（1440x1100）与 mobile（390x844）viewport 检查。两种尺寸均显示 ContextLab workspace 与已挂载的 private Workflow binding controls，无 page error、意外 console error、失败 HTTP response 或 horizontal overflow。desktop interaction 填写仅存于内存的 Bearer field 并点击真实 private binding inspector；preview mode 返回预期 `503` unavailable response、一条本地 notice，且无 upstream diagnostic text。

Fresh receipt / 新鲜回执: `scripts/verify-contextlab-browser-smoke.py` exited `0` under the production server. It reported 27 buttons and 25 inputs on both viewports, all required selectors present, empty page/console/HTTP failure collections, and viewport scroll widths equal to viewport widths. The desktop private-read probe reported status `503`, notice count `1`, and the stable local error body `contextlab_web_api_unavailable`.

新鲜回执：`scripts/verify-contextlab-browser-smoke.py` 在 production server 下以 `0` 退出。两种 viewport 均报告 27 个 buttons、25 个 inputs，所有 required selector 存在，page/console/HTTP failure collection 为空，viewport scroll width 与 viewport width 相等。desktop private-read probe 报告 status `503`、notice count `1` 与稳定本地 error body `contextlab_web_api_unavailable`。

This receipt is preview-mode browser evidence, not authenticated protected-runtime evidence. Authenticated browser-to-BFF-to-protected-Axum runtime, PostgreSQL/Docker, Git, remote CI, operator rehearsal, release, production, and public-write readiness remain `unobserved` or `deferred`. No application source or public contract changed; the active long-term goal remains open.

本回执是 preview-mode browser evidence，不是 authenticated protected-runtime evidence。authenticated browser-to-BFF-to-protected-Axum runtime、PostgreSQL/Docker、Git、remote CI、operator rehearsal、release、production 与 public-write readiness 继续为 `unobserved` 或 `deferred`。未修改 application source 或 public contract；active long-term goal 继续开放。

## 2026-08-31 Protected Browser Runtime Dependency Audit / 2026-08-31 受保护浏览器运行时依赖审计

This audit is `completed / verified locally` as an environment receipt only. Docker CLI `29.6.2`
is installed, but the Docker server did not respond after one start attempt and
`com.docker.service` remained `Stopped/Manual`. The Windows PostgreSQL command set
(`postgres`, `initdb`, `pg_ctl`, `createdb`, `psql`) is absent, and WSL has no usable distribution.
Protected mode still requires PostgreSQL, authentication, and rate-limit configuration as stated
by `.env.example`.

本审计作为环境回执为 `completed / verified locally`。Docker CLI `29.6.2` 已安装，但一次启动尝试后 Docker server
仍无响应，`com.docker.service` 继续为 `Stopped/Manual`。Windows PostgreSQL command set（`postgres`、`initdb`、`pg_ctl`、
`createdb`、`psql`）不存在，WSL 没有可用发行版。`.env.example` 仍明确 protected mode 需要 PostgreSQL、authentication 与 rate-limit configuration。

This does not establish protected runtime success, live PostgreSQL persistence, or authenticated
browser-to-BFF-to-protected-Axum success. Those boundaries remain `unobserved`; no application or
public contract changed, and the long-term goal remains `active`.

本审计不构成 protected runtime success、live PostgreSQL persistence 或 authenticated browser-to-BFF-to-protected-Axum
success 证据。上述边界继续为 `unobserved`；未改变 application 或 public contract，长期目标保持 `active`。
