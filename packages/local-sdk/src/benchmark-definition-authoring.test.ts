import assert from "node:assert/strict";
import test from "node:test";

import {
  ContextLabLocalBenchmarkDefinitionClient,
  ContextLabLocalBenchmarkDefinitionConflictError,
  ContextLabLocalBenchmarkDefinitionReplayConflictError,
  LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1,
  parseLocalBenchmarkDefinitionAuthoringRequest,
  parseLocalBenchmarkDefinitionBinding,
  type LocalBenchmarkDefinitionBindingRequest
} from "./benchmark-definition-authoring";

const PROJECT_ID = "22222222-2222-4222-8222-222222222222";
const CONTEXT_ID = "11111111-1111-4111-8111-111111111111";
const COMMIT_ID = "77777777-7777-4777-8777-777777777777";
const BINDING_ID = "33333333-3333-4333-8333-333333333333";
const DATASET_ID = "44444444-4444-4444-8444-444444444444";
const CASE_ID = "55555555-5555-4555-8555-555555555555";
const SUITE_ID = "66666666-6666-4666-8666-666666666666";

const request: LocalBenchmarkDefinitionBindingRequest = {
  schema_version: 1,
  binding_id: BINDING_ID,
  branch_name: "main",
  expected_head_commit_id: COMMIT_ID,
  datasets: [{
    id: DATASET_ID,
    name: "smoke",
    cases: [{
      id: CASE_ID,
      name: "case",
      input: { value: "hello" },
      expected_output: { mode: "unspecified" }
    }]
  }],
  suite: {
    id: SUITE_ID,
    name: "smoke suite",
    dataset_ids: [DATASET_ID],
    thresholds: [{ metric: "accuracy", direction: "minimum", value: 0.5 }]
  }
};

test("parses the exact server-owned V1 receipt and preserves stable identifiers", () => {
  const binding = parseLocalBenchmarkDefinitionBinding(authoringResponse());

  assert.equal(binding.commit_id, COMMIT_ID);
  assert.equal(binding.binding_id, BINDING_ID);
  assert.deepEqual(binding.dataset_ids, [DATASET_ID]);
  assert.equal(binding.suite_id, SUITE_ID);
  assert.equal(Object.isFrozen(binding), true);
  assert.equal(Object.isFrozen(binding.dataset_ids), true);
});

test("client posts the exact V1 request with request-scoped bearer and no cookies", async () => {
  let captured: { url: string; init: RequestInit } | undefined;
  const client = new ContextLabLocalBenchmarkDefinitionClient({
    baseUrl: "http://contextlab.test/",
    fetch: async (url, init = {}) => {
      captured = { url: String(url), init };
      return jsonResponse(authoringResponse("replayed"));
    }
  });

  const binding = await client.createBenchmarkDefinitionBinding(
    PROJECT_ID,
    CONTEXT_ID,
    COMMIT_ID,
    request,
    { bearerToken: "request-token", idempotencyKey: "authoring-1" }
  );

  assert.equal(binding.disposition, "replayed");
  assert.ok(captured);
  assert.equal(
    captured.url,
    `http://contextlab.test/api/v1/local/projects/${PROJECT_ID}/contexts/${CONTEXT_ID}/commits/${COMMIT_ID}/benchmark-definition-bindings`
  );
  assert.equal(captured.init.method, "POST");
  assert.equal(captured.init.credentials, "omit");
  const headers = new Headers(captured.init.headers);
  assert.equal(headers.get("authorization"), "Bearer request-token");
  assert.equal(headers.get("idempotency-key"), "authoring-1");
  assert.equal(headers.get("content-type"), "application/json");
  assert.equal(headers.get("cookie"), null);
  assert.deepEqual(JSON.parse(String(captured.init.body)), request);
});

test("maps changed idempotency payloads to a typed replay conflict", async () => {
  const client = errorClient("storage_idempotency_conflict");

  await assert.rejects(
    () => create(client),
    (error: unknown) => {
      assert.ok(error instanceof ContextLabLocalBenchmarkDefinitionReplayConflictError);
      assert.equal(error.status, 409);
      assert.equal(error.code, "storage_idempotency_conflict");
      return true;
    }
  );
});

for (const code of ["storage_branch_head_conflict", "storage_benchmark_definition_conflict"]) {
  test(`maps ${code} to a typed immutable authoring conflict`, async () => {
    const client = errorClient(code);

    await assert.rejects(
      () => create(client),
      (error: unknown) => {
        assert.ok(error instanceof ContextLabLocalBenchmarkDefinitionConflictError);
        assert.equal(error.status, 409);
        assert.equal(error.code, code);
        return true;
      }
    );
  });
}

test("rejects unknown schema, response fields, and raw benchmark payloads", () => {
  assert.throws(() => parseLocalBenchmarkDefinitionBinding({
    ...authoringResponse(),
    schema_version: "contextlab.local-benchmark-definition-authoring.v2"
  }), TypeError);
  assert.throws(() => parseLocalBenchmarkDefinitionBinding({
    ...authoringResponse(),
    raw_cases: []
  }), TypeError);
  assert.throws(() => parseLocalBenchmarkDefinitionBinding({
    ...authoringResponse(),
    input: { secret: true }
  }), TypeError);
});

test("rejects malformed request shapes and membership before transport", async () => {
  const malformed = structuredClone(request) as LocalBenchmarkDefinitionBindingRequest;
  malformed.suite.dataset_ids = [];
  assert.throws(() => parseLocalBenchmarkDefinitionAuthoringRequest(malformed), TypeError);

  let called = false;
  const client = new ContextLabLocalBenchmarkDefinitionClient({
    fetch: async () => {
      called = true;
      return jsonResponse(authoringResponse());
    }
  });
  await assert.rejects(
    () => client.createBenchmarkDefinitionBinding(
      PROJECT_ID,
      CONTEXT_ID,
      COMMIT_ID,
      malformed,
      { bearerToken: "token", idempotencyKey: "authoring-1" }
    ),
    TypeError
  );
  assert.equal(called, false);
});

test("rejects empty request credentials before transport", async () => {
  const client = new ContextLabLocalBenchmarkDefinitionClient({
    fetch: async () => {
      throw new Error("transport must not be called");
    }
  });

  await assert.rejects(
    () => client.createBenchmarkDefinitionBinding(
      PROJECT_ID,
      CONTEXT_ID,
      COMMIT_ID,
      request,
      { bearerToken: "", idempotencyKey: "authoring-1" }
    ),
    TypeError
  );
});

test("rejects response scope or stable-ID drift", async () => {
  const client = new ContextLabLocalBenchmarkDefinitionClient({
    fetch: async () => jsonResponse({
      ...authoringResponse(),
      binding_id: "88888888-8888-4888-8888-888888888888"
    })
  });

  await assert.rejects(() => create(client), TypeError);
});

function authoringResponse(disposition: "created" | "replayed" = "created") {
  return {
    schema_version: LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1,
    disposition,
    message: disposition === "created"
      ? { en: "Benchmark definitions authored.", zh: "Benchmark 定义已创建。" }
      : {
          en: "Benchmark definition authoring request replayed.",
          zh: "Benchmark 定义创建请求已重放。"
        },
    binding_id: BINDING_ID,
    project_id: PROJECT_ID,
    context_id: CONTEXT_ID,
    commit_id: COMMIT_ID,
    branch_name: request.branch_name,
    definition_schema_version: 1 as const,
    dataset_ids: request.suite.dataset_ids,
    suite_id: request.suite.id,
    captured_at: "2026-07-27T12:00:00Z"
  };
}

function errorClient(code: string) {
  return new ContextLabLocalBenchmarkDefinitionClient({
    fetch: async () => jsonResponse({ error: code, message: "authoring conflict" }, 409)
  });
}

function create(client: ContextLabLocalBenchmarkDefinitionClient) {
  return client.createBenchmarkDefinitionBinding(
    PROJECT_ID,
    CONTEXT_ID,
    COMMIT_ID,
    request,
    { bearerToken: "token", idempotencyKey: "authoring-1" }
  );
}

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" }
  });
}
