# Private Exact-Commit Context Lifecycle Atomic Read / 私有精确提交 Context 生命周期原子读取

## Necessity Record / 必要性记录

### Named criteria and charter principle / 对应条件与宪章原则

This increment directly advances Criterion 1 (Context-first platform coverage) and Criterion 2
(replayable versioning and diff workflows). An exact Context commit read must return a mutually
consistent lifecycle state, replay state, graph snapshot, component content, and metadata through
reusable Rust storage boundaries. It does not close either criterion or the active long-term goal.

本增量直接推进条件 1（Context-first platform coverage）与条件 2（可回放版本与 Diff workflow）。精确 Context commit read 必须通过可复用 Rust storage boundary 返回相互一致的 lifecycle state、replay state、graph snapshot、component content 与 metadata。不关闭任一条件或 active long-term goal。

### Gap and dependencies / 缺口与依赖

The existing `ContextLifecycleService::read_state_at_commit` and API read path already validate
replay and graph consistency, but storage reads for lifecycle state, component content, and graph
snapshot are composed from separate repository calls. Without one read boundary, a future mutable
backend could combine facts observed at different points. Existing Memory/PostgreSQL repository
ports, replay validation, graph snapshot persistence, and API/SDK/Web read contracts are ready; the
missing piece is a parity-preserving atomic read contract and regression evidence.

现有 `ContextLifecycleService::read_state_at_commit` 与 API read path 已校验 replay 和 graph consistency，但 lifecycle state、component content 与 graph snapshot 的 storage read 仍由多个 repository call 组合。没有单一 read boundary，未来可变 backend 可能组合出不同观察时点的 facts。既有 Memory/PostgreSQL repository port、replay validation、graph snapshot persistence 与 API/SDK/Web read contract 均已就绪；缺口是保持 parity 的 atomic read contract 与回归证据。

### Why now / 为什么现在优先

This is the next dependency-ready Context-first increment after the fresh Knowledge/Memory runtime
receipt. It improves the correctness of the existing exact-commit local read without adding a
new screen, route, write path, benchmark surface, provider, or release dependency. It is higher
priority than expanding benchmark UI because version-consistent Context history is the system
backbone on which later evaluation and workflow inspection depend.

这是 fresh Knowledge/Memory runtime receipt 之后下一个依赖就绪的 Context-first 增量。它直接改善既有 exact-commit local read 的正确性，不新增 screen、route、write path、benchmark surface、provider 或 release dependency。相比扩展 benchmark UI，它优先级更高，因为版本一致的 Context history 是后续 evaluation 与 workflow inspection 依赖的系统骨架。

### Explicit non-goals / 明确非目标

- No public REST/OpenAPI/public SDK method, write route, mutation, merge writer, branch mutation, rollback, scheduler, provider, migration, operator transport, or production claim.
- No new GraphDiff implementation; `GraphDiff::between` remains the sole graph-diff calculator.
- No Knowledge/Memory projection change, no raw content expansion, and no authenticated browser or external release evidence.
- Do not claim PostgreSQL runtime atomicity until a fresh local runtime fixture observes the new contract; Memory parity and SQL/static contracts may be verified independently.

- 不新增 public REST/OpenAPI/public SDK method、写入 route、mutation、merge writer、branch mutation、rollback、scheduler、provider、migration、operator transport 或 production 声明。
- 不新增 GraphDiff implementation；`GraphDiff::between` 继续是唯一 graph-diff calculator。
- 不修改 Knowledge/Memory projection，不扩大 raw content，也不进行 authenticated browser 或 external release evidence。
- 在 fresh local runtime fixture 观测到新 contract 前，不声称 PostgreSQL runtime atomicity；Memory parity 与 SQL/static contract 可独立验证。

### Smallest boundary and bilingual documentation / 最小边界与双语文档

The smallest implementation boundary is the reusable `crates/storage` lifecycle read contract and
its Memory/PostgreSQL adapters, with only the existing API adapter changed if required to consume
the aggregate. Focused storage/API tests and this plan plus bilingual roadmap/completion/parallel
receipts are required. The existing SDK/Web read contract should remain unchanged unless a type
adapter is strictly necessary.

最小实现边界是可复用的 `crates/storage` lifecycle read contract 及其 Memory/PostgreSQL adapter；只有在消费 aggregate 必需时才修改既有 API adapter。必须有 focused storage/API test、本计划与双语 roadmap/completion/parallel receipt。除非严格必要，既有 SDK/Web read contract 保持不变。

### Fresh verification before the next increment / 下一增量前的新鲜验证

First add a red contract test that proves the aggregate cannot be assembled from mismatched exact
scope or inconsistent replay/graph facts. Then require Memory parity, PostgreSQL SQL/static contract,
the protected API compatibility tests, workspace Rust, format, strict offline Clippy, locked Rust
1.85 check, `pnpm check:web`, local contract verifier, and `GRAPH_DIFF_IMPL_COUNT=1`. A disposable
PostgreSQL runtime receipt is required before claiming runtime atomicity; remote CI, operator,
release, and production remain deferred.

先增加红 contract test，证明 aggregate 不能由 scope 不一致或 replay/graph facts 不一致的组合构成。随后必须通过 Memory parity、PostgreSQL SQL/static contract、protected API compatibility tests、workspace Rust、format、strict offline Clippy、锁定 Rust 1.85 check、`pnpm check:web`、local contract verifier 与 `GRAPH_DIFF_IMPL_COUNT=1`。只有 disposable PostgreSQL runtime receipt 通过后，才能声称 runtime atomicity；remote CI、operator、release 与 production 继续延期。

## Execution checklist / 执行清单

- [x] Add the atomic lifecycle read port and a red/green Memory parity contract.
- [x] Implement the smallest PostgreSQL read transaction adapter or SQL contract required by the existing storage boundary.
- [x] Preserve existing API/SDK/Web response shape and run focused plus full verification.
- [x] Update bilingual receipts, keep the long-term goal active, and choose the next criterion-driven increment.

- [x] 增加 atomic lifecycle read port 与 red/green Memory parity contract。
- [x] 按既有 storage boundary 实现最小 PostgreSQL read transaction adapter 或所需 SQL contract。
- [x] 保持既有 API/SDK/Web response shape，运行 focused 与完整验证。
- [x] 更新双语 receipt，保持长期目标 active，并选择下一项 criterion-driven 增量。

## Completion Receipt / 完成回执

This bounded local increment is `completed / verified locally`; it advances Criteria 1 and 2
without closing either criterion or the active long-term goal. `ContextLifecycleReadRepository`
now returns one exact-commit aggregate carrying the graph snapshot, replay state, component
inventory, immutable content witnesses, and `(ProjectId, ContextId, CommitId)` scope. Memory holds
one read guard while assembling the facts. PostgreSQL performs the graph, ancestry, revision, and
replay queries in one `REPEATABLE READ READ ONLY` transaction. The service rejects returned
Context/commit scope drift before adapting the existing response DTO.

本次有界 local 增量标记为 `completed / verified locally`；推进条件 1 与 2，但不关闭任一条件或
active long-term goal。`ContextLifecycleReadRepository` 现返回一个 exact-commit aggregate，携带
graph snapshot、replay state、component inventory、immutable content witness 与
`(ProjectId, ContextId, CommitId)` scope。Memory 在单个 read guard 内组装 facts；PostgreSQL 在一个
`REPEATABLE READ READ ONLY` transaction 内读取 graph、ancestry、revision 与 replay。service 在适配既有
response DTO 前拒绝返回的 Context/commit scope drift。

Fresh evidence / 新鲜证据：

- Storage lifecycle focused tests: `15 passed`; PostgreSQL aggregate SQL contract: `1 passed`.
- API lifecycle focused tests: `5 passed`; full workspace Rust: storage `219 passed, 39 ignored`.
- `cargo fmt --all -- --check`, strict offline Clippy, and `cargo +1.85.0 check --workspace --locked --offline` passed.
- `pnpm check:web` passed with public SDK `15`, local SDK `135`, Web `284`, and production build.
- `scripts/verify-local-contracts.ps1` scoped checks passed; live `overall=unobserved` only because no unified diff input was supplied. `GRAPH_DIFF_IMPL_COUNT=1` passed.

新鲜证据：

- storage lifecycle 聚焦测试 `15 passed`；PostgreSQL aggregate SQL contract `1 passed`。
- API lifecycle 聚焦测试 `5 passed`；完整 workspace Rust 通过，storage `219 passed, 39 ignored`。
- `cargo fmt --all -- --check`、strict offline Clippy 与 `cargo +1.85.0 check --workspace --locked --offline` 通过。
- `pnpm check:web` 通过（public SDK `15`、local SDK `135`、Web `284` 与 production build）。
- `scripts/verify-local-contracts.ps1` 范围检查通过；因未提供 unified diff input，live `overall=unobserved`；`GRAPH_DIFF_IMPL_COUNT=1` 通过。

The disposable PostgreSQL runtime receipt for this new aggregate is `unobserved`: no
`CONTEXTLAB_TEST_DATABASE_URL` was available in this turn. The transaction implementation is
compiled and statically checked, but no runtime claim is made. Docker/virtualization, browser,
Git, remote CI, operator rehearsal, release, and production remain unobserved or deferred. No
public REST/OpenAPI/public SDK write, Web mutation, migration, provider, secret access, operator
transport, or second GraphDiff calculator was added.

本次新 aggregate 的 disposable PostgreSQL runtime receipt 为 `unobserved`：本回合没有可用的
`CONTEXTLAB_TEST_DATABASE_URL`。transaction implementation 已编译并通过静态检查，但不作 runtime 声明。
Docker/virtualization、browser、Git、remote CI、operator rehearsal、release 与 production 继续为
`unobserved` 或 `deferred`。未新增 public REST/OpenAPI/public SDK write、Web mutation、migration、provider、
secret access、operator transport 或第二个 GraphDiff calculator。

Next admitted increment / 下一项准入增量：before implementation, create a fresh bilingual Necessity
Record for the next dependency-ready named criterion. The current evidence-backed candidates are
benchmark persistence closure or Context lifecycle read consumption in a private read-only workflow;
do not widen this increment into mutation or public transport.

下一项实现前，先为下一项依赖就绪的命名条件创建新的双语 Necessity Record。当前有证据支持的候选是
Benchmark persistence closure 或在 private read-only workflow 中消费 Context lifecycle read；不得把本增量扩大为 mutation 或 public transport。
