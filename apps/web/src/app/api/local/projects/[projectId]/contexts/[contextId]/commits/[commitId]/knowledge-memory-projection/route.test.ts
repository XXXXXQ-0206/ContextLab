import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

test("Knowledge/Memory BFF rejects missing Bearer before upstream access", async () => {
  const originalBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;
  const originalFetch = globalThis.fetch;
  let upstreamCalled = false;
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://127.0.0.1:3100";
  globalThis.fetch = (async () => {
    upstreamCalled = true;
    return new Response();
  }) as typeof fetch;

  try {
    const response = await GET(
      new Request("http://localhost/api/local/projects/project/contexts/context/commits/commit/knowledge-memory-projection"),
      { params: Promise.resolve({ projectId: "project", contextId: "context", commitId: "commit" }) }
    );
    assert.equal(response.status, 401);
    assert.equal(upstreamCalled, false);
    assert.equal((await response.json()).error, "authentication_required");
  } finally {
    globalThis.fetch = originalFetch;
    if (originalBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
    else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalBaseUrl;
  }
});

test("Knowledge/Memory BFF rejects query and body drift", async () => {
  const response = await GET(
    new Request("http://localhost/api/local/projects/project/contexts/context/commits/commit/knowledge-memory-projection?unexpected=true", {
      method: "GET"
    }),
    { params: Promise.resolve({ projectId: "project", contextId: "context", commitId: "commit" }) }
  );
  assert.equal(response.status, 400);
  assert.equal((await response.json()).error, "invalid_local_knowledge_memory_projection_request");
});
