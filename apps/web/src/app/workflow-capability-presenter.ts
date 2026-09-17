import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import type {
  FrozenWorkflowCapabilityResource,
  FrozenWorkflowCapabilityResourceFixture
} from "./workflow-capability-data";

const capability: BilingualCapabilityText = Object.freeze({
  en: "Workflow capabilities",
  zh: "工作流能力"
});

export type WorkflowCapabilityResourceModel = Readonly<{
  status: CapabilityStateScreenModel;
  facts: ReadonlyArray<Readonly<{ label: string; value: string }>>;
}>;

export type WorkflowCapabilityScreenModel = Readonly<{
  title: string;
  description: string;
  status: CapabilityStateScreenModel;
  resources: ReadonlyArray<WorkflowCapabilityResourceModel>;
}>;

export function presentWorkflowCapabilityResourceFixture(
  fixture: FrozenWorkflowCapabilityResourceFixture
): WorkflowCapabilityScreenModel {
  const state = fixture.state;
  const status = presentCapabilityState(
    frozenStatusDto("workflow-capability", state, fixture.failure?.message)
  );

  return Object.freeze({
    title: "Workflow capabilities / 工作流能力",
    description: "Local redacted V1 resource fixtures / 本地脱敏 V1 资源固定数据。",
    status,
    resources:
      state === "available" || state === "unavailable"
        ? Object.freeze(fixture.resources.map((resource) => presentResource(resource, state)))
        : Object.freeze([])
  });
}

function presentResource(
  resource: FrozenWorkflowCapabilityResource,
  state: "available" | "unavailable"
): WorkflowCapabilityResourceModel {
  return Object.freeze({
    status: frozenResourceStatus(resource, state),
    facts: Object.freeze([
      Object.freeze({
        label: `${resource.capability.en} resource schema / ${resource.capability.zh}资源模式`,
        value: resource.schemaVersion
      }),
      Object.freeze({
        label: "Boundary / 边界",
        value: `${resource.boundary.en} / ${resource.boundary.zh}`
      })
    ])
  });
}

function frozenResourceStatus(
  resource: FrozenWorkflowCapabilityResource,
  state: "available" | "unavailable"
): CapabilityStateScreenModel {
  return presentCapabilityState(
    frozenStatusDto(resource.id, state, resource.boundary, resource.capability)
  );
}

function frozenStatusDto(
  id: string,
  state: FrozenCapabilityStateDto["state"],
  summary?: BilingualCapabilityText,
  capabilityText: BilingualCapabilityText = capability
): FrozenCapabilityStateDto {
  return Object.freeze({
    id,
    capability: Object.freeze({ ...capabilityText }),
    state,
    ...(summary ? { summary: Object.freeze({ ...summary }) } : {})
  });
}
