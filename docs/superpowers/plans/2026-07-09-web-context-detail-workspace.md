# Web Context Detail Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade the Web workspace shell into a design-system-first Context detail surface that exposes Context discovery, commit history, and evaluation run discovery using API-shaped data.

**Architecture:** The page stays presentation-only and imports typed preview data from a local app data module shaped after the REST discovery contracts. Styling remains token-driven through `@contextlab/design-system` and reusable primitives from `@contextlab/ui`; no business logic or backend fetch coupling is added inside UI components.

**Tech Stack:** Next.js App Router, React, TypeScript, CSS custom properties, `@contextlab/ui`, `lucide-react`.

---

## Task 1: API-Shaped Preview Data

**Files:**
- Create: `apps/web/src/app/context-workspace-preview.ts`

- [x] **Step 1: Add typed discovery DTOs**

Define `ListResponse<T>`, `WorkspaceItem`, `ProjectItem`, `ContextItem`, `CommitItem`, and `EvaluationRunItem` using the same field names as the REST APIs.

- [x] **Step 2: Add deterministic preview responses**

Export `workspacePreview`, `projectPreview`, `contextPreview`, `commitPreview`, and `evaluationRunPreview` using the preview ids `default`, `support-ai`, `support-resolution-agent`, `support-resolution-agent-initial`, and `safety-regression`.

## Task 2: Context Detail Page

**Files:**
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`

- [x] **Step 1: Replace ad hoc data with preview imports**

Import the API-shaped preview data and render the selected context, commit history, and evaluation run rows from those typed values.

- [x] **Step 2: Add context detail and route cards**

Add a compact context detail panel showing `project_id`, `experiment_id`, `description`, and the relevant REST discovery endpoints.

- [x] **Step 3: Add commit timeline**

Render commit rows with branch name, message, parent count, change count, authored timestamp, and the `GET /api/v1/contexts/{context_id}/commits` route.

- [x] **Step 4: Add evaluation run table**

Render evaluation run rows with suite, model, temperature, metric count, executed timestamp, and the `GET /api/v1/contexts/{context_id}/evaluation-runs` route.

- [x] **Step 5: Add shared Panel primitive**

Move the framed workspace surface into `packages/ui/src/primitives/panel.tsx`, export it from `packages/ui/src/index.ts`, and style it through token-driven `.cl-panel` classes in `packages/ui/src/styles.css`.

- [x] **Step 6: Make the layout responsive**

Use token-driven CSS grids with stable min widths, avoid nested cards, keep text from overflowing, and keep the dense operational feel on desktop and mobile.

## Task 3: Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/design-system/foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Document the frontend surface bilingually**

Mention that the Web shell now mirrors Context, commit, and evaluation run discovery contracts in a design-system-first workspace.

- [x] **Step 2: Document design-system expectations**

Note that the page uses tokens and shared primitives rather than isolated UI styles.

## Task 4: Verification

- [x] **Step 1: Format/check TypeScript**

Run `pnpm --filter @contextlab/web lint`.

- [x] **Step 2: Build Web**

Run `pnpm --filter @contextlab/web build`.

- [x] **Step 3: Run visual workspace QA**

Run `python apps/web/verify-context-workspace.py` against a local `@contextlab/web` dev server.

Expected: PASS with `target/context-workspace-desktop.png` and `target/context-workspace-mobile.png`.

- [x] **Step 4: Run default checks**

Run `cargo test --workspace` and `pnpm test`.

Expected: all checks pass; the Web build should not require a live API server because the preview data is local and API-shaped.
