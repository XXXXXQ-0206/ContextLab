import assert from "node:assert/strict";
import test from "node:test";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import {
  presentLocalBenchmarkDecisionDiscovery,
  type BenchmarkDecisionDiscoveryResource
} from "./context-benchmark-decision-discovery-presenter";
import { BenchmarkDecisionDiscoveryState } from "./context-benchmark-evidence-inspector";

const target = {
  projectId: "project-a",
  contextId: "context-a",
  commitId: "commit-a"
};

test("benchmark decision discovery presents the first exact decision deterministically", () => {
  const view = presentLocalBenchmarkDecisionDiscovery({
    kind: "ready",
    target,
    summary: {
      schema_version: "contextlab.local-benchmark-decision-discovery.v1",
      project_id: "project-a",
      context_id: "context-a",
      commit_id: "commit-a",
      decisions: [
        {
          decision_id: "decision-b",
          suite_id: "suite-b",
          status: "regressed",
          recorded_at: "2026-07-22T00:01:00Z"
        },
        {
          decision_id: "decision-a",
          suite_id: "suite-a",
          status: "passed",
          recorded_at: "2026-07-22T00:00:00Z"
        }
      ]
    }
  });

  assert.equal(view.status.state, "available");
  assert.equal(view.firstDecisionId, "decision-b");
  assert.equal(view.decisions[0]?.decisionId, "decision-b");
  assert.match(view.status.description, /精确范围内/);
});

test("benchmark decision discovery screen renders all shared capability states and redacts fields", () => {
  const resources: BenchmarkDecisionDiscoveryResource[] = [
    { kind: "loading", target },
    { kind: "error", target, message: "Safe error / 安全错误" },
    { kind: "empty", target },
    { kind: "unavailable", target },
    {
      kind: "ready",
      target,
      summary: {
        schema_version: "contextlab.local-benchmark-decision-discovery.v1",
        project_id: "project-a",
        context_id: "context-a",
        commit_id: "commit-a",
        decisions: [{
          decision_id: "decision-a",
          suite_id: "suite-a",
          status: "passed",
          recorded_at: "2026-07-22T00:00:00Z"
        }]
      }
    }
  ];

  for (const resource of resources) {
    const markup = renderToStaticMarkup(
      <BenchmarkDecisionDiscoveryState view={presentLocalBenchmarkDecisionDiscovery(resource)} />
    );
    assert.match(markup, /Benchmark decision discovery/);
    assert.match(markup, /Benchmark decision 发现/);
    assert.match(markup, /Capability status/);
    assert.match(markup, /role="(status|alert)"/);
    assert.doesNotMatch(markup, /raw_output|input|expected output|case payload/i);
  }
});

test("benchmark decision discovery maps unavailable and error states bilingually", () => {
  const unavailable = presentLocalBenchmarkDecisionDiscovery({ kind: "unavailable", target });
  const error = presentLocalBenchmarkDecisionDiscovery({
    kind: "error",
    target,
    message: "Rate limit active / 速率限制已生效。"
  });

  assert.equal(unavailable.status.state, "unavailable");
  assert.equal(error.status.state, "error");
  assert.match(error.status.detail ?? "", /速率限制/);
});
