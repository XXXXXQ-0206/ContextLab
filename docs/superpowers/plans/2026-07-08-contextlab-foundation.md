# ContextLab Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the first verifiable ContextLab foundation: bilingual docs, a modular Rust workspace, tested context/version/diff/evaluation crates, and a minimal REST API.

**Architecture:** Keep business logic in Rust crates and expose it through framework-owned presentation layers. The first slice creates domain contracts and deterministic behavior without introducing persistence or UI business logic.

**Tech Stack:** Rust stable, Cargo workspace, Axum, Tokio, Serde, Chrono, UUID, pnpm workspace metadata.

---

## File Structure

- Create: `Cargo.toml` for the Rust workspace.
- Create: `crates/context-core/*` for Context identity, components, metadata, and validation.
- Create: `crates/versioning/*` for branch, change, and commit domain types.
- Create: `crates/diff-engine/*` for deterministic line-oriented text diff.
- Create: `crates/evaluation/*` for metrics, evaluation runs, and scorecards.
- Create: `server/api/*` for Axum routes.
- Create: `README.md`, `docs/roadmap/long-term-roadmap.md`, and `docs/adr/0001-modular-monorepo.md` for bilingual project guidance.

## Task 1: Workspace Contract

**Files:**
- Create: `Cargo.toml`
- Create: `package.json`
- Create: `pnpm-workspace.yaml`
- Create: `.editorconfig`
- Create: `.gitignore`
- Create: `.env.example`
- Create: `rust-toolchain.toml`

- [x] **Step 1: Define Rust workspace members**

```toml
[workspace]
resolver = "3"
members = [
  "crates/context-core",
  "crates/versioning",
  "crates/diff-engine",
  "crates/evaluation",
  "server/api",
]
```

- [x] **Step 2: Add shared package metadata and dependencies**

```toml
[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "Apache-2.0 OR MIT"
```

- [ ] **Step 3: Verify workspace**

Run: `cargo metadata --format-version 1 --no-deps`

Expected: Cargo reports all five workspace packages.

## Task 2: Context Core

**Files:**
- Create: `crates/context-core/Cargo.toml`
- Create: `crates/context-core/src/lib.rs`
- Create: `crates/context-core/src/identity.rs`
- Create: `crates/context-core/src/validation.rs`
- Create: `crates/context-core/src/component.rs`
- Create: `crates/context-core/src/context.rs`

- [x] **Step 1: Implement stable IDs**

```rust
pub struct ContextId(Uuid);
pub struct ComponentId(Uuid);
```

- [x] **Step 2: Implement validated names and components**

```rust
pub struct NonEmptyString(String);
pub struct ContextComponent {
    id: ComponentId,
    kind: ContextComponentKind,
    name: NonEmptyString,
    content_hash: ContentHash,
}
```

- [x] **Step 3: Implement Context aggregate**

```rust
pub struct Context {
    id: ContextId,
    project_id: ProjectId,
    name: NonEmptyString,
    components: Vec<ContextComponent>,
}
```

- [ ] **Step 4: Verify tests**

Run: `cargo test -p contextlab-context-core`

Expected: Context validation and component attachment tests pass.

## Task 3: Version, Diff, Evaluation, and API Shell

**Files:**
- Create: `crates/versioning/*`
- Create: `crates/diff-engine/*`
- Create: `crates/evaluation/*`
- Create: `server/api/*`

- [x] **Step 1: Implement context commits**

```rust
pub struct ContextCommit {
    id: CommitId,
    parent_ids: Vec<CommitId>,
    context_id: ContextId,
    branch: BranchName,
    changes: Vec<ContextChange>,
}
```

- [x] **Step 2: Implement deterministic text diff**

```rust
pub enum DiffLine {
    Unchanged(String),
    Added(String),
    Removed(String),
}
```

- [x] **Step 3: Implement evaluation scorecards**

```rust
pub struct Scorecard {
    run_count: usize,
    averages: BTreeMap<MetricKind, f64>,
}
```

- [x] **Step 4: Implement API health and metadata routes**

```rust
Router::new()
    .route("/healthz", get(healthz))
    .route("/api/v1/meta", get(meta))
```

- [ ] **Step 5: Verify workspace tests**

Run: `cargo test --workspace`

Expected: all crate and API tests pass.
