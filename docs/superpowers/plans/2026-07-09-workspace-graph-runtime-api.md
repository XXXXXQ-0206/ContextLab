# Workspace Graph Runtime API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose a workspace-scoped Context Graph API and let runtime configuration choose the workspace graph repository implementation.

**Architecture:** Preview graph reads remain deterministic and in-memory. Workspace graph reads use the same `ContextGraphProjectionRepository` contract and can run through either the in-memory repository for local development or `PostgresContextGraphRepository` when explicitly configured.

**Tech Stack:** Rust stable, Axum path extractors, SQLx lazy PostgreSQL pool construction, ContextLab storage repository traits.

---

## Task 1: Runtime Repository Selection

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `crates/storage/src/postgres.rs`
- Modify: `.env.example`

- [x] **Step 1: Add lazy Postgres repository construction**

Expose a `PostgresContextGraphRepository::connect_lazy(database_url)` constructor so API composition can build the adapter without connecting during startup.

- [x] **Step 2: Split preview and workspace graph repositories in `AppState`**

Keep preview graph reads stable through the in-memory fixture while allowing workspace graph reads to use a configured repository.

- [x] **Step 3: Add env-driven state construction**

Use `CONTEXTLAB_GRAPH_REPOSITORY=memory|postgres` and `CONTEXTLAB_DATABASE_URL` for workspace graph repository selection, with `DATABASE_URL` as a compatibility fallback.

## Task 2: Workspace Graph API

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `server/api/src/routes.rs`
- Test: `server/api/src/lib.rs`

- [x] **Step 1: Add `GET /api/v1/workspaces/{workspace_id}/context-graph`**

The route should load `GraphProjectionScope::Workspace { workspace_id }`, project it into `ContextGraph`, and return the same graph response shape as preview.

- [x] **Step 2: Cover success, unavailable workspace, and Postgres invalid-scope behavior**

Add API tests for the default in-memory workspace graph route, unknown workspace 404, and Postgres lazy repository returning 400 for a non-UUID workspace id without connecting to a database.

## Task 3: Documentation and Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-09-workspace-graph-runtime-api.md`

- [x] **Step 1: Document route and env config bilingually**

Explain the preview route, workspace route, and repository selection environment variables.

- [x] **Step 2: Run full verification**

Run: `cargo fmt --all -- --check`, `cargo test --workspace --locked`, `pnpm check`, API smoke, and secret scan.

Expected: all checks pass, graph routes return 200 for preview/default workspace, and no raw API keys appear outside `.env`.
