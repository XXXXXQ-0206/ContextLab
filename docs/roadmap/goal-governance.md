# Goal Governance and Increment Admission / 目标治理与增量准入

## Purpose / 目的

This document governs long-term goal adjustments and the admission of new ContextLab work. It protects the project charter: Context remains the primary abstraction; reusable Rust domain boundaries, stable contracts, persistent storage, a design-system-first interface, security, reproducibility, and bilingual documentation remain non-negotiable.

本文约束 ContextLab 的长期目标调整与新工作准入。它守护项目宪章：Context 始终是首要抽象；可复用的 Rust 领域边界、稳定契约、持久化存储、设计系统优先的界面、安全性、可复现性和中英双语文档都不可牺牲。

## 2026-07-15 Local Delivery Scope / 2026-07-15 本地交付范围

Remote disposable CI, operator-approved change rehearsals, and public protected-write, release, or production promotion are explicitly deferred because their external operating conditions are unavailable. They are future deployment prerequisites, not active completion work, gates, subagents, or waiting states for the local open-source engineering roadmap. No current increment may audit, request, simulate, or substitute receipts for them.

由于外部运行条件不可用，远端 disposable CI、operator 批准的变更演练，以及 public protected-write、release 或生产推广均被明确延期。它们是未来部署前置，不属于当前本地开源工程路线图中的活跃完成工作、门禁、子任务或等待状态。任何当前增量都不得审计、索取、模拟或替代这些回执。

## 2026-07-13 Assessment / 2026-07-13 评估

The previous goal adjustment had real operational effect, but incomplete metadata effect. The long-running task was renamed to `持续推进 ContextLab 平台建设`; its subsequent work followed the intended dependency order: trusted group-to-RBAC resolution, private protected-route rate limiting, audit-retention metadata, redacted audit review, then fresh disposable-PostgreSQL and CI evidence. The task's immutable preview still shows the original prompt because the task system exposes no external objective-metadata editor. The governing source of truth is therefore this document, `active-long-term-goal.md`, the completion criteria, and the follow-up instruction sent to the active task.

上次目标调整已经改变实际执行，但没有完全改写任务元数据。长期任务已更名为 `持续推进 ContextLab 平台建设`；其后续工作遵循了预期依赖顺序：可信 group-to-RBAC 解析、private protected-route 限流、审计留存元数据、去标识化审计审查，再到新鲜 disposable PostgreSQL 与 CI 证据。任务的不可变预览仍显示原始提示，因为任务系统没有提供外部目标元数据编辑接口。因此，本文件、`active-long-term-goal.md`、完成条件和发送给活跃任务的后续指令共同构成治理事实来源。

The adjustment is effective for execution and prioritization, but insufficiently explicit about why a new increment may start. This gap risks feature drift even when each individual slice is well implemented.

该调整已经对执行顺序生效，但对“新增量为何可以启动”的约束还不够明确。即使每个切片都实现良好，这个缺口仍可能导致功能漂移。

## Adjusted Objective / 调整后的目标

ContextLab continues toward the charter's full platform vision, but now follows a convergence-first, evidence-gated delivery strategy. Work advances only when it closes a named unmet completion condition or supplies evidence required to decide that condition. A feature must not start merely because it is attractive, easy, or locally adjacent.

ContextLab 继续迈向宪章规定的完整平台愿景，但从现在起采用“收束优先、证据门控”的交付策略。只有能收束已命名的未完成条件，或能为该条件提供决策所需证据的工作，才可推进。不得因为功能吸引人、实现容易或恰好相邻而启动。

### Priority Order / 优先顺序

1. **Advance dependency-ready core convergence.** Continue private Context-first domain, versioning, graph, storage, evaluation, and design-system increments when their own Necessity Records show that local dependencies are satisfied. Missing external release receipts do not stop this core work.
2. **Defer unavailable external deployment work.** Do not implement, audit, wait for, or decide public protected-write promotion, release, or production rollout while their external operating conditions are unavailable. Keep those future prerequisites separate from local core convergence.
3. **Build the Context editing vertical slice.** Reuse the approved versioning, graph, authorization, storage, API, SDK, and design-system contracts for Context/Prompt/Schema and graph-relationship editing, then branches, merges, and replay.
4. **Build benchmark-driven decisions.** Add reusable datasets, benchmark suites, regression thresholds, scorecard/dashboard decisions, and semantic/behavior/evaluation diff before expanding AI-assistance features.
5. **Expand remaining convergence areas.** Advance workflow, memory, knowledge, MCP/plugins, Desktop/CLI, release automation, and contributor workflows only when their dependencies are ready and their completion-condition link is recorded.

1. **推进依赖就绪的核心收束。** 当私有 Context-first domain、versioning、graph、storage、evaluation 与 design-system 增量通过各自 Necessity Record 证明本地依赖已满足时，继续实施。缺失外部发布回执不得停止这些核心工作。
2. **延期不可用的外部部署工作。** 在外部运行条件不可用时，不实施、审计、等待或决定 public protected-write promotion、release 或生产推广；这些未来前置必须与本地核心收束保持分离。
3. **建设 Context 编辑垂直切片。** 复用已批准的版本、图谱、授权、存储、API、SDK 与设计系统契约，推进 Context/Prompt/Schema 和图谱关系编辑，之后再推进分支、合并与回放。
4. **建设 benchmark 驱动的决策能力。** 在扩展 AI 辅助功能前，先增加可复用 dataset、benchmark suite、回归阈值、scorecard/dashboard 决策，以及 semantic/behavior/evaluation diff。
5. **扩展其余收束领域。** 仅当依赖准备完成且已记录与完成条件的关系时，才推进 workflow、memory、knowledge、MCP/plugin、Desktop/CLI、发布自动化和贡献工作流。

## Increment Admission Rule / 增量准入规则

Before implementation starts, every new increment must add a **Necessity Record / 必要性记录** to its implementation plan. Small documentation-only corrections may record the same fields in this document's decision log instead. The record must state:

1. the exact completion criterion or charter principle served;
2. the unmet dependency, risk, or evidence gap;
3. why this increment is the next dependency-ready choice rather than a later roadmap item;
4. explicit non-goals that prevent scope expansion;
5. the smallest affected boundary and expected bilingual documentation;
6. fresh verification commands or evidence required before the next increment may begin.

每个新增量在开始实现前，都必须在实施计划中增加 **Necessity Record / 必要性记录**。小型纯文档修正可在本文件的决策日志中记录同样字段。记录必须写明：

1. 服务的确切完成条件或宪章原则；
2. 尚未满足的依赖、风险或证据缺口；
3. 为什么该增量是当前依赖已满足的下一选择，而不是路线图中的后续项目；
4. 防止范围膨胀的明确非目标；
5. 最小受影响边界和预期的中英双语文档；
6. 在开始下一增量前必须取得的新鲜验证命令或证据。

An increment without this record is out of scope. Root-cause fixes discovered while running an admitted release gate remain admissible only when the record is updated to show the blocked criterion, the minimal fix, and its regression proof.

没有该记录的增量一律视为超出范围。执行已准入的发布门禁时发现的根因修复，只有在记录更新为“被阻塞的条件、最小修复和回归证明”后，才可继续推进。

## Decision Log / 决策日志

### 2026-07-13: Disposable PostgreSQL evidence and CI gate

**Criterion / 条件：** Reliable release gates and production security/collaboration evidence.

**Need / 必要性：** The guarded write, group authorization, audit retention, and redacted review contracts have ignored PostgreSQL tests. Without fresh isolated execution, the project cannot truthfully treat persistence, migration compatibility, concurrency, or redaction claims as release evidence.

**Why now / 为什么现在做：** This is the nearest open dependency after the private authorization and retention contracts. It also exposed two genuine PostgreSQL compatibility defects: unsupported JSONB object counting and nanosecond-to-microsecond snapshot replay mismatch. Fixing those defects is necessary to make the same release gate meaningful, not an unrelated refactor.

**Non-goals / 非目标：** No public commit route, operator audit transport, purge executor, OpenAPI operation, SDK method, Web mutation control, or GraphDiff expansion.

**Required evidence / 所需证据：** A loopback-only disposable database, per-test schema reset guarded by an explicit database name and role, all selected ignored PostgreSQL tests passing, CI service configuration, and cross-stack regression checks. Production migration verification remains a separate later gate.

**Outcome and regression proof / 结果与回归证明：** A fresh local PostgreSQL 16.14 disposable container ran all 11 named ignored storage tests with a schema reset before each test, covering legacy identity migration, audit-retention upgrade, graph seed projection, group-RBAC precedence, concurrent replay, stale-head competition, scope constraints, database unavailability, audit persistence, redacted review pagination, and guarded branch replay. The gate exposed two PostgreSQL compatibility defects: `jsonb_object_length` is unavailable and timestamp values must be normalized to PostgreSQL microsecond precision. The minimal fixes use PostgreSQL-supported JSONB object-key counting and normalize `CreateContextCommitSnapshot` capture times before persistence; unit regressions cover both. CI setup now uses PostgreSQL 16.14, an explicit `postgresql-client-16`, a dedicated disposable role, a loopback-only URL guard, an explicit database-name/role check, and a shell regression that proves remote URLs and mismatched roles stop before schema reset. Fresh `cargo fmt --all -- --check`, `cargo test --workspace` (123 passed, 11 explicitly ignored), `pnpm check:web`, shell syntax, and shell guard tests all pass. A successful remote GitHub Actions run and production migration evidence remain open requirements.

**阻塞条件、最小修复与回归证明：** 新鲜的本地 PostgreSQL 16.14 disposable container 在每个测试前重置 schema 后，运行了全部 11 个指定的 ignored storage test，覆盖 legacy identity migration、audit-retention upgrade、graph seed projection、group-RBAC precedence、concurrent replay、stale-head competition、scope constraint、database unavailability、audit persistence、redacted review pagination 与 guarded branch replay。该门禁暴露两个 PostgreSQL 兼容性缺陷：`jsonb_object_length` 不可用，timestamp 必须规范化到 PostgreSQL 的微秒精度。最小修复改为使用 PostgreSQL 支持的 JSONB object-key 计数，并在持久化前规范化 `CreateContextCommitSnapshot` capture time；两者均有 unit regression 覆盖。CI setup 现使用 PostgreSQL 16.14、显式 `postgresql-client-16`、专用 disposable role、仅限 loopback 的 URL guard，以及显式的 database-name/role check；shell regression 证明 remote URL 与不匹配 role 会在 schema reset 前停止。新鲜的 `cargo fmt --all -- --check`、`cargo test --workspace`（123 passed、11 个显式 ignored）、`pnpm check:web`、shell syntax 与 shell guard test 均已通过。一次成功的远端 GitHub Actions 运行与 production migration 证据仍是开放要求。

**Restricted purge executor evidence / 受限 purge executor 证据：** A fresh PostgreSQL 16.14 disposable run now executes all 12 named storage tests with a schema reset before each test. The new role-isolation test proves a dedicated runtime role has no direct audit-table `SELECT` or `DELETE` privilege, while its separate pool can execute the private procedure; cutoff-boundary rows remain, redacted manifest items are retained, and a repeated empty call returns zero. It exposed and repaired the minimum definer privileges needed for `FOR UPDATE SKIP LOCKED` and `INSERT ... RETURNING`. Fresh `cargo fmt --all -- --check`, `cargo test --workspace` (125 passed, 12 explicitly ignored), and `pnpm check:web` pass. Remote CI success and a production-like migration rehearsal remain required before public write readiness can be decided.

**受限 purge executor 证据（中文）：** 新鲜的 PostgreSQL 16.14 disposable run 现已在每个测试前重置 schema 后执行全部 12 个指定 storage test。新的角色隔离测试证明专用 runtime role 没有直接 audit-table `SELECT` 或 `DELETE` 权限，而其独立 pool 可以执行 private procedure；cutoff 边界行会保留，去标识化 manifest item 会被保留，重复的空调用返回零。该测试暴露并修复了 `FOR UPDATE SKIP LOCKED` 与 `INSERT ... RETURNING` 所需的最小 definer 权限。新鲜的 `cargo fmt --all -- --check`、`cargo test --workspace`（125 passed、12 个显式 ignored）和 `pnpm check:web` 均已通过。远端 CI 成功记录与生产相似环境 migration rehearsal 仍然是决定 public write readiness 前的必需条件。

**Production-like migration rehearsal / 生产相似迁移演练：** A fresh PostgreSQL 16.14 disposable run applies the historical `0001–0007` schema, preserves a legacy authorization-audit event through `0008` retention governance and `0009` append-only ledger, then provisions the separate private purge roles/procedure. It rejects ledger mutation and duplicate migration IDs. The guarded CI list now contains 13 individually reset storage tests. This is forward-upgrade evidence in a disposable environment, not a production deployment or rollback proof; remote CI success and an operator-approved production change rehearsal remain open.

**生产相似迁移演练（中文）：** 新鲜的 PostgreSQL 16.14 disposable run 会应用历史 `0001–0007` schema，使 legacy authorization-audit event 前向通过 `0008` retention governance 与 `0009` append-only ledger，然后 provision 独立的 private purge role/procedure。它拒绝 ledger 修改与重复 migration ID。受保护的 CI 清单现在包含 13 个逐例 reset 的 storage test。这是 disposable 环境中的前向升级证据，不是生产部署或回滚证明；远端 CI 成功记录与 operator 批准的生产变更演练仍未完成。

### 2026-07-13: Shared atomic protected-route limiter / 共享原子受保护路由限流器

**Criterion / 条件：** Production security and reliable release gates require a shared atomic
multi-replica quota before public protected writes can be considered.

**Need and boundary / 必要性与边界：** The in-memory limiter is correct only inside one process.
Migration `0010_shared_protected_route_rate_limits.sql` and the private
`PostgresProtectedRouteRateLimiter` preserve the existing `contextlab-auth` port while storing one
deployment policy/clock guard and one bounded issuer/subject/operation queue per state row. The adapter
uses a transaction-scoped advisory lock for configuration agreement, expiry cleanup, capacity checks,
and the row update. It adds no HTTP route, public write, operator transport, OpenAPI operation, SDK
method, Web control, or GraphDiff calculation.

**必要性与边界（中文）：** 内存 limiter 只在单个进程内保持正确。迁移
`0010_shared_protected_route_rate_limits.sql` 与私有的 `PostgresProtectedRouteRateLimiter` 保持既有
`contextlab-auth` port 不变，同时持久化一个 deployment policy/clock guard，并为每个 state row 保存一个
有界 issuer/subject/operation queue。adapter 使用 transaction-scoped advisory lock 保护 configuration
agreement、expiry cleanup、capacity check 与 row update。它不新增 HTTP route、public write、operator
transport、OpenAPI operation、SDK method、Web control 或 GraphDiff calculation。

**Fresh evidence and outcome / 新鲜证据与结果：** A new local PostgreSQL 16.14 container ran the
two-pool, twelve-request integration test with exactly three admissions, plus identity-isolation,
expiry-recovery, configuration-drift, and closed-pool fail-closed tests. The disposable CI script now
contains 15 individually reset named storage tests. Root cause found during verification was an inactive
local Docker daemon, not an adapter fault; starting Docker Desktop restored the already-installed local
test dependency, after which both focused integration tests passed. This is local non-production
evidence only. A remote CI success and an operator-approved production change rehearsal remain open;
the private adapter is not a public-write readiness decision.

**新鲜证据与结果（中文）：** 一个新的本地 PostgreSQL 16.14 container 运行了双 pool、十二次请求的
integration test，恰好放行三次；identity-isolation、expiry-recovery、configuration-drift 与 closed-pool
fail-closed 测试也已通过。disposable CI script 现包含 15 个逐例 reset 的指定 storage test。验证期间发现的
根因是本地 Docker daemon 未启动，而不是 adapter 缺陷；启动 Docker Desktop 后，已安装的本地测试依赖恢复，
两个聚焦 integration test 均通过。这仅是本地非生产证据。远端 CI 成功与 operator 批准的 production change
rehearsal 仍未完成；私有 adapter 不是 public-write readiness 决策。

**Concurrency-review correction / 并发审阅修正：** The initial global-lock design made every active
key wait behind first-key admission. Two new regression tests exposed the bottleneck and a future-state
acceptance flaw. Forward migration `0011` makes the configuration immutable, moves normal decisions to a
deterministic per-key advisory lock, retains the global lock only for first-key expiry/capacity/insertion,
and rejects future or unordered timestamps. Fresh local PostgreSQL tests pass for both regressions and
the original cross-pool quota/expiry cases; the guarded CI list now contains 17 reset-per-test entries.
Policy rotation remains an explicit operator-approved deployment procedure, not an automatic runtime
behavior. Remote CI and operator-approved production evidence remain open.

**并发审阅修正（中文）：** 初始 global-lock 设计让每个 active key 都等待 first-key admission。两项新的
regression test 暴露了该瓶颈与 future-state acceptance flaw。前向迁移 `0011` 使 configuration 不可变，把
正常 decision 移到确定性的 per-key advisory lock，只为 first-key expiry/capacity/insertion 保留 global
lock，并拒绝 future 或无序 timestamp。两个 regression 与原有 cross-pool quota/expiry case 的新鲜本地
PostgreSQL 测试均已通过；受保护 CI 清单现有 17 个 reset-per-test entry。policy rotation 仍是显式的
operator 批准 deployment procedure，不是自动 runtime behavior。远端 CI 与 operator 批准的 production
evidence 仍保持开放。

### 2026-07-14 External Release-Evidence Admission / 2026-07-14 外部发布证据准入

**Criterion / 条件：** This documentation-only increment serves the named `Reliable release gates` and `Production security and collaboration` completion conditions. It is the final dependency-ready evidence collection step before a separate public-write readiness decision may be considered; it is not that decision.

**条件：** 本次仅文档增量服务于已命名的 `Reliable release gates` 与 `Production security and collaboration` 完成条件。它是在单独考虑 public-write readiness 决策前最后一个依赖已满足的证据收集步骤；它本身不是该决策。

**Necessity, gap, and priority / 必要性、缺口与优先级：** Fresh local PostgreSQL 16.14 evidence now covers the 17-test reset-per-test suite, the isolated purge-role contract, the hardened shared limiter, and a disposable production-like forward rehearsal. These local results cannot prove a remote CI execution or an operator-approved production-change rehearsal. No reviewable remote run receipt, approval/change record, environment-isolation proof, or migration-asset manifest exists. The two receipts therefore remain required for public protected-write promotion, release, or production rollout, while dependency-ready private core increments continue through their own Necessity Records.

**必要性、缺口与优先级：** 新鲜的本地 PostgreSQL 16.14 证据现已覆盖 17 项 reset-per-test suite、隔离的 purge-role contract、硬化的 shared limiter 与 disposable 的 production-like forward rehearsal。这些本地结果无法证明远端 CI 执行或 operator 批准的 production-change rehearsal。当前没有可审阅的 remote run receipt、approval/change record、environment-isolation proof 或 migration-asset manifest。因此两份回执仍是 public protected-write promotion、release 或生产推广的必需证据，而依赖就绪的私有核心增量继续通过各自 Necessity Record 准入。

**Boundary and non-goals / 边界与非目标：** The smallest boundary is the bilingual protocol at `docs/roadmap/external-release-evidence-protocol.md` and the status record here. It adds no production connection, SSH tunnel, port-forward, shared-cluster reset, secret access, migration, route, operator transport, OpenAPI/SDK method, Web control, or graph-diff calculator. `GraphDiff` remains the sole graph-diff calculator.

**边界与非目标：** 最小边界是 `docs/roadmap/external-release-evidence-protocol.md` 中的中英双语协议与本处状态记录。它不增加 production connection、SSH tunnel、port-forward、shared-cluster reset、secret access、migration、route、operator transport、OpenAPI/SDK method、Web control 或 graph-diff calculator。`GraphDiff` 仍是唯一的 graph-diff calculator。

**Current receipt state / 当前回执状态：** Remote disposable CI is `unobserved`; operator-approved production-change rehearsal is `unobserved`; local-worktree binding is `unavailable` because usable local Git metadata is absent. No remote CI success, operator approval, production deployment, production migration, rollback proof, or public-write readiness is claimed.

**当前回执状态：** remote disposable CI 为 `unobserved`；operator 批准的 production-change rehearsal 为 `unobserved`；local-worktree binding 为 `unavailable`，因为可用的本地 Git metadata 缺失。本文不声称远端 CI 成功、operator 批准、生产部署、生产 migration、rollback proof 或 public-write readiness。

**Deferred release evidence / 延期发布证据：** `unobserved` receipts are recorded release evidence gaps, not a global development blocker. Do not repeat empty receipt audits or mark the long-term goal blocked solely because they are absent. Re-open receipt review only when new external evidence arrives or a public protected-write, release, or production-promotion decision is being prepared. This deferral never weakens the receipt fields or permits local evidence to impersonate external proof.

`unobserved` 回执是已记录的发布证据缺口，不是全局开发阻塞条件。不得因为回执缺失而重复空转审计，也不得仅据此把长期目标标记为 blocked。只有在出现新的外部证据，或准备 public protected-write、release、production promotion 决策时，才重新审阅回执。该延期绝不弱化回执字段，也不允许本地证据冒充外部证明。

**VCS-binding availability audit / VCS 归属可用性审计：** A fresh read-only audit found that the local `.git` path is an empty directory with no `HEAD`; `git rev-parse`, object verification, and status queries all report that this is not a Git repository. No `.git/config`, `.env`, credential, or network source was read. Therefore the local worktree cannot currently establish any commit identity, and creating or inferring a new repository would not be a valid repair. The smallest safe recovery source is a future remote CI receipt that retains both its immutable run URL/ID and the checked-out `git rev-parse --verify HEAD^{commit}` output, with an independently reviewed equality check against provider `head_sha`. That adds a concrete `exact_commit` proof without claiming that this local worktree is bound to it. The regression proof is a fresh documentation check for that required equality field, the still-`unobserved` status, and the absence of runtime/public-surface references; it is not an external receipt.

**VCS 归属可用性审计：** 一次新鲜的只读审计发现，本地 `.git` 路径是没有 `HEAD` 的空目录；`git rev-parse`、对象校验和状态查询都报告它不是 Git repository。没有读取 `.git/config`、`.env`、credential 或 network source。因此，本地工作树当前无法建立任何 commit identity，创建或猜测一个新 repository 也不是有效修复。最小安全恢复来源是未来 remote CI receipt 同时保留其不可变 run URL/ID 与 checked-out `git rev-parse --verify HEAD^{commit}` 输出，并独立审阅它与 provider `head_sha` 的相等性。这会提供具体的 `exact_commit` proof，但不会声称本地工作树已归属到它。回归证明是对该相等字段、仍为 `unobserved` 的状态以及没有 runtime/public-surface reference 的新鲜文档检查；它不是外部回执。

**CI evidence-capture repair / CI 证据捕获修复：** The active remote-CI gate exposed a second root cause: `verify.yml` ran all checks but retained no checkout provenance or workflow fingerprint, so a green remote run could not supply the mandatory evidence input. The minimum repair adds a Bash static contract test, a success-only capture after `pnpm check:web`, and a 90-day `contextlab-ci-evidence` artifact containing only the checkout/provider-SHA equality, workflow SHA-256, UTC capture time, safe scope text, redaction statement, and the capture-file SHA-256. The static test rejects reordering, missing upload semantics, and database/credential-bearing strings in the capture block. Fresh local tests prove the workflow shape only; a remote artifact plus independent review remains required before its state can become `observed_pass`.

**CI 证据捕获修复：** 活跃的 remote-CI 门禁暴露第二个根因：`verify.yml` 虽会运行全部检查，却没有保留 checkout provenance 或 workflow fingerprint，因此一次 green remote run 不能提供强制要求的 evidence input。最小修复增加 Bash static contract test、位于 `pnpm check:web` 之后的 success-only capture，以及保留 90 天的 `contextlab-ci-evidence` artifact；它只包含 checkout/provider-SHA equality、workflow SHA-256、UTC capture time、safe scope text、redaction statement 和 capture-file SHA-256。static test 会拒绝错误排序、缺少 upload 语义，以及 capture block 中包含 database/credential 的字符串。新鲜本地测试只证明 workflow shape；在状态可成为 `observed_pass` 前，仍需 remote artifact 与独立审阅。

**Rehearsal asset-manifest repair / 演练资产清单修复：** The operator-rehearsal gate exposed a third root cause: the append-only ledger and CI provenance artifact did not preserve real ID/SHA-256 entries for the regular migrations and separately ordered privileged SQL files exercised by the production-like path. The minimum repair adds a pure Bash writer, a dynamic manifest test that derives every expected checked-in asset digest, a static CI layout guard, and a success-only 90-day `contextlab-rehearsal-evidence` artifact. The capture reads no environment values and never connects to a database. It is limited to technical asset integrity; approval, isolated target identity, execution roles, cleanup, independent review, and final evidence state remain external requirements. Fresh local tests prove the manifest and workflow shape only; the operator-approved receipt remains `unobserved`.

**演练资产清单修复：** operator-rehearsal 门禁暴露第三个根因：append-only ledger 与 CI provenance artifact 没有保留 production-like path 所用 regular migration 和单独有序 privileged SQL file 的真实 ID/SHA-256 条目。最小修复增加纯 Bash writer、从每个当前已检入 asset digest 推导期望值的动态 manifest test、static CI layout guard，以及仅限成功路径、保留 90 天的 `contextlab-rehearsal-evidence` artifact。该 capture 不读取 environment value，也绝不连接数据库。它只限于技术 asset integrity；审批、隔离 target identity、execution role、cleanup、独立审阅与最终 evidence state 仍是外部要求。新鲜本地测试只证明 manifest 与 workflow shape；operator 批准的 receipt 仍为 `unobserved`。

**Gate root cause, minimum repair, and regression proof / 门禁根因、最小修复与回归证明：** Review found that a loopback URL in the disposable rehearsal script cannot rule out an SSH tunnel or incorrectly mapped shared target, and that the current ledger check proves append-only behavior but does not attest real migration asset execution or digests. The minimum repair for this documentation increment is to reject the script and ledger alone as operator evidence, require independent non-production isolation proof and a full migration-asset manifest, and record only redacted receipts. The regression proof is the new protocol's required fields, prohibitions, binding states, and the fresh local documentation checks; an operator rehearsal is not claimed.

**门禁根因、最小修复与回归证明：** 审阅发现 disposable rehearsal script 中的 loopback URL 无法排除 SSH tunnel 或错误映射的 shared target，当前 ledger check 只证明 append-only 行为，不能证明真实 migration asset 的执行或 digest。本次文档增量的最小修复是拒绝将该 script 与 ledger 单独视为 operator evidence，要求独立的 non-production isolation proof 与完整 migration-asset manifest，并且只记录脱敏回执。回归证明是新协议中的必填字段、禁止项、binding state 与新鲜本地文档检查；本文不声称已有 operator rehearsal。

**Admission decision / 准入决策：** The remote receipt must be `observed_pass` with `remote_snapshot_binding` equal to `exact_commit`, and the operator rehearsal receipt must be independently reviewable `observed_pass` evidence before a new Necessity Record may decide whether public-write readiness is worth assessing. `provider_only` never satisfies the remote release gate. Any `unobserved`, `observed_fail`, or `inconclusive` state keeps the gate open. The long-term goal remains open regardless of this documentation increment.

**准入决策：** 只有 remote receipt 为 `observed_pass` 且 `remote_snapshot_binding` 等于 `exact_commit`，并且 operator rehearsal receipt 成为可独立审阅的 `observed_pass` 证据后，新的 Necessity Record 才能决定是否值得评估 public-write readiness。`provider_only` 永远不能满足远端 release gate。任何 `unobserved`、`observed_fail` 或 `inconclusive` 状态都会使门禁保持打开。无论本次文档增量如何，长期目标始终保持开放。

## Goal Closure / 目标关闭

No priority update, completed increment, or green test may close the long-term goal. The goal closes only when every item in `completion-criteria.md` has fresh evidence, including Context-first coverage, versioning and all required diffs, benchmark-driven evaluation, graph editing/diffing, design-system-first interfaces, production security/collaboration, extensibility, release gates, and bilingual documentation.

任何优先级调整、已完成增量或测试通过都不能关闭长期目标。只有 `completion-criteria.md` 中的全部项目都获得新鲜证据后，目标才可关闭，包括 Context-first 覆盖、版本与全部必需 Diff、benchmark 驱动评测、图谱编辑/比较、设计系统优先界面、生产安全/协作、可扩展性、发布门禁和中英双语文档。

## Superseding local VCS status / 覆盖后的本地 VCS 状态

The earlier VCS paragraph remains historical evidence for the environment at that time. A fresh
read-only check on 2026-07-30 still could not resolve `HEAD^{commit}`; `git rev-parse` failed,
while `git status` enumerated the current worktree with 595 entries, mostly untracked. Therefore
Git metadata is available enough to report a worktree status but not enough to establish a complete,
reviewable change-set or exact commit binding. This remains `unobserved` and is not a blocker for
local product development. No `.env`, credential, or network source was read.

此前 VCS 段落保留为当时环境的历史证据。2026-07-30 新鲜只读检查仍无法解析 `HEAD^{commit}`；`git rev-parse` 失败，而 `git status`
列出当前工作树 595 项，其中大部分为未跟踪文件。因此 Git metadata 足以报告工作树状态，但不足以建立完整可审阅的 change-set 或 exact
commit binding。该证据继续标记为 `unobserved`，不阻断本地产品开发。没有读取 `.env`、credential 或 network source。
