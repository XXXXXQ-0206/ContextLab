import type { LocalApiErrorBody } from "@contextlab/local-sdk";
import { parseLocalBenchmarkDecisionList } from "@contextlab/local-sdk";

export const LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1 =
  "contextlab.local-benchmark-decision-discovery.v1";

export type LocalBenchmarkDecisionDiscoveryStatus = "passed" | "regressed" | "insufficient_data";

export type LocalBenchmarkDecisionSummary = Readonly<{
  decision_id: string;
  suite_id: string;
  status: LocalBenchmarkDecisionDiscoveryStatus;
  recorded_at: string;
}>;

export type LocalBenchmarkDecisionDiscovery = Readonly<{
  schema_version: typeof LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1;
  project_id: string;
  context_id: string;
  commit_id: string;
  decisions: ReadonlyArray<LocalBenchmarkDecisionSummary>;
}>;

export class LocalBenchmarkDecisionDiscoveryProxyError extends Error {
  readonly status: number;
  readonly body: LocalApiErrorBody;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalBenchmarkDecisionDiscoveryProxyError";
    this.status = status;
    this.body = body;
    this.retryAfterMs = retryAfterMs;
  }
}

export async function loadLocalBenchmarkDecisionDiscovery(
  projectId: string,
  contextId: string,
  commitId: string,
  bearerToken: string
): Promise<LocalBenchmarkDecisionDiscovery> {
  const response = await fetch(
    `/api/local/projects/${encodeURIComponent(projectId)}/contexts/${encodeURIComponent(contextId)}/commits/${encodeURIComponent(commitId)}/benchmark-decisions`,
    {
      headers: {
        accept: "application/json",
        authorization: `Bearer ${bearerToken.trim()}`
      },
      credentials: "omit",
      cache: "no-store"
    }
  );

  if (!response.ok) {
    throw new LocalBenchmarkDecisionDiscoveryProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }

  const discovery = parseLocalBenchmarkDecisionDiscovery(await response.json());
  if (
    discovery.project_id !== projectId
    || discovery.context_id !== contextId
    || discovery.commit_id !== commitId
  ) {
    throw new TypeError("benchmark decision discovery scope does not match the requested commit");
  }

  return discovery;
}

export function parseLocalBenchmarkDecisionDiscovery(
  value: unknown
): LocalBenchmarkDecisionDiscovery {
  if (isRecord(value) && value.schema_version === "contextlab.local-benchmark-decision-list.v1") {
    const list = parseLocalBenchmarkDecisionList(value);
    return Object.freeze({
      schema_version: LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1,
      project_id: list.project_id,
      context_id: list.context_id,
      commit_id: list.commit_id,
      decisions: Object.freeze(
        list.decisions.map((decision) =>
          Object.freeze({
            decision_id: decision.decision_id,
            suite_id: decision.suite.id,
            status: decision.status,
            recorded_at: decision.recorded_at
          })
        )
      )
    });
  }

  const record = asRecord(value, "benchmark decision discovery");
  assertExactKeys(record, ["schema_version", "project_id", "context_id", "commit_id", "decisions"]);

  if (record.schema_version !== LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1) {
    throw new TypeError(
      "schema_version must be contextlab.local-benchmark-decision-discovery.v1"
    );
  }

  const decisions = asArray(record.decisions, "decisions").map(parseLocalBenchmarkDecisionSummary);
  const identities = new Set<string>();
  for (const decision of decisions) {
    if (identities.has(decision.decision_id)) {
      throw new TypeError("decisions contains duplicate identities");
    }
    identities.add(decision.decision_id);
  }
  assertStableDecisionOrder(decisions);

  return Object.freeze({
    schema_version: LOCAL_BENCHMARK_DECISION_DISCOVERY_SCHEMA_V1,
    project_id: asNonBlankString(record.project_id, "project_id"),
    context_id: asNonBlankString(record.context_id, "context_id"),
    commit_id: asNonBlankString(record.commit_id, "commit_id"),
    decisions: Object.freeze(decisions)
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
    error: "contextlab_benchmark_decision_discovery_proxy_error",
    message: `ContextLab benchmark decision discovery request failed with status ${response.status}`
  };
}

function parseLocalBenchmarkDecisionSummary(value: unknown): LocalBenchmarkDecisionSummary {
  const record = asRecord(value, "benchmark decision summary");
  assertExactKeys(record, ["decision_id", "suite_id", "status", "recorded_at"]);

  if (
    record.status !== "passed"
    && record.status !== "regressed"
    && record.status !== "insufficient_data"
  ) {
    throw new TypeError("status must be passed, regressed, or insufficient_data");
  }

  const recordedAt = asUtcTimestamp(record.recorded_at, "recorded_at");

  return Object.freeze({
    decision_id: asNonBlankString(record.decision_id, "decision_id"),
    suite_id: asNonBlankString(record.suite_id, "suite_id"),
    status: record.status,
    recorded_at: recordedAt
  });
}

function assertStableDecisionOrder(decisions: ReadonlyArray<LocalBenchmarkDecisionSummary>): void {
  for (let index = 1; index < decisions.length; index += 1) {
    const previous = decisions[index - 1]!;
    const current = decisions[index]!;
    const previousTime = Date.parse(previous.recorded_at);
    const currentTime = Date.parse(current.recorded_at);
    if (
      previousTime < currentTime
      || (previousTime === currentTime
        && compareCodePoints(previous.decision_id, current.decision_id) >= 0)
    ) {
      throw new TypeError(
        "decisions must be ordered by recorded_at descending and decision_id ascending"
      );
    }
  }
}

function assertExactKeys(record: Record<string, unknown>, expected: string[]): void {
  const actual = Object.keys(record).sort(compareCodePoints);
  const sortedExpected = [...expected].sort(compareCodePoints);
  if (actual.length !== sortedExpected.length || actual.some((key, index) => key !== sortedExpected[index])) {
    throw new TypeError("benchmark decision discovery payload contains unsupported fields");
  }
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as Record<string, unknown>;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new TypeError(`${field} must be an array`);
  }
  return value;
}

function asNonBlankString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be a non-blank string`);
  }
  return value;
}

function asUtcTimestamp(value: unknown, field: string): string {
  const timestamp = asNonBlankString(value, field);
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?Z$/.exec(timestamp);
  if (!match) {
    throw new TypeError(`${field} must be a valid timestamp`);
  }

  const [, year, month, day, hour, minute, second] = match;
  const date = new Date(0);
  date.setUTCFullYear(Number(year), Number(month) - 1, Number(day));
  date.setUTCHours(Number(hour), Number(minute), Number(second), 0);
  if (
    date.getUTCFullYear() !== Number(year)
    || date.getUTCMonth() !== Number(month) - 1
    || date.getUTCDate() !== Number(day)
    || date.getUTCHours() !== Number(hour)
    || date.getUTCMinutes() !== Number(minute)
    || date.getUTCSeconds() !== Number(second)
  ) {
    throw new TypeError(`${field} must be a valid timestamp`);
  }

  return timestamp;
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) {
    return undefined;
  }

  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}

function compareCodePoints(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
