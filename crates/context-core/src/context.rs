//! Context aggregate.

use crate::{
    ContextComponent, ContextId, DomainValidationError, ExperimentId, NonEmptyString, ProjectId,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Descriptive metadata for a Context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextMetadata {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    labels: BTreeMap<String, String>,
}

impl ContextMetadata {
    /// Creates metadata with the same creation and update timestamp.
    #[must_use]
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            created_at: now,
            updated_at: now,
            labels: BTreeMap::new(),
        }
    }

    /// Returns creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns last update timestamp.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Returns immutable labels.
    #[must_use]
    pub const fn labels(&self) -> &BTreeMap<String, String> {
        &self.labels
    }

    /// Validates the metadata timestamp invariants before persistence or replay.
    pub fn validate(&self) -> Result<(), DomainValidationError> {
        if self.updated_at < self.created_at {
            return Err(DomainValidationError::MetadataTimestampsOutOfOrder {
                created_at: self.created_at,
                updated_at: self.updated_at,
            });
        }
        Ok(())
    }

    /// Sets or replaces a metadata label.
    pub fn set_label(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        now: DateTime<Utc>,
    ) {
        self.labels.insert(key.into(), value.into());
        self.updated_at = now;
    }

    /// Marks the metadata as updated.
    pub fn touch(&mut self, now: DateTime<Utc>) {
        self.updated_at = now;
    }
}

/// A complete versionable unit of AI context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Context {
    id: ContextId,
    project_id: ProjectId,
    experiment_id: Option<ExperimentId>,
    name: NonEmptyString,
    description: Option<NonEmptyString>,
    components: Vec<ContextComponent>,
    metadata: ContextMetadata,
}

impl Context {
    /// Creates a new Context aggregate.
    pub fn new(
        project_id: ProjectId,
        name: impl Into<String>,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainValidationError> {
        Ok(Self {
            id: ContextId::new(),
            project_id,
            experiment_id: None,
            name: NonEmptyString::new("context.name", name)?,
            description: None,
            components: Vec::new(),
            metadata: ContextMetadata::new(now),
        })
    }

    /// Returns the Context identifier.
    #[must_use]
    pub const fn id(&self) -> ContextId {
        self.id
    }

    /// Returns the parent project identifier.
    #[must_use]
    pub const fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Returns the optional experiment identifier.
    #[must_use]
    pub const fn experiment_id(&self) -> Option<ExperimentId> {
        self.experiment_id
    }

    /// Returns the context name.
    #[must_use]
    pub fn name(&self) -> &NonEmptyString {
        &self.name
    }

    /// Returns all attached components.
    #[must_use]
    pub fn components(&self) -> &[ContextComponent] {
        &self.components
    }

    /// Returns metadata.
    #[must_use]
    pub const fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }

    /// Attaches this context to an experiment.
    pub fn attach_to_experiment(&mut self, experiment_id: ExperimentId, now: DateTime<Utc>) {
        self.experiment_id = Some(experiment_id);
        self.metadata.touch(now);
    }

    /// Sets a validated optional description.
    pub fn set_description(
        &mut self,
        description: impl Into<String>,
        now: DateTime<Utc>,
    ) -> Result<(), DomainValidationError> {
        self.description = Some(NonEmptyString::new("context.description", description)?);
        self.metadata.touch(now);
        Ok(())
    }

    /// Adds a component and updates metadata.
    pub fn add_component(&mut self, component: ContextComponent, now: DateTime<Utc>) {
        self.components.push(component);
        self.metadata.touch(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ContextComponentKind;

    #[test]
    fn creates_context_with_trimmed_name() {
        let now = Utc::now();
        let context = Context::new(ProjectId::new(), "  Customer Support Agent  ", now)
            .expect("valid context");

        assert_eq!(context.name().as_str(), "Customer Support Agent");
        assert!(context.components().is_empty());
        assert_eq!(context.metadata().created_at(), now);
    }

    #[test]
    fn adds_context_component_and_updates_timestamp() {
        let created_at = Utc::now();
        let updated_at = created_at + chrono::Duration::seconds(5);
        let mut context =
            Context::new(ProjectId::new(), "Research Agent", created_at).expect("valid context");
        let component = ContextComponent::new(
            ContextComponentKind::SystemPrompt,
            "System Contract",
            "sha256:contract",
        )
        .expect("valid component");

        context.add_component(component, updated_at);

        assert_eq!(context.components().len(), 1);
        assert_eq!(context.metadata().updated_at(), updated_at);
    }

    #[test]
    fn metadata_validation_rejects_an_update_before_creation() {
        let created_at = Utc::now();
        let updated_at = created_at - chrono::Duration::seconds(1);
        let mut metadata = ContextMetadata::new(created_at);
        metadata.touch(updated_at);

        assert_eq!(
            metadata.validate(),
            Err(DomainValidationError::MetadataTimestampsOutOfOrder {
                created_at,
                updated_at,
            })
        );
    }
}
