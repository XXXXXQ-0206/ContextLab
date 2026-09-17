import test from "node:test";
import assert from "node:assert/strict";
import type { ContextWorkspaceData } from "./context-workspace-data";
import {
  commitPreview,
  componentPreview,
  contextPreview,
  evaluationScorecardPreview,
  evaluationRunPreview,
  projectPreview,
  selectedComponentDetail,
  selectedCommitDetail,
  selectedEvaluationRunDetail,
  workspaceContextGraphPreview,
  workspacePreview
} from "./context-workspace-preview";
import {
  presentContextWorkspaceScreen,
  presentGraphNodeInspector,
  type ContextGraphViewModel
} from "./context-workspace-presenter";

function createPreviewWorkspaceData(): ContextWorkspaceData {
  return {
    source: "preview",
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
    commitGraphDiffUnavailableReason: null,
    selectedWorkspace: workspacePreview.items[0]!,
    selectedProject: projectPreview.items[0]!,
    selectedContext: contextPreview.items[0]!,
    latestCommit: commitPreview.items[0]!,
    latestEvaluationRun: evaluationRunPreview.items[0]!,
    componentCount: componentPreview.pagination.total
  };
}

test("graph node inspector selects the project with its owns relationships", () => {
  const workspaceData = createPreviewWorkspaceData();
  const graph: ContextGraphViewModel = presentContextWorkspaceScreen(workspaceData).graph;

  const inspector = presentGraphNodeInspector(graph, "project:support-ai");

  assert.equal(inspector.selectedNode?.kindLabel, "Project");
  assert.equal(inspector.selectedNode?.label, "Project");
  assert.equal(inspector.selectedNode?.detail, "Support AI Project");
  assert.deepEqual(
    inspector.incomingRelationships.map((relationship) => relationship.relationshipLabel),
    ["Owns"]
  );
  assert.deepEqual(
    inspector.outgoingRelationships.map((relationship) => relationship.relationshipLabel),
    ["Owns"]
  );
  assert.deepEqual(
    inspector.incomingRelationships.map((relationship) => [relationship.sourceId, relationship.targetId]),
    [["workspace:default", "project:support-ai"]]
  );
  assert.deepEqual(
    inspector.outgoingRelationships.map((relationship) => [relationship.sourceId, relationship.targetId]),
    [["project:support-ai", "experiment:rag-v2"]]
  );
});

test("graph node inspector returns an empty selection for a missing node", () => {
  const workspaceData = createPreviewWorkspaceData();
  const graph: ContextGraphViewModel = presentContextWorkspaceScreen(workspaceData).graph;

  const inspector = presentGraphNodeInspector(graph, "missing:node");

  assert.equal(inspector.selectedNode, null);
  assert.deepEqual(inspector.incomingRelationships, []);
  assert.deepEqual(inspector.outgoingRelationships, []);
});

test("graph node inspector returns an empty selection without a node identifier", () => {
  const workspaceData = createPreviewWorkspaceData();
  const graph: ContextGraphViewModel = presentContextWorkspaceScreen(workspaceData).graph;

  assert.deepEqual(presentGraphNodeInspector(graph, null), {
    selectedNode: null,
    incomingRelationships: [],
    outgoingRelationships: []
  });
});

test("graph node inspector preserves the evaluation relationship direction", () => {
  const workspaceData = createPreviewWorkspaceData();
  const graph: ContextGraphViewModel = presentContextWorkspaceScreen(workspaceData).graph;

  const inspector = presentGraphNodeInspector(graph, "evaluation:safety-regression");

  assert.equal(inspector.selectedNode?.kindLabel, "Evaluation");
  assert.deepEqual(inspector.incomingRelationships, []);
  assert.deepEqual(
    inspector.outgoingRelationships.map((relationship) => [relationship.sourceId, relationship.relationshipLabel, relationship.targetId]),
    [["evaluation:safety-regression", "Evaluates", "context:support-resolution-agent"]]
  );
});

test("graph node inspector can select graph nodes beyond the visual layout limit", () => {
  const workspaceData = createPreviewWorkspaceData();
  workspaceData.workspaceContextGraph = {
    graph: {
      nodes: {
        ...workspaceData.workspaceContextGraph.graph.nodes,
        "component:workflow-input": {
          id: "component:workflow-input",
          kind: "component",
          label: "Workflow Input"
        },
        "workflow:follow-up": {
          id: "workflow:follow-up",
          kind: "workflow",
          label: "Follow-up Workflow"
        },
        "unknown_node:overflow": {
          id: "unknown_node:overflow",
          kind: "unknown_node",
          label: "Overflow Graph Node"
        }
      },
      edges: [
        ...workspaceData.workspaceContextGraph.graph.edges,
        {
          source: "context:support-resolution-agent",
          target: "unknown_node:overflow",
          kind: "uses"
        }
      ]
    }
  };

  const graph = presentContextWorkspaceScreen(workspaceData).graph;
  const inspector = presentGraphNodeInspector(graph, "unknown_node:overflow");

  assert.equal(graph.nodes.length, 12);
  assert.equal(graph.inspectorNodes.length, 13);
  assert.equal(inspector.selectedNode?.detail, "Overflow Graph Node");
  assert.deepEqual(inspector.incomingRelationships.map((relationship) => relationship.relationshipLabel), ["Uses"]);
});

test("presenter exposes stable routes, shortened hashes, and sorted metrics", () => {
  const workspaceData = createPreviewWorkspaceData();
  const model = presentContextWorkspaceScreen(workspaceData);

  assert.equal(model.source.label, "Preview data");
  assert.equal(model.source.tone, "neutral");
  assert.equal(model.source.description, "No API base URL configured; showing local preview records.");
  assert.deepEqual(
    model.source.modes.map((mode) => [mode.id, mode.statusLabel, mode.active]),
    [
      ["preview", "Active", true],
      ["preview-fallback", "Recovery path", false],
      ["live", "Needs URL", false]
    ]
  );
  assert.deepEqual(
    model.workspace.discoveryRoutes.map((route) => route.value),
    [
      "/api/v1/projects/support-ai/contexts",
      "/api/v1/workspaces/default/context-graph",
      "/api/v1/contexts/support-resolution-agent/commits",
      "/api/v1/contexts/support-resolution-agent/commits/support-resolution-agent-initial",
      "/api/v1/contexts/support-resolution-agent/components",
      "/api/v1/contexts/support-resolution-agent/components/refund-policy",
      "/api/v1/contexts/support-resolution-agent/evaluation-runs",
      "/api/v1/contexts/support-resolution-agent/evaluation-scorecard?suite_name=Safety+Regression+Suite&model_version=deepseek-chat",
      "/api/v1/contexts/support-resolution-agent/evaluation-runs/safety-regression"
    ]
  );
  assert.deepEqual(model.graph.scoreFacts.at(-1), {
    label: "Scorecard metrics",
    value: "2",
    detail: "1 runs averaged"
  });
  assert.deepEqual(
    model.graph.nodes.map((node) => node.label),
    ["Workspace", "Project", "Experiment", "Context", "Prompt", "Memory", "Knowledge", "Tool", "Model", "Evaluation"]
  );
  assert.equal(model.graph.edges.length, 9);
  assert.deepEqual(
    model.graph.relationships.slice(0, 3).map((relationship) => [
      relationship.sourceKind,
      relationship.sourceLabel,
      relationship.relationshipLabel,
      relationship.targetKind,
      relationship.targetLabel
    ]),
    [
      ["Workspace", "Default Workspace", "Owns", "Project", "Support AI Project"],
      ["Project", "Support AI Project", "Owns", "Experiment", "RAG v2 Experiment"],
      ["Experiment", "RAG v2 Experiment", "Tracks", "Context", "Support Resolution Agent"]
    ]
  );
  assert.equal(model.operations.graphReview.title, "Commit Graph Review / 提交图谱审查");
  assert.equal(model.operations.graphReview.initialResult, null);
  assert.deepEqual(
    model.operations.commitDetail.facts.map((item) => item.label),
    ["Commit ID", "Authored", "Created", "Changes", "Parents"]
  );
  assert.deepEqual(
    model.operations.commitDetail.changeItems.map((item) => [item.operation, item.target, item.summary]),
    [["add", "refund-policy", "Seed support policy knowledge"]]
  );
  assert.deepEqual(
    model.operations.commitDetail.changeItems[0]?.payloadItems.map((item) => item.label),
    ["component_id", "operation", "summary"]
  );
  assert.equal(model.operations.componentInventory[0]?.hash, "sha256:preview-refu");
  assert.deepEqual(
    model.operations.evaluationDetail?.metricItems.map((item) => item.label),
    ["accuracy", "latency_ms"]
  );
  assert.deepEqual(
    model.operations.evaluationScorecard?.metricItems.map((item) => [item.name, item.average, item.sampleCount]),
    [
      ["accuracy", "0.92", "1"],
      ["latency_ms", "820", "1"]
    ]
  );
});

test("presenter maps a persisted graph diff into bilingual review rows and counts", () => {
  const workspaceData = createPreviewWorkspaceData();
  workspaceData.commitGraphDiff = {
    context_id: "support-resolution-agent",
    pair_witness: {
      schema_version: 1,
      project_id: "support-project",
      context_id: "support-resolution-agent",
      baseline_commit_id: "support-resolution-agent-baseline",
      revised_commit_id: "support-resolution-agent-initial"
    },
    original: {
      commit_id: "support-resolution-agent-baseline",
      captured_at: "2026-07-08T00:00:00Z",
      schema_version: 1
    },
    revised: {
      commit_id: "support-resolution-agent-initial",
      captured_at: "2026-07-09T00:00:00Z",
      schema_version: 1
    },
    diff: {
      added_nodes: [{ id: "memory:timeline", kind: "memory", label: "Memory Timeline" }],
      removed_nodes: [],
      modified_nodes: [
        {
          node_id: "prompt:system-contract",
          original_kind: "prompt",
          revised_kind: "prompt",
          original_label: "System Contract v1",
          revised_label: "System Contract"
        }
      ],
      added_edges: [
        { source: "context:support-resolution-agent", target: "memory:timeline", kind: "contains" }
      ],
      removed_edges: []
    }
  };

  const review = presentContextWorkspaceScreen(workspaceData).operations.graphReview;

  assert.deepEqual(
    review.initialResult?.stats.map((item) => [item.label, item.value]),
    [
      ["Added nodes / 新增节点", "1"],
      ["Removed nodes / 移除节点", "0"],
      ["Modified nodes / 修改节点", "1"],
      ["Added edges / 新增关系", "1"],
      ["Removed edges / 移除关系", "0"]
    ]
  );
  assert.deepEqual(review.initialResult?.sections.map((section) => [section.id, section.rows.length]), [
    ["added-nodes", 1],
    ["removed-nodes", 0],
    ["modified-nodes", 1],
    ["added-edges", 1],
    ["removed-edges", 0]
  ]);
  assert.equal(review.initialResult?.original.facts[0]?.value, "support-resolution-agent-baseline");
  assert.equal(review.initialResult?.revised.facts[1]?.value, "2026-07-09 00:00:00 UTC");
});

test("presenter labels server-rendered graph review as request-memory Bearer protected", () => {
  const workspaceData = createPreviewWorkspaceData();
  workspaceData.commitGraphDiff = null;
  workspaceData.commitGraphDiffUnavailableReason = "authenticated-local-read-required";

  const review = presentContextWorkspaceScreen(workspaceData).operations.graphReview;

  assert.equal(review.initialResult, null);
  assert.match(review.unavailableMessage ?? "", /request-memory Bearer token/);
});

test("presenter uses bilingual description fallback and omits missing evaluation detail routes", () => {
  const workspaceData = createPreviewWorkspaceData();
  workspaceData.source = "preview-fallback";
  workspaceData.selectedContext = {
    ...workspaceData.selectedContext,
    description: null
  };
  workspaceData.evaluationRunPreview = {
    items: [],
    pagination: {
      page: 1,
      per_page: 20,
      total: 0
    }
  };
  workspaceData.latestEvaluationRun = null;
  workspaceData.selectedEvaluationRunDetail = null;
  workspaceData.evaluationScorecard = null;

  const model = presentContextWorkspaceScreen(workspaceData);

  assert.equal(model.source.label, "Preview fallback");
  assert.equal(model.source.tone, "warning");
  assert.equal(model.source.description, "API fetch failed; showing local preview records instead.");
  assert.deepEqual(
    model.source.modes.map((mode) => [mode.id, mode.statusLabel, mode.active]),
    [
      ["preview", "Standby", false],
      ["preview-fallback", "Active", true],
      ["live", "Fetch failed", false]
    ]
  );
  assert.equal(model.workspace.description, "No context description available. / 暂无上下文说明。");
  assert.equal(
    model.workspace.discoveryRoutes.some((route) => route.label === "Evaluation Run Detail"),
    false
  );
  assert.equal(model.operations.evaluationDetail, null);
  assert.equal(model.operations.evaluationScorecard, null);
});

test("presenter keeps evaluation detail route when live discovery exists without detail payload", () => {
  const workspaceData = createPreviewWorkspaceData();
  workspaceData.source = "live";
  workspaceData.selectedEvaluationRunDetail = null;
  workspaceData.workspaceContextGraph = {
    graph: {
      nodes: {
        "workspace:live-workspace": {
          id: "workspace:live-workspace",
          kind: "workspace",
          label: "Live Workspace Graph"
        },
        "context:live-context": {
          id: "context:live-context",
          kind: "context",
          label: "Live Context Graph"
        }
      },
      edges: [
        {
          source: "workspace:live-workspace",
          target: "context:live-context",
          kind: "owns"
        }
      ]
    }
  };

  const model = presentContextWorkspaceScreen(workspaceData);

  assert.equal(model.source.label, "Live API");
  assert.equal(model.source.tone, "info");
  assert.equal(model.source.description, "Reading live records from CONTEXTLAB_WEB_API_BASE_URL.");
  assert.deepEqual(
    model.source.modes.map((mode) => [mode.id, mode.statusLabel, mode.active]),
    [
      ["preview", "Standby", false],
      ["preview-fallback", "Standby", false],
      ["live", "Active", true]
    ]
  );
  assert.equal(
    model.workspace.discoveryRoutes.some((route) => route.value.endsWith("/evaluation-runs/safety-regression")),
    true
  );
  assert.deepEqual(
    model.graph.nodes.map((node) => [node.label, node.detail]),
    [
      ["Workspace", "Live Workspace Graph"],
      ["Context", "Live Context Graph"]
    ]
  );
  assert.deepEqual(model.graph.scoreFacts[0], {
    label: "Graph nodes",
    value: "2",
    detail: "1 relationships"
  });
  assert.equal(model.graph.edges.length, 1);
  assert.deepEqual(model.graph.relationships, [
    {
      id: "workspace:live-workspace-owns-context:live-context-0",
      sourceId: "workspace:live-workspace",
      sourceKind: "Workspace",
      sourceLabel: "Live Workspace Graph",
      relationshipLabel: "Owns",
      targetId: "context:live-context",
      targetKind: "Context",
      targetLabel: "Live Context Graph"
    }
  ]);
  assert.equal(model.operations.evaluationDetail, null);
});

test("presenter formats component labels and preserves empty metadata fallback", () => {
  const workspaceData = createPreviewWorkspaceData();
  workspaceData.componentPreview = {
    ...workspaceData.componentPreview,
    items: [
      {
        ...workspaceData.componentPreview.items[1]!,
        kind: "mcp_server"
      }
    ]
  };
  workspaceData.selectedComponentDetail = {
    ...workspaceData.selectedComponentDetail,
    metadata: {}
  };

  const model = presentContextWorkspaceScreen(workspaceData);

  assert.equal(model.operations.componentInventory[0]?.kindLabel, "MCP Server");
  assert.deepEqual(
    model.operations.componentDetail.facts.map((item) => item.label),
    ["Component ID", "Updated"]
  );
  assert.deepEqual(model.operations.componentDetail.metadataItems, [
    {
      id: "metadata-empty",
      label: "metadata",
      value: "empty"
    }
  ]);
});
