# Private Workflow Binding Interaction Receipt / 私有 Workflow Binding 交互回执

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** The private Workflow
Context-binding read must be usable as a real local `data -> presenter -> screen` workflow, not only
as a statically renderable component. A user-triggered inspect must preserve the selected exact
`(Context, commit)` scope, use request-scoped credentials, and expose the server-owned redacted
summary through the shared bilingual capability states. This directly advances Criteria 1, 2, 4, 5,
6, and 9 while preserving Context-first modeling, reusable Rust/API contracts, and the design-system
boundary.

**完成条件与章程原则：** 私有 Workflow Context binding read 必须是可真实触发的本地
`data -> presenter -> screen` workflow，而不能只有可静态渲染的组件。用户触发 inspect 后，必须保持选定的精确
`(Context, commit)` scope，使用 request-scoped credential，并通过共享双语 capability state 展示 server-owned 的
脱敏 summary。本增量直接推进条件 1、2、4、5、6 与 9，同时保持 Context-first 建模、可复用 Rust/API contract 与
design-system 边界。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The BFF route, local SDK
parser, and screen state mapping already exist and have focused static tests, but the inspector test
does not invoke its inspect control. Without a lifecycle receipt, the request URL, Bearer header,
cookie omission, loading transition, success projection, and typed failure transition are not all
verified at the Web composition boundary. This is an evidence gap, not permission to add another
route or schema implementation.

**未满足依赖、风险或证据缺口：** BFF route、local SDK parser 与 screen state mapping 已存在并具备聚焦静态 test，但
inspector test 尚未真正触发 inspect control。缺少生命周期回执时，Web composition boundary 还没有整体验证 request URL、
Bearer header、cookie omission、loading transition、成功 projection 与 typed failure transition。这是证据缺口，不是
新增 route 或第二套 schema implementation 的理由。

**Why now / 为何现在优先:** The requested local Workflow read vertical slice is otherwise complete,
and this is the smallest missing proof at its existing ownership boundary. It is closer to the named
readability, secure collaboration, and design-system criteria than starting a new transport or
provider feature. The next separate core increment is the atomic three-snapshot read contract for
persisted merge review; it is intentionally outside this receipt.

**为何现在优先：** 当前本地 Workflow read vertical slice 的其余部分已经完成；这是现有 ownership boundary 中最小的
缺失证明。相比启动新的 transport 或 provider feature，它更直接服务已命名的可用性、安全协作与 design-system 条件。
下一项独立核心增量是持久化 merge review 的 atomic three-snapshot read contract，本回执明确不包含它。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Only
`apps/web/src/app/local-workflow-context-bindings.test.tsx` and this plan are owned by this receipt.
The existing inspector, BFF, local SDK, shared presenter, and design-system primitives remain the
implementation path. Roadmap evidence is updated only after commands are observed.

**明确非目标：** No route, schema, SDK method, public API/OpenAPI surface, Web mutation, provider,
workflow execution, raw private field, credential persistence, PostgreSQL runtime, Docker, browser
automation, release, production claim, or second `GraphDiff` calculator is added.

**Explicit non-goals / 明确非目标：** 不新增 route、schema、SDK method、public API/OpenAPI surface、Web mutation、provider、
Workflow execution、raw private field、credential persistence、PostgreSQL runtime、Docker、browser automation、release、
production claim 或第二个 `GraphDiff` calculator。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** The
inspect-handler, API response-header, and Web out-of-order regressions are now present and focused-
verified. The formatting-only follow-up in `server/api/src/lib.rs` has also passed, so the local
hardening gate is full-green. PostgreSQL runtime, authenticated browser, Git, and remote CI remain
`unobserved`; operator rehearsal, release, and production remain `deferred` unless directly
observed.

**下一增量前的新鲜验证：** inspect-handler、API response-header 与 Web out-of-order regression 已存在并完成聚焦
验证。`server/api/src/lib.rs` 的仅格式相关 follow-up 也已通过，因此本地 hardening gate 为 full-green。除非直接观测，
PostgreSQL runtime、authenticated browser、Git 与 remote CI 仍为 `unobserved`；operator rehearsal、release 与 production
仍为 `deferred`。

## Implementation Checklist / 实施清单

- [x] Execute the inspector's inspect control with a request-memory Bearer token and assert the
  exact same-origin BFF URL, `credentials: "omit"`, `cache: "no-store"`, and authorization header.
- [x] Observe successful redacted payload projection and typed failure projection without exposing
  raw Workflow fields or changing the selected commit.
- [x] Record only fresh command output and leave the long-term goal active.
- [x] Re-run `cargo fmt --all -- --check` after the formatting-only follow-up; the complete local
  hardening gate is now full-green.

- [x] 使用 request-memory Bearer token 执行 inspector 的 inspect control，并断言精确同源 BFF URL、
  `credentials: "omit"`、`cache: "no-store"` 与 authorization header。
- [x] 观测成功的脱敏 payload projection 与 typed failure projection，不暴露 raw Workflow field，也不改变选定 commit。
- [x] 只记录新鲜 command output，并保持长期目标 active。
- [x] 在仅格式相关的后续处理后重新运行 `cargo fmt --all -- --check`；完整本地 hardening gate 现为 full-green。

## Fresh Verification Receipt / 新鲜验证回执

The inspector lifecycle tests now execute the React control handler. The success case observed the
exact encoded BFF path, request-memory `Authorization: Bearer ...`, `credentials: "omit"`,
`cache: "no-store"`, and a ready projection that retained the selected Context and commit. The 403
case observed a bilingual typed error state, retained the exact selected commit, and proved that the
upstream diagnostic was not rendered. The focused file is included in the Web suite.

inspector lifecycle test 现已真正执行 React control handler。成功 case 观测到精确编码的 BFF path、request-memory
`Authorization: Bearer ...`、`credentials: "omit"`、`cache: "no-store"`，并得到保持选定 Context 与 commit 的 ready
projection。403 case 观测到双语 typed error state、保持精确选定 commit，并证明上游 diagnostic 不会被渲染。该 focused
file 已纳入 Web suite。

Observed commands:

```powershell
pnpm --filter @contextlab/web test -- src/app/local-workflow-context-bindings.test.tsx
cargo fmt --all -- --check
cargo test --workspace --quiet
cargo test -p contextlab-api local_workflow_bindings --quiet
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo +1.85.0 check --workspace --all-targets --locked
pnpm check:web
```

The observed package test command expanded to the full Web suite and passed `188` tests; the three
Workflow binding lifecycle cases were included as tests `172`-`174`. `pnpm check:web` also passed
public SDK `15`, local SDK `92`, Web `188`, and the local Web production build. Rust workspace tests
passed with API `183` and storage `186 passed, 39 ignored`; the focused API hardening test passed `4`,
strict offline Clippy and locked Rust check passed. `cargo fmt --all -- --check` passed after the
behavior-neutral formatting follow-up in `server/api/src/lib.rs`, so the complete hardening gate is
full-green locally. PostgreSQL runtime,
authenticated browser, Git change-set, and remote CI remain `unobserved`; operator rehearsal,
release, and production remain `deferred`. No Docker runtime or secret was used, and no public API,
SDK, OpenAPI, mutation, or GraphDiff surface changed.

观测到的 package test command 展开为完整 Web suite，并通过 `188` 项；其中三个 Workflow binding lifecycle case 为测试
`172`-`174`。`pnpm check:web` 也通过 public SDK `15`、local SDK `92`、Web `188` 与本地 Web production build。Rust
workspace test 通过 API `183` 与 storage `186 passed, 39 ignored`；API hardening focused test 通过 `4` 项，strict offline
Clippy 与锁定 Rust check 通过。`cargo fmt --all -- --check` 已在 `server/api/src/lib.rs` 的 behavior-neutral formatting
follow-up 后通过，因此完整 hardening gate 已在本地 full-green。PostgreSQL runtime、authenticated browser、Git change-set 与 remote CI 仍为 `unobserved`；operator rehearsal、
release 与 production 仍为 `deferred`。未启用 Docker runtime、未读取 secret，也未改变 public API、SDK、OpenAPI、mutation 或
GraphDiff surface。

## Gate Correction / 门禁修正

Independent review exposed two root-cause gaps: the protected Axum read router lacked the existing
`private_no_store_response` middleware, and an older in-flight inspector response could overwrite a
newly selected commit. Both minimum source repairs are now present, and the focused API response-
header test (`4 passed`) plus the Web out-of-order response test passed. The formatting-only
follow-up also passed, so the hardening status is `full-green locally`. No new transport or policy
algorithm is introduced.

独立审查发现两个根因缺口：protected Axum read router 缺少既有 `private_no_store_response` middleware；inspector 的旧
in-flight response 可能覆盖新选定的 commit。两个最小 source repair 现已存在，focused API response-header test（`4 passed`）
与 Web out-of-order response test 也已通过。仅格式相关的 follow-up 也已通过，因此 hardening 状态为 `full-green locally`。
未新增 transport 或 policy algorithm。

The correction expands the minimal owned boundary only to `server/api/src/lib.rs`, its focused
Workflow route assertions, and `apps/web/src/app/local-workflow-context-bindings-inspector.tsx` plus
its lifecycle test. The required regression proof and formatting follow-up are now observed, so this
local gate can be called full-green. PostgreSQL, browser, Git, remote, operator,
release, and production evidence remain outside this gate.

本次修正只将最小 ownership 边界扩展到 `server/api/src/lib.rs`、其 focused Workflow route assertion，以及
`apps/web/src/app/local-workflow-context-bindings-inspector.tsx` 与其 lifecycle test。所需回归证明与格式相关后续处理现已观测到，因此本地 gate 已可称为 full-green。PostgreSQL、browser、Git、remote、operator、release 与 production
evidence 仍不在此门禁内。

## 2026-07-30 Availability Reachability Correction / 2026-07-30 可用性状态可达性修正

### Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** The private Workflow
binding read must expose the shared five-state `data -> presenter -> screen` contract at the real
inspector boundary. A server-side `503` must become the existing `unavailable` state rather than an
indistinguishable generic error. This closes a concrete Criteria 1, 5, 6, and 9 interaction-evidence
gap while preserving the Context-first and design-system boundaries.

**完成条件与章程原则：** 私有 Workflow binding read 必须在真实 inspector boundary 暴露既有五态
`data -> presenter -> screen` contract。服务端 `503` 必须进入已有 `unavailable` state，而不能与普通 error 混淆。
这直接收束条件 1、5、6 与 9 的交互证据缺口，同时保持 Context-first 与 design-system 边界。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The BFF already uses
`503` for an unavailable local integration and the data/presenter/screen layers already define the
state, but the inspector catch path maps every non-success response to `error`. The unavailable
state is therefore unreachable from the actual user-triggered read. The same review found duplicate
Web/local-SDK parsing and non-cancelled stale requests. The shared local SDK request helper also
omits an explicit `cache: "no-store"`, which is a small private-read safety gap. The parser
duplication and request cancellation remain later maintenance work; the cache directive is included
in this minimal contract repair because it is a one-line shared transport invariant.

**未满足依赖、风险或证据缺口：** BFF 已使用 `503` 表示不可用的本地 integration，data/presenter/screen 也已定义
该 state，但 inspector 的 catch path 将所有非成功 response 都映射为 `error`，因此真实用户触发的 read 无法到达
unavailable。共享 local SDK request helper 还缺少显式 `cache: "no-store"`，构成一个小型 private-read safety gap。
parser 重复及旧请求未取消仍记录为后续维护工作；cache directive 因为是单行共享 transport invariant，纳入本次最小契约修复。

**Why now / 为何现在优先:** This is the smallest root-cause repair inside the already admitted
Workflow read receipt and is directly testable without Docker, PostgreSQL, browser automation,
external services, or any new transport. It is closer to the named local usability and safe
collaboration criteria than starting another feature slice.

**为何现在优先：** 这是已准入 Workflow read 回执内部最小的根因修复，无需 Docker、PostgreSQL、browser automation、
外部服务或新增 transport 即可直接测试。相比启动另一条功能线，它更直接服务已命名的本地可用性与安全协作条件。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Only
`apps/web/src/app/local-workflow-context-bindings-inspector.tsx`, its focused lifecycle test,
`packages/local-sdk/src/client.ts`, its existing client contract assertions, and this receipt are
affected. The existing BFF parser, presenter, screen, design-system primitives, and server
authorization boundary remain unchanged.

**明确非目标：** No route, schema, SDK method, public API/OpenAPI surface, Web mutation, provider,
workflow execution, parser consolidation, request cancellation, credential persistence, PostgreSQL,
Docker, browser automation, release, production claim, or second `GraphDiff` calculator is added.

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** The focused Web
Workflow suite must prove a 503 reaches `unavailable`, a 403 remains `error`, and a successful
inspector response renders its redacted binding row through the existing Screen. Then rerun
`pnpm check:web`, `cargo fmt --all -- --check`, workspace tests, strict offline Clippy, and locked
Rust 1.85.0 check. PostgreSQL runtime, authenticated browser, Git, remote CI, operator rehearsal,
release, and production remain `unobserved` or `deferred`.

**下一增量前的新鲜验证：** focused Web Workflow suite 必须证明 503 到达 `unavailable`、403 仍为 `error`，并且成功
inspector response 会通过既有 Screen 渲染脱敏 binding row。随后重新运行 `pnpm check:web`、`cargo fmt --all -- --check`、
workspace tests、strict offline Clippy 与锁定 Rust 1.85.0 check。PostgreSQL runtime、authenticated browser、Git、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

### Minimal correction and regression receipt / 最小修复与回归回执

The inspector now maps only the typed upstream `503` to `unavailable`; authentication, authorization,
rate-limit, protocol, and transport failures retain `error`. The shared local SDK request helper now
sets `cache: "no-store"` for its private requests. The success regression also renders the selected
resource through `LocalWorkflowContextBindingsScreen`, proving that the fetched redacted row reaches
the final shared screen boundary. No parser or public surface was changed.

inspector 现仅将 typed upstream `503` 映射为 `unavailable`；authentication、authorization、rate-limit、protocol 与
transport failure 继续保持 `error`。共享 local SDK request helper 现为 private request 设置 `cache: "no-store"`。成功 regression 还会将 selected resource 交给
`LocalWorkflowContextBindingsScreen` 渲染，证明获取到的脱敏 row 到达最终 shared screen boundary。没有修改 parser、transport
或 public surface。

- [x] Add the 503 unavailable regression and inspector-to-screen render assertion.
- [x] Assert `cache: "no-store"` on the shared local SDK request helper.
- [x] Run and record focused Web, full Web, Rust, format, Clippy, and MSRV checks.
- [x] Keep the long-term goal active and select the next increment only after this receipt is fresh.

- [x] 增加 503 unavailable regression 与 inspector-to-screen render assertion。
- [x] 断言共享 local SDK request helper 的 `cache: "no-store"`。
- [x] 运行并记录 focused Web、Web 全量、Rust、format、Clippy 与 MSRV check。
- [x] 保持长期目标 active，仅在本回执新鲜后选择下一增量。

### Fresh Verification Receipt / 新鲜验证回执

The red-to-green focused Web test passed `11/11`, including the new `503 -> unavailable` state
and inspector-to-screen redacted-row assertion. The focused local SDK Workflow contract passed
`7/7`, including the shared `cache: "no-store"` transport invariant. Fresh full verification then
passed `pnpm check:web` with public SDK `15`, local SDK `99`, Web `202`, TypeScript/lint, and the
production build; `cargo test --workspace --quiet --no-fail-fast` with storage `193 passed, 39 ignored`;
API Workflow binding `4 passed`; `cargo fmt --all -- --check`; strict offline workspace Clippy; and
locked Rust `1.85.0` check. The long-term goal remains active. Docker/PostgreSQL runtime,
authenticated browser, Git change-set, remote CI, operator rehearsal, release, and production
remain `unobserved` or `deferred`; no new public route, OpenAPI/SDK method, mutation, provider,
secret, or GraphDiff calculator was added.

红转绿 focused Web test 通过 `11/11`，包含新增的 `503 -> unavailable` state 与 inspector-to-screen 脱敏 row assertion。
focused local SDK Workflow contract 通过 `7/7`，包含共享 `cache: "no-store"` transport invariant。随后新鲜全量验证通过
`pnpm check:web`（public SDK `15`、local SDK `99`、Web `202`、TypeScript/lint 与 production build）；
`cargo test --workspace --quiet --no-fail-fast`（storage `193 passed, 39 ignored`）；API Workflow binding `4 passed`；
`cargo fmt --all -- --check`；strict offline workspace Clippy；以及锁定 Rust `1.85.0` check。长期目标保持 active。
Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator rehearsal、release 与 production
仍为 `unobserved` 或 `deferred`；没有新增 public route、OpenAPI/SDK method、mutation、provider、secret 或 GraphDiff calculator。
