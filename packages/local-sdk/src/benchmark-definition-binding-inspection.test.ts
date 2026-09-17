import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalBenchmarkDefinitionBindingInspectionClient,
  parseLocalBenchmarkDefinitionBindingList
} from "./benchmark-definition-authoring";

const projectId = "22222222-2222-4222-8222-222222222222";
const contextId = "33333333-3333-4333-8333-333333333333";
const commitId = "44444444-4444-4444-8444-444444444444";

test("parses a redacted exact-scope benchmark binding list", () => {
  const parsed = parseLocalBenchmarkDefinitionBindingList(payload());
  assert.equal(parsed.project_id, projectId);
  assert.equal(parsed.context_id, contextId);
  assert.equal(parsed.commit_id, commitId);
  assert.equal(parsed.bindings[0]?.suite_name, "Release gate");
  assert.deepEqual(parsed.bindings[0]?.dataset_names, ["Safety prompts"]);
  assert.equal(Object.isFrozen(parsed), true);
  assert.equal(Object.isFrozen(parsed.bindings), true);
  assert.throws(
    () => parseLocalBenchmarkDefinitionBindingList({ ...payload(), raw_cases: [] }),
    /unexpected shape|unknown field|exact keys/i
  );
});

test("lists exact bindings with request-scoped bearer and no cookies", async () => {
  const requests: Array<{ url: string; init?: RequestInit }> = [];
  const client = new ContextLabLocalBenchmarkDefinitionBindingInspectionClient({
    baseUrl: "http://contextlab.test",
    fetch: (async (input: string | URL | Request, init?: RequestInit) => {
      requests.push({ url: String(input), init });
      return new Response(JSON.stringify(payload()), {
        status: 200,
        headers: { "content-type": "application/json" }
      });
    }) as typeof fetch
  });

  const result = await client.listBenchmarkDefinitionBindings(
    projectId,
    contextId,
    commitId,
    { bearerToken: "request-token" }
  );
  assert.equal(
    requests[0]?.url,
    `http://contextlab.test/api/v1/local/projects/${projectId}/contexts/${contextId}/commits/${commitId}/benchmark-definition-bindings`
  );
  const headers = new Headers(requests[0]?.init?.headers);
  assert.equal(headers.get("authorization"), "Bearer request-token");
  assert.equal(headers.get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
  assert.equal(result.bindings.length, 1);
});

function payload() {
  return {
    schema_version: "contextlab.local-benchmark-definition-binding-inspection.v1",
    project_id: projectId,
    context_id: contextId,
    commit_id: commitId,
    bindings: [{
      binding_id: "55555555-5555-4555-8555-555555555555",
      project_id: projectId,
      context_id: contextId,
      commit_id: commitId,
      branch_name: "main",
      definition_schema_version: 1,
      suite_id: "66666666-6666-4666-8666-666666666666",
      suite_name: "Release gate",
      dataset_ids: ["77777777-7777-4777-8777-777777777777"],
      dataset_names: ["Safety prompts"],
      captured_at: "2026-07-27T08:00:00.000Z"
    }]
  };
}
