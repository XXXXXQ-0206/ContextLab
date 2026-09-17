# Atomic Context Commit Snapshot Capture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist a validated `ContextCommit`, its ordered parents, and one immutable `ContextGraph` snapshot as one atomic storage operation.

**Architecture:** `contextlab-versioning` continues to own framework-independent `ContextCommit`; `contextlab-storage` introduces a write command that binds the commit to a validated snapshot without adding a public HTTP write API. The existing in-memory repository keeps a synchronized mutable overlay for deterministic writer semantics. `PostgresContextGraphRepository` maps the command through one SQL transaction: validate the active Context and same-context parents, insert the commit, insert parent positions, insert the snapshot payload, then commit. Any validation or SQL failure leaves no partial commit state.

**Tech Stack:** Rust, ContextLab versioning/graph/storage crates, SQLx, PostgreSQL, Serde JSON, Tokio tests, Cargo, pnpm.

---

### Task 1: Define the Storage Write Contract

**Files:**
- Create: `crates/storage/src/commit_snapshot_writer.rs`
- Modify: `crates/storage/src/commit_graph_snapshot.rs`
- Modify: `crates/storage/src/repository.rs`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/Cargo.toml`

- [ ] **Step 1: Write failing command-construction tests**

```rust
let command = CreateContextCommitSnapshot::new(commit, graph, captured_at, 1)
    .expect("valid command");
assert_eq!(command.snapshot().commit_id(), command.commit().id().to_string());
```

Add a second test with duplicate parent ids and assert `CommitSnapshotWriteError::DuplicateParentCommitId`.

- [ ] **Step 2: Run the focused contract tests**

Run: `cargo test -p contextlab-storage commit_snapshot_writer`

Expected: FAIL because the command, trait, and error types do not exist.

- [ ] **Step 3: Implement the smallest validated command**

```rust
pub trait ContextCommitSnapshotWriter: Send + Sync {
    async fn create_commit_snapshot(
        &self,
        command: CreateContextCommitSnapshot,
    ) -> Result<CommitGraphSnapshot, StorageRepositoryError>;
}
```

`CreateContextCommitSnapshot::new` owns a `ContextCommit`, constructs its matching `CommitGraphSnapshot`, rejects duplicate parents before any storage I/O, and exposes immutable accessors. Make the graph JSON conversion crate-visible so both readers and writers use one validated payload representation.

- [ ] **Step 4: Run focused contract tests**

Run: `cargo test -p contextlab-storage commit_snapshot_writer`

Expected: PASS.

### Task 2: Implement and Test In-Memory Atomic Semantics

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Test: `crates/storage/src/memory.rs`

- [ ] **Step 1: Write failing async tests**

```rust
let snapshot = repository.create_commit_snapshot(command).await?;
assert_eq!(snapshot.commit_id(), command.commit().id().to_string());
assert!(repository.get_commit_graph_snapshot(context_id, commit_id).await?.is_some());
```

Add one test proving an unknown parent returns `StorageRepositoryError::ScopeUnavailable` and leaves the requested commit unreadable, plus one test proving retrying the same command returns `CommitAlreadyExists` without replacing its snapshot.

- [ ] **Step 2: Run the focused in-memory tests**

Run: `cargo test -p contextlab-storage in_memory_repository_creates_commit_snapshot`

Expected: FAIL because the writer is not implemented.

- [ ] **Step 3: Add synchronized in-memory write state**

Keep the static graph projection immutable and store accepted commits and snapshots in one lock-protected overlay. Check Context ownership and every ordered parent before inserting either map entry. Extend the existing commit read and snapshot read paths to include the overlay, retaining all current fixture behavior.

- [ ] **Step 4: Run storage tests**

Run: `cargo test -p contextlab-storage --lib`

Expected: PASS with the new atomic writer tests.

### Task 3: Implement and Test PostgreSQL Transaction Capture

**Files:**
- Modify: `crates/storage/src/postgres.rs`
- Test: `crates/storage/src/postgres.rs`

- [ ] **Step 1: Write failing tests for pre-I/O validation and disposable PostgreSQL behavior**

Add a no-connection test for command input that cannot target an active UUID Context through the SQL adapter. Extend the ignored disposable-database test to create a root commit and a child commit with an ordered parent, then read both through the existing discovery and snapshot contracts.

- [ ] **Step 2: Run focused tests**

Run: `cargo test -p contextlab-storage postgres_context_commit_snapshot_writer`

Expected: FAIL because the SQL writer does not exist.

- [ ] **Step 3: Add a single write transaction**

Within one `PgPool::begin()` transaction, lock/check the active Context, verify all parent ids belong to that Context, insert `context_commits` using conflict-aware SQL, insert ordered parent rows, serialize and insert `context_commit_graph_snapshots`, then commit. Return redacted structured storage errors; do not expose database URLs or driver messages.

- [ ] **Step 4: Run the storage suite and optional integration**

Run: `cargo test -p contextlab-storage --lib`

Expected: PASS; the disposable test remains ignored without `CONTEXTLAB_TEST_DATABASE_URL`.

When a disposable database is configured, run:

```powershell
$env:CONTEXTLAB_TEST_DATABASE_URL = 'postgres://.../contextlab_test'
cargo test -p contextlab-storage projects_seed_workspace_graph_from_postgres -- --ignored --exact
```

Expected: PASS with both pre-existing seed reads and transaction-created commit/snapshot reads.

### Task 4: Record the New Completion Boundary

**Files:**
- Modify: `ARCHITECTURE.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [ ] **Step 1: Update bilingual documentation**

State that commit capture now writes a commit, ordered same-context parents, and a snapshot atomically in the storage layer. Explicitly keep public mutation API, authorization, branch-head concurrency policy, historical GraphDiff orchestration, and UI editing/review out of the completed boundary.

- [ ] **Step 2: Run release gates**

Run: `cargo fmt --all -- --check; cargo test --workspace; pnpm check:web`

Expected: all commands pass; database integration remains explicitly optional.

### Self-Review

- The versioning domain has no SQLx or framework dependency.
- One capture command produces exactly one immutable snapshot for one new commit.
- Parent ordering is preserved, parents must belong to the same Context, and duplicate parents are rejected before persistence.
- Neither an API write route nor a Web mutation surface is introduced in this increment.
