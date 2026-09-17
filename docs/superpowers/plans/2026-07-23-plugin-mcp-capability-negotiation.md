# Plugin-MCP Capability Negotiation / Plugin-MCP 能力协商

## Necessity Record / 必要性记录

**Completion criterion and charter principle / 完成条件与宪章原则：** This increment advances
Criterion 7, provider and plugin extensibility, and Criterion 9, bilingual documentation. It
preserves the charter requirement that reusable Rust contracts own extensibility policy, rather
than lifecycle factories, adapters, or future transports.

本增量推进条件 7“Provider 与 plugin 可扩展性”和条件 9“中英双语文档”。它遵守项目宪章：可复用的
Rust 契约必须承担 extensibility policy，而不是 lifecycle factory、adapter 或未来 transport。

**Gap and risk / 缺口与风险：** `contextlab-mcp` validates one plugin manifest and canonicalizes
its capability list, while `contextlab-plugin-runtime` loads an already validated manifest. There
is no typed, versioned local contract that proves an MCP server description declares exactly the
same stable capabilities before a future bridge uses it. A later adapter could silently accept a
malformed description, compatible-but-not-identical version, omitted capability, or reordered
duplicate declaration.

`contextlab-mcp` 目前校验单个 plugin manifest 并规范化其 capability list，
`contextlab-plugin-runtime` 则加载已经校验过的 manifest。当前没有类型化、带版本的本地契约，能在未来
bridge 使用 MCP server description 前证明其声明了完全相同的稳定 capability。因此后续 adapter 可能
静默接受格式错误的 description、仅兼容但不完全相同的 version、遗漏 capability 或重排的重复声明。

**Why now / 为什么现在：** The manifest, stable identifier, canonical ordering, and fail-closed
lifecycle contracts already exist and need no provider, transport, or runtime dependency. A pure
descriptor parser plus negotiation result is the smallest dependency-ready increment that freezes
the cross-contract rule before dynamic MCP integration is considered.

manifest、稳定 identifier、规范顺序与 fail-closed lifecycle contract 均已存在，且不依赖 provider、
transport 或 runtime。纯 descriptor parser 加 negotiation result 是当前最小、依赖已就绪的增量，可在
考虑 dynamic MCP integration 前冻结跨契约规则。

**Explicit non-goals / 明确非目标：** Do not change workspace manifests, workflow, knowledge,
API, SDK, Web, CLI, Desktop, storage, or roadmap documents. Do not add dynamic loading, plugin
execution, provider calls, network I/O, public transport, Docker, persistence, secrets, or server
processes. Do not alter `PluginBundle::new`, `PluginRuntime::load_all`, the capability registry,
or existing bridge behavior.

不修改 workspace manifest、workflow、knowledge、API、SDK、Web、CLI、Desktop、storage 或 roadmap
文档。不新增 dynamic loading、plugin execution、provider call、network I/O、public transport、Docker、
persistence、secret 或 server process。不改变 `PluginBundle::new`、`PluginRuntime::load_all`、
capability registry 或既有 bridge behavior。

**Smallest boundary / 最小边界：** `contextlab-mcp` owns a strict V1 MCP server capability
descriptor with stable `CapabilityId` values, exact three-part capability versions, canonical
identifier ordering, and parser validation. `contextlab-plugin-runtime` owns a pure negotiation
report that accepts a manifest and raw descriptor description, returns no negotiated capabilities
on any diagnostic, and exposes structured diagnostic code, source, and optional capability ID.
The runtime lifecycle API remains additive and unchanged.

最小边界是：`contextlab-mcp` 拥有严格的 V1 MCP server capability descriptor，包含稳定的
`CapabilityId`、精确三段式 capability version、按 identifier 的规范顺序与 parser validation。
`contextlab-plugin-runtime` 拥有纯 negotiation report：它接受 manifest 与原始 descriptor description，
任一 diagnostic 存在时不返回任何 negotiated capability，并暴露结构化 diagnostic code、source 与可选
capability ID。runtime lifecycle API 仅新增能力且保持不变。

**Fresh verification / 新鲜验证：** Before another increment begins, observe the focused RED
contract test before the new types exist, then run focused MCP and runtime regression tests,
`cargo clippy -p contextlab-mcp --all-targets -- -D warnings`,
`cargo clippy -p contextlab-plugin-runtime --all-targets -- -D warnings`, and
`cargo fmt --all -- --check`. This is local source evidence only; workspace-wide, provider,
transport, Docker, browser, remote, release, and production evidence remain out of scope.

开始下一增量前，必须先观察新类型尚不存在时的聚焦 RED contract test，再运行聚焦 MCP 与 runtime
回归测试、`cargo clippy -p contextlab-mcp --all-targets -- -D warnings`、
`cargo clippy -p contextlab-plugin-runtime --all-targets -- -D warnings` 以及
`cargo fmt --all -- --check`。这只是本地源码证据；workspace 全量、provider、transport、Docker、
browser、remote、release 与 production 证据仍不在本增量范围内。

## Contract Boundary / 契约边界

- Descriptor schema version is exactly `1.0.0`; unknown fields, malformed stable IDs, duplicate
  capability IDs, and any other schema version are rejected.
- Negotiation succeeds only when descriptor plugin ID, capability IDs, kinds, and three-part
  versions exactly equal the manifest declaration. Compatible-major or lower versions do not
  satisfy this contract.
- Both descriptor and successful negotiation output are sorted by stable capability ID. Any
  malformed, unsupported, or mismatched raw description returns a deterministic structured
  diagnostic and an empty negotiated capability list.
- Existing manifest discovery, registry resolution, availability projection, `PluginBundle`, and
  `PluginRuntime` lifecycle semantics are not invoked or modified by negotiation.

- descriptor schema version 必须精确为 `1.0.0`；unknown field、格式错误的稳定 ID、重复 capability ID
  以及其他 schema version 均被拒绝。
- 仅当 descriptor 的 plugin ID、capability ID、kind 与三段式 version 和 manifest 声明完全相等时，
  negotiation 才成功。仅 major 兼容或更低的 version 不能满足本契约。
- descriptor 与成功 negotiation output 都按稳定 capability ID 排序。任一格式错误、不支持或不匹配的
  raw description 都返回确定性的结构化 diagnostic 与空 negotiated capability list。
- negotiation 不调用也不修改既有 manifest discovery、registry resolution、availability projection、
  `PluginBundle` 与 `PluginRuntime` lifecycle semantics。

## Integration-review root cause / 集成审阅根因

Independent review found that malformed identifier values were embedded in parser errors and then
copied into the supposedly safe negotiation diagnostic. An attacker-controlled descriptor could
therefore retain a credential-like marker or local path in diagnostic output. This blocks the safe,
redacted diagnostic boundary. The minimum repair is a fixed diagnostic message for malformed wire
input while preserving the existing structured code and local source label; a focused regression
must prove that an invalid identifier marker is not echoed.

独立审阅发现，非法 identifier 值会被写入 parser error，随后又被复制进 supposedly-safe negotiation
diagnostic。攻击者控制的 descriptor 因而可能在诊断输出中保留类似 credential 的标记或本地路径，阻塞
安全脱敏的诊断边界。最小修复是在保留既有 structured code 与本地 source label 的同时，对 malformed wire
input 使用固定诊断文案；聚焦回归必须证明非法 identifier 标记不会被回显。

## TDD Evidence / TDD 证据

- [x] RED: focused contract suite imports the missing descriptor and negotiation APIs.
- [x] GREEN: minimum parser and negotiation implementation passes focused malformed, unsupported,
  mismatch, exact-match, canonical-order, and lifecycle-regression tests.
- [x] VERIFY: focused crate tests, full owned-crate regressions, strict owned-crate Clippy, and scoped
  owned-crate format checks pass. The workspace-wide format check is blocked by unrelated files.

- [x] 红测：聚焦 contract suite 导入尚不存在的 descriptor 与 negotiation API。
- [x] 绿测：最小 parser 与 negotiation 实现通过格式错误、不支持、mismatch、exact-match、
  canonical-order 与 lifecycle-regression test。
- [x] 验证：聚焦 crate test、owned crate 全量回归、严格 owned-crate Clippy 与 scoped owned-crate
  format check 通过；workspace 全量 format check 被无关文件阻塞。

## Fresh Outcome / 新鲜结果

**RED / 红测：** `cargo test -p contextlab-mcp --test mcp_server_capability_descriptor
--no-run` failed with `E0432` for the missing descriptor type, parse error, and V1 constant.
`cargo test -p contextlab-plugin-runtime --test capability_negotiation --no-run` independently
failed with `E0432` for the missing negotiator and diagnostic code. These were the expected
pre-implementation contract failures.

MCP 聚焦命令因 descriptor type、parse error 与 V1 constant 尚不存在而以 `E0432` 失败；runtime
聚焦命令也因 negotiator 与 diagnostic code 尚不存在而独立以 `E0432` 失败。这些都是预期的实现前契约
失败。

**GREEN and regressions / 绿测与回归：** The focused MCP suite passes `3` tests and the focused
runtime negotiation suite passes `5` tests, including the untrusted-identifier redaction regression.
The complete `contextlab-mcp` crate passes `8` integration tests, and the complete
`contextlab-plugin-runtime` crate passes `9` integration tests. Existing manifest
discovery, registry projection, and all four lifecycle runtime tests remain green. Strict Clippy and
scoped format checks pass for both owned crates.

MCP 聚焦 suite 的 `3` 项测试与 runtime negotiation suite 的 `5` 项测试均通过，其中包括 untrusted
identifier 脱敏回归。完整 `contextlab-mcp` crate 通过 `8` 项 integration test，完整
`contextlab-plugin-runtime` crate 通过 `9` 项 integration test。既有 manifest discovery、registry projection 与全部 `4` 项 lifecycle runtime
test 保持通过。两个 owned crate 的严格 Clippy 与 scoped format check 均通过。

**Blocked broad evidence / 被阻塞的宽范围证据：** `cargo fmt --all -- --check` remains blocked
by pre-existing formatting drift outside this worker's ownership in CLI adapter-contract,
diff-engine, evaluation, knowledge, and workflow files. No out-of-scope file was reformatted. This
does not weaken the two owned-crate format receipts and is not a provider, transport, release, or
production claim.

`cargo fmt --all -- --check` 仍被本 worker 所有权范围之外的既有格式漂移阻塞，涉及 CLI
adapter-contract、diff-engine、evaluation、knowledge 与 workflow 文件。本增量未格式化任何越界文件。
这不削弱两个 owned crate 的 format 回执，也不构成 provider、transport、release 或 production 声明。
