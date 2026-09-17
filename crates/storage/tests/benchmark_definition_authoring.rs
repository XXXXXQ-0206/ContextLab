//! Red-first tests for private benchmark definition authoring and Context binding.

use chrono::{TimeZone, Timelike, Utc};
use contextlab_auth::{AuthenticatedPrincipal, IdentitySourceId, PrincipalId, PrincipalIdentity};
use contextlab_context_core::{ContextId, ProjectId};
use contextlab_evaluation::{
    BenchmarkCase, BenchmarkDataset, BenchmarkExpectedOutput, BenchmarkSuite, MetricKind,
    RegressionThreshold, ThresholdDirection,
};
use contextlab_storage::{
    BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION, BenchmarkDefinitionBinding,
    BenchmarkDefinitionBindingCommand, BenchmarkDefinitionBindingError,
    BenchmarkDefinitionBindingId, BenchmarkDefinitionBindingRepository,
    BenchmarkDefinitionBindingSummary, BenchmarkDefinitionBindingWriteDisposition,
    BenchmarkDefinitionBindingWriter, ContextCommitRecord, ContextGraphProjection, ContextRecord,
    InMemoryContextGraphRepository, ProjectRecord, WorkspaceRecord,
};
use contextlab_versioning::{BranchName, CommitId};
use serde_json::json;
use uuid::Uuid;

fn principal() -> AuthenticatedPrincipal {
    AuthenticatedPrincipal::new(PrincipalIdentity::new(
        IdentitySourceId::new("https://issuer.contextlab.test").expect("identity source"),
        PrincipalId::new("user:author").expect("principal id"),
    ))
}

fn dataset(id: Uuid, case_id: Uuid, name: &str) -> BenchmarkDataset {
    BenchmarkDataset::with_id(
        contextlab_evaluation::BenchmarkDatasetId::from_uuid(id),
        name,
        vec![
            BenchmarkCase::with_id(
                contextlab_evaluation::BenchmarkCaseId::from_uuid(case_id),
                "case",
                json!({"input": "private"}),
                BenchmarkExpectedOutput::Unspecified,
            )
            .expect("valid case"),
        ],
    )
    .expect("valid dataset")
}

fn command(
    context_commit_id: CommitId,
    expected_head: CommitId,
    datasets: Vec<BenchmarkDataset>,
    suite: BenchmarkSuite,
) -> BenchmarkDefinitionBindingCommand {
    BenchmarkDefinitionBindingCommand::new(
        principal(),
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa),
        ProjectId::from_uuid(Uuid::from_u128(0x11111111111111111111111111111111)),
        ContextId::from_uuid(Uuid::from_u128(0x22222222222222222222222222222222)),
        context_commit_id,
        BranchName::new("main").expect("branch"),
        expected_head,
        "authoring-replay-1",
        "sha256:authoring-1",
        datasets,
        suite,
        Utc.with_ymd_and_hms(2026, 7, 27, 0, 0, 0)
            .single()
            .expect("timestamp")
            .with_nanosecond(123_456_789)
            .expect("nanosecond timestamp"),
        BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
    )
    .expect("valid authoring command")
}

fn rehydrate_binding(
    schema_version: u16,
    datasets: Vec<BenchmarkDataset>,
    suite: BenchmarkSuite,
) -> Result<BenchmarkDefinitionBinding, BenchmarkDefinitionBindingError> {
    BenchmarkDefinitionBinding::rehydrate(
        BenchmarkDefinitionBindingId::from_uuid(Uuid::from_u128(
            0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        )),
        ProjectId::from_uuid(Uuid::from_u128(0x11111111111111111111111111111111)),
        ContextId::from_uuid(Uuid::from_u128(0x22222222222222222222222222222222)),
        CommitId::from_uuid(Uuid::from_u128(0x33333333333333333333333333333333)),
        BranchName::new("main").expect("branch"),
        schema_version,
        datasets,
        suite,
        Utc.with_ymd_and_hms(2026, 7, 27, 0, 0, 0)
            .single()
            .expect("timestamp"),
    )
}

fn repository_and_scope() -> (
    InMemoryContextGraphRepository,
    ProjectId,
    ContextId,
    CommitId,
) {
    let project_id = ProjectId::from_uuid(Uuid::from_u128(0x11111111111111111111111111111111));
    let context_id = ContextId::from_uuid(Uuid::from_u128(0x22222222222222222222222222222222));
    let commit_id = CommitId::from_uuid(Uuid::from_u128(0x33333333333333333333333333333333));
    let timestamp = Utc
        .with_ymd_and_hms(2026, 7, 27, 0, 0, 0)
        .single()
        .expect("timestamp");
    let mut projection = ContextGraphProjection::default();
    projection.workspaces.push(WorkspaceRecord {
        id: "99999999-9999-4999-8999-999999999999".to_owned(),
        name: "Workspace".to_owned(),
        slug: "workspace".to_owned(),
        created_at: timestamp,
    });
    projection.projects.push(ProjectRecord {
        id: project_id.to_string(),
        workspace_id: "99999999-9999-4999-8999-999999999999".to_owned(),
        name: "Project".to_owned(),
        slug: "project".to_owned(),
        created_at: timestamp,
    });
    projection.contexts.push(ContextRecord {
        id: context_id.to_string(),
        project_id: project_id.to_string(),
        experiment_id: None,
        name: "Context".to_owned(),
        description: None,
        created_at: timestamp,
    });
    projection.commits.push(ContextCommitRecord {
        id: commit_id.to_string(),
        context_id: context_id.to_string(),
        branch_name: "main".to_owned(),
        message: "Initial context".to_owned(),
        parent_commit_ids: Vec::new(),
        changes: json!([]),
        change_count: 0,
        authored_at: timestamp,
        created_at: timestamp,
    });
    (
        InMemoryContextGraphRepository::new(projection),
        project_id,
        context_id,
        commit_id,
    )
}

#[test]
fn authoring_command_sorts_datasets_and_retains_exact_context_source() {
    let first_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1);
    let second_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2);
    let first = dataset(
        first_id,
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3),
        "first",
    );
    let second = dataset(
        second_id,
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4),
        "second",
    );
    let suite = BenchmarkSuite::new(
        "release",
        vec![
            contextlab_evaluation::BenchmarkDatasetId::from_uuid(second_id),
            contextlab_evaluation::BenchmarkDatasetId::from_uuid(first_id),
        ],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let commit_id = CommitId::from_uuid(Uuid::from_u128(0x33333333333333333333333333333333));

    let command = command(commit_id, commit_id, vec![second, first], suite);

    assert_eq!(command.context_commit_id(), commit_id);
    assert_eq!(command.expected_head(), commit_id);
    assert_eq!(
        command.schema_version(),
        BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION
    );
    assert_eq!(command.datasets()[0].id().as_uuid(), first_id);
    assert_eq!(command.datasets()[1].id().as_uuid(), second_id);
    assert_eq!(command.suite().dataset_ids(), command.dataset_ids());
}

#[test]
fn authoring_command_rejects_a_moved_expected_head_before_persistence() {
    let dataset_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1);
    let dataset = dataset(
        dataset_id,
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3),
        "only",
    );
    let suite = BenchmarkSuite::new(
        "release",
        vec![contextlab_evaluation::BenchmarkDatasetId::from_uuid(
            dataset_id,
        )],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let source = CommitId::from_uuid(Uuid::from_u128(0x33333333333333333333333333333333));
    let moved = CommitId::from_uuid(Uuid::from_u128(0x44444444444444444444444444444444));

    let error = BenchmarkDefinitionBindingCommand::try_new(
        principal(),
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa),
        ProjectId::from_uuid(Uuid::from_u128(0x11111111111111111111111111111111)),
        ContextId::from_uuid(Uuid::from_u128(0x22222222222222222222222222222222)),
        source,
        BranchName::new("main").expect("branch"),
        moved,
        "authoring-replay-2",
        "sha256:authoring-2",
        vec![dataset],
        suite,
        Utc.with_ymd_and_hms(2026, 7, 27, 0, 0, 0)
            .single()
            .expect("timestamp"),
        BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
    )
    .expect_err("a binding must use its expected branch head as its exact source");

    assert_eq!(
        error,
        BenchmarkDefinitionBindingError::ExpectedHeadDoesNotMatchSource {
            source_commit_id: source,
            expected: moved,
        }
    );
}

#[test]
fn postgres_rehydration_rejects_unknown_dataset_payload_version_before_payload_validation() {
    let dataset_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1);
    let suite = BenchmarkSuite::new(
        "release",
        vec![contextlab_evaluation::BenchmarkDatasetId::from_uuid(
            dataset_id,
        )],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let unknown_version = BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION + 1;

    let error = rehydrate_binding(unknown_version, Vec::new(), suite)
        .expect_err("unknown dataset payload versions must fail closed");

    assert_eq!(
        error,
        BenchmarkDefinitionBindingError::InvalidSchemaVersion {
            expected: BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
            actual: unknown_version,
        }
    );
}

#[test]
fn postgres_rehydration_rejects_unknown_suite_payload_version_before_payload_validation() {
    let dataset_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1);
    let other_dataset_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2);
    let dataset = dataset(
        dataset_id,
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3),
        "only",
    );
    let suite = BenchmarkSuite::new(
        "release",
        vec![contextlab_evaluation::BenchmarkDatasetId::from_uuid(
            other_dataset_id,
        )],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let unknown_version = BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION + 1;

    let error = rehydrate_binding(unknown_version, vec![dataset], suite)
        .expect_err("unknown suite payload versions must fail closed");

    assert_eq!(
        error,
        BenchmarkDefinitionBindingError::InvalidSchemaVersion {
            expected: BENCHMARK_DEFINITION_BINDING_SCHEMA_VERSION,
            actual: unknown_version,
        }
    );
}

#[tokio::test]
async fn in_memory_writer_persists_exact_binding_and_replays_identical_request() {
    let (repository, project_id, context_id, commit_id) = repository_and_scope();
    let dataset_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1);
    let dataset = dataset(
        dataset_id,
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3),
        "only",
    );
    let suite = BenchmarkSuite::new(
        "release",
        vec![contextlab_evaluation::BenchmarkDatasetId::from_uuid(
            dataset_id,
        )],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let request = command(commit_id, commit_id, vec![dataset], suite);

    let created = repository
        .persist_benchmark_definition_binding(request.clone())
        .await
        .expect("create binding");
    assert_eq!(
        created.disposition(),
        BenchmarkDefinitionBindingWriteDisposition::Created
    );
    let replayed = repository
        .persist_benchmark_definition_binding(request)
        .await
        .expect("replay binding");
    assert_eq!(
        replayed.disposition(),
        BenchmarkDefinitionBindingWriteDisposition::Replayed
    );
    assert_eq!(replayed.binding(), created.binding());
    assert_eq!(created.binding().captured_at().nanosecond(), 123_456_000);

    let exact = repository
        .get_benchmark_definition_binding(project_id, context_id, commit_id, created.binding().id())
        .await
        .expect("exact binding read")
        .expect("binding exists");
    assert_eq!(&exact, created.binding());
    let listed = repository
        .list_benchmark_definition_bindings_at_commit(project_id, context_id, commit_id)
        .await
        .expect("binding list");
    assert_eq!(listed, vec![exact]);
}

#[tokio::test]
async fn binding_summary_is_redacted_and_preserves_exact_scope_and_order() {
    let (repository, project_id, context_id, commit_id) = repository_and_scope();
    let first_dataset_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1);
    let second_dataset_id = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2);
    let first_dataset = dataset(
        first_dataset_id,
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa3),
        "first",
    );
    let second_dataset = dataset(
        second_dataset_id,
        Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa4),
        "second",
    );
    let suite = BenchmarkSuite::new(
        "release",
        vec![
            contextlab_evaluation::BenchmarkDatasetId::from_uuid(first_dataset_id),
            contextlab_evaluation::BenchmarkDatasetId::from_uuid(second_dataset_id),
        ],
        vec![
            RegressionThreshold::new(MetricKind::Accuracy, ThresholdDirection::Minimum, 0.9)
                .expect("threshold"),
        ],
    )
    .expect("suite");
    let binding = repository
        .persist_benchmark_definition_binding(command(
            commit_id,
            commit_id,
            vec![second_dataset, first_dataset],
            suite,
        ))
        .await
        .expect("create binding")
        .binding()
        .clone();

    let summary = BenchmarkDefinitionBindingSummary::from_binding(&binding);
    assert_eq!(summary.binding_id(), binding.id());
    assert_eq!(summary.project_id(), project_id);
    assert_eq!(summary.context_id(), context_id);
    assert_eq!(summary.context_commit_id(), commit_id);
    assert_eq!(summary.branch().as_str(), "main");
    assert_eq!(summary.suite_id(), binding.suite().id());
    assert_eq!(summary.suite_name(), "release");
    assert_eq!(summary.dataset_ids(), binding.dataset_ids());
    assert_eq!(summary.dataset_names(), &["first", "second"]);
    assert_eq!(summary.captured_at(), binding.captured_at());
}
