import type {
  LocalBenchmarkDecisionDiff,
  LocalBenchmarkMetricEvidence
} from "@contextlab/local-sdk";
import type { StatusPillTone } from "@contextlab/ui";

export type BenchmarkDecisionDiffViewModel = {
  status: { label: string; tone: StatusPillTone };
  scopes: Array<{ id: string; label: string; value: string }>;
  metrics: BenchmarkDecisionDiffMetricViewModel[];
};

export type BenchmarkDecisionDiffMetricViewModel = {
  id: string;
  change: string;
  metric: string;
  baseline: string;
  revised: string;
  coverage: string;
  outcome: { label: string; tone: StatusPillTone };
};

export function presentBenchmarkDecisionDiff(
  diff: LocalBenchmarkDecisionDiff
): BenchmarkDecisionDiffViewModel {
  return {
    status: presentStatusChange(diff.status_change),
    scopes: [
      { id: "baseline-commit", label: "Baseline commit / 基线提交", value: diff.baseline.commit_id },
      { id: "baseline-decision", label: "Baseline decision / 基线决策", value: diff.baseline.decision_id },
      { id: "revised-commit", label: "Revised commit / 修订提交", value: diff.revised.commit_id },
      { id: "revised-decision", label: "Revised decision / 修订决策", value: diff.revised.decision_id }
    ],
    metrics: diff.metric_changes.map(presentMetricChange)
  };
}

export function canCompareBenchmarkDecisions(state: {
  bearerToken: string;
  baselineCommitId: string;
  baselineDecisionId: string;
  revisedCommitId: string;
  revisedDecisionId: string;
  isLoading: boolean;
}): boolean {
  return Boolean(
    state.bearerToken.trim()
    && state.baselineCommitId.trim()
    && state.baselineDecisionId.trim()
    && state.revisedCommitId.trim()
    && state.revisedDecisionId.trim()
    && !state.isLoading
  );
}

export function benchmarkDecisionDiffErrorMessage(status: number, retryAfterMs?: number): string {
  if (status === 401) {
    return "Bearer authentication is required / 需要 Bearer 身份验证。";
  }
  if (status === 403) {
    return "You do not have permission for this Context / 你没有此 Context 的权限。";
  }
  if (status === 404) {
    return "One exact benchmark decision was not found / 未找到某个精确范围内的 benchmark decision。";
  }
  if (status === 409) {
    return "The benchmark decisions cannot be compared / 两个 benchmark decision 无法比较。";
  }
  if (status === 429) {
    const retry = retryAfterMs ? ` Retry in ${Math.ceil(retryAfterMs / 1_000)} seconds.` : "";
    return `The local rate limit is active / 本地速率限制已生效。${retry}`;
  }
  if (status === 503) {
    return "Benchmark decision comparison is unavailable / Benchmark decision 比较暂不可用。";
  }
  return "The local benchmark decision comparison could not be completed / 本地 benchmark decision 比较未能完成。";
}

function presentMetricChange(
  change: LocalBenchmarkDecisionDiff["metric_changes"][number]
): BenchmarkDecisionDiffMetricViewModel {
  const baseline = change.kind === "added" ? null : change.baseline;
  const revised = change.kind === "removed" ? null : change.revised;
  const evidence = revised ?? baseline;

  return {
    id: `${change.kind}:${change.metric}`,
    change: changeLabel(change.kind),
    metric: formatMetric(change.metric),
    baseline: baseline ? formatObserved(baseline) : "Not present / 不存在",
    revised: revised ? formatObserved(revised) : "Not present / 不存在",
    coverage: evidence ? formatCoverage(evidence) : "Not observed / 未观测",
    outcome: presentDecisionStatus(evidence?.outcome ?? "insufficient_data")
  };
}

function presentStatusChange(
  statusChange: LocalBenchmarkDecisionDiff["status_change"]
): { label: string; tone: StatusPillTone } {
  if (!statusChange) {
    return { label: "No status change / 无状态变化", tone: "neutral" };
  }
  return {
    label: `${formatStatus(statusChange.baseline)} -> ${formatStatus(statusChange.revised)}`,
    tone: statusChange.revised === "regressed" ? "warning" : "info"
  };
}

function presentDecisionStatus(status: "passed" | "regressed" | "insufficient_data"): {
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

function formatStatus(status: "passed" | "regressed" | "insufficient_data"): string {
  return presentDecisionStatus(status).label;
}

function changeLabel(kind: "added" | "removed" | "modified"): string {
  if (kind === "added") {
    return "Added / 已新增";
  }
  if (kind === "removed") {
    return "Removed / 已移除";
  }
  return "Modified / 已修改";
}

function formatMetric(metric: string): string {
  return metric
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function formatObserved(metric: LocalBenchmarkMetricEvidence): string {
  return metric.observed === null ? "Not observed / 未观测" : formatNumber(metric.observed);
}

function formatCoverage(metric: LocalBenchmarkMetricEvidence): string {
  return `${metric.sample_count} / ${metric.required_sample_count} ${metric.has_complete_coverage ? "complete / 完整" : "incomplete / 不完整"}`;
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 4 }).format(value);
}
