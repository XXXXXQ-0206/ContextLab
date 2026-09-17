import assert from "node:assert/strict";
import test from "node:test";
import {
  ContextLabLocalClient,
  parseLocalWorkflowCapabilityStatus
} from "./index";

test("parses the exact fail-closed local workflow capability V1 response", () => {
  const status = parseLocalWorkflowCapabilityStatus(workflowCapabilityPayload());

  assert.deepEqual(status, workflowCapabilityPayload());
  assert.throws(
    () => parseLocalWorkflowCapabilityStatus({ ...workflowCapabilityPayload(), raw_workflow: {} }),
    TypeError
  );
});

test("reads the private workflow capability scope with request-scoped credentials only", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  const client = new ContextLabLocalClient({
    baseUrl: "http://127.0.0.1:3100/",
    fetch: async (input, init) => {
      requests.push({ input: String(input), init });
      return new Response(JSON.stringify(workflowCapabilityPayload()), {
        headers: { "content-type": "application/json" }
      });
    }
  });

  const status = await client.getWorkflowCapabilityStatus("context/id", { bearerToken: "request-token" });

  assert.equal(status.availability, "unavailable");
  assert.equal(
    requests[0]?.input,
    "http://127.0.0.1:3100/api/v1/local/contexts/context%2Fid/workflow/capability-status"
  );
  assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
  assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
  assert.equal(requests[0]?.init?.credentials, "omit");
});

function workflowCapabilityPayload() {
  return {
    schema_version: "contextlab.local-workflow-capability-status.v1",
    capability: "workflow",
    enabled: false,
    availability: "unavailable",
    reason: "shared_integration_not_registered"
  };
}
