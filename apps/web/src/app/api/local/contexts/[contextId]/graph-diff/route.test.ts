import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("local graph diff BFF rejects invalid paths, parameters, and missing request-memory credentials before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCalls = 0;
  globalThis.fetch = (async () => {
    upstreamCalls += 1;
    return jsonResponse(graphDiff());
  }) as typeof fetch;

  try {
    const invalid = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${CONTEXT_ID}/graph-diff?original_commit_id=${ORIGINAL_COMMIT_ID}&revised_commit_id=${ORIGINAL_COMMIT_ID}`),
      routeContext()
    );
    const missingBearer = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${CONTEXT_ID}/graph-diff?original_commit_id=${ORIGINAL_COMMIT_ID}&revised_commit_id=${REVISED_COMMIT_ID}`),
      routeContext()
    );
    const invalidPath = await GET(request(), routeContext("not-a-uuid"));

    assert.equal(invalid.status, 400);
    assert.equal((await invalid.json()).error, "invalid_local_commit_graph_diff_request");
    assert.equal(missingBearer.status, 401);
    assert.equal((await missingBearer.json()).error, "authentication_required");
    assert.equal(invalidPath.status, 400);
    assert.equal((await invalidPath.json()).error, "invalid_local_commit_graph_diff_request");
    assert.equal(invalid.headers.get("cache-control"), "private, no-store");
    assert.equal(missingBearer.headers.get("cache-control"), "private, no-store");
    assert.equal(upstreamCalls, 0);
  } finally {
    restoreEnvironment();
  }
});

test("local graph diff BFF fails closed without an upstream", async () => {
  delete process.env.CONTEXTLAB_WEB_API_BASE_URL;

  try {
    const response = await GET(request(), routeContext());
    assert.equal(response.status, 503);
    assert.equal((await response.json()).error, "contextlab_web_api_unavailable");
  } finally {
    restoreEnvironment();
  }
});

test("local graph diff BFF forwards only the request-memory Bearer credential", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const calls: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    calls.push({ input: String(input), init });
    return jsonResponse(graphDiff());
  }) as typeof fetch;

  try {
    const response = await GET(request(CONTEXT_ID, { authorization: "Bearer request-token", cookie: "ignored=true" }), routeContext(CONTEXT_ID));
    assert.equal(response.status, 200);
    const payload = await response.json();
    assert.equal(payload.revised.commit_id, REVISED_COMMIT_ID);
    assert.equal(payload.pair_witness.project_id, "dddddddd-dddd-4ddd-8ddd-dddddddddddd");
    assert.equal(payload.pair_witness.baseline_commit_id, ORIGINAL_COMMIT_ID);
    assert.equal(payload.pair_witness.revised_commit_id, REVISED_COMMIT_ID);
    assert.equal(calls[0]?.input, `http://upstream.contextlab.test/api/v1/local/contexts/${CONTEXT_ID}/graph-diff?original_commit_id=${ORIGINAL_COMMIT_ID}&revised_commit_id=${REVISED_COMMIT_ID}`);
    const headers = new Headers(calls[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer request-token");
    assert.equal(headers.get("cookie"), null);
    assert.equal(calls[0]?.init?.credentials, "omit");
    assert.equal(calls[0]?.init?.cache, "no-store");
    assert.equal(calls[0]?.init?.method, "GET");
  } finally {
    restoreEnvironment();
  }
});

test("local graph diff BFF preserves structured upstream errors and redacts malformed or thrown failures", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () => jsonResponse({ error: "context_read_forbidden", message: "Context read denied" }, { status: 403 })) as typeof fetch;

  try {
    const denied = await GET(request(), routeContext());
    assert.equal(denied.status, 403);
    assert.deepEqual(await denied.json(), {
      error: "context_read_forbidden",
      message: "ContextLab local API request failed with status 403"
    });

    globalThis.fetch = (async () => new Response("gateway diagnostic", { status: 502 })) as typeof fetch;
    const malformed = await GET(request(), routeContext());
    assert.equal(malformed.status, 502);
    assert.deepEqual(await malformed.json(), {
      error: "contextlab_local_api_error",
      message: "ContextLab local API request failed with status 502"
    });

    globalThis.fetch = (async () => { throw new Error("upstream diagnostic"); }) as typeof fetch;
    const thrown = await GET(request(), routeContext());
    assert.equal(thrown.status, 502);
    assert.equal((await thrown.json()).error, "contextlab_web_api_error");
  } finally {
    restoreEnvironment();
  }
});

const CONTEXT_ID = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const ORIGINAL_COMMIT_ID = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const REVISED_COMMIT_ID = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";

function request(contextId = CONTEXT_ID, headers: HeadersInit = { authorization: "Bearer request-token" }) {
  return new Request(
    `http://contextlab.test/api/local/contexts/${encodeURIComponent(contextId)}/graph-diff?original_commit_id=${ORIGINAL_COMMIT_ID}&revised_commit_id=${REVISED_COMMIT_ID}`,
    { headers }
  );
}

function routeContext(contextId = CONTEXT_ID) {
  return { params: Promise.resolve({ contextId }) };
}

function graphDiff() {
  return {
    context_id: CONTEXT_ID,
    pair_witness: {
      schema_version: 1,
      project_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
      context_id: CONTEXT_ID,
      baseline_commit_id: ORIGINAL_COMMIT_ID,
      revised_commit_id: REVISED_COMMIT_ID
    },
    original: { commit_id: ORIGINAL_COMMIT_ID, captured_at: "2026-07-08T00:00:00Z", schema_version: 1 },
    revised: { commit_id: REVISED_COMMIT_ID, captured_at: "2026-07-09T00:00:00Z", schema_version: 1 },
    diff: { added_nodes: [], removed_nodes: [], modified_nodes: [], added_edges: [], removed_edges: [] }
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), { headers: { "content-type": "application/json" }, ...init });
}

function restoreEnvironment() {
  if (originalApiBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalApiBaseUrl;
  globalThis.fetch = originalFetch;
}
