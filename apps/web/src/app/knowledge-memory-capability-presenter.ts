import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import type {
  FrozenKnowledgeMemoryCapabilityFixture,
  FrozenKnowledgeMemoryCapabilityProjection
} from "./knowledge-memory-capability-data";

const capability: BilingualCapabilityText = Object.freeze({
  en: "Knowledge and memory capabilities",
  zh: "知识与记忆能力"
});

export type KnowledgeMemoryCapabilityProjectionModel = Readonly<{
  status: CapabilityStateScreenModel;
  facts: ReadonlyArray<Readonly<{ label: string; value: string }>>;
}>;

export type KnowledgeMemoryCapabilityScreenModel = Readonly<{
  title: string;
  description: string;
  status: CapabilityStateScreenModel;
  projections: ReadonlyArray<KnowledgeMemoryCapabilityProjectionModel>;
}>;

export function presentKnowledgeMemoryCapabilityFixture(
  fixture: FrozenKnowledgeMemoryCapabilityFixture
): KnowledgeMemoryCapabilityScreenModel {
  const state = fixture.state;
  const status = presentCapabilityState(
    frozenStatusDto("knowledge-memory-capability", capability, state)
  );

  return Object.freeze({
    title: "Knowledge and memory capabilities / 知识与记忆能力",
    description: "Local redacted capability fixture / 本地脱敏能力固定数据。",
    status,
    projections:
      state === "available" || state === "unavailable"
        ? Object.freeze(fixture.projections.map((projection) => presentProjection(projection, state)))
        : Object.freeze([])
  });
}

function presentProjection(
  projection: FrozenKnowledgeMemoryCapabilityProjection,
  state: "available" | "unavailable"
): KnowledgeMemoryCapabilityProjectionModel {
  return Object.freeze({
    status: presentCapabilityState(frozenStatusDto(projection.id, projection.capability, state, projection.boundary)),
    facts: Object.freeze([
      Object.freeze({
        label: `${projection.capability.en} capability metadata / ${projection.capability.zh}能力元数据`,
        value: projection.schemaVersion
      }),
      Object.freeze({
        label: "Boundary / 边界",
        value: `${projection.boundary.en} / ${projection.boundary.zh}`
      })
    ])
  });
}

function frozenStatusDto(
  id: string,
  capabilityText: BilingualCapabilityText,
  state: FrozenCapabilityStateDto["state"],
  summary?: BilingualCapabilityText
): FrozenCapabilityStateDto {
  return Object.freeze({
    id,
    capability: Object.freeze({ ...capabilityText }),
    state,
    ...(summary ? { summary: Object.freeze({ ...summary }) } : {})
  });
}
