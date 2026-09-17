import {
  CodeChip,
  DefinitionGrid,
  StackTable,
  StatusPill
} from "@contextlab/ui";
import React from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import type { LocalWorkflowContextBindingsScreenModel } from "./local-workflow-context-bindings-presenter";

export type LocalWorkflowContextBindingsScreenProps = Readonly<{
  view: LocalWorkflowContextBindingsScreenModel;
}>;

export function LocalWorkflowContextBindingsScreen({
  view
}: LocalWorkflowContextBindingsScreenProps) {
  return (
    <section
      aria-labelledby="local-workflow-context-bindings-heading"
      className="context-workflow-bindings"
      data-state={view.status.state}
      id="local-workflow-context-bindings"
    >
      <div className="context-workflow-bindings__heading">
        <h2 id="local-workflow-context-bindings-heading">{view.title}</h2>
        <p>{view.description}</p>
        <StatusPill tone="info">read-only / 只读</StatusPill>
      </div>

      <CapabilityStateScreen view={view.status} />

      <DefinitionGrid
        aria-label="Workflow binding scope / 工作流绑定范围"
        columns={2}
        compact
        items={view.scope.map((item) => ({
          id: item.id,
          label: item.label,
          value: <CodeChip>{item.value}</CodeChip>
        }))}
        surface="raised"
        valueTone="info"
      />

      {view.rows.length > 0 ? (
        <StackTable
          aria-label="Workflow context binding summaries / 工作流上下文绑定摘要"
          columnTemplate="minmax(8rem, 1fr) minmax(8rem, 1fr) minmax(5rem, 0.6fr) minmax(5rem, 0.6fr) minmax(5rem, 0.6fr)"
          headers={[
            "Binding / 绑定",
            "Workflow / 工作流",
            "Revision / 修订",
            "Elements / 元素",
            "Relations / 关系"
          ]}
          rows={view.rows.map((row) => ({
            id: row.id,
            cells: [
              <CodeChip key={`${row.id}-binding`}>{row.bindingId}</CodeChip>,
              <CodeChip key={`${row.id}-workflow`}>{row.workflowId}</CodeChip>,
              row.workflowRevision,
              row.nodeCount,
              row.edgeCount
            ]
          }))}
        />
      ) : null}
    </section>
  );
}
