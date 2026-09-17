import {
  ContextLabLocalBranchHeadError,
  ContextLabLocalClient
} from "@contextlab/local-sdk";

export const dynamic = "force-dynamic";

export async function GET(
  request: Request,
  context: { params: Promise<{ contextId: string }> }
): Promise<Response> {
  const { contextId } = await context.params;
  if (!isUuid(contextId) || request.body !== null) {
    return jsonResponse(
      { error: "invalid_local_context_branch_heads_request", message: "A valid Context ID is required" },
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
      { error: "contextlab_web_api_unavailable", message: "ContextLab Web API base URL is not configured" },
      503
    );
  }

  try {
    const client = new ContextLabLocalClient({
      baseUrl: apiBaseUrl,
      fetch: (input, init) => fetch(input, { ...init, cache: "no-store", credentials: "omit" })
    });
    return jsonResponse(await client.getContextBranchHeads(contextId, { bearerToken }));
  } catch (error) {
    if (error instanceof ContextLabLocalBranchHeadError) {
      return jsonResponse(
        { error: error.code, message: error.message },
        error.status,
        error.retryAfterMs === undefined
          ? undefined
          : { "retry-after": String(Math.ceil(error.retryAfterMs / 1_000)) }
      );
    }

    return jsonResponse(
      { error: "contextlab_web_api_error", message: "Unable to load local Context branch heads" },
      502
    );
  }
}

function requestBearerToken(request: Request): string | null {
  const match = request.headers.get("authorization")?.trim().match(/^Bearer ([^\s]+)$/);
  return match?.[1] ?? null;
}

function isUuid(value: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(value);
}

function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  const responseHeaders = new Headers(headers);
  responseHeaders.set("cache-control", "private, no-store");
  responseHeaders.set("content-type", "application/json");
  return new Response(JSON.stringify(body), { headers: responseHeaders, status });
}
