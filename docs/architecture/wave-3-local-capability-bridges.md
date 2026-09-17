# Admitted 2026-07-19 Local Capability Bridge Wave / 2026-07-19 已准入的本地能力桥接波次

> **Admission status / 准入状态：** This document admits a bounded local-read integration order. Knowledge/Memory and Plugin/MCP remain at accepted Core projections or fixture inspection only.
>
> Workflow's fail-closed availability read and commit-scoped source-binding read are composed privately through protected API routes, the non-public local SDK, same-origin BFF routes, and Web inspectors. Sealed Benchmark decision discovery likewise has a later private, exact project/Context/commit read transport. No bridge is registered for provider execution, public release, or production promotion.
>
> **准入状态：** 本文准入一个有界的本地只读集成顺序。Knowledge/Memory 与 Plugin/MCP 仍只有已验收的 Core projection 或 fixture inspection。
>
> Workflow 的 fail-closed availability read 与 commit-scoped source-binding read 已通过 protected API route、非公开 local SDK、同源 BFF route 与 Web inspector 私有组合。sealed Benchmark decision discovery 也已有后续补充的、精确限定 project/Context/commit 的私有读取 transport。没有 bridge 为 provider execution、public release 或 production promotion 注册。

## Purpose and Scope / 目的与范围

The Local Capability Bridge Wave connects already-owned, provider-free capability projections to inspectable local-read adapters only after their domain contracts are accepted. It is a dependency ledger for Benchmark, Workflow, Knowledge/Memory, and Plugin/MCP capability inspection. Domain policy remains in Rust; API, local SDK, CLI, Desktop, and Web adapt accepted results and never reconstruct policy.

本地能力桥接波次仅在领域契约被接受后，将已归属、无 provider 依赖的能力投影连接至可检查的本地只读 adapter。它是 Benchmark、Workflow、Knowledge/Memory 和 Plugin/MCP 能力检查的依赖台账。领域 policy 保留在 Rust 中；API、local SDK、CLI、Desktop 与 Web 只适配已接受的结果，绝不重建 policy。

This wave does not add or alter public REST, OpenAPI, public SDK, public writes, provider invocation, dynamic plugin registration, browser E2E, Docker or PostgreSQL runtime, remote CI, release, or production wiring. It does not add another graph-diff calculator: `GraphDiff::between` remains the sole graph-diff calculator.

本波次不新增或修改 public REST、OpenAPI、public SDK、public write、provider 调用、动态 plugin 注册、browser E2E、Docker 或 PostgreSQL runtime、remote CI、release 或 production 接线。它不增加第二个 graph-diff calculator：`GraphDiff::between` 仍是唯一的 graph-diff calculator。

## Exclusive Ownership / 独占所有权

| Layer / 层 | Exclusive owner and paths / 独占所有者与路径 | Responsibility / 职责 | Explicitly not owned / 明确不归属 |
| --- | --- | --- | --- |
| Benchmark core / Benchmark 核心 | Evaluation owner: `crates/evaluation/**` | Creates deterministic, redacted workspace projections from sealed evaluation evidence; preserves upstream stable identifiers and comparison semantics. / 从 sealed evaluation evidence 创建确定性、脱敏的 workspace projection；保留上游稳定标识与比较语义。 | Transport, Web policy, public API/SDK changes, and a second graph-diff calculation. / transport、Web policy、public API/SDK 变更及第二个 graph-diff calculation。 |
| Workflow core / 工作流核心 | Workflow owner: `crates/workflow/**` | Owns versioned capability status, replay eligibility, deterministic ordering, and structured unavailable or incompatible outcomes. / 拥有带版本的 capability status、replay eligibility、确定性排序，以及结构化的 unavailable 或 incompatible outcome。 | API routes, local-SDK transport, and React-side workflow inference. / API route、local-SDK transport 及 React 侧 workflow 推断。 |
| Knowledge and Memory core / 知识与记忆核心 | Knowledge/Memory owner: `crates/knowledge/**`, `crates/memory/**` | Owns provider-free citation and retention capability projections; exposes frozen redacted metadata only. / 拥有无 provider 的 citation 与 retention capability projection；只暴露冻结的脱敏 metadata。 | Raw knowledge, memory content, credentials, provider calls, API routes, and Web-owned retention policy. / 原始 knowledge、memory 内容、credential、provider 调用、API route 及 Web 自有 retention policy。 |
| Plugin and MCP core / Plugin 与 MCP 核心 | Plugin/MCP owner: `crates/mcp/**`, `crates/plugin-runtime/**` | Owns manifest, registry compatibility, capability lifecycle, deterministic load order, and fail-closed activation results. / 拥有 manifest、registry compatibility、capability lifecycle、确定性加载顺序及 fail-closed activation result。 | Dynamic public registration, Web capability calculation, and unrelated fallback providers. / 动态 public 注册、Web capability 计算及无关 fallback provider。 |
| Local API composition / 本地 API 组合 | Integration Lead: `server/api/**`, `packages/local-sdk/**` | After domain DTO review, composes private authenticated local reads, validates scope and schema, preserves structured failures, and forwards only redacted safe DTOs. / 在领域 DTO 审查后，组合私有且经认证的本地读取，验证 scope 与 schema，保留结构化失败，并只转发脱敏安全 DTO。 | Root manifests, public REST/OpenAPI/public SDK, public write, domain policy, and client-created identifiers. / root manifest、public REST/OpenAPI/public SDK、public write、领域 policy 及由 client 创建的标识。 |
| Web inspection / Web 检查 | Web owner: `apps/web/src/app/**` | Adapts accepted local DTOs as `data -> presenter -> screen`, keeps the five shared UI states, and renders deterministic bilingual safe text. / 将已接受的 local DTO 适配为 `data -> presenter -> screen`，保持五种共享 UI 状态，并渲染确定性的双语安全文本。 | Rust business logic, API/local-SDK contract invention, raw-content storage, transport fallback, and capability/diff policy calculation. / Rust 业务逻辑、API/local-SDK 契约发明、原始内容存储、transport fallback 及 capability/diff policy 计算。 |
| Existing staging shells / 既有 staging shell | CLI/Desktop owners: `apps/cli/**`, `apps/desktop/**` | Preserve the shared availability projection without duplicating registration or availability policy. / 保留共享 availability projection，不复制 registration 或 availability policy。 | A second core implementation or evidence of a registered shared integration. / 第二份核心实现或已注册共享 integration 的证据。 |

## Shared V1 Contract and Safe Projection / 共享 V1 契约与安全投影

`contextlab.local-capability-availability.v1` is the existing strict Rust availability wire schema used to report an unavailable shared integration. It uses stable operation and integration identifiers and a machine-readable reason. A consumer must fail closed when its literal schema version, operation identifier, integration mapping, availability state, or reason is unsupported or inconsistent.

`contextlab.local-capability-availability.v1` 是既有的严格 Rust availability wire schema，用于报告不可用的共享 integration。它使用稳定的 operation 与 integration 标识，以及机器可读的 reason。当字面 schema version、operation identifier、integration mapping、availability state 或 reason 不受支持或不一致时，consumer 必须 fail closed。

The string-schema Rust wire DTO is not interchangeable with the Web presentation resource whose `schema_version` is numeric `1`. The Web resource remains local presentation state; a future transport shape requires a separately reviewed, explicit V1 contract and focused parser evidence.

字符串 schema 的 Rust wire DTO 不能与 `schema_version` 为数值 `1` 的 Web presentation resource 互换。Web resource 仍是本地展示状态；未来 transport shape 必须拥有单独审查的显式 V1 契约与聚焦 parser 证据。

Every admitted bridge projection must preserve opaque, stable UUID/string identifiers from its accepted domain DTO. It must not derive replacement IDs from titles, payloads, hashes, list positions, or UI state. Collections expose deterministic ordering defined by the owning Rust domain contract, never incidental map iteration or client-side re-sorting that changes the domain result.

每个已准入 bridge projection 必须保留其已接受领域 DTO 中不透明且稳定的 UUID/string identifier。不得从标题、payload、hash、列表位置或 UI 状态派生替代 ID。集合必须暴露由所属 Rust domain contract 定义的 deterministic ordering，绝不能依赖偶然的 map iteration 或会改变领域结果的 client-side re-sorting。

Safe DTOs contain only stable identifiers, version markers, state, redacted summaries, and the minimal capability metadata needed for inspection. They must exclude raw benchmark cases, inputs, oracles, knowledge/memory content, provider credentials, access tokens, unredacted logs, and mutable operational secrets.

安全 DTO 只包含稳定 identifier、version marker、state、脱敏 summary 与检查所需的最小 capability metadata。它们必须排除原始 benchmark case、input、oracle、knowledge/memory content、provider credential、access token、未脱敏日志及可变 operational secret。

## Fail-Closed Error Boundary / Fail-closed 错误边界

The Core owner returns a typed domain outcome; the Integration Lead maps only accepted outcomes to a structured local error with a stable machine-readable code and a redacted bilingual safe summary. Missing integration, unknown V1 schema, invalid stable identifier, scope mismatch, incompatible capability, and unavailable repository state all stop the read. No adapter retries through another provider, reconstructs a result, exposes a partial raw payload, or changes the caller's scope.

Core owner 返回类型化的 domain outcome；Integration Lead 只将已接受 outcome 映射为带稳定 machine-readable code 与脱敏双语 safe summary 的结构化 local error。缺失 integration、未知 V1 schema、无效稳定 identifier、scope mismatch、incompatible capability 与 unavailable repository state 都会停止读取。任何 adapter 都不得通过其他 provider 重试、重建结果、暴露部分原始 payload，或改变 caller 的 scope。

Web renders this result as the shared `error` or `unavailable` state. It does not relabel a fail-closed response as `available`, infer authorization, or render a diagnostic error as an execution receipt.

Web 将此结果渲染为共享的 `error` 或 `unavailable` 状态。它不得将 fail-closed response 改标为 `available`、推断 authorization，或把 diagnostic error 渲染为 execution receipt。

## Integration Dependencies / 集成依赖

1. **Domain acceptance / 领域验收：** The respective Core owner supplies a documented explicit V1 safe DTO, stable ID rules, deterministic ordering, redaction rules, typed failure behavior, and focused contract tests. Fresh observed commands are listed in the verification matrix; a projection without an observed receipt remains unconnected.
2. **Composition review / 组合审查：** The Integration Lead reviews the frozen domain DTO before adding any private authenticated adapter in `server/api/**` or `packages/local-sdk/**`. The review rejects public-surface changes and preserves exact project/Context/commit or equivalent domain scope.
3. **Presentation adaptation / 展示适配：** The Web owner consumes only the accepted shape through `data -> presenter -> screen`; five shared states are `loading`, `error`, `empty`, `available`, and `unavailable`. It cannot begin from a provisional or raw domain fixture.
4. **Evidence recording / 证据记录：** QA records command, scope, result, and exclusions after each owned increment. A missing receipt is `unobserved`; it never becomes `passed` by admission, source inspection, or UI text.

## Observed Integration State / 已观察到的集成状态

- **Workflow / 工作流：** `contextlab-workflow` owns a provider-free redacted `WorkflowStatusProjectionV1`. The server publishes its explicit fail-closed local availability DTO, while a separate protected read returns redacted source-binding summaries for one exact Context commit. The non-public SDK, same-origin BFF routes, and Web inspectors adapt these two contracts without serializing raw Workflow definitions or rebuilding policy.
- **Benchmark / Benchmark：** `BenchmarkWorkspaceProjectionV1` includes only the stable execution `cohort_id` from a sealed receipt. A later protected discovery read now lists persisted sealed decisions at one exact project/Context/commit scope through the non-public local SDK, same-origin BFF, and Web selection/inspection adapters. Neither projection exposes raw cases, inputs, expected outputs, model outputs, decision policy recomputation, or a graph-diff calculation.
- **Knowledge and Memory / 知识与记忆：** Core contracts provide a redacted deterministic local citation projection and retention/replay facts, while the current Web view remains fixture-only. API/local-SDK transport is not claimed.
- **Plugin and MCP / Plugin 与 MCP：** The existing capability availability projection now rejects non-V1 wire schemas and rebuilds canonical entries through its existing constructor. No dynamic loading or transport is added by this bridge wave.

- **Workflow / 工作流：** `contextlab-workflow` 拥有 provider-free、脱敏的 `WorkflowStatusProjectionV1`。server 发布显式 fail-closed 的本地 availability DTO，另一条独立 protected read 则返回精确 Context commit 的脱敏 source-binding summary。非公开 local SDK、同源 BFF route 与 Web inspector 适配这两份契约，不序列化 raw Workflow definition，也不重建 policy。
- **Benchmark / Benchmark：** `BenchmarkWorkspaceProjectionV1` 只包含 sealed receipt 的稳定 execution `cohort_id`。后续补充的 protected discovery read 现可通过非公开 local SDK、同源 BFF 与 Web selection/inspection adapter，列出精确 project/Context/commit scope 下已持久化的 sealed decision。两类 projection 均不暴露 raw case、input、expected output、model output，不重算 decision policy 或 graph diff。
- **Knowledge and Memory / 知识与记忆：** Core contract 提供脱敏、确定性的本地 citation projection 与 retention/replay fact，当前 Web 视图仍仅为 fixture。此处不声称已有 API/local-SDK transport。
- **Plugin and MCP / Plugin 与 MCP：** 既有 capability availability projection 现拒绝非 V1 wire schema，并通过已有 constructor 重建 canonical entry。本桥接波次不新增 dynamic loading 或 transport。

## Private Workflow Binding Read / 私有 Workflow Binding 读取

The `2026-07-22-private-workflow-binding-read` plan produced a private consumer of the already recorded Context-to-Workflow source binding. Source inspection observes the protected API DTO/route, local SDK parser/client, same-origin BFF route, and Web binding data/presenter/screen/inspector files. Previously recorded focused local receipts cover those source and adapter boundaries; they are not PostgreSQL-backed authenticated runtime, browser, visual, or external deployment evidence.

`2026-07-22-private-workflow-binding-read` 计划已产出已记录 Context 到 Workflow source binding 的私有 consumer。源码检查已观测到 protected API DTO/route、local SDK parser/client、同源 BFF route，以及 Web binding data/presenter/screen/inspector 文件。此前记录的聚焦本地回执覆盖这些源码与 adapter 边界；它们不是 PostgreSQL-backed authenticated runtime、browser、visual 或外部部署证据。

### Architecture Boundary / 架构边界

The source binding remains owned by the Workflow and storage contracts. The read adapter must project only the server-owned, versioned redacted schema `contextlab.local-workflow-context-bindings.v1`:

source binding 仍由 Workflow 与 storage contract 所有。读取 adapter 只能投影 server-owned、带版本的脱敏 schema `contextlab.local-workflow-context-bindings.v1`：

```text
ContextId + exact commit_id
  -> protected local route
  -> ContextPermission::Read before repository access
  -> exact Context/commit repository read
  -> redacted binding summary
  -> fail-closed local SDK parser
  -> same-origin BFF
  -> data -> presenter -> screen
```

```text
ContextId + 精确 commit_id
  -> protected local route
  -> 在 repository access 前执行 ContextPermission::Read
  -> 精确 Context/commit repository read
  -> 脱敏 binding summary
  -> fail-closed local SDK parser
  -> 同源 BFF
  -> data -> presenter -> screen
```

The safe projection contains `schema_version`, `context_id`, `commit_id`, and ordered entries with `binding_id`, `workflow_id`, `workflow_revision`, `node_count`, and `edge_count`. Ordering remains the storage-defined `(workflow_id, workflow_revision, binding_id)` order. An absent repository returns typed `unavailable`; the adapter does not fall back to preview data, current head, another Context, or a raw Workflow payload.

安全投影包含 `schema_version`、`context_id`、`commit_id`，以及带有 `binding_id`、`workflow_id`、`workflow_revision`、`node_count` 与 `edge_count` 的有序条目。排序保持 storage 定义的 `(workflow_id, workflow_revision, binding_id)` 顺序。repository 缺失时返回类型化的 `unavailable`；adapter 不得回退到 preview data、current head、其他 Context 或 raw Workflow payload。

### Observed implementation boundary / 已观测实现边界

The API route and redacted response are present; the existing record reports `4 passed` for its focused tests. The non-public local SDK parser/client is present; the existing record reports `35 passed` for its focused contract run. These are previously recorded implementation receipts for their own boundaries, not a newly run command or an authenticated runtime receipt.

API route 与脱敏 response 已存在；既有记录报告其聚焦测试为 `4 passed`。非公开 local SDK parser/client 已存在；既有记录报告其聚焦契约测试为 `35 passed`。这些是此前记录的各自边界 implementation receipt，不是本次新运行的 command，也不是 authenticated runtime 回执。

The Web source contains `local-workflow-context-bindings-data.ts`, `local-workflow-context-bindings-presenter.ts`, `local-workflow-context-bindings-screen.tsx`, and an inspector wired to the selected Context commit. The existing focused record reports a passing Web `tsc --noEmit` and `84 passed`; the inspector preserves loading/error/empty/available/unavailable semantics through shared capability primitives and rejects response scope drift before presentation.

Web 源码包含 `local-workflow-context-bindings-data.ts`、`local-workflow-context-bindings-presenter.ts`、`local-workflow-context-bindings-screen.tsx`，以及已接入选定 Context commit 的 inspector。既有聚焦记录报告 Web `tsc --noEmit` 通过和 `84 passed`；inspector 通过 shared capability primitive 保持 loading/error/empty/available/unavailable 语义，并在 presenter 前拒绝 response scope 漂移。

The inspector's BFF boundary is the same-origin path `/api/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings`. The route forwards only the request-scoped bearer token, omits cookies, uses `cache: no-store`, preserves typed `401`/`403`/`429`/`503` failures, rejects query/body drift, and fails closed on raw workflow fields. The existing focused route record is `7 passed`; the default Web glob does not discover this nested route test, so that historical receipt is recorded separately.

Inspector 的 BFF boundary 是同源 path `/api/local/contexts/{context_id}/commits/{commit_id}/workflow-bindings`。该 route 只转发 request-scoped bearer token、忽略 cookie、使用 `cache: no-store`、保留 typed `401`/`403`/`429`/`503` failure、拒绝 query/body drift，并对 raw Workflow field fail closed。既有 focused route 记录为 `7 passed`；默认 Web glob 不会发现此嵌套 route test，因此这份历史回执单独记录。

### Explicit Non-Claims / 明确不声明

Previously recorded focused API, local SDK, Web, and BFF receipts cover their local source/test boundaries. `GraphDiff::between` remains the sole graph-diff calculator. This increment does not admit public REST/OpenAPI/public SDK, writes, workflow execution, provider calls, branch/merge/rebind lifecycle, or credentials. PostgreSQL-backed authenticated runtime and browser/visual E2E remain `unobserved`; remote CI, operator rehearsal, release, and production promotion remain `deferred`.

此前记录的 API、local SDK、Web 与 BFF focused receipt 覆盖各自本地源码/测试边界。`GraphDiff::between` 仍是唯一的 graph-diff calculator。本次增量不准入 public REST/OpenAPI/public SDK、write、Workflow execution、provider call、branch/merge/rebind lifecycle 或 credential。PostgreSQL-backed authenticated runtime 与 browser/visual E2E 仍为 `unobserved`；remote CI、operator rehearsal、release 与 production promotion 仍为 `deferred`。

The evidence state is explicitly scoped: existing focused local source/test receipts are recorded as `passed`; PostgreSQL-backed authenticated runtime and browser/visual evidence are `unobserved`; remote CI, operator rehearsal, release, and production promotion are `deferred`. The long-term goal remains active; this local read slice is evidence for progress, not completion of the repository convergence criteria.

证据状态按范围明确标注：既有聚焦本地源码/测试回执记录为 `passed`；PostgreSQL-backed authenticated runtime 与 browser/visual evidence 为 `unobserved`；remote CI、operator rehearsal、release 与 production promotion 为 `deferred`。长期目标保持 active；本地 read slice 只是进展证据，不代表 repository convergence criteria 已完成。

## Private Benchmark Decision Discovery Extension / 私有 Benchmark Decision Discovery 扩展

The admitted Benchmark bridge now has one private read extension for discovering persisted sealed
decisions at an exact project/Context/commit scope. The storage owner exposes a separate
`BenchmarkDecisionDiscoveryRepository`; the application service combines it with the existing
evidence repository so discovery filtering and safe definition projection do not become duplicated
Web policy. Memory and PostgreSQL preserve sealed-only filtering and
`recorded_at DESC, decision_id ASC` ordering, including the decision-ID tie break.

当前准入的 Benchmark bridge 增加了一条私有 read extension，用于在精确 project/Context/commit
scope 发现已持久化的 sealed decision。storage owner 暴露独立的
`BenchmarkDecisionDiscoveryRepository`；application service 将它与既有 evidence repository
组合，避免 discovery filtering 与安全 definition projection 变成重复的 Web policy。Memory 与
PostgreSQL 保持 sealed-only filtering 以及 `recorded_at DESC, decision_id ASC` ordering，并覆盖
相同 timestamp 时的 decision-ID tie break。

The private API/local-SDK/BFF/Web composition returns only stable decision identity, suite/dataset
metadata, status, recorded time, and run count. It preserves the existing authenticated read,
Context RBAC, audit, rate-limit, private/no-store, and `data -> presenter -> screen` boundaries.
Both canonical and compatibility parsers reject scope drift, duplicate identities, invalid
timestamps, unstable ordering, and raw benchmark fields. This extension remains outside public
REST/OpenAPI/public SDK, benchmark writes, provider calls, and production evidence.

私有 API/local SDK/BFF/Web composition 只返回稳定的 decision identity、suite/dataset metadata、
status、recorded time 与 run count。它保持既有 authenticated read、Context RBAC、audit、rate-limit、
private/no-store 与 `data -> presenter -> screen` boundary。canonical 与 compatibility parser 都
拒绝 scope drift、duplicate identity、invalid timestamp、unstable ordering 与 raw benchmark field。
本扩展仍不属于 public REST/OpenAPI/public SDK，不包含 benchmark write、provider call 或 production
evidence。

The existing local receipt records a Rust workspace pass, API `150 passed`, storage discovery
`22 passed`, and `pnpm check:web` with public SDK `14`, local SDK `46`, and Web `91` plus a Web
production build. This documentation correction does not claim a new test run. PostgreSQL-backed
authenticated runtime and browser visual/authenticated E2E remain `unobserved`; remote CI, operator
rehearsal, release, and production promotion remain `deferred`.

既有本地回执记录了 Rust workspace pass、API `150 passed`、storage discovery `22 passed`，以及
`pnpm check:web` 中 public SDK `14`、local SDK `46`、Web `91` 与 Web production build。本次文档
纠正不声明新运行了测试。PostgreSQL-backed authenticated runtime 与 browser visual/authenticated
E2E 仍为 `unobserved`；remote CI、operator rehearsal、release 与 production promotion 仍为
`deferred`。

## Workflow Binding Read Fresh Revalidation / Workflow Binding Read 新鲜复核

The exact private Workflow context-binding read is now freshly revalidated across its local adapter chain: API `4 passed`, storage `3 passed`, local SDK `70 passed`, BFF route `9 passed`, focused Web binding/presenter/inspector `8 passed`, `cargo fmt --all -- --check`, and `pnpm check:web` with public SDK `14`, local SDK `70`, Web `160`, and a successful production build. The route remains request-scoped, bearer-only, cookie-free, typed-failure preserving, and `private, no-store`; the Web remains `data -> presenter -> screen` and presents only the server-owned redacted summary. No public OpenAPI/public SDK operation or write was added, and `GraphDiff::between` remains the sole graph-diff calculator. PostgreSQL-backed authenticated runtime, browser/visual E2E, Git binding, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

精确私有 Workflow context-binding read 已在本地 adapter chain 上新鲜复核：API `4 passed`、storage `3 passed`、local SDK `70 passed`、BFF route `9 passed`、聚焦 Web binding/presenter/inspector `8 passed`、`cargo fmt --all -- --check`，以及 `pnpm check:web`（public SDK `14`、local SDK `70`、Web `160`，并成功完成 production build）。route 仍是 request-scoped、bearer-only、cookie-free、保留 typed failure，并使用 `private, no-store`；Web 仍遵循 `data -> presenter -> screen`，只呈现 server-owned 脱敏 summary。没有新增 public OpenAPI/public SDK operation 或 write，`GraphDiff::between` 仍是唯一 graph-diff calculator。PostgreSQL-backed authenticated runtime、browser/visual E2E、Git binding、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。

## 2026-08-02 Scope and Snapshot Hardening / 2026-08-02 Scope 与 Snapshot 硬化

The Workflow/Plugin bridge now has an explicit `snapshot_from_registry_snapshot` entry point. It
resolves requirements against the existing immutable `CapabilityRegistrySnapshotV1`, while the
legacy live-registry helper remains available for compatibility. A later plugin registration cannot
change a capability set already captured for a Workflow scheduling decision. Version, kind, stable
identifier, deterministic ordering, and fail-closed errors remain owned by Rust; no scheduler,
dynamic loader, provider, or transport is introduced.

Workflow/Plugin bridge 现增加显式的 `snapshot_from_registry_snapshot` 入口。它针对既有不可变的
`CapabilityRegistrySnapshotV1` 解析 requirement，同时保留旧的 live-registry helper 以兼容现有调用。
后续 plugin registration 不能改变已经为 Workflow scheduling decision 捕获的 capability set。版本、kind、稳定
identifier、确定性排序与 fail-closed error 继续由 Rust 所有；没有引入 scheduler、dynamic loader、provider 或 transport。

The in-memory Context Graph repository now validates project, Context, and commit membership before
delegating Knowledge/Memory projection persistence or reads. This mirrors the PostgreSQL composite
foreign-key scope and prevents an unknown exact commit from appearing locally persisted. The
projection repository remains immutable, redacted, and provider-free; this repair does not add raw
content, ingestion, retrieval, or a new transport.

内存 Context Graph repository 现会在委托 Knowledge/Memory projection 持久化或读取前校验 project、Context 与
commit membership。该行为与 PostgreSQL 复合外键 scope 对齐，防止未知 exact commit 在本地看似已持久化。
projection repository 继续保持 immutable、脱敏且 provider-free；本修复不增加 raw content、ingestion、retrieval 或新 transport。

Fresh focused evidence for this wave is `contextlab-workflow` plugin bridge `8 passed`, MCP package
tests `10 passed`, storage Knowledge/Memory projection `3 passed`, embedding `6 passed`, and
`cargo fmt --all -- --check`. Full workspace, strict Clippy, locked Rust, Web, and contract checks
remain required before this wave is recorded as integrated. PostgreSQL runtime, Docker, authenticated
browser/visual smoke, Git, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred` unless independently observed.

本波次的新鲜 focused evidence 为 `contextlab-workflow` plugin bridge `8 passed`、MCP package tests `10 passed`、
storage Knowledge/Memory projection `3 passed`、embedding `6 passed` 与 `cargo fmt --all -- --check`。在记录本波次
integrated 前仍必须取得 workspace、strict Clippy、锁定 Rust、Web 与 contract checks。PostgreSQL runtime、Docker、
authenticated browser/visual smoke、Git、remote CI、operator rehearsal、release 与 production 在独立观测前继续为
`unobserved` 或 `deferred`。

## Related Records / 关联记录

- [Wave 3 local capability bridge verification matrix](../verification/wave-3-local-capability-matrix.md)
- [Wave 2 local capability availability contracts](wave-2-local-capability-availability.md)
- [Admitted local capability bridge wave plan](../superpowers/plans/2026-07-19-local-capability-bridge-wave.md)
- [Private Workflow binding read plan](../superpowers/plans/2026-07-22-private-workflow-binding-read.md)
- [Local Context lifecycle API boundary](../api/local-context-lifecycle.md)
- [Private Workflow binding read user flow](../user-flows/wave-1-workspace-integration.md)
- [Active long-term goal](../roadmap/active-long-term-goal.md)
- [Parallel development plan](../roadmap/parallel-development-plan.md)
