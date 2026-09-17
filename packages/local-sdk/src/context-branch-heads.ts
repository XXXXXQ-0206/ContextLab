import type { LocalLifecycleReadCredentials } from "./types";
import type { ContextLabLocalClientOptions, FetchLike } from "./client";

export const LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1 =
  "contextlab.local-context-branch-heads.v1" as const;

type ReadonlyRecord = Readonly<Record<string, unknown>>;

export type LocalContextBranchHeadV1 = Readonly<{
  branch_name: string;
  head_commit_id: string | null;
  revision: number;
}>;

export type LocalContextBranchHeadsResourceV1 = Readonly<{
  schema_version: typeof LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1;
  context_id: string;
  branches: ReadonlyArray<LocalContextBranchHeadV1>;
}>;

export type LocalContextBranchHeadsResponseV1 = LocalContextBranchHeadsResourceV1;

export class ContextLabLocalBranchHeadError extends Error {
  readonly status: number;
  readonly code: string;
  readonly retryAfterMs?: number;

  constructor(
    status: number,
    body: Readonly<{ error: string; message: string }>,
    retryAfterMs?: number
  ) {
    super(body.message);
    this.name = "ContextLabLocalBranchHeadError";
    this.status = status;
    this.code = body.error;
    this.retryAfterMs = retryAfterMs;
  }
}

export class ContextLabLocalBranchHeadClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getContextBranchHeads(
    contextId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalContextBranchHeadsResourceV1> {
    const requestedContextId = asUuid(contextId, "contextId");
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/contexts/${encodeURIComponent(requestedContextId)}/branches`,
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
      throw new ContextLabLocalBranchHeadError(
        response.status,
        await parseErrorBody(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const resource = parseLocalContextBranchHeadsResourceV1(await response.json());
    if (resource.context_id !== requestedContextId) {
      throw new TypeError("context branch heads response does not match the requested scope");
    }
    return resource;
  }
}

export function parseLocalContextBranchHeadsResourceV1(
  value: unknown
): LocalContextBranchHeadsResourceV1 {
  const record = asRecord(value, "context branch heads response");
  assertExactKeys(record, ["schema_version", "context_id", "branches"]);
  if (record.schema_version !== LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1) {
    throw new TypeError(`schema_version must be ${LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1}`);
  }

  const branches = asArray(record.branches, "branches").map((item) => parseBranchHead(item));
  assertStrictlyAscending(branches.map((branch) => branch.branch_name), "branches");

  return freeze({
    schema_version: LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1,
    context_id: asUuid(record.context_id, "context_id"),
    branches
  });
}

export const parseLocalContextBranchHeadsResponseV1 =
  parseLocalContextBranchHeadsResourceV1;

function parseBranchHead(value: unknown): LocalContextBranchHeadV1 {
  const record = asRecord(value, "branch head");
  assertExactKeys(record, ["branch_name", "head_commit_id", "revision"]);

  return freeze({
    branch_name: asBranchName(record.branch_name),
    head_commit_id: record.head_commit_id === null
      ? null
      : asUuid(record.head_commit_id, "head_commit_id"),
    revision: asRevision(record.revision)
  });
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

function asBranchName(value: unknown): string {
  const parsed = asNonBlank(value, "branch_name");
  if (parsed !== parsed.trim() || !/^[A-Za-z0-9/_.-]+$/.test(parsed)) {
    throw new TypeError("branch_name contains unsupported characters");
  }
  return parsed;
}

function asRevision(value: unknown): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new TypeError("revision must be a non-negative safe integer");
  }
  return value;
}

function assertExactKeys(record: ReadonlyRecord, expected: readonly string[]): void {
  const actual = Object.keys(record).sort(compareStrings);
  const required = [...expected].sort(compareStrings);
  if (actual.length !== required.length || actual.some((key, index) => key !== required[index])) {
    throw new TypeError("context branch heads response contains an unexpected shape");
  }
}

function assertStrictlyAscending(values: readonly string[], field: string): void {
  for (let index = 1; index < values.length; index += 1) {
    if (compareStrings(values[index - 1]!, values[index]!) >= 0) {
      throw new TypeError(`${field} must be unique and deterministically ordered`);
    }
  }
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
  "context_branch_heads_unavailable"
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
