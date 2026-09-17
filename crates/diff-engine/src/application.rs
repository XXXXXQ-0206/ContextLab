//! Pure application service for deterministic Context diffs.

use crate::{
    BehaviorCaseChangeV1, BehaviorDiffV1, ContextDiffError, ContextDiffRequestV1,
    ContextDiffResultV1, DiffSnapshotSide, EvaluationDiffV1, EvaluationMetricChangeV1, GraphDiff,
    SemanticDiffV1, SemanticDocumentChangeV1, TextDiff,
};
use std::collections::BTreeSet;

/// Compares complete V1 snapshots without persistence, model execution, or policy recalculation.
#[derive(Debug, Default, Clone, Copy)]
pub struct ContextDiffService;

impl ContextDiffService {
    /// Produces one complete V1 diff or returns a structured error with no partial result.
    pub fn compare(request: ContextDiffRequestV1) -> Result<ContextDiffResultV1, ContextDiffError> {
        request
            .original()
            .validate()
            .map_err(|source| ContextDiffError::InvalidSnapshot {
                side: DiffSnapshotSide::Original,
                source,
            })?;
        request
            .revised()
            .validate()
            .map_err(|source| ContextDiffError::InvalidSnapshot {
                side: DiffSnapshotSide::Revised,
                source,
            })?;

        let behavior =
            compare_behavior(request.original().behavior(), request.revised().behavior())?;
        let evaluation = compare_evaluation(
            request.original().evaluation(),
            request.revised().evaluation(),
        )?;
        let semantic =
            compare_semantic(request.original().semantic(), request.revised().semantic());

        Ok(ContextDiffResultV1::new(semantic, behavior, evaluation))
    }
}

fn compare_semantic(
    original: &crate::SemanticSnapshotV1,
    revised: &crate::SemanticSnapshotV1,
) -> SemanticDiffV1 {
    let document_ids = original
        .documents()
        .keys()
        .chain(revised.documents().keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    let document_changes = document_ids
        .into_iter()
        .filter_map(|document_id| {
            match (
                original.documents().get(&document_id),
                revised.documents().get(&document_id),
            ) {
                (None, Some(revised_document)) => Some(SemanticDocumentChangeV1::Added {
                    document: revised_document.clone(),
                }),
                (Some(original_document), None) => Some(SemanticDocumentChangeV1::Removed {
                    document: original_document.clone(),
                }),
                (Some(original_document), Some(revised_document))
                    if original_document.content() != revised_document.content() =>
                {
                    Some(SemanticDocumentChangeV1::Modified {
                        document_id,
                        text_diff: TextDiff::between(
                            original_document.content(),
                            revised_document.content(),
                        ),
                    })
                }
                (Some(_), Some(_)) | (None, None) => None,
            }
        })
        .collect();

    let metadata_change = match (original.metadata(), revised.metadata()) {
        (None, None) => None,
        (None, Some(revised)) => Some(crate::ContextMetadataChangeV1::Added {
            revised: revised.clone(),
        }),
        (Some(original), None) => Some(crate::ContextMetadataChangeV1::Removed {
            original: original.clone(),
        }),
        (Some(original), Some(revised)) if original != revised => {
            Some(crate::ContextMetadataChangeV1::Modified {
                original: original.clone(),
                revised: revised.clone(),
            })
        }
        (Some(_), Some(_)) => None,
    };

    SemanticDiffV1::new(
        GraphDiff::between(original.graph(), revised.graph()),
        document_changes,
        metadata_change,
    )
}

fn compare_behavior(
    original: &crate::BehaviorSnapshotV1,
    revised: &crate::BehaviorSnapshotV1,
) -> Result<BehaviorDiffV1, ContextDiffError> {
    let case_ids = original
        .cases()
        .keys()
        .chain(revised.cases().keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut case_changes = Vec::new();

    for case_id in case_ids {
        match (
            original.cases().get(&case_id),
            revised.cases().get(&case_id),
        ) {
            (None, Some(revised_case)) => case_changes.push(BehaviorCaseChangeV1::Added {
                revised: revised_case.clone(),
            }),
            (Some(original_case), None) => case_changes.push(BehaviorCaseChangeV1::Removed {
                original: original_case.clone(),
            }),
            (Some(original_case), Some(revised_case)) => {
                if original_case.input_fingerprint() != revised_case.input_fingerprint() {
                    return Err(ContextDiffError::BehaviorInputMismatch {
                        case_id,
                        original_input_fingerprint: original_case.input_fingerprint().clone(),
                        revised_input_fingerprint: revised_case.input_fingerprint().clone(),
                    });
                }
                if original_case.outcome() != revised_case.outcome() {
                    case_changes.push(BehaviorCaseChangeV1::Modified {
                        original: original_case.clone(),
                        revised: revised_case.clone(),
                    });
                }
            }
            (None, None) => {}
        }
    }

    Ok(BehaviorDiffV1::new(case_changes))
}

fn compare_evaluation(
    original: &crate::EvaluationSnapshotV1,
    revised: &crate::EvaluationSnapshotV1,
) -> Result<EvaluationDiffV1, ContextDiffError> {
    if original.comparability_fingerprint() != revised.comparability_fingerprint() {
        return Err(ContextDiffError::EvaluationComparabilityMismatch {
            original_fingerprint: original.comparability_fingerprint().clone(),
            revised_fingerprint: revised.comparability_fingerprint().clone(),
        });
    }

    let metric_ids = original
        .metrics()
        .keys()
        .chain(revised.metrics().keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let metric_changes = metric_ids
        .into_iter()
        .filter_map(|metric_id| {
            match (
                original.metrics().get(&metric_id),
                revised.metrics().get(&metric_id),
            ) {
                (None, Some(revised_metric)) => Some(EvaluationMetricChangeV1::Added {
                    revised: revised_metric.clone(),
                }),
                (Some(original_metric), None) => Some(EvaluationMetricChangeV1::Removed {
                    original: original_metric.clone(),
                }),
                (Some(original_metric), Some(revised_metric))
                    if original_metric != revised_metric =>
                {
                    Some(EvaluationMetricChangeV1::Modified {
                        original: original_metric.clone(),
                        revised: revised_metric.clone(),
                    })
                }
                (Some(_), Some(_)) | (None, None) => None,
            }
        })
        .collect();

    Ok(EvaluationDiffV1::new(
        original.comparability_fingerprint().clone(),
        metric_changes,
    ))
}
