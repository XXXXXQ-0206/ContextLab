# Context Persistence Graph Projection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish Context Persistence + Graph Projection v1 without requiring a live database in local verification.

**Architecture:** `contextlab-storage` owns persistence-facing record shapes, SQL migration assets, and conversion into `contextlab-graph`. API routes consume storage projections rather than directly constructing graph fixtures.

**Tech Stack:** Rust stable, Serde, PostgreSQL SQL migration assets, ContextLab graph crate, Axum API composition.

---

## Task 1: Storage Crate

**Files:**
- Create: `crates/storage/Cargo.toml`
- Create: `crates/storage/src/lib.rs`
- Create: `crates/storage/migrations/0001_context_platform.sql`
- Modify: `Cargo.toml`

- [x] **Step 1: Add workspace member**

```toml
"crates/storage",
```

- [x] **Step 2: Add PostgreSQL schema migration**

The migration creates `workspaces`, `projects`, `experiments`, `contexts`, `context_components`, `context_commits`, and `evaluation_runs`.

- [x] **Step 3: Add record-to-graph projection**

`ContextGraphProjection::project()` converts storage records into a validated `ContextGraph`.

- [x] **Step 4: Add tests**

Run: `cargo test -p contextlab-storage`

Expected: migration asset and graph projection tests pass.

## Task 2: API Composition

**Files:**
- Modify: `server/api/Cargo.toml`
- Modify: `server/api/src/routes.rs`

- [x] **Step 1: Add storage dependency**

```toml
contextlab-storage = { path = "../../crates/storage" }
```

- [x] **Step 2: Use storage projection in context graph preview**

```rust
ContextGraphProjection::context_engineering_preview()
    .project()
    .expect("static projection records are valid")
```

## Task 3: Verification

- [ ] **Step 1: Run full checks**

Run: `pnpm check`

Expected: Rust tests, TypeScript typecheck, and Next build pass.

- [ ] **Step 2: Smoke test graph route**

Run: `curl http://127.0.0.1:3100/api/v1/context-graph/preview`

Expected: response includes graph nodes and edges from storage projection.
