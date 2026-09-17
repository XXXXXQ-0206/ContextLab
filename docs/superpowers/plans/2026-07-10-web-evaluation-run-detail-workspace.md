# Web Evaluation Run Detail Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect `GET /api/v1/contexts/{context_id}/evaluation-runs/{run_id}` into the Web Context workspace so users can inspect one selected run's persisted metrics JSON beside benchmark discovery.

**Architecture:** `apps/web/src/app/context-workspace-data.ts` remains the only Web live-data boundary and selects the first listed evaluation run before fetching its detail through the TypeScript SDK. Evaluation detail is optional for live data so a missing or failed detail read does not substitute preview metrics. `apps/web/src/app/context-workspace-preview.ts` owns deterministic API-shaped fixtures, while `apps/web/src/app/page.tsx` stays presentation-only and renders raw metrics without deriving scorecard or regression conclusions.

**Tech Stack:** Next.js App Router, React, TypeScript, `@contextlab/ts-sdk`, `@contextlab/ui`, CSS custom properties, `lucide-react`, Playwright visual QA, bilingual Markdown docs.

---

### Task 1: Extend Web Workspace Data Contracts

**Files:**
- Modify: `apps/web/src/app/context-workspace-preview.ts`
- Modify: `apps/web/src/app/context-workspace-data.ts`

- [x] **Step 1: Import the evaluation run detail DTO**

Add `EvaluationRunDetail` to the existing `@contextlab/ts-sdk` type imports in both Web data files.

- [x] **Step 2: Add deterministic evaluation detail fixture**

Export `selectedEvaluationRunDetail: EvaluationRunDetail` for `safety-regression`, including `metrics: { accuracy: 0.92, latency_ms: 820 }`. Keep it API-shaped and do not add scorecard, pass/fail, baseline, or regression fields.

- [x] **Step 3: Add detail to the workspace aggregate**

Add `selectedEvaluationRunDetail: EvaluationRunDetail | null` to `ContextWorkspaceData` and to `buildWorkspaceData(...)`.

- [x] **Step 4: Fetch live detail through the SDK boundary**

In `loadContextWorkspace()`, select the first run from `liveEvaluationRunPreview`, call `client.getEvaluationRun(selectedContext.id, selectedRun.id)`, and pass the returned detail into `buildWorkspaceData("live", ...)`.

### Task 2: Render Evaluation Run Detail in the Operations Panel

**Files:**
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`

- [x] **Step 1: Read selected detail from the data boundary**

Destructure `selectedEvaluationRunDetail` from `loadContextWorkspace()` and compute sorted metric entries.

- [x] **Step 2: Add evaluation detail route visibility**

Add `/api/v1/contexts/${selectedContext.id}/evaluation-runs/${selectedEvaluationRun.id}` to the route stack when a selected run exists so the detail contract is visible beside list contracts without inventing a preview detail for empty live data.

- [x] **Step 3: Render a compact metrics detail block**

Inside the Operations panel, render suite name, model version, temperature, executed timestamp, and raw metric key/value rows. Include bilingual copy that these are persisted metrics and scorecard/regression conclusions are future workflows.

- [x] **Step 4: Add resilient styles**

Add `.evaluation-detail`, `.evaluation-detail__summary`, `.evaluation-detail__meta`, `.metric-grid`, and `.metric-row` styles using design tokens, stable grids, and `overflow-wrap: anywhere`.

### Task 3: Update Bilingual Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/design-system/foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/sdk/typescript-sdk.md`

- [x] **Step 1: Update README current foundation**

Mention that `apps/web` now mirrors evaluation run discovery and selected metrics detail with API-shaped preview data and optional live API loading.

- [x] **Step 2: Update design-system foundation**

Document the selected evaluation run detail block as a dense operational pattern for raw metrics inspection.

- [x] **Step 3: Update roadmap**

Mark Web contract mirroring as including evaluation run detail before scorecard and regression dashboard workflows.

- [x] **Step 4: Update SDK runtime boundary docs**

Clarify that the Web live-data boundary now requests discovery lists plus selected component and evaluation detail reads.

### Task 4: Verify the Increment

**Files:**
- Modify: `apps/web/verify-context-workspace.py`

- [x] **Step 1: Extend visual QA assertions**

Assert the rendered page contains `Evaluation Run Detail`, the evaluation detail route, `metrics JSON`, `accuracy`, and `latency_ms`.

- [x] **Step 2: Run Web lint/build**

Run:

```powershell
pnpm --filter @contextlab/web lint
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

- Spec coverage: This plan advances benchmark-driven development by making the persisted metrics payload visible in the design-system-first workspace without moving API calls into UI components.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: `selectedEvaluationRunDetail`, `EvaluationRunDetail | null`, and `client.getEvaluationRun(...)` match the SDK and REST operation names while preserving a truthful empty state for live data.
