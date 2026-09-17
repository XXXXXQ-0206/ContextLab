//! OIDC and JWKS authentication contracts.

use crate::authentication::{AuthenticationError, BearerToken, PrincipalAuthenticator};
use crate::authorization::{
    AuthenticatedPrincipal, ExternalGroupId, IdentitySourceId, PrincipalId, PrincipalIdentity,
    TrustedExternalGroups,
};
use async_trait::async_trait;
use jsonwebtoken::jwk::{AlgorithmParameters, Jwk, JwkSet, KeyAlgorithm, PublicKeyUse};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::time::{Duration, Instant};
use url::Url;

/// Validated OIDC configuration for a JWKS-backed RS256 verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OidcJwksConfig {
    jwks_url: Url,
    issuer: IdentitySourceId,
    audience: String,
    cache_ttl_seconds: u64,
    group_claim: Option<OidcGroupClaimConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OidcGroupClaimConfig {
    name: String,
    max_token_lifetime_seconds: u64,
}

impl OidcJwksConfig {
    /// Validates an HTTPS JWKS endpoint, OIDC issuer/audience, and bounded cache TTL.
    pub fn new(
        jwks_url: &str,
        issuer: &str,
        audience: &str,
        cache_ttl_seconds: u64,
    ) -> Result<Self, AuthenticationError> {
        let jwks_url =
            Url::parse(jwks_url).map_err(|_| AuthenticationError::InvalidConfiguration)?;
        let audience = audience.trim();

        if jwks_url.scheme() != "https"
            || jwks_url.host_str().is_none()
            || audience.is_empty()
            || !(1..=3600).contains(&cache_ttl_seconds)
        {
            return Err(AuthenticationError::InvalidConfiguration);
        }
        let issuer =
            IdentitySourceId::new(issuer).map_err(|_| AuthenticationError::InvalidConfiguration)?;

        Ok(Self {
            jwks_url,
            issuer,
            audience: audience.to_owned(),
            cache_ttl_seconds,
            group_claim: None,
        })
    }

    /// Enables one validated top-level group claim with a bounded token lifetime.
    pub fn with_group_claim(
        mut self,
        group_claim_name: &str,
        max_token_lifetime_seconds: u64,
    ) -> Result<Self, AuthenticationError> {
        if !is_valid_group_claim_name(group_claim_name)
            || !(1..=3600).contains(&max_token_lifetime_seconds)
        {
            return Err(AuthenticationError::InvalidConfiguration);
        }

        self.group_claim = Some(OidcGroupClaimConfig {
            name: group_claim_name.to_owned(),
            max_token_lifetime_seconds,
        });
        Ok(self)
    }

    /// Returns the validated JWKS endpoint.
    #[must_use]
    pub const fn jwks_url(&self) -> &Url {
        &self.jwks_url
    }

    /// Returns the validated issuer.
    #[must_use]
    pub fn issuer(&self) -> &str {
        self.issuer.as_str()
    }

    /// Returns the validated audience.
    #[must_use]
    pub fn audience(&self) -> &str {
        &self.audience
    }

    /// Returns the bounded cache lifetime in seconds.
    #[must_use]
    pub const fn cache_ttl_seconds(&self) -> u64 {
        self.cache_ttl_seconds
    }

    fn group_claim(&self) -> Option<&OidcGroupClaimConfig> {
        self.group_claim.as_ref()
    }
}

fn is_valid_group_claim_name(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

/// Fetches a parsed JWKS document from the configured trusted source.
#[async_trait]
pub trait JwksSource: Send + Sync {
    /// Retrieves the current JWK set for a validated HTTPS endpoint.
    async fn fetch(&self, jwks_url: &Url) -> Result<JwkSet, JwksSourceError>;
}

/// Safe errors returned by a JWKS source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum JwksSourceError {
    /// The JWKS source could not provide a usable document.
    #[error("JWKS source is unavailable")]
    Unavailable,
}

const JWKS_FETCH_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_JWKS_RESPONSE_BYTES: usize = 1024 * 1024;
const UNKNOWN_KID_REFRESH_COOLDOWN: Duration = Duration::from_secs(1);

/// HTTPS JWKS source with a bounded, redirect-free client policy.
pub struct HttpsJwksSource {
    client: reqwest::Client,
}

impl HttpsJwksSource {
    /// Builds an HTTPS-only JWKS source with a fixed timeout and no redirects.
    pub fn new() -> Result<Self, JwksSourceError> {
        let client = reqwest::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(JWKS_FETCH_TIMEOUT)
            .build()
            .map_err(|_| JwksSourceError::Unavailable)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl JwksSource for HttpsJwksSource {
    async fn fetch(&self, jwks_url: &Url) -> Result<JwkSet, JwksSourceError> {
        let mut response = self
            .client
            .get(jwks_url.clone())
            .send()
            .await
            .map_err(|_| JwksSourceError::Unavailable)?;
        if !response.status().is_success()
            || response
                .content_length()
                .is_some_and(|length| length > MAX_JWKS_RESPONSE_BYTES as u64)
        {
            return Err(JwksSourceError::Unavailable);
        }

        let mut response_body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| JwksSourceError::Unavailable)?
        {
            let Some(next_length) = response_body.len().checked_add(chunk.len()) else {
                return Err(JwksSourceError::Unavailable);
            };
            if next_length > MAX_JWKS_RESPONSE_BYTES {
                return Err(JwksSourceError::Unavailable);
            }
            response_body.extend_from_slice(&chunk);
        }

        parse_jwks_response(&response_body)
    }
}

fn parse_jwks_response(response_body: &[u8]) -> Result<JwkSet, JwksSourceError> {
    serde_json::from_slice(response_body).map_err(|_| JwksSourceError::Unavailable)
}

/// RS256 OIDC verifier backed by a bounded, refreshable JWKS cache.
pub struct OidcJwksAuthenticator<S> {
    source: S,
    config: OidcJwksConfig,
    cache: Mutex<JwksCacheState>,
}

struct CachedJwks {
    jwks: JwkSet,
    expires_at: Instant,
}

#[derive(Default)]
struct JwksCacheState {
    cached: Option<CachedJwks>,
    refresh_in_flight: bool,
    unknown_kid_refresh_not_before: Option<Instant>,
}

#[derive(Clone, Copy)]
enum JwksRefreshReason {
    CacheMiss,
    UnknownKid,
}

struct JwksRefreshLease<'a> {
    cache: &'a Mutex<JwksCacheState>,
    armed: bool,
}

impl<'a> JwksRefreshLease<'a> {
    fn new(cache: &'a Mutex<JwksCacheState>) -> Self {
        Self { cache, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for JwksRefreshLease<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }

        let mut cache = self
            .cache
            .lock()
            .expect("JWKS cache state lock remains available");
        cache.refresh_in_flight = false;
        cache.unknown_kid_refresh_not_before = Some(Instant::now() + UNKNOWN_KID_REFRESH_COOLDOWN);
    }
}

impl<S> OidcJwksAuthenticator<S> {
    /// Builds an OIDC verifier with an initially empty JWKS cache.
    #[must_use]
    pub fn new(source: S, config: OidcJwksConfig) -> Self {
        Self {
            source,
            config,
            cache: Mutex::new(JwksCacheState::default()),
        }
    }
}

impl<S> OidcJwksAuthenticator<S>
where
    S: JwksSource,
{
    async fn authenticate(
        &self,
        token: &BearerToken,
    ) -> Result<AuthenticatedPrincipal, AuthenticationError> {
        let header =
            decode_header(token.as_str()).map_err(|_| AuthenticationError::InvalidToken)?;
        if header.alg != Algorithm::RS256 {
            return Err(AuthenticationError::InvalidToken);
        }
        let kid = header
            .kid
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or(AuthenticationError::InvalidToken)?;

        let decoding_key = self.decoding_key_for_kid(kid).await?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_required_spec_claims(&["exp", "iss", "aud"]);
        validation.set_issuer(&[self.config.issuer()]);
        validation.set_audience(&[self.config.audience()]);
        validation.leeway = 0;
        validation.validate_nbf = true;
        let claims = decode::<OidcJwtClaims>(token.as_str(), &decoding_key, &validation)
            .map_err(|_| AuthenticationError::InvalidToken)?
            .claims;
        let _registered_claims = (claims.exp, claims.iss.as_deref(), &claims.aud);
        let external_groups = match self.config.group_claim() {
            Some(config) => trusted_external_groups_from_claim(&claims, config)?,
            None => TrustedExternalGroups::default(),
        };
        let subject =
            PrincipalId::new(claims.sub).map_err(|_| AuthenticationError::MissingSubject)?;

        Ok(AuthenticatedPrincipal::with_trusted_external_groups(
            PrincipalIdentity::new(self.config.issuer.clone(), subject),
            external_groups,
        ))
    }

    async fn decoding_key_for_kid(&self, kid: &str) -> Result<DecodingKey, AuthenticationError> {
        let now = Instant::now();
        let refresh_reason = {
            let mut cache = self
                .cache
                .lock()
                .expect("JWKS cache state lock remains available");
            let has_fresh_cache = cache
                .cached
                .as_ref()
                .is_some_and(|cached| now < cached.expires_at);

            let refresh_reason = if has_fresh_cache {
                let cached_decoding_key = {
                    let cached = cache
                        .cached
                        .as_ref()
                        .expect("fresh cache is present during key lookup");
                    select_rs256_jwk(&cached.jwks, kid)
                        .ok()
                        .map(DecodingKey::from_jwk)
                };
                if let Some(decoding_key) = cached_decoding_key {
                    return decoding_key.map_err(|_| AuthenticationError::InvalidToken);
                }

                if cache.refresh_in_flight
                    || cache
                        .unknown_kid_refresh_not_before
                        .is_some_and(|not_before| now < not_before)
                {
                    return Err(AuthenticationError::InvalidToken);
                }
                JwksRefreshReason::UnknownKid
            } else {
                if cache.refresh_in_flight
                    || cache
                        .unknown_kid_refresh_not_before
                        .is_some_and(|not_before| now < not_before)
                {
                    return Err(AuthenticationError::InvalidToken);
                }
                JwksRefreshReason::CacheMiss
            };
            cache.refresh_in_flight = true;
            refresh_reason
        };

        let mut refresh_lease = JwksRefreshLease::new(&self.cache);
        let refreshed_cache = self.fetch_cached_jwks().await;
        let refreshed_at = Instant::now();
        let mut cache = self
            .cache
            .lock()
            .expect("JWKS cache state lock remains available");
        cache.refresh_in_flight = false;
        cache.unknown_kid_refresh_not_before = Some(refreshed_at + UNKNOWN_KID_REFRESH_COOLDOWN);
        let refreshed_cache = match refreshed_cache {
            Ok(refreshed_cache) => refreshed_cache,
            Err(error) => {
                refresh_lease.disarm();
                return Err(error);
            }
        };
        cache.cached = Some(refreshed_cache);
        if matches!(refresh_reason, JwksRefreshReason::CacheMiss) {
            cache.unknown_kid_refresh_not_before = None;
        }
        let jwk = select_rs256_jwk(
            &cache
                .cached
                .as_ref()
                .expect("successful refresh installs a cache entry")
                .jwks,
            kid,
        )?;
        let decoding_key =
            DecodingKey::from_jwk(jwk).map_err(|_| AuthenticationError::InvalidToken);
        refresh_lease.disarm();
        decoding_key
    }

    async fn fetch_cached_jwks(&self) -> Result<CachedJwks, AuthenticationError> {
        let jwks = self
            .source
            .fetch(self.config.jwks_url())
            .await
            .map_err(|_| AuthenticationError::InvalidToken)?;
        Ok(CachedJwks {
            jwks,
            expires_at: Instant::now() + Duration::from_secs(self.config.cache_ttl_seconds()),
        })
    }
}

#[async_trait]
impl<S> PrincipalAuthenticator for OidcJwksAuthenticator<S>
where
    S: JwksSource,
{
    async fn authenticate_authorization_header(
        &self,
        authorization_header: Option<&str>,
    ) -> Result<AuthenticatedPrincipal, AuthenticationError> {
        let token = BearerToken::from_authorization_header(authorization_header)?;
        self.authenticate(&token).await
    }
}

#[derive(Debug, Deserialize)]
struct OidcJwtClaims {
    sub: String,
    exp: usize,
    #[serde(default)]
    iat: Option<usize>,
    #[serde(default)]
    iss: Option<String>,
    aud: Value,
    #[serde(flatten)]
    additional_claims: HashMap<String, Value>,
}

fn trusted_external_groups_from_claim(
    claims: &OidcJwtClaims,
    config: &OidcGroupClaimConfig,
) -> Result<TrustedExternalGroups, AuthenticationError> {
    let issued_at = claims.iat.ok_or(AuthenticationError::InvalidToken)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AuthenticationError::InvalidToken)?
        .as_secs() as usize;
    if issued_at > now
        || claims.exp <= issued_at
        || claims.exp - issued_at > config.max_token_lifetime_seconds as usize
    {
        return Err(AuthenticationError::InvalidToken);
    }

    let Some(value) = claims.additional_claims.get(&config.name) else {
        return Ok(TrustedExternalGroups::default());
    };
    let values = value.as_array().ok_or(AuthenticationError::InvalidToken)?;
    let groups = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or(AuthenticationError::InvalidToken)
                .and_then(|value| {
                    ExternalGroupId::new(value).map_err(|_| AuthenticationError::InvalidToken)
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    TrustedExternalGroups::new(groups).map_err(|_| AuthenticationError::InvalidToken)
}

fn select_rs256_jwk<'a>(jwks: &'a JwkSet, kid: &str) -> Result<&'a Jwk, AuthenticationError> {
    let matching_keys = jwks
        .keys
        .iter()
        .filter(|jwk| jwk.common.key_id.as_deref() == Some(kid))
        .collect::<Vec<_>>();

    let [jwk] = matching_keys.as_slice() else {
        return Err(AuthenticationError::InvalidToken);
    };

    if !matches!(&jwk.algorithm, AlgorithmParameters::RSA(_))
        || jwk
            .common
            .public_key_use
            .as_ref()
            .is_some_and(|key_use| key_use != &PublicKeyUse::Signature)
        || jwk
            .common
            .key_algorithm
            .as_ref()
            .is_some_and(|algorithm| algorithm != &KeyAlgorithm::RS256)
    {
        return Err(AuthenticationError::InvalidToken);
    }

    Ok(jwk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use base64::Engine;
    use jsonwebtoken::jwk::JwkSet;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use rand::rngs::OsRng;
    use rsa::RsaPrivateKey;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::traits::PublicKeyParts;
    use serde::Serialize;
    use serde_json::json;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    #[test]
    fn selects_only_one_eligible_rs256_rsa_signing_key() {
        let valid = jwks(json!({
            "kty": "RSA",
            "kid": "current",
            "use": "sig",
            "alg": "RS256",
            "n": "sXchS0Q7r6F_srnTuaR5JZxS7cY5L6UcQOaO4Kliw_0",
            "e": "AQAB"
        }));
        let duplicate = jwks(json!({
            "kty": "RSA",
            "kid": "current",
            "use": "sig",
            "alg": "RS256",
            "n": "sXchS0Q7r6F_srnTuaR5JZxS7cY5L6UcQOaO4Kliw_0",
            "e": "AQAB"
        }));
        let wrong_algorithm = jwks(json!({
            "kty": "RSA",
            "kid": "current",
            "use": "sig",
            "alg": "RS384",
            "n": "sXchS0Q7r6F_srnTuaR5JZxS7cY5L6UcQOaO4Kliw_0",
            "e": "AQAB"
        }));

        assert!(select_rs256_jwk(&valid, "current").is_ok());
        assert!(select_rs256_jwk(&duplicate, "missing").is_err());
        assert!(select_rs256_jwk(&wrong_algorithm, "current").is_err());
        assert!(
            select_rs256_jwk(
                &JwkSet {
                    keys: [valid.keys, duplicate.keys].concat(),
                },
                "current",
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn verifier_reuses_cached_keys_and_refreshes_once_for_an_unknown_kid() {
        let source = CountingJwksSource::new([
            jwks(json!({
                "kty": "RSA",
                "kid": "current",
                "use": "sig",
                "alg": "RS256",
                "n": "sXchS0Q7r6F_srnTuaR5JZxS7cY5L6UcQOaO4Kliw_0",
                "e": "AQAB"
            })),
            jwks(json!({
                "kty": "RSA",
                "kid": "rotated",
                "use": "sig",
                "alg": "RS256",
                "n": "sXchS0Q7r6F_srnTuaR5JZxS7cY5L6UcQOaO4Kliw_0",
                "e": "AQAB"
            })),
        ]);
        let authenticator = OidcJwksAuthenticator::new(
            source.clone(),
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );

        let _ = authenticator
            .authenticate_authorization_header(Some(&bearer_token("current", "RS256")))
            .await;
        assert_eq!(source.fetch_count(), 1);

        let _ = authenticator
            .authenticate_authorization_header(Some(&bearer_token("current", "RS256")))
            .await;
        assert_eq!(source.fetch_count(), 1);

        let _ = authenticator
            .authenticate_authorization_header(Some(&bearer_token("rotated", "RS256")))
            .await;
        assert_eq!(source.fetch_count(), 2);
    }

    #[tokio::test]
    async fn verifier_limits_unknown_kid_refreshes_within_one_cache_lifetime() {
        let signing_key = TestRsaKey::new("current");
        let source = CountingJwksSource::new([signing_key.jwks()]);
        let authenticator = OidcJwksAuthenticator::new(
            source.clone(),
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );
        let valid_token = signing_key.token("https://issuer.contextlab.test", "contextlab-web");
        authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {valid_token}")))
            .await
            .expect("warm JWKS cache");
        assert_eq!(source.fetch_count(), 1);

        let unknown_kid = bearer_token("rotated", "RS256");
        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&unknown_kid))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
        assert_eq!(source.fetch_count(), 2);
        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&unknown_kid))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
        assert_eq!(source.fetch_count(), 2);
    }

    #[tokio::test]
    async fn verifier_keeps_cached_known_keys_available_during_unknown_kid_refresh() {
        let signing_key = TestRsaKey::new("current");
        let source = BlockingRefreshJwksSource::new(signing_key.jwks(), signing_key.jwks());
        let authenticator = Arc::new(OidcJwksAuthenticator::new(
            source.clone(),
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        ));
        let valid_token = signing_key.token("https://issuer.contextlab.test", "contextlab-web");
        authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {valid_token}")))
            .await
            .expect("warm JWKS cache");

        let refresh_authenticator = authenticator.clone();
        let refresh_task = tokio::spawn(async move {
            refresh_authenticator
                .authenticate_authorization_header(Some(&bearer_token("rotated", "RS256")))
                .await
        });
        source.wait_until_refresh_started().await;

        let known_key_result = tokio::time::timeout(
            Duration::from_millis(100),
            authenticator.authenticate_authorization_header(Some(&format!("Bearer {valid_token}"))),
        )
        .await;

        source.release_refresh();
        let _ = refresh_task.await.expect("refresh task completed");
        assert!(matches!(known_key_result, Ok(Ok(_))));
    }

    #[tokio::test]
    async fn verifier_retries_a_failed_unknown_kid_refresh_after_its_cooldown() {
        let current_key = TestRsaKey::new("current");
        let rotated_key = TestRsaKey::new("rotated");
        let source = RecoveringJwksSource::new([
            Ok(current_key.jwks()),
            Err(JwksSourceError::Unavailable),
            Ok(rotated_key.jwks()),
        ]);
        let authenticator = OidcJwksAuthenticator::new(
            source.clone(),
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );
        let current_token = current_key.token("https://issuer.contextlab.test", "contextlab-web");
        let rotated_token = rotated_key.token("https://issuer.contextlab.test", "contextlab-web");
        authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {current_token}")))
            .await
            .expect("warm JWKS cache");

        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&format!("Bearer {rotated_token}")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
        assert_eq!(source.fetch_count(), 2);
        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&format!("Bearer {rotated_token}")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
        assert_eq!(source.fetch_count(), 2);

        tokio::time::sleep(UNKNOWN_KID_REFRESH_COOLDOWN + Duration::from_millis(50)).await;

        let principal = authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {rotated_token}")))
            .await
            .expect("retry after JWKS refresh cooldown");
        assert_eq!(principal.id().as_str(), "user:alex");
        assert_eq!(source.fetch_count(), 3);
    }

    #[tokio::test]
    async fn verifier_requires_a_valid_rs256_signature_issuer_and_audience() {
        let signing_key = TestRsaKey::new("current");
        let source = CountingJwksSource::new([signing_key.jwks()]);
        let authenticator = OidcJwksAuthenticator::new(
            source.clone(),
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );

        let valid = signing_key.token("https://issuer.contextlab.test", "contextlab-web");
        let principal = authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {valid}")))
            .await
            .expect("valid OIDC token");
        assert_eq!(principal.id().as_str(), "user:alex");

        let invalid_issuer =
            signing_key.token("https://different-issuer.contextlab.test", "contextlab-web");
        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&format!("Bearer {invalid_issuer}")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );

        let invalid_audience = signing_key.token(
            "https://issuer.contextlab.test",
            "another-contextlab-client",
        );
        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&format!("Bearer {invalid_audience}")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
        assert_eq!(source.fetch_count(), 1);
    }

    #[tokio::test]
    async fn verifier_rejects_tokens_missing_a_required_issuer_or_audience_claim() {
        let signing_key = TestRsaKey::new("current");
        let source = CountingJwksSource::new([signing_key.jwks()]);
        let authenticator = OidcJwksAuthenticator::new(
            source,
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );

        for claims in [
            json!({
                "sub": "user:alex",
                "exp": 4_100_000_000u64,
                "aud": "contextlab-web",
            }),
            json!({
                "sub": "user:alex",
                "exp": 4_100_000_000u64,
                "iss": "https://issuer.contextlab.test",
            }),
        ] {
            let token = signing_key.token_with_json_claims(claims);
            assert_eq!(
                authenticator
                    .authenticate_authorization_header(Some(&format!("Bearer {token}")))
                    .await,
                Err(AuthenticationError::InvalidToken)
            );
        }
    }

    #[tokio::test]
    async fn verifier_accepts_an_audience_array_containing_the_configured_client() {
        let signing_key = TestRsaKey::new("current");
        let source = CountingJwksSource::new([signing_key.jwks()]);
        let authenticator = OidcJwksAuthenticator::new(
            source,
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );
        let token = signing_key.token_with_json_claims(json!({
            "sub": "user:alex",
            "exp": 4_100_000_000u64,
            "iss": "https://issuer.contextlab.test",
            "aud": ["contextlab-web", "another-client"],
        }));

        let principal = authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {token}")))
            .await
            .expect("audience array containing the configured client");

        assert_eq!(principal.id().as_str(), "user:alex");
    }

    #[tokio::test]
    async fn verifier_rejects_a_token_with_a_future_not_before_claim() {
        let signing_key = TestRsaKey::new("current");
        let source = CountingJwksSource::new([signing_key.jwks()]);
        let authenticator = OidcJwksAuthenticator::new(
            source,
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );
        let token = signing_key.token_with_json_claims(json!({
            "sub": "user:alex",
            "exp": 4_100_000_000u64,
            "iss": "https://issuer.contextlab.test",
            "aud": "contextlab-web",
            "nbf": current_unix_timestamp() + 60,
        }));

        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&format!("Bearer {token}")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
    }

    #[tokio::test]
    async fn verifier_rejects_non_rs256_headers_without_fetching_keys() {
        let source = CountingJwksSource::new(std::iter::empty());
        let authenticator = OidcJwksAuthenticator::new(
            source.clone(),
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration"),
        );

        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&bearer_token("current", "HS256")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
        assert_eq!(source.fetch_count(), 0);
    }

    #[tokio::test]
    async fn verifier_rejects_when_an_expired_cache_cannot_refresh() {
        let signing_key = TestRsaKey::new("current");
        let source = ExpiringJwksSource::new(signing_key.jwks());
        let authenticator = OidcJwksAuthenticator::new(
            source.clone(),
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                1,
            )
            .expect("OIDC configuration"),
        );
        let token = signing_key.token("https://issuer.contextlab.test", "contextlab-web");

        authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {token}")))
            .await
            .expect("initial cache population");
        tokio::time::sleep(Duration::from_millis(1_100)).await;

        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&format!("Bearer {token}")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
        assert_eq!(source.fetch_count(), 2);
    }

    #[test]
    fn jwks_response_parser_accepts_a_valid_document_and_rejects_malformed_json() {
        let parsed = parse_jwks_response(
            br#"{"keys":[{"kty":"RSA","kid":"current","use":"sig","alg":"RS256","n":"sXchS0Q7r6F_srnTuaR5JZxS7cY5L6UcQOaO4Kliw_0","e":"AQAB"}]}"#,
        )
        .expect("valid JWKS document");
        assert_eq!(parsed.keys.len(), 1);
        assert!(matches!(
            parse_jwks_response(b"not-json"),
            Err(JwksSourceError::Unavailable)
        ));
    }

    fn jwks(key: serde_json::Value) -> JwkSet {
        serde_json::from_value(json!({ "keys": [key] })).expect("valid JWKS fixture")
    }

    fn bearer_token(kid: &str, algorithm: &str) -> String {
        let header = match (kid, algorithm) {
            ("current", "RS256") => "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6ImN1cnJlbnQifQ",
            ("rotated", "RS256") => "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6InJvdGF0ZWQifQ",
            ("current", "HS256") => "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiIsImtpZCI6ImN1cnJlbnQifQ",
            _ => panic!("unsupported JWT test header"),
        };

        format!("Bearer {header}.eyJzdWIiOiJ1c2VyOmFsZXgiLCJleHAiOjQxMDAwMDAwMDB9.signature")
    }

    #[derive(Clone)]
    struct CountingJwksSource {
        responses: Arc<Mutex<VecDeque<JwkSet>>>,
        fetch_count: Arc<Mutex<usize>>,
    }

    #[derive(Clone)]
    struct BlockingRefreshJwksSource {
        initial_jwks: JwkSet,
        refreshed_jwks: JwkSet,
        fetch_count: Arc<AtomicUsize>,
        refresh_started: Arc<tokio::sync::Semaphore>,
        release_refresh: Arc<tokio::sync::Semaphore>,
    }

    impl BlockingRefreshJwksSource {
        fn new(initial_jwks: JwkSet, refreshed_jwks: JwkSet) -> Self {
            Self {
                initial_jwks,
                refreshed_jwks,
                fetch_count: Arc::new(AtomicUsize::new(0)),
                refresh_started: Arc::new(tokio::sync::Semaphore::new(0)),
                release_refresh: Arc::new(tokio::sync::Semaphore::new(0)),
            }
        }

        async fn wait_until_refresh_started(&self) {
            self.refresh_started
                .acquire()
                .await
                .expect("refresh-start semaphore remains available")
                .forget();
        }

        fn release_refresh(&self) {
            self.release_refresh.add_permits(1);
        }
    }

    #[async_trait]
    impl JwksSource for BlockingRefreshJwksSource {
        async fn fetch(&self, _jwks_url: &Url) -> Result<JwkSet, JwksSourceError> {
            if self.fetch_count.fetch_add(1, Ordering::SeqCst) == 0 {
                return Ok(self.initial_jwks.clone());
            }

            self.refresh_started.add_permits(1);
            self.release_refresh
                .acquire()
                .await
                .expect("release semaphore remains available")
                .forget();
            Ok(self.refreshed_jwks.clone())
        }
    }

    #[derive(Clone)]
    struct RecoveringJwksSource {
        responses: Arc<Mutex<VecDeque<Result<JwkSet, JwksSourceError>>>>,
        fetch_count: Arc<AtomicUsize>,
    }

    impl RecoveringJwksSource {
        fn new(responses: impl IntoIterator<Item = Result<JwkSet, JwksSourceError>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into_iter().collect())),
                fetch_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn fetch_count(&self) -> usize {
            self.fetch_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl JwksSource for RecoveringJwksSource {
        async fn fetch(&self, _jwks_url: &Url) -> Result<JwkSet, JwksSourceError> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            self.responses
                .lock()
                .expect("responses lock")
                .pop_front()
                .unwrap_or(Err(JwksSourceError::Unavailable))
        }
    }

    impl CountingJwksSource {
        fn new(responses: impl IntoIterator<Item = JwkSet>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into_iter().collect())),
                fetch_count: Arc::new(Mutex::new(0)),
            }
        }

        fn fetch_count(&self) -> usize {
            *self.fetch_count.lock().expect("fetch count lock")
        }
    }

    #[async_trait]
    impl JwksSource for CountingJwksSource {
        async fn fetch(&self, _jwks_url: &Url) -> Result<JwkSet, JwksSourceError> {
            *self.fetch_count.lock().expect("fetch count lock") += 1;
            self.responses
                .lock()
                .expect("responses lock")
                .pop_front()
                .ok_or(JwksSourceError::Unavailable)
        }
    }

    #[derive(Clone)]
    struct ExpiringJwksSource {
        initial_jwks: JwkSet,
        fetch_count: Arc<AtomicUsize>,
    }

    impl ExpiringJwksSource {
        fn new(initial_jwks: JwkSet) -> Self {
            Self {
                initial_jwks,
                fetch_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn fetch_count(&self) -> usize {
            self.fetch_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl JwksSource for ExpiringJwksSource {
        async fn fetch(&self, _jwks_url: &Url) -> Result<JwkSet, JwksSourceError> {
            if self.fetch_count.fetch_add(1, Ordering::SeqCst) == 0 {
                Ok(self.initial_jwks.clone())
            } else {
                Err(JwksSourceError::Unavailable)
            }
        }
    }

    #[tokio::test]
    async fn verifier_accepts_only_bounded_groups_from_a_valid_configured_claim() {
        let signing_key = TestRsaKey::new("current");
        let source = CountingJwksSource::new([signing_key.jwks()]);
        let authenticator = OidcJwksAuthenticator::new(
            source,
            OidcJwksConfig::new(
                "https://issuer.contextlab.test/keys",
                "https://issuer.contextlab.test",
                "contextlab-web",
                60,
            )
            .expect("OIDC configuration")
            .with_group_claim("groups", 300)
            .expect("group configuration"),
        );
        let token = signing_key.token_with_groups(
            "https://issuer.contextlab.test",
            "contextlab-web",
            current_unix_timestamp(),
            json!(["group:beta", "group:alpha", "group:beta"]),
        );

        let principal = authenticator
            .authenticate_authorization_header(Some(&format!("Bearer {token}")))
            .await
            .expect("authenticated principal");
        assert_eq!(
            principal
                .external_groups()
                .iter()
                .map(|group| group.as_str())
                .collect::<Vec<_>>(),
            vec!["group:alpha", "group:beta"]
        );

        let malformed_token = signing_key.token_with_groups(
            "https://issuer.contextlab.test",
            "contextlab-web",
            current_unix_timestamp(),
            json!("not-an-array"),
        );
        assert_eq!(
            authenticator
                .authenticate_authorization_header(Some(&format!("Bearer {malformed_token}")))
                .await,
            Err(AuthenticationError::InvalidToken)
        );
    }

    struct TestRsaKey {
        kid: String,
        private_key_der: Vec<u8>,
        modulus: String,
        exponent: String,
    }

    impl TestRsaKey {
        fn new(kid: &str) -> Self {
            let private_key =
                RsaPrivateKey::new(&mut OsRng, 2048).expect("generate an ephemeral test key");
            Self {
                kid: kid.to_owned(),
                private_key_der: private_key
                    .to_pkcs1_der()
                    .expect("encode ephemeral test key")
                    .as_bytes()
                    .to_vec(),
                modulus: base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .encode(private_key.n().to_bytes_be()),
                exponent: base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .encode(private_key.e().to_bytes_be()),
            }
        }

        fn jwks(&self) -> JwkSet {
            jwks(json!({
                "kty": "RSA",
                "kid": self.kid,
                "use": "sig",
                "alg": "RS256",
                "n": self.modulus,
                "e": self.exponent,
            }))
        }

        fn token(&self, issuer: &str, audience: &str) -> String {
            let mut header = Header::new(Algorithm::RS256);
            header.kid = Some(self.kid.clone());
            encode(
                &header,
                &TestClaims {
                    sub: "user:alex".to_owned(),
                    exp: 4_100_000_000,
                    iss: issuer.to_owned(),
                    aud: audience.to_owned(),
                },
                &EncodingKey::from_rsa_der(&self.private_key_der),
            )
            .expect("sign test token")
        }

        fn token_with_json_claims(&self, claims: serde_json::Value) -> String {
            let mut header = Header::new(Algorithm::RS256);
            header.kid = Some(self.kid.clone());
            encode(
                &header,
                &claims,
                &EncodingKey::from_rsa_der(&self.private_key_der),
            )
            .expect("sign test token")
        }

        fn token_with_groups(
            &self,
            issuer: &str,
            audience: &str,
            issued_at: usize,
            groups: serde_json::Value,
        ) -> String {
            let mut header = Header::new(Algorithm::RS256);
            header.kid = Some(self.kid.clone());
            encode(
                &header,
                &TestGroupClaims {
                    sub: "user:alex".to_owned(),
                    exp: issued_at + 200,
                    iat: issued_at,
                    iss: issuer.to_owned(),
                    aud: audience.to_owned(),
                    groups,
                },
                &EncodingKey::from_rsa_der(&self.private_key_der),
            )
            .expect("sign test token")
        }
    }

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        exp: usize,
        iss: String,
        aud: String,
    }

    #[derive(Serialize)]
    struct TestGroupClaims {
        sub: String,
        exp: usize,
        iat: usize,
        iss: String,
        aud: String,
        groups: serde_json::Value,
    }

    fn current_unix_timestamp() -> usize {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_secs() as usize
    }
}
