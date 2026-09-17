# API Route Contract Guard Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an API-side route catalog and OpenAPI drift guard so Axum route registration, runtime routes, and the checked-in REST contract stay aligned.

**Architecture:** `server/api/src/lib.rs` will own a small catalog of public GET routes. `build_router_with_state` will install routes through that catalog, and Rust tests will parse `docs/api/openapi.json` to verify path, operationId, path parameters, and query parameters against the same catalog. Documentation remains bilingual and describes the guard as a contract-maintenance rule.

**Tech Stack:** Rust stable, Axum 0.8, Serde JSON, existing `contextlab-api` tests, bilingual Markdown docs.

---

### Task 1: Add the Route Catalog

**Files:**
- Modify: `server/api/src/lib.rs`

- [x] **Step 1: Add the public GET route enum and metadata helpers**

Add a private `PublicGetRoute` enum near `build_router_with_state`, plus a `PUBLIC_GET_ROUTES` slice. Each variant returns `path`, `operation_id`, `path_parameters`, and `query_parameters` values for one public GET route.

Expected route metadata:

```rust
("/healthz", "healthz", &[], &[])
("/api/v1/meta", "getMeta", &[], &[])
("/api/v1/openapi.json", "getOpenApiDocument", &[], &[])
("/api/v1/providers", "listProviders", &[], &[])
("/api/v1/workspaces", "listWorkspaces", &[], &["page", "per_page", "search", "sort"])
("/api/v1/workspaces/{workspace_id}/projects", "listProjects", &["workspace_id"], &["page", "per_page", "search", "sort"])
("/api/v1/projects/{project_id}/experiments", "listExperiments", &["project_id"], &["page", "per_page", "search", "sort"])
("/api/v1/projects/{project_id}/contexts", "listContexts", &["project_id"], &["page", "per_page", "search", "sort", "experiment_id"])
("/api/v1/contexts/{context_id}/commits", "listCommits", &["context_id"], &["page", "per_page", "search", "sort", "branch_name"])
("/api/v1/contexts/{context_id}/evaluation-runs", "listEvaluationRuns", &["context_id"], &["page", "per_page", "search", "sort", "suite_name", "model_version"])
("/api/v1/context-graph/preview", "getContextGraphPreview", &[], &[])
("/api/v1/workspaces/{workspace_id}/context-graph", "getWorkspaceContextGraph", &["workspace_id"], &[])
```

- [x] **Step 2: Install routes through the catalog**

Replace the hand-written route chain inside `build_router_with_state` with a loop over `PUBLIC_GET_ROUTES`. Add an `install` method on `PublicGetRoute` that matches each variant to its existing handler and returns the updated `Router<AppState>`.

- [x] **Step 3: Run the API tests**

Run:

```powershell
cargo test -p contextlab-api
```

Expected: the existing API tests still pass before the new drift guard is added.

### Task 2: Add the OpenAPI Drift Guard

**Files:**
- Modify: `server/api/src/lib.rs`

- [x] **Step 1: Add a failing Rust test for route-contract parity**

Add a test in `#[cfg(test)] mod tests` that parses `docs/api/openapi.json`, resolves local `#/components/parameters/*` references, and verifies every OpenAPI GET path exactly matches `PUBLIC_GET_ROUTES` by:

- path
- operationId
- path parameter names
- query parameter names

The test should use `std::collections::BTreeMap` and sorted vectors so failures are deterministic.

- [x] **Step 2: Run the focused test**

Run:

```powershell
cargo test -p contextlab-api public_get_routes_match_openapi_contract -- --exact
```

Expected: PASS, because the current OpenAPI document already covers the route surface.

- [x] **Step 3: Run the full API test package**

Run:

```powershell
cargo test -p contextlab-api
```

Expected: PASS.

### Task 3: Document the Guard

**Files:**
- Modify: `docs/api/rest-api.md`
- Modify: `README.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Update REST API docs**

Add a bilingual section explaining that `server/api/src/lib.rs` owns the API-side public GET route catalog, while `docs/api/openapi.json` remains the checked-in contract artifact.

- [x] **Step 2: Update README verification notes**

Mention that `cargo test -p contextlab-api` checks route catalog and OpenAPI parity, while SDK tests check TypeScript client parity.

- [x] **Step 3: Update the roadmap**

Add a bilingual Phase 2 item recording the API-side route catalog and OpenAPI drift guard as part of the contract-first platform foundation.

### Task 4: Verify the Increment

**Files:**
- Read-only verification across Rust and TypeScript checks.

- [x] **Step 1: Format-check Rust**

Run:

```powershell
cargo fmt --all -- --check
```

Expected: PASS.

- [x] **Step 2: Run targeted Rust tests**

Run:

```powershell
cargo test -p contextlab-api
```

Expected: PASS.

- [x] **Step 3: Run SDK contract tests**

Run:

```powershell
pnpm --filter @contextlab/ts-sdk test
```

Expected: PASS.

- [x] **Step 4: Run broad checks when targeted checks pass**

Run:

```powershell
pnpm check
pnpm test
```

Expected: PASS.

### Self-Review

- Spec coverage: This plan protects REST-first API documentation, SDK alignment, bilingual documentation, and clean architecture by keeping route metadata in the API composition layer.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: Route metadata names match current SDK operation IDs and OpenAPI paths.
