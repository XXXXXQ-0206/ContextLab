# Benchmark Execution Workspace Materialization Plan / Benchmark 执行工作台物化计划

**Status / 状态：** Implemented and verified locally. / 已实现并完成本地验证。

## Necessity Record / 必要性记录

**Completion criteria and charter principles / 收束条件与宪章原则：** This increment directly
advances Criterion 3 by making the existing provider-free `BenchmarkExecutionService` produce the
durable, redacted workspace projection consumed by the protected local read. It also advances
Criteria 1 and 2 by preserving exact project, Context, immutable commit, decision, cohort, and
case-to-run identities through one reusable Rust application path. / 本增量让现有无 Provider 的
`BenchmarkExecutionService` 生成 protected local read 所消费的持久、脱敏 workspace projection，
直接推进条件 3；同时通过一条可复用 Rust application path 保持精确 project、Context、不可变 commit、
decision、cohort 与 case-to-run identity，从而推进条件 1 与 2。

**Gap before implementation, dependency, and risk / 实现前缺口、依赖与风险：** At planning time,
migration `0019`, memory/PostgreSQL projection writers, exact readers, and the protected API/local
SDK had been implemented, but no normal application service called
`persist_benchmark_workspace_projection`. Normal benchmark execution therefore sealed decision
evidence without creating the durable source required by the workspace GET. A naive evidence-write
followed by a projection-write could fail between transactions; because execution replayed existing
decisions without invoking the evaluator, the next replay had to repair that failure instead of
leaving permanent partial state. / 计划制定时，migration `0019`、memory/PostgreSQL projection writer、
精确 reader 与 protected API/local SDK 已实现，但还没有常规 application service 调用
`persist_benchmark_workspace_projection`。因此当时的正常 benchmark execution 会封存 decision
evidence，却不会创建 workspace GET 所需的 durable source。简单的先 evidence 后 projection 双写
可能在事务间失败；execution 对既有 decision 会无 evaluator 重放，因此下次 replay 必须能够修复
该失败，不能留下永久 partial state。

**Why this was prioritized / 当时优先原因：** At planning time, the projection persistence/read
contract and its PostgreSQL runtime evidence had been verified, and the Web/BFF consumer was being
integrated independently against the frozen DTO. Connecting the only existing execution application
service was the smallest dependency-ready change that turned durable projection rows from
test-seeded infrastructure into a real local product data path. / 计划制定时，projection persistence/
read contract 与 PostgreSQL runtime evidence 已验证，Web/BFF consumer 正在基于冻结 DTO 独立集成。
连接唯一现有 execution application service，是把测试 seed 的基础设施转化为真实本地产品数据路径
的最小依赖就绪改动。

**Explicit non-goals / 明确非目标：** No provider HTTP client, API-key access, scheduler, queue,
public or local write transport, REST/OpenAPI/public SDK change, Web mutation, benchmark definition
editing, second projection algorithm, second graph-diff calculator, release, or production claim.
`GraphDiff::between` remains the sole graph-diff calculator. / 不包含 provider HTTP client、API key
访问、scheduler、queue、public 或 local write transport、REST/OpenAPI/public SDK 变更、Web mutation、
benchmark definition 编辑、第二套 projection 算法、第二个 graph-diff calculator、release 或 production
声明。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Smallest affected boundary and bilingual documentation / 最小受影响边界与双语文档：** Expose
the existing deterministic case-to-run identity through `contextlab-evaluation`; make
`BenchmarkExecutionService` always materialize a validated `PersistBenchmarkWorkspaceProjectionV1`
after created or replayed evidence; delegate memory parity to the existing in-memory projection
repository and PostgreSQL to the existing writer; return the exact projection scope and independent
projection disposition from the private service; add focused recovery tests; then update this plan
and bilingual architecture evidence. No schema or route changes are admitted. / 在
`contextlab-evaluation` 中公开既有确定性
case-to-run identity；让 `BenchmarkExecutionService` 在 evidence created 或 replayed 后始终物化经过
校验的 `PersistBenchmarkWorkspaceProjectionV1`；memory parity 委托给既有 in-memory projection
repository，PostgreSQL 委托给既有 writer；私有 service 返回精确 projection scope 与独立
projection disposition；补充聚焦恢复测试，再更新本计划及双语 architecture evidence。不准入
schema 或 route 变更。

**Planned verification / 计划验证：** The implementation was required to observe focused RED, then
GREEN for created execution producing an exact readable projection, identical replay avoiding
evaluator calls, a simulated first projection failure followed by evidence-backed replay repair,
missing/conflicting stored runs failing closed, memory reader parity, and compiled PostgreSQL adapter
coverage. The closure gate required formatting, focused evaluation/storage tests, strict workspace
Clippy, the Rust 1.85 workspace check, full workspace tests, `pnpm check:web`, and local contract
verifiers. PostgreSQL runtime could be claimed only after an observed disposable local run;
`SQL_ASCII` evidence could not be represented as production encoding readiness. / 实现必须先观察聚焦
RED，再取得以下 GREEN：created execution 产生可精确读取的 projection、identical replay 不再次调用
evaluator、模拟首次 projection 失败后由 evidence-backed replay 修复、缺失或冲突 stored run fail
closed、memory reader parity，以及可编译的 PostgreSQL adapter coverage。收束门禁要求运行格式检查、
聚焦 evaluation/storage test、严格 workspace Clippy、Rust 1.85 workspace check、完整 workspace test、
`pnpm check:web` 与本地 contract verifier。只有实际观察 disposable local run 后才能声明 PostgreSQL
runtime；`SQL_ASCII` 证据不得表述为 production encoding readiness。

## Tasks / 任务

- [x] Add focused execution-to-workspace RED and replay-repair RED. / 增加 execution-to-workspace
  与 replay-repair 聚焦红灯。
- [x] Expose deterministic run identity and reconstruct a receipt from exact stored evidence. / 暴露
  确定性 run identity，并从精确 stored evidence 重建 receipt。
- [x] Materialize through the existing memory/PostgreSQL projection writer after create or replay. /
  在 create 或 replay 后通过既有 memory/PostgreSQL projection writer 完成物化。
- [x] Prove failure recovery, exact read scope, and no extra evaluator call. / 证明 failure recovery、
  精确 read scope 与不发生额外 evaluator call。
- [x] Normalize replay identity to PostgreSQL microsecond timestamp precision and add the regression.
  / 将 replay identity 统一到 PostgreSQL 微秒 timestamp 精度，并增加 regression。
- [x] Add an ignored exact PostgreSQL producer test and observe it against a disposable local server.
  / 增加一项 ignored 的精确 PostgreSQL producer test，并在 disposable local server 上实际观测。
- [x] Run fresh scope-matched verification and update bilingual evidence. / 运行新鲜且 scope-matched
  的验证，并更新双语证据。

## Observed Evidence / 已观察证据

- RED: `cargo test -p contextlab-evaluation --test benchmark_execution` failed with the expected
  nine missing deterministic identity/reconstruction API diagnostics before production changes. / 在
  production 修改前，该命令按预期因九处确定性 identity/reconstruction API 缺失而失败。
- RED: `cargo test -p contextlab-storage --test benchmark_execution` failed with the expected ten
  missing materialization result/error and in-memory projection-port diagnostics before storage
  implementation. / storage 实现前，该命令按预期因十处物化 result/error 与内存 projection port
  缺失而失败。
- GREEN: `cargo test -p contextlab-evaluation --test benchmark_execution --test
  benchmark_execution_receipt --test benchmark_workspace_projection` passed 15/15 tests. / 15/15
  测试通过。
- GREEN: `cargo test -p contextlab-storage --test benchmark_execution --test benchmark_evidence
  --test benchmark_workspace_projection` passed 38/38 tests after adding the sub-microsecond replay
  regression. Coverage includes retry repair, exact read, missing/altered run fail-closed behavior,
  projection conflict propagation, memory parity, and PostgreSQL-compatible timestamp identity. /
  新增亚微秒 replay regression 后，38/38 测试通过；覆盖 retry repair、exact read、缺失或篡改 run
  的 fail-closed、projection conflict 传播、memory parity 与 PostgreSQL-compatible timestamp
  identity。
- GREEN: created execution materializes a readable projection with projection disposition
  `Created`; identical evidence replay returns execution/projection `Replayed` without evaluator
  recall. An injected first projection-write failure leaves sealed evidence, and the next replay
  repairs the projection with execution `Replayed` and projection `Created`, still without another
  evaluator call. / created execution 会物化可读取 projection，并返回 projection disposition
  `Created`；相同 evidence replay 返回 execution/projection `Replayed`，不再次调用 evaluator。注入
  的首次 projection-write failure 会保留 sealed evidence；下一次 replay 以 execution `Replayed`、
  projection `Created` 修复 projection，仍不再次调用 evaluator。
- GREEN: `BenchmarkExecutionResult` returned the exact project/Context/commit/cohort
  `BenchmarkWorkspaceProjectionReceiptScope` and the independent workspace projection disposition
  for created, replayed, and repaired outcomes. / `BenchmarkExecutionResult` 对 created、replayed 与
  repaired outcome 均返回精确 project/Context/commit/cohort
  `BenchmarkWorkspaceProjectionReceiptScope` 及独立 workspace projection disposition。
- FORMAT/QUALITY: `cargo fmt --all -- --check`, strict workspace Clippy, and the locked Rust `1.85`
  workspace check passed. / `cargo fmt --all -- --check`、严格 workspace Clippy 与锁定 Rust `1.85`
  的 workspace check 均通过。
- WORKSPACE: `cargo test --workspace --quiet` passed with API `162` and storage `166 passed, 38
  ignored`. / workspace 全量测试通过，其中 API 为 `162`，storage 为 `166 passed, 38 ignored`。
- WEB/CONTRACT: `pnpm check:web` passed public SDK `14`, local SDK `59`, Web `138`, and the production
  build. Both static contract verifiers passed; updated desktop/mobile Playwright smoke observed no
  horizontal overflow or console errors. / `pnpm check:web` 通过 public SDK `14`、local SDK `59`、
  Web `138` 与 production build；两项静态 contract verifier 通过；更新后的桌面/移动端 Playwright
  smoke 未观察到横向 overflow 或 console error。
- POSTGRESQL: the exact ignored
  `postgres_benchmark_execution_materializes_and_replays_workspace_projection` test passed `1
  passed` against a disposable loopback PostgreSQL `16.14` service; the server was stopped after the
  run. The database used `SQL_ASCII`, so this is bounded producer runtime evidence, not production
  encoding/Unicode readiness, migration safety, release, or operational readiness. / 精确选择的 ignored
  test `postgres_benchmark_execution_materializes_and_replays_workspace_projection` 在 disposable、
  loopback-only PostgreSQL `16.14` service 上以 `1 passed` 通过，随后 server 已停止。database 使用
  `SQL_ASCII`，因此这只是有界 producer runtime 证据，不代表 production encoding/Unicode
  readiness、migration safety、release 或 operational readiness。
- UNOBSERVED/DEFERRED: authenticated browser-to-BFF-to-Axum runtime and Git change-set evidence
  remain unobserved. Remote CI, operator rehearsal, public promotion, release, and production remain
  deferred. / authenticated browser-to-BFF-to-Axum runtime 与 Git change-set evidence 仍未观测；
  remote CI、operator rehearsal、public promotion、release 与 production 仍为 deferred。
