//! Domain validation primitives.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Validation errors produced by ContextLab domain constructors.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainValidationError {
    /// Context metadata has an update timestamp earlier than its creation timestamp.
    #[error(
        "Context metadata updated_at {updated_at} must be greater than or equal to created_at {created_at}"
    )]
    MetadataTimestampsOutOfOrder {
        /// Metadata creation timestamp.
        created_at: DateTime<Utc>,
        /// Metadata update timestamp.
        updated_at: DateTime<Utc>,
    },

    /// A metadata successor attempted to change its parent's creation timestamp.
    #[error(
        "Context metadata created_at changed from parent {parent_created_at} to successor {successor_created_at}"
    )]
    MetadataCreatedAtChanged {
        /// Creation timestamp reconstructed from the parent replay state.
        parent_created_at: DateTime<Utc>,
        /// Creation timestamp supplied by the successor.
        successor_created_at: DateTime<Utc>,
    },

    /// A user-visible name or label was empty after trimming whitespace.
    #[error("{field} must not be empty")]
    Empty {
        /// Name of the invalid field.
        field: &'static str,
    },

    /// A bounded string exceeded the domain limit.
    #[error("{field} must be at most {max} characters")]
    TooLong {
        /// Name of the invalid field.
        field: &'static str,
        /// Maximum allowed character count.
        max: usize,
    },

    /// A directed relationship requires distinct source and target endpoints.
    #[error("{relationship} relationship endpoints must differ")]
    RelationshipEndpointsMustDiffer {
        /// Stable name of the relationship being validated.
        relationship: &'static str,
    },

    /// A typed relationship payload omitted a required endpoint.
    #[error("{relationship} relationship requires both endpoints")]
    RelationshipEndpointRequired {
        /// Stable name of the relationship being validated.
        relationship: &'static str,
    },

    /// A change payload carried fields that are invalid for its semantic kind.
    #[error("{change} change payload contains incompatible fields")]
    InvalidChangePayload {
        /// Stable name of the change kind being validated.
        change: &'static str,
    },
}

/// A trimmed, non-empty string for domain names and labels.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Maximum number of Unicode scalar values accepted for a domain label.
    pub const MAX_CHARS: usize = 240;

    /// Creates a validated domain string.
    pub fn new(
        field: &'static str,
        value: impl Into<String>,
    ) -> Result<Self, DomainValidationError> {
        let trimmed = value.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(DomainValidationError::Empty { field });
        }
        if trimmed.chars().count() > Self::MAX_CHARS {
            return Err(DomainValidationError::TooLong {
                field,
                max: Self::MAX_CHARS,
            });
        }
        Ok(Self(trimmed))
    }

    /// Returns the validated string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_valid_values() {
        let value = NonEmptyString::new("name", "  Context Alpha  ").expect("valid name");

        assert_eq!(value.as_str(), "Context Alpha");
    }

    #[test]
    fn rejects_empty_values() {
        let error = NonEmptyString::new("name", "   ").expect_err("empty name must fail");

        assert_eq!(error, DomainValidationError::Empty { field: "name" });
    }
}
