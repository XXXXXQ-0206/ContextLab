import type { ContextLabLocalClientOptions, FetchLike } from "./client";
import type { LocalLifecycleReadCredentials } from "./types";

export const LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1 = "v1" as const;

type ReadonlyRecord = Readonly<Record<string, unknown>>;

export type LocalContextMergeReviewScopeV1 = Readonly<{
  project_id: string;
  context_id: string;
  commit_id: string;
}>;

export type LocalContextMergeReviewPlanV1 = Readonly<{
  kind: "three_way";
  base: string;
  left: string;
  right: string;
}>;

export type LocalContextMergeReviewEdgeKindV1 =
  | "owns"
  | "contains"
  | "configures"
  | "retrieves"
  | "uses"
  | "evaluates"
  | "produces"
  | "tracks";

export type LocalContextMergeReviewChangeV1 =
  | Readonly<{ kind: "node"; node_id: string }>
  | Readonly<{
      kind: "edge";
      source: string;
      target: string;
      edge_kind: LocalContextMergeReviewEdgeKindV1;
    }>;

export type LocalContextMergeReviewClassificationV1 =
  | Readonly<{ kind: "clean"; changes: ReadonlyArray<LocalContextMergeReviewChangeV1> }>
  | Readonly<{ kind: "equivalent"; changes: ReadonlyArray<LocalContextMergeReviewChangeV1> }>
  | Readonly<{ kind: "conflict"; conflicts: ReadonlyArray<LocalContextMergeReviewChangeV1> }>;

export type LocalContextMergeReviewProjectionV1 = Readonly<{
  schema_version: typeof LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1;
  plan: LocalContextMergeReviewPlanV1;
  base_scope: LocalContextMergeReviewScopeV1;
  left_scope: LocalContextMergeReviewScopeV1;
  right_scope: LocalContextMergeReviewScopeV1;
  classification: LocalContextMergeReviewClassificationV1;
}>;

export class ContextLabLocalContextMergeReviewError extends Error {
  readonly status: number;
  readonly code: string;
  readonly retryAfterMs?: number;

  constructor(status: number, code: string, retryAfterMs?: number) {
    super(`ContextLab local API request failed with status ${status}`);
    this.name = "ContextLabLocalContextMergeReviewError";
    this.status = status;
    this.code = code;
    this.retryAfterMs = retryAfterMs;
  }
}

export class ContextLabLocalContextMergeReviewClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getContextMergeReview(
    projectId: string,
    contextId: string,
    leftCommitId: string,
    rightCommitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalContextMergeReviewProjectionV1> {
    const requestedProjectId = asUuid(projectId, "projectId");
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedLeftCommitId = asUuid(leftCommitId, "leftCommitId");
    const requestedRightCommitId = asUuid(rightCommitId, "rightCommitId");
    if (requestedLeftCommitId === requestedRightCommitId) {
      throw new RangeError("leftCommitId and rightCommitId must differ");
    }
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const query = new URLSearchParams({
      left_commit_id: requestedLeftCommitId,
      right_commit_id: requestedRightCommitId
    });
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/projects/${encodeURIComponent(requestedProjectId)}`
        + `/contexts/${encodeURIComponent(requestedContextId)}/merge-review?${query.toString()}`,
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
      throw new ContextLabLocalContextMergeReviewError(
        response.status,
        await parseErrorCode(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const review = parseLocalContextMergeReviewProjectionV1(await response.json());
    if (
      review.base_scope.project_id !== requestedProjectId
      || review.base_scope.context_id !== requestedContextId
      || review.left_scope.project_id !== requestedProjectId
      || review.left_scope.context_id !== requestedContextId
      || review.left_scope.commit_id !== requestedLeftCommitId
      || review.right_scope.project_id !== requestedProjectId
      || review.right_scope.context_id !== requestedContextId
      || review.right_scope.commit_id !== requestedRightCommitId
    ) {
      throw new TypeError("local Context merge review response does not match the requested scope");
    }
    return review;
  }
}

export function parseLocalContextMergeReviewProjectionV1(
  value: unknown
): LocalContextMergeReviewProjectionV1 {
  const record = asRecord(value, "local Context merge review");
  assertExactKeys(record, [
    "schema_version",
    "plan",
    "base_scope",
    "left_scope",
    "right_scope",
    "classification"
  ]);
  if (record.schema_version !== LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1) {
    throw new TypeError("schema_version must be v1");
  }

  const plan = parsePlan(record.plan);
  const baseScope = parseScope(record.base_scope, "base_scope");
  const leftScope = parseScope(record.left_scope, "left_scope");
  const rightScope = parseScope(record.right_scope, "right_scope");
  if (
    baseScope.project_id !== leftScope.project_id
    || baseScope.project_id !== rightScope.project_id
    || baseScope.context_id !== leftScope.context_id
    || baseScope.context_id !== rightScope.context_id
  ) {
    throw new TypeError("local Context merge review scopes must share project and context");
  }
  if (
    plan.base !== baseScope.commit_id
    || plan.left !== leftScope.commit_id
    || plan.right !== rightScope.commit_id
  ) {
    throw new TypeError("local Context merge review plan identities must match the exact scopes");
  }
  if (new Set([baseScope.commit_id, leftScope.commit_id, rightScope.commit_id]).size !== 3) {
    throw new TypeError("local Context merge review scopes must identify distinct commits");
  }

  return freeze({
    schema_version: LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1,
    plan,
    base_scope: baseScope,
    left_scope: leftScope,
    right_scope: rightScope,
    classification: parseClassification(record.classification)
  });
}

export const parseLocalContextMergeReviewV1 = parseLocalContextMergeReviewProjectionV1;

function parsePlan(value: unknown): LocalContextMergeReviewPlanV1 {
  const record = asRecord(value, "plan");
  assertExactKeys(record, ["ThreeWay"]);
  const threeWay = asRecord(record.ThreeWay, "plan.ThreeWay");
  assertExactKeys(threeWay, ["base", "left", "right"]);
  return freeze({
    kind: "three_way",
    base: asUuid(threeWay.base, "plan.ThreeWay.base"),
    left: asUuid(threeWay.left, "plan.ThreeWay.left"),
    right: asUuid(threeWay.right, "plan.ThreeWay.right")
  });
}

function parseScope(value: unknown, field: string): LocalContextMergeReviewScopeV1 {
  const record = asRecord(value, field);
  assertExactKeys(record, ["project_id", "context_id", "commit_id"]);
  return freeze({
    project_id: asUuid(record.project_id, `${field}.project_id`),
    context_id: asUuid(record.context_id, `${field}.context_id`),
    commit_id: asUuid(record.commit_id, `${field}.commit_id`)
  });
}

function parseClassification(value: unknown): LocalContextMergeReviewClassificationV1 {
  const record = asRecord(value, "classification");
  const keys = Object.keys(record);
  if (keys.length !== 1 || !["Clean", "Equivalent", "Conflict"].includes(keys[0] ?? "")) {
    throw new TypeError("classification must be clean, equivalent, or conflict");
  }

  const variant = keys[0]!;
  const body = asRecord(record[variant], `classification.${variant}`);
  if (variant === "Conflict") {
    assertExactKeys(body, ["conflicts"]);
    return freeze({
      kind: "conflict",
      conflicts: parseChanges(body.conflicts, "classification.Conflict.conflicts")
    });
  }

  assertExactKeys(body, ["changes"]);
  const changes = parseChanges(body.changes, `classification.${variant}.changes`);
  return freeze({
    kind: variant === "Clean" ? "clean" : "equivalent",
    changes
  });
}

function parseChanges(value: unknown, field: string): ReadonlyArray<LocalContextMergeReviewChangeV1> {
  const changes = asArray(value, field).map((item) => parseChange(item, field));
  for (let index = 1; index < changes.length; index += 1) {
    if (compareChanges(changes[index - 1]!, changes[index]!) >= 0) {
      throw new TypeError(`${field} must be unique and deterministically ordered`);
    }
  }
  return freeze(changes);
}

function parseChange(value: unknown, field: string): LocalContextMergeReviewChangeV1 {
  const record = asRecord(value, `${field} entry`);
  const keys = Object.keys(record);
  if (keys.length !== 1 || !["Node", "Edge"].includes(keys[0] ?? "")) {
    throw new TypeError(`${field} entry has an unexpected shape`);
  }

  const variant = keys[0]!;
  const body = asRecord(record[variant], `${field}.${variant}`);
  if (variant === "Node") {
    assertExactKeys(body, ["node_id"]);
    return freeze({
      kind: "node",
      node_id: asCanonicalNonBlank(body.node_id, `${field}.Node.node_id`)
    });
  }

  assertExactKeys(body, ["source", "target", "kind"]);
  const source = asCanonicalNonBlank(body.source, `${field}.Edge.source`);
  const target = asCanonicalNonBlank(body.target, `${field}.Edge.target`);
  if (source === target) {
    throw new TypeError(`${field}.Edge must not be self-referential`);
  }
  return freeze({
    kind: "edge",
    source,
    target,
    edge_kind: asEdgeKind(body.kind, `${field}.Edge.kind`)
  });
}

function compareChanges(left: LocalContextMergeReviewChangeV1, right: LocalContextMergeReviewChangeV1): number {
  if (left.kind !== right.kind) return left.kind === "node" ? -1 : 1;
  if (left.kind === "node" && right.kind === "node") {
    return compareStrings(left.node_id, right.node_id);
  }
  if (left.kind !== "edge" || right.kind !== "edge") return 0;
  return compareStrings(left.source, right.source)
    || compareStrings(left.target, right.target)
    || edgeKindOrder(left.edge_kind) - edgeKindOrder(right.edge_kind);
}

function edgeKindOrder(kind: LocalContextMergeReviewEdgeKindV1): number {
  return ["owns", "contains", "configures", "retrieves", "uses", "evaluates", "produces", "tracks"]
    .indexOf(kind);
}

function asEdgeKind(value: unknown, field: string): LocalContextMergeReviewEdgeKindV1 {
  if (
    value !== "owns"
    && value !== "contains"
    && value !== "configures"
    && value !== "retrieves"
    && value !== "uses"
    && value !== "evaluates"
    && value !== "produces"
    && value !== "tracks"
  ) {
    throw new TypeError(`${field} is unsupported`);
  }
  return value;
}

function asRecord(value: unknown, field: string): ReadonlyRecord {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as ReadonlyRecord;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) throw new TypeError(`${field} must be an array`);
  return value;
}

function asCanonicalNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.length === 0 || value.trim() !== value) {
    throw new TypeError(`${field} must be a canonical non-blank string`);
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
  const parsed = asCanonicalNonBlank(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function assertExactKeys(record: ReadonlyRecord, expected: readonly string[]): void {
  const actual = Object.keys(record).sort(compareStrings);
  const required = [...expected].sort(compareStrings);
  if (actual.length !== required.length || actual.some((key, index) => key !== required[index])) {
    throw new TypeError("local Context merge review response contains an unexpected shape");
  }
}

function compareStrings(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function freeze<T>(value: T): Readonly<T> {
  return Object.freeze(value);
}

async function parseErrorCode(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as Partial<{ error: unknown }>;
    if (typeof body.error === "string" && body.error.trim().length > 0) return body.error;
  } catch {
  }
  return "contextlab_local_api_error";
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const delaySeconds = Number(value);
  if (Number.isFinite(delaySeconds) && delaySeconds >= 0) return delaySeconds * 1_000;
  const retryAt = Date.parse(value);
  return Number.isNaN(retryAt) ? undefined : Math.max(0, retryAt - Date.now());
}
