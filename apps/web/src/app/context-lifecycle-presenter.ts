import type {
  ContextComponentKind,
  LocalComponentLifecycleCommitRequest,
  LocalContextLifecycleState,
  LocalContextMetadata,
  LocalJsonValue
} from "@contextlab/local-sdk";
import type { CapabilityStateKind, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import type { LocalLifecycleComponentMetadata } from "./context-lifecycle-data";

export type LifecycleEditorOperation =
  | "initialize"
  | "update_metadata"
  | "create"
  | "update"
  | "update_descriptor"
  | "remove"
  | "add_uses_relationship"
  | "remove_uses_relationship";

export type LifecycleEditorInput = {
  branchName: string;
  expectedHeadCommitId: string;
  message: string;
  operation: LifecycleEditorOperation;
  componentKind: ContextComponentKind;
  componentId?: string;
  targetComponentId?: string;
  name: string;
  metadataText: string;
  content: string;
  removeConfirmed?: boolean;
};

export type LifecycleCommandBuildResult =
  | { ok: true; value: LocalComponentLifecycleCommitRequest }
  | { ok: false; message: string };

export type LocalLifecycleMetadataDraft = Readonly<{
  name: string;
  metadataText: string;
}>;

export function presentLocalLifecycleMetadataDraft(
  metadata: LocalLifecycleComponentMetadata
): LocalLifecycleMetadataDraft {
  return {
    name: metadata.name,
    metadataText: JSON.stringify(metadata.metadata, null, 2)
  };
}

export type ContextLifecycleReadTarget = Readonly<{
  contextId: string;
  commitId: string;
}>;

export type ContextLifecycleReadResource =
  | Readonly<{ kind: "loading"; target: ContextLifecycleReadTarget }>
  | Readonly<{ kind: "error"; target: ContextLifecycleReadTarget; message?: string }>
  | Readonly<{ kind: "empty"; target: ContextLifecycleReadTarget }>
  | Readonly<{ kind: "unavailable"; target: ContextLifecycleReadTarget; message?: string }>
  | Readonly<{
      kind: "available";
      target: ContextLifecycleReadTarget;
      state: LocalContextLifecycleState;
    }>;

export type ContextLifecycleReadComponentViewModel = Readonly<{
  id: string;
  componentId: string;
  kindLabel: string;
  name: string;
  content: string;
  metadataText: string;
  contentHash: string;
  creationCommitId: string;
  contentCommitId: string;
}>;

export type ContextLifecycleReadInspectorViewModel = Readonly<{
  title: string;
  description: string;
  status: CapabilityStateScreenModel;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  metadataText: string | null;
  components: ReadonlyArray<ContextLifecycleReadComponentViewModel>;
  relationships: ContextLifecycleRelationshipSectionModel | null;
}>;

export function presentContextLifecycleReadInspector(
  resource: ContextLifecycleReadResource
): ContextLifecycleReadInspectorViewModel {
  const state = resource.kind as CapabilityStateKind;
  const statusDto: FrozenCapabilityStateDto = Object.freeze({
    id: "context-lifecycle-read",
    capability: Object.freeze({
      en: "Context lifecycle state",
      zh: "Context 生命周期状态"
    }),
    state,
    ...(resource.kind === "available"
      ? {
          summary: Object.freeze({
            en: "Exact selected commit lifecycle state is available.",
            zh: "已获取选定精确提交的生命周期状态。"
          }),
          detail: Object.freeze({
            en: `Schema ${resource.state.schema_version}`,
            zh: `模式 ${resource.state.schema_version}`
          })
        }
      : {})
  });
  const status = presentCapabilityState(statusDto);

  return Object.freeze({
    title: "Context lifecycle state / Context 生命周期状态",
    description: "Protected local read-only inspection / 受保护的本地只读检查。",
    status,
    scope: Object.freeze([
      Object.freeze({
        id: "context-id",
        label: "Context / 上下文",
        value: resource.target.contextId
      }),
      Object.freeze({
        id: "commit-id",
        label: "Commit / 提交",
        value: resource.target.commitId
      })
    ]),
    metadataText: resource.kind === "available" && resource.state.metadata !== null
      ? JSON.stringify(resource.state.metadata, null, 2)
      : null,
    components: Object.freeze(resource.kind === "available"
      ? resource.state.components.map((component) => Object.freeze({
          id: component.component_id,
          componentId: component.component_id,
          kindLabel: `${component.component_kind} / ${component.component_kind}`,
          name: component.name,
          content: component.content,
          metadataText: JSON.stringify(component.metadata, null, 2),
          contentHash: component.content_hash,
          creationCommitId: component.creation_commit_id,
          contentCommitId: component.content_commit_id
        }))
      : []),
    relationships: resource.kind === "available"
      ? presentContextLifecycleRelationships(resource.state)
      : null
  });
}

export type LocalLifecycleSubmissionState = {
  bearerToken: string;
  selectedCommitId: string;
  allowsUnbornHead?: boolean;
  isCommandValid: boolean;
  isLoading: boolean;
  isSubmitting: boolean;
};

export function canSubmitLocalLifecycle(state: LocalLifecycleSubmissionState): boolean {
  return Boolean(
    state.bearerToken.trim()
      && state.isCommandValid
      && !state.isLoading
      && !state.isSubmitting
      && (state.allowsUnbornHead || state.selectedCommitId)
  );
}

export function buildLocalLifecycleCommitRequest(
  input: LifecycleEditorInput
): LifecycleCommandBuildResult {
  const branchName = input.branchName.trim();
  const expectedHeadCommitId = input.expectedHeadCommitId.trim();
  const message = input.message.trim();

  if (!branchName || !message) {
    return {
      ok: false,
      message: "Branch and commit message are required / 需要分支与提交说明。"
    };
  }

  if (input.operation === "initialize") {
    return {
      ok: true,
      value: {
        branch_name: branchName,
        expected_head_commit_id: null,
        message,
        operation: { kind: "initialize" }
      }
    };
  }

  if (!expectedHeadCommitId) {
    return {
      ok: false,
      message: "A materialized head is required outside initialization / 初始化之外需要已物化 head。"
    };
  }

  if (input.operation === "update_metadata") {
    const metadata = parseContextMetadata(input.metadataText);
    if (!metadata.ok) {
      return metadata;
    }

    return {
      ok: true,
      value: {
        branch_name: branchName,
        expected_head_commit_id: expectedHeadCommitId,
        message,
        operation: { kind: "update_metadata", metadata: metadata.value }
      }
    };
  }

  if (input.operation === "create") {
    const name = input.name.trim();
    const content = input.content;
    if (!name || !content) {
      return {
        ok: false,
        message: "Name and content are required for creation / 创建需要名称与正文。"
      };
    }

    const metadata = parseJsonMetadata(input.metadataText);
    if (!metadata.ok) {
      return metadata;
    }

    return {
      ok: true,
      value: {
        branch_name: branchName,
        expected_head_commit_id: expectedHeadCommitId,
        message,
        operation: {
          kind: "create",
          component_kind: input.componentKind,
          name,
          metadata: metadata.value,
          content
        }
      }
    };
  }

  if (input.operation === "update") {
    const componentId = input.componentId?.trim() ?? "";
    const content = input.content;
    if (!componentId || !content) {
      return {
        ok: false,
        message: "Component ID and content are required for an update / 更新需要组件 ID 与正文。"
      };
    }

    return {
      ok: true,
      value: {
        branch_name: branchName,
        expected_head_commit_id: expectedHeadCommitId,
        message,
        operation: { kind: "update", component_id: componentId, content }
      }
    };
  }

  if (input.operation === "update_descriptor") {
    const componentId = input.componentId?.trim() ?? "";
    const name = input.name.trim();
    if (!componentId || !name) {
      return {
        ok: false,
        message: "Component ID and name are required for a descriptor update / 描述符更新需要组件 ID 与名称。"
      };
    }

    const metadata = parseJsonMetadata(input.metadataText);
    if (!metadata.ok) {
      return metadata;
    }

    return {
      ok: true,
      value: {
        branch_name: branchName,
        expected_head_commit_id: expectedHeadCommitId,
        message,
        operation: {
          kind: "update_descriptor",
          component_id: componentId,
          name,
          metadata: metadata.value
        }
      }
    };
  }

  if (input.operation === "add_uses_relationship" || input.operation === "remove_uses_relationship") {
    const sourceComponentId = input.componentId?.trim() ?? "";
    const targetComponentId = input.targetComponentId?.trim() ?? "";
    if (!sourceComponentId || !targetComponentId) {
      return {
        ok: false,
        message: "Source and target component IDs are required / 需要源与目标组件 ID。"
      };
    }
    if (sourceComponentId === targetComponentId) {
      return {
        ok: false,
        message: "Uses endpoints must differ / Uses 关系端点必须不同。"
      };
    }

    return {
      ok: true,
      value: {
        branch_name: branchName,
        expected_head_commit_id: expectedHeadCommitId,
        message,
        operation: {
          kind: input.operation,
          source_component_id: sourceComponentId,
          target_component_id: targetComponentId
        }
      }
    };
  }

  const componentId = input.componentId?.trim() ?? "";
  if (!componentId) {
    return {
      ok: false,
      message: "Component ID is required / 需要组件 ID。"
    };
  }

  if (!input.removeConfirmed) {
    return {
      ok: false,
      message: "Confirm component removal before committing / 提交前请确认移除组件。"
    };
  }

  return {
    ok: true,
    value: {
      branch_name: branchName,
      expected_head_commit_id: expectedHeadCommitId,
      message,
      operation: { kind: "remove", component_id: componentId }
    }
  };
}

export function lifecycleErrorMessage(status: number, retryAfterMs?: number, errorCode?: string): string {
  if (status === 401) {
    return "Bearer authentication is required / 需要 Bearer 身份验证。";
  }
  if (status === 403 && errorCode === "local_lifecycle_disabled") {
    return "Local Context lifecycle changes are disabled in this development environment. Enable the local lifecycle development gate and retry / 此开发环境已禁用本地 Context 生命周期变更。请启用本地生命周期开发开关后重试。";
  }
  if (status === 403) {
    return "You do not have permission for this Context / 你没有此 Context 的权限。";
  }
  if (status === 409) {
    return "The selected branch head is stale; reload and review before retrying / 选定分支 head 已过期，请重新加载并审阅后重试。";
  }
  if (status === 429) {
    const retry = retryAfterMs ? ` Retry in ${Math.ceil(retryAfterMs / 1_000)} seconds.` : "";
    return `The local rate limit is active / 本地速率限制已生效。${retry}`;
  }
  return "The local lifecycle request could not be completed / 本地生命周期请求未能完成。";
}

export type LifecycleNoticeTone = "success" | "error";

export function lifecycleNoticeRole(tone: LifecycleNoticeTone): "status" | "alert" {
  return tone === "error" ? "alert" : "status";
}

export type LifecycleUsesEdge = Readonly<{
  sourceComponentId: string;
  targetComponentId: string;
}>;

export type ContextLifecycleRelationshipRow = Readonly<{
  id: string;
  kindLabel: string;
  sourceLabel: string;
  sourceId: string;
  targetLabel: string;
  targetId: string;
}>;

export type ContextLifecycleRelationshipSectionModel = Readonly<{
  commitId: string;
  rows: ReadonlyArray<ContextLifecycleRelationshipRow>;
}>;

export type LifecycleGraphEdgeKind =
  | "owns"
  | "contains"
  | "configures"
  | "retrieves"
  | "uses"
  | "evaluates"
  | "produces"
  | "tracks";

export type LifecycleGraphRelationshipFact = Readonly<{
  source: string;
  target: string;
  kind: LifecycleGraphEdgeKind;
}>;

const knownGraphEdgeKinds: readonly LifecycleGraphEdgeKind[] = [
  "owns",
  "contains",
  "configures",
  "retrieves",
  "uses",
  "evaluates",
  "produces",
  "tracks"
];

const knownGraphEdgeKindSet = new Set<LifecycleGraphEdgeKind>(knownGraphEdgeKinds);

const relationshipKindLabels: Record<LifecycleGraphEdgeKind, string> = {
  owns: "Owns / 拥有",
  contains: "Contains / 包含",
  configures: "Configures / 配置",
  retrieves: "Retrieves / 检索",
  uses: "Uses / 使用",
  evaluates: "Evaluates / 评估",
  produces: "Produces / 产出",
  tracks: "Tracks / 跟踪"
};

export function presentContextLifecycleRelationships(
  lifecycleState: LocalContextLifecycleState | null
): ContextLifecycleRelationshipSectionModel | null {
  if (!lifecycleState) {
    return null;
  }

  const nodes = lifecycleState.graph_snapshot.graph.nodes;
  const rows = lifecycleState.graph_snapshot.graph.edges
    .map((edge, index) => {
      const source = nodes[edge.source];
      const target = nodes[edge.target];
      return {
        id: `${edge.kind}:${edge.source}:${edge.target}:${index}`,
        kindLabel: relationshipKindLabels[edge.kind as LifecycleGraphEdgeKind]
          ?? `${edge.kind} / ${edge.kind}`,
        sourceLabel: source
          ? `${source.label} (${source.kind})`
          : `Unknown node / 未知节点 (${edge.source})`,
        sourceId: edge.source,
        targetLabel: target
          ? `${target.label} (${target.kind})`
          : `Unknown node / 未知节点 (${edge.target})`,
        targetId: edge.target,
        kindRank: knownGraphEdgeKinds.indexOf(edge.kind as LifecycleGraphEdgeKind),
        sourceSort: edge.source,
        targetSort: edge.target
      };
    })
    .sort((left, right) =>
      compareStrings(left.sourceSort, right.sourceSort)
      || compareStrings(left.targetSort, right.targetSort)
      || (left.kindRank < 0 ? knownGraphEdgeKinds.length : left.kindRank)
        - (right.kindRank < 0 ? knownGraphEdgeKinds.length : right.kindRank)
      || compareStrings(left.id, right.id)
    )
    .map(({ id, kindLabel, sourceLabel, sourceId, targetLabel, targetId }) => ({
      id,
      kindLabel,
      sourceLabel,
      sourceId,
      targetLabel,
      targetId
    }));

  return Object.freeze({
    commitId: lifecycleState.commit_id,
    rows: Object.freeze(rows)
  });
}

export function deriveLocalGraphRelationshipFacts(state: unknown): LifecycleGraphRelationshipFact[] {
  return readLocalGraphRelationshipFacts(state)?.relationships ?? [];
}

export function deriveLocalUsesEdges(state: unknown): LifecycleUsesEdge[] {
  return readLocalUsesGraphFacts(state)?.usesEdges ?? [];
}

export function deriveLocalUsesAddCandidates(state: unknown, sourceComponentId: string): string[] {
  const facts = readLocalUsesGraphFacts(state);
  if (!facts || !facts.componentIds.includes(sourceComponentId)) {
    return [];
  }

  const existingTargets = new Set(
    facts.usesEdges
      .filter((edge) => edge.sourceComponentId === sourceComponentId)
      .map((edge) => edge.targetComponentId)
  );

  return facts.componentIds.filter(
    (componentId) => componentId !== sourceComponentId && !existingTargets.has(componentId)
  );
}

export function deriveLocalUsesRemoveCandidates(state: unknown): LifecycleUsesEdge[] {
  return deriveLocalUsesEdges(state);
}

type LocalGraphRelationshipFacts = {
  relationships: LifecycleGraphRelationshipFact[];
};

type LocalUsesGraphFacts = {
  componentIds: string[];
  usesEdges: LifecycleUsesEdge[];
};

function readLocalGraphRelationshipFacts(state: unknown): LocalGraphRelationshipFacts | null {
  const stateRecord = asRecord(state);
  const contextId = asNonEmptyString(stateRecord?.context_id);
  const commitId = asNonEmptyString(stateRecord?.commit_id);
  const graphSnapshot = asRecord(stateRecord?.graph_snapshot);
  const snapshotContextId = asNonEmptyString(graphSnapshot?.context_id);
  const snapshotCommitId = asNonEmptyString(graphSnapshot?.commit_id);
  const graph = asRecord(graphSnapshot?.graph);
  const nodes = asRecord(graph?.nodes);
  const edges = graph?.edges;

  if (
    !contextId
    || !commitId
    || !graphSnapshot
    || !snapshotContextId
    || !snapshotCommitId
    || snapshotContextId !== contextId
    || snapshotCommitId !== commitId
    || !graph
    || !nodes
    || !Array.isArray(edges)
  ) {
    return null;
  }

  const relationships: LifecycleGraphRelationshipFact[] = [];
  for (const edge of edges) {
    const edgeRecord = asRecord(edge);
    const source = asNonEmptyString(edgeRecord?.source);
    const target = asNonEmptyString(edgeRecord?.target);
    const kind = asNonEmptyString(edgeRecord?.kind);
    if (
      !source
      || !target
      || !kind
      || !knownGraphEdgeKindSet.has(kind as LifecycleGraphEdgeKind)
      || !hasOwn(nodes, source)
      || !hasOwn(nodes, target)
      || source === target
    ) {
      return null;
    }

    relationships.push({
      source,
      target,
      kind: kind as LifecycleGraphEdgeKind
    });
  }

  relationships.sort(compareGraphRelationshipFacts);
  return { relationships };
}

function readLocalUsesGraphFacts(state: unknown): LocalUsesGraphFacts | null {
  const stateRecord = asRecord(state);
  const components = stateRecord?.components;
  const graphSnapshot = asRecord(stateRecord?.graph_snapshot);
  const graph = asRecord(graphSnapshot?.graph);
  const nodes = asRecord(graph?.nodes);
  const edges = graph?.edges;

  if (!Array.isArray(components) || !graphSnapshot || !graph || !nodes || !Array.isArray(edges)) {
    return null;
  }

  const componentIds: string[] = [];
  for (const component of components) {
    const componentId = asNonEmptyString(asRecord(component)?.component_id);
    if (!componentId || componentIds.includes(componentId)) {
      return null;
    }
    componentIds.push(componentId);
  }

  for (const componentId of componentIds) {
    if (!hasOwn(nodes, componentNodeId(componentId))) {
      return null;
    }
  }

  const usesEdges: LifecycleUsesEdge[] = [];
  const seenEdges = new Set<string>();
  for (const edge of edges) {
    const edgeRecord = asRecord(edge);
    const source = asNonEmptyString(edgeRecord?.source);
    const target = asNonEmptyString(edgeRecord?.target);
    const kind = asNonEmptyString(edgeRecord?.kind);
    if (!source || !target || !kind || !knownGraphEdgeKindSet.has(kind as LifecycleGraphEdgeKind) || !hasOwn(nodes, source) || !hasOwn(nodes, target)) {
      return null;
    }

    if (kind !== "uses") {
      continue;
    }

    const sourceComponentId = componentIdFromNodeId(source);
    const targetComponentId = componentIdFromNodeId(target);
    if (
      !sourceComponentId
      || !targetComponentId
      || !componentIds.includes(sourceComponentId)
      || !componentIds.includes(targetComponentId)
      || sourceComponentId === targetComponentId
    ) {
      return null;
    }

    const edgeKey = `${sourceComponentId}\u0000${targetComponentId}`;
    if (seenEdges.has(edgeKey)) {
      return null;
    }
    seenEdges.add(edgeKey);
    usesEdges.push({ sourceComponentId, targetComponentId });
  }

  usesEdges.sort(compareUsesEdges);
  componentIds.sort(compareStrings);
  return { componentIds, usesEdges };
}

function compareUsesEdges(left: LifecycleUsesEdge, right: LifecycleUsesEdge): number {
  return compareStrings(left.sourceComponentId, right.sourceComponentId)
    || compareStrings(left.targetComponentId, right.targetComponentId);
}

function compareGraphRelationshipFacts(
  left: LifecycleGraphRelationshipFact,
  right: LifecycleGraphRelationshipFact
): number {
  return compareStrings(left.source, right.source)
    || compareStrings(left.target, right.target)
    || knownGraphEdgeKinds.indexOf(left.kind) - knownGraphEdgeKinds.indexOf(right.kind);
}

function compareStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function asNonEmptyString(value: unknown): string | null {
  return typeof value === "string" && value.trim().length > 0 ? value : null;
}

function hasOwn(record: Record<string, unknown>, key: string): boolean {
  return Object.prototype.hasOwnProperty.call(record, key);
}

function componentNodeId(componentId: string): string {
  return `component:${componentId}`;
}

function componentIdFromNodeId(nodeId: string): string | null {
  const prefix = "component:";
  if (!nodeId.startsWith(prefix)) {
    return null;
  }

  return asNonEmptyString(nodeId.slice(prefix.length));
}

function parseJsonMetadata(value: string): { ok: true; value: LocalJsonValue } | { ok: false; message: string } {
  try {
    const parsed: unknown = JSON.parse(value.trim() || "null");
    if (isLocalJsonValue(parsed)) {
      return { ok: true, value: parsed };
    }
  } catch {
  }

  return {
    ok: false,
    message: "Metadata must be valid JSON / 元数据必须是有效 JSON。"
  };
}

function parseContextMetadata(
  value: string
): { ok: true; value: LocalContextMetadata } | { ok: false; message: string } {
  const parsed = parseJsonMetadata(value);
  if (!parsed.ok || !asRecord(parsed.value)) {
    return {
      ok: false,
      message: "Context metadata must be a JSON object / Context 元数据必须是 JSON 对象。"
    };
  }

  const record = asRecord(parsed.value);
  if (!record || !hasExactKeys(record, ["created_at", "updated_at", "labels"])) {
    return {
      ok: false,
      message: "Context metadata requires created_at, updated_at, and labels / Context 元数据需要 created_at、updated_at 与 labels。"
    };
  }

  const labels = asRecord(record.labels);
  if (
    typeof record.created_at !== "string"
    || typeof record.updated_at !== "string"
    || !labels
    || !Object.values(labels).every((label) => typeof label === "string")
  ) {
    return {
      ok: false,
      message: "Context metadata has an invalid shape / Context 元数据结构无效。"
    };
  }

  return {
    ok: true,
    value: {
      created_at: record.created_at,
      updated_at: record.updated_at,
      labels: Object.fromEntries(Object.entries(labels).map(([key, label]) => [key, label as string]))
    }
  };
}

function hasExactKeys(record: Record<string, unknown>, keys: string[]): boolean {
  const actual = Object.keys(record).sort();
  return actual.length === keys.length && actual.every((key, index) => key === [...keys].sort()[index]);
}

function isLocalJsonValue(value: unknown): value is LocalJsonValue {
  if (value === null || typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return true;
  }

  if (Array.isArray(value)) {
    return value.every(isLocalJsonValue);
  }

  return typeof value === "object" && Object.values(value).every(isLocalJsonValue);
}
