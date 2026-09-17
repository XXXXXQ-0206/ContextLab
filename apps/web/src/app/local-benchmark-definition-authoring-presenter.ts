import type { CapabilityStateKind } from "@contextlab/ui";
import type {
  LocalBenchmarkDefinitionAuthoringCommand,
  LocalBenchmarkDefinitionExpectedOutput,
  LocalBenchmarkDefinitionAuthoringResource,
  LocalBenchmarkDefinitionAuthoringTarget,
  LocalBenchmarkDefinitionJsonValue,
  LocalBenchmarkDefinitionMetric
} from "./local-benchmark-definition-authoring-data";

export type LocalBenchmarkDefinitionAuthoringFact = Readonly<{
  id: string;
  label: string;
  value: string;
}>;

export type LocalBenchmarkDefinitionAuthoringDraft = Readonly<{
  bearerToken: string;
  branchName: string;
  datasetName: string;
  caseName: string;
  caseInputText: string;
  expectedOutputText: string;
  suiteName: string;
  metric: LocalBenchmarkDefinitionMetric;
  direction: "minimum" | "maximum";
  thresholdValue: string;
}>;

export type LocalBenchmarkDefinitionAuthoringDraftField = keyof LocalBenchmarkDefinitionAuthoringDraft;

export type LocalBenchmarkDefinitionAuthoringFormView = Readonly<{
  values: LocalBenchmarkDefinitionAuthoringDraft;
  enabled: boolean;
  disabled: boolean;
  canSubmit: boolean;
  submitLabel: string;
  scopeNotice: string;
}>;

export type LocalBenchmarkDefinitionAuthoringViewModel = Readonly<{
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
  scope: ReadonlyArray<LocalBenchmarkDefinitionAuthoringFact>;
  result: Readonly<{
    disposition: string;
    bindingId: string;
    branch: string;
    suiteId: string;
    datasetCount: string;
    capturedAt: string;
    message: string;
  }> | null;
}>;

export type LocalBenchmarkDefinitionCommandBuildResult =
  | Readonly<{ ok: true; value: LocalBenchmarkDefinitionAuthoringCommand }>
  | Readonly<{ ok: false; message: string }>;

const stateCopy: Record<CapabilityStateKind, Readonly<{ label: string; description: string }>> = {
  loading: {
    label: "Authoring... / 正在创作...",
    description: "The private adapter is sealing the exact definition binding. / 私有适配器正在封存精确定义绑定。"
  },
  error: {
    label: "Unable to author / 创作失败",
    description: "The protected authoring request failed closed. / 受保护的创作请求已按失败关闭。"
  },
  empty: {
    label: "Ready for input / 等待输入",
    description: "No benchmark definition has been submitted for this exact target. / 尚未为此精确目标提交 Benchmark 定义。"
  },
  available: {
    label: "Binding available / 绑定可用",
    description: "The server returned an immutable exact-commit binding. / 服务端已返回不可变的精确提交绑定。"
  },
  unavailable: {
    label: "Authoring unavailable / 创作不可用",
    description: "This private local mutation is disabled or has no exact commit target. / 此私有本地变更已禁用或缺少精确提交目标。"
  }
};

export const localBenchmarkDefinitionMetricOptions: ReadonlyArray<Readonly<{
  value: LocalBenchmarkDefinitionMetric;
  label: string;
}>> = Object.freeze([
  { value: "accuracy", label: "Accuracy / 准确率" },
  { value: "latency_ms", label: "Latency (ms) / 延迟（毫秒）" },
  { value: "cost_usd", label: "Cost (USD) / 成本（美元）" },
  { value: "hallucination_rate", label: "Hallucination rate / 幻觉率" },
  { value: "tool_usage_count", label: "Tool usage count / 工具使用次数" },
  { value: "token_count", label: "Token count / Token 数量" },
  { value: "execution_time_ms", label: "Execution time (ms) / 执行时间（毫秒）" },
  { value: "output_quality", label: "Output quality / 输出质量" },
  { value: "success_rate", label: "Success rate / 成功率" }
]);

export function createLocalBenchmarkDefinitionAuthoringDraft(): LocalBenchmarkDefinitionAuthoringDraft {
  return Object.freeze({
    bearerToken: "",
    branchName: "main",
    datasetName: "",
    caseName: "",
    caseInputText: "{}",
    expectedOutputText: "",
    suiteName: "",
    metric: "accuracy",
    direction: "minimum",
    thresholdValue: ""
  });
}

export function createLocalBenchmarkDefinitionAuthoringFormView(input: Readonly<{
  enabled: boolean;
  isPending: boolean;
  canSubmit: boolean;
  values?: LocalBenchmarkDefinitionAuthoringDraft;
}>): LocalBenchmarkDefinitionAuthoringFormView {
  return Object.freeze({
    values: input.values ?? createLocalBenchmarkDefinitionAuthoringDraft(),
    enabled: input.enabled,
    disabled: !input.enabled || input.isPending,
    canSubmit: input.canSubmit,
    submitLabel: input.isPending
      ? "Authoring exact binding... / 正在创作精确绑定..."
      : "Create dataset and suite / 创建数据集与套件",
    scopeNotice: input.enabled
      ? "Local development scope; private mutation only / 本地开发范围；仅限私有变更。"
      : "Private mutation is off by default / 私有变更默认关闭。"
  });
}

export function presentLocalBenchmarkDefinitionAuthoring(
  resource: LocalBenchmarkDefinitionAuthoringResource,
  enabled: boolean
): LocalBenchmarkDefinitionAuthoringViewModel {
  const copy = stateCopy[resource.state];
  return deepFreeze({
    title: "Private Benchmark Definition Authoring / 私有 Benchmark 定义创作",
    description:
      "Create one minimal dataset and suite for an exact immutable Context commit. / 为精确且不可变的 Context 提交创建一个最小数据集与套件。",
    state: resource.state,
    status: {
      state: resource.state,
      ariaLabel: "Private benchmark definition authoring status / 私有 Benchmark 定义创作状态",
      label: "Exact definition binding / 精确定义绑定",
      stateLabel: copy.label,
      description: copy.description,
      detail: resource.state === "available"
        ? `${resource.result.disposition}; ${resource.result.binding_id}`
        : resource.message ?? (enabled
          ? targetDetail(resource.target)
          : "Disabled local development gate / 本地开发开关已关闭")
    },
    scope: [
      { id: "project", label: "Project / 项目", value: resource.target.projectId || "-" },
      { id: "context", label: "Context / 上下文", value: resource.target.contextId || "-" },
      { id: "commit", label: "Exact commit / 精确提交", value: resource.target.commitId || "-" }
    ],
    result: resource.state === "available"
      ? {
          disposition: resource.result.disposition === "created"
            ? "Created / 已创建"
            : "Replayed / 已重放",
          bindingId: resource.result.binding_id,
          branch: resource.result.branch_name,
          suiteId: resource.result.suite_id,
          datasetCount: `${resource.result.dataset_ids.length} / ${resource.result.dataset_ids.length} 个`,
          capturedAt: resource.result.captured_at,
          message: `${resource.result.message.en} / ${resource.result.message.zh}`
        }
      : null
  });
}

export function buildLocalBenchmarkDefinitionAuthoringCommand(
  target: LocalBenchmarkDefinitionAuthoringTarget,
  draft: Omit<LocalBenchmarkDefinitionAuthoringDraft, "bearerToken">,
  createId: () => string = createUuid
): LocalBenchmarkDefinitionCommandBuildResult {
  const branchName = draft.branchName.trim();
  const datasetName = draft.datasetName.trim();
  const caseName = draft.caseName.trim();
  const suiteName = draft.suiteName.trim();
  if (!target.projectId.trim() || !target.contextId.trim() || !target.commitId.trim()) {
    return { ok: false, message: "An exact project, Context, and commit target is required / 需要精确的项目、Context 与提交目标。" };
  }
  if (!branchName || !datasetName || !caseName || !suiteName) {
    return { ok: false, message: "Branch, dataset, case, and suite names are required / 需要分支、数据集、用例与套件名称。" };
  }

  const caseInput = parseJson(draft.caseInputText, "Case input must be valid JSON / 用例输入必须是有效 JSON。");
  if (!caseInput.ok) return caseInput;
  let expectedOutput: LocalBenchmarkDefinitionExpectedOutput = { mode: "unspecified" };
  if (draft.expectedOutputText.trim().length > 0) {
    const parsedExpectedOutput = parseJson(
      draft.expectedOutputText,
      "Expected output must be valid JSON / 预期输出必须是有效 JSON。"
    );
    if (!parsedExpectedOutput.ok) return parsedExpectedOutput;
    expectedOutput = { mode: "exact", value: parsedExpectedOutput.value };
  }

  const thresholdValue = Number(draft.thresholdValue);
  if (!draft.thresholdValue.trim() || !Number.isFinite(thresholdValue)) {
    return { ok: false, message: "Threshold must be a finite number / 阈值必须是有限数字。" };
  }

  const bindingId = createId();
  const datasetId = createId();
  const caseId = createId();
  const suiteId = createId();
  return {
    ok: true,
    value: deepFreeze({
      schema_version: 1,
      binding_id: bindingId,
      branch_name: branchName,
      expected_head_commit_id: target.commitId,
      datasets: [{
        id: datasetId,
        name: datasetName,
        cases: [{
          id: caseId,
          name: caseName,
          input: caseInput.value,
          expected_output: expectedOutput
        }]
      }],
      suite: {
        id: suiteId,
        name: suiteName,
        dataset_ids: [datasetId],
        thresholds: [{
          metric: draft.metric,
          direction: draft.direction,
          value: thresholdValue
        }]
      }
    })
  };
}

export function canSubmitLocalBenchmarkDefinition(input: Readonly<{
  enabled: boolean;
  bearerToken: string;
  hasCommit: boolean;
  isValid: boolean;
  isPending: boolean;
}>): boolean {
  return Boolean(
    input.enabled
    && input.bearerToken.trim()
    && input.hasCommit
    && input.isValid
    && !input.isPending
  );
}

export function benchmarkDefinitionAuthoringErrorMessage(
  status: number,
  errorCode?: string,
  retryAfterMs?: number
): string {
  if (status === 401) return "Bearer authentication is required / 需要 Bearer 身份验证。";
  if (status === 403 && errorCode === "local_benchmark_definition_authoring_disabled") {
    return "Benchmark definition authoring is disabled in this development environment / 此开发环境已禁用 Benchmark 定义创作。";
  }
  if (status === 403) return "You do not have permission to author this exact Context target / 你没有为此精确 Context 目标创作的权限。";
  if (status === 409) return "The exact branch head or immutable definition conflicts; reload before retrying / 精确分支 head 或不可变定义发生冲突；请重新加载后重试。";
  if (status === 429) {
    const retry = retryAfterMs === undefined ? "" : ` Retry in ${Math.ceil(retryAfterMs / 1_000)} seconds.`;
    return `The private authoring rate limit is active / 私有创作速率限制已生效。${retry}`;
  }
  return "The private benchmark definition request could not be completed / 私有 Benchmark 定义请求未能完成。";
}

function parseJson(
  value: string,
  message: string
): Readonly<{ ok: true; value: LocalBenchmarkDefinitionJsonValue }> | Readonly<{ ok: false; message: string }> {
  try {
    const parsed: unknown = JSON.parse(value);
    if (isJsonValue(parsed)) {
      return { ok: true, value: parsed };
    }
  } catch {
  }
  return { ok: false, message };
}

function isJsonValue(value: unknown): value is LocalBenchmarkDefinitionJsonValue {
  if (value === null || typeof value === "string" || typeof value === "boolean") return true;
  if (typeof value === "number") return Number.isFinite(value);
  if (Array.isArray(value)) return value.every(isJsonValue);
  return typeof value === "object" && Object.values(value).every(isJsonValue);
}

function createUuid(): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  throw new Error("Secure UUID generation is unavailable");
}

function targetDetail(target: LocalBenchmarkDefinitionAuthoringTarget): string {
  return `${target.projectId || "-"} / ${target.contextId || "-"} @ ${target.commitId || "-"}`;
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
