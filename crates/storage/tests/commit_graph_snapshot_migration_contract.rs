//! Static contract tests for ContextGraph snapshot migration invariants.

const CONTEXT_COMMIT_GRAPH_SNAPSHOTS_MIGRATION: &str =
    include_str!("../migrations/0002_context_commit_graph_snapshots.sql");
const CONTEXT_COMMIT_GRAPH_SNAPSHOT_INTEGRITY_MIGRATION: &str =
    include_str!("../migrations/0025_context_commit_graph_snapshot_integrity.sql");

#[test]
fn context_commit_graph_snapshot_migration_declares_existing_storage_invariants() {
    for required in [
        "CREATE TABLE context_commit_graph_snapshots",
        "commit_id UUID PRIMARY KEY REFERENCES context_commits(id) ON DELETE RESTRICT",
        "schema_version SMALLINT NOT NULL CHECK (schema_version > 0)",
        "graph JSONB NOT NULL CHECK (jsonb_typeof(graph) = 'object')",
        "captured_at TIMESTAMPTZ NOT NULL",
    ] {
        assert!(
            CONTEXT_COMMIT_GRAPH_SNAPSHOTS_MIGRATION.contains(required),
            "ContextGraph snapshot migration should contain {required}"
        );
    }
}

#[test]
fn context_commit_graph_snapshot_integrity_migration_enforces_v1_append_only_storage() {
    for required in [
        "ALTER TABLE context_commit_graph_snapshots",
        "CHECK (schema_version = 1)",
        "CREATE FUNCTION prevent_context_commit_graph_snapshot_mutation()",
        "RAISE EXCEPTION 'Context commit graph snapshots are append-only'",
        "CREATE TRIGGER context_commit_graph_snapshots_append_only",
        "BEFORE UPDATE OR DELETE ON context_commit_graph_snapshots",
    ] {
        assert!(
            CONTEXT_COMMIT_GRAPH_SNAPSHOT_INTEGRITY_MIGRATION.contains(required),
            "snapshot integrity migration should contain {required}"
        );
    }
}

#[test]
fn context_platform_migration_composes_snapshot_integrity_after_base_snapshot_schema() {
    let platform = contextlab_storage::CONTEXT_PLATFORM_MIGRATION;
    let base = platform
        .find("CREATE TABLE context_commit_graph_snapshots")
        .expect("base snapshot schema is composed");
    let integrity = platform
        .find("CREATE TRIGGER context_commit_graph_snapshots_append_only")
        .expect("snapshot integrity migration is composed");
    assert!(
        base < integrity,
        "snapshot integrity must be applied after the base snapshot table"
    );
}
