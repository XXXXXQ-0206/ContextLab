import { parseLocalContextMergeReviewV1 } from "../../../../../../../local-context-merge-review-data";

export const dynamic = "force-dynamic";

export async function GET(
  request: Request,
  context: { params: Promise<{ projectId: string; contextId: string }> }
): Promise<Response> {
  if (request.body !== null) {
    return jsonResponse({
      error: "invalid_local_context_merge_review_request",
      message: "Query parameters and request bodies are not supported"
    }, 400);
  }
  const url = new URL(request.url);
  const queryEntries = [...url.searchParams.entries()];
  const leftCommitId = singleQueryValue(queryEntries, "left_commit_id");
  const rightCommitId = singleQueryValue(queryEntries, "right_commit_id");
  if (queryEntries.length !== 2 || leftCommitId === undefined || rightCommitId === undefined) {
    return jsonResponse({
      error: "invalid_local_context_merge_review_request",
      message: "Exact left_commit_id and right_commit_id are required"
    }, 400);
  }
  const authorization = request.headers.get("authorization")?.trim();
  if (!authorization?.startsWith("Bearer ") || authorization.slice("Bearer ".length).trim().length === 0) {
    return jsonResponse({ error: "authentication_required", message: "Bearer authentication is required" }, 401);
  }
  const apiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL?.trim();
  if (!apiBaseUrl) return jsonResponse({ error: "contextlab_web_api_unavailable", message: "ContextLab Web API base URL is not configured" }, 503);

  const { projectId, contextId } = await context.params;
  const upstreamUrl = `${apiBaseUrl.replace(/\/+$/, "")}/api/v1/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/merge-review?${new URLSearchParams({ left_commit_id: leftCommitId, right_commit_id: rightCommitId }).toString()}`;
  try {
    const upstream = await fetch(upstreamUrl, {
      method: "GET",
      credentials: "omit",
      cache: "no-store",
      headers: {
        accept: "application/json",
        authorization
      }
    });
    const retryAfter = retryAfterHeader(upstream.headers.get("retry-after"));
    if (!upstream.ok) return jsonResponse({ error: "contextlab_web_api_error", message: "Unable to load local Context merge review" }, upstream.status, retryAfter);
    const review = parseLocalContextMergeReviewV1(await upstream.json());
    if (
      review.base_scope.project_id !== projectId
      || review.base_scope.context_id !== contextId
      || review.left_scope.commit_id !== leftCommitId
      || review.right_scope.commit_id !== rightCommitId
    ) throw new TypeError("upstream Context merge review scope drift");
    return jsonResponse(review);
  } catch {
    return jsonResponse({ error: "contextlab_web_api_error", message: "Unable to load local Context merge review" }, 502);
  }
}

function singleQueryValue(entries: Array<[string, string]>, name: string): string | undefined {
  const values = entries.filter(([key]) => key === name).map(([, value]) => value);
  return values.length === 1 && values[0]?.trim() ? values[0] : undefined;
}

function retryAfterHeader(value: string | null): HeadersInit | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? { "retry-after": value } : undefined;
}

function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  const responseHeaders = new Headers(headers);
  responseHeaders.set("content-type", "application/json");
  responseHeaders.set("cache-control", "private, no-store");
  return new Response(JSON.stringify(body), { headers: responseHeaders, status });
}
