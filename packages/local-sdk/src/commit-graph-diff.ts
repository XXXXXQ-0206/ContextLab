import type { LocalLifecycleReadCredentials } from "./types";
import type { ContextLabLocalClientOptions, FetchLike } from "./client";

export const LOCAL_COMMIT_GRAPH_DIFF_READ_SCHEMA_V1 =
  "contextlab.local-commit-graph-diff-read.v1" as const;

type ReadonlyRecord = Readonly<Record<string, unknown>>;

export type LocalGraphNodeKindV1 =
  | "workspace"
  | "project"
  | "experiment"
  | "context"
  | "component"
  | "prompt"
  | "memory"
  | "knowledge"
  | "tool"
  | "model"
  | "evaluation"
  | "workflow";

export type LocalGraphEdgeKindV1 =
  | "owns"
  | "contains"
  | "configures"
  | "retrieves"
  | "uses"
  | "evaluates"
  | "produces"
  | "tracks";

export type LocalCommitGraphSnapshotReferenceV1 = Readonly<{
  commit_id: string;
  captured_at: string;
  schema_version: 1;
}>;

export type LocalCommitGraphDiffNodeV1 = Readonly<{
  id: string;
  kind: LocalGraphNodeKindV1;
  label: string;
}>;

export type LocalCommitGraphDiffNodeChangeV1 = Readonly<{
  node_id: string;
  original_kind: LocalGraphNodeKindV1;
  revised_kind: LocalGraphNodeKindV1;
  original_label: string;
  revised_label: string;
}>;

export type LocalCommitGraphDiffPairWitnessV1 = Readonly<{
  schema_version: 1;
  project_id: string;
  context_id: string;
  baseline_commit_id: string;
  revised_commit_id: string;
}>;

export type LocalCommitGraphDiffEdgeV1 = Readonly<{
  source: string;
  target: string;
  kind: LocalGraphEdgeKindV1;
}>;

export type LocalCommitGraphDiffProjectionV1 = Readonly<{
  added_nodes: ReadonlyArray<LocalCommitGraphDiffNodeV1>;
  removed_nodes: ReadonlyArray<LocalCommitGraphDiffNodeV1>;
  modified_nodes: ReadonlyArray<LocalCommitGraphDiffNodeChangeV1>;
  added_edges: ReadonlyArray<LocalCommitGraphDiffEdgeV1>;
  removed_edges: ReadonlyArray<LocalCommitGraphDiffEdgeV1>;
}>;

export type LocalCommitGraphDiffResponseV1 = Readonly<{
  context_id: string;
  pair_witness: LocalCommitGraphDiffPairWitnessV1;
  original: LocalCommitGraphSnapshotReferenceV1;
  revised: LocalCommitGraphSnapshotReferenceV1;
  diff: LocalCommitGraphDiffProjectionV1;
}>;

export class ContextLabLocalCommitGraphDiffClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getCommitGraphDiff(
    contextId: string,
    originalCommitId: string,
    revisedCommitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalCommitGraphDiffResponseV1> {
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedOriginalCommitId = asUuid(originalCommitId, "originalCommitId");
    const requestedRevisedCommitId = asUuid(revisedCommitId, "revisedCommitId");
    if (requestedOriginalCommitId === requestedRevisedCommitId) {
      throw new RangeError("originalCommitId and revisedCommitId must differ");
    }
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const query = new URLSearchParams({
      original_commit_id: requestedOriginalCommitId,
      revised_commit_id: requestedRevisedCommitId
    });
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/contexts/${encodeURIComponent(requestedContextId)}`
        + `/graph-diff?${query.toString()}`,
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
      throw new ContextLabLocalCommitGraphDiffError(
        response.status,
        await parseErrorBody(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const result = parseLocalCommitGraphDiffResponseV1(await response.json());
    if (
      result.context_id !== requestedContextId
      || result.original.commit_id !== requestedOriginalCommitId
      || result.revised.commit_id !== requestedRevisedCommitId
    ) {
      throw new TypeError("commit graph diff response does not match the requested scope");
    }
    return result;
  }
}

export class ContextLabLocalCommitGraphDiffError extends Error {
  readonly status: number;
  readonly code: string;
  readonly retryAfterMs?: number;

  constructor(
    status: number,
    body: Readonly<{ error: string; message: string }>,
    retryAfterMs?: number
  ) {
    super(body.message);
    this.name = "ContextLabLocalCommitGraphDiffError";
    this.status = status;
    this.code = body.error;
    this.retryAfterMs = retryAfterMs;
  }
}

export function parseLocalCommitGraphDiffResponseV1(value: unknown): LocalCommitGraphDiffResponseV1 {
  const record = asRecord(value, "commit graph diff response");
  assertExactKeys(record, ["context_id", "pair_witness", "original", "revised", "diff"]);
  const contextId = asUuid(record.context_id, "context_id");
  const pairWitness = parsePairWitness(record.pair_witness);
  const original = parseSnapshotReference(record.original, "original");
  const revised = parseSnapshotReference(record.revised, "revised");
  if (original.commit_id === revised.commit_id) {
    throw new TypeError("commit graph diff response must compare distinct commits");
  }
  if (
    pairWitness.context_id !== contextId
    || pairWitness.baseline_commit_id !== original.commit_id
    || pairWitness.revised_commit_id !== revised.commit_id
  ) {
    throw new TypeError("commit graph diff response contains a mixed scope");
  }

  return freeze({
    context_id: contextId,
    pair_witness: pairWitness,
    original,
    revised,
    diff: parseGraphDiff(record.diff)
  });
}

function parsePairWitness(value: unknown): LocalCommitGraphDiffPairWitnessV1 {
  const record = asRecord(value, "pair_witness");
  assertExactKeys(record, [
    "schema_version",
    "project_id",
    "context_id",
    "baseline_commit_id",
    "revised_commit_id"
  ]);
  if (record.schema_version !== 1) {
    throw new TypeError("pair_witness.schema_version must be 1");
  }

  const baselineCommitId = asUuid(
    record.baseline_commit_id,
    "pair_witness.baseline_commit_id"
  );
  const revisedCommitId = asUuid(
    record.revised_commit_id,
    "pair_witness.revised_commit_id"
  );
  if (baselineCommitId === revisedCommitId) {
    throw new TypeError("pair_witness must compare distinct commits");
  }

  return freeze({
    schema_version: 1 as const,
    project_id: asUuid(record.project_id, "pair_witness.project_id"),
    context_id: asUuid(record.context_id, "pair_witness.context_id"),
    baseline_commit_id: baselineCommitId,
    revised_commit_id: revisedCommitId
  });
}

function parseSnapshotReference(
  value: unknown,
  field: string
): LocalCommitGraphSnapshotReferenceV1 {
  const record = asRecord(value, field);
  assertExactKeys(record, ["commit_id", "captured_at", "schema_version"]);
  if (record.schema_version !== 1) {
    throw new TypeError(`${field}.schema_version must be 1`);
  }

  return freeze({
    commit_id: asUuid(record.commit_id, `${field}.commit_id`),
    captured_at: asTimestamp(record.captured_at, `${field}.captured_at`),
    schema_version: 1 as const
  });
}

function parseGraphDiff(value: unknown): LocalCommitGraphDiffProjectionV1 {
  const record = asRecord(value, "graph diff");
  assertExactKeys(record, [
    "added_nodes",
    "removed_nodes",
    "modified_nodes",
    "added_edges",
    "removed_edges"
  ]);
  const addedNodes = asArray(record.added_nodes, "diff.added_nodes").map(parseNode);
  const removedNodes = asArray(record.removed_nodes, "diff.removed_nodes").map(parseNode);
  const modifiedNodes = asArray(record.modified_nodes, "diff.modified_nodes").map(parseNodeChange);
  const addedEdges = asArray(record.added_edges, "diff.added_edges").map(parseEdge);
  const removedEdges = asArray(record.removed_edges, "diff.removed_edges").map(parseEdge);
  assertStrictlyAscending(addedNodes.map((node) => node.id), "diff.added_nodes");
  assertStrictlyAscending(removedNodes.map((node) => node.id), "diff.removed_nodes");
  assertStrictlyAscending(modifiedNodes.map((node) => node.node_id), "diff.modified_nodes");
  assertAscendingEdges(addedEdges, "diff.added_edges");
  assertAscendingEdges(removedEdges, "diff.removed_edges");
  const nodeIds = [
    ...addedNodes.map((node) => node.id),
    ...removedNodes.map((node) => node.id),
    ...modifiedNodes.map((node) => node.node_id)
  ];
  if (new Set(nodeIds).size !== nodeIds.length) {
    throw new TypeError("diff node categories must be disjoint");
  }

  return freeze({
    added_nodes: Object.freeze(addedNodes),
    removed_nodes: Object.freeze(removedNodes),
    modified_nodes: Object.freeze(modifiedNodes),
    added_edges: Object.freeze(addedEdges),
    removed_edges: Object.freeze(removedEdges)
  });
}

function parseNode(value: unknown): LocalCommitGraphDiffNodeV1 {
  const record = asRecord(value, "graph diff node");
  assertExactKeys(record, ["id", "kind", "label"]);
  return freeze({
    id: asNonBlank(record.id, "node.id"),
    kind: asGraphNodeKind(record.kind, "node.kind"),
    label: asNonBlank(record.label, "node.label")
  });
}

function parseNodeChange(value: unknown): LocalCommitGraphDiffNodeChangeV1 {
  const record = asRecord(value, "graph diff node change");
  assertExactKeys(record, [
    "node_id",
    "original_kind",
    "revised_kind",
    "original_label",
    "revised_label"
  ]);
  const originalKind = asGraphNodeKind(record.original_kind, "node change.original_kind");
  const revisedKind = asGraphNodeKind(record.revised_kind, "node change.revised_kind");
  const originalLabel = asNonBlank(record.original_label, "node change.original_label");
  const revisedLabel = asNonBlank(record.revised_label, "node change.revised_label");
  if (originalKind === revisedKind && originalLabel === revisedLabel) {
    throw new TypeError("graph diff node change must contain a change");
  }
  return freeze({
    node_id: asNonBlank(record.node_id, "node change.node_id"),
    original_kind: originalKind,
    revised_kind: revisedKind,
    original_label: originalLabel,
    revised_label: revisedLabel
  });
}

function parseEdge(value: unknown): LocalCommitGraphDiffEdgeV1 {
  const record = asRecord(value, "graph diff edge");
  assertExactKeys(record, ["source", "target", "kind"]);
  return freeze({
    source: asNonBlank(record.source, "edge.source"),
    target: asNonBlank(record.target, "edge.target"),
    kind: asGraphEdgeKind(record.kind, "edge.kind")
  });
}

function asGraphNodeKind(value: unknown, field: string): LocalGraphNodeKindV1 {
  const kind = asNonBlank(value, field);
  const supported: readonly LocalGraphNodeKindV1[] = [
    "workspace", "project", "experiment", "context", "component", "prompt", "memory",
    "knowledge", "tool", "model", "evaluation", "workflow"
  ];
  if (!supported.includes(kind as LocalGraphNodeKindV1)) {
    throw new TypeError(`${field} is unsupported`);
  }
  return kind as LocalGraphNodeKindV1;
}

function asGraphEdgeKind(value: unknown, field: string): LocalGraphEdgeKindV1 {
  const kind = asNonBlank(value, field);
  const supported: readonly LocalGraphEdgeKindV1[] = [
    "owns", "contains", "configures", "retrieves", "uses", "evaluates", "produces", "tracks"
  ];
  if (!supported.includes(kind as LocalGraphEdgeKindV1)) {
    throw new TypeError(`${field} is unsupported`);
  }
  return kind as LocalGraphEdgeKindV1;
}

function assertAscendingEdges(
  edges: readonly LocalCommitGraphDiffEdgeV1[],
  field: string
): void {
  for (let index = 1; index < edges.length; index += 1) {
    if (compareEdge(edges[index - 1]!, edges[index]!) > 0) {
      throw new TypeError(`${field} must be deterministically ordered`);
    }
  }
}

function compareEdge(left: LocalCommitGraphDiffEdgeV1, right: LocalCommitGraphDiffEdgeV1): number {
  return compareStrings(left.source, right.source)
    || compareStrings(left.target, right.target)
    || graphEdgeKindOrder(left.kind) - graphEdgeKindOrder(right.kind);
}

function graphEdgeKindOrder(kind: LocalGraphEdgeKindV1): number {
  return ["owns", "contains", "configures", "retrieves", "uses", "evaluates", "produces", "tracks"]
    .indexOf(kind);
}

function assertStrictlyAscending(values: readonly string[], field: string): void {
  for (let index = 1; index < values.length; index += 1) {
    if (compareStrings(values[index - 1]!, values[index]!) >= 0) {
      throw new TypeError(`${field} must be unique and deterministically ordered`);
    }
  }
}

function assertExactKeys(record: ReadonlyRecord, expected: readonly string[]): void {
  const actual = Object.keys(record).sort(compareStrings);
  const required = [...expected].sort(compareStrings);
  if (actual.length !== required.length || actual.some((key, index) => key !== required[index])) {
    throw new TypeError("commit graph diff response contains an unexpected shape");
  }
}

function asRecord(value: unknown, field: string): ReadonlyRecord {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as ReadonlyRecord;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new TypeError(`${field} must be an array`);
  }
  return value;
}

function asNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be non-blank`);
  }
  return value;
}

function asUuid(value: unknown, field: string): string {
  const parsed = asNonBlank(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function asTimestamp(value: unknown, field: string): string {
  const timestamp = asNonBlank(value, field);
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(?:Z|\+00:00)$/.exec(timestamp);
  if (!match) {
    throw new TypeError(`${field} must be a UTC RFC3339 timestamp`);
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
    throw new TypeError(`${field} must be a UTC RFC3339 timestamp`);
  }
  return timestamp;
}

async function parseErrorBody(response: Response): Promise<Readonly<{ error: string; message: string }>> {
  let code: unknown;
  try {
    const value = await response.json();
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      code = (value as Record<string, unknown>).error;
    }
  } catch {
    // Only a status-based error crosses this local SDK boundary on malformed JSON.
  }
  return freeze({
    error: typeof code === "string" && SAFE_ERROR_CODES.has(code)
      ? code
      : "contextlab_local_api_error",
    message: `ContextLab local API request failed with status ${response.status}`
  });
}

const SAFE_ERROR_CODES = new Set([
  "contextlab_local_api_error",
  "context_read_forbidden",
  "authentication_required",
  "authentication_failed",
  "authorization_unavailable",
  "rate_limit_exceeded",
  "invalid_commit_graph_diff_query",
  "commit_graph_snapshot_missing",
  "commit_graph_snapshot_scope_conflict",
  "commit_graph_diff_unavailable",
  "storage_scope_unavailable"
]);

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0) return seconds * 1_000;
  const timestamp = Date.parse(value);
  return Number.isNaN(timestamp) ? undefined : Math.max(0, timestamp - Date.now());
}

function compareStrings(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function freeze<T>(value: T): T {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value as Record<string, unknown>)) freeze(child);
    Object.freeze(value);
  }
  return value;
}
