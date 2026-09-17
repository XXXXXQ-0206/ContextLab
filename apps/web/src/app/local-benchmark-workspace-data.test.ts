import assert from "node:assert/strict";
import test from "node:test";
import {
  loadLocalBenchmarkWorkspace,
  LocalBenchmarkWorkspaceProxyError
} from "./local-benchmark-workspace-data";

const originalFetch = globalThis.fetch;

test("loads, scope-matches, and recursively freezes the exact same-origin projection", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(benchmarkWorkspacePayload());
  }) as typeof fetch;

  try {
    const workspace = await loadLocalBenchmarkWorkspace(
      {
        projectId: "project/id",
        contextId: "context/id",
        commitId: "revised/commit",
        cohortId: "revised/cohort",
        baseline: { commitId: "baseline/commit", cohortId: "baseline/cohort" }
      },
      " request-token "
    );

    assert.equal(
      requests[0]?.url,
      "/api/local/projects/project%2Fid/contexts/context%2Fid/commits/revised%2Fcommit/benchmark-workspace/revised%2Fcohort?baseline_commit_id=baseline%2Fcommit&baseline_cohort_id=baseline%2Fcohort"
    );
    const headers = new Headers(requests[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer request-token");
    assert.equal(headers.get("cookie"), null);
    assert.equal(headers.get("accept"), "application/json");
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(Object.isFrozen(workspace), true);
    assert.equal(Object.isFrozen(workspace.projection), true);
    assert.equal(Object.isFrozen(workspace.projection.datasets), true);
    assert.equal(Object.isFrozen(workspace.projection.datasets[0]), true);
    assert.equal(Object.isFrozen(workspace.projection.scorecard.metrics), true);
    assert.deepEqual(
      workspace.projection.datasets.map((dataset) => dataset.id),
      ["dataset-a", "dataset-b"]
    );
    assert.deepEqual(
      workspace.projection.scorecard.metrics.map((metric) => metric.metric),
      ["latency_ms", "accuracy"]
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("preserves an exact server-owned decision pair witness without deriving identity from cohorts", async () => {
  globalThis.fetch = (async () => jsonResponse(decisionBenchmarkWorkspacePayload())) as typeof fetch;

  try {
    const workspace = await loadLocalBenchmarkWorkspace(
      {
        projectId: "project/id",
        contextId: "context/id",
        commitId: "revised/commit",
        decisionId: "revised/decision",
        baseline: { commitId: "baseline/commit", decisionId: "baseline/decision" }
      },
      "request-token"
    );

    assert.deepEqual(workspace.decision_pair_witness, {
      schema_version: 1,
      project_id: "project/id",
      context_id: "context/id",
      baseline: { commit_id: "baseline/commit", decision_id: "baseline/decision" },
      revised: { commit_id: "revised/commit", decision_id: "revised/decision" }
    });
    assert.equal(Object.isFrozen(workspace.decision_pair_witness), true);
    assert.equal(Object.isFrozen(workspace.decision_pair_witness?.baseline), true);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("rejects decision pair witness identity drift and unknown witness fields", async () => {
  const payload = decisionBenchmarkWorkspacePayload();
  payload.decision_pair_witness.revised.decision_id = "other/decision";
  globalThis.fetch = (async () => jsonResponse(payload)) as typeof fetch;

  try {
    await assert.rejects(
      () =>
        loadLocalBenchmarkWorkspace(
          {
            projectId: "project/id",
            contextId: "context/id",
            commitId: "revised/commit",
            decisionId: "revised/decision",
            baseline: { commitId: "baseline/commit", decisionId: "baseline/decision" }
          },
          "request-token"
        ),
      /requested scope/
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  const rawPayload = decisionBenchmarkWorkspacePayload() as typeof payload & {
    decision_pair_witness: typeof payload.decision_pair_witness & { raw_output: string };
  };
  rawPayload.decision_pair_witness.raw_output = "private";
  globalThis.fetch = (async () => jsonResponse(rawPayload)) as typeof fetch;

  try {
    await assert.rejects(
      () =>
        loadLocalBenchmarkWorkspace(
          {
            projectId: "project/id",
            contextId: "context/id",
            commitId: "revised/commit",
            decisionId: "revised/decision",
            baseline: { commitId: "baseline/commit", decisionId: "baseline/decision" }
          },
          "request-token"
        ),
      /unexpected shape/
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("rejects a successful BFF envelope that drifts outside the requested scope", async () => {
  const payload = benchmarkWorkspacePayload();
  payload.project_id = "other-project";
  globalThis.fetch = (async () => jsonResponse(payload)) as typeof fetch;

  try {
    await assert.rejects(
      () =>
        loadLocalBenchmarkWorkspace(
          {
            projectId: "project/id",
            contextId: "context/id",
            commitId: "revised/commit",
            cohortId: "revised/cohort",
            baseline: { commitId: "baseline/commit", cohortId: "baseline/cohort" }
          },
          "request-token"
        ),
      /requested scope/
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("exposes only safe structured BFF errors and retry timing", async () => {
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "benchmark_workspace_rate_limited",
        message: "Too many reads",
        internal_trace: "private"
      },
      { status: 429, headers: { "content-type": "application/json", "retry-after": "12" } }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () =>
        loadLocalBenchmarkWorkspace(
          {
            projectId: "project",
            contextId: "context",
            commitId: "commit",
            cohortId: "cohort"
          },
          "request-token"
        ),
      (error: unknown) => {
        assert.ok(error instanceof LocalBenchmarkWorkspaceProxyError);
        assert.equal(error.status, 429);
        assert.equal(error.retryAfterMs, 12_000);
        assert.deepEqual(error.body, {
          error: "benchmark_workspace_rate_limited",
          message: "Too many reads"
        });
        assert.equal(Object.isFrozen(error.body), true);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("rejects incomplete request scope before same-origin access", async () => {
  let requestCount = 0;
  globalThis.fetch = (async () => {
    requestCount += 1;
    return jsonResponse(benchmarkWorkspacePayload());
  }) as typeof fetch;

  try {
    await assert.rejects(
      () =>
        loadLocalBenchmarkWorkspace(
          {
            projectId: "project",
            contextId: "context",
            commitId: "commit",
            cohortId: " ",
            baseline: { commitId: "baseline", cohortId: "cohort" }
          },
          "request-token"
        ),
      /cohortId/
    );
    await assert.rejects(
      () =>
        loadLocalBenchmarkWorkspace(
          {
            projectId: "project",
            contextId: "context",
            commitId: "commit",
            cohortId: "cohort"
          },
          " "
        ),
      /bearerToken/
    );
    assert.equal(requestCount, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function benchmarkWorkspacePayload() {
  return {
    schema_version: "contextlab.local-benchmark-workspace.v1",
    project_id: "project/id",
    context_id: "context/id",
    revised: { commit_id: "revised/commit", cohort_id: "revised/cohort" },
    baseline: { commit_id: "baseline/commit", cohort_id: "baseline/cohort" },
    projection: {
      schema_version: 1,
      receipt: { cohort_id: "revised/cohort" },
      suite: { id: "suite-release", name: "Release gate" },
      datasets: [
        { id: "dataset-a", name: "Core", case_count: 1 },
        { id: "dataset-b", name: "Safety", case_count: 1 }
      ],
      runs: [
        { dataset_id: "dataset-a", case_id: "case-a", metric_count: 2 },
        { dataset_id: "dataset-b", case_id: "case-b", metric_count: 2 }
      ],
      scorecard: {
        run_count: 2,
        metrics: [
          metric("latency_ms", "maximum", 800, 720, "passed"),
          metric("accuracy", "minimum", 0.9, 0.94, "passed")
        ]
      },
      regression_status: "passed",
      evaluation_diff: {
        baseline_cohort_id: "baseline/cohort",
        revised_cohort_id: "revised/cohort",
        status_change: null,
        metric_changes: [
          {
            metric: "accuracy",
            change_kind: "modified",
            baseline: metric("accuracy", "minimum", 0.9, 0.91, "passed"),
            revised: metric("accuracy", "minimum", 0.9, 0.94, "passed")
          }
        ]
      }
    }
  };
}

function decisionBenchmarkWorkspacePayload() {
  return {
    ...benchmarkWorkspacePayload(),
    decision_pair_witness: {
      schema_version: 1,
      project_id: "project/id",
      context_id: "context/id",
      baseline: { commit_id: "baseline/commit", decision_id: "baseline/decision" },
      revised: { commit_id: "revised/commit", decision_id: "revised/decision" }
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
    sample_count: 2,
    required_sample_count: 2,
    has_complete_coverage: true,
    outcome
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
