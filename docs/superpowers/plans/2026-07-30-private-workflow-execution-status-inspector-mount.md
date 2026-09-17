# Private Workflow Execution Status Inspector Mount / 私有 Workflow 执行状态检查器挂载

**Status / 状态:** completed / verified locally / 已完成，本地已验证

## Necessity Record / 必要性记录

### Criterion and charter principle / 完成条件与章程原则

This increment directly advances Criterion 1 (Context-first platform coverage) and Criterion 5
(workflow provenance and reproducibility). The schema-versioned execution-status projection is
already validated by the Rust core, protected local API, non-public SDK, BFF, and shared screen,
but the mounted Context workspace cannot reach it from the existing Workflow binding inspector.

本增量直接推进条件 1（Context-first 平台覆盖）与条件 5（workflow provenance 与可复现性）。schema-versioned execution-status
projection 已由 Rust core、protected local API、非公开 SDK、BFF 与 shared screen 验证，但已挂载的 Context workspace 还不能从现有
Workflow binding inspector 到达它。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The current binding summary contains exact Context/commit/binding/workflow/revision identity but
no workflow `run_id`. Inferring a run from evaluation IDs, fixtures, preview state, or current
head would violate exact replay provenance. The smallest safe user path is an explicit canonical
workflow `run_id` input, selected against the redacted binding row, with the existing status
loader and scope validator remaining the source of truth.

当前 binding summary 含有精确的 Context/commit/binding/workflow/revision identity，但没有 workflow `run_id`。从 evaluation ID、fixture、
preview state 或 current head 推断 run 会破坏 exact replay provenance。最小安全用户路径是让用户显式输入 canonical workflow `run_id`，
将其绑定到脱敏 binding row，并继续以既有 status loader 与 scope validator 作为 source of truth。

### Why now / 为什么现在优先

This is the nearest dependency-ready product gap after the status read closure: it makes an
already-tested private read actually reachable from the local Context workspace without adding a
producer, persistence, transport, polling, or mutation. It is more necessary than another
read-only projection or benchmark feature because the existing contract is otherwise unreachable.

这是 status read 收束后的最近依赖就绪产品缺口：它让已测试的 private read 真正从本地 Context workspace 可达，同时不增加 producer、
持久化、transport、polling 或 mutation。相比另一个 read-only projection 或 benchmark feature，它更必要，因为现有 contract 否则不可用。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

Own only `apps/web/src/app/local-workflow-context-bindings-inspector.tsx`, its focused test,
`apps/web/src/app/context-workspace-screen.test.tsx`, the Web TypeScript generated-cache exclusion
in `apps/web/tsconfig.json`, and this plan plus the bilingual roadmap receipts. Reuse
`loadLocalWorkflowExecutionStatus`, `LocalWorkflowExecutionStatusScreen`, shared
design-system `Input`, existing bearer-in-memory behavior, `credentials: omit`, `cache: no-store`,
and exact scope validation.

仅负责 `apps/web/src/app/local-workflow-context-bindings-inspector.tsx`、其 focused test、`apps/web/src/app/context-workspace-screen.test.tsx`、
`apps/web/tsconfig.json` 中的 Web TypeScript generated-cache exclusion，以及本计划与双语 roadmap 回执。复用 `loadLocalWorkflowExecutionStatus`、`LocalWorkflowExecutionStatusScreen`、shared design-system `Input`、
既有 bearer-in-memory 行为、`credentials: omit`、`cache: no-store` 与 exact scope validation。

### Explicit non-goals / 明确非目标

- No workflow execution producer, persistence repository, polling, retry scheduler, write, mutation, or run discovery.
- No reuse of evaluation run IDs, fixtures, preview/current-head state, or client-side status/replay calculation.
- No API/SDK/OpenAPI route or DTO change, public surface, provider call, migration, secret, Docker/PostgreSQL, browser, release, or production claim.
- No second `GraphDiff` calculator; `GraphDiff::between` remains the sole graph-diff calculator.

- 不增加 workflow execution producer、持久化 repository、polling、retry scheduler、write、mutation 或 run discovery。
- 不复用 evaluation run ID、fixture、preview/current-head state，也不在客户端计算 status/replay。
- 不修改 API/SDK/OpenAPI route 或 DTO，不增加 public surface、provider、migration、secret、Docker/PostgreSQL、browser、release 或 production 声明。
- 不增加第二个 `GraphDiff` calculator；`GraphDiff::between` 仍是唯一 graph-diff calculator。

### Fresh verification before the next increment / 下一增量前的新鲜验证

Run the focused inspector and workspace reachability tests, the Web BFF/status tests, `pnpm check:web`,
`cargo fmt --all -- --check`, workspace Rust tests, strict offline Clippy, locked Rust `1.85.0`
check, and the sole-GraphDiff/public-surface checks. PostgreSQL/Docker, authenticated browser,
Git, remote CI, operator, release, and production evidence remain `unobserved` or `deferred`.

先运行 inspector 与 workspace reachability focused tests、Web BFF/status tests、`pnpm check:web`、`cargo fmt --all -- --check`、workspace Rust tests、
strict offline Clippy、锁定 Rust `1.85.0` check 与 sole-GraphDiff/public-surface checks。PostgreSQL/Docker、authenticated browser、Git、remote CI、
operator、release 与 production evidence 继续为 `unobserved` 或 `deferred`。

## Ownership / 所有权

- Worker A: `local-workflow-context-bindings-inspector.tsx` and `local-workflow-context-bindings.test.tsx` only.
- Worker B: `context-workspace-screen.test.tsx` only.
- Integration Lead: this plan, roadmap receipts, final integration, and verification.

所有 worker 使用 `gpt-5.6-luna`，不读取 secrets，不回退其他改动；跨边界需求必须回报，不得编辑他人 ownership 文件。

## Completion receipt / 收束回执

The private Context workspace now reaches the existing workflow execution-status projection from
the mounted Workflow binding inspector. The user must select an exact redacted binding row and
enter a canonical `run_id`; the client never derives a run from evaluation IDs, fixtures, preview
state, or current head. Context/commit remount keys and render-time scope guards prevent stale
resource or status state from crossing a selection change. A missing run is represented as `empty`.
Upstream error messages are redacted at both workflow BFF routes and the Web data/presenter
boundary. The run input is required and exposes `aria-required`, `aria-invalid`, and an associated
error alert while all status rendering continues through the shared design-system screen.

本地 Context workspace 现可从已挂载的 Workflow binding inspector 到达既有 workflow execution-status projection。用户必须选择精确的
脱敏 binding row 并输入 canonical `run_id`；客户端不会从 evaluation ID、fixture、preview state 或 current head 推导 run。Context/commit
remount key 与渲染时 scope guard 防止 selection 切换时旧 resource 或 status 泄漏。不存在的 run 显示为 `empty`。两个 workflow BFF
route 与 Web data/presenter boundary 均会脱敏 upstream error message。run input 具备 required、`aria-required`、`aria-invalid` 与关联的
error alert，所有 status UI 继续通过 shared design-system screen 呈现。

| Verification / 验证 | Status / 状态 | Fresh receipt / 新鲜回执 |
| --- | --- | --- |
| Focused inspector and workspace tests | `passed` | Binding inspector `15 passed`; Context workspace reachability `2 passed`. / binding inspector `15 passed`；Context workspace 可达性 `2 passed`。 |
| Execution-status data and presenter tests | `passed` | `7 passed`, including upstream-message redaction and caller-supplied message rejection. / `7 passed`，包含 upstream message 脱敏与 caller message 拒绝。 |
| `pnpm --filter @contextlab/web test` | `passed` | Web `310 passed`; all BFF route tests included by the workspace glob. / Web `310 passed`；所有 BFF route test 均由 workspace glob 纳入。 |
| `pnpm --filter @contextlab/web lint` and `pnpm check:web` | `passed` | TypeScript passed; full check passed with public SDK `15`, local SDK `148`, Web `310`, and production build. / TypeScript 通过；完整检查为 public SDK `15`、local SDK `148`、Web `310` 与 production build。 |
| Rust format/tests/Clippy/MSRV | `passed` | `cargo +1.85.0 fmt --all -- --check`; workspace Rust tests passed with API `223 passed` and storage `239 passed, 41 ignored`; strict offline Clippy; locked Rust `1.85.0` check. / 均通过。 |
| GraphDiff and public-surface boundary | `passed` with legacy verifier note | Exactly one production `impl GraphDiff`; public OpenAPI/SDK contract remains read-only except the existing GraphDiff POST. The legacy verifier still reports baseline `benchmark-workspace-route-method-count:2` because two protected GET route variants exist; it is not claimed as green and was not changed by this increment. / 唯一 production `impl GraphDiff`；公开 OpenAPI/SDK 仍无写入，唯一既有公开 POST 为 GraphDiff。旧 verifier 因两个受保护 GET route variant 报 baseline `benchmark-workspace-route-method-count:2`，不伪称通过，本增量未修改它。 |

No producer, persistence repository, polling, run discovery, mutation, public REST/OpenAPI/SDK
operation, provider, migration, secret, Docker/PostgreSQL runtime, authenticated browser, Git
change-set, release, or production claim was added. Docker/PostgreSQL, browser, visual, and Git
remain `unobserved`; remote CI, operator rehearsal, release, and production remain `deferred`.
The long-term goal remains active.

The Web TypeScript configuration now excludes the transient `.next/dev` tree while retaining
production `.next/types`; this prevents partial development route-type writes from invalidating
the repository lint gate. / Web TypeScript 配置现排除临时 `.next/dev` tree，同时保留 production `.next/types`；这样开发期间部分写入的
route type 不会再使 repository lint 门禁失效。

本增量没有新增 producer、persistence repository、polling、run discovery、mutation、public REST/OpenAPI/SDK operation、provider、migration、secret、
Docker/PostgreSQL runtime、authenticated browser、Git change-set、release 或 production claim。Docker/PostgreSQL、browser、visual 与 Git 继续为
`unobserved`；remote CI、operator rehearsal、release 与 production 继续为 `deferred`。长期目标保持 active。
