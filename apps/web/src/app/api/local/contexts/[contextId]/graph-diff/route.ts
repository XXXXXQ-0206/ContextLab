export const dynamic = "force-dynamic";

export async function GET(
  request: Request,
  context: { params: Promise<{ contextId: string }> }
): Promise<Response> {
  const { contextId } = await context.params;
  const query = new URL(request.url).searchParams;
  const originalCommitId = query.get("original_commit_id")?.trim();
  const revisedCommitId = query.get("revised_commit_id")?.trim();

  if (
    !contextId.trim()
    || !isUuid(contextId)
    || !originalCommitId
    || !revisedCommitId
    || !isUuid(originalCommitId)
    || !isUuid(revisedCommitId)
    || originalCommitId === revisedCommitId
    || query.getAll("original_commit_id").length !== 1
    || query.getAll("revised_commit_id").length !== 1
    || [...query.keys()].some((key) => key !== "original_commit_id" && key !== "revised_commit_id")
    || request.body !== null
  ) {
    return jsonResponse(
      {
        error: "invalid_local_commit_graph_diff_request",
        message: "A Context ID and distinct original_commit_id and revised_commit_id values are required"
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
      { error: "contextlab_web_api_unavailable", message: "ContextLab Web API base URL is not configured" },
      503
    );
  }

  try {
    const client = new ContextLabLocalClient({
      baseUrl: apiBaseUrl,
      fetch: (input, init) => fetch(input, { ...init, cache: "no-store", credentials: "omit" })
    });
    return jsonResponse(
      await client.getCommitGraphDiff(contextId, originalCommitId, revisedCommitId, { bearerToken })
    );
  } catch (error) {
    if (error instanceof ContextLabLocalCommitGraphDiffError) {
      return jsonResponse(
        { error: error.code, message: error.message },
        error.status,
        error.retryAfterMs === undefined ? undefined : { "retry-after": String(Math.ceil(error.retryAfterMs / 1_000)) }
      );
    }

    return jsonResponse(
      { error: "contextlab_web_api_error", message: "Unable to load the local commit graph diff" },
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
import {
  ContextLabLocalCommitGraphDiffError,
  ContextLabLocalClient
} from "@contextlab/local-sdk";
