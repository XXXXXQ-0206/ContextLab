# Context Graph Inspector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let Web workspace users select a Context Graph node and inspect its direct incoming and outgoing relationships without changing the existing API or SDK graph contract.

**Architecture:** `context-workspace-presenter.ts` remains the only transformation boundary for graph DTO-derived view models and supplies a pure selection helper. A new client-only graph panel owns transient selected-node state and composes the helper output with existing `@contextlab/ui` primitives. `context-workspace-screen.tsx` stays a server-safe composition surface.

**Tech Stack:** Next.js 16, React 19, TypeScript, `@contextlab/ui`, Node test runner through `tsx`, Playwright Python visual verification.

---

## File Structure

- Modify: `apps/web/src/app/context-workspace-presenter.ts` — add graph node kind data, relationship endpoint identifiers, and a pure selected-node inspector helper.
- Modify: `apps/web/src/app/context-workspace-presenter.test.ts` — specify and verify presenter selection behavior for preview, sparse live, and missing node cases.
- Create: `apps/web/src/app/context-graph-inspector.tsx` — client-only interaction adapter for graph node buttons and inspector rendering.
- Modify: `apps/web/src/app/context-workspace-screen.tsx` — replace the page-local static graph panel with the client inspector adapter.
- Modify: `apps/web/src/app/globals.css` — make graph nodes stable accessible buttons, expose selected and focus-visible states, and lay out inspector blocks responsively.
- Modify: `apps/web/verify-context-workspace.py` — assert source ownership, keyboard-accessible node selection, inspector detail, and desktop/mobile no-overflow behavior.

## Task 1: Presenter Selection Contract

**Files:**
- Modify: `apps/web/src/app/context-workspace-presenter.ts`
- Modify: `apps/web/src/app/context-workspace-presenter.test.ts`

- [ ] **Step 1: Write the failing presenter tests**

Append tests that specify graph endpoint identifiers and direct relationship selection:

```ts
test("presenter selects a graph node with direct incoming and outgoing relationships", () => {
  const model = presentContextWorkspaceScreen(createPreviewWorkspaceData());
  const inspector = presentGraphNodeInspector(model.graph, "project:support-ai");

  assert.deepEqual(inspector.selectedNode, {
    id: "project:support-ai",
    kindLabel: "Project",
    label: "Project",
    detail: "Support AI Project"
  });
  assert.deepEqual(inspector.incomingRelationships.map((item) => item.relationshipLabel), ["Owns"]);
  assert.deepEqual(inspector.outgoingRelationships.map((item) => item.relationshipLabel), ["Owns"]);
});

test("presenter returns a stable empty inspector for an unknown node", () => {
  const model = presentContextWorkspaceScreen(createPreviewWorkspaceData());

  assert.deepEqual(presentGraphNodeInspector(model.graph, "missing:node"), {
    selectedNode: null,
    incomingRelationships: [],
    outgoingRelationships: []
  });
});

test("presenter preserves the evaluation relationship direction", () => {
  const model = presentContextWorkspaceScreen(createPreviewWorkspaceData());
  const inspector = presentGraphNodeInspector(model.graph, "evaluation:safety-regression");

  assert.equal(inspector.selectedNode?.kindLabel, "Evaluation");
  assert.deepEqual(inspector.incomingRelationships, []);
  assert.deepEqual(inspector.outgoingRelationships.map((item) => item.relationshipLabel), ["Evaluates"]);
});
```

- [ ] **Step 2: Run the focused tests and verify they fail**

Run: `pnpm --filter @contextlab/web test -- context-workspace-presenter.test.ts`

Expected: FAIL because `presentGraphNodeInspector` and relationship endpoint identifiers do not yet exist.

- [ ] **Step 3: Implement the smallest presenter contract**

Extend the graph types and relationship mapping without exposing raw SDK DTOs to the screen:

```ts
export type GraphRelationshipItem = {
  id: string;
  sourceId: string;
  sourceKind: string;
  sourceLabel: string;
  relationshipLabel: string;
  targetId: string;
  targetKind: string;
  targetLabel: string;
};

export function presentGraphNodeInspector(graph: ContextGraphViewModel, nodeId: string | null) {
  const selectedNode = graph.nodes.find((node) => node.id === nodeId) ?? null;

  return {
    selectedNode,
    incomingRelationships: selectedNode
      ? graph.relationships.filter((relationship) => relationship.targetId === selectedNode.id)
      : [],
    outgoingRelationships: selectedNode
      ? graph.relationships.filter((relationship) => relationship.sourceId === selectedNode.id)
      : []
  };
}
```

Expose `ContextGraphViewModel`, give each `GraphNode` a `kindLabel` while retaining the existing `label` and `detail` meanings, and populate `sourceId` and `targetId` from the same graph edges that already produce relationship rows.

- [ ] **Step 4: Run the focused tests and verify they pass**

Run: `pnpm --filter @contextlab/web test -- context-workspace-presenter.test.ts`

Expected: PASS, including the existing preview and sparse-live graph cases.

- [ ] **Step 5: Review the presenter boundary**

Confirm `context-workspace-presenter.ts` imports no React or `@contextlab/ui`, and that no API, SDK, or storage file changes are present.

## Task 2: Client Graph Inspector Adapter

**Files:**
- Create: `apps/web/src/app/context-graph-inspector.tsx`
- Modify: `apps/web/src/app/context-workspace-screen.tsx`
- Modify: `apps/web/src/app/globals.css`

- [ ] **Step 1: Write the failing visual-verifier source assertions**

In `inspect_source_boundaries`, require the client adapter and forbid page-local relationship filtering:

```python
graph_inspector_source = (APP_DIR / "context-graph-inspector.tsx").read_text(encoding="utf-8")

assert '"use client"' in graph_inspector_source
assert "presentGraphNodeInspector" in graph_inspector_source
assert "ContextGraphInspector" in screen_source
assert ".filter((relationship)" not in screen_source
```

Run: `python apps/web/verify-context-workspace.py`

Expected: FAIL with `FileNotFoundError` because the client adapter does not yet exist.

- [ ] **Step 2: Create the client adapter and connect it to the screen**

Create a `"use client"` component that receives `ContextGraphViewModel`, initializes selection with `graph.nodes[0]?.id ?? null`, and calls `presentGraphNodeInspector(graph, selectedNodeId)` for display data. Render each graph node as a `button` using this shape:

```tsx
<button
  aria-label={`Select ${node.detail} (${node.kindLabel})`}
  aria-pressed={node.id === selectedNodeId}
  className="graph-node"
  data-root={node.root}
  data-selected={node.id === selectedNodeId}
  key={node.id}
  onClick={() => setSelectedNodeId(node.id)}
  style={{ "--x": node.x, "--y": node.y } as CSSProperties}
  type="button"
>
  <strong>{node.kindLabel}</strong>
  <span>{node.detail}</span>
</button>
```

Use `DefinitionGrid` for node identifier, type, and label. Render separate `StackTable` blocks headed `Incoming Relationships / 入边` and `Outgoing Relationships / 出边`, each with the existing source, relationship, and target cells. Retain the complete `Graph Relationships / 图谱关系` table below the inspector. Replace `ContextGraphPanel` in `context-workspace-screen.tsx` with a direct import and composition of `ContextGraphInspector`.

Keep the visual node layout bounded by `graphNodePositions`, but expose every `graph.inspectorNodes` item through a native labeled `select`. Both the visual buttons and the selector update the same local selected-node identifier, so relationships that refer to nodes outside the visual layout remain inspectable without changing the graph API contract.

- [ ] **Step 3: Add the selected and keyboard-focus styles**

Convert `.graph-node` from static-card styling to stable button styling while preserving mobile grid layout:

```css
.graph-node {
  appearance: none;
  color: inherit;
  cursor: pointer;
  font: inherit;
  text-align: left;
}

.graph-node[data-selected="true"] {
  box-shadow: 0 0 0 2px var(--cl-color-accent), var(--cl-shadow-sm);
}

.graph-node:focus-visible {
  outline: 2px solid var(--cl-color-accent);
  outline-offset: 3px;
}
```

Add a token-driven `.graph-inspector` grid with responsive one-column behavior under the existing `760px` breakpoint. Keep no fixed height that can clip relationship tables.

- [ ] **Step 4: Run lint and unit tests**

Run: `pnpm --filter @contextlab/web lint && pnpm --filter @contextlab/web test`

Expected: PASS with no TypeScript errors and all presenter tests green.

## Task 3: Browser Interaction Verification

**Files:**
- Modify: `apps/web/verify-context-workspace.py`

- [ ] **Step 1: Write failing interaction assertions**

After the static text checks in `inspect_viewport`, add a role-based selection assertion:

```python
project_node = page.get_by_role("button", name="Select Support AI Project (Project)")
assert project_node.get_attribute("aria-pressed") == "false"
project_node.focus()
page.keyboard.press("Enter")
assert project_node.get_attribute("aria-pressed") == "true"
assert page.get_by_text("Selected Node / 已选节点", exact=False).count() > 0
assert page.get_by_text("Incoming Relationships / 入边", exact=False).count() > 0
assert page.get_by_text("Outgoing Relationships / 出边", exact=False).count() > 0
```

Run: `python apps/web/verify-context-workspace.py`

Expected: FAIL before the adapter exists because the graph node is not a button.

- [ ] **Step 2: Run the browser verifier against the implemented workspace**

Run the web development server in one terminal:

```powershell
pnpm --filter @contextlab/web dev
```

Then run: `python apps/web/verify-context-workspace.py`

Expected: PASS; desktop and mobile screenshots are written to `target/context-workspace-desktop.png` and `target/context-workspace-mobile.png`, both with no horizontal overflow or browser console errors.

- [ ] **Step 3: Run the full Web release gate**

Run: `pnpm check:web`

Expected: PASS for SDK lint/test and Web lint/test/build.

- [ ] **Step 4: Record the result without overstating project completion**

Update `docs/design-system/foundation.md` to describe the graph inspector as a read-only node selection pattern. Update `docs/roadmap/completion-criteria.md` to state that graph detail selection is complete while graph diffing and editing remain open.

## Plan Self-Review

- The plan implements every approved requirement: node selection, inspector details, direct relationship partitions, keyboard state, empty graph behavior, responsive presentation, and the existing graph-wide relationship table.
- The plan deliberately excludes graph writes, graph diffing, API and SDK changes, scorecard policy, and canvas migration.
- The only new public symbols are `ContextGraphViewModel` and `presentGraphNodeInspector`; both are defined before the client adapter consumes them.
- The plan contains no placeholder implementation steps and does not require a Git commit because this workspace currently lacks a readable Git repository.
