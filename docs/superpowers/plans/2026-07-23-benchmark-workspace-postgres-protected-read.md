# Benchmark Workspace PostgreSQL + Protected Read Plan / Benchmark Workspace PostgreSQL 与受保护读取计划

## Closure Status / 收束状态

The durable Benchmark workspace projection, execution producer, and protected local read increment
are implemented and locally verified. The product path now materializes the projection after both
created and replayed evidence; this plan is retained as the storage/read history. / 持久 Benchmark
workspace projection、execution producer 与受保护本地读取增量已经实现并完成本地验证。当前 product
path 会在 created 与 replayed evidence 后物化 projection；本文保留为 storage/read 历史记录。

## Necessity Record / 必要性记录

**Completion criteria and charter principles / 收束条件与宪章原则：** This increment advances
Criterion 1 through a strict local inspection contract, Criterion 3 through durable and queryable
projection provenance, Criterion 6 through authenticated, rate-limited, authorized, and audited
reads, and Criterion 9 through bilingual architecture and evidence. Every source is bound to one
exact project, Context, immutable Context commit, decision, and execution cohort; each case is bound
to its exact run. / 本增量通过严格的本地审阅契约推进条件 1，通过持久且可查询的 projection
provenance 推进条件 3，通过经过认证、限流、授权与审计的读取推进条件 6，并通过双语架构和证据
推进条件 9。每份 source 都绑定到精确的 project、Context、不可变 Context commit、decision 与
execution cohort；每条 case 都绑定到其精确 run。

**Delivered boundary / 已交付边界：** Migration
`0019_benchmark_workspace_projection_receipts.sql`, PostgreSQL writer/reader parity, one protected
local GET, and a strict non-public local SDK are implemented. The public router, checked-in OpenAPI,
and public TypeScript SDK are unchanged. `GraphDiff::between` remains the sole graph-diff
calculator. / 已实现 migration `0019_benchmark_workspace_projection_receipts.sql`、PostgreSQL
writer/reader parity、一条 protected local GET 与严格的非公开 local SDK。public router、已检入的
OpenAPI 与 public TypeScript SDK 均未改变。`GraphDiff::between` 仍是唯一 graph-diff calculator。

**Explicit non-goals / 明确非目标：** No provider/evaluator transport, benchmark definition
editing, policy or scorecard recalculation, raw case/input/expected-output/model-output response,
public promotion, local write route, operator transport, secret read, authenticated browser E2E,
remote CI, release, or production claim is included. / 不包含 provider/evaluator transport、
benchmark definition 编辑、policy 或 scorecard 重算、raw case/input/expected-output/model-output
response、public promotion、local write route、operator transport、secret 读取、authenticated
browser E2E、remote CI、release 或 production 声明。

## Durable Projection Contract / 持久 Projection 契约

- Migration `0019` stores one immutable receipt at exact
  `(project_id, context_id, context_commit_id, decision_id, cohort_id)` scope and ordered
  `(dataset_id, case_id) -> run_id` provenance. Database foreign keys bind each case to the exact
  project dataset case and exact project/Context/commit/decision run. / Migration `0019` 在精确
  `(project_id, context_id, context_commit_id, decision_id, cohort_id)` scope 保存一份不可变
  receipt，并保存有序的 `(dataset_id, case_id) -> run_id` provenance。数据库 foreign key 将每条
  case 同时绑定到精确 project dataset case 与精确 project/Context/commit/decision run。
- The migration first makes the decision evidence digest unique at its exact decision scope, then
  binds each receipt to `(project_id, context_id, context_commit_id, decision_id, evidence_digest)`
  with a composite foreign key. A matching decision seal is also required. / migration 先在精确
  decision scope 约束 decision evidence digest 的唯一性，再通过复合 foreign key 将 receipt 绑定到
  `(project_id, context_id, context_commit_id, decision_id, evidence_digest)`；同时要求对应的
  decision seal 已存在。
- Completeness validation is `DEFERRABLE INITIALLY DEFERRED`. It proves contiguous positions,
  exact suite dataset/case membership, and exact decision run membership. Constraint triggers
  automatically create the receipt seal only after completeness succeeds. / 完整性校验采用
  `DEFERRABLE INITIALLY DEFERRED`，验证 position 连续、suite dataset/case membership 精确，并且
  decision run membership 精确。只有完整性验证成功后，constraint trigger 才会自动创建 receipt
  seal。
- Receipt, provenance, and seal rows reject update/delete. Provenance inserts after sealing are
  rejected, and an incomplete transaction leaves no receipt residue. / receipt、provenance 与 seal
  row 均拒绝 update/delete；seal 后继续插入 provenance 也会被拒绝。不完整 transaction 不会留下
  receipt 残留。

## Replay And Read Semantics / 重放与读取语义

PostgreSQL does not return `Replayed` from identity equality alone. It reloads the seal, ordered case
links, sealed decision evidence, suite, datasets, and exact runs; rebuilds the execution plan and
receipt; checks the evidence digest, cohort identity, case/run identities, and full
`PersistBenchmarkWorkspaceProjectionV1`; and returns `Replayed` only when that reconstructed command
equals the submitted command. Any mismatch fails closed as conflict or invalid stored source. /
PostgreSQL 不会只因 identity 相等就返回 `Replayed`。它会重新加载 seal、有序 case link、已封存
decision evidence、suite、dataset 与精确 run，重建 execution plan 和 receipt，再校验 evidence
digest、cohort identity、case/run identity 及完整 `PersistBenchmarkWorkspaceProjectionV1`；只有重建
command 与提交 command 完全相等时才返回 `Replayed`。任意不一致都会以 conflict 或 invalid stored
source fail closed。

Every PostgreSQL single or baseline/revised read runs inside one transaction configured as
`REPEATABLE READ` and `READ ONLY`. The repository fully reconstructs the revised source and optional
baseline source before producing the existing redacted `BenchmarkWorkspaceProjectionV1`; comparison
still delegates to the existing evaluation-domain diff. / 每次 PostgreSQL 单项读取或
baseline/revised 成对读取都在同一个设为 `REPEATABLE READ` 与 `READ ONLY` 的 transaction 内完成。
repository 会先完整重建 revised source 与可选 baseline source，再生成既有脱敏
`BenchmarkWorkspaceProjectionV1`；比较仍委托给既有 evaluation-domain diff。

## Frozen Local Read Contract / 冻结的本地读取契约

- Route: `GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}`.
- Optional comparison query: `baseline_commit_id` and `baseline_cohort_id` must be supplied
  together; unknown query keys fail closed. / 可选 comparison query 中，`baseline_commit_id` 与
  `baseline_cohort_id` 必须成对提供，unknown query key 会 fail closed。
- Envelope: `contextlab.local-benchmark-workspace.v1`, with exact project/Context/revised scope,
  optional exact baseline scope, and the existing safe `BenchmarkWorkspaceProjectionV1`. / envelope
  使用 `contextlab.local-benchmark-workspace.v1`，包含精确 project/Context/revised scope、可选精确
  baseline scope，以及既有安全 `BenchmarkWorkspaceProjectionV1`。
- Composition order: Bearer authentication, dedicated `BenchmarkWorkspaceRead` quota, request
  parsing, `ContextPermission::Read` RBAC decision plus mandatory audit, then repository access.
  Authentication therefore occurs before quota; RBAC/audit occur before repository access. /
  组合顺序为 Bearer authentication、独立 `BenchmarkWorkspaceRead` quota、request parsing、
  `ContextPermission::Read` RBAC decision 与强制 audit，最后才访问 repository。因此 authentication
  先于 quota，RBAC/audit 先于 repository。
- The protected router applies `Cache-Control: private, no-store` to success and error responses.
  The route also emits the same header on successful responses. / protected router 会对成功与错误
  response 统一施加 `Cache-Control: private, no-store`；route 本身也会在成功响应中设置同一 header。

### Stable Error Mappings / 稳定错误映射

| Condition / 条件 | HTTP | Stable code / 稳定 code |
| --- | ---: | --- |
| Missing or invalid credentials / 缺失或无效 credential | 401 | `authentication_required` / `authentication_failed` |
| Dedicated quota exhausted / 独立 quota 耗尽 | 429 | `rate_limit_exceeded` |
| Rate-limit state unavailable / 限流状态不可用 | 503 | `rate_limit_unavailable` |
| Context read denied / Context 读取被拒绝 | 403 | `context_read_forbidden` |
| Authorization or audit unavailable / authorization 或 audit 不可用 | 503 | `authorization_unavailable` / `authorization_audit_unavailable` |
| Malformed scope, partial pair, or unknown query / scope 损坏、参数对不完整或 unknown query | 400 | `invalid_benchmark_workspace_request` |
| Exact receipt missing / 精确 receipt 缺失 | 404 | `benchmark_workspace_not_found` |
| Baseline/revised comparison incompatible / baseline/revised 不可比较 | 409 | `benchmark_workspace_comparison_unavailable` |
| Conflicting or invalid stored source, including response scope drift / 持久 source 冲突、损坏或 response scope drift | 409 | `benchmark_workspace_source_conflict` |
| Optional reader not installed / optional reader 未安装 | 503 | `benchmark_workspace_unavailable` |
| Repository failure / repository 失败 | 500 | `benchmark_workspace_unavailable` |

## Local SDK And Public Boundary / Local SDK 与公开边界

`ContextLabLocalClient.getBenchmarkWorkspace` URL-encodes every scope segment, sends only a
request-scoped Bearer token, omits cookies and browser credentials, validates exact response scope,
and supports only a complete optional baseline pair. `parseLocalBenchmarkWorkspace` rejects unknown
keys, raw fields, malformed versions, unstable or duplicate ordering, count/coverage drift, and
receipt/diff scope drift. This client lives only in `packages/local-sdk`; no method or schema was
added to `packages/ts-sdk`, the public route catalog, or `docs/api/openapi.json`. /
`ContextLabLocalClient.getBenchmarkWorkspace` 会对每个 scope segment 做 URL encoding，只发送
request-scoped Bearer token，省略 cookie 与 browser credential，校验精确 response scope，并且只接受
完整的可选 baseline 参数对。`parseLocalBenchmarkWorkspace` 会拒绝 unknown key、raw field、错误
version、不稳定或重复排序、count/coverage drift，以及 receipt/diff scope drift。该 client 仅位于
`packages/local-sdk`；`packages/ts-sdk`、public route catalog 与 `docs/api/openapi.json` 均未新增 method
或 schema。

## Verification Receipt / 验证回执

Fresh local evidence recorded for this increment: / 本增量记录的新鲜本地证据：

- `cargo fmt --all -- --check`: passed / 通过。
- `cargo clippy --workspace --all-targets -- -D warnings`: passed / 通过。
- `cargo test --workspace --quiet`: passed; API `162 passed`; storage `166 passed, 37 ignored`.
  / 通过；API `162 passed`；storage `166 passed, 37 ignored`。
- `cargo +1.85.0 check --workspace --all-targets --locked`: passed / 通过。
- `pnpm check:web`: passed; public SDK `14`, local SDK `59`, Web `114`, and production Web build.
  / 通过；public SDK `14`、local SDK `59`、Web `114`，且 production Web build 成功。
- Local contract verifier source/DTO/`GraphDiff` checks passed. They preserve one
  application-layer `GraphDiff::between` entry point and safe local DTO fields. / 本地 contract
  verifier 的 source、DTO 与 `GraphDiff` 检查通过；application layer 保持唯一
  `GraphDiff::between` 入口，local DTO field 保持安全。
- The exact ignored test
  `postgres_benchmark_workspace_projection_creates_replays_and_reads_exact_scope` was selected
  against a disposable loopback PostgreSQL `16.14` service and passed with `1 passed`. The database
  encoding was `SQL_ASCII`, and the disposable server was stopped after the run. This proves the
  bounded projection create/replay/read, automatic seal, post-seal rejection, and deferred
  incomplete-write rollback path only. / 精确选择的 ignored test
  `postgres_benchmark_workspace_projection_creates_replays_and_reads_exact_scope` 在 disposable、
  loopback-only PostgreSQL `16.14` service 上以 `1 passed` 通过。数据库编码为 `SQL_ASCII`，运行后
  disposable server 已停止。该证据只证明有界的 projection create/replay/read、自动 seal、seal 后
  拒绝与延迟校验失败回滚路径。

`SQL_ASCII` is not evidence of production encoding readiness, Unicode correctness, production
migration safety, or long-lived database operation. Git binding is unavailable in this workspace;
authenticated browser E2E, remote CI, operator rehearsal, release, public promotion, and production
behavior remain unobserved or deferred. / `SQL_ASCII` 不能证明 production encoding readiness、
Unicode correctness、production migration safety 或长期数据库运行。当前 workspace 无可用 Git
binding；authenticated browser E2E、remote CI、operator rehearsal、release、public promotion 与
production behavior 仍为 unobserved 或 deferred。

## Superseded Product Gap / 已取代的产品缺口

The former missing-producer gap is closed by `BenchmarkExecutionService`, which reconstructs the
exact sealed definitions and runs after created or replayed evidence and persists the durable
projection through the existing writer. The remaining Benchmark gap is authoring-definition
inspection and selection, which is admitted separately by the 2026-07-27 private authoring plan. /
此前缺失 producer 的产品缺口已由 `BenchmarkExecutionService` 收束：它会在 created 或 replayed evidence 后
重建精确 sealed definition 与 run，并通过既有 writer 持久化 projection。当前 Benchmark 的剩余缺口是
authoring definition 的 inspection 与 selection，已由 2026-07-27 private authoring plan 单独准入。

## Tasks / 任务

- [x] Record the missing PostgreSQL/API/local-SDK implementation as a historical RED-evidence gap;
  no durable RED output was available, but the implementation and green receipts are now present.
  / 将 PostgreSQL/API/local SDK 缺失实现记录为历史 RED-evidence gap；虽然没有可用的持久 RED 输出，
  但实现与 green receipt 现已存在。
- [x] Implement immutable PostgreSQL writer/reader parity with the memory repository, migration
  `0019`, deferred completeness, automatic sealing, and replay reconstruction. / 实现 PostgreSQL 与
  memory repository 的不可变 writer/reader parity、migration `0019`、延迟完整性校验、自动 seal 与
  replay reconstruction。
- [x] Add the protected API DTO/route with exact authentication, dedicated quota, RBAC/audit,
  repository ordering, no-store behavior, and stable mappings. / 增加 protected API DTO/route，并
  固化 authentication、独立 quota、RBAC/audit、repository 顺序、no-store 行为与稳定映射。
- [x] Add the strict non-public local SDK parser/client and exact-scope validation while preserving
  the public router/OpenAPI/public SDK. / 增加严格的非公开 local SDK parser/client 与 exact-scope
  validation，同时保持 public router/OpenAPI/public SDK 不变。
- [x] Run focused and full local verification and update the owned bilingual architecture/API
  evidence. Roadmap files were intentionally not edited because they are outside this owner's write
  scope. / 完成聚焦与完整本地验证，并更新本 owner 范围内的双语 architecture/API 证据；roadmap 文件
  不在本 owner 写入范围内，因此未作修改。
