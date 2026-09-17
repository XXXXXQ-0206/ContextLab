//! Static migration and adapter parity contracts for diff snapshots.

use contextlab_storage::{
    CONTEXT_DIFF_SNAPSHOT_MIGRATION, ContextDiffSnapshotV1Repository,
    PostgresContextGraphRepository,
};

#[test]
fn migration_declares_exact_scope_digest_and_append_only_constraints() {
    for required in [
        "CREATE TABLE context_diff_snapshots",
        "PRIMARY KEY (project_id, context_id, context_commit_id, schema_version)",
        "FOREIGN KEY (project_id, context_id)",
        "FOREIGN KEY (context_id, context_commit_id)",
        "schema_version = 'context-diff-snapshot-v1'",
        "snapshot JSONB NOT NULL CHECK (jsonb_typeof(snapshot) = 'object')",
        "snapshot_digest ~ '^sha256:[0-9a-f]{64}$'",
        "CREATE TRIGGER context_diff_snapshots_append_only",
        "BEFORE UPDATE OR DELETE ON context_diff_snapshots",
    ] {
        assert!(
            CONTEXT_DIFF_SNAPSHOT_MIGRATION.contains(required),
            "diff snapshot migration should contain {required}"
        );
    }
}

#[test]
fn existing_postgres_adapter_implements_the_same_repository_port() {
    fn assert_repository<T: ContextDiffSnapshotV1Repository>() {}

    assert_repository::<PostgresContextGraphRepository>();
}
