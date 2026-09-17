//! Private, redacted authorization-audit review contracts.

use crate::StorageRepositoryError;
use crate::listing::normalize_per_page;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use contextlab_auth::{AuthorizationDecision, ContextPermission};
use contextlab_context_core::{ContextId, WorkspaceId};
use uuid::Uuid;

/// Redacted retention state visible to an authorized audit reviewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationAuditRetentionDisposition {
    /// The event has no eligible purge schedule.
    Hold,
    /// The event is associated with a retention policy and eligible timestamp.
    PurgeEligible,
}

impl AuthorizationAuditRetentionDisposition {
    pub(crate) fn from_database(value: &str) -> Result<Self, StorageRepositoryError> {
        match value {
            "hold" => Ok(Self::Hold),
            "purge_eligible" => Ok(Self::PurgeEligible),
            _ => Err(StorageRepositoryError::Database {
                message: "authorization audit review contains an unknown retention disposition"
                    .to_owned(),
            }),
        }
    }
}

/// Redacted authorization-audit evidence for one Context-scoped decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationAuditReviewItem {
    /// Database-recorded event timestamp.
    pub recorded_at: DateTime<Utc>,
    /// Context scope of the authorization decision.
    pub context_id: ContextId,
    /// Requested Context permission.
    pub permission: ContextPermission,
    /// Authorization decision recorded before a guarded write.
    pub decision: AuthorizationDecision,
    /// Retention state assigned to the event.
    pub retention_disposition: AuthorizationAuditRetentionDisposition,
    /// Immutable retention-policy revision when the event is eligible for purge.
    pub retention_policy_revision_id: Option<Uuid>,
}

impl AuthorizationAuditReviewItem {
    /// Creates redacted review evidence without accepting principal or request data.
    #[must_use]
    pub const fn new(
        recorded_at: DateTime<Utc>,
        context_id: ContextId,
        permission: ContextPermission,
        decision: AuthorizationDecision,
        retention_disposition: AuthorizationAuditRetentionDisposition,
        retention_policy_revision_id: Option<Uuid>,
    ) -> Self {
        Self {
            recorded_at,
            context_id,
            permission,
            decision,
            retention_disposition,
            retention_policy_revision_id,
        }
    }
}

/// Opaque continuation state for a private audit-review query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationAuditReviewCursor {
    recorded_at: DateTime<Utc>,
    audit_event_id: Uuid,
}

impl AuthorizationAuditReviewCursor {
    pub(crate) const fn from_database(recorded_at: DateTime<Utc>, audit_event_id: Uuid) -> Self {
        Self {
            recorded_at,
            audit_event_id,
        }
    }

    pub(crate) const fn recorded_at(&self) -> DateTime<Utc> {
        self.recorded_at
    }

    pub(crate) const fn audit_event_id(&self) -> Uuid {
        self.audit_event_id
    }
}

/// Input for a private, cursor-paginated audit-review query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationAuditReviewQuery {
    cursor: Option<AuthorizationAuditReviewCursor>,
    per_page: u32,
}

impl AuthorizationAuditReviewQuery {
    /// Creates a bounded review query from an optional prior result cursor.
    #[must_use]
    pub fn new(cursor: Option<AuthorizationAuditReviewCursor>, per_page: Option<u32>) -> Self {
        Self {
            cursor,
            per_page: normalize_per_page(per_page),
        }
    }

    /// Returns the opaque continuation state supplied by the prior page.
    #[must_use]
    pub const fn cursor(&self) -> Option<&AuthorizationAuditReviewCursor> {
        self.cursor.as_ref()
    }

    /// Returns the bounded private page size.
    #[must_use]
    pub const fn per_page(&self) -> u32 {
        self.per_page
    }
}

impl Default for AuthorizationAuditReviewQuery {
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// One page of redacted audit-review evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationAuditReviewPage {
    /// Redacted evidence ordered by `(recorded_at DESC, audit_event_id DESC)`.
    pub items: Vec<AuthorizationAuditReviewItem>,
    /// Opaque continuation state when another page is available.
    pub next_cursor: Option<AuthorizationAuditReviewCursor>,
}

impl AuthorizationAuditReviewPage {
    /// Creates a private review result page.
    #[must_use]
    pub const fn new(
        items: Vec<AuthorizationAuditReviewItem>,
        next_cursor: Option<AuthorizationAuditReviewCursor>,
    ) -> Self {
        Self { items, next_cursor }
    }
}

/// Storage boundary for workspace-scoped, redacted audit-review evidence.
#[async_trait]
pub trait AuthorizationAuditReviewRepository: Send + Sync {
    /// Lists review evidence only after the caller has passed the owner-only authorization boundary.
    async fn list_authorization_audit_review(
        &self,
        workspace_id: WorkspaceId,
        query: AuthorizationAuditReviewQuery,
    ) -> Result<AuthorizationAuditReviewPage, StorageRepositoryError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use contextlab_auth::{AuthorizationDecision, ContextPermission};
    use contextlab_context_core::ContextId;

    #[test]
    fn review_query_normalizes_the_private_page_size() {
        let default_query = AuthorizationAuditReviewQuery::new(None, None);
        let minimum_query = AuthorizationAuditReviewQuery::new(None, Some(0));

        assert_eq!(default_query.per_page(), crate::DEFAULT_PER_PAGE);
        assert_eq!(minimum_query.per_page(), 1);
        assert!(default_query.cursor().is_none());
    }

    #[test]
    fn review_item_contains_only_redacted_audit_evidence() {
        let context_id = ContextId::new();
        let recorded_at = Utc
            .with_ymd_and_hms(2026, 7, 13, 0, 0, 0)
            .single()
            .expect("valid timestamp");
        let item = AuthorizationAuditReviewItem::new(
            recorded_at,
            context_id,
            ContextPermission::Write,
            AuthorizationDecision::Granted,
            AuthorizationAuditRetentionDisposition::Hold,
            None,
        );

        assert_eq!(item.recorded_at, recorded_at);
        assert_eq!(item.context_id, context_id);
        assert_eq!(item.permission, ContextPermission::Write);
        assert_eq!(item.decision, AuthorizationDecision::Granted);
        assert_eq!(
            item.retention_disposition,
            AuthorizationAuditRetentionDisposition::Hold
        );
        assert_eq!(item.retention_policy_revision_id, None);
    }
}
