import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import type { FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState } from "./capability-state-presenter";
import { CapabilityStateAdapter } from "./capability-state-screen";
import {
  LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1,
  parseLocalCapabilityAvailability
} from "./local-capability-availability-data";
import { presentLocalCapabilityAvailability } from "./local-capability-availability-presenter";
import { LocalCapabilityAvailabilityAdapter } from "./local-capability-availability-screen";

const states = ["loading", "error", "empty", "available", "unavailable"] as const;

test("capability-state presenter accepts a frozen bilingual DTO for every supported state", () => {
  for (const state of states) {
    const dto = createFrozenCapabilityStateDto(state);
    const view = presentCapabilityState(dto);

    assert.equal(Object.isFrozen(dto), true);
    assert.equal(Object.isFrozen(dto.capability), true);
    assert.equal(view.state, state);
    assert.match(view.capabilityLabel, /Context graph \/ Context 图谱/);
    assert.match(view.stateLabel, /\//);
    assert.match(view.description, /\//);
  }
});

test("capability-state adapter renders bilingual accessible status semantics for every state", () => {
  for (const state of states) {
    const markup = renderToStaticMarkup(<CapabilityStateAdapter dto={createFrozenCapabilityStateDto(state)} />);

    assert.match(markup, new RegExp(`data-state="${state}"`));
    assert.match(markup, /Context graph \/ Context 图谱/);
    assert.match(markup, /Capability status \/ 能力状态/);
    assert.match(markup, /min-width:0/);

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
  }
});

test("local availability adapter accepts only the V1 unavailable contract and reuses shared status presentation", () => {
  const dto = parseLocalCapabilityAvailability({
    schema_version: LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1,
    operation_id: "evaluation-run",
    integration: "contextlab-evaluation",
    availability: "unavailable",
    reason: "shared_integration_not_registered"
  });
  const view = presentLocalCapabilityAvailability(dto);
  const markup = renderToStaticMarkup(<LocalCapabilityAvailabilityAdapter dto={dto} />);

  assert.equal(Object.isFrozen(dto), true);
  assert.equal(view.state, "unavailable");
  assert.match(view.capabilityLabel, /Evaluation engine \/ 评测引擎/);
  assert.match(markup, /data-state="unavailable"/);
  assert.match(markup, /Capability status \/ 能力状态/);
  assert.throws(
    () =>
      parseLocalCapabilityAvailability({
        ...dto,
        schema_version: "contextlab.local-capability-availability.v2"
      }),
    /schema_version/
  );
});

function createFrozenCapabilityStateDto(
  state: FrozenCapabilityStateDto["state"]
): FrozenCapabilityStateDto {
  return Object.freeze({
    id: "context-graph",
    capability: Object.freeze({ en: "Context graph", zh: "Context 图谱" }),
    state,
    summary: Object.freeze({
      en: "Committed graph information is displayed here.",
      zh: "已提交的图谱信息将在此处显示。"
    }),
    detail: Object.freeze({ en: "Snapshot revision: r42", zh: "快照版本：r42" })
  });
}
