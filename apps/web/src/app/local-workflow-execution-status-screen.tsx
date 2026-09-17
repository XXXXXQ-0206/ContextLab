import { CapabilityState, CodeChip, DefinitionGrid, StatusPill } from "@contextlab/ui";
import React from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import type {
  LocalWorkflowExecutionStatusDto,
  LocalWorkflowExecutionStatusResource
} from "./local-workflow-execution-status-data";
import {
  presentLocalWorkflowExecutionStatus,
  type LocalWorkflowExecutionStatusViewModel
} from "./local-workflow-execution-status-presenter";

export type LocalWorkflowExecutionStatusScreenProps = Readonly<{
  resource?: LocalWorkflowExecutionStatusResource | LocalWorkflowExecutionStatusDto;
  view?: LocalWorkflowExecutionStatusViewModel;
}>;

export function LocalWorkflowExecutionStatusScreen({
  resource,
  view: providedView
}: LocalWorkflowExecutionStatusScreenProps) {
  const view = providedView ?? (resource === undefined
    ? undefined
    : presentLocalWorkflowExecutionStatus(resource));

  if (view === undefined) {
    return null;
  }

  return (
    <section
      aria-labelledby="local-workflow-execution-status-heading"
      className="operation-block local-workflow-execution-status"
      data-state={view.capabilityState.state}
      id="local-workflow-execution-status"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Workflow operations / 工作流操作</span>
          <h2 id="local-workflow-execution-status-heading">{view.title}</h2>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="info">read-only / 只读</StatusPill>
      </div>

      <CapabilityStateScreen view={view.capabilityState} />

      <DefinitionGrid
        aria-label="Exact workflow execution scope / 精确工作流执行范围"
        columns={3}
        compact
        items={view.scope.map((item) => ({
          ...item,
          value: <CodeChip>{item.value}</CodeChip>
        }))}
        surface="raised"
        valueTone="info"
      />

      {view.execution ? (
        <>
          <DefinitionGrid
            aria-label="Redacted workflow execution metadata / 脱敏工作流执行元数据"
            columns={3}
            compact
            items={[
              { id: "run-state", label: "Run state / 运行状态", value: view.execution.runState },
              { id: "replay-of", label: "Replay source / 回放源", value: <CodeChip>{view.execution.replayOf}</CodeChip> },
              { id: "event-count", label: "Event count / 事件数", value: view.execution.eventCount },
              { id: "last-event-sequence", label: "Last event / 最后事件", value: view.execution.lastEventSequence },
              { id: "snapshot-digest", label: "Capability snapshot / 能力快照", value: <CodeChip>{view.execution.capabilitySnapshotDigest}</CodeChip> }
            ]}
            surface="raised"
          />
          <DefinitionGrid
            aria-label="Redacted node status counts / 脱敏节点状态计数"
            columns={4}
            compact
            items={[
              { id: "pending", label: "Pending / 待处理", value: view.execution.nodeStatusCounts.pending },
              { id: "running", label: "Running / 运行中", value: view.execution.nodeStatusCounts.running },
              { id: "succeeded", label: "Succeeded / 已成功", value: view.execution.nodeStatusCounts.succeeded },
              { id: "failed", label: "Failed / 已失败", value: view.execution.nodeStatusCounts.failed },
              { id: "blocked", label: "Blocked / 已阻塞", value: view.execution.nodeStatusCounts.blocked }
            ]}
            surface="raised"
          />
        </>
      ) : null}
    </section>
  );
}

export function LocalWorkflowExecutionStatusCapabilityState({
  view
}: Readonly<{ view: LocalWorkflowExecutionStatusViewModel }>) {
  return (
    <CapabilityState
      ariaLabel={view.capabilityState.ariaLabel}
      description={view.capabilityState.description}
      detail={view.capabilityState.detail}
      id={view.capabilityState.id}
      label={view.capabilityState.capabilityLabel}
      state={view.capabilityState.state}
      stateLabel={view.capabilityState.stateLabel}
    />
  );
}
