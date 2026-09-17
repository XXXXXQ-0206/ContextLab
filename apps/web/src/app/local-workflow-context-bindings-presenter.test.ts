import assert from "node:assert/strict";
import test from "node:test";
import {
  LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
  parseLocalWorkflowContextBindingsV1,
  type LocalWorkflowContextBindingsResource
} from "./local-workflow-context-bindings-data";
import { presentLocalWorkflowContextBindings } from "./local-workflow-context-bindings-presenter";

const target = Object.freeze({
  context_id: "context-001",
  commit_id: "commit-042",
  capability: Object.freeze({
    en: "Workflow context bindings",
    zh: "Workflow 上下文绑定"
  })
});

const states = ["loading", "error", "empty", "available", "unavailable"] as const;

test("local Workflow binding presenter exposes every state with exact scope and redacted rows", () => {
  const available = parseLocalWorkflowContextBindingsV1({
    schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
    context_id: target.context_id,
    commit_id: target.commit_id,
    bindings: [{
      binding_id: "binding-001",
      workflow_id: "workflow-001",
      workflow_revision: 7,
      node_count: 3,
      edge_count: 2
    }]
  });
  const resources: Record<(typeof states)[number], LocalWorkflowContextBindingsResource> = {
    loading: { kind: "loading", target },
    error: { kind: "error", target },
    empty: { kind: "empty", target },
    available: { kind: "ready", target, summary: available },
    unavailable: { kind: "unavailable", target }
  };

  for (const state of states) {
    const view = presentLocalWorkflowContextBindings(resources[state]);

    assert.equal(view.status.state, state);
    assert.deepEqual(view.scope, [
      { id: "context-id", label: "Context / 上下文", value: target.context_id },
      { id: "commit-id", label: "Commit / 提交", value: target.commit_id }
    ]);
    assert.equal(Object.isFrozen(view), true);
    assert.equal(Object.isFrozen(view.status), true);

    if (state === "available") {
      assert.deepEqual(view.rows, [{
        id: "binding-001",
        bindingId: "binding-001",
        workflowId: "workflow-001",
        workflowRevision: 7,
        nodeCount: 3,
        edgeCount: 2
      }]);
    } else {
      assert.deepEqual(view.rows, []);
    }
  }
});

test("local Workflow binding data rejects a non-deterministic binding order", () => {
  assert.throws(
    () =>
      parseLocalWorkflowContextBindingsV1({
        schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
        context_id: target.context_id,
        commit_id: target.commit_id,
        bindings: [
          {
            binding_id: "binding-002",
            workflow_id: "workflow-002",
            workflow_revision: 2,
            node_count: 1,
            edge_count: 0
          },
          {
            binding_id: "binding-001",
            workflow_id: "workflow-001",
            workflow_revision: 7,
            node_count: 3,
            edge_count: 2
          }
        ]
      }),
    /ordered by workflow_id, workflow_revision, and binding_id/
  );
});
