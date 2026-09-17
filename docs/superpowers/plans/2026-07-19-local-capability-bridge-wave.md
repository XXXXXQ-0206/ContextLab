# Local Capability Bridge Wave Plan / 本地能力桥接波次计划

## Necessity Record / 必要性记录

**Criteria served / 服务条件：** This wave directly advances Criterion 1, Context-first platform coverage, by connecting already-tested Workflow, Knowledge/Memory, and plugin capability contracts to inspectable local adapter paths. Its benchmark item advances Criterion 3, benchmark-driven evaluation, by making the existing sealed, redacted domain projection usable as one cohesive local read workflow.

**未满足条件：** 本波次直接推进条件 1（Context-first 平台覆盖）：将已经过测试的 Workflow、Knowledge/Memory 与 plugin capability 契约连接到可检查的本地 adapter path。Benchmark 项目推进条件 3（benchmark 驱动评测），将既有 sealed、脱敏的领域投影组成一条连贯的本地只读 workflow。

**Gap and dependency evidence / 缺口与依赖证据：** The reusable Rust cores and the `contextlab.local-capability-availability.v1` adapter contract now have fresh local tests. What remains is bounded local read adaptation and capability-state inspection; no consumer may invent availability, replay, evaluation, or diff policy in transport or React.

**缺口与依赖证据（中文）：** 可复用 Rust core 与 `contextlab.local-capability-availability.v1` adapter contract 已获得新鲜本地测试。剩余工作是有界的本地只读适配与 capability-state 检查；任何 transport 或 React consumer 都不得自行定义 availability、replay、evaluation 或 diff policy。

**Why now / 为什么现在：** The completed F+G availability path supplies a shared, versioned, fail-closed presentation vocabulary. The next smallest dependency-ready step is to reuse it for domain contracts that already have deterministic fixtures, instead of adding another storage-only increment or broad public surface.

**Non-goals / 非目标：** No public REST, OpenAPI, public SDK, public write, provider invocation, raw case/output/content exposure, browser E2E claim, Docker/PostgreSQL runtime, release, production, or second graph-diff calculator. `GraphDiff::between` remains the sole graph-diff calculator.

**Minimal boundaries and ownership / 最小边界与所有权：**

| Work item / 工作项 | Exclusive owner boundary / 独占边界 | Deliverable / 交付物 |
| --- | --- | --- |
| Benchmark local read loop / Benchmark 本地读取闭环 | `crates/evaluation/**` | Deterministic redacted workspace projection and contract tests; no transport. |
| Workflow capability core / Workflow 能力核心 | `crates/workflow/**` | Versioned, deterministic capability status/replay failure projection and tests. |
| Knowledge/Memory capability core / 知识记忆能力核心 | `crates/knowledge/**`, `crates/memory/**` | Provider-free redacted citation/retention capability projection and tests. |
| Plugin/MCP adapter contract / Plugin/MCP 适配契约 | `crates/mcp/**`, `crates/plugin-runtime/**` | Capability availability projection through the existing fail-closed registry and tests. |
| Local API composition / 本地 API 组合 | `server/api/**`, `packages/local-sdk/**` | Integration Lead only: private authenticated read adapters after domain DTO review. |
| Web capability inspectors / Web 能力检查器 | `apps/web/src/app/**` | `data -> presenter -> screen` local read adapters using shared primitives; no domain algorithm. |
| QA documentation / QA 文档 | `docs/architecture/wave-3-local-capability-bridges.md`, `docs/verification/wave-3-local-capability-matrix.md` | Bilingual contract/evidence matrix with passed, ignored, and unobserved states only. |

**Shared contract / 共享契约：** Local resources use stable identifier strings, an explicit V1 schema, deterministic ordering, structured fail-closed errors, frozen safe DTOs, bilingual display text, and five shared UI states (`loading`, `error`, `empty`, `available`, `unavailable`). Domain crates calculate policy; local API, CLI, Desktop, and Web only adapt it.

**Fresh verification before another increment / 下一增量前的新鲜验证：** Focused domain and adapter tests, strict Clippy for owned packages, `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, `pnpm check:web`, schema/boundary searches, and bilingual documentation validation. Docker-backed PostgreSQL runtime and browser visual/E2E remain `unobserved` while disabled or unavailable.

## Integration Order / 集成顺序

1. Domain owners deliver deterministic V1 projections and focused tests in disjoint crates.
2. The Integration Lead reviews schemas and composes private local API/local-SDK adapters only after domain tests pass.
3. The Web owner adapts accepted DTOs through `data -> presenter -> screen` with shared capability-state primitives.
4. QA records only observed command output; a failed or unavailable environment remains `failed` or `unobserved` and never becomes a passing receipt.
