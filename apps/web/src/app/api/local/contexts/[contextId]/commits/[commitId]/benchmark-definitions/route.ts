export const dynamic = "force-dynamic";

type RouteContext = {
  params: Promise<{ contextId: string; commitId: string }>;
};

export type BenchmarkDefinitionAuthoringScope = Readonly<{
  projectId?: string;
  contextId: string;
  commitId: string;
}>;

type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };
type JsonRecord = { [key: string]: JsonValue };

const REQUEST_KEYS = [
  "schema_version",
  "binding_id",
  "branch_name",
  "expected_head_commit_id",
  "datasets",
  "suite"
];

const RESPONSE_KEYS = [
  "schema_version",
  "disposition",
  "project_id",
  "context_id",
  "commit_id",
  "binding_id",
  "branch_name",
  "definition_schema_version",
  "dataset_ids",
  "suite_id",
  "captured_at",
  "message"
];

const INSPECTION_RESPONSE_KEYS = [
  "schema_version",
  "project_id",
  "context_id",
  "commit_id",
  "bindings"
];

export async function POST(request: Request, context: RouteContext): Promise<Response> {
  void request;
  void context;
  return jsonResponse(
    {
      error: "benchmark_definition_route_gone",
      message: "Use the project-scoped benchmark-definition-bindings route"
    },
    410,
    { "sunset": "2026-07-27" }
  );
}

export async function postBenchmarkDefinitionAuthoring(
  request: Request,
  scope: BenchmarkDefinitionAuthoringScope
): Promise<Response> {
  const { projectId, contextId, commitId } = scope;
  const url = new URL(request.url);

  if (url.search || !hasExactEncodedScope(url, scope)) {
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

  const authoringRequest = await parseAuthoringRequest(request);
  if (authoringRequest === null) {
    return invalidRequest("Request body must be a valid benchmark definition authoring request");
  }

  if (authoringRequest.expected_head_commit_id !== commitId) {
    return invalidRequest("expected_head_commit_id must match the exact route commit");
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
    upstream = await fetch(
      upstreamUrl(apiBaseUrl, contextId, commitId, projectId),
      {
        method: "POST",
        headers: {
          accept: "application/json",
          authorization: `Bearer ${bearerToken}`,
          "content-type": "application/json",
          "idempotency-key": idempotencyKey
        },
        body: JSON.stringify(authoringRequest),
        cache: "no-store",
        credentials: "omit"
      }
    );
  } catch {
    return unavailableUpstream();
  }

  if (!upstream.ok) {
    return upstreamErrorResponse(upstream);
  }

  try {
    const payload = await upstream.json();
    if (!isSafeAuthoringResponse(payload, contextId, commitId, authoringRequest)) {
      return unavailableUpstream();
    }
    return jsonResponse(payload, upstream.status);
  } catch {
    return unavailableUpstream();
  }
}

export async function getBenchmarkDefinitionBindings(
  request: Request,
  scope: Required<BenchmarkDefinitionAuthoringScope>
): Promise<Response> {
  const { projectId, contextId, commitId } = scope;
  const url = new URL(request.url);
  if (url.search || !hasExactEncodedScope(url, scope)) {
    return invalidRequest("Query parameters and mismatched route scopes are not supported");
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

  let upstream: Response;
  try {
    upstream = await fetch(upstreamUrl(apiBaseUrl, contextId, commitId, projectId), {
      method: "GET",
      headers: {
        accept: "application/json",
        authorization: `Bearer ${bearerToken}`
      },
      cache: "no-store",
      credentials: "omit"
    });
  } catch {
    return unavailableInspectionUpstream();
  }

  if (!upstream.ok) {
    return upstreamErrorResponse(upstream);
  }

  try {
    const payload = await upstream.json();
    if (!isSafeBindingListResponse(payload, projectId, contextId, commitId)) {
      return unavailableInspectionUpstream();
    }
    return jsonResponse(payload, upstream.status);
  } catch {
    return unavailableInspectionUpstream();
  }
}

function hasExactEncodedScope(url: URL, scope: BenchmarkDefinitionAuthoringScope): boolean {
  const expectedPath = scope.projectId === undefined
    ? [
      "/api/local/contexts",
      encodeURIComponent(scope.contextId),
      "commits",
      encodeURIComponent(scope.commitId),
      "benchmark-definitions"
    ].join("/")
    : [
      "/api/local/projects",
      encodeURIComponent(scope.projectId),
      "contexts",
      encodeURIComponent(scope.contextId),
      "commits",
      encodeURIComponent(scope.commitId),
      "benchmark-definition-bindings"
    ].join("/");
  return url.pathname === expectedPath;
}

function parseBearerToken(authorization: string | null): string | null {
  const match = authorization?.match(/^Bearer ([^\s]+)$/);
  return match?.[1] ?? null;
}

async function parseAuthoringRequest(request: Request): Promise<JsonRecord | null> {
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
    || !isNonBlankString(value.branch_name)
    || !isNonBlankString(value.expected_head_commit_id)
    || !Array.isArray(value.datasets)
    || value.datasets.length === 0
    || !isAuthoringSuite(value.suite)
    || !value.datasets.every(isAuthoringDataset)
  ) {
    return null;
  }

  return value;
}

function isAuthoringDataset(value: JsonValue): value is JsonRecord {
  return isRecord(value)
    && hasExactKeys(value, ["id", "name", "cases"])
    && isNonBlankString(value.id)
    && isNonBlankString(value.name)
    && Array.isArray(value.cases)
    && value.cases.length > 0
    && value.cases.every(isAuthoringCase);
}

function isAuthoringCase(value: JsonValue): value is JsonRecord {
  return isRecord(value)
    && hasExactKeys(value, ["id", "name", "input", "expected_output"])
    && isNonBlankString(value.id)
    && isNonBlankString(value.name)
    && isJsonValue(value.input)
    && isExpectedOutput(value.expected_output);
}

function isExpectedOutput(value: JsonValue): value is JsonRecord {
  if (!isRecord(value) || !isNonBlankString(value.mode)) {
    return false;
  }

  if (value.mode === "unspecified") {
    return hasExactKeys(value, ["mode"]);
  }

  return value.mode === "exact" && hasExactKeys(value, ["mode", "value"]) && isJsonValue(value.value);
}

function isAuthoringSuite(value: JsonValue): value is JsonRecord {
  return isRecord(value)
    && hasExactKeys(value, ["id", "name", "dataset_ids", "thresholds"])
    && isNonBlankString(value.id)
    && isNonBlankString(value.name)
    && Array.isArray(value.dataset_ids)
    && value.dataset_ids.length > 0
    && value.dataset_ids.every(isNonBlankString)
    && Array.isArray(value.thresholds)
    && value.thresholds.length > 0
    && value.thresholds.every(isThreshold);
}

function isThreshold(value: JsonValue): value is JsonRecord {
  return isRecord(value)
    && hasExactKeys(value, ["metric", "direction", "value"])
    && isNonBlankString(value.metric)
    && (value.direction === "minimum" || value.direction === "maximum")
    && typeof value.value === "number"
    && Number.isFinite(value.value);
}

function isSafeAuthoringResponse(
  value: unknown,
  contextId: string,
  commitId: string,
  request: JsonRecord
): value is JsonRecord {
  if (!isRecord(value) || !hasExactKeys(value, RESPONSE_KEYS, ["message"])) {
    return false;
  }

  return value.schema_version === "contextlab.local-benchmark-definition-authoring.v1"
    && (value.disposition === "created" || value.disposition === "replayed")
    && isNonBlankString(value.project_id)
    && value.context_id === contextId
    && value.commit_id === commitId
    && isNonBlankString(value.binding_id)
    && isNonBlankString(value.branch_name)
    && value.definition_schema_version === 1
    && Array.isArray(value.dataset_ids)
    && value.dataset_ids.length > 0
    && value.dataset_ids.every(isNonBlankString)
    && isNonBlankString(value.suite_id)
    && isNonBlankString(value.captured_at)
    && value.binding_id === request.binding_id
    && value.branch_name === request.branch_name
    && value.suite_id === (request.suite as JsonRecord).id
    && sameStrings(value.dataset_ids, (request.suite as JsonRecord).dataset_ids as JsonValue[])
    && (value.message === undefined || isBilingualMessage(value.message));
}

function isSafeBindingListResponse(
  value: unknown,
  projectId: string,
  contextId: string,
  commitId: string
): value is JsonRecord {
  if (!isRecord(value) || !hasExactKeys(value, INSPECTION_RESPONSE_KEYS)) return false;
  if (
    value.schema_version !== "contextlab.local-benchmark-definition-binding-inspection.v1"
    || value.project_id !== projectId
    || value.context_id !== contextId
    || value.commit_id !== commitId
    || !Array.isArray(value.bindings)
  ) return false;

  let previousBindingId: string | undefined;
  return value.bindings.every((binding) => {
    if (!isRecord(binding) || !hasExactKeys(binding, [
      "binding_id",
      "project_id",
      "context_id",
      "commit_id",
      "branch_name",
      "definition_schema_version",
      "suite_id",
      "suite_name",
      "dataset_ids",
      "dataset_names",
      "captured_at"
    ])) return false;
    const bindingId = binding.binding_id;
    const datasetIds = binding.dataset_ids;
    const datasetNames = binding.dataset_names;
    if (
      !isNonBlankString(bindingId)
      || (previousBindingId !== undefined && previousBindingId >= bindingId)
      || binding.project_id !== projectId
      || binding.context_id !== contextId
      || binding.commit_id !== commitId
      || binding.definition_schema_version !== 1
      || !isNonBlankString(binding.branch_name)
      || !isNonBlankString(binding.suite_id)
      || !isNonBlankString(binding.suite_name)
      || !isNonBlankString(binding.captured_at)
      || !Array.isArray(datasetIds)
      || datasetIds.length === 0
      || !datasetIds.every(isNonBlankString)
      || !Array.isArray(datasetNames)
      || datasetIds.length !== datasetNames.length
      || !datasetNames.every(isNonBlankString)
    ) return false;
    previousBindingId = bindingId;
    return true;
  });
}

function isBilingualMessage(value: JsonValue): value is JsonRecord {
  return isRecord(value)
    && hasExactKeys(value, ["en", "zh"])
    && isNonBlankString(value.en)
    && isNonBlankString(value.zh);
}

function sameStrings(left: JsonValue[], right: JsonValue[]): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

async function upstreamErrorResponse(upstream: Response): Promise<Response> {
  try {
    const value = await upstream.json();
    if (isTypedError(value)) {
      return jsonResponse(
        { error: value.error, message: value.message },
        upstream.status,
        retryAfterHeader(upstream.headers.get("retry-after"))
      );
    }
  } catch {
  }

  return unavailableUpstream();
}

function isTypedError(value: unknown): value is { error: string; message: string } {
  return isRecord(value) && isNonBlankString(value.error) && isNonBlankString(value.message);
}

function upstreamUrl(
  apiBaseUrl: string,
  contextId: string,
  commitId: string,
  projectId?: string
): string {
  const base = apiBaseUrl.replace(/\/+$/, "");
  if (projectId === undefined) {
    return `${base}/api/v1/local/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/benchmark-definitions`;
  }
  return `${base}/api/v1/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/benchmark-definition-bindings`;
}

function retryAfterHeader(value: string | null): HeadersInit | undefined {
  return value === null ? undefined : { "retry-after": value };
}

function invalidRequest(message: string): Response {
  return jsonResponse(
    { error: "invalid_local_benchmark_definition_request", message },
    400
  );
}

function unavailableUpstream(): Response {
  return jsonResponse(
    {
      error: "contextlab_web_api_error",
      message: "Unable to author local benchmark definitions"
    },
    502
  );
}

function unavailableInspectionUpstream(): Response {
  return jsonResponse(
    {
      error: "contextlab_web_api_error",
      message: "Unable to inspect local benchmark definition bindings"
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

function hasExactKeys(
  record: JsonRecord,
  expected: readonly string[],
  optional: readonly string[] = []
): boolean {
  const keys = Object.keys(record);
  return keys.length >= expected.length - optional.length
    && keys.length <= expected.length
    && keys.every((key) => expected.includes(key))
    && expected.every((key) => optional.includes(key) || Object.hasOwn(record, key));
}

function isNonBlankString(value: JsonValue): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isJsonValue(value: unknown): value is JsonValue {
  if (value === null || typeof value === "string" || typeof value === "boolean") {
    return true;
  }
  if (typeof value === "number") {
    return Number.isFinite(value);
  }
  if (Array.isArray(value)) {
    return value.every(isJsonValue);
  }
  return isRecord(value) && Object.values(value).every(isJsonValue);
}
