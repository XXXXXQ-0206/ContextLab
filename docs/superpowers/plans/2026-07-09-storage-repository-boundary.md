# Storage Repository Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a clean repository boundary for Context Graph projection so API and future SQLx adapters depend on storage contracts instead of raw tables.

**Architecture:** `contextlab-storage` owns persistence records, projection logic, repository traits, and a deterministic in-memory repository for tests and preview routes. SQLx/PostgreSQL adapters can later implement the same trait without changing API handlers or graph semantics.

**Tech Stack:** Rust stable, async-trait, Serde, ContextLab graph crate, Axum API composition.

---

## Task 1: Storage Module Split

**Files:**
- Modify: `crates/storage/src/lib.rs`
- Create: `crates/storage/src/records.rs`
- Create: `crates/storage/src/projection.rs`
- Create: `crates/storage/src/repository.rs`
- Create: `crates/storage/src/memory.rs`

- [x] **Step 1: Move record types into `records.rs`**

Keep `WorkspaceRecord`, `ProjectRecord`, `ExperimentRecord`, `ContextRecord`, `ContextComponentRecord`, `EvaluationRunRecord`, and `StoredComponentKind` public.

- [x] **Step 2: Move `ContextGraphProjection` into `projection.rs`**

Keep `ContextGraphProjection::project()` as the only place that maps records into graph semantics.

## Task 2: Repository Contract

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/storage/Cargo.toml`
- Create: `crates/storage/src/repository.rs`
- Create: `crates/storage/src/memory.rs`

- [x] **Step 1: Add `async-trait` dependency**

```toml
async-trait = "0.1.89"
```

- [x] **Step 2: Add repository trait**

```rust
#[async_trait]
pub trait ContextGraphProjectionRepository {
    async fn load_context_graph_projection(&self, scope: GraphProjectionScope)
        -> Result<ContextGraphProjection, StorageRepositoryError>;
}
```

- [x] **Step 3: Add in-memory implementation**

`InMemoryContextGraphRepository` stores a projection and returns it for preview and tests.

- [x] **Step 4: Add tests**

Run: `cargo test -p contextlab-storage`

Expected: in-memory repository loads preview records and projects them into a graph.

## Task 3: API Composition

**Files:**
- Modify: `server/api/src/lib.rs`
- Modify: `server/api/src/routes.rs`

- [x] **Step 1: Add repository to `AppState`**

API state owns `InMemoryContextGraphRepository` until SQLx is introduced.

- [x] **Step 2: Make `/api/v1/context-graph/preview` load via repository trait**

The route awaits repository output and then projects records into `ContextGraph`.

- [x] **Step 3: Return structured API errors**

Projection failures map to `graph_projection_failed` instead of panicking, keeping the API route ready for SQL-backed repositories.

## Task 3.5: Migration Hardening

**Files:**
- Modify: `crates/storage/migrations/0001_context_platform.sql`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/records.rs`

- [x] **Step 1: Make soft-delete uniqueness reusable**

Use partial unique indexes for active `workspaces.slug`, `projects(workspace_id, slug)`, and `experiments(project_id, branch_name)`.

- [x] **Step 2: Normalize commit parent edges**

Replace `parent_commit_ids UUID[]` with `context_commit_parents(commit_id, parent_commit_id, position)`.

- [x] **Step 3: Constrain stored component kinds**

Add the v1 `context_components.kind` check constraint and align `StoredComponentKind` with `evaluation`.

## Task 4: Verification

- [x] **Step 1: Run full checks**

Run: `pnpm check`

Expected: Rust tests, TypeScript typecheck, and Next production build pass.

- [x] **Step 2: Smoke test graph route**

Run: `curl http://127.0.0.1:3100/api/v1/context-graph/preview`

Expected: graph response still includes context and evaluation nodes.
