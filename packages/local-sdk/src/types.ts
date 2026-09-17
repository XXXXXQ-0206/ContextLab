export type ContextComponentKind =
  | "prompt"
  | "system_prompt"
  | "memory"
  | "knowledge"
  | "retrieval"
  | "embedding"
  | "model_configuration"
  | "tool"
  | "mcp_server"
  | "variable"
  | "output_schema"
  | "workflow"
  | "conversation"
  | "evaluation";

export type LocalJsonPrimitive = string | number | boolean | null;

export type LocalJsonValue =
  | LocalJsonPrimitive
  | LocalJsonValue[]
  | { [key: string]: LocalJsonValue };

export const LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1 =
  "contextlab.local-context-lifecycle-state.v1" as const;
export const LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1 =
  "contextlab.local-component-lifecycle-commit.v1" as const;
export const LOCAL_COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1 = 1 as const;

export type LocalContextMetadata = Readonly<{
  created_at: string;
  updated_at: string;
  labels: Readonly<Record<string, string>>;
}>;

export type LocalLifecycleCreateOperation = {
  kind: "create";
  component_kind: ContextComponentKind;
  name: string;
  metadata: LocalJsonValue;
  content: string;
};

export type LocalLifecycleInitializeOperation = {
  kind: "initialize";
};

export type LocalLifecycleUpdateMetadataOperation = {
  kind: "update_metadata";
  metadata: LocalContextMetadata;
};

export type LocalLifecycleUpdateOperation = {
  kind: "update";
  component_id: string;
  content: string;
};

export type LocalLifecycleUpdateDescriptorOperation = {
  kind: "update_descriptor";
  component_id: string;
  name: string;
  metadata: LocalJsonValue;
};

export type LocalLifecycleRemoveOperation = {
  kind: "remove";
  component_id: string;
};

export type LocalLifecycleAddUsesRelationshipOperation = {
  kind: "add_uses_relationship";
  source_component_id: string;
  target_component_id: string;
};

export type LocalLifecycleRemoveUsesRelationshipOperation = {
  kind: "remove_uses_relationship";
  source_component_id: string;
  target_component_id: string;
};

export type LocalLifecycleOperation =
  | LocalLifecycleInitializeOperation
  | LocalLifecycleUpdateMetadataOperation
  | LocalLifecycleCreateOperation
  | LocalLifecycleUpdateOperation
  | LocalLifecycleUpdateDescriptorOperation
  | LocalLifecycleRemoveOperation
  | LocalLifecycleAddUsesRelationshipOperation
  | LocalLifecycleRemoveUsesRelationshipOperation;

export type LocalComponentLifecycleCommitRequest = {
  branch_name: string;
  expected_head_commit_id: string | null;
  message: string;
  operation: LocalLifecycleOperation;
};

export type LocalLifecycleReadCredentials = {
  bearerToken: string;
};

export type LocalLifecycleWriteCredentials = LocalLifecycleReadCredentials & {
  idempotencyKey: string;
};

export type LocalWorkflowCapabilityAvailability = "available" | "unavailable";

export type LocalWorkflowCapabilityStatus = {
  schema_version: "contextlab.local-workflow-capability-status.v1";
  capability: "workflow";
  enabled: boolean;
  availability: LocalWorkflowCapabilityAvailability;
  reason: string;
};

export type LocalWorkflowContextBinding = {
  binding_id: string;
  workflow_id: string;
  workflow_revision: number;
  node_count: number;
  edge_count: number;
};

export type LocalWorkflowContextBindings = {
  schema_version: "contextlab.local-workflow-context-bindings.v1";
  context_id: string;
  commit_id: string;
  bindings: LocalWorkflowContextBinding[];
};

export type LocalGraphNode = Readonly<{
  id: string;
  kind: string;
  label: string;
}>;

export type LocalGraphEdge = Readonly<{
  source: string;
  target: string;
  kind: string;
}>;

export type LocalContextGraph = Readonly<{
  nodes: Readonly<Record<string, LocalGraphNode>>;
  edges: ReadonlyArray<LocalGraphEdge>;
}>;

export type LocalCommitGraphSnapshot = Readonly<{
  project_id: string;
  context_id: string;
  commit_id: string;
  graph: LocalContextGraph;
  captured_at: string;
  schema_version: typeof LOCAL_COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1;
}>;

export type LocalContextLifecycleComponent = Readonly<{
  component_id: string;
  component_kind: ContextComponentKind;
  name: string;
  metadata: LocalJsonValue;
  content: string;
  content_hash: string;
  creation_commit_id: string;
  content_commit_id: string;
}>;

export type LocalContextLifecycleState = Readonly<{
  schema_version: typeof LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1;
  context_id: string;
  commit_id: string;
  metadata: LocalContextMetadata | null;
  components: ReadonlyArray<LocalContextLifecycleComponent>;
  graph_snapshot: LocalCommitGraphSnapshot;
}>;

export type LocalComponentLifecycleCommitResponse = Readonly<{
  schema_version: typeof LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1;
  disposition: "created" | "replayed";
  commit_id: string;
  snapshot: LocalCommitGraphSnapshot;
}>;

export function parseLocalComponentLifecycleCommitRequest(
  value: unknown
): LocalComponentLifecycleCommitRequest {
  const record = asRecord(value, "local component lifecycle commit request");
  assertExactKeys(record, ["branch_name", "expected_head_commit_id", "message", "operation"]);
  const operation = parseLocalLifecycleOperation(record.operation);
  const expectedHeadCommitId =
    operation.kind === "initialize"
      ? parseUnbornExpectedHead(record.expected_head_commit_id)
      : asString(record.expected_head_commit_id, "expected_head_commit_id");

  return {
    branch_name: asString(record.branch_name, "branch_name"),
    expected_head_commit_id: expectedHeadCommitId,
    message: asString(record.message, "message"),
    operation
  };
}

export function parseLocalContextLifecycleState(
  value: unknown
): LocalContextLifecycleState {
  const record = asRecord(value, "local Context lifecycle state");
  assertExactKeys(record, [
    "schema_version",
    "context_id",
    "commit_id",
    "metadata",
    "components",
    "graph_snapshot"
  ]);
  if (record.schema_version !== LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1) {
    throw new TypeError(
      `schema_version must be ${LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1}`
    );
  }

  const contextId = asUuid(record.context_id, "context_id");
  const commitId = asUuid(record.commit_id, "commit_id");
  const components = asArray(record.components, "components").map(parseLocalContextLifecycleComponent);
  assertLocalLifecycleComponentOrder(components);
  const graphSnapshot = parseLocalCommitGraphSnapshot(
    record.graph_snapshot,
    contextId,
    commitId,
    "graph_snapshot"
  );

  return freeze({
    schema_version: LOCAL_CONTEXT_LIFECYCLE_STATE_SCHEMA_V1,
    context_id: contextId,
    commit_id: commitId,
    metadata: record.metadata === null ? null : parseLocalContextMetadata(record.metadata),
    components,
    graph_snapshot: graphSnapshot
  });
}

export function parseLocalComponentLifecycleCommitResponse(
  value: unknown
): LocalComponentLifecycleCommitResponse {
  const record = asRecord(value, "local component lifecycle commit response");
  assertExactKeys(record, ["schema_version", "disposition", "commit_id", "snapshot"]);
  if (record.schema_version !== LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1) {
    throw new TypeError(
      `schema_version must be ${LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1}`
    );
  }
  if (record.disposition !== "created" && record.disposition !== "replayed") {
    throw new TypeError("disposition is invalid");
  }

  const commitId = asUuid(record.commit_id, "commit_id");
  const snapshot = parseLocalCommitGraphSnapshot(
    record.snapshot,
    undefined,
    commitId,
    "snapshot"
  );

  return freeze({
    schema_version: LOCAL_COMPONENT_LIFECYCLE_COMMIT_RESPONSE_SCHEMA_V1,
    disposition: record.disposition,
    commit_id: commitId,
    snapshot
  });
}

export const parseLocalContextLifecycleStateV1 = parseLocalContextLifecycleState;
export const parseLocalComponentLifecycleCommitResponseV1 =
  parseLocalComponentLifecycleCommitResponse;

export type LocalBenchmarkMetricKind =
  | "latency_ms"
  | "cost_usd"
  | "accuracy"
  | "hallucination_rate"
  | "tool_usage_count"
  | "token_count"
  | "execution_time_ms"
  | "output_quality"
  | "success_rate";

export type LocalBenchmarkThresholdDirection = "minimum" | "maximum";

export type LocalBenchmarkDecisionStatus = "passed" | "regressed" | "insufficient_data";

export type LocalBenchmarkMetricOutcome = LocalBenchmarkDecisionStatus;

export type LocalBenchmarkComparability = {
  evaluator_key: string;
  evaluator_version: string;
  fingerprint: string;
};

export type LocalBenchmarkMetricEvidence = {
  metric: LocalBenchmarkMetricKind;
  threshold_direction: LocalBenchmarkThresholdDirection;
  threshold_value: number;
  observed: number | null;
  sample_count: number;
  required_sample_count: number;
  has_complete_coverage: boolean;
  outcome: LocalBenchmarkMetricOutcome;
};

export type LocalBenchmarkDefinitionThreshold = {
  metric: LocalBenchmarkMetricKind;
  direction: LocalBenchmarkThresholdDirection;
  value: number;
};

export type LocalBenchmarkDecisionDefinitionSuite = {
  id: string;
  name: string;
  thresholds: LocalBenchmarkDefinitionThreshold[];
};

export type LocalBenchmarkDecisionDefinitionDataset = {
  id: string;
  name: string;
  case_count: number;
};

export type LocalBenchmarkDecisionDefinition = {
  suite: LocalBenchmarkDecisionDefinitionSuite;
  datasets: LocalBenchmarkDecisionDefinitionDataset[];
};

export type LocalBenchmarkDecision = {
  project_id: string;
  context_id: string;
  commit_id: string;
  decision_id: string;
  suite_id: string;
  dataset_ids: string[];
  definition: LocalBenchmarkDecisionDefinition;
  run_ids: string[];
  comparability: LocalBenchmarkComparability;
  evidence_digest: string;
  status: LocalBenchmarkDecisionStatus;
  recorded_at: string;
  metrics: LocalBenchmarkMetricEvidence[];
};

export type LocalBenchmarkDecisionListSuite = {
  id: string;
  name: string;
};

export type LocalBenchmarkDecisionListDataset = {
  id: string;
  name: string;
  case_count: number;
};

export type LocalBenchmarkDecisionListItem = {
  decision_id: string;
  suite: LocalBenchmarkDecisionListSuite;
  datasets: LocalBenchmarkDecisionListDataset[];
  status: LocalBenchmarkDecisionStatus;
  recorded_at: string;
  run_count: number;
};

export type LocalBenchmarkDecisionList = {
  schema_version: "contextlab.local-benchmark-decision-list.v1";
  project_id: string;
  context_id: string;
  commit_id: string;
  decisions: LocalBenchmarkDecisionListItem[];
};

export type LocalBenchmarkDecisionScope = {
  commit_id: string;
  decision_id: string;
};

export type LocalBenchmarkDecisionStatusChange = {
  baseline: LocalBenchmarkDecisionStatus;
  revised: LocalBenchmarkDecisionStatus;
};

export type LocalBenchmarkDecisionMetricChange =
  | {
      kind: "added";
      metric: LocalBenchmarkMetricKind;
      revised: LocalBenchmarkMetricEvidence;
    }
  | {
      kind: "removed";
      metric: LocalBenchmarkMetricKind;
      baseline: LocalBenchmarkMetricEvidence;
    }
  | {
      kind: "modified";
      metric: LocalBenchmarkMetricKind;
      baseline: LocalBenchmarkMetricEvidence;
      revised: LocalBenchmarkMetricEvidence;
    };

export type LocalBenchmarkDecisionDiff = {
  project_id: string;
  context_id: string;
  baseline: LocalBenchmarkDecisionScope;
  revised: LocalBenchmarkDecisionScope;
  status_change: LocalBenchmarkDecisionStatusChange | null;
  metric_changes: LocalBenchmarkDecisionMetricChange[];
};

export type LocalBenchmarkDecisionRunMetric = {
  metric: LocalBenchmarkMetricKind;
  value: number;
};

export type LocalBenchmarkDecisionRun = {
  run_id: string;
  model_version: string;
  temperature: number;
  metrics: LocalBenchmarkDecisionRunMetric[];
  executed_at: string;
};

export type LocalBenchmarkDecisionRunDetails = {
  project_id: string;
  context_id: string;
  commit_id: string;
  decision_id: string;
  runs: LocalBenchmarkDecisionRun[];
};

export function parseLocalBenchmarkDecision(value: unknown): LocalBenchmarkDecision {
  assertNoRawBenchmarkPayload(value);
  const record = asRecord(value, "benchmark decision");
  assertExactKeys(record, [
    "project_id",
    "context_id",
    "commit_id",
    "decision_id",
    "suite_id",
    "dataset_ids",
    "definition",
    "run_ids",
    "comparability",
    "evidence_digest",
    "status",
    "recorded_at",
    "metrics"
  ]);

  const suiteId = asString(record.suite_id, "suite_id");
  const datasetIds = asUniqueStringArray(record.dataset_ids, "dataset_ids");
  assertAscendingValues(datasetIds, "dataset_ids");

  return {
    project_id: asString(record.project_id, "project_id"),
    context_id: asString(record.context_id, "context_id"),
    commit_id: asString(record.commit_id, "commit_id"),
    decision_id: asString(record.decision_id, "decision_id"),
    suite_id: suiteId,
    dataset_ids: datasetIds,
    definition: parseBenchmarkDecisionDefinition(record.definition, suiteId, datasetIds),
    run_ids: asUniqueStringArray(record.run_ids, "run_ids"),
    comparability: parseBenchmarkComparability(record.comparability),
    evidence_digest: asString(record.evidence_digest, "evidence_digest"),
    status: asDecisionStatus(record.status, "status"),
    recorded_at: asTimestamp(record.recorded_at, "recorded_at"),
    metrics: asArray(record.metrics, "metrics").map(parseBenchmarkMetricEvidence)
  };
}

export function parseLocalBenchmarkDecisionList(value: unknown): LocalBenchmarkDecisionList {
  assertNoRawBenchmarkPayload(value);
  const record = asRecord(value, "benchmark decision list");
  assertExactKeys(record, ["schema_version", "project_id", "context_id", "commit_id", "decisions"]);

  const schemaVersion = asString(record.schema_version, "schema_version");
  if (schemaVersion !== "contextlab.local-benchmark-decision-list.v1") {
    throw new TypeError("schema_version must be contextlab.local-benchmark-decision-list.v1");
  }

  const decisions = asArray(record.decisions, "decisions").map(
    parseLocalBenchmarkDecisionListItem
  );
  assertUniqueValues(
    decisions.map((decision) => decision.decision_id),
    "decisions"
  );
  assertStableBenchmarkDecisionListOrder(decisions);

  return {
    schema_version: schemaVersion,
    project_id: asNonBlankString(record.project_id, "project_id"),
    context_id: asNonBlankString(record.context_id, "context_id"),
    commit_id: asNonBlankString(record.commit_id, "commit_id"),
    decisions
  };
}

export function parseLocalBenchmarkDecisionDiff(value: unknown): LocalBenchmarkDecisionDiff {
  assertNoRawBenchmarkPayload(value);
  const record = asRecord(value, "benchmark decision diff");
  assertExactKeys(record, [
    "project_id",
    "context_id",
    "baseline",
    "revised",
    "status_change",
    "metric_changes"
  ]);

  return {
    project_id: asString(record.project_id, "project_id"),
    context_id: asString(record.context_id, "context_id"),
    baseline: parseBenchmarkDecisionScope(record.baseline, "baseline"),
    revised: parseBenchmarkDecisionScope(record.revised, "revised"),
    status_change: parseBenchmarkDecisionStatusChange(record.status_change),
    metric_changes: asArray(record.metric_changes, "metric_changes").map(
      parseBenchmarkDecisionMetricChange
    )
  };
}

export function parseLocalBenchmarkDecisionRunDetails(value: unknown): LocalBenchmarkDecisionRunDetails {
  assertNoRawBenchmarkRunDetailsPayload(value);
  const record = asRecord(value, "benchmark decision run details");
  assertExactKeys(record, ["project_id", "context_id", "commit_id", "decision_id", "runs"]);
  const runs = asArray(record.runs, "runs").map(parseBenchmarkDecisionRun);
  assertUniqueValues(
    runs.map((run) => run.run_id),
    "runs"
  );

  return {
    project_id: asString(record.project_id, "project_id"),
    context_id: asString(record.context_id, "context_id"),
    commit_id: asString(record.commit_id, "commit_id"),
    decision_id: asString(record.decision_id, "decision_id"),
    runs
  };
}

export type LocalApiErrorBody = {
  error: string;
  message: string;
};

export function parseLocalWorkflowCapabilityStatus(value: unknown): LocalWorkflowCapabilityStatus {
  const record = asRecord(value, "local workflow capability status");
  assertExactKeys(record, ["schema_version", "capability", "enabled", "availability", "reason"]);

  const schemaVersion = asString(record.schema_version, "schema_version");
  if (schemaVersion !== "contextlab.local-workflow-capability-status.v1") {
    throw new TypeError("schema_version must be contextlab.local-workflow-capability-status.v1");
  }
  const capability = asString(record.capability, "capability");
  if (capability !== "workflow") {
    throw new TypeError("capability must be workflow");
  }
  const availability = asWorkflowCapabilityAvailability(record.availability);
  const enabled = asBoolean(record.enabled, "enabled");
  if ((availability === "unavailable" && enabled) || (availability === "available" && !enabled)) {
    throw new TypeError("enabled must match workflow capability availability");
  }

  return {
    schema_version: schemaVersion,
    capability,
    enabled,
    availability,
    reason: asString(record.reason, "reason")
  };
}

export function parseLocalWorkflowContextBindings(value: unknown): LocalWorkflowContextBindings {
  const record = asRecord(value, "local workflow Context bindings");
  assertExactKeys(record, ["schema_version", "context_id", "commit_id", "bindings"]);

  const schemaVersion = asString(record.schema_version, "schema_version");
  if (schemaVersion !== "contextlab.local-workflow-context-bindings.v1") {
    throw new TypeError("schema_version must be contextlab.local-workflow-context-bindings.v1");
  }

  const contextId = asNonBlankString(record.context_id, "context_id");
  const commitId = asNonBlankString(record.commit_id, "commit_id");
  const bindings = asArray(record.bindings, "bindings").map(parseLocalWorkflowContextBinding);
  assertStableWorkflowContextBindingOrder(bindings);

  return {
    schema_version: schemaVersion,
    context_id: contextId,
    commit_id: commitId,
    bindings
  };
}

function parseLocalWorkflowContextBinding(value: unknown): LocalWorkflowContextBinding {
  const record = asRecord(value, "workflow Context binding");
  assertExactKeys(record, [
    "binding_id",
    "workflow_id",
    "workflow_revision",
    "node_count",
    "edge_count"
  ]);

  return {
    binding_id: asNonBlankString(record.binding_id, "binding_id"),
    workflow_id: asNonBlankString(record.workflow_id, "workflow_id"),
    workflow_revision: asPositiveInteger(record.workflow_revision, "workflow_revision"),
    node_count: asNonNegativeInteger(record.node_count, "node_count"),
    edge_count: asNonNegativeInteger(record.edge_count, "edge_count")
  };
}

function parseLocalLifecycleOperation(value: unknown): LocalLifecycleOperation {
  const record = asRecord(value, "operation");
  const kind = asString(record.kind, "operation.kind");

  switch (kind) {
    case "initialize":
      assertExactKeys(record, ["kind"]);
      return { kind };
    case "update_metadata":
      assertExactKeys(record, ["kind", "metadata"]);
      return {
        kind,
        metadata: parseLocalContextMetadata(record.metadata)
      };
    case "create":
      assertExactKeys(record, ["kind", "component_kind", "name", "metadata", "content"]);
      return {
        kind,
        component_kind: asContextComponentKind(record.component_kind),
        name: asString(record.name, "operation.name"),
        metadata: parseLocalJsonValue(record.metadata, "operation.metadata"),
        content: asString(record.content, "operation.content")
      };
    case "update":
      assertExactKeys(record, ["kind", "component_id", "content"]);
      return {
        kind,
        component_id: asString(record.component_id, "operation.component_id"),
        content: asString(record.content, "operation.content")
      };
    case "update_descriptor":
      assertExactKeys(record, ["kind", "component_id", "name", "metadata"]);
      return {
        kind,
        component_id: asString(record.component_id, "operation.component_id"),
        name: asString(record.name, "operation.name"),
        metadata: parseLocalJsonValue(record.metadata, "operation.metadata")
      };
    case "remove":
      assertExactKeys(record, ["kind", "component_id"]);
      return {
        kind,
        component_id: asString(record.component_id, "operation.component_id")
      };
    case "add_uses_relationship":
    case "remove_uses_relationship":
      assertExactKeys(record, ["kind", "source_component_id", "target_component_id"]);
      return {
        kind,
        source_component_id: asString(record.source_component_id, "operation.source_component_id"),
        target_component_id: asString(record.target_component_id, "operation.target_component_id")
      };
    default:
      throw new TypeError("operation.kind is unsupported");
  }
}

function parseLocalContextMetadata(value: unknown): LocalContextMetadata {
  const record = asRecord(value, "operation.metadata");
  assertExactKeys(record, ["created_at", "updated_at", "labels"]);
  const labels = asRecord(record.labels, "operation.metadata.labels");

  return {
    created_at: asTimestamp(record.created_at, "operation.metadata.created_at"),
    updated_at: asTimestamp(record.updated_at, "operation.metadata.updated_at"),
    labels: Object.fromEntries(
      Object.entries(labels).map(([key, label]) => [
        key,
        asString(label, `operation.metadata.labels.${key}`)
      ])
    )
  };
}

function parseLocalContextLifecycleComponent(value: unknown): LocalContextLifecycleComponent {
  const record = asRecord(value, "lifecycle component");
  assertExactKeys(record, [
    "component_id",
    "component_kind",
    "name",
    "metadata",
    "content",
    "content_hash",
    "creation_commit_id",
    "content_commit_id"
  ]);

  return freeze({
    component_id: asUuid(record.component_id, "component_id"),
    component_kind: asContextComponentKind(record.component_kind),
    name: asNonBlankString(record.name, "component.name"),
    metadata: parseLocalJsonValue(record.metadata, "component.metadata"),
    content: asString(record.content, "component.content"),
    content_hash: asNonBlankString(record.content_hash, "component.content_hash"),
    creation_commit_id: asUuid(record.creation_commit_id, "component.creation_commit_id"),
    content_commit_id: asUuid(record.content_commit_id, "component.content_commit_id")
  });
}

function parseLocalCommitGraphSnapshot(
  value: unknown,
  expectedContextId: string | undefined,
  expectedCommitId: string,
  field: string
): LocalCommitGraphSnapshot {
  const record = asRecord(value, field);
  assertExactKeys(record, [
    "project_id",
    "context_id",
    "commit_id",
    "graph",
    "captured_at",
    "schema_version"
  ]);
  if (record.schema_version !== LOCAL_COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1) {
    throw new TypeError(`${field}.schema_version must be 1`);
  }

  const projectId = asUuid(record.project_id, `${field}.project_id`);
  const contextId = asUuid(record.context_id, `${field}.context_id`);
  const commitId = asUuid(record.commit_id, `${field}.commit_id`);
  if (expectedContextId !== undefined && contextId !== expectedContextId) {
    throw new TypeError(`${field}.context_id does not match the lifecycle scope`);
  }
  if (commitId !== expectedCommitId) {
    throw new TypeError(`${field}.commit_id does not match the lifecycle scope`);
  }

  return freeze({
    project_id: projectId,
    context_id: contextId,
    commit_id: commitId,
    graph: parseLocalContextGraph(record.graph, `${field}.graph`),
    captured_at: asTimestamp(record.captured_at, `${field}.captured_at`),
    schema_version: LOCAL_COMMIT_GRAPH_SNAPSHOT_SCHEMA_V1
  });
}

function parseLocalContextGraph(value: unknown, field: string): LocalContextGraph {
  const record = asRecord(value, field);
  assertExactKeys(record, ["nodes", "edges"]);
  const nodesRecord = asRecord(record.nodes, `${field}.nodes`);
  const nodes = Object.fromEntries(
    Object.entries(nodesRecord).map(([nodeId, node]) => {
      const parsedNode = parseLocalGraphNode(node, `${field}.nodes.${nodeId}`);
      if (parsedNode.id !== nodeId) {
        throw new TypeError(`${field}.nodes key must match node.id`);
      }
      return [nodeId, parsedNode];
    })
  );
  const edges = asArray(record.edges, `${field}.edges`).map((edge) =>
    parseLocalGraphEdge(edge, `${field}.edges`)
  );
  for (const edge of edges) {
    if (!(edge.source in nodes) || !(edge.target in nodes)) {
      throw new TypeError(`${field}.edges references an unknown node`);
    }
  }

  return freeze({
    nodes: freeze(nodes),
    edges: freeze(edges)
  });
}

function parseLocalGraphNode(value: unknown, field: string): LocalGraphNode {
  const record = asRecord(value, field);
  assertExactKeys(record, ["id", "kind", "label"]);
  return freeze({
    id: asNonBlankString(record.id, `${field}.id`),
    kind: asGraphNodeKind(record.kind, `${field}.kind`),
    label: asString(record.label, `${field}.label`)
  });
}

function parseLocalGraphEdge(value: unknown, field: string): LocalGraphEdge {
  const record = asRecord(value, field);
  assertExactKeys(record, ["source", "target", "kind"]);
  return freeze({
    source: asNonBlankString(record.source, `${field}.source`),
    target: asNonBlankString(record.target, `${field}.target`),
    kind: asGraphEdgeKind(record.kind, `${field}.kind`)
  });
}

function assertLocalLifecycleComponentOrder(
  components: LocalContextLifecycleComponent[]
): void {
  const componentIds = new Set<string>();
  for (let index = 0; index < components.length; index += 1) {
    const component = components[index]!;
    if (componentIds.has(component.component_id)) {
      throw new TypeError("components contains duplicate identities");
    }
    componentIds.add(component.component_id);

    const previous = components[index - 1];
    if (previous !== undefined && compareCodePoints(previous.component_id, component.component_id) >= 0) {
      throw new TypeError("components must be ordered by component_id");
    }
  }
}

function asWorkflowCapabilityAvailability(value: unknown): LocalWorkflowCapabilityAvailability {
  if (value === "available" || value === "unavailable") {
    return value;
  }

  throw new TypeError("availability must be available or unavailable");
}

function parseUnbornExpectedHead(value: unknown): null {
  if (value !== null) {
    throw new TypeError("expected_head_commit_id must be null for initialize");
  }

  return null;
}

function parseLocalJsonValue(value: unknown, field: string): LocalJsonValue {
  if (value === null || typeof value === "string" || typeof value === "boolean") {
    return value;
  }

  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw new TypeError(`${field} must contain only JSON values`);
    }
    return value;
  }

  if (Array.isArray(value)) {
    return value.map((item, index) => parseLocalJsonValue(item, `${field}[${index}]`));
  }

  if (value !== null && typeof value === "object") {
    const prototype = Object.getPrototypeOf(value);
    if (prototype !== Object.prototype && prototype !== null) {
      throw new TypeError(`${field} must contain only JSON values`);
    }

    return Object.fromEntries(
      Object.entries(value).map(([key, nestedValue]) => [
        key,
        parseLocalJsonValue(nestedValue, `${field}.${key}`)
      ])
    );
  }

  throw new TypeError(`${field} must contain only JSON values`);
}

function asContextComponentKind(value: unknown): ContextComponentKind {
  const kind = asString(value, "operation.component_kind");
  const supported: ContextComponentKind[] = [
    "prompt",
    "system_prompt",
    "memory",
    "knowledge",
    "retrieval",
    "embedding",
    "model_configuration",
    "tool",
    "mcp_server",
    "variable",
    "output_schema",
    "workflow",
    "conversation",
    "evaluation"
  ];
  if (!supported.includes(kind as ContextComponentKind)) {
    throw new TypeError("operation.component_kind is unsupported");
  }

  return kind as ContextComponentKind;
}

function asGraphNodeKind(value: unknown, field: string): string {
  const kind = asString(value, field);
  const supported = [
    "workspace",
    "project",
    "experiment",
    "context",
    "component",
    "prompt",
    "memory",
    "knowledge",
    "tool",
    "model",
    "evaluation",
    "workflow"
  ];
  if (!supported.includes(kind)) {
    throw new TypeError(`${field} is unsupported`);
  }
  return kind;
}

function asGraphEdgeKind(value: unknown, field: string): string {
  const kind = asString(value, field);
  const supported = [
    "owns",
    "contains",
    "configures",
    "retrieves",
    "uses",
    "evaluates",
    "produces",
    "tracks"
  ];
  if (!supported.includes(kind)) {
    throw new TypeError(`${field} is unsupported`);
  }
  return kind;
}

function parseBenchmarkComparability(value: unknown): LocalBenchmarkComparability {
  const record = asRecord(value, "comparability");
  assertExactKeys(record, ["evaluator_key", "evaluator_version", "fingerprint"]);

  return {
    evaluator_key: asString(record.evaluator_key, "comparability.evaluator_key"),
    evaluator_version: asString(record.evaluator_version, "comparability.evaluator_version"),
    fingerprint: asString(record.fingerprint, "comparability.fingerprint")
  };
}

function parseLocalBenchmarkDecisionListItem(value: unknown): LocalBenchmarkDecisionListItem {
  const record = asRecord(value, "benchmark decision list item");
  assertExactKeys(record, [
    "decision_id",
    "suite",
    "datasets",
    "status",
    "recorded_at",
    "run_count"
  ]);

  const datasets = asArray(record.datasets, "decisions.datasets").map(
    parseLocalBenchmarkDecisionListDataset
  );
  assertUniqueValues(
    datasets.map((dataset) => dataset.id),
    "decisions.datasets"
  );
  assertAscendingValues(
    datasets.map((dataset) => dataset.id),
    "decisions.datasets"
  );

  return {
    decision_id: asNonBlankString(record.decision_id, "decisions.decision_id"),
    suite: parseLocalBenchmarkDecisionListSuite(record.suite),
    datasets,
    status: asDecisionStatus(record.status, "decisions.status"),
    recorded_at: asTimestamp(record.recorded_at, "decisions.recorded_at"),
    run_count: asNonNegativeInteger(record.run_count, "decisions.run_count")
  };
}

function parseLocalBenchmarkDecisionListSuite(value: unknown): LocalBenchmarkDecisionListSuite {
  const record = asRecord(value, "decisions.suite");
  assertExactKeys(record, ["id", "name"]);

  return {
    id: asNonBlankString(record.id, "decisions.suite.id"),
    name: asNonBlankString(record.name, "decisions.suite.name")
  };
}

function parseLocalBenchmarkDecisionListDataset(
  value: unknown
): LocalBenchmarkDecisionListDataset {
  const record = asRecord(value, "decisions.datasets");
  assertExactKeys(record, ["id", "name", "case_count"]);

  return {
    id: asNonBlankString(record.id, "decisions.datasets.id"),
    name: asNonBlankString(record.name, "decisions.datasets.name"),
    case_count: asNonNegativeInteger(record.case_count, "decisions.datasets.case_count")
  };
}

function parseBenchmarkDecisionDefinition(
  value: unknown,
  suiteId: string,
  datasetIds: string[]
): LocalBenchmarkDecisionDefinition {
  const record = asRecord(value, "definition");
  assertExactKeys(record, ["suite", "datasets"]);

  const suite = parseBenchmarkDecisionDefinitionSuite(record.suite);
  if (suite.id !== suiteId) {
    throw new TypeError("definition.suite.id does not match suite_id");
  }

  const datasets = asArray(record.datasets, "definition.datasets").map(
    parseBenchmarkDecisionDefinitionDataset
  );
  assertUniqueValues(
    datasets.map((dataset) => dataset.id),
    "definition.datasets"
  );
  assertAscendingValues(
    datasets.map((dataset) => dataset.id),
    "definition.datasets"
  );
  assertSameMembership(datasetIds, datasets.map((dataset) => dataset.id));

  return { suite, datasets };
}

function parseBenchmarkDecisionDefinitionSuite(
  value: unknown
): LocalBenchmarkDecisionDefinitionSuite {
  const record = asRecord(value, "definition.suite");
  assertExactKeys(record, ["id", "name", "thresholds"]);

  const thresholds = asArray(record.thresholds, "definition.suite.thresholds").map(
    parseBenchmarkDefinitionThreshold
  );
  assertUniqueValues(
    thresholds.map((threshold) => threshold.metric),
    "definition.suite.thresholds"
  );
  assertAscendingMetrics(
    thresholds.map((threshold) => threshold.metric),
    "definition.suite.thresholds"
  );

  return {
    id: asString(record.id, "definition.suite.id"),
    name: asNonBlankString(record.name, "definition.suite.name"),
    thresholds
  };
}

function parseBenchmarkDefinitionThreshold(value: unknown): LocalBenchmarkDefinitionThreshold {
  const record = asRecord(value, "definition.suite.thresholds");
  assertExactKeys(record, ["metric", "direction", "value"]);

  return {
    metric: asMetricKind(record.metric),
    direction: asThresholdDirection(record.direction),
    value: asFiniteNumber(record.value, "definition.suite.thresholds.value")
  };
}

function parseBenchmarkDecisionDefinitionDataset(
  value: unknown
): LocalBenchmarkDecisionDefinitionDataset {
  const record = asRecord(value, "definition.datasets");
  assertExactKeys(record, ["id", "name", "case_count"]);

  return {
    id: asString(record.id, "definition.datasets.id"),
    name: asNonBlankString(record.name, "definition.datasets.name"),
    case_count: asNonNegativeInteger(record.case_count, "definition.datasets.case_count")
  };
}

function parseBenchmarkDecisionScope(
  value: unknown,
  field: string
): LocalBenchmarkDecisionScope {
  const record = asRecord(value, field);
  assertExactKeys(record, ["commit_id", "decision_id"]);

  return {
    commit_id: asString(record.commit_id, `${field}.commit_id`),
    decision_id: asString(record.decision_id, `${field}.decision_id`)
  };
}

function parseBenchmarkDecisionStatusChange(
  value: unknown
): LocalBenchmarkDecisionStatusChange | null {
  if (value === null) {
    return null;
  }
  const record = asRecord(value, "status_change");
  assertExactKeys(record, ["baseline", "revised"]);

  return {
    baseline: asDecisionStatus(record.baseline, "status_change.baseline"),
    revised: asDecisionStatus(record.revised, "status_change.revised")
  };
}

function parseBenchmarkMetricEvidence(value: unknown): LocalBenchmarkMetricEvidence {
  const record = asRecord(value, "metric evidence");
  assertExactKeys(record, [
    "metric",
    "threshold_direction",
    "threshold_value",
    "observed",
    "sample_count",
    "required_sample_count",
    "has_complete_coverage",
    "outcome"
  ]);

  return {
    metric: asMetricKind(record.metric),
    threshold_direction: asThresholdDirection(record.threshold_direction),
    threshold_value: asFiniteNumber(record.threshold_value, "threshold_value"),
    observed: record.observed === null ? null : asFiniteNumber(record.observed, "observed"),
    sample_count: asNonNegativeInteger(record.sample_count, "sample_count"),
    required_sample_count: asNonNegativeInteger(record.required_sample_count, "required_sample_count"),
    has_complete_coverage: asBoolean(record.has_complete_coverage, "has_complete_coverage"),
    outcome: asDecisionStatus(record.outcome, "outcome")
  };
}

function parseBenchmarkDecisionRun(value: unknown): LocalBenchmarkDecisionRun {
  const record = asRecord(value, "benchmark decision run");
  assertExactKeys(record, ["run_id", "model_version", "temperature", "metrics", "executed_at"]);

  return {
    run_id: asString(record.run_id, "runs.run_id"),
    model_version: asString(record.model_version, "runs.model_version"),
    temperature: asFiniteNumber(record.temperature, "runs.temperature"),
    metrics: asArray(record.metrics, "runs.metrics").map(parseBenchmarkDecisionRunMetric),
    executed_at: asTimestamp(record.executed_at, "runs.executed_at")
  };
}

function parseBenchmarkDecisionRunMetric(value: unknown): LocalBenchmarkDecisionRunMetric {
  const record = asRecord(value, "benchmark decision run metric");
  assertExactKeys(record, ["metric", "value"]);

  return {
    metric: asMetricKind(record.metric),
    value: asFiniteNumber(record.value, "runs.metrics.value")
  };
}

function parseBenchmarkDecisionMetricChange(value: unknown): LocalBenchmarkDecisionMetricChange {
  const record = asRecord(value, "metric change");
  const kind = asString(record.kind, "metric change.kind");
  const metric = asMetricKind(record.metric);

  switch (kind) {
    case "added": {
      assertExactKeys(record, ["kind", "metric", "revised"]);
      const revised = parseBenchmarkMetricEvidence(record.revised);
      assertMetricChangeEvidence(metric, revised, "revised");
      return { kind, metric, revised };
    }
    case "removed": {
      assertExactKeys(record, ["kind", "metric", "baseline"]);
      const baseline = parseBenchmarkMetricEvidence(record.baseline);
      assertMetricChangeEvidence(metric, baseline, "baseline");
      return { kind, metric, baseline };
    }
    case "modified": {
      assertExactKeys(record, ["kind", "metric", "baseline", "revised"]);
      const baseline = parseBenchmarkMetricEvidence(record.baseline);
      const revised = parseBenchmarkMetricEvidence(record.revised);
      assertMetricChangeEvidence(metric, baseline, "baseline");
      assertMetricChangeEvidence(metric, revised, "revised");
      return { kind, metric, baseline, revised };
    }
    default:
      throw new TypeError("metric change.kind is unsupported");
  }
}

function assertMetricChangeEvidence(
  metric: LocalBenchmarkMetricKind,
  evidence: LocalBenchmarkMetricEvidence,
  field: string
): void {
  if (evidence.metric !== metric) {
    throw new TypeError(`metric change ${field} evidence does not match metric`);
  }
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }

  return value as Record<string, unknown>;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new TypeError(`${field} must be an array`);
  }

  return value;
}

function asString(value: unknown, field: string): string {
  if (typeof value !== "string") {
    throw new TypeError(`${field} must be a string`);
  }

  return value;
}

function asStringArray(value: unknown, field: string): string[] {
  return asArray(value, field).map((item, index) => asString(item, `${field}[${index}]`));
}

function asUniqueStringArray(value: unknown, field: string): string[] {
  const values = asStringArray(value, field);
  assertUniqueValues(values, field);
  return values;
}

function asFiniteNumber(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new TypeError(`${field} must be a finite number`);
  }

  return value;
}

function asNonNegativeInteger(value: unknown, field: string): number {
  const number = asFiniteNumber(value, field);
  if (!Number.isInteger(number) || number < 0) {
    throw new TypeError(`${field} must be a non-negative integer`);
  }

  return number;
}

function asPositiveInteger(value: unknown, field: string): number {
  const number = asNonNegativeInteger(value, field);
  if (number === 0) {
    throw new TypeError(`${field} must be a positive integer`);
  }

  return number;
}

function asBoolean(value: unknown, field: string): boolean {
  if (typeof value !== "boolean") {
    throw new TypeError(`${field} must be a boolean`);
  }

  return value;
}

function asNonBlankString(value: unknown, field: string): string {
  const text = asString(value, field);
  if (!text.trim()) {
    throw new TypeError(`${field} must not be blank`);
  }

  return text;
}

function asUuid(value: unknown, field: string): string {
  const parsed = asNonBlankString(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function asTimestamp(value: unknown, field: string): string {
  const timestamp = asString(value, field);
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?Z$/.exec(timestamp);
  if (!match) {
    throw new TypeError(`${field} must be a valid timestamp`);
  }

  const [, year, month, day, hour, minute, second] = match;
  const date = new Date(0);
  date.setUTCFullYear(Number(year), Number(month) - 1, Number(day));
  date.setUTCHours(Number(hour), Number(minute), Number(second), 0);
  if (
    date.getUTCFullYear() !== Number(year)
    || date.getUTCMonth() !== Number(month) - 1
    || date.getUTCDate() !== Number(day)
    || date.getUTCHours() !== Number(hour)
    || date.getUTCMinutes() !== Number(minute)
    || date.getUTCSeconds() !== Number(second)
  ) {
    throw new TypeError(`${field} must be a valid timestamp`);
  }

  return timestamp;
}

function asMetricKind(value: unknown): LocalBenchmarkMetricKind {
  const metric = asString(value, "metric");
  const supported: LocalBenchmarkMetricKind[] = [
    "latency_ms",
    "cost_usd",
    "accuracy",
    "hallucination_rate",
    "tool_usage_count",
    "token_count",
    "execution_time_ms",
    "output_quality",
    "success_rate"
  ];
  if (!supported.includes(metric as LocalBenchmarkMetricKind)) {
    throw new TypeError("metric is unsupported");
  }

  return metric as LocalBenchmarkMetricKind;
}

function benchmarkMetricOrder(metric: LocalBenchmarkMetricKind): number {
  return [
    "latency_ms",
    "cost_usd",
    "accuracy",
    "hallucination_rate",
    "tool_usage_count",
    "token_count",
    "execution_time_ms",
    "output_quality",
    "success_rate"
  ].indexOf(metric);
}

function asThresholdDirection(value: unknown): LocalBenchmarkThresholdDirection {
  if (value !== "minimum" && value !== "maximum") {
    throw new TypeError("threshold_direction is unsupported");
  }

  return value;
}

function asDecisionStatus(value: unknown, field: string): LocalBenchmarkDecisionStatus {
  if (value !== "passed" && value !== "regressed" && value !== "insufficient_data") {
    throw new TypeError(`${field} is unsupported`);
  }

  return value;
}

function assertExactKeys(
  record: Record<string, unknown>,
  expected: string[],
  field = "value"
): void {
  const actual = Object.keys(record).sort(compareCodePoints);
  const sortedExpected = [...expected].sort(compareCodePoints);
  if (
    actual.length !== sortedExpected.length
    || actual.some((key, index) => key !== sortedExpected[index])
  ) {
    throw new TypeError(`${field} contains an unexpected shape`);
  }
}

function freeze<T>(value: T): T {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) {
      freeze(child);
    }
    Object.freeze(value);
  }
  return value;
}

function assertUniqueValues(values: string[], field: string): void {
  if (new Set(values).size !== values.length) {
    throw new TypeError(`${field} contains duplicate identities`);
  }
}

function assertAscendingValues(values: string[], field: string): void {
  if (values.some((value, index) => index > 0 && compareCodePoints(values[index - 1]!, value) >= 0)) {
    throw new TypeError(`${field} must be ordered by stable identifier`);
  }
}

function assertStableWorkflowContextBindingOrder(bindings: LocalWorkflowContextBinding[]): void {
  const bindingIds = new Set<string>();
  for (let index = 0; index < bindings.length; index += 1) {
    const binding = bindings[index]!;
    if (bindingIds.has(binding.binding_id)) {
      throw new TypeError("bindings contains duplicate identities");
    }
    bindingIds.add(binding.binding_id);

    const previous = bindings[index - 1];
    if (previous === undefined) {
      continue;
    }
    const workflowComparison = compareCodePoints(previous.workflow_id, binding.workflow_id);
    if (
      workflowComparison > 0
      || (workflowComparison === 0 && previous.workflow_revision > binding.workflow_revision)
      || (
        workflowComparison === 0
        && previous.workflow_revision === binding.workflow_revision
        && compareCodePoints(previous.binding_id, binding.binding_id) >= 0
      )
    ) {
      throw new TypeError("bindings must be ordered by workflow_id, workflow_revision, and binding_id");
    }
    if (
      workflowComparison === 0
      && previous.workflow_revision === binding.workflow_revision
    ) {
      throw new TypeError("bindings contains a duplicate workflow revision");
    }
  }
}

function assertStableBenchmarkDecisionListOrder(
  decisions: LocalBenchmarkDecisionListItem[]
): void {
  for (let index = 1; index < decisions.length; index += 1) {
    const previous = decisions[index - 1]!;
    const current = decisions[index]!;
    const previousTime = Date.parse(previous.recorded_at);
    const currentTime = Date.parse(current.recorded_at);
    if (
      previousTime < currentTime
      || (previousTime === currentTime
        && compareCodePoints(previous.decision_id, current.decision_id) >= 0)
    ) {
      throw new TypeError(
        "decisions must be ordered by recorded_at descending and decision_id ascending"
      );
    }
  }
}

function assertAscendingMetrics(values: LocalBenchmarkMetricKind[], field: string): void {
  if (
    values.some(
      (metric, index) =>
        index > 0 && benchmarkMetricOrder(values[index - 1]!) >= benchmarkMetricOrder(metric)
    )
  ) {
    throw new TypeError(`${field} must be ordered by stable metric`);
  }
}

function assertSameMembership(expected: string[], actual: string[]): void {
  if (
    expected.length !== actual.length
    || expected.some((id, index) => id !== actual[index])
  ) {
    throw new TypeError("definition.datasets does not match dataset_ids");
  }
}

function assertNoRawBenchmarkPayload(value: unknown): void {
  if (Array.isArray(value)) {
    value.forEach(assertNoRawBenchmarkPayload);
    return;
  }

  if (value === null || typeof value !== "object") {
    return;
  }

  for (const [key, nestedValue] of Object.entries(value)) {
    if (
      key === "cases"
      || key === "input"
      || key === "expected_output"
      || key === "runs"
      || key === "measurements"
      || key === "output"
      || key === "model_output"
    ) {
      throw new TypeError("benchmark evidence contains a raw benchmark payload");
    }
    assertNoRawBenchmarkPayload(nestedValue);
  }
}

function assertNoRawBenchmarkRunDetailsPayload(value: unknown): void {
  if (Array.isArray(value)) {
    value.forEach(assertNoRawBenchmarkRunDetailsPayload);
    return;
  }
  if (value === null || typeof value !== "object") {
    return;
  }

  for (const [key, nestedValue] of Object.entries(value)) {
    if (
      ["cases", "input", "expected_output", "output", "model_output", "measurements"].includes(
        key
      )
    ) {
      throw new TypeError("benchmark raw payloads are not available through the local SDK");
    }
    assertNoRawBenchmarkRunDetailsPayload(nestedValue);
  }
}

function compareCodePoints(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}
