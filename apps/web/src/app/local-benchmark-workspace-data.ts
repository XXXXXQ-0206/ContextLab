import {
  parseLocalBenchmarkWorkspace,
  type LocalApiErrorBody,
  type LocalBenchmarkWorkspace
} from "@contextlab/local-sdk";
import {
  loadLocalBenchmarkDecisionDiscovery,
  type LocalBenchmarkDecisionDiscovery
} from "./context-benchmark-decision-discovery-data";

type DeepReadonly<T> = T extends (...args: never[]) => unknown
  ? T
  : T extends ReadonlyArray<infer TItem>
    ? ReadonlyArray<DeepReadonly<TItem>>
    : T extends object
      ? { readonly [TKey in keyof T]: DeepReadonly<T[TKey]> }
      : T;

export type LocalBenchmarkWorkspaceResponse = LocalBenchmarkWorkspace;

export type FrozenLocalBenchmarkWorkspace = DeepReadonly<LocalBenchmarkWorkspaceResponse>;

export type LocalBenchmarkWorkspaceBaselineTarget = Readonly<{
  commitId: string;
  decisionId?: string;
  cohortId?: string;
}>;

export type LocalBenchmarkWorkspaceTarget = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
  decisionId?: string;
  cohortId?: string;
  baseline?: LocalBenchmarkWorkspaceBaselineTarget;
}>;

export type LocalBenchmarkWorkspaceDecisionDiscoveryTarget = Readonly<{
  projectId: string;
  contextId: string;
  revisedCommitId: string;
  baselineCommitId?: string;
}>;

export type LocalBenchmarkWorkspaceDecisionDiscoveries = Readonly<{
  revised: LocalBenchmarkDecisionDiscovery;
  baseline: LocalBenchmarkDecisionDiscovery | null;
}>;

export type LocalBenchmarkWorkspaceResourceState =
  | "loading"
  | "error"
  | "empty"
  | "available"
  | "unavailable";

type LocalBenchmarkWorkspaceResourceWithoutProjection = Readonly<{
  state: Exclude<LocalBenchmarkWorkspaceResourceState, "available">;
  target: LocalBenchmarkWorkspaceTarget;
  message?: string;
}>;

type LocalBenchmarkWorkspaceAvailableResource = Readonly<{
  state: "available";
  target: LocalBenchmarkWorkspaceTarget;
  workspace: FrozenLocalBenchmarkWorkspace;
}>;

export type LocalBenchmarkWorkspaceResource =
  | LocalBenchmarkWorkspaceResourceWithoutProjection
  | LocalBenchmarkWorkspaceAvailableResource;

type LocalBenchmarkWorkspaceResourceInput =
  | Readonly<{
      state: Exclude<LocalBenchmarkWorkspaceResourceState, "available">;
      target: LocalBenchmarkWorkspaceTarget;
      message?: string;
    }>
  | Readonly<{
      state: "available";
      target: LocalBenchmarkWorkspaceTarget;
      workspace: LocalBenchmarkWorkspace | FrozenLocalBenchmarkWorkspace;
    }>;

export class LocalBenchmarkWorkspaceProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<LocalApiErrorBody>;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalBenchmarkWorkspaceProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
    this.retryAfterMs = retryAfterMs;
  }
}

export async function loadLocalBenchmarkWorkspaceDecisionDiscoveries(
  target: LocalBenchmarkWorkspaceDecisionDiscoveryTarget,
  bearerToken: string
): Promise<LocalBenchmarkWorkspaceDecisionDiscoveries> {
  const revisedCommitId = requireNonBlank(target.revisedCommitId, "revisedCommitId");
  const baselineCommitId = target.baselineCommitId === undefined
    ? undefined
    : requireNonBlank(target.baselineCommitId, "baselineCommitId");
  const [revised, baseline] = await Promise.all([
    loadLocalBenchmarkDecisionDiscovery(
      requireNonBlank(target.projectId, "projectId"),
      requireNonBlank(target.contextId, "contextId"),
      revisedCommitId,
      bearerToken
    ),
    baselineCommitId === undefined
      ? Promise.resolve(null)
      : loadLocalBenchmarkDecisionDiscovery(
          requireNonBlank(target.projectId, "projectId"),
          requireNonBlank(target.contextId, "contextId"),
          baselineCommitId,
          bearerToken
        )
  ]);

  return Object.freeze({ revised, baseline });
}

export async function loadLocalBenchmarkWorkspace(
  target: LocalBenchmarkWorkspaceTarget,
  bearerToken: string
): Promise<FrozenLocalBenchmarkWorkspace> {
  const projectId = requireNonBlank(target.projectId, "projectId");
  const contextId = requireNonBlank(target.contextId, "contextId");
  const commitId = requireNonBlank(target.commitId, "commitId");
  const decisionId = target.decisionId === undefined
    ? undefined
    : requireNonBlank(target.decisionId, "decisionId");
  const cohortId = decisionId === undefined
    ? requireNonBlank(target.cohortId, "cohortId")
    : undefined;
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const baseline: LocalBenchmarkWorkspaceTarget["baseline"] = target.baseline === undefined
    ? undefined
    : Object.freeze({
        commitId: requireNonBlank(target.baseline.commitId, "baseline.commitId"),
        ...(target.baseline.decisionId === undefined
          ? { cohortId: requireNonBlank(target.baseline.cohortId, "baseline.cohortId") }
          : { decisionId: requireNonBlank(target.baseline.decisionId, "baseline.decisionId") })
      });
  const query = new URLSearchParams();
  if (baseline) {
    query.set("baseline_commit_id", baseline.commitId);
    query.set(
      baseline.decisionId === undefined ? "baseline_cohort_id" : "baseline_decision_id",
      baseline.decisionId ?? baseline.cohortId!
    );
  }
  const queryString = query.size > 0 ? `?${query.toString()}` : "";
  const path = decisionId === undefined
    ? `/api/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/benchmark-workspace/${encodeURIComponent(cohortId!)}${queryString}`
    : `/api/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/benchmark-decisions/${encodeURIComponent(decisionId)}/workspace${queryString}`;
  const response = await fetch(
    path,
    {
      headers: {
        accept: "application/json",
        authorization: `Bearer ${token}`
      },
      credentials: "omit",
      cache: "no-store"
    }
  );

  if (!response.ok) {
    throw new LocalBenchmarkWorkspaceProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }

  const workspace = parseLocalBenchmarkWorkspaceResponse(await response.json());
  assertRequestedScope(workspace, {
    projectId,
    contextId,
    commitId,
    ...(decisionId === undefined ? { cohortId: cohortId! } : { decisionId }),
    ...(baseline ? { baseline } : {})
  });
  return deepFreeze(workspace);
}

export function createLocalBenchmarkWorkspaceResource(
  input: LocalBenchmarkWorkspaceResourceInput
): LocalBenchmarkWorkspaceResource {
  const target = freezeTarget(input.target);
  if (input.state === "available") {
    assertRequestedScope(input.workspace, target);
    return Object.freeze({
      state: "available",
      target,
      workspace: deepFreeze(input.workspace)
    });
  }

  return Object.freeze({
    state: input.state,
    target,
    ...(input.message === undefined ? {} : { message: input.message })
  });
}

function freezeTarget(target: LocalBenchmarkWorkspaceTarget): LocalBenchmarkWorkspaceTarget {
  return Object.freeze({
    projectId: target.projectId,
    contextId: target.contextId,
    commitId: target.commitId,
    ...(target.decisionId === undefined ? { cohortId: target.cohortId } : { decisionId: target.decisionId }),
    ...(target.baseline
      ? {
          baseline: Object.freeze({
            commitId: target.baseline.commitId,
            ...(target.baseline.decisionId === undefined
              ? { cohortId: target.baseline.cohortId }
              : { decisionId: target.baseline.decisionId })
          })
        }
      : {})
  });
}

function assertRequestedScope(
  workspace: LocalBenchmarkWorkspaceResponse | FrozenLocalBenchmarkWorkspace,
  target: LocalBenchmarkWorkspaceTarget
): void {
  const baselineMatches = target.baseline === undefined
    ? workspace.baseline === null
    : workspace.baseline?.commit_id === target.baseline.commitId
      && (target.baseline.decisionId !== undefined || workspace.baseline.cohort_id === target.baseline.cohortId);
  const decisionPairRequested = target.decisionId !== undefined
    && target.baseline?.decisionId !== undefined;
  const witness = workspace.decision_pair_witness;
  if (
    workspace.project_id !== target.projectId
    || workspace.context_id !== target.contextId
    || workspace.revised.commit_id !== target.commitId
    || (target.decisionId === undefined && workspace.revised.cohort_id !== target.cohortId)
    || !baselineMatches
    || (decisionPairRequested
      ? witness === undefined
        || witness.project_id !== target.projectId
        || witness.context_id !== target.contextId
        || witness.revised.commit_id !== target.commitId
        || witness.revised.decision_id !== target.decisionId
        || witness.baseline.commit_id !== target.baseline!.commitId
        || witness.baseline.decision_id !== target.baseline!.decisionId
      : witness !== undefined)
  ) {
    throw new TypeError("local benchmark workspace response does not match the requested scope");
  }
}

function parseLocalBenchmarkWorkspaceResponse(value: unknown): LocalBenchmarkWorkspaceResponse {
  return parseLocalBenchmarkWorkspace(value);
}

function requireNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new RangeError(`${field} must be non-empty`);
  }
  return value;
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalApiErrorBody>;
    if (typeof body.error === "string" && typeof body.message === "string") {
      return { error: body.error, message: body.message };
    }
  } catch {
  }

  return {
    error: "contextlab_benchmark_workspace_proxy_error",
    message: `ContextLab benchmark workspace request failed with status ${response.status}`
  };
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) {
    return undefined;
  }
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0) {
    return seconds * 1_000;
  }
  const retryAt = Date.parse(value);
  return Number.isNaN(retryAt) ? undefined : Math.max(0, retryAt - Date.now());
}

function deepFreeze<T>(value: T): DeepReadonly<T> {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) {
      deepFreeze(child);
    }
    Object.freeze(value);
  }
  return value as DeepReadonly<T>;
}
