import assert from "node:assert/strict";
import test from "node:test";
import { GET, POST } from "./route";

const originalFetch = globalThis.fetch;
const originalBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;

function response(payload: unknown, status = 201): Response {
  return new Response(JSON.stringify(payload), {
    status,
    headers: { "content-type": "application/json" }
  });
}

function command() {
  return {
    schema_version: 1,
    binding_id: "11111111-1111-4111-8111-111111111111",
    branch_name: "main",
    expected_head_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    datasets: [{
      id: "22222222-2222-4222-8222-222222222222",
      name: "Safety prompts",
      cases: [{
        id: "33333333-3333-4333-8333-333333333333",
        name: "Case",
        input: { question: "What is Context?" },
        expected_output: { mode: "unspecified" }
      }]
    }],
    suite: {
      id: "44444444-4444-4444-8444-444444444444",
      name: "Safety suite",
      dataset_ids: ["22222222-2222-4222-8222-222222222222"],
      thresholds: [{ metric: "accuracy", direction: "minimum", value: 0.9 }]
    }
  };
}

function receipt() {
  return {
    schema_version: "contextlab.local-benchmark-definition-authoring.v1",
    disposition: "created",
    message: { en: "Benchmark definitions authored.", zh: "Benchmark 定义已创建。" },
    project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    binding_id: "11111111-1111-4111-8111-111111111111",
    branch_name: "main",
    definition_schema_version: 1,
    dataset_ids: ["22222222-2222-4222-8222-222222222222"],
    suite_id: "44444444-4444-4444-8444-444444444444",
    captured_at: "2026-07-27T12:00:00Z"
  };
}

function inspectionPayload() {
  return {
    schema_version: "contextlab.local-benchmark-definition-binding-inspection.v1",
    project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    bindings: [{
      binding_id: "11111111-1111-4111-8111-111111111111",
      project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
      commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
      branch_name: "main",
      definition_schema_version: 1,
      suite_id: "44444444-4444-4444-8444-444444444444",
      suite_name: "Safety suite",
      dataset_ids: ["22222222-2222-4222-8222-222222222222"],
      dataset_names: ["Safety prompts"],
      captured_at: "2026-07-27T12:00:00Z"
    }]
  };
}

test("forwards the exact project, Context, and commit scope to the private API", async () => {
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "https://upstream.contextlab.test";
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    return response(receipt());
  }) as typeof fetch;

  try {
    const projectId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const contextId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    const commitId = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
    const result = await POST(
      new Request(`http://contextlab.test/api/local/projects/${projectId}/contexts/${contextId}/commits/${commitId}/benchmark-definition-bindings`, {
        method: "POST",
        headers: {
          authorization: "Bearer token",
          "content-type": "application/json",
          "idempotency-key": "binding-1"
        },
        body: JSON.stringify(command())
      }),
      { params: Promise.resolve({ projectId, contextId, commitId }) }
    );

    assert.equal(result.status, 201);
    assert.equal(calls[0]?.url, "https://upstream.contextlab.test/api/v1/local/projects/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/contexts/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/commits/cccccccc-cccc-4ccc-8ccc-cccccccccccc/benchmark-definition-bindings");
    assert.equal(new Headers(calls[0]?.init?.headers).get("authorization"), "Bearer token");
    assert.equal(calls[0]?.init?.credentials, "omit");
  } finally {
    globalThis.fetch = originalFetch;
    if (originalBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
    else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalBaseUrl;
  }
});

test("forwards exact binding inspection scope with bearer-only no-store transport", async () => {
  const calls: Array<{ url: string; init?: RequestInit }> = [];
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "https://upstream.contextlab.test";
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    calls.push({ url: String(input), init });
    return response(inspectionPayload(), 200);
  }) as typeof fetch;

  try {
    const projectId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const contextId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    const commitId = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
    const result = await GET(
      new Request(
        `http://contextlab.test/api/local/projects/${projectId}/contexts/${contextId}/commits/${commitId}/benchmark-definition-bindings`,
        {
          headers: {
            authorization: "Bearer token",
            cookie: "session=must-not-cross"
          }
        }
      ),
      { params: Promise.resolve({ projectId, contextId, commitId }) }
    );

    assert.equal(result.status, 200);
    assert.equal(result.headers.get("cache-control"), "private, no-store");
    assert.equal(calls[0]?.url, "https://upstream.contextlab.test/api/v1/local/projects/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/contexts/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/commits/cccccccc-cccc-4ccc-8ccc-cccccccccccc/benchmark-definition-bindings");
    const headers = new Headers(calls[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer token");
    assert.equal(headers.get("cookie"), null);
    assert.equal(calls[0]?.init?.credentials, "omit");
    assert.equal(calls[0]?.init?.cache, "no-store");
    assert.equal((await result.json()).bindings[0].suite_name, "Safety suite");
  } finally {
    globalThis.fetch = originalFetch;
    if (originalBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
    else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalBaseUrl;
  }
});

test("fails closed when binding inspection receives raw or scope-drifting data", async () => {
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "https://upstream.contextlab.test";
  globalThis.fetch = (async () =>
    response({ ...inspectionPayload(), raw_cases: [{ input: "private" }] }, 200)) as typeof fetch;

  try {
    const result = await GET(
      new Request("http://contextlab.test/api/local/projects/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/contexts/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/commits/cccccccc-cccc-4ccc-8ccc-cccccccccccc/benchmark-definition-bindings", {
        headers: { authorization: "Bearer token" }
      }),
      { params: Promise.resolve({
        projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
        contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
      }) }
    );
    assert.equal(result.status, 502);
    assert.doesNotMatch(await result.text(), /private|raw_cases/);
  } finally {
    globalThis.fetch = originalFetch;
    if (originalBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
    else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalBaseUrl;
  }
});
