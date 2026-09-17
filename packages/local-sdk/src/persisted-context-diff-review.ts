import type { LocalContextMetadata, LocalLifecycleReadCredentials } from "./types";
import type { ContextLabLocalClientOptions, FetchLike } from "./client";
import type { LocalGraphEdgeKindV1, LocalGraphNodeKindV1 } from "./commit-graph-diff";

export const LOCAL_PERSISTED_CONTEXT_DIFF_REVIEW_SCHEMA_V1 = "v1" as const;

type ReadonlyRecord = Readonly<Record<string, unknown>>;
type DiffVersion = typeof LOCAL_PERSISTED_CONTEXT_DIFF_REVIEW_SCHEMA_V1;

export type LocalPersistedContextDiffReviewScopeV1 = Readonly<{
  project_id: string;
  context_id: string;
  commit_id: string;
}>;

export type LocalPersistedContextDiffReviewV1 = Readonly<{
  schema_version: DiffVersion;
  source_scope: LocalPersistedContextDiffReviewScopeV1;
  target_scope: LocalPersistedContextDiffReviewScopeV1;
  diff: LocalContextDiffResultV1;
}>;

export type LocalContextDiffResultV1 = Readonly<{
  contract_version: DiffVersion;
  semantic: LocalSemanticDiffV1;
  behavior: LocalBehaviorDiffV1;
  evaluation: LocalEvaluationDiffV1;
}>;

export type LocalSemanticDiffV1 = Readonly<{
  graph_diff: LocalGraphDiffV1;
  document_changes: ReadonlyArray<LocalSemanticDocumentChangeV1>;
  metadata_change?: LocalContextMetadataChangeV1;
}>;

export type LocalContextMetadataChangeV1 =
  | Readonly<{ kind: "added"; revised: LocalContextMetadata }>
  | Readonly<{ kind: "removed"; original: LocalContextMetadata }>
  | Readonly<{
      kind: "modified";
      original: LocalContextMetadata;
      revised: LocalContextMetadata;
    }>;

export type LocalGraphDiffV1 = Readonly<{
  added_nodes: ReadonlyArray<LocalGraphNodeV1>;
  removed_nodes: ReadonlyArray<LocalGraphNodeV1>;
  modified_nodes: ReadonlyArray<LocalGraphNodeChangeV1>;
  added_edges: ReadonlyArray<LocalGraphEdgeV1>;
  removed_edges: ReadonlyArray<LocalGraphEdgeV1>;
}>;

export type LocalGraphNodeV1 = Readonly<{
  id: string;
  kind: LocalGraphNodeKindV1;
  label: string;
}>;

export type LocalGraphNodeChangeV1 = Readonly<{
  node_id: string;
  original_kind: LocalGraphNodeKindV1;
  revised_kind: LocalGraphNodeKindV1;
  original_label: string;
  revised_label: string;
}>;

export type LocalGraphEdgeV1 = Readonly<{
  source: string;
  target: string;
  kind: LocalGraphEdgeKindV1;
}>;

export type LocalSemanticDocumentV1 = Readonly<{ id: string; content: string }>;

export type LocalTextDiffV1 = Readonly<{
  lines: ReadonlyArray<LocalTextDiffLineV1>;
}>;

export type LocalTextDiffLineV1 = Readonly<{
  kind: "unchanged" | "added" | "removed";
  text: string;
}>;

export type LocalSemanticDocumentChangeV1 =
  | Readonly<{ kind: "added"; document: LocalSemanticDocumentV1 }>
  | Readonly<{ kind: "removed"; document: LocalSemanticDocumentV1 }>
  | Readonly<{ kind: "modified"; document_id: string; text_diff: LocalTextDiffV1 }>;

export type LocalBehaviorDiffV1 = Readonly<{
  case_changes: ReadonlyArray<LocalBehaviorCaseChangeV1>;
}>;

export type LocalBehaviorObservationV1 = Readonly<{
  case_id: string;
  input_fingerprint: string;
  outcome: LocalBehaviorOutcomeV1;
}>;

export type LocalBehaviorOutcomeV1 =
  | Readonly<{ kind: "succeeded"; output: string }>
  | Readonly<{ kind: "failed"; error_code: string }>;

export type LocalBehaviorCaseChangeV1 =
  | Readonly<{ kind: "added"; revised: LocalBehaviorObservationV1 }>
  | Readonly<{ kind: "removed"; original: LocalBehaviorObservationV1 }>
  | Readonly<{
      kind: "modified";
      original: LocalBehaviorObservationV1;
      revised: LocalBehaviorObservationV1;
    }>;

export type LocalEvaluationDiffV1 = Readonly<{
  comparability_fingerprint: string;
  metric_changes: ReadonlyArray<LocalEvaluationMetricChangeV1>;
}>;

export type LocalEvaluationMetricObservationV1 = Readonly<{
  metric_id: string;
  value: number;
  sample_count: number;
}>;

export type LocalEvaluationMetricChangeV1 =
  | Readonly<{ kind: "added"; revised: LocalEvaluationMetricObservationV1 }>
  | Readonly<{ kind: "removed"; original: LocalEvaluationMetricObservationV1 }>
  | Readonly<{
      kind: "modified";
      original: LocalEvaluationMetricObservationV1;
      revised: LocalEvaluationMetricObservationV1;
    }>;

export class ContextLabLocalPersistedContextDiffReviewError extends Error {
  readonly status: number;
  readonly code: string;
  readonly retryAfterMs?: number;

  constructor(status: number, code: string, retryAfterMs?: number) {
    super(`ContextLab local API request failed with status ${status}`);
    this.name = "ContextLabLocalPersistedContextDiffReviewError";
    this.status = status;
    this.code = code;
    this.retryAfterMs = retryAfterMs;
  }
}

export class ContextLabLocalPersistedContextDiffReviewClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getPersistedContextDiffReview(
    projectId: string,
    contextId: string,
    sourceCommitId: string,
    targetCommitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalPersistedContextDiffReviewV1> {
    const requestedProjectId = asUuid(projectId, "projectId");
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedSourceCommitId = asUuid(sourceCommitId, "sourceCommitId");
    const requestedTargetCommitId = asUuid(targetCommitId, "targetCommitId");
    if (requestedSourceCommitId === requestedTargetCommitId) {
      throw new RangeError("sourceCommitId and targetCommitId must differ");
    }
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const query = new URLSearchParams({
      source_commit_id: requestedSourceCommitId,
      target_commit_id: requestedTargetCommitId
    });
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/projects/${encodeURIComponent(requestedProjectId)}`
        + `/contexts/${encodeURIComponent(requestedContextId)}/diff-review?${query.toString()}`,
      {
        method: "GET",
        credentials: "omit",
        cache: "no-store",
        headers: {
          accept: "application/json",
          authorization: `Bearer ${bearerToken}`
        }
      }
    );

    if (!response.ok) {
      throw new ContextLabLocalPersistedContextDiffReviewError(
        response.status,
        await parseErrorCode(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const result = parsePersistedContextDiffReviewV1(await response.json());
    if (
      result.source_scope.project_id !== requestedProjectId
      || result.source_scope.context_id !== requestedContextId
      || result.source_scope.commit_id !== requestedSourceCommitId
      || result.target_scope.project_id !== requestedProjectId
      || result.target_scope.context_id !== requestedContextId
      || result.target_scope.commit_id !== requestedTargetCommitId
    ) {
      throw new TypeError("persisted Context diff review response does not match the requested scope");
    }
    return result;
  }
}

export function parsePersistedContextDiffReviewV1(
  value: unknown
): LocalPersistedContextDiffReviewV1 {
  const record = asRecord(value, "persisted Context diff review");
  assertExactKeys(record, ["schema_version", "source_scope", "target_scope", "diff"]);
  if (record.schema_version !== LOCAL_PERSISTED_CONTEXT_DIFF_REVIEW_SCHEMA_V1) {
    throw new TypeError("schema_version must be v1");
  }
  const sourceScope = parseScope(record.source_scope, "source_scope");
  const targetScope = parseScope(record.target_scope, "target_scope");
  if (sourceScope.project_id !== targetScope.project_id || sourceScope.context_id !== targetScope.context_id) {
    throw new TypeError("persisted Context diff review scopes must share project and context");
  }
  if (sourceScope.commit_id === targetScope.commit_id) {
    throw new TypeError("persisted Context diff review scopes must compare distinct commits");
  }
  return freeze({
    schema_version: "v1",
    source_scope: sourceScope,
    target_scope: targetScope,
    diff: parseDiffResult(record.diff)
  });
}

export const parseLocalPersistedContextDiffReviewV1 = parsePersistedContextDiffReviewV1;

function parseScope(value: unknown, field: string): LocalPersistedContextDiffReviewScopeV1 {
  const record = asRecord(value, field);
  assertExactKeys(record, ["project_id", "context_id", "commit_id"]);
  return freeze({
    project_id: asUuid(record.project_id, `${field}.project_id`),
    context_id: asUuid(record.context_id, `${field}.context_id`),
    commit_id: asUuid(record.commit_id, `${field}.commit_id`)
  });
}

function parseDiffResult(value: unknown): LocalContextDiffResultV1 {
  const record = asRecord(value, "diff");
  assertExactKeys(record, ["contract_version", "semantic", "behavior", "evaluation"]);
  if (record.contract_version !== "v1") {
    throw new TypeError("diff.contract_version must be v1");
  }
  return freeze({
    contract_version: "v1",
    semantic: parseSemanticDiff(record.semantic),
    behavior: parseBehaviorDiff(record.behavior),
    evaluation: parseEvaluationDiff(record.evaluation)
  });
}

function parseSemanticDiff(value: unknown): LocalSemanticDiffV1 {
  const record = asRecord(value, "diff.semantic");
  assertExactKeysWithOptional(record, ["graph_diff", "document_changes"], ["metadata_change"]);
  const documentChanges = asArray(record.document_changes, "diff.semantic.document_changes")
    .map(parseDocumentChange);
  assertStrictlyAscending(documentChanges.map(documentChangeId), "diff.semantic.document_changes");
  const semantic: {
    graph_diff: LocalGraphDiffV1;
    document_changes: ReadonlyArray<LocalSemanticDocumentChangeV1>;
    metadata_change?: LocalContextMetadataChangeV1;
  } = {
    graph_diff: parseGraphDiff(record.graph_diff),
    document_changes: freeze(documentChanges)
  };
  if (Object.prototype.hasOwnProperty.call(record, "metadata_change")) {
    semantic.metadata_change = parseMetadataChange(record.metadata_change);
  }
  return freeze(semantic);
}

function parseMetadataChange(value: unknown): LocalContextMetadataChangeV1 {
  const record = asRecord(value, "diff.semantic.metadata_change");
  const kind = asKind(record.kind, "diff.semantic.metadata_change.kind");
  if (kind === "added") {
    assertExactKeys(record, ["kind", "revised"]);
    return freeze({
      kind: "added",
      revised: parseContextMetadata(record.revised, "metadata_change.revised")
    });
  }
  if (kind === "removed") {
    assertExactKeys(record, ["kind", "original"]);
    return freeze({
      kind: "removed",
      original: parseContextMetadata(record.original, "metadata_change.original")
    });
  }
  if (kind !== "modified") {
    throw new TypeError("diff.semantic.metadata_change.kind is unsupported");
  }
  assertExactKeys(record, ["kind", "original", "revised"]);
  const original = parseContextMetadata(record.original, "metadata_change.original");
  const revised = parseContextMetadata(record.revised, "metadata_change.revised");
  if (metadataEqual(original, revised)) {
    throw new TypeError("modified metadata must contain a change");
  }
  return freeze({ kind: "modified", original, revised });
}

function parseContextMetadata(value: unknown, field: string): LocalContextMetadata {
  const record = asRecord(value, field);
  assertExactKeys(record, ["created_at", "updated_at", "labels"]);
  const labelsRecord = asRecord(record.labels, `${field}.labels`);
  const labels = Object.fromEntries(
    Object.entries(labelsRecord).map(([key, label]) => [
      key,
      asString(label, `${field}.labels.${key}`)
    ])
  );
  return freeze({
    created_at: asTimestamp(record.created_at, `${field}.created_at`),
    updated_at: asTimestamp(record.updated_at, `${field}.updated_at`),
    labels: freeze(labels)
  });
}

function metadataEqual(original: LocalContextMetadata, revised: LocalContextMetadata): boolean {
  const originalLabels = Object.entries(original.labels).sort(([left], [right]) => compareStrings(left, right));
  const revisedLabels = Object.entries(revised.labels).sort(([left], [right]) => compareStrings(left, right));
  return original.created_at === revised.created_at
    && original.updated_at === revised.updated_at
    && originalLabels.length === revisedLabels.length
    && originalLabels.every(([key, value], index) => {
      const revisedEntry = revisedLabels[index];
      return revisedEntry !== undefined && key === revisedEntry[0] && value === revisedEntry[1];
    });
}

function parseGraphDiff(value: unknown): LocalGraphDiffV1 {
  const record = asRecord(value, "diff.semantic.graph_diff");
  assertExactKeys(record, ["added_nodes", "removed_nodes", "modified_nodes", "added_edges", "removed_edges"]);
  const addedNodes = asArray(record.added_nodes, "added_nodes").map((item) => parseNode(item, "added node"));
  const removedNodes = asArray(record.removed_nodes, "removed_nodes").map((item) => parseNode(item, "removed node"));
  const modifiedNodes = asArray(record.modified_nodes, "modified_nodes").map(parseNodeChange);
  const addedEdges = asArray(record.added_edges, "added_edges").map((item) => parseEdge(item, "added edge"));
  const removedEdges = asArray(record.removed_edges, "removed_edges").map((item) => parseEdge(item, "removed edge"));
  assertStrictlyAscending(addedNodes.map((node) => node.id), "added_nodes");
  assertStrictlyAscending(removedNodes.map((node) => node.id), "removed_nodes");
  assertStrictlyAscending(modifiedNodes.map((node) => node.node_id), "modified_nodes");
  assertAscendingEdges(addedEdges, "added_edges");
  assertAscendingEdges(removedEdges, "removed_edges");
  return freeze({
    added_nodes: freeze(addedNodes),
    removed_nodes: freeze(removedNodes),
    modified_nodes: freeze(modifiedNodes),
    added_edges: freeze(addedEdges),
    removed_edges: freeze(removedEdges)
  });
}

function parseNode(value: unknown, field: string): LocalGraphNodeV1 {
  const record = asRecord(value, field);
  assertExactKeys(record, ["id", "kind", "label"]);
  return freeze({
    id: asCanonicalNonBlank(record.id, `${field}.id`),
    kind: asGraphNodeKind(record.kind, `${field}.kind`),
    label: asCanonicalNonBlank(record.label, `${field}.label`)
  });
}

function parseNodeChange(value: unknown): LocalGraphNodeChangeV1 {
  const record = asRecord(value, "modified node");
  assertExactKeys(record, ["node_id", "original_kind", "revised_kind", "original_label", "revised_label"]);
  const originalKind = asGraphNodeKind(record.original_kind, "modified node.original_kind");
  const revisedKind = asGraphNodeKind(record.revised_kind, "modified node.revised_kind");
  const originalLabel = asCanonicalNonBlank(record.original_label, "modified node.original_label");
  const revisedLabel = asCanonicalNonBlank(record.revised_label, "modified node.revised_label");
  if (originalKind === revisedKind && originalLabel === revisedLabel) {
    throw new TypeError("modified node must contain a change");
  }
  return freeze({
    node_id: asCanonicalNonBlank(record.node_id, "modified node.node_id"),
    original_kind: originalKind,
    revised_kind: revisedKind,
    original_label: originalLabel,
    revised_label: revisedLabel
  });
}

function parseEdge(value: unknown, field: string): LocalGraphEdgeV1 {
  const record = asRecord(value, field);
  assertExactKeys(record, ["source", "target", "kind"]);
  const source = asCanonicalNonBlank(record.source, `${field}.source`);
  const target = asCanonicalNonBlank(record.target, `${field}.target`);
  if (source === target) throw new TypeError(`${field} must not be self-referential`);
  return freeze({ source, target, kind: asGraphEdgeKind(record.kind, `${field}.kind`) });
}

function parseDocumentChange(value: unknown): LocalSemanticDocumentChangeV1 {
  const record = asRecord(value, "document change");
  const kind = asKind(record.kind, "document change.kind");
  if (kind === "added" || kind === "removed") {
    assertExactKeys(record, ["kind", "document"]);
    return freeze({ kind, document: parseDocument(record.document, "document change.document") });
  }
  if (kind !== "modified") {
    throw new TypeError("document change.kind is unsupported");
  }
  assertExactKeys(record, ["kind", "document_id", "text_diff"]);
  const textDiff = parseTextDiff(record.text_diff);
  if (!textDiff.lines.some((line) => line.kind !== "unchanged")) {
    throw new TypeError("modified document must contain an added or removed line");
  }
  return freeze({
    kind: "modified",
    document_id: asCanonicalNonBlank(record.document_id, "document change.document_id"),
    text_diff: textDiff
  });
}

function parseDocument(value: unknown, field: string): LocalSemanticDocumentV1 {
  const record = asRecord(value, field);
  assertExactKeys(record, ["id", "content"]);
  return freeze({
    id: asCanonicalNonBlank(record.id, `${field}.id`),
    content: asString(record.content, `${field}.content`)
  });
}

function parseTextDiff(value: unknown): LocalTextDiffV1 {
  const record = asRecord(value, "text_diff");
  assertExactKeys(record, ["lines"]);
  const lines = asArray(record.lines, "text_diff.lines").map((item) => {
    const line = asRecord(item, "text diff line");
    assertExactKeys(line, ["kind", "text"]);
    const kind = asKind(line.kind, "text diff line.kind");
    if (kind !== "unchanged" && kind !== "added" && kind !== "removed") {
      throw new TypeError("text diff line.kind is unsupported");
    }
    const parsedLine: LocalTextDiffLineV1 = {
      kind: kind as LocalTextDiffLineV1["kind"],
      text: asString(line.text, "text diff line.text")
    };
    return freeze(parsedLine);
  });
  return freeze({ lines: freeze(lines) });
}

function parseBehaviorDiff(value: unknown): LocalBehaviorDiffV1 {
  const record = asRecord(value, "diff.behavior");
  assertExactKeys(record, ["case_changes"]);
  const changes = asArray(record.case_changes, "diff.behavior.case_changes").map(parseBehaviorChange);
  assertStrictlyAscending(changes.map(behaviorChangeId), "diff.behavior.case_changes");
  return freeze({ case_changes: freeze(changes) });
}

function parseBehaviorChange(value: unknown): LocalBehaviorCaseChangeV1 {
  const record = asRecord(value, "behavior case change");
  const kind = asKind(record.kind, "behavior case change.kind");
  if (kind === "added") {
    assertExactKeys(record, ["kind", "revised"]);
    return freeze({ kind, revised: parseObservation(record.revised, "revised") });
  }
  if (kind === "removed") {
    assertExactKeys(record, ["kind", "original"]);
    return freeze({ kind, original: parseObservation(record.original, "original") });
  }
  if (kind !== "modified") {
    throw new TypeError("behavior case change.kind is unsupported");
  }
  assertExactKeys(record, ["kind", "original", "revised"]);
  const original = parseObservation(record.original, "original");
  const revised = parseObservation(record.revised, "revised");
  if (original.case_id !== revised.case_id || original.input_fingerprint !== revised.input_fingerprint) {
    throw new TypeError("modified behavior case must preserve identity and input fingerprint");
  }
  if (JSON.stringify(original.outcome) === JSON.stringify(revised.outcome)) {
    throw new TypeError("modified behavior case must contain an outcome change");
  }
  return freeze({ kind: "modified", original, revised });
}

function parseObservation(value: unknown, field: string): LocalBehaviorObservationV1 {
  const record = asRecord(value, `behavior ${field}`);
  assertExactKeys(record, ["case_id", "input_fingerprint", "outcome"]);
  return freeze({
    case_id: asCanonicalNonBlank(record.case_id, `behavior ${field}.case_id`),
    input_fingerprint: asCanonicalNonBlank(record.input_fingerprint, `behavior ${field}.input_fingerprint`),
    outcome: parseOutcome(record.outcome, `behavior ${field}.outcome`)
  });
}

function parseOutcome(value: unknown, field: string): LocalBehaviorOutcomeV1 {
  const record = asRecord(value, field);
  const kind = asKind(record.kind, `${field}.kind`);
  if (kind === "succeeded") {
    assertExactKeys(record, ["kind", "output"]);
    return freeze({ kind, output: asString(record.output, `${field}.output`) });
  }
  if (kind === "failed") {
    assertExactKeys(record, ["kind", "error_code"]);
    return freeze({ kind, error_code: asCanonicalNonBlank(record.error_code, `${field}.error_code`) });
  }
  throw new TypeError(`${field}.kind is unsupported`);
}

function parseEvaluationDiff(value: unknown): LocalEvaluationDiffV1 {
  const record = asRecord(value, "diff.evaluation");
  assertExactKeys(record, ["comparability_fingerprint", "metric_changes"]);
  const changes = asArray(record.metric_changes, "diff.evaluation.metric_changes").map(parseMetricChange);
  assertStrictlyAscending(changes.map(metricChangeId), "diff.evaluation.metric_changes");
  return freeze({
    comparability_fingerprint: asCanonicalNonBlank(record.comparability_fingerprint, "comparability_fingerprint"),
    metric_changes: freeze(changes)
  });
}

function parseMetricChange(value: unknown): LocalEvaluationMetricChangeV1 {
  const record = asRecord(value, "evaluation metric change");
  const kind = asKind(record.kind, "evaluation metric change.kind");
  if (kind === "added") {
    assertExactKeys(record, ["kind", "revised"]);
    return freeze({ kind, revised: parseMetric(record.revised, "revised") });
  }
  if (kind === "removed") {
    assertExactKeys(record, ["kind", "original"]);
    return freeze({ kind, original: parseMetric(record.original, "original") });
  }
  if (kind !== "modified") {
    throw new TypeError("evaluation metric change.kind is unsupported");
  }
  assertExactKeys(record, ["kind", "original", "revised"]);
  const original = parseMetric(record.original, "original");
  const revised = parseMetric(record.revised, "revised");
  if (original.metric_id !== revised.metric_id) {
    throw new TypeError("modified evaluation metric must preserve identity");
  }
  if (original.value === revised.value && original.sample_count === revised.sample_count) {
    throw new TypeError("modified evaluation metric must contain a value or sample-count change");
  }
  return freeze({ kind: "modified", original, revised });
}

function parseMetric(value: unknown, field: string): LocalEvaluationMetricObservationV1 {
  const record = asRecord(value, `evaluation ${field}`);
  assertExactKeys(record, ["metric_id", "value", "sample_count"]);
  return freeze({
    metric_id: asCanonicalNonBlank(record.metric_id, `evaluation ${field}.metric_id`),
    value: asFiniteNumber(record.value, `evaluation ${field}.value`),
    sample_count: asNonNegativeInteger(record.sample_count, `evaluation ${field}.sample_count`)
  });
}

function documentChangeId(change: LocalSemanticDocumentChangeV1): string {
  return change.kind === "modified" ? change.document_id : change.document.id;
}

function behaviorChangeId(change: LocalBehaviorCaseChangeV1): string {
  return change.kind === "removed" ? change.original.case_id : change.revised.case_id;
}

function metricChangeId(change: LocalEvaluationMetricChangeV1): string {
  return change.kind === "removed" ? change.original.metric_id : change.revised.metric_id;
}

function asGraphNodeKind(value: unknown, field: string): LocalGraphNodeKindV1 {
  const kind = asKind(value, field);
  const supported = [
    "workspace", "project", "experiment", "context", "component", "prompt", "memory",
    "knowledge", "tool", "model", "evaluation", "workflow"
  ] as const;
  if (!supported.includes(kind as LocalGraphNodeKindV1)) throw new TypeError(`${field} is unsupported`);
  return kind as LocalGraphNodeKindV1;
}

function asGraphEdgeKind(value: unknown, field: string): LocalGraphEdgeKindV1 {
  const kind = asKind(value, field);
  const supported = ["owns", "contains", "configures", "retrieves", "uses", "evaluates", "produces", "tracks"] as const;
  if (!supported.includes(kind as LocalGraphEdgeKindV1)) throw new TypeError(`${field} is unsupported`);
  return kind as LocalGraphEdgeKindV1;
}

function asKind(value: unknown, field: string): string {
  return asCanonicalNonBlank(value, field);
}

function asRecord(value: unknown, field: string): ReadonlyRecord {
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw new TypeError(`${field} must be an object`);
  return value as ReadonlyRecord;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) throw new TypeError(`${field} must be an array`);
  return value;
}

function asString(value: unknown, field: string): string {
  if (typeof value !== "string") throw new TypeError(`${field} must be a string`);
  return value;
}

function asNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be non-blank`);
  }
  return value;
}

function asTimestamp(value: unknown, field: string): string {
  const timestamp = asString(value, field);
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?Z$/.exec(timestamp);
  if (!match) throw new TypeError(`${field} must be a valid timestamp`);

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

function asCanonicalNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0 || value.trim() !== value) {
    throw new TypeError(`${field} must be a canonical non-blank string`);
  }
  return value;
}

function asUuid(value: unknown, field: string): string {
  const parsed = asCanonicalNonBlank(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function asFiniteNumber(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new TypeError(`${field} must be finite`);
  return value;
}

function asNonNegativeInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) throw new TypeError(`${field} must be a non-negative integer`);
  return value;
}

function assertExactKeys(record: ReadonlyRecord, expected: readonly string[]): void {
  const actual = Object.keys(record).sort(compareStrings);
  const required = [...expected].sort(compareStrings);
  if (actual.length !== required.length || actual.some((key, index) => key !== required[index])) {
    throw new TypeError("persisted Context diff review response contains an unexpected shape");
  }
}

function assertExactKeysWithOptional(
  record: ReadonlyRecord,
  required: readonly string[],
  optional: readonly string[]
): void {
  const allowed = new Set([...required, ...optional]);
  const actual = Object.keys(record);
  if (required.some((key) => !Object.prototype.hasOwnProperty.call(record, key)) || actual.some((key) => !allowed.has(key))) {
    throw new TypeError("persisted Context diff review response contains an unexpected shape");
  }
}

function assertStrictlyAscending(values: readonly string[], field: string): void {
  for (let index = 1; index < values.length; index += 1) {
    if (compareStrings(values[index - 1]!, values[index]!) >= 0) throw new TypeError(`${field} must be unique and deterministically ordered`);
  }
}

function assertAscendingEdges(edges: readonly LocalGraphEdgeV1[], field: string): void {
  for (let index = 1; index < edges.length; index += 1) {
    if (compareEdges(edges[index - 1]!, edges[index]!) > 0) throw new TypeError(`${field} must be deterministically ordered`);
  }
}

function compareEdges(left: LocalGraphEdgeV1, right: LocalGraphEdgeV1): number {
  return compareStrings(left.source, right.source)
    || compareStrings(left.target, right.target)
    || edgeKindOrder(left.kind) - edgeKindOrder(right.kind);
}

function edgeKindOrder(kind: LocalGraphEdgeKindV1): number {
  return ["owns", "contains", "configures", "retrieves", "uses", "evaluates", "produces", "tracks"].indexOf(kind);
}

function compareStrings(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

async function parseErrorCode(response: Response): Promise<string> {
  let code: unknown;
  try {
    const value = await response.json();
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      code = (value as Record<string, unknown>).error;
    }
  } catch {
    // Only the status-based local error crosses this boundary on malformed JSON.
  }
  return typeof code === "string" && SAFE_ERROR_CODES.has(code)
    ? code
    : "contextlab_local_api_error";
}

const SAFE_ERROR_CODES = new Set([
  "contextlab_local_api_error",
  "context_read_forbidden",
  "authentication_required",
  "authentication_failed",
  "authorization_unavailable",
  "rate_limit_exceeded",
  "context_diff_snapshot_missing",
  "context_diff_review_scope_conflict",
  "persisted_context_diff_review_unavailable"
]);

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0) return seconds * 1_000;
  const timestamp = Date.parse(value);
  return Number.isNaN(timestamp) ? undefined : Math.max(0, timestamp - Date.now());
}

function freeze<T>(value: T): T {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value as Record<string, unknown>)) freeze(child);
    Object.freeze(value);
  }
  return value;
}
