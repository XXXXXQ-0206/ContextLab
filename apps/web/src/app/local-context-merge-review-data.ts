export const LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1 = "v1" as const;

type ReadonlyRecord = Readonly<Record<string, unknown>>;

export type LocalContextMergeReviewScopeV1 = Readonly<{ project_id: string; context_id: string; commit_id: string }>;
export type LocalContextMergeReviewPlanV1 = Readonly<{ kind: "three_way"; base: string; left: string; right: string }>;
export type LocalContextMergeReviewEdgeKindV1 = "owns" | "contains" | "configures" | "retrieves" | "uses" | "evaluates" | "produces" | "tracks";
export type LocalContextMergeReviewChangeV1 =
  | Readonly<{ kind: "node"; node_id: string }>
  | Readonly<{ kind: "edge"; source: string; target: string; edge_kind: LocalContextMergeReviewEdgeKindV1 }>;
export type LocalContextMergeReviewClassificationV1 =
  | Readonly<{ kind: "clean"; changes: ReadonlyArray<LocalContextMergeReviewChangeV1> }>
  | Readonly<{ kind: "equivalent"; changes: ReadonlyArray<LocalContextMergeReviewChangeV1> }>
  | Readonly<{ kind: "conflict"; conflicts: ReadonlyArray<LocalContextMergeReviewChangeV1> }>;
export type LocalContextMergeReviewV1 = Readonly<{
  schema_version: typeof LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1;
  plan: LocalContextMergeReviewPlanV1;
  base_scope: LocalContextMergeReviewScopeV1;
  left_scope: LocalContextMergeReviewScopeV1;
  right_scope: LocalContextMergeReviewScopeV1;
  classification: LocalContextMergeReviewClassificationV1;
}>;

export type LocalContextMergeReviewTarget = Readonly<{ project_id: string; context_id: string; left_commit_id: string; right_commit_id: string }>;
export type LocalContextMergeReviewResource =
  | Readonly<{ kind: "loading" | "error" | "empty" | "unavailable"; target: LocalContextMergeReviewTarget; message?: string }>
  | Readonly<{ kind: "ready"; target: LocalContextMergeReviewTarget; review: LocalContextMergeReviewV1 }>;
export type LocalContextMergeReviewResourceInput =
  | Readonly<{ kind: "loading" | "error" | "empty" | "unavailable"; target: LocalContextMergeReviewTarget; message?: string }>
  | Readonly<{ kind: "ready"; target: LocalContextMergeReviewTarget; review: LocalContextMergeReviewV1 | ReadonlyRecord }>;

export class LocalContextMergeReviewProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<{ error: string; message: string }>;
  readonly retryAfterMs?: number;

  constructor(status: number, retryAfterMs?: number) {
    const message = `ContextLab local API request failed with status ${status}`;
    super(message);
    this.name = "LocalContextMergeReviewProxyError";
    this.status = status;
    this.body = Object.freeze({ error: "contextlab_web_api_error", message });
    this.retryAfterMs = retryAfterMs;
  }
}

export function parseLocalContextMergeReviewV1(value: unknown): LocalContextMergeReviewV1 {
  const record = asRecord(value, "local Context merge review");
  assertExactKeys(record, ["schema_version", "plan", "base_scope", "left_scope", "right_scope", "classification"]);
  if (record.schema_version !== LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1) throw new TypeError("schema_version must be v1");
  const plan = parsePlan(record.plan);
  const baseScope = parseScope(record.base_scope, "base_scope");
  const leftScope = parseScope(record.left_scope, "left_scope");
  const rightScope = parseScope(record.right_scope, "right_scope");
  if (
    baseScope.project_id !== leftScope.project_id
    || baseScope.project_id !== rightScope.project_id
    || baseScope.context_id !== leftScope.context_id
    || baseScope.context_id !== rightScope.context_id
  ) throw new TypeError("local Context merge review scopes must share project and context");
  if (plan.base !== baseScope.commit_id || plan.left !== leftScope.commit_id || plan.right !== rightScope.commit_id) {
    throw new TypeError("local Context merge review plan identities must match the exact scopes");
  }
  if (new Set([baseScope.commit_id, leftScope.commit_id, rightScope.commit_id]).size !== 3) {
    throw new TypeError("local Context merge review scopes must identify distinct commits");
  }
  return deepFreeze({
    schema_version: LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1,
    plan,
    base_scope: baseScope,
    left_scope: leftScope,
    right_scope: rightScope,
    classification: parseClassification(record.classification)
  });
}

export function createLocalContextMergeReviewResource(input: LocalContextMergeReviewResourceInput): LocalContextMergeReviewResource {
  const target = freezeTarget(input.target);
  return input.kind === "ready"
    ? Object.freeze({
      kind: "ready" as const,
      target,
      review: isNormalizedReview(input.review) ? deepFreeze(input.review) : parseLocalContextMergeReviewV1(input.review)
    })
    : Object.freeze({ ...input, target });
}

export function adaptLocalContextMergeReviewV1(resource: LocalContextMergeReviewResource): Readonly<{
  state: "loading" | "error" | "empty" | "unavailable" | "available";
  target: LocalContextMergeReviewTarget;
  review?: LocalContextMergeReviewV1;
}> {
  const target = freezeTarget(resource.target);
  if (resource.kind !== "ready") return Object.freeze({ state: resource.kind, target });
  assertTargetScope(resource.review, target);
  return Object.freeze({ state: "available" as const, target, review: resource.review });
}

export async function loadLocalContextMergeReview(target: LocalContextMergeReviewTarget, bearerToken: string): Promise<LocalContextMergeReviewV1> {
  const exactTarget = freezeTarget(target);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const query = new URLSearchParams({ left_commit_id: exactTarget.left_commit_id, right_commit_id: exactTarget.right_commit_id });
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(exactTarget.project_id)}/contexts/${encodeURIComponent(exactTarget.context_id)}/merge-review?${query.toString()}`,
    { method: "GET", credentials: "omit", cache: "no-store", headers: { accept: "application/json", authorization: `Bearer ${token}` } }
  );
  if (!response.ok) throw new LocalContextMergeReviewProxyError(response.status, parseRetryAfterMs(response.headers.get("retry-after")));
  const review = parseLocalContextMergeReviewV1(await response.json());
  assertTargetScope(review, exactTarget);
  return review;
}

function parsePlan(value: unknown): LocalContextMergeReviewPlanV1 {
  const record = asRecord(value, "plan");
  assertExactKeys(record, ["ThreeWay"]);
  const threeWay = asRecord(record.ThreeWay, "plan.ThreeWay");
  assertExactKeys(threeWay, ["base", "left", "right"]);
  return Object.freeze({ kind: "three_way", base: requireUuid(threeWay.base, "plan.ThreeWay.base"), left: requireUuid(threeWay.left, "plan.ThreeWay.left"), right: requireUuid(threeWay.right, "plan.ThreeWay.right") });
}

function isNormalizedReview(value: LocalContextMergeReviewV1 | ReadonlyRecord): value is LocalContextMergeReviewV1 {
  const plan = value.plan;
  return value.schema_version === LOCAL_CONTEXT_MERGE_REVIEW_SCHEMA_V1
    && plan !== null
    && typeof plan === "object"
    && !Array.isArray(plan)
    && (plan as ReadonlyRecord).kind === "three_way";
}

function parseScope(value: unknown, field: string): LocalContextMergeReviewScopeV1 {
  const record = asRecord(value, field);
  assertExactKeys(record, ["project_id", "context_id", "commit_id"]);
  return Object.freeze({ project_id: requireUuid(record.project_id, `${field}.project_id`), context_id: requireUuid(record.context_id, `${field}.context_id`), commit_id: requireUuid(record.commit_id, `${field}.commit_id`) });
}

function parseClassification(value: unknown): LocalContextMergeReviewClassificationV1 {
  const record = asRecord(value, "classification");
  const keys = Object.keys(record);
  if (keys.length !== 1 || !["Clean", "Equivalent", "Conflict"].includes(keys[0] ?? "")) throw new TypeError("classification must be Clean, Equivalent, or Conflict");
  const variant = keys[0]!;
  const body = asRecord(record[variant], `classification.${variant}`);
  if (variant === "Conflict") {
    assertExactKeys(body, ["conflicts"]);
    return Object.freeze({ kind: "conflict", conflicts: parseChanges(body.conflicts, "classification.Conflict.conflicts") });
  }
  assertExactKeys(body, ["changes"]);
  return Object.freeze({ kind: variant === "Clean" ? "clean" : "equivalent", changes: parseChanges(body.changes, `classification.${variant}.changes`) });
}

function parseChanges(value: unknown, field: string): ReadonlyArray<LocalContextMergeReviewChangeV1> {
  const changes = asArray(value, field).map((item) => parseChange(item, field));
  for (let index = 1; index < changes.length; index += 1) {
    if (compareChanges(changes[index - 1]!, changes[index]!) >= 0) throw new TypeError(`${field} must be unique and deterministically ordered`);
  }
  return Object.freeze(changes);
}

function parseChange(value: unknown, field: string): LocalContextMergeReviewChangeV1 {
  const record = asRecord(value, `${field} entry`);
  const keys = Object.keys(record);
  if (keys.length !== 1 || !["Node", "Edge"].includes(keys[0] ?? "")) throw new TypeError(`${field} entry has an unexpected shape`);
  const body = asRecord(record[keys[0]!], `${field}.${keys[0]}`);
  if (keys[0] === "Node") {
    assertExactKeys(body, ["node_id"]);
    return Object.freeze({ kind: "node", node_id: requireCanonicalNonBlank(body.node_id, `${field}.Node.node_id`) });
  }
  assertExactKeys(body, ["source", "target", "kind"]);
  const source = requireCanonicalNonBlank(body.source, `${field}.Edge.source`);
  const target = requireCanonicalNonBlank(body.target, `${field}.Edge.target`);
  if (source === target) throw new TypeError(`${field}.Edge must not be self-referential`);
  return Object.freeze({ kind: "edge", source, target, edge_kind: parseEdgeKind(body.kind, `${field}.Edge.kind`) });
}

function compareChanges(left: LocalContextMergeReviewChangeV1, right: LocalContextMergeReviewChangeV1): number {
  if (left.kind !== right.kind) return left.kind === "node" ? -1 : 1;
  if (left.kind === "node" && right.kind === "node") return compareStrings(left.node_id, right.node_id);
  if (left.kind !== "edge" || right.kind !== "edge") return 0;
  return compareStrings(left.source, right.source) || compareStrings(left.target, right.target) || edgeKindOrder(left.edge_kind) - edgeKindOrder(right.edge_kind);
}

function edgeKindOrder(kind: LocalContextMergeReviewEdgeKindV1): number {
  return ["owns", "contains", "configures", "retrieves", "uses", "evaluates", "produces", "tracks"].indexOf(kind);
}

function parseEdgeKind(value: unknown, field: string): LocalContextMergeReviewEdgeKindV1 {
  const kind = requireCanonicalNonBlank(value, field);
  if (!["owns", "contains", "configures", "retrieves", "uses", "evaluates", "produces", "tracks"].includes(kind)) throw new TypeError(`${field} is unsupported`);
  return kind as LocalContextMergeReviewEdgeKindV1;
}

function assertTargetScope(review: LocalContextMergeReviewV1, target: LocalContextMergeReviewTarget): void {
  if (review.base_scope.project_id !== target.project_id || review.base_scope.context_id !== target.context_id || review.left_scope.project_id !== target.project_id || review.left_scope.context_id !== target.context_id || review.right_scope.project_id !== target.project_id || review.right_scope.context_id !== target.context_id || review.left_scope.commit_id !== target.left_commit_id || review.right_scope.commit_id !== target.right_commit_id) throw new TypeError("Context merge review response is outside the requested scope");
}

function freezeTarget(target: LocalContextMergeReviewTarget): LocalContextMergeReviewTarget {
  const exact = { project_id: requireUuid(target.project_id, "target.project_id"), context_id: requireUuid(target.context_id, "target.context_id"), left_commit_id: requireUuid(target.left_commit_id, "target.left_commit_id"), right_commit_id: requireUuid(target.right_commit_id, "target.right_commit_id") };
  if (exact.left_commit_id === exact.right_commit_id) throw new RangeError("left and right commits must differ");
  return Object.freeze(exact);
}

function asRecord(value: unknown, field: string): ReadonlyRecord {
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw new TypeError(`${field} must be an object`);
  return value as ReadonlyRecord;
}
function asArray(value: unknown, field: string): unknown[] { if (!Array.isArray(value)) throw new TypeError(`${field} must be an array`); return value; }
function assertExactKeys(record: ReadonlyRecord, expected: readonly string[]): void { const actual = Object.keys(record).sort(compareStrings); const wanted = [...expected].sort(compareStrings); if (actual.length !== wanted.length || actual.some((key, index) => key !== wanted[index])) throw new TypeError("local Context merge review response contains an unexpected shape"); }
function compareStrings(left: string, right: string): number { return left < right ? -1 : left > right ? 1 : 0; }
function requireCanonicalNonBlank(value: unknown, field: string): string { if (typeof value !== "string" || value.length === 0 || value.trim() !== value) throw new TypeError(`${field} must be a canonical non-blank string`); return value; }
function requireNonBlank(value: unknown, field: string): string { if (typeof value !== "string" || value.trim().length === 0) throw new TypeError(`${field} must be non-blank`); return value; }
function requireUuid(value: unknown, field: string): string { const parsed = requireCanonicalNonBlank(value, field); if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) throw new TypeError(`${field} must be a lowercase canonical UUID`); return parsed; }
function parseRetryAfterMs(value: string | null): number | undefined { if (value === null) return undefined; const seconds = Number(value); return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined; }
function deepFreeze<T>(value: T): T { if (value !== null && typeof value === "object" && !Object.isFrozen(value)) { for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child); Object.freeze(value); } return value; }
