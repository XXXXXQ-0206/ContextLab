//! Persistence-facing record shapes.

use chrono::{DateTime, Utc};
use contextlab_context_core::ContextComponentKind;
use contextlab_graph::{GraphEdgeKind, GraphNodeKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use thiserror::Error;

/// A persisted workspace row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    /// Workspace UUID.
    pub id: String,
    /// Workspace display name.
    pub name: String,
    /// Workspace URL-safe slug.
    pub slug: String,
    /// Workspace creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// A persisted project row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRecord {
    /// Project UUID.
    pub id: String,
    /// Parent workspace UUID.
    pub workspace_id: String,
    /// Project display name.
    pub name: String,
    /// Project URL-safe slug within its workspace.
    pub slug: String,
    /// Project creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// A persisted experiment row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperimentRecord {
    /// Experiment UUID.
    pub id: String,
    /// Parent project UUID.
    pub project_id: String,
    /// Experiment display name.
    pub name: String,
    /// Versioning branch tracked by this experiment.
    pub branch_name: String,
    /// Experiment creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// A persisted context row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRecord {
    /// Context UUID.
    pub id: String,
    /// Parent project UUID.
    pub project_id: String,
    /// Optional experiment UUID.
    pub experiment_id: Option<String>,
    /// Context display name.
    pub name: String,
    /// Optional context description.
    pub description: Option<String>,
    /// Context creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// A persisted context commit row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextCommitRecord {
    /// Commit UUID.
    pub id: String,
    /// Parent context UUID.
    pub context_id: String,
    /// Versioning branch name.
    pub branch_name: String,
    /// Commit message.
    pub message: String,
    /// Ordered parent commit UUIDs.
    pub parent_commit_ids: Vec<String>,
    /// Replayable changes stored on this commit.
    pub changes: Value,
    /// Number of replayable changes stored on this commit.
    pub change_count: u32,
    /// Authored timestamp.
    pub authored_at: DateTime<Utc>,
    /// Commit row creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// Persisted component kinds used by graph projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredComponentKind {
    /// Prompt component.
    Prompt,
    /// System prompt component.
    SystemPrompt,
    /// Memory component.
    Memory,
    /// Knowledge component.
    Knowledge,
    /// Retrieval component.
    Retrieval,
    /// Embedding component.
    Embedding,
    /// Model configuration component.
    ModelConfiguration,
    /// Tool component.
    Tool,
    /// MCP server component.
    McpServer,
    /// Variable component.
    Variable,
    /// Output schema component.
    OutputSchema,
    /// Workflow component.
    Workflow,
    /// Conversation component.
    Conversation,
    /// Evaluation dataset, suite, metric, or run component.
    Evaluation,
}

impl StoredComponentKind {
    /// All v1 stored component kinds in migration order.
    pub const ALL: &'static [Self] = &[
        Self::Prompt,
        Self::SystemPrompt,
        Self::Memory,
        Self::Knowledge,
        Self::Retrieval,
        Self::Embedding,
        Self::ModelConfiguration,
        Self::Tool,
        Self::McpServer,
        Self::Variable,
        Self::OutputSchema,
        Self::Workflow,
        Self::Conversation,
        Self::Evaluation,
    ];

    /// Converts the shared Context-domain kind into its persisted taxonomy value.
    #[must_use]
    pub(crate) const fn from_context_component_kind(kind: ContextComponentKind) -> Self {
        match kind {
            ContextComponentKind::Prompt => Self::Prompt,
            ContextComponentKind::SystemPrompt => Self::SystemPrompt,
            ContextComponentKind::Memory => Self::Memory,
            ContextComponentKind::Knowledge => Self::Knowledge,
            ContextComponentKind::Retrieval => Self::Retrieval,
            ContextComponentKind::Embedding => Self::Embedding,
            ContextComponentKind::ModelConfiguration => Self::ModelConfiguration,
            ContextComponentKind::Tool => Self::Tool,
            ContextComponentKind::McpServer => Self::McpServer,
            ContextComponentKind::Variable => Self::Variable,
            ContextComponentKind::OutputSchema => Self::OutputSchema,
            ContextComponentKind::Workflow => Self::Workflow,
            ContextComponentKind::Conversation => Self::Conversation,
            ContextComponentKind::Evaluation => Self::Evaluation,
        }
    }

    /// Returns the persisted snake_case representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::SystemPrompt => "system_prompt",
            Self::Memory => "memory",
            Self::Knowledge => "knowledge",
            Self::Retrieval => "retrieval",
            Self::Embedding => "embedding",
            Self::ModelConfiguration => "model_configuration",
            Self::Tool => "tool",
            Self::McpServer => "mcp_server",
            Self::Variable => "variable",
            Self::OutputSchema => "output_schema",
            Self::Workflow => "workflow",
            Self::Conversation => "conversation",
            Self::Evaluation => "evaluation",
        }
    }

    /// Returns the graph node kind for this stored component kind.
    #[must_use]
    pub(crate) fn graph_node_kind(self) -> GraphNodeKind {
        match self {
            Self::Prompt | Self::SystemPrompt => GraphNodeKind::Prompt,
            Self::Memory => GraphNodeKind::Memory,
            Self::Knowledge | Self::Retrieval | Self::Embedding => GraphNodeKind::Knowledge,
            Self::ModelConfiguration => GraphNodeKind::Model,
            Self::Tool | Self::McpServer => GraphNodeKind::Tool,
            Self::Variable | Self::OutputSchema | Self::Conversation => GraphNodeKind::Component,
            Self::Evaluation => GraphNodeKind::Evaluation,
            Self::Workflow => GraphNodeKind::Workflow,
        }
    }

    /// Returns the graph edge kind for this stored component kind.
    #[must_use]
    pub(crate) fn graph_edge_kind(self) -> GraphEdgeKind {
        match self {
            Self::Knowledge | Self::Retrieval | Self::Embedding => GraphEdgeKind::Retrieves,
            Self::ModelConfiguration => GraphEdgeKind::Configures,
            Self::Tool | Self::McpServer => GraphEdgeKind::Uses,
            _ => GraphEdgeKind::Contains,
        }
    }
}

impl FromStr for StoredComponentKind {
    type Err = StoredComponentKindParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "prompt" => Ok(Self::Prompt),
            "system_prompt" => Ok(Self::SystemPrompt),
            "memory" => Ok(Self::Memory),
            "knowledge" => Ok(Self::Knowledge),
            "retrieval" => Ok(Self::Retrieval),
            "embedding" => Ok(Self::Embedding),
            "model_configuration" => Ok(Self::ModelConfiguration),
            "tool" => Ok(Self::Tool),
            "mcp_server" => Ok(Self::McpServer),
            "variable" => Ok(Self::Variable),
            "output_schema" => Ok(Self::OutputSchema),
            "workflow" => Ok(Self::Workflow),
            "conversation" => Ok(Self::Conversation),
            "evaluation" => Ok(Self::Evaluation),
            other => Err(StoredComponentKindParseError {
                value: other.to_owned(),
            }),
        }
    }
}

/// Error returned when a stored component kind is outside the v1 taxonomy.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown stored component kind: {value}")]
pub struct StoredComponentKindParseError {
    value: String,
}

/// A persisted context component row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextComponentRecord {
    /// Component UUID.
    pub id: String,
    /// Parent context UUID.
    pub context_id: String,
    /// Component kind.
    pub kind: StoredComponentKind,
    /// Component display name.
    pub name: String,
    /// Reproducible content fingerprint.
    pub content_hash: String,
    /// Flexible component metadata.
    pub metadata: Value,
    /// Component creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Component update timestamp.
    pub updated_at: DateTime<Utc>,
}

/// A persisted evaluation run row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRunRecord {
    /// Evaluation run UUID.
    pub id: String,
    /// Evaluated context UUID.
    pub context_id: String,
    /// Evaluation suite name.
    pub suite_name: String,
    /// Model version used for this run.
    pub model_version: String,
    /// Sampling temperature used for this run.
    pub temperature: f32,
    /// Number of stored metric values.
    pub metric_count: u32,
    /// Persisted evaluation metrics payload.
    pub metrics: Value,
    /// Execution timestamp.
    pub executed_at: DateTime<Utc>,
    /// Run row creation timestamp.
    pub created_at: DateTime<Utc>,
}
