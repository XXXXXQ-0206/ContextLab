import {
  ContextLabLocalApiError,
  ContextLabLocalClient,
  type LocalBenchmarkDecisionScope,
  type LocalComponentLifecycleCommitRequest
} from "@contextlab/local-sdk";

const localLifecycleDevelopmentFlag = "CONTEXTLAB_ENABLE_LOCAL_LIFECYCLE";
const localContextMergeReviewDevelopmentFlag = "CONTEXTLAB_ENABLE_LOCAL_CONTEXT_MERGE_REVIEW";

export function isLocalLifecycleDevelopmentEnabled(
  environment: Record<string, string | undefined> = process.env
): boolean {
  return environment[localLifecycleDevelopmentFlag] === "true";
}

export function isLocalContextMergeReviewDevelopmentEnabled(
  environment: Record<string, string | undefined> = process.env
): boolean {
  return environment[localContextMergeReviewDevelopmentFlag] === "true";
}

export async function proxyLocalContextLifecycleState(
  request: Request,
  contextId: string,
  commitId: string
): Promise<Response> {
  const bearerToken = requestBearerToken(request);

  if (!bearerToken) {
    return jsonResponse(authenticationRequired(), 401);
  }

  const client = localClient();

  if (!client) {
    return jsonResponse(apiUnavailable(), 503);
  }

  try {
    return jsonResponse(
      await client.getContextLifecycleState(contextId, commitId, {
        bearerToken
      })
    );
  } catch (error) {
    return localApiErrorResponse(error, "Unable to load local Context lifecycle state");
  }
}

export async function proxyLocalBenchmarkDecision(
  request: Request,
  projectId: string,
  contextId: string,
  commitId: string,
  decisionId: string
): Promise<Response> {
  const bearerToken = requestBearerToken(request);

  if (!bearerToken) {
    return jsonResponse(authenticationRequired(), 401);
  }

  const client = localClient();

  if (!client) {
    return jsonResponse(apiUnavailable(), 503);
  }

  try {
    return jsonResponse(
      await client.getBenchmarkDecision(projectId, contextId, commitId, decisionId, {
        bearerToken
      })
    );
  } catch (error) {
    return localApiErrorResponse(error, "Unable to load local benchmark decision evidence");
  }
}

export async function proxyLocalBenchmarkDecisionRunDetails(
  request: Request,
  projectId: string,
  contextId: string,
  commitId: string,
  decisionId: string
): Promise<Response> {
  const bearerToken = requestBearerToken(request);

  if (!bearerToken) {
    return jsonResponse(authenticationRequired(), 401);
  }

  const client = localClient();

  if (!client) {
    return jsonResponse(apiUnavailable(), 503);
  }

  try {
    return jsonResponse(
      await client.getBenchmarkDecisionRunDetails(projectId, contextId, commitId, decisionId, {
        bearerToken
      })
    );
  } catch (error) {
    return localApiErrorResponse(error, "Unable to load local benchmark decision run details");
  }
}

export async function proxyLocalBenchmarkDecisionDiff(
  request: Request,
  projectId: string,
  contextId: string,
  baseline: LocalBenchmarkDecisionScope,
  revised: LocalBenchmarkDecisionScope
): Promise<Response> {
  const bearerToken = requestBearerToken(request);

  if (!bearerToken) {
    return jsonResponse(authenticationRequired(), 401);
  }

  const client = localClient();

  if (!client) {
    return jsonResponse(apiUnavailable(), 503);
  }

  try {
    return jsonResponse(
      await client.getBenchmarkDecisionDiff(projectId, contextId, baseline, revised, {
        bearerToken
      })
    );
  } catch (error) {
    return localApiErrorResponse(error, "Unable to load local benchmark decision comparison");
  }
}

export async function proxyLocalWorkflowCapabilityStatus(
  request: Request,
  contextId: string
): Promise<Response> {
  const bearerToken = requestBearerToken(request);

  if (!bearerToken) {
    return jsonResponse(authenticationRequired(), 401);
  }

  const client = localClient();

  if (!client) {
    return jsonResponse(apiUnavailable(), 503);
  }

  try {
    return jsonResponse(
      await client.getWorkflowCapabilityStatus(contextId, {
        bearerToken
      })
    );
  } catch (error) {
    return localApiErrorResponse(error, "Unable to load local workflow capability status");
  }
}

export async function proxyLocalComponentLifecycleCommit(
  request: Request,
  contextId: string
): Promise<Response> {
  if (!isLocalLifecycleDevelopmentEnabled()) {
    return jsonResponse(localLifecycleDisabled(), 403);
  }

  const bearerToken = requestBearerToken(request);

  if (!bearerToken) {
    return jsonResponse(authenticationRequired(), 401);
  }

  const idempotencyKey = request.headers.get("idempotency-key")?.trim();

  if (!idempotencyKey) {
    return jsonResponse(
      {
        error: "invalid_context_lifecycle_request",
        message: "Idempotency-Key is required"
      },
      400
    );
  }

  let command: LocalComponentLifecycleCommitRequest;
  try {
    command = (await request.json()) as LocalComponentLifecycleCommitRequest;
  } catch {
    return jsonResponse(
      {
        error: "invalid_context_lifecycle_request",
        message: "Request body must be valid JSON"
      },
      400
    );
  }

  const client = localClient();

  if (!client) {
    return jsonResponse(apiUnavailable(), 503);
  }

  try {
    const result = await client.commitComponentLifecycle(contextId, command, {
      bearerToken,
      idempotencyKey
    });
    return jsonResponse(result, result.disposition === "created" ? 201 : 200);
  } catch (error) {
    return localApiErrorResponse(error, "Unable to write local Context lifecycle state");
  }
}

function requestBearerToken(request: Request): string | null {
  const authorization = request.headers.get("authorization")?.trim();

  if (!authorization?.startsWith("Bearer ")) {
    return null;
  }

  const bearerToken = authorization.slice("Bearer ".length).trim();
  return bearerToken || null;
}

function localClient(): ContextLabLocalClient | null {
  const apiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL?.trim();

  if (!apiBaseUrl) {
    return null;
  }

  return new ContextLabLocalClient({
    baseUrl: apiBaseUrl,
    fetch: (input, init) => fetch(input, { ...init, cache: "no-store" })
  });
}

function localApiErrorResponse(error: unknown, fallbackMessage: string): Response {
  if (error instanceof ContextLabLocalApiError) {
    return jsonResponse(error.body, error.status, retryAfterHeader(error.retryAfterMs));
  }

  return jsonResponse(
    {
      error: "contextlab_web_api_error",
      message: fallbackMessage
    },
    502
  );
}

function authenticationRequired() {
  return {
    error: "authentication_required",
    message: "Bearer authentication is required"
  };
}

function localLifecycleDisabled() {
  return {
    error: "local_lifecycle_disabled",
    message: "Local Context lifecycle mutations are disabled"
  };
}

function apiUnavailable() {
  return {
    error: "contextlab_web_api_unavailable",
    message: "ContextLab Web API base URL is not configured"
  };
}

function retryAfterHeader(retryAfterMs: number | undefined): HeadersInit | undefined {
  if (retryAfterMs === undefined) {
    return undefined;
  }

  return { "retry-after": String(Math.ceil(retryAfterMs / 1_000)) };
}

function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  const responseHeaders = new Headers(headers);
  responseHeaders.set("content-type", "application/json");
  responseHeaders.set("cache-control", "private, no-store");

  return new Response(JSON.stringify(body), {
    headers: responseHeaders,
    status
  });
}
