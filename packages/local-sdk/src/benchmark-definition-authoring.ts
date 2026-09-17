import {
  ContextLabLocalApiError,
  type ContextLabLocalClientOptions,
  type FetchLike
} from "./client";
import type {
  LocalApiErrorBody,
  LocalBenchmarkMetricKind,
  LocalBenchmarkThresholdDirection,
  LocalJsonValue,
  LocalLifecycleReadCredentials,
  LocalLifecycleWriteCredentials
} from "./types";

export const LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1 =
  "contextlab.local-benchmark-definition-authoring.v1" as const;
export const LOCAL_BENCHMARK_DEFINITION_AUTHORING_SCHEMA_V1 =
  LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1;
export const LOCAL_BENCHMARK_DEFINITION_BINDING_INSPECTION_SCHEMA_V1 =
  "contextlab.local-benchmark-definition-binding-inspection.v1" as const;

const DEFAULT_BASE_URL = "http://127.0.0.1:3100";
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
const METRIC_ORDER: readonly LocalBenchmarkMetricKind[] = [
  "latency_ms",
  "cost_usd",
  "accuracy",
  "hallucination_rate",
  "tool_usage_count",
  "token_count",
  "execution_time_ms",
  "output_quality",
  "success_rate"
];

export type LocalBenchmarkExpectedOutput =
  | { mode: "unspecified" }
  | { mode: "exact"; value: LocalJsonValue };

export type LocalBenchmarkDefinitionCase = {
  id: string;
  name: string;
  input: LocalJsonValue;
  expected_output: LocalBenchmarkExpectedOutput;
};

export type LocalBenchmarkDefinitionDataset = {
  id: string;
  name: string;
  cases: LocalBenchmarkDefinitionCase[];
};

export type LocalBenchmarkDefinitionThreshold = {
  metric: LocalBenchmarkMetricKind;
  direction: LocalBenchmarkThresholdDirection;
  value: number;
};

export type LocalBenchmarkDefinitionSuite = {
  id: string;
  name: string;
  dataset_ids: string[];
  thresholds: LocalBenchmarkDefinitionThreshold[];
};

export type LocalBenchmarkDefinitionBindingRequest = {
  schema_version: 1;
  binding_id: string;
  branch_name: string;
  expected_head_commit_id: string;
  datasets: LocalBenchmarkDefinitionDataset[];
  suite: LocalBenchmarkDefinitionSuite;
};

export type LocalBilingualText = {
  en: string;
  zh: string;
};

export type LocalBenchmarkDefinitionBinding = {
  schema_version: typeof LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1;
  disposition: "created" | "replayed";
  message: LocalBilingualText;
  binding_id: string;
  project_id: string;
  context_id: string;
  commit_id: string;
  branch_name: string;
  definition_schema_version: 1;
  dataset_ids: string[];
  suite_id: string;
  captured_at: string;
};

export type LocalBenchmarkDefinitionBindingSummary = {
  binding_id: string;
  project_id: string;
  context_id: string;
  commit_id: string;
  branch_name: string;
  definition_schema_version: 1;
  suite_id: string;
  suite_name: string;
  dataset_ids: string[];
  dataset_names: string[];
  captured_at: string;
};

export type LocalBenchmarkDefinitionBindingList = {
  schema_version: typeof LOCAL_BENCHMARK_DEFINITION_BINDING_INSPECTION_SCHEMA_V1;
  project_id: string;
  context_id: string;
  commit_id: string;
  bindings: LocalBenchmarkDefinitionBindingSummary[];
};

export type LocalBenchmarkDefinitionAuthoringRequestV1 =
  LocalBenchmarkDefinitionBindingRequest;
export type LocalBenchmarkDefinitionAuthoringResponseV1 =
  LocalBenchmarkDefinitionBinding;
export type LocalBenchmarkDefinitionBindingV1 = LocalBenchmarkDefinitionBinding;
export type LocalBenchmarkDefinitionCaseAuthoringV1 = LocalBenchmarkDefinitionCase;
export type LocalBenchmarkDefinitionDatasetAuthoringV1 = LocalBenchmarkDefinitionDataset;
export type LocalBenchmarkDefinitionSuiteAuthoringV1 = LocalBenchmarkDefinitionSuite;
export type LocalBenchmarkDefinitionThresholdV1 = LocalBenchmarkDefinitionThreshold;

export class ContextLabLocalBenchmarkDefinitionReplayConflictError
  extends ContextLabLocalApiError {
  constructor(status: number, body: LocalApiErrorBody) {
    super(status, body);
    this.name = "ContextLabLocalBenchmarkDefinitionReplayConflictError";
  }
}

export class ContextLabLocalBenchmarkDefinitionConflictError extends ContextLabLocalApiError {
  constructor(status: number, body: LocalApiErrorBody) {
    super(status, body);
    this.name = "ContextLabLocalBenchmarkDefinitionConflictError";
  }
}

export class ContextLabLocalBenchmarkDefinitionClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = options.baseUrl?.replace(/\/+$/, "") ?? DEFAULT_BASE_URL;
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async createBenchmarkDefinitionBinding(
    projectId: string,
    contextId: string,
    commitId: string,
    request: LocalBenchmarkDefinitionBindingRequest,
    credentials: LocalLifecycleWriteCredentials
  ): Promise<LocalBenchmarkDefinitionBinding> {
    const requestedProjectId = asUuid(projectId, "projectId");
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedCommitId = asUuid(commitId, "commitId");
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken");
    const idempotencyKey = asNonBlank(credentials.idempotencyKey, "idempotencyKey");
    const parsedRequest = parseLocalBenchmarkDefinitionAuthoringRequest(request);
    if (parsedRequest.expected_head_commit_id !== requestedCommitId) {
      throw new TypeError("expected_head_commit_id must match the exact Context commit scope");
    }

    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/projects/${encodeURIComponent(requestedProjectId)}`
        + `/contexts/${encodeURIComponent(requestedContextId)}`
        + `/commits/${encodeURIComponent(requestedCommitId)}/benchmark-definition-bindings`,
      {
        method: "POST",
        credentials: "omit",
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
      if (response.status === 409 && body.error === "storage_idempotency_conflict") {
        throw new ContextLabLocalBenchmarkDefinitionReplayConflictError(response.status, body);
      }
      if (response.status === 409 && isAuthoringConflictCode(body.error)) {
        throw new ContextLabLocalBenchmarkDefinitionConflictError(response.status, body);
      }
      throw new ContextLabLocalApiError(response.status, body);
    }

    const parsed = parseLocalBenchmarkDefinitionBinding(await response.json());
    if (
      parsed.project_id !== requestedProjectId
      || parsed.context_id !== requestedContextId
      || parsed.commit_id !== requestedCommitId
      || parsed.binding_id !== parsedRequest.binding_id
      || parsed.branch_name !== parsedRequest.branch_name
      || parsed.suite_id !== parsedRequest.suite.id
      || !sameStrings(parsed.dataset_ids, parsedRequest.suite.dataset_ids)
    ) {
      throw new TypeError("benchmark definition authoring response does not match the request scope");
    }
    return parsed;
  }

  async authorBenchmarkDefinition(
    projectId: string,
    contextId: string,
    commitId: string,
    request: LocalBenchmarkDefinitionBindingRequest,
    credentials: LocalLifecycleWriteCredentials
  ): Promise<LocalBenchmarkDefinitionBinding> {
    return this.createBenchmarkDefinitionBinding(
      projectId, contextId, commitId, request, credentials
    );
  }

  async listBenchmarkDefinitionBindings(
    projectId: string,
    contextId: string,
    commitId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalBenchmarkDefinitionBindingList> {
    const requestedProjectId = asUuid(projectId, "projectId");
    const requestedContextId = asUuid(contextId, "contextId");
    const requestedCommitId = asUuid(commitId, "commitId");
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken");
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/projects/${encodeURIComponent(requestedProjectId)}`
        + `/contexts/${encodeURIComponent(requestedContextId)}`
        + `/commits/${encodeURIComponent(requestedCommitId)}/benchmark-definition-bindings`,
      {
        method: "GET",
        credentials: "omit",
        cache: "no-store",
        headers: {
          accept: "application/json",
          authorization: `Bearer ${bearerToken}`
        }
      }
    );
    if (!response.ok) {
      throw new ContextLabLocalApiError(response.status, await parseErrorBody(response));
    }
    const parsed = parseLocalBenchmarkDefinitionBindingList(await response.json());
    if (
      parsed.project_id !== requestedProjectId
      || parsed.context_id !== requestedContextId
      || parsed.commit_id !== requestedCommitId
    ) {
      throw new TypeError("benchmark definition binding list does not match the requested scope");
    }
    return parsed;
  }
}

export { ContextLabLocalBenchmarkDefinitionClient as ContextLabLocalBenchmarkDefinitionAuthoringClient };
export {
  ContextLabLocalBenchmarkDefinitionClient as ContextLabLocalBenchmarkDefinitionBindingInspectionClient
};

export function parseLocalBenchmarkDefinitionAuthoringRequest(
  value: unknown
): LocalBenchmarkDefinitionBindingRequest {
  const record = asRecord(value, "benchmark definition authoring request");
  assertExactKeys(record, [
    "schema_version",
    "binding_id",
    "branch_name",
    "expected_head_commit_id",
    "datasets",
    "suite"
  ]);
  if (record.schema_version !== 1) {
    throw new TypeError("schema_version must be 1");
  }
  const datasets = asArray(record.datasets, "datasets").map(parseDataset);
  assertNonEmpty(datasets, "datasets");
  assertOrderedUnique(datasets.map((dataset) => dataset.id), "datasets");
  const suite = parseSuite(record.suite);
  if (!sameStrings(datasets.map((dataset) => dataset.id), suite.dataset_ids)) {
    throw new TypeError("suite.dataset_ids must exactly match datasets in stable identifier order");
  }
  return {
    schema_version: 1,
    binding_id: asUuid(record.binding_id, "binding_id"),
    branch_name: asNonBlank(record.branch_name, "branch_name"),
    expected_head_commit_id: asUuid(record.expected_head_commit_id, "expected_head_commit_id"),
    datasets,
    suite
  };
}

export function parseLocalBenchmarkDefinitionBinding(
  value: unknown
): LocalBenchmarkDefinitionBinding {
  const record = asRecord(value, "benchmark definition authoring response");
  assertExactKeys(record, [
    "schema_version",
    "disposition",
    "message",
    "binding_id",
    "project_id",
    "context_id",
    "commit_id",
    "branch_name",
    "definition_schema_version",
    "dataset_ids",
    "suite_id",
    "captured_at"
  ]);
  if (record.schema_version !== LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1) {
    throw new TypeError(
      `schema_version must be ${LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1}`
    );
  }
  if (record.disposition !== "created" && record.disposition !== "replayed") {
    throw new TypeError("disposition must be created or replayed");
  }
  if (record.definition_schema_version !== 1) {
    throw new TypeError("definition_schema_version must be 1");
  }
  const messageRecord = asRecord(record.message, "message");
  assertExactKeys(messageRecord, ["en", "zh"]);
  const datasetIds = asArray(record.dataset_ids, "dataset_ids")
    .map((item) => asUuid(item, "dataset_ids[]"));
  assertNonEmpty(datasetIds, "dataset_ids");
  assertOrderedUnique(datasetIds, "dataset_ids");
  return Object.freeze({
    schema_version: LOCAL_BENCHMARK_DEFINITION_BINDING_SCHEMA_V1,
    disposition: record.disposition,
    message: Object.freeze({
      en: asNonBlank(messageRecord.en, "message.en"),
      zh: asNonBlank(messageRecord.zh, "message.zh")
    }),
    binding_id: asUuid(record.binding_id, "binding_id"),
    project_id: asUuid(record.project_id, "project_id"),
    context_id: asUuid(record.context_id, "context_id"),
    commit_id: asUuid(record.commit_id, "commit_id"),
    branch_name: asNonBlank(record.branch_name, "branch_name"),
    definition_schema_version: 1,
    dataset_ids: Object.freeze(datasetIds) as unknown as string[],
    suite_id: asUuid(record.suite_id, "suite_id"),
    captured_at: asTimestamp(record.captured_at, "captured_at")
  });
}

export const parseLocalBenchmarkDefinitionAuthoringResponse =
  parseLocalBenchmarkDefinitionBinding;

export function parseLocalBenchmarkDefinitionBindingList(
  value: unknown
): LocalBenchmarkDefinitionBindingList {
  const record = asRecord(value, "benchmark definition binding list");
  assertExactKeys(record, ["schema_version", "project_id", "context_id", "commit_id", "bindings"]);
  if (record.schema_version !== LOCAL_BENCHMARK_DEFINITION_BINDING_INSPECTION_SCHEMA_V1) {
    throw new TypeError(
      `schema_version must be ${LOCAL_BENCHMARK_DEFINITION_BINDING_INSPECTION_SCHEMA_V1}`
    );
  }
  const projectId = asUuid(record.project_id, "project_id");
  const contextId = asUuid(record.context_id, "context_id");
  const commitId = asUuid(record.commit_id, "commit_id");
  const bindings = asArray(record.bindings, "bindings").map((item) =>
    parseLocalBenchmarkDefinitionBindingSummary(item, projectId, contextId, commitId)
  );
  assertOrderedUnique(bindings.map((binding) => binding.binding_id), "bindings");
  return Object.freeze({
    schema_version: LOCAL_BENCHMARK_DEFINITION_BINDING_INSPECTION_SCHEMA_V1,
    project_id: projectId,
    context_id: contextId,
    commit_id: commitId,
    bindings: Object.freeze(bindings) as unknown as LocalBenchmarkDefinitionBindingSummary[]
  });
}

function parseLocalBenchmarkDefinitionBindingSummary(
  value: unknown,
  projectId: string,
  contextId: string,
  commitId: string
): LocalBenchmarkDefinitionBindingSummary {
  const record = asRecord(value, "benchmark definition binding summary");
  assertExactKeys(record, [
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
  ]);
  if (record.definition_schema_version !== 1) {
    throw new TypeError("definition_schema_version must be 1");
  }
  const summaryProjectId = asUuid(record.project_id, "binding.project_id");
  const summaryContextId = asUuid(record.context_id, "binding.context_id");
  const summaryCommitId = asUuid(record.commit_id, "binding.commit_id");
  if (
    summaryProjectId !== projectId
    || summaryContextId !== contextId
    || summaryCommitId !== commitId
  ) {
    throw new TypeError("benchmark definition binding summary is outside the requested scope");
  }
  const datasetIds = asArray(record.dataset_ids, "binding.dataset_ids")
    .map((item) => asUuid(item, "binding.dataset_ids[]"));
  const datasetNames = asArray(record.dataset_names, "binding.dataset_names")
    .map((item) => asNonBlank(item, "binding.dataset_names[]"));
  assertNonEmpty(datasetIds, "binding.dataset_ids");
  assertOrderedUnique(datasetIds, "binding.dataset_ids");
  if (datasetIds.length !== datasetNames.length) {
    throw new TypeError("binding dataset IDs and names must have equal length");
  }
  return Object.freeze({
    binding_id: asUuid(record.binding_id, "binding.binding_id"),
    project_id: summaryProjectId,
    context_id: summaryContextId,
    commit_id: summaryCommitId,
    branch_name: asNonBlank(record.branch_name, "binding.branch_name"),
    definition_schema_version: 1,
    suite_id: asUuid(record.suite_id, "binding.suite_id"),
    suite_name: asNonBlank(record.suite_name, "binding.suite_name"),
    dataset_ids: Object.freeze(datasetIds) as unknown as string[],
    dataset_names: Object.freeze(datasetNames) as unknown as string[],
    captured_at: asTimestamp(record.captured_at, "binding.captured_at")
  });
}

function parseDataset(value: unknown): LocalBenchmarkDefinitionDataset {
  const record = asRecord(value, "dataset");
  assertExactKeys(record, ["id", "name", "cases"]);
  const cases = asArray(record.cases, "dataset.cases").map(parseCase);
  assertNonEmpty(cases, "dataset.cases");
  assertOrderedUnique(cases.map((item) => item.id), "dataset.cases");
  return {
    id: asUuid(record.id, "dataset.id"),
    name: asNonBlank(record.name, "dataset.name"),
    cases
  };
}

function parseCase(value: unknown): LocalBenchmarkDefinitionCase {
  const record = asRecord(value, "case");
  assertExactKeys(record, ["id", "name", "input", "expected_output"]);
  return {
    id: asUuid(record.id, "case.id"),
    name: asNonBlank(record.name, "case.name"),
    input: parseJsonValue(record.input, "case.input"),
    expected_output: parseExpectedOutput(record.expected_output)
  };
}

function parseExpectedOutput(value: unknown): LocalBenchmarkExpectedOutput {
  const record = asRecord(value, "case.expected_output");
  if (record.mode === "unspecified") {
    assertExactKeys(record, ["mode"]);
    return { mode: "unspecified" };
  }
  if (record.mode === "exact") {
    assertExactKeys(record, ["mode", "value"]);
    return { mode: "exact", value: parseJsonValue(record.value, "expected_output.value") };
  }
  throw new TypeError("case.expected_output.mode is unsupported");
}

function parseSuite(value: unknown): LocalBenchmarkDefinitionSuite {
  const record = asRecord(value, "suite");
  assertExactKeys(record, ["id", "name", "dataset_ids", "thresholds"]);
  const datasetIds = asArray(record.dataset_ids, "suite.dataset_ids")
    .map((item) => asUuid(item, "suite.dataset_ids[]"));
  assertNonEmpty(datasetIds, "suite.dataset_ids");
  assertOrderedUnique(datasetIds, "suite.dataset_ids");
  const thresholds = asArray(record.thresholds, "suite.thresholds").map(parseThreshold);
  assertNonEmpty(thresholds, "suite.thresholds");
  assertThresholdOrder(thresholds);
  return {
    id: asUuid(record.id, "suite.id"),
    name: asNonBlank(record.name, "suite.name"),
    dataset_ids: datasetIds,
    thresholds
  };
}

function parseThreshold(value: unknown): LocalBenchmarkDefinitionThreshold {
  const record = asRecord(value, "threshold");
  assertExactKeys(record, ["metric", "direction", "value"]);
  return {
    metric: asMetric(record.metric),
    direction: asDirection(record.direction),
    value: asFiniteNumber(record.value, "threshold.value")
  };
}

function parseJsonValue(value: unknown, field: string): LocalJsonValue {
  if (value === null || typeof value === "string" || typeof value === "boolean") {
    return value;
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) {
      throw new TypeError(`${field} must contain only finite JSON numbers`);
    }
    return value;
  }
  if (Array.isArray(value)) {
    return value.map((item, index) => parseJsonValue(item, `${field}[${index}]`));
  }
  if (typeof value === "object" && value !== null) {
    return Object.fromEntries(
      Object.entries(value).map(([key, nested]) => [
        key, parseJsonValue(nested, `${field}.${key}`)
      ])
    );
  }
  throw new TypeError(`${field} must contain only JSON values`);
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as Record<string, unknown>;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new TypeError(`${field} must be an array`);
  }
  return value;
}

function asNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be a non-blank string`);
  }
  return value;
}

function asUuid(value: unknown, field: string): string {
  const parsed = asNonBlank(value, field);
  if (!UUID_PATTERN.test(parsed)) {
    throw new TypeError(`${field} must be a lowercase canonical UUID`);
  }
  return parsed;
}

function asFiniteNumber(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new TypeError(`${field} must be finite`);
  }
  return value;
}

function asTimestamp(value: unknown, field: string): string {
  const parsed = asNonBlank(value, field);
  if (Number.isNaN(Date.parse(parsed))) {
    throw new TypeError(`${field} must be a valid timestamp`);
  }
  return parsed;
}

function asMetric(value: unknown): LocalBenchmarkMetricKind {
  if (typeof value !== "string" || !METRIC_ORDER.includes(value as LocalBenchmarkMetricKind)) {
    throw new TypeError("threshold.metric is unsupported");
  }
  return value as LocalBenchmarkMetricKind;
}

function asDirection(value: unknown): LocalBenchmarkThresholdDirection {
  if (value !== "minimum" && value !== "maximum") {
    throw new TypeError("threshold.direction is unsupported");
  }
  return value;
}

function assertExactKeys(record: Record<string, unknown>, expected: readonly string[]): void {
  const actual = Object.keys(record).sort();
  const required = [...expected].sort();
  if (!sameStrings(actual, required)) {
    throw new TypeError("benchmark definition authoring value contains an unexpected shape");
  }
}

function assertNonEmpty(values: readonly unknown[], field: string): void {
  if (values.length === 0) {
    throw new TypeError(`${field} must not be empty`);
  }
}

function assertOrderedUnique(values: readonly string[], field: string): void {
  for (let index = 1; index < values.length; index += 1) {
    if (values[index - 1]! >= values[index]!) {
      throw new TypeError(`${field} must be unique and ordered by stable identifier`);
    }
  }
}

function assertThresholdOrder(thresholds: readonly LocalBenchmarkDefinitionThreshold[]): void {
  for (let index = 1; index < thresholds.length; index += 1) {
    if (
      METRIC_ORDER.indexOf(thresholds[index - 1]!.metric)
      >= METRIC_ORDER.indexOf(thresholds[index]!.metric)
    ) {
      throw new TypeError("suite.thresholds must be unique and ordered by stable metric");
    }
  }
}

function sameStrings(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function isAuthoringConflictCode(code: string): boolean {
  return code === "storage_branch_head_conflict"
    || code === "storage_benchmark_definition_conflict"
    || code === "storage_benchmark_definition_binding_conflict";
}

async function parseErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const record = asRecord(await response.json(), "error response");
    assertExactKeys(record, ["error", "message"]);
    return {
      error: asNonBlank(record.error, "error"),
      message: asNonBlank(record.message, "message")
    };
  } catch {
    return {
      error: "contextlab_local_api_error",
      message: `ContextLab local API request failed with status ${response.status}`
    };
  }
}
