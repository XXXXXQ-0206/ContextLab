import type { LocalLifecycleReadCredentials } from "./types";
import type { ContextLabLocalClientOptions, FetchLike } from "./client";
import { ContextLabLocalApiError } from "./client";

export const LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1 =
  "contextlab.local-plugin-capability-availability.v1" as const;

export type LocalPluginCapabilityAvailabilityEntryV1 = Readonly<{
  plugin_id: string;
  capability_id: string;
  capability_version: string;
  availability: "available" | "unavailable";
  compatibility: "compatible" | "incompatible";
  diagnostic_code: string | null;
}>;

export type LocalPluginCapabilityAvailabilityResourceV1 = Readonly<{
  schema_version: typeof LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1;
  context_id: string;
  entries: ReadonlyArray<LocalPluginCapabilityAvailabilityEntryV1>;
}>;

export class ContextLabLocalPluginCapabilityAvailabilityError extends ContextLabLocalApiError {
  constructor(status: number, body: Readonly<{ error: string; message: string }>, retryAfterMs?: number) {
    super(status, body, retryAfterMs);
    this.name = "ContextLabLocalPluginCapabilityAvailabilityError";
  }
}

export class ContextLabLocalPluginCapabilityAvailabilityClient {
  private readonly baseUrl: string;
  private readonly fetchImpl: FetchLike;

  constructor(options: ContextLabLocalClientOptions = {}) {
    this.baseUrl = (options.baseUrl ?? "http://127.0.0.1:3100").replace(/\/+$/, "");
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
  }

  async getPluginCapabilityAvailability(
    contextId: string,
    credentials: LocalLifecycleReadCredentials
  ): Promise<LocalPluginCapabilityAvailabilityResourceV1> {
    const requestedContextId = asUuid(contextId, "contextId");
    const bearerToken = asNonBlank(credentials.bearerToken, "bearerToken").trim();
    const response = await this.fetchImpl(
      `${this.baseUrl}/api/v1/local/contexts/${encodeURIComponent(requestedContextId)}/plugins/capabilities`,
      {
        credentials: "omit",
        cache: "no-store",
        headers: {
          accept: "application/json",
          authorization: `Bearer ${bearerToken}`
        }
      }
    );

    if (!response.ok) {
      throw new ContextLabLocalPluginCapabilityAvailabilityError(
        response.status,
        await parseErrorBody(response),
        parseRetryAfterMs(response.headers.get("retry-after"))
      );
    }

    const resource = parseLocalPluginCapabilityAvailability(await response.json());
    if (resource.context_id !== requestedContextId) {
      throw new TypeError("Plugin capability availability does not match the requested Context scope");
    }
    return resource;
  }
}

export function parseLocalPluginCapabilityAvailability(
  value: unknown
): LocalPluginCapabilityAvailabilityResourceV1 {
  const record = asRecord(value, "Plugin capability availability");
  assertExactKeys(record, ["schema_version", "context_id", "entries"]);
  if (record.schema_version !== LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1) {
    throw new TypeError(`schema_version must be ${LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1}`);
  }
  const entries = asArray(record.entries, "entries").map(parseEntry);
  const keys = entries.map((entry) => `${entry.plugin_id}\u0000${entry.capability_id}\u0000${entry.capability_version}`);
  if (keys.some((key, index) => index > 0 && key <= keys[index - 1]!)) {
    throw new TypeError("entries must be deterministically ordered");
  }
  return freeze({
    schema_version: LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
    context_id: asUuid(record.context_id, "context_id"),
    entries: Object.freeze(entries)
  });
}

function parseEntry(value: unknown): LocalPluginCapabilityAvailabilityEntryV1 {
  const record = asRecord(value, "capability entry");
  assertExactKeys(record, [
    "plugin_id",
    "capability_id",
    "capability_version",
    "availability",
    "compatibility",
    "diagnostic_code"
  ]);
  const availability = record.availability;
  if (availability !== "available" && availability !== "unavailable") {
    throw new TypeError("availability is unsupported");
  }
  const compatibility = record.compatibility;
  if (compatibility !== "compatible" && compatibility !== "incompatible") {
    throw new TypeError("compatibility is unsupported");
  }
  const diagnosticCode = record.diagnostic_code;
  if (diagnosticCode !== null && (typeof diagnosticCode !== "string" || !/^[a-z][a-z0-9_]*$/.test(diagnosticCode))) {
    throw new TypeError("diagnostic_code is unsupported");
  }
  if (availability === "available" && (compatibility !== "compatible" || diagnosticCode !== null)) {
    throw new TypeError("available capability entry has inconsistent outcome");
  }
  if (availability === "unavailable" && diagnosticCode === null) {
    throw new TypeError("unavailable capability entry requires a diagnostic code");
  }
  return freeze({
    plugin_id: asStableIdentifier(record.plugin_id, "plugin_id"),
    capability_id: asStableIdentifier(record.capability_id, "capability_id"),
    capability_version: asVersion(record.capability_version, "capability_version"),
    availability,
    compatibility,
    diagnostic_code: diagnosticCode
  });
}

function parseErrorBody(response: Response): Promise<{ error: string; message: string }> {
  return response.json().then((value: unknown) => {
    const record = asRecord(value, "API error");
    return {
      error: asNonBlank(record.error, "error"),
      message: asNonBlank(record.message, "message")
    };
  });
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (!value || !/^\d+$/.test(value)) return undefined;
  return Number(value) * 1_000;
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new TypeError(`${field} must be an object`);
  }
  return value as Record<string, unknown>;
}

function asArray(value: unknown, field: string): unknown[] {
  if (!Array.isArray(value)) throw new TypeError(`${field} must be an array`);
  return value;
}

function assertExactKeys(record: Record<string, unknown>, keys: readonly string[]): void {
  const actual = Object.keys(record).sort();
  const expected = [...keys].sort();
  if (actual.length !== expected.length || actual.some((key, index) => key !== expected[index])) {
    throw new TypeError("response contains an unexpected shape");
  }
}

function asNonBlank(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) throw new TypeError(`${field} must be non-blank`);
  return value;
}

function asUuid(value: unknown, field: string): string {
  const text = asNonBlank(value, field);
  if (!/^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(text)) {
    throw new TypeError(`${field} must be a UUID`);
  }
  return text.toLowerCase();
}

function asStableIdentifier(value: unknown, field: string): string {
  const text = asNonBlank(value, field);
  if (!/^[A-Za-z0-9][A-Za-z0-9._:-]*$/.test(text)) throw new TypeError(`${field} is not stable`);
  return text;
}

function asVersion(value: unknown, field: string): string {
  const text = asNonBlank(value, field);
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(text)) throw new TypeError(`${field} must be semver`);
  return text;
}

function freeze<T>(value: T): Readonly<T> {
  return Object.freeze(value);
}
