# Private Context Diff Pair Read / 私有 Context Diff 成对读取

## Necessity Record / 必要性记录

### Named criteria and charter principles / 对应完成条件与宪章原则

- **Criterion 2 / 条件 2:** exact Context commit history must be replayable and reviewed from one consistent version-backed snapshot boundary.
  / 条件 2：精确 Context commit history 必须从一致的版本化 snapshot boundary 回放与审阅。
- **Criterion 4 / 条件 4:** Context Graph and reusable Rust storage contracts remain the system backbone; `GraphDiff::between` remains the sole graph-diff calculator.
  / 条件 4：Context Graph 与可复用 Rust storage contract 继续作为系统骨架；`GraphDiff::between` 继续是唯一 graph-diff calculator。

### Gap, dependency, and evidence / 缺口、依赖与证据

`PersistedContextDiffReviewService` currently calls the single-snapshot port twice. Memory takes two independent lock scopes, and PostgreSQL creates two independent `REPEATABLE READ READ ONLY` transactions. The current diff table is append-only, which limits present replacement risk, but the contract still permits a temporally mixed pair and does not make the read boundary explicit. Existing exact-scope validation, immutable records, GraphDiff delegation, and the three-way graph batch pattern are available; no migration or external runtime is required for the local contract.

`PersistedContextDiffReviewService` 当前两次调用 single-snapshot port。Memory 使用两个独立 lock scope，PostgreSQL 创建两个独立的 `REPEATABLE READ READ ONLY` transaction。当前 diff table 是 append-only，降低了现阶段 replacement risk，但 contract 仍允许时间上混合的 pair，也没有显式表达 read boundary。既有 exact-scope validation、immutable record、GraphDiff delegation 与 three-way graph batch pattern 已满足；local contract 不需要 migration 或外部 runtime。

### Why now / 为何现在优先

This is the next dependency-ready hardening directly attached to commit-associated snapshots and the protected version-backed review path. It closes a consistency contract before further consumers or Context editing depend on pair semantics. Web mounting and benchmark breadth already have local receipts; public transport is explicitly out of scope.

这是直接依附于 commit-associated snapshot 与 protected version-backed review path 的下一项依赖就绪 hardening。在更多 consumer 或 Context editing 依赖 pair semantics 前，先收束一致性 contract。Web mounting 与 benchmark breadth 已有本地回执；public transport 明确排除。

### Explicit non-goals / 明确非目标

- No new Web route, inspector, SDK surface, benchmark execution, public REST/OpenAPI/public SDK exposure, or mutation.
  / 不新增 Web route、inspector、SDK surface、benchmark execution、public REST/OpenAPI/public SDK exposure 或 mutation。
- No migration, provider, secret access, Docker/PostgreSQL runtime claim, browser E2E, remote CI, operator rehearsal, release, or production claim.
  / 不新增 migration、provider、secret access、Docker/PostgreSQL runtime 声明、browser E2E、remote CI、operator rehearsal、release 或 production 声明。
- No compatibility default that performs two independent reads in production; the pair operation must be implemented natively by Memory and PostgreSQL adapters.
  / 不提供在 production 中退化为两次独立读取的 compatibility default；pair operation 必须由 Memory 与 PostgreSQL adapter 原生实现。
- No second graph-diff calculator; the pair service only supplies both Rust-owned snapshots to the existing `VersionedContextDiffReviewService`.
  / 不新增第二个 graph-diff calculator；pair service 只向既有 `VersionedContextDiffReviewService` 提供两份 Rust-owned snapshot。

### Smallest boundary, ownership, and bilingual docs / 最小边界、所有权与双语文档

Wave 0 owns the pair type/port in `crates/storage/src/context_diff_snapshot.rs` and routes the service through it in `crates/storage/src/context_diff_review.rs`. Wave 1 assigns Memory implementation plus its contract test to `crates/storage/src/memory.rs` and `crates/storage/tests/context_diff_snapshot_repository.rs`; PostgreSQL implementation plus its SQL/ignored runtime contract to `crates/storage/src/postgres.rs` and `crates/storage/tests/context_diff_snapshot_postgres_contract.rs`; pair service tests remain in `crates/storage/tests/context_diff_review.rs`. The Integration Lead owns this plan, roadmap receipts, and final cross-stack verification. The API composition layer also received the minimal repository/typed-error wiring required by the new internal pair-read contract; no public API, SDK, or Web production surface changed.

Wave 0 负责 `crates/storage/src/context_diff_snapshot.rs` 中 pair type/port，并在 `crates/storage/src/context_diff_review.rs` 让 service 使用它。Wave 1 将 Memory implementation 与 contract test 分配给 `crates/storage/src/memory.rs`、`crates/storage/tests/context_diff_snapshot_repository.rs`；PostgreSQL implementation 与 SQL/ignored runtime contract 分配给 `crates/storage/src/postgres.rs`、`crates/storage/tests/context_diff_snapshot_postgres_contract.rs`；pair service test 保持在 `crates/storage/tests/context_diff_review.rs`。Integration Lead 负责本计划、路线图回执与最终跨栈验证。API composition layer 另外完成了该内部 pair-read contract 所需的最小 repository/typed-error wiring；没有改变 public API、SDK 或 Web production surface。

### Fresh verification required / 下一增量前的新鲜验证

First observe a red service call-count/atomicity regression, then observe green Memory and PostgreSQL contract tests with one-lock/one-transaction implementations. Run focused storage tests, `cargo fmt --all -- --check`, offline workspace tests, strict offline Clippy, locked Rust `1.85.0` check, `pnpm check:web`, local contract verification, and `GRAPH_DIFF_IMPL_COUNT=1`. PostgreSQL runtime remains `unobserved` when Docker is unavailable; an ignored runtime test is not a pass.

先观察 red service call-count/atomicity regression，再观察 one-lock/one-transaction implementation 的 Memory 与 PostgreSQL contract test 变绿。运行 focused storage tests、`cargo fmt --all -- --check`、offline workspace tests、strict offline Clippy、锁定 Rust `1.85.0` check、`pnpm check:web`、local contract verification 与 `GRAPH_DIFF_IMPL_COUNT=1`。Docker 不可用时 PostgreSQL runtime 继续为 `unobserved`；ignored runtime test 不算通过。

## Execution Checklist / 执行清单

- [x] Wave 0: define the typed `ContextDiffSnapshotV1Pair` result and separate `ContextDiffSnapshotV1PairRepository` port, route `PersistedContextDiffReviewService` through one pair read, and retain the existing single-snapshot write/read port for compatibility.
  / Wave 0：定义 typed `ContextDiffSnapshotV1Pair` result 与独立 `ContextDiffSnapshotV1PairRepository` port，让 `PersistedContextDiffReviewService` 使用一次 pair read，并保留既有 single-snapshot write/read port 以保持兼容。
- [x] Wave 1: implement native one-lock Memory and one-transaction PostgreSQL adapters with disjoint tests. The two assigned Luna workers stopped producing output during the implementation window; the Integration Lead took over their disjoint ownership after recording and closing the workers.
  / Wave 1：实现原生 one-lock Memory 与 one-transaction PostgreSQL adapter，并使用互斥 tests。两名被分派的 Luna worker 在实施窗口内停止产出；记录后关闭 worker，由 Integration Lead 接管其不重叠 ownership。
- [x] Run fresh local verification, document evidence states, and keep the long-term goal active. Workspace tests, strict offline Clippy, locked Rust `1.85.0`, Web, contract verifier, and the sole-calculator count all passed; PostgreSQL runtime remains unobserved.
  / 运行新鲜 local verification，记录 evidence state，并保持长期目标 active。Workspace tests、strict offline Clippy、锁定 Rust `1.85.0`、Web、contract verifier 与唯一 calculator count 均通过；PostgreSQL runtime 仍未观测。

## Observed implementation receipt / 已观测实施回执

The pair contract is implemented without changing public transport: `ContextDiffSnapshotV1Pair` and `ContextDiffSnapshotV1PairRepository` are Rust storage contracts; the review service validates both returned exact scopes and delegates only to `VersionedContextDiffReviewService`. The standalone Memory repository and `InMemoryContextGraphRepository` both hold one read lock across the pair; PostgreSQL uses one `REPEATABLE READ READ ONLY` transaction for both exact selects and digest/schema decoding. `GraphDiff::between` remains the sole graph-diff calculator.

本 pair contract 未改变 public transport：`ContextDiffSnapshotV1Pair` 与 `ContextDiffSnapshotV1PairRepository` 是 Rust storage contract；review service 校验两份返回记录的 exact scope，并且只委托 `VersionedContextDiffReviewService`。独立 Memory repository 与 `InMemoryContextGraphRepository` 都在一次 pair read 中持有一个 read lock；PostgreSQL 对两次 exact select 与 digest/schema decode 使用一个 `REPEATABLE READ READ ONLY` transaction。`GraphDiff::between` 仍是唯一 graph-diff calculator。

Observed commands / 已观测命令：review `9 passed`; Memory repository `5 passed`; PostgreSQL contract `1 passed, 2 ignored`; storage library `212 passed, 39 ignored`; workspace tests passed; `cargo fmt --all -- --check`, strict offline Clippy, `cargo +1.85.0 check --workspace --all-targets --locked --offline`, `pnpm check:web` (`15/134/272 + production build`), verifier fixture/live scoped checks, and `GRAPH_DIFF_IMPL_COUNT=1` passed. The live verifier remains `overall=unobserved` only because no unified diff input was supplied. The ignored PostgreSQL runtime cases remain `unobserved`, and Docker/virtualization, browser, Git, remote CI, operator rehearsal, release, and production remain `unobserved` or `deferred`.

已观测命令：review `9 passed`；Memory repository `5 passed`；PostgreSQL contract `1 passed, 2 ignored`；storage library `212 passed, 39 ignored`；workspace tests 通过；`cargo fmt --all -- --check`、strict offline Clippy、`cargo +1.85.0 check --workspace --all-targets --locked --offline`、`pnpm check:web`（`15/134/272 + production build`）、verifier fixture/live scoped checks 与 `GRAPH_DIFF_IMPL_COUNT=1` 通过。live verifier 仅因未提供 unified diff input 而保持 `overall=unobserved`。PostgreSQL ignored runtime case 仍为 `unobserved`；Docker/virtualization、browser、Git、remote CI、operator rehearsal、release 与 production 仍为 `unobserved` 或 `deferred`。
