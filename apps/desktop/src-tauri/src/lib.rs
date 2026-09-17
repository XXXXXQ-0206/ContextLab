#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Thin Tauri-facing staging shell for ContextLab shared application adapters.

use contextlab_adapter_contract::{
    AdapterRequest, AdapterResponseError, AdapterState, ApplicationAdapter,
    LocalCapabilityAvailabilityParseError, LocalCapabilityAvailabilityPresentationV1,
    LocalCapabilityAvailabilityV1, ReplayStateSnapshotProjectionParseError,
    ReplayStateSnapshotProjectionV1, SharedIntegration,
};

/// A Tauri-facing shell that delegates desktop requests to one application adapter.
#[derive(Debug)]
pub struct DesktopStagingShell<A> {
    adapter: A,
}

impl<A> DesktopStagingShell<A> {
    /// Creates a desktop staging shell from an application adapter.
    #[must_use]
    pub const fn new(adapter: A) -> Self {
        Self { adapter }
    }

    /// Parses a serialized local availability contract and hands its shared presentation to Desktop.
    pub fn present_serialized_local_capability_availability(
        &self,
        serialized: &str,
    ) -> Result<LocalCapabilityAvailabilityPresentationV1, LocalCapabilityAvailabilityParseError>
    {
        Ok(LocalCapabilityAvailabilityV1::parse_serialized(serialized)?.present())
    }

    /// Parses a serialized replay snapshot into the shared read-only projection.
    pub fn present_serialized_replay_state_snapshot(
        &self,
        serialized: &str,
    ) -> Result<ReplayStateSnapshotProjectionV1, ReplayStateSnapshotProjectionParseError> {
        ReplayStateSnapshotProjectionV1::parse_serialized(serialized)
    }
}

impl<A> DesktopStagingShell<A>
where
    A: ApplicationAdapter,
{
    /// Invokes a typed desktop request through the application adapter.
    #[must_use = "handle the desktop response or its identity-drift error"]
    pub fn invoke(
        &self,
        request: AdapterRequest,
    ) -> Result<DesktopExecution, AdapterResponseError> {
        let command = request.command_label();
        let response = self.adapter.execute(request.clone());
        let availability = response.local_capability_availability(&request)?;

        Ok(match response.state {
            AdapterState::Unavailable => DesktopExecution {
                status: DesktopStatus::Unavailable(response.integration),
                message: format!(
                    "{command}: unavailable; awaiting {} registration",
                    response.integration
                ),
                availability,
            },
        })
    }
}

/// The typed desktop state reported by the staging shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopStatus {
    /// The named shared integration has not yet been registered.
    Unavailable(SharedIntegration),
}

/// A desktop staging result that can be returned by a future Tauri command binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopExecution {
    /// The typed state produced by the application adapter.
    pub status: DesktopStatus,
    /// The presentation-ready status message.
    pub message: String,
    /// Versioned local availability transport data for other presentation adapters.
    pub availability: LocalCapabilityAvailabilityV1,
}
