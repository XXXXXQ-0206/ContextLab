//! Provider configuration and model gateway primitives for ContextLab.
//!
//! This crate is the boundary between ContextLab's domain/application layers
//! and external model providers. It starts with deterministic configuration and
//! secret redaction so API and UI surfaces can reason about provider readiness
//! without exposing credentials.

use serde::Serialize;
use std::collections::BTreeMap;
use thiserror::Error;
use url::Url;

/// Supported model provider families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// DeepSeek official API.
    DeepSeek,
    /// OpenAI-compatible API endpoint.
    OpenAiCompatible,
}

impl ProviderKind {
    /// Returns a stable provider id for API and UI clients.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }

    /// Returns a human-readable display name.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::DeepSeek => "DeepSeek",
            Self::OpenAiCompatible => "OpenAI Compatible",
        }
    }

    /// Returns the environment variable that carries the API base URL.
    #[must_use]
    pub const fn base_url_env(self) -> &'static str {
        match self {
            Self::DeepSeek => "DEEPSEEK_API_BASE_URL",
            Self::OpenAiCompatible => "OPENAI_COMPAT_API_BASE_URL",
        }
    }

    /// Returns the environment variable that carries the API key.
    #[must_use]
    pub const fn api_key_env(self) -> &'static str {
        match self {
            Self::DeepSeek => "DEEPSEEK_API_KEY",
            Self::OpenAiCompatible => "OPENAI_COMPAT_API_KEY",
        }
    }
}

/// A configured provider with credential metadata redacted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderConfig {
    kind: ProviderKind,
    base_url: Url,
    credential: ProviderCredential,
}

impl ProviderConfig {
    /// Creates a provider config from a base URL and API key.
    pub fn new(
        kind: ProviderKind,
        base_url: &str,
        api_key: &str,
    ) -> Result<Self, ProviderConfigError> {
        let base_url =
            Url::parse(base_url).map_err(|source| ProviderConfigError::InvalidBaseUrl {
                provider: kind,
                source,
            })?;
        let credential = ProviderCredential::from_api_key(api_key)?;

        Ok(Self {
            kind,
            base_url,
            credential,
        })
    }

    /// Returns the provider kind.
    #[must_use]
    pub const fn kind(&self) -> ProviderKind {
        self.kind
    }

    /// Returns the configured base URL.
    #[must_use]
    pub const fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Returns redacted credential metadata.
    #[must_use]
    pub const fn credential(&self) -> &ProviderCredential {
        &self.credential
    }

    /// Converts this provider to a public status object.
    #[must_use]
    pub fn to_public_status(&self) -> PublicProviderStatus {
        PublicProviderStatus {
            id: self.kind.id(),
            display_name: self.kind.display_name(),
            configured: true,
            base_url: self.base_url.as_str().to_owned(),
            api_key_env: self.kind.api_key_env(),
            api_key_fingerprint: Some(self.credential.fingerprint().to_owned()),
        }
    }
}

/// Redacted credential metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderCredential {
    fingerprint: String,
    length: usize,
}

impl ProviderCredential {
    /// Builds a redacted credential representation from an API key.
    pub fn from_api_key(api_key: &str) -> Result<Self, ProviderConfigError> {
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            return Err(ProviderConfigError::MissingApiKey);
        }

        let prefix: String = trimmed.chars().take(3).collect();
        let suffix_rev: String = trimmed.chars().rev().take(4).collect();
        let suffix: String = suffix_rev.chars().rev().collect();
        let length = trimmed.chars().count();

        Ok(Self {
            fingerprint: format!("{prefix}...{suffix}"),
            length,
        })
    }

    /// Returns the redacted API key fingerprint.
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Returns API key character length.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.length
    }
}

/// Public provider status exposed through API surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicProviderStatus {
    /// Stable provider id.
    pub id: &'static str,
    /// Human-readable provider name.
    pub display_name: &'static str,
    /// Whether both endpoint and credential are configured.
    pub configured: bool,
    /// Configured API base URL.
    pub base_url: String,
    /// Environment variable used for the API key.
    pub api_key_env: &'static str,
    /// Redacted API key fingerprint when configured.
    pub api_key_fingerprint: Option<String>,
}

impl PublicProviderStatus {
    fn unconfigured(kind: ProviderKind, base_url: String) -> Self {
        Self {
            id: kind.id(),
            display_name: kind.display_name(),
            configured: false,
            base_url,
            api_key_env: kind.api_key_env(),
            api_key_fingerprint: None,
        }
    }
}

/// Registry of provider configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRegistry {
    providers: BTreeMap<ProviderKind, ProviderConfig>,
    defaults: BTreeMap<ProviderKind, Url>,
}

impl ProviderRegistry {
    /// Builds a registry from the process environment.
    #[must_use]
    pub fn from_current_env() -> Self {
        Self::from_env(std::env::vars())
    }

    /// Builds a registry from environment-like key/value pairs.
    #[must_use]
    pub fn from_env<I, K, V>(env: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let values: BTreeMap<String, String> = env
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        let defaults = default_base_urls();
        let mut providers = BTreeMap::new();

        for kind in [ProviderKind::DeepSeek, ProviderKind::OpenAiCompatible] {
            let default_base_url = defaults
                .get(&kind)
                .expect("every provider has a default base URL");
            let base_url = values
                .get(kind.base_url_env())
                .map(String::as_str)
                .unwrap_or(default_base_url.as_str());

            if let Some(api_key) = values.get(kind.api_key_env()) {
                if let Ok(provider) = ProviderConfig::new(kind, base_url, api_key) {
                    providers.insert(kind, provider);
                }
            }
        }

        Self {
            providers,
            defaults,
        }
    }

    /// Returns configured providers.
    #[must_use]
    pub const fn configured_providers(&self) -> &BTreeMap<ProviderKind, ProviderConfig> {
        &self.providers
    }

    /// Returns public provider statuses for all known providers.
    #[must_use]
    pub fn public_statuses(&self) -> Vec<PublicProviderStatus> {
        [ProviderKind::DeepSeek, ProviderKind::OpenAiCompatible]
            .into_iter()
            .map(|kind| {
                self.providers.get(&kind).map_or_else(
                    || {
                        let base_url = self
                            .defaults
                            .get(&kind)
                            .expect("every provider has a default base URL")
                            .as_str()
                            .to_owned();
                        PublicProviderStatus::unconfigured(kind, base_url)
                    },
                    ProviderConfig::to_public_status,
                )
            })
            .collect()
    }
}

/// Provider configuration errors.
#[derive(Debug, Error)]
pub enum ProviderConfigError {
    /// Base URL was not a valid URL.
    #[error("invalid base URL for {provider:?}: {source}")]
    InvalidBaseUrl {
        /// Provider being configured.
        provider: ProviderKind,
        /// URL parser source error.
        source: url::ParseError,
    },
    /// API key was empty.
    #[error("provider API key must not be empty")]
    MissingApiKey,
}

fn default_base_urls() -> BTreeMap<ProviderKind, Url> {
    [
        (ProviderKind::DeepSeek, "https://api.deepseek.com"),
        (ProviderKind::OpenAiCompatible, "https://codex.hiyo.top"),
    ]
    .into_iter()
    .map(|(kind, url)| {
        (
            kind,
            Url::parse(url).expect("static provider default URL is valid"),
        )
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_keys_without_exposing_secret() {
        let credential =
            ProviderCredential::from_api_key("sk-1234567890abcdef").expect("valid credential");

        assert_eq!(credential.fingerprint(), "sk-...cdef");
        assert_eq!(credential.length(), 19);
        assert!(!credential.fingerprint().contains("1234567890"));
    }

    #[test]
    fn builds_registry_from_environment_pairs() {
        let registry = ProviderRegistry::from_env([
            ("DEEPSEEK_API_BASE_URL", "https://api.deepseek.com"),
            ("DEEPSEEK_API_KEY", "sk-deepseek-test-key"),
            ("OPENAI_COMPAT_API_BASE_URL", "https://codex.hiyo.top"),
            ("OPENAI_COMPAT_API_KEY", "sk-openai-compatible-key"),
        ]);

        let statuses = registry.public_statuses();

        assert_eq!(statuses.len(), 2);
        assert!(statuses.iter().all(|status| status.configured));
        assert!(statuses.iter().any(|status| status.id == "deepseek"
            && status.api_key_fingerprint == Some("sk-...-key".to_owned())));
    }

    #[test]
    fn reports_unconfigured_provider_without_secret_metadata() {
        let registry = ProviderRegistry::from_env([("DEEPSEEK_API_KEY", "sk-deepseek-test-key")]);

        let statuses = registry.public_statuses();
        let openai_compatible = statuses
            .iter()
            .find(|status| status.id == "openai_compatible")
            .expect("status");

        assert!(!openai_compatible.configured);
        assert_eq!(openai_compatible.base_url, "https://codex.hiyo.top/");
        assert_eq!(openai_compatible.api_key_fingerprint, None);
    }
}
