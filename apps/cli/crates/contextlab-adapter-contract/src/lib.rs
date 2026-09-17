#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Typed presentation-adapter contracts for ContextLab shared application crates.

use std::fmt;

use contextlab_versioning::{ReplayState, ReplayStateSnapshotV1};
use serde::{Deserialize, Serialize};

const LOCAL_CAPABILITY_AVAILABILITY_V1: &str = "contextlab.local-capability-availability.v1";

/// A request issued by a CLI, desktop, or future presentation adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterRequest {
    /// Inspects a workspace through the future context-core integration.
    Workspace(WorkspaceRequest),
    /// Inspects a context through the future context-core integration.
    Context(ContextRequest),
    /// Runs an evaluation through the future evaluation integration.
    Evaluation(EvaluationRequest),
    /// Compares revisions through the future diff-engine integration.
    Diff(DiffRequest),
    /// Inspects a workflow through the future workflow integration.
    Workflow(WorkflowRequest),
}

impl AdapterRequest {
    /// Returns the shared integration that must be registered to execute this request.
    #[must_use]
    pub const fn required_integration(&self) -> SharedIntegration {
        match self {
            Self::Workspace(_) | Self::Context(_) => SharedIntegration::ContextCore,
            Self::Evaluation(_) => SharedIntegration::Evaluation,
            Self::Diff(_) => SharedIntegration::DiffEngine,
            Self::Workflow(_) => SharedIntegration::Workflow,
        }
    }

    /// Returns the stable presentation label for this command path.
    #[must_use]
    pub const fn command_label(&self) -> &'static str {
        match self {
            Self::Workspace(_) => "workspace inspect",
            Self::Context(_) => "context inspect",
            Self::Evaluation(_) => "evaluation run",
            Self::Diff(_) => "diff compare",
            Self::Workflow(_) => "workflow inspect",
        }
    }

    /// Returns the stable local transport operation identifier.
    #[must_use]
    pub const fn operation_id(&self) -> &'static str {
        match self {
            Self::Workspace(_) => "workspace-inspect",
            Self::Context(_) => "context-inspect",
            Self::Evaluation(_) => "evaluation-run",
            Self::Diff(_) => "diff-compare",
            Self::Workflow(_) => "workflow-inspect",
        }
    }
}

/// A workspace inspection request owned by the presentation layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceRequest {
    /// The workspace selected by the user.
    pub workspace_id: String,
}

impl WorkspaceRequest {
    /// Creates a workspace inspection request.
    #[must_use]
    pub fn inspect(workspace_id: impl Into<String>) -> Self {
        Self {
            workspace_id: workspace_id.into(),
        }
    }
}

/// A context inspection request owned by the presentation layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextRequest {
    /// The context selected by the user.
    pub context_id: String,
}

impl ContextRequest {
    /// Creates a context inspection request.
    #[must_use]
    pub fn inspect(context_id: impl Into<String>) -> Self {
        Self {
            context_id: context_id.into(),
        }
    }
}

/// An evaluation-run request owned by the presentation layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationRequest {
    /// The context selected for evaluation.
    pub context_id: String,
    /// The benchmark suite selected for evaluation.
    pub suite_id: String,
}

impl EvaluationRequest {
    /// Creates an evaluation-run request.
    #[must_use]
    pub fn run(context_id: impl Into<String>, suite_id: impl Into<String>) -> Self {
        Self {
            context_id: context_id.into(),
            suite_id: suite_id.into(),
        }
    }
}

/// A revision comparison request owned by the presentation layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffRequest {
    /// The baseline revision selected by the user.
    pub base_revision: String,
    /// The revision compared against the baseline.
    pub comparison_revision: String,
}

impl DiffRequest {
    /// Creates a revision comparison request.
    #[must_use]
    pub fn compare(
        base_revision: impl Into<String>,
        comparison_revision: impl Into<String>,
    ) -> Self {
        Self {
            base_revision: base_revision.into(),
            comparison_revision: comparison_revision.into(),
        }
    }
}

/// A workflow inspection request owned by the presentation layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRequest {
    /// The workflow selected by the user.
    pub workflow_id: String,
}

impl WorkflowRequest {
    /// Creates a workflow inspection request.
    #[must_use]
    pub fn inspect(workflow_id: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
        }
    }
}

/// A shared crate integration that the Integration Lead must register.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SharedIntegration {
    /// The `contextlab-context-core` integration.
    ContextCore,
    /// The `contextlab-evaluation` integration.
    Evaluation,
    /// The `contextlab-diff-engine` integration.
    DiffEngine,
    /// The future ContextLab workflow integration.
    Workflow,
}

impl fmt::Display for SharedIntegration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ContextCore => "contextlab-context-core",
            Self::Evaluation => "contextlab-evaluation",
            Self::DiffEngine => "contextlab-diff-engine",
            Self::Workflow => "contextlab-workflow",
        })
    }
}

/// The execution state exposed to a presentation adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterState {
    /// The requested shared integration has not been registered yet.
    Unavailable,
}

/// A typed response from a presentation adapter boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterResponse {
    /// The current execution state.
    pub state: AdapterState,
    /// The missing shared integration responsible for the request.
    pub integration: SharedIntegration,
}

impl AdapterResponse {
    /// Projects this adapter response into the versioned local availability transport contract.
    pub fn local_capability_availability(
        &self,
        request: &AdapterRequest,
    ) -> Result<LocalCapabilityAvailabilityV1, AdapterResponseError> {
        if self.integration != request.required_integration() {
            return Err(AdapterResponseError);
        }

        let (availability, reason) = match self.state {
            AdapterState::Unavailable => (
                LocalCapabilityAvailability::Unavailable,
                "shared_integration_not_registered",
            ),
        };

        Ok(LocalCapabilityAvailabilityV1 {
            schema_version: LOCAL_CAPABILITY_AVAILABILITY_V1.to_owned(),
            operation_id: request.operation_id().to_owned(),
            integration: self.integration.to_string(),
            availability,
            reason: reason.to_owned(),
        })
    }
}

/// A fail-closed error for a response whose integration identity drifted from its request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterResponseError;

impl fmt::Display for AdapterResponseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("adapter response identity does not match request")
    }
}

impl std::error::Error for AdapterResponseError {}

/// Local availability state exposed by a presentation adapter.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalCapabilityAvailability {
    /// The requested core integration is not registered in this local staging adapter.
    Unavailable,
}

/// Versioned, serializable local availability projection for CLI, Desktop, and Web adapters.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalCapabilityAvailabilityV1 {
    schema_version: String,
    operation_id: String,
    integration: String,
    availability: LocalCapabilityAvailability,
    reason: String,
}

impl LocalCapabilityAvailabilityV1 {
    /// Parses and validates one serialized local availability contract.
    pub fn parse_serialized(
        serialized: &str,
    ) -> Result<Self, LocalCapabilityAvailabilityParseError> {
        let availability = serde_json::from_str::<Self>(serialized)
            .map_err(|_| LocalCapabilityAvailabilityParseError)?;
        let expected_integration = integration_for_operation(&availability.operation_id)
            .ok_or(LocalCapabilityAvailabilityParseError)?;

        if availability.schema_version != LOCAL_CAPABILITY_AVAILABILITY_V1
            || availability.integration != expected_integration
            || availability.reason != "shared_integration_not_registered"
        {
            return Err(LocalCapabilityAvailabilityParseError);
        }

        Ok(availability)
    }

    /// Returns the explicit local contract schema version.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Returns the stable operation identifier, not user-supplied request contents.
    #[must_use]
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Returns the required shared integration identifier.
    #[must_use]
    pub fn integration(&self) -> &str {
        &self.integration
    }

    /// Returns the typed local availability state.
    #[must_use]
    pub const fn availability(&self) -> LocalCapabilityAvailability {
        self.availability
    }

    /// Returns the stable machine-readable unavailability reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Projects this validated contract into deterministic bilingual presentation metadata.
    #[must_use]
    pub fn present(&self) -> LocalCapabilityAvailabilityPresentationV1 {
        let (capability, summary, detail) = presentation_metadata(&self.integration);

        LocalCapabilityAvailabilityPresentationV1 {
            schema_version: self.schema_version.clone(),
            operation_id: self.operation_id.clone(),
            capability_id: format!("local-capability-{}", self.operation_id),
            integration: self.integration.clone(),
            availability: self.availability,
            reason: self.reason.clone(),
            capability,
            summary,
            detail,
        }
    }
}

/// A validated, canonical, read-only projection of one V1 replay state snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayStateSnapshotProjectionV1 {
    snapshot: ReplayStateSnapshotV1,
}

impl ReplayStateSnapshotProjectionV1 {
    /// Parses a serialized replay snapshot through the shared versioning invariants.
    pub fn parse_serialized(
        serialized: &str,
    ) -> Result<Self, ReplayStateSnapshotProjectionParseError> {
        let value = serde_json::from_str::<serde_json::Value>(serialized)
            .map_err(|_| ReplayStateSnapshotProjectionParseError)?;
        let state =
            ReplayState::from_json(value).map_err(|_| ReplayStateSnapshotProjectionParseError)?;

        Ok(Self {
            snapshot: state.to_snapshot(),
        })
    }

    /// Serializes the projection in the canonical V1 field and collection order.
    pub fn to_serialized(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.snapshot)
    }

    /// Returns the validated versioning snapshot without granting mutation access.
    #[must_use]
    pub const fn snapshot(&self) -> &ReplayStateSnapshotV1 {
        &self.snapshot
    }

    /// Returns the explicit replay snapshot schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.snapshot.schema_version()
    }

    /// Returns whether the Context creation transition is represented.
    #[must_use]
    pub const fn is_initialized(&self) -> bool {
        self.snapshot.is_initialized()
    }

    /// Returns the number of active descriptor-only components.
    #[must_use]
    pub fn component_count(&self) -> usize {
        self.snapshot.components().len()
    }

    /// Returns the number of active `Uses` relationships.
    #[must_use]
    pub fn relationship_count(&self) -> usize {
        self.snapshot.relationships().len()
    }
}

/// A fail-closed parse error for a replay state snapshot projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayStateSnapshotProjectionParseError;

impl fmt::Display for ReplayStateSnapshotProjectionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid replay state snapshot projection / 回放状态快照投影无效")
    }
}

impl std::error::Error for ReplayStateSnapshotProjectionParseError {}

fn integration_for_operation(operation_id: &str) -> Option<&'static str> {
    match operation_id {
        "workspace-inspect" | "context-inspect" => Some("contextlab-context-core"),
        "evaluation-run" => Some("contextlab-evaluation"),
        "diff-compare" => Some("contextlab-diff-engine"),
        "workflow-inspect" => Some("contextlab-workflow"),
        _ => None,
    }
}

fn presentation_metadata(integration: &str) -> (BilingualText, BilingualText, BilingualText) {
    match integration {
        "contextlab-context-core" => (
            BilingualText::new("Context core", "上下文核心"),
            BilingualText::new(
                "Context core is unavailable because shared registration has not been completed.",
                "上下文核心因共享注册尚未完成而不可用。",
            ),
            BilingualText::new(
                "Shared integration: contextlab-context-core.",
                "共享集成：contextlab-context-core。",
            ),
        ),
        "contextlab-evaluation" => (
            BilingualText::new("Evaluation engine", "评测引擎"),
            BilingualText::new(
                "Evaluation engine is unavailable because shared registration has not been completed.",
                "评测引擎因共享注册尚未完成而不可用。",
            ),
            BilingualText::new(
                "Shared integration: contextlab-evaluation.",
                "共享集成：contextlab-evaluation。",
            ),
        ),
        "contextlab-diff-engine" => (
            BilingualText::new("Diff engine", "差异引擎"),
            BilingualText::new(
                "Diff engine is unavailable because shared registration has not been completed.",
                "差异引擎因共享注册尚未完成而不可用。",
            ),
            BilingualText::new(
                "Shared integration: contextlab-diff-engine.",
                "共享集成：contextlab-diff-engine。",
            ),
        ),
        "contextlab-workflow" => (
            BilingualText::new("Workflow engine", "工作流引擎"),
            BilingualText::new(
                "Workflow engine is unavailable because shared registration has not been completed.",
                "工作流引擎因共享注册尚未完成而不可用。",
            ),
            BilingualText::new(
                "Shared integration: contextlab-workflow.",
                "共享集成：contextlab-workflow。",
            ),
        ),
        _ => unreachable!("validated local availability integration"),
    }
}

/// A fail-closed parse error for a local capability availability contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalCapabilityAvailabilityParseError;

impl fmt::Display for LocalCapabilityAvailabilityParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid local capability availability contract")
    }
}

impl std::error::Error for LocalCapabilityAvailabilityParseError {}

/// Bilingual metadata supplied by a local presentation adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BilingualText {
    en: &'static str,
    zh: &'static str,
}

impl BilingualText {
    const fn new(en: &'static str, zh: &'static str) -> Self {
        Self { en, zh }
    }

    /// Returns the English presentation text.
    #[must_use]
    pub const fn en(&self) -> &'static str {
        self.en
    }

    /// Returns the Chinese presentation text.
    #[must_use]
    pub const fn zh(&self) -> &'static str {
        self.zh
    }
}

/// Deterministic local presentation metadata derived from a validated availability contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalCapabilityAvailabilityPresentationV1 {
    schema_version: String,
    operation_id: String,
    capability_id: String,
    integration: String,
    availability: LocalCapabilityAvailability,
    reason: String,
    capability: BilingualText,
    summary: BilingualText,
    detail: BilingualText,
}

impl LocalCapabilityAvailabilityPresentationV1 {
    /// Returns the validated source contract schema version.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Returns the stable source operation identifier.
    #[must_use]
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Returns the stable local capability identifier derived from the operation identifier.
    #[must_use]
    pub fn capability_id(&self) -> &str {
        &self.capability_id
    }

    /// Returns the exact shared integration identifier from the validated source contract.
    #[must_use]
    pub fn integration(&self) -> &str {
        &self.integration
    }

    /// Returns the validated availability state.
    #[must_use]
    pub const fn availability(&self) -> LocalCapabilityAvailability {
        self.availability
    }

    /// Returns the machine-readable unavailable reason from the validated source contract.
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns the bilingual capability label.
    #[must_use]
    pub const fn capability(&self) -> BilingualText {
        self.capability
    }

    /// Returns the bilingual unavailable summary.
    #[must_use]
    pub const fn summary(&self) -> BilingualText {
        self.summary
    }

    /// Returns the bilingual local-boundary detail.
    #[must_use]
    pub const fn detail(&self) -> BilingualText {
        self.detail
    }
}

/// An interface implemented by adapters backed by shared application crates.
pub trait ApplicationAdapter {
    /// Executes a presentation-layer request.
    fn execute(&self, request: AdapterRequest) -> AdapterResponse;
}

/// An adapter used until the Integration Lead wires shared application crates.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnavailableApplicationAdapter;

impl ApplicationAdapter for UnavailableApplicationAdapter {
    fn execute(&self, request: AdapterRequest) -> AdapterResponse {
        AdapterResponse {
            state: AdapterState::Unavailable,
            integration: request.required_integration(),
        }
    }
}
