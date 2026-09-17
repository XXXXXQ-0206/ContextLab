import assert from "node:assert/strict";
import test from "node:test";
import { POST } from "./route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("forwards the exact execution scope with bearer-only no-store transport", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(executionResponse(), { status: 201 });
  }) as typeof fetch;

  try {
    const scope = routeScope();
    const result = await POST(
      new Request(
        "http://contextlab.test/api/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-executions",
        {
          method: "POST",
          headers: {
            authorization: "Bearer request-token",
            cookie: "ignored=session",
            "content-type": "application/json",
            "idempotency-key": "execution-1"
          },
          body: JSON.stringify(executionRequest())
        }
      ),
      scope
    );

    assert.equal(result.status, 201);
    assert.equal(result.headers.get("cache-control"), "private, no-store");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-executions"
    );
    const headers = new Headers(requests[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer request-token");
    assert.equal(headers.get("idempotency-key"), "execution-1");
    assert.equal(headers.get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.deepEqual(JSON.parse(String(requests[0]?.init?.body)), executionRequest());
  } finally {
    restoreEnvironment();
  }
});

test("rejects path, body, bearer, and idempotency drift before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCalls = 0;
  globalThis.fetch = (async () => {
    upstreamCalls += 1;
    return jsonResponse(executionResponse(), { status: 201 });
  }) as typeof fetch;

  try {
    const cases = [
      new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions?unexpected=true", {
        method: "POST",
        headers: { authorization: "Bearer token", "idempotency-key": "key" },
        body: JSON.stringify(executionRequest())
      }),
      new Request("http://contextlab.test/api/local/projects/other/contexts/context/commits/commit/benchmark-executions", {
        method: "POST",
        headers: { authorization: "Bearer token", "idempotency-key": "key" },
        body: JSON.stringify(executionRequest())
      }),
      new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
        method: "POST",
        headers: { authorization: "Basic token", "idempotency-key": "key" },
        body: JSON.stringify(executionRequest())
      }),
      new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
        method: "POST",
        headers: { authorization: "Bearer token" },
        body: JSON.stringify(executionRequest())
      }),
      new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
        method: "POST",
        headers: { authorization: "Bearer token", "idempotency-key": "key" },
        body: JSON.stringify({ ...executionRequest(), extra: true })
      })
    ];

    const statuses = [];
    for (const request of cases) {
      statuses.push((await POST(request, routeScope("project", "context", "commit"))).status);
    }
    assert.deepEqual(statuses, [400, 400, 401, 400, 400]);
    assert.equal(upstreamCalls, 0);
  } finally {
    restoreEnvironment();
  }
});

test("fails closed on malformed or scope-drifting success payloads", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const payloads = [
    { ...executionResponse(), raw_cases: [{ input: "private" }] },
    { ...executionResponse(), context_id: "wrong-context" },
    { ...executionResponse(), schema_version: "unknown" }
  ];

  try {
    for (const payload of payloads) {
      globalThis.fetch = (async () => jsonResponse(payload, { status: 201 })) as typeof fetch;
      const result = await POST(
        new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
          method: "POST",
          headers: { authorization: "Bearer token", "idempotency-key": "key" },
          body: JSON.stringify(executionRequest())
        }),
        routeScope("project", "context", "commit")
      );
      assert.equal(result.status, 502);
      assert.equal(result.headers.get("cache-control"), "private, no-store");
      assert.doesNotMatch(await result.text(), /private|raw_cases|wrong-context/);
    }
  } finally {
    restoreEnvironment();
  }
});

test("rejects unsupported temperatures before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const upstreamCalls: Request[] = [];
  globalThis.fetch = (async (input, init) => {
    upstreamCalls.push(new Request(input, init));
    return jsonResponse(executionResponse(), { status: 201 });
  }) as typeof fetch;

  try {
    for (const temperature of [-0.01, 2.01, Number.NaN, Number.POSITIVE_INFINITY]) {
      const result = await POST(
        new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
          method: "POST",
          headers: { authorization: "Bearer token", "idempotency-key": "key" },
          body: JSON.stringify({ ...executionRequest(), temperature })
        }),
        routeScope("project", "context", "commit")
      );
      assert.equal(result.status, 400);
    }
    assert.equal(upstreamCalls.length, 0);
  } finally {
    restoreEnvironment();
  }
});

test("fails closed on empty, duplicate, or unordered dataset ids", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const upstreamCalls: Request[] = [];

  try {
    for (const dataset_ids of [[], ["dataset/a", "dataset/a"], ["dataset/b", "dataset/a"]]) {
      globalThis.fetch = (async (input, init) => {
        upstreamCalls.push(new Request(input, init));
        return jsonResponse({ ...executionResponse(), dataset_ids }, { status: 201 });
      }) as typeof fetch;
      const result = await POST(
        new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
          method: "POST",
          headers: { authorization: "Bearer token", "idempotency-key": "key" },
          body: JSON.stringify(executionRequest())
        }),
        routeScope("project", "context", "commit")
      );
      assert.equal(result.status, 502);
      assert.doesNotMatch(await result.text(), /dataset\/a|dataset\/b/);
    }
    assert.equal(upstreamCalls.length, 3);
  } finally {
    restoreEnvironment();
  }
});

test("preserves typed upstream errors without forwarding private fields", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "benchmark_execution_rate_limited",
        message: "Too many local benchmark executions",
        internal_trace: "must-not-cross",
        raw_output: "private"
      },
      { status: 429, headers: { "retry-after": "30" } }
    )) as typeof fetch;

  try {
    const result = await POST(
      new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
        method: "POST",
        headers: { authorization: "Bearer token", "idempotency-key": "key" },
        body: JSON.stringify(executionRequest())
      }),
      routeScope("project", "context", "commit")
    );
    assert.equal(result.status, 429);
    assert.equal(result.headers.get("cache-control"), "private, no-store");
    assert.equal(result.headers.get("retry-after"), "30");
    assert.deepEqual(await result.json(), {
      error: "benchmark_execution_rate_limited",
      message: "The local benchmark execution rate limit is active / 本地 Benchmark 执行速率限制已生效。"
    });
  } finally {
    restoreEnvironment();
  }
});

test("maps upstream transport and malformed error failures to a redacted 502", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  const upstreamFailures = [
    async () => { throw new Error("host and token must not escape"); },
    async () => jsonResponse({ private_details: "must-not-cross" }, { status: 503 }),
    async () => new Response("not json", { status: 503 })
  ];

  try {
    for (const failure of upstreamFailures) {
      globalThis.fetch = failure as typeof fetch;
      const result = await POST(
        new Request("http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-executions", {
          method: "POST",
          headers: { authorization: "Bearer token", "idempotency-key": "key" },
          body: JSON.stringify(executionRequest())
        }),
        routeScope("project", "context", "commit")
      );
      assert.equal(result.status, 502);
      assert.deepEqual(await result.json(), {
        error: "contextlab_web_api_error",
        message: "Unable to execute the local benchmark"
      });
    }
  } finally {
    restoreEnvironment();
  }
});

function executionRequest() {
  return {
    schema_version: 1,
    binding_id: "binding/id",
    decision_id: "decision/id",
    model_version: "model-v1",
    temperature: 0.2,
    evaluator_key: "deterministic",
    evaluator_version: "v1"
  };
}

function executionResponse() {
  return {
    schema_version: "contextlab.local-benchmark-execution.v1",
    disposition: "created",
    projection_disposition: "created",
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    binding_id: "binding/id",
    decision_id: "decision/id",
    suite_id: "suite/id",
    dataset_ids: ["dataset/id"],
    cohort_id: "cohort/id"
  };
}

function routeScope(projectId = "project/id", contextId = "context/id", commitId = "commit/id") {
  return { params: Promise.resolve({ projectId, contextId, commitId }) };
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
