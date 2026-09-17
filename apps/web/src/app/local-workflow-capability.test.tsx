import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { loadLocalWorkflowCapabilityStatus } from "./local-workflow-capability-data";
import { LocalWorkflowCapabilityInspector } from "./local-workflow-capability-inspector";

const originalFetch = globalThis.fetch;

test("workflow capability data client reads the exact same-origin private scope without cookies", async () => {
  const requests: Array<{ input: string; init: RequestInit | undefined }> = [];
  globalThis.fetch = (async (input: string | URL | Request, init?: RequestInit) => {
    requests.push({ input: String(input), init });
    return jsonResponse(workflowCapabilityPayload());
  }) as typeof fetch;

  try {
    const status = await loadLocalWorkflowCapabilityStatus("context/id", "request-token");

    assert.equal(status.availability, "unavailable");
    assert.equal(requests[0]?.input, "/api/local/contexts/context%2Fid/workflow/capability-status");
    assert.equal(new Headers(requests[0]?.init?.headers).get("authorization"), "Bearer request-token");
    assert.equal(new Headers(requests[0]?.init?.headers).get("cookie"), null);
    assert.equal(requests[0]?.init?.credentials, "omit");
    assert.equal(requests[0]?.init?.cache, "no-store");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("local workflow capability inspector renders a bilingual private read control with an initially disabled request", () => {
  const markup = renderToStaticMarkup(<LocalWorkflowCapabilityInspector contextId="context-a" />);

  assert.match(markup, new RegExp("Local Workflow Capability / 本地工作流能力"));
  assert.match(markup, new RegExp("Bearer token / 访问令牌"));
  assert.match(markup, new RegExp("Memory only / 仅内存"));
  assert.match(markup, new RegExp("Inspect capability / 审阅能力"));
  assert.match(markup, /disabled=""/);
  assert.match(markup, /workflow-capability-heading/);
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

function jsonResponse(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    headers: { "content-type": "application/json" }
  });
}
