# Context Evaluation Run Detail API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}` so clients can inspect one benchmark or regression run's persisted metrics JSON before dashboards and evaluation diff workflows exist.

**Architecture:** `contextlab-storage` owns the detail DTO and repository method, backed by deterministic in-memory projection records and SQLx/PostgreSQL reads from `evaluation_runs.metrics`. `server/api` remains a thin Axum boundary through the public GET route catalog, while the TypeScript SDK, OpenAPI contract, and bilingual docs expose the same REST contract.

**Tech Stack:** Rust, Axum, SQLx, PostgreSQL JSONB, Serde, TypeScript, OpenAPI, pnpm, bilingual Markdown docs.

---

### Task 1: Extend Storage Evaluation Run Contracts

**Files:**
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/evaluation_run.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/projection.rs`

- [x] **Step 1: Store metrics in projection records**

Extend `EvaluationRunRecord` with `metrics: serde_json::Value`. The deterministic preview run should use metrics matching the current `metric_count` contract, for example `accuracy` and `latency_ms`.

- [x] **Step 2: Add evaluation run detail DTO**

Add `EvaluationRunDetail` with `id`, `context_id`, `suite_name`, `model_version`, `temperature`, `metric_count`, `metrics`, `executed_at`, and `created_at`. Do not add derived scorecard judgments or regression conclusions.

- [x] **Step 3: Extend repository trait**

Add `get_evaluation_run(&self, context_id: String, run_id: String) -> Result<EvaluationRunDetail, StorageRepositoryError>` to `EvaluationRunRepository` and re-export `EvaluationRunDetail`.

### Task 2: Implement In-Memory and PostgreSQL Reads

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement in-memory detail lookup**

Verify the context exists, find the exact run id within that context, return `EvaluationRunDetail`, and use `StorageRepositoryError::ScopeUnavailable { scope: "evaluation_run:{context_id}/{run_id}" }` when missing.

- [x] **Step 2: Preserve list/detail metrics behavior**

Keep list responses lightweight with `metric_count`, but compute/return detail `metrics` from the record payload.

- [x] **Step 3: Implement PostgreSQL detail lookup**

Parse `context_id` and `run_id` as UUIDs before connecting. Verify the context exists, read one active `evaluation_runs` row by `(context_id, id)`, select `metrics` and `jsonb_object_length(metrics)` as `metric_count`, and return unavailable scope for a missing run.

- [x] **Step 4: Add focused storage tests**

Cover in-memory detail success, missing run, missing context, and PostgreSQL invalid run UUID behavior.

### Task 3: Add API Route and Route Guard Coverage

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`

- [x] **Step 1: Add route handler**

Expose `context_evaluation_run` using `Path<(String, String)>` and returning `Json<EvaluationRunDetail>`.

- [x] **Step 2: Add public GET route catalog entry**

Add `PublicGetRoute::ContextEvaluationRun` with path `/api/v1/contexts/{context_id}/evaluation-runs/{run_id}`, operationId `getEvaluationRun`, path parameters `context_id` and `run_id`, and no query parameters.

- [x] **Step 3: Add API tests**

Cover default preview detail success with `metrics.accuracy`, unknown run, and invalid PostgreSQL run UUID.

### Task 4: Update SDK and OpenAPI Contract

**Files:**
- Modify: `packages/ts-sdk/src/types.ts`
- Modify: `packages/ts-sdk/src/client.ts`
- Modify: `packages/ts-sdk/src/client.test.ts`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`
- Modify: `packages/ts-sdk/src/index.ts`
- Modify: `docs/api/openapi.json`

- [x] **Step 1: Add SDK detail type and method**

Add `EvaluationRunDetail = EvaluationRunItem & { metrics: Record<string, unknown> }` and `getEvaluationRun(contextId, runId)`.

- [x] **Step 2: Extend SDK route tests**

Assert `getEvaluationRun("context/id", "run/id")` builds `/api/v1/contexts/context%2Fid/evaluation-runs/run%2Fid`.

- [x] **Step 3: Extend OpenAPI**

Add the detail path, `RunId` parameter, `EvaluationRunDetail` schema, and matching contract test expectation.

### Task 5: Update Bilingual Documentation and Verify

**Files:**
- Modify: `README.md`
- Modify: `docs/api/rest-api.md`
- Modify: `docs/sdk/typescript-sdk.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Document the detail route**

Update bilingual docs to state that evaluation run detail returns persisted metrics JSON while list responses remain lightweight.

- [x] **Step 2: Run focused checks**

Run:

```powershell
cargo test -p contextlab-storage evaluation_run
cargo test -p contextlab-api evaluation_run
pnpm --filter @contextlab/ts-sdk lint
pnpm --filter @contextlab/ts-sdk test
```

Expected: PASS.

- [x] **Step 3: Run full checks**

Run:

```powershell
pnpm check
pnpm test
```

Expected: PASS, with the existing PostgreSQL seed integration test still ignored unless a disposable database is supplied.

### Self-Review

- Spec coverage: This plan advances benchmark-driven development by exposing the persisted metrics payload needed for future scorecards, dashboards, regression detection, and evaluation diff workflows.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: `EvaluationRunDetail`, `get_evaluation_run`, `context_evaluation_run`, and `getEvaluationRun` describe one context-scoped evaluation run detail read.
