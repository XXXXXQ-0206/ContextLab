import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("forwards the exact private decision-list scope without cookies", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(decisionListPayload());
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions",
        { headers: { authorization: "Bearer request-token", cookie: "ignored=session" } }
      ),
      routeContext("project/id", "context/id", "commit/id")
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).commit_id, "commit/id");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/projects/project%2Fid/contexts/context%2Fid/commits/commit%2Fid/benchmark-decisions"
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

test("rejects query drift and missing credentials before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCalls = 0;
  globalThis.fetch = (async () => {
    upstreamCalls += 1;
    return jsonResponse(decisionListPayload());
  }) as typeof fetch;

  try {
    const query = await GET(
      new Request(
        "http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-decisions?unexpected=true",
        { headers: { authorization: "Bearer token" } }
      ),
      routeContext()
    );
    assert.equal(query.status, 400);

    const missingAuth = await GET(
      new Request(
        "http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-decisions"
      ),
      routeContext()
    );
    assert.equal(missingAuth.status, 401);
    assert.equal(upstreamCalls, 0);
  } finally {
    restoreEnvironment();
  }
});

test("preserves structured upstream failures without forwarding extra response data", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "benchmark_decision_discovery_rate_limited",
        message: "Too many local benchmark decision discovery reads",
        internal_trace: "must-not-cross-the-bff",
        raw_decisions: [{ decision_id: "private-decision" }]
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
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-decisions",
        { headers: { authorization: "Bearer request-token" } }
      ),
      routeContext("project", "context", "commit")
    );

    assert.equal(response.status, 429);
    assert.equal(response.headers.get("cache-control"), "private, no-store");
    assert.equal(response.headers.get("retry-after"), "30");
    assert.deepEqual(await response.json(), {
      error: "benchmark_decision_discovery_rate_limited",
      message: "Too many local benchmark decision discovery reads"
    });
  } finally {
    restoreEnvironment();
  }
});

test("maps transport failures to a redacted no-store BFF error", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () => {
    throw new Error("upstream host and credential details must not escape the BFF");
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/projects/project/contexts/context/commits/commit/benchmark-decisions",
        { headers: { authorization: "Bearer request-token" } }
      ),
      routeContext("project", "context", "commit")
    );

    assert.equal(response.status, 502);
    assert.equal(response.headers.get("cache-control"), "private, no-store");
    assert.deepEqual(await response.json(), {
      error: "contextlab_web_api_error",
      message: "Unable to load local benchmark decision discovery"
    });
  } finally {
    restoreEnvironment();
  }
});

function routeContext(projectId = "project/id", contextId = "context/id", commitId = "commit/id") {
  return { params: Promise.resolve({ projectId, contextId, commitId }) };
}

function decisionListPayload() {
  return {
    schema_version: "contextlab.local-benchmark-decision-list.v1",
    project_id: "project/id",
    context_id: "context/id",
    commit_id: "commit/id",
    decisions: [
      {
        decision_id: "decision/id",
        suite: { id: "suite/id", name: "Release gate" },
        datasets: [{ id: "dataset/id", name: "Release dataset", case_count: 1 }],
        status: "passed",
        recorded_at: "2026-07-18T02:00:00Z",
        run_count: 1
      }
    ]
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
