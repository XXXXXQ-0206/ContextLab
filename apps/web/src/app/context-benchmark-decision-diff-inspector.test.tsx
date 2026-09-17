import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  ContextBenchmarkDecisionDiffInspectorControl,
  canEditBenchmarkDecisionDiffScope
} from "./context-benchmark-decision-diff-inspector";

const originalFetch = globalThis.fetch;

test("benchmark decision diff inspector renders a bilingual local-only comparison form", () => {
  const markup = renderToStaticMarkup(
    <ContextBenchmarkDecisionDiffInspectorControl
      candidates={[{ id: "commit-a", label: "Materialized head" }]}
      contextId="context-a"
      projectId="project-a"
    />
  );

  assert.match(markup, /Benchmark Decision Diff/);
  assert.match(markup, /只读本地受保护比较/);
  assert.match(markup, /Baseline decision \/ 基线决策/);
  assert.match(markup, /Revised decision \/ 修订决策/);
  assert.match(markup, /No sealed decisions \/ 暂无 sealed decision/);
  assert.match(markup, /Discover exact decisions \/ 发现精确 decision/);
  assert.match(markup, /No comparison selected/);
  assert.match(markup, /type="password"/);
});

test("benchmark decision diff inspector locks scopes while a read is pending", () => {
  assert.equal(canEditBenchmarkDecisionDiffScope(false), true);
  assert.equal(canEditBenchmarkDecisionDiffScope(true), false);
});

test("discovered baseline and revised decision lists remain bound to their exact commits", async () => {
  const requests: string[] = [];
  globalThis.fetch = createFetchStub(requests);

  try {
    const harness = createInspectorHarness();
    change(harness, "benchmark-diff-revised-commit", "commit-revised");
    await discover(harness);

    assert.deepEqual(discoveryRequests(requests), [
      discoveryPath("commit-base"),
      discoveryPath("commit-revised")
    ]);
    assert.deepEqual(optionValues(harness.tree, "benchmark-diff-baseline-decision"), [
      "commit-base/decision-a",
      "commit-base/decision-b"
    ]);
    assert.deepEqual(optionValues(harness.tree, "benchmark-diff-revised-decision"), [
      "commit-revised/decision-a",
      "commit-revised/decision-b"
    ]);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("changing one commit clears stale selections and comparison before rediscovery completes", async () => {
  const requests: string[] = [];
  const deferredDiscoveries: DeferredDiscovery[] = [];
  let deferDiscovery = false;
  globalThis.fetch = createFetchStub(requests, (input) => {
    if (!deferDiscovery || !isDiscoveryRequest(input)) {
      return null;
    }

    return new Promise<Response>((resolve) => {
      deferredDiscoveries.push({ input, resolve });
    });
  });

  try {
    const harness = createInspectorHarness();
    change(harness, "benchmark-diff-revised-commit", "commit-revised");
    await discover(harness);
    await compare(harness);
    assert.ok(findByClassName(harness.tree, "context-benchmark-evidence__result"));

    deferDiscovery = true;
    change(harness, "benchmark-diff-baseline-commit", "commit-next");

    assert.equal(deferredDiscoveries.length, 2);
    assert.deepEqual(optionValues(harness.tree, "benchmark-diff-baseline-decision"), [""]);
    assert.deepEqual(optionValues(harness.tree, "benchmark-diff-revised-decision"), [""]);
    assert.equal(findByClassName(harness.tree, "context-benchmark-evidence__result"), undefined);
    assert.match(textContent(harness.tree), /No comparison selected/);

    for (const pending of deferredDiscoveries) {
      pending.resolve(jsonResponse(discoveryPayload(commitFromDiscoveryPath(pending.input))));
    }
    await flushAsyncWork();
    harness.render();

    assert.deepEqual(optionValues(harness.tree, "benchmark-diff-baseline-decision"), [
      "commit-next/decision-a",
      "commit-next/decision-b"
    ]);
    assert.deepEqual(optionValues(harness.tree, "benchmark-diff-revised-decision"), [
      "commit-revised/decision-a",
      "commit-revised/decision-b"
    ]);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("inspect requests use complete scopes selected from discovered commit lists", async () => {
  const requests: string[] = [];
  globalThis.fetch = createFetchStub(requests);

  try {
    const harness = createInspectorHarness();
    change(harness, "benchmark-diff-revised-commit", "commit-revised");
    await discover(harness);

    const baselineSelect = findByName(harness.tree, "benchmark-diff-baseline-decision");
    const revisedSelect = findByName(harness.tree, "benchmark-diff-revised-decision");
    assert.equal(baselineSelect.type, "select");
    assert.equal(revisedSelect.type, "select");
    change(harness, "benchmark-diff-baseline-decision", "commit-base/decision-b");
    change(harness, "benchmark-diff-revised-decision", "commit-revised/decision-b");
    await compare(harness);

    const inspectRequest = requests.find((request) => request.includes("benchmark-decision-diffs?"));
    assert.ok(inspectRequest);
    const query = new URL(inspectRequest, "http://contextlab.test").searchParams;
    assert.deepEqual(Object.fromEntries(query), {
      baseline_commit_id: "commit-base",
      baseline_decision_id: "commit-base/decision-b",
      revised_commit_id: "commit-revised",
      revised_decision_id: "commit-revised/decision-b"
    });
    assert.ok(optionValues(harness.tree, "benchmark-diff-baseline-decision").includes(query.get("baseline_decision_id") ?? ""));
    assert.ok(optionValues(harness.tree, "benchmark-diff-revised-decision").includes(query.get("revised_decision_id") ?? ""));
  } finally {
    globalThis.fetch = originalFetch;
  }
});

type InspectorTree = React.ReactElement<Record<string, unknown>>;
type HookSlot =
  | { kind: "state"; value: unknown }
  | { kind: "ref"; value: { current: unknown } };
type DeferredDiscovery = { input: string; resolve: (response: Response) => void };

function createInspectorHarness() {
  const slots: HookSlot[] = [];
  let cursor = 0;
  let tree: InspectorTree;
  const internals = React as unknown as {
    __CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE: { H: unknown };
  };
  const dispatcher = {
    useState<T>(initialValue: T | (() => T)): [T, React.Dispatch<React.SetStateAction<T>>] {
      const index = cursor;
      cursor += 1;
      if (!slots[index]) {
        slots[index] = {
          kind: "state",
          value: typeof initialValue === "function" ? (initialValue as () => T)() : initialValue
        };
      }
      const slot = slots[index];
      assert.equal(slot?.kind, "state");
      const setValue: React.Dispatch<React.SetStateAction<T>> = (nextValue) => {
        assert.equal(slot?.kind, "state");
        slot.value = typeof nextValue === "function"
          ? (nextValue as (previous: T) => T)(slot.value as T)
          : nextValue;
      };
      return [slot.value as T, setValue];
    },
    useRef<T>(initialValue: T): React.RefObject<T> {
      const index = cursor;
      cursor += 1;
      if (!slots[index]) {
        slots[index] = { kind: "ref", value: { current: initialValue } };
      }
      const slot = slots[index];
      assert.equal(slot?.kind, "ref");
      return slot.value as React.RefObject<T>;
    }
  };
  const harness = {
    get tree() {
      return tree;
    },
    render() {
      cursor = 0;
      const previousDispatcher = internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H;
      internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H = dispatcher;
      try {
        tree = ContextBenchmarkDecisionDiffInspectorControl({
          candidates: [
            { id: "commit-base", label: "Baseline" },
            { id: "commit-revised", label: "Revised" },
            { id: "commit-next", label: "Next baseline" }
          ],
          contextId: "context/id",
          projectId: "project/id"
        }) as InspectorTree;
      } finally {
        internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H = previousDispatcher;
      }
      return tree;
    }
  };
  harness.render();
  return harness;
}

async function discover(harness: ReturnType<typeof createInspectorHarness>) {
  change(harness, "benchmark-diff-bearer-token", "request-token");
  const tokenInput = findByName(harness.tree, "benchmark-diff-bearer-token");
  invoke(tokenInput, "onBlur");
  await flushAsyncWork();
  harness.render();
}

async function compare(harness: ReturnType<typeof createInspectorHarness>) {
  const button = findElement(
    harness.tree,
    (element) =>
      typeof element.props.onClick === "function"
      && textContent(element.props.children as React.ReactNode).includes("Compare decisions")
  );
  assert.ok(button);
  assert.notEqual(button.props.disabled, true);
  invoke(button, "onClick");
  await flushAsyncWork();
  harness.render();
}

function change(
  harness: ReturnType<typeof createInspectorHarness>,
  name: string,
  value: string
) {
  const element = findByName(harness.tree, name);
  invoke(element, "onChange", { target: { value } });
  harness.render();
}

function invoke(element: InspectorTree, handler: string, event?: unknown) {
  const callback = element.props[handler];
  assert.equal(typeof callback, "function", `${handler} must be executable`);
  (callback as (event?: unknown) => void)(event);
}

function findByName(tree: InspectorTree, name: string): InspectorTree {
  const element = findElement(tree, (candidate) => candidate.props.name === name);
  assert.ok(element, `expected control ${name}`);
  return element;
}

function findByClassName(tree: InspectorTree, className: string): InspectorTree | undefined {
  return findElement(tree, (candidate) => candidate.props.className === className);
}

function findElement(
  node: React.ReactNode,
  predicate: (element: InspectorTree) => boolean
): InspectorTree | undefined {
  if (!React.isValidElement(node)) {
    return undefined;
  }
  const element = node as InspectorTree;
  if (predicate(element)) {
    return element;
  }
  for (const child of React.Children.toArray(element.props.children as React.ReactNode)) {
    const match = findElement(child, predicate);
    if (match) {
      return match;
    }
  }
  return undefined;
}

function optionValues(tree: InspectorTree, name: string): string[] {
  const select = findByName(tree, name);
  return React.Children.toArray(select.props.children as React.ReactNode)
    .filter(React.isValidElement)
    .map((option) => String((option as InspectorTree).props.value));
}

function textContent(node: React.ReactNode): string {
  if (typeof node === "string" || typeof node === "number") {
    return String(node);
  }
  if (!React.isValidElement(node)) {
    return React.Children.toArray(node).map(textContent).join(" ");
  }
  return textContent((node as InspectorTree).props.children as React.ReactNode);
}

function createFetchStub(
  requests: string[],
  intercept: (input: string) => Promise<Response> | null = () => null
): typeof fetch {
  return (async (input: string | URL | Request) => {
    const request = String(input);
    requests.push(request);
    const intercepted = intercept(request);
    if (intercepted) {
      return intercepted;
    }
    if (isDiscoveryRequest(request)) {
      return jsonResponse(discoveryPayload(commitFromDiscoveryPath(request)));
    }

    const url = new URL(request, "http://contextlab.test");
    return jsonResponse({
      project_id: "project/id",
      context_id: "context/id",
      baseline: {
        commit_id: url.searchParams.get("baseline_commit_id"),
        decision_id: url.searchParams.get("baseline_decision_id")
      },
      revised: {
        commit_id: url.searchParams.get("revised_commit_id"),
        decision_id: url.searchParams.get("revised_decision_id")
      },
      status_change: null,
      metric_changes: []
    });
  }) as typeof fetch;
}

function discoveryPayload(commitId: string) {
  return {
    schema_version: "contextlab.local-benchmark-decision-discovery.v1",
    project_id: "project/id",
    context_id: "context/id",
    commit_id: commitId,
    decisions: [
      decision(`${commitId}/decision-a`, "2026-07-22T00:02:00Z"),
      decision(`${commitId}/decision-b`, "2026-07-22T00:01:00Z")
    ]
  };
}

function decision(decisionId: string, recordedAt: string) {
  return {
    decision_id: decisionId,
    suite_id: "suite/quality",
    status: "passed",
    recorded_at: recordedAt
  };
}

function isDiscoveryRequest(input: string): boolean {
  return input.includes("/commits/") && input.endsWith("/benchmark-decisions");
}

function discoveryRequests(requests: string[]): string[] {
  return requests.filter(isDiscoveryRequest);
}

function discoveryPath(commitId: string): string {
  return `/api/local/projects/project%2Fid/contexts/context%2Fid/commits/${encodeURIComponent(commitId)}/benchmark-decisions`;
}

function commitFromDiscoveryPath(input: string): string {
  const match = input.match(/\/commits\/([^/]+)\/benchmark-decisions$/);
  assert.ok(match?.[1], `expected commit-scoped discovery path: ${input}`);
  return decodeURIComponent(match[1]);
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" }
  });
}

async function flushAsyncWork(): Promise<void> {
  await new Promise<void>((resolve) => setImmediate(resolve));
  await new Promise<void>((resolve) => setImmediate(resolve));
}
