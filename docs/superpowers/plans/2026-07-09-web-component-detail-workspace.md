# Web Component Detail Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect `GET /api/v1/contexts/{context_id}/components/{component_id}` into the Web Context workspace so users can inspect the selected component's metadata, timestamps, and reproducible fingerprint without implying body-content storage exists.

**Architecture:** `apps/web/src/app/context-workspace-data.ts` remains the only live SDK caller and selects the first listed component before fetching its detail. `apps/web/src/app/context-workspace-preview.ts` owns deterministic API-shaped fixtures, while `apps/web/src/app/page.tsx` stays presentation-only and renders the `ComponentDetail` DTO with token-driven UI.

**Tech Stack:** Next.js App Router, React, TypeScript, `@contextlab/ts-sdk`, `@contextlab/ui`, CSS custom properties, `lucide-react`, Playwright visual QA, bilingual Markdown docs.

---

### Task 1: Extend Web Workspace Data Contracts

**Files:**
- Modify: `apps/web/src/app/context-workspace-preview.ts`
- Modify: `apps/web/src/app/context-workspace-data.ts`

- [x] **Step 1: Import the component detail DTO**

Add `ComponentDetail` to the existing `@contextlab/ts-sdk` type imports in both Web data files.

- [x] **Step 2: Add deterministic component detail fixture**

Export `selectedComponentDetail: ComponentDetail` for `refund-policy`, including `metadata`, `content_hash`, `created_at`, and `updated_at`. Keep it API-shaped and do not add a `content` field.

- [x] **Step 3: Add detail to the workspace aggregate**

Add `selectedComponentDetail: ComponentDetail` to `ContextWorkspaceData` and to `buildWorkspaceData(...)`.

- [x] **Step 4: Fetch live detail through the SDK boundary**

In `loadContextWorkspace()`, select the first component from `liveComponentPreview`, call `client.getComponent(selectedContext.id, selectedComponent.id)`, and pass the returned detail into `buildWorkspaceData("live", ...)`.

### Task 2: Render Component Detail in the Operations Panel

**Files:**
- Modify: `apps/web/src/app/page.tsx`
- Modify: `apps/web/src/app/globals.css`

- [x] **Step 1: Read selected detail from the data boundary**

Destructure `selectedComponentDetail` from `loadContextWorkspace()`.

- [x] **Step 2: Add component detail route visibility**

Add `/api/v1/contexts/${selectedContext.id}/components/${selectedComponentDetail.id}` to the route stack so the detail contract is visible beside list contracts.

- [x] **Step 3: Render a compact detail block**

Inside the Operations panel, render `name`, formatted `kind`, short hash, `updated_at`, and metadata entries from `selectedComponentDetail.metadata`. Include bilingual copy that the current contract exposes metadata, not body content.

- [x] **Step 4: Add resilient styles**

Add `.component-detail`, `.component-detail__meta`, `.metadata-grid`, and related selectors using design tokens, stable grids, and `overflow-wrap: anywhere` for UUIDs/hashes/JSON-like values.

### Task 3: Update Bilingual Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/design-system/foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`

- [x] **Step 1: Update README current foundation**

Mention that `apps/web` now mirrors component list and component detail contracts with API-shaped preview data and optional live API loading.

- [x] **Step 2: Update design-system foundation**

Document the selected component detail block as a dense operational pattern for metadata-first inspection.

- [x] **Step 3: Update roadmap**

Mark Web contract mirroring as including component detail before versioned body/object storage work.

### Task 4: Verify the Increment

**Files:**
- Modify: `apps/web/verify-context-workspace.py`

- [x] **Step 1: Extend visual QA assertions**

Assert the rendered page contains `Component Detail`, the component detail route, `metadata only`, and a metadata field from the fixture.

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

- Spec coverage: This plan advances the context-first Web workspace by exposing selected component metadata through the public SDK contract while preserving the explicit no-body-content boundary.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: `selectedComponentDetail`, `ComponentDetail`, and `client.getComponent(...)` match the SDK and REST operation names.
