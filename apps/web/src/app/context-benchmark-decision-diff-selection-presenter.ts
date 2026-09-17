import type { StatusPillTone } from "@contextlab/ui";
import type { LocalBenchmarkDecisionDiscovery } from "./context-benchmark-decision-discovery-data";
import type { LocalBenchmarkDecisionDiffSelections } from "./context-benchmark-decision-diff-selection-data";

export type BenchmarkDecisionDiffSelectionTarget = Readonly<{
  baselineCommitId: string;
  revisedCommitId: string;
}>;

export type BenchmarkDecisionDiffSelectionResource =
  | Readonly<{
      kind: "empty" | "loading";
      target: BenchmarkDecisionDiffSelectionTarget;
    }>
  | Readonly<{
      kind: "error";
      target: BenchmarkDecisionDiffSelectionTarget;
      message: string;
    }>
  | Readonly<{
      kind: "ready";
      target: BenchmarkDecisionDiffSelectionTarget;
      selections: LocalBenchmarkDecisionDiffSelections;
    }>;

export type BenchmarkDecisionDiffSelectionOption = Readonly<{
  id: string;
  label: string;
  value: string;
}>;

export type BenchmarkDecisionDiffSelectionViewModel = Readonly<{
  state: "empty" | "loading" | "ready" | "error";
  message: string;
  baseline: Readonly<{
    commitId: string;
    options: ReadonlyArray<BenchmarkDecisionDiffSelectionOption>;
    firstDecisionId: string | null;
  }>;
  revised: Readonly<{
    commitId: string;
    options: ReadonlyArray<BenchmarkDecisionDiffSelectionOption>;
    firstDecisionId: string | null;
  }>;
}>;

export function presentBenchmarkDecisionDiffSelection(
  resource: BenchmarkDecisionDiffSelectionResource
): BenchmarkDecisionDiffSelectionViewModel {
  const baseline = resource.kind === "ready"
    ? presentSide(resource.selections.baseline)
    : emptySide(resource.target.baselineCommitId);
  const revised = resource.kind === "ready"
    ? presentSide(resource.selections.revised)
    : emptySide(resource.target.revisedCommitId);
  const message = resource.kind === "loading"
    ? "Loading exact sealed decisions / 正在加载精确的已封存 decision。"
    : resource.kind === "error"
      ? resource.message
      : resource.kind === "ready"
        ? baseline.options.length > 0 && revised.options.length > 0
          ? "Select one exact decision per commit / 为每个提交选择一个精确 decision。"
          : "Both commits need a sealed decision / 两个提交都需要存在 sealed decision。"
        : "Enter a bearer token to discover exact decisions / 输入 Bearer token 以发现精确 decision。";

  return Object.freeze({
    state: resource.kind,
    message,
    baseline,
    revised
  });
}

export function selectionStatusTone(state: BenchmarkDecisionDiffSelectionViewModel["state"]): StatusPillTone {
  if (state === "ready") {
    return "success";
  }
  if (state === "error") {
    return "warning";
  }
  if (state === "loading") {
    return "info";
  }
  return "neutral";
}

function presentSide(summary: LocalBenchmarkDecisionDiscovery) {
  const options = Object.freeze(
    summary.decisions.map((decision) =>
      Object.freeze({
        id: decision.decision_id,
        label: `${decision.suite_id} · ${decision.status} · ${decision.recorded_at}`,
        value: decision.decision_id
      })
    )
  );
  return Object.freeze({
    commitId: summary.commit_id,
    options,
    firstDecisionId: options[0]?.value ?? null
  });
}

function emptySide(commitId: string) {
  return Object.freeze({
    commitId,
    options: Object.freeze([] as BenchmarkDecisionDiffSelectionOption[]),
    firstDecisionId: null
  });
}
