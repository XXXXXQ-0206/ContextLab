# Project Experiment List API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add project-scoped experiment discovery so ContextLab can query the Experiment layer of the Context Graph with pagination, filtering, sorting, branch visibility, and backend-consistent repository semantics.

**Architecture:** `contextlab-storage` owns experiment list contracts and repository implementations. `server/api` parses path/query parameters and delegates to `ExperimentRepository`, keeping Axum presentation separate from storage logic.

**Tech Stack:** Rust stable, Axum, Serde, SQLx PostgreSQL runtime queries, Chrono timestamps, async repository traits.

---

## Task 1: Experiment Storage Contract

**Files:**
- Create: `crates/storage/src/experiment.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Add experiment list DTOs**

Create `ExperimentListQuery`, `ExperimentSort`, `ExperimentListItem`, `ExperimentList`, `ExperimentListPagination`, and `ExperimentRepository`.

- [x] **Step 2: Preserve branch visibility**

Add `branch_name` and `created_at` to `ExperimentRecord` and expose both on `ExperimentListItem`.

- [x] **Step 3: Add query tests**

Cover normalization, overflow-safe offsets, and accepted/rejected sort values including `branch_name` and `-branch_name`.

## Task 2: Repository Implementations

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement memory experiment listing**

Validate project existence in the preview projection, filter experiments by project, search `id/name/branch_name`, sort deterministically, and paginate.

- [x] **Step 2: Implement PostgreSQL experiment listing**

Parse project UUIDs before querying, return `InvalidScope` for invalid UUIDs, return `ScopeUnavailable` for missing projects, and run count/page reads inside one repeatable-read read-only transaction.

- [x] **Step 3: Add storage tests**

Cover memory list, search, pagination, branch-name return, created-at sorting, branch-name sorting, missing project behavior, safe SQL order clauses, and invalid PostgreSQL UUID without live DB access.

## Task 3: API Route and Documentation

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Add API state plumbing**

Expose `experiment_repository()` from `AppState` and delegate through the runtime repository enum.

- [x] **Step 2: Add `GET /api/v1/projects/{project_id}/experiments`**

Support `page`, `per_page`, `search`, and `sort`, returning `ExperimentList`.

- [x] **Step 3: Add API tests**

Cover default preview experiment list, filtering/pagination, `sort=created_at`, `sort=branch_name`, invalid experiment sort, unknown memory project, and invalid PostgreSQL UUID.

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
