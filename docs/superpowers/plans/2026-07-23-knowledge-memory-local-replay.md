# Knowledge Citation-to-Memory Local Replay Bridge / 知识引用到记忆本地回放桥接

## Necessity Record / 必要性记录

### Completion criterion and charter principle / 完成条件与宪章原则

This increment directly advances Completion Criterion 1, Context-first platform coverage, and
Criterion 8, reliable local quality gates. It also implements the architecture's distinct Knowledge
Engine citation and retrieval responsibility together with the Memory Engine's timeline, retention,
and replay responsibility through a reusable Rust-domain boundary rather than UI, transport, or
provider code.

本增量直接推进完成条件 1（以 Context 为核心的平台覆盖）和条件 8（可靠的本地质量门禁）。它也以可复用
的 Rust 领域边界连接架构中彼此独立的 Knowledge Engine 引用与检索职责，以及 Memory Engine 时间线、留存和
回放职责，而不是由 UI、transport 或 provider 代码承担。

### Gap and risk / 缺口与风险

`contextlab-knowledge` already emits deterministic, redacted local citation projections with stable
retrieval and source identifiers. `contextlab-memory` already emits deterministic retention capability
records with stable memory identifiers, timeline versions, retention decisions, and replay state. No
typed local-only bridge currently binds those already-redacted facts. Later Context assembly could
therefore discard provenance, confuse source versions with memory versions, or rebuild ordering and
redaction policy inconsistently.

`contextlab-knowledge` 已能生成确定性、去标识化的本地 citation projection，其中包含稳定的 retrieval 和
source identifier。`contextlab-memory` 已能生成确定性的 retention capability record，其中包含稳定的
memory identifier、timeline version、retention decision 和 replay state。目前没有类型化、本地专用的桥接来
绑定这些已经去标识化的事实。后续 Context 组装可能因此丢失 provenance、混淆 source version 与 memory version，
或以不一致的方式重复实现排序和去标识化策略。

### Why now / 为什么现在

Both core contracts are provider-free, independently tested, and expose the exact stable metadata
needed for a small deterministic composition. This is the next dependency-ready knowledge/memory
increment because it closes the core replay handoff before storage persistence, embedding execution,
plugin/MCP resolution, routes, or a Context editor adds a second cross-boundary representation.

两个核心契约均不依赖 provider、已独立测试，并且暴露了小型确定性组合所需的精确稳定 metadata。该工作是
当前依赖已满足的下一项 knowledge/memory 增量，因为它会在 storage persistence、embedding execution、plugin/MCP
resolution、route 或 Context editor 形成第二套跨边界表示之前，先收束 core replay handoff。

### Explicit non-goals / 明确非目标

No raw document content, memory content, query text or fingerprints, embedding values, provider
configuration, credentials, persistence adapter, migration, route, OpenAPI operation, SDK, Web,
Docker, storage, plugin/MCP change, or retention-policy recalculation. The existing plugin citation
bridge remains unchanged and is not reused as a surrogate for this core-to-core composition.

不新增 raw document content、memory content、query text 或 fingerprint、embedding value、provider
configuration、credential、persistence adapter、migration、route、OpenAPI operation、SDK、Web、Docker、storage、
plugin/MCP 变更，也不重新计算 retention policy。既有 plugin citation bridge 保持不变，且不得将其作为此
core-to-core composition 的替代品。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

Add one crate-local dependency from `contextlab-knowledge` to `contextlab-memory`, a provider-free
bridge module owned by `contextlab-knowledge`, and focused contract tests in the existing knowledge
test boundary. The bridge accepts existing redacted local citation and memory retention capability
records, validates exact explicit schema versions and required source facts, derives a UUID v5 replay
projection identifier, preserves canonical citation ordering, and returns typed fail-closed errors.
This plan is the required bilingual documentation; no roadmap or architecture document changes are
needed for this local increment.

仅为 `contextlab-knowledge` 增加一个指向 `contextlab-memory` 的 crate-local dependency，在
`contextlab-knowledge` 内增加一个 provider-free bridge module，并在既有 knowledge test boundary 中增加
聚焦 contract test。该 bridge 接受已有的去标识化 local citation 和 memory retention capability record，校验精确
的 explicit schema version 与必需 source fact，生成 UUID v5 replay projection identifier，保持 canonical citation
ordering，并返回 typed、fail-closed error。本计划即为所需中英文文档；该 local increment 不需要改动 roadmap 或
architecture 文档。

### Fresh verification required / 所需新鲜验证

Strict TDD evidence must show a focused contract test fail before the bridge exists and pass after the
minimal implementation. Before another increment starts, run `cargo fmt --all`,
`cargo test -p contextlab-knowledge --quiet`, `cargo test -p contextlab-memory --quiet`, and strict
Clippy for both crates with `--all-targets -- -D warnings`.

严格 TDD 证据必须显示：在 bridge 尚不存在时，聚焦 contract test 先失败；完成最小实现后再通过。开始下一增量前，
必须运行 `cargo fmt --all`、`cargo test -p contextlab-knowledge --quiet`、
`cargo test -p contextlab-memory --quiet`，以及两个 crate 的 `--all-targets -- -D warnings` 严格 Clippy。

## Delivery Status / 交付状态

- [x] Add focused failing citation-to-memory replay contract tests / 增加聚焦且先失败的 citation-to-memory replay contract test。
- [x] Implement the minimal provider-free, fail-closed core bridge / 实现最小的 provider-free、fail-closed core bridge。
- [x] Record fresh RED, GREEN, formatting, test, and strict-Clippy evidence / 记录新鲜的 RED、GREEN、format、test 与 strict-Clippy 证据。
- [x] Correct the reviewed projection identity and deserialization invariants / 修复评审发现的 projection identity 与反序列化不变量。

## Reviewed Projection Invariant Root-Cause Record / 评审投影不变量根因记录

### Root cause / 根因

The replay projection UUID bound the citation source, memory identity, scope, timeline version,
capability schema, and policy version, but omitted the serialized retention decision and replay
state. Two projections could therefore serialize different retention outcomes while retaining the
same UUID. The projection also derived `Deserialize`; `deny_unknown_fields` rejected only extra
keys, so inbound records could carry an unsupported bridge schema, noncanonical citation order, or
source facts inconsistent with the serialized UUID.

replay projection UUID 已绑定 citation source、memory identity、scope、timeline version、capability
schema 与 policy version，却遗漏了序列化输出中的 retention decision 和 replay state。因此，两个 retention
outcome 不同的 projection 可能序列化为不同内容，却共享同一个 UUID。该 projection 还直接派生
`Deserialize`；`deny_unknown_fields` 只能拒绝额外字段，无法阻止不受支持的 bridge schema、非 canonical
citation 顺序，或与序列化 UUID 不一致的 source fact 进入领域类型。

### Minimal invariant repair / 最小不变量修复

One central length-prefixed identity encoder now binds every serialized projection fact, including
the complete redacted citation metadata, retention decision, and replay state. Custom
deserialization uses a private deny-unknown-fields wire shape, then requires exact bridge and source
schemas, strictly ascending citation identifiers, a consistent replay-state/retention-decision pair,
and an exact recomputation of the UUID. Valid records still round-trip. No raw query, knowledge body,
memory body, vector, provider configuration, or credential was added.

现在由一个统一的 length-prefixed identity encoder 绑定所有序列化 projection fact，包括完整的已去标识化
citation metadata、retention decision 与 replay state。自定义反序列化先读取私有的
deny-unknown-fields wire shape，再校验精确的 bridge/source schema、严格递增的 citation identifier、相互一致的
replay-state/retention-decision 组合，以及重新计算后完全一致的 UUID。合法记录仍可 round-trip。未新增 raw
query、knowledge body、memory body、vector、provider configuration 或 credential。

## Fresh Evidence / 新鲜证据

- Reviewed-invariant RED: `cargo test -p contextlab-knowledge --test
  knowledge_memory_local_replay_bridge` completed with `3 passed` and `4 failed`. The failures were
  exactly the retention-outcome UUID collision and acceptance of an unknown bridge schema,
  noncanonical citation order, and source facts inconsistent with the UUID. The collision printed
  the same UUID, `f1b767c8-87ca-53ac-a93a-66b60521311d`, for different retention outcomes.
- GREEN: `cargo test -p contextlab-knowledge --test knowledge_memory_local_replay_bridge --quiet`
  completed with `7 passed`, `0 failed`; the deterministic/redacted contract also verifies a valid
  serialize/deserialize round trip.
- Verification: `cargo test -p contextlab-knowledge --quiet` completed all groups with
  `2`, `4`, `7`, and `3` passing tests; `cargo test -p contextlab-memory --quiet` completed groups
  with `2` and `5` passing tests. `cargo fmt -p contextlab-knowledge -- --check`,
  `cargo fmt -p contextlab-memory -- --check`,
  `cargo clippy -p contextlab-knowledge --all-targets -- -D warnings`, and
  `cargo clippy -p contextlab-memory --all-targets -- -D warnings` all passed.
- Rust 1.85: the installed `1.85.0-x86_64-pc-windows-msvc` toolchain ran both crate test suites with
  the same passing group counts.
- Workspace formatter blocker: `cargo fmt --all -- --check` remains red because the unowned
  `crates/workflow/tests/workflow_deterministic_replay.rs:190` and subsequent nearby lines need
  pre-existing formatting-only rewrites. This increment did not modify that path.

- 评审不变量红灯：`cargo test -p contextlab-knowledge --test
  knowledge_memory_local_replay_bridge` 结果为 `3 passed`、`4 failed`。四项失败分别精确对应 retention
  outcome UUID 冲突，以及错误接受未知 bridge schema、非 canonical citation 顺序、与 UUID 不一致的 source
  fact。不同 retention outcome 打印出了相同 UUID：`f1b767c8-87ca-53ac-a93a-66b60521311d`。
- 绿灯：分别完成最小实现后，
  `cargo test -p contextlab-knowledge --test knowledge_memory_local_replay_bridge --quiet` 以
  `7 passed`、`0 failed` 完成；deterministic/redacted contract 同时验证合法记录可完成序列化/反序列化
  round trip。
- 验证：`cargo test -p contextlab-knowledge --quiet` 的各组分别有 `2`、`4`、`7`、`3` 个通过；
  `cargo test -p contextlab-memory --quiet` 的各组分别有 `2`、`5` 个通过。
  `cargo fmt -p contextlab-knowledge -- --check`、`cargo fmt -p contextlab-memory -- --check`、
  `cargo clippy -p contextlab-knowledge --all-targets -- -D warnings` 与
  `cargo clippy -p contextlab-memory --all-targets -- -D warnings` 均已通过。
- Rust 1.85：已安装的 `1.85.0-x86_64-pc-windows-msvc` toolchain 对两个 crate 运行测试，得到相同的通过
  分组计数。
- workspace formatter 阻塞：`cargo fmt --all -- --check` 仍为红灯，因为不属于本增量的
  `crates/workflow/tests/workflow_deterministic_replay.rs:190` 及其后相邻行需要预先存在的仅格式改写。本增量未
  修改该路径。
