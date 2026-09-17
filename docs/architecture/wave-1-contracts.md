# Wave 1 Contracts and Integration Boundary / Wave 1 契约与集成边界

> **Status / 状态:** Active documentation contract for the parallel Wave 1 slice. This document records interfaces and evidence boundaries; it does not authorize a public route, a root-manifest change, or a production claim.
>
> **状态：** 并行 Wave 1 切片的活动文档契约。本文记录接口与证据边界；不授权新增公开 route、修改根 manifest 或作出生产声明。

## Purpose / 目的

Wave 1 lets bounded domain and adapter owners work in parallel behind a shared contract ledger. The Integration Lead owns root workspace membership, public API/SDK transport composition, and cross-domain integration. H owns this documentation and QA boundary inside `docs/**`.

Wave 1 允许有界领域与 adapter 所有人在共享契约台账之后并行工作。Integration Lead 负责根 workspace membership、public API/SDK transport composition 与跨领域集成。H 只在 `docs/**` 内负责本文档与 QA 边界。

The source ownership ledger is `docs/roadmap/parallel-development-plan.md`. This page adds the integration rules that every Wave 1 owner must satisfy before a pairwise integration begins.

源所有权台账位于 `docs/roadmap/parallel-development-plan.md`。本页补充每个 Wave 1 owner 在成对集成开始前必须满足的集成规则。

## Shared Contract Rules / 共享契约规则

1. **Typed and versioned / 类型化且带版本:** New records use stable identifiers, explicit `schema_version`, declared ordering, and typed error states. JSON field names use explicit `*_id` names where an identifier is represented.
2. **Deterministic / 确定性:** Fixtures, clocks, identifiers, collection ordering, and error mapping are deterministic. A consumer must not recover ordering from incidental insertion order.
3. **Redacted reads / 脱敏读取:** Read projections expose only the fields admitted by their contract. Raw cases, inputs, expected outputs, model outputs, diagnostics, secrets, and raw tool payloads stay outside local and public read shapes.
4. **Domain-owned policy and diff / 领域拥有 policy 与 diff:** Evaluation policy and semantic or behavior diff calculations stay in Rust domain crates. API, SDK, Web, CLI, and Desktop adapt results and do not recalculate them.
5. **Explicit transport boundary / 显式传输边界:** Public REST/OpenAPI/public SDK remain unchanged unless the Integration Lead admits a separate contract. Private local reads and writes remain authenticated, RBAC-checked, audited, rate-limited, idempotent where applicable, and `private, no-store`.
6. **Additive integration / 增量式集成:** A Wave 1 owner may add an owned contract and fixtures, but may not edit root manifests, public route catalogs, or another owner's implementation files.

## Wave 1 Ownership Matrix / Wave 1 所有权矩阵

| Owner / Owner | Bounded contract / 有界契约 | Integration handoff / 集成交接 | Boundary / 边界 |
| --- | --- | --- | --- |
| A Benchmark/Evaluation | Private sealed-decision history summaries and queries at exact project/Context/commit scope; redacted, cursor-paginated, deterministic. | A -> API/SDK -> G; exact decision, run-detail, and evaluation-diff selection. | No raw benchmark payload, policy recomputation, public write, or provider call. |
| B Diff | Semantic and behavior diff result contracts with deterministic fixtures. | B -> API/SDK -> G; presenter consumes domain results. | No UI-local diff algorithm and no second `GraphDiff` calculator. |
| C Workflow | Replayable workflow state machine with explicit node/edge identifiers and failure propagation. | C + E; later API/SDK/Web adapters consume typed state. | No scheduler, provider execution, or root-manifest ownership. |
| D Knowledge/Memory | Provider-free knowledge and memory ports, deterministic local adapters, citation and retention facts. | D + E; later retrieval/plugin adapters consume safe citations. | No provider credentials, raw private content, or network proof. |
| E Plugin/MCP | Manifest, capability registry, compatibility, and fail-closed lifecycle contracts. | C + E and D + E; capability resolution remains version-checked. | No dynamic production loading, public route, or root-manifest ownership. |
| F Desktop/CLI | Shared-core command and adapter staging contracts. | F + G; Desktop/CLI/Web consume the same Rust/domain DTOs. | No duplicated domain logic, signing, or release packaging claim. |
| G Web/Design System | Data boundary, presenter, screen, loading/error/empty/accessibility/responsive states using shared primitives. | G + H; bilingual presentation and QA evidence are reviewed together. | No business logic in components and no browser-auth evidence claim. |
| H Docs/Contribution/QA | Bilingual architecture/API/user-flow/ADR docs, contribution rules, contract matrix, and evidence ledger. | H -> Wave 3; docs and checklist expose the integration and evidence boundary. | `docs/**` only for this task; no code, manifest, script, `.github`, or workflow edit. |

## Workspace Integration Requirements / Workspace 集成要求

These requirements apply when a Wave 1 contract moves from its owner to an integration pair:

以下要求适用于 Wave 1 契约从 owner 交给联调 pair 时：

1. The owner publishes the Rust type/port, fixture, parser or validation rule, ordering rule, and focused test in the owned boundary before transport work starts.
2. Integration consumes the exact owner type or a named, versioned projection. It must not copy the domain algorithm into an API handler, SDK client, Web presenter, CLI command, or Desktop adapter.
3. Root `Cargo.toml`, `package.json`, `pnpm-workspace.yaml`, public route catalogs, OpenAPI, and public SDK membership are Integration Lead surfaces. A dependency that is not registered there is `unavailable`, not an invitation for a Wave 1 owner to edit the root.
4. Cross-domain imports use the shared workspace crate/package contract and preserve the owner-defined error and ordering semantics. A fixture-only adapter is clearly labeled as fixture or preview and is never called live production data.
5. The pair records a focused receipt and the exact remaining boundary. A failing pair blocks only that pair; independent Wave 1 work continues with a typed unavailable state or deterministic fixture.
6. H documentation is updated in the same increment as the contract handoff. The update must state the public/private surface, user flow, test command, and passed, ignored, unobserved, or blocked state.

## Evidence Classification / 证据分类

The following classification is the contract for this Wave 1 documentation slice. A status describes only the named scope.

以下分类是本 Wave 1 文档切片的证据契约。每个状态只描述所指明的范围。

| State / 状态 | Meaning / 含义 | Wave 1 use / Wave 1 用法 |
| --- | --- | --- |
| `passed` | A named local command or docs-only check returned success and its scope is recorded. | Local domain/contract receipts and this docs-only checklist may be marked passed. |
| `ignored` | A test is intentionally excluded by its declared prerequisite, usually disposable PostgreSQL configuration. | Ignored PostgreSQL tests may compile, but ignored is not runtime proof. |
| `unobserved` | No reviewable runtime receipt exists in this worktree. | Docker-backed PostgreSQL, authenticated browser E2E, remote CI, operator rehearsal, release, and production behavior stay unobserved. |
| `blocked` | A named verification cannot proceed or is not clean because a concrete local prerequisite or baseline failure is recorded. | Current Git binding is unavailable because `.git` is empty; the documented strict Clippy baseline remains open where the checked-in ledger says so. |
| `inconclusive` | A receipt exists but is incomplete, contradictory, or cannot be bound to the claimed snapshot. | Do not convert an inconclusive external receipt into a passed Wave 1 contract. |

## Evidence Boundary / 证据边界

Existing checked-in roadmap records include passed local Rust/Web/SDK receipts and intentionally ignored PostgreSQL tests. They do not prove Docker runtime, authenticated browser-to-BFF-to-Axum behavior, remote CI, operator approval, production deployment, rollback, or public-write readiness. This document preserves those distinctions and does not add a new claim.

现有已检入 roadmap 记录包含已通过的本地 Rust/Web/SDK 回执以及有意 ignored 的 PostgreSQL test。它们不能证明 Docker runtime、authenticated browser-to-BFF-to-Axum 行为、远端 CI、operator 批准、生产部署、rollback 或 public-write readiness。本文保留这些区分，不新增相关声明。

The docs-only validation record is `docs/verification/wave-1-contract-checklist.md`. It validates documentation links, required status vocabulary, source anchors, and prohibited overclaims; it is not a substitute for product, database, browser, remote, operator, or production verification.

文档专用验证记录位于 `docs/verification/wave-1-contract-checklist.md`。它验证文档链接、必需状态词、源代码锚点与禁止性过度声明；它不能替代产品、数据库、浏览器、远端、operator 或生产验证。
