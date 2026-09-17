//! Integration tests for private benchmark definition and decision persistence.

use chrono::Utc;
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkDataset, BenchmarkDecisionComparisonError, BenchmarkEvaluation,
    BenchmarkExecutionReceipt, BenchmarkExpectedOutput, BenchmarkSuite, EvaluationError,
    EvaluationRun, MetricKind, MetricMeasurement, RegressionDecisionStatus, RegressionThreshold,
    ThresholdDirection,
};
use contextlab_storage::{
    BenchmarkDecisionComparisonScope, BenchmarkDecisionComparisonService,
    BenchmarkDecisionComparisonServiceError, BenchmarkDecisionDefinitionSummaryError,
    BenchmarkDecisionDefinitionSummaryService, BenchmarkDecisionDiscoveryRepository,
    BenchmarkDecisionDiscoveryService, BenchmarkDecisionDiscoveryServiceError, BenchmarkDecisionId,
    BenchmarkDecisionPair, BenchmarkDecisionRunDetailsError, BenchmarkDecisionRunDetailsService,
    BenchmarkEvidenceRepository, BenchmarkEvidenceWriteDisposition, BenchmarkEvidenceWriter,
    ContextCommitRecord, ContextGraphProjection, ContextRecord, EvaluationRunListQuery,
    EvaluationRunRepository, InMemoryContextGraphRepository, PersistBenchmarkEvaluationEvidence,
    ProjectRecord, StorageRepositoryError,
};
use contextlab_versioning::CommitId;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn provider_free_execution_receipt_reuses_existing_sealed_evidence_boundary() {
    let fixture = fixture();
    let plan = contextlab_evaluation::BenchmarkExecutionPlan::new(
        fixture.suite.clone(),
        vec![fixture.dataset.clone()],
    )
    .expect("execution plan");
    let receipt = BenchmarkExecutionReceipt::from_plan(
        &plan,
        fixture.decision_id.as_uuid(),
        fixture.context_id,
        "model-a",
        0.2,
        fixture.runs[0].executed_at(),
        "sha256:receipt-fixture",
        vec![
            contextlab_evaluation::BenchmarkCaseExecutionResult::new(
                fixture.dataset.id(),
                fixture.dataset.cases()[0].id(),
                vec![
                    MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("accuracy"),
                    MetricMeasurement::new(MetricKind::LatencyMs, 700.0).expect("latency"),
                ],
            )
            .expect("case result"),
        ],
    )
    .expect("provider-free receipt");
    let repository = fixture.repository();
    let persisted = repository
        .persist_benchmark_evaluation(
            PersistBenchmarkEvaluationEvidence::new(
                fixture.decision_id,
                fixture.project_id,
                fixture.commit_id,
                vec![fixture.dataset.clone()],
                fixture.suite.clone(),
                receipt.cohort().runs(),
                receipt.evaluation().clone(),
                "contextlab.exact-match",
                "receipt-fixture",
                receipt.cohort().entries()[0].run().executed_at(),
            )
            .expect("sealed evidence command"),
        )
        .await
        .expect("persist receipt evidence");

    assert_eq!(
        persisted.evidence().status(),
        receipt.decision_input().status()
    );
    for metric in persisted.evidence().metric_results() {
        let input = receipt
            .decision_input()
            .metrics()
            .get(&metric.metric())
            .expect("receipt metric");
        assert_eq!(metric.observed(), input.observed());
        assert_eq!(metric.sample_count(), input.sample_count());
        assert_eq!(
            metric.required_sample_count(),
            input.required_sample_count()
        );
        assert_eq!(
            metric.has_complete_coverage(),
            input.has_complete_coverage()
        );
        assert_eq!(metric.outcome(), input.outcome());
    }
}

#[tokio::test]
async fn in_memory_benchmark_evidence_persists_multi_run_cohort_and_replays_atomically() {
    let fixture = fixture();
    let repository = fixture.repository();
    let observer = repository.clone();
    let command = fixture.command("evaluator-v1");

    let created = repository
        .persist_benchmark_evaluation(command.clone())
        .await
        .expect("create evidence");
    let replayed = repository
        .persist_benchmark_evaluation(command)
        .await
        .expect("replay evidence");

    assert_eq!(
        created.disposition(),
        BenchmarkEvidenceWriteDisposition::Created
    );
    assert_eq!(
        replayed.disposition(),
        BenchmarkEvidenceWriteDisposition::Replayed
    );
    assert_eq!(created.evidence(), replayed.evidence());
    assert_eq!(created.evidence().run_ids().len(), 2);
    assert_eq!(
        created.evidence().status(),
        RegressionDecisionStatus::Passed
    );
    assert_eq!(
        created.evidence().metric_results()[0].required_sample_count(),
        2
    );
    assert!(created.evidence().evidence_digest().starts_with("sha256:"));
    assert!(
        created
            .evidence()
            .comparability()
            .fingerprint()
            .starts_with("sha256:")
    );
    assert_eq!(
        observer
            .get_benchmark_dataset(fixture.project_id, fixture.dataset.id())
            .await
            .expect("dataset read"),
        Some(fixture.dataset.clone())
    );
    assert_eq!(
        observer
            .get_benchmark_suite(fixture.project_id, fixture.suite.id())
            .await
            .expect("suite read"),
        Some(fixture.suite.clone())
    );
    assert_eq!(
        observer
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
            )
            .await
            .expect("decision read"),
        Some(created.evidence().clone())
    );
    for run in &fixture.runs {
        assert_eq!(
            observer
                .get_benchmark_run(
                    fixture.project_id,
                    fixture.context_id,
                    fixture.commit_id,
                    run.id(),
                )
                .await
                .expect("run read"),
            Some(run.clone())
        );
    }
    assert_eq!(
        observer
            .list_evaluation_runs(
                fixture.context_id.to_string(),
                EvaluationRunListQuery::default()
            )
            .await
            .expect("public preview list")
            .pagination
            .total,
        0,
        "private benchmark runs must not leak into the existing preview reader"
    );
}

#[tokio::test]
async fn in_memory_benchmark_decision_definition_summary_projects_only_safe_metadata() {
    let fixture = fixture();
    let repository = fixture.repository();
    let evidence = repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("persist benchmark evidence")
        .evidence()
        .clone();

    let summary = BenchmarkDecisionDefinitionSummaryService::new(&repository)
        .summarize(&evidence)
        .await
        .expect("summary storage access");

    assert_eq!(summary.suite().id(), fixture.suite.id());
    assert_eq!(summary.suite().name(), fixture.suite.name());
    assert_eq!(summary.suite().thresholds(), fixture.suite.thresholds());
    assert_eq!(summary.datasets().len(), 1);
    assert_eq!(summary.datasets()[0].id(), fixture.dataset.id());
    assert_eq!(summary.datasets()[0].name(), fixture.dataset.name());
    assert_eq!(
        summary.datasets()[0].case_count(),
        fixture.dataset.cases().len()
    );
}

#[tokio::test]
async fn in_memory_benchmark_decision_discovery_returns_sealed_redacted_summaries_in_order() {
    let fixture = fixture();
    let repository = fixture.repository();
    let older_decision_id = BenchmarkDecisionId::from_uuid(
        Uuid::parse_str("00000000-0000-4000-8000-000000000001").expect("decision uuid"),
    );
    let newer_decision_id = BenchmarkDecisionId::from_uuid(
        Uuid::parse_str("ffffffff-ffff-4fff-8fff-ffffffffffff").expect("decision uuid"),
    );

    repository
        .persist_benchmark_evaluation(
            fixture.command_with_recorded_at(
                older_decision_id,
                Utc::now() - chrono::Duration::minutes(1),
            ),
        )
        .await
        .expect("persist older sealed decision");
    repository
        .persist_benchmark_evaluation(
            fixture.command_with_recorded_at(newer_decision_id, Utc::now()),
        )
        .await
        .expect("persist newer sealed decision");

    let decisions = repository
        .list_benchmark_decisions(fixture.project_id, fixture.context_id, fixture.commit_id)
        .await
        .expect("list exact sealed decisions");
    assert_eq!(
        decisions
            .iter()
            .map(|decision| decision.decision_id())
            .collect::<Vec<_>>(),
        vec![newer_decision_id, older_decision_id]
    );

    let summaries = BenchmarkDecisionDiscoveryService::new(&repository)
        .summarize(fixture.project_id, fixture.context_id, fixture.commit_id)
        .await
        .expect("project redacted decision summaries");
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries[0].decision_id(), newer_decision_id);
    assert_eq!(summaries[0].suite().id(), fixture.suite.id());
    assert_eq!(summaries[0].suite().name(), fixture.suite.name());
    assert_eq!(summaries[0].datasets().len(), 1);
    assert_eq!(summaries[0].datasets()[0].id(), fixture.dataset.id());
    assert_eq!(summaries[0].datasets()[0].name(), fixture.dataset.name());
    assert_eq!(
        summaries[0].datasets()[0].case_count(),
        fixture.dataset.cases().len()
    );
    assert_eq!(summaries[0].run_count(), fixture.runs.len());
    assert_eq!(summaries[0].status(), RegressionDecisionStatus::Passed);
}

#[tokio::test]
async fn benchmark_decision_discovery_is_exactly_scoped_and_fails_closed() {
    let fixture = fixture();
    let repository = fixture.repository();
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("persist sealed decision");

    assert!(matches!(
        BenchmarkDecisionDiscoveryService::new(&repository)
            .summarize(ProjectId::new(), fixture.context_id, fixture.commit_id)
            .await,
        Err(BenchmarkDecisionDiscoveryServiceError::Storage(
            StorageRepositoryError::ScopeUnavailable { .. }
        ))
    ));
    assert!(matches!(
        BenchmarkDecisionDiscoveryService::new(&repository)
            .summarize(fixture.project_id, ContextId::new(), fixture.commit_id)
            .await,
        Err(BenchmarkDecisionDiscoveryServiceError::Storage(
            StorageRepositoryError::ScopeUnavailable { .. }
        ))
    ));
    assert!(
        BenchmarkDecisionDiscoveryService::new(&repository)
            .summarize(fixture.project_id, fixture.context_id, CommitId::new())
            .await
            .expect("wrong commit should remain a safe empty read")
            .is_empty()
    );
}

#[tokio::test]
async fn benchmark_decision_discovery_breaks_recorded_at_ties_by_decision_id() {
    let fixture = fixture();
    let repository = fixture.repository();
    let first_id = BenchmarkDecisionId::from_uuid(
        Uuid::parse_str("00000000-0000-4000-8000-000000000001").expect("decision uuid"),
    );
    let second_id = BenchmarkDecisionId::from_uuid(
        Uuid::parse_str("00000000-0000-4000-8000-000000000002").expect("decision uuid"),
    );
    let recorded_at = Utc::now();

    repository
        .persist_benchmark_evaluation(fixture.command_with_recorded_at(first_id, recorded_at))
        .await
        .expect("persist first tied decision");
    repository
        .persist_benchmark_evaluation(fixture.command_with_recorded_at(second_id, recorded_at))
        .await
        .expect("persist second tied decision");

    let decisions = repository
        .list_benchmark_decisions(fixture.project_id, fixture.context_id, fixture.commit_id)
        .await
        .expect("list tied decisions");
    assert_eq!(
        decisions
            .iter()
            .map(|decision| decision.decision_id())
            .collect::<Vec<_>>(),
        vec![first_id, second_id]
    );
}

#[tokio::test]
async fn benchmark_decision_definition_summary_orders_datasets_and_fails_closed() {
    let fixture = fixture();
    let second_dataset = BenchmarkDataset::new(
        "Secondary benchmark cases",
        vec![
            BenchmarkCase::new(
                "Follow-up request",
                json!({"secret_input": "do not expose"}),
                BenchmarkExpectedOutput::Exact(json!({"secret_expected": false})),
            )
            .expect("benchmark case"),
        ],
    )
    .expect("benchmark dataset");
    let suite = BenchmarkSuite::with_id(
        fixture.suite.id(),
        fixture.suite.name(),
        vec![second_dataset.id(), fixture.dataset.id()],
        fixture.suite.thresholds().to_vec(),
    )
    .expect("benchmark suite");
    let repository = fixture.repository();
    let evidence = repository
        .persist_benchmark_evaluation(
            PersistBenchmarkEvaluationEvidence::new(
                fixture.decision_id,
                fixture.project_id,
                fixture.commit_id,
                vec![fixture.dataset.clone(), second_dataset.clone()],
                suite,
                fixture.runs.clone(),
                BenchmarkEvaluation::from_runs(&fixture.suite, &fixture.runs),
                "evaluator-v1",
                "1.0.0",
                Utc::now(),
            )
            .expect("benchmark evidence command"),
        )
        .await
        .expect("persist benchmark evidence")
        .evidence()
        .clone();

    let summary = BenchmarkDecisionDefinitionSummaryService::new(&repository)
        .summarize(&evidence)
        .await
        .expect("summary storage access");
    assert_eq!(
        summary
            .datasets()
            .iter()
            .map(|dataset| dataset.id())
            .collect::<Vec<_>>(),
        {
            let mut dataset_ids = vec![fixture.dataset.id(), second_dataset.id()];
            dataset_ids.sort_unstable();
            dataset_ids
        }
    );

    let unavailable = DefinitionSummaryRepository {
        suite: None,
        datasets: Vec::new(),
    };
    assert!(matches!(
        BenchmarkDecisionDefinitionSummaryService::new(&unavailable)
            .summarize(&evidence)
            .await,
        Err(BenchmarkDecisionDefinitionSummaryError::DefinitionUnavailable)
    ));

    let mismatched_suite = BenchmarkSuite::with_id(
        evidence.suite_id(),
        "Mismatched membership",
        vec![fixture.dataset.id()],
        fixture.suite.thresholds().to_vec(),
    )
    .expect("mismatched suite fixture");
    let mismatched = DefinitionSummaryRepository {
        suite: Some(mismatched_suite),
        datasets: vec![fixture.dataset.clone()],
    };
    assert!(matches!(
        BenchmarkDecisionDefinitionSummaryService::new(&mismatched)
            .summarize(&evidence)
            .await,
        Err(BenchmarkDecisionDefinitionSummaryError::DefinitionMembershipMismatch)
    ));
}

#[tokio::test]
async fn benchmark_decision_run_details_preserve_sealed_order_and_fail_closed() {
    let fixture = fixture();
    let repository = fixture.repository();
    let evidence = repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("persist benchmark evidence")
        .evidence()
        .clone();

    let summary = BenchmarkDecisionRunDetailsService::new(&repository)
        .summarize(&evidence)
        .await
        .expect("run-detail storage access");

    assert_eq!(
        summary
            .runs()
            .iter()
            .map(|run| run.id())
            .collect::<Vec<_>>(),
        evidence.run_ids()
    );
    let sealed_first_run = fixture
        .runs
        .iter()
        .find(|run| run.id() == evidence.run_ids()[0])
        .expect("sealed run must originate from the fixture");
    assert_eq!(
        summary.runs()[0].model_version(),
        sealed_first_run.model_version()
    );
    assert_eq!(
        summary.runs()[0].temperature(),
        sealed_first_run.temperature()
    );
    assert_eq!(
        summary.runs()[0].executed_at(),
        sealed_first_run.executed_at()
    );
    assert_eq!(
        summary.runs()[0]
            .measurements()
            .iter()
            .map(|measurement| (measurement.metric(), measurement.value()))
            .collect::<Vec<_>>(),
        sealed_first_run
            .measurements()
            .iter()
            .map(|measurement| (measurement.kind(), measurement.value()))
            .collect::<Vec<_>>()
    );

    let unavailable = DefinitionSummaryRepository {
        suite: None,
        datasets: Vec::new(),
    };
    assert!(matches!(
        BenchmarkDecisionRunDetailsService::new(&unavailable)
            .summarize(&evidence)
            .await,
        Err(BenchmarkDecisionRunDetailsError::RunUnavailable)
    ));
}

struct DefinitionSummaryRepository {
    suite: Option<BenchmarkSuite>,
    datasets: Vec<BenchmarkDataset>,
}

#[async_trait::async_trait]
impl BenchmarkEvidenceRepository for DefinitionSummaryRepository {
    async fn get_benchmark_dataset(
        &self,
        _project_id: ProjectId,
        dataset_id: contextlab_evaluation::BenchmarkDatasetId,
    ) -> Result<Option<BenchmarkDataset>, StorageRepositoryError> {
        Ok(self
            .datasets
            .iter()
            .find(|dataset| dataset.id() == dataset_id)
            .cloned())
    }

    async fn get_benchmark_suite(
        &self,
        _project_id: ProjectId,
        suite_id: contextlab_evaluation::BenchmarkSuiteId,
    ) -> Result<Option<BenchmarkSuite>, StorageRepositoryError> {
        Ok(self
            .suite
            .as_ref()
            .filter(|suite| suite.id() == suite_id)
            .cloned())
    }

    async fn get_benchmark_run(
        &self,
        _project_id: ProjectId,
        _context_id: ContextId,
        _context_commit_id: CommitId,
        _run_id: contextlab_evaluation::EvaluationRunId,
    ) -> Result<Option<EvaluationRun>, StorageRepositoryError> {
        Ok(None)
    }

    async fn get_benchmark_decision(
        &self,
        _project_id: ProjectId,
        _context_id: ContextId,
        _context_commit_id: CommitId,
        _decision_id: BenchmarkDecisionId,
    ) -> Result<Option<contextlab_storage::BenchmarkDecisionEvidence>, StorageRepositoryError> {
        Ok(None)
    }

    async fn get_benchmark_decision_pair(
        &self,
        _project_id: ProjectId,
        _context_id: ContextId,
        _baseline: BenchmarkDecisionComparisonScope,
        _revised: BenchmarkDecisionComparisonScope,
    ) -> Result<BenchmarkDecisionPair, StorageRepositoryError> {
        Err(StorageRepositoryError::InMemoryStateUnavailable)
    }
}

#[tokio::test]
async fn in_memory_benchmark_evidence_rejects_same_decision_with_changed_payload() {
    let fixture = fixture();
    let repository = fixture.repository();
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("create evidence");

    let error = repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v2"))
        .await
        .expect_err("changed canonical payload must fail");

    assert!(matches!(
        error,
        StorageRepositoryError::BenchmarkEvidenceDigestConflict { .. }
    ));
}

#[tokio::test]
async fn in_memory_benchmark_evidence_replays_reconstructed_logical_payload() {
    let fixture = fixture();
    let repository = fixture.repository();
    let created = repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("create evidence");
    let replayed = repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("replay reconstructed evidence");

    assert_eq!(
        replayed.disposition(),
        BenchmarkEvidenceWriteDisposition::Replayed
    );
    assert_eq!(replayed.evidence(), created.evidence());
}

#[tokio::test]
async fn in_memory_benchmark_evidence_scopes_decisions_and_replay_to_exact_commit() {
    let fixture = fixture();
    let second_commit_id = CommitId::new();
    let repository = fixture.repository_with_commits(vec![fixture.commit_id, second_commit_id]);
    let first_created = repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("create first evidence");
    let second_runs = vec![
        evaluation_run(fixture.context_id, 0.94, 720.0),
        evaluation_run(fixture.context_id, 0.95, 730.0),
    ];
    let second_command = fixture.command_with_commit(
        fixture.decision_id,
        second_commit_id,
        vec![fixture.dataset.clone()],
        second_runs,
        "evaluator-v1",
    );
    let second_created = repository
        .persist_benchmark_evaluation(second_command.clone())
        .await
        .expect("create evidence for the second commit");
    let second_replayed = repository
        .persist_benchmark_evaluation(second_command)
        .await
        .expect("replay evidence for the second commit");

    assert_eq!(
        second_replayed.disposition(),
        BenchmarkEvidenceWriteDisposition::Replayed
    );
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                fixture.decision_id,
            )
            .await
            .expect("first decision read"),
        Some(first_created.evidence().clone())
    );
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                second_commit_id,
                fixture.decision_id,
            )
            .await
            .expect("second decision read"),
        Some(second_created.evidence().clone())
    );
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                CommitId::new(),
                fixture.decision_id,
            )
            .await
            .expect("wrong commit decision read"),
        None
    );
}

#[tokio::test]
async fn in_memory_benchmark_evidence_compares_two_exact_commit_scopes_without_re_evaluation() {
    let fixture = fixture();
    let revised_commit_id = CommitId::new();
    let repository = fixture.repository_with_commits(vec![fixture.commit_id, revised_commit_id]);
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("persist baseline evidence");
    let revised_runs = vec![
        evaluation_run(fixture.context_id, 0.84, 700.0),
        evaluation_run(fixture.context_id, 0.85, 710.0),
    ];
    repository
        .persist_benchmark_evaluation(fixture.command_with_commit(
            fixture.decision_id,
            revised_commit_id,
            vec![fixture.dataset.clone()],
            revised_runs,
            "evaluator-v1",
        ))
        .await
        .expect("persist revised evidence");

    let comparison = BenchmarkDecisionComparisonService::new(&repository)
        .compare(
            fixture.project_id,
            fixture.context_id,
            BenchmarkDecisionComparisonScope::new(fixture.commit_id, fixture.decision_id),
            BenchmarkDecisionComparisonScope::new(revised_commit_id, fixture.decision_id),
        )
        .await
        .expect("comparison storage access")
        .expect("both exact decisions exist");

    assert_eq!(
        comparison.status_change(),
        Some((
            RegressionDecisionStatus::Passed,
            RegressionDecisionStatus::Regressed
        ))
    );
    assert_eq!(comparison.metric_changes().len(), 1);
    assert_eq!(
        comparison.metric_changes()[0].metric(),
        MetricKind::Accuracy
    );
}

#[tokio::test]
async fn in_memory_benchmark_evidence_comparison_uses_exact_decision_scope() {
    let fixture = fixture();
    let repository = fixture.repository();
    let revised_decision_id = BenchmarkDecisionId::new();
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("persist baseline decision");
    let revised_runs = vec![
        evaluation_run(fixture.context_id, 0.84, 700.0),
        evaluation_run(fixture.context_id, 0.85, 710.0),
    ];
    repository
        .persist_benchmark_evaluation(fixture.command_with(
            revised_decision_id,
            vec![fixture.dataset.clone()],
            revised_runs,
            "evaluator-v1",
        ))
        .await
        .expect("persist revised decision in the same commit");

    let comparison = BenchmarkDecisionComparisonService::new(&repository)
        .compare(
            fixture.project_id,
            fixture.context_id,
            BenchmarkDecisionComparisonScope::new(fixture.commit_id, fixture.decision_id),
            BenchmarkDecisionComparisonScope::new(fixture.commit_id, revised_decision_id),
        )
        .await
        .expect("comparison storage access")
        .expect("both exact decisions exist");

    assert_eq!(
        comparison.status_change(),
        Some((
            RegressionDecisionStatus::Passed,
            RegressionDecisionStatus::Regressed,
        ))
    );
    assert_eq!(comparison.metric_changes().len(), 1);
    assert_eq!(
        comparison.metric_changes()[0].metric(),
        MetricKind::Accuracy
    );
    let exact_revised = repository
        .get_benchmark_decision(
            fixture.project_id,
            fixture.context_id,
            fixture.commit_id,
            revised_decision_id,
        )
        .await
        .expect("read exact revised decision")
        .expect("revised decision exists");
    assert_eq!(exact_revised.decision_id(), revised_decision_id);
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                BenchmarkDecisionId::new(),
            )
            .await
            .expect("read unknown decision"),
        None
    );
}

#[tokio::test]
async fn in_memory_benchmark_evidence_comparison_returns_none_when_one_exact_scope_is_missing() {
    let fixture = fixture();
    let missing_commit_id = CommitId::new();
    let repository = fixture.repository_with_commits(vec![fixture.commit_id, missing_commit_id]);
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("persist baseline evidence");

    let comparison = BenchmarkDecisionComparisonService::new(&repository)
        .compare(
            fixture.project_id,
            fixture.context_id,
            BenchmarkDecisionComparisonScope::new(fixture.commit_id, fixture.decision_id),
            BenchmarkDecisionComparisonScope::new(missing_commit_id, fixture.decision_id),
        )
        .await
        .expect("comparison storage access");

    assert_eq!(comparison, None);
}

#[tokio::test]
async fn in_memory_benchmark_evidence_comparison_fails_closed_for_different_fingerprints() {
    let fixture = fixture();
    let revised_commit_id = CommitId::new();
    let repository = fixture.repository_with_commits(vec![fixture.commit_id, revised_commit_id]);
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("persist baseline evidence");
    repository
        .persist_benchmark_evaluation(fixture.command_with_commit(
            fixture.decision_id,
            revised_commit_id,
            vec![fixture.dataset.clone()],
            vec![
                evaluation_run(fixture.context_id, 0.95, 700.0),
                evaluation_run(fixture.context_id, 0.96, 710.0),
            ],
            "evaluator-v2",
        ))
        .await
        .expect("persist non-comparable revised evidence");

    let error = BenchmarkDecisionComparisonService::new(&repository)
        .compare(
            fixture.project_id,
            fixture.context_id,
            BenchmarkDecisionComparisonScope::new(fixture.commit_id, fixture.decision_id),
            BenchmarkDecisionComparisonScope::new(revised_commit_id, fixture.decision_id),
        )
        .await
        .expect_err("different immutable comparability fingerprints must not be compared");

    assert!(matches!(
        error,
        BenchmarkDecisionComparisonServiceError::Comparison(
            BenchmarkDecisionComparisonError::ComparabilityMismatch { .. }
        )
    ));
}

#[tokio::test]
async fn in_memory_benchmark_evidence_rejects_definition_conflict_without_partial_write() {
    let fixture = fixture();
    let repository = fixture.repository();
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("create evidence");
    let conflicting_dataset = BenchmarkDataset::with_id(
        fixture.dataset.id(),
        "Changed dataset",
        fixture.dataset.cases().to_vec(),
    )
    .expect("conflicting dataset");
    let next_decision_id = BenchmarkDecisionId::new();
    let command = fixture.command_with(
        next_decision_id,
        vec![conflicting_dataset],
        fixture.runs.clone(),
        "evaluator-v1",
    );

    let error = repository
        .persist_benchmark_evaluation(command)
        .await
        .expect_err("definition conflict must fail");

    assert!(matches!(
        error,
        StorageRepositoryError::BenchmarkDefinitionConflict { .. }
    ));
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                fixture.commit_id,
                next_decision_id,
            )
            .await
            .expect("decision read"),
        None
    );
}

#[tokio::test]
async fn in_memory_benchmark_evidence_rejects_run_id_reuse_across_commits_without_decision() {
    let fixture = fixture();
    let second_commit_id = CommitId::new();
    let repository = fixture.repository_with_commits(vec![fixture.commit_id, second_commit_id]);
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("create first evidence");

    let decision_id = BenchmarkDecisionId::new();
    let error = repository
        .persist_benchmark_evaluation(fixture.command_with_commit(
            decision_id,
            second_commit_id,
            vec![fixture.dataset.clone()],
            fixture.runs.clone(),
            "evaluator-v1",
        ))
        .await
        .expect_err("run id reused for another commit must fail");

    assert!(matches!(
        error,
        StorageRepositoryError::BenchmarkDefinitionConflict {
            definition_kind: "evaluation_run",
            ..
        }
    ));
    assert_eq!(
        repository
            .get_benchmark_decision(
                fixture.project_id,
                fixture.context_id,
                second_commit_id,
                decision_id,
            )
            .await
            .expect("decision read"),
        None
    );
}

#[tokio::test]
async fn in_memory_benchmark_evidence_scopes_private_reads_by_project_and_commit() {
    let fixture = fixture();
    let repository = fixture.repository();
    repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect("create evidence");

    let decision_error = repository
        .get_benchmark_decision(
            ProjectId::new(),
            fixture.context_id,
            fixture.commit_id,
            fixture.decision_id,
        )
        .await
        .expect_err("wrong project must not read a decision");
    assert!(matches!(
        decision_error,
        StorageRepositoryError::ScopeUnavailable { .. }
    ));
    assert_eq!(
        repository
            .get_benchmark_run(
                fixture.project_id,
                fixture.context_id,
                CommitId::new(),
                fixture.runs[0].id(),
            )
            .await
            .expect("wrong commit read"),
        None
    );
}

#[tokio::test]
async fn in_memory_benchmark_evidence_rejects_scope_mismatch_without_definitions() {
    let fixture = fixture();
    let repository = fixture.project_only_repository();

    let error = repository
        .persist_benchmark_evaluation(fixture.command("evaluator-v1"))
        .await
        .expect_err("missing context must fail");

    assert!(matches!(
        error,
        StorageRepositoryError::ScopeUnavailable { .. }
    ));
    assert_eq!(
        repository
            .get_benchmark_dataset(fixture.project_id, fixture.dataset.id())
            .await
            .expect("dataset read"),
        None
    );
}

#[test]
fn benchmark_evidence_command_requires_membership_and_homogeneous_runs() {
    let fixture = fixture();
    let evaluation = BenchmarkEvaluation::from_runs(&fixture.suite, &fixture.runs);
    assert!(
        PersistBenchmarkEvaluationEvidence::new(
            fixture.decision_id,
            fixture.project_id,
            fixture.commit_id,
            Vec::new(),
            fixture.suite.clone(),
            fixture.runs.clone(),
            evaluation,
            "evaluator",
            "v1",
            Utc::now(),
        )
        .is_err()
    );

    let mut mismatched_runs = fixture.runs.clone();
    mismatched_runs.push(
        EvaluationRun::new(
            fixture.context_id,
            "model-b",
            0.2,
            vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("metric")],
            Utc::now(),
        )
        .expect("valid mismatched run"),
    );
    let mismatched_evaluation = BenchmarkEvaluation::from_runs(&fixture.suite, &mismatched_runs);
    assert!(
        PersistBenchmarkEvaluationEvidence::new(
            BenchmarkDecisionId::new(),
            fixture.project_id,
            fixture.commit_id,
            vec![fixture.dataset],
            fixture.suite,
            mismatched_runs,
            mismatched_evaluation,
            "evaluator",
            "v1",
            Utc::now(),
        )
        .is_err()
    );
}

#[test]
fn benchmark_evidence_rejects_invalid_runs_before_storage_construction() {
    let fixture = fixture();
    let error = EvaluationRun::new(
        fixture.context_id,
        "   ",
        f32::NAN,
        vec![MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("metric")],
        Utc::now(),
    )
    .expect_err("invalid run configuration must fail before adapter persistence");

    assert_eq!(
        error,
        EvaluationError::EmptyModelVersion,
        "storage cannot receive an evaluation run without a model identity"
    );
}

#[test]
fn benchmark_evidence_command_rejects_a_domain_artifact_for_changed_run_payload() {
    let fixture = fixture();
    let evaluation = BenchmarkEvaluation::from_runs(&fixture.suite, &fixture.runs);
    let original = &fixture.runs[0];
    let changed = EvaluationRun::from_persisted(
        original.id(),
        original.context_id(),
        original.model_version(),
        original.temperature(),
        vec![
            MetricMeasurement::new(MetricKind::Accuracy, 0.5).expect("changed accuracy"),
            MetricMeasurement::new(MetricKind::LatencyMs, 700.0).expect("latency"),
        ],
        original.executed_at(),
    )
    .expect("changed run remains individually valid");
    let mut changed_runs = fixture.runs.clone();
    changed_runs[0] = changed;

    assert!(matches!(
        PersistBenchmarkEvaluationEvidence::new(
            BenchmarkDecisionId::new(),
            fixture.project_id,
            fixture.commit_id,
            vec![fixture.dataset],
            fixture.suite,
            changed_runs,
            evaluation,
            "evaluator",
            "v1",
            Utc::now(),
        ),
        Err(contextlab_storage::BenchmarkEvidenceError::DomainEvaluationMismatch)
    ));
}

#[tokio::test]
async fn benchmark_evidence_preserves_duplicate_metric_as_fail_closed_coverage() {
    let fixture = fixture();
    let repository = fixture.repository();
    let duplicate_metric_run = EvaluationRun::new(
        fixture.context_id,
        "model-a",
        0.2,
        vec![
            MetricMeasurement::new(MetricKind::Accuracy, 0.95).expect("first accuracy"),
            MetricMeasurement::new(MetricKind::Accuracy, 0.96).expect("duplicate accuracy"),
            MetricMeasurement::new(MetricKind::LatencyMs, 700.0).expect("latency"),
        ],
        Utc::now(),
    )
    .expect("valid duplicate-metric run");

    let command = fixture.command_with(
        BenchmarkDecisionId::new(),
        vec![fixture.dataset.clone()],
        vec![duplicate_metric_run],
        "evaluator-v1",
    );
    let created = repository
        .persist_benchmark_evaluation(command)
        .await
        .expect("persist fail-closed evidence");
    let accuracy = created
        .evidence()
        .metric_results()
        .iter()
        .find(|result| result.metric() == MetricKind::Accuracy)
        .expect("accuracy evidence");

    assert_eq!(accuracy.sample_count(), 1);
    assert_eq!(accuracy.required_sample_count(), 1);
    assert!(!accuracy.has_complete_coverage());
    assert_eq!(
        accuracy.outcome(),
        contextlab_evaluation::RegressionCheckStatus::InsufficientData
    );
}

struct Fixture {
    project_id: ProjectId,
    context_id: ContextId,
    commit_id: CommitId,
    decision_id: BenchmarkDecisionId,
    dataset: BenchmarkDataset,
    suite: BenchmarkSuite,
    runs: Vec<EvaluationRun>,
}

impl Fixture {
    fn project_only_repository(&self) -> InMemoryContextGraphRepository {
        InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![self.project_record()],
            ..ContextGraphProjection::default()
        })
    }

    fn repository(&self) -> InMemoryContextGraphRepository {
        self.repository_with_commits(vec![self.commit_id])
    }

    fn repository_with_commits(&self, commit_ids: Vec<CommitId>) -> InMemoryContextGraphRepository {
        InMemoryContextGraphRepository::new(ContextGraphProjection {
            projects: vec![self.project_record()],
            contexts: vec![ContextRecord {
                id: self.context_id.to_string(),
                project_id: self.project_id.to_string(),
                experiment_id: None,
                name: "Benchmark context".to_owned(),
                description: None,
                created_at: Utc::now(),
            }],
            commits: commit_ids
                .into_iter()
                .map(|commit_id| ContextCommitRecord {
                    id: commit_id.to_string(),
                    context_id: self.context_id.to_string(),
                    branch_name: "main".to_owned(),
                    message: "Benchmark candidate".to_owned(),
                    parent_commit_ids: Vec::new(),
                    changes: json!([]),
                    change_count: 0,
                    authored_at: Utc::now(),
                    created_at: Utc::now(),
                })
                .collect(),
            ..ContextGraphProjection::default()
        })
    }

    fn project_record(&self) -> ProjectRecord {
        ProjectRecord {
            id: self.project_id.to_string(),
            workspace_id: Uuid::new_v4().to_string(),
            name: "Benchmark project".to_owned(),
            slug: "benchmark-project".to_owned(),
            created_at: Utc::now(),
        }
    }

    fn command(&self, evaluator_version: &str) -> PersistBenchmarkEvaluationEvidence {
        self.command_with(
            self.decision_id,
            vec![self.dataset.clone()],
            self.runs.clone(),
            evaluator_version,
        )
    }

    fn command_with_recorded_at(
        &self,
        decision_id: BenchmarkDecisionId,
        recorded_at: chrono::DateTime<Utc>,
    ) -> PersistBenchmarkEvaluationEvidence {
        let evaluation = BenchmarkEvaluation::from_runs(&self.suite, &self.runs);
        PersistBenchmarkEvaluationEvidence::new(
            decision_id,
            self.project_id,
            self.commit_id,
            vec![self.dataset.clone()],
            self.suite.clone(),
            self.runs.clone(),
            evaluation,
            "contextlab.exact-match",
            "evaluator-v1",
            recorded_at,
        )
        .expect("command")
    }

    fn command_with(
        &self,
        decision_id: BenchmarkDecisionId,
        datasets: Vec<BenchmarkDataset>,
        runs: Vec<EvaluationRun>,
        evaluator_version: &str,
    ) -> PersistBenchmarkEvaluationEvidence {
        self.command_with_commit(
            decision_id,
            self.commit_id,
            datasets,
            runs,
            evaluator_version,
        )
    }

    fn command_with_commit(
        &self,
        decision_id: BenchmarkDecisionId,
        commit_id: CommitId,
        datasets: Vec<BenchmarkDataset>,
        runs: Vec<EvaluationRun>,
        evaluator_version: &str,
    ) -> PersistBenchmarkEvaluationEvidence {
        let evaluation = BenchmarkEvaluation::from_runs(&self.suite, &runs);
        PersistBenchmarkEvaluationEvidence::new(
            decision_id,
            self.project_id,
            commit_id,
            datasets,
            self.suite.clone(),
            runs,
            evaluation,
            "contextlab.exact-match",
            evaluator_version,
            Utc::now(),
        )
        .expect("command")
    }
}

fn fixture() -> Fixture {
    let project_id = ProjectId::new();
    let context_id = ContextId::new();
    let dataset = BenchmarkDataset::new(
        "Support cases",
        vec![
            BenchmarkCase::new(
                "Refund request",
                json!({"question": "Can I return this?"}),
                BenchmarkExpectedOutput::Exact(json!({"eligible": true})),
            )
            .expect("case"),
        ],
    )
    .expect("dataset");
    let suite = BenchmarkSuite::new(
        "Release gate",
        vec![dataset.id()],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
            RegressionThreshold::new(MetricKind::LatencyMs, ThresholdDirection::Maximum, 800.0)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let runs = vec![
        evaluation_run(context_id, 0.95, 700.0),
        evaluation_run(context_id, 0.96, 710.0),
    ];

    Fixture {
        project_id,
        context_id,
        commit_id: CommitId::new(),
        decision_id: BenchmarkDecisionId::new(),
        dataset,
        suite,
        runs,
    }
}

fn evaluation_run(context_id: ContextId, accuracy: f64, latency: f64) -> EvaluationRun {
    EvaluationRun::new(
        context_id,
        "model-a",
        0.2,
        vec![
            MetricMeasurement::new(MetricKind::Accuracy, accuracy).expect("metric"),
            MetricMeasurement::new(MetricKind::LatencyMs, latency).expect("metric"),
        ],
        Utc::now(),
    )
    .expect("valid evaluation run")
}
