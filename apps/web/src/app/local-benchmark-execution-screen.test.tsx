import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { createLocalBenchmarkExecutionResource } from "./local-benchmark-execution-data";
import { presentLocalBenchmarkExecution } from "./local-benchmark-execution-presenter";
import { LocalBenchmarkExecutionScreen } from "./local-benchmark-execution-screen";

const target = Object.freeze({
  projectId: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  contextId: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  commitId: "cccccccc-cccc-4ccc-8ccc-cccccccccccc"
});
const receipt = {
  schema_version: "contextlab.local-benchmark-execution.v1" as const,
  disposition: "replayed" as const,
  projection_disposition: "replayed" as const,
  project_id: target.projectId,
  context_id: target.contextId,
  commit_id: target.commitId,
  binding_id: "11111111-1111-4111-8111-111111111111",
  decision_id: "22222222-2222-4222-8222-222222222222",
  suite_id: "33333333-3333-4333-8333-333333333333",
  dataset_ids: ["44444444-4444-4444-8444-444444444444"],
  cohort_id: "55555555-5555-4555-8555-555555555555"
};

test("screen renders accessible bilingual execution states and redacted receipt metadata", () => {
  for (const state of ["loading", "error", "empty", "created", "replayed", "conflict", "unavailable"] as const) {
    const resource = state === "created" || state === "replayed"
      ? createLocalBenchmarkExecutionResource({ state, target, receipt: { ...receipt, disposition: state } })
      : createLocalBenchmarkExecutionResource({ state, target, message: "stable message / 稳定消息" });
    const markup = renderToStaticMarkup(
      <LocalBenchmarkExecutionScreen view={presentLocalBenchmarkExecution(resource)} />
    );
    assert.match(markup, /Benchmark execution/);
    assert.match(markup, /Benchmark 执行/);
    assert.match(markup, /aria-live="(polite|assertive)"/);
    assert.match(markup, /data-state="(loading|error|empty|available|unavailable)"/);
    assert.doesNotMatch(markup, /raw|input|output|trace|secret/i);
  }
});
