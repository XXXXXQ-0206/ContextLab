# Web Scorecard Filter Workspace Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Scope the Web workspace scorecard to the selected evaluation run's suite and model so benchmark averages are inspectable without mixing unrelated runs.

**Architecture:** `context-workspace-data.ts` remains the only SDK/live-data boundary and derives scorecard query filters from the first selected evaluation run. `context-workspace-presenter.ts` exposes the filtered scorecard route as deterministic view-model data. `context-workspace-screen.tsx` only renders the prepared model through design-system primitives.

**Tech Stack:** Next.js App Router, TypeScript, `@contextlab/ts-sdk`, node:test, Playwright visual QA, bilingual Markdown docs.

---

### Task 1: Add Failing Web Contract Tests

**Files:**
- Modify: `apps/web/src/app/context-workspace-data.test.ts`
- Modify: `apps/web/src/app/context-workspace-presenter.test.ts`

- [x] **Step 1: Assert live scorecard filters**

Extend the live workspace loader test so the fake API only returns a scorecard when the request includes:

```text
suite_name=Live+Regression&model_version=deepseek-chat
```

Expected before implementation: `data.evaluationScorecard` is `null` because the current loader calls the unfiltered endpoint.

- [x] **Step 2: Assert route query visibility**

Extend the presenter route assertion so the scorecard route is:

```text
/api/v1/contexts/support-resolution-agent/evaluation-scorecard?suite_name=Safety+Regression+Suite&model_version=deepseek-chat
```

Expected before implementation: the current presenter returns the unfiltered scorecard path.

### Task 2: Implement Filtered Scorecard Loading

**Files:**
- Modify: `apps/web/src/app/context-workspace-data.ts`
- Modify: `apps/web/src/app/context-workspace-presenter.ts`

- [x] **Step 1: Move scorecard loading after run selection**

Select the first live evaluation run before calling `getEvaluationScorecard`.

- [x] **Step 2: Pass suite/model filters**

Call:

```ts
client.getEvaluationScorecard(selectedContext.id, {
  suite_name: selectedEvaluationRun.suite_name,
  model_version: selectedEvaluationRun.model_version
})
```

When there is no selected run, return `null` for `evaluationScorecard`.

- [x] **Step 3: Encode the presenter route**

Use `URLSearchParams` in the presenter to append `suite_name` and `model_version` to the scorecard route when a selected evaluation run exists.

### Task 3: Update Bilingual Docs and Visual QA

**Files:**
- Modify: `README.md`
- Modify: `docs/design-system/foundation.md`
- Modify: `docs/roadmap/long-term-roadmap.md`
- Modify: `apps/web/verify-context-workspace.py`

- [x] **Step 1: Update docs**

Document that the Web shell mirrors scorecard aggregates and scopes the displayed scorecard to the selected run's suite/model filter.

- [x] **Step 2: Extend visual QA**

Assert `Evaluation Scorecard`, `aggregated averages`, the filtered scorecard route, `Average`, and `Samples` render without horizontal overflow.

### Task 4: Verify

**Files:**
- No source changes beyond the files above.

- [x] **Step 1: Run targeted Web tests**

```powershell
pnpm --filter @contextlab/web test
```

Expected: PASS after implementation.

- [x] **Step 2: Run Web lint/build**

```powershell
pnpm --filter @contextlab/web lint
pnpm --filter @contextlab/web build
```

Expected: PASS.

### Self-Review

- Spec coverage: This keeps scorecard presentation tied to benchmark-driven development and avoids unscoped aggregates in the workspace.
- Placeholder scan: No TBD/TODO/fill-in steps remain.
- Type consistency: Uses existing `EvaluationScorecardQuery` fields from the TypeScript SDK.
