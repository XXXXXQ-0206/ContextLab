# GraphDiff API and SDK Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose the tested `GraphDiff` domain operation as a validated, non-persistent REST and TypeScript SDK contract.

**Architecture:** `contextlab-diff-engine` remains the only layer that calculates a graph diff. The Axum handler accepts two explicit array-based snapshot DTOs, constructs validated `ContextGraph` aggregates, and maps the domain output into public DTOs. The SDK owns HTTP serialization; it does not reconstruct a diff. A POST route catalog mirrors the existing GET catalog so router registrations and checked-in OpenAPI remain synchronized.

**Tech Stack:** Rust, Axum, Serde, ContextLab graph/diff crates, TypeScript, Node test runner, checked-in OpenAPI 3.1.

---

### Task 1: Specify API and SDK Contract Tests

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `packages/ts-sdk/src/client.test.ts`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`

- [x] Add an Axum test that POSTs two node-array snapshots to `/api/v1/graph-diffs`, expects `200`, and asserts node-label changes, edge additions, and empty unrelated collections.
- [x] Add an Axum test with a duplicate node identifier or missing edge endpoint, expecting `400` and `{ "error": "invalid_graph_snapshot" }`.
- [x] Add SDK request tests that call `compareGraphs`, assert `POST`, `accept` and `content-type` headers, and assert a JSON body using `original` and `revised` snapshots.
- [x] Extend SDK/OpenAPI tests to enumerate `compareGraphs` separately from GET methods and require the request body plus `GraphDiffResponse` schema.
- [x] Run `cargo test -p contextlab-api` and `pnpm --filter @contextlab/ts-sdk test`; confirm the new tests fail only because the route and client method do not exist.

### Task 2: Implement Validated GraphDiff API

**Files:**
- Modify: `server/api/Cargo.toml`
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`

- [x] Add the `contextlab-diff-engine` dependency to the API crate.
- [x] Define explicit request DTOs with `original` and `revised` snapshots; a snapshot owns `nodes: Vec<...>` and `edges: Vec<...>`.
- [x] Convert each request snapshot by constructing graph nodes and edges through the graph crate constructors and `ContextGraph::add_node`/`add_edge`, preserving duplicate and endpoint validation.
- [x] Map `GraphDiff::between(&original, &revised)` to public response DTOs for added/removed nodes, modified nodes, and added/removed edges.
- [x] Map graph-construction failures to `ApiError::InvalidGraphSnapshot`, producing `400` and the established error body.
- [x] Introduce `PublicPostRoute::GraphDiff`, register it with `axum::routing::post`, and give it `operationId` `compareGraphs`.
- [x] Add a POST OpenAPI catalog test beside the existing GET guard; it must compare every declared public POST route’s path, operation ID, and no-parameter contract against OpenAPI.
- [x] Run `cargo test -p contextlab-api`; confirm the focused API tests and existing API suite pass.

### Task 3: Implement SDK and OpenAPI Surface

**Files:**
- Modify: `packages/ts-sdk/src/types.ts`
- Modify: `packages/ts-sdk/src/client.ts`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`
- Modify: `docs/api/openapi.json`

- [x] Add graph-node-kind and graph-edge-kind unions, snapshot input types, `GraphDiffRequest`, `GraphNodeChange`, and `GraphDiffResponse`.
- [x] Add `ContextLabClient.compareGraphs(request)` that POSTs JSON to `/api/v1/graph-diffs`; keep existing GET behavior unchanged and use one private request helper that accepts an explicit method/body.
- [x] Add the `compareGraphs` OpenAPI operation with a required JSON request body, `200` response, and the established `400` error response.
- [x] Describe all request and response schemas explicitly: graph snapshots use node arrays, not the read-only graph response map.
- [x] Run `pnpm --filter @contextlab/ts-sdk test` and `pnpm --filter @contextlab/ts-sdk lint`; confirm the client and contract tests pass.

### Task 4: Document Boundaries and Verify Integration

**Files:**
- Modify: `ARCHITECTURE.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] Update English and Chinese architecture/roadmap text: GraphDiff is now available through a pure snapshot API/SDK operation, while commit comparison, snapshot persistence, graph editing, and Web diff interaction remain unfinished.
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace`, and `pnpm check:web`.
- [x] Inspect the route and SDK contract assertions after the full gates complete; report only fresh evidence and choose the next unblocked roadmap increment without marking the long-term goal complete.

### Self-Review

- Domain ownership is preserved: API validates and presents; `diff-engine` computes; SDK transports; no React code participates.
- The request format is intentionally separate from `ContextGraphResponse` so malformed graph input cannot evade domain validation through map deserialization.
- Scope excludes storage migrations, version commits, graph editing, semantic/behavior/evaluation diffs, and Web UI.
