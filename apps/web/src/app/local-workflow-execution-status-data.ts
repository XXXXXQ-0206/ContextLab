import type { BilingualCapabilityText, CapabilityStateKind, FrozenCapabilityStateDto } from "./capability-state-data";
import {
  assertLocalWorkflowExecutionStatusScope,
  parseLocalWorkflowExecutionStatusV1,
  type LocalWorkflowExecutionStatusScope,
  type LocalWorkflowExecutionStatusV1
} from "@contextlab/local-sdk";

export type LocalWorkflowExecutionStatusTarget = Readonly<LocalWorkflowExecutionStatusScope & {
  capability: BilingualCapabilityText;
}>;

export type LocalWorkflowExecutionStatusResource =
  | Readonly<{ kind: "loading" | "error" | "empty" | "unavailable"; target: LocalWorkflowExecutionStatusTarget; message?: string }>
  | Readonly<{ kind: "ready"; target: LocalWorkflowExecutionStatusTarget; status: LocalWorkflowExecutionStatusV1 }>;

export type LocalWorkflowExecutionStatusDto = Readonly<{
  id: "local-workflow-execution-status";
  capability: BilingualCapabilityText;
  state: CapabilityStateKind;
  target: LocalWorkflowExecutionStatusTarget;
  status?: LocalWorkflowExecutionStatusV1;
}>;

export class LocalWorkflowExecutionStatusProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<{ error: string; message: string }>;

  constructor(status: number, body: Readonly<{ error: string; message: string }>) {
    super(body.message);
    this.name = "LocalWorkflowExecutionStatusProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
  }
}

export async function loadLocalWorkflowExecutionStatus(
  target: LocalWorkflowExecutionStatusTarget,
  bearerToken: string
): Promise<LocalWorkflowExecutionStatusV1> {
  const exactTarget = freezeTarget(target);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const response = await fetch(
    `/api/local/contexts/${encodeURIComponent(exactTarget.context_id)}`
      + `/workflow/runs/${encodeURIComponent(exactTarget.run_id)}/status`,
    {
      method: "GET",
      credentials: "omit",
      cache: "no-store",
      headers: {
        accept: "application/json",
        authorization: `Bearer ${token}`
      }
    }
  );

  if (!response.ok) {
    throw new LocalWorkflowExecutionStatusProxyError(
      response.status,
      await parseProxyErrorBody(response)
    );
  }

  const status = parseLocalWorkflowExecutionStatusV1(await response.json());
  assertLocalWorkflowExecutionStatusScope(status, exactTarget);
  return status;
}

export function adaptLocalWorkflowExecutionStatusV1(
  resource: LocalWorkflowExecutionStatusResource
): LocalWorkflowExecutionStatusDto {
  const target = freezeTarget(resource.target);
  switch (resource.kind) {
    case "ready": {
      const status = parseLocalWorkflowExecutionStatusV1(resource.status);
      assertLocalWorkflowExecutionStatusScope(status, target);
      return freezeDto(target, "available", status);
    }
    case "loading":
      return freezeDto(target, "loading");
    case "error":
      return freezeDto(target, "error");
    case "empty":
      return freezeDto(target, "empty");
    case "unavailable":
      return freezeDto(target, "unavailable");
  }
}

function freezeTarget(value: LocalWorkflowExecutionStatusTarget): LocalWorkflowExecutionStatusTarget {
  return Object.freeze({
    context_id: requireUuid(value.context_id, "target.context_id"),
    context_commit_id: requireUuid(value.context_commit_id, "target.context_commit_id"),
    binding_id: requireUuid(value.binding_id, "target.binding_id"),
    workflow_id: requireUuid(value.workflow_id, "target.workflow_id"),
    workflow_revision: requirePositiveInteger(value.workflow_revision, "target.workflow_revision"),
    run_id: requireUuid(value.run_id, "target.run_id"),
    capability: Object.freeze({
      en: requireNonBlank(value.capability.en, "target.capability.en"),
      zh: requireNonBlank(value.capability.zh, "target.capability.zh")
    })
  });
}

function freezeDto(
  target: LocalWorkflowExecutionStatusTarget,
  state: CapabilityStateKind,
  status?: LocalWorkflowExecutionStatusV1
): LocalWorkflowExecutionStatusDto {
  return Object.freeze({
    id: "local-workflow-execution-status" as const,
    capability: target.capability,
    state,
    target,
    ...(status ? { status } : {})
  });
}

async function parseProxyErrorBody(
  response: Response
): Promise<Readonly<{ error: string; message: string }>> {
  try {
    const value = await response.json();
    if (
      value !== null
      && typeof value === "object"
      && !Array.isArray(value)
      && typeof value.error === "string"
      && typeof value.message === "string"
      && value.error.trim()
      && value.message.trim()
    ) {
      return Object.freeze({
        error: value.error,
        message: redactedProxyErrorMessage(response.status)
      });
    }
  } catch {
    // Only a redacted status-based error crosses the same-origin boundary.
  }
  return Object.freeze({
    error: "contextlab_web_api_error",
    message: "The local workflow execution status is unavailable / 本地工作流执行状态不可用。"
  });
}

function redactedProxyErrorMessage(status: number): string {
  return `ContextLab local API request failed with status ${status}`;
}

function requireNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be non-blank`);
  }
  return value;
}

function requireUuid(value: unknown, field: string): string {
  const parsed = requireNonBlank(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function requirePositiveInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value <= 0) {
    throw new TypeError(`${field} must be a positive integer`);
  }
  return value;
}
