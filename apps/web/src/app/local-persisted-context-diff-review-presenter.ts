import type { LocalPersistedContextDiffReviewResource } from "./local-persisted-context-diff-review-data";
import type {
  LocalContextMetadata,
  LocalSemanticDiffV1
} from "@contextlab/local-sdk";

type LocalContextMetadataChangeV1 = NonNullable<LocalSemanticDiffV1["metadata_change"]>;

export type LocalPersistedContextDiffReviewView = Readonly<{
  state: "loading" | "error" | "empty" | "unavailable" | "available";
  title: string;
  message: string;
  sourceCommitId: string;
  targetCommitId: string;
    stats: ReadonlyArray<Readonly<{ label: string; value: string; detail: string }>>;
  sections: ReadonlyArray<Readonly<{ id: string; title: string; rows: ReadonlyArray<Readonly<{ id: string; cells: ReadonlyArray<string> }>> }>>;
}>;

export function presentLocalPersistedContextDiffReview(
  resource: LocalPersistedContextDiffReviewResource
): LocalPersistedContextDiffReviewView {
  const target = resource.target;
  if (resource.kind !== "ready") {
    const state = resource.kind === "loading" ? "loading" : resource.kind;
    return Object.freeze({
      state,
      title: "Persisted Context diff review / 持久化 Context Diff 审阅",
      message: resource.message ?? stateMessage(state),
      sourceCommitId: target.source_commit_id,
      targetCommitId: target.target_commit_id,
      stats: Object.freeze([]),
      sections: Object.freeze([])
    });
  }

  const diff = resource.review.diff;
  const metadataChange = readMetadataChange(diff.semantic);
  return Object.freeze({
    state: "available",
    title: "Persisted Context diff review / 持久化 Context Diff 审阅",
    message: "Rust-owned semantic, behavior, and evaluation projection / Rust 所有的 semantic、behavior 与 evaluation 投影",
    sourceCommitId: target.source_commit_id,
    targetCommitId: target.target_commit_id,
    stats: Object.freeze([
      stat("Semantic documents / Semantic 文档", diff.semantic.document_changes.length, "changes"),
      stat("Graph changes / 图谱变更", graphChangeCount(diff.semantic.graph_diff), "changes"),
      stat("Metadata changes / Metadata 变更", metadataChange ? 1 : 0, "changes"),
      stat("Behavior cases / Behavior 案例", diff.behavior.case_changes.length, "changes"),
      stat("Evaluation metrics / Evaluation 指标", diff.evaluation.metric_changes.length, "changes")
    ]),
    sections: Object.freeze([
      {
        id: "semantic-documents",
        title: "Semantic documents / Semantic 文档",
        rows: Object.freeze(diff.semantic.document_changes.map((change, index) => ({
          id: `document-${index}-${change.kind}`,
          cells: [change.kind, "document_id" in change ? change.document_id : change.document.id]
        })))
      },
      {
        id: "semantic-metadata",
        title: "Context metadata / Context 元数据",
        rows: metadataChange ? Object.freeze([{
          id: "metadata-modified",
          cells: [metadataChangeLabel(metadataChange.kind), formatMetadataChange(metadataChange)]
        }]) : Object.freeze([])
      },
      {
        id: "behavior-cases",
        title: "Behavior cases / Behavior 案例",
        rows: Object.freeze(diff.behavior.case_changes.map((change, index) => ({
          id: `behavior-${index}-${change.kind}`,
          cells: [change.kind, "original" in change ? change.original.case_id : change.revised.case_id]
        })))
      },
      {
        id: "evaluation-metrics",
        title: "Evaluation metrics / Evaluation 指标",
        rows: Object.freeze(diff.evaluation.metric_changes.map((change, index) => ({
          id: `metric-${index}-${change.kind}`,
          cells: [change.kind, "original" in change ? change.original.metric_id : change.revised.metric_id]
        })))
      }
    ])
  });
}

function readMetadataChange(semantic: LocalSemanticDiffV1): LocalContextMetadataChangeV1 | null {
  return semantic.metadata_change ?? null;
}

function formatMetadataChange(change: LocalContextMetadataChangeV1): string {
  switch (change.kind) {
    case "added":
      return `revised / 修订: ${formatMetadataSummary(change.revised)}`;
    case "removed":
      return `original / 原始: ${formatMetadataSummary(change.original)}`;
    case "modified":
      return `original / 原始: ${formatMetadataSummary(change.original)} -> revised / 修订: ${formatMetadataSummary(change.revised)}`;
  }
  return "";
}

function metadataChangeLabel(kind: LocalContextMetadataChangeV1["kind"]): string {
  switch (kind) {
    case "added": return "added / 新增";
    case "removed": return "removed / 移除";
    case "modified": return "modified / 修改";
  }
  return "metadata / 元数据";
}

function formatMetadataSummary(metadata: LocalContextMetadata): string {
  return `created_at: ${metadata.created_at}; updated_at: ${metadata.updated_at}; labels: ${formatLabels(metadata.labels)}`;
}

function formatLabels(labels: Readonly<Record<string, string>>): string {
  const entries = Object.entries(labels).sort(([left], [right]) => compareStrings(left, right));
  return entries.length === 0 ? "none / 无" : entries.map(([key, value]) => `${key}=${value}`).join(", ");
}

function compareStrings(left: string, right: string): number {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

function graphChangeCount(graph: { added_nodes: ReadonlyArray<unknown>; removed_nodes: ReadonlyArray<unknown>; modified_nodes: ReadonlyArray<unknown>; added_edges: ReadonlyArray<unknown>; removed_edges: ReadonlyArray<unknown> }): number {
  return graph.added_nodes.length + graph.removed_nodes.length + graph.modified_nodes.length + graph.added_edges.length + graph.removed_edges.length;
}

function stat(label: string, value: number, detail: string) {
  return Object.freeze({ label, value: String(value), detail });
}

function stateMessage(state: LocalPersistedContextDiffReviewView["state"]): string {
  switch (state) {
    case "loading": return "Loading persisted review / 正在加载持久化审阅。";
    case "empty": return "No persisted review is available for this commit pair / 此提交对暂无持久化审阅。";
    case "unavailable": return "Persisted review is unavailable / 持久化审阅暂不可用。";
    case "error": return "Unable to load persisted review / 无法加载持久化审阅。";
    case "available": return "";
  }
}
