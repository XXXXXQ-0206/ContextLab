import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./api/local/contexts/[contextId]/plugins/capabilities/route";

const contextId = "11111111-1111-4111-8111-111111111111";
const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;

test("plugin capability BFF rejects query/body drift and missing credentials before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let calls = 0;
  globalThis.fetch = (async () => {
    calls += 1;
    return jsonResponse(resourcePayload());
  }) as typeof fetch;
  try {
    const query = await GET(new Request(`http://contextlab.test/api/local/contexts/${contextId}/plugins/capabilities?x=1`), routeContext());
    const body = await GET(new Request(`http://contextlab.test/api/local/contexts/${contextId}/plugins/capabilities`, { method: "POST", body: "{}" }), routeContext());
    assert.equal(query.status, 400);
    assert.equal(body.status, 400);
    assert.equal((await query.json()).error, "invalid_local_plugin_capability_request");
    assert.equal((await body.json()).error, "invalid_local_plugin_capability_request");
    const missing = await GET(new Request(`http://contextlab.test/api/local/contexts/${contextId}/plugins/capabilities`), routeContext());
    assert.equal(missing.status, 401);
    assert.equal(calls, 0);
  } finally {
    restoreEnvironment();
  }
});

test("plugin capability BFF forwards bearer-only exact scope and enforces private no-store", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(resourcePayload());
  }) as typeof fetch;
  try {
    const response = await GET(new Request(`http://contextlab.test/api/local/contexts/${contextId}/plugins/capabilities`, {
      headers: { authorization: "Bearer request-token", cookie: "session=ignored" }
    }), routeContext());
    assert.equal(response.status, 200);
    assert.deepEqual(await response.json(), resourcePayload());
    assert.equal(requests[0]?.url, `http://upstream.contextlab.test/api/v1/local/contexts/${contextId}/plugins/capabilities`);
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("plugin capability BFF redacts upstream transport failures", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () => { throw new Error("private upstream details"); }) as typeof fetch;
  try {
    const response = await GET(new Request(`http://contextlab.test/api/local/contexts/${contextId}/plugins/capabilities`, {
      headers: { authorization: "Bearer request-token" }
    }), routeContext());
    assert.equal(response.status, 502);
    assert.deepEqual(await response.json(), {
      error: "contextlab_web_api_error",
      message: "Unable to load local Plugin/MCP capabilities"
    });
  } finally {
    restoreEnvironment();
  }
});

function routeContext() {
  return { params: Promise.resolve({ contextId }) };
}

function resourcePayload() {
  return {
    schema_version: "contextlab.local-plugin-capability-availability.v1",
    context_id: contextId,
    entries: []
  };
}

function jsonResponse(body: unknown, init?: ResponseInit): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
    ...init
  });
}

function restoreEnvironment(): void {
  if (originalApiBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalApiBaseUrl;
  globalThis.fetch = originalFetch;
}
