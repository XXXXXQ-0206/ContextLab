# Private Context Commit-History Consistent Read / 私有 Context 提交历史一致读取

## Necessity Record / 必要性记录

**Served completion criteria / 服务的完成条件**

- Criterion 2, replayable version history: a complete `CommitHistory` and its durable branch heads must be observed from one backend-owned read boundary.
- Criterion 4, Context Graph as the system skeleton: the existing private graph-diff review must bind graph snapshots to the same validated history witness.
- 该增量服务条件 2（可重放版本历史）与条件 4（Context Graph 系统骨架）：完整 `CommitHistory` 及持久分支 head 必须来自后端拥有的同一读取边界，现有私有 graph-diff review 必须绑定同一份已校验的历史见证。

**Unmet dependency, risk, or evidence gap / 未满足依赖、风险或证据缺口**

`ContextCommitHistoryRepository` currently composes independent commit/detail and branch-head ports. Memory obtains separate locks and PostgreSQL obtains separate transactions or pool reads, so the current contract proves validation but not one consistent observation point.

当前 `ContextCommitHistoryRepository` 仍组合独立的 commit/detail 与 branch-head port。Memory 分别获取锁，PostgreSQL 分别使用事务或 pool 读取；现有契约能证明校验规则，但不能证明所有数据来自同一观察点。

**Why now / 为什么现在优先**

Typed commit records, `CommitHistory::try_from_parts`, branch-head selection, graph snapshot batch reads, and the private history-bound review already exist. This is the smallest dependency-ready storage increment that closes the remaining consistency gap before adding any new Context editing surface.

typed commit record、`CommitHistory::try_from_parts`、branch-head selection、graph snapshot batch read 与私有 history-bound review 均已存在。本增量是新增 Context 编辑面之前，直接收束一致性缺口的最小依赖就绪 storage 工作。

**Explicit non-goals / 明确非目标**

- No new public REST, OpenAPI, SDK, or Web mutation/read surface; the existing private graph-diff route remains unchanged.
- No migration, provider, operator transport, production claim, or PostgreSQL runtime receipt is manufactured.
- No second graph-diff calculator; `GraphDiff::between` remains the sole calculator.
- 不新增 public REST、OpenAPI、SDK 或 Web mutation/read surface；现有私有 graph-diff route 保持不变。
- 不新增 migration、provider、operator transport 或生产声明，不伪造 PostgreSQL runtime 回执。
- 不新增第二个图差异计算器；`GraphDiff::between` 仍是唯一计算器。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档**

The boundary is the storage crate's `ContextCommitHistoryRepository`, its in-memory and PostgreSQL concrete adapters, the API state's private repository injection, focused contract tests, and this bilingual plan plus the roadmap receipt updates. Existing generic adapter behavior remains available for isolated test doubles but is not used by the real API path.

边界限定为 storage crate 的 `ContextCommitHistoryRepository`、Memory 与 PostgreSQL concrete adapter、API state 的私有 repository 注入、聚焦契约测试，以及本计划和路线图回执更新。现有 generic adapter 继续供隔离 test double 使用，但真实 API path 不再使用它。

**Fresh verification before the next increment / 下一增量前的新鲜验证**

Run focused storage/API tests, workspace Rust tests, format, strict offline Clippy, locked Rust 1.85 checks, Web checks, the local contract verifier, and the exact-one `GraphDiff` source check. PostgreSQL/Docker runtime, browser/visual, Git, remote CI, operator rehearsal, release, and production evidence remain unobserved or deferred for this increment.

运行 storage/API focused tests、workspace Rust tests、格式检查、strict offline Clippy、锁定 Rust 1.85 检查、Web 检查、本地 contract verifier 与 exact-one `GraphDiff` source check。对于本增量，PostgreSQL/Docker runtime、browser/visual、Git、remote CI、operator rehearsal、release 与 production 证据仍为未观测或延期。

## Implementation Boundary / 实施边界

The concrete repository reads all commit details, parents, and branch heads in one Memory read guard or one PostgreSQL `REPEATABLE READ READ ONLY` transaction, then delegates rehydration and invariant validation to `CommitHistory::try_from_parts`. The protected API route consumes the concrete contract through `AppState`; no route or response schema changes.

具体 repository 在 Memory 的单一 read guard 或 PostgreSQL 的单一 `REPEATABLE READ READ ONLY` transaction 内读取全部 commit detail、parent 与 branch head，随后仍委托 `CommitHistory::try_from_parts` 完成重建和不变量校验。受保护 API route 通过 `AppState` 使用该 concrete contract，不改变 route 或 response schema。

## Completion Receipt / 完成回执

This bounded storage/application increment is `completed / verified locally`. The concrete Memory and
PostgreSQL repositories now own the complete Context commit-history read boundary, and the protected
graph-diff handler consumes that injected repository. The earlier generic multi-port adapter remains
available only for isolated test doubles; it is not the real runtime path. This receipt supersedes the
earlier focused-only wording for this specific history path. Broader future multi-aggregate reads still
require their own Necessity Record.

本有界 storage/application 增量现标记为 `completed / verified locally`。Memory 与 PostgreSQL concrete
repository 现共同拥有完整 Context 提交历史读取边界，protected graph-diff handler 使用注入的该 repository。
较早的 generic multi-port adapter 仅继续供隔离 test double 使用，不是真实 runtime path。本回执 supersede 了此前
针对该 history path 的 focused-only 表述；更广泛的未来 multi-aggregate read 仍需各自的 Necessity Record。

Fresh local verification / 新鲜本地验证:

- `cargo test --workspace --quiet`: API `223 passed`; storage `233 passed, 41 ignored`; no failures.
- `cargo fmt --all -- --check`; strict offline Clippy with `-D warnings`; locked Rust `1.85.0`
  `cargo check --workspace --all-targets --locked --offline`: passed.
- `pnpm check:web`: public SDK `15`, local SDK checks passed, Web `298 passed`, TypeScript/lint,
  and production build: passed.
- `tests/contract/verify-local-contracts.test.ps1`: passed; source inspection found exactly one
  production `impl GraphDiff` and `10` `GraphDiff::between` call sites.

- `cargo test --workspace --quiet`：API `223 passed`；storage `233 passed, 41 ignored`；无失败。
- `cargo fmt --all -- --check`、strict offline Clippy（`-D warnings`）与锁定 Rust `1.85.0` 的
  `cargo check --workspace --all-targets --locked --offline`：通过。
- `pnpm check:web`：public SDK `15`、local SDK checks 通过、Web `298 passed`、TypeScript/lint 与
  production build：通过。
- `tests/contract/verify-local-contracts.test.ps1`：通过；源码检查确认只有一个 production
  `impl GraphDiff`，以及 `10` 个 `GraphDiff::between` 调用点。

PostgreSQL runtime requiring `CONTEXTLAB_TEST_DATABASE_URL`, Docker, authenticated browser/visual
smoke, Git change-set, remote CI, operator rehearsal, release, and production remain
`ignored`, `unobserved`, or `deferred`. No public REST/OpenAPI/SDK write, Web mutation, migration,
provider, secret access, operator transport, or second graph-diff calculator was added. The
long-term goal remains active; the next increment must begin with a new bilingual Necessity Record.

需要 `CONTEXTLAB_TEST_DATABASE_URL` 的 PostgreSQL runtime、Docker、authenticated browser/visual
smoke、Git change-set、remote CI、operator rehearsal、release 与 production 继续为
`ignored`、`unobserved` 或 `deferred`。未新增 public REST/OpenAPI/SDK write、Web mutation、migration、
provider、secret access、operator transport 或第二个 graph-diff calculator。长期目标保持 active；下一项增量
必须以新的双语 Necessity Record 开始。

## Gate Correction: Explicit History Injection / 门禁修正：显式注入 History

**Necessity Record / 必要性记录:** The independent review found a real construction-path gap after
the initial receipt: public `AppState::with_workspace_repositories` still silently built the old
split-port adapter, so a custom builder could bypass the backend-owned consistent-read contract.
The API focused tests also used that fallback and therefore did not prove concrete repository use.
This directly affects Criteria 2 and 4 and must be corrected before selecting another increment.

**必要性记录：** 初始回执后的独立审查发现真实的 construction-path 缺口：public
`AppState::with_workspace_repositories` 仍会静默构造旧 split-port adapter，因此 custom builder 可以绕过
backend-owned consistent-read contract。API focused tests 也使用了该 fallback，不能证明 concrete repository 被使用。
这直接影响条件 2 与 4，必须在选择下一项增量前修正。

**Minimum repair and non-goals / 最小修复与非目标:** `WorkspaceRepositories` will carry an
explicit optional `ContextCommitHistoryRepository`. The environment path supplies the concrete
Memory/PostgreSQL repository. Generic custom composition without an explicit history repository
will fail closed as unavailable rather than silently falling back to split reads; graph-diff test
fixtures will inject the shared Memory repository explicitly. No public route, OpenAPI/SDK method,
Web mutation, migration, provider, secret access, operator transport, or second `GraphDiff`
calculator is in scope.

`WorkspaceRepositories` 将携带显式 optional `ContextCommitHistoryRepository`。environment path 提供
Memory/PostgreSQL concrete repository。未显式提供 history repository 的 generic custom composition 将以 unavailable
fail closed，而不是静默 fallback 到 split read；graph-diff test fixture 显式注入共享 Memory repository。范围不包括
public route、OpenAPI/SDK method、Web mutation、migration、provider、secret access、operator transport 或第二个
`GraphDiff` calculator。

**Fresh verification before the next increment / 下一增量前的新鲜验证:** First observe the new
construction-path regression fail, then its green result; run the focused protected graph-diff
tests, workspace Rust tests, format, strict offline Clippy, locked Rust 1.85 checks, Web checks,
the local contract verifier, and the exact-one `GraphDiff` source check. Keep PostgreSQL runtime,
Docker, browser/visual, Git, remote CI, operator rehearsal, release, and production evidence
separate and honestly `unobserved` or `deferred`.

先观测新的 construction-path regression 红灯，再取得绿灯；随后运行 protected graph-diff focused tests、workspace Rust
tests、format、strict offline Clippy、锁定 Rust 1.85 checks、Web checks、local contract verifier 与 exact-one `GraphDiff`
source check。PostgreSQL runtime、Docker、browser/visual、Git、remote CI、operator rehearsal、release 与 production evidence
继续单独且如实标记为 `unobserved` 或 `deferred`。

## Gate Correction Receipt / 门禁修正回执

The construction-path red/green regression and the cross-crate scope contract are now green:
the API library reported `224 passed`, protected graph-diff focused tests `7 passed`, and the
independent `commit_graph_snapshot_scope_contract` reported `3 passed`. Integrated local gates
reported workspace API `224 passed`, storage `233 passed, 41 ignored`, public/local/Web SDK test
counts `15/148/298`, production Web build, format, strict offline Clippy, locked Rust `1.85.0`,
the local contract verifier, and exactly one production `impl GraphDiff` with `10` call sites.

construction-path red/green regression 与 cross-crate scope contract 现已通过：API library `224 passed`，protected
graph-diff focused `7 passed`，独立 `commit_graph_snapshot_scope_contract` `3 passed`。集成本地 gate 报告 workspace API
`224 passed`、storage `233 passed, 41 ignored`、public/local/Web SDK 测试数 `15/148/298`、production Web build、format、
strict offline Clippy、锁定 Rust `1.85.0`、local contract verifier，以及一个 production `impl GraphDiff` 与 `10` 个调用点。

The root cause is corrected without expanding any public transport or mutation surface. PostgreSQL
runtime, Docker, browser/visual, Git, remote CI, operator rehearsal, release, and production remain
`ignored`, `unobserved`, or `deferred`; the long-term goal remains active.

根因已修复，且没有扩展任何 public transport 或 mutation surface。PostgreSQL runtime、Docker、browser/visual、Git、remote CI、
operator rehearsal、release 与 production 继续为 `ignored`、`unobserved` 或 `deferred`；长期目标保持 active。
