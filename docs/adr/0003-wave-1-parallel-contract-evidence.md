# ADR 0003: Wave 1 Parallel Contracts and Evidence Boundaries / ADR 0003：Wave 1 并行契约与证据边界

## Status / 状态

Accepted for the local Wave 1 documentation and QA slice.

已接受，适用于本地 Wave 1 文档与 QA 切片。

## Context / 背景

Wave 1 has independent owners for benchmark/evaluation, diff, workflow, knowledge/memory, plugin/MCP, Desktop/CLI, Web/design system, and docs/QA. Parallel work is useful only when the workspace integration boundary is explicit. Without that boundary, a source file can be mistaken for a public contract, a fixture can be mistaken for live data, or a compile-only database test can be mistaken for runtime evidence.

Wave 1 为 benchmark/evaluation、diff、workflow、knowledge/memory、plugin/MCP、Desktop/CLI、Web/design system 与 docs/QA 设置了独立 owner。只有明确 workspace 集成边界，并行工作才是安全的。没有该边界时，源文件可能被误认为 public contract，fixture 可能被误认为 live data，或仅编译的 database test 可能被误认为 runtime evidence。

## Decision / 决策

1. The shared contract ledger in `docs/roadmap/parallel-development-plan.md` remains the ownership source of truth.
2. Each Wave 1 owner publishes typed, versioned, deterministic contracts and focused fixtures/tests within its exclusive boundary.
3. The Integration Lead alone admits root workspace membership, public API/SDK transport, and cross-domain composition.
4. H maintains bilingual architecture/API/contribution/user-flow/ADR/verification documentation under `docs/**` and does not edit code, manifests, scripts, `.github`, or workflows for this slice.
5. Evidence is scoped and classified as `passed`, `ignored`, `unobserved`, `blocked`, or `inconclusive`. Missing external operating conditions do not become passing evidence through prose, screenshots, local previews, or compile-only tests.
6. Public REST/OpenAPI/public SDK remain unchanged unless a separate decision admits a new surface with its own integration and evidence record.

1. `docs/roadmap/parallel-development-plan.md` 中的共享契约台账继续作为所有权事实来源。
2. 每个 Wave 1 owner 在其独占边界内提供类型化、带版本、确定性的 contract 与 focused fixture/test。
3. 只有 Integration Lead 可以准入根 workspace membership、public API/SDK transport 与跨领域组合。
4. H 在 `docs/**` 下维护双语 architecture/API/contribution/user-flow/ADR/verification 文档；本切片不编辑 code、manifest、script、`.github` 或 workflow。
5. 证据必须有 scope，并分类为 `passed`、`ignored`、`unobserved`、`blocked` 或 `inconclusive`。外部运行条件缺失时，不能通过 prose、screenshot、本地 preview 或仅编译测试把它变成通过证据。
6. 除非单独决策以其集成与证据记录准入新 surface，否则 public REST/OpenAPI/public SDK 保持不变。

## Consequences / 后果

- Independent Wave 1 work can continue when one integration pair is unavailable or blocked.
- 一个联调 pair 不可用或被阻塞时，独立的 Wave 1 工作仍可继续。
- Consumers must handle typed `unavailable` and preserve fixture/live distinctions.
- Consumer 必须处理类型化 `unavailable`，并保持 fixture/live 区分。
- Documentation becomes part of the handoff contract rather than a retrospective status page.
- 文档成为交接契约的一部分，而不是事后状态页面。
- PostgreSQL runtime, authenticated browser E2E, remote CI, operator rehearsal, release, and production evidence remain separate gates.
- PostgreSQL runtime、authenticated browser E2E、remote CI、operator rehearsal、release 与 production evidence 仍是独立门禁。

## Evidence / 证据

The docs-only implementation and validation record is `docs/verification/wave-1-contract-checklist.md`. Existing local test receipts and deferred external evidence are tracked by `docs/roadmap/active-long-term-goal.md`, `docs/roadmap/completion-criteria.md`, and `docs/roadmap/external-release-evidence-protocol.md`.

文档专用实现与验证记录位于 `docs/verification/wave-1-contract-checklist.md`。现有本地测试回执与延期外部证据由 `docs/roadmap/active-long-term-goal.md`、`docs/roadmap/completion-criteria.md` 与 `docs/roadmap/external-release-evidence-protocol.md` 跟踪。
