# Private Versioned Graph Review Ancestry / 私有版本化图审阅 Ancestry

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与章程原则

This increment directly serves Criteria 2 (replayable version history) and Criterion 4
(Context Graph as the system skeleton). A version-backed graph review must compare two
snapshots that belong to one valid normal first-parent replay range; commit existence alone
does not establish that relationship. `GraphDiff::between` remains the sole graph-diff
calculator.

本增量直接服务条件 2（可回放版本历史）与条件 4（Context Graph 作为系统骨架）。版本化图审阅必须比较属于同一条合法
normal first-parent replay range 的两个 snapshot；仅证明 commit 存在不足以建立该关系。`GraphDiff::between` 仍是唯一的图差异计算器。

### Unmet dependency, risk, and evidence gap / 未满足依赖、风险与证据缺口

`PersistedContextGraphHistoryReviewService::review` currently validates exact Context scope and
commit membership, then delegates to the graph review. It accepts reversed, unrelated, and
merge-containing source/target pairs. This can make a read review appear replayable when its
pair is not a normal linear history segment. The missing evidence is a pure versioning contract
and storage regression coverage that rejects those pairs before snapshot reads or GraphDiff.

当前 `PersistedContextGraphHistoryReviewService::review` 只验证 exact Context scope 与 commit membership，随后直接交给 graph
review。它仍接受 reversed、不相关以及包含 merge 的 source/target pair，可能让不可回放的 pair 看起来像合法 review。证据缺口是：
由纯 versioning contract 约束 normal linear history，并由 storage regression 证明这些 pair 在 snapshot read 或 GraphDiff 之前被拒绝。

### Why now / 为什么现在优先

The commit-associated graph snapshot contract, parent-snapshot ancestry invariant, and atomic
branch-head witness are already locally verified. This is the smallest remaining dependency for
trustworthy version-backed graph comparison and precedes any editor, merge-review transport, or
benchmark expansion. It is local-only and does not depend on deferred external deployment receipts.

commit-associated graph snapshot contract、parent-snapshot ancestry invariant 与 atomic branch-head witness 已完成本地验证。
这是可信 version-backed graph comparison 的最小剩余依赖，优先于 editor、merge-review transport 或 benchmark 扩展。它只依赖本地环境，
不依赖延期的外部部署回执。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/SDK route or method, Web/CLI/Desktop mutation, branch mutation, merge writer, rollback, migration, or provider call.
- No change to branch-head witness semantics and no second GraphDiff implementation or calculation path.
- No Docker/PostgreSQL runtime, browser, remote CI, operator, release, production, or secret evidence claim.
- Merge ancestry remains the responsibility of the existing merge-review path; this increment only rejects it from normal graph review.

- 不新增 public REST/OpenAPI/SDK route 或 method、Web/CLI/Desktop mutation、branch mutation、merge writer、rollback、migration 或 provider call。
- 不改变 branch-head witness 语义，不新增第二个 GraphDiff 实现或计算路径。
- 不宣称 Docker/PostgreSQL runtime、browser、remote CI、operator、release、production 或 secrets 证据。
- merge ancestry 仍由既有 merge-review 路径负责；本增量只把它从 normal graph review 中拒绝。

### Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档

Add one read-only method to `contextlab-versioning::CommitHistory`, one fail-closed adapter
error, focused versioning/storage tests, and this bilingual plan plus roadmap ledger receipts.
The service keeps delegating actual graph comparison to the existing graph review projection.

只在 `contextlab-versioning::CommitHistory` 增加一个只读方法、增加一个 fail-closed adapter error、补充 versioning/storage focused tests，
并更新本双语计划与 roadmap ledger 回执。service 仍把实际 graph comparison 委托给既有 graph review projection。

### Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证

First observe red tests for reversed, unrelated, and merge ancestry pairs. Then run the focused
versioning and storage tests, `cargo fmt --all -- --check`, offline workspace tests, strict offline
Clippy, locked Rust `1.85.0` check, `pnpm check:web`, local contract verification, and the exact-one
GraphDiff implementation check. PostgreSQL runtime, Docker, browser, Git, remote CI, operator,
release, and production remain ignored, unobserved, or deferred by actual evidence.

先观测 reversed、不相关与 merge ancestry pair 的红测，再运行 versioning/storage focused tests、`cargo fmt --all -- --check`、offline workspace
tests、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`、local contract verification 与 exact-one GraphDiff implementation check。
PostgreSQL runtime、Docker、browser、Git、remote CI、operator、release 与 production 继续按真实证据标记为 ignored、unobserved 或 deferred。

## Implementation Checklist / 实施清单

- [x] Add the pure normal first-parent range contract and fail-closed error.
- [x] Add red/green versioning and storage review regressions.
- [x] Run fresh verification and update bilingual roadmap receipts.

- [x] 增加纯 normal first-parent range contract 与 fail-closed error。
- [x] 增加 versioning 与 storage review 红绿回归。
- [x] 运行新鲜验证并更新双语 roadmap 回执。

## Status / 状态

`completed / verified locally`; external release evidence remains explicitly deferred and does not
block this local core increment. / `completed / verified locally`；外部发布证据继续明确延期，不阻断本地核心增量。

## Implementation Receipt / 实施回执

The red contract stage first failed because `CommitHistory` had no source-to-target normal replay
method or typed range error. The green implementation adds
`CommitHistory::normal_first_parent_path`, which returns a deterministic inclusive path and
rejects missing commits, reversed/unrelated ranges, and any merge commit on the path. The
history-bound review service and atomic graph witness both enforce it before graph comparison;
merge ancestry remains with the existing merge-review path. / 红测阶段首先因 `CommitHistory` 缺少 source-to-target normal replay
method 与 typed range error 而失败。绿实现增加 `CommitHistory::normal_first_parent_path`，返回确定性的 inclusive path，并拒绝 missing
commit、reversed/unrelated range 以及路径上的任何 merge commit。history-bound review service 与 atomic graph witness 都在 graph
comparison 前执行该校验；merge ancestry 继续交由既有 merge-review path。

Fresh local evidence: versioning history contract `11 passed`; storage history-review contract
`8 passed`; workspace `cargo test --workspace --quiet --offline -j 1` passed with API `222 passed`
and storage `237 passed, 41 ignored`; `cargo fmt --all -- --check`; strict offline Clippy;
locked Rust `1.85.0` check; `pnpm check:web` with public SDK `15`, local SDK `148`, Web `298`,
and production build; and `tests/contract/verify-local-contracts.test.ps1`. Static source checks
found one production `impl GraphDiff` and `10` current `GraphDiff::between` matches. The direct
full verifier remains `overall=blocked` on the pre-existing `LocalBenchmarkWorkspaceResponse`
safe-DTO baseline; this increment did not alter or relabel that unrelated baseline. / 新鲜本地证据：versioning
history contract `11 passed`；storage history-review contract `8 passed`；workspace
`cargo test --workspace --quiet --offline -j 1` 通过，API `222 passed`、storage `237 passed, 41 ignored`；`cargo fmt --all -- --check`；
strict offline Clippy；锁定 Rust `1.85.0` check；`pnpm check:web`（public SDK `15`、local SDK `148`、Web `298` 与 production build）；以及
`tests/contract/verify-local-contracts.test.ps1`。源码检查确认一个 production `impl GraphDiff` 与当前 `10` 个 `GraphDiff::between` 匹配。
直接运行完整 verifier 仍因既有 `LocalBenchmarkWorkspaceResponse` safe-DTO baseline 报告 `overall=blocked`；本增量未修改或重新标记该无关 baseline。

No public REST/OpenAPI/SDK write, Web mutation, migration, provider, secret access, operator
transport, Docker/PostgreSQL runtime claim, browser claim, release, production claim, or second
GraphDiff calculator was added. The long-term goal remains active. / 未新增 public REST/OpenAPI/SDK write、Web mutation、migration、
provider、secret access、operator transport、Docker/PostgreSQL runtime 声明、browser 声明、release、production 声明或第二个 GraphDiff
calculator。长期目标保持 active。
