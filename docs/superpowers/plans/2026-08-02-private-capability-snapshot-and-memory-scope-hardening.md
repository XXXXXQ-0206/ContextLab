# ContextLab Private Capability Snapshot and Memory Scope Hardening
# ContextLab 私有能力快照与 Memory Scope 硬化

**Status / 状态:** completed / verified locally / 已完成，本地验证

This plan records two independently owned, provider-free contract hardening increments discovered
by bounded `gpt-5.6-luna` workers. The Integration Lead accepts either change only after reviewing
its patch and obtaining fresh workspace evidence. Neither increment changes the public API surface.

本计划记录两个由有界 `gpt-5.6-luna` worker 发现的、互不重叠且 provider-free 的 contract hardening 增量。
Integration Lead 只有在审查 patch 并取得新鲜 workspace 证据后才接受任一变更。两项增量都不改变 public API surface。

## Necessity Record A: Immutable Workflow/Plugin Snapshot Binding
## 必要性记录 A：Workflow/Plugin 不可变快照绑定

**Criterion and charter principle / 条件与章程原则:**

- Criterion 7, Provider and plugin extensibility: Workflow must resolve plugin capabilities through a
  reusable, versioned, fail-closed boundary without coupling scheduling decisions to a live registry.
- Criterion 5 and the clean-architecture rule: domain consumers use an owned contract and do not
  recreate registry policy in adapters or screens.
- 条件 7（Provider 与 plugin 可扩展性）：Workflow 必须通过可复用、版本化、fail-closed 的边界解析 plugin
  capability，不能让 scheduling decision 依赖 live registry。
- 条件 5 与 clean architecture 原则：domain consumer 使用所属 contract，adapter 或 screen 不重复实现 registry policy。

**Gap, risk, and missing evidence / 缺口、风险与缺失证据:**

The existing bridge accepted a mutable `CapabilityRegistry`; a later registration could change a
repeated resolution. `CapabilityRegistrySnapshotV1` already owns an immutable clone and a secret-free
fingerprint, but Workflow had no consumer entry point. Without the entry point, capability resolution
was not reproducible at the scheduling boundary.

既有 bridge 接受可变 `CapabilityRegistry`，后续 registration 可能改变重复 resolution。虽然
`CapabilityRegistrySnapshotV1` 已拥有不可变 clone 与无 secret fingerprint，但 Workflow 没有 consumer 入口。
缺少该入口时 scheduling boundary 的 capability resolution 不具备可重现性。

**Why now / 为什么现在:**

The MCP snapshot contract and Workflow capability bridge are already present and independently tested;
this is the smallest dependency-ready repair that closes a named extensibility boundary before any
workflow execution or transport work.

MCP snapshot contract 与 Workflow capability bridge 已存在且各自有测试；这是在推进 workflow execution 或 transport
之前，收束具名 extensibility boundary 的最小依赖就绪修复。

**Non-goals / 非目标:**

No dynamic loading, provider calls, scheduler/execution producer, registry mutation, public REST,
OpenAPI/SDK method, Web mutation, operator transport, migration, secret access, or second
`GraphDiff` calculator.

不做 dynamic loading、provider call、scheduler/execution producer、registry mutation、public REST、OpenAPI/SDK method、
Web mutation、operator transport、migration、secret access 或第二个 `GraphDiff` calculator。

**Minimal boundary and bilingual documentation / 最小边界与双语文档:**

Only `crates/workflow/src/plugin_capability_bridge.rs`, its focused test, this plan, and the existing
Wave 3 capability-bridge architecture record are in scope. The bridge accepts the existing immutable
V1 snapshot and preserves the existing live-registry compatibility helper.

范围仅包括 `crates/workflow/src/plugin_capability_bridge.rs`、其 focused test、本计划与既有 Wave 3 capability-bridge 架构记录。
bridge 接受既有 immutable V1 snapshot，同时保留现有 live-registry compatibility helper。

**Fresh verification before the next increment / 下一增量前的新鲜验证:**

Run focused Workflow/MCP tests, storage/evaluation workspace tests as applicable, `cargo fmt`, strict
offline Clippy, locked Rust check, the full workspace test, and the static single-`GraphDiff` boundary
check. Record unavailable Docker/PostgreSQL, browser, Git, remote, operator, release, and production
evidence honestly.

下一增量前运行 Workflow/MCP focused tests、适用的 storage/evaluation workspace tests、`cargo fmt`、strict offline Clippy、
锁定 Rust check、workspace test 与 single-`GraphDiff` 静态边界检查；Docker/PostgreSQL、browser、Git、remote、operator、release
与 production evidence 若不可用则如实记录。

## Necessity Record B: Memory Projection Scope Integrity
## 必要性记录 B：Memory Projection Scope 完整性

**Criterion and charter principle / 条件与章程原则:**

- Criterion 1, Context-first platform coverage: exact Context/commit-scoped memory evidence must
  share the same persistence scope as Context Graph and version history.
- Criterion 6, production security and collaboration: fail closed on foreign or unknown scope rather
  than persisting a projection that PostgreSQL would reject through its composite foreign keys.
- 条件 1（Context-first 平台覆盖）：exact Context/commit-scoped memory evidence 必须与 Context Graph 和版本历史共享同一持久化 scope。
- 条件 6（生产安全与协作）：对 foreign 或 unknown scope fail closed，不能持久化 PostgreSQL 复合外键会拒绝的 projection。

**Gap, risk, and missing evidence / 缺口、风险与缺失证据:**

The in-memory Memory repository could accept an unknown project/context/commit scope, diverging from
the PostgreSQL composite foreign-key contract. This allowed invalid local evidence to look persisted
and made adapter behavior non-portable.

内存 Memory repository 可能接受未知 project/context/commit scope，与 PostgreSQL 复合外键 contract 漂移。
这会让非法本地 evidence 看起来像已持久化，也使 adapter behavior 不可移植。

**Why now / 为什么现在:**

The existing Knowledge/Memory projection and replay path is dependency-ready; the worker found a
concrete contract defect during the parallel audit. Fixing it now is smaller and more necessary than
adding another read surface or benchmark feature.

既有 Knowledge/Memory projection 与 replay path 依赖已就绪；并行审计发现了具体 contract defect。
现在修复它比增加新的 read surface 或 benchmark feature 更小、更必要。

**Non-goals / 非目标:**

No knowledge ingestion, embedding provider, retrieval service, raw private content, new route,
OpenAPI/SDK method, Web mutation, migration, secret access, Docker/PostgreSQL claim, or GraphDiff change.

不做 knowledge ingestion、embedding provider、retrieval service、raw private content、新 route、OpenAPI/SDK method、Web mutation、
migration、secret access、Docker/PostgreSQL 声明或 GraphDiff 变更。

**Minimal boundary and bilingual documentation / 最小边界与双语文档:**

Only `crates/storage/src/memory.rs`, `crates/storage/src/knowledge_memory_projection.rs`, their focused
regression test, this plan, and the existing Knowledge/Memory architecture record are in scope.
The in-memory repository now applies the same exact-scope precondition expected by PostgreSQL.

范围仅包括 `crates/storage/src/memory.rs`、`crates/storage/src/knowledge_memory_projection.rs`、其 focused regression test、本计划与既有
Knowledge/Memory 架构记录。内存 repository 现执行与 PostgreSQL 相同的 exact-scope 前置条件。

**Fresh verification before the next increment / 下一增量前的新鲜验证:**

Run the focused storage/embedding tests, full Rust workspace tests, format, strict offline Clippy,
locked Rust check, Web checks, and the local contract verifier. PostgreSQL runtime remains unobserved
unless a real disposable database is available; no local result may impersonate that evidence.

下一增量前运行 storage/embedding focused tests、Rust workspace tests、format、strict offline Clippy、锁定 Rust check、Web checks 与 local
contract verifier。除非真实 disposable database 可用，PostgreSQL runtime 继续为 unobserved；任何 local result 都不得冒充该证据。

## Checklist / 清单

- [x] Bounded Luna workers used disjoint ownership / 使用有界 Luna worker 且 ownership 不重叠
- [x] Review both patches in the Integration Lead worktree / Integration Lead 审查两项 patch
- [x] Run fresh focused and workspace verification / 运行新鲜 focused 与 workspace 验证
- [x] Add bilingual architecture and roadmap receipts / 增加双语架构与路线图回执
- [x] Keep the long-term goal active and select the next dependency-ready increment / 保持长期目标 active 并选择下一项依赖就绪增量

## Completion Receipt / 完成回执

The Workflow/Plugin track added the immutable snapshot consumer and retained the live-registry
compatibility helper. The Knowledge/Memory track aligned the in-memory Context Graph repository
with the PostgreSQL composite scope precondition. Both tracks remain private, provider-free, and
read/contract-only.

Workflow/Plugin track 增加了 immutable snapshot consumer，同时保留 live-registry compatibility helper。Knowledge/Memory
track 让内存 Context Graph repository 与 PostgreSQL composite scope precondition 对齐。两条 track 均保持 private、provider-free，
且只涉及 read/contract boundary。

Fresh evidence / 新鲜证据：Workflow bridge `8 passed`; MCP `10 passed`; Knowledge/Memory projection `3 passed`; embedding
`6 passed`; storage `222 passed, 41 ignored`; full workspace Rust; strict offline Clippy; locked Rust `1.85.0` check;
`cargo fmt --all -- --check`; `pnpm check:web` with public SDK `15`, Web `298`, and production build; local contract fixture
verifier; and `GRAPH_DIFF_IMPL_COUNT=1` all passed. PostgreSQL runtime, Docker, authenticated browser/visual smoke, Git,
remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

新鲜证据：Workflow bridge `8 passed`、MCP `10 passed`、Knowledge/Memory projection `3 passed`、embedding `6 passed`、storage
`222 passed, 41 ignored`、workspace Rust、strict offline Clippy、锁定 Rust `1.85.0` check、`cargo fmt --all -- --check`、
`pnpm check:web`（public SDK `15`、Web `298` 与 production build）、local contract fixture verifier 以及
`GRAPH_DIFF_IMPL_COUNT=1` 全部通过。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git、remote CI、
operator rehearsal、release 与 production 继续为 `unobserved` 或 `deferred`。

This receipt advances the named local boundaries but does not close any repository convergence
condition or the long-term goal. The next implementation requires a new bilingual Necessity Record
after a fresh open-criteria audit; no public write, public transport, or external-release review is
admitted by this plan.

本回执推进已命名的本地边界，但不关闭任何 repository convergence condition 或长期目标。下一项 implementation 必须在新鲜开放条件审计
后新增双语 Necessity Record；本计划不准入 public write、public transport 或 external-release review。
