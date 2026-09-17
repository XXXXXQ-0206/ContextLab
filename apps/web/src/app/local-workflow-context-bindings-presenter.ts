import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import {
  adaptLocalWorkflowContextBindingsV1,
  type FrozenLocalWorkflowContextBindingsDto,
  type LocalWorkflowContextBindingsResource
} from "./local-workflow-context-bindings-data";

export type LocalWorkflowContextBindingsRowModel = Readonly<{
  id: string;
  bindingId: string;
  workflowId: string;
  workflowRevision: number;
  nodeCount: number;
  edgeCount: number;
}>;

export type LocalWorkflowContextBindingsScreenModel = Readonly<{
  title: string;
  description: string;
  status: CapabilityStateScreenModel;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  rows: ReadonlyArray<LocalWorkflowContextBindingsRowModel>;
}>;

export function presentLocalWorkflowContextBindings(
  resource: LocalWorkflowContextBindingsResource
): LocalWorkflowContextBindingsScreenModel {
  const dto = adaptLocalWorkflowContextBindingsV1(resource);
  const status = Object.freeze(presentCapabilityState(
    Object.freeze({
      id: dto.id,
      capability: dto.capability,
      state: dto.state,
      ...(dto.state === "available"
        ? {
            summary: bilingual(
              "Redacted workflow bindings at this commit.",
              "此提交上的脱敏工作流绑定。"
            )
          }
        : {})
    }) satisfies FrozenCapabilityStateDto
  ));

  return Object.freeze({
    title: "Workflow context bindings / Workflow 上下文绑定",
    description: "Read-only exact commit scope / 只读精确提交范围。",
    status,
    scope: Object.freeze([
      Object.freeze({
        id: "context-id",
        label: "Context / 上下文",
        value: dto.context_id
      }),
      Object.freeze({
        id: "commit-id",
        label: "Commit / 提交",
        value: dto.commit_id
      })
    ]),
    rows: Object.freeze(dto.bindings.map(presentBinding))
  });
}

function presentBinding(
  binding: FrozenLocalWorkflowContextBindingsDto["bindings"][number]
): LocalWorkflowContextBindingsRowModel {
  return Object.freeze({
    id: binding.binding_id,
    bindingId: binding.binding_id,
    workflowId: binding.workflow_id,
    workflowRevision: binding.workflow_revision,
    nodeCount: binding.node_count,
    edgeCount: binding.edge_count
  });
}

function bilingual(en: string, zh: string): BilingualCapabilityText {
  return Object.freeze({ en, zh });
}
