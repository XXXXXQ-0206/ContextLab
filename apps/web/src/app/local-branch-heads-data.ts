import type { BilingualCapabilityText, CapabilityStateKind } from "./capability-state-data";
import {
  LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1,
  parseLocalContextBranchHeadsResourceV1
} from "@contextlab/local-sdk";
import type {
  LocalContextBranchHeadV1,
  LocalContextBranchHeadsResourceV1
} from "@contextlab/local-sdk";

export { LOCAL_CONTEXT_BRANCH_HEADS_SCHEMA_V1 };

export type LocalBranchHeadV1 = LocalContextBranchHeadV1;

export type LocalBranchHeadsV1 = LocalContextBranchHeadsResourceV1;

export type LocalBranchHeadTarget = Readonly<{
  branch_name: string;
  head_commit_id: string;
}>;

export type LocalBranchHeadsTarget = Readonly<{
  contextId: string;
  capability: BilingualCapabilityText;
}>;

export type LocalBranchHeadsResource =
  | Readonly<{ kind: "loading" | "error" | "empty" | "unavailable"; target: LocalBranchHeadsTarget; message?: string }>
  | Readonly<{ kind: "ready"; target: LocalBranchHeadsTarget; summary: LocalBranchHeadsV1 }>;

export type FrozenLocalBranchHeadsDto = Readonly<{
  id: "local-branch-heads";
  contextId: string;
  capability: BilingualCapabilityText;
  state: CapabilityStateKind;
  branches: ReadonlyArray<LocalBranchHeadV1>;
}>;

export type LocalBranchHeadsAvailability = Readonly<{
  capabilityAvailable: boolean;
  hasRequestMemoryCredential: boolean;
}>;

const emptyBranches = Object.freeze([]) as ReadonlyArray<LocalBranchHeadV1>;
const defaultCapability = Object.freeze({
  en: "Local branch heads",
  zh: "本地分支 head"
});
const localBranchHeadsErrorMessage = "Unable to load local branch heads / 无法加载本地分支 head。";

export async function loadLocalBranchHeads(
  targetOrContextId: LocalBranchHeadsTarget | string,
  bearerToken: string
): Promise<LocalBranchHeadsV1> {
  const requestedTarget = freezeTarget(targetOrContextId);
  const contextId = asCanonicalUuid(requestedTarget.contextId, "contextId");
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const response = await fetch(`/api/local/contexts/${encodeURIComponent(contextId)}/branches`, {
    headers: {
      accept: "application/json",
      authorization: `Bearer ${token}`
    },
    credentials: "omit",
    cache: "no-store"
  });

  if (!response.ok) {
    throw new LocalBranchHeadsProxyError(response.status, await parseErrorBody(response));
  }

  const result = parseLocalBranchHeadsV1(await response.json());
  if (result.context_id !== contextId) {
    throw new TypeError("local branch heads response does not match the requested Context");
  }
  return result;
}

export class LocalBranchHeadsProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<{ error: string; message: string }>;

  constructor(status: number, body: Readonly<{ error: string; message: string }>) {
    super(body.message);
    this.name = "LocalBranchHeadsProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
  }
}

export function createInitialLocalBranchHeadsResource(
  targetOrContextId: LocalBranchHeadsTarget | string,
  availability: LocalBranchHeadsAvailability = {
    capabilityAvailable: false,
    hasRequestMemoryCredential: false
  }
): LocalBranchHeadsResource {
  const target = freezeTarget(targetOrContextId);
  const enabled = availability.capabilityAvailable && availability.hasRequestMemoryCredential;
  return Object.freeze({
    kind: enabled ? "empty" : "unavailable",
    target,
    message: enabled
      ? "No branch-head read has been requested. / 尚未请求分支 head 读取。"
      : "Branch-head inspection requires a request-memory credential and capability. / 分支 head 检查需要请求内存凭据和能力。"
  });
}

export function createLocalBranchHeadsResource(
  input: LocalBranchHeadsResource
): LocalBranchHeadsResource {
  const target = freezeTarget(input.target);
  if (input.kind !== "ready") {
    return Object.freeze({
      kind: input.kind,
      target,
      ...(input.message === undefined ? {} : { message: input.message })
    });
  }

  const summary = parseLocalBranchHeadsV1(input.summary);
  if (summary.context_id !== target.contextId) {
    throw new TypeError("branch heads response does not match the requested Context");
  }
  return Object.freeze({ kind: "ready", target, summary });
}

export function adaptLocalBranchHeadsV1(
  resource: LocalBranchHeadsResource
): FrozenLocalBranchHeadsDto {
  const target = freezeTarget(resource.target);
  switch (resource.kind) {
    case "loading":
      return freezeDto(target, "loading", emptyBranches);
    case "error":
      return freezeDto(target, "error", emptyBranches);
    case "empty":
      return freezeDto(target, "empty", emptyBranches);
    case "unavailable":
      return freezeDto(target, "unavailable", emptyBranches);
    case "ready": {
      const summary = parseLocalBranchHeadsV1(resource.summary);
      if (summary.context_id !== target.contextId) {
        throw new TypeError("branch heads response does not match the requested Context");
      }
      const branches = freezeBranches(summary.branches);
      return freezeDto(target, branches.length === 0 ? "empty" : "available", branches);
    }
  }
}

export function parseLocalBranchHeadsV1(value: unknown): LocalBranchHeadsV1 {
  return parseLocalContextBranchHeadsResourceV1(value);
}

export function selectedLocalBranchHeadCommit(
  resource: LocalBranchHeadsResource,
  branchName: string
): string | null {
  return selectedLocalBranchHeadTarget(resource, branchName)?.head_commit_id ?? null;
}

export function selectedLocalBranchHeadTarget(
  resource: LocalBranchHeadsResource,
  branchName: string
): LocalBranchHeadTarget | null {
  if (resource.kind !== "ready") return null;

  const branch = resource.summary.branches.find((candidate) => candidate.branch_name === branchName);
  if (!branch?.head_commit_id) return null;

  return Object.freeze({
    branch_name: branch.branch_name,
    head_commit_id: branch.head_commit_id
  });
}

function freezeTarget(value: LocalBranchHeadsTarget | string): LocalBranchHeadsTarget {
  if (typeof value === "string") {
    return Object.freeze({ contextId: asNonBlankString(value, "contextId"), capability: defaultCapability });
  }
  return Object.freeze({
    contextId: asNonBlankString(value.contextId, "target.contextId"),
    capability: parseBilingualText(value.capability, "target.capability")
  });
}

function freezeDto(
  target: LocalBranchHeadsTarget,
  state: CapabilityStateKind,
  branches: ReadonlyArray<LocalBranchHeadV1>
): FrozenLocalBranchHeadsDto {
  return Object.freeze({
    id: "local-branch-heads",
    contextId: target.contextId,
    capability: target.capability,
    state,
    branches: freezeBranches(branches)
  });
}

function freezeBranches(branches: ReadonlyArray<LocalBranchHeadV1>): ReadonlyArray<LocalBranchHeadV1> {
  return Object.freeze(branches.map((branch) => Object.freeze({ ...branch })));
}

function parseBilingualText(value: unknown, field: string): BilingualCapabilityText {
  const record = asRecord(value, field);
  assertExactKeys(record, ["en", "zh"]);
  return Object.freeze({
    en: asNonBlankString(record.en, `${field}.en`),
    zh: asNonBlankString(record.zh, `${field}.zh`)
  });
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as Record<string, unknown>;
}

function assertExactKeys(record: Record<string, unknown>, expected: string[]): void {
  const actual = Object.keys(record).sort(compareStrings);
  const sortedExpected = [...expected].sort(compareStrings);
  if (actual.length !== sortedExpected.length || actual.some((key, index) => key !== sortedExpected[index])) {
    throw new TypeError("local branch heads contain an unexpected shape");
  }
}

function compareStrings(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function asNonBlankString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be a non-blank string`);
  }
  return value;
}

function asCanonicalUuid(value: unknown, field: string): string {
  const parsed = asNonBlankString(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function requireNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new RangeError(`${field} must be non-empty`);
  }
  return value;
}

async function parseErrorBody(response: Response): Promise<Readonly<{ error: string; message: string }>> {
  try {
    const body = (await response.json()) as Partial<{ error: string; message: string }>;
    if (typeof body.error === "string" && typeof body.message === "string") {
      return { error: body.error, message: localBranchHeadsErrorMessage };
    }
  } catch {
  }
  return {
    error: "contextlab_local_branch_heads_error",
    message: localBranchHeadsErrorMessage
  };
}
