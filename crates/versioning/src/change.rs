//! Replayable Context changes.

use contextlab_context_core::{
    ComponentId, ContentHash, ContextComponentKind, ContextMetadata, DomainValidationError,
    NonEmptyString,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Schema version for the typed Context metadata change payload.
pub const CONTEXT_METADATA_PAYLOAD_SCHEMA_VERSION: u16 = 1;

/// Typed, versioned Context metadata carried by an `UpdatedMetadata` change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextMetadataPayload {
    schema_version: u16,
    metadata: ContextMetadata,
}

impl ContextMetadataPayload {
    /// Wraps Context metadata in the current replay payload schema.
    #[must_use]
    pub fn new(metadata: ContextMetadata) -> Self {
        Self {
            schema_version: CONTEXT_METADATA_PAYLOAD_SCHEMA_VERSION,
            metadata,
        }
    }

    /// Returns the payload schema version.
    #[must_use]
    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    /// Returns the Context metadata.
    #[must_use]
    pub const fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }

    fn validate(&self) -> Result<(), DomainValidationError> {
        if self.schema_version != CONTEXT_METADATA_PAYLOAD_SCHEMA_VERSION {
            return Err(DomainValidationError::InvalidChangePayload {
                change: "updated_metadata",
            });
        }
        self.metadata.validate()?;
        Ok(())
    }
}

/// The semantic category of a Context change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextChangeKind {
    /// The Context aggregate was created.
    CreatedContext,
    /// A component was added.
    AddedComponent,
    /// A component content hash changed.
    UpdatedComponent,
    /// A component display name or metadata changed without changing its body.
    UpdatedComponentDescriptor,
    /// A component was removed.
    RemovedComponent,
    /// A directed `Uses` relationship was added between two components.
    AddedUsesRelationship,
    /// A directed `Uses` relationship was removed between two components.
    RemovedUsesRelationship,
    /// Context metadata changed.
    UpdatedMetadata,
}

/// A replayable change in a Context commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContextChange {
    kind: ContextChangeKind,
    component_id: Option<ComponentId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_component_id: Option<ComponentId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_component_id: Option<ComponentId>,
    component_kind: Option<ContextComponentKind>,
    component_name: Option<NonEmptyString>,
    component_metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_metadata: Option<ContextMetadataPayload>,
    previous_content_hash: Option<ContentHash>,
    resulting_content_hash: Option<ContentHash>,
    summary: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextChangeWire {
    kind: ContextChangeKind,
    component_id: Option<ComponentId>,
    #[serde(default)]
    source_component_id: Option<ComponentId>,
    #[serde(default)]
    target_component_id: Option<ComponentId>,
    component_kind: Option<ContextComponentKind>,
    component_name: Option<NonEmptyString>,
    component_metadata: Option<Value>,
    #[serde(default)]
    context_metadata: Option<ContextMetadataPayload>,
    previous_content_hash: Option<ContentHash>,
    resulting_content_hash: Option<ContentHash>,
    summary: String,
}

impl TryFrom<ContextChangeWire> for ContextChange {
    type Error = DomainValidationError;

    fn try_from(value: ContextChangeWire) -> Result<Self, Self::Error> {
        let change = Self {
            kind: value.kind,
            component_id: value.component_id,
            source_component_id: value.source_component_id,
            target_component_id: value.target_component_id,
            component_kind: value.component_kind,
            component_name: value.component_name,
            component_metadata: value.component_metadata,
            context_metadata: value.context_metadata,
            previous_content_hash: value.previous_content_hash,
            resulting_content_hash: value.resulting_content_hash,
            summary: value.summary,
        };
        change.validate_payload()?;
        Ok(change)
    }
}

impl<'de> Deserialize<'de> for ContextChange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        ContextChangeWire::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl ContextChange {
    /// Records creation of a new Context.
    #[must_use]
    pub fn created_context(summary: impl Into<String>) -> Self {
        Self {
            kind: ContextChangeKind::CreatedContext,
            component_id: None,
            source_component_id: None,
            target_component_id: None,
            component_kind: None,
            component_name: None,
            component_metadata: None,
            context_metadata: None,
            previous_content_hash: None,
            resulting_content_hash: None,
            summary: summary.into(),
        }
    }

    /// Records an added component.
    #[must_use]
    pub fn added_component(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            kind: ContextChangeKind::AddedComponent,
            component_id: Some(component_id),
            source_component_id: None,
            target_component_id: None,
            component_kind: Some(component_kind),
            component_name: None,
            component_metadata: None,
            context_metadata: None,
            previous_content_hash: None,
            resulting_content_hash: None,
            summary: summary.into(),
        }
    }

    /// Records a newly created component with replayable name, metadata, and initial body hash.
    pub fn added_component_content_with_details(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        component_name: impl Into<String>,
        component_metadata: Value,
        resulting_content_hash: ContentHash,
        summary: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        Ok(Self {
            kind: ContextChangeKind::AddedComponent,
            component_id: Some(component_id),
            source_component_id: None,
            target_component_id: None,
            component_kind: Some(component_kind),
            component_name: Some(NonEmptyString::new("component.name", component_name)?),
            component_metadata: Some(component_metadata),
            context_metadata: None,
            previous_content_hash: None,
            resulting_content_hash: Some(resulting_content_hash),
            summary: summary.into(),
        })
    }

    /// Records an existing component body update with replayable hash references.
    #[must_use]
    pub fn updated_component_content(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        previous_content_hash: ContentHash,
        resulting_content_hash: ContentHash,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            kind: ContextChangeKind::UpdatedComponent,
            component_id: Some(component_id),
            source_component_id: None,
            target_component_id: None,
            component_kind: Some(component_kind),
            component_name: None,
            component_metadata: None,
            context_metadata: None,
            previous_content_hash: Some(previous_content_hash),
            resulting_content_hash: Some(resulting_content_hash),
            summary: summary.into(),
        }
    }

    /// Records an existing component descriptor revision without a body-hash transition.
    pub fn updated_component_descriptor(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        component_name: impl Into<String>,
        component_metadata: Value,
        summary: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        Ok(Self {
            kind: ContextChangeKind::UpdatedComponentDescriptor,
            component_id: Some(component_id),
            source_component_id: None,
            target_component_id: None,
            component_kind: Some(component_kind),
            component_name: Some(NonEmptyString::new("component.name", component_name)?),
            component_metadata: Some(component_metadata),
            context_metadata: None,
            previous_content_hash: None,
            resulting_content_hash: None,
            summary: summary.into(),
        })
    }

    /// Records a complete, typed Context metadata replacement.
    #[must_use]
    pub fn updated_metadata(metadata: ContextMetadata, summary: impl Into<String>) -> Self {
        Self {
            kind: ContextChangeKind::UpdatedMetadata,
            component_id: None,
            source_component_id: None,
            target_component_id: None,
            component_kind: None,
            component_name: None,
            component_metadata: None,
            context_metadata: Some(ContextMetadataPayload::new(metadata)),
            previous_content_hash: None,
            resulting_content_hash: None,
            summary: summary.into(),
        }
    }

    /// Records removal of an existing component with its final replayable hash precondition.
    #[must_use]
    pub fn removed_component(
        component_id: ComponentId,
        component_kind: ContextComponentKind,
        previous_content_hash: ContentHash,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            kind: ContextChangeKind::RemovedComponent,
            component_id: Some(component_id),
            source_component_id: None,
            target_component_id: None,
            component_kind: Some(component_kind),
            component_name: None,
            component_metadata: None,
            context_metadata: None,
            previous_content_hash: Some(previous_content_hash),
            resulting_content_hash: None,
            summary: summary.into(),
        }
    }

    /// Records addition of a directed `Uses` relationship between components.
    pub fn added_uses_relationship(
        source_component_id: ComponentId,
        target_component_id: ComponentId,
        summary: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        Self::uses_relationship(
            ContextChangeKind::AddedUsesRelationship,
            source_component_id,
            target_component_id,
            summary,
        )
    }

    /// Records removal of a directed `Uses` relationship between components.
    pub fn removed_uses_relationship(
        source_component_id: ComponentId,
        target_component_id: ComponentId,
        summary: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        Self::uses_relationship(
            ContextChangeKind::RemovedUsesRelationship,
            source_component_id,
            target_component_id,
            summary,
        )
    }

    fn uses_relationship(
        kind: ContextChangeKind,
        source_component_id: ComponentId,
        target_component_id: ComponentId,
        summary: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        let change = Self {
            kind,
            component_id: None,
            source_component_id: Some(source_component_id),
            target_component_id: Some(target_component_id),
            component_kind: None,
            component_name: None,
            component_metadata: None,
            context_metadata: None,
            previous_content_hash: None,
            resulting_content_hash: None,
            summary: summary.into(),
        };
        change.validate_payload()?;
        Ok(change)
    }

    fn validate_payload(&self) -> Result<(), DomainValidationError> {
        self.validate_relationship_endpoints()?;
        if self.kind == ContextChangeKind::UpdatedMetadata {
            if self.component_id.is_some()
                || self.component_kind.is_some()
                || self.component_name.is_some()
                || self.component_metadata.is_some()
                || self.previous_content_hash.is_some()
                || self.resulting_content_hash.is_some()
            {
                return Err(DomainValidationError::InvalidChangePayload {
                    change: "updated_metadata",
                });
            }
            self.context_metadata
                .as_ref()
                .ok_or(DomainValidationError::InvalidChangePayload {
                    change: "updated_metadata",
                })?
                .validate()?;
        } else if self.context_metadata.is_some() {
            return Err(DomainValidationError::InvalidChangePayload {
                change: change_kind_name(self.kind),
            });
        }
        Ok(())
    }

    fn validate_relationship_endpoints(&self) -> Result<(), DomainValidationError> {
        let is_uses_relationship = matches!(
            self.kind,
            ContextChangeKind::AddedUsesRelationship | ContextChangeKind::RemovedUsesRelationship
        );
        let has_relationship_endpoints =
            self.source_component_id.is_some() || self.target_component_id.is_some();

        if !is_uses_relationship {
            if has_relationship_endpoints {
                return Err(DomainValidationError::InvalidChangePayload {
                    change: change_kind_name(self.kind),
                });
            }
            return Ok(());
        }

        if self.component_id.is_some()
            || self.component_kind.is_some()
            || self.component_name.is_some()
            || self.component_metadata.is_some()
            || self.context_metadata.is_some()
            || self.previous_content_hash.is_some()
            || self.resulting_content_hash.is_some()
        {
            return Err(DomainValidationError::InvalidChangePayload { change: "uses" });
        }

        let source_component_id = self.source_component_id.ok_or(
            DomainValidationError::RelationshipEndpointRequired {
                relationship: "uses",
            },
        )?;
        let target_component_id = self.target_component_id.ok_or(
            DomainValidationError::RelationshipEndpointRequired {
                relationship: "uses",
            },
        )?;

        if source_component_id == target_component_id {
            return Err(DomainValidationError::RelationshipEndpointsMustDiffer {
                relationship: "uses",
            });
        }

        Ok(())
    }

    /// Returns the change kind.
    #[must_use]
    pub const fn kind(&self) -> ContextChangeKind {
        self.kind
    }

    /// Returns the optional component identifier.
    #[must_use]
    pub const fn component_id(&self) -> Option<ComponentId> {
        self.component_id
    }

    /// Returns the source endpoint of a typed relationship change.
    #[must_use]
    pub const fn source_component_id(&self) -> Option<ComponentId> {
        self.source_component_id
    }

    /// Returns the target endpoint of a typed relationship change.
    #[must_use]
    pub const fn target_component_id(&self) -> Option<ComponentId> {
        self.target_component_id
    }

    /// Returns the optional component kind.
    #[must_use]
    pub const fn component_kind(&self) -> Option<ContextComponentKind> {
        self.component_kind
    }

    /// Returns the optional name carried by a replayable component creation.
    #[must_use]
    pub const fn component_name(&self) -> Option<&NonEmptyString> {
        self.component_name.as_ref()
    }

    /// Returns optional flexible metadata carried by a replayable component creation.
    #[must_use]
    pub const fn component_metadata(&self) -> Option<&Value> {
        self.component_metadata.as_ref()
    }

    /// Returns typed Context metadata carried by an `UpdatedMetadata` change.
    #[must_use]
    pub const fn context_metadata(&self) -> Option<&ContextMetadataPayload> {
        self.context_metadata.as_ref()
    }

    /// Returns the body hash before an updated component change.
    #[must_use]
    pub const fn previous_content_hash(&self) -> Option<&ContentHash> {
        self.previous_content_hash.as_ref()
    }

    /// Returns the body hash after an updated component change.
    #[must_use]
    pub const fn resulting_content_hash(&self) -> Option<&ContentHash> {
        self.resulting_content_hash.as_ref()
    }

    /// Returns the human-readable summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }
}

const fn change_kind_name(kind: ContextChangeKind) -> &'static str {
    match kind {
        ContextChangeKind::CreatedContext => "created_context",
        ContextChangeKind::AddedComponent => "added_component",
        ContextChangeKind::UpdatedComponent => "updated_component",
        ContextChangeKind::UpdatedComponentDescriptor => "updated_component_descriptor",
        ContextChangeKind::RemovedComponent => "removed_component",
        ContextChangeKind::AddedUsesRelationship | ContextChangeKind::RemovedUsesRelationship => {
            "uses"
        }
        ContextChangeKind::UpdatedMetadata => "updated_metadata",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use contextlab_context_core::ContentHash;

    #[test]
    fn records_component_content_hash_transition() {
        let component_id = ComponentId::new();
        let previous_content_hash = ContentHash::new("sha256:previous").expect("previous hash");
        let resulting_content_hash = ContentHash::new("sha256:resulting").expect("resulting hash");

        let change = ContextChange::updated_component_content(
            component_id,
            ContextComponentKind::SystemPrompt,
            previous_content_hash.clone(),
            resulting_content_hash.clone(),
            "Update response policy",
        );

        assert_eq!(change.kind(), ContextChangeKind::UpdatedComponent);
        assert_eq!(change.component_id(), Some(component_id));
        assert_eq!(
            change.component_kind(),
            Some(ContextComponentKind::SystemPrompt)
        );
        assert_eq!(change.previous_content_hash(), Some(&previous_content_hash));
        assert_eq!(
            change.resulting_content_hash(),
            Some(&resulting_content_hash)
        );
    }

    #[test]
    fn updated_metadata_has_typed_versioned_serialization() {
        let metadata = ContextMetadata::new(Utc::now());
        let change = ContextChange::updated_metadata(metadata.clone(), "Update labels");
        let json = serde_json::to_value(&change).expect("serialize metadata change");

        assert_eq!(change.kind(), ContextChangeKind::UpdatedMetadata);
        assert_eq!(
            change.context_metadata().expect("payload").metadata(),
            &metadata
        );
        assert_eq!(
            json["context_metadata"]["schema_version"],
            CONTEXT_METADATA_PAYLOAD_SCHEMA_VERSION
        );
        assert_eq!(
            serde_json::from_value::<ContextChange>(json).expect("round trip"),
            change
        );
    }

    #[test]
    fn rejects_missing_or_malformed_metadata_payload() {
        let change = ContextChange::updated_metadata(ContextMetadata::new(Utc::now()), "Update");
        let mut missing = serde_json::to_value(&change).expect("serialize");
        missing
            .as_object_mut()
            .expect("change object")
            .remove("context_metadata");
        assert!(serde_json::from_value::<ContextChange>(missing).is_err());

        let mut malformed = serde_json::to_value(&change).expect("serialize");
        malformed["context_metadata"]["schema_version"] = serde_json::json!(99);
        assert!(serde_json::from_value::<ContextChange>(malformed).is_err());
    }

    #[test]
    fn rejects_unknown_change_fields_instead_of_dropping_schema_drift() {
        let change = ContextChange::updated_metadata(ContextMetadata::new(Utc::now()), "Update");
        let mut value = serde_json::to_value(&change).expect("serialize");
        value["unexpected"] = serde_json::json!(true);

        assert!(serde_json::from_value::<ContextChange>(value).is_err());
    }

    #[test]
    fn rejects_context_metadata_with_reversed_timestamps_during_replay_decode() {
        let change = ContextChange::updated_metadata(ContextMetadata::new(Utc::now()), "Update");
        let mut value = serde_json::to_value(&change).expect("serialize");
        let created_at = Utc::now();
        let updated_at = created_at - chrono::Duration::seconds(1);
        value["context_metadata"]["metadata"]["created_at"] =
            serde_json::to_value(created_at).expect("created timestamp");
        value["context_metadata"]["metadata"]["updated_at"] =
            serde_json::to_value(updated_at).expect("updated timestamp");

        assert!(serde_json::from_value::<ContextChange>(value).is_err());
    }

    #[test]
    fn updated_component_descriptor_records_replayable_metadata_without_body_hashes() {
        let component_id = ComponentId::new();
        let metadata = serde_json::json!({"locale": "zh-CN", "owner": "support"});

        let change = ContextChange::updated_component_descriptor(
            component_id,
            ContextComponentKind::SystemPrompt,
            "Support policy",
            metadata.clone(),
            "Correct policy descriptor",
        )
        .expect("valid descriptor revision");

        assert_eq!(change.kind(), ContextChangeKind::UpdatedComponentDescriptor);
        assert_eq!(change.component_id(), Some(component_id));
        assert_eq!(
            change.component_kind(),
            Some(ContextComponentKind::SystemPrompt)
        );
        assert_eq!(
            change.component_name().map(NonEmptyString::as_str),
            Some("Support policy")
        );
        assert_eq!(change.component_metadata(), Some(&metadata));
        assert_eq!(change.previous_content_hash(), None);
        assert_eq!(change.resulting_content_hash(), None);
    }

    #[test]
    fn records_component_removal_hash_precondition() {
        let component_id = ComponentId::new();
        let previous_content_hash = ContentHash::new("sha256:previous").expect("previous hash");

        let change = ContextChange::removed_component(
            component_id,
            ContextComponentKind::Prompt,
            previous_content_hash.clone(),
            "Remove stale prompt",
        );

        assert_eq!(change.kind(), ContextChangeKind::RemovedComponent);
        assert_eq!(change.component_id(), Some(component_id));
        assert_eq!(change.component_kind(), Some(ContextComponentKind::Prompt));
        assert_eq!(change.previous_content_hash(), Some(&previous_content_hash));
        assert_eq!(change.resulting_content_hash(), None);
        assert_eq!(change.component_name(), None);
        assert_eq!(change.component_metadata(), None);
    }

    #[test]
    fn records_initial_component_content_hash() {
        let component_id = ComponentId::new();
        let resulting_content_hash =
            ContentHash::new("sha256:initial").expect("initial content hash");

        let change = ContextChange::added_component_content_with_details(
            component_id,
            ContextComponentKind::Prompt,
            "Initial Prompt",
            serde_json::json!({"locale": "en"}),
            resulting_content_hash.clone(),
            "Create prompt",
        )
        .expect("valid replayable component change");

        assert_eq!(change.kind(), ContextChangeKind::AddedComponent);
        assert_eq!(change.component_id(), Some(component_id));
        assert_eq!(change.component_kind(), Some(ContextComponentKind::Prompt));
        assert_eq!(change.previous_content_hash(), None);
        assert_eq!(
            change.resulting_content_hash(),
            Some(&resulting_content_hash)
        );
        assert_eq!(
            change.component_name().map(NonEmptyString::as_str),
            Some("Initial Prompt")
        );
    }

    #[test]
    fn records_replayable_component_creation_details() {
        let component_id = ComponentId::new();
        let metadata = serde_json::json!({"locale": "en", "priority": 1});
        let change = ContextChange::added_component_content_with_details(
            component_id,
            ContextComponentKind::Prompt,
            "Initial Prompt",
            metadata.clone(),
            ContentHash::new("sha256:initial").expect("initial content hash"),
            "Create prompt",
        )
        .expect("valid replayable component change");

        assert_eq!(
            change.component_name().map(NonEmptyString::as_str),
            Some("Initial Prompt")
        );
        assert_eq!(change.component_metadata(), Some(&metadata));
    }

    #[test]
    fn records_added_uses_relationship_endpoints() {
        let source_component_id = ComponentId::new();
        let target_component_id = ComponentId::new();

        let change = ContextChange::added_uses_relationship(
            source_component_id,
            target_component_id,
            "Connect prompt to knowledge",
        )
        .expect("valid Uses relationship");

        assert_eq!(change.kind(), ContextChangeKind::AddedUsesRelationship);
        assert_eq!(change.source_component_id(), Some(source_component_id));
        assert_eq!(change.target_component_id(), Some(target_component_id));
        assert_eq!(change.component_id(), None);
        assert_eq!(change.component_kind(), None);
        assert_eq!(change.component_name(), None);
        assert_eq!(change.component_metadata(), None);
        assert_eq!(change.previous_content_hash(), None);
        assert_eq!(change.resulting_content_hash(), None);
    }

    #[test]
    fn rejects_self_uses_relationship() {
        let component_id = ComponentId::new();

        let error = ContextChange::removed_uses_relationship(
            component_id,
            component_id,
            "Reject self reference",
        )
        .expect_err("self relationship must fail");

        assert_eq!(
            error,
            DomainValidationError::RelationshipEndpointsMustDiffer {
                relationship: "uses",
            }
        );
    }

    #[test]
    fn preserves_legacy_change_json_shape_without_relationship_endpoints() {
        let change = ContextChange::created_context("Create Context");
        let serialized = serde_json::to_value(change).expect("serialize legacy change");

        assert!(serialized.get("source_component_id").is_none());
        assert!(serialized.get("target_component_id").is_none());
    }

    #[test]
    fn deserializes_legacy_change_json_without_relationship_endpoints() {
        let serialized = serde_json::to_value(ContextChange::created_context("Create Context"))
            .expect("serialize legacy change");
        let restored: ContextChange =
            serde_json::from_value(serialized).expect("deserialize legacy change");

        assert_eq!(restored.kind(), ContextChangeKind::CreatedContext);
        assert_eq!(restored.source_component_id(), None);
        assert_eq!(restored.target_component_id(), None);
    }

    #[test]
    fn serializes_typed_uses_relationship_endpoints_for_add_and_remove() {
        let source_component_id = ComponentId::new();
        let target_component_id = ComponentId::new();

        for change in [
            ContextChange::added_uses_relationship(
                source_component_id,
                target_component_id,
                "Add Uses relationship",
            )
            .expect("valid add"),
            ContextChange::removed_uses_relationship(
                source_component_id,
                target_component_id,
                "Remove Uses relationship",
            )
            .expect("valid remove"),
        ] {
            let serialized = serde_json::to_value(change).expect("serialize relationship change");

            assert_eq!(
                serialized["source_component_id"],
                serde_json::Value::String(source_component_id.to_string())
            );
            assert_eq!(
                serialized["target_component_id"],
                serde_json::Value::String(target_component_id.to_string())
            );
        }
    }

    #[test]
    fn rejects_deserialized_self_uses_relationship() {
        let component_id = ComponentId::new();
        let change = ContextChange::added_uses_relationship(
            component_id,
            ComponentId::new(),
            "Add Uses relationship",
        )
        .expect("valid relationship");
        let mut serialized = serde_json::to_value(change).expect("serialize relationship change");
        serialized["target_component_id"] = serde_json::Value::String(component_id.to_string());

        let error = serde_json::from_value::<ContextChange>(serialized)
            .expect_err("deserialized self relationship must fail");

        assert!(error.to_string().contains("endpoints must differ"));
    }

    #[test]
    fn rejects_deserialized_non_relationship_change_with_endpoints() {
        let mut serialized = serde_json::to_value(ContextChange::created_context("Create Context"))
            .expect("serialize legacy change");
        serialized["source_component_id"] =
            serde_json::Value::String(ComponentId::new().to_string());
        serialized["target_component_id"] =
            serde_json::Value::String(ComponentId::new().to_string());

        assert!(serde_json::from_value::<ContextChange>(serialized).is_err());
    }

    #[test]
    fn rejects_deserialized_relationship_change_with_component_fields() {
        let change = ContextChange::added_uses_relationship(
            ComponentId::new(),
            ComponentId::new(),
            "Add Uses relationship",
        )
        .expect("valid relationship");
        let mut serialized = serde_json::to_value(change).expect("serialize relationship change");
        serialized["component_id"] = serde_json::Value::String(ComponentId::new().to_string());

        assert!(serde_json::from_value::<ContextChange>(serialized).is_err());
    }

    #[test]
    fn round_trips_added_and_removed_uses_relationship_changes() {
        let source_component_id = ComponentId::new();
        let target_component_id = ComponentId::new();
        let changes = [
            ContextChange::added_uses_relationship(
                source_component_id,
                target_component_id,
                "Add Uses relationship",
            )
            .expect("valid add"),
            ContextChange::removed_uses_relationship(
                source_component_id,
                target_component_id,
                "Remove Uses relationship",
            )
            .expect("valid remove"),
        ];

        for change in changes {
            let serialized = serde_json::to_value(&change).expect("serialize relationship change");
            let restored =
                serde_json::from_value::<ContextChange>(serialized).expect("deserialize change");

            assert_eq!(restored, change);
        }
    }

    #[test]
    fn rejects_uses_relationship_json_with_missing_or_null_individual_endpoints() {
        let source_component_id = ComponentId::new();
        let target_component_id = ComponentId::new();
        let changes = [
            ContextChange::added_uses_relationship(
                source_component_id,
                target_component_id,
                "Add Uses relationship",
            )
            .expect("valid add"),
            ContextChange::removed_uses_relationship(
                source_component_id,
                target_component_id,
                "Remove Uses relationship",
            )
            .expect("valid remove"),
        ];

        for change in changes {
            for field in ["source_component_id", "target_component_id"] {
                let mut missing =
                    serde_json::to_value(&change).expect("serialize relationship change");
                missing
                    .as_object_mut()
                    .expect("relationship payload object")
                    .remove(field);
                let missing_error = serde_json::from_value::<ContextChange>(missing)
                    .expect_err("missing relationship endpoint must fail");
                assert!(
                    missing_error
                        .to_string()
                        .contains("requires both endpoints")
                );

                let mut null =
                    serde_json::to_value(&change).expect("serialize relationship change");
                null[field] = serde_json::Value::Null;
                let null_error = serde_json::from_value::<ContextChange>(null)
                    .expect_err("null relationship endpoint must fail");
                assert!(null_error.to_string().contains("requires both endpoints"));
            }
        }
    }
}
