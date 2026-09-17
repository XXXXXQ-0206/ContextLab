import type { BilingualCapabilityText, CapabilityStateKind, FrozenCapabilityStateDto } from "./capability-state-data";

export const LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1 = "contextlab.local-workflow-context-bindings.v1";

export type LocalWorkflowContextBindingV1 = Readonly<{
  binding_id: string;
  workflow_id: string;
  workflow_revision: number;
  node_count: number;
  edge_count: number;
}>;

export type LocalWorkflowContextBindingsV1 = Readonly<{
  schema_version: typeof LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1;
  context_id: string;
  commit_id: string;
  bindings: ReadonlyArray<LocalWorkflowContextBindingV1>;
}>;

export type LocalWorkflowContextBindingsTarget = Readonly<{
  context_id: string;
  commit_id: string;
  capability: BilingualCapabilityText;
}>;

export type LocalWorkflowContextBindingsResource =
  | Readonly<{ kind: "loading"; target: LocalWorkflowContextBindingsTarget }>
  | Readonly<{ kind: "error"; target: LocalWorkflowContextBindingsTarget }>
  | Readonly<{ kind: "empty"; target: LocalWorkflowContextBindingsTarget }>
  | Readonly<{ kind: "unavailable"; target: LocalWorkflowContextBindingsTarget }>
  | Readonly<{
      kind: "ready";
      target: LocalWorkflowContextBindingsTarget;
      summary: LocalWorkflowContextBindingsV1;
    }>;

export type FrozenLocalWorkflowContextBindingsDto = Readonly<{
  id: "local-workflow-context-bindings";
  context_id: string;
  commit_id: string;
  capability: BilingualCapabilityText;
  state: CapabilityStateKind;
  bindings: ReadonlyArray<LocalWorkflowContextBindingV1>;
}>;

const emptyBindings = Object.freeze([]) as ReadonlyArray<LocalWorkflowContextBindingV1>;

export function parseLocalWorkflowContextBindingsV1(value: unknown): LocalWorkflowContextBindingsV1 {
  const record = asRecord(value, "local workflow Context bindings");
  assertExactKeys(record, ["schema_version", "context_id", "commit_id", "bindings"]);

  if (record.schema_version !== LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1) {
    throw new TypeError("schema_version must be contextlab.local-workflow-context-bindings.v1");
  }

  const bindings = asArray(record.bindings, "bindings").map(parseLocalWorkflowContextBindingV1);
  assertStableBindingOrder(bindings);

  return Object.freeze({
    schema_version: LOCAL_WORKFLOW_CONTEXT_BINDINGS_SCHEMA_V1,
    context_id: asNonBlankString(record.context_id, "context_id"),
    commit_id: asNonBlankString(record.commit_id, "commit_id"),
    bindings: Object.freeze(bindings)
  });
}

export function adaptLocalWorkflowContextBindingsV1(
  resource: LocalWorkflowContextBindingsResource
): FrozenLocalWorkflowContextBindingsDto {
  const target = freezeTarget(resource.target);

  switch (resource.kind) {
    case "loading":
      return freezeDto(target, "loading", emptyBindings);
    case "error":
      return freezeDto(target, "error", emptyBindings);
    case "empty":
      return freezeDto(target, "empty", emptyBindings);
    case "unavailable":
      return freezeDto(target, "unavailable", emptyBindings);
    case "ready": {
      const summary = parseLocalWorkflowContextBindingsV1(resource.summary);
      if (summary.context_id !== target.context_id || summary.commit_id !== target.commit_id) {
        throw new TypeError("workflow Context and commit scope does not match the requested target");
      }

      const bindings = freezeBindings(summary.bindings);
      return freezeDto(target, bindings.length === 0 ? "empty" : "available", bindings);
    }
  }
}

function parseLocalWorkflowContextBindingV1(value: unknown): LocalWorkflowContextBindingV1 {
  const record = asRecord(value, "workflow Context binding");
  assertExactKeys(record, [
    "binding_id",
    "workflow_id",
    "workflow_revision",
    "node_count",
    "edge_count"
  ]);

  return Object.freeze({
    binding_id: asNonBlankString(record.binding_id, "binding_id"),
    workflow_id: asNonBlankString(record.workflow_id, "workflow_id"),
    workflow_revision: asPositiveInteger(record.workflow_revision, "workflow_revision"),
    node_count: asNonNegativeInteger(record.node_count, "node_count"),
    edge_count: asNonNegativeInteger(record.edge_count, "edge_count")
  });
}

function freezeTarget(value: LocalWorkflowContextBindingsTarget): LocalWorkflowContextBindingsTarget {
  const record = asRecord(value, "workflow Context binding target");
  assertExactKeys(record, ["context_id", "commit_id", "capability"]);

  return Object.freeze({
    context_id: asNonBlankString(record.context_id, "target.context_id"),
    commit_id: asNonBlankString(record.commit_id, "target.commit_id"),
    capability: parseBilingualText(record.capability, "target.capability")
  });
}

function freezeDto(
  target: LocalWorkflowContextBindingsTarget,
  state: CapabilityStateKind,
  bindings: ReadonlyArray<LocalWorkflowContextBindingV1>
): FrozenLocalWorkflowContextBindingsDto {
  return Object.freeze({
    id: "local-workflow-context-bindings" as const,
    context_id: target.context_id,
    commit_id: target.commit_id,
    capability: target.capability,
    state,
    bindings: freezeBindings(bindings)
  });
}

function freezeBindings(
  bindings: ReadonlyArray<LocalWorkflowContextBindingV1>
): ReadonlyArray<LocalWorkflowContextBindingV1> {
  return Object.freeze(
    bindings.map((binding) =>
      Object.freeze({
        binding_id: binding.binding_id,
        workflow_id: binding.workflow_id,
        workflow_revision: binding.workflow_revision,
        node_count: binding.node_count,
        edge_count: binding.edge_count
      })
    )
  );
}

function assertStableBindingOrder(bindings: ReadonlyArray<LocalWorkflowContextBindingV1>): void {
  const bindingIds = new Set<string>();

  for (let index = 0; index < bindings.length; index += 1) {
    const binding = bindings[index]!;
    if (bindingIds.has(binding.binding_id)) {
      throw new TypeError("bindings contains duplicate identities");
    }
    bindingIds.add(binding.binding_id);

    const previous = bindings[index - 1];
    if (previous === undefined) {
      continue;
    }

    const workflowComparison = compareCodePoints(previous.workflow_id, binding.workflow_id);
    if (
      workflowComparison > 0
      || (workflowComparison === 0 && previous.workflow_revision > binding.workflow_revision)
      || (
        workflowComparison === 0
        && previous.workflow_revision === binding.workflow_revision
        && compareCodePoints(previous.binding_id, binding.binding_id) >= 0
      )
    ) {
      throw new TypeError("bindings must be ordered by workflow_id, workflow_revision, and binding_id");
    }

    if (
      workflowComparison === 0
      && previous.workflow_revision === binding.workflow_revision
    ) {
      throw new TypeError("bindings contains a duplicate workflow revision");
    }
  }
}

function parseBilingualText(value: unknown, field: string): BilingualCapabilityText {
  const record = asRecord(value, field);
  assertExactKeys(record, ["en", "zh"]);
  return Object.freeze({
    en: asNonBlankString(record.en, `${field}.en`),
    zh: asNonBlankString(record.zh, `${field}.zh`)
  });
}

function asRecord(value: unknown, field: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
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

function asNonBlankString(value: unknown, field: string): string {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new TypeError(`${field} must be a non-blank string`);
  }

  return value;
}

function asPositiveInteger(value: unknown, field: string): number {
  const number = asNonNegativeInteger(value, field);
  if (number === 0) {
    throw new TypeError(`${field} must be a positive integer`);
  }

  return number;
}

function asNonNegativeInteger(value: unknown, field: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new TypeError(`${field} must be a non-negative integer`);
  }

  return value;
}

function assertExactKeys(record: Record<string, unknown>, expected: string[]): void {
  const actual = Object.keys(record).sort(compareCodePoints);
  const sortedExpected = [...expected].sort(compareCodePoints);
  if (
    actual.length !== sortedExpected.length
    || actual.some((key, index) => key !== sortedExpected[index])
  ) {
    throw new TypeError("workflow Context bindings contain an unexpected shape");
  }
}

function compareCodePoints(left: string, right: string): number {
  if (left < right) {
    return -1;
  }
  if (left > right) {
    return 1;
  }
  return 0;
}
