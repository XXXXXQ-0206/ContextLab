import {
  ContextLabLocalWorkflowExecutionStatusClient,
  ContextLabLocalWorkflowExecutionStatusError
} from "@contextlab/local-sdk";

export const dynamic = "force-dynamic";

export async function GET(
  request: Request,
  context: { params: Promise<{ contextId: string; runId: string }> }
): Promise<Response> {
  const { contextId, runId } = await context.params;
  if (
    request.method !== "GET"
    || new URL(request.url).search
    || request.body !== null
    || !isUuid(contextId)
    || !isUuid(runId)
  ) {
    return jsonResponse(
      {
        error: "invalid_local_workflow_execution_status_request",
        message: "Canonical Context and run UUIDs with no query or body are required"
      },
      400
    );
  }

  const bearerToken = requestBearerToken(request);
  if (!bearerToken) {
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

  const client = new ContextLabLocalWorkflowExecutionStatusClient({
    baseUrl: apiBaseUrl,
    fetch: (input, init) => fetch(input, { ...init, credentials: "omit", cache: "no-store" })
  });

  try {
    return jsonResponse(
      await client.getWorkflowExecutionStatus(contextId, runId, { bearerToken })
    );
  } catch (error) {
    if (error instanceof ContextLabLocalWorkflowExecutionStatusError) {
      return jsonResponse(
        { error: error.code, message: redactedErrorMessage(error.status) },
        error.status,
        error.retryAfterMs === undefined
          ? undefined
          : { "retry-after": String(Math.ceil(error.retryAfterMs / 1_000)) }
      );
    }
    if (error instanceof TypeError) {
      return jsonResponse(
        {
          error: "invalid_local_workflow_execution_status_response",
          message: "Local workflow execution status response is invalid"
        },
        502
      );
    }
    return jsonResponse(
      {
        error: "contextlab_web_api_error",
        message: "Unable to load local workflow execution status"
      },
      502
    );
  }
}

function redactedErrorMessage(status: number): string {
  return `ContextLab local API request failed with status ${status}`;
}

function requestBearerToken(request: Request): string | null {
  const match = request.headers.get("authorization")?.trim().match(/^Bearer ([^\s]+)$/);
  return match?.[1] ?? null;
}

function isUuid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value);
}

function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  const responseHeaders = new Headers(headers);
  responseHeaders.set("content-type", "application/json");
  responseHeaders.set("cache-control", "private, no-store");
  return new Response(JSON.stringify(body), { headers: responseHeaders, status });
}
