//! Private, provider-free Plugin/MCP capability availability composition.
//!
//! The reusable MCP/runtime crates own compatibility and lifecycle policy. This module only
//! adapts their redacted V1 projection to the protected local API boundary.

use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_mcp::{
    CapabilityAvailability, CapabilityAvailabilityProjection, CapabilityCompatibility,
};
use serde::Serialize;
use thiserror::Error;

/// Stable private local resource schema.
pub const LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1: &str =
    "contextlab.local-plugin-capability-availability.v1";

/// One safe capability availability entry.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginCapabilityAvailabilityEntry {
    pub plugin_id: String,
    pub capability_id: String,
    pub capability_version: String,
    pub availability: &'static str,
    pub compatibility: &'static str,
    pub diagnostic_code: Option<&'static str>,
}

/// The exact Context-scoped private Plugin/MCP read resource.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PluginCapabilityAvailabilityResource {
    pub schema_version: &'static str,
    pub context_id: String,
    pub entries: Vec<PluginCapabilityAvailabilityEntry>,
}

impl PluginCapabilityAvailabilityResource {
    /// Converts the reusable projection without exposing source paths or raw manifests.
    #[must_use]
    pub fn from_projection(
        context_id: ContextId,
        projection: &CapabilityAvailabilityProjection,
    ) -> Self {
        Self {
            schema_version: LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1,
            context_id: context_id.as_uuid().to_string(),
            entries: projection
                .entries()
                .iter()
                .map(|entry| PluginCapabilityAvailabilityEntry {
                    plugin_id: entry.plugin_id().to_string(),
                    capability_id: entry.capability_id().to_string(),
                    capability_version: entry.capability_version().to_string(),
                    availability: match entry.availability() {
                        CapabilityAvailability::Available => "available",
                        CapabilityAvailability::Unavailable => "unavailable",
                    },
                    compatibility: match entry.compatibility() {
                        CapabilityCompatibility::Compatible => "compatible",
                        CapabilityCompatibility::Incompatible => "incompatible",
                    },
                    diagnostic_code: entry.diagnostic_code().map(|code| code.as_str()),
                })
                .collect(),
        }
    }

    /// Validates the response scope before it crosses the API boundary.
    pub fn validate_for_scope(
        &self,
        context_id: ContextId,
    ) -> Result<(), PluginCapabilityAvailabilityError> {
        if self.schema_version != LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1
            || self.context_id != context_id.as_uuid().to_string()
        {
            return Err(PluginCapabilityAvailabilityError::InvalidScope);
        }
        Ok(())
    }
}

/// Errors from the private Plugin/MCP projection adapter.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PluginCapabilityAvailabilityError {
    /// The projection cannot safely be returned for the requested scope.
    #[error("plugin capability availability scope is invalid")]
    InvalidScope,
}

/// Repository boundary for the protected local Plugin/MCP read.
#[async_trait]
pub trait PluginCapabilityAvailabilityRepository: Send + Sync {
    /// Reads a redacted capability projection for one exact Context.
    async fn read(
        &self,
        context_id: ContextId,
    ) -> Result<PluginCapabilityAvailabilityResource, PluginCapabilityAvailabilityError>;
}

/// Default local adapter: no plugin runtime is registered, so the projection is empty.
#[derive(Clone, Debug, Default)]
pub struct EmptyPluginCapabilityAvailabilityRepository;

#[async_trait]
impl PluginCapabilityAvailabilityRepository for EmptyPluginCapabilityAvailabilityRepository {
    async fn read(
        &self,
        context_id: ContextId,
    ) -> Result<PluginCapabilityAvailabilityResource, PluginCapabilityAvailabilityError> {
        let runtime = contextlab_plugin_runtime::PluginRuntime::current();
        Ok(PluginCapabilityAvailabilityResource::from_projection(
            context_id,
            &runtime.registry().capability_availability(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EmptyPluginCapabilityAvailabilityRepository,
        LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1, PluginCapabilityAvailabilityRepository,
    };
    use contextlab_context_core::ContextId;

    #[tokio::test]
    async fn default_projection_is_empty_and_exactly_scoped() {
        let context_id = ContextId::new();
        let resource = EmptyPluginCapabilityAvailabilityRepository
            .read(context_id)
            .await
            .expect("empty projection");

        assert_eq!(
            resource.schema_version,
            LOCAL_PLUGIN_CAPABILITY_AVAILABILITY_SCHEMA_V1
        );
        assert_eq!(resource.context_id, context_id.as_uuid().to_string());
        assert!(resource.entries.is_empty());
        resource.validate_for_scope(context_id).expect("scope");
    }
}
