import {
  ContextLabLocalPersistedContextDiffReviewClient,
  ContextLabLocalPersistedContextDiffReviewError
} from "@contextlab/local-sdk";

export const dynamic = "force-dynamic";

export async function GET(
  request: Request,
  context: { params: Promise<{ projectId: string; contextId: string }> }
): Promise<Response> {
  const url = new URL(request.url);
  const keys = [...url.searchParams.keys()].sort();
  if (request.body !== null || keys.join(",") !== "source_commit_id,target_commit_id") {
    return jsonResponse(
      {
        error: "invalid_local_persisted_context_diff_review_request",
        message: "source_commit_id and target_commit_id are required; no other query fields are supported"
      },
      400
    );
  }

  const authorization = request.headers.get("authorization")?.trim();
  if (!authorization?.startsWith("Bearer ") || authorization.slice("Bearer ".length).trim().length === 0) {
    return jsonResponse({ error: "authentication_required", message: "Bearer authentication is required" }, 401);
  }

  const apiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL?.trim();
  if (!apiBaseUrl) {
    return jsonResponse(
      { error: "contextlab_web_api_unavailable", message: "ContextLab Web API base URL is not configured" },
      503
    );
  }

  const { projectId, contextId } = await context.params;
  const client = new ContextLabLocalPersistedContextDiffReviewClient({
    baseUrl: apiBaseUrl,
    fetch: (input, init) => fetch(input, { ...init, cache: "no-store" })
  });

  try {
    return jsonResponse(
      await client.getPersistedContextDiffReview(
        projectId,
        contextId,
        url.searchParams.get("source_commit_id") ?? "",
        url.searchParams.get("target_commit_id") ?? "",
        { bearerToken: authorization.slice("Bearer ".length).trim() }
      )
    );
  } catch (error) {
    if (error instanceof ContextLabLocalPersistedContextDiffReviewError) {
      return jsonResponse(
        { error: error.code, message: "Unable to load the local persisted Context diff review" },
        error.status,
        retryAfterHeader(error.retryAfterMs)
      );
    }
    return jsonResponse(
      { error: "contextlab_web_api_error", message: "Unable to load the local persisted Context diff review" },
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
