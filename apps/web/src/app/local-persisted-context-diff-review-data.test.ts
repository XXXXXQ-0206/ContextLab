import assert from "node:assert/strict";
import test from "node:test";
import {
  createLocalPersistedContextDiffReviewResource,
  loadLocalPersistedContextDiffReview,
  LocalPersistedContextDiffReviewProxyError
} from "./local-persisted-context-diff-review-data";

const originalFetch = globalThis.fetch;
const target = {
  project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  source_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  target_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
} as const;

test("persisted diff data adapter sends exact scope with request-memory Bearer only", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input, init) => {
    requests.push({ input: String(input), init });
    return jsonResponse(review());
  }) as typeof fetch;
  try {
    const result = await loadLocalPersistedContextDiffReview(target, " request-token ");
    assert.equal(result.source_scope.commit_id, target.source_commit_id);
    assert.equal(requests[0]?.input, `/api/local/projects/${target.project_id}/contexts/${target.context_id}/diff-review?source_commit_id=${target.source_commit_id}&target_commit_id=${target.target_commit_id}`);
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("persisted diff data adapter rejects response scope drift and redacts failures", async (t) => {
  await t.test("scope drift", async () => {
    globalThis.fetch = (async () => jsonResponse({
      ...review(),
      target_scope: { ...review().target_scope, context_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee" }
    })) as typeof fetch;
    try {
      await assert.rejects(() => loadLocalPersistedContextDiffReview(target, "request-token"), TypeError);
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
  await t.test("structured proxy failure", async () => {
    globalThis.fetch = (async () => jsonResponse({ error: "authentication_required", message: "Bearer authentication is required" }, { status: 401 })) as typeof fetch;
    try {
      await assert.rejects(
        () => loadLocalPersistedContextDiffReview(target, "request-token"),
        (error: unknown) => error instanceof LocalPersistedContextDiffReviewProxyError && error.status === 401
      );
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
  await t.test("unknown structured proxy messages are redacted", async () => {
    globalThis.fetch = (async () => jsonResponse({
      error: "upstream_private_failure",
      message: "sql://internal-db?token=secret diagnostic payload"
    }, { status: 599 })) as typeof fetch;
    try {
      await assert.rejects(
        () => loadLocalPersistedContextDiffReview(target, "request-token"),
        (error: unknown) => error instanceof LocalPersistedContextDiffReviewProxyError
          && error.status === 599
          && error.body.error === "upstream_private_failure"
          && error.body.message === "The local persisted Context diff review is unavailable. / 本地持久化 Context diff review 不可用。"
          && !error.body.message.includes("internal-db")
          && !error.body.message.includes("secret")
      );
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
});

test("persisted diff data adapter fails closed for malformed metadata changes", () => {
  const malformed = {
    kind: "ready",
    target,
    review: {
      schema_version: "v1",
      source_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.source_commit_id },
      target_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.target_commit_id },
      diff: {
        contract_version: "v1",
        semantic: {
          graph_diff: { added_nodes: [], removed_nodes: [], modified_nodes: [], added_edges: [], removed_edges: [] },
          document_changes: [],
          metadata_change: {
            kind: "modified",
            original: { created_at: "2025-07-07T00:00:00Z", updated_at: "2025-07-07T00:00:00Z", labels: {} },
            revised: { created_at: "2025-07-07T00:00:00Z", updated_at: "2025-07-07T00:00:10Z", labels: {} },
            unexpected: true
          }
        },
        behavior: { case_changes: [] },
        evaluation: { comparability_fingerprint: "fp", metric_changes: [] }
      }
    }
  } as unknown as Parameters<typeof createLocalPersistedContextDiffReviewResource>[0];

  assert.throws(() => createLocalPersistedContextDiffReviewResource(malformed), TypeError);
});

function review() {
  return {
    schema_version: "v1",
    source_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.source_commit_id },
    target_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.target_commit_id },
    diff: {
      contract_version: "v1",
      semantic: { graph_diff: { added_nodes: [], removed_nodes: [], modified_nodes: [], added_edges: [], removed_edges: [] }, document_changes: [] },
      behavior: { case_changes: [] },
      evaluation: { comparability_fingerprint: "fp", metric_changes: [] }
    }
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), { headers: { "content-type": "application/json" }, ...init });
}
