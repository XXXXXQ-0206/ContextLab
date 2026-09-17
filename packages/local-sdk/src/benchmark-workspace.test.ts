import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalClient,
  createLocalBenchmarkWorkspaceDecisionSelectionResource,
  loadLocalBenchmarkWorkspaceForDecisionSelection,
  parseLocalBenchmarkWorkspace,
  selectLocalBenchmarkWorkspaceDecision,
  type FetchLike
} from "./index";

test("parses the exact redacted benchmark workspace comparison envelope", () => {
  const payload = benchmarkWorkspacePayload();

  assert.deepEqual(parseLocalBenchmarkWorkspace(payload), payload);
});

test("parses the optional server-owned decision pair witness", () => {
  const payload = {
    ...benchmarkWorkspacePayload(),
    decision_pair_witness: benchmarkDecisionPairWitnessPayload()
  };

  assert.deepEqual(parseLocalBenchmarkWorkspace(payload), payload);
});

test("rejects benchmark workspace schema drift, raw fields, unstable order, and count drift", () => {
  const payload = benchmarkWorkspacePayload();

  for (const [name, mutate] of [
    ["unknown envelope field", (value: typeof payload) => { (value as Record<string, unknown>).metadata = {}; }],
    ["raw case input", (value: typeof payload) => { (value.projection.runs[0] as Record<string, unknown>).input = {}; }],
    ["raw model output", (value: typeof payload) => { (value.projection as Record<string, unknown>).model_output = {}; }],
    ["dataset order", (value: typeof payload) => { value.projection.datasets.reverse(); }],
    ["metric order", (value: typeof payload) => { value.projection.scorecard.metrics.reverse(); }],
    ["run count", (value: typeof payload) => { value.projection.scorecard.run_count = 1; }],
    ["scope drift", (value: typeof payload) => { value.projection.receipt.cohort_id = "other/cohort"; }],
    ["diff drift", (value: typeof payload) => { value.projection.evaluation_diff.baseline_cohort_id = "other/baseline"; }]
  ] as const) {
    assert.throws(
      () => parseLocalBenchmarkWorkspace(mutatePayload(payload, mutate)),
      TypeError,
      name
    );
  }
});

test("rejects decision pair witness schema and scope drift", () => {
  const payload = {
    ...benchmarkWorkspacePayload(),
    decision_pair_witness: benchmarkDecisionPairWitnessPayload()
  };

  for (const [name, mutate] of [
    ["unknown witness field", (value: typeof payload) => { (value.decision_pair_witness as Record<string, unknown>).metadata = {}; }],
    ["null witness", (value: typeof payload) => { (value as unknown as { decision_pair_witness: null }).decision_pair_witness = null; }],
    ["witness schema", (value: typeof payload) => { value.decision_pair_witness.schema_version = 2; }],
    ["witness project", (value: typeof payload) => { value.decision_pair_witness.project_id = "other/project"; }],
    ["witness context", (value: typeof payload) => { value.decision_pair_witness.context_id = "other/context"; }],
    ["witness baseline commit", (value: typeof payload) => { value.decision_pair_witness.baseline.commit_id = "other/baseline"; }],
    ["witness revised commit", (value: typeof payload) => { value.decision_pair_witness.revised.commit_id = "other/revised"; }],
    ["witness self-pair commit", (value: typeof payload) => {
      value.revised.commit_id = "baseline/commit";
      value.decision_pair_witness.revised.commit_id = "baseline/commit";
    }],
    ["witness self-pair decision", (value: typeof payload) => {
      value.decision_pair_witness.revised.decision_id = "baseline/decision";
    }],
    ["witness without baseline", (value: typeof payload) => {
      (value as unknown as { baseline: null }).baseline = null;
      (value.projection as unknown as { evaluation_diff: null }).evaluation_diff = null;
    }]
  ] as const) {
    assert.throws(
      () => parseLocalBenchmarkWorkspace(mutatePayload(payload, mutate)),
      TypeError,
      name
    );
  }
});

test("reads an exact private benchmark workspace comparison with encoded scope and bearer-only transport", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ input: String(input), init });
    return jsonResponse(benchmarkWorkspacePayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const workspace = await client.getBenchmarkWorkspace(
    "project/id",
    "context/id",
    "revised/commit",
    "revised/cohort",
    { bearerToken: "request-token" },
    { commitId: "baseline/commit", cohortId: "baseline/cohort" }
  );

  assert.equal(workspace.projection.receipt.cohort_id, "revised/cohort");
  assert.equal(
    requests[0]?.input,
    "http://127.0.0.1:3100/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/revised%2Fcommit/benchmark-workspace/revised%2Fcohort?baseline_commit_id=baseline%2Fcommit&baseline_cohort_id=baseline%2Fcohort"
  );
  const headers = new Headers(requests[0]?.init?.headers);
  assert.equal(headers.get("authorization"), "Bearer request-token");
  assert.equal(headers.get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.cache, "no-store");
  assert.equal(requests[0]?.init?.method, undefined);
});

test("reads a benchmark workspace through the exact sealed decision binding without cohort input", async () => {
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  const client = new ContextLabLocalClient({
    fetch: async (input, init) => {
      calls.push({ url: String(input), init });
      return jsonResponse({
        ...benchmarkWorkspacePayload(),
        decision_pair_witness: benchmarkDecisionPairWitnessPayload()
      });
    }
  });

  const workspace = await client.getBenchmarkWorkspaceForDecision(
    "project/id",
    "context/id",
    "revised/commit",
    "revised/decision",
    { bearerToken: "request-token" },
    { commitId: "baseline/commit", decisionId: "baseline/decision" }
  );

  assert.equal(workspace.revised.cohort_id, "revised/cohort");
  assert.deepEqual(workspace.decision_pair_witness, benchmarkDecisionPairWitnessPayload());
  assert.equal(calls[0]?.url, "http://127.0.0.1:3100/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/revised%2Fcommit/benchmark-decisions/revised%2Fdecision/workspace?baseline_commit_id=baseline%2Fcommit&baseline_decision_id=baseline%2Fdecision");
  assert.equal(calls[0]?.init?.credentials, "omit");
  const headers = new Headers(calls[0]?.init?.headers);
  assert.equal(headers.get("authorization"), "Bearer request-token");
  assert.equal(headers.get("cookie"), null);
  assert.equal(calls[0]?.init?.cache, "no-store");
});

test("requires an exact decision pair witness for decision-bound comparisons", async (t) => {
  for (const [name, mutate] of [
    ["missing witness", (value: ReturnType<typeof benchmarkWorkspacePayload> & { decision_pair_witness?: unknown }) => { delete value.decision_pair_witness; }],
    ["baseline decision", (value: ReturnType<typeof benchmarkWorkspacePayload> & { decision_pair_witness: ReturnType<typeof benchmarkDecisionPairWitnessPayload> }) => { value.decision_pair_witness.baseline.decision_id = "other/baseline-decision"; }],
    ["revised decision", (value: ReturnType<typeof benchmarkWorkspacePayload> & { decision_pair_witness: ReturnType<typeof benchmarkDecisionPairWitnessPayload> }) => { value.decision_pair_witness.revised.decision_id = "other/revised-decision"; }]
  ] as const) {
    await t.test(name, async () => {
      const payload = {
        ...benchmarkWorkspacePayload(),
        decision_pair_witness: benchmarkDecisionPairWitnessPayload()
      };
      mutate(payload as never);
      const client = new ContextLabLocalClient({ fetch: async () => jsonResponse(payload) });

      await assert.rejects(
        () => client.getBenchmarkWorkspaceForDecision(
          "project/id",
          "context/id",
          "revised/commit",
          "revised/decision",
          { bearerToken: "request-token" },
          { commitId: "baseline/commit", decisionId: "baseline/decision" }
        ),
        TypeError
      );
    });
  }
});

test("rejects a decision witness on the cohort-only route", async () => {
  const client = new ContextLabLocalClient({
    fetch: async () => jsonResponse({
      ...benchmarkWorkspacePayload(),
      decision_pair_witness: benchmarkDecisionPairWitnessPayload()
    })
  });

  await assert.rejects(
    () => client.getBenchmarkWorkspace(
      "project/id",
      "context/id",
      "revised/commit",
      "revised/cohort",
      { bearerToken: "request-token" }
    ),
    TypeError
  );
});

test("rejects a benchmark workspace response outside the requested exact scope", async (t) => {
  for (const [name, mutate] of [
    ["project", (value: ReturnType<typeof benchmarkWorkspacePayload>) => { value.project_id = "other/project"; }],
    ["context", (value: ReturnType<typeof benchmarkWorkspacePayload>) => { value.context_id = "other/context"; }],
    ["revised commit", (value: ReturnType<typeof benchmarkWorkspacePayload>) => { value.revised.commit_id = "other/commit"; }],
    ["baseline commit", (value: ReturnType<typeof benchmarkWorkspacePayload>) => { value.baseline.commit_id = "other/baseline"; }]
  ] as const) {
    await t.test(name, async () => {
      const payload = benchmarkWorkspacePayload();
      mutate(payload);
      const client = new ContextLabLocalClient({ fetch: async () => jsonResponse(payload) });

      await assert.rejects(
        () => client.getBenchmarkWorkspace(
          "project/id",
          "context/id",
          "revised/commit",
          "revised/cohort",
          { bearerToken: "request-token" },
          { commitId: "baseline/commit", cohortId: "baseline/cohort" }
        ),
        TypeError
      );
    });
  }
});

test("binds an exact workspace selection to a listed decision and preserves empty semantics", () => {
  const scope = {
    projectId: "project/id",
    contextId: "context/id",
    commitId: "revised/commit"
  } as const;
  const list = benchmarkDecisionListPayload();
  const resource = createLocalBenchmarkWorkspaceDecisionSelectionResource({
    state: "available",
    scope,
    list,
    selectedDecisionId: "decision/b"
  });

  assert.equal(resource.state, "available");
  assert.equal(resource.selection?.decisionId, "decision/b");
  assert.equal(resource.selection?.decision.decision_id, "decision/b");
  assert.equal(resource.selection?.commitId, "revised/commit");
  assert.deepEqual(
    resource.decisions.map((decision) => decision.decision_id),
    ["decision/a", "decision/b"]
  );
  assert.equal(Object.isFrozen(resource), true);
  assert.equal(Object.isFrozen(resource.selection), true);

  const empty = createLocalBenchmarkWorkspaceDecisionSelectionResource({
    state: "available",
    scope,
    list: { ...list, decisions: [] },
    selectedDecisionId: null
  });
  assert.deepEqual(empty, {
    state: "empty",
    scope,
    decisions: [],
    selection: null
  });
});

test("keeps loading, error, and unavailable selection states explicit", () => {
  const scope = {
    projectId: "project/id",
    contextId: "context/id",
    commitId: "revised/commit"
  } as const;

  assert.deepEqual(
    createLocalBenchmarkWorkspaceDecisionSelectionResource({ state: "loading", scope }),
    { state: "loading", scope }
  );
  assert.deepEqual(
    createLocalBenchmarkWorkspaceDecisionSelectionResource({
      state: "error",
      scope,
      message: "decision list read failed"
    }),
    { state: "error", scope, message: "decision list read failed" }
  );
  assert.deepEqual(
    createLocalBenchmarkWorkspaceDecisionSelectionResource({ state: "unavailable", scope }),
    { state: "unavailable", scope }
  );
});

test("fails closed for list scope drift, unknown selection, and schema drift", () => {
  const scope = {
    projectId: "project/id",
    contextId: "context/id",
    commitId: "revised/commit"
  } as const;
  const list = benchmarkDecisionListPayload();

  assert.throws(
    () => selectLocalBenchmarkWorkspaceDecision(
      { ...list, project_id: "other/project" },
      scope,
      "decision/a"
    ),
    TypeError
  );
  assert.throws(
    () => selectLocalBenchmarkWorkspaceDecision(list, scope, "handwritten/decision"),
    TypeError
  );
  assert.throws(
    () => createLocalBenchmarkWorkspaceDecisionSelectionResource({
      state: "available",
      scope,
      list: { ...list, metadata: {} } as never,
      selectedDecisionId: "decision/a"
    }),
    TypeError
  );
});

test("loads the workspace only through the selected list identity and exact scope", async () => {
  const calls: Array<{
    projectId: string;
    contextId: string;
    commitId: string;
    decisionId: string;
    baseline?: { commitId: string; decisionId: string };
  }> = [];
  const scope = {
    projectId: "project/id",
    contextId: "context/id",
    commitId: "revised/commit"
  } as const;
  const list = benchmarkDecisionListPayload();
  const selection = selectLocalBenchmarkWorkspaceDecision(list, scope, "decision/b");
  assert.notEqual(selection, null);

  const workspace = await loadLocalBenchmarkWorkspaceForDecisionSelection(
    {
      getBenchmarkWorkspaceForDecision: async (
        projectId,
        contextId,
        commitId,
        decisionId,
        _credentials,
        baseline
      ) => {
        calls.push({ projectId, contextId, commitId, decisionId, baseline });
        return parseLocalBenchmarkWorkspace(benchmarkWorkspacePayload());
      }
    },
    selection!,
    { bearerToken: "request-token" },
    { commitId: "baseline/commit", decisionId: "decision/a" }
  );

  assert.equal(workspace.revised.commit_id, "revised/commit");
  assert.deepEqual(calls, [{
    projectId: "project/id",
    contextId: "context/id",
    commitId: "revised/commit",
    decisionId: "decision/b",
    baseline: { commitId: "baseline/commit", decisionId: "decision/a" }
  }]);
});

function benchmarkWorkspacePayload() {
  const baselineMetric = metric(700, "passed");
  const revisedMetric = metric(900, "regressed");
  return {
    schema_version: "contextlab.local-benchmark-workspace.v1",
    project_id: "project/id",
    context_id: "context/id",
    revised: {
      commit_id: "revised/commit",
      cohort_id: "revised/cohort"
    },
    baseline: {
      commit_id: "baseline/commit",
      cohort_id: "baseline/cohort"
    },
    projection: {
      schema_version: 1,
      receipt: { cohort_id: "revised/cohort" },
      suite: { id: "suite/id", name: "Release gate" },
      datasets: [
        { id: "dataset/a", name: "First dataset", case_count: 1 },
        { id: "dataset/b", name: "Second dataset", case_count: 1 }
      ],
      runs: [
        { dataset_id: "dataset/a", case_id: "case/a", metric_count: 1 },
        { dataset_id: "dataset/b", case_id: "case/b", metric_count: 1 }
      ],
      scorecard: {
        run_count: 2,
        metrics: [revisedMetric, accuracyMetric(0.75, "regressed")]
      },
      regression_status: "regressed",
      evaluation_diff: {
        baseline_cohort_id: "baseline/cohort",
        revised_cohort_id: "revised/cohort",
        status_change: ["passed", "regressed"],
        metric_changes: [
          {
            metric: "latency_ms",
            change_kind: "modified",
            baseline: baselineMetric,
            revised: revisedMetric
          },
          {
            metric: "accuracy",
            change_kind: "modified",
            baseline: accuracyMetric(0.95, "passed"),
            revised: accuracyMetric(0.75, "regressed")
          }
        ]
      }
    }
  };
}

function benchmarkDecisionListPayload() {
  return {
    schema_version: "contextlab.local-benchmark-decision-list.v1" as const,
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "revised/commit",
    decisions: [
      {
        decision_id: "decision/a",
        suite: { id: "suite/a", name: "First suite" },
        datasets: [{ id: "dataset/a", name: "First dataset", case_count: 1 }],
        status: "passed" as const,
        recorded_at: "2026-07-30T12:00:00Z",
        run_count: 1
      },
      {
        decision_id: "decision/b",
        suite: { id: "suite/b", name: "Second suite" },
        datasets: [{ id: "dataset/b", name: "Second dataset", case_count: 2 }],
        status: "regressed" as const,
        recorded_at: "2026-07-30T11:00:00Z",
        run_count: 2
      }
    ]
  };
}

function benchmarkDecisionPairWitnessPayload() {
  return {
    schema_version: 1,
    project_id: "project/id",
    context_id: "context/id",
    baseline: {
      commit_id: "baseline/commit",
      decision_id: "baseline/decision"
    },
    revised: {
      commit_id: "revised/commit",
      decision_id: "revised/decision"
    }
  };
}

function metric(observed: number, outcome: "passed" | "regressed") {
  return {
    metric: "latency_ms",
    threshold_direction: "maximum",
    threshold_value: 800,
    observed,
    sample_count: 2,
    required_sample_count: 2,
    has_complete_coverage: true,
    outcome
  };
}

function accuracyMetric(observed: number, outcome: "passed" | "regressed") {
  return {
    metric: "accuracy",
    threshold_direction: "minimum",
    threshold_value: 0.9,
    observed,
    sample_count: 2,
    required_sample_count: 2,
    has_complete_coverage: true,
    outcome
  };
}

function mutatePayload<T>(payload: T, mutate: (value: T) => void): T {
  const copy = structuredClone(payload);
  mutate(copy);
  return copy;
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" }
  });
}
