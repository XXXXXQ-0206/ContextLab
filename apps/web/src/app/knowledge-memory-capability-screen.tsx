import React from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import type { FrozenKnowledgeMemoryCapabilityFixture } from "./knowledge-memory-capability-data";
import { presentKnowledgeMemoryCapabilityFixture } from "./knowledge-memory-capability-presenter";

export type KnowledgeMemoryCapabilityInspectorProps = {
  fixture: FrozenKnowledgeMemoryCapabilityFixture;
};

export function KnowledgeMemoryCapabilityInspector({ fixture }: KnowledgeMemoryCapabilityInspectorProps) {
  const view = presentKnowledgeMemoryCapabilityFixture(fixture);

  return (
    <section aria-labelledby="knowledge-memory-capability-heading">
      <h2 id="knowledge-memory-capability-heading">{view.title}</h2>
      <p>{view.description}</p>
      <CapabilityStateScreen view={view.status} />
      {view.projections.map((projection) => (
        <section key={projection.status.id}>
          <CapabilityStateScreen view={projection.status} />
          <dl>
            {projection.facts.map((fact) => (
              <React.Fragment key={fact.label}>
                <dt>{fact.label}</dt>
                <dd>{fact.value}</dd>
              </React.Fragment>
            ))}
          </dl>
        </section>
      ))}
    </section>
  );
}
