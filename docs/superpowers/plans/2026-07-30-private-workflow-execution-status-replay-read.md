# Private Workflow Execution Status and Replay Provenance Read / 私有 Workflow 执行状态与回放 provenance 读取

**Status / 状态:** completed and verified locally / 已完成，本地验证通过

## Necessity Record / 必要性记录

### Criterion and charter principle / 完成条件与章程原则

This increment directly advances Criterion 1 (Context-first versioned, replayable state) and
Criterion 5 (workflow provenance and reproducibility). The reusable `contextlab-workflow` core
already validates immutable bindings, deterministic execution transitions, replay sources, and
redacted scheduler status. A local consumer is still missing, so an execution that can be
replayed cannot yet be inspected at its exact Context scope through the product's read layers.

本增量直接推进条件 1（Context-first 的版本化、可回放状态）与条件 5（workflow provenance 与可复现性）。可复用的
`contextlab-workflow` core 已验证 immutable binding、确定性执行 transition、replay source 与脱敏 scheduler status，
但仍缺少 execution consumer，因此可回放的 execution 还不能在精确 Context scope 通过产品读取层审阅。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

The current private API exposes only Workflow capability availability and exact Context binding
summaries. It has no read contract for a sealed run's Context source, binding identity, run state,
event sequence, replay parent, or capability snapshot digest. Returning raw events, failure text,
provider payloads, or guessing a current head would leak private data or weaken replay provenance.
The smallest safe dependency is an opt-in execution-status repository port whose projection is
already validated by the Rust core and is unavailable by default.

当前 private API 只有 Workflow capability availability 与精确 Context binding summary，没有 sealed run 的 Context source、binding
identity、run state、event sequence、replay parent 或 capability snapshot digest read contract。返回 raw event、failure text、
provider payload，或猜测 current head 都会泄漏 private data 或削弱 replay provenance。最小安全依赖是 opt-in execution-status
repository port；其 projection 由 Rust core 先验证，默认不可用。

### Why now / 为什么现在优先

The independent Luna review identified this consumer gap after the Workflow core and binding read
were already locally verified. It closes a named Context/replay observability boundary without
adding execution, provider calls, persistence migrations, or mutation. It is therefore higher
priority than new Workflow authoring or another Plugin/Benchmark feature, which have no proven
contract defect in the current local wave.

独立 Luna review 在 Workflow core 与 binding read 已本地验证后识别出该 consumer 缺口。它不增加 execution、provider call、
persistence migration 或 mutation，就能收束一个已命名的 Context/replay 可观察性边界。因此优先于新的 Workflow authoring 或
Plugin/Benchmark feature；当前 wave 没有证据证明这些领域存在 contract defect。

### Minimal boundary / 最小边界

- Add a deterministic, schema-versioned, redacted Rust execution-status projection and focused
  replay/scope/schema tests.
- Add an optional API repository port and one protected local GET at an exact Context/run scope;
  missing registration returns a typed unavailable response.
- Add the matching non-public local SDK parser/client and same-origin BFF, then compose Web only as
  `data -> presenter -> screen` with shared capability-state primitives.
- Update only this plan and the bilingual roadmap/parallel verification receipts.

- 增加确定性、schema-versioned、脱敏的 Rust execution-status projection，以及 replay/scope/schema 聚焦测试。
- 增加 optional API repository port 与一个精确 Context/run scope 的 protected local GET；未注册 repository 时返回 typed unavailable。
- 增加匹配的 non-public local SDK parser/client 与同源 BFF；Web 只按 `data -> presenter -> screen` 使用 shared capability-state primitive。
- 文档仅更新本计划与双语 roadmap/parallel verification receipt。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK method, execution start route, workflow mutation, provider call,
  dynamic plugin loading, storage migration, operator transport, or production-readiness claim.
- No raw event log, failure detail, tool payload, model output, credential, secret, or client-side
  replay/status calculation.
- No second `GraphDiff` calculator; `GraphDiff::between` remains the sole graph-diff calculator.

- 不增加 public REST/OpenAPI/public SDK method、execution start route、Workflow mutation、provider call、dynamic plugin loading、
  storage migration、operator transport 或 production-readiness 声明。
- 不返回 raw event log、failure detail、tool payload、model output、credential、secret，也不在客户端计算 replay/status。
- 不增加第二个 `GraphDiff` calculator；`GraphDiff::between` 继续是唯一 graph-diff calculator。

### Fresh verification required / 新鲜验证要求

Before recording completion, run focused Workflow/API/local SDK/BFF/Web tests, `pnpm check:web`,
workspace Rust tests, `cargo fmt --all -- --check`, strict offline Clippy, the locked Rust
`1.85.0` check, and the sole-GraphDiff/public-surface checks. PostgreSQL/Docker runtime,
authenticated browser, visual, Git, remote CI, operator, release, and production evidence remain
unobserved or deferred.

记录完成前必须运行 focused Workflow/API/local SDK/BFF/Web tests、`pnpm check:web`、workspace Rust tests、`cargo fmt --all -- --check`、
strict offline Clippy、锁定 Rust `1.85.0` check，以及 sole-GraphDiff/public-surface checks。PostgreSQL/Docker runtime、authenticated
browser、visual、Git、remote CI、operator、release 与 production evidence 继续为 unobserved 或 deferred。

## Ownership / 所有权

The Workflow core worker owns only `crates/workflow/src` and `crates/workflow/tests`. The
Integration Lead owns API, local SDK, BFF, Web, and roadmap integration. All workers use
`gpt-5.6-luna`, do not read secrets, and must return changed paths plus observed commands.

## Implementation and verification / 实现与验证

`completed / verified locally`. The redacted projection, protected local API read, non-public
local SDK parser/client, same-origin BFF, and shared Web `data -> presenter -> screen` composition
are implemented. The default adapter has no execution repository and therefore returns typed
`unavailable`; this increment adds no execution producer or persistence path.

`completed / verified locally`。脱敏 projection、protected local API read、非公开 local SDK parser/client、同源 BFF 与 shared Web
`data -> presenter -> screen` composition 均已实现。默认 adapter 没有 execution repository，因此返回 typed `unavailable`；本增量不增加
execution producer 或持久化路径。

| Command / 命令 | Status / 状态 | Receipt / 回执 |
| --- | --- | --- |
| `cargo test -p contextlab-workflow --test workflow_execution_status_projection --offline` | `passed: 5` | Replay, scope, schema, sequence, and redaction cases. / 覆盖 replay、scope、schema、sequence 与脱敏。 |
| `cargo test -p contextlab-api local_workflow_execution_status_is_private_and_unavailable_without_a_reader --offline` | `passed: 1` | Public route absent, protected route authenticated, default reader unavailable. / public route 不存在，protected route 需认证，默认 reader 返回 unavailable。 |
| `pnpm --filter @contextlab/local-sdk test` | `passed: 111` | Exact V1 parsing, transport, scope, replay, and 503 behavior. / 覆盖 V1 parsing、transport、scope、replay 与 503。 |
| `pnpm --filter @contextlab/web exec tsx --test "src/app/api/local/**/*.test.ts"` | `passed: 51` | Includes the four execution-status BFF route cases. / 包含 execution-status BFF route 的 4 项测试。 |
| `pnpm check:web` | `passed` | Public SDK `15`, local SDK `111`, Web `229`, TypeScript/lint, and production build. / public SDK `15`、local SDK `111`、Web `229`、TypeScript/lint 与 production build。 |
| `cargo test --workspace --quiet --no-fail-fast --offline` | `passed` | Workspace passed; storage `204 passed, 39 ignored`. / workspace 通过；storage `204 passed, 39 ignored`。 |
| `cargo fmt --all -- --check`; strict offline Clippy; locked Rust `1.85.0` check | `passed` | Repository quality and MSRV gates passed. / repository quality 与 MSRV 门禁通过。 |
| `GRAPH_DIFF_IMPL_COUNT=1`; public workflow/plugin capability surface hits `0` | `passed` | `GraphDiff::between` remains the sole graph-diff calculator and no public capability/status surface was added. / `GraphDiff::between` 仍是唯一 graph-diff calculator，未新增 public capability/status surface。 |

No public REST/OpenAPI/public SDK method, execution start route, workflow mutation, provider,
migration, operator transport, secret, or production-readiness claim was added. Docker/PostgreSQL,
authenticated browser, visual, Git, remote CI, operator, release, and production evidence remain
`unobserved` or `deferred`; the long-term goal remains active.

没有新增 public REST/OpenAPI/public SDK method、execution start route、Workflow mutation、provider、migration、operator transport、secret
或 production-readiness 声明。Docker/PostgreSQL、authenticated browser、visual、Git、remote CI、operator、release 与 production evidence
继续为 `unobserved` 或 `deferred`；长期目标保持 active。

The broader `scripts/verify-local-contracts.ps1` baseline is not a scoped green receipt: it stops
at the pre-existing `benchmark-workspace-route-method-count:2` check. The narrower GraphDiff and
public Workflow/Plugin capability checks above passed; this unrelated benchmark baseline remains
outside the increment.

更宽的 `scripts/verify-local-contracts.ps1` baseline 不是本增量的绿色回执：它在既有
`benchmark-workspace-route-method-count:2` 检查处停止。上方更窄的 GraphDiff 与 public Workflow/Plugin capability check 已通过；该无关
benchmark baseline 仍在本增量之外。

Workflow core worker 仅负责 `crates/workflow/src` 与 `crates/workflow/tests`。Integration Lead 负责 API、local SDK、BFF、Web 与
roadmap integration。所有 worker 使用 `gpt-5.6-luna`，不读取 secret，并必须返回变更路径与实际命令结果。
