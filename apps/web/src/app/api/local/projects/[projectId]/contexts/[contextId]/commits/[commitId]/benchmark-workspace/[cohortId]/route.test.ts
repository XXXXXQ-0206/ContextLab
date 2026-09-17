import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("forwards an exact paired benchmark workspace scope with bearer-only no-store transport", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(benchmarkWorkspacePayload({
      projectId: "project/id",
      contextId: "context/id",
      revisedCommitId: "revised/commit",
      revisedCohortId: "revised/cohort",
      baselineCommitId: "baseline/commit",
      baselineCohortId: "baseline/cohort"
    }));
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/projects/project%2Fid/contexts/context%2Fid/commits/revised%2Fcommit/benchmark-workspace/revised%2Fcohort?baseline_commit_id=baseline%2Fcommit&baseline_cohort_id=baseline%2Fcohort",
        {
          headers: {
            authorization: "Bearer request-token",
            cookie: "session=must-not-cross-the-bff"
          }
        }
      ),
      routeContext()
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).projection.receipt.cohort_id, "revised/cohort");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/revised%2Fcommit/benchmark-workspace/revised%2Fcohort?baseline_commit_id=baseline%2Fcommit&baseline_cohort_id=baseline%2Fcohort"
    );
    const headers = new Headers(requests[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer request-token");
    assert.equal(headers.get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("accepts no query for a single workspace projection", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse(benchmarkWorkspacePayload({
      projectId: "project",
      contextId: "context",
      revisedCommitId: "commit",
      revisedCohortId: "cohort"
    }))) as typeof fetch;

  try {
    const response = await GET(
      requestFor("", { authorization: "Bearer request-token" }),
      routeContext("project", "context", "commit", "cohort")
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).baseline, null);
  } finally {
    restoreEnvironment();
  }
});

test("rejects unknown, partial, duplicate, and blank query values before upstream access", async (t) => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCalls = 0;
  globalThis.fetch = (async () => {
    upstreamCalls += 1;
    return jsonResponse({});
  }) as typeof fetch;

  try {
    for (const query of [
      "?unexpected=true",
      "?baseline_commit_id=baseline",
      "?baseline_cohort_id=cohort",
      "?baseline_commit_id=baseline&baseline_cohort_id=",
      "?baseline_commit_id=&baseline_cohort_id=cohort",
      "?baseline_commit_id=one&baseline_commit_id=two&baseline_cohort_id=cohort"
    ]) {
      await t.test(query, async () => {
        const response = await GET(
          requestFor(query, { authorization: "Bearer request-token" }),
          routeContext("project", "context", "commit", "cohort")
        );

        assert.equal(response.status, 400);
        assert.deepEqual(await response.json(), {
          error: "invalid_local_benchmark_workspace_request",
          message: "Only a complete baseline_commit_id and baseline_cohort_id pair is supported"
        });
      });
    }
    assert.equal(upstreamCalls, 0);
  } finally {
    restoreEnvironment();
  }
});

test("rejects missing or malformed bearer authentication before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCalls = 0;
  globalThis.fetch = (async () => {
    upstreamCalls += 1;
    return jsonResponse({});
  }) as typeof fetch;

  try {
    for (const authorization of [undefined, "Basic token", "Bearer", "Bearer ", "Bearer token with-space"]) {
      const response = await GET(
        requestFor("", authorization ? { authorization } : undefined),
        routeContext("project", "context", "commit", "cohort")
      );
      assert.equal(response.status, 401);
    }
    assert.equal(upstreamCalls, 0);
  } finally {
    restoreEnvironment();
  }
});

test("preserves safe structured upstream errors and retry timing", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "benchmark_workspace_rate_limited",
        message: "Too many exact benchmark workspace reads",
        internal_trace: "must-not-cross-the-bff",
        raw_outputs: ["private"]
      },
      { status: 429, headers: { "content-type": "application/json", "retry-after": "15" } }
    )) as typeof fetch;

  try {
    const response = await GET(
      requestFor("", { authorization: "Bearer request-token" }),
      routeContext("project", "context", "commit", "cohort")
    );

    assert.equal(response.status, 429);
    assert.equal(response.headers.get("retry-after"), "15");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
    assert.deepEqual(await response.json(), {
      error: "benchmark_workspace_rate_limited",
      message: "Too many exact benchmark workspace reads"
    });
  } finally {
    restoreEnvironment();
  }
});

test("maps malformed successful projections and transport failures to a redacted BFF error", async (t) => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";

  try {
    await t.test("malformed success", async () => {
      const malformed = benchmarkWorkspacePayload({
        projectId: "project",
        contextId: "context",
        revisedCommitId: "commit",
        revisedCohortId: "cohort"
      });
      (malformed.projection as Record<string, unknown>).raw_output = "private";
      globalThis.fetch = (async () => jsonResponse(malformed)) as typeof fetch;

      const response = await GET(
        requestFor("", { authorization: "Bearer request-token" }),
        routeContext("project", "context", "commit", "cohort")
      );
      assert.equal(response.status, 502);
      assert.deepEqual(await response.json(), {
        error: "contextlab_web_api_error",
        message: "Unable to load local benchmark workspace"
      });
    });

    await t.test("transport failure", async () => {
      globalThis.fetch = (async () => {
        throw new Error("upstream host details must not escape");
      }) as typeof fetch;

      const response = await GET(
        requestFor("", { authorization: "Bearer request-token" }),
        routeContext("project", "context", "commit", "cohort")
      );
      assert.equal(response.status, 502);
      assert.deepEqual(await response.json(), {
        error: "contextlab_web_api_error",
        message: "Unable to load local benchmark workspace"
      });
    });
  } finally {
    restoreEnvironment();
  }
});

function requestFor(query: string, headers?: HeadersInit): Request {
  return new Request(
    `http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-workspace/cohort${query}`,
    { headers }
  );
}

function routeContext(
  projectId = "project/id",
  contextId = "context/id",
  commitId = "revised/commit",
  cohortId = "revised/cohort"
) {
  return { params: Promise.resolve({ projectId, contextId, commitId, cohortId }) };
}

type BenchmarkWorkspaceScope = {
  projectId: string;
  contextId: string;
  revisedCommitId: string;
  revisedCohortId: string;
  baselineCommitId?: string;
  baselineCohortId?: string;
};

function benchmarkWorkspacePayload(scope: BenchmarkWorkspaceScope) {
  const baseline = scope.baselineCommitId && scope.baselineCohortId
    ? { commit_id: scope.baselineCommitId, cohort_id: scope.baselineCohortId }
    : null;
  return {
    schema_version: "contextlab.local-benchmark-workspace.v1",
    project_id: scope.projectId,
    context_id: scope.contextId,
    revised: { commit_id: scope.revisedCommitId, cohort_id: scope.revisedCohortId },
    baseline,
    projection: {
      schema_version: 1,
      receipt: { cohort_id: scope.revisedCohortId },
      suite: { id: "suite/id", name: "Release gate" },
      datasets: [{ id: "dataset/id", name: "Release dataset", case_count: 1 }],
      runs: [{ dataset_id: "dataset/id", case_id: "case/id", metric_count: 1 }],
      scorecard: {
        run_count: 1,
        metrics: [metric(0.9, 0.94, "passed")]
      },
      regression_status: "passed",
      evaluation_diff: baseline
        ? {
            baseline_cohort_id: baseline.cohort_id,
            revised_cohort_id: scope.revisedCohortId,
            status_change: null,
            metric_changes: []
          }
        : null
    }
  };
}

function metric(threshold: number, observed: number, outcome: "passed" | "regressed") {
  return {
    metric: "accuracy",
    threshold_direction: "minimum",
    threshold_value: threshold,
    observed,
    sample_count: 1,
    required_sample_count: 1,
    has_complete_coverage: true,
    outcome
  };
}

function restoreEnvironment() {
  if (originalApiBaseUrl === undefined) {
    delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  } else {
    process.env.CONTEXTLAB_WEB_API_BASE_URL = originalApiBaseUrl;
  }
  globalThis.fetch = originalFetch;
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
