import { CodeChip, StackTable, StatGrid, StatusPill } from "@contextlab/ui";
import React from "react";
import type { LocalPersistedContextDiffReviewView } from "./local-persisted-context-diff-review-presenter";

export function LocalPersistedContextDiffReviewScreen({ view }: { view: LocalPersistedContextDiffReviewView }) {
  return (
    <section aria-busy={view.state === "loading"} aria-labelledby="local-persisted-context-diff-review-heading" className="context-benchmark-evidence">
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Version review / 版本审阅</span>
          <h3 id="local-persisted-context-diff-review-heading">{view.title}</h3>
          <p>{view.message}</p>
        </div>
        <StatusPill tone={view.state === "available" ? "success" : view.state === "loading" ? "info" : "warning"}>
          {view.state}
        </StatusPill>
      </div>
      <div className="context-benchmark-evidence__scope">
        <span>Source / 来源: <CodeChip>{view.sourceCommitId}</CodeChip></span>
        <span>Target / 目标: <CodeChip>{view.targetCommitId}</CodeChip></span>
      </div>
      {view.state !== "available" ? (
        <p aria-live="polite" role="status">{view.message}</p>
      ) : (
        <>
          <StatGrid columns={4} items={view.stats.map((item) => ({ ...item }))} />
          {view.sections.map((section) => (
            <section key={section.id} className="operation-block" aria-labelledby={`${section.id}-heading`}>
              <h4 id={`${section.id}-heading`}>{section.title}</h4>
              {section.rows.length > 0 ? <StackTable headers={["Change / 变更", "Identity / 标识"]} rows={section.rows.map((row) => ({ id: row.id, cells: [...row.cells] }))} /> : <p>No changes / 无变更</p>}
            </section>
          ))}
        </>
      )}
    </section>
  );
}
