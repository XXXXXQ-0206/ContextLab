//! Version-bound, local-only review projection for two Context Graph snapshots.

use crate::{GraphDiff, VersionedContextGraphSnapshotV1, VersionedContextScopeV1};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_versioning::CommitId;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// The explicit schema revision for a version-bound two-way graph review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionedGraphDiffReviewSchemaVersion {
    /// The initial stable version-bound graph review schema.
    V1,
}

/// Side of a two-way version-bound graph review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionedGraphDiffReviewSide {
    /// The baseline graph snapshot.
    Source,
    /// The revised graph snapshot.
    Target,
}

/// Server-owned identity witness for one ordered pair of graph versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VersionedContextGraphDiffReviewIdentityWitnessV1 {
    project_id: ProjectId,
    context_id: ContextId,
    baseline_commit_id: CommitId,
    revised_commit_id: CommitId,
}

impl VersionedContextGraphDiffReviewIdentityWitnessV1 {
    /// Creates a witness only for two valid, distinct scopes in one Context.
    pub fn new(
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Result<Self, VersionedContextGraphDiffReviewError> {
        VersionedContextGraphDiffReviewRequestV1::validate_scopes(source_scope, target_scope)?;
        Ok(Self::from_validated_scopes(source_scope, target_scope))
    }

    fn from_validated_scopes(
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Self {
        Self {
            project_id: source_scope.project_id(),
            context_id: source_scope.context_id(),
            baseline_commit_id: source_scope.commit_id(),
            revised_commit_id: target_scope.commit_id(),
        }
    }

    /// Returns the exact project identity covered by the witness.
    #[must_use]
    pub const fn project_id(self) -> ProjectId {
        self.project_id
    }

    /// Returns the exact Context identity covered by the witness.
    #[must_use]
    pub const fn context_id(self) -> ContextId {
        self.context_id
    }

    /// Returns the ordered source, or baseline, commit identity.
    #[must_use]
    pub const fn source_commit_id(self) -> CommitId {
        self.baseline_commit_id
    }

    /// Returns the ordered source, or baseline, commit identity.
    #[must_use]
    pub const fn baseline_commit_id(self) -> CommitId {
        self.baseline_commit_id
    }

    /// Returns the ordered target, or revised, commit identity.
    #[must_use]
    pub const fn target_commit_id(self) -> CommitId {
        self.revised_commit_id
    }

    /// Returns the ordered target, or revised, commit identity.
    #[must_use]
    pub const fn revised_commit_id(self) -> CommitId {
        self.revised_commit_id
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionedContextGraphDiffReviewIdentityWitnessWireV1 {
    project_id: ProjectId,
    context_id: ContextId,
    baseline_commit_id: CommitId,
    revised_commit_id: CommitId,
}

impl<'de> Deserialize<'de> for VersionedContextGraphDiffReviewIdentityWitnessV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = VersionedContextGraphDiffReviewIdentityWitnessWireV1::deserialize(deserializer)?;
        Self::new(
            VersionedContextScopeV1::new(wire.project_id, wire.context_id, wire.baseline_commit_id),
            VersionedContextScopeV1::new(wire.project_id, wire.context_id, wire.revised_commit_id),
        )
        .map_err(serde::de::Error::custom)
    }
}

/// Complete version-bound inputs for one ordered pair of Context Graph versions.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VersionedContextGraphDiffReviewRequestV1 {
    schema_version: VersionedGraphDiffReviewSchemaVersion,
    source: VersionedContextGraphSnapshotV1,
    target: VersionedContextGraphSnapshotV1,
}

impl VersionedContextGraphDiffReviewRequestV1 {
    /// Creates a request for two distinct snapshots in one exact Context scope.
    pub fn new(
        source: VersionedContextGraphSnapshotV1,
        target: VersionedContextGraphSnapshotV1,
    ) -> Result<Self, VersionedContextGraphDiffReviewError> {
        Self::validate_scopes(source.scope(), target.scope())?;

        Ok(Self {
            schema_version: VersionedGraphDiffReviewSchemaVersion::V1,
            source,
            target,
        })
    }

    /// Validates the exact pair before any snapshot payload is read or compared.
    pub fn validate_scopes(
        source_scope: VersionedContextScopeV1,
        target_scope: VersionedContextScopeV1,
    ) -> Result<(), VersionedContextGraphDiffReviewError> {
        validate_scope(VersionedGraphDiffReviewSide::Source, source_scope)?;
        validate_scope(VersionedGraphDiffReviewSide::Target, target_scope)?;
        if source_scope.project_id() != target_scope.project_id()
            || source_scope.context_id() != target_scope.context_id()
        {
            return Err(
                VersionedContextGraphDiffReviewError::MismatchedContextScope {
                    source_scope,
                    target_scope,
                },
            );
        }
        if source_scope == target_scope {
            return Err(
                VersionedContextGraphDiffReviewError::IdenticalVersionScopes {
                    scope: source_scope,
                },
            );
        }
        Ok(())
    }

    /// Returns the explicit request schema version.
    #[must_use]
    pub const fn schema_version(&self) -> VersionedGraphDiffReviewSchemaVersion {
        self.schema_version
    }

    /// Returns the baseline snapshot.
    #[must_use]
    pub const fn source(&self) -> &VersionedContextGraphSnapshotV1 {
        &self.source
    }

    /// Returns the revised snapshot.
    #[must_use]
    pub const fn target(&self) -> &VersionedContextGraphSnapshotV1 {
        &self.target
    }

    /// Returns the baseline exact scope.
    #[must_use]
    pub const fn source_scope(&self) -> VersionedContextScopeV1 {
        self.source.scope()
    }

    /// Returns the revised exact scope.
    #[must_use]
    pub const fn target_scope(&self) -> VersionedContextScopeV1 {
        self.target.scope()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionedContextGraphDiffReviewRequestWireV1 {
    schema_version: VersionedGraphDiffReviewSchemaVersion,
    source: VersionedContextGraphSnapshotV1,
    target: VersionedContextGraphSnapshotV1,
}

impl<'de> Deserialize<'de> for VersionedContextGraphDiffReviewRequestV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = VersionedContextGraphDiffReviewRequestWireV1::deserialize(deserializer)?;
        let request = Self::new(wire.source, wire.target).map_err(serde::de::Error::custom)?;
        if wire.schema_version != request.schema_version {
            return Err(serde::de::Error::custom(
                VersionedContextGraphDiffReviewError::UnsupportedSchemaVersion {
                    schema_version: wire.schema_version,
                },
            ));
        }
        Ok(request)
    }
}

/// Complete provider-free projection for one ordered version-bound graph pair.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VersionedContextGraphDiffReviewProjectionV1 {
    schema_version: VersionedGraphDiffReviewSchemaVersion,
    identity_witness: VersionedContextGraphDiffReviewIdentityWitnessV1,
    source_scope: VersionedContextScopeV1,
    target_scope: VersionedContextScopeV1,
    diff: GraphDiff,
}

impl VersionedContextGraphDiffReviewProjectionV1 {
    /// Returns the explicit projection schema version.
    #[must_use]
    pub const fn schema_version(&self) -> VersionedGraphDiffReviewSchemaVersion {
        self.schema_version
    }

    /// Returns the server-owned identity witness for the ordered pair.
    #[must_use]
    pub const fn identity_witness(&self) -> VersionedContextGraphDiffReviewIdentityWitnessV1 {
        self.identity_witness
    }

    /// Returns the baseline exact scope.
    #[must_use]
    pub const fn source_scope(&self) -> VersionedContextScopeV1 {
        self.source_scope
    }

    /// Returns the revised exact scope.
    #[must_use]
    pub const fn target_scope(&self) -> VersionedContextScopeV1 {
        self.target_scope
    }

    /// Returns the deterministic structural graph diff.
    #[must_use]
    pub const fn diff(&self) -> &GraphDiff {
        &self.diff
    }
}

/// Fail-closed errors for a version-bound two-way graph review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionedContextGraphDiffReviewError {
    /// One snapshot scope contains a nil identity.
    InvalidSnapshotScope {
        /// Snapshot side with the invalid identity.
        side: VersionedGraphDiffReviewSide,
        /// Invalid exact scope.
        scope: VersionedContextScopeV1,
    },
    /// The two snapshots belong to different projects or Contexts.
    MismatchedContextScope {
        /// Baseline exact scope.
        source_scope: VersionedContextScopeV1,
        /// Revised exact scope.
        target_scope: VersionedContextScopeV1,
    },
    /// A review cannot compare one exact version with itself.
    IdenticalVersionScopes {
        /// Repeated exact scope.
        scope: VersionedContextScopeV1,
    },
    /// A serialized request claimed an unsupported schema version.
    UnsupportedSchemaVersion {
        /// Unsupported schema version.
        schema_version: VersionedGraphDiffReviewSchemaVersion,
    },
}

impl fmt::Display for VersionedContextGraphDiffReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSnapshotScope { side, .. } => {
                write!(
                    formatter,
                    "versioned graph diff review has an invalid {side:?} scope"
                )
            }
            Self::MismatchedContextScope { .. } => {
                formatter.write_str("versioned graph diff review scopes do not match")
            }
            Self::IdenticalVersionScopes { scope } => {
                write!(
                    formatter,
                    "versioned graph diff review repeats exact scope {scope:?}"
                )
            }
            Self::UnsupportedSchemaVersion { schema_version } => {
                write!(
                    formatter,
                    "unsupported versioned graph diff review schema: {schema_version:?}"
                )
            }
        }
    }
}

impl std::error::Error for VersionedContextGraphDiffReviewError {}

/// Projects two exact Context Graph snapshots through the sole `GraphDiff` calculator.
#[derive(Debug, Default, Clone, Copy)]
pub struct VersionedContextGraphDiffReviewService;

impl VersionedContextGraphDiffReviewService {
    /// Produces one complete projection without persistence or policy recalculation.
    #[must_use]
    pub fn project(
        request: VersionedContextGraphDiffReviewRequestV1,
    ) -> VersionedContextGraphDiffReviewProjectionV1 {
        let source_scope = request.source_scope();
        let target_scope = request.target_scope();
        VersionedContextGraphDiffReviewProjectionV1 {
            schema_version: VersionedGraphDiffReviewSchemaVersion::V1,
            identity_witness:
                VersionedContextGraphDiffReviewIdentityWitnessV1::from_validated_scopes(
                    source_scope,
                    target_scope,
                ),
            source_scope,
            target_scope,
            diff: GraphDiff::between(request.source().graph(), request.target().graph()),
        }
    }
}

fn validate_scope(
    side: VersionedGraphDiffReviewSide,
    scope: VersionedContextScopeV1,
) -> Result<(), VersionedContextGraphDiffReviewError> {
    if scope.project_id().as_uuid().is_nil()
        || scope.context_id().as_uuid().is_nil()
        || scope.commit_id().as_uuid().is_nil()
    {
        return Err(VersionedContextGraphDiffReviewError::InvalidSnapshotScope { side, scope });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlab_context_core::{ContextId, ProjectId};
    use contextlab_graph::{ContextGraph, GraphNode, GraphNodeKind};
    use contextlab_versioning::CommitId;
    use serde_json::json;
    use uuid::Uuid;

    fn scope(commit: u128) -> VersionedContextScopeV1 {
        VersionedContextScopeV1::new(
            ProjectId::from_uuid(Uuid::from_u128(1)),
            ContextId::from_uuid(Uuid::from_u128(2)),
            CommitId::from_uuid(Uuid::from_u128(commit)),
        )
    }

    fn snapshot(scope: VersionedContextScopeV1, label: &str) -> VersionedContextGraphSnapshotV1 {
        let mut graph = ContextGraph::new();
        graph
            .add_node(GraphNode::new("context:agent", GraphNodeKind::Context, label).expect("node"))
            .expect("graph node");
        VersionedContextGraphSnapshotV1::new(scope, graph)
    }

    #[test]
    fn projects_exact_scopes_through_graph_diff() {
        let request = VersionedContextGraphDiffReviewRequestV1::new(
            snapshot(scope(3), "v1"),
            snapshot(scope(4), "v2"),
        )
        .expect("request");

        let projection = VersionedContextGraphDiffReviewService::project(request);

        assert_eq!(projection.source_scope(), scope(3));
        assert_eq!(projection.target_scope(), scope(4));
        assert_eq!(projection.diff().modified_nodes().len(), 1);
        assert_eq!(
            projection.diff().modified_nodes()[0].node_id(),
            "context:agent"
        );
    }

    #[test]
    fn rejects_mismatched_or_identical_scopes_before_projection() {
        let mismatched = VersionedContextScopeV1::new(
            ProjectId::from_uuid(Uuid::from_u128(9)),
            ContextId::from_uuid(Uuid::from_u128(2)),
            CommitId::from_uuid(Uuid::from_u128(4)),
        );
        assert!(matches!(
            VersionedContextGraphDiffReviewRequestV1::new(
                snapshot(scope(3), "v1"),
                snapshot(mismatched, "v2"),
            ),
            Err(VersionedContextGraphDiffReviewError::MismatchedContextScope { .. })
        ));
        assert!(matches!(
            VersionedContextGraphDiffReviewRequestV1::new(
                snapshot(scope(3), "v1"),
                snapshot(scope(3), "v2"),
            ),
            Err(VersionedContextGraphDiffReviewError::IdenticalVersionScopes { .. })
        ));
    }

    #[test]
    fn rejects_nil_scope_and_unknown_wire_fields() {
        let nil_scope = VersionedContextScopeV1::new(
            ProjectId::from_uuid(Uuid::nil()),
            ContextId::from_uuid(Uuid::from_u128(2)),
            CommitId::from_uuid(Uuid::from_u128(3)),
        );
        assert!(matches!(
            VersionedContextGraphDiffReviewRequestV1::new(
                snapshot(nil_scope, "v1"),
                snapshot(scope(4), "v2"),
            ),
            Err(VersionedContextGraphDiffReviewError::InvalidSnapshotScope {
                side: VersionedGraphDiffReviewSide::Source,
                ..
            })
        ));

        let wire = json!({
            "schema_version": "v1",
            "source": {
                "scope": scope(3),
                "graph": {"nodes": [], "edges": []}
            },
            "target": {
                "scope": scope(4),
                "graph": {"nodes": [], "edges": []}
            },
            "unexpected": true
        });
        assert!(serde_json::from_value::<VersionedContextGraphDiffReviewRequestV1>(wire).is_err());
    }
}
