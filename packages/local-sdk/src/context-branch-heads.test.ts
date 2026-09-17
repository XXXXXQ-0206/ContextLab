import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalBranchHeadClient,
  ContextLabLocalBranchHeadError,
  LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1,
  parseLocalContextBranchHeadsResourceV1
} from "./context-branch-heads";

const contextId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const headCommitId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

function branchHeadsPayload() {
  return {
    schema_version: LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1,
    context_id: contextId,
    branches: [
      { branch_name: "feature/read-only", head_commit_id: headCommitId, revision: 7 },
      { branch_name: "main", head_commit_id: null, revision: 0 }
    ]
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}

test("parses the exact branch-head resource and freezes deterministic DTOs", () => {
  const result = parseLocalContextBranchHeadsResourceV1(branchHeadsPayload());

  assert.deepEqual(result, branchHeadsPayload());
  assert.equal(result.branches[1]?.head_commit_id, null);
  assert.equal(Object.isFrozen(result), true);
  assert.equal(Object.isFrozen(result.branches), true);
  assert.equal(Object.isFrozen(result.branches[0]), true);
});

test("rejects schema drift, malformed UUIDs, invalid revisions, unexpected heads, and branch names", () => {
  const payload = branchHeadsPayload();
  const invalidValues: unknown[] = [
    { ...payload, extra: true },
    { ...payload, schema_version: "contextlab.local-context-branch-heads.v2" },
    { ...payload, context_id: "not-a-uuid" },
    { ...payload, branches: [{ ...payload.branches[0], head_commit_id: true }] },
    { ...payload, branches: [{ ...payload.branches[0], head_commit_id: "not-a-uuid" }] },
    { ...payload, branches: [{ ...payload.branches[0], revision: -1 }] },
    { ...payload, branches: [{ ...payload.branches[0], revision: 1.5 }] },
    { ...payload, branches: [{ ...payload.branches[0], revision: "0" }] },
    { ...payload, branches: [{ ...payload.branches[0], branch_name: "feature branch" }] },
    { ...payload, branches: [{ ...payload.branches[0], extra: false }] }
  ];

  for (const value of invalidValues) {
    assert.throws(() => parseLocalContextBranchHeadsResourceV1(value), TypeError);
  }
});

test("rejects duplicate and unsorted branch names", () => {
  const payload = branchHeadsPayload();

  assert.throws(
    () => parseLocalContextBranchHeadsResourceV1({
      ...payload,
      branches: [payload.branches[1], { ...payload.branches[1], head_commit_id: headCommitId }]
    }),
    TypeError
  );
  assert.throws(
    () => parseLocalContextBranchHeadsResourceV1({
      ...payload,
      branches: [...payload.branches].reverse()
    }),
    TypeError
  );
});

test("client performs an exact private read and verifies response scope", async () => {
  const calls: Array<{ url: string; init: RequestInit | undefined }> = [];
  const client = new ContextLabLocalBranchHeadClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: async (input, init) => {
      calls.push({ url: String(input), init });
      return jsonResponse(branchHeadsPayload());
    }
  });

  const result = await client.getContextBranchHeads(contextId, { bearerToken: " local-token " });

  assert.equal(result.context_id, contextId);
  assert.equal(calls[0]?.url, `http://127.0.0.1:3100/api/v1/local/contexts/${contextId}/branches`);
  assert.equal(calls[0]?.init?.method, "GET");
  assert.equal(calls[0]?.init?.credentials, "omit");
  assert.equal(calls[0]?.init?.cache, "no-store");
  assert.equal(calls[0]?.init?.body, undefined);
  assert.equal(new Headers(calls[0]?.init?.headers).get("authorization"), "Bearer local-token");
  assert.equal(new Headers(calls[0]?.init?.headers).get("cookie"), null);
});

test("client fails closed on response scope drift and redacts untrusted API errors", async (t) => {
  await t.test("scope drift", async () => {
    const client = new ContextLabLocalBranchHeadClient({
      fetch: async () => jsonResponse({ ...branchHeadsPayload(), context_id: headCommitId })
    });

    await assert.rejects(
      client.getContextBranchHeads(contextId, { bearerToken: "local-token" }),
      /requested scope/
    );
  });

  await t.test("untrusted error body", async () => {
    const client = new ContextLabLocalBranchHeadClient({
      fetch: async () => jsonResponse(
        { error: "internal_sql_diagnostic", message: "secret database details" },
        { status: 502 }
      )
    });

    await assert.rejects(
      client.getContextBranchHeads(contextId, { bearerToken: "local-token" }),
      (error: unknown) => error instanceof ContextLabLocalBranchHeadError
        && error.status === 502
        && error.code === "contextlab_local_api_error"
        && !error.message.includes("secret database details")
    );
  });
});
