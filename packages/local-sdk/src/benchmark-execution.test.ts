import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalBenchmarkExecutionClient,
  LOCAL_BENCHMARK_EXECUTION_SCHEMA_V1,
  parseLocalBenchmarkExecutionResponse
} from "./benchmark-execution";

const scope = {
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
};

const request = {
  schema_version: 1 as const,
  binding_id: "11111111-1111-4111-8111-111111111111",
  decision_id: "22222222-2222-4222-8222-222222222222",
  model_version: "deterministic-v1",
  temperature: 0.2,
  evaluator_key: "local-deterministic",
  evaluator_version: "v1"
};

const response = {
  schema_version: LOCAL_BENCHMARK_EXECUTION_SCHEMA_V1,
  disposition: "created" as const,
  projection_disposition: "created" as const,
  project_id: scope.projectId,
  context_id: scope.contextId,
  commit_id: scope.commitId,
  binding_id: request.binding_id,
  decision_id: request.decision_id,
  suite_id: "33333333-3333-4333-8333-333333333333",
  dataset_ids: ["44444444-4444-4444-8444-444444444444"],
  cohort_id: "55555555-5555-4555-8555-555555555555"
};

test("execution client sends a frozen request-memory bearer and no-store idempotent POST", async () => {
  const calls: Array<{ url: string; init: RequestInit | undefined }> = [];
  const client = new ContextLabLocalBenchmarkExecutionClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: async (input, init) => {
      calls.push({ url: String(input), init });
      return new Response(JSON.stringify(response), {
        status: 201,
        headers: { "content-type": "application/json" }
      });
    }
  });

  const result = await client.executeBenchmark(
    scope.projectId,
    scope.contextId,
    scope.commitId,
    request,
    { bearerToken: " request-token ", idempotencyKey: " execution-1 " }
  );

  assert.deepEqual(result, response);
  assert.equal(calls[0]?.url, `http://127.0.0.1:3100/api/v1/local/projects/${scope.projectId}/contexts/${scope.contextId}/commits/${scope.commitId}/benchmark-executions`);
  const headers = new Headers(calls[0]?.init?.headers);
  assert.equal(headers.get("authorization"), "Bearer request-token");
  assert.equal(headers.get("idempotency-key"), "execution-1");
  assert.equal(calls[0]?.init?.credentials, "omit");
  assert.equal(calls[0]?.init?.cache, "no-store");
  assert.deepEqual(JSON.parse(String(calls[0]?.init?.body)), request);
  assert(Object.isFrozen(result));
});

test("execution response parser rejects raw payloads, unknown fields, and scope drift", () => {
  assert.throws(() => parseLocalBenchmarkExecutionResponse({ ...response, output: "secret" }));
  assert.throws(() => parseLocalBenchmarkExecutionResponse({ ...response, extra: true }));
  assert.throws(() => parseLocalBenchmarkExecutionResponse({ ...response, context_id: "not-a-uuid" }));
  assert.throws(() => parseLocalBenchmarkExecutionResponse({ ...response, dataset_ids: [response.dataset_ids[0], response.dataset_ids[0]] }));
});

test("execution client classifies protected conflicts without exposing upstream fields", async () => {
  const client = new ContextLabLocalBenchmarkExecutionClient({
    fetch: async () => new Response(JSON.stringify({
      error: "benchmark_execution_conflict",
      message: "stable conflict",
      internal_trace: "must-not-cross"
    }), { status: 409, headers: { "content-type": "application/json" } })
  });

  await assert.rejects(
    client.executeBenchmark(scope.projectId, scope.contextId, scope.commitId, request, {
      bearerToken: "token",
      idempotencyKey: "key"
    }),
    (error: unknown) => error instanceof Error
      && error.name === "ContextLabLocalBenchmarkExecutionConflictError"
      && !error.message.includes("internal_trace")
  );
});
