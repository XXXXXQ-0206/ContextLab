//! Framework-independent authorization contracts.

use async_trait::async_trait;
use contextlab_context_core::{ContextId, WorkspaceId};
use std::collections::BTreeSet;
use std::fmt;
use thiserror::Error;

const MAX_IDENTITY_SOURCE_BYTES: usize = 2_048;
const MAX_PRINCIPAL_ID_BYTES: usize = 512;
const MAX_EXTERNAL_GROUP_ID_BYTES: usize = 512;
const MAX_TRUSTED_EXTERNAL_GROUPS: usize = 128;
const MAX_TRUSTED_EXTERNAL_GROUP_BYTES: usize = 16 * 1024;
const LEGACY_MIGRATION_IDENTITY_SOURCE: &str = "legacy";

/// A stable, issuer-scoped identity source supplied by an authenticated transport.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentitySourceId(String);

impl IdentitySourceId {
    /// Validates an opaque identity source without normalizing it.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentitySourceIdError> {
        let value = value.into();
        validate_opaque_identifier(&value, MAX_IDENTITY_SOURCE_BYTES)
            .map_err(IdentitySourceIdError::from)?;
        if value == LEGACY_MIGRATION_IDENTITY_SOURCE {
            return Err(IdentitySourceIdError::ReservedMigrationSource);
        }
        Ok(Self(value))
    }

    /// Returns the exact validated identity source.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdentitySourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Identity-source identifier validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum IdentitySourceIdError {
    /// The value was empty.
    #[error("identity source must not be empty")]
    Empty,
    /// The value contained leading or trailing whitespace.
    #[error("identity source must not contain surrounding whitespace")]
    SurroundingWhitespace,
    /// The value contained a Unicode control character.
    #[error("identity source must not contain control characters")]
    ControlCharacter,
    /// The value exceeded the supported size bound.
    #[error("identity source exceeds the supported size bound")]
    TooLong,
    /// The migration-only legacy source cannot authenticate a principal.
    #[error("identity source is reserved for legacy migration records")]
    ReservedMigrationSource,
}

/// A stable, non-empty identity subject supplied by an authenticated transport.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrincipalId(String);

impl PrincipalId {
    /// Validates an opaque principal identity subject without normalizing it.
    pub fn new(value: impl Into<String>) -> Result<Self, PrincipalIdError> {
        let value = value.into();
        validate_opaque_identifier(&value, MAX_PRINCIPAL_ID_BYTES)
            .map_err(PrincipalIdError::from)?;
        Ok(Self(value))
    }

    /// Returns the validated subject value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PrincipalId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Principal identifier validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PrincipalIdError {
    /// The identity subject was empty.
    #[error("principal id must not be empty")]
    Empty,
    /// The identity subject contained leading or trailing whitespace.
    #[error("principal id must not contain surrounding whitespace")]
    SurroundingWhitespace,
    /// The identity subject contained a Unicode control character.
    #[error("principal id must not contain control characters")]
    ControlCharacter,
    /// The identity subject exceeded the supported size bound.
    #[error("principal id exceeds the supported size bound")]
    TooLong,
}

/// An issuer-scoped stable security identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrincipalIdentity {
    source: IdentitySourceId,
    subject: PrincipalId,
}

impl PrincipalIdentity {
    /// Builds an identity from exact validated source and subject values.
    #[must_use]
    pub const fn new(source: IdentitySourceId, subject: PrincipalId) -> Self {
        Self { source, subject }
    }

    /// Returns the authenticated identity source.
    #[must_use]
    pub const fn source(&self) -> &IdentitySourceId {
        &self.source
    }

    /// Returns the exact authenticated subject.
    #[must_use]
    pub const fn subject(&self) -> &PrincipalId {
        &self.subject
    }
}

/// An authenticated actor accepted by an authentication transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedPrincipal {
    identity: PrincipalIdentity,
    external_groups: TrustedExternalGroups,
}

impl AuthenticatedPrincipal {
    /// Builds an authenticated principal from its issuer-scoped identity.
    #[must_use]
    pub fn new(identity: PrincipalIdentity) -> Self {
        Self {
            identity,
            external_groups: TrustedExternalGroups::default(),
        }
    }

    /// Builds an authenticated principal with verified external group context.
    #[must_use]
    pub const fn with_trusted_external_groups(
        identity: PrincipalIdentity,
        external_groups: TrustedExternalGroups,
    ) -> Self {
        Self {
            identity,
            external_groups,
        }
    }

    /// Returns the principal's issuer-scoped identity.
    #[must_use]
    pub const fn identity(&self) -> &PrincipalIdentity {
        &self.identity
    }

    /// Returns the principal subject.
    #[must_use]
    pub const fn id(&self) -> &PrincipalId {
        self.identity.subject()
    }

    /// Returns verified external group identifiers without deriving a role.
    #[must_use]
    pub const fn external_groups(&self) -> &TrustedExternalGroups {
        &self.external_groups
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpaqueIdentifierError {
    Empty,
    SurroundingWhitespace,
    ControlCharacter,
    TooLong,
}

fn validate_opaque_identifier(
    value: &str,
    maximum_bytes: usize,
) -> Result<(), OpaqueIdentifierError> {
    if value.is_empty() {
        return Err(OpaqueIdentifierError::Empty);
    }
    if value.trim() != value {
        return Err(OpaqueIdentifierError::SurroundingWhitespace);
    }
    if value.chars().any(char::is_control) {
        return Err(OpaqueIdentifierError::ControlCharacter);
    }
    if value.len() > maximum_bytes {
        return Err(OpaqueIdentifierError::TooLong);
    }
    Ok(())
}

impl From<OpaqueIdentifierError> for IdentitySourceIdError {
    fn from(error: OpaqueIdentifierError) -> Self {
        match error {
            OpaqueIdentifierError::Empty => Self::Empty,
            OpaqueIdentifierError::SurroundingWhitespace => Self::SurroundingWhitespace,
            OpaqueIdentifierError::ControlCharacter => Self::ControlCharacter,
            OpaqueIdentifierError::TooLong => Self::TooLong,
        }
    }
}

impl From<OpaqueIdentifierError> for PrincipalIdError {
    fn from(error: OpaqueIdentifierError) -> Self {
        match error {
            OpaqueIdentifierError::Empty => Self::Empty,
            OpaqueIdentifierError::SurroundingWhitespace => Self::SurroundingWhitespace,
            OpaqueIdentifierError::ControlCharacter => Self::ControlCharacter,
            OpaqueIdentifierError::TooLong => Self::TooLong,
        }
    }
}

/// A stable external group identifier accepted from a verified identity provider claim.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExternalGroupId(String);

impl ExternalGroupId {
    /// Validates an opaque external group identifier without normalizing it.
    pub fn new(value: impl Into<String>) -> Result<Self, ExternalGroupIdError> {
        let value = value.into();
        validate_opaque_identifier(&value, MAX_EXTERNAL_GROUP_ID_BYTES)
            .map_err(ExternalGroupIdError::from)?;
        Ok(Self(value))
    }

    /// Returns the exact validated external group identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExternalGroupId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// External group identifier validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExternalGroupIdError {
    /// The value was empty.
    #[error("external group id must not be empty")]
    Empty,
    /// The value contained leading or trailing whitespace.
    #[error("external group id must not contain surrounding whitespace")]
    SurroundingWhitespace,
    /// The value contained a Unicode control character.
    #[error("external group id must not contain control characters")]
    ControlCharacter,
    /// The value exceeded the supported size bound.
    #[error("external group id exceeds the supported size bound")]
    TooLong,
}

impl From<OpaqueIdentifierError> for ExternalGroupIdError {
    fn from(error: OpaqueIdentifierError) -> Self {
        match error {
            OpaqueIdentifierError::Empty => Self::Empty,
            OpaqueIdentifierError::SurroundingWhitespace => Self::SurroundingWhitespace,
            OpaqueIdentifierError::ControlCharacter => Self::ControlCharacter,
            OpaqueIdentifierError::TooLong => Self::TooLong,
        }
    }
}

/// A bounded, canonical collection of verified external group identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrustedExternalGroups(Vec<ExternalGroupId>);

impl TrustedExternalGroups {
    /// Validates, deduplicates, and sorts verified external group identifiers.
    pub fn new(
        groups: impl IntoIterator<Item = ExternalGroupId>,
    ) -> Result<Self, TrustedExternalGroupsError> {
        let groups = groups.into_iter().collect::<Vec<_>>();
        if groups.len() > MAX_TRUSTED_EXTERNAL_GROUPS {
            return Err(TrustedExternalGroupsError::TooManyGroups);
        }
        if groups
            .iter()
            .map(|group| group.as_str().len())
            .sum::<usize>()
            > MAX_TRUSTED_EXTERNAL_GROUP_BYTES
        {
            return Err(TrustedExternalGroupsError::TotalSizeExceeded);
        }

        let groups = groups.into_iter().collect::<BTreeSet<_>>();
        Ok(Self(groups.into_iter().collect()))
    }

    /// Returns the verified group identifiers in deterministic order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ExternalGroupId> {
        self.0.iter()
    }

    /// Returns whether the verified group collection is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }
}

/// Trusted external group collection validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TrustedExternalGroupsError {
    /// The verified claim contained more groups than the accepted bound.
    #[error("too many trusted external groups")]
    TooManyGroups,
    /// The verified group identifiers exceeded the accepted aggregate byte bound.
    #[error("trusted external groups exceed the supported size bound")]
    TotalSizeExceeded,
}

/// Permission evaluated against a Context resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextPermission {
    /// Read Context resources.
    Read,
    /// Append a guarded Context commit.
    Write,
}

impl ContextPermission {
    /// Returns the stable persistence representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

/// Result of evaluating a Context authorization decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizationDecision {
    /// The requested permission was granted.
    Granted,
    /// The principal was known but lacked the requested permission.
    Forbidden,
    /// Authorization state could not be read reliably.
    Unavailable,
}

impl AuthorizationDecision {
    /// Returns the stable persistence representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Granted => "granted",
            Self::Forbidden => "forbidden",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Immutable authorization decision event for audit sinks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationAuditEvent {
    principal_identity: PrincipalIdentity,
    context_id: ContextId,
    permission: ContextPermission,
    decision: AuthorizationDecision,
}

impl AuthorizationAuditEvent {
    /// Creates an authorization decision event.
    #[must_use]
    pub const fn new(
        principal_identity: PrincipalIdentity,
        context_id: ContextId,
        permission: ContextPermission,
        decision: AuthorizationDecision,
    ) -> Self {
        Self {
            principal_identity,
            context_id,
            permission,
            decision,
        }
    }

    /// Returns the issuer-scoped authenticated principal identity.
    #[must_use]
    pub const fn principal_identity(&self) -> &PrincipalIdentity {
        &self.principal_identity
    }

    /// Returns the authenticated principal subject.
    #[must_use]
    pub const fn principal_id(&self) -> &PrincipalId {
        self.principal_identity.subject()
    }

    /// Returns the Context scope.
    #[must_use]
    pub const fn context_id(&self) -> ContextId {
        self.context_id
    }

    /// Returns the requested permission.
    #[must_use]
    pub const fn permission(&self) -> ContextPermission {
        self.permission
    }

    /// Returns the authorization result.
    #[must_use]
    pub const fn decision(&self) -> AuthorizationDecision {
        self.decision
    }
}

/// Errors returned by an authorization audit sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AuthorizationAuditError {
    /// The audit sink could not durably accept the decision.
    #[error("authorization audit sink is unavailable")]
    Unavailable,
}

/// Replaceable application boundary for authorization decision audit records.
#[async_trait]
pub trait AuthorizationAuditSink: Send + Sync {
    /// Records one authorization decision without exposing credential material.
    async fn record(&self, event: AuthorizationAuditEvent) -> Result<(), AuthorizationAuditError>;
}

/// No-op audit sink used by preview and test state until production storage is configured.
#[derive(Debug, Default)]
pub struct NoopAuthorizationAuditSink;

#[async_trait]
impl AuthorizationAuditSink for NoopAuthorizationAuditSink {
    async fn record(&self, _event: AuthorizationAuditEvent) -> Result<(), AuthorizationAuditError> {
        Ok(())
    }
}

/// Workspace-level role used by an authorization adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceRole {
    /// Full workspace administration rights.
    Owner,
    /// Context editing rights without workspace administration.
    Editor,
    /// Read-only workspace access.
    Reader,
}

impl WorkspaceRole {
    /// Parses the role representation stored in PostgreSQL.
    #[must_use]
    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "owner" => Some(Self::Owner),
            "editor" => Some(Self::Editor),
            "reader" => Some(Self::Reader),
            _ => None,
        }
    }

    /// Returns whether this role grants a Context permission.
    #[must_use]
    pub const fn allows(self, permission: ContextPermission) -> bool {
        match (self, permission) {
            (Self::Owner | Self::Editor | Self::Reader, ContextPermission::Read) => true,
            (Self::Owner | Self::Editor, ContextPermission::Write) => true,
            (Self::Reader, ContextPermission::Write) => false,
        }
    }
}

/// The only workspace roles that may originate from an external group binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupWorkspaceRole {
    /// Context editing rights granted through a trusted external group.
    Editor,
    /// Read-only Context access granted through a trusted external group.
    Reader,
}

impl GroupWorkspaceRole {
    /// Parses the restricted role representation stored for an external group binding.
    #[must_use]
    pub fn from_storage(value: &str) -> Option<Self> {
        match value {
            "editor" => Some(Self::Editor),
            "reader" => Some(Self::Reader),
            _ => None,
        }
    }

    /// Returns whether this group-derived role grants a Context permission.
    #[must_use]
    pub const fn allows(self, permission: ContextPermission) -> bool {
        match (self, permission) {
            (Self::Editor | Self::Reader, ContextPermission::Read) => true,
            (Self::Editor, ContextPermission::Write) => true,
            (Self::Reader, ContextPermission::Write) => false,
        }
    }
}

/// Direct and group-derived workspace roles awaiting domain policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceRoleAssignments {
    direct_role: Option<WorkspaceRole>,
    group_roles: Vec<GroupWorkspaceRole>,
}

impl WorkspaceRoleAssignments {
    /// Builds deterministic assignments where direct membership takes precedence.
    #[must_use]
    pub fn new(
        direct_role: Option<WorkspaceRole>,
        group_roles: impl IntoIterator<Item = GroupWorkspaceRole>,
    ) -> Self {
        let mut group_roles = group_roles.into_iter().collect::<Vec<_>>();
        group_roles.sort_by_key(|role| match role {
            GroupWorkspaceRole::Editor => 0,
            GroupWorkspaceRole::Reader => 1,
        });
        group_roles.dedup();

        Self {
            direct_role,
            group_roles,
        }
    }

    /// Returns the direct managed-membership role, when present.
    #[must_use]
    pub const fn direct_role(&self) -> Option<WorkspaceRole> {
        self.direct_role
    }

    /// Returns deterministic group-derived roles.
    #[must_use]
    pub fn group_roles(&self) -> &[GroupWorkspaceRole] {
        &self.group_roles
    }

    /// Applies direct-membership precedence and then group-role permissions.
    #[must_use]
    pub fn allows(&self, permission: ContextPermission) -> bool {
        self.direct_role.map_or_else(
            || self.group_roles.iter().any(|role| role.allows(permission)),
            |role| role.allows(permission),
        )
    }
}

/// Authorization failures that are safe to expose through an API adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AuthorizationError {
    /// The principal lacks permission for the Context resource.
    #[error("context permission is forbidden")]
    Forbidden,
    /// The authorization state could not be read reliably.
    #[error("authorization state is unavailable")]
    Unavailable,
}

/// Authorizes an authenticated principal for a Context permission.
#[async_trait]
pub trait ContextAuthorizer: Send + Sync {
    /// Verifies the principal can perform the requested action on the Context.
    async fn authorize(
        &self,
        principal: &AuthenticatedPrincipal,
        context_id: ContextId,
        permission: ContextPermission,
    ) -> Result<(), AuthorizationError>;
}

/// Resolves a principal's workspace role for a Context without applying policy.
#[async_trait]
pub trait ContextRoleResolver: Send + Sync {
    /// Returns the active role, no role when membership is absent, or a safe infrastructure error.
    async fn resolve_context_role(
        &self,
        principal: &AuthenticatedPrincipal,
        context_id: ContextId,
    ) -> Result<Option<WorkspaceRole>, AuthorizationError>;

    /// Returns direct and group-derived roles without applying permission policy.
    async fn resolve_context_role_assignments(
        &self,
        principal: &AuthenticatedPrincipal,
        context_id: ContextId,
    ) -> Result<WorkspaceRoleAssignments, AuthorizationError> {
        let direct_role = self.resolve_context_role(principal, context_id).await?;
        Ok(WorkspaceRoleAssignments::new(
            direct_role,
            std::iter::empty(),
        ))
    }
}

/// Resolves a principal's direct workspace role without applying audit-review policy.
#[async_trait]
pub trait WorkspaceRoleResolver: Send + Sync {
    /// Returns the active direct role, no role when membership is absent, or a safe infrastructure error.
    async fn resolve_workspace_role(
        &self,
        principal: &AuthenticatedPrincipal,
        workspace_id: WorkspaceId,
    ) -> Result<Option<WorkspaceRole>, AuthorizationError>;
}

/// Authorizes a principal to review redacted authorization-audit evidence.
#[async_trait]
pub trait AuthorizationAuditReviewAuthorizer: Send + Sync {
    /// Verifies that the principal may review audit evidence within one workspace.
    async fn authorize_review(
        &self,
        principal: &AuthenticatedPrincipal,
        workspace_id: WorkspaceId,
    ) -> Result<(), AuthorizationError>;
}

/// Audit-review policy that permits only a direct workspace owner.
pub struct DirectOwnerAuthorizationAuditReviewAuthorizer<R> {
    role_resolver: R,
}

impl<R> DirectOwnerAuthorizationAuditReviewAuthorizer<R> {
    /// Builds an audit-review authorizer backed by an explicit workspace-role resolver.
    #[must_use]
    pub const fn new(role_resolver: R) -> Self {
        Self { role_resolver }
    }
}

#[async_trait]
impl<R> AuthorizationAuditReviewAuthorizer for DirectOwnerAuthorizationAuditReviewAuthorizer<R>
where
    R: WorkspaceRoleResolver,
{
    async fn authorize_review(
        &self,
        principal: &AuthenticatedPrincipal,
        workspace_id: WorkspaceId,
    ) -> Result<(), AuthorizationError> {
        match self
            .role_resolver
            .resolve_workspace_role(principal, workspace_id)
            .await?
        {
            Some(WorkspaceRole::Owner) => Ok(()),
            Some(WorkspaceRole::Editor | WorkspaceRole::Reader) | None => {
                Err(AuthorizationError::Forbidden)
            }
        }
    }
}

/// Reusable Context authorizer that applies the role policy after role resolution.
pub struct RoleBasedContextAuthorizer<R> {
    role_resolver: R,
}

impl<R> RoleBasedContextAuthorizer<R> {
    /// Builds an authorizer backed by an explicit Context role resolver.
    #[must_use]
    pub const fn new(role_resolver: R) -> Self {
        Self { role_resolver }
    }
}

#[async_trait]
impl<R> ContextAuthorizer for RoleBasedContextAuthorizer<R>
where
    R: ContextRoleResolver,
{
    async fn authorize(
        &self,
        principal: &AuthenticatedPrincipal,
        context_id: ContextId,
        permission: ContextPermission,
    ) -> Result<(), AuthorizationError> {
        let assignments = self
            .role_resolver
            .resolve_context_role_assignments(principal, context_id)
            .await?;

        if assignments.allows(permission) {
            Ok(())
        } else {
            Err(AuthorizationError::Forbidden)
        }
    }
}

/// Production-safe default authorization policy that denies every request.
#[derive(Debug, Default)]
pub struct DenyAllContextAuthorizer;

#[async_trait]
impl ContextAuthorizer for DenyAllContextAuthorizer {
    async fn authorize(
        &self,
        _principal: &AuthenticatedPrincipal,
        _context_id: ContextId,
        _permission: ContextPermission,
    ) -> Result<(), AuthorizationError> {
        Err(AuthorizationError::Forbidden)
    }
}
