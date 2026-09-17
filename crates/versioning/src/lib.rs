//! Git-like versioning primitives for ContextLab.
//!
//! The versioning crate records how Context changes become replayable commits.
//! It intentionally avoids persistence and API concerns so storage adapters can
//! evolve independently.

mod branch;
mod change;
mod commit;
mod history;
mod merge_base;
mod replay;

pub use branch::{
    BranchHeadConflict, BranchName, ExpectedBranchHead, VersioningError, normal_commit_parent,
};
pub use change::{
    CONTEXT_METADATA_PAYLOAD_SCHEMA_VERSION, ContextChange, ContextChangeKind,
    ContextMetadataPayload,
};
pub use commit::{CommitId, ContextCommit};
pub use history::{BranchHead, CommitHistory, HistoryError};
pub use merge_base::{
    CommitGraph, CommitGraphNode, CommitGraphValidationError, MergeBaseConflict, MergePlan,
    MergePlanError,
};
pub use replay::{
    REPLAY_STATE_SCHEMA_VERSION, ReplayComponentState, ReplayError, ReplayRelationship,
    ReplayState, ReplayStateSnapshotV1,
};
