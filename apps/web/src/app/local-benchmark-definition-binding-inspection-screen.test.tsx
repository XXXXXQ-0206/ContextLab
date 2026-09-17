import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createLocalBenchmarkDefinitionBindingInspectionResource } from "./local-benchmark-definition-binding-inspection-data";
import {
  presentLocalBenchmarkDefinitionBindingInspection
} from "./local-benchmark-definition-binding-inspection-presenter";
import { LocalBenchmarkDefinitionBindingInspectionScreen } from "./local-benchmark-definition-binding-inspection-screen";

const target = Object.freeze({
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
});

test("renders bilingual accessible states and redacted exact binding metadata", () => {
  for (const state of ["loading", "error", "empty", "available", "unavailable"] as const) {
    const resource = createLocalBenchmarkDefinitionBindingInspectionResource(
      state === "available"
        ? { state, target, result: payload() }
        : { state, target, message: `${state} / ${state}` }
    );
    const view = presentLocalBenchmarkDefinitionBindingInspection(resource, state !== "unavailable");
    const markup = renderToStaticMarkup(
      <LocalBenchmarkDefinitionBindingInspectionScreen
        onSelect={() => undefined}
        view={view}
      />
    );
    assert.match(markup, /Authored Benchmark Bindings/);
    assert.match(markup, /已创作 Benchmark 绑定/);
    assert.match(markup, /Exact binding metadata/);
    assert.match(markup, new RegExp(`data-state="${state}"`));
    assert.doesNotMatch(markup, /raw_cases|expected_output|private-input/);
  }
});

function payload() {
  return {
    schema_version: "contextlab.local-benchmark-definition-binding-inspection.v1" as const,
    project_id: target.projectId,
    context_id: target.contextId,
    commit_id: target.commitId,
    bindings: [{
      binding_id: "11111111-1111-4111-8111-111111111111",
      project_id: target.projectId,
      context_id: target.contextId,
      commit_id: target.commitId,
      branch_name: "main",
      definition_schema_version: 1 as const,
      suite_id: "22222222-2222-4222-8222-222222222222",
      suite_name: "Release gate",
      dataset_ids: ["33333333-3333-4333-8333-333333333333"],
      dataset_names: ["Safety prompts"],
      captured_at: "2026-07-27T08:00:00.000Z"
    }]
  };
}
