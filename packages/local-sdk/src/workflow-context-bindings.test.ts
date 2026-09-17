import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalClient,
  parseLocalWorkflowContextBindings,
  type FetchLike
} from "./index";

test("parses the exact redacted workflow Context binding summary", () => {
  const payload = workflowContextBindingsPayload();

  assert.deepEqual(parseLocalWorkflowContextBindings(payload), payload);
});

test("rejects unknown schema and raw workflow fields", () => {
  const payload = workflowContextBindingsPayload();

  for (const [name, mutate] of [
    ["unknown schema", (value: typeof payload) => { value.schema_version = "contextlab.local-workflow-context-bindings.v2"; }],
    ["unknown root field", (value: typeof payload) => { (value as Record<string, unknown>).metadata = {}; }],
    ["raw workflow summary", (value: typeof payload) => { (value.bindings[0] as Record<string, unknown>).raw_workflow = {}; }],
    ["raw workflow field", (value: typeof payload) => { (value.bindings[0] as Record<string, unknown>).workflow = {}; }],
    ["raw workflow definition", (value: typeof payload) => { (value.bindings[0] as Record<string, unknown>).definition = {}; }]
  ] as const) {
    assert.throws(
      () => parseLocalWorkflowContextBindings(mutatePayload(payload, mutate)),
      TypeError,
      name
    );
  }
});

test("rejects unstable binding ordering and invalid summary counts", () => {
  const payload = workflowContextBindingsPayload();

  for (const [name, mutate] of [
    ["workflow ordering", (value: typeof payload) => { value.bindings.reverse(); }],
    ["workflow revision ordering", (value: typeof payload) => { value.bindings[1]!.workflow_revision = 0; }],
    ["node count", (value: typeof payload) => { value.bindings[0]!.node_count = -1; }],
    ["edge count", (value: typeof payload) => { value.bindings[0]!.edge_count = 1.5; }]
  ] as const) {
    assert.throws(
      () => parseLocalWorkflowContextBindings(mutatePayload(payload, mutate)),
      TypeError,
      name
    );
  }
});

test("reads the exact private workflow binding scope with request-scoped bearer credentials", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  const fetchImpl: FetchLike = async (input, init) => {
    requests.push({ input: String(input), init });
    return jsonResponse(workflowContextBindingsPayload());
  };
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: fetchImpl
  });

  const bindings = await client.getWorkflowContextBindings(
    "context/id",
    "commit/id",
    { bearerToken: "request-token" }
  );

  assert.equal(bindings.commit_id, "commit/id");
  assert.equal(
    requests[0]?.input,
    "http://127.0.0.1:3100/api/v1/local/contexts/context%2Fid/commits/commit%2Fid/workflow-bindings"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.cache, "no-store");
  assert.equal(requests[0]?.init?.method, undefined);
});

test("rejects a workflow binding summary whose response scope differs from the request", async (t) => {
  for (const [name, field] of [
    ["context", "context_id"],
    ["commit", "commit_id"]
  ] as const) {
    await t.test(name, async () => {
      const payload = workflowContextBindingsPayload();
      payload[field] = `${payload[field]}-other`;
      const client = new ContextLabLocalClient({
        fetch: async () => jsonResponse(payload)
      });

      await assert.rejects(
        () => client.getWorkflowContextBindings("context/id", "commit/id", { bearerToken: "request-token" }),
        TypeError
      );
    });
  }
});

function workflowContextBindingsPayload() {
  return {
    schema_version: "contextlab.local-workflow-context-bindings.v1",
    context_id: "context/id",
    commit_id: "commit/id",
    bindings: [
      {
        binding_id: "binding/a",
        workflow_id: "workflow/a",
        workflow_revision: 1,
        node_count: 2,
        edge_count: 1
      },
      {
        binding_id: "binding/b",
        workflow_id: "workflow/a",
        workflow_revision: 2,
        node_count: 3,
        edge_count: 2
      },
      {
        binding_id: "binding/c",
        workflow_id: "workflow/b",
        workflow_revision: 1,
        node_count: 1,
        edge_count: 0
      }
    ]
  };
}

function mutatePayload<T>(payload: T, mutate: (value: T) => void): T {
  const copy = structuredClone(payload);
  mutate(copy);
  return copy;
}

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" }
  });
}
