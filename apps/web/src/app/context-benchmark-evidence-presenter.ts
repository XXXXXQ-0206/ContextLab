import type {
  LocalBenchmarkDecision,
  LocalBenchmarkDecisionRunDetails,
  LocalBenchmarkDecisionRunMetric,
  LocalBenchmarkMetricEvidence
} from "@contextlab/local-sdk";
import type { StatusPillTone } from "@contextlab/ui";

export type BenchmarkDecisionViewModel = {
  status: { label: string; tone: StatusPillTone };
  facts: Array<{ id: string; label: string; value: string }>;
  comparability: Array<{ id: string; label: string; value: string }>;
  definition: BenchmarkDefinitionViewModel;
  datasets: string[];
  runs: string[];
  metrics: BenchmarkMetricViewModel[];
};

export type BenchmarkDefinitionViewModel = {
  suite: {
    id: string;
    name: string;
    thresholds: BenchmarkThresholdViewModel[];
  };
  datasets: BenchmarkDatasetViewModel[];
};

export type BenchmarkThresholdViewModel = {
  id: string;
  label: string;
  value: string;
};

export type BenchmarkDatasetViewModel = {
  id: string;
  name: string;
  caseCount: string;
};

export type BenchmarkMetricViewModel = {
  id: string;
  metric: string;
  threshold: string;
  observed: string;
  coverage: string;
  outcome: { label: string; tone: StatusPillTone };
};

export type BenchmarkDecisionRunDetailsViewModel = {
  runs: BenchmarkDecisionRunViewModel[];
};

export type BenchmarkDecisionRunViewModel = {
  id: string;
  facts: Array<{ id: string; label: string; value: string }>;
  metrics: BenchmarkDecisionRunMetricViewModel[];
};

export type BenchmarkDecisionRunMetricViewModel = {
  id: string;
  label: string;
  value: string;
};

export function presentBenchmarkDecision(evidence: LocalBenchmarkDecision): BenchmarkDecisionViewModel {
  return {
    status: presentDecisionStatus(evidence.status),
    facts: [
      { id: "decision-id", label: "Decision / 决策", value: evidence.decision_id },
      { id: "suite-id", label: "Suite / 套件", value: evidence.suite_id },
      { id: "commit-id", label: "Commit / 提交", value: evidence.commit_id },
      { id: "recorded-at", label: "Recorded / 记录时间", value: formatTimestamp(evidence.recorded_at) }
    ],
    comparability: [
      { id: "evaluator-key", label: "Evaluator / 评测器", value: evidence.comparability.evaluator_key },
      { id: "evaluator-version", label: "Version / 版本", value: evidence.comparability.evaluator_version },
      { id: "fingerprint", label: "Fingerprint / 指纹", value: evidence.comparability.fingerprint },
      { id: "evidence-digest", label: "Evidence digest / 证据摘要", value: evidence.evidence_digest }
    ],
    definition: presentDefinition(evidence),
    datasets: [...evidence.dataset_ids].sort(compareCodePoints),
    runs: [...evidence.run_ids].sort(compareCodePoints),
    metrics: [...evidence.metrics]
      .sort((left, right) => compareCodePoints(left.metric, right.metric))
      .map(presentMetric)
  };
}

export function presentBenchmarkDecisionRunDetails(
  details: LocalBenchmarkDecisionRunDetails
): BenchmarkDecisionRunDetailsViewModel {
  return {
    runs: [...details.runs]
      .map((run) => ({
        id: run.run_id,
        facts: [
          { id: "run-id", label: "Run / 运行", value: run.run_id },
          { id: "model-version", label: "Model version / 模型版本", value: run.model_version },
          { id: "temperature", label: "Temperature / 温度", value: formatNumber(run.temperature) },
          { id: "executed-at", label: "Executed / 执行时间", value: formatTimestamp(run.executed_at) }
        ],
        metrics: [...run.metrics]
          .sort((left, right) => compareCodePoints(left.metric, right.metric))
          .map(presentRunMetric)
      }))
  };
}

function presentDefinition(evidence: LocalBenchmarkDecision): BenchmarkDefinitionViewModel {
  const definition = evidence.definition;

  return {
    suite: {
      id: definition.suite.id,
      name: definition.suite.name,
      thresholds: definition.suite.thresholds.map((threshold) => ({
        id: threshold.metric,
        label: formatThresholdLabel(threshold.metric, threshold.direction),
        value: `${threshold.direction === "minimum" ? ">=" : "<="} ${formatNumber(threshold.value)}`
      }))
    },
    datasets: definition.datasets.map((dataset) => ({
      id: dataset.id,
      name: dataset.name,
      caseCount: formatCaseCount(dataset.case_count)
    }))
  };
}

export function canInspectBenchmarkEvidence(state: {
  bearerToken: string;
  decisionId: string;
  commitId: string;
  isLoading: boolean;
}): boolean {
  return Boolean(
    state.bearerToken.trim() && state.decisionId.trim() && state.commitId.trim() && !state.isLoading
  );
}

export function benchmarkEvidenceErrorMessage(status: number, retryAfterMs?: number): string {
  if (status === 401) {
    return "Bearer authentication is required / 需要 Bearer 身份验证。";
  }
  if (status === 403) {
    return "You do not have permission for this Context / 你没有此 Context 的权限。";
  }
  if (status === 404) {
    return "The exact benchmark decision was not found / 未找到精确范围内的 benchmark decision。";
  }
  if (status === 429) {
    const retry = retryAfterMs ? ` Retry in ${Math.ceil(retryAfterMs / 1_000)} seconds.` : "";
    return `The local rate limit is active / 本地速率限制已生效。${retry}`;
  }
  if (status === 503) {
    return "Benchmark evidence inspection is unavailable / Benchmark 证据审阅暂不可用。";
  }
  return "The local benchmark evidence request could not be completed / 本地 benchmark 证据请求未能完成。";
}

function presentMetric(metric: LocalBenchmarkMetricEvidence): BenchmarkMetricViewModel {
  const direction = metric.threshold_direction === "minimum" ? ">=" : "<=";

  return {
    id: metric.metric,
    metric: formatMetric(metric.metric),
    threshold: `${direction} ${formatNumber(metric.threshold_value)}`,
    observed: metric.observed === null ? "Not observed / 未观测" : formatNumber(metric.observed),
    coverage: `${metric.sample_count} / ${metric.required_sample_count} ${metric.has_complete_coverage ? "complete / 完整" : "incomplete / 不完整"}`,
    outcome: presentDecisionStatus(metric.outcome)
  };
}

function presentRunMetric(metric: LocalBenchmarkDecisionRunMetric): BenchmarkDecisionRunMetricViewModel {
  return {
    id: metric.metric,
    label: formatMetricLabel(metric.metric),
    value: formatNumber(metric.value)
  };
}

function presentDecisionStatus(status: LocalBenchmarkDecision["status"]): {
  label: string;
  tone: StatusPillTone;
} {
  if (status === "passed") {
    return { label: "Passed / 通过", tone: "success" };
  }
  if (status === "regressed") {
    return { label: "Regressed / 回归", tone: "warning" };
  }
  return { label: "Insufficient data / 数据不足", tone: "neutral" };
}

function formatMetric(metric: string): string {
  return metric
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function formatMetricLabel(metric: LocalBenchmarkMetricEvidence["metric"]): string {
  const label = metricLabels[metric];
  return `${label.english} / ${label.chinese}`;
}

function formatThresholdLabel(
  metric: LocalBenchmarkMetricEvidence["metric"],
  direction: LocalBenchmarkMetricEvidence["threshold_direction"]
): string {
  const metricLabel = metricLabels[metric];
  const directionLabel = direction === "minimum"
    ? { english: "minimum", chinese: "最低" }
    : { english: "maximum", chinese: "最高" };

  return `${metricLabel.english} ${directionLabel.english} / ${metricLabel.chinese}${directionLabel.chinese}`;
}

function formatCaseCount(caseCount: number): string {
  const count = formatNumber(caseCount);
  const unit = caseCount === 1 ? "case" : "cases";

  return `${count} ${unit} / ${count} 个用例`;
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 4 }).format(value);
}

function formatTimestamp(value: string): string {
  return value.replace("T", " ").replace("Z", " UTC");
}

function compareCodePoints(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}

const metricLabels: Record<LocalBenchmarkMetricEvidence["metric"], { english: string; chinese: string }> = {
  latency_ms: { english: "Latency", chinese: "延迟" },
  cost_usd: { english: "Cost", chinese: "成本" },
  accuracy: { english: "Accuracy", chinese: "准确率" },
  hallucination_rate: { english: "Hallucination rate", chinese: "幻觉率" },
  tool_usage_count: { english: "Tool usage", chinese: "工具使用" },
  token_count: { english: "Token count", chinese: "Token 数" },
  execution_time_ms: { english: "Execution time", chinese: "执行时间" },
  output_quality: { english: "Output quality", chinese: "输出质量" },
  success_rate: { english: "Success rate", chinese: "成功率" }
};
