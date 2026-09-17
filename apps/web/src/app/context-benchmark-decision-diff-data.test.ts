import assert from "node:assert/strict";
import test from "node:test";
import {
  LocalBenchmarkDecisionDiffProxyError,
  loadLocalBenchmarkDecisionDiff
} from "./context-benchmark-decision-diff-data";

const originalFetch = globalThis.fetch;

test("benchmark decision diff data client uses the exact same-origin scopes without cookies", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse(diffPayload());
  }) as typeof fetch;

  try {
    const diff = await loadLocalBenchmarkDecisionDiff(
      "project/id",
      "context/id",
      { commit_id: "baseline/commit", decision_id: "baseline/decision" },
      { commit_id: "revised/commit", decision_id: "revised/decision" },
      "request-token"
    );

    assert.equal(diff.revised.decision_id, "revised/decision");
    assert.equal(
      requests[0]?.input,
      "/api/local/projects/project%2Fid/contexts/context%2Fid/benchmark-decision-diffs?baseline_commit_id=baseline%2Fcommit&baseline_decision_id=baseline%2Fdecision&revised_commit_id=revised%2Fcommit&revised_decision_id=revised%2Fdecision"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("benchmark decision diff data client preserves structured conflicts", async () => {
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "benchmark_decision_comparison_unavailable",
        message: "benchmark decisions cannot be compared"
      },
      { status: 409 }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBenchmarkDecisionDiff("project", "context", scope("base"), scope("revised"), "token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalBenchmarkDecisionDiffProxyError);
        assert.equal(error.status, 409);
        assert.equal(error.body.error, "benchmark_decision_comparison_unavailable");
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function scope(commitId: string) {
  return { commit_id: commitId, decision_id: `${commitId}-decision` };
}

function diffPayload() {
  return {
    project_id: "project/id",
    context_id: "context/id",
    baseline: { commit_id: "baseline/commit", decision_id: "baseline/decision" },
    revised: { commit_id: "revised/commit", decision_id: "revised/decision" },
    status_change: null,
    metric_changes: []
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
