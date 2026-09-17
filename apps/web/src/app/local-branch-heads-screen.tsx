import { CodeChip, DefinitionGrid, Select, StackTable, StatusPill } from "@contextlab/ui";
import React from "react";
import { CapabilityStateScreen } from "./capability-state-screen";
import type { LocalBranchHeadsScreenModel } from "./local-branch-heads-presenter";

export type LocalBranchHeadsScreenProps = Readonly<{
  view: LocalBranchHeadsScreenModel;
  onSelectBranch?: (branch: string) => void;
}>;

export function LocalBranchHeadsScreen({ view, onSelectBranch }: LocalBranchHeadsScreenProps) {
  return (
    <section aria-labelledby="local-branch-heads-heading" className="local-branch-heads">
      <div className="local-branch-heads__heading">
        <div>
          <h2 id="local-branch-heads-heading">{view.title}</h2>
          <p>{view.description}</p>
        </div>
        <StatusPill tone="info">read-only / 只读</StatusPill>
      </div>

      <CapabilityStateScreen view={view.status} />

      <DefinitionGrid
        aria-label="Branch-head inspection scope / 分支 head 检查范围"
        columns={1}
        compact
        items={view.scope.map((item) => ({
          ...item,
          value: <CodeChip>{item.value}</CodeChip>
        }))}
        surface="raised"
        valueTone="info"
      />

      {view.rows.length > 0 ? (
        <>
          <Select
            label="Selected branch / 选中分支"
            name="local-branch-heads-selected-branch"
            onChange={(event) => onSelectBranch?.(event.target.value)}
            value={view.selectedBranch}
          >
            {view.rows.map((row) => (
              <option key={row.id} value={row.branch}>{row.branch}</option>
            ))}
          </Select>
          <StackTable
            aria-label="Durable Context branch heads / 持久 Context 分支 head"
            columnTemplate="minmax(8rem, 1fr) minmax(12rem, 1.4fr) minmax(5rem, 0.6fr)"
            headers={["Branch / 分支", "Head commit / Head 提交", "Revision / 修订"]}
            rows={view.rows.map((row) => ({
              id: row.id,
              cells: [
                <CodeChip key={`${row.id}-branch`}>{row.branch}</CodeChip>,
                <CodeChip key={`${row.id}-commit`}>{row.headCommitId}</CodeChip>,
                row.revision
              ]
            }))}
          />
        </>
      ) : null}
    </section>
  );
}
