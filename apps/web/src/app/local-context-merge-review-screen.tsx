import { CapabilityState, CodeChip, DefinitionGrid, StackTable, StatusPill } from "@contextlab/ui";
import React from "react";
import type { LocalContextMergeReviewResource } from "./local-context-merge-review-data";
import {
  presentLocalContextMergeReview,
  type LocalContextMergeReviewView
} from "./local-context-merge-review-presenter";

export type LocalContextMergeReviewScreenProps = Readonly<{
  resource?: LocalContextMergeReviewResource;
  view?: LocalContextMergeReviewView;
}>;

export function LocalContextMergeReviewScreen({ resource, view: providedView }: LocalContextMergeReviewScreenProps) {
  const view = providedView ?? (resource === undefined ? undefined : presentLocalContextMergeReview(resource));
  if (view === undefined) return null;
  const stateLabel = view.state === "available" ? "Ready / 已就绪" : `${view.state} / ${stateInChinese(view.state)}`;

  return (
    <section
      aria-busy={view.state === "loading" || undefined}
      aria-labelledby="local-context-merge-review-heading"
      className="operation-block local-context-merge-review"
      data-state={view.state}
      id="local-context-merge-review"
    >
      <div className="context-benchmark-evidence__heading">
        <div>
          <span className="eyebrow">Context graph / Context 图谱</span>
          <h2 id="local-context-merge-review-heading">{view.title}</h2>
          <p>{view.message}</p>
        </div>
        <StatusPill tone="info">read-only / 只读</StatusPill>
      </div>

      <CapabilityState
        ariaLabel="Context merge review state / Context Merge 审阅状态"
        description={view.message}
        id="local-context-merge-review-state"
        label="Merge review state / Merge Review 状态"
        state={view.state}
        stateLabel={stateLabel}
      />

      {view.state === "available" && view.classification ? (
        <>
          <DefinitionGrid
            aria-label="Exact three-way Context scope / 精确三路 Context 范围"
            columns={3}
            compact
            items={view.scopes.map((scope) => ({
              id: scope.id,
              label: scope.label,
              value: <ScopeValue scope={scope} />
            }))}
            surface="raised"
          />
          <DefinitionGrid
            aria-label="Server-owned merge plan / 服务端拥有的 Merge 计划"
            columns={1}
            compact
            items={[{ id: "plan", label: "Plan / 计划", value: view.plan }]}
            surface="raised"
          />
          <div className="context-benchmark-evidence__heading">
            <div>
              <h3>Classification / 分类</h3>
              <p>Server-owned graph classification / 服务端拥有的图谱分类。</p>
            </div>
            <StatusPill tone={view.classification.kind === "conflict" ? "warning" : "success"}>
              {view.classification.label}
            </StatusPill>
          </div>
          <StackTable
            aria-label="Merge classification identities / Merge 分类标识"
            headers={["Kind / 类型", "Identity / 标识"]}
            rows={view.classification.items.map((item) => ({
              id: item.id,
              cells: [item.kind, <CodeChip key={`${item.id}-value`}>{item.value}</CodeChip>]
            }))}
          />
        </>
      ) : null}
    </section>
  );
}

function ScopeValue({ scope }: { scope: LocalContextMergeReviewView["scopes"][number] }) {
  return (
    <span>
      <CodeChip>{scope.projectId}</CodeChip>{" / "}
      <CodeChip>{scope.contextId}</CodeChip>{" / "}
      <CodeChip>{scope.commitId}</CodeChip>
    </span>
  );
}

function stateInChinese(state: Exclude<LocalContextMergeReviewView["state"], "available">): string {
  return { loading: "加载中", error: "错误", empty: "空", unavailable: "不可用" }[state];
}
