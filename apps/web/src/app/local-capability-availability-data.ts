import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";

export const LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1 = "contextlab.local-capability-availability.v1";

const integrationByOperation = {
  "workspace-inspect": "contextlab-context-core",
  "context-inspect": "contextlab-context-core",
  "evaluation-run": "contextlab-evaluation",
  "diff-compare": "contextlab-diff-engine",
  "workflow-inspect": "contextlab-workflow"
} as const;

export type LocalCapabilityOperationId = keyof typeof integrationByOperation;
export type LocalCapabilityIntegration = (typeof integrationByOperation)[LocalCapabilityOperationId];

export type LocalCapabilityAvailabilityDto = Readonly<{
  schema_version: typeof LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1;
  operation_id: LocalCapabilityOperationId;
  integration: LocalCapabilityIntegration;
  availability: "unavailable";
  reason: "shared_integration_not_registered";
}>;

export type LocalCapabilityAvailabilityTarget = Readonly<{
  capability_id: string;
  capability: BilingualCapabilityText;
}>;

export type LocalCapabilityAvailabilityV1 = Readonly<{
  schema_version: 1;
  capability_id: string;
  capability: BilingualCapabilityText;
  availability: "available" | "unavailable";
  summary?: BilingualCapabilityText;
  detail?: BilingualCapabilityText;
}>;

export type LocalCapabilityAvailabilityResource =
  | Readonly<{ kind: "loading"; target: LocalCapabilityAvailabilityTarget }>
  | Readonly<{ kind: "error"; target: LocalCapabilityAvailabilityTarget }>
  | Readonly<{ kind: "empty"; target: LocalCapabilityAvailabilityTarget }>
  | Readonly<{ kind: "ready"; availability: LocalCapabilityAvailabilityV1 }>;

export function parseLocalCapabilityAvailability(value: unknown): LocalCapabilityAvailabilityDto {
  if (!isRecord(value)) {
    throw new TypeError("local capability availability must be an object");
  }
  assertAllowedKeys(
    value,
    ["schema_version", "operation_id", "integration", "availability", "reason"],
    "local capability availability"
  );
  if (value.schema_version !== LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1) {
    throw new TypeError("local capability availability schema_version is unsupported");
  }
  if (!isOperationId(value.operation_id)) {
    throw new TypeError("local capability availability operation_id is unsupported");
  }
  if (value.integration !== integrationByOperation[value.operation_id]) {
    throw new TypeError("local capability availability integration does not match operation_id");
  }
  if (value.availability !== "unavailable") {
    throw new TypeError("local capability availability state is unsupported");
  }
  if (value.reason !== "shared_integration_not_registered") {
    throw new TypeError("local capability availability reason is unsupported");
  }

  return Object.freeze({
    schema_version: LOCAL_CAPABILITY_AVAILABILITY_SCHEMA_V1,
    operation_id: value.operation_id,
    integration: integrationByOperation[value.operation_id],
    availability: "unavailable",
    reason: "shared_integration_not_registered"
  });
}

export function parseLocalCapabilityAvailabilityV1(value: unknown): LocalCapabilityAvailabilityV1 {
  if (!isRecord(value)) {
    throw new TypeError("local capability availability V1 must be an object");
  }
  assertAllowedKeys(
    value,
    ["schema_version", "capability_id", "capability", "availability", "summary", "detail"],
    "local capability availability V1"
  );
  if (value.schema_version !== 1) {
    throw new TypeError("local capability availability V1 schema_version is unsupported");
  }
  if (!isStableCapabilityId(value.capability_id)) {
    throw new TypeError("local capability availability V1 capability_id is unsupported");
  }
  if (value.availability !== "available" && value.availability !== "unavailable") {
    throw new TypeError("local capability availability V1 state is unsupported");
  }

  const capability = parseBilingualText(value.capability, "capability");
  const summary = parseOptionalBilingualText(value, "summary");
  const detail = parseOptionalBilingualText(value, "detail");

  return Object.freeze({
    schema_version: 1,
    capability_id: value.capability_id,
    capability,
    availability: value.availability,
    ...(summary ? { summary } : {}),
    ...(detail ? { detail } : {})
  });
}

export function toFrozenCapabilityStateDto(dto: LocalCapabilityAvailabilityDto): FrozenCapabilityStateDto {
  const capability = integrationCopy[dto.integration];

  return Object.freeze({
    id: `local-capability-${dto.operation_id}`,
    capability: Object.freeze(capability),
    state: "unavailable",
    summary: Object.freeze({
      en: `${capability.en} is not registered in this local adapter.`,
      zh: `${capability.zh}尚未注册到此本地适配器。`
    }),
    detail: Object.freeze({
      en: "Shared core registration is required before this command can run.",
      zh: "此命令需要先注册共享核心能力。"
    })
  });
}

export function adaptLocalCapabilityAvailabilityV1(
  resource: LocalCapabilityAvailabilityResource
): FrozenCapabilityStateDto {
  switch (resource.kind) {
    case "loading":
      return frozenCapabilityState(resource.target, "loading");
    case "error":
      return frozenCapabilityState(resource.target, "error");
    case "empty":
      return frozenCapabilityState(resource.target, "empty");
    case "ready":
      return frozenCapabilityState(
        resource.availability,
        resource.availability.availability,
        resource.availability.summary,
        resource.availability.detail
      );
  }
}

const integrationCopy: Record<LocalCapabilityIntegration, BilingualCapabilityText> = {
  "contextlab-context-core": { en: "Context core", zh: "上下文核心" },
  "contextlab-evaluation": { en: "Evaluation engine", zh: "评测引擎" },
  "contextlab-diff-engine": { en: "Diff engine", zh: "差异引擎" },
  "contextlab-workflow": { en: "Workflow engine", zh: "工作流引擎" }
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function frozenCapabilityState(
  target: LocalCapabilityAvailabilityTarget,
  state: FrozenCapabilityStateDto["state"],
  summary?: BilingualCapabilityText,
  detail?: BilingualCapabilityText
): FrozenCapabilityStateDto {
  return Object.freeze({
    id: target.capability_id,
    capability: Object.freeze({ ...target.capability }),
    state,
    ...(summary ? { summary: Object.freeze({ ...summary }) } : {}),
    ...(detail ? { detail: Object.freeze({ ...detail }) } : {})
  });
}

function isStableCapabilityId(value: unknown): value is string {
  return typeof value === "string" && /^[A-Za-z0-9][A-Za-z0-9._:-]*$/.test(value);
}

function parseOptionalBilingualText(
  record: Record<string, unknown>,
  field: "summary" | "detail"
): BilingualCapabilityText | undefined {
  if (!Object.hasOwn(record, field)) {
    return undefined;
  }

  return parseBilingualText(record[field], field);
}

function parseBilingualText(value: unknown, field: string): BilingualCapabilityText {
  if (!isRecord(value) || !isNonBlankText(value.en) || !isNonBlankText(value.zh)) {
    throw new TypeError(`local capability availability V1 ${field} must contain bilingual text`);
  }
  assertAllowedKeys(value, ["en", "zh"], `local capability availability V1 ${field}`);

  return Object.freeze({ en: value.en, zh: value.zh });
}

function assertAllowedKeys(
  record: Record<string, unknown>,
  allowed: readonly string[],
  field: string
): void {
  if (Object.keys(record).some((key) => !allowed.includes(key))) {
    throw new TypeError(`${field} contains an unexpected shape`);
  }
}

function isNonBlankText(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0;
}

function isOperationId(value: unknown): value is LocalCapabilityOperationId {
  return typeof value === "string" && Object.hasOwn(integrationByOperation, value);
}
