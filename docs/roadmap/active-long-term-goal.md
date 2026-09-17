# Active Long-Term Goal / 当前长期目标

## Mission / 使命

ContextLab is being built as a bilingual, open-source Context Engineering platform for production AI applications. Context, rather than an isolated prompt, is the primary abstraction. A unified Context Graph and replayable version history form the system backbone for context design, editing, versioning, semantic/behavior/evaluation diffs, experiments, benchmark-driven evaluation, collaborative AI workflows, knowledge, memory, extensible providers/plugins, and design-system-first Web, CLI, and Desktop experiences.

ContextLab 正在建设为中英双语、开源、面向生产级 AI 应用的 Context Engineering 平台。Context 而非孤立 Prompt 是首要抽象；统一 Context Graph 与可重放版本历史构成系统骨架，用于逐步建设上下文设计与编辑、版本化、语义/行为/评测 Diff、实验与 benchmark 驱动评测、协作式 AI workflow、知识、记忆、可扩展 provider/plugin，以及 design-system-first 的 Web、CLI 与 Desktop 体验。

## Current Tracks / 当前双轨

Core platform convergence is the active local delivery track. Dependency-ready private Context-first work continues through a bilingual Necessity Record and fresh local verification. The public version-backed commit graph-diff GET route, its OpenAPI operation, and its public SDK method have been removed; the read now exists only on the default-off protected-local route and through the non-public local SDK and same-origin BFF. The separate public pure graph-diff POST calculation remains, with `GraphDiff::between` still the sole calculator. Remote disposable CI, operator-approved change rehearsal, and public protected-write, release, or production promotion are explicitly deferred future deployment prerequisites because their operating conditions are unavailable; they are not current gates, subagents, or waiting states.

核心平台收束是当前活跃的本地交付轨道。依赖就绪的私有 Context-first 工作通过双语 Necessity Record 与新鲜本地验证继续实施。public version-backed commit graph-diff GET route、对应 OpenAPI operation 与 public SDK method 已移除；读取现在只存在于默认关闭的 protected-local route，以及 non-public local SDK 与同源 BFF。独立的 public pure graph-diff POST calculation 仍保留，`GraphDiff::between` 仍是唯一 calculator。远端 disposable CI、operator 批准的变更演练，以及 public protected-write、release 或生产推广因外部运行条件不可用而明确延期；它们不是当前门禁、子任务或等待状态。

Current local work advances Context/Prompt/Schema and graph-relationship editing, branches/merges/replay, benchmark datasets, regression thresholds, evaluation dashboards, semantic/behavior/evaluation diffs, workflows, memory, knowledge, MCP/plugins, Desktop/CLI, and contributor workflows only when their own Necessity Records are dependency-ready. Any future public deployment decision begins only when external operating conditions actually exist; no such work is active now.

当前本地工作仅在各自 Necessity Record 证明依赖就绪时，推进 Context/Prompt/Schema 与图谱关系编辑、分支/合并/回放、benchmark dataset、回归阈值、评测仪表盘、semantic/behavior/evaluation diff、workflow、memory、knowledge、MCP/plugin、Desktop/CLI 与贡献流程。任何未来 public deployment 决策都只会在外部运行条件实际具备时才开始；当前没有此类工作。

## Latest Verified Increment / 最近已验证增量

### 2026-08-01 Private Branch-Head Error Redaction / 2026-08-01 私有 Branch-Head 错误脱敏

`completed / verified locally` for this bounded Criterion 4 security increment; the long-term
goal remains `active`. The private Web branch-head data adapter now preserves the upstream error
code and HTTP status but never forwards a structured diagnostic message to the shared presenter.
Unknown-status responses use one stable bilingual local message. Existing scope validation,
request-memory bearer handling, retry behavior, and shared presenter/screen states are unchanged.

本次有界条件 4 安全增量标记为 `completed / verified locally`；长期目标保持 `active`。私有 Web branch-head data adapter 现保留 upstream error code 与 HTTP status，但不再将 structured diagnostic message 传给 shared presenter。unknown-status response 统一使用稳定双语本地文案。既有 scope validation、request-memory bearer、retry behavior 与 shared presenter/screen state 均未改变。

Fresh red/green and local verification / 新鲜红绿与本地验证：the old behavior observed `6 passed,
2 failed` with both raw upstream messages exposed; the repaired focused suite observed `8 passed`.
`cargo fmt --all -- --check`, offline workspace Rust (`storage 212 passed, 39 ignored`), strict
offline Clippy, locked Rust `1.85.0`, `pnpm check:web` (`public SDK 15`, `local SDK 135`, `Web
284`, TypeScript/lint, and production build), scoped local verifier, and `GRAPH_DIFF_IMPL_COUNT=1`
passed. The verifier remains `overall=unobserved` without unified diff input.

新鲜红绿与本地验证：旧行为观测到 `6 passed, 2 failed`，两个 raw upstream message 均暴露；修复后的 focused suite 观测到 `8 passed`。`cargo fmt --all -- --check`、offline workspace Rust（storage `212 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`（public SDK `15`、local SDK `135`、Web `284`、TypeScript/lint 与 production build）、范围化 local verifier 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。verifier 因未提供 unified diff input 继续为 `overall=unobserved`。

No Rust/API/SDK/OpenAPI/route/write/migration/provider/secret/public transport change was added.
Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. Criterion 4 and the broader convergence
conditions remain open; the next implementation requires a new bilingual Necessity Record.

未新增 Rust/API/SDK/OpenAPI/route/write/migration/provider/secret/public transport 变更。Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。条件 4 与更广泛收束条件继续开放；下一项实现必须新增双语 Necessity Record。

### 2026-08-01 Private Memory Writer-to-Review Composition / 2026-08-01 私有 Memory Writer-to-Review 组合

`completed / verified locally` for this bounded Criterion 1/2/4 root-cause correction; the
long-term goal remains `active`. The Memory `WorkspaceDataRepository` previously created a fresh
empty diff snapshot adapter while the commit writer used the shared `InMemoryContextGraphRepository`
state. The adapter now reuses the repository clone, and a red-to-green regression proves a guarded
commit's derived diff snapshot is readable at its exact scope through the persisted-review adapter.

本有界条件 1/2/4 根因修复标记为 `completed / verified locally`；长期目标保持 `active`。Memory `WorkspaceDataRepository` 过去在 commit writer 使用共享
`InMemoryContextGraphRepository` state 时，却为 diff snapshot read 新建空 adapter。adapter 现复用 repository clone；red-to-green regression 证明 guarded commit 派生的 diff snapshot
可通过 persisted-review adapter 按 exact scope 读回。

Fresh local evidence / 新鲜本地证据：API lib `215 passed`（含 writer-to-review regression）；storage snapshot repository `4 passed`；workspace Rust API `215 passed`、storage `212 passed, 39 ignored`；
`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）、local contract verifier scoped checks 与
`GRAPH_DIFF_IMPL_COUNT=1` 均通过。verifier 因未提供 unified diff input，整体保持 `overall=unobserved`。

The evidence is limited to in-memory repository identity. The generic `with_workspace_repositories`
fixture constructor remains test-composed; protected runtime uses PostgreSQL and no PostgreSQL
runtime receipt is claimed. Behavior/evaluation collections remain empty without real producer
facts. No public write, OpenAPI/public SDK write method, Web mutation, migration, provider, secret,
operator transport, or second `GraphDiff` calculator was added. The next increment requires a new
bilingual Necessity Record.

证据仅限 in-memory repository identity。通用 `with_workspace_repositories` fixture constructor 仍由 test 组合；protected runtime 使用 PostgreSQL，本回执不声称 PostgreSQL runtime evidence。
没有真实 producer facts 时 behavior/evaluation collection 继续为空。没有新增 public write、OpenAPI/public SDK write method、Web mutation、migration、provider、secret、operator transport 或第二个
`GraphDiff` calculator。下一增量必须新增双语 Necessity Record。

### 2026-08-01 Private Versioned Context Diff Boundary Witness / 2026-08-01 私有版本化 Context Diff 边界见证

`completed / verified locally` for this bounded read-contract evidence increment; Criteria 2 and 4
are advanced but remain open, and the long-term goal remains `active`. An independent black-box
protected-router test now witnesses authentication, exact Context/source/target commit scope,
private cache headers, stable non-empty graph diff output, public-route retirement, and same-commit
rejection. The local SDK parser matrix now covers `added` and `removed` semantic documents,
behavior cases, and evaluation metrics with fail-closed extra-field checks.

本有界读取契约证据增量标记为 `completed / verified locally`；条件 2 与 4 得到推进但仍开放，长期目标保持 `active`。独立 black-box protected-router test 现已见证
authentication、精确 Context/source/target commit scope、private cache header、稳定非空 graph diff output、public route retirement 与 same-commit rejection。local SDK parser matrix
现覆盖 `added` 与 `removed` semantic document、behavior case 与 evaluation metric，并对 extra-field fail closed。

Fresh local evidence / 新鲜本地证据：protected API boundary `3 passed`；local SDK parser `10 passed`；workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；
`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）、
local contract verifier scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。verifier 因未提供 unified diff input，整体保持 `overall=unobserved`。

The witness uses in-memory fixtures and does not prove Memory writer-to-review repository identity,
PostgreSQL-backed service flow, authenticated browser/visual E2E, Git, remote CI, operator rehearsal,
release, or production. No public write, OpenAPI/public SDK write method, Web mutation, migration,
provider, secret access, operator transport, or second `GraphDiff` calculator was added. The next
increment requires a new bilingual Necessity Record for the discovered local repository-composition
gap before any implementation begins.

本 witness 使用 in-memory fixture，不能证明 Memory writer-to-review repository identity、PostgreSQL-backed service flow、authenticated browser/visual E2E、Git、remote CI、operator rehearsal、
release 或 production。没有新增 public write、OpenAPI/public SDK write method、Web mutation、migration、provider、secret access、operator transport 或第二个 `GraphDiff` calculator。下一增量在开始实现前必须
针对已发现的本地 repository-composition gap 新增双语 Necessity Record。

### 2026-08-01 Private Context Diff V1 Read-Scope Hardening / 2026-08-01 私有 Context Diff V1 读取范围硬化

`completed / verified locally` for the bounded storage contract hardening; Criteria 2 is advanced
but remains open, and the long-term goal stays `active`. The PostgreSQL V1 read now binds the only
supported `context-diff-snapshot-v1` schema instead of selecting an arbitrary schema row. Memory
tests independently reject project, Context, and commit scope drift with typed `NotFound` errors.

本有界 storage contract hardening 标记为 `completed / verified locally`；条件 2 得到推进但仍开放，长期目标保持 `active`。PostgreSQL V1 read 现显式绑定唯一支持的
`context-diff-snapshot-v1` schema，不再选择任意 schema row。Memory tests 分别以 typed `NotFound` 拒绝 project、Context 与 commit scope drift。

Fresh local evidence / 新鲜本地证据：focused Memory scope `4 passed`、PostgreSQL SQL contract `1 passed`；workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；
format、strict offline Clippy、locked Rust `1.85.0` check、`pnpm check:web`（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）、local verifier
scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。verifier 因未提供 unified diff input 保持 `overall=unobserved`。

新鲜本地证据：focused Memory scope `4 passed`、PostgreSQL SQL contract `1 passed`；workspace Rust API `214 passed`、storage `212 passed, 39 ignored`；format、strict offline Clippy、
锁定 Rust `1.85.0` check、`pnpm check:web`（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）、local verifier scoped checks 与
`GRAPH_DIFF_IMPL_COUNT=1` 均通过。verifier 因未提供 unified diff input 保持 `overall=unobserved`。

No migration, PostgreSQL runtime execution, public REST/OpenAPI/public SDK method, Web mutation,
operator transport, provider, secret access, or second `GraphDiff` calculator was added. Docker/
PostgreSQL runtime, authenticated browser, visual smoke, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`; the next increment requires a new
bilingual Necessity Record.

没有新增 migration、PostgreSQL runtime execution、public REST/OpenAPI/public SDK method、Web mutation、operator transport、provider、secret access 或第二个 `GraphDiff` calculator。
Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；下一增量必须新增双语 Necessity Record。

### 2026-08-01 Private Persisted Context Diff Writer and Three-Section Review Evidence / 2026-08-01 私有持久化 Context Diff Writer 与三段 Review 证据

`completed / verified locally` for this bounded evidence increment; Criteria 1, 2, and 4 are
advanced but remain open, and the long-term goal stays `active`. The audit found that production
commit writers persisted only the commit and graph snapshot while diff rows existed only in
fixtures. The minimum repair now derives a valid `ContextDiffSnapshotV1` from the same immutable
graph and persists it in the existing Memory/PostgreSQL normal and guarded commit transaction.
Memory exact-scope read and guarded idempotent replay return the same record. Empty behavior and
evaluation collections remain honest when no benchmark evidence exists; explicit redacted test
producer facts prove non-empty three-section API/BFF projections without inventing runtime results.

本有界证据增量标记为 `completed / verified locally`；条件 1、2、4 得到推进但仍开放，长期目标保持 `active`。审计发现 production commit writer
过去只持久化 commit 与 graph snapshot，而 diff row 仅存在于 fixture。最小修复现从同一不可变 graph 派生合法 `ContextDiffSnapshotV1`，并在
既有 Memory/PostgreSQL 普通与 guarded commit transaction 中持久化。Memory exact-scope read 与 guarded 幂等 replay 返回同一 record。没有 benchmark
evidence 时 behavior 与 evaluation collection 继续诚实地为空；明确的脱敏测试 producer facts 用于证明 API/BFF 三段 projection 非空，不虚构运行时结果。

Fresh local evidence / 新鲜本地证据：`cargo fmt --all -- --check` passed；workspace Rust passed with API `214 passed` and storage `212 passed, 39 ignored`；
focused storage guarded replay `1 passed`、storage review `8 passed`、direct API review `1 passed`、protected API review `3 passed`；strict offline Clippy
passed；locked Rust `1.85.0` check passed；`pnpm check:web` passed with public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build；
local contract verifier scoped checks passed with `graph_diff_application=passed count=1` and `overall=unobserved` because no unified diff input was supplied；
`GRAPH_DIFF_IMPL_COUNT=1` passed。

新鲜本地证据：`cargo fmt --all -- --check` 通过；workspace Rust 通过（API `214 passed`、storage `212 passed, 39 ignored`）；focused storage guarded replay `1 passed`、
storage review `8 passed`、direct API review `1 passed`、protected API review `3 passed`；strict offline Clippy 通过；锁定 Rust `1.85.0` check 通过；`pnpm check:web` 通过
（public SDK `15`、local SDK `134`、Web `272`、TypeScript/lint 与 production build）；local contract verifier scoped checks 通过，`graph_diff_application=passed count=1`，
因未提供 unified diff input 整体为 `overall=unobserved`；`GRAPH_DIFF_IMPL_COUNT=1` 通过。

`GraphDiff::between` remains the sole calculator. No public REST/OpenAPI/public SDK write, operator
transport, Web mutation, provider, secret access, or production migration was added. PostgreSQL/
Docker runtime, authenticated browser, visual smoke, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`; the next increment requires a new
bilingual Necessity Record.

`GraphDiff::between` 仍是唯一 calculator。没有新增 public REST/OpenAPI/public SDK write、operator transport、Web mutation、provider、secret access 或 production migration。
PostgreSQL/Docker runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；
下一增量必须新增双语 Necessity Record。

### 2026-07-30 Private Workflow Execution Status and Replay Provenance Read Closure / 2026-07-30 私有 Workflow 执行状态与回放 provenance 读取收束

`completed / verified locally`. This local, private, read-only increment advances Criteria 1 and
5. A schema-versioned Rust projection now validates exact Context-to-Workflow binding scope and
replay provenance while exposing only deterministic counts, identifiers, state, and capability
snapshot digest. The protected local API, non-public local SDK, same-origin BFF, and shared Web
`data -> presenter -> screen` composition consume the projection. The default runtime has no
execution repository, so the read is typed `unavailable`; this is not an execution producer,
persistence feature, or production-readiness claim.

`completed / verified locally`。本地 private、read-only 增量推进条件 1 与 5。schema-versioned Rust projection 现会校验精确的
Context-to-Workflow binding scope 与 replay provenance，只暴露确定性 counters、identifiers、state 与 capability snapshot digest。
protected local API、非公开 local SDK、同源 BFF 与 shared Web `data -> presenter -> screen` composition 已消费该 projection。默认运行时
没有 execution repository，因此读取返回 typed `unavailable`；这不是 execution producer、持久化功能或 production-readiness 声明。

Fresh local evidence / 新鲜本地证据：Workflow projection `5 passed`；protected API route `1 passed`；local SDK `111 passed`；execution-status BFF
route `4 passed`；Web
data/presenter/screen `7 passed`；`pnpm check:web` passed with public SDK `15`, local SDK `111`, Web `229`, TypeScript/lint 与
production build；workspace Rust passed with storage `204 passed, 39 ignored`；`cargo fmt --all -- --check`、strict offline Clippy、locked
Rust `1.85.0` check、`GRAPH_DIFF_IMPL_COUNT=1` 与 public workflow/plugin capability surface hits `0` 均通过。

没有新增 public REST/OpenAPI/public SDK method、execution start route、Workflow mutation、provider、migration、operator transport、secret 或
第二个 `GraphDiff` calculator。Docker/PostgreSQL、authenticated browser、visual、Git、remote CI、operator、release 与 production 继续为
`unobserved` 或 `deferred`；长期目标保持 active。下一项工作必须先有独立双语 Necessity Record，并只选择依赖就绪的本地完成条件。

The broader `scripts/verify-local-contracts.ps1` baseline remains outside this receipt because its
unrelated benchmark workspace route check stops at `benchmark-workspace-route-method-count:2`;
the narrower Workflow/Plugin public-surface and GraphDiff checks passed.

更宽的 `scripts/verify-local-contracts.ps1` baseline 仍不属于本回执，因为其无关的 benchmark workspace route check 在
`benchmark-workspace-route-method-count:2` 处停止；更窄的 Workflow/Plugin public-surface 与 GraphDiff check 已通过。

### 2026-07-30 Private Plugin/MCP Capability Availability Read Closure / 2026-07-30 私有 Plugin/MCP 能力可用性读取收束

`completed / verified locally`. The admitted private Plugin/MCP read is now implemented across
the existing MCP/plugin-runtime projection, protected local API, non-public local SDK,
same-origin BFF, and shared bilingual Web capability-state composition. The resource is exact
Context-scoped, redacted, deterministic, provider-free, request-scoped, and `no-store`; missing
runtime registration remains empty/unavailable. This advances Criterion 7 but does not close it
or the long-term goal.

`completed / verified locally`。已准入的 private Plugin/MCP read 现已贯通既有 MCP/plugin-runtime projection、protected local API、
非公开 local SDK、同源 BFF 与 shared 双语 Web capability-state composition。resource 保持 exact Context scope、脱敏、确定性、
provider-free、request-scoped 与 `no-store`；缺少 runtime registration 时继续表示为 empty/unavailable。本增量推进条件 7，
但不关闭条件 7 或长期目标。

Fresh local evidence / 新鲜本地证据：focused MCP `8 passed`、plugin-runtime `9 passed`、API `2 passed`、local SDK `3 passed`、
Web BFF/inspector `5 passed`；`pnpm check:web` passed with public SDK `15`、local SDK `107`、Web `218`、TypeScript/lint 与
production build；workspace Rust passed with storage `204 passed, 39 ignored`；locked Rust `1.85.0` check passed；static
`GRAPH_DIFF_IMPL_COUNT=1` and public Plugin/MCP capability surface hits `0`。

新鲜本地证据：focused MCP `8 passed`、plugin-runtime `9 passed`、API `2 passed`、local SDK `3 passed`、Web BFF/inspector `5 passed`；
`pnpm check:web` 通过（public SDK `15`、local SDK `107`、Web `218`、TypeScript/lint 与 production build）；workspace Rust 通过，
其中 storage `204 passed, 39 ignored`；锁定 Rust `1.85.0` check 通过；静态 `GRAPH_DIFF_IMPL_COUNT=1`，public Plugin/MCP capability
surface 命中 `0`。

`cargo fmt --all -- --check` and strict offline Clippy were observed as `failed` on unrelated
Workflow formatting and `missing_docs` drift in `crates/workflow/src/execution_status.rs`; this
does not change the scoped Plugin/MCP completion, and remains a repository-wide follow-up outside
this Docs/QA ownership. PostgreSQL/Docker runtime, browser, Git, remote CI, operator, release,
and production remain `unobserved` or `deferred`. Two failed coding-worker dispatches are recorded
as execution provenance only, not as product blockers.

`cargo fmt --all -- --check` 与 strict offline Clippy 在无关的 `crates/workflow/src/execution_status.rs` Workflow formatting 与
`missing_docs` drift 上观测为 `failed`；这不改变 scoped Plugin/MCP completion，仍是超出本 Docs/QA ownership 的 repository-wide
follow-up。PostgreSQL/Docker runtime、browser、Git、remote CI、operator、release 与 production 继续为 `unobserved` 或 `deferred`。
两次 failed coding-worker dispatch 仅作为 execution provenance 记录，不是 product blocker。

### 2026-07-30 Private Workflow Binding Availability Closure / 2026-07-30 私有 Workflow Binding 可用性收束

The private Workflow Context-binding read now reaches the shared five-state Web contract at the
real inspector boundary: a typed upstream `503` maps to `unavailable`, while authorization,
rate-limit, protocol, and transport failures remain `error`. The successful inspector regression
also renders the selected redacted binding through `LocalWorkflowContextBindingsScreen`. The shared
local SDK request helper now declares `cache: "no-store"` together with request-scoped Bearer and
cookie omission. This is a local read-only correction advancing Criteria 1, 5, 6, and 9; it adds
no route, public API/OpenAPI/SDK method, mutation, provider, secret, or second GraphDiff calculator.

私有 Workflow Context binding read 现已在真实 inspector boundary 到达 shared five-state Web contract：typed upstream `503`
映射为 `unavailable`，authorization、rate-limit、protocol 与 transport failure 继续保持 `error`。成功 inspector regression
还会通过 `LocalWorkflowContextBindingsScreen` 渲染选定的脱敏 binding。共享 local SDK request helper 现与 request-scoped
Bearer、cookie omission 一起显式声明 `cache: "no-store"`。本增量是推进条件 1、5、6 与 9 的本地只读修正；未新增 route、
public API/OpenAPI/SDK method、mutation、provider、secret 或第二个 GraphDiff calculator。

Fresh local evidence passed focused Web Workflow `11/11`, focused local SDK Workflow `7/7`,
`pnpm check:web` with public SDK `15`, local SDK `99`, Web `202`, TypeScript/lint, and production
build, `cargo test --workspace --quiet --no-fail-fast` with storage `193 passed, 39 ignored`, API
Workflow binding `4 passed`, `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust
`1.85.0` check. Docker/PostgreSQL runtime, authenticated browser, Git change-set, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`; the long-term goal
remains `active` and the next increment requires a new bilingual Necessity Record.

新鲜本地证据已通过 focused Web Workflow `11/11`、focused local SDK Workflow `7/7`、`pnpm check:web`（public SDK `15`、
local SDK `99`、Web `202`、TypeScript/lint 与 production build）、`cargo test --workspace --quiet --no-fail-fast`（storage
`193 passed, 39 ignored`）、API Workflow binding `4 passed`、`cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust
`1.85.0` check。Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release
与 production 继续为 `unobserved` 或 `deferred`；长期目标保持 `active`，下一增量必须先新增双语 Necessity Record。

### 2026-07-30 Private Context Lifecycle Graph-Diff Review / 2026-07-30 私有 Context 生命周期 Graph-Diff 审阅

The local Context lifecycle workflow now binds a successful guarded commit to an immediately
reviewable version-backed graph diff. The editor captures the exact selected prior head and the
server-returned new commit for create, content update, removal, and relationship operations. Empty
or same-commit replay pairs are rejected. The bridge delegates to the existing graph-review
composition, selects the exact original/revised pair, clears stale results, and remounts when the
Context or pair identity changes. Stale-head/conflict failures remain write failures and do not
trigger a graph-diff request.

本地 Context lifecycle workflow 现会把成功的 guarded commit 绑定到可立即审阅的 version-backed graph diff。editor 会为 create、
content update、removal 与 relationship operation 捕获精确的已选 prior head 与 server 返回的新 commit；空 pair 或 same-commit
replay 会被拒绝。bridge 委托既有 graph-review composition，选择 exact original/revised pair、清除旧 result，并在 Context 或 pair
identity 变化时重新挂载。stale-head/conflict failure 仍是 write failure，不会触发 graph-diff request。

Fresh local evidence passed `context-lifecycle-editor.test.tsx` `7/7`,
`local-branch-heads-graph-review.test.tsx` `5/5`, `pnpm check:web` with public SDK `15`, local SDK
`99`, Web `207`, TypeScript/lint, and production
build, `cargo test --workspace --quiet --no-fail-fast` with storage `193 passed, 39 ignored`,
`cargo fmt --all -- --check`, strict offline workspace Clippy, locked Rust `1.85.0` check, and
static `impl GraphDiff` count `1`. This is a local private composition only; no new transport,
public REST/OpenAPI/SDK write method, Web mutation, migration, provider, secret, release, or
production claim was added.

新鲜本地 evidence 已通过 `context-lifecycle-editor.test.tsx` `7/7`、`local-branch-heads-graph-review.test.tsx` `5/5`、`pnpm check:web`（public SDK `15`、
local SDK `99`、Web `207`、TypeScript/lint 与 production build）、`cargo test --workspace --quiet --no-fail-fast`（storage `193 passed, 39 ignored`）、
`cargo fmt --all -- --check`、strict offline workspace Clippy、锁定 Rust `1.85.0` check 与 static `impl GraphDiff` count `1`。这只是
local private composition；没有新增 transport、public REST/OpenAPI/SDK write method、Web mutation、migration、provider、secret、release
或 production 声明。

Docker/PostgreSQL runtime, authenticated browser, Git change-set, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`. The long-term goal remains active; the
next increment must begin with a new bilingual Necessity Record and must close a named local
completion criterion rather than expand the public surface.

Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production 继续为
`unobserved` 或 `deferred`。长期目标保持 active；下一增量必须先新增双语 Necessity Record，并且必须收束命名的本地完成条件，
不得扩大 public surface。

### 2026-07-29 Private Capability Availability Schema Hardening / 2026-07-29 私有 Capability Availability Schema 硬化

The private numeric V1 capability-availability parser is now schema-closed. A red test reproduced
the prior `Missing expected exception` for an unknown outer field. The parser now allowlists the
outer V1 fields and nested bilingual `en`/`zh` fields, preserving the existing frozen DTO,
presenter, screen states, and transport boundary while failing closed on shape drift.

私有 numeric V1 capability-availability parser 现已完成 schema-closed 硬化。红测以 unknown outer field 复现此前的
`Missing expected exception`。parser 现对 outer V1 field 与 nested bilingual `en`/`zh` field 使用 allowlist，在保持既有
frozen DTO、presenter、screen state 与 transport boundary 的同时，对 shape drift fail closed。

Fresh local evidence passed the focused parser test (`3 passed`), `pnpm check:web` (public SDK
`15`, local SDK `99`, Web `201`, TypeScript/lint, production build), Rust format, workspace tests
(storage `193 passed, 39 ignored`), strict offline Clippy, and locked Rust `1.85.0` check. Static
inspection observed one `impl GraphDiff`; public SDK branch-head hits and retired public
commit-graph-diff read hits were both `0`.

新鲜本地证据已通过 focused parser test（`3 passed`）、`pnpm check:web`（public SDK `15`、local SDK `99`、Web `201`、
TypeScript/lint 与 production build）、Rust format、workspace test（storage `193 passed, 39 ignored`）、strict offline Clippy
与锁定 Rust `1.85.0` check。静态 inspection 观测到一个 `impl GraphDiff`；public SDK branch-head hit 与 retired public
commit-graph-diff read hit 均为 `0`。

This is a private contract-hardening increment only. No API/OpenAPI/SDK method, Web transport,
mutation, provider, migration, secret, Docker/PostgreSQL runtime, authenticated browser, Git,
remote CI, operator rehearsal, release, or production evidence was added. The long-term goal
remains `active`; the next increment requires a new bilingual Necessity Record.

本增量仅硬化 private contract。没有新增 API/OpenAPI/SDK method、Web transport、mutation、provider、migration、secret、
Docker/PostgreSQL runtime、authenticated browser、Git、remote CI、operator rehearsal、release 或 production evidence。长期
目标保持 `active`；下一增量必须先新增双语 Necessity Record。

### 2026-07-29 Private Branch-Head Graph Review Selection / 2026-07-29 私有 Branch-Head 图谱审阅选择

The private branch-head selection composition is now implemented and freshly verified. The
existing branch-head inspector callback passes the exact non-null server-owned `head_commit_id`
into `LocalBranchHeadsGraphReview`, which adds it to the existing graph-review candidates without
duplicates and makes it the revised-commit default. Unborn/null heads preserve the existing review
defaults. The keyed composition remounts the review when the Context or selected head changes, so
the prior review state is not reused across that selection boundary.

私有 branch-head selection composition 现已实现并完成新鲜验证。既有 branch-head inspector callback 将 exact、非 null 的
server-owned `head_commit_id` 传入 `LocalBranchHeadsGraphReview`；它把该 ID 加入既有 graph-review candidates，不产生
duplicate，并将其设为 revised-commit default。Unborn/null head 保持原有 review defaults。带 key 的 composition 会在
Context 或 selected head 变化时重新挂载 review，避免跨 selection boundary 复用旧 review state。

Fresh local evidence passed the focused branch-head/graph-review command (`8 passed`), `pnpm
check:web` (public SDK `15`, local SDK `99`, Web `200`, TypeScript/lint, production build), Rust
format, workspace tests (storage `193 passed, 39 ignored`), strict offline Clippy, and locked
Rust `1.85.0` check. Static inspection observed one `impl GraphDiff`; public SDK branch-head and
retired public commit-graph-diff read hits were both `0`.

新鲜本地证据已通过 focused branch-head/graph-review command（`8 passed`）、`pnpm check:web`（public SDK `15`、local SDK `99`、
Web `200`、TypeScript/lint 与 production build）、Rust format、workspace test（storage `193 passed, 39 ignored`）、strict
offline Clippy 与锁定 Rust `1.85.0` check。静态 inspection 观测到一个 `impl GraphDiff`；public SDK branch-head 与 retired
public commit-graph-diff read hit 均为 `0`。

This remains a private read-only local composition. No REST/OpenAPI/public SDK method, branch
mutation, merge/rollback, Web mutation, provider, migration, secret, Docker/PostgreSQL runtime,
authenticated browser, Git change-set, remote CI, operator rehearsal, release, or production
evidence was added. The long-term goal remains `active`; the next increment requires a new
bilingual Necessity Record.

本切片仍是 private read-only local composition。没有新增 REST/OpenAPI/public SDK method、branch mutation、merge/rollback、
Web mutation、provider、migration、secret、Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、
operator rehearsal、release 或 production evidence。长期目标保持 `active`；下一增量必须先新增双语 Necessity Record。

### 2026-07-30 Private Versioned ContextGraph Merge Review (recorded after persisted review) / 2026-07-30 私有版本化 ContextGraph Merge Review（在持久化审阅之后记录）

The private version-bound graph review contract is now implemented and locally verified.
`VersionedContextGraphSnapshotV1` binds owned graph data to exact
`(ProjectId, ContextId, CommitId)` identity. Its V1 request and projection preserve the matching
`MergePlan`, deterministic `GraphMergeClassification`, and explicit schema version. The service
rejects nil or duplicate snapshot identities and delegates only to `GraphMergeConflictClassifier`.
The storage review service is now a thin adapter over this contract and retains its existing
private repository shape.

私有 version-bound graph review contract 现已实现并完成本地验证。`VersionedContextGraphSnapshotV1` 将 owned graph data 绑定到
exact `(ProjectId, ContextId, CommitId)` identity。其 V1 request 与 projection 保留匹配的 `MergePlan`、确定性的
`GraphMergeClassification` 与显式 schema version。service 拒绝 nil 或 duplicate snapshot identity，并只委托
`GraphMergeConflictClassifier`。storage review service 现是该 contract 上的薄 adapter，保持原有 private repository shape。

Fresh local evidence passed focused diff-engine `4`, focused storage `8`, workspace Rust `186 passed,
39 ignored`, format, strict offline Clippy, locked Rust `1.85.0`, and `pnpm check:web` with public
SDK `15`, local SDK `92`, Web `188`, and production build. Static inspection observed one
`impl GraphDiff`. The storage adapter now reads base/left/right through one batch contract: Memory
uses one guard and PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction, while the versioned
service remains the sole classifier path. PostgreSQL/Docker runtime, browser, Git, remote CI,
operator, release, and production remain `unobserved` or `deferred`; no public write or transport
was added. The long-term goal remains active; the next increment requires a new bilingual Necessity
Record.

新鲜 local evidence 已通过 focused diff-engine `4`、focused storage `8`、workspace Rust `186 passed, 39 ignored`、format、
strict offline Clippy、锁定 Rust `1.85.0`，以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `188` 与 production
build）。静态 inspection 观测到一个 `impl GraphDiff`。storage adapter 现通过一个 batch contract 读取 base/left/right：
Memory 使用一个 guard，PostgreSQL 使用一个 `REPEATABLE READ READ ONLY` transaction，versioned service 仍是唯一
classifier path。PostgreSQL/Docker runtime、browser、Git、remote CI、operator、release 与 production 继续为
`unobserved` 或 `deferred`；未新增 public write 或 transport。长期目标保持 active；下一增量必须先新增双语 Necessity Record。

### 2026-07-30 Private Persisted ContextGraph Merge Review / 2026-07-30 私有持久化 ContextGraph Merge Review

The private storage bridge is now implemented and locally verified. `ContextMergeInputScope`
binds one project/Context to three distinct base/left/right commits. `PersistedContextGraphMergeReviewService`
rejects non-three-way or plan-identity drift before repository reads, reads the exact persisted
base/left/right snapshots, fails closed for missing or returned-scope-drifted records, and
delegates classification only to `GraphMergeConflictClassifier`. The bridge remains read-only and
does not create a merged graph, mutate a branch, or expose transport.

私有 storage bridge 现已实现并完成本地验证。`ContextMergeInputScope` 将一个 project/Context 绑定到三个不同的
base/left/right commit。`PersistedContextGraphMergeReviewService` 在 repository read 前拒绝 non-three-way 或
plan-identity drift，读取 exact persisted base/left/right snapshot，遇到 missing 或 returned-scope drift 时 fail closed，
并只委托 `GraphMergeConflictClassifier` 完成 classification。本 bridge 仍为 read-only，不创建 merged graph、不修改
branch，也不暴露 transport。

Focused storage coverage passed `5` tests for exact three-side delegation, missing-side failure,
duplicate commit rejection, repository scope drift, and non-three-way rejection. The broader
workspace receipt passed Rust `186 passed, 39 ignored`, formatting, strict offline Clippy, locked
Rust `1.85.0`, and `pnpm check:web` with public SDK `15`, local SDK `92`, Web `185`, and the
production build. Static inspection still finds one `impl GraphDiff`. PostgreSQL runtime,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`; no secret, public write, migration, Web mutation, or second calculator
was added.

Focused storage coverage 通过 `5` 项测试，覆盖 exact three-side delegation、missing-side failure、duplicate commit
rejection、repository scope drift 与 non-three-way rejection。更大 workspace 回执通过 Rust `186 passed, 39 ignored`、
formatting、strict offline Clippy、锁定 Rust `1.85.0`，以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185`
与 production build）。静态 inspection 仍只发现一个 `impl GraphDiff`。PostgreSQL runtime、authenticated browser、Git、
remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未新增 secret、public write、
migration、Web mutation 或第二个 calculator。

The PostgreSQL implementation now overrides the batch repository port with one
`REPEATABLE READ READ ONLY` transaction, while the Memory adapter uses one read guard and existing
repository doubles retain a compatible default. This closes the previously documented cross-read
consistency gap at the local contract boundary, but is not PostgreSQL runtime or production
evidence. It advances Criteria 2 and 4 without closing either; the long-term goal remains active.
The next increment requires a new bilingual Necessity Record for a dependency-ready private
Context editing, benchmark, or replay contract.

PostgreSQL implementation 现已通过一个 `REPEATABLE READ READ ONLY` transaction override batch repository port；Memory
adapter 使用一个 read guard，既有 repository double 则保留兼容的 default。此前记录的 cross-read consistency gap 已在
local contract boundary 收束，但这不是 PostgreSQL runtime 或 production evidence。本增量推进条件 2 与 4，但不关闭其中
任何一项；长期目标保持 active。下一增量必须先为依赖就绪的 private Context editing、benchmark 或 replay contract 新增
双语 Necessity Record。

### 2026-07-28 Private Commit-Scoped Diff Snapshot Persistence / 2026-07-28 私有按 Commit 绑定的 Diff Snapshot 持久化

The admitted storage wave is locally implemented and verified. `ContextDiffSnapshotV1Record`
binds the validated semantic/behavior/evaluation `ContextDiffSnapshotV1` to exact
`(ProjectId, ContextId, CommitId, schema_version)` through `VersionedContextScopeV1`, stores a
deterministic `sha256:` digest and microsecond-normalized capture time, and preserves immutable
create/replay/conflict semantics. Memory and PostgreSQL adapters share the repository contract;
migration `0023_context_diff_snapshots.sql` enforces composite scope, fixed schema, and append-only
behavior. The private `PersistedContextDiffReviewService` now consumes only two exact persisted
records, validates returned scope/schema, and delegates the complete comparison to the existing
`VersionedContextDiffReviewService`; no caller-supplied snapshot path was promoted.

本轮准入的 storage wave 已在本地实现并验证。`ContextDiffSnapshotV1Record` 通过 `VersionedContextScopeV1` 将已校验的
semantic/behavior/evaluation `ContextDiffSnapshotV1` 绑定到 exact `(ProjectId, ContextId, CommitId, schema_version)`，
保存确定性的 `sha256:` digest 与微秒规范化 capture time，并保持 immutable create/replay/conflict 语义。Memory 与
PostgreSQL adapter 共享 repository contract；migration `0023_context_diff_snapshots.sql` 强制 composite scope、固定
schema 与 append-only behavior。私有 `PersistedContextDiffReviewService` 现只消费两个 exact persisted record，校验返回的
scope/schema，并委托既有 `VersionedContextDiffReviewService` 完成比较；没有推广 caller-supplied snapshot path。

Fresh local evidence passed: focused Memory `4 passed`, migration contract `2 passed`, PostgreSQL adapter contract
`1 passed, 1 ignored`; workspace Rust API `183 passed` and storage `179 passed, 39 ignored`; format; strict offline
workspace Clippy; locked Rust `1.85.0` workspace check; and `pnpm check:web` with public SDK `15`, local SDK `92`,
Web `185`, and production build. PostgreSQL runtime, authenticated browser, Git binding/change-set, remote CI, operator,
release, and production remain `unobserved` or `deferred`. No secret, Docker runtime, provider call, public transport,
Web mutation, or second GraphDiff calculator was added. This advances Criteria 2 and 4 without closing either; the
long-term goal remains active.

新鲜 local evidence 已通过：focused Memory `4 passed`、migration contract `2 passed`、PostgreSQL adapter contract
`1 passed, 1 ignored`；workspace Rust（API `183 passed`、storage `179 passed, 39 ignored`）；format；strict offline
workspace Clippy；锁定 Rust `1.85.0` workspace check；以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185`，
并完成 production build）。PostgreSQL runtime、authenticated browser、Git binding/change-set、remote CI、operator、
release 与 production 仍为 `unobserved` 或 `deferred`。未读取 secret、未运行 Docker runtime、未调用 provider、未新增
public transport、Web mutation 或第二个 GraphDiff calculator。本增量推进条件 2 与 4，但不关闭其中任何一项；长期
目标保持 active。

**Current review adapter / 当前 review adapter:** The storage-backed semantic/behavior/evaluation
review composition is implemented and locally verified through the new bilingual plan
`docs/superpowers/plans/2026-07-29-private-persisted-diff-review.md`. Its scope-only service reads
source and target once, fails closed with side-aware redacted errors, and delegates only to the
existing version-bound review service. It remains a private Rust boundary; no REST/OpenAPI/SDK/Web
surface changed.

**Next admitted decision / 下一项准入决策:** Select the next dependency-ready private benchmark or
Context editing increment only after its own bilingual Necessity Record. Do not add transport or UI
just to expose this storage service; the local protected graph-diff read remains the current read
surface and the long-term goal remains active.

**当前 review adapter：** storage-backed semantic/behavior/evaluation review composition 已通过新的双语计划
`docs/superpowers/plans/2026-07-29-private-persisted-diff-review.md` 实现并完成本地验证。其 scope-only service 只读取
source 与 target 各一次，以带 side 信息且脱敏的错误 fail closed，并只委托既有 version-bound review service。它仍是
private Rust boundary；没有改变 REST/OpenAPI/SDK/Web surface。

**下一项准入决策：** 只有在新增双语 Necessity Record 后，才选择下一个依赖就绪的 private benchmark 或 Context editing
增量。不得为了暴露该 storage service 而新增 transport 或 UI；当前 protected graph-diff read 仍是 read surface，长期目标
保持 active。

### Private Benchmark Regression, Scorecard, and Evaluation-Diff Closure / 私有 Benchmark 回归、Scorecard 与 Evaluation-Diff 收束

This local benchmark closure is now verified at the contract boundary. The evaluation domain already
owns the single `BenchmarkSuite -> Scorecard -> RegressionDecision` policy path, while storage tests
now prove same-commit exact decision comparison and a complete redacted workspace projection with
baseline/revised scope, values, status, stable ordering, and fail-closed scope checks. The local SDK
and Web audit found no implementation gap; Web continues to present server-owned conclusions through
`data -> presenter -> screen` without recalculating policy or diff.

本地 benchmark 收束现已在 contract boundary 完成验证。evaluation domain 继续拥有唯一的
`BenchmarkSuite -> Scorecard -> RegressionDecision` policy path；storage tests 现证明 same-commit exact decision
comparison，以及包含 baseline/revised scope、value、status、稳定排序与 fail-closed scope check 的完整脱敏 workspace
projection。local SDK 与 Web audit 未发现实现缺口；Web 继续通过 `data -> presenter -> screen` 呈现 server-owned
conclusion，不在页面内重新计算 policy 或 diff。

Fresh local evidence is evaluation `45 passed`, storage benchmark evidence `23 passed`, workspace
projection `7 passed`, execution `11 passed`, API `183 passed`, workspace Rust `179 passed, 39 ignored`,
format, strict offline workspace Clippy, locked Rust `1.85.0`, and `pnpm check:web` with public SDK
`15`, local SDK `92`, Web `185`, and production build. Static inspection observed exactly one
`impl GraphDiff`; the public SDK/OpenAPI retirement assertions passed. The direct ad hoc Vitest command
remains unobserved because no direct workspace binary exists; the repository-owned Web command passed.

新鲜 local evidence 为 evaluation `45 passed`、storage benchmark evidence `23 passed`、workspace projection `7 passed`、
execution `11 passed`、API `183 passed`、workspace Rust `179 passed, 39 ignored`、format、strict offline workspace
Clippy、锁定 Rust `1.85.0`，以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185` 与 production build）。
静态 inspection 观测到恰好一个 `impl GraphDiff`；public SDK/OpenAPI retirement assertion 通过。直接调用 Vitest
因 workspace 没有 direct binary 仍为 unobserved；仓库既有 Web command 已通过。

This advances the local benchmark/evaluation portions of Criteria 3 and 4 without closing the
repository. PostgreSQL runtime, authenticated browser runtime, Git change-set, remote CI, operator
rehearsal, release, and production remain `unobserved` or `deferred`; no secret, Docker runtime,
provider call, public write, migration, or public surface was added. The long-term goal remains
active. The next candidate is private typed branch-head discovery as a read-only prerequisite for
future branch/merge/replay, but it requires its own bilingual Necessity Record before implementation.

本增量推进条件 3 与 4 的本地 benchmark/evaluation 部分，但不关闭仓库。PostgreSQL runtime、authenticated browser
runtime、Git change-set、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；没有
新增 secret、Docker runtime、provider call、public write、migration 或 public surface。长期目标保持 active。下一候选
是作为未来 branch/merge/replay 只读前置的 private typed branch-head discovery，但实现前必须先具备其独立的双语
Necessity Record。

## 2026-08-02 Private Context Graph Review Witness: Integrated Local Receipt / 2026-08-02 私有 Context Graph 审查见证：集成本地回执

Necessity Record / 必要性记录: This increment serves Criteria 1, 2, and 4. The missing evidence was
that commit history, branch heads, and source/target graph snapshots were assembled from one
backend-owned observation boundary. The minimum scope was storage witness contracts, Memory and
PostgreSQL adapters, protected-route wiring, focused tests, and bilingual documentation. It did
not add public writes, transport, SDK methods, Web mutations, migrations, providers, or release claims.
/ 本增量服务条件 1、2、4。缺口是 commit history、branch heads 与 source/target graph snapshot 尚未证明来自同一个
backend-owned observation boundary。最小范围是 storage witness contract、Memory/PostgreSQL adapter、protected route 接线、
focused tests 与双语文档；未新增 public write、transport、SDK method、Web mutation、migration、provider 或 release 声明。

Implementation / 实现: `ContextGraphReviewWitnessRepository` now returns a validated complete history
and exact snapshot pair. Memory uses one read guard; PostgreSQL uses one `REPEATABLE READ READ ONLY`
transaction. The protected version-backed route fails closed without the witness dependency, while
custom fixtures inject the repository explicitly. `GraphDiff::between` remains the sole calculator.
/ `ContextGraphReviewWitnessRepository` 现在返回经过校验的完整 history 与 exact snapshot pair。Memory 使用单一 read guard；
PostgreSQL 使用单一 `REPEATABLE READ READ ONLY` transaction。protected version-backed route 缺少 witness dependency 时
fail closed；custom fixture 显式注入 repository。`GraphDiff::between` 仍是唯一 calculator。

Fresh integrated evidence / 新鲜集成证据（2026-08-02T15:02:14+08:00 receipt window / 回执窗口）:

- storage composition `4 passed`; PostgreSQL contract `2 passed, 1 ignored`;
- protected API focused `7 passed`; exact-scope API integration `4 passed`;
- workspace Rust passed with storage `233 passed, 41 ignored`;
- `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust `1.85.0` check passed;
- `pnpm check:web`: public SDK `15`, local SDK `148`, Web `298`, production build passed;
- local contract fixture verifier passed; `GRAPH_DIFF_IMPL_COUNT=1`.

新鲜集成证据（2026-08-02T15:02:14+08:00 回执窗口）：storage composition `4 passed`；PostgreSQL contract
`2 passed, 1 ignored`；protected API focused `7 passed`；exact-scope API integration `4 passed`；workspace Rust 通过且
storage 为 `233 passed, 41 ignored`；`cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0` check 通过；
`pnpm check:web` 的 public SDK `15`、local SDK `148`、Web `298` 与 production build 通过；local contract fixture verifier
通过；`GRAPH_DIFF_IMPL_COUNT=1`。

This is local non-production evidence and advances but does not close Criteria 1, 2, or 4. Live
PostgreSQL, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release,
production, and public protected-write readiness remain `ignored`, `unobserved`, or `deferred`. The
long-term goal remains `active`; the next increment requires a fresh bilingual Necessity Record.
/ 这是本地非生产 evidence，推进但不关闭条件 1、2、4。live PostgreSQL、Docker、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、`unobserved` 或
`deferred`。长期目标保持 `active`；下一项增量必须先有新的双语 Necessity Record。

## 2026-08-02 Private Context Graph Review Witness / 2026-08-02 私有 Context Graph 审查见证

This bounded Criterion 1/2/4 increment is `completed / verified locally`; the integrated receipt
above supersedes its focused-only evidence, and the long-term goal remains `active`.
`ContextGraphReviewWitnessRepository` now binds complete `CommitHistory` (and
branch heads) with exact source and target `CommitGraphSnapshot` values. Memory uses one read guard;
PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction. The existing versioned review service
then receives the pair and remains the only path that calculates `GraphDiff::between`.

本有界条件 1/2/4 增量现为 `implemented / focused-verified`；长期目标继续 `active`。
`ContextGraphReviewWitnessRepository` 现将完整 `CommitHistory`（包括 branch heads）与 exact source/target
`CommitGraphSnapshot` 绑定。Memory 使用单一 read guard；PostgreSQL 使用单一 `REPEATABLE READ READ ONLY` transaction。
随后由既有 versioned review service 接收 pair，`GraphDiff::between` 仍只有这一条计算路径。

The environment-backed protected graph-diff read injects the witness repository without changing
the route, response, OpenAPI, public SDK, local SDK, BFF, or Web shapes. A focused Memory contract
observed `4 passed`; PostgreSQL SQL-shape/lazy contract observed `2 passed` with one runtime test
`ignored`; protected API graph-diff observed `7 passed`. The witness rejects cross-project scope and
the API keeps scope drift fail-closed as the existing unavailable response.

environment-backed protected graph-diff read 已注入 witness repository，但没有改变 route、response、OpenAPI、public SDK、
local SDK、BFF 或 Web shape。Memory focused contract 观测到 `4 passed`；PostgreSQL SQL-shape/lazy contract 观测到
`2 passed`，另有一个 runtime test 为 `ignored`；protected API graph-diff 观测到 `7 passed`。witness 拒绝 cross-project
scope，API 继续将 scope drift fail-closed 为既有 unavailable response。

This is local non-production evidence only. PostgreSQL runtime, Docker, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, and production remain `ignored`, `unobserved`, or
`deferred`. No public write, migration, provider, secret access, operator transport, Web mutation, or
second GraphDiff calculator was added. The next implementation still requires a fresh bilingual
Necessity Record; the long-term goal must not be closed by this feature slice.

本回执仅是本地非生产 evidence。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator
rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。未新增 public write、migration、provider、
secret access、operator transport、Web mutation 或第二个 GraphDiff calculator。下一项实现仍需新的双语 Necessity Record；
不得因本功能切片关闭长期目标。

### 2026-07-28 Protected Local Commit Graph-Diff Read / 2026-07-28 受保护的本地 Commit Graph-Diff 读取

The protected-local commit graph-diff read increment is complete at the local contract boundary. The
public version-backed GET alias, its checked-in OpenAPI operation, and its public TypeScript SDK method
are removed. The read is available only through the default-off
`GET /api/v1/local/contexts/{context_id}/graph-diff` route, the non-public local SDK, and the same-origin
Bearer-only BFF. Authentication, `ContextPermission::Read`, audit, operation quota, exact
project/Context/commit scope, structured errors, and `Cache-Control: private, no-store` remain on the
protected boundary. The public pure `POST /api/v1/graph-diffs` calculation is a separate contract and
remains; `GraphDiff::between` is still the sole graph-diff calculator. The server-rendered workspace
does not substitute a server credential and keeps the capability unavailable until an authenticated
local request exists.

本轮 protected-local commit graph-diff read 增量已在 local contract boundary 完成。public version-backed GET alias、
checked-in OpenAPI operation 与 public TypeScript SDK method 已移除。读取现在只通过默认关闭的
`GET /api/v1/local/contexts/{context_id}/graph-diff` route、non-public local SDK 与同源 Bearer-only BFF 提供。
protected boundary 继续承担 authentication、`ContextPermission::Read`、audit、operation quota、精确
project/Context/commit scope、structured errors 与 `Cache-Control: private, no-store`。独立的 public pure
`POST /api/v1/graph-diffs` calculation 属于另一份 contract，仍然保留；`GraphDiff::between` 仍是唯一 graph-diff
calculator。server-rendered workspace 不替代 server credential，在 authenticated local request 出现前保持 capability
unavailable。

Fresh local evidence passed: `cargo fmt --all -- --check`; the focused typed-scope contract `2 passed`;
workspace Rust tests with API `183 passed` and storage `179 passed, 39 ignored`; strict offline workspace
Clippy; `pnpm check:web` with public SDK `15 passed`, non-public local SDK `92 passed`, Web `185 passed`,
and a successful production build; and the static verifier with `local_contract_source=passed`,
`graph_diff_application=passed count=1`, `safe_local_dto_fields=passed`, and public-surface/local-SDK
checks passed. The verifier reports `public_write_additions=unobserved` because no unified diff was supplied,
and `overall=unobserved` because it does not read Git evidence. PostgreSQL runtime, authenticated browser
runtime, Git binding/change-set evidence, remote CI, operator rehearsal, release, and production promotion
were not observed; PostgreSQL, browser, Git, and remote runtime evidence remain `unobserved`, while release
and production promotion remain `deferred`. No secrets, provider call, Docker runtime, public write, or
long-term completion claim is made. This advances Criteria 1, 2, 4, and 6 but closes none of them; the
long-term goal remains active.

新鲜 local evidence 已通过：`cargo fmt --all -- --check`；focused typed-scope contract `2 passed`；workspace Rust test，
其中 API `183 passed`、storage `179 passed, 39 ignored`；strict offline workspace Clippy；`pnpm check:web`，其中
public SDK `15 passed`、non-public local SDK `92 passed`、Web `185 passed`，并成功完成 production build；以及 static
verifier，其中 `local_contract_source=passed`、`graph_diff_application=passed count=1`、`safe_local_dto_fields=passed`，
public-surface/local-SDK checks 均通过。由于未提供 unified diff，verifier 将 `public_write_additions` 标为
`unobserved`，并因不读取 Git evidence 将 `overall` 标为 `unobserved`。PostgreSQL runtime、authenticated browser
runtime、Git binding/change-set evidence、remote CI、operator rehearsal、release 与 production promotion 未被观测；
PostgreSQL、browser、Git 与 remote runtime evidence 仍为 `unobserved`，release 与 production promotion 仍为 `deferred`。
本轮未读取 secrets、未调用 provider、未运行 Docker runtime、未新增 public write，也不作长期目标完成声明。本增量推进
条件 1、2、4、6，但不关闭其中任何一项；长期目标保持 active。

### 2026-07-27 Private Versioned Context Diff Contract / 2026-07-27 私有版本化 Context Diff 契约

The current local receipt supersedes the older "latest" headings below. `VersionedContextScopeV1`
now binds private semantic/behavior/evaluation review inputs and outputs to exact
`(ProjectId, ContextId, CommitId)` identities. The review contract rejects cross-project,
cross-Context, and identical-scope comparisons before calculating a result; it keeps
`ContextDiffService` as the unified comparison path and `GraphDiff::between` as the sole graph
calculator. The admitted EOF-newline correction ensures a semantic document change cannot result
in an empty `TextDiff`.

当前本地回执取代下方较早的“latest”标题。`VersionedContextScopeV1` 现将 private semantic/behavior/evaluation
review 的输入与输出绑定到精确的 `(ProjectId, ContextId, CommitId)` 身份。该 review contract 会在计算结果前拒绝
cross-project、cross-Context 与 identical-scope 比较；继续保持 `ContextDiffService` 为统一 comparison path，
`GraphDiff::between` 为唯一 graph calculator。已准入的 EOF-newline 修复确保 semantic document change 不会产生空的
`TextDiff`。

Fresh local evidence passed: diff-engine `20/20`; workspace Rust with API `184 passed` and storage
`179 passed, 39 ignored`; format; strict offline Clippy; Rust `1.85.0` workspace check; and
`pnpm check:web` with public SDK `14`, local SDK `85`, Web `179`, and a production build. The static
contract verifier reports its graph-diff application check as passed; Git, browser, PostgreSQL,
remote CI, operator, release, and production evidence remain `unobserved` or `deferred` unless
directly observed. No public write, OpenAPI/SDK write method, Web mutation, secret access, or
second GraphDiff calculator was added. This advances Criteria 2 and 4 without closing either or the
long-term goal.

新鲜本地证据均通过：diff-engine `20/20`；workspace Rust（API `184 passed`、storage `179 passed, 39 ignored`）；
format；strict offline Clippy；Rust `1.85.0` workspace check；以及 `pnpm check:web`（public SDK `14`、local SDK `85`、
Web `179`，并完成 production build）。静态 contract verifier 的 graph-diff application check 已通过；Git、browser、
PostgreSQL、remote CI、operator、release 与 production evidence 在直接观测前仍为 `unobserved` 或 `deferred`。没有新增
public write、OpenAPI/SDK write method、Web mutation、secret access 或第二个 GraphDiff calculator。本增量推进条件 2 与
条件 4，但不关闭其中任何一项或长期目标。

**Historical admitted decision / 历史准入决策（superseded）：** Establish a new bilingual Necessity Record for the
existing version-backed graph-diff read access boundary before changing it. That decision has since
been resolved by the protected-local commit graph-diff read receipt below; it is retained here as a
historical receipt and is no longer the current pointer.

**历史准入决策（已被后续状态取代）：** 在修改既有 version-backed graph-diff read access boundary 前，先建立新的双语
Necessity Record。该决策已由下方 protected-local commit graph-diff read 回执收束；本段作为历史回执保留，不再是当前指针。

### 2026-07-27 Private Benchmark Execution Web Adapter / 2026-07-27 私有 Benchmark 执行 Web 适配器

This is the latest local integration receipt for the already-admitted protected Benchmark execution
adapter. The non-public local SDK now owns the V1 request/receipt parser and client, with canonical
UUID validation, the shared `0.0..=2.0` temperature range, deterministic ordered-unique dataset IDs,
request-memory Bearer credentials, `credentials: "omit"`, `cache: "no-store"`, and typed immutable
conflict errors. The same-origin BFF forwards only the exact project/Context/commit scope and
allowlisted bilingual error codes; upstream diagnostic messages and unknown receipt fields are
redacted. The shared Web `data -> presenter -> screen` path and default-off execution inspector are
mounted in the existing Benchmark workspace region behind `localLifecycleEnabled`. The inspector
only displays server-owned redacted receipt metadata and never computes evaluation policy or Diff.

这是已准入 protected Benchmark execution adapter 的最新本地集成回执。非公开 local SDK 现负责 V1 request/receipt
parser 与 client，执行 canonical UUID 校验、共享的 `0.0..=2.0` temperature 范围、确定性的有序唯一 dataset ID、
request-memory Bearer credential、`credentials: "omit"`、`cache: "no-store"` 与 typed immutable conflict error。
同源 BFF 只转发精确 project/Context/commit scope 与 allowlisted 双语 error code；上游 diagnostic message 与未知
receipt field 均被脱敏。共享 Web `data -> presenter -> screen` 路径与默认关闭的 execution inspector 已挂载到现有
Benchmark workspace region，并受 `localLifecycleEnabled` gate 控制。inspector 只呈现服务端控制的脱敏 receipt metadata，
不计算 evaluation policy 或 Diff。

Fresh local evidence passed: local SDK `73` tests and typecheck; Web `173` tests and typecheck;
the full `pnpm check:web` gate with public SDK `14`, local SDK `73`, Web `173`, and production
build; `cargo fmt --all -- --check`; `cargo test --workspace --quiet` with API `176` and storage
`169 passed, 39 ignored`; strict workspace Clippy; and locked Rust `1.85.0` workspace check.
No public REST/OpenAPI/public SDK write, provider call, secret access, Docker/PostgreSQL runtime,
authenticated browser runtime, Git binding, remote CI, operator rehearsal, release, or production
claim was added. PostgreSQL runtime and authenticated browser-to-BFF-to-Axum evidence remain
unobserved; external release evidence remains deferred. Criterion 3 is advanced but not closed and
the long-term goal remains active.

新鲜本地证据均已通过：local SDK `73` 项测试与 typecheck；Web `173` 项测试与 typecheck；完整 `pnpm check:web` 门禁，
其中 public SDK `14`、local SDK `73`、Web `173` 且 production build 成功；`cargo fmt --all -- --check`；
`cargo test --workspace --quiet`，其中 API `176`、storage `169 passed, 39 ignored`；strict workspace Clippy；
以及锁定 Rust `1.85.0` 的 workspace check。没有新增 public REST/OpenAPI/public SDK write、provider call、
secret access、Docker/PostgreSQL runtime、authenticated browser runtime、Git binding、remote CI、operator rehearsal、
release 或 production claim。PostgreSQL runtime 与 authenticated browser-to-BFF-to-Axum evidence 仍为 unobserved；
外部 release evidence 仍延期。条件 3 得到推进但尚未关闭，长期目标保持 active。

**Next admitted increment / 下一项准入增量：** Close the next Criterion 1/3 local gap by exposing
provider-free Knowledge/Memory citation and retention projections through exact private read contracts
and shared Web presenters, after a new bilingual Necessity Record. The next increment must not add
public routes, provider access, raw private content, or client-side retrieval policy.

**下一项准入增量：** 在新增双语 Necessity Record 后，通过精确 private read contract 与共享 Web presenter 暴露
provider-free Knowledge/Memory citation 与 retention projection，收束条件 1/3 的下一项本地缺口。该增量不得新增
public route、provider access、raw private content 或 client-side retrieval policy。

### 2026-07-23 Benchmark Web and Durable Execution Projection / 2026-07-23 Benchmark Web 与持久执行投影

This is the authoritative latest local increment. It supersedes the current-status and next-increment statements in the preserved Benchmark workspace entry immediately below: the real Web BFF/`data -> presenter -> screen` integration and the `BenchmarkExecutionService` production application caller are complete. The product boundary remains a private, local, read-only workflow. The browser keeps the bearer token only in request memory and sends it in the `Authorization` header to the same-origin BFF; cookies are not used or forwarded, upstream requests use `credentials: "omit"`, and responses are `private, no-store`. The Web consumes a server-owned redacted projection and does not calculate benchmark policy. The default public router, checked-in public OpenAPI, and public TypeScript SDK remain unchanged; `GraphDiff::between` remains the sole graph-diff calculator.

这是当前权威的最近本地增量。它取代紧邻下方保留的 Benchmark workspace 条目中的当前状态与下一增量判断：真实 Web BFF/`data -> presenter -> screen` 集成，以及由 `BenchmarkExecutionService` 承担的 production application caller 均已完成。产品边界仍严格限定为 private、local、read-only workflow。浏览器只在请求内存中保存 Bearer token，并通过 `Authorization` header 发送给同源 BFF；不使用或转发 cookie，上游请求采用 `credentials: "omit"`，response 为 `private, no-store`。Web 消费 server-owned 的脱敏 projection，不计算 benchmark policy。默认 public router、已检入的 public OpenAPI 与 public TypeScript SDK 均未改变；`GraphDiff::between` 仍是唯一 graph-diff calculator。

For both created and replayed evidence, `BenchmarkExecutionService` reloads the exact sealed suite/dataset definitions and sealed runs, reconstructs the deterministic execution receipt, and materializes through the existing `BenchmarkWorkspaceProjectionV1Writer` via `persist_benchmark_workspace_projection`. If the projection write fails after evidence sealing, the next evidence-backed replay repairs it without invoking the evaluator again. PostgreSQL-compatible microsecond timestamp normalization preserves replay identity, and memory/PostgreSQL adapters implement the same producer contract and projection semantics.

对于 created 与 replayed evidence，`BenchmarkExecutionService` 都会重新加载精确的 sealed suite/dataset definition 与 sealed run，重建确定性的 execution receipt，并经由 `persist_benchmark_workspace_projection` 使用既有 `BenchmarkWorkspaceProjectionV1Writer` 完成物化。若 evidence 封存后 projection write 失败，下一次基于 evidence 的 replay 会在不再次调用 evaluator 的情况下完成修复。与 PostgreSQL 兼容的微秒级 timestamp normalization 保持 replay identity，memory/PostgreSQL adapter 实现相同的 producer contract 与 projection 语义。

Fresh local evidence passed: evaluation-focused `15/15`; storage-focused `38/38`; the confirmed full storage suite `166 passed, 38 ignored`; `pnpm check:web` with public SDK `14`, local SDK `59`, Web `138`, and the full production Web build; workspace API `162 passed`; strict workspace Clippy; the locked Rust `1.85.0` workspace check; both required contract verifiers; and desktop/mobile Playwright with no horizontal overflow or console errors. One PostgreSQL 16.14 `SQL_ASCII` producer runtime test passed on a loopback-only server, which was then stopped. That result is focused runtime evidence, not production encoding readiness.

新鲜本地证据均已通过：evaluation 聚焦测试 `15/15`；storage 聚焦测试 `38/38`；已确认的 storage 全量 `166 passed, 38 ignored`；`pnpm check:web`（public SDK `14`、local SDK `59`、Web `138`，并完成完整 production Web build）；workspace API `162 passed`；严格 workspace Clippy；锁定 Rust `1.85.0` 的 workspace check；两项所需 contract verifier；以及桌面/移动 Playwright，且无横向溢出或 console error。一项 PostgreSQL 16.14 `SQL_ASCII` producer runtime test 已在仅 loopback 的 server 上通过，随后 server 已停止。该结果只是聚焦 runtime 证据，不代表 production encoding readiness。

Authenticated browser-to-BFF-to-Axum runtime and Git change-set evidence remain `unobserved`; external deployment, public promotion, release, and production evidence remain `deferred`. Criterion 3 is not closed and the long-term goal remains active. No stricter dependency-ready local predecessor appears in the current completion criteria, so the next Criterion 3 increment is private benchmark dataset/suite authoring plus guarded version binding, admitted only after a new bilingual Necessity Record.

authenticated browser-to-BFF-to-Axum runtime 与 Git change-set evidence 仍为 `unobserved`；外部 deployment、public promotion、release 与 production evidence 仍为 `deferred`。条件 3 尚未关闭，长期目标保持 active。当前收束条件没有更严格且依赖就绪的本地前置，因此下一项条件 3 增量确定为私有 benchmark dataset/suite authoring 加 guarded version binding，并且只能在新增一份双语 Necessity Record 后准入。

### 2026-07-23 Benchmark Workspace PostgreSQL and Protected Local Read / 2026-07-23 Benchmark Workspace PostgreSQL 与受保护本地读取

This is the authoritative current local increment and supersedes earlier present-tense counts and next-increment labels while preserving their history. Migration `0019_benchmark_workspace_projection_receipts.sql` durably stores exact project, Context, immutable Context commit, decision, and execution-cohort scope plus exact dataset-case-to-run provenance. A composite foreign key binds each receipt's decision-evidence digest to the matching sealed decision evidence; deferred completeness checks, append-only triggers, and immutable seals admit only complete provenance, identical replay, and conflict-free immutable receipts.

这是当前权威本地增量；它取代此前使用当前时态的测试总数与下一增量标签，同时保留其历史。迁移 `0019_benchmark_workspace_projection_receipts.sql` 持久保存精确 project、Context、不可变 Context commit、decision 与 execution cohort scope，以及精确 dataset-case-to-run provenance。复合外键把每份 receipt 的 decision-evidence digest 绑定到匹配的已封存 decision evidence；延迟完整性检查、append-only trigger 与不可变 seal 只准入完整 provenance、相同 replay 与无冲突的不可变 receipt。

The implemented protected local read is `GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}`, with an optional exact baseline pair and the existing authentication, dedicated rate limiting, `ContextPermission::Read` audit, private no-store, and fail-closed response boundaries. A non-public local SDK consumes that contract. The default public router, checked-in OpenAPI, and public TypeScript SDK remain unchanged. No production application service currently calls `persist_benchmark_workspace_projection`, so the durable projection is not yet produced by the real benchmark execution application path.

已实现的受保护本地读取为 `GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}`，支持可选的精确 baseline pair，并复用既有 authentication、专用 rate limit、`ContextPermission::Read` audit、private no-store 与 fail-closed response 边界；非公开 local SDK 消费该 contract。默认 public router、已检入 OpenAPI 与 public TypeScript SDK 保持不变。当前没有 production application service 调用 `persist_benchmark_workspace_projection`，因此真实 benchmark execution application path 尚不会生成这份 durable projection。

Fresh local evidence passed: `cargo fmt --all -- --check`; strict workspace Clippy; `cargo test --workspace --quiet` with API `162 passed` and storage `166 passed, 37 ignored`; the Rust 1.85 workspace check; `pnpm check:web` with public SDK `14`, local SDK `59`, Web `114`, and the production build; both required contract verifiers; and `python apps/web/verify-context-workspace.py`. One disposable PostgreSQL 16.14 `SQL_ASCII` runtime test passed on a loopback-only server, which was stopped afterward. This is focused PostgreSQL migration/adapter evidence and not production encoding readiness. Git and authenticated browser evidence are `unobserved`; remote CI, operator rehearsal, public promotion, release, and production are `deferred`.

新鲜本地证据均已通过：`cargo fmt --all -- --check`；严格 workspace Clippy；`cargo test --workspace --quiet`，其中 API `162 passed`、storage `166 passed, 37 ignored`；Rust 1.85 workspace check；`pnpm check:web`，其中 public SDK `14`、local SDK `59`、Web `114`，并完成 production build；两项所需 contract verifier；以及 `python apps/web/verify-context-workspace.py`。一项 disposable PostgreSQL 16.14 `SQL_ASCII` runtime test 已在仅 loopback 的 server 上通过，随后 server 已停止。这是聚焦 PostgreSQL migration/adapter 证据，不代表 production encoding readiness。Git 与 authenticated browser evidence 为 `unobserved`；remote CI、operator rehearsal、public promotion、release 与 production 为 `deferred`。

**Next increment / 下一增量：** Complete the real Benchmark workspace Web BFF plus `data -> presenter -> screen` integration, then connect the missing production projection producer that calls `persist_benchmark_workspace_projection`. The long-term goal remains active.

**下一增量：** 完成真实 Benchmark workspace Web BFF 与 `data -> presenter -> screen` 集成，随后接入调用 `persist_benchmark_workspace_projection` 的缺失 production projection producer。长期目标保持 active。

### 2026-07-23 Parallel Core and Integration Closure (Historical Snapshot) / 2026-07-23 并行核心与集成收束（历史快照）

The current totals and queue status in this preserved snapshot are superseded by the Benchmark workspace entry above.

该保留快照中的当前测试总数与队列状态已由上方 Benchmark workspace 条目取代。

The current local pairwise and quality evidence has been directly rechecked after independent review.
Provider-free core increments now include exact Benchmark definition binding and workspace projection,
deterministic Workflow execution with validated replay provenance, validated Knowledge/Memory replay,
versioned MCP/plugin negotiation with redacted diagnostics, versioned diff-review projection, a memory
Benchmark workspace repository contract, and Rust 1.85-compatible CLI/Desktop inspection. Focused
receipts are adapter/CLI/Desktop `6`/`4`/`3`, evaluation projection `6`, Workflow replay `11`, MCP
descriptor `3`, plugin negotiation `5`, Knowledge/Memory replay `7`, and storage projection `4`.
Formatting, workspace tests (API `154`; storage `166 passed, 36 ignored`), strict workspace Clippy,
Rust `1.85.0` workspace check, and `pnpm check:web` (public SDK `14`, local SDK `51`, Web `114`,
production build) pass. Desktop/mobile preview smoke also passes with screenshots, interaction checks,
no horizontal overflow, and no console errors. Docker/PostgreSQL runtime and authenticated
browser-to-BFF-to-Axum E2E are `unobserved`; remote CI, operator rehearsal, public promotion, release,
and production are `deferred`; Git change-set evidence is `unobserved`. The long-term goal remains active.

当前本地 pairwise 与质量证据已在独立审阅后直接复核。provider-free core 现包含精确 Benchmark
definition 绑定与 workspace projection、带可信 replay provenance 的确定性 Workflow execution、经过验证的
Knowledge/Memory replay、带脱敏 diagnostic 的 versioned MCP/plugin negotiation、versioned diff-review
projection、内存 Benchmark workspace repository contract，以及兼容 Rust 1.85 的 CLI/Desktop inspection。
聚焦回执为 adapter/CLI/Desktop `6`/`4`/`3`、evaluation projection `6`、Workflow replay `11`、MCP
descriptor `3`、plugin negotiation `5`、Knowledge/Memory replay `7` 与 storage projection `4`。格式检查、
workspace test（API `154`；storage `166 passed, 36 ignored`）、严格 workspace Clippy、Rust `1.85.0`
workspace check，以及 `pnpm check:web`（public SDK `14`、local SDK `51`、Web `114`、production build）
均通过。桌面/移动 preview smoke 也已通过，生成截图并验证关键交互、无横向溢出和无 console error。
Docker/PostgreSQL runtime 与 authenticated browser-to-BFF-to-Axum E2E 为 `unobserved`；remote CI、
operator rehearsal、public promotion、release 与 production 为 `deferred`；Git change-set evidence 为
`unobserved`。长期目标保持 active。

### 2026-07-19 Wave 3 Local Capability Bridges / 2026-07-19 Wave 3 本地能力桥接

At the 2026-07-19 snapshot, Wave 3 was locally verified as a bounded Core-to-adapter increment, not as a release or project completion. It added deterministic redacted V1 Core projections for sealed benchmark receipt cohort identity, Workflow status/replay, Knowledge citation, Memory retention/replay, and Plugin/MCP capability availability. The existing private Workflow availability route was then the sole newly composed local read: the non-public local SDK parsed it fail-closed, the same-origin BFF forwarded only request-scoped Bearer credentials with `private, no-store`, and the design-system Web inspector exposed shared loading, error, empty, available, and unavailable states. At that snapshot, the detailed Workflow projection and the new Benchmark, Knowledge/Memory, and Plugin/MCP projections were not transported. The later pairwise and transport records below, plus the current Docs/QA closure above, supersede that queue status. Docker/PostgreSQL runtime, browser E2E, remote, release, and production evidence remain unobserved or deferred.

在 2026-07-19 快照中，Wave 3 已作为有界的 Core-to-adapter 增量在本地验证，而不是 release 或项目完成。它为 sealed benchmark receipt cohort identity、Workflow status/replay、Knowledge citation、Memory retention/replay 与 Plugin/MCP capability availability 增加确定性、脱敏的 V1 Core projection。既有 private Workflow availability route 当时是唯一新组合的 local read：非公开 local SDK 会 fail-closed 地解析它，同源 BFF 只转发 request-scoped Bearer 凭据并使用 `private, no-store`，design-system Web inspector 展示共享的 loading、error、empty、available 与 unavailable 状态。在该快照中，详细 Workflow projection 以及新的 Benchmark、Knowledge/Memory、Plugin/MCP projection 尚未 transport。下方后续 pairwise 与 transport 记录及上方当前 Docs/QA 收束已经取代该排队状态。Docker/PostgreSQL runtime、browser E2E、remote、release 与 production evidence 仍为 unobserved 或 deferred。

**Historical next-dependency record / 历史下一依赖记录：** The source-binding prerequisite was recorded as the private domain/storage increment below, and the separately scoped private Workflow binding read in `docs/superpowers/plans/2026-07-22-private-workflow-binding-read.md` was its next local consumer. The completed local implementation is recorded below; public transport, writes, execution, and runtime evidence remain outside that admission.

**历史下一依赖记录：** source-binding prerequisite 当时已在下方私有 domain/storage 增量中记录，`docs/superpowers/plans/2026-07-22-private-workflow-binding-read.md` 中单独 scope 的私有 Workflow binding read 是下一位 local consumer。下方已记录其完成的本地实现；public transport、write、execution 与 runtime evidence 仍排除在该准入范围之外。

### 2026-07-22 Private Workflow Binding Read / 2026-07-22 私有 Workflow Binding 读取

The private Workflow binding read is complete for the locally observed implementation scope. The protected API and non-public local SDK retain exact Context/commit scope and redacted V1 fields. The Web `data -> presenter -> screen` and selected-commit inspector path is present; its focused Web receipt is `84 passed`, and the Web TypeScript check passed. The same-origin BFF route is present and its focused route receipt is `7 passed`, including raw-field rejection. These receipts do not add public REST/OpenAPI/public SDK writes, workflow execution, provider calls, or a second graph-diff calculator; `GraphDiff::between` remains the sole graph-diff calculator.

私有 Workflow binding read 已在本地已观测的实现范围内完成。protected API 与非公开 local SDK 保持精确 Context/commit scope 与脱敏 V1 字段。Web `data -> presenter -> screen` 与选定 commit inspector path 已存在；聚焦 Web 回执为 `84 passed`，Web TypeScript check 已通过。同源 BFF route 已存在，其聚焦 route 回执为 `7 passed`，包括 raw-field rejection。这些回执不新增 public REST/OpenAPI/public SDK write、Workflow execution、provider call 或第二个 graph-diff calculator；`GraphDiff::between` 仍是唯一的 graph-diff calculator。

The full `pnpm check:web` command and production build are now freshly passed for this local source check. Authenticated browser-to-BFF-to-protected-Axum runtime, Docker/PostgreSQL runtime, remote CI, operator rehearsal, release, and production evidence remain `unobserved` or `deferred`; the long-term goal remains active.

本记录中的完整 `pnpm check:web` command 与 production build 现已在本地 source check 中新鲜通过。authenticated browser-to-BFF-to-protected-Axum runtime、Docker/PostgreSQL runtime、remote CI、operator rehearsal、release 与 production evidence 仍为 `unobserved` 或 `deferred`；长期目标保持 active。

The private benchmark definition and decision evidence boundary in `docs/superpowers/plans/2026-07-15-benchmark-definition-decision-persistence.md` is implemented. `contextlab-evaluation` owns the `BenchmarkEvaluation` policy artifact and validates model identity plus finite inclusive `0.0..=2.0` temperature; `contextlab-storage` atomically persists and replays sealed project/Context/commit-scoped datasets, suites, runs, coverage facts, and decision evidence through equivalent memory/PostgreSQL repository contracts. Decision identity, child rows, and seals are exact-commit-scoped, and PostgreSQL timestamp normalization rebuilds the domain evaluation from normalized runs. Fresh local receipts are evaluation `27 passed`, storage `159 passed, 31 ignored`, `cargo fmt --all -- --check`, scoped Clippy, and `cargo test --workspace --quiet`. The PostgreSQL benchmark tests compile and remain ignored/unobserved while Docker is disabled and no local PostgreSQL service is configured. Public REST/OpenAPI/SDK/Web remain unchanged, and `GraphDiff::between` remains the sole graph-diff calculator.

`docs/superpowers/plans/2026-07-15-benchmark-definition-decision-persistence.md` 中的私有 benchmark definition 与 decision evidence boundary 已实现。`contextlab-evaluation` 拥有 `BenchmarkEvaluation` policy artifact，并校验 model identity 与有限、包含边界的 `0.0..=2.0` temperature；`contextlab-storage` 则通过语义等价的 memory/PostgreSQL repository contract，原子持久化并 replay 已 seal、按 project/Context/commit 作用域划分的 dataset、suite、run、coverage fact 与 decision evidence。decision identity、child row 与 seal 均按精确 commit 分区，PostgreSQL timestamp normalization 会从规范化后的 run 重建 domain evaluation。新鲜本地回执为：evaluation `27 passed`、storage `159 passed, 31 ignored`、`cargo fmt --all -- --check`、范围化 Clippy 与 `cargo test --workspace --quiet`。PostgreSQL benchmark test 已完成编译；Docker 关闭且未配置本地 PostgreSQL service 时，它们仍为 ignored/unobserved。public REST/OpenAPI/SDK/Web 保持不变，`GraphDiff::between` 仍是唯一 graph-diff calculator。

The evidence boundary remains strict: the dated 25-case PostgreSQL run is only an underlying storage baseline, not lifecycle PostgreSQL E2E evidence. Lifecycle PostgreSQL E2E and authenticated browser-to-BFF-to-protected-Axum smoke are deferred/unobserved. Remote CI, operator approval, public promotion, release, and production rollout also remain deferred external evidence; no publication or release is claimed.

证据边界保持严格：带日期的 25-case PostgreSQL 运行仅是底层 storage baseline，不是 lifecycle PostgreSQL E2E 证据。lifecycle PostgreSQL E2E 与 authenticated browser-to-BFF-to-protected-Axum smoke 仍为 deferred/unobserved。remote CI、operator approval、public promotion、release 与 production rollout 也仍是延期外部证据；本仓库不声称已经发布或上线。

On 2026-07-18, the separate private benchmark evidence inspection increment added an authenticated local-only GET for one exact project/Context/commit/decision, a non-public local SDK, same-origin BFF, and a design-system inspection panel. It enforces `ContextPermission::Read`, a dedicated rate-limit operation, recursive response redaction/shape validation, request-scoped Bearer forwarding without cookies, `Cache-Control: private, no-store`, disabled scope controls while a read is pending, and code-point-stable presentation order. Fresh local evidence includes `cargo test -p contextlab-api local_benchmark_decision` (`5 passed`), `cargo test --workspace` (API `129 passed`; storage `159 passed, 31 ignored`), public SDK `14 passed`, local SDK `6 passed`, Web `41 passed`, package TypeScript checks, and a successful production Web build. This is a private/local read workflow only: public REST/OpenAPI/public SDK remain unchanged, there is no mutation, execution, release, production claim, or additional graph-diff calculator.

2026-07-18 的独立私有 benchmark evidence inspection 增量新增了一个只读、仅本地的 authenticated GET，用于读取一条精确 project/Context/commit/decision；同时新增非公开 local SDK、同源 BFF 与 design-system inspection panel。它强制 `ContextPermission::Read`、独立 rate-limit operation、递归 response redaction/shape validation、无 cookie 的 request-scoped Bearer 转发、`Cache-Control: private, no-store`、read pending 期间禁用 scope control，以及按 code point 稳定排序的展示。新鲜本地证据包括 `cargo test -p contextlab-api local_benchmark_decision`（`5 passed`）、`cargo test --workspace`（API `129 passed`；storage `159 passed, 31 ignored`）、public SDK `14 passed`、local SDK `6 passed`、Web `41 passed`、各 package 的 TypeScript check 和成功的 production Web build。这只是一条 private/local read workflow：public REST/OpenAPI/public SDK 未改变，不包含 mutation、execution、release、production claim 或额外 graph-diff calculator。

## Latest Increment / 最近增量

The private benchmark evaluation diff plan is implemented through a pure evaluation comparison engine, atomic pair reads, an authenticated local-only route, the non-public local SDK, same-origin BFF, and a bilingual design-system Web inspector. The route is absent from public REST/OpenAPI/public SDK catalogs and uses the distinct `BenchmarkDecisionDiffRead` limiter. It compares only immutable status and metric evidence after fingerprint equality, so API, SDK, BFF, and Web never re-run policy. Fresh local receipts are `cargo fmt --all -- --check`, `cargo test --workspace`, and `pnpm check:web`; the scoped Clippy gate is accurately open because an unrelated Rust 1.85 MSRV lint in `contextlab-auth` fails at `crates/auth/src/authorization.rs:320`. PostgreSQL runtime evidence is still compiled/ignored and deferred while Docker remains disabled.

私有 benchmark evaluation diff plan 已通过纯 evaluation comparison engine、原子 pair read、authenticated local-only route、非公开 local SDK、同源 BFF 与双语 design-system Web inspector 实现。该 route 不存在于 public REST/OpenAPI/public SDK catalog 中，并使用独立的 `BenchmarkDecisionDiffRead` limiter。它只在 fingerprint 相等后比较不可变 status 与 metric evidence，因此 API、SDK、BFF 与 Web 都不会重新运行 policy。新鲜本地回执为 `cargo fmt --all -- --check`、`cargo test --workspace` 与 `pnpm check:web`；范围化 Clippy gate 仍按事实保持开放，因为无关的 Rust 1.85 MSRV lint 在 `contextlab-auth` 的 `crates/auth/src/authorization.rs:320` 失败。Docker 仍关闭时 PostgreSQL runtime evidence 已编译/ignored 并继续延期。

The private benchmark execution orchestration increment is now implemented. It loads sealed definitions at an exact project/Context/commit scope, gives each immutable case only to an injected evaluator port, rejects duplicate metric results before a run exists, and derives a UUIDv5 case-to-run provenance identity from the exact decision. Existing decisions replay without evaluator invocation; new evidence uses the existing sole writer and remains inspectable/diffable through the prior local read workflows. Fresh local evidence is evaluation `4 passed`, storage `5 passed`, `cargo fmt --all -- --check`, evaluation-scoped Clippy, and `cargo test --workspace --quiet` with API `131 passed` and storage `159 passed, 32 ignored`. It adds no provider call, public REST/OpenAPI/public SDK/Web mutation, schema migration, Docker use, or secret access. PostgreSQL execution runtime remains compiled/ignored and unobserved.

私有 benchmark execution orchestration 增量现已实现。它会在精确 project/Context/commit scope 加载已 seal 的 definition，只把每条不可变 case 交给注入的 evaluator port，在任何 run 出现前拒绝重复 metric result，并从精确 decision 派生 UUIDv5 的 case-to-run provenance identity。已存在的 decision 会在不调用 evaluator 的情况下 replay；新的 evidence 只使用既有的唯一 writer，并继续可通过先前的 local read workflow 审阅与比较。新鲜本地证据为 evaluation `4 passed`、storage `5 passed`、`cargo fmt --all -- --check`、evaluation 范围化 Clippy，以及 `cargo test --workspace --quiet`（API `131 passed`、storage `159 passed, 32 ignored`）。它不新增 provider call、public REST/OpenAPI/public SDK/Web mutation、schema migration、Docker 使用或密钥访问。PostgreSQL execution runtime 仍为 compiled/ignored 且未观测。

The follow-on sealed definition-metadata inspection is implemented as one private read vertical slice. The storage summary consumes the already-loaded exact sealed decision, validates immutable suite/dataset membership, and projects `definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }` without raw cases, inputs, expected outputs, runs, measurements, or policy output. The existing protected local reader, non-public SDK, same-origin BFF, and shared-primitive Web inspector preserve authentication, RBAC, rate-limit, audit, private/no-store, and no-public-contract boundaries. Fresh evidence: `cargo fmt --all -- --check`, focused storage `2 passed`, focused API `10 passed`, `cargo test --workspace --quiet` (API `134 passed`; storage `159 passed, 32 ignored`), and `pnpm check:web` (public SDK, local SDK, BFF, Web checks, and production Web build). Scoped strict Clippy remains open only due to the unrelated Rust 1.85 MSRV lint at `crates/auth/src/authorization.rs:320`; Docker/PostgreSQL runtime, browser E2E, remote CI, and production evidence remain unobserved.

后续的 sealed definition-metadata inspection 已作为一条私有读取垂直切片实现。storage summary 会消费已经加载的精确 sealed decision、校验不可变 suite/dataset membership，并投影 `definition: { suite: { id, name, thresholds: [{ metric, direction, value }] }, datasets: [{ id, name, case_count }] }`，不包含 raw case、input、expected output、run、measurement 或 policy output。既有 protected local reader、非公开 SDK、同源 BFF 与共享 primitive 的 Web inspector 保持 authentication、RBAC、rate-limit、audit、private/no-store 与无 public contract 的边界。新鲜证据为：`cargo fmt --all -- --check`、聚焦 storage `2 passed`、聚焦 API `10 passed`、`cargo test --workspace --quiet`（API `134 passed`；storage `159 passed, 32 ignored`）以及 `pnpm check:web`（public SDK、local SDK、BFF、Web check 与 production Web build）。范围化 strict Clippy 仅因无关的 Rust 1.85 MSRV lint 在 `crates/auth/src/authorization.rs:320` 保持开放；Docker/PostgreSQL runtime、browser E2E、remote CI 与 production evidence 仍未观测。

The private component-descriptor revision increment is implemented through the existing guarded lifecycle command. `UpdatedComponentDescriptor` records only a validated name/metadata replacement, preserves the immutable body hash and source content commit, and produces a successor graph snapshot with the matching node label. Its private `ComponentDescriptorRevisionWrite` attachment updates the current component projection in the same transaction as commit, snapshot, branch head, and receipt. Idempotency is now branch-scoped as `(identity source, principal, Context, branch, key)` in both adapters; migration `0017_branch_scoped_commit_idempotency.sql` backfills the immutable referenced branch before extending the receipt key, and PostgreSQL rechecks that binding on replay. The protected local route, non-public local SDK, same-origin BFF, and design-system editor reuse existing authentication, RBAC, audit, rate limiting, idempotency, and private/no-store boundaries; no public REST/OpenAPI/public SDK route or GraphDiff calculator changed. Fresh local evidence is `cargo fmt --all -- --check`, `cargo test --workspace --quiet` (API `136 passed`; storage `163 passed, 33 ignored`), and `pnpm check:web` (public SDK `14`, local SDK `23`, Web `51`, TypeScript checks, production build). Strict Clippy remains an open repository baseline: the scoped command reports pre-existing MSRV and storage/API warnings, so it is not presented as a clean slice receipt. PostgreSQL runtime remains compiled/ignored and unobserved while Docker is disabled.

私有 component-descriptor revision 增量已通过既有 guarded lifecycle command 实现。`UpdatedComponentDescriptor` 只记录经过校验的 name/metadata 替换，保留不可变 body hash 与 source content commit，并生成 node label 匹配的 successor graph snapshot。其私有 `ComponentDescriptorRevisionWrite` attachment 会在 commit、snapshot、branch head 与 receipt 的同一 transaction 中更新当前 component projection。两个 adapter 的 idempotency 现以 `(identity source, principal, Context, branch, key)` 作为 branch-scoped 作用域；迁移 `0017_branch_scoped_commit_idempotency.sql` 会在扩展 receipt key 前回填不可变的被引用 branch，PostgreSQL 在 replay 时会再次校验该绑定。protected local route、非公开 local SDK、同源 BFF 与 design-system editor 复用既有 authentication、RBAC、audit、rate limiting、idempotency 与 private/no-store 边界；没有改变 public REST/OpenAPI/public SDK route 或 GraphDiff calculator。新鲜本地证据为 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`（API `136 passed`；storage `163 passed, 33 ignored`）与 `pnpm check:web`（public SDK `14`、local SDK `23`、Web `51`、TypeScript check、production build）。strict Clippy 仍是未收束的 repository baseline：范围化 command 报告已有 MSRV 以及 storage/API warning，因此不将其表述为本切片的 clean receipt。Docker 关闭时 PostgreSQL runtime 仍为 compiled/ignored 且未观测。

The private unborn-branch Context initialization increment is implemented through the existing guarded lifecycle path. `initialize` accepts `expected_head_commit_id: null` only for an active persisted Context with no head on the named branch; the service reads only the Context name, creates one parentless `CreatedContext` commit and one-node root graph snapshot, and attaches no component or body revision. Same-branch retries replay the original root after later head advance, while the branch-scoped receipt permits independent roots on other unborn branches. The protected local route, non-public local SDK, same-origin BFF, and shared-primitive editor reuse authentication, RBAC, audit, rate limiting, idempotency, private/no-store, and public-contract exclusion. The editor and mutation BFF are default-deny unless the exact server-owned `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE=true` enables local development. Fresh local evidence is recorded in the current implementation plan: API `140 passed`, storage `165 passed, 34 ignored`, public SDK `14`, local SDK `24`, and Web `57`; PostgreSQL runtime and authenticated browser mutation smoke remain unobserved while Docker is disabled.

私有未出生分支 Context 初始化增量已通过既有 guarded lifecycle path 实现。`initialize` 只会为已持久化且在指定 branch 没有 head 的 active Context 接受 `expected_head_commit_id: null`；service 只读取 Context name，创建一条无 parent 的 `CreatedContext` commit 和只含 root 的 graph snapshot，不附带 component 或 body revision。同一 branch 的 retry 会在后续 head 推进后回放原 root，而 branch-scoped receipt 允许在其他 unborn branch 上独立创建 root。protected local route、非公开 local SDK、同源 BFF 与 shared-primitive editor 复用 authentication、RBAC、audit、rate limiting、idempotency、private/no-store 与 public contract exclusion。editor 与 mutation BFF 默认拒绝，只有精确的服务端 `CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE=true` 才会启用本地开发。当前 implementation plan 已记录新鲜本地证据：API `140 passed`、storage `165 passed, 34 ignored`、public SDK `14`、local SDK `24`、Web `57`；Docker 关闭时 PostgreSQL runtime 与 authenticated browser mutation smoke 仍未观测。

The current private typed Uses relationship increment extends only the existing guarded local lifecycle. `add_uses_relationship` and `remove_uses_relationship` accept two distinct component identifiers at a non-null exact materialized head; the server reconstructs and validates both active same-Context endpoints, then changes exactly one directed `Uses` edge while preserving unrelated graph facts. Duplicate addition and missing removal fail closed. The relationship-only commit carries no component mutation attachment and reuses exact-head compare-and-swap, branch-scoped idempotency replay, and atomic commit/snapshot/head/receipt persistence. The server-owned local Web gate remains default-deny, public REST/OpenAPI/public SDK and `GraphDiff::between` remain unchanged, and no next increment begins from this documentation closure. Docker-backed PostgreSQL runtime and authenticated browser mutation runtime for this increment remain unobserved; no test total is claimed here.

当前私有类型化 Uses relationship 增量只扩展既有 guarded local lifecycle。`add_uses_relationship` 与 `remove_uses_relationship` 在非空的精确 materialized head 上接收两个不同的 component identifier；服务端重建并校验同一 Context 中两个 active endpoint，再只修改一条有向 `Uses` edge，同时保留无关 graph fact。重复新增和移除不存在的关系都会 fail closed。仅包含 relationship 的 commit 不携带 component mutation attachment，并复用 exact-head compare-and-swap、branch-scoped idempotency replay，以及 commit/snapshot/head/receipt 的原子持久化。由服务端拥有的 local Web gate 仍默认拒绝，public REST/OpenAPI/public SDK 与 `GraphDiff::between` 保持不变，本次文档收束不启动下一增量。本增量的 Docker-backed PostgreSQL runtime 与 authenticated browser mutation runtime 仍未观测；此处不声明测试总数。

### 2026-07-22 Private Context-to-Workflow Source Binding / 2026-07-22 私有 Context 到 Workflow 源绑定

The next dependency-ready core increment is now implemented as a private domain/storage contract. `WorkflowContextBinding` seals a complete Workflow definition revision to one exact typed Context commit source. `ContextWorkflowBindingRepository` accepts only a commit that already has a materialized graph snapshot, returns `Created` or identical `Replayed`, rejects a different source for the same Workflow identity/revision, and lists exact commit scope in canonical order. Memory and PostgreSQL adapters share the contract; migration `0018_context_workflow_bindings.sql` adds composite Context/commit integrity, a materialized-snapshot foreign key, and immutable Workflow revision uniqueness. Fresh local evidence is `cargo fmt --all -- --check`, Workflow binding tests (`2 passed`), storage binding tests (`3 passed`), and the full storage crate (`166 passed, 36 ignored`). PostgreSQL runtime remains ignored/unobserved while Docker is disabled.

下一个依赖就绪的核心增量现已作为私有 domain/storage contract 实现。`WorkflowContextBinding` 将完整 Workflow definition revision 封存到一个精确的类型化 Context commit source。`ContextWorkflowBindingRepository` 只接受已经具有 materialized graph snapshot 的 commit，返回 `Created` 或相同 binding 的 `Replayed`，拒绝同一 Workflow identity/revision 指向不同 source，并按 canonical order 列出精确 commit scope。Memory 与 PostgreSQL adapter 共享同一 contract；迁移 `0018_context_workflow_bindings.sql` 增加 Context/commit 复合完整性、materialized-snapshot foreign key 与不可变 Workflow revision 唯一性。新鲜本地证据为 `cargo fmt --all -- --check`、Workflow binding test（`2 passed`）、storage binding test（`3 passed`）以及 storage crate 全量（`166 passed, 36 ignored`）。Docker 关闭时 PostgreSQL runtime 仍为 ignored/unobserved。

This increment advances Criteria 1, 2, 4, and 9 but does not close them. It deliberately adds no API/local-SDK/Web transport, workflow execution, or public write. The next transport decision remains gated on a separate bilingual Necessity Record and exact Context authorization tests.

本增量推进条件 1、2、4、9，但不关闭这些条件。它明确不新增 API/local-SDK/Web transport、Workflow execution 或 public write。下一步 transport 决策仍需单独的双语 Necessity Record 与精确 Context authorization test 作为门禁。

### 2026-07-22 Private Workflow Binding Read Documentation Admission / 2026-07-22 私有 Workflow Binding 读取文档准入

The private Workflow binding read is now implemented as a local exact-commit read vertical slice: `GET /api/v1/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings`, the server-owned `contextlab.local-workflow-context-bindings.v1` redacted summary, a fail-closed non-public SDK parser/client, a same-origin Web BFF, and a shared `data -> presenter -> screen` inspector anchored to `selectedCommitDetail.id`. This advances Criteria 1, 2, 4, 6, and 9 without closing them; it does not add public REST/OpenAPI/public SDK writes, Web mutations, workflow execution, provider calls, or production readiness.

私有 Workflow binding read 现已实现为本地精确 commit 的读取垂直切片：`GET /api/v1/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings`、server-owned 的 `contextlab.local-workflow-context-bindings.v1` 脱敏 summary、fail-closed 的非公开 SDK parser/client、同源 Web BFF，以及以 `selectedCommitDetail.id` 为锚点的共享 `data -> presenter -> screen` inspector。本增量推进条件 1、2、4、6 与 9，但不关闭这些条件；不新增 public REST/OpenAPI/public SDK write、Web mutation、Workflow execution、provider call 或 production readiness。

Focused local implementation evidence for this slice remains API binding `4 passed`, local SDK `35 passed`, Web TypeScript `tsc --noEmit`, Web `84 passed`, and nested BFF route `7 passed`. The broader 2026-07-23 receipt above supersedes the earlier pending status and totals, and records the passing unauthenticated desktop/mobile preview smoke. Docker/PostgreSQL runtime and authenticated browser E2E remain `unobserved`; remote CI, operator rehearsal, release, and production remain `deferred`. The long-term goal remains active.

本切片的聚焦本地 implementation evidence 仍为：API binding `4 passed`、local SDK `35 passed`、Web TypeScript `tsc --noEmit`、Web `84 passed` 与嵌套 BFF route `7 passed`。上方 2026-07-23 更广回执取代此前待验证状态与测试总数，并记录了通过的未认证桌面/移动 preview smoke。Docker/PostgreSQL runtime 与 authenticated browser E2E 仍为 `unobserved`；remote CI、operator rehearsal、release 与 production 仍为 `deferred`。长期目标保持 active。

## Operating Rule / 执行规则

Every increment re-reads the project charter, architecture specification, roadmap, and current code state; selects the closest verifiable unmet convergence condition; and supplies risk-proportionate tests, bilingual documentation, and architecture-boundary evidence. Independent research, implementation, and verification use multiple bounded subagents when useful.

每轮增量都先重读项目宪章、架构规范、路线图与当前代码状态，选择最接近收束条件的可验证未完成项，并提供与风险相称的测试、双语文档与架构边界证据。相互独立的调研、实现与验证在有价值时使用多个有界 subagent。

No page, API, SDK, test suite, or feature slice closes this goal. The goal remains active until every condition in `docs/roadmap/completion-criteria.md` has fresh verification.

任何页面、API、SDK、测试套件或功能切片都不能关闭本目标。只有 `docs/roadmap/completion-criteria.md` 的全部条件获得新鲜验证后，目标才可关闭。

### 2026-07-18 Private Sealed Benchmark Decision Run Details / 2026-07-18 私有已封存 Benchmark Decision Run 明细

The admitted run-details increment is a private local read path for one sealed benchmark decision. It resolves the decision's exact ordered run membership, applies fail-closed redaction and exact scope checks, and exposes only safe run summaries through the existing protected local inspection composition. It directly advances Criterion 3 while preserving the no-public-write boundary and the single `GraphDiff::between` calculator. This is a scope record, not a completion claim.

准入的 run-details 增量是一条针对单个 sealed benchmark decision 的私有 local read path。它解析 decision 的精确有序 run membership，执行 fail-closed redaction 与 exact scope 校验，并通过既有 protected local inspection composition 只暴露安全 run summary。它直接推进条件 3，同时保持 no-public-write boundary 与唯一的 `GraphDiff::between` calculator。这是范围记录，不是完成声明。

The plan does not claim Docker/PostgreSQL runtime or authenticated browser E2E; both remain unobserved. Fresh main-thread evidence includes five consecutive focused storage passes, focused API/local-SDK/Web checks, `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, `pnpm check:web`, and ignored PostgreSQL compile-only coverage. The scoped strict Clippy command remains blocked only by the existing Rust 1.85 MSRV lint at `crates/auth/src/authorization.rs:320`.

本计划不声称 Docker/PostgreSQL runtime 或 authenticated browser E2E；二者仍未观测。main-thread 的新鲜证据包括连续五次通过的聚焦 storage 测试、聚焦 API/local SDK/Web 检查、`cargo fmt --all -- --check`、`cargo test --workspace --quiet`、`pnpm check:web`，以及 ignored PostgreSQL 的仅编译覆盖。范围化 strict Clippy 仅受既有 Rust 1.85 MSRV lint 阻断，位置为 `crates/auth/src/authorization.rs:320`。

### 2026-07-22 Private Benchmark Decision Discovery / 2026-07-22 私有 Benchmark Decision 发现

The private benchmark decision discovery increment is implemented as a protected local, read-only
path. A separate `BenchmarkDecisionDiscoveryRepository` lists only safe sealed-decision summaries
at the exact project/Context/commit scope; `BenchmarkDecisionDiscoveryService` consumes that port
without hydrating raw decision evidence. The projection contains stable suite/dataset metadata,
status, recording time, and run count, but no raw cases, inputs, expected outputs, runs, or
measurements. The protected API route, non-public local SDK, same-origin BFF, and Web
`data -> presenter -> screen` adapter preserve exact scope, Context read authorization, audit, rate
limiting, private/no-store behavior, deterministic ordering, and fail-closed parsing. This advances
Criterion 3 and the dataset/suite inspection path but does not close either condition.

私有 benchmark decision discovery 增量已实现为 protected local、只读 path。独立的
`BenchmarkDecisionDiscoveryRepository` 只在精确 project/Context/commit scope 列出安全的
sealed-decision summary；`BenchmarkDecisionDiscoveryService` 只消费该 port，不 hydrate 原始
decision evidence。projection 只包含稳定的 suite/dataset metadata、status、recording time 与 run
count，不暴露 raw case、input、expected output、run 或 measurement。protected API route、非公开
local SDK、同源 BFF 与 Web `data -> presenter -> screen` adapter 保持 exact scope、Context read
authorization、audit、rate limiting、private/no-store、确定性排序与 fail-closed parsing。本增量推进
条件 3 与 dataset/suite inspection path，但不关闭任一条件。

The earlier cross-stack receipt snapshot predates the current safe-summary, scope-echo, and strict
timestamp hardening and is historical only. The 2026-07-23 refresh above now supplies the full local
receipt: formatting passed, the Rust workspace passed with API `153` and storage `166 passed, 36
ignored`, and `pnpm check:web` passed with public SDK `14`, local SDK `51`, Web `109`, and the
production Web build. Focused receipts additionally cover strict timestamp parsing `5`, nested BFF
security `4`, and public OpenAPI/SDK exclusion `8`. Docker/PostgreSQL runtime, browser
visual/authenticated E2E, remote CI, operator rehearsal, release, and production evidence remain
unobserved or deferred. No public REST/OpenAPI/public SDK write, Web mutation, provider call,
operator transport, migration, or production-readiness claim was added; the long-term goal remains
active.

先前的 cross-stack 回执快照早于当前 safe-summary、scope-echo 与严格 timestamp hardening，因此只属于
历史记录。上方 2026-07-23 刷新现已提供完整本地回执：格式检查通过，Rust workspace 通过且 API 为
`153`、storage 为 `166 passed, 36 ignored`，`pnpm check:web` 通过且 public SDK `14`、local SDK
`51`、Web `109`，并完成 production Web build。聚焦回执还覆盖严格 timestamp parser `5`、嵌套 BFF
security `4` 与 public OpenAPI/SDK exclusion `8`。Docker/PostgreSQL runtime、browser
visual/authenticated E2E、remote CI、operator rehearsal、release 与 production evidence 仍为未观测或
延期。本增量没有新增 public REST/OpenAPI/public SDK write、Web mutation、provider call、operator
transport、migration 或 production-readiness 声明；长期目标保持 active。

The private selection flow reuses this exact sealed decision list for the existing version-backed
evaluation diff read. Its Web data/presenter/screen implementation and focused selection receipt
(`5 passed`) are included in the fresh cross-stack verification above. It retains exact dual scope
and keeps `GraphDiff::between` as the sole graph-diff calculator.

私有 selection flow 会复用本次精确 sealed decision list，为既有 version-backed evaluation diff
read 提供选择。其 Web data/presenter/screen 实现与聚焦 selection 回执（`5 passed`）已包含在上方的
新鲜 cross-stack 验证中。它保持 exact dual scope，并确保 `GraphDiff::between` 仍是唯一
graph-diff calculator。

### 2026-07-27 Private Benchmark Definition Authoring / 2026-07-27 私有 Benchmark 定义创作

The admitted Criterion 3 increment now has a Rust domain/storage implementation: immutable
dataset/suite authoring is bound to an exact Context commit and guarded by schema, deterministic
membership, principal-scoped idempotency, request digest, branch-head equality, and PostgreSQL
transaction-local write authorization. Memory/PostgreSQL exact read/list ports preserve parity,
including microsecond capture-time normalization. Migration 0020 is append-only and composite
scope constrained. Fresh local proof is authoring 3/3, migration 6/6, storage 166 passed and 38
ignored, strict storage Clippy, and 26/26 named isolated UTF-8 loopback PostgreSQL 16.14 tests
passed; the native service and disposable cluster were stopped. The increment advances Criterion 3
but does not close it. Wave 2 private local transport, SDK, BFF, and Web editor remain pending.
Public REST/OpenAPI/public SDK writes, provider calls, Context commit mutation, browser-auth runtime,
Git, remote CI, operator rehearsal, release, and production remain out of scope or unobserved/
deferred. The long-term goal remains active.

本次准入的条件 3 增量现已具备 Rust domain/storage implementation：不可变 dataset/suite authoring
绑定精确 Context commit，并受 schema、确定性 membership、principal-scoped idempotency、request
digest、branch-head equality 与 PostgreSQL transaction-local write authorization 保护。
Memory/PostgreSQL exact read/list port 保持 parity，包括微秒级 capture-time normalization。迁移
0020 具备 append-only 与复合 scope 约束。新鲜本地证明为 authoring 3/3、migration 6/6、
storage 166 passed and 38 ignored、严格 storage Clippy，以及 UTF-8 loopback PostgreSQL 16.14
的 26/26 个隔离 test 全部通过；native service 与 disposable cluster 随后已停止。本增量推进
条件 3 但不关闭它。Wave 2 private local transport、SDK、BFF 与 Web editor 仍待完成。
Public REST/OpenAPI/public SDK write、provider call、Context commit mutation、browser-auth
runtime、Git、remote CI、operator rehearsal、release 与 production 均不在当前范围或仍为
unobserved/deferred。长期目标保持 active。

## 2026-07-27 Benchmark Authoring Wave 2 / 2026-07-27 Benchmark Authoring Wave 2

This superseding entry records the completed private product transport for the authoring contract.
The canonical route is the project-scoped `benchmark-definition-bindings` POST; the non-public local
SDK, same-origin BFF, and Web editor all echo the exact project/Context/commit scope. The browser
keeps Bearer credentials in request memory, omits cookies, uses `credentials: "omit"`, and receives
`private, no-store` responses. The editor is default-off and consumes server-owned redacted receipts;
it does not recalculate benchmark policy or graph diffs. The old context-only BFF route now fails closed
with `410 benchmark_definition_route_gone` and never forwards upstream.

本条取代前述“Wave 2 transport 仍待完成”的当前时态记录，记录 authoring contract 已完成 private product
transport。canonical route 是 project-scoped `benchmark-definition-bindings` POST；非公开 local SDK、同源 BFF
与 Web editor 都回显精确 project/Context/commit scope。浏览器只在请求内存保存 Bearer credential，不携带 cookie，
使用 `credentials: "omit"`，并接收 `private, no-store` response。editor 默认关闭，只消费 server-owned 脱敏 receipt，
不重新计算 benchmark policy 或 graph diff。旧的 context-only BFF route 现 fail closed 返回
`410 benchmark_definition_route_gone`，绝不向 upstream 转发。

Public REST/OpenAPI/public SDK write, provider execution, Context commit mutation, operator transport,
release, and production promotion remain out of scope. The next dependency-ready local increment is
the smallest Criterion 3 completion gap identified after fresh verification; the long-term goal remains
active and external release evidence remains deferred.

Public REST/OpenAPI/public SDK write、provider execution、Context commit mutation、operator transport、release 与
production promotion 仍不在范围内。下一项依赖就绪的本地增量将在新鲜验证后按条件 3 的最小缺口选择；长期目标
保持 active，外部 release evidence 继续延期。

Fresh verification for this Wave 2 closure is `cargo fmt --all -- --check`; workspace Rust tests with
storage `167 passed, 39 ignored`; strict workspace Clippy; locked Rust `1.85.0` check; `pnpm check:web`
with public SDK `14`, local SDK `68`, Web `151`, and production build; authoring data/presenter/screen
focused tests `11/11`; the full Web route receipt including the retired `410` route and canonical route;
the wave2 verifier self-test; and a live verifier result of `wave2_local_contracts=passed` and
`graph_diff_calculators=passed count=1`. Its `overall=unobserved` is limited to unavailable Git evidence.
The local workspace Playwright verifier was not observed because no Web server was running; Docker/
PostgreSQL runtime, authenticated browser runtime, Git binding, remote CI, operator rehearsal, release,
and production remain `ignored`, `unobserved`, or `deferred`. This is progress on Criterion 3, not closure.

本 Wave 2 收束的新鲜验证包括 `cargo fmt --all -- --check`；workspace Rust test（storage `167 passed, 39 ignored`）；
strict workspace Clippy；锁定 Rust `1.85.0` check；`pnpm check:web`（public SDK `14`、local SDK `68`、Web `151`，
并完成 production build）；authoring data/presenter/screen 聚焦 `11/11`；包含退役 `410` route 与 canonical route 的
Web 全量回执；wave2 verifier 自测；以及实际 verifier 的 `wave2_local_contracts=passed` 与
`graph_diff_calculators=passed count=1`。其 `overall=unobserved` 仅限 Git evidence 不可用。local workspace Playwright
verifier 因未启动 Web server 而未观测到；Docker/PostgreSQL runtime、authenticated browser runtime、Git binding、
remote CI、operator rehearsal、release 与 production 仍为 `ignored`、`unobserved` 或 `deferred`。这推进条件 3，
但不构成收束。

## 2026-07-27 Binding Inspection Follow-on / 2026-07-27 Binding Inspection 后续增量

The next dependency-ready local increment is now implemented: exact benchmark-definition binding
inspection and selection. The Rust storage summary, protected API GET, non-public local SDK, canonical
same-origin BFF, and Web `data -> presenter -> screen` inspector all preserve project/Context/commit
scope and deterministic binding order. The surface is redacted, default-off, request-memory Bearer,
cookie-free, `credentials: "omit"`, `private, no-store`, and outside public OpenAPI/public SDK. The
Web only selects an immutable binding ID; it does not resolve mutable latest state or recalculate
benchmark policy/diffs. The long-term goal remains active.

下一项依赖就绪的本地增量现已实现：精确 benchmark-definition binding inspection 与 selection。Rust storage summary、
protected API GET、非公开 local SDK、canonical 同源 BFF 与 Web `data -> presenter -> screen` inspector 均保持
project/Context/commit scope 与确定性 binding order。该 surface 脱敏、默认关闭、使用 request-memory Bearer、
不携带 cookie、采用 `credentials: "omit"`、返回 `private, no-store`，且不进入 public OpenAPI/public SDK。Web 只选择
不可变 binding ID，不解析可变 latest state，也不重新计算 benchmark policy/diff。长期目标保持 active。

Fresh receipt for this increment is `cargo fmt --all -- --check`; storage `167 passed, 39 ignored`;
`wave2_local_contracts=passed`; `graph_diff_calculators=passed count=1`; and `pnpm check:web` with
public SDK `14`, local SDK `70`, Web `156`, and a successful production build. The live verifier is
`overall=unobserved` only for unavailable Git change-set evidence; no Web server was running for the
local Playwright verifier. Six requested `gpt-5.6-luna` workers were rejected by the full agent thread
limit and are recorded as scheduling failure only. The next admitted local increment is the
provider-free exact binding execution-selection bridge in
`docs/superpowers/plans/2026-07-27-private-benchmark-binding-execution-selection.md`.

本增量的新鲜回执为：`cargo fmt --all -- --check`；storage `167 passed, 39 ignored`；
`wave2_local_contracts=passed`；`graph_diff_calculators=passed count=1`；以及 `pnpm check:web`，其中
public SDK `14`、local SDK `70`、Web `156`，并成功完成 production build。live verifier 仅因 Git
change-set evidence 不可用而为 `overall=unobserved`；local Playwright verifier 因未启动 Web server
而未观测。请求的 6 个 `gpt-5.6-luna` worker 因 agent thread limit 已满而被拒绝，仅记录为调度失败。
下一项准入的本地增量是
`docs/superpowers/plans/2026-07-27-private-benchmark-binding-execution-selection.md` 中的 provider-free
exact binding execution-selection bridge。
### 2026-07-27 Private Binding Execution Selection / 2026-07-27 私有绑定执行选择

The exact immutable binding now feeds the existing private benchmark execution service through a
provider-free `BenchmarkDefinitionBindingExecutionSelection`. Scope, stable dataset membership,
deterministic case count, replay, and redacted diagnostics are enforced in Rust; no API, SDK, BFF, Web,
provider, migration, or public surface changed. Fresh evidence is selection `2/2`, execution `11/11`,
workspace Rust API `170`, storage `167 passed, 39 ignored`, strict Clippy, Rust `1.85.0`, and Web
public SDK `14`, local SDK `70`, Web `156`, production build. The next admitted increment is the
protected local execution adapter in
`docs/superpowers/plans/2026-07-27-private-benchmark-execution-adapter.md`; the long-term goal remains active.

精确不可变 binding 现已通过 provider-free `BenchmarkDefinitionBindingExecutionSelection` 接入既有私有 benchmark
execution service。Rust 内强制 scope、稳定 dataset membership、确定性 case count、replay 与脱敏 diagnostic；没有
改变 API、SDK、BFF、Web、provider、migration 或 public surface。新鲜证据为 selection `2/2`、execution `11/11`、
workspace Rust API `170`、storage `167 passed, 39 ignored`、strict Clippy、Rust `1.85.0`，以及 Web public SDK `14`、
local SDK `70`、Web `156`、production build。下一项准入增量是
`docs/superpowers/plans/2026-07-27-private-benchmark-execution-adapter.md` 中的 protected local execution adapter；
长期目标保持 active。

### 2026-07-27 Workflow Binding Read Fresh Revalidation / 2026-07-27 Workflow Binding Read 新鲜复核

The current local Workflow context-binding read boundary is revalidated and remains private, exact-commit, redacted, and read-only. Fresh receipts are API `4 passed`, storage `3 passed`, local SDK `70 passed`, BFF route `9 passed`, focused Web binding/presenter/inspector `8 passed`, `cargo fmt --all -- --check`, and `pnpm check:web` with public SDK `14`, local SDK `70`, Web `160`, and a successful production build. The route forwards only request-scoped Bearer credentials, omits cookies, preserves typed failures, and uses `private, no-store`; Web remains `data -> presenter -> screen` with shared bilingual capability states. This does not claim authenticated runtime, browser/visual E2E, PostgreSQL-backed service flow, Git binding, release, or production evidence. The unique current pointer is now the protected local Benchmark execution adapter plan; the long-term goal remains active.

当前本地 Workflow context-binding read boundary 已重新验证，仍保持 private、精确 commit、脱敏与只读。新鲜回执为 API `4 passed`、storage `3 passed`、local SDK `70 passed`、BFF route `9 passed`、聚焦 Web binding/presenter/inspector `8 passed`、`cargo fmt --all -- --check`，以及 `pnpm check:web`（public SDK `14`、local SDK `70`、Web `160`，并成功完成 production build）。route 只转发 request-scoped Bearer credential、忽略 cookie、保留 typed failure 并使用 `private, no-store`；Web 继续使用带双语 shared capability state 的 `data -> presenter -> screen`。这不声称 authenticated runtime、browser/visual E2E、PostgreSQL-backed service flow、Git binding、release 或 production evidence。当前唯一指针已转为 protected local Benchmark execution adapter plan；长期目标保持 active。

### 2026-07-27 Private Benchmark Execution Adapter / 2026-07-27 私有 Benchmark 执行适配器

The protected local execution adapter is now closed at the server/storage contract. The route keeps
the exact project/Context/commit/binding scope, authenticates and authorizes `ContextPermission::Write`
before body parsing and quota, and remains absent from the public router, OpenAPI, and public SDK.
`Idempotency-Key` and a canonical request digest now flow through `BenchmarkExecutionRequest`,
`PersistBenchmarkEvaluationEvidence`, and the memory/PostgreSQL receipt store introduced by migration
`0021_benchmark_execution_idempotency.sql`. Identical retries return `replayed` without evaluator recall;
changed payloads conflict before evaluator invocation; replay reuses stored decision/timestamp data.

受保护的 private local execution adapter 现已在 server/storage contract 层收束。route 保持精确
project/Context/commit/binding scope，在 body parsing 与 quota 前完成 authentication 与
`ContextPermission::Write` authorization，并继续不进入 public router、OpenAPI 与 public SDK。
`Idempotency-Key` 与 canonical request digest 现已贯穿 `BenchmarkExecutionRequest`、
`PersistBenchmarkEvaluationEvidence` 以及迁移 `0021_benchmark_execution_idempotency.sql` 引入的
Memory/PostgreSQL receipt store。相同 retry 返回 `replayed` 且不再次调用 evaluator；payload 改变会在
evaluator invocation 前返回 conflict；replay 复用已存储的 decision/timestamp data。

Fresh local evidence is API execution `3 passed`, storage `169 passed, 39 ignored`, storage/API compile,
and `cargo fmt --all -- --check`. PostgreSQL runtime, authenticated browser-to-BFF-to-Axum execution,
Git change-set, remote CI, operator rehearsal, release, and production promotion remain
`ignored`, `unobserved`, or `deferred`; no public write or release claim was made. Criterion 3 is advanced
but remains open, and the long-term goal remains active. The next increment must be selected after the
full workspace/Web verification and the remaining evidence gaps are recorded.

新鲜本地证据为 API execution `3 passed`、storage `169 passed, 39 ignored`、storage/API compile 以及
`cargo fmt --all -- --check`。PostgreSQL runtime、authenticated browser-to-BFF-to-Axum execution、Git
change-set、remote CI、operator rehearsal、release 与 production promotion 继续按实际情况标记为
`ignored`、`unobserved` 或 `deferred`；本轮没有 public write 或 release claim。条件 3 得到推进但仍开放，
长期目标保持 active。下一增量将在 workspace/Web 全量验证并记录剩余证据缺口后选择。

### 2026-07-27 Private Knowledge/Memory Projection / 2026-07-27 私有 Knowledge/Memory 投影

The Knowledge/Memory projection increment is now revalidated and closed at its current local read
boundary. Rust-compatible deterministic identity helpers bind every projection to the exact
`context:{ContextId}` scope and replay identity. API and local SDK envelopes now carry and validate
`source_project_id` and `source_commit_id`; authentication, authorization, and projection failures
remain restricted to the SDK allowlist. Web request races are guarded, live-region semantics are
owned by the shared capability presenter/screen path, and the projection remains redacted metadata:
`content_fingerprint` is never raw private content. No public write, provider call, second
`GraphDiff` calculator, or mutation surface was added.

Knowledge/Memory projection increment 已在当前本地只读边界完成复核并收束。Rust 兼容的确定性 identity helper 将每份
projection 绑定到精确的 `context:{ContextId}` scope 与 replay identity。API 与 local SDK envelope 现携带并校验
`source_project_id` 与 `source_commit_id`；authentication、authorization 与 projection failure 仍限制在 SDK
allowlist 内。Web request race 已受保护，live-region 语义由共享 capability presenter/screen 路径统一负责，projection
仍只包含脱敏 metadata：`content_fingerprint` 绝不是 raw private content。本轮没有新增 public write、provider call、
第二个 `GraphDiff` calculator 或 mutation surface。

Fresh local evidence passed: `pnpm check:web` with public SDK `14`, local SDK `85`, Web `179`, and
the production build; API `182` tests; storage `169 passed, 39 ignored`; `cargo fmt --all -- --check`;
strict offline Clippy; the focused API receipt `3 passed`; Knowledge Context projection
`4 passed`; and Knowledge replay bridge `9 passed`. No secrets, Docker, browser, remote, production,
or Git evidence was accessed; PostgreSQL runtime and authenticated browser evidence remain
unobserved, while external release evidence remains deferred. Criterion 3 is advanced but not
closed, and the long-term goal remains active.

新鲜本地证据均已通过：`pnpm check:web`（public SDK `14`、local SDK `85`、Web `179`，并完成 production build）；API
`182` 项测试；storage `169 passed, 39 ignored`；`cargo fmt --all -- --check`；strict offline Clippy；聚焦 API 回执
`3 passed`；Knowledge Context projection `4 passed`；以及 Knowledge replay bridge `9 passed`。本轮未访问 secrets、Docker、
browser、remote、production 或 Git evidence；PostgreSQL runtime 与 authenticated browser evidence 仍为 `unobserved`，
外部 release evidence 仍延期。条件 3 得到推进但尚未关闭，长期目标保持 active。

**Next admitted increment / 下一项准入增量：** Add private read-only exact-commit Knowledge/Memory
persistence and replay: introduce a reusable storage projection repository, preserve Memory/PostgreSQL
parity, and replace the request-time fixture with persisted data keyed by the exact `(project, Context,
commit)` tuple. This requires a new bilingual Necessity Record and focused tests before broader
integration. It must not add public writes, new UI, provider calls, raw private content, Docker,
production claims, or another graph-diff implementation.

**下一项准入增量：** 增加 private read-only 的 exact-commit Knowledge/Memory 持久化与 replay：引入可复用的 storage
projection repository，保持 Memory/PostgreSQL parity，并以精确 `(project, Context, commit)` tuple 的持久化数据替换
request-time fixture。该工作必须先具备新的双语 Necessity Record 与聚焦测试，再进行更广泛集成；不得新增 public write、
新 UI、provider call、raw private content、Docker、production claim 或另一套 graph-diff 实现。

### 2026-07-27 Knowledge/Memory Exact-Commit Persistence / 2026-07-27 Knowledge/Memory 精确提交持久化

The private exact-commit Knowledge/Memory persistence increment is implemented and verified at the
local contract boundary. A reusable storage repository now persists only the redacted Context
projection under exact `(project, Context, commit)` scope, replays identical sources, rejects
immutable conflicts, and fails closed on schema or scope drift. Memory and PostgreSQL adapters share
the same port; PostgreSQL mode is wired through the private API adapter. Migration `0022` adds
append-only storage and composite project/Context/commit foreign-key integrity. No public REST,
OpenAPI, SDK, Web mutation, provider, raw-content, or GraphDiff surface changed. The long-term goal
remains active; this is progress on the Context-first and replayable-history criteria, not closure.

本次 private exact-commit Knowledge/Memory persistence 增量已在本地 contract boundary 实现并验证。可复用
storage repository 仅在精确 `(project, Context, commit)` scope 下保存脱敏 Context projection，能够 replay 相同
source、拒绝 immutable conflict，并在 schema 或 scope drift 时 fail closed。Memory 与 PostgreSQL adapter 共用
同一 port；PostgreSQL mode 已通过 private API adapter 接入。迁移 `0022` 增加 append-only storage 与 project/Context/
commit composite foreign-key integrity。没有改变 public REST、OpenAPI、SDK、Web mutation、provider、raw-content
或 GraphDiff surface。长期目标保持 active；本增量推进 Context-first 与可回放历史条件，但不构成收束。

Fresh receipts are focused storage `2/2`, focused API `6/6`, workspace Rust API `182 passed`, storage
`171 passed, 39 ignored`, strict offline Clippy, `cargo fmt --all -- --check`, and `pnpm check:web`
with public SDK `14`, local SDK `85`, Web `179`, and production build. `psql` is unavailable and
Docker/virtualization is disabled, so PostgreSQL runtime, authenticated browser, Git, remote CI,
operator rehearsal, release, and production evidence remain `unobserved` or `deferred`; no secret was
read or external receipt implied.

新鲜回执为 focused storage `2/2`、focused API `6/6`、workspace Rust API `182 passed`、storage
`171 passed, 39 ignored`、strict offline Clippy、`cargo fmt --all -- --check`，以及
`pnpm check:web`（public SDK `14`、local SDK `85`、Web `179`、production build）。`psql` 不可用且
Docker/virtualization 当前关闭，因此 PostgreSQL runtime、authenticated browser、Git、remote CI、operator
rehearsal、release 与 production evidence 仍为 `unobserved` 或 `deferred`；未读取 secret，也未暗示外部 receipt。

The next admitted pointer is a new bilingual Necessity Record for the commit-associated ContextGraph
snapshot domain/repository contract. Version-backed graph comparison is deferred until that contract
and its focused tests are green; the long-term goal must continue active.

下一项准入指针是在实现前新增双语 Necessity Record，收束 commit-associated ContextGraph snapshot
domain/repository contract。version-backed graph comparison 要等该 contract 与聚焦测试全绿后再接入；长期目标
继续保持 active。

## 2026-07-27 Commit-Associated ContextGraph Snapshot Receipt / 2026-07-27 Commit 关联 ContextGraph Snapshot 回执

The snapshot contract and its existing read integration are now implemented locally. Typed
`CommitGraphSnapshotScope` enforces exact project/Context/commit identity; Memory and PostgreSQL
ports derive ownership server-side, retain immutable replay/conflict semantics, and fail closed on
unknown or malformed scope. The API resolves both commit scopes before reading the existing graph
diff route, and `GraphDiff::between` remains the only calculator. The guarded commit route uses the
same ownership boundary before creating a new snapshot. The former public version-backed graph-diff
read route, OpenAPI operation, and public SDK method are retired; the read is now protected-local
only through the non-public local SDK/BFF. The separate public pure graph-diff POST calculation and
`GraphDiff::between` semantics remain unchanged.

commit-associated snapshot contract 及现有 read integration 已在本地完成。typed
`CommitGraphSnapshotScope` 强制 exact project/Context/commit identity；Memory 与 PostgreSQL port 在服务端解析
ownership，保持 immutable replay/conflict 语义，并对 unknown 或 malformed scope fail closed。API 会在读取现有
graph diff route 前解析两个 commit scope，`GraphDiff::between` 仍是唯一 calculator。guarded commit route 在创建
新 snapshot 前使用同一 ownership boundary。原 public version-backed graph-diff read route、OpenAPI operation 与
public SDK method 已退役；该 read 现在只通过 non-public local SDK/BFF 的 protected-local boundary 提供。独立的
public pure graph-diff POST calculation 与 `GraphDiff::between` semantics 保持不变。

Fresh local evidence is `cargo fmt --all -- --check`, workspace Rust tests with API `184 passed` and
storage `179 passed, 39 ignored`, strict offline workspace Clippy, API typed-scope `2 passed`, storage
snapshot repository `5 passed`, auth `44 passed`, and `pnpm check:web` including the production build.
The PostgreSQL runtime was not observed because `psql` is unavailable and Docker/virtualization is
disabled; authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`. No secrets were read.

新鲜本地证据为 `cargo fmt --all -- --check`、workspace Rust test（API `184 passed`、storage `179 passed, 39 ignored`）、
strict offline workspace Clippy、API typed-scope `2 passed`、storage snapshot repository `5 passed`、auth `44 passed`，
以及包含 production build 的 `pnpm check:web`。由于 `psql` 不可用且 Docker/virtualization 已关闭，PostgreSQL runtime
未观测；authenticated browser、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或
`deferred`。未读取 secrets。

This is a local private contract receipt, not a public-write, release, or production-readiness decision. The long-term
goal remains active. The next pointer is a new bilingual Necessity Record for the private semantic/behavior/evaluation
diff contract; no implementation begins until its dependencies and evidence needs are recorded.

这是本地 private contract 回执，不是 public-write、release 或 production-readiness decision。长期目标保持 active。下一指针
是为 private semantic/behavior/evaluation diff contract 新增双语 Necessity Record；在记录依赖与证据需求前不开始实现。

## 2026-07-28 Private Commit-Scoped Diff Snapshot Persistence / 2026-07-28 私有 Commit 范围 Diff Snapshot 持久化

The admitted storage-first increment is complete and locally verified. The immutable
`ContextDiffSnapshotV1Record` binds validated semantic, behavior, and evaluation inputs to the
exact `(ProjectId, ContextId, CommitId, schema_version)` tuple with deterministic digest,
microsecond capture normalization, Memory/PostgreSQL parity, immutable replay/conflict behavior,
and fail-closed scope/schema validation. `PersistedContextDiffReviewService` is the only storage
consumer and delegates comparison to the existing diff-engine review service; it does not add a
second graph-diff algorithm or any transport surface.

准入的 storage-first 增量现已完成并取得本地验证。不可变 `ContextDiffSnapshotV1Record` 将经过验证的 semantic、
behavior 与 evaluation input 绑定到 exact `(ProjectId, ContextId, CommitId, schema_version)` tuple，具备确定性
digest、微秒 capture normalization、Memory/PostgreSQL parity、immutable replay/conflict 与 fail-closed
scope/schema validation。`PersistedContextDiffReviewService` 是唯一 storage consumer，并委托既有 diff-engine review
service 进行 comparison；没有新增第二套 graph-diff algorithm 或任何 transport surface。

Fresh local verification is recorded by the implementation plan and current full receipt: storage
`186 passed, 39 ignored`, workspace Rust `186 passed, 39 ignored`, format, strict offline Clippy,
locked Rust `1.85.0`, and `pnpm check:web` with public SDK `15`, local SDK `92`, Web `185`, and a
successful production build. PostgreSQL runtime, authenticated browser, Git change-set, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`; no secrets were read.
The long-term goal remains active.

新鲜本地验证已由 implementation plan 与当前全量回执记录：storage `186 passed, 39 ignored`、workspace Rust
`186 passed, 39 ignored`、format、strict offline Clippy、锁定 Rust `1.85.0`，以及 `pnpm check:web`（public SDK `15`、
local SDK `92`、Web `185`、production build）通过。PostgreSQL runtime、authenticated browser、Git change-set、
remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`；未读取 secrets。长期目标保持 active。

## 2026-07-28 Private Typed Branch-Head Discovery Receipt / 2026-07-28 私有 Typed Branch-Head Discovery 回执

The private branch-head read contract is now locally verified. `ContextBranchHead` carries typed
`ContextId`, `BranchName`, nullable `CommitId`, and `u64` revision; the repository port distinguishes
unknown Context, unknown branch, malformed stored names, invalid/overflowing revisions, and
cross-Context heads. Memory now represents nullable unborn branches, performs exact branch lookup
without scanning unrelated rows, and sorts rehydrated names deterministically; PostgreSQL performs
the matching exact-scope read. Guarded writer CAS, idempotency, audit, and revision update semantics
remain unchanged. No branch create/fork/rename/delete/merge/rollback or transport was added.

private branch-head read contract 现已在本地验证。`ContextBranchHead` 携带 typed `ContextId`、`BranchName`、可空
`CommitId` 与 `u64` revision；repository port 区分 unknown Context、unknown branch、malformed stored name、
invalid/overflowing revision 与 cross-Context head。Memory 现可表达 nullable unborn branch，exact branch lookup 不再
扫描无关 row，并对 rehydrated name 做确定性排序；PostgreSQL 执行匹配的 exact-scope read。Guarded writer 的 CAS、
idempotency、audit 与 revision update 语义保持不变。本增量未新增 branch create/fork/rename/delete/merge/rollback
或 transport。

Fresh evidence is focused `branch_head` `2 passed`, Memory adapter regressions `3 passed`, storage
`186 passed, 39 ignored`, workspace Rust `186 passed, 39 ignored`, `cargo fmt --all -- --check`,
strict offline workspace Clippy, locked Rust `1.85.0` check, and `pnpm check:web` with public SDK
`15`, local SDK `92`, Web `185`, and production build. PostgreSQL runtime is unobserved because
`psql` is unavailable and Docker/virtualization is disabled; authenticated browser, Git change-set,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`. No
secrets were read. The long-term goal remains active.

新鲜证据为 focused `branch_head` `2 passed`、Memory adapter regression `3 passed`、storage `186 passed, 39 ignored`、
workspace Rust `186 passed, 39 ignored`、`cargo fmt --all -- --check`、strict offline workspace Clippy、锁定 Rust
`1.85.0` check，以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185`、production build）。由于 `psql`
不可用且 Docker/virtualization 已关闭，PostgreSQL runtime 未观测；authenticated browser、Git change-set、remote
CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。未读取 secrets。长期目标保持 active。

This receipt advances the replayable version-history and future branch/merge/replay criteria only;
merge/rollback remains deferred until independent merge-base and conflict contracts exist. The next
increment must begin with a new bilingual Necessity Record and may not expand public writes or add a
second graph-diff calculator.

本回执只推进可回放版本历史与未来 branch/merge/replay 条件；在独立 merge-base 与 conflict contract 具备前，
merge/rollback 继续后置。下一增量必须先新增双语 Necessity Record，不得扩大 public write 或新增第二个 graph-diff calculator。

protected-local commit graph-diff read 回执已在 local contract boundary 通过，相关文档也已收束。下一项正式准入增量为
`docs/superpowers/plans/2026-07-28-private-commit-scoped-diff-snapshot-persistence.md`。当前实现进行中，尚未声称通过。
目标是对 semantic、behavior 与 evaluation diff 做 private exact snapshot persistence，并严格按
`(ProjectId, ContextId, CommitId, schema_version)` 精确 tuple 建立持久化身份。本计划服务条件 2 与条件 4，且当前只做
storage 第一波。

该准入计划不得新增或修改任何 public API、public SDK 或 Web mutation；必须继续保持 `GraphDiff::between` 为唯一
graph-diff calculator。PostgreSQL runtime、authenticated browser runtime、Git evidence、remote CI、operator rehearsal、
release 与 production evidence 均未被观测。本轮不作实现通过、release、production-readiness 或长期目标完成声明；长期目标
保持 active。

## 2026-07-28 Private Merge-Base and Ancestry Conflict Contract / 2026-07-28 私有 Merge-Base 与 Ancestry Conflict 契约

The previously admitted versioning increment is now locally implemented and verified. The
framework-independent `CommitGraph` validates duplicate commits, duplicate parents, missing
parents, cross-Context parent edges, disconnected nodes outside one graph Context, and cycles.
`MergePlan::resolve` deterministically classifies identical tips, fast-forward ancestry, one-base
three-way ancestry, no common ancestor, and ambiguous maximal common bases. It consumes topology
only; content-level conflict resolution, merge writes, rollback, and all graph diff calculation
remain outside this boundary.

此前准入的 versioning 增量现已在本地实现并验证。framework-independent `CommitGraph` 会校验 duplicate commit、
duplicate parent、missing parent、跨 Context parent edge、脱离 graph Context 的断开节点与 cycle。`MergePlan::resolve`
以确定性方式区分 identical tip、fast-forward ancestry、single-base three-way ancestry、no common ancestor 与
ambiguous maximal common base。它只消费 topology；content-level conflict resolution、merge write、rollback 与全部
graph diff calculation 仍在本边界之外。

Fresh local evidence: `cargo test -p contextlab-versioning --quiet` passed `31` tests; focused
versioning strict offline Clippy, `cargo fmt --all -- --check`, workspace Rust, locked Rust
`1.85.0`, and repository Web checks are required/recorded only from fresh commands. Static scope
continues to find one `impl GraphDiff`. PostgreSQL/Docker runtime, authenticated browser, Git,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; no
secret was read. The long-term goal remains active.

新鲜本地证据为 `cargo test -p contextlab-versioning --quiet` 通过 `31` 项测试；focused versioning strict offline
Clippy、`cargo fmt --all -- --check`、workspace Rust、锁定 Rust `1.85.0` 与仓库 Web checks 只在新鲜命令实际完成后记录。
静态 scope 仍只发现一个 `impl GraphDiff`。PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator
rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未读取 secret。长期目标保持 active。

The next admitted increment is not a merge writer. It requires a new bilingual Necessity Record
for the smallest dependency-ready private storage/repository or content-conflict contract, with
fresh red/green evidence before any branch merge, rollback, public transport, or UI mutation work.

下一项准入增量不是 merge writer。任何 branch merge、rollback、public transport 或 UI mutation 工作前，必须先为最小的、
依赖已满足的 private storage/repository 或 content-conflict contract 新增双语 Necessity Record，并取得新鲜 red/green 证据。

## 2026-07-29 Private ContextGraph Three-Way Conflict Classification / 2026-07-29 私有 ContextGraph 三路冲突分类

The admitted pure-Rust conflict-safety increment is now implemented and locally verified.
`GraphSnapshotRef` binds `(ProjectId, ContextId, CommitId)` for base, left, and right snapshots.
`GraphMergeConflictClassifier` accepts only matching `MergePlan::ThreeWay`, calls the existing
`GraphDiff::between` once for each base-to-branch comparison, and classifies deterministic Clean,
Equivalent, or Conflict node/edge changes. It does not produce a merge result or mutate storage.

本次准入的 pure-Rust conflict-safety 增量现已实现并完成本地验证。`GraphSnapshotRef` 为 base、left、right snapshot
绑定 `(ProjectId, ContextId, CommitId)`。`GraphMergeConflictClassifier` 只接受匹配的 `MergePlan::ThreeWay`，对每个
base-to-branch comparison 各调用一次既有 `GraphDiff::between`，并对 node/edge change 返回确定性的 Clean、Equivalent
或 Conflict。它不生成 merge result，也不修改 storage。

Fresh local evidence passed: diff-engine focused targets `6`, workspace Rust `186 passed, 39 ignored`, format,
strict offline workspace Clippy, locked Rust `1.85.0`, and `pnpm check:web` with public SDK `15`, local SDK `92`,
Web `185`, and production build. Static inspection observed `impl GraphDiff count=1`. PostgreSQL/Docker runtime,
authenticated browser, Git change-set, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`; no secrets were read. The long-term goal remains active.

新鲜本地证据为 diff-engine focused targets `6`、workspace Rust `186 passed, 39 ignored`、format、strict offline workspace
Clippy、锁定 Rust `1.85.0`，以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185` 与 production build）。
静态 inspection 观测到 `impl GraphDiff count=1`。PostgreSQL/Docker runtime、authenticated browser、Git change-set、
remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未读取 secrets。长期目标保持 active。

The next decision remains private and evidence-gated: bind this classification to a storage-owned
exact-scope merge-input/repository contract before considering any merge or rollback writer. A new
bilingual Necessity Record and fresh red/green evidence are required; no public surface is admitted.

下一项仍是 private 且 evidence-gated：在考虑任何 merge 或 rollback writer 前，先将本 classification 绑定到
storage-owned exact-scope merge-input/repository contract。必须新增双语 Necessity Record 并取得新鲜 red/green 证据；不准入任何 public surface。

## 2026-07-29 Private Workflow Binding Interaction Evidence / 2026-07-29 私有 Workflow Binding 交互证据

The private Workflow context-binding read now has a fresh Web lifecycle receipt in addition to its
existing API, SDK, BFF, and static presentation contracts. The inspector test executes the real
control handler, asserts the exact encoded same-origin BFF path, request-memory Bearer header,
`credentials: "omit"`, and `cache: "no-store"`, then observes a ready projection bound to the
selected Context commit. A second case observes the bilingual typed 403 state and proves the
upstream diagnostic is not rendered. The two hardening repairs are present: the protected Axum read
router applies `private_no_store_response`, and request-generation guards reject late responses for
an old selection. Their focused regressions and the behavior-neutral formatting follow-up passed,
so the local hardening gate is full-green. This closes only the local interaction-evidence gap and
adds no route, schema, SDK method, public write, Web mutation, provider call, or second diff calculator.

私有 Workflow Context binding read 在既有 API、SDK、BFF 与静态 presentation contract 之外，现已获得新鲜 Web lifecycle
回执。inspector test 真正执行 control handler，断言精确编码的同源 BFF path、request-memory Bearer header、
`credentials: "omit"` 与 `cache: "no-store"`，并观测绑定到选定 Context commit 的 ready projection。第二个 case 观测
双语 typed 403 state，并证明上游 diagnostic 不会被渲染。两个 hardening repair 已存在：protected Axum read router 已应用
`private_no_store_response`，request-generation guard 会拒绝旧 selection 的 late response；其 focused regression 已通过。
但行为等价的 formatting follow-up 现已通过，因此 local hardening gate 已 full-green。本增量只收束 local interaction
evidence gap，不新增 route、schema、SDK method、public write、Web mutation、provider call 或第二个 diff calculator。

Fresh local evidence is the package test command `pnpm --filter @contextlab/web test -- src/app/local-workflow-context-bindings.test.tsx`,
which expands to the Web suite and passed `188` tests, plus the focused API hardening command
`cargo test -p contextlab-api local_workflow_bindings --quiet` with `4 passed`. `cargo test --workspace --quiet` passed API `183`
and storage `186 passed, 39 ignored`; strict offline Clippy and locked Rust `1.85.0` check passed; `pnpm check:web` passed public
SDK `15`, local SDK `92`, Web `188`, and the local Web production build. `cargo fmt --all -- --check` passed after the behavior-
neutral formatting follow-up in `server/api/src/lib.rs`, so this local gate is full-green. PostgreSQL runtime, authenticated browser, Git change-set, and
remote CI remain `unobserved`; operator rehearsal, release, and production remain `deferred`. No Docker runtime or secrets were
used. The long-term goal remains active.

新鲜本地证据为 package test command `pnpm --filter @contextlab/web test -- src/app/local-workflow-context-bindings.test.tsx`（该命令
展开为 Web suite 并通过 `188` 项），以及 focused API hardening command `cargo test -p contextlab-api local_workflow_bindings --quiet`
（`4 passed`）。`cargo test --workspace --quiet` 通过 API `183`、storage `186 passed, 39 ignored`；strict offline Clippy 与锁定
Rust `1.85.0` check 通过；`pnpm check:web` 通过 public SDK `15`、local SDK `92`、Web `188` 与本地 Web production build。
`cargo fmt --all -- --check` 已在 `server/api/src/lib.rs` 的行为等价 formatting follow-up 后通过，因此 local gate 已
full-green。PostgreSQL runtime、
authenticated browser、Git change-set 与 remote CI 仍为 `unobserved`；operator rehearsal、release 与 production 仍为 `deferred`。
未启用 Docker runtime、未读取 secrets。长期目标保持 active。

The private atomic base/left/right ContextGraph snapshot read contract in
`docs/superpowers/plans/2026-07-29-private-atomic-context-graph-snapshot-read.md` is now locally
verified. It uses one Memory guard, one PostgreSQL read transaction, and one storage review call,
while preserving private transport boundaries. The next admitted work must begin with a new
bilingual Necessity Record for the next dependency-ready private Context editing, benchmark, or
replay contract; no merge writer, rollback, public transport, or production claim is authorized.

`docs/superpowers/plans/2026-07-29-private-atomic-context-graph-snapshot-read.md` 中的私有 atomic base/left/right ContextGraph
snapshot read contract 现已完成本地验证：Memory 使用一个 guard，PostgreSQL 使用一个 read transaction，storage review 只
调用一次 batch port，并保持 private transport boundary。下一项工作必须先为下一项依赖就绪的 private Context editing、
benchmark 或 replay contract 新增双语 Necessity Record；本回执不授权 merge writer、rollback、public transport 或
production claim。

## 2026-07-29 Private Context Commit Replay State / 2026-07-29 私有 Context Commit 回放状态

The new bilingual Necessity Record in
`docs/superpowers/plans/2026-07-29-private-context-commit-replay-state-projection.md` was admitted
within the reusable versioning boundary. `ReplayState` now folds ordered normal-parent
`ContextCommit` history into deterministic descriptor-only component and `Uses` relationship state,
with exact Context/parent checks, atomic failed transitions, removal-to-absence, and explicit
`REPLAY_STATE_SCHEMA_VERSION`. `ReplayState::from_commits` provides the one-shot projection entry
point. This is private Rust core only; no storage, transport, public write, Web mutation, or second
`GraphDiff` calculator was added.

新的双语 Necessity Record
`docs/superpowers/plans/2026-07-29-private-context-commit-replay-state-projection.md` 已在可复用 versioning boundary 内准入。
`ReplayState` 现将有序 normal-parent `ContextCommit` history 确定性折叠为 descriptor-only component 与 `Uses`
relationship state，具备 exact Context/parent check、失败 transition 原子性、removal-to-absence 与显式
`REPLAY_STATE_SCHEMA_VERSION`；`ReplayState::from_commits` 提供一次性 projection entry point。本增量仅限 private
Rust core，未新增 storage、transport、public write、Web mutation 或第二个 `GraphDiff` calculator。

Fresh local evidence: versioning focused tests `37 passed`, package format passed, and strict offline package Clippy passed.
The independent Web benchmark workspace sidecar added exact-scope loading lifecycle coverage; the repository Web suite
observed `189 passed`, TypeScript/lint passed, and the local production build passed. Git change-set, PostgreSQL/Docker runtime,
authenticated browser, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`; no secret was
read. The long-term goal remains active.

新鲜本地证据：versioning focused test `37 passed`，package format 与 strict offline package Clippy 通过。独立 Web benchmark
workspace sidecar 增加 exact-scope loading lifecycle coverage；仓库 Web suite 观测到 `189 passed`，TypeScript/lint 与本地
production build 通过。Git change-set、PostgreSQL/Docker runtime、authenticated browser、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`；未读取 secret。长期目标保持 active。

This receipt advances Criteria 1 and 2 but does not close either criterion or the repository. The next increment must begin
with a new bilingual Necessity Record for the next dependency-ready replay consumer, private Context editing, or benchmark
evidence gap; external deployment evidence remains explicitly deferred.

本回执推进条件 1 与 2，但不关闭任一条件或整个仓库。下一增量必须先为下一条依赖就绪的 replay consumer、private Context
editing 或 benchmark evidence gap 新增双语 Necessity Record；外部部署 evidence 继续明确延期。

## 2026-07-29 Private Context Replay Metadata / 2026-07-29 私有 Context 回放元数据

The metadata replay increment is now `completed` / `verified locally`. `ContextMetadataPayload` is a
typed schema-versioned payload, `UpdatedMetadata` replays into `ReplayState`, malformed or missing
payloads fail closed, failed commits remain atomic, and the outer `ContextChangeWire` rejects unknown
fields after a focused schema-drift regression. This remains private Rust in-memory state only.

Context metadata replay 增量现为 `completed` / `verified locally`。`ContextMetadataPayload` 是带 schema version 的 typed
payload，`UpdatedMetadata` 会回放进入 `ReplayState`，缺失或 malformed payload fail closed，失败 commit 保持原子性；新增
focused schema-drift regression 后，外层 `ContextChangeWire` 也会拒绝 unknown fields。本增量仍仅是 private Rust in-memory state。

Fresh local evidence after the repair: versioning `41 passed`; workspace Rust passed with API `183` and
storage `186 passed, 39 ignored`; format, strict offline workspace Clippy, and locked Rust `1.85.0`
check passed; `pnpm check:web` passed public SDK `15`, local SDK `92`, Web `189`, TypeScript/lint,
and the local production build; static inspection found one `impl GraphDiff`. PostgreSQL/Docker
runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`; no secrets, public write, transport, or Web mutation were added. The
long-term goal remains active and this does not close Criteria 1 or 2.

修复后的新鲜本地证据：versioning `41 passed`；workspace Rust 通过，其中 API `183`、storage `186 passed, 39 ignored`；format、
strict offline workspace Clippy 与锁定 Rust `1.85.0` check 通过；`pnpm check:web` 通过 public SDK `15`、local SDK `92`、Web
`189`、TypeScript/lint 与本地 production build；静态检查发现一个 `impl GraphDiff`。PostgreSQL/Docker runtime、authenticated
browser、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`；未新增 secrets、public
write、transport 或 Web mutation。长期目标保持 active，本增量不关闭条件 1 或 2。

The next admitted increment requires a new bilingual Necessity Record for a private storage
`ReplayState::from_commits` adapter with Memory/PostgreSQL parity; replay serialization, descriptor
stale-write preconditions, and component metadata content limits remain separate future boundaries.

下一项准入增量必须先为 private storage `ReplayState::from_commits` adapter 新增双语 Necessity Record，并取得 Memory/PostgreSQL
parity；ReplayState serialization、descriptor stale-write precondition 与 component metadata content limit 继续作为后续独立边界。

## 2026-07-29 Private Storage ReplayState Adapter / 2026-07-29 私有 Storage ReplayState Adapter

The admitted storage consumer is now `completed` / `verified locally`. The private
`ContextReplayStateAtCommitRepository` reconstructs persisted commit records with their original
identities and delegates replay to the reusable versioning `ReplayState::from_commits` policy.
Memory and PostgreSQL both use validated normal-parent ancestry; PostgreSQL additionally validates
recursive-history flags and malformed rows before conversion. This advances Criteria 1 and 2 only.

准入的 storage consumer 现为 `completed` / `verified locally`。私有
`ContextReplayStateAtCommitRepository` 会保留持久化 commit record 的原始 identity，并将 replay 委托给可复用
versioning `ReplayState::from_commits` policy。Memory 与 PostgreSQL 均使用经过校验的 normal-parent ancestry；PostgreSQL
还会在转换前校验 recursive-history flags 与 malformed rows。本增量只推进条件 1 与 2。

Fresh local evidence: the Memory trait-qualified parity test passed `1`; the PostgreSQL
replay-history SQL/row contract tests passed `4`; the storage package passed `193` library tests
with `39 ignored` and all auxiliary targets passed; `cargo fmt --all -- --check` passed. A PostgreSQL
worker failed at the external endpoint with `502`, so the Integration Lead took over the bounded
static verification; this is not a PostgreSQL runtime receipt. Docker/PostgreSQL runtime,
authenticated browser, Git change-set, remote CI, operator rehearsal, release, and production
remain `unobserved` or `deferred`. No public REST/OpenAPI/SDK/Web write, migration, or secret access
was added. The long-term goal remains active.

新鲜本地证据：Memory trait-qualified parity test 通过 `1` 项；PostgreSQL replay-history SQL/row contract test 通过 `4` 项；
storage package 通过 `193` 个 library test、`39` 个 ignored，所有 auxiliary target 均通过；`cargo fmt --all -- --check` 通过。
PostgreSQL worker 在外部 endpoint 处收到 `502`，因此由 Integration Lead 接管有界 static verification；这不是 PostgreSQL
runtime receipt。Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与
production 继续为 `unobserved` 或 `deferred`。未新增 public REST/OpenAPI/SDK/Web write、migration 或 secret access。长期目标保持 active。

The final workspace verification also passed `cargo test --workspace --quiet --no-fail-fast` (API
`183`, versioning `45`, storage `193 passed, 39 ignored`, all other targets passed), strict offline
workspace Clippy, locked Rust `1.85.0` check, and `pnpm check:web` (public SDK `15`, local SDK `92`,
Web `189`, TypeScript/lint, and local production build). Static inspection observed one
`impl GraphDiff`. These are local engineering receipts only.

最终 workspace verification 还通过了 `cargo test --workspace --quiet --no-fail-fast`（API `183`、versioning `45`、storage
`193 passed, 39 ignored`，其余 target 均通过）、strict offline workspace Clippy、锁定 Rust `1.85.0` check 与
`pnpm check:web`（public SDK `15`、local SDK `92`、Web `189`、TypeScript/lint 与本地 production build）。静态检查观测到
唯一一个 `impl GraphDiff`。这些仅是本地工程 receipt。

The next implementation must begin with a new bilingual Necessity Record for a dependency-ready
Context editing, benchmark, or replay consumer increment. ReplayState serialization, descriptor
stale-write preconditions, and component metadata content policy remain separate future boundaries.

下一项实现必须先为依赖就绪的 Context editing、benchmark 或 replay consumer 增量新增双语 Necessity Record。ReplayState
serialization、descriptor stale-write precondition 与 component metadata content policy 继续作为独立的后续边界。

## 2026-07-29 Private ReplayState Serialization Envelope / 2026-07-29 私有 ReplayState 序列化信封

The next private replay increment is now `completed` / `verified locally`. Versioning owns an
explicit `ReplayStateSnapshotV1` envelope and canonical JSON conversion for exact replay state.
It preserves stable Context/commit identity, metadata, descriptor-only components, and Uses
relationships; restore rejects schema drift, unknown fields, duplicate identities, nil IDs, and
relationships with absent endpoints. This advances Criteria 1 and 2 only and does not create a
storage writer or transport.

下一项 private replay 增量现为 `completed` / `verified locally`。versioning 现拥有明确的 `ReplayStateSnapshotV1` envelope
与 exact replay state 的 canonical JSON conversion。它保留稳定 Context/commit identity、metadata、descriptor-only component 与
Uses relationship；restore 会拒绝 schema drift、unknown field、重复 identity、nil ID 以及 endpoint 缺失的 relationship。本增量
只推进条件 1 与 2，不创建 storage writer 或 transport。

Fresh local evidence: serialization integration `8 passed`; versioning package `42` unit tests
plus `8` integration tests passed; workspace Rust passed with API `183`, storage `193 passed, 39
ignored`, and all other targets passed; format, strict offline workspace Clippy, locked Rust
`1.85.0`, and `pnpm check:web` passed with public SDK `15`, local SDK `92`, Web `189`, TypeScript/
lint, and local production build. No storage/API/SDK/Web/migration surface or public write changed;
PostgreSQL runtime, authenticated browser, Git, remote CI, operator, release, and production remain
`unobserved` or `deferred`. The long-term goal remains active.

新鲜本地证据：serialization integration `8 passed`；versioning package `42` 个 unit test 加 `8` 个 integration test 通过；
workspace Rust 通过，其中 API `183`、storage `193 passed, 39 ignored`，其余 target 均通过；format、strict offline workspace
Clippy、锁定 Rust `1.85.0` 与 `pnpm check:web` 通过（public SDK `15`、local SDK `92`、Web `189`、TypeScript/lint 与本地
production build）。未改变 storage/API/SDK/Web/migration surface 或 public write；PostgreSQL runtime、authenticated browser、
Git、remote CI、operator、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active。

The next implementation still requires a new bilingual Necessity Record for a dependency-ready
Context editing, benchmark evidence, or replay consumer increment. Serialization remains a local
Rust contract and does not close Criteria 1 or 2.

下一项实现仍必须先为依赖就绪的 Context editing、benchmark evidence 或 replay consumer 增量新增双语 Necessity Record。
serialization 仍是本地 Rust contract，不关闭条件 1 或 2。

## 2026-07-29 Private Branch-Head Discovery / 2026-07-29 私有 Branch-Head 发现

The private typed branch-head discovery slice is now `completed` / `verified locally`. It
connects the existing Rust `ContextBranchRepository` boundary to a default-off protected local
read, a non-public SDK, a same-origin BFF, and a shared design-system inspector. Exact Context
scope, authorization/audit/rate-limit middleware, deterministic server ordering, nullable unborn
heads, private/no-store responses, and redacted structured failures are covered. Web now reuses
the SDK V1 parser and fails closed on canonical UUID, legal branch name, duplicate, unsorted, or
unknown-field drift; preview display targets remain usable without pretending to be transport IDs.

私有 typed branch-head discovery slice 现为 `completed` / `verified locally`。它把既有 Rust
`ContextBranchRepository` boundary 接入 default-off protected local read、非公开 SDK、同源 BFF 与 shared
design-system inspector。Exact Context scope、authorization/audit/rate-limit middleware、服务端确定性排序、nullable
unborn head、private/no-store response 与脱敏结构化 failure 均有覆盖。Web 现复用 SDK V1 parser，并对 canonical UUID、
合法 branch name、duplicate、unsorted 或 unknown-field drift fail closed；preview display target 仍可用，但不会伪装成
transport ID。

Fresh local evidence / 新鲜本地证据: API handler `6 passed`, protected router `2 passed`, storage contract `7 passed`,
local SDK focused `7 passed` and full `99 passed`, BFF `3 passed`, Web branch-head `5 passed`, workspace Rust passed with
storage `193 passed, 39 ignored`, strict offline Clippy, locked offline check, format, and `pnpm check:web` with public SDK
`15`, local SDK `99`, Web `197`, TypeScript/lint, and production build. Static inspection observed one `impl GraphDiff`.

本地外部边界仍诚实标记：PostgreSQL/Docker runtime、authenticated browser、Git change-set 为 `unobserved`；remote CI、operator
rehearsal、release 与 production 为 `deferred`。本增量没有新增 public REST/OpenAPI/public SDK write、branch mutation、
merge/rollback、migration、operator transport、provider call、secret access 或第二个 GraphDiff calculator。长期目标保持 active；
下一项工作仍必须先写新的双语 Necessity Record。

### 2026-07-30 Private Replay/ContextGraph Consistency / 2026-07-30 私有 Replay/ContextGraph 一致性

This increment is completed and verified locally for the admitted local contract, while the
long-term goal remains active. A storage-private validate_replay_graph_consistency now compares
the reusable ReplayState with the immutable CommitGraphSnapshot for the exact Context and commit
scope. It checks schema versions, Context and commit identity, the Context root, component node
identity/kind/name, context-to-component edges, replayed Uses edges, duplicate edges, and
unexpected nodes/edges. It does not calculate a diff; GraphDiff::between remains the only graph
diff calculator.

本增量对已准入的本地 contract 标记为 completed / verified locally，但长期目标继续保持 active。storage-private
validate_replay_graph_consistency 现在针对 exact Context 与 commit scope 对账可复用 ReplayState 与不可变
CommitGraphSnapshot，校验 schema version、Context 与 commit identity、Context root、component node identity/kind/name、
Context-to-component edge、回放出的 Uses edge、重复 edge 与多余 node/edge。它不计算 diff；GraphDiff::between
仍是唯一 graph diff calculator。

The existing component-specific fail-closed error precedence is preserved. The API internal
WorkspaceDataRepository now forwards the replay port; no public REST/OpenAPI/SDK route or write
surface changed.

既有 component-specific fail-closed error precedence 已保留。API 内部 WorkspaceDataRepository 现在转发 replay port；
没有改变 public REST/OpenAPI/SDK route 或 write surface。

Fresh local evidence / 新鲜本地证据: consistency focused 6 passed; lifecycle 9 passed;
storage 199 passed, 39 ignored; API 189 passed; workspace Rust passed; format, strict offline
Clippy, locked Rust 1.85.0 check; pnpm check:web public SDK 15, local SDK 99, Web 207,
TypeScript/lint and production build; static GRAPH_DIFF_IMPL_COUNT=1.

External boundary / 外部边界: PostgreSQL/Docker runtime, authenticated browser, and Git change-set
remain unobserved; remote CI, operator rehearsal, release and production remain deferred.
The next increment still requires a new bilingual Necessity Record and must directly close a named
local completion criterion; branch/merge/rollback writes and public promotion remain out of scope.

外部边界：PostgreSQL/Docker runtime、authenticated browser、Git change-set 继续为 unobserved；remote CI、operator rehearsal、
release 与 production 继续为 deferred。下一增量仍必须先新增双语 Necessity Record，并直接收束一个命名的本地完成条件；
branch/merge/rollback writes 与 public promotion 继续不在范围内。

## 2026-07-30 Private Versioned ContextGraph Diff Adapter and Replay Consumers / 2026-07-30 私有版本化 ContextGraph Diff 适配器与回放消费者

This parallel local wave completed two independent read-only increments under new bilingual
Necessity Records. `contextlab-diff-engine` now owns the schema-versioned
`VersionedContextGraphDiffReviewRequestV1` and projection. `contextlab-storage` owns
`PersistedContextGraphDiffReviewService`, which reads two exact immutable commit snapshots and
delegates to the diff-engine service. The protected API handler remains an adapter and its route,
response, authentication, authorization, rate limit, OpenAPI, SDK, and Web boundaries are
unchanged. `GraphDiff::between` remains the sole graph-diff calculator.

本次并行本地 wave 在新增双语 Necessity Record 后完成了两条独立的 read-only 增量。
`contextlab-diff-engine` 现负责带 schema version 的 `VersionedContextGraphDiffReviewRequestV1` 与 projection。
`contextlab-storage` 负责 `PersistedContextGraphDiffReviewService`，读取两份 exact immutable commit snapshot，
并委托给 diff-engine service。protected API handler 仍是 adapter，route、response、authentication、authorization、
rate limit、OpenAPI、SDK 与 Web boundary 均未改变。`GraphDiff::between` 仍是唯一 graph-diff calculator。

The CLI/Desktop sidecar adds a typed immutable `ReplayStateSnapshotProjectionV1` consumer using
the existing versioning parser. CLI `replay inspect` and the Desktop staging presenter expose
only canonical, redacted, read-only state; schema drift, unknown fields, invalid relationships,
and unstable ordering fail closed. No replay logic, storage transport, mutation, provider,
secret, or public API was added.

CLI/Desktop sidecar 使用既有 versioning parser 增加 typed immutable `ReplayStateSnapshotProjectionV1` consumer。
CLI `replay inspect` 与 Desktop staging presenter 只暴露 canonical、redacted、read-only state；schema drift、unknown
field、invalid relationship 与不稳定排序都会 fail closed。未新增 replay logic、storage transport、mutation、provider、
secret 或 public API。

Fresh local evidence: diff-engine `3`, storage `3`, protected API `6`, adapter `8`, CLI `5`, and
Desktop `5` focused tests passed; workspace Rust passed with storage `202 passed, 39 ignored` and
API `189`; format, full strict offline Clippy, locked Rust `1.85.0`, and `pnpm check:web` passed
with public SDK `15`, local SDK `99`, Web `207`, TypeScript/lint, and production build. Static
`GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL/Docker runtime, Tauri runtime, authenticated browser, Git,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

新鲜本地证据：diff-engine `3`、storage `3`、protected API `6`、adapter `8`、CLI `5` 与 Desktop `5` 个 focused test
通过；workspace Rust 通过，其中 storage `202 passed, 39 ignored`、API `189`；format、完整 strict offline Clippy、锁定 Rust
`1.85.0` 与 `pnpm check:web` 通过（public SDK `15`、local SDK `99`、Web `207`、TypeScript/lint 与 production build）。
静态 `GRAPH_DIFF_IMPL_COUNT=1`。PostgreSQL/Docker runtime、Tauri runtime、authenticated browser、Git、remote CI、operator
rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

The long-term goal remains active. This wave advances Criteria 1, 2, and 4 but does not close
them or the repository. The next increment must begin with a new bilingual Necessity Record;
public writes, branch/merge/rollback writers, and external deployment gates remain out of scope.

长期目标保持 active。本 wave 推进条件 1、2 与 4，但不关闭这些条件或仓库。下一增量必须先新增双语 Necessity Record；
public write、branch/merge/rollback writer 与 external deployment gate 继续不在范围内。

## 2026-07-30 Private Benchmark Decision Workspace and Workflow Replay Provenance / 2026-07-30 私有 Benchmark Decision Workspace 与 Workflow Replay Provenance

This local parallel wave completed two dependency-ready read/replay increments under separate
bilingual Necessity Records. Benchmark now resolves an exact sealed decision through
`BenchmarkWorkspaceProjectionDecisionQuery` in the reusable storage reader contract, with Memory
and PostgreSQL implementations. The protected decision-keyed route, non-public SDK, same-origin
BFF, and Web adapter reuse the existing sealed workspace projection; Web does not derive cohorts
from run IDs or recalculate evaluation policy. Workflow execution logs now seal a versioned
canonical capability snapshot and digest, and restore/replay rejects schema drift, digest
tampering, capability substitution, and missing capabilities.

本次本地并行 wave 在两份独立双语 Necessity Record 下完成了两项依赖就绪的 read/replay 增量。Benchmark 现通过可复用 storage
reader contract 中的 `BenchmarkWorkspaceProjectionDecisionQuery` 解析 exact sealed decision，并提供 Memory 与 PostgreSQL 实现。
protected decision-keyed route、非公开 SDK、同源 BFF 与 Web adapter 复用既有 sealed workspace projection；Web 不从 run ID 推导 cohort，
也不重新计算 evaluation policy。Workflow execution log 现封存带版本的 canonical capability snapshot 与 digest；restore/replay 会拒绝
schema drift、digest tampering、capability substitution 与 missing capability。

The first benchmark adapter worker returned a valid storage-boundary blocker; a Luna backup wrote
the adapter but did not return a timely final receipt, so the Integration Lead completed review and
verification. The first workflow worker was closed without delivery; a separate `gpt-5.6-luna`
backup completed the core contract. No Sol worker was used. These scheduler events are execution
provenance, not product evidence.

首个 Benchmark adapter worker 返回了有效 storage-boundary blocker；Luna backup 落盘 adapter 但未及时返回最终回执，Integration
Lead 完成审查与验证。首个 Workflow worker 未交付后关闭，由另一个 `gpt-5.6-luna` backup 完成 core contract。未使用 Sol worker。
这些调度事件是 execution provenance，不是产品通过证据。

Fresh local evidence / 新鲜本地证据: storage `204 passed, 39 ignored`, API `191 passed`, workflow
replay focused `15 passed`, workspace Rust, `cargo fmt --all -- --check`, strict offline workspace
Clippy, locked Rust `1.85.0` check, local SDK `100 passed`, and `pnpm check:web` with public SDK
`15`, local SDK `100`, Web `207`, TypeScript/lint, and production build. Static inspection reports
`GRAPH_DIFF_IMPL_COUNT=1`; resolver focused tests passed `2` and protected decision API focused
tests passed `10`.

新鲜本地证据：storage `204 passed, 39 ignored`、API `191 passed`、workflow replay focused `15 passed`、workspace Rust、
`cargo fmt --all -- --check`、strict offline workspace Clippy、锁定 Rust `1.85.0` check、local SDK `100 passed`，以及
`pnpm check:web`（public SDK `15`、local SDK `100`、Web `207`、TypeScript/lint 与 production build）均通过。静态检查为
`GRAPH_DIFF_IMPL_COUNT=1`；resolver focused test `2` 项通过，protected decision API focused test `10` 项通过。

This advances Criteria 1, 2, 3, and 4 but does not close them or the long-term goal. No public
write, OpenAPI/public SDK write, Web mutation, provider, migration, secret, operator transport,
Docker/PostgreSQL runtime, authenticated browser, Git change-set, remote CI, release, or production
claim is made. PostgreSQL/Docker runtime, authenticated browser, and Git remain `unobserved`; remote
CI, operator rehearsal, release, and production remain `deferred`.

本 wave 推进条件 1、2、3 与 4，但不关闭这些条件或长期目标。没有新增 public write、OpenAPI/public SDK write、Web mutation、provider、
migration、secret、operator transport，也不宣称 Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、release
或 production 已通过。PostgreSQL/Docker runtime、authenticated browser 与 Git 继续为 `unobserved`；remote CI、operator rehearsal、release
与 production 继续为 `deferred`。

Next queue / 下一队列: add a new bilingual Necessity Record before implementing private
decision-list selection binding, so a listed sealed decision opens the exact decision-keyed
workspace without manual decision-ID entry. Keep the existing cohort route compatible, keep
GraphDiff as the sole graph-diff calculator, and keep public/release boundaries unchanged.

下一队列：在实现 private decision-list selection binding 前新增双语 Necessity Record，使已列出的 sealed decision 能无需手填 decision ID
打开 exact decision-keyed workspace。保持既有 cohort route 兼容，保持 GraphDiff 为唯一 graph-diff calculator，并保持 public/release boundary 不变。

## 2026-07-30 Private Benchmark Decision-List Selection Binding Review / 2026-07-30 私有 Benchmark Decision 列表选择绑定审阅

The shared increment is completed and verified locally within a local-only, private, read-only boundary.
The local SDK now owns the frozen listed-decision selection resource, exact scope/membership
checks, and selected decision-keyed loader. Web loads revised/baseline discovery lists, clears
selections on commit changes, and renders select options; the cohort-keyed workspace route remains
compatible. `pnpm check:web` passes with public SDK `15`, local SDK `104`, Web `208`, TypeScript/lint,
and production build. This advances Criterion 3 usability but closes no long-term criterion.

这是 local-only、private、read-only 边界内已完成且本地验证的增量。local SDK 现拥有 frozen listed-decision
selection resource、精确 scope/membership check 与 selected decision-keyed loader。Web 会加载 revised/baseline
discovery list，在 commit 变化时清空 selection，并渲染 select option；cohort-keyed workspace route 继续兼容。
`pnpm check:web` 通过，public SDK `15`、local SDK `104`、Web `208`、TypeScript/lint 与 production build 均完成。
本增量推进条件 3 可用性，但不关闭长期条件。

Current local evidence: `pnpm --filter @contextlab/local-sdk test` (`104 passed`), the workspace
selection focused test (`16 passed`), discovery data/presenter tests (`8 passed`), and `pnpm check:web`
with public SDK `15`, local SDK `104`, Web `208`, TypeScript/lint, and production build. Rust workspace
tests passed with storage `204 passed, 39 ignored` and API `191 passed`; format, strict offline Clippy,
locked Rust `1.85.0`, and `GRAPH_DIFF_IMPL_COUNT=1` also passed. PostgreSQL/Docker, authenticated browser,
visual, remote CI, Git change-set, operator rehearsal, release, and production remain `unobserved` or
`deferred`.

当前本地证据为：`pnpm --filter @contextlab/local-sdk test`（`104 passed`）、workspace selection focused test（`16 passed`）、
discovery data/presenter tests（`8 passed`），以及 `pnpm check:web`（public SDK `15`、local SDK `104`、Web `208`、
TypeScript/lint 与 production build）。Rust workspace tests 通过（storage `204 passed, 39 ignored`、API `191 passed`），
format、strict offline Clippy、锁定 Rust `1.85.0` 与 `GRAPH_DIFF_IMPL_COUNT=1` 也已通过。PostgreSQL/Docker、authenticated
browser、visual、remote CI、Git change-set、operator rehearsal、release 与 production 保持 `unobserved` 或 `deferred`。

No public REST/OpenAPI/public SDK write, Web mutation, provider, migration, secret, external
release, or production work is admitted by this increment. The next increment requires its own
bilingual Necessity Record; exact list-to-target provenance and the unchanged cohort compatibility
path remain required invariants.

本增量不准入 public REST/OpenAPI/public SDK write、Web mutation、provider、migration、secret、external
release 或 production work。下一增量必须有其独立的双语 Necessity Record；exact list-to-target provenance 与不变的
cohort compatibility path 继续是必需不变量。

## 2026-07-30 Private Context Graph Edge-Aware Uses Editor / 2026-07-30 私有 Context 图谱边感知 Uses 编辑器

This local-only increment is `completed / verified locally`. The existing guarded lifecycle contract
already supported typed Add/Remove Uses commits, but Web choices were component-pair based. The
presenter now reads the same validated `component:<id>` graph facts used by the lifecycle snapshot:
Add excludes self and existing directed Uses edges, Remove lists only existing edges, and malformed
or unknown graph facts fail closed. The editor reselects valid pairs after loading or reloading a
commit, while commit review, idempotency, default-off gating, and server validation remain unchanged.

本地增量现为 `completed / verified locally`。既有 guarded lifecycle contract 已支持类型化 Add/Remove Uses commit，但 Web
此前按 component pair 提供选项。presenter 现读取 lifecycle snapshot 中同一套经过验证的 `component:<id>` graph fact：Add
排除 self 与已存在的有向 Uses edge，Remove 只列出现有 edge，malformed 或 unknown graph fact fail closed。editor 在加载或
reload commit 后重新选择有效 pair；commit review、idempotency、默认关闭 gate 与服务端校验保持不变。

Fresh local evidence: focused presenter/editor `22 passed`; `pnpm check:web` public SDK `15`, local SDK `104`, Web `213`,
TypeScript/lint and production build; workspace Rust with storage `204 passed, 39 ignored` and API `191 passed`; format, strict
offline Clippy, locked Rust `1.85.0`, and `GRAPH_DIFF_IMPL_COUNT=1` all passed. This advances Criteria 1 and 4 without closing
either or the long-term goal. PostgreSQL/Docker runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and
production remain `unobserved` or `deferred`; no public write, migration, provider, or secret boundary changed.

新鲜本地证据：focused presenter/editor `22 passed`；`pnpm check:web` 通过 public SDK `15`、local SDK `104`、Web `213`、
TypeScript/lint 与 production build；workspace Rust 通过，storage `204 passed, 39 ignored`、API `191 passed`；format、strict
offline Clippy、锁定 Rust `1.85.0` 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。本增量推进条件 1 与 4，但不关闭它们或长期目标。
PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator rehearsal、release 与 production 继续为
`unobserved` 或 `deferred`；public write、migration、provider 与 secret boundary 均未改变。

Next queue / 下一队列：select the next dependency-ready increment with a new bilingual Necessity Record. The leading local
evidence gap is a canonical Criterion 3 benchmark closure receipt covering exact decision-list selection through decision-bound
workspace, run details, server-owned scorecard/regression/evaluation diff, and Web visibility; no new benchmark functionality is
admitted until that evidence is freshly reconciled.

下一队列：以新的双语 Necessity Record 选择下一项依赖就绪增量。当前本地证据缺口首选为 Criterion 3 benchmark closure receipt，
统一记录 exact decision-list selection、decision-bound workspace、run details、服务端 scorecard/regression/evaluation diff 与 Web
可见性；在该证据新鲜对账前，不准入新的 benchmark 功能。

## 2026-07-30 Private Benchmark Regression and Scorecard Closure Receipt / 2026-07-30 私有 Benchmark 回归与 Scorecard 收束回执

The Criterion 3 benchmark workflow is now reconciled as `completed / verified locally` for this
increment. The local path is decision-list discovery -> exact decision-bound workspace -> server-
owned run details, scorecard, regression, and evaluation-diff evidence -> Web presentation. The
existing cohort-keyed compatibility read remains intact, and no client-side benchmark policy is
recomputed. Fresh focused evidence is evaluation `45 passed`, storage evidence `23 passed`,
workspace projection `7 passed`, execution `11 passed`, API `191 passed`, local SDK `104 passed`,
and Web presenter/editor `22 passed`; the full local gate includes storage `204 passed, 39 ignored`,
`pnpm check:web` public SDK `15`, local SDK `104`, Web `213`, production build, format, strict
offline Clippy, locked Rust `1.85.0`, and `GRAPH_DIFF_IMPL_COUNT=1`.

本次 Criterion 3 benchmark workflow 已对账为 `completed / verified locally`。本地路径为 decision-list discovery ->
exact decision-bound workspace -> 服务端拥有的 run details、scorecard、regression 与 evaluation-diff evidence -> Web
presentation；既有 cohort-keyed compatibility read 保持不变，客户端不重新计算 benchmark policy。新鲜 focused evidence 为
evaluation `45 passed`、storage evidence `23 passed`、workspace projection `7 passed`、execution `11 passed`、API `191 passed`、
local SDK `104 passed` 与 Web presenter/editor `22 passed`；完整本地门禁还包括 storage `204 passed, 39 ignored`、
`pnpm check:web` 的 public SDK `15`、local SDK `104`、Web `213`、production build、format、strict offline Clippy、锁定 Rust
`1.85.0` 与 `GRAPH_DIFF_IMPL_COUNT=1`。

This receipt advances Criterion 3 but does not close it or the long-term goal: benchmark breadth,
authenticated browser runtime, Git binding, and all other completion conditions remain open. No
public REST/OpenAPI/public SDK write, Web mutation, provider, migration, secret, or external
deployment claim was added. PostgreSQL/Docker runtime, authenticated browser, visual, and Git remain
`unobserved`; remote CI, operator rehearsal, release, and production remain `deferred`.

本回执推进条件 3，但不关闭条件 3 或长期目标：benchmark breadth、authenticated browser runtime、Git binding 与其他完成条件
仍未收束。没有新增 public REST/OpenAPI/public SDK write、Web mutation、provider、migration、secret 或 external deployment 声明。
PostgreSQL/Docker runtime、authenticated browser、visual 与 Git 仍为 `unobserved`；remote CI、operator rehearsal、release 与
production 仍为 `deferred`。

Next queue / 下一队列：do not add more benchmark functionality without a demonstrated contract
defect. After the independent Workflow/Plugin, Knowledge/Memory, and CLI/Desktop review returns,
write a new bilingual Necessity Record for the highest-priority dependency-ready local criterion.

下一队列：除非新鲜验证暴露真实 contract defect，否则不再增加 benchmark 功能。等待 Workflow/Plugin、Knowledge/Memory 与
CLI/Desktop 独立 review 返回后，为最高优先级且依赖就绪的本地完成条件新增双语 Necessity Record。

## 2026-07-30 Private Plugin/MCP Capability Availability Read Admission / 2026-07-30 私有 Plugin/MCP 能力可用性读取准入

The next admitted local increment is the private Plugin/MCP capability-availability read recorded
in `docs/superpowers/plans/2026-07-30-private-plugin-capability-availability-read.md`. The existing
Rust MCP/plugin-runtime crates already own versioned manifest, compatibility, lifecycle, registry,
diagnostic, and fail-closed projection policy; the missing boundary is a Context-scoped local API,
non-public SDK, same-origin BFF, and shared capability-state Web read composition. The frozen
response exposes only stable plugin/capability IDs, versions, availability, compatibility, and safe
diagnostic codes. Missing registration is represented honestly as empty/unavailable data.

下一项准入的本地增量是 `docs/superpowers/plans/2026-07-30-private-plugin-capability-availability-read.md` 记录的 private
Plugin/MCP capability-availability read。既有 Rust MCP/plugin-runtime crate 已拥有 versioned manifest、compatibility、lifecycle、
registry、diagnostic 与 fail-closed projection policy；当前缺口是 Context-scoped local API、非公开 SDK、同源 BFF 与 shared
capability-state Web read composition。冻结 response 只暴露稳定 plugin/capability ID、version、availability、compatibility 与安全
diagnostic code；缺少 registration 时诚实地表示为 empty/unavailable data。

This increment is admitted only as local, private, read-only work. It adds no public REST/OpenAPI/
public SDK write, registry mutation, dynamic loading, provider call, migration, secret access,
operator transport, or second `GraphDiff` calculator. The long-term goal remains active and this
admission does not close Criterion 7 or any other completion condition.

本增量仅准入 local、private、read-only 工作。不增加 public REST/OpenAPI/public SDK write、registry mutation、dynamic loading、
provider call、migration、secret access、operator transport 或第二个 `GraphDiff` calculator。长期目标保持 active，本准入不关闭条件 7
或任何其他完成条件。

## 2026-07-30 Private Workflow Execution Status Inspector Mount / 2026-07-30 私有 Workflow 执行状态检查器挂载

### Status / 状态

`completed / verified locally` / `completed / 本地已验证`. This increment makes the already
validated private workflow execution-status projection reachable from the mounted Context
workspace. The binding inspector requires an exact redacted binding row and an explicitly entered
canonical `run_id`; it never derives a run from evaluation IDs, fixtures, preview data, or current
head. Context/commit remount keys and render-time scope guards reject stale state, 404 is mapped to
the honest `empty` state, and all status rendering remains in the shared design-system screen.

`completed / verified locally` / `completed / 本地已验证`。本增量让已验证的 private workflow execution-status projection 从已挂载的
Context workspace 可达。binding inspector 要求精确的脱敏 binding row 与用户显式输入的 canonical `run_id`；不会从 evaluation ID、
fixture、preview data 或 current head 推导 run。Context/commit remount key 与渲染时 scope guard 拒绝旧状态，404 诚实映射为 `empty`，
所有 status rendering 继续由 shared design-system screen 承载。

### Fresh local evidence / 新鲜本地证据

- Focused binding inspector: `15 passed`; Context workspace reachability: `2 passed`.
- Execution-status data and presenter: `7 passed`, including upstream-message and caller-message
  redaction.
- `pnpm --filter @contextlab/web test`: `310 passed`; `pnpm check:web`: public SDK `15`,
  local SDK `148`, Web `310`, TypeScript/lint, and production build passed.
- `cargo +1.85.0 fmt --all -- --check`, `cargo +1.85.0 test --workspace --quiet --offline -j 1`
  (API `223 passed`; storage `239 passed, 41 ignored`), strict offline Clippy, and locked Rust `1.85.0` check
  passed.
- Static inspection found exactly one production `impl GraphDiff`; public OpenAPI/SDK remains
  mutation-free except the existing GraphDiff POST. The legacy verifier remains a baseline
  `benchmark-workspace-route-method-count:2` failure because two protected GET route variants
  exist; this increment did not change that verifier or claim it green.
- Web `tsconfig.json` excludes only the transient `.next/dev` tree, retaining production
  `.next/types`, so partial development route-type writes do not invalidate TypeScript lint.

- focused binding inspector：`15 passed`；Context workspace reachability：`2 passed`。
- execution-status data 与 presenter：`7 passed`，包含 upstream message 与 caller message 脱敏。
- `pnpm --filter @contextlab/web test`：`310 passed`；`pnpm check:web`：public SDK `15`、local SDK `148`、Web `310`、TypeScript/lint
  与 production build 通过。
- `cargo +1.85.0 fmt --all -- --check`、`cargo +1.85.0 test --workspace --quiet --offline -j 1`（API `223 passed`、storage `239 passed, 41 ignored`）、strict offline
  Clippy 与锁定 Rust `1.85.0` check 通过。
- 静态检查发现唯一 production `impl GraphDiff`；public OpenAPI/SDK 除既有 GraphDiff POST 外仍无 mutation。旧 verifier 因存在两个受保护
  GET route variant 仍报告 baseline `benchmark-workspace-route-method-count:2`；本增量未修改 verifier，也不将其伪称为 green。
- Web `tsconfig.json` 仅排除临时 `.next/dev` tree，并保留 production `.next/types`，因此开发期间部分 route type 写入不会使 TypeScript lint 失效。

No producer, persistence repository, polling, run discovery, mutation, public REST/OpenAPI/SDK
operation, provider, migration, secret, Docker/PostgreSQL runtime, authenticated browser, Git
change-set, release, or production claim was added. Docker/PostgreSQL, browser, visual, and Git
remain `unobserved`; remote CI, operator rehearsal, release, and production remain `deferred`.
The long-term goal remains active.

本增量没有新增 producer、persistence repository、polling、run discovery、mutation、public REST/OpenAPI/SDK operation、provider、migration、secret、
Docker/PostgreSQL runtime、authenticated browser、Git change-set、release 或 production claim。Docker/PostgreSQL、browser、visual 与 Git 继续为
`unobserved`；remote CI、operator rehearsal、release 与 production 继续为 `deferred`。长期目标保持 active。

### Next queue / 下一队列

The next increment must begin with a new bilingual Necessity Record. The current highest-priority
local evidence gap is Criterion 8's documented CLI read-only smoke path; a documentation-only
receipt may be admitted if the existing executable contracts and exact exit codes are freshly
verified. Do not add public transport, workflow mutation, or release work.

下一增量必须先有新的双语 Necessity Record。当前本地最高优先级证据缺口是条件 8 的 CLI 只读 smoke path 文档；只有在既有可执行契约与
准确 exit code 获得新鲜验证后，才可准入 documentation-only receipt。不得新增 public transport、workflow mutation 或 release 工作。

## 2026-07-30 Private CLI Read-only Smoke Receipt / 2026-07-30 私有 CLI 只读 Smoke 回执

### Status / 状态

`completed / verified locally` / `completed / 本地已验证`. This documentation-only increment
supplies Criterion 8's documented CLI smoke evidence without adding a command or changing the
shared Rust core. The guide records replay projection exit `0`, valid unavailable capability
projection exit `2`, capability contract drift exit `64`, and the ordinary workspace adapter
unavailable exit `2`.

`completed / verified locally` / `completed / 本地已验证`。本 documentation-only 增量补齐条件 8 的 CLI smoke 文档证据，不增加 command
也不修改 shared Rust core。文档记录 replay projection 退出 `0`、合法 unavailable capability projection 退出 `2`、capability
contract drift 退出 `64`，以及普通 workspace adapter unavailable 退出 `2`。

### Fresh local evidence / 新鲜本地证据

- `cargo build --offline -p contextlab-cli` passed and all four documented binary probes were
  executed with the exit codes above.
- Adapter contract `8 passed`, CLI `5 passed`, and Desktop staging `5 passed`.
- Scoped strict Clippy passed; format and locked Rust `1.85.0` check passed.
- No CLI command, public transport, Web mutation, provider, secret, Tauri runtime, Docker/
  PostgreSQL, release, or production surface changed. Tauri runtime, Docker/PostgreSQL,
  authenticated browser, Git change-set, remote CI, operator rehearsal, release, and production
  remain `unobserved` or `deferred`.

- `cargo build --offline -p contextlab-cli` 通过，四个文档 binary probe 均已按上述退出码真实执行。
- adapter contract `8 passed`、CLI `5 passed`、Desktop staging `5 passed`。
- scoped strict Clippy、format 与锁定 Rust `1.85.0` check 通过。
- 没有改变 CLI command、public transport、Web mutation、provider、secret、Tauri runtime、Docker/PostgreSQL、release 或 production surface。
  Tauri runtime、Docker/PostgreSQL、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production 继续为
  `unobserved` 或 `deferred`。

## Next queue / 下一队列

The long-term goal remains active. Before admitting another implementation, perform a bounded
Criterion 1 audit for the smallest dependency-ready Context-first gap, with special attention to
component content update/replay coverage and whether its existing guarded lifecycle contract is
already complete. Any new work requires a separate bilingual Necessity Record; no public write,
operator transport, or release work is admitted.

长期目标保持 active。下一项 implementation 准入前，先对条件 1 的最小依赖就绪 Context-first gap 做有界审计，重点核对 component content
update/replay coverage，以及既有 guarded lifecycle contract 是否已完整。任何新工作都必须有独立双语 Necessity Record；不准入 public
write、operator transport 或 release work。

## 2026-07-30 Private Component Content Replay Witness Integrity / 2026-07-30 私有组件正文回放 Witness 一致性

### Status / 状态

`completed / verified locally` / `completed / 本地已验证`. The Criterion 1 audit found that
component create/update/remove and exact commit replay already existed, but the aggregate read
did not explicitly bind the immutable body revision to the replayed state's
`content_commit_id`. The private Rust lifecycle read now fails closed unless Context identity,
component identity, kind, resulting hash, and recording commit all agree.

`completed / verified locally` / `completed / 本地已验证`。条件 1 审计确认 component create/update/remove 与 exact commit replay
已经存在，但 aggregate read 未显式把 immutable body revision 绑定到 replay state 的 `content_commit_id`。现在 private Rust
lifecycle read 只有在 Context identity、component identity、kind、resulting hash 与 recording commit 全部一致时才接受；否则
fail closed。

### Fresh local evidence / 新鲜本地证据

- The red phase was observed: the new focused regression failed to compile with missing
  `validate_component_content_witness`; after implementation the regression passed (`1`).
- Focused storage evidence: lifecycle `10 passed`; component-content `10 passed, 5 ignored`;
  replay-state `7 passed`.
- `cargo test --workspace --quiet --no-fail-fast --offline` passed; storage reported
  `205 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline workspace Clippy, and
  `cargo +1.85.0 check --workspace --all-targets --locked --offline` passed.
- `pnpm check:web` passed with public SDK `15`, local SDK `111`, Web `236`, TypeScript/lint,
  and production build. Static `impl GraphDiff` count remained `1`.

- 已真实观察到红阶段：新增 focused regression 在 helper 缺失时以
  `validate_component_content_witness` missing 编译错误失败；实现后回归通过（`1`）。
- storage focused 证据：lifecycle `10 passed`；component-content `10 passed, 5 ignored`；
  replay-state `7 passed`。
- `cargo test --workspace --quiet --no-fail-fast --offline` 通过；storage 报告
  `205 passed, 39 ignored`。
- `cargo fmt --all -- --check`、strict offline workspace Clippy 与
  `cargo +1.85.0 check --workspace --all-targets --locked --offline` 通过。
- `pnpm check:web` 通过，public SDK `15`、local SDK `111`、Web `236`、TypeScript/lint 与
  production build 均通过；静态 `impl GraphDiff` 数量仍为 `1`。

### Boundary and next queue / 边界与下一队列

The implementation is limited to `crates/storage/src/context_lifecycle.rs` and its focused
tests, under the bilingual Necessity Record in
`docs/superpowers/plans/2026-07-30-private-component-content-replay-witness-integrity.md`.
It adds no REST/OpenAPI/public SDK method, mutation route, Web mutation, migration, provider,
secret, operator transport, or second graph-diff calculator. The legacy verifier's
`benchmark-workspace-route-method-count:2` remains a pre-existing baseline mismatch. Docker/
PostgreSQL runtime, authenticated browser, visual, Git change-set, remote CI, operator,
release, and production evidence remain `unobserved` or `deferred`; the long-term goal remains
active. The next increment must begin with a new bilingual Necessity Record and target the
highest-priority dependency-ready local completion gap rather than repeat this audit.

实现仅限 `crates/storage/src/context_lifecycle.rs` 及其 focused tests，并受
`docs/superpowers/plans/2026-07-30-private-component-content-replay-witness-integrity.md` 双语 Necessity Record 约束。
没有新增 REST/OpenAPI/public SDK method、mutation route、Web mutation、migration、provider、secret、operator transport 或第二个
graph-diff calculator。旧 verifier 的 `benchmark-workspace-route-method-count:2` 仍是既有 baseline mismatch。Docker/PostgreSQL
runtime、authenticated browser、visual、Git change-set、remote CI、operator、release 与 production evidence 继续为
`unobserved` 或 `deferred`；长期目标保持 active。下一项必须先新增双语 Necessity Record，选择最高优先级且依赖就绪的本地收束缺口，
不重复本次审计。

## 2026-07-30 Private Persisted Context Diff Review Read / 2026-07-30 私有持久化 Context Diff Review 读取

### Status / 状态

`completed / verified locally` / `completed / 本地已验证`. This local, private, read-only
increment advances Criteria 2 and 4. The protected route now reads one complete persisted
version-backed Context diff-review projection at exact project, Context, source-commit, and
target-commit scope. It delegates comparison to the existing Rust service and keeps
`GraphDiff::between` as the sole graph-diff calculator. The local SDK validates the frozen V1
response fail-closed, and the same-origin Web BFF consumes it through the shared
`data -> presenter -> screen` boundary.

`completed / verified locally` / `completed / 本地已验证`。本地 private、read-only 增量推进条件 2 与 4。protected route 现在以
project、Context、source commit 与 target commit 的 exact scope 读取完整的持久化 version-backed Context diff-review projection。
比较仍委托既有 Rust service，`GraphDiff::between` 仍是唯一 graph-diff calculator。local SDK 对冻结 V1 response 执行 fail-closed
校验，同源 Web BFF 通过共享 `data -> presenter -> screen` 边界消费该 projection。

### Fresh local evidence / 新鲜本地证据

- Focused API diff-review tests: `6 passed`.
- `cargo fmt --all -- --check`: `passed`.
- `cargo test --workspace --quiet --no-fail-fast`: `passed`; storage reported `205 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: `passed`.
- Locked Rust `1.85.0` workspace check: `passed`.
- `pnpm check:web`: `passed`; public SDK `15`, local SDK `117`, Web `245`, TypeScript/lint, and production build.
- Static boundary checks: `GRAPH_DIFF_IMPL_COUNT=1`, public OpenAPI diff-review hits `0`, private handler hits `1`.

- focused API diff-review 测试：`6 passed`。
- `cargo fmt --all -- --check`：`passed`。
- `cargo test --workspace --quiet --no-fail-fast`：`passed`；storage 为 `205 passed, 39 ignored`。
- strict offline Clippy：`passed`。
- 锁定 Rust `1.85.0` workspace check：`passed`。
- `pnpm check:web`：`passed`；public SDK `15`、local SDK `117`、Web `245`、TypeScript/lint 与 production build 均通过。
- 静态边界检查：`GRAPH_DIFF_IMPL_COUNT=1`、public OpenAPI diff-review 命中 `0`、private handler 命中 `1`。

### Boundary and next queue / 边界与下一队列

No public REST/OpenAPI/public SDK method, write route, Web mutation, provider, migration, secret
access, operator transport, or second `GraphDiff` calculator was added. Docker/PostgreSQL runtime,
authenticated browser, visual smoke, Git change-set, remote CI, operator rehearsal, release, and
production remain `unobserved` or `deferred`. This advances Criteria 2 and 4 but closes neither
criterion nor the long-term goal.

未新增 public REST/OpenAPI/public SDK method、写入 route、Web mutation、provider、migration、secret access、operator transport 或第二个
`GraphDiff` calculator。Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、
release 与 production 继续为 `unobserved` 或 `deferred`。本增量推进条件 2 与 4，但不关闭任一条件或长期目标。

The next implementation must begin with a new bilingual Necessity Record after the independent
bounded review completes. It must target a demonstrated dependency-ready local completion gap;
the likely candidates are a private read-only Context/graph relationship consumer or a narrowly
scoped benchmark/evaluation evidence gap, but no new route or feature is admitted by this receipt.

下一项实现必须在独立有界审查完成后先建立新的双语 Necessity Record，并且只能针对已有证据证明依赖就绪的本地收束缺口。候选方向可以是
private read-only Context/graph relationship consumer 或严格有界的 benchmark/evaluation evidence gap，但本回执不准入任何新 route 或新功能。

## 2026-07-30 Private Context Commit Ancestry and Server-Owned Merge Plan / 2026-07-30 私有 Context Commit Ancestry 与服务端 Merge Plan

### Status / 状态

`completed / verified locally` / `completed / 本地已验证`. This paired local Rust increment advances
Criteria 2 and 4. The storage port loads one exact-Context commit DAG, API composition delegates it
internally, and private merge review can resolve a three-way `MergePlan` from two typed tips without
trusting caller-supplied ancestry. `GraphDiff::between` remains the sole graph-diff calculator.

`completed / verified locally` / `completed / 本地已验证`。本地 Rust 成对增量推进条件 2 与 4。storage port 加载单一 exact-Context commit DAG，
API composition 在内部转发；private merge review 能从两个 typed tip 解析 three-way `MergePlan`，不信任 caller 提交的 ancestry。
`GraphDiff::between` 仍是唯一 graph-diff calculator。

### Fresh local evidence / 新鲜本地证据

- Ancestry focused storage tests: `2` integration tests and `13` unit tests passed; API repository contract tests: `4 passed`.
- Server-owned merge-review focused storage contract: `11 passed`; workspace Rust: `208 passed, 39 ignored` in storage.
- `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0`, and `pnpm check:web` passed; Web evidence was public SDK `15`, local SDK `117`, Web `245`, TypeScript/lint, and production build.
- Public OpenAPI exclusion contract: `1 passed`; `GRAPH_DIFF_IMPL_COUNT=1`.

- ancestry storage focused test：`2` 个 integration test 与 `13` 个 unit test 通过；API repository contract test：`4 passed`。
- server-owned merge-review storage focused contract：`11 passed`；workspace Rust storage：`208 passed, 39 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` 与 `pnpm check:web` 通过；Web 证据为 public SDK `15`、local SDK `117`、Web `245`、TypeScript/lint 与 production build。
- public OpenAPI exclusion contract：`1 passed`；`GRAPH_DIFF_IMPL_COUNT=1`。

### Boundary and next queue / 边界与下一队列

No REST/OpenAPI/SDK method, Web/CLI/Desktop surface, merge writer, branch mutation, rollback,
migration, provider, secret access, operator transport, Docker/runtime claim, browser claim, or
second GraphDiff calculator was added. PostgreSQL runtime, authenticated browser, Git, remote CI,
operator rehearsal, release, and production remain `unobserved` or `deferred`. The long-term goal
remains active. The next admitted work must begin with a new bilingual Necessity Record and may
only select a dependency-ready private consumer or benchmark/evaluation evidence gap.

没有新增 REST/OpenAPI/SDK method、Web/CLI/Desktop surface、merge writer、branch mutation、rollback、migration、provider、secret access、
operator transport、Docker/runtime 声明、browser 声明或第二个 GraphDiff calculator。PostgreSQL runtime、authenticated browser、Git、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active。下一项准入工作必须先有新的双语 Necessity
Record，并且只能选择依赖就绪的 private consumer 或 benchmark/evaluation evidence gap。

## 2026-07-30 Private Context Merge Review Inspector Mount / 2026-07-30 私有 Context Merge Review Inspector 挂载

`completed / verified locally` / `completed / 本地已验证`. The merge-review projection is now
mounted in the workspace as a read-only local inspector. It uses the independent default-off
`CONTEXTLAB_ENABLE_LOCAL_CONTEXT_MERGE_REVIEW` gate, exact left/right commit selectors, request-memory
Bearer transport, typed V1 resource normalization, and shared bilingual screen states. Candidate
or Context changes reset selections and stale resources. The mount is an adapter completion only;
the server-owned Rust projection and `GraphDiff::between` remain the sole sources of merge semantics
and graph diff calculation.

`completed / verified locally` / `completed / 本地已验证`。merge-review projection 现已作为只读 local inspector 挂载到 workspace。它使用独立的默认关闭
`CONTEXTLAB_ENABLE_LOCAL_CONTEXT_MERGE_REVIEW` gate、exact left/right commit selector、request-memory Bearer transport、typed V1 resource normalization
与共享双语 screen state。candidate 或 Context 变化会重置 selection 与旧 resource。本次只是 adapter 接线收束；server-owned Rust projection 与
`GraphDiff::between` 仍是 merge semantics 与 graph diff calculation 的唯一来源。

Fresh evidence / 新鲜证据：`pnpm check:web` passed with public SDK `15`, local SDK `126`, Web `260`, and production build; workspace Rust passed
with storage `208 passed, 39 ignored`; format, strict offline Clippy, locked Rust `1.85.0`, `impl GraphDiff=1`, and public merge-review surface `0` also
passed. The red phase was observed as `ERR_MODULE_NOT_FOUND` before the inspector existed.

新鲜证据：`pnpm check:web` 通过，public SDK `15`、local SDK `126`、Web `260` 与 production build 通过；workspace Rust 通过，storage 为
`208 passed, 39 ignored`；format、strict offline Clippy、锁定 Rust `1.85.0`、`impl GraphDiff=1` 与 public merge-review surface `0` 均通过。
inspector 尚不存在时的红阶段已真实观察为 `ERR_MODULE_NOT_FOUND`。

No public route, public SDK method, mutation, migration, provider, secret, Docker/PostgreSQL
runtime, browser, visual, Git, release, or production claim was added. PostgreSQL/Docker runtime,
authenticated browser, visual smoke, and Git remain `unobserved`; remote CI, operator rehearsal,
release, and production remain `deferred`. The long-term goal remains active.

未新增 public route、public SDK method、mutation、migration、provider、secret、Docker/PostgreSQL runtime、browser、visual、Git、release 或 production
声明。PostgreSQL/Docker runtime、authenticated browser、visual smoke 与 Git 继续为 `unobserved`；remote CI、operator rehearsal、release 与 production
继续为 `deferred`。长期目标保持 active。

## 2026-07-30 Private Context Metadata Lifecycle Closure / 2026-07-30 私有 Context Metadata 生命周期收束

`completed / verified locally` for this bounded private lifecycle metadata slice only; the active
long-term goal and every broader completion criterion remain open. The observed contract crosses the
existing versioned `UpdateMetadata` operation, guarded writer, replayed exact-commit state, protected
API, strict local SDK, and existing Web presenter/editor. Local receipts are storage `3 passed`, API
`1 passed`, lifecycle data/proxy focused `14 passed`, full local SDK `131 passed`, and full Web `265
passed` from the bounded metadata/lifecycle commands.

仅对本次有界 private lifecycle metadata slice 标记 `completed / verified locally`；active long-term goal 与更广泛的所有收束条件仍开放。已观察
contract 横跨既有 versioned `UpdateMetadata` operation、guarded writer、精确 commit 回放 state、protected API、strict local SDK 与既有 Web
presenter/editor。本地回执为 storage `3 passed`、API `1 passed`、lifecycle data/proxy focused `14 passed`、完整 local SDK `131 passed`、
完整 Web `265 passed`。

No code or tests were changed by this documentation integration. Full workspace/production checks,
PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git change-set, remote CI, operator
rehearsal, release, production, public promotion, secrets, and external systems are `unobserved` or
`deferred`. Any next implementation requires a new bilingual Necessity Record.

本次文档集成未修改 code 或 tests。full workspace/production checks、PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git change-set、
remote CI、operator rehearsal、release、production、public promotion、secret 与 external system 均为 `unobserved` 或 `deferred`。下一项实现必须
先有新的双语 Necessity Record。

## 2026-07-30 Fixture Contract Repair and Next Local Diff Gap (Historical Snapshot) / 2026-07-30 Fixture 契约修复与下一本地 Diff 缺口（历史快照）

At the 2026-07-30 snapshot, the only Web red phase was three stale lifecycle fixtures: one old component identity and
two success payloads without the frozen response schema/UUID snapshot contract. The focused command
now passes `14`; `pnpm check:web` passes public SDK `15`, local SDK `131`, Web `265`, TypeScript/lint,
and production build. Rust workspace tests pass with `40` ignored, format, strict offline Clippy,
locked Rust `1.85.0`, and `impl GraphDiff` count `1` pass as well.

2026-07-30 快照中的 Web 唯一红灯是三个过时 lifecycle fixture：一个旧 component identity，以及两个不符合 frozen response schema/UUID snapshot contract 的成功
payload。focused command 现通过 `14`；`pnpm check:web` 通过 public SDK `15`、local SDK `131`、Web `265`、TypeScript/lint 与 production build。
Rust workspace test 通过且观察到 `40` 个 ignored，format、strict offline Clippy、锁定 Rust `1.85.0` 与 `impl GraphDiff` count `1` 也通过。

This historical local closure did not close the long-term goal or Criteria 1/2. The next admitted increment at that time was
private version-bound metadata semantic diff because exact metadata replay existed while the then-current `ContextDiffSnapshotV1` did not carry Context metadata. It required a fresh bilingual
Necessity Record, red/green Rust evidence, and no public transport or second diff calculator.

该历史本地收束不关闭长期目标或条件 1/2。当时下一项准入增量是 private version-bound metadata semantic diff：exact metadata replay 已存在，但当时的
`ContextDiffSnapshotV1` 不携带 Context metadata。该项当时必须先有新的双语 Necessity Record、Rust red/green evidence，且不得新增 public transport
或第二个 diff calculator。

## 2026-07-30 Private Context Metadata Semantic Diff: Documentation/QA Check (Historical Snapshot) / 2026-07-30 私有 Context Metadata Semantic Diff：文档与 QA 核验（历史快照）

Historical 2026-07-30 snapshot / 2026-07-30 历史快照：`in_progress / blocked by source-contract mismatch` / `in_progress / 被源码契约不匹配阻塞`. The admitted local increment had a current
Necessity Record in `docs/superpowers/plans/2026-07-30-private-context-metadata-semantic-diff.md`, but its focused Rust contract is not green:
the test requires exported `ContextMetadataChangeV1::Modified { original, revised }`, while the implementation inspection still finds
`SemanticMetadataChangeV1` as a struct. This is a bounded implementation blocker, not a project-wide blocker and not evidence that the long-term
goal is complete.

2026-07-30 historical snapshot / 2026-07-30 历史快照：`in_progress / blocked by source-contract mismatch` / `in_progress / 被源码契约不匹配阻塞`。当时准入的本地增量已在
`docs/superpowers/plans/2026-07-30-private-context-metadata-semantic-diff.md` 中具备 Necessity Record，但 focused Rust contract 尚未变绿：测试
要求导出的 `ContextMetadataChangeV1::Modified { original, revised }`，而源码检查仍发现 `SemanticMetadataChangeV1` 结构体。这是有界实现阻塞，
不是全项目阻塞，也不代表长期目标完成。

The preceding private metadata lifecycle/Web fixture receipt remains valid: storage `3 passed`, API `1 passed`, lifecycle data/proxy focused
`14 passed`, local SDK `131 passed`, and Web `265 passed`. Those results do not substitute for metadata semantic-diff red/green proof. This
documentation-only pass does not claim a new workspace test, SDK, Web, PostgreSQL/Docker, browser, visual, Git change-set, remote CI, operator,
release, or production receipt. The working tree has broad uncommitted/untracked baseline content and two existing modified Web files, so Git
change-set provenance is `unobserved`.

此前 private metadata lifecycle/Web fixture 回执仍然有效：storage `3 passed`、API `1 passed`、lifecycle data/proxy focused `14 passed`、local SDK
`131 passed`、Web `265 passed`。这些结果不能替代 metadata semantic-diff 的 red/green proof。本次 docs-only pass 不声称取得新的 workspace test、
SDK、Web、PostgreSQL/Docker、browser、visual、Git change-set、remote CI、operator、release 或 production 回执。工作树存在大量未提交/未跟踪的基线
内容及两个已有修改的 Web 文件，因此 Git change-set provenance 为 `unobserved`。

No public REST/OpenAPI/SDK write surface, Web mutation, operator transport, migration, provider,
secret access, or second `GraphDiff` calculator is admitted. Remote CI, operator rehearsal,
release, and production remain deferred future deployment prerequisites; PostgreSQL/Docker runtime,
authenticated browser, and visual smoke remain unobserved. Keep the long-term goal active. The next
implementation step is the minimal enum-contract repair followed by fresh focused red/green evidence.

不准入 public REST/OpenAPI/SDK write surface、Web mutation、operator transport、migration、provider、secret access 或第二个 `GraphDiff`
calculator。remote CI、operator rehearsal、release 与 production 继续是延期的未来部署前置；PostgreSQL/Docker runtime、authenticated browser 与
visual smoke 继续为 `unobserved`。保持长期目标 active。下一实现步骤是最小 enum contract 修复，随后取得 focused red/green 新鲜证据。

## 2026-07-31 Private Context Metadata Semantic Diff Receipt / 2026-07-31 私有 Context Metadata Semantic Diff 回执

This is the current receipt for the bounded private semantic-diff slice. The historical source-contract
mismatch above is not the current state. Criteria 1, 2, 4, and 9 are advanced but none is closed; the
long-term goal remains active.

这是当前有界 private semantic-diff slice 的回执。上方 source-contract mismatch 是历史状态，不代表当前状态。条件 1、2、4、9
得到推进但均未关闭；长期目标保持 active。

| Necessity/evidence / 必要性与证据 | Current observation / 当前观察 |
| --- | --- |
| Context-first versioned diff and replay / Context-first 版本化 diff 与回放 | `ContextMetadataChangeV1` now carries typed `added`/`removed`/`modified` transitions; optional metadata is carried by `SemanticSnapshotV1`; `GraphDiff::between` remains the sole graph calculator. / `ContextMetadataChangeV1` 已承载 typed `added`/`removed`/`modified` transition；`SemanticSnapshotV1` 携带可选 metadata；`GraphDiff::between` 仍是唯一 graph calculator。 |
| Fresh local verification / 新鲜本地验证 | Diff focused `4 passed`; storage review `7 passed`; workspace passed with `40 ignored`; strict offline Clippy, Rust `1.85.0` check, fmt, and `impl GraphDiff=1` passed. / diff focused `4 passed`；storage review `7 passed`；workspace 通过且 `40 ignored`；Clippy、Rust `1.85.0`、fmt 与唯一 GraphDiff 通过。 |
| Web/SDK read boundary / Web/SDK 读取边界 | `pnpm check:web` passed: public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build. / `pnpm check:web` 通过：public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build。 |
| Remaining evidence / 剩余证据 | Rust enum variant unknown-field regression and storage exact-commit `added`/`removed` replay are the next local evidence gap. PostgreSQL/Docker, authenticated browser/visual, Git, remote CI, operator, release, and production remain `unobserved`/`deferred`. / Rust enum variant unknown-field 回归与 storage exact-commit `added`/`removed` replay 是下一本地证据缺口；其余 runtime、浏览器、Git 与发布证据继续 `unobserved`/`deferred`。 |
| Scope boundary / 范围边界 | No public write, OpenAPI/SDK write, Web mutation, migration, provider, operator transport, secret access, or second diff calculator was added. / 未新增 public write、OpenAPI/SDK write、Web mutation、migration、provider、operator transport、secret access 或第二个 diff calculator。 |

### Next necessity record / 下一项必要性记录

The next increment is limited to tests in the existing Rust diff/storage boundaries. It serves Criteria 1 and 2, closes a concrete
fail-closed/replay evidence gap, has no new transport or UI boundary, and must obtain fresh focused, workspace, Clippy, MSRV, fmt,
Web, and sole-GraphDiff verification before another increment is selected.

下一项仅限既有 Rust diff/storage 边界内的测试，服务条件 1、2，补齐具体的 fail-closed/replay 证据缺口，不新增 transport 或 UI 边界；在选择下一增量
前，必须取得 focused、workspace、Clippy、MSRV、fmt、Web 与唯一 GraphDiff 的新鲜验证。

## 2026-07-31 Metadata Transition Evidence Closure / 2026-07-31 Metadata Transition 证据收束

The bounded follow-up test increment is `completed / verified locally`; this does not close Criteria 1, 2, 4, or 9 and does not close
the active long-term goal. The real red phase showed that serde accepted unknown fields on metadata-change variants. The minimal repair
adds `deny_unknown_fields` to `ContextMetadataChangeV1`, while storage tests now replay both nullable-prior directions at exact commits.

本次有界后续测试增量标记为 `completed / verified locally`；不关闭条件 1、2、4、9，也不关闭 active long-term goal。真实红阶段显示 serde 会接受 metadata-change
variant 的未知字段；最小修复是在 `ContextMetadataChangeV1` 加入 `deny_unknown_fields`，同时 storage tests 现已在 exact commits 回放 nullable-prior 的两个方向。

| Fresh evidence / 新鲜证据 | Result / 结果 |
| --- | --- |
| Focused Rust / Rust focused | diff-engine metadata `5 passed`; storage Context diff review `8 passed`, including `added`, `removed`, and `modified`. / diff-engine metadata `5 passed`；storage Context diff review `8 passed`，覆盖 `added`、`removed` 与 `modified`。 |
| Workspace gates / workspace 门禁 | `cargo test --workspace --quiet --no-fail-fast` passed; storage `212 passed, 39 ignored`; strict offline Clippy, locked Rust `1.85.0` check, and fmt passed. / workspace 通过；storage `212 passed, 39 ignored`；Clippy、锁定 Rust `1.85.0` 与 fmt 通过。 |
| Web/SDK and static boundary / Web/SDK 与静态边界 | `pnpm check:web` passed with public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build; `impl GraphDiff=1`. / `pnpm check:web` 通过：public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build；唯一 GraphDiff。 |
| Evidence boundary / 证据边界 | No public write, Web mutation, migration, provider, operator transport, secret access, Docker/PostgreSQL runtime claim, external receipt, release, production, or second diff calculator. Those runtime/release facts remain `unobserved`/`deferred`. / 未新增 public write、Web mutation、migration、provider、operator transport、secret access、Docker/PostgreSQL runtime 声明、external receipt、release、production 或第二个 diff calculator；相关事实继续 `unobserved/deferred`。 |

### Next necessity record / 下一项必要性记录

The metadata transition evidence gate is closed only for this bounded local slice. The next increment must receive a new bilingual Necessity Record,
serve a named completion criterion, and be selected from the next dependency-ready local gap; the long-term goal remains active.

metadata transition evidence gate 仅对本地有界 slice 收束。下一项必须先取得新的双语 Necessity Record、服务一个命名的 completion criterion，并从下一项依赖就绪
本地缺口中选择；长期目标保持 active。

## 2026-08-01 Private Local Contract Verifier Route Catalog Repair / 2026-08-01 私有本地契约 Verifier 路由目录修复

`completed / verified locally` for this bounded Criterion 8 evidence increment only. The local
contract verifier previously stopped at the stale `benchmark-workspace-route-method-count:2`
baseline because the protected Benchmark workspace capability now has two legitimate GET variants:
the cohort-keyed workspace route and the decision-keyed workspace route. The verifier and its
isolated fixture now assert the exact two paths, exact GET methods, exact handlers, and exact
protected route catalog. Wrong methods, non-local paths, duplicate/missing handlers, duplicate/missing
catalog entries, and public-router leakage remain fail-closed.

本次仅对有界的条件 8 证据增量标记为 `completed / verified locally`。此前 local contract verifier 因受保护 Benchmark workspace
能力现有两个合法 GET variant，在过时的 `benchmark-workspace-route-method-count:2` baseline 停止：cohort-keyed workspace route 与
decision-keyed workspace route。verifier 与隔离 fixture 现已断言两个精确 path、GET method、handler 与 protected route catalog；错误 method、非 local path、
重复或缺失 handler、重复或缺失 catalog entry 以及 public-router leakage 继续 fail-closed。

Fresh evidence / 新鲜证据：focused verifier fixture passed; live verifier reported
`local_contract_source=passed`, `benchmark_workspace_route=passed`,
`benchmark_workspace_public_surface=passed`, `benchmark_definition_schema=passed`,
`benchmark_definition_public_boundary=passed`, `graph_diff_application=passed count=1`, and
`overall=unobserved` only because no diff input was supplied. `cargo fmt --all -- --check`, offline
workspace Rust tests (`storage 212 passed, 39 ignored`), strict offline Clippy, locked Rust `1.85.0`
check, and `pnpm check:web` (`public SDK 15`, `local SDK 134`, `Web 270`, production build) passed.

新鲜证据：focused verifier fixture 通过；live verifier 报告 `local_contract_source=passed`、`benchmark_workspace_route=passed`、
`benchmark_workspace_public_surface=passed`、`benchmark_definition_schema=passed`、`benchmark_definition_public_boundary=passed`、
`graph_diff_application=passed count=1`；由于未提供 diff input，`overall=unobserved`。`cargo fmt --all -- --check`、Rust workspace
（storage `212 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0` check 与 `pnpm check:web`（public SDK `15`、local SDK `134`、
Web `270`、production build）均通过。

No public route, OpenAPI/SDK method, Web mutation, Rust domain behavior, migration, provider,
secret, Docker/PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal,
release, or production claim was added. The long-term goal remains active; this receipt advances
Criterion 8 without closing it. Its historical next-wave note is superseded by the later
exact-commit relationship inspector receipt below; the current queue still requires a new bilingual
Necessity Record before implementation.

没有新增 public route、OpenAPI/SDK method、Web mutation、Rust domain behavior、migration、provider、secret、Docker/PostgreSQL runtime、authenticated browser、
Git、remote CI、operator rehearsal、release 或 production 声明。本回执推进条件 8 但不关闭条件 8；长期目标保持 active。该处的历史下一波说明已由下方
exact-commit relationship inspector 回执取代；当前队列仍要求下一项实现前先新增双语 Necessity Record。

## 2026-08-01 Private Benchmark Multi-Dataset Breadth Receipt / 2026-08-01 私有 Benchmark 多 Dataset 宽度回执

This bounded evidence increment is `completed / verified locally`; it advances Criterion 3 but
does not close Criterion 3 or the active long-term goal. The storage fixture retains two datasets
and four cases across baseline/revised Context commits, stable ordering, idempotent replay, exact
commit scope, persisted run coverage, scorecard/regression metadata, evaluation-diff metadata,
decision-bound workspace reads, and redacted projections. The API fixture retains two datasets and
four cases for one revised exact commit through protected authoring, workspace, and decision-workspace
routes. Its execution step calls the real `BenchmarkExecutionService` directly because adapter
wiring is `pub(crate)` in the test boundary; it is not an execution POST, API-level comparison, or
full REST execution transport receipt.

本次有界证据增量标记为 `completed / verified locally`；推进条件 3，但不关闭条件 3 或 active long-term goal。storage fixture 在
baseline/revised Context commit 上保留两个 dataset、四个 case、稳定排序、幂等 replay、精确 commit scope、持久化 run coverage、
scorecard/regression metadata、evaluation-diff metadata、decision-bound workspace read 与脱敏 projection。API fixture 在一个 revised 精确
commit 上通过受保护的 authoring、workspace 与 decision-workspace route 保留两个 dataset/四个 case。其 execution 阶段因测试边界中的
adapter wiring 为 `pub(crate)`，直接调用真实 `BenchmarkExecutionService`；这不是 execution POST、API-level comparison 或完整 REST execution transport 回执。

### Fresh verification / 新鲜验证

- Storage focused breadth: `1 passed`; API focused breadth: `1 passed`.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage `212 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build.
- Local verifier: source, graph, safe DTO, protected Benchmark route/catalog, and public-boundary checks
  passed; `overall=unobserved` because no unified diff input was supplied.
- `GRAPH_DIFF_IMPL_COUNT=1`.

- storage focused breadth：`1 passed`；API focused breadth：`1 passed`。
- `cargo fmt --all -- --check`：通过。
- `cargo test --workspace --quiet --no-fail-fast --offline`：通过；storage `212 passed, 39 ignored`。
- `cargo clippy --workspace --all-targets --offline -- -D warnings`：通过。
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`：通过。
- `pnpm check:web`：通过；public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build。
- local verifier：source、graph、safe DTO、受保护 Benchmark route/catalog 与 public-boundary checks 通过；因未提供 unified diff input，
  `overall=unobserved`。
- `GRAPH_DIFF_IMPL_COUNT=1`。

No public write, OpenAPI/SDK write, Web mutation, migration, provider, secret access, second
`GraphDiff` calculator, PostgreSQL/Docker runtime, authenticated browser, visual smoke, Git
change-set, remote CI, operator rehearsal, release, or production claim was added. The runtime and
release facts remain `unobserved` or `deferred`. The next implementation requires a new bilingual
Necessity Record and must target another dependency-ready named gap; keep the long-term goal active.

未新增 public write、OpenAPI/SDK write、Web mutation、migration、provider、secret access、第二个 `GraphDiff` calculator、PostgreSQL/Docker runtime、
authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。runtime 与 release 事实继续为
`unobserved` 或 `deferred`。下一次实现必须先有新的双语 Necessity Record，并服务另一个依赖就绪的命名缺口；保持长期目标 active。

## 2026-08-01 Private Benchmark Vertical Breadth Completion / 2026-08-01 私有 Benchmark 垂直宽度收束

This bounded Criterion 3 evidence increment is `completed / verified locally`; it advances but does
not close Criterion 3 or the active long-term goal. The API test now covers baseline/revised
in-memory execution, replay without new evaluator calls, protected workspace and decision-workspace
comparisons, two datasets/four cases, exact scope, scorecard coverage, `passed -> regressed`
evaluation diff, stable rows, and recursive redaction. Execution remains a direct service fixture,
not execution POST evidence. The Web inspector test carries the same two-dataset/four-case redacted
projection through the existing data -> presenter -> screen path and verifies ordering, coverage,
regression/diff rendering, bilingual accessible status, and raw-payload exclusion; it is not browser
or producer-to-browser runtime evidence.

本次有界条件 3 证据增量标记为 `completed / verified locally`；推进但不关闭条件 3 或 active long-term goal。API test 现覆盖 baseline/revised in-memory
execution、无新 evaluator call 的 replay、protected workspace 与 decision-workspace comparison、两个 dataset/四个 case、精确 scope、scorecard coverage、
`passed -> regressed` evaluation diff、稳定 row 与递归脱敏。execution 仍是 direct service fixture，不是 execution POST 证据。Web inspector test 将相同的
双 dataset/四 case 脱敏 projection 穿过既有 data -> presenter -> screen path，并验证排序、coverage、regression/diff 渲染、双语 accessible status 与 raw-payload 排除；
这不是 browser 或 producer-to-browser runtime 证据。

### Fresh verification / 新鲜验证

- API breadth focused: `1 passed`.
- Web inspector breadth focused: `6 passed`.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --quiet --no-fail-fast --offline`: passed; storage `212 passed, 39 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed; public SDK `15`, local SDK `134`, Web `270`, TypeScript/lint and production build.
- local verifier source/graph/safe DTO/protected route/catalog/public boundary checks passed; `overall=unobserved`
  because no unified diff input was supplied.
- `GRAPH_DIFF_IMPL_COUNT=1`.

- API breadth focused：`1 passed`。
- Web inspector breadth focused：`6 passed`。
- `cargo fmt --all -- --check`：通过。
- `cargo test --workspace --quiet --no-fail-fast --offline`：通过；storage `212 passed, 39 ignored`。
- `cargo clippy --workspace --all-targets --offline -- -D warnings`：通过。
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`：通过。
- `pnpm check:web`：通过；public SDK `15`、local SDK `134`、Web `270`、TypeScript/lint 与 production build。
- local verifier 的 source/graph/safe DTO/protected route/catalog/public boundary checks 通过；因未提供 unified diff input，`overall=unobserved`。
- `GRAPH_DIFF_IMPL_COUNT=1`。

No public write, OpenAPI/SDK write, Web mutation, migration, provider, secret access, second
`GraphDiff` calculator, PostgreSQL/Docker runtime, authenticated browser, visual smoke, Git
change-set, remote CI, operator rehearsal, release, or production claim was added. The next
increment requires a new bilingual Necessity Record; keep the long-term goal active.

未新增 public write、OpenAPI/SDK write、Web mutation、migration、provider、secret access、第二个 `GraphDiff` calculator、PostgreSQL/Docker runtime、
authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。下一增量必须先有新的双语 Necessity Record；保持长期目标 active。

## 2026-08-01 Private CLI Read-only Smoke Receipt / 2026-08-01 私有 CLI 只读 Smoke 回执

This bounded Criterion 8 evidence increment is `completed / verified locally`; it does not close
Criterion 8 or the active long-term goal. The offline-built provider-free CLI process observed all
four documented read-only paths: valid replay exit `0` with deterministic identity/count/ID output;
valid unavailable capability exit `2` with bilingual presentation; operation/integration drift exit
`64` with empty stdout and only the bilingual invalid-contract error; and adapter-backed workspace
exit `2` with the explicit unavailable-registration message. Focused adapter `8`, CLI `5`, and
Desktop staging `5` tests plus scoped strict Clippy passed.

本次有界条件 8 证据增量标记为 `completed / verified locally`；不关闭条件 8 或 active long-term goal。离线构建的 provider-free CLI 进程观测到四条文档化只读路径：
valid replay 退出 `0`，输出确定性的 identity/count/ID；valid unavailable capability 退出 `2`，输出双语 presentation；operation/integration drift 退出
`64`，stdout 为空且只有双语 invalid-contract error；adapter-backed workspace 退出 `2`，输出明确的 unavailable-registration message。adapter `8`、CLI `5`、
Desktop staging `5` focused test 与 scoped strict Clippy 均通过。

Fresh evidence / 新鲜证据：`cargo build --offline -p contextlab-cli` passed; `cargo test --offline -p contextlab-adapter-contract --test unavailable_contract`
`8 passed`; `cargo test --offline -p contextlab-cli --test command_path` `5 passed`; `cargo test --offline -p contextlab-desktop-tauri-staging --test staging_shell`
`5 passed`; `cargo fmt --all -- --check` and scoped strict Clippy passed; the current locked Rust workspace check, local verifier, and `GRAPH_DIFF_IMPL_COUNT=1` also passed.

新鲜证据：`cargo build --offline -p contextlab-cli` 通过；adapter contract `8 passed`、CLI `5 passed`、Desktop staging `5 passed`；`cargo fmt --all -- --check` 与 scoped strict
Clippy 通过；当前锁定 Rust workspace check、local verifier 与 `GRAPH_DIFF_IMPL_COUNT=1` 也通过。

No CLI write, Context mutation, provider/network/credential access, Desktop/Tauri runtime claim,
public REST/OpenAPI/SDK method, Web mutation, migration, PostgreSQL/Docker runtime, authenticated
browser, visual smoke, Git change-set, remote CI, operator rehearsal, release, production claim, or
second `GraphDiff` calculator was added. Keep the long-term goal active and require a new bilingual
Necessity Record before the next increment.

未新增 CLI write、Context mutation、provider/network/credential access、Desktop/Tauri runtime 声明、public REST/OpenAPI/SDK method、Web mutation、migration、
PostgreSQL/Docker runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release、production 声明或第二个 `GraphDiff` calculator。
保持长期目标 active，下一增量前必须新增双语 Necessity Record。

## 2026-08-01 Private Context Diff Pair Read Consistency / 2026-08-01 私有 Context Diff 成对读取一致性

This bounded local storage increment is `completed / verified locally`; it advances Criteria 2 and 4 but does not close either or the active long-term goal. `ContextDiffSnapshotV1PairRepository` now provides a typed pair boundary, `PersistedContextDiffReviewService` calls it once, validates both exact records, and delegates only to `VersionedContextDiffReviewService`. Memory uses one read lock and PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction. `GraphDiff::between` remains the sole calculator.

本次有界 local storage 增量标记为 `completed / verified locally`；推进条件 2 与 4，但不关闭任一条件或 active long-term goal。`ContextDiffSnapshotV1PairRepository` 现提供 typed pair boundary，`PersistedContextDiffReviewService` 只调用一次，校验两份 exact record，并且只委托 `VersionedContextDiffReviewService`。Memory 使用一个 read lock，PostgreSQL 使用一个 `REPEATABLE READ READ ONLY` transaction。`GraphDiff::between` 仍是唯一 calculator。

Fresh evidence: review `9 passed`; Memory repository `5 passed`; PostgreSQL contract `1 passed, 2 ignored`; storage library `212 passed, 39 ignored`; workspace tests, `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0`, `pnpm check:web` (`15/134/272 + production build`), verifier fixture/live scoped checks, and `GRAPH_DIFF_IMPL_COUNT=1` passed. The live verifier remains `overall=unobserved` only because no unified diff input was supplied. The assigned Luna Memory/PostgreSQL workers stopped producing output and were closed; the Integration Lead completed their disjoint file ownership without retaining conflicting edits. PostgreSQL runtime, Docker/virtualization, browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

新鲜证据：review `9 passed`；Memory repository `5 passed`；PostgreSQL contract `1 passed, 2 ignored`；storage library `212 passed, 39 ignored`；workspace tests、`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0`、`pnpm check:web`（`15/134/272 + production build`）、verifier fixture/live scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。live verifier 仅因未提供 unified diff input 而保持 `overall=unobserved`。被分派的 Luna Memory/PostgreSQL worker 停止产出后已关闭；Integration Lead 完成其不重叠文件 ownership，未保留冲突编辑。PostgreSQL runtime、Docker/virtualization、browser、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。

No public route, OpenAPI/SDK write, Web mutation, migration, provider, secret access, operator transport, release, or production claim was added. Keep the long-term goal active; the next implementation requires a new bilingual Necessity Record.

未新增 public route、OpenAPI/SDK write、Web mutation、migration、provider、secret access、operator transport、release 或 production 声明。保持长期目标 active；下一项实现必须先新增双语 Necessity Record。

## 2026-08-01 Private Exact-Commit Context Graph Relationship Inspector / 2026-08-01 私有精确提交 Context Graph 关系检查器

This bounded local read increment is `completed / verified locally`; it advances Criteria 1, 2, and 4
without closing them or the active long-term goal. The private local SDK now accepts the Rust-shaped
`graph_snapshot.project_id` field and fails closed for malformed scope or unknown snapshot fields.
The Web presenter owns the exact-commit relationship model and preserves all supported graph edge
kinds, deterministic ordering, exact commit identity, redacted node/edge facts, and empty-state
semantics. The editor only renders that presenter model through shared design-system primitives.

本次有界 local read 增量标记为 `completed / verified locally`；推进条件 1、2、4，但不关闭这些条件或 active long-term goal。私有 local SDK 现接受 Rust-shaped
`graph_snapshot.project_id`，并对错误 scope 或未知 snapshot field fail closed。Web presenter 负责 exact-commit relationship model，保留全部支持的 graph edge kind、
确定性排序、exact commit identity、脱敏 node/edge fact 与 empty-state 语义；editor 仅通过 shared design-system primitive 渲染该 presenter model。

Fresh verification / 新鲜验证：

- Focused Web presenter/editor `29 passed`; local SDK `135 passed` and TypeScript checks passed.
- `cargo fmt --all -- --check`, `cargo test --workspace --quiet` (storage `212 passed, 39 ignored`), strict offline Clippy, and locked Rust `1.85.0` check passed.
- `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `276`, TypeScript/lint, and production build.
- The local contract verifier passed its scoped source/graph/safe-DTO/protected-route checks; `overall=unobserved` only because no unified diff input was supplied. `GRAPH_DIFF_IMPL_COUNT=1` passed.

新鲜验证：

- Web presenter/editor 聚焦测试 `29 passed`；local SDK `135 passed` 且 TypeScript checks 通过。
- `cargo fmt --all -- --check`、`cargo test --workspace --quiet`（storage `212 passed, 39 ignored`）、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `276`，TypeScript/lint 与 production build 均通过。
- local contract verifier 的 scoped source/graph/safe-DTO/protected-route checks 通过；因未提供 unified diff input，`overall=unobserved`。`GRAPH_DIFF_IMPL_COUNT=1` 通过。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
operator transport, second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser,
visual smoke, Git change-set, remote CI, operator rehearsal, release, or production claim was added.
Those runtime and release facts remain `unobserved` or `deferred`. The long-term goal remains active;
the next implementation requires a new bilingual Necessity Record.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、operator transport、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、
authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。上述 runtime 与 release 事实继续为 `unobserved` 或 `deferred`。
长期目标保持 active；下一项实现必须先新增双语 Necessity Record。

## 2026-08-01 Private Context Lifecycle Atomic Read / 2026-08-01 私有 Context 生命周期原子读取

This bounded local increment is `completed / verified locally`; it advances Criteria 1 and 2
without closing either criterion or the active long-term goal. The reusable storage layer now
exposes `ContextLifecycleReadRepository`, whose exact-commit aggregate carries graph snapshot,
replay state, component inventory, immutable content witnesses, and the derived
`(ProjectId, ContextId, CommitId)` scope. Memory assembles the aggregate under one read guard;
PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction. The service rejects Context/commit
scope drift before returning the existing local read DTO.

本次有界 local 增量标记为 `completed / verified locally`；推进条件 1 与 2，但不关闭任一条件或 active
long-term goal。可复用 storage layer 现提供 `ContextLifecycleReadRepository`，exact-commit aggregate 携带
graph snapshot、replay state、component inventory、immutable content witness 与推导出的
`(ProjectId, ContextId, CommitId)` scope。Memory 在一个 read guard 下组装 aggregate；PostgreSQL 使用一个
`REPEATABLE READ READ ONLY` transaction。service 在返回既有 local read DTO 前拒绝 Context/commit scope drift。

Fresh verification / 新鲜验证：storage lifecycle `15 passed`、PostgreSQL SQL contract `1 passed`、API lifecycle
`5 passed`、workspace Rust storage `219 passed, 39 ignored`、fmt、strict offline Clippy、locked Rust `1.85.0`
check、`pnpm check:web` `15/135/284 + production build`、scoped local verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`
均通过。新 aggregate 没有可用的 `CONTEXTLAB_TEST_DATABASE_URL`，因此 PostgreSQL runtime 为 `unobserved`，不升级
为 runtime receipt；Docker、browser、Git、remote CI、operator rehearsal、release 与 production 继续为
`unobserved` 或 `deferred`。

Fresh verification / 新鲜验证：storage lifecycle `15 passed`、PostgreSQL SQL contract `1 passed`、API lifecycle
`5 passed`、workspace Rust storage `219 passed, 39 ignored`、fmt、strict offline Clippy、locked Rust `1.85.0`
check、`pnpm check:web` `15/135/284 + production build`、范围化 local verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`
均通过。本 aggregate 没有可用的 `CONTEXTLAB_TEST_DATABASE_URL`，因此 PostgreSQL runtime 为 `unobserved`，不升级
为 runtime receipt；Docker、browser、Git、remote CI、operator rehearsal、release 与 production 继续为
`unobserved` 或 `deferred`。

No public REST/OpenAPI/public SDK write, Web mutation, migration, provider, secret access, operator transport,
second GraphDiff calculator, or production claim was added. The next implementation requires a fresh bilingual
Necessity Record for a dependency-ready named criterion; deferred external release evidence remains outside the queue.

未新增 public REST/OpenAPI/public SDK write、Web mutation、migration、provider、secret access、operator transport、
第二个 GraphDiff calculator 或 production 声明。下一项实现必须先为依赖就绪的命名条件新增双语 Necessity Record；
延期 external release evidence 继续不进入当前队列。

## 2026-08-01 Private Workflow Execution Status Application Composition / 2026-08-01 私有 Workflow 执行状态应用组合

This bounded local increment is `completed / verified locally`; it advances Criteria 1 and 8
without closing either criterion or the active long-term goal. In PostgreSQL repository mode,
`AppState::try_from_env` now injects the existing `PostgresContextGraphRepository` through the
existing storage-backed `WorkflowExecutionStatusRepository` adapter. Memory mode retains the
typed unavailable reader. A review-found state-drift risk was repaired by replacing the
independent boolean with one `Unavailable`/`Custom`/`StorageBacked` backend discriminator.

本次有界 local 增量标记为 `completed / verified locally`；推进条件 1 与 8，但不关闭这两个条件或 active long-term goal。在 PostgreSQL repository mode 中，
`AppState::try_from_env` 现通过既有 storage-backed `WorkflowExecutionStatusRepository` adapter 注入既有
`PostgresContextGraphRepository`；memory mode 继续使用 typed unavailable reader。审查发现的 state-drift 风险已修复：以单一
`Unavailable`/`Custom`/`StorageBacked` backend discriminator 替代独立 boolean。

Fresh evidence / 新鲜证据：

- PostgreSQL composition focused test: `1 passed`.
- Memory unavailable focused regression: `1 passed`.
- Builder override regression: `1 passed`.

新鲜证据：

- PostgreSQL composition focused test：`1 passed`。
- Memory unavailable focused regression：`1 passed`。
- Builder override regression：`1 passed`。

Evidence boundary / 证据边界：

The focused tests prove application composition, builder-state consistency, and
memory/PostgreSQL mode isolation only. A protected read against the same temporary live
PostgreSQL fixture was not separately observed in this increment and remains `unobserved`; a lazy
PostgreSQL URL is not runtime database evidence. Remote CI, operator rehearsal, release,
production migration, authenticated browser/visual smoke, and Git change-set evidence remain
`unobserved` or `deferred`. No public REST/OpenAPI/public SDK write, Web mutation, operator
transport, secret access, scheduler, provider, or second `GraphDiff` calculator was added. The
long-term goal remains `active`; the next implementation requires a new bilingual Necessity Record.

证据边界：

focused test 只证明 application composition、builder-state consistency 与 memory/PostgreSQL mode isolation。本增量没有单独观测同一临时
live PostgreSQL fixture 上的 protected read，继续标记为 `unobserved`；lazy PostgreSQL URL 不是 runtime database evidence。remote CI、operator rehearsal、release、
production migration、authenticated browser/visual smoke 与 Git change-set evidence 继续为 `unobserved` 或 `deferred`。未新增 public REST/OpenAPI/public SDK write、Web mutation、operator transport、
secret access、scheduler、provider 或第二个 `GraphDiff` calculator。长期目标保持 `active`；下一项实现必须先新增双语 Necessity Record。

## 2026-08-01 Private Cross-Domain Capability Composition / 2026-08-01 私有跨域能力组合

`completed / verified locally` for this bounded local contract increment; Criteria 1, 5, 7, 8,
and 9 are advanced but remain open, and the long-term goal remains `active`. The MCP core now
offers immutable `CapabilityRegistrySnapshotV1` state with explicit schema, canonical availability,
and deterministic manifest fingerprint. One cross-domain fixture consumes the exact same snapshot
through the existing Workflow and Knowledge bridges, proving version/kind checks, deterministic
ordering, replay identity, and redacted citation projection without adding a second policy engine.

本次有界 local contract 增量标记为 `completed / verified locally`；条件 1、5、7、8 与 9 得到推进但仍开放，长期目标保持 `active`。MCP core 现提供不可变的
`CapabilityRegistrySnapshotV1`，包含显式 schema、canonical availability 与确定性 manifest fingerprint。一个 cross-domain fixture 使用同一 exact snapshot 通过既有 Workflow 与 Knowledge bridge，
证明 version/kind check、确定性排序、replay identity 与脱敏 citation projection，没有新增第二套 policy engine。

Fresh local evidence / 新鲜本地证据：

- Red mcp compile -> green snapshot contract `2 passed`; cross-domain composition `1 passed`.
- Knowledge/Memory red-to-green replay identity, Knowledge package tests passed; CLI/Desktop adapter `8`, CLI `5`, Desktop staging `5 passed`; Web raw-field red-to-green and `pnpm check:web` passed with `15/135/282 + production build`.
- Workflow/Plugin, Diff/versioning, and Evaluation reviews returned no defensible defect and no speculative changes.
- Final local gate passed: `cargo fmt --all -- --check`; workspace Rust with storage `212 passed, 39 ignored`; strict offline Clippy; locked Rust `1.85.0`; scoped verifier fixture; and `GRAPH_DIFF_IMPL_COUNT=1`. The latest `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `282`, TypeScript/lint, and production build. All claims retain their local-only boundary.

新鲜本地证据：

- mcp 红 compile 后 snapshot contract `2 passed`，cross-domain composition `1 passed`。
- Knowledge/Memory replay identity 红绿、Knowledge package tests 通过；CLI/Desktop adapter `8`、CLI `5`、Desktop staging `5 passed`；Web raw-field 红绿与 `pnpm check:web` 通过（`15/135/282 + production build`）。
- Workflow/Plugin、Diff/versioning 与 Evaluation review 未发现可 defensibly 修复的缺陷，也未做 speculative change。
- 本波次最终本地门禁已通过：`cargo fmt --all -- --check`；workspace Rust（storage `212 passed, 39 ignored`）；strict offline Clippy；锁定 Rust `1.85.0`；范围化 verifier fixture；以及 `GRAPH_DIFF_IMPL_COUNT=1`。最新 `pnpm check:web` 通过（public SDK `15`、local SDK `135`、Web `282`、TypeScript/lint 与 production build）。所有声明继续保持 local-only boundary。

The first four sidecar workers failed with transport `502 Bad Gateway`; they were closed and
reassigned to replacement `gpt-5.6-luna` workers. This is execution provenance, not a product
blocker. No public write, Web mutation, provider/network call, migration, secret access, second
GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git, remote
CI, operator rehearsal, release, or production claim was added. Deferred external release evidence
remains outside the local queue. Before the next implementation, create a new bilingual Necessity
Record and choose the next dependency-ready named criterion; do not close the long-term goal.

首轮四条 sidecar worker 因 transport `502 Bad Gateway` 失败，已关闭并由 replacement `gpt-5.6-luna` 接管。这是 execution provenance，不是 product blocker。未新增 public write、Web mutation、provider/network call、migration、secret access、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。延期 external release evidence 继续不进入本地队列。下一项实现前必须新增双语 Necessity Record，并选择下一项依赖就绪的命名条件；不得关闭长期目标。

## 2026-08-01 Private Graph Diff Error Redaction / 2026-08-01 私有 Graph Diff 错误脱敏

This bounded local security increment is `completed / verified locally`; it advances Criterion 4
without closing it or the active long-term goal. The Web local graph-diff adapter now keeps the
structured upstream error code and HTTP status but replaces structured and malformed messages with
one stable bilingual local review message. A red regression demonstrated that the old behavior
could expose `sql://internal-db?token=secret diagnostic payload`.

本次有界 local security 增量标记为 `completed / verified locally`；推进条件 4，但不关闭条件 4 或 active long-term goal。Web local graph-diff adapter 现保留
structured upstream error code 与 HTTP status，但将 structured 与 malformed message 替换为稳定的双语 local review message。红回归证明旧行为可能暴露
`sql://internal-db?token=secret diagnostic payload`。

Fresh verification / 新鲜验证：

- Red focused adapter run: `2 passed, 1 failed`; green focused run: `3 passed`.
- `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `277`, TypeScript/lint and production build.
- `cargo fmt --all -- --check`, workspace tests (storage `212 passed, 39 ignored`), strict offline Clippy, and locked Rust `1.85.0` check passed.
- Local verifier scoped checks and `GRAPH_DIFF_IMPL_COUNT=1` passed; verifier `overall=unobserved` because no unified diff input was supplied.

新鲜验证：

- 红 focused adapter run：`2 passed, 1 failed`；绿 focused run：`3 passed`。
- `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `277`，TypeScript/lint 与 production build 均通过。
- `cargo fmt --all -- --check`、workspace tests（storage `212 passed, 39 ignored`）、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- local verifier scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过；因未提供 unified diff input，verifier `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
operator transport, second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser,
visual smoke, Git, remote CI, operator rehearsal, release, or production claim was added. Those
facts remain `unobserved` or `deferred`. The long-term goal remains active; the next implementation
requires a new bilingual Necessity Record.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、operator transport、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、
authenticated browser、visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。上述事实继续为 `unobserved` 或 `deferred`。
长期目标保持 active；下一项实现必须先新增双语 Necessity Record。

## 2026-08-02 Private Benchmark Multi-Dataset PostgreSQL Breadth Receipt / 2026-08-02 私有 Benchmark 多 Dataset PostgreSQL 宽度回执

This bounded local evidence increment is `completed / verified locally`; it advances Criterion 3
without closing Criterion 3 or the active long-term goal. The earlier port `55439` run represents
only direct `BenchmarkDecisionEvidence` decision/run persistence. The corrected port `55441`
test-only PostgreSQL breadth fixture is the receipt that proves the sealed workspace projection,
dataset-case provenance, replay/readback, exact scope, and raw-payload redaction for two datasets
and four benchmark cases, including exact baseline/revised commit scopes, scorecard threshold and
coverage, and evaluation-diff evidence. It does not add or change a public transport, SDK method,
Web mutation, provider, or benchmark domain contract.

本次有界 local evidence 增量标记为 `completed / verified locally`；它推进条件 3，但不关闭条件 3 或
active long-term goal。早期 port `55439` run 只代表直接 `BenchmarkDecisionEvidence` decision/run persistence。
修正后的 port `55441` test-only PostgreSQL breadth fixture 才是完整 receipt：它证明 sealed workspace projection、
dataset-case provenance、replay/readback、exact scope 与 raw-payload redaction，并覆盖 two datasets、four
benchmark cases、exact baseline/revised commit scope、scorecard threshold/coverage 与 evaluation-diff evidence。
本次未新增或改变 public transport、SDK method、Web mutation、provider 或 benchmark domain contract。

Fresh verification / 新鲜验证：

- No configured PostgreSQL URL: focused breadth test `1 ignored` by design.
- Fresh loopback PostgreSQL 16 cluster on port `55439`: the early direct decision/run persistence
  test `1 passed`; this is not the full projection/provenance/replay/scope/redaction receipt.
- Separate fresh loopback PostgreSQL 16 cluster on port `55441`: the corrected test-only projection
  breadth test `1 passed`, covering the full projection/provenance/replay/scope/redaction boundary.
  Both temporary clusters were stopped afterward. The existing PostgreSQL service on `5432` was
  not inspected or used, and temporary-directory cleanup remains `unobserved`.
- `cargo test --workspace --quiet --no-fail-fast --offline`: `220 passed`, `40 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0` checks, and
  `pnpm check:web`: public SDK `15`, local SDK `135`, Web `284/284`, production build passed.
- `tests/contract/verify-local-contracts.test.ps1` passed; `scripts/verify-local-contracts.ps1`
  passed all local source/graph/DTO/route checks and correctly reported missing unified-diff, Git,
  browser, and production receipts as `unobserved`.

新鲜验证：

- 未配置 PostgreSQL URL：focused breadth test 按设计为 `1 ignored`。
- port `55439` 的 fresh loopback PostgreSQL 16 cluster：早期 direct decision/run persistence test `1 passed`；
  这不是完整 projection/provenance/replay/scope/redaction receipt。
- 独立的 port `55441` fresh loopback PostgreSQL 16 cluster：修正后的 test-only projection breadth test `1 passed`，
  覆盖完整 projection/provenance/replay/scope/redaction boundary。两个临时 cluster 随后均已停止。未检查或使用既有
  port `5432` 服务；临时目录 cleanup 继续为 `unobserved`。
- `cargo test --workspace --quiet --no-fail-fast --offline`：`220 passed`、`40 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check 与 `pnpm check:web` 通过，
  其中 public SDK `15`、local SDK `135`、Web `284/284`、production build 均通过。
- `tests/contract/verify-local-contracts.test.ps1` 通过；`scripts/verify-local-contracts.ps1` 的 local
  source/graph/DTO/route checks 全部通过，并正确将缺少 unified-diff、Git、browser 与 production receipt
  标为 `unobserved`。

No secret, `.env`, Docker, provider, migration, public REST/OpenAPI/SDK write, Web mutation, operator
transport, remote CI, release, or production access was used. Authenticated browser/visual smoke,
Git change-set, remote CI, operator rehearsal, release, production, and filesystem cleanup remain
`unobserved` or `deferred`. The next increment must start with a fresh bilingual Necessity Record;
the goal remains `active`.

未读取 secret 或 `.env`，未使用 Docker、provider、migration、public REST/OpenAPI/SDK write、Web mutation、
operator transport、remote CI，也未接触 release 或 production。authenticated browser/visual smoke、Git
change-set、remote CI、operator rehearsal、release、production 与 filesystem cleanup 继续为 `unobserved` 或
`deferred`。下一增量必须先创建双语 Necessity Record；目标保持 `active`。

## 2026-08-02 Private Context Lifecycle PostgreSQL Runtime Receipt / 2026-08-02 私有 Context 生命周期 PostgreSQL 运行时回执

This bounded local increment advances Criteria 1 and 2 without closing either criterion or the
active long-term goal. Existing Context lifecycle PostgreSQL persistence and exact-commit read now
have fresh runtime evidence on separate fresh loopback PostgreSQL 16 clusters. The initialization
test passed after a minimal test-only correction of a stale seed-count expectation (`0` to `6`);
the relationship lifecycle test passed unchanged.

本有界 local 增量推进条件 1 与 2，但不关闭任一条件或 active long-term goal。既有 Context lifecycle PostgreSQL
持久化与 exact-commit read 现获得两个独立 fresh loopback PostgreSQL 16 cluster 的新鲜 runtime evidence。初始化
test 在最小 test-only 修正陈旧 seed-count expectation（`0` 改为 `6`）后通过；relationship lifecycle test 无需修改即通过。

Fresh evidence / 新鲜证据：

- `postgres_lifecycle_initialization_creates_and_replays_an_unborn_branch_root`: red stale fixture,
  then fresh-cluster green `1 passed` after the seed-count correction.
- `postgres_lifecycle_replays_typed_uses_relationship_addition_and_removal`: separate fresh-cluster
  green `1 passed`, including exact commit read and relationship add/remove replay.
- The first red result was `(1, 0, 1, 1, 1, 6, 0)` versus the stale `(1, 0, 1, 1, 1, 0, 0)`; the
  correction changed only the test expectation and preserves all other persistence assertions.

新鲜证据：

- 初始化 named test 首次因陈旧 fixture 红灯，修正 seed-count 后在 fresh cluster 上 `1 passed`。
- relationship named test 在独立 fresh cluster 上 `1 passed`，覆盖 exact commit read 与 relationship add/remove replay。
- 首次红灯为实际 `(1, 0, 1, 1, 1, 6, 0)` 对陈旧 `(1, 0, 1, 1, 1, 0, 0)`；修正只改变 test expectation，其余 persistence assertion 保持不变。

Both temporary database processes were stopped. The pre-existing local PostgreSQL listener on port
5432 was not used, inspected for application data, or changed; temporary-directory cleanup remains
`unobserved` due local tool policy. No public REST/OpenAPI/public SDK write, Web mutation, provider,
new migration, second GraphDiff calculator, browser, Git, remote CI, operator rehearsal, release,
or production claim was added. The long-term goal remains `active` and the next increment requires
a fresh bilingual Necessity Record.

两个临时 database process 均已停止。既有本地 5432 PostgreSQL listener 未被使用、未读取其 application data 且未修改；
由于本地工具策略，临时目录 cleanup 继续为 `unobserved`。未新增 public REST/OpenAPI/public SDK write、Web mutation、provider、
new migration、第二个 GraphDiff calculator、browser、Git、remote CI、operator rehearsal、release 或 production 声明。长期目标保持
`active`，下一增量必须新建双语 Necessity Record。

## 2026-08-01 Private Benchmark Persistence Runtime Receipt / 2026-08-01 私有 Benchmark 持久化运行时回执

This bounded local increment advances Criterion 3 without closing it or the active long-term goal.
Existing Benchmark workspace projection persistence now has fresh local PostgreSQL runtime evidence:
two named provider-free tests passed on separate fresh loopback PostgreSQL 16 clusters. The first
proved migration, immutable projection create/replay, exact scope reads, persisted seal/case rows,
and wrong-scope rejection. The second proved execution materialization and durable replay/readback.

本有界 local 增量推进条件 3，但不关闭条件 3 或 active long-term goal。既有 Benchmark workspace projection
持久化现拥有新鲜本地 PostgreSQL runtime evidence：两个 provider-free named test 分别在两个 fresh loopback
PostgreSQL 16 cluster 上通过。第一项证明 migration、immutable projection create/replay、exact scope read、
持久 seal/case row 与 wrong-scope rejection；第二项证明 execution materialization 与 durable replay/readback。

Fresh evidence / 新鲜证据：

- `postgres_benchmark_workspace_projection_creates_replays_and_reads_exact_scope`: `1 passed`.
- `postgres_benchmark_execution_materializes_and_replays_workspace_projection`: `1 passed`.
- Benchmark projection focused contract: `7 passed`; `cargo fmt --all -- --check`: passed; `pnpm check:web`:
  public SDK `15`, local SDK `135`, Web `284/284`, production build passed.
- A combined run against one database failed at the test harness boundary with
  `relation "workspaces" already exists`; this is recorded as a fixture-isolation rule, not a product
  failure. Each ignored PostgreSQL test must receive its own empty disposable database.

新鲜证据：

- 两个 named PostgreSQL test 均为 `1 passed`，分别使用独立 fresh loopback database。
- Benchmark projection focused contract `7 passed`；format 通过；`pnpm check:web` 为 `15/135/284`，production
  build 通过。
- 单数据库组合运行因测试 harness 的 schema 残留得到 `relation "workspaces" already exists`；该问题记录为
  fixture isolation rule，而不是产品失败。每个 ignored PostgreSQL test 必须使用独立的 empty disposable database。

Both temporary database processes were stopped, and `pg_ctl status` confirmed that neither temporary
cluster is running. Recursive temporary-directory cleanup was rejected by local tool policy and
remains `unobserved`. A pre-existing local PostgreSQL listener on port 5432 was not used, inspected
for application data, or changed. This is local non-production evidence only. Public REST/OpenAPI/public SDK writes, Web mutation, provider
execution, browser/Git, remote CI, operator rehearsal, release, and production evidence remain
`unobserved` or `deferred`. The next increment requires a fresh bilingual Necessity Record and the
long-term goal remains `active`.

两个临时 database process 均已停止，且 `pg_ctl status` 确认两个临时 cluster 均未运行。工具策略拒绝递归清理
临时目录，因此 cleanup 为 `unobserved`。既有本地 5432 PostgreSQL listener 未被本回合使用、未读取其应用数据且
未修改。本证据仅限本地非生产范围。public REST/OpenAPI/public SDK write、Web mutation、provider
execution、browser/Git、remote CI、operator rehearsal、release 与 production evidence 继续为 `unobserved` 或
`deferred`。下一增量必须新建双语 Necessity Record，长期目标保持 `active`。

## 2026-08-01 Private Knowledge/Memory PostgreSQL Runtime Receipt / 2026-08-01 私有 Knowledge/Memory PostgreSQL 运行时回执

This bounded local increment is `completed / verified locally`; it advances Criteria 1 and 8
without closing either criterion or the active long-term goal. The existing private Knowledge/Memory
projection storage contract now has a fresh disposable PostgreSQL runtime receipt. The test is
opt-in, ignored without an explicit URL, loopback-only, and provider-free; it does not add a route,
SDK method, Web mutation, migration, or second graph-diff calculator.

本次有界 local 增量标记为 `completed / verified locally`；推进条件 1 与 8，但不关闭这两个条件或 active long-term goal。既有 private Knowledge/Memory
projection storage contract 现拥有新鲜 disposable PostgreSQL runtime receipt。test 是 opt-in、未显式 URL 时 ignored、仅 loopback 且 provider-free；不新增 route、SDK method、Web mutation、migration 或第二个 graph-diff calculator。

Fresh evidence / 新鲜证据：

- Fresh local loopback PostgreSQL 16 cluster: `postgres_projection_runtime_receipt_covers_exact_immutable_redacted_replay` -> `1 passed`.
- The receipt covers migration/seed, `Created`, identical `Replayed`, second-connection read, stored-JSON redaction, exact project/context/commit scope enforcement, immutable conflict, and malformed stored projection rejection.
- No URL run remains `1 ignored`; full local Rust/storage (`218 passed, 39 ignored`), format, strict offline Clippy, locked Rust 1.85, `pnpm check:web` Web `284/284` plus production build, and local contract verifier all passed.

新鲜证据：

- fresh local loopback PostgreSQL 16 cluster：`postgres_projection_runtime_receipt_covers_exact_immutable_redacted_replay` -> `1 passed`。
- 回执覆盖 migration/seed、`Created`、相同 `Replayed`、第二连接读取、stored-JSON redaction、exact project/context/commit scope enforcement、immutable conflict 与 malformed stored projection rejection。
- 无 URL 运行继续为 `1 ignored`；完整 local Rust/storage（`218 passed, 39 ignored`）、format、strict offline Clippy、锁定 Rust 1.85、`pnpm check:web` Web `284/284` 与 production build，以及 local contract verifier 均通过。

Evidence boundary / 证据边界：

The temporary database process was stopped. Recursive removal of the verified temporary data
directory was rejected by the local tool policy, so filesystem cleanup remains `unobserved`; no
database process remains running. This is local non-production evidence only. Remote CI, operator
rehearsal, release, production, authenticated browser/visual smoke, and Git change-set evidence
remain `unobserved` or `deferred`. The long-term goal remains `active`; the next implementation
requires a new bilingual Necessity Record.

证据边界：

临时 database process 已停止。工具策略拒绝删除已核验的临时 data directory，因此 filesystem cleanup 继续为 `unobserved`；没有数据库进程继续运行。本证据仅限本地非生产环境。remote CI、operator rehearsal、release、production、authenticated browser/visual smoke 与 Git change-set evidence 继续为 `unobserved` 或 `deferred`。长期目标保持 `active`；下一项实现必须先新增双语 Necessity Record。

## 2026-08-01 Private Workflow Execution Status PostgreSQL Persistence / 2026-08-01 私有 Workflow 执行状态 PostgreSQL 持久化

This bounded local increment is `completed / verified locally`; it advances Criteria 1 and 8
without closing either criterion or the active long-term goal. `contextlab-storage` now has a
private `PostgresContextGraphRepository` implementation for the existing immutable
`WorkflowExecutionStatusRepository`, plus migration `0024_workflow_execution_status.sql`.
Only the validated redacted `WorkflowExecutionStatusProjectionV1` is stored. The schema enforces
exact Context/run scope, v1 shape, foreign keys, uniqueness, and append-only behavior. A root
cause found by a fresh database run removed a duplicate `uq_contexts_project_id_id` declaration
from migration `0022`; a static one-owner regression prevents recurrence.

本次有界 local 增量标记为 `completed / verified locally`；推进条件 1 与 8，但不关闭这两个条件或 active long-term goal。
`contextlab-storage` 现为既有不可变 `WorkflowExecutionStatusRepository` 提供私有
`PostgresContextGraphRepository` 实现，并增加 `0024_workflow_execution_status.sql`。只保存已校验的脱敏
`WorkflowExecutionStatusProjectionV1`。schema 强制 exact Context/run scope、v1 shape、foreign key、唯一性与
append-only。fresh database run 发现并修复 `0022` 重复创建 `uq_contexts_project_id_id` 的根因，并以 one-owner 静态回归防止复发。

Fresh evidence / 新鲜证据：

- Adapter unit contract: `3 passed`; migration contract: `3 passed`; default no-URL live test: `1 ignored`.
- Temporary loopback PostgreSQL 16.14 fresh cluster: the named integration test `1 passed`, including schema application, seed, create/replay, cross-connection read, and conflict.
- The first runtime attempt was red on the duplicate migration constraint; after the minimal correction, the fresh-cluster rerun passed without an open-transaction warning.
- Final local quality gate: workspace Rust storage `218 passed, 39 ignored`; `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0` check; `pnpm check:web` public SDK `15`, local SDK `135`, Web `284`, and production build; local verifier `graph_diff_application=passed count=1`, `safe_local_dto_fields=passed`, `overall=unobserved` only for absent unified diff input.

新鲜证据：

- adapter unit contract `3 passed`；migration contract `3 passed`；默认无 URL 的 live test `1 ignored`。
- 临时 loopback PostgreSQL 16.14 fresh cluster：指定 integration test `1 passed`，覆盖 schema application、seed、create/replay、跨连接读取与 conflict。
- 首次 runtime attempt 因重复 migration constraint 红灯；最小修复后 fresh-cluster rerun 通过，且不再出现 open-transaction warning。
- 最终 local quality gate：workspace Rust storage `218 passed, 39 ignored`；`cargo fmt --all -- --check`；strict offline Clippy；锁定 Rust `1.85.0` check；`pnpm check:web` public SDK `15`、local SDK `135`、Web `284` 与 production build；local verifier `graph_diff_application=passed count=1`、`safe_local_dto_fields=passed`，仅因缺少 unified diff input 保持 `overall=unobserved`。

No public REST/OpenAPI/public SDK write, Web mutation, execution-start route, provider, scheduler,
operator transport, second GraphDiff calculator, secret access, release, or production claim was
added. Remote CI, operator rehearsal, release/production migration, authenticated browser/visual
smoke, and Git change-set evidence remain `unobserved` or `deferred`. The long-term goal remains
active; the next increment requires a fresh bilingual Necessity Record.

未新增 public REST/OpenAPI/public SDK write、Web mutation、execution-start route、provider、scheduler、operator
transport、第二个 GraphDiff calculator、secret access、release 或 production 声明。remote CI、operator rehearsal、
release/production migration、authenticated browser/visual smoke 与 Git change-set evidence 继续为 `unobserved` 或
`deferred`。长期目标保持 active；下一增量必须先新增双语 Necessity Record。

## 2026-08-01 Private Workflow Execution Producer and Repository / 2026-08-01 私有 Workflow 执行 Producer 与 Repository

This bounded local increment is `completed / verified locally`; it advances Criterion 1 without
closing it or the active long-term goal. The reusable storage crate now owns an immutable
`WorkflowExecutionStatusRepository` contract and deterministic in-memory implementation keyed by
exact `(ContextId, WorkflowRunId)`. `WorkflowExecutionStatusService` validates root and replay
logs through the existing `WorkflowExecutionStatusProjectionV1` before persistence. Identical
immutable writes replay, conflicting reuse fails closed, and the API adapter exposes only the
existing redacted status resource. The application default remains the typed unavailable adapter
until a repository is explicitly injected.

本次有界 local 增量标记为 `completed / verified locally`；推进条件 1，但不关闭条件 1 或 active long-term goal。可复用 storage crate 现拥有不可变
`WorkflowExecutionStatusRepository` contract 与确定性 in-memory implementation，按 exact `(ContextId, WorkflowRunId)` 建立 key。`WorkflowExecutionStatusService` 在持久化前通过既有
`WorkflowExecutionStatusProjectionV1` 校验 root/replay log。相同不可变写入可 replay，冲突复用 fail closed，API adapter 只暴露既有脱敏 status resource。应用默认在显式注入 repository 前继续使用 typed unavailable adapter。

Fresh red/green and local evidence / 新鲜红绿与本地证据：

- Initial storage compile was red because the admitted repository contract and types did not exist; storage producer/repository green `3 passed`.
- API storage adapter focused green `6 passed`, including exact read, redacted serialization, and default-unavailable preservation.
- `cargo test --workspace --quiet --no-fail-fast --offline` passed; storage `215 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline workspace Clippy, and locked Rust `1.85.0` workspace check passed.
- `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `284`, TypeScript/lint, and production build.
- Scoped local contract verifier source/graph/safe-DTO/protected-route checks passed; `overall=unobserved` without unified diff input. `GRAPH_DIFF_IMPL_COUNT=1` passed.

新鲜红绿与本地证据：

- 初始 storage compile 因准入的 repository contract 与 types 尚不存在而红灯；storage producer/repository green `3 passed`。
- API storage adapter focused green `6 passed`，覆盖 exact read、脱敏序列化与 default-unavailable 保留。
- `cargo test --workspace --quiet --no-fail-fast --offline` 通过；storage `215 passed, 39 ignored`。
- `cargo fmt --all -- --check`、strict offline workspace Clippy 与锁定 Rust `1.85.0` workspace check 通过。
- `pnpm check:web` 通过（public SDK `15`、local SDK `135`、Web `284`、TypeScript/lint 与 production build）。
- scoped local contract verifier 的 source/graph/safe-DTO/protected-route checks 通过；未提供 unified diff input，`overall=unobserved`；`GRAPH_DIFF_IMPL_COUNT=1` 通过。

This receipt is private, provider-free, and in-memory. It adds no execution start route, public
REST/OpenAPI/public SDK write, Web mutation, scheduler, migration, provider, secret access,
operator transport, or production claim. PostgreSQL/Docker runtime, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`. The long-term goal remains active; the next increment requires a fresh bilingual
Necessity Record for the next dependency-ready named criterion.

本回执是 private、provider-free 且 in-memory 的。未新增 execution start route、public REST/OpenAPI/public SDK write、Web mutation、scheduler、migration、provider、secret access、operator transport 或 production 声明。PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active；下一增量必须先为下一项依赖就绪的命名条件新增双语 Necessity Record。

## 2026-08-01 Private Workflow Read and Lifecycle Metadata Closure / 2026-08-01 私有 Workflow Read 与生命周期 Metadata 收束

This turn closes two bounded local evidence gaps without closing the active long-term goal. The
guarded lifecycle writer now carries exact resulting or inherited `ContextMetadata` into each
immutable `SemanticSnapshotV1`; a later non-metadata successor preserves the parent metadata, and
idempotent replay returns the original snapshot. The existing private Workflow binding read is
also fully connected from the same-origin BFF through `data -> presenter -> screen` and mounted at
the selected exact Context commit in the workspace.

本轮收束两个有界 local evidence gap，但不关闭 active long-term goal。guarded lifecycle writer 现将 exact resulting 或 inherited `ContextMetadata` 写入每个不可变 `SemanticSnapshotV1`；后续 non-metadata successor 继承 parent metadata，幂等 replay 返回原始 snapshot。既有 private Workflow binding read 也已从同源 BFF 经 `data -> presenter -> screen` 完整接通，并挂载到 workspace 选定的 exact Context commit。

Fresh verification / 新鲜验证：

- Storage metadata focused test: `1 passed`; API writer-to-review metadata test: `1 passed`.
- Workspace Rust: passed; storage reports `212 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, and `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: passed with public SDK `15`, local SDK `135`, Web `280`, and a successful production Web build.
- BFF focused route: `10 passed`; Workflow binding/presenter/inspector focused run: `17 passed`.
- `pwsh -NoProfile -File .\\tests\\contract\\verify-local-contracts.test.ps1`: fixture tests passed with a deterministic safe diff and `overall=passed`.
- Live local contract verifier: source/graph/safe-DTO/protected-route checks passed and `graph_diff_application=passed count=1`; live `overall=unobserved` only because no live unified diff input was provided.

No public REST/OpenAPI/public SDK write, Web mutation, migration, provider, secret access,
operator transport, second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser,
visual smoke, Git change-set, remote CI, operator rehearsal, release, or production claim is made.
Those remain `unobserved` or `deferred`. The next implementation requires a fresh bilingual
Necessity Record for the next dependency-ready named criterion; the long-term goal stays active.

未新增 public REST/OpenAPI/public SDK write、Web mutation、migration、provider、secret access、operator transport、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。上述证据继续为 `unobserved` 或 `deferred`。下一项实现必须先为下一项依赖就绪的命名条件新增双语 Necessity Record；长期目标保持 active。

## 2026-08-01 Private Persisted Context Diff Error Redaction / 2026-08-01 私有持久化 Context Diff 错误脱敏

This bounded local security increment is `completed / verified locally`; it advances Criterion 4
without closing it or the active long-term goal. The private persisted Context diff Web adapter now
preserves HTTP status and structured error code while replacing structured and malformed upstream
messages with the stable bilingual unavailable message. A red regression proved the old adapter
could expose `sql://internal-db?token=secret diagnostic payload`.

本次有界 local security 增量标记为 `completed / verified locally`；推进条件 4，但不关闭条件 4 或 active long-term goal。私有 persisted Context diff Web adapter 现保留
HTTP status 与 structured error code，同时将 structured 与 malformed upstream message 替换为稳定的双语 unavailable message。红回归证明旧 adapter 可能暴露
`sql://internal-db?token=secret diagnostic payload`。

Fresh verification / 新鲜验证：

- Red focused adapter run: `4 passed, 2 failed`; green focused run: `6 passed`.
- `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `278`, TypeScript/lint and production build.
- `cargo fmt --all -- --check`, workspace tests (storage `212 passed, 39 ignored`), strict offline Clippy, and locked Rust `1.85.0` check passed.
- Local verifier scoped checks and `GRAPH_DIFF_IMPL_COUNT=1` passed; verifier `overall=unobserved` because no unified diff input was supplied.

新鲜验证：

- 红 focused adapter run：`4 passed, 2 failed`；绿 focused run：`6 passed`。
- `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `278`，TypeScript/lint 与 production build 均通过。
- `cargo fmt --all -- --check`、workspace tests（storage `212 passed, 39 ignored`）、strict offline Clippy 与锁定 Rust `1.85.0` check 通过。
- local verifier scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过；因未提供 unified diff input，verifier `overall=unobserved`。

No public route, OpenAPI/public SDK method, Web mutation, migration, provider, secret access,
operator transport, second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser,
visual smoke, Git, remote CI, operator rehearsal, release, or production claim was added. Those
facts remain `unobserved` or `deferred`. The long-term goal remains active; the next implementation
requires a new bilingual Necessity Record.

没有新增 public route、OpenAPI/public SDK method、Web mutation、migration、provider、secret access、operator transport、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、
authenticated browser、visual smoke、Git、remote CI、operator rehearsal、release 或 production 声明。上述事实继续为 `unobserved` 或 `deferred`。
长期目标保持 active；下一项实现必须先新增双语 Necessity Record。
## 2026-08-02 Private Context Lifecycle Read Inspector / 2026-08-02 私有 Context 生命周期只读检查器

This bounded local Web increment advances Criterion 1 without closing Criterion 1 or the active
long-term goal. The existing protected exact-commit lifecycle read is now composed as a dedicated
private read-only inspector through `data -> presenter -> screen`. It renders server-owned Context
metadata, component content/provenance, and graph relationships for the selected `(Context, commit)`;
request credentials remain memory-only, and stale responses are discarded when the selected scope changes.

本次有界本地 Web 增量推进条件 1，但不关闭条件 1 或 active 长期目标。既有受保护 exact-commit lifecycle
read 现通过 `data -> presenter -> screen` 组合为独立私有只读检查器。它针对选定 `(Context, commit)` 呈现
server-owned Context metadata、组件正文/来源与图谱关系；请求凭据仅驻留内存，selected scope 改变时会丢弃过期响应。

Fresh verification / 新鲜验证：

- Focused lifecycle presenter/inspector tests: `23 passed`.
- `pnpm --filter @contextlab/web lint`: passed; full Web suite: `290 passed`.
- `pnpm check:web`: public SDK `15` passed, local SDK `135` passed, Web `290` passed, production build passed.
- Rust `cargo fmt --all -- --check`, workspace tests (`220 passed`, `41 ignored`), strict offline Clippy, and locked
  Rust `1.85.0` check passed. The only root-cause repair was test-only helper argument reduction in the PostgreSQL breadth fixture.

新鲜验证：

- lifecycle presenter/inspector 聚焦测试：`23 passed`。
- `pnpm --filter @contextlab/web lint` 通过；完整 Web suite：`290 passed`。
- `pnpm check:web` 通过：public SDK `15`、local SDK `135`、Web `290` 与 production build 均通过。
- Rust `cargo fmt --all -- --check`、workspace tests（`220 passed`、`41 ignored`）、strict offline Clippy 与锁定 Rust
  `1.85.0` check 通过。唯一根因修复是 PostgreSQL breadth fixture 的 test-only helper 参数收敛。

No public REST/OpenAPI/public SDK write, Web mutation, Rust/storage/migration change, operator
transport, provider, secret access, second `GraphDiff` calculator, Docker, browser/visual smoke,
remote CI, operator rehearsal, release, or production claim was added. PostgreSQL live runtime is not
claimed by this Web receipt; its separate `55441` evidence remains local non-production only. The
long-term goal remains `active`, and the next increment requires a fresh bilingual Necessity Record.

本增量未新增 public REST/OpenAPI/public SDK write、Web mutation、Rust/storage/migration change、operator transport、
provider、secret access、第二个 `GraphDiff` calculator、Docker、browser/visual smoke、remote CI、operator rehearsal、
release 或 production 声明。本 Web 回执不声称 PostgreSQL live runtime；独立的 `55441` 证据仍仅是本地非生产证据。
长期目标保持 `active`，下一项增量必须先创建新的双语 Necessity Record。
## 2026-08-02 Private Lifecycle Read Transport Receipt / 2026-08-02 私有生命周期读取传输回执

This bounded local transport increment advances Criterion 1 without closing it or the active
long-term goal. The existing lifecycle read loader now explicitly sends `cache: "no-store"` while
preserving request-memory Bearer authentication and `credentials: "omit"`. Focused BFF tests pin
exact Context/commit scope, cookie omission, `private, no-store` response headers, and 503 mapping;
the Inspector test pins the accessible `unavailable` state and redacts upstream diagnostics.

本次有界本地传输增量推进条件 1，但不关闭条件 1 或 active 长期目标。既有 lifecycle read loader 现显式发送
`cache: "no-store"`，同时保持请求内存 Bearer 认证与 `credentials: "omit"`。BFF 聚焦测试固定精确 Context/commit
scope、cookie omission、`private, no-store` response header 与 503 mapping；Inspector 测试固定可访问的
`unavailable` state 并脱敏 upstream diagnostic。

Fresh verification / 新鲜验证：focused lifecycle data/proxy/presenter/inspector tests `41 passed`; full Web suite
`294 passed`; `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `294`, and production build; Rust
format, workspace tests `220 passed, 41 ignored`, strict offline Clippy, locked Rust `1.85.0` check, and local contract
verifier passed.

新鲜验证：lifecycle data/proxy/presenter/inspector 聚焦测试 `41 passed`；完整 Web suite `294 passed`；`pnpm check:web`
通过，public SDK `15`、local SDK `135`、Web `294` 与 production build 均通过；Rust format、workspace tests `220 passed, 41 ignored`、
strict offline Clippy、锁定 Rust `1.85.0` check 与 local contract verifier 通过。

No Rust/storage/migration/API route/OpenAPI/public SDK write/Web mutation/operator transport/provider/secret/GraphDiff
change was added. PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release,
production, and filesystem cleanup remain `unobserved` or `deferred`. The long-term goal remains `active`; the next
increment requires a fresh bilingual Necessity Record.

未新增 Rust/storage/migration/API route/OpenAPI/public SDK write/Web mutation/operator transport/provider/secret/GraphDiff
变更。PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release、production 与
filesystem cleanup 继续为 `unobserved` 或 `deferred`。长期目标保持 `active`，下一项增量必须先创建新的双语 Necessity Record。

## 2026-08-02 Versioned Context Graph Pair Witness / 2026-08-02 版本化 Context Graph 成对见证

This local Criterion 2 increment is completed and locally verified, but it does not close Criterion 2 or the active
long-term goal. The Rust diff domain now owns an immutable ordered pair identity witness for one exact
`(project, context, source/baseline commit, target/revised commit)` scope. The protected local graph-diff response
returns a server-owned `pair_witness` with `schema_version=1`, project, Context, baseline commit, and revised commit;
the local SDK requires the exact witness and rejects missing, unknown, mixed-scope, schema-drifting, or self-pair
responses before they reach Web presentation. The graph projection still delegates to the single
`GraphDiff::between` calculator.

本地条件 2 增量已完成并取得本地验证，但不关闭条件 2 或 active 长期目标。Rust diff domain 现拥有一个不可变的有序
pair identity witness，绑定精确的 `(project, context, source/baseline commit, target/revised commit)` scope。
受保护 local graph-diff response 返回 server-owned `pair_witness`，包含 `schema_version=1`、project、Context、baseline
commit 与 revised commit；local SDK 强制要求该 exact witness，并在进入 Web presentation 前拒绝缺失、未知字段、混合 scope、schema 漂移或 self-pair。
图谱 projection 仍只委托给唯一的 `GraphDiff::between` calculator。

Fresh verification / 新鲜验证：

- `cargo test -p contextlab-diff-engine --offline -- --nocapture`: all package/unit/integration tests passed (`40 passed`).
- `cargo test -p contextlab-api --test commit_graph_snapshot_scope_contract --offline -- --nocapture`: `3 passed`.
- `cargo fmt --all -- --check`: passed; `cargo clippy --workspace --all-targets --offline -- -D warnings`: passed.
- `cargo test --workspace --quiet`: workspace passed; storage reported `220 passed, 41 ignored`.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: public SDK `15`, local SDK `137`, Web `294`, and production Web build passed.
- The initial Web graph-diff fixture run correctly failed on the missing new witness; after the fixture was updated, the focused and full Web suites passed. This red-to-green migration is retained as contract evidence.
- `pwsh -NoProfile -File .\\tests\\contract\\verify-local-contracts.test.ps1`: fixture tests passed. The live verifier reported source/graph/DTO/route checks passed, `graph_diff_application=passed count=1`, and `overall=unobserved` because no unified diff input was supplied.

新鲜验证：

- `cargo test -p contextlab-diff-engine --offline -- --nocapture`：package/unit/integration 全部通过（`40 passed`）。
- API scope contract：`3 passed`；`cargo fmt --all -- --check` 通过；strict offline Clippy 通过。
- `cargo test --workspace --quiet` 通过，storage 为 `220 passed, 41 ignored`；锁定 Rust `1.85.0` check 通过。
- `pnpm check:web` 通过：public SDK `15`、local SDK `137`、Web `294` 与 production Web build 均通过。
- Web graph-diff fixture 首次因缺失新 witness 正确红灯；补齐 fixture 后 focused 与 full Web suite 通过，该红绿迁移作为契约证据保留。
- fixture verifier 通过；live verifier 的 source/graph/DTO/route checks 与 `graph_diff_application=passed count=1` 通过，但因未提供 unified diff input，`overall=unobserved`。

No public REST/OpenAPI/public SDK write, Web mutation, migration, provider, secret access, operator transport,
second GraphDiff calculator, Docker/PostgreSQL runtime claim, authenticated browser/visual smoke, Git change-set,
remote CI, operator rehearsal, release, or production claim was added. PostgreSQL runtime, browser, Git, and external
release evidence remain `unobserved` or `deferred`; the long-term goal stays active and the next increment requires a
new bilingual Necessity Record.

未新增 public REST/OpenAPI/public SDK write、Web mutation、migration、provider、secret access、operator transport、第二个
GraphDiff calculator、Docker/PostgreSQL runtime 声明、authenticated browser/visual smoke、Git change-set、remote CI、operator
rehearsal、release 或 production 声明。PostgreSQL runtime、browser、Git 与外部 release evidence 继续为 `unobserved` 或
`deferred`；长期目标保持 active，下一项增量必须新增双语 Necessity Record。

## 2026-08-02 Pair Witness Integration Repair / 2026-08-02 成对见证集成修复

The pair-witness slice remained locally green after an independent review found and repaired a real API/SDK
timestamp mismatch. The local SDK accepts UTC RFC3339 `Z` and `+00:00`, preserves the protected route's safe
unavailability codes, and rejects overlapping GraphDiff node categories. The repair is parser validation only;
the server-owned pair witness and the sole Rust `GraphDiff::between` boundary are unchanged.

成对见证 slice 在独立审查发现并修复真实 API/SDK timestamp mismatch 后保持本地绿色。local SDK 接受 UTC RFC3339 `Z` 与
`+00:00`，保留受保护 route 的安全 unavailable error code，并拒绝重叠的 GraphDiff node category。本修复仅加强 parser
validation；server-owned pair witness 与唯一 Rust `GraphDiff::between` boundary 未改变。

Fresh integrated evidence at `2026-08-02T03:33:12.7143168+08:00`: Rust workspace `220 passed, 41 ignored`, fmt,
strict offline Clippy, locked Rust `1.85.0`, local SDK `141/141`, Web `294`, production Web build, and fixture
contract verifier passed. The scoped verifier reported `graph_diff_application=passed count=1` and
`overall=unobserved` without unified diff input. External release, PostgreSQL runtime for this slice, browser/visual,
Git, operator, and production evidence remain `unobserved` or `deferred`.

`2026-08-02T03:33:12.7143168+08:00` 的 integrated fresh evidence：Rust workspace `220 passed, 41 ignored`、fmt、strict offline
Clippy、锁定 Rust `1.85.0`、local SDK `141/141`、Web `294`、production Web build 与 fixture contract verifier 通过。范围 verifier
在未提供 unified diff input 时报告 `graph_diff_application=passed count=1` 与 `overall=unobserved`。本 slice 的 external release、
PostgreSQL runtime、browser/visual、Git、operator 与 production evidence 继续为 `unobserved` 或 `deferred`。

The long-term goal remains active. The next admitted increment is the private benchmark direct decision-diff multi-dataset
evidence test, after its own bilingual Necessity Record was written; it adds no product surface.

长期目标保持 active。下一项准入增量是 private benchmark direct decision-diff multi-dataset evidence test；其独立双语
Necessity Record 已创建，本增量不增加产品 surface。

## 2026-08-02 Benchmark Direct Decision-Diff Multi-Dataset Receipt / 2026-08-02 Benchmark 直接 Decision-Diff 多 Dataset 回执

This test-only receipt advances Criterion 3 without closing it. The existing protected direct benchmark decision-diff read now has a fresh integration receipt over the same two-dataset, four-case sealed fixture used by the benchmark breadth path. It proves HTTP `200`, exact project/Context and baseline/revised commit+decision scope, `passed -> regressed` status, modified accuracy evidence, `sample_count=4` and `required_sample_count=4` on both sides, deterministic replay, and recursive redaction. The response intentionally does not expose dataset IDs or raw case payloads; cross-dataset breadth is proven through the sealed fixture and aggregate counts rather than by widening the read contract.

本 test-only 回执推进条件 3，但不关闭条件 3。现有受保护 direct benchmark decision-diff read 现已取得同一 two-dataset、four-case sealed fixture 的新鲜 integration receipt。它证明 HTTP `200`、精确 project/Context 与 baseline/revised commit+decision scope、`passed -> regressed` status、modified accuracy evidence、两侧 `sample_count=4` 与 `required_sample_count=4`、确定性 replay 以及递归脱敏。响应按设计不暴露 dataset ID 或 raw case payload；跨 dataset 宽度通过 sealed fixture 与聚合计数证明，不扩大 read contract。

Fresh focused evidence at `2026-08-02T03:45:26.6799857+08:00`:

- `cargo test -p contextlab-api --test benchmark_breadth --offline -- --nocapture`: `1 passed`.
- `cargo test -p contextlab-api --lib local_benchmark_decision_diff --offline -- --nocapture`: `2 passed`.
- `cargo test -p contextlab-storage --test benchmark_breadth --offline -- --nocapture`: `1 passed`.
- `cargo test -p contextlab-evaluation --test benchmark_decision_diff --offline -- --nocapture`: `3 passed`.
- `cargo fmt --all -- --check`: passed.

Full workspace/Web gates, PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production remain separate evidence boundaries. The long-term goal remains active; the next increment requires a new bilingual Necessity Record after re-auditing the open criteria.

新鲜聚焦证据（`2026-08-02T03:45:26.6799857+08:00`）：上述 API breadth 为 `1 passed`、API direct diff 为 `2 passed`、storage breadth 为 `1 passed`、evaluation diff 为 `3 passed`，且 `cargo fmt --all -- --check` 通过。

Full workspace/Web gate、PostgreSQL runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 仍属于独立证据边界。长期目标保持 active；重新审计开放条件后，下一项增量必须先创建新的双语 Necessity Record。
## 2026-08-02 Private Benchmark Decision-Pair Witness / 2026-08-02 私有 Benchmark Decision-Pair Witness

This local contract receipt advances Criteria 2 and 3 without closing either criterion or the active long-term goal. The protected decision-bound benchmark workspace returns optional server-owned `decision_pair_witness` with `schema_version=1`, exact project/Context, and ordered baseline/revised commit+decision identities. Cohort-bound and single-scope routes omit it rather than inferring decision identity. The non-public local SDK owns strict parsing and client scope checks; Web consumes the SDK DTO through the existing `data -> presenter -> screen` boundary without recalculating evaluation or graph Diff.

本地 contract 回执推进条件 2 与条件 3，但不关闭任一条件或 active 长期目标。受保护 decision-bound benchmark workspace 返回可选的 server-owned `decision_pair_witness`，包含 `schema_version=1`、精确 project/Context 以及有序 baseline/revised commit+decision identity。cohort-bound 与 single-scope route 不返回它，也不从 cohort 推断 decision identity。非公开 local SDK 负责严格 parsing 与 client scope check；Web 经由既有 `data -> presenter -> screen` boundary 消费 SDK DTO，不重新计算 evaluation 或 graph Diff。

Fresh verification / 新鲜验证：API breadth `1 passed`; API local benchmark workspace unit filter `10 passed`; local SDK lint and `148 passed`; Web workspace/inspector focused tests `20 passed`; full Web suite `298 passed`; `pnpm check:web` passed with public SDK `15`, local SDK `148`, Web `298`, and production build; `cargo fmt --all -- --check`, workspace Rust `220 passed, 41 ignored`, strict offline Clippy, locked Rust `1.85.0` check, and `verify-local-contracts.test.ps1` fixture tests passed.

新鲜验证：API breadth `1 passed`；API local benchmark workspace unit filter `10 passed`；local SDK lint 与 `148 passed`；Web workspace/inspector focused tests `20 passed`；完整 Web suite `298 passed`；`pnpm check:web` 通过，public SDK `15`、local SDK `148`、Web `298` 与 production build 通过；`cargo fmt --all -- --check`、workspace Rust `220 passed, 41 ignored`、strict offline Clippy、锁定 Rust `1.85.0` check 与 `verify-local-contracts.test.ps1` fixture tests 通过。

Migration/API integrity verification is recorded as local evidence only; it validates the local migration and API contract wiring and does not establish PostgreSQL runtime or external release evidence.

Migration/API integrity verification 仅记录为 local evidence；它只验证本地 migration 与 API contract wiring，不构成 PostgreSQL runtime 或 external release evidence。

No public REST/OpenAPI/public SDK write, public write, migration, provider, operator transport, raw benchmark payload, secret access, Web mutation, or second `GraphDiff` calculator was added. PostgreSQL runtime remains `ignored/unobserved` because no disposable `CONTEXTLAB_TEST_DATABASE_URL` was provided; Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`. The long-term goal remains active; the next increment requires a new bilingual Necessity Record.

未新增 public REST/OpenAPI/public SDK write、public write、migration、provider、operator transport、raw benchmark payload、secret access、Web mutation 或第二个 `GraphDiff` calculator。由于未提供 disposable `CONTEXTLAB_TEST_DATABASE_URL`，PostgreSQL runtime 继续为 `ignored/unobserved`；Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。长期目标保持 active；下一项增量必须新增双语 Necessity Record。

## 2026-08-02 Private Lifecycle-Witness GraphDiff Review / 2026-08-02 私有 Lifecycle-Witness GraphDiff Review

This private version-backed read increment advances the lifecycle and versioned-diff criteria without closing either criterion or the active long-term goal. `PersistedContextGraphDiffReviewService` now requires exact-commit `ContextLifecycleReadFacts` for both review sides. The facts validate component content/provenance, graph nodes/edges, replay state, Context identity, and commit scope before comparison; missing, mixed-scope, or inconsistent witnesses fail closed. The API adapter maps lifecycle/repository failures to a safe unavailable response without exposing internal details.

本次 private version-backed read 增量推进 lifecycle 与 versioned-diff 条件，但不关闭任一条件或 active long-term goal。`PersistedContextGraphDiffReviewService` 现在要求两侧 review 都提供 exact-commit `ContextLifecycleReadFacts`。该 facts 在比较前校验 component content/provenance、graph nodes/edges、replay state、Context identity 与 commit scope；缺失、混合 scope 或不一致 witness 均 fail closed。API adapter 将 lifecycle/repository failure 映射为安全的 unavailable response，不暴露内部细节。

`GraphDiff::between` remains the sole graph-diff calculator. The increment is a private local read contract only; it adds no public REST/OpenAPI/SDK write, Web mutation, operator transport, migration, provider, secret access, or production-readiness claim.

`GraphDiff::between` 仍是唯一 graph-diff calculator。本增量仅是 private local read contract；未新增 public REST/OpenAPI/SDK write、Web mutation、operator transport、migration、provider、secret access 或 production-readiness 声明。

Fresh verification / 新鲜验证：storage lifecycle-witness GraphDiff review `4 passed`; API GraphDiff focused tests `13 passed`; API integration contract `3 passed`; workspace Rust `221 passed, 41 ignored`; format, strict offline Clippy, and locked Rust `1.85.0` checks passed; `pnpm check:web` passed with public SDK `15`, local SDK `148`, Web `298`, and production Web build; `tests/contract/verify-local-contracts.test.ps1` fixture verification passed.

新鲜验证：storage lifecycle-witness GraphDiff review `4 passed`；API GraphDiff focused tests `13 passed`；API integration contract `3 passed`；workspace Rust `221 passed, 41 ignored`；format、strict offline Clippy 与锁定 Rust `1.85.0` checks 通过；`pnpm check:web` 通过（public SDK `15`、local SDK `148`、Web `298` 与 production Web build）；`tests/contract/verify-local-contracts.test.ps1` fixture verification 通过。

These receipts do not establish PostgreSQL runtime, Docker, authenticated browser/visual smoke, remote CI, operator rehearsal, Git, release, or production evidence; those boundaries remain `unobserved` or `deferred`. The long-term goal remains active and the next increment still requires a new bilingual Necessity Record.

上述回执不构成 PostgreSQL runtime、Docker、authenticated browser/visual smoke、remote CI、operator rehearsal、Git、release 或 production evidence；这些边界继续为 `unobserved` 或 `deferred`。长期目标保持 active，下一项增量仍必须先创建新的双语 Necessity Record。

## 2026-08-02 Guarded Lifecycle to GraphDiff Integration / 2026-08-02 Guarded Lifecycle 到 GraphDiff 集成

This test-only local increment advances Criteria 2 and 4 without closing either or the active
long-term goal. The new storage integration test drives the real guarded lifecycle writer through
Context initialization, component creation, immutable content update, a second component, and
typed Uses add/remove. The same in-memory repository supplies exact replay facts and graph snapshots
to `PersistedContextGraphDiffReviewService`; the review projection proves exact commit scopes and
edge add/remove results, while missing and mismatched scopes fail closed.

本 test-only 本地增量推进条件 2 与 4，但不关闭任一条件或 active 长期目标。新的 storage integration test 通过真实 guarded
lifecycle writer 执行 Context initialization、component creation、immutable content update、第二个 component 以及 typed Uses
添加/移除。同一 in-memory repository 将 exact replay facts 与 graph snapshot 提供给
`PersistedContextGraphDiffReviewService`；review projection 证明 exact commit scope 与 edge add/remove result，missing 与
mismatched scope 均 fail closed。

Fresh local evidence / 新鲜本地证据：integration `3 passed`; focused lifecycle-witness review `4 passed`; API scope contract
`3 passed`; workspace Rust `221 passed, 41 ignored`; format; strict offline Clippy; locked Rust `1.85.0`; `pnpm check:web`
with public SDK `15`, local SDK `148`, Web `298`, and production build; local contract fixture verifier; and exactly one
`impl GraphDiff` source. PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, and production remain `ignored`, `unobserved`, or `deferred`.

新鲜本地证据：integration `3 passed`；focused lifecycle-witness review `4 passed`；API scope contract `3 passed`；workspace Rust
`221 passed, 41 ignored`；format；strict offline Clippy；锁定 Rust `1.85.0`；`pnpm check:web`（public SDK `15`、local SDK `148`、
Web `298` 与 production build）；local contract fixture verifier；以及唯一一个 `impl GraphDiff` source。PostgreSQL runtime、Docker、
authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或
`deferred`。

No public REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, operator transport, or second graph-diff
calculator was added. The next increment requires a new bilingual Necessity Record after re-reading the open criteria; external
release evidence remains deferred and outside the local queue.

未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret access、operator transport 或第二个 graph-diff
calculator。重新阅读开放条件后，下一项增量仍需新的双语 Necessity Record；external release evidence 继续延期且不进入本地队列。

## 2026-08-02 Private Capability Snapshot and Memory Scope Hardening / 2026-08-02 私有能力快照与 Memory Scope 硬化

This parallel local wave advances Criteria 1, 5, 6, and 7 without closing any criterion or the
active long-term goal. A bounded `gpt-5.6-luna` Workflow/Plugin worker added
`WorkflowPluginCapabilityBridge::snapshot_from_registry_snapshot`, so a Workflow scheduling decision
can resolve against one immutable `CapabilityRegistrySnapshotV1`; the existing live-registry helper
remains for compatibility. A bounded Knowledge/Memory worker found and repaired a real scope defect:
the in-memory Context Graph repository now rejects unknown project/Context/commit membership before
delegating projection persistence or reads, matching the PostgreSQL composite foreign-key boundary.

本次并行本地 wave 推进条件 1、5、6、7，但不关闭任一条件或 active 长期目标。有界 `gpt-5.6-luna` Workflow/Plugin worker
增加 `WorkflowPluginCapabilityBridge::snapshot_from_registry_snapshot`，使 Workflow scheduling decision 可以针对一个 immutable
`CapabilityRegistrySnapshotV1` 解析；既有 live-registry helper 继续保留以兼容。Knowledge/Memory worker 发现并修复了真实 scope defect：
内存 Context Graph repository 现在会在委托 projection 持久化或读取前拒绝未知 project/Context/commit membership，与 PostgreSQL 复合外键边界对齐。

Fresh local evidence / 新鲜本地证据：Workflow bridge `8 passed`; MCP `10 passed`; Knowledge/Memory projection `3 passed`;
embedding `6 passed`; storage `222 passed, 41 ignored`; full workspace Rust; strict offline Clippy; locked Rust `1.85.0`;
`cargo fmt --all -- --check`; `pnpm check:web` with public SDK `15`, Web `298`, and production build; local contract fixture
verification; and exactly one production `impl GraphDiff` all passed. A benchmark worker failed during dispatch with an invalid
request parameter and produced no product evidence; CLI/Docs worker found no code gap and its prior smoke receipts remain historical.

新鲜本地证据：Workflow bridge `8 passed`、MCP `10 passed`、Knowledge/Memory projection `3 passed`、embedding `6 passed`、storage
`222 passed, 41 ignored`、workspace Rust、strict offline Clippy、锁定 Rust `1.85.0`、`cargo fmt --all -- --check`、`pnpm check:web`
（public SDK `15`、Web `298` 与 production build）、local contract fixture verification，以及唯一一个 production `impl GraphDiff` 均通过。
Benchmark worker 因调度请求参数错误失败且没有产生产品证据；CLI/Docs worker 未发现代码缺口，其既有 smoke receipt 仍属于历史证据。

No public REST/OpenAPI/public SDK write, Web mutation, dynamic loading, provider call, migration,
secret access, operator transport, Docker/PostgreSQL runtime claim, authenticated browser/visual
smoke, Git change-set, remote CI, operator rehearsal, release, or production claim was added.
Those boundaries remain `unobserved` or `deferred`; external release evidence stays outside the local
queue. The next increment requires a new bilingual Necessity Record and a fresh open-criteria audit.

未新增 public REST/OpenAPI/public SDK write、Web mutation、dynamic loading、provider call、migration、secret access、operator transport、
Docker/PostgreSQL runtime 声明、authenticated browser/visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。
这些边界继续为 `unobserved` 或 `deferred`；external release evidence 继续不进入本地队列。下一项增量必须先进行新鲜开放条件审计并新增双语 Necessity Record。

## 2026-08-02 Private Commit-History Replay and Diff Correctness / 2026-08-02 私有提交历史回放与 Diff 正确性

The admitted private versioning increment is now locally integrated. `CommitHistory` and
`BranchHead` provide a read-only aggregate over one Context's commit graph, explicit born/unborn
branch heads, deterministic root-to-head replay, fail-closed missing-parent and scope validation,
and ancestry-only merge planning. The Diff review also repaired the narrow case where a line-ending
change and an EOF newline change occurred together; `TextDiff` now preserves that exact replacement
without changing the sole `GraphDiff::between` ownership.

准入的私有 versioning 增量现已完成本地集成。`CommitHistory` 与 `BranchHead` 为单一 Context 的 commit graph 提供只读
aggregate，支持显式 born/unborn branch head、确定性 root-to-head replay、missing-parent 与 scope fail-closed 校验，
以及仅基于 ancestry 的 merge planning。Diff review 同时修复了 line-ending change 与 EOF newline change 同时发生时的
窄边界；`TextDiff` 现保留 exact replacement，但没有改变唯一的 `GraphDiff::between` ownership。

Fresh local evidence / 新鲜本地证据:

- versioning focused history `8 passed`; full versioning crate `54 passed`; Diff crate `13 passed`;
  full workspace Rust exited successfully with storage `222 passed, 41 ignored`;
- `cargo fmt --all -- --check`, strict offline Clippy, and locked Rust `1.85.0` check passed;
  `pnpm check:web` passed with Web `298 passed` and production build;
- `tests/contract/verify-local-contracts.test.ps1` passed and source inspection found exactly one
  `impl GraphDiff`.

- versioning focused history `8 passed`；完整 versioning crate `54 passed`；Diff crate `13 passed`；full workspace Rust
  成功退出，storage 报告 `222 passed, 41 ignored`；
- `cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0` check 通过；`pnpm check:web` 通过，Web
  `298 passed` 且 production build 成功；
- `tests/contract/verify-local-contracts.test.ps1` 通过，source inspection 确认只有一个 `impl GraphDiff`。

This receipt advances Criterion 2 and supports Criterion 4 but does not close either criterion or
the active long-term goal. PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred`.
No public REST/OpenAPI/SDK write, Web mutation, operator transport, provider, migration, secret
access, or second graph-diff calculator was added. The next increment requires a new bilingual
Necessity Record after re-reading the open criteria; the existing benchmark decision-pair witness
record is the current candidate for dependency review.

本回执推进条件 2 并支持条件 4，但不关闭任一条件或 active long-term goal。PostgreSQL runtime、Docker、authenticated
browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或
`deferred`。未新增 public REST/OpenAPI/SDK write、Web mutation、operator transport、provider、migration、secret access
或第二个 graph-diff calculator。下一项增量需重新阅读开放条件并新增双语 Necessity Record；现有 benchmark decision-pair
witness 记录是当前依赖审查候选。

## 2026-08-02 Commit History Bound to Versioned Context Graph Review / 2026-08-02 提交历史绑定版本化 Context Graph 审阅

The private read boundary now composes the existing commit list/detail and branch-head ports into
the reusable versioning crate's validated `CommitHistory`. The new storage
`ContextCommitHistoryRepository` preserves `CommitHistory` ownership in `contextlab-versioning`,
rehydrates persisted `ContextCommit` values through the existing record decoder, validates complete
ancestry and explicit born/unborn heads, and fails closed for malformed changes, scope drift,
unknown heads, duplicate branches, or incomplete parents. `ContextCommitHistoryRepositoryAdapter`
allows API state to compose separately owned commit and branch-head trait objects.

The protected local `GET /api/v1/local/contexts/{context_id}/graph-diff` path now validates this
history witness before delegating to the unchanged `PersistedContextGraphDiffReviewService`. The
response contract, route, OpenAPI, public SDK, local SDK, Web mutation boundary, and the sole
`GraphDiff::between` calculator are unchanged. A fixture-only root cause repair aligned preview
commit change JSON and parent identity with the persisted snapshot fixture after fail-closed history
rehydration exposed the mismatch.

本地 private read boundary 现已将既有 commit list/detail 与 branch-head ports 组合为可复用 versioning crate 的
validated `CommitHistory`。新增 storage `ContextCommitHistoryRepository` 将 `CommitHistory` ownership 保留在
`contextlab-versioning`，通过既有 record decoder 重建持久化 `ContextCommit`，校验完整 ancestry 与显式
born/unborn head，并对 malformed changes、scope drift、unknown head、duplicate branch 与 incomplete parent
fail closed。`ContextCommitHistoryRepositoryAdapter` 允许 API state 组合分离拥有的 commit 与 branch-head trait object。

受保护 local `GET /api/v1/local/contexts/{context_id}/graph-diff` 现会在委托给未改变的
`PersistedContextGraphDiffReviewService` 前验证 history witness。response contract、route、OpenAPI、public SDK、
local SDK、Web mutation boundary 与唯一 `GraphDiff::between` calculator 均未改变。一次 fixture-only 根因修复在
history rehydration 正确暴露不匹配后，使 preview commit change JSON 与 parent identity 和 persisted snapshot fixture 对齐。

Fresh local receipt / 新鲜本地回执:

- storage history composition `2 passed`; history-bound graph review `3 passed`; independent
  history/snapshot contract `2 passed`; API graph-diff focused route `1 passed`; API crate `223 passed`;
  workspace Rust `227 passed, 41 ignored`;
- `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0` check,
  `pnpm check:web` (`15` public SDK, `148` local SDK, `298` Web tests, production build),
  `tests/contract/verify-local-contracts.test.ps1`, and `GRAPH_DIFF_IMPL_COUNT=1` passed;
- PostgreSQL/Docker runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
  release, and production remain `unobserved` or `deferred`.

- storage history composition `2 passed`；history-bound graph review `3 passed`；独立 history/snapshot contract
  `2 passed`；API graph-diff focused route `1 passed`；API crate `223 passed`；workspace Rust `227 passed, 41 ignored`；
- `cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`（public SDK `15`、
  local SDK `148`、Web `298`、production build）、`tests/contract/verify-local-contracts.test.ps1` 与
  `GRAPH_DIFF_IMPL_COUNT=1` 通过；
- PostgreSQL/Docker runtime、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与
  production 仍为 `unobserved` 或 `deferred`。

This advances Criterion 2 and supports Criterion 4; it does not close either criterion or the
active long-term goal. The next admitted increment requires a new bilingual Necessity Record. The
explicit branch-head selection policy and atomic multi-port read are deferred rather than implied
by this receipt.

本增量推进条件 2 并支持条件 4，但不关闭任一条件或 active long-term goal。下一项准入增量必须新增双语
Necessity Record。显式 branch-head selection policy 与 atomic multi-port read 继续 deferred，本回执不暗示它们已完成。

## 2026-08-02 Branch-Head-Bound Graph Review Hardening / 2026-08-02 Branch-Head-Bound Graph Review 硬化

This bounded Criterion 2/4 increment is `completed / verified locally`; the active long-term goal
remains active. The private read adapter now rejects an invalid source snapshot scope before any
branch-history repository read, and rejects a loaded `CommitHistory` whose Context differs from
either requested graph snapshot scope. The API maps that fail-closed condition to the existing
private review-unavailable response. / 本有界条件 2/4 增量标记为 `completed / verified locally`；长期目标继续 active。私有 read
adapter 现在会在任何 branch-history repository read 前拒绝无效 source snapshot scope，并拒绝与任一 graph snapshot scope
Context 不同的 loaded `CommitHistory`。API 将该 fail-closed condition 映射到既有 private review-unavailable response。

The fix was admitted by the bilingual Necessity Record in
`docs/superpowers/plans/2026-08-02-branch-head-bound-graph-review.md`. It is limited to storage
validation/error mapping and focused tests; no new public REST/OpenAPI/SDK route or method, Web
mutation, branch mutation, merge, rollback, migration, provider, operator transport, secret
access, or second graph-diff calculator was added. `GraphDiff::between` remains the sole calculator. /
该修复由 `docs/superpowers/plans/2026-08-02-branch-head-bound-graph-review.md` 中的双语 Necessity Record 准入。范围仅限
storage validation/error mapping 与 focused tests；未新增 public REST/OpenAPI/SDK route 或 method、Web mutation、branch
mutation、merge、rollback、migration、provider、operator transport、secret access 或第二个 graph-diff calculator。
`GraphDiff::between` 仍是唯一 calculator。

Fresh local receipt / 新鲜本地回执:

- branch-head/history review focused tests: `7 passed`; API graph-diff focused tests: `13 passed`;
- workspace Rust: API `223 passed`; storage `231 passed, 41 ignored`; no failures;
- `cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0`, and
  `tests/contract/verify-local-contracts.test.ps1` passed;
- `pnpm check:web`: public SDK `15`, local SDK `148`, Web `298`, production build passed;
- source inspection: `GRAPH_DIFF_IMPL_COUNT=1`, `GraphDiff::between` call count `10`.

- branch-head/history review focused tests：`7 passed`；API graph-diff focused tests：`13 passed`；
- workspace Rust：API `223 passed`；storage `231 passed, 41 ignored`；无失败；
- `cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` 与
  `tests/contract/verify-local-contracts.test.ps1` 通过；
- `pnpm check:web`：public SDK `15`、local SDK `148`、Web `298`、production build 通过；
- source inspection：`GRAPH_DIFF_IMPL_COUNT=1`，`GraphDiff::between` call count `10`。

PostgreSQL runtime tests requiring `CONTEXTLAB_TEST_DATABASE_URL` remained ignored; Docker,
authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production
remain unobserved or deferred. This receipt advances but does not close Criteria 2 or 4, any other
completion criterion, or the repository. The next local increment must begin with a fresh bilingual
Necessity Record; atomic multi-port read remains deferred. / 需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime tests
继续 ignored；Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续
为 unobserved 或 deferred。本回执推进但不关闭条件 2、条件 4、其他 completion criterion 或仓库整体。下一项本地增量必须
以新的双语 Necessity Record 开始；atomic multi-port read 继续 deferred。

## 2026-08-02 Private Context Commit-History Consistent Read / 2026-08-02 私有 Context 提交历史一致读取

Necessity Record / 必要性记录: This increment serves Criteria 2 and 4. The former history adapter
composed independent commit/detail and branch-head ports, so Memory and PostgreSQL could not prove
one observation point. The minimum boundary is one Memory read guard or one PostgreSQL
`REPEATABLE READ READ ONLY` transaction, followed by `CommitHistory::try_from_parts`. / 本增量服务条件 2 与 4。
原有 history adapter 组合独立 commit/detail 与 branch-head port，Memory 与 PostgreSQL 无法证明同一观察点。最小边界是
Memory 单一 read guard 或 PostgreSQL 单一 `REPEATABLE READ READ ONLY` transaction，随后使用 `CommitHistory::try_from_parts`。

`InMemoryContextGraphRepository` and `PostgresContextGraphRepository` now own this consistent read;
the protected graph-diff handler consumes the injected repository. No route, response, OpenAPI, SDK,
Web mutation, migration, provider, secret access, operator transport, or second GraphDiff calculator
was added. / `InMemoryContextGraphRepository` 与 `PostgresContextGraphRepository` 现在拥有该一致读取；protected graph-diff
handler 使用注入的 repository。未新增 route、response、OpenAPI、SDK、Web mutation、migration、provider、secret access、
operator transport 或第二个 GraphDiff calculator。

Fresh focused receipt / 新鲜 focused 回执: storage history `3 passed`; concrete Memory history `1 passed`;
PostgreSQL SQL-shape `1 passed`; protected API graph-diff `7 passed`; storage/API library checks passed;
`cargo fmt --all` completed. Full workspace/Web/Clippy/MSRV/contract, PostgreSQL runtime, Docker,
browser/visual, Git, remote CI, operator rehearsal, release, and production evidence are not claimed.
/ storage history `3 passed`；Memory concrete history `1 passed`；PostgreSQL SQL-shape `1 passed`；protected API graph-diff
`7 passed`；storage/API library check 通过；`cargo fmt --all` 已完成。不宣称 workspace/Web/Clippy/MSRV/contract、PostgreSQL
runtime、Docker、browser/visual、Git、remote CI、operator rehearsal、release 或 production 证据。

The increment is locally focused-verified but does not close Criteria 2 or 4; the long-term goal remains
`active`. / 本增量已完成本地 focused verification，但不关闭条件 2 或 4；长期目标保持 `active`。

## 2026-08-02 Private Context Commit-History Consistent Read: Full Local Receipt / 2026-08-02 私有 Context 提交历史一致读取：全量本地回执

This superseding receipt marks the specific `ContextCommitHistoryRepository` increment
`completed / verified locally`. Memory now reads the complete Context commit details, parents, and
branch heads under one read guard; PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction.
The protected graph-diff handler consumes the injected concrete repository, then reuses
`CommitHistory::try_from_parts` and the unchanged `GraphDiff::between` calculator. The earlier
focused-only receipt is historical; it no longer describes the current state of this path.

本 superseding 回执将特定 `ContextCommitHistoryRepository` 增量标记为 `completed / verified locally`。Memory
现以单一 read guard 读取完整 Context commit detail、parent 与 branch head；PostgreSQL 使用单一
`REPEATABLE READ READ ONLY` transaction。protected graph-diff handler 使用注入的 concrete repository，随后复用
`CommitHistory::try_from_parts` 与未改变的 `GraphDiff::between` calculator。此前 focused-only 回执属于历史状态，
不再描述该 path 的当前状态。

Fresh local verification / 新鲜本地验证: `cargo test --workspace --quiet` passed with API `223 passed`
and storage `233 passed, 41 ignored`; `cargo fmt --all -- --check`, strict offline Clippy (`-D warnings`),
and locked Rust `1.85.0` `cargo check --workspace --all-targets --locked --offline` passed; `pnpm check:web`
passed with public SDK `15`, local SDK `148 passed`, Web `298 passed`, and production build; the local contract
verifier passed; and source inspection found exactly one production `impl GraphDiff` with `10`
`GraphDiff::between` call sites.

新鲜本地验证：`cargo test --workspace --quiet` 通过，API `223 passed`、storage `233 passed, 41 ignored`；
`cargo fmt --all -- --check`、strict offline Clippy（`-D warnings`）与锁定 Rust `1.85.0` 的
`cargo check --workspace --all-targets --locked --offline` 通过；`pnpm check:web` 通过，public SDK `15`、
local SDK `148 passed`、Web `298 passed` 与 production build 通过；local contract verifier 通过；源码检查确认只有一个
production `impl GraphDiff` 与 `10` 个 `GraphDiff::between` 调用点。

This remains local non-production evidence. PostgreSQL runtime requiring `CONTEXTLAB_TEST_DATABASE_URL`,
Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production
remain `ignored`, `unobserved`, or `deferred`. Criteria 2 and 4 are advanced but remain open, as do
the other completion criteria; the long-term goal remains `active`. No public write, public
REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, operator transport, or
second graph-diff calculator was added. The next implementation requires a fresh bilingual Necessity Record.

本回执仍是本地非生产 evidence。需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime、Docker、authenticated
browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或
`deferred`。条件 2 与 4 得到推进但仍开放，其他 completion criteria 也继续开放；长期目标保持 `active`。未新增
public write、public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret access、operator transport
或第二个 graph-diff calculator。下一项实现必须先有新的双语 Necessity Record。

## 2026-08-02 Context History Construction-Path Hardening / 2026-08-02 Context History Construction-Path 硬化

The independent review found and the red regression reproduced a construction-path bypass:
`AppState::with_workspace_repositories` could silently compose the generic split-port adapter,
even though the real environment path used the backend-owned repository. The minimum repair adds
an explicit optional history repository to `WorkspaceRepositories`, makes omission fail closed,
and updates the versioned graph-diff fixture to inject the shared Memory repository. This keeps
all existing route, response, auth, quota, audit, and `GraphDiff::between` contracts unchanged.

独立审查发现并通过红色回归复现 construction-path bypass：即使真实 environment path 使用 backend-owned repository，
`AppState::with_workspace_repositories` 仍可能静默组合 generic split-port adapter。最小修复是在
`WorkspaceRepositories` 增加 explicit optional history repository，缺失时 fail closed，并让 versioned graph-diff fixture
显式注入共享 Memory repository。既有 route、response、auth、quota、audit 与 `GraphDiff::between` contract 均保持不变。

Fresh red/green evidence / 新鲜红绿证据: the old builder first returned a complete `CommitHistory`
and failed the new regression; after the repair, the regression passed. API library tests passed
`224`, protected graph-diff focused tests passed `7`, and the public-catalog versioned graph-diff
regression passed `1`. This is local non-production evidence only; PostgreSQL runtime, Docker,
authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, and production
remain `ignored`, `unobserved`, or `deferred`. The long-term goal remains active and the next
increment still requires its own bilingual Necessity Record.

新鲜红绿证据：旧 builder 首先返回完整 `CommitHistory`，新的 regression 因此失败；修复后该 regression 通过。API library
测试 `224` 个通过，protected graph-diff focused 测试 `7` 个通过，public-catalog versioned graph-diff regression `1` 个通过。
这仅是本地非生产 evidence；PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator
rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。长期目标保持 active，下一项增量仍需自己的双语
Necessity Record。

## 2026-08-02 Private Atomic Branch-Head Graph Witness / 2026-08-02 私有原子 Branch-Head Graph Witness

Necessity Record / 必要性记录: This increment directly serves Criteria 2 (replayable version
history) and 4 (Context Graph as the system skeleton). The remaining gap was branch selection:
the server-owned `BranchName`, selected head, complete history, and both immutable graph snapshots
needed one observation point. The minimum boundary is the private
`ContextGraphBranchHeadReviewWitnessRepository` in reusable Rust storage. It is preferred now
because exact-scope history and graph witnesses are already verified, while adding another Context
consumer or transport before atomic branch selection would leave a consistency gap. Non-goals are
public REST/OpenAPI/SDK writes, Web mutation, branch mutation, merge, rollback, migration, provider,
operator transport, release, production, and live PostgreSQL/Docker claims. / 必要性记录：本增量直接服务条件 2（可回放
版本历史）与条件 4（Context Graph 作为系统骨架）。剩余缺口是 branch selection：server-owned `BranchName`、selected head、
完整 history 与两份 immutable graph snapshot 必须来自同一个 observation point。最小边界是可复用 Rust storage 中的 private
`ContextGraphBranchHeadReviewWitnessRepository`。现在优先处理它，是因为 exact-scope history 与 graph witness 已经验证；在
atomic branch selection 之前增加其他 Context consumer 或 transport 会保留一致性缺口。非目标包括 public REST/OpenAPI/SDK write、
Web mutation、branch mutation、merge、rollback、migration、provider、operator transport、release、production 以及 live
PostgreSQL/Docker 声明。

The split `PersistedContextGraphHistoryReviewService` exposes only explicit commit-scope review;
it has no callable `review_branch_head`. Branch selection is exclusively on
`PersistedContextGraphWitnessReviewService`, constrained by the atomic witness repository. Memory
uses one `RwLock` read guard; PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction. The
application adapter delegates graph comparison to the existing review projection, and
`GraphDiff::between` remains the sole calculator. / split `PersistedContextGraphHistoryReviewService` 只暴露 explicit commit-scope
review，不存在可调用的 `review_branch_head`。Branch selection 仅位于受 atomic witness repository 约束的
`PersistedContextGraphWitnessReviewService`。Memory 使用一个 `RwLock` read guard；PostgreSQL 使用一个
`REPEATABLE READ READ ONLY` transaction。application adapter 将 graph comparison 委托给既有 review projection；
`GraphDiff::between` 仍是唯一 calculator。

Fresh local verification / 新鲜本地验证: Memory branch-bound `6 passed`; PostgreSQL SQL contract
`3 passed, 1 ignored`; API `222 passed`; storage `230 passed, 41 ignored`; workspace Rust had no
failures; `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0` check;
`pnpm check:web` with public SDK `15`, local SDK `148`, Web `298`, and production build; and
`tests/contract/verify-local-contracts.test.ps1` passed. Source inspection found zero split-service
branch selectors, one witness-service branch selector, one production `impl GraphDiff`, and ten
`GraphDiff::between` call sites. / 新鲜本地验证：Memory branch-bound `6 passed`；PostgreSQL SQL contract `3 passed, 1 ignored`；
API `222 passed`；storage `230 passed, 41 ignored`；workspace Rust 无失败；`cargo fmt --all -- --check`；strict offline Clippy；
锁定 Rust `1.85.0` check；`pnpm check:web` 的 public SDK `15`、local SDK `148`、Web `298` 与 production build；以及
`tests/contract/verify-local-contracts.test.ps1` 通过。源码检查发现 split service branch selector 为零、witness service branch selector 为一、
一个 production `impl GraphDiff` 与十个 `GraphDiff::between` 调用点。

This receipt is `completed / verified locally` for the named increment, but the long-term goal
remains `active`; Criteria 2 and 4 and the other completion criteria remain open. PostgreSQL live
runtime, Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release,
and production remain `ignored`, `unobserved`, or `deferred`. No public write, public
REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, or operator transport
was added. The next local implementation must begin with a new bilingual Necessity Record. / 本回执对命名增量标记为
`completed / verified locally`，但长期目标保持 `active`；条件 2、条件 4 与其他 completion criteria 仍开放。PostgreSQL live
runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为
`ignored`、`unobserved` 或 `deferred`。未新增 public write、public REST/OpenAPI/SDK write、Web mutation、migration、provider、
secret access 或 operator transport。下一项本地实现必须以新的双语 Necessity Record 开始。

## 2026-08-02 Private Commit-Graph Snapshot Scope Invariant / 2026-08-02 私有 Commit Graph Snapshot Scope 不变量

This bounded domain increment is `completed / verified locally`; the long-term goal remains
`active`. It serves Criteria 1 and 2 and the charter's fail-closed stable-UUID rule. A typed
`CommitGraphSnapshotScope` could still carry a nil project, Context, or commit UUID until the review
layer, so `CommitGraphSnapshot::new` now rejects that scope before graph materialization or
persistence with `CommitGraphSnapshotError::InvalidScope`. / 本有界 domain 增量标记为
`completed / verified locally`；长期目标保持 `active`。它服务条件 1、2 与章程的 fail-closed stable-UUID 原则。typed
`CommitGraphSnapshotScope` 在 review layer 之前仍可能携带 nil project、Context 或 commit UUID，因此
`CommitGraphSnapshot::new` 现在会在 graph materialization 或 persistence 前以 `CommitGraphSnapshotError::InvalidScope`
拒绝该 scope。

Necessity Record / 必要性记录: The snapshot domain, writer, Memory/PostgreSQL repositories, and
version-backed review already existed and were not expanded. The smallest affected boundary was
`crates/storage/src/commit_graph_snapshot.rs` plus its focused tests and bilingual plan/roadmap
records. No transport, schema, writer policy, migration, or second `GraphDiff` calculator changed.
/ 必要性记录：snapshot domain、writer、Memory/PostgreSQL repository 与 version-backed review 已存在，本次未扩展。最小受影响
边界是 `crates/storage/src/commit_graph_snapshot.rs`、focused tests 与双语 plan/roadmap 记录。没有改变 transport、schema、writer
policy、migration，也没有新增第二个 `GraphDiff` calculator。

Fresh red/green verification / 新鲜红绿验证: the nil-scope test failed before the constructor
guard, then focused snapshot tests passed `6`; snapshot repository contract tests passed `5`.
Workspace Rust had no failures with API `222 passed` and storage `230 passed, 41 ignored`;
`cargo fmt --all -- --check`, strict offline Clippy, locked Rust `1.85.0` check,
`pnpm check:web` (`15/148/298` plus production build), and
`tests/contract/verify-local-contracts.test.ps1` passed. Source inspection found one production
`impl GraphDiff` and ten `GraphDiff::between` call sites. / 新鲜红绿验证：nil-scope test 在 constructor guard 前失败，接入后
focused snapshot tests `6` 项通过；snapshot repository contract tests `5` 项通过。workspace Rust 无失败，API `222 passed`、
storage `230 passed, 41 ignored`；`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust `1.85.0` check、
`pnpm check:web`（`15/148/298` 与 production build）以及 `tests/contract/verify-local-contracts.test.ps1` 通过。源码检查发现一个
production `impl GraphDiff` 与十个 `GraphDiff::between` 调用点。

This receipt advances but does not close Criteria 1 or 2, any other completion criterion, or the
repository. PostgreSQL live runtime, Docker, authenticated browser/visual smoke, Git, remote CI,
operator rehearsal, release, and production remain `ignored`, `unobserved`, or `deferred`. The
next admitted increment is the parent-snapshot ancestry invariant: an existing parent commit
without a materialized graph snapshot must not be accepted for a new child snapshot. / 本回执推进但不关闭条件 1、2、
其他 completion criterion 或仓库整体。PostgreSQL live runtime、Docker、authenticated browser/visual smoke、Git、remote CI、
operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。下一项准入增量是 parent-snapshot
ancestry invariant：已有 parent commit 但缺少 materialized graph snapshot 时，不得被新 child snapshot 接受。

## 2026-08-02 Private Parent Graph Snapshot Ancestry Invariant / 2026-08-02 私有 Parent Graph Snapshot Ancestry 不变量

Necessity Record / 必要性记录: This increment directly serves Criteria 1, 2, and 4 and the
charter's fail-closed atomic writer rule. A child Context commit must not be durable when a
declared parent commit has no immutable graph snapshot. The minimum boundary is the private Rust
Memory/PostgreSQL writer contract and focused tests; no transport or UI surface is required. / 本增量直接服务条件 1、2、4
与章程的 fail-closed atomic writer rule。当声明的 parent commit 没有 immutable graph snapshot 时，Context child commit 不得
持久化。最小边界是 private Rust Memory/PostgreSQL writer contract 与 focused tests；不需要 transport 或 UI surface。

Memory now checks `snapshot_exists_for_context_commit` under its single write guard. PostgreSQL
`PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL` now joins `context_commit_graph_snapshots` and retains
`FOR KEY SHARE OF context_commits`; both reject with the existing `ScopeUnavailable` before child
persistence. / Memory 现在在单一 write guard 下检查 `snapshot_exists_for_context_commit`。PostgreSQL 的
`PARENT_COMMITS_FOR_CONTEXT_WRITE_SQL` 现在 join `context_commit_graph_snapshots`，并保留
`FOR KEY SHARE OF context_commits`；两者都在 child 持久化前使用现有 `ScopeUnavailable` 拒绝。

Fresh red/green and local verification / 新鲜红绿与本地验证: the new Memory regression first
failed by accepting a child with an existing but unmaterialized parent, then passed after the
guard; writer rejection tests `5 passed`; PostgreSQL SQL contract `1 passed`; workspace Rust
`cargo test --workspace --quiet --offline -j 1` passed with API `222 passed`, storage `233 passed,
41 ignored`; format, strict offline Clippy, and locked Rust `1.85.0` check passed; `pnpm check:web`
passed with public SDK `15`, local SDK `148`, Web `298`, and production build; the local contract
verifier passed; and source inspection found one production `impl GraphDiff` with `9` current
`GraphDiff::between` call sites. / 新鲜红绿与本地验证：新 Memory regression 首先因接受 existing but unmaterialized parent 的 child
而失败，接入 guard 后通过；writer rejection tests `5 passed`；PostgreSQL SQL contract `1 passed`；workspace Rust 的
`cargo test --workspace --quiet --offline -j 1` 通过，API `222 passed`、storage `233 passed, 41 ignored`；format、strict offline
Clippy 与锁定 Rust `1.85.0` check 通过；`pnpm check:web` 通过，public SDK `15`、local SDK `148`、Web `298` 与 production build
通过；local contract verifier 通过；源码检查确认一个 production `impl GraphDiff` 与当前 `9` 个 `GraphDiff::between` 调用点。

This is local non-production writer evidence only. PostgreSQL runtime requiring
`CONTEXTLAB_TEST_DATABASE_URL`, Docker, authenticated browser/visual smoke, Git, remote CI,
operator rehearsal, release, production, and public protected-write readiness remain
`ignored`, `unobserved`, or `deferred`; arbitrary direct SQL bypass is not claimed to be impossible.
No public REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, operator
transport, or second graph-diff calculator was added. The increment is `completed / verified
locally`, Criteria 1, 2, and 4 remain open, and the long-term goal remains `active`. / 这只是本地非生产 writer evidence；
需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、operator
rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`；不宣称 arbitrary
direct SQL bypass 不可能。未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret access、operator transport
或第二个 graph-diff calculator。本增量为 `completed / verified locally`，条件 1、2、4 仍开放，长期目标保持 `active`。

## 2026-08-02 Private Normal First-Parent Graph Review Ancestry / 2026-08-02 私有 Normal First-Parent 图审阅 Ancestry

The versioning core now exposes the read-only `CommitHistory::normal_first_parent_path` contract.
It returns an ordered inclusive source-to-target path only when the target is a normal first-parent
descendant, and fail-closes for unknown, reversed, unrelated, or merge-containing ranges. The
history-bound review service and atomic graph witness invoke this guard before snapshot review;
actual graph comparison still delegates to the sole `GraphDiff::between` implementation.

versioning core 现提供只读 `CommitHistory::normal_first_parent_path` contract。只有 target 是 normal first-parent descendant 时才返回
有序 inclusive source-to-target path，并对 unknown、reversed、unrelated 或包含 merge 的 range fail closed。history-bound review service
与 atomic graph witness 在 snapshot review 前调用该 guard；实际 graph comparison 仍委托给唯一的 `GraphDiff::between` implementation。

Fresh evidence / 新鲜证据：versioning `11 passed`、storage history review `8 passed`、workspace Rust API `222 passed` 与 storage
`237 passed, 41 ignored`、format、strict offline Clippy、locked Rust `1.85.0`、Web `15/148/298` plus production build、fixture verifier
通过；production `impl GraphDiff` 为 `1`，当前 `GraphDiff::between` matches 为 `10`。完整 local verifier 的 `overall=blocked` 仍来自既有
`LocalBenchmarkWorkspaceResponse` safe-DTO baseline，本增量未改变该基线。

This is local non-production read/replay evidence only. No public write, REST/OpenAPI/SDK write,
Web mutation, migration, provider, secret access, operator transport, Docker/PostgreSQL runtime,
browser, Git, remote CI, operator rehearsal, release, or production claim was added. Criteria 1,
2, and 4 remain open and the long-term goal remains active; deferred external evidence does not
block core development. / 本回执仅是本地非生产 read/replay 证据。未新增 public write、REST/OpenAPI/SDK write、Web mutation、migration、
provider、secret access、operator transport、Docker/PostgreSQL runtime、browser、Git、remote CI、operator rehearsal、release 或 production 声明。
条件 1、2、4 仍开放，长期目标保持 active；延期 external evidence 不阻断核心开发。

## 2026-08-02 Private Local Verifier Decision-Pair Witness Alignment / 2026-08-02 私有 Local Verifier Decision-Pair Witness 对齐

The local contract verifier now understands the already-approved optional redacted
`decision_pair_witness` on `LocalBenchmarkWorkspaceResponse`. It checks the root response fields,
the private nested witness envelope, and the nested decision scopes exactly; unknown and raw nested
fields remain fail-closed. / local contract verifier 现理解 `LocalBenchmarkWorkspaceResponse` 上已批准的可选脱敏
`decision_pair_witness`。它精确检查 root response field、private nested witness envelope 与 nested decision scope；unknown 与 raw nested
field 继续 fail closed。

Fresh evidence / 新鲜证据：fixture verifier `tests/contract/verify-local-contracts.test.ps1` 通过。direct verifier 修复前为
`safe_local_dto_fields=blocked`、`overall=blocked`；修复后为 `safe_local_dto_fields=passed`，local source/route/SDK/public-boundary checks
通过，`public_write_additions=unobserved`，无 unified diff input 时 `overall=unobserved`。本增量只修复 evidence-tool contract，不改变产品 route、
API/SDK/Web behavior、migration、provider、secret 或 GraphDiff calculation。

This is Criterion 8 local evidence convergence only, not a project closeout. PostgreSQL/Docker,
authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release, production, and
public-write readiness remain `ignored`, `unobserved`, or `deferred`; the long-term goal remains
active. / 本回执仅收束条件 8 的 local evidence convergence，不是项目收束。PostgreSQL/Docker、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release、production 与 public-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`；长期目标保持 active。

## 2026-08-02 Private Atomic Merge Review Witness / 2026-08-02 私有原子 Merge Review Witness

This increment closes the named local consistency gap for Criterion 2 (replayable version history)
and Criterion 4 (Context Graph as the system skeleton): the server-owned merge plan and its
immutable base/left/right graph snapshots now come from one Memory read guard or one PostgreSQL
`REPEATABLE READ READ ONLY` transaction. The service rejects a repository witness whose project,
Context, or requested tips do not match the caller scope. Missing `AppState` wiring fails closed at
the protected route rather than falling back to preview data. `GraphDiff::between` remains the
sole calculator. / 本增量收束条件 2（可回放版本历史）与条件 4（Context Graph 作为系统骨架）的本地一致性缺口：server-owned merge
plan 与 immutable base/left/right graph snapshot 现来自单一 Memory read guard 或 PostgreSQL `REPEATABLE READ READ ONLY` transaction。
service 拒绝 project、Context 或 requested tips 不匹配 caller scope 的 repository witness。`AppState` wiring 缺失时 protected route
fail closed，不回退到 preview data。`GraphDiff::between` 仍是唯一 calculator。

Fresh evidence / 新鲜证据：focused storage merge review `16 passed`; PostgreSQL SQL-shape contract `3 passed, 1 ignored`; protected
API library `223 passed`; workspace Rust completed with storage `238 passed, 41 ignored`; `cargo fmt --all -- --check`; strict offline
Clippy; locked Rust `1.85.0` check; `pnpm check:web` with public SDK `15`, local SDK `148`, Web `298`, and production build;
`tests/contract/verify-local-contracts.test.ps1`; `GRAPH_DIFF_IMPL_COUNT=1`; and public merge-review surface hits `0`.
新鲜证据：storage focused `16 passed`；PostgreSQL SQL-shape `3 passed, 1 ignored`；protected API `223 passed`；workspace Rust 完成且
storage `238 passed, 41 ignored`；fmt、strict offline Clippy、锁定 Rust `1.85.0`、Web `15/148/298 + production build`、contract
verifier、唯一 GraphDiff `1` 与 public merge-review surface `0` 均通过。

The ignored PostgreSQL runtime test requires `CONTEXTLAB_TEST_DATABASE_URL`; no Docker, secret,
production, remote CI, operator, Git, release, or authenticated browser evidence was fabricated.
The long-term goal remains active. The next implementation must begin with a new bilingual
Necessity Record for the dependency-ready private lifecycle branch-head target binding; it must
not expand public transport or mutation scope. / ignored PostgreSQL runtime test 需要 `CONTEXTLAB_TEST_DATABASE_URL`；未伪造 Docker、secret、
production、remote CI、operator、Git、release 或 authenticated browser evidence。长期目标保持 active。下一项实现必须先为依赖就绪的
private lifecycle branch-head target binding 新增双语 Necessity Record；不得扩大 public transport 或 mutation scope。

## 2026-08-02 Private Lifecycle Branch-Head Target Binding / 2026-08-02 私有 Lifecycle Branch-Head Target Binding

Status / 状态: `completed / verified locally` for this named local Web increment; the long-term
goal remains `active`. / 本命名本地 Web 增量状态为 `completed / verified locally`；长期目标保持 `active`。

The private lifecycle editor and graph review now share the same server-owned branch target. The
editor uses the exact branch name and expected head commit; the graph review uses that head as its
revised candidate. Callback-identity changes no longer clear a valid selection, while loading,
error, empty, unborn, null, or changed-head transitions cannot reuse an old committed pair. The
bridge remains composition-only and does not calculate a diff.

私有 lifecycle editor 与 graph review 现共享同一 server-owned branch target。editor 使用精确 branch name 与 expected head commit；graph
review 将该 head 作为 revised candidate。callback identity 变化不再清空有效 selection；loading、error、empty、unborn、null 或 head 变化
都不能复用旧 committed pair。bridge 仍仅负责组合，不计算 diff。

Fresh local evidence / 新鲜本地证据: P1 red `22 passed, 2 failed`; focused Web `33/33`;
Web lint; `pnpm check:web` TS SDK `15`, local SDK `148`, Web `304`, production build; workspace
Rust storage `238 passed, 41 ignored`; fmt; strict offline Clippy; locked Rust `1.85.0`; local
contract verifier and its fixture suite; one production `impl GraphDiff`; ten call sites; public
merge-review surface hits `0`.

新鲜本地证据：P1 红测 `22 passed, 2 failed`；focused Web `33/33`；Web lint；`pnpm check:web` TS SDK `15`、local SDK `148`、Web `304`、
production build；workspace Rust storage `238 passed, 41 ignored`；fmt；strict offline Clippy；锁定 Rust `1.85.0`；local contract verifier 与
fixture suite；一个 production `impl GraphDiff`；十个 call site；public merge-review surface hits `0`。

The direct verifier remains `overall=unobserved` without unified diff input. PostgreSQL runtime,
Docker, authenticated browser/visual smoke, Git, remote CI, operator rehearsal, release,
production, and public protected-write readiness remain `ignored`, `unobserved`, or `deferred`.
No public REST/OpenAPI/SDK write, migration, provider, secret, operator transport, or production
claim was added. Criteria 1, 2, and 4 advance but remain open; the next increment requires a
fresh bilingual Necessity Record.

未提供 unified diff input 时 direct verifier 继续为 `overall=unobserved`。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release、production 与 public protected-write readiness 继续为 `ignored`、`unobserved` 或 `deferred`。未新增
public REST/OpenAPI/SDK write、migration、provider、secret、operator transport 或 production 声明。条件 1、2、4 得到推进但仍开放；下一
增量必须有新的双语 Necessity Record。

## 2026-08-02 Private Context Lifecycle Error Redaction / 2026-08-02 私有 Context 生命周期错误脱敏

Status / 状态: `completed / verified locally` for the bounded local security increment; the
long-term goal remains `active`. / 本有界本地安全增量状态为 `completed / verified locally`；长期目标保持 `active`。

An independent Luna review found that the lifecycle Web data adapter retained upstream
`body.message` and the editor appended it for structured failures other than the disabled gate.
The minimum repair is now in place: proxy parsing and `LocalLifecycleProxyError` construction both
replace upstream diagnostics with a status-only local message, while the editor uses the existing
bilingual status/error-code presenter. Conflict, rate-limit, and unknown-status red regressions
prove that private diagnostics do not enter the UI notice or error object.

独立 Luna 审查发现 lifecycle Web data adapter 会保留 upstream `body.message`，而 editor 在 disabled gate 之外会把它拼接到 structured failure。
现在已完成最小修复：proxy parsing 与 `LocalLifecycleProxyError` construction 都将 upstream diagnostic 替换为仅含 status 的本地 message，
editor 继续使用既有双语 status/error-code presenter。conflict、rate-limit 与 unknown-status 红绿回归证明 private diagnostic 不会进入 UI notice 或 error object。

Fresh local evidence / 新鲜本地证据: lifecycle data `9 passed`; editor `14 passed`;
`pnpm check:web` passed with public SDK `15`, local SDK `148`, Web `308` tests and production
build; `cargo fmt --all -- --check`; offline workspace Rust API `223 passed`, storage `238 passed,
41 ignored`; strict offline Clippy; locked Rust `1.85.0`; fixture verifier; `GRAPH_DIFF_IMPL_COUNT=1`;
and `10` `GraphDiff::between` matches.

新鲜本地证据：lifecycle data `9 passed`；editor `14 passed`；`pnpm check:web` 通过（public SDK `15`、local SDK `148`、Web `308` tests 与
production build）；`cargo fmt --all -- --check`；offline workspace Rust API `223 passed`、storage `238 passed, 41 ignored`；strict offline
Clippy；锁定 Rust `1.85.0`；fixture verifier；`GRAPH_DIFF_IMPL_COUNT=1`；以及 `10` 个 `GraphDiff::between` 匹配。

No public write, REST/OpenAPI/SDK change, migration, provider, secret access, operator transport,
or second GraphDiff calculator was added. PostgreSQL runtime, Docker, authenticated browser/visual
smoke, Git, remote CI, operator rehearsal, release, and production remain `ignored`, `unobserved`,
or `deferred`. The stale benchmark-breadth pointer in older completion-criteria text is historical;
later benchmark breadth, direct diff, and decision-pair receipts already exist and must not trigger
duplicate implementation. The next increment still requires a fresh bilingual Necessity Record
after a dependency audit.

未新增 public write、REST/OpenAPI/SDK change、migration、provider、secret access、operator transport 或第二个 GraphDiff calculator。PostgreSQL runtime、
Docker、authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。
旧 completion-criteria 文本中的 benchmark breadth 指针属于历史表述；后续 benchmark breadth、direct diff 与 decision-pair 回执已经存在，不得因此重复实现。
下一项增量仍须在依赖审计后先写新的双语 Necessity Record。

## 2026-08-02 Private Benchmark Evidence Error Redaction / 2026-08-02 私有 Benchmark Evidence 错误脱敏

This bounded local security increment is `completed / verified locally` and advances Criterion 6;
the long-term goal remains `active`. Fresh red tests observed upstream diagnostic leakage in both
decision and run-details errors and in the inspector notice path. The data adapter now retains only
stable error code/status/retry-after plus a bilingual local message, and the inspector delegates to
the existing status/retry presenter without reading `body.message`.

本有界本地安全增量状态为 `completed / verified locally`，推进条件 6；长期目标保持 `active`。新鲜红测在 decision、run-details error 与
inspector notice path 真实观察到 upstream diagnostic 泄漏。data adapter 现仅保留 stable error code/status/retry-after 与双语本地文案，inspector
委托既有 status/retry presenter，不读取 `body.message`。

Fresh verification / 新鲜验证: red focused `3 passed, 3 failed`; green focused `11 passed`; `pnpm check:web` passed with public SDK `15`, Web `310`,
lint, local SDK checks, and production build; workspace Rust API `223 passed`, storage `238 passed, 41 ignored`; fmt; strict offline Clippy; locked
Rust `1.85.0`; contract fixture; `GRAPH_DIFF_IMPL_COUNT=1`; and `10` `GraphDiff::between` matches.

新鲜验证：focused 红测 `3 passed, 3 failed`；绿测 `11 passed`；`pnpm check:web` 通过（public SDK `15`、Web `310`、lint、local SDK checks 与 production build）；
workspace Rust API `223 passed`、storage `238 passed, 41 ignored`；fmt；strict offline Clippy；锁定 Rust `1.85.0`；contract fixture；
`GRAPH_DIFF_IMPL_COUNT=1`；以及 `10` 个 `GraphDiff::between` 匹配。

No Rust/API/SDK/OpenAPI/public write, migration, provider, secret access, operator transport,
Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git, remote CI, operator rehearsal,
release, or production claim was added. Those boundaries remain `ignored`, `unobserved`, or
`deferred`. The next increment requires a new bilingual Necessity Record for private ContextGraph
review path snapshot completeness, so an intermediate missing commit snapshot cannot be compared.

未新增 Rust/API/SDK/OpenAPI/public write、migration、provider、secret access、operator transport、Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release 或 production 声明。上述边界继续为 `ignored`、`unobserved` 或 `deferred`。下一项必须先为 private ContextGraph review path
snapshot completeness 新增双语 Necessity Record，确保中间 commit snapshot 缺失时不能执行比较。

## 2026-08-30 Private ContextGraph Review Path Snapshot Completeness / 2026-08-30 私有 ContextGraph 审阅路径 Snapshot 完整性

Status / 状态: `completed / verified locally` for this bounded private storage increment; the long-term goal remains `active`. / 本有界私有存储增量为 `completed / verified locally`；长期目标保持 `active`。

The backend-owned graph review witness now loads and validates ordered immutable snapshots for every intermediate commit on a normal `source -> middle -> target` first-parent path. Memory uses one read guard and PostgreSQL one consistent read transaction. Missing, out-of-order, duplicate, or scope-drifting intermediate snapshots are rejected before the existing comparison path reaches the sole `GraphDiff::between` implementation. A persisted InMemory regression materializes only source/target snapshots while retaining the middle commit record and proves that the middle scope fails with `StorageRepositoryError::ScopeUnavailable` before comparison.

backend-owned graph review witness 现为 normal `source -> middle -> target` first-parent path 的每个中间 commit 读取并校验有序 immutable snapshot。Memory 使用一个 read guard，PostgreSQL 使用一个一致性只读 transaction。任一缺失、乱序、重复或 scope 漂移的中间 snapshot 都会在既有 comparison path 到达唯一的 `GraphDiff::between` implementation 前被拒绝。持久化 InMemory 回归仅 materialize source/target snapshot，保留 middle commit record，并证明 middle scope 会在比较前以 `StorageRepositoryError::ScopeUnavailable` fail closed。

Fresh evidence / 新鲜证据：`cargo +1.85.0 test --workspace` exited `0` and includes the focused storage composition suite (`7 passed`); fmt, strict offline Clippy, locked offline Rust 1.85 check, `pnpm check:web` (Web `310 passed` plus production build), and the local contract fixture passed. The direct local verifier exited `0`, with source/DTO/route/GraphDiff checks passed and `overall=unobserved` solely because no unified diff was supplied. Source inspection found exactly one production `impl GraphDiff` at `crates/diff-engine/src/lib.rs:81`; `GraphDiff::between` has ten Rust textual matches, which is distinct from implementation count. The intended mutation-red receipt is `unobserved`; the temporary loader mutation was restored without an assertion-bearing failure capture.

新鲜证据：`cargo +1.85.0 test --workspace` 以 `0` 退出，包含 focused storage composition suite（`7 passed`）；fmt、strict offline Clippy、locked offline Rust 1.85 check、`pnpm check:web`（Web `310 passed` 与 production build）及 local contract fixture 均通过。direct local verifier 以 `0` 退出，source/DTO/route/GraphDiff 检查通过；仅因未提供 unified diff 而为 `overall=unobserved`。源码检查发现 `crates/diff-engine/src/lib.rs:81` 只有一个 production `impl GraphDiff`；`GraphDiff::between` 有十个 Rust textual match，二者不能混同。预期 mutation-red 回执为 `unobserved`：临时 loader mutation 已恢复，未捕获带断言的失败输出。

PostgreSQL live runtime still requires a disposable database and `CONTEXTLAB_TEST_DATABASE_URL`; it remains `ignored`/`unobserved`, not live evidence for a missing intermediate row. Docker, browser, Git/remote CI, operator rehearsal, release, production, public-write readiness, and all transport or mutation expansion remain `unobserved` or `deferred`. The long-term goal remains active and the next increment must begin with a fresh bilingual Necessity Record.

PostgreSQL live runtime 仍需要 disposable database 与 `CONTEXTLAB_TEST_DATABASE_URL`，因此保持 `ignored`/`unobserved`，不是缺失中间 row 的 live evidence。Docker、browser、Git/remote CI、operator rehearsal、release、production、public-write readiness，以及所有 transport 或 mutation 扩展继续为 `unobserved` 或 `deferred`。长期目标保持 active；下一增量必须先有新的双语 Necessity Record。

## 2026-08-30 Private ContextLab Browser Smoke Evidence / 2026-08-30 私有 ContextLab 浏览器 Smoke 证据

Status / 状态: `completed / verified locally` for this bounded browser evidence increment; the long-term goal remains `active`. / 本有界浏览器证据增量为 `completed / verified locally`；长期目标保持 `active`。

The current production Next build was exercised through a black-box Playwright harness at desktop (1440x1100) and mobile (390x844) viewports. Both pages rendered the ContextLab workspace with all required workspace and private Workflow binding controls, no page errors, no unexpected console errors, no failed HTTP responses, and no horizontal overflow. A real desktop click on the private binding inspector, after entering a memory-only Bearer token, produced the expected preview mode `503` unavailable response and one redacted local notice. The response body contained only the stable `contextlab_web_api_unavailable` code and local configuration message.

当前 production Next build 已通过 black-box Playwright harness 在 desktop（1440x1100）与 mobile（390x844）viewport 实际检查。两种尺寸均渲染 ContextLab workspace、完整 workspace 与 private Workflow binding controls，无 page error、无意外 console error、无失败 HTTP response，且没有 horizontal overflow。desktop 在输入仅存于内存的 Bearer token 后真实点击 private binding inspector，得到预期 preview mode `503` unavailable response 与一条脱敏本地 notice。response body 仅含稳定的 `contextlab_web_api_unavailable` code 与本地 configuration message。

Fresh evidence / 新鲜证据: `python scripts/verify-contextlab-browser-smoke.py` under `with_server.py` with the built Next server exited `0`; desktop and mobile receipts both reported empty selector, page-error, response-failure, and overflow fields. The desktop probe reported status `503`, notice count `1`, and no upstream diagnostic. The inspected screenshots are `%TEMP%/contextlab-browser-smoke-desktop.png` and `%TEMP%/contextlab-browser-smoke-mobile.png`.

新鲜证据：使用已构建 Next server 通过 `with_server.py` 运行 `python scripts/verify-contextlab-browser-smoke.py`，退出码为 `0`；desktop 与 mobile 回执的 selector、page-error、response-failure 与 overflow 字段均为空。desktop probe 报告 status `503`、notice count `1`，且没有 upstream diagnostic。检查过的截图为 `%TEMP%/contextlab-browser-smoke-desktop.png` 与 `%TEMP%/contextlab-browser-smoke-mobile.png`。

This is preview-mode browser evidence only. Authenticated browser-to-BFF-to-protected-Axum runtime, PostgreSQL/Docker, Git/remote CI, operator rehearsal, release, production, and public-write readiness remain `unobserved` or `deferred`. No application behavior or public contract was changed; the long-term goal remains active and the next increment requires a fresh bilingual Necessity Record.

本次仅为 preview-mode browser evidence。authenticated browser-to-BFF-to-protected-Axum runtime、PostgreSQL/Docker、Git/remote CI、operator rehearsal、release、production 与 public-write readiness 继续为 `unobserved` 或 `deferred`。未修改 application behavior 或 public contract；长期目标保持 active，下一增量必须先有新的双语 Necessity Record。

## 2026-08-31 Protected Browser Runtime Dependency Audit / 2026-08-31 受保护浏览器运行时依赖审计

This documentation-only increment records a fresh dependency audit for the next named boundary:
authenticated browser -> same-origin BFF -> protected Axum runtime. Docker CLI `29.6.2` is present,
but `docker info` and `docker version` did not obtain a server response; one `docker desktop start`
attempt left `com.docker.service` at `Stopped/Manual`. Windows has no `postgres`, `initdb`, `pg_ctl`,
`createdb`, or `psql` command, and WSL has no usable distribution. `.env.example` confirms that
protected mode requires PostgreSQL, authentication settings, and bounded rate-limit settings.

本次仅更新文档，记录下一条具名边界 authenticated browser -> same-origin BFF -> protected Axum runtime 的新鲜依赖审计。
Docker CLI `29.6.2` 存在，但 `docker info` 与 `docker version` 未取得 server response；一次 `docker desktop start` 后
`com.docker.service` 仍为 `Stopped/Manual`。Windows 命令路径中没有 `postgres`、`initdb`、`pg_ctl`、`createdb` 或 `psql`，
WSL 没有可用发行版。`.env.example` 确认 protected mode 需要 PostgreSQL、authentication settings 与 bounded rate-limit settings。

The audit itself is `completed / verified locally`; protected runtime success, live PostgreSQL
persistence, and authenticated browser-to-BFF-to-protected-Axum success remain `unobserved`.
No source, API, SDK, OpenAPI, Web, migration, provider, secret, database, write, or GraphDiff
behavior changed. The long-term goal remains `active`; runtime evidence will be revisited only
when a disposable loopback PostgreSQL service is directly available.

本次审计本身为 `completed / verified locally`；protected runtime success、live PostgreSQL persistence 与 authenticated
browser-to-BFF-to-protected-Axum success 继续为 `unobserved`。未修改 source、API、SDK、OpenAPI、Web、migration、provider、
secret、database、write 或 GraphDiff behavior。长期目标保持 `active`；仅在 disposable loopback PostgreSQL service 实际可用时重新进入 runtime evidence。
