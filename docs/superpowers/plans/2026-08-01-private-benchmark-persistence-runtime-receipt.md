# Private Benchmark Persistence Runtime Receipt / 私有 Benchmark 持久化运行时回执

## Necessity Record / 必要性记录

**Named completion criterion and charter principle / 命名完成条件与宪章原则：** This increment
directly supplies fresh evidence for Criterion 3: benchmark runs, scorecards, regression state,
and evaluation-diff projections must be durably persisted and queryable. It also protects the
Context-first charter through exact `(ProjectId, ContextId, CommitId)` scope, deterministic replay,
and redacted server-owned evidence. / 本增量直接为完成条件 3 提供新鲜证据：benchmark run、scorecard、
regression state 与 evaluation-diff projection 必须可持久化、可查询；同时通过精确
`(ProjectId, ContextId, CommitId)` scope、确定性 replay 与服务端脱敏 evidence 保持 Context-first 宪章边界。

**Unmet evidence and risk / 未满足证据与风险：** The PostgreSQL writer, reader, migration, and
ignored integration tests already exist, but the latest workspace audit has not observed a fresh
runtime receipt for this benchmark projection on the current machine. Treating static contracts or
an ignored test as runtime proof would overstate Criterion 3. The remaining risk is transaction,
scope, immutability, replay, provenance, or raw-payload leakage behavior diverging at runtime. /
PostgreSQL writer、reader、migration 与 ignored integration tests 已存在，但当前工作区审计尚未在本机
观察到本 benchmark projection 的新鲜运行时回执。将 static contract 或 ignored test 当作 runtime proof
会夸大条件 3 的完成度；剩余风险是 transaction、scope、immutability、replay、provenance 或 raw-payload
redaction 在运行时发生漂移。

**Why now / 当前优先原因：** This is the smallest dependency-ready increment that closes an
existing evidence gap in an already implemented local persistence path. It is narrower than adding
new authoring, transport, UI, or provider behavior and is independent of deferred remote, operator,
release, and production conditions. / 这是现有本地持久化路径中唯一已实现但缺少新鲜证据的最小依赖就绪增量，
比新增 authoring、transport、UI 或 provider 行为更小，并且不依赖延期的 remote、operator、release 与 production 条件。

**Explicit non-goals / 明确非目标：** No public REST/OpenAPI/public SDK write, Web mutation,
dataset or suite authoring, provider/evaluator call, new migration, second `GraphDiff` calculator,
Docker, production database, secret or environment-file read, release, or production claim. /
不新增 public REST/OpenAPI/public SDK write、Web mutation、dataset/suite authoring、provider/evaluator
调用、新 migration、第二个 `GraphDiff` calculator、Docker、生产数据库、secret 或 environment-file 读取、
release 或 production 声明。

**Smallest affected boundary and bilingual documentation / 最小影响边界与双语文档：** Use only
the existing benchmark workspace PostgreSQL adapter and its named ignored tests, the local temporary
PostgreSQL process, this bilingual receipt, and the bilingual roadmap/audit entries. No production
application or transport code changes are admitted unless a red runtime failure proves a minimal
repair is necessary. / 仅使用既有 benchmark workspace PostgreSQL adapter 与其命名 ignored tests、本地临时
PostgreSQL 进程、本双语回执及双语 roadmap/audit 条目。除非 red runtime failure 证明必须最小修复，否则不准入
production application 或 transport code 变更。

**Fresh verification required before the next increment / 下一增量前必须取得的新鲜验证：** Run
the projection creation/replay/read and execution materialization/replay tests against a fresh
loopback PostgreSQL database, then run focused static contracts, workspace Rust tests, format,
strict offline Clippy, locked Rust 1.85 check, and Web checks. Record no-URL ignored behavior and
filesystem cleanup separately. / 在新建 loopback PostgreSQL 数据库上运行 projection create/replay/read 与
execution materialization/replay tests，随后运行 focused static contracts、workspace Rust tests、format、
strict offline Clippy、锁定 Rust 1.85 check 与 Web checks；无 URL 时的 ignored 行为和 filesystem cleanup 必须单独记录。

## Execution Record / 执行记录

- [x] Fresh loopback PostgreSQL projection runtime receipt.
- [x] Fresh loopback PostgreSQL execution materialization/replay receipt.
- [x] Workspace and cross-stack quality gates.
- [x] Bilingual roadmap and active-goal update; keep long-term goal `active`.
- [x] PostgreSQL, browser, Git, remote, operator, release, and production evidence classified honestly.

## Evidence Record / 证据记录

Observed locally on 2026-08-01 without Docker, secrets, environment-file reads, provider calls,
public transport changes, or production access: / 2026-08-01 在未使用 Docker、未读取 secret 或
environment file、未调用 provider、未改变 public transport 且未接触 production 的条件下观察到：

- Fresh loopback PostgreSQL 16 cluster, named test
  `postgres_benchmark_workspace_projection_creates_replays_and_reads_exact_scope`: `1 passed`.
  It covered migration, immutable create/replay, exact read scope, persisted seal/case rows, and
  wrong-scope rejection. / fresh loopback PostgreSQL 16 cluster 上的 projection named test 为 `1 passed`，
  覆盖 migration、immutable create/replay、exact read scope、持久 seal/case row 与 wrong-scope rejection。
- A second fresh loopback PostgreSQL 16 cluster, named test
  `postgres_benchmark_execution_materializes_and_replays_workspace_projection`: `1 passed`.
  It covered provider-free execution materialization, durable projection replay, and exact readback.
  / 第二个 fresh loopback PostgreSQL 16 cluster 上的 execution named test 为 `1 passed`，覆盖 provider-free
  execution materialization、durable projection replay 与 exact readback。
- Running both named tests against one database was intentionally recorded as a red test-isolation
  finding: the second test hit `relation "workspaces" already exists` after the first test applied
  migrations. The minimal operational repair is one fresh empty database per ignored test; the two
  isolated reruns are the regression proof. No product code change was required. / 两个 named test 共用一个
  database 时记录了测试隔离红灯：第一测试完成 migration 后，第二测试因
  `relation "workspaces" already exists` 失败。最小操作修复是每个 ignored test 使用一个 fresh empty database；
  两次独立重跑构成回归证明，无需修改产品代码。
- Static/focused projection contract: `7 passed`; `cargo fmt --all -- --check`: passed;
  `pnpm check:web`: public SDK `15`, local SDK `135`, Web `284/284`, production build passed.
  / static/focused projection contract 为 `7 passed`；format 通过；Web 全量与 production build 通过。

The temporary database processes were stopped, and `pg_ctl status` confirmed that neither temporary
cluster is running. Tool policy rejected recursive removal of the temporary data directories, so
filesystem cleanup is `unobserved`. A pre-existing local PostgreSQL listener on port 5432 was not
used, inspected for application data, or changed. PostgreSQL runtime is now locally observed for this
projection boundary, but this is not production readiness. Browser, Git, remote CI, operator rehearsal, release, and production evidence remain
`unobserved` or `deferred`; the long-term goal remains `active`.

临时 database process 均已停止，且 `pg_ctl status` 确认两个临时 cluster 均未运行。工具策略拒绝递归删除临时
data directory，因此 filesystem cleanup 为 `unobserved`。既有本地 5432 PostgreSQL listener 未被本回合使用、
未读取其应用数据且未修改。本回执只证明该 projection boundary 的本地 runtime，不代表 production readiness。
browser、Git、remote CI、operator rehearsal、release 与 production evidence 继续为
`unobserved` 或 `deferred`；长期目标保持 `active`。
