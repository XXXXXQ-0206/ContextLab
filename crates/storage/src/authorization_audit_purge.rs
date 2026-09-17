//! Private authorization-audit purge execution contracts.

use crate::StorageRepositoryError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_context_core::WorkspaceId;
use uuid::Uuid;

const DEFAULT_AUDIT_PURGE_BATCH_LIMIT: u32 = 100;
const MAX_AUDIT_PURGE_BATCH_LIMIT: u32 = 1_000;

/// A bounded, workspace-scoped private audit purge command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationAuditPurgeRequest {
    workspace_id: WorkspaceId,
    policy_revision_id: Uuid,
    cutoff: DateTime<Utc>,
    batch_limit: u32,
}

impl AuthorizationAuditPurgeRequest {
    /// Creates a purge request with a database-compatible bounded batch size.
    #[must_use]
    pub fn new(
        workspace_id: WorkspaceId,
        policy_revision_id: Uuid,
        cutoff: DateTime<Utc>,
        batch_limit: Option<u32>,
    ) -> Self {
        Self {
            workspace_id,
            policy_revision_id,
            cutoff,
            batch_limit: batch_limit
                .unwrap_or(DEFAULT_AUDIT_PURGE_BATCH_LIMIT)
                .clamp(1, MAX_AUDIT_PURGE_BATCH_LIMIT),
        }
    }

    /// Returns the workspace owning the selected retention policy.
    #[must_use]
    pub const fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }

    /// Returns the immutable retention-policy revision to purge.
    #[must_use]
    pub const fn policy_revision_id(&self) -> Uuid {
        self.policy_revision_id
    }

    /// Returns the exclusive eligibility cutoff.
    #[must_use]
    pub const fn cutoff(&self) -> DateTime<Utc> {
        self.cutoff
    }

    /// Returns the bounded number of events selected in one transaction.
    #[must_use]
    pub const fn batch_limit(&self) -> u32 {
        self.batch_limit
    }
}

/// Result of one private database purge transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationAuditPurgeResult {
    manifest_id: Option<Uuid>,
    purged_event_count: u64,
}

impl AuthorizationAuditPurgeResult {
    /// Creates a purge result returned by the private database procedure.
    #[must_use]
    pub const fn new(manifest_id: Option<Uuid>, purged_event_count: u64) -> Self {
        Self {
            manifest_id,
            purged_event_count,
        }
    }

    /// Returns the immutable manifest when events were selected.
    #[must_use]
    pub const fn manifest_id(&self) -> Option<Uuid> {
        self.manifest_id
    }

    /// Returns the number of deleted authorization-audit events.
    #[must_use]
    pub const fn purged_event_count(&self) -> u64 {
        self.purged_event_count
    }
}

/// Private storage boundary for a separately authenticated purge executor.
#[async_trait]
pub trait AuthorizationAuditPurgeExecutor: Send + Sync {
    /// Purges one bounded batch through the dedicated database role.
    async fn purge_authorization_audit_events(
        &self,
        request: AuthorizationAuditPurgeRequest,
    ) -> Result<AuthorizationAuditPurgeResult, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use contextlab_context_core::WorkspaceId;
    use uuid::Uuid;

    #[test]
    fn purge_request_keeps_the_batch_within_the_private_database_bound() {
        let request = AuthorizationAuditPurgeRequest::new(
            WorkspaceId::new(),
            Uuid::new_v4(),
            Utc.with_ymd_and_hms(2026, 7, 13, 0, 0, 0)
                .single()
                .expect("timestamp"),
            Some(10_000),
        );

        assert_eq!(request.batch_limit(), 1_000);
    }
}
