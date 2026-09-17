import { CapabilityState, CodeChip, DefinitionGrid, StatusPill } from "@contextlab/ui";
import React from "react";
import type { LocalPluginCapabilityAvailabilityViewModel } from "./local-plugin-capability-availability-presenter";

export type LocalPluginCapabilityAvailabilityScreenProps = Readonly<{
  view: LocalPluginCapabilityAvailabilityViewModel;
  controls?: React.ReactNode;
}>;

export function LocalPluginCapabilityAvailabilityScreen({
  controls,
  view
}: LocalPluginCapabilityAvailabilityScreenProps) {
  return (
    <section
      aria-labelledby="local-plugin-capability-availability-heading"
      aria-busy={view.status.state === "loading"}
      className="operation-block local-plugin-capability-availability"
      data-state={view.status.state}
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Context extensions / Context 扩展</span>
          <h3 id="local-plugin-capability-availability-heading">{view.title}</h3>
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
        aria-label="Exact Plugin/MCP Context scope / 精确 Plugin/MCP Context 范围"
        columns={1}
        compact
        items={view.scope.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
        surface="raised"
        valueTone="info"
      />
      {view.entries.map((entry) => (
        <section key={entry.id} aria-label={entry.status.capabilityLabel}>
          <CapabilityState
            ariaLabel={entry.status.ariaLabel}
            description={entry.status.description}
            detail={entry.status.detail}
            label={entry.status.capabilityLabel}
            state={entry.status.state}
            stateLabel={entry.status.stateLabel}
          />
          <DefinitionGrid
            aria-label="Capability metadata / 能力元数据"
            columns={2}
            compact
            items={entry.facts.map((fact) => ({ ...fact, value: <CodeChip>{fact.value}</CodeChip> }))}
            surface="raised"
          />
        </section>
      ))}
    </section>
  );
}
