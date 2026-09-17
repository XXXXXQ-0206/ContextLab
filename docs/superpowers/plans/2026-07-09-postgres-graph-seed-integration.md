# Postgres Graph Seed Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a PostgreSQL seed fixture and optional integration path for validating workspace Context Graph projection against real database rows.

**Architecture:** `contextlab-storage` continues to own schema, fixtures, and SQLx repository behavior. Default tests stay no-database and deterministic, while an ignored integration test can be run explicitly with `CONTEXTLAB_TEST_DATABASE_URL` to exercise migration, seed data, and `PostgresContextGraphRepository`.

**Tech Stack:** Rust stable, SQLx PostgreSQL runtime queries, Axum-independent storage repository tests, PostgreSQL SQL fixture assets.

---

## Task 1: Consistent Postgres Reads

**Files:**
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Load workspace projection inside a read-only transaction**

Start a transaction, set it to read-only, run all projection queries against that transaction, and commit only after the complete `ContextGraphProjection` is assembled.

- [x] **Step 2: Keep no-database tests green**

Run: `cargo test -p contextlab-storage`

Expected: all existing storage tests pass without a live PostgreSQL database.

## Task 2: Seed Fixture

**Files:**
- Create: `crates/storage/fixtures/workspace_graph_seed.sql`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Add deterministic workspace graph seed SQL**

The fixture should insert one active workspace graph containing workspace, project, experiment, context, prompt/memory/knowledge/tool/model components, and evaluation run rows. It should also include soft-deleted rows to prove repository filters can exclude them in integration tests.

- [x] **Step 2: Export and test the fixture asset**

Expose `WORKSPACE_GRAPH_SEED` and assert it contains the seed workspace id plus soft-deleted rows.

## Task 3: Optional Integration Test

**Files:**
- Modify: `crates/storage/src/postgres.rs`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/superpowers/plans/2026-07-09-postgres-graph-seed-integration.md`

- [x] **Step 1: Add ignored SQLx integration test**

The test should run only when explicitly requested, use `CONTEXTLAB_TEST_DATABASE_URL`, apply `CONTEXT_PLATFORM_MIGRATION`, apply `WORKSPACE_GRAPH_SEED`, and verify `PostgresContextGraphRepository` projects the seed workspace into a graph with context and evaluation nodes.

- [x] **Step 2: Document the gated test command bilingually**

Document that normal checks do not need PostgreSQL, and that the integration test is opt-in for local/CI environments with a disposable test database.

- [x] **Step 3: Run full verification**

Run: `cargo fmt --all -- --check`, `cargo test --workspace --locked`, `pnpm check`, and secret scan.

Expected: default verification passes without PostgreSQL, and no real API keys appear outside `.env`.
