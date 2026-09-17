import {
  ContextLabLocalApiError,
  type ContextLabLocalClientOptions,
  type FetchLike
} from "./client";
import {
  knowledgeMemoryReplayProjectionId,
  knowledgeMemoryScopeForContext
} from "./knowledge-memory-identity";
import type { LocalApiErrorBody, LocalLifecycleReadCredentials } from "./types";

export const LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1 =
  "contextlab.local-knowledge-memory-projection.v1" as const;
export const KNOWLEDGE_MEMORY_REPLAY_PROJECTION_SCHEMA_V1 =
  "knowledge-memory-local-replay-v2" as const;

type ReadonlyRecord = Readonly<Record<string, unknown>>;

export type LocalKnowledgeCitationRangeV1 = Readonly<{
  start_byte: number;
  end_byte: number;
}>;

export type LocalKnowledgeCitationV1 = Readonly<{
  document_id: string;
  document_revision_id: string;
  chunk_id: string;
  source_version: string;
  chunking_version: string;
  ordinal: number;
  range: LocalKnowledgeCitationRangeV1;
  content_fingerprint: string;
}>;

export type LocalKnowledgeMemoryReplayProjectionV1 = Readonly<{
  id: string;
  schema_version: typeof KNOWLEDGE_MEMORY_REPLAY_PROJECTION_SCHEMA_V1;
  citation_projection_id: string;
  citation_projection_schema_version: "knowledge-local-citation-projection-v1";
  knowledge_scope: string;
  retrieval_id: string;
  retrieval_version: string;
  citations: ReadonlyArray<LocalKnowledgeCitationV1>;
  memory_id: string;
  memory_scope: string;
  memory_timeline_version: number;
  memory_capability_schema_version: "memory-retention-capability-v1";
  retention_policy_version: string;
  retention_decision: LocalRetentionDecision;
  replay_state: LocalReplayState;
}>;

export type LocalKnowledgeMemoryProjectionV1 = Readonly<{
  schema_version: typeof LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1;
  project_id: string;
  context_id: string;
  commit_id: string;
  source_project_id: string;
  source_commit_id: string;
  projection: LocalKnowledgeMemoryReplayProjectionV1;
}>;

export type LocalKnowledgeMemoryProjection = LocalKnowledgeMemoryProjectionV1;
export type LocalKnowledgeMemoryReplayProjection = LocalKnowledgeMemoryReplayProjectionV1;

export type LocalRetentionDecision =
  | "retained_pinned"
  | "retained_fresh"
  | "retained_important"
  | "expired_forgotten"
  | "expired_low_importance";

export type LocalReplayState = "active" | "forgotten";

export class ContextLabLocalKnowledgeMemoryProjectionError extends ContextLabLocalApiError {
  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(status, body, retryAfterMs);
    this.name = "ContextLabLocalKnowledgeMemoryProjectionError";
  }
}

export class ContextLabLocalKnowledgeMemoryProjectionClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getKnowledgeMemoryProjection(
    projectId: string,
    contextId: string,
    commitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalKnowledgeMemoryProjectionV1> {
    const requestedProjectId = asUuid(projectId, "projectId");
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedCommitId = asUuid(commitId, "commitId");
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/projects/${encodeURIComponent(requestedProjectId)}`
        + `/contexts/${encodeURIComponent(requestedContextId)}`
        + `/commits/${encodeURIComponent(requestedCommitId)}/knowledge-memory-projection`,
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
      throw new ContextLabLocalKnowledgeMemoryProjectionError(
        response.status,
        await parseErrorBody(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const projection = parseLocalKnowledgeMemoryProjection(await response.json());
    if (
      projection.project_id !== requestedProjectId
      || projection.context_id !== requestedContextId
      || projection.commit_id !== requestedCommitId
    ) {
      throw new TypeError("Knowledge/Memory projection does not match the requested Context scope");
    }
    return projection;
  }

  async getKnowledgeMemoryReplayProjection(
    projectId: string,
    contextId: string,
    commitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalKnowledgeMemoryProjectionV1> {
    return this.getKnowledgeMemoryProjection(projectId, contextId, commitId, credentials);
  }
}

export function parseLocalKnowledgeMemoryProjection(
  value: unknown
): LocalKnowledgeMemoryProjectionV1 {
  const record = asRecord(value, "Knowledge/Memory projection");
  assertExactKeys(record, [
    "schema_version",
    "project_id",
    "context_id",
    "commit_id",
    "source_project_id",
    "source_commit_id",
    "projection"
  ]);
  if (record.schema_version !== LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1) {
    throw new TypeError(
      `schema_version must be ${LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1}`
    );
  }

  const contextId = asUuid(record.context_id, "context_id");
  const projectId = asUuid(record.project_id, "project_id");
  const commitId = asUuid(record.commit_id, "commit_id");
  const sourceProjectId = asUuid(record.source_project_id, "source_project_id");
  const sourceCommitId = asUuid(record.source_commit_id, "source_commit_id");
  if (sourceProjectId !== projectId || sourceCommitId !== commitId) {
    throw new TypeError("Knowledge/Memory projection source scope does not match its envelope");
  }
  const projection = parseReplayProjection(record.projection);
  const expectedScope = knowledgeMemoryScopeForContext(contextId);
  if (projection.knowledge_scope !== expectedScope || projection.memory_scope !== expectedScope) {
    throw new TypeError("Knowledge and Memory scopes must match the requested Context");
  }

  return freeze({
    schema_version: LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1,
    project_id: projectId,
    context_id: contextId,
    commit_id: commitId,
    source_project_id: sourceProjectId,
    source_commit_id: sourceCommitId,
    projection
  });
}

export const parseLocalKnowledgeMemoryReplayProjection = parseLocalKnowledgeMemoryProjection;

function parseReplayProjection(value: unknown): LocalKnowledgeMemoryReplayProjectionV1 {
  const record = asRecord(value, "Knowledge/Memory replay projection");
  assertExactKeys(record, [
    "id",
    "schema_version",
    "citation_projection_id",
    "citation_projection_schema_version",
    "knowledge_scope",
    "retrieval_id",
    "retrieval_version",
    "citations",
    "memory_id",
    "memory_scope",
    "memory_timeline_version",
    "memory_capability_schema_version",
    "retention_policy_version",
    "retention_decision",
    "replay_state"
  ]);
  if (record.schema_version !== KNOWLEDGE_MEMORY_REPLAY_PROJECTION_SCHEMA_V1) {
    throw new TypeError(
      `projection.schema_version must be ${KNOWLEDGE_MEMORY_REPLAY_PROJECTION_SCHEMA_V1}`
    );
  }
  if (record.citation_projection_schema_version !== "knowledge-local-citation-projection-v1") {
    throw new TypeError("citation_projection_schema_version is unsupported");
  }
  if (record.memory_capability_schema_version !== "memory-retention-capability-v1") {
    throw new TypeError("memory_capability_schema_version is unsupported");
  }

  const citations = asArray(record.citations, "citations").map(parseCitation);
  assertStrictlyAscending(citations.map((citation) => citation.chunk_id), "citations");
  const retentionDecision = asRetentionDecision(record.retention_decision);
  const replayState = asReplayState(record.replay_state);
  if (
    (replayState === "forgotten" && retentionDecision !== "expired_forgotten")
    || (replayState === "active" && retentionDecision === "expired_forgotten")
  ) {
    throw new TypeError("replay_state is inconsistent with retention_decision");
  }

  const projection = {
    id: asUuid(record.id, "projection.id"),
    schema_version: KNOWLEDGE_MEMORY_REPLAY_PROJECTION_SCHEMA_V1,
    citation_projection_id: asUuid(record.citation_projection_id, "citation_projection_id"),
    citation_projection_schema_version: "knowledge-local-citation-projection-v1" as const,
    knowledge_scope: asUuid(record.knowledge_scope, "knowledge_scope"),
    retrieval_id: asUuid(record.retrieval_id, "retrieval_id"),
    retrieval_version: asNonBlank(record.retrieval_version, "retrieval_version"),
    citations: Object.freeze(citations),
    memory_id: asUuid(record.memory_id, "memory_id"),
    memory_scope: asUuid(record.memory_scope, "memory_scope"),
    memory_timeline_version: asPositiveInteger(record.memory_timeline_version, "memory_timeline_version"),
    memory_capability_schema_version: "memory-retention-capability-v1" as const,
    retention_policy_version: asNonBlank(record.retention_policy_version, "retention_policy_version"),
    retention_decision: retentionDecision,
    replay_state: replayState
  };
  if (knowledgeMemoryReplayProjectionId(projection) !== projection.id) {
    throw new TypeError("projection.id does not match its deterministic source facts");
  }
  return freeze(projection);
}

function parseCitation(value: unknown): LocalKnowledgeCitationV1 {
  const record = asRecord(value, "citation");
  assertExactKeys(record, [
    "document_id",
    "document_revision_id",
    "chunk_id",
    "source_version",
    "chunking_version",
    "ordinal",
    "range",
    "content_fingerprint"
  ]);
  return freeze({
    document_id: asUuid(record.document_id, "citation.document_id"),
    document_revision_id: asUuid(record.document_revision_id, "citation.document_revision_id"),
    chunk_id: asUuid(record.chunk_id, "citation.chunk_id"),
    source_version: asNonBlank(record.source_version, "citation.source_version"),
    chunking_version: asNonBlank(record.chunking_version, "citation.chunking_version"),
    ordinal: asNonNegativeInteger(record.ordinal, "citation.ordinal"),
    range: parseCitationRange(record.range),
    content_fingerprint: asNonBlank(record.content_fingerprint, "citation.content_fingerprint")
  });
}

function parseCitationRange(value: unknown): LocalKnowledgeCitationRangeV1 {
  const record = asRecord(value, "citation.range");
  assertExactKeys(record, ["start_byte", "end_byte"]);
  const start = asNonNegativeInteger(record.start_byte, "citation.range.start_byte");
  const end = asNonNegativeInteger(record.end_byte, "citation.range.end_byte");
  if (end < start) {
    throw new TypeError("citation.range.end_byte must be greater than or equal to start_byte");
  }
  return freeze({ start_byte: start, end_byte: end });
}

function asRetentionDecision(value: unknown): LocalRetentionDecision {
  if (
    value === "retained_pinned"
    || value === "retained_fresh"
    || value === "retained_important"
    || value === "expired_forgotten"
    || value === "expired_low_importance"
  ) {
    return value;
  }
  throw new TypeError("retention_decision is unsupported");
}

function asReplayState(value: unknown): LocalReplayState {
  if (value === "active" || value === "forgotten") return value;
  throw new TypeError("replay_state is unsupported");
}

async function parseErrorBody(response: Response): Promise<LocalApiErrorBody> {
  let code: unknown;
  try {
    const value = await response.json();
    if (typeof value === "object" && value !== null && !Array.isArray(value)) {
      code = (value as Record<string, unknown>).error;
    }
  } catch {
    // Use the status-only fallback below for malformed or empty error bodies.
  }

  const safeCode = typeof code === "string" && SAFE_ERROR_CODES.has(code)
    ? code
    : "contextlab_local_api_error";
  return {
    error: safeCode,
    message: `ContextLab local API request failed with status ${response.status}`
  };
}

const SAFE_ERROR_CODES = new Set([
  "contextlab_local_api_error",
  "knowledge_memory_projection_not_found",
  "knowledge_memory_projection_scope_conflict",
  "knowledge_memory_projection_unavailable",
  "knowledge_memory_projection_invalid",
  "context_read_forbidden",
  "authentication_required",
  "authentication_failed",
  "authorization_unavailable",
  "rate_limit_exceeded",
  "unauthorized",
  "forbidden"
]);

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0) return seconds * 1_000;
  const timestamp = Date.parse(value);
  return Number.isNaN(timestamp) ? undefined : Math.max(0, timestamp - Date.now());
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

function asNonNegativeInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${field} must be a non-negative integer`);
  }
  return value;
}

function asPositiveInteger(value: unknown, field: string): number {
  const parsed = asNonNegativeInteger(value, field);
  if (parsed === 0) throw new TypeError(`${field} must be positive`);
  return parsed;
}

function assertExactKeys(record: ReadonlyRecord, expected: readonly string[]): void {
  const actual = Object.keys(record).sort();
  const required = [...expected].sort();
  if (actual.length !== required.length || actual.some((key, index) => key !== required[index])) {
    throw new TypeError("Knowledge/Memory projection contains an unexpected shape");
  }
}

function assertStrictlyAscending(values: readonly string[], field: string): void {
  for (let index = 1; index < values.length; index += 1) {
    if (values[index - 1]! >= values[index]!) {
      throw new TypeError(`${field} must be unique and deterministically ordered`);
    }
  }
}

function freeze<T>(value: T): T {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value as Record<string, unknown>)) freeze(child);
    Object.freeze(value);
  }
  return value;
}
