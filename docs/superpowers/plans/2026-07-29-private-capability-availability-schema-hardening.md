# Private Capability Availability Schema Hardening / 私有 Capability Availability Schema 硬化

## Necessity Record / 必要性记录

**Service completion criterion and charter principle / 服务完成条件与章程原则:** This increment
directly serves the fail-closed, stable-contract, bilingual design-system and reusable-adapter
principles. A V1 capability availability parser must reject server-owned schema drift before the
shared presenter and screen receive it.

**完成条件与章程原则：** 本增量直接服务于 fail-closed、稳定 contract、双语 design-system 与可复用 adapter 原则。
V1 capability availability parser 必须在 shared presenter 与 screen 接收数据前拒绝 server-owned schema drift。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:** The existing numeric
`schema_version: 1` parser validated known values but silently discarded unknown outer fields and
unknown fields nested in bilingual text. That allowed an upstream contract to evolve without a
typed failure and made it weaker than the surrounding strict local adapters.

**未满足依赖、风险或证据缺口：** 既有 numeric `schema_version: 1` parser 会校验已知值，却静默丢弃 outer unknown fields 与
bilingual text 内的 unknown fields，使 upstream contract 能在没有 typed failure 的情况下漂移，并弱于周边 strict local adapter。

**Why now / 为何现在优先:** The branch-head graph-review selection slice has just reached fresh
local verification. This is the smallest adjacent contract-hardening increment identified by
independent review, requires no external runtime, and prevents unsafe schema drift before further
local capability surfaces are added.

**为何现在优先：** branch-head graph-review selection slice 刚取得新鲜本地验证。这是独立审查发现的最小相邻 contract-hardening
增量，不依赖 external runtime，并能在新增更多 local capability surface 前阻止不安全的 schema drift。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Only
`apps/web/src/app/local-capability-availability-data.ts`, its existing focused test, this plan,
and the four roadmap ledgers are in scope. Reuse the current DTO, presenter, screen, and transport
contracts.

**最小受影响边界与双语文档：** 仅修改 `apps/web/src/app/local-capability-availability-data.ts`、既有 focused test、本计划与四份
roadmap ledger。复用当前 DTO、presenter、screen 与 transport contract。

**Explicit non-goals / 明确非目标：** No new API route, OpenAPI/public SDK method, local transport,
capability state, provider, credentials, raw content, mutation, migration, Docker/PostgreSQL
runtime, browser E2E, remote CI, operator rehearsal, release, production claim, or Diff logic.

**明确非目标：** 不新增 API route、OpenAPI/public SDK method、local transport、capability state、provider、credential、raw content、
mutation、migration、Docker/PostgreSQL runtime、browser E2E、remote CI、operator rehearsal、release、production claim 或 Diff logic。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** The
focused parser red/green test, Web TypeScript and full Web test/build gate, Rust format/workspace
gate, strict offline Clippy, locked Rust check, and static GraphDiff/public-surface inspection
must be observed. Runtime and external release labels remain unchanged.

**下一增量前的新鲜验证：** 必须观测 focused parser red/green test、Web TypeScript 与完整 Web test/build gate、Rust format/workspace
gate、strict offline Clippy、locked Rust check，以及 static GraphDiff/public-surface inspection。runtime 与 external release label 保持不变。

## Implementation Checklist / 实施清单

- [x] Add red coverage for unknown outer V1 fields.
- [x] Reject unknown outer V1 fields and nested bilingual-text fields with a shared allowlist helper.
- [x] Run focused and full verification; record evidence without closing the long-term goal.

- [x] 增加 unknown outer V1 field 的红测。
- [x] 使用共享 allowlist helper 拒绝 unknown outer V1 field 与 nested bilingual-text field。
- [x] 运行 focused 与完整验证；记录证据但不关闭长期目标。

## Local Verification Receipt / 本地验证回执

The red test reproduced the original gap with `Missing expected exception`. After the parser
allowlist was added, the focused test passed `3/3`. The parser now preserves the existing DTO and
rejects unknown outer and nested bilingual fields before presentation.

红测以 `Missing expected exception` 复现原始缺口。加入 parser allowlist 后 focused test 通过 `3/3`。parser 保持既有 DTO，
并在 presentation 前拒绝 outer 与 nested bilingual field 的 unknown shape。

The long-term goal remains active. No public or production boundary is changed by this increment.

长期目标保持 active。本增量不改变 public 或 production boundary。

Fresh full verification also passed `pnpm check:web` with public SDK `15`, local SDK `99`, Web
`201`, TypeScript/lint, and production build; `cargo fmt --all -- --check`; workspace Rust tests
with storage `193 passed, 39 ignored`; strict offline workspace Clippy; and locked Rust `1.85.0`
workspace check. Static inspection observed `impl GraphDiff` count `1`, public SDK branch-head hits
`0`, and retired public commit-graph-diff read hits `0`. Docker/PostgreSQL runtime, authenticated
browser, Git change-set, remote CI, operator rehearsal, release, and production remain
`unobserved` or `deferred`.

新鲜完整验证还通过了 `pnpm check:web`（public SDK `15`、local SDK `99`、Web `201`、TypeScript/lint 与 production build）、
`cargo fmt --all -- --check`、workspace Rust test（storage `193 passed, 39 ignored`）、strict offline workspace Clippy 与锁定
Rust `1.85.0` workspace check。静态 inspection 观测到 `impl GraphDiff` count `1`、public SDK branch-head hit `0` 与 retired
public commit-graph-diff read hit `0`。Docker/PostgreSQL runtime、authenticated browser、Git change-set、remote CI、operator
rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。
