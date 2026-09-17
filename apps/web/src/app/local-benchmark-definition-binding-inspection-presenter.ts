import type { CapabilityStateKind } from "@contextlab/ui";
import type {
  LocalBenchmarkDefinitionBindingInspectionResource,
  LocalBenchmarkDefinitionBindingInspectionState,
  LocalBenchmarkDefinitionBindingInspectionTarget
} from "./local-benchmark-definition-binding-inspection-data";

export type LocalBenchmarkDefinitionBindingInspectionViewModel = Readonly<{
  title: string;
  description: string;
  state: CapabilityStateKind;
  status: Readonly<{
    state: CapabilityStateKind;
    ariaLabel: string;
    label: string;
    stateLabel: string;
    description: string;
    detail: string;
  }>;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  bindings: ReadonlyArray<Readonly<{
    id: string;
    label: string;
    suite: string;
    branch: string;
    datasetCount: string;
    capturedAt: string;
  }>>;
  selectedBindingId: string;
}>;

const stateCopy: Record<LocalBenchmarkDefinitionBindingInspectionState, Readonly<{
  label: string;
  description: string;
}>> = {
  loading: {
    label: "Inspecting... / 正在检查...",
    description: "The server is loading exact immutable bindings. / 服务端正在加载精确不可变绑定。"
  },
  error: {
    label: "Inspection failed / 检查失败",
    description: "The protected binding read failed closed. / 受保护的绑定读取已失败关闭。"
  },
  empty: {
    label: "No bindings / 没有绑定",
    description: "No authored definition is recorded for this exact commit. / 此精确提交尚无已记录定义。"
  },
  available: {
    label: "Bindings available / 绑定可用",
    description: "The server returned redacted immutable binding metadata. / 服务端返回了脱敏不可变绑定 metadata。"
  },
  unavailable: {
    label: "Inspection unavailable / 检查不可用",
    description: "This local read is disabled or has no exact commit target. / 本地读取已禁用或缺少精确提交目标。"
  }
};

export function presentLocalBenchmarkDefinitionBindingInspection(
  resource: LocalBenchmarkDefinitionBindingInspectionResource,
  enabled: boolean,
  selectedBindingId = ""
): LocalBenchmarkDefinitionBindingInspectionViewModel {
  const copy = stateCopy[resource.state];
  const bindings = resource.state === "available"
    ? resource.result.bindings.map((binding) => ({
        id: binding.binding_id,
        label: `${binding.suite_name} / ${binding.suite_id}`,
        suite: binding.suite_name,
        branch: binding.branch_name,
        datasetCount: `${binding.dataset_ids.length} / ${binding.dataset_ids.length} datasets / 数据集`,
        capturedAt: binding.captured_at
      }))
    : [];
  const nextSelectedBindingId = bindings.some((binding) => binding.id === selectedBindingId)
    ? selectedBindingId
    : (bindings[0]?.id ?? "");
  return Object.freeze({
    title: "Authored Benchmark Bindings / 已创作 Benchmark 绑定",
    description: "Inspect and select one exact immutable definition binding. / 检查并选择一条精确不可变定义绑定。",
    state: resource.state,
    status: {
      state: resource.state,
      ariaLabel: "Private benchmark definition binding inspection status / 私有 Benchmark 定义绑定检查状态",
      label: "Exact binding metadata / 精确绑定 metadata",
      stateLabel: copy.label,
      description: copy.description,
      detail: resource.state === "available"
        ? `${bindings.length} binding(s) / ${bindings.length} 条绑定`
        : resource.message ?? (enabled
          ? `${resource.target.commitId}`
          : "Disabled local development gate / 本地开发开关已关闭")
    },
    scope: [
      { id: "project", label: "Project / 项目", value: resource.target.projectId || "-" },
      { id: "context", label: "Context / 上下文", value: resource.target.contextId || "-" },
      { id: "commit", label: "Exact commit / 精确提交", value: resource.target.commitId || "-" }
    ],
    bindings,
    selectedBindingId: nextSelectedBindingId
  });
}

export function bindingInspectionErrorMessage(status: number, errorCode?: string): string {
  if (status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
  if (status === 403) return "You cannot inspect this exact Context target / 你无权检查此精确 Context 目标。";
  if (status === 429) return "The private binding read rate limit is active / 私有绑定读取速率限制已生效。";
  if (status === 503 && errorCode === "benchmark_definition_binding_unavailable") {
    return "Binding inspection is unavailable in this runtime / 此运行时无法检查绑定。";
  }
  return "The protected binding inspection could not be completed / 受保护绑定检查未完成。";
}

export function createInitialBindingInspectionResource(
  target: LocalBenchmarkDefinitionBindingInspectionTarget,
  enabled: boolean
): LocalBenchmarkDefinitionBindingInspectionResource {
  return Object.freeze({
    state: enabled && target.commitId.trim() ? "empty" : "unavailable",
    target: Object.freeze(target),
    message: enabled && target.commitId.trim()
      ? "No binding metadata loaded. / 尚未加载绑定 metadata。"
      : "Private inspection is off or no exact commit exists. / 私有检查已关闭或不存在精确提交。"
  });
}
