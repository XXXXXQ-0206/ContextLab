import {
  LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
  parseLocalPluginCapabilityAvailability as parseSdkPluginCapabilityAvailability,
  type LocalApiErrorBody,
  type LocalPluginCapabilityAvailabilityResourceV1
} from "@contextlab/local-sdk";
import type { BilingualCapabilityText, CapabilityStateKind } from "./capability-state-data";

export { LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1 } from "@contextlab/local-sdk";
export type {
  LocalPluginCapabilityAvailabilityEntryV1,
  LocalPluginCapabilityAvailabilityResourceV1
} from "@contextlab/local-sdk";

export type LocalPluginCapabilityAvailabilityTarget = Readonly<{
  context_id: string;
  capability: BilingualCapabilityText;
}>;

export type LocalPluginCapabilityAvailabilityResource =
  | Readonly<{ kind: "loading" | "error" | "empty" | "unavailable"; target: LocalPluginCapabilityAvailabilityTarget; message?: string }>
  | Readonly<{ kind: "ready"; target: LocalPluginCapabilityAvailabilityTarget; summary: LocalPluginCapabilityAvailabilityResourceV1 }>;

export class LocalPluginCapabilityAvailabilityProxyError extends Error {
  readonly status: number;
  readonly body: Readonly<LocalApiErrorBody>;
  readonly retryAfterMs?: number;

  constructor(status: number, body: LocalApiErrorBody, retryAfterMs?: number) {
    super(body.message);
    this.name = "LocalPluginCapabilityAvailabilityProxyError";
    this.status = status;
    this.body = Object.freeze({ ...body });
    this.retryAfterMs = retryAfterMs;
  }
}

export function createLocalPluginCapabilityAvailabilityResource(
  input:
    | Readonly<{ kind: "loading" | "error" | "empty" | "unavailable"; target: LocalPluginCapabilityAvailabilityTarget; message?: string }>
    | Readonly<{ kind: "ready"; target: LocalPluginCapabilityAvailabilityTarget; summary: LocalPluginCapabilityAvailabilityResourceV1 }>
): LocalPluginCapabilityAvailabilityResource {
  const target = freezeTarget(input.target);
  if (input.kind !== "ready") {
    return Object.freeze({ kind: input.kind, target, ...(input.message ? { message: input.message } : {}) });
  }
  const summary = parseLocalPluginCapabilityAvailabilityV1(input.summary);
  if (summary.context_id !== target.context_id) throw new TypeError("Plugin capability response is outside the requested Context");
  return Object.freeze({ kind: "ready", target, summary });
}

export async function loadLocalPluginCapabilityAvailability(
  target: LocalPluginCapabilityAvailabilityTarget,
  bearerToken: string
): Promise<LocalPluginCapabilityAvailabilityResourceV1> {
  const exactTarget = freezeTarget(target);
  const token = requireNonBlank(bearerToken, "bearerToken").trim();
  const response = await fetch(
    `/api/local/contexts/${encodeURIComponent(exactTarget.context_id)}/plugins/capabilities`,
    {
      credentials: "omit",
      cache: "no-store",
      headers: { accept: "application/json", authorization: `Bearer ${token}` }
    }
  );
  if (!response.ok) {
    throw new LocalPluginCapabilityAvailabilityProxyError(
      response.status,
      await parseProxyErrorBody(response),
      parseRetryAfterMs(response.headers.get("retry-after"))
    );
  }
  const summary = parseLocalPluginCapabilityAvailabilityV1(await response.json());
  if (summary.context_id !== exactTarget.context_id) throw new TypeError("Plugin capability response is outside the requested Context");
  return summary;
}

export function adaptLocalPluginCapabilityAvailabilityV1(resource: LocalPluginCapabilityAvailabilityResource): Readonly<{
  id: string;
  context_id: string;
  capability: BilingualCapabilityText;
  state: CapabilityStateKind;
  summary: LocalPluginCapabilityAvailabilityResourceV1 | null;
}> {
  const target = freezeTarget(resource.target);
  const state: CapabilityStateKind = resource.kind === "ready"
    ? resource.summary.entries.length === 0
      ? "empty"
      : resource.summary.entries.every((entry) => entry.availability === "unavailable")
        ? "unavailable"
        : "available"
    : resource.kind;
  return Object.freeze({
    id: "local-plugin-capability-availability",
    context_id: target.context_id,
    capability: target.capability,
    state,
    summary: resource.kind === "ready" ? resource.summary : null
  });
}

export function parseLocalPluginCapabilityAvailabilityV1(value: unknown): LocalPluginCapabilityAvailabilityResourceV1 {
  return parseSdkPluginCapabilityAvailability(value);
}

function freezeTarget(target: LocalPluginCapabilityAvailabilityTarget): LocalPluginCapabilityAvailabilityTarget {
  return Object.freeze({
    context_id: requireNonBlank(target.context_id, "target.context_id"),
    capability: Object.freeze({
      en: requireNonBlank(target.capability.en, "target.capability.en"),
      zh: requireNonBlank(target.capability.zh, "target.capability.zh")
    })
  });
}

async function parseProxyErrorBody(response: Response): Promise<LocalApiErrorBody> {
  try {
    const value = await response.json();
    if (typeof value === "object" && value !== null && !Array.isArray(value)
      && typeof value.error === "string" && typeof value.message === "string") {
      return { error: value.error, message: value.message };
    }
  } catch {
  }
  return {
    error: "contextlab_plugin_capability_proxy_error",
    message: "The local Plugin/MCP capability projection is unavailable. / 本地 Plugin/MCP 能力投影不可用。"
  };
}

function parseRetryAfterMs(value: string | null): number | undefined {
  if (value === null) return undefined;
  const seconds = Number(value);
  return Number.isFinite(seconds) && seconds >= 0 ? seconds * 1_000 : undefined;
}

function requireNonBlank(value: string, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) throw new TypeError(`${field} must be non-blank`);
  return value;
}
