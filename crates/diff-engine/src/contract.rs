//! Stable V1 DTOs for pure ContextLab diffing.

use crate::{GraphDiff, TextDiff};
use contextlab_context_core::ContextMetadata;
use contextlab_graph::ContextGraph;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// The wire-compatible contract revision for this diff model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffContractVersion {
    /// The initial stable diff contract.
    V1,
}

/// A canonical identifier used to key deterministic diff records.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiffEntityId(String);

impl DiffEntityId {
    /// Creates a canonical, non-empty identifier.
    pub fn new(value: impl Into<String>) -> Result<Self, DiffInputError> {
        let value = value.into();
        validate_identifier("diff_entity.id", &value)?;
        Ok(Self(value))
    }

    /// Returns the canonical identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn validate(&self, field: &'static str) -> Result<(), DiffInputError> {
        validate_identifier(field, &self.0)
    }
}

impl fmt::Display for DiffEntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A named semantic document that can influence model behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticDocumentV1 {
    id: DiffEntityId,
    content: String,
}

impl SemanticDocumentV1 {
    /// Creates a document with an explicit stable identity.
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Result<Self, DiffInputError> {
        Ok(Self {
            id: DiffEntityId::new(id)?,
            content: content.into(),
        })
    }

    /// Returns the document identifier.
    #[must_use]
    pub const fn id(&self) -> &DiffEntityId {
        &self.id
    }

    /// Returns the exact document content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    fn validate(&self) -> Result<(), DiffInputError> {
        self.id.validate("semantic_document.id")
    }
}

/// A deterministic semantic snapshot containing a graph and keyed documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticSnapshotV1 {
    graph: ContextGraph,
    documents: BTreeMap<DiffEntityId, SemanticDocumentV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata: Option<ContextMetadata>,
}

impl SemanticSnapshotV1 {
    /// Creates a semantic snapshot and rejects duplicate document identities.
    pub fn new(
        graph: ContextGraph,
        documents: Vec<SemanticDocumentV1>,
    ) -> Result<Self, DiffInputError> {
        let mut indexed_documents = BTreeMap::new();
        for document in documents {
            document.validate()?;
            let document_id = document.id.clone();
            if indexed_documents
                .insert(document_id.clone(), document)
                .is_some()
            {
                return Err(DiffInputError::DuplicateSemanticDocument { document_id });
            }
        }

        let snapshot = Self {
            graph,
            documents: indexed_documents,
            metadata: None,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    /// Creates a semantic snapshot with exact Context metadata.
    pub fn new_with_metadata(
        graph: ContextGraph,
        documents: Vec<SemanticDocumentV1>,
        metadata: ContextMetadata,
    ) -> Result<Self, DiffInputError> {
        Self::new(graph, documents)?.with_metadata(metadata)
    }

    /// Attaches exact Context metadata to an existing semantic snapshot.
    pub fn with_metadata(mut self, metadata: ContextMetadata) -> Result<Self, DiffInputError> {
        metadata
            .validate()
            .map_err(|error| DiffInputError::InvalidContextMetadata {
                reason: error.to_string(),
            })?;
        self.metadata = Some(metadata);
        self.validate()?;
        Ok(self)
    }

    /// Returns the semantic graph snapshot.
    #[must_use]
    pub const fn graph(&self) -> &ContextGraph {
        &self.graph
    }

    /// Returns documents in stable identifier order.
    #[must_use]
    pub const fn documents(&self) -> &BTreeMap<DiffEntityId, SemanticDocumentV1> {
        &self.documents
    }

    /// Returns exact Context metadata when the snapshot was built from it.
    #[must_use]
    pub const fn metadata(&self) -> Option<&ContextMetadata> {
        self.metadata.as_ref()
    }

    fn validate(&self) -> Result<(), DiffInputError> {
        validate_graph(&self.graph)?;

        for (document_id, document) in &self.documents {
            document_id.validate("semantic_snapshot.documents.key")?;
            document.validate()?;
            if document_id != document.id() {
                return Err(DiffInputError::MismatchedCollectionKey {
                    collection: "semantic_snapshot.documents",
                    key: document_id.to_string(),
                    value_id: document.id().to_string(),
                });
            }
        }

        if let Some(metadata) = &self.metadata {
            metadata
                .validate()
                .map_err(|error| DiffInputError::InvalidContextMetadata {
                    reason: error.to_string(),
                })?;
        }

        Ok(())
    }
}

/// The observed outcome of one deterministic behavior case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum BehaviorOutcomeV1 {
    /// The case completed and produced an exact output payload.
    Succeeded {
        /// The observed output payload, which may be empty.
        output: String,
    },
    /// The case failed with a stable machine-readable code.
    Failed {
        /// The stable error code.
        error_code: DiffEntityId,
    },
}

impl BehaviorOutcomeV1 {
    /// Creates a successful behavior outcome.
    #[must_use]
    pub fn succeeded(output: impl Into<String>) -> Self {
        Self::Succeeded {
            output: output.into(),
        }
    }

    /// Creates a failed behavior outcome.
    pub fn failed(error_code: impl Into<String>) -> Result<Self, DiffInputError> {
        Ok(Self::Failed {
            error_code: DiffEntityId::new(error_code)?,
        })
    }

    fn validate(&self) -> Result<(), DiffInputError> {
        match self {
            Self::Succeeded { .. } => Ok(()),
            Self::Failed { error_code } => error_code.validate("behavior_outcome.error_code"),
        }
    }
}

/// One observed behavior case keyed by identity and its immutable input fingerprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorObservationV1 {
    case_id: DiffEntityId,
    input_fingerprint: DiffEntityId,
    outcome: BehaviorOutcomeV1,
}

impl BehaviorObservationV1 {
    /// Creates a behavior observation for one immutable input.
    pub fn new(
        case_id: impl Into<String>,
        input_fingerprint: impl Into<String>,
        outcome: BehaviorOutcomeV1,
    ) -> Result<Self, DiffInputError> {
        let observation = Self {
            case_id: DiffEntityId::new(case_id)?,
            input_fingerprint: DiffEntityId::new(input_fingerprint)?,
            outcome,
        };
        observation.validate()?;
        Ok(observation)
    }

    /// Returns the behavior case identifier.
    #[must_use]
    pub const fn case_id(&self) -> &DiffEntityId {
        &self.case_id
    }

    /// Returns the immutable input fingerprint.
    #[must_use]
    pub const fn input_fingerprint(&self) -> &DiffEntityId {
        &self.input_fingerprint
    }

    /// Returns the observed outcome.
    #[must_use]
    pub const fn outcome(&self) -> &BehaviorOutcomeV1 {
        &self.outcome
    }

    fn validate(&self) -> Result<(), DiffInputError> {
        self.case_id.validate("behavior_observation.case_id")?;
        self.input_fingerprint
            .validate("behavior_observation.input_fingerprint")?;
        self.outcome.validate()
    }
}

/// A deterministic set of behavior observations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorSnapshotV1 {
    cases: BTreeMap<DiffEntityId, BehaviorObservationV1>,
}

impl BehaviorSnapshotV1 {
    /// Creates a behavior snapshot and rejects duplicate case identities.
    pub fn new(cases: Vec<BehaviorObservationV1>) -> Result<Self, DiffInputError> {
        let mut indexed_cases = BTreeMap::new();
        for case in cases {
            case.validate()?;
            let case_id = case.case_id.clone();
            if indexed_cases.insert(case_id.clone(), case).is_some() {
                return Err(DiffInputError::DuplicateBehaviorCase { case_id });
            }
        }

        let snapshot = Self {
            cases: indexed_cases,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    /// Returns behavior cases in stable identifier order.
    #[must_use]
    pub const fn cases(&self) -> &BTreeMap<DiffEntityId, BehaviorObservationV1> {
        &self.cases
    }

    fn validate(&self) -> Result<(), DiffInputError> {
        for (case_id, case) in &self.cases {
            case_id.validate("behavior_snapshot.cases.key")?;
            case.validate()?;
            if case_id != case.case_id() {
                return Err(DiffInputError::MismatchedCollectionKey {
                    collection: "behavior_snapshot.cases",
                    key: case_id.to_string(),
                    value_id: case.case_id().to_string(),
                });
            }
        }

        Ok(())
    }
}

/// One finite aggregate evaluation metric observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationMetricObservationV1 {
    metric_id: DiffEntityId,
    value: f64,
    sample_count: usize,
}

impl EvaluationMetricObservationV1 {
    /// Creates a finite aggregate metric observation.
    pub fn new(
        metric_id: impl Into<String>,
        value: f64,
        sample_count: usize,
    ) -> Result<Self, DiffInputError> {
        let observation = Self {
            metric_id: DiffEntityId::new(metric_id)?,
            value,
            sample_count,
        };
        observation.validate()?;
        Ok(observation)
    }

    /// Returns the stable metric identifier.
    #[must_use]
    pub const fn metric_id(&self) -> &DiffEntityId {
        &self.metric_id
    }

    /// Returns the finite aggregate value.
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }

    /// Returns the number of contributing samples.
    #[must_use]
    pub const fn sample_count(&self) -> usize {
        self.sample_count
    }

    fn validate(&self) -> Result<(), DiffInputError> {
        self.metric_id.validate("evaluation_metric.metric_id")?;
        if !self.value.is_finite() {
            return Err(DiffInputError::NonFiniteEvaluationMetric {
                metric_id: self.metric_id.clone(),
            });
        }
        Ok(())
    }
}

/// A deterministic evaluation snapshot for one immutable evaluation configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationSnapshotV1 {
    comparability_fingerprint: DiffEntityId,
    metrics: BTreeMap<DiffEntityId, EvaluationMetricObservationV1>,
}

impl EvaluationSnapshotV1 {
    /// Creates an evaluation snapshot and rejects duplicate metric identities.
    pub fn new(
        comparability_fingerprint: impl Into<String>,
        metrics: Vec<EvaluationMetricObservationV1>,
    ) -> Result<Self, DiffInputError> {
        let mut indexed_metrics = BTreeMap::new();
        for metric in metrics {
            metric.validate()?;
            let metric_id = metric.metric_id.clone();
            if indexed_metrics.insert(metric_id.clone(), metric).is_some() {
                return Err(DiffInputError::DuplicateEvaluationMetric { metric_id });
            }
        }

        let snapshot = Self {
            comparability_fingerprint: DiffEntityId::new(comparability_fingerprint)?,
            metrics: indexed_metrics,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    /// Projects already-observed decision evidence without recalculating policy.
    pub fn from_decision_metrics<MetricId, Metrics>(
        comparability_fingerprint: impl Into<String>,
        metrics: Metrics,
    ) -> Result<Self, DiffInputError>
    where
        MetricId: Into<String>,
        Metrics: IntoIterator<Item = (MetricId, Option<f64>, usize)>,
    {
        let observations = metrics
            .into_iter()
            .map(|(metric_id, observed, sample_count)| {
                let metric_id = metric_id.into();
                let value =
                    observed.ok_or_else(|| DiffInputError::MissingEvaluationMetricValue {
                        metric_id: metric_id.clone(),
                    })?;
                EvaluationMetricObservationV1::new(metric_id, value, sample_count)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Self::new(comparability_fingerprint, observations)
    }

    /// Returns the immutable evaluation comparability fingerprint.
    #[must_use]
    pub const fn comparability_fingerprint(&self) -> &DiffEntityId {
        &self.comparability_fingerprint
    }

    /// Returns metrics in stable identifier order.
    #[must_use]
    pub const fn metrics(&self) -> &BTreeMap<DiffEntityId, EvaluationMetricObservationV1> {
        &self.metrics
    }

    fn validate(&self) -> Result<(), DiffInputError> {
        self.comparability_fingerprint
            .validate("evaluation_snapshot.comparability_fingerprint")?;

        for (metric_id, metric) in &self.metrics {
            metric_id.validate("evaluation_snapshot.metrics.key")?;
            metric.validate()?;
            if metric_id != metric.metric_id() {
                return Err(DiffInputError::MismatchedCollectionKey {
                    collection: "evaluation_snapshot.metrics",
                    key: metric_id.to_string(),
                    value_id: metric.metric_id().to_string(),
                });
            }
        }

        Ok(())
    }
}

/// All pure inputs needed to compare one Context revision with another.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextDiffSnapshotV1 {
    semantic: SemanticSnapshotV1,
    behavior: BehaviorSnapshotV1,
    evaluation: EvaluationSnapshotV1,
}

impl ContextDiffSnapshotV1 {
    /// Creates a complete, validated pure diff snapshot.
    pub fn new(
        semantic: SemanticSnapshotV1,
        behavior: BehaviorSnapshotV1,
        evaluation: EvaluationSnapshotV1,
    ) -> Result<Self, DiffInputError> {
        let snapshot = Self {
            semantic,
            behavior,
            evaluation,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    /// Returns semantic comparison inputs.
    #[must_use]
    pub const fn semantic(&self) -> &SemanticSnapshotV1 {
        &self.semantic
    }

    /// Returns behavior comparison inputs.
    #[must_use]
    pub const fn behavior(&self) -> &BehaviorSnapshotV1 {
        &self.behavior
    }

    /// Returns evaluation comparison inputs.
    #[must_use]
    pub const fn evaluation(&self) -> &EvaluationSnapshotV1 {
        &self.evaluation
    }

    pub(crate) fn validate(&self) -> Result<(), DiffInputError> {
        self.semantic.validate()?;
        self.behavior.validate()?;
        self.evaluation.validate()
    }
}

/// A V1 request to compare two complete pure Context snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextDiffRequestV1 {
    contract_version: DiffContractVersion,
    original: ContextDiffSnapshotV1,
    revised: ContextDiffSnapshotV1,
}

impl ContextDiffRequestV1 {
    /// Creates a request using the supported V1 contract revision.
    pub fn new(
        original: ContextDiffSnapshotV1,
        revised: ContextDiffSnapshotV1,
    ) -> Result<Self, DiffInputError> {
        original.validate()?;
        revised.validate()?;
        Ok(Self {
            contract_version: DiffContractVersion::V1,
            original,
            revised,
        })
    }

    /// Returns the request contract revision.
    #[must_use]
    pub const fn contract_version(&self) -> DiffContractVersion {
        self.contract_version
    }

    /// Returns the baseline snapshot.
    #[must_use]
    pub const fn original(&self) -> &ContextDiffSnapshotV1 {
        &self.original
    }

    /// Returns the revised snapshot.
    #[must_use]
    pub const fn revised(&self) -> &ContextDiffSnapshotV1 {
        &self.revised
    }
}

/// The source snapshot that failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffSnapshotSide {
    /// The baseline snapshot.
    Original,
    /// The revised snapshot.
    Revised,
}

/// An invalid condition within a pure diff input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffInputError {
    /// An identifier was empty or whitespace-only.
    EmptyIdentifier {
        /// The invalid field.
        field: &'static str,
    },
    /// An identifier was not in canonical trimmed form.
    NonCanonicalIdentifier {
        /// The invalid field.
        field: &'static str,
    },
    /// A semantic document identity was repeated.
    DuplicateSemanticDocument {
        /// The duplicate document identifier.
        document_id: DiffEntityId,
    },
    /// A behavior case identity was repeated.
    DuplicateBehaviorCase {
        /// The duplicate case identifier.
        case_id: DiffEntityId,
    },
    /// An evaluation metric identity was repeated.
    DuplicateEvaluationMetric {
        /// The duplicate metric identifier.
        metric_id: DiffEntityId,
    },
    /// A collection key and its record identity differed.
    MismatchedCollectionKey {
        /// The invalid collection.
        collection: &'static str,
        /// The map key.
        key: String,
        /// The record identity.
        value_id: String,
    },
    /// A graph node identifier was invalid.
    InvalidGraphNodeIdentifier {
        /// The invalid node identifier.
        node_id: String,
    },
    /// A graph node label was empty or not canonical.
    InvalidGraphNodeLabel {
        /// The invalid node identifier.
        node_id: String,
    },
    /// A graph edge referenced a node that was not present.
    GraphEdgeReferencesMissingNode {
        /// The edge source identifier.
        source: String,
        /// The edge target identifier.
        target: String,
    },
    /// A graph edge pointed to itself.
    SelfReferentialGraphEdge {
        /// The self-referenced node identifier.
        node_id: String,
    },
    /// An evaluation metric was NaN or infinite.
    NonFiniteEvaluationMetric {
        /// The metric identifier.
        metric_id: DiffEntityId,
    },
    /// An evaluation metric did not provide an observed aggregate value.
    MissingEvaluationMetricValue {
        /// The metric identifier.
        metric_id: String,
    },
    /// Context metadata failed its domain validation.
    InvalidContextMetadata {
        /// Stable validation detail.
        reason: String,
    },
}

impl fmt::Display for DiffInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid diff input: {self:?}")
    }
}

impl std::error::Error for DiffInputError {}

/// An error that prevents a complete, trustworthy diff result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextDiffError {
    /// One snapshot violated the V1 input contract.
    InvalidSnapshot {
        /// The invalid snapshot side.
        side: DiffSnapshotSide,
        /// The structural input error.
        source: DiffInputError,
    },
    /// A behavior case reused an identity for different inputs.
    BehaviorInputMismatch {
        /// The mismatched case identity.
        case_id: DiffEntityId,
        /// The baseline input fingerprint.
        original_input_fingerprint: DiffEntityId,
        /// The revised input fingerprint.
        revised_input_fingerprint: DiffEntityId,
    },
    /// The evaluation snapshots used different configurations or datasets.
    EvaluationComparabilityMismatch {
        /// The baseline comparability fingerprint.
        original_fingerprint: DiffEntityId,
        /// The revised comparability fingerprint.
        revised_fingerprint: DiffEntityId,
    },
}

impl fmt::Display for ContextDiffError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "context diff failed: {self:?}")
    }
}

impl std::error::Error for ContextDiffError {}

/// One deterministic semantic document transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum SemanticDocumentChangeV1 {
    /// A document exists only in the revised snapshot.
    Added {
        /// The revised document.
        document: SemanticDocumentV1,
    },
    /// A document exists only in the original snapshot.
    Removed {
        /// The original document.
        document: SemanticDocumentV1,
    },
    /// A document exists in both snapshots with changed text.
    Modified {
        /// The shared document identifier.
        document_id: DiffEntityId,
        /// The deterministic line diff.
        text_diff: TextDiff,
    },
}

impl SemanticDocumentChangeV1 {
    /// Returns the stable document identifier for this transition.
    #[must_use]
    pub fn document_id(&self) -> &DiffEntityId {
        match self {
            Self::Added { document } | Self::Removed { document } => document.id(),
            Self::Modified { document_id, .. } => document_id,
        }
    }
}

/// A Context metadata transition between two exact semantic snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum ContextMetadataChangeV1 {
    /// Metadata was introduced in the revised snapshot.
    Added {
        /// The revised metadata.
        revised: ContextMetadata,
    },
    /// Metadata was removed from the revised snapshot.
    Removed {
        /// The original metadata.
        original: ContextMetadata,
    },
    /// Metadata exists in both snapshots and changed.
    Modified {
        /// The original exact metadata.
        original: ContextMetadata,
        /// The revised exact metadata.
        revised: ContextMetadata,
    },
}

/// Semantic changes, including the canonical graph diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticDiffV1 {
    graph_diff: GraphDiff,
    document_changes: Vec<SemanticDocumentChangeV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata_change: Option<ContextMetadataChangeV1>,
}

impl SemanticDiffV1 {
    pub(crate) fn new(
        graph_diff: GraphDiff,
        document_changes: Vec<SemanticDocumentChangeV1>,
        metadata_change: Option<ContextMetadataChangeV1>,
    ) -> Self {
        Self {
            graph_diff,
            document_changes,
            metadata_change,
        }
    }

    /// Returns the graph diff calculated by GraphDiff between.
    #[must_use]
    pub const fn graph_diff(&self) -> &GraphDiff {
        &self.graph_diff
    }

    /// Returns document transitions in stable identifier order.
    #[must_use]
    pub fn document_changes(&self) -> &[SemanticDocumentChangeV1] {
        &self.document_changes
    }

    /// Returns the exact Context metadata transition, when one exists.
    #[must_use]
    pub const fn metadata_change(&self) -> Option<&ContextMetadataChangeV1> {
        self.metadata_change.as_ref()
    }

    /// Returns whether no semantic transition was found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.graph_diff.is_empty()
            && self.document_changes.is_empty()
            && self.metadata_change.is_none()
    }
}

/// One deterministic behavior-case transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum BehaviorCaseChangeV1 {
    /// A case exists only in the revised snapshot.
    Added {
        /// The revised observation.
        revised: BehaviorObservationV1,
    },
    /// A case exists only in the original snapshot.
    Removed {
        /// The original observation.
        original: BehaviorObservationV1,
    },
    /// A compatible case has a changed observed outcome.
    Modified {
        /// The original observation.
        original: BehaviorObservationV1,
        /// The revised observation.
        revised: BehaviorObservationV1,
    },
}

impl BehaviorCaseChangeV1 {
    /// Returns the stable behavior-case identifier.
    #[must_use]
    pub const fn case_id(&self) -> &DiffEntityId {
        match self {
            Self::Added { revised } | Self::Modified { revised, .. } => revised.case_id(),
            Self::Removed { original } => original.case_id(),
        }
    }
}

/// Behavior-case transitions in stable case order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehaviorDiffV1 {
    case_changes: Vec<BehaviorCaseChangeV1>,
}

impl BehaviorDiffV1 {
    pub(crate) fn new(case_changes: Vec<BehaviorCaseChangeV1>) -> Self {
        Self { case_changes }
    }

    /// Returns case transitions in stable case order.
    #[must_use]
    pub fn case_changes(&self) -> &[BehaviorCaseChangeV1] {
        &self.case_changes
    }

    /// Returns whether no behavior transition was found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.case_changes.is_empty()
    }
}

/// One deterministic evaluation-metric transition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum EvaluationMetricChangeV1 {
    /// A metric exists only in the revised snapshot.
    Added {
        /// The revised metric observation.
        revised: EvaluationMetricObservationV1,
    },
    /// A metric exists only in the original snapshot.
    Removed {
        /// The original metric observation.
        original: EvaluationMetricObservationV1,
    },
    /// A comparable metric observation changed.
    Modified {
        /// The original metric observation.
        original: EvaluationMetricObservationV1,
        /// The revised metric observation.
        revised: EvaluationMetricObservationV1,
    },
}

impl EvaluationMetricChangeV1 {
    /// Returns the stable metric identifier.
    #[must_use]
    pub const fn metric_id(&self) -> &DiffEntityId {
        match self {
            Self::Added { revised } | Self::Modified { revised, .. } => revised.metric_id(),
            Self::Removed { original } => original.metric_id(),
        }
    }
}

/// Evaluation-metric transitions in stable metric order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationDiffV1 {
    comparability_fingerprint: DiffEntityId,
    metric_changes: Vec<EvaluationMetricChangeV1>,
}

impl EvaluationDiffV1 {
    pub(crate) fn new(
        comparability_fingerprint: DiffEntityId,
        metric_changes: Vec<EvaluationMetricChangeV1>,
    ) -> Self {
        Self {
            comparability_fingerprint,
            metric_changes,
        }
    }

    /// Returns the shared evaluation comparability fingerprint.
    #[must_use]
    pub const fn comparability_fingerprint(&self) -> &DiffEntityId {
        &self.comparability_fingerprint
    }

    /// Returns metric transitions in stable metric order.
    #[must_use]
    pub fn metric_changes(&self) -> &[EvaluationMetricChangeV1] {
        &self.metric_changes
    }

    /// Returns whether no evaluation transition was found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.metric_changes.is_empty()
    }
}

/// The complete V1 result of comparing two pure Context snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextDiffResultV1 {
    contract_version: DiffContractVersion,
    semantic: SemanticDiffV1,
    behavior: BehaviorDiffV1,
    evaluation: EvaluationDiffV1,
}

impl ContextDiffResultV1 {
    pub(crate) fn new(
        semantic: SemanticDiffV1,
        behavior: BehaviorDiffV1,
        evaluation: EvaluationDiffV1,
    ) -> Self {
        Self {
            contract_version: DiffContractVersion::V1,
            semantic,
            behavior,
            evaluation,
        }
    }

    /// Returns the result contract revision.
    #[must_use]
    pub const fn contract_version(&self) -> DiffContractVersion {
        self.contract_version
    }

    /// Returns semantic changes.
    #[must_use]
    pub const fn semantic(&self) -> &SemanticDiffV1 {
        &self.semantic
    }

    /// Returns behavior changes.
    #[must_use]
    pub const fn behavior(&self) -> &BehaviorDiffV1 {
        &self.behavior
    }

    /// Returns evaluation changes.
    #[must_use]
    pub const fn evaluation(&self) -> &EvaluationDiffV1 {
        &self.evaluation
    }

    /// Returns whether every diff dimension is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.semantic.is_empty() && self.behavior.is_empty() && self.evaluation.is_empty()
    }
}

fn validate_identifier(field: &'static str, value: &str) -> Result<(), DiffInputError> {
    if value.trim().is_empty() {
        return Err(DiffInputError::EmptyIdentifier { field });
    }
    if value.trim() != value {
        return Err(DiffInputError::NonCanonicalIdentifier { field });
    }
    Ok(())
}

fn validate_graph(graph: &ContextGraph) -> Result<(), DiffInputError> {
    for node in graph.nodes().values() {
        if validate_identifier("semantic_graph.node.id", node.id().as_str()).is_err() {
            return Err(DiffInputError::InvalidGraphNodeIdentifier {
                node_id: node.id().as_str().to_owned(),
            });
        }
        if node.label().as_str().trim().is_empty()
            || node.label().as_str().trim() != node.label().as_str()
        {
            return Err(DiffInputError::InvalidGraphNodeLabel {
                node_id: node.id().as_str().to_owned(),
            });
        }
    }

    for edge in graph.edges() {
        if edge.source() == edge.target() {
            return Err(DiffInputError::SelfReferentialGraphEdge {
                node_id: edge.source().as_str().to_owned(),
            });
        }
        if !graph.nodes().contains_key(edge.source()) || !graph.nodes().contains_key(edge.target())
        {
            return Err(DiffInputError::GraphEdgeReferencesMissingNode {
                source: edge.source().as_str().to_owned(),
                target: edge.target().as_str().to_owned(),
            });
        }
    }

    Ok(())
}
