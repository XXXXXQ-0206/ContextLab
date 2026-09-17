import {
  parseLocalBenchmarkDefinitionBindingList,
  type LocalApiErrorBody,
  type LocalBenchmarkDefinitionBindingList,
  type LocalBenchmarkDefinitionBindingSummary
} from "@contextlab/local-sdk";

type DeepReadonly<T> = T extends (...args: never[]) => unknown
  ? T
  : T extends ReadonlyArray<infer TItem>
    ? ReadonlyArray<DeepReadonly<TItem>>
    : T extends object
      ? { readonly [TKey in keyof T]: DeepReadonly<T[TKey]> }
      : T;

export type LocalBenchmarkDefinitionBindingInspectionTarget = Readonly<{
  projectId: string;
  contextId: string;
  commitId: string;
}>;

export type LocalBenchmarkDefinitionBindingInspectionResult = DeepReadonly<LocalBenchmarkDefinitionBindingList>;
export type LocalBenchmarkDefinitionBindingInspectionSummary = DeepReadonly<LocalBenchmarkDefinitionBindingSummary>;
export type LocalBenchmarkDefinitionBindingInspectionState =
  | "loading"
  | "error"
  | "empty"
  | "available"
  | "unavailable";

export type LocalBenchmarkDefinitionBindingInspectionResource =
  | Readonly<{
      state: Exclude<LocalBenchmarkDefinitionBindingInspectionState, "available">;
      target: LocalBenchmarkDefinitionBindingInspectionTarget;
      message?: string;
    }>
  | Readonly<{
      state: "available";
      target: LocalBenchmarkDefinitionBindingInspectionTarget;
      result: LocalBenchmarkDefinitionBindingInspectionResult;
    }>;

export class LocalBenchmarkDefinitionBindingInspectionProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<LocalApiErrorBody>;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalBenchmarkDefinitionBindingInspectionProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
    this.retryAfterMs = retryAfterMs;
  }
}

export async function loadLocalBenchmarkDefinitionBindings(
  target: LocalBenchmarkDefinitionBindingInspectionTarget,
  bearerToken: string
): Promise<LocalBenchmarkDefinitionBindingInspectionResult> {
  const frozenTarget = freezeTarget(target, true);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(frozenTarget.projectId)}`
      + `/contexts/${encodeURIComponent(frozenTarget.contextId)}`
      + `/commits/${encodeURIComponent(frozenTarget.commitId)}/benchmark-definition-bindings`,
    {
      method: "GET",
      headers: {
        accept: "application/json",
        authorization: `Bearer ${token}`
      },
      credentials: "omit",
      cache: "no-store"
    }
  );
  if (!response.ok) {
    throw new LocalBenchmarkDefinitionBindingInspectionProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }
  const result = deepFreeze(parseLocalBenchmarkDefinitionBindingList(await response.json()));
  if (
    result.project_id !== frozenTarget.projectId
    || result.context_id !== frozenTarget.contextId
    || result.commit_id !== frozenTarget.commitId
  ) {
    throw new TypeError("benchmark definition binding list does not match the exact target");
  }
  return result;
}

export function createLocalBenchmarkDefinitionBindingInspectionResource(
  input: LocalBenchmarkDefinitionBindingInspectionResource
): LocalBenchmarkDefinitionBindingInspectionResource {
  const target = freezeTarget(input.target, false);
  if (input.state === "available") {
    if (
      input.result.project_id !== target.projectId
      || input.result.context_id !== target.contextId
      || input.result.commit_id !== target.commitId
    ) {
      throw new TypeError("available binding list does not match the exact target");
    }
    return Object.freeze({ state: "available", target, result: deepFreeze(input.result) });
  }
  return Object.freeze({
    state: input.state,
    target,
    ...(input.message === undefined ? {} : { message: input.message })
  });
}

function freezeTarget(
  target: LocalBenchmarkDefinitionBindingInspectionTarget,
  validate: boolean
): LocalBenchmarkDefinitionBindingInspectionTarget {
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
    error: "contextlab_benchmark_definition_binding_inspection_proxy_error",
    message: `ContextLab benchmark definition binding inspection failed with status ${response.status}`
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

function deepFreeze<T>(value: T): DeepReadonly<T> {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value as DeepReadonly<T>;
}
