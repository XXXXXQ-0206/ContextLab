import assert from "node:assert/strict";
import test from "node:test";
import { parseLocalBenchmarkWorkspace } from "@contextlab/local-sdk";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  createLocalBenchmarkWorkspaceResource,
  type LocalBenchmarkWorkspaceTarget
} from "./local-benchmark-workspace-data";
import { presentLocalBenchmarkWorkspace } from "./local-benchmark-workspace-presenter";
import { LocalBenchmarkWorkspaceScreen } from "./local-benchmark-workspace-screen";

const target: LocalBenchmarkWorkspaceTarget = Object.freeze({
  projectId: "project-a",
  contextId: "context-a",
  commitId: "commit-revised",
  cohortId: "cohort-revised",
  baseline: Object.freeze({ commitId: "commit-baseline", cohortId: "cohort-baseline" })
});

test("renders accessible bilingual semantics for all five workspace states", () => {
  const states = ["loading", "error", "empty", "available", "unavailable"] as const;
  const workspace = parseLocalBenchmarkWorkspace(benchmarkWorkspacePayload());

  for (const state of states) {
    const resource = createLocalBenchmarkWorkspaceResource(
      state === "available"
        ? { state, target, workspace }
        : { state, target, message: `${state} message / ${state} 消息` }
    );
    const markup = renderToStaticMarkup(
      <LocalBenchmarkWorkspaceScreen view={presentLocalBenchmarkWorkspace(resource)} />
    );

    assert.match(markup, new RegExp(`data-state="${state}"`));
    assert.match(markup, /Local Benchmark Workspace/);
    assert.match(markup, /本地 Benchmark 工作台/);
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

test("renders only the server-owned projection in deterministic table order", () => {
  const workspace = parseLocalBenchmarkWorkspace(benchmarkWorkspacePayload());
  const view = presentLocalBenchmarkWorkspace(
    createLocalBenchmarkWorkspaceResource({ state: "available", target, workspace })
  );
  const markup = renderToStaticMarkup(<LocalBenchmarkWorkspaceScreen view={view} />);

  assert.match(markup, /project-a/);
  assert.match(markup, /commit-revised/);
  assert.match(markup, /cohort-baseline/);
  assert.match(markup, /Release gate/);
  assert.match(markup, /Datasets \/ 数据集/);
  assert.match(markup, /Run provenance \/ 运行来源/);
  assert.match(markup, /Server scorecard \/ 服务端记分卡/);
  assert.match(markup, /Regression status \/ 回归状态/);
  assert.match(markup, /Evaluation diff \/ 评测差异/);
  assert.ok(markup.indexOf("Core dataset") < markup.indexOf("Safety dataset"));
  assert.ok(markup.indexOf("Latency (ms)") < markup.indexOf("Accuracy"));
  assert.match(markup, /720/);
  assert.match(markup, /0\.94/);
  assert.match(markup, /No status change \/ 状态未变化/);
  assert.doesNotMatch(markup, /raw case|raw output|expected_output|input_payload|cookie/i);
});

test("renders read-only decision pair witness facts through the shared scope grid", () => {
  const workspace = {
    ...parseLocalBenchmarkWorkspace(benchmarkWorkspacePayload()),
    decision_pair_witness: {
      schema_version: 1 as const,
      project_id: "project-a",
      context_id: "context-a",
      baseline: { commit_id: "commit-baseline", decision_id: "decision-baseline" },
      revised: { commit_id: "commit-revised", decision_id: "decision-revised" }
    }
  };
  const view = presentLocalBenchmarkWorkspace(
    createLocalBenchmarkWorkspaceResource({
      state: "available",
      target: {
        ...target,
        decisionId: "decision-revised",
        cohortId: "cohort-derived",
        baseline: { commitId: "commit-baseline", decisionId: "decision-baseline" }
      },
      workspace
    })
  );
  const markup = renderToStaticMarkup(<LocalBenchmarkWorkspaceScreen view={view} />);

  assert.match(markup, /Revised decision witness/);
  assert.match(markup, /decision-revised/);
  assert.match(markup, /Baseline decision witness/);
  assert.match(markup, /decision-baseline/);
  assert.doesNotMatch(markup, /cohort-derived/);
  assert.doesNotMatch(markup, /decision_pair_witness|raw_output|expected_output|input_payload|cookie/i);
});

function benchmarkWorkspacePayload() {
  return {
    schema_version: "contextlab.local-benchmark-workspace.v1",
    project_id: "project-a",
    context_id: "context-a",
    revised: { commit_id: "commit-revised", cohort_id: "cohort-revised" },
    baseline: { commit_id: "commit-baseline", cohort_id: "cohort-baseline" },
    projection: {
      schema_version: 1,
      receipt: { cohort_id: "cohort-revised" },
      suite: { id: "suite-release", name: "Release gate" },
      datasets: [
        { id: "dataset-a", name: "Core dataset", case_count: 1 },
        { id: "dataset-b", name: "Safety dataset", case_count: 1 }
      ],
      runs: [
        { dataset_id: "dataset-a", case_id: "case-a", metric_count: 2 },
        { dataset_id: "dataset-b", case_id: "case-b", metric_count: 2 }
      ],
      scorecard: {
        run_count: 2,
        metrics: [
          metric("latency_ms", "maximum", 800, 720, "passed"),
          metric("accuracy", "minimum", 0.9, 0.94, "passed")
        ]
      },
      regression_status: "passed",
      evaluation_diff: {
        baseline_cohort_id: "cohort-baseline",
        revised_cohort_id: "cohort-revised",
        status_change: null,
        metric_changes: [
          {
            metric: "accuracy",
            change_kind: "modified",
            baseline: metric("accuracy", "minimum", 0.9, 0.91, "passed"),
            revised: metric("accuracy", "minimum", 0.9, 0.94, "passed")
          }
        ]
      }
    }
  };
}

function metric(
  name: "latency_ms" | "accuracy",
  direction: "minimum" | "maximum",
  threshold: number,
  observed: number,
  outcome: "passed" | "regressed"
) {
  return {
    metric: name,
    threshold_direction: direction,
    threshold_value: threshold,
    observed,
    sample_count: 2,
    required_sample_count: 2,
    has_complete_coverage: true,
    outcome
  };
}
