import { CapabilityState, CodeChip, DefinitionGrid, StackTable, StatusPill } from "@contextlab/ui";
import React from "react";
import type { LocalKnowledgeMemoryProjectionViewModel } from "./local-knowledge-memory-projection-presenter";

export type LocalKnowledgeMemoryProjectionScreenProps = Readonly<{
  view: LocalKnowledgeMemoryProjectionViewModel;
  controls?: React.ReactNode;
}>;

export function LocalKnowledgeMemoryProjectionScreen({ controls, view }: LocalKnowledgeMemoryProjectionScreenProps) {
  return (
    <section
      aria-labelledby="local-knowledge-memory-projection-heading"
      aria-busy={view.status.state === "loading"}
      className="operation-block local-knowledge-memory-projection"
      data-state={view.status.state}
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Context inputs / Context 输入</span>
          <h3 id="local-knowledge-memory-projection-heading">{view.title}</h3>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="info">read only / 只读</StatusPill>
      </div>
      {controls}
      <CapabilityState
        ariaLabel={view.status.ariaLabel}
        description={view.status.description}
        detail={view.status.detail}
        label={view.status.capabilityLabel}
        state={view.status.state}
        stateLabel={view.status.stateLabel}
      />
      <DefinitionGrid
        aria-label="Exact Knowledge/Memory scope / 精确 Knowledge/Memory 范围"
        columns={2}
        compact
        items={view.scope.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
        surface="raised"
        valueTone="info"
      />
      {view.projectionFacts.length > 0 ? (
        <DefinitionGrid
          aria-label="Redacted projection facts / 脱敏投影事实"
          columns={3}
          compact
          items={view.projectionFacts.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
          surface="raised"
        />
      ) : null}
      {view.citations.length > 0 ? (
        <StackTable
          aria-label="Redacted Knowledge citations / 脱敏 Knowledge 引用"
          columnTemplate="minmax(8rem, 1fr) minmax(8rem, 1fr) minmax(9rem, 1fr) minmax(7rem, 0.8fr) minmax(10rem, 1.2fr)"
          headers={["Source / 来源", "Revision / 修订", "Chunk / 分块", "Range / 范围", "Fingerprint / 指纹"]}
          rows={view.citations.map((citation) => ({
            id: citation.id,
            cells: [
              <CodeChip key={`${citation.id}-source`}>{citation.source}</CodeChip>,
              <CodeChip key={`${citation.id}-revision`}>{citation.revision}</CodeChip>,
              <CodeChip key={`${citation.id}-chunk`}>{citation.chunk}</CodeChip>,
              citation.range,
              <CodeChip key={`${citation.id}-fingerprint`}>{citation.fingerprint}</CodeChip>
            ]
          }))}
        />
      ) : null}
      {view.memoryFacts.length > 0 ? (
        <DefinitionGrid
          aria-label="Redacted Memory replay and retention facts / 脱敏 Memory 回放与留存事实"
          columns={3}
          compact
          items={view.memoryFacts.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
          surface="raised"
        />
      ) : null}
    </section>
  );
}
