import assert from "node:assert/strict";
import test from "node:test";
import {
  buildLocalBenchmarkDefinitionAuthoringCommand,
  canSubmitLocalBenchmarkDefinition,
  presentLocalBenchmarkDefinitionAuthoring
} from "./local-benchmark-definition-authoring-presenter";
import { createLocalBenchmarkDefinitionAuthoringResource } from "./local-benchmark-definition-authoring-data";

const target = Object.freeze({
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
});

test("presenter builds minimal typed intent bound to the exact selected commit", () => {
  const ids = [
    "11111111-1111-4111-8111-111111111111",
    "22222222-2222-4222-8222-222222222222",
    "33333333-3333-4333-8333-333333333333",
    "44444444-4444-4444-8444-444444444444"
  ];
  const result = buildLocalBenchmarkDefinitionAuthoringCommand(
    target,
    {
      branchName: " main ",
      datasetName: " Safety prompts ",
      caseName: " Refusal ",
      caseInputText: '{"prompt":"unsafe request"}',
      expectedOutputText: "",
      suiteName: " Release gate ",
      metric: "accuracy",
      direction: "minimum",
      thresholdValue: "0.9"
    },
    () => ids.shift()!
  );

  assert.equal(result.ok, true);
  if (!result.ok) return;
  assert.equal(result.value.expected_head_commit_id, target.commitId);
  assert.equal(result.value.datasets[0]?.id, result.value.suite.dataset_ids[0]);
  assert.deepEqual(result.value.datasets[0]?.cases[0]?.expected_output, { mode: "unspecified" });
  assert.deepEqual(result.value.suite.thresholds, [
    { metric: "accuracy", direction: "minimum", value: 0.9 }
  ]);
});

test("presenter rejects incomplete or malformed authoring fields without domain recalculation", () => {
  const result = buildLocalBenchmarkDefinitionAuthoringCommand(target, {
    branchName: "main",
    datasetName: "Dataset",
    caseName: "Case",
    caseInputText: "not-json",
    expectedOutputText: "",
    suiteName: "Suite",
    metric: "accuracy",
    direction: "minimum",
    thresholdValue: "0.9"
  });

  assert.deepEqual(result, {
    ok: false,
    message: "Case input must be valid JSON / 用例输入必须是有效 JSON。"
  });
});

test("presenter exposes bilingual loading error empty available and unavailable states", () => {
  for (const state of ["loading", "error", "empty", "available", "unavailable"] as const) {
    const resource = createLocalBenchmarkDefinitionAuthoringResource(
      state === "available"
        ? { state, target, result: successResult() }
        : { state, target, message: `${state} / ${state} 中文` }
    );
    const view = presentLocalBenchmarkDefinitionAuthoring(resource, state === "unavailable" ? false : true);

    assert.equal(view.state, state);
    assert.match(view.status.stateLabel, / \/ /);
    assert.equal(view.scope[1]?.value, target.contextId);
    assert.equal(view.scope[2]?.value, target.commitId);
    assert.equal(view.result === null, state !== "available");
  }
});

test("presenter keeps mutation default-off and blocks pending or incomplete submission", () => {
  assert.equal(canSubmitLocalBenchmarkDefinition({ enabled: false, bearerToken: "token", hasCommit: true, isValid: true, isPending: false }), false);
  assert.equal(canSubmitLocalBenchmarkDefinition({ enabled: true, bearerToken: "token", hasCommit: true, isValid: true, isPending: true }), false);
  assert.equal(canSubmitLocalBenchmarkDefinition({ enabled: true, bearerToken: "", hasCommit: true, isValid: true, isPending: false }), false);
  assert.equal(canSubmitLocalBenchmarkDefinition({ enabled: true, bearerToken: "token", hasCommit: true, isValid: true, isPending: false }), true);
});

function successResult() {
  return {
    schema_version: "contextlab.local-benchmark-definition-authoring.v1" as const,
    disposition: "created" as const,
    message: { en: "Authored.", zh: "已创作。" },
    binding_id: "11111111-1111-4111-8111-111111111111",
    project_id: target.projectId, context_id: target.contextId, commit_id: target.commitId,
    branch_name: "main", definition_schema_version: 1 as const,
    dataset_ids: ["22222222-2222-4222-8222-222222222222"],
    suite_id: "44444444-4444-4444-8444-444444444444", captured_at: "2026-07-27T08:00:00.000Z"
  };
}
