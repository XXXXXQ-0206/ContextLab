# Private Merge-Base and Conflict Contract / 私有 Merge-Base 与 Conflict 契约

**Status / 状态:** completed and verified locally / 已完成，已取得本地验证

## Necessity Record / 必要性记录

### Completion criterion and charter principle / 完成条件与章程原则

This increment directly advances the replayable version-history criterion: branch merge and
rollback cannot be safely admitted while the reusable Rust core has no deterministic merge-base or
ancestry-conflict contract. It also preserves the charter rules that Context identity, version
semantics, validation, and replay decisions belong in framework-independent Rust.

本增量直接推进可回放版本历史条件：在可复用 Rust core 尚无确定性 merge-base 或 ancestry-conflict 契约前，不能
安全准入 branch merge 与 rollback。它同时维护 Context identity、version semantics、validation 与 replay decision
必须位于 framework-independent Rust 中的宪章原则。

### Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口

Typed branch-head discovery is now green, but `ContextCommit` only exposes parent identifiers and
the guarded writer intentionally accepts normal single-parent commits. There is no reusable
contract for validating a finite commit DAG, rejecting missing/cross-Context/cyclic ancestry, or
deterministically classifying identical, fast-forward, three-way, and unresolved merge bases.
Without this boundary, a future merge writer or graph review adapter could invent incompatible
ancestry rules or silently treat ambiguous history as safe.

typed branch-head discovery 已通过，但 `ContextCommit` 目前只暴露 parent identifier，且 guarded writer 有意只接受
normal single-parent commit。当前没有可复用契约来验证 finite commit DAG、拒绝 missing/cross-Context/cyclic ancestry，
或确定性区分 identical、fast-forward、three-way 与 unresolved merge base。缺少该边界，未来 merge writer 或 graph
review adapter 可能各自发明不兼容的 ancestry rule，或将 ambiguous history 静默视为安全。

### Why now / 为什么现在优先

This is the smallest dependency-ready core increment after branch-head discovery and the existing
commit-bound diff review contract. It closes a named version-history decision gap without adding
storage, migrations, transport, UI, or mutation. Benchmark/evaluation and graph-diff foundations
are already locally verified; branch merge remains the nearest open dependency in the versioning
backbone.

这是 branch-head discovery 与既有 commit-bound diff review contract 之后最小的依赖就绪 core 增量。它直接收束
version-history 的命名决策缺口，不新增 storage、migration、transport、UI 或 mutation。Benchmark/evaluation 与
graph-diff foundation 已在本地验证；branch merge 仍是 versioning backbone 最近的开放依赖。

### Explicit non-goals / 明确非目标

- No branch create, fork, rename, delete, merge write, rollback, commit rewrite, or persistence adapter.
- No graph content diff, semantic/behavior/evaluation calculation, or second `GraphDiff` implementation.
- No REST/OpenAPI/SDK/BFF/Web/CLI/Desktop/operator/release/production work, Docker, provider call, or secret access.
- No claim that an ancestry base is sufficient to resolve content-level merge conflicts.

- 不新增 branch create、fork、rename、delete、merge write、rollback、commit rewrite 或 persistence adapter。
- 不实现 graph content diff、semantic/behavior/evaluation calculation，也不新增第二个 `GraphDiff` 实现。
- 不做 REST/OpenAPI/SDK/BFF/Web/CLI/Desktop/operator/release/production、Docker、provider 或 secret access。
- 不宣称 ancestry base 足以解决 content-level merge conflict。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

Own one framework-independent module under `crates/versioning`, its public export, and focused
versioning tests. The contract accepts validated same-Context commit nodes, checks a finite DAG,
and returns deterministic `MergePlan` values. Documentation is this plan plus the architecture and
roadmap receipts; no storage or transport documentation is expanded.

仅拥有 `crates/versioning` 下一个 framework-independent module、public export 与 focused versioning tests。契约接收
经过验证的同一 Context commit node，检查 finite DAG，并返回确定性的 `MergePlan`。文档为本计划及 architecture、
roadmap 回执；不扩展 storage 或 transport 文档。

### Fresh verification required before the next increment / 下一增量前的新鲜验证

Run focused versioning tests covering fast-forward, no-op, one-base three-way, ambiguous bases,
no common ancestor, missing parent, cross-Context parent, cycles, and deterministic base order;
then run `cargo fmt --all -- --check`, workspace Rust tests, strict offline Clippy, locked Rust
`1.85.0` checks, Web checks, and a static proof that `GraphDiff` remains singular and no transport
surface changed. Runtime, browser, Git, remote, operator, release, production, Docker, and secrets
remain outside this increment.

运行 focused versioning tests，覆盖 fast-forward、no-op、one-base three-way、ambiguous bases、no common ancestor、
missing parent、cross-Context parent、cycle 与 deterministic base order；随后运行 `cargo fmt --all -- --check`、workspace
Rust tests、strict offline Clippy、锁定 Rust `1.85.0` checks、Web checks，并静态证明 `GraphDiff` 仍唯一且没有 transport
surface 改变。runtime、browser、Git、remote、operator、release、production、Docker 与 secrets 继续不在本增量范围。

## Implementation Checklist / 实施清单

- [x] Add validated commit DAG storage and deterministic merge-base resolution.
- [x] Add explicit no-op, fast-forward, three-way, and ancestry-conflict outcomes.
- [x] Add focused red/green tests for malformed and ambiguous histories.
- [x] Run fresh verification and update bilingual roadmap/architecture receipts; keep the long-term goal active.

- [x] 增加 validated commit DAG storage 与确定性 merge-base resolution。
- [x] 增加 explicit no-op、fast-forward、three-way 与 ancestry-conflict outcome。
- [x] 增加 malformed 与 ambiguous history 的 focused red/green tests。
- [x] 运行新鲜验证并更新双语 roadmap/architecture 回执；保持长期目标 active。

## Implementation and Fresh Verification Receipt / 实现与新鲜验证回执

`contextlab-versioning` now owns a framework-independent `CommitGraph` validator and
`MergePlan::resolve`. It rejects duplicate/missing/cross-Context/cyclic ancestry and classifies
identical tips, fast-forward, one-base three-way, no-common-ancestor, and ambiguous maximal-base
outcomes in stable commit-key order. It consumes topology only; content-level conflict resolution and
all graph diff calculation remain outside this module.

`contextlab-versioning` 现负责 framework-independent 的 `CommitGraph` validator 与 `MergePlan::resolve`。它拒绝
duplicate/missing/cross-Context/cyclic ancestry，并以稳定的 commit-key order 区分 identical tip、fast-forward、
one-base three-way、no-common-ancestor 与 ambiguous maximal-base outcome。它只消费 topology；content-level conflict
resolution 与所有 graph diff calculation 仍在本 module 外部。

Fresh evidence: `cargo test -p contextlab-versioning --quiet` (`30 passed`),
`cargo fmt --all -- --check`, versioning strict Clippy, workspace Rust (`186 passed, 39 ignored`),
workspace strict offline Clippy, locked Rust `1.85.0` check, and `pnpm check:web` with public SDK
`15`, local SDK `92`, Web `185`, and production build. Static scope still finds one `impl GraphDiff`.
PostgreSQL/Docker runtime, authenticated browser, Git, remote CI, operator rehearsal, release, and
production remain `unobserved` or `deferred`; no secrets were read.

新鲜证据为 `cargo test -p contextlab-versioning --quiet`（`30 passed`）、`cargo fmt --all -- --check`、versioning
strict Clippy、workspace Rust（`186 passed, 39 ignored`）、workspace strict offline Clippy、锁定 Rust `1.85.0` check，
以及 `pnpm check:web`（public SDK `15`、local SDK `92`、Web `185`、production build）。静态 scope 仍只发现一个
`impl GraphDiff`。PostgreSQL/Docker runtime、authenticated browser、Git、remote CI、operator rehearsal、release 与
production 继续为 `unobserved` 或 `deferred`；未读取 secrets。

## Evidence Boundary / 证据边界

This is a local pure-domain contract, not merge execution or release evidence. No secrets are read;
no external, production, Docker, or PostgreSQL runtime receipt is implied.

这是 local pure-domain contract，不是 merge execution 或 release evidence。不读取 secrets；不暗示任何 external、
production、Docker 或 PostgreSQL runtime 回执。
