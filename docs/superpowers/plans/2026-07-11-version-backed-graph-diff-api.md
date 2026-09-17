# Version-Backed GraphDiff API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compare two materialized Context commit graph snapshots through a stable read-only REST and TypeScript SDK contract.

**Architecture:** `contextlab-storage` remains the owner of immutable snapshot reads, while `contextlab-diff-engine` remains the only owner of graph comparison. `server/api` composes two `CommitGraphSnapshotRepository` reads from one Context scope, returns capture metadata with the existing structured GraphDiff payload, and distinguishes a missing snapshot from unknown Context or commit. The TypeScript SDK mirrors the checked-in OpenAPI contract; no writer, authorization policy, or graph-editing interface is introduced.

**Tech Stack:** Rust, Axum, async-trait, ContextLab storage/graph/diff crates, Serde, OpenAPI JSON, TypeScript SDK, Cargo, pnpm.

---

### Task 1: Extend API State with the Existing Snapshot Reader

**Files:**
- Modify: `server/api/src/lib.rs`
- Test: `server/api/src/lib.rs`

- [ ] **Step 1: Add failing route-state tests**

```rust
let response = state
    .commit_graph_snapshot_repository()
    .get_commit_graph_snapshot(context_id, commit_id)
    .await?;
assert!(response.is_some());
```

Create a state fixture that carries one materialized original commit and one materialized revised commit. Assert the current route catalog has no version-backed graph-diff operation before implementation.

- [ ] **Step 2: Run the focused API test**

Run: `cargo test -p contextlab-api version_backed_graph_diff`

Expected: FAIL because `AppState` has no snapshot repository or route.

- [ ] **Step 3: Add the dependency boundary**

Add `Arc<dyn CommitGraphSnapshotRepository>` to `AppState`, its debug output, constructor composition, and accessor. Keep the existing `ContextCommitRepository` read boundary intact; `WorkspaceDataRepository` delegates the snapshot trait to its in-memory and PostgreSQL variants.

- [ ] **Step 4: Run the focused API test**

Run: `cargo test -p contextlab-api version_backed_graph_diff`

Expected: route-state fixture compiles and existing API tests remain green.

### Task 2: Add Version-Backed GraphDiff Route and Error Semantics

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`
- Test: `server/api/src/lib.rs`

- [ ] **Step 1: Write failing route tests**

```rust
GET /api/v1/contexts/{context_id}/graph-diff?original_commit_id={id}&revised_commit_id={id}

assert_eq!(response.status(), StatusCode::OK);
assert_eq!(payload["original"]["commit_id"], original_commit_id);
assert_eq!(payload["diff"]["modified_nodes"][0]["node_id"], "context:agent");
```

Add negative tests: an existing commit without a materialized snapshot returns `409` and `commit_graph_snapshot_unavailable`; an unknown Context or commit continues to return the existing `404` storage scope error.

- [ ] **Step 2: Run focused route tests**

Run: `cargo test -p contextlab-api version_backed_graph_diff`

Expected: FAIL because the GET operation and response types do not exist.

- [ ] **Step 3: Implement the read-only composition**

Add `CommitGraphDiffResponse` with `context_id`, `original`, `revised`, and `diff` fields. Each snapshot reference exposes `commit_id`, RFC 3339 `captured_at`, and `schema_version`. Read both snapshots through `CommitGraphSnapshotRepository`, map `None` to `409 commit_graph_snapshot_missing`, and compute `GraphDiff::between` only after both immutable graphs are available. Add the route to the public GET catalog with the exact operation id `getCommitGraphDiff`.

- [ ] **Step 4: Run API tests**

Run: `cargo test -p contextlab-api`

Expected: PASS, including route/OpenAPI drift guards and the new success/error tests.

### Task 3: Extend OpenAPI and TypeScript SDK

**Files:**
- Modify: `docs/api/openapi.json`
- Modify: `packages/ts-sdk/src/types.ts`
- Modify: `packages/ts-sdk/src/client.ts`
- Modify: `packages/ts-sdk/src/index.ts`
- Modify: `packages/ts-sdk/src/client.test.ts`
- Modify: `packages/ts-sdk/src/openapi-contract.test.ts`

- [ ] **Step 1: Write failing SDK and contract tests**

```ts
await client.getCommitGraphDiff("context-1", "commit-a", "commit-b");
assert.equal(
  requestUrl,
  "http://127.0.0.1:3100/api/v1/contexts/context-1/graph-diff?original_commit_id=commit-a&revised_commit_id=commit-b"
);
```

Assert the OpenAPI GET operation uses `compareCommitGraphs`, has three path parameters, returns `CommitGraphDiffResponse`, and declares a `409` error response.

- [ ] **Step 2: Run focused SDK tests**

Run: `pnpm --filter @contextlab/ts-sdk test -- --test-name-pattern "commit graph"`

Expected: FAIL because the client method and OpenAPI operation are missing.

- [ ] **Step 3: Implement contract parity**

Add the OpenAPI path and schemas for snapshot references and the version-backed diff response. Add SDK types, export them, and implement `getCommitGraphDiff(contextId, originalCommitId, revisedCommitId)` as a GET request with encoded path and query segments.

- [ ] **Step 4: Run SDK checks**

Run: `pnpm --filter @contextlab/ts-sdk lint && pnpm --filter @contextlab/ts-sdk test`

Expected: PASS.

### Task 4: Record the Bilingual Boundary and Verify

**Files:**
- Modify: `ARCHITECTURE.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/storage/persistence-foundation.md`

- [ ] **Step 1: Update bilingual documentation**

State that version-backed GraphDiff now compares two already-materialized commit snapshots through read-only API/SDK contracts. Explicitly exclude public commit mutation, authorization, idempotency, branch-head concurrency, graph editing, semantic/behavior/evaluation diff, and Web diff review.

- [ ] **Step 2: Run full release gates**

Run: `cargo fmt --all -- --check; cargo test --workspace; pnpm check:web`

Expected: all commands pass; disposable PostgreSQL integration stays opt-in without `CONTEXTLAB_TEST_DATABASE_URL`.

### Self-Review

- The API never deserializes or re-implements graph payload validation; storage returns validated `ContextGraph` snapshots.
- The route cannot compare commits across Context scopes because one `context_id` owns both reads.
- A known commit without a snapshot is not reported as missing data or silently compared against the current projection.
- This plan deliberately excludes public mutation, branch policy, Web diff review, and any new graph persistence model.
