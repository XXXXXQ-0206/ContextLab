//! Static contract tests for the private workflow execution status migration.

use contextlab_storage::{CONTEXT_PLATFORM_MIGRATION, WORKFLOW_EXECUTION_STATUS_MIGRATION};

const MIGRATION_ASSET: &str = include_str!("../migrations/0024_workflow_execution_status.sql");

#[test]
fn workflow_execution_status_migration_is_registered_as_the_v1_projection_asset() {
    assert_eq!(WORKFLOW_EXECUTION_STATUS_MIGRATION, MIGRATION_ASSET);
    assert!(CONTEXT_PLATFORM_MIGRATION.contains(WORKFLOW_EXECUTION_STATUS_MIGRATION));
    let normalized_migration = MIGRATION_ASSET
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    for required in [
        "CREATE TABLE workflow_execution_status_projections",
        "project_id UUID NOT NULL",
        "context_id UUID NOT NULL",
        "context_commit_id UUID NOT NULL",
        "run_id UUID NOT NULL",
        "schema_version TEXT COLLATE \"C\" NOT NULL",
        "schema_version = 'v1'",
        "projection JSONB NOT NULL CHECK ( jsonb_typeof(projection) = 'object'",
        "PRIMARY KEY (project_id, context_id, context_commit_id, run_id)",
        "UNIQUE (project_id, context_id, run_id)",
        "project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT",
        "FOREIGN KEY (project_id, context_id)",
        "REFERENCES contexts(project_id, id) ON DELETE RESTRICT",
        "FOREIGN KEY (context_id, context_commit_id)",
        "REFERENCES context_commits(context_id, id) ON DELETE RESTRICT",
        "CREATE INDEX idx_workflow_execution_status_projections_context_run",
        "CREATE FUNCTION prevent_workflow_execution_status_mutation()",
        "CREATE TRIGGER workflow_execution_status_projections_append_only",
        "BEFORE UPDATE OR DELETE",
    ] {
        assert!(
            normalized_migration.contains(required),
            "workflow execution status migration should contain {required}"
        );
    }
}

#[test]
fn workflow_execution_status_migration_does_not_store_raw_execution_or_failure_payloads() {
    for forbidden in [
        "CREATE TABLE workflow_execution_events",
        "CREATE TABLE workflow_execution_failures",
        "raw_events",
        "failure_payload",
        "failure_message",
        "provider_output",
        "GraphDiff::between",
        "context_diff_snapshots",
    ] {
        assert!(
            !MIGRATION_ASSET.contains(forbidden),
            "workflow execution status migration must not contain {forbidden}"
        );
    }
}

#[test]
fn composed_migrations_declare_context_scope_constraint_once() {
    assert_eq!(
        CONTEXT_PLATFORM_MIGRATION
            .matches("ADD CONSTRAINT uq_contexts_project_id_id UNIQUE (project_id, id)")
            .count(),
        1,
        "the context scope constraint must have one migration owner"
    );
}
