# Web Component Inventory Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect `GET /api/v1/contexts/{context_id}/components` into the Web Context workspace so users can inspect the versionable component inventory and reproducible content fingerprints beside commits and evaluation runs.

**Architecture:** `apps/web/src/app/context-workspace-data.ts` remains the runtime data boundary and is the only place that calls the TypeScript SDK. `apps/web/src/app/context-workspace-preview.ts` owns deterministic API-shaped fixtures, while `apps/web/src/app/page.tsx` stays presentation-only and composes shared primitives from `@contextlab/ui` with token-driven CSS.

**Tech Stack:** Next.js App Router, React, TypeScript, `@contextlab/ts-sdk`, `@contextlab/ui`, CSS custom properties, `lucide-react`, bilingual Markdown docs.

---

### Task 1: Extend Web Workspace Data Contracts

**Files:**
- Modify: `apps/web/src/app/context-workspace-preview.ts`
- Modify: `apps/web/src/app/context-workspace-data.ts`

- [x] **Step 1: Import component DTOs in preview data**

Add `ComponentItem` to the existing `@contextlab/ts-sdk` type imports in `context-workspace-preview.ts`.

- [x] **Step 2: Add deterministic component fixtures**

Export `componentPreview: ListResponse<ComponentItem>` with system prompt, memory, knowledge, MCP server, and model configuration rows for `support-resolution-agent`, matching the Rust preview projection. Each row must include `id`, `context_id`, `kind`, `name`, `content_hash`, and `created_at`.

- [x] **Step 3: Add component data to the workspace aggregate**

Add `componentPreview: ListResponse<ComponentItem>` and `componentCount: number` to `ContextWorkspaceData` in `context-workspace-data.ts`.

- [x] **Step 4: Fetch live components through the SDK boundary**

In `loadContextWorkspace()`, call `client.listComponents(selectedContext.id, { page: 1, per_page: 20, sort: "kind" })` in the same `Promise.all` as commits and evaluation runs. Validate the list is non-empty with `firstItem(liveComponentPreview, "component")`, and pass it through `buildWorkspaceData()`.

### Task 2: Render Component Inventory in the Workspace

**Files:**
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`

- [x] **Step 1: Read component inventory from the data boundary**

Destructure `componentPreview` and `componentCount` from `loadContextWorkspace()`.

- [x] **Step 2: Add component route visibility**

Add `/api/v1/contexts/${selectedContext.id}/components` to the route stack so component discovery is visible with the other context-scoped contracts.

- [x] **Step 3: Add graph and score signals**

Add a `Components` graph node and update the score grid so component count is visible as a core Context fact.

- [x] **Step 4: Add a compact component inventory block**

Inside the Operations panel, render a token-driven table/list with component kind, name, short hash, and created timestamp. Keep the layout dense, responsive, and free of nested panels.

- [x] **Step 5: Add resilient CSS**

Add `.component-table`, `.component-table__head`, `.component-table__row`, and `.hash-chip` styles using design tokens, stable grid tracks, `overflow-wrap: anywhere`, and mobile single-column behavior.

### Task 3: Update Bilingual Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/design-system/foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Update README current foundation**

Mention that `apps/web` now mirrors Context, commit, component, and evaluation run discovery contracts with API-shaped preview data and optional live API loading.

- [x] **Step 2: Update design-system foundation**

Document that the workspace exposes component inventory and fingerprints through shared primitives and token-driven operational tables.

- [x] **Step 3: Update roadmap**

Mark Web contract mirroring as including component discovery before prompt/schema editing workflows.

### Task 4: Verify the Increment

**Files:**
- Read-only verification across Web, workspace checks, and visual QA.

- [x] **Step 1: Lint the Web app**

Run:

```powershell
pnpm --filter @contextlab/web lint
```

Expected: PASS.

- [x] **Step 2: Build the Web app**

Run:

```powershell
pnpm --filter @contextlab/web build
```

Expected: PASS.

- [x] **Step 3: Run repository checks**

Run:

```powershell
pnpm check
pnpm test
```

Expected: PASS.

- [x] **Step 4: Run visual workspace QA**

Start the Web dev server:

```powershell
pnpm --filter @contextlab/web dev -- --hostname 127.0.0.1 --port 3000
```

Then run:

```powershell
python apps/web/verify-context-workspace.py
```

Expected: PASS with refreshed `target/context-workspace-desktop.png` and `target/context-workspace-mobile.png`.

### Self-Review

- Spec coverage: This plan advances the context-first architecture by making Context components and content fingerprints visible in the Web workspace without moving data fetching into UI components.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: `componentPreview`, `componentCount`, `ComponentItem`, and `client.listComponents(...)` match the existing SDK and Web data-boundary naming style.
