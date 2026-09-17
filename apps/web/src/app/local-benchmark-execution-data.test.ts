import assert from "node:assert/strict";
import test from "node:test";
import {
  createLocalBenchmarkExecutionResource,
  executeLocalBenchmarkExecution,
  LocalBenchmarkExecutionProxyError
} from "./local-benchmark-execution-data";

const target = Object.freeze({
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
});

const command = {
  schema_version: 1 as const,
  binding_id: "11111111-1111-4111-8111-111111111111",
  decision_id: "22222222-2222-4222-8222-222222222222",
  model_version: "deterministic-v1",
  temperature: 0.2,
  evaluator_key: "local-deterministic",
  evaluator_version: "v1"
};

const receipt = {
  schema_version: "contextlab.local-benchmark-execution.v1" as const,
  disposition: "replayed" as const,
  projection_disposition: "replayed" as const,
  project_id: target.projectId,
  context_id: target.contextId,
  commit_id: target.commitId,
  binding_id: command.binding_id,
  decision_id: command.decision_id,
  suite_id: "33333333-3333-4333-8333-333333333333",
  dataset_ids: ["44444444-4444-4444-8444-444444444444"],
  cohort_id: "55555555-5555-4555-8555-555555555555"
};

test("data adapter sends the request-memory token and returns a frozen replay receipt", async () => {
  let observed: RequestInit | undefined;
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async (_input, init) => {
    observed = init;
    return new Response(JSON.stringify(receipt), {
      status: 200,
      headers: { "content-type": "application/json" }
    });
  }) as typeof fetch;
  try {
    const result = await executeLocalBenchmarkExecution(target, command, " token ", " key ");
    assert.equal(result.disposition, "replayed");
    assert(Object.isFrozen(result));
    const headers = new Headers(observed?.headers);
    assert.equal(headers.get("authorization"), "Bearer token");
    assert.equal(headers.get("idempotency-key"), "key");
    assert.equal(observed?.credentials, "omit");
    assert.equal(observed?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("data adapter classifies conflict and rejects response scope drift", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = (async () => new Response(JSON.stringify({
    error: "benchmark_execution_conflict",
    message: "stable conflict"
  }), { status: 409 })) as typeof fetch;
  try {
    await assert.rejects(
      executeLocalBenchmarkExecution(target, command, "token", "key"),
      (error: unknown) => error instanceof LocalBenchmarkExecutionProxyError && error.status === 409
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  globalThis.fetch = (async () => new Response(JSON.stringify({ ...receipt, context_id: target.projectId }), { status: 201 })) as typeof fetch;
  try {
    await assert.rejects(executeLocalBenchmarkExecution(target, command, "token", "key"));
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("resource states remain frozen and preserve created, replayed, conflict, and unavailable semantics", () => {
  for (const state of ["loading", "empty", "conflict", "unavailable"] as const) {
    assert(Object.isFrozen(createLocalBenchmarkExecutionResource({ state, target, message: "state" })));
  }
  assert.equal(createLocalBenchmarkExecutionResource({ state: "created", target, receipt: { ...receipt, disposition: "created" } }).state, "created");
});
