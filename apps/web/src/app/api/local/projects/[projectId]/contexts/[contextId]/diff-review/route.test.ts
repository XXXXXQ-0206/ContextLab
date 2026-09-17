import assert from "node:assert/strict";
import test from "node:test";
import { GET } from "./route";

const originalFetch = globalThis.fetch;
const originalBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL;

test("persisted diff BFF rejects query drift and missing Bearer before upstream access", async (t) => {
  let calls = 0;
  globalThis.fetch = (async () => { calls += 1; return new Response(); }) as typeof fetch;
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  try {
    const missingAuth = await GET(new Request("http://contextlab.test/api/local/projects/p/contexts/c/diff-review?source_commit_id=s&target_commit_id=t"), { params: Promise.resolve({ projectId: "p", contextId: "c" }) });
    assert.equal(missingAuth.status, 401);
    const drift = await GET(new Request("http://contextlab.test/api/local/projects/p/contexts/c/diff-review?source_commit_id=s&target_commit_id=t&extra=1", { headers: { authorization: "Bearer token" } }), { params: Promise.resolve({ projectId: "p", contextId: "c" }) });
    assert.equal(drift.status, 400);
    assert.equal(calls, 0);
    assert.equal(missingAuth.headers.get("cache-control"), "private, no-store");
  } finally {
    restoreEnv();
  }
});

test("persisted diff BFF forwards only request Bearer and preserves private no-store", async () => {
  const calls: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input, init) => {
    calls.push({ input: String(input), init });
    return new Response(JSON.stringify(review()), { headers: { "content-type": "application/json" } });
  }) as typeof fetch;
  process.env.CONTEXTLAB_WEB_API_BASE_URL = "http://upstream.contextlab.test";
  try {
    const response = await GET(new Request("http://contextlab.test/api/local/projects/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/contexts/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/diff-review?source_commit_id=cccccccc-cccc-4ccc-8ccc-cccccccccccc&target_commit_id=dddddddd-dddd-4ddd-8ddd-dddddddddddd", { headers: { authorization: "Bearer request-token", cookie: "must-not-forward" } }), { params: Promise.resolve({ projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb" }) });
    assert.equal(response.status, 200);
    assert.equal(response.headers.get("cache-control"), "private, no-store");
    const payload = await response.json() as {
      diff: {
        semantic: { graph_diff: { added_nodes: unknown[] } };
        behavior: { case_changes: unknown[] };
        evaluation: { metric_changes: unknown[] };
      };
    };
    assert.equal(payload.diff.semantic.graph_diff.added_nodes.length > 0, true);
    assert.equal(payload.diff.behavior.case_changes.length > 0, true);
    assert.equal(payload.diff.evaluation.metric_changes.length > 0, true);
    assert.equal(calls[0]?.input, "http://upstream.contextlab.test/api/v1/local/projects/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/contexts/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/diff-review?source_commit_id=cccccccc-cccc-4ccc-8ccc-cccccccccccc&target_commit_id=dddddddd-dddd-4ddd-8ddd-dddddddddddd");
    assert.equal(new Headers(calls[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(calls[0]?.init?.headers).get("cookie"), null);
  } finally {
    restoreEnv();
  }
});

function restoreEnv() {
  globalThis.fetch = originalFetch;
  if (originalBaseUrl === undefined) delete process.env.CONTEXTLAB_WEB_API_BASE_URL;
  else process.env.CONTEXTLAB_WEB_API_BASE_URL = originalBaseUrl;
}

function review() {
  return {
    schema_version: "v1",
    source_scope: { project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc" },
    target_scope: { project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd" },
    diff: {
      contract_version: "v1",
      semantic: { graph_diff: { added_nodes: [{ id: "prompt:system", kind: "prompt", label: "System" }], removed_nodes: [], modified_nodes: [], added_edges: [], removed_edges: [] }, document_changes: [] },
      behavior: { case_changes: [{ kind: "added", revised: { case_id: "case:context-review", input_fingerprint: "input:context-review:v1", outcome: { kind: "succeeded", output: "baseline" } } }] },
      evaluation: { comparability_fingerprint: "suite:context-review:v1", metric_changes: [{ kind: "added", revised: { metric_id: "accuracy", value: 0.9, sample_count: 10 } }] }
    }
  };
}
