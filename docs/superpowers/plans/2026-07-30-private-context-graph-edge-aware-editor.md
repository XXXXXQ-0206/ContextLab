# Private Context Graph Edge-Aware Uses Editor / 私有 Context 图谱边感知 Uses 编辑器

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** This increment
directly advances Criteria 1 and 4. The private Context lifecycle editor already writes typed
`Uses` relationship commits through the guarded Rust/API/local-SDK boundary, but its Web choices
are still component-pair based. The editor must consume the same Context Graph relationship facts
that it reviews so graph editing is backed by the shared graph contract rather than page-local
reconstruction.

**完成条件与章程原则：** 本增量直接推进条件 1 与 4。private Context lifecycle editor 已经通过 guarded
Rust/API/local-SDK boundary 写入类型化 `Uses` relationship commit，但 Web 当前仍按组件 pair 选择。编辑器必须消费
同一套 Context Graph relationship facts，使 graph editing 由共享 graph contract 支撑，而不是页面本地重建。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The lifecycle state
already contains a validated graph snapshot and the existing editor already receives the
component inventory. It does not yet derive current directed `Uses` edges, so Add can offer an
already-existing edge and Remove can offer an absent edge. The server rejects these cases, but the
Web workflow does not reflect the graph state before submission or confirm it after reload.

**未满足依赖、风险或证据缺口：** lifecycle state 已经包含经过验证的 graph snapshot，现有 editor 也已经接收 component
inventory，但尚未派生当前有向 `Uses` edge。因此 Add 可能提供已存在的 edge，Remove 可能提供不存在的 edge。服务端会
拒绝这些情况，但 Web workflow 在提交前没有反映 graph state，也没有在 reload 后确认 graph state。

**Why now / 为何现在优先:** The Rust domain, storage replay, protected local route, local SDK
transport, and version-backed graph review are already locally verified. This is the smallest
dependency-ready Web completion for the named graph editing criterion and is closer to the shared
Context Graph contract than adding another benchmark feature or a new transport surface.

**为何现在优先：** Rust domain、storage replay、protected local route、local SDK transport 与 version-backed graph review
均已完成本地验证。这是当前命名 graph editing criterion 最小且依赖已满足的 Web 收束项，相比新增 benchmark feature 或
transport surface 更直接服务共享 Context Graph contract。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Keep
the existing `data -> presenter -> screen` boundary. Add only pure edge-selection helpers and
their focused tests in `apps/web/src/app/context-lifecycle-presenter.ts` and its test, then wire
the existing editor in `apps/web/src/app/context-lifecycle-editor.tsx` and its test. Update this
plan and the bilingual roadmap/audit records with observed commands only.

**最小受影响边界与双语文档：** 保持既有 `data -> presenter -> screen` boundary。仅在
`apps/web/src/app/context-lifecycle-presenter.ts` 及其测试中增加纯 edge-selection helper，再接入
`apps/web/src/app/context-lifecycle-editor.tsx` 及其测试。只用实际观测到的命令更新本计划与双语 roadmap/audit 记录。

**Explicit non-goals / 明确非目标:** No Rust domain or storage change, API route, OpenAPI
operation, SDK method, migration, drag-and-drop editor, arbitrary edge kind, branch/merge/
rollback writer, public write, provider call, raw private content, secret, Docker/PostgreSQL
runtime claim, authenticated browser claim, remote CI, operator rehearsal, release, production
work, or second `GraphDiff` calculator.

**明确非目标：** 不修改 Rust domain 或 storage，不新增 API route、OpenAPI operation、SDK method、migration、拖拽编辑器、
任意 edge kind、branch/merge/rollback writer、public write、provider call、raw private content、secret、Docker/PostgreSQL
runtime claim、authenticated browser claim、remote CI、operator rehearsal、release、production work 或第二个 `GraphDiff` calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Add
red/green presenter tests for existing-edge filtering, self-edge rejection, deterministic
ordering, and exact removal membership. Add editor regressions showing the rendered Add/Remove
options follow the graph snapshot and the committed state reload remains the source of truth.
Then run focused Web tests, `pnpm check:web`, `cargo test --workspace --quiet --no-fail-fast`,
`cargo fmt --all -- --check`, strict offline Clippy, locked Rust 1.85 check, and the static
`GRAPH_DIFF_IMPL_COUNT=1` check. PostgreSQL/Docker runtime, authenticated browser, Git, remote CI,
operator, release, and production remain `unobserved` or `deferred`.

**下一增量前必须取得的新鲜验证：** 增加 existing-edge filtering、self-edge rejection、确定性排序与 exact removal
membership 的 presenter red/green test；增加 editor regression，证明 Add/Remove option 跟随 graph snapshot，且提交后的
state reload 仍是 source of truth。随后运行 focused Web tests、`pnpm check:web`、`cargo test --workspace --quiet --no-fail-fast`、
`cargo fmt --all -- --check`、strict offline Clippy、锁定 Rust 1.85 check 与 static `GRAPH_DIFF_COUNT=1` check。PostgreSQL/Docker
runtime、authenticated browser、Git、remote CI、operator、release 与 production 继续为 `unobserved` 或 `deferred`。

## Implementation Checklist / 实施清单

- [x] Define deterministic graph-edge selection helpers without duplicating graph validation.
- [x] Bind Add/Remove Uses controls to the current graph snapshot and preserve guarded submit/reload semantics.
- [x] Add focused red/green tests for graph-aware options, replay refresh, and fail-closed malformed graph input.
- [x] Run fresh local verification and update bilingual roadmap/audit receipts.

- [x] 定义确定性的 graph-edge selection helper，不复制 graph validation。
- [x] 将 Add/Remove Uses control 绑定到当前 graph snapshot，并保持 guarded submit/reload 语义。
- [x] 增加 graph-aware option、replay refresh 与 malformed graph input fail-closed 的 focused red/green test。
- [x] 运行新鲜本地验证并更新双语 roadmap/audit 回执。

## Evidence Boundary / 证据边界

This is a private, default-off, local development read/write composition. The existing server-side
authentication, RBAC, audit, rate limiting, idempotency, branch-head CAS, guarded writer, and
GraphDiff ownership remain unchanged. Missing external deployment evidence must remain deferred and
must not be represented by local tests.

这是 private、默认关闭、面向本地开发的 read/write composition。既有服务端 authentication、RBAC、audit、rate limiting、
idempotency、branch-head CAS、guarded writer 与 GraphDiff ownership 保持不变。缺失的外部部署证据继续延期，不能由本地
测试替代。

## Fresh Local Receipt / 新鲜本地回执 (2026-07-30)

The edge-aware binding is completed and verified locally. `deriveLocalUsesEdges` accepts only
validated `component:<id>` graph endpoints and known edge kinds, returns deterministic raw component
IDs, and fails closed on unknown nodes, duplicate Uses edges, self-edges, or malformed graph facts.
Add Uses options exclude self and existing directed edges; Remove Uses options come only from the
current graph snapshot. The editor reselects valid graph-backed pairs after state load and after a
successful guarded commit reload. The actual Rust/API/local-SDK relationship contract, request
credentials, default-off gate, idempotency, branch-head CAS, and graph review composition are
unchanged.

本增量已完成并在本地验证。`deriveLocalUsesEdges` 只接受经过验证的 `component:<id>` graph endpoint 与已知 edge kind，返回
确定性排序的原始 component ID，并对 unknown node、重复 Uses edge、self-edge 或 malformed graph fact fail closed。Add Uses
选项排除 self 与已存在的有向 edge；Remove Uses 选项只来自当前 graph snapshot。editor 会在 state load 以及 guarded commit 成功
reload 后重新选择有效的 graph-backed pair。Rust/API/local-SDK relationship contract、请求凭据、默认关闭 gate、idempotency、
branch-head CAS 与 graph review composition 均未改变。

Fresh focused evidence: `context-lifecycle-presenter.test.ts` and `context-lifecycle-editor.test.tsx`
passed `22`; `pnpm check:web` passed public SDK `15`, local SDK `104`, Web `213`, TypeScript/lint,
and the production build. `cargo fmt --all -- --check`, `cargo test --workspace --quiet --no-fail-fast`
(storage `204 passed, 39 ignored`, API `191 passed`), strict offline Clippy, locked Rust `1.85.0`
check, and `GRAPH_DIFF_IMPL_COUNT=1` all passed.

新鲜 focused evidence：`context-lifecycle-presenter.test.ts` 与 `context-lifecycle-editor.test.tsx` 共 `22` 项通过；
`pnpm check:web` 通过 public SDK `15`、local SDK `104`、Web `213`、TypeScript/lint 与 production build。`cargo fmt --all -- --check`、
`cargo test --workspace --quiet --no-fail-fast`（storage `204 passed, 39 ignored`、API `191 passed`）、strict offline Clippy、锁定
Rust `1.85.0` check 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过。

This advances Criteria 1 and 4 but closes neither criterion nor the long-term goal. No public
REST/OpenAPI/public SDK write, migration, provider, secret, operator transport, Docker/PostgreSQL
runtime, authenticated browser, Git, remote CI, release, or production evidence is claimed.

本增量推进条件 1 与 4，但不关闭任何条件或长期目标。没有新增 public REST/OpenAPI/public SDK write、migration、provider、secret、
operator transport，也不声称 Docker/PostgreSQL runtime、authenticated browser、Git、remote CI、release 或 production evidence 已通过。
