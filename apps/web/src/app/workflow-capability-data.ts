import type { BilingualCapabilityText, CapabilityStateKind } from "./capability-state-data";

export const WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1 = "contextlab.workflow-capability-resource.v1";

export type WorkflowCapabilityResourceId = "workflow-definition" | "workflow-execution";

export type FrozenWorkflowCapabilityResource = Readonly<{
  id: WorkflowCapabilityResourceId;
  schemaVersion: typeof WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1;
  capability: BilingualCapabilityText;
  boundary: BilingualCapabilityText;
}>;

export type FrozenWorkflowCapabilityFailure = Readonly<{
  code: "workflow_capability_fixture_unavailable";
  message: BilingualCapabilityText;
}>;

export type FrozenWorkflowCapabilityResourceFixture = Readonly<{
  schemaVersion: typeof WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1;
  state: CapabilityStateKind;
  resources: ReadonlyArray<FrozenWorkflowCapabilityResource>;
  failure?: FrozenWorkflowCapabilityFailure;
}>;

const workflowDefinition = Object.freeze({
  id: "workflow-definition" as const,
  schemaVersion: WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
  capability: Object.freeze({ en: "Workflow definition", zh: "工作流定义" }),
  boundary: Object.freeze({
    en: "Metadata only; source, variables, and tool configuration are excluded.",
    zh: "仅展示元数据；不包含源内容、变量和工具配置。"
  })
});

const workflowExecution = Object.freeze({
  id: "workflow-execution" as const,
  schemaVersion: WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
  capability: Object.freeze({ en: "Workflow execution", zh: "工作流执行" }),
  boundary: Object.freeze({
    en: "Readiness metadata only; runs, credentials, inputs, and outputs are excluded.",
    zh: "仅展示就绪元数据；不包含运行记录、凭据、输入和输出。"
  })
});

const workflowResources = Object.freeze([workflowDefinition, workflowExecution]);
const noWorkflowResources = Object.freeze([]) as ReadonlyArray<FrozenWorkflowCapabilityResource>;
const unavailableFailure = Object.freeze({
  code: "workflow_capability_fixture_unavailable" as const,
  message: Object.freeze({
    en: "Workflow capability metadata cannot be safely displayed.",
    zh: "无法安全显示工作流能力元数据。"
  })
});

export const WORKFLOW_CAPABILITY_RESOURCE_FIXTURES = Object.freeze({
  loading: Object.freeze({
    schemaVersion: WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
    state: "loading" as const,
    resources: noWorkflowResources
  }),
  error: Object.freeze({
    schemaVersion: WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
    state: "error" as const,
    resources: noWorkflowResources,
    failure: unavailableFailure
  }),
  empty: Object.freeze({
    schemaVersion: WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
    state: "empty" as const,
    resources: noWorkflowResources
  }),
  available: Object.freeze({
    schemaVersion: WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
    state: "available" as const,
    resources: workflowResources
  }),
  unavailable: Object.freeze({
    schemaVersion: WORKFLOW_CAPABILITY_RESOURCE_SCHEMA_V1,
    state: "unavailable" as const,
    resources: workflowResources
  })
}) as Readonly<Record<CapabilityStateKind, FrozenWorkflowCapabilityResourceFixture>>;
