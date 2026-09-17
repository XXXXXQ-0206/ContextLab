# OpenAPI Contract Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first OpenAPI contract for the current REST API and verify that the TypeScript SDK discovery client stays aligned with it.

**Architecture:** Keep the OpenAPI document as a checked-in contract artifact under `docs/api/` for this early phase. Add SDK-side contract tests that parse the document and verify operation IDs, paths, query parameters, and client method coverage without introducing a heavyweight generator before the API stabilizes further.

**Tech Stack:** OpenAPI 3.1 JSON, TypeScript, Node test runner via `tsx`, Axum REST API, pnpm workspaces.

---

## Task 1: OpenAPI Contract

**Files:**
- Create: `docs/api/openapi.json`

- [x] **Step 1: Document platform routes**

Add OpenAPI paths for `GET /healthz`, `GET /api/v1/meta`, `GET /api/v1/providers`, `GET /api/v1/context-graph/preview`, and `GET /api/v1/workspaces/{workspace_id}/context-graph`.

- [x] **Step 2: Document discovery routes**

Add OpenAPI paths for `GET /api/v1/workspaces`, `GET /api/v1/workspaces/{workspace_id}/projects`, `GET /api/v1/projects/{project_id}/experiments`, `GET /api/v1/projects/{project_id}/contexts`, `GET /api/v1/contexts/{context_id}/commits`, and `GET /api/v1/contexts/{context_id}/evaluation-runs`.

- [x] **Step 3: Document shared schemas**

Add schemas for list pagination, workspace/project/experiment/context/commit/evaluation-run list responses, structured error responses, provider status, meta response, and a deliberately minimal context graph response.

## Task 2: SDK Contract Tests

**Files:**
- Create: `packages/ts-sdk/src/openapi-contract.test.ts`

- [x] **Step 1: Parse the OpenAPI document**

Read `../../docs/api/openapi.json` from the SDK package test process and assert `openapi` starts with `3.`.

- [x] **Step 2: Verify SDK operation coverage**

Assert that all six SDK client methods have matching `operationId`s and paths in OpenAPI: `listWorkspaces`, `listProjects`, `listExperiments`, `listContexts`, `listCommits`, and `listEvaluationRuns`.

- [x] **Step 3: Verify query parameter coverage**

Assert common list query parameters `page`, `per_page`, `search`, and `sort`; also assert route-specific filters `experiment_id`, `branch_name`, `suite_name`, and `model_version`.

## Task 3: Documentation

**Files:**
- Create: `docs/api/rest-api.md`
- Modify: `README.md`
- Modify: `docs/sdk/typescript-sdk.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `docs/superpowers/plans/2026-07-09-openapi-contract-foundation.md`

- [x] **Step 1: Add bilingual REST API documentation**

Explain where the OpenAPI contract lives, which routes it covers, and why it is checked in before generated SDKs.

- [x] **Step 2: Link OpenAPI from README and SDK docs**

Mention `docs/api/openapi.json` in the verification and SDK sections.

- [x] **Step 3: Update the roadmap**

Record that OpenAPI is now the bridge between Axum routes and future generated SDKs.

## Task 4: Verification

- [x] **Step 1: Run SDK tests**

Run `pnpm --filter @contextlab/ts-sdk test`.

Expected: PASS, including the new OpenAPI contract coverage tests.

- [x] **Step 2: Run SDK and Web type checks**

Run `pnpm --filter @contextlab/ts-sdk lint`, `pnpm --filter @contextlab/web lint`, and `pnpm --filter @contextlab/web build`.

Expected: PASS.

- [x] **Step 3: Run full workspace checks**

Run `pnpm check` and `pnpm test`.

Expected: PASS; PostgreSQL opt-in integration tests may remain ignored unless `CONTEXTLAB_TEST_DATABASE_URL` is provided.
