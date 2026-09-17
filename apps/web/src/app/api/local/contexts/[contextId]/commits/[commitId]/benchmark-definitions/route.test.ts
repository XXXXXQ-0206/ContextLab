import assert from "node:assert/strict";
import test from "node:test";
import { POST } from "./route";

test("fails closed because the context-only authoring route is retired", async () => {
  let upstreamCalls = 0;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => {
    upstreamCalls += 1;
    throw new Error("the retired route must never contact an upstream service");
  }) as typeof fetch;

  try {
    const response = await POST(
      new Request("http://contextlab.test/api/local/contexts/context/commits/commit/benchmark-definitions", {
        method: "POST",
        headers: {
          authorization: "Bearer must-not-be-used",
          "idempotency-key": "must-not-be-used"
        },
        body: "{}"
      }),
      { params: Promise.resolve({ contextId: "context", commitId: "commit" }) }
    );

    assert.equal(response.status, 410);
    assert.equal(response.headers.get("cache-control"), "private, no-store");
    assert.equal(response.headers.get("sunset"), "2026-07-27");
    assert.deepEqual(await response.json(), {
      error: "benchmark_definition_route_gone",
      message: "Use the project-scoped benchmark-definition-bindings route"
    });
    assert.equal(upstreamCalls, 0);
  } finally {
    globalThis.fetch = originalFetch;
  }
});
