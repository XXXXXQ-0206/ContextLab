# Context Evaluation Scorecard API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose the existing context-scoped evaluation scorecard aggregation through the REST API, OpenAPI contract, and TypeScript SDK.

**Architecture:** The storage crate already owns scorecard domain/query semantics. The API layer should only deserialize query parameters, delegate to `EvaluationRunRepository`, and keep the public route catalog aligned with the checked-in OpenAPI document. The TypeScript SDK should mirror the REST contract without adding frontend business logic.

**Tech Stack:** Rust, Axum, Serde, SQLx-backed storage contracts, OpenAPI 3.x, TypeScript, node:test.

---

### Task 1: Contract Tests

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `packages/ts-sdk/src/client.test.ts`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`

- [ ] **Step 1: Add a Rust route test**

Add a test beside the existing evaluation run route tests:

```rust
#[tokio::test]
async fn context_evaluation_scorecard_route_returns_metric_averages() {
    let response = test_router()
        .oneshot(
            Request::builder()
                .uri("/api/v1/contexts/support-resolution-agent/evaluation-scorecard")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let payload: Value = serde_json::from_slice(&body).expect("json payload");

    assert_eq!(payload["context_id"], "support-resolution-agent");
    assert_eq!(payload["run_count"], 1);
    assert_eq!(payload["metrics"][0]["name"], "accuracy");
    assert_eq!(payload["metrics"][0]["average"], 0.92);
    assert_eq!(payload["metrics"][0]["sample_count"], 1);
    assert_eq!(payload["metrics"][1]["name"], "latency_ms");
    assert_eq!(payload["metrics"][1]["average"], 820.0);
    assert_eq!(payload["metrics"][1]["sample_count"], 1);
}
```

- [ ] **Step 2: Add filtered Rust route coverage**

Add a test proving `suite_name` and `model_version` filters are delegated:

```rust
#[tokio::test]
async fn context_evaluation_scorecard_route_filters_by_suite_and_model() {
    let response = test_router()
        .oneshot(
            Request::builder()
                .uri("/api/v1/contexts/support-resolution-agent/evaluation-scorecard?suite_name=Safety%20Regression%20Suite&model_version=other-model")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);

    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let payload: Value = serde_json::from_slice(&body).expect("json payload");

    assert_eq!(payload["context_id"], "support-resolution-agent");
    assert_eq!(payload["run_count"], 0);
    assert!(payload["metrics"].as_array().expect("metrics").is_empty());
}
```

- [ ] **Step 3: Add SDK route-building coverage**

Extend `packages/ts-sdk/src/client.test.ts` so the client must build:

```text
http://127.0.0.1:3100/api/v1/contexts/context%2Fid/evaluation-scorecard?search=safety&suite_name=Safety+Regression+Suite&model_version=deepseek-chat
```

- [ ] **Step 4: Add SDK/OpenAPI contract expectation**

Extend `getOperations` in `packages/ts-sdk/src/openapi-contract.test.ts` with:

```ts
{
  methodName: "getEvaluationScorecard",
  path: "/api/v1/contexts/{context_id}/evaluation-scorecard",
  pathParameters: ["context_id"],
  queryParameters: ["search", "suite_name", "model_version"]
}
```

- [ ] **Step 5: Verify red**

Run:

```powershell
cargo test -p contextlab-api context_evaluation_scorecard_route_returns_metric_averages -- --exact
pnpm --filter @contextlab/ts-sdk test
```

Expected: Rust cannot pass until the API adapter/route exists; TypeScript cannot pass until the SDK method and OpenAPI path exist.

### Task 2: API Implementation

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`

- [ ] **Step 1: Add route query params**

Add `EvaluationScorecardParams` with `search`, `suite_name`, and `model_version`, plus a `From<EvaluationScorecardParams> for EvaluationScorecardQuery` conversion.

- [ ] **Step 2: Add handler**

Add `context_evaluation_scorecard` that delegates to `state.evaluation_run_repository().get_evaluation_scorecard(context_id, query).await?`.

- [ ] **Step 3: Add route catalog metadata**

Add `ContextEvaluationScorecard` to `PublicGetRoute`, path `/api/v1/contexts/{context_id}/evaluation-scorecard`, operation id `getEvaluationScorecard`, context path parameters, and scorecard query parameters.

- [ ] **Step 4: Implement repository adapter method**

Add `get_evaluation_scorecard` to `impl EvaluationRunRepository for WorkspaceDataRepository`, delegating to Memory or Postgres repositories.

### Task 3: OpenAPI and SDK

**Files:**
- Modify: `docs/api/openapi.json`
- Modify: `packages/ts-sdk/src/types.ts`
- Modify: `packages/ts-sdk/src/client.ts`

- [ ] **Step 1: Add OpenAPI path**

Add `GET /api/v1/contexts/{context_id}/evaluation-scorecard` with operation id `getEvaluationScorecard`, `ContextId`, `Search`, `suite_name`, and `model_version` parameters, and a `200` response referencing `EvaluationScorecard`.

- [ ] **Step 2: Add OpenAPI schemas**

Add `EvaluationScorecardMetric` and `EvaluationScorecard` schemas with required fields `name`, `average`, `sample_count`, `context_id`, `run_count`, and `metrics`.

- [ ] **Step 3: Add SDK types**

Add `EvaluationScorecardMetric`, `EvaluationScorecard`, and `EvaluationScorecardQuery`.

- [ ] **Step 4: Add SDK method**

Add `getEvaluationScorecard(contextId, query = {})` to `ContextLabClient`.

### Task 4: Docs and Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/api/rest-api.md`
- Modify: `docs/sdk/typescript-sdk.md`
- Modify: `docs/storage/persistence-foundation.md`

- [ ] **Step 1: Update bilingual docs**

Document the scorecard route in English and Chinese, and replace future-only scorecard wording with the new aggregate endpoint boundary.

- [ ] **Step 2: Format and test**

Run:

```powershell
cargo fmt
cargo test -p contextlab-api
pnpm --filter @contextlab/ts-sdk test
pnpm --filter @contextlab/ts-sdk lint
```

Expected: all commands exit 0.
