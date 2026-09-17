# PostgreSQL Commit Graph Snapshots Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist immutable graph snapshots by Context commit in PostgreSQL and read them through the existing `CommitGraphSnapshotRepository` contract.

**Architecture:** Add a second migration that creates `context_commit_graph_snapshots` keyed only by `commit_id`; context ownership is verified through an indexed join to `context_commits`, preventing duplicate or divergent context identifiers. The JSONB payload uses an explicit vector-shaped graph DTO, which reconstructs `ContextGraph` via its validated constructors before `CommitGraphSnapshot::new` is called. `PostgresContextGraphRepository` implements the existing read-only trait; no writer, API, SDK, or UI surface is added.

**Tech Stack:** PostgreSQL, SQLx, Rust, Serde, Chrono, ContextLab storage/graph crates, disposable-database integration tests.

---

### Task 1: Specify Migration and SQLx Reader Tests

**Files:**
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/postgres.rs`
- Modify: `crates/storage/fixtures/workspace_graph_seed.sql`

- [ ] Add a migration assertion for `context_commit_graph_snapshots`, its primary key, JSONB payload, schema-version check, and commit lookup index.
- [ ] Add unit tests proving PostgreSQL snapshot reads reject non-UUID context and commit scopes before opening a database connection.
- [ ] Extend the ignored disposable PostgreSQL integration test to seed one commit/snapshot, read it through `get_commit_graph_snapshot`, and assert `Some` plus graph structure; assert a second seeded commit without a snapshot returns `None`.
- [ ] Run focused storage tests; confirm the new implementation tests fail before the migration and reader exist.

### Task 2: Add Immutable Schema and Validated Payload Conversion

**Files:**
- Create: `crates/storage/migrations/0002_context_commit_graph_snapshots.sql`
- Modify: `crates/storage/src/lib.rs`
- Modify: `crates/storage/src/commit_graph_snapshot.rs`

- [ ] Create `context_commit_graph_snapshots` with `commit_id UUID PRIMARY KEY REFERENCES context_commits(id) ON DELETE RESTRICT`, `schema_version SMALLINT NOT NULL CHECK (schema_version > 0)`, `graph JSONB NOT NULL CHECK (jsonb_typeof(graph) = 'object')`, and `captured_at TIMESTAMPTZ NOT NULL`.
- [ ] Add an index through the existing `context_commits(context_id, id)` lookup path; do not store a duplicate context id in the snapshot table.
- [ ] Compose `CONTEXT_PLATFORM_MIGRATION` from `0001` and `0002` so fresh disposable databases continue to bootstrap from one exported asset.
- [ ] Add internal vector-shaped payload types and conversion helpers that serialize an existing graph and rebuild nodes/edges through `GraphNode::new`, `GraphEdge::new`, `ContextGraph::add_node`, and `ContextGraph::add_edge`.
- [ ] Map malformed JSONB or invalid graph payloads to the existing redacted storage database error boundary without exposing raw database URLs or credentials.

### Task 3: Implement PostgreSQL Read Adapter

**Files:**
- Modify: `crates/storage/src/postgres.rs`

- [ ] Parse context and commit UUIDs with existing helpers before opening a transaction.
- [ ] Use the existing repeatable-read transaction helper, verify the context exists, then fetch the commit-associated snapshot with a left join.
- [ ] Return `ScopeUnavailable` for a commit outside the context, `None` for an existing commit without a row, and a validated `CommitGraphSnapshot` for a materialized payload.
- [ ] Commit the read transaction before returning each successful result.
- [ ] Run `cargo test -p contextlab-storage` and the ignored disposable-DB test when `CONTEXTLAB_TEST_DATABASE_URL` is configured.

### Task 4: Document Boundary and Verify

**Files:**
- Modify: `ARCHITECTURE.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [ ] Update English and Chinese documentation: snapshots are now persistable/readable in PostgreSQL, but capture-on-commit writes, historical GraphDiff API orchestration, and Web review are still open.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --workspace`, and `pnpm check:web`.
- [ ] Choose the next increment as snapshot writer/capture semantics or version-backed GraphDiff API orchestration based on which dependency remains unmet.

### Self-Review

- Snapshot ownership derives from the authoritative commit row, so a payload cannot claim a different context.
- JSONB is only used as the immutable graph payload; scope, timestamps, and version are normalized columns with constraints.
- The plan excludes graph editing, snapshot mutation/upsert behavior, public write endpoints, and UI work.
