import type { StatusPillTone } from "@contextlab/ui";
import type {
  LocalBenchmarkDecisionDiscovery,
  LocalBenchmarkDecisionDiscoveryStatus
} from "./context-benchmark-decision-discovery-data";
import {
  presentCapabilityState,
  type CapabilityStateScreenModel
} from "./capability-state-presenter";
import type { FrozenCapabilityStateDto } from "./capability-state-data";

export type BenchmarkDecisionDiscoveryResource =
  | Readonly<{
      kind: "loading" | "empty" | "unavailable";
      target: BenchmarkDecisionDiscoveryTarget;
    }>
  | Readonly<{
      kind: "error";
      target: BenchmarkDecisionDiscoveryTarget;
      message: string;
    }>
  | Readonly<{
      kind: "ready";
      target: BenchmarkDecisionDiscoveryTarget;
      summary: LocalBenchmarkDecisionDiscovery;
    }>;

export type BenchmarkDecisionDiscoveryTarget = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
}>;

export type BenchmarkDecisionDiscoveryViewModel = Readonly<{
  title: string;
  description: string;
  status: CapabilityStateScreenModel;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  decisions: ReadonlyArray<BenchmarkDecisionSummaryViewModel>;
  firstDecisionId: string | null;
}>;

export type BenchmarkDecisionSummaryViewModel = Readonly<{
  id: string;
  decisionId: string;
  suiteId: string;
  status: { label: string; tone: StatusPillTone };
  recordedAt: string;
}>;

export function presentLocalBenchmarkDecisionDiscovery(
  resource: BenchmarkDecisionDiscoveryResource
): BenchmarkDecisionDiscoveryViewModel {
  const dto = toCapabilityDto(resource);
  const status = presentCapabilityState(dto);
  const decisions = resource.kind === "ready"
    ? Object.freeze(resource.summary.decisions.map(presentDecisionSummary))
    : Object.freeze([] as BenchmarkDecisionSummaryViewModel[]);

  return Object.freeze({
    title: "Benchmark decision discovery / Benchmark decision 发现",
    description: "Exact commit-scoped redacted summaries / 精确提交范围的脱敏摘要。",
    status,
    scope: Object.freeze([
      Object.freeze({ id: "project-id", label: "Project / 项目", value: resource.target.projectId }),
      Object.freeze({ id: "context-id", label: "Context / 上下文", value: resource.target.contextId }),
      Object.freeze({ id: "commit-id", label: "Commit / 提交", value: resource.target.commitId })
    ]),
    decisions,
    firstDecisionId: decisions[0]?.decisionId ?? null
  });
}

export function benchmarkDecisionDiscoveryErrorMessage(
  status: number,
  retryAfterMs?: number
): string {
  if (status === 401) {
    return "Bearer authentication is required / 需要 Bearer 身份验证。";
  }
  if (status === 403) {
    return "You do not have permission for this Context / 你没有此 Context 的权限。";
  }
  if (status === 404) {
    return "No exact benchmark decision discovery was found / 未找到精确范围内的 benchmark decision 发现结果。";
  }
  if (status === 429) {
    const retry = retryAfterMs ? ` Retry in ${Math.ceil(retryAfterMs / 1_000)} seconds.` : "";
    return `The local rate limit is active / 本地速率限制已生效。${retry}`;
  }
  if (status === 503) {
    return "Benchmark decision discovery is unavailable / Benchmark decision 发现暂不可用。";
  }
  return "The local benchmark decision discovery request could not be completed / 本地 benchmark decision 发现请求未能完成。";
}

function toCapabilityDto(resource: BenchmarkDecisionDiscoveryResource): FrozenCapabilityStateDto {
  const state = resource.kind === "ready"
    ? resource.summary.decisions.length > 0 ? "available" : "empty"
    : resource.kind;

  const summary = state === "available"
    ? bilingual("The first exact decision is selected for inspection.", "已选择精确范围内的首个 decision 进行审阅。")
    : state === "empty"
      ? bilingual("No sealed decisions are available at this commit.", "此提交暂无可用的已封存 decision。")
      : state === "unavailable"
        ? bilingual("Decision discovery is not available in this local environment.", "当前本地环境不可用 decision 发现。")
        : state === "error"
          ? bilingual("The exact decision list could not be loaded.", "无法加载精确范围内的 decision 列表。")
          : bilingual("The exact decision list is being loaded.", "正在加载精确范围内的 decision 列表。");

  return Object.freeze({
    id: "local-benchmark-decision-discovery",
    capability: bilingual("Benchmark decision discovery", "Benchmark decision 发现"),
    state,
    summary,
    ...(resource.kind === "error" ? { detail: bilingual(resource.message, resource.message) } : {})
  }) satisfies FrozenCapabilityStateDto;
}

function presentDecisionSummary(
  decision: LocalBenchmarkDecisionDiscovery["decisions"][number]
): BenchmarkDecisionSummaryViewModel {
  return Object.freeze({
    id: decision.decision_id,
    decisionId: decision.decision_id,
    suiteId: decision.suite_id,
    status: presentDecisionStatus(decision.status),
    recordedAt: decision.recorded_at
  });
}

function presentDecisionStatus(status: LocalBenchmarkDecisionDiscoveryStatus): {
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

function bilingual(en: string, zh: string) {
  return Object.freeze({ en, zh });
}
