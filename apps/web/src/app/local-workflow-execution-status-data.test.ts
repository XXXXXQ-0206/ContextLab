import assert from "node:assert/strict";
import test from "node:test";
import {
  LocalWorkflowExecutionStatusProxyError,
  adaptLocalWorkflowExecutionStatusV1,
  loadLocalWorkflowExecutionStatus,
  type LocalWorkflowExecutionStatusTarget
} from "./local-workflow-execution-status-data";

const target: LocalWorkflowExecutionStatusTarget = Object.freeze({
  context_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  binding_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  workflow_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
  workflow_revision: 7,
  run_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee",
  capability: Object.freeze({
    en: "Workflow execution status",
    zh: "工作流执行状态"
  })
});

const originalFetch = globalThis.fetch;

test("Web data reads the same-origin BFF with request-scoped bearer, cookie omission, and no-store", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse(statusPayload());
  }) as typeof fetch;

  try {
    const status = await loadLocalWorkflowExecutionStatus(target, "request-token");
    assert.equal(status.run_state, "failed");
    assert.equal(
      requests[0]?.input,
      "/api/local/contexts/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/workflow/runs/eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee/status"
    );
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Web data preserves 503 for the unavailable resource state and rejects scope drift", async () => {
  globalThis.fetch = (async () => jsonResponse(
    { error: "workflow_execution_status_unavailable", message: "unavailable" },
    { status: 503 }
  )) as typeof fetch;
  try {
    await assert.rejects(
      () => loadLocalWorkflowExecutionStatus(target, "request-token"),
      (error: unknown) => error instanceof LocalWorkflowExecutionStatusProxyError && error.status === 503
    );

    const resource = adaptLocalWorkflowExecutionStatusV1({ kind: "unavailable", target });
    assert.equal(resource.state, "unavailable");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Web data redacts upstream error messages before they enter the resource contract", async () => {
  globalThis.fetch = (async () => jsonResponse(
    { error: "workflow_execution_status_failed", message: "private upstream diagnostic" },
    { status: 502 }
  )) as typeof fetch;
  try {
    await assert.rejects(
      () => loadLocalWorkflowExecutionStatus(target, "request-token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalWorkflowExecutionStatusProxyError);
        assert.equal(error.body.message, "ContextLab local API request failed with status 502");
        assert.doesNotMatch(error.body.message, /private upstream diagnostic/);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("Web data adapts loading, error, empty, unavailable, and available without changing run semantics", () => {
  const states = ["loading", "error", "empty", "unavailable"] as const;
  for (const kind of states) {
    const dto = adaptLocalWorkflowExecutionStatusV1({ kind, target });
    assert.equal(dto.state, kind);
    assert.equal(dto.status, undefined);
  }

  const dto = adaptLocalWorkflowExecutionStatusV1({ kind: "ready", target, status: statusPayload() });
  assert.equal(dto.state, "available");
  assert.equal(dto.status?.run_state, "failed");
  assert.equal(dto.status?.replay_of, "ffffffff-ffff-4fff-8fff-ffffffffffff");
});

function statusPayload() {
  return {
    schema_version: "contextlab.local-workflow-execution-status.v1",
    context_id: target.context_id,
    context_commit_id: target.context_commit_id,
    binding_id: target.binding_id,
    workflow_id: target.workflow_id,
    workflow_revision: target.workflow_revision,
    run_id: target.run_id,
    replay_of: "ffffffff-ffff-4fff-8fff-ffffffffffff",
    run_state: "failed",
    event_count: 6,
    last_event_sequence: 6,
    capability_snapshot_digest: "sha256:0123456789abcdef",
    node_status_counts: { pending: 0, running: 0, succeeded: 2, failed: 1, blocked: 3 }
  } as const;
}

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
