//! Stable identity types for ContextLab domain objects.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! domain_id {
    ($name:ident) => {
        #[doc = concat!("Stable unique identifier for a ", stringify!($name), ".")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new random identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Wraps an existing UUID.
            #[must_use]
            pub const fn from_uuid(value: Uuid) -> Self {
                Self(value)
            }

            /// Returns the underlying UUID value.
            #[must_use]
            pub const fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "{}", self.0)
            }
        }
    };
}

domain_id!(WorkspaceId);
domain_id!(ProjectId);
domain_id!(ExperimentId);
domain_id!(ContextId);
domain_id!(ComponentId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique() {
        assert_ne!(ContextId::new(), ContextId::new());
    }
}
