import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("rejects method, path, query, and body drift before authentication or upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCallCount = 0;
  globalThis.fetch = (async () => {
    upstreamCallCount += 1;
    return jsonResponse(workflowContextBindingsPayload());
  }) as typeof fetch;

  try {
    const queryResponse = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings?unexpected=true"
      ),
      routeContext()
    );
    const bodyResponse = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        { method: "POST", body: JSON.stringify({ unexpected: true }) }
      ),
      routeContext()
    );
    const methodResponse = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        { method: "POST" }
      ),
      routeContext()
    );
    const pathResponse = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/foreign/commits/commit%2Fid/workflow-bindings"
      ),
      routeContext()
    );

    for (const response of [queryResponse, bodyResponse, methodResponse, pathResponse]) {
      assert.equal(response.status, 400);
      assert.equal((await response.json()).error, "invalid_local_workflow_bindings_request");
      assert.equal(response.headers.get("cache-control"), "private, no-store");
    }
    assert.equal(upstreamCallCount, 0);
  } finally {
    restoreEnvironment();
  }
});

test("rejects malformed Bearer headers before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCallCount = 0;
  globalThis.fetch = (async () => {
    upstreamCallCount += 1;
    return jsonResponse(workflowContextBindingsPayload());
  }) as typeof fetch;

  try {
    for (const authorization of ["Bearer", "Bearer ", "Bearer token with-spaces", "Basic token"]) {
      const response = await GET(
        new Request(
          "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
          { headers: { authorization } }
        ),
        routeContext()
      );

      assert.equal(response.status, 401);
      assert.equal((await response.json()).error, "authentication_required");
    }
    assert.equal(upstreamCallCount, 0);
  } finally {
    restoreEnvironment();
  }
});

test("rejects missing request-scoped credentials", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCallCount = 0;
  globalThis.fetch = (async () => {
    upstreamCallCount += 1;
    return jsonResponse(workflowContextBindingsPayload());
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings"
      ),
      routeContext()
    );

    assert.equal(response.status, 401);
    assert.equal((await response.json()).error, "authentication_required");
    assert.equal(upstreamCallCount, 0);
  } finally {
    restoreEnvironment();
  }
});

test("fails closed when the upstream base URL is not configured", async () => {
  delete process.env.CONTEXTLAB_WEB_API_BASE_URL;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        { headers: { authorization: "Bearer request-token" } }
      ),
      routeContext()
    );

    assert.equal(response.status, 503);
    assert.equal((await response.json()).error, "contextlab_web_api_unavailable");
  } finally {
    restoreEnvironment();
  }
});

test("forwards the exact private scope with encoded path segments and no cookies", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(workflowContextBindingsPayload());
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        {
          headers: {
            authorization: "Bearer request-token",
            cookie: "session=ignored"
          }
        }
      ),
      routeContext("context/id", "commit/id")
    );

    assert.equal(response.status, 200);
    assert.equal((await response.json()).commit_id, "commit/id");
    assert.equal(
      requests[0]?.url,
      "http://upstream.contextlab.test/api/v1/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("accept"), "application/json");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("preserves upstream structured errors and retry guidance", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse(
      {
        error: "workflow_context_bindings_rate_limited",
        message: "Too many local workflow binding reads"
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
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        { headers: { authorization: "Bearer request-token" } }
      ),
      routeContext()
    );

    assert.equal(response.status, 429);
    assert.equal(response.headers.get("retry-after"), "30");
    assert.equal((await response.json()).error, "workflow_context_bindings_rate_limited");
  } finally {
    restoreEnvironment();
  }
});

test("preserves upstream authentication, authorization, and availability errors", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";

  try {
    for (const [status, error] of [
      [401, "workflow_context_bindings_authentication_required"],
      [403, "workflow_context_bindings_forbidden"],
      [503, "workflow_context_bindings_unavailable"]
    ] as const) {
      globalThis.fetch = (async () =>
        jsonResponse(
          {
            error,
            message: `upstream status ${status}`
          },
          { status }
        )) as typeof fetch;

      const response = await GET(
        new Request(
          "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
          { headers: { authorization: "Bearer request-token" } }
        ),
        routeContext()
      );

      assert.equal(response.status, status);
      assert.deepEqual(await response.json(), {
        error,
        message: `ContextLab local API request failed with status ${status}`
      });
      assert.equal(response.headers.get("cache-control"), "private, no-store");
    }
  } finally {
    restoreEnvironment();
  }
});

test("maps an upstream transport failure to a redacted Web API error", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () => {
    throw new Error("upstream connection details must not escape the BFF");
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        { headers: { authorization: "Bearer request-token" } }
      ),
      routeContext()
    );

    assert.equal(response.status, 502);
    assert.deepEqual(await response.json(), {
      error: "contextlab_web_api_error",
      message: "Unable to load local Workflow Context bindings"
    });
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("fails closed and redacts raw workflow fields from an invalid upstream payload", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse({
      schema_version: "contextlab.local-workflow-context-bindings.v1",
      context_id: "context/id",
      commit_id: "commit/id",
      bindings: [
        {
          binding_id: "binding/id",
          workflow_id: "workflow/id",
          workflow_revision: 1,
          node_count: 2,
          edge_count: 1,
          nodes: [{ id: "raw-node" }]
        }
      ]
    })) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        { headers: { authorization: "Bearer request-token" } }
      ),
      routeContext()
    );

    assert.equal(response.status, 502);
    const body = await response.text();
    assert.match(body, /Unable to load local Workflow Context bindings/);
    assert.doesNotMatch(body, /raw-node|nodes/);
  } finally {
    restoreEnvironment();
  }
});

test("fails closed and redacts an upstream scope mismatch", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    jsonResponse({
      ...workflowContextBindingsPayload(),
      context_id: "foreign-context"
    })) as typeof fetch;

  try {
    const response = await GET(
      new Request(
        "http://contextlab.test/api/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings",
        { headers: { authorization: "Bearer request-token" } }
      ),
      routeContext()
    );

    assert.equal(response.status, 502);
    const body = await response.text();
    assert.match(body, /Unable to load local Workflow Context bindings/);
    assert.doesNotMatch(body, /foreign-context/);
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

function routeContext(contextId = "context/id", commitId = "commit/id") {
  return { params: Promise.resolve({ contextId, commitId }) };
}

function workflowContextBindingsPayload() {
  return {
    schema_version: "contextlab.local-workflow-context-bindings.v1",
    context_id: "context/id",
    commit_id: "commit/id",
    bindings: [
      {
        binding_id: "binding/id",
        workflow_id: "workflow/id",
        workflow_revision: 1,
        node_count: 2,
        edge_count: 1
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
