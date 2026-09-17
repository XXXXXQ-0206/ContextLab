//! Projection from persistence records into Context Graphs.

use crate::{
    ContextCommitRecord, ContextComponentRecord, ContextRecord, EvaluationRunRecord,
    ExperimentRecord, ProjectRecord, StoredComponentKind, WorkspaceRecord,
};
use chrono::{DateTime, Utc};
use contextlab_graph::{
    ContextGraph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind, GraphValidationError,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

/// Records required to project a Context Graph.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ContextGraphProjection {
    /// Workspace records.
    pub workspaces: Vec<WorkspaceRecord>,
    /// Project records.
    pub projects: Vec<ProjectRecord>,
    /// Experiment records.
    pub experiments: Vec<ExperimentRecord>,
    /// Context records.
    pub contexts: Vec<ContextRecord>,
    /// Context commit records.
    pub commits: Vec<ContextCommitRecord>,
    /// Component records.
    pub components: Vec<ContextComponentRecord>,
    /// Evaluation run records.
    pub evaluation_runs: Vec<EvaluationRunRecord>,
}

impl ContextGraphProjection {
    /// Builds the preview records used until a real repository is connected.
    #[must_use]
    pub fn context_engineering_preview() -> Self {
        Self {
            workspaces: vec![WorkspaceRecord {
                id: "default".to_owned(),
                name: "Default Workspace".to_owned(),
                slug: "default".to_owned(),
                created_at: preview_created_at(),
            }],
            projects: vec![ProjectRecord {
                id: "support-ai".to_owned(),
                workspace_id: "default".to_owned(),
                name: "Support AI Project".to_owned(),
                slug: "support-ai".to_owned(),
                created_at: preview_created_at(),
            }],
            experiments: vec![ExperimentRecord {
                id: "rag-v2".to_owned(),
                project_id: "support-ai".to_owned(),
                name: "RAG v2 Experiment".to_owned(),
                branch_name: "experiment/rag-v2".to_owned(),
                created_at: preview_created_at(),
            }],
            contexts: vec![ContextRecord {
                id: "support-resolution-agent".to_owned(),
                project_id: "support-ai".to_owned(),
                experiment_id: Some("rag-v2".to_owned()),
                name: "Support Resolution Agent".to_owned(),
                description: Some(
                    "Production support context for resolving customer cases.".to_owned(),
                ),
                created_at: preview_created_at(),
            }],
            commits: vec![ContextCommitRecord {
                id: "support-resolution-agent-initial".to_owned(),
                context_id: "support-resolution-agent".to_owned(),
                branch_name: "main".to_owned(),
                message: "Create support resolution context".to_owned(),
                parent_commit_ids: Vec::new(),
                changes: json!([
                    {
                        "operation": "create_context",
                        "path": "/contexts/support-resolution-agent",
                        "summary": "Created the production support resolution context"
                    }
                ]),
                change_count: 1,
                authored_at: preview_created_at(),
                created_at: preview_created_at(),
            }],
            components: vec![
                ContextComponentRecord {
                    id: "system-contract".to_owned(),
                    context_id: "support-resolution-agent".to_owned(),
                    kind: StoredComponentKind::SystemPrompt,
                    name: "System Contract".to_owned(),
                    content_hash: "sha256:preview-system-contract".to_owned(),
                    metadata: json!({
                        "preview": true,
                        "owner": "context-core",
                        "locale": "bilingual"
                    }),
                    created_at: preview_created_at(),
                    updated_at: preview_created_at(),
                },
                ContextComponentRecord {
                    id: "timeline".to_owned(),
                    context_id: "support-resolution-agent".to_owned(),
                    kind: StoredComponentKind::Memory,
                    name: "Memory Timeline".to_owned(),
                    content_hash: "sha256:preview-memory-timeline".to_owned(),
                    metadata: json!({
                        "preview": true,
                        "retention": "session-summary"
                    }),
                    created_at: preview_created_at(),
                    updated_at: preview_created_at(),
                },
                ContextComponentRecord {
                    id: "refund-policy".to_owned(),
                    context_id: "support-resolution-agent".to_owned(),
                    kind: StoredComponentKind::Knowledge,
                    name: "Refund Policy Knowledge".to_owned(),
                    content_hash: "sha256:preview-refund-policy".to_owned(),
                    metadata: json!({
                        "preview": true,
                        "source": "policy-handbook"
                    }),
                    created_at: preview_created_at(),
                    updated_at: preview_created_at(),
                },
                ContextComponentRecord {
                    id: "mcp-search".to_owned(),
                    context_id: "support-resolution-agent".to_owned(),
                    kind: StoredComponentKind::McpServer,
                    name: "MCP Search Tool".to_owned(),
                    content_hash: "sha256:preview-mcp-search".to_owned(),
                    metadata: json!({
                        "preview": true,
                        "transport": "stdio"
                    }),
                    created_at: preview_created_at(),
                    updated_at: preview_created_at(),
                },
                ContextComponentRecord {
                    id: "deepseek".to_owned(),
                    context_id: "support-resolution-agent".to_owned(),
                    kind: StoredComponentKind::ModelConfiguration,
                    name: "DeepSeek Model Config".to_owned(),
                    content_hash: "sha256:preview-deepseek-model-config".to_owned(),
                    metadata: json!({
                        "preview": true,
                        "provider": "deepseek",
                        "model": "deepseek-chat"
                    }),
                    created_at: preview_created_at(),
                    updated_at: preview_created_at(),
                },
            ],
            evaluation_runs: vec![EvaluationRunRecord {
                id: "safety-regression".to_owned(),
                context_id: "support-resolution-agent".to_owned(),
                suite_name: "Safety Regression Suite".to_owned(),
                model_version: "deepseek-chat".to_owned(),
                temperature: 0.2,
                metric_count: 2,
                metrics: json!({
                    "accuracy": 0.92,
                    "latency_ms": 820
                }),
                executed_at: preview_created_at(),
                created_at: preview_created_at(),
            }],
        }
    }

    /// Projects persistence records into a validated Context Graph.
    pub fn project(&self) -> Result<ContextGraph, StorageProjectionError> {
        let mut graph = ContextGraph::new();

        for workspace in &self.workspaces {
            graph.add_node(GraphNode::new(
                node_id("workspace", &workspace.id),
                GraphNodeKind::Workspace,
                workspace.name.clone(),
            )?)?;
        }

        for project in &self.projects {
            graph.add_node(GraphNode::new(
                node_id("project", &project.id),
                GraphNodeKind::Project,
                project.name.clone(),
            )?)?;
            graph.add_edge(GraphEdge::new(
                node_id("workspace", &project.workspace_id),
                node_id("project", &project.id),
                GraphEdgeKind::Owns,
            )?)?;
        }

        for experiment in &self.experiments {
            graph.add_node(GraphNode::new(
                node_id("experiment", &experiment.id),
                GraphNodeKind::Experiment,
                experiment.name.clone(),
            )?)?;
            graph.add_edge(GraphEdge::new(
                node_id("project", &experiment.project_id),
                node_id("experiment", &experiment.id),
                GraphEdgeKind::Owns,
            )?)?;
        }

        for context in &self.contexts {
            graph.add_node(GraphNode::new(
                node_id("context", &context.id),
                GraphNodeKind::Context,
                context.name.clone(),
            )?)?;
            graph.add_edge(GraphEdge::new(
                node_id("project", &context.project_id),
                node_id("context", &context.id),
                GraphEdgeKind::Owns,
            )?)?;
            if let Some(experiment_id) = &context.experiment_id {
                graph.add_edge(GraphEdge::new(
                    node_id("experiment", experiment_id),
                    node_id("context", &context.id),
                    GraphEdgeKind::Tracks,
                )?)?;
            }
        }

        for component in &self.components {
            graph.add_node(GraphNode::new(
                node_id("component", &component.id),
                component.kind.graph_node_kind(),
                component.name.clone(),
            )?)?;
            graph.add_edge(GraphEdge::new(
                node_id("context", &component.context_id),
                node_id("component", &component.id),
                component.kind.graph_edge_kind(),
            )?)?;
        }

        for run in &self.evaluation_runs {
            graph.add_node(GraphNode::new(
                node_id("evaluation", &run.id),
                GraphNodeKind::Evaluation,
                run.suite_name.clone(),
            )?)?;
            graph.add_edge(GraphEdge::new(
                node_id("evaluation", &run.id),
                node_id("context", &run.context_id),
                GraphEdgeKind::Evaluates,
            )?)?;
        }

        Ok(graph)
    }
}

/// Projection errors.
#[derive(Debug, Error)]
pub enum StorageProjectionError {
    /// A record could not be converted into a valid graph element.
    #[error("invalid graph projection: {0}")]
    Graph(#[from] GraphValidationError),
}

fn node_id(prefix: &str, id: &str) -> String {
    format!("{prefix}:{}", id.trim())
}

fn preview_created_at() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-07-09T00:00:00Z")
        .expect("preview timestamp must be valid RFC3339")
        .with_timezone(&Utc)
}
