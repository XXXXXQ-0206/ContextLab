import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  getLocalBenchmarkWorkspaceControlState,
  LocalBenchmarkWorkspaceInspector
} from "./local-benchmark-workspace-inspector";

const originalFetch = globalThis.fetch;

test("requires an exact revised scope and an optional complete baseline pair", () => {
  const base = {
    bearerToken: "token",
    projectId: "project",
    contextId: "context",
    revisedCommitId: "commit",
    revisedDecisionId: "decision",
    isLoading: false
  };

  assert.equal(
    getLocalBenchmarkWorkspaceControlState({
      ...base,
      baselineCommitId: "",
      baselineDecisionId: ""
    }).canLoad,
    true
  );
  assert.equal(
    getLocalBenchmarkWorkspaceControlState({
      ...base,
      baselineCommitId: "baseline",
      baselineDecisionId: "baseline-decision"
    }).canLoad,
    true
  );
  assert.equal(
    getLocalBenchmarkWorkspaceControlState({
      ...base,
      baselineCommitId: "baseline",
      baselineDecisionId: ""
    }).canLoad,
    false
  );
  assert.equal(
    getLocalBenchmarkWorkspaceControlState({
      ...base,
      baselineCommitId: "",
      baselineDecisionId: "baseline-decision"
    }).canLoad,
    false
  );
});

test("fails closed when a decision is not in the current exact discovery list", () => {
  assert.equal(
    getLocalBenchmarkWorkspaceControlState({
      bearerToken: "token",
      projectId: "project",
      contextId: "context",
      revisedCommitId: "commit",
      revisedDecisionId: "stale-decision",
      baselineCommitId: "",
      baselineDecisionId: "",
      revisedDecisionOptions: ["current-decision"],
      baselineDecisionOptions: [],
      isLoading: false
    }).canLoad,
    false
  );
});

test("disables every scope control and request action while a read is pending", () => {
  const state = getLocalBenchmarkWorkspaceControlState({
    bearerToken: "token",
    projectId: "project",
    contextId: "context",
    revisedCommitId: "commit",
    revisedDecisionId: "decision",
    baselineCommitId: "baseline",
    baselineDecisionId: "baseline-decision",
    isLoading: true
  });

  assert.equal(state.controlsDisabled, true);
  assert.equal(state.baselineDecisionDisabled, true);
  assert.equal(state.canLoad, false);
});

test("starts empty for a selectable commit and keeps both existing decision inspectors mounted", () => {
  const markup = renderToStaticMarkup(
    <LocalBenchmarkWorkspaceInspector
      candidates={[
        { id: "commit-a", label: "Baseline commit" },
        { id: "commit-b", label: "Revised commit" }
      ]}
      contextId="context-a"
      projectId="project-a"
    />
  );

  assert.match(markup, /data-state="empty"/);
  assert.match(markup, /No server projection loaded/);
  assert.match(markup, /name="benchmark-workspace-bearer-token"/);
  assert.match(markup, /name="benchmark-workspace-revised-commit"/);
  assert.match(markup, /name="benchmark-workspace-revised-decision"/);
  assert.match(markup, /name="benchmark-workspace-baseline-commit"/);
  assert.match(markup, /name="benchmark-workspace-baseline-decision"/);
  assert.match(markup, /Load exact workspace \/ 加载精确工作台/);
  assert.match(markup, /disabled=""/);
  assert.match(markup, /Benchmark Evidence Inspection/);
  assert.match(markup, /Benchmark Decision Diff/);
  assert.doesNotMatch(markup, /evaluationScorecard/);
});

test("starts unavailable and disables exact-scope controls without a materialized commit", () => {
  const markup = renderToStaticMarkup(
    <LocalBenchmarkWorkspaceInspector
      candidates={[]}
      contextId="context-empty"
      projectId="project-empty"
    />
  );

  assert.match(markup, /data-state="unavailable"/);
  assert.match(markup, /No materialized commits/);
  assert.match(markup, /aria-live="polite"/);
  assert.match(markup, /disabled=""/);
});

test("loads the exact workspace through the inspector lifecycle and returns to an available projection", async () => {
  let resolveResponse: ((response: Response) => void) | undefined;
  let request: { input: string | URL | Request; init?: RequestInit } | undefined;
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    const requestUrl = new URL(String(input), "http://contextlab.test");
    if (requestUrl.pathname.endsWith("/benchmark-decisions")) {
      const isBaseline = requestUrl.pathname.includes("/commits/commit-baseline/");
      return jsonResponse(
        benchmarkDecisionDiscoveryPayload(
          isBaseline ? "commit-baseline" : "commit-revised",
          isBaseline ? "decision-baseline" : "decision-revised"
        )
      );
    }
    request = { input, init };
    return new Promise<Response>((resolve) => {
      resolveResponse = resolve;
    });
  }) as typeof fetch;

  try {
    const harness = createWorkspaceInspectorHarness();
    change(harness, "benchmark-workspace-bearer-token", "request-token");
    blur(harness, "benchmark-workspace-bearer-token");
    await flushAsyncWork();
    harness.render();
    change(harness, "benchmark-workspace-baseline-commit", "commit-baseline");
    await flushAsyncWork();
    harness.render();
    submit(harness);

    assert.match(textContent(harness.tree), /Loading exact workspace/);
    assert.match(textContent(harness.tree), /正在加载精确工作台/);
    const loadingState = findElement(
      harness.tree,
      (element) => element.props["data-state"] === "loading"
    );
    assert.ok(loadingState);
    assert.equal(loadingState.props.role, "status");
    assert.equal(loadingState.props["aria-live"], "polite");
    assert.equal(findByName(harness.tree, "benchmark-workspace-revised-decision").props.disabled, true);

    assert.ok(resolveResponse);
    resolveResponse(jsonResponse(benchmarkWorkspacePayload()));
    await flushAsyncWork();
    harness.render();

    assert.ok(request);
    const requestUrl = new URL(String(request.input), "http://contextlab.test");
    assert.equal(
      requestUrl.pathname,
      "/api/local/projects/project-a/contexts/context-a/commits/commit-revised/benchmark-decisions/decision-revised/workspace"
    );
    assert.equal(
      requestUrl.search,
      "?baseline_commit_id=commit-baseline&baseline_decision_id=decision-baseline"
    );
    assert.equal(new Headers(request.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(request.init?.credentials, "omit");
    assert.equal(request.init?.cache, "no-store");
    const rendered = textContent(harness.tree);
    assert.match(rendered, /project-a/);
    assert.match(rendered, /context-a/);
    assert.match(rendered, /commit-revised/);
    assert.match(rendered, /decision-revised/);
    assert.match(rendered, /commit-baseline/);
    assert.match(rendered, /decision-baseline/);
    assert.match(rendered, /dataset-a/);
    assert.match(rendered, /Support cases/);
    assert.match(rendered, /dataset-b/);
    assert.match(rendered, /Billing cases/);
    assert.match(rendered, /case-a1/);
    assert.match(rendered, /case-a2/);
    assert.match(rendered, /case-b1/);
    assert.match(rendered, /case-b2/);
    assert.ok(rendered.indexOf("dataset-a") < rendered.indexOf("dataset-b"));
    assert.ok(rendered.indexOf("case-a1") < rendered.indexOf("case-a2"));
    assert.ok(rendered.indexOf("case-a2") < rendered.indexOf("case-b1"));
    assert.ok(rendered.indexOf("case-b1") < rendered.indexOf("case-b2"));
    assert.match(rendered, /Server scorecard \/ 服务端记分卡/);
    assert.match(rendered, /4 runs \/ 4 次运行/);
    assert.match(rendered, /4 \/ 4/);
    assert.match(rendered, /Complete \/ 完整/);
    assert.match(rendered, /Regression status \/ 回归状态/);
    assert.match(rendered, /Regressed \/ 回归/);
    assert.match(rendered, /Evaluation diff \/ 评测差异/);
    assert.match(rendered, /cohort-baseline/);
    assert.match(rendered, /cohort-revised/);
    assert.match(rendered, /Passed \/ 通过 -> Regressed \/ 回归/);
    assert.match(rendered, /Modified \/ 已修改/);
    assert.match(rendered, /observed 700/);
    assert.match(rendered, /observed 900/);
    assert.match(rendered, /observed 0\.95/);
    assert.match(rendered, /observed 0\.75/);
    const availableState = findElement(
      harness.tree,
      (element) => element.props["data-state"] === "available"
    );
    assert.ok(availableState);
    assert.equal(availableState.props.role, "status");
    assert.equal(availableState.props["aria-live"], "polite");
    assert.doesNotMatch(rendered, /raw_output|expected_output|input_payload|cookie|secret/i);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

type InspectorTree = React.ReactElement<Record<string, unknown>>;
type HookSlot =
  | { kind: "state"; value: unknown }
  | { kind: "ref"; value: { current: unknown } };

function createWorkspaceInspectorHarness() {
  const slots: HookSlot[] = [];
  let cursor = 0;
  let tree: InspectorTree;
  const internals = React as unknown as {
    __CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE: { H: unknown };
  };
  const dispatcher = {
    useState<T>(initialValue: T | (() => T)): [T, React.Dispatch<React.SetStateAction<T>>] {
      const index = cursor++;
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
      const index = cursor++;
      if (!slots[index]) {
        slots[index] = { kind: "ref", value: { current: initialValue } };
      }
      const slot = slots[index];
      assert.equal(slot?.kind, "ref");
      return slot.value as React.RefObject<T>;
    },
    useId() {
      return `workspace-test-id-${cursor++}`;
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
        const root = LocalBenchmarkWorkspaceInspector({
          candidates: [
            { id: "commit-revised", label: "Revised commit" },
            { id: "commit-baseline", label: "Baseline commit" }
          ],
          contextId: "context-a",
          projectId: "project-a"
        }) as InspectorTree;
        tree = materialize(root) as InspectorTree;
      } finally {
        internals.__CLIENT_INTERNALS_DO_NOT_USE_OR_WARN_USERS_THEY_CANNOT_UPGRADE.H = previousDispatcher;
      }
      return tree;
    }
  };
  harness.render();
  return harness;
}

function materialize(node: React.ReactNode): React.ReactNode {
  if (!React.isValidElement(node)) {
    return node;
  }
  const element = node as InspectorTree;
  if (typeof element.type === "function") {
    return materialize((element.type as (props: Record<string, unknown>) => React.ReactNode)(element.props));
  }
  return React.cloneElement(
    element,
    undefined,
    ...React.Children.toArray(element.props.children as React.ReactNode).map(materialize)
  );
}

function change(harness: ReturnType<typeof createWorkspaceInspectorHarness>, name: string, value: string) {
  const element = findByName(harness.tree, name);
  invoke(element, "onChange", { target: { value } });
  harness.render();
}

function blur(harness: ReturnType<typeof createWorkspaceInspectorHarness>, name: string) {
  const element = findByName(harness.tree, name);
  invoke(element, "onBlur");
}

function submit(harness: ReturnType<typeof createWorkspaceInspectorHarness>) {
  const form = findElement(harness.tree, (element) => element.props.className === "local-benchmark-workspace__controls");
  assert.ok(form);
  invoke(form, "onSubmit", { preventDefault() {} });
  harness.render();
}

function findByName(tree: InspectorTree, name: string): InspectorTree {
  const element = findElement(tree, (candidate) => candidate.props.name === name);
  assert.ok(element, `expected control ${name}`);
  return element;
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

function invoke(element: InspectorTree, handler: string, event?: unknown) {
  const callback = element.props[handler];
  assert.equal(typeof callback, "function", `${handler} must be executable`);
  (callback as (event?: unknown) => void)(event);
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

function flushAsyncWork(): Promise<void> {
  return new Promise((resolve) => setImmediate(resolve));
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { "content-type": "application/json" }
  });
}

function benchmarkWorkspacePayload() {
  return {
    schema_version: "contextlab.local-benchmark-workspace.v1",
    project_id: "project-a",
    context_id: "context-a",
    revised: { commit_id: "commit-revised", cohort_id: "cohort-revised" },
    baseline: { commit_id: "commit-baseline", cohort_id: "cohort-baseline" },
    decision_pair_witness: {
      schema_version: 1,
      project_id: "project-a",
      context_id: "context-a",
      baseline: { commit_id: "commit-baseline", decision_id: "decision-baseline" },
      revised: { commit_id: "commit-revised", decision_id: "decision-revised" }
    },
    projection: {
      schema_version: 1,
      receipt: { cohort_id: "cohort-revised" },
      suite: { id: "suite-release", name: "Release gate" },
      datasets: [
        { id: "dataset-a", name: "Support cases", case_count: 2 },
        { id: "dataset-b", name: "Billing cases", case_count: 2 }
      ],
      runs: [
        { dataset_id: "dataset-a", case_id: "case-a1", metric_count: 2 },
        { dataset_id: "dataset-a", case_id: "case-a2", metric_count: 2 },
        { dataset_id: "dataset-b", case_id: "case-b1", metric_count: 2 },
        { dataset_id: "dataset-b", case_id: "case-b2", metric_count: 2 }
      ],
      scorecard: {
        run_count: 4,
        metrics: [
          metric("latency_ms", "maximum", 800, 900, "regressed"),
          metric("accuracy", "minimum", 0.9, 0.75, "regressed")
        ]
      },
      regression_status: "regressed",
      evaluation_diff: {
        baseline_cohort_id: "cohort-baseline",
        revised_cohort_id: "cohort-revised",
        status_change: ["passed", "regressed"],
        metric_changes: [
          {
            metric: "latency_ms",
            change_kind: "modified",
            baseline: metric("latency_ms", "maximum", 800, 700, "passed"),
            revised: metric("latency_ms", "maximum", 800, 900, "regressed")
          },
          {
            metric: "accuracy",
            change_kind: "modified",
            baseline: metric("accuracy", "minimum", 0.9, 0.95, "passed"),
            revised: metric("accuracy", "minimum", 0.9, 0.75, "regressed")
          }
        ]
      }
    }
  };
}

function metric(
  name: "latency_ms" | "accuracy",
  direction: "minimum" | "maximum",
  threshold: number,
  observed: number,
  outcome: "passed" | "regressed"
) {
  return {
    metric: name,
    threshold_direction: direction,
    threshold_value: threshold,
    observed,
    sample_count: 4,
    required_sample_count: 4,
    has_complete_coverage: true,
    outcome
  };
}

function benchmarkDecisionDiscoveryPayload(commitId: string, decisionId: string) {
  return {
    schema_version: "contextlab.local-benchmark-decision-discovery.v1",
    project_id: "project-a",
    context_id: "context-a",
    commit_id: commitId,
    decisions: [{
      decision_id: decisionId,
      suite_id: "suite-release",
      status: "passed",
      recorded_at: "2026-07-22T00:00:00Z"
    }]
  };
}
