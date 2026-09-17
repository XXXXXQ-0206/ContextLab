//! Private storage projection into the reusable versioning replay state.

use crate::{ContextCommitRecord, StorageRepositoryError};
use async_trait::async_trait;
use contextlab_context_core::ContextId;
use contextlab_versioning::{BranchName, CommitId, ContextChange, ContextCommit, ReplayState};
use uuid::Uuid;

/// Private read access to the complete descriptor/metadata/relationship state at a commit.
#[async_trait]
pub trait ContextReplayStateAtCommitRepository: Send + Sync {
    /// Reconstructs one exact normal-parent replay state from durable commit history.
    async fn get_context_replay_state_at_commit(
        &self,
        context_id: ContextId,
        commit_id: CommitId,
    ) -> Result<ReplayState, StorageRepositoryError>;
}

/// Replays ordered root-to-target records through the reusable versioning contract.
pub(crate) fn replay_state_from_records(
    context_id: ContextId,
    records: impl IntoIterator<Item = ContextCommitRecord>,
) -> Result<ReplayState, StorageRepositoryError> {
    let commits = records
        .into_iter()
        .map(context_commit_from_record)
        .collect::<Result<Vec<_>, _>>()?;
    ReplayState::from_commits(context_id, &commits)
        .map_err(|error| replay_state_conflict(error.to_string()))
}

/// Decodes one persistence record without generating a replacement commit identity.
pub(crate) fn context_commit_from_record(
    record: ContextCommitRecord,
) -> Result<ContextCommit, StorageRepositoryError> {
    let commit_id = parse_uuid(&record.id, "commit identifier")?;
    let context_id = parse_uuid(&record.context_id, "Context identifier")?;
    let parent_ids = record
        .parent_commit_ids
        .iter()
        .map(|value| parse_uuid(value, "parent commit identifier"))
        .map(|result| result.map(CommitId::from_uuid))
        .collect::<Result<Vec<_>, _>>()?;
    let changes = serde_json::from_value::<Vec<ContextChange>>(record.changes).map_err(|_| {
        replay_state_conflict("replay state encountered invalid stored commit changes")
    })?;
    let branch = BranchName::new(record.branch_name).map_err(|error| {
        replay_state_conflict(format!("replay state encountered invalid branch: {error}"))
    })?;

    ContextCommit::from_persisted(
        CommitId::from_uuid(commit_id),
        ContextId::from_uuid(context_id),
        branch,
        record.message,
        parent_ids,
        changes,
        record.authored_at,
    )
    .map_err(|error| {
        replay_state_conflict(format!("replay state encountered invalid commit: {error}"))
    })
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, StorageRepositoryError> {
    Uuid::parse_str(value)
        .map_err(|_| replay_state_conflict(format!("replay state encountered invalid {field}")))
}

fn replay_state_conflict(reason: impl Into<String>) -> StorageRepositoryError {
    StorageRepositoryError::ComponentStateReplayConflict {
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use contextlab_context_core::{
        ComponentId, ContentHash, ContextComponentKind, ContextMetadata,
    };
    use serde_json::json;

    fn record(
        id: Uuid,
        context_id: Uuid,
        parent_commit_ids: Vec<Uuid>,
        changes: Vec<ContextChange>,
    ) -> ContextCommitRecord {
        let changes = serde_json::to_value(changes).expect("changes");
        ContextCommitRecord {
            id: id.to_string(),
            context_id: context_id.to_string(),
            branch_name: "main".to_owned(),
            message: "replay".to_owned(),
            parent_commit_ids: parent_commit_ids
                .into_iter()
                .map(|value| value.to_string())
                .collect(),
            change_count: changes.as_array().expect("array").len() as u32,
            changes,
            authored_at: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn maps_persisted_records_into_the_single_replay_policy() {
        let context_uuid = Uuid::from_u128(1);
        let root_uuid = Uuid::from_u128(2);
        let child_uuid = Uuid::from_u128(3);
        let component_id = ComponentId::from_uuid(Uuid::from_u128(4));
        let metadata = ContextMetadata::new(Utc::now());
        let initial_hash = ContentHash::new("sha256:initial").expect("hash");
        let root = record(
            root_uuid,
            context_uuid,
            Vec::new(),
            vec![
                ContextChange::created_context("root"),
                ContextChange::added_component_content_with_details(
                    component_id,
                    ContextComponentKind::Prompt,
                    "Prompt",
                    json!({"locale": "en"}),
                    initial_hash,
                    "add",
                )
                .expect("component"),
            ],
        );
        let child = record(
            child_uuid,
            context_uuid,
            vec![root_uuid],
            vec![ContextChange::updated_metadata(
                metadata.clone(),
                "metadata",
            )],
        );

        let state = replay_state_from_records(ContextId::from_uuid(context_uuid), [root, child])
            .expect("replay persisted records");
        assert_eq!(state.context_id(), ContextId::from_uuid(context_uuid));
        assert_eq!(state.commit_id(), Some(CommitId::from_uuid(child_uuid)));
        assert_eq!(
            state.context_metadata().expect("metadata").metadata(),
            &metadata
        );
        assert_eq!(state.components().count(), 1);
    }

    #[test]
    fn rejects_non_uuid_persistence_identity_before_replay() {
        let record = ContextCommitRecord {
            id: "not-a-uuid".to_owned(),
            context_id: Uuid::from_u128(1).to_string(),
            branch_name: "main".to_owned(),
            message: "replay".to_owned(),
            parent_commit_ids: Vec::new(),
            changes: json!([]),
            change_count: 0,
            authored_at: Utc::now(),
            created_at: Utc::now(),
        };

        assert!(matches!(
            context_commit_from_record(record),
            Err(StorageRepositoryError::ComponentStateReplayConflict { .. })
        ));
    }
}
