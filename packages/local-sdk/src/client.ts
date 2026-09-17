import type {
  LocalApiErrorBody,
  LocalBenchmarkDecision,
  LocalBenchmarkDecisionDiff,
  LocalBenchmarkDecisionList,
  LocalBenchmarkDecisionRunDetails,
  LocalBenchmarkDecisionScope,
  LocalComponentLifecycleCommitRequest,
  LocalComponentLifecycleCommitResponse,
  LocalContextLifecycleState,
  LocalLifecycleReadCredentials,
  LocalWorkflowContextBindings,
  LocalWorkflowCapabilityStatus,
  LocalLifecycleWriteCredentials
} from "./types";
import {
  parseLocalBenchmarkDecision,
  parseLocalBenchmarkDecisionList,
  parseLocalBenchmarkDecisionDiff,
  parseLocalBenchmarkDecisionRunDetails,
  parseLocalComponentLifecycleCommitRequest,
  parseLocalComponentLifecycleCommitResponse,
  parseLocalContextLifecycleState,
  parseLocalWorkflowContextBindings,
  parseLocalWorkflowCapabilityStatus
} from "./types";
import type {
  LocalBenchmarkWorkspace,
  LocalBenchmarkWorkspaceBaselineRequest,
  LocalBenchmarkWorkspaceDecisionBaselineRequest
} from "./benchmark-workspace";
import { parseLocalBenchmarkWorkspace } from "./benchmark-workspace";
import {
  ContextLabLocalCommitGraphDiffClient,
  type LocalCommitGraphDiffResponseV1
} from "./commit-graph-diff";
import {
  ContextLabLocalBranchHeadClient,
  type LocalContextBranchHeadsResourceV1
} from "./context-branch-heads";
import {
  ContextLabLocalPersistedContextDiffReviewClient,
  type LocalPersistedContextDiffReviewV1
} from "./persisted-context-diff-review";

export type FetchLike = (input: string | URL, init?: RequestInit) => Promise<Response>;

export type ContextLabLocalClientOptions = {
  baseUrl?: string;
  fetch?: FetchLike;
};

const DEFAULT_BASE_URL = "http://127.0.0.1:3100";

export class ContextLabLocalApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly body: LocalApiErrorBody;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "ContextLabLocalApiError";
    this.status = status;
    this.code = body.error;
    this.body = body;
    this.retryAfterMs = retryAfterMs;
  }
}

export class ContextLabLocalClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = normalizeBaseUrl(options.baseUrl ?? DEFAULT_BASE_URL);
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getContextLifecycleState(
    contextId: string,
    commitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalContextLifecycleState> {
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedCommitId = requireNonEmpty(commitId, "commitId");
    const state = parseLocalContextLifecycleState(await this.request<unknown>(
      `/api/v1/local/contexts/${encodePathSegment(contextId)}/commits/${encodePathSegment(commitId)}/lifecycle-state`,
      credentials
    ));
    if (
      state.context_id !== requestedContextId
      || state.commit_id !== requestedCommitId
      || state.graph_snapshot.context_id !== requestedContextId
      || state.graph_snapshot.commit_id !== requestedCommitId
    ) {
      throw new TypeError("Context lifecycle state response does not match the requested scope");
    }
    return state;
  }

  async getCommitGraphDiff(
    contextId: string,
    originalCommitId: string,
    revisedCommitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalCommitGraphDiffResponseV1> {
    return new ContextLabLocalCommitGraphDiffClient({
      baseUrl: this.baseUrl,
      fetch: this.fetchImpl
    }).getCommitGraphDiff(contextId, originalCommitId, revisedCommitId, credentials);
  }

  async getPersistedContextDiffReview(
    projectId: string,
    contextId: string,
    sourceCommitId: string,
    targetCommitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalPersistedContextDiffReviewV1> {
    return new ContextLabLocalPersistedContextDiffReviewClient({
      baseUrl: this.baseUrl,
      fetch: this.fetchImpl
    }).getPersistedContextDiffReview(
      projectId,
      contextId,
      sourceCommitId,
      targetCommitId,
      credentials
    );
  }

  async getContextBranchHeads(
    contextId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalContextBranchHeadsResourceV1> {
    return new ContextLabLocalBranchHeadClient({
      baseUrl: this.baseUrl,
      fetch: this.fetchImpl
    }).getContextBranchHeads(contextId, credentials);
  }

  async getBenchmarkDecision(
    projectId: string,
    contextId: string,
    commitId: string,
    decisionId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalBenchmarkDecision> {
    const requestedProjectId = requireNonEmpty(projectId, "projectId");
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedCommitId = requireNonEmpty(commitId, "commitId");
    const requestedDecisionId = requireNonEmpty(decisionId, "decisionId");
    const decision = await this.request<unknown>(
      `/api/v1/local/projects/${encodePathSegment(requestedProjectId)}/contexts/${encodePathSegment(requestedContextId)}/commits/${encodePathSegment(requestedCommitId)}/benchmark-decisions/${encodePathSegment(requestedDecisionId)}`,
      credentials
    );
    const parsed = parseLocalBenchmarkDecision(decision);
    if (
      parsed.project_id !== requestedProjectId
      || parsed.context_id !== requestedContextId
      || parsed.commit_id !== requestedCommitId
      || parsed.decision_id !== requestedDecisionId
    ) {
      throw new TypeError("benchmark decision response does not match the requested scope");
    }
    return parsed;
  }

  async listBenchmarkDecisions(
    projectId: string,
    contextId: string,
    commitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalBenchmarkDecisionList> {
    const requestedProjectId = requireNonEmpty(projectId, "projectId");
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedCommitId = requireNonEmpty(commitId, "commitId");
    const response = await this.request<unknown>(
      `/api/v1/local/projects/${encodePathSegment(requestedProjectId)}/contexts/${encodePathSegment(requestedContextId)}/commits/${encodePathSegment(requestedCommitId)}/benchmark-decisions`,
      credentials
    );
    const list = parseLocalBenchmarkDecisionList(response);
    if (
      list.project_id !== requestedProjectId
      || list.context_id !== requestedContextId
      || list.commit_id !== requestedCommitId
    ) {
      throw new TypeError("benchmark decision list response does not match the requested scope");
    }

    return list;
  }

  async getBenchmarkDecisionDiff(
    projectId: string,
    contextId: string,
    baseline: LocalBenchmarkDecisionScope,
    revised: LocalBenchmarkDecisionScope,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalBenchmarkDecisionDiff> {
    const requestedProjectId = requireNonEmpty(projectId, "projectId");
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedBaseline = {
      commit_id: requireNonEmpty(baseline.commit_id, "baseline.commit_id"),
      decision_id: requireNonEmpty(baseline.decision_id, "baseline.decision_id")
    };
    const requestedRevised = {
      commit_id: requireNonEmpty(revised.commit_id, "revised.commit_id"),
      decision_id: requireNonEmpty(revised.decision_id, "revised.decision_id")
    };
    const query = new URLSearchParams({
      baseline_commit_id: requestedBaseline.commit_id,
      baseline_decision_id: requestedBaseline.decision_id,
      revised_commit_id: requestedRevised.commit_id,
      revised_decision_id: requestedRevised.decision_id
    });
    const diff = await this.request<unknown>(
      `/api/v1/local/projects/${encodePathSegment(requestedProjectId)}/contexts/${encodePathSegment(requestedContextId)}/benchmark-decision-diffs?${query.toString()}`,
      credentials
    );
    const parsed = parseLocalBenchmarkDecisionDiff(diff);
    if (
      parsed.project_id !== requestedProjectId
      || parsed.context_id !== requestedContextId
      || parsed.baseline.commit_id !== requestedBaseline.commit_id
      || parsed.baseline.decision_id !== requestedBaseline.decision_id
      || parsed.revised.commit_id !== requestedRevised.commit_id
      || parsed.revised.decision_id !== requestedRevised.decision_id
    ) {
      throw new TypeError("benchmark decision diff response does not match the requested scopes");
    }
    return parsed;
  }

  async getBenchmarkDecisionRunDetails(
    projectId: string,
    contextId: string,
    commitId: string,
    decisionId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalBenchmarkDecisionRunDetails> {
    const requestedProjectId = requireNonEmpty(projectId, "projectId");
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedCommitId = requireNonEmpty(commitId, "commitId");
    const requestedDecisionId = requireNonEmpty(decisionId, "decisionId");
    const details = await this.request<unknown>(
      `/api/v1/local/projects/${encodePathSegment(requestedProjectId)}/contexts/${encodePathSegment(requestedContextId)}/commits/${encodePathSegment(requestedCommitId)}/benchmark-decisions/${encodePathSegment(requestedDecisionId)}/run-details`,
      credentials
    );
    const parsed = parseLocalBenchmarkDecisionRunDetails(details);
    if (
      parsed.project_id !== requestedProjectId
      || parsed.context_id !== requestedContextId
      || parsed.commit_id !== requestedCommitId
      || parsed.decision_id !== requestedDecisionId
    ) {
      throw new TypeError("benchmark decision run details response does not match the requested scope");
    }
    return parsed;
  }

  async getWorkflowCapabilityStatus(
    contextId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalWorkflowCapabilityStatus> {
    const status = await this.request<unknown>(
      `/api/v1/local/contexts/${encodePathSegment(contextId)}/workflow/capability-status`,
      credentials
    );
    return parseLocalWorkflowCapabilityStatus(status);
  }

  async getBenchmarkWorkspace(
    projectId: string,
    contextId: string,
    commitId: string,
    cohortId: string,
    credentials: LocalLifecycleReadCredentials,
    baseline?: LocalBenchmarkWorkspaceBaselineRequest
  ): Promise<LocalBenchmarkWorkspace> {
    const requestedProjectId = requireNonEmpty(projectId, "projectId");
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedCommitId = requireNonEmpty(commitId, "commitId");
    const requestedCohortId = requireNonEmpty(cohortId, "cohortId");
    let query = "";
    let requestedBaseline: LocalBenchmarkWorkspaceBaselineRequest | undefined;
    if (baseline !== undefined) {
      requestedBaseline = {
        commitId: requireNonEmpty(baseline.commitId, "baseline.commitId"),
        cohortId: requireNonEmpty(baseline.cohortId, "baseline.cohortId")
      };
      const params = new URLSearchParams();
      params.set("baseline_commit_id", requestedBaseline.commitId);
      params.set("baseline_cohort_id", requestedBaseline.cohortId);
      query = `?${params.toString()}`;
    }
    const response = await this.request<unknown>(
      `/api/v1/local/projects/${encodePathSegment(requestedProjectId)}/contexts/${encodePathSegment(requestedContextId)}/commits/${encodePathSegment(requestedCommitId)}/benchmark-workspace/${encodePathSegment(requestedCohortId)}${query}`,
      credentials
    );
    const workspace = parseLocalBenchmarkWorkspace(response);
    if (workspace.decision_pair_witness !== undefined) {
      throw new TypeError("cohort benchmark workspace response must not contain decision identity");
    }
    const baselineMatches = requestedBaseline === undefined
      ? workspace.baseline === null
      : workspace.baseline?.commit_id === requestedBaseline.commitId
        && workspace.baseline.cohort_id === requestedBaseline.cohortId;
    if (
      workspace.project_id !== requestedProjectId
      || workspace.context_id !== requestedContextId
      || workspace.revised.commit_id !== requestedCommitId
      || workspace.revised.cohort_id !== requestedCohortId
      || !baselineMatches
    ) {
      throw new TypeError("benchmark workspace response does not match the requested scope");
    }
    return workspace;
  }

  async getBenchmarkWorkspaceForDecision(
    projectId: string,
    contextId: string,
    commitId: string,
    decisionId: string,
    credentials: LocalLifecycleReadCredentials,
    baseline?: LocalBenchmarkWorkspaceDecisionBaselineRequest
  ): Promise<LocalBenchmarkWorkspace> {
    const requestedProjectId = requireNonEmpty(projectId, "projectId");
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedCommitId = requireNonEmpty(commitId, "commitId");
    const requestedDecisionId = requireNonEmpty(decisionId, "decisionId");
    const requestedBaseline = baseline === undefined
      ? undefined
      : {
          commitId: requireNonEmpty(baseline.commitId, "baseline.commitId"),
          decisionId: requireNonEmpty(baseline.decisionId, "baseline.decisionId")
        };
    let query = "";
    if (requestedBaseline !== undefined) {
      const params = new URLSearchParams();
      params.set("baseline_commit_id", requestedBaseline.commitId);
      params.set("baseline_decision_id", requestedBaseline.decisionId);
      query = `?${params.toString()}`;
    }
    const response = await this.request<unknown>(
      `/api/v1/local/projects/${encodePathSegment(requestedProjectId)}/contexts/${encodePathSegment(requestedContextId)}/commits/${encodePathSegment(requestedCommitId)}/benchmark-decisions/${encodePathSegment(requestedDecisionId)}/workspace${query}`,
      credentials
    );
    const workspace = parseLocalBenchmarkWorkspace(response);
    const witness = workspace.decision_pair_witness;
    const baselineMatches = requestedBaseline === undefined
      ? workspace.baseline === null && witness === undefined
      : workspace.baseline?.commit_id === requestedBaseline.commitId
        && witness !== undefined
        && witness.project_id === requestedProjectId
        && witness.context_id === requestedContextId
        && witness.baseline.commit_id === requestedBaseline.commitId
        && witness.baseline.decision_id === requestedBaseline.decisionId
        && witness.revised.commit_id === requestedCommitId
        && witness.revised.decision_id === requestedDecisionId;
    if (
      workspace.project_id !== requestedProjectId
      || workspace.context_id !== requestedContextId
      || workspace.revised.commit_id !== requestedCommitId
      || !baselineMatches
    ) {
      throw new TypeError("benchmark decision workspace response does not match the requested scope");
    }
    return workspace;
  }

  async getWorkflowContextBindings(
    contextId: string,
    commitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalWorkflowContextBindings> {
    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const requestedCommitId = requireNonEmpty(commitId, "commitId");
    const response = await this.request<unknown>(
      `/api/v1/local/contexts/${encodePathSegment(requestedContextId)}/commits/${encodePathSegment(requestedCommitId)}/workflow-bindings`,
      credentials
    );
    const bindings = parseLocalWorkflowContextBindings(response);
    if (bindings.context_id !== requestedContextId || bindings.commit_id !== requestedCommitId) {
      throw new TypeError("workflow Context bindings response does not match the requested scope");
    }

    return bindings;
  }

  async commitComponentLifecycle(
    contextId: string,
    command: LocalComponentLifecycleCommitRequest,
    credentials: LocalLifecycleWriteCredentials
  ): Promise<LocalComponentLifecycleCommitResponse> {
    const idempotencyKey = requireNonEmpty(credentials.idempotencyKey, "idempotencyKey");
    const parsedCommand = parseLocalComponentLifecycleCommitRequest(command);

    const requestedContextId = requireNonEmpty(contextId, "contextId");
    const response = parseLocalComponentLifecycleCommitResponse(await this.request<unknown>(
      `/api/v1/local/contexts/${encodePathSegment(contextId)}/component-lifecycle-commits`,
      credentials,
      {
        method: "POST",
        headers: {
          "content-type": "application/json",
          "idempotency-key": idempotencyKey
        },
        body: JSON.stringify(parsedCommand)
      }
    ));
    if (
      response.snapshot.context_id !== requestedContextId
      || response.snapshot.commit_id !== response.commit_id
    ) {
      throw new TypeError("component lifecycle commit response does not match the requested scope");
    }
    return response;
  }

  private async request<TResponse>(
    path: string,
    credentials: LocalLifecycleReadCredentials,
    init: RequestInit = {}
  ): Promise<TResponse> {
    const bearerToken = requireNonEmpty(credentials.bearerToken, "bearerToken");
    const headers = new Headers(init.headers);
    headers.set("accept", "application/json");
    headers.set("authorization", `Bearer ${bearerToken}`);
    const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
      ...init,
      credentials: "omit",
      cache: "no-store",
      headers
    });

    if (!response.ok) {
      throw new ContextLabLocalApiError(
        response.status,
        await parseErrorBody(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    return response.json() as Promise<TResponse>;
  }
}

function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.replace(/\/+$/, "");
}

function encodePathSegment(value: string): string {
  return encodeURIComponent(value);
}

function requireNonEmpty(value: string, field: string): string {
  if (value.trim().length === 0) {
    throw new RangeError(`${field} must be non-empty`);
  }

  return value;
}

async function parseErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalApiErrorBody>;

    if (typeof body.error === "string" && typeof body.message === "string") {
      return {
        error: body.error,
        message: body.message
      };
    }
  } catch {
  }

  return {
    error: "contextlab_local_api_error",
    message: `ContextLab local API request failed with status ${response.status}`
  };
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) {
    return undefined;
  }

  const delaySeconds = Number(value);
  if (Number.isFinite(delaySeconds) && delaySeconds >= 0) {
    return delaySeconds * 1_000;
  }

  const retryAt = Date.parse(value);
  if (Number.isNaN(retryAt)) {
    return undefined;
  }

  return Math.max(0, retryAt - Date.now());
}
