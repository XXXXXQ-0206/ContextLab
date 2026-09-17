import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalWorkflowExecutionStatusClient,
  ContextLabLocalWorkflowExecutionStatusError,
  LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1,
  parseLocalWorkflowExecutionStatusV1,
  type LocalWorkflowExecutionStatusV1
} from "./workflow-execution-status";

const contextId = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const contextCommitId = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const bindingId = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const workflowId = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
const sourceRunId = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";
const replayRunId = "ffffffff-ffff-4fff-8fff-ffffffffffff";

test("parses failed and replayed redacted status payloads with exact core counts", () => {
  const failed = parseLocalWorkflowExecutionStatusV1(payload({
    run_id: sourceRunId,
    replay_of: null,
    run_state: "failed",
    node_status_counts: { pending: 0, running: 0, succeeded: 2, failed: 1, blocked: 3 }
  }));
  const replayed = parseLocalWorkflowExecutionStatusV1(payload({
    run_id: replayRunId,
    replay_of: sourceRunId,
    run_state: "pending",
    node_status_counts: { pending: 6, running: 0, succeeded: 0, failed: 0, blocked: 0 }
  }));

  assert.equal(failed.run_state, "failed");
  assert.equal(failed.replay_of, null);
  assert.equal(replayed.replay_of, sourceRunId);
  assert.deepEqual(replayed.node_status_counts, {
    pending: 6,
    running: 0,
    succeeded: 0,
    failed: 0,
    blocked: 0
  });
  assert.equal(Object.isFrozen(failed), true);
  assert.equal(Object.isFrozen(failed.node_status_counts), true);
  assert.doesNotMatch(JSON.stringify(failed), /failure|provider|secret|event_payload/);
});

test("rejects schema, scope-shaped, raw, unknown, and non-deterministic fields", () => {
  const valid = payload({});
  assert.throws(
    () => parseLocalWorkflowExecutionStatusV1({ ...valid, schema_version: "v2" }),
    /schema_version/
  );
  assert.throws(
    () => parseLocalWorkflowExecutionStatusV1({ ...valid, context_id: "not-a-uuid" }),
    /context_id/
  );
  assert.throws(
    () => parseLocalWorkflowExecutionStatusV1({ ...valid, unexpected: true }),
    /unexpected shape/
  );
  assert.throws(
    () => parseLocalWorkflowExecutionStatusV1({ ...valid, failure: { provider: "secret" } }),
    /forbidden raw or secret/
  );
  assert.throws(
    () => parseLocalWorkflowExecutionStatusV1({
      ...valid,
      node_status_counts: { pending: 0, running: 0, failed: 0, succeeded: 0, blocked: 0 }
    }),
    /deterministic core field order/
  );
});

test("client uses the exact upstream path and request-scoped bearer transport", async () => {
  const requests: Array<{ url: string; init: RequestInit | undefined }> = [];
  const client = new ContextLabLocalWorkflowExecutionStatusClient({
    baseUrl: "http://upstream.contextlab.test/",
    fetch: async (input, init) => {
      requests.push({ url: String(input), init });
      return jsonResponse(payload({}));
    }
  });

  const result = await client.getWorkflowExecutionStatus(contextId, sourceRunId, {
    bearerToken: "request-token"
  });

  assert.equal(result.run_id, sourceRunId);
  assert.equal(
    requests[0]?.url,
    `http://upstream.contextlab.test/api/v1/local/contexts/${contextId}/workflow/runs/${sourceRunId}/status`
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(requests[0]?.init?.cache, "no-store");
});

test("client fails closed on response run drift and preserves upstream 503", async () => {
  const driftClient = new ContextLabLocalWorkflowExecutionStatusClient({
    fetch: async () => jsonResponse(payload({ run_id: replayRunId }))
  });
  await assert.rejects(
    () => driftClient.getWorkflowExecutionStatus(contextId, sourceRunId, { bearerToken: "token" }),
    /scope or run/
  );

  const unavailableClient = new ContextLabLocalWorkflowExecutionStatusClient({
    fetch: async () => jsonResponse(
      { error: "workflow_execution_status_unavailable", message: "unavailable" },
      { status: 503 }
    )
  });
  await assert.rejects(
    () => unavailableClient.getWorkflowExecutionStatus(contextId, sourceRunId, { bearerToken: "token" }),
    (error: unknown) => error instanceof ContextLabLocalWorkflowExecutionStatusError && error.status === 503
  );
});

function payload(
  overrides: Partial<LocalWorkflowExecutionStatusV1> & Record<string, unknown> = {}
) {
  return {
    schema_version: LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1,
    context_id: contextId,
    context_commit_id: contextCommitId,
    binding_id: bindingId,
    workflow_id: workflowId,
    workflow_revision: 7,
    run_id: sourceRunId,
    replay_of: null,
    run_state: "running",
    event_count: 6,
    last_event_sequence: 6,
    capability_snapshot_digest: "sha256:0123456789abcdef",
    node_status_counts: { pending: 1, running: 1, succeeded: 2, failed: 0, blocked: 0 },
    ...overrides
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
