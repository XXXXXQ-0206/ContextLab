# Web Version-Backed Graph Diff Review Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let ContextLab users inspect a read-only graph diff between two persisted commits, with bilingual UI and no preview substitution after a live failure.

**Architecture:** Preserve the existing `data -> presenter -> screen` boundary. The server loader selects adjacent commits and fetches their version-backed diff with the TypeScript SDK; a client-only review island owns only commit selection and request state. A same-origin Next route proxies live follow-up requests so `CONTEXTLAB_WEB_API_BASE_URL` remains server-only. Preview supplies a deterministic fixture; preview fallback intentionally reports unavailable rather than presenting fixture data as live.

**Tech Stack:** Next.js App Router, React/TypeScript, `@contextlab/ts-sdk`, `@contextlab/ui`, Node test runner via `tsx --test`.

---

## File Responsibilities

- `apps/web/src/app/context-workspace-preview.ts`: deterministic commit graph-diff preview fixture.
- `apps/web/src/app/context-workspace-data.ts`: select the default ordered commit pair and fetch its live diff without changing fallback semantics.
- `apps/web/src/app/context-workspace-presenter.ts`: translate commits, snapshot references, diff rows, source availability, and bilingual labels into a screen view model.
- `apps/web/src/app/context-graph-review.tsx`: client island for accessible commit selectors, compare command, request state, and read-only result rendering.
- `apps/web/src/app/api/contexts/[contextId]/graph-diff/route.ts`: validate route query, call the server-only SDK client, and return a controlled JSON error.
- `apps/web/src/app/context-workspace-screen.tsx`: replace the static semantic-diff sample with the review island.
- `apps/web/src/app/globals.css`: responsive review layout styles using existing design tokens.
- `packages/ui/src/primitives/select.tsx` and `packages/ui/src/index.ts`: reusable tokenized native select primitive; no page-local form control.
- `apps/web/src/app/*.test.ts`: focused data, presenter, proxy-route, and review rendering behavior tests.

## Task 1: Establish the Data Contract With Failing Tests

**Files:**
- Modify: `apps/web/src/app/context-workspace-data.test.ts`
- Modify: `apps/web/src/app/context-workspace-presenter.test.ts`
- Modify: `apps/web/src/app/context-workspace-data.ts`
- Modify: `apps/web/src/app/context-workspace-presenter.ts`

- [x] Add a failing data test for two newest distinct commits selecting older as original and newer as revised.
- [x] Add a failing data test that a `409 commit_graph_snapshot_missing` produces unavailable review data, not a preview replacement on live data.
- [x] Add a failing presenter test for bilingual snapshot metadata, five diff counts, and read-only change rows.
- [x] Run `pnpm --filter @contextlab/web test` and confirm failures name the absent review contract.
- [x] Add the minimal loader and presenter fields to satisfy the tests.
- [x] Run the focused Web tests and confirm they pass.

## Task 2: Add a Reusable Select and Server-Only Proxy

**Files:**
- Create: `packages/ui/src/primitives/select.tsx`
- Modify: `packages/ui/src/index.ts`
- Create: `apps/web/src/app/api/contexts/[contextId]/graph-diff/route.ts`
- Create: `apps/web/src/app/api/contexts/[contextId]/graph-diff/route.test.ts`

- [x] Add a failing primitive test or type-level usage that requires label association, native keyboard operation, and tokenized class composition.
- [x] Add failing route tests for missing query values, identical commit ids, successful SDK proxying, and `409` propagation.
- [x] Run the focused tests and confirm red state.
- [x] Implement the minimal reusable `Select` primitive and index export.
- [x] Implement the route with strict query validation and a server-only `ContextLabClient`; never accept a client-provided API base URL.
- [x] Run focused tests until green.

## Task 3: Render the Read-Only Review Island

**Files:**
- Create: `apps/web/src/app/context-graph-review.tsx`
- Modify: `apps/web/src/app/context-workspace-screen.tsx`
- Modify: `apps/web/src/app/globals.css`
- Modify: `apps/web/src/app/context-workspace-preview.ts`
- Create: `apps/web/src/app/context-graph-review.test.tsx`

- [x] Add a failing review test for default commits, disabled compare when the ids match, and explicit unavailable state.
- [x] Add a failing review test for returned diff rows/counts and the snapshot-not-materialized message.
- [x] Run focused review tests and confirm red state.
- [x] Implement a client island that only manages selector/request state and delegates all domain presentation to presenter-shaped props.
- [x] Replace `Semantic Diff Preview` with the bilingual `Commit Graph Review / 提交图谱审查` block using `Select`, `Button`, `DefinitionGrid`, `StatGrid`, `StackTable`, `StatusPill`, and `CodeChip`.
- [x] Add responsive token-based styles with no horizontal overflow at mobile width.
- [x] Run focused review tests until green.

## Task 4: Verify the Vertical Slice

**Files:**
- Modify: `apps/web/verify-context-workspace.py` only if its current assertions mention the removed static preview.

- [x] Update the visual verifier to assert selectors, compare state, success rows, unavailable state, and mobile overflow behavior.
- [x] Run `pnpm --filter @contextlab/ts-sdk test`.
- [x] Run `pnpm --filter @contextlab/web test`, `pnpm --filter @contextlab/web lint`, and `pnpm --filter @contextlab/web build`.
- [x] Run `cargo fmt --all -- --check` and `cargo test --workspace` to protect the shared contract.
- [x] Start the Web app and run `python apps/web/verify-context-workspace.py` for desktop and mobile evidence.
- [x] Review the final diff for secret exposure, API-base leakage, preview/live confusion, and undocumented behavior.
