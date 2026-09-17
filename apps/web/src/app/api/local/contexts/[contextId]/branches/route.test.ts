import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;
const contextId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const headCommitId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

test("branch-head BFF rejects invalid input and missing request-memory credentials", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let calls = 0;
  globalThis.fetch = (async () => {
    calls += 1;
    return jsonResponse(branchHeads());
  }) as typeof fetch;

  try {
    const invalid = await GET(new Request("http://contextlab.test/api/local/contexts/not-a-uuid/branches"), routeContext("not-a-uuid"));
    const missing = await GET(new Request(`http://contextlab.test/api/local/contexts/${contextId}/branches`), routeContext());
    assert.equal(invalid.status, 400);
    assert.equal(missing.status, 401);
    assert.equal(calls, 0);
    assert.equal(invalid.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("branch-head BFF forwards only the request-memory Bearer and preserves the safe resource", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const calls: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    calls.push({ input: String(input), init });
    return jsonResponse(branchHeads());
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${contextId}/branches`, {
        headers: { authorization: "Bearer request-token", cookie: "ignored=true" }
      }),
      routeContext()
    );
    assert.equal(response.status, 200);
    assert.equal((await response.json()).context_id, contextId);
    assert.equal(calls[0]?.input, `http://upstream.contextlab.test/api/v1/local/contexts/${contextId}/branches`);
    const headers = new Headers(calls[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer request-token");
    assert.equal(headers.get("cookie"), null);
    assert.equal(calls[0]?.init?.credentials, "omit");
    assert.equal(calls[0]?.init?.cache, "no-store");
  } finally {
    restoreEnvironment();
  }
});

test("branch-head BFF redacts untrusted upstream errors", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () => new Response(JSON.stringify({ error: "internal", message: "secret" }), { status: 503 })) as typeof fetch;
  try {
    const response = await GET(
      new Request(`http://contextlab.test/api/local/contexts/${contextId}/branches`, { headers: { authorization: "Bearer token" } }),
      routeContext()
    );
    assert.equal(response.status, 503);
    const body = await response.json();
    assert.equal(body.error, "contextlab_local_api_error");
    assert.doesNotMatch(body.message, /secret/);
  } finally {
    restoreEnvironment();
  }
});

const routeContext = (value = contextId) => ({ params: Promise.resolve({ contextId: value }) });

function branchHeads() {
  return {
    schema_version: "contextlab.local-context-branch-heads.v1",
    context_id: contextId,
    branches: [{ branch_name: "main", head_commit_id: headCommitId, revision: 1 }]
  };
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), { headers: { "content-type": "application/json" } });
}

function restoreEnvironment() {
  if (originalApiBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalApiBaseUrl;
  globalThis.fetch = originalFetch;
}
