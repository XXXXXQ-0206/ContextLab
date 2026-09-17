import type { CommitItem } from "@contextlab/ts-sdk";
import type { LocalCommitGraphDiffResponseV1 } from "@contextlab/local-sdk";
import type { ContextWorkspaceData, ContextWorkspaceSource } from "./context-workspace-data";

export type StatusTone = "neutral" | "success" | "warning" | "info";

export type DefinitionItem = {
  id: string;
  label: string;
  value: string;
};

export type GraphInspectorNode = {
  id: string;
  kindLabel: string;
  label: string;
  detail: string;
};

export type GraphNode = GraphInspectorNode & {
  x: string;
  y: string;
  root?: boolean;
};

export type GraphEdgeLine = {
  id: string;
  x1: string;
  y1: string;
  length: string;
  angle: string;
};

export type GraphRelationshipItem = {
  id: string;
  sourceId: string;
  sourceKind: string;
  sourceLabel: string;
  relationshipLabel: string;
  targetId: string;
  targetKind: string;
  targetLabel: string;
};

export type StatItem = {
  label: string;
  value: string;
  detail: string;
};

export type CommitTimelineItem = {
  id: string;
  message: string;
  branchName: string;
  authoredAt: string;
  changeCount: number;
  parentCount: number;
};

export type CommitChangeItem = {
  id: string;
  operation: string;
  target: string;
  summary: string;
  payloadItems: DefinitionItem[];
};

export type CommitDetailViewModel = {
  branchName: string;
  message: string;
  commitId: string;
  facts: DefinitionItem[];
  changeItems: CommitChangeItem[];
};

export type ComponentInventoryItem = {
  id: string;
  tone: StatusTone;
  kindLabel: string;
  name: string;
  createdAt: string;
  hash: string;
};

export type EvaluationRunListItem = {
  id: string;
  suiteName: string;
  modelVersion: string;
  metricsSummary: string;
};

export type ComponentDetailViewModel = {
  tone: StatusTone;
  kindLabel: string;
  name: string;
  hash: string;
  facts: DefinitionItem[];
  metadataItems: DefinitionItem[];
};

export type EvaluationDetailViewModel = {
  suiteName: string;
  modelVersion: string;
  facts: DefinitionItem[];
  metricItems: DefinitionItem[];
};

export type EvaluationScorecardMetricItem = {
  id: string;
  name: string;
  average: string;
  sampleCount: string;
};

export type EvaluationScorecardViewModel = {
  contextId: string;
  runCount: string;
  metricItems: EvaluationScorecardMetricItem[];
};

export type CommitGraphReviewOption = {
  id: string;
  label: string;
};

export type CommitGraphReviewSnapshot = {
  title: string;
  facts: DefinitionItem[];
};

export type CommitGraphReviewSection = {
  id: string;
  title: string;
  headers: string[];
  rows: Array<{ id: string; cells: string[] }>;
};

export type CommitGraphReviewResult = {
  original: CommitGraphReviewSnapshot;
  revised: CommitGraphReviewSnapshot;
  stats: StatItem[];
  sections: CommitGraphReviewSection[];
};

export type CommitGraphReviewViewModel = {
  title: string;
  contextId: string;
  candidates: CommitGraphReviewOption[];
  defaultOriginalCommitId: string | null;
  defaultRevisedCommitId: string | null;
  isInteractive: boolean;
  initialResult: CommitGraphReviewResult | null;
  unavailableMessage: string | null;
};

export type SourceModeItem = {
  id: ContextWorkspaceSource;
  label: string;
  description: string;
  statusLabel: string;
  tone: StatusTone;
  active: boolean;
};

export type ContextGraphViewModel = {
  inspectorNodes: GraphInspectorNode[];
  nodes: GraphNode[];
  edges: GraphEdgeLine[];
  relationships: GraphRelationshipItem[];
  scoreFacts: StatItem[];
};

export type ContextWorkspaceScreenModel = {
  source: {
    label: string;
    tone: StatusTone;
    description: string;
    modes: SourceModeItem[];
  };
  workspace: {
    title: string;
    description: string;
    contextFacts: DefinitionItem[];
    discoveryRoutes: DefinitionItem[];
  };
  graph: ContextGraphViewModel;
  operations: {
    commits: CommitTimelineItem[];
    commitDetail: CommitDetailViewModel;
    componentCount: number;
    componentInventory: ComponentInventoryItem[];
    componentDetail: ComponentDetailViewModel;
    evaluationRuns: EvaluationRunListItem[];
    evaluationDetail: EvaluationDetailViewModel | null;
    evaluationScorecard: EvaluationScorecardViewModel | null;
    graphReview: CommitGraphReviewViewModel;
    benchmarkEvidence: {
      projectId: string;
      contextId: string;
      candidates: CommitGraphReviewOption[];
    };
  };
};

export function presentContextWorkspaceScreen(
  data: ContextWorkspaceData
): ContextWorkspaceScreenModel {
  const selectedEvaluationRun = data.evaluationRunPreview.items[0] ?? null;
  const selectedComponentMetadata = recordEntries(data.selectedComponentDetail.metadata);
  const selectedCommitChanges = data.selectedCommitDetail.changes.map(presentCommitChange);
  const selectedEvaluationMetrics = data.selectedEvaluationRunDetail
    ? recordEntries(data.selectedEvaluationRunDetail.metrics)
    : [];
  const selectedScorecardMetrics = data.evaluationScorecard
    ? [...data.evaluationScorecard.metrics].sort((left, right) => left.name.localeCompare(right.name))
    : [];
  const graph = presentGraph(data);

  return {
    source: {
      label: sourceLabel(data.source),
      tone: sourceTone(data.source),
      description: sourceDescription(data.source),
      modes: sourceModes(data.source)
    },
    workspace: {
      title: data.selectedContext.name,
      description: data.selectedContext.description ?? "No context description available. / 暂无上下文说明。",
      contextFacts: [
        {
          id: "context-id",
          label: "Context ID",
          value: data.selectedContext.id
        },
        {
          id: "project-id",
          label: "Project",
          value: data.selectedContext.project_id
        },
        {
          id: "experiment-id",
          label: "Experiment",
          value: data.selectedContext.experiment_id ?? "Untracked"
        },
        {
          id: "created-at",
          label: "Created",
          value: formatStamp(data.selectedContext.created_at)
        }
      ],
      discoveryRoutes: [
        {
          id: "contexts",
          label: "Contexts",
          value: `/api/v1/projects/${data.selectedProject.id}/contexts`
        },
        {
          id: "workspace-context-graph",
          label: "Workspace Context Graph",
          value: `/api/v1/workspaces/${data.selectedWorkspace.id}/context-graph`
        },
        {
          id: "commits",
          label: "Commits",
          value: `/api/v1/contexts/${data.selectedContext.id}/commits`
        },
        {
          id: "commit-detail",
          label: "Commit Detail",
          value: `/api/v1/contexts/${data.selectedContext.id}/commits/${data.selectedCommitDetail.id}`
        },
        {
          id: "components",
          label: "Components",
          value: `/api/v1/contexts/${data.selectedContext.id}/components`
        },
        {
          id: "component-detail",
          label: "Component Detail",
          value: `/api/v1/contexts/${data.selectedContext.id}/components/${data.selectedComponentDetail.id}`
        },
        {
          id: "evaluation-runs",
          label: "Evaluation Runs",
          value: `/api/v1/contexts/${data.selectedContext.id}/evaluation-runs`
        },
        {
          id: "evaluation-scorecard",
          label: "Evaluation Scorecard",
          value: buildEvaluationScorecardRoute(data.selectedContext.id, selectedEvaluationRun)
        },
        ...(selectedEvaluationRun
          ? [
              {
                id: "evaluation-run-detail",
                label: "Evaluation Run Detail",
                value: `/api/v1/contexts/${data.selectedContext.id}/evaluation-runs/${selectedEvaluationRun.id}`
              }
            ]
          : [])
      ]
    },
    graph: {
      inspectorNodes: graph.inspectorNodes,
      nodes: graph.nodes,
      edges: graph.edges,
      relationships: graph.relationships,
      scoreFacts: [
        {
          label: "Graph nodes",
          value: graph.nodes.length.toString(),
          detail: `${data.workspaceContextGraph.graph.edges.length} relationships`
        },
        {
          label: "Context records",
          value: data.contextPreview.pagination.total.toString(),
          detail: "Project scoped"
        },
        {
          label: "Replayable changes",
          value: data.latestCommit.change_count.toString(),
          detail: data.latestCommit.branch_name
        },
        {
          label: "Component fingerprints",
          value: data.componentCount.toString(),
          detail: "Version review"
        },
        {
          label: "Scorecard metrics",
          value: data.evaluationScorecard?.metrics.length.toString() ?? "0",
          detail: data.evaluationScorecard
            ? `${data.evaluationScorecard.run_count} runs averaged`
            : "No scorecard"
        }
      ]
    },
    operations: {
      commits: data.commitPreview.items.map((commit) => ({
        id: commit.id,
        message: commit.message,
        branchName: commit.branch_name,
        authoredAt: formatStamp(commit.authored_at),
        changeCount: commit.change_count,
        parentCount: commit.parent_commit_ids.length
      })),
      commitDetail: {
        branchName: data.selectedCommitDetail.branch_name,
        message: data.selectedCommitDetail.message,
        commitId: data.selectedCommitDetail.id,
        facts: [
          {
            id: "commit-id",
            label: "Commit ID",
            value: data.selectedCommitDetail.id
          },
          {
            id: "authored-at",
            label: "Authored",
            value: formatStamp(data.selectedCommitDetail.authored_at)
          },
          {
            id: "created-at",
            label: "Created",
            value: formatStamp(data.selectedCommitDetail.created_at)
          },
          {
            id: "change-count",
            label: "Changes",
            value: String(data.selectedCommitDetail.change_count)
          },
          {
            id: "parent-count",
            label: "Parents",
            value: String(data.selectedCommitDetail.parent_commit_ids.length)
          }
        ],
        changeItems:
          selectedCommitChanges.length > 0
            ? selectedCommitChanges
            : [
                {
                  id: "changes-empty",
                  operation: "none",
                  target: "context",
                  summary: "No recorded changes",
                  payloadItems: [
                    {
                      id: "changes",
                      label: "changes",
                      value: "empty"
                    }
                  ]
                }
              ]
      },
      componentCount: data.componentCount,
      componentInventory: data.componentPreview.items.map((component) => ({
        id: component.id,
        tone: componentTone(component.kind),
        kindLabel: formatComponentKind(component.kind),
        name: component.name,
        createdAt: formatStamp(component.created_at),
        hash: shortHash(component.content_hash)
      })),
      componentDetail: {
        tone: componentTone(data.selectedComponentDetail.kind),
        kindLabel: formatComponentKind(data.selectedComponentDetail.kind),
        name: data.selectedComponentDetail.name,
        hash: shortHash(data.selectedComponentDetail.content_hash),
        facts: [
          {
            id: "component-id",
            label: "Component ID",
            value: data.selectedComponentDetail.id
          },
          {
            id: "component-updated",
            label: "Updated",
            value: formatStamp(data.selectedComponentDetail.updated_at)
          }
        ],
        metadataItems:
          selectedComponentMetadata.length > 0
            ? selectedComponentMetadata.map(([key, value]) => ({
                id: key,
                label: key,
                value: formatRecordValue(value)
              }))
            : [
                {
                  id: "metadata-empty",
                  label: "metadata",
                  value: "empty"
                }
              ]
      },
      evaluationRuns: data.evaluationRunPreview.items.map((run) => ({
        id: run.id,
        suiteName: run.suite_name,
        modelVersion: run.model_version,
        metricsSummary: `${run.metric_count} @ temp ${run.temperature}`
      })),
      evaluationDetail: data.selectedEvaluationRunDetail
        ? {
            suiteName: data.selectedEvaluationRunDetail.suite_name,
            modelVersion: data.selectedEvaluationRunDetail.model_version,
            facts: [
              {
                id: "run-id",
                label: "Run ID",
                value: data.selectedEvaluationRunDetail.id
              },
              {
                id: "executed-at",
                label: "Executed",
                value: formatStamp(data.selectedEvaluationRunDetail.executed_at)
              },
              {
                id: "temperature",
                label: "Temperature",
                value: String(data.selectedEvaluationRunDetail.temperature)
              },
              {
                id: "metric-count",
                label: "Metric Count",
                value: String(data.selectedEvaluationRunDetail.metric_count)
              }
            ],
            metricItems:
              selectedEvaluationMetrics.length > 0
                ? selectedEvaluationMetrics.map(([key, value]) => ({
                    id: key,
                    label: key,
                    value: formatRecordValue(value)
                  }))
                : [
                    {
                      id: "metrics-empty",
                      label: "metrics",
                      value: "empty"
                    }
                  ]
          }
        : null,
      evaluationScorecard: data.evaluationScorecard
        ? {
            contextId: data.evaluationScorecard.context_id,
            runCount: String(data.evaluationScorecard.run_count),
            metricItems: selectedScorecardMetrics.map((metric) => ({
              id: metric.name,
              name: metric.name,
              average: formatRecordValue(metric.average),
              sampleCount: String(metric.sample_count)
            }))
          }
        : null,
      graphReview: presentCommitGraphReview(data),
      benchmarkEvidence: {
        projectId: data.selectedProject.id,
        contextId: data.selectedContext.id,
        candidates: data.commitPreview.items.map(presentCommitGraphReviewOption)
      }
    }
  };
}

export function presentCommitGraphDiff(diff: LocalCommitGraphDiffResponseV1): CommitGraphReviewResult {
  return {
    original: presentCommitGraphSnapshot("Base / 基线", diff.original),
    revised: presentCommitGraphSnapshot("Compare / 对比", diff.revised),
    stats: [
      { label: "Added nodes / 新增节点", value: String(diff.diff.added_nodes.length), detail: "Nodes" },
      { label: "Removed nodes / 移除节点", value: String(diff.diff.removed_nodes.length), detail: "Nodes" },
      { label: "Modified nodes / 修改节点", value: String(diff.diff.modified_nodes.length), detail: "Nodes" },
      { label: "Added edges / 新增关系", value: String(diff.diff.added_edges.length), detail: "Edges" },
      { label: "Removed edges / 移除关系", value: String(diff.diff.removed_edges.length), detail: "Edges" }
    ],
    sections: [
      {
        id: "added-nodes",
        title: "Added Nodes / 新增节点",
        headers: ["Node ID / 节点 ID", "Kind / 类型", "Label / 标签"],
        rows: diff.diff.added_nodes.map((node) => ({ id: node.id, cells: [node.id, node.kind, node.label] }))
      },
      {
        id: "removed-nodes",
        title: "Removed Nodes / 移除节点",
        headers: ["Node ID / 节点 ID", "Kind / 类型", "Label / 标签"],
        rows: diff.diff.removed_nodes.map((node) => ({ id: node.id, cells: [node.id, node.kind, node.label] }))
      },
      {
        id: "modified-nodes",
        title: "Modified Nodes / 修改节点",
        headers: ["Node ID / 节点 ID", "Before / 修改前", "After / 修改后"],
        rows: diff.diff.modified_nodes.map((node) => ({
          id: node.node_id,
          cells: [node.node_id, `${node.original_kind}: ${node.original_label}`, `${node.revised_kind}: ${node.revised_label}`]
        }))
      },
      {
        id: "added-edges",
        title: "Added Edges / 新增关系",
        headers: ["Source / 源", "Relation / 关系", "Target / 目标"],
        rows: diff.diff.added_edges.map((edge, index) => ({
          id: `${edge.source}-${edge.kind}-${edge.target}-${index}`,
          cells: [edge.source, edge.kind, edge.target]
        }))
      },
      {
        id: "removed-edges",
        title: "Removed Edges / 移除关系",
        headers: ["Source / 源", "Relation / 关系", "Target / 目标"],
        rows: diff.diff.removed_edges.map((edge, index) => ({
          id: `${edge.source}-${edge.kind}-${edge.target}-${index}`,
          cells: [edge.source, edge.kind, edge.target]
        }))
      }
    ]
  };
}

function presentCommitGraphReview(data: ContextWorkspaceData): CommitGraphReviewViewModel {
  const revisedCommit = data.commitPreview.items[0] ?? null;
  const originalCommit = data.commitPreview.items[1] ?? null;

  return {
    title: "Commit Graph Review / 提交图谱审查",
    contextId: data.selectedContext.id,
    candidates: data.commitPreview.items.map(presentCommitGraphReviewOption),
    defaultOriginalCommitId: originalCommit?.id ?? null,
    defaultRevisedCommitId: revisedCommit?.id ?? null,
    isInteractive: data.source === "live",
    initialResult: data.commitGraphDiff ? presentCommitGraphDiff(data.commitGraphDiff) : null,
    unavailableMessage: data.commitGraphDiff ? null : graphReviewUnavailableMessage(data.commitGraphDiffUnavailableReason)
  };
}

function presentCommitGraphReviewOption(commit: CommitItem): CommitGraphReviewOption {
  return {
    id: commit.id,
    label: `${commit.message} · ${formatStamp(commit.authored_at)}`
  };
}

function presentCommitGraphSnapshot(
  title: string,
  snapshot: LocalCommitGraphDiffResponseV1["original"]
): CommitGraphReviewSnapshot {
  return {
    title,
    facts: [
      { id: "commit-id", label: "Commit ID", value: snapshot.commit_id },
      { id: "captured-at", label: "Captured", value: formatStamp(snapshot.captured_at) },
      { id: "schema-version", label: "Schema", value: String(snapshot.schema_version) }
    ]
  };
}

function graphReviewUnavailableMessage(reason: ContextWorkspaceData["commitGraphDiffUnavailableReason"]) {
  if (reason === "authenticated-local-read-required") {
    return "A request-memory Bearer token is required for local graph review / 本地图谱审阅需要请求内存中的 Bearer 令牌。";
  }
  if (reason === "snapshot-not-materialized") {
    return "Snapshot not materialized / 快照尚未物化。";
  }

  if (reason === "not-enough-commits") {
    return "At least two commits are required / 至少需要两个提交。";
  }

  if (reason === "live-data-unavailable") {
    return "Live review unavailable / 实时审阅不可用。";
  }

  return "Graph review unavailable / 图谱审阅暂不可用。";
}

export function presentGraphNodeInspector(graph: ContextGraphViewModel, nodeId: string | null) {
  const selectedNode = graph.inspectorNodes.find((node) => node.id === nodeId) ?? null;

  if (!selectedNode) {
    return {
      selectedNode: null,
      incomingRelationships: [],
      outgoingRelationships: []
    };
  }

  return {
    selectedNode,
    incomingRelationships: graph.relationships.filter((relationship) => relationship.targetId === nodeId),
    outgoingRelationships: graph.relationships.filter((relationship) => relationship.sourceId === nodeId)
  };
}

function presentCommitChange(change: Record<string, unknown>, index: number): CommitChangeItem {
  const entries = recordEntries(change);
  const rawId = firstString(change.id, change.change_id, change.component_id, change.target);

  return {
    id: `${index + 1}-${rawId ?? "change"}`,
    operation: firstString(change.operation, change.type, change.kind) ?? "change",
    target: firstString(change.component_id, change.target, change.path, change.field) ?? "context",
    summary: firstString(change.summary, change.description, change.message) ?? "No summary",
    payloadItems:
      entries.length > 0
        ? entries.map(([key, value]) => ({
            id: key,
            label: key,
            value: formatRecordValue(value)
          }))
        : [
            {
              id: "payload",
              label: "payload",
              value: "empty"
            }
          ]
  };
}

const graphKindLabels: Record<string, string> = {
  workspace: "Workspace",
  project: "Project",
  experiment: "Experiment",
  context: "Context",
  component: "Component",
  prompt: "Prompt",
  memory: "Memory",
  knowledge: "Knowledge",
  tool: "Tool",
  model: "Model",
  evaluation: "Evaluation",
  workflow: "Workflow"
};

const graphRelationshipLabels: Record<string, string> = {
  configures: "Configures",
  contains: "Contains",
  evaluates: "Evaluates",
  owns: "Owns",
  produces: "Produces",
  retrieves: "Retrieves",
  tracks: "Tracks",
  uses: "Uses"
};

const graphKindOrder: Record<string, number> = {
  workspace: 0,
  project: 1,
  experiment: 2,
  context: 3,
  prompt: 4,
  memory: 5,
  knowledge: 6,
  tool: 7,
  model: 8,
  component: 9,
  evaluation: 10,
  workflow: 11
};

const graphNodePositions = [
  ["9%", "12%"],
  ["35%", "16%"],
  ["62%", "18%"],
  ["35%", "52%"],
  ["65%", "38%"],
  ["65%", "52%"],
  ["65%", "66%"],
  ["35%", "78%"],
  ["12%", "70%"],
  ["12%", "44%"],
  ["62%", "82%"],
  ["86%", "58%"]
] as const;

type PresentedGraph = {
  inspectorNodes: GraphInspectorNode[];
  nodes: GraphNode[];
  edges: GraphEdgeLine[];
  relationships: GraphRelationshipItem[];
};

function presentGraph(data: ContextWorkspaceData): PresentedGraph {
  const inspectorNodes = Object.values(data.workspaceContextGraph.graph.nodes)
    .sort((left, right) => {
      const kindDelta = graphKindRank(left.kind) - graphKindRank(right.kind);

      if (kindDelta !== 0) {
        return kindDelta;
      }

      return left.id.localeCompare(right.id);
    })
    .map((node) => {
      const kindLabel = graphNodeKindLabel(node.kind);

      return {
        id: node.id,
        kindLabel,
        label: kindLabel,
        detail: node.label
      };
    });
  const nodes = inspectorNodes.slice(0, graphNodePositions.length).map((node, index) => {
      const [x, y] = graphNodePositions[index] ?? ["50%", "50%"];

      return {
        ...node,
        x,
        y,
        root: node.id.startsWith("context:")
      };
    });
  const positionsByNodeId = new Map(nodes.map((node) => [node.id, node]));

  return {
    inspectorNodes,
    nodes,
    edges: data.workspaceContextGraph.graph.edges.flatMap((edge, index) => {
      const source = positionsByNodeId.get(edge.source);
      const target = positionsByNodeId.get(edge.target);

      if (!source || !target) {
        return [];
      }

      const sourceX = percentNumber(source.x);
      const sourceY = percentNumber(source.y);
      const targetX = percentNumber(target.x);
      const targetY = percentNumber(target.y);
      const deltaX = targetX - sourceX;
      const deltaY = targetY - sourceY;
      const length = Math.max(4, Math.hypot(deltaX, deltaY));
      const angle = Math.atan2(deltaY, deltaX) * (180 / Math.PI);

      return [
        {
          id: `${edge.source}-${edge.kind}-${edge.target}-${index}`,
          x1: `${sourceX}%`,
          y1: `${sourceY}%`,
          length: `${length.toFixed(1)}%`,
          angle: `${angle.toFixed(1)}deg`
        }
      ];
    }),
    relationships: data.workspaceContextGraph.graph.edges.map((edge, index) => {
      const source = data.workspaceContextGraph.graph.nodes[edge.source];
      const target = data.workspaceContextGraph.graph.nodes[edge.target];

      return {
        id: `${edge.source}-${edge.kind}-${edge.target}-${index}`,
        sourceId: edge.source,
        sourceKind: source ? graphNodeKindLabel(source.kind) : "Unknown node",
        sourceLabel: source?.label ?? edge.source,
        relationshipLabel: graphRelationshipLabel(edge.kind),
        targetId: edge.target,
        targetKind: target ? graphNodeKindLabel(target.kind) : "Unknown node",
        targetLabel: target?.label ?? edge.target
      };
    })
  };
}

function graphKindRank(kind: string) {
  return graphKindOrder[kind] ?? 99;
}

function percentNumber(value: string) {
  return Number(value.replace("%", ""));
}

function formatGraphKind(value: string) {
  return value
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function graphNodeKindLabel(kind: string) {
  return graphKindLabels[kind] ?? formatGraphKind(kind);
}

function graphRelationshipLabel(kind: string) {
  return graphRelationshipLabels[kind] ?? formatGraphKind(kind);
}

function firstString(...values: unknown[]): string | null {
  for (const value of values) {
    if (typeof value === "string" && value.trim().length > 0) {
      return value;
    }
  }

  return null;
}

function formatStamp(value: string) {
  return value.replace("T", " ").replace("Z", " UTC");
}

function buildEvaluationScorecardRoute(
  contextId: string,
  selectedEvaluationRun: ContextWorkspaceData["latestEvaluationRun"]
) {
  const baseRoute = `/api/v1/contexts/${contextId}/evaluation-scorecard`;

  if (!selectedEvaluationRun) {
    return baseRoute;
  }

  const params = new URLSearchParams({
    suite_name: selectedEvaluationRun.suite_name,
    model_version: selectedEvaluationRun.model_version
  });

  return `${baseRoute}?${params.toString()}`;
}

function sourceLabel(source: ContextWorkspaceSource) {
  if (source === "live") {
    return "Live API";
  }

  if (source === "preview-fallback") {
    return "Preview fallback";
  }

  return "Preview data";
}

function sourceTone(source: ContextWorkspaceSource): StatusTone {
  if (source === "live") {
    return "info";
  }

  if (source === "preview-fallback") {
    return "warning";
  }

  return "neutral";
}

function sourceDescription(source: ContextWorkspaceSource) {
  if (source === "live") {
    return "Reading live records from CONTEXTLAB_WEB_API_BASE_URL.";
  }

  if (source === "preview-fallback") {
    return "API fetch failed; showing local preview records instead.";
  }

  return "No API base URL configured; showing local preview records.";
}

const sourceModeDescriptions: Record<ContextWorkspaceSource, string> = {
  preview: "Bundled local records for demo and offline review.",
  "preview-fallback": "Local records shown after a live API fetch fails.",
  live: "Remote records fetched from the configured API base URL."
};

const sourceModeOrder: ContextWorkspaceSource[] = ["preview", "preview-fallback", "live"];

function sourceModes(activeSource: ContextWorkspaceSource): SourceModeItem[] {
  return sourceModeOrder.map((mode) => ({
    id: mode,
    label: sourceLabel(mode),
    description: sourceModeDescriptions[mode],
    statusLabel: sourceModeStatus(mode, activeSource),
    tone: mode === activeSource ? sourceTone(mode) : "neutral",
    active: mode === activeSource
  }));
}

function sourceModeStatus(mode: ContextWorkspaceSource, activeSource: ContextWorkspaceSource) {
  if (mode === activeSource) {
    return "Active";
  }

  if (mode === "live" && activeSource === "preview") {
    return "Needs URL";
  }

  if (mode === "live" && activeSource === "preview-fallback") {
    return "Fetch failed";
  }

  if (mode === "preview-fallback" && activeSource === "preview") {
    return "Recovery path";
  }

  return "Standby";
}

function formatComponentKind(value: string) {
  const labels: Record<string, string> = {
    mcp_server: "MCP Server",
    model_configuration: "Model Config",
    output_schema: "Output Schema",
    system_prompt: "System Prompt"
  };

  if (labels[value]) {
    return labels[value];
  }

  return value
    .split("_")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

function componentTone(kind: string): StatusTone {
  if (kind === "knowledge" || kind === "memory") {
    return "success";
  }

  if (kind === "system_prompt" || kind === "prompt") {
    return "warning";
  }

  if (kind === "model_configuration" || kind === "mcp_server") {
    return "info";
  }

  return "neutral";
}

function shortHash(value: string) {
  const [algorithm, digest] = value.split(":");

  if (!digest) {
    return value;
  }

  return `${algorithm}:${digest.slice(0, 12)}`;
}

function recordEntries(record: Record<string, unknown>) {
  return Object.entries(record).sort(([left], [right]) => left.localeCompare(right));
}

function formatRecordValue(value: unknown) {
  if (value === null) {
    return "null";
  }

  if (value === undefined) {
    return "undefined";
  }

  if (typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }

  return JSON.stringify(value);
}
