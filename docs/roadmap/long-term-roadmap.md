# ContextLab Long-Term Roadmap / 长期路线图

## North Star / 北极星

ContextLab becomes the reference open-source platform for Context Engineering: a place to design, version, diff, evaluate, visualize, replay, and collaborate on the complete context behind AI systems.

ContextLab 成为 Context Engineering 的代表性开源平台：用于设计、版本化、Diff、评测、可视化、回放和协作管理 AI 系统背后的完整 Context。

## Active Long-Term Objective / 当前长期目标

ContextLab will be continuously developed into a bilingual, open-source Context Engineering platform for designing, versioning, comparing, evaluating, and collaborating on production AI workflows. It will treat Context, rather than prompts alone, as the primary abstraction and will support reusable Rust domain cores, stable REST and SDK contracts, persistent storage, and a design-system-first interface.

ContextLab 将持续建设为中英双语的开源 Context Engineering 平台，用于设计、版本化、比较、评测和协作管理生产级 AI workflow。项目以 Context 而不是单一 prompt 为核心抽象，并以可复用的 Rust 领域核心、稳定的 REST 与 SDK 契约、持久化存储和设计系统优先的界面为基础。

The objective stays open until every convergence condition in `docs/roadmap/completion-criteria.md` is freshly verified. A page, API route, SDK method, test suite, or feature slice is evidence of progress, never evidence that the repository is complete. Each delivery cycle must record its scope, preserve the architecture boundaries, provide proportionate automated verification, and update bilingual documentation before the next cycle begins.

在 `docs/roadmap/completion-criteria.md` 的全部收束条件经过新鲜验证之前，本目标始终保持开放。任何页面、API 路由、SDK 方法、测试套件或功能切片都只能证明进展，不能证明仓库完成。每个交付周期都必须记录范围，保持架构边界，提供与风险相称的自动化验证，并在进入下一周期前更新中英双语文档。

Completion is governed by `docs/roadmap/completion-criteria.md`, which keeps the current audit and missing platform areas explicit; unavailable external deployment prerequisites are deferred outside the active local delivery scope.

项目完成度由 `docs/roadmap/completion-criteria.md` 约束，该文档显式维护当前审计与缺失平台能力；不可用的外部部署前置已在活跃本地交付范围之外延期。

The active long-term goal remains open until those convergence conditions are met. The current Web graph/scorecard work, public GraphDiff GET/POST API/SDK contracts, storage-level atomic commit snapshot capture, and explicitly opt-in protected commit route count as bounded slices, not project completion. Private PostgreSQL/HMAC and PostgreSQL/OIDC profiles authenticate before a bounded in-process sliding-window limiter keyed only by authenticated issuer-scoped `PrincipalIdentity` and `ProtectedRouteOperation::ContextCommitWrite`; the same source-and-subject pair namespaces membership, audit, and idempotency records, and limiter rejection cannot reach authorization audit or the writer. Migration `0006` retains existing rows in the explicit `legacy` namespace for operator reconciliation. Quota rejection is `429 rate_limit_exceeded` with integer `Retry-After`, while capacity or internal failure closes with `503 rate_limit_unavailable`. The trusted membership/group-to-RBAC slice is now implemented as a private boundary: verified bounded OIDC groups resolve only workspace-scoped `reader` or `editor` grants, direct membership overrides those grants, and PostgreSQL repeats the decision under `FOR UPDATE`. Migrations `0008–0015` now provide private audit-retention storage, a separately proven isolated purge executor, a hardened shared PostgreSQL limiter, commit-bound component-content creation/revisions, initial-revision integrity enforcement, and durable same-Context commit-parent links. The disposable selection contains 25 reset-per-test cases; fresh local non-production evidence covers creation, replay, projection overlay, malformed nullable-prior rejection, full normal first-parent body replay, and fail-fast parent-scope migration integrity. Remote CI, operator rehearsal, public protected-write promotion, release, and production rollout are explicitly deferred future deployment work outside the active local scope. Public OpenAPI/SDK and `GraphDiff` remain unchanged.

当前长期目标必须持续保持打开，直到这些收束条件全部满足。当前 Web graph/scorecard 工作、public GraphDiff GET/POST API/SDK contract、storage-level atomic commit snapshot capture 与显式 opt-in protected commit route 都只算边界明确的 slice，不代表项目完成。private PostgreSQL/HMAC 与 PostgreSQL/OIDC profile 会先 authentication，再执行仅以认证后、按 issuer 分区的 `PrincipalIdentity` 与 `ProtectedRouteOperation::ContextCommitWrite` 为 key 的有界 in-process sliding-window limiter；同一 source-and-subject 组合会为 membership、audit 与 idempotency record 分区，limiter rejection 不会进入 authorization audit 或 writer。迁移 `0006` 会把既有行保留在显式 `legacy` 命名空间中，等待 operator 对齐。quota rejection 返回带整数 `Retry-After` 的 `429 rate_limit_exceeded`，容量或内部故障则 fail closed 为 `503 rate_limit_unavailable`。可信 membership/group-to-RBAC slice 现已作为 private boundary 实现：经过验证且有界的 OIDC group 只能解析为 workspace 范围的 `reader` 或 `editor` grant，direct membership 覆盖这些 grant，PostgreSQL 会在 `FOR UPDATE` 下重复该决策。迁移 `0008–0015` 现已提供私有 audit-retention storage、单独证明的隔离 purge executor、硬化的共享 PostgreSQL limiter、绑定 commit 的 component-content creation/revision、initial-revision integrity enforcement 与持久化的同 Context commit-parent link。disposable 选择包含 25 个逐例 reset case；新鲜的本地 non-production 证据覆盖 creation、replay、projection overlay、malformed nullable-prior rejection、完整 normal first-parent body replay，以及 fail-fast 的 parent-scope migration integrity。远端 CI、operator 演练、public protected-write promotion、release 与生产推广均明确延期为活跃本地范围外的未来部署工作。public OpenAPI/SDK 与 `GraphDiff` 保持不变。

The completed group increment accepts one explicitly configured, bounded OIDC group claim only after full token validation. It stores opaque group identifiers in `workspace_external_group_role_bindings`, scoped by exact issuer and workspace, resolves them through `contextlab-auth`, keeps direct membership as the override, and permits only `reader` or `editor` group grants. It adds no public mutation route, OpenAPI operation, SDK method, or GraphDiff behavior. Its delivery record is `docs/superpowers/plans/2026-07-12-trusted-identity-group-rbac.md`.

### 2026-07-13 Private Shared Rate-Limit Evidence / 2026-07-13 私有共享限流证据

Migration `0010_shared_protected_route_rate_limits.sql` and
`PostgresProtectedRouteRateLimiter` now provide a private storage adapter for the existing auth port.
One transaction-scoped advisory lock guards policy agreement, database-clock monotonicity, expiry
reclamation, active-key capacity, and the exact issuer/subject/operation timestamp queue across
independent pools. Fresh local PostgreSQL 16.14 evidence proves that twelve concurrent checks through
two pools admit only three permits and that isolation, expiry recovery, policy drift, and a closed pool
fail closed. It adds no route, OpenAPI, SDK, Web, or GraphDiff behavior. The earlier roadmap snapshot's
 shared-limiter prerequisite is therefore locally satisfied. Remote CI, operator rehearsal, and public-write
 promotion are unavailable future deployment work and are not tracked by the active local roadmap.

迁移 `0010_shared_protected_route_rate_limits.sql` 与
`PostgresProtectedRouteRateLimiter` 现已为既有 auth port 提供私有 storage adapter。一个
transaction-scoped advisory lock 会在独立 pool 之间保护 policy agreement、database-clock
monotonicity、expiry reclamation、active-key capacity 以及精确 issuer/subject/operation timestamp queue。
新鲜本地 PostgreSQL 16.14 证据证明两个 pool 的十二次并发检查只放行三次，并且 isolation、expiry
recovery、policy drift 与 closed pool 都会 fail closed。它不新增 route、OpenAPI、SDK、Web 或 GraphDiff
 behavior。因此，较早路线图快照中的 shared-limiter prerequisite 已在本地满足。远端 CI、operator
 演练与 public-write promotion 属于外部条件不可用的未来部署工作，不由当前本地路线图追踪。

Migration `0011_shared_protected_route_rate_limit_hardening.sql` is part of that local evidence. It
keeps the private policy immutable, gives independent existing keys separate advisory locks, reserves the
global admission lock for a new key only, and fails closed for future or unordered timestamps. The
 17-test disposable suite and the local production-like forward rehearsal are historical local evidence.
 They make no remote, operator-approved, release, or production claim.

迁移 `0011_shared_protected_route_rate_limit_hardening.sql` 是该本地证据的一部分。它保持私有 policy
不可变，为独立 existing key 分配独立 advisory lock，只为 new key 保留 global admission lock，并对 future
或无序 timestamp fail closed。17 项 disposable suite 与 production-like forward rehearsal 覆盖了这项
 hardening；它们仅是历史本地证据，不声称远端、operator 批准、release 或 production 事实。

已完成的 group 增量只会在完整 token 校验后接受一个显式配置且有界的 OIDC group claim。它会将不透明 group identifier 按精确 issuer 与 workspace 范围存入 `workspace_external_group_role_bindings`，再交由 `contextlab-auth` 解析；direct membership 仍是覆盖规则，group grant 只允许 `reader` 或 `editor`。它不会新增 public mutation route、OpenAPI operation、SDK method 或 GraphDiff behavior。其交付记录位于 `docs/superpowers/plans/2026-07-12-trusted-identity-group-rbac.md`。

### 2026-07-15 Private Component-Content Replay Resolution / 2026-07-15 私有 Component 正文回放解析

The private component-content creation slice now has fresh local evidence for its atomic creation, immutable initial revision, projection consistency, and `0013 -> 0014` PostgreSQL integrity path. The next private storage contract is `get_component_content_at_commit(context, commit, component)`: it walks only the normal first-parent ancestry and returns the nearest immutable revision, preserving the revision's source commit identity. It returns no body when no captured revision is reachable and rejects merge or cyclic ancestry instead of guessing. A focused memory test and a fresh disposable PostgreSQL case cover unchanged descendants, revised descendants, unknown commits, no-body components, and merge boundaries. This remains storage-only; public body reads/writes, graph editing, merge policy, OpenAPI/SDK/Web changes, operator transport, release promotion, and `GraphDiff` remain out of scope.

私有 component-content creation slice 现已获得原子创建、不可变初始 revision、projection consistency 与 `0013 -> 0014` PostgreSQL integrity path 的新鲜本地 evidence。下一条私有 storage contract 是 `get_component_content_at_commit(context, commit, component)`：它只沿 normal first-parent ancestry 回放，并返回最近的不可变 revision，同时保留 revision 的 source commit identity。没有可达 captured revision 时返回 no body；遇到 merge 或 cycle ancestry 时显式拒绝，不进行猜测。聚焦内存 test 与新鲜 disposable PostgreSQL case 覆盖 unchanged descendant、revised descendant、unknown commit、无正文 component 与 merge boundary。本增量仍仅限 storage；public body read/write、graph editing、merge policy、OpenAPI/SDK/Web change、operator transport、release promotion 与 `GraphDiff` 均不在范围内。

## Phase 1: Foundation / 第一阶段：地基

- Establish the modular monorepo.
- 建立模块化 monorepo。
- Implement `context-core`, `versioning`, `diff-engine`, and `evaluation` as tested Rust crates.
- 实现可测试的 `context-core`、`versioning`、`diff-engine` 和 `evaluation` Rust crates。
- Provide a minimal REST API shell with health and metadata routes.
- 提供最小 REST API 外壳，包括健康检查与元信息路由。
- Add the first explicit Context Graph domain model and preview API.
- 增加第一版显式 Context Graph 领域模型与预览 API。
- Start the design system token package and the web workspace shell.
- 启动设计系统 token package 与 Web workspace shell。
- Keep project documentation bilingual.
- 保持项目文档中英双语同步。

## Phase 2: Context Platform / 第二阶段：Context 平台

- Persist workspaces, projects, experiments, contexts, components, and commits in PostgreSQL.
- 在 PostgreSQL 中持久化 workspace、project、experiment、context、component 和 commit。
- Add SQLx migrations, repositories, and API pagination/filtering/sorting.
- 添加 SQLx migration、repository，以及 API 分页、过滤、排序。
- Keep list/query semantics identical across in-memory preview repositories and PostgreSQL repositories, including slugs, timestamps, pagination, filtering, and sorting.
- 保持 in-memory preview repository 与 PostgreSQL repository 的 list/query 语义一致，包括 slug、timestamp、分页、过滤和排序。
- Add workspace-scoped project discovery before expanding the same list/query pattern to experiments, contexts, commits, and evaluation records.
- 在扩展到 experiment、context、commit 和 evaluation record 之前，先增加 workspace-scoped project discovery。
- Add project-scoped experiment discovery with branch-name visibility before connecting experiment branching to version commits and evaluation runs.
- 在把 experiment branching 接入 version commit 与 evaluation run 之前，先增加带 branch-name 可见性的 project-scoped experiment discovery。
- Add project-scoped Context discovery with optional experiment filtering before context editing, versioning, diff, and evaluation workflows.
- 在 context editing、versioning、diff 与 evaluation workflow 之前，先增加带可选 experiment 过滤的 project-scoped Context discovery。
- Add context-scoped commit history discovery before write, merge, rollback, replay, and semantic diff workflows.
- 在 write、merge、rollback、replay 与 semantic diff workflow 之前，先增加 context-scoped commit history discovery。
- Add context-scoped component discovery with reproducible content fingerprints before prompt, memory, knowledge, schema, tool, and model configuration editing workflows.
- 在 prompt、memory、knowledge、schema、tool 与 model configuration 编辑流程之前，先增加带可复现 content fingerprint 的 context-scoped component discovery。
- Component detail reads expose metadata, fingerprints, and timestamps only. Private guarded storage now records one immutable UTF-8 component-body revision per commit with replayable prior/resulting hashes; no body is added to public REST/SDK/Web reads, and historical components without a captured revision remain unavailable.
- Component detail read 只暴露 metadata、fingerprint 与 timestamp。私有 guarded storage 现已为每个 commit 记录一条不可变 UTF-8 component-body revision，并保存可回放的前后 hash；public REST/SDK/Web read 不新增正文，未捕获 revision 的历史 component 仍不可用。
- Add context-scoped evaluation run discovery before scorecards, dashboards, regression detection, and evaluation diff workflows.
- 在 scorecard、dashboard、regression detection 与 evaluation diff workflow 之前，先增加 context-scoped evaluation run discovery。
- Add evaluation run detail reads with persisted metrics JSON before scorecards, dashboards, regression detection, and evaluation diff workflows.
- 在 scorecard、dashboard、regression detection 与 evaluation diff workflow 之前，先增加带持久化 metrics JSON 的 evaluation run detail read。
- Mirror Context, workspace graph, commit, component discovery/detail, evaluation run discovery/detail, and selected scorecard aggregate contracts in the Web workspace shell with API-shaped preview data and the live SDK/fetch adapter before regression dashboard workflows.
- 在 regression dashboard workflow 之前，通过 API-shaped preview data 与 live SDK/fetch adapter，在 Web workspace shell 中对齐 Context、workspace graph、commit、component discovery/detail、evaluation run discovery/detail 与 selected scorecard aggregate contract。
- Add the first TypeScript SDK boundary for REST discovery DTOs, query types, route construction, and Web runtime live-data fallback before moving to generated OpenAPI SDKs.
- 在转向 generated OpenAPI SDK 之前，先增加第一版 TypeScript SDK boundary，用于 REST discovery DTO、query type、route construction，以及 Web runtime live-data fallback。
- Add a checked-in OpenAPI contract for the initial REST read surface and make SDK tests verify discovery operation coverage before introducing generated SDKs.
- 在引入 generated SDK 之前，先为初始 REST read surface 增加 checked-in OpenAPI contract，并让 SDK tests 验证 discovery operation coverage。
- Serve the OpenAPI contract at runtime and expand the TypeScript SDK to cover platform, provider, OpenAPI, and Context Graph read APIs.
- 在运行时提供 OpenAPI contract，并将 TypeScript SDK 扩展到 platform、provider、OpenAPI 与 Context Graph read API。
- Add API-side public GET/POST route catalogs and a Rust OpenAPI drift guard so Axum routes, checked-in contracts, and SDK coverage evolve together.
- 增加 API 侧 public GET/POST route catalog 与 Rust OpenAPI drift guard，使 Axum routes、checked-in contracts 与 SDK coverage 一起演进。
- Add PostgreSQL schema assets for workspace, project, experiment, context, component, commit, and evaluation records.
- 增加 workspace、project、experiment、context、component、commit 和 evaluation 记录的 PostgreSQL schema 资产。
- Wire the SQLx-backed graph repository into runtime configuration and keep the same storage trait contract for tests and local preview.
- 将 SQLx-backed graph repository 接入运行时配置，同时为测试和本地 preview 保持同一个 storage trait 契约。
- Introduce workspace-scoped Context Graph query APIs.
- 引入 workspace-scoped Context Graph 查询 API。
- Maintain the named SQLx seed and storage integration suite in disposable PostgreSQL CI, then add list/query APIs with pagination/filtering/sorting only when they serve an unmet completion condition.
- 持续维护 disposable PostgreSQL CI 中指定的 SQLx seed 与存储集成套件；只有当其服务于未满足的完成条件时，才增加带分页、过滤、排序的 list/query API。
- Add prompt and schema editing workflows through the design system UI.
- 通过设计系统 UI 提供 Prompt 与 Schema 编辑流程。

## Phase 3: Evaluation and Diff / 第三阶段：评测与 Diff

- Establish deterministic GraphDiff domain primitives and a pure API/SDK contract for supplied Context Graph snapshots, then connect graph diffs to version history, storage snapshots, and a read-only Web review workflow.
- 为给定的 Context Graph 快照建立确定性的 GraphDiff 领域原语与纯 API/SDK contract，并将 graph diff 接入版本历史、存储快照和只读 Web 审查 workflow。
- Add benchmark datasets, benchmark suites, regression checks, and A/B experiments.
- 增加 benchmark dataset、benchmark suite、回归检查和 A/B experiment。
- Expand text diff into semantic, behavior, and evaluation diff.
- 将文本 diff 扩展为语义 diff、行为 diff 和评测 diff。
- Store latency, cost, token, quality, tool usage, hallucination, and success metrics.
- 存储 latency、cost、token、quality、tool usage、hallucination 和 success 指标。

## Phase 4: Workflow, Knowledge, and Memory / 第四阶段：Workflow、Knowledge 与 Memory

- Add workflow execution, scheduling, and visual graph editing.
- 增加 workflow 执行、调度和可视化图编辑。
- Add knowledge ingestion, chunking, embedding, retrieval, and citations.
- 增加 knowledge ingestion、chunk、embedding、retrieval 和 citation。
- Add memory timeline, replay, importance, and retention policies.
- 增加 memory timeline、replay、importance 和 retention policy。

## Phase 5: Ecosystem / 第五阶段：生态

- Add plugin loading for models, tools, MCP servers, evaluators, importers, exporters, renderers, authentication, and storage.
- 增加针对 model、tool、MCP server、evaluator、importer、exporter、renderer、authentication 和 storage 的插件加载能力。
- Publish SDKs and contribution guides.
- 发布 SDK 与贡献指南。
- Build marketplace and community extension workflows.
- 建设 marketplace 与社区扩展工作流。
