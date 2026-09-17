# SQLx Graph Repository Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first SQLx/PostgreSQL adapter for loading `ContextGraphProjection` records behind the existing storage repository contract.

**Architecture:** `contextlab-storage` gains a `PostgresContextGraphRepository` that owns SQL query details and implements `ContextGraphProjectionRepository`. `server/api` changes `AppState` to depend on the repository trait object, so in-memory preview and future SQL-backed runtime composition share the same route code.

**Tech Stack:** Rust stable, SQLx PostgreSQL runtime, async-trait, Axum, ContextLab storage and graph crates.

---

## Task 1: Repository Polymorphism

**Files:**
- Modify: `server/api/src/lib.rs`
- Test: `server/api/src/lib.rs`

- [x] **Step 1: Make `AppState` own a graph repository trait object**

Use `Arc<dyn ContextGraphProjectionRepository>` so API composition can accept in-memory and PostgreSQL repositories through one boundary.

- [x] **Step 2: Keep existing API route tests green**

Run: `cargo test -p contextlab-api`

Expected: health, meta, providers, graph success, and graph error route tests pass.

## Task 2: SQLx Storage Adapter

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/storage/Cargo.toml`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/repository.rs`
- Create: `crates/storage/src/postgres.rs`
- Test: `crates/storage/src/postgres.rs`

- [x] **Step 1: Add SQLx and UUID dependencies**

Add workspace dependencies for SQLx PostgreSQL and UUID decoding.

- [x] **Step 2: Add stored component kind parsing**

Implement string parsing for the v1 component taxonomy so SQL rows can become typed `StoredComponentKind` values.

- [x] **Step 3: Add `PostgresContextGraphRepository`**

Implement `ContextGraphProjectionRepository` for workspace-scoped graph projection queries. The adapter should compile without a live database by using runtime SQLx queries rather than compile-time query macros.

- [x] **Step 4: Add no-database unit coverage**

Test invalid workspace UUID handling and component kind parsing without connecting to PostgreSQL.

## Task 3: Documentation and Verification

**Files:**
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-09-sqlx-graph-repository.md`

- [x] **Step 1: Document the adapter boundary bilingually**

Explain that PostgreSQL query details live in `contextlab-storage`, while API uses only the repository contract.

- [x] **Step 2: Run full verification**

Run: `cargo fmt --all`, `cargo test --workspace --locked`, and `pnpm check`.

Expected: all Rust and Web checks pass.

## Task 4: Review Hardening

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `crates/storage/src/postgres.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Split storage API error status codes**

Map unavailable scope to 404, invalid scope to 400, and storage/database failures to 500.

- [x] **Step 2: Add no-database preview-scope test**

Verify `PostgresContextGraphRepository` rejects preview scope without connecting to PostgreSQL.

- [x] **Step 3: Add migration taxonomy drift coverage**

Verify every `StoredComponentKind::ALL` value appears in the migration component kind check.
