import assert from "node:assert/strict";
import test from "node:test";
import {
  presentBenchmarkDecisionDiffSelection,
  selectionStatusTone
} from "./context-benchmark-decision-diff-selection-presenter";

test("diff selection presenter exposes first decision per exact commit and clears stale options", () => {
  const view = presentBenchmarkDecisionDiffSelection({
    kind: "ready",
    target: { baselineCommitId: "baseline/commit", revisedCommitId: "revised/commit" },
    selections: {
      baseline: discovery("baseline/commit", "baseline/decision"),
      revised: discovery("revised/commit", "revised/decision")
    }
  });

  assert.equal(view.state, "ready");
  assert.equal(view.baseline.firstDecisionId, "baseline/decision");
  assert.equal(view.revised.firstDecisionId, "revised/decision");
  assert.equal(view.baseline.options[0]?.value, "baseline/decision");
  assert.equal(view.revised.options[0]?.value, "revised/decision");

  const empty = presentBenchmarkDecisionDiffSelection({
    kind: "empty",
    target: { baselineCommitId: "new-base", revisedCommitId: "new-revised" }
  });
  assert.equal(empty.baseline.firstDecisionId, null);
  assert.equal(empty.revised.firstDecisionId, null);
  assert.equal(empty.baseline.options.length, 0);
  assert.equal(empty.revised.options.length, 0);
});

test("diff selection presenter maps state tones and bilingual messages", () => {
  const error = presentBenchmarkDecisionDiffSelection({
    kind: "error",
    target: { baselineCommitId: "base", revisedCommitId: "revised" },
    message: "Discovery failed / 发现失败。"
  });
  assert.equal(selectionStatusTone(error.state), "warning");
  assert.match(error.message, /Discovery failed/);
  assert.equal(selectionStatusTone("loading"), "info");
  assert.equal(selectionStatusTone("empty"), "neutral");
  assert.equal(selectionStatusTone("ready"), "success");
});

function discovery(commitId: string, decisionId: string) {
  return {
    schema_version: "contextlab.local-benchmark-decision-discovery.v1" as const,
    project_id: "project",
    context_id: "context",
    commit_id: commitId,
    decisions: [{
      decision_id: decisionId,
      suite_id: "suite/quality",
      status: "passed" as const,
      recorded_at: "2026-07-22T00:01:00Z"
    }]
  };
}
