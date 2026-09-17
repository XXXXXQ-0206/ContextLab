import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const originalApiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
const originalFetch = globalThis.fetch;
const scope = {
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  leftCommitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  rightCommitId: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
} as const;

test("BFF forwards exact merge scope with bearer-only no-store transport", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test/";
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input, init) => {
    requests.push({ url: String(input), init });
    return jsonResponse(reviewPayload());
  }) as typeof fetch;
  try {
    const response = await GET(
      requestFor(`?left_commit_id=${scope.leftCommitId}&right_commit_id=${scope.rightCommitId}`, {
        authorization: "Bearer request-token",
        cookie: "session=must-not-cross-the-bff"
      }),
      routeContext()
    );
    assert.equal(response.status, 200);
    assert.equal((await response.json()).base_scope.commit_id, "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee");
    assert.equal(
      requests[0]?.url,
      `http://upstream.contextlab.test/api/v1/local/projects/${scope.projectId}/contexts/${scope.contextId}/merge-review?left_commit_id=${scope.leftCommitId}&right_commit_id=${scope.rightCommitId}`
    );
    const headers = new Headers(requests[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer request-token");
    assert.equal(headers.get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(response.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnvironment();
  }
});

test("BFF rejects missing, unknown, blank, and duplicate query scope before upstream access", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  let upstreamCalls = 0;
  globalThis.fetch = (async () => {
    upstreamCalls += 1;
    return jsonResponse(reviewPayload());
  }) as typeof fetch;
  try {
    for (const query of [
      "",
      `?left_commit_id=${scope.leftCommitId}`,
      `?right_commit_id=${scope.rightCommitId}`,
      `?left_commit_id=&right_commit_id=${scope.rightCommitId}`,
      `?left_commit_id=${scope.leftCommitId}&right_commit_id=`,
      `?left_commit_id=${scope.leftCommitId}&right_commit_id=${scope.rightCommitId}&unexpected=true`,
      `?left_commit_id=${scope.leftCommitId}&left_commit_id=${scope.rightCommitId}&right_commit_id=${scope.rightCommitId}`
    ]) {
      const response = await GET(requestFor(query, { authorization: "Bearer request-token" }), routeContext());
      assert.equal(response.status, 400);
      assert.deepEqual(await response.json(), {
        error: "invalid_local_context_merge_review_request",
        message: "Exact left_commit_id and right_commit_id are required"
      });
    }
    assert.equal(upstreamCalls, 0);
  } finally {
    restoreEnvironment();
  }
});

test("BFF redacts upstream error messages and transport details", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  globalThis.fetch = (async () => jsonResponse(
    { error: "upstream_internal", message: "private prompt and stack" },
    { status: 502, headers: { "retry-after": "4" } }
  )) as typeof fetch;
  try {
    const response = await GET(
      requestFor(`?left_commit_id=${scope.leftCommitId}&right_commit_id=${scope.rightCommitId}`, { authorization: "Bearer request-token" }),
      routeContext()
    );
    assert.equal(response.status, 502);
    assert.equal(response.headers.get("retry-after"), "4");
    assert.deepEqual(await response.json(), {
      error: "contextlab_web_api_error",
      message: "Unable to load local Context merge review"
    });
  } finally {
    restoreEnvironment();
  }
});

function requestFor(query: string, headers?: HeadersInit): Request {
  return new Request(`http://contextlab.test/api/local/projects/${scope.projectId}/contexts/${scope.contextId}/merge-review${query}`, { headers });
}

function routeContext() {
  return { params: Promise.resolve({ projectId: scope.projectId, contextId: scope.contextId }) };
}

function reviewPayload() {
  return {
    schema_version: "v1",
    plan: { ThreeWay: { base: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee", left: scope.leftCommitId, right: scope.rightCommitId } },
    base_scope: { project_id: scope.projectId, context_id: scope.contextId, commit_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee" },
    left_scope: { project_id: scope.projectId, context_id: scope.contextId, commit_id: scope.leftCommitId },
    right_scope: { project_id: scope.projectId, context_id: scope.contextId, commit_id: scope.rightCommitId },
    classification: { Equivalent: { changes: [] } }
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
