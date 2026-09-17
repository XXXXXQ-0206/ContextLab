import assert from "node:assert/strict";
import test from "node:test";
import { loadLocalCommitGraphDiff, LocalCommitGraphDiffProxyError } from "./local-commit-graph-diff-data";

const originalFetch = globalThis.fetch;

test("local graph diff data adapter sends the exact same-origin scope with request-memory Bearer only", async () => {
  const calls: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    calls.push({ input: String(input), init });
    return jsonResponse(graphDiff());
  }) as typeof fetch;

  try {
    const contextId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const originalCommitId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
    const revisedCommitId = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
    const result = await loadLocalCommitGraphDiff(contextId, originalCommitId, revisedCommitId, " request-token ");
    assert.equal(result.context_id, contextId);
    assert.equal(result.pair_witness.project_id, "dddddddd-dddd-4ddd-8ddd-dddddddddddd");
    assert.equal(result.pair_witness.baseline_commit_id, originalCommitId);
    assert.equal(result.pair_witness.revised_commit_id, revisedCommitId);
    assert.equal(Object.isFrozen(result), true);
    assert.equal(calls[0]?.input, `/api/local/contexts/${contextId}/graph-diff?original_commit_id=${originalCommitId}&revised_commit_id=${revisedCommitId}`);
    assert.equal(new Headers(calls[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(calls[0]?.init?.headers).get("cookie"), null);
    assert.equal(calls[0]?.init?.credentials, "omit");
    assert.equal(calls[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("local graph diff data adapter exposes stable structured failures", async () => {
  globalThis.fetch = (async () => jsonResponse({ error: "authentication_required", message: "Bearer authentication is required" }, { status: 401 })) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalCommitGraphDiff("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "cccccccc-cccc-4ccc-8ccc-cccccccccccc", "request-token"),
      (error: unknown) => error instanceof LocalCommitGraphDiffProxyError && error.status === 401 && error.body.error === "authentication_required"
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("local graph diff data adapter redacts unknown upstream messages while preserving status and code", async () => {
  globalThis.fetch = (async () => jsonResponse({
    error: "upstream_private_failure",
    message: "sql://internal-db?token=secret diagnostic payload"
  }, { status: 599 })) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalCommitGraphDiff("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", "cccccccc-cccc-4ccc-8ccc-cccccccccccc", "request-token"),
      (error: unknown) => error instanceof LocalCommitGraphDiffProxyError
        && error.status === 599
        && error.body.error === "upstream_private_failure"
        && error.body.message === "Unable to load local graph review / 无法加载本地图谱审阅。"
        && !error.body.message.includes("internal-db")
        && !error.body.message.includes("secret")
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function graphDiff() {
  return {
    context_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    pair_witness: {
      schema_version: 1,
      project_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
      context_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      baseline_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
      revised_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
    },
    original: { commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", captured_at: "2026-07-08T00:00:00Z", schema_version: 1 },
    revised: { commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", captured_at: "2026-07-09T00:00:00Z", schema_version: 1 },
    diff: { added_nodes: [], removed_nodes: [], modified_nodes: [], added_edges: [], removed_edges: [] }
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), { headers: { "content-type": "application/json" }, ...init });
}
