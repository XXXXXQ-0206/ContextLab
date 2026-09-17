import {
  ContextLabLocalApiError,
  ContextLabLocalPluginCapabilityAvailabilityClient
} from "@contextlab/local-sdk";

export const dynamic = "force-dynamic";

export async function GET(
  request: Request,
  context: { params: Promise<{ contextId: string }> }
): Promise<Response> {
  if (new URL(request.url).search || request.body !== null) {
    return jsonResponse(
      {
        error: "invalid_local_plugin_capability_request",
        message: "Query parameters and request bodies are not supported"
      },
      400
    );
  }

  const authorization = request.headers.get("authorization")?.trim();
  if (!authorization?.startsWith("Bearer ") || authorization.slice("Bearer ".length).trim().length === 0) {
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

  const { contextId } = await context.params;
  const client = new ContextLabLocalPluginCapabilityAvailabilityClient({
    baseUrl: apiBaseUrl,
    fetch: (input, init) => fetch(input, { ...init, cache: "no-store" })
  });

  try {
    return jsonResponse(
      await client.getPluginCapabilityAvailability(contextId, {
        bearerToken: authorization.slice("Bearer ".length).trim()
      })
    );
  } catch (error) {
    if (error instanceof ContextLabLocalApiError) {
      return jsonResponse(error.body, error.status, retryAfterHeader(error.retryAfterMs));
    }
    if (error instanceof TypeError) {
      return jsonResponse(
        { error: "invalid_local_plugin_capability_response", message: "Plugin capability response is invalid" },
        502
      );
    }
    return jsonResponse(
      { error: "contextlab_web_api_error", message: "Unable to load local Plugin/MCP capabilities" },
      502
    );
  }
}

function retryAfterHeader(retryAfterMs: number | undefined): HeadersInit | undefined {
  return retryAfterMs === undefined ? undefined : { "retry-after": String(Math.ceil(retryAfterMs / 1_000)) };
}

function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  const responseHeaders = new Headers(headers);
  responseHeaders.set("content-type", "application/json");
  responseHeaders.set("cache-control", "private, no-store");
  return new Response(JSON.stringify(body), { headers: responseHeaders, status });
}
