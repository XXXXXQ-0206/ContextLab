import {
  parseLocalBenchmarkExecutionRequest,
  parseLocalBenchmarkExecutionResponse,
  type LocalApiErrorBody,
  type LocalBenchmarkExecutionRequestV1,
  type LocalBenchmarkExecutionResponseV1
} from "@contextlab/local-sdk";

export type LocalBenchmarkExecutionTarget = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
}>;

export type LocalBenchmarkExecutionResourceState =
  | "loading"
  | "error"
  | "empty"
  | "created"
  | "replayed"
  | "conflict"
  | "unavailable";

type LocalBenchmarkExecutionResourceWithoutReceipt = Readonly<{
  state: Exclude<LocalBenchmarkExecutionResourceState, "created" | "replayed">;
  target: LocalBenchmarkExecutionTarget;
  message?: string;
}>;

type LocalBenchmarkExecutionReceiptResource = Readonly<{
  state: "created" | "replayed";
  target: LocalBenchmarkExecutionTarget;
  receipt: LocalBenchmarkExecutionResponseV1;
}>;

export type LocalBenchmarkExecutionResource =
  | LocalBenchmarkExecutionResourceWithoutReceipt
  | LocalBenchmarkExecutionReceiptResource;

export class LocalBenchmarkExecutionProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<LocalApiErrorBody>;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalBenchmarkExecutionProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
    this.retryAfterMs = retryAfterMs;
  }
}

export async function executeLocalBenchmarkExecution(
  target: LocalBenchmarkExecutionTarget,
  request: LocalBenchmarkExecutionRequestV1,
  bearerToken: string,
  idempotencyKey: string
): Promise<LocalBenchmarkExecutionResponseV1> {
  const exactTarget = freezeTarget(target);
  const parsedRequest = parseLocalBenchmarkExecutionRequest(request);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const key = requireNonBlank(idempotencyKey, "idempotencyKey").trim();
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(exactTarget.projectId)}`
      + `/contexts/${encodeURIComponent(exactTarget.contextId)}`
      + `/commits/${encodeURIComponent(exactTarget.commitId)}/benchmark-executions`,
    {
      method: "POST",
      credentials: "omit",
      cache: "no-store",
      headers: {
        accept: "application/json",
        authorization: `Bearer ${token}`,
        "content-type": "application/json",
        "idempotency-key": key
      },
      body: JSON.stringify(parsedRequest)
    }
  );
  if (!response.ok) {
    throw new LocalBenchmarkExecutionProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }
  const receipt = parseLocalBenchmarkExecutionResponse(await response.json());
  if (
    receipt.project_id !== exactTarget.projectId
    || receipt.context_id !== exactTarget.contextId
    || receipt.commit_id !== exactTarget.commitId
    || receipt.binding_id !== parsedRequest.binding_id
    || receipt.decision_id !== parsedRequest.decision_id
  ) {
    throw new TypeError("benchmark execution receipt does not match the requested exact scope");
  }
  return receipt;
}

export function createLocalBenchmarkExecutionResource(
  input:
    | Readonly<{
        state: Exclude<LocalBenchmarkExecutionResourceState, "created" | "replayed">;
        target: LocalBenchmarkExecutionTarget;
        message?: string;
      }>
    | Readonly<{
        state: "created" | "replayed";
        target: LocalBenchmarkExecutionTarget;
        receipt: LocalBenchmarkExecutionResponseV1;
      }>
): LocalBenchmarkExecutionResource {
  const target = freezeTarget(input.target);
  if (input.state === "created" || input.state === "replayed") {
    if (input.receipt.disposition !== input.state) {
      throw new TypeError("execution resource state must match receipt disposition");
    }
    if (
      input.receipt.project_id !== target.projectId
      || input.receipt.context_id !== target.contextId
      || input.receipt.commit_id !== target.commitId
    ) {
      throw new TypeError("execution receipt is outside the requested target");
    }
    return Object.freeze({ state: input.state, target, receipt: deepFreeze(input.receipt) });
  }
  return Object.freeze({
    state: input.state,
    target,
    ...("message" in input && input.message === undefined ? {} : "message" in input ? { message: input.message } : {})
  });
}

function freezeTarget(target: LocalBenchmarkExecutionTarget): LocalBenchmarkExecutionTarget {
  return Object.freeze({
    projectId: requireNonBlank(target.projectId, "projectId"),
    contextId: requireNonBlank(target.contextId, "contextId"),
    commitId: requireNonBlank(target.commitId, "commitId")
  });
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const value = await response.json();
    if (
      typeof value === "object"
      && value !== null
      && !Array.isArray(value)
      && typeof value.error === "string"
      && typeof value.message === "string"
      && value.error.trim()
      && value.message.trim()
    ) {
      return { error: value.error, message: value.message };
    }
  } catch {
  }
  return { error: "contextlab_web_api_error", message: "The local benchmark execution adapter is unavailable / 本地 Benchmark 执行适配器不可用。" };
}

function requireNonBlank(value: string, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) throw new TypeError(`${field} must be non-blank`);
  return value;
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
