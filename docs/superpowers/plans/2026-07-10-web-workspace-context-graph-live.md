# Web Workspace Context Graph Live Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect `GET /api/v1/workspaces/{workspace_id}/context-graph` into the Web Context workspace so graph topology comes from the SDK/API graph contract instead of page-local reconstruction.

**Architecture:** `context-workspace-data.ts` owns the SDK call and preview fallback fixture. `context-workspace-presenter.ts` maps API graph nodes into deterministic screen coordinates and compact graph facts. `context-workspace-screen.tsx` remains a design-system-first renderer that only consumes the presenter model.

**Tech Stack:** Next.js App Router, TypeScript, `@contextlab/ts-sdk`, node:test, Playwright visual QA, bilingual Markdown docs.

---

### Task 1: Add Graph Data Contract Tests

**Files:**
- Modify: `apps/web/src/app/context-workspace-data.test.ts`
- Modify: `apps/web/src/app/context-workspace-presenter.test.ts`

- [x] **Step 1: Assert preview graph payload exists**

`loadContextWorkspace()` without an API URL exposes `workspaceContextGraph.graph.nodes["workspace:default"]`.

- [x] **Step 2: Assert live graph SDK request**

The live loader requests `/api/v1/workspaces/live-workspace/context-graph` and preserves the returned graph nodes/edges.

- [x] **Step 3: Assert presenter graph nodes come from payload**

Presenter tests inject a live graph payload and expect graph nodes and facts to reflect that payload instead of selected preview records.

### Task 2: Implement Data and Presenter Flow

**Files:**
- Modify: `apps/web/src/app/context-workspace-preview.ts`
- Modify: `apps/web/src/app/context-workspace-data.ts`
- Modify: `apps/web/src/app/context-workspace-presenter.ts`

- [x] **Step 1: Add API-shaped graph fixture**

Export `workspaceContextGraphPreview: ContextGraphResponse` with prefixed graph ids such as `workspace:default` and `context:support-resolution-agent`.

- [x] **Step 2: Fetch live workspace graph**

Call `client.getWorkspaceContextGraph(selectedWorkspace.id)` after selecting the workspace and before building live workspace data.

- [x] **Step 3: Present graph payload**

Sort graph nodes by kind/id, map node kinds to display labels, assign deterministic coordinates, and expose a `Graph nodes` score fact using node and edge counts.

### Task 3: Update UI QA and Docs

**Files:**
- Modify: `apps/web/verify-context-workspace.py`
- Modify: `README.md`
- Modify: `docs/design-system/foundation.md`
- Modify: `docs/sdk/typescript-sdk.md`
- Modify: `docs/roadmap/completion-criteria.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Extend visual QA**

Assert graph route visibility, graph node count facts, and source-boundary constraints.

- [x] **Step 2: Update bilingual docs**

Document that the Web shell consumes workspace Context Graph SDK/API payload and that remaining graph work is editing, diffing, and relationship inspection.

### Task 4: Verify

**Files:**
- No source changes beyond the files above.

- [x] **Step 1: Run targeted Web tests**

```powershell
pnpm --filter @contextlab/web test
```

Expected: PASS.

- [x] **Step 2: Run visual workspace QA**

```powershell
pnpm --filter @contextlab/web dev -- --hostname 127.0.0.1 --port 3000
python apps/web/verify-context-workspace.py
```

Expected: PASS with refreshed `target/context-workspace-desktop.png` and `target/context-workspace-mobile.png`.

- [x] **Step 3: Run Web/SDK gate**

```powershell
pnpm check:web
```

Expected: PASS.

### Self-Review

- Spec coverage: This closes the gap identified in the completion criteria: Web graph inspection now consumes the same SDK/API graph contract as backend/storage.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: Uses existing `ContextGraphResponse`, `GraphNode`, and `GraphEdge` DTOs from `@contextlab/ts-sdk`.
