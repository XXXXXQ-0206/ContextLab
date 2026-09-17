import type {
  ApiErrorBody,
  CommitDetail,
  CommitItem,
  CommitListQuery,
  ComponentDetail,
  ComponentItem,
  ComponentListQuery,
  ContextGraphResponse,
  ContextItem,
  ContextListQuery,
  EvaluationRunDetail,
  EvaluationRunItem,
  EvaluationRunListQuery,
  EvaluationScorecard,
  EvaluationScorecardQuery,
  ExperimentItem,
  ExperimentListQuery,
  GraphDiffRequest,
  GraphDiffResponse,
  HealthResponse,
  ListResponse,
  MetaResponse,
  OpenApiDocument,
  ProjectItem,
  ProjectListQuery,
  ProvidersResponse,
  WorkspaceItem,
  WorkspaceListQuery
} from "./types";

export type FetchLike = (input: string | URL, init?: RequestInit) => Promise<Response>;

export type ContextLabClientOptions = {
  baseUrl?: string;
  fetch?: FetchLike;
};

type QueryValue = string | number | undefined | null;
type QueryRecord = Record<string, QueryValue>;

const DEFAULT_BASE_URL = "http://127.0.0.1:3100";

export class ContextLabApiError extends Error {
  readonly status: number;
  readonly code: string;
  readonly body: ApiErrorBody;

  constructor(status: number, body: ApiErrorBody) {
    super(body.message);
    this.name = "ContextLabApiError";
    this.status = status;
    this.code = body.error;
    this.body = body;
  }
}

export class ContextLabClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabClientOptions = {}) {
    this.baseUrl = normalizeBaseUrl(options.baseUrl ?? DEFAULT_BASE_URL);
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async healthz(): Promise<HealthResponse> {
    return this.request("/healthz", {});
  }

  async getMeta(): Promise<MetaResponse> {
    return this.request("/api/v1/meta", {});
  }

  async getOpenApiDocument(): Promise<OpenApiDocument> {
    return this.request("/api/v1/openapi.json", {});
  }

  async listProviders(): Promise<ProvidersResponse> {
    return this.request("/api/v1/providers", {});
  }

  async listWorkspaces(query: WorkspaceListQuery = {}): Promise<ListResponse<WorkspaceItem>> {
    return this.request("/api/v1/workspaces", query);
  }

  async listProjects(
    workspaceId: string,
    query: ProjectListQuery = {}
  ): Promise<ListResponse<ProjectItem>> {
    return this.request(`/api/v1/workspaces/${encodePathSegment(workspaceId)}/projects`, query);
  }

  async listExperiments(
    projectId: string,
    query: ExperimentListQuery = {}
  ): Promise<ListResponse<ExperimentItem>> {
    return this.request(`/api/v1/projects/${encodePathSegment(projectId)}/experiments`, query);
  }

  async listContexts(
    projectId: string,
    query: ContextListQuery = {}
  ): Promise<ListResponse<ContextItem>> {
    return this.request(`/api/v1/projects/${encodePathSegment(projectId)}/contexts`, query);
  }

  async listCommits(
    contextId: string,
    query: CommitListQuery = {}
  ): Promise<ListResponse<CommitItem>> {
    return this.request(`/api/v1/contexts/${encodePathSegment(contextId)}/commits`, query);
  }

  async getCommit(contextId: string, commitId: string): Promise<CommitDetail> {
    return this.request(
      `/api/v1/contexts/${encodePathSegment(contextId)}/commits/${encodePathSegment(commitId)}`,
      {}
    );
  }

  async listComponents(
    contextId: string,
    query: ComponentListQuery = {}
  ): Promise<ListResponse<ComponentItem>> {
    return this.request(`/api/v1/contexts/${encodePathSegment(contextId)}/components`, query);
  }

  async getComponent(contextId: string, componentId: string): Promise<ComponentDetail> {
    return this.request(
      `/api/v1/contexts/${encodePathSegment(contextId)}/components/${encodePathSegment(componentId)}`,
      {}
    );
  }

  async listEvaluationRuns(
    contextId: string,
    query: EvaluationRunListQuery = {}
  ): Promise<ListResponse<EvaluationRunItem>> {
    return this.request(`/api/v1/contexts/${encodePathSegment(contextId)}/evaluation-runs`, query);
  }

  async getEvaluationScorecard(
    contextId: string,
    query: EvaluationScorecardQuery = {}
  ): Promise<EvaluationScorecard> {
    return this.request(
      `/api/v1/contexts/${encodePathSegment(contextId)}/evaluation-scorecard`,
      query
    );
  }

  async getEvaluationRun(contextId: string, runId: string): Promise<EvaluationRunDetail> {
    return this.request(
      `/api/v1/contexts/${encodePathSegment(contextId)}/evaluation-runs/${encodePathSegment(runId)}`,
      {}
    );
  }

  async getContextGraphPreview(): Promise<ContextGraphResponse> {
    return this.request("/api/v1/context-graph/preview", {});
  }

  async getWorkspaceContextGraph(workspaceId: string): Promise<ContextGraphResponse> {
    return this.request(
      `/api/v1/workspaces/${encodePathSegment(workspaceId)}/context-graph`,
      {}
    );
  }

  async compareGraphs(request: GraphDiffRequest): Promise<GraphDiffResponse> {
    return this.request("/api/v1/graph-diffs", {}, {
      method: "POST",
      headers: {
        "content-type": "application/json"
      },
      body: JSON.stringify(request)
    });
  }

  private async request<TResponse>(
    path: string,
    query: QueryRecord,
    init: RequestInit = {}
  ): Promise<TResponse> {
    const headers = new Headers(init.headers);
    headers.set("accept", "application/json");
    const response = await this.fetchImpl(this.url(path, query), { ...init, headers });

    if (!response.ok) {
      throw new ContextLabApiError(response.status, await parseErrorBody(response));
    }

    return response.json() as Promise<TResponse>;
  }

  private url(path: string, query: QueryRecord): string {
    const url = new URL(`${this.baseUrl}${path}`);

    for (const [key, value] of Object.entries(query)) {
      if (value !== undefined && value !== null) {
        url.searchParams.set(key, String(value));
      }
    }

    return url.toString();
  }
}

function normalizeBaseUrl(baseUrl: string): string {
  return baseUrl.replace(/\/+$/, "");
}

function encodePathSegment(value: string): string {
  return encodeURIComponent(value);
}

async function parseErrorBody(response: Response): Promise<ApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<ApiErrorBody>;

    if (typeof body.error === "string" && typeof body.message === "string") {
      return {
        error: body.error,
        message: body.message
      };
    }
  } catch {
    // Fall through to the generic error below when the server returns non-JSON.
  }

  return {
    error: "contextlab_api_error",
    message: `ContextLab API request failed with status ${response.status}`
  };
}
