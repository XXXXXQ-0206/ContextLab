import type {
  LocalBenchmarkWorkspaceDecisionStatus,
  LocalBenchmarkWorkspaceMetricKind
} from "@contextlab/local-sdk";
import type { CapabilityStateKind, StatusPillTone } from "@contextlab/ui";
import {
  presentLocalBenchmarkDecisionDiscovery,
  type BenchmarkDecisionDiscoveryResource,
  type BenchmarkDecisionDiscoveryViewModel
} from "./context-benchmark-decision-discovery-presenter";
import type { LocalBenchmarkDecisionDiscovery } from "./context-benchmark-decision-discovery-data";
import type {
  FrozenLocalBenchmarkWorkspace,
  LocalBenchmarkWorkspaceDecisionDiscoveryTarget,
  LocalBenchmarkWorkspaceResource,
  LocalBenchmarkWorkspaceTarget
} from "./local-benchmark-workspace-data";

export type LocalBenchmarkWorkspaceFact = Readonly<{
  id: string;
  label: string;
  value: string;
}>;

export type LocalBenchmarkWorkspaceDecisionOption = Readonly<{
  id: string;
  label: string;
  value: string;
}>;

export type LocalBenchmarkWorkspaceDecisionDiscoveryViewModel = Readonly<{
  status: BenchmarkDecisionDiscoveryViewModel["status"];
  options: ReadonlyArray<LocalBenchmarkWorkspaceDecisionOption>;
  firstDecisionId: string | null;
}>;

export type LocalBenchmarkWorkspaceTableRow = Readonly<{
  id: string;
  cells: ReadonlyArray<string>;
}>;

export type LocalBenchmarkWorkspaceStatusViewModel = Readonly<{
  state: CapabilityStateKind;
  ariaLabel: string;
  label: string;
  stateLabel: string;
  description: string;
  detail: string;
}>;

export type LocalBenchmarkWorkspaceProjectionViewModel = Readonly<{
  suite: Readonly<{ id: string; name: string }>;
  datasets: Readonly<{
    rows: ReadonlyArray<LocalBenchmarkWorkspaceTableRow>;
    emptyMessage: string;
  }>;
  runs: Readonly<{
    rows: ReadonlyArray<LocalBenchmarkWorkspaceTableRow>;
    emptyMessage: string;
  }>;
  scorecard: Readonly<{
    runCount: string;
    rows: ReadonlyArray<LocalBenchmarkWorkspaceTableRow>;
    emptyMessage: string;
  }>;
  regressionStatus: Readonly<{
    label: string;
    tone: StatusPillTone;
  }>;
  evaluationDiff: Readonly<{
    scopes: ReadonlyArray<LocalBenchmarkWorkspaceFact>;
    statusChange: string;
    rows: ReadonlyArray<LocalBenchmarkWorkspaceTableRow>;
    emptyMessage: string;
  }> | null;
}>;

export type LocalBenchmarkWorkspaceViewModel = Readonly<{
  title: string;
  description: string;
  state: CapabilityStateKind;
  status: LocalBenchmarkWorkspaceStatusViewModel;
  scope: ReadonlyArray<LocalBenchmarkWorkspaceFact>;
  projection: LocalBenchmarkWorkspaceProjectionViewModel | null;
}>;

type FrozenMetric = FrozenLocalBenchmarkWorkspace["projection"]["scorecard"]["metrics"][number];

const metricLabels: Record<LocalBenchmarkWorkspaceMetricKind, string> = {
  latency_ms: "Latency (ms) / 延迟（毫秒）",
  cost_usd: "Cost (USD) / 成本（美元）",
  accuracy: "Accuracy / 准确率",
  hallucination_rate: "Hallucination rate / 幻觉率",
  tool_usage_count: "Tool usage count / 工具使用次数",
  token_count: "Token count / Token 数量",
  execution_time_ms: "Execution time (ms) / 执行时间（毫秒）",
  output_quality: "Output quality / 输出质量",
  success_rate: "Success rate / 成功率"
};

const stateCopy: Record<
  CapabilityStateKind,
  Readonly<{ stateLabel: string; description: string }>
> = {
  loading: {
    stateLabel: "Loading / 加载中",
    description: "The exact server projection is loading. / 正在加载精确的服务端投影。"
  },
  error: {
    stateLabel: "Unable to load / 加载失败",
    description: "The protected benchmark workspace read failed. / 受保护的 Benchmark 工作台读取失败。"
  },
  empty: {
    stateLabel: "No projection / 暂无投影",
    description: "No server projection loaded. / 尚未加载服务端投影。"
  },
  available: {
    stateLabel: "Available / 可用",
    description: "The exact server-owned projection is available. / 精确的服务端投影已可用。"
  },
  unavailable: {
    stateLabel: "Unavailable / 不可用",
    description: "An exact benchmark workspace read is unavailable for this scope. / 当前范围无法进行精确的 Benchmark 工作台读取。"
  }
};

export function presentLocalBenchmarkWorkspace(
  resource: LocalBenchmarkWorkspaceResource
): LocalBenchmarkWorkspaceViewModel {
  const copy = stateCopy[resource.state];
  return deepFreeze({
    title: "Local Benchmark Workspace / 本地 Benchmark 工作台",
    description:
      "Protected server-owned scorecard, regression status, provenance, and evaluation diff. / 受保护的服务端记分卡、回归状态、来源与评测差异。",
    state: resource.state,
    status: {
      state: resource.state,
      ariaLabel: "Local benchmark workspace status / 本地 Benchmark 工作台状态",
      label: "Benchmark workspace projection / Benchmark 工作台投影",
      stateLabel: copy.stateLabel,
      description: copy.description,
      detail: resource.state === "available"
        ? `Schema ${resource.workspace.schema_version}; projection v${resource.workspace.projection.schema_version}`
        : resource.message ?? targetDetail(resource.target)
    },
    scope: presentScope(
      resource.target,
      resource.state === "available" ? resource.workspace : undefined
    ),
    projection: resource.state === "available" ? presentProjection(resource.workspace) : null
  });
}

export function presentLocalBenchmarkWorkspaceDecisionDiscovery(
  resource: BenchmarkDecisionDiscoveryResource
): LocalBenchmarkWorkspaceDecisionDiscoveryViewModel {
  const view = presentLocalBenchmarkDecisionDiscovery(resource);
  const options = Object.freeze(view.decisions.map((decision) => Object.freeze({
    id: decision.id,
    label: `${decision.suiteId} · ${decision.status.label} · ${decision.recordedAt}`,
    value: decision.decisionId
  })));

  return Object.freeze({
    status: view.status,
    options,
    firstDecisionId: view.firstDecisionId
  });
}

export function presentLocalBenchmarkWorkspaceDecisionDiscoveries(
  target: LocalBenchmarkWorkspaceDecisionDiscoveryTarget,
  discoveries: Readonly<{
    revised: LocalBenchmarkDecisionDiscovery;
    baseline: LocalBenchmarkDecisionDiscovery | null;
  }>
): Readonly<{
  revised: LocalBenchmarkWorkspaceDecisionDiscoveryViewModel;
  baseline: LocalBenchmarkWorkspaceDecisionDiscoveryViewModel;
}> {
  const revisedTarget = {
    projectId: target.projectId,
    contextId: target.contextId,
    commitId: target.revisedCommitId
  };
  const revised = presentLocalBenchmarkWorkspaceDecisionDiscovery({
    kind: "ready",
    target: revisedTarget,
    summary: discoveries.revised
  });
  const baseline = target.baselineCommitId === undefined
    ? presentLocalBenchmarkWorkspaceDecisionDiscovery({
        kind: "empty",
        target: {
          projectId: target.projectId,
          contextId: target.contextId,
          commitId: ""
        }
      })
    : discoveries.baseline === null
      ? presentLocalBenchmarkWorkspaceDecisionDiscovery({
          kind: "empty",
          target: {
            projectId: target.projectId,
            contextId: target.contextId,
            commitId: target.baselineCommitId
          }
        })
      : presentLocalBenchmarkWorkspaceDecisionDiscovery({
          kind: "ready",
          target: {
            projectId: target.projectId,
            contextId: target.contextId,
            commitId: target.baselineCommitId
          },
          summary: discoveries.baseline
        });

  return Object.freeze({ revised, baseline });
}

function presentScope(
  target: LocalBenchmarkWorkspaceTarget,
  workspace: FrozenLocalBenchmarkWorkspace | undefined
): LocalBenchmarkWorkspaceFact[] {
  const witness = workspace?.decision_pair_witness;
  if (witness !== undefined) {
    return [
      { id: "project", label: "Project / 项目", value: witness.project_id },
      { id: "context", label: "Context / 上下文", value: witness.context_id },
      {
        id: "revised-commit",
        label: "Revised commit / 修订提交",
        value: witness.revised.commit_id
      },
      {
        id: "revised-decision",
        label: "Revised decision witness / 修订 decision witness",
        value: witness.revised.decision_id
      },
      {
        id: "baseline-commit",
        label: "Baseline commit / 基线提交",
        value: witness.baseline.commit_id
      },
      {
        id: "baseline-decision",
        label: "Baseline decision witness / 基线 decision witness",
        value: witness.baseline.decision_id
      }
    ];
  }

  return [
    { id: "project", label: "Project / 项目", value: target.projectId },
    { id: "context", label: "Context / 上下文", value: target.contextId },
    { id: "revised-commit", label: "Revised commit / 修订提交", value: target.commitId },
    {
      id: "revised-decision",
      label: "Revised sealed decision / 修订 sealed decision",
      value: target.decisionId ?? target.cohortId ?? "-"
    },
    ...(target.baseline
      ? [
          {
            id: "baseline-commit",
            label: "Baseline commit / 基线提交",
            value: target.baseline.commitId
          },
          {
            id: "baseline-decision",
            label: "Baseline sealed decision / 基线 sealed decision",
            value: target.baseline.decisionId ?? target.baseline.cohortId ?? "-"
          }
        ]
      : [])
  ];
}

function presentProjection(
  workspace: FrozenLocalBenchmarkWorkspace
): LocalBenchmarkWorkspaceProjectionViewModel {
  const projection = workspace.projection;
  return {
    suite: {
      id: projection.suite.id,
      name: projection.suite.name
    },
    datasets: {
      rows: projection.datasets.map((dataset) => ({
        id: dataset.id,
        cells: [dataset.id, dataset.name, String(dataset.case_count)]
      })),
      emptyMessage: "No datasets in the server projection. / 服务端投影中没有数据集。"
    },
    runs: {
      rows: projection.runs.map((run) => ({
        id: `${run.dataset_id}\u0000${run.case_id}`,
        cells: [run.dataset_id, run.case_id, String(run.metric_count)]
      })),
      emptyMessage: "No run provenance in the server projection. / 服务端投影中没有运行来源。"
    },
    scorecard: {
      runCount: `${projection.scorecard.run_count} runs / ${projection.scorecard.run_count} 次运行`,
      rows: projection.scorecard.metrics.map((metric) => ({
        id: metric.metric,
        cells: [
          metricLabels[metric.metric],
          thresholdDirectionLabel(metric.threshold_direction),
          formatNumber(metric.threshold_value),
          formatObserved(metric.observed),
          `${metric.sample_count} / ${metric.required_sample_count}`,
          metric.has_complete_coverage ? "Complete / 完整" : "Incomplete / 不完整",
          statusView(metric.outcome).label
        ]
      })),
      emptyMessage: "No scorecard metrics in the server projection. / 服务端投影中没有记分卡指标。"
    },
    regressionStatus: statusView(projection.regression_status),
    evaluationDiff: projection.evaluation_diff === null
      ? null
      : {
          scopes: [
            {
              id: "diff-baseline-cohort",
              label: "Baseline cohort / 基线队列",
              value: projection.evaluation_diff.baseline_cohort_id
            },
            {
              id: "diff-revised-cohort",
              label: "Revised cohort / 修订队列",
              value: projection.evaluation_diff.revised_cohort_id
            }
          ],
          statusChange: projection.evaluation_diff.status_change === null
            ? "No status change / 状态未变化"
            : `${statusView(projection.evaluation_diff.status_change[0]).label} -> ${statusView(projection.evaluation_diff.status_change[1]).label}`,
          rows: projection.evaluation_diff.metric_changes.map((change) => ({
            id: `${change.metric}:${change.change_kind}`,
            cells: [
              metricLabels[change.metric],
              changeKindLabel(change.change_kind),
              formatDiffMetric(change.baseline),
              formatDiffMetric(change.revised)
            ]
          })),
          emptyMessage: "No metric changes in the server evaluation diff. / 服务端评测差异中没有指标变更。"
        }
  };
}

function statusView(
  status: LocalBenchmarkWorkspaceDecisionStatus
): Readonly<{ label: string; tone: StatusPillTone }> {
  if (status === "passed") {
    return { label: "Passed / 通过", tone: "success" };
  }
  if (status === "regressed") {
    return { label: "Regressed / 回归", tone: "warning" };
  }
  return { label: "Insufficient data / 数据不足", tone: "neutral" };
}

function thresholdDirectionLabel(direction: "minimum" | "maximum"): string {
  return direction === "minimum" ? "Minimum / 最小值" : "Maximum / 最大值";
}

function changeKindLabel(kind: "added" | "removed" | "modified"): string {
  if (kind === "added") {
    return "Added / 已新增";
  }
  if (kind === "removed") {
    return "Removed / 已移除";
  }
  return "Modified / 已修改";
}

function formatDiffMetric(metric: FrozenMetric | null): string {
  if (metric === null) {
    return "Not present / 不存在";
  }
  return [
    `observed ${formatObserved(metric.observed)}`,
    `${thresholdDirectionLabel(metric.threshold_direction)} ${formatNumber(metric.threshold_value)}`,
    `${metric.sample_count} / ${metric.required_sample_count}`,
    statusView(metric.outcome).label
  ].join("; ");
}

function formatObserved(value: number | null): string {
  return value === null ? "Not observed / 未观测" : formatNumber(value);
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 6 }).format(value);
}

function targetDetail(target: LocalBenchmarkWorkspaceTarget): string {
  const revised = `${target.commitId || "-"} @ ${target.decisionId ?? target.cohortId ?? "-"}`;
  return target.baseline
    ? `${revised}; baseline ${target.baseline.commitId || "-"} @ ${target.baseline.decisionId ?? target.baseline.cohortId ?? "-"}`
    : revised;
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) {
      deepFreeze(child);
    }
    Object.freeze(value);
  }
  return value;
}
