# Private Context Lifecycle PostgreSQL Runtime Receipt / 私有 Context 生命周期 PostgreSQL 运行时回执

## Necessity Record / 必要性记录

**Named completion criterion and charter principle / 命名完成条件与宪章原则:** This increment
directly supplies fresh local evidence for Criteria 1 and 2: a Context lifecycle must be persisted
through the reusable Rust storage boundary, replayable by commit, and readable as exact graph,
component, relationship, and content state. It protects the Context-first, versioned-history, and
single-source-of-truth principles. / 本增量直接为条件 1 与 2 提供新鲜本地证据：Context lifecycle 必须通过
可复用 Rust storage boundary 持久化，能够按 commit replay，并可读取 exact graph、component、relationship 与
content state；同时保持 Context-first、versioned-history 与 single-source-of-truth 原则。

**Unmet evidence, risk, and dependency / 未满足证据、风险与依赖:** The lifecycle aggregate
read contract and Memory/PostgreSQL implementation are already present, but the latest aggregate
receipt explicitly recorded PostgreSQL runtime as `unobserved`. Static SQL checks and Memory parity
cannot prove migration application, transaction behavior, guarded create/replay, exact commit read,
or relationship removal against a real database. / lifecycle aggregate read contract 与 Memory/PostgreSQL 实现
已经存在，但最新 aggregate 回执明确将 PostgreSQL runtime 记为 `unobserved`。static SQL check 与 Memory parity
无法证明真实数据库上的 migration application、transaction behavior、guarded create/replay、exact commit read 或
relationship removal。

**Why now / 当前优先原因:** This is the smallest dependency-ready evidence increment for the
Context/versioning backbone and is independent of the already deferred external release conditions.
It is narrower and safer than adding another Context mutation, public transport, or benchmark feature,
and the current machine can create a fresh local PostgreSQL cluster without Docker. / 这是 Context/versioning
骨架中最小且依赖就绪的 evidence 增量，不依赖已延期的 external release conditions。它比新增 Context mutation、
public transport 或 benchmark feature 更小；当前机器无需 Docker 即可创建 fresh local PostgreSQL cluster。

**Explicit non-goals / 明确非目标:** No production code change unless a runtime red test proves
a minimal repair is necessary; no public REST/OpenAPI/public SDK write, Web mutation, provider,
second `GraphDiff` calculator, new migration, secret or environment-file read, use of the existing
5432 listener, Docker, browser, release, or production claim. / 除非 runtime red test 证明必须最小修复，否则
不修改 production code；不新增 public REST/OpenAPI/public SDK write、Web mutation、provider、第二个
`GraphDiff` calculator、new migration、secret 或 environment-file read；不使用既有 5432 listener、Docker、
browser，也不作 release 或 production 声明。

**Smallest affected boundary and bilingual documentation / 最小影响边界与双语文档:** Use only
the existing ignored PostgreSQL lifecycle tests, a fresh loopback temporary cluster per test, this
receipt, and the active/completion/parallel evidence entries. The pre-existing PostgreSQL listener
on port 5432 remains untouched and its application data is not inspected. / 仅使用既有 ignored PostgreSQL
lifecycle tests、每个测试独立的 fresh loopback 临时 cluster、本回执，以及 active/completion/parallel evidence 条目。
既有 5432 PostgreSQL listener 保持 untouched，不读取其 application data。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证:** Run
`postgres_lifecycle_initialization_creates_and_replays_an_unborn_branch_root` and
`postgres_lifecycle_replays_typed_uses_relationship_addition_and_removal` separately against fresh
empty loopback PostgreSQL databases. Then run the focused lifecycle/static contracts, workspace Rust,
format, strict offline Clippy, locked Rust 1.85 check, Web check, and contract verifier. Record temporary
process stop and directory cleanup independently. / 分别在 fresh empty loopback PostgreSQL database 上运行
`postgres_lifecycle_initialization_creates_and_replays_an_unborn_branch_root` 与
`postgres_lifecycle_replays_typed_uses_relationship_addition_and_removal`；随后运行 focused lifecycle/static contract、
workspace Rust、format、strict offline Clippy、锁定 Rust 1.85 check、Web check 与 contract verifier。临时 process stop 与
目录 cleanup 必须独立记录。

## Execution Record / 执行记录

- [x] Fresh lifecycle initialization/replay PostgreSQL receipt.
- [x] Fresh lifecycle relationship read/replay PostgreSQL receipt.
- [x] Workspace and cross-stack quality gates.
- [x] Bilingual active-goal, completion-audit, and parallel-plan update; keep long-term goal `active`.
- [x] Existing port-5432 listener, browser, Git, remote, operator, release, and production evidence classified honestly.

## Evidence Record / 证据记录

Observed locally on 2026-08-02 without Docker, secret or environment-file reads, provider calls,
public transport changes, or production access: / 2026-08-02 在未使用 Docker、未读取 secret 或
environment file、未调用 provider、未改变 public transport 且未接触 production 的条件下观察到：

- `postgres_lifecycle_initialization_creates_and_replays_an_unborn_branch_root`: first fresh-cluster
  run was red only at the stale fixture expectation `(1, 0, 1, 1, 1, 0, 0)`; the real seeded database
  returned `(1, 0, 1, 1, 1, 6, 0)`. The minimum test-only repair changed the expected seeded component
  count to `6`; a second independent fresh-cluster rerun passed `1`. / 首次 fresh-cluster run 只在陈旧
  fixture expectation `(1, 0, 1, 1, 1, 0, 0)` 处红灯；真实 seeded database 返回 `(1, 0, 1, 1, 1, 6, 0)`。
  最小 test-only 修复将 seeded component count 改为 `6`；第二个独立 fresh-cluster rerun 为 `1 passed`。
- `postgres_lifecycle_replays_typed_uses_relationship_addition_and_removal`: `1 passed` on a
  separate fresh loopback PostgreSQL 16 cluster, covering guarded lifecycle creation, exact commit
  read, relationship add/remove, and replay. / 在独立 fresh loopback PostgreSQL 16 cluster 上为 `1 passed`，
  覆盖 guarded lifecycle creation、exact commit read、relationship add/remove 与 replay。
- The pre-existing PostgreSQL listener on port 5432 was not used, inspected for application data,
  or changed. Each temporary cluster was stopped; recursive data-directory cleanup remains
  `unobserved` due local tool policy. / 既有 5432 PostgreSQL listener 未被使用、未读取其 application data 且未修改。
  每个临时 cluster 均已停止；由于本地工具策略，递归 data-directory cleanup 继续为 `unobserved`。

The runtime receipt is local, non-production evidence for the existing Context lifecycle boundary.
It does not close Criteria 1 or 2, and browser, Git, remote CI, operator rehearsal, release, and
production evidence remain `unobserved` or `deferred`. / 本 runtime receipt 是既有 Context lifecycle boundary
的本地非生产 evidence，不关闭条件 1 或 2；browser、Git、remote CI、operator rehearsal、release 与 production
evidence 继续为 `unobserved` 或 `deferred`。
