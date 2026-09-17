import type { BilingualCapabilityText, FrozenCapabilityStateDto } from "./capability-state-data";
import { presentCapabilityState, type CapabilityStateScreenModel } from "./capability-state-presenter";
import {
  adaptLocalKnowledgeMemoryProjectionV1,
  type LocalKnowledgeMemoryProjectionResource
} from "./local-knowledge-memory-projection-data";
import type { LocalKnowledgeCitationV1 } from "@contextlab/local-sdk";

export type LocalKnowledgeMemoryCitationRowModel = Readonly<{
  id: string;
  source: string;
  revision: string;
  chunk: string;
  range: string;
  fingerprint: string;
}>;

export type LocalKnowledgeMemoryProjectionViewModel = Readonly<{
  title: string;
  description: string;
  status: CapabilityStateScreenModel;
  scope: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  projectionFacts: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
  citations: ReadonlyArray<LocalKnowledgeMemoryCitationRowModel>;
  memoryFacts: ReadonlyArray<Readonly<{ id: string; label: string; value: string }>>;
}>;

export function presentLocalKnowledgeMemoryProjection(
  resource: LocalKnowledgeMemoryProjectionResource
): LocalKnowledgeMemoryProjectionViewModel {
  const dto = adaptLocalKnowledgeMemoryProjectionV1(resource);
  const status = presentCapabilityState(Object.freeze({
    id: dto.id,
    capability: dto.capability,
    state: dto.state,
    summary: stateSummary(dto.state),
    detail: dto.projection ? bilingual(
      "Server-owned redacted citation and memory replay facts.",
      "服务端控制的脱敏引用与记忆回放事实。"
    ) : undefined
  }) satisfies FrozenCapabilityStateDto);
  const projection = dto.projection;

  return deepFreeze({
    title: "Knowledge and memory projection / Knowledge 与 Memory 投影",
    description: "Private, read-only exact Context commit scope. / 私有、只读的精确 Context 提交范围。",
    status,
    scope: [
      { id: "context", label: "Context / 上下文", value: dto.context_id },
      { id: "commit", label: "Commit / 提交", value: dto.commit_id }
    ],
    projectionFacts: projection ? [
      { id: "projection", label: "Replay projection / 回放投影", value: projection.id },
      { id: "schema", label: "Schema / 模式", value: projection.schema_version },
      { id: "citation-projection", label: "Citation projection / 引用投影", value: projection.citation_projection_id },
      { id: "retrieval", label: "Retrieval / 检索", value: projection.retrieval_id }
    ] : [],
    citations: projection?.citations.map(presentCitation) ?? [],
    memoryFacts: projection ? [
      { id: "memory", label: "Memory / 记忆", value: projection.memory_id },
      { id: "timeline", label: "Timeline version / 时间线版本", value: String(projection.memory_timeline_version) },
      { id: "retention", label: "Retention decision / 留存决策", value: `${projection.retention_decision} / ${retentionLabel(projection.retention_decision)}` },
      { id: "replay-state", label: "Replay state / 回放状态", value: `${projection.replay_state} / ${projection.replay_state === "active" ? "活动" : "已遗忘"}` },
      { id: "policy", label: "Policy version / 策略版本", value: projection.retention_policy_version }
    ] : []
  });
}

function presentCitation(citation: LocalKnowledgeCitationV1): LocalKnowledgeMemoryCitationRowModel {
  return {
    id: citation.chunk_id,
    source: citation.document_id,
    revision: `${citation.document_revision_id} / ${citation.source_version}`,
    chunk: `${citation.chunk_id} / ${citation.chunking_version} #${citation.ordinal}`,
    range: `${citation.range.start_byte}..${citation.range.end_byte}`,
    fingerprint: citation.content_fingerprint
  };
}

function stateSummary(state: "loading" | "error" | "empty" | "unavailable" | "available"): BilingualCapabilityText {
  switch (state) {
    case "loading": return bilingual("Reading the exact projection.", "正在读取精确投影。");
    case "error": return bilingual("The protected projection read failed.", "受保护投影读取失败。");
    case "empty": return bilingual("No projection is available for this commit.", "此提交暂无投影。");
    case "unavailable": return bilingual("The local Knowledge/Memory adapter is unavailable.", "本地 Knowledge/Memory 适配器不可用。");
    case "available": return bilingual("The exact redacted projection is available.", "精确脱敏投影已可用。");
  }
}

function retentionLabel(value: string): string {
  return value === "retained_pinned" ? "固定留存" : value === "retained_fresh" ? "新鲜留存" : value === "retained_important" ? "重要留存" : value === "expired_forgotten" ? "过期遗忘" : "低重要性过期";
}

function bilingual(en: string, zh: string): BilingualCapabilityText {
  return Object.freeze({ en, zh });
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object") {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
