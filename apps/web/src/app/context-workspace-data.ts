import { ContextLabClient } from "@contextlab/ts-sdk";
import type {
  CommitDetail,
  CommitItem,
  ComponentDetail,
  ComponentItem,
  ContextItem,
  ContextGraphResponse,
  EvaluationRunDetail,
  EvaluationRunItem,
  EvaluationScorecard,
  ListResponse,
  ProjectItem,
  WorkspaceItem
} from "@contextlab/ts-sdk";
import {
  commitPreview,
  componentPreview,
  contextPreview,
  evaluationRunPreview,
  evaluationScorecardPreview,
  projectPreview,
  selectedComponentDetail,
  selectedCommitDetail,
  selectedEvaluationRunDetail,
  workspaceContextGraphPreview,
  workspacePreview
} from "./context-workspace-preview";
import type { LocalCommitGraphDiffResponseV1 } from "@contextlab/local-sdk";

export type ContextWorkspaceSource = "preview" | "live" | "preview-fallback";
export type CommitGraphDiffUnavailableReason =
  | "authenticated-local-read-required"
  | "not-enough-commits"
  | "snapshot-not-materialized"
  | "live-data-unavailable";

export type ContextWorkspaceData = {
  source: ContextWorkspaceSource;
  workspacePreview: ListResponse<WorkspaceItem>;
  workspaceContextGraph: ContextGraphResponse;
  projectPreview: ListResponse<ProjectItem>;
  contextPreview: ListResponse<ContextItem>;
  commitPreview: ListResponse<CommitItem>;
  selectedCommitDetail: CommitDetail;
  componentPreview: ListResponse<ComponentItem>;
  selectedComponentDetail: ComponentDetail;
  evaluationRunPreview: ListResponse<EvaluationRunItem>;
  selectedEvaluationRunDetail: EvaluationRunDetail | null;
  evaluationScorecard: EvaluationScorecard | null;
  commitGraphDiff: LocalCommitGraphDiffResponseV1 | null;
  commitGraphDiffUnavailableReason: CommitGraphDiffUnavailableReason | null;
  selectedWorkspace: WorkspaceItem;
  selectedProject: ProjectItem;
  selectedContext: ContextItem;
  latestCommit: CommitItem;
  latestEvaluationRun: EvaluationRunItem | null;
  componentCount: number;
};

export async function loadContextWorkspace(): Promise<ContextWorkspaceData> {
  const previewData = buildWorkspaceData("preview", {
    workspacePreview,
    workspaceContextGraph: workspaceContextGraphPreview,
    projectPreview,
    contextPreview,
    commitPreview,
    selectedCommitDetail,
    componentPreview,
    selectedComponentDetail,
    evaluationRunPreview,
    selectedEvaluationRunDetail,
    evaluationScorecard: evaluationScorecardPreview,
    commitGraphDiff: null,
    commitGraphDiffUnavailableReason: "authenticated-local-read-required"
  });
  const apiBaseUrl = process.env.CONTEXTLAB_WEB_API_BASE_URL?.trim();

  if (!apiBaseUrl) {
    return previewData;
  }

  try {
    const client = new ContextLabClient({
      baseUrl: apiBaseUrl,
      fetch: (input, init) =>
        fetch(input, {
          ...init,
          cache: "no-store"
        })
    });
    const liveWorkspacePreview = await client.listWorkspaces({
      page: 1,
      per_page: 20,
      sort: "created_at"
    });
    const selectedWorkspace = firstItem(liveWorkspacePreview, "workspace");
    const liveProjectPreview = await client.listProjects(selectedWorkspace.id, {
      page: 1,
      per_page: 20,
      sort: "created_at"
    });
    const liveWorkspaceContextGraph = await client.getWorkspaceContextGraph(selectedWorkspace.id);
    const selectedProject = firstItem(liveProjectPreview, "project");
    const liveContextPreview = await client.listContexts(selectedProject.id, {
      page: 1,
      per_page: 20,
      sort: "created_at"
    });
    const selectedContext = firstItem(liveContextPreview, "context");
    const [liveCommitPreview, liveComponentPreview, liveEvaluationRunPreview] = await Promise.all([
      client.listCommits(selectedContext.id, {
        page: 1,
        per_page: 20,
        sort: "-authored_at"
      }),
      client.listComponents(selectedContext.id, {
        page: 1,
        per_page: 20,
        sort: "kind"
      }),
      client.listEvaluationRuns(selectedContext.id, {
        page: 1,
        per_page: 20,
        sort: "-executed_at"
      })
    ]);
    const selectedCommit = firstItem(liveCommitPreview, "commit");
    const selectedComponent = firstItem(liveComponentPreview, "component");
    const selectedEvaluationRun = firstOptional(liveEvaluationRunPreview);
    const [
      liveSelectedCommitDetail,
      liveSelectedComponentDetail,
      liveSelectedEvaluationRunDetail,
      liveEvaluationScorecard
    ] =
      await Promise.all([
        client.getCommit(selectedContext.id, selectedCommit.id),
        client.getComponent(selectedContext.id, selectedComponent.id),
        selectedEvaluationRun
          ? client.getEvaluationRun(selectedContext.id, selectedEvaluationRun.id).catch(() => null)
          : Promise.resolve(null),
        selectedEvaluationRun
          ? client
              .getEvaluationScorecard(selectedContext.id, {
                suite_name: selectedEvaluationRun.suite_name,
                model_version: selectedEvaluationRun.model_version
              })
              .catch(() => null)
          : Promise.resolve(null)
      ]);

    return buildWorkspaceData("live", {
      workspacePreview: liveWorkspacePreview,
      workspaceContextGraph: liveWorkspaceContextGraph,
      projectPreview: liveProjectPreview,
      contextPreview: liveContextPreview,
      commitPreview: liveCommitPreview,
      selectedCommitDetail: liveSelectedCommitDetail,
      componentPreview: liveComponentPreview,
      selectedComponentDetail: liveSelectedComponentDetail,
      evaluationRunPreview: liveEvaluationRunPreview,
      selectedEvaluationRunDetail: liveSelectedEvaluationRunDetail,
      evaluationScorecard: liveEvaluationScorecard,
      // Server rendering must never substitute a credential for a local protected graph-diff read.
      commitGraphDiff: null,
      commitGraphDiffUnavailableReason: "authenticated-local-read-required"
    });
  } catch {
    return {
      ...previewData,
      source: "preview-fallback",
      commitGraphDiff: null,
      commitGraphDiffUnavailableReason: "live-data-unavailable"
    };
  }
}

function buildWorkspaceData(
  source: ContextWorkspaceSource,
  responses: Pick<
    ContextWorkspaceData,
    | "workspacePreview"
    | "workspaceContextGraph"
    | "projectPreview"
    | "contextPreview"
    | "commitPreview"
    | "selectedCommitDetail"
    | "componentPreview"
    | "selectedComponentDetail"
    | "evaluationRunPreview"
    | "selectedEvaluationRunDetail"
    | "evaluationScorecard"
    | "commitGraphDiff"
    | "commitGraphDiffUnavailableReason"
  >
): ContextWorkspaceData {
  return {
    ...responses,
    source,
    selectedWorkspace: firstItem(responses.workspacePreview, "workspace"),
    selectedProject: firstItem(responses.projectPreview, "project"),
    selectedContext: firstItem(responses.contextPreview, "context"),
    latestCommit: firstItem(responses.commitPreview, "commit"),
    latestEvaluationRun: firstOptional(responses.evaluationRunPreview),
    componentCount: responses.componentPreview.pagination.total
  };
}

function firstItem<TItem>(response: ListResponse<TItem>, label: string): TItem {
  const item = response.items[0];

  if (!item) {
    throw new Error(`ContextLab ${label} list returned no items`);
  }

  return item;
}

function firstOptional<TItem>(response: ListResponse<TItem>): TItem | null {
  return response.items[0] ?? null;
}
