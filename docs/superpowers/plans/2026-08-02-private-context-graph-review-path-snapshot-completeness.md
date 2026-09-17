# Private ContextGraph Review Path Snapshot Completeness / 私有 ContextGraph 审阅路径 Snapshot 完整性

## Necessity Record / 必要性记录

### Named criteria and charter principles / 命名条件与宪章原则

This increment directly advances Criterion 2 (replayable version history and version Diff) and
Criterion 4 (Context Graph as the system skeleton). It follows the charter's reusable Rust core,
immutable snapshot, fail-closed validation, and single `GraphDiff::between` calculator principles.

本增量直接推进条件 2（可回放版本历史与版本 Diff）与条件 4（Context Graph 作为系统骨架），遵循宪章的可复用 Rust core、immutable snapshot、
fail-closed validation 与唯一 `GraphDiff::between` calculator 原则。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

`ContextGraphReviewWitness` validates the source and target snapshots and the normal first-parent
topology, but the backend-owned Memory/PostgreSQL witness currently loads no snapshot for commits
between those endpoints. A missing intermediate immutable graph snapshot can therefore pass the
topology guard while the claimed replay path is not fully materialized.

`ContextGraphReviewWitness` 会校验 source/target snapshot 与 normal first-parent topology，但 backend-owned Memory/PostgreSQL witness 当前不读取
两端之间 commit 的 snapshot。因此中间 immutable graph snapshot 缺失时，仍可能通过 topology guard，却无法证明 replay path 已完整 materialize。

### Why now / 为何现在优先

Independent Luna review identified this as the remaining concrete ContextGraph review-path gap
after commit-associated snapshots, parent-snapshot admission, atomic branch-head witnesses, and
normal first-parent validation were already verified. It is local, dependency-ready Rust work and
more necessary than adding another transport, UI surface, or already-completed benchmark breadth.

独立 Luna 审查确认，在 commit-associated snapshots、parent-snapshot admission、atomic branch-head witness 与 normal first-parent validation 已完成验证后，
这是 ContextGraph review path 剩余的具体缺口。它是本地、依赖就绪的 Rust 工作，比新增 transport、UI surface 或重复已完成 benchmark breadth 更必要。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/SDK write, operator transport, Web mutation, migration, provider, secret
  access, Docker/PostgreSQL runtime, browser, Git, remote CI, operator, release, or production claim.
- No merge policy change, graph editor, new repository transport, snapshot API expansion, or second
  Diff calculator. `GraphDiff::between` remains the only graph-diff path.
- Do not expose intermediate snapshots in a response; use them only to validate the backend-owned
  witness before the existing endpoint graph comparison.

- 不新增 public REST/OpenAPI/SDK write、operator transport、Web mutation、migration、provider、secret access、Docker/PostgreSQL runtime、browser、Git、
  remote CI、operator、release 或 production 声明。
- 不改变 merge policy、graph editor、repository transport、snapshot API，也不新增第二个 Diff calculator；`GraphDiff::between` 继续是唯一图 Diff 路径。
- 不在 response 暴露中间 snapshot；只用它们在既有 endpoint graph comparison 前校验 backend-owned witness。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

The minimum code boundary is `crates/storage/src/context_graph_history_review.rs`, the existing
Memory and PostgreSQL witness read implementations, and focused Rust contract tests. The witness
stores the ordered intermediate snapshots only as an internal validated aggregate; the existing
graph review projection and all transport layers remain unchanged. This plan, the completion
criteria, active goal, and parallel ledger form the bilingual documentation boundary.

最小代码边界是 `crates/storage/src/context_graph_history_review.rs`、既有 Memory 与 PostgreSQL witness read implementation 及 focused Rust contract tests。
witness 只在内部 validated aggregate 中保存有序中间 snapshot；既有 graph review projection 与所有 transport layer 保持不变。本计划、completion criteria、
active goal 与 parallel ledger 构成双语文档边界。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First observe a red domain regression for a `source -> middle -> target` history when the middle
snapshot list is empty. Then require the focused versioning/storage tests, workspace Rust tests,
format, strict offline Clippy, locked Rust 1.85 check, local contract fixture, and exact-one
GraphDiff source inspection. PostgreSQL live runtime, Docker, browser, Git, and external release
evidence remain unobserved or deferred.

先观测 `source -> middle -> target` history 在 middle snapshot list 为空时的 domain 红测。随后必须通过 versioning/storage focused tests、workspace Rust tests、
format、strict offline Clippy、锁定 Rust 1.85 check、local contract fixture 与 exact-one GraphDiff 源码检查。PostgreSQL live runtime、Docker、browser、Git 与外部发布证据继续为
unobserved 或 deferred。

## Status / 状态

`completed / verified locally` for this bounded private storage increment; the long-term goal remains `active`. / 本有界私有存储增量为 `completed / verified locally`；长期目标保持 `active`。

## Completion Receipt / 完成回执

`ContextGraphReviewWitness` now owns ordered immutable snapshots for every commit strictly between the source and target of a normal first-parent review path. Memory reads them under one read guard; PostgreSQL reads them in one consistent read transaction. Missing, out-of-order, duplicate, or scope-drifting intermediate snapshots fail closed before the existing endpoint comparison reaches the sole `GraphDiff::between` calculator.

`ContextGraphReviewWitness` 现持有 normal first-parent 审阅路径中 source 与 target 之间每个 commit 的有序 immutable snapshot。Memory 在同一 read guard 中读取它们；PostgreSQL 在同一一致性只读 transaction 中读取它们。缺失、乱序、重复或 scope 漂移的中间 snapshot 会在既有 endpoint comparison 到达唯一的 `GraphDiff::between` calculator 前 fail closed。

The persisted InMemory regression `atomic_witness_rejects_a_persisted_missing_intermediate_snapshot` persists `source -> middle -> target` commit records, materializes only source and target snapshots, and proves the exact middle scope returns `StorageRepositoryError::ScopeUnavailable` before graph comparison. The intended loader-mutation red receipt is `unobserved`: the temporary mutation was restored before an assertion-bearing failure was captured.

持久化 InMemory 回归 `atomic_witness_rejects_a_persisted_missing_intermediate_snapshot` 写入 `source -> middle -> target` commit record，仅 materialize source 与 target snapshot，并证明精确 middle scope 会在图比较前以 `StorageRepositoryError::ScopeUnavailable` fail closed。预期的 loader-mutation 红测证据为 `unobserved`：临时 mutation 已恢复，但未捕获带断言的失败输出。

Fresh local evidence / 新鲜本地证据：

- `cargo +1.85.0 test --workspace` exited `0`; it includes the focused storage composition suite with `7 passed`, including the persisted missing-middle regression. PostgreSQL runtime tests requiring a disposable `CONTEXTLAB_TEST_DATABASE_URL` remain `ignored`; their static query/transaction contracts do not substitute for live missing-middle-row evidence.
- `cargo +1.85.0 fmt --all -- --check`, strict offline `cargo +1.85.0 clippy --workspace --all-targets --locked --offline -- -D warnings`, and locked offline `cargo +1.85.0 check --workspace --all-targets --locked --offline` passed. Necessary behavior-preserving warning cleanup used direct preallocated SHA-256 hex encoding in MCP/API, `writeln!` CLI rendering, a direct evaluation error assertion, and Clippy lifetime elision in the affected storage implementations.
- `pnpm check:web` passed, including public and local SDK checks, Web lint/tests (`310 passed`), and the Next.js production build. `tests/contract/verify-local-contracts.test.ps1` passed.
- The direct `scripts/verify-local-contracts.ps1` exited `0`: source, safe DTO, private route, and GraphDiff checks passed; `public_write_additions=unobserved` and therefore `overall=unobserved` without unified diff input.
- Source inspection found exactly one production `impl GraphDiff` at `crates/diff-engine/src/lib.rs:81`; `GraphDiff::between` has `10` Rust textual matches. The former is implementation count, not a claim that every textual match is a production call site.

No public REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, second graph Diff calculator, Docker/PostgreSQL runtime, authenticated browser/visual smoke, Git/remote CI, operator rehearsal, release, or production claim was added. Those boundaries remain `ignored`, `unobserved`, or `deferred`. This closes one locally verified private storage increment only; Criteria 1, 2, and 4 remain open and the long-term goal remains active.

未新增 public REST/OpenAPI/SDK write、Web mutation、migration、provider、secret access、第二个 graph Diff calculator、Docker/PostgreSQL runtime、已认证 browser/visual smoke、Git/remote CI、operator rehearsal、release 或 production 声明。上述边界保持 `ignored`、`unobserved` 或 `deferred`。本回执只关闭一个本地验证的私有存储增量；条件 1、2、4 仍开放，长期目标保持 active。
