//! Static contract tests for PostgreSQL branch-head discovery.

const POSTGRES_ADAPTER: &str = include_str!("../src/postgres.rs");

#[test]
fn branch_head_list_query_is_context_scoped_and_deterministically_ordered() {
    assert!(
        POSTGRES_ADAPTER
            .contains("WHERE branches.context_id = $1\nORDER BY branches.branch_name ASC"),
        "branch-head listing must scope rows to the requested Context and order by branch name"
    );
}

#[test]
fn branch_head_queries_keep_cross_context_integrity_as_a_fail_closed_projection() {
    assert!(
        POSTGRES_ADAPTER.contains(
            "WHERE commits.context_id = branches.context_id\n                 AND commits.id = branches.head_commit_id"
        ),
        "branch-head queries must verify that a non-null head commit belongs to its Context"
    );
    assert!(
        POSTGRES_ADAPTER.contains("AS head_belongs_to_context"),
        "branch-head queries must project integrity state for typed fail-closed parsing"
    );
}
