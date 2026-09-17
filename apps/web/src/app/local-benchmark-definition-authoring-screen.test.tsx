import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createLocalBenchmarkDefinitionAuthoringResource } from "./local-benchmark-definition-authoring-data";
import {
  createLocalBenchmarkDefinitionAuthoringFormView,
  presentLocalBenchmarkDefinitionAuthoring
} from "./local-benchmark-definition-authoring-presenter";
import { LocalBenchmarkDefinitionAuthoringScreen } from "./local-benchmark-definition-authoring-screen";

const target = Object.freeze({
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
});

test("screen renders accessible bilingual semantics for all five authoring states", () => {
  for (const state of ["loading", "error", "empty", "available", "unavailable"] as const) {
    const resource = createLocalBenchmarkDefinitionAuthoringResource(
      state === "available"
        ? { state, target, result: successResult() }
        : { state, target, message: `${state} message / ${state} 消息` }
    );
    const markup = renderToStaticMarkup(
      <LocalBenchmarkDefinitionAuthoringScreen
        form={createLocalBenchmarkDefinitionAuthoringFormView({ enabled: state !== "unavailable", isPending: state === "loading", canSubmit: false })}
        onChange={() => undefined}
        onSubmit={() => undefined}
        view={presentLocalBenchmarkDefinitionAuthoring(resource, state !== "unavailable")}
      />
    );

    assert.match(markup, new RegExp(`data-state=\"${state}\"`));
    assert.match(markup, /Private Benchmark Definition Authoring/);
    assert.match(markup, /私有 Benchmark 定义创作/);
    assert.match(markup, /Local development scope/);
    assert.match(markup, /本地开发范围/);
    if (state === "error") {
      assert.match(markup, /role="alert"/);
      assert.match(markup, /aria-live="assertive"/);
    } else {
      assert.match(markup, /role="status"/);
      assert.match(markup, /aria-live="polite"/);
    }
    if (state === "loading") assert.match(markup, /aria-busy="true"/);
  }
});

test("screen shows exact immutable target and disables every control while pending", () => {
  const view = presentLocalBenchmarkDefinitionAuthoring(
    createLocalBenchmarkDefinitionAuthoringResource({ state: "loading", target }),
    true
  );
  const markup = renderToStaticMarkup(
    <LocalBenchmarkDefinitionAuthoringScreen
      form={createLocalBenchmarkDefinitionAuthoringFormView({ enabled: true, isPending: true, canSubmit: false })}
      onChange={() => undefined}
      onSubmit={() => undefined}
      view={view}
    />
  );

  assert.match(markup, new RegExp(target.contextId));
  assert.match(markup, new RegExp(target.commitId));
  assert.match(markup, /Exact immutable target/);
  assert.match(markup, /aria-busy="true"/);
  assert.equal((markup.match(/disabled=""/g) ?? []).length, 11);
  assert.doesNotMatch(markup, /production.ready|production-safe|production claim/i);
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
