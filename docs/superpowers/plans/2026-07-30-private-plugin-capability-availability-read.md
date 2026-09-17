# Private Plugin/MCP Capability Availability Read / 私有 Plugin/MCP 能力可用性读取

**Status / 状态:** completed / verified locally / 已完成 / 本地已验证

## Necessity Record / 必要性记录

### Criterion and charter principle / 完成条件与宪章原则

This increment directly advances Criterion 7, provider and plugin extensibility, and supports
Criteria 1 and 9: Context-scoped capability facts must be inspectable without making the Web a
second policy engine, and the contract must remain bilingual and fail-closed. The reusable
`contextlab-mcp` and `contextlab-plugin-runtime` crates already own manifest, version,
compatibility, lifecycle, registry, diagnostic, and `CapabilityAvailabilityProjection` policy.

本增量直接推进条件 7（provider 与 plugin 可扩展性），并支持条件 1 与 9：Context 作用域的 capability fact 必须可审阅，
Web 不能成为第二个 policy engine，contract 必须保持双语与 fail-closed。可复用的 `contextlab-mcp` 与
`contextlab-plugin-runtime` crate 已拥有 manifest、version、compatibility、lifecycle、registry、diagnostic 与
`CapabilityAvailabilityProjection` policy。

### Gap and dependency / 缺口与依赖

The Rust core projection is not yet exposed through the private local API/SDK/Web read boundary.
Knowledge/Memory already has a local read path, while Plugin/MCP has no transport adapter. The
smallest safe bridge is a Context-scoped, read-only resource that serializes only stable plugin and
capability identifiers, versions, availability, compatibility, and safe diagnostic codes. Missing
registration, unsupported schema, incompatible versions, or malformed upstream data must produce
an empty or unavailable state rather than a guessed capability.

Rust core projection 尚未进入 private local API/SDK/Web read boundary。Knowledge/Memory 已有本地读取链路，而 Plugin/MCP
尚无 transport adapter。最小安全桥接是 Context-scoped、read-only resource，只序列化稳定 plugin/capability identifier、version、
availability、compatibility 与安全 diagnostic code。缺少 registration、unsupported schema、incompatible version 或 malformed
upstream data 时必须返回 empty 或 unavailable state，不得猜测 capability。

### Why now / 为什么现在优先

The core contract and shared capability-state design primitives are already present, so this closes
a named extensibility/readability gap with a bounded adapter change. It is smaller and lower risk
than adding plugin loading, provider execution, or Workflow execution persistence. The independent
Luna review found no additional Knowledge/Memory, CLI/Desktop, or design-system prerequisite.

核心 contract 与 shared capability-state design primitive 已存在，因此本增量可用有界 adapter 直接收束已命名的可扩展性/可读性缺口。
相比新增 plugin loading、provider execution 或 Workflow execution persistence，它更小、风险更低。独立 Luna review 未发现额外的
Knowledge/Memory、CLI/Desktop 或 design-system 前置。

### Minimal boundary / 最小边界

- Rust adapter and tests may touch `crates/mcp`, `crates/plugin-runtime`, and the API-owned plugin
  capability adapter/route only as required by the existing projection contract.
- The non-public local SDK owns a frozen V1 parser and request-scoped credential/no-store client
  method. Web owns only `data -> presenter -> screen` composition using shared capability state
  primitives and bilingual text.
- Documentation is limited to this plan and matching architecture/API/user-workflow receipts.

- Rust adapter 与 tests 仅在 `crates/mcp`、`crates/plugin-runtime` 及 API-owned plugin capability adapter/route 中按既有 projection
  contract 的必要范围修改。
- 非公开 local SDK 负责 frozen V1 parser 与 request-scoped credential/no-store client method；Web 只负责使用 shared capability
  state primitive 与双语文本完成 `data -> presenter -> screen` 组合。
- 文档范围仅为本计划及对应的 architecture/API/user-workflow 回执。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK method, write route, registry mutation, dynamic loading,
  provider/evaluator invocation, migration, operator transport, or production-readiness claim.
- No raw manifest content, source path, credentials, tool payload, provider secret, or plugin
  execution result is returned.
- No second capability policy, no client-side availability calculation, and no second
  `GraphDiff` calculator.

- 不增加 public REST/OpenAPI/public SDK method、写 route、registry mutation、dynamic loading、provider/evaluator invocation、migration、
  operator transport 或 production-readiness 声明。
- 不返回 raw manifest content、source path、credential、tool payload、provider secret 或 plugin execution result。
- 不增加第二套 capability policy、客户端 availability calculation 或第二个 `GraphDiff` calculator。

### Fresh verification required / 新鲜验证要求

Before the increment can be recorded as locally complete, run focused MCP/runtime/API/SDK/Web
tests, `pnpm check:web`, workspace Rust tests, `cargo fmt --all -- --check`, strict offline Clippy,
and the locked Rust `1.85.0` check. Also re-run the sole-GraphDiff and public-surface static checks.
PostgreSQL/Docker runtime, authenticated browser, visual, Git, remote CI, operator, release, and
production evidence remain unobserved or deferred.

在记录本增量本地完成前，必须运行 focused MCP/runtime/API/SDK/Web tests、`pnpm check:web`、workspace Rust tests、
`cargo fmt --all -- --check`、strict offline Clippy 与锁定 Rust `1.85.0` check，并重新运行唯一 GraphDiff 与 public-surface
静态检查。PostgreSQL/Docker runtime、authenticated browser、visual、Git、remote CI、operator、release 与 production evidence
继续为 unobserved 或 deferred。

## Ownership / 所有权

The implementation worker must use `gpt-5.6-luna`, edit only its assigned files, avoid reverting
existing work, and return changed paths plus observed commands. The Integration Lead owns any
cross-file contract reconciliation and the bilingual roadmap receipt.

实现 worker 必须使用 `gpt-5.6-luna`，只编辑分配文件，不回退已有改动，并返回修改路径与实际命令结果。跨文件 contract 对账与双语
路线图回执由 Integration Lead 负责。

## Completion Receipt / 完成回执

`completed / verified locally`. The handoff implementation now exposes the existing
`CapabilityAvailabilityProjection` through a private, Context-scoped API read, a non-public local
SDK parser/client, a same-origin BFF, and the shared bilingual Web capability-state composition.
The response remains redacted, deterministic, provider-free, request-scoped, and `no-store`; no
public Plugin/MCP surface, registry mutation, dynamic loading, provider call, secret, or second
`GraphDiff` calculator was added.

`completed / verified locally`。handoff 实现现已将既有 `CapabilityAvailabilityProjection` 接入 private、Context-scoped API
read、非公开 local SDK parser/client、同源 BFF 与 shared 双语 Web capability-state composition。response 继续保持脱敏、确定性、
provider-free、request-scoped 与 `no-store`；没有新增 public Plugin/MCP surface、registry mutation、dynamic loading、provider call、
secret 或第二个 `GraphDiff` calculator。

Fresh local receipts / 新鲜本地回执：focused MCP `8 passed`，plugin-runtime `9 passed`，API `2 passed`，local SDK `3 passed`，
Web BFF/inspector `5 passed`；`pnpm check:web` passed with public SDK `15`、local SDK `107`、Web `218`、TypeScript/lint 与
production build；`cargo test --workspace --quiet --no-fail-fast` passed with storage `204 passed, 39 ignored`；locked Rust
`1.85.0` check passed；static `GRAPH_DIFF_IMPL_COUNT=1` and public Plugin/MCP capability surface hits `0`.

新鲜本地回执：focused MCP `8 passed`、plugin-runtime `9 passed`、API `2 passed`、local SDK `3 passed`、Web BFF/inspector `5 passed`；
`pnpm check:web` 通过（public SDK `15`、local SDK `107`、Web `218`、TypeScript/lint 与 production build）；
`cargo test --workspace --quiet --no-fail-fast` 通过（storage `204 passed, 39 ignored`）；锁定 Rust `1.85.0` check 通过；
静态 `GRAPH_DIFF_IMPL_COUNT=1`，public Plugin/MCP capability surface 命中 `0`。

Repository-wide format and strict offline Clippy were observed as `failed` in unrelated
`crates/workflow/src/execution_status.rs` drift: `cargo fmt --all -- --check` reported formatting
diffs, and `cargo clippy --workspace --all-targets --offline -- -D warnings` reported two
`missing_docs` errors. These are recorded honestly and are not a Plugin/MCP product blocker;
they remain outside this Docs/QA worker's file ownership. PostgreSQL/Docker runtime, browser,
Git, remote CI, operator, release, and production remain `unobserved` or `deferred`.

Repository-wide format 与 strict offline Clippy 在无关的 `crates/workflow/src/execution_status.rs` drift 上观测为 `failed`：
`cargo fmt --all -- --check` 报告 formatting diff，`cargo clippy --workspace --all-targets --offline -- -D warnings` 报告两个
`missing_docs` error。这里如实记录，且它们不是 Plugin/MCP product blocker；仍超出本 Docs/QA worker 的文件 ownership。
PostgreSQL/Docker runtime、browser、Git、remote CI、operator、release 与 production 继续为 `unobserved` 或 `deferred`。

Two coding-worker dispatch attempts failed before any worker patch was accepted; this is execution
provenance only, not a product blocker. / 两次 coding-worker dispatch 均在接受 worker patch 前失败；这只是 execution provenance，
不是 product blocker。
