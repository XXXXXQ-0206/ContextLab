export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{ projectId: string; contextId: string; commitId: string }>;
};

type JsonRecord = Record<string, unknown>;

const REQUEST_KEYS = [
  "schema_version",
  "binding_id",
  "decision_id",
  "model_version",
  "temperature",
  "evaluator_key",
  "evaluator_version"
] as const;

const RESPONSE_KEYS = [
  "schema_version",
  "disposition",
  "projection_disposition",
  "project_id",
  "context_id",
  "commit_id",
  "binding_id",
  "decision_id",
  "suite_id",
  "dataset_ids",
  "cohort_id"
] as const;

const RESPONSE_SCHEMA = "contextlab.local-benchmark-execution.v1";

export async function POST(request: Request, context: RouteContext): Promise<Response> {
  const scope = await context.params;
  const url = new URL(request.url);
  if (url.search || !hasExactPath(url, scope)) {
    return invalidRequest("Query parameters and mismatched route scopes are not supported");
  }

  const bearerToken = parseBearerToken(request.headers.get("authorization"));
  if (bearerToken === null) {
    return jsonResponse(
      { error: "authentication_required", message: "Bearer authentication is required" },
      401
    );
  }

  const idempotencyKey = request.headers.get("idempotency-key")?.trim();
  if (!idempotencyKey) {
    return invalidRequest("Idempotency-Key is required");
  }

  const executionRequest = await parseExecutionRequest(request);
  if (executionRequest === null) {
    return invalidRequest("Request body must be a valid benchmark execution request");
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

  let upstream: Response;
  try {
    upstream = await fetch(upstreamUrl(apiBaseUrl, scope), {
      method: "POST",
      headers: {
        accept: "application/json",
        authorization: `Bearer ${bearerToken}`,
        "content-type": "application/json",
        "idempotency-key": idempotencyKey
      },
      body: JSON.stringify(executionRequest),
      cache: "no-store",
      credentials: "omit"
    });
  } catch {
    return unavailableUpstream();
  }

  if (!upstream.ok) {
    return upstreamErrorResponse(upstream);
  }

  try {
    const payload = await upstream.json();
    if (!isSafeExecutionResponse(payload, scope, executionRequest)) {
      return unavailableUpstream();
    }
    return jsonResponse(payload, upstream.status);
  } catch {
    return unavailableUpstream();
  }
}

function hasExactPath(
  url: URL,
  scope: { projectId: string; contextId: string; commitId: string }
): boolean {
  return url.pathname === [
    "/api/local/projects",
    encodeURIComponent(scope.projectId),
    "contexts",
    encodeURIComponent(scope.contextId),
    "commits",
    encodeURIComponent(scope.commitId),
    "benchmark-executions"
  ].join("/");
}

function parseBearerToken(value: string | null): string | null {
  const match = value?.match(/^Bearer ([^\s]+)$/);
  return match?.[1] ?? null;
}

async function parseExecutionRequest(request: Request): Promise<JsonRecord | null> {
  let value: unknown;
  try {
    value = await request.json();
  } catch {
    return null;
  }

  if (!isRecord(value) || !hasExactKeys(value, REQUEST_KEYS)) {
    return null;
  }

  if (
    value.schema_version !== 1
    || !isNonBlankString(value.binding_id)
    || !isNonBlankString(value.decision_id)
    || !isNonBlankString(value.model_version)
    || !isSupportedTemperature(value.temperature)
    || !isNonBlankString(value.evaluator_key)
    || !isNonBlankString(value.evaluator_version)
  ) {
    return null;
  }

  return value;
}

function isSafeExecutionResponse(
  value: unknown,
  scope: { projectId: string; contextId: string; commitId: string },
  request: JsonRecord
): value is JsonRecord {
  if (!isRecord(value) || !hasExactKeys(value, RESPONSE_KEYS)) {
    return false;
  }

  return value.schema_version === RESPONSE_SCHEMA
    && (value.disposition === "created" || value.disposition === "replayed")
    && (value.projection_disposition === "created" || value.projection_disposition === "replayed")
    && value.project_id === scope.projectId
    && value.context_id === scope.contextId
    && value.commit_id === scope.commitId
    && value.binding_id === request.binding_id
    && value.decision_id === request.decision_id
    && isNonBlankString(value.suite_id)
    && isOrderedUniqueNonBlankStrings(value.dataset_ids)
    && isNonBlankString(value.cohort_id);
}

async function upstreamErrorResponse(upstream: Response): Promise<Response> {
  try {
    const value = await upstream.json();
    if (isTypedError(value) && stableErrorMessage(value.error) !== null) {
      return jsonResponse(
        { error: value.error, message: stableErrorMessage(value.error) },
        upstream.status,
        retryAfterHeader(upstream.headers.get("retry-after"))
      );
    }
  } catch {
  }

  return unavailableUpstream();
}

function isTypedError(value: unknown): value is { error: string; message: string } {
  return isRecord(value)
    && isNonBlankString(value.error)
    && isNonBlankString(value.message);
}

function stableErrorMessage(error: string): string | null {
  const messages: Record<string, string> = {
    authentication_required: "Bearer authentication is required / 需要 Bearer 身份验证。",
    context_write_forbidden: "This exact Context scope is forbidden / 无权访问此精确 Context 范围。",
    authorization_unavailable: "Authorization is unavailable / 授权服务暂不可用。",
    benchmark_execution_scope_not_found: "The exact benchmark execution scope was not found / 未找到精确 Benchmark 执行范围。",
    benchmark_execution_conflict: "The immutable benchmark execution conflicts with existing state / 不可变 Benchmark 执行与现有状态冲突。",
    benchmark_definition_not_found: "The bound benchmark definition is unavailable / 绑定的 Benchmark 定义不可用。",
    invalid_benchmark_execution_request: "The benchmark execution request is invalid / Benchmark 执行请求无效。",
    benchmark_evaluator_unavailable: "The local benchmark evaluator is unavailable / 本地 Benchmark 评测器不可用。",
    benchmark_execution_rate_limited: "The local benchmark execution rate limit is active / 本地 Benchmark 执行速率限制已生效。",
    benchmark_execution_unavailable: "The local benchmark execution adapter is unavailable / 本地 Benchmark 执行适配器不可用。"
  };
  return messages[error] ?? null;
}

function upstreamUrl(
  apiBaseUrl: string,
  scope: { projectId: string; contextId: string; commitId: string }
): string {
  const base = apiBaseUrl.replace(/\/+$/, "");
  return `${base}/api/v1/local/projects/${encodeURIComponent(scope.projectId)}/contexts/${encodeURIComponent(scope.contextId)}/commits/${encodeURIComponent(scope.commitId)}/benchmark-executions`;
}

function retryAfterHeader(value: string | null): HeadersInit | undefined {
  return value === null ? undefined : { "retry-after": value };
}

function invalidRequest(message: string): Response {
  return jsonResponse(
    { error: "invalid_local_benchmark_execution_request", message },
    400
  );
}

function unavailableUpstream(): Response {
  return jsonResponse(
    {
      error: "contextlab_web_api_error",
      message: "Unable to execute the local benchmark"
    },
    502
  );
}

function jsonResponse(body: unknown, status = 200, headers?: HeadersInit): Response {
  const responseHeaders = new Headers(headers);
  responseHeaders.set("content-type", "application/json");
  responseHeaders.set("cache-control", "private, no-store");
  return new Response(JSON.stringify(body), { headers: responseHeaders, status });
}

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasExactKeys(record: JsonRecord, expected: readonly string[]): boolean {
  const keys = Object.keys(record);
  return keys.length === expected.length && keys.every((key) => expected.includes(key));
}

function isNonBlankString(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function isSupportedTemperature(value: unknown): value is number {
  return isFiniteNumber(value) && value >= 0 && value <= 2;
}

function isOrderedUniqueNonBlankStrings(value: unknown): value is string[] {
  if (!Array.isArray(value) || value.length === 0 || !value.every(isNonBlankString)) {
    return false;
  }

  for (let index = 1; index < value.length; index += 1) {
    if (value[index - 1]! >= value[index]!) {
      return false;
    }
  }

  return true;
}
