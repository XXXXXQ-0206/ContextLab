import assert from "node:assert/strict";
import test from "node:test";
import {
  authorLocalBenchmarkDefinition,
  LocalBenchmarkDefinitionAuthoringProxyError,
  parseLocalBenchmarkDefinitionAuthoringResult,
  type LocalBenchmarkDefinitionAuthoringCommand,
  type LocalBenchmarkDefinitionAuthoringTarget
} from "./local-benchmark-definition-authoring-data";

const originalFetch = globalThis.fetch;

const target: LocalBenchmarkDefinitionAuthoringTarget = Object.freeze({
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
});

const command: LocalBenchmarkDefinitionAuthoringCommand = Object.freeze({
  schema_version: 1,
  binding_id: "11111111-1111-4111-8111-111111111111",
  branch_name: "main",
  expected_head_commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  datasets: Object.freeze([
    Object.freeze({
      id: "22222222-2222-4222-8222-222222222222",
      name: "Safety prompts",
      cases: Object.freeze([
        Object.freeze({
          id: "33333333-3333-4333-8333-333333333333",
          name: "Refusal",
          input: Object.freeze({ prompt: "unsafe request" }),
          expected_output: Object.freeze({ mode: "unspecified" as const })
        })
      ])
    })
  ]),
  suite: Object.freeze({
    id: "44444444-4444-4444-8444-444444444444",
    name: "Release gate",
    dataset_ids: Object.freeze(["22222222-2222-4222-8222-222222222222"]),
    thresholds: Object.freeze([
      Object.freeze({ metric: "accuracy", direction: "minimum" as const, value: 0.9 })
    ])
  })
});

test("authoring data posts one exact Context commit through the private same-origin adapter", async () => {
  const requests: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return jsonResponse(successPayload());
  }) as typeof fetch;

  try {
    const result = await authorLocalBenchmarkDefinition(
      target,
      " request-token ",
      " request-key ",
      command
    );

    assert.equal(requests[0]?.url, "/api/local/projects/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/contexts/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/commits/cccccccc-cccc-4ccc-8ccc-cccccccccccc/benchmark-definition-bindings");
    const headers = new Headers(requests[0]?.init?.headers);
    assert.equal(headers.get("authorization"), "Bearer request-token");
    assert.equal(headers.get("idempotency-key"), "request-key");
    assert.equal(headers.get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.deepEqual(JSON.parse(String(requests[0]?.init?.body)), command);
    assert.equal(result.commit_id, target.commitId);
    assert.equal(Object.isFrozen(result), true);
    assert.equal(Object.isFrozen(result.dataset_ids), true);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("authoring data fails closed on malformed or scope-mismatched success payloads", async (t) => {
  await t.test("malformed payload", () => {
    assert.throws(
      () => parseLocalBenchmarkDefinitionAuthoringResult({ ...successPayload(), disposition: "updated" }),
      TypeError
    );
  });

  await t.test("scope mismatch", async () => {
    const payload = successPayload();
    payload.commit_id = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
    globalThis.fetch = (async () => jsonResponse(payload)) as typeof fetch;
    try {
      await assert.rejects(
        () => authorLocalBenchmarkDefinition(target, "token", "key", command),
        /requested scope/
      );
    } finally {
      globalThis.fetch = originalFetch;
    }
  });
});

test("authoring data preserves only structured private adapter failures", async () => {
  globalThis.fetch = (async () =>
    jsonResponse(
      { error: "benchmark_definition_conflict", message: "Branch head moved", trace: "private" },
      { status: 409, headers: { "retry-after": "3" } }
    )) as typeof fetch;

  try {
    await assert.rejects(
      () => authorLocalBenchmarkDefinition(target, "token", "key", command),
      (error: unknown) => {
        assert.ok(error instanceof LocalBenchmarkDefinitionAuthoringProxyError);
        assert.equal(error.status, 409);
        assert.equal(error.retryAfterMs, 3_000);
        assert.deepEqual(error.body, {
          error: "benchmark_definition_conflict",
          message: "Branch head moved"
        });
        return true;
      }
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function successPayload() {
  return {
    schema_version: "contextlab.local-benchmark-definition-authoring.v1",
    disposition: "created",
    message: { en: "Benchmark definitions authored.", zh: "Benchmark 定义已创作。" },
    binding_id: "11111111-1111-4111-8111-111111111111",
    project_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    context_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    commit_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    branch_name: "main",
    definition_schema_version: 1,
    suite_id: "44444444-4444-4444-8444-444444444444",
    dataset_ids: ["22222222-2222-4222-8222-222222222222"],
    captured_at: "2026-07-27T08:00:00.000Z"
  };
}

function jsonResponse(body: unknown, init: ResponseInit = {}) {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" },
    ...init
  });
}
