import assert from "node:assert/strict";
import test from "node:test";
import type { LocalBenchmarkDecision, LocalBenchmarkDecisionRunDetails } from "@contextlab/local-sdk";
import {
  benchmarkEvidenceErrorMessage,
  presentBenchmarkDecision,
  presentBenchmarkDecisionRunDetails
} from "./context-benchmark-evidence-presenter";

test("benchmark evidence presenter renders deterministic redacted metric rows", () => {
  const model = presentBenchmarkDecision({
    project_id: "project-a",
    context_id: "context-a",
    commit_id: "commit-a",
    decision_id: "decision-a",
    suite_id: "suite-a",
    dataset_ids: ["Ä", "a", "Z"],
    definition: {
      suite: {
        id: "suite-a",
        name: "Suite A",
        thresholds: []
      },
      datasets: [
        { id: "Ä", name: "Dataset Ä", case_count: 1 },
        { id: "a", name: "Dataset A", case_count: 1 },
        { id: "Z", name: "Dataset Z", case_count: 1 }
      ]
    },
    run_ids: ["run-b", "run-a"],
    comparability: {
      evaluator_key: "quality",
      evaluator_version: "1.0.0",
      fingerprint: "f".repeat(64)
    },
    evidence_digest: "d".repeat(64),
    status: "passed",
    recorded_at: "2026-07-18T00:00:00Z",
    metrics: [
      {
        metric: "latency_ms",
        threshold_direction: "maximum",
        threshold_value: 320,
        observed: null,
        sample_count: 2,
        required_sample_count: 3,
        has_complete_coverage: false,
        outcome: "insufficient_data"
      },
      {
        metric: "accuracy",
        threshold_direction: "minimum",
        threshold_value: 0.9,
        observed: 0.94,
        sample_count: 3,
        required_sample_count: 3,
        has_complete_coverage: true,
        outcome: "passed"
      }
    ]
  } satisfies LocalBenchmarkDecision);

  assert.equal(model.status.label, "Passed / 通过");
  assert.deepEqual(model.datasets, ["Z", "a", "Ä"]);
  assert.deepEqual(model.runs, ["run-a", "run-b"]);
  assert.deepEqual(
    model.metrics.map((metric) => metric.id),
    ["accuracy", "latency_ms"]
  );
  assert.deepEqual(
    model.metrics.map((metric) => metric.metric),
    ["Accuracy", "Latency Ms"]
  );
  assert.equal(model.metrics[0]?.coverage, "3 / 3 complete / 完整");
  assert.equal(model.metrics[1]?.observed, "Not observed / 未观测");
  assert.equal("cases" in model, false);
  assert.equal("input" in model, false);
  assert.equal("expected_output" in model, false);
});

test("benchmark evidence presenter preserves sealed definition order and presents bilingual metadata", () => {
  const evidence = {
    project_id: "project-a",
    context_id: "context-a",
    commit_id: "commit-a",
    decision_id: "decision-a",
    suite_id: "suite-a",
    dataset_ids: ["dataset-z", "dataset-a"],
    run_ids: [],
    comparability: {
      evaluator_key: "quality",
      evaluator_version: "1.0.0",
      fingerprint: "f".repeat(64)
    },
    evidence_digest: "d".repeat(64),
    status: "passed",
    recorded_at: "2026-07-18T00:00:00Z",
    metrics: [],
    definition: {
      suite: {
        id: "suite-a",
        name: "Release readiness",
        thresholds: [
          { metric: "accuracy", direction: "minimum", value: 0.9 },
          { metric: "latency_ms", direction: "maximum", value: 320 }
        ]
      },
      datasets: [
        { id: "dataset-z", name: "Zebra safety", case_count: 2 },
        { id: "dataset-a", name: "Alpha quality", case_count: 12 }
      ]
    }
  } satisfies LocalBenchmarkDecision;

  const model = presentBenchmarkDecision(evidence);

  assert.deepEqual(model.definition, {
    suite: {
      id: "suite-a",
      name: "Release readiness",
      thresholds: [
        { id: "accuracy", label: "Accuracy minimum / 准确率最低", value: ">= 0.9" },
        { id: "latency_ms", label: "Latency maximum / 延迟最高", value: "<= 320" }
      ]
    },
    datasets: [
      { id: "dataset-z", name: "Zebra safety", caseCount: "2 cases / 2 个用例" },
      { id: "dataset-a", name: "Alpha quality", caseCount: "12 cases / 12 个用例" }
    ]
  });
  assert.equal("cases" in model.definition, false);
  assert.equal("input" in model.definition, false);
  assert.equal("expected_output" in model.definition, false);
  assert.equal("runs" in model.definition, false);
});

test("benchmark evidence presenter renders deterministic bilingual redacted run details", () => {
  const model = presentBenchmarkDecisionRunDetails({
    project_id: "project-a",
    context_id: "context-a",
    commit_id: "commit-a",
    decision_id: "decision-a",
    runs: [
      {
        run_id: "run-Ä",
        model_version: "model-z",
        temperature: 0.7,
        metrics: [{ metric: "latency_ms", value: 320.25 }],
        executed_at: "2026-07-18T00:02:00Z"
      },
      {
        run_id: "run-Z",
        model_version: "model-z",
        temperature: 0.2,
        metrics: [
          { metric: "token_count", value: 1_250.5 },
          { metric: "accuracy", value: 0.95 }
        ],
        executed_at: "2026-07-18T00:01:00Z"
      },
      {
        run_id: "run-a",
        model_version: "model-a",
        temperature: 0,
        metrics: [{ metric: "cost_usd", value: 0.0125 }],
        executed_at: "2026-07-18T00:00:00Z"
      }
    ]
  } satisfies LocalBenchmarkDecisionRunDetails);

  assert.deepEqual(
    model.runs.map((run) => run.id),
    ["run-Ä", "run-Z", "run-a"]
  );
  assert.deepEqual(model.runs[0], {
    id: "run-Ä",
    facts: [
      { id: "run-id", label: "Run / 运行", value: "run-Ä" },
      { id: "model-version", label: "Model version / 模型版本", value: "model-z" },
      { id: "temperature", label: "Temperature / 温度", value: "0.7" },
      { id: "executed-at", label: "Executed / 执行时间", value: "2026-07-18 00:02:00 UTC" }
    ],
    metrics: [{ id: "latency_ms", label: "Latency / 延迟", value: "320.25" }]
  });
  assert.equal("project_id" in model, false);
  assert.equal("context_id" in model, false);
  assert.equal("input" in model.runs[0]!, false);
  assert.equal("output" in model.runs[0]!, false);
  assert.equal("cases" in model.runs[0]!, false);
  assert.equal("measurements" in model.runs[0]!, false);
  assert.equal("outcome" in model.runs[0]!.metrics[0]!, false);
});

test("benchmark evidence presenter gives bilingual remediation for protected errors", () => {
  assert.match(benchmarkEvidenceErrorMessage(401), /Bearer authentication/);
  assert.match(benchmarkEvidenceErrorMessage(403), /permission/);
  assert.match(benchmarkEvidenceErrorMessage(503), /unavailable/);
  assert.match(benchmarkEvidenceErrorMessage(429, 5_000), /5 seconds/);
});
