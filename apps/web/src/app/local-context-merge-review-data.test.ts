import assert from "node:assert/strict";
import test from "node:test";
import {
  LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1,
  LocalContextMergeReviewProxyError,
  adaptLocalContextMergeReviewV1,
  createLocalContextMergeReviewResource,
  loadLocalContextMergeReview,
  parseLocalContextMergeReviewV1
} from "./local-context-merge-review-data";

const originalFetch = globalThis.fetch;
const target = {
  project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  left_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  right_commit_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd"
} as const;

test("parser accepts the exact SDK-compatible V1 projection and normalizes classification state", () => {
  const review = parseLocalContextMergeReviewV1(payload());

  assert.equal(review.schema_version, LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1);
  assert.deepEqual(review.plan, {
    kind: "three_way",
    base: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee",
    left: target.left_commit_id,
    right: target.right_commit_id
  });
  assert.equal(review.base_scope.commit_id, "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee");
  assert.equal(review.classification.kind, "clean");
  assert.deepEqual(review.classification.changes, [
    { kind: "node", node_id: "context:prompt" },
    { kind: "edge", source: "context:prompt", target: "context:model", edge_kind: "uses" }
  ]);
  assert.equal(Object.isFrozen(review), true);
});

test("parser rejects schema, unknown shape, non-three-way plan, and malformed classification", () => {
  assert.throws(() => parseLocalContextMergeReviewV1({ ...payload(), schema_version: "v2" }), /schema_version/);
  assert.throws(() => parseLocalContextMergeReviewV1({ ...payload(), unexpected: true }), /exact keys|unexpected shape/);
  assert.throws(() => parseLocalContextMergeReviewV1({ ...payload(), plan: { FastForward: { base: target.left_commit_id, target: target.right_commit_id } } }), /keys|required|unexpected shape/);
  assert.throws(() => parseLocalContextMergeReviewV1({ ...payload(), classification: { Clean: { changes: [{ Node: { node_id: "" } }] } } }), /node_id/);
});

test("loader sends exact query scope with bearer-only no-store transport and rejects response drift", async (t) => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input, init) => {
    requests.push({ input: String(input), init });
    return jsonResponse(payload());
  }) as typeof fetch;
  try {
    const review = await loadLocalContextMergeReview(target, " request-token ");
    assert.equal(review.right_scope.commit_id, target.right_commit_id);
    assert.equal(
      requests[0]?.input,
      `/api/local/projects/${target.project_id}/contexts/${target.context_id}/merge-review?left_commit_id=${target.left_commit_id}&right_commit_id=${target.right_commit_id}`
    );
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);

    await t.test("response scope drift fails closed", async () => {
      globalThis.fetch = (async () => jsonResponse({
        ...payload(),
        right_scope: { ...payload().right_scope, context_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee" }
      })) as typeof fetch;
      await assert.rejects(() => loadLocalContextMergeReview(target, "request-token"), /scope/);
    });
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("loader exposes only a redacted proxy failure", async () => {
  globalThis.fetch = (async () => jsonResponse(
    { error: "upstream_internal", message: "private upstream stack and prompt" },
    { status: 502, headers: { "retry-after": "3" } }
  )) as typeof fetch;
  try {
    await assert.rejects(
      () => loadLocalContextMergeReview(target, "request-token"),
      (error: unknown) => error instanceof LocalContextMergeReviewProxyError
        && error.status === 502
        && error.message === "ContextLab local API request failed with status 502"
        && error.body.message === "ContextLab local API request failed with status 502"
        && error.retryAfterMs === 3_000
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("resource adapter keeps loading, error, empty, unavailable, and ready states explicit", () => {
  for (const kind of ["loading", "error", "empty", "unavailable"] as const) {
    assert.equal(adaptLocalContextMergeReviewV1({ kind, target, message: "raw upstream detail" }).state, kind);
  }
  const ready = createLocalContextMergeReviewResource({ kind: "ready", target, review: parseLocalContextMergeReviewV1(payload()) });
  assert.equal(adaptLocalContextMergeReviewV1(ready).state, "available");
});

function payload() {
  const baseCommitId = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";
  return {
    schema_version: "v1",
    plan: {
      ThreeWay: { base: baseCommitId, left: target.left_commit_id, right: target.right_commit_id }
    },
    base_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: baseCommitId },
    left_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.left_commit_id },
    right_scope: { project_id: target.project_id, context_id: target.context_id, commit_id: target.right_commit_id },
    classification: {
      Clean: {
        changes: [
          { Node: { node_id: "context:prompt" } },
          { Edge: { source: "context:prompt", target: "context:model", kind: "uses" } }
        ]
      }
    }
  } as const;
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), { headers: { "content-type": "application/json" }, ...init });
}
