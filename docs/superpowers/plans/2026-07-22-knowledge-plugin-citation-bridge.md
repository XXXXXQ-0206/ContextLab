# Knowledge Plugin Citation Bridge / Knowledge Plugin Citation 桥接

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This increment directly
serves the local Knowledge/Retrieval and Plugin/MCP convergence boundary in the Context-first
platform charter. It connects the existing provider-free knowledge citation compatibility contract
to the existing version-checked MCP capability registry without introducing a second registry or a
second retrieval implementation.

本增量直接服务 Context-first 平台宪章中的本地 Knowledge/Retrieval 与 Plugin/MCP 收束边界。它将
既有 provider-free knowledge citation compatibility contract 接入既有、带版本检查的 MCP capability
registry，不新增第二个 registry，也不新增第二套 retrieval 实现。

**Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口：** Knowledge already
produces a redacted `KnowledgeCitationCapability`, while MCP already resolves stable capability IDs
against compatible versions. There is no typed bridge proving that a selected plugin capability is
a readable resource of the required version and that the supplied citation capability satisfies the
consumer's citation/embedding compatibility requirement. A caller could otherwise accidentally
accept a wrong-kind or stale capability before consuming citation metadata.

Knowledge 已能生成脱敏的 `KnowledgeCitationCapability`，MCP 也能按兼容版本解析稳定 capability ID；
但当前没有 typed bridge 可证明所选 plugin capability 是所需版本的 readable resource，且输入的
citation capability 满足 consumer 声明的 citation/embedding compatibility requirement。缺少该边界时，
caller 可能在消费 citation metadata 前误接收 wrong-kind 或 stale capability。

**Why now / 为什么现在优先：** Both source contracts and their deterministic local adapters exist,
so this is the smallest dependency-ready D+E integration slice. It closes a real cross-crate contract
gap without waiting for provider execution, public transport, Web mutation, Docker, or external
release evidence.

两侧 source contract 与 deterministic local adapter 均已存在，因此这是当前依赖就绪的最小 D+E
integration slice。它直接收束真实的跨 crate contract 缺口，无需等待 provider execution、public
transport、Web mutation、Docker 或外部发布证据。

**Explicit non-goals / 明确非目标：** No provider invocation, ingestion or retrieval execution,
raw document/chunk/query/vector output, provider configuration or secret output, dynamic loading,
new registry, public REST/OpenAPI/public SDK/Web surface, persistence change, or production-readiness
claim. The bridge does not alter `GraphDiff::between` or introduce any diff calculator.

不包含 provider invocation、ingestion 或 retrieval execution、raw document/chunk/query/vector 输出、
provider configuration 或 secret 输出、dynamic loading、新 registry、public REST/OpenAPI/public SDK/Web
surface、持久化变更或 production-readiness 声明。桥接不会修改 `GraphDiff::between`，也不引入任何
diff calculator。

**Smallest affected boundary and bilingual docs / 最小受影响边界与双语文档：** Only
`crates/knowledge/**`, its crate-local dependency on `contextlab-mcp`, crate-local tests, and this
bilingual record are in scope. The bridge consumes `CapabilityRegistry`, `CapabilityKind`, `Version`,
and the existing `KnowledgeCitationCapability`; it projects only stable plugin/capability identity,
contract versions, citation compatibility, retrieval identity, and canonical citation metadata.

范围仅包括 `crates/knowledge/**`、其对 `contextlab-mcp` 的 crate-local dependency、crate-local test 与
本双语记录。桥接消费 `CapabilityRegistry`、`CapabilityKind`、`Version` 与既有
`KnowledgeCitationCapability`；输出仅含稳定 plugin/capability identity、contract version、citation
compatibility、retrieval identity 与 canonical citation metadata。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Observe
a focused RED before implementation, then GREEN tests for compatible resolution, missing capability,
incompatible version, wrong kind, incompatible citation metadata, deterministic citation ordering,
and redaction. Run `cargo test -p contextlab-knowledge`,
`cargo clippy -p contextlab-knowledge --all-targets -- -D warnings`, and
`cargo fmt --all -- --check`. Broader runtime, browser, remote, release, operator, and production
evidence remains unobserved or deferred.

实现前先观察聚焦 RED；随后以 GREEN test 覆盖 compatible resolution、missing capability、
incompatible version、wrong kind、incompatible citation metadata、deterministic citation ordering 与
脱敏。运行 `cargo test -p contextlab-knowledge`、
`cargo clippy -p contextlab-knowledge --all-targets -- -D warnings` 与
`cargo fmt --all -- --check`。更广的 runtime、browser、remote、release、operator 与 production
证据仍为 unobserved 或 deferred。

## Status / 状态

- [x] Add focused failing bridge contract tests. / 新增聚焦失败的 bridge contract tests。
- [x] Implement the provider-free fail-closed bridge. / 实现 provider-free、fail-closed bridge。
- [x] Record focused tests and strict Clippy evidence. / 记录聚焦测试与 strict Clippy 证据。
- [x] Observe the workspace-wide format check after the parallel Workflow owner formats its files. /
  在并行 Workflow owner 格式化其文件后观察 workspace-wide format check。

## Evidence / 证据

- `RED` / 红灯：`cargo test -p contextlab-knowledge --test knowledge_plugin_citation_bridge
  --no-run` failed with `E0432` because `KnowledgePluginCitationBridge`,
  `KnowledgePluginCitationBridgeError`, and `KnowledgePluginCitationRequirement` did not exist.
  This was the expected pre-implementation contract failure. / 该命令因三个待实现 bridge type 不存在而以
  `E0432` 失败，属于预期的实现前契约红灯。
- `GREEN` / 绿灯：`cargo test -p contextlab-knowledge --test
  knowledge_plugin_citation_bridge -- --nocapture` returned `3 passed, 0 failed`. / 返回
  `3 passed, 0 failed`。
- `HARDENING RED/GREEN` / 脱敏强化红绿：the first recursive serialization-key test returned
  `2 passed, 1 failed` because its draft rule incorrectly rejected the allowed stable
  `content_fingerprint` citation field. The rule was narrowed to actual private payload keys while
  continuing to reject query/query-fingerprint, body/chunk text, vectors, secrets, and credentials;
  the focused test then returned `3 passed, 0 failed`. / 首次递归序列化字段测试返回
  `2 passed, 1 failed`，原因是草案规则误拒绝了允许的稳定 citation 字段 `content_fingerprint`；
  规则收窄为真实私密 payload 字段后，仍拒绝 query/query-fingerprint、正文/chunk text、vector、
  secret 与 credential，聚焦测试恢复为 `3 passed, 0 failed`。
- `GREEN` / 绿灯：`cargo test -p contextlab-knowledge --quiet` returned three groups totaling
  `9 passed, 0 failed`. / 三组测试合计 `9 passed, 0 failed`。
- `GREEN` / 绿灯：`cargo clippy -p contextlab-knowledge --all-targets -- -D warnings` passed. /
  strict Clippy 通过。
- `GREEN` / 绿灯：`cargo fmt -p contextlab-knowledge -- --check` passed. / Knowledge crate
  格式检查通过。
- `GREEN` / 绿灯：a fresh `cargo fmt --all -- --check` passed after the parallel Workflow owner
  formatted its files. No Workflow file was changed by this increment. / 并行 Workflow owner 完成其
  文件格式化后，新鲜的 `cargo fmt --all -- --check` 通过；本增量未修改 Workflow 文件。

The bridge is locally validated for the named D+E boundary. It does not constitute public transport,
provider execution, release, or production-readiness evidence, and it does not close ContextLab's
long-term goal.

该桥接已针对命名的 D+E 边界完成本地验证；它不构成 public transport、provider execution、release
或 production-readiness 证据，也不关闭 ContextLab 长期目标。
