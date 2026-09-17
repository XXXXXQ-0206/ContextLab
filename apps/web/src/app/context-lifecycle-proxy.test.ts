import assert from "node:assert/strict";
import test from "node:test";
import {
  proxyLocalBenchmarkDecisionDiff,
  proxyLocalBenchmarkDecision,
  proxyLocalBenchmarkDecisionRunDetails,
  proxyLocalComponentLifecycleCommit,
  proxyLocalContextLifecycleState,
  proxyLocalWorkflowCapabilityStatus
} from "./context-lifecycle-proxy";
import { POST as postComponentLifecycleCommit } from "./api/local/contexts/[contextId]/component-lifecycle-commits/route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalLocalLifecycleEnabled = process.env.CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE;
const originalFetch = globalThis.fetch;
const lifecycleContextId = "11111111-1111-4111-8111-111111111111";
const lifecycleCommitA = "22222222-2222-4222-8222-222222222222";
const lifecycleCommitB = "33333333-3333-4333-8333-333333333333";

test("local lifecycle proxy rejects missing request-scoped credentials", async () => {
  delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  process.env.CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE = "true";

  try {
    const state = await proxyLocalContextLifecycleState(
      new Request("http://contextlab.test/api/local/contexts/context-a/commits/commit-a/lifecycle-state"),
      "context-a",
      "commit-a"
    );
    const mutation = await proxyLocalComponentLifecycleCommit(
      new Request("http://contextlab.test/api/local/contexts/context-a/component-lifecycle-commits", {
        method: "POST",
        body: JSON.stringify({})
      }),
      "context-a"
    );

    assert.equal(state.status, 401);
    assert.equal((await state.json()).error, "authentication_required");
    assert.equal(mutation.status, 401);
    assert.equal((await mutation.json()).error, "authentication_required");
  } finally {
    restoreEnvironment();
  }
});

test("local lifecycle commit BFF fails closed without explicit local development enablement", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  delete process.env.CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE;
  let upstreamCallCount = 0;
  globalThis.fetch = (async () => {
    upstreamCallCount += 1;
    return jsonResponse({});
  }) as typeof fetch;

  try {
    const response = await postComponentLifecycleCommit(
      new Request("http://contextlab.test/api/local/contexts/context-a/component-lifecycle-commits", {
        method: "POST",
        headers: {
          authorization: "Bearer request-token",
          "idempotency-key": "request-key",
          "content-type": "application/json"
        },
        body: JSON.stringify({
          branch_name: "main",
          expected_head_commit_id: "commit-a",
          message: "Create component",
          operation: {
            kind: "create",
            component: {
              component_id: "component-a",
              kind: "prompt",
              name: "Instruction",
              metadata: {},
              content: "Return concise results."
            }
          }
        })
      }),
      { params: Promise.resolve({ contextId: "context-a" }) }
    );

    assert.equal(response.status, 403);
    assert.equal((await response.json()).error, "local_lifecycle_disabled");
    assert.equal(upstreamCallCount, 0);
  } finally {
    restoreEnvironment();
  }
});

test("local lifecycle read BFF forwards the exact Context commit scope privately", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      schema_version: "contextlab.local-context-lifecycle-state.v1",
      context_id: lifecycleContextId,
      commit_id: lifecycleCommitA,
      metadata: null,
      components: [],
      graph_snapshot: {
        project_id: "55555555-5555-4555-8555-555555555555",
        context_id: lifecycleContextId,
        commit_id: lifecycleCommitA,
        graph: { nodes: {}, edges: [] },
        captured_at: "2026-08-02T00:00:00Z",
        schema_version: 1
      }
    });
  }) as typeof fetch;

  try {
    const response = await proxyLocalContextLifecycleState(
      new Request(
        `http://contextlab.test/api/local/contexts/${lifecycleContextId}/commits/${lifecycleCommitA}/lifecycle-state`,
        { headers: { authorization: "Bearer request-token", cookie: "session=ignored" } }
      ),
      lifecycleContextId,
      lifecycleCommitA
    );

    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), {
      schema_version: "contextlab.local-context-lifecycle-state.v1",
      context_id: lifecycleContextId,
      commit_id: lifecycleCommitA,
      metadata: null,
      components: [],
      graph_snapshot: {
        project_id: "55555555-5555-4555-8555-555555555555",
        context_id: lifecycleContextId,
        commit_id: lifecycleCommitA,
        graph: { nodes: {}, edges: [] },
        captured_at: "2026-08-02T00:00:00Z",
        schema_version: 1
      }
    });
    assert.equal(
      requests[0]?.url,
      `http://upstream.contextlab.test/api/v1/local/contexts/${lifecycleContextId}/commits/${lifecycleCommitA}/lifecycle-state`
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("local lifecycle read BFF returns a private 503 when the upstream is unavailable", async () => {
  delete process.env.CONTEXTLAB_WEB_API_BASE_URL;

  try {
    const response = await proxyLocalContextLifecycleState(
      new Request("http://contextlab.test/api/local/contexts/context-a/commits/commit-a/lifecycle-state", {
        headers: { authorization: "Bearer request-token" }
      }),
      "context-a",
      "commit-a"
    );

    assert.equal(response.status, 503);
    assert.equal((await response.json()).error, "contextlab_web_api_unavailable");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("local lifecycle proxy forwards descriptor-only commands with request-scoped credentials", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  process.env.CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE = "true";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      schema_version: "contextlab.local-component-lifecycle-commit.v1",
      disposition: "created",
      commit_id: lifecycleCommitB,
      snapshot: {
        project_id: "55555555-5555-4555-8555-555555555555",
        context_id: lifecycleContextId,
        commit_id: lifecycleCommitB,
        graph: { nodes: {}, edges: [] },
        captured_at: "2026-07-15T00:00:00Z",
        schema_version: 1
      }
    });
  }) as typeof fetch;

  try {
    const response = await proxyLocalComponentLifecycleCommit(
      new Request(`http://contextlab.test/api/local/contexts/${lifecycleContextId}/component-lifecycle-commits`, {
        method: "POST",
        headers: {
          authorization: "Bearer request-token",
          "idempotency-key": "request-key",
          "content-type": "application/json"
        },
        body: JSON.stringify({
          branch_name: "main",
          expected_head_commit_id: "commit-a",
          message: "Correct descriptor",
          operation: {
            kind: "update_descriptor",
            component_id: "component-a",
            name: "Instruction / 操作说明",
            metadata: { locale: "zh-CN" }
          }
        })
      }),
      lifecycleContextId
    );

    assert.equal(response.status, 201);
    assert.equal((await response.json()).commit_id, lifecycleCommitB);
    assert.equal(
      requests[0]?.url,
      `http://upstream.contextlab.test/api/v1/local/contexts/${lifecycleContextId}/component-lifecycle-commits`
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("idempotency-key"), "request-key");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
    assert.deepEqual(JSON.parse(String(requests[0]?.init?.body)), {
      branch_name: "main",
      expected_head_commit_id: "commit-a",
      message: "Correct descriptor",
      operation: {
        kind: "update_descriptor",
        component_id: "component-a",
        name: "Instruction / 操作说明",
        metadata: { locale: "zh-CN" }
      }
    });
  } finally {
    restoreEnvironment();
  }
});

test("local lifecycle proxy forwards unborn Context initialization without cookies", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  process.env.CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE = "true";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      schema_version: "contextlab.local-component-lifecycle-commit.v1",
      disposition: "created",
      commit_id: lifecycleCommitA,
      snapshot: {
        project_id: "55555555-5555-4555-8555-555555555555",
        context_id: lifecycleContextId,
        commit_id: lifecycleCommitA,
        graph: { nodes: {}, edges: [] },
        captured_at: "2026-07-18T00:00:00Z",
        schema_version: 1
      }
    });
  }) as typeof fetch;

  try {
    const response = await proxyLocalComponentLifecycleCommit(
      new Request(`http://contextlab.test/api/local/contexts/${lifecycleContextId}/component-lifecycle-commits`, {
        method: "POST",
        headers: {
          authorization: "Bearer request-token",
          "idempotency-key": "initialize-key",
          "content-type": "application/json"
        },
        body: JSON.stringify({
          branch_name: "main",
          expected_head_commit_id: null,
          message: "Initialize Context lifecycle",
          operation: { kind: "initialize" }
        })
      }),
      lifecycleContextId
    );

    assert.equal(response.status, 201);
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("idempotency-key"), "initialize-key");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.deepEqual(JSON.parse(String(requests[0]?.init?.body)), {
      branch_name: "main",
      expected_head_commit_id: null,
      message: "Initialize Context lifecycle",
      operation: { kind: "initialize" }
    });
  } finally {
    restoreEnvironment();
  }
});

test("local benchmark proxy forwards a redacted exact-scope read without cookies or caching", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      project_id: "project/id",
      context_id: "context/id",
      commit_id: "commit/id",
      decision_id: "decision/id",
      suite_id: "suite/id",
      dataset_ids: [],
      definition: {
        suite: {
          id: "suite/id",
          name: "Release gate",
          thresholds: []
        },
        datasets: []
      },
      run_ids: [],
      comparability: {
        evaluator_key: "quality",
        evaluator_version: "1.0.0",
        fingerprint: "f".repeat(64)
      },
      evidence_digest: "d".repeat(64),
      status: "passed",
      recorded_at: "2026-07-18T00:00:00Z",
      metrics: []
    });
  }) as typeof fetch;

  try {
    const response = await proxyLocalBenchmarkDecision(
      new Request(
        "http://contextlab.test/api/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid",
        { headers: { authorization: "Bearer request-token", cookie: "session=ignored" } }
      ),
      "project/id",
      "context/id",
      "commit/id",
      "decision/id"
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).decision_id, "decision/id");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("local workflow capability proxy forwards only request-scoped credentials and private V1 status", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      schema_version: "contextlab.local-workflow-capability-status.v1",
      capability: "workflow",
      enabled: false,
      availability: "unavailable",
      reason: "shared_integration_not_registered"
    });
  }) as typeof fetch;

  try {
    const response = await proxyLocalWorkflowCapabilityStatus(
      new Request("http://contextlab.test/api/local/contexts/context%2Fid/workflow/capability-status", {
        headers: { authorization: "Bearer request-token", cookie: "session=ignored" }
      }),
      "context/id"
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).availability, "unavailable");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/contexts/context%2Fid/workflow/capability-status"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("local benchmark run-details proxy forwards only the exact private scope", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      project_id: "project/id",
      context_id: "context/id",
      commit_id: "commit/id",
      decision_id: "decision/id",
      runs: [
        {
          run_id: "run/id",
          model_version: "model-a",
          temperature: 0.2,
          metrics: [{ metric: "accuracy", value: 0.95 }],
          executed_at: "2026-07-18T00:00:00Z"
        }
      ]
    });
  }) as typeof fetch;

  try {
    const response = await proxyLocalBenchmarkDecisionRunDetails(
      new Request(
        "http://contextlab.test/api/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid/run-details",
        { headers: { authorization: "Bearer request-token", cookie: "session=ignored" } }
      ),
      "project/id",
      "context/id",
      "commit/id",
      "decision/id"
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).runs[0].run_id, "run/id");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions/decision%2Fid/run-details"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("local benchmark diff proxy forwards both exact scopes without cookies or caching", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse({
      project_id: "project/id",
      context_id: "context/id",
      baseline: { commit_id: "baseline/commit", decision_id: "baseline/decision" },
      revised: { commit_id: "revised/commit", decision_id: "revised/decision" },
      status_change: null,
      metric_changes: []
    });
  }) as typeof fetch;

  try {
    const response = await proxyLocalBenchmarkDecisionDiff(
      new Request("http://contextlab.test/api/local/projects/project%2Fid/contexts/context%2Fid/benchmark-decision-diffs", {
        headers: { authorization: "Bearer request-token", cookie: "session=ignored" }
      }),
      "project/id",
      "context/id",
      { commit_id: "baseline/commit", decision_id: "baseline/decision" },
      { commit_id: "revised/commit", decision_id: "revised/decision" }
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).baseline.commit_id, "baseline/commit");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/projects/project%2Fid/contexts/context%2Fid/benchmark-decision-diffs?baseline_commit_id=baseline%2Fcommit&baseline_decision_id=baseline%2Fdecision&revised_commit_id=revised%2Fcommit&revised_decision_id=revised%2Fdecision"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("local lifecycle proxy preserves retry guidance from protected rate limits", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "context_lifecycle_rate_limited",
        message: "Too many local lifecycle requests"
      },
      {
        status: 429,
        headers: {
          "content-type": "application/json",
          "retry-after": "30"
        }
      }
    )) as typeof fetch;

  try {
    const response = await proxyLocalContextLifecycleState(
      new Request("http://contextlab.test/api/local/contexts/context-a/commits/commit-a/lifecycle-state", {
        headers: { authorization: "Bearer request-token" }
      }),
      "context-a",
      "commit-a"
    );

    assert.equal(response.status, 429);
    assert.equal(response.headers.get("retry-after"), "30");
    assert.equal((await response.json()).error, "context_lifecycle_rate_limited");
  } finally {
    restoreEnvironment();
  }
});

function restoreEnvironment() {
  if (originalApiBaseUrl === undefined) {
    delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  } else {
    process.env.CONTEXTLAB_WEB_API_BASE_URL = originalApiBaseUrl;
  }

  if (originalLocalLifecycleEnabled === undefined) {
    delete process.env.CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE;
  } else {
    process.env.CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE = originalLocalLifecycleEnabled;
  }

  globalThis.fetch = originalFetch;
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
