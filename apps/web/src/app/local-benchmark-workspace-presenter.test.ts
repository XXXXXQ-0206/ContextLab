import assert from "node:assert/strict";
import test from "node:test";
import { parseLocalBenchmarkWorkspace } from "@contextlab/local-sdk";
import {
  createLocalBenchmarkWorkspaceResource,
  type LocalBenchmarkWorkspaceTarget
} from "./local-benchmark-workspace-data";
import { presentLocalBenchmarkWorkspace } from "./local-benchmark-workspace-presenter";

const target: LocalBenchmarkWorkspaceTarget = Object.freeze({
  projectId: "project-a",
  contextId: "context-a",
  commitId: "commit-revised",
  cohortId: "cohort-revised",
  baseline: Object.freeze({ commitId: "commit-baseline", cohortId: "cohort-baseline" })
});

test("presents every strict inspector state as a frozen bilingual status model", () => {
  const states = ["loading", "error", "empty", "available", "unavailable"] as const;
  const workspace = parseLocalBenchmarkWorkspace(benchmarkWorkspacePayload());

  for (const state of states) {
    const resource = createLocalBenchmarkWorkspaceResource(
      state === "available"
        ? { state, target, workspace }
        : {
            state,
            target,
            message: `${state} detail / ${state} 详情`
          }
    );
    const view = presentLocalBenchmarkWorkspace(resource);

    assert.equal(view.state, state);
    assert.match(view.title, /Local Benchmark Workspace \/ 本地 Benchmark 工作台/);
    assert.match(view.status.stateLabel, /\//);
    assert.match(view.status.description, /\//);
    assert.equal(Object.isFrozen(resource), true);
    assert.equal(Object.isFrozen(resource.target), true);
    assert.equal(Object.isFrozen(view), true);
    assert.equal(Object.isFrozen(view.status), true);
    assert.equal(Object.isFrozen(view.scope), true);

    if (state === "available") {
      assert.notEqual(view.projection, null);
    } else {
      assert.equal(view.projection, null);
    }
  }
});

test("preserves server-owned projection order and values without calculating benchmark conclusions", () => {
  const workspace = parseLocalBenchmarkWorkspace(benchmarkWorkspacePayload());
  const view = presentLocalBenchmarkWorkspace(
    createLocalBenchmarkWorkspaceResource({ state: "available", target, workspace })
  );

  assert.deepEqual(
    view.scope.map((fact) => fact.id),
    [
      "project",
      "context",
      "revised-commit",
      "revised-decision",
      "baseline-commit",
      "baseline-decision"
    ]
  );
  assert.equal(view.projection?.suite.name, "Release gate");
  assert.deepEqual(
    view.projection?.datasets.rows.map((row) => row.id),
    ["dataset-a", "dataset-b"]
  );
  assert.deepEqual(
    view.projection?.runs.rows.map((row) => row.id),
    ["dataset-a\u0000case-a", "dataset-b\u0000case-b"]
  );
  assert.deepEqual(
    view.projection?.scorecard.rows.map((row) => row.id),
    ["latency_ms", "accuracy"]
  );
  assert.deepEqual(view.projection?.scorecard.rows[0]?.cells, [
    "Latency (ms) / 延迟（毫秒）",
    "Maximum / 最大值",
    "800",
    "720",
    "2 / 2",
    "Complete / 完整",
    "Passed / 通过"
  ]);
  assert.equal(view.projection?.regressionStatus.label, "Passed / 通过");
  assert.equal(view.projection?.evaluationDiff?.statusChange, "No status change / 状态未变化");
  assert.deepEqual(
    view.projection?.evaluationDiff?.rows.map((row) => row.id),
    ["accuracy:modified"]
  );
  assert.match(view.projection?.evaluationDiff?.rows[0]?.cells[2] ?? "", /0\.91/);
  assert.match(view.projection?.evaluationDiff?.rows[0]?.cells[3] ?? "", /0\.94/);
  assert.equal(Object.isFrozen(view.projection), true);
  assert.equal(Object.isFrozen(view.projection?.scorecard.rows[0]?.cells), true);
  assert.doesNotMatch(JSON.stringify(view), /raw_output|expected_output|input_payload|cookie/i);
});

test("presents server-owned decision pair witness facts instead of cohort-derived decision identity", () => {
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

  assert.deepEqual(
    view.scope.map((fact) => [fact.id, fact.value]),
    [
      ["project", "project-a"],
      ["context", "context-a"],
      ["revised-commit", "commit-revised"],
      ["revised-decision", "decision-revised"],
      ["baseline-commit", "commit-baseline"],
      ["baseline-decision", "decision-baseline"]
    ]
  );
  assert.doesNotMatch(JSON.stringify(view.scope), /cohort-derived|cohort-revised|cohort-baseline/);
  assert.doesNotMatch(JSON.stringify(view), /raw_output|expected_output|input_payload|cookie/i);
});

test("keeps an available zero-row server projection honest and distinct from unavailable", () => {
  const payload = benchmarkWorkspacePayload();
  payload.projection.datasets = [];
  payload.projection.runs = [];
  payload.projection.scorecard.run_count = 0;
  payload.projection.scorecard.metrics = [];
  payload.projection.evaluation_diff.metric_changes = [];
  const workspace = parseLocalBenchmarkWorkspace(payload);

  const available = presentLocalBenchmarkWorkspace(
    createLocalBenchmarkWorkspaceResource({ state: "available", target, workspace })
  );
  const unavailable = presentLocalBenchmarkWorkspace(
    createLocalBenchmarkWorkspaceResource({
      state: "unavailable",
      target,
      message: "No materialized commit is available / 没有可用的已物化提交"
    })
  );

  assert.equal(available.state, "available");
  assert.equal(available.projection?.scorecard.rows.length, 0);
  assert.match(available.projection?.scorecard.emptyMessage ?? "", /No scorecard metrics/);
  assert.equal(unavailable.state, "unavailable");
  assert.equal(unavailable.projection, null);
});

test("resource creation rejects an available state whose response scope does not match its target", () => {
  const workspace = parseLocalBenchmarkWorkspace(benchmarkWorkspacePayload());
  const wrongTarget: LocalBenchmarkWorkspaceTarget = {
    ...target,
    cohortId: "other-cohort"
  };

  assert.throws(
    () => createLocalBenchmarkWorkspaceResource({ state: "available", target: wrongTarget, workspace }),
    /requested scope/
  );
});

function benchmarkWorkspacePayload() {
  return {
    schema_version: "contextlab.local-benchmark-workspace.v1" as const,
    project_id: "project-a",
    context_id: "context-a",
    revised: { commit_id: "commit-revised", cohort_id: "cohort-revised" },
    baseline: { commit_id: "commit-baseline", cohort_id: "cohort-baseline" },
    projection: {
      schema_version: 1 as const,
      receipt: { cohort_id: "cohort-revised" },
      suite: { id: "suite-release", name: "Release gate" },
      datasets: [
        { id: "dataset-a", name: "Core", case_count: 1 },
        { id: "dataset-b", name: "Safety", case_count: 1 }
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
      regression_status: "passed" as const,
      evaluation_diff: {
        baseline_cohort_id: "cohort-baseline",
        revised_cohort_id: "cohort-revised",
        status_change: null,
        metric_changes: [
          {
            metric: "accuracy" as const,
            change_kind: "modified" as const,
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
