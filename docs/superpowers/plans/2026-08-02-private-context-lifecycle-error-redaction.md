# Private Context Lifecycle Error Redaction / 私有 Context 生命周期错误脱敏

## Necessity Record / 必要性记录

### Named criterion and charter principle / 命名条件与宪章原则

This increment directly advances Criterion 6 (production security and collaboration) and the
charter's secure-by-default, secret-redaction, and fail-closed principles. A protected local
Context lifecycle editor must not render upstream diagnostic text merely because the response
contains a structured error object.

本增量直接推进条件 6（生产级安全与协作）以及宪章的 secure-by-default、secret-redaction 与 fail-closed 原则。受保护的
local Context lifecycle editor 不得因为 response 带有结构化 error object，就把上游诊断文本渲染出来。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The existing lifecycle loader preserves `body.message` from the BFF and
`presentLifecycleError` appends it for every error except `local_lifecycle_disabled`. Existing
tests cover only the disabled gate, so conflict, rate-limit, unavailable, and unknown-status
responses lack a regression proving that private upstream diagnostics are not exposed.

现有 lifecycle loader 会保留 BFF 的 `body.message`，而 `presentLifecycleError` 除
`local_lifecycle_disabled` 外会将其拼接到 UI 文案。现有测试只覆盖 disabled gate，因此 conflict、rate-limit、unavailable 与未知
status response 缺少“private upstream diagnostic 不得暴露”的回归证明。

### Why now / 为何现在优先

The existing protected route, request-memory Bearer handling, no-store transport, guarded writer,
RBAC, rate limiting, and local lifecycle composition are already implemented. This is the smallest
dependency-ready security correction found by an independent review, and it closes a concrete
redaction gap before any new lifecycle consumer or benchmark expansion. It is more necessary than
repeating already completed benchmark breadth work whose roadmap wording is stale.

现有 protected route、request-memory Bearer、no-store transport、guarded writer、RBAC、限流与 local lifecycle composition 均已实现。
这是独立审查发现的最小依赖就绪安全修正；在新增 lifecycle consumer 或扩展 benchmark 前先收束该具体脱敏缺口，比重复已完成而路线图表述滞后的 benchmark breadth 更必要。

### Explicit non-goals / 明确非目标

- No Rust, storage, migration, API, REST, OpenAPI, SDK, or public write change.
- No Web mutation, transport redesign, provider call, secret access, Docker/PostgreSQL runtime,
  browser, Git, remote CI, operator, release, or production claim.
- Preserve stable error codes, HTTP status, retry-after behavior, exact scope validation,
  `credentials: "omit"`, `cache: "no-store"`, and the existing disabled-gate remediation.
- Do not add a second GraphDiff implementation or put business policy in the screen component.

- 不修改 Rust、storage、migration、API、REST、OpenAPI、SDK 或 public write。
- 不新增 Web mutation、transport redesign、provider call、secret access、Docker/PostgreSQL runtime、browser、Git、remote CI、operator、release 或 production 声明。
- 保持稳定 error code、HTTP status、retry-after、exact scope validation、`credentials: "omit"`、`cache: "no-store"` 与既有 disabled-gate remediation。
- 不新增第二个 GraphDiff implementation，也不把业务 policy 放进 screen component。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

The implementation boundary is `apps/web/src/app/context-lifecycle-data.ts` and
`context-lifecycle-editor.tsx`, plus their focused tests. The data adapter may retain only a safe
local message in `LocalLifecycleProxyError`; the presenter must map status and stable error code
without reading the upstream message. This plan, `active-long-term-goal.md`,
`completion-criteria.md`, `parallel-development-plan.md`, and the final receipt are the bilingual
documentation boundary.

实现边界是 `apps/web/src/app/context-lifecycle-data.ts`、`context-lifecycle-editor.tsx` 及其 focused tests。data adapter 只可在
`LocalLifecycleProxyError` 中保留安全本地 message；presenter 必须依据 status 与稳定 error code 映射文案，不读取 upstream message。
本计划、`active-long-term-goal.md`、`completion-criteria.md`、`parallel-development-plan.md` 与最终回执构成双语文档边界。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First observe red tests for a conflict, rate-limit, and unknown-status lifecycle error carrying a
private upstream message. Then require the focused data/editor/proxy tests, `pnpm check:web`, Rust
workspace tests, format, strict offline Clippy, locked Rust 1.85 check, local contract verification,
and the exact-one `GraphDiff` source check. Docker/PostgreSQL runtime, authenticated browser,
visual smoke, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or
`deferred`.

先观测带有 private upstream message 的 conflict、rate-limit 与 unknown-status lifecycle error 红测。随后必须通过 focused
data/editor/proxy tests、`pnpm check:web`、Rust workspace tests、format、strict offline Clippy、锁定 Rust 1.85 check、local contract
verification 与 exact-one `GraphDiff` source check。Docker/PostgreSQL runtime、authenticated browser、visual smoke、Git、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

## Ownership / 文件归属

- Luna Web worker: lifecycle data adapter and focused data/editor tests.
- Integration Lead: lifecycle presenter/editor mapping, bilingual roadmap receipts, and final
  cross-stack verification.

## Completion Receipt / 收束回执

Status: `completed / verified locally` for this bounded Web security increment; the long-term
goal remains `active`. / 状态：本有界 Web 安全增量为 `completed / verified locally`；长期目标保持 `active`。

The red phase observed the editor exposing `private stale-head diagnostics`; the data red phase
observed three failures for conflict, rate-limit, and unknown-status upstream messages. The green
implementation redacts the message at proxy parsing and again at `LocalLifecycleProxyError`
construction, preserves the stable error code, HTTP status, retry-after, exact scope checks,
request-memory Bearer, `credentials: "omit"`, and `cache: "no-store"`, and maps editor notices
through the existing bilingual status presenter. No screen-local policy or transport contract was
added.

红阶段真实观察到 editor 暴露 `private stale-head diagnostics`；data 红阶段真实观察到 conflict、rate-limit 与 unknown-status
upstream message 的三项失败。绿实现会在 proxy parsing 与 `LocalLifecycleProxyError` construction 两层脱敏，保留 stable error code、
HTTP status、retry-after、exact scope checks、request-memory Bearer、`credentials: "omit"` 与 `cache: "no-store"`，并通过既有双语 status
presenter 映射 editor notice。未新增 screen-local policy 或 transport contract。

Fresh local verification / 新鲜本地验证:

- Focused lifecycle data `9 passed` and editor `14 passed`.
- `pnpm check:web` passed: public SDK `15`, local SDK `148`, Web `308` tests, TypeScript/lint,
  and production build.
- `cargo fmt --all -- --check` passed.
- `cargo test --workspace --quiet --no-fail-fast --offline` passed: API `223 passed`; storage
  `238 passed, 41 ignored`.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` passed.
- `cargo +1.85.0 check --workspace --all-targets --locked --offline` passed.
- `tests/contract/verify-local-contracts.test.ps1` passed.
- Source inspection found `GRAPH_DIFF_IMPL_COUNT=1` and `GraphDiff::between` in `10` Rust matches.

新鲜本地验证：

- lifecycle data 聚焦测试 `9 passed`，editor 聚焦测试 `14 passed`。
- `pnpm check:web` 通过：public SDK `15`、local SDK `148`、Web `308` tests、TypeScript/lint 与 production build。
- `cargo fmt --all -- --check` 通过。
- `cargo test --workspace --quiet --no-fail-fast --offline` 通过：API `223 passed`；storage `238 passed, 41 ignored`。
- `cargo clippy --workspace --all-targets --offline -- -D warnings` 通过。
- `cargo +1.85.0 check --workspace --all-targets --locked --offline` 通过。
- `tests/contract/verify-local-contracts.test.ps1` 通过。
- 源码检查确认 `GRAPH_DIFF_IMPL_COUNT=1`，Rust 中 `GraphDiff::between` 共 `10` 个匹配。

No Rust/API/SDK/OpenAPI/public write, migration, provider, secret access, operator transport,
second GraphDiff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, or production claim was added. Those runtime and external
release boundaries remain `ignored`, `unobserved`, or `deferred`. This receipt advances Criterion 6
only and does not close any completion criterion or the long-term goal. / 未新增 Rust/API/SDK/OpenAPI/public write、migration、
provider、secret access、operator transport、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git、
remote CI、operator rehearsal、release 或 production 声明。上述 runtime 与外部发布边界继续为 `ignored`、`unobserved` 或 `deferred`。
本回执仅推进条件 6，不关闭任何 completion criterion 或长期目标。

## Status / 状态

`completed / verified locally`; the long-term goal remains `active`. / `completed / verified locally`；长期目标保持 `active`。
