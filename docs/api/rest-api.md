# REST API Contract / REST API 合约

ContextLab is REST-first. The current checked-in contract lives at `docs/api/openapi.json`, is served at `GET /api/v1/openapi.json`, and describes the public GET and bounded POST surface that exists in the Axum API today.

ContextLab 采用 REST-first。当前已纳入仓库的合约位于 `docs/api/openapi.json`，运行时通过 `GET /api/v1/openapi.json` 提供，描述的是 Axum API 当前已经存在的公开 GET 与受限 POST surface。

## Contract Scope / 合约范围

The first OpenAPI contract covers:

第一版 OpenAPI contract 覆盖：

- `GET /healthz`
- `GET /api/v1/meta`
- `GET /api/v1/openapi.json`
- `GET /api/v1/providers`
- `GET /api/v1/context-graph/preview`
- `GET /api/v1/workspaces/{workspace_id}/context-graph`
- `POST /api/v1/graph-diffs`
- `GET /api/v1/workspaces`
- `GET /api/v1/workspaces/{workspace_id}/projects`
- `GET /api/v1/projects/{project_id}/experiments`
- `GET /api/v1/projects/{project_id}/contexts`
- `GET /api/v1/contexts/{context_id}/commits`
- `GET /api/v1/contexts/{context_id}/components`
- `GET /api/v1/contexts/{context_id}/components/{component_id}`
- `GET /api/v1/contexts/{context_id}/evaluation-runs`
- `GET /api/v1/contexts/{context_id}/evaluation-scorecard`
- `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}`

Discovery routes share `{ items, pagination }` responses and support consistent `page`, `per_page`, `search`, and `sort` semantics. Route-specific filters remain explicit: `experiment_id`, `branch_name`, `kind`, `suite_name`, and `model_version`. Component detail returns one `ComponentDetail` object with metadata and timestamps, but not body content: body revisions are persisted only through the private guarded commit-storage contract and are not part of the public REST surface. Evaluation run detail returns one `EvaluationRunDetail` object with persisted metrics JSON while list responses stay lightweight. Evaluation scorecard returns one aggregate `{ context_id, run_count, metrics }` object and accepts `search`, `suite_name`, and `model_version` filters without pagination.

Discovery route 共享 `{ items, pagination }` 响应，并保持一致的 `page`、`per_page`、`search` 与 `sort` 语义。特定 route 的过滤参数保持显式：`experiment_id`、`branch_name`、`kind`、`suite_name` 与 `model_version`。Component detail 会返回一个包含 metadata 与 timestamp 的 `ComponentDetail` 对象，但不返回正文内容：正文 revision 只通过私有 guarded commit-storage contract 持久化，不属于 public REST surface。Evaluation run detail 会返回一个包含持久化 metrics JSON 的 `EvaluationRunDetail` 对象，同时 list response 保持轻量。Evaluation scorecard 返回单个聚合 `{ context_id, run_count, metrics }` 对象，支持 `search`、`suite_name` 与 `model_version` 过滤，但不分页。

Private component-content creation is also absent from this contract. It is a storage-only guarded command that captures a replayable descriptor and initial immutable revision with a commit; it adds no body-create endpoint, body-read endpoint, protected-write promotion, or operator transport. The checked-in OpenAPI document, route catalog, and public SDK therefore remain read-only for component bodies.

私有 component-content creation 同样不属于本合约。它是一个 storage-only guarded command，用于随 commit 捕获可回放 descriptor 与初始不可变 revision；它不增加 body-create endpoint、body-read endpoint、protected-write promotion 或 operator transport。因此 checked-in OpenAPI document、route catalog 与 public SDK 对 component body 仍保持只读。

`POST /api/v1/graph-diffs` compares caller-supplied `original` and `revised` graph snapshots and returns deterministic node and edge changes. It validates the supplied graph but does not persist snapshots, compare commits, or edit a graph.

`POST /api/v1/graph-diffs` 比较调用方提供的 `original` 与 `revised` graph snapshot，并返回确定性的节点和边变化。它会验证提供的图，但不会持久化 snapshot、比较 commit 或编辑图。

The protected local route `GET /api/v1/local/contexts/{context_id}/graph-diff` compares two already-materialized snapshots from commits in the same Context. It requires `original_commit_id` and `revised_commit_id`, authenticates before authorization, audit, rate-limit, or snapshot access, and returns private no-store responses. The server resolves and rechecks exact `(Context, commit)` scope before the existing versioned Rust review delegates to `GraphDiff::between`; the response includes server-owned capture references and pair identity alongside the deterministic diff. This private local route is not part of the public OpenAPI or public SDK contract, is read-only, and does not create commits, mutate snapshots, or expose graph editing.

受保护的 private local route `GET /api/v1/local/contexts/{context_id}/graph-diff` 会比较同一 Context 内两个已经物化的 commit snapshot。它要求 `original_commit_id` 与 `revised_commit_id`，会在 authorization、audit、rate-limit 或 snapshot access 之前完成 authentication，并对成功与失败响应使用 private no-store。server 会解析并再次校验精确的 `(Context, commit)` scope，随后由既有 versioned Rust review 委托唯一的 `GraphDiff::between`；response 会在确定性 diff 旁返回 server-owned capture reference 与 pair identity。该 private local route 不属于 public OpenAPI 或 public SDK contract，只读，不创建 commit、不修改 snapshot，也不暴露 graph editing。

## Identifier Semantics / 标识符语义

Path identifiers are documented as strings. The in-memory preview repository uses readable ids such as `default` and `support-ai`; PostgreSQL mode validates production scopes as UUIDs before querying.

Path identifier 在 OpenAPI 中记录为 string。In-memory preview repository 使用 `default`、`support-ai` 这类可读 id；PostgreSQL 模式会在查询前把生产 scope 校验为 UUID。

## SDK Alignment / SDK 对齐

`packages/ts-sdk/src/openapi-contract.test.ts` parses `docs/api/openapi.json` and verifies that every public GET method plus `compareGraphs` in `ContextLabClient` has a matching OpenAPI operation and contract shape.

`packages/ts-sdk/src/openapi-contract.test.ts` 会解析 `docs/api/openapi.json`，验证 `ContextLabClient` 中每个 public GET method 以及 `compareGraphs` 都有对应的 OpenAPI operation 与 contract shape。

## API Route Guard / API 路由护栏

`server/api/src/lib.rs` owns API-side public GET and POST route catalogs. Axum route installation goes through those catalogs, and `cargo test -p contextlab-api` verifies that every catalog entry has a matching checked-in OpenAPI path, `operationId`, path parameter set, and query parameter set.

`server/api/src/lib.rs` 负责 API 侧 public GET 与 POST route catalog。Axum route installation 通过这些目录表完成，`cargo test -p contextlab-api` 会验证每个目录项都有对应的 checked-in OpenAPI path、`operationId`、path parameter set 与 query parameter set。

When adding a public route, update the matching route catalog, handler, checked-in OpenAPI document, REST docs, and SDK client surface in the same change.

新增 public route 时，需要在同一变更中更新对应 route catalog、handler、checked-in OpenAPI document、REST docs 和 SDK client surface。

## Verification / 验证

```bash
cargo test -p contextlab-api
pnpm --filter @contextlab/ts-sdk test
pnpm check
```

`cargo test -p contextlab-api` is the fastest API-side route/OpenAPI drift check. `pnpm --filter @contextlab/ts-sdk test` is the fastest SDK-side contract check. `pnpm check` runs the SDK contract tests plus Rust and Web checks.

`cargo test -p contextlab-api` 是最快的 API 侧 route/OpenAPI drift 检查。`pnpm --filter @contextlab/ts-sdk test` 是最快的 SDK 侧合约检查。`pnpm check` 会同时运行 SDK contract tests、Rust checks 与 Web checks。

## Wave 1 Integration Boundary / Wave 1 集成边界

Wave 1 domain owners do not add routes directly. Their typed records, ordering rules, error states, and deterministic fixtures are admitted by the Integration Lead through the contract described in `docs/api/wave-1-integration-contracts.md`. Root workspace membership, API/SDK transport composition, public route catalogs, and OpenAPI registration remain Integration Lead-owned.

Wave 1 领域 owner 不直接新增 route。其类型化 record、ordering rule、error state 与确定性 fixture，必须通过 `docs/api/wave-1-integration-contracts.md` 所描述的契约由 Integration Lead 准入。根 workspace membership、API/SDK transport composition、public route catalog 与 OpenAPI registration 仍由 Integration Lead 负责。

The current public surface remains unchanged. A local-only Wave 1 consumer may use an exact private scope, an authenticated same-origin BFF, and a redacted typed projection; if its dependency is not registered, it reports typed `unavailable` rather than editing a root manifest. This documentation does not admit public benchmark execution, plugin loading, provider transport, or write promotion.

当前 public surface 保持不变。local-only Wave 1 consumer 可以使用精确的 private scope、authenticated same-origin BFF 与脱敏的 typed projection；如果其 dependency 尚未 registration，consumer 应报告类型化 `unavailable`，而不是编辑根 manifest。本文档不准入 public benchmark execution、plugin loading、provider transport 或 write promotion。

For the H-owned evidence vocabulary and current passed/ignored/unobserved/blocked boundaries, see `docs/verification/wave-1-contract-checklist.md`. Existing local test receipts remain scoped to the commands recorded in the roadmap; they do not prove Docker runtime, authenticated browser E2E, remote CI, operator rehearsal, release, or production behavior.

关于 H 负责的 evidence vocabulary 与当前 passed/ignored/unobserved/blocked 边界，请参阅 `docs/verification/wave-1-contract-checklist.md`。现有本地 test receipt 仍只属于 roadmap 记录的 command；它们不能证明 Docker runtime、authenticated browser E2E、remote CI、operator rehearsal、release 或 production behavior。

## Future Direction / 后续方向

This OpenAPI document is intentionally checked in before introducing generated SDKs. Once the API surface is broader and more stable, the project can generate SDKs and docs from the same contract while keeping the current tests as compatibility guards.

当前 OpenAPI document 会先作为 checked-in contract 存在，再引入 generated SDK。等 API surface 更完整、更稳定后，项目可以从同一份 contract 生成 SDK 与文档，并保留当前测试作为兼容性保护。
