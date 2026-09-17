# Real Benchmark Workspace Web Integration Implementation Plan / 真实 Benchmark 工作台 Web 集成实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:test-driven-development` for behavior changes and `superpowers:verification-before-completion` before changing this receipt. Completed steps use checkbox (`- [x]`) syntax.

**Status / 状态：** Implemented and verified locally. / 已实现并完成本地验证。

**Goal / 目标：** Connect the durable, protected `BenchmarkWorkspaceProjectionV1` read to a real
same-origin Context workspace flow while preserving exact scope, server-owned evaluation semantics,
request-memory credentials, and the existing decision inspectors. / 将持久、受保护的
`BenchmarkWorkspaceProjectionV1` 读取接入真实同源 Context workspace flow，同时保持精确 scope、
服务端拥有的 evaluation semantics、仅请求内存 credential 与既有 decision inspector。

**Architecture / 架构：** The private path is protected Axum GET -> non-public local SDK ->
same-origin Next.js BFF -> Web data -> presenter -> screen, composed by one client inspector. The
local SDK remains the only wire parser, the server remains authoritative for scorecard/regression/
evaluation diff, and React owns only request state and presentation. / 私有路径为 protected Axum GET
-> 非公开 local SDK -> 同源 Next.js BFF -> Web data -> presenter -> screen，并由一个 client
inspector 组合。local SDK 仍是唯一 wire parser，server 仍拥有 scorecard/regression/evaluation diff
权威，React 只负责 request state 与展示。

**Tech Stack / 技术栈：** Next.js App Router, React, TypeScript, `@contextlab/local-sdk`,
`@contextlab/ui`, Node test runner, Playwright source/viewport smoke.

---

## Supersession Record / 取代关系记录

This plan supersedes `docs/superpowers/plans/2026-07-22-local-benchmark-workspace-flow.md` as the
current Benchmark Web integration contract. The 2026-07-22 plan remains unchanged as the historical
receipt for a fixture-only/composition-only screen that arranged existing evidence, run-detail,
scorecard, and diff inspectors. Supersession does not rewrite its RED/GREEN evidence or claim that it
had a real projection-backed BFF. / 本计划取代
`docs/superpowers/plans/2026-07-22-local-benchmark-workspace-flow.md`，成为当前 Benchmark Web
integration contract。2026-07-22 计划保持不变，继续作为 fixture-only/composition-only screen 的
历史回执；该 screen 当时只编排既有 evidence、run-detail、scorecard 与 diff inspector。取代关系不会
改写其 RED/GREEN 证据，也不会声称它当时已有真实 projection-backed BFF。

## Necessity Record / 必要性记录

**Completion criteria and charter principles / 收束条件与宪章原则：** This increment advances
Criterion 3 by making the durable Benchmark projection usable in the operational Context workspace.
It also advances Criteria 1 and 2 by retaining exact project, Context, immutable commit, execution
cohort, and optional comparison scope across every adapter. It follows Context-first,
design-system-first, clean-boundary, and server-authoritative principles. / 本增量让持久 Benchmark
projection 可在可操作的 Context workspace 中使用，推进条件 3；同时让每层 adapter 保持精确
project、Context、不可变 commit、execution cohort 与可选 comparison scope，推进条件 1 和 2。
它遵循 Context-first、design-system-first、clean-boundary 与 server-authoritative 原则。

**Gap before implementation / 实现前缺口：** The protected Axum route and strict non-public local
SDK could read a safe persisted projection, but Web still exposed only the earlier fixture/composition
workflow. There was no same-origin BFF for this resource, no real data adapter, no exact revised plus
optional paired-baseline controls, and no lifecycle that distinguished loading, transport failure,
missing data, available data, and unavailable composition. / protected Axum route 与严格非公开
local SDK 已能读取安全持久 projection，但 Web 仍只有较早的 fixture/composition workflow；缺少该
resource 的同源 BFF、真实 data adapter、精确 revised 加可选成对 baseline control，也没有区分
loading、transport failure、missing data、available data 与 unavailable composition 的 lifecycle。

**Why now / 为什么现在优先：** The projection DTO, PostgreSQL reconstruction, protected GET,
strict parser/client, and stable error mapping were frozen. Integrating that existing contract was
the smallest dependency-ready change that replaced fixture-only presentation with real local product
data without adding a public API or another evaluation implementation. / projection DTO、PostgreSQL
reconstruction、protected GET、严格 parser/client 与稳定 error mapping 已冻结。集成该既有 contract
是把 fixture-only 展示替换为真实本地产品数据的最小依赖就绪变更，无需新增 public API 或另一套
evaluation implementation。

**Security and privacy boundary / 安全与隐私边界：** The user supplies a Bearer value only to a
password input with `autocomplete="off"`; React state holds it only for the current mounted inspector.
Browser-to-BFF and BFF-to-Axum requests omit cookies and credentials and use no-store behavior. The
BFF returns `private, no-store` for success and failure, forwards only safe structured errors, and
does not persist, refresh, log, or cookie the token. / 用户只在 `autocomplete="off"` 的 password
input 中提供 Bearer 值；React state 仅在当前挂载 inspector 内持有该值。browser-to-BFF 与
BFF-to-Axum request 均省略 cookie/credential 并使用 no-store。BFF 对成功和失败都返回
`private, no-store`，只转发安全结构化 error，不持久化、刷新、记录 token，也不写入 cookie。

**Exact scope and parser boundary / 精确 scope 与 parser 边界：** Revised project, Context, commit,
and cohort are mandatory. Baseline commit and cohort are optional only as one complete pair. The BFF
rejects unknown/duplicate/partial query keys; the local client URL-encodes all scope values. Web data
reuses `parseLocalBenchmarkWorkspace`, verifies the exact echoed revised and baseline scope, and
recursively freezes the accepted projection. Unknown keys, raw fields, schema drift, unstable or
duplicate ordering, inconsistent counts/coverage, and receipt/diff scope drift fail closed. / revised
project、Context、commit 与 cohort 必填；baseline commit 与 cohort 只能作为完整参数对省略或提供。
BFF 拒绝 unknown/duplicate/partial query key，local client 对所有 scope value 做 URL encoding。Web
data 复用 `parseLocalBenchmarkWorkspace`，校验精确回显的 revised/baseline scope，并递归冻结通过的
projection。unknown key、raw field、schema drift、不稳定或重复 ordering、不一致 count/coverage、
receipt/diff scope drift 均 fail closed。

**Domain-authority boundary / 领域权威边界：** The presenter formats the server projection into a
frozen bilingual view model. The screen renders the server-owned suite, dataset/run provenance,
scorecard, regression status, and optional evaluation diff. Web does not aggregate metrics, apply
thresholds, classify regression, compare evaluations, authorize requests, or reconstruct storage
facts. `GraphDiff::between` remains the sole graph-diff calculator. / presenter 将 server projection
格式化为冻结双语 view model。screen 展示 server-owned suite、dataset/run provenance、scorecard、
regression status 与可选 evaluation diff。Web 不聚合 metric、不应用 threshold、不判定 regression、
不比较 evaluation、不执行 authorization，也不重建 storage fact。`GraphDiff::between` 仍是唯一
graph-diff calculator。

**State, reset, and concurrency boundary / 状态、重置与并发边界：** The inspector exposes exactly
`loading`, `error`, `empty`, `available`, and `unavailable`. Scope edits invalidate the displayed
projection; request identifiers prevent a stale response from replacing newer scope state; controls
are disabled while loading. `ContextWorkspaceScreen` keys the inspector from project, Context, and
the complete candidate commit identities, so project/Context/candidate changes remount it and clear
the token, cohort inputs, baseline inputs, and stale responses. / inspector 精确暴露 `loading`、
`error`、`empty`、`available` 与 `unavailable`。scope 修改会使已显示 projection 失效；request
identifier 阻止旧 response 覆盖较新 scope state；loading 期间 control 禁用。
`ContextWorkspaceScreen` 使用 project、Context 与完整 candidate commit identity 生成 inspector
key，因此 project/Context/candidate 变化会触发 remount，并清除 token、cohort input、baseline input
与陈旧 response。

**Design system and accessibility / 设计系统与可访问性：** Controls use shared `Button`, `Input`,
and `Select`; output uses shared `CapabilityState`, `CodeChip`, `DefinitionGrid`, `StackTable`, and
`StatusPill`. State copy, labels, accessible names, status/live regions, table names, headings, and
empty messages are bilingual. The existing `ContextBenchmarkEvidenceInspector` and
`ContextBenchmarkDecisionDiffInspector` remain available below the real workspace. / control 复用
共享 `Button`、`Input` 与 `Select`；output 复用 `CapabilityState`、`CodeChip`、`DefinitionGrid`、
`StackTable` 与 `StatusPill`。state copy、label、accessible name、status/live region、table name、
heading 与 empty message 均为双语。既有 `ContextBenchmarkEvidenceInspector` 与
`ContextBenchmarkDecisionDiffInspector` 继续保留在真实 workspace 下方。

**Smallest affected boundary and ownership / 最小影响边界与 ownership：** Implementation is
limited to the Benchmark same-origin route, its Web data/presenter/screen/inspector adapters,
workspace composition, focused tests, responsive styles, and static/browser verification. It reuses
the existing server route and local SDK contract. / 实现范围限定为 Benchmark 同源 route、Web
data/presenter/screen/inspector adapter、workspace composition、聚焦测试、响应式 style 与静态/browser
verification；复用既有 server route 与 local SDK contract。

**Explicit non-goals / 明确非目标：** No public REST/OpenAPI/public SDK change, write or mutation
transport, provider/evaluator execution, credential persistence, cookie authentication, raw case or
model output display, policy/diff recomputation, benchmark authoring, Git/release work, or production
claim. / 不涉及 public REST/OpenAPI/public SDK 变更、write/mutation transport、provider/evaluator
execution、credential persistence、cookie authentication、raw case 或 model output 展示、policy/diff
重算、benchmark authoring、Git/release 工作或 production 声明。

**Required verification / 必需验证：** Focused RED then GREEN for BFF forwarding/error mapping,
strict data parsing, all five view states, exact optional baseline, shared-primitives rendering,
preserved inspectors, stale-request protection, and scope-key remount. Then run Web lint/tests/build,
static contract verification, desktop/mobile Playwright smoke, and the integration-owned Rust/
PostgreSQL closure gates. / 先为 BFF forwarding/error mapping、严格 data parsing、五种 view state、
精确可选 baseline、shared-primitive rendering、既有 inspector 保留、stale-request protection 与
scope-key remount 取得 RED 再转 GREEN；随后运行 Web lint/test/build、静态 contract verification、
桌面/移动端 Playwright smoke，以及 Integration Lead 拥有的 Rust/PostgreSQL 收束门禁。

## Tasks / 任务

### Task 1: Same-Origin BFF / 同源 BFF

**Files / 文件：**
- Create: `apps/web/src/app/api/local/projects/[projectId]/contexts/[contextId]/commits/[commitId]/benchmark-workspace/[cohortId]/route.ts`
- Create: `apps/web/src/app/api/local/projects/[projectId]/contexts/[contextId]/commits/[commitId]/benchmark-workspace/[cohortId]/route.test.ts`

- [x] Test exact path scope, complete optional baseline pair, Bearer-only forwarding, no-store/
  credentials-omit behavior, safe upstream mappings, retry timing, and `private, no-store` responses.
  / 测试精确 path scope、完整可选 baseline 参数对、仅 Bearer forwarding、no-store/
  credentials-omit 行为、安全 upstream mapping、retry timing 与 `private, no-store` response。
- [x] Implement the thin BFF through `ContextLabLocalClient.getBenchmarkWorkspace` with no second
  parser or business policy. / 通过 `ContextLabLocalClient.getBenchmarkWorkspace` 实现薄 BFF，不
  新增第二套 parser 或 business policy。

### Task 2: Strict Web Data Boundary / 严格 Web Data 边界

**Files / 文件：**
- Create: `apps/web/src/app/local-benchmark-workspace-data.ts`
- Create: `apps/web/src/app/local-benchmark-workspace-data.test.ts`

- [x] Test URL encoding, exact baseline query construction, `credentials: "omit"`, `cache:
  "no-store"`, strict local-SDK parsing, exact scope echo, frozen payloads, safe errors, retry timing,
  and incomplete-scope rejection before fetch. / 测试 URL encoding、精确 baseline query 构造、
  `credentials: "omit"`、`cache: "no-store"`、严格 local-SDK parsing、精确 scope echo、冻结 payload、
  安全 error、retry timing，以及 fetch 前拒绝不完整 scope。
- [x] Implement the typed target/resource union and five-state data contract. / 实现类型化
  target/resource union 与五状态 data contract。

### Task 3: Presenter And Screen / Presenter 与 Screen

**Files / 文件：**
- Modify: `apps/web/src/app/local-benchmark-workspace-presenter.ts`
- Modify: `apps/web/src/app/local-benchmark-workspace-presenter.test.ts`
- Modify: `apps/web/src/app/local-benchmark-workspace-screen.tsx`
- Modify: `apps/web/src/app/local-benchmark-workspace-screen.test.tsx`

- [x] Replace fixture-only composition output with a frozen bilingual projection view model for all
  five states and exact scope facts. / 用覆盖五种状态与精确 scope fact 的冻结双语 projection view
  model 替换 fixture-only composition output。
- [x] Render suite, datasets, run provenance, server scorecard, regression status, and optional
  evaluation diff only through shared `@contextlab/ui` primitives and accessible semantics. / 只用
  共享 `@contextlab/ui` primitive 与可访问 semantics 展示 suite、dataset、run provenance、server
  scorecard、regression status 与可选 evaluation diff。
- [x] Prove that zero-row available data remains distinct from `empty` and `unavailable`. / 证明
  zero-row available data 与 `empty`、`unavailable` 保持不同语义。

### Task 4: Inspector And Workspace Composition / Inspector 与 Workspace 组合

**Files / 文件：**
- Create: `apps/web/src/app/local-benchmark-workspace-inspector.tsx`
- Create: `apps/web/src/app/local-benchmark-workspace-inspector.test.tsx`
- Modify: `apps/web/src/app/context-workspace-screen.tsx`
- Modify: `apps/web/src/app/context-workspace-screen.test.tsx`
- Modify: `apps/web/src/app/globals.css`

- [x] Add memory-only Bearer and exact revised/optional baseline controls; invalidate stale output on
  edits, suppress stale async responses, and disable controls during loading. / 增加仅内存 Bearer 与
  精确 revised/可选 baseline control；编辑时使陈旧 output 失效，抑制陈旧 async response，并在
  loading 期间禁用 control。
- [x] Preserve the existing evidence and decision-diff inspectors below the real projection. / 在
  真实 projection 下方保留既有 evidence 与 decision-diff inspector。
- [x] Key the inspector by project, Context, and every candidate commit identity so ownership-scope
  changes remount all request-memory state. / 以 project、Context 与每个 candidate commit identity
  为 inspector 生成 key，使 ownership scope 变化时 remount 全部 request-memory state。
- [x] Keep desktop/mobile layout stable without nested cards or horizontal overflow. / 保持桌面与
  移动 layout 稳定，不引入 nested card 或横向 overflow。

### Task 5: Verification And Documentation / 验证与文档

**Files / 文件：**
- Modify: `apps/web/verify-context-workspace.py`
- Modify: `scripts/verify-local-contracts.ps1`
- Modify: `ARCHITECTURE.md`
- Modify: `docs/api/wave-1-integration-contracts.md`
- Create: `docs/superpowers/plans/2026-07-23-benchmark-workspace-web-integration.md`

- [x] Run focused and complete Web gates, static contract verifiers, and desktop/mobile Playwright
  smoke. / 运行聚焦与完整 Web gate、静态 contract verifier 及桌面/移动端 Playwright smoke。
- [x] Record only observed receipts and preserve authenticated-runtime, Git, remote, release, and
  production exclusions. / 只记录已观测 receipt，并保留 authenticated runtime、Git、remote、
  release 与 production 排除项。

## Observed RED/GREEN Evidence / 已观测 RED/GREEN 证据

**RED / 红灯：** Before production changes, the newly added focused same-origin route and Web data
tests could not resolve the missing real BFF/data modules. Presenter, screen, inspector, and workspace
integration assertions also failed against the prior fixture-only composition because it had no
strict server projection resource, five-state lifecycle, exact paired-baseline controls, or
project/Context/candidate remount contract. These were the expected failures that admitted this
increment; they are not current failures. / production 变更前，新加入的同源 route 与 Web data 聚焦
测试因真实 BFF/data module 缺失而失败；presenter、screen、inspector 与 workspace integration
assertion 在较早 fixture-only composition 上也失败，因为当时没有严格 server projection resource、
五状态 lifecycle、精确成对 baseline control 或 project/Context/candidate remount contract。这些是
准入本增量的预期红灯，不是当前 failure。

**GREEN - Web / 绿灯 - Web：** Focused BFF/data/presenter/screen/inspector/workspace tests passed.
`pnpm --filter @contextlab/web lint` passed. The complete Web suite passed `138`; `pnpm check:web`
also passed public SDK `14`, local SDK `59`, Web `138`, and the production build. / 聚焦
BFF/data/presenter/screen/inspector/workspace 测试通过；`pnpm --filter @contextlab/web lint` 通过；
完整 Web suite 通过 `138` 项；`pnpm check:web` 同时通过 public SDK `14`、local SDK `59`、Web
`138` 与 production build。

**GREEN - Static and visual smoke / 绿灯 - 静态与视觉 smoke：** Both static contract scripts
passed. Updated Playwright smoke passed at desktop `1440x1000` and mobile `390x900`, with no
horizontal overflow or console errors. This smoke verifies the local preview composition and
responsive/accessibility surface; it is not authenticated browser-to-BFF-to-Axum evidence. / 两项
静态 contract script 通过。更新后的 Playwright smoke 在桌面 `1440x1000` 与移动端 `390x900`
通过，无横向 overflow 或 console error。该 smoke 验证 local preview composition 与响应式/可访问性
surface，不是 authenticated browser-to-BFF-to-Axum evidence。

**GREEN - Integration closure / 绿灯 - 集成收束：** Focused evaluation passed `15/15`; focused
storage passed `38/38` after the timestamp regression; full workspace tests passed API `162` and
storage `166 passed, 38 ignored`; strict workspace Clippy and the Rust `1.85` workspace check passed.
One exact producer test passed `1 passed` against a disposable loopback PostgreSQL `16.14`
`SQL_ASCII` database, and the server was stopped afterward. / evaluation 聚焦测试通过 `15/15`；
timestamp regression 后 storage 聚焦测试通过 `38/38`；workspace 全量测试通过 API `162` 与 storage
`166 passed, 38 ignored`；严格 workspace Clippy 与 Rust `1.85` workspace check 通过。一项精确
producer test 在 disposable、loopback-only PostgreSQL `16.14` `SQL_ASCII` database 上以 `1 passed`
通过，随后 server 已停止。

**Boundary / 边界：** `SQL_ASCII` is not production encoding/Unicode readiness or migration-safety
evidence. Authenticated browser-to-BFF-to-Axum runtime and Git change-set evidence remain
`unobserved`. Remote CI, operator rehearsal, public promotion, release, and production evidence
remain `deferred`. Public OpenAPI and the public SDK are unchanged. / `SQL_ASCII` 不构成 production
encoding/Unicode readiness 或 migration-safety 证据。authenticated browser-to-BFF-to-Axum runtime
与 Git change-set evidence 仍为 `unobserved`；remote CI、operator rehearsal、public promotion、
release 与 production evidence 仍为 `deferred`。public OpenAPI 与 public SDK 保持不变。
