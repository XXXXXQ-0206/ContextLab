import {
  adaptLocalContextMergeReviewV1,
  type LocalContextMergeReviewResource,
  type LocalContextMergeReviewV1
} from "./local-context-merge-review-data";

export type LocalContextMergeReviewView = Readonly<{
  state: "loading" | "error" | "empty" | "unavailable" | "available";
  title: string;
  message: string;
  plan: string | null;
  scopes: ReadonlyArray<Readonly<{ id: "base" | "left" | "right"; label: string; projectId: string; contextId: string; commitId: string }>>;
  classification: Readonly<{ kind: "clean" | "equivalent" | "conflict"; label: string; items: ReadonlyArray<Readonly<{ id: string; value: string; kind: string }>> }> | null;
}>;

export function presentLocalContextMergeReview(resource: LocalContextMergeReviewResource): LocalContextMergeReviewView {
  const dto = adaptLocalContextMergeReviewV1(resource);
  const state = dto.state;
  if (state !== "available" || dto.review === undefined) {
    return Object.freeze({
      state,
      title: "Context Merge Review / Context Merge 审阅",
      message: stateMessage(state as Exclude<LocalContextMergeReviewView["state"], "available">),
      plan: null,
      scopes: Object.freeze([]),
      classification: null
    });
  }
  return deepFreeze({
    state: "available" as const,
    title: "Context Merge Review / Context Merge 审阅",
    message: "Read-only server-owned three-way review / 只读服务端拥有的三路审阅。",
    plan: `Three-way ancestry / 三路 ancestry: base=${dto.review.plan.base}; left=${dto.review.plan.left}; right=${dto.review.plan.right}`,
    scopes: [
      scopeView("base", "Base / 基线", dto.review.base_scope),
      scopeView("left", "Left / 左支线", dto.review.left_scope),
      scopeView("right", "Right / 右支线", dto.review.right_scope)
    ],
    classification: classificationView(dto.review)
  });
}

function scopeView(id: "base" | "left" | "right", label: string, scope: LocalContextMergeReviewV1["base_scope"]) {
  return { id, label, projectId: scope.project_id, contextId: scope.context_id, commitId: scope.commit_id };
}

function classificationView(review: LocalContextMergeReviewV1) {
  const labels = {
    clean: "Clean / 干净",
    equivalent: "Equivalent / 等价",
    conflict: "Conflict / 冲突"
  } as const;
  const changes = review.classification.kind === "conflict"
    ? review.classification.conflicts
    : review.classification.changes;
  return {
    kind: review.classification.kind,
    label: labels[review.classification.kind],
    items: changes.map((item, index) => ({
      id: `${review.classification.kind}-${index}`,
      kind: item.kind === "node" ? "Node / 节点" : "Edge / 边",
      value: item.kind === "node" ? item.node_id : `${item.source} -> ${item.target} (${item.edge_kind})`
    }))
  };
}

function stateMessage(state: Exclude<LocalContextMergeReviewView["state"], "available">): string {
  switch (state) {
    case "loading": return "Loading Context merge review / 正在加载 Context Merge 审阅。";
    case "error": return "Unable to load Context merge review / 无法加载 Context Merge 审阅。";
    case "empty": return "No Context merge review is available / 暂无 Context Merge 审阅。";
    case "unavailable": return "Context merge review is unavailable / Context Merge 审阅暂不可用。";
  }
}

function deepFreeze<T>(value: T): T {
  if (value !== null && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
}
