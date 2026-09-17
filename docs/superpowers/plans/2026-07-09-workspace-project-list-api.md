# Workspace Project List API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add workspace-scoped project discovery so ContextLab can query the next Context Graph layer through the same paginated, filterable, sortable repository pattern as workspaces.

**Architecture:** `contextlab-storage` owns project list contracts, in-memory preview behavior, and PostgreSQL SQLx reads. `server/api` only parses Axum path/query inputs and returns stable REST DTOs from a `ProjectRepository`.

**Tech Stack:** Rust stable, Axum, Serde, SQLx PostgreSQL runtime queries, Chrono timestamps, async repository traits.

---

## Task 1: Shared List Infrastructure

**Files:**
- Create: `crates/storage/src/listing.rs`
- Modify: `crates/storage/src/workspace.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Extract common pagination helpers**

Move default page values, search normalization, overflow-safe offset calculation, and pagination metadata into `listing.rs`.

- [x] **Step 2: Keep workspace behavior unchanged**

Update workspace list types to use the shared helpers while preserving public names such as `WorkspaceListPagination`.

## Task 2: Project Storage Contract

**Files:**
- Create: `crates/storage/src/project.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Extend `ProjectRecord`**

Add `slug` and `created_at` to match the PostgreSQL project table fields needed for list parity.

- [x] **Step 2: Add project list DTOs and sort parsing**

Create `ProjectListQuery`, `ProjectSort`, `ProjectListItem`, `ProjectList`, `ProjectListPagination`, and `ProjectRepository`.

- [x] **Step 3: Add query tests**

Cover normalization, overflow-safe offsets, and accepted/rejected sort values.

## Task 3: Repository Implementations

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement memory project listing**

Validate workspace existence in the preview projection, filter projects by workspace, search `id/name/slug`, sort deterministically, and paginate.

- [x] **Step 2: Implement PostgreSQL project listing**

Parse workspace UUIDs before querying, return `InvalidScope` for invalid UUIDs, return `ScopeUnavailable` for missing workspaces, and run count/page reads inside one repeatable-read read-only transaction.

- [x] **Step 3: Add memory tests**

Cover list, search, pagination, real slug return, created-at ordering, and missing workspace behavior without requiring a live database.

## Task 4: API Route and Documentation

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Add API state plumbing**

Expose a `project_repository()` from `AppState` and delegate through the runtime repository enum.

- [x] **Step 2: Add `GET /api/v1/workspaces/{workspace_id}/projects`**

Support `page`, `per_page`, `search`, and `sort`, returning `ProjectList`.

- [x] **Step 3: Add API tests**

Cover default preview project list, filtering/pagination, `sort=created_at`, invalid project sort, unknown memory workspace, and invalid PostgreSQL UUID.

- [x] **Step 4: Document bilingual behavior**

Document route shape, response fields, supported sort values, and the preview `default` versus PostgreSQL UUID distinction.

## Task 5: Verification

- [x] **Step 1: Format**

Run `cargo fmt --all`.

- [x] **Step 2: Run targeted tests**

Run `cargo test -p contextlab-storage` and `cargo test -p contextlab-api`.

- [x] **Step 3: Run full default tests**

Run `cargo test --workspace` and `pnpm test`.

Expected: all default tests pass; the existing opt-in PostgreSQL integration test remains ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is configured.
