//! Static contract tests for immutable benchmark evidence migration assets.

use contextlab_storage::{
    BENCHMARK_DEFINITION_BINDING_MIGRATION, BENCHMARK_EVIDENCE_MIGRATION,
    CONTEXT_PLATFORM_MIGRATION,
};

#[test]
fn benchmark_definition_binding_migration_is_exact_scope_and_append_only() {
    for expected in [
        "CREATE TABLE benchmark_definition_bindings",
        "PRIMARY KEY (project_id, binding_id)",
        "UNIQUE (project_id, context_id, context_commit_id, suite_id)",
        "UNIQUE (\n        identity_source,\n        principal_id,\n        context_id,\n        branch_name,\n        idempotency_key",
        "FOREIGN KEY (project_id, context_id)",
        "REFERENCES contexts(project_id, id) ON DELETE RESTRICT",
        "FOREIGN KEY (context_id, context_commit_id)",
        "REFERENCES context_commits(context_id, id) ON DELETE RESTRICT",
        "FOREIGN KEY (project_id, suite_id)",
        "REFERENCES benchmark_suite_definitions(project_id, id) ON DELETE RESTRICT",
        "CHECK (schema_version = 1)",
        "CREATE TRIGGER benchmark_definition_bindings_append_only",
        "BEFORE UPDATE OR DELETE",
    ] {
        assert!(
            BENCHMARK_DEFINITION_BINDING_MIGRATION.contains(expected),
            "binding migration should contain {expected}"
        );
    }
    assert!(CONTEXT_PLATFORM_MIGRATION.contains(BENCHMARK_DEFINITION_BINDING_MIGRATION));
    assert!(!BENCHMARK_DEFINITION_BINDING_MIGRATION.contains("threshold_value"));
    assert!(!BENCHMARK_DEFINITION_BINDING_MIGRATION.contains("observed_value"));
}

#[test]
fn benchmark_evidence_migration_declares_project_scoped_immutable_contract() {
    for expected in [
        "CREATE TABLE benchmark_dataset_definitions",
        "CREATE TABLE benchmark_dataset_cases",
        "CREATE TABLE benchmark_suite_definitions",
        "CREATE TABLE benchmark_suite_datasets",
        "CREATE TABLE benchmark_suite_thresholds",
        "CREATE TABLE benchmark_evaluation_runs",
        "CREATE TABLE benchmark_run_measurements",
        "CREATE TABLE benchmark_decision_evidence",
        "CREATE TABLE benchmark_decision_runs",
        "CREATE TABLE benchmark_decision_metric_results",
        "FOREIGN KEY (project_id, context_id)",
        "FOREIGN KEY (context_id, context_commit_id)",
        "FOREIGN KEY (project_id, suite_id)",
        "CHECK (expected_mode IN ('unspecified', 'exact'))",
        "CHECK (direction IN ('minimum', 'maximum'))",
        "temperature >= 0.0",
        "temperature <= 2.0",
        "CHECK (status IN ('passed', 'regressed', 'insufficient_data'))",
        "CREATE FUNCTION prevent_benchmark_evidence_mutation()",
        "BEFORE UPDATE OR DELETE",
        "REFERENCES benchmark_suite_thresholds(project_id, suite_id, metric)",
        "idx_benchmark_decision_evidence_context_recorded_at",
        "DEFERRABLE INITIALLY DEFERRED",
        "validate_benchmark_decision_complete",
    ] {
        assert!(
            BENCHMARK_EVIDENCE_MIGRATION.contains(expected),
            "migration should contain {expected}"
        );
    }
}

#[test]
fn benchmark_evidence_migration_preserves_legacy_evaluation_runs() {
    assert!(!BENCHMARK_EVIDENCE_MIGRATION.contains("ALTER COLUMN suite_name"));
    assert!(!BENCHMARK_EVIDENCE_MIGRATION.contains("DELETE FROM evaluation_runs"));
    assert!(!BENCHMARK_EVIDENCE_MIGRATION.contains("UPDATE evaluation_runs"));
    assert!(BENCHMARK_EVIDENCE_MIGRATION.contains("benchmark_evaluation_runs"));
    assert!(CONTEXT_PLATFORM_MIGRATION.contains(BENCHMARK_EVIDENCE_MIGRATION));
    assert!(!BENCHMARK_EVIDENCE_MIGRATION.contains("INSERT INTO evaluation_runs"));
}

#[test]
fn benchmark_evidence_migration_rejects_child_inserts_after_aggregate_sealing() {
    for expected in [
        "CREATE TABLE benchmark_dataset_seals",
        "CREATE TABLE benchmark_suite_seals",
        "CREATE TABLE benchmark_run_seals",
        "CREATE TABLE benchmark_decision_seals",
        "CREATE FUNCTION prevent_benchmark_child_insert_after_seal()",
        "CREATE TRIGGER benchmark_dataset_cases_reject_after_dataset_seal BEFORE INSERT ON benchmark_dataset_cases FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();",
        "CREATE TRIGGER benchmark_suite_datasets_reject_after_suite_seal BEFORE INSERT ON benchmark_suite_datasets FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();",
        "CREATE TRIGGER benchmark_suite_thresholds_reject_after_suite_seal BEFORE INSERT ON benchmark_suite_thresholds FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();",
        "CREATE TRIGGER benchmark_run_measurements_reject_after_run_seal BEFORE INSERT ON benchmark_run_measurements FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();",
        "CREATE TRIGGER benchmark_decision_runs_reject_after_decision_seal BEFORE INSERT ON benchmark_decision_runs FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();",
        "CREATE TRIGGER benchmark_decision_metric_results_reject_after_decision_seal BEFORE INSERT ON benchmark_decision_metric_results FOR EACH ROW EXECUTE FUNCTION prevent_benchmark_child_insert_after_seal();",
    ] {
        assert!(
            BENCHMARK_EVIDENCE_MIGRATION.contains(expected),
            "migration should reject child inserts after aggregate sealing: {expected}"
        );
    }
}

#[test]
fn benchmark_evidence_migration_scopes_runs_and_decision_children_to_context_commits() {
    for expected in [
        "CREATE TABLE benchmark_evaluation_runs (\n    project_id UUID NOT NULL,\n    context_id UUID NOT NULL,\n    context_commit_id UUID NOT NULL,\n    run_id UUID NOT NULL,",
        "PRIMARY KEY (project_id, context_id, context_commit_id, run_id),",
        "PRIMARY KEY (project_id, context_id, context_commit_id, decision_id),",
        "PRIMARY KEY (project_id, context_id, context_commit_id, decision_id, run_id),",
        "UNIQUE (project_id, context_id, context_commit_id, decision_id, position),",
        "PRIMARY KEY (project_id, context_id, context_commit_id, decision_id, metric),",
        "CREATE TABLE benchmark_decision_seals (\n    project_id UUID NOT NULL,\n    context_id UUID NOT NULL,\n    context_commit_id UUID NOT NULL,\n    decision_id UUID NOT NULL,",
        "FOREIGN KEY (project_id, context_id) REFERENCES contexts(project_id, id) ON DELETE RESTRICT",
        "FOREIGN KEY (context_id, context_commit_id)\n        REFERENCES context_commits(context_id, id) ON DELETE RESTRICT",
        "CREATE TABLE benchmark_run_measurements (\n    project_id UUID NOT NULL,\n    context_id UUID NOT NULL,\n    context_commit_id UUID NOT NULL,\n    run_id UUID NOT NULL,",
        "FOREIGN KEY (project_id, context_id, context_commit_id, run_id)\n        REFERENCES benchmark_evaluation_runs(project_id, context_id, context_commit_id, run_id) ON DELETE RESTRICT",
        "CREATE TABLE benchmark_decision_runs (\n    project_id UUID NOT NULL,\n    context_id UUID NOT NULL,\n    context_commit_id UUID NOT NULL,\n    decision_id UUID NOT NULL,",
        "FOREIGN KEY (project_id, context_id, context_commit_id, decision_id)\n        REFERENCES benchmark_decision_evidence(project_id, context_id, context_commit_id, decision_id) ON DELETE RESTRICT",
        "FOREIGN KEY (project_id, context_id, context_commit_id, run_id)\n        REFERENCES benchmark_evaluation_runs(project_id, context_id, context_commit_id, run_id) ON DELETE RESTRICT",
        "CREATE TABLE benchmark_decision_metric_results (\n    project_id UUID NOT NULL,\n    context_id UUID NOT NULL,\n    context_commit_id UUID NOT NULL,\n    decision_id UUID NOT NULL,",
        "FOREIGN KEY (project_id, context_id, context_commit_id, suite_id, decision_id)\n        REFERENCES benchmark_decision_evidence(project_id, context_id, context_commit_id, suite_id, decision_id) ON DELETE RESTRICT",
    ] {
        assert!(
            BENCHMARK_EVIDENCE_MIGRATION.contains(expected),
            "migration should declare context-commit-scoped benchmark identity: {expected}"
        );
    }

    let normalized_migration = BENCHMARK_EVIDENCE_MIGRATION
        .lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        normalized_migration.contains(
            "WHEN 'benchmark_decision_runs', 'benchmark_decision_metric_results' THEN\nIF EXISTS (\nSELECT 1 FROM benchmark_decision_seals\nWHERE project_id = (payload ->> 'project_id')::UUID\nAND context_id = (payload ->> 'context_id')::UUID\nAND context_commit_id = (payload ->> 'context_commit_id')::UUID\nAND decision_id = (payload ->> 'decision_id')::UUID",
        ),
        "decision sealing must remain scoped to the exact Context commit"
    );
}

#[test]
fn benchmark_evidence_migration_keeps_policy_calculation_out_of_sql() {
    for forbidden in [
        "metric_result.outcome <> 'insufficient_data'",
        "computed_status",
    ] {
        assert!(
            !BENCHMARK_EVIDENCE_MIGRATION.contains(forbidden),
            "migration must not independently calculate evaluation policy: {forbidden}"
        );
    }
}
