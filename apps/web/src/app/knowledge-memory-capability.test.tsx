import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  KNOWLEDGE_MEMORY_CAPABILITY_FIXTURES,
  type FrozenKnowledgeMemoryCapabilityFixture
} from "./knowledge-memory-capability-data";
import { presentKnowledgeMemoryCapabilityFixture } from "./knowledge-memory-capability-presenter";
import { KnowledgeMemoryCapabilityInspector } from "./knowledge-memory-capability-screen";

const states = ["loading", "error", "empty", "available", "unavailable"] as const;

test("knowledge and memory capability fixtures are frozen and expose only redacted contract metadata", () => {
  const available = KNOWLEDGE_MEMORY_CAPABILITY_FIXTURES.available;

  assert.equal(Object.isFrozen(KNOWLEDGE_MEMORY_CAPABILITY_FIXTURES), true);
  assert.equal(Object.isFrozen(available), true);
  assert.equal(Object.isFrozen(available.projections), true);
  assert.deepEqual(
    available.projections.map((projection) => projection.id),
    ["knowledge-citation", "memory-retention"]
  );

  for (const projection of available.projections) {
    assert.equal(Object.isFrozen(projection), true);
    assert.equal(Object.isFrozen(projection.capability), true);
    assert.equal(Object.isFrozen(projection.boundary), true);
    assert.deepEqual(Object.keys(projection).sort(), ["boundary", "capability", "id", "schemaVersion"]);
    assert.equal("rawContent" in projection, false);
    assert.equal("citations" in projection, false);
    assert.equal("memoryEvents" in projection, false);
  }
});

test("knowledge and memory capability presenter maps every fixture state into bilingual status models", () => {
  for (const state of states) {
    const fixture = fixtureFor(state);
    const view = presentKnowledgeMemoryCapabilityFixture(fixture);

    assert.equal(Object.isFrozen(fixture), true);
    assert.equal(view.status.state, state);
    assert.match(view.title, new RegExp("Knowledge and memory capabilities / 知识与记忆能力"));
    assert.match(view.description, /fixture/i);

    if (state === "available" || state === "unavailable") {
      assert.deepEqual(
        view.projections.map((projection) => projection.status.id),
        ["knowledge-citation", "memory-retention"]
      );
      assert.match(view.projections[0]!.facts[0]!.value, /knowledge-citation-capability-v1/);
      assert.match(view.projections[1]!.facts[0]!.value, /memory-retention-capability-v1/);
    } else {
      assert.deepEqual(view.projections, []);
    }
  }
});

test("knowledge and memory capability inspector renders all states with accessible redacted status semantics", () => {
  for (const state of states) {
    const markup = renderToStaticMarkup(<KnowledgeMemoryCapabilityInspector fixture={fixtureFor(state)} />);

    assert.match(markup, /aria-labelledby="knowledge-memory-capability-heading"/);
    assert.match(markup, new RegExp("Knowledge and memory capabilities / 知识与记忆能力"));
    assert.match(markup, new RegExp(`data-state="${state}"`));
    assert.match(markup, new RegExp("Capability status / 能力状态:"));

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
      assert.match(markup, new RegExp("Knowledge citation / 知识引用"));
      assert.match(markup, new RegExp("Memory retention / 记忆留存"));
      assert.match(markup, /knowledge-citation-capability-v1/);
      assert.match(markup, /memory-retention-capability-v1/);
      assert.match(markup, new RegExp("Knowledge citation capability metadata / 知识引用能力元数据"));
      assert.match(markup, new RegExp("Memory retention capability metadata / 记忆留存能力元数据"));
    } else {
      assert.doesNotMatch(markup, /knowledge-citation-capability-v1/);
      assert.doesNotMatch(markup, /memory-retention-capability-v1/);
    }

    assert.doesNotMatch(markup, /Private alpha source document/);
    assert.doesNotMatch(markup, /Only in short-lived memory/);
  }
});

function fixtureFor(
  state: (typeof states)[number]
): FrozenKnowledgeMemoryCapabilityFixture {
  return KNOWLEDGE_MEMORY_CAPABILITY_FIXTURES[state];
}
