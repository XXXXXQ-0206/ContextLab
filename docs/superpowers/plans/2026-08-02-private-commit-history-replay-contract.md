# Private Commit-History Replay Contract / 私有提交历史回放契约

## Necessity Record / 必要性记录

**Named completion criteria and charter principles / 命名完成条件与章程原则:**
This increment directly serves Criterion 2 (replayable version history) and the charter's
Context-first, immutable, reusable Rust-core principles. A branch head must be readable as an
exact, deterministic commit history before later Context Graph review or replay consumers can
rely on it.

本增量直接服务条件 2（可回放版本历史）以及章程中的 Context-first、不可变和可复用 Rust core 原则。
后续 Context Graph 审阅或回放 consumer 依赖 branch head 前，必须先能读取精确且确定性的 commit history。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口:**
The versioning crate had commit and replay primitives, but no read-only aggregate that validated
one Context scope, explicit branch heads, complete parent ancestry, deterministic ordering, and
ancestry-only merge planning together. The bounded worker change exists in the worktree, but it
was not yet admitted with this record or integrated with fresh main-thread evidence.

versioning crate 已有 commit 与 replay primitive，但缺少一个只读 aggregate，同时校验单一 Context scope、
显式 branch head、完整 parent ancestry、确定性排序与仅基于 ancestry 的 merge planning。工作树中已有有界
worker 改动，但此前尚未用本记录准入，也尚未取得主线程的新鲜集成证据。

**Why now / 为什么现在优先:**
The private graph-diff lifecycle witness already validates snapshot pairs before delegating to
`GraphDiff::between`. This history contract is the smallest dependency-ready read boundary that
makes branch replay and future version-backed Context editing consistent; it is more direct than
starting a new benchmark, provider, or UI mutation path.

现有 private graph-diff lifecycle witness 已在委托 `GraphDiff::between` 前校验 snapshot pair。该 history
contract 是使 branch replay 与后续 version-backed Context editing 保持一致的最小依赖就绪读取边界，
比启动新的 benchmark、provider 或 UI mutation path 更直接。

**Explicit non-goals / 明确非目标:**

- No commit creation, branch-head mutation, merge execution, persistence, migration, HTTP route,
  public REST/OpenAPI/SDK write, Web mutation, operator transport, provider call, or secret access.
- `GraphDiff::between` remains the sole graph-diff calculator; this crate only validates and reads
  versioning facts.
- No claim of PostgreSQL, browser, remote CI, operator, release, or production evidence.

- 不新增 commit 创建、branch head 修改、merge 执行、持久化、migration、HTTP route、public REST/OpenAPI/SDK
  write、Web mutation、operator transport、provider call 或 secret access。
- `GraphDiff::between` 继续是唯一 graph-diff calculator；本 crate 只校验和读取 versioning facts。
- 不声称已取得 PostgreSQL、browser、remote CI、operator、release 或 production 证据。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档:**
The admitted implementation boundary is `crates/versioning/src/history.rs`, its public re-export in
`crates/versioning/src/lib.rs`, and `crates/versioning/tests/history_contract.rs`. This bilingual
record is the required roadmap boundary note; no API, SDK, Web, storage, or migration file is in
scope.

准入实现边界是 `crates/versioning/src/history.rs`、`crates/versioning/src/lib.rs` 中的 public re-export，以及
`crates/versioning/tests/history_contract.rs`。本双语记录是所需的路线图边界说明；API、SDK、Web、storage 与
migration 文件均不在范围内。

**Fresh verification before the next increment / 下一增量前的新鲜验证:**
Observe focused history tests and the crate format check, then run the integrated workspace Rust
tests, strict offline Clippy, locked Rust 1.85 check, Web checks, local contract verifier, and a
source count proving one `GraphDiff` implementation. Record timeout or unavailable environment
states as such; do not convert them into passed evidence.

先观测 focused history tests 与 crate format check，再运行集成 workspace Rust tests、strict offline Clippy、
锁定 Rust 1.85 check、Web checks、local contract verifier，以及证明只有一个 `GraphDiff` implementation 的
source count。对 timeout 或环境不可用如实记录，不将其转换为通过证据。

## Implementation Boundary / 实现边界

- `BranchHead` represents a named born or unborn branch without mutating it.
- `CommitHistory::try_from_parts` validates one Context, unique commits and branches, known heads,
  and the existing commit graph validation contract.
- Branch replay is deterministic root-to-head traversal and rejects merge ancestry when a linear
  replay is requested. `merge_plan` delegates ancestry classification to the existing versioning
  domain contract.

- `BranchHead` 表示一个已出生或未出生的命名 branch，但不修改它。
- `CommitHistory::try_from_parts` 校验单一 Context、唯一 commit 与 branch、已知 head，以及既有 commit graph
  validation contract。
- Branch replay 按 root-to-head 确定性遍历；要求 linear replay 时拒绝 merge ancestry。`merge_plan` 委托既有
  versioning domain contract 完成 ancestry classification。

## Evidence Receipt / 证据回执

The Integration Lead reran the admitted boundary against the current worktree on 2026-08-02.
The focused history contract reported `8 passed, 0 failed`; the full versioning crate test suite
reported `54 passed, 0 failed`; and the versioning format check passed. The Diff review found a
mixed `CRLF/LF` plus EOF-newline edge case. The minimal fix tightened the EOF-only guard and added
a regression; the Diff crate then reported `13 passed, 0 failed` and its format check passed.

Integration gates also passed: `cargo test --workspace --quiet --offline` exited successfully
(the storage suite reported `222 passed, 41 ignored`), `cargo fmt --all -- --check`, strict offline
Clippy, `cargo +1.85.0 check --workspace --all-targets --locked --offline`,
`pnpm check:web` (public SDK checks, Web `298 passed`, and production build), and
`pwsh -NoProfile -ExecutionPolicy Bypass -File tests/contract/verify-local-contracts.test.ps1`.
A source count found exactly one `impl GraphDiff`.

集成负责人已在 2026-08-02 针对当前工作树重新运行准入边界。focused history contract 报告 `8 passed, 0 failed`；
完整 versioning crate 报告 `54 passed, 0 failed`；versioning format check 通过。Diff review 发现混合 `CRLF/LF`
与 EOF newline 同时变化的边界；最小修复收紧 EOF-only guard 并新增 regression，随后 Diff crate 报告
`13 passed, 0 failed` 且 format check 通过。

集成门禁也已通过：`cargo test --workspace --quiet --offline` 成功退出（storage suite 报告 `222 passed, 41 ignored`）、
`cargo fmt --all -- --check`、strict offline Clippy、`cargo +1.85.0 check --workspace --all-targets --locked --offline`、
`pnpm check:web`（public SDK checks、Web `298 passed` 与 production build），以及
`pwsh -NoProfile -ExecutionPolicy Bypass -File tests/contract/verify-local-contracts.test.ps1`。source count 确认只有一个
`impl GraphDiff`。

This is local contract evidence only. PostgreSQL runtime, Docker, authenticated browser/visual
smoke, Git change-set binding, remote CI, operator rehearsal, release, and production remain
`ignored`, `unobserved`, or `deferred`. The history increment is locally verified but does not
close Criterion 2, any other completion criterion, or the active long-term goal. The next admitted
increment requires a fresh bilingual Necessity Record; the existing benchmark decision-pair witness
record is the current dependency-ready candidate for review, not an automatic implementation order.

本回执仅是本地 contract evidence。PostgreSQL runtime、Docker、authenticated browser/visual smoke、Git change-set binding、
remote CI、operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`。history 增量已在本地
验证，但不关闭条件 2、任何其他完成条件或 active long-term goal。下一项准入增量仍需新的双语 Necessity Record；现有
benchmark decision-pair witness 记录是当前依赖就绪的审查候选，不是自动实施顺序。
