import { ContextLabLocalApiError, ContextLabLocalClient } from "@contextlab/local-sdk";

export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{
    projectId: string;
    contextId: string;
    commitId: string;
    cohortId: string;
  }>;
};

type BaselineScope = Readonly<{
  commitId: string;
  cohortId: string;
}>;

export async function GET(request: Request, context: RouteContext): Promise<Response> {
  const baseline = parseBaselineScope(new URL(request.url));
  if (baseline === null || request.body !== null) {
    return jsonResponse(
      {
        error: "invalid_local_benchmark_workspace_request",
        message: "Only a complete baseline_commit_id and baseline_cohort_id pair is supported"
      },
      400
    );
  }

  const bearerToken = parseBearerToken(request.headers.get("authorization"));
  if (bearerToken === null) {
    return jsonResponse(
      { error: "authentication_required", message: "Bearer authentication is required" },
      401
    );
  }

  const apiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL?.trim();
  if (!apiBaseUrl) {
    return jsonResponse(
      {
        error: "contextlab_web_api_unavailable",
        message: "ContextLab Web API base URL is not configured"
      },
      503
    );
  }

  const { projectId, contextId, commitId, cohortId } = await context.params;
  const client = new ContextLabLocalClient({
    baseUrl: apiBaseUrl,
    fetch: (input, init) => fetch(input, { ...init, cache: "no-store", credentials: "omit" })
  });

  try {
    const workspace = await client.getBenchmarkWorkspace(
      projectId,
      contextId,
      commitId,
      cohortId,
      { bearerToken },
      baseline ?? undefined
    );
    return jsonResponse(workspace);
  } catch (error) {
    if (error instanceof ContextLabLocalApiError) {
      return jsonResponse(error.body, error.status, retryAfterHeader(error.retryAfterMs));
    }
    return jsonResponse(
      {
        error: "contextlab_web_api_error",
        message: "Unable to load local benchmark workspace"
      },
      502
    );
  }
}

function parseBaselineScope(url: URL): BaselineScope | undefined | null {
  const entries = [...url.searchParams.entries()];
  if (entries.length === 0) {
    return undefined;
  }

  const allowedKeys = new Set(["baseline_commit_id", "baseline_cohort_id"]);
  if (
    entries.length !== 2
    || entries.some(([key]) => !allowedKeys.has(key))
    || url.searchParams.getAll("baseline_commit_id").length !== 1
    || url.searchParams.getAll("baseline_cohort_id").length !== 1
  ) {
    return null;
  }

  const commitId = url.searchParams.get("baseline_commit_id") ?? "";
  const cohortId = url.searchParams.get("baseline_cohort_id") ?? "";
  if (!commitId.trim() || !cohortId.trim()) {
    return null;
  }

  return Object.freeze({ commitId, cohortId });
}

function parseBearerToken(value: string | null): string | null {
  const match = value?.match(/^Bearer ([^\s]+)$/);
  return match?.[1] ?? null;
}

function retryAfterHeader(retryAfterMs: number | undefined): HeadersInit | undefined {
  return retryAfterMs === undefined
    ? undefined
    : { "retry-after": String(Math.ceil(retryAfterMs / 1_000)) };
}

function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  const responseHeaders = new Headers(headers);
  responseHeaders.set("content-type", "application/json");
  responseHeaders.set("cache-control", "private, no-store");
  return new Response(JSON.stringify(body), { headers: responseHeaders, status });
}
