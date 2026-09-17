# Private Cross-Domain Capability Composition / 私有跨域能力组合

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与章程原则

- **Criteria 1, 5, 7, and 9 / 条件 1、5、7 与 9:** Context bindings, Workflow capability
  requirements, Knowledge/Memory citation replay, and plugin capability negotiation must share
  deterministic versioned Rust contracts rather than independently inferring availability.
  / Context binding、Workflow capability requirement、Knowledge/Memory citation replay 与 plugin
  capability negotiation 必须共享确定性的版本化 Rust contract，而不能各自推断能力可用性。
- **Context-first reusable core / Context-first 可复用核心：** A private Context projection may
  consume only a validated, provider-free capability snapshot and must not expose raw plugin,
  citation, memory, or provider payloads.
  / 私有 Context projection 只能消费经过校验的 provider-free capability snapshot，不能暴露 raw
  plugin、citation、memory 或 provider payload。

### Gap, dependencies, and evidence / 缺口、依赖与证据

The existing `WorkflowPluginCapabilityBridge` and `KnowledgePluginCitationBridge` each validate
their own registry requirements, while `KnowledgeMemoryReplayBridge` validates the replay side.
The repository has no cross-domain contract proving that one exact, immutable capability registry
snapshot can satisfy both a Workflow binding and a Knowledge/Memory citation replay projection,
with deterministic ordering and fail-closed behavior for a missing capability, wrong kind, major
version conflict, or incompatible citation schema. The existing Rust bridges, stable identifiers,
and provider-free fixtures are already available; no external runtime is required.

现有 `WorkflowPluginCapabilityBridge` 与 `KnowledgePluginCitationBridge` 分别校验各自的 registry
requirement，`KnowledgeMemoryReplayBridge` 则校验 replay 侧。仓库当前没有跨域 contract 证明同一份精确、不可变
capability registry snapshot 能同时满足 Workflow binding 与 Knowledge/Memory citation replay projection，
并在缺少 capability、类别错误、major version 冲突或 citation schema 不兼容时确定性 fail closed。既有 Rust bridge、
稳定标识与 provider-free fixture 已就绪；不依赖外部 runtime。

### Why now / 为何现在优先

The latest local receipts already cover commit-bound ContextGraph review, benchmark decision
workspace breadth, private Workflow read, lifecycle metadata propagation, and plugin/Knowledge
read adapters. The next unresolved local convergence risk is the boundary between those existing
read contracts. Closing this narrow composition evidence gap is more necessary than adding another
screen or transport and follows the roadmap's explicit Workflow/Plugin and Knowledge/Memory bridge
queue.

最近的本地回执已经覆盖 commit-bound ContextGraph review、benchmark decision workspace breadth、私有 Workflow read、
lifecycle metadata propagation 以及 Plugin/Knowledge read adapter。当前未收束的本地 convergence 风险是这些既有 read
contract 之间的边界。收束这一狭窄的组合证据缺口，比增加新的页面或 transport 更必要，也符合路线图明确列出的
Workflow/Plugin 与 Knowledge/Memory bridge 队列。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK write, operator transport, Web mutation, execution producer,
  provider/network call, secret access, migration, or production claim.
  / 不新增 public REST/OpenAPI/public SDK write、operator transport、Web mutation、execution producer、
  provider/network call、secret access、migration 或 production 声明。
- No second graph-diff calculator, benchmark policy calculator, or client-side policy inference.
  / 不新增第二个 graph-diff calculator、benchmark policy calculator 或客户端策略推断。
- No raw plugin descriptor, citation source body, memory body, query, vector, or provider output
  may cross the composition boundary.
  / raw plugin descriptor、citation source body、memory body、query、vector 或 provider output 不得穿过组合边界。

### Smallest boundary and ownership / 最小边界与所有权

- **Workflow/Plugin owner / Workflow/Plugin 所有者:** `crates/workflow/src/plugin_capability_bridge.rs`
  and its focused contract test only.
  / 仅负责 `crates/workflow/src/plugin_capability_bridge.rs` 及其聚焦 contract test。
- **Knowledge/Memory owner / Knowledge/Memory 所有者:** `crates/knowledge/src/bridge.rs`,
  `crates/knowledge/src/memory_replay.rs`, and their focused contract test only.
  / 仅负责 `crates/knowledge/src/bridge.rs`、`crates/knowledge/src/memory_replay.rs` 及其聚焦 contract test。
- **Integration Lead / 集成负责人:** this plan, shared fixture review, docs/roadmap receipts,
  any adapter wiring outside the owned files, and final cross-workspace verification.
  / 本计划、shared fixture 审查、docs/roadmap 回执、ownership 之外的 adapter wiring 与最终全 workspace 验证由集成负责人负责。

### Fresh verification required / 所需新鲜验证

Observe a red cross-domain assertion before implementation. Then observe green focused tests for
deterministic shared-registry composition, exact version/kind/schema rejection, redaction, and
replay identity. Run workspace Rust tests, format, strict offline Clippy, locked Rust `1.85.0`
check, the scoped local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. Docker/PostgreSQL,
authenticated browser, Git, remote CI, operator rehearsal, release, and production remain
explicitly unobserved or deferred.

实现前必须先观测跨域 red assertion。随后必须观测 green focused tests，覆盖 shared registry 组合的确定性、精确
version/kind/schema rejection、脱敏与 replay identity。运行 workspace Rust tests、format、strict offline Clippy、锁定
Rust `1.85.0` check、范围化 local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。Docker/PostgreSQL、authenticated
browser、Git、remote CI、operator rehearsal、release 与 production 明确继续为 unobserved 或 deferred。

## Execution Checklist / 执行清单

- [x] Add the red cross-domain composition contract before production edits.
  / 在修改 production code 前增加跨域组合 red contract。
- [x] Implement the smallest provider-free, fail-closed composition change within ownership.
  / 在 ownership 范围内实现最小 provider-free、fail-closed 组合修复。
- [x] Run fresh cross-stack verification and record the evidence without closing the long-term goal.
  / 运行新鲜 cross-stack verification 并记录证据，不关闭长期目标。

## Fresh Verification / 新鲜验证

The red phase compiled the new mcp contract test before the snapshot existed and failed on the
missing `CapabilityRegistrySnapshotV1`, schema constant, and `CapabilityRegistry::snapshot()`.
The green phase added the immutable cloned registry, canonical availability projection, explicit V1
schema, and deterministic SHA-256 fingerprint. A server/API integration fixture then consumed the
same snapshot through the existing Workflow and Knowledge bridges and proved deterministic scope,
version/kind checks, and raw-content redaction.

红阶段在 snapshot 尚不存在时编译新的 mcp contract test，真实失败于缺少
`CapabilityRegistrySnapshotV1`、schema constant 与 `CapabilityRegistry::snapshot()`。绿阶段增加不可变 cloned registry、canonical
availability projection、显式 V1 schema 与确定性 SHA-256 fingerprint。随后 server/API integration fixture 使用同一 snapshot
通过既有 Workflow 与 Knowledge bridge，证明确定性 scope、version/kind check 与 raw-content 脱敏。

Additional independent red/green receipts:

- Knowledge/Memory replay identity: red identity regression, then `14 passed` and package tests passed;
  the stable `MemoryRetentionCapabilityId` is now part of the UUIDv5 identity.
- CLI/Desktop: red response/request integration drift, then adapter `8`, CLI `5`, and Desktop staging
  `5 passed`; the typed `AdapterResponseError` is fail-closed and exact integration identity is
  carried into presentation.
- Web: red unknown/raw field acceptance, then focused parser coverage and `pnpm check:web` with Web
  `282 passed`, TypeScript/lint, and production build passed.
- Workflow/Plugin and Diff/versioning audits found no defensible defect; existing focused contracts
  passed without speculative edits. Evaluation audit likewise found existing deterministic,
  scope-closed workspace projection coverage and made no change.

Knowledge/Memory replay identity：红 regression 后 `14 passed` 且 package tests 通过；稳定的
`MemoryRetentionCapabilityId` 现纳入 UUIDv5 identity。CLI/Desktop：红 response/request integration drift 后 adapter `8`、CLI `5`、Desktop staging
`5 passed`；typed `AdapterResponseError` fail-closed，并将 exact integration identity 传到 presentation。Web：红 unknown/raw field acceptance 后，focused parser 与
`pnpm check:web` 通过，Web `282 passed`、TypeScript/lint 与 production build 均通过。Workflow/Plugin、Diff/versioning 与 Evaluation 审查均未发现可 defensibly 修复的缺陷，未做 speculative edit。

Dispatch evidence / 调度证据：the first Knowledge/Memory, Evaluation, CLI/Desktop, and Web
workers returned transport `502 Bad Gateway` without product output; they were closed and their
ownerships were immediately reassigned to replacement `gpt-5.6-luna` workers. The replacement
workers returned the evidence above. This is execution provenance, not a product blocker.

首轮 Knowledge/Memory、Evaluation、CLI/Desktop 与 Web worker 因 transport `502 Bad Gateway` 未产生产品输出；它们已关闭，并立即由 replacement `gpt-5.6-luna` worker 接管相同 ownership。replacement 已返回上述证据。这属于执行 provenance，不是产品阻塞。

No public REST/OpenAPI/public SDK write, operator transport, Web mutation, execution producer,
provider, migration, secret access, second GraphDiff calculator, Docker/PostgreSQL runtime,
authenticated browser/visual smoke, Git change-set, remote CI, operator rehearsal, release, or
production claim was added. The long-term goal remains active; the next increment still requires
a new bilingual Necessity Record.

未新增 public REST/OpenAPI/public SDK write、operator transport、Web mutation、execution producer、provider、migration、secret access、第二个 GraphDiff calculator、Docker/PostgreSQL runtime、authenticated browser/visual smoke、Git change-set、remote CI、operator rehearsal、release 或 production 声明。长期目标保持 active；下一项增量仍必须先新增双语 Necessity Record。

Final local gate / 最终本地门禁：`cargo fmt --all -- --check` passed; `cargo test --workspace --quiet --no-fail-fast --offline` passed with storage `212 passed, 39 ignored`; strict offline Clippy passed; `cargo +1.85.0 check --workspace --all-targets --locked --offline` passed; the local verifier fixture passed; and `GRAPH_DIFF_IMPL_COUNT=1`. The latest `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `282`, TypeScript/lint, and production build. / `cargo fmt --all -- --check`、workspace Rust（storage `212 passed, 39 ignored`）、strict offline Clippy、锁定 Rust `1.85.0`、local verifier fixture 与 `GRAPH_DIFF_IMPL_COUNT=1` 均通过；最新 `pnpm check:web` 通过（public SDK `15`、local SDK `135`、Web `282`、TypeScript/lint 与 production build）。
