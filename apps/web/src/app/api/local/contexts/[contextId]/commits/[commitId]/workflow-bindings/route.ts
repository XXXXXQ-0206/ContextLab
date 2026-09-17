import {
  ContextLabLocalApiError,
  ContextLabLocalClient
} from "@contextlab/local-sdk";

export const dynamic = "force-dynamic";

export async function GET(
  request: Request,
  context: { params: Promise<{ contextId: string; commitId: string }> }
): Promise<Response> {
  const { contextId, commitId } = await context.params;
  const requestUrl = new URL(request.url);

  if (
    request.method !== "GET"
    || requestUrl.search
    || request.body !== null
    || !isNonEmptyScopeValue(contextId)
    || !isNonEmptyScopeValue(commitId)
    || !hasExactEncodedScope(requestUrl, contextId, commitId)
  ) {
    return jsonResponse(
      {
        error: "invalid_local_workflow_bindings_request",
        message: "Query parameters and request bodies are not supported"
      },
      400
    );
  }

  const bearerToken = requestBearerToken(request);

  if (!bearerToken) {
    return jsonResponse(
      {
        error: "authentication_required",
        message: "Bearer authentication is required"
      },
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

  const client = new ContextLabLocalClient({
    baseUrl: apiBaseUrl,
    fetch: (input, init) => fetch(input, { ...init, cache: "no-store" })
  });

  try {
    return jsonResponse(
      await client.getWorkflowContextBindings(contextId, commitId, {
        bearerToken
      })
    );
  } catch (error) {
    return localApiErrorResponse(error);
  }
}

function requestBearerToken(request: Request): string | null {
  const match = request.headers.get("authorization")?.trim().match(/^Bearer ([^\s]+)$/);
  return match?.[1] ?? null;
}

function hasExactEncodedScope(url: URL, contextId: string, commitId: string): boolean {
  return url.pathname === [
    "/api/local/contexts",
    encodeURIComponent(contextId),
    "commits",
    encodeURIComponent(commitId),
    "workflow-bindings"
  ].join("/");
}

function isNonEmptyScopeValue(value: string): boolean {
  return value.trim().length > 0;
}

function localApiErrorResponse(error: unknown): Response {
  if (error instanceof ContextLabLocalApiError) {
    return jsonResponse(
      { error: error.body.error, message: redactedErrorMessage(error.status) },
      error.status,
      retryAfterHeader(error.retryAfterMs)
    );
  }

  return jsonResponse(
    {
      error: "contextlab_web_api_error",
      message: "Unable to load local Workflow Context bindings"
    },
    502
  );
}

function redactedErrorMessage(status: number): string {
  return `ContextLab local API request failed with status ${status}`;
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
