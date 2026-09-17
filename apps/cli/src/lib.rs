#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! ContextLab command-line adapter staging shell.

use std::error::Error;
use std::fmt::{self, Write as _};

use contextlab_adapter_contract::{
    AdapterRequest, AdapterResponseError, AdapterState, ApplicationAdapter, ContextRequest,
    DiffRequest, EvaluationRequest, LocalCapabilityAvailabilityPresentationV1,
    LocalCapabilityAvailabilityV1, ReplayStateSnapshotProjectionV1, SharedIntegration,
    WorkflowRequest, WorkspaceRequest,
};

/// Parses a minimal ContextLab CLI command into a typed adapter request.
pub fn parse_arguments<I, S>(arguments: I) -> Result<AdapterRequest, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<_>>();

    match arguments.as_slice() {
        [resource, action, workspace_id] if resource == "workspace" && action == "inspect" => Ok(
            AdapterRequest::Workspace(WorkspaceRequest::inspect(workspace_id)),
        ),
        [resource, action, context_id] if resource == "context" && action == "inspect" => {
            Ok(AdapterRequest::Context(ContextRequest::inspect(context_id)))
        }
        [resource, action, context_id, suite_id] if resource == "evaluation" && action == "run" => {
            Ok(AdapterRequest::Evaluation(EvaluationRequest::run(
                context_id, suite_id,
            )))
        }
        [resource, action, base_revision, comparison_revision]
            if resource == "diff" && action == "compare" =>
        {
            Ok(AdapterRequest::Diff(DiffRequest::compare(
                base_revision,
                comparison_revision,
            )))
        }
        [resource, action, workflow_id] if resource == "workflow" && action == "inspect" => Ok(
            AdapterRequest::Workflow(WorkflowRequest::inspect(workflow_id)),
        ),
        _ => Err(CliError),
    }
}

/// Executes a typed request through a registered presentation adapter.
#[must_use = "handle the adapter response or its identity-drift error"]
pub fn execute<A>(
    adapter: &A,
    request: AdapterRequest,
) -> Result<CliExecution, AdapterResponseError>
where
    A: ApplicationAdapter,
{
    let command = request.command_label();
    let response = adapter.execute(request.clone());
    let availability = response.local_capability_availability(&request)?;

    Ok(match response.state {
        AdapterState::Unavailable => CliExecution {
            status: CliStatus::Unavailable(response.integration),
            message: format!(
                "{command}: unavailable; awaiting {} registration",
                response.integration
            ),
            availability,
        },
    })
}

/// Inspects one serialized local capability-availability contract without executing a domain command.
pub fn inspect_serialized_local_capability_availability(
    serialized: &str,
) -> Result<CliCapabilityInspection, CliInspectionError> {
    let availability = LocalCapabilityAvailabilityV1::parse_serialized(serialized)
        .map_err(|_| CliInspectionError)?;

    Ok(CliCapabilityInspection {
        presentation: availability.present(),
    })
}

/// Inspects one serialized replay snapshot through the shared versioning contract.
pub fn inspect_serialized_replay_state_snapshot(
    serialized: &str,
) -> Result<CliReplayStateInspection, CliReplayInspectionError> {
    let projection = ReplayStateSnapshotProjectionV1::parse_serialized(serialized)
        .map_err(|_| CliReplayInspectionError)?;

    Ok(CliReplayStateInspection { projection })
}

/// The typed terminal status for a CLI command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CliStatus {
    /// The command cannot execute until the named shared integration is registered.
    Unavailable(SharedIntegration),
}

/// A rendered CLI result suitable for a terminal adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliExecution {
    /// The typed command status.
    pub status: CliStatus,
    /// The precise user-facing status message.
    pub message: String,
    /// Versioned local availability transport data for other presentation adapters.
    pub availability: LocalCapabilityAvailabilityV1,
}

/// A deterministic, read-only terminal inspection of a local availability contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliCapabilityInspection {
    presentation: LocalCapabilityAvailabilityPresentationV1,
}

impl CliCapabilityInspection {
    /// Renders the complete bilingual inspection output in a stable field order.
    #[must_use]
    pub fn render(&self) -> String {
        let presentation = &self.presentation;

        format!(
            "schema_version: {}\noperation_id: {}\ncapability_id: {}\nintegration: {}\navailability: {}\ncapability: {} / {}\nsummary: {} / {}\ndetail: {} / {}\n",
            presentation.schema_version(),
            presentation.operation_id(),
            presentation.capability_id(),
            presentation.integration(),
            match presentation.availability() {
                contextlab_adapter_contract::LocalCapabilityAvailability::Unavailable =>
                    "unavailable",
            },
            presentation.capability().en(),
            presentation.capability().zh(),
            presentation.summary().en(),
            presentation.summary().zh(),
            presentation.detail().en(),
            presentation.detail().zh(),
        )
    }
}

/// A deterministic, read-only terminal inspection of a replay state snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliReplayStateInspection {
    projection: ReplayStateSnapshotProjectionV1,
}

impl CliReplayStateInspection {
    /// Renders stable replay identity, counts, and canonical relationship order.
    #[must_use]
    pub fn render(&self) -> String {
        let snapshot = self.projection.snapshot();
        let commit_id = snapshot
            .commit_id()
            .map(|commit_id| commit_id.to_string())
            .unwrap_or_else(|| "none".to_owned());
        let component_ids = snapshot
            .components()
            .iter()
            .map(|component| component.component_id().to_string())
            .collect::<Vec<_>>();
        let relationships = snapshot
            .relationships()
            .iter()
            .map(|relationship| {
                format!(
                    "{} -> {}",
                    relationship.source_component_id, relationship.target_component_id
                )
            })
            .collect::<Vec<_>>();
        let mut component_lines = String::new();
        for component_id in &component_ids {
            writeln!(&mut component_lines, "- {component_id}")
                .expect("writing to a String cannot fail");
        }
        let mut relationship_lines = String::new();
        for relationship in &relationships {
            writeln!(&mut relationship_lines, "relationship: {relationship}")
                .expect("writing to a String cannot fail");
        }

        format!(
            "schema_version: {}\ncontext_id: {}\ncommit_id: {commit_id}\ninitialized: {}\ncomponent_count: {}\nrelationship_count: {}\ncomponent_ids:\n{}{}",
            self.projection.schema_version(),
            snapshot.context_id(),
            self.projection.is_initialized(),
            self.projection.component_count(),
            self.projection.relationship_count(),
            component_lines,
            relationship_lines,
        )
    }
}

/// A fail-closed error returned for an invalid serialized local availability contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliInspectionError;

impl fmt::Display for CliInspectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .write_str("invalid local capability availability contract / 本地能力可用性契约无效")
    }
}

impl Error for CliInspectionError {}

/// A fail-closed error returned for an invalid serialized replay snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliReplayInspectionError;

impl fmt::Display for CliReplayInspectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid replay state snapshot / 回放状态快照无效")
    }
}

impl Error for CliReplayInspectionError {}

/// A usage error returned for unsupported command shapes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliError;

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "usage: contextlab-cli {workspace|context|workflow} inspect <id> | \\
             evaluation run <context-id> <suite-id> | diff compare <base-revision> <comparison-revision> | \\
             replay inspect <serialized-snapshot> | capability inspect <serialized-contract>",
        )
    }
}

impl Error for CliError {}
