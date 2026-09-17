# TypeScript SDK / TypeScript SDK

`@contextlab/ts-sdk` is the first client-side contract package for ContextLab REST discovery APIs and bounded pure computations. It keeps API DTOs and route construction outside the Web UI so product surfaces can move from preview data to live data without duplicating backend contracts.

`@contextlab/ts-sdk` 是 ContextLab REST discovery API 与受限纯计算的第一版客户端合约包。它把 API DTO 与 route construction 从 Web UI 中抽离出来，使产品界面可以从 preview data 平滑迁移到 live data，而不重复维护后端合约。

## Scope / 范围

The SDK now covers the current public GET surface plus one explicit GraphDiff POST operation:

SDK 当前覆盖公开 GET surface，以及一个显式 GraphDiff POST operation：

- `GET /healthz`
- `GET /api/v1/meta`
- `GET /api/v1/openapi.json`
- `GET /api/v1/providers`
- `GET /api/v1/context-graph/preview`
- `GET /api/v1/workspaces/{workspace_id}/context-graph`
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
- `POST /api/v1/graph-diffs` via `compareGraphs`
- `GET /api/v1/contexts/{context_id}/graph-diff` via `getCommitGraphDiff`

It provides shared DTOs, query option types, platform/provider/graph response types, a fetch-based `ContextLabClient`, and structured `ContextLabApiError` handling.

它提供共享 DTO、query option type、platform/provider/graph response type、基于 `fetch` 的 `ContextLabClient`，以及结构化的 `ContextLabApiError` 处理。

## Usage / 使用方式

```ts
import { ContextLabClient } from "@contextlab/ts-sdk";

const client = new ContextLabClient({
  baseUrl: "http://127.0.0.1:3100"
});

const workspaces = await client.listWorkspaces({
  page: 1,
  per_page: 20,
  sort: "created_at"
});

const meta = await client.getMeta();
const graph = await client.getWorkspaceContextGraph("default");
const components = await client.listComponents("support-resolution-agent", {
  kind: "knowledge",
  sort: "kind"
});
const refundPolicy = await client.getComponent("support-resolution-agent", "refund-policy");
const safetyRun = await client.getEvaluationRun(
  "support-resolution-agent",
  "safety-regression"
);
const scorecard = await client.getEvaluationScorecard("support-resolution-agent", {
  suite_name: "Safety Regression Suite",
  model_version: "deepseek-chat"
});
const graphDiff = await client.compareGraphs({
  original: { nodes: [], edges: [] },
  revised: { nodes: [], edges: [] }
});
const commitGraphDiff = await client.getCommitGraphDiff(
  "support-resolution-agent",
  "support-resolution-agent-initial",
  "support-resolution-agent-revised"
);
```

The client removes trailing slashes from `baseUrl`, encodes path segments and query parameters, and throws `ContextLabApiError` when the API returns a non-2xx response with `{ error, message }`.

Client 会移除 `baseUrl` 末尾斜杠，编码 path segment 与 query parameter，并在 API 返回带 `{ error, message }` 的非 2xx 响应时抛出 `ContextLabApiError`。

`getComponent` returns metadata, timestamps, and the reproducible `content_hash`. It does not return component body content. Private guarded storage persists commit-bound body revisions, but body retrieval remains outside the public SDK contract.

`getComponent` 会返回 metadata、timestamp 与可复现的 `content_hash`。它不会返回 component body content。私有 guarded storage 会持久化绑定 commit 的正文 revision，但正文读取仍不属于 public SDK contract。

The SDK intentionally has no component-body creation, component-body retrieval, or protected commit-write method. Private `ComponentContentCreationWrite` stays behind the Rust storage boundary until separately admitted public-write and release evidence exists; it does not change OpenAPI, SDK, or Web mutation behavior.

SDK 刻意不提供 component-body creation、component-body retrieval 或 protected commit-write method。私有 `ComponentContentCreationWrite` 会留在 Rust storage 边界之后，直到单独具备 public-write 与 release 证据；它不改变 OpenAPI、SDK 或 Web mutation 行为。

`getEvaluationRun` returns persisted `metrics` JSON for one run alongside the list fields.

`getEvaluationRun` 会返回单次 run 的持久化 `metrics` JSON，并包含 list 字段。

`getEvaluationScorecard` returns numeric metric averages across matching runs as `{ context_id, run_count, metrics }`. It accepts `search`, `suite_name`, and `model_version`; it does not compute pass/fail decisions or regression conclusions.

`getEvaluationScorecard` 会以 `{ context_id, run_count, metrics }` 形式返回匹配 run 的 numeric metric average。它支持 `search`、`suite_name` 与 `model_version`；它不计算 pass/fail decision 或 regression conclusion。

`compareGraphs` sends explicit node-array graph snapshots and returns deterministic structural changes. It does not persist either snapshot, compare version commits, or provide graph editing.

`compareGraphs` 发送显式 node-array graph snapshot，并返回确定性的结构变化。它不会持久化任一 snapshot、比较 version commit 或提供 graph editing。

`getCommitGraphDiff` sends no graph payload. It reads two materialized snapshots from commits in one Context and returns `{ context_id, original, revised, diff }`. It raises `ContextLabApiError` with `409 commit_graph_snapshot_missing` when either known commit has not yet captured a graph snapshot; it does not create commits or provide graph review/editing.

`getCommitGraphDiff` 不发送 graph payload。它读取同一 Context 内两个 commit 的 materialized snapshot，并返回 `{ context_id, original, revised, diff }`。任一已知 commit 尚未捕获 graph snapshot 时，它会以 `409 commit_graph_snapshot_missing` 抛出 `ContextLabApiError`；该方法不创建 commit，也不提供 graph review/editing。

## Web Runtime Boundary / Web 运行时边界

`apps/web/src/app/context-workspace-data.ts` is the only Web live-data boundary for the current workspace shell.

`apps/web/src/app/context-workspace-data.ts` 是当前 Web workspace shell 唯一的 live-data 边界。

- Without `CONTEXTLAB_WEB_API_BASE_URL`, the Web app renders deterministic preview data.
- 未设置 `CONTEXTLAB_WEB_API_BASE_URL` 时，Web app 渲染确定性的 preview data。
- With `CONTEXTLAB_WEB_API_BASE_URL`, the Web app requests live discovery lists, workspace Context Graph data, selected component detail, selected evaluation detail, and a scorecard scoped to the selected run's suite/model filters at runtime.
- 设置 `CONTEXTLAB_WEB_API_BASE_URL` 后，Web app 会在运行时请求 live discovery list、workspace Context Graph data、selected component detail、selected evaluation detail，以及按选中 run 的 suite/model filter 收敛的 scorecard。
- If live loading fails, the Web app falls back to preview data and labels the source as `preview-fallback`.
- 如果 live loading 失败，Web app 会回退到 preview data，并将来源标记为 `preview-fallback`。

The Web page is intentionally dynamic server-rendered so live API data is not accidentally baked into static build output.

Web 页面刻意采用 dynamic server rendering，避免 live API data 被意外写入静态构建产物。

## Verification / 验证

```bash
pnpm --filter @contextlab/ts-sdk lint
pnpm --filter @contextlab/ts-sdk test
pnpm --filter @contextlab/web build
```

To prove the Web build does not require a live API:

验证 Web build 不依赖 live API：

```powershell
$env:CONTEXTLAB_WEB_API_BASE_URL='http://127.0.0.1:9'
pnpm --filter @contextlab/web build
Remove-Item Env:\CONTEXTLAB_WEB_API_BASE_URL
```

## Future Direction / 后续方向

This package is a deliberate hand-written contract layer for the earliest platform phase. Later, OpenAPI generation can replace or augment it, but the current SDK tests should remain as compatibility checks for route construction and error semantics.

这个 package 是早期平台阶段刻意保留的手写合约层。后续可以用 OpenAPI generation 替换或增强它，但当前 SDK tests 应继续作为 route construction 与 error semantics 的兼容性检查。

The checked-in OpenAPI contract lives at `docs/api/openapi.json` and is served by the API at `GET /api/v1/openapi.json`. SDK tests parse it and verify that every public GET client method and `compareGraphs` has a matching operation contract.

已纳入仓库的 OpenAPI contract 位于 `docs/api/openapi.json`，并由 API 通过 `GET /api/v1/openapi.json` 提供。SDK tests 会解析它，并验证每个 public GET client method 与 `compareGraphs` 都有对应的 operation contract。
