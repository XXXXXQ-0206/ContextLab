# Local Benchmark Workspace Flow Implementation Plan / 本地 Benchmark Workspace 流程实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:test-driven-development` while implementing each behavior.

**Goal / 目标：** Compose the existing protected benchmark discovery, sealed decision evidence, run-detail, scorecard, regression-threshold, and exact decision-diff readers into one locally usable Context workspace flow. / 将既有受保护 benchmark discovery、sealed decision evidence、run-detail、scorecard、regression threshold 与 exact decision diff 读取能力组合为一个可在本地使用的 Context workspace 流程。

**Architecture / 架构：** Add a Web-only orchestration presenter that describes the ordered workflow and adapts the already-presented scorecard into stable display rows. Add a thin screen that composes the existing evidence and diff inspectors in that order, then replace their separate placement in `context-workspace-screen.tsx`. Existing data clients, parsers, authorization boundaries, scorecard policy, regression conclusions, and diff engines remain authoritative. / 新增仅限 Web 的编排 presenter，用于描述有序 workflow，并将已呈现的 scorecard 适配为稳定展示行；新增薄 screen，按顺序组合既有 evidence 与 diff inspector，再替换它们在 `context-workspace-screen.tsx` 中的分散位置。既有 data client、parser、授权边界、scorecard policy、regression conclusion 与 diff engine 继续保持权威。

**Tech Stack / 技术栈：** Next.js, React, TypeScript, `@contextlab/ui`, Node test runner.

---

## Necessity Record / 必要性记录

**Completion criteria and charter principle / 完成条件与宪章原则：** This increment directly advances completion criteria 1 and 3: the Context workspace must expose an operational evaluation path, and benchmark evidence must support dataset/suite metadata, sealed run inspection, scorecards, regression thresholds, and evaluation comparison. It follows the charter's Context-first and design-system-first principles by placing version-scoped evaluation evidence inside the Context workspace without moving domain policy into React. / 本增量直接推进完成条件 1 与 3：Context workspace 必须提供可操作的评测路径，benchmark 证据必须支持 dataset/suite metadata、sealed run 审阅、scorecard、regression threshold 与 evaluation comparison。它遵循宪章的 Context-first 与 design-system-first 原则，将版本范围内的评测证据放入 Context workspace，同时不把领域策略搬入 React。

**Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口：** Exact-commit decision discovery, sealed evidence/run details, evaluation diff selection, and the workspace scorecard already exist, but they are rendered as disconnected operational blocks. A user cannot scan the intended path as one workflow, and scorecard/regression evidence is visually detached from the exact decision readers. The risk is accidental duplication of requests, policy, or diff logic while composing them. / exact-commit decision discovery、sealed evidence/run details、evaluation diff selection 与 workspace scorecard 已存在，但目前作为彼此分离的操作区块呈现。用户无法把预期路径作为一个 workflow 扫描，scorecard/regression evidence 也与 exact decision reader 视觉脱节。组合过程中主要风险是意外重复 request、policy 或 diff logic。

**Why now / 为什么现在优先：** All required read-only contracts and presenters are present and focused security tests are green. A Web-only composition is therefore the smallest dependency-ready step toward a usable benchmark workspace and does not require new backend or public surface area. / 所需只读 contract 与 presenter 均已存在，聚焦安全测试已有绿色证据。因此，仅 Web 的组合是通向可用 benchmark workspace 的最小依赖就绪步骤，不需要新增 backend 或 public surface。

**Explicit non-goals / 明确非目标：** No Rust, API, SDK package, public REST/OpenAPI/public SDK, benchmark mutation, provider/evaluator execution, policy/diff recalculation, raw case/output payload, credential persistence, cookie forwarding, Docker, browser E2E, release, or production work. `GraphDiff::between` remains untouched and authoritative. / 不涉及 Rust、API、SDK package、public REST/OpenAPI/public SDK、benchmark mutation、provider/evaluator execution、policy/diff 重算、raw case/output payload、凭据持久化、cookie forwarding、Docker、browser E2E、release 或 production。`GraphDiff::between` 保持不变并继续作为权威实现。

**Smallest affected boundary and bilingual docs / 最小受影响边界与双语文档：** Own only `apps/web/**` plus this plan. Add one orchestration presenter, one screen, focused tests, and a minimal `context-workspace-screen.tsx` replacement. Reuse `@contextlab/ui`, `ContextBenchmarkEvidenceInspector`, and `ContextBenchmarkDecisionDiffInspector`; do not add another data client. / ownership 仅限 `apps/web/**` 与本计划。新增一个编排 presenter、一个 screen、聚焦测试，并对 `context-workspace-screen.tsx` 做最小替换。复用 `@contextlab/ui`、`ContextBenchmarkEvidenceInspector` 与 `ContextBenchmarkDecisionDiffInspector`；不新增另一套 data client。

**Fresh verification required / 必须取得的新鲜验证：** Observe a focused RED caused by the missing orchestration contract, then GREEN proving bilingual ordered stages, exact-scope identifiers, empty and populated scorecard states, regression-threshold evidence, composition order, and absence of raw payload language. Run focused Web tests, `pnpm --filter @contextlab/web lint`, and `pnpm --filter @contextlab/web test`. Browser, Docker, authenticated runtime, visual, remote, release, and production evidence remain unobserved or deferred. / 先观察因缺少编排 contract 而产生的聚焦 RED，再取得 GREEN，证明双语有序 stage、exact-scope identifier、scorecard 空/有数据状态、regression-threshold evidence、组合顺序以及不存在 raw payload 文案。运行聚焦 Web tests、`pnpm --filter @contextlab/web lint` 与 `pnpm --filter @contextlab/web test`。browser、Docker、authenticated runtime、visual、remote、release 与 production 证据仍为未观测或延期。

## TDD Steps / TDD 步骤

- [x] Add presenter and screen tests for ordered workflow, scorecard states, and composition boundaries; run them and observe the expected missing-module failure.
- [x] Implement the minimal orchestration presenter and screen by composing existing inspectors and shared primitives.
- [x] Replace the disconnected scorecard/evidence/diff blocks in `context-workspace-screen.tsx` with the orchestration screen.
- [x] Run focused tests, Web lint, and the complete Web test suite; record only observed evidence.

## Observed Evidence / 已观测证据

**RED / 红灯：** The initial focused command failed with `ERR_MODULE_NOT_FOUND` for
`local-benchmark-workspace-presenter` and `local-benchmark-workspace-screen`. This was the expected
failure because the orchestration contract did not yet exist. / 初次聚焦命令因缺少
`local-benchmark-workspace-presenter` 与 `local-benchmark-workspace-screen` 而以
`ERR_MODULE_NOT_FOUND` 失败；这是编排 contract 尚不存在时的预期红灯。A later ordering
regression failed `1/2` while the scorecard preceded decision evidence, then passed after the DOM
was aligned to evidence/run details → scorecard/regression evidence → comparison. / 后续顺序回归在
scorecard 位于 decision evidence 之前时以 `1/2` 失败；DOM 调整为 evidence/run details →
scorecard/regression evidence → comparison 后转绿。

**GREEN / 绿灯：** The focused orchestration tests passed `4/4`. The integration set covering the
workspace mount, decision evidence, sealed run details, and exact dual-decision comparison passed
`19/19`. `pnpm --filter @contextlab/web lint` passed, and
`pnpm --filter @contextlab/web test` passed `113/113`. / 聚焦编排测试通过 `4/4`；覆盖 workspace
挂载、decision evidence、sealed run details 与 exact 双 decision comparison 的联调测试通过
`19/19`；`pnpm --filter @contextlab/web lint` 通过，`pnpm --filter @contextlab/web test` 通过
`113/113`。

**Boundary / 边界：** The implementation adds no data client and performs no threshold, regression,
scorecard, or diff calculation. It composes existing protected readers and adapts the already-presented
scorecard at the screen boundary. Credentials remain request-scoped inside the existing inspectors;
no cookies, writes, providers, raw payloads, Rust, API, SDK package, public surface, or GraphDiff code
was added or changed. Browser, Docker, authenticated runtime, visual, remote, release, and production
evidence remain unobserved or deferred. / 实现没有新增 data client，也不执行 threshold、regression、
scorecard 或 diff 计算；它只组合既有受保护 reader，并在 screen 边界适配已经呈现的 scorecard。
凭据继续由既有 inspector 保持 request-scoped；未新增或修改 cookie、写入、provider、raw payload、
Rust、API、SDK package、public surface 或 GraphDiff 代码。browser、Docker、authenticated runtime、
visual、remote、release 与 production 证据仍为未观测或延期。
