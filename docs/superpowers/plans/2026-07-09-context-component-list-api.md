# Context Component List API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `GET /api/v1/contexts/{context_id}/components` so clients can discover the prompt, memory, knowledge, tool, schema, model, workflow, and evaluation components that make up a Context, including each component's reproducible content fingerprint.

**Architecture:** `contextlab-storage` owns component list contracts, sort parsing, repository traits, in-memory behavior, and SQLx/PostgreSQL reads. `server/api` stays a thin Axum boundary that parses path/query inputs and delegates to `ContextComponentRepository`; OpenAPI, SDK types, SDK client methods, and bilingual docs follow the same checked-in contract guards used by the existing discovery routes.

**Tech Stack:** Rust stable, Axum, SQLx/PostgreSQL, Serde, TypeScript SDK, OpenAPI 3.1, bilingual Markdown docs.

---

### Task 1: Add Storage Component List Contracts

**Files:**
- Create: `crates/storage/src/component.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Add component list query, sort, item, list, and repository trait**

Create `crates/storage/src/component.rs` with `ComponentListQuery`, `ComponentSort`, `ComponentListItem`, `ComponentList`, and `ContextComponentRepository`. Query parameters are `page`, `per_page`, `search`, `kind`, and `sort`. Supported sort values are `name`, `-name`, `kind`, `-kind`, `created_at`, and `-created_at`.

- [x] **Step 2: Include `content_hash` and `created_at` on `ContextComponentRecord`**

Add `content_hash: String` and `created_at: DateTime<Utc>` to `ContextComponentRecord` because the migration already has `context_components.content_hash` and `context_components.created_at`. Component discovery should expose the hash for reproducibility while keeping full content and metadata out of the list response.

- [x] **Step 3: Export component contracts from storage**

Add `mod component;` and re-export component list contracts from `crates/storage/src/lib.rs`.

- [x] **Step 4: Update SQL projection reads**

Update PostgreSQL workspace graph projection to select `context_components.content_hash` and `context_components.created_at`, then construct `ContextComponentRecord { content_hash, created_at }`.

- [x] **Step 5: Run focused storage tests**

Run:

```powershell
cargo test -p contextlab-storage component -- --nocapture
```

Expected: component query normalization and sort parsing tests pass.

### Task 2: Implement Repository Behavior

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement in-memory component listing**

`InMemoryContextGraphRepository` should implement `ContextComponentRepository`, validate the context exists, filter by exact `kind`, search `id`, `kind`, `name`, and `content_hash`, sort deterministically, paginate, and return `ComponentList`.

- [x] **Step 2: Implement PostgreSQL component listing**

`PostgresContextGraphRepository` should parse `context_id` as UUID before connecting, verify the context exists, count active rows, filter by exact `kind`, search `id`, `kind`, `name`, and `content_hash`, sort through `ComponentSort::order_by_sql()`, and return `ComponentList`.

- [x] **Step 3: Add repository tests**

Add in-memory tests for search/pagination, kind filtering, unknown context, and created-at sorting. Add PostgreSQL lazy tests for non-UUID context rejection and safe `ORDER BY` clauses.

- [x] **Step 4: Run storage tests**

Run:

```powershell
cargo test -p contextlab-storage
```

Expected: all storage tests pass, with the existing PostgreSQL seed integration test still ignored by default.

### Task 3: Expose the API Route

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `server/api/src/routes.rs`

- [x] **Step 1: Add component repository to `AppState`**

Add `component_repository: Arc<dyn ContextComponentRepository>` to API state and thread it through environment-backed repository construction.

- [x] **Step 2: Add route params and handler**

Add `ComponentListParams` in `routes.rs`, parse `ComponentSort`, map invalid sort to `invalid_component_sort`, and expose `context_components` for `GET /api/v1/contexts/{context_id}/components`.

- [x] **Step 3: Register the route through the public GET route catalog**

Add `PublicGetRoute::ContextComponents`, path `/api/v1/contexts/{context_id}/components`, operationId `listComponents`, path parameter `context_id`, and query parameters `page`, `per_page`, `search`, `kind`, and `sort`.

- [x] **Step 4: Add API route tests**

Add tests for default component listing, kind filtering, invalid sort, unknown context, and PostgreSQL invalid context id rejection.

- [x] **Step 5: Run API tests**

Run:

```powershell
cargo test -p contextlab-api
```

Expected: API tests pass, including the route catalog/OpenAPI guard after the OpenAPI contract is updated in Task 4.

### Task 4: Update OpenAPI, SDK, and Docs

**Files:**
- Modify: `docs/api/openapi.json`
- Modify: `packages/ts-sdk/src/types.ts`
- Modify: `packages/ts-sdk/src/client.ts`
- Modify: `packages/ts-sdk/src/client.test.ts`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`
- Modify: `docs/api/rest-api.md`
- Modify: `docs/sdk/typescript-sdk.md`
- Modify: `README.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Update OpenAPI**

Add `GET /api/v1/contexts/{context_id}/components` with operationId `listComponents`, query parameters `page`, `per_page`, `search`, `kind`, and `sort`, and schemas for `ComponentListResponse` and `ComponentListItem` including `content_hash`.

- [x] **Step 2: Update TypeScript SDK**

Add `ComponentItem`, `ComponentSort`, `ComponentListQuery`, and `listComponents(contextId, query)` to the SDK, then extend route construction and OpenAPI contract tests.

- [x] **Step 3: Update bilingual docs**

Document component discovery in REST docs, SDK docs, README verification notes, and the long-term roadmap.

- [x] **Step 4: Run SDK tests**

Run:

```powershell
pnpm --filter @contextlab/ts-sdk lint
pnpm --filter @contextlab/ts-sdk test
```

Expected: SDK typecheck and tests pass.

### Task 5: Verify the Increment

**Files:**
- Read-only verification across Rust and TypeScript checks.

- [x] **Step 1: Format-check Rust**

Run:

```powershell
cargo fmt --all -- --check
```

Expected: PASS.

- [x] **Step 2: Run targeted packages**

Run:

```powershell
cargo test -p contextlab-storage
cargo test -p contextlab-api
pnpm --filter @contextlab/ts-sdk test
```

Expected: PASS.

- [x] **Step 3: Run broad checks**

Run:

```powershell
pnpm check
pnpm test
```

Expected: PASS.

### Self-Review

- Spec coverage: This plan advances context-first architecture by exposing Context components as first-class discoverable resources while preserving clean storage/API/SDK boundaries.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: Route operationId is `listComponents`; path and query names match the planned API catalog, SDK method, and OpenAPI document.
