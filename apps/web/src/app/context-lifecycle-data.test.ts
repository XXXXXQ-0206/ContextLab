import assert from "node:assert/strict";
import test from "node:test";
import {
  LocalLifecycleProxyError,
  loadLocalContextLifecycleState,
  readLocalLifecycleComponentMetadata,
  readLocalLifecycleContextMetadata,
  submitLocalComponentLifecycleCommit
} from "./context-lifecycle-data";

const originalFetch = globalThis.fetch;
const contextId = "11111111-1111-4111-8111-111111111111";
const commitA = "22222222-2222-4222-8222-222222222222";
const commitB = "33333333-3333-4333-8333-333333333333";
const componentA = "44444444-4444-4444-8444-444444444444";

test("lifecycle data adapter reads metadata for the exact loaded component", () => {
  const state = {
    schema_version: "contextlab.local-context-lifecycle-state.v1",
    context_id: contextId,
    commit_id: commitA,
    components: [
      {
        component_id: componentA,
        component_kind: "prompt" as const,
        name: "Instruction",
        metadata: { locale: "zh-CN", audience: "operator" },
        content: "private content",
        content_hash: "hash-a",
        creation_commit_id: commitA,
        content_commit_id: commitA
      }
    ],
    graph_snapshot: {
      project_id: "55555555-5555-4555-8555-555555555555",
      context_id: contextId,
      commit_id: commitA,
      graph: { nodes: {}, edges: [] },
      captured_at: "2026-07-30T00:00:00Z",
      schema_version: 1
    },
    metadata: null
  };

  assert.deepEqual(readLocalLifecycleComponentMetadata(state, componentA), {
    component_id: componentA,
    name: "Instruction",
    metadata: { locale: "zh-CN", audience: "operator" }
  });
  assert.equal(readLocalLifecycleComponentMetadata(state, "component-missing"), null);
});

test("lifecycle data adapter preserves the typed Context metadata at the selected commit", () => {
  const metadata = {
    created_at: "2026-07-30T00:00:00Z",
    updated_at: "2026-07-30T01:00:00Z",
    labels: { owner: "luna" }
  };

  assert.deepEqual(
    readLocalLifecycleContextMetadata({ metadata }),
    metadata
  );
  assert.equal(readLocalLifecycleContextMetadata({ metadata: null }), null);
});

test("lifecycle data client forwards only request-scoped credentials to same-origin routes", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    if (init?.method === "POST") {
      return jsonResponse({
        schema_version: "contextlab.local-component-lifecycle-commit.v1",
        disposition: "created",
        commit_id: commitB,
        snapshot: {
          project_id: "55555555-5555-4555-8555-555555555555",
          context_id: contextId,
          commit_id: commitB,
          graph: { nodes: {}, edges: [] },
          captured_at: "2026-07-30T00:00:00Z",
          schema_version: 1
        }
      });
    }

    return jsonResponse({
      schema_version: "contextlab.local-context-lifecycle-state.v1",
      context_id: contextId,
      commit_id: commitA,
      metadata: null,
      components: [],
      graph_snapshot: {
        project_id: "55555555-5555-4555-8555-555555555555",
        context_id: contextId,
        commit_id: commitA,
        graph: { nodes: {}, edges: [] },
        captured_at: "2026-07-30T00:00:00Z",
        schema_version: 1
      }
    });
  }) as typeof fetch;

  try {
    await loadLocalContextLifecycleState(contextId, commitA, "request-token");
    await submitLocalComponentLifecycleCommit(contextId, "request-token", "request-key", {
      branch_name: "main",
      expected_head_commit_id: "commit-a",
      message: "Create prompt",
      operation: {
        kind: "create",
        component_kind: "prompt",
        name: "Instruction",
        metadata: null,
        content: "Use committed Context."
      }
    });

    assert.equal(requests[0]?.input, `/api/local/contexts/${contextId}/commits/${commitA}/lifecycle-state`);
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("idempotency-key"), null);
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(requests[1]?.input, `/api/local/contexts/${contextId}/component-lifecycle-commits`);
    assert.equal(new Headers(requests[1]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[1]?.init?.headers).get("idempotency-key"), "request-key");
    assert.equal(new Headers(requests[1]?.init?.headers).get("cookie"), null);
    assert.equal(requests[1]?.init?.credentials, "omit");
    assert.equal(requests[1]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("lifecycle data client fails closed when the response scope does not match the requested Context commit", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse({
      schema_version: "contextlab.local-context-lifecycle-state.v1",
      context_id: contextId,
      commit_id: commitB,
      metadata: null,
      components: [],
      graph_snapshot: {
        project_id: "55555555-5555-4555-8555-555555555555",
        context_id: contextId,
        commit_id: commitB,
        graph: { nodes: {}, edges: [] },
        captured_at: "2026-07-30T00:00:00Z",
        schema_version: 1
      }
    });
  }) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalContextLifecycleState(contextId, commitA, "request-token"),
      (error: unknown) => error instanceof TypeError
        && error.message.includes("does not match the requested scope")
    );
    assert.equal(requests[0]?.input, `/api/local/contexts/${contextId}/commits/${commitA}/lifecycle-state`);
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("lifecycle data client removes an attempted body rewrite from descriptor commits", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse({
      schema_version: "contextlab.local-component-lifecycle-commit.v1",
      disposition: "created",
      commit_id: commitB,
      snapshot: {
        project_id: "55555555-5555-4555-8555-555555555555",
        context_id: contextId,
        commit_id: commitB,
        graph: { nodes: {}, edges: [] },
        captured_at: "2026-07-30T00:00:00Z",
        schema_version: 1
      }
    });
  }) as typeof fetch;

  try {
    await submitLocalComponentLifecycleCommit(contextId, "request-token", "request-key", {
      branch_name: "main",
      expected_head_commit_id: "commit-a",
      message: "Correct descriptor",
      operation: {
        kind: "update_descriptor",
        component_id: componentA,
        name: "Instruction / 操作说明",
        metadata: { locale: "zh-CN" },
        content: "Attempted immutable body rewrite"
      }
    } as never);

    const requestBody = JSON.parse(String(requests[0]?.init?.body)) as {
      operation: Record<string, unknown>;
    };
    assert.deepEqual(requestBody.operation, {
      kind: "update_descriptor",
      component_id: componentA,
      name: "Instruction / 操作说明",
      metadata: { locale: "zh-CN" }
    });
    assert.equal("content" in requestBody.operation, false);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("lifecycle data client preserves structured proxy failures", async () => {
  globalThis.fetch = (async () =>
    jsonResponse(
      { error: "context_lifecycle_state_conflict", message: "The selected head is stale" },
      { status: 409 }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalContextLifecycleState(contextId, commitA, "request-token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalLifecycleProxyError);
        assert.equal(error.status, 409);
        assert.equal(error.body.error, "context_lifecycle_state_conflict");
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("lifecycle data client redacts upstream conflict diagnostics", async () => {
  const upstreamMessage = "private branch topology and tenant identifier";
  globalThis.fetch = (async () =>
    jsonResponse(
      { error: "context_lifecycle_state_conflict", message: upstreamMessage },
      { status: 409 }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalContextLifecycleState(contextId, commitA, "request-token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalLifecycleProxyError);
        assert.equal(error.status, 409);
        assert.equal(error.body.error, "context_lifecycle_state_conflict");
        assert.equal(error.body.message, "Context lifecycle request failed with status 409");
        assert.notEqual(error.message, upstreamMessage);
        assert.notEqual(error.body.message, upstreamMessage);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("lifecycle data client redacts upstream rate-limit diagnostics but preserves retry-after", async () => {
  const upstreamMessage = "private rate-limit bucket and internal quota policy";
  globalThis.fetch = (async () =>
    jsonResponse(
      { error: "context_lifecycle_rate_limited", message: upstreamMessage },
      { status: 429, headers: { "retry-after": "7" } }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalContextLifecycleState(contextId, commitA, "request-token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalLifecycleProxyError);
        assert.equal(error.status, 429);
        assert.equal(error.retryAfterMs, 7_000);
        assert.equal(error.body.error, "context_lifecycle_rate_limited");
        assert.equal(error.body.message, "Context lifecycle request failed with status 429");
        assert.notEqual(error.message, upstreamMessage);
        assert.notEqual(error.body.message, upstreamMessage);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("lifecycle data client redacts upstream diagnostics for unknown statuses", async () => {
  const upstreamMessage = "private stack trace and database connection details";
  globalThis.fetch = (async () =>
    jsonResponse(
      { error: "private_internal_error", message: upstreamMessage },
      { status: 599 }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => loadLocalContextLifecycleState(contextId, commitA, "request-token"),
      (error: unknown) => {
        assert.ok(error instanceof LocalLifecycleProxyError);
        assert.equal(error.status, 599);
        assert.equal(error.body.error, "private_internal_error");
        assert.equal(error.body.message, "Context lifecycle request failed with status 599");
        assert.notEqual(error.message, upstreamMessage);
        assert.notEqual(error.body.message, upstreamMessage);
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
