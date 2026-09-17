import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const contextId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const runId = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";
const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("BFF rejects query, body, invalid UUIDs, and missing credentials before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let calls = 0;
  globalThis.fetch = (async () => {
    calls += 1;
    return jsonResponse(statusPayload());
  }) as typeof fetch;

  try {
    const query = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${contextId}/workflow/runs/${runId}/status?extra=1`),
      routeContext()
    );
    const body = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${contextId}/workflow/runs/${runId}/status`, {
        method: "POST",
        body: "{}"
      }),
      routeContext()
    );
    const invalid = await GET(
      new Request("http://contextlab.test/api/local/contexts/not-a-uuid/workflow/runs/not-a-uuid/status"),
      routeContext("not-a-uuid", "not-a-uuid")
    );
    const missing = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${contextId}/workflow/runs/${runId}/status`),
      routeContext()
    );

    assert.equal(query.status, 400);
    assert.equal(body.status, 400);
    assert.equal(invalid.status, 400);
    assert.equal(missing.status, 401);
    assert.equal(calls, 0);
    assert.equal(query.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("BFF forwards exact run scope with only the request-scoped bearer and private no-store transport", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(statusPayload());
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${contextId}/workflow/runs/${runId}/status`, {
        headers: { authorization: "Bearer request-token", cookie: "ignored=true" }
      }),
      routeContext()
    );

    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), statusPayload());
    assert.equal(
      requests[0]?.url,
      `http://upstream.contextlab.test/api/v1/local/contexts/${contextId}/workflow/runs/${runId}/status`
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

test("BFF preserves 503 unavailable and redacts other upstream failures", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  try {
    globalThis.fetch = (async () => jsonResponse(
      { error: "workflow_execution_status_unavailable", message: "unavailable" },
      { status: 503 }
    )) as typeof fetch;
    const unavailable = await GET(request(), routeContext());
    assert.equal(unavailable.status, 503);
    assert.deepEqual(await unavailable.json(), {
      error: "workflow_execution_status_unavailable",
      message: "ContextLab local API request failed with status 503"
    });

    globalThis.fetch = (async () => { throw new Error("private upstream diagnostic"); }) as typeof fetch;
    const thrown = await GET(request(), routeContext());
    assert.equal(thrown.status, 502);
    assert.deepEqual(await thrown.json(), {
      error: "contextlab_web_api_error",
      message: "Unable to load local workflow execution status"
    });
  } finally {
    restoreEnvironment();
  }
});

test("BFF turns malformed upstream JSON into a redacted response error", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () => jsonResponse({ ...statusPayload(), event_count: "six" })) as typeof fetch;
  try {
    const response = await GET(request(), routeContext());
    assert.equal(response.status, 502);
    assert.deepEqual(await response.json(), {
      error: "invalid_local_workflow_execution_status_response",
      message: "Local workflow execution status response is invalid"
    });
  } finally {
    restoreEnvironment();
  }
});

function request(): Request {
  return new Request(
    `http://contextlab.test/api/local/contexts/${contextId}/workflow/runs/${runId}/status`,
    { headers: { authorization: "Bearer request-token" } }
  );
}

function routeContext(requestedContextId = contextId, requestedRunId = runId) {
  return { params: Promise.resolve({ contextId: requestedContextId, runId: requestedRunId }) };
}

function statusPayload() {
  return {
    schema_version: "contextlab.local-workflow-execution-status.v1",
    context_id: contextId,
    context_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    binding_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    workflow_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
    workflow_revision: 7,
    run_id: runId,
    replay_of: null,
    run_state: "failed",
    event_count: 6,
    last_event_sequence: 6,
    capability_snapshot_digest: "sha256:0123456789abcdef",
    node_status_counts: { pending: 0, running: 0, succeeded: 2, failed: 1, blocked: 3 }
  } as const;
}

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}

function restoreEnvironment(): void {
  if (originalApiBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalApiBaseUrl;
  globalThis.fetch = originalFetch;
}
