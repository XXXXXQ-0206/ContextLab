# Persistence Foundation / 持久化地基

ContextLab's persistence layer starts with storage contracts rather than API-owned database access. The storage crate owns schema assets and projection records; API handlers compose storage outputs and never query tables directly.

ContextLab 的持久化层从 storage contract 开始，而不是让 API 直接访问数据库。`storage` crate 负责 schema 资产与 projection record；API handler 只组合 storage 输出，不直接查询数据表。

## Migration / 迁移

The persistence foundation composes `crates/storage/migrations/0001_context_platform.sql` and `crates/storage/migrations/0002_context_commit_graph_snapshots.sql`.

持久化地基由 `crates/storage/migrations/0001_context_platform.sql` 与 `crates/storage/migrations/0002_context_commit_graph_snapshots.sql` 组合而成。

It establishes:

它建立了：

- `workspaces`
- `projects`
- `experiments`
- `contexts`
- `context_components`
- `context_commits`
- `context_commit_parents`
- `context_commit_graph_snapshots`
- `evaluation_runs`

The schema uses UUID primary keys, JSONB metadata, soft-delete timestamps where resources are user-facing, and indexes for common relationship lookups.

该 schema 使用 UUID 主键、JSONB metadata、面向用户资源的 soft-delete timestamp，并为常见关系查询建立索引。

Important migration choices:

关键迁移选择：

- Active workspace, project, and experiment names use partial unique indexes so soft-deleted rows do not block future reuse.
- 活跃的 workspace、project 和 experiment 名称使用 partial unique index，避免 soft-delete 后阻塞后续复用。
- Commit parent relationships are normalized through `context_commit_parents` instead of UUID arrays.
- Commit 父关系通过 `context_commit_parents` 规范化建模，而不是使用 UUID 数组。
- A commit graph snapshot is keyed only by `commit_id`; its Context ownership derives from the authoritative commit row instead of duplicated payload data.
- commit graph snapshot 仅以 `commit_id` 为键；其 Context ownership 从权威 commit row 推导，而不是重复存储在 payload 中。
- `context_components.kind` is constrained to the v1 Context component taxonomy, including `evaluation`.
- `context_components.kind` 被约束为 v1 Context component 分类，并包含 `evaluation`。

## Graph Projection / 图投影

`ContextGraphProjection` converts persistence records into the shared `ContextGraph` contract. This keeps graph semantics reusable by API, Web, CLI, and future storage repositories.

`ContextGraphProjection` 会把持久化记录转换成共享的 `ContextGraph` 契约，使 graph 语义可以被 API、Web、CLI 和未来 storage repository 复用。

Current API graph flows:

当前 API graph 数据流：

```text
AppState
  -> InMemoryContextGraphRepository
  -> ContextGraphProjection records
  -> ContextGraph
  -> GET /api/v1/context-graph/preview

AppState
  -> workspace ContextGraphProjectionRepository
  -> ContextGraphProjection records
  -> ContextGraph
  -> GET /api/v1/workspaces/{workspace_id}/context-graph
```

The repository boundary is intentionally async and trait-based. The preview API already depends on `ContextGraphProjectionRepository`, so a SQLx-backed repository can replace the in-memory implementation without changing graph projection semantics.

Repository 边界刻意设计为 async trait。当前 preview API 已经依赖 `ContextGraphProjectionRepository`，因此未来可以用 SQLx-backed repository 替换 in-memory 实现，而不改变 graph projection 语义。

`PostgresContextGraphRepository` is the first SQLx-backed adapter. It loads workspace-scoped projection records from PostgreSQL and keeps SQL details inside `contextlab-storage`. It uses runtime SQLx queries so the crate compiles and tests without requiring a live database during normal development.

`PostgresContextGraphRepository` 是第一版 SQLx-backed adapter。它从 PostgreSQL 读取 workspace-scoped projection records，并把 SQL 细节保留在 `contextlab-storage` 内部。它使用 SQLx runtime query，因此普通开发中的编译与测试不需要 live database。

Workspace graph reads run inside a read-only transaction so workspace, project, context, component, and evaluation records are projected from one consistent database snapshot.

Workspace graph read 会在 read-only transaction 内执行，因此 workspace、project、context、component 和 evaluation 记录会来自同一个一致的数据库快照。

The adapter intentionally does not serve preview scope. Preview remains an in-memory contract fixture, while PostgreSQL reads require an explicit workspace scope. API error mapping distinguishes unavailable scope, invalid scope, storage contract drift, projection failure, and database failure.

该 adapter 刻意不服务 preview scope。Preview 仍然是 in-memory contract fixture，而 PostgreSQL 读取需要明确的 workspace scope。API 错误映射会区分 scope unavailable、invalid scope、storage contract drift、projection failure 和 database failure。

Next step:

下一步：

```text
PostgreSQL rows
  -> PostgresContextGraphRepository
  -> ContextGraphProjection
  -> ContextGraph
  -> API and Web
```

The API state now stores `Arc<dyn ContextGraphProjectionRepository>`, keeping route handlers independent from the concrete in-memory or PostgreSQL implementation.

API state 现在保存 `Arc<dyn ContextGraphProjectionRepository>`，使 route handler 不依赖具体的 in-memory 或 PostgreSQL 实现。

## Workspace and Project Listing / Workspace 与 Project 列表

`WorkspaceRepository` owns paginated workspace discovery for API and future SDK clients. It supports normalized `page`, `per_page`, `search`, and `sort` inputs, with `sort` limited to `name`, `-name`, `created_at`, and `-created_at`.

`WorkspaceRepository` 负责面向 API 与未来 SDK client 的分页 workspace discovery。它支持规范化后的 `page`、`per_page`、`search` 和 `sort` 输入，其中 `sort` 仅允许 `name`、`-name`、`created_at` 和 `-created_at`。

Workspace list items expose `id`, `name`, `slug`, and `created_at`. The in-memory repository now reads `slug` and `created_at` from `WorkspaceRecord`, so preview behavior matches PostgreSQL instead of deriving values from `id`.

Workspace list item 会暴露 `id`、`name`、`slug` 和 `created_at`。in-memory repository 现在从 `WorkspaceRecord` 读取 `slug` 与 `created_at`，因此 preview 行为与 PostgreSQL 保持一致，而不是从 `id` 临时推导。

PostgreSQL workspace listing runs count and page-item reads inside one `REPEATABLE READ` read-only transaction. This gives clients a consistent pagination snapshot while keeping SQL details behind the storage boundary.

PostgreSQL workspace listing 会在同一个 `REPEATABLE READ` read-only transaction 中读取 count 与 page item。这能为 client 提供一致的分页快照，同时把 SQL 细节保留在 storage 边界内部。

`ProjectRepository` follows the same contract pattern for `GET /api/v1/workspaces/{workspace_id}/projects`. Project queries are workspace-scoped and support the same normalized `page`, `per_page`, `search`, and `sort` inputs.

`ProjectRepository` 为 `GET /api/v1/workspaces/{workspace_id}/projects` 沿用同一套 contract pattern。Project query 以 workspace 为作用域，并支持同样规范化后的 `page`、`per_page`、`search` 和 `sort` 输入。

Project list items expose `id`, `workspace_id`, `name`, `slug`, and `created_at`. `ProjectRecord` stores `slug` and `created_at` so in-memory preview data and PostgreSQL rows share the same listing semantics.

Project list item 会暴露 `id`、`workspace_id`、`name`、`slug` 和 `created_at`。`ProjectRecord` 存储 `slug` 与 `created_at`，使 in-memory preview 数据与 PostgreSQL 行共享同一套列表语义。

PostgreSQL project listing validates workspace ids as UUIDs before opening a query path, returns unavailable scope for missing workspaces, and reads workspace existence, count, and page items from one `REPEATABLE READ` read-only transaction.

PostgreSQL project listing 会先把 workspace id 校验为 UUID；workspace 不存在时返回 unavailable scope；workspace existence、count 与 page item 会在同一个 `REPEATABLE READ` read-only transaction 中读取。

Preview mode intentionally keeps human-readable fixture ids such as `default` for local ergonomics. PostgreSQL mode requires UUID workspace ids, so clients should treat `default` as preview-only data.

Preview 模式刻意保留 `default` 这类便于本地开发的人类可读 fixture id。PostgreSQL 模式要求 workspace id 为 UUID，因此 client 应把 `default` 视作仅用于 preview 的数据。

`ExperimentRepository` provides project-scoped discovery for `GET /api/v1/projects/{project_id}/experiments`. It uses the same normalized list contract while adding `branch_name` as a first-class response and sort field because experiments track versioning branches.

`ExperimentRepository` 为 `GET /api/v1/projects/{project_id}/experiments` 提供 project-scoped discovery。它沿用同一套规范化 list contract，同时把 `branch_name` 作为一等响应字段和排序字段，因为 experiment 会跟踪 versioning branch。

Experiment list items expose `id`, `project_id`, `name`, `branch_name`, and `created_at`. PostgreSQL experiment listing validates project ids as UUIDs before querying, returns unavailable scope for missing projects, and reads project existence, count, and page items from one `REPEATABLE READ` read-only transaction.

Experiment list item 会暴露 `id`、`project_id`、`name`、`branch_name` 和 `created_at`。PostgreSQL experiment listing 会先把 project id 校验为 UUID；project 不存在时返回 unavailable scope；project existence、count 与 page item 会在同一个 `REPEATABLE READ` read-only transaction 中读取。

`ContextRepository` provides project-scoped discovery for `GET /api/v1/projects/{project_id}/contexts`. It keeps Context as the central abstraction and supports normalized `page`, `per_page`, `search`, optional `experiment_id`, and `sort` inputs.

`ContextRepository` 为 `GET /api/v1/projects/{project_id}/contexts` 提供 project-scoped discovery。它保持 Context 作为核心抽象，并支持规范化后的 `page`、`per_page`、`search`、可选 `experiment_id` 与 `sort` 输入。

Context list items expose `id`, `project_id`, `experiment_id`, `name`, `description`, and `created_at`. PostgreSQL context listing validates project ids as UUIDs before querying, validates optional `experiment_id` filters as UUIDs before database I/O, returns unavailable scope for missing projects, and reads project existence, count, and page items from one `REPEATABLE READ` read-only transaction.

Context list item 会暴露 `id`、`project_id`、`experiment_id`、`name`、`description` 和 `created_at`。PostgreSQL context listing 会先把 project id 校验为 UUID，并在数据库 I/O 前把可选 `experiment_id` filter 校验为 UUID；project 不存在时返回 unavailable scope；project existence、count 与 page item 会在同一个 `REPEATABLE READ` read-only transaction 中读取。

`ContextCommitRepository` provides context-scoped version history discovery for `GET /api/v1/contexts/{context_id}/commits`. It supports normalized `page`, `per_page`, `search`, optional `branch_name`, and `sort` inputs while keeping full replay/change payloads out of the list response.

`ContextCommitRepository` 为 `GET /api/v1/contexts/{context_id}/commits` 提供 context-scoped version history discovery。它支持规范化后的 `page`、`per_page`、`search`、可选 `branch_name` 与 `sort` 输入，同时不在列表响应中返回完整 replay/change payload。

Commit list items expose `id`, `context_id`, `branch_name`, `message`, ordered `parent_commit_ids`, `change_count`, `authored_at`, and `created_at`. PostgreSQL commit listing validates context ids as UUIDs before querying, returns unavailable scope for missing contexts, reads ordered parent ids from `context_commit_parents`, computes `change_count` from `changes`, and reads context existence, count, and page items from one `REPEATABLE READ` read-only transaction.

Commit list item 会暴露 `id`、`context_id`、`branch_name`、`message`、有序 `parent_commit_ids`、`change_count`、`authored_at` 和 `created_at`。PostgreSQL commit listing 会先把 context id 校验为 UUID；context 不存在时返回 unavailable scope；有序 parent id 来自 `context_commit_parents`；`change_count` 从 `changes` 计算；context existence、count 与 page item 会在同一个 `REPEATABLE READ` read-only transaction 中读取。

`CreateContextCommitSnapshot` and `ContextCommitSnapshotWriter` are the storage-only capture boundary. One accepted command creates a `ContextCommit`, its ordered same-Context parent relationships, and exactly one `CommitGraphSnapshot`. The in-memory adapter holds the new records in one synchronized overlay. PostgreSQL locks/checks the active Context and all parents, then inserts all three record classes in a single transaction; a validation or database failure commits none of them.

`CreateContextCommitSnapshot` 与 `ContextCommitSnapshotWriter` 是仅限 storage 的 capture 边界。一个被接受的 command 会创建 `ContextCommit`、其有序且同 Context 的 parent relationship，以及唯一的 `CommitGraphSnapshot`。内存 adapter 在一个同步 overlay 中保存新记录。PostgreSQL 会锁定并检查活跃 Context 与全部 parent，再在一个 transaction 中插入全部三类 record；validation 或 database failure 都不会提交其中任何一类。

API/SDK compose this read boundary into a read-only version-backed GraphDiff for two materialized commits in one Context. Storage also exposes the guarded commit writer, and the API has an explicitly configured protected route with authentication, authorization, idempotency, and branch-head compare-and-swap. This is not a public mutation workflow: the default route catalog and API/SDK write surface remain closed by the current local admission decision. Any future promotion requires a separate decision when external deployment conditions exist; merge semantics, snapshot detail APIs, and graph editing remain separate platform work.

API/SDK 已把这个 read boundary 组合为同一 Context 内两个 materialized commit 的只读 version-backed GraphDiff。storage 还提供 guarded commit writer，API 也在显式配置的 protected route 后提供 authentication、authorization、idempotency 与 branch-head compare-and-swap。这不是 public mutation workflow：根据当前本地准入决策，default route catalog 与 API/SDK write surface 保持关闭。未来只有在外部部署条件具备时，才能通过单独决策考虑推广；merge semantic、snapshot detail API 与 graph editing 仍属于独立的平台工作。

`PostgresProtectedRouteRateLimiter` is a separate private storage adapter for the existing
`contextlab-auth::ProtectedRouteRateLimiter` port. Migration `0010_shared_protected_route_rate_limits.sql`
stores one deployment policy/clock guard and one bounded timestamp queue keyed by exact issuer, subject,
and protected operation. Each decision takes a transaction-scoped advisory lock before it reclaims expiry,
checks capacity, and updates or rejects the requested queue, so independent pools cannot each spend the
same quota. SQLx failures, policy disagreement, and database-clock regression fail closed as
`RateLimitError::Unavailable`. It does not replace the in-process development adapter or wire a route.

`PostgresProtectedRouteRateLimiter` 是既有 `contextlab-auth::ProtectedRouteRateLimiter` port 的独立私有 storage adapter。迁移 `0010_shared_protected_route_rate_limits.sql` 会持久化一个 deployment policy/clock guard，以及一个按精确 issuer、subject 与 protected operation 分区的有界 timestamp queue。每个 decision 都会先取得 transaction-scoped advisory lock，再回收 expiry、检查 capacity 并更新或拒绝请求 queue，因此独立 pool 无法分别花掉同一份 quota。SQLx failure、policy disagreement 与 database-clock regression 都会以 `RateLimitError::Unavailable` fail closed。它不会替换 in-process development adapter，也不会接入任何 route。

Migration `0011_shared_protected_route_rate_limit_hardening.sql` makes the policy row immutable and separates the locks: every existing key uses its own deterministic advisory lock, and only first admission uses the global lock for expiry reclamation and capacity. The configured capacity is explicitly a count of active identity-operation keys, not de-duplicated subjects. Future or unordered timestamps fail closed. A different policy remains unavailable until an operator-approved deployment procedure changes the private configuration.

迁移 `0011_shared_protected_route_rate_limit_hardening.sql` 使 policy row 不可变并分离锁：每个 existing key 使用自己的确定性 advisory lock，只有 first admission 才使用 global lock 来进行 expiry reclamation 与 capacity。配置的 capacity 明确统计 active identity-operation key，而不是去重后的 subject。future 或无序 timestamp 会 fail closed。不同 policy 在 operator 批准的 deployment procedure 改变私有 configuration 前都会保持 unavailable。

`ContextComponentRepository` provides context-scoped component inventory for `GET /api/v1/contexts/{context_id}/components`. It supports normalized `page`, `per_page`, `search`, optional exact `kind`, and `sort` inputs while keeping full component content and metadata out of the list response.

`ContextComponentRepository` 为 `GET /api/v1/contexts/{context_id}/components` 提供 context-scoped component inventory。它支持规范化后的 `page`、`per_page`、`search`、可选精确 `kind` 与 `sort` 输入，同时不在列表响应中返回完整 component content 与 metadata。

Component list items expose `id`, `context_id`, `kind`, `name`, `content_hash`, and `created_at`. `content_hash` is intentionally visible as a reproducible fingerprint for version review, diff workflows, and UI comparison. PostgreSQL component listing validates context ids as UUIDs before querying, returns unavailable scope for missing contexts, filters active rows from `context_components`, and reads context existence, count, and page items from one `REPEATABLE READ` read-only transaction.

Component list item 会暴露 `id`、`context_id`、`kind`、`name`、`content_hash` 和 `created_at`。`content_hash` 被刻意暴露为可复现 fingerprint，用于版本审查、diff workflow 与 UI 对比。PostgreSQL component listing 会先把 context id 校验为 UUID；context 不存在时返回 unavailable scope；活跃行来自 `context_components`；context existence、count 与 page item 会在同一个 `REPEATABLE READ` read-only transaction 中读取。

`get_component` provides the detail contract for `GET /api/v1/contexts/{context_id}/components/{component_id}`. Detail responses add `metadata` and `updated_at` to the list fields, matching the columns that already exist in `context_components`. They intentionally do not include body content. Private guarded storage now writes immutable commit-bound body revisions and updates the projected `content_hash` in the same transaction; those revisions are not part of this public read contract.

`get_component` 为 `GET /api/v1/contexts/{context_id}/components/{component_id}` 提供 detail contract。Detail response 会在 list 字段基础上增加 `metadata` 与 `updated_at`，与 `context_components` 中已经存在的列保持一致。该响应刻意不包含正文内容。私有 guarded storage 现会在同一事务中写入不可变、绑定 commit 的正文 revision 并更新投影 `content_hash`；这些 revision 不属于此 public read contract。

`EvaluationRunRepository` provides context-scoped benchmark and regression discovery for `GET /api/v1/contexts/{context_id}/evaluation-runs`. It supports normalized `page`, `per_page`, `search`, optional `suite_name`, optional `model_version`, and `sort` inputs while keeping full metrics JSON out of the list response.

`EvaluationRunRepository` 为 `GET /api/v1/contexts/{context_id}/evaluation-runs` 提供 context-scoped benchmark 与 regression discovery。它支持规范化后的 `page`、`per_page`、`search`、可选 `suite_name`、可选 `model_version` 与 `sort` 输入，同时不在列表响应中返回完整 metrics JSON。

Evaluation run list items expose `id`, `context_id`, `suite_name`, `model_version`, `temperature`, `metric_count`, `executed_at`, and `created_at`. PostgreSQL evaluation run listing validates context ids as UUIDs before querying, returns unavailable scope for missing contexts, computes `metric_count` from `metrics`, and reads context existence, count, and page items from one `REPEATABLE READ` read-only transaction.

Evaluation run list item 会暴露 `id`、`context_id`、`suite_name`、`model_version`、`temperature`、`metric_count`、`executed_at` 和 `created_at`。PostgreSQL evaluation run listing 会先把 context id 校验为 UUID；context 不存在时返回 unavailable scope；`metric_count` 从 `metrics` 计算；context existence、count 与 page item 会在同一个 `REPEATABLE READ` read-only transaction 中读取。

`get_evaluation_run` provides the detail contract for `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}`. Detail responses add persisted `metrics` JSON to the list fields, matching the `evaluation_runs.metrics` JSONB column. They intentionally do not derive pass/fail decisions or regression conclusions.

`get_evaluation_run` 为 `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}` 提供 detail contract。Detail response 会在 list 字段基础上增加持久化的 `metrics` JSON，与 `evaluation_runs.metrics` JSONB 列保持一致。它刻意不推导 pass/fail decision 或 regression conclusion。

`get_evaluation_scorecard` provides the aggregate contract for `GET /api/v1/contexts/{context_id}/evaluation-scorecard`. It accepts normalized `search`, optional exact `suite_name`, and optional exact `model_version`, then aggregates numeric values from persisted `metrics` JSON into sorted metric averages with sample counts. It returns the number of matching runs even when no numeric metrics are present, and it intentionally leaves pass/fail decisions and regression conclusions to higher-level evaluation workflows.

`get_evaluation_scorecard` 为 `GET /api/v1/contexts/{context_id}/evaluation-scorecard` 提供 aggregate contract。它支持规范化后的 `search`、可选精确 `suite_name` 与可选精确 `model_version`，然后把持久化 `metrics` JSON 中的 numeric value 聚合为按 metric name 排序的 average 与 sample count。即使没有 numeric metric，也会返回匹配 run 的数量；pass/fail decision 与 regression conclusion 则留给更高层的 evaluation workflow。

Preview project ids such as `support-ai` and context ids such as `support-resolution-agent` are local fixture ids. PostgreSQL mode requires UUID project ids for experiment and context discovery, and UUID context ids for commit, component, evaluation run, and evaluation scorecard discovery.

`support-ai` 这类 preview project id 和 `support-resolution-agent` 这类 preview context id 是本地 fixture id。PostgreSQL 模式的 experiment 与 context discovery 要求 project id 为 UUID，commit、component、evaluation run 与 evaluation scorecard discovery 要求 context id 为 UUID。

## Private Benchmark Evidence / 私有 Benchmark Evidence

Migration `0016_benchmark_definition_decision_evidence.sql` adds a private persistence contract for immutable benchmark datasets, suites, commit-bound evaluation runs, and decision evidence. It does not modify legacy `evaluation_runs`, which remains the existing public read-model source.

迁移 `0016_benchmark_definition_decision_evidence.sql` 增加了用于不可变 benchmark dataset、suite、绑定 commit 的 evaluation run 与 decision evidence 的私有持久化 contract。它不会修改 legacy `evaluation_runs`；后者仍是既有 public read model 的来源。

Every private run and decision is keyed by project, Context, exact Context commit, and its own identifier. Composite foreign keys make a decision membership reference only a run from that same scope and make every metric result reference the decision's own suite. Decision child rows and seals repeat that commit key, so the same decision identifier can be persisted and replayed independently for different commits without cross-commit reads or sealing conflicts.

每个私有 run 与 decision 都以 project、Context、精确 Context commit 及其自身 identifier 为键。复合外键保证 decision membership 只能引用同一作用域的 run，并保证每个 metric result 只能引用该 decision 自己的 suite。decision child row 与 seal 会重复该 commit key，因此同一 decision identifier 可以在不同 commit 中独立持久化与 replay，不会发生跨 commit read 或 sealing conflict。

Dataset, suite, run, and decision aggregates receive a deferred seal only after their required members are present. The schema rejects updates and deletes everywhere in this evidence boundary, and child-insert guards reject later cases, suite members, thresholds, measurements, decision-run membership, or metric results after sealing. The writer still constructs every aggregate in one transaction, while replay compares the canonical evidence digest before returning the original immutable record.

dataset、suite、run 与 decision aggregate 只有在必需成员齐备后才会获得延迟 seal。schema 会拒绝该 evidence boundary 中的所有 update 和 delete；seal 后的 child-insert guard 会拒绝后续 case、suite member、threshold、measurement、decision-run membership 或 metric result。writer 仍会在一个 transaction 内构造每个 aggregate；replay 会先比较 canonical evidence digest，再返回原始不可变 record。

`contextlab-evaluation` owns threshold and decision policy. Storage records `sample_count`, `required_sample_count`, and `has_complete_coverage`, then validates those facts against run membership without recalculating threshold outcomes. A duplicate metric therefore remains durable evidence but rehydrates as fail-closed `InsufficientData`.

`contextlab-evaluation` 负责 threshold 与 decision policy。storage 会记录 `sample_count`、`required_sample_count` 和 `has_complete_coverage`，并仅根据 run membership 校验这些事实，而不重新计算 threshold outcome。因此重复 metric 仍是可持久化 evidence，但会以 fail-closed 的 `InsufficientData` 被 rehydrate。

`EvaluationRun` validates a nonblank model identity and inclusive finite `0.0..=2.0` temperature range before a command can reach storage; `benchmark_evaluation_runs` enforces the same range. The PostgreSQL adapter truncates timestamps to database microsecond precision and rebuilds `BenchmarkEvaluation` from those normalized runs before it validates the command, preserving the domain artifact's exact run binding without adding a second policy calculator.

`EvaluationRun` 会在 command 到达 storage 前校验非空白 model identity 与包含边界的有限 `0.0..=2.0` temperature range；`benchmark_evaluation_runs` 会施加同一范围。PostgreSQL adapter 会把 timestamp 截断到数据库微秒精度，并在校验 command 前从规范化后的 run 重建 `BenchmarkEvaluation`，从而保持 domain artifact 对精确 run 的绑定，且不新增第二个 policy calculator。

`BenchmarkEvidenceWriter` and `BenchmarkEvidenceRepository` are storage-only ports. Both run and decision reads require project, Context, and exact commit scope, and memory/PostgreSQL keep the same identity and replay rules. The ports themselves add no public REST route, OpenAPI operation, or public SDK method. The separate local inspection slice composes one exact `get_benchmark_decision` behind authenticated `ContextPermission::Read`, a benchmark-specific protected-read limiter, and an exact project/Context/commit/decision path. `BenchmarkDecisionDefinitionSummaryService` consumes that already-loaded sealed evidence, reads only its immutable project-scoped suite and datasets, validates sealed membership, and projects stable `definition` metadata with suite identifier/name/thresholds plus dataset identifier/name/case count. Its response deliberately excludes dataset cases, inputs, expected outputs, per-run model or measurement payloads, and policy output. The local SDK and browser data client reject malformed shapes and forbidden raw keys recursively. The same-origin BFF forwards only the request-scoped Bearer token, omits cookies, and marks every private response `Cache-Control: private, no-store`.

`BenchmarkEvidenceWriter` 与 `BenchmarkEvidenceRepository` 都是 storage-only port。run read 与 decision read 均需要 project、Context 与精确 commit scope，memory 与 PostgreSQL 保持相同的 identity 与 replay 规则。这些 port 本身不会新增 public REST route、OpenAPI operation 或 public SDK method。独立的 local inspection 切片会把一条精确的 `get_benchmark_decision` 组合在 authenticated `ContextPermission::Read`、benchmark 专用 protected-read limiter 以及精确 project/Context/commit/decision path 之后。`BenchmarkDecisionDefinitionSummaryService` 会消费这条已经加载的 sealed evidence，只读取其不可变 project-scoped suite 与 dataset、校验 sealed membership，并投影稳定的 `definition` 元数据：suite identifier/name/threshold 与 dataset identifier/name/case count。其 response 刻意排除 dataset case、input、expected output、逐 run 的 model 或 measurement payload，以及 policy output。local SDK 与 browser data client 会递归拒绝不合规 shape 和禁止的 raw key；同源 BFF 只转发 request-scoped Bearer token、忽略 cookie，并为每条私有 response 标记 `Cache-Control: private, no-store`。

`get_benchmark_decision_pair` is the only storage read port for a two-decision comparison. It accepts shared project/Context scope plus two exact `(commit, decision)` identities and returns a named pair from one consistent snapshot: a single read lock in memory and one `REPEATABLE READ`, read-only transaction in PostgreSQL. The comparison service projects only status, comparability fingerprint, and stored metric threshold/coverage/outcome evidence into `contextlab-evaluation`; either missing side returns no diff, differing fingerprints fail closed, and the read path never evaluates runs again.

`get_benchmark_decision_pair` 是两条 decision 比较唯一可用的 storage read port。它接收共享的 project/Context scope 与两组精确 `(commit, decision)` identity，并从一个一致 snapshot 返回具名 pair：memory 中使用单个 read lock，PostgreSQL 中使用一个 `REPEATABLE READ`、只读 transaction。comparison service 只把 status、comparability fingerprint 与已存储的 metric threshold/coverage/outcome evidence 投影给 `contextlab-evaluation`；任一侧缺失时不产生 diff，fingerprint 不同会 fail closed，read path 永不重新评测 run。

`BenchmarkExecutionService` is a private application service layered above those ports. It accepts only typed project/Context/commit/suite/decision and model/evaluator configuration, validates that configuration before evaluator invocation, and first reads the exact decision scope for idempotent replay. A new execution loads the suite's sealed datasets, passes each ordered internal case to an injected evaluator, rejects duplicate metric kinds in its result before a run exists, and then delegates exactly once to `BenchmarkSuite::evaluate_runs` and `BenchmarkEvidenceWriter`. `BenchmarkExecutionCohort` binds each run to a composite `(dataset_id, case_id)` key and deterministically derives its UUIDv5 run identity from the decision UUID, so sealed `benchmark_decision_runs` membership preserves reproducible case-to-run provenance without a second table or a public payload. The service has no Axum, OpenAPI, SDK, Web, provider, or migration dependency.

`BenchmarkExecutionService` 是位于这些 port 之上的私有 application service。它只接受类型化的 project/Context/commit/suite/decision 以及 model/evaluator configuration，在调用 evaluator 前校验这些 configuration，并先读取精确 decision scope 以实现幂等 replay。新的 execution 会加载 suite 已 seal 的 dataset，将每条有序内部 case 传给注入 evaluator，在任何 run 出现前拒绝其 result 中的重复 metric kind，然后恰好一次委托给 `BenchmarkSuite::evaluate_runs` 与 `BenchmarkEvidenceWriter`。`BenchmarkExecutionCohort` 会把每条 run 绑定到复合 `(dataset_id, case_id)` key，并使用 decision UUID 通过 UUIDv5 确定性派生 run identity，因此已 seal 的 `benchmark_decision_runs` membership 在不增加第二张表或 public payload 的前提下保留可重现的 case-to-run provenance。该 service 不依赖 Axum、OpenAPI、SDK、Web、provider 或 migration。

The PostgreSQL tests for this contract are compiled and registered as ignored disposable-database tests. They have not run in the current non-Docker environment, so static/memory evidence must not be presented as PostgreSQL execution evidence.

该 contract 的 PostgreSQL test 已完成编译并登记为 ignored disposable-database test。它们尚未在当前非 Docker 环境执行，因此 static/memory evidence 不得被表述为 PostgreSQL execution evidence。

Runtime configuration:

运行时配置：

- `CONTEXTLAB_GRAPH_REPOSITORY=memory` keeps workspace graph reads on the in-memory projection fixture.
- `CONTEXTLAB_GRAPH_REPOSITORY=memory` 会让 workspace graph read 使用 in-memory projection fixture。
- `CONTEXTLAB_GRAPH_REPOSITORY=postgres` and `CONTEXTLAB_DATABASE_URL` route workspace graph reads through `PostgresContextGraphRepository`.
- `CONTEXTLAB_GRAPH_REPOSITORY=postgres` 与 `CONTEXTLAB_DATABASE_URL` 会让 workspace graph read 通过 `PostgresContextGraphRepository`。
- `DATABASE_URL` is accepted as a compatibility fallback, but `CONTEXTLAB_DATABASE_URL` takes precedence.
- `DATABASE_URL` 作为兼容 fallback 被支持，但 `CONTEXTLAB_DATABASE_URL` 优先级更高。
- Preview graph reads always use the deterministic in-memory fixture.
- Preview graph read 始终使用确定性的 in-memory fixture。

## Principal Identity Namespace / 主体身份命名空间

`0006_principal_identity_namespace.sql` makes protected-write identity keys issuer-scoped without rewriting or deleting historical rows. It adds `identity_source` to `workspace_memberships`, `context_commit_idempotency`, and `context_authorization_audit_events`, backfills existing records with the explicit reserved `legacy` sentinel, changes identity columns to `C` collation, then replaces affected primary keys and adds source-and-subject indexes.

`0006_principal_identity_namespace.sql` 在不重写或删除历史行的前提下，使 protected-write identity key 按 issuer 分区。它为 `workspace_memberships`、`context_commit_idempotency` 与 `context_authorization_audit_events` 增加 `identity_source`，把既有记录回填为显式且保留的 `legacy` sentinel，将 identity column 改为 `C` collation，再替换受影响的主键并增加 source-and-subject index。

The migration validates source and new subject values as non-empty, byte-bounded, control-character-free, and free from surrounding whitespace. Its subject constraints are `NOT VALID`, so malformed historic rows do not make an upgrade fail but no new malformed write can enter; the accompanying ignored disposable-PostgreSQL test exercises that upgrade path. `legacy` is an upgrade namespace, not an authentication issuer or an automatic authorization grant, and `contextlab-auth` rejects it for authenticated identities. Operators must reconcile pre-existing membership rows to a real trusted identity source before the new source isolation can be relied on for those actors. The storage adapter only resolves stored roles; `contextlab-auth` remains the owner of role-to-permission policy, and trusted external group extraction or group-to-RBAC bindings are deliberately not part of this migration.

该迁移会校验 source 与新的 subject 非空、按字节限制、无控制字符且无首尾空白。它的 subject constraint 使用 `NOT VALID`，因此不规范的历史行不会阻断升级，但新的不规范写入无法进入；配套的 ignored disposable-PostgreSQL test 覆盖这条升级路径。`legacy` 是升级命名空间，不是 authentication issuer，也不会自动授予 authorization，`contextlab-auth` 会拒绝它作为认证身份。operator 必须在对这些 actor 依赖新 source 隔离前，把既有 membership 行对齐到真实且可信的 identity source。storage adapter 只解析已存储的 role；`contextlab-auth` 继续负责 role-to-permission policy，可信外部 group extraction 或 group-to-RBAC binding 刻意不属于本迁移范围。

## Trusted External Group Bindings / 可信外部组绑定

`0007_workspace_external_group_role_bindings.sql` stores private workspace-scoped group grants. Every active binding has a `workspace_id`, exact `identity_source`, opaque `external_group_id`, and a role constrained to `reader` or `editor`; `C` collation and byte/format checks preserve the same identity semantics as migration `0006`. Its partial unique index permits historical soft-deleted rows while forbidding duplicate active bindings.

`0007_workspace_external_group_role_bindings.sql` 保存私有的 workspace-scoped group grant。每条 active binding 都有 `workspace_id`、精确 `identity_source`、不透明 `external_group_id`，以及被限制为 `reader` 或 `editor` 的 role；`C` collation 与字节/格式校验保持与迁移 `0006` 相同的身份语义。partial unique index 允许保留历史 soft-deleted 行，同时禁止重复的 active binding。

The repository never stores an IdP token or treats a provider role as a ContextLab role. It first checks direct membership; only when absent does it resolve active bindings for the principal's verified groups. `contextlab-auth` owns the resulting precedence and permission policy, while the guarded PostgreSQL writer repeats the same decision under `FOR UPDATE` locks on the Context, Project, Workspace, and matching authorization row. The ignored disposable-database test covers group-editor creation and direct-reader override; disposable CI now runs it as non-production storage evidence, while production migration validation remains a separate release gate.

repository 从不存储 IdP token，也不会把 provider role 直接当作 ContextLab role。它会先检查 direct membership；只有 direct membership 缺失时，才为 principal 的已验证 group 解析 active binding。结果的 precedence 与 permission policy 仍归 `contextlab-auth` 所有；guarded PostgreSQL writer 会在 Context、Project、Workspace 与命中的 authorization 行上使用 `FOR UPDATE` lock 重复同一决策。ignored disposable-database test 覆盖 group-editor 创建与 direct-reader 覆盖；disposable CI 现在将其作为非生产存储证据运行，生产迁移验证仍是独立 release gate。

## Authorization Audit Retention Governance / 授权审计留存治理

`0008_context_authorization_audit_governance.sql` extends the append-only audit schema without adding an audit API. It creates immutable, workspace-owned policy revisions and a separately mutable active-policy scope; it does not retroactively assign a policy to events. Existing and new audit events therefore default to `hold`, with no policy revision or purge timestamp, and cannot be deleted automatically. A future writer may create a `purge_eligible` event only with a policy revision belonging to the event Context's workspace; the insertion trigger derives its eligible timestamp from the recorded timestamp plus that immutable policy duration. The migration adds an expiry-selection index reached through the existing Context-to-Project-to-Workspace scope and append-only purge-selection manifests that record the workspace, policy revision, bounded cutoff, and selected count. `AuthorizationAuditReviewRepository` returns only timestamp, Context identifier, permission, decision, retention disposition, and policy revision after an external direct-owner boundary; its cursor keeps the audit event ID private. The migration deliberately does not provide a purge procedure, `SECURITY DEFINER` function, REST route, OpenAPI operation, SDK method, or Web control. Ordinary `UPDATE` and `DELETE` on audit events remain blocked by the existing append-only trigger; the current database role model is not treated as proof that a future privileged executor is safe.

`0008_context_authorization_audit_governance.sql` 在不增加 audit API 的前提下扩展 append-only audit schema。它创建不可变的 workspace 所有 policy revision，以及独立可替换的 active-policy scope；不会为既有 event 追溯性地分配 policy。因此既有和新追加的 audit event 都默认 `hold`，没有 policy revision 或 purge timestamp，且不会被自动删除。未来 writer 只有在具备属于 event Context workspace 的 policy revision 时，才能创建 `purge_eligible` event；insertion trigger 会用记录 timestamp 加上该不可变 policy duration 推导其 eligible timestamp。迁移增加通过既有 Context-to-Project-to-Workspace scope 使用的 expiry-selection index，以及记录 workspace、policy revision、有界 cutoff 与 selected count 的追加式 purge-selection manifest。`AuthorizationAuditReviewRepository` 会在外部 direct-owner boundary 之后仅返回 timestamp、Context identifier、permission、decision、retention disposition 与 policy revision；它的 cursor 会将 audit event ID 保持私有。迁移刻意不提供 purge procedure、`SECURITY DEFINER` function、REST route、OpenAPI operation、SDK method 或 Web control。普通的 audit event `UPDATE` 和 `DELETE` 仍由既有 append-only trigger 拒绝；当前 database role model 不会被当作未来 privileged executor 安全的证明。

## Seed Integration / Seed 集成验证

`crates/storage/fixtures/workspace_graph_seed.sql` provides deterministic PostgreSQL rows for one workspace graph. It includes active graph records and soft-deleted rows that should be excluded by repository filters.

`crates/storage/fixtures/workspace_graph_seed.sql` 提供一组确定性的 PostgreSQL workspace graph 数据。它包含活跃 graph 记录，也包含应被 repository filter 排除的 soft-deleted 行。

Normal checks do not require PostgreSQL. `scripts/verify-disposable-postgres-storage.sh` resets the disposable `public` schema before every named storage test. Local runs require an explicitly supplied loopback disposable database, its dedicated test role, an explicit reset opt-in, and `psql`; the verifier never infers a database from `.env`. The 25-case local run is non-production evidence for the current storage boundary. Remote CI and operator-approved deployment evidence are unavailable future deployment prerequisites outside the current local delivery scope. To verify the full migration, seed, read projection, and atomic commit-snapshot capture path directly, run the ignored integration test against an empty disposable database:

普通检查不需要 PostgreSQL。`scripts/verify-disposable-postgres-storage.sh` 会在每个指定存储测试前重置 disposable `public` schema。本地运行需要显式提供 loopback 的一次性数据库、其专用测试角色、显式 reset 确认以及 `psql`；验证器绝不会从 `.env` 推断数据库。25-case 本地运行是当前 storage boundary 的非生产证据。远端 CI 与 operator 批准的部署证据属于外部条件不可用、当前本地交付范围之外的未来部署前置。若要直接验证完整 migration、seed、read projection 与 atomic commit-snapshot capture 路径，可以对空的、可丢弃数据库运行 ignored integration test：

```bash
CONTEXTLAB_TEST_DATABASE_URL=postgres://contextlab_test_runner:password@localhost/contextlab_test \
  cargo test -p contextlab-storage projects_seed_workspace_graph_from_postgres -- --ignored --exact
```

## Private Component Content Creation / 私有 Component 正文创建

Migrations `0013_component_content_initial_revisions.sql` and `0014_component_content_initial_revision_integrity.sql` are forward-only changes. `0013` drops the `NOT NULL` requirement without rewriting `0012`; `0014` locks and validates existing nullable history, permits at most one `NULL` prior per component, and requires it to be the first revision recorded at the component creation timestamp. Existing revisions retain their non-null prior hash, while the first immutable revision for a newly created component stores `NULL` to mean that no predecessor body exists.

迁移 `0013_component_content_initial_revisions.sql` 与 `0014_component_content_initial_revision_integrity.sql` 都是仅向前的变更。`0013` 移除 `context_component_content_revisions.previous_content_hash` 的 `NOT NULL` 要求，不会改写 `0012`；`0014` 会锁定并校验已有 nullable history、每个 component 最多允许一条 `NULL` prior，并要求它是记录在 component creation timestamp 的第一条 revision。既有 revision 保留非空的 prior hash；新建 component 的第一条不可变 revision 使用 `NULL` 表示没有前序正文。

`ComponentContentCreationWrite` is private storage input to `GuardedContextCommitWrite`. It carries a commit-owned component UUID, kind, non-empty name, JSON metadata, opaque UTF-8 body, and capture time. The matching `AddedComponent` change persists the same name, metadata, and deterministic SHA-256 result hash, while the commit snapshot must contain `component:{id}` with the projected node kind and label plus the kind-specific relationship from `context:{context_id}`. This preserves replayable component identity and avoids inventing a second graph-diff calculation.

`ComponentContentCreationWrite` 是 `GuardedContextCommitWrite` 的私有 storage 输入。它携带由 commit 所有的 component UUID、kind、非空 name、JSON metadata、不透明 UTF-8 正文与 capture time。匹配的 `AddedComponent` change 会持久化相同的 name、metadata 与确定性 SHA-256 result hash；commit snapshot 还必须包含带投影 node kind/label 的 `component:{id}`，以及从 `context:{context_id}` 指向它的 kind-specific relationship。这样可保持 component identity 可回放，也不会新增第二个 graph-diff 计算器。

Both adapters validate before mutating their guarded transaction state. A guarded body creation, body revision, or removal requires exactly one matching private attachment after idempotency replay; a single `UpdatedComponentDescriptor` transition instead requires exactly one private `ComponentDescriptorRevisionWrite`, never a body or mixed attachment. That attachment validates the successor graph node before the same transaction updates only the current name, metadata, and timestamp while preserving the immutable body witness and content commit. Memory receipts and PostgreSQL advisory-lock/query/primary-key paths use the exact `(identity source, principal, Context, branch, key)` scope. Forward migration `0017_branch_scoped_commit_idempotency.sql` backfills `branch_name` from the immutable referenced commit, makes it non-null and validated, and extends the primary key; PostgreSQL additionally rejects a receipt whose stored branch and referenced commit branch differ. Memory writes one synchronized overlay, while PostgreSQL replays a matching idempotency receipt before its component precheck. Both persist the commit, snapshot, branch head, projection update, and receipt atomically.

两个 adapter 都会在修改 guarded transaction state 前完成校验。body creation、body revision 或 removal 在 idempotency replay 后必须恰好拥有一个匹配的私有 attachment；单个 `UpdatedComponentDescriptor` transition 则必须恰好拥有一个私有 `ComponentDescriptorRevisionWrite`，绝不能使用 body 或混合 attachment。该 attachment 会先校验 successor graph node，再在同一 transaction 中只更新当前 name、metadata 与 timestamp，同时保留不可变 body witness 与 content commit。memory receipt 与 PostgreSQL advisory-lock/query/primary-key path 使用精确的 `(identity source, principal, Context, branch, key)` 作用域。仅向前的迁移 `0017_branch_scoped_commit_idempotency.sql` 从不可变的被引用 commit 回填 `branch_name`、使其 non-null 并校验后扩展 primary key；PostgreSQL 还会拒绝 stored branch 与被引用 commit branch 不一致的 receipt。Memory 在一个同步 overlay 中写入；PostgreSQL 会在 component precheck 前先回放匹配的 idempotency receipt。两者都会原子持久化 commit、snapshot、branch head、projection update 与 receipt。Docker 关闭时 descriptor PostgreSQL test 已编译但保持 ignored/未观测。

This remains a private storage-only capability. Component body creation and body retrieval are absent from the public REST catalog, OpenAPI, TypeScript SDK, and public Web mutation controls; a separate local-only BFF/editor consumes the protected route with request-scoped credentials. Local disposable PostgreSQL evidence is non-production evidence and does not promote the protected route to public write readiness.

该能力仍仅限私有 storage。component body 创建与 body 读取均未加入 public REST catalog、OpenAPI、TypeScript SDK 或 public Web mutation control；独立的仅本地 BFF/editor 使用 request-scoped credential 消费 protected route。本地 disposable PostgreSQL 证据只是 non-production 证据，不会把 protected route 提升为 public write readiness。

## Private Unborn-Branch Context Initialization / 私有未出生分支 Context 初始化

`ContextLifecycleRootRepository` is a private one-fact port that reads only an active Context name. `ContextLifecycleOperation::Initialize` consumes that name to construct the server-owned root node and one `CreatedContext` change for `ExpectedBranchHead::Unborn`; no component, content revision, mutable projection update, or client graph is accepted. The existing `GuardedContextCommitWriter` supplies the atomic commit/snapshot/branch-head/receipt transaction and uses the established `(identity source, principal, Context, branch, key)` replay scope. Memory coverage proves parentless creation, a later same-branch head advance followed by replay of the original root, and independent creation on another branch with the same key/digest. A PostgreSQL counterpart is compiled and marked ignored for the disposable database; Docker is disabled, so its runtime result is unobserved.

`ContextLifecycleRootRepository` 是一个仅提供单一事实的私有 port，只读取 active Context name。`ContextLifecycleOperation::Initialize` 使用该 name 为 `ExpectedBranchHead::Unborn` 构造 server-owned root node 与一条 `CreatedContext` change；它不接受 component、content revision、可变 projection update 或客户端 graph。既有 `GuardedContextCommitWriter` 提供原子 commit/snapshot/branch-head/receipt transaction，并使用已建立的 `(identity source, principal, Context, branch, key)` replay 作用域。memory 覆盖证明无 parent 创建、同一 branch 在后续 head 推进后对原 root 的 replay，以及使用相同 key/digest 在另一 branch 上独立创建。PostgreSQL counterpart 已编译并标记为 disposable database ignored；Docker 关闭，因此 runtime 结果未观测。

This is a private local lifecycle extension only. It adds no public REST/OpenAPI/SDK method, public Web control, operator transport, database migration, provider call, release claim, or additional graph-diff calculator.

这仅是私有 local lifecycle 扩展。不新增 public REST/OpenAPI/SDK method、public Web control、operator transport、database migration、provider call、release 声明或额外 graph-diff calculator。

## Private Component Content Replay / 私有 Component 正文回放

`ComponentContentRevisionRepository::get_component_content_at_commit` is a private read port for replay consumers. Given one Context, target commit, and component identity, it validates the target commit's complete normal first-parent ancestry and then returns the nearest immutable revision, including the commit that actually recorded that revision. A known commit with no reachable captured body returns `None`; an unknown Context, unknown commit, merge ancestry, cycle, or malformed cross-Context parent is an explicit storage error. No body is guessed from metadata or hash projections.

`ComponentContentRevisionRepository::get_component_content_at_commit` 是供 replay consumer 使用的私有 read port。给定一个 Context、target commit 与 component identity，它会验证 target commit 的完整 normal first-parent ancestry，再返回最近的不可变 revision，同时包含真正记录该 revision 的 commit。已知 commit 没有可达 captured body 时返回 `None`；不存在的 Context、unknown commit、merge ancestry、cycle 或不合规的跨 Context parent 都会返回明确的 storage error。不会从 metadata 或 hash projection 猜测正文。

The in-memory adapter walks the stored `ContextCommitRecord` parent IDs under its existing read lock, retaining the first revision only after the whole chain is valid. PostgreSQL uses a parameterized recursive CTE over `context_commits` and `context_commit_parents`, carries an ancestry path for cycle detection, detects parent Context mismatches, and rejects any parent count greater than one. This contract does not add a public body-read route, graph editor, merge policy, API/SDK/Web method, operator transport, or another graph-diff calculator; `GraphDiff` remains the sole graph-diff calculator.

Memory adapter 会在既有 read lock 下遍历存储的 `ContextCommitRecord` parent ID，并仅在整条链有效后保留第一个 revision。PostgreSQL 使用参数化 recursive CTE 查询 `context_commits` 与 `context_commit_parents`，携带 ancestry path 检测 cycle，检测 parent Context 不匹配，并拒绝 parent count 大于一的情况。该 contract 不新增 public body-read route、graph editor、merge policy、API/SDK/Web method、operator transport 或其他 graph-diff calculator；`GraphDiff` 仍是唯一的 graph-diff calculator。

## Private Component State at Commit Replay / 私有 Component 提交状态回放

`ComponentStateAtCommitRepository::get_component_state_at_commit` reconstructs a component descriptor at a known target commit without reading the current component projection as historical truth. Both adapters validate the whole normal first-parent history before they fold root-to-target detailed `AddedComponent`, `UpdatedComponent`, and typed `RemovedComponent` payloads. Every creation or update must match the commit-local immutable revision witness, including component identity, kind, prior hash, and resulting hash. A removal must match the effective kind and hash before it makes later state absent. The state returns the replayed descriptor, metadata, effective hash, target, creation, and last content-change commit when state exists; the immutable body is used only as an internal hash witness and is not part of the returned state.

`ComponentStateAtCommitRepository::get_component_state_at_commit` 会在已知 target commit 上重建 component descriptor，而不会把当前 component projection 当作历史事实。两个 adapter 都会先验证完整的 normal first-parent history，再从 root 到 target 折叠带详情的 `AddedComponent`、`UpdatedComponent` 与 typed `RemovedComponent` payload。每次 creation 或 update 都必须与 commit-local 的不可变 revision witness 匹配，包括 component identity、kind、prior hash 与 resulting hash。removal 必须匹配有效 kind 与 hash，随后使较晚 state 缺席。state 存在时，返回值包含回放后的 descriptor、metadata、有效 hash、target、creation 与最近 content-change commit；不可变 body 仅作为内部 hash witness 使用，不属于返回 state。

 A known target without a reachable detailed creation or after a valid removal returns `None`. Invalid, stale, repeated, or post-removal transitions, missing or partial revisions, descriptor or hash discontinuity, merge ancestry, cycles, and cross-Context parents return a fail-closed component-state replay conflict. The contract adds no public REST/OpenAPI/SDK/Web surface, operator transport, or additional graph-diff calculator.

已知 target 没有可达 detailed creation 或位于合法 removal 之后时返回 `None`。无效、陈旧、重复或 removal 后的 transition、缺失或不完整的 revision、descriptor 或 hash 不连续、merge ancestry、cycle 与跨 Context parent 都会返回 fail-closed 的 component-state replay conflict。该 contract 不新增 public REST/OpenAPI/SDK/Web surface、operator transport 或额外 graph-diff calculator。

## Private Context Component-State Snapshot / 私有 Context Component-State Snapshot

`ContextComponentStateSnapshotAtCommitRepository::get_context_component_state_snapshot_at_commit` returns the complete descriptor-only component inventory at one known target commit. It validates the normal first-parent history once, parses each stored change payload once, batches immutable revision witnesses for the commits in that history, and folds detailed creation, body update, descriptor-only `UpdatedComponentDescriptor`, and typed removal transitions from root to target. The returned collection is deterministically ordered by component identifier; valid removal omits the component from later inventories, while each remaining state retains its descriptor, metadata, effective hash, creation commit, and last content-change commit. It does not inspect `context_components` to discover historical members, so it cannot fill missing history with current rows.

`ContextComponentStateSnapshotAtCommitRepository::get_context_component_state_snapshot_at_commit` 返回一个已知 target commit 上完整的、仅含 descriptor 的 component inventory。它会一次性验证 normal first-parent history、一次性解析该 history 中每个存储的 change payload、批量读取这些 commit 的不可变 revision witness，并从 root 到 target 折叠带详情的 creation、body update、descriptor-only `UpdatedComponentDescriptor` 与 typed removal transition。返回 collection 按 component identifier 确定性排序；合法 removal 会使该 component 在较晚 inventory 中缺席，而保留下来的每项仍包含 descriptor、metadata、有效 hash、creation commit 与最近 content-change commit。它不会检查 `context_components` 来发现历史成员，因此不会用当前 row 填补缺失历史。

The memory adapter reads one synchronized overlay and records removed component identifiers separately from immutable revisions. PostgreSQL reuses the validated recursive history CTE, joins immutable revision rows only for those commits, locks the active component to validate its kind and prior hash, and soft-deletes the current `context_components` projection only after the guarded commit and snapshot are inserted in the same transaction. Both reject malformed, stale, repeated, or post-removal change/revision evidence, merge/cycle/cross-Context ancestry, and unknown scope without returning a partial inventory. Historical replay never depends on the mutable projection. The contract remains private: no body exposure, REST/OpenAPI/SDK/Web surface, guarded write transport, migration, operator transport, or additional graph-diff calculator.

memory adapter 读取一个同步 overlay，并将已移除的 component identifier 与不可变 revision 分别记录。PostgreSQL 复用经过验证的 recursive history CTE，且只为这些 commit 关联不可变 revision row；它会锁定 active component 来验证 kind 与 prior hash，并仅在同一 guarded transaction 中插入 commit 与 snapshot 后，soft-delete 当前 `context_components` projection。两个 adapter 都会拒绝损坏、陈旧、重复或 removal 后的 change/revision 证据、merge/cycle/跨 Context ancestry 与 unknown scope，不返回部分 inventory。历史 replay 绝不依赖可变 projection。该 contract 仍保持私有：不暴露 body，也不新增 REST/OpenAPI/SDK/Web surface、guarded write transport、migration、operator transport 或额外 graph-diff calculator。

## Durable Commit Parent Scope / 持久化 Commit 父范围

Migration `0015_context_commit_parent_scope_integrity.sql` first preflights existing child and parent ownership before any schema mutation. It then adds `context_id` to `context_commit_parents`, backfills it from the child commit, and enforces composite `(context_id, commit_id)` and `(context_id, parent_commit_id)` references. Valid historical links upgrade in place. A legacy cross-Context link fails with SQLSTATE `23503` before the column or constraints are added; this repository does not repair external databases. The guarded writer binds the Context explicitly, parented idempotent replay creates no duplicate durable rows, child deletion cascades its links, and a referenced parent remains restricted.

迁移 `0015_context_commit_parent_scope_integrity.sql` 会在任何 schema mutation 前先检查既有 child 与 parent ownership，然后为 `context_commit_parents` 增加 `context_id`、从 child commit 回填，并强制执行复合 `(context_id, commit_id)` 与 `(context_id, parent_commit_id)` 引用。合法历史 link 会原地升级。历史跨 Context link 会在 column 或 constraint 加入前以 SQLSTATE `23503` 失败；本仓库不会修复外部数据库。guarded writer 会显式绑定 Context，带 parent 的幂等 replay 不会创建重复持久化记录，child 删除会级联其 link，而仍被引用的 parent 会保持受限。

## Private Local Lifecycle Aggregate / 私有本地生命周期聚合

`ContextLifecycleService` composes the descriptor inventory, nearest immutable body revision, and immutable graph snapshot for one materialized normal first-parent commit. It rejects a missing body, descriptor/body hash disagreement, and any missing or conflicting component graph node or relationship before it returns an aggregate. Its create, body-update, descriptor-update, and removal operations build one typed change and successor snapshot, then persist only through `GuardedContextCommitWriter` with branch-head compare-and-swap and idempotency replay. A descriptor update changes only the validated name/metadata and the corresponding graph node label.

`ContextLifecycleService` 为一个已 materialize 的 normal first-parent commit 组合 descriptor inventory、最近的不可变 body revision 与不可变 graph snapshot。它会在返回 aggregate 前拒绝缺失 body、descriptor/body hash 不一致，以及任何缺失或冲突的 component graph node 或 relationship。它的 create、body-update、descriptor-update 与 removal operation 会构造一个 typed change 和 successor snapshot，然后只通过带有 branch-head compare-and-swap 与 idempotency replay 的 `GuardedContextCommitWriter` 持久化。descriptor update 只更改经过校验的 name/metadata 与对应 graph node label。

The aggregate can be transported only by the explicitly protected local lifecycle API after authenticated `Read` authorization; it remains absent from public REST, OpenAPI, and the public SDK. That route does not alter PostgreSQL migration evidence, publication readiness, or the single-calculator rule for `GraphDiff`.

该 aggregate 只能在经认证的 `Read` 授权后由显式 protected 的 local lifecycle API 传输；它仍不进入 public REST、OpenAPI 与 public SDK。该 route 不改变 PostgreSQL migration evidence、发布就绪性，或 `GraphDiff` 的单一 calculator 规则。

## Private Typed Uses Relationship Persistence / 私有类型化 Uses 关系持久化

`AddedUsesRelationship` and `RemovedUsesRelationship` are attachment-free `ContextChange` transitions by design. They carry only distinct source and target component identifiers and never carry component descriptors, content hashes, bodies, or a caller-owned graph. At the exact expected materialized commit, `ContextLifecycleService` reconstructs both endpoint states, requires both components to remain active in the same Context, validates their graph nodes and structural Context relationships, and then adds or removes exactly one directed `GraphEdgeKind::Uses` edge. Duplicate addition and missing removal are explicit conflicts; graph reconstruction preserves every unrelated node and edge. Removing a component continues to remove all edges incident to its graph node.

`AddedUsesRelationship` 与 `RemovedUsesRelationship` 按设计是不附带 component mutation attachment 的 `ContextChange` transition。它们只携带彼此不同的 source/target component identifier，绝不携带 component descriptor、content hash、正文或调用方拥有的 graph。`ContextLifecycleService` 会在精确的 expected materialized commit 重建两个 endpoint 的 state，要求两个 component 仍是同一 Context 中的 active component，校验其 graph node 与结构性 Context relationship，再新增或移除恰好一条有向 `GraphEdgeKind::Uses` edge。重复新增和移除不存在的关系会返回明确 conflict；graph reconstruction 会保留所有无关 node 与 edge。移除 component 时仍会删除与其 graph node 相连的所有 edge。

The relationship-only command passes through the same `GuardedContextCommitWriter` without a component mutation attachment. Both adapters retain the established exact-head compare-and-swap and branch-scoped idempotency replay, and atomically persist the immutable commit, server-derived snapshot, branch head, and receipt. No migration, public transport, or alternate diff implementation is introduced; `GraphDiff::between` remains the sole graph-diff calculator. Docker-backed PostgreSQL runtime and authenticated browser mutation runtime for this Uses increment are unobserved.

仅包含 relationship 的 command 会在不附带 component mutation attachment 的情况下通过同一 `GuardedContextCommitWriter`。两个 adapter 都保留既有的 exact-head compare-and-swap 与 branch-scoped idempotency replay，并原子持久化不可变 commit、服务端派生 snapshot、branch head 与 receipt。本增量不新增 migration、public transport 或其他 diff implementation；`GraphDiff::between` 仍是唯一 graph-diff calculator。本次 Uses 增量的 Docker-backed PostgreSQL runtime 与 authenticated browser mutation runtime 均未观测。

## Private Sealed Benchmark Decision Run Details / 私有已封存 Benchmark Decision Run 明细

The admitted run-details projection is read-only and begins with the already-authorized sealed decision evidence. It resolves only the evidence's ordered `run_ids`, requires every run to match the exact project/Context/commit/run scope, preserves that stored order, and returns only the narrow run summary needed by the private inspector. Any absent or out-of-scope member fails closed; raw cases, inputs, expected outputs, model outputs, evaluator diagnostics, and other nested benchmark payloads are redacted rather than passed through. This increment adds no persistence mutation or migration.

准入的 run-details projection 是只读的，并从已经完成授权的 sealed decision evidence 开始。它只解析 evidence 中的有序 `run_ids`，要求每个 run 都匹配精确的 project/Context/commit/run scope，保留该存储顺序，并只返回私有 inspector 所需的狭窄 run summary。任何缺失或越界 member 都会 fail closed；raw case、input、expected output、model output、evaluator diagnostic 与其他 nested benchmark payload 会被 redacted，不会透传。本增量不新增 persistence mutation 或 migration。

This storage boundary is not a public REST, OpenAPI, or public SDK write contract, and it does not modify `GraphDiff::between`. Fresh focused storage coverage passed five consecutive runs after the test resolves fixture data by sealed run identifier rather than insertion order; the ignored PostgreSQL parity test compiles with `--no-run`. Docker/PostgreSQL runtime remains unobserved, including ignored adapter runtime; authenticated browser E2E also remains unobserved. The workspace, SDK, and Web verification receipts are recorded in the active-goal and completion-criteria documents.

该 storage boundary 不是 public REST、OpenAPI 或 public SDK write contract，也不修改 `GraphDiff::between`。在测试按 sealed run identifier 而不是插入顺序解析 fixture 后，新鲜的聚焦 storage 覆盖连续五次通过；ignored PostgreSQL parity 测试已使用 `--no-run` 完成编译。Docker/PostgreSQL runtime（包括 ignored adapter runtime）仍未观测；authenticated browser E2E 也仍未观测。工作区、SDK 与 Web 的验证回执记录在 active-goal 与 completion-criteria 文档中。

## Private Server-Owned Commit Ancestry Review / 私有服务端拥有的 Commit Ancestry Review

`ContextCommitGraphRepository` loads the complete commit DAG for one typed Context identity. It
converts persisted UUID and parent rows into the reusable `contextlab-versioning::CommitGraph`
and rejects invalid, missing, duplicate, cross-Context, disconnected, or cyclic history before a
consumer can resolve ancestry. Memory and PostgreSQL adapters share this validation boundary;
`WorkspaceDataRepository` and `AppState` expose it internally without adding transport.

`ContextCommitGraphRepository` 为一个 typed Context identity 加载完整 commit DAG。它把持久化 UUID 与 parent row 转换为可复用的
`contextlab-versioning::CommitGraph`，并在 consumer 解析 ancestry 前拒绝 invalid、missing、duplicate、cross-Context、disconnected 或
cyclic history。Memory 与 PostgreSQL adapter 共享该 validation boundary；`WorkspaceDataRepository` 与 `AppState` 仅在内部暴露它，
不新增 transport。

`PersistedContextGraphMergeReviewService::review_server_owned` accepts only a typed project,
Context, and two branch tips. It resolves `MergePlan` from that exact DAG, rejects unknown tips
and non-three-way outcomes, then reuses the existing exact snapshot batch and
`VersionedContextGraphMergeReviewService`. The legacy caller-supplied method remains for existing
private consumers until a separate transport decision; no route, SDK, Web, mutation, or second
`GraphDiff` calculator is introduced. Runtime and release evidence remain classified separately.

`PersistedContextGraphMergeReviewService::review_server_owned` 只接受 typed project、Context 与两个 branch tip。它从 exact DAG 解析
`MergePlan`，拒绝 unknown tip 与 non-three-way outcome，然后复用既有 exact snapshot batch 和 `VersionedContextGraphMergeReviewService`。
既有 caller-supplied method 在单独 transport 决策前保持给现有 private consumer 使用；不新增 route、SDK、Web、mutation 或第二个
`GraphDiff` calculator。runtime 与 release evidence 继续单独分类。
