//! Framework-independent Context domain model for ContextLab.
//!
//! This crate owns the language of Context Engineering: contexts, components,
//! stable identities, and validation primitives. API handlers, UI surfaces,
//! and persistence adapters should compose this crate rather than redefine
//! domain behavior.

mod component;
mod context;
mod identity;
mod validation;

pub use component::{ComponentContent, ContentHash, ContextComponent, ContextComponentKind};
pub use context::{Context, ContextMetadata};
pub use identity::{ComponentId, ContextId, ExperimentId, ProjectId, WorkspaceId};
pub use validation::{DomainValidationError, NonEmptyString};
