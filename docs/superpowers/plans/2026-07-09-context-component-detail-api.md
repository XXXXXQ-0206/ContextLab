# Context Component Detail API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `GET /api/v1/contexts/{context_id}/components/{component_id}` so clients can read one Context component's stable identity, taxonomy kind, reproducible fingerprint, metadata, and timestamps before edit and diff workflows exist.

**Architecture:** `contextlab-storage` owns the component detail response shape and repository method, backed by both the deterministic in-memory projection and SQLx/PostgreSQL. `server/api` remains a thin Axum boundary using the existing public GET route catalog and OpenAPI drift guard, while the TypeScript SDK and bilingual docs mirror the same REST contract.

**Tech Stack:** Rust stable, Axum, SQLx/PostgreSQL, Serde, serde_json, TypeScript SDK, OpenAPI 3.1, bilingual Markdown docs.

---

### Task 1: Add Storage Component Detail Contract

**Files:**
- Modify: `crates/storage/src/component.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/postgres.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Add metadata and updated timestamp to component records**

Extend `ContextComponentRecord` with `metadata: serde_json::Value` and `updated_at: DateTime<Utc>`. The PostgreSQL migration already has `context_components.metadata` and `context_components.updated_at`; preview projection records should use deterministic JSON metadata and the existing preview timestamp.

- [x] **Step 2: Add component detail item**

Add `ComponentDetail` to `crates/storage/src/component.rs` with `id`, `context_id`, `kind`, `name`, `content_hash`, `metadata`, `created_at`, and `updated_at`. Do not add a `content` field because the current schema does not persist component body content.

- [x] **Step 3: Extend repository trait**

Add `get_component(&self, context_id: String, component_id: String) -> Result<ComponentDetail, StorageRepositoryError>` to `ContextComponentRepository`.

- [x] **Step 4: Export detail type**

Re-export `ComponentDetail` from `crates/storage/src/lib.rs`.

### Task 2: Implement In-Memory and PostgreSQL Detail Reads

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement in-memory detail lookup**

`InMemoryContextGraphRepository::get_component` should verify the context exists, find an exact component id within that context, return `ComponentDetail`, and return `StorageRepositoryError::ScopeUnavailable { scope: "component:{context_id}/{component_id}" }` when the component is missing.

- [x] **Step 2: Implement PostgreSQL detail lookup**

`PostgresContextGraphRepository::get_component` should parse both ids as UUIDs before connecting, verify the context exists, query one active `context_components` row by `(context_id, component_id)`, parse `kind`, and return `ComponentDetail`. Missing context should use `scope: "context:{context_id}"`; missing component should use `scope: "component:{context_id}/{component_id}"`.

- [x] **Step 3: Add storage tests**

Add tests for in-memory happy path, missing component, missing context, and PostgreSQL invalid component id rejection before DB access.

### Task 3: Expose API Route

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`

- [x] **Step 1: Add handler**

Add `context_component` to `routes.rs` using `Path<(String, String)>` and returning `Json<ComponentDetail>`.

- [x] **Step 2: Add public route catalog entry**

Add `PublicGetRoute::ContextComponent`, path `/api/v1/contexts/{context_id}/components/{component_id}`, operationId `getComponent`, and path parameters `context_id` and `component_id`.

- [x] **Step 3: Add API tests**

Add tests for default detail response, unknown component, and PostgreSQL invalid component id rejection.

### Task 4: Update SDK, OpenAPI, and Bilingual Docs

**Files:**
- Modify: `docs/api/openapi.json`
- Modify: `packages/ts-sdk/src/types.ts`
- Modify: `packages/ts-sdk/src/client.ts`
- Modify: `packages/ts-sdk/src/client.test.ts`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`
- Modify: `docs/api/rest-api.md`
- Modify: `docs/sdk/typescript-sdk.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `README.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Update OpenAPI**

Add `GET /api/v1/contexts/{context_id}/components/{component_id}` with operationId `getComponent`, `ContextId` and `ComponentId` path parameters, and `ComponentDetail` schema.

- [x] **Step 2: Update TypeScript SDK**

Add `ComponentDetail` type and `getComponent(contextId, componentId)` client method. Extend SDK route-construction and OpenAPI contract tests.

- [x] **Step 3: Update docs bilingually**

Document that component list responses remain lightweight, while detail responses expose metadata and timestamps but not body content because body storage is a future object/versioning concern.

### Task 5: Verify the Increment

**Files:**
- Read-only verification across Rust and TypeScript checks.

- [x] **Step 1: Run focused storage tests**

Run:

```powershell
cargo test -p contextlab-storage component
```

Expected: PASS.

- [x] **Step 2: Run focused API tests**

Run:

```powershell
cargo test -p contextlab-api context_component
```

Expected: PASS.

- [x] **Step 3: Run SDK checks**

Run:

```powershell
pnpm --filter @contextlab/ts-sdk lint
pnpm --filter @contextlab/ts-sdk test
```

Expected: PASS.

- [x] **Step 4: Run full checks**

Run:

```powershell
pnpm check
pnpm test
```

Expected: PASS. PostgreSQL seed integration remains ignored unless a disposable database is provided.

### Self-Review

- Spec coverage: This plan advances context-first infrastructure by adding a precise detail read contract for Context components, without pretending component body storage exists.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: `ComponentDetail`, `get_component`, `context_component`, and SDK `getComponent` all describe one component detail read.
