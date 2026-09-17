import assert from "node:assert/strict";
import test from "node:test";
import {
  adaptLocalWorkflowExecutionStatusV1,
  type LocalWorkflowExecutionStatusResource
} from "./local-workflow-execution-status-data";
import { presentLocalWorkflowExecutionStatus } from "./local-workflow-execution-status-presenter";

const target = {
  context_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  context_commit_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  binding_id: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  workflow_id: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
  workflow_revision: 7,
  run_id: "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee",
  capability: { en: "Workflow execution status", zh: "工作流执行状态" }
} as const;

test("presenter routes all five resource states through the shared capability state model", () => {
  const states = ["loading", "error", "empty", "unavailable"] as const;
  for (const kind of states) {
    const resource: LocalWorkflowExecutionStatusResource = { kind, target };
    const view = presentLocalWorkflowExecutionStatus(resource);
    assert.equal(view.capabilityState.state, kind);
    assert.match(view.capabilityState.capabilityLabel, /Workflow execution status \/ 工作流执行状态/);
    assert.equal(view.execution, null);
    assert.equal(Object.isFrozen(view), true);
  }
});

test("presenter does not expose caller-supplied error messages", () => {
  const view = presentLocalWorkflowExecutionStatus({
    kind: "error",
    target,
    message: "private upstream diagnostic"
  });

  assert.doesNotMatch(JSON.stringify(view), /private upstream diagnostic/);
});

test("presenter exposes exact scope, redacted counters, and source replay metadata", () => {
  const resource: LocalWorkflowExecutionStatusResource = {
    kind: "ready",
    target,
    status: {
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
    }
  };
  const view = presentLocalWorkflowExecutionStatus(adaptLocalWorkflowExecutionStatusV1(resource));

  assert.equal(view.capabilityState.state, "available");
  assert.equal(view.execution?.runState, "Failed / 已失败");
  assert.equal(view.execution?.replayOf, "ffffffff-ffff-4fff-8fff-ffffffffffff / 回放源");
  assert.deepEqual(view.execution?.nodeStatusCounts, {
    pending: 0,
    running: 0,
    succeeded: 2,
    failed: 1,
    blocked: 3
  });
  assert.deepEqual(view.scope[0], {
    id: "context-id",
    label: "Context / 上下文",
    value: target.context_id
  });
});
