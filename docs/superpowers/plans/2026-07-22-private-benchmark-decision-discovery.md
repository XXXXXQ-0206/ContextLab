# Private Benchmark Decision Discovery Plan / 私有 Benchmark Decision 发现计划

## Necessity Record / 必要性记录

**Completion criteria and charter principle / 完成条件与宪章原则：** This increment directly serves Criterion 3, `Benchmark-driven evaluation`, by making already-persisted sealed benchmark decisions discoverable at an exact project/Context/commit scope. It also advances Criterion 1's dataset/suite inspection path and the charter requirement that reusable evaluation history be queryable and visible without moving policy into a page.

本增量直接服务条件 3“Benchmark-driven evaluation”：让已经持久化的 sealed benchmark decision 可以在精确 project/Context/commit scope 下被发现。它也推进条件 1 的 dataset/suite inspection path，以及宪章关于可复用 evaluation history 必须可查询、可见且不能把 policy 搬进页面的要求。

**Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口：** The repository already persists benchmark definitions, runs, scorecards, decisions, run-details, and evaluation diff. The current local Web workflow requires a manually entered `decision_id`; there is no repository/API/SDK contract that lists only sealed decisions for one exact Context commit. Adding a Web-only discovery list would duplicate storage scope, authorization, ordering, and redaction policy.

仓库已经持久化 benchmark definition、run、scorecard、decision、run-details 与 evaluation diff，但当前 local Web workflow 仍要求用户手填 `decision_id`；没有只列出一个精确 Context commit 下 sealed decision 的 repository/API/SDK contract。若只做 Web discovery，会重复 storage scope、authorization、ordering 与 redaction policy。

**Why now / 为什么现在优先：** The private Workflow binding read is locally validated, and its fresh Rust/Web checks establish the current transport and presentation boundary. Criterion 3 is the highest-priority dependency-ready gap: persisted benchmark evidence cannot form a practical dataset/suite/run-details user flow while decision identity remains manual. A redacted read-only discovery contract is smaller and safer than benchmark mutation, execution, or new policy.

私有 Workflow binding read 已完成本地验证，且新鲜 Rust/Web check 已建立当前 transport 与 presentation boundary。条件 3 是当前最高优先级且依赖已满足的缺口：decision identity 仍需手工输入时，持久化 benchmark evidence 不能形成可实际使用的 dataset/suite/run-details 用户流程。脱敏只读 discovery contract 比 benchmark mutation、execution 或新 policy 更小、更安全。

**Explicit non-goals / 明确非目标：** No benchmark write route, runner, provider/evaluator invocation, dataset case read, raw input/output, threshold calculation, regression recalculation, public REST/OpenAPI/public SDK method, Web mutation, Docker/PostgreSQL runtime, browser E2E, remote CI, release, or production work. `GraphDiff::between` remains the sole graph-diff calculator.

不包含 benchmark write route、runner、provider/evaluator invocation、dataset case read、raw input/output、threshold calculation、regression recalculation、public REST/OpenAPI/public SDK method、Web mutation、Docker/PostgreSQL runtime、browser E2E、remote CI、release 或 production 工作。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Smallest affected boundary and bilingual docs / 最小受影响边界与双语文档：** Add one storage repository method and deterministic redacted summary projection, one protected local GET at exact project/Context/commit scope, one non-public SDK parser/client, and one same-origin BFF plus Web `data -> presenter -> screen` adapter that feeds the existing benchmark evidence inspector. Keep ownership within `crates/storage`, `server/api`, `packages/local-sdk`, `apps/web`, and this plan/roadmap documentation.

增加一个 storage repository method 与确定性脱敏 summary projection、一条精确 project/Context/commit scope 的 protected local GET、一个非公开 SDK parser/client，以及一个接入既有 benchmark evidence inspector 的同源 BFF 与 Web `data -> presenter -> screen` adapter。ownership 仅限 `crates/storage`、`server/api`、`packages/local-sdk`、`apps/web` 与本计划/roadmap 文档。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Observe red/green tests for exact scope, authorization/unavailable behavior, sealed-only filtering, deterministic ordering, raw-field rejection, same-origin no-cookie forwarding, and automatic Web decision selection. Then run `cargo fmt --all -- --check`, `cargo test --workspace --quiet`, `pnpm check:web`, focused API/local-SDK/Web/BFF tests, and public-surface/GraphDiff searches. PostgreSQL runtime, browser E2E, and external release evidence remain explicitly unobserved or deferred.

先观察 exact scope、authorization/unavailable、sealed-only filtering、确定性排序、raw-field rejection、同源无 cookie forwarding 与 Web 自动 decision selection 的红绿测试。随后运行 `cargo fmt --all -- --check`、`cargo test --workspace --quiet`、`pnpm check:web`、聚焦 API/local-SDK/Web/BFF test，以及 public-surface/GraphDiff search。PostgreSQL runtime、browser E2E 与外部 release evidence 仍明确为 unobserved 或 deferred。

## Contract Sketch / 契约草案

```text
GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-decisions
schema_version: contextlab.local-benchmark-decision-list.v1
```

The response contains only stable decision identity, suite identity/name, dataset identity/name/count metadata, status, recorded timestamp, and run count. Results are sorted by recorded timestamp descending, then decision identity ascending; only sealed decisions belonging to the exact project/Context/commit are returned. The public REST/OpenAPI/public SDK catalog remains unchanged.

response 只包含稳定 decision identity、suite identity/name、dataset identity/name/count metadata、status、recorded timestamp 与 run count。结果按 recorded timestamp descending、decision identity ascending 排序；只返回属于精确 project/Context/commit 且已 sealed 的 decision。public REST/OpenAPI/public SDK catalog 保持不变。

## Status / 状态

- [x] Freeze repository/application contract and red tests.
- [x] Add API/local SDK/BFF adapters and redaction tests.
- [x] Connect the existing Web benchmark inspector without page-local policy.
- [ ] Re-run focused and cross-stack verification after the safe-summary and aggregate-query hardening.
- [x] Admit the private diff-selection follow-on; keep the long-term goal active.

## Implementation and Evidence / 实现与证据

The contract is implemented with a separate `BenchmarkDecisionDiscoveryRepository` port.
`BenchmarkDecisionDiscoveryService` consumes only that port, so discovery ordering, exact-scope
checks, sealed-only filtering, and redaction remain storage/application concerns rather than Web
policy. The repository returns safe summaries directly: Memory constructs redacted summaries, and
PostgreSQL uses an aggregate metadata query rather than hydrating raw dataset input or expected
output. The API route remains behind the existing authenticated benchmark-read middleware, Context
RBAC, audit, rate limiting, and private/no-store composition.

本契约通过独立的 `BenchmarkDecisionDiscoveryRepository` port 实现。
`BenchmarkDecisionDiscoveryService` 只消费该 port，因此 discovery ordering、exact-scope 校验、
sealed-only filtering 与 redaction 仍属于 storage/application，而不是 Web policy。repository 直接
返回安全 summary：Memory 构造脱敏 summary，PostgreSQL 使用 aggregate metadata query，而不 hydrate
原始 dataset input 或 expected output。API route 继续复用既有 authenticated benchmark-read middleware、
Context RBAC、audit、rate limiting 与 private/no-store 组合。

The private response is `contextlab.local-benchmark-decision-list.v1`. It contains only decision,
suite, dataset metadata, status, recording time, and run count. Both the authoritative local SDK
parser and the compatibility Web parser reject raw benchmark fields, scope drift, duplicate IDs,
invalid timestamps, and unstable ordering. The Web adapter preserves `data -> presenter -> screen`
and the presenter chooses only the first server-ordered decision; no policy or diff algorithm is
reimplemented in React. `GraphDiff::between` remains the sole graph-diff calculator.

私有 response 为 `contextlab.local-benchmark-decision-list.v1`，只包含 decision、suite、dataset
metadata、status、recording time 与 run count。权威 local SDK parser 与兼容 Web parser 都拒绝
raw benchmark field、scope drift、duplicate ID、无效 timestamp 与不稳定 ordering。Web adapter
保持 `data -> presenter -> screen`，presenter 只选择 server-ordered 的第一项；React 不重建 policy
或 diff algorithm。`GraphDiff::between` 仍是唯一 graph-diff calculator。

Historical evidence snapshot / 历史证据快照：

The previously recorded format, focused storage/API/SDK/Web, workspace, and Web-build receipts
predate the current safe-summary, strict timestamp, scope-echo, and aggregate-count hardening.
They remain useful provenance, but are not fresh verification for the current implementation and
are deliberately not repeated here as a passing claim.

此前记录的 format、聚焦 storage/API/SDK/Web、workspace 与 Web-build 回执早于当前
safe-summary、严格 timestamp、scope-echo 与 aggregate-count hardening。它们仍是有用的来源记录，
但不能作为当前实现的新鲜验证，因此这里不再把它们重复写成通过声明。

Current worker-observed focused receipts / 当前 worker 已观察的聚焦回执：

- API library tests after route-security regressions: `153 passed`.
- Local SDK tests after scope-echo and strict timestamp regressions: `51 passed`.
- Compatibility Web strict timestamp tests: `5 passed`; Web lint passed in that worker.
- Nested BFF security tests: `4 passed`; Web lint passed in that worker.
- Public OpenAPI/SDK exclusion contract tests: `8 passed`; TypeScript lint passed in that worker.

- route security regression 后的 API library test：`153 passed`。
- scope-echo 与严格 timestamp regression 后的 local SDK test：`51 passed`。
- 兼容 Web 严格 timestamp test：`5 passed`；该 worker 的 Web lint 通过。
- 嵌套 BFF security test：`4 passed`；该 worker 的 Web lint 通过。
- public OpenAPI/SDK exclusion contract test：`8 passed`；该 worker 的 TypeScript lint 通过。

Focused storage/API/local-SDK/Web/BFF checks, Rust formatting/workspace tests, and the full Web
check must be freshly observed after the hardening before the next documentation closure.
Docker/PostgreSQL runtime, authenticated browser/visual E2E, remote CI, operator rehearsal,
release, and production evidence remain `unobserved` or `deferred`. This slice adds no benchmark
writes, provider calls, public REST/OpenAPI/public SDK method, Web mutation, operator transport,
migration, or production-readiness claim. The long-term goal remains active.

在完成下一次文档收束前，必须新鲜观察 hardening 后的聚焦 storage/API/local-SDK/Web/BFF 检查、
Rust formatting/workspace test 与完整 Web check。Docker/PostgreSQL runtime、authenticated
browser/visual E2E、remote CI、operator rehearsal、release 与 production evidence 仍为 `unobserved`
或 `deferred`。本切片不新增 benchmark write、provider call、public REST/OpenAPI/public SDK method、
Web mutation、operator transport、migration，也不作 production-readiness 声明。长期目标保持 active。

## Next Admitted Increment / 下一项准入增量

The admitted private benchmark decision comparison selection flow now has its own bilingual
Necessity Record and implementation. It reuses the exact discovered sealed decision list to feed
the existing version-backed evaluation diff read, with no new policy calculation, write route, or
public transport. Its final verification remains pending and must prove that both selected decisions
retain exact project/Context/commit scope and that `GraphDiff::between` remains untouched.

已准入的私有 benchmark decision comparison selection flow 现已有独立的双语 Necessity Record 与实现。
它复用已发现的 sealed decision list，为既有 version-backed evaluation diff read 提供选择，不新增
policy calculation、write route 或 public transport。其最终验证仍待完成，必须证明两个 selected
decision 都保留 exact project/Context/commit scope，且 `GraphDiff::between` 不发生变化。
