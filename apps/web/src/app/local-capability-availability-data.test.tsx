import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  adaptLocalCapabilityAvailabilityV1,
  parseLocalCapabilityAvailability,
  parseLocalCapabilityAvailabilityV1,
  type LocalCapabilityAvailabilityResource,
  type LocalCapabilityAvailabilityV1
} from "./local-capability-availability-data";
import { presentCapabilityState } from "./capability-state-presenter";
import { CapabilityStateAdapter } from "./capability-state-screen";
import { LocalCapabilityAvailabilityAdapter } from "./local-capability-availability-screen";

const target = Object.freeze({
  capability_id: "workflow.execution",
  capability: Object.freeze({ en: "Workflow execution", zh: "工作流执行" })
});

test("local capability-availability V1 data adapts every resource state through the presenter and accessible screen", () => {
  const available = parseLocalCapabilityAvailabilityV1({
    schema_version: 1,
    capability_id: target.capability_id,
    capability: target.capability,
    availability: "available",
    summary: { en: "The workflow runtime is registered.", zh: "工作流运行时已注册。" },
    detail: { en: "Version 1.0.0", zh: "版本 1.0.0" }
  });
  const unavailable = parseLocalCapabilityAvailabilityV1({
    schema_version: 1,
    capability_id: target.capability_id,
    capability: target.capability,
    availability: "unavailable"
  });
  const cases: ReadonlyArray<Readonly<{ expectedState: "loading" | "error" | "empty" | "available" | "unavailable"; resource: LocalCapabilityAvailabilityResource }>> = [
    { expectedState: "loading", resource: { kind: "loading", target } },
    { expectedState: "error", resource: { kind: "error", target } },
    { expectedState: "empty", resource: { kind: "empty", target } },
    { expectedState: "available", resource: { kind: "ready", availability: available } },
    { expectedState: "unavailable", resource: { kind: "ready", availability: unavailable } }
  ];

  for (const { expectedState, resource } of cases) {
    const dto = adaptLocalCapabilityAvailabilityV1(resource);
    const view = presentCapabilityState(dto);
    const markup = renderToStaticMarkup(<CapabilityStateAdapter dto={dto} />);

    assert.equal(Object.isFrozen(dto), true);
    assert.equal(dto.id, target.capability_id);
    assert.equal(dto.state, expectedState);
    assert.equal(view.state, expectedState);
    assert.match(markup, new RegExp(`data-state="${expectedState}"`));
    assert.match(markup, /Workflow execution \/ 工作流执行/);

    if (expectedState === "error") {
      assert.match(markup, /role="alert"/);
      assert.match(markup, /aria-live="assertive"/);
    } else {
      assert.match(markup, /role="status"/);
      assert.match(markup, /aria-live="polite"/);
    }

    if (expectedState === "loading") {
      assert.match(markup, /aria-busy="true"/);
    } else {
      assert.doesNotMatch(markup, /aria-busy="true"/);
    }
  }
});

test("local capability-availability adapter presents the server-owned V1 availability", () => {
  const availability: LocalCapabilityAvailabilityV1 = parseLocalCapabilityAvailabilityV1({
    schema_version: 1,
    capability_id: target.capability_id,
    capability: target.capability,
    availability: "available",
    summary: { en: "The workflow runtime is registered.", zh: "工作流运行时已注册。" }
  });

  const markup = renderToStaticMarkup(<LocalCapabilityAvailabilityAdapter dto={availability} />);

  assert.match(markup, /data-state="available"/);
  assert.match(markup, /The workflow runtime is registered\. \/ 工作流运行时已注册。/);
});

test("local capability-availability parser rejects a non-V1 JSON payload", () => {
  assert.throws(
    () =>
      parseLocalCapabilityAvailabilityV1({
        schema_version: 2,
        capability_id: target.capability_id,
        capability: target.capability,
        availability: "available"
      }),
    /schema_version/
  );
});

test("local capability-availability V1 parser rejects unknown fields", () => {
  assert.throws(
    () =>
      parseLocalCapabilityAvailabilityV1({
        schema_version: 1,
        capability_id: target.capability_id,
        capability: target.capability,
        availability: "available",
        unexpected: true
      }),
    /unexpected shape/
  );
});

test("local unavailable availability parser rejects raw upstream fields", () => {
  assert.throws(
    () =>
      parseLocalCapabilityAvailability({
        schema_version: "contextlab.local-capability-availability.v1",
        operation_id: "evaluation-run",
        integration: "contextlab-evaluation",
        availability: "unavailable",
        reason: "shared_integration_not_registered",
        raw_provider_payload: "must not cross the Web boundary"
      }),
    /unexpected shape/
  );
});
