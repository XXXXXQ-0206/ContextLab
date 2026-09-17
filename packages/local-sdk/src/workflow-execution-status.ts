import type { LocalLifecycleReadCredentials } from "./types";
import type { ContextLabLocalClientOptions, FetchLike } from "./client";

export const LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1 =
  "contextlab.local-workflow-execution-status.v1" as const;

const COUNT_FIELDS = ["pending", "running", "succeeded", "failed", "blocked"] as const;
const RUN_STATES = ["pending", "running", "succeeded", "failed"] as const;

type ReadonlyRecord = Readonly<Record<string, unknown>>;

export type LocalWorkflowExecutionNodeStatusCountsV1 = Readonly<{
  pending: number;
  running: number;
  succeeded: number;
  failed: number;
  blocked: number;
}>;

export type LocalWorkflowExecutionStatusV1 = Readonly<{
  schema_version: typeof LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1;
  context_id: string;
  context_commit_id: string;
  binding_id: string;
  workflow_id: string;
  workflow_revision: number;
  run_id: string;
  replay_of: string | null;
  run_state: (typeof RUN_STATES)[number];
  event_count: number;
  last_event_sequence: number;
  capability_snapshot_digest: string;
  node_status_counts: LocalWorkflowExecutionNodeStatusCountsV1;
}>;

export type LocalWorkflowExecutionStatusScope = Readonly<{
  context_id: string;
  context_commit_id: string;
  binding_id: string;
  workflow_id: string;
  workflow_revision: number;
  run_id: string;
}>;

export class ContextLabLocalWorkflowExecutionStatusError extends Error {
  readonly status: number;
  readonly code: string;
  readonly retryAfterMs?: number;

  constructor(
    status: number,
    body: Readonly<{ error: string; message: string }>,
    retryAfterMs?: number
  ) {
    super(body.message);
    this.name = "ContextLabLocalWorkflowExecutionStatusError";
    this.status = status;
    this.code = body.error;
    this.retryAfterMs = retryAfterMs;
  }
}

export class ContextLabLocalWorkflowExecutionStatusClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getWorkflowExecutionStatus(
    contextId: string,
    runId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalWorkflowExecutionStatusV1> {
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedRunId = asUuid(runId, "runId");
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/contexts/${encodeURIComponent(requestedContextId)}`
        + `/workflow/runs/${encodeURIComponent(requestedRunId)}/status`,
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
      throw new ContextLabLocalWorkflowExecutionStatusError(
        response.status,
        await parseErrorBody(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const status = parseLocalWorkflowExecutionStatusV1(await response.json());
    assertLocalWorkflowExecutionStatusScope(status, {
      context_id: requestedContextId,
      context_commit_id: status.context_commit_id,
      binding_id: status.binding_id,
      workflow_id: status.workflow_id,
      workflow_revision: status.workflow_revision,
      run_id: requestedRunId
    });
    return status;
  }
}

export function parseLocalWorkflowExecutionStatusV1(
  value: unknown
): LocalWorkflowExecutionStatusV1 {
  const record = asRecord(value, "local workflow execution status");
  assertNoRawFields(record);
  assertExactKeys(record, [
    "schema_version",
    "context_id",
    "context_commit_id",
    "binding_id",
    "workflow_id",
    "workflow_revision",
    "run_id",
    "replay_of",
    "run_state",
    "event_count",
    "last_event_sequence",
    "capability_snapshot_digest",
    "node_status_counts"
  ]);
  if (record.schema_version !== LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1) {
    throw new TypeError(
      `schema_version must be ${LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1}`
    );
  }

  const runId = asUuid(record.run_id, "run_id");
  const replayOf = record.replay_of === null
    ? null
    : asUuid(record.replay_of, "replay_of");
  if (replayOf === runId) {
    throw new TypeError("replay_of must identify a distinct source run");
  }

  return freeze({
    schema_version: LOCAL_WORKFLOW_EXECUTION_STATUS_SCHEMA_V1,
    context_id: asUuid(record.context_id, "context_id"),
    context_commit_id: asUuid(record.context_commit_id, "context_commit_id"),
    binding_id: asUuid(record.binding_id, "binding_id"),
    workflow_id: asUuid(record.workflow_id, "workflow_id"),
    workflow_revision: asPositiveInteger(record.workflow_revision, "workflow_revision"),
    run_id: runId,
    replay_of: replayOf,
    run_state: asRunState(record.run_state),
    event_count: asNonNegativeInteger(record.event_count, "event_count"),
    last_event_sequence: asNonNegativeInteger(
      record.last_event_sequence,
      "last_event_sequence"
    ),
    capability_snapshot_digest: asSafeOpaqueString(
      record.capability_snapshot_digest,
      "capability_snapshot_digest"
    ),
    node_status_counts: parseNodeStatusCounts(record.node_status_counts)
  });
}

export function assertLocalWorkflowExecutionStatusScope(
  status: LocalWorkflowExecutionStatusV1,
  scope: LocalWorkflowExecutionStatusScope
): void {
  const requested = {
    context_id: asUuid(scope.context_id, "scope.context_id"),
    context_commit_id: asUuid(scope.context_commit_id, "scope.context_commit_id"),
    binding_id: asUuid(scope.binding_id, "scope.binding_id"),
    workflow_id: asUuid(scope.workflow_id, "scope.workflow_id"),
    workflow_revision: asPositiveInteger(scope.workflow_revision, "scope.workflow_revision"),
    run_id: asUuid(scope.run_id, "scope.run_id")
  };

  if (
    status.context_id !== requested.context_id
    || status.context_commit_id !== requested.context_commit_id
    || status.binding_id !== requested.binding_id
    || status.workflow_id !== requested.workflow_id
    || status.workflow_revision !== requested.workflow_revision
    || status.run_id !== requested.run_id
  ) {
    throw new TypeError("workflow execution status response does not match the requested scope or run");
  }
}

function parseNodeStatusCounts(value: unknown): LocalWorkflowExecutionNodeStatusCountsV1 {
  const record = asRecord(value, "node_status_counts");
  assertExactKeys(record, COUNT_FIELDS);
  assertDeterministicCountOrder(record);
  assertNoRawFields(record);

  return freeze({
    pending: asNonNegativeInteger(record.pending, "node_status_counts.pending"),
    running: asNonNegativeInteger(record.running, "node_status_counts.running"),
    succeeded: asNonNegativeInteger(record.succeeded, "node_status_counts.succeeded"),
    failed: asNonNegativeInteger(record.failed, "node_status_counts.failed"),
    blocked: asNonNegativeInteger(record.blocked, "node_status_counts.blocked")
  });
}

function assertDeterministicCountOrder(record: ReadonlyRecord): void {
  const actual = Object.keys(record);
  if (actual.some((key, index) => key !== COUNT_FIELDS[index])) {
    throw new TypeError("node_status_counts must use the deterministic core field order");
  }
}

function assertNoRawFields(record: ReadonlyRecord): void {
  const forbidden = new Set([
    "events",
    "event",
    "failure",
    "failure_code",
    "provider",
    "provider_request",
    "provider_response",
    "secret",
    "token",
    "api_key",
    "credentials"
  ]);
  if (Object.keys(record).some((key) => forbidden.has(key))) {
    throw new TypeError("workflow execution status contains forbidden raw or secret fields");
  }
}

function assertExactKeys(record: ReadonlyRecord, expected: readonly string[]): void {
  const actual = Object.keys(record).sort(compareStrings);
  const sortedExpected = [...expected].sort(compareStrings);
  if (
    actual.length !== sortedExpected.length
    || actual.some((key, index) => key !== sortedExpected[index])
  ) {
    throw new TypeError("local workflow execution status contains an unexpected shape");
  }
}

function asRecord(value: unknown, field: string): ReadonlyRecord {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as ReadonlyRecord;
}

function asUuid(value: unknown, field: string): string {
  const parsed = asNonBlank(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function asNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be non-blank`);
  }
  return value;
}

function asSafeOpaqueString(value: unknown, field: string): string {
  const parsed = asNonBlank(value, field);
  if (parsed !== parsed.trim() || parsed.length > 512 || !/^[\x21-\x7e]+$/.test(parsed)) {
    throw new TypeError(`${field} must be a safe opaque string`);
  }
  return parsed;
}

function asPositiveInteger(value: unknown, field: string): number {
  const parsed = asNonNegativeInteger(value, field);
  if (parsed === 0) {
    throw new TypeError(`${field} must be a positive integer`);
  }
  return parsed;
}

function asNonNegativeInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${field} must be a non-negative integer`);
  }
  return value;
}

function asRunState(value: unknown): LocalWorkflowExecutionStatusV1["run_state"] {
  if (!RUN_STATES.includes(value as (typeof RUN_STATES)[number])) {
    throw new TypeError("run_state is unsupported");
  }
  return value as LocalWorkflowExecutionStatusV1["run_state"];
}

async function parseErrorBody(
  response: Response
): Promise<Readonly<{ error: string; message: string }>> {
  let code: unknown;
  try {
    const value = await response.json();
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      code = (value as Record<string, unknown>).error;
    }
  } catch {
    // Only a redacted status-based error crosses this local SDK boundary.
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
  "workflow_execution_status_unavailable",
  "workflow_execution_status_not_found"
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
