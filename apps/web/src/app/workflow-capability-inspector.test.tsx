import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
  WORKFLOW_CAPABILITY_RESOURCE_FIXTURES,
  type FrozenWorkflowCapabilityResourceFixture
} from "./workflow-capability-data";
import { presentWorkflowCapabilityResourceFixture } from "./workflow-capability-presenter";
import { WorkflowCapabilityInspector } from "./workflow-capability-screen";

const states = ["loading", "error", "empty", "available", "unavailable"] as const;

test("workflow capability inspector fixtures are frozen V1 resources with deterministic redacted metadata", () => {
  const available = WORKFLOW_CAPABILITY_RESOURCE_FIXTURES.available;

  assert.equal(Object.isFrozen(WORKFLOW_CAPABILITY_RESOURCE_FIXTURES), true);
  assert.equal(Object.isFrozen(available), true);
  assert.equal(available.schemaVersion, WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1);
  assert.equal(Object.isFrozen(available.resources), true);
  assert.deepEqual(
    available.resources.map((resource) => resource.id),
    ["workflow-definition", "workflow-execution"]
  );

  for (const resource of available.resources) {
    assert.equal(Object.isFrozen(resource), true);
    assert.equal(Object.isFrozen(resource.capability), true);
    assert.equal(Object.isFrozen(resource.boundary), true);
    assert.equal(resource.schemaVersion, WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1);
    assert.deepEqual(Object.keys(resource).sort(), ["boundary", "capability", "id", "schemaVersion"]);
    assert.equal("credentials" in resource, false);
    assert.equal("definition" in resource, false);
    assert.equal("runInput" in resource, false);
    assert.equal("runOutput" in resource, false);
  }
});

test("workflow capability presenter maps every frozen fixture state into bilingual status models", () => {
  for (const state of states) {
    const view = presentWorkflowCapabilityResourceFixture(fixtureFor(state));

    assert.equal(view.status.state, state);
    assert.match(view.title, /Workflow capabilities \/ 工作流能力/);
    assert.match(view.description, /redacted/i);

    if (state === "available" || state === "unavailable") {
      assert.deepEqual(
        view.resources.map((resource) => resource.status.id),
        ["workflow-definition", "workflow-execution"]
      );
      assert.match(view.resources[0]!.facts[0]!.value, new RegExp(WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1));
      assert.match(view.resources[1]!.facts[0]!.value, new RegExp(WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1));
    } else {
      assert.deepEqual(view.resources, []);
    }
  }
});

test("workflow capability inspector renders every state through the shared accessible capability status screen", () => {
  for (const state of states) {
    const markup = renderToStaticMarkup(<WorkflowCapabilityInspector fixture={fixtureFor(state)} />);

    assert.match(markup, /aria-labelledby="workflow-capability-heading"/);
    assert.match(markup, /Workflow capabilities \/ 工作流能力/);
    assert.match(markup, new RegExp(`data-state="${state}"`));
    assert.match(markup, /Capability status \/ 能力状态:/);

    if (state === "error") {
      assert.match(markup, /role="alert"/);
      assert.match(markup, /aria-live="assertive"/);
    } else {
      assert.match(markup, /role="status"/);
      assert.match(markup, /aria-live="polite"/);
    }

    if (state === "loading") {
      assert.match(markup, /aria-busy="true"/);
    } else {
      assert.doesNotMatch(markup, /aria-busy="true"/);
    }

    if (state === "available" || state === "unavailable") {
      assert.match(markup, /Workflow definition \/ 工作流定义/);
      assert.match(markup, /Workflow execution \/ 工作流执行/);
      assert.match(markup, new RegExp(WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1));
    } else {
      assert.doesNotMatch(markup, new RegExp(WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1));
    }

    assert.doesNotMatch(markup, /Production credential/);
    assert.doesNotMatch(markup, /Private workflow source/);
    assert.doesNotMatch(markup, /Sensitive run payload/);
  }
});

function fixtureFor(
  state: (typeof states)[number]
): FrozenWorkflowCapabilityResourceFixture {
  return WORKFLOW_CAPABILITY_RESOURCE_FIXTURES[state];
}
