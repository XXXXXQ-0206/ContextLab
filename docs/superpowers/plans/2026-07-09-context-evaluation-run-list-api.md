# Context Evaluation Run List API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add context-scoped evaluation run discovery so ContextLab can expose benchmark, regression, and model-run history before evaluation dashboards and scorecards.

**Architecture:** `contextlab-storage` owns evaluation run list DTOs and repository implementations. `server/api` exposes a thin Axum route that parses context path/query parameters and delegates to `EvaluationRunRepository`, keeping metrics storage details behind the storage boundary.

**Tech Stack:** Rust stable, Axum, Serde, SQLx PostgreSQL runtime queries, Chrono timestamps, async repository traits.

---

## Task 1: Evaluation Storage Contract

**Files:**
- Create: `crates/storage/src/evaluation_run.rs`
- Modify: `crates/storage/src/records.rs`
- Modify: `crates/storage/src/projection.rs`
- Modify: `crates/storage/src/lib.rs`

- [x] **Step 1: Add evaluation run list DTOs**

Create `EvaluationRunListQuery`, `EvaluationRunSort`, `EvaluationRunListItem`, `EvaluationRunList`, `EvaluationRunListPagination`, and `EvaluationRunRepository`. Query inputs are `page`, `per_page`, `search`, optional `suite_name`, optional `model_version`, and `sort`.

- [x] **Step 2: Preserve dashboard fields**

Extend `EvaluationRunRecord` with `model_version`, `temperature`, `metric_count`, `executed_at`, and `created_at`. Seed the preview run with `deepseek-chat`, `0.2`, two metrics, and the deterministic preview timestamp.

- [x] **Step 3: Add query tests**

Cover query normalization, overflow-safe offsets, supported sort values (`executed_at`, `-executed_at`, `created_at`, `-created_at`, `suite_name`, `-suite_name`, `model_version`, `-model_version`), and invalid sort rejection.

## Task 2: Repository Implementations

**Files:**
- Modify: `crates/storage/src/memory.rs`
- Modify: `crates/storage/src/postgres.rs`

- [x] **Step 1: Implement memory evaluation run listing**

Validate context existence in the preview projection, filter runs by context, optionally filter by `suite_name` and `model_version`, search `id/suite_name/model_version`, sort deterministically, and paginate.

- [x] **Step 2: Implement PostgreSQL evaluation run listing**

Parse context UUIDs before querying, return `InvalidScope` for invalid UUIDs, return `ScopeUnavailable` for missing contexts, compute `metric_count` from `jsonb_object_length(metrics)`, and run count/page reads inside one repeatable-read read-only transaction.

- [x] **Step 3: Add storage tests**

Cover memory list, search, pagination, suite/model filtering, executed-at sorting, missing context behavior, safe SQL order clauses, and invalid PostgreSQL UUID without live DB access.

## Task 3: API Route and Documentation

**Files:**
- Modify: `server/api/src/routes.rs`
- Modify: `server/api/src/lib.rs`
- Modify: `README.md`
- Modify: `docs/storage/persistence-foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Add API state plumbing**

Expose `evaluation_run_repository()` from `AppState` and delegate through `WorkspaceDataRepository` so memory and PostgreSQL runtimes share the same route code.

- [x] **Step 2: Add `GET /api/v1/contexts/{context_id}/evaluation-runs`**

Support `page`, `per_page`, `search`, `suite_name`, `model_version`, and `sort`, returning `EvaluationRunList`.

- [x] **Step 3: Add API tests**

Cover default preview evaluation run list, filtering/pagination, suite filtering, model filtering, `sort=executed_at`, invalid evaluation run sort, unknown memory context, and invalid PostgreSQL context UUID.

- [x] **Step 4: Document bilingual behavior**

Document route shape, response fields, supported sort values, filters, metric count behavior, and preview human-readable context id versus PostgreSQL UUID distinction.

## Task 4: Verification

- [x] **Step 1: Format**

Run `cargo fmt --all`.

- [x] **Step 2: Run targeted tests**

Run `cargo test -p contextlab-storage` and `cargo test -p contextlab-api`.

- [x] **Step 3: Run full default tests**

Run `cargo test --workspace` and `pnpm test`.

Expected: all default tests pass; the existing opt-in PostgreSQL integration test remains ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is configured.
