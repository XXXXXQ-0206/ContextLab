import type { BilingualCapabilityText, CapabilityStateKind } from "./capability-state-data";

export type FrozenKnowledgeMemoryCapabilityProjection = Readonly<{
  id: "knowledge-citation" | "memory-retention";
  schemaVersion: string;
  capability: BilingualCapabilityText;
  boundary: BilingualCapabilityText;
}>;

export type FrozenKnowledgeMemoryCapabilityFixture = Readonly<{
  state: CapabilityStateKind;
  projections: ReadonlyArray<FrozenKnowledgeMemoryCapabilityProjection>;
}>;

const knowledgeCitation = Object.freeze({
  id: "knowledge-citation" as const,
  schemaVersion: "knowledge-citation-capability-v1",
  capability: Object.freeze({ en: "Knowledge citation", zh: "知识引用" }),
  boundary: Object.freeze({
    en: "Citation metadata contains no raw knowledge chunks.",
    zh: "引用元数据不包含原始知识分块。"
  })
});

const memoryRetention = Object.freeze({
  id: "memory-retention" as const,
  schemaVersion: "memory-retention-capability-v1",
  capability: Object.freeze({ en: "Memory retention", zh: "记忆留存" }),
  boundary: Object.freeze({
    en: "Retention metadata contains no raw memory content.",
    zh: "留存元数据不包含原始记忆内容。"
  })
});

const capabilityProjections = Object.freeze([knowledgeCitation, memoryRetention]);
const emptyProjections = Object.freeze([]) as ReadonlyArray<FrozenKnowledgeMemoryCapabilityProjection>;

export const KNOWLEDGE_MEMORY_CAPABILITY_FIXTURES = Object.freeze({
  loading: Object.freeze({ state: "loading" as const, projections: emptyProjections }),
  error: Object.freeze({ state: "error" as const, projections: emptyProjections }),
  empty: Object.freeze({ state: "empty" as const, projections: emptyProjections }),
  available: Object.freeze({ state: "available" as const, projections: capabilityProjections }),
  unavailable: Object.freeze({ state: "unavailable" as const, projections: capabilityProjections })
}) as Readonly<Record<CapabilityStateKind, FrozenKnowledgeMemoryCapabilityFixture>>;
