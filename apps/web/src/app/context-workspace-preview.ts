import type {
  CommitDetail,
  CommitItem,
  ComponentDetail,
  ComponentItem,
  ContextItem,
  EvaluationRunDetail,
  EvaluationRunItem,
  EvaluationScorecard,
  ContextGraphResponse,
  ListResponse,
  ProjectItem,
  WorkspaceItem
} from "@contextlab/ts-sdk";

export const workspacePreview: ListResponse<WorkspaceItem> = {
  items: [
    {
      id: "default",
      name: "Default Workspace",
      slug: "default",
      created_at: "2026-07-09T00:00:00Z"
    }
  ],
  pagination: {
    page: 1,
    per_page: 20,
    total: 1
  }
};

export const workspaceContextGraphPreview: ContextGraphResponse = {
  graph: {
    nodes: {
      "workspace:default": {
        id: "workspace:default",
        kind: "workspace",
        label: "Default Workspace"
      },
      "project:support-ai": {
        id: "project:support-ai",
        kind: "project",
        label: "Support AI Project"
      },
      "experiment:rag-v2": {
        id: "experiment:rag-v2",
        kind: "experiment",
        label: "RAG v2 Experiment"
      },
      "context:support-resolution-agent": {
        id: "context:support-resolution-agent",
        kind: "context",
        label: "Support Resolution Agent"
      },
      "prompt:system-contract": {
        id: "prompt:system-contract",
        kind: "prompt",
        label: "System Contract"
      },
      "memory:timeline": {
        id: "memory:timeline",
        kind: "memory",
        label: "Memory Timeline"
      },
      "knowledge:refund-policy": {
        id: "knowledge:refund-policy",
        kind: "knowledge",
        label: "Refund Policy Knowledge"
      },
      "tool:mcp-search": {
        id: "tool:mcp-search",
        kind: "tool",
        label: "MCP Search Tool"
      },
      "model:deepseek": {
        id: "model:deepseek",
        kind: "model",
        label: "DeepSeek Model Config"
      },
      "evaluation:safety-regression": {
        id: "evaluation:safety-regression",
        kind: "evaluation",
        label: "Safety Regression Suite"
      }
    },
    edges: [
      {
        source: "workspace:default",
        target: "project:support-ai",
        kind: "owns"
      },
      {
        source: "project:support-ai",
        target: "experiment:rag-v2",
        kind: "owns"
      },
      {
        source: "experiment:rag-v2",
        target: "context:support-resolution-agent",
        kind: "tracks"
      },
      {
        source: "context:support-resolution-agent",
        target: "prompt:system-contract",
        kind: "contains"
      },
      {
        source: "context:support-resolution-agent",
        target: "memory:timeline",
        kind: "contains"
      },
      {
        source: "context:support-resolution-agent",
        target: "knowledge:refund-policy",
        kind: "retrieves"
      },
      {
        source: "context:support-resolution-agent",
        target: "tool:mcp-search",
        kind: "uses"
      },
      {
        source: "context:support-resolution-agent",
        target: "model:deepseek",
        kind: "configures"
      },
      {
        source: "evaluation:safety-regression",
        target: "context:support-resolution-agent",
        kind: "evaluates"
      }
    ]
  }
};

export const projectPreview: ListResponse<ProjectItem> = {
  items: [
    {
      id: "support-ai",
      workspace_id: "default",
      name: "Support AI Project",
      slug: "support-ai",
      created_at: "2026-07-09T00:00:00Z"
    }
  ],
  pagination: {
    page: 1,
    per_page: 20,
    total: 1
  }
};

export const contextPreview: ListResponse<ContextItem> = {
  items: [
    {
      id: "support-resolution-agent",
      project_id: "support-ai",
      experiment_id: "rag-v2",
      name: "Support Resolution Agent",
      description: "Production support context for resolving customer cases.",
      created_at: "2026-07-09T00:00:00Z"
    }
  ],
  pagination: {
    page: 1,
    per_page: 20,
    total: 1
  }
};

export const commitPreview: ListResponse<CommitItem> = {
  items: [
    {
      id: "support-resolution-agent-initial",
      context_id: "support-resolution-agent",
      branch_name: "main",
      message: "Create support resolution context",
      parent_commit_ids: [],
      change_count: 1,
      authored_at: "2026-07-09T00:00:00Z",
      created_at: "2026-07-09T00:00:00Z"
    },
    {
      id: "support-resolution-agent-baseline",
      context_id: "support-resolution-agent",
      branch_name: "main",
      message: "Establish support context baseline",
      parent_commit_ids: [],
      change_count: 1,
      authored_at: "2026-07-08T00:00:00Z",
      created_at: "2026-07-08T00:00:00Z"
    }
  ],
  pagination: {
    page: 1,
    per_page: 20,
    total: 2
  }
};

export const selectedCommitDetail: CommitDetail = {
  id: "support-resolution-agent-initial",
  context_id: "support-resolution-agent",
  branch_name: "main",
  message: "Create support resolution context",
  parent_commit_ids: [],
  changes: [
    {
      component_id: "refund-policy",
      operation: "add",
      summary: "Seed support policy knowledge"
    }
  ],
  change_count: 1,
  authored_at: "2026-07-09T00:00:00Z",
  created_at: "2026-07-09T00:00:00Z"
};

export const componentPreview: ListResponse<ComponentItem> = {
  items: [
    {
      id: "refund-policy",
      context_id: "support-resolution-agent",
      kind: "knowledge",
      name: "Refund Policy Knowledge",
      content_hash: "sha256:preview-refund-policy",
      created_at: "2026-07-09T00:00:00Z"
    },
    {
      id: "mcp-search",
      context_id: "support-resolution-agent",
      kind: "mcp_server",
      name: "MCP Search Tool",
      content_hash: "sha256:preview-mcp-search",
      created_at: "2026-07-09T00:00:00Z"
    },
    {
      id: "timeline",
      context_id: "support-resolution-agent",
      kind: "memory",
      name: "Memory Timeline",
      content_hash: "sha256:preview-memory-timeline",
      created_at: "2026-07-09T00:00:00Z"
    },
    {
      id: "deepseek",
      context_id: "support-resolution-agent",
      kind: "model_configuration",
      name: "DeepSeek Model Config",
      content_hash: "sha256:preview-deepseek-model-config",
      created_at: "2026-07-09T00:00:00Z"
    },
    {
      id: "system-contract",
      context_id: "support-resolution-agent",
      kind: "system_prompt",
      name: "System Contract",
      content_hash: "sha256:preview-system-contract",
      created_at: "2026-07-09T00:00:00Z"
    }
  ],
  pagination: {
    page: 1,
    per_page: 20,
    total: 5
  }
};

export const selectedComponentDetail: ComponentDetail = {
  id: "refund-policy",
  context_id: "support-resolution-agent",
  kind: "knowledge",
  name: "Refund Policy Knowledge",
  content_hash: "sha256:preview-refund-policy",
  metadata: {
    preview: true,
    source: "policy-handbook"
  },
  created_at: "2026-07-09T00:00:00Z",
  updated_at: "2026-07-09T00:00:00Z"
};

export const evaluationRunPreview: ListResponse<EvaluationRunItem> = {
  items: [
    {
      id: "safety-regression",
      context_id: "support-resolution-agent",
      suite_name: "Safety Regression Suite",
      model_version: "deepseek-chat",
      temperature: 0.2,
      metric_count: 2,
      executed_at: "2026-07-09T00:00:00Z",
      created_at: "2026-07-09T00:00:00Z"
    }
  ],
  pagination: {
    page: 1,
    per_page: 20,
    total: 1
  }
};

export const selectedEvaluationRunDetail: EvaluationRunDetail = {
  id: "safety-regression",
  context_id: "support-resolution-agent",
  suite_name: "Safety Regression Suite",
  model_version: "deepseek-chat",
  temperature: 0.2,
  metric_count: 2,
  metrics: {
    accuracy: 0.92,
    latency_ms: 820
  },
  executed_at: "2026-07-09T00:00:00Z",
  created_at: "2026-07-09T00:00:00Z"
};

export const evaluationScorecardPreview: EvaluationScorecard = {
  context_id: "support-resolution-agent",
  run_count: 1,
  metrics: [
    {
      name: "accuracy",
      average: 0.92,
      sample_count: 1
    },
    {
      name: "latency_ms",
      average: 820,
      sample_count: 1
    }
  ]
};
