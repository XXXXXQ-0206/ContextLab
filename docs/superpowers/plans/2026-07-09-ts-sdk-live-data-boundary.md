# TypeScript SDK Live Data Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first `@contextlab/ts-sdk` package for REST discovery contracts and move the Web workspace toward optional live data without making builds depend on a running API.

**Architecture:** The SDK owns transport-neutral REST DTOs, query types, route construction, and a small fetch-based client. The Web app imports SDK types and uses a server-side data boundary that can read from a live API when explicitly configured, while preserving deterministic preview data as the default build-safe path.

**Tech Stack:** TypeScript, pnpm workspaces, Next.js App Router, native `fetch`, Node test runner through `tsx`.

---

## Task 1: SDK Package Foundation

**Files:**
- Create: `packages/ts-sdk/package.json`
- Create: `packages/ts-sdk/tsconfig.json`
- Create: `packages/ts-sdk/src/index.ts`
- Create: `packages/ts-sdk/src/types.ts`
- Create: `packages/ts-sdk/src/client.ts`
- Create: `packages/ts-sdk/src/client.test.ts`
- Modify: `tsconfig.base.json`
- Modify: `apps/web/next.config.ts`

- [x] **Step 1: Create the workspace package manifest**

Add `@contextlab/ts-sdk` with ESM exports from `./src/index.ts`, a `lint` script using `tsc --noEmit`, and a `test` script using `tsx --test src/*.test.ts`.

- [x] **Step 2: Add SDK TypeScript config**

Extend the root TypeScript config and include `src/**/*.ts` so the package can be checked independently.

- [x] **Step 3: Add REST DTOs and query types**

Define `ListPagination`, `ListResponse<T>`, `WorkspaceItem`, `ProjectItem`, `ExperimentItem`, `ContextItem`, `CommitItem`, `EvaluationRunItem`, and list query option types matching the Axum route contracts.

- [x] **Step 4: Add fetch client**

Implement `ContextLabClient` with `listWorkspaces`, `listProjects`, `listExperiments`, `listContexts`, `listCommits`, and `listEvaluationRuns`. Encode query parameters with `URLSearchParams`, remove trailing slashes from `baseUrl`, and throw `ContextLabApiError` for non-2xx JSON error bodies.

- [x] **Step 5: Add SDK tests**

Test route construction with query parameters, trailing-slash base URL normalization, and structured API error handling using an injected mock fetch.

- [x] **Step 6: Register package paths**

Add `@contextlab/ts-sdk` to the root `paths` map and Next `transpilePackages`.

## Task 2: Web Data Boundary

**Files:**
- Create: `apps/web/src/app/context-workspace-data.ts`
- Modify: `apps/web/src/app/context-workspace-preview.ts`
- Modify: `apps/web/src/app/page.tsx`

- [x] **Step 1: Replace page-local DTO ownership**

Move the preview module to import `ListResponse`, `WorkspaceItem`, `ProjectItem`, `ContextItem`, `CommitItem`, and `EvaluationRunItem` from `@contextlab/ts-sdk` instead of defining duplicate API types.

- [x] **Step 2: Add workspace data loader**

Create `loadContextWorkspace()` that returns selected preview data by default. When `CONTEXTLAB_WEB_API_BASE_URL` is set, instantiate `ContextLabClient` and request the same discovery resources. On fetch failure, return preview data plus a source value of `preview-fallback`.

- [x] **Step 3: Keep the page presentation-only**

Make `page.tsx` consume `loadContextWorkspace()` and render source status without embedding fetch calls inside JSX rendering details.

## Task 3: Documentation

**Files:**
- Modify: `README.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Create: `docs/sdk/typescript-sdk.md`
- Modify: `docs/superpowers/plans/2026-07-09-ts-sdk-live-data-boundary.md`

- [x] **Step 1: Document SDK usage bilingually**

Explain `@contextlab/ts-sdk`, `ContextLabClient`, and `CONTEXTLAB_WEB_API_BASE_URL` in English and Chinese.

- [x] **Step 2: Update roadmap**

Mark the TypeScript SDK/live data boundary as the bridge between API-shaped preview data and generated SDKs or full OpenAPI later.

- [x] **Step 3: Track completed plan items**

Mark each checkbox as work finishes and record the verification commands used.

## Task 4: Verification

- [x] **Step 1: Run SDK tests**

Run: `pnpm --filter @contextlab/ts-sdk test`

Expected: PASS with route construction and error handling tests.

- [x] **Step 2: Run SDK typecheck**

Run: `pnpm --filter @contextlab/ts-sdk lint`

Expected: PASS with no TypeScript errors.

- [x] **Step 3: Run Web checks**

Run: `pnpm --filter @contextlab/web lint` and `pnpm --filter @contextlab/web build`

Expected: PASS without a live API server. Also run a build with `CONTEXTLAB_WEB_API_BASE_URL=http://127.0.0.1:9` to prove the dynamic page does not require API availability during build.

- [x] **Step 4: Run Rust and workspace tests**

Run: `cargo fmt --all -- --check`, `cargo test --workspace`, and `pnpm test`

Expected: PASS; PostgreSQL opt-in integration tests may remain ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is provided.

- [x] **Step 5: Run visual workspace QA**

Run `python apps/web/verify-context-workspace.py` against a local `@contextlab/web` dev server.

Expected: PASS with refreshed desktop and mobile screenshots in `target/`.
