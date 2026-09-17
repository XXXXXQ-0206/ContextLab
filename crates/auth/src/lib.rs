//! Authentication and authorization contracts for ContextLab.

mod authentication;
mod authorization;
mod oidc;
mod rate_limit;

pub use authentication::{
    AuthenticationError, BearerToken, HmacJwtAuthenticator, PrincipalAuthenticator,
};
pub use authorization::{
    AuthenticatedPrincipal, AuthorizationAuditError, AuthorizationAuditEvent,
    AuthorizationAuditReviewAuthorizer, AuthorizationAuditSink, AuthorizationDecision,
    AuthorizationError, ContextAuthorizer, ContextPermission, ContextRoleResolver,
    DenyAllContextAuthorizer, DirectOwnerAuthorizationAuditReviewAuthorizer, ExternalGroupId,
    ExternalGroupIdError, GroupWorkspaceRole, IdentitySourceId, IdentitySourceIdError,
    NoopAuthorizationAuditSink, PrincipalId, PrincipalIdError, PrincipalIdentity,
    RoleBasedContextAuthorizer, TrustedExternalGroups, TrustedExternalGroupsError, WorkspaceRole,
    WorkspaceRoleAssignments, WorkspaceRoleResolver,
};
pub use oidc::{
    HttpsJwksSource, JwksSource, JwksSourceError, OidcJwksAuthenticator, OidcJwksConfig,
};
pub use rate_limit::{
    InMemoryProtectedRouteRateLimiter, ProtectedRouteOperation, ProtectedRouteRateLimitKey,
    ProtectedRouteRateLimitPolicy, ProtectedRouteRateLimiter, RateLimitDecision, RateLimitError,
    RateLimitPolicyError,
};

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[test]
    fn rejects_an_empty_principal_identifier() {
        assert!(PrincipalId::new("  ").is_err());
    }

    #[test]
    fn identity_values_are_bounded_case_sensitive_and_opaque() {
        let source = IdentitySourceId::new("https://id.contextlab.test/tenant-a").expect("source");
        let subject = PrincipalId::new("User:Alex").expect("subject");
        let identity = PrincipalIdentity::new(source.clone(), subject.clone());

        assert_eq!(identity.source(), &source);
        assert_eq!(identity.subject(), &subject);
        assert_ne!(
            subject,
            PrincipalId::new("user:alex").expect("case-sensitive subject")
        );
        assert!(IdentitySourceId::new("").is_err());
        assert!(IdentitySourceId::new(" source ").is_err());
        assert!(IdentitySourceId::new("source\n").is_err());
        assert_eq!(
            IdentitySourceId::new("legacy"),
            Err(IdentitySourceIdError::ReservedMigrationSource)
        );
        assert!(PrincipalId::new(" subject ").is_err());
        assert!(PrincipalId::new("subject\n").is_err());
        assert!(PrincipalId::new("x".repeat(513)).is_err());

        let principal = AuthenticatedPrincipal::new(identity.clone());
        assert_eq!(principal.identity(), &identity);
        assert_eq!(principal.id(), identity.subject());
    }

    #[test]
    fn trusted_external_groups_are_bounded_deduplicated_and_opaque() {
        let groups = TrustedExternalGroups::new([
            ExternalGroupId::new("group:beta").expect("group"),
            ExternalGroupId::new("group:alpha").expect("group"),
            ExternalGroupId::new("group:beta").expect("duplicate group"),
        ])
        .expect("trusted groups");

        assert_eq!(
            groups
                .iter()
                .map(ExternalGroupId::as_str)
                .collect::<Vec<_>>(),
            vec!["group:alpha", "group:beta"]
        );
        assert!(ExternalGroupId::new(" group:alpha ").is_err());
        assert!(ExternalGroupId::new("group\nalpha").is_err());
        assert!(ExternalGroupId::new("x".repeat(513)).is_err());
        assert_eq!(
            TrustedExternalGroups::new(
                (0..129).map(|_| ExternalGroupId::new("group:repeat").expect("group")),
            ),
            Err(TrustedExternalGroupsError::TooManyGroups)
        );
        assert_eq!(
            TrustedExternalGroups::new((0..33).map(|index| {
                ExternalGroupId::new(format!("{index:03}{}", "x".repeat(509))).expect("group")
            }),),
            Err(TrustedExternalGroupsError::TotalSizeExceeded)
        );
    }

    #[test]
    fn trusted_external_groups_empty_check_preserves_empty_and_non_empty_semantics() {
        assert!(TrustedExternalGroups::default().is_empty());
        assert!(
            !TrustedExternalGroups::new([ExternalGroupId::new("group:alpha").expect("group"),])
                .expect("trusted groups")
                .is_empty()
        );
    }

    #[test]
    fn direct_membership_overrides_group_role_assignments() {
        let direct_reader = WorkspaceRoleAssignments::new(
            Some(WorkspaceRole::Reader),
            [GroupWorkspaceRole::Editor],
        );
        assert!(direct_reader.allows(ContextPermission::Read));
        assert!(!direct_reader.allows(ContextPermission::Write));

        let group_editor = WorkspaceRoleAssignments::new(
            None,
            [GroupWorkspaceRole::Reader, GroupWorkspaceRole::Editor],
        );
        assert!(group_editor.allows(ContextPermission::Read));
        assert!(group_editor.allows(ContextPermission::Write));
    }

    #[test]
    fn workspace_roles_apply_the_context_permission_matrix() {
        assert!(WorkspaceRole::Owner.allows(ContextPermission::Read));
        assert!(WorkspaceRole::Owner.allows(ContextPermission::Write));
        assert!(WorkspaceRole::Editor.allows(ContextPermission::Read));
        assert!(WorkspaceRole::Editor.allows(ContextPermission::Write));
        assert!(WorkspaceRole::Reader.allows(ContextPermission::Read));
        assert!(!WorkspaceRole::Reader.allows(ContextPermission::Write));
    }

    #[test]
    fn parses_persisted_workspace_roles_without_accepting_unknown_values() {
        assert_eq!(
            WorkspaceRole::from_storage("owner"),
            Some(WorkspaceRole::Owner)
        );
        assert_eq!(
            WorkspaceRole::from_storage("editor"),
            Some(WorkspaceRole::Editor)
        );
        assert_eq!(
            WorkspaceRole::from_storage("reader"),
            Some(WorkspaceRole::Reader)
        );
        assert_eq!(WorkspaceRole::from_storage("admin"), None);
    }

    #[tokio::test]
    async fn deny_first_authorizer_rejects_context_writes() {
        let principal = test_principal();
        let authorizer = DenyAllContextAuthorizer;

        let error = authorizer
            .authorize(
                &principal,
                contextlab_context_core::ContextId::new(),
                ContextPermission::Write,
            )
            .await
            .expect_err("default authorizer must deny writes");

        assert_eq!(error, AuthorizationError::Forbidden);
    }

    #[test]
    fn authorization_contract_distinguishes_forbidden_from_unavailable() {
        assert_ne!(
            AuthorizationError::Forbidden,
            AuthorizationError::Unavailable
        );
        assert_eq!(
            AuthorizationError::Unavailable.to_string(),
            "authorization state is unavailable"
        );
    }

    #[test]
    fn authorization_audit_event_preserves_scope_and_decision() {
        let identity = PrincipalIdentity::new(
            IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:alex").expect("principal"),
        );
        let context_id = contextlab_context_core::ContextId::new();
        let event = AuthorizationAuditEvent::new(
            identity.clone(),
            context_id,
            ContextPermission::Write,
            AuthorizationDecision::Granted,
        );

        assert_eq!(event.principal_identity(), &identity);
        assert_eq!(event.principal_id(), identity.subject());
        assert_eq!(event.context_id(), context_id);
        assert_eq!(event.permission(), ContextPermission::Write);
        assert_eq!(event.decision(), AuthorizationDecision::Granted);
    }

    #[test]
    fn authorization_values_have_stable_storage_representations() {
        assert_eq!(ContextPermission::Read.as_str(), "read");
        assert_eq!(ContextPermission::Write.as_str(), "write");
        assert_eq!(AuthorizationDecision::Granted.as_str(), "granted");
        assert_eq!(AuthorizationDecision::Forbidden.as_str(), "forbidden");
        assert_eq!(AuthorizationDecision::Unavailable.as_str(), "unavailable");
    }

    #[tokio::test]
    async fn role_based_authorizer_applies_workspace_roles_and_fails_closed() {
        let context_id = contextlab_context_core::ContextId::new();
        let principal = test_principal();

        let reader =
            RoleBasedContextAuthorizer::new(StaticRoleResolver::role(WorkspaceRole::Reader));
        assert!(
            reader
                .authorize(&principal, context_id, ContextPermission::Read)
                .await
                .is_ok()
        );
        assert_eq!(
            reader
                .authorize(&principal, context_id, ContextPermission::Write)
                .await,
            Err(AuthorizationError::Forbidden)
        );

        let editor =
            RoleBasedContextAuthorizer::new(StaticRoleResolver::role(WorkspaceRole::Editor));
        assert!(
            editor
                .authorize(&principal, context_id, ContextPermission::Write)
                .await
                .is_ok()
        );

        let unavailable = RoleBasedContextAuthorizer::new(StaticRoleResolver::unavailable());
        assert_eq!(
            unavailable
                .authorize(&principal, context_id, ContextPermission::Read)
                .await,
            Err(AuthorizationError::Unavailable)
        );
    }

    #[tokio::test]
    async fn role_based_authorizer_applies_direct_membership_precedence_to_group_roles() {
        let context_id = contextlab_context_core::ContextId::new();
        let principal = test_principal();

        let direct_reader = RoleBasedContextAuthorizer::new(StaticAssignmentsResolver(
            WorkspaceRoleAssignments::new(
                Some(WorkspaceRole::Reader),
                [GroupWorkspaceRole::Editor],
            ),
        ));
        assert_eq!(
            direct_reader
                .authorize(&principal, context_id, ContextPermission::Write)
                .await,
            Err(AuthorizationError::Forbidden)
        );

        let group_editor = RoleBasedContextAuthorizer::new(StaticAssignmentsResolver(
            WorkspaceRoleAssignments::new(None, [GroupWorkspaceRole::Editor]),
        ));
        assert!(
            group_editor
                .authorize(&principal, context_id, ContextPermission::Write)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn direct_owner_authorizes_authorization_audit_review_and_all_other_roles_fail_closed() {
        let principal = test_principal();
        let workspace_id = contextlab_context_core::WorkspaceId::new();

        let owner = DirectOwnerAuthorizationAuditReviewAuthorizer::new(
            StaticWorkspaceRoleResolver::role(WorkspaceRole::Owner),
        );
        assert!(
            owner
                .authorize_review(&principal, workspace_id)
                .await
                .is_ok()
        );

        for role in [WorkspaceRole::Editor, WorkspaceRole::Reader] {
            let authorizer = DirectOwnerAuthorizationAuditReviewAuthorizer::new(
                StaticWorkspaceRoleResolver::role(role),
            );
            assert_eq!(
                authorizer.authorize_review(&principal, workspace_id).await,
                Err(AuthorizationError::Forbidden)
            );
        }

        let absent = DirectOwnerAuthorizationAuditReviewAuthorizer::new(
            StaticWorkspaceRoleResolver(Ok(None)),
        );
        assert_eq!(
            absent.authorize_review(&principal, workspace_id).await,
            Err(AuthorizationError::Forbidden)
        );

        let unavailable = DirectOwnerAuthorizationAuditReviewAuthorizer::new(
            StaticWorkspaceRoleResolver(Err(AuthorizationError::Unavailable)),
        );
        assert_eq!(
            unavailable.authorize_review(&principal, workspace_id).await,
            Err(AuthorizationError::Unavailable)
        );
    }

    struct StaticRoleResolver(Result<Option<WorkspaceRole>, AuthorizationError>);

    struct StaticAssignmentsResolver(WorkspaceRoleAssignments);

    struct StaticWorkspaceRoleResolver(Result<Option<WorkspaceRole>, AuthorizationError>);

    fn test_principal() -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(PrincipalIdentity::new(
            IdentitySourceId::new("https://issuer.contextlab.test").expect("source"),
            PrincipalId::new("user:alex").expect("principal"),
        ))
    }

    impl StaticRoleResolver {
        fn role(role: WorkspaceRole) -> Self {
            Self(Ok(Some(role)))
        }

        fn unavailable() -> Self {
            Self(Err(AuthorizationError::Unavailable))
        }
    }

    impl StaticWorkspaceRoleResolver {
        fn role(role: WorkspaceRole) -> Self {
            Self(Ok(Some(role)))
        }
    }

    #[async_trait]
    impl ContextRoleResolver for StaticRoleResolver {
        async fn resolve_context_role(
            &self,
            _principal: &AuthenticatedPrincipal,
            _context_id: contextlab_context_core::ContextId,
        ) -> Result<Option<WorkspaceRole>, AuthorizationError> {
            self.0
        }
    }

    #[async_trait]
    impl ContextRoleResolver for StaticAssignmentsResolver {
        async fn resolve_context_role(
            &self,
            _principal: &AuthenticatedPrincipal,
            _context_id: contextlab_context_core::ContextId,
        ) -> Result<Option<WorkspaceRole>, AuthorizationError> {
            Ok(self.0.direct_role())
        }

        async fn resolve_context_role_assignments(
            &self,
            _principal: &AuthenticatedPrincipal,
            _context_id: contextlab_context_core::ContextId,
        ) -> Result<WorkspaceRoleAssignments, AuthorizationError> {
            Ok(self.0.clone())
        }
    }

    #[async_trait]
    impl WorkspaceRoleResolver for StaticWorkspaceRoleResolver {
        async fn resolve_workspace_role(
            &self,
            _principal: &AuthenticatedPrincipal,
            _workspace_id: contextlab_context_core::WorkspaceId,
        ) -> Result<Option<WorkspaceRole>, AuthorizationError> {
            self.0
        }
    }

    #[test]
    fn oidc_jwks_configuration_requires_https_and_bounded_values() {
        assert!(matches!(
            OidcJwksConfig::new(
                "http://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                300,
            ),
            Err(AuthenticationError::InvalidConfiguration)
        ));
        assert!(matches!(
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                300,
            )
            .and_then(|configuration| configuration.with_group_claim("not a valid claim", 300)),
            Err(AuthenticationError::InvalidConfiguration)
        ));
        assert!(matches!(
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                " ",
                "contextlab-web",
                300,
            ),
            Err(AuthenticationError::InvalidConfiguration)
        ));
        assert!(matches!(
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                0,
            ),
            Err(AuthenticationError::InvalidConfiguration)
        ));
    }
}
