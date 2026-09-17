import React from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import type { FrozenWorkflowCapabilityResourceFixture } from "./workflow-capability-data";
import { presentWorkflowCapabilityResourceFixture } from "./workflow-capability-presenter";

export type WorkflowCapabilityInspectorProps = {
  fixture: FrozenWorkflowCapabilityResourceFixture;
};

export function WorkflowCapabilityInspector({ fixture }: WorkflowCapabilityInspectorProps) {
  const view = presentWorkflowCapabilityResourceFixture(fixture);

  return (
    <section aria-labelledby="workflow-capability-heading">
      <h2 id="workflow-capability-heading">{view.title}</h2>
      <p>{view.description}</p>
      <CapabilityStateScreen view={view.status} />
      {view.resources.map((resource) => (
        <section aria-label={resource.status.capabilityLabel} key={resource.status.id}>
          <CapabilityStateScreen view={resource.status} />
          <dl>
            {resource.facts.map((fact) => (
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
