export type ListPagination = {
  page: number;
  per_page: number;
  total: number;
};

export type ListResponse<TItem> = {
  items: TItem[];
  pagination: ListPagination;
};

export type WorkspaceItem = {
  id: string;
  name: string;
  slug: string;
  created_at: string;
};

export type ProjectItem = {
  id: string;
  workspace_id: string;
  name: string;
  slug: string;
  created_at: string;
};

export type ExperimentItem = {
  id: string;
  project_id: string;
  name: string;
  branch_name: string;
  created_at: string;
};

export type ContextItem = {
  id: string;
  project_id: string;
  experiment_id: string | null;
  name: string;
  description: string | null;
  created_at: string;
};

export type CommitItem = {
  id: string;
  context_id: string;
  branch_name: string;
  message: string;
  parent_commit_ids: string[];
  change_count: number;
  authored_at: string;
  created_at: string;
};

export type CommitChange = Record<string, unknown>;

export type CommitDetail = CommitItem & {
  changes: CommitChange[];
};

export type ComponentKind =
  | "prompt"
  | "system_prompt"
  | "memory"
  | "knowledge"
  | "retrieval"
  | "embedding"
  | "model_configuration"
  | "tool"
  | "mcp_server"
  | "variable"
  | "output_schema"
  | "workflow"
  | "conversation"
  | "evaluation";

export type ComponentItem = {
  id: string;
  context_id: string;
  kind: ComponentKind;
  name: string;
  content_hash: string;
  created_at: string;
};

export type ComponentDetail = ComponentItem & {
  metadata: Record<string, unknown>;
  updated_at: string;
};

export type EvaluationRunItem = {
  id: string;
  context_id: string;
  suite_name: string;
  model_version: string;
  temperature: number;
  metric_count: number;
  executed_at: string;
  created_at: string;
};

export type EvaluationRunDetail = EvaluationRunItem & {
  metrics: Record<string, unknown>;
};

export type EvaluationScorecardMetric = {
  name: string;
  average: number;
  sample_count: number;
};

export type EvaluationScorecard = {
  context_id: string;
  run_count: number;
  metrics: EvaluationScorecardMetric[];
};

export type HealthResponse = {
  status: "ok";
};

export type MetaResponse = {
  product: string;
  primary_abstraction: string;
  bilingual: boolean;
  context_component_kinds: string[];
  capabilities: string[];
};

export type ProviderStatus = {
  id: string;
  display_name: string;
  configured: boolean;
  base_url: string;
  api_key_env: string;
  api_key_fingerprint: string | null;
};

export type ProvidersResponse = {
  providers: ProviderStatus[];
};

export type GraphNodeKind =
  | "workspace"
  | "project"
  | "experiment"
  | "context"
  | "component"
  | "prompt"
  | "memory"
  | "knowledge"
  | "tool"
  | "model"
  | "evaluation"
  | "workflow";

export type GraphEdgeKind =
  | "owns"
  | "contains"
  | "configures"
  | "retrieves"
  | "uses"
  | "evaluates"
  | "produces"
  | "tracks";

export type GraphNode = {
  id: string;
  kind: string;
  label: string;
};

export type GraphEdge = {
  source: string;
  target: string;
  kind: string;
};

export type ContextGraph = {
  nodes: Record<string, GraphNode>;
  edges: GraphEdge[];
};

export type ContextGraphResponse = {
  graph: ContextGraph;
};

export type GraphSnapshotNode = {
  id: string;
  kind: GraphNodeKind;
  label: string;
};

export type GraphSnapshotEdge = {
  source: string;
  target: string;
  kind: GraphEdgeKind;
};

export type GraphSnapshot = {
  nodes: GraphSnapshotNode[];
  edges: GraphSnapshotEdge[];
};

export type GraphDiffRequest = {
  original: GraphSnapshot;
  revised: GraphSnapshot;
};

export type GraphNodeChange = {
  node_id: string;
  original_kind: GraphNodeKind;
  revised_kind: GraphNodeKind;
  original_label: string;
  revised_label: string;
};

export type GraphDiffResponse = {
  added_nodes: GraphNode[];
  removed_nodes: GraphNode[];
  modified_nodes: GraphNodeChange[];
  added_edges: GraphEdge[];
  removed_edges: GraphEdge[];
};

export type OpenApiDocument = {
  openapi: string;
  info: Record<string, unknown>;
  paths: Record<string, unknown>;
  [key: string]: unknown;
};

export type WorkspaceSort = "name" | "-name" | "created_at" | "-created_at";
export type ProjectSort = "name" | "-name" | "created_at" | "-created_at";
export type ExperimentSort =
  | "name"
  | "-name"
  | "branch_name"
  | "-branch_name"
  | "created_at"
  | "-created_at";
export type ContextSort = "name" | "-name" | "created_at" | "-created_at";
export type CommitSort =
  | "authored_at"
  | "-authored_at"
  | "created_at"
  | "-created_at"
  | "branch_name"
  | "-branch_name";
export type ComponentSort = "name" | "-name" | "kind" | "-kind" | "created_at" | "-created_at";
export type EvaluationRunSort =
  | "executed_at"
  | "-executed_at"
  | "created_at"
  | "-created_at"
  | "suite_name"
  | "-suite_name"
  | "model_version"
  | "-model_version";

export type ListQuery<TSort extends string> = {
  page?: number;
  per_page?: number;
  search?: string;
  sort?: TSort;
};

export type WorkspaceListQuery = ListQuery<WorkspaceSort>;
export type ProjectListQuery = ListQuery<ProjectSort>;
export type ExperimentListQuery = ListQuery<ExperimentSort>;

export type ContextListQuery = ListQuery<ContextSort> & {
  experiment_id?: string;
};

export type CommitListQuery = ListQuery<CommitSort> & {
  branch_name?: string;
};

export type ComponentListQuery = ListQuery<ComponentSort> & {
  kind?: ComponentKind;
};

export type EvaluationRunListQuery = ListQuery<EvaluationRunSort> & {
  suite_name?: string;
  model_version?: string;
};

export type EvaluationScorecardQuery = {
  search?: string;
  suite_name?: string;
  model_version?: string;
};

export type ApiErrorBody = {
  error: string;
  message: string;
};
