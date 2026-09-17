# Private Benchmark Evidence Inspection Plan / 私有 Benchmark Evidence Inspection 计划

> **For agentic workers:** Use `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Each task is tracked with checkbox steps and must retain the read-only boundary.

**Goal / 目标：** Make one sealed benchmark decision inspectable through an authenticated local API, non-public local SDK/BFF, and existing design-system Web workspace without exposing a public contract or mutation path.

**Architecture / 架构：** `contextlab-storage` remains the owner of immutable evidence and `contextlab-evaluation` remains the sole policy calculator. A protected local Axum read handler authorizes the exact Context with `ContextPermission::Read`, rate-limits a benchmark-specific read operation, and maps the storage record into a deliberately narrow response projection. The local SDK and same-origin BFF relay that projection using request-scoped credentials; Web data, presenter, and screen only render the response with shared primitives.

## Necessity Record / 必要性记录

**Criterion / 条件：** Completion criteria 1 and 3 require Context-first benchmark datasets, runs, decisions, API/SDK contracts, and an inspection path. The completed private persistence increment has exact project/Context/commit/decision reads but no authenticated inspection workflow.

完成条件 1 与 3 要求 Context-first 的 benchmark dataset、run、decision、API/SDK contract 与 inspection path。已完成的私有持久化增量具有精确的 project/Context/commit/decision read，但尚无经过认证的 inspection workflow。

**Gap and priority / 缺口与优先级：** A storage-only repository cannot prove that a collaborator can inspect immutable decision evidence safely. This is the closest dependency-ready increment because identity, exact-commit persistence, read authorization, protected local BFF, local SDK, and design-system composition already exist. It precedes dashboard expansion, benchmark execution, A/B orchestration, and evaluation diff because those need an honest single-decision read surface first.

仅有 storage-only repository 无法证明协作者能安全审阅不可变 decision evidence。由于 identity、精确 commit 持久化、read authorization、protected local BFF、local SDK 与 design-system composition 都已存在，这是当前最接近、依赖就绪的增量。它优先于 dashboard 扩张、benchmark execution、A/B orchestration 与 evaluation diff，因为后者先需要诚实的单 decision read surface。

**Non-goals / 非目标：** No benchmark execution, provider call, evaluator plugin, dataset or decision mutation, public REST route, checked-in OpenAPI operation, public TypeScript SDK method, public Web control, raw benchmark case input/oracle exposure, dashboard expansion, A/B orchestration, evaluation diff, GraphDiff change, Docker, release, or production claim.

不包含 benchmark execution、provider call、evaluator plugin、dataset 或 decision mutation、public REST route、检入的 OpenAPI operation、public TypeScript SDK method、public Web control、原始 benchmark case input/oracle 暴露、dashboard 扩张、A/B orchestration、evaluation diff、GraphDiff 变更、Docker、release 或 production 声明。

**Minimal boundary / 最小边界：** Add an opt-in `BenchmarkEvidenceRepository` dependency to the protected local API composition; add only `GET /api/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}` behind authentication, a benchmark-specific protected-read limiter operation, `ContextPermission::Read`, and a response projection containing scope, decision/suite/dataset/run identifiers, comparability, digest, status, recorded timestamp, and metric threshold/coverage/outcomes. Dataset names/cases and per-run model/measurement payloads remain deferred because they require additional reads and widen private disclosure. Add the matching non-public local SDK method, same-origin BFF proxy, Web data/presenter/screen composition, and bilingual documentation.

向 protected local API composition 增加 opt-in 的 `BenchmarkEvidenceRepository` 依赖；只增加位于 authentication 后的 `GET /api/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions/{decision_id}`、专用 benchmark protected-read limiter operation、`ContextPermission::Read`，以及只包含 scope、decision/suite/dataset/run identifier、comparability、digest、status、recorded timestamp、metric threshold/coverage/outcome 的 response projection。dataset name/case 与每个 run 的 model/measurement payload 因需要额外 read 并扩大私有披露而保持延期。增加匹配的非公开 local SDK method、同源 BFF proxy、Web data/presenter/screen composition 与双语文档。

**Blocked criterion, root cause, and minimal repair / 被阻塞条件、根因与最小修复：** The original path omitted `project_id`, while `BenchmarkEvidenceRepository::get_benchmark_decision` intentionally requires it to prove project/Context scope. Guessing it from Context would add an unverified lookup or weaken the contract. The minimal repair is the explicit `{project_id}` path segment and a handler that passes all four identities unchanged; route, SDK, BFF, and presenter tests must prove that exact binding.

原始路径遗漏了 `project_id`，而 `BenchmarkEvidenceRepository::get_benchmark_decision` 有意要求它以证明 project/Context scope。从 Context 猜测 project 会增加未验证查询或削弱 contract。最小修复是显式的 `{project_id}` 路径段，以及把四个 identity 原样传入的 handler；route、SDK、BFF 与 presenter test 必须证明该精确绑定。

**Fresh verification / 新鲜验证：** First observe failing API route, authorization/limiter, local SDK/BFF, and presenter tests. Then run focused Rust/API/local-SDK/Web tests, `pnpm check`, formatter, scoped Clippy, full Rust workspace tests, and a boundary search proving no public OpenAPI/SDK route, write handler, raw case payload, or second graph-diff calculator was added. PostgreSQL integration execution remains unobserved while Docker is disabled.

先观察 API route、authorization/limiter、local SDK/BFF 与 presenter test 的失败。随后运行聚焦 Rust/API/local-SDK/Web test、`pnpm check`、formatter、范围化 Clippy、完整 Rust workspace test，以及边界检索，证明没有新增 public OpenAPI/SDK route、write handler、原始 case payload 或第二个 graph-diff calculator。Docker 关闭时 PostgreSQL integration execution 仍保持未观测。

## Tasks / 任务

- [x] **Task 1: Define the private read projection and opt-in service dependency.**
  - Files: `server/api/src/lib.rs`, `server/api/src/routes.rs`, API route tests.
  - Write failing tests for absent optional composition, exact Context/commit/decision parsing, and a response that excludes benchmark case input and oracle payloads.
  - Add an optional `BenchmarkEvidenceRepository` dependency, narrow serializable response types, stable error mapping, and an exact-commit lookup. Preserve the public router and OpenAPI document.

- [x] **Task 2: Protect the local decision route.**
  - Files: `crates/auth/src/rate_limit.rs`, `crates/storage/src/protected_route_rate_limit.rs`, `server/api/src/lib.rs`, `server/api/src/routes.rs`, focused unit/API tests.
  - Write failing tests proving bearer authentication, `ContextPermission::Read`, a benchmark-specific rate-limit key, and missing local composition fail closed.
  - Add the protected local router entry and middleware only after the red tests establish that no public router exposes it.

- [x] **Task 3: Extend the non-public local SDK and BFF.**
  - Files: `packages/local-sdk/src/*`, `apps/web/src/app/api/local/contexts/[contextId]/commits/[commitId]/benchmark-decisions/[decisionId]/route.ts`, BFF/local-SDK tests.
  - Write failing tests for exact path encoding, bearer forwarding, no cookie forwarding, no cache of private responses, and normalized API errors.
  - Add a read-only local SDK method and same-origin proxy; do not change `packages/ts-sdk`, public OpenAPI, or public Web fetches.

- [x] **Task 4: Compose a design-system inspection panel.**
  - Files: `apps/web/src/app/context-benchmark-evidence-data.ts`, `apps/web/src/app/context-benchmark-evidence-presenter.ts`, `apps/web/src/app/context-workspace-screen.tsx`, `apps/web/src/app/globals.css`, focused data/presenter/screen tests.
  - Write failing tests for bilingual labels, unavailable/forbidden states, deterministic metric ordering, no raw benchmark case values, and stable layout.
  - Compose shared `StatusPill`, table, and existing workspace pattern primitives; keep all data transformation outside the screen.

- [x] **Task 5: Document and verify the boundary.**
  - Files: `ARCHITECTURE.md`, `docs/storage/persistence-foundation.md`, `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`.
  - Record only fresh evidence, preserve the PostgreSQL runtime boundary, and update the next dependency-ready increment after all checks pass.

## Fresh Outcome / 新鲜结果

The protected local route now forwards the exact project/Context/commit/decision tuple to the opt-in evidence repository only after authentication, `ContextPermission::Read`, authorization auditing, and `BenchmarkDecisionRead` rate limiting. Its response projection recursively excludes raw `cases`, `input`, and `expected_output` keys; unavailable composition, authorization failures, rate limiting, missing evidence, and storage failures remain generic and fail closed. The non-public local SDK validates the full response shape and status enum before returning it. The same-origin BFF and browser data client reuse that validation, forward only request-scoped Bearer credentials with `credentials: "omit"`, emit `Cache-Control: private, no-store`, and reject any malformed or raw payload. The design-system panel locks its decision scope while a read is in flight and renders deterministic code-point-sorted identifiers and metric outcomes only.

protected local route 现在只会在 authentication、`ContextPermission::Read`、authorization auditing 与 `BenchmarkDecisionRead` rate limiting 之后，才将精确 project/Context/commit/decision tuple 转发给 opt-in evidence repository。其 response projection 会递归排除 raw `cases`、`input` 与 `expected_output` key；unavailable composition、authorization failure、rate limiting、missing evidence 与 storage failure 均保持 generic 并 fail closed。非公开 local SDK 会在返回前校验完整 response shape 与 status enum。同源 BFF 与 browser data client 复用该校验、仅转发 request-scoped Bearer credential 并使用 `credentials: "omit"`、发出 `Cache-Control: private, no-store`，且拒绝任何 malformed 或 raw payload。design-system panel 会在 read in-flight 时锁定 decision scope，并且只渲染按 code point 确定排序的 identifier 与 metric outcome。

Fresh local verification observed `cargo test -p contextlab-api local_benchmark_decision` with `5 passed`; `cargo test --workspace` with API `129 passed` and storage `159 passed, 31 ignored`; public SDK `14 passed`; local SDK `6 passed`; Web `41 passed`; TypeScript checks; `cargo fmt --all -- --check`; and a successful production Web build. PostgreSQL benchmark evidence runtime tests are compiled but ignored/unobserved while Docker is disabled and no disposable database is configured. This does not create public REST/OpenAPI/public SDK/Web exposure, a mutation path, a benchmark executor, a release receipt, or another graph-diff calculator.

新鲜本地验证已观测到 `cargo test -p contextlab-api local_benchmark_decision` 的 `5 passed`；`cargo test --workspace` 的 API `129 passed` 与 storage `159 passed, 31 ignored`；public SDK `14 passed`；local SDK `6 passed`；Web `41 passed`；TypeScript check；`cargo fmt --all -- --check`；以及成功的 production Web build。Docker 关闭且未配置 disposable database 时，PostgreSQL benchmark evidence runtime test 已完成编译但仍为 ignored/unobserved。本切片不创建 public REST/OpenAPI/public SDK/Web exposure、mutation path、benchmark executor、release receipt 或额外的 graph-diff calculator。
