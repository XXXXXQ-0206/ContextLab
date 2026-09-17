import {
  knowledgeMemoryReplayProjectionId,
  knowledgeMemoryScopeForContext,
  parseLocalKnowledgeMemoryProjection as parseSdkKnowledgeMemoryProjection,
  type LocalApiErrorBody,
  type LocalKnowledgeMemoryProjectionV1
} from "@contextlab/local-sdk";
import type { BilingualCapabilityText, CapabilityStateKind } from "./capability-state-data";

export { LOCAL_KNOWLEDGE_MEMORY_PROJECTION_SCHEMA_V1 } from "@contextlab/local-sdk";
export { knowledgeMemoryReplayProjectionId, knowledgeMemoryScopeForContext } from "@contextlab/local-sdk";
export type {
  LocalKnowledgeCitationV1,
  LocalKnowledgeMemoryProjectionV1,
  LocalKnowledgeMemoryReplayProjectionV1
} from "@contextlab/local-sdk";

export type LocalKnowledgeMemoryProjectionTarget = Readonly<{
  project_id: string;
  context_id: string;
  commit_id: string;
  capability: BilingualCapabilityText;
}>;

export type LocalKnowledgeMemoryProjectionResource =
  | Readonly<{
      kind: "loading" | "error" | "empty" | "unavailable";
      target: LocalKnowledgeMemoryProjectionTarget;
      message?: string;
    }>
  | Readonly<{
      kind: "ready";
      target: LocalKnowledgeMemoryProjectionTarget;
      summary: LocalKnowledgeMemoryProjectionV1;
    }>;

export class LocalKnowledgeMemoryProjectionProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<LocalApiErrorBody>;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalKnowledgeMemoryProjectionProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
    this.retryAfterMs = retryAfterMs;
  }
}

export function createLocalKnowledgeMemoryProjectionResource(
  input:
    | Readonly<{
        kind: "loading" | "error" | "empty" | "unavailable";
        target: LocalKnowledgeMemoryProjectionTarget;
        message?: string;
      }>
    | Readonly<{
        kind: "ready";
        target: LocalKnowledgeMemoryProjectionTarget;
        summary: LocalKnowledgeMemoryProjectionV1;
      }>
): LocalKnowledgeMemoryProjectionResource {
  const target = freezeTarget(input.target);
  if (input.kind !== "ready") {
    return Object.freeze({ kind: input.kind, target, ...(input.message ? { message: input.message } : {}) });
  }
  const summary = parseLocalKnowledgeMemoryProjectionV1(input.summary);
  assertScope(summary, target);
  return Object.freeze({ kind: "ready", target, summary });
}

export async function loadLocalKnowledgeMemoryProjection(
  target: LocalKnowledgeMemoryProjectionTarget,
  bearerToken: string
): Promise<LocalKnowledgeMemoryProjectionV1> {
  const exactTarget = freezeTarget(target);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(exactTarget.project_id)}`
      + `/contexts/${encodeURIComponent(exactTarget.context_id)}`
      + `/commits/${encodeURIComponent(exactTarget.commit_id)}/knowledge-memory-projection`,
    {
      credentials: "omit",
      cache: "no-store",
      headers: { accept: "application/json", authorization: `Bearer ${token}` }
    }
  );
  if (!response.ok) {
    throw new LocalKnowledgeMemoryProjectionProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }
  const summary = parseLocalKnowledgeMemoryProjectionV1(await response.json());
  assertScope(summary, exactTarget);
  return summary;
}

export function adaptLocalKnowledgeMemoryProjectionV1(
  resource: LocalKnowledgeMemoryProjectionResource
): Readonly<{
  id: string;
  project_id: string;
  context_id: string;
  commit_id: string;
  capability: BilingualCapabilityText;
  state: CapabilityStateKind;
  projection: LocalKnowledgeMemoryProjectionV1["projection"] | null;
}> {
  const target = freezeTarget(resource.target);
  return Object.freeze({
    id: "local-knowledge-memory-projection",
    project_id: target.project_id,
    context_id: target.context_id,
    commit_id: target.commit_id,
    capability: target.capability,
    state: resource.kind === "ready" ? "available" : resource.kind,
    projection: resource.kind === "ready" ? resource.summary.projection : null
  });
}

export function parseLocalKnowledgeMemoryProjectionV1(value: unknown): LocalKnowledgeMemoryProjectionV1 {
  return parseSdkKnowledgeMemoryProjection(value);
}

function assertScope(
  summary: LocalKnowledgeMemoryProjectionV1,
  target: LocalKnowledgeMemoryProjectionTarget
): void {
  if (
    summary.project_id !== target.project_id
    || summary.context_id !== target.context_id
    || summary.commit_id !== target.commit_id
    || summary.projection.knowledge_scope !== summary.projection.memory_scope
  ) {
    throw new TypeError("Knowledge/Memory projection is outside the requested exact scope");
  }
}

function freezeTarget(target: LocalKnowledgeMemoryProjectionTarget): LocalKnowledgeMemoryProjectionTarget {
  return Object.freeze({
    project_id: requireNonBlank(target.project_id, "target.project_id"),
    context_id: requireNonBlank(target.context_id, "target.context_id"),
    commit_id: requireNonBlank(target.commit_id, "target.commit_id"),
    capability: Object.freeze({
      en: requireNonBlank(target.capability.en, "target.capability.en"),
      zh: requireNonBlank(target.capability.zh, "target.capability.zh")
    })
  });
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const value = await response.json();
    if (
      typeof value === "object"
      && value !== null
      && !Array.isArray(value)
      && typeof value.error === "string"
      && typeof value.message === "string"
    ) {
      return { error: value.error, message: value.message };
    }
  } catch {
  }
  return {
    error: "contextlab_knowledge_memory_projection_proxy_error",
    message: "The local Knowledge/Memory projection is unavailable / 本地 Knowledge/Memory 投影不可用。"
  };
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}

function requireNonBlank(value: string, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) throw new TypeError(`${field} must be non-blank`);
  return value;
}
