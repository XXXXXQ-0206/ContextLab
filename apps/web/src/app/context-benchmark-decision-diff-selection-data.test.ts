import assert from "node:assert/strict";
import test from "node:test";
import { loadLocalBenchmarkDecisionDiffSelections } from "./context-benchmark-decision-diff-selection-data";

const originalFetch = globalThis.fetch;

test("diff selection loads both exact commit discovery scopes without cookies", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    const commitId = String(input).includes("baseline") ? "baseline/commit" : "revised/commit";
    return new Response(JSON.stringify(discoveryPayload(commitId)), {
      headers: { "content-type": "application/json" }
    });
  }) as typeof fetch;

  try {
    const selections = await loadLocalBenchmarkDecisionDiffSelections(
      "project/id",
      "context/id",
      "baseline/commit",
      "revised/commit",
      "request-token"
    );

    assert.equal(selections.baseline.commit_id, "baseline/commit");
    assert.equal(selections.revised.commit_id, "revised/commit");
    assert.deepEqual(
      requests.map((request) => request.input).sort(),
      [
        "/api/local/projects/project%2Fid/contexts/context%2Fid/commits/baseline%2Fcommit/benchmark-decisions",
        "/api/local/projects/project%2Fid/contexts/context%2Fid/commits/revised%2Fcommit/benchmark-decisions"
      ].sort()
    );
    for (const request of requests) {
      assert.equal(new Headers(request.init?.headers).get("authorization"), "Bearer request-token");
      assert.equal(new Headers(request.init?.headers).get("cookie"), null);
      assert.equal(request.init?.credentials, "omit");
      assert.equal(request.init?.cache, "no-store");
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function discoveryPayload(commitId: string) {
  return {
    schema_version: "contextlab.local-benchmark-decision-discovery.v1",
    project_id: "project/id",
    context_id: "context/id",
    commit_id: commitId,
    decisions: [
      {
        decision_id: `${commitId}/decision`,
        suite_id: "suite/quality",
        status: "passed",
        recorded_at: "2026-07-22T00:01:00Z"
      }
    ]
  };
}
