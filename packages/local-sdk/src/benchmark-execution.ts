import {
  ContextLabLocalApiError,
  type ContextLabLocalClientOptions,
  type FetchLike
} from "./client";
import type { LocalApiErrorBody, LocalLifecycleWriteCredentials } from "./types";

export const LOCAL_BENCHMARK_EXECUTION_SCHEMA_V1 = "contextlab.local-benchmark-execution.v1";

export type LocalBenchmarkExecutionRequestV1 = Readonly<{
  schema_version: 1;
  binding_id: string;
  decision_id: string;
  model_version: string;
  temperature: number;
  evaluator_key: string;
  evaluator_version: string;
}>;

export type LocalBenchmarkExecutionResponseV1 = Readonly<{
  schema_version: typeof LOCAL_BENCHMARK_EXECUTION_SCHEMA_V1;
  disposition: "created" | "replayed";
  projection_disposition: "created" | "replayed";
  project_id: string;
  context_id: string;
  commit_id: string;
  binding_id: string;
  decision_id: string;
  suite_id: string;
  dataset_ids: ReadonlyArray<string>;
  cohort_id: string;
}>;

export class ContextLabLocalBenchmarkExecutionConflictError extends ContextLabLocalApiError {
  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(status, body, retryAfterMs);
    this.name = "ContextLabLocalBenchmarkExecutionConflictError";
  }
}

export class ContextLabLocalBenchmarkExecutionClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async executeBenchmark(
    projectId: string,
    contextId: string,
    commitId: string,
    request: LocalBenchmarkExecutionRequestV1,
    credentials: LocalLifecycleWriteCredentials
  ): Promise<LocalBenchmarkExecutionResponseV1> {
    const requestedProjectId = asUuid(projectId, "projectId");
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedCommitId = asUuid(commitId, "commitId");
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const idempotencyKey = asNonBlank(credentials.idempotencyKey, "idempotencyKey").trim();
    const parsedRequest = parseLocalBenchmarkExecutionRequest(request);
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/projects/${encodeURIComponent(requestedProjectId)}`
        + `/contexts/${encodeURIComponent(requestedContextId)}`
        + `/commits/${encodeURIComponent(requestedCommitId)}/benchmark-executions`,
      {
        method: "POST",
        credentials: "omit",
        cache: "no-store",
        headers: {
          accept: "application/json",
          authorization: `Bearer ${bearerToken}`,
          "content-type": "application/json",
          "idempotency-key": idempotencyKey
        },
        body: JSON.stringify(parsedRequest)
      }
    );

    if (!response.ok) {
      const body = await parseErrorBody(response);
      if (response.status === 409) {
        throw new ContextLabLocalBenchmarkExecutionConflictError(
          response.status,
          body,
          parseRetryAfterMs(response.headers.get("retry-after"))
        );
      }
      throw new ContextLabLocalApiError(
        response.status,
        body,
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const parsedResponse = parseLocalBenchmarkExecutionResponse(await response.json());
    assertResponseScope(
      parsedResponse,
      requestedProjectId,
      requestedContextId,
      requestedCommitId,
      parsedRequest
    );
    return parsedResponse;
  }
}

export function parseLocalBenchmarkExecutionRequest(
  value: unknown
): LocalBenchmarkExecutionRequestV1 {
  const record = asRecord(value, "benchmark execution request");
  assertExactKeys(record, [
    "schema_version",
    "binding_id",
    "decision_id",
    "model_version",
    "temperature",
    "evaluator_key",
    "evaluator_version"
  ]);
  if (record.schema_version !== 1) throw new TypeError("schema_version must be 1");
  return freeze({
    schema_version: 1,
    binding_id: asUuid(record.binding_id, "binding_id"),
    decision_id: asUuid(record.decision_id, "decision_id"),
    model_version: asNonBlank(record.model_version, "model_version"),
    temperature: asTemperature(record.temperature),
    evaluator_key: asNonBlank(record.evaluator_key, "evaluator_key"),
    evaluator_version: asNonBlank(record.evaluator_version, "evaluator_version")
  });
}

export function parseLocalBenchmarkExecutionResponse(
  value: unknown
): LocalBenchmarkExecutionResponseV1 {
  const record = asRecord(value, "benchmark execution response");
  assertExactKeys(record, [
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
  ]);
  if (record.schema_version !== LOCAL_BENCHMARK_EXECUTION_SCHEMA_V1) {
    throw new TypeError(`schema_version must be ${LOCAL_BENCHMARK_EXECUTION_SCHEMA_V1}`);
  }
  if (record.disposition !== "created" && record.disposition !== "replayed") {
    throw new TypeError("disposition is invalid");
  }
  if (record.projection_disposition !== "created" && record.projection_disposition !== "replayed") {
    throw new TypeError("projection_disposition is invalid");
  }
  const datasetIds = asArray(record.dataset_ids, "dataset_ids").map((item) => asUuid(item, "dataset_ids[]"));
  assertOrderedUnique(datasetIds, "dataset_ids");
  return freeze({
    schema_version: LOCAL_BENCHMARK_EXECUTION_SCHEMA_V1,
    disposition: record.disposition,
    projection_disposition: record.projection_disposition,
    project_id: asUuid(record.project_id, "project_id"),
    context_id: asUuid(record.context_id, "context_id"),
    commit_id: asUuid(record.commit_id, "commit_id"),
    binding_id: asUuid(record.binding_id, "binding_id"),
    decision_id: asUuid(record.decision_id, "decision_id"),
    suite_id: asUuid(record.suite_id, "suite_id"),
    dataset_ids: Object.freeze(datasetIds),
    cohort_id: asUuid(record.cohort_id, "cohort_id")
  });
}

function assertResponseScope(
  response: LocalBenchmarkExecutionResponseV1,
  projectId: string,
  contextId: string,
  commitId: string,
  request: LocalBenchmarkExecutionRequestV1
): void {
  if (
    response.project_id !== projectId
    || response.context_id !== contextId
    || response.commit_id !== commitId
    || response.binding_id !== request.binding_id
    || response.decision_id !== request.decision_id
  ) {
    throw new TypeError("benchmark execution response does not match the requested scope");
  }
}

function parseErrorBody(response: Response): Promise<LocalApiErrorBody> {
  return response.json().then((value: unknown) => {
    const record = asRecord(value, "error response");
    assertExactKeys(record, ["error", "message"]);
    return {
      error: asNonBlank(record.error, "error"),
      message: asNonBlank(record.message, "message")
    };
  }).catch(() => ({
    error: "contextlab_local_api_error",
    message: `ContextLab local API request failed with status ${response.status}`
  }));
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0) return seconds * 1_000;
  const timestamp = Date.parse(value);
  return Number.isNaN(timestamp) ? undefined : Math.max(0, timestamp - Date.now());
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) throw new TypeError(`${field} must be an object`);
  return value as Record<string, unknown>;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) throw new TypeError(`${field} must be an array`);
  return value;
}

function asNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) throw new TypeError(`${field} must be non-blank`);
  return value;
}

function asUuid(value: unknown, field: string): string {
  const parsed = asNonBlank(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function asFiniteNumber(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new TypeError(`${field} must be finite`);
  return value;
}

function asTemperature(value: unknown): number {
  const temperature = asFiniteNumber(value, "temperature");
  if (temperature < 0 || temperature > 2) {
    throw new TypeError("temperature must be between 0.0 and 2.0");
  }
  return temperature;
}

function assertExactKeys(record: Record<string, unknown>, expected: readonly string[]): void {
  const actual = Object.keys(record).sort();
  const required = [...expected].sort();
  if (actual.length !== required.length || actual.some((key, index) => key !== required[index])) {
    throw new TypeError("benchmark execution value contains an unexpected shape");
  }
}

function assertOrderedUnique(values: readonly string[], field: string): void {
  for (let index = 1; index < values.length; index += 1) {
    if (values[index - 1]! >= values[index]!) throw new TypeError(`${field} must be unique and ordered`);
  }
}

function freeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
}
