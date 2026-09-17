use crate::authorization::{
    AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity,
};
use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use thiserror::Error;

/// A validated bearer token extracted from an HTTP authorization header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BearerToken(String);

impl BearerToken {
    /// Parses a bearer authorization header without interpreting the token.
    pub fn from_authorization_header(value: Option<&str>) -> Result<Self, AuthenticationError> {
        let value = value.ok_or(AuthenticationError::MissingCredentials)?;
        let mut parts = value.split_whitespace();
        let scheme = parts
            .next()
            .ok_or(AuthenticationError::MissingCredentials)?;
        if !scheme.eq_ignore_ascii_case("Bearer") {
            return Err(AuthenticationError::UnsupportedScheme);
        }

        let token = parts.next().ok_or(AuthenticationError::EmptyToken)?;
        if parts.next().is_some() {
            return Err(AuthenticationError::MalformedCredentials);
        }
        if token.is_empty() {
            return Err(AuthenticationError::EmptyToken);
        }

        Ok(Self(token.to_owned()))
    }

    /// Returns the opaque token value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// HMAC JWT verifier for deployments that use a shared signing secret.
pub struct HmacJwtAuthenticator {
    decoding_key: DecodingKey,
    validation: Validation,
    identity_source: IdentitySourceId,
}

/// Replaceable authentication port for HTTP and other credential transports.
#[async_trait]
pub trait PrincipalAuthenticator: Send + Sync {
    /// Extracts and verifies an authenticated principal from an authorization header.
    async fn authenticate_authorization_header(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<AuthenticatedPrincipal, AuthenticationError>;
}

impl HmacJwtAuthenticator {
    /// Builds a verifier with required issuer and audience restrictions.
    pub fn new(
        secret: impl AsRef<str>,
        issuer: &str,
        audience: &str,
    ) -> Result<Self, AuthenticationError> {
        let secret = secret.as_ref().trim();
        if secret.is_empty() {
            return Err(AuthenticationError::InvalidConfiguration);
        }
        let identity_source =
            IdentitySourceId::new(issuer).map_err(|_| AuthenticationError::InvalidConfiguration)?;
        if audience.trim().is_empty() {
            return Err(AuthenticationError::InvalidConfiguration);
        }

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[identity_source.as_str()]);
        validation.set_audience(&[audience]);

        Ok(Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation,
            identity_source,
        })
    }

    /// Verifies a bearer token and returns its authenticated subject.
    pub fn authenticate(
        &self,
        token: &BearerToken,
    ) -> Result<AuthenticatedPrincipal, AuthenticationError> {
        let claims = decode::<JwtClaims>(token.as_str(), &self.decoding_key, &self.validation)
            .map_err(|_| AuthenticationError::InvalidToken)?
            .claims;
        let _registered_claims = (claims.exp, claims.iss, claims.aud);
        let subject =
            PrincipalId::new(claims.sub).map_err(|_| AuthenticationError::MissingSubject)?;

        Ok(AuthenticatedPrincipal::new(PrincipalIdentity::new(
            self.identity_source.clone(),
            subject,
        )))
    }
}

#[async_trait]
impl PrincipalAuthenticator for HmacJwtAuthenticator {
    async fn authenticate_authorization_header(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<AuthenticatedPrincipal, AuthenticationError> {
        let token = BearerToken::from_authorization_header(authorization_header)?;
        self.authenticate(&token)
    }
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    sub: String,
    exp: usize,
    iss: String,
    aud: String,
}

/// Errors produced while extracting an authentication credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AuthenticationError {
    /// No authorization header was supplied.
    #[error("authentication credentials are required")]
    MissingCredentials,
    /// The authorization scheme is not bearer authentication.
    #[error("authorization scheme is not supported")]
    UnsupportedScheme,
    /// The bearer scheme did not contain a token.
    #[error("bearer token is empty")]
    EmptyToken,
    /// The authorization header contained more than one token.
    #[error("authorization credentials are malformed")]
    MalformedCredentials,
    /// JWT verifier configuration is incomplete or unsafe.
    #[error("JWT authentication configuration is invalid")]
    InvalidConfiguration,
    /// The bearer token is not a valid accepted JWT.
    #[error("JWT authentication failed")]
    InvalidToken,
    /// The JWT did not contain a usable subject.
    #[error("JWT subject is missing")]
    MissingSubject,
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde::Serialize;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        exp: usize,
        iss: String,
        aud: String,
    }

    #[test]
    fn parses_a_bearer_authorization_header() {
        let token =
            BearerToken::from_authorization_header(Some("Bearer token-123")).expect("bearer token");

        assert_eq!(token.as_str(), "token-123");
    }

    #[test]
    fn rejects_missing_and_malformed_authorization_headers() {
        assert_eq!(
            BearerToken::from_authorization_header(None),
            Err(AuthenticationError::MissingCredentials)
        );
        assert_eq!(
            BearerToken::from_authorization_header(Some("Basic token-123")),
            Err(AuthenticationError::UnsupportedScheme)
        );
        assert_eq!(
            BearerToken::from_authorization_header(Some("Bearer")),
            Err(AuthenticationError::EmptyToken)
        );
    }

    #[tokio::test]
    async fn authenticates_a_signed_jwt_into_a_principal() {
        let authenticator = HmacJwtAuthenticator::new(
            "test-secret",
            "https://issuer.contextlab.test",
            "contextlab-web",
        )
        .expect("authenticator");
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestClaims {
                sub: "user:alex".to_owned(),
                exp: current_unix_timestamp() + 300,
                iss: "https://issuer.contextlab.test".to_owned(),
                aud: "contextlab-web".to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");
        let bearer = BearerToken::from_authorization_header(Some(&format!("Bearer {token}")))
            .expect("bearer");

        let principal = authenticator.authenticate(&bearer).expect("principal");
        let port_principal = authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {token}")))
            .await
            .expect("port principal");

        assert_eq!(principal.id().as_str(), "user:alex");
        assert_eq!(
            principal.identity().source().as_str(),
            "https://issuer.contextlab.test"
        );
        assert_eq!(port_principal.id().as_str(), "user:alex");
        assert!(principal.external_groups().is_empty());
        assert!(port_principal.external_groups().is_empty());
    }

    #[test]
    fn rejects_invalid_jwt_configuration_and_subjects() {
        assert!(matches!(
            HmacJwtAuthenticator::new(" ", "https://issuer.contextlab.test", "contextlab-web"),
            Err(AuthenticationError::InvalidConfiguration)
        ));
        assert!(matches!(
            HmacJwtAuthenticator::new("test-secret", " issuer ", "contextlab-web"),
            Err(AuthenticationError::InvalidConfiguration)
        ));
        assert!(matches!(
            HmacJwtAuthenticator::new("test-secret", "legacy", "contextlab-web"),
            Err(AuthenticationError::InvalidConfiguration)
        ));

        let authenticator = HmacJwtAuthenticator::new(
            "test-secret",
            "https://issuer.contextlab.test",
            "contextlab-web",
        )
        .expect("authenticator");
        let token = encode(
            &Header::new(Algorithm::HS256),
            &TestClaims {
                sub: String::new(),
                exp: current_unix_timestamp() + 300,
                iss: "https://issuer.contextlab.test".to_owned(),
                aud: "contextlab-web".to_owned(),
            },
            &EncodingKey::from_secret(b"test-secret"),
        )
        .expect("token");
        let bearer = BearerToken::from_authorization_header(Some(&format!("Bearer {token}")))
            .expect("bearer");

        assert_eq!(
            authenticator.authenticate(&bearer),
            Err(AuthenticationError::MissingSubject)
        );
    }

    fn current_unix_timestamp() -> usize {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_secs() as usize
    }
}
