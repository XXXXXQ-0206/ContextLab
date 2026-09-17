import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import {
  adaptLocalWorkflowExecutionStatusV1,
  type LocalWorkflowExecutionStatusDto,
  type LocalWorkflowExecutionStatusResource
} from "./local-workflow-execution-status-data";

export type LocalWorkflowExecutionStatusViewModel = Readonly<{
  title: string;
  description: string;
  capabilityState: CapabilityStateScreenModel;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  execution: Readonly<{
    runState: string;
    replayOf: string;
    eventCount: number;
    lastEventSequence: number;
    capabilitySnapshotDigest: string;
    nodeStatusCounts: Readonly<{
      pending: number;
      running: number;
      succeeded: number;
      failed: number;
      blocked: number;
    }>;
  }> | null;
}>;

export function presentLocalWorkflowExecutionStatus(
  resource: LocalWorkflowExecutionStatusResource | LocalWorkflowExecutionStatusDto
): LocalWorkflowExecutionStatusViewModel {
  const dto = "id" in resource
    ? resource
    : adaptLocalWorkflowExecutionStatusV1(resource);
  const capabilityState = presentCapabilityState(toCapabilityStateDto(dto));

  return deepFreeze({
    title: "Workflow execution status / 工作流执行状态",
    description: "Private redacted run metadata / 私有脱敏运行元数据。",
    capabilityState,
    scope: [
      { id: "context-id", label: "Context / 上下文", value: dto.target.context_id },
      { id: "context-commit-id", label: "Context commit / 上下文提交", value: dto.target.context_commit_id },
      { id: "binding-id", label: "Binding / 绑定", value: dto.target.binding_id },
      { id: "workflow-id", label: "Workflow / 工作流", value: dto.target.workflow_id },
      { id: "workflow-revision", label: "Workflow revision / 工作流修订", value: String(dto.target.workflow_revision) },
      { id: "run-id", label: "Run / 运行", value: dto.target.run_id }
    ],
    execution: dto.status === undefined
      ? null
      : {
          runState: formatRunState(dto.status.run_state),
          replayOf: dto.status.replay_of === null
            ? "None / 无"
            : `${dto.status.replay_of} / 回放源`,
          eventCount: dto.status.event_count,
          lastEventSequence: dto.status.last_event_sequence,
          capabilitySnapshotDigest: dto.status.capability_snapshot_digest,
          nodeStatusCounts: dto.status.node_status_counts
        }
  });
}

function toCapabilityStateDto(dto: LocalWorkflowExecutionStatusDto): FrozenCapabilityStateDto {
  const summary = dto.state === "available"
    ? bilingual("A redacted workflow execution status is available.", "脱敏的工作流执行状态已可用。")
    : dto.state === "unavailable"
      ? bilingual("The local workflow execution adapter is unavailable.", "本地工作流执行适配器不可用。")
      : dto.state === "error"
        ? bilingual("The protected workflow execution status could not be loaded.", "受保护的工作流执行状态无法加载。")
        : dto.state === "empty"
          ? bilingual("No workflow execution status is available yet.", "暂时没有工作流执行状态。")
          : bilingual("The workflow execution status is being retrieved.", "正在读取工作流执行状态。");

  return Object.freeze({
    id: dto.id,
    capability: dto.capability,
    state: dto.state,
    summary,
  });
}

function formatRunState(value: "pending" | "running" | "succeeded" | "failed"): string {
  const labels = {
    pending: "Pending / 待处理",
    running: "Running / 运行中",
    succeeded: "Succeeded / 已成功",
    failed: "Failed / 已失败"
  } as const;
  return labels[value];
}

function bilingual(en: string, zh: string): BilingualCapabilityText {
  return Object.freeze({ en, zh });
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
