import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalContextMergeReviewClient,
  ContextLabLocalContextMergeReviewError,
  parseLocalContextMergeReviewProjectionV1
} from "./context-merge-review";

const PROJECT_ID = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const CONTEXT_ID = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const BASE_COMMIT_ID = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const LEFT_COMMIT_ID = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
const RIGHT_COMMIT_ID = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}

function scope(commit_id: string) {
  return { project_id: PROJECT_ID, context_id: CONTEXT_ID, commit_id };
}

function payload(classification: unknown = {
  Clean: {
    changes: [
      { Node: { node_id: "context:agent" } },
      { Edge: { source: "context:agent", target: "prompt:system", kind: "contains" } }
    ]
  }
}) {
  return {
    schema_version: "v1",
    plan: {
      ThreeWay: {
        base: BASE_COMMIT_ID,
        left: LEFT_COMMIT_ID,
        right: RIGHT_COMMIT_ID
      }
    },
    base_scope: scope(BASE_COMMIT_ID),
    left_scope: scope(LEFT_COMMIT_ID),
    right_scope: scope(RIGHT_COMMIT_ID),
    classification
  };
}

test("parses and freezes the V1 server-owned merge review projection", () => {
  const result = parseLocalContextMergeReviewProjectionV1(payload());

  assert.equal(result.schema_version, "v1");
  assert.deepEqual(result.plan, {
    kind: "three_way",
    base: BASE_COMMIT_ID,
    left: LEFT_COMMIT_ID,
    right: RIGHT_COMMIT_ID
  });
  assert.deepEqual(result.classification, {
    kind: "clean",
    changes: [
      { kind: "node", node_id: "context:agent" },
      {
        kind: "edge",
        source: "context:agent",
        target: "prompt:system",
        edge_kind: "contains"
      }
    ]
  });
  assert.equal(Object.isFrozen(result), true);
  assert.equal(Object.isFrozen(result.plan), true);
  assert.equal(Object.isFrozen(result.classification), true);
  assert.equal(Object.isFrozen(result.classification.changes), true);
});

test("parses equivalent and conflict arrays without calculating a graph diff", () => {
  const equivalent = parseLocalContextMergeReviewProjectionV1(payload({
    Equivalent: { changes: [{ Node: { node_id: "prompt:system" } }] }
  }));
  const conflict = parseLocalContextMergeReviewProjectionV1(payload({
    Conflict: {
      conflicts: [
        { Node: { node_id: "context:agent" } },
        { Edge: { source: "context:agent", target: "tool:search", kind: "uses" } }
      ]
    }
  }));

  assert.deepEqual(equivalent.classification, {
    kind: "equivalent",
    changes: [{ kind: "node", node_id: "prompt:system" }]
  });
  assert.deepEqual(conflict.classification, {
    kind: "conflict",
    conflicts: [
      { kind: "node", node_id: "context:agent" },
      { kind: "edge", source: "context:agent", target: "tool:search", edge_kind: "uses" }
    ]
  });
});

test("fails closed for schema, plan, UUID, scope, and unknown-key drift", () => {
  const valid = payload();

  assert.throws(() => parseLocalContextMergeReviewProjectionV1({ ...valid, schema_version: "v2" }), /schema_version/);
  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({
      ...valid,
      plan: { FastForward: { base: BASE_COMMIT_ID, target: LEFT_COMMIT_ID } }
    }),
    /shape|plan|three_way/i
  );
  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({ ...valid, base_scope: scope("not-a-uuid") }),
    /uuid/i
  );
  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({ ...valid, unexpected: true }),
    /shape|unexpected/i
  );
  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({
      ...valid,
      left_scope: { ...scope(LEFT_COMMIT_ID), project_id: "ffffffff-ffff-4fff-8fff-ffffffffffff" }
    }),
    /scope|project/i
  );
  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({
      ...valid,
      plan: { ThreeWay: { base: BASE_COMMIT_ID, left: RIGHT_COMMIT_ID, right: RIGHT_COMMIT_ID } }
    }),
    /plan|scope|commit|distinct/i
  );
});

test("fails closed for duplicate or unsorted classification entries", () => {
  const valid = payload();

  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({
      ...valid,
      classification: {
        Clean: {
          changes: [
            { Node: { node_id: "prompt:system" } },
            { Node: { node_id: "prompt:system" } }
          ]
        }
      }
    }),
    /unique|ordered|deterministic/i
  );
  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({
      ...valid,
      classification: {
        Clean: {
          changes: [
            { Node: { node_id: "prompt:z" } },
            { Node: { node_id: "prompt:a" } }
          ]
        }
      }
    }),
    /ordered|deterministic/i
  );
  assert.throws(
    () => parseLocalContextMergeReviewProjectionV1({
      ...valid,
      classification: { Clean: { changes: [{ Node: { node_id: "prompt:a", extra: true } }] } }
    }),
    /shape|unexpected/i
  );
});

test("loads the exact private route with Bearer-only, no-store transport", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const client = new ContextLabLocalContextMergeReviewClient({
    baseUrl: "http://contextlab.test/",
    fetch: async (input, init) => {
      requests.push({ url: String(input), init });
      return jsonResponse(payload());
    }
  });

  const result = await client.getContextMergeReview(
    PROJECT_ID,
    CONTEXT_ID,
    LEFT_COMMIT_ID,
    RIGHT_COMMIT_ID,
    { bearerToken: " local-read-token " }
  );

  assert.equal(result.classification.kind, "clean");
  assert.equal(
    requests[0]?.url,
    `http://contextlab.test/api/v1/local/projects/${PROJECT_ID}/contexts/${CONTEXT_ID}/merge-review?left_commit_id=${LEFT_COMMIT_ID}&right_commit_id=${RIGHT_COMMIT_ID}`
  );
  assert.equal(requests[0]?.init?.method, "GET");
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer local-read-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.cache, "no-store");
  assert.equal(requests[0]?.init?.body, undefined);
});

test("rejects identical tips, response scope drift, and typed API errors", async (t) => {
  await t.test("identical tips do not issue a request", async () => {
    let calls = 0;
    const client = new ContextLabLocalContextMergeReviewClient({
      fetch: async () => {
        calls += 1;
        return jsonResponse(payload());
      }
    });

    await assert.rejects(
      () => client.getContextMergeReview(PROJECT_ID, CONTEXT_ID, LEFT_COMMIT_ID, LEFT_COMMIT_ID, { bearerToken: "token" }),
      RangeError
    );
    assert.equal(calls, 0);
  });

  await t.test("response scope drift fails closed", async () => {
    const client = new ContextLabLocalContextMergeReviewClient({
      fetch: async () => jsonResponse({ ...payload(), right_scope: scope(LEFT_COMMIT_ID) })
    });

    await assert.rejects(
      () => client.getContextMergeReview(PROJECT_ID, CONTEXT_ID, LEFT_COMMIT_ID, RIGHT_COMMIT_ID, { bearerToken: "token" }),
      /scope|commit/i
    );
  });

  await t.test("structured API errors remain typed and redacted", async () => {
    const client = new ContextLabLocalContextMergeReviewClient({
      fetch: async () => jsonResponse(
        { error: "context_read_forbidden", message: "private upstream detail" },
        { status: 403, headers: { "retry-after": "2" } }
      )
    });

    await assert.rejects(
      () => client.getContextMergeReview(PROJECT_ID, CONTEXT_ID, LEFT_COMMIT_ID, RIGHT_COMMIT_ID, { bearerToken: "token" }),
      (error: unknown) =>
        error instanceof ContextLabLocalContextMergeReviewError
        && error.status === 403
        && error.code === "context_read_forbidden"
        && error.retryAfterMs === 2_000
        && !error.message.includes("private upstream detail")
    );
  });
});
