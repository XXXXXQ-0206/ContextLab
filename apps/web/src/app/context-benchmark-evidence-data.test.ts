import assert from "node:assert/strict";
import test from "node:test";
import {
  LocalBenchmarkEvidenceProxyError,
  loadLocalBenchmarkDecision,
  loadLocalBenchmarkDecisionRunDetails
} from "./context-benchmark-evidence-data";

const originalFetch = globalThis.fetch;

test("benchmark evidence data client uses the exact same-origin scope without cookies", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse(decisionPayload());
  }) as typeof fetch;

  try {
    const decision = await loadLocalBenchmarkDecision(
      "project/id",
      "context/id",
      "commit/id",
      "decision/id",
      "request-token"
    );

    assert.equal(decision.decision_id, "decision/id");
    assert.equal(
      requests[0]?.input,
      "/api/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("benchmark evidence data client fetches only the exact private run-details scope without cookies", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse(runDetailsPayload());
  }) as typeof fetch;

  try {
    const details = await loadLocalBenchmarkDecisionRunDetails(
      "project/id",
      "context/id",
      "commit/id",
      "decision/id",
      "request-token"
    );

    assert.deepEqual(details.runs.map((run) => run.run_id), ["run/b", "run/a"]);
    assert.equal(
      requests[0]?.input,
      "/api/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid/run-details"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("benchmark evidence data client preserves structured forbidden responses", async () => {
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "context_read_forbidden",
        message: "private upstream database path: /srv/contextlab/prod"
      },
      { status: 403 }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBenchmarkDecision("project", "context", "commit", "decision", "request-token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalBenchmarkEvidenceProxyError);
        assert.equal(error.status, 403);
        assert.equal(error.body.error, "context_read_forbidden");
        assert.equal(
          error.body.message,
          "Benchmark evidence request failed with status 403 / Benchmark 证据请求失败，状态 403"
        );
        assert.equal(error.message, error.body.message);
        assert.doesNotMatch(error.body.message, /private upstream database path/);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("benchmark evidence run-details errors redact upstream diagnostics", async () => {
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "benchmark_run_unavailable",
        message: "private evaluator trace: evaluator_secret=redacted-at-source"
      },
      { status: 503 }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBenchmarkDecisionRunDetails("project", "context", "commit", "decision", "request-token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalBenchmarkEvidenceProxyError);
        assert.equal(
          error.body.message,
          "Benchmark evidence request failed with status 503 / Benchmark 证据请求失败，状态 503"
        );
        assert.doesNotMatch(error.message, /private evaluator trace|evaluator_secret/);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("benchmark evidence data client rejects raw payload keys from the same-origin BFF", async () => {
  globalThis.fetch = (async () =>
    jsonResponse({
      ...decisionPayload(),
      cases: [{ input: "must not cross the browser boundary" }]
    })) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalBenchmarkDecision("project", "context", "commit", "decision", "request-token"),
      TypeError
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function decisionPayload() {
  return {
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    decision_id: "decision/id",
    suite_id: "suite/id",
    dataset_ids: ["dataset/id"],
    definition: {
      suite: {
        id: "suite/id",
        name: "Release gate",
        thresholds: []
      },
      datasets: [{ id: "dataset/id", name: "Release dataset", case_count: 1 }]
    },
    run_ids: ["run/id"],
    comparability: {
      evaluator_key: "quality",
      evaluator_version: "1.0.0",
      fingerprint: "f".repeat(64)
    },
    evidence_digest: "d".repeat(64),
    status: "passed",
    recorded_at: "2026-07-18T00:00:00Z",
    metrics: []
  };
}

function runDetailsPayload() {
  return {
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    decision_id: "decision/id",
    runs: [
      {
        run_id: "run/b",
        model_version: "model-v2",
        temperature: 0.7,
        metrics: [{ metric: "accuracy", value: 0.94 }],
        executed_at: "2026-07-18T00:01:00Z"
      },
      {
        run_id: "run/a",
        model_version: "model-v1",
        temperature: 0,
        metrics: [{ metric: "latency_ms", value: 320 }],
        executed_at: "2026-07-18T00:00:00Z"
      }
    ]
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
