import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { adaptLocalWorkflowExecutionStatusV1 } from "./local-workflow-execution-status-data";
import { LocalWorkflowExecutionStatusScreen } from "./local-workflow-execution-status-screen";

const target = {
  context_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  binding_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  workflow_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
  workflow_revision: 7,
  run_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee",
  capability: { en: "Workflow execution status", zh: "工作流执行状态" }
} as const;

test("screen keeps every capability state accessible and bilingual through shared CapabilityStateScreen", () => {
  const states = ["loading", "error", "empty", "available", "unavailable"] as const;
  for (const kind of states) {
    const resource = kind === "available"
      ? { kind: "ready" as const, target, status: statusPayload() }
      : { kind, target };
    const markup = renderToStaticMarkup(
      <LocalWorkflowExecutionStatusScreen resource={adaptLocalWorkflowExecutionStatusV1(resource)} />
    );

    assert.match(markup, new RegExp(`data-state="${kind}"`));
    assert.match(markup, /Workflow execution status \/ 工作流执行状态/);
    assert.match(markup, /Capability status \/ 能力状态/);
    assert.match(markup, /aria-live="(polite|assertive)"/);
    if (kind === "error") {
      assert.match(markup, /role="alert"/);
      assert.match(markup, /aria-live="assertive"/);
    } else {
      assert.match(markup, /role="status"/);
      assert.match(markup, /aria-live="polite"/);
    }
    if (kind === "loading") assert.match(markup, /aria-busy="true"/);
  }
});

test("available screen renders only redacted status metadata and never node or edge collections", () => {
  const markup = renderToStaticMarkup(
    <LocalWorkflowExecutionStatusScreen
      resource={adaptLocalWorkflowExecutionStatusV1({ kind: "ready", target, status: statusPayload() })}
    />
  );

  assert.match(markup, /Failed \/ 已失败/);
  assert.match(markup, /回放源/);
  assert.match(markup, /Redacted node status counts \/ 脱敏节点状态计数/);
  assert.doesNotMatch(markup, /failure_code|provider_request|api_key|&quot;nodes&quot;|&quot;edges&quot;/);
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
