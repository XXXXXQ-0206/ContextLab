# Workspace List API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first paginated, filterable, sortable list/query API for ContextLab workspaces.

**Architecture:** `contextlab-storage` owns the workspace list repository contract and implementations. `server/api` only parses HTTP query parameters, calls `WorkspaceRepository`, and returns a stable REST response shape.

**Tech Stack:** Rust stable, Axum query extractors, Serde, SQLx PostgreSQL runtime queries, existing ContextLab storage repository boundaries.

---

## Task 1: Storage Workspace List Contract

**Files:**
- Create: `crates/storage/src/workspace.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Add workspace list types**

Create `WorkspaceListQuery`, `WorkspaceList`, `WorkspaceListItem`, `WorkspaceListPagination`, and `WorkspaceSort` with default page/per-page limits and `name`, `-name`, `created_at`, `-created_at` sort strings.

- [x] **Step 2: Add `WorkspaceRepository` trait**

The trait should expose `list_workspaces(query)` and return `StorageRepositoryError`.

- [x] **Step 3: Implement in-memory workspace listing**

List preview workspaces, support search on name/id/slug, sort, and paginate deterministically.

- [x] **Step 4: Implement PostgreSQL workspace listing**

Query active workspaces, filter by search term, return total count, and sort only through a safe enum-to-SQL mapping.

## Task 2: Workspace REST API

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `server/api/src/routes.rs`

- [x] **Step 1: Add workspace repository to `AppState`**

Runtime selection should reuse memory or Postgres mode without exposing SQLx details to routes.

- [x] **Step 2: Add `GET /api/v1/workspaces`**

Support `page`, `per_page`, `search`, and `sort` query params. Invalid sort should return a 400 response.

- [x] **Step 3: Add API tests**

Cover default listing, search filtering, pagination, invalid sort, and Postgres invalid database URL redaction/config paths already covered by storage.

## Task 3: Documentation and Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-09-workspace-list-api.md`

- [x] **Step 1: Document route and query params bilingually**

Explain the response shape and default query behavior.

- [x] **Step 2: Run full verification**

Run `cargo fmt --all -- --check`, `cargo test --workspace --locked`, `pnpm check`, API smoke, and secret scan.

Expected: all default checks pass without PostgreSQL, `/api/v1/workspaces` returns a paginated response, and no real API keys appear outside `.env`.

## Follow-up Consistency Pass / 后续一致性加固

The contract consistency pass in `2026-07-09-workspace-list-contract-consistency.md` tightened the initial API by making memory and PostgreSQL workspace list semantics match for real `slug` values, `created_at` response fields, overflow-safe offsets, and repeatable-read read-only PostgreSQL pagination snapshots.

`2026-07-09-workspace-list-contract-consistency.md` 中的一致性加固让初版 API 更稳：memory 与 PostgreSQL workspace list 现在在真实 `slug`、`created_at` 响应字段、overflow-safe offset，以及 repeatable-read read-only PostgreSQL 分页快照上保持一致。
