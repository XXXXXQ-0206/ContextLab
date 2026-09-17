import assert from "node:assert/strict";
import test from "node:test";
import {
  createLocalBenchmarkDefinitionBindingInspectionResource,
  loadLocalBenchmarkDefinitionBindings,
  LocalBenchmarkDefinitionBindingInspectionProxyError,
  type LocalBenchmarkDefinitionBindingInspectionTarget
} from "./local-benchmark-definition-binding-inspection-data";

const originalFetch = globalThis.fetch;
const target: LocalBenchmarkDefinitionBindingInspectionTarget = Object.freeze({
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
});

test("loads an exact redacted binding list with request-scoped transport", async () => {
  const requests: Array<{ url: string; init?: RequestInit }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ url: String(input), init });
    return new Response(JSON.stringify(payload()), { headers: { "content-type": "application/json" } });
  }) as typeof fetch;
  try {
    const result = await loadLocalBenchmarkDefinitionBindings(target, " request-token ");
    assert.equal(requests[0]?.url, "/api/local/projects/aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa/contexts/bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb/commits/cccccccc-cccc-4ccc-8ccc-cccccccccccc/benchmark-definition-bindings");
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
    assert.equal(result.bindings[0]?.suite_name, "Release gate");
    assert.equal(Object.isFrozen(result), true);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("maps typed failures and rejects available scope drift", async () => {
  globalThis.fetch = (async () => new Response(JSON.stringify({ error: "context_read_forbidden", message: "denied" }), { status: 403 })) as typeof fetch;
  try {
    await assert.rejects(
      () => loadLocalBenchmarkDefinitionBindings(target, "token"),
      (error: unknown) => error instanceof LocalBenchmarkDefinitionBindingInspectionProxyError && error.status === 403
    );
    assert.throws(() => createLocalBenchmarkDefinitionBindingInspectionResource({
      state: "available",
      target,
      result: { ...payload(), context_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd" } as never
    }), /exact target/);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

function payload() {
  return {
    schema_version: "contextlab.local-benchmark-definition-binding-inspection.v1",
    project_id: target.projectId,
    context_id: target.contextId,
    commit_id: target.commitId,
    bindings: [{
      binding_id: "11111111-1111-4111-8111-111111111111",
      project_id: target.projectId,
      context_id: target.contextId,
      commit_id: target.commitId,
      branch_name: "main",
      definition_schema_version: 1,
      suite_id: "22222222-2222-4222-8222-222222222222",
      suite_name: "Release gate",
      dataset_ids: ["33333333-3333-4333-8333-333333333333"],
      dataset_names: ["Safety prompts"],
      captured_at: "2026-07-27T08:00:00.000Z"
    }]
  };
}
