# OpenAPI Runtime SDK Platform Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Serve the checked-in OpenAPI contract from the Axum API and expand `@contextlab/ts-sdk` beyond discovery lists to cover the current public GET surface.

**Architecture:** The checked-in `docs/api/openapi.json` remains the source contract artifact. The API exposes the same document at runtime through a read-only route, while the SDK adds platform, provider, graph, and OpenAPI read methods that are verified against the contract by operation ID and route metadata.

**Tech Stack:** Rust, Axum, Serde JSON, TypeScript, OpenAPI 3.1, Node test runner via `tsx`.

---

## Task 1: Runtime OpenAPI Route

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`
- Modify: `docs/api/openapi.json`

- [x] **Step 1: Add an OpenAPI route handler**

Add `openapi()` in `routes.rs` that parses `include_str!("../../../docs/api/openapi.json")` into `serde_json::Value` and returns `Json<Value>`.

- [x] **Step 2: Register the route**

Add `GET /api/v1/openapi.json` to `build_router_with_state`.

- [x] **Step 3: Add a Rust route test**

Add a test asserting `/api/v1/openapi.json` returns 200, `openapi` starts with `3.`, and includes `paths["/api/v1/workspaces"]`.

- [x] **Step 4: Add the route to OpenAPI**

Add `GET /api/v1/openapi.json` with `operationId: getOpenApiDocument`.

## Task 2: SDK Platform Coverage

**Files:**
- Modify: `packages/ts-sdk/src/types.ts`
- Modify: `packages/ts-sdk/src/client.ts`
- Modify: `packages/ts-sdk/src/index.ts`
- Modify: `packages/ts-sdk/src/client.test.ts`

- [x] **Step 1: Add platform and graph DTOs**

Add `HealthResponse`, `MetaResponse`, `ProviderStatus`, `ProvidersResponse`, `GraphNode`, `GraphEdge`, `ContextGraph`, `ContextGraphResponse`, and `OpenApiDocument` types.

- [x] **Step 2: Add SDK methods**

Add `healthz`, `getMeta`, `listProviders`, `getContextGraphPreview`, `getWorkspaceContextGraph`, and `getOpenApiDocument`.

- [x] **Step 3: Extend SDK route tests**

Test route construction for a platform method, a workspace graph method with encoded path segment, and OpenAPI document loading.

## Task 3: Contract Tests and Documentation

**Files:**
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`
- Modify: `README.md`
- Modify: `docs/api/rest-api.md`
- Modify: `docs/sdk/typescript-sdk.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-09-openapi-runtime-sdk-platform.md`

- [x] **Step 1: Expand contract test coverage**

Verify every SDK GET method has a matching OpenAPI operation: `healthz`, `getMeta`, `listProviders`, `getOpenApiDocument`, `getContextGraphPreview`, `getWorkspaceContextGraph`, plus the six discovery methods.

- [x] **Step 2: Document runtime OpenAPI route bilingually**

Mention `/api/v1/openapi.json`, SDK platform methods, and the contract test guard.

- [x] **Step 3: Update roadmap**

Record that the OpenAPI contract is now served by the API and consumed by SDK tests.

## Task 4: Verification

- [x] **Step 1: Run SDK tests and typecheck**

Run `pnpm --filter @contextlab/ts-sdk test` and `pnpm --filter @contextlab/ts-sdk lint`.

- [x] **Step 2: Run API tests**

Run `cargo fmt --all -- --check` and `cargo test -p contextlab-api`.

- [x] **Step 3: Run full workspace checks**

Run `pnpm check` and `pnpm test`.

Expected: all commands pass; PostgreSQL opt-in integration tests may remain ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is provided.
