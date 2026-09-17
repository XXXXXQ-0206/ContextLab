# Project Context List API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add project-scoped Context discovery so ContextLab can query the central Context layer with pagination, filtering, sorting, descriptions, and optional experiment filtering.

**Architecture:** `contextlab-storage` owns context list contracts and repository implementations. `server/api` exposes a thin Axum route that parses path/query parameters and delegates to `ContextRepository`.

**Tech Stack:** Rust stable, Axum, Serde, SQLx PostgreSQL runtime queries, Chrono timestamps, async repository traits.

---

## Task 1: Context Storage Contract

**Files:**
- Create: `crates/storage/src/context.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Add context list DTOs**

Create `ContextListQuery`, `ContextSort`, `ContextListItem`, `ContextList`, `ContextListPagination`, and `ContextRepository`.

- [x] **Step 2: Preserve discovery fields**

Add `description` and `created_at` to `ContextRecord`, and expose `experiment_id`, `description`, and `created_at` on `ContextListItem`.

- [x] **Step 3: Add optional experiment filter**

Support `experiment_id` in `ContextListQuery` so callers can narrow project contexts to one tracked experiment without losing contexts that have no experiment when the filter is absent.

- [x] **Step 4: Add query tests**

Cover normalization, overflow-safe offsets, and accepted/rejected sort values.

## Task 2: Repository Implementations

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement memory context listing**

Validate project existence in the preview projection, filter contexts by project, optionally filter by `experiment_id`, search `id/name/description`, sort deterministically, and paginate.

- [x] **Step 2: Implement PostgreSQL context listing**

Parse project UUIDs before querying, parse optional `experiment_id` UUID filters before querying, return `InvalidScope` for invalid UUIDs, return `ScopeUnavailable` for missing projects, and run count/page reads inside one repeatable-read read-only transaction.

- [x] **Step 3: Add storage tests**

Cover memory list, search, pagination, description return, experiment linkage, optional experiment filtering, created-at sorting, missing project behavior, safe SQL order clauses, and invalid PostgreSQL UUID without live DB access.

## Task 3: API Route and Documentation

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Add API state plumbing**

Expose `context_repository()` from `AppState` and delegate through the runtime repository enum.

- [x] **Step 2: Add `GET /api/v1/projects/{project_id}/contexts`**

Support `page`, `per_page`, `search`, `experiment_id`, and `sort`, returning `ContextList`.

- [x] **Step 3: Add API tests**

Cover default preview context list, filtering/pagination, `experiment_id` filtering, `sort=created_at`, invalid context sort, unknown memory project, invalid PostgreSQL project UUID, and invalid PostgreSQL experiment filter UUID.

- [x] **Step 4: Document bilingual behavior**

Document route shape, response fields, supported sort values, and the preview human-readable project id versus PostgreSQL UUID distinction.

## Task 4: Verification

- [x] **Step 1: Format**

Run `cargo fmt --all`.

- [x] **Step 2: Run targeted tests**

Run `cargo test -p contextlab-storage` and `cargo test -p contextlab-api`.

- [x] **Step 3: Run full default tests**

Run `cargo test --workspace` and `pnpm test`.

Expected: all default tests pass; the existing opt-in PostgreSQL integration test remains ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is configured.
