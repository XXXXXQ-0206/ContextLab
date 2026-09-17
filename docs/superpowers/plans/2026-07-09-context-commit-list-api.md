# Context Commit List API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add context-scoped commit history discovery so ContextLab can expose Git-like version history before write/merge/replay workflows.

**Architecture:** `contextlab-storage` owns commit list DTOs and repository implementations. `server/api` exposes a thin Axum route that parses context path/query parameters and delegates to `ContextCommitRepository`, preserving clean architecture and PostgreSQL details behind storage.

**Tech Stack:** Rust stable, Axum, Serde, SQLx PostgreSQL runtime queries, Chrono timestamps, async repository traits.

---

## Task 1: Commit Storage Contract

**Files:**
- Create: `crates/storage/src/commit.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Add commit list DTOs**

Create `CommitListQuery`, `CommitSort`, `CommitListItem`, `CommitList`, `CommitListPagination`, and `ContextCommitRepository`. Query inputs are `page`, `per_page`, `search`, optional `branch_name`, and `sort`.

- [x] **Step 2: Preserve versioning fields**

Add `ContextCommitRecord` with `id`, `context_id`, `branch_name`, `message`, `parent_commit_ids`, `change_count`, `authored_at`, and `created_at`. Add `commits: Vec<ContextCommitRecord>` to `ContextGraphProjection` and seed at least one preview commit for `support-resolution-agent`.

- [x] **Step 3: Add query tests**

Cover query normalization, overflow-safe offsets, supported sort values (`authored_at`, `-authored_at`, `created_at`, `-created_at`, `branch_name`, `-branch_name`), and invalid sort rejection.

## Task 2: Repository Implementations

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement memory commit listing**

Validate context existence in the preview projection, filter commits by context, optionally filter by `branch_name`, search `id/message/branch_name`, sort deterministically, and paginate.

- [x] **Step 2: Implement PostgreSQL commit listing**

Parse context UUIDs before querying, return `InvalidScope` for invalid UUIDs, return `ScopeUnavailable` for missing contexts, read parent ids from `context_commit_parents` with ordered aggregation, compute `change_count` from `jsonb_array_length(changes)`, and run count/page reads inside one repeatable-read read-only transaction.

- [x] **Step 3: Add storage tests**

Cover memory list, search, pagination, branch filtering, authored-at sorting, parent ids, missing context behavior, safe SQL order clauses, and invalid PostgreSQL UUID without live DB access.

## Task 3: API Route and Documentation

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Add API state plumbing**

Expose `commit_repository()` from `AppState` and delegate through `WorkspaceDataRepository` so memory and PostgreSQL runtimes share the same route code.

- [x] **Step 2: Add `GET /api/v1/contexts/{context_id}/commits`**

Support `page`, `per_page`, `search`, `branch_name`, and `sort`, returning `CommitList`.

- [x] **Step 3: Add API tests**

Cover default preview commit list, filtering/pagination, branch filtering, `sort=authored_at`, invalid commit sort, unknown memory context, and invalid PostgreSQL context UUID.

- [x] **Step 4: Document bilingual behavior**

Document route shape, response fields, supported sort values, branch filter, and preview human-readable context id versus PostgreSQL UUID distinction.

## Task 4: Verification

- [x] **Step 1: Format**

Run `cargo fmt --all`.

- [x] **Step 2: Run targeted tests**

Run `cargo test -p contextlab-storage` and `cargo test -p contextlab-api`.

- [x] **Step 3: Run full default tests**

Run `cargo test --workspace` and `pnpm test`.

Expected: all default tests pass; the existing opt-in PostgreSQL integration test remains ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is configured.
