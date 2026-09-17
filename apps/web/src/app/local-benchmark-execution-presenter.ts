import type { CapabilityStateKind, StatusPillTone } from "@contextlab/ui";
import type {
  LocalBenchmarkExecutionResource,
  LocalBenchmarkExecutionTarget,
  LocalBenchmarkExecutionResourceState
} from "./local-benchmark-execution-data";

export type LocalBenchmarkExecutionViewModel = Readonly<{
  title: string;
  description: string;
  state: LocalBenchmarkExecutionResourceState;
  capabilityState: CapabilityStateKind;
  status: Readonly<{
    ariaLabel: string;
    label: string;
    stateLabel: string;
    description: string;
    detail: string;
  }>;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  receipt: Readonly<{
    disposition: string;
    projectionDisposition: string;
    bindingId: string;
    decisionId: string;
    suiteId: string;
    datasetIds: string;
    cohortId: string;
  }> | null;
}>;

const stateCopy: Record<LocalBenchmarkExecutionResourceState, Readonly<{
  capabilityState: CapabilityStateKind;
  stateLabel: string;
  description: string;
  tone: StatusPillTone;
}>> = {
  loading: { capabilityState: "loading", stateLabel: "Loading / 加载中", description: "The exact benchmark execution is running. / 正在执行精确 Benchmark。", tone: "info" },
  error: { capabilityState: "error", stateLabel: "Unable to load / 加载失败", description: "The protected execution request failed. / 受保护的执行请求失败。", tone: "warning" },
  empty: { capabilityState: "empty", stateLabel: "Ready / 已准备", description: "No execution receipt has been created yet. / 尚未创建执行回执。", tone: "neutral" },
  created: { capabilityState: "available", stateLabel: "Created / 已创建", description: "A new server-owned execution receipt is available. / 新的服务端执行回执已可用。", tone: "success" },
  replayed: { capabilityState: "available", stateLabel: "Replayed / 已回放", description: "The server replayed the immutable execution receipt. / 服务端已回放不可变执行回执。", tone: "success" },
  conflict: { capabilityState: "error", stateLabel: "Conflict / 冲突", description: "The idempotent execution conflicts with immutable state. / 幂等执行与不可变状态冲突。", tone: "warning" },
  unavailable: { capabilityState: "unavailable", stateLabel: "Unavailable / 不可用", description: "The local evaluator or execution adapter is unavailable. / 本地评测器或执行适配器不可用。", tone: "warning" }
};

export function presentLocalBenchmarkExecution(
  resource: LocalBenchmarkExecutionResource
): LocalBenchmarkExecutionViewModel {
  const copy = stateCopy[resource.state];
  return deepFreeze({
    title: "Local Benchmark execution / 本地 Benchmark 执行",
    description: "Private, server-owned execution with replayable metadata only. / 私有、服务端控制且仅可回放元数据的执行。",
    state: resource.state,
    capabilityState: copy.capabilityState,
    status: {
      ariaLabel: "Local Benchmark execution status / 本地 Benchmark 执行状态",
      label: "Benchmark execution / Benchmark 执行",
      stateLabel: copy.stateLabel,
      description: copy.description,
      detail: resource.state === "created" || resource.state === "replayed"
        ? `Disposition ${resource.receipt.disposition}; projection ${resource.receipt.projection_disposition}.`
        : ("message" in resource ? resource.message : undefined) ?? targetDetail(resource.target)
    },
    scope: presentScope(resource.target),
    receipt: resource.state === "created" || resource.state === "replayed"
      ? {
          disposition: `${resource.receipt.disposition} / ${resource.receipt.disposition === "created" ? "已创建" : "已回放"}`,
          projectionDisposition: `${resource.receipt.projection_disposition} / ${resource.receipt.projection_disposition === "created" ? "已创建" : "已回放"}`,
          bindingId: resource.receipt.binding_id,
          decisionId: resource.receipt.decision_id,
          suiteId: resource.receipt.suite_id,
          datasetIds: resource.receipt.dataset_ids.join(", "),
          cohortId: resource.receipt.cohort_id
        }
      : null
  });
}

function presentScope(target: LocalBenchmarkExecutionTarget) {
  return [
    { id: "project", label: "Project / 项目", value: target.projectId },
    { id: "context", label: "Context / 上下文", value: target.contextId },
    { id: "commit", label: "Commit / 提交", value: target.commitId }
  ];
}

function targetDetail(target: LocalBenchmarkExecutionTarget): string {
  return `${target.projectId} / ${target.contextId} @ ${target.commitId}`;
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
