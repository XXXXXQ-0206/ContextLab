# Workflow-to-Plugin/MCP Capability Bridge / Workflow 到 Plugin/MCP 能力桥接

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This increment directly
advances Criterion 1, Context-first platform coverage, and Criterion 7, provider and plugin
extensibility. It also preserves the charter rule that reusable Rust domain/application contracts,
not UI or transport adapters, own workflow capability policy.

本增量直接推进条件 1“以 Context 为核心的平台覆盖”和条件 7“Provider 与 plugin 可扩展性”。它同时
遵守项目宪章：Workflow capability policy 必须由可复用的 Rust domain/application contract 承担，
而不是由 UI 或 transport adapter 承担。

**Gap and risk / 缺口与风险：** The Workflow core already validates stable, versioned
`WorkflowCapabilityRequirement` values against an injected `WorkflowCapabilitySnapshot`, while
`contextlab-mcp` already owns compatible plugin registration and version-checked capability
resolution. No typed bridge currently derives the Workflow snapshot from that registry. Adapters
could therefore bypass MCP compatibility, ignore capability kind, or rebuild resolution policy.

Workflow core 已能用注入的 `WorkflowCapabilitySnapshot` 校验稳定、带版本的
`WorkflowCapabilityRequirement`；`contextlab-mcp` 也已拥有兼容 plugin registration 与带版本检查的
capability resolution。但目前没有类型化桥接从该 registry 生成 Workflow snapshot，因此 adapter 可能
绕过 MCP compatibility、忽略 capability kind，或重复实现 resolution policy。

**Why now / 为什么现在：** Both sides of the contract are dependency-ready and provider-free. A
small bridge closes the nearest C+E integration gap without waiting for provider execution or public
transport, and prevents later Workflow API/UI work from inventing a second capability policy.

契约两端已经依赖就绪且不依赖 provider。一个最小 bridge 可在不等待 provider execution 或 public
transport 的情况下收束最近的 C+E 集成缺口，并防止后续 Workflow API/UI 再造第二套 capability policy。

**Explicit non-goals / 明确非目标：** Do not change `contextlab-mcp` or plugin runtime; do not add
provider calls, secrets, dynamic loading, persistence, scheduling, Workflow binding reads, API,
OpenAPI, SDK, Web, Docker, release, or production behavior. Do not duplicate registry version
resolution or the existing Workflow scheduler.

不修改 `contextlab-mcp` 或 plugin runtime；不新增 provider call、secret、dynamic loading、
persistence、scheduling、Workflow binding read、API、OpenAPI、SDK、Web、Docker、release 或 production
行为；不复制 registry version resolution 或既有 Workflow scheduler。

**Smallest boundary / 最小边界：** Add one crate-local dependency from `contextlab-workflow` to
`contextlab-mcp`, one provider-free bridge module, and focused integration tests. The input wraps the
existing Workflow requirement with an expected MCP `CapabilityKind`; the output is the existing
`WorkflowCapabilitySnapshot`. Stable identifiers and explicit three-part versions cross the boundary,
and canonical ordering is preserved.

仅为 `contextlab-workflow` 增加一个指向 `contextlab-mcp` 的 crate-local dependency、一个
provider-free bridge module 与聚焦 integration test。输入用期望的 MCP `CapabilityKind` 包装既有
Workflow requirement，输出仍是既有 `WorkflowCapabilitySnapshot`。边界只传递稳定 ID 与显式三段式
版本，并保持规范顺序。

**Fresh verification / 新鲜验证：** Before another increment starts, observe RED tests for the
missing bridge, then GREEN results from `cargo test -p contextlab-workflow`,
`cargo clippy -p contextlab-workflow --all-targets -- -D warnings`, and
`cargo fmt --all -- --check`. Broader workspace, provider, transport, browser, PostgreSQL, remote,
release, and production evidence is outside this increment and must not be inferred.

开始下一增量前，必须先观察缺失 bridge 的 RED test，再观察 `cargo test -p contextlab-workflow`、
`cargo clippy -p contextlab-workflow --all-targets -- -D warnings` 与
`cargo fmt --all -- --check` 的 GREEN 结果。不得由此推断 workspace 全量、provider、transport、browser、
PostgreSQL、remote、release 或 production 证据。

## Contract Boundary / 契约边界

- `contextlab-mcp::CapabilityRegistry::resolve_capability` remains the only version-checked plugin
  capability resolver.
- The bridge validates exact capability kind after registry resolution and fails closed on missing,
  incompatible, wrong-kind, or internally conflicting requirements.
- Duplicate requirements for one stable ID are deterministically consolidated only when their kinds
  and major-version families agree; the highest minimum compatible version is resolved.
- The bridge returns the existing Workflow snapshot. Existing definition validation, scheduler,
  replay, status projection, and immutable Context binding behavior remain unchanged.

- `contextlab-mcp::CapabilityRegistry::resolve_capability` 仍是唯一带版本检查的 plugin capability
  resolver。
- bridge 在 registry resolution 后校验精确 capability kind，并对缺失、不兼容、kind 错误或内部冲突的
  requirement 执行 fail closed。
- 同一稳定 ID 的重复 requirement 仅在 kind 与 major-version family 一致时确定性归并，并按最高最低兼容
  版本执行 resolution。
- bridge 返回既有 Workflow snapshot；既有 definition validation、scheduler、replay、status projection
  与不可变 Context binding 行为均不改变。

## Fresh Outcome / 新鲜结果

**RED / 红测：** The first focused run failed at compilation because
`WorkflowPluginCapabilityBridge`, `WorkflowPluginCapabilityBridgeError`, and
`WorkflowPluginCapabilityRequirement` did not exist. No existing implementation accidentally
satisfied the new contract.

首次聚焦运行在编译阶段失败，因为 `WorkflowPluginCapabilityBridge`、
`WorkflowPluginCapabilityBridgeError` 与 `WorkflowPluginCapabilityRequirement` 尚不存在；没有既有实现
意外满足新增契约。

**GREEN / 绿测：** The focused bridge suite passes `7` tests. The complete
`contextlab-workflow` crate passes `25` integration tests with no ignored tests. Strict crate Clippy
and workspace formatting checks pass:

- `cargo test -p contextlab-workflow --test workflow_plugin_capability_bridge`
- `cargo test -p contextlab-workflow`
- `cargo clippy -p contextlab-workflow --all-targets -- -D warnings`
- `cargo fmt --all -- --check`

聚焦 bridge suite 的 `7` 项测试全部通过；完整 `contextlab-workflow` crate 的 `25` 项 integration test
全部通过且无 ignored test。crate strict Clippy 与 workspace 格式检查也均通过。以上是本地源码级证据，
不代表 provider、transport、Docker、remote、release 或 production 已验证。
