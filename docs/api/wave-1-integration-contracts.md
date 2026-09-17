# Wave 1 API Integration Contracts / Wave 1 API 集成契约

## Scope / 范围

Wave 1 domain work reaches API, SDK, and Web only through an admitted integration boundary. The domain owner supplies typed records and deterministic fixtures; the Integration Lead composes transport and workspace membership. H documents the boundary and does not add a route.

Wave 1 领域工作只能通过已准入的集成边界进入 API、SDK 与 Web。领域 owner 提供类型化 record 与确定性 fixture；Integration Lead 负责 transport 与 workspace membership composition。H 负责记录边界，不新增 route。

This page supplements `docs/api/rest-api.md` and `docs/architecture/wave-1-contracts.md`. The checked-in public OpenAPI and public SDK remain the authority for the current public surface.

本页补充 `docs/api/rest-api.md` 与 `docs/architecture/wave-1-contracts.md`。已检入的 public OpenAPI 与 public SDK 仍是当前公开 surface 的权威来源。

## Public and Private Surfaces / 公共与私有 Surface

| Surface / Surface | Allowed Wave 1 use / 允许的 Wave 1 用途 | Not implied / 不代表 |
| --- | --- | --- |
| Public REST/OpenAPI/public SDK | Existing discovery reads and the already-admitted GraphDiff contract. | No new Wave 1 write, benchmark execution, plugin loading, or provider transport. |
| Private local API and local SDK | Exact-scope authenticated reads or explicitly guarded local lifecycle workflows where an existing contract admits them. | No public promotion, operator transport, or production readiness. |
| Same-origin BFF | Request-scoped Bearer forwarding, cookies omitted, `private, no-store` response behavior. | No credential persistence, refresh, logging, or browser-auth evidence. |
| Web data/presenter/screen/inspector | Adapt strict typed local payloads, or explicitly labeled deterministic fixtures, through shared design-system primitives. / 通过共享 design-system primitive 适配严格类型化的 local payload，或明确标记的确定性 fixture。 | No policy, semantic diff, authorization, or storage logic in UI code. / UI code 不包含 policy、semantic diff、authorization 或 storage logic。 |

## Handoff Contract / 交接契约

Every API/SDK/Web handoff includes the following fields in its review note:

每次 API/SDK/Web 交接都必须在审阅记录中包含以下字段：

1. `owner`: Wave 1 owner and exclusive source boundary.
2. `schema_version`: DTO/projection version and its parser or validation test.
3. `scope`: workspace, project, Context, commit, decision, or other exact scope.
4. `ordering`: stable ordering for every collection, cursor, or history.
5. `errors`: typed domain errors and their fail-closed transport mapping.
6. `surface`: public, private local, preview/fixture, or unavailable.
7. `evidence`: focused command, result, and remaining ignored/unobserved/blocked boundary.

1. `owner`：Wave 1 owner 与独占 source boundary。
2. `schema_version`：DTO/projection 版本及其 parser 或 validation test。
3. `scope`：workspace、project、Context、commit、decision 或其他精确作用域。
4. `ordering`：每个 collection、cursor 或 history 的稳定排序。
5. `errors`：类型化 domain error 及其 fail-closed transport mapping。
6. `surface`：public、private local、preview/fixture 或 unavailable。
7. `evidence`：聚焦 command、结果，以及仍然 ignored/unobserved/blocked 的边界。

## Pairwise Integration Requirements / 成对集成要求

| Pair / 配对 | Required contract proof / 必需契约证明 |
| --- | --- |
| A + G | Select exact private benchmark decision/run-detail/diff reads from redacted history; preserve private/no-store BFF behavior. |
| B + G | Render the domain diff result through a presenter; no page-local threshold or diff calculation. |
| C + E | Resolve workflow capabilities by version and fail closed for missing or incompatible plugin/MCP capabilities. |
| D + E | Preserve citation, retention, and capability boundaries without provider secrets or raw private content. |
| F + G | Consume the same Rust/domain DTOs through CLI, Desktop, and Web adapters; do not fork business logic. |
| G + H | Verify loading, error, empty, accessibility, responsive, and bilingual states against shared components and this evidence vocabulary. |

## Route and Workspace Admission / Route 与 Workspace 准入

An owner must not edit `Cargo.toml`, `package.json`, `pnpm-workspace.yaml`, the public route catalog, OpenAPI, or public SDK registration to make a dependency visible. The Integration Lead admits the dependency after checking the typed contract, fixture, test, and boundary note. Until then, the consumer reports a typed `unavailable` state or uses an explicitly labeled deterministic fixture.

owner 不得通过编辑 `Cargo.toml`、`package.json`、`pnpm-workspace.yaml`、public route catalog、OpenAPI 或 public SDK registration 来让依赖变得可见。Integration Lead 会在检查类型化 contract、fixture、test 与 boundary note 后准入该依赖。在此之前，consumer 必须报告类型化 `unavailable` state，或使用明确标记的确定性 fixture。

`docs/api/openapi.json` remains a checked-in public contract. `packages/ts-sdk` remains aligned with it through its existing contract tests. A Wave 1 local-only contract must remain outside the public catalog unless a separate decision and evidence record explicitly admits promotion.

`docs/api/openapi.json` 仍是已检入的 public contract。`packages/ts-sdk` 通过既有 contract test 与其保持对齐。Wave 1 local-only contract 必须留在 public catalog 之外，除非单独的决策与证据记录明确准入 promotion。

## Durable Benchmark Workspace Protected Read / 持久 Benchmark Workspace 受保护读取

This increment admits one private local read over the existing redacted
`BenchmarkWorkspaceProjectionV1`. It does not admit a public API. / 本增量准入一条读取既有脱敏
`BenchmarkWorkspaceProjectionV1` 的 private local path，不准入 public API。

| Handoff field / 交接字段 | Frozen contract / 冻结契约 |
| --- | --- |
| `owner` | `contextlab-storage` owns execution materialization and durable source reconstruction; `server/api` owns protected transport; `packages/local-sdk` owns strict local parsing/client behavior; `apps/web` owns the same-origin BFF and presentation adapters. / `contextlab-storage` 负责 execution materialization 与持久 source reconstruction；`server/api` 负责 protected transport；`packages/local-sdk` 负责严格 local parser/client；`apps/web` 负责同源 BFF 与 presentation adapter。 |
| `schema_version` | Envelope `contextlab.local-benchmark-workspace.v1`; nested projection `1`. / envelope 为 `contextlab.local-benchmark-workspace.v1`；内层 projection 为 `1`。 |
| `scope` | Exact project, Context, revised commit and cohort; optional exact baseline commit and cohort. / 精确 project、Context、revised commit 与 cohort；可选精确 baseline commit 与 cohort。 |
| `ordering` | Storage retains ordered case-to-run provenance; the projection and local parser require deterministic unique dataset, case/run, metric, and metric-change order. / storage 保留有序 case-to-run provenance；projection 与 local parser 要求 dataset、case/run、metric 与 metric-change 顺序确定且唯一。 |
| `surface` | Protected local GET, non-public local SDK, same-origin BFF, and local Web inspector only. / 仅 protected local GET、非公开 local SDK、同源 BFF 与 local Web inspector。 |

### Persistence And Reconstruction / 持久化与重建

Migration `0019_benchmark_workspace_projection_receipts.sql` stores the exact
`(project_id, context_id, context_commit_id, decision_id, cohort_id)` receipt and every ordered
`(dataset_id, case_id) -> run_id` link. Foreign keys bind case and run provenance to those exact
scopes. A composite foreign key also binds the receipt's canonical evidence digest to the exact
sealed `benchmark_decision_evidence` row. Completeness validation is `DEFERRABLE INITIALLY
DEFERRED`; successful validation automatically seals the receipt, while update/delete and
post-seal provenance inserts are rejected. / Migration
`0019_benchmark_workspace_projection_receipts.sql` 保存精确
`(project_id, context_id, context_commit_id, decision_id, cohort_id)` receipt，以及每条有序
`(dataset_id, case_id) -> run_id` link。foreign key 将 case 与 run provenance 绑定到这些精确
scope；另一条复合 foreign key 将 receipt 的 canonical evidence digest 绑定到精确且已 seal 的
`benchmark_decision_evidence` row。完整性校验采用 `DEFERRABLE INITIALLY DEFERRED`；校验成功后
自动 seal receipt，同时拒绝 update/delete 与 seal 后 provenance insert。

An existing cohort is returned as `Replayed` only after PostgreSQL reloads and validates its seal,
ordered provenance, decision evidence, suite, datasets, and exact runs; rebuilds the execution plan
and receipt; and proves the complete reconstructed persistence command equals the submitted command.
Single and baseline/revised reads fully reconstruct their sources inside one PostgreSQL
`REPEATABLE READ`, `READ ONLY` transaction before projecting. / 既有 cohort 只有在 PostgreSQL
重新加载并校验 seal、有序 provenance、decision evidence、suite、dataset 与精确 run，重建
execution plan 与 receipt，并证明完整 reconstruction command 等于提交 command 后，才会返回
`Replayed`。单项与 baseline/revised 读取都会在同一个 PostgreSQL `REPEATABLE READ`、`READ ONLY`
transaction 内完整重建 source，再生成 projection。

### HTTP Contract / HTTP 契约

Route:

```text
GET /api/v1/local/projects/{project_id}/contexts/{context_id}/commits/{commit_id}/benchmark-workspace/{cohort_id}
```

`baseline_commit_id` and `baseline_cohort_id` are optional only as a complete pair. Unknown query
keys, partial pairs, and malformed UUIDs fail closed. The response echoes exact project, Context,
revised scope, optional baseline scope, and the safe projection. / `baseline_commit_id` 与
`baseline_cohort_id` 只能成对省略或提供；unknown query key、不完整参数对与错误 UUID 都会 fail
closed。response 会回显精确 project、Context、revised scope、可选 baseline scope 与安全 projection。

The protected router authenticates the Bearer credential before charging the dedicated
`BenchmarkWorkspaceRead` quota. The handler then evaluates and records the
`ContextPermission::Read` RBAC decision before resolving the optional reader from `AppState` or
calling the repository. Audit failure is fail closed. Middleware applies `Cache-Control: private,
no-store` to success and error responses. / protected router 会先认证 Bearer credential，再计入独立
`BenchmarkWorkspaceRead` quota。handler 随后评估并记录 `ContextPermission::Read` RBAC decision，
之后才从 `AppState` 解析 optional reader 或调用 repository。audit failure 会 fail closed。
middleware 对成功和错误 response 都施加 `Cache-Control: private, no-store`。

### Stable Mappings / 稳定映射

| Condition / 条件 | HTTP | Error code / 错误 code |
| --- | ---: | --- |
| Credentials missing/invalid / credential 缺失或无效 | 401 | `authentication_required` / `authentication_failed` |
| Quota exhausted / quota 耗尽 | 429 | `rate_limit_exceeded` |
| Rate limiter unavailable / rate limiter 不可用 | 503 | `rate_limit_unavailable` |
| Context read denied / Context read 被拒绝 | 403 | `context_read_forbidden` |
| Authorization/audit unavailable / authorization/audit 不可用 | 503 | `authorization_unavailable` / `authorization_audit_unavailable` |
| Invalid scope/query contract / scope 或 query contract 无效 | 400 | `invalid_benchmark_workspace_request` |
| Exact receipt absent / 精确 receipt 不存在 | 404 | `benchmark_workspace_not_found` |
| Incompatible comparison / comparison 不兼容 | 409 | `benchmark_workspace_comparison_unavailable` |
| Conflicting, invalid, or scope-drifting stored source / stored source 冲突、损坏或 scope drift | 409 | `benchmark_workspace_source_conflict` |
| Reader not composed / reader 未组合 | 503 | `benchmark_workspace_unavailable` |
| Repository unavailable / repository 不可用 | 500 | `benchmark_workspace_unavailable` |

### SDK And Public Exclusion / SDK 与公开面排除

`ContextLabLocalClient.getBenchmarkWorkspace` and `parseLocalBenchmarkWorkspace` exist only in
`packages/local-sdk`. The client URL-encodes every scope, sends a request-scoped Bearer token with
cookies/credentials omitted, and rejects a response outside the requested exact scope. The parser
rejects unknown keys, raw fields, schema drift, unstable or duplicate ordering, inconsistent
counts/coverage, and receipt/diff scope drift. The public router, `docs/api/openapi.json`, and
`packages/ts-sdk` have no Benchmark workspace addition. This path does not calculate graph changes;
`GraphDiff::between` remains the sole graph-diff calculator. /
`ContextLabLocalClient.getBenchmarkWorkspace` 与 `parseLocalBenchmarkWorkspace` 只存在于
`packages/local-sdk`。client 会 URL-encode 每个 scope，只发送 request-scoped Bearer token，省略
cookie/credential，并拒绝超出请求精确 scope 的 response。parser 会拒绝 unknown key、raw field、
schema drift、不稳定或重复排序、不一致的 count/coverage，以及 receipt/diff scope drift。public
router、`docs/api/openapi.json` 与 `packages/ts-sdk` 均未增加 Benchmark workspace contract。该 path
不计算 graph change；`GraphDiff::between` 仍是唯一 graph-diff calculator。

### Execution Producer Contract / 执行 Producer 契约

`BenchmarkExecutionService` now materializes a validated
`PersistBenchmarkWorkspaceProjectionV1` after both `Created` and `Replayed` evidence outcomes. It
reloads the exact sealed suite, ordered datasets, and the evidence's exact runs; rebuilds the plan;
verifies the domain-owned deterministic `(dataset_id, case_id) -> run_id` mapping; reconstructs the
receipt; and calls the existing `BenchmarkWorkspaceProjectionV1Writer`. Memory and PostgreSQL use
that same writer contract. / `BenchmarkExecutionService` 现在会在 evidence 为 `Created` 或
`Replayed` 后物化经过校验的 `PersistBenchmarkWorkspaceProjectionV1`。它重新加载精确 sealed suite、
有序 dataset 与 evidence 绑定的精确 run，重建 plan，校验由 domain 拥有的确定性
`(dataset_id, case_id) -> run_id` 映射，重建 receipt，并调用既有
`BenchmarkWorkspaceProjectionV1Writer`；memory 与 PostgreSQL 使用同一 writer contract。

If evidence is sealed but the first projection write fails, the next exact replay repairs the
projection from stored definitions and runs without another evaluator call. Timestamp identity is
compared at PostgreSQL microsecond precision, preventing sub-microsecond adapter normalization from
breaking replay. `BenchmarkExecutionResult` returns the evidence disposition plus the exact
project/Context/commit/cohort projection scope and its independent projection write disposition. /
若 evidence 已封存而首次 projection write 失败，下一次精确 replay 会从已存 definition 与 run 修复
projection，且不再次调用 evaluator。timestamp identity 按 PostgreSQL 微秒精度比较，避免 adapter
的亚微秒 normalization 破坏 replay。`BenchmarkExecutionResult` 会返回 evidence disposition、精确
project/Context/commit/cohort projection scope，以及独立的 projection write disposition。

### Same-Origin BFF And Web Contract / 同源 BFF 与 Web 契约

The same-origin BFF route mirrors the exact revised path scope and accepts
`baseline_commit_id`/`baseline_cohort_id` only as one complete optional pair. It extracts a strict
Bearer value from the request, creates the non-public local client from the server-owned API base
URL, and forwards with cookies omitted and no-store behavior. Both success and mapped failure
responses use `Cache-Control: private, no-store`; no token is persisted, refreshed, logged, or
placed in a cookie. / 同源 BFF route 镜像精确 revised path scope，且只把
`baseline_commit_id`/`baseline_cohort_id` 作为一组完整可选参数接受。它从 request 提取严格 Bearer
值，使用 server-owned API base URL 创建非公开 local client，并以省略 cookie 和 no-store 的方式
转发。成功与映射后的失败 response 都使用 `Cache-Control: private, no-store`；token 不会被持久化、
刷新、记录或放入 cookie。

The Web boundary is `data -> presenter -> screen`, composed by
`LocalBenchmarkWorkspaceInspector`. Data calls only the same-origin BFF, uses `credentials: "omit"`
and `cache: "no-store"`, reparses the response with `parseLocalBenchmarkWorkspace`, requires exact
scope echo, and deep-freezes the accepted value. The presenter maps only the server-owned safe
projection; the screen uses shared `@contextlab/ui` primitives. Neither layer recalculates scorecard
thresholds, regression status, or evaluation diff. / Web boundary 为 `data -> presenter -> screen`，
由 `LocalBenchmarkWorkspaceInspector` 组合。data 只调用同源 BFF，使用 `credentials: "omit"` 与
`cache: "no-store"`，通过 `parseLocalBenchmarkWorkspace` 再次解析 response，要求精确 scope echo，
并深度冻结通过校验的值。presenter 只映射 server-owned safe projection，screen 使用共享
`@contextlab/ui` primitive；两层均不重新计算 scorecard threshold、regression status 或 evaluation
diff。

The inspector exposes exactly five states: `loading`, `error`, `empty`, `available`, and
`unavailable`. Controls, live/status semantics, labels, table names, and empty messages are
bilingual and accessible. The existing sealed decision-evidence and exact decision-diff inspectors
remain mounted. The parent workspace keys the inspector by project, Context, and the complete
candidate commit list, so a project/Context/candidate change remounts and clears request-memory
credentials and stale local scope. / inspector 精确暴露 `loading`、`error`、`empty`、`available` 与
`unavailable` 五种状态。control、live/status semantics、label、table name 与 empty message 均为双语
且可访问。既有 sealed decision-evidence 与 exact decision-diff inspector 继续挂载。父 workspace 以
project、Context 与完整 candidate commit list 为 inspector 生成 key，因此 project/Context/candidate
变化时会 remount，并清除 request-memory credential 与陈旧 local scope。

## Verification Boundary / 验证边界

Fresh closure evidence passed focused evaluation `15/15`, focused storage `38/38` after the
timestamp-normalization regression, `cargo fmt --all -- --check`, strict workspace Clippy,
`cargo test --workspace --quiet` (API `162`; storage `166 passed, 38 ignored`), and the Rust `1.85`
workspace check. `pnpm check:web` passed public SDK `14`, local SDK `59`, Web `138`, and the
production build. Both static contract verifiers and the updated desktop/mobile Playwright smoke
passed with no horizontal overflow or console errors. / 新鲜收束证据已通过 evaluation 聚焦测试
`15/15`、timestamp-normalization regression 后的 storage 聚焦测试 `38/38`、
`cargo fmt --all -- --check`、严格 workspace Clippy、`cargo test --workspace --quiet`（API `162`；
storage `166 passed, 38 ignored`）以及 Rust `1.85` workspace check。`pnpm check:web` 通过 public
SDK `14`、local SDK `59`、Web `138` 与 production build。两项静态 contract verifier 和更新后的
桌面/移动端 Playwright smoke 均已通过，且无横向 overflow 或 console error。

One exact ignored producer test then passed with `1 passed` against a disposable loopback
PostgreSQL `16.14` service, and that server was stopped afterward. / 随后一项精确选择的 ignored
producer test 在 disposable、loopback-only PostgreSQL `16.14` service 上以 `1 passed` 通过，随后
server 已停止。

That disposable database used `SQL_ASCII`. The result proves this bounded projection runtime only;
it is not evidence of production encoding/Unicode readiness, production migration safety, remote
operation, or release readiness. The remaining PostgreSQL tests stay ignored in the ordinary
workspace run. Authenticated browser-to-BFF-to-Axum runtime and Git change-set evidence remain
unobserved; remote CI, operator rehearsal, release, public promotion, and production behavior remain
deferred. / 该
disposable database 使用 `SQL_ASCII`。该结果只证明本次有界 projection runtime，不能证明
production encoding/Unicode readiness、production migration safety、remote operation 或 release
readiness。其余 PostgreSQL test 在普通 workspace run 中仍保持 ignored。Git binding 不可用；
authenticated browser-to-BFF-to-Axum runtime 与 Git change-set evidence 仍为 unobserved；remote
CI、operator rehearsal、release、public promotion 与 production behavior 仍为 deferred。

## Private Benchmark Decision Pair Witness / 私有 Benchmark Decision Pair Witness

The protected benchmark decision-diff response may include an optional
`decision_pair_witness` only when the comparison is decision-bound. The witness is
server-owned, immutable for the response, and must echo the exact requested
`project_id`, `context_id`, `baseline_commit_id`, `revised_commit_id`,
`baseline_decision_id`, and `revised_decision_id`, with `schema_version=1`.
`baseline` and `revised` are ordered roles; neither commit nor decision may be a
self-pair. / 受保护的 benchmark decision-diff response 仅在 comparison 绑定具体
decision 时，可选返回 `decision_pair_witness`。该 witness 为 server-owned，且在本次
response 内不可变；必须回显请求中的精确 `project_id`、`context_id`、
`baseline_commit_id`、`revised_commit_id`、`baseline_decision_id` 与
`revised_decision_id`，并使用 `schema_version=1`。`baseline` 与 `revised` 是有序角色；
commit 与 decision 均不得形成 self-pair。

The private local SDK requires the witness and rejects missing, null, unknown-key,
wrong-version, malformed, mixed-scope, mismatched, or self-pair values before Web
presentation. The API and SDK fail closed: they do not infer identity from a cohort,
commit, dataset, or local selection. / 非公开 local SDK 必须要求该 witness，并在进入
Web presentation 前拒绝缺失、null、unknown key、错误版本、格式错误、混合 scope、
不匹配或 self-pair 值。API 与 SDK 均 fail closed；不得从 cohort、commit、dataset 或
本地选择推断 decision identity。

The witness is a safe identity projection only. Dataset IDs, raw cases, inputs,
expected outputs, measurements, model outputs, provider details, credentials, and
secrets remain redacted. The route, local SDK, and same-origin BFF are private local
surfaces; this increment adds no public REST/OpenAPI/public SDK entry, public write,
operator transport, migration, provider, scheduler, or production claim. / witness
仅是安全 identity projection。dataset ID、raw case、input、expected output、measurement、
model output、provider detail、credential 与 secret 均保持脱敏。route、local SDK 与同源
BFF 均为 private local surface；本增量不新增 public REST/OpenAPI/public SDK entry、
public write、operator transport、migration、provider、scheduler 或 production 声明。

The detailed contract and ownership ledger live in
`docs/architecture/private-benchmark-decision-diff.md`. / 详细契约与 ownership ledger
见 `docs/architecture/private-benchmark-decision-diff.md`。

### Private Benchmark Definition Binding / 私有 Benchmark 定义绑定

BenchmarkDefinitionBindingCommand is a private Rust/storage contract for authoring one complete
suite and its datasets at an exact Context commit. It carries schema version 1, stable binding
identity, project/Context/commit scope, branch and expected-head guard, issuer-scoped principal,
idempotency key, request digest, capture time, and immutable definitions. The suite dataset IDs
must exactly equal the supplied dataset IDs in stable order. Memory and PostgreSQL expose the same
writer disposition (created or replayed) and exact read/list ports. PostgreSQL verifies write
authorization and branch head inside one transaction, persists definitions and binding atomically,
and uses microsecond capture-time normalization for replay parity. The binding route is not yet
public or transport-complete; no public OpenAPI/public SDK write is added.

BenchmarkDefinitionBindingCommand 是私有 Rust/storage contract，用于在精确 Context commit 上创作
一条完整 suite 及其 dataset。它携带 schema version 1、稳定 binding identity、project/Context/
commit scope、branch 与 expected-head guard、issuer-scoped principal、idempotency key、request
digest、capture time 与不可变 definition。suite dataset ID 必须与传入 dataset ID 按稳定顺序
完全一致。Memory 与 PostgreSQL 暴露相同的 writer disposition（created 或 replayed）及 exact
read/list port。PostgreSQL 在一个 transaction 内复核 write authorization 和 branch head，原子
持久化 definitions 与 binding，并通过微秒级 capture-time normalization 保持 replay parity。
binding route 还未成为 public 或完整 transport；没有新增 public OpenAPI/public SDK write。

The fresh local storage gate is authoring 3/3, migration contract 6/6, workspace storage
166 passed and 38 ignored, strict storage Clippy, and 26 isolated native PostgreSQL 16.14 UTF-8
loopback tests passed after schema reset (26/26). The server was stopped afterward. This does not
prove production encoding, public write readiness, authenticated browser runtime, Git, remote CI,
operator rehearsal, release, or production.

新鲜 local storage gate 包括 authoring 3/3、migration contract 6/6、workspace storage
166 passed and 38 ignored、严格 storage Clippy，以及 schema reset 后 native PostgreSQL 16.14
UTF-8 loopback 的 26 个隔离 test 全部通过（26/26）。server 随后已停止。这不能证明 production
encoding、public write readiness、authenticated browser runtime、Git、remote CI、operator
rehearsal、release 或 production。
