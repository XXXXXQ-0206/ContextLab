import assert from "node:assert/strict";
import test from "node:test";
import type { LocalBenchmarkDecisionDiff } from "@contextlab/local-sdk";
import {
  benchmarkDecisionDiffErrorMessage,
  presentBenchmarkDecisionDiff
} from "./context-benchmark-decision-diff-presenter";

test("benchmark decision diff presenter renders bilingual status and stable metric changes", () => {
  const model = presentBenchmarkDecisionDiff({
    project_id: "project-a",
    context_id: "context-a",
    baseline: { commit_id: "commit-base", decision_id: "decision-base" },
    revised: { commit_id: "commit-revised", decision_id: "decision-revised" },
    status_change: { baseline: "passed", revised: "regressed" },
    metric_changes: [
      {
        kind: "modified",
        metric: "accuracy",
        baseline: metric("accuracy", 0.94, "passed"),
        revised: metric("accuracy", 0.84, "regressed")
      }
    ]
  } satisfies LocalBenchmarkDecisionDiff);

  assert.equal(model.status.label, "Passed / 通过 -> Regressed / 回归");
  assert.deepEqual(model.scopes.map((fact) => fact.id), ["baseline-commit", "baseline-decision", "revised-commit", "revised-decision"]);
  assert.equal(model.metrics[0]?.change, "Modified / 已修改");
  assert.equal(model.metrics[0]?.revised, "0.84");
  assert.match(benchmarkDecisionDiffErrorMessage(409), /cannot be compared/);
});

function metric(metric: "accuracy", observed: number, outcome: "passed" | "regressed") {
  return {
    metric,
    threshold_direction: "minimum" as const,
    threshold_value: 0.9,
    observed,
    sample_count: 1,
    required_sample_count: 1,
    has_complete_coverage: true,
    outcome
  };
}
