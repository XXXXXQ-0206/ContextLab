import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  BenchmarkDefinitionMetadata,
  BenchmarkDecisionRunDetails,
  ContextBenchmarkEvidenceInspectorControl,
  canEditBenchmarkEvidenceScope,
  presentBenchmarkEvidenceError
} from "./context-benchmark-evidence-inspector";
import { LocalBenchmarkEvidenceProxyError } from "./context-benchmark-evidence-data";

test("benchmark evidence inspector renders a bilingual local-only read form", () => {
  const markup = renderToStaticMarkup(
    <ContextBenchmarkEvidenceInspectorControl
      candidates={[{ id: "commit-a", label: "Materialized head" }]}
      contextId="context-a"
      projectId="project-a"
    />
  );

  assert.match(markup, /Benchmark Evidence Inspection/);
  assert.match(markup, /只读本地受保护审阅/);
  assert.match(markup, /Bearer token/);
  assert.match(markup, /type="password"/);
  assert.match(markup, /Decision ID/);
  assert.match(markup, /No decision selected/);
  assert.match(markup, /disabled=""/);
  assert.match(markup, /No credential is persisted/);
});

test("benchmark evidence inspector locks scope controls while a read is pending", () => {
  assert.equal(canEditBenchmarkEvidenceScope(false), true);
  assert.equal(canEditBenchmarkEvidenceScope(true), false);
});

test("benchmark evidence inspector does not append upstream diagnostics to notices", () => {
  const error = new LocalBenchmarkEvidenceProxyError(503, {
    error: "benchmark_proxy_error",
    message: "private upstream SQL detail"
  });

  const message = presentBenchmarkEvidenceError(error);
  assert.doesNotMatch(message, /private upstream SQL detail/);
  assert.match(message, /Benchmark evidence inspection is unavailable/);
  assert.match(message, /Benchmark 证据审阅暂不可用/);
});

test("benchmark evidence inspector renders sealed bilingual definition metadata without raw payloads", () => {
  const markup = renderToStaticMarkup(
    <BenchmarkDefinitionMetadata
      definition={{
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
      }}
    />
  );

  assert.match(markup, /Sealed definition/);
  assert.match(markup, /已封存定义/);
  assert.match(markup, /Suite ID/);
  assert.match(markup, /Thresholds/);
  assert.match(markup, /Accuracy minimum/);
  assert.match(markup, /Dataset name/);
  assert.match(markup, /Case count/);
  assert.ok(markup.indexOf("Zebra safety") < markup.indexOf("Alpha quality"));
  assert.doesNotMatch(markup, /input|expected output|raw run|policy detail|case payload/i);
});

test("benchmark evidence inspector renders safe sealed run details in response order", () => {
  const markup = renderToStaticMarkup(
    <BenchmarkDecisionRunDetails
      details={{
        project_id: "project-a",
        context_id: "context-a",
        commit_id: "commit-a",
        decision_id: "decision-a",
        runs: [
          {
            run_id: "run-b",
            model_version: "model-v2",
            temperature: 0.25,
            metrics: [
              { metric: "accuracy", value: 0.95 },
              { metric: "latency_ms", value: 120 }
            ],
            executed_at: "2026-07-18T09:00:00Z"
          },
          {
            run_id: "run-a",
            model_version: "model-v2",
            temperature: 0.25,
            metrics: [{ metric: "accuracy", value: 0.91 }],
            executed_at: "2026-07-18T09:01:00Z"
          }
        ]
      }}
      status="ready"
    />
  );

  assert.match(markup, /Sealed run details/);
  assert.match(markup, /已封存运行详情/);
  assert.match(markup, /Run \/ 运行/);
  assert.match(markup, /Model/);
  assert.match(markup, /模型/);
  assert.match(markup, /Metric \/ 指标/);
  assert.ok(markup.indexOf("run-b") < markup.indexOf("run-a"));
  assert.match(markup, /Accuracy \/ 准确率/);
  assert.match(markup, /Latency \/ 延迟/);
  assert.match(markup, /0.95/);
  assert.match(markup, /120/);
  assert.doesNotMatch(markup, /accuracy: 0.95|latency_ms: 120/);
  assert.doesNotMatch(markup, /input|expected output|case payload|evaluator request|tool call/i);
});

test("benchmark evidence inspector renders bilingual run-detail loading, error, and empty states", () => {
  const loading = renderToStaticMarkup(<BenchmarkDecisionRunDetails details={null} status="loading" />);
  const error = renderToStaticMarkup(
    <BenchmarkDecisionRunDetails
      details={null}
      errorMessage="Context access is forbidden"
      status="error"
    />
  );
  const empty = renderToStaticMarkup(
    <BenchmarkDecisionRunDetails
      details={{
        project_id: "project-a",
        context_id: "context-a",
        commit_id: "commit-a",
        decision_id: "decision-a",
        runs: []
      }}
      status="ready"
    />
  );

  assert.match(loading, /Loading sealed run details/);
  assert.match(loading, /正在加载已封存运行详情/);
  assert.match(error, /role="alert"/);
  assert.match(error, /Run details unavailable/);
  assert.match(error, /运行详情不可用/);
  assert.match(empty, /No sealed runs/);
  assert.match(empty, /暂无已封存运行/);
});
