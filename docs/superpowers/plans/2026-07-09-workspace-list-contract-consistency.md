# Workspace List Contract Consistency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make workspace list semantics consistent across in-memory preview storage and PostgreSQL storage before higher-level project, experiment, and collaboration APIs build on the same repository pattern.

**Architecture:** `contextlab-storage` remains the owner of persistence-facing records and repository contracts. Workspace list responses expose the same identity, slug, creation timestamp, pagination, filtering, and sorting semantics regardless of backend; `server/api` stays a thin Axum presentation layer.

**Tech Stack:** Rust stable, Axum, Serde, SQLx PostgreSQL runtime queries, Chrono timestamps, async repository traits.

---

## Task 1: Align Workspace Record Shape

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/storage/Cargo.toml`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Add timestamp support to storage**

Add `chrono.workspace = true` to `crates/storage/Cargo.toml`, and enable SQLx's `chrono` feature in the workspace dependency so `TIMESTAMPTZ` rows decode into `DateTime<Utc>`.

- [x] **Step 2: Extend `WorkspaceRecord`**

Store `slug` and `created_at` in `WorkspaceRecord` so in-memory fixtures can represent the same workspace facts as PostgreSQL rows.

- [x] **Step 3: Update projection fixtures and Postgres projection reads**

Give the deterministic preview workspace an explicit `slug` and `created_at`, and have PostgreSQL graph projection select `id`, `name`, `slug`, and `created_at`.

## Task 2: Align Workspace List Semantics

**Files:**
- Modify: `crates/storage/src/workspace.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Add `created_at` to list items**

Expose `created_at` in `WorkspaceListItem` so clients can inspect the field used by `created_at` and `-created_at` sorting.

- [x] **Step 2: Use real slug and timestamp in memory**

Build memory list items from `WorkspaceRecord.slug` and `WorkspaceRecord.created_at`, not from derived `id` values.

- [x] **Step 3: Sort memory by the requested field**

Implement `NameAsc`, `NameDesc`, `CreatedAtAsc`, and `CreatedAtDesc` as distinct comparators with deterministic `id` tie-breakers.

- [x] **Step 4: Make offset arithmetic safe**

Change `WorkspaceListQuery::offset()` to return a `u64` computed from normalized values, avoiding `u32` multiplication overflow for large page numbers.

- [x] **Step 5: Query Postgres from one consistent snapshot**

Run workspace count and item reads inside a `REPEATABLE READ READ ONLY` transaction, matching the repository boundary used by graph projection reads.

## Task 3: Tests, Documentation, and Verification

**Files:**
- Modify: `crates/storage/src/workspace.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `server/api/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-09-workspace-list-api.md`
- Modify: `docs/superpowers/plans/2026-07-09-workspace-list-contract-consistency.md`

- [x] **Step 1: Add storage unit tests**

Cover normalized query trimming/clamping, overflow-safe offsets, real preview slugs, and memory sorting by creation time where name order differs.

- [x] **Step 2: Add API tests**

Assert `/api/v1/workspaces` returns `slug` and `created_at`, and accepts `sort=created_at` through the route.

- [x] **Step 3: Update bilingual docs**

Document `/api/v1/workspaces`, response fields, sort values, backend parity, and the consistent read snapshot rule in English and Chinese.

- [x] **Step 4: Verify**

Run:

```bash
cargo fmt --all
cargo test -p contextlab-storage
cargo test -p contextlab-api
cargo test --workspace
pnpm test
```

Expected: all default checks pass, with the existing PostgreSQL integration test still ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is explicitly configured.
