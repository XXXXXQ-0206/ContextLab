import {
  parseLocalBenchmarkDefinitionAuthoringRequest,
  parseLocalBenchmarkDefinitionAuthoringResponse,
  type LocalApiErrorBody,
  type LocalBenchmarkDefinitionBinding,
  type LocalBenchmarkDefinitionBindingRequest,
  type LocalBenchmarkExpectedOutput,
  type LocalBenchmarkMetricKind,
  type LocalJsonValue
} from "@contextlab/local-sdk";

type DeepReadonly<T> = T extends (...args: never[]) => unknown
  ? T
  : T extends ReadonlyArray<infer TItem>
    ? ReadonlyArray<DeepReadonly<TItem>>
    : T extends object
      ? { readonly [TKey in keyof T]: DeepReadonly<T[TKey]> }
      : T;

export type LocalBenchmarkDefinitionMetric = LocalBenchmarkMetricKind;
export type LocalBenchmarkDefinitionJsonValue = LocalJsonValue;
export type LocalBenchmarkDefinitionExpectedOutput = LocalBenchmarkExpectedOutput;
export type LocalBenchmarkDefinitionAuthoringCommand = DeepReadonly<LocalBenchmarkDefinitionBindingRequest>;
export type LocalBenchmarkDefinitionAuthoringResult = DeepReadonly<LocalBenchmarkDefinitionBinding>;

export type LocalBenchmarkDefinitionAuthoringTarget = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
}>;

export type LocalBenchmarkDefinitionAuthoringState =
  | "loading"
  | "error"
  | "empty"
  | "available"
  | "unavailable";

type ResourceWithoutResult = Readonly<{
  state: Exclude<LocalBenchmarkDefinitionAuthoringState, "available">;
  target: LocalBenchmarkDefinitionAuthoringTarget;
  message?: string;
}>;

type AvailableResource = Readonly<{
  state: "available";
  target: LocalBenchmarkDefinitionAuthoringTarget;
  result: LocalBenchmarkDefinitionAuthoringResult;
}>;

export type LocalBenchmarkDefinitionAuthoringResource = ResourceWithoutResult | AvailableResource;
type ResourceInput = ResourceWithoutResult | AvailableResource;

export class LocalBenchmarkDefinitionAuthoringProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<LocalApiErrorBody>;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalBenchmarkDefinitionAuthoringProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
    this.retryAfterMs = retryAfterMs;
  }
}

export async function authorLocalBenchmarkDefinition(
  target: LocalBenchmarkDefinitionAuthoringTarget,
  bearerToken: string,
  idempotencyKey: string,
  command: LocalBenchmarkDefinitionAuthoringCommand
): Promise<LocalBenchmarkDefinitionAuthoringResult> {
  const frozenTarget = freezeTarget(target, true);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const requestKey = requireNonBlank(idempotencyKey, "idempotencyKey").trim();
  const parsedCommand = parseLocalBenchmarkDefinitionAuthoringRequest(command);
  if (parsedCommand.expected_head_commit_id !== frozenTarget.commitId) {
    throw new TypeError("benchmark definition command does not match the requested commit");
  }

  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(frozenTarget.projectId)}/contexts/${encodeURIComponent(frozenTarget.contextId)}/commits/${encodeURIComponent(frozenTarget.commitId)}/benchmark-definition-bindings`,
    {
      method: "POST",
      headers: {
        accept: "application/json",
        authorization: `Bearer ${token}`,
        "content-type": "application/json",
        "idempotency-key": requestKey
      },
      body: JSON.stringify(parsedCommand),
      credentials: "omit",
      cache: "no-store"
    }
  );

  if (!response.ok) {
    throw new LocalBenchmarkDefinitionAuthoringProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }

  const result = deepFreeze(parseLocalBenchmarkDefinitionAuthoringResponse(await response.json()));
  assertRequestedScope(result, frozenTarget, parsedCommand);
  return result;
}

export function parseLocalBenchmarkDefinitionAuthoringResult(
  value: unknown
): LocalBenchmarkDefinitionAuthoringResult {
  return deepFreeze(parseLocalBenchmarkDefinitionAuthoringResponse(value));
}

export function createLocalBenchmarkDefinitionAuthoringResource(
  input: ResourceInput
): LocalBenchmarkDefinitionAuthoringResource {
  const target = freezeTarget(input.target, false);
  if (input.state === "available") {
    assertRequestedScope(input.result, target);
    return Object.freeze({ state: "available", target, result: deepFreeze(input.result) });
  }
  return Object.freeze({
    state: input.state,
    target,
    ...(input.message === undefined ? {} : { message: input.message })
  });
}

function assertRequestedScope(
  result: LocalBenchmarkDefinitionAuthoringResult,
  target: LocalBenchmarkDefinitionAuthoringTarget,
  command?: LocalBenchmarkDefinitionBindingRequest
): void {
  if (
    result.project_id !== target.projectId
    || result.context_id !== target.contextId
    || result.commit_id !== target.commitId
    || (command !== undefined && (
      result.binding_id !== command.binding_id
      || result.branch_name !== command.branch_name
      || result.suite_id !== command.suite.id
      || !sameStrings(result.dataset_ids, command.suite.dataset_ids)
    ))
  ) {
    throw new TypeError("local benchmark definition authoring response does not match the requested scope");
  }
}

function freezeTarget(
  target: LocalBenchmarkDefinitionAuthoringTarget,
  validate: boolean
): LocalBenchmarkDefinitionAuthoringTarget {
  return Object.freeze({
    projectId: validate ? requireNonBlank(target.projectId, "projectId") : target.projectId,
    contextId: validate ? requireNonBlank(target.contextId, "contextId") : target.contextId,
    commitId: validate ? requireNonBlank(target.commitId, "commitId") : target.commitId
  });
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const body = (await response.json()) as Partial<LocalApiErrorBody>;
    if (typeof body.error === "string" && typeof body.message === "string") {
      return { error: body.error, message: body.message };
    }
  } catch {
  }
  return {
    error: "contextlab_benchmark_definition_authoring_proxy_error",
    message: `ContextLab benchmark definition authoring failed with status ${response.status}`
  };
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  if (Number.isFinite(seconds) && seconds >= 0) return seconds * 1_000;
  const retryAt = Date.parse(value);
  return Number.isNaN(retryAt) ? undefined : Math.max(0, retryAt - Date.now());
}

function requireNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new RangeError(`${field} must be non-empty`);
  }
  return value;
}

function sameStrings(left: ReadonlyArray<string>, right: ReadonlyArray<string>): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function deepFreeze<T>(value: T): DeepReadonly<T> {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value as DeepReadonly<T>;
}
