import assert from "node:assert/strict";
import test from "node:test";
import {
  LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1,
  LocalBenchmarkDecisionDiscoveryProxyError,
  loadLocalBenchmarkDecisionDiscovery,
  parseLocalBenchmarkDecisionDiscovery
} from "./context-benchmark-decision-discovery-data";

const originalFetch = globalThis.fetch;

test("benchmark decision discovery uses the exact commit scope without cookies", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse(discoveryPayload());
  }) as typeof fetch;

  try {
    const discovery = await loadLocalBenchmarkDecisionDiscovery(
      "project/id",
      "context/id",
      "commit/id",
      "request-token"
    );

    assert.equal(discovery.decisions[0]?.decision_id, "decision/first");
    assert.equal(
      requests[0]?.input,
      "/api/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("benchmark decision discovery rejects scope drift and raw fields", async () => {
  globalThis.fetch = (async () =>
    jsonResponse({
      ...discoveryPayload(),
      context_id: "other-context",
      decisions: [{
        ...discoveryPayload().decisions[0],
        raw_output: "must not cross the browser boundary"
      }]
    })) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBenchmarkDecisionDiscovery("project/id", "context/id", "commit/id", "token"),
      TypeError
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("benchmark decision discovery parser is frozen and schema-closed", () => {
  const parsed = parseLocalBenchmarkDecisionDiscovery(discoveryPayload());

  assert.equal(Object.isFrozen(parsed), true);
  assert.equal(Object.isFrozen(parsed.decisions), true);
  assert.equal(parsed.schema_version, LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1);
  assert.throws(
    () => parseLocalBenchmarkDecisionDiscovery({ ...discoveryPayload(), unsupported: true }),
    /unsupported fields/
  );
  assert.throws(
    () => parseLocalBenchmarkDecisionDiscovery({
      ...discoveryPayload(),
      decisions: [discoveryPayload().decisions[0], discoveryPayload().decisions[0]]
    }),
    /duplicate identities/
  );
  assert.throws(
    () => parseLocalBenchmarkDecisionDiscovery({
      ...discoveryPayload(),
      decisions: [...discoveryPayload().decisions].reverse()
    }),
    /ordered by recorded_at descending/
  );
});

test("benchmark decision discovery rejects malformed, impossible, and non-UTC timestamps", () => {
  for (const recordedAt of [
    "2026-02-30T00:00:00Z",
    "2026-07-22T00:00:00+08:00",
    "not-a-timestamp"
  ]) {
    assert.throws(
      () => parseLocalBenchmarkDecisionDiscovery(canonicalTimestampPayload(recordedAt)),
      /valid timestamp/
    );
    assert.throws(
      () => parseLocalBenchmarkDecisionDiscovery(compatibilityPayload({ recorded_at: recordedAt })),
      /valid timestamp/
    );
  }
});

test("benchmark decision discovery preserves structured unavailable responses", async () => {
  globalThis.fetch = (async () =>
    jsonResponse(
      { error: "contextlab_web_api_unavailable", message: "API unavailable" },
      { status: 503, headers: { "retry-after": "2" } }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBenchmarkDecisionDiscovery("project", "context", "commit", "token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalBenchmarkDecisionDiscoveryProxyError);
        assert.equal(error.status, 503);
        assert.equal(error.retryAfterMs, 2_000);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function discoveryPayload(decisionOverrides: Record<string, unknown> = {}) {
  return {
    schema_version: LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1,
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    decisions: [
      {
        decision_id: "decision/first",
        suite_id: "suite/quality",
        status: "passed",
        recorded_at: "2026-07-22T00:01:00Z",
        ...decisionOverrides
      },
      {
        decision_id: "decision/second",
        suite_id: "suite/safety",
        status: "insufficient_data",
        recorded_at: "2026-07-22T00:00:00Z"
      }
    ]
  };
}

function compatibilityPayload(decisionOverrides: Record<string, unknown> = {}) {
  return {
    schema_version: "contextlab.local-benchmark-decision-list.v1",
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    decisions: [
      {
        decision_id: "decision/first",
        suite: { id: "suite/quality", name: "Quality" },
        datasets: [{ id: "dataset/quality", name: "Quality", case_count: 1 }],
        status: "passed",
        recorded_at: "2026-07-22T00:01:00Z",
        run_count: 1,
        ...decisionOverrides
      }
    ]
  };
}

function canonicalTimestampPayload(recordedAt: string) {
  const payload = discoveryPayload({ recorded_at: recordedAt });
  return { ...payload, decisions: [payload.decisions[0]] };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
