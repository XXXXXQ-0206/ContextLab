# Commit Graph Snapshot Contract Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish a validated, immutable-in-use graph snapshot model and read-only repository boundary for ContextGraph state associated with a Context commit.

**Architecture:** `contextlab-graph` remains the owner of valid graph aggregates. `contextlab-storage` adds `CommitGraphSnapshot`, which binds an already-valid `ContextGraph` to a context id, commit id, capture time, and schema version. `CommitGraphSnapshotRepository` returns `Option<CommitGraphSnapshot>` so a known commit without a materialized snapshot is distinguished from an unknown context. The existing in-memory storage repository implements the contract; PostgreSQL migration, write APIs, history diff API, and Web presentation remain future increments.

**Tech Stack:** Rust, Serde, Chrono, async-trait, ContextLab graph/storage crates, `cargo test`.

---

### Task 1: Specify Snapshot Domain and Repository Behavior

**Files:**
- Create: `crates/storage/src/commit_graph_snapshot.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] Define failing unit tests for a snapshot that preserves its context id, commit id, captured time, schema version, and graph node count.
- [x] Define failing unit tests for empty context id, empty commit id, and schema version `0`.
- [x] Define failing async repository tests: a known context and matching commit returns `Some(snapshot)`; a known context without a snapshot returns `None`; an unknown context returns the existing `StorageRepositoryError::ScopeUnavailable` form.
- [x] Run `cargo test -p contextlab-storage commit_graph_snapshot`; confirm failures are caused by missing symbols and behavior.

### Task 2: Implement Domain Contract and In-Memory Reader

**Files:**
- Create: `crates/storage/src/commit_graph_snapshot.rs`
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/lib.rs`

- [ ] Implement `CommitGraphSnapshot` with this public shape:

```rust
pub struct CommitGraphSnapshot {
    context_id: String,
    commit_id: String,
    graph: ContextGraph,
    captured_at: DateTime<Utc>,
    schema_version: u16,
}
```

- [x] Implement `CommitGraphSnapshot::new(...) -> Result<Self, CommitGraphSnapshotError>` that trims and rejects empty identifiers and rejects schema version `0`.
- [x] Implement getters returning immutable data or references; do not expose mutation of the stored graph.
- [x] Define `CommitGraphSnapshotRepository` with:

```rust
async fn get_commit_graph_snapshot(
    &self,
    context_id: String,
    commit_id: String,
) -> Result<Option<CommitGraphSnapshot>, StorageRepositoryError>;
```

- [x] Extend `InMemoryContextGraphRepository` with a private keyed snapshot collection and a `with_commit_graph_snapshots(preview, snapshots)` constructor that rejects duplicate `(context_id, commit_id)` keys.
- [x] Implement the repository method by validating the context through the existing projection helper and then looking up the exact context/commit key.
- [x] Export the new public types from `crates/storage/src/lib.rs`.
- [x] Run `cargo test -p contextlab-storage`; confirm all storage tests pass.

### Task 3: Record Scope and Verify

**Files:**
- Modify: `ARCHITECTURE.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] Update English and Chinese text to state that the repository now has a read-only, in-memory commit graph snapshot contract, while PostgreSQL persistence, capture-on-commit writes, history diff API integration, and Web review remain open.
- [x] Run `cargo fmt --all -- --check`, `cargo test --workspace`, and `pnpm check:web`.
- [x] Treat the resulting contract only as a prerequisite for the next PostgreSQL snapshot migration; do not declare version-backed GraphDiff complete.

### Self-Review

- The snapshot is a storage boundary over the validated graph domain rather than a second graph implementation.
- `Option` is used only for a missing snapshot of a known context; invalid or unknown scopes continue to use the project-wide structured repository error.
- This plan deliberately excludes database schema changes, snapshot writers, API/SDK endpoints, GraphDiff orchestration, and Web UI.
